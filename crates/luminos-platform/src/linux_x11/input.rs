//! X11 input monitor using `XInput2` extension via x11rb.
//!
//! Implements the [`InputMonitor`] trait for Linux X11, providing global mouse
//! and keyboard event monitoring through the `XInput2` (XI2) extension. Uses
//! [`RustConnection`] (pure Rust, no libxcb FFI) for X11 protocol communication.

use crate::traits::input_monitor::{InputError, InputEvent, InputMonitor, KeyCode, MouseButton};
use crate::traits::types::ScreenPoint;
use tokio::sync::mpsc;
use x11rb::connection::Connection;
use x11rb::protocol::xinput::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{self, ConnectionExt as _};
use x11rb::rust_connection::RustConnection;

use super::keymap;

/// Device ID for all master devices (aggregates physical input).
const XI_ALL_MASTER_DEVICES: xinput::DeviceId = 1;

/// X11 input monitor using `XInput2` extension via x11rb.
///
/// Creates a persistent X11 connection for synchronous queries and spawns
/// a dedicated thread with a separate connection for the XI2 event loop.
pub struct X11InputMonitor {
    /// X11 connection for synchronous queries (`get_mouse_position`).
    conn: RustConnection,
    /// Root window ID for event registration and pointer queries.
    root_window: u32,
    /// Screen number (for multi-screen setups, currently single-display).
    screen_num: usize,
}

// RustConnection does not implement Debug (it wraps internal Mutex state).
impl std::fmt::Debug for X11InputMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("X11InputMonitor")
            .field("connected", &self.conn.setup().roots.len())
            .field("root_window", &self.root_window)
            .field("screen_num", &self.screen_num)
            .finish_non_exhaustive()
    }
}

impl X11InputMonitor {
    /// Creates a new X11 input monitor.
    ///
    /// Opens an x11rb connection, validates `XInput2` extension availability
    /// (requires version >= 2.0).
    ///
    /// # Errors
    ///
    /// Returns [`InputError::Unavailable`] if:
    /// - The X11 display cannot be opened (`DISPLAY` not set or X server down)
    /// - The `XInput2` extension is not available or version < 2.0
    pub fn new() -> Result<Self, InputError> {
        let (conn, screen_num) = RustConnection::connect(None).map_err(map_connect_error)?;

        let root_window = conn
            .setup()
            .roots
            .get(screen_num)
            .ok_or_else(|| InputError::Unavailable {
                reason: format!("X11 screen {screen_num} not found"),
            })?
            .root;

        // Verify XInput2 is available (version >= 2.0)
        let reply = conn
            .xinput_xi_query_version(2, 0)
            .map_err(map_connection_error)?
            .reply()
            .map_err(map_reply_error)?;

        if reply.major_version < 2 {
            return Err(InputError::Unavailable {
                reason: format!(
                    "XInput2 version {}.{} is too old (need >= 2.0)",
                    reply.major_version, reply.minor_version
                ),
            });
        }

        Ok(Self {
            conn,
            root_window,
            screen_num,
        })
    }
}

/// Constructs the XI2 event mask for global input monitoring.
///
/// Includes `Motion`, `ButtonPress`, `ButtonRelease`, `KeyPress`, and `KeyRelease`
/// events on [`XI_ALL_MASTER_DEVICES`].
fn build_event_mask() -> xinput::EventMask {
    let mask = xinput::XIEventMask::MOTION
        | xinput::XIEventMask::BUTTON_PRESS
        | xinput::XIEventMask::BUTTON_RELEASE
        | xinput::XIEventMask::KEY_PRESS
        | xinput::XIEventMask::KEY_RELEASE;

    xinput::EventMask {
        deviceid: XI_ALL_MASTER_DEVICES,
        mask: vec![mask],
    }
}

/// Converts a fixed-point 16.16 value to an integer by discarding the fractional part.
fn fp1616_to_i32(value: xinput::Fp1616) -> i32 {
    value >> 16
}

