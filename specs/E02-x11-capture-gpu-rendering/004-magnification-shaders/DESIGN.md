# Design: Story E02/004 -- Magnification Shaders & Viewport

**Story:** [STORY.md](./STORY.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** Spec Writer 2
**Risk Refs:** [RISK-004](../../tech-strategy/10-risk-register.md) (render thread starvation), [RISK-006](../../tech-strategy/10-risk-register.md) (multi-display coordinate inconsistencies), [RISK-016](../../tech-strategy/10-risk-register.md) (wgpu backend compatibility)

---

## Overview

This design specifies the WGSL magnification shaders, wgpu render pipeline setup, and viewport calculation logic for the Luminos magnification pipeline. Two shader variants are provided: bilinear (Phase 0 default, single texture sample per pixel) and bicubic Catmull-Rom (higher quality, 16 texture samples per pixel). Both share the same vertex shader, uniform layout, and bind group layout, differing only in the fragment shader interpolation function.

The viewport calculator is a pure arithmetic module that computes which region of the screen to capture based on a tracking target (cursor position) and zoom level. It is independent of any GPU or windowing code, making it trivially testable.

**RISK-004 awareness:** The shader stage is the fastest in the pipeline (~0.2-1ms per doc-03 Section 2.3). Render thread starvation risk comes primarily from capture and upload, not shader execution. However, the bicubic shader's 16-tap pattern should be benchmarked on Intel UHD-class GPUs to confirm it stays within budget.

**RISK-006 awareness:** `compute_source_region()` operates in physical pixel coordinates. All coordinates are i32/u32 (not floating-point display coordinates). Multi-display is out of scope for E02 (single display only), but the function accepts `screen_bounds: ScreenRect` to support multi-display in future epics.

**RISK-016 awareness:** Both shaders use only basic WGSL features (texture sampling, arithmetic, conditionals) available in the base WebGPU feature set. No required wgpu features are needed. The GL backend (Mesa llvmpipe in CI) supports all operations used.

## Architecture

### Component Diagram

```
crates/luminos-gpu/src/
  |
  +-- lib.rs                             # Module declarations
  +-- error.rs                           # RenderError (from Story 002)
  +-- device.rs                          # GPU device init (from Story 002)
  +-- surface.rs                         # Surface config (from Story 002)
  |
  +-- viewport.rs                        # compute_source_region(), smooth_viewport_position() (NEW)
  |
  +-- shaders/
        +-- mod.rs                       # Shader compilation, pipeline creation, MagnifyUniforms (NEW)
        +-- magnify_bilinear.wgsl        # Bilinear magnification shader (NEW)
        +-- magnify_bicubic.wgsl         # Bicubic Catmull-Rom shader (NEW)
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `luminos-gpu::viewport` | New | `compute_source_region()`, `smooth_viewport_position()` |
| `luminos-gpu::shaders` | New | Shader module with pipeline creation, `MagnifyUniforms` |
| `luminos-gpu::shaders::magnify_bilinear.wgsl` | New | Bilinear magnification WGSL shader |
| `luminos-gpu::shaders::magnify_bicubic.wgsl` | New | Bicubic Catmull-Rom WGSL shader |
| `luminos-gpu::lib` | Modified | Add `pub mod viewport;` and `pub mod shaders;` |

### Data Flow

```
1. Tracking target (ScreenPoint) + zoom level (f32) + viewport size (u32, u32)
   |
   v
2. compute_source_region() -> ScreenRect (source region to capture)
   |
   v
   [Capture and upload happen in Stories 001/003 -- not this story]
   |
   v
3. Source texture (Rgba8UnormSrgb) is bound to shader via bind group
   |
   v
4. MagnifyUniforms written to GPU buffer:
   - viewport_size: output dimensions
   - source_size: source texture dimensions
   - is_bgra: 0.0 for X11 (xcap returns RGBA), 0.0 for macOS, 1.0 for Windows (DXGI returns BGRA)
   |
   v
5. Render pass: draw 3 vertices (full-screen triangle)
   |
   v
6. Vertex shader: generate full-screen triangle positions + UVs from vertex_index
   |
   v
7. Fragment shader: sample source texture (bilinear or bicubic), apply BGRA swizzle
   |
   v
8. Output: magnified pixels written to render target (swap chain or intermediate texture)
```

---

## API Design

### `MagnifyUniforms` -- `crates/luminos-gpu/src/shaders/mod.rs`

```rust
/// GPU uniform buffer data for the magnification shader.
///
/// This struct is uploaded to a wgpu uniform buffer every frame.
/// All fields must be 16-byte aligned per WebGPU uniform buffer
/// layout requirements.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MagnifyUniforms {
    /// Output viewport dimensions (width, height) in pixels.
    pub viewport_size: [f32; 2],
    /// Source texture dimensions (width, height) in pixels.
    pub source_size: [f32; 2],
    /// Pixel format flag: 0.0 = RGBA (X11 via xcap, macOS), 1.0 = BGRA (Windows DXGI).
    pub is_bgra: f32,
    /// Padding for 16-byte alignment.
    pub _pad: [f32; 3],
}
```

**Note on `bytemuck`:** `bytemuck` is used for safe casting of the uniform struct to `&[u8]` for `Queue::write_buffer()`. It is already a transitive dependency of `wgpu`. If not available directly, the `bytemuck` crate should be added to `luminos-gpu/Cargo.toml`.

### Shader Interpolation Variant Enum

```rust
/// Selects the magnification interpolation algorithm.
///
/// Bilinear is the Phase 0 default (single texture sample per pixel).
/// Bicubic provides higher quality at higher GPU cost (16 samples per pixel).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationMethod {
    /// Single-sample bilinear interpolation (hardware texture filtering).
    Bilinear,
    /// 4x4 Catmull-Rom bicubic interpolation (16 manual texture lookups).
    Bicubic,
}
```

### Shader Pipeline Creation -- `crates/luminos-gpu/src/shaders/mod.rs`

```rust
/// Resources for the magnification shader pipeline.
pub struct MagnifyPipeline {
    /// The compiled render pipeline (bilinear or bicubic variant).
    pub pipeline: wgpu::RenderPipeline,
    /// Bind group layout for source texture + sampler + uniforms.
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// Uniform buffer for MagnifyUniforms.
    pub uniform_buffer: wgpu::Buffer,
}

