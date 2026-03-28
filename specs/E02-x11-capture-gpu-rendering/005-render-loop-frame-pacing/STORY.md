# Story E02/005: Render Loop, Frame Pacing & CI

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** 001 (X11 screen capture), 003 (GPU texture pipeline), 004 (magnification shaders & viewport)

---

## Problem Statement

Stories 001, 002, 003, and 004 deliver the individual components of the rendering pipeline: screen capture, overlay window, GPU texture management, and magnification shaders. But these components are not yet connected -- there is no orchestration layer that drives the per-frame cycle of capture, upload, render, and present. Without this integration, no magnified image appears on screen.

This story assembles the complete rendering pipeline and drives it at 60fps. It implements the `Renderer` struct that holds all persistent GPU resources, the `FrameTimings` ring buffer for performance monitoring, and the winit event loop that executes the pipeline every frame. It also updates the CI pipeline to support headless X11 (Xvfb) and GPU testing (Mesa llvmpipe), and delivers the integration test that validates the full capture-to-present pipeline.

After this story, a user launches Luminos on a Linux X11 desktop and sees their screen magnified at a chosen zoom level, rendered smoothly at 60fps in a transparent overlay -- the core value proposition made visible for the first time.

## User Scenarios

### US-1: Complete Rendering Pipeline Execution

As a low-vision user, I want the magnification pipeline to run continuously at 60fps so that I see a smooth, real-time magnified view of my screen.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given the Renderer initialized with all GPU resources, when `render_frame()` is called with a valid `CaptureFrame`, then the pipeline executes: upload to source texture, execute magnification shader, present to swap chain -- producing a magnified frame on the overlay window.
- **AC-1.2:** Given the render loop running at `PresentMode::Fifo` (vsync), when the display runs at 60Hz, then the pipeline delivers frames at approximately 60fps (±2fps variance).
- **AC-1.3:** Given a sequence of 120 frames, when `FrameTimings` is inspected, then `p99()` returns a value under 20ms (the P99 target from doc-03 Section 8.3).

### US-2: Frame Timing Monitoring

As the rendering pipeline, I want to track frame-to-frame timing so that performance degradation can be detected and reported to the control panel.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given a `FrameTimings` ring buffer, when `record()` is called with 120 frame durations, then `p99()` returns the 99th percentile frame time from the last 120 samples.
- **AC-2.2:** Given frame timings, when `average()`, `min()`, `max()` are called, then they return correct aggregate statistics over the last 120 frames.
- **AC-2.3:** Given a `FrameTimings` instance, when `summary(target_fps)` is called, then it returns a `FrameTimingSummary` with `average_ms`, `p99_ms`, `min_ms`, `max_ms`, and `target_fps` fields.
- **AC-2.4:** Given P99 frame time exceeding 20ms for 300 consecutive measurements (5 seconds at 60fps), when the performance threshold check runs, then a `warn!` log is emitted indicating performance degradation.
- **AC-2.5:** Given P99 frame time exceeding 33ms (under 30fps) for 300 consecutive measurements, when the threshold check runs, then an `error!` log is emitted indicating critical performance degradation.

### US-3: Graceful Capture Failure Handling

As a low-vision user, I want the magnified view to persist when screen capture temporarily fails so that I do not see a blank or flickering screen.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given a capture failure during `render_frame()`, when the renderer detects the error, then it renders the previous frame from the existing source texture (stale frame) rather than presenting a blank screen.
- **AC-3.2:** Given the renderer in stale frame mode, when capture resumes successfully, then the renderer uploads the new frame and returns to normal rendering.

### US-4: Shader Variant Selection

As a rendering pipeline developer, I want the Renderer to support selecting between bilinear and bicubic shader variants so that the interpolation quality can be configured.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given `Renderer::new()` with `InterpolationMethod::Bilinear`, when `render_frame()` executes, then the bilinear shader is used.
- **AC-4.2:** Given `Renderer::new()` with `InterpolationMethod::Bicubic`, when `render_frame()` executes, then the bicubic shader is used.

### US-5: Window Resize Handling

As a user resizing the overlay window, I want the rendering pipeline to adapt to the new window dimensions so that the magnified view fills the new window size without artifacts.

**Priority:** P1
**Acceptance Criteria:**

- **AC-5.1:** Given a window resize event, when the winit event loop processes it, then the wgpu surface is reconfigured with the new dimensions.
- **AC-5.2:** Given a surface reconfiguration, when the next frame renders, then the magnified view fills the new window dimensions without distortion or black borders.

### US-6: CI Pipeline for X11 and GPU Tests

As a CI pipeline, I want to run platform-specific X11 capture tests and GPU shader tests under headless X11 (Xvfb) with software rendering (Mesa llvmpipe) so that the rendering pipeline is validated on every push.

**Priority:** P0
**Acceptance Criteria:**

