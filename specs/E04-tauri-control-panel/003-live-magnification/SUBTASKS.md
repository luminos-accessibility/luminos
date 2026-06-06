# Subtasks: Story E04/003 -- Live Full-Screen Magnification Integration

**Status:** DONE
**Started:** 2026-06-05
**Completed:** 2026-06-05
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 1 | 1 | 0 | 0 |
| 2. Core (renderer + capture wiring) | 4 | 4 | 0 | 0 |
| 3. Integration (input pipeline + loop) | 4 | 4 | 0 | 0 |
| 4. Polish & Acceptance | 1 | 1 | 0 | 0 |
| **Total** | **10** | **10** | **0** | **0** |

> Reuses E2 `Renderer` and E3 `StateManager`/`TrackingEngine`/`HotkeyMatcher`/`InputProcessingTask`/X11 capture+input **as-is**. New code is loop glue only. **`wgpu::Device` AND `wgpu::Queue` are both `#[derive(Clone)]` in wgpu 29.0.3 (Arc-backed)** — the one wgpu API check is RESOLVED: `OverlayGpu` keeps cloned `device`/`queue` for surface (re)config + the inactive clear path, and moves clones into `Renderer::new`. No `Renderer::reconfigure_surface` helper was needed.
>
> **Completed 2026-06-05.** Frame path as built: the marshaled ~60 Hz heartbeat (DC-9) wakes `MainEventsCleared`; `present_if_ready` loads `AppState` lock-free, branches inactive→`render_clear` / active→`CaptureDriver` (loop-owned `TrackingEngine` + `XcbCapture`, excluded once) → `OverlayGpu::render(&CaptureFrame)` (E2 `Renderer::render_frame`, `is_bgra` from `frame.format`). Input pipeline (`X11InputMonitor` → `InputProcessingTask`) spawned at `Ready` over the SAME `ArcSwap`. Frame-timing summary published to `LuminosHandle.frame_timings` each frame for story 005.

---

## Phase 1: Setup

### T001 -- Module scaffold + dependencies
**Traces to:** FR-1, FR-2
**Status:** DONE
**Files:** `crates/luminos-app/src/capture_driver.rs`, `crates/luminos-app/src/overlay_gpu.rs`, `crates/luminos-app/Cargo.toml`, `crates/luminos-gpu/src/lib.rs`, `crates/luminos-app/src/lib.rs`

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
**Status:** DONE
**Files:** `crates/luminos-app/src/overlay_gpu.rs`

**TDD Cycle:**
1. **Red:** `overlay_gpu_render_magnifies` (GPU, Mesa) -- feed a known `CaptureFrame`, render offscreen, assert source scaled by zoom; `frame_timing_summary()` non-zero after N renders.
2. **Green:** `OverlayGpu::new(window, w, h, method)` creates instance/adapter/device/queue, configures surface, moves device/queue into `Renderer::new`. Implement `render(&CaptureFrame)` → `renderer.render_frame(&surface, frame, is_bgra)`; `frame_timing_summary()`; `handle_capture_failure()`; `resize`. **Confirm `wgpu::Device` clone (or add `Renderer::reconfigure_surface`) for resize.**
3. **Refactor:** Map `RenderError` → `AppError::Gpu`.

**Completion Notes:**
> `OverlayGpu` now holds `surface, _window, device(clone), queue(clone), config, renderer, target_fps`. `new(window, w, h, method, target_fps)` builds the `Renderer` ONCE from cloned device/queue + `config.format`. `render(&CaptureFrame)` calls `Renderer::render_frame(&surface, frame, is_bgra)` and reconfigures+retries once on a recoverable surface error (`is_recoverable_surface_error` matches `RenderError::SurfaceTexture` "Lost"/"Outdated"). Added `handle_capture_failure()`, `frame_timing_summary()` (= `renderer.frame_timings().summary(target_fps)`), `viewport_size()`, `resize()` (also resizes renderer viewport). **wgpu API check RESOLVED:** `wgpu::Device` AND `wgpu::Queue` are `#[derive(Clone)]` in 29.0.3. GPU caveat (DC-10): present FAILS under headless Xvfb (`OverlayGpu::new` errors `NoAdapter`), so the live magnify path is unobservable headless; magnify SHADER correctness is covered by `luminos-gpu::shader_output` + the offscreen `overlay_gpu_renderer_summary_zeroed_before_render` unit test (real `Renderer` via `compatible_surface: None`).