/// Creates the bind group layout shared by both shader variants.
///
/// Layout:
/// - Binding 0: source texture (2D, float, sampled)
/// - Binding 1: source sampler (filtering)
/// - Binding 2: uniform buffer (MagnifyUniforms)
pub fn create_magnify_bind_group_layout(
    device: &wgpu::Device,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("magnify_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

/// Creates a magnification render pipeline for the specified shader variant.
///
/// Both bilinear and bicubic variants share the same vertex shader,
/// bind group layout, and pipeline layout. They differ only in the
/// fragment shader (interpolation function).
///
/// # Arguments
///
/// * `device` -- The wgpu device.
/// * `surface_format` -- The swap chain surface texture format (sRGB).
/// * `bind_group_layout` -- The bind group layout from `create_magnify_bind_group_layout`.
/// * `method` -- The interpolation method (Bilinear or Bicubic).
///
/// # Errors
///
/// Returns [`RenderError::ShaderCompilation`] if the shader fails to compile.
pub fn create_magnify_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
    method: InterpolationMethod,
) -> Result<MagnifyPipeline, RenderError> {
    let shader_source = match method {
        InterpolationMethod::Bilinear => include_str!("magnify_bilinear.wgsl"),
        InterpolationMethod::Bicubic => include_str!("magnify_bicubic.wgsl"),
    };

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(match method {
            InterpolationMethod::Bilinear => "magnify_bilinear_shader",
            InterpolationMethod::Bicubic => "magnify_bicubic_shader",
        }),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("magnify_pipeline_layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(match method {
            InterpolationMethod::Bilinear => "magnify_bilinear_pipeline",
            InterpolationMethod::Bicubic => "magnify_bicubic_pipeline",
        }),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            buffers: &[], // Full-screen triangle: no vertex buffer
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("magnify_uniforms_buffer"),
        size: std::mem::size_of::<MagnifyUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    Ok(MagnifyPipeline {
        pipeline,
        bind_group_layout: bind_group_layout.clone(),
        uniform_buffer,
    })
}

/// Creates a bind group for a specific source texture.
///
/// Called each frame (or when the source texture changes) to bind
/// the current source texture to the shader.
pub fn create_magnify_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    source_texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("magnify_bind_group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform_buffer.as_entire_binding(),
            },
        ],
    })
}
```

### Viewport Calculation -- `crates/luminos-gpu/src/viewport.rs`

```rust
use luminos_platform::traits::{ScreenPoint, ScreenRect};

