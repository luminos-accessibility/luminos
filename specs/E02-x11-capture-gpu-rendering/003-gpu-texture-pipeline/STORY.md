# Story E02/003: GPU Texture Pipeline

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** 002 (wgpu device and queue must be initialized)

---

## Problem Statement

The rendering pipeline has two halves: CPU-side screen capture (Story 001) and GPU-side shader rendering (Story 004). Between them sits the GPU texture pipeline -- the bridge that transfers captured screen pixels from CPU memory to GPU textures where shaders can consume them. Without this bridge, captured `CaptureFrame` data sits in system memory with no path to the GPU, and the magnification shaders have no source texture to sample from.

This story implements the `SourceTextureManager` in the `luminos-gpu` crate, responsible for GPU texture lifecycle management: creating source textures in `Rgba8UnormSrgb` format for gamma-correct rendering, uploading `CaptureFrame` pixel data via `wgpu::Queue::write_texture()`, over-allocating textures by 1.5x to minimize reallocation frequency, and falling back to stale frames when capture fails. The design follows doc-03 Sections 5.1-5.4 precisely.

Single-buffer texture upload is used because the E02 rendering pipeline is sequential: upload completes before the render pass begins within the same frame, so there is no concurrent read/write and no visible tearing. Double-buffered texture swap (write-to-back, read-from-front) is a Phase 1 optimization for when capture and render are pipelined across separate threads.

## User Scenarios

### US-1: Texture Upload from CaptureFrame

As the rendering pipeline, I want captured screen pixels uploaded to a GPU texture every frame so that the magnification shader can sample them for rendering.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given a `CaptureFrame` with known dimensions and pixel data, when `SourceTextureManager::upload()` is called, then the pixel data is transferred to the GPU source texture via `wgpu::Queue::write_texture()` and the texture is available for shader sampling.
- **AC-1.2:** Given a `CaptureFrame` with `format: PixelFormat::Rgba8` (from xcap on X11), when uploaded to the source texture, then the texture uses `Rgba8UnormSrgb` format (RGBA data maps directly to the texture format with no channel reordering needed). For future platform backends that produce BGRA (e.g., Windows DXGI), the BGRA-to-RGBA conversion is handled by the shader, not the texture pipeline.
- **AC-1.3:** Given a `CaptureFrame` with stride padding (i.e., `stride > width * 4`), when uploaded, then the texture data correctly accounts for the row padding via `TexelCopyBufferLayout::bytes_per_row`.

### US-2: Texture Over-Allocation and Reallocation

As the rendering pipeline, I want the source texture to be over-allocated so that zoom level changes do not trigger a GPU texture reallocation every frame.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given initial texture allocation for a 960x540 source region, when `SourceTextureManager::new()` is called, then the allocated texture capacity is at least 1440x810 (1.5x over-allocation in each dimension).
- **AC-2.2:** Given a texture allocated with 1440x810 capacity, when a `CaptureFrame` of 1280x720 is uploaded (fits within capacity), then no reallocation occurs.
- **AC-2.3:** Given a texture allocated with 1440x810 capacity, when a `CaptureFrame` of 1920x1080 is uploaded (exceeds capacity), then the texture is reallocated to at least 2880x1620 (1.5x of the new dimensions).
- **AC-2.4:** Given a texture reallocation, when the new texture is created, then the old texture is dropped and the new texture is immediately available for upload.

### US-3: Stale Frame Fallback

As the rendering pipeline, I want the last successfully uploaded texture to be preserved when capture fails so that the user sees the previous frame instead of a blank screen.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given a successful upload followed by a capture failure, when the renderer requests the source texture, then the texture from the last successful upload is returned (stale frame).
- **AC-3.2:** Given 60 consecutive capture failures (1 second at 60fps), when the stale frame count reaches the threshold, then a `warn!` log message is emitted indicating prolonged capture failure.
- **AC-3.3:** Given a stale frame situation that recovers (capture succeeds again), when the new frame is uploaded, then the stale frame counter resets to zero.

### US-4: Texture View for Shader Binding

