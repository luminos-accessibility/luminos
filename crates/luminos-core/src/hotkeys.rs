//! Global keyboard shortcut matching and dispatch.
//!
//! Provides [`HotkeyMatcher`] for matching keyboard events against
//! registered hotkey bindings, and [`dispatch_hotkey`] for executing
//! the corresponding state mutations via [`StateManager`].
//!
//! # Phase 0 Shortcuts
//!
//! Four `ZoomText`-convention shortcuts are registered by default:
//!
//! | Shortcut | Action |
//! |----------|--------|
//! | Ctrl+Alt+= / Ctrl+Alt+NumpadAdd | Zoom In |
//! | Ctrl+Alt+- / Ctrl+Alt+NumpadSubtract | Zoom Out |
//! | Ctrl+Alt+8 | Toggle Magnification |
//! | Ctrl+Alt+0 / Ctrl+Alt+Numpad0 | Reset Zoom |

use std::collections::HashMap;

use luminos_platform::traits::input_monitor::{InputEvent, KeyCode, Modifiers};

use crate::config::schema::HotkeyAction;
use crate::state_manager::StateManager;

/// Zoom step multiplier for zoom in/out.
///
/// `ZoomText` uses multiplicative zoom steps (not additive) for a natural
/// zoom progression: 2.0 -> 3.0 -> 4.5 -> 6.75 -> 10.125 -> 15.1875 -> 20.0.
const ZOOM_STEP: f32 = 1.5;

/// Matches keyboard events against registered hotkey bindings.
///
/// Holds a mapping from `(KeyCode, Modifiers)` pairs to `HotkeyAction`.
/// Matches only on key press (not release) with exact modifier matching
/// (no extra modifiers allowed).
///
/// # Phase 0 Shortcuts
///
/// The default matcher registers four `ZoomText`-convention shortcuts:
///
/// | Shortcut | Action |
/// |----------|--------|
/// | Ctrl+Alt+= / Ctrl+Alt+NumpadAdd | Zoom In |
/// | Ctrl+Alt+- / Ctrl+Alt+NumpadSubtract | Zoom Out |
/// | Ctrl+Alt+8 | Toggle Magnification |
/// | Ctrl+Alt+0 / Ctrl+Alt+Numpad0 | Reset Zoom |
pub struct HotkeyMatcher {
    /// Map from (key, modifiers) to action.
    bindings: HashMap<(KeyCode, Modifiers), HotkeyAction>,
}

impl HotkeyMatcher {
    /// Creates a new hotkey matcher with the given bindings.
    #[must_use]
    pub fn new(bindings: HashMap<(KeyCode, Modifiers), HotkeyAction>) -> Self {
        Self { bindings }
    }

    /// Matches an input event against registered hotkeys.
    ///
    /// Returns `Some(action)` if the event is a key press that exactly
    /// matches a registered shortcut. Returns `None` for:
    /// - Key release events
    /// - Non-KeyEvent input events
    /// - Unregistered key combinations
    /// - Key combinations with extra modifiers held
    #[must_use]
    pub fn match_event(&self, event: &InputEvent) -> Option<HotkeyAction> {
        match event {
            InputEvent::KeyEvent {
                code,
                pressed: true,
                modifiers,
            } => self.bindings.get(&(*code, *modifiers)).copied(),
            _ => None,
        }
    }
}

