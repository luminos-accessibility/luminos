//! Window management trait and associated types.
//!
//! Defines the [`WindowManager`] trait for controlling the magnification
//! overlay window, along with [`OverlayMode`], [`DockEdge`], [`LensShape`],
//! and [`WindowError`].

use super::types::ScreenRect;

/// The edge of the screen where a docked overlay attaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockEdge {
    /// Top edge of the screen.
    Top,
    /// Bottom edge of the screen.
    Bottom,
    /// Left edge of the screen.
    Left,
    /// Right edge of the screen.
    Right,
}

/// The shape of a lens-mode overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LensShape {
    /// Rectangular lens boundary.
    Rectangle,
    /// Elliptical lens boundary.
    Ellipse,
}

/// The magnification overlay display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayMode {
    /// The overlay covers the entire display.
    FullScreen,
    /// A movable lens that follows the cursor.
    Lens {
        /// Width of the lens in pixels.
        width: u32,
        /// Height of the lens in pixels.
        height: u32,
        /// Shape of the lens boundary.
        shape: LensShape,
    },
    /// The overlay is docked to one edge of the screen.
    Docked {
        /// Which screen edge to dock against.
        edge: DockEdge,
        /// Size of the docked region in pixels (perpendicular to the edge).
        size_px: u32,
    },
}

/// Errors that can occur during window management.
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    /// The window could not be created.
    #[error("window creation failed: {message}")]
    CreationFailed {
        /// Description of the creation failure.
        message: String,
    },

    /// A window property could not be set.
    #[error("failed to set window property '{property}': {message}")]
    PropertyFailed {
        /// The property that failed to set.
        property: String,
        /// Description of the failure.
        message: String,
    },

    /// The requested display for the overlay was not found.
    #[error("target display not found: '{0}'")]
    DisplayNotFound(String),

    /// Platform-specific dock/strut reservation failed.
    #[error("dock reservation failed: {message}")]
    DockFailed {
        /// Description of the dock failure.
        message: String,
    },

    /// A platform-specific error occurred.
    #[error("platform window error: {message}")]
    Platform {
        /// Description of the platform error.
        message: String,
    },
}

/// Magnification overlay window management.
///
/// This trait controls the winit-based magnification overlay window.
/// The overlay is independent of the Tauri control panel -- it is a native
/// window with wgpu rendering, transparent, borderless, and always-on-top.
///
/// # Platform Implementations
///
/// | Platform | Struct | Dock Mechanism |
/// |----------|--------|----------------|
/// | Linux X11 | `X11WindowManager` | EWMH `_NET_WM_STRUT_PARTIAL` |
/// | Linux Wayland | `WaylandWindowManager` | Layer-shell protocol |
/// | macOS | `CocoaWindowManager` | Floating NSPanel (no reservation) |
/// | OpenBSD | `X11WindowManager` | Shared with Linux X11 (EWMH) |
/// | Windows | `Win32WindowManager` | `SHAppBarMessage` / AppBar API |
pub trait WindowManager: Send + Sync {
    /// Creates the magnification overlay window on the specified display.
    fn create_overlay(&mut self, display_id: &str) -> Result<(), WindowError>;

    /// Sets the overlay's position and size in screen coordinates.
    fn set_overlay_bounds(&self, bounds: ScreenRect) -> Result<(), WindowError>;

    /// Switches the overlay to the specified display mode.
    fn set_overlay_mode(&mut self, mode: OverlayMode) -> Result<(), WindowError>;

    /// Sets whether the overlay is always above other windows.
    fn set_always_on_top(&self, always_on_top: bool) -> Result<(), WindowError>;

    /// Shows or hides the overlay window.
    fn set_visible(&self, visible: bool) -> Result<(), WindowError>;

    /// Returns the raw window handle for wgpu surface creation.
    /// Returns `None` if the overlay has not been created yet.
    fn raw_window_handle(&self) -> Option<&dyn raw_window_handle::HasWindowHandle>;

    /// Returns the raw display handle for wgpu surface creation.
    fn raw_display_handle(&self) -> Option<&dyn raw_window_handle::HasDisplayHandle>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_window_error_display_creation_failed() {
        let err = WindowError::CreationFailed {
            message: "X11 connection refused".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("window creation failed"));
        assert!(msg.contains("X11 connection refused"));
    }

    #[test]
    fn error_window_error_display_property_failed() {
        let err = WindowError::PropertyFailed {
            property: "always_on_top".into(),
            message: "not supported".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("failed to set window property"));
        assert!(msg.contains("always_on_top"));
        assert!(msg.contains("not supported"));
    }

    #[test]
    fn error_window_error_display_display_not_found() {
        let err = WindowError::DisplayNotFound("HDMI-2".into());
        assert_eq!(err.to_string(), "target display not found: 'HDMI-2'");
    }

    #[test]
    fn error_window_error_display_dock_failed() {
        let err = WindowError::DockFailed {
            message: "strut rejected".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("dock reservation failed"));
        assert!(msg.contains("strut rejected"));
    }

    #[test]
    fn error_window_error_display_platform() {
        let err = WindowError::Platform {
            message: "NSWindow error".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("platform window error"));
        assert!(msg.contains("NSWindow error"));
    }
}
