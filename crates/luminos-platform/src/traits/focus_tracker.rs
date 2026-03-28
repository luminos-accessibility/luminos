//! Focus tracking trait and associated types.
//!
//! Defines the [`FocusTracker`] trait for monitoring keyboard focus changes
//! via platform accessibility APIs, along with [`FocusChangedEvent`],
//! [`ElementType`], and [`FocusError`].

use tokio::sync::mpsc;

use super::types::ScreenRect;

/// The type of UI element that received focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementType {
    /// A text input field or text area.
    TextInput,
    /// A button, checkbox, radio button, or similar control.
    Control,
    /// A menu or menu item.
    Menu,
    /// A list item, tree item, or table cell.
    ListItem,
    /// A hyperlink.
    Link,
    /// An element type not specifically categorized.
    Other(String),
}

/// A focus change event from the accessibility API.
#[derive(Debug, Clone)]
pub struct FocusChangedEvent {
    /// Platform-specific identifier for the focused element.
    pub element_id: String,
    /// Screen-coordinate bounds of the focused element.
    pub bounds: ScreenRect,
    /// The semantic type of the focused element.
    pub element_type: ElementType,
    /// The accessible name/label of the element, if available.
    pub label: Option<String>,
    /// The PID of the application owning the focused element.
    pub pid: Option<u32>,
}

/// Errors that can occur during focus tracking.
#[derive(Debug, thiserror::Error)]
pub enum FocusError {
    /// The accessibility API is not available on this platform.
    #[error("accessibility API unavailable: {reason}")]
    ApiUnavailable {
        /// Reason the API is unavailable.
        reason: String,
    },

    /// The required accessibility permission was not granted.
    #[error("accessibility permission denied")]
    PermissionDenied,

    /// The focused element could not be queried (e.g., application crashed).
    #[error("failed to query focused element: {message}")]
    QueryFailed {
        /// Description of the query failure.
        message: String,
    },

    /// The accessibility bus or service disconnected.
    #[error("accessibility service disconnected: {message}")]
    Disconnected {
        /// Description of the disconnection.
        message: String,
    },

    /// A platform-specific error occurred.
    #[error("platform focus error: {message}")]
    Platform {
        /// Description of the platform error.
        message: String,
    },
}

/// Keyboard focus tracking via platform accessibility APIs.
///
/// Focus tracking is inherently event-driven and asynchronous (events arrive
/// from D-Bus, the Accessibility API, or UI Automation at unpredictable times).
/// The `subscribe_focus_changes` method returns a channel receiver; the
/// implementation runs an event loop internally.
///
/// # Platform Implementations
///
/// | Platform | Struct | Mechanism |
/// |----------|--------|-----------|
/// | Linux X11 | `AtSpiTracker` | AT-SPI2 via D-Bus (`atspi` crate) |
/// | Linux Wayland | `AtSpiTracker` | Same (AT-SPI2 is display-protocol-independent) |
/// | macOS | `AxTracker` | AXUIElement + AXObserver (`objc2` crate) |
/// | OpenBSD | `MouseFallbackTracker` | No AT-SPI2 in base; mouse position only |
/// | Windows | `UiaTracker` | UI Automation (`windows` crate) |
pub trait FocusTracker: Send + Sync {
    /// Begins monitoring focus changes and returns a receiver for events.
    ///
    /// The implementation spawns an internal task that listens for
    /// accessibility events and sends `FocusChangedEvent` values to the
    /// returned channel. The channel is bounded (`buffer_size` capacity).
    ///
    /// Calling this method multiple times is idempotent; subsequent calls
    /// return new receivers attached to the same internal event source.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError`] if the accessibility API is unavailable or permission is denied.
    fn subscribe_focus_changes(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<FocusChangedEvent>, FocusError>;

    /// Queries the currently focused element synchronously.
    ///
    /// Returns `None` if no element has focus or if the accessibility API
    /// cannot determine the focused element.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError`] if querying the focused element fails.
    fn get_focused_element(&self) -> Result<Option<FocusChangedEvent>, FocusError>;

    /// Returns the screen-coordinate bounds of a previously identified element.
    ///
    /// The `element_id` is the platform-specific identifier from a prior
    /// `FocusChangedEvent`. Returns `None` if the element no longer exists
    /// or its bounds cannot be determined.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError`] if the element bounds cannot be queried.
    fn get_element_bounds(&self, element_id: &str) -> Result<Option<ScreenRect>, FocusError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_focus_error_display_api_unavailable() {
        let err = FocusError::ApiUnavailable {
            reason: "AT-SPI2 not running".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("accessibility API unavailable"));
        assert!(msg.contains("AT-SPI2 not running"));
    }

    #[test]
    fn error_focus_error_display_permission_denied() {
        let err = FocusError::PermissionDenied;
        assert_eq!(err.to_string(), "accessibility permission denied");
    }

    #[test]
    fn error_focus_error_display_query_failed() {
        let err = FocusError::QueryFailed {
            message: "app crashed".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("failed to query focused element"));
        assert!(msg.contains("app crashed"));
    }

    #[test]
    fn error_focus_error_display_disconnected() {
        let err = FocusError::Disconnected {
            message: "bus lost".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("accessibility service disconnected"));
        assert!(msg.contains("bus lost"));
    }

    #[test]
    fn error_focus_error_display_platform() {
        let err = FocusError::Platform {
            message: "D-Bus timeout".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("platform focus error"));
        assert!(msg.contains("D-Bus timeout"));
    }
}
