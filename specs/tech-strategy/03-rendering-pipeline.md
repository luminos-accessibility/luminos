# 03 -- Rendering Pipeline

**Status:** DRAFT v1.1 (post audit review)
**Date:** 2026-03-15
**Audience:** Engineers, AI agents implementing the magnification rendering pipeline
**Source Documents:** [Product Strategy](../PRODUCT_STRATEGY.md) (v1.3, Sections 5, 7, 8), [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) (FINAL, Sections 4.2, 4.3, 4.8), [System Architecture](./01-system-architecture.md) (Sections 4.4, 5.1, 9), [Platform Abstraction](./02-platform-abstraction.md) (ScreenCapture, WindowManager)

---

## 1. Overview

### 1.1 Purpose

This document defines the GPU-accelerated rendering pipeline that transforms captured screen pixels into the magnified view displayed on the user's screen. It is the engineering specification for the performance-critical hot path -- the only code in Luminos that must execute within a 16.67ms frame budget on integrated GPUs.

This document answers: **How does captured screen content become magnified pixels on the overlay window, at 60fps, across all zoom modes?**

### 1.2 Scope

This document covers:
- Pipeline stages from viewport calculation through GPU present
- GPU texture management (upload, double buffering, format conversion)
- WGSL shader design (interpolation, color filters, cursor overlay)
- Zoom mode rendering (full-screen, lens, docked)
- Frame pacing, vsync control, and adaptive frame rate
- Cursor enhancement rendering (enlargement, crosshairs, halo, locator)
- Color filter pipeline (inversion, contrast, high-contrast schemes)
- Performance optimization roadmap (XShm, GPU texture sharing, dirty regions)
- Font re-rendering research direction (Phase 3)
- Testing strategy for GPU code

This document does NOT cover:
- The `ScreenCapture` trait definition or per-platform capture implementations (see [02 -- Platform Abstraction](./02-platform-abstraction.md))
- The `WindowManager` trait definition or per-platform window management (see [02](./02-platform-abstraction.md))
- TTS pipeline design (see [04 -- TTS Pipeline](./04-tts-pipeline.md))
- Application-level data flow (see [01 -- System Architecture](./01-system-architecture.md), Section 5)

### 1.3 Key Constraints

| Constraint | Target | Source |
|------------|--------|--------|
| Frame rate | 60fps (16.67ms frame budget) | Product Strategy 8.6 |
| Frame time P99 | <20ms | System Architecture 9.1 |
| GPU hardware | Integrated GPUs (Intel UHD, Apple M-series, AMD Vega) | Product Strategy 8.6 |
| Zoom range | 1.5x -- 20x | Product Strategy 7.1 |
| RAM budget (rendering) | ~100MB for GPU textures | System Architecture 9.3 |
| Startup to first frame | <400ms from process start | System Architecture 9.4 |

### 1.4 Relationship to Other Documents

```
02-platform-abstraction.md         -- Defines ScreenCapture, WindowManager traits
    |
    v
03-rendering-pipeline.md (this)    -- HOW captured pixels become magnified frames
    |
    v
Implementation stories             -- Per-shader, per-mode implementation tasks
```

The rendering pipeline consumes `CaptureFrame` values produced by `ScreenCapture` trait implementations and renders to the overlay window managed by the `WindowManager` trait. It is the bridge between platform capture and user-visible output.

---

## 2. Pipeline Architecture

### 2.1 Pipeline Stages

The rendering pipeline executes once per frame (up to 60 times per second). Each execution produces one magnified frame on the overlay window. The pipeline is a sequential, five-stage process:

```
Every frame (~16.67ms at 60fps):

  1. VIEWPORT   Calculate source region from tracking target + zoom level
  2. CAPTURE    Acquire source region pixels via ScreenCapture trait
  3. UPLOAD     Transfer pixel buffer from CPU to GPU texture
  4. RENDER     Execute GPU shader pipeline (magnify + filter + cursor)
  5. PRESENT    Submit rendered frame to overlay window swap chain
```

### 2.2 Stage Data Flow

```
Tracking Target (cursor pos, focus bounds)    App State (zoom, mode, filters)
                  |                                        |
                  v                                        v
         +--------------------------------------------------+
Stage 1: | Viewport Calculator                               |
         | source_rect = compute_source_region(target, zoom) |
         +--------------------------------------------------+
                  |
                  v  ScreenRect
         +--------------------------------------------------+
Stage 2: | ScreenCapture::capture_frame(display, region)     |
         | Returns CaptureFrame { data, width, height,       |
         |   stride, format: Bgra8 | Rgba8 }                |
         +--------------------------------------------------+
                  |
                  v  CaptureFrame (CPU buffer)
         +--------------------------------------------------+
Stage 3: | Texture Upload                                    |
         | wgpu::Queue::write_texture(source_texture, data)  |
         | Handles format conversion (BGRA -> RGBA if needed)|
         | Double-buffered: write to back, read from front   |
         +--------------------------------------------------+
                  |
                  v  wgpu::Texture (GPU)
         +--------------------------------------------------+
Stage 4: | GPU Render Passes                                 |
         | Pass 1: Magnification shader                      |
         |   - Bicubic interpolation (16 taps)               |
         |   - Gamma-correct resampling (sRGB linearize)     |
         | Pass 2: Color filter shader (if active)           |
         |   - Inversion, contrast, color remap              |
         | Pass 3: Cursor overlay shader                     |
         |   - Enlarged cursor, crosshairs, halo             |
         +--------------------------------------------------+
                  |
                  v  Rendered frame in swap chain texture
         +--------------------------------------------------+
Stage 5: | Present                                           |
         | wgpu::SurfaceTexture::present()                   |
         | PresentMode controls vsync behavior               |
         +--------------------------------------------------+
                  |
                  v
         User's display (magnified view)
```

### 2.3 Performance Budget

| Stage | Typical Time | Worst Case | Notes |
|-------|-------------|------------|-------|
| Viewport calculation | <0.01ms | <0.01ms | Pure arithmetic |
| Screen capture | 1-5ms | 8ms | Region size and platform dependent |
| Texture upload | 0.5-1.5ms | 2ms | Proportional to region pixel count |
| Shader execution (all passes) | 0.2-1ms | 2ms | Trivial for integrated GPUs |
| Present (excluding vsync wait) | 0.1-0.5ms | 1ms | wgpu command submission |
| **Total (excluding vsync)** | **2-8ms** | **13ms** | Well within 16.67ms budget |

The vsync wait (`PresentMode::Fifo`) occupies the remaining time in the frame budget. This is idle time where the GPU waits for the display's vertical blanking interval.

**Critical insight:** The pipeline budget scales inversely with zoom level. At 20x zoom on 1080p, the source region is 96x54 pixels (0.02MB) -- capture and upload are nearly free. At 1.5x zoom, the source region is 1280x720 pixels (3.5MB) -- capture dominates the budget. Optimization effort must focus on low-zoom, high-resolution scenarios.

---

## 3. Viewport Calculation (Stage 1)

### 3.1 Source Region Computation

The viewport calculator determines which region of the screen to capture, based on the current tracking target (cursor position or focused element bounds) and the current zoom level.

