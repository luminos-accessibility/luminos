# Story E02/004: Magnification Shaders & Viewport

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** 002

---

## Problem Statement

The core of Luminos is the GPU magnification pipeline: a captured screen region must be scaled up to fill the overlay viewport using high-quality interpolation, producing a smooth magnified view at zoom levels from 1.5x to 20x. Without magnification shaders, the captured screen content is just raw pixels sitting in a GPU texture with no way to produce the magnified output the user sees.

This story implements the WGSL magnification shaders (bilinear and bicubic Catmull-Rom variants), the viewport calculation logic that determines which region of the screen to capture at a given zoom level, and the wgpu render pipeline infrastructure to compile and execute these shaders. Together, these components transform a source texture containing captured screen pixels into a magnified full-screen view on the overlay window.

## User Scenarios

### US-1: Magnified View at Configurable Zoom Levels

As a low-vision user, I want my screen content magnified at zoom levels from 1.5x to 20x so that I can read text and see details that are otherwise too small.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given a source texture containing captured screen pixels and a zoom level of 2x, when the bilinear magnification shader executes, then the output fills the overlay viewport with a 2x magnified view of the source content.
- **AC-1.2:** Given a source texture and zoom levels of 1.5x, 5x, 10x, and 20x, when the magnification shader executes at each level, then the output is correctly magnified without visible artifacts (no black regions, no distortion, no UV wrapping).
- **AC-1.3:** Given a zoom level and a viewport size, when `compute_source_region()` is called, then the returned `ScreenRect` has dimensions equal to `ceil(viewport_size / zoom_level)` and is centered on the tracking target.
- **AC-1.4:** Given a tracking target near a screen edge, when `compute_source_region()` is called, then the source region is clamped to stay within screen bounds (no negative coordinates, no extending beyond screen dimensions).

### US-2: High-Quality Bicubic Interpolation

As a low-vision user reading text at high zoom (10x-20x), I want bicubic interpolation available so that text edges are sharp and smooth rather than blocky.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given a source texture with sharp edges (e.g., black text on white background), when the bicubic (Catmull-Rom) shader executes at 10x zoom, then the output edges are smoother than bilinear interpolation output (verified by shader output comparison test).
- **AC-2.2:** Given the bicubic shader, when it samples the source texture, then it performs 16 texture lookups per output pixel (4x4 tap pattern) using the Catmull-Rom weight function with a = -0.5.
- **AC-2.3:** Given both bilinear and bicubic shader variants, when render pipelines are created, then both compile successfully and can be selected at pipeline initialization time.

### US-3: BGRA/RGBA Platform Pixel Format Handling

As a rendering pipeline component receiving frames from different platforms, I want the shader to handle both BGRA and RGBA pixel formats so that colors are displayed correctly regardless of capture source.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given a source texture with BGRA pixel data and `is_bgra` uniform set to 1.0, when the magnification shader executes, then the output has correct RGB channel ordering (red displays as red, not blue).
- **AC-3.2:** Given a source texture with RGBA pixel data and `is_bgra` uniform set to 0.0, when the magnification shader executes, then the output has correct RGB channel ordering (no swizzle applied).

**Note:** The `xcap` crate returns RGBA on X11 (not BGRA as stated in doc-03 Section 4.3). Therefore, the E02 default path on X11 uses `is_bgra = 0.0` (no swizzle). The BGRA swizzle path is still required for future Windows DXGI backends, which return BGRA. Both paths must be tested.

### US-4: Shader Pipeline Infrastructure

As a rendering pipeline developer, I want the wgpu render pipeline, bind group layout, and uniform buffer set up correctly so that shaders can be compiled once at startup and executed efficiently every frame.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given the bilinear shader WGSL source, when `device.create_shader_module()` is called, then the shader compiles without errors on both Vulkan and GL backends.
- **AC-4.2:** Given the bicubic shader WGSL source, when `device.create_shader_module()` is called, then the shader compiles without errors on both Vulkan and GL backends.
- **AC-4.3:** Given a compiled shader module, when a `RenderPipeline` is created with the magnification bind group layout, then pipeline creation succeeds.
- **AC-4.4:** Given the `MagnifyUniforms` struct, when written to a wgpu buffer, then the struct has correct 16-byte alignment and all fields are at expected byte offsets.