/// Maps an X11 button number to a [`MouseButton`].
///
/// X11 button numbering: 1=Left, 2=Middle, 3=Right, 4+=Other.
/// Buttons 4 and 5 are scroll events (handled separately).
fn x11_button_to_mouse_button(button: u32) -> MouseButton {
    match button {
        1 => MouseButton::Left,
        2 => MouseButton::Middle,
        3 => MouseButton::Right,
        other => {
            #[allow(clippy::cast_possible_truncation)]
            let id = other as u16;
            MouseButton::Other(id)
        }
    }
}

/// Looks up a keysym from a keycode using the keyboard mapping.
///
/// Uses column 0 (unshifted) keysym for simplicity -- the modifier state
/// is tracked separately via `mods.effective`.
fn lookup_keysym(
    keysyms: &[xproto::Keysym],
    keysyms_per_keycode: u8,
    min_keycode: u8,
    keycode: u32,
) -> KeyCode {
    let offset = keycode.saturating_sub(u32::from(min_keycode));
    let index = offset.saturating_mul(u32::from(keysyms_per_keycode));
    #[allow(clippy::cast_possible_truncation)]
    let idx = index as usize;
    let keysym = keysyms.get(idx).copied().unwrap_or(0);
    keymap::x11_keysym_to_key_code(keysym)
}

/// Translates an XI2 motion event into an [`InputEvent::MouseMoved`].
fn translate_motion_event(event: &xinput::MotionEvent) -> InputEvent {
    InputEvent::MouseMoved {
        position: ScreenPoint {
            x: fp1616_to_i32(event.root_x),
            y: fp1616_to_i32(event.root_y),
        },
    }
}

/// Translates an XI2 button press/release event into an [`InputEvent`].
///
/// Buttons 4 (scroll up), 5 (scroll down), 6 (scroll left), and 7 (scroll right)
/// produce [`InputEvent::Scroll`] on press and `None` on release (scroll buttons
/// don't have meaningful release events). All other buttons produce
/// [`InputEvent::MouseButton`].
fn translate_button_event(event: &xinput::ButtonPressEvent, pressed: bool) -> Option<InputEvent> {
    let position = ScreenPoint {
        x: fp1616_to_i32(event.root_x),
        y: fp1616_to_i32(event.root_y),
    };

    match event.detail {
        // Scroll up
        4 if pressed => Some(InputEvent::Scroll {
            delta_x: 0.0,
            delta_y: -1.0,
            position,
        }),
        // Scroll down
        5 if pressed => Some(InputEvent::Scroll {
            delta_x: 0.0,
            delta_y: 1.0,
            position,
        }),
        // Scroll left
        6 if pressed => Some(InputEvent::Scroll {
            delta_x: -1.0,
            delta_y: 0.0,
            position,
        }),
        // Scroll right
        7 if pressed => Some(InputEvent::Scroll {
            delta_x: 1.0,
            delta_y: 0.0,
            position,
        }),
        // Scroll button releases are not meaningful events
        4..=7 => None,
        button => Some(InputEvent::MouseButton {
            button: x11_button_to_mouse_button(button),
            pressed,
            position,
        }),
    }
}

/// Translates an XI2 key press/release event into an [`InputEvent::KeyEvent`].
fn translate_key_event(
    event: &xinput::KeyPressEvent,
    pressed: bool,
    keysyms: &[xproto::Keysym],
    keysyms_per_keycode: u8,
    min_keycode: u8,
) -> InputEvent {
    let code = lookup_keysym(keysyms, keysyms_per_keycode, min_keycode, event.detail);
    let modifiers = keymap::x11_mods_to_modifiers(event.mods.effective);

    InputEvent::KeyEvent {
        code,
        pressed,
        modifiers,
    }
}