```rust
/// Computes the source region of the screen to capture for magnification.
///
/// The source region is the unmagnified rectangle of screen content that,
/// when scaled by `zoom_level`, fills the overlay viewport.
pub(crate) fn compute_source_region(
    tracking_target: ScreenPoint,
    zoom_level: f32,
    viewport_size: (u32, u32),
    screen_bounds: ScreenRect,
) -> ScreenRect {
    let source_width = (viewport_size.0 as f32 / zoom_level).ceil() as i32;
    let source_height = (viewport_size.1 as f32 / zoom_level).ceil() as i32;

    // Center the source region on the tracking target.
    let mut x = tracking_target.x - source_width / 2;
    let mut y = tracking_target.y - source_height / 2;

    // Clamp to screen bounds (prevent capturing outside the display).
    x = x.clamp(screen_bounds.x, screen_bounds.x + screen_bounds.width as i32 - source_width);
    y = y.clamp(screen_bounds.y, screen_bounds.y + screen_bounds.height as i32 - source_height);

    ScreenRect {
        x,
        y,
        width: source_width as u32,
        height: source_height as u32,
    }
}
```

### 3.2 Source Region Size by Zoom Level

The source region shrinks as zoom increases, which is the primary reason higher zoom levels are cheaper to render:

| Zoom Level | Source Width (1920px viewport) | Source Height (1080px viewport) | Pixel Count | Data Size (BGRA) |
|-----------|-------------------------------|--------------------------------|-------------|-------------------|
| 1.5x | 1280 | 720 | 921,600 | 3.5 MB |
| 2x | 960 | 540 | 518,400 | 2.0 MB |
| 5x | 384 | 216 | 82,944 | 0.3 MB |
| 10x | 192 | 108 | 20,736 | 0.08 MB |
| 20x | 96 | 54 | 5,184 | 0.02 MB |

### 3.3 Tracking Modes

The tracking target varies by user interaction mode:

| Mode | Tracking Target | Update Source |
|------|----------------|---------------|
| Mouse follow | Mouse cursor position | `InputMonitor` events (every pointer move) |
| Focus follow | Center of focused element bounds | `FocusTracker` events (on focus change) |
| Hybrid (default) | Mouse when moving; focus on keyboard input | Switches based on last input type |

**Smooth panning:** When the tracking target changes, the viewport does not jump immediately. A configurable easing function (default: exponential ease-out) smoothly interpolates the viewport position over 3-5 frames to prevent disorienting jumps. The user can disable smoothing for maximum responsiveness.

```rust
/// Smoothly interpolates the viewport center toward the tracking target.
///
/// `smoothing_factor` controls the interpolation speed:
/// - 1.0 = instant (no smoothing)
/// - 0.1-0.3 = typical range for comfortable panning
pub(crate) fn smooth_viewport_position(
    current: ScreenPoint,
    target: ScreenPoint,
    smoothing_factor: f32,
) -> ScreenPoint {
    ScreenPoint {
        x: current.x + ((target.x - current.x) as f32 * smoothing_factor) as i32,
        y: current.y + ((target.y - current.y) as f32 * smoothing_factor) as i32,
    }
}
```

### 3.4 Edge Panning

In all zoom modes, when the cursor approaches the edge of the magnified view, the viewport pans to keep surrounding context visible. The panning speed increases as the cursor gets closer to the edge (proportional panning):

```
+-------------------------------------------+
|                                           |
|    Dead zone (no pan)                     |
|    +-------------------------------+      |
|    |                               |      |
|    |    Content area               |      |
|    |                               |      |
|    +-------------------------------+      |
|    Edge margin (pan proportional to       |
|    distance from inner boundary)          |
+-------------------------------------------+
```

The edge margin is configurable (default: 15% of viewport width/height). When the cursor position is within the edge margin, the viewport pans at a speed proportional to how far into the margin the cursor has moved.

---

## 4. Screen Capture Integration (Stage 2)

### 4.1 Capture Strategy

The rendering pipeline calls `ScreenCapture::capture_frame()` once per frame with the source region computed in Stage 1. The capture returns a `CaptureFrame` containing CPU-accessible pixel data.

```rust
// Per-frame capture (simplified)
let source_region = compute_source_region(target, zoom, viewport, screen);
let frame = screen_capture.capture_frame(display_id, Some(source_region))?;
```

**Region capture vs. full-screen capture:** The pipeline always requests the minimal source region, not the full screen. At high zoom levels this dramatically reduces capture time and bandwidth. However, some platforms may not support region-specific capture efficiently (capturing the full screen and cropping in software). The `ScreenCapture` trait allows either approach -- implementations choose the most efficient path for their platform.

### 4.2 Capture Frame Properties

The `CaptureFrame` struct (defined in [02 -- Platform Abstraction](./02-platform-abstraction.md)) provides:

| Field | Type | Purpose |
|-------|------|---------|
| `data` | `Arc<[u8]>` | Raw pixel data, row-major, top-left origin |
| `width` | `u32` | Frame width in pixels |
| `height` | `u32` | Frame height in pixels |
| `stride` | `u32` | Bytes per row (may include padding) |
| `format` | `PixelFormat` | `Bgra8` (X11/Windows) or `Rgba8` (macOS) |

### 4.3 Platform Pixel Format Handling

Different platforms return different pixel formats:

| Platform | Native Format | Handling |
|----------|--------------|----------|
| Linux X11 | BGRA8 | Swizzle in shader (zero-cost GPU operation) |
| Linux Wayland | BGRA8 | Swizzle in shader |
| macOS | RGBA8 | Direct use (wgpu native format) |
| OpenBSD | BGRA8 | Swizzle in shader |
| Windows | BGRA8 | Swizzle in shader |

Rather than performing a CPU-side format conversion (costly for large buffers), the magnification shader accepts the pixel format as a uniform and applies a channel swizzle in the fragment shader. This is a single GPU instruction per pixel -- effectively free.

### 4.4 Capture Failure Handling

If `capture_frame()` returns an error (display disconnected, permission revoked, transient platform error), the pipeline renders the previous frame rather than showing a blank screen. This provides graceful degradation:

```
Frame N:     capture succeeds -> upload -> render -> present (fresh)
Frame N+1:   capture fails    -> skip upload -> render from existing texture -> present (stale)
Frame N+2:   capture succeeds -> upload -> render -> present (fresh)
```

A stale frame counter tracks how many consecutive frames used stale data. After 60 consecutive stale frames (1 second), the pipeline emits a `warn!` log and sends a status event to the control panel.

---

## 5. GPU Texture Management (Stage 3)

### 5.1 Texture Layout

The rendering pipeline maintains four GPU textures:

| Texture | Format | Size | Purpose |
|---------|--------|------|---------|
| `source_texture` | `Rgba8UnormSrgb` | Source region dimensions | Holds captured screen pixels after upload |
| `intermediate_texture_a` | `Rgba8UnormSrgb` | Overlay viewport dimensions | Ping-pong buffer A for multi-pass rendering |
| `intermediate_texture_b` | `Rgba8UnormSrgb` | Overlay viewport dimensions | Ping-pong buffer B for multi-pass rendering |
| `cursor_texture` | `Rgba8UnormSrgb` | Small (64x64 -- 256x256) | Pre-rendered cursor sprite |

The swap chain surface texture (provided by wgpu for the overlay window) is the final render target.

### 5.2 Source Texture Upload

Each frame, the captured pixel buffer is uploaded from CPU memory to the source GPU texture:

