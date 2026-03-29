# Story E03/005: End-to-End Pipeline Integration

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** 003 (Cursor-Follow Viewport Tracking), 004 (Global Keyboard Shortcuts)

---

## Problem Statement

Stories 001 through 004 deliver the individual components of the interactive magnification pipeline: X11 input monitoring produces mouse and keyboard events (001), `ArcSwap<AppState>` and `EventLoopProxy` provide the shared state infrastructure (002), the tracking engine computes smooth viewport positions (003), and the hotkey matcher translates key combinations into zoom and toggle actions (004). But these components are not yet wired together -- there is no orchestration layer that connects the input event stream to the tracking engine and hotkey handler, feeds their output into the render loop, and drives the full interactive cycle.

This story assembles the complete interactive magnification pipeline: the input monitor thread produces events, an input processing task dispatches mouse moves to the tracking engine and key events to the hotkey matcher, state changes flow through `ArcSwap<AppState>`, and the render loop reads state each frame to produce the magnified view. It upgrades the E02 static render loop into an event-driven loop using `EventLoopProxy`, adds graceful shutdown, and delivers the integration tests that verify all five E03 success criteria end-to-end on Xvfb.

After this story, a user launches Luminos on a Linux X11 desktop, moves their mouse to pan the magnified view, presses hotkeys to zoom in/out and toggle magnification -- the core interactive magnification experience.

## User Scenarios

### US-1: Interactive Render Loop

As a low-vision user, I want the magnification pipeline to run as an event-driven loop that responds to my mouse movement and keyboard shortcuts in real time so that I can control the magnified view interactively.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given the application started with all E03 components initialized (input monitor, state manager, tracking engine, hotkey matcher, renderer), when the event loop runs, then it processes `LuminosEvent::StateChanged` events by requesting a redraw, and on each frame: loads `AppState` from `ArcSwap`, runs `TrackingEngine::update()`, calls `compute_source_region()`, captures the frame, uploads, renders, and presents.
- **AC-1.2:** Given the event-driven render loop running, when no state changes occur (user is idle), then the render loop continues rendering at the target frame rate (driven by `AboutToWait` -> `request_redraw()`), maintaining the last known viewport position.
- **AC-1.3:** Given a `LuminosEvent::StateChanged` event from the input processing task, when the event loop receives it, then a redraw is requested immediately (within the current event loop iteration) rather than waiting for the next vsync tick.

### US-2: Input Event Dispatching

As the magnification system, I need an input processing task that reads events from the input monitor channel and dispatches them to the appropriate handler so that mouse moves update the viewport and key presses trigger hotkeys.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given the input processing task running and `X11InputMonitor` producing `InputEvent::MouseMoved { position }` events, when a mouse move event is received, then it calls `StateManager::update_mouse_position(position)` to write the position to `ArcSwap<AppState>`.
- **AC-2.2:** Given the input processing task running and `X11InputMonitor` producing `InputEvent::KeyEvent` events, when a key event is received, then it passes the event to `HotkeyMatcher::match_event()` and, if a match is found, calls `dispatch_hotkey()` to execute the state mutation.
- **AC-2.3:** Given the input processing task receiving `InputEvent::MouseButton` or `InputEvent::Scroll` events, when these events arrive, then they are ignored (no action taken) -- mouse button and scroll handling is not in E03 scope.

### US-3: Mouse-to-Viewport Latency

As a low-vision user, I want the magnified view to respond to my mouse movement within one frame so that the tracking feels immediate and natural.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given the full pipeline running on Xvfb, when `xdotool mousemove` moves the cursor to a new position, then the viewport source region reflects the new position within 2 frames (33ms) -- 1 frame for event propagation through the pipeline, 1 frame for rendering. *(Note: SC1 specifies < 16.67ms from mouse event to viewport update in state; the rendered frame appears on the next vsync.)*
- **AC-3.2:** Given rapid mouse movement (continuous `xdotool mousemove` calls), when frame timings are inspected, then P99 frame time remains under 20ms (E03 SC5: no dropped frames during rapid mouse movement).

### US-4: Hotkey-to-State Latency

As a low-vision user, I want zoom changes from keyboard shortcuts to take effect on the next rendered frame so that the response feels instantaneous.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given the full pipeline running on Xvfb with zoom level 2.0, when `xdotool key ctrl+alt+equal` is executed, then the `AppState.settings.magnification.zoom_level` value read by the render thread on the next frame is 3.0 (2.0 * 1.5).
- **AC-4.2:** Given the full pipeline running on Xvfb with magnification active, when `xdotool key ctrl+alt+8` is executed, then `AppState.is_active` is `false` on the next render thread read.

### US-5: Graceful Shutdown

As the application lifecycle manager, I want the interactive pipeline to shut down cleanly when requested so that resources are released and no threads are left running.

**Priority:** P1
**Acceptance Criteria:**