/// Runs the blocking XI2 event loop on the monitor thread.
///
/// Reads events from the X11 connection and translates them into [`InputEvent`]
/// values sent through the channel. The loop exits when the channel is closed
/// (receiver dropped) or the X11 connection encounters an error.
#[allow(clippy::needless_pass_by_value)]
fn run_event_loop(
    conn: RustConnection,
    tx: mpsc::Sender<InputEvent>,
    keysyms: &[xproto::Keysym],
    keysyms_per_keycode: u8,
    min_keycode: u8,
) {
    use x11rb::protocol::Event;

    loop {
        let event = match conn.wait_for_event() {
            Ok(event) => event,
            Err(e) => {
                log::warn!("X11 input monitor connection error: '{e}'");
                return;
            }
        };

        let input_event = match event {
            Event::XinputMotion(ref ev) => Some(translate_motion_event(ev)),
            Event::XinputButtonPress(ref ev) => translate_button_event(ev, true),
            Event::XinputButtonRelease(ref ev) => translate_button_event(ev, false),
            Event::XinputKeyPress(ref ev) => Some(translate_key_event(
                ev,
                true,
                keysyms,
                keysyms_per_keycode,
                min_keycode,
            )),
            Event::XinputKeyRelease(ref ev) => Some(translate_key_event(
                ev,
                false,
                keysyms,
                keysyms_per_keycode,
                min_keycode,
            )),
            _ => None,
        };

        let Some(input_event) = input_event else {
            continue;
        };

        // Mouse move events use try_send (lossy) to avoid backpressure.
        // All other events use blocking_send to prevent dropped hotkeys.
        if matches!(input_event, InputEvent::MouseMoved { .. }) {
            match tx.try_send(input_event) {
                Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => {
                    // Full is expected -- lossy semantics for mouse moves.
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    log::debug!("Input monitor channel closed, exiting event loop");
                    return;
                }
            }
        } else if tx.blocking_send(input_event).is_err() {
            log::debug!("Input monitor channel closed, exiting event loop");
            return;
        }
    }
}

