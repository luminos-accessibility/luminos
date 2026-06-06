# Story E04/003: Live Full-Screen Magnification Integration

**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** APPROVED (approved-as-authoritative per E04 execution, 2026-06-05)
**Depends On:** 002 (controllable, self-capture-safe overlay), 001 (loop + `OverlayGpu` surface)

---

## Problem Statement

After stories 001-002 there is a running app with a transparent, controllable overlay that renders a clear color, plus the lock-free `AppState` and a wake mechanism. The engine modules that actually magnify — the E2 GPU `Renderer`, the X11 `ScreenCapture` backend (xcap), and the E3 `InputProcessingTask`/`TrackingEngine`/`HotkeyMatcher`/`StateManager` — exist and are unit-tested but are **not driven by any loop**.

This story connects them: each redraw, the loop captures the screen (excluding the overlay, per story 002), magnifies the region around the tracked viewport at the current zoom using `Renderer::render_frame`, and presents to the overlay surface. The `InputProcessingTask` runs so cursor movement pans the viewport and Phase-0 hotkeys change zoom/state. Frame timings are collected so story 005's `get_frame_timings` has real data. The result is a **working full-screen magnifier on Linux X11** — the core Phase 0 deliverable.

This story writes **no new magnification/capture/input logic**; it wires existing, tested modules into the `luminos-app` runtime.

## User Scenarios

> **AC count = 5.**

### US-1: Live full-screen magnification
As a low-vision user, I want my screen magnified live in the overlay, so that I can see content enlarged.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1 (capture→magnify→present):** Given the app running with the overlay and self-capture exclusion active, when the render loop ticks, then each frame calls `ScreenCapture::capture_frame`, feeds the `CaptureFrame` to `Renderer::render_frame(&surface, &frame, is_bgra)`, and presents — verified by a subprocess test rendering a known source pattern and asserting the overlay shows it magnified at the configured zoom (sampled pixel check via screenshot diff or a logged sample). *(FR-1, FR-2)*
- **AC-1.2 (zoom reflected):** Given a zoom level in `AppState.settings.magnification.zoom_level`, when it changes, then the next frame magnifies at the new level (the loop reads zoom + viewport lock-free from `ArcSwap` each frame). *(FR-3)*

### US-2: Cursor tracking
As a user, I want the magnified region to follow my cursor, so that I can navigate the screen.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1 (viewport follows cursor):** Given the `InputProcessingTask` running and feeding `MouseMoved` events, when the pointer moves (xdotool), then `TrackingEngine` updates the viewport center (dead zone + edge panning per E3) and the magnified region shifts accordingly, verified by asserting the viewport center in `AppState`/logs shifts in the pointer's direction beyond the dead zone. *(FR-4)*

### US-3: Hotkeys and frame timings
As a user, I want keyboard shortcuts to control zoom, and as a developer I want frame-timing data, so that the magnifier is controllable and observable.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1 (hotkeys drive state):** Given the Phase-0 hotkeys (Ctrl+Alt and `=`/`-`/`8`/`0`, plus numpad equivalents) wired through `HotkeyMatcher`, when each is pressed (xdotool), then zoom-in / zoom-out / toggle-magnification / zoom-reset mutate `AppState` via `StateManager` and the next frame reflects the change. *(FR-5)*
- **AC-3.2 (timings populated):** Given the render loop producing frames, when frames are recorded, then `Renderer::frame_timings()` yields a populated `FrameTimings` whose `FrameTimingSummary` reports a non-zero P99, reachable for story 005's `get_frame_timings`. *(FR-6)*

## Functional Requirements

- **FR-1:** The loop MUST call the X11 `ScreenCapture` backend each render to obtain a `CaptureFrame`, applying the story-002 self-capture exclusion. *(AC-1.1)*
- **FR-2:** The loop MUST feed the `CaptureFrame` to the existing `luminos_gpu::Renderer::render_frame(&surface, &frame, is_bgra)` against the overlay surface and present. `is_bgra` MUST be derived from `CaptureFrame.format` (`Bgra8` vs `Rgba8`). *(AC-1.1)*
- **FR-3:** The loop MUST read `zoom_level` and the viewport from `AppState` lock-free each frame; zoom changes MUST take effect on the next frame. *(AC-1.2)*
- **FR-4:** The loop MUST spawn the existing `InputProcessingTask::spawn(receiver, state_manager, hotkey_matcher, notifier)` wired to the X11 `InputMonitor` and the story-001 `AppNotifier`; cursor movement MUST update the viewport via `TrackingEngine`. *(AC-2.1)*
- **FR-5:** Phase-0 hotkeys MUST drive `StateManager` (`update_zoom_level`/`toggle_magnification`/`reset_zoom`) via `HotkeyMatcher::default()` Phase-0 bindings. *(AC-3.1)*
- **FR-6:** `FrameTimings` MUST be recorded each frame (already done inside `Renderer::render_frame`) and exposed via the renderer for later IPC. *(AC-3.2)*
- **FR-7:** On capture failure, the loop MUST call `Renderer::handle_capture_failure()` (stale-frame handling) rather than panicking. *(AC-1.1)*

## Non-Functional Requirements

- **NFR-1:** Frame time P99 < 20 ms (60fps budget; doc-06). Measured via `FrameTimingSummary`; a regression beyond 20 ms over a sustained window is a logged warning (doc-07 threshold).
- **NFR-2:** Input-to-viewport latency < 1 frame (16.67 ms) — reuses E3's `TrackingEngine` which already meets this; this story must not add blocking work on the input→state path.
- **NFR-3:** The render path MUST remain lock-free for state reads (`ArcSwap`); capture/upload MUST fit the render budget or be pipelined (RISK-004 awareness; full pipelining is out of scope, but the loop must not stall on capture).
- **NFR-4:** No `unwrap()`/`expect()` in production paths.

## Out of Scope

- Lens and Docked rendering/modes → Epic 5 (full-screen only here).
- Color filters, cursor enhancement → Epic 6.
- Focus/caret tracking modes → Epic 7 (cursor tracking only here).
- Capture pipelining/double-buffering optimization beyond "don't stall" → later perf work (RISK-004).
- IPC exposure of timings/zoom (commands) → story 005.

## Open Questions

- [x] Does the `Renderer` own the surface or take it per-call? — **Resolved:** `Renderer::new` owns `device`/`queue`; `render_frame` takes `&wgpu::Surface` per call. Story 003 refactors story-001 `OverlayGpu` to own the surface and host a `Renderer`, calling `render_frame(&self.surface, &frame, is_bgra)` each redraw. The exact device/queue/surface ownership split (and whether `wgpu::Device` can be cloned for `surface.configure`) is confirmed during implementation. *(Integration detail, not a spec blocker.)*
- [x] Where does `is_bgra` come from? — **Resolved:** from `CaptureFrame.format` (`PixelFormat::Bgra8` → `true`, `PixelFormat::Rgba8` → `false`), never assumed. **As shipped, the X11 `XcbCapture` backend yields RGBA** (`PixelFormat::Rgba8` → `is_bgra = false`); the flag is derived from `frame.format` at runtime, so the loop stays correct regardless of the backend's channel order.
- [x] How is the X11 `InputMonitor` obtained? — **Resolved:** `X11InputMonitor::new()?.subscribe_input_events(capacity)` (E3) yields the `mpsc::Receiver<InputEvent>` passed to `InputProcessingTask::spawn`.