- **AC-5.1:** Given the event loop running with all E03 threads active, when `LuminosEvent::RequestExit` is sent via `EventLoopProxy`, then the event loop exits, the input monitor thread is stopped, and all channels are dropped.
- **AC-5.2:** Given shutdown is initiated, when the input monitor thread receives a channel close (receiver dropped), then it stops its X11 event loop and the thread terminates.

### US-6: ArcSwap State Visibility Across Threads

As the rendering pipeline, I need state changes written by the input thread to be visible to the render thread on the very next frame read so that the magnified view is always up to date.

**Priority:** P0
**Acceptance Criteria:**

- **AC-6.1:** Given a writer thread calling `state_manager.update_mouse_position()` with a known position, when the render thread calls `ArcSwap::load()` on the next frame, then the loaded `AppState.mouse_position` matches the written value.
- **AC-6.2:** Given the `ArcSwap::load()` read on the render thread, when measured, then each read completes in under 100ns (E03 D4 verification).

### US-7: CI Pipeline Updates

As the CI infrastructure, I need the CI pipeline updated to support the E03 integration tests so that the interactive pipeline is validated on every push.

**Priority:** P0
**Acceptance Criteria:**

- **AC-7.1:** Given the GitHub Actions CI workflow, when the `test-platform` job runs, then `xdotool` is available in the test environment for simulating input events.
- **AC-7.2:** Given the CI test jobs, when E03 integration tests run under `xvfb-run`, then all tests pass with the `ci_platform_tests` feature enabled.

## Functional Requirements

- **FR-1:** Upgrade the render loop to an event-driven loop: handle `LuminosEvent::StateChanged` (request redraw), `LuminosEvent::RequestExit` (exit loop), `Event::AboutToWait` (tracking update + request redraw), and `Event::RedrawRequested` (full render pipeline). *(Traced by AC-1.1, AC-1.2, AC-1.3)*
- **FR-2:** Implement the input processing task: a synchronous loop on a dedicated thread that reads from the `mpsc::Receiver<InputEvent>` channel, dispatches `MouseMoved` to `StateManager::update_mouse_position()`, dispatches `KeyEvent` to `HotkeyMatcher::match_event()` + `dispatch_hotkey()`, and ignores other event types. *(Traced by AC-2.1, AC-2.2, AC-2.3)*
- **FR-3:** Per-frame render cycle: load `ArcSwap<AppState>` -> `TrackingEngine::update()` -> `compute_source_region()` -> `ScreenCapture::capture_frame()` -> `Renderer::render_frame()` -> present. *(Traced by AC-1.1)*
- **FR-4:** Wire input monitor startup: spawn `X11InputMonitor::subscribe_input_events(32)` and pass the receiver to the input processing task. *(Traced by AC-2.1)*
- **FR-5:** Implement graceful shutdown: `LuminosEvent::RequestExit` stops the event loop, drops channels, and joins the input processing thread. *(Traced by AC-5.1, AC-5.2)*
- **FR-6:** Add `xdotool` to the CI runner's `apt-get install` step in `.github/workflows/ci.yml`. *(Traced by AC-7.1)*
- **FR-7:** Write integration tests (gated behind `ci_platform_tests`): *(Traced by AC-3.1, AC-3.2, AC-4.1, AC-4.2, AC-6.1, AC-6.2, AC-7.2)*
  - Mouse move via `xdotool mousemove` produces viewport position change within 2 frames
  - `xdotool key ctrl+alt+equal` changes zoom level
  - `xdotool key ctrl+alt+8` toggles magnification
  - Frame timing P99 < 20ms during rapid mouse movement
  - ArcSwap state convergence: write from input thread, read on next frame

## Non-Functional Requirements

- **NFR-1:** P99 frame time must remain under 20ms during rapid mouse movement (E03 SC5). Relaxed threshold for CI software rendering: P99 < 50ms.
- **NFR-2:** `ArcSwap::load()` must complete in under 100ns per read (E03 D4).
- **NFR-3:** Mouse movement must update viewport position within 1 frame (< 16.67ms) from event delivery to state write (E03 SC1).
- **NFR-4:** No `unwrap()` or `expect()` in production code paths.
- **NFR-5:** All new public items must have `///` doc-comments.
- **NFR-6:** CI pipeline additions must not increase the total CI wall time by more than 2 minutes.

## Out of Scope

- Focus tracking integration (AT-SPI2 events driving the tracking engine) -- deferred to E07.
- Tauri control panel integration (IPC commands for zoom, mode) -- deferred to E04.
- Lens mode and docked mode rendering behavior -- deferred to E05.
- Configurable keybindings -- deferred to E07.
- Performance benchmark CI stage with regression detection -- deferred to when self-hosted runners are available.
- Adaptive frame rate / dynamic frame limiter -- deferred to E05.
- Visual feedback overlays (zoom level OSD, cursor highlighting) -- deferred to E06.

## Open Questions

*None -- all integration decisions resolved in HIGH_LEVEL_PLAN.md shared context and architecture decisions sections.*