```rust
/// Uploads a CaptureFrame to the GPU source texture.
///
/// Handles stride padding and texture reallocation when dimensions change.
pub(crate) fn upload_capture_frame(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    frame: &CaptureFrame,
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &frame.data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(frame.stride),
            rows_per_image: Some(frame.height),
        },
        wgpu::Extent3d {
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        },
    );
}
```

### 5.3 Texture Reallocation

The source region dimensions change when:
- The zoom level changes (different source region size)
- The overlay window resizes (different viewport, therefore different source region)
- The display resolution changes

When the source region dimensions change, the source texture must be reallocated. To avoid per-frame allocation, the pipeline over-allocates by a factor of 1.5x in each dimension and only reallocates when the captured frame exceeds the current texture capacity:

```
source_texture capacity: 1920 x 1080  (allocated once for a 1080p display)

Zoom 2x capture:  960 x 540   -> fits, no reallocation
Zoom 1.5x capture: 1280 x 720 -> fits, no reallocation
Zoom 1.5x on 4K:  2560 x 1440 -> exceeds capacity, reallocate to 3840 x 2160
```

### 5.4 sRGB Handling

Captured screen content is in sRGB color space. The source texture uses `Rgba8UnormSrgb` format, which tells wgpu to perform automatic sRGB-to-linear conversion when the texture is sampled in a shader. The swap chain surface also uses an sRGB format, so wgpu automatically converts linear-space shader output back to sRGB on write. This ensures gamma-correct interpolation without manual conversion in the shader:

```
Capture (sRGB) -> GPU texture (Rgba8UnormSrgb) -> shader reads linear -> interpolate in linear space -> write linear -> surface (sRGB)
```

**Why this matters:** Naive interpolation in sRGB space produces visible banding and color shifts at high zoom levels. Magnified text appears to have dark halos around bright edges. Linear-space interpolation produces perceptually correct results. Using sRGB-format textures achieves this automatically with zero performance cost.

---

## 6. Shader Pipeline (Stage 4)

### 6.1 Render Pass Architecture

The shader pipeline executes up to three render passes per frame. Each pass reads from one texture and writes to another (or to the swap chain surface):

```
Pass 1: Magnification
  Input:  source_texture (captured pixels)
  Output: intermediate_texture OR swap chain (if no filters active)
  Shader: magnify.wgsl

Pass 2: Color Filter (optional, skipped when no filter is active)
  Input:  intermediate_texture
  Output: swap chain surface
  Shader: color_filter.wgsl

Pass 3: Cursor Overlay (optional, composited on top)
  Input:  cursor_texture + swap chain surface
  Output: swap chain surface
  Shader: cursor.wgsl
```

When no color filter is active and cursor overlay is disabled, Pass 1 renders directly to the swap chain surface -- a single render pass per frame. This is the common case for users who only need magnification.

### 6.2 Magnification Shader (`magnify.wgsl`)

The magnification shader is the core of the rendering pipeline. It samples the source texture and produces a magnified view using bicubic interpolation.

```wgsl
// magnify.wgsl -- Bicubic magnification with sRGB-correct sampling

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

struct MagnifyUniforms {
    // Viewport dimensions (width, height) in pixels.
    viewport_size: vec2f,
    // Source texture dimensions (width, height) in pixels.
    source_size: vec2f,
    // Pixel format flag: 0.0 = RGBA (macOS), 1.0 = BGRA (X11/Win).
    is_bgra: f32,
    // Padding for 16-byte alignment.
    _pad: f32,
};

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: MagnifyUniforms;

// Full-screen triangle vertex shader (3 vertices, no vertex buffer needed).
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Generate a full-screen triangle from vertex index.
    let uv = vec2f(
        f32((vertex_index << 1u) & 2u),
        f32(vertex_index & 2u),
    );
    out.position = vec4f(uv * 2.0 - 1.0, 0.0, 1.0);
    // Flip Y for texture coordinates (top-left origin).
    out.uv = vec2f(uv.x, 1.0 - uv.y);
    return out;
}

// Cubic interpolation weight function (Catmull-Rom spline, a = -0.5).
// Produces sharper results than bilinear while avoiding ringing artifacts.
//
// Standard Catmull-Rom weights:
//   |x| <= 1:  1.5*|x|^3 - 2.5*|x|^2 + 1
//   1 < |x| < 2: -0.5*|x|^3 + 2.5*|x|^2 - 4*|x| + 2
//   |x| >= 2:  0
fn cubic_weight(x: f32) -> f32 {
    let ax = abs(x);
    if ax <= 1.0 {
        return 1.5 * ax * ax * ax - 2.5 * ax * ax + 1.0;
    }
    if ax < 2.0 {
        return -0.5 * ax * ax * ax + 2.5 * ax * ax - 4.0 * ax + 2.0;
    }
    return 0.0;
}

// Bicubic interpolation: 4x4 tap pattern (16 texture lookups per pixel).
fn sample_bicubic(tex: texture_2d<f32>, samp: sampler, uv: vec2f, tex_size: vec2f) -> vec4f {
    let pixel = uv * tex_size - 0.5;
    let pixel_floor = floor(pixel);
    let frac = pixel - pixel_floor;

    var result = vec4f(0.0);
    var weight_sum = 0.0;

    for (var j = -1; j <= 2; j++) {
        for (var i = -1; i <= 2; i++) {
            let offset = vec2f(f32(i), f32(j));
            let sample_pos = (pixel_floor + offset + 0.5) / tex_size;
            let w = cubic_weight(frac.x - f32(i)) * cubic_weight(frac.y - f32(j));
            result += textureSampleLevel(tex, samp, sample_pos, 0.0) * w;
            weight_sum += w;
        }
    }

    return result / weight_sum;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    // Sample the source texture with bicubic interpolation.
    var color = sample_bicubic(source_tex, source_sampler, in.uv, uniforms.source_size);

    // Channel swizzle for BGRA sources (X11, Windows).
    if uniforms.is_bgra > 0.5 {
        color = vec4f(color.b, color.g, color.r, color.a);
    }

    return color;
}
```

**Design notes:**

1. **Full-screen triangle:** A single triangle covering the entire viewport is more efficient than a quad (2 triangles). The vertex shader generates positions from `vertex_index` alone -- no vertex buffer is needed.

2. **Bilinear in Phase 0, bicubic in Phase 1:** The Product Strategy specifies bilinear interpolation for Phase 0 and "smooth text rendering (shader-based smoothing)" for Phase 1. Phase 0 uses a bilinear variant with a single `textureSampleLevel` call. Phase 1 upgrades to the Catmull-Rom bicubic shader shown above (a = -0.5), which produces noticeably sharper text and edges at high zoom (10-20x). Both shader variants are provided; the active variant is selected at pipeline initialization.

3. **Bilinear as performance fallback:** Even after Phase 1 enables bicubic by default, the bilinear variant remains available as a "performance mode" option. A single `textureSampleLevel` call instead of 16 taps reduces shader cost significantly, relevant on extremely constrained GPUs or when the user explicitly selects performance mode.

4. **BGRA swizzle:** A uniform flag controls channel order. The per-pixel cost is one comparison and one swizzle instruction -- effectively zero overhead on modern GPUs.

### 6.3 Color Filter Shader (`color_filter.wgsl`)

Color filters transform the magnified image for users with specific vision conditions. The filter shader runs as a separate pass to keep the magnification shader simple and to allow filters to be toggled without recompiling shaders.

