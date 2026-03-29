# Story E03/001: X11 Global Input Monitoring (x11rb)

**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** None

---

## Problem Statement

After Epic E02, Luminos renders a magnified view of the screen at 60fps -- but the view is static. The magnification viewport is hardcoded and does not respond to user input. For the magnifier to become usable, it must continuously track the user's mouse position and keyboard activity globally, even when Luminos does not have window focus.

This story implements the `InputMonitor` trait for Linux X11, providing the raw input event stream that downstream stories (viewport tracking, keyboard shortcuts) consume. Without global input monitoring, the magnifier cannot follow the cursor or respond to hotkeys -- it remains a static viewer rather than an interactive accessibility tool.

The implementation uses `x11rb` directly with XInput2 extension instead of the `rdev` crate recommended by the tech strategy. This is a deliberate deviation: `rdev` is a single-maintainer crate at v0.5.x (RISK-031) with known Wayland issues (RISK-012), and it wraps the same X11 APIs internally. Using `x11rb` directly gives full control, removes a dependency, and reuses an existing workspace dependency.

## User Scenarios

### US-1: Cursor Position Tracking

As a low-vision user, I want my magnified view to track my mouse cursor so that I can navigate the screen naturally without manually repositioning the magnification viewport.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given a valid X11 display connection, when `subscribe_input_events()` is called with `buffer_size=32`, then a `Receiver<InputEvent>` is returned and the input monitor begins emitting events.
- **AC-1.2:** Given the input monitor is active and the mouse is moved on the X11 display, when the mouse moves from position (100, 100) to (200, 300), then an `InputEvent::MouseMoved { position: ScreenPoint { x: 200, y: 300 } }` event is received on the channel within 16ms.
- **AC-1.3:** Given the input monitor is active and the channel is full (32 pending events), when additional `MouseMoved` events arrive, then the oldest mouse move events are dropped (lossy semantics via `try_send()`) and no backpressure is applied to the X11 event loop.
- **AC-1.4:** Given a valid X11 display connection, when `get_mouse_position()` is called, then the current absolute screen-coordinate mouse position is returned as a `ScreenPoint`.

### US-2: Keyboard Event Capture

As a low-vision user, I want Luminos to detect my keyboard activity globally so that keyboard shortcuts (zoom in/out, toggle magnification) work regardless of which application has focus.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given the input monitor is active, when a key is pressed on the keyboard, then an `InputEvent::KeyEvent { code, pressed: true, modifiers }` event is received with the correct `KeyCode` and `Modifiers` state.
- **AC-2.2:** Given the input monitor is active, when a key is released, then an `InputEvent::KeyEvent { code, pressed: false, modifiers }` event is received.
- **AC-2.3:** Given the input monitor is active and modifier keys (Ctrl, Alt) are held, when the '=' key is pressed, then the `KeyEvent` contains `modifiers: Modifiers { ctrl: true, alt: true, shift: false, meta: false }` and `code: KeyCode::Equal`.
- **AC-2.4:** Given the input monitor is active and the channel has pending events, when key events arrive, then key events are NOT dropped (use blocking send, not `try_send()`), preserving all keyboard input for hotkey detection.

### US-3: Error Handling and Resilience

As a developer, I want clear error reporting when X11 input monitoring is unavailable so that the application can degrade gracefully.

**Priority:** P1
**Acceptance Criteria:**

- **AC-3.1:** Given no X11 display is available (e.g., `DISPLAY` not set), when `X11InputMonitor::new()` is called, then `InputError::Unavailable` is returned with a descriptive reason.
- **AC-3.2:** Given the X server does not support XInput2 (version < 2.0), when `X11InputMonitor::new()` is called, then `InputError::Unavailable` is returned indicating the missing extension.
- **AC-3.3:** Given the input monitor is active and the X11 connection is lost, when the monitoring thread detects the disconnection, then `InputError::Disconnected` is sent (or the channel is closed), and the monitoring thread exits cleanly without panicking.

### US-4: Mouse Button and Scroll Events

As a low-vision user, I want mouse button and scroll events captured so that future features (click-to-read, scroll-to-zoom) can build on this input stream.

**Priority:** P2
**Acceptance Criteria:**

