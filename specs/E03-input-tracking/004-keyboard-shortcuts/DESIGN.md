# Design: Story E03/004 -- Global Keyboard Shortcuts

**Story:** [STORY.md](STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** spec-writer-2
**Risk Refs:** None specific. RISK-012 (Wayland input) is mitigated by X11-only scope. Configurable keybinding complexity (RISK noted in doc-09) is explicitly deferred to E07.

---

## Overview

Implement the global keyboard shortcut detection and dispatch system. A `HotkeyMatcher` struct holds a mapping from `(KeyCode, Modifiers)` pairs to `HotkeyAction` variants. When an `InputEvent::KeyEvent` arrives (from Story 001's input monitor via Story 005's event dispatch), the matcher checks for a registered shortcut on key press (not release), with exact modifier matching. On match, a `dispatch_hotkey()` function executes the corresponding state mutation via Story 002's `StateManager`.

Four hardcoded shortcuts are registered following accessibility tool conventions (Ctrl+Alt prefix):
- **Ctrl+Alt+=** / **Ctrl+Alt+NumpadAdd** -> Zoom In (multiply by 1.5)
- **Ctrl+Alt+-** / **Ctrl+Alt+NumpadSubtract** -> Zoom Out (divide by 1.5)
- **Ctrl+Alt+8** -> Toggle Magnification
- **Ctrl+Alt+0** / **Ctrl+Alt+Numpad0** -> Reset Zoom (to 2.0 default)

The `HotkeyMatcher` and `dispatch_hotkey()` are pure-logic components in `luminos-core`. They have no I/O, no async, and no platform dependency. The wiring of key events from the input channel to the matcher is handled by Story 005.

## Architecture

### Component Diagram

```
luminos-core/src/
  lib.rs                    [Modified] Add `pub mod hotkeys;`
  hotkeys.rs                [New]      HotkeyMatcher, dispatch_hotkey()
  state_manager.rs          [From Story 002] StateManager (used by dispatch_hotkey)
  config/
    schema.rs               [Existing] HotkeyAction enum (reused, not modified)
  state.rs                  [Existing] AppState (read/written via StateManager)

luminos-platform/src/
  traits/
    input_monitor.rs        [Existing, unchanged] InputEvent, KeyCode, Modifiers
```

```
Key event data flow (wired by Story 005):

  InputEvent::KeyEvent { code, pressed, modifiers }
       |
       v
  +-------------------+
  | HotkeyMatcher     |
  | .match_event()    |
  |                   |
  | Checks:           |
  | - pressed == true |
  | - exact modifiers |
  | - registered key  |
  +--------+----------+
           |
           v  Option<HotkeyAction>
  +-------------------+
  | dispatch_hotkey() |
  |                   |
  | ZoomIn:   zoom *= 1.5, clamp 20.0  |
  | ZoomOut:  zoom /= 1.5, clamp 1.5   |
  | Toggle:   is_active = !is_active    |
  | Reset:    zoom = 2.0               |
  +--------+----------+
           |
           v  StateManager.update_*() / toggle_*()
  +-------------------+
  | ArcSwap<AppState> |
  | (visible to       |
  |  render thread)   |
  +-------------------+
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `luminos-core::hotkeys` | New | `HotkeyMatcher` struct, `dispatch_hotkey()` function |
| `luminos-core::lib.rs` | Modified | Add `pub mod hotkeys;` and re-export `HotkeyMatcher` |
| `luminos-core::config::schema::HotkeyAction` | Unchanged | Reused enum (ZoomIn, ZoomOut, ZoomReset, ToggleMagnification) |
| `luminos-core::state_manager::StateManager` | Unchanged | Used by `dispatch_hotkey()` for state mutations |
| `luminos-platform::traits::input_monitor` | Unchanged | `InputEvent`, `KeyCode`, `Modifiers` consumed by `match_event()` |

### Data Flow

1. **Initialization:** `HotkeyMatcher::default()` constructs the matcher with the hardcoded shortcut table. Seven entries total (four actions, three with numpad alternatives).

2. **Per-event matching (called by Story 005's input processing task):**
   a. The input processing task receives an `InputEvent` from the mpsc channel.
   b. For `InputEvent::KeyEvent` variants, it calls `hotkey_matcher.match_event(&event)`.
   c. The matcher checks: `pressed == true` AND `modifiers` match exactly (no extra modifiers) AND `code` is in the registered map.
   d. Returns `Some(HotkeyAction)` on match, `None` otherwise.

3. **Dispatch (called by Story 005's input processing task):**
   a. On `Some(action)`, calls `dispatch_hotkey(action, &state_manager)`.
   b. The dispatch function executes the state mutation via `StateManager`:
      - `ZoomIn`: reads current zoom, multiplies by 1.5, calls `update_zoom_level()` (which clamps to [1.5, 20.0])
      - `ZoomOut`: reads current zoom, divides by 1.5, calls `update_zoom_level()`
      - `ToggleMagnification`: calls `toggle_magnification()`
      - `ZoomReset`: calls `reset_zoom()`
   c. After the state mutation, the caller (Story 005) sends `LuminosEvent::StateChanged` via `EventLoopProxy`.

## API Design

### HotkeyMatcher

```rust
use std::collections::HashMap;
use luminos_platform::traits::input_monitor::{InputEvent, KeyCode, Modifiers};
use crate::config::schema::HotkeyAction;

/// Matches keyboard events against registered hotkey bindings.
///
/// Holds a mapping from `(KeyCode, Modifiers)` pairs to `HotkeyAction`.
/// Matches only on key press (not release) with exact modifier matching
/// (no extra modifiers allowed).
///
/// # Phase 0 Shortcuts
///
/// The default matcher registers four ZoomText-convention shortcuts:
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
    /// Creates a matcher with the hardcoded Phase 0 ZoomText shortcuts.
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
        bindings.insert(
            (KeyCode::NumpadSubtract, ctrl_alt),
            HotkeyAction::ZoomOut,
        );

        // Toggle Magnification: Ctrl+Alt+8
        bindings.insert(
            (KeyCode::Key8, ctrl_alt),
            HotkeyAction::ToggleMagnification,
        );

        // Reset Zoom: Ctrl+Alt+0 and Ctrl+Alt+Numpad0
        bindings.insert((KeyCode::Key0, ctrl_alt), HotkeyAction::ZoomReset);
        bindings.insert((KeyCode::Numpad0, ctrl_alt), HotkeyAction::ZoomReset);

        Self::new(bindings)
    }
}
```

### dispatch_hotkey

```rust
use crate::config::schema::HotkeyAction;
use crate::state_manager::StateManager;

