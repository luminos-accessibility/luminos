//! X11 overlay window manager using winit.
//!
//! Creates and manages a transparent, borderless, always-on-top,
//! override-redirect window for magnification rendering on X11.

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::EventLoop;
use winit::platform::x11::{EventLoopBuilderExtX11, WindowAttributesExtX11};
use winit::window::{Window, WindowAttributes, WindowLevel};

use crate::traits::{OverlayMode, ScreenRect, WindowError, WindowManager};

/// X11 overlay window manager using winit.
///
/// Creates and manages a transparent, borderless, always-on-top,
/// override-redirect window for magnification rendering on X11. Uses
/// winit for window creation. Only `FullScreen` mode is implemented in
/// E02; docked and lens modes are deferred to Epic 5.
///
/// # Override-Redirect
///
/// The overlay uses `with_override_redirect(true)` to bypass the window
/// manager. This is critical: it prevents the WM from adding decorations,
/// applying focus policies, or interfering with always-on-top behavior.
/// Override-redirect windows are also naturally excluded from some X11
/// composite capture paths, which aids self-capture prevention (RISK-002).
///
/// # Event Loop Integration (E02)
///
/// In E02, window creation uses the deprecated `EventLoop::create_window()`
/// which works on X11 without an active event loop. The event loop is
/// created per `create_overlay()` call and dropped after window creation;
/// the X11 window survives because the underlying X connection is reference-
/// counted. In E05, the event loop will be managed by the render loop and
/// `create_window` will be called via `ActiveEventLoop` in the `Resumed`
/// callback.
///
/// # Platform Notes
///
/// - Transparency requires a compositing WM (Mutter, `KWin`, Picom).
///   On non-compositing WMs, the window background will be opaque black.
/// - Always-on-top is implemented via EWMH `_NET_WM_STATE_ABOVE`.
/// - Docked/Lens modes are deferred to Epic 5.
// Not yet wired into PlatformBackends; suppress dead_code until Story 002 integration.
#[allow(dead_code)]
pub struct X11WindowManager {
    /// The winit window for the overlay. `None` before `create_overlay()`.
    window: Option<Window>,
    /// Current overlay mode.
    current_mode: OverlayMode,
    /// Display bounds for the target display.
    display_bounds: Option<ScreenRect>,
}

#[allow(dead_code)]
impl X11WindowManager {
    /// Creates a new `X11WindowManager` with no active overlay.
    #[must_use]
    pub fn new() -> Self {
        Self {
            window: None,
            current_mode: OverlayMode::FullScreen,
            display_bounds: None,
        }
    }

    /// Returns the X11 window ID of the overlay window for self-capture
    /// exclusion (RISK-002).
    ///
    /// The returned ID is passed to `XcbCapture::new()` (Story 001) so that
    /// the capture backend can exclude this window from captured frames,
    /// preventing infinite feedback loops.
    ///
    /// # Returns
    ///
    /// `Some(window_id)` if the overlay has been created, `None` otherwise.
    #[must_use]
    pub fn overlay_window_id(&self) -> Option<u64> {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let handle = self.window.as_ref()?.window_handle().ok()?;
        match handle.as_raw() {
            // c_ulong is u64 on 64-bit, u32 on 32-bit; cast is needed for portability.
            #[allow(clippy::unnecessary_cast)]
            RawWindowHandle::Xlib(xlib) => Some(xlib.window as u64),
            RawWindowHandle::Xcb(xcb) => Some(u64::from(xcb.window.get())),
            _ => None,
        }
    }

    /// Returns a reference to the underlying winit window, if created.
    #[must_use]
    pub fn window(&self) -> Option<&Window> {
        self.window.as_ref()
    }
}

impl Default for X11WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Finds a monitor matching `display_id` via xcap and returns its bounds.
///
/// Matches against the monitor name (e.g., "HDMI-1") or the string
/// representation of its numeric ID. Returns the first match.
fn find_display_bounds(display_id: &str) -> Result<ScreenRect, WindowError> {
    let monitors = xcap::Monitor::all().map_err(|e| {
        WindowError::DisplayNotFound(format!("{display_id} (monitor enumeration failed: {e})"))
    })?;

    for monitor in &monitors {
        let name_matches = monitor.name().is_ok_and(|n| n == display_id);
        let id_matches = monitor.id().is_ok_and(|id| id.to_string() == display_id);

        if name_matches || id_matches {
            let x = monitor.x().unwrap_or(0);
            let y = monitor.y().unwrap_or(0);
            let width = monitor.width().unwrap_or(0);
            let height = monitor.height().unwrap_or(0);

            if width == 0 || height == 0 {
                return Err(WindowError::CreationFailed {
                    message: format!(
                        "display '{display_id}' reported zero dimensions ({width}x{height})"
                    ),
                });
            }

            return Ok(ScreenRect {
                x,
                y,
                width,
                height,
            });
        }
    }

    Err(WindowError::DisplayNotFound(display_id.to_string()))
}