/// Computes the source region of the screen to capture for magnification.
///
/// The source region is the unmagnified rectangle of screen content that,
/// when scaled by `zoom_level`, fills the overlay viewport.
///
/// The region is centered on `tracking_target` and clamped to `screen_bounds`
/// to prevent capturing outside the display.
///
/// # Arguments
///
/// * `tracking_target` -- The point to center the magnified view on (cursor position).
/// * `zoom_level` -- The magnification factor (1.5 to 20.0).
/// * `viewport_size` -- The overlay viewport dimensions (width, height) in pixels.
/// * `screen_bounds` -- The display bounds in physical pixel coordinates.
///
/// # Panics
///
/// Does not panic. Returns a zero-size region if `zoom_level` is zero or negative.
pub fn compute_source_region(
    tracking_target: ScreenPoint,
    zoom_level: f32,
    viewport_size: (u32, u32),
    screen_bounds: ScreenRect,
) -> ScreenRect {
    if zoom_level <= 0.0 {
        return ScreenRect {
            x: tracking_target.x,
            y: tracking_target.y,
            width: 0,
            height: 0,
        };
    }

    let source_width = (viewport_size.0 as f32 / zoom_level).ceil() as i32;
    let source_height = (viewport_size.1 as f32 / zoom_level).ceil() as i32;

    // Center the source region on the tracking target.
    let mut x = tracking_target.x - source_width / 2;
    let mut y = tracking_target.y - source_height / 2;

    // Clamp to screen bounds (prevent capturing outside the display).
    let max_x = screen_bounds.x + screen_bounds.width as i32 - source_width;
    let max_y = screen_bounds.y + screen_bounds.height as i32 - source_height;

    x = x.clamp(screen_bounds.x, max_x.max(screen_bounds.x));
    y = y.clamp(screen_bounds.y, max_y.max(screen_bounds.y));

    ScreenRect {
        x,
        y,
        width: source_width.max(0) as u32,
        height: source_height.max(0) as u32,
    }
}

/// Smoothly interpolates the viewport center toward the tracking target.
///
/// Uses linear interpolation with a configurable smoothing factor to
/// prevent disorienting viewport jumps when the tracking target moves.
///
/// # Arguments
///
/// * `current` -- The current viewport center position.
/// * `target` -- The desired viewport center (tracking target).
/// * `smoothing_factor` -- Interpolation speed (0.0 = no movement, 1.0 = instant).
///   Typical range: 0.1-0.3 for comfortable panning.
pub fn smooth_viewport_position(
    current: ScreenPoint,
    target: ScreenPoint,
    smoothing_factor: f32,
) -> ScreenPoint {
    let factor = smoothing_factor.clamp(0.0, 1.0);
    ScreenPoint {
        x: current.x + ((target.x - current.x) as f32 * factor) as i32,
        y: current.y + ((target.y - current.y) as f32 * factor) as i32,
    }
}
```

### Bilinear Shader -- `magnify_bilinear.wgsl`

```wgsl
// magnify_bilinear.wgsl -- Bilinear magnification with sRGB-correct sampling
//
// Phase 0 default shader. Uses hardware texture filtering (single
// textureSampleLevel call) for maximum performance. Text may appear
// slightly blurry at high zoom levels (10x+). The bicubic variant
// provides sharper results at higher GPU cost.

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

struct MagnifyUniforms {
    viewport_size: vec2f,
    source_size: vec2f,
    is_bgra: f32,
    _pad: f32,
    _pad2: vec2f,
};

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: MagnifyUniforms;

// Full-screen triangle vertex shader (3 vertices, no vertex buffer needed).
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2f(
        f32((vertex_index << 1u) & 2u),
        f32(vertex_index & 2u),
    );
    out.position = vec4f(uv * 2.0 - 1.0, 0.0, 1.0);
    // Flip Y for texture coordinates (top-left origin).
    out.uv = vec2f(uv.x, 1.0 - uv.y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    // Sample the source texture with bilinear interpolation (hardware filtering).
    var color = textureSampleLevel(source_tex, source_sampler, in.uv, 0.0);

    // Channel swizzle for BGRA sources (X11, Windows).
    if uniforms.is_bgra > 0.5 {
        color = vec4f(color.b, color.g, color.r, color.a);
    }

    return color;
}
```

### Bicubic Shader -- `magnify_bicubic.wgsl`

```wgsl
// magnify_bicubic.wgsl -- Bicubic Catmull-Rom magnification with sRGB-correct sampling
//
// Higher-quality shader using 4x4 tap pattern (16 texture lookups per pixel).
// Produces sharper text and edges at high zoom levels (10x-20x) compared to
// bilinear. Uses Catmull-Rom spline weights (a = -0.5) for optimal sharpness
// without ringing artifacts.

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

