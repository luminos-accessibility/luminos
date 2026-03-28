//! Input monitoring trait and associated types.
//!
//! Defines the [`InputMonitor`] trait for global input event monitoring,
//! along with [`InputEvent`], [`MouseButton`], [`KeyCode`], [`Modifiers`],
//! and [`InputError`].

use tokio::sync::mpsc;

use super::types::ScreenPoint;

/// Keyboard modifier state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct Modifiers {
    /// Shift key is held.
    pub shift: bool,
    /// Ctrl key is held.
    pub ctrl: bool,
    /// Alt key is held.
    pub alt: bool,
    /// "Super" on Linux, "Cmd" on macOS, "Win" on Windows.
    pub meta: bool,
}

/// A global input event.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// The mouse pointer moved to a new position.
    MouseMoved {
        /// Absolute screen-coordinate position.
        position: ScreenPoint,
    },
    /// A mouse button was pressed or released.
    MouseButton {
        /// The button that changed state.
        button: MouseButton,
        /// `true` if pressed, `false` if released.
        pressed: bool,
        /// Current pointer position.
        position: ScreenPoint,
    },
    /// The scroll wheel was moved.
    Scroll {
        /// Horizontal scroll delta (positive = right).
        delta_x: f64,
        /// Vertical scroll delta (positive = down).
        delta_y: f64,
        /// Current pointer position.
        position: ScreenPoint,
    },
    /// A keyboard key was pressed or released.
    KeyEvent {
        /// Platform-independent key code.
        code: KeyCode,
        /// `true` if pressed, `false` if released.
        pressed: bool,
        /// Active modifier keys.
        modifiers: Modifiers,
    },
}

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button (scroll wheel click).
    Middle,
    /// Other mouse button with platform-specific ID.
    Other(u16),
}

/// Platform-independent key code.
///
/// A simplified subset covering keys used for Luminos shortcuts.
/// A full keycode mapping is deferred to the input backend implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum KeyCode {
    // Alphanumeric
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    // Navigation
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    // Modifiers (as standalone key events)
    ShiftLeft,
    ShiftRight,
    CtrlLeft,
    CtrlRight,
    AltLeft,
    AltRight,
    MetaLeft,
    MetaRight,
    // Common
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    // Punctuation used in shortcuts
    Plus,
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    // Numpad
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    // Catch-all for keys not in this enum
    Unknown(u32),
}

/// Errors that can occur during input monitoring.
#[derive(Debug, thiserror::Error)]
pub enum InputError {
    /// Global input monitoring is not available (e.g., permission denied).
    #[error("input monitoring unavailable: {reason}")]
    Unavailable {
        /// Reason input monitoring is unavailable.
        reason: String,
    },

    /// The input monitoring backend disconnected unexpectedly.
    #[error("input monitor disconnected: {message}")]
    Disconnected {
        /// Description of the disconnection.
        message: String,
    },

    /// A platform-specific error occurred.
    #[error("platform input error: {message}")]
    Platform {
        /// Description of the platform error.
        message: String,
    },
}

/// Global input event monitoring.
///
/// Monitors mouse movement, clicks, scroll events, and keyboard events
/// globally (across all applications, not just when Luminos has focus).
/// This is essential for cursor-follow magnification.
///
/// # Platform Considerations
///
/// | Platform | Primary | Fallback |
/// |----------|---------|----------|
/// | Linux X11 | rdev | XInput2 / XRecord extension |
/// | Linux Wayland | rdev (evdev) | libinput (requires permissions) |
/// | macOS | rdev | CGEvent tap (requires Accessibility permission) |
/// | OpenBSD | rdev | XInput2 / XRecord |
/// | Windows | rdev | Raw Input / Low-level hooks |
pub trait InputMonitor: Send + Sync {
    /// Begins monitoring input events and returns a receiver.
    ///
    /// The implementation spawns an internal event loop that captures
    /// global input events and sends them to the returned channel.
    /// The channel is bounded (`buffer_size` capacity).
    ///
    /// # Errors
    ///
    /// Returns [`InputError`] if input monitoring is unavailable or permission is denied.
    fn subscribe_input_events(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<InputEvent>, InputError>;

    /// Returns the current mouse pointer position.
    ///
    /// # Errors
    ///
    /// Returns [`InputError`] if the mouse position cannot be queried.
    fn get_mouse_position(&self) -> Result<ScreenPoint, InputError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_input_error_display_unavailable() {
        let err = InputError::Unavailable {
            reason: "no permission".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("input monitoring unavailable"));
        assert!(msg.contains("no permission"));
    }

    #[test]
    fn error_input_error_display_disconnected() {
        let err = InputError::Disconnected {
            message: "evdev gone".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("input monitor disconnected"));
        assert!(msg.contains("evdev gone"));
    }

    #[test]
    fn error_input_error_display_platform() {
        let err = InputError::Platform {
            message: "XRecord error".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("platform input error"));
        assert!(msg.contains("XRecord error"));
    }
}
