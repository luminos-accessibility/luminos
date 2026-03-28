//! Screen capture trait and associated types.
//!
//! Defines the [`ScreenCapture`] trait for capturing screen content as
//! CPU pixel buffers, along with [`DisplayChangeEvent`] for hot-plug
//! monitoring and [`CaptureError`] for error handling.

use tokio::sync::mpsc;

use super::types::{CaptureFrame, DisplayInfo, ScreenRect};

/// A display configuration change event.
#[derive(Debug, Clone)]
pub enum DisplayChangeEvent {
    /// A new display was connected.
    Connected(DisplayInfo),
    /// A display was disconnected. Contains the display ID.
    Disconnected(String),
    /// A display's configuration changed (resolution, scale, position).
    Reconfigured(DisplayInfo),
}

/// Errors that can occur during screen capture.
#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    /// The requested display was not found.
    #[error("display not found: '{0}'")]
    DisplayNotFound(String),

    /// The requested region is outside the display bounds.
    #[error("capture region {region:?} exceeds display bounds {bounds:?}")]
    RegionOutOfBounds {
        /// The requested capture region.
        region: ScreenRect,
        /// The actual display bounds.
        bounds: ScreenRect,
    },

    /// The user denied the required screen capture permission.
    #[error("screen capture permission denied")]
    PermissionDenied,

    /// The capture backend is not available on this system.
    #[error("capture backend unavailable: {reason}")]
    BackendUnavailable {
        /// Reason the backend is unavailable.
        reason: String,
    },

    /// A platform-specific error occurred.
    #[error("platform capture error: {message}")]
    Platform {
        /// Description of the platform error.
        message: String,
        /// Optional underlying error source.
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Screen capture abstraction.
///
/// Implementations capture screen content as CPU pixel buffers.
/// The capture is synchronous and blocking -- it completes within a single
/// frame budget (target: <8ms). The caller (the rendering pipeline) drives
/// the capture cadence.
///
/// # Platform Implementations
///
/// | Platform | Struct | Mechanism |
/// |----------|--------|-----------|
/// | Linux X11 | `XcbCapture` | xcap via XCB (`xcb_get_image`); XShm planned Phase 1 |
/// | Linux Wayland | `PipeWireCapture` | PipeWire + XDG Desktop Portal |
/// | macOS | `SCKitCapture` | ScreenCaptureKit via xcap |
/// | OpenBSD | `XcbCapture` | Shared with Linux X11 (xenocara) |
/// | Windows | `DxgiCapture` | DXGI Desktop Duplication via windows-capture |
pub trait ScreenCapture: Send + Sync {
    /// Lists all connected displays.
    ///
    /// Returns display metadata including bounds and scale factor.
    /// Used during initialization and when displays are added/removed.
    fn list_displays(&self) -> Result<Vec<DisplayInfo>, CaptureError>;

    /// Captures a rectangular region of the specified display.
    ///
    /// The `region` is in display-local coordinates (relative to the display's
    /// top-left corner). If `region` is `None`, captures the entire display.
    ///
    /// This is the hot-path method called every frame (up to 60fps).
    /// Implementations must target <8ms for the source region sizes typical
    /// in magnification (small regions at high zoom).
    fn capture_frame(
        &self,
        display_id: &str,
        region: Option<ScreenRect>,
    ) -> Result<CaptureFrame, CaptureError>;

    /// Subscribes to display configuration change events (hot-plug,
    /// resolution change, scale factor change).
    ///
    /// Returns a receiver that emits events when displays are connected,
    /// disconnected, or reconfigured. The core engine uses these events
    /// to refresh the display list and reposition the overlay.
    ///
    /// Returns `Err` if the platform does not support display change
    /// notifications (graceful degradation: the engine can poll
    /// `list_displays()` periodically as a fallback).
    fn subscribe_display_changes(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<DisplayChangeEvent>, CaptureError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_capture_error_display_not_found() {
        let err = CaptureError::DisplayNotFound("HDMI-1".into());
        assert_eq!(err.to_string(), "display not found: 'HDMI-1'");
    }

    #[test]
    fn error_capture_error_display_region_out_of_bounds() {
        let err = CaptureError::RegionOutOfBounds {
            region: ScreenRect {
                x: 0,
                y: 0,
                width: 3000,
                height: 2000,
            },
            bounds: ScreenRect {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
        };
        let msg = err.to_string();
        assert!(msg.contains("capture region"));
        assert!(msg.contains("exceeds display bounds"));
    }

    #[test]
    fn error_capture_error_display_permission_denied() {
        let err = CaptureError::PermissionDenied;
        assert_eq!(err.to_string(), "screen capture permission denied");
    }

    #[test]
    fn error_capture_error_display_backend_unavailable() {
        let err = CaptureError::BackendUnavailable {
            reason: "X11 not running".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("capture backend unavailable"));
        assert!(msg.contains("X11 not running"));
    }

    #[test]
    fn error_capture_error_display_platform() {
        let err = CaptureError::Platform {
            message: "XCB connection failed".into(),
            source: None,
        };
        let msg = err.to_string();
        assert!(msg.contains("platform capture error"));
        assert!(msg.contains("XCB connection failed"));
    }
}