```wgsl
// color_filter.wgsl -- Post-magnification color transformations

struct FilterUniforms {
    // Filter type: 0=none, 1=invert, 2=smart_invert, 3=grayscale,
    //              4=high_contrast, 5=custom_remap
    filter_type: u32,
    // Brightness adjustment (-1.0 to 1.0, 0.0 = no change).
    brightness: f32,
    // Contrast adjustment (0.0 to 3.0, 1.0 = no change).
    contrast: f32,
    // Padding.
    _pad: f32,
    // Custom color matrix (4x4, row-major) for filter_type=5.
    color_matrix: mat4x4f,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> filter: FilterUniforms;

@fragment
fn fs_main(@location(0) uv: vec2f) -> @location(0) vec4f {
    var color = textureSampleLevel(input_tex, input_sampler, uv, 0.0);

    // Apply brightness (additive in linear space).
    color = vec4f(color.rgb + vec3f(filter.brightness), color.a);

    // Apply contrast (multiply around 0.5 midpoint in linear space).
    // Note: Perceptual mid-gray in linear space is ~0.214 (sRGB 0.5).
    // Using 0.5 as the pivot in linear space is a deliberate simplification
    // that matches common real-time rendering practice. The visual effect is
    // slightly different from sRGB-space contrast adjustment: shadows are
    // affected more than highlights. This is acceptable for magnification
    // use and avoids an expensive sRGB encode/decode round-trip.
    color = vec4f((color.rgb - 0.5) * filter.contrast + 0.5, color.a);

    // Apply filter type.
    switch filter.filter_type {
        case 1u: {
            // Full inversion.
            color = vec4f(1.0 - color.rgb, color.a);
        }
        case 2u: {
            // Smart inversion: invert luminance, preserve hue.
            let luminance = dot(color.rgb, vec3f(0.2126, 0.7152, 0.0722));
            let inv_lum = 1.0 - luminance;
            let ratio = select(inv_lum / luminance, 0.0, luminance < 0.001);
            color = vec4f(color.rgb * ratio, color.a);
        }
        case 3u: {
            // Grayscale.
            let gray = dot(color.rgb, vec3f(0.2126, 0.7152, 0.0722));
            color = vec4f(vec3f(gray), color.a);
        }
        case 4u: {
            // High contrast: quantize to 2-level per channel.
            let threshold = vec3f(0.5);
            color = vec4f(step(threshold, color.rgb), color.a);
        }
        case 5u: {
            // Custom color matrix remap.
            color = filter.color_matrix * color;
        }
        default: {
            // No filter (filter_type=0). Pass through.
        }
    }

    return clamp(color, vec4f(0.0), vec4f(1.0));
}
```

**Preset high-contrast schemes** are implemented as custom color matrix remaps:

| Scheme | Description | Use Case |
|--------|-------------|----------|
| White on black | Invert + boost contrast | General low vision |
| Yellow on blue | Warm on cool | Cataracts, glare sensitivity |
| Green on black | High-luminance text | Night use, photophobia |
| Yellow on black | High-contrast warm | AMD, diabetic retinopathy |

Each scheme is a pre-computed 4x4 color matrix stored in the configuration and uploaded as a uniform.

### 6.4 Cursor Overlay Shader (`cursor.wgsl`)

The cursor overlay renders an enhanced cursor on top of the magnified view. This is essential at high zoom levels where the system cursor becomes relatively small.

```wgsl
// cursor.wgsl -- Cursor enhancement overlay

struct CursorUniforms {
    // Cursor position in viewport pixels.
    position: vec2f,
    // Cursor size in viewport pixels.
    size: vec2f,
    // Crosshair line width (0 = disabled).
    crosshair_width: f32,
    // Halo radius (0 = disabled).
    halo_radius: f32,
    // Cursor color (RGBA).
    cursor_color: vec4f,
    // Crosshair color (RGBA).
    crosshair_color: vec4f,
    // Halo color (RGBA, typically semi-transparent).
    halo_color: vec4f,
    // Viewport dimensions.
    viewport_size: vec2f,
    _pad: vec2f,
};

@group(0) @binding(0) var scene_tex: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
@group(0) @binding(2) var cursor_tex: texture_2d<f32>;
@group(0) @binding(3) var cursor_sampler: sampler;
@group(0) @binding(4) var<uniform> cursor: CursorUniforms;

@fragment
fn fs_main(@location(0) uv: vec2f) -> @location(0) vec4f {
    var color = textureSampleLevel(scene_tex, scene_sampler, uv, 0.0);
    let pixel_pos = uv * cursor.viewport_size;
    let delta = pixel_pos - cursor.position;

    // Halo (soft circle behind cursor).
    // smoothstep requires edge0 < edge1, so we compute the falloff from inner
    // to outer edge and invert to get alpha = 1.0 at center, 0.0 at boundary.
    if cursor.halo_radius > 0.0 {
        let dist = length(delta);
        let halo_alpha = 1.0 - smoothstep(cursor.halo_radius * 0.7, cursor.halo_radius, dist);
        color = mix(color, cursor.halo_color, halo_alpha * cursor.halo_color.a);
    }

    // Crosshairs (full viewport lines through cursor center).
    if cursor.crosshair_width > 0.0 {
        let half_w = cursor.crosshair_width * 0.5;
        let on_h_line = abs(delta.y) < half_w;
        let on_v_line = abs(delta.x) < half_w;
        // Exclude the cursor region itself from crosshairs.
        let in_cursor = abs(delta.x) < cursor.size.x * 0.5
                     && abs(delta.y) < cursor.size.y * 0.5;
        if (on_h_line || on_v_line) && !in_cursor {
            color = mix(color, cursor.crosshair_color, cursor.crosshair_color.a);
        }
    }

    // Cursor sprite (alpha-blended on top).
    let cursor_uv = (delta + cursor.size * 0.5) / cursor.size;
    if cursor_uv.x >= 0.0 && cursor_uv.x <= 1.0
        && cursor_uv.y >= 0.0 && cursor_uv.y <= 1.0 {
        let cursor_color = textureSampleLevel(cursor_tex, cursor_sampler, cursor_uv, 0.0);
        color = mix(color, cursor_color, cursor_color.a);
    }

    return color;
}
```

**Cursor enhancement features (Phase 1):**

| Feature | Description | Implementation |
|---------|-------------|----------------|
| Enlarged cursor | System cursor rendered at 2-4x normal size | Pre-rendered to `cursor_texture` from system cursor image |
| Crosshairs | Full-viewport lines intersecting at cursor | Shader-computed per-pixel, configurable width and color |
| Halo | Semi-transparent circle around cursor | Shader-computed `smoothstep`, configurable radius and color |
| Locator animation | Temporary "find my cursor" animation | Animated halo radius + alpha pulse, triggered by hotkey (Phase 1) |
| Color options | Customizable cursor/crosshair/halo colors | Uniform-driven, configurable in control panel |

### 6.5 Single-Pass Optimization

For the common case (no color filters, no cursor overlay), the three passes collapse into one:

```
No filters, no cursor:    source_texture --[magnify]--> swap chain          (1 pass)
Filters, no cursor:       source_texture --[magnify]--> inter_a --[filter]--> swap chain   (2 passes)
No filters, with cursor:  source_texture --[magnify]--> inter_a --[cursor]--> swap chain   (2 passes)
All active:               source_texture --[magnify]--> inter_a --[filter]--> inter_b --[cursor]--> swap chain   (3 passes)
```

The pipeline dynamically selects the pass configuration each frame based on active features. Inactive passes have zero cost.

