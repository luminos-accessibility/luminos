# Subtasks: Story E03/004 -- Global Keyboard Shortcuts

**Status:** COMPLETE
**Started:** 2026-03-29
**Completed:** 2026-03-29
**Story:** [STORY.md](STORY.md)
**Design:** [DESIGN.md](DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 1 | 1 | 0 | 0 |
| 2. Core Implementation | 4 | 4 | 0 | 0 |
| 3. Integration | 1 | 1 | 0 | 0 |
| 4. Polish & Acceptance | 1 | 1 | 0 | 0 |
| **Total** | **7** | **7** | **0** | **0** |

---

## Phase 1: Setup

### T001 -- Add Hash derive to Modifiers and create hotkeys module scaffolding
**Traces to:** FR-1, FR-6
**Status:** DONE
**Files:** `crates/luminos-platform/src/traits/input_monitor.rs`, `crates/luminos-core/src/hotkeys.rs`, `crates/luminos-core/src/lib.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `modifiers_hash_consistent` -- Verify two identical `Modifiers` values produce the same hash (required for HashMap key correctness)
2. **Green** -- Implement minimum code to pass:
   - [ ] Add `Hash` to the derive list on `Modifiers` in `luminos-platform/src/traits/input_monitor.rs`: `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]`
   - [ ] Create `crates/luminos-core/src/hotkeys.rs` with module doc-comment
   - [ ] Add `pub mod hotkeys;` to `luminos-core/src/lib.rs`
   - [ ] Add re-export: `pub use hotkeys::HotkeyMatcher;`
   - [ ] Verify `cargo check -p luminos-platform -p luminos-core` passes
3. **Refactor** -- Clean up while tests stay green:
   - [x] Verify no regressions in existing `input_monitor.rs` tests

**Completion Notes:**
> Hash derive was pre-applied to Modifiers by team lead. Module scaffolding (`pub mod hotkeys;`, re-export) also pre-applied. Wrote `modifiers_hash_consistent` test verifying identical Modifiers produce same hash. All input_monitor tests unaffected.

---

## Phase 2: Core Implementation

### T002 -- Implement HotkeyMatcher with default bindings
**Traces to:** FR-1, FR-2, AC-5.1
**Status:** DONE
**Files:** `crates/luminos-core/src/hotkeys.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `hotkey_matcher_default_has_seven_bindings` -- `HotkeyMatcher::default()` internal map has 7 entries (4 actions * 2 key variants - 1 for Key8 which has no numpad alt)
   - [ ] `hotkey_matcher_zoom_in_equal` -- Match `KeyEvent { code: Equal, pressed: true, modifiers: ctrl_alt }` returns `Some(ZoomIn)`
   - [ ] `hotkey_matcher_zoom_in_numpad_add` -- Match `KeyEvent { code: NumpadAdd, pressed: true, modifiers: ctrl_alt }` returns `Some(ZoomIn)`
   - [ ] `hotkey_matcher_zoom_out_minus` -- Match `KeyEvent { code: Minus, pressed: true, modifiers: ctrl_alt }` returns `Some(ZoomOut)`
   - [ ] `hotkey_matcher_zoom_out_numpad_subtract` -- Match with `NumpadSubtract` returns `Some(ZoomOut)`
   - [ ] `hotkey_matcher_toggle_8` -- Match with `Key8` + ctrl_alt returns `Some(ToggleMagnification)`
   - [ ] `hotkey_matcher_reset_key0` -- Match with `Key0` + ctrl_alt returns `Some(ZoomReset)`
   - [ ] `hotkey_matcher_reset_numpad0` -- Match with `Numpad0` + ctrl_alt returns `Some(ZoomReset)`
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `HotkeyMatcher` struct with `bindings: HashMap<(KeyCode, Modifiers), HotkeyAction>`
   - [ ] Implement `HotkeyMatcher::new(bindings)` constructor
   - [ ] Implement `Default for HotkeyMatcher` with 7 hardcoded ZoomText-convention entries
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Define `ctrl_alt` modifier constant as a helper function or constant in the test module
   - [x] Add doc-comments to the struct and `Default` impl

