# Story E03/004: Global Keyboard Shortcuts

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DONE (2026-03-29)
**Depends On:** 001 (X11 Global Input Monitoring), 002 (ArcSwap State Management & EventLoopProxy)

---

## Problem Statement

After E02, the magnification overlay has a fixed zoom level set at initialization -- there is no way for the user to change the zoom level, toggle magnification on or off, or reset the zoom while the application is running. After E03 stories 001 and 002, keyboard events flow from the X11 input monitor through a bounded channel and the `ArcSwap<AppState>` infrastructure is in place, but nothing interprets those key events as actions.

This story implements the global keyboard shortcut system: a `HotkeyMatcher` that detects specific key combinations from the `InputEvent::KeyEvent` stream, and a dispatch function that executes the corresponding state mutations via `StateManager`. Four hardcoded shortcuts following accessibility tool conventions (Ctrl+Alt prefix) are implemented: zoom in, zoom out, toggle magnification, and reset zoom. State changes are written to `ArcSwap<AppState>` and trigger a `LuminosEvent::StateChanged` via `EventLoopProxy` to wake the render loop. This gives the user real-time control over their magnification experience using the keyboard alone -- essential for accessibility users who may not be able to precisely manipulate a mouse.

## User Scenarios

### US-1: Zoom In via Keyboard Shortcut

As a low-vision user, I want to press Ctrl+Alt+= to increase the magnification level so that I can get a closer look at screen content.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given magnification is active with zoom level 2.0, when the user presses Ctrl+Alt+= (Equal key with Ctrl and Alt modifiers), then the zoom level increases by a multiplicative factor of 1.5 to 3.0.
- **AC-1.2:** Given magnification is active with zoom level 2.0, when the user presses Ctrl+Alt+NumpadAdd, then the zoom level increases by a multiplicative factor of 1.5 to 3.0 (numpad alternative).
- **AC-1.3:** Given the zoom level is at the maximum (20.0), when the user presses Ctrl+Alt+=, then the zoom level remains at 20.0 (clamped to maximum).

### US-2: Zoom Out via Keyboard Shortcut

As a low-vision user, I want to press Ctrl+Alt+- to decrease the magnification level so that I can see more of the screen at a lower zoom.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given magnification is active with zoom level 3.0, when the user presses Ctrl+Alt+- (Minus key with Ctrl and Alt modifiers), then the zoom level decreases by dividing by 1.5 to 2.0.
- **AC-2.2:** Given magnification is active with zoom level 3.0, when the user presses Ctrl+Alt+NumpadSubtract, then the zoom level decreases by dividing by 1.5 to 2.0 (numpad alternative).
- **AC-2.3:** Given the zoom level is at the minimum (1.5), when the user presses Ctrl+Alt+-, then the zoom level remains at 1.5 (clamped to minimum).

### US-3: Toggle Magnification via Keyboard Shortcut

As a low-vision user, I want to press Ctrl+Alt+8 to toggle magnification on and off so that I can quickly switch between magnified and unmagnified views.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given magnification is active (`is_active = true`), when the user presses Ctrl+Alt+8, then magnification is deactivated (`is_active = false`).
- **AC-3.2:** Given magnification is inactive (`is_active = false`), when the user presses Ctrl+Alt+8, then magnification is activated (`is_active = true`).
- **AC-3.3:** Given magnification is toggled off and then on again, when the render thread reads `AppState` on the next frame, then the zoom level and viewport position are preserved from before the toggle-off (no state reset on toggle).

### US-4: Reset Zoom via Keyboard Shortcut

As a low-vision user, I want to press Ctrl+Alt+0 to reset zoom to the default level so that I can quickly return to a known magnification state.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given magnification is active with zoom level 5.0, when the user presses Ctrl+Alt+0 (Key0 with Ctrl and Alt modifiers), then the zoom level resets to the default value of 2.0.
- **AC-4.2:** Given magnification is active with zoom level 5.0, when the user presses Ctrl+Alt+Numpad0, then the zoom level resets to the default value of 2.0 (numpad alternative).

### US-5: Shortcut Matching Correctness

As the magnification system, I need the hotkey matcher to correctly distinguish the four registered shortcuts from other key combinations so that random key presses do not trigger unintended actions.

**Priority:** P0
**Acceptance Criteria:**