/// Zoom step multiplier for zoom in/out.
///
/// ZoomText uses multiplicative zoom steps (not additive) for a natural
/// zoom progression: 2.0 -> 3.0 -> 4.5 -> 6.75 -> 10.125 -> 15.1875 -> 20.0.
const ZOOM_STEP: f32 = 1.5;

/// Default zoom level for reset.
const DEFAULT_ZOOM: f32 = 2.0;

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
            // update_zoom_level() clamps to [1.5, 20.0] internally
        }
        HotkeyAction::ZoomOut => {
            let current = state_manager.load();
            let new_level = current.settings.magnification.zoom_level / ZOOM_STEP;
            state_manager.update_zoom_level(new_level);
            // update_zoom_level() clamps to [1.5, 20.0] internally
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
```

### Required Trait Implementations

For `HotkeyMatcher` to use `Modifiers` as a `HashMap` key, `Modifiers` needs `Hash` and `Eq`. Checking the existing definition:

```rust
// In luminos-platform::traits::input_monitor (existing):
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers { ... }
```

`Modifiers` already has `Eq` but NOT `Hash`. The `HashMap` key `(KeyCode, Modifiers)` requires both `Hash` and `Eq`. Two options:

1. **Add `Hash` derive to `Modifiers` and `KeyCode`** in `luminos-platform::traits::input_monitor`. This is a backward-compatible change (adding a trait impl).
2. **Use a wrapper key type** that hashes the modifier fields manually.

**Decision: Option 1** -- add `#[derive(Hash)]` to `Modifiers`. `KeyCode` already needs `Hash` (it has `#[derive(Hash)]` in the existing code). `Modifiers` is a simple struct of 4 bools, so `Hash` derivation is trivial and correct.

This is a one-line change to `luminos-platform/src/traits/input_monitor.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Modifiers { ... }
```

Additionally, `HotkeyAction` needs `Copy` for `.copied()` in `match_event()`. Checking existing definition:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HotkeyAction { ... }
```
`HotkeyAction` already has `Copy`. No change needed.

## Error Handling

This story introduces no new error types. All functions are infallible:
- `HotkeyMatcher::new()` is a HashMap wrapper construction.
- `HotkeyMatcher::match_event()` is a HashMap lookup returning `Option`.
- `dispatch_hotkey()` delegates to `StateManager` methods which are all infallible (`rcu()` retries internally on contention, never fails).

The wildcard `_ => {}` match arm in `dispatch_hotkey()` silently ignores `HotkeyAction` variants not handled in Phase 0. This is intentional -- future phases add handlers for `CycleMode`, `ReadWhatISee`, etc.

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| All platforms | Pure Rust logic | No platform-specific code. `HotkeyMatcher` operates on `KeyCode`/`Modifiers` types which are platform-independent abstractions. |

The hotkey system is platform-independent. Platform-specific keyboard handling (X11 keycode-to-`KeyCode` mapping) is handled by Story 001's `X11InputMonitor`. The hotkey matcher receives platform-independent `InputEvent::KeyEvent` values.

## Testing Strategy

### Unit Tests

- **Default bindings registered:** Verify `HotkeyMatcher::default()` contains all 7 expected entries.
- **Zoom in match:** `Ctrl+Alt+Equal` press -> `Some(ZoomIn)`.
- **Zoom in numpad match:** `Ctrl+Alt+NumpadAdd` press -> `Some(ZoomIn)`.
- **Zoom out match:** `Ctrl+Alt+Minus` press -> `Some(ZoomOut)`.
- **Zoom out numpad match:** `Ctrl+Alt+NumpadSubtract` press -> `Some(ZoomOut)`.
- **Toggle match:** `Ctrl+Alt+8` press -> `Some(ToggleMagnification)`.
- **Reset match:** `Ctrl+Alt+Key0` press -> `Some(ZoomReset)`.
- **Reset numpad match:** `Ctrl+Alt+Numpad0` press -> `Some(ZoomReset)`.
- **Wrong modifiers:** `Ctrl+Shift+Equal` -> `None`.
- **Extra modifier:** `Ctrl+Alt+Shift+Equal` -> `None` (exact match required).
- **Key release ignored:** `Ctrl+Alt+Equal` with `pressed=false` -> `None`.
- **Non-KeyEvent ignored:** `MouseMoved` event -> `None`.
- **Unregistered key:** `Ctrl+Alt+A` -> `None`.
- **dispatch_hotkey ZoomIn:** Start at 2.0, dispatch ZoomIn, verify zoom is 3.0.
- **dispatch_hotkey ZoomOut:** Start at 3.0, dispatch ZoomOut, verify zoom is 2.0.
- **dispatch_hotkey ZoomIn at max:** Start at 20.0, dispatch ZoomIn, verify zoom stays at 20.0.
- **dispatch_hotkey ZoomOut at min:** Start at 1.5, dispatch ZoomOut, verify zoom stays at 1.5.
- **dispatch_hotkey Toggle on:** Start active, dispatch Toggle, verify inactive.
- **dispatch_hotkey Toggle off:** Start inactive, dispatch Toggle, verify active.
- **dispatch_hotkey Reset:** Start at 5.0, dispatch Reset, verify zoom is 2.0.
- **dispatch_hotkey repeated zoom in:** 2.0 -> 3.0 -> 4.5 -> 6.75 -> 10.125 -> 15.1875 -> 20.0 (clamped).

### Integration Tests

- **Full match+dispatch round-trip:** Create `StateManager` + `HotkeyMatcher`, construct `InputEvent::KeyEvent`, match and dispatch, verify state changed.
- **Xvfb integration (ci_platform_tests):** Not in this story -- Story 005 handles the `xdotool` integration tests.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Unit | StateManager at 2.0, dispatch ZoomIn, load() shows 3.0 |
| AC-1.2 | Unit | match_event with NumpadAdd + Ctrl+Alt -> Some(ZoomIn), dispatch, verify 3.0 |
| AC-1.3 | Unit | StateManager at 20.0, dispatch ZoomIn, load() shows 20.0 (clamped) |
| AC-2.1 | Unit | StateManager at 3.0, dispatch ZoomOut, load() shows 2.0 |
| AC-2.2 | Unit | match_event with NumpadSubtract + Ctrl+Alt -> Some(ZoomOut), dispatch, verify 2.0 |
| AC-2.3 | Unit | StateManager at 1.5, dispatch ZoomOut, load() shows 1.5 (clamped) |
| AC-3.1 | Unit | StateManager with is_active=true, dispatch Toggle, load() shows is_active=false |
| AC-3.2 | Unit | StateManager with is_active=false, dispatch Toggle, load() shows is_active=true |
| AC-3.3 | Unit | Set zoom=5.0, toggle off, toggle on, verify zoom still 5.0 |
| AC-4.1 | Unit | StateManager at 5.0, dispatch Reset, load() shows 2.0 |
| AC-4.2 | Unit | match_event with Numpad0 + Ctrl+Alt -> Some(ZoomReset), dispatch, verify 2.0 |
| AC-5.1 | Unit | Construct KeyEvent with Ctrl+Alt+Equal pressed=true, match_event returns Some(ZoomIn) |
| AC-5.2 | Unit | Construct KeyEvent with Ctrl+Shift+Equal, match_event returns None |
| AC-5.3 | Unit | Construct KeyEvent with Ctrl+Alt+Equal pressed=false, match_event returns None |
| AC-5.4 | Unit | Construct MouseMoved event, match_event returns None |
| AC-5.5 | Unit | Construct KeyEvent with Ctrl+Alt+Shift+Equal (shift=true), match_event returns None |

## Performance Targets

| Metric | Target | Source |
|--------|--------|--------|
| `match_event()` latency | < 1us | NFR-1, HashMap O(1) lookup |
| `dispatch_hotkey()` latency | < 10us | StateManager rcu() + ArcSwap write |
| Allocations per match | 0 | HashMap lookup, no allocation |
| Allocations per dispatch | 1 AppState clone | rcu() clones current state to create new version |

## Security Considerations

- **No key logging:** The hotkey matcher receives `KeyCode` enum variants, not raw characters or scan codes. It only checks for registered shortcuts -- unregistered key presses are discarded with `None`. No key data is logged, stored, or transmitted.
- **No privilege escalation:** Hotkeys trigger only application-internal state changes (zoom level, magnification toggle). No system commands, file operations, or network access.

## Alternatives Considered

### Alternative 1: Use configurable KeyBinding from AppSettings (deferred to E07)

The `AppSettings` struct already has a `keybindings: HashMap<HotkeyAction, Option<KeyBinding>>` field. This story could load bindings from settings rather than hardcoding them. Deferred because: (1) Phase 0 goal is minimum viable magnifier with ZoomText defaults; (2) configurable bindings require a UI for binding configuration (E07 scope); (3) the `KeyBinding` struct uses string-based key names which need conversion to `KeyCode` -- a separate concern from hotkey matching. The `HotkeyMatcher::new()` API accepts a `HashMap`, making it straightforward to switch from hardcoded to configurable bindings in E07.

### Alternative 2: Pattern matching instead of HashMap (rejected)

The four shortcuts could be matched with a `match` statement on `(code, modifiers)` instead of a `HashMap`. Rejected because: (1) the `HashMap` approach is more extensible -- E07 adds configurable bindings by populating the map from settings; (2) the performance difference is negligible for 7 entries; (3) the `HashMap` approach makes the registered bindings introspectable (can iterate for UI display).

### Alternative 3: Ctrl+Alt+F1 for toggle (rejected)

Ctrl+Alt+F1 was considered as the toggle shortcut (ZoomText convention). Rejected because Ctrl+Alt+F1 triggers VT1 (text console) switching on most Linux desktop environments — this is a kernel-level intercept that would prevent X11 from delivering the key to Luminos. Ctrl+Alt+8 was chosen instead to align with GNOME's magnifier toggle (Super+Alt+8) while avoiding VT switching conflicts.

### Alternative 4: Ctrl+Alt+M for toggle (rejected)

Ctrl+Alt+M was considered ("M" for Magnification). Rejected in favor of Ctrl+Alt+8 which aligns with GNOME's magnifier convention and is less likely to conflict with application shortcuts.
