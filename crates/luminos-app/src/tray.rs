//! System tray icon + menu (story E04/007, D6).
//!
//! Adds a `StatusNotifierItem` (SNI) tray icon with a Show/Hide and Quit menu so
//! Luminos can run unobtrusively (minimize-to-tray lives in `app::run`'s
//! `.on_window_event`). On Linux the tray relies on an SNI host
//! (libayatana-appindicator); not every desktop environment runs one (and a
//! headless Xvfb never does). The KEY requirement (FR-3) is **graceful
//! degrade**: where no SNI host exists the app logs a `warn!`, keeps the
//! control-panel window visible, and NEVER panics — `init_tray` returns
//! `Ok(None)` rather than aborting startup.
//!
//! FR-1 INVARIANT (RISK-001): the tray runs entirely inside the single
//! `tauri::App::run` loop (no separate event loop). Quit routes through
//! `AppHandle::exit(0)` so the existing `ExitRequested`/`Exit` teardown
//! (thread join + GPU drop) runs exactly once.

use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Manager, Wry};

use crate::app_error::AppError;

/// Menu item id for the Show/Hide control-panel toggle.
pub(crate) const MENU_ID_TOGGLE: &str = "toggle";
/// Menu item id for the Quit action.
pub(crate) const MENU_ID_QUIT: &str = "quit";

/// Window label of the control panel (the overlay must NEVER be hidden by the
/// tray — hiding it kills magnification).
const CONTROL_PANEL_LABEL: &str = "control-panel";

/// Structured marker logged when the tray icon was created successfully.
pub(crate) const TRAY_READY_MARKER: &str = "tray=ready";
/// Structured marker logged when the tray degraded (no SNI host / build error).
pub(crate) const TRAY_DEGRADED_MARKER: &str = "tray=degraded";

/// Initializes the system tray, returning the live [`TrayIcon`] on success or
/// `None` when no SNI host is available (graceful degrade, FR-3).
///
/// The returned icon MUST be stashed by the caller: `TrayIcon` is reference
/// counted and dropping it removes the icon from the tray. The caller stores it
/// on [`crate::handle::LuminosHandle`] (Linux) so it outlives `setup`.
///
/// # Errors
///
/// This function does NOT propagate a tray-build failure as an error: an absent
/// SNI host is a normal runtime condition, not a startup failure (FR-3). It
/// returns `Ok(None)` and logs in that case. The [`Result`] signature is
/// retained so a future hard-failure mode (e.g. a malformed bundled icon) can
/// surface without changing the call site.
//
// `unnecessary_wraps`: every path today returns `Ok` by design — FR-3 forbids
// aborting startup, so no failure mode currently propagates. The `Result` is
// deliberate forward-looking API surface (see doc comment) and lets the
// `AppError::Tauri` conversions stay at the seam; allow the lint here.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn init_tray(app: &tauri::App) -> Result<Option<TrayIcon<Wry>>, AppError> {
    // Heuristic pre-check (FR-3): an SNI host is reachable over the session
    // D-Bus. With no session bus there is provably no host, so skip the build
    // and degrade deterministically rather than relying on the platform's
    // Ok-on-no-host behavior. This makes the degrade path testable headless.
    if !session_bus_available() {
        log::warn!(
            "{TRAY_DEGRADED_MARKER}: no session D-Bus ($DBUS_SESSION_BUS_ADDRESS unset); \
             tray unavailable, keeping control panel visible"
        );
        return Ok(None);
    }

    let menu = match build_menu(app) {
        Ok(menu) => menu,
        Err(e) => {
            // A menu-build failure is non-fatal: degrade rather than abort.
            log::warn!(
                "{TRAY_DEGRADED_MARKER}: tray menu build failed: '{e}'; keeping panel visible"
            );
            return Ok(None);
        }
    };

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Luminos")
        .on_menu_event(handle_menu_event);
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    } else {
        log::warn!("no default window icon available; tray icon will use the platform default");
    }

    match builder.build(app) {
        Ok(tray) => {
            log::info!("{TRAY_READY_MARKER}: system tray icon created (Show/Hide + Quit menu)");
            Ok(Some(tray))
        }
        Err(e) => {
            // Even with a session bus the build can fail when no SNI *host*
            // listens (e.g. a bare Xvfb session). Degrade, never `?`-propagate.
            log::warn!(
                "{TRAY_DEGRADED_MARKER}: tray build failed: '{e}'; keeping control panel visible"
            );
            Ok(None)
        }
    }
}

/// Builds the tray context menu (Show/Hide Panel, separator, Quit).
fn build_menu(app: &tauri::App) -> Result<Menu<Wry>, AppError> {
    let toggle = MenuItem::with_id(app, MENU_ID_TOGGLE, "Show/Hide Panel", true, None::<&str>)
        .map_err(AppError::Tauri)?;
    let separator = PredefinedMenuItem::separator(app).map_err(AppError::Tauri)?;
    let quit = PredefinedMenuItem::quit(app, Some("Quit Luminos")).map_err(AppError::Tauri)?;
    Menu::with_items(app, &[&toggle, &separator, &quit]).map_err(AppError::Tauri)
}