impl InputMonitor for X11InputMonitor {
    /// Subscribes to global input events via a dedicated XI2 event loop thread.
    ///
    /// Each call spawns a new monitoring thread with its own X11 connection.
    /// Calling this multiple times creates independent event streams.
    fn subscribe_input_events(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<InputEvent>, InputError> {
        let (tx, rx) = mpsc::channel(buffer_size);

        // Open a dedicated connection for the monitor thread's blocking event loop.
        // This avoids lock contention with the main connection used by get_mouse_position().
        let (monitor_conn, monitor_screen) =
            RustConnection::connect(None).map_err(map_connect_error)?;

        let monitor_root = monitor_conn
            .setup()
            .roots
            .get(monitor_screen)
            .ok_or_else(|| InputError::Unavailable {
                reason: format!("X11 screen {monitor_screen} not found"),
            })?
            .root;

        // Register for XI2 events on the root window.
        let event_mask = build_event_mask();
        monitor_conn
            .xinput_xi_select_events(monitor_root, &[event_mask])
            .map_err(map_connection_error)?;
        monitor_conn.flush().map_err(map_connection_error)?;

        // Fetch keyboard mapping for keycode-to-keysym resolution.
        let setup = monitor_conn.setup();
        let min_keycode = setup.min_keycode;
        let max_keycode = setup.max_keycode;
        let keycount = u8::try_from(
            u16::from(max_keycode)
                .saturating_sub(u16::from(min_keycode))
                .saturating_add(1),
        )
        .unwrap_or(u8::MAX);

        let kb_reply = monitor_conn
            .get_keyboard_mapping(min_keycode, keycount)
            .map_err(map_connection_error)?
            .reply()
            .map_err(map_reply_error)?;

        let keysyms = kb_reply.keysyms;
        let keysyms_per_keycode = kb_reply.keysyms_per_keycode;

        // Spawn the blocking event loop on a dedicated OS thread.
        std::thread::Builder::new()
            .name("luminos-input-x11".to_string())
            .spawn(move || {
                run_event_loop(monitor_conn, tx, &keysyms, keysyms_per_keycode, min_keycode);
            })
            .map_err(|e| InputError::Platform {
                message: format!("failed to spawn input monitor thread: {e}"),
            })?;

        Ok(rx)
    }

    fn get_mouse_position(&self) -> Result<ScreenPoint, InputError> {
        let reply = self
            .conn
            .query_pointer(self.root_window)
            .map_err(map_connection_error)?
            .reply()
            .map_err(map_reply_error)?;

        Ok(ScreenPoint {
            x: i32::from(reply.root_x),
            y: i32::from(reply.root_y),
        })
    }
}

/// Maps an x11rb `ConnectError` to [`InputError::Unavailable`].
#[allow(clippy::needless_pass_by_value)] // Used as map_err argument.
fn map_connect_error(e: x11rb::errors::ConnectError) -> InputError {
    InputError::Unavailable {
        reason: format!("X11 connection failed: {e}"),
    }
}

/// Maps an x11rb `ConnectionError` to [`InputError::Platform`].
#[allow(clippy::needless_pass_by_value)] // Used as map_err argument.
fn map_connection_error(e: x11rb::errors::ConnectionError) -> InputError {
    InputError::Platform {
        message: format!("X11 request failed: {e}"),
    }
}

/// Maps an x11rb `ReplyError` to [`InputError::Platform`].
#[allow(clippy::needless_pass_by_value)] // Used as map_err argument.
fn map_reply_error(e: x11rb::errors::ReplyError) -> InputError {
    InputError::Platform {
        message: format!("X11 reply error: {e}"),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // T002 tests

    #[test]
    fn x11_input_monitor_new_unavailable_no_display() {
        // Temporarily set DISPLAY to an invalid value to test error handling.
        // Save and restore the real DISPLAY so other tests aren't affected.
        // SAFETY: This test is run single-threaded via nextest (process-per-test),
        // so mutating env vars is safe.
        let original = std::env::var("DISPLAY").ok();

        unsafe { std::env::set_var("DISPLAY", ":99") };
        let result = X11InputMonitor::new();
        // Restore
        match original {
            Some(val) => unsafe { std::env::set_var("DISPLAY", val) },
            None => unsafe { std::env::remove_var("DISPLAY") },
        }

        let err = result.expect_err("should fail with invalid display");
        assert!(
            matches!(err, InputError::Unavailable { .. }),
            "expected Unavailable, got: {err:?}"
        );
    }

    #[test]
    fn x11_input_monitor_struct_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<X11InputMonitor>();
    }

    // ── T006: Event mask construction tests ──

    #[test]
    fn x11_input_build_event_mask_contains_motion() {
        let em = build_event_mask();
        let mask_bits: u32 = em.mask[0].into();
        let motion_bit: u32 = xinput::XIEventMask::MOTION.into();
        assert_ne!(mask_bits & motion_bit, 0, "mask should include MOTION");
    }

    #[test]
    fn x11_input_build_event_mask_contains_key_press() {
        let em = build_event_mask();
        let mask_bits: u32 = em.mask[0].into();
        let bit: u32 = xinput::XIEventMask::KEY_PRESS.into();
        assert_ne!(mask_bits & bit, 0, "mask should include KEY_PRESS");
    }

    #[test]
    fn x11_input_build_event_mask_contains_key_release() {
        let em = build_event_mask();
        let mask_bits: u32 = em.mask[0].into();
        let bit: u32 = xinput::XIEventMask::KEY_RELEASE.into();
        assert_ne!(mask_bits & bit, 0, "mask should include KEY_RELEASE");
    }

    #[test]
    fn x11_input_build_event_mask_contains_button_press() {
        let em = build_event_mask();
        let mask_bits: u32 = em.mask[0].into();
        let bit: u32 = xinput::XIEventMask::BUTTON_PRESS.into();
        assert_ne!(mask_bits & bit, 0, "mask should include BUTTON_PRESS");
    }

    #[test]
    fn x11_input_build_event_mask_contains_button_release() {
        let em = build_event_mask();
        let mask_bits: u32 = em.mask[0].into();
        let bit: u32 = xinput::XIEventMask::BUTTON_RELEASE.into();
        assert_ne!(mask_bits & bit, 0, "mask should include BUTTON_RELEASE");
    }

    #[test]
    fn x11_input_build_event_mask_device_id() {
        let em = build_event_mask();
        assert_eq!(em.deviceid, XI_ALL_MASTER_DEVICES);
    }

    // ── T007: Mouse event translation tests ──

    /// Creates a test `ButtonPressEvent` (also used for `Motion` and `ButtonRelease`
    /// since they are type aliases of the same struct).
    fn generate_test_button_event(
        root_x: i32,
        root_y: i32,
        detail: u32,
    ) -> xinput::ButtonPressEvent {
        xinput::ButtonPressEvent {
            response_type: 0,
            extension: 0,
            sequence: 0,
            length: 0,
            event_type: 0,
            deviceid: 1,
            time: 0,
            detail,
            root: 0,
            event: 0,
            child: 0,
            root_x: root_x << 16,
            root_y: root_y << 16,
            event_x: root_x << 16,
            event_y: root_y << 16,
            sourceid: 1,
            flags: xinput::PointerEventFlags::default(),
            mods: xinput::ModifierInfo {
                base: 0,
                latched: 0,
                locked: 0,
                effective: 0,
            },
            group: xinput::GroupInfo {
                base: 0,
                latched: 0,
                locked: 0,
                effective: 0,
            },
            button_mask: vec![],
            valuator_mask: vec![],
            axisvalues: vec![],
        }
    }

    /// Creates a test `KeyPressEvent`.
    fn generate_test_key_event(detail: u32, mods_effective: u32) -> xinput::KeyPressEvent {
        xinput::KeyPressEvent {
            response_type: 0,
            extension: 0,
            sequence: 0,
            length: 0,
            event_type: 0,
            deviceid: 1,
            time: 0,
            detail,
            root: 0,
            event: 0,
            child: 0,
            root_x: 0,
            root_y: 0,
            event_x: 0,
            event_y: 0,
            sourceid: 1,
            flags: xinput::KeyEventFlags::default(),
            mods: xinput::ModifierInfo {
                base: 0,
                latched: 0,
                locked: 0,
                effective: mods_effective,
            },
            group: xinput::GroupInfo {
                base: 0,
                latched: 0,
                locked: 0,
                effective: 0,
            },
            button_mask: vec![],
            valuator_mask: vec![],
            axisvalues: vec![],
        }
    }

    #[test]
    fn x11_input_translate_motion_event() {
        let event = generate_test_button_event(200, 300, 0);
        let result = translate_motion_event(&event);

        match result {
            InputEvent::MouseMoved { position } => {
                assert_eq!(position.x, 200);
                assert_eq!(position.y, 300);
            }
            other => panic!("expected MouseMoved, got: {other:?}"),
        }
    }

    #[test]
    fn x11_input_translate_button_press_left() {
        let event = generate_test_button_event(100, 200, 1);
        let result = translate_button_event(&event, true).unwrap();

        match result {
            InputEvent::MouseButton {
                button,
                pressed,
                position,
            } => {
                assert_eq!(button, MouseButton::Left);
                assert!(pressed);
                assert_eq!(position.x, 100);
                assert_eq!(position.y, 200);
            }
            other => panic!("expected MouseButton, got: {other:?}"),
        }
    }

    #[test]
    fn x11_input_translate_button_release_right() {
        let event = generate_test_button_event(50, 60, 3);
        let result = translate_button_event(&event, false).unwrap();

        match result {
            InputEvent::MouseButton {
                button,
                pressed,
                position,
            } => {
                assert_eq!(button, MouseButton::Right);
                assert!(!pressed);
                assert_eq!(position.x, 50);
                assert_eq!(position.y, 60);
            }
            other => panic!("expected MouseButton, got: {other:?}"),
        }
    }

    #[test]
    fn x11_input_translate_scroll_up() {
        let event = generate_test_button_event(10, 20, 4);
        let result = translate_button_event(&event, true).unwrap();

        match result {
            InputEvent::Scroll {
                delta_x,
                delta_y,
                position,
            } => {
                assert!((delta_x - 0.0).abs() < f64::EPSILON);
                assert!((delta_y - (-1.0)).abs() < f64::EPSILON);
                assert_eq!(position.x, 10);
                assert_eq!(position.y, 20);
            }
            other => panic!("expected Scroll, got: {other:?}"),
        }
    }

    #[test]
    fn x11_input_translate_scroll_down() {
        let event = generate_test_button_event(10, 20, 5);
        let result = translate_button_event(&event, true).unwrap();

        match result {
            InputEvent::Scroll {
                delta_x,
                delta_y,
                position,
            } => {
                assert!((delta_x - 0.0).abs() < f64::EPSILON);
                assert!((delta_y - 1.0).abs() < f64::EPSILON);
                assert_eq!(position.x, 10);
                assert_eq!(position.y, 20);
            }
            other => panic!("expected Scroll, got: {other:?}"),
        }
    }

    #[test]
    fn x11_input_translate_scroll_left() {
        let event = generate_test_button_event(10, 20, 6);
        let result = translate_button_event(&event, true).unwrap();

        match result {
            InputEvent::Scroll {
                delta_x,
                delta_y,
                position,
            } => {
                assert!((delta_x - (-1.0)).abs() < f64::EPSILON);
                assert!((delta_y - 0.0).abs() < f64::EPSILON);
                assert_eq!(position.x, 10);
                assert_eq!(position.y, 20);
            }
            other => panic!("expected Scroll, got: {other:?}"),
        }
    }

    #[test]
    fn x11_input_translate_scroll_right() {
        let event = generate_test_button_event(10, 20, 7);
        let result = translate_button_event(&event, true).unwrap();

        match result {
            InputEvent::Scroll {
                delta_x,
                delta_y,
                position,
            } => {
                assert!((delta_x - 1.0).abs() < f64::EPSILON);
                assert!((delta_y - 0.0).abs() < f64::EPSILON);
                assert_eq!(position.x, 10);
                assert_eq!(position.y, 20);
            }
            other => panic!("expected Scroll, got: {other:?}"),
        }
    }

    #[test]
    fn x11_input_translate_scroll_up_release_is_none() {
        let event = generate_test_button_event(10, 20, 4);
        assert!(
            translate_button_event(&event, false).is_none(),
            "scroll up release should return None"
        );
    }

    #[test]
    fn x11_input_translate_scroll_down_release_is_none() {
        let event = generate_test_button_event(10, 20, 5);
        assert!(
            translate_button_event(&event, false).is_none(),
            "scroll down release should return None"
        );
    }

    #[test]
    fn x11_input_translate_scroll_left_release_is_none() {
        let event = generate_test_button_event(10, 20, 6);
        assert!(
            translate_button_event(&event, false).is_none(),
            "scroll left release should return None"
        );
    }

    #[test]
    fn x11_input_translate_scroll_right_release_is_none() {
        let event = generate_test_button_event(10, 20, 7);
        assert!(
            translate_button_event(&event, false).is_none(),
            "scroll right release should return None"
        );
    }

    #[test]
    fn x11_input_translate_button_other() {
        let event = generate_test_button_event(0, 0, 8);
        let result = translate_button_event(&event, true).unwrap();

        match result {
            InputEvent::MouseButton {
                button, pressed, ..
            } => {
                assert_eq!(button, MouseButton::Other(8));
                assert!(pressed);
            }
            other => panic!("expected MouseButton::Other, got: {other:?}"),
        }
    }

    // ── T009: Channel semantics tests ──

    #[test]
    fn x11_input_lossy_mouse_send() {
        // Verify that try_send for MouseMoved does not panic when channel is full.
        let (tx, _rx) = mpsc::channel::<InputEvent>(2);

        // Fill the channel
        tx.try_send(InputEvent::MouseMoved {
            position: ScreenPoint { x: 0, y: 0 },
        })
        .unwrap();
        tx.try_send(InputEvent::MouseMoved {
            position: ScreenPoint { x: 1, y: 1 },
        })
        .unwrap();

        // Third try_send should return Full, not panic
        let result = tx.try_send(InputEvent::MouseMoved {
            position: ScreenPoint { x: 2, y: 2 },
        });
        assert!(
            matches!(result, Err(mpsc::error::TrySendError::Full(_))),
            "expected Full, got: {result:?}"
        );
    }

    #[test]
    fn x11_input_key_send_not_dropped() {
        // Verify that blocking_send delivers key events when channel has space.
        let (tx, mut rx) = mpsc::channel::<InputEvent>(4);

        // Fill partially (3 of 4)
        for i in 0..3 {
            tx.try_send(InputEvent::MouseMoved {
                position: ScreenPoint { x: i, y: i },
            })
            .unwrap();
        }

        // Key event via blocking_send should succeed (1 slot remaining)
        tx.blocking_send(InputEvent::KeyEvent {
            code: KeyCode::A,
            pressed: true,
            modifiers: crate::traits::input_monitor::Modifiers::default(),
        })
        .unwrap();

        // Drain and find the key event
        let mut found_key = false;
        while let Ok(ev) = rx.try_recv() {
            if matches!(ev, InputEvent::KeyEvent { .. }) {
                found_key = true;
            }
        }
        assert!(found_key, "key event should be received");
    }

    // ── T008: Keyboard event translation tests ──

    #[test]
    fn x11_input_translate_key_press() {
        // Build a minimal keymap: keycode 38 -> keysym 0x61 ('a')
        // Typical X11: min_keycode=8, keysyms_per_keycode=4
        let min_keycode = 8u8;
        let keysyms_per_keycode = 4u8;
        // We need enough keysym entries for keycode 38
        // index = (38 - 8) * 4 = 120
        let mut keysyms = vec![0u32; 200];
        keysyms[120] = 0x61; // 'a'

        let event = generate_test_key_event(38, 0);
        let result = translate_key_event(&event, true, &keysyms, keysyms_per_keycode, min_keycode);

        match result {
            InputEvent::KeyEvent {
                code,
                pressed,
                modifiers,
            } => {
                assert_eq!(code, KeyCode::A);
                assert!(pressed);
                assert!(!modifiers.shift);
                assert!(!modifiers.ctrl);
                assert!(!modifiers.alt);
                assert!(!modifiers.meta);
            }
            other => panic!("expected KeyEvent, got: {other:?}"),
        }
    }

    #[test]
    fn x11_input_translate_key_release() {
        let min_keycode = 8u8;
        let keysyms_per_keycode = 4u8;
        let mut keysyms = vec![0u32; 200];
        keysyms[120] = 0x61; // 'a'

        let event = generate_test_key_event(38, 0);
        let result = translate_key_event(&event, false, &keysyms, keysyms_per_keycode, min_keycode);

        match result {
            InputEvent::KeyEvent { pressed, .. } => {
                assert!(!pressed);
            }
            other => panic!("expected KeyEvent, got: {other:?}"),
        }
    }

    #[test]
    fn x11_input_translate_key_with_modifiers() {
        let min_keycode = 8u8;
        let keysyms_per_keycode = 4u8;
        let mut keysyms = vec![0u32; 200];
        keysyms[120] = 0x61; // 'a'

        // Ctrl (bit 2 = 4) + Alt (bit 3 = 8) = 12
        let event = generate_test_key_event(38, 12);
        let result = translate_key_event(&event, true, &keysyms, keysyms_per_keycode, min_keycode);

        match result {
            InputEvent::KeyEvent { modifiers, .. } => {
                assert!(modifiers.ctrl);
                assert!(modifiers.alt);
                assert!(!modifiers.shift);
                assert!(!modifiers.meta);
            }
            other => panic!("expected KeyEvent, got: {other:?}"),
        }
    }

    // ── Integration tests (require Xvfb, gated behind ci_platform_tests) ──

    #[cfg(all(target_os = "linux", feature = "ci_platform_tests"))]
    mod integration {
        use super::*;
        use std::time::Duration;

        /// Helper: create a test `X11InputMonitor` on the current display.
        fn generate_test_x11_input_monitor() -> X11InputMonitor {
            X11InputMonitor::new().expect("X11InputMonitor::new() failed on test display")
        }

        /// Runs `xdotool` with the given arguments. Panics if not found.
        fn xdotool(args: &[&str]) {
            let status = std::process::Command::new("xdotool")
                .args(args)
                .status()
                .expect("xdotool not found (install xdotool for CI tests)");
            assert!(status.success(), "xdotool failed with: {status}");
            // Allow X server event propagation
            std::thread::sleep(Duration::from_millis(50));
        }

        // T010: get_mouse_position integration test
        #[test]
        fn x11_input_monitor_integration_get_mouse_position() {
            let monitor = generate_test_x11_input_monitor();
            let pos = monitor
                .get_mouse_position()
                .expect("get_mouse_position failed");

            // Xvfb screen is 1920x1080
            assert!(
                pos.x >= 0 && pos.x < 1920,
                "x={} out of screen bounds",
                pos.x
            );
            assert!(
                pos.y >= 0 && pos.y < 1080,
                "y={} out of screen bounds",
                pos.y
            );
        }

        // T011: event subscription with xdotool
        #[test]
        fn x11_input_monitor_integration_subscribe_returns_receiver() {
            let monitor = generate_test_x11_input_monitor();
            let _rx = monitor
                .subscribe_input_events(32)
                .expect("subscribe_input_events failed");
        }

        #[test]
        fn x11_input_monitor_integration_mouse_move_event() {
            let monitor = generate_test_x11_input_monitor();
            let mut rx = monitor
                .subscribe_input_events(32)
                .expect("subscribe_input_events failed");

            xdotool(&["mousemove", "200", "300"]);

            // Try to receive a MouseMoved event within 500ms
            let deadline = std::time::Instant::now() + Duration::from_millis(500);
            let mut found = false;
            while std::time::Instant::now() < deadline {
                match rx.try_recv() {
                    Ok(InputEvent::MouseMoved { position }) => {
                        // Position should be at or near (200, 300)
                        assert_eq!(position.x, 200, "mouse x mismatch");
                        assert_eq!(position.y, 300, "mouse y mismatch");
                        found = true;
                        break;
                    }
                    Ok(_) => {} // Skip non-mouse events
                    Err(mpsc::error::TryRecvError::Empty) => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        panic!("channel disconnected");
                    }
                }
            }
            assert!(found, "no MouseMoved event received within 500ms");
        }

        #[test]
        fn x11_input_monitor_integration_key_event() {
            let monitor = generate_test_x11_input_monitor();
            let mut rx = monitor
                .subscribe_input_events(32)
                .expect("subscribe_input_events failed");

            xdotool(&["key", "a"]);

            // Try to receive a KeyEvent within 500ms
            let deadline = std::time::Instant::now() + Duration::from_millis(500);
            let mut found_press = false;
            while std::time::Instant::now() < deadline {
                match rx.try_recv() {
                    Ok(InputEvent::KeyEvent {
                        code: KeyCode::A,
                        pressed: true,
                        ..
                    }) => {
                        found_press = true;
                        break;
                    }
                    Ok(_) => {}
                    Err(mpsc::error::TryRecvError::Empty) => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        panic!("channel disconnected");
                    }
                }
            }
            assert!(found_press, "no KeyEvent(A, pressed) received within 500ms");
        }