---

### T003 -- `is_bgra` from `CaptureFrame.format`
**Traces to:** FR-2
**Status:** DONE
**Files:** `crates/luminos-app/src/overlay_gpu.rs`

**TDD Cycle:**
1. **Red:** `is_bgra_derived_from_format` -- `Bgra8`→true, `Rgba8`→false.
2. **Green:** Derive `is_bgra` in `render`.
3. **Refactor:** —

**Completion Notes:**
> `is_bgra_format(PixelFormat) -> bool` (pure, `matches!(format, Bgra8)`) in `overlay_gpu.rs`, called by `render`. Tests `overlay_gpu_is_bgra_{true_for_bgra8,false_for_rgba8}`. **is_bgra reality (Deviation):** shipped `XcbCapture` hardcodes `PixelFormat::Rgba8` → `is_bgra=false` for X11 today; the DESIGN/STORY "xcap yields BGRA" prose is WRONG. Plumbing retained for future Windows DXGI (`Bgra8`).

---

### T004 -- `CaptureDriver`: region from state + capture (overlay-excluded)
**Traces to:** FR-1, FR-3, FR-7, AC-1.1, AC-1.2
**Status:** DONE
**Files:** `crates/luminos-app/src/capture_driver.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `capture_driver_computes_region_from_state` -- region == `compute_source_region(center, zoom, bounds)` for sample states.
2. **Green:** `CaptureDriver::new` builds the X11 `ScreenCapture` (with overlay XID for story-002 exclusion); `capture(state)` computes region and returns a `CaptureFrame`; on `Err` returns a sentinel so the loop calls `handle_capture_failure`.
3. **Refactor:** Map `CaptureError` → `AppError`; `warn!` on transient failure.

**Completion Notes:**
> `CaptureDriver` (Linux) owns `XcbCapture` + the loop-owned `TrackingEngine` + `display_id` + `screen_bounds`. `new(overlay_xid, screen_bounds)` sets self-capture exclusion ONCE (skipped when `overlay_xid` is `None`) and resolves the display id. **API split (Deviation from DESIGN's `capture(state)`):** to advance the `TrackingEngine` exactly once per frame, the loop calls `region_for_state(mouse, viewport, zoom)` (advances tracking → region) ONCE, then `capture_region(region)` (`&self`, captures). A single combined `capture()` would have double-advanced tracking. `region_for` = `TrackingEngine::update` → `compute_source_region` (both reused as-is). Tests: `capture_driver_region_matches_compute_source_region_centered`, `_region_clamped_to_bounds_at_edge`, `_region_reflects_zoom_change_next_frame` (AC-1.2), plus X11-gated `capture_driver_capture_region_{out_of_bounds_errors,valid_succeeds}` (FR-7).

---

### T005 -- Wire capture→render into the run loop
**Traces to:** FR-1, FR-2, FR-3, FR-7, AC-1.1, AC-1.2
**Status:** DONE
**Files:** `crates/luminos-app/src/app.rs` (NOT main.rs — see Deviations)

**TDD Cycle:**
1. **Red:** `live_magnification_presents` (subprocess, Mesa) -- known pattern shown magnified; `zoom_change_reflected` -- zoom change → next-frame scale change.
2. **Green:** In `MainEventsCleared`: load state; if inactive present empty; else `frame = capture_driver.capture(&state)`; `overlay_gpu.render(&frame)` (or `handle_capture_failure` on capture sentinel).
3. **Refactor:** Keep capture off the lock path; `SurfaceError` reconfigure.

**Checkpoint:** Live full-screen magnification renders and tracks zoom from state.

**Completion Notes:**
> `run_event_loop` now takes the `Arc<ArcSwap<AppState>>` and owns `capture_driver: Option<CaptureDriver>` built at `Ready` (after `init_window_manager`, so the overlay XID is bound). `present_if_ready(gpu, capture_driver, app_state)` loads state lock-free: inactive → `present_clear` (`inactive_clear`/`present_skipped`); active → `region_for_state` (logs `magnify_region zoom/mouse/region`) → `capture_region` (logs `magnify_capture`) → `gpu.render` (`magnify_present`/`magnify_present_skipped`). On capture `Err` → `gpu.handle_capture_failure()` + `capture_failed` warn (FR-7, never panics). Capture is inline, NEVER behind a lock on the state read path (NFR-3). Self-capture decision via `LUMINOS_NO_EXCLUDE` env (default = exclude overlay XID). Subprocess tests (`tests/live_magnification.rs`): `live_magnification_capture_path_wired` (AC-1.1: `capture_driver=ready` + heartbeat advances through the active path + clean exit), `live_zoom_change_reflected_next_frame` (AC-1.2).

---

## Phase 3: Integration (input pipeline + loop)

### T006 -- Spawn `InputProcessingTask` wired to X11 input + `AppNotifier`
**Traces to:** FR-4, AC-2.1
**Status:** DONE
**Files:** `crates/luminos-app/src/app.rs` (NOT main.rs — see Deviations)

**TDD Cycle:**
1. **Red:** `cursor_moves_viewport` (subprocess, xdotool) -- pointer move shifts viewport center in state past the dead zone.
2. **Green:** `X11InputMonitor::new()?.subscribe_input_events(cap)` → `InputProcessingTask::spawn(receiver, state_manager, HotkeyMatcher::default(), app_notifier)`. (`StateManager` wraps the same `Arc<ArcSwap<AppState>>` as `LuminosHandle`.)
3. **Refactor:** Join the task on shutdown (extend story-001 graceful shutdown).

**Completion Notes:**
> `init_input_pipeline` (Linux) at `Ready`: `X11InputMonitor::new()?` → `subscribe_input_events(256)?` (spawns the XI2 thread, returns `tokio::sync::mpsc::Receiver`; `blocking_recv` needs no tokio runtime) → `InputProcessingTask::spawn(rx, StateManager::new(Arc::clone(&handle.app_state)), HotkeyMatcher::default(), notifier.clone())?`. The `StateManager` wraps the SAME `ArcSwap` as the loop (writes visible to `load()`); `notify_state_changed()` sets the same dirty flag the loop drains (DC-11 wake path). All non-fatal on failure. **Shutdown (Deviation from note #8):** the task is DROPPED (detached), NOT joined — the XI2 monitor thread owns the channel Sender and only releases it on a connection error or the next event after its Receiver closes, while the processor thread (owning the Receiver) only exits once the Sender drops: a circular ownership that makes a blocking `join()` hang at shutdown. Both daemon threads are reaped by process exit. Verified by `tests/live_magnification.rs::live_cursor_moves_viewport` (AC-2.1) and that `app_lifecycle`/`redraw_cadence` still exit cleanly.

---

### T007 -- Phase-0 hotkeys drive state
**Traces to:** FR-5, AC-3.1
**Status:** DONE
**Files:** `crates/luminos-app/src/app.rs` (NOT main.rs — see Deviations) (verification), reuses E3 `HotkeyMatcher`/`dispatch_hotkey`

**TDD Cycle:**
1. **Red:** `hotkeys_drive_state` (subprocess, xdotool) -- `ctrl+alt+equal/minus/8/0` → zoom in/out/toggle/reset reflected in state + next frame.
2. **Green:** Confirm wiring (no new logic; `HotkeyMatcher::default()` + `dispatch_hotkey` from E3); add debug state logging for assertion.
3. **Refactor:** —

**Completion Notes:**
> Verification-only (no new hotkey logic; the pipeline already routes `KeyEvent` → `HotkeyMatcher::match_event` → `dispatch_hotkey` → `StateManager`). Added a test-only `LUMINOS_LOG_STATE=1` state-observation probe (`log_state_if_enabled`, logs `state mouse=... zoom=... active=...` on change, de-duped) so subprocess tests can assert state mutations independent of the GPU path (DC-10). `tests/live_magnification.rs::live_hotkeys_drive_state` verifies all four via xdotool: `ctrl+alt+equal` (2→3), `ctrl+alt+minus` (3→2), `ctrl+alt+8` (toggle active), `ctrl+alt+0` (reset to 2, active preserved) — AC-3.1.

---

### T008 -- Toggle-off (inactive) path + capture-failure resilience
**Traces to:** FR-7, AC-1.1
**Status:** DONE
**Files:** `crates/luminos-app/src/app.rs` (NOT main.rs — see Deviations), `crates/luminos-app/src/overlay_gpu.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `inactive_presents_empty` -- `is_active=false` → overlay transparent/empty (no magnified content).
   - [ ] `capture_failure_reuses_last_frame` -- forced capture error → `handle_capture_failure` path, no panic, last frame retained.