---

## 7. Zoom Mode Rendering

### 7.1 Full-Screen Mode

In full-screen mode, the overlay covers the entire display. The magnified view follows the tracking target (cursor or focus), showing a zoomed-in portion of the screen.

```
Display: 1920 x 1080
Overlay: 1920 x 1080 (covers entire display)
Zoom: 5x
Source region: 384 x 216 (centered on cursor)
```

**Rendering:** The magnification shader stretches the source region to fill the entire overlay viewport. The overlay window is transparent in this mode only where no magnified content is displayed (which is nowhere -- the entire overlay is opaque magnified content).

**Self-capture prevention:** The overlay must be excluded from screen capture to prevent infinite feedback (magnified view capturing itself). The `ScreenCapture` implementation is responsible for exclusion, but the rendering pipeline must cooperate by providing the overlay window identifier to the capture backend at initialization. Per-platform mechanisms:

| Platform | Self-Capture Exclusion Mechanism |
|----------|----------------------------------|
| Linux X11 | Capture the root window's composite pixmap (excludes overlay windows), or temporarily unmap the overlay before capture and remap after |
| Linux Wayland | PipeWire screen capture can filter by window; the overlay's PipeWire node ID is excluded |
| macOS | `SCContentFilter` in ScreenCaptureKit accepts an exclusion list of `SCWindow` objects |
| OpenBSD | Same as Linux X11 (xenocara X11) |
| Windows | DXGI Desktop Duplication excludes windows by default; WGC supports `GraphicsCaptureSession.IsBorderRequired` and window exclusion |

The rendering pipeline exposes the overlay window handle to the `ScreenCapture` backend during initialization via the `WindowManager::raw_window_handle()` method.

### 7.2 Lens Mode

In lens mode, the overlay is a movable, resizable rectangle or ellipse that follows the cursor. The area outside the lens is transparent (click-through), allowing the user to interact with content beneath.

```
Display: 1920 x 1080
Overlay: 1920 x 1080 (full display, mostly transparent)
Lens: 400 x 300 rectangle (follows cursor)
Zoom: 5x
Source region: 80 x 60 (centered on cursor)
```

**Rendering:** The magnification shader writes to the lens region only. All pixels outside the lens boundary have alpha = 0 (fully transparent). The shader computes the lens boundary per-pixel:

```wgsl
// In the fragment shader, compute whether this pixel is inside the lens.
let lens_center = cursor.position;  // lens follows cursor
let pixel_pos = uv * viewport_size;
let delta = pixel_pos - lens_center;

// For rectangular lens:
let inside = abs(delta.x) < lens_half_w && abs(delta.y) < lens_half_h;

// For elliptical lens:
let inside = (delta.x * delta.x) / (lens_half_w * lens_half_w)
           + (delta.y * delta.y) / (lens_half_h * lens_half_h) <= 1.0;

if !inside {
    return vec4f(0.0); // Fully transparent (click-through)
}
```

**Lens border:** A configurable border (2-4px, high-contrast color) is rendered at the lens boundary to make the magnified region clearly delineated from the surrounding content.

### 7.3 Docked Mode

In docked mode, the overlay reserves a portion of the screen edge (top, bottom, left, or right). The reserved area shows the magnified view; other windows are prevented from overlapping it (on platforms that support screen reservation).

```
Display: 1920 x 1080
Docked edge: Top, height: 400px
Overlay: 1920 x 400 (top edge of screen)
Remaining desktop: 1920 x 680
Zoom: 5x
Source region: 384 x 80 (centered on cursor, within remaining desktop)
```

**Rendering:** The magnification shader fills the docked region. The overlay window is positioned and sized to match the docked area. The `WindowManager` trait handles setting EWMH struts (X11), AppBar registration (Windows), or floating panel properties (macOS).

**Source region in docked mode:** The source region is computed relative to the non-docked area of the screen. The magnified view never shows content from within the docked region itself -- this would create a feedback loop.

```rust
// Adjust screen bounds to exclude the docked region when computing source region.
let available_bounds = match dock_edge {
    DockEdge::Top => ScreenRect {
        x: screen.x,
        y: screen.y + dock_size as i32,
        width: screen.width,
        height: screen.height - dock_size,
    },
    DockEdge::Bottom => ScreenRect {
        x: screen.x,
        y: screen.y,
        width: screen.width,
        height: screen.height - dock_size,
    },
    // ... Left, Right similarly
};
```

### 7.4 Mode Transitions

When the user switches between zoom modes (e.g., full-screen to lens), the transition is handled by:

1. `WindowManager::set_overlay_mode()` updates window properties (bounds, struts, transparency)
2. The rendering pipeline updates its viewport configuration
3. On the next frame, the new mode's rendering logic takes effect

No animation is applied to mode transitions in Phase 0 (instant switch). Animated transitions are a Phase 1 enhancement.

---

## 8. Frame Pacing and VSync

### 8.1 Present Modes

The rendering pipeline supports three frame pacing strategies via wgpu's `PresentMode`:

| Mode | wgpu Enum | Behavior | Use Case |
|------|-----------|----------|----------|
| Quality (default) | `PresentMode::Fifo` | VSync'd at display refresh rate (typically 60fps) | Normal use -- smooth, no tearing. Available on all platforms. |
| Low-latency | `PresentMode::Mailbox` | VSync'd but allows frame replacement (latest frame wins) | Minimizes cursor-to-display latency. **Limited availability:** supported on DX12 (Windows), NVIDIA Vulkan, and Wayland Vulkan. NOT available on macOS (Metal), AMD/Intel X11 Vulkan, or OpenBSD. Pipeline falls back to `Fifo` when `Mailbox` is unavailable. |
| Performance | `PresentMode::Immediate` | No vsync, uncapped frame rate | Custom frame limiting for power-saving mode (20-30fps) |

### 8.2 Adaptive Frame Rate

In performance mode, the pipeline uses a software frame limiter to target a user-specified frame rate (default: 30fps in performance mode):

```rust
/// Software frame limiter for performance mode.
///
/// Skips rendering when the elapsed time since the last frame
/// is less than the target frame interval.
pub(crate) fn should_render_frame(
    last_frame_time: Instant,
    target_fps: u32,
) -> bool {
    let target_interval = Duration::from_secs_f64(1.0 / target_fps as f64);
    last_frame_time.elapsed() >= target_interval
}
```

**Power-saving benefit:** At 30fps, the GPU is idle for ~50% of the time compared to 60fps. On laptop hardware, this can meaningfully extend battery life. The user can set target FPS from the control panel (range: 15-144fps).

### 8.3 Frame Time Monitoring

The pipeline tracks frame timing to detect performance issues:

