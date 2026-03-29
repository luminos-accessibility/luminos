# Subtasks: Story E03/002 -- ArcSwap State Management & EventLoopProxy

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
| 1. Setup | 1 | 0 | 0 | 1 |
| 2. Core Implementation | 5 | 0 | 0 | 5 |
| 3. Integration | 2 | 0 | 0 | 2 |
| 4. Polish & Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **9** | **0** | **0** | **9** |

---

## Phase 1: Setup

### T001 -- Extend AppState with mouse_position and create module scaffolding
**Traces to:** FR-5, AC-4.1, AC-4.2
**Status:** TODO
**Files:** `crates/luminos-core/src/state.rs`, `crates/luminos-core/src/state_manager.rs`, `crates/luminos-core/src/event.rs`, `crates/luminos-core/src/lib.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `app_state_default_mouse_position_at_origin` -- `AppState::default().mouse_position == ScreenPoint { x: 0, y: 0 }`
   - [ ] `app_state_clone_preserves_mouse_position` -- Create `AppState` with `mouse_position: ScreenPoint { x: 100, y: 200 }`, clone, verify clone has same position
2. **Green** -- Implement minimum code to pass:
   - [ ] Add `pub mouse_position: ScreenPoint` field to `AppState` in `state.rs`
   - [ ] Add `use luminos_types::ScreenPoint;` import
   - [ ] Update `Default::default()` to include `mouse_position: ScreenPoint { x: 0, y: 0 }`
   - [ ] Create empty `crates/luminos-core/src/state_manager.rs` with module doc-comment
   - [ ] Create empty `crates/luminos-core/src/event.rs` with module doc-comment
   - [ ] Add `pub mod state_manager;` and `pub mod event;` to `lib.rs`
   - [ ] Add re-exports to `lib.rs`: `pub use state_manager::StateManager;` and `pub use event::LuminosEvent;`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Verify all existing tests in `state.rs` still pass (the new field may require adjustments to test assertions)

**Completion Notes:**
>

---

## Phase 2: Core Implementation

### T002 -- Implement LuminosEvent enum
**Traces to:** FR-4, AC-3.1, AC-3.2
**Status:** TODO
**Files:** `crates/luminos-core/src/event.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `luminos_event_state_changed_debug` -- `format!("{:?}", LuminosEvent::StateChanged)` contains "StateChanged"
   - [ ] `luminos_event_request_exit_debug` -- `format!("{:?}", LuminosEvent::RequestExit)` contains "RequestExit"
   - [ ] `luminos_event_is_send` -- Static assertion: `fn assert_send<T: Send>() {}; assert_send::<LuminosEvent>();`
   - [ ] `luminos_event_is_clone` -- Clone `LuminosEvent::StateChanged`, verify match
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `LuminosEvent` enum with variants `StateChanged` and `RequestExit`
   - [ ] Derive `Debug`, `Clone`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to each variant explaining their purpose and when they are sent

**Completion Notes:**
>

---

### T003 -- Implement StateManager constructor and load
**Traces to:** FR-1, FR-2, FR-6, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-core/src/state_manager.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `state_manager_new_and_load` -- Create `Arc<ArcSwap<AppState>>` with default state, create `StateManager`, call `load()`, verify returns default `AppState`
   - [ ] `state_manager_load_returns_guard` -- Call `load()`, dereference to `&AppState`, verify fields accessible
   - [ ] `state_manager_is_clone` -- Clone `StateManager`, load from clone, verify same state
   - [ ] `state_manager_inner_returns_shared_arc` -- `inner()` returns the same `Arc` (pointer equality)
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `StateManager` struct with `state: Arc<ArcSwap<AppState>>` field
   - [ ] Derive `Clone`
   - [ ] Implement `StateManager::new(state: Arc<ArcSwap<AppState>>) -> Self`
   - [ ] Implement `StateManager::load() -> Guard<Arc<AppState>>` calling `self.state.load()`
   - [ ] Implement `StateManager::inner() -> Arc<ArcSwap<AppState>>`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to the struct and all public methods

**Completion Notes:**
>

---

### T004 -- Implement StateManager::update_mouse_position
**Traces to:** FR-3, AC-2.1
**Status:** TODO
**Files:** `crates/luminos-core/src/state_manager.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `state_manager_update_mouse_position_basic` -- Update to `(500, 300)`, `load()`, assert `mouse_position == ScreenPoint { x: 500, y: 300 }`
   - [ ] `state_manager_update_mouse_position_preserves_other_fields` -- Set zoom to 5.0, then update mouse position, verify zoom is still 5.0
   - [ ] `state_manager_update_mouse_position_negative_coords` -- Update to `(-100, -50)`, verify negative coordinates preserved (valid for multi-monitor setups)
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `update_mouse_position(&self, position: ScreenPoint)`:
     - Call `self.state.rcu(|current| { let mut new = (**current).clone(); new.mouse_position = position; new });`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] None expected

**Completion Notes:**
>

---

### T005 -- Implement StateManager::update_zoom_level, toggle_magnification, reset_zoom
**Traces to:** FR-3, AC-2.2, AC-2.3, AC-2.4
**Status:** TODO
**Files:** `crates/luminos-core/src/state_manager.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `state_manager_update_zoom_level_valid` -- Update to 5.0, verify `settings.magnification.zoom_level == 5.0`
   - [ ] `state_manager_update_zoom_level_clamp_high` -- Update to 25.0, verify clamped to 20.0
   - [ ] `state_manager_update_zoom_level_clamp_low` -- Update to 0.5, verify clamped to 1.5
   - [ ] `state_manager_toggle_magnification_on_to_off` -- Set `is_active = true`, toggle, verify `is_active == false`
   - [ ] `state_manager_toggle_magnification_off_to_on` -- Default (is_active = false), toggle, verify `is_active == true`
   - [ ] `state_manager_toggle_magnification_double_toggle` -- Toggle twice, verify back to original
   - [ ] `state_manager_reset_zoom_to_default` -- Set zoom to 10.0, reset, verify zoom is 2.0
   - [ ] `state_manager_reset_zoom_preserves_other_settings` -- Set `is_active = true` and zoom to 10.0, reset zoom, verify `is_active` still true
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `update_zoom_level(&self, level: f32)`: clamp to [1.5, 20.0], `rcu()` write
   - [ ] Implement `toggle_magnification(&self)`: `rcu()` with `!is_active`
   - [ ] Implement `reset_zoom(&self)`: `rcu()` with `zoom_level = 2.0`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Define zoom bounds as named constants: `MIN_ZOOM: f32 = 1.5`, `MAX_ZOOM: f32 = 20.0`, `DEFAULT_ZOOM: f32 = 2.0`