**Completion Notes:**
> Implemented HotkeyMatcher struct with HashMap<(KeyCode, Modifiers), HotkeyAction> bindings field. new() constructor with #[must_use]. Default impl registers 7 entries. All 8 positive match tests pass (equal, numpad_add, minus, numpad_subtract, key8, key0, numpad0 + binding count). Defined ctrl_alt() test helper for reuse.

---

### T003 -- Implement match_event with exact modifier matching
**Traces to:** FR-3, AC-5.1, AC-5.2, AC-5.3, AC-5.4, AC-5.5
**Status:** DONE
**Files:** `crates/luminos-core/src/hotkeys.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `hotkey_matcher_wrong_modifiers_returns_none` -- `Ctrl+Shift+Equal` (no alt) returns `None`
   - [ ] `hotkey_matcher_extra_modifier_returns_none` -- `Ctrl+Alt+Shift+Equal` (extra shift) returns `None`
   - [ ] `hotkey_matcher_key_release_returns_none` -- `Ctrl+Alt+Equal` with `pressed=false` returns `None`
   - [ ] `hotkey_matcher_mouse_moved_returns_none` -- `InputEvent::MouseMoved { position }` returns `None`
   - [ ] `hotkey_matcher_unregistered_key_returns_none` -- `Ctrl+Alt+A` returns `None`
   - [ ] `hotkey_matcher_mouse_button_returns_none` -- `InputEvent::MouseButton { .. }` returns `None`
   - [ ] `hotkey_matcher_scroll_returns_none` -- `InputEvent::Scroll { .. }` returns `None`
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `match_event(&self, event: &InputEvent) -> Option<HotkeyAction>`:
     - Match on `InputEvent::KeyEvent { code, pressed: true, modifiers }` only
     - Look up `(code, modifiers)` in `self.bindings`
     - Return `.copied()` result (or `None` for non-KeyEvent, key release, unmatched)
3. **Refactor** -- Clean up while tests stay green:
   - [x] Add doc-comment explaining exact modifier matching semantics

**Completion Notes:**
> match_event implemented as HashMap::get().copied() on (code, modifiers) tuple, gated by pressed=true. 7 negative tests: wrong_modifiers, extra_modifier, key_release, mouse_moved, unregistered_key, mouse_button, scroll. All return None as expected. Exact modifier matching is achieved naturally by HashMap equality -- no extra modifiers check needed because Modifiers has Eq.

---

### T004 -- Implement dispatch_hotkey with zoom step logic
**Traces to:** FR-4, AC-1.1, AC-1.3, AC-2.1, AC-2.3, AC-3.1, AC-3.2, AC-3.3, AC-4.1
**Status:** DONE
**Files:** `crates/luminos-core/src/hotkeys.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `hotkey_dispatch_zoom_in_from_2` -- StateManager at zoom 2.0, dispatch `ZoomIn`, verify zoom is 3.0 (2.0 * 1.5)
   - [ ] `hotkey_dispatch_zoom_out_from_3` -- StateManager at zoom 3.0, dispatch `ZoomOut`, verify zoom is 2.0 (3.0 / 1.5)
   - [ ] `hotkey_dispatch_zoom_in_at_max` -- StateManager at zoom 20.0, dispatch `ZoomIn`, verify zoom stays 20.0 (clamped)
   - [ ] `hotkey_dispatch_zoom_out_at_min` -- StateManager at zoom 1.5, dispatch `ZoomOut`, verify zoom stays 1.5 (clamped: 1.5/1.5=1.0 -> clamped to 1.5)
   - [ ] `hotkey_dispatch_toggle_on_to_off` -- StateManager with `is_active=true`, dispatch `ToggleMagnification`, verify `is_active=false`
   - [ ] `hotkey_dispatch_toggle_off_to_on` -- Default (inactive), dispatch Toggle, verify active
   - [ ] `hotkey_dispatch_toggle_preserves_zoom` -- Set zoom 5.0, toggle off, toggle on, verify zoom still 5.0
   - [ ] `hotkey_dispatch_reset_from_5` -- StateManager at zoom 5.0, dispatch `ZoomReset`, verify zoom is 2.0
   - [ ] `hotkey_dispatch_repeated_zoom_in` -- Start at 2.0, dispatch ZoomIn 7 times. Verify progression: 3.0 -> 4.5 -> 6.75 -> 10.125 -> 15.1875 -> 20.0 -> 20.0 (clamped on last two)
   - [ ] `hotkey_dispatch_unhandled_action_silent` -- Dispatch `CycleMode` (not handled in Phase 0), verify no panic and state unchanged
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `ZOOM_STEP: f32 = 1.5` constant
   - [ ] Define `DEFAULT_ZOOM: f32 = 2.0` constant
   - [ ] Implement `dispatch_hotkey(action: HotkeyAction, state_manager: &StateManager)`:
     - `ZoomIn`: load current zoom, multiply by ZOOM_STEP, call `update_zoom_level()` (which clamps)
     - `ZoomOut`: load current zoom, divide by ZOOM_STEP, call `update_zoom_level()`
     - `ToggleMagnification`: call `toggle_magnification()`
     - `ZoomReset`: call `reset_zoom()`
     - `_ => {}` for unhandled actions
