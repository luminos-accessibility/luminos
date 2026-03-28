# Design: Story E02/003 -- GPU Texture Pipeline

**Story:** [STORY.md](./STORY.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** Spec Writer Agent
**Risk Refs:** [RISK-008](../../tech-strategy/10-risk-register.md#risk-008-cpu-to-gpu-texture-upload-bandwidth-pressure) (texture upload bandwidth), [RISK-010](../../tech-strategy/10-risk-register.md#risk-010-memory-pressure-on-4gb-total-ram-systems) (memory pressure), [RISK-017](../../tech-strategy/10-risk-register.md#risk-017-screen-content-and-tts-text-leakage-via-logs-and-gpu-memory) (screen content leakage)

---

## Overview

This design implements the GPU texture management layer that bridges CPU-side screen capture and GPU-side shader rendering. The `SourceTextureManager` struct manages the lifecycle of the wgpu source texture: creation, upload, reallocation, stale frame tracking, and texture view provision for shader binding.

The design follows doc-03 Section 5 precisely. Key decisions:
- **`Rgba8UnormSrgb` format** for gamma-correct sampling without manual conversion
- **1.5x over-allocation** to absorb zoom changes without per-frame reallocation
- **Stale frame fallback** to prevent blank screens on capture failure
- **Single-buffer upload** for Phase 0 (the sequential pipeline ensures upload completes before the render pass begins, so no tearing occurs; double-buffered swap is a Phase 1 optimization for threaded capture/render pipelining)

RISK-008 (upload bandwidth pressure) is monitored: at 1.5x zoom on 1080p, the upload is ~3.5MB per frame (1280*720*4). At 60fps, this is ~210MB/s through the system memory bus, well within DDR4 capacity. Performance logging enables detection if the budget is exceeded.

## Architecture

### Component Diagram

```
crates/luminos-gpu/src/
  |
  +-- lib.rs                # mod texture; (add module declaration)
  |
  +-- texture.rs            # SourceTextureManager struct
  |
  +-- device.rs             # GPU device initialization (Story 002, referenced)
```

```
Data Flow (per frame):

  CaptureFrame (CPU buffer, from Story 001)
       |
       v
  SourceTextureManager::upload()
       |
       +-- Check if reallocation needed (frame > texture capacity)
       |     +-- Yes: create new texture with 1.5x over-allocation
       |     +-- No: reuse existing texture
       |
       +-- wgpu::Queue::write_texture(source_texture, frame.data)
       |     +-- Respect frame.stride for row padding
       |
       +-- Update current_dimensions to (frame.width, frame.height)
       +-- Reset stale_frame_count to 0
       |
       v
  SourceTextureManager::texture_view()
       |
       v
  wgpu::TextureView (consumed by magnification shader, Story 004)
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `luminos-gpu::texture` | New | `SourceTextureManager` struct and associated methods |
| `luminos-gpu::lib.rs` | Modified | Add `pub mod texture;` declaration |
| `luminos-gpu::Cargo.toml` | Possibly modified | May need `luminos-platform` features for test utils |

### Data Flow

**Upload path:**
1. `CaptureFrame` arrives from `ScreenCapture::capture_frame()` (Story 001)
2. `SourceTextureManager::upload()` checks if `frame.width > capacity_width || frame.height > capacity_height`
3. If reallocation needed: create new `wgpu::Texture` with `(frame.width * 1.5, frame.height * 1.5)` dimensions
4. `queue.write_texture()` copies `frame.data` to the source texture, using `frame.stride` for `bytes_per_row`
5. Update `current_width` and `current_height` to the frame's actual dimensions
6. Reset `stale_frame_count` to 0

**Stale frame path:**
1. `capture_frame()` returns an error
2. Caller invokes `SourceTextureManager::record_capture_failure()`
3. `stale_frame_count` increments
4. If `stale_frame_count == 60`: emit `warn!("Capture stale for '{}' consecutive frames", count)`
5. Shader continues reading from the existing texture (stale data, but not blank)

---

## API Design

```rust
// crates/luminos-gpu/src/texture.rs

use luminos_platform::traits::types::CaptureFrame;

/// Manages the GPU source texture for the rendering pipeline.
///
/// Handles texture creation, upload from `CaptureFrame`, over-allocation
/// to minimize reallocation frequency, and stale frame tracking when
/// capture fails.
///
/// # Texture Format
///
/// The source texture uses `Rgba8UnormSrgb` format. This enables automatic
/// sRGB-to-linear conversion when the shader samples the texture, producing
/// gamma-correct interpolation without manual conversion. On X11 with xcap,
/// the pixel data is already RGBA, so it maps directly to `Rgba8UnormSrgb`
/// with no channel reordering needed. For future platform backends that
/// produce BGRA (e.g., Windows DXGI), the BGRA-to-RGBA channel swizzle is
/// handled by the magnification shader via a uniform flag, not by this module.
///
/// # Over-Allocation Strategy
///
/// Textures are allocated at 1.5x the requested dimensions to absorb
/// dimension changes from zoom level adjustments without reallocation.
/// Reallocation only occurs when the captured frame exceeds the current
/// texture capacity.
pub struct SourceTextureManager {
    /// The wgpu device for texture creation.
    device: wgpu::Device,
    /// The current GPU source texture.
    texture: wgpu::Texture,
    /// The texture view for shader binding.
    view: wgpu::TextureView,
    /// Allocated texture width (over-allocated, >= current_width).
    capacity_width: u32,
    /// Allocated texture height (over-allocated, >= current_height).
    capacity_height: u32,
    /// Width of the most recently uploaded frame.
    current_width: u32,
    /// Height of the most recently uploaded frame.
    current_height: u32,
    /// Count of consecutive frames where capture failed (stale frame count).
    stale_frame_count: u32,
}

/// The stale frame warning threshold (60 frames = 1 second at 60fps).
const STALE_FRAME_WARN_THRESHOLD: u32 = 60;

/// The texture over-allocation factor (1.5x in each dimension).
const OVER_ALLOCATION_FACTOR: f32 = 1.5;

/// The GPU texture format for source textures.
const SOURCE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

impl SourceTextureManager {
    /// Creates a new source texture manager with an initial texture.
    ///
    /// The initial texture is over-allocated by 1.5x in each dimension
    /// to absorb zoom-related dimension changes.
    ///
    /// # Arguments
    ///
    /// * `device` - The wgpu device for texture creation.
    /// * `initial_width` - Expected initial source region width.
    /// * `initial_height` - Expected initial source region height.
    pub fn new(device: wgpu::Device, initial_width: u32, initial_height: u32) -> Self {
        let capacity_width = over_allocate(initial_width);
        let capacity_height = over_allocate(initial_height);

        let texture = create_source_texture(&device, capacity_width, capacity_height);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            device,
            texture,
            view,
            capacity_width,
            capacity_height,
            current_width: initial_width,
            current_height: initial_height,
            stale_frame_count: 0,
        }
    }

    /// Uploads a `CaptureFrame` to the GPU source texture.
    ///
    /// If the frame dimensions exceed the current texture capacity,
    /// the texture is reallocated with 1.5x over-allocation.
    ///
    /// Resets the stale frame counter on successful upload.
    ///
    /// # Arguments
    ///
    /// * `queue` - The wgpu queue for texture data transfer.
    /// * `frame` - The captured frame to upload.
    pub fn upload(&mut self, queue: &wgpu::Queue, frame: &CaptureFrame) {
        // Reallocate if frame exceeds capacity
        if frame.width > self.capacity_width || frame.height > self.capacity_height {
            self.reallocate(frame.width, frame.height);
        }

        // Upload pixel data to GPU texture
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
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

        self.current_width = frame.width;
        self.current_height = frame.height;
        self.stale_frame_count = 0;
    }

    /// Records a capture failure (stale frame).
    ///
    /// Increments the stale frame counter. Emits a `warn!` log when
    /// the counter reaches 60 (1 second at 60fps).
    pub fn record_capture_failure(&mut self) {
        self.stale_frame_count += 1;

        if self.stale_frame_count == STALE_FRAME_WARN_THRESHOLD {
            log::warn!(
                "Capture stale for '{}' consecutive frames ({}s at 60fps)",
                self.stale_frame_count,
                self.stale_frame_count / 60
            );
        }
    }

    /// Returns the texture view for shader binding.
    ///
    /// The view is always valid -- even during stale frame situations,
    /// it references the last successfully uploaded texture data.
    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Returns the dimensions of the most recently uploaded frame.
    ///
    /// These are the actual frame dimensions, not the over-allocated
    /// texture capacity. The shader uses these as `source_size` in
    /// the `MagnifyUniforms` struct.
    pub fn current_dimensions(&self) -> (u32, u32) {
        (self.current_width, self.current_height)
    }

    /// Returns the number of consecutive stale frames.
    pub fn stale_frame_count(&self) -> u32 {
        self.stale_frame_count
    }

    /// Reallocates the source texture with 1.5x over-allocation.
    fn reallocate(&mut self, new_width: u32, new_height: u32) {
        self.capacity_width = over_allocate(new_width);
        self.capacity_height = over_allocate(new_height);

        log::info!(
            "Reallocating source texture: {}x{} -> {}x{} (capacity {}x{})",
            self.current_width,
            self.current_height,
            new_width,
            new_height,
            self.capacity_width,
            self.capacity_height,
        );

        self.texture = create_source_texture(
            &self.device,
            self.capacity_width,
            self.capacity_height,
        );
        self.view = self.texture.create_view(&wgpu::TextureViewDescriptor::default());
    }
}

/// Computes the over-allocated dimension (1.5x, rounded up).
fn over_allocate(dimension: u32) -> u32 {
    (dimension as f32 * OVER_ALLOCATION_FACTOR).ceil() as u32
}

/// Creates a wgpu texture for source pixel data.
fn create_source_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("luminos_source_texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SOURCE_TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}
```

---

## Error Handling

The texture pipeline does not define new error types. It operates within the infallible wgpu API surface:

| Operation | Failure Mode | Handling |
|-----------|-------------|----------|
| Texture creation | wgpu device lost | Propagated via wgpu device error callback (not handled in this module) |
| Texture upload | wgpu device lost | Same |
| Frame dimensions = 0 | Invalid `CaptureFrame` | Not expected (capture validates dimensions); no explicit check |
| Texture reallocation | wgpu out of memory | wgpu device error callback |

The `upload()` and `record_capture_failure()` methods do not return `Result` because `wgpu::Queue::write_texture()` is infallible from the API perspective (errors are reported asynchronously via the device error callback). This matches the wgpu 28.0 API design.

---

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | **This story.** wgpu Vulkan backend. | Primary target. Tests on Mesa llvmpipe in CI. |
| Linux Wayland | Same wgpu API. | Texture pipeline is platform-independent. |
| macOS | Same wgpu API (Metal backend). | No changes needed. |
| OpenBSD | Same wgpu API. | No changes needed. |
| Windows | Same wgpu API (DX12/Vulkan backend). | No changes needed. |

The `SourceTextureManager` is platform-independent -- it operates on wgpu abstractions and `CaptureFrame` values. The only platform-specific concern is the `PixelFormat` (BGRA vs RGBA), which is handled by the shader (Story 004) via the `is_bgra` uniform flag, not this module. Note: xcap on X11 produces RGBA, so `is_bgra` will be `false` for the E02 target platform. The BGRA swizzle path exists for future platform backends (Windows DXGI, etc.).

---

## Testing Strategy

### Unit Tests

Unit tests verify pure logic without GPU:

- **`over_allocate` function:** Test 1.5x calculation at various dimensions (1, 100, 960, 1920, 3840).
- **Stale frame counter:** Test counter increment, threshold logging, and reset.
- **Dimension tracking:** Test that `current_dimensions()` returns the last uploaded frame's dimensions.

### Integration Tests

Integration tests require wgpu (Mesa llvmpipe in CI):

- **Texture creation:** Create `SourceTextureManager` on Mesa llvmpipe, verify texture is created with `Rgba8UnormSrgb` format and over-allocated dimensions.
- **Upload and readback:** Upload a `generate_test_capture_frame()` to GPU, read back pixels via a staging buffer, verify pixel values match (accounting for sRGB encoding).
- **Reallocation:** Upload a small frame, then a larger frame exceeding capacity, verify reallocation occurs and the larger frame is correctly uploaded.
- **Stale frame preservation:** Upload a frame, call `record_capture_failure()` 60 times, verify `texture_view()` still returns a valid view and `stale_frame_count()` returns 60.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Integration | Upload test frame, read back pixels, verify match |
| AC-1.2 | Unit | Verify `SOURCE_TEXTURE_FORMAT == Rgba8UnormSrgb` |
| AC-1.3 | Integration | Upload frame with stride > width*4, read back, verify correct |
| AC-2.1 | Unit | `over_allocate(960) >= 1440`, `over_allocate(540) >= 810` |
| AC-2.2 | Unit / Integration | Upload 1280x720 frame to 1440x810 capacity, verify no realloc |
| AC-2.3 | Integration | Upload 1920x1080 frame to 1440x810 capacity, verify realloc to >= 2880x1620 |
| AC-2.4 | Integration | After reallocation, upload succeeds to new texture |
| AC-3.1 | Integration | Upload frame, then call `record_capture_failure()`, verify `texture_view()` still valid |
| AC-3.2 | Unit | Call `record_capture_failure()` 60 times, verify warn log (use `log::set_logger` test helper or check stale count) |
| AC-3.3 | Unit | After failures, upload new frame, verify `stale_frame_count() == 0` |
| AC-4.1 | Integration | Verify `texture_view()` returns a view compatible with `texture_2d<f32>` binding |
| AC-4.2 | Unit | Upload 960x540 frame, verify `current_dimensions() == (960, 540)` |
| AC-5.1 | Unit | Verify `SOURCE_TEXTURE_FORMAT == Rgba8UnormSrgb` constant |
| AC-5.2 | Integration | Upload known sRGB values, sample in test shader, verify linear output |

---

## Performance Targets

| Metric | Target | Source | Measurement |
|--------|--------|--------|-------------|
| Upload time (small, 96x54 = 0.02MB) | < 0.5ms | doc-03 Section 2.3 | `Instant::now()` around `upload()` |
| Upload time (medium, 960x540 = 2.0MB) | < 1.5ms | doc-03 Section 2.3 | Same |
| Upload time (large, 1280x720 = 3.5MB) | < 2ms | doc-03 Section 2.3 | Same |
| Texture reallocation | < 1ms | Startup budget | Same |
| GPU memory (source texture, 1080p) | < 18MB | doc-03 Section 1.3 | `1920*1.5 * 1080*1.5 * 4` |

Note: Upload times on Mesa llvmpipe in CI will be dominated by software rendering overhead. CI benchmark assertions should use relaxed thresholds.

---

## Security Considerations

- **RISK-017:** The `SourceTextureManager` handles raw pixel data (screen content). Logging must never include pixel data. The `upload()` method logs dimensions only, never buffer contents. The `CaptureFrame` custom `Debug` impl (E01) prevents accidental pixel leakage in debug output.
- **GPU memory:** On integrated GPUs sharing system memory, texture data resides in the same physical memory as application data. No additional isolation is available at the application level. This is inherent to integrated GPU architectures and documented in RISK-017.

---

## Alternatives Considered

1. **Double-buffered texture swap (write to back, read from front):** Prevents potential GPU pipeline stalls if upload and shader read happen simultaneously. Adds complexity (two textures, swap management). Decision: deferred to Phase 1. The single-buffer approach is sufficient for Phase 0 because `write_texture()` completes before the render pass begins (sequential pipeline execution within a frame).

2. **CPU-side BGRA-to-RGBA conversion before upload:** Would allow using `Rgba8Unorm` format directly on platforms that produce BGRA. Decision: not needed for E02 (xcap on X11 already produces RGBA). For future BGRA backends: rejected per doc-03 Section 4.3. CPU conversion is expensive for large buffers; GPU swizzle is free (single instruction per pixel in the shader).

3. **`Rgba8Unorm` (non-sRGB) format with manual gamma conversion in shader:** Would give explicit control over the sRGB decode. Decision: rejected per doc-03 Section 5.4. `Rgba8UnormSrgb` achieves the same result with zero performance cost and no shader complexity.

4. **Separate upload queue/thread:** Would decouple upload from rendering. Decision: deferred. The Phase 0 sequential pipeline does not benefit from async upload because the render pass must wait for the upload to complete anyway.