struct MagnifyUniforms {
    viewport_size: vec2f,
    source_size: vec2f,
    is_bgra: f32,
    _pad: f32,
    _pad2: vec2f,
};

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: MagnifyUniforms;

// Full-screen triangle vertex shader (shared with bilinear variant).
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2f(
        f32((vertex_index << 1u) & 2u),
        f32(vertex_index & 2u),
    );
    out.position = vec4f(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2f(uv.x, 1.0 - uv.y);
    return out;
}

// Cubic interpolation weight function (Catmull-Rom spline, a = -0.5).
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

---

## Error Handling

- **Shader compilation errors:** `create_magnify_pipeline()` captures shader compilation failures via `RenderError::ShaderCompilation`. wgpu reports WGSL validation errors at `create_shader_module()` time.
- **Pipeline creation errors:** Wrapped in `RenderError::RenderFailed`.
- **Viewport edge cases:** `compute_source_region()` handles zero/negative zoom (returns zero-size region), source region larger than screen (clamps to screen bounds), and tracking target outside screen bounds (clamps to valid position).
- **No `unwrap()`/`expect()`:** All fallible operations return `Result`. Uniform buffer writes via `Queue::write_buffer()` are infallible in wgpu's API.

---

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 (xcap) | wgpu Vulkan backend, `is_bgra = 0.0` | xcap returns RGBA on X11 (not BGRA as doc-03 Section 4.3 states). No swizzle needed. GL fallback via Mesa llvmpipe in CI. |
| Linux Wayland | Same shaders, TBD | PipeWire capture format TBD. Shaders are platform-independent. |
| macOS | Same shaders, `is_bgra = 0.0` | Metal backend. RGBA native format, no swizzle needed. |
| OpenBSD | Same shaders, TBD | Vulkan via Mesa, capture format depends on backend. |
| Windows | Same shaders, `is_bgra = 1.0` | DX12 backend. DXGI returns BGRA, swizzle required. |

The shaders and viewport calculation are fully platform-independent. Only the `is_bgra` uniform value changes per platform. This is set by the render loop (Story 005) based on the `CaptureFrame::format` field.

