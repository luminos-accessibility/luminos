//! Window management trait and associated types.
//!
//! Defines the [`WindowManager`] trait for controlling the magnification
//! overlay window, along with [`OverlayMode`], [`DockEdge`], [`LensShape`],
//! and [`WindowError`].

use super::types::ScreenRect;

pub use luminos_types::{DockEdge, LensShape, OverlayMode};

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
/// This trait controls the magnification overlay window. On Linux X11 the
/// backend (`X11WindowManager`) drives an already-created overlay window (the
/// tao/Tauri overlay window opened by `luminos-app`) by its X11 window id via
/// raw `x11rb` protocol requests -- it uses no winit and creates no window.
/// The overlay is transparent, borderless, and always-on-top, with wgpu
/// rendering whose surface is sourced by `luminos-app` (not this trait).
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
    /// Binds the manager to the target display and confirms the overlay window.
    ///
    /// On the X11 backend this does **not** create a window (the overlay window
    /// is opened by `luminos-app`); it resolves the target display's bounds and
    /// confirms the bound X11 window id.
    ///
    /// # Errors
    ///
    /// Returns [`WindowError`] if the specified display cannot be resolved.
    fn create_overlay(&mut self, display_id: &str) -> Result<(), WindowError>;

    /// Sets the overlay's position and size in screen coordinates.
    ///
    /// # Errors
    ///
    /// Returns [`WindowError`] if the overlay bounds cannot be applied.
    fn set_overlay_bounds(&self, bounds: ScreenRect) -> Result<(), WindowError>;

    /// Switches the overlay to the specified display mode.
    ///
    /// # Errors
    ///
    /// Returns [`WindowError`] if the overlay mode cannot be changed.
    fn set_overlay_mode(&mut self, mode: OverlayMode) -> Result<(), WindowError>;

    /// Sets whether the overlay is always above other windows.
    ///
    /// # Errors
    ///
    /// Returns [`WindowError`] if the always-on-top property cannot be set.
    fn set_always_on_top(&self, always_on_top: bool) -> Result<(), WindowError>;

    /// Shows or hides the overlay window.
    ///
    /// # Errors
    ///
    /// Returns [`WindowError`] if the visibility cannot be changed.
    fn set_visible(&self, visible: bool) -> Result<(), WindowError>;

    /// Returns the raw window handle for wgpu surface creation, or `None` when
    /// the surface is sourced elsewhere.
    ///
    /// The X11 backend returns `None`: it controls an externally-owned window
    /// and the wgpu surface is built by `luminos-app` from the owned Tauri
    /// window, not through this trait (AD-3).
    fn raw_window_handle(&self) -> Option<&dyn raw_window_handle::HasWindowHandle>;

    /// Returns the raw display handle for wgpu surface creation, or `None` when
    /// the surface is sourced elsewhere (the X11 backend returns `None`).
    fn raw_display_handle(&self) -> Option<&dyn raw_window_handle::HasDisplayHandle>;
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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

    #[test]
    fn dock_edge_serde_roundtrip() {
        let variants = [
            (DockEdge::Top, "\"Top\""),
            (DockEdge::Bottom, "\"Bottom\""),
            (DockEdge::Left, "\"Left\""),
            (DockEdge::Right, "\"Right\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: DockEdge = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }

    #[test]
    fn lens_shape_serde_roundtrip() {
        let variants = [
            (LensShape::Rectangle, "\"Rectangle\""),
            (LensShape::Ellipse, "\"Ellipse\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: LensShape = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }

    #[test]
    fn overlay_mode_serde_roundtrip() {
        let variants_json = [
            (OverlayMode::FullScreen, "\"FullScreen\""),
            (
                OverlayMode::Lens {
                    width: 400,
                    height: 300,
                    shape: LensShape::Ellipse,
                },
                "{\"Lens\":{\"width\":400,\"height\":300,\"shape\":\"Ellipse\"}}",
            ),
            (
                OverlayMode::Docked {
                    edge: DockEdge::Bottom,
                    size_px: 540,
                },
                "{\"Docked\":{\"edge\":\"Bottom\",\"size_px\":540}}",
            ),
        ];
        for (variant, expected_json) in &variants_json {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: OverlayMode = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }
}
