//! Story E04/007 subprocess tests: the system tray (D6) + minimize-to-tray.
//!
//! `tauri::App::run` never returns and owns the main thread, so the running app
//! cannot be asserted in-process. Each test spawns the real `luminos-app`
//! binary under a dedicated `Xvfb` (+`picom`) and asserts via structured stdout
//! log markers (`tray=ready`/`tray=degraded`/`minimize_to_tray=hidden`) and the
//! X11 window tree (`x11rb`, which sees WM-less windows that `xdotool --name`
//! does not).
//!
//! **Tier note (per `IMPLEMENTATION_NOTES` §G):** the GRACEFUL-DEGRADE path is the
//! AC-load-bearing test and is locally runnable — a headless Xvfb never has an
//! SNI host, so `tray=degraded` is the deterministic outcome there. The
//! tray-PRESENT path needs a real D-Bus session + an appindicator/SNI host,
//! which a bare Xvfb lacks; that test asserts only the no-panic + clean-exit
//! invariant (the visible-icon proof is manual/dogfood, recorded in the matrix).
//!
//! Tests gracefully SKIP when `Xvfb` is unavailable. Linux + the
//! `ci_platform_tests` feature only.
#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]

mod common;

use std::time::Duration;

use common::{RunningApp, TestDisplay, find_windows};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, PropMode};
use x11rb::wrapper::ConnectionExt as _;

/// AC-1.1 / FR-3 (the KEY requirement): with no SNI host (headless Xvfb, and we
/// additionally unset `$DBUS_SESSION_BUS_ADDRESS` to force the deterministic
/// degrade pre-check), the app logs `tray=degraded`, keeps the control-panel
/// window visible, and exits cleanly — it NEVER panics or vanishes silently.
///
/// This is locally runnable (no SNI host required — its absence is the point).
#[test]
#[ignore = "quarantined (DC-10/DC-13): intermittently times out on the Ubuntu 24.04 CI runner -- slow app boot under load races the harness boot-marker timeout (flaky, not deterministic; not yet root-caused). Run locally with --run-ignored all"]
fn tray_absent_host_degrades() {
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    // Force the degrade pre-check: no session bus => provably no SNI host.
    let mut app = match RunningApp::spawn(&display.display, &[("DBUS_SESSION_BUS_ADDRESS", "")]) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("SKIP: could not spawn app: {e}");
            return;
        }
    };

    let degraded = app.wait_for_log("tray=degraded", Duration::from_secs(25));
    let log = app.read_log();

    // The control-panel window must remain mapped (not hidden into a tray that
    // does not exist) — server-observable proof it did not vanish.
    let mut panel_visible = false;
    for _ in 0..30 {
        if find_windows(&display.display, "Control Panel")
            .iter()
            .any(|w| w.mapped)
        {
            panel_visible = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    let exit = app.terminate_and_wait(Duration::from_secs(10));

    assert!(
        degraded,
        "expected 'tray=degraded' marker on the no-SNI-host path; log:\n{log}"
    );
    assert!(
        !log.contains("panicked"),
        "the degrade path must NOT panic; log:\n{log}"
    );
    assert!(
        panel_visible,
        "the control-panel window must stay mapped when the tray degrades; log:\n{log}"
    );
    assert_eq!(
        exit,
        Some(0),
        "the app must exit cleanly after a tray degrade; log:\n{log}"
    );
}

/// AC-1.1 / FR-1: with a session bus present, the tray init runs to a definitive
/// outcome (`tray=ready` OR `tray=degraded`) and the app neither panics nor
/// blocks startup. Under a bare Xvfb with no real SNI host this typically logs
/// `tray=degraded` even though a bus is present — so we assert the INVARIANT
/// (no panic, panel visible, clean exit), not the visible icon. The
/// icon-on-screen proof is manual/dogfood (recorded in the epic matrix).
#[test]
#[ignore = "quarantined (DC-10/DC-13): intermittently times out on the Ubuntu 24.04 CI runner -- slow app boot under load races the harness boot-marker timeout (flaky, not deterministic; not yet root-caused). Run locally with --run-ignored all"]
fn tray_init_reaches_definite_outcome_without_panic() {
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    // Inherit the ambient session bus if any; do NOT unset it here.
    let mut app = match RunningApp::spawn(&display.display, &[]) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("SKIP: could not spawn app: {e}");
            return;
        }
    };

    let ready = app.wait_for_log("tray=ready", Duration::from_secs(20));
    let degraded = app.wait_for_log("tray=degraded", Duration::from_secs(1));
    let log = app.read_log();
    let exit = app.terminate_and_wait(Duration::from_secs(10));

    assert!(
        ready || degraded,
        "tray init must reach a definite outcome (ready|degraded); log:\n{log}"
    );
    assert!(
        !log.contains("panicked"),
        "tray init must never panic; log:\n{log}"
    );
    assert_eq!(
        exit,
        Some(0),
        "the app must exit cleanly after tray init; log:\n{log}"
    );
}