```rust
/// Frame timing statistics for performance monitoring.
pub(crate) struct FrameTimings {
    /// Circular buffer of the last 120 frame times.
    history: [Duration; 120],
    /// Write index into the circular buffer.
    index: usize,
}

impl FrameTimings {
    pub(crate) fn record(&mut self, frame_time: Duration) {
        self.history[self.index] = frame_time;
        self.index = (self.index + 1) % self.history.len();
    }

    /// Returns the P99 frame time over the last 120 frames (2 seconds at 60fps).
    pub(crate) fn p99(&self) -> Duration {
        let mut sorted = self.history;
        sorted.sort_unstable();
        sorted[118] // 99th percentile of 120 samples: ceil(0.99 * 120) - 1
    }

    /// Returns the average frame time over the last 120 frames.
    pub(crate) fn average(&self) -> Duration {
        let sum: Duration = self.history.iter().sum();
        sum / self.history.len() as u32
    }

    /// Returns the minimum frame time over the last 120 frames.
    pub(crate) fn min(&self) -> Duration {
        self.history.iter().copied().min().unwrap_or_default()
    }

    /// Returns the maximum frame time over the last 120 frames.
    pub(crate) fn max(&self) -> Duration {
        self.history.iter().copied().max().unwrap_or_default()
    }

    /// Creates a `FrameTimingSummary` for the control panel IPC response.
    /// See [05 -- Control Panel](./05-control-panel.md) Section 3.4.
    pub(crate) fn summary(&self, target_fps: u32) -> FrameTimingSummary {
        FrameTimingSummary {
            average_ms: self.average().as_secs_f64() * 1000.0,
            p99_ms: self.p99().as_secs_f64() * 1000.0,
            min_ms: self.min().as_secs_f64() * 1000.0,
            max_ms: self.max().as_secs_f64() * 1000.0,
            target_fps,
        }
    }
}
```

Frame timing data is exposed to the control panel for a real-time performance overlay (debug feature, disabled by default) and logged at `trace` level for profiling.

**Performance degradation response:** Two severity thresholds are defined:

| Threshold | Condition | Response |
|-----------|-----------|----------|
| **Warning (amber)** | P99 frame time > 20ms for 5 consecutive seconds | Log `warn!`, emit `performance_warning` event with recommendation |
| **Critical (red)** | P99 frame time > 33ms (under 30fps) for 5 consecutive seconds | Log `error!`, emit `performance_warning` event with stronger recommendation |

The control panel uses these thresholds to display appropriate visual indicators (amber/red) and suggest corrective actions (switch to performance mode, reduce zoom level). See [05 -- Control Panel](./05-control-panel.md) Section 10.1 for the UI treatment.

---

## 9. wgpu Initialization and Device Management

### 9.1 Device Selection

The pipeline requests a wgpu device with minimal feature requirements to maximize hardware compatibility:

```rust
/// Creates the wgpu device and queue for the rendering pipeline.
///
/// Requests low power preference by default (integrated GPU) to avoid
/// switching to a discrete GPU on dual-GPU systems, which would increase
/// power consumption without meaningful benefit for our workload.
pub(crate) async fn create_gpu_device(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
) -> Result<(wgpu::Device, wgpu::Queue), RenderError> {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| RenderError::NoAdapter)?;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("luminos_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            },
            None,
        )
        .await
        .map_err(|e| RenderError::DeviceCreation { message: e.to_string() })?;

    Ok((device, queue))
}
```

**Key choices:**
- **`LowPower` preference:** Selects the integrated GPU on dual-GPU systems. The magnification workload is trivial for any GPU; using the discrete GPU wastes power. **Note:** On some older dual-GPU systems (e.g., NVIDIA Optimus), explicitly selecting the integrated GPU may prevent the discrete GPU from powering down fully. A user-configurable GPU preference override is provided in the control panel for edge cases where system-default selection produces better power behavior.
- **`downlevel_webgl2_defaults`:** Starts with the most conservative limits, then raises them to the adapter's actual capabilities via `using_resolution()`. This ensures the pipeline runs on the widest range of hardware.
- **No required features:** All shader operations (texture sampling, uniform buffers, basic math) are available in the base WebGPU feature set.

### 9.2 Surface Configuration

```rust
/// Configures the wgpu surface for the overlay window.
pub(crate) fn configure_surface(
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    width: u32,
    height: u32,
    present_mode: wgpu::PresentMode,
) {
    let caps = surface.get_capabilities(adapter);

    // Prefer sRGB format for gamma-correct rendering.
    let format = caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(caps.formats[0]);

    surface.configure(
        device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::PreMultiplied,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        },
    );
}
```

**`alpha_mode: PreMultiplied`:** Required for the transparent overlay to composite correctly with the desktop. In lens mode, pixels outside the lens have alpha = 0 and must be fully transparent. Pre-multiplied alpha avoids fringing artifacts at lens boundaries.

### 9.3 Pipeline Caching

Render pipelines and bind group layouts are created once at startup and reused for every frame. Shader compilation happens at initialization time (within the ~100ms wgpu initialization window from T=100ms to T=200ms in the startup budget). Pipeline objects are stored in the `Renderer` struct:

```rust
/// Holds all persistent GPU resources for the rendering pipeline.
pub(crate) struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    magnify_pipeline: wgpu::RenderPipeline,
    filter_pipeline: wgpu::RenderPipeline,
    cursor_pipeline: wgpu::RenderPipeline,
    magnify_bind_group_layout: wgpu::BindGroupLayout,
    filter_bind_group_layout: wgpu::BindGroupLayout,
    cursor_bind_group_layout: wgpu::BindGroupLayout,
    source_texture: wgpu::Texture,
    intermediate_texture_a: wgpu::Texture,
    intermediate_texture_b: wgpu::Texture,
    cursor_texture: wgpu::Texture,
    frame_timings: FrameTimings,
    present_mode: wgpu::PresentMode,
    stale_frame_count: u32,
}
```

---

## 10. Performance Optimization Roadmap

### 10.1 Phase 0: Baseline Implementation

The Phase 0 implementation uses the simplest correct approach:

- xcap captures the source region as a CPU buffer
- The entire buffer is uploaded to a GPU texture every frame
- Bilinear shader renders the magnified view (per Product Strategy Phase 0 specification)
- `PresentMode::Fifo` for vsync

Phase 1 upgrades the magnification shader to bicubic (Catmull-Rom) interpolation, which provides sharper text at high zoom levels, fulfilling the "smooth text rendering" Phase 1 feature.

This baseline is sufficient for 60fps at medium-to-high zoom levels (3x+) on integrated GPUs. Low zoom on high-resolution displays may exceed the frame budget due to large capture regions.

### 10.2 Phase 1: XShm Capture Backend (Linux X11)

The xcap crate uses `xcb_get_image` for X11 capture, which performs a full X server round-trip per capture. The `x11rb`-based XShm capture backend eliminates this by using shared memory:

```
xcb_get_image (current):
  Client -> X Server request -> X Server copies pixels -> response to client
  Round-trip latency: 1-8ms depending on region size

XShm (Phase 1 optimization):
  Client -> X Server: "copy to shared memory segment"
  X Server writes directly to shared memory (no network/socket copy)
  Client reads from shared memory
  Latency: 0.5-3ms, lower variance
```

This optimization is the top priority for Phase 1 because it directly addresses the worst-case capture time at low zoom levels.

### 10.3 Phase 1+: Dirty Region Tracking

When the captured screen content changes only partially between frames (e.g., a blinking cursor in a text editor), uploading the entire source region is wasteful. Dirty region tracking uploads only the changed pixels:

1. Capture the full source region
2. Compare with the previous frame (CPU-side, row-by-row byte comparison)
3. Upload only the dirty rectangular sub-regions to the GPU texture

This is most beneficial at low zoom levels where the source region is large but changes are often localized. The comparison cost (~0.1ms for a 1280x720 region) is offset by the reduced upload cost when only a small area changed.

**Caution:** The comparison introduces complexity and is only beneficial when changes are sparse. If the entire screen is changing (e.g., video playback, scrolling), comparison wastes time. A heuristic should disable dirty tracking when more than 30% of pixels change, falling back to full upload.