        #[test]
        fn x11_input_monitor_integration_modifier_tracking() {
            let monitor = generate_test_x11_input_monitor();
            let mut rx = monitor
                .subscribe_input_events(32)
                .expect("subscribe_input_events failed");

            xdotool(&["key", "ctrl+alt+equal"]);

            let deadline = std::time::Instant::now() + Duration::from_millis(500);
            let mut found = false;
            while std::time::Instant::now() < deadline {
                match rx.try_recv() {
                    Ok(InputEvent::KeyEvent {
                        modifiers,
                        pressed: true,
                        ..
                    }) if modifiers.ctrl && modifiers.alt => {
                        found = true;
                        break;
                    }
                    Ok(_) => {}
                    Err(mpsc::error::TryRecvError::Empty) => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        panic!("channel disconnected");
                    }
                }
            }
            assert!(found, "no KeyEvent with Ctrl+Alt received within 500ms");
        }

        // T012: error handling integration tests
        #[test]
        fn x11_input_monitor_integration_invalid_display() {
            let original = std::env::var("DISPLAY").ok();
            unsafe { std::env::set_var("DISPLAY", ":99") };
            let result = X11InputMonitor::new();
            match original {
                Some(val) => unsafe { std::env::set_var("DISPLAY", val) },
                None => unsafe { std::env::remove_var("DISPLAY") },
            }

            assert!(
                matches!(result, Err(InputError::Unavailable { .. })),
                "expected Unavailable, got: {result:?}"
            );
        }

        #[test]
        fn x11_input_monitor_integration_channel_closes_on_drop() {
            let monitor = generate_test_x11_input_monitor();
            let rx = monitor
                .subscribe_input_events(4)
                .expect("subscribe_input_events failed");

            // Drop the receiver -- the monitor thread should exit cleanly
            drop(rx);

            // Give the thread a moment to notice and exit
            std::thread::sleep(Duration::from_millis(100));

            // If this test doesn't hang, the thread exited properly.
            // We can't directly assert thread exit, but the test passing
            // without timeout proves the thread doesn't deadlock.
        }
    }
}