/// Routes a tray menu event to its action.
///
/// Closures registered on the tray are `Send + Sync + 'static`, so they reach
/// app state through the `AppHandle` (never a borrow). The predefined Quit item
/// already calls `app.exit(0)` internally, so only the custom toggle id is
/// handled here; the explicit `MENU_ID_QUIT` arm is retained for the case where
/// a non-predefined quit item is wired.
//
// `needless_pass_by_value`: the `MenuEvent` is taken by value to match Tauri's
// `on_menu_event` closure signature `Fn(&AppHandle<R>, MenuEvent)`; it cannot be
// a reference here.
#[allow(clippy::needless_pass_by_value)]
fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        MENU_ID_TOGGLE => toggle_control_panel(app),
        MENU_ID_QUIT => {
            log::info!("tray=quit requested; exiting");
            app.exit(0);
        }
        other => log::debug!("tray menu event ignored: id='{other}'"),
    }
}

/// Shows or hides the control-panel window from the tray (FR-2 restore path).
///
/// Hides ONLY the control panel — NEVER the overlay (hiding the overlay kills
/// magnification). All window calls are logged-and-continued: a failure here is
/// non-fatal and must not panic.
pub(crate) fn toggle_control_panel(app: &AppHandle) {
    let Some(window) = app.get_webview_window(CONTROL_PANEL_LABEL) else {
        log::warn!("tray toggle: control-panel window '{CONTROL_PANEL_LABEL}' not found");
        return;
    };

    match window.is_visible() {
        Ok(true) => {
            if let Err(e) = window.hide() {
                log::warn!("tray toggle: hide failed: '{e}'");
            } else {
                log::info!("tray=hide control-panel");
            }
        }
        Ok(false) => show_and_focus(&window),
        Err(e) => {
            // Visibility query failed; default to showing so the user is never
            // stuck with an unreachable panel.
            log::warn!("tray toggle: is_visible failed: '{e}'; showing panel");
            show_and_focus(&window);
        }
    }
}

/// Shows the control panel and gives it focus, logging any non-fatal failure.
fn show_and_focus(window: &tauri::WebviewWindow) {
    if let Err(e) = window.show() {
        log::warn!("tray toggle: show failed: '{e}'");
        return;
    }
    if let Err(e) = window.set_focus() {
        log::warn!("tray toggle: set_focus failed: '{e}'");
    }
    log::info!("tray=show control-panel");
}

/// Whether a session D-Bus is reachable (`$DBUS_SESSION_BUS_ADDRESS` set).
///
/// An SNI tray host is reached over the session bus, so with no session bus
/// there is provably no host — the deterministic degrade signal (FR-3).
fn session_bus_available() -> bool {
    std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some_and(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_menu_ids_are_stable() {
        // The wire ids the menu-event handler matches on must not drift (the
        // subprocess tray test and the menu wiring both depend on them).
        assert_eq!(MENU_ID_TOGGLE, "toggle");
        assert_eq!(MENU_ID_QUIT, "quit");
    }

    #[test]
    fn tray_markers_are_distinct() {
        // The degrade subprocess test (`tray_absent_host_degrades`) keys on
        // these markers; ready and degraded must be unambiguous.
        assert_ne!(TRAY_READY_MARKER, TRAY_DEGRADED_MARKER);
        assert!(TRAY_READY_MARKER.starts_with("tray="));
        assert!(TRAY_DEGRADED_MARKER.starts_with("tray="));
    }

    #[test]
    fn tray_session_bus_detected_when_address_set() {
        // SAFETY: single-threaded test; restore the prior value after.
        let prior = std::env::var_os("DBUS_SESSION_BUS_ADDRESS");
        unsafe {
            std::env::set_var("DBUS_SESSION_BUS_ADDRESS", "unix:path=/tmp/test-bus");
        }
        assert!(
            session_bus_available(),
            "a set, non-empty session bus address must be detected"
        );
        unsafe {
            match prior {
                Some(v) => std::env::set_var("DBUS_SESSION_BUS_ADDRESS", v),
                None => std::env::remove_var("DBUS_SESSION_BUS_ADDRESS"),
            }
        }
    }

    #[test]
    fn tray_session_bus_absent_when_address_unset() {
        // SAFETY: single-threaded test; restore the prior value after.
        let prior = std::env::var_os("DBUS_SESSION_BUS_ADDRESS");
        unsafe {
            std::env::remove_var("DBUS_SESSION_BUS_ADDRESS");
        }
        assert!(
            !session_bus_available(),
            "an unset session bus address means no SNI host (degrade path)"
        );
        // An empty value is also "no bus".
        unsafe {
            std::env::set_var("DBUS_SESSION_BUS_ADDRESS", "");
        }
        assert!(
            !session_bus_available(),
            "an empty session bus address must be treated as no bus"
        );
        unsafe {
            match prior {
                Some(v) => std::env::set_var("DBUS_SESSION_BUS_ADDRESS", v),
                None => std::env::remove_var("DBUS_SESSION_BUS_ADDRESS"),
            }
        }
    }
}
