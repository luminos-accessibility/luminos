//! Bridges the tao/Tauri overlay window to the platform `WindowManager`.
//!
//! `luminos-platform` must not depend on `tauri` (correct dependency
//! direction), so the Tauri → X11-window-id extraction lives here in
//! `luminos-app`. The overlay `WebviewWindow` already implements
//! `raw_window_handle::HasWindowHandle`; on X11 (tao/GTK3) it yields a
//! `RawWindowHandle::Xlib { window: <XID> }` (or `Xcb`). We extract that XID and
//! construct an [`X11WindowManager`](luminos_platform::linux_x11::X11WindowManager)
//! bound to it. The manager then drives geometry/visibility/stacking via raw
//! `x11rb` requests (no winit, no second event loop).
//!
//! This INVERTS the literal FR-8 prescription (`gtk_window()` → gdk →
//! `gdk_x11_window_get_xid`): the raw-window-handle path is main-thread-safe,
//! involves no extra unsafe GTK FFI, and reuses the same rwh-0.6 handle wgpu
//! already consumes for the surface. See SUBTASKS Deviations.

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tauri::WebviewWindow;

use luminos_platform::linux_x11::X11WindowManager;
use luminos_platform::traits::ScreenRect;

use crate::app_error::AppError;

/// Extracts the overlay window's X11 window id via `raw_window_handle`.
///
/// # Errors
///
/// Returns [`AppError::Bridge`] if the window handle cannot be obtained or the
/// handle is not an X11 (`Xlib`/`Xcb`) handle (e.g. on a non-X11 backend).
pub(crate) fn extract_overlay_xid(window: &WebviewWindow) -> Result<u32, AppError> {
    let handle = window
        .window_handle()
        .map_err(|e| AppError::Bridge(format!("overlay window_handle() failed: {e}")))?;

    match handle.as_raw() {
        // `Xlib.window` is `c_ulong`; truncating to the 32-bit X11 resource id
        // is correct (XIDs are 32-bit on the wire).
        #[allow(clippy::cast_possible_truncation)]
        RawWindowHandle::Xlib(xlib) => Ok(xlib.window as u32),
        RawWindowHandle::Xcb(xcb) => Ok(xcb.window.get()),
        other => Err(AppError::Bridge(format!(
            "overlay handle is not an X11 handle (got {other:?}); \
             the x11rb WindowManager backend requires X11"
        ))),
    }
}

/// Extracts the overlay XID and constructs the bound [`X11WindowManager`].
///
/// `display_bounds` is the overlay's target display rectangle (resolved by the
/// caller from the primary monitor). The manager performs no window creation —
/// it only binds to the already-open overlay window.
///
/// # Errors
///
/// Returns [`AppError::Bridge`] if the XID cannot be extracted, or maps the
/// platform [`WindowError`](luminos_platform::traits::WindowError) into
/// [`AppError::Bridge`] if the X server connection cannot be opened.
pub(crate) fn build_window_manager(
    window: &WebviewWindow,
    display_bounds: ScreenRect,
) -> Result<X11WindowManager, AppError> {
    let xid = extract_overlay_xid(window)?;
    log::info!("overlay_xid={xid} (extracted from overlay raw-window-handle)");

    let manager = X11WindowManager::new(xid, display_bounds)
        .map_err(|e| AppError::Bridge(format!("X11WindowManager::new failed: {e}")))?;
    log::info!(
        "windowmanager_bound: X11WindowManager bound to overlay xid '{xid}' \
         (display {}x{} at {},{})",
        display_bounds.width,
        display_bounds.height,
        display_bounds.x,
        display_bounds.y
    );
    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The XID extraction needs a realized X11 window, which is only available
    // in the subprocess integration tests (`tests/overlay_control.rs`). Here we
    // assert the pure-logic seam: a Bridge error renders with the expected text.
    #[test]
    fn overlay_bridge_error_display() {
        let err = AppError::Bridge("no handle".to_string());
        assert!(
            err.to_string().contains("overlay bridge error"),
            "unexpected display: '{err}'"
        );
        assert!(err.to_string().contains("no handle"));
    }
}
