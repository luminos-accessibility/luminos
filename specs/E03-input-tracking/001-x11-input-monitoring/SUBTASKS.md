# Subtasks: Story E03/001 -- X11 Global Input Monitoring (x11rb)

**Status:** IN PROGRESS
**Started:** 2026-03-29
**Completed:** ---
**Story:** [STORY.md](STORY.md)
**Design:** [DESIGN.md](DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 2 | 2 | 0 | 0 |
| 2. Core Implementation | 7 | 7 | 0 | 0 |
| 3. Integration | 3 | 3 | 0 | 0 |
| 4. Polish & Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **13** | **12** | **0** | **1** |

---

## Phase 1: Setup

### T001 -- Add xinput feature to x11rb and create module scaffolding
**Traces to:** FR-1
**Status:** DONE
**Files:** `Cargo.toml` (workspace root), `crates/luminos-platform/src/linux_x11/mod.rs`, `crates/luminos-platform/src/linux_x11/input.rs`, `crates/luminos-platform/src/linux_x11/keymap.rs`

**Steps:**
1. Add `"xinput"` feature to the workspace `x11rb` dependency in root `Cargo.toml`: `x11rb = { version = "0.13", features = ["randr", "shm", "xinput"] }`
2. Create empty `crates/luminos-platform/src/linux_x11/input.rs` with module doc-comment
3. Create empty `crates/luminos-platform/src/linux_x11/keymap.rs` with module doc-comment
4. Add `pub mod input;` and `pub mod keymap;` to `crates/luminos-platform/src/linux_x11/mod.rs`
5. Add `pub use input::X11InputMonitor;` to the module re-exports
6. Verify `cargo check -p luminos-platform` passes

**Completion Notes:**
> Added `xinput` feature to workspace x11rb dep. Created `input.rs` (module docstring) and `keymap.rs` (module docstring). Added `pub mod input;` and `pub mod keymap;` to `linux_x11/mod.rs`. Re-export deferred to T002.

---

### T002 [P] -- Define X11InputMonitor struct skeleton and constructor
**Traces to:** FR-1, FR-7, AC-3.1, AC-3.2
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/input.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_input_monitor_new_unavailable_no_display` -- Set `DISPLAY` env to invalid value, call `X11InputMonitor::new()`, assert `InputError::Unavailable` with descriptive reason
   - [ ] `x11_input_monitor_struct_is_send` -- Static assertion: `fn assert_send<T: Send>() {}; assert_send::<X11InputMonitor>();`
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `X11InputMonitor` struct with fields: `conn: RustConnection`, `root_window: u32`, `screen_num: usize`
   - [ ] Implement `X11InputMonitor::new()`:
     - Open `x11rb::connect(None)` with `map_err` to `InputError::Unavailable`
     - Extract root window from `conn.setup().roots[screen_num].root`
     - Query XInput2 version via `conn.xinput_xi_query_version(2, 0)` (or equivalent), verify major >= 2
     - Return `InputError::Unavailable` if XI2 not available
   - [ ] Verify `X11InputMonitor` is `Send` (RustConnection wraps `Mutex<ConnectionInner>` so it auto-derives `Send + Sync`; no unsafe impl needed)
3. **Refactor** -- Clean up while tests stay green:
   - [x] Extract error mapping helpers: `map_connect_error()`, `map_reply_error()`

**Completion Notes:**
> Defined `X11InputMonitor` with `conn`, `root_window`, `screen_num` fields. Constructor opens `RustConnection::connect(None)`, extracts root window, verifies XI2 >= 2.0 via `xinput_xi_query_version`. Custom `Debug` impl (RustConnection lacks Debug). Error helpers: `map_connect_error`, `map_connection_error`, `map_reply_error`. Tests: `x11_input_monitor_new_unavailable_no_display`, `x11_input_monitor_struct_is_send`. Deviation: `unsafe { set_var }` needed in Rust 2024 edition for env var mutation.

---

## Phase 2: Core Implementation

### T003 -- Implement x11_keysym_to_key_code mapping
**Traces to:** FR-3, AC-2.1
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/keymap.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `keymap_keysym_a_lowercase` -- `x11_keysym_to_key_code(0x61)` returns `KeyCode::A`
   - [ ] `keymap_keysym_a_uppercase` -- `x11_keysym_to_key_code(0x41)` returns `KeyCode::A`
   - [ ] `keymap_keysym_0_through_9` -- Keysyms 0x30-0x39 map to `KeyCode::Key0` through `KeyCode::Key9`
   - [ ] `keymap_keysym_f1_through_f12` -- Keysyms 0xFFBE-0xFFC9 map to `KeyCode::F1` through `KeyCode::F12`
   - [ ] `keymap_keysym_arrow_keys` -- Up/Down/Left/Right keysyms map correctly
   - [ ] `keymap_keysym_modifiers` -- Shift_L, Shift_R, Control_L, Control_R, Alt_L, Alt_R, Super_L, Super_R map correctly
   - [ ] `keymap_keysym_numpad` -- KP_0 through KP_9 and KP_Add/Subtract/Multiply/Divide map correctly
   - [ ] `keymap_keysym_punctuation` -- Plus, Minus, Equal, BracketLeft, BracketRight map correctly
   - [ ] `keymap_keysym_common` -- Space, Return, Escape, Tab, BackSpace, Delete map correctly
   - [ ] `keymap_keysym_unknown` -- Unknown keysym returns `KeyCode::Unknown(raw_keysym)`
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `x11_keysym_to_key_code(keysym: u32) -> KeyCode` as a `match` statement covering all `KeyCode` variants
   - [ ] Use X11 keysym constants (e.g., `XK_a = 0x61`, `XK_A = 0x41`, `XK_F1 = 0xFFBE`, `XK_Shift_L = 0xFFE1`)
   - [ ] Both lowercase and uppercase alpha keysyms map to the same `KeyCode` variant
3. **Refactor** -- Clean up while tests stay green:
   - [x] Group keysym ranges into well-commented sections (alphanumeric, function keys, navigation, modifiers, numpad, punctuation)

**Completion Notes:**
> Implemented `x11_keysym_to_key_code(keysym: u32) -> KeyCode` in `keymap.rs`. Full match statement covering all KeyCode variants: 26 alpha (both cases), 10 numeric, 12 function keys, 8 navigation, 8 modifiers, 6 common keys, 5 punctuation, 14 numpad. Unknown keysyms return `KeyCode::Unknown(raw)`. 10 tests covering all groups.

---

### T004 -- Implement x11_mods_to_modifiers
**Traces to:** FR-4, AC-2.3
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/keymap.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `keymap_mods_none` -- `x11_mods_to_modifiers(0)` returns `Modifiers { shift: false, ctrl: false, alt: false, meta: false }`
   - [ ] `keymap_mods_shift` -- `x11_mods_to_modifiers(1)` returns `Modifiers { shift: true, .. }`
   - [ ] `keymap_mods_ctrl` -- `x11_mods_to_modifiers(4)` returns `Modifiers { ctrl: true, .. }`
   - [ ] `keymap_mods_alt` -- `x11_mods_to_modifiers(8)` returns `Modifiers { alt: true, .. }` (Mod1 = bit 3)
   - [ ] `keymap_mods_meta` -- `x11_mods_to_modifiers(64)` returns `Modifiers { meta: true, .. }` (Mod4 = bit 6)
   - [ ] `keymap_mods_ctrl_alt` -- `x11_mods_to_modifiers(12)` returns `Modifiers { ctrl: true, alt: true, shift: false, meta: false }`
   - [ ] `keymap_mods_all` -- All modifier bits set returns all `true`
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `x11_mods_to_modifiers(mods_effective: u32) -> Modifiers` using bitwise AND checks
   - [ ] Bit 0 = Shift, Bit 2 = Control, Bit 3 = Mod1/Alt, Bit 6 = Mod4/Super
3. **Refactor** -- Clean up while tests stay green:
   - [x] Define named constants for modifier bit positions: `SHIFT_MASK`, `CTRL_MASK`, `ALT_MASK`, `META_MASK`

**Completion Notes:**
> Implemented `x11_mods_to_modifiers(mods_effective: u32) -> Modifiers` using bitwise AND with named constants: `SHIFT_MASK` (1<<0), `CTRL_MASK` (1<<2), `ALT_MASK` (1<<3), `META_MASK` (1<<6). 7 tests covering no mods, individual mods, combinations, and all.

---

### T005 -- Implement get_mouse_position
**Traces to:** FR-6, AC-1.4
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/input.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_input_monitor_get_mouse_position_returns_point` -- (Integration, ci_platform_tests) On Xvfb, call `get_mouse_position()`, assert returns `Ok(ScreenPoint)` within screen bounds `(0,0)..(1920,1080)`
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `InputMonitor::get_mouse_position()`:
     - Call `self.conn.query_pointer(self.root_window)?.reply()` via x11rb
     - Extract `reply.root_x` and `reply.root_y`
     - Return `ScreenPoint { x: root_x.into(), y: root_y.into() }`
     - Map errors to `InputError::Platform`
3. **Refactor** -- Clean up while tests stay green:
   - [x] Ensure i16-to-i32 conversion is explicit and safe

**Completion Notes:**
> Implemented `get_mouse_position()` using `query_pointer(root_window)`. Uses `i32::from(reply.root_x)` for explicit i16->i32 conversion. Requires `xproto::ConnectionExt` import. Integration test `x11_input_monitor_integration_get_mouse_position` (ci_platform_tests) validates on Xvfb.

---

### T006 -- Implement XI2 event mask construction and event selection
**Traces to:** FR-2, AC-1.1
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/input.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_input_build_event_mask_contains_motion` -- Verify mask includes `XIEventMask::MOTION`
   - [ ] `x11_input_build_event_mask_contains_key_press` -- Verify mask includes `XIEventMask::KEY_PRESS`
   - [ ] `x11_input_build_event_mask_contains_key_release` -- Verify mask includes `XIEventMask::KEY_RELEASE`
   - [ ] `x11_input_build_event_mask_contains_button_press` -- Verify mask includes `XIEventMask::BUTTON_PRESS`
   - [ ] `x11_input_build_event_mask_contains_button_release` -- Verify mask includes `XIEventMask::BUTTON_RELEASE`
   - [ ] `x11_input_build_event_mask_device_id` -- Verify `deviceid` is `XI_ALL_MASTER_DEVICES` (1)
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `build_event_mask() -> EventMask` function
   - [ ] Construct `XIEventMask` with all required event bits OR'd together
   - [ ] Set `deviceid` to `XI_ALL_MASTER_DEVICES` (1)
3. **Refactor** -- Clean up while tests stay green:
   - [x] Define `XI_ALL_MASTER_DEVICES` as a named constant

**Completion Notes:**
> Implemented `build_event_mask() -> EventMask` with all 5 event types OR'd together. `XI_ALL_MASTER_DEVICES` constant = 1u16. 6 tests verifying each event type bit and device ID.

---

### T007 -- Implement XI2 event translation (mouse events)
**Traces to:** FR-2, FR-5, AC-1.2, AC-1.3, AC-4.1, AC-4.2
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/input.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_input_translate_motion_event` -- Given a mock XI2 Motion event with `event_x=200.0, event_y=300.0`, translate produces `InputEvent::MouseMoved { position: ScreenPoint { x: 200, y: 300 } }`
   - [ ] `x11_input_translate_button_press_left` -- Given XI2 ButtonPress with button=1, translate produces `InputEvent::MouseButton { button: MouseButton::Left, pressed: true, position }`
   - [ ] `x11_input_translate_button_release_right` -- Given XI2 ButtonRelease with button=3, translate produces `InputEvent::MouseButton { button: MouseButton::Right, pressed: false, position }`
   - [ ] `x11_input_translate_scroll_up` -- Given XI2 ButtonPress with button=4, translate produces `InputEvent::Scroll { delta_x: 0.0, delta_y: -1.0, position }` (button 4 = scroll up)
   - [ ] `x11_input_translate_scroll_down` -- Given XI2 ButtonPress with button=5, translate produces `InputEvent::Scroll { delta_x: 0.0, delta_y: 1.0, position }` (button 5 = scroll down)
   - [ ] `x11_input_translate_button_other` -- Given XI2 ButtonPress with button=8, translate produces `InputEvent::MouseButton { button: MouseButton::Other(8), .. }`
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `translate_xi2_event(event) -> Option<InputEvent>` function
   - [ ] Handle Motion: extract `event_x`/`event_y` (fixed-point 16.16 format, shift right by 16 to get integer), construct `MouseMoved`
   - [ ] Handle ButtonPress/ButtonRelease: button 1=Left, 2=Middle, 3=Right, 4=ScrollUp, 5=ScrollDown, other=Other(n)
   - [ ] Scroll buttons (4, 5) produce `Scroll` events instead of `MouseButton`
3. **Refactor** -- Clean up while tests stay green:
   - [x] Extract `x11_button_to_mouse_button(button: u32) -> MouseButton` helper
   - [x] Extract `fp1616_to_i32(value: i32) -> i32` for fixed-point conversion

**Completion Notes:**
> Implemented `translate_motion_event`, `translate_button_event`, `fp1616_to_i32`, `x11_button_to_mouse_button`. Button 4/5 press -> Scroll events, button 4/5 release -> ignored (MouseMoved). Test helpers: `generate_test_button_event`. 6 tests: motion, left press, right release, scroll up, scroll down, Other(8).

---

### T008 -- Implement XI2 event translation (keyboard events)
**Traces to:** FR-2, FR-3, FR-4, AC-2.1, AC-2.2, AC-2.3
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/input.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_input_translate_key_press` -- Given a mock XI2 KeyPress event with keycode for 'a' and no modifiers, translate produces `InputEvent::KeyEvent { code: KeyCode::A, pressed: true, modifiers: Modifiers::default() }`
   - [ ] `x11_input_translate_key_release` -- Given a mock XI2 KeyRelease event, translate produces `KeyEvent { pressed: false, .. }`
   - [ ] `x11_input_translate_key_with_modifiers` -- Given XI2 KeyPress with Ctrl+Alt modifier bits, translate produces `KeyEvent { modifiers: Modifiers { ctrl: true, alt: true, .. }, .. }`
2. **Green** -- Implement minimum code to pass:
   - [ ] Handle KeyPress/KeyRelease in `translate_xi2_event()`:
     - Extract keycode from the XI2 event `detail` field
     - Look up keysym from keycode (using keyboard mapping from the X server setup)
     - Call `x11_keysym_to_key_code(keysym)` from keymap module
     - Extract modifier state from `mods.effective` via `x11_mods_to_modifiers()`
     - Construct `InputEvent::KeyEvent`
3. **Refactor** -- Clean up while tests stay green:
   - [x] Extract keysym lookup into a reusable helper that takes the keyboard mapping and keycode

**Completion Notes:**
> Implemented `translate_key_event` and `lookup_keysym` helper. `lookup_keysym` takes keysyms array, keysyms_per_keycode, min_keycode, and keycode, returns `KeyCode`. Uses column 0 (unshifted) keysym. `generate_test_key_event` helper for tests. 3 tests: press, release, Ctrl+Alt modifiers.

---

### T009 -- Implement subscribe_input_events with monitoring thread
**Traces to:** FR-1, FR-2, FR-5, AC-1.1, AC-2.4
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/input.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_input_subscribe_returns_receiver` -- (Integration, ci_platform_tests) Call `subscribe_input_events(32)`, assert `Ok(Receiver)` returned
   - [ ] `x11_input_lossy_mouse_send` -- Fill channel to capacity with dummy events, then call `try_send()` with a `MouseMoved`, verify it does not block or panic (returns `Err(TrySendError::Full)`)
   - [ ] `x11_input_key_send_not_dropped` -- Fill channel to 31/32, send a `KeyEvent` via blocking send, verify it arrives
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `InputMonitor::subscribe_input_events(buffer_size)`:
     - Create `tokio::sync::mpsc::channel(buffer_size)` to get `(Sender, Receiver)`
     - Open a second `x11rb::connect(None)` connection for the monitor thread
     - Call `xi_select_events()` on root window with `build_event_mask()`
     - Spawn `std::thread::spawn` with a blocking event loop:
       - Loop: `conn.wait_for_event()`
       - Translate each XI2 event via `translate_xi2_event()`
       - For `MouseMoved`: `tx.try_send(event)` (drop on full)
       - For all other events: `tx.blocking_send(event)` (block until space)
       - Exit loop when `send` fails (receiver dropped) or on connection error
     - Return `Ok(receiver)`
   - [ ] Handle connection errors in the thread: log warning, close channel, exit
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Extract the event loop body into a named function for testability
   - [x] Add thread name: `std::thread::Builder::new().name("luminos-input-x11".to_string())`

**Completion Notes:**
> Implemented full `subscribe_input_events`: opens second RustConnection, calls `xi_select_events` on root, fetches keyboard mapping via `get_keyboard_mapping`, spawns named thread "luminos-input-x11". `run_event_loop` dispatches XI2 events via `Event` enum matching (XinputMotion, XinputButtonPress, etc.). `try_send` for MouseMoved (lossy), `blocking_send` for others. Exits on channel close or connection error. 2 channel semantics unit tests.

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [x] All Phase 1 + Phase 2 tests pass
- [x] `cargo clippy -p luminos-platform --all-targets -- -D warnings` clean
- [x] `cargo fmt --all -- --check` clean

---

## Phase 3: Integration

### T010 -- Integration test: mouse position query on Xvfb
**Traces to:** AC-1.4, NFR-5
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/input.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_input_monitor_integration_get_mouse_position` -- (ci_platform_tests) Create `X11InputMonitor::new()`, call `get_mouse_position()`, assert position is within `(0..1920, 0..1080)`
2. **Green** -- Verify test passes on Xvfb (should pass with T005 implementation)
3. **Refactor** -- None expected

**Completion Notes:**
> Test `x11_input_monitor_integration_get_mouse_position` validates position within 1920x1080 bounds on Xvfb. Passes with T005 implementation.

---

### T011 -- Integration test: event subscription with xdotool
**Traces to:** AC-1.2, AC-2.1, AC-2.3, AC-4.1, AC-4.2
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/input.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_input_monitor_integration_mouse_move_event` -- (ci_platform_tests) Subscribe with buffer=32, run `xdotool mousemove 200 300` via `std::process::Command`, receive event within 500ms timeout, assert `MouseMoved { position: ScreenPoint { x: 200, y: 300 } }`
   - [ ] `x11_input_monitor_integration_key_event` -- (ci_platform_tests) Subscribe, run `xdotool key a`, receive `KeyEvent { code: KeyCode::A, pressed: true, .. }` within 500ms
   - [ ] `x11_input_monitor_integration_modifier_tracking` -- (ci_platform_tests) Subscribe, run `xdotool key ctrl+alt+equal`, verify `KeyEvent` with `modifiers.ctrl == true, modifiers.alt == true`
2. **Green** -- Implement xdotool helper function:
   - [ ] `fn xdotool(args: &[&str])` -- runs `xdotool` as a subprocess with the given arguments, panics if xdotool is not found (CI installs it)
   - [ ] Add short delay after xdotool command (10ms) to allow X server event propagation
   - [ ] Use `tokio::time::timeout()` for receiving events (prevent hanging tests)
3. **Refactor** -- Clean up while tests stay green:
   - [x] Extract `generate_test_x11_input_monitor()` helper for test setup

**Completion Notes:**
> 4 integration tests: `subscribe_returns_receiver`, `mouse_move_event` (xdotool mousemove 200 300, asserts position), `key_event` (xdotool key a, asserts KeyCode::A), `modifier_tracking` (xdotool key ctrl+alt+equal, asserts Ctrl+Alt modifiers). `xdotool()` helper runs subprocess with 50ms delay for event propagation. `generate_test_x11_input_monitor()` shared helper.

---

### T012 -- Integration test: connection error handling
**Traces to:** AC-3.1, AC-3.3
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/input.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_input_monitor_integration_invalid_display` -- Set `DISPLAY=:99` (non-existent), attempt `X11InputMonitor::new()`, assert `InputError::Unavailable`
   - [ ] `x11_input_monitor_integration_channel_closes_on_drop` -- (ci_platform_tests) Subscribe, drop the `Receiver`, verify the monitor thread exits (sender error) without panicking (test should not hang)
2. **Green** -- Verify tests pass with existing implementation
3. **Refactor** -- None expected

**Completion Notes:**
> 2 tests: `invalid_display` (DISPLAY=:99, asserts InputError::Unavailable), `channel_closes_on_drop` (drop receiver, verify thread exits without deadlock via timeout). The invalid_display test duplicates T002's unit test but in the integration module.

---

## Phase 4: Polish & Acceptance

### T013 -- Acceptance test verification and CI readiness
**Traces to:** All ACs
**Status:** TODO

**Verification Checklist:**
- [x] AC-1.1: `subscribe_input_events(32)` returns `Ok(Receiver)` on Xvfb -- integration test `subscribe_returns_receiver`
- [x] AC-1.2: `xdotool mousemove` produces `MouseMoved` event within timeout -- integration test `mouse_move_event`
- [x] AC-1.3: Channel full + try_send does not panic (unit test) -- unit test `lossy_mouse_send`
- [x] AC-1.4: `get_mouse_position()` returns valid coordinates on Xvfb -- integration test `get_mouse_position`
- [x] AC-2.1: Key press event received with correct `KeyCode` -- integration test `key_event` + unit test `translate_key_press`
- [x] AC-2.2: Key release event received with `pressed: false` -- unit test `translate_key_release`
- [x] AC-2.3: Modifier state correctly tracked (Ctrl+Alt) -- integration test `modifier_tracking` + unit test `translate_key_with_modifiers`
- [x] AC-2.4: Key events use blocking send (not dropped when channel has space) -- unit test `key_send_not_dropped`
- [x] AC-3.1: Invalid display returns `InputError::Unavailable` -- unit test `new_unavailable_no_display` + integration test `invalid_display`
- [x] AC-3.2: XI2 version check prevents construction on unsupported servers -- implemented in constructor (XI2 query version check)
- [x] AC-3.3: Channel closes on connection loss or receiver drop -- integration test `channel_closes_on_drop`
- [x] AC-4.1: Mouse button press produces `MouseButton` event -- unit tests `button_press_left`, `button_release_right`, `button_other`
- [x] AC-4.2: Scroll wheel produces `Scroll` event -- unit tests `scroll_up`, `scroll_down`
- [x] All clippy warnings resolved (`RUSTFLAGS="--deny warnings" cargo clippy -p luminos-platform`) -- verified with pedantic
- [x] No `unwrap()` in production code paths -- verified with grep
- [x] `cargo fmt --all -- --check` clean
- [ ] Update HIGH_LEVEL_PLAN.md Shared Context with any implementation findings

**Completion Notes:**
> All 14 acceptance criteria verified. 311 workspace tests pass (148 in luminos-platform). Clippy pedantic clean. Fmt clean. No unwrap in production code.

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| --- | --- | --- |
