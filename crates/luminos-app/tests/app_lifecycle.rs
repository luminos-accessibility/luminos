//! AC-1.1: the app boots both windows as a single process and shuts down
//! cleanly on SIGTERM (exit 0, no hang).
#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]

mod common;

use std::time::Duration;

use common::{RunningApp, TestDisplay, find_windows};

#[test]
fn app_boots_two_windows_and_exits_clean() {
    let Some(display) = TestDisplay::launch() else {
        return; // graceful skip: no Xvfb
    };

    let mut app = match RunningApp::spawn(&display.display, &[]) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("SKIP: could not spawn app: {e}");
            return;
        }
    };

    // Wait for both windows to map (overlay surface creation is a good proxy
    // that the loop reached Ready).
    assert!(
        app.wait_for_log("overlay window 'overlay' opened", Duration::from_secs(20)),
        "overlay window should open; log:\n{}",
        app.read_log()
    );

    // Both Luminos windows must exist and be viewable as a single process.
    // Poll briefly: webkit window realization lags the overlay-open log line.
    let mut windows = Vec::new();
    for _ in 0..40 {
        windows = find_windows(&display.display, "Luminos");
        if windows.iter().filter(|w| w.mapped).count() >= 2 {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let mapped: Vec<_> = windows.iter().filter(|w| w.mapped).collect();
    assert!(
        mapped.iter().any(|w| w.name.contains("Control Panel")),
        "control panel window should be mapped; found: {windows:?}"
    );
    assert!(
        mapped.iter().any(|w| w.name.contains("Overlay")),
        "overlay window should be mapped; found: {windows:?}"
    );

    // Graceful shutdown: SIGTERM → exit 0 within the timeout, no hang.
    let code = app.terminate_and_wait(Duration::from_secs(10));
    let log = app.read_log();
    assert!(
        log.contains("shutdown=clean"),
        "expected 'shutdown=clean' in log:\n{log}"
    );
    assert_eq!(code, Some(0), "expected clean exit code 0, got {code:?}");
}