- **AC-4.1:** Given the input monitor is active, when a mouse button is pressed, then an `InputEvent::MouseButton { button, pressed: true, position }` event is received with the correct button identifier and cursor position.
- **AC-4.2:** Given the input monitor is active, when the scroll wheel is rotated vertically, then an `InputEvent::Scroll { delta_x: 0.0, delta_y, position }` event is received with non-zero `delta_y`.

## Functional Requirements

- **FR-1:** Implement the `InputMonitor` trait (`subscribe_input_events()`, `get_mouse_position()`) for Linux X11 using `x11rb` with XInput2 extension.
- **FR-2:** Spawn a dedicated input monitoring thread that connects to the X server, registers for XInput2 `Motion`, `ButtonPress`, `ButtonRelease`, `KeyPress`, and `KeyRelease` events on the root window (via `XIAllMasterDevices`), and translates them into `InputEvent` variants. Regular (non-Raw) events are used because they provide absolute screen coordinates directly, avoiding the need for delta accumulation (see DESIGN.md Alternative 3).
- **FR-3:** Implement X11 keycode-to-`KeyCode` mapping covering all variants defined in the `KeyCode` enum (alphanumeric, function keys, navigation, modifiers, numpad, punctuation).
- **FR-4:** Track modifier state (Shift, Ctrl, Alt, Meta/Super) by monitoring modifier key press/release events and including current modifier state in every `InputEvent::KeyEvent`.
- **FR-5:** Use `try_send()` for `MouseMoved` events (lossy semantics) and blocking channel send for all other event types (key events, button events, scroll events) per the `InputMonitor` trait contract.
- **FR-6:** Implement `get_mouse_position()` as a synchronous `QueryPointer` request to the X server, returning the current root-window-relative cursor position.
- **FR-7:** Validate X11 display availability and XInput2 extension presence during construction (`X11InputMonitor::new()`), returning appropriate `InputError` variants on failure.

## Non-Functional Requirements

- **NFR-1:** Input event latency from physical input to channel delivery must be less than 1 frame (< 16.67ms) for mouse move events. (SC1)
- **NFR-2:** The input monitoring thread must not block or interfere with the render thread. All communication is via the bounded mpsc channel.
- **NFR-3:** The X11 connection must be persistent (not per-event or per-call) to avoid the connection-per-call overhead discovered with xcap in E02 (see E02 Shared Context, Research Findings).
- **NFR-4:** The input monitoring thread must exit cleanly when the channel sender is dropped or a shutdown signal is received, without leaking the X11 connection.
- **NFR-5:** The `get_mouse_position()` synchronous query must complete in < 1ms on a local X11 display.

## Out of Scope

- Wayland input monitoring (deferred to E08, doc-09 Section 4.8)
- macOS, Windows, and OpenBSD input backends (future epics per platform development order)
- Focus tracking via AT-SPI2 (`FocusTracker` trait, deferred to E07)
- Configurable keybindings (E07) -- this story provides raw input events; hotkey matching is Story 004
- Scroll-to-zoom behavior (depends on Story 004 hotkey dispatch)
- Mouse button event dispatch to higher-level actions (future epics)
- Touch input monitoring (XInput2 touch events, future epic)
- `rdev` crate integration -- deliberately excluded in favor of x11rb (see Architecture Decisions in HIGH_LEVEL_PLAN.md)
- XRecord extension for keyboard capture -- using XInput2 `RawKeyPress`/`RawKeyRelease` instead (XInput2 provides both mouse and keyboard events from a single extension)

## Open Questions

- [x] Should we use XInput2 for both mouse and keyboard events, or XRecord for keyboard only? **Decision: XInput2 for both.** XInput2's `KeyPress`/`KeyRelease` events (regular, non-Raw) on `XIAllMasterDevices` provide global keyboard capture without needing a separate XRecord connection. This simplifies the implementation to a single X11 extension.
- [x] Which XInput2 device ID should we listen on -- `XIAllDevices` or `XIAllMasterDevices`? **Decision: XIAllMasterDevices (device ID 1).** Master devices aggregate all physical input devices. Using `XIAllDevices` would produce duplicate events from both master and slave devices.
- [x] How to handle X11 keycode-to-keysym conversion? **Decision: Use the core protocol `GetKeyboardMapping` request (no extra x11rb features needed) to obtain keysym values from keycodes, then map keysyms to `KeyCode` enum variants.** This is more portable than hardcoding keycode offsets (which vary by keyboard layout) and avoids requiring the `xkb` x11rb feature.