### 10.4 Phase 2+: GPU Texture Sharing

The highest-performance capture path avoids CPU buffers entirely by sharing GPU textures between the capture API and wgpu:

| Platform | Mechanism | Benefit |
|----------|-----------|---------|
| Windows | DXGI texture shared with wgpu DX12 backend | Zero-copy capture-to-render |
| macOS | IOSurface shared with wgpu Metal backend | Zero-copy capture-to-render |
| Linux (Wayland) | DMA-BUF shared with wgpu Vulkan backend | Zero-copy capture-to-render |
| Linux X11 | Not available (X11 has no GPU texture sharing for captures) | XShm is the best available |

GPU texture sharing eliminates the CPU-to-GPU upload step entirely, removing the largest variable cost from the pipeline. This is pursued after the baseline implementation is stable.

### 10.5 Optimization Priority Matrix

| Optimization | Phase | Impact | Complexity | Prerequisite |
|-------------|-------|--------|------------|--------------|
| XShm capture (X11) | 1 | High (2-5ms saved at low zoom) | Medium | x11rb integration |
| Dirty region tracking | 1+ | Medium (0.5-2ms saved when screen is static) | Medium | Per-frame comparison logic |
| GPU texture sharing (Windows) | 2+ | High (eliminates upload entirely) | High | DXGI/wgpu interop |
| GPU texture sharing (macOS) | 2+ | High | High | IOSurface/Metal interop |
| GPU texture sharing (Wayland) | 2+ | High | High | DMA-BUF/Vulkan interop |
| Bilinear fallback shader | 0 | Low (reduces shader cost, but shader is already fast) | Low | Shader variant |
| Bicubic interpolation upgrade | 1 | Medium (sharper text at high zoom) | Low | Shader variant swap |
| Render on demand (skip unchanged) | 1 | Medium (skip entire pipeline when nothing changes) | Low | Change detection |

### 10.6 Anti-Aliasing Strategy (Phase 1)

Product Strategy Phase 1 specifies "Smooth text rendering -- anti-aliased text at high magnification (shader-based smoothing)." This is addressed through a combination of techniques:

1. **Bicubic interpolation (Phase 1 shader upgrade):** Catmull-Rom bicubic interpolation is the primary anti-aliasing mechanism. At high zoom (10-20x), it produces significantly sharper text edges than bilinear. See Section 6.2.

2. **MSAA (optional, Phase 1):** wgpu supports multisample anti-aliasing via `wgpu::MultisampleState`. Enabling 4x MSAA smooths edges at the lens boundary (lens mode) and the cursor overlay. MSAA is less beneficial for the magnified content itself (which is texture-sampled, not geometry-rasterized).

3. **FXAA post-process (deferred evaluation):** A fast approximate anti-aliasing post-process pass could further smooth magnified content. This adds one additional render pass (~0.2ms on integrated GPUs). Evaluation deferred to Phase 1 -- bicubic may provide sufficient quality alone.

The rendering pipeline architecture (multi-pass, texture ping-pong) supports adding anti-aliasing passes without structural changes.

---

## 11. Font Re-Rendering (Phase 3 Research Direction)

### 11.1 The Problem

At high magnification, rasterized text looks fuzzy even with bicubic interpolation because the original text was rasterized at its native size using sub-pixel hinting optimized for that size. Magnifying rasterized glyphs amplifies hinting artifacts and reveals the limited resolution of the original render.

Commercial tools (ZoomText's xFont, Dolphin SuperNova's TrueFont) solve this by re-rendering text at the magnified size using the system's font renderer. The re-rendered text is crisp because it uses full-resolution hinting and anti-aliasing appropriate for the magnified size.

### 11.2 Design Direction

Font re-rendering requires:

1. **Text extraction:** Identify text content, position, font, size, and color from the screen.
   - Source: Accessibility APIs (`FocusTracker` provides element bounds and text content)
   - Source: OCR (Phase 3) for applications without accessibility support

2. **Font matching:** Determine which system font produced the rendered text.
   - Accessibility APIs may report font family (UIA on Windows, AXFont on macOS)
   - Heuristic matching when API data is unavailable

3. **Re-rendering:** Render the identified text at magnified size using the matched font.
   - Platform font APIs: DirectWrite (Windows), Core Text (macOS), FreeType (Linux)
   - Render to a texture, composite on top of the magnified bitmap

4. **Compositing:** Overlay re-rendered text on the magnified view, aligning precisely with the original text positions.

### 11.3 Risks and Open Questions

| Question | Status | Notes |
|----------|--------|-------|
| Can we reliably extract font metadata from accessibility APIs? | Needs investigation | Coverage varies by platform and application |
| How do we handle custom/embedded fonts? | Needs investigation | Web apps often use custom fonts not installed on the system |
| What about text in images, canvas elements, or games? | OCR-dependent | OCR can extract text but not font identity |
| Performance impact of re-rendering text every frame? | Needs benchmarking | May need to cache re-rendered text tiles and invalidate on change |

Font re-rendering is a Phase 3 feature. Research should begin in Phase 1 (investigating accessibility API font data availability). The rendering pipeline's architecture supports font re-rendering as an additional compositing layer without modifying the core magnification shader.

---

## 12. Testing Strategy

### 12.1 Unit Tests

| Component | Test Approach | Example Tests |
|-----------|--------------|---------------|
| Viewport calculator | Pure function tests | `viewport_calc_centered_on_cursor`, `viewport_calc_clamped_to_screen`, `viewport_calc_zoom_level_scaling` |
| Smooth panning | Pure function tests | `smooth_pan_converges_to_target`, `smooth_pan_factor_one_is_instant` |
| Frame limiter | Pure function tests | `frame_limiter_skips_when_too_early`, `frame_limiter_allows_when_due` |
| Frame timings | Pure function tests | `frame_timings_p99_returns_correct_percentile`, `frame_timings_average_correct` |
| Source region sizing | Table-driven tests | `source_region_size_at_zoom_levels` (parameterized across zoom levels) |

### 12.2 Shader Tests

WGSL shaders cannot be unit-tested directly in Rust. The testing strategy for shaders:

1. **Screenshot comparison tests:** Render a known input texture through each shader, capture the output, and compare against reference images (pixel-level diff with a tolerance threshold for GPU floating-point variance).

2. **wgpu headless rendering:** wgpu supports headless (offscreen) rendering for CI environments without a display. Use `wgpu::Backends::GL` or `wgpu::Backends::VULKAN` with a headless surface.

3. **Shader-specific test cases:**

| Shader | Test | Verification |
|--------|------|-------------|
| `magnify.wgsl` | 2x zoom of a checkerboard pattern | Bicubic output matches reference (no aliasing artifacts at edges) |
| `magnify.wgsl` | BGRA swizzle produces same output as RGBA input | Pixel-exact comparison after swizzle |
| `magnify.wgsl` | 1x zoom produces pixel-exact copy of input | Identity transform verification |
| `color_filter.wgsl` | Full inversion of solid white produces black | Pixel-exact comparison |
| `color_filter.wgsl` | Grayscale of pure red matches expected luminance | Channel-value verification |
| `cursor.wgsl` | Crosshair renders at correct position | Line presence at expected pixel coordinates |
| `cursor.wgsl` | Halo has correct radius and alpha falloff | Radial profile comparison |

### 12.3 Integration Tests