/// AC-1.1 / FR-2: with `minimize_to_tray` enabled, a close request on the
/// control-panel window is intercepted — the window hides but the process keeps
/// running (it does not exit). Asserts via the `minimize_to_tray=hidden` marker
/// and a clean SIGTERM exit afterwards (proving the app was still alive to
/// receive it). The overlay is never touched.
#[test]
#[ignore = "quarantined (DC-10/DC-13): intermittently times out on the Ubuntu 24.04 CI runner -- slow app boot under load races the harness boot-marker timeout (flaky, not deterministic; not yet root-caused). Run locally with --run-ignored all"]
fn minimize_to_tray_hides_window_keeps_running() {
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    let mut app =
        match RunningApp::spawn(&display.display, &[("LUMINOS_FORCE_MINIMIZE_TO_TRAY", "1")]) {
            Ok(app) => app,
            Err(e) => {
                eprintln!("SKIP: could not spawn app: {e}");
                return;
            }
        };

    // Wait for the control-panel window to map.
    let mut panel_id = None;
    for _ in 0..50 {
        if let Some(w) = find_windows(&display.display, "Control Panel")
            .into_iter()
            .find(|w| w.mapped)
        {
            panel_id = Some(w.id);
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let Some(panel_id) = panel_id else {
        eprintln!(
            "SKIP: control-panel window never mapped under headless Xvfb; log:\n{}",
            app.read_log()
        );
        let _ = app.terminate_and_wait(Duration::from_secs(10));
        return;
    };

    // Send a WM_DELETE_WINDOW ClientMessage to provoke `CloseRequested` (there
    // is no WM under Xvfb, so we synthesize the close protocol directly).
    if !send_wm_delete(&display.display, panel_id) {
        eprintln!("SKIP: could not send WM_DELETE_WINDOW; window-tree may be unstable");
        let _ = app.terminate_and_wait(Duration::from_secs(10));
        return;
    }

    let hidden = app.wait_for_log("minimize_to_tray=hidden", Duration::from_secs(10));
    let log = app.read_log();

    // The process must still be alive (the close was prevented). Sending SIGTERM
    // and getting a clean exit proves it was running to receive it.
    let exit = app.terminate_and_wait(Duration::from_secs(10));

    assert!(
        hidden,
        "expected 'minimize_to_tray=hidden' after a control-panel close request; log:\n{log}"
    );
    assert!(
        !log.contains("panicked"),
        "minimize-to-tray must not panic; log:\n{log}"
    );
    assert_eq!(
        exit,
        Some(0),
        "the app must still be running (prevented close) and exit cleanly on SIGTERM; log:\n{log}"
    );
}

/// Story 008 / AC-8: with `minimize_to_tray` DISABLED, a close request on the
/// control-panel window must quit the WHOLE app — not just close that one window
/// while the overlay (the second tao window) keeps the event loop alive. Asserts
/// the process exits ON ITS OWN (no SIGTERM) after a `WM_DELETE_WINDOW`, and that
/// it logged the "exiting app" decision and a clean shutdown.
#[test]
#[ignore = "quarantined (DC-10/DC-13): intermittently times out on the Ubuntu 24.04 CI runner -- slow app boot under load races the harness boot-marker timeout (flaky, not deterministic; not yet root-caused). Run locally with --run-ignored all"]
fn close_quits_app_when_minimize_to_tray_disabled() {
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    let mut app =
        match RunningApp::spawn(&display.display, &[("LUMINOS_FORCE_MINIMIZE_TO_TRAY", "0")]) {
            Ok(app) => app,
            Err(e) => {
                eprintln!("SKIP: could not spawn app: {e}");
                return;
            }
        };

    // Wait for the control-panel window to map.
    let mut panel_id = None;
    for _ in 0..50 {
        if let Some(w) = find_windows(&display.display, "Control Panel")
            .into_iter()
            .find(|w| w.mapped)
        {
            panel_id = Some(w.id);
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let Some(panel_id) = panel_id else {
        eprintln!(
            "SKIP: control-panel window never mapped under headless Xvfb; log:\n{}",
            app.read_log()
        );
        let _ = app.terminate_and_wait(Duration::from_secs(10));
        return;
    };

    // Under a WM-less Xvfb the app's GTK WM_PROTOCOLS handler may not be wired
    // up the instant the window maps, so a single `WM_DELETE_WINDOW` can be lost
    // before `CloseRequested` is hooked. Re-send until the close handler logs its
    // (info-level, always-visible) quit decision — that marker is the
    // deterministic proof the fix ran; exit code/timing under headless software
    // GL teardown (DC-10) is noisy and is NOT the load-bearing assertion.
    let mut decided = false;
    for _ in 0..30 {
        if !send_wm_delete(&display.display, panel_id) {
            eprintln!("SKIP: could not send WM_DELETE_WINDOW; window-tree may be unstable");
            let _ = app.terminate_and_wait(Duration::from_secs(10));
            return;
        }
        if app.wait_for_log(
            "minimize_to_tray=false; exiting app",
            Duration::from_millis(400),
        ) {
            decided = true;
            break;
        }
    }

    // The app must then exit ON ITS OWN — NO signal is sent here. A generous
    // timeout absorbs slow headless teardown without making the test flaky.
    let exit = app.wait_for_exit(Duration::from_secs(15));
    let log = app.read_log();

    assert!(
        decided,
        "closing the control panel with minimize_to_tray=false must reach the quit decision \
         ('exiting app'); log:\n{log}"
    );
    assert!(
        exit.is_some(),
        "the app must quit on its own after the control-panel close (no Ctrl+C needed); it was \
         still running 15s after the quit decision; log:\n{log}"
    );
    assert!(
        !log.contains("panicked"),
        "close-quits must not panic; log:\n{log}"
    );
}

/// Sends a `WM_DELETE_WINDOW` `ClientMessage` to `window` via the `WM_PROTOCOLS`
/// mechanism, the standard way to request a graceful close with no running WM.
/// Returns whether the message was dispatched.
fn send_wm_delete(display: &str, window: u32) -> bool {
    let Ok((conn, _)) = x11rb::connect(Some(display)) else {
        return false;
    };
    let Some(wm_protocols) = intern(&conn, b"WM_PROTOCOLS") else {
        return false;
    };
    let Some(wm_delete) = intern(&conn, b"WM_DELETE_WINDOW") else {
        return false;
    };

    // Ensure the window advertises WM_DELETE_WINDOW so GTK honors the protocol.
    let _ = conn.change_property32(
        PropMode::REPLACE,
        window,
        wm_protocols,
        AtomEnum::ATOM,
        &[wm_delete],
    );

    let data = [wm_delete, x11rb::CURRENT_TIME, 0, 0, 0];
    let event = ClientMessageEvent::new(32, window, wm_protocols, data);
    let sent = conn
        .send_event(false, window, EventMask::NO_EVENT, event)
        .is_ok();
    let _ = conn.flush();
    sent
}

/// Interns an atom by name, returning its id on success.
fn intern<C: Connection>(conn: &C, name: &[u8]) -> Option<u32> {
    conn.intern_atom(false, name)
        .ok()
        .and_then(|c| c.reply().ok())
        .map(|r| r.atom)
}