impl Default for HotkeyMatcher {
    /// Creates a matcher with the hardcoded Phase 0 `ZoomText` shortcuts.
    fn default() -> Self {
        let ctrl_alt = Modifiers {
            shift: false,
            ctrl: true,
            alt: true,
            meta: false,
        };

        let mut bindings = HashMap::new();

        // Zoom In: Ctrl+Alt+= and Ctrl+Alt+NumpadAdd
        bindings.insert((KeyCode::Equal, ctrl_alt), HotkeyAction::ZoomIn);
        bindings.insert((KeyCode::NumpadAdd, ctrl_alt), HotkeyAction::ZoomIn);

        // Zoom Out: Ctrl+Alt+- and Ctrl+Alt+NumpadSubtract
        bindings.insert((KeyCode::Minus, ctrl_alt), HotkeyAction::ZoomOut);
        bindings.insert((KeyCode::NumpadSubtract, ctrl_alt), HotkeyAction::ZoomOut);

        // Toggle Magnification: Ctrl+Alt+8
        bindings.insert((KeyCode::Key8, ctrl_alt), HotkeyAction::ToggleMagnification);

        // Reset Zoom: Ctrl+Alt+0 and Ctrl+Alt+Numpad0
        bindings.insert((KeyCode::Key0, ctrl_alt), HotkeyAction::ZoomReset);
        bindings.insert((KeyCode::Numpad0, ctrl_alt), HotkeyAction::ZoomReset);

        Self::new(bindings)
    }
}