/// Builds winit `WindowAttributes` for the overlay window.
fn build_overlay_attributes(bounds: &ScreenRect) -> WindowAttributes {
    Window::default_attributes()
        .with_transparent(true)
        .with_decorations(false)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_override_redirect(true)
        .with_inner_size(PhysicalSize::new(bounds.width, bounds.height))
        .with_position(PhysicalPosition::new(bounds.x, bounds.y))
        .with_title("Luminos Overlay")
}

impl WindowManager for X11WindowManager {
    fn create_overlay(&mut self, display_id: &str) -> Result<(), WindowError> {
        let bounds = find_display_bounds(display_id)?;

        let attrs = build_overlay_attributes(&bounds);

        // Create a temporary event loop for window creation. On X11, the
        // deprecated `create_window` works because the X connection is
        // reference-counted and survives event loop drop. This will be
        // replaced in E05 with ActiveEventLoop-based creation.
        // `with_any_thread(true)` is required because nextest (and the
        // real app's render thread) may initialise this off the main thread.
        #[allow(deprecated)]
        let event_loop = EventLoop::builder()
            .with_any_thread(true)
            .build()
            .map_err(|e| WindowError::CreationFailed {
                message: format!("failed to create event loop: {e}"),
            })?;

        #[allow(deprecated)]
        let window = event_loop
            .create_window(attrs)
            .map_err(|e| WindowError::CreationFailed {
                message: format!("winit window creation failed: {e}"),
            })?;

        // Start hidden; caller controls visibility via set_visible().
        window.set_visible(false);

        self.display_bounds = Some(bounds);
        self.window = Some(window);

        log::info!(
            "Created X11 overlay window on display '{}' ({}x{} at {},{})",
            display_id,
            bounds.width,
            bounds.height,
            bounds.x,
            bounds.y
        );

        Ok(())
    }

    fn set_overlay_bounds(&self, bounds: ScreenRect) -> Result<(), WindowError> {
        let window = self
            .window
            .as_ref()
            .ok_or_else(|| WindowError::PropertyFailed {
                property: "overlay_bounds".into(),
                message: "no overlay window exists".into(),
            })?;

        let _ = window.request_inner_size(PhysicalSize::new(bounds.width, bounds.height));
        window.set_outer_position(PhysicalPosition::new(bounds.x, bounds.y));

        Ok(())
    }

    fn set_overlay_mode(&mut self, mode: OverlayMode) -> Result<(), WindowError> {
        match mode {
            OverlayMode::FullScreen => {
                let bounds = self
                    .display_bounds
                    .ok_or_else(|| WindowError::PropertyFailed {
                        property: "overlay_mode".into(),
                        message: "no display bounds available (create_overlay not called)".into(),
                    })?;
                self.set_overlay_bounds(bounds)?;
                self.current_mode = OverlayMode::FullScreen;
                Ok(())
            }
            OverlayMode::Docked { .. } => Err(WindowError::PropertyFailed {
                property: "overlay_mode".into(),
                message: "Docked mode deferred to E05".into(),
            }),
            OverlayMode::Lens { .. } => Err(WindowError::PropertyFailed {
                property: "overlay_mode".into(),
                message: "Lens mode deferred to E05".into(),
            }),
        }
    }

    fn set_always_on_top(&self, always_on_top: bool) -> Result<(), WindowError> {
        let window = self
            .window
            .as_ref()
            .ok_or_else(|| WindowError::PropertyFailed {
                property: "always_on_top".into(),
                message: "no overlay window exists".into(),
            })?;

        let level = if always_on_top {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        };
        window.set_window_level(level);

        Ok(())
    }

    fn set_visible(&self, visible: bool) -> Result<(), WindowError> {
        let window = self
            .window
            .as_ref()
            .ok_or_else(|| WindowError::PropertyFailed {
                property: "visible".into(),
                message: "no overlay window exists".into(),
            })?;

        window.set_visible(visible);

        Ok(())
    }