As the magnification shader, I want to obtain a `TextureView` from the source texture so that I can bind it to the shader's texture input.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given a `SourceTextureManager` with an uploaded texture, when `texture_view()` is called, then it returns a `wgpu::TextureView` suitable for binding to a shader's `texture_2d<f32>` input.
- **AC-4.2:** Given a `SourceTextureManager`, when `current_dimensions()` is called, then it returns the `(width, height)` of the most recently uploaded frame (not the over-allocated texture capacity), which the shader uses as `source_size` in the `MagnifyUniforms`.

### US-5: sRGB-Correct Texture Format

As the rendering pipeline, I want the source texture to use `Rgba8UnormSrgb` format so that wgpu automatically performs sRGB-to-linear conversion when the shader samples it, producing gamma-correct interpolation.

**Priority:** P0
**Acceptance Criteria:**

- **AC-5.1:** Given a newly created or reallocated source texture, when inspected, then its format is `wgpu::TextureFormat::Rgba8UnormSrgb`.
- **AC-5.2:** Given a source texture with sRGB format and known pixel values, when the magnification shader samples it, then the read values are in linear color space (automatic sRGB decode by wgpu).

## Functional Requirements

- **FR-1:** Implement `SourceTextureManager` struct in `crates/luminos-gpu/src/texture.rs` managing the GPU source texture lifecycle. *(Traced by US-1, US-2, US-3, US-4)*
- **FR-2:** Implement `SourceTextureManager::new(device, initial_width, initial_height)` constructor that creates an over-allocated GPU texture in `Rgba8UnormSrgb` format. *(Traced by AC-2.1, AC-5.1)*
- **FR-3:** Implement `SourceTextureManager::upload(queue, frame)` method that transfers `CaptureFrame` pixel data to the GPU texture via `Queue::write_texture()`, handling stride padding. *(Traced by AC-1.1, AC-1.3)*
- **FR-4:** Implement automatic texture reallocation when uploaded frame dimensions exceed current texture capacity, with 1.5x over-allocation. *(Traced by AC-2.2, AC-2.3, AC-2.4)*
- **FR-5:** Implement `SourceTextureManager::record_capture_failure()` method that increments the stale frame counter and emits `warn!` at the 60-frame threshold. *(Traced by AC-3.1, AC-3.2, AC-3.3)*
- **FR-6:** Implement `SourceTextureManager::texture_view()` method returning a `wgpu::TextureView` for shader binding. *(Traced by AC-4.1)*
- **FR-7:** Implement `SourceTextureManager::current_dimensions()` returning `(u32, u32)` of the last uploaded frame. *(Traced by AC-4.2)*
- **FR-8:** Use `Rgba8UnormSrgb` texture format for all source textures to enable automatic sRGB-to-linear conversion on shader read. *(Traced by AC-5.1, AC-5.2)*

## Non-Functional Requirements

- **NFR-1:** Texture upload must complete in under 2ms for typical source regions (up to 960x540 at 2x zoom on 1080p) per doc-03 Section 2.3 performance budget.
- **NFR-2:** No `unwrap()` or `expect()` in production code paths. `unwrap()` is acceptable in `#[cfg(test)]` blocks.
- **NFR-3:** GPU memory usage for source texture must stay within the 100MB rendering budget from doc-03 Section 1.3. At 1080p with 1.5x over-allocation: 1920*1.5 * 1080*1.5 * 4 bytes = ~17.9MB, well within budget.
- **NFR-4:** All public items must have `///` doc-comments. `cargo doc -p luminos-gpu --no-deps` must produce documentation without warnings.
- **NFR-5:** `cargo clippy -p luminos-gpu -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` must pass.

## Out of Scope

- Intermediate textures for multi-pass rendering (ping-pong buffers). These are created when color filters or cursor overlay are implemented (E06).
- Cursor texture management (E06).
- Dirty-region tracking (Phase 1+, see doc-03 Section 10.3).
- GPU texture sharing / DMA-BUF zero-copy (Phase 2+, see doc-03 Section 10.4).
- BGRA-to-RGBA pixel format conversion (the `is_bgra` shader uniform from Story 004 handles this for future platform backends like Windows DXGI; xcap on X11 already produces RGBA, so no conversion is needed for the E02 target).
- Render pipeline or shader creation (Story 004).
- Frame pacing or present mode (Story 005).

## Open Questions

*None -- all design decisions resolved via doc-03 Sections 5.1-5.4 and HIGH_LEVEL_PLAN.md architecture decisions.*