2. **Green:** Implement inactive branch + capture-failure branch.
3. **Refactor:** —

**Checkpoint:** Cursor tracking + hotkeys + resilience all functional end-to-end.

**Completion Notes:**
> Inactive path: `present_if_ready` gates on `state.is_active`; inactive → `present_clear` (no magnified content). Capture-failure path: `capture_region` `Err` → `gpu.handle_capture_failure()` (→ `Renderer`/`SourceTextureManager::record_capture_failure`, stale-frame reuse) + `capture_failed` warn, never panics. Coverage: X11-gated `capture_driver_capture_region_out_of_bounds_errors` proves the `Err` path that triggers `handle_capture_failure`; `live_magnification_capture_path_wired` proves the loop survives the active path (heartbeat keeps advancing, clean exit). The inactive-vs-active branch is also observable via the state probe (`active='false'` vs `'true'`). NOTE: both inactive and active rendering require a live `OverlayGpu`, which fails to init headless (DC-10), so the `inactive_clear`/`magnify_present` *present* markers are not emitted under Xvfb; the branch decision itself is unit/log-verified.

---

### T009 -- Frame-timing exposure probe
**Traces to:** FR-6, AC-3.2
**Status:** DONE
**Files:** `crates/luminos-app/src/app.rs` (NOT main.rs — see Deviations)

