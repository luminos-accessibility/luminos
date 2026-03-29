# Subtasks: Story E03/005 -- End-to-End Pipeline Integration

**Status:** NOT STARTED
**Started:** ---
**Completed:** ---
**Story:** [STORY.md](STORY.md)
**Design:** [DESIGN.md](DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 2 | 0 | 0 | 2 |
| 2. Core Implementation | 4 | 0 | 0 | 4 |
| 3. Integration | 4 | 0 | 0 | 4 |
| 4. Polish & Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **11** | **0** | **0** | **11** |

---

## Phase 1: Setup

### T001 -- Create pipeline module scaffolding and add xdotool to CI
**Traces to:** FR-6, AC-7.1
**Status:** TODO
**Files:** `crates/luminos-core/src/pipeline.rs`, `crates/luminos-core/src/lib.rs`, `.github/workflows/ci.yml`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `pipeline_module_exists` -- Verify `use luminos_core::pipeline::InputProcessingTask;` compiles (compilation test)
2. **Green** -- Implement minimum code to pass:
   - [ ] Create `crates/luminos-core/src/pipeline.rs` with module doc-comment and an empty `InputProcessingTask` struct placeholder
   - [ ] Add `pub mod pipeline;` to `crates/luminos-core/src/lib.rs`
   - [ ] Add re-export: `pub use pipeline::InputProcessingTask;`
   - [ ] In `.github/workflows/ci.yml`, add `xdotool` to the `apt-get install` list in the `test-platform` job's "Install system dependencies" step
   - [ ] Verify `cargo check -p luminos-core` passes
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Verify no regressions in existing `luminos-core` tests

**Completion Notes:**
>

---

### T002 -- Add winit and tokio dependencies to luminos-core Cargo.toml
**Traces to:** FR-1, FR-2
**Status:** TODO
**Files:** `crates/luminos-core/Cargo.toml`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `pipeline_imports_compile` -- Verify that `use winit::event_loop::EventLoopProxy;` and `use tokio::sync::mpsc;` compile within `pipeline.rs`
2. **Green** -- Implement minimum code to pass:
   - [ ] Add `winit` dependency to `crates/luminos-core/Cargo.toml` (from workspace)
   - [ ] Add `tokio` dependency to `crates/luminos-core/Cargo.toml` with `sync` feature (from workspace)
   - [ ] Verify `cargo check -p luminos-core` passes
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Verify dependency versions match workspace versions

**Completion Notes:**
>

---

## Phase 2: Core Implementation

### T003 -- Implement InputProcessingTask::dispatch_event static method
**Traces to:** FR-2, AC-2.1, AC-2.2, AC-2.3
**Status:** TODO
**Files:** `crates/luminos-core/src/pipeline.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `dispatch_event_mouse_moved_updates_position` -- Create `StateManager` with default state, create `EventLoopProxy` (from a test event loop), call `dispatch_event()` with `InputEvent::MouseMoved { position: ScreenPoint { x: 500, y: 300 } }`. Load state, verify `mouse_position == ScreenPoint { x: 500, y: 300 }`
   - [ ] `dispatch_event_key_event_matching_hotkey_changes_zoom` -- Create `StateManager` with default state (zoom=2.0), create `HotkeyMatcher::default()`, call `dispatch_event()` with `InputEvent::KeyEvent { code: KeyCode::Equal, pressed: true, modifiers: Modifiers { ctrl: true, alt: true, shift: false, meta: false } }`. Load state, verify `zoom_level == 3.0`
   - [ ] `dispatch_event_key_event_no_match_no_state_change` -- Call `dispatch_event()` with `InputEvent::KeyEvent { code: KeyCode::A, pressed: true, modifiers: Modifiers { ctrl: true, alt: true, .. } }`. Load state, verify zoom unchanged from default
   - [ ] `dispatch_event_mouse_button_ignored` -- Call `dispatch_event()` with `InputEvent::MouseButton { button: MouseButton::Left, pressed: true, position: ScreenPoint { x: 100, y: 100 } }`. Load state, verify `mouse_position` unchanged from default (0, 0)
   - [ ] `dispatch_event_scroll_ignored` -- Call `dispatch_event()` with `InputEvent::Scroll { delta_x: 0.0, delta_y: 1.0, position: ScreenPoint { x: 100, y: 100 } }`. Load state, verify no state change
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `InputProcessingTask::dispatch_event()` as a static method:
     - Match on `InputEvent::MouseMoved { position }` -> call `state_manager.update_mouse_position(*position)`, then `event_loop_proxy.send_event(LuminosEvent::StateChanged)` (ignore result)
     - Match on `InputEvent::KeyEvent { .. }` -> call `hotkey_matcher.match_event(event)`, if `Some(action)` then `dispatch_hotkey(action, state_manager)` and send `StateChanged`
     - Match on `InputEvent::MouseButton { .. } | InputEvent::Scroll { .. }` -> no-op
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to `dispatch_event()` explaining the dispatch logic

**Completion Notes:**
>

---

### T004 -- Implement InputProcessingTask::run synchronous event loop
**Traces to:** FR-2, FR-4, AC-5.2
**Status:** TODO
**Files:** `crates/luminos-core/src/pipeline.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `run_exits_on_channel_close` -- Create a `tokio::sync::mpsc::channel(4)`, drop the sender immediately. Create StateManager, HotkeyMatcher, EventLoopProxy. Call `InputProcessingTask::run()` in a thread. Verify the thread completes (join succeeds within 1 second)
   - [ ] `run_processes_events_until_channel_close` -- Create channel, send 3 `MouseMoved` events and then drop the sender. Run `InputProcessingTask::run()` in a thread. After join, load state and verify `mouse_position` matches the last sent position
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `InputProcessingTask::run()` as a private method:
     - `loop { match receiver.blocking_recv() { Some(event) => dispatch_event(...), None => { log::info!("..."); break; } } }`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comment explaining that `blocking_recv()` blocks the OS thread (correct for a dedicated thread, does not require tokio runtime)

**Completion Notes:**
>

---

### T005 -- Implement InputProcessingTask::spawn and join
**Traces to:** FR-2, FR-5, AC-5.1, AC-5.2
**Status:** TODO
**Files:** `crates/luminos-core/src/pipeline.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `spawn_creates_named_thread` -- Call `InputProcessingTask::spawn()` with a channel receiver. Verify the returned `InputProcessingTask` is valid (non-panicking construction). Drop sender, call `join()`, verify it completes
   - [ ] `spawn_and_dispatch_events` -- Spawn task, send `MouseMoved { position: ScreenPoint { x: 42, y: 84 } }` on the channel, wait briefly (10ms), load state, verify `mouse_position == (42, 84)`. Drop sender, join
   - [ ] `join_after_channel_close` -- Spawn task, drop sender immediately, call `join()`, verify it completes within 1 second (no hang)
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `InputProcessingTask` struct with `thread_handle: Option<JoinHandle<()>>` field
   - [ ] Implement `spawn()`: use `thread::Builder::new().name("luminos-input-processor".to_string()).spawn(...)` to call `Self::run()` in the new thread. Return `Self { thread_handle: Some(handle) }`
   - [ ] Implement `join(mut self)`: take the `JoinHandle` from the `Option`, call `handle.join()`, ignore the result
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to `spawn()` and `join()` explaining lifecycle semantics
   - [ ] Consider whether the `.expect("failed to spawn input processing thread")` in `spawn()` is acceptable (it is -- thread spawn failure during startup is unrecoverable)

**Completion Notes:**
>

---

### T006 -- Implement InputProcessingTask Send assertion and trait bounds
**Traces to:** NFR-4
**Status:** TODO
**Files:** `crates/luminos-core/src/pipeline.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `input_processing_task_is_send` -- Static assertion: `fn assert_send<T: Send>() {}; assert_send::<InputProcessingTask>();`
2. **Green** -- Should pass automatically (`InputProcessingTask` contains `Option<JoinHandle<()>>`, which is `Send`)
3. **Refactor** -- None expected

**Completion Notes:**
>

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [ ] All Phase 1 + Phase 2 tests pass
- [ ] `cargo clippy -p luminos-core --all-targets -- -D warnings` clean
- [ ] `cargo fmt --all -- --check` clean

---

## Phase 3: Integration

### T007 -- Integration test: mouse move via xdotool updates AppState
**Traces to:** AC-2.1, AC-3.1, AC-6.1, AC-7.2
**Status:** TODO
**Files:** `crates/luminos-core/tests/e03_integration.rs` (or within `pipeline.rs` test module, gated behind `ci_platform_tests`)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `integration_mouse_move_updates_state` -- Gate with `#[cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]`. Create `X11InputMonitor::new()`, call `subscribe_input_events(32)`, create `StateManager` (default state), `HotkeyMatcher::default()`. Create a mock/stub `EventLoopProxy` (or use a real event loop on a separate thread). Spawn `InputProcessingTask::spawn()`. Execute `xdotool mousemove 500 300` via `std::process::Command`. Wait up to 200ms (polling state every 10ms). Load state, verify `mouse_position` is approximately `(500, 300)` (allow +/- 5px tolerance for Xvfb cursor warping). Drop sender, join task.
   - [ ] `integration_arcswap_cross_thread_visibility` -- Write `mouse_position` from the input processing thread (via xdotool), read from the test thread via `state_manager.load()`. Verify the written value is visible without additional synchronization (ArcSwap guarantee).
2. **Green** -- Wire the full input pipeline: `X11InputMonitor` -> channel -> `InputProcessingTask` -> `StateManager`. Use `xdotool` for input simulation.
3. **Refactor** -- Extract test helper: `fn spawn_test_pipeline() -> (StateManager, InputProcessingTask, ...)` to reduce boilerplate across integration tests

**Completion Notes:**
>

---

### T008 -- Integration test: hotkey via xdotool changes zoom and toggles magnification
**Traces to:** AC-2.2, AC-4.1, AC-4.2, AC-7.2
**Status:** TODO
**Files:** `crates/luminos-core/tests/e03_integration.rs` (gated behind `ci_platform_tests`)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `integration_hotkey_zoom_in` -- Gate with `ci_platform_tests`. Start full pipeline (X11InputMonitor + InputProcessingTask + StateManager at default zoom 2.0). Execute `xdotool key ctrl+alt+equal`. Wait up to 200ms (polling state every 10ms). Load state, verify `settings.magnification.zoom_level == 3.0` (2.0 * 1.5).
   - [ ] `integration_hotkey_toggle_magnification` -- Start full pipeline with `is_active = false` (default). Execute `xdotool key ctrl+alt+8`. Wait up to 200ms. Load state, verify `is_active == true`.
   - [ ] `integration_hotkey_zoom_out` -- Start at zoom 3.0 (set via `StateManager::update_zoom_level(3.0)` before starting). Execute `xdotool key ctrl+alt+minus`. Wait up to 200ms. Verify `zoom_level == 2.0` (3.0 / 1.5).
   - [ ] `integration_hotkey_zoom_reset` -- Start at zoom 4.5 (set via StateManager). Execute `xdotool key ctrl+alt+0`. Wait up to 200ms. Verify `zoom_level == 2.0` (reset to default).
2. **Green** -- Tests should pass with existing implementation (InputProcessingTask dispatches KeyEvent to HotkeyMatcher)
3. **Refactor** -- Extract `fn wait_for_state_condition(state_manager, predicate, timeout_ms)` helper to reduce polling boilerplate

**Completion Notes:**
>

---

### T009 -- Integration test: graceful shutdown and ArcSwap load latency
**Traces to:** AC-5.1, AC-5.2, AC-6.2
**Status:** TODO
**Files:** `crates/luminos-core/tests/e03_integration.rs` (gated behind `ci_platform_tests`)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `integration_graceful_shutdown_channel_close` -- Start full pipeline. Drop the `X11InputMonitor` (which drops the channel sender). Verify `InputProcessingTask::join()` completes within 2 seconds (thread exits when `blocking_recv()` returns `None`).
   - [ ] `integration_arcswap_load_latency_under_100ns` -- Create `StateManager`, warm up with 1,000 `load()` calls. Measure 1,000,000 `load()` calls using `std::time::Instant` and `std::hint::black_box()`. Calculate average. Assert average < 100ns.
2. **Green** -- Shutdown test should pass (channel close triggers thread exit). Latency test should pass (ArcSwap's documented performance).
3. **Refactor** -- None expected

**Completion Notes:**
>

---

### T010 -- Integration test: frame timing under rapid mouse movement
**Traces to:** AC-3.2, NFR-1
**Status:** TODO
**Files:** `crates/luminos-core/tests/e03_integration.rs` (gated behind `ci_platform_tests`)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `integration_rapid_mouse_movement_no_stall` -- Start full pipeline. Execute 50 rapid `xdotool mousemove` calls in a tight loop (varying positions across the screen, e.g., (100+i*30, 100+i*10) for i in 0..50). After each batch of 10 moves, load state and record timestamp. Calculate delta between consecutive reads. Assert no single state update takes more than 50ms (relaxed threshold for CI software rendering; production target is 16.67ms). Verify final `mouse_position` approximately matches the last xdotool target.
   - [ ] `integration_mouse_event_propagation_latency` -- Execute a single `xdotool mousemove 800 600`. Record timestamp immediately before. Poll state every 1ms up to 100ms. Record timestamp when `mouse_position` changes. Assert propagation latency < 50ms (relaxed for CI; production target < 16.67ms per SC1).
2. **Green** -- Tests should pass with existing pipeline (channel capacity 32, dedicated threads).
3. **Refactor** -- None expected

**Completion Notes:**
>

**Checkpoint:** After completing Phase 3, run full test suite and verify:
- [ ] All Phase 1-3 tests pass
- [ ] `cargo clippy -p luminos-core --all-targets -- -D warnings` clean
- [ ] `cargo fmt --all -- --check` clean
- [ ] Integration tests pass under `xvfb-run` with `ci_platform_tests` feature

---

## Phase 4: Polish & Acceptance

### T011 -- Acceptance test verification
**Traces to:** All ACs
**Status:** TODO

**Verification Checklist:**
- [ ] AC-1.1: Per-frame render cycle (load state, tracking update, compute region, render) -- verified by architecture review and integration test pipeline wiring
- [ ] AC-1.2: Idle pipeline continues rendering (no crash) -- verified by integration tests running without timeout
- [ ] AC-1.3: `StateChanged` event triggers immediate redraw request -- verified by unit test (T003 dispatch sends event)
- [ ] AC-2.1: `xdotool mousemove` updates `AppState.mouse_position` (integration test T007)
- [ ] AC-2.2: `xdotool key ctrl+alt+equal` changes zoom level (integration test T008)
- [ ] AC-2.3: `MouseButton` and `Scroll` events ignored (unit test T003)
- [ ] AC-3.1: Mouse move propagates to state within 2 frames (integration test T010)
- [ ] AC-3.2: P99 frame time < 20ms during rapid movement (< 50ms on CI) (integration test T010)
- [ ] AC-4.1: Zoom in from 2.0 to 3.0 via hotkey (integration test T008)
- [ ] AC-4.2: Toggle magnification via hotkey (integration test T008)
- [ ] AC-5.1: `RequestExit` / channel close stops pipeline (integration test T009)
- [ ] AC-5.2: Input processing thread exits on channel close (integration test T009)
- [ ] AC-6.1: ArcSwap cross-thread visibility (integration test T007)
- [ ] AC-6.2: ArcSwap load latency < 100ns (benchmark test T009)
- [ ] AC-7.1: `xdotool` available in CI test-platform runner (verified by T001 CI change + integration tests passing)
- [ ] AC-7.2: All `ci_platform_tests` pass under `xvfb-run` (verified by CI pipeline)
- [ ] All clippy warnings resolved (`RUSTFLAGS="--deny warnings" cargo clippy -p luminos-core -p luminos-platform`)
- [ ] No `unwrap()` in production code paths (test code only)
- [ ] `cargo fmt --all -- --check` clean
- [ ] Update HIGH_LEVEL_PLAN.md Shared Context with integration findings (e.g., actual propagation latencies, any discovered constraints)

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