**Completion Notes:**
>

---

### T006 -- Implement StateManager Send + Sync verification
**Traces to:** NFR-3
**Status:** TODO
**Files:** `crates/luminos-core/src/state_manager.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `state_manager_is_send` -- Static assertion: `fn assert_send<T: Send>() {}; assert_send::<StateManager>();`
   - [ ] `state_manager_is_sync` -- Static assertion: `fn assert_sync<T: Sync>() {}; assert_sync::<StateManager>();`
2. **Green** -- Should pass automatically (StateManager contains `Arc<ArcSwap<AppState>>`, both `Send + Sync`)
3. **Refactor** -- None expected

**Completion Notes:**
>

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [ ] All Phase 1 + Phase 2 tests pass
- [ ] `cargo clippy -p luminos-core --all-targets -- -D warnings` clean
- [ ] `cargo fmt --all -- --check` clean
- [ ] All existing luminos-core tests still pass (no regressions from AppState change)

---

## Phase 3: Integration

### T007 -- Cross-thread state visibility test
**Traces to:** AC-1.3, AC-2.5
**Status:** TODO
**Files:** `crates/luminos-core/src/state_manager.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `state_manager_cross_thread_visibility` -- Spawn writer thread: `update_mouse_position(ScreenPoint { x: 999, y: 888 })`. In main thread (after writer completes): `load()` and assert `mouse_position == (999, 888)`. Use `std::sync::Barrier` to synchronize.
   - [ ] `state_manager_concurrent_writers_no_lost_updates` -- Spawn two threads: thread A writes `is_active = true` (via toggle), thread B writes `zoom_level = 5.0`. After both complete, verify both `is_active == true` AND `zoom_level == 5.0` in the final state. Repeat 100 times for statistical confidence.
2. **Green** -- Tests should pass with existing `rcu()` implementation (ArcSwap retries on contention)
3. **Refactor** -- None expected

**Completion Notes:**
>

---

### T008 -- ArcSwap load latency benchmark
**Traces to:** AC-1.2, NFR-1
**Status:** TODO
**Files:** `crates/luminos-core/src/state_manager.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `state_manager_load_latency_under_100ns` -- Create `StateManager`, warm up with 1000 `load()` calls, then measure 100,000 `load()` calls. Calculate average. Assert average < 100ns. Use `std::time::Instant` and `std::hint::black_box()` to prevent optimization.
2. **Green** -- Should pass with ArcSwap's documented performance characteristics
3. **Refactor** -- None expected

**Completion Notes:**
>

---

## Phase 4: Polish & Acceptance

### T009 -- Acceptance test verification
**Traces to:** All ACs
**Status:** TODO

**Verification Checklist:**
- [ ] AC-1.1: `StateManager::load()` returns `Guard` reference without locks (unit test)
- [ ] AC-1.2: `load()` average latency < 100ns (benchmark test)
- [ ] AC-1.3: Cross-thread visibility verified (integration test)
- [ ] AC-2.1: `update_mouse_position()` updates `mouse_position` field (unit test)
- [ ] AC-2.2: `update_zoom_level()` updates `zoom_level` field (unit test)
- [ ] AC-2.3: `toggle_magnification()` flips `is_active` (unit test)
- [ ] AC-2.4: `reset_zoom()` sets zoom to 2.0 default (unit test)
- [ ] AC-2.5: Concurrent writers verified -- no lost updates (integration test)
- [ ] AC-3.1: `LuminosEvent::StateChanged` is constructable and `Send` (unit test)
- [ ] AC-3.2: `LuminosEvent::RequestExit` is constructable and matchable (unit test)
- [ ] AC-3.3: `LuminosEvent: Clone` verified (unit test); `EventLoopProxy<LuminosEvent>: Send` verified at compile time in Story 005
- [ ] AC-4.1: `AppState::default().mouse_position == (0, 0)` (unit test)
- [ ] AC-4.2: Clone preserves `mouse_position` (unit test)
- [ ] All clippy warnings resolved (`RUSTFLAGS="--deny warnings" cargo clippy -p luminos-core`)
- [ ] No `unwrap()` in production code paths
- [ ] `cargo fmt --all -- --check` clean
- [ ] All existing luminos-core tests pass (no regressions)
- [ ] Update HIGH_LEVEL_PLAN.md Shared Context with any implementation findings

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