**TDD Cycle:**
1. **Red:** `frame_timings_populated` (subprocess/probe) -- after a render window, P99 non-zero; reachable for story-005 `get_frame_timings`.
2. **Green:** Surface `overlay_gpu.frame_timing_summary()` through `LuminosHandle`/a debug path so story 005's command can read it.
3. **Refactor:** —

**Completion Notes:**
> Added `LuminosHandle.frame_timings: Arc<Mutex<FrameTimingSummary>>` (starts zeroed) with `set_frame_timings()` / `frame_timings()`. The loop publishes `gpu.frame_timing_summary()` to it after each `MainEventsCleared` present so story-005's `get_frame_timings` reads live data from the SAME Arc. Unit tests: `handle_frame_timing_slot_starts_zeroed`, `handle_frame_timing_slot_round_trips`. **P99-non-zero caveat (DC-10):** `FrameTimings::record` only runs inside a SUCCESSFUL `render_frame` present; under headless Xvfb `OverlayGpu` fails to init, so no present, so P99 stays 0 here AND on CI's software stack (which can't present a surface). The seam (slot write/read) is unit-proven; the live P99-non-zero assertion belongs to a real-GPU run / story 007 (per HLP "the end-to-end assertion lives in story 007").

---

## Phase 4: Polish & Acceptance

### T010 -- Acceptance + perf check + AC matrix
**Traces to:** All ACs, NFR-1
**Status:** DONE
**Files:** story docs

**Verification Checklist:**
- [x] AC-1.1 capture→magnify→present (subprocess `live_magnification_capture_path_wired` + offscreen unit `overlay_gpu_renderer_summary_zeroed_before_render`/`luminos-gpu::shader_output`; live *present* unobservable headless per DC-10)
- [x] AC-1.2 zoom reflected next frame (`live_zoom_change_reflected_next_frame` + `capture_driver_region_reflects_zoom_change_next_frame`)
- [x] AC-2.1 cursor tracking (`live_cursor_moves_viewport`; dead zone/edge pan reuse E3 `TrackingEngine`)
- [x] AC-3.1 Phase-0 hotkeys (`live_hotkeys_drive_state`: equal/minus/8/0)
- [~] AC-3.2 FrameTimings P99 non-zero — seam unit-proven (`handle_frame_timing_slot_round_trips`); live P99>0 requires a real present (DC-10), deferred to a real-GPU/story-007 run
- [~] NFR-1 P99 < 20 ms — UNMEASURABLE under headless software GL (no present → no recorded frame time). On llvmpipe the render is software so any measured P99 would NOT represent real-GPU performance; flagged for a real-GPU run. The capture path's known cost (per-frame x11rb connect + non-SHM `xcb_get_image`) is recorded as RISK-004 follow-up (B002).
- [x] `cargo fmt`/clippy clean; no `unwrap`/`expect` in production
- [x] No new capture/magnify/input logic introduced (reuse confirmed: `Renderer`, `compute_source_region`, `TrackingEngine`, `InputProcessingTask`, `HotkeyMatcher`, `StateManager`, `XcbCapture`, `X11InputMonitor` all driven, not modified)

**Completion Notes:**
> AC→test matrix produced in the agent report. Full suites: workspace excl app 436 pass / 3 skip; luminos-app 44 pass / 0 skip (30 lib unit + 14 subprocess); luminos-gpu `shader_output` 8/8. fmt clean; both clippy forms clean. deny/audit green (exit 0), no new external deps. The two `luminos-gpu::integration` `render_pipeline_*_shader_renders` failures are PRE-EXISTING dev-box env flakiness (`NoAdapter`: no EGL/DRI2/software-Vulkan surface adapter on this box) — proven by stashing the only luminos-gpu change (lib.rs re-exports) and re-running; they pass on CI llvmpipe.

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |
| B002 | 2026-06-05 | **RISK-004 per-frame-connect smell (DC-12).** `XcbCapture::{unmap,remap}_excluded_windows` open a FRESH `x11rb::connect(None)` per `capture_frame` when an exclusion set is active (~120 connect/disconnect/sec at 60 fps, in the frame budget). Plus non-SHM `xcb_get_image` (~8ms at 1080p). | DEFERRED per glue-only mandate (platform work, out of scope). Documented & flagged for Phase 1 perf (cache a connection in `XcbCapture` or reuse the `X11WindowManager`'s persistent `RustConnection`). Empirical escape hatch shipped: `LUMINOS_NO_EXCLUDE=1` skips exclusion (and thus the per-frame connect) when the transparent overlay does not self-capture. | OPEN (Phase 1) |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| All | DESIGN/SUBTASKS say `main.rs`; the run loop is in `app.rs` (`run`/`run_event_loop`/`present_if_ready`). | `main.rs` is a thin shim (logger + `app::run()`); IMPLEMENTATION_NOTES §D.1 correction. All `main.rs` refs above patched to `app.rs`. |
| T003 | DESIGN/STORY prose "xcap yields BGRA"; real `XcbCapture` hardcodes `PixelFormat::Rgba8` → `is_bgra=false` for X11 today. | `is_bgra` is DERIVED from `frame.format`, never assumed. Plumbing kept for future Windows DXGI (`Bgra8`). IMPLEMENTATION_NOTES §D.3. |
| T002 | `OverlayGpu::new` takes `(window, w, h, method, target_fps)` (DESIGN showed `(window, w, h, method)`); struct keeps cloned `device`+`queue` (DESIGN showed a single `renderer` owning both). | `target_fps` is needed for `frame_timing_summary()`; `wgpu::Device`/`Queue` are both `Clone`, so `OverlayGpu` keeps clones for surface (re)config + the inactive clear path while `Renderer` owns its own (IMPLEMENTATION_NOTES §A). |
| T004/T005 | DESIGN's `CaptureDriver::capture(&state)` split into `region_for_state()` (advances `TrackingEngine`) + `capture_region(region)` (`&self`). | A single combined call would advance the stateful `TrackingEngine` twice per frame (once for the logged region, once for capture). Splitting advances it exactly once (story-003 §1). |
| T006/T009 | `InputProcessingTask` is DROPPED (detached) on shutdown, not joined (IMPLEMENTATION_NOTES #8 said join). `frame_timing_summary` is surfaced via a new `LuminosHandle.frame_timings: Arc<Mutex<FrameTimingSummary>>` slot. | The X11 XI2 monitor thread and the processor thread have CIRCULAR channel-endpoint ownership (Sender held by the XI2 thread, Receiver by the processor), so a blocking `join()` can hang shutdown; detaching + process-exit reap is correct. The handle slot is the cleanest cross-thread seam to story-005's `get_frame_timings`. |
| Test scaffolding | Added env-gated test hooks: `LUMINOS_FORCE_ACTIVE` (seed `is_active=true`), `LUMINOS_LOG_STATE` (log AppState on change), `LUMINOS_NO_EXCLUDE` (skip self-capture exclusion). | Headless Xvfb has no surface-compatible GPU adapter (DC-10) so the magnify *present* is unobservable; these hooks let subprocess tests assert the capture/input/state wiring. All gated like the existing `LUMINOS_SELF_CAPTURE_PROBE`/`LUMINOS_DEBUG_NOTIFY`, never affecting production. |