3. **Refactor** -- Clean up while tests stay green:
   - [x] Extract test helper: `generate_test_state_manager_with_zoom(zoom: f32) -> StateManager`

**Completion Notes:**
> dispatch_hotkey implemented with ZOOM_STEP=1.5 constant. ZoomIn/ZoomOut use multiplicative steps via StateManager::update_zoom_level() (which clamps to [1.5, 20.0]). Toggle and Reset delegate to StateManager methods. Wildcard arm silently ignores unhandled actions (CycleMode tested). Removed unused DEFAULT_ZOOM constant (redundant with state_manager::DEFAULT_ZOOM used by reset_zoom()). 10 dispatch tests pass including repeated zoom in progression and unhandled action.

---

### T005 -- Implement match_event + dispatch_hotkey integration
**Traces to:** AC-1.2, AC-2.2, AC-4.2
**Status:** DONE
**Files:** `crates/luminos-core/src/hotkeys.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `hotkey_match_and_dispatch_zoom_in_numpad` -- Construct `InputEvent::KeyEvent` with `NumpadAdd` + ctrl_alt, match_event, dispatch, verify zoom changes from 2.0 to 3.0
   - [ ] `hotkey_match_and_dispatch_zoom_out_numpad` -- NumpadSubtract, verify zoom decreases
   - [ ] `hotkey_match_and_dispatch_reset_numpad` -- Numpad0, verify zoom resets to 2.0
2. **Green** -- Construct full pipeline: create StateManager + HotkeyMatcher, build InputEvent, match, dispatch, verify
3. **Refactor** -- Extract test helper: `generate_test_key_event(code: KeyCode, modifiers: Modifiers, pressed: bool) -> InputEvent`

**Completion Notes:**
> 3 round-trip tests: NumpadAdd->ZoomIn (2.0->3.0), NumpadSubtract->ZoomOut (3.0->2.0), Numpad0->ZoomReset (5.0->2.0). Extracted generate_test_key_event helper. All pass.

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [x] All Phase 1 + Phase 2 tests pass
- [x] `cargo clippy -p luminos-core --all-targets -- -D warnings` clean (hotkeys.rs clean; tracking.rs has pre-existing issues from story-003 teammate)
- [x] `cargo clippy -p luminos-platform --all-targets -- -D warnings` clean (for Modifiers Hash change)
- [x] `cargo fmt --all -- --check` clean (hotkeys.rs clean; tracking.rs has pre-existing fmt diffs from story-003 teammate)

---

## Phase 3: Integration

### T006 -- Full round-trip integration test with StateManager
**Traces to:** FR-5, AC-3.3
**Status:** DONE
**Files:** `crates/luminos-core/src/hotkeys.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `hotkey_integration_full_workflow` -- Create StateManager (default state, is_active=false, zoom=2.0). Dispatch Toggle (activates). Dispatch ZoomIn (zoom to 3.0). Dispatch ZoomIn (zoom to 4.5). Dispatch ZoomOut (zoom to 3.0). Dispatch Reset (zoom to 2.0). Dispatch Toggle (deactivates). Verify final state: `is_active=false, zoom=2.0`.
   - [ ] `hotkey_integration_state_visibility_across_operations` -- After each dispatch, verify that `load()` reflects the mutation immediately (no stale reads from the same thread).