- **AC-5.1:** Given a key press of Ctrl+Alt+= (correct modifiers and key), when `HotkeyMatcher::match_event()` is called, then it returns `Some(HotkeyAction::ZoomIn)`.
- **AC-5.2:** Given a key press of Ctrl+Shift+= (wrong modifier combination), when `HotkeyMatcher::match_event()` is called, then it returns `None`.
- **AC-5.3:** Given a key press of Ctrl+Alt+= with `pressed = false` (key release, not press), when `HotkeyMatcher::match_event()` is called, then it returns `None` -- shortcuts trigger only on key press.
- **AC-5.4:** Given a non-KeyEvent `InputEvent` (e.g., `MouseMoved`), when `match_event()` is called, then it returns `None`.
- **AC-5.5:** Given a key press of Ctrl+Alt+Shift+= (extra modifier held), when `HotkeyMatcher::match_event()` is called, then it returns `None` -- exact modifier match is required (no extra modifiers).

## Functional Requirements

- **FR-1:** Implement `HotkeyMatcher` struct in `crates/luminos-core/src/hotkeys.rs` that holds the mapping from `(KeyCode, Modifiers)` to `HotkeyAction`. *(Traced by US-5)*
- **FR-2:** Implement `HotkeyMatcher::default()` that initializes the hardcoded ZoomText-convention shortcut table: *(Traced by US-1, US-2, US-3, US-4)*
  - `Ctrl+Alt+Equal` and `Ctrl+Alt+NumpadAdd` -> `HotkeyAction::ZoomIn`
  - `Ctrl+Alt+Minus` and `Ctrl+Alt+NumpadSubtract` -> `HotkeyAction::ZoomOut`
  - `Ctrl+Alt+8` -> `HotkeyAction::ToggleMagnification`
  - `Ctrl+Alt+Key0` and `Ctrl+Alt+Numpad0` -> `HotkeyAction::ZoomReset`
- **FR-3:** Implement `HotkeyMatcher::match_event(&self, event: &InputEvent) -> Option<HotkeyAction>` that returns the matched action only on key press (not release), with exact modifier matching (no extra modifiers). *(Traced by AC-5.1, AC-5.2, AC-5.3, AC-5.4, AC-5.5)*
- **FR-4:** Implement `dispatch_hotkey(action: HotkeyAction, state_manager: &StateManager)` function that executes the state mutation corresponding to the matched action: *(Traced by AC-1.1, AC-2.1, AC-3.1, AC-4.1)*
  - `ZoomIn`: multiply current zoom by 1.5, clamped to maximum 20.0
  - `ZoomOut`: divide current zoom by 1.5, clamped to minimum 1.5
  - `ToggleMagnification`: flip `is_active`
  - `ZoomReset`: set zoom to default 2.0
- **FR-5:** After each state mutation, send `LuminosEvent::StateChanged` via `EventLoopProxy` to wake the render loop. *(Traced by US-1, US-2, US-3, US-4)*
- **FR-6:** Reuse the existing `HotkeyAction` enum from `luminos-core::config::schema`. *(Traced by FR-1)*

## Non-Functional Requirements

- **NFR-1:** Hotkey matching must complete in under 1 microsecond per event -- it is a `HashMap` lookup with no allocation. *(Source: input processing must not add latency to the 16.67ms frame budget)*
- **NFR-2:** State mutations via `ArcSwap::rcu()` must be visible to the render thread on the next frame read. *(Source: E03 SC4)*
- **NFR-3:** No `unwrap()` or `expect()` in production code.
- **NFR-4:** All public items in `hotkeys.rs` must have `///` doc-comments.
- **NFR-5:** Zoom level must always stay within the valid range [1.5, 20.0] regardless of how many times zoom in/out is pressed. *(Source: doc-03 Section 1.3 zoom range constraint)*

## Out of Scope

- Configurable keybindings (loading bindings from `AppSettings.keybindings`) -- deferred to E07.
- Additional hotkey actions beyond the four implemented (CycleMode, ReadWhatISee, ReadSelection, StopSpeech, FindCursor) -- deferred to E07/E11.
- Visual feedback for hotkey activation (e.g., OSD showing current zoom level) -- deferred to E06.
- Key repeat handling (holding Ctrl+Alt+= to continuously zoom in) -- initial implementation triggers once per key press event; repeat behavior depends on X11 auto-repeat which generates separate press events.
- Hotkey conflict detection with other applications -- not in Phase 0 scope.
- Integration with the event-driven render loop (consuming key events from the mpsc channel and dispatching to `HotkeyMatcher`) -- handled by Story 005.

## Open Questions

*None -- shortcut assignments and zoom step factor resolved in HIGH_LEVEL_PLAN.md architecture decisions.*
