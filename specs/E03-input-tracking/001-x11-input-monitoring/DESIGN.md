# Design: Story E03/001 -- X11 Global Input Monitoring (x11rb)

**Story:** [STORY.md](STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** spec-writer-1
**Risk Refs:** RISK-031 (single-maintainer crate deps -- mitigated by choosing x11rb over rdev), RISK-012 (Wayland input -- X11-only, Wayland deferred to E08)

---

## Overview

Implement the `InputMonitor` trait for Linux X11 using `x11rb` directly with the XInput2 (XI2) extension. The implementation creates a persistent X11 connection, registers for XI2 input events (motion, button, key) on the root window via `xi_select_events()`, and translates X11 events into `InputEvent` variants sent through a bounded `tokio::sync::mpsc` channel.

XInput2 is used for all event types (mouse motion, mouse buttons, scroll, and keyboard), avoiding the need for a separate XRecord connection. XInput2's `KeyPress`/`KeyRelease` events (on `XIAllMasterDevices`) provide global keyboard capture from a single extension. The implementation listens on `XIAllMasterDevices` (device ID 1) to capture aggregated input from all physical devices without duplicate events.

## Architecture

### Component Diagram

```
luminos-platform/src/
  traits/
    input_monitor.rs        [Existing] InputMonitor trait, InputEvent, KeyCode, etc.
  linux_x11/
    mod.rs                  [Modified] Add `pub mod input;` and re-export X11InputMonitor
    input.rs                [New]      X11InputMonitor struct, XInput2 event loop
    keymap.rs               [New]      X11 keycode/keysym -> KeyCode mapping
    capture.rs              [Existing] Unchanged
    window.rs               [Existing] Unchanged
```

```
                              +-------------------+
                              |  X11InputMonitor  |
                              |  (implements      |
                              |   InputMonitor)   |
                              +--------+----------+
                                       |
                        +--------------+---------------+
                        |                              |
                +-------v--------+            +--------v--------+
                | Main X11 Conn  |            | Monitor Thread  |
                | (for queries:  |            | (dedicated X11  |
                |  QueryPointer) |            |  connection for |
                +----------------+            |  event loop)    |
                                              +--------+--------+
                                                       |
                                              +--------v--------+
                                              | xi_select_events |
                                              | on root window   |
                                              | XIAllMasterDevices|
                                              +--------+--------+
                                                       |
                                              +--------v--------+
                                              | wait_for_event() |
                                              | loop             |
                                              +--------+--------+
                                                       |
                              +------------------------+
                              |  Translate XI2 events  |
                              |  to InputEvent         |
                              +----------+-------------+
                                         |
                              +----------v-------------+
                              | tokio::sync::mpsc::    |
                              | Sender<InputEvent>     |
                              | (try_send for mouse,   |
                              |  blocking for keys)    |
                              +------------------------+
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `InputMonitor` (trait) | Unchanged | Existing trait from E01, implemented here |
| `luminos-platform::linux_x11::input` | New | `X11InputMonitor` struct and XI2 event loop |
| `luminos-platform::linux_x11::keymap` | New | X11 keycode-to-`KeyCode` mapping |
| `luminos-platform::linux_x11::mod` | Modified | Add `pub mod input; pub mod keymap;` and re-exports |
| Workspace `Cargo.toml` | Modified | Add `xinput` feature to x11rb (transitively enables `xfixes`) |

### Data Flow

1. **Construction:** `X11InputMonitor::new()` opens an x11rb connection to the X server, queries XInput2 extension version (require >= 2.0), and stores the connection for `get_mouse_position()` queries.

2. **Subscription:** `subscribe_input_events(buffer_size)` opens a **second** x11rb connection dedicated to the monitor thread. The monitor thread blocks indefinitely in `wait_for_event()`, holding the connection's internal `Mutex`. A second connection allows the main thread to execute synchronous `QueryPointer` requests without contending with the blocking event loop. It calls `xi_select_events()` on the root window for `XIAllMasterDevices` with the mask: `MOTION | BUTTON_PRESS | BUTTON_RELEASE | KEY_PRESS | KEY_RELEASE`. It spawns a `std::thread` that runs the blocking event loop.

3. **Event Loop (monitor thread):**
   - Calls `conn.wait_for_event()` in a loop (blocking).
   - Dispatches XInput2 events based on event type:
     - `Motion` -> `InputEvent::MouseMoved` with position from event's `root_x`/`root_y` (fixed-point 16.16 to i32)
     - `ButtonPress`/`ButtonRelease` -> `InputEvent::MouseButton` (buttons 1-3 map to Left/Right/Middle; 4-5 are scroll, mapped to `InputEvent::Scroll`)
     - `KeyPress`/`KeyRelease` -> `InputEvent::KeyEvent` with keycode mapped via `keymap::x11_keycode_to_key_code()` and modifier state tracked from `mods.effective`
   - Mouse move events: `tx.try_send()` (lossy, per trait contract)
   - All other events: `tx.blocking_send()` (blocks the OS thread until channel has capacity; does not require a tokio runtime)
   - Loop exits when `tx.send()` fails (receiver dropped) or when a stop flag is set.

4. **Position Query:** `get_mouse_position()` uses the main connection to call `QueryPointer` on the root window, returning `ScreenPoint { x: root_x, y: root_y }`.

## API Design

### X11InputMonitor

```rust
use crate::traits::input_monitor::{InputError, InputEvent, InputMonitor};
use crate::traits::types::ScreenPoint;
use tokio::sync::mpsc;
use x11rb::connection::Connection;
use x11rb::rust_connection::RustConnection;

/// X11 input monitor using XInput2 extension via x11rb.
///
/// Creates a persistent X11 connection for synchronous queries and spawns
/// a dedicated thread with a separate connection for the XI2 event loop.
pub struct X11InputMonitor {
    /// X11 connection for synchronous queries (get_mouse_position).
    conn: RustConnection,
    /// Root window ID for event registration and pointer queries.
    root_window: u32,
    /// Screen number (for multi-screen setups, currently single-display).
    screen_num: usize,
}

impl X11InputMonitor {
    /// Creates a new X11 input monitor.
    ///
    /// Opens an x11rb connection, validates XInput2 extension availability
    /// (requires version >= 2.0).
    ///
    /// # Errors
    ///
    /// Returns `InputError::Unavailable` if:
    /// - The X11 display cannot be opened (DISPLAY not set or X server down)
    /// - The XInput2 extension is not available or version < 2.0
    pub fn new() -> Result<Self, InputError> { ... }
}

impl InputMonitor for X11InputMonitor {
    fn subscribe_input_events(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<InputEvent>, InputError> { ... }

    fn get_mouse_position(&self) -> Result<ScreenPoint, InputError> { ... }
}
```

### Keymap Module

```rust
use crate::traits::input_monitor::{KeyCode, Modifiers};

/// Maps an X11 keysym to a Luminos `KeyCode`.
///
/// Keysyms are obtained from X11 keycodes via `GetKeyboardMapping` (core protocol).
/// This function maps the resulting keysym value to the corresponding `KeyCode` variant.
///
/// Returns `KeyCode::Unknown(keysym)` for unmapped keysyms.
pub fn x11_keysym_to_key_code(keysym: u32) -> KeyCode { ... }

/// Extracts `Modifiers` from XInput2 modifier state.
///
/// Maps X11 modifier bits to the `Modifiers` struct:
/// - Bit 0 (Shift): `modifiers.shift`
/// - Bit 2 (Control): `modifiers.ctrl`
/// - Bit 3 (Mod1/Alt): `modifiers.alt`
/// - Bit 6 (Mod4/Super): `modifiers.meta`
pub fn x11_mods_to_modifiers(mods_effective: u32) -> Modifiers { ... }
```

### XI2 Event Mask Construction

```rust
use x11rb::protocol::xinput::{self, EventMask, XIEventMask};

/// Device ID for all master devices (aggregates physical input).
const XI_ALL_MASTER_DEVICES: u16 = 1;

/// Constructs the XI2 event mask for global input monitoring.
fn build_event_mask() -> EventMask {
    let mask = XIEventMask::MOTION
        | XIEventMask::BUTTON_PRESS
        | XIEventMask::BUTTON_RELEASE
        | XIEventMask::KEY_PRESS
        | XIEventMask::KEY_RELEASE;

    EventMask {
        deviceid: XI_ALL_MASTER_DEVICES,
        mask: vec![mask],
    }
}
```

## Error Handling

All errors are expressed through the existing `InputError` enum (from `luminos-platform::traits::input_monitor`):

| Condition | Error Variant | Recovery |
|-----------|--------------|----------|
| `DISPLAY` not set / X server down | `InputError::Unavailable { reason }` | Caller logs and disables input monitoring |
| XInput2 not available or < v2.0 | `InputError::Unavailable { reason }` | Caller logs and disables input monitoring |
| X11 connection lost during event loop | `InputError::Disconnected { message }` | Channel closed; caller can attempt reconnection |
| `xi_select_events` failure | `InputError::Platform { message }` | Propagated to caller |
| `QueryPointer` failure | `InputError::Platform { message }` | Propagated to caller |

X11 connection errors from `x11rb` implement `std::error::Error` and are converted to `InputError` via pattern matching (not blanket `From` impl, to keep error messages descriptive):

```rust
fn map_connect_error(e: x11rb::errors::ConnectError) -> InputError {
    InputError::Unavailable {
        reason: format!("X11 connection failed: {e}"),
    }
}

fn map_reply_error(e: x11rb::errors::ReplyError) -> InputError {
    InputError::Platform {
        message: format!("X11 request failed: {e}"),
    }
}
```

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | x11rb XInput2 (this story) | Primary target. Uses `RustConnection` (pure Rust, no libxcb FFI). |
| Linux Wayland | Deferred to E08 | Requires `libinput` or compositor-specific protocols. RISK-012. |
| macOS | Deferred to future epic | CGEvent tap (requires Accessibility permission). |
| OpenBSD | Deferred to future epic | Shares X11 backend (XInput2 via x11rb). |
| Windows | Deferred to future epic | Raw Input or low-level hooks. |

**x11rb `RustConnection` vs `XCBConnection`:** This design uses `RustConnection` (pure Rust implementation) rather than `XCBConnection` (FFI to libxcb). `RustConnection` avoids a native library dependency and is the default for x11rb. Performance difference is negligible for input monitoring (event rate is human-speed, not GPU-speed).

**Two X11 connections:** Although x11rb's `RustConnection` is `Send + Sync` (it wraps a `Mutex<ConnectionInner>`), the monitor thread blocks indefinitely in `wait_for_event()`, holding the connection's internal lock. If the main thread attempted a `QueryPointer` on the same connection, it would block until the next X11 event wakes the monitor thread. A second connection avoids this contention. This is standard practice in X11 programming -- one connection per thread that does blocking I/O. The query connection (main thread) and the event loop connection (monitor thread) are independent.

## Testing Strategy

### Unit Tests

- **Keymap module:** Exhaustive mapping of X11 keysyms to `KeyCode` variants. Test all alphanumeric, function key, navigation, modifier, numpad, and punctuation entries. Test unknown keysym returns `KeyCode::Unknown`.
- **Modifier extraction:** Test all modifier bit combinations map correctly to `Modifiers` struct fields.
- **Event mask construction:** Verify the event mask contains all required XI2 event types.

### Integration Tests

Gated behind `#[cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]`, running on Xvfb in CI:

- **Mouse position query:** Call `get_mouse_position()` on Xvfb, verify result is within screen bounds.
- **Event subscription:** Call `subscribe_input_events()`, simulate mouse move with `xdotool`, verify `MouseMoved` event received.
- **Key event capture:** Simulate key press with `xdotool key a`, verify `KeyEvent` with `KeyCode::A` received.
- **Modifier tracking:** Simulate `xdotool key ctrl+alt+equal`, verify `KeyEvent` with correct modifiers.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Integration | `subscribe_input_events(32)` returns `Ok(Receiver)` on Xvfb |
| AC-1.2 | Integration | `xdotool mousemove 200 300` produces `MouseMoved { position: ScreenPoint { x: 200, y: 300 } }` within 16ms |
| AC-1.3 | Unit | Fill channel to capacity, send additional `MouseMoved`, verify no panic and channel still operational |
| AC-1.4 | Integration | `get_mouse_position()` on Xvfb returns position within `(0,0)..(1920,1080)` |
| AC-2.1 | Integration | `xdotool key a` produces `KeyEvent { code: KeyCode::A, pressed: true, modifiers }` |
| AC-2.2 | Integration | Key release event after `xdotool key a` has `pressed: false` |
| AC-2.3 | Integration | `xdotool key ctrl+alt+equal` produces `KeyEvent` with `modifiers.ctrl == true, modifiers.alt == true` |
| AC-2.4 | Unit | Fill channel to 31/32, send key event via blocking send, verify it is delivered (not dropped) |
| AC-3.1 | Unit | `X11InputMonitor::new()` with invalid `DISPLAY` returns `InputError::Unavailable` |
| AC-3.2 | Unit | Mock XI2 version < 2.0 response, verify `InputError::Unavailable` |
| AC-3.3 | Integration | Start monitor, verify channel closes when X11 connection is severed |
| AC-4.1 | Integration | Mouse button press produces `MouseButton` event with correct button |
| AC-4.2 | Integration | Scroll wheel produces `Scroll` event with non-zero `delta_y` |

## Performance Targets

| Metric | Target | Source |
|--------|--------|--------|
| Input event latency (mouse move to channel) | < 16.67ms (1 frame) | NFR-1, SC1 |
| `get_mouse_position()` latency | < 1ms | NFR-5 |
| X11 connection: persistent, not per-call | Single connection per thread | NFR-3 |
| Monitor thread CPU usage (idle) | < 0.1% | Blocking `wait_for_event()` |

## Security Considerations

- **No elevated privileges required:** XInput2 event monitoring on the root window is available to any X11 client. No special permissions, setuid, or capabilities are needed.
- **Global keylogging risk:** The `InputMonitor` captures all keyboard input globally. This is inherent to the accessibility tool's requirements (global hotkey detection). The input events are only dispatched within the application process (via the mpsc channel) and are never logged or persisted. The `InputEvent::KeyEvent` type does NOT capture the character value, only the keycode -- this limits exposure.
- **X11 security model:** X11 has no per-client input isolation. Any X11 client can monitor all input on the display. This is a known X11 limitation (and a motivation for Wayland's security model). Luminos operates within the standard X11 security context.

## Alternatives Considered

### Alternative 1: Use `rdev` crate (rejected)

The tech strategy (doc-02 Section 3.6, 8.1) recommends `rdev` as the primary input library. Rejected because:
- `rdev` v0.5.x is a single-maintainer crate (RISK-031) with infrequent releases
- `rdev`'s `listen()` function wraps the same X11 APIs we can use directly via x11rb
- `rdev`'s Wayland `grab()` path intercepts events rather than passively monitoring (unsuitable)
- Using x11rb directly avoids an additional dependency and gives full XI2 control
- x11rb is already a workspace dependency (v0.13)

### Alternative 2: XRecord extension for keyboard (rejected)

XRecord provides global keyboard event capture as an alternative to XInput2. Rejected because:
- XInput2 already provides `RawKeyPress`/`RawKeyRelease` for global key capture
- Using a single extension (XI2) for all event types simplifies the implementation
- XRecord would require a second X11 extension initialization and a separate event dispatch path
- XRecord's API is more complex (context creation, data interception ranges)

### Alternative 3: Separate `RawMotion` vs. `Motion` events (design choice)

XInput2 offers both `RawMotion` and `Motion` event types. `RawMotion` events bypass window manager transforms and are always in screen coordinates. `Motion` events may be transformed by grabs. The design uses regular `Motion` events (via the `MOTION` mask) because:
- Regular Motion events on the root window already provide absolute screen coordinates
- They include the final position after acceleration, which is what the user perceives
- `RawMotion` events provide delta values (relative motion), which would require accumulation to get absolute position -- less reliable than absolute coordinates from standard `Motion`