**Important correction from research (Task #1):** doc-03 Section 4.3 states X11 uses BGRA, but the `xcap` crate used in E02 returns RGBA on X11. The `is_bgra` uniform will be `0.0` for the E02 X11 path. The BGRA swizzle code is retained for future backends (Windows DXGI, potentially raw X11/XShm captures). Both code paths must be tested.

---

## Testing Strategy

### Unit Tests

- **`compute_source_region` tests:**
  - `viewport_source_region_2x_zoom` -- 1920x1080 viewport at 2x = 960x540 source
  - `viewport_source_region_5x_zoom` -- 1920x1080 viewport at 5x = 384x216 source
  - `viewport_source_region_10x_zoom` -- 1920x1080 viewport at 10x = 192x108 source
  - `viewport_source_region_20x_zoom` -- 1920x1080 viewport at 20x = 96x54 source
  - `viewport_source_region_1_5x_zoom` -- 1920x1080 viewport at 1.5x = 1280x720 source
  - `viewport_source_region_centered` -- Target at (960, 540) produces centered region
  - `viewport_source_region_clamp_left` -- Target at (0, 540) clamps x to 0
  - `viewport_source_region_clamp_top` -- Target at (960, 0) clamps y to 0
  - `viewport_source_region_clamp_right` -- Target near right edge clamps x
  - `viewport_source_region_clamp_bottom` -- Target near bottom edge clamps y
  - `viewport_source_region_clamp_corner` -- Target at (0, 0) clamps both x and y
  - `viewport_source_region_zero_zoom` -- Returns zero-size region

- **`smooth_viewport_position` tests:**
  - `smooth_viewport_factor_1_0` -- Factor 1.0 = instant (returns target)
  - `smooth_viewport_factor_0_0` -- Factor 0.0 = no movement (returns current)
  - `smooth_viewport_factor_0_5` -- Factor 0.5 = halfway between current and target
  - `smooth_viewport_clamp_factor` -- Factor > 1.0 clamped to 1.0

- **`MagnifyUniforms` tests:**
  - `magnify_uniforms_size` -- Verify `std::mem::size_of::<MagnifyUniforms>()` is 32 bytes (2 vec2f + 1 f32 + 3 f32 padding = 8 * 4)
  - `magnify_uniforms_alignment` -- Verify struct is valid for bytemuck `Pod`/`Zeroable`

- **`InterpolationMethod` tests:**
  - `interpolation_method_equality` -- Verify `Bilinear != Bicubic`

### Integration Tests (GPU, requires wgpu + Mesa llvmpipe)

- **Shader compilation tests:**
  - `shader_bilinear_compiles` -- Compile bilinear shader on GL backend, verify no error
  - `shader_bicubic_compiles` -- Compile bicubic shader on GL backend, verify no error
  - `pipeline_bilinear_creates` -- Create full render pipeline with bilinear shader, verify success
  - `pipeline_bicubic_creates` -- Create full render pipeline with bicubic shader, verify success

- **Shader output tests:**
  - `shader_bilinear_solid_color` -- Render a solid red source texture through bilinear shader, read back output, verify all pixels are red
  - `shader_bicubic_solid_color` -- Same test with bicubic shader
  - `shader_bilinear_bgra_swizzle` -- Render a BGRA source (blue in R channel) with `is_bgra = 1.0`, verify output shows correct blue
  - `shader_bicubic_bgra_swizzle` -- Same test with bicubic shader

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Integration | Render 2x magnified source, verify output dimensions match viewport |
| AC-1.2 | Integration | Render at 1.5x, 5x, 10x, 20x, verify no black regions or artifacts |
| AC-1.3 | Unit | `compute_source_region` dimension tests at all zoom levels |
| AC-1.4 | Unit | `compute_source_region` edge clamping tests |
| AC-2.1 | Integration | Compare bilinear vs bicubic output on sharp edge texture at 10x |
| AC-2.2 | Code review | Verify bicubic shader has 4x4 loop with `cubic_weight` function |
| AC-2.3 | Integration | Both pipelines compile and can be swapped at init time |
| AC-3.1 | Integration | BGRA swizzle test (red source with `is_bgra = 1.0`) |
| AC-3.2 | Integration | RGBA passthrough test (red source with `is_bgra = 0.0`) |
| AC-4.1 | Integration | Bilinear shader compiles on Vulkan and GL backends |
| AC-4.2 | Integration | Bicubic shader compiles on Vulkan and GL backends |
| AC-4.3 | Integration | Render pipeline creation succeeds for both variants |
| AC-4.4 | Unit | `MagnifyUniforms` size and alignment tests |
| AC-5.1 | Integration | Full-screen triangle covers entire render target |
| AC-5.2 | Integration | UV coordinates map correctly (verified by color gradient test) |

---

## Performance Targets

| Metric | Target | Source |
|--------|--------|--------|
| Shader execution (bilinear) | <0.5ms | Single texture sample per pixel, trivial on any GPU |
| Shader execution (bicubic) | <2ms | 16 texture samples per pixel, within doc-03 Section 2.3 budget |
| `compute_source_region()` | <0.01ms | Pure arithmetic, no allocations |
| `smooth_viewport_position()` | <0.01ms | Two multiply-add operations |
| Shader compilation (startup) | <100ms | One-time cost at pipeline initialization |

---

## Security Considerations

- **RISK-017 compliance:** Shader code does not log or expose pixel data. GPU textures are not readable from CPU without explicit readback. Uniform buffers contain only numeric parameters (viewport size, source size, format flag) -- no sensitive data.
- **No external data:** Shaders process only internally captured screen content. No external data sources.
- **WGSL safety:** WGSL is memory-safe by design. Out-of-bounds texture access returns zero. No buffer overflows are possible in shader code.

---

## Alternatives Considered

1. **Single shader with uniform-controlled interpolation mode:** Rejected. A single shader with an `if` branch on interpolation method would execute the branch check for every pixel. While the GPU branch cost is minimal, having two separate shader modules makes the code clearer and allows wgpu to optimize each pipeline independently. The shared vertex shader and uniform layout already minimizes duplication.

2. **Lanczos interpolation instead of Catmull-Rom:** Considered. Lanczos (sinc-based) produces slightly sharper results but has more ringing artifacts on text. Catmull-Rom (a = -0.5) is the standard choice for screen magnification tools because it balances sharpness and artifact avoidance. Lanczos can be added as a third variant in a future phase if needed.

3. **Viewport calculation in `luminos-core` instead of `luminos-gpu`:** Considered. The viewport calculation is pure arithmetic and could live in either crate. Placing it in `luminos-gpu` keeps all rendering pipeline logic in one crate and avoids adding a `luminos-gpu` dependency on `luminos-core` (which would create a circular dependency since `luminos-core` may need GPU types in the future). The function depends only on `luminos-platform::traits::{ScreenPoint, ScreenRect}` which `luminos-gpu` already imports.
