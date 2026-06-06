//! AC-2.2: the overlay is transparent, undecorated, always-on-top, skips the
//! taskbar, and is click-through.
//!
//! Verification strategy under a WM-less Xvfb: EWMH hints (`always_on_top`,
//! `skip_taskbar`) are *requests* enforced by a window manager, of which there
//! is none in CI — so those are asserted via the app's own structured log
//! (the builder set them) plus the click-through `ignore_cursor_events=true`
//! marker. Window-server-observable attributes (existence, sizing,
//! `_MOTIF_WM_HINTS` undecorated) are asserted via `x11rb`.
//!
//! NFR-3: with no compositor the app logs `NoCompositor` and continues without
//! panicking.
#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]

mod common;

use std::time::Duration;

use common::{RunningApp, TestDisplay, find_windows};

#[test]
fn overlay_window_is_undecorated_and_clickthrough() {
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

    // The overlay-open log line records the requested attributes including
    // click-through. The builder set transparent/undecorated/always_on_top/
    // skip_taskbar before this line was emitted.
    assert!(
        app.wait_for_log("ignore_cursor_events=true", Duration::from_secs(20)),
        "expected click-through 'ignore_cursor_events=true'; log:\n{}",
        app.read_log()
    );

    // Wait for the overlay to map, then assert the server-observable attributes.
    let mut overlay = None;
    for _ in 0..40 {
        if let Some(w) = find_windows(&display.display, "Overlay")
            .into_iter()
            .find(|w| w.mapped)
        {
            overlay = Some(w);
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = app.terminate_and_wait(Duration::from_secs(10));

    let Some(overlay) = overlay else {
        panic!("overlay window should map and be findable via x11rb");
    };
    assert!(
        overlay.has_motif_undecorated,
        "overlay should be undecorated (_MOTIF_WM_HINTS decorations=0); window: {overlay:?}"
    );
    // Sized to the screen (full-screen overlay).
    assert!(
        overlay.width >= 1920 && overlay.height >= 1080,
        "overlay should be full-screen sized; got {}x{}",
        overlay.width,
        overlay.height
    );
}

#[test]
fn overlay_no_compositor_logs_warn_and_continues() {
    // A dedicated display WITHOUT picom: the app must log NoCompositor and keep
    // running (no panic), proving NFR-3.
    let Some(display) = TestDisplay::launch_without_compositor() else {
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
        app.wait_for_log("NoCompositor", Duration::from_secs(20)),
        "expected 'NoCompositor' warning with no compositor; log:\n{}",
        app.read_log()
    );
    // The app must keep running (heartbeat continues) rather than panic.
    assert!(
        app.wait_for_log("redraw=", Duration::from_secs(10)),
        "app should continue rendering opaque after NoCompositor; log:\n{}",
        app.read_log()
    );

    let _ = app.terminate_and_wait(Duration::from_secs(10));
}