/// Dispatches a hotkey action by mutating application state.
///
/// Executes the state mutation corresponding to the given `HotkeyAction`
/// via the provided `StateManager`. The caller is responsible for sending
/// `LuminosEvent::StateChanged` via `EventLoopProxy` after this call.
///
/// # Arguments
///
/// * `action` -- The hotkey action to execute.
/// * `state_manager` -- The shared state manager for read-copy-update writes.
pub fn dispatch_hotkey(action: HotkeyAction, state_manager: &StateManager) {
    match action {
        HotkeyAction::ZoomIn => {
            let current = state_manager.load();
            let new_level = current.settings.magnification.zoom_level * ZOOM_STEP;
            state_manager.update_zoom_level(new_level);
        }
        HotkeyAction::ZoomOut => {
            let current = state_manager.load();
            let new_level = current.settings.magnification.zoom_level / ZOOM_STEP;
            state_manager.update_zoom_level(new_level);
        }
        HotkeyAction::ToggleMagnification => {
            state_manager.toggle_magnification();
        }
        HotkeyAction::ZoomReset => {
            state_manager.reset_zoom();
        }
        // Actions not handled in Phase 0 -- ignore silently.
        _ => {}
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use arc_swap::ArcSwap;
    use luminos_platform::traits::input_monitor::{InputEvent, KeyCode, Modifiers};
    use luminos_types::ScreenPoint;

    use super::*;
    use crate::config::schema::HotkeyAction;
    use crate::state::AppState;

    // ---------------------------------------------------------------
    // Test helpers
    // ---------------------------------------------------------------

    /// Standard Ctrl+Alt modifier combination used by all Phase 0 shortcuts.
    fn ctrl_alt() -> Modifiers {
        Modifiers {
            shift: false,
            ctrl: true,
            alt: true,
            meta: false,
        }
    }

    /// Creates a `StateManager` with default state.
    fn generate_test_state_manager() -> StateManager {
        let shared = Arc::new(ArcSwap::from_pointee(AppState::default()));
        StateManager::new(shared)
    }

    /// Creates a `StateManager` with a specific zoom level.
    fn generate_test_state_manager_with_zoom(zoom: f32) -> StateManager {
        let mut state = AppState::default();
        state.settings.magnification.zoom_level = zoom;
        let shared = Arc::new(ArcSwap::from_pointee(state));
        StateManager::new(shared)
    }

    /// Creates a key event for testing.
    fn generate_test_key_event(code: KeyCode, modifiers: Modifiers, pressed: bool) -> InputEvent {
        InputEvent::KeyEvent {
            code,
            pressed,
            modifiers,
        }
    }

    // ---------------------------------------------------------------
    // T001 -- Modifiers Hash consistency
    // ---------------------------------------------------------------

    #[test]
    fn modifiers_hash_consistent() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = Modifiers {
            shift: false,
            ctrl: true,
            alt: true,
            meta: false,
        };
        let b = Modifiers {
            shift: false,
            ctrl: true,
            alt: true,
            meta: false,
        };

        let mut hasher_a = DefaultHasher::new();
        a.hash(&mut hasher_a);
        let mut hasher_b = DefaultHasher::new();
        b.hash(&mut hasher_b);

        assert_eq!(
            hasher_a.finish(),
            hasher_b.finish(),
            "identical Modifiers should produce the same hash"
        );
    }

    // ---------------------------------------------------------------
    // T002 -- HotkeyMatcher default bindings
    // ---------------------------------------------------------------

    #[test]
    fn hotkey_matcher_default_has_seven_bindings() {
        let matcher = HotkeyMatcher::default();
        assert_eq!(
            matcher.bindings.len(),
            7,
            "default matcher should have 7 entries (4 actions, 3 with numpad alternatives)"
        );
    }

    #[test]
    fn hotkey_matcher_zoom_in_equal() {
        let matcher = HotkeyMatcher::default();
        let event = generate_test_key_event(KeyCode::Equal, ctrl_alt(), true);
        assert_eq!(matcher.match_event(&event), Some(HotkeyAction::ZoomIn));
    }

    #[test]
    fn hotkey_matcher_zoom_in_numpad_add() {
        let matcher = HotkeyMatcher::default();
        let event = generate_test_key_event(KeyCode::NumpadAdd, ctrl_alt(), true);
        assert_eq!(matcher.match_event(&event), Some(HotkeyAction::ZoomIn));
    }

    #[test]
    fn hotkey_matcher_zoom_out_minus() {
        let matcher = HotkeyMatcher::default();
        let event = generate_test_key_event(KeyCode::Minus, ctrl_alt(), true);
        assert_eq!(matcher.match_event(&event), Some(HotkeyAction::ZoomOut));
    }

    #[test]
    fn hotkey_matcher_zoom_out_numpad_subtract() {
        let matcher = HotkeyMatcher::default();
        let event = generate_test_key_event(KeyCode::NumpadSubtract, ctrl_alt(), true);
        assert_eq!(matcher.match_event(&event), Some(HotkeyAction::ZoomOut));
    }

    #[test]
    fn hotkey_matcher_toggle_8() {
        let matcher = HotkeyMatcher::default();
        let event = generate_test_key_event(KeyCode::Key8, ctrl_alt(), true);
        assert_eq!(
            matcher.match_event(&event),
            Some(HotkeyAction::ToggleMagnification)
        );
    }

    #[test]
    fn hotkey_matcher_reset_key0() {
        let matcher = HotkeyMatcher::default();
        let event = generate_test_key_event(KeyCode::Key0, ctrl_alt(), true);
        assert_eq!(matcher.match_event(&event), Some(HotkeyAction::ZoomReset));
    }

    #[test]
    fn hotkey_matcher_reset_numpad0() {
        let matcher = HotkeyMatcher::default();
        let event = generate_test_key_event(KeyCode::Numpad0, ctrl_alt(), true);
        assert_eq!(matcher.match_event(&event), Some(HotkeyAction::ZoomReset));
    }

    // ---------------------------------------------------------------
    // T003 -- match_event exact modifier matching (negative tests)
    // ---------------------------------------------------------------

    #[test]
    fn hotkey_matcher_wrong_modifiers_returns_none() {
        let matcher = HotkeyMatcher::default();
        let ctrl_shift = Modifiers {
            shift: true,
            ctrl: true,
            alt: false,
            meta: false,
        };
        let event = generate_test_key_event(KeyCode::Equal, ctrl_shift, true);
        assert_eq!(matcher.match_event(&event), None);
    }

    #[test]
    fn hotkey_matcher_extra_modifier_returns_none() {
        let matcher = HotkeyMatcher::default();
        let ctrl_alt_shift = Modifiers {
            shift: true,
            ctrl: true,
            alt: true,
            meta: false,
        };
        let event = generate_test_key_event(KeyCode::Equal, ctrl_alt_shift, true);
        assert_eq!(matcher.match_event(&event), None);
    }

    #[test]
    fn hotkey_matcher_key_release_returns_none() {
        let matcher = HotkeyMatcher::default();
        let event = generate_test_key_event(KeyCode::Equal, ctrl_alt(), false);
        assert_eq!(matcher.match_event(&event), None);
    }

    #[test]
    fn hotkey_matcher_mouse_moved_returns_none() {
        let matcher = HotkeyMatcher::default();
        let event = InputEvent::MouseMoved {
            position: ScreenPoint { x: 100, y: 200 },
        };
        assert_eq!(matcher.match_event(&event), None);
    }

    #[test]
    fn hotkey_matcher_unregistered_key_returns_none() {
        let matcher = HotkeyMatcher::default();
        let event = generate_test_key_event(KeyCode::A, ctrl_alt(), true);
        assert_eq!(matcher.match_event(&event), None);
    }

    #[test]
    fn hotkey_matcher_mouse_button_returns_none() {
        let matcher = HotkeyMatcher::default();
        let event = InputEvent::MouseButton {
            button: luminos_platform::traits::input_monitor::MouseButton::Left,
            pressed: true,
            position: ScreenPoint { x: 0, y: 0 },
        };
        assert_eq!(matcher.match_event(&event), None);
    }

    #[test]
    fn hotkey_matcher_scroll_returns_none() {
        let matcher = HotkeyMatcher::default();
        let event = InputEvent::Scroll {
            delta_x: 0.0,
            delta_y: 1.0,
            position: ScreenPoint { x: 0, y: 0 },
        };
        assert_eq!(matcher.match_event(&event), None);
    }

    // ---------------------------------------------------------------
    // T004 -- dispatch_hotkey tests
    // ---------------------------------------------------------------

    #[test]
    fn hotkey_dispatch_zoom_in_from_2() {
        let mgr = generate_test_state_manager_with_zoom(2.0);
        dispatch_hotkey(HotkeyAction::ZoomIn, &mgr);
        let zoom = mgr.load().settings.magnification.zoom_level;
        assert!(
            (zoom - 3.0).abs() < f32::EPSILON,
            "zoom should be 3.0 (2.0 * 1.5), got {zoom}"
        );
    }

    #[test]
    fn hotkey_dispatch_zoom_out_from_3() {
        let mgr = generate_test_state_manager_with_zoom(3.0);
        dispatch_hotkey(HotkeyAction::ZoomOut, &mgr);
        let zoom = mgr.load().settings.magnification.zoom_level;
        assert!(
            (zoom - 2.0).abs() < f32::EPSILON,
            "zoom should be 2.0 (3.0 / 1.5), got {zoom}"
        );
    }

    #[test]
    fn hotkey_dispatch_zoom_in_at_max() {
        let mgr = generate_test_state_manager_with_zoom(20.0);
        dispatch_hotkey(HotkeyAction::ZoomIn, &mgr);
        let zoom = mgr.load().settings.magnification.zoom_level;
        assert!(
            (zoom - 20.0).abs() < f32::EPSILON,
            "zoom should remain clamped at 20.0, got {zoom}"
        );
    }

    #[test]
    fn hotkey_dispatch_zoom_out_at_min() {
        let mgr = generate_test_state_manager_with_zoom(1.5);
        dispatch_hotkey(HotkeyAction::ZoomOut, &mgr);
        let zoom = mgr.load().settings.magnification.zoom_level;
        assert!(
            (zoom - 1.5).abs() < f32::EPSILON,
            "zoom should remain clamped at 1.5, got {zoom}"
        );
    }

    #[test]
    fn hotkey_dispatch_toggle_on_to_off() {
        let mgr = generate_test_state_manager();
        // Activate first
        mgr.toggle_magnification();
        assert!(mgr.load().is_active, "precondition: should be active");

        dispatch_hotkey(HotkeyAction::ToggleMagnification, &mgr);
        assert!(
            !mgr.load().is_active,
            "should be inactive after toggle dispatch"
        );
    }

    #[test]
    fn hotkey_dispatch_toggle_off_to_on() {
        let mgr = generate_test_state_manager();
        assert!(!mgr.load().is_active, "precondition: default is inactive");

        dispatch_hotkey(HotkeyAction::ToggleMagnification, &mgr);
        assert!(
            mgr.load().is_active,
            "should be active after toggle dispatch"
        );
    }

    #[test]
    fn hotkey_dispatch_toggle_preserves_zoom() {
        let mgr = generate_test_state_manager_with_zoom(5.0);
        // Toggle off then on
        dispatch_hotkey(HotkeyAction::ToggleMagnification, &mgr);
        dispatch_hotkey(HotkeyAction::ToggleMagnification, &mgr);
        let zoom = mgr.load().settings.magnification.zoom_level;
        assert!(
            (zoom - 5.0).abs() < f32::EPSILON,
            "zoom should be preserved after toggle off+on, got {zoom}"
        );
    }

    #[test]
    fn hotkey_dispatch_reset_from_5() {
        let mgr = generate_test_state_manager_with_zoom(5.0);
        dispatch_hotkey(HotkeyAction::ZoomReset, &mgr);
        let zoom = mgr.load().settings.magnification.zoom_level;
        assert!(
            (zoom - 2.0).abs() < f32::EPSILON,
            "zoom should be reset to 2.0, got {zoom}"
        );
    }

    #[test]
    fn hotkey_dispatch_repeated_zoom_in() {
        let mgr = generate_test_state_manager_with_zoom(2.0);

        let expected = [3.0, 4.5, 6.75, 10.125, 15.187_5, 20.0, 20.0];
        for (i, &exp) in expected.iter().enumerate() {
            dispatch_hotkey(HotkeyAction::ZoomIn, &mgr);
            let zoom = mgr.load().settings.magnification.zoom_level;
            assert!(
                (zoom - exp).abs() < 0.001,
                "step {}: zoom should be {exp}, got {zoom}",
                i + 1
            );
        }
    }

    #[test]
    fn hotkey_dispatch_unhandled_action_silent() {
        let mgr = generate_test_state_manager();
        let before = mgr.load().clone();
        dispatch_hotkey(HotkeyAction::CycleMode, &mgr);
        let after = mgr.load();
        assert_eq!(
            *after, before,
            "state should be unchanged after unhandled action"
        );
    }

    // ---------------------------------------------------------------
    // T005 -- Match + dispatch round-trip integration tests
    // ---------------------------------------------------------------

    #[test]
    fn hotkey_match_and_dispatch_zoom_in_numpad() {
        let matcher = HotkeyMatcher::default();
        let mgr = generate_test_state_manager_with_zoom(2.0);
        let event = generate_test_key_event(KeyCode::NumpadAdd, ctrl_alt(), true);

        let action = matcher.match_event(&event).unwrap();
        assert_eq!(action, HotkeyAction::ZoomIn);
        dispatch_hotkey(action, &mgr);

        let zoom = mgr.load().settings.magnification.zoom_level;
        assert!(
            (zoom - 3.0).abs() < f32::EPSILON,
            "zoom should be 3.0, got {zoom}"
        );
    }

    #[test]
    fn hotkey_match_and_dispatch_zoom_out_numpad() {
        let matcher = HotkeyMatcher::default();
        let mgr = generate_test_state_manager_with_zoom(3.0);
        let event = generate_test_key_event(KeyCode::NumpadSubtract, ctrl_alt(), true);

        let action = matcher.match_event(&event).unwrap();
        assert_eq!(action, HotkeyAction::ZoomOut);
        dispatch_hotkey(action, &mgr);

        let zoom = mgr.load().settings.magnification.zoom_level;
        assert!(
            (zoom - 2.0).abs() < f32::EPSILON,
            "zoom should be 2.0, got {zoom}"
        );
    }

    #[test]
    fn hotkey_match_and_dispatch_reset_numpad() {
        let matcher = HotkeyMatcher::default();
        let mgr = generate_test_state_manager_with_zoom(5.0);
        let event = generate_test_key_event(KeyCode::Numpad0, ctrl_alt(), true);

        let action = matcher.match_event(&event).unwrap();
        assert_eq!(action, HotkeyAction::ZoomReset);
        dispatch_hotkey(action, &mgr);

        let zoom = mgr.load().settings.magnification.zoom_level;
        assert!(
            (zoom - 2.0).abs() < f32::EPSILON,
            "zoom should be reset to 2.0, got {zoom}"
        );
    }

    // ---------------------------------------------------------------
    // T006 -- Full round-trip integration tests
    // ---------------------------------------------------------------

    #[test]
    fn hotkey_integration_full_workflow() {
        let mgr = generate_test_state_manager();

        // Default: is_active=false, zoom=2.0
        assert!(!mgr.load().is_active);
        assert!((mgr.load().settings.magnification.zoom_level - 2.0).abs() < f32::EPSILON);

        // Toggle on
        dispatch_hotkey(HotkeyAction::ToggleMagnification, &mgr);
        assert!(mgr.load().is_active);

        // Zoom in: 2.0 -> 3.0
        dispatch_hotkey(HotkeyAction::ZoomIn, &mgr);
        assert!((mgr.load().settings.magnification.zoom_level - 3.0).abs() < f32::EPSILON);

        // Zoom in: 3.0 -> 4.5
        dispatch_hotkey(HotkeyAction::ZoomIn, &mgr);
        assert!((mgr.load().settings.magnification.zoom_level - 4.5).abs() < f32::EPSILON);

        // Zoom out: 4.5 -> 3.0
        dispatch_hotkey(HotkeyAction::ZoomOut, &mgr);
        assert!((mgr.load().settings.magnification.zoom_level - 3.0).abs() < f32::EPSILON);

        // Reset: -> 2.0
        dispatch_hotkey(HotkeyAction::ZoomReset, &mgr);
        assert!((mgr.load().settings.magnification.zoom_level - 2.0).abs() < f32::EPSILON);

        // Toggle off
        dispatch_hotkey(HotkeyAction::ToggleMagnification, &mgr);
        assert!(!mgr.load().is_active);

        // Final state: is_active=false, zoom=2.0
        let final_state = mgr.load();
        assert!(!final_state.is_active);
        assert!((final_state.settings.magnification.zoom_level - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hotkey_integration_state_visibility_across_operations() {
        let mgr = generate_test_state_manager_with_zoom(2.0);

        // Each dispatch should be immediately visible via load()
        dispatch_hotkey(HotkeyAction::ZoomIn, &mgr);
        let after_zoom_in = mgr.load().settings.magnification.zoom_level;
        assert!(
            (after_zoom_in - 3.0).abs() < f32::EPSILON,
            "zoom should be visible immediately after ZoomIn"
        );

        dispatch_hotkey(HotkeyAction::ToggleMagnification, &mgr);
        let after_toggle = mgr.load().is_active;
        assert!(
            after_toggle,
            "is_active should be visible immediately after Toggle"
        );

        dispatch_hotkey(HotkeyAction::ZoomReset, &mgr);
        let after_reset = mgr.load().settings.magnification.zoom_level;
        assert!(
            (after_reset - 2.0).abs() < f32::EPSILON,
            "zoom should be visible immediately after Reset"
        );

        dispatch_hotkey(HotkeyAction::ZoomOut, &mgr);
        let after_zoom_out = mgr.load().settings.magnification.zoom_level;
        assert!(
            (after_zoom_out - 1.5).abs() < 0.001,
            "zoom should be visible immediately after ZoomOut: {after_zoom_out}"
        );
    }
}