2. **Green** -- Tests should pass with existing implementation
3. **Refactor** -- None expected

**Completion Notes:**
> Full workflow test: default state -> toggle on -> zoom in x2 -> zoom out -> reset -> toggle off, verifying final state matches initial. State visibility test: verifies each dispatch is immediately visible via load() (no stale reads). Both pass.

---

## Phase 4: Polish & Acceptance

### T007 -- Acceptance test verification
**Traces to:** All ACs
**Status:** DONE

**Verification Checklist:**
- [x] AC-1.1: ZoomIn from 2.0 -> 3.0 — `hotkey_dispatch_zoom_in_from_2`
- [x] AC-1.2: ZoomIn via NumpadAdd — `hotkey_match_and_dispatch_zoom_in_numpad`
- [x] AC-1.3: ZoomIn at max 20.0 clamped — `hotkey_dispatch_zoom_in_at_max`
- [x] AC-2.1: ZoomOut from 3.0 -> 2.0 — `hotkey_dispatch_zoom_out_from_3`
- [x] AC-2.2: ZoomOut via NumpadSubtract — `hotkey_match_and_dispatch_zoom_out_numpad`
- [x] AC-2.3: ZoomOut at min 1.5 clamped — `hotkey_dispatch_zoom_out_at_min`
- [x] AC-3.1: Toggle active->inactive — `hotkey_dispatch_toggle_on_to_off`
- [x] AC-3.2: Toggle inactive->active — `hotkey_dispatch_toggle_off_to_on`
- [x] AC-3.3: Toggle preserves zoom and viewport — `hotkey_dispatch_toggle_preserves_zoom`
- [x] AC-4.1: Reset from 5.0 -> 2.0 — `hotkey_dispatch_reset_from_5`
- [x] AC-4.2: Reset via Numpad0 — `hotkey_match_and_dispatch_reset_numpad`
- [x] AC-5.1: Exact match returns Some(action) — `hotkey_matcher_zoom_in_equal`
- [x] AC-5.2: Wrong modifiers returns None — `hotkey_matcher_wrong_modifiers_returns_none`
- [x] AC-5.3: Key release returns None — `hotkey_matcher_key_release_returns_none`
- [x] AC-5.4: Non-KeyEvent returns None — `hotkey_matcher_mouse_moved_returns_none`
- [x] AC-5.5: Extra modifier returns None — `hotkey_matcher_extra_modifier_returns_none`
- [x] All clippy warnings resolved for hotkeys.rs (tracking.rs has pre-existing issues from story-003)
- [x] No `unwrap()` in production code paths (only in #[cfg(test)] block with #[allow(clippy::unwrap_used)])
- [x] `cargo fmt` clean for hotkeys.rs
- [ ] Update HIGH_LEVEL_PLAN.md Shared Context with implementation findings (deferred -- team lead responsibility)

**Completion Notes:**
> 31 tests total: 1 hash consistency, 8 positive match, 7 negative match, 10 dispatch, 3 match+dispatch round-trip, 2 full integration. All pass. Deviation: removed `DEFAULT_ZOOM` constant from hotkeys.rs as it was unused (reset_zoom() in StateManager already uses its own DEFAULT_ZOOM constant). This avoids a dead_code warning that would fail CI with --deny warnings.

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| T004 | Removed `DEFAULT_ZOOM` constant from `hotkeys.rs` | `dispatch_hotkey` calls `state_manager.reset_zoom()` which uses `state_manager::DEFAULT_ZOOM` internally. Keeping an unused constant in hotkeys.rs would cause a `dead_code` warning that fails CI with `--deny warnings`. |