- **AC-6.1:** Given the GitHub Actions CI workflow, when X11 platform tests run, then they execute under Xvfb with a 1920x1080 virtual screen.
- **AC-6.2:** Given the CI runner with Mesa llvmpipe installed, when GPU tests (`luminos-gpu` test suite) run, then shader compilation tests pass using the GL backend.
- **AC-6.3:** Given the CI workflow, when the `test-platform` job runs, then it executes `cargo nextest run -p luminos-platform --features ci_platform_tests` under `xvfb-run`.
- **AC-6.4:** Given the CI workflow, when the `test-gpu` job runs, then it executes `cargo nextest run -p luminos-gpu` under `xvfb-run` with `MESA_GL_VERSION_OVERRIDE=4.5` and `LIBGL_ALWAYS_SOFTWARE=1`.

### US-7: End-to-End Pipeline Integration Test

As a testing infrastructure, I want a single integration test that validates the complete capture-to-present pipeline so that pipeline regressions are caught automatically.

**Priority:** P0
**Acceptance Criteria:**

- **AC-7.1:** Given the integration test running on Xvfb, when the test creates an overlay, captures a frame, uploads it, renders through the magnification shader, and presents, then all steps complete without error.
- **AC-7.2:** Given the integration test, when the output frame is inspected, then it has non-zero pixel data and dimensions matching the overlay viewport.
- **AC-7.3:** Given the integration test, when `FrameTimings` is queried, then at least one frame time has been recorded.

## Functional Requirements

- **FR-1:** Implement `Renderer` struct in `crates/luminos-gpu/src/renderer.rs` holding all persistent GPU resources: device, queue, magnification pipeline(s), bind group layout, source texture manager, frame timings. *(Traced by US-1, US-4)*
- **FR-2:** Implement `Renderer::new()` constructor accepting `wgpu::Device`, `wgpu::Queue`, surface format, initial viewport dimensions, and `InterpolationMethod`. *(Traced by AC-4.1, AC-4.2)*
- **FR-3:** Implement `Renderer::render_frame()` method executing the single-pass magnification pipeline: upload `CaptureFrame` to source texture, create per-frame bind group, encode render pass, present to swap chain. *(Traced by AC-1.1)*
- **FR-4:** Implement `Renderer::handle_capture_failure()` delegating to `SourceTextureManager::record_capture_failure()` and rendering the stale frame. *(Traced by AC-3.1, AC-3.2)*
- **FR-5:** Implement `Renderer::resize()` for surface reconfiguration on window resize. *(Traced by AC-5.1, AC-5.2)*
- **FR-6:** Implement `FrameTimings` struct in `crates/luminos-gpu/src/frame_timings.rs` with a circular buffer of 120 `Duration` values. *(Traced by US-2)*
- **FR-7:** Implement `FrameTimings::record()`, `p99()`, `average()`, `min()`, `max()`, `summary()` methods per doc-03 Section 8.3. *(Traced by AC-2.1, AC-2.2, AC-2.3)*
- **FR-8:** Implement performance degradation detection: warn at P99 > 20ms for 5s, error at P99 > 33ms for 5s. *(Traced by AC-2.4, AC-2.5)*
- **FR-9:** Implement `FrameTimingSummary` struct for IPC-ready performance data. *(Traced by AC-2.3)*
- **FR-10:** Add CI jobs to `.github/workflows/ci.yml`: `test-platform` (Xvfb + `ci_platform_tests`) and `test-gpu` (Xvfb + Mesa llvmpipe). *(Traced by AC-6.1, AC-6.2, AC-6.3, AC-6.4)*
- **FR-11:** Write `render_pipeline_capture_to_present` integration test validating the full pipeline on Xvfb. *(Traced by AC-7.1, AC-7.2, AC-7.3)*

## Non-Functional Requirements

- **NFR-1:** Frame time P99 must be under 20ms at 2x zoom on the CI runner (Mesa llvmpipe) per doc-03 Section 8.3. Relaxed threshold for software rendering: P99 < 50ms.
- **NFR-2:** `FrameTimings` operations (`record`, `p99`, `average`) must complete in under 10 microseconds (pure arithmetic on a 120-element array).
- **NFR-3:** No `unwrap()` or `expect()` in production code paths.
- **NFR-4:** All public items must have `///` doc-comments.
- **NFR-5:** `cargo clippy` must pass for both `luminos-gpu` and `luminos-platform` with the project's standard clippy configuration.
- **NFR-6:** CI pipeline additions must not increase the total CI wall time by more than 5 minutes.

## Out of Scope

- Input handling and cursor tracking (E03).
- Tauri integration and `EventLoopProxy` (E03/E04).
- `ArcSwap<AppState>` integration for lock-free render thread reads (E03).
- Lens mode and docked mode rendering (E05).
- Color filter and cursor overlay shaders (E06).
- XShm capture optimization (Phase 1).
- Performance benchmark CI stage with regression detection (deferred to when self-hosted runners are available).
- Adaptive frame rate / performance mode with software frame limiter (E05 or later).
- `FrameTimingSummary` IPC exposure to control panel (E04).

## Open Questions

*None -- all design decisions resolved via doc-03 Sections 2, 8-9 and HIGH_LEVEL_PLAN.md architecture decisions.*
