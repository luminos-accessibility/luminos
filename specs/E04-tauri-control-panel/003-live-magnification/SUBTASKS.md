# Subtasks: Story E04/003 -- Live Full-Screen Magnification Integration

**Status:** NOT STARTED
**Started:** ---
**Completed:** ---
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 1 | 0 | 0 | 1 |
| 2. Core (renderer + capture wiring) | 4 | 0 | 0 | 4 |
| 3. Integration (input pipeline + loop) | 4 | 0 | 0 | 4 |
| 4. Polish & Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **10** | **0** | **0** | **10** |

> Reuses E2 `Renderer` and E3 `StateManager`/`TrackingEngine`/`HotkeyMatcher`/`InputProcessingTask`/X11 capture+input **as-is**. New code is loop glue only. Confirm `wgpu::Device` clone-for-reconfigure semantics during T002 (the one wgpu API check).

---

## Phase 1: Setup

### T001 -- Module scaffold + dependencies
**Traces to:** FR-1, FR-2
**Status:** TODO
**Files:** `crates/luminos-app/src/capture_driver.rs`, `crates/luminos-app/src/overlay_gpu.rs`, `crates/luminos-app/Cargo.toml`

**TDD Cycle:** (setup)
1. **Green:**
   - [ ] Add `luminos-gpu`/`luminos-platform` to `luminos-app`'s `tauri`-gated deps (workspace pins).
   - [ ] **Add `pub use` re-exports** (no crate-root re-exports exist today): `luminos-gpu` → `Renderer`, `FrameTimingSummary`, `InterpolationMethod`; `luminos-platform` → `ScreenCapture`, `InputMonitor`. (So `luminos_gpu::Renderer` etc. resolve; otherwise use full module paths `luminos_gpu::renderer::Renderer`, `luminos_gpu::shaders::InterpolationMethod`, `luminos_gpu::frame_timings::FrameTimingSummary`, `luminos_platform::traits::ScreenCapture`.)
   - [ ] Scaffold `CaptureDriver` + extended `OverlayGpu` signatures.
2. **Refactor:** —

> **Two real-code couplings to honor (auditor P-001/P-003):**
> - `FrameTimings` ≠ `FrameTimingSummary`: the loop must call `renderer.frame_timings().summary(target_fps)` to produce the `FrameTimingSummary` written into the handle's shared slot (for story-005 `get_frame_timings`).
> - `settings.magnification.interpolation` is a `luminos_types::InterpolationMode`; `Renderer::new` takes `luminos_gpu::InterpolationMethod`. Map between them. `Renderer` bakes the method at construction (no `set_method`), so for Phase 0 interpolation is **fixed at startup** — state this; runtime interpolation switching is later.

**Completion Notes:**
>

---

## Phase 2: Core (renderer + capture wiring)

### T002 -- `OverlayGpu` hosts `Renderer`; replace clear with `render(&CaptureFrame)`
**Traces to:** FR-2, FR-6, AC-1.1, AC-3.2
**Status:** TODO
**Files:** `crates/luminos-app/src/overlay_gpu.rs`

**TDD Cycle:**
1. **Red:** `overlay_gpu_render_magnifies` (GPU, Mesa) -- feed a known `CaptureFrame`, render offscreen, assert source scaled by zoom; `frame_timing_summary()` non-zero after N renders.
2. **Green:** `OverlayGpu::new(window, w, h, method)` creates instance/adapter/device/queue, configures surface, moves device/queue into `Renderer::new`. Implement `render(&CaptureFrame)` → `renderer.render_frame(&surface, frame, is_bgra)`; `frame_timing_summary()`; `handle_capture_failure()`; `resize`. **Confirm `wgpu::Device` clone (or add `Renderer::reconfigure_surface`) for resize.**
3. **Refactor:** Map `RenderError` → `AppError::Gpu`.

**Completion Notes:**
>

---

### T003 -- `is_bgra` from `CaptureFrame.format`
**Traces to:** FR-2
**Status:** TODO
**Files:** `crates/luminos-app/src/overlay_gpu.rs`

**TDD Cycle:**
1. **Red:** `is_bgra_derived_from_format` -- `Bgra8`→true, `Rgba8`→false.
2. **Green:** Derive `is_bgra` in `render`.
3. **Refactor:** —

**Completion Notes:**
>

---

