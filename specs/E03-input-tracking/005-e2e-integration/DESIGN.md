# Design: Story E03/005 -- End-to-End Pipeline Integration

**Story:** [STORY.md](STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** spec-writer-2
**Risk Refs:** RISK-001 (dual event loop coexistence -- `EventLoopProxy` validated as bridge pattern in E02, extended here), RISK-004 (render thread starvation -- per-frame tracking + state load must stay within budget)

---

## Overview

Wire all E03 components into the complete interactive magnification pipeline. This story transforms the E02 static render loop into an event-driven loop that responds to mouse movement and keyboard shortcuts in real time. The key integration points are:

1. **Input monitor thread startup** -- spawn `X11InputMonitor::subscribe_input_events(32)` and pass the receiver to a dedicated input processing task.
2. **Input processing task** -- a synchronous loop on a dedicated thread that reads `InputEvent` values from the channel, dispatches `MouseMoved` events to `StateManager::update_mouse_position()`, and dispatches `KeyEvent` events to `HotkeyMatcher::match_event()` + `dispatch_hotkey()`.
3. **Event-driven render loop** -- `EventLoopProxy<LuminosEvent>` wakes the winit event loop on state changes. Per frame: load `ArcSwap<AppState>` -> `TrackingEngine::update()` -> `compute_source_region()` -> capture -> upload -> render -> present.
4. **Graceful shutdown** -- `LuminosEvent::RequestExit` stops all threads and exits the event loop.

The integration tests use `xdotool` on Xvfb to simulate mouse movement and keyboard shortcuts, verifying all five E03 success criteria end-to-end.

## Architecture

### Component Diagram

```
                    +---------------------+
                    | X11InputMonitor     |
                    | (Story 001)         |
                    | subscribe_input_    |
                    | events(32)          |
                    +----------+----------+
                               |
                               v  mpsc::Receiver<InputEvent>
                    +---------------------+
                    | Input Processing    |
                    | Task (new, this     |
                    | story)              |
                    |                     |
                    | MouseMoved:         |
                    |   state_manager     |
                    |   .update_mouse_pos |
                    |                     |
                    | KeyEvent:           |
                    |   hotkey_matcher    |
                    |   .match_event()    |
                    |   dispatch_hotkey() |
                    +---+--------+--------+
                        |        |
    EventLoopProxy      |        | StateManager
    .send_event()       |        | .update_*()
         |              |        |
         v              v        v
  +------+----------------------------------+
  | winit Event Loop (main thread)          |
  |                                         |
  | Event::UserEvent(StateChanged):         |
  |   window.request_redraw()               |
  |                                         |
  | Event::AboutToWait:                     |
  |   window.request_redraw()               |
  |                                         |
  | Event::WindowEvent(RedrawRequested):    |
  |   1. state = state_manager.load()       |
  |   2. center = tracking_engine.update()  |
  |   3. region = compute_source_region()   |
  |   4. frame = screen_capture.capture()   |
  |   5. renderer.render_frame()            |
  |   6. frame_timings.record()             |
  |                                         |
  | Event::UserEvent(RequestExit):          |
  |   exit event loop                       |
  +--+-----------------------------------+--+
     |                                   |
     v                                   v
  [Overlay Window]                    [Cleanup]
```

### File Structure

```
This story modifies or creates files across multiple crates:

luminos-core/src/
  pipeline.rs               [New]      InputProcessingTask, PipelineConfig
  lib.rs                    [Modified] Add `pub mod pipeline;`

luminos-gpu/src/
  (no new files -- Renderer, viewport.rs used as-is)

.github/workflows/
  ci.yml                    [Modified] Add xdotool to apt-get install in test-platform

tests/ or crates/luminos-core/tests/
  e03_integration.rs        [New]      E2E integration tests (ci_platform_tests)
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `luminos-core::pipeline` | New | `InputProcessingTask` struct, `PipelineConfig` |
| `luminos-core::lib.rs` | Modified | Add `pub mod pipeline;` |
| `.github/workflows/ci.yml` | Modified | Add `xdotool` to test-platform apt-get |
| `luminos-gpu::viewport` | Unchanged | `compute_source_region()` called per-frame |
| `luminos-gpu::renderer` | Unchanged | `Renderer::render_frame()` called per-frame |
| `luminos-core::state_manager` | Unchanged | `StateManager` used by input processing task |
| `luminos-core::tracking` | Unchanged | `TrackingEngine` called per-frame |
| `luminos-core::hotkeys` | Unchanged | `HotkeyMatcher`, `dispatch_hotkey()` used by input processing task |
| `luminos-core::event` | Unchanged | `LuminosEvent` used for EventLoopProxy |
| `luminos-platform::linux_x11::input` | Unchanged | `X11InputMonitor` spawned at startup |

### Data Flow

1. **Application startup:**
   a. Create `Arc<ArcSwap<AppState>>` with `AppState::default()`.
   b. Create `StateManager::new(arc_swap.clone())`.
   c. Create `TrackingEngine::new(TrackingConfig::default())`.
   d. Create `HotkeyMatcher::default()`.
   e. Create `X11InputMonitor::new()?`.
   f. Call `input_monitor.subscribe_input_events(32)` to get the event receiver.
   g. Create `EventLoop::<LuminosEvent>::with_user_event()` and get `EventLoopProxy`.
   h. Create the overlay window and initialize `Renderer`.
   i. Spawn the input processing thread with the event receiver, state_manager clone, hotkey_matcher, and event_loop_proxy clone.
   j. Run the winit event loop.

2. **Input processing thread (synchronous loop):**
   ```
   loop {
       match receiver.blocking_recv() {
           Some(InputEvent::MouseMoved { position }) => {
               state_manager.update_mouse_position(position);
               let _ = event_loop_proxy.send_event(LuminosEvent::StateChanged);
           }
           Some(InputEvent::KeyEvent { .. } = event) => {
               if let Some(action) = hotkey_matcher.match_event(&event) {
                   dispatch_hotkey(action, &state_manager);
                   let _ = event_loop_proxy.send_event(LuminosEvent::StateChanged);
               }
           }
           Some(_) => {} // Ignore mouse button, scroll for now
           None => break, // Channel closed, exit thread
       }
   }
   ```

3. **Per-frame render cycle (in winit event loop):**
   ```
   Event::WindowEvent { event: RedrawRequested, .. } => {
       let state = state_manager.load();
       if !state.is_active {
           return; // Magnification disabled, skip rendering
       }

       let mouse_pos = state.mouse_position;
       let zoom = state.settings.magnification.zoom_level;
       let viewport_size = (overlay_width, overlay_height);
       let screen_bounds = active_display_bounds;

       let center = tracking_engine.update(mouse_pos, viewport_size, screen_bounds, zoom);
       let source_region = compute_source_region(center, zoom, viewport_size, screen_bounds);

       match screen_capture.capture_frame(&display_info, Some(source_region)) {
           Ok(frame) => { renderer.render_frame(&surface, &frame, is_bgra)?; }
           Err(_) => { renderer.handle_capture_failure(); }
       }
   }
   ```

4. **Graceful shutdown:**
   - `LuminosEvent::RequestExit` received -> set `ControlFlow::Exit`.
   - Drop the `mpsc::Sender` (implicitly when input monitor is dropped) -> input processing thread's `blocking_recv()` returns `None` -> thread exits.
   - Join the input processing thread handle.

## API Design

### InputProcessingTask

```rust
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc;
use luminos_platform::traits::input_monitor::InputEvent;
use crate::event::LuminosEvent;
use crate::hotkeys::{dispatch_hotkey, HotkeyMatcher};
use crate::state_manager::StateManager;

/// Manages the input processing thread that dispatches input events
/// to the state manager and hotkey handler.
pub struct InputProcessingTask {
    /// Handle to the input processing thread.
    thread_handle: Option<JoinHandle<()>>,
}

impl InputProcessingTask {
    /// Spawns the input processing thread.
    ///
    /// The thread reads `InputEvent` values from the receiver, dispatches
    /// mouse moves to `StateManager::update_mouse_position()`, and
    /// dispatches key events to `HotkeyMatcher::match_event()` +
    /// `dispatch_hotkey()`. After each state mutation, sends
    /// `LuminosEvent::StateChanged` via `EventLoopProxy`.
    ///
    /// The thread exits when the receiver channel is closed (sender dropped).
    pub fn spawn(
        receiver: mpsc::Receiver<InputEvent>,
        state_manager: StateManager,
        hotkey_matcher: HotkeyMatcher,
        event_loop_proxy: winit::event_loop::EventLoopProxy<LuminosEvent>,
    ) -> Self {
        let handle = thread::Builder::new()
            .name("luminos-input-processor".to_string())
            .spawn(move || {
                Self::run(receiver, state_manager, hotkey_matcher, event_loop_proxy);
            })
            .expect("failed to spawn input processing thread");

        Self {
            thread_handle: Some(handle),
        }
    }

    /// Waits for the input processing thread to finish.
    ///
    /// Call this during shutdown after the input monitor channel is closed.
    pub fn join(mut self) {
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    fn run(
        mut receiver: mpsc::Receiver<InputEvent>,
        state_manager: StateManager,
        hotkey_matcher: HotkeyMatcher,
        event_loop_proxy: winit::event_loop::EventLoopProxy<LuminosEvent>,
    ) {
        loop {
            match receiver.blocking_recv() {
                Some(event) => {
                    Self::dispatch_event(
                        &event,
                        &state_manager,
                        &hotkey_matcher,
                        &event_loop_proxy,
                    );
                }
                None => {
                    log::info!("Input event channel closed, stopping input processor");
                    break;
                }
            }
        }
    }

    fn dispatch_event(
        event: &InputEvent,
        state_manager: &StateManager,
        hotkey_matcher: &HotkeyMatcher,
        event_loop_proxy: &winit::event_loop::EventLoopProxy<LuminosEvent>,
    ) {
        match event {
            InputEvent::MouseMoved { position } => {
                state_manager.update_mouse_position(*position);
                let _ = event_loop_proxy.send_event(LuminosEvent::StateChanged);
            }
            InputEvent::KeyEvent { .. } => {
                if let Some(action) = hotkey_matcher.match_event(event) {
                    dispatch_hotkey(action, state_manager);
                    let _ = event_loop_proxy.send_event(LuminosEvent::StateChanged);
                }
            }
            InputEvent::MouseButton { .. } | InputEvent::Scroll { .. } => {
                // Ignored in E03. Future epics may use these.
            }
        }
    }
}
```

### CI Workflow Changes

```yaml
# In .github/workflows/ci.yml, test-platform job, apt-get install step:
# Add xdotool to the package list:
sudo apt-get update && sudo apt-get install -y
  xvfb
  picom
  mesa-utils
  mesa-vulkan-drivers
  libegl-dev
  libgl1-mesa-dri
  libgbm-dev
  libpipewire-0.3-dev
  libasound2-dev
  libx11-dev
  libxi-dev
  libxtst-dev
  xdotool          # <-- Added for E03 integration tests
```

## Error Handling

This story introduces minimal new error handling:

- **`InputProcessingTask::spawn()`** uses `thread::Builder::new().spawn()` which returns `io::Result<JoinHandle>`. The `.expect()` is acceptable here because failure to spawn a thread is an unrecoverable condition during application startup (not in the per-frame hot path). In production, this could be converted to a `LuminosError::Internal` with `?` propagation if the startup function returns `Result`.

- **`event_loop_proxy.send_event()`** returns `Err` if the event loop has been dropped. The caller uses `let _ = ...` to ignore this error because: (1) if the event loop is dropped, the application is shutting down; (2) logging here would spam during shutdown.

- **Capture failures** during the render cycle are handled by the existing `Renderer::handle_capture_failure()` (from E02) which renders the stale frame.

- **Input monitor failures** (`InputError`) during `X11InputMonitor::new()` and `subscribe_input_events()` are propagated via `?` to the application startup function, which logs the error and may disable input monitoring (degrade to static magnification).

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | `X11InputMonitor` + `xdotool` integration tests | Primary target. E03 scope is X11-only. |
| Linux Wayland | Deferred to E08 | Different input monitor backend, same pipeline architecture. |
| macOS | Deferred to future epic | CGEvent-based input monitor, same pipeline architecture. |
| OpenBSD | Deferred to future epic | X11 backend (same as Linux X11). |
| Windows | Deferred to future epic | Raw Input/low-level hooks-based input monitor. |

The integration pipeline (`InputProcessingTask`, render loop structure, `StateManager` + `TrackingEngine` + `HotkeyMatcher`) is platform-independent. Only the `InputMonitor` implementation is platform-specific (Story 001 for X11). When future platform backends are added, they produce the same `InputEvent` types on the same channel -- the pipeline code is unchanged.

## Testing Strategy

### Unit Tests

- **InputProcessingTask dispatch_event:** Test the static dispatch method directly:
  - `MouseMoved` event -> `StateManager.update_mouse_position()` called (verify state changed).
  - `KeyEvent` matching a hotkey -> `dispatch_hotkey()` called (verify state changed).
  - `KeyEvent` not matching -> no state change.
  - `MouseButton` event -> no state change.
  - `Scroll` event -> no state change.

### Integration Tests (gated behind `ci_platform_tests`)

These tests run under Xvfb in the CI `test-platform` job. They require `xdotool` for input simulation.

- **Mouse move updates viewport:** Start input monitor, move mouse with `xdotool mousemove 500 300`, wait up to 100ms, read `AppState.mouse_position`, verify it reflects the move.
- **Hotkey changes zoom:** Start input monitor, send `xdotool key ctrl+alt+equal`, wait up to 100ms, read `AppState.settings.magnification.zoom_level`, verify it increased.
- **Hotkey toggles magnification:** Start input monitor, send `xdotool key ctrl+alt+8`, verify `AppState.is_active` flipped.
- **ArcSwap cross-thread visibility:** Write from input processing thread, read from test thread on next load(), verify convergence.
- **ArcSwap load latency benchmark:** Measure 1M `load()` calls, verify average < 100ns.
- **Frame timing under rapid movement:** Simulate rapid mouse movement (10 xdotool calls in quick succession), verify no state update takes more than 16ms.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Unit + Integration | Verify per-frame cycle: load state, tracking update, compute region, render. Integration test starts pipeline, moves mouse, verifies viewport changes. |
| AC-1.2 | Integration | Let pipeline idle for 1 second, verify frames continue rendering (no crash, frame count > 0) |
| AC-1.3 | Unit | Send `LuminosEvent::StateChanged` via proxy, verify event loop receives `UserEvent` and can request redraw |
| AC-2.1 | Integration | `xdotool mousemove 500 300` -> verify `AppState.mouse_position` updated |
| AC-2.2 | Integration | `xdotool key ctrl+alt+equal` -> verify zoom level changed |
| AC-2.3 | Unit | Send `MouseButton` event through dispatch, verify no state change |
| AC-3.1 | Integration | `xdotool mousemove` -> verify viewport changes within 2 frames (33ms) |
| AC-3.2 | Integration | Rapid mouse movement -> verify P99 frame time < 20ms (relaxed to < 50ms on llvmpipe) |
| AC-4.1 | Integration | Start at zoom 2.0, `xdotool key ctrl+alt+equal`, verify zoom is 3.0 |
| AC-4.2 | Integration | `xdotool key ctrl+alt+8`, verify `is_active` toggled |
| AC-5.1 | Integration | Send `RequestExit`, verify event loop exits and threads terminate |
| AC-5.2 | Unit | Drop receiver, verify input processing thread exits (join returns) |
| AC-6.1 | Integration | Writer thread updates position, reader thread verifies on next load() |
| AC-6.2 | Integration (benchmark) | 1M load() calls, average < 100ns |
| AC-7.1 | CI | `xdotool` present in test-platform runner (verified by integration tests running) |
| AC-7.2 | CI | All ci_platform_tests pass under xvfb-run |

## Performance Targets

| Metric | Target | Source |
|--------|--------|--------|
| Mouse event to state write | < 16.67ms (1 frame) | SC1, NFR-3 |
| Per-frame tracking + state load | < 0.1ms combined | TrackingEngine < 0.01ms + ArcSwap load < 100ns |
| P99 frame time during rapid mouse movement | < 20ms (< 50ms on llvmpipe) | SC5, NFR-1 |
| ArcSwap load latency | < 100ns average | SC4, NFR-2 |
| Input processing thread CPU (idle) | < 0.1% | Blocking recv, no polling |

## Security Considerations

- **xdotool in CI only:** `xdotool` is added to the CI test environment only, not to the application's runtime dependencies. It is used exclusively for integration test input simulation.
- **Input event processing:** The input processing task runs in userspace, receives events from the X11 input monitor (which operates within the X11 security model), and dispatches them to application-internal state changes. No privilege escalation, no network access, no persistent storage of input data.

## Alternatives Considered

### Alternative 1: Async input processing with tokio runtime (rejected)

The input processing task could use an async `tokio::spawn()` with `receiver.recv().await` instead of a synchronous `std::thread` with `blocking_recv()`. Rejected because: (1) the input processing task has no async I/O -- it reads from a channel and calls synchronous `StateManager` methods; (2) introducing a tokio runtime on the render path risks interference with the winit event loop (RISK-001); (3) `blocking_recv()` on a dedicated thread is simpler, more predictable, and easier to debug; (4) doc-01 Section 6.3 specifies the input monitor thread as "Normal" priority with "Yes" for "Can Block?".

### Alternative 2: Process input events in the winit event loop (rejected)

Instead of a separate thread, the winit event loop could poll the `mpsc::Receiver` in `Event::AboutToWait` using `try_recv()`. Rejected because: (1) `try_recv()` is a polling pattern that wastes CPU when idle; (2) the winit event loop would need to drain all pending events each frame, adding latency variance; (3) the separate thread model matches the architecture in doc-01 Section 6.2 ("Input Monitor Thread") and ensures the input processing cannot stall the render loop.

### Alternative 3: Separate render thread (deferred)

Doc-01 Section 6.2 shows a separate "Render Thread" driven by `RedrawRequested`. The current E02 implementation renders on the main thread inside the winit event loop (which is the typical pattern for single-window winit applications). A separate render thread could improve responsiveness by decoupling rendering from event processing. Deferred because: (1) single-thread rendering is simpler and sufficient for 60fps; (2) the E02 Renderer struct is not `Send` (contains wgpu resources that may not be thread-safe on all backends); (3) separation can be added in a future optimization epic if profiling reveals the need.