    fn raw_window_handle(&self) -> Option<&dyn raw_window_handle::HasWindowHandle> {
        self.window
            .as_ref()
            .map(|w| w as &dyn raw_window_handle::HasWindowHandle)
    }

    fn raw_display_handle(&self) -> Option<&dyn raw_window_handle::HasDisplayHandle> {
        self.window
            .as_ref()
            .map(|w| w as &dyn raw_window_handle::HasDisplayHandle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x11_window_manager_new_default() {
        let wm = X11WindowManager::new();
        assert!(
            wm.raw_window_handle().is_none(),
            "raw_window_handle must be None before create_overlay"
        );
    }

    #[test]
    fn x11_window_manager_raw_display_handle_before_create() {
        let wm = X11WindowManager::new();
        assert!(
            wm.raw_display_handle().is_none(),
            "raw_display_handle must be None before create_overlay"
        );
    }

    #[test]
    fn x11_window_manager_overlay_window_id_before_create() {
        let wm = X11WindowManager::new();
        assert!(
            wm.overlay_window_id().is_none(),
            "overlay_window_id must be None before create_overlay"
        );
    }

    #[test]
    fn x11_window_manager_default_mode_is_fullscreen() {
        let wm = X11WindowManager::new();
        assert_eq!(wm.current_mode, OverlayMode::FullScreen);
    }

    #[test]
    fn x11_window_manager_default_display_bounds_none() {
        let wm = X11WindowManager::new();
        assert!(wm.display_bounds.is_none());
    }

    #[test]
    fn x11_window_manager_create_overlay_invalid_display() {
        let mut wm = X11WindowManager::new();
        let result = wm.create_overlay("nonexistent_display_xyz_12345");
        assert!(result.is_err(), "expected error for invalid display_id");
        match result {
            Err(WindowError::DisplayNotFound(id)) => {
                assert!(
                    id.contains("nonexistent_display_xyz_12345"),
                    "error should contain the display_id, got: {id}"
                );
            }
            other => panic!("expected DisplayNotFound, got: {other:?}"),
        }
    }

    #[test]
    fn x11_window_manager_set_visible_no_window() {
        let wm = X11WindowManager::new();
        let result = wm.set_visible(true);
        assert!(result.is_err());
        match result {
            Err(WindowError::PropertyFailed { property, .. }) => {
                assert_eq!(property, "visible");
            }
            other => panic!("expected PropertyFailed, got: {other:?}"),
        }
    }

    #[test]
    fn x11_window_manager_set_always_on_top_no_window() {
        let wm = X11WindowManager::new();
        let result = wm.set_always_on_top(true);
        assert!(result.is_err());
        match result {
            Err(WindowError::PropertyFailed { property, .. }) => {
                assert_eq!(property, "always_on_top");
            }
            other => panic!("expected PropertyFailed, got: {other:?}"),
        }
    }

    #[test]
    fn x11_window_manager_set_overlay_bounds_no_window() {
        let wm = X11WindowManager::new();
        let bounds = ScreenRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let result = wm.set_overlay_bounds(bounds);
        assert!(result.is_err());
        match result {
            Err(WindowError::PropertyFailed { property, .. }) => {
                assert_eq!(property, "overlay_bounds");
            }
            other => panic!("expected PropertyFailed, got: {other:?}"),
        }
    }

    #[test]
    fn x11_window_manager_set_overlay_mode_docked_rejected() {
        let mut wm = X11WindowManager::new();
        let result = wm.set_overlay_mode(OverlayMode::Docked {
            edge: luminos_types::DockEdge::Bottom,
            size_px: 540,
        });
        assert!(result.is_err());
        match &result {
            Err(WindowError::PropertyFailed { message, .. }) => {
                assert!(
                    message.contains("deferred to E05"),
                    "expected E05 deferral message, got: {message}"
                );
            }
            other => panic!("expected PropertyFailed, got: {other:?}"),
        }
    }

    #[test]
    fn x11_window_manager_set_overlay_mode_lens_rejected() {
        let mut wm = X11WindowManager::new();
        let result = wm.set_overlay_mode(OverlayMode::Lens {
            width: 400,
            height: 300,
            shape: luminos_types::LensShape::Ellipse,
        });
        assert!(result.is_err());
        match &result {
            Err(WindowError::PropertyFailed { message, .. }) => {
                assert!(
                    message.contains("deferred to E05"),
                    "expected E05 deferral message, got: {message}"
                );
            }
            other => panic!("expected PropertyFailed, got: {other:?}"),
        }
    }

    #[test]
    fn x11_window_manager_set_overlay_mode_fullscreen_no_bounds() {
        let mut wm = X11WindowManager::new();
        let result = wm.set_overlay_mode(OverlayMode::FullScreen);
        assert!(result.is_err());
        match &result {
            Err(WindowError::PropertyFailed { property, .. }) => {
                assert_eq!(property, "overlay_mode");
            }
            other => panic!("expected PropertyFailed, got: {other:?}"),
        }
    }

    // --- Integration tests requiring X11 display server ---

    #[cfg(all(target_os = "linux", feature = "ci_platform_tests"))]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    mod integration {
        use super::*;

        #[test]
        fn x11_window_manager_create_overlay_on_xvfb() {
            let mut wm = X11WindowManager::new();
            // On Xvfb, the default display is typically unnamed or "default".
            // Use xcap to find the first available monitor.
            let monitors = xcap::Monitor::all().expect("should enumerate monitors");
            assert!(
                !monitors.is_empty(),
                "Xvfb should provide at least one monitor"
            );
            let first = &monitors[0];
            let display_id = first
                .name()
                .unwrap_or_else(|_| first.id().unwrap().to_string());
            let result = wm.create_overlay(&display_id);
            assert!(result.is_ok(), "create_overlay failed: {result:?}");
            assert!(wm.window().is_some());
            assert!(wm.raw_window_handle().is_some());
            assert!(wm.raw_display_handle().is_some());
            assert!(wm.overlay_window_id().is_some());
        }

        #[test]
        fn x11_window_manager_set_visible_after_create() {
            let mut wm = X11WindowManager::new();
            let monitors = xcap::Monitor::all().expect("should enumerate monitors");
            let first = &monitors[0];
            let display_id = first
                .name()
                .unwrap_or_else(|_| first.id().unwrap().to_string());
            wm.create_overlay(&display_id)
                .expect("create_overlay should succeed");
            assert!(wm.set_visible(true).is_ok());
            assert!(wm.set_visible(false).is_ok());
        }

        #[test]
        fn x11_window_manager_set_always_on_top_after_create() {
            let mut wm = X11WindowManager::new();
            let monitors = xcap::Monitor::all().expect("should enumerate monitors");
            let first = &monitors[0];
            let display_id = first
                .name()
                .unwrap_or_else(|_| first.id().unwrap().to_string());
            wm.create_overlay(&display_id)
                .expect("create_overlay should succeed");
            assert!(wm.set_always_on_top(true).is_ok());
            assert!(wm.set_always_on_top(false).is_ok());
        }

        #[test]
        fn x11_window_manager_set_overlay_bounds_after_create() {
            let mut wm = X11WindowManager::new();
            let monitors = xcap::Monitor::all().expect("should enumerate monitors");
            let first = &monitors[0];
            let display_id = first
                .name()
                .unwrap_or_else(|_| first.id().unwrap().to_string());
            wm.create_overlay(&display_id)
                .expect("create_overlay should succeed");
            let bounds = ScreenRect {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            };
            assert!(wm.set_overlay_bounds(bounds).is_ok());
        }

        #[test]
        fn x11_window_manager_set_fullscreen_mode_after_create() {
            let mut wm = X11WindowManager::new();
            let monitors = xcap::Monitor::all().expect("should enumerate monitors");
            let first = &monitors[0];
            let display_id = first
                .name()
                .unwrap_or_else(|_| first.id().unwrap().to_string());
            wm.create_overlay(&display_id)
                .expect("create_overlay should succeed");
            assert!(wm.set_overlay_mode(OverlayMode::FullScreen).is_ok());
        }

        #[test]
        fn x11_window_manager_overlay_window_id_nonzero() {
            let mut wm = X11WindowManager::new();
            let monitors = xcap::Monitor::all().expect("should enumerate monitors");
            let first = &monitors[0];
            let display_id = first
                .name()
                .unwrap_or_else(|_| first.id().unwrap().to_string());
            wm.create_overlay(&display_id)
                .expect("create_overlay should succeed");
            let id = wm.overlay_window_id().expect("should have window ID");
            assert!(id > 0, "window ID should be non-zero, got {id}");
        }
    }
}