| Test | Scope | Method |
|------|-------|--------|
| Full pipeline frame | Capture -> upload -> render -> readback | Use `generate_test_capture_frame()` mock, render, read back GPU output |
| Mode switching | Full-screen -> lens -> docked | Verify window properties and render output change correctly |
| Stale frame handling | Simulate capture failure | Verify previous frame is re-displayed, stale counter increments |
| Texture reallocation | Change zoom level rapidly | Verify no crashes and correct rendering after reallocation |
| Performance regression | Render 600 frames, measure P99 | Assert P99 < 20ms on CI hardware |

### 12.4 Test Fixtures

```rust
#[cfg(test)]
pub mod render_test_utils {
    use crate::platform::CaptureFrame;
    use std::sync::Arc;

    /// Generates a test CaptureFrame with a checkerboard pattern.
    ///
    /// Useful for verifying interpolation behavior at shader boundaries.
    pub fn generate_test_checkerboard_frame(
        width: u32,
        height: u32,
        cell_size: u32,
    ) -> CaptureFrame {
        let stride = width * 4;
        let mut data = vec![0u8; (stride * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let cell_x = x / cell_size;
                let cell_y = y / cell_size;
                let is_white = (cell_x + cell_y) % 2 == 0;
                let offset = ((y * stride) + (x * 4)) as usize;
                let val = if is_white { 255 } else { 0 };
                data[offset] = val;     // R
                data[offset + 1] = val; // G
                data[offset + 2] = val; // B
                data[offset + 3] = 255; // A
            }
        }
        CaptureFrame {
            data: Arc::from(data.into_boxed_slice()),
            width,
            height,
            stride,
            format: crate::platform::PixelFormat::Rgba8,
        }
    }
}
```

---

## 13. Error Handling

### 13.1 Error Types

```rust
/// Errors that can occur in the rendering pipeline.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// No compatible GPU adapter found.
    #[error("no compatible GPU adapter found")]
    NoAdapter,

    /// GPU device creation failed.
    #[error("GPU device creation failed: {message}")]
    DeviceCreation { message: String },

    /// Surface configuration failed (e.g., window not available).
    #[error("surface configuration failed: {message}")]
    SurfaceConfig { message: String },

    /// The swap chain surface texture could not be acquired.
    #[error("failed to acquire surface texture: {message}")]
    SurfaceAcquire { message: String },

    /// Shader compilation failed.
    #[error("shader compilation failed for '{shader}': {message}")]
    ShaderCompilation { shader: String, message: String },

    /// Screen capture failed (delegates to CaptureError).
    #[error("screen capture failed: {0}")]
    Capture(#[from] crate::platform::CaptureError),

    /// A GPU device error occurred (out of memory, device lost).
    #[error("GPU device error: {message}")]
    DeviceError { message: String },
}
```

### 13.2 Graceful Degradation

The rendering pipeline follows a "never go black" principle: the user should always see something on the overlay, even when errors occur.

| Error | Response | User Impact |
|-------|----------|-------------|
| Capture failure (transient) | Re-render previous frame | Brief stale content (imperceptible for 1-2 frames) |
| Capture failure (persistent, >1s) | Display stale frame + status bar warning | User sees stale view; warning suggests action |
| Surface texture lost | Reconfigure surface, skip frame | Single dropped frame during resize |
| GPU device lost | Reinitialize entire GPU stack | Brief blank (~200ms), then recovery |
| Shader compilation failure | Fatal at startup | Application cannot start; clear error message |

---

## 14. Architectural Decisions Register

| # | Decision | Choice | Rationale | Status |
|---|----------|--------|-----------|--------|
| RD-01 | Interpolation algorithm | Bicubic (Catmull-Rom) default, bilinear fallback | Bicubic provides sharper text at high zoom; 16 taps is trivial on modern GPUs | Decided |
| RD-02 | Pixel format handling | Shader-side swizzle via uniform flag | Zero CPU cost; avoids per-frame buffer manipulation | Decided |
| RD-03 | Render pass architecture | Multi-pass (1-3 passes depending on active features) | Keeps shaders simple and composable; inactive passes have zero cost | Decided |
| RD-04 | sRGB handling | Hardware sRGB textures (`Rgba8UnormSrgb`) | Automatic gamma-correct interpolation with zero shader cost | Decided |
| RD-05 | Present mode | `Fifo` (quality), `Immediate` (performance) | Matches P0 adjustable refresh rate requirement directly | Decided |
| RD-06 | GPU power preference | `LowPower` (integrated GPU), user-overridable | Magnification workload is trivially simple; discrete GPU wastes power. User override for dual-GPU edge cases. | Decided |
| RD-07 | Texture over-allocation | 1.5x capacity to reduce reallocation frequency | Avoids per-frame allocation when zoom changes gradually | Decided |
| RD-08 | Stale frame tolerance | Re-render previous frame for up to 60 frames (1s) before warning | "Never go black" -- user always sees content | Decided |
| RD-09 | Full-screen triangle | Single triangle, no vertex buffer | Standard technique; fewer vertices, no VBO overhead | Decided |
| RD-10 | Composite alpha mode | Pre-multiplied alpha | Required for correct lens mode transparency compositing | Decided |

---

## 15. Cross-References

| Topic | Document | Section |
|-------|----------|---------|
| `ScreenCapture` trait and `CaptureFrame` type | [02 -- Platform Abstraction](./02-platform-abstraction.md) | Section 3.2 |
| `WindowManager` trait and `OverlayMode` enum | [02 -- Platform Abstraction](./02-platform-abstraction.md) | Section 3.5 |
| `PixelFormat` enum (BGRA/RGBA) | [02 -- Platform Abstraction](./02-platform-abstraction.md) | Section 3.1 |
| High-level pipeline overview and data flow | [01 -- System Architecture](./01-system-architecture.md) | Sections 4.4, 5.1 |
| Thread model (render thread isolation) | [01 -- System Architecture](./01-system-architecture.md) | Section 6 |
| Performance targets and memory budget | [01 -- System Architecture](./01-system-architecture.md) | Section 9 |
| wgpu validation and magnification pipeline | [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) | Sections 4.2, 4.8 |
| xcap capture strategy and XShm plan | [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) | Section 4.3 |
| Phase 0-1 rendering features | [Product Strategy](../PRODUCT_STRATEGY.md) | Section 7 |
| TTS pipeline (runs independently, same process) | [04 -- TTS Pipeline](./04-tts-pipeline.md) | All |
| Consolidated performance targets and memory budget | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Sections 2.1, 2.2 |
| Hot path budget breakdown | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 2.3 |
| Degradation strategy (auto-degrade at sustained P99 > 33ms) | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 2.5 |
| Profiling tools (tracy, perf, Instruments) | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 2.4 |
| Error handling policy and RenderError recovery | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 7 |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-15 | Initial rendering pipeline specification |
| 1.1 | 2026-03-15 | Post audit review: fixed bicubic weight function (F-001), smoothstep edge order (F-002), configure_surface signature (F-003), data size consistency (F-004), Phase attribution (F-005/F-006), added ping-pong textures for 3-pass (F-007), DeviceDescriptor defaults (F-008), P99 index (F-009), startup budget (F-010), timing alignment (F-011). Added Mailbox platform caveats (P-001), contrast midpoint documentation (P-002), Phase 1 anti-aliasing strategy (P-003), per-platform self-capture exclusion (P-004), dual-GPU power note (P-005). |
