# Epic E02: X11 Screen Capture & GPU Magnification

**Status:** IN PROGRESS
**Roadmap Ref:** [tech-strategy/09-implementation-roadmap.md Section 4.2](../tech-strategy/09-implementation-roadmap.md#42-epic-2----x11-screen-capture--gpu-magnification)
**Phase:** Phase 0: Foundation (Months 1-3)
**Started:** ---
**Completed:** ---
**Hard Dependencies:** E1 (Project Scaffolding, Platform Traits & CI/CD) -- DONE 2026-03-28
**Soft Dependencies:** None
**Primary Docs:** [02 -- Platform Abstraction](../tech-strategy/02-platform-abstraction.md) Section 8.1, [03 -- Rendering Pipeline](../tech-strategy/03-rendering-pipeline.md) Sections 2-10, [07 -- Testing Strategy](../tech-strategy/07-testing-strategy.md) Sections 4.5 and 10

---

## Overview

Implement the end-to-end rendering pipeline on Linux X11: capture screen content via `xcap`, upload it to a GPU texture, apply magnification via WGSL shaders (bilinear and bicubic Catmull-Rom variants), and present the result in a transparent always-on-top winit overlay window at 60fps. This is the first visual proof that the architecture works -- a user sees magnified screen content rendered in real-time.

This epic delivers user-perceivable value for the first time: a user launches Luminos on a Linux X11 desktop and sees their screen magnified at a chosen zoom level (1.5x-20x), rendered smoothly at 60fps in a transparent overlay. The overlay excludes itself from capture via self-capture prevention (RISK-002), preventing infinite feedback loops. The epic also establishes the GPU pipeline infrastructure (texture management, shader compilation, frame pacing) that all subsequent rendering work builds upon.

## Success Criteria

Copied from [doc-09 Section 4.2](../tech-strategy/09-implementation-roadmap.md#42-epic-2----x11-screen-capture--gpu-magnification):

- [ ] `cargo nextest run -p luminos-platform --test x11_capture` passes on Xvfb
- [ ] `cargo nextest run -p luminos-gpu` passes (shader compilation, texture upload)
- [ ] Frame time P99 < 20ms at 2x zoom on CI runner (Mesa llvmpipe)
- [ ] Overlay window is transparent and always-on-top on at least two X11 WMs
- [ ] Zoom levels 1.5x, 5x, 10x, 20x all render without artifacts

---

## Story Breakdown

### Progress Summary

| # | Story | Status | Depends On | Est. Effort | Notes |
|---|-------|--------|------------|-------------|-------|
| 001 | X11 Screen Capture Backend | DONE | --- | L (12-16 subtasks) | Parallel with 002. Covers D1. |
| 002 | X11 Overlay Window & GPU Surface | DONE | --- | L (14 subtasks) | Parallel with 001. Covers D2. Type unification done via luminos-types crate. |
| 003 | GPU Texture Pipeline | DONE | 002 | M (11 subtasks) | Needs wgpu device/surface from 002. Covers D5. |
| 004 | Magnification Shaders & Viewport | DONE | 002 | L (14 subtasks) | Needs wgpu device for shader compilation. Covers D3. |
| 005 | Render Loop, Frame Pacing & CI | NOT STARTED | 001, 003, 004 | L (12-16 subtasks) | Assembles full pipeline. Covers D4. |

**Total Stories:** 5 | **Done:** 4 | **In Progress:** 0 | **Blocked:** 0

**Dependency graph:**

```
001 X11 Capture ──────────────────────────────────┐
                                                  │
002 Overlay + GPU ──┬──> 003 Texture Pipeline ────┼──> 005 Render Loop & CI
                    │                             │
                    └──> 004 Shaders & Viewport ──┘
```

Stories 001 and 002 can execute in parallel (no internal dependencies). Stories 003 and 004 can execute in parallel once 002 is complete. Story 005 depends on 001, 003, and 004 (it assembles the full capture-to-present pipeline).

### Deliverable Traceability

Every roadmap deliverable (D1-D5) and success criterion (SC1-SC5) maps to at least one story:

| Deliverable | Description | Story |
|-------------|-------------|-------|
| D1 | `ScreenCapture` impl captures full-screen content on X11 | 001 |
| D2 | Winit overlay window renders transparently on top of other windows | 002 |
| D3 | Captured content displayed magnified at configurable zoom level (1.5x-20x) | 004 |
| D4 | Frame rate sustains 60fps at 2x zoom on integrated GPU | 005 |
| D5 | Double-buffered texture upload prevents visible tearing | 003 |

| Success Criterion | Story |
|-------------------|-------|
| SC1: `cargo nextest run -p luminos-platform --test x11_capture` passes on Xvfb | 001, 005 |
| SC2: `cargo nextest run -p luminos-gpu` passes (shader compilation, texture upload) | 003, 004, 005 |
| SC3: Frame time P99 < 20ms at 2x zoom on CI runner | 005 |
| SC4: Overlay transparent and always-on-top on at least two X11 WMs | 002 |
| SC5: Zoom levels 1.5x, 5x, 10x, 20x render without artifacts | 004, 005 |

### Story Descriptions

#### 001 -- X11 Screen Capture Backend

**Scope:** Implement the `ScreenCapture` trait for Linux X11 using the `xcap` crate. The implementation supports display enumeration, full-screen and region capture, and self-capture prevention (RISK-002) by excluding the overlay window from captured frames via an unmap/remap cycle. Tests run on Xvfb in CI. **Important:** xcap 0.9.3 returns RGBA pixel data on X11 (it converts internally from X11's native BGRA), so the `CaptureFrame::format` will be `PixelFormat::Rgba8`, not `Bgra8` as the tech strategy assumed.

**Key Deliverables:**
- `crates/luminos-platform/src/linux_x11/capture.rs` containing `XcbCapture` struct implementing `ScreenCapture`
- `XcbCapture::new()` constructor; self-capture exclusion configured via `set_excluded_windows()` (see trait modification below)
- `list_displays()` implementation returning X11 screen information via xcap
- `capture_frame()` implementation supporting both full-display and region-specific capture
- `subscribe_display_changes()` implementation via `x11rb` RandR event subscription (xcap does NOT provide display change events -- must be implemented separately)
- Self-capture prevention (RISK-002): **unmap/remap cycle** around each `capture_frame()` call. The overlay window is unmapped before capture and remapped after. This is the only reliable mechanism on X11 -- composite pixmap capture does NOT exclude override-redirect windows (compositors composite all visible windows). Fallback: software watermark detection.
- **`ScreenCapture` trait modification:** Add `set_excluded_windows(&mut self, window_ids: &[u64])` method to the `ScreenCapture` trait in `luminos-platform`. This is a breaking change to the E01 trait definition. The method accepts X11 window IDs (or platform-native identifiers on other platforms) and configures the capture backend to exclude those windows. Default implementation is a no-op.
- Unit tests: display enumeration, region boundary validation, pixel format verification (RGBA from xcap), error cases
- Integration tests gated behind `#[cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]` running on Xvfb: actual capture produces non-zero pixel data, capture dimensions match requested region
- Self-capture prevention integration test: render a known solid-color overlay, capture a frame, verify captured frame does NOT contain the overlay color (requires a compositor like picom running under Xvfb for realistic testing)

**Estimated Effort:** L (12-16 subtasks)

**Notes:** This story has no internal E02 dependencies (can start immediately). The `ScreenCapture` trait and all common types (`CaptureFrame`, `DisplayInfo`, `ScreenRect`, `PixelFormat`, etc.) are already defined in `luminos-platform` from E01. xcap v0.9.3 is already declared as a workspace dependency. **Critical xcap behaviors discovered by research:** (1) xcap creates a new XCB connection per `capture_frame()` call -- not pooled. This is a performance concern at 60fps and must be benchmarked. Connection pooling or caching may be needed. (2) xcap does not support display change event subscription. (3) xcap returns RGBA, not BGRA (internal conversion). The `tokio` dev-dependency may need expanded features (`rt`, `macros`) for async tests involving `subscribe_display_changes()`. New workspace dependency: `x11rb` may need the `randr` feature for display change events (already in workspace deps). The `ci_platform_tests` feature flag must be added to `luminos-platform/Cargo.toml`.

---

#### 002 -- X11 Overlay Window & GPU Surface

**Scope:** Implement the `WindowManager` trait for Linux X11 using winit. Create a transparent, always-on-top, borderless overlay window suitable for GPU rendering. Initialize the wgpu device and surface (Vulkan backend) on this window. Unify the `DockEdge` and `LensShape` type definitions between `luminos-platform` and `luminos-core` to eliminate the duplication discovered in E01.

**Key Deliverables:**
- `crates/luminos-platform/src/linux_x11/window.rs` containing `X11WindowManager` struct implementing `WindowManager`
- `X11WindowManager::new()` constructor creating a winit `Window` with: transparent, borderless/undecorated, always-on-top attributes
- `create_overlay()` implementation targeting a specific display
- `set_overlay_bounds()`, `set_always_on_top()`, `set_visible()` implementations
- `set_overlay_mode()` implementation supporting `OverlayMode::FullScreen` only (lens and docked modes are out of scope for E02 -- they are E05)
- `raw_window_handle()` and `raw_display_handle()` implementations returning handles for wgpu surface creation
- wgpu initialization module in `crates/luminos-gpu/src/device.rs`:
  - `create_gpu_device()` async function (Vulkan backend, `LowPower` preference, `downlevel_webgl2_defaults`)
  - `configure_surface()` function (sRGB format preference, `Fifo` default present mode)
  - **`CompositeAlphaMode` fallback chain:** Query `surface.get_capabilities(adapter).alpha_modes` and select in priority order: `PreMultiplied` → `PostMultiplied` → `Opaque`. Driver support varies -- do NOT hardcode `PreMultiplied`.
- `DockEdge` and `LensShape` type unification: move canonical definitions to `luminos-platform::traits::window_manager` (add `Serialize`/`Deserialize` derives), re-export from `luminos-core::config::schema`, remove duplicate definitions
- Unit tests: window creation attributes, handle validity, wgpu device creation (on Mesa llvmpipe), surface configuration
- Integration tests on Xvfb: overlay is visible, transparency works, always-on-top is set

**Estimated Effort:** L (12-16 subtasks)

**Notes:** This story has no internal E02 dependencies (can start immediately, parallel with 001). winit 0.30.13 and wgpu 28.0.0 are already workspace dependencies. The `DockEdge`/`LensShape` duplication was discovered in E01 (see E01 Shared Context): `luminos-platform::traits::window_manager` defines them without serde, `luminos-core::config::schema` defines them with serde. Unification adds `serde` derives to the platform definitions and makes `luminos-core` re-export them. This requires adding `serde` as a dependency of `luminos-platform` (gated behind a `serde` feature, default-enabled). The `x11rb` crate may be needed for setting raw X11 properties not exposed by winit (e.g., `_NET_WM_WINDOW_TYPE`). New workspace dependencies: possibly `x11rb` (already declared from E01 workspace deps).

---

#### 003 -- GPU Texture Pipeline

**Scope:** Implement the GPU texture management layer that transfers `CaptureFrame` CPU pixel data to GPU textures for shader consumption. This includes single-buffer texture upload (sequential pipeline), source texture over-allocation (1.5x) with reallocation on resize, sRGB format handling, and stale frame fallback when capture fails.

**Key Deliverables:**
- `crates/luminos-gpu/src/texture.rs` containing:
  - `SourceTextureManager` struct managing source texture lifecycle (creation, reallocation)
  - `upload_capture_frame()` function transferring `CaptureFrame` data to GPU texture via `Queue::write_texture()`
  - Texture over-allocation strategy: allocate 1.5x source region dimensions, reallocate only when capture exceeds capacity
  - Stride handling: respect `CaptureFrame::stride` for row padding
  - sRGB texture format: `Rgba8UnormSrgb` for gamma-correct interpolation in shaders
- Stale frame fallback: when `capture_frame()` returns an error, skip upload and render from the existing texture. Track consecutive stale frame count; log `warn!` after 60 consecutive stale frames (1 second at 60fps)
- Unit tests: texture creation at various dimensions, reallocation triggers, stride padding handling, stale frame counter behavior
- Integration tests (wgpu on Mesa llvmpipe): upload a `generate_test_capture_frame()` to GPU, read back pixels, verify correctness

**Estimated Effort:** M (8-12 subtasks)

**Notes:** Depends on Story 002 (needs the wgpu `Device` and `Queue` from GPU initialization). The `CaptureFrame` struct and `PixelFormat` enum are defined in `luminos-platform::traits::screen_capture`. Channel reordering (BGRA→RGBA) is NOT handled here -- it is done in the magnification shader via a uniform flag (zero-cost GPU swizzle per doc-03 Section 4.3). The texture format is `Rgba8UnormSrgb` regardless of input pixel format. **xcap note:** Since xcap 0.9.3 returns RGBA on X11, no swizzle is needed for E02. The texture upload path should still use `CaptureFrame::format` to set the upload format correctly and remain backend-agnostic.

---

#### 004 -- Magnification Shaders & Viewport

**Scope:** Implement the WGSL magnification shaders (bilinear and bicubic Catmull-Rom variants) and the viewport calculation logic. The shaders read from the source texture, apply interpolation, handle BGRA/RGBA channel swizzle via a uniform, and write the magnified result. The viewport calculator computes the source region from a tracking target position and zoom level. Full-screen zoom mode (1.5x-20x) is implemented.

**Key Deliverables:**
- `crates/luminos-gpu/src/shaders/magnify_bilinear.wgsl`: bilinear magnification shader (single `textureSampleLevel` call, full-screen triangle vertex shader)
- `crates/luminos-gpu/src/shaders/magnify_bicubic.wgsl`: bicubic Catmull-Rom magnification shader (4x4 tap, 16 texture lookups, `cubic_weight` function per doc-03 Section 6.2)
- `crates/luminos-gpu/src/shaders/mod.rs`: shader module loading and pipeline creation (compile both shader variants at initialization, store `RenderPipeline` objects)
- `MagnifyUniforms` struct: `viewport_size`, `source_size`, `is_bgra` flag, padding for 16-byte alignment
- Bind group layout: source texture + sampler + uniform buffer
- `crates/luminos-gpu/src/viewport.rs`: `compute_source_region()` function per doc-03 Section 3.1 (center on tracking target, clamp to screen bounds)
- `crates/luminos-core/src/viewport.rs` (or in `luminos-gpu`): viewport calculation is pure arithmetic, testable without GPU
- Unit tests: `compute_source_region()` at various zoom levels (1.5x, 2x, 5x, 10x, 20x), edge clamping at screen boundaries, correct source region dimensions
- Shader compilation tests (wgpu on Mesa llvmpipe): both shaders compile without errors, render pipeline creation succeeds
- Shader output tests: render a known solid-color source texture through the bilinear shader, verify output matches expected color (including BGRA swizzle)

**Estimated Effort:** L (12-16 subtasks)

**Notes:** Depends on Story 002 (needs wgpu device for shader compilation and render pipeline creation). The shader code follows doc-03 Section 6.2. The bilinear shader is the Phase 0 default; the bicubic shader is provided simultaneously (confirmed user decision) and can be selected at pipeline creation. Both shaders share the same vertex shader (full-screen triangle from `vertex_index`), uniform layout, and bind group layout -- they differ only in the fragment shader sampling function. The `compute_source_region()` function is pure arithmetic and can be unit-tested without any GPU or windowing dependency. Color filter and cursor overlay shaders are excluded (Epic 6). **xcap pixel format note:** Since xcap 0.9.3 returns RGBA on X11 (not BGRA as the tech strategy assumed), the `is_bgra` uniform will be `0.0` (no swizzle) when using xcap. The swizzle logic must still be implemented and tested for correctness -- it will be needed by future platform backends (Windows DXGI returns BGRA) and if the capture backend is replaced with a direct XShm implementation in Phase 1 (which returns native BGRA).

---

#### 005 -- Render Loop, Frame Pacing & CI

**Scope:** Assemble the full rendering pipeline: capture screen → upload texture → execute shader → present frame. Implement the render loop driven by winit's event loop with vsync (Fifo present mode). Add `FrameTimings` ring buffer for performance monitoring. Update CI to support Xvfb and Mesa llvmpipe for headless X11 and GPU testing. Write the integration test that validates the complete capture-to-present pipeline.

**Key Deliverables:**
- `crates/luminos-gpu/src/renderer.rs` containing the `Renderer` struct:
  - Holds persistent GPU resources: device, queue, render pipelines (bilinear/bicubic), bind group layouts, source texture manager, frame timings
  - `render_frame()` method: execute the single-pass magnification pipeline (source texture → magnify shader → swap chain surface)
  - Shader variant selection: choose bilinear or bicubic pipeline at construction time (configurable)
  - Stale frame awareness: skip texture upload on capture failure, render from existing texture
- `crates/luminos-gpu/src/frame_timings.rs` containing the `FrameTimings` struct:
  - Circular buffer of last 120 frame times (2 seconds at 60fps)
  - `record()`, `p99()`, `average()`, `min()`, `max()`, `summary()` methods per doc-03 Section 8.3
  - Performance degradation detection: warn threshold (P99 > 20ms for 5s), critical threshold (P99 > 33ms for 5s)
- Render loop integration (in `luminos-app` or a dedicated binary for E02 testing):
  - winit event loop driving the capture → upload → render → present cycle
  - `PresentMode::Fifo` for vsync (default)
  - Graceful handling of window resize (reconfigure surface, reallocate textures)
- CI pipeline additions to `.github/workflows/ci.yml`:
  - Install Xvfb, Mesa lavapipe (`mesa-vulkan-drivers` for Vulkan software rendering), and Mesa llvmpipe (`mesa-utils`, `libegl-dev`, `libgl-dev` for GL fallback) on Linux CI runner
  - Install `picom` compositor for realistic self-capture prevention testing under Xvfb (the unmap/remap RISK-002 mitigation requires a compositor to be meaningful)
  - Run X11 platform tests under Xvfb: `xvfb-run cargo nextest run -p luminos-platform --features ci_platform_tests`
  - Run GPU tests under Xvfb: `xvfb-run cargo nextest run -p luminos-gpu`
  - Shader compilation validation as part of luminos-gpu tests
- Integration test: `render_pipeline_capture_to_present`:
  - On Xvfb: create overlay window, capture frame, upload to GPU, render through magnification shader, present
  - Verify: output frame has non-zero pixel data, frame dimensions match overlay size, frame timing is recorded
  - Verify: P99 frame time < 20ms at 2x zoom on CI runner (Mesa llvmpipe)
- Unit tests: `FrameTimings` ring buffer behavior (record, p99 calculation, min/max, summary, degradation thresholds)

**Estimated Effort:** L (12-16 subtasks)

**Notes:** Depends on Stories 001 (screen capture), 003 (texture pipeline), and 004 (shaders and viewport). This story assembles all the pieces into the working pipeline. The render loop is NOT the final application event loop (that is E3/E4 with `EventLoopProxy` and Tauri integration) -- it is a standalone loop for E02 that demonstrates the capture-to-present pipeline. The `FrameTimings` struct is `pub(crate)` within `luminos-gpu`; the `FrameTimingSummary` for IPC is defined in doc-05 Section 3.4 and exposed later (E4/E5). The CI additions build on the existing GitHub Actions workflow from E01 Story 005. The `libegl-dev` package is already added to CI (see commit `da0185f`).

---

## Shared Context

This section contains cross-cutting knowledge that applies to all stories in this epic. Agents working on any story should read this section. Update it as stories are completed and new knowledge emerges.

### Architecture Decisions

These decisions are drawn from the tech strategy and apply across all E02 stories:

- **Dual-window architecture (overlay is NOT a webview):** The magnification overlay is a native winit+wgpu window, completely separate from the Tauri webview control panel. The overlay bypasses the webview entirely for performance reasons. See [doc-01 Section 6.5](../tech-strategy/01-system-architecture.md#65-event-loop-integration).

- **Five-stage rendering pipeline:** Viewport → Capture → Upload → Render → Present. Executed once per frame at up to 60fps. The pipeline is sequential within a frame. See [doc-03 Section 2.1](../tech-strategy/03-rendering-pipeline.md#21-pipeline-stages).

- **Synchronous capture, GPU-accelerated rendering:** `ScreenCapture::capture_frame()` is synchronous and blocking (target: <8ms). GPU rendering is the fast path. The hot loop is: call capture (blocking) → upload to GPU → execute shader → present. No async runtime in the render thread. See [doc-02 Section 2.3](../tech-strategy/02-platform-abstraction.md#23-async-where-needed-sync-by-default).

- **sRGB-correct interpolation via texture formats:** Source textures use `Rgba8UnormSrgb` so that wgpu automatically linearizes on shader read and re-encodes on surface write. This produces gamma-correct interpolation without manual conversion. See [doc-03 Section 5.4](../tech-strategy/03-rendering-pipeline.md#54-srgb-handling).

- **Pixel format swizzle in shader, not CPU:** Platform pixel format differences are handled by a uniform flag (`is_bgra`) in the magnification shader. One GPU instruction per pixel, effectively free. No CPU-side format conversion. See [doc-03 Section 4.3](../tech-strategy/03-rendering-pipeline.md#43-platform-pixel-format-handling). **Research correction:** xcap 0.9.3 returns RGBA on all platforms (it converts internally). On X11 with xcap, the `is_bgra` flag is `0.0` (no swizzle needed). The swizzle remains necessary for future backends (direct XShm returns native BGRA, Windows DXGI returns BGRA). The tech strategy's claim that "X11 native format is BGRA" is correct for X11 itself, but xcap abstracts this away.

- **Self-capture prevention via unmap/remap cycle (RISK-002):** The overlay window is unmapped before each `capture_frame()` call and remapped after. This is the only reliable self-capture prevention mechanism on X11. **Research correction:** The composite pixmap approach described in doc-03 Section 7.1 and the risk register is INCORRECT -- compositors composite ALL visible windows into the root pixmap, including override-redirect windows. The unmap/remap approach introduces a potential flicker risk, but at 60fps the unmap duration is sub-millisecond (within a single frame's capture call). The `ScreenCapture` trait will be extended with `set_excluded_windows(&mut self, window_ids: &[u64])` to configure exclusion (breaking change to E01 trait). See [doc-03 Section 7.1](../tech-strategy/03-rendering-pipeline.md#71-full-screen-mode) and RISK-002.

- **Both bilinear and bicubic shader variants in E02:** The original roadmap placed bilinear in Phase 0 and bicubic in Phase 1. User decision: include both in E02. Bilinear is the default; bicubic (Catmull-Rom, 16 taps) is available for sharper text at high zoom. Selection is at pipeline initialization time. See [doc-03 Section 6.2](../tech-strategy/03-rendering-pipeline.md#62-magnification-shader-magnifywgsl).

- **`DockEdge`/`LensShape` type unification via `luminos-types` crate (Story 002):** E01 discovered duplicate definitions in `luminos-platform` (without serde) and `luminos-core` (with serde). Story 002 resolved this by creating a new `luminos-types` crate as the canonical source for all shared data types. `luminos-types` has zero workspace dependencies (only `serde`), breaking the circular dependency risk. Both `luminos-platform` and `luminos-core` re-export from `luminos-types`. This was a user-directed deviation from the original DESIGN.md approach (which proposed re-exports from `luminos-platform`). Types moved: `ScreenRect`, `ScreenPoint`, `DisplayInfo`, `PixelFormat`, `CaptureFrame`, `DockEdge`, `LensShape`, `OverlayMode`, `MagnificationMode`, `TrackingMode`, `ColorFilterType`, `TtsStatus`, `PresentMode`, `GpuPreference`, `InterpolationMode`. `CaptureFrame` skips `Serialize`/`Deserialize` (runtime GPU type with `Arc<[u8]>`). Original locations re-export from `luminos-types` for backward compatibility.

### Key Type Definitions

The following types are used extensively in E02. Canonical definitions live in `luminos-types`; re-exported from `luminos-platform::traits` and `luminos-core` for backward compatibility.

**From `luminos-types` (canonical source, re-exported by `luminos-platform::traits`):**
- `ScreenRect { x: i32, y: i32, width: u32, height: u32 }` -- screen-coordinate rectangle
- `ScreenPoint { x: i32, y: i32 }` -- screen-coordinate point
- `DisplayInfo { id: String, name: String, bounds: ScreenRect, scale_factor: f64, is_primary: bool }`
- `PixelFormat` enum: `Bgra8`, `Rgba8`
- `CaptureFrame { data: Arc<[u8]>, width: u32, height: u32, stride: u32, format: PixelFormat }` (no serde -- runtime GPU type)
- `OverlayMode` enum: `FullScreen`, `Lens { width, height, shape }`, `Docked { edge, size_px }`
- `DockEdge` enum: `Top`, `Bottom`, `Left`, `Right`
- `LensShape` enum: `Rectangle`, `Ellipse`

**Traits (remain in `luminos-platform::traits`, NOT moved to `luminos-types`):**
- `ScreenCapture` trait: `list_displays()`, `capture_frame()`, `subscribe_display_changes()`
- `WindowManager` trait: `create_overlay()`, `set_overlay_bounds()`, `set_overlay_mode()`, `set_always_on_top()`, `set_visible()`, `raw_window_handle()`, `raw_display_handle()`

**New types introduced in E02:**
- `XcbCapture` (Story 001) -- `ScreenCapture` impl for X11
- `X11WindowManager` (Story 002) -- `WindowManager` impl for X11
- `SourceTextureManager` (Story 003) -- GPU texture lifecycle management
- `MagnifyUniforms` (Story 004) -- shader uniform struct
- `Renderer` (Story 005) -- holds all persistent GPU resources
- `FrameTimings` (Story 005) -- frame timing ring buffer

### Integration Points

- **`luminos-platform` → `luminos-gpu` interface:** The `CaptureFrame` produced by `XcbCapture::capture_frame()` is consumed by `SourceTextureManager::upload_capture_frame()` in `luminos-gpu`. The types are defined in `luminos-platform`; `luminos-gpu` depends on `luminos-platform`.

- **`WindowManager` → wgpu surface creation:** `X11WindowManager::raw_window_handle()` and `raw_display_handle()` return handles that `luminos-gpu` uses to create a `wgpu::Surface`. The surface is configured with the sRGB format and the best available `CompositeAlphaMode` (query capabilities, fallback chain: `PreMultiplied` → `PostMultiplied` → `Opaque`).

- **`ScreenCapture` ← overlay window ID:** The `X11WindowManager` exposes the overlay window's X11 window ID. The `XcbCapture` receives this ID at construction (or via a setter) to exclude it from capture. This is the RISK-002 mitigation path.

- **Render loop → all components:** Story 005's render loop orchestrates the full pipeline: `XcbCapture` → `SourceTextureManager` → `Renderer` (shader execution) → swap chain present. This is a standalone integration point for E02; the full application event loop with `EventLoopProxy` and `ArcSwap<AppState>` integration comes in E03/E04.

- **CI → Xvfb + Mesa + picom:** Platform integration tests (`ci_platform_tests` feature) and GPU tests run under `xvfb-run` on the CI runner. Mesa lavapipe (`mesa-vulkan-drivers`) provides software Vulkan for wgpu rendering tests. Mesa llvmpipe provides GL fallback. `picom` compositor runs under Xvfb for realistic self-capture prevention testing (the unmap/remap cycle is meaningless without a compositor).

### Discovered Constraints

_Updated as stories are implemented. Research findings from Task #1 are marked with [RESEARCH]._

#### Research Findings (pre-implementation)

- **[RESEARCH] RISK-002 composite pixmap approach is INCORRECT.** The risk register (RISK-002) and doc-03 Section 7.1 recommend `xcb_composite_redirect_window` to capture from the root composite buffer, claiming it "excludes override-redirect windows." This is WRONG -- compositors composite ALL visible windows (including override-redirect) into the root pixmap. The correct self-capture prevention mechanism on X11 is an **unmap/remap cycle**: unmap the overlay window before `capture_frame()`, capture, then remap after. This introduces a sub-millisecond window where the overlay is invisible, but at 60fps this is imperceptible. The risk register should be updated to reflect this correction.

- **[RESEARCH] xcap 0.9.3 returns RGBA, not BGRA on X11.** The tech strategy (doc-03 Section 4.3) states that X11 native format is BGRA and the shader must swizzle. While X11 natively uses BGRA, xcap converts internally to RGBA before returning the `CaptureFrame`. Therefore, `CaptureFrame::format` will be `PixelFormat::Rgba8` when using xcap on X11, and the `is_bgra` shader uniform should be `0.0` (no swizzle). The swizzle logic must still be implemented for future backends (direct XShm in Phase 1 returns native BGRA, Windows DXGI returns BGRA). **Impact:** Story 001 (pixel format handling), Story 004 (shader uniform value).

- **[RESEARCH] xcap creates a new XCB connection per capture call.** xcap does NOT pool XCB connections. Each `capture_frame()` call opens a new connection to the X server and closes it after. At 60fps, this is 60 connection cycles per second -- a performance concern (connection setup involves a TCP/Unix socket handshake + authentication). **Impact:** Story 001 must benchmark this overhead. If it exceeds ~1ms, connection pooling or caching (wrapping xcap's internal XCB connection) may be needed. Alternatively, the Phase 1 XShm backend (via `x11rb`) will manage its own persistent connection, making this a Phase 0 known limitation.

- **[RESEARCH] xcap does NOT support display change events.** The `subscribe_display_changes()` trait method cannot be implemented using xcap alone. Display change detection must be implemented separately using `x11rb` with RandR extension event subscription. **Impact:** Story 001 needs `x11rb` with the `randr` feature as an additional dependency for this method.

- **[RESEARCH] `ScreenCapture` trait requires modification for self-capture prevention.** The existing `ScreenCapture` trait from E01 has no method for window exclusion. Research recommends adding `set_excluded_windows(&mut self, window_ids: &[u64])` to the trait. This is a **breaking change to the E01 trait** that requires: (1) adding the method with a default no-op implementation, (2) updating the `MockScreenCapture` to support it, (3) updating mock tests. **Impact:** Story 001 must modify `luminos-platform::traits::screen_capture` and `luminos-platform::mock::capture`.

- **[RESEARCH] wgpu `CompositeAlphaMode` support varies by driver.** `PreMultiplied` is not universally supported on X11 Vulkan drivers. The surface configuration must query `surface.get_capabilities(adapter).alpha_modes` and select the best available mode in priority order: `PreMultiplied` → `PostMultiplied` → `Opaque`. Hardcoding `PreMultiplied` (as doc-03 Section 9.2 does) will fail on some drivers. **Impact:** Story 002 (surface configuration).

- **[RESEARCH] CI requires `mesa-vulkan-drivers` for lavapipe.** Mesa llvmpipe provides GL rendering but NOT Vulkan. For wgpu Vulkan testing in CI, the `mesa-vulkan-drivers` package provides lavapipe (software Vulkan). Both packages should be installed. Additionally, `picom` compositor should run under Xvfb for realistic self-capture prevention tests (unmap/remap is meaningless without a compositor). **Impact:** Story 005 (CI setup).

#### Story 002 Implementation Findings

- **[IMPL] `luminos-types` crate created for type unification (user-directed).** Instead of the DESIGN.md approach of canonical definitions in `luminos-platform` with re-exports from `luminos-core`, a new `luminos-types` crate was created with zero workspace dependencies (only `serde`). This avoids circular dependency risk and provides a cleaner architecture. All shared data types live in `luminos-types`; both `luminos-platform` and `luminos-core` re-export them. Backward compatibility is preserved.

- **[IMPL] winit event loop: using deprecated `EventLoop::create_window()`.** On X11, the deprecated `create_window` works because the X connection is reference-counted and the window survives event loop drop. The E05 render loop will migrate to `ActiveEventLoop::create_window()` in the `Resumed` callback. This is a known deviation noted in the code.

- **[IMPL] wgpu v28 API difference: `request_adapter` returns `Result`, not `Option`.** The DESIGN.md code sample showed `.await.ok_or(RenderError::NoAdapter)?` but wgpu v28's `request_adapter` returns `Result<Adapter, RequestAdapterError>`. The implementation uses `.await.map_err(|_| RenderError::NoAdapter)?`.

- **[IMPL] `linux_x11` module made `pub` for cross-crate integration tests.** The `X11WindowManager` struct needs to be accessible from `luminos-gpu` integration tests that wire together the full window-to-GPU pipeline. The module was changed from `mod linux_x11` to `pub mod linux_x11` in `luminos-platform/src/lib.rs`.

#### Story 003 Implementation Findings

- **[IMPL] `SourceTextureManager` takes ownership of `wgpu::Device`.** The constructor signature is `new(device: wgpu::Device, initial_width: u32, initial_height: u32)`. The device is stored as a field because it is needed for texture reallocation (creating new textures when capacity is exceeded). Story 005's `Renderer` must account for this ownership pattern.

- **[IMPL] `over_allocate()` uses `f64` arithmetic, not `f32`.** The DESIGN.md showed `f32` for the over-allocation calculation, but `f32` has only 24 bits of mantissa, which would cause precision loss for dimensions > 16 million pixels. `f64` avoids this. The `#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]` annotation handles the float-to-int cast.

- **[IMPL] `CaptureFrame` imported from `luminos_types`, not `luminos_platform`.** Due to Story 002's type unification into `luminos-types`, the import path is `luminos_types::CaptureFrame`, not `luminos_platform::traits::types::CaptureFrame` as the DESIGN.md originally specified.

- **[IMPL] `capacity()` accessor added beyond DESIGN.md spec.** A public `capacity() -> (u32, u32)` method was added to `SourceTextureManager` for integration tests to verify over-allocation behavior without accessing private fields. Not in original DESIGN.md.

- **[IMPL] Test helpers use GL+Vulkan backends for CI compatibility.** `generate_test_gpu_device()` requests adapters with `Backends::GL | Backends::VULKAN` and gracefully skips tests when no adapter is available (returns `Option`). This pattern is reused by Story 004's integration tests.

#### Story 004 Implementation Findings

- **[IMPL] `MagnifyPipeline` struct introduced to bundle pipeline resources.** Not in the original DESIGN.md. Bundles `RenderPipeline`, `BindGroupLayout`, and uniform `Buffer` into a single struct returned by `create_magnify_pipeline()`. Simplifies resource management for Story 005's `Renderer`.

- **[IMPL] `MagnifyUniforms._pad` is `[f32; 3]` array, not separate fields.** The WGSL struct has `_pad: f32` and `_pad2: vec2f` for alignment, but the Rust `#[repr(C)]` struct uses `_pad: [f32; 3]` for simplicity. Both produce identical 32-byte layout.

- **[IMPL] `create_magnify_pipeline()` combines shader compilation + pipeline creation.** The DESIGN.md showed separate functions for shader module creation and pipeline creation. The implementation combines them into a single function that returns `Result<MagnifyPipeline, RenderError>`, adding `ShaderCompilation` variant to `RenderError`.

- **[IMPL] Shader files loaded via `include_str!()` at compile time.** Both `.wgsl` files are embedded in the binary via `include_str!()` in `shaders/mod.rs`. No runtime file I/O needed. The `shaders/` directory contains only the WGSL files and `mod.rs`.

- **[IMPL] Viewport module placed in `luminos-gpu`, not `luminos-core`.** The DESIGN.md mentioned `luminos-core` as an alternative location for `viewport.rs`. The implementation placed it in `luminos-gpu` alongside the shader infrastructure, keeping the viewport calculation co-located with its primary consumers.

- **[IMPL] `bytemuck` added as workspace dependency.** Version 1.x with `derive` feature, used for `MagnifyUniforms` GPU buffer serialization via `Pod` and `Zeroable` derives.

- **[IMPL] Integration tests in separate files.** `tests/pipeline_creation.rs` for pipeline compilation tests, `tests/shader_output.rs` for render-and-readback tests. Both use headless GPU rendering (no window) for CI compatibility.

#### wgpu v28 API Deviations from DESIGN.md

The E02 DESIGN.md documents were authored based on earlier wgpu versions. The following wgpu v28.0.0 API differences were discovered during Stories 002-004 and must be used by Story 005 (and all future GPU code):

1. **`Instance::new(&InstanceDescriptor)`** — takes a reference, not an owned value.
2. **`request_adapter()` returns `Result<Adapter, RequestAdapterError>`** — NOT `Option<Adapter>`. Use `.map_err()`, not `.ok_or()`.
3. **`request_device(&DeviceDescriptor)` takes only 1 arg** — NO `trace_path: Option<&Path>` second parameter. Tracing is configured via `DeviceDescriptor.trace`.
4. **`DeviceDescriptor` has 6 fields** — `label`, `required_features`, `required_limits`, `experimental_features`, `memory_hints`, `trace`. Use `..Default::default()` for trailing fields.
5. **`PipelineLayoutDescriptor` uses `immediate_size: 0`** — NOT `push_constant_ranges: &[]`.
6. **`RenderPipelineDescriptor` uses `multiview_mask: None`** — NOT `multiview: None`.
7. **`SamplerDescriptor.mipmap_filter`** is `wgpu::MipmapFilterMode::Nearest` — NOT `wgpu::FilterMode::Nearest` (separate type).
8. **`device.poll()` takes `PollType::Wait { submission_index: None, timeout: None }`** — NOT `Maintain::Wait`.
9. **`RenderPassDescriptor` requires `multiview_mask: None`** field.
10. **`RenderPassColorAttachment` requires `depth_slice: None`** field.

**Impact for Story 005:** The `Renderer` struct will create render passes, poll the device, and manage pipelines. All of the above apply. Future DESIGN.md documents should be validated against `cargo doc -p wgpu` output before approval.

#### Audit Findings (pre-implementation)

- **[AUDIT F-001] E02 uses single-buffer texture upload (sequential pipeline).** Double-buffered texture swap is deferred to Phase 1 when capture and render are pipelined across separate threads. D5 ("no visible tearing") is satisfied by the sequential upload-then-render order within a single frame: the texture is fully written before the shader reads it. There is no concurrent read/write hazard in the E02 sequential pipeline.

- **[AUDIT F-006] Bicubic interpolation moved from E06 to E02 per user decision (2026-03-28).** Doc-09 Section 4.2 explicitly excludes bicubic interpolation from E02 scope ("Excluded: Bicubic interpolation (Epic 6)"). The user confirmed both bilinear and bicubic should be implemented in E02. E06 scope should be updated to remove bicubic when that epic is decomposed.

#### E01 Carry-Forward Constraints

- **E01 carry-forward: `tokio` workspace dep has minimal "sync" feature.** E02 backends need expanded features (`rt`, `macros`) for async tests. Update workspace `tokio` dependency features or use expanded features in `[dev-dependencies]` only (as E01 Story 003 did for mock async tests).

- **E01 carry-forward: `DockEdge`/`LensShape` duplication.** RESOLVED by Story 002 via `luminos-types` crate. Canonical definitions now in `luminos-types`, re-exported by both `luminos-platform::traits` and `luminos-core::config::schema`.

- **E01 carry-forward: Tauri is optional behind feature flag.** `luminos-app` does not include Tauri setup. The E02 render loop is a standalone demo, not integrated with Tauri. The binary target for E02 integration testing may be a separate example binary or an integration test in `luminos-gpu`, not `luminos-app`.

- **E01 carry-forward: `sherpa-rs-sys` v0.6.8 panics under custom Cargo profiles.** Avoid the `dist` profile when building crates with transitive sherpa-rs dependencies. Not directly relevant to E02 (no TTS code) but important context.

- **E01 carry-forward: Virtual workspaces require explicit `resolver = "3"`.** Already set in workspace `Cargo.toml`.

### Cross-Story Dependencies

| Dependency | Source Story | Target Story | Nature |
|------------|-------------|--------------|--------|
| wgpu Device and Queue exist | 002 | 003 | Hard: texture upload requires GPU device |
| wgpu Device exists for shader compilation | 002 | 004 | Hard: render pipeline creation requires device |
| Overlay window ID available | 002 | 001 | Soft: self-capture prevention needs window ID via `set_excluded_windows()`; can be set after construction |
| `ScreenCapture` trait modification | 001 | --- | Hard (E01 breaking change): adds `set_excluded_windows()` method with default no-op; mock must be updated |
| `ScreenCapture` impl produces `CaptureFrame` | 001 | 005 | Hard: render loop needs capture source |
| `SourceTextureManager` uploads frames | 003 | 005 | Hard: render loop needs texture pipeline |
| Magnification shaders and `compute_source_region()` | 004 | 005 | Hard: render loop needs shader pipeline and viewport calc |
| `DockEdge`/`LensShape` unification | 002 | --- | Soft: unblocks clean type usage in later epics |

### Relevant Risks

The following risks from the [Risk Register](../tech-strategy/10-risk-register.md) are relevant to E02 work:

| Risk ID | Title | Score | Relevance to E02 |
|---------|-------|-------|-------------------|
| RISK-001 | Dual event loop coexistence (winit + Tauri) | 8 (Mitigate) | Not directly applicable to E02 (only winit, no Tauri). Validation deferred to E04 when Tauri is introduced. E02 establishes the winit event loop side; E04 adds the Tauri side. |
| RISK-002 | Self-capture infinite feedback loop | 9 (Mitigate) | **Critical for Story 001.** The `XcbCapture` implementation must exclude the overlay window from capture. **Research correction:** composite pixmap does NOT work (compositors include all windows). Primary mechanism: **unmap/remap cycle** around capture calls. Requires `set_excluded_windows()` trait modification (breaking E01 change). Validated in E02 integration tests with picom compositor. |
| RISK-004 | Render thread starvation under load | 6 (Monitor) | Story 005 establishes the `FrameTimings` ring buffer to detect starvation. The E02 render loop is the first measurement point. P99 < 20ms target at 2x zoom. |
| RISK-006 | Multi-display and HiDPI coordinate inconsistencies | 6 (Monitor) | Story 001 (`list_displays`) and Story 004 (`compute_source_region`) must use physical pixel coordinates consistently. Document the convention. E02 targets single-display only; multi-display is E05+. |
| RISK-007 | X11 capture bottleneck at low zoom on high-res displays | 9 (Mitigate) | Story 001 uses xcap's non-SHM path. At 1.5x zoom on 1080p, capture region is 1280x720 (3.5MB). Benchmark capture time. XShm optimization is deferred to Phase 1 (E08). Mitigation: higher zoom levels have smaller capture regions. |
| RISK-008 | CPU-to-GPU texture upload bandwidth pressure | 6 (Monitor) | Story 003 implements texture upload. Over-allocation (1.5x) reduces reallocation frequency. Double buffering prevents stalls. Benchmark upload time at various region sizes. |
| RISK-010 | Memory pressure on 4GB total RAM systems | 6 (Monitor) | Story 003's texture allocation strategy should be conscious of GPU memory. At 1080p: source texture ~12MB (1.5x over-alloc), intermediates ~16MB total. Well within 100MB GPU budget from doc-03 Section 1.3. |
| RISK-016 | wgpu backend compatibility across platforms | 4 (Monitor) | Story 002 initializes wgpu with Vulkan backend. Mesa llvmpipe is the CI fallback. Document any driver-specific issues encountered. |
| RISK-017 | Screen content leakage via logs and GPU memory | 6 (Monitor) | E01 already implements custom `Debug` for `CaptureFrame` that omits pixel data. E02 stories must maintain this discipline: never log pixel data, use metadata-only debugging. |
| RISK-030 | wgpu/winit major version upgrade cascade | 9 (Mitigate) | E02 is the first epic to exercise wgpu 28.0.0 and winit 0.30.13 in production code. Pin versions via workspace deps. Report any API issues encountered. |

---

## Blockers & Issues Log

| ID | Date | Story | Description | Resolution | Status |
|----|------|-------|-------------|------------|--------|
| B001 | --- | --- | --- | --- | --- |

## Retrospective Notes

_Filled in when the epic is DONE. What went well, what didn't, what to carry forward to future epics._