### US-5: Full-Screen Triangle Vertex Shader

As a GPU rendering component, I want a full-screen triangle vertex shader that covers the entire viewport without requiring a vertex buffer so that the fragment shader processes every pixel in the output.

**Priority:** P0
**Acceptance Criteria:**

- **AC-5.1:** Given a draw call with 3 vertices and no vertex buffer, when the vertex shader executes, then the output triangle covers the entire viewport (all pixels receive a fragment shader invocation).
- **AC-5.2:** Given the vertex shader output, when the UV coordinates are inspected, then they map correctly to the source texture (0,0 at top-left, 1,1 at bottom-right, with Y-flip for top-left origin textures).

## Functional Requirements

- **FR-1:** Create `crates/luminos-gpu/src/shaders/magnify_bilinear.wgsl` with a bilinear magnification fragment shader using a single `textureSampleLevel` call and a full-screen triangle vertex shader. *(Traced by AC-1.1, AC-1.2, AC-5.1, AC-5.2)*
- **FR-2:** Create `crates/luminos-gpu/src/shaders/magnify_bicubic.wgsl` with a bicubic Catmull-Rom fragment shader performing 16-tap interpolation per doc-03 Section 6.2. *(Traced by AC-2.1, AC-2.2)*
- **FR-3:** Both shaders must accept a `MagnifyUniforms` uniform buffer containing `viewport_size: vec2f`, `source_size: vec2f`, `is_bgra: f32`, and padding for 16-byte alignment. *(Traced by AC-3.1, AC-3.2, AC-4.4)*
- **FR-4:** Both shaders must implement BGRA channel swizzle controlled by the `is_bgra` uniform flag. *(Traced by AC-3.1, AC-3.2)*
- **FR-5:** Create `crates/luminos-gpu/src/shaders/mod.rs` with functions to compile both shader variants and create `RenderPipeline` objects. *(Traced by AC-4.1, AC-4.2, AC-4.3)*
- **FR-6:** Define the `MagnifyUniforms` Rust struct with `#[repr(C)]` for GPU buffer compatibility. *(Traced by AC-4.4)*
- **FR-7:** Define the bind group layout: source texture (binding 0) + sampler (binding 1) + uniform buffer (binding 2). *(Traced by AC-4.3)*
- **FR-8:** Create `crates/luminos-gpu/src/viewport.rs` with `compute_source_region()` per doc-03 Section 3.1. *(Traced by AC-1.3, AC-1.4)*
- **FR-9:** `compute_source_region()` must clamp the source region to screen bounds, preventing negative coordinates or regions extending beyond the display. *(Traced by AC-1.4)*
- **FR-10:** Create `smooth_viewport_position()` function for smooth panning interpolation per doc-03 Section 3.3. *(Traced by AC-1.3)*

## Non-Functional Requirements

- **NFR-1:** Shader execution must complete within 2ms on integrated GPUs (within the shader stage budget from doc-03 Section 2.3).
- **NFR-2:** Bicubic shader must not exceed the texture sampling throughput of Intel UHD 620-class GPUs at 1080p output resolution.
- **NFR-3:** No `unwrap()` or `expect()` in production code paths. `unwrap()` is acceptable in `#[cfg(test)]` blocks.
- **NFR-4:** All public APIs must have `///` doc-comments.
- **NFR-5:** `cargo clippy -p luminos-gpu -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` must pass.
- **NFR-6:** `compute_source_region()` must be pure arithmetic (no GPU, no I/O, no allocations) for sub-microsecond execution.

## Out of Scope

- Screen capture implementation (Story 001).
- GPU texture upload and double buffering (Story 003).
- Render loop, frame pacing, and vsync (Story 005).
- Color filter shader (`color_filter.wgsl`) -- Epic 6.
- Cursor overlay shader (`cursor.wgsl`) -- Epic 6.
- Lens mode and docked mode viewport adjustments -- Epic 5.
- Smooth panning with easing function (smooth_viewport_position is provided but integration into the render loop is Story 005/E03).
- Anti-aliasing (MSAA, FXAA) -- Phase 1.
- Font re-rendering -- Phase 3.

## Open Questions

*None -- all questions resolved by doc-03 rendering pipeline specification.*