### T004 -- `CaptureDriver`: region from state + capture (overlay-excluded)
**Traces to:** FR-1, FR-3, FR-7, AC-1.1, AC-1.2
**Status:** TODO
**Files:** `crates/luminos-app/src/capture_driver.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `capture_driver_computes_region_from_state` -- region == `compute_source_region(center, zoom, bounds)` for sample states.
2. **Green:** `CaptureDriver::new` builds the X11 `ScreenCapture` (with overlay XID for story-002 exclusion); `capture(state)` computes region and returns a `CaptureFrame`; on `Err` returns a sentinel so the loop calls `handle_capture_failure`.
3. **Refactor:** Map `CaptureError` → `AppError`; `warn!` on transient failure.

**Completion Notes:**
>

---

### T005 -- Wire capture→render into the run loop
**Traces to:** FR-1, FR-2, FR-3, FR-7, AC-1.1, AC-1.2
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:** `live_magnification_presents` (subprocess, Mesa) -- known pattern shown magnified; `zoom_change_reflected` -- zoom change → next-frame scale change.
2. **Green:** In `MainEventsCleared`: load state; if inactive present empty; else `frame = capture_driver.capture(&state)`; `overlay_gpu.render(&frame)` (or `handle_capture_failure` on capture sentinel).
3. **Refactor:** Keep capture off the lock path; `SurfaceError` reconfigure.

**Checkpoint:** Live full-screen magnification renders and tracks zoom from state.

**Completion Notes:**
>

---

## Phase 3: Integration (input pipeline + loop)

### T006 -- Spawn `InputProcessingTask` wired to X11 input + `AppNotifier`
**Traces to:** FR-4, AC-2.1
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:** `cursor_moves_viewport` (subprocess, xdotool) -- pointer move shifts viewport center in state past the dead zone.
2. **Green:** `X11InputMonitor::new()?.subscribe_input_events(cap)` → `InputProcessingTask::spawn(receiver, state_manager, HotkeyMatcher::default(), app_notifier)`. (`StateManager` wraps the same `Arc<ArcSwap<AppState>>` as `LuminosHandle`.)
3. **Refactor:** Join the task on shutdown (extend story-001 graceful shutdown).

**Completion Notes:**
>

---

### T007 -- Phase-0 hotkeys drive state
**Traces to:** FR-5, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs` (verification), reuses E3 `HotkeyMatcher`/`dispatch_hotkey`

**TDD Cycle:**
1. **Red:** `hotkeys_drive_state` (subprocess, xdotool) -- `ctrl+alt+equal/minus/8/0` → zoom in/out/toggle/reset reflected in state + next frame.
2. **Green:** Confirm wiring (no new logic; `HotkeyMatcher::default()` + `dispatch_hotkey` from E3); add debug state logging for assertion.
3. **Refactor:** —

**Completion Notes:**
>

---

### T008 -- Toggle-off (inactive) path + capture-failure resilience
**Traces to:** FR-7, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs`, `crates/luminos-app/src/overlay_gpu.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `inactive_presents_empty` -- `is_active=false` → overlay transparent/empty (no magnified content).
   - [ ] `capture_failure_reuses_last_frame` -- forced capture error → `handle_capture_failure` path, no panic, last frame retained.
2. **Green:** Implement inactive branch + capture-failure branch.
3. **Refactor:** —

**Checkpoint:** Cursor tracking + hotkeys + resilience all functional end-to-end.

**Completion Notes:**
>

---

### T009 -- Frame-timing exposure probe
**Traces to:** FR-6, AC-3.2
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:** `frame_timings_populated` (subprocess/probe) -- after a render window, P99 non-zero; reachable for story-005 `get_frame_timings`.
2. **Green:** Surface `overlay_gpu.frame_timing_summary()` through `LuminosHandle`/a debug path so story 005's command can read it.
3. **Refactor:** —

**Completion Notes:**
>

---

## Phase 4: Polish & Acceptance

### T010 -- Acceptance + perf check + AC matrix
**Traces to:** All ACs, NFR-1
**Status:** TODO
**Files:** story docs

**Verification Checklist:**
- [ ] AC-1.1 capture→magnify→present (subprocess + offscreen unit)
- [ ] AC-1.2 zoom reflected next frame
- [ ] AC-2.1 cursor tracking (dead zone/edge pan)
- [ ] AC-3.1 Phase-0 hotkeys
- [ ] AC-3.2 FrameTimings P99 non-zero
- [ ] NFR-1 P99 < 20 ms over a sustained window (Mesa llvmpipe note: software renderer may exceed; record real-GPU expectation + flag llvmpipe variance)
- [ ] `cargo fmt`/clippy clean; no `unwrap`/`expect`
- [ ] No new capture/magnify/input logic introduced (reuse confirmed)

**Completion Notes:**
>

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| --- | --- | --- |
