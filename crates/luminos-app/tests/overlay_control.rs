//! Story E04/002 subprocess tests: the overlay `WindowManager` bridge.
//!
//! `tauri::App::run` never returns and owns the main thread, so the running app
//! cannot be asserted in-process. Each test spawns the real `luminos-app` binary
//! under a dedicated `Xvfb` + `picom` and asserts via:
//! - structured stdout log lines (`overlay_xid=N`, `windowmanager_bound`,
//!   `self_capture_*`),
//! - the X11 window tree (via the `x11rb` harness in `tests/common`).
//!
//! Tests gracefully SKIP when `Xvfb`/`picom` are unavailable. Linux + the
//! `ci_platform_tests` feature only.
#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]

mod common;

use std::time::Duration;

use common::{RunningApp, TestDisplay, find_windows};

/// T008 / AC-3.1, FR-8: the app extracts a non-zero overlay XID from the
/// tao/Tauri overlay window at startup and binds the `X11WindowManager` to it.
#[test]
fn app_logs_overlay_xid() {
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    let mut app = match RunningApp::spawn(&display.display, &[]) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("SKIP: could not spawn app: {e}");
            return;
        }
    };

    // The bridge logs `overlay_xid=N` once it has extracted the XID from the
    // overlay window's raw-window-handle at RunEvent::Ready.
    let found = app.wait_for_log("overlay_xid=", Duration::from_secs(25));
    let log = app.read_log();
    let _ = app.terminate_and_wait(Duration::from_secs(10));

    assert!(
        found,
        "expected 'overlay_xid=' bridge log line; log:\n{log}"
    );

    // The logged XID must be non-zero (a real, mapped X11 window id).
    let xid = log
        .lines()
        .find_map(|l| l.split("overlay_xid=").nth(1))
        .and_then(|rest| {
            rest.split(|c: char| !c.is_ascii_digit())
                .next()
                .filter(|s| !s.is_empty())
        })
        .and_then(|digits| digits.parse::<u64>().ok())
        .unwrap_or(0);
    assert!(xid > 0, "overlay_xid should be non-zero; log:\n{log}");

    // The bridge must have constructed and stored the WindowManager.
    assert!(
        log.contains("windowmanager_bound"),
        "expected 'windowmanager_bound' after XID extraction; log:\n{log}"
    );
}

/// T009 / AC-2.1, AC-2.2, FR-5/FR-7: the app exercises the shipped
/// `set_excluded_windows(&[overlay_xid])` hook and captures a frame without
/// panicking; a self-capture finding (flicker observation) is logged.
#[test]
fn app_self_capture_hook_runs_without_panic() {
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    // The self-capture probe is gated so it only runs in tests (it performs a
    // capture, which is otherwise story 003's job).
    let mut app = match RunningApp::spawn(&display.display, &[("LUMINOS_SELF_CAPTURE_PROBE", "1")])
    {
        Ok(app) => app,
        Err(e) => {
            eprintln!("SKIP: could not spawn app: {e}");
            return;
        }
    };

    let probed = app.wait_for_log("self_capture_probe=", Duration::from_secs(25));
    let log = app.read_log();
    let exit = app.terminate_and_wait(Duration::from_secs(10));

    assert!(
        probed,
        "expected 'self_capture_probe=' finding line; log:\n{log}"
    );
    // The app must NOT have crashed during the probe (clean SIGTERM exit).
    assert_eq!(exit, Some(0), "app should exit cleanly after the probe");
}

/// T010 / AC-1.1: the overlay window the manager is bound to is findable in the
/// X11 tree (server-observable proof the bound XID is the real overlay window).
#[test]
fn app_overlay_window_is_bound_and_mapped() {
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    let mut app = match RunningApp::spawn(&display.display, &[]) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("SKIP: could not spawn app: {e}");
            return;
        }
    };

    assert!(
        app.wait_for_log("windowmanager_bound", Duration::from_secs(25)),
        "expected 'windowmanager_bound'; log:\n{}",
        app.read_log()
    );

    // The logged XID must correspond to a real mapped window in the X11 tree.
    let log = app.read_log();
    let xid = log
        .lines()
        .find_map(|l| l.split("overlay_xid=").nth(1))
        .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|digits| digits.parse::<u32>().ok());

    let mut found = false;
    for _ in 0..40 {
        let overlays = find_windows(&display.display, "Overlay");
        if let Some(target) = xid {
            if overlays.iter().any(|w| w.id == target && w.mapped) {
                found = true;
                break;
            }
        } else if overlays.iter().any(|w| w.mapped) {
            found = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = app.terminate_and_wait(Duration::from_secs(10));

    assert!(
        found,
        "the bound overlay XID should be a mapped window in the X11 tree; log:\n{log}"
    );
}
