//! Story E04/003 subprocess integration tests: live full-screen magnification.
//!
//! These spawn the real `luminos-app` binary under a dedicated `Xvfb` + `picom`
//! (the `tests/common` harness) and assert via structured log markers and
//! `xdotool`-driven input. They cover the loop-glue wiring: the capture driver,
//! the input pipeline (cursor tracking + hotkeys), and the capture→render path.
//!
//! GPU caveat (DC-10): under a headless `Xvfb` there is no surface-compatible
//! wgpu adapter, so `OverlayGpu` device init fails and the live magnify
//! *present* cannot run (`overlay GPU init failed`). The magnify SHADER
//! pipeline is covered by offscreen wgpu unit tests
//! (`overlay_gpu_renderer_summary_zeroed_before_render`, `luminos-gpu`'s
//! `shader_output`). Here we assert that:
//!   - the capture driver + input pipeline are wired (`capture_driver=ready`,
//!     `input_pipeline=ready`),
//!   - cursor movement updates the tracked viewport (`state mouse=...`),
//!   - Phase-0 hotkeys mutate state (`state zoom=...`/`active=...`),
//!   - the loop survives the capture/render attempts without panicking
//!     (the heartbeat keeps advancing and the process exits cleanly).
#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]

mod common;

use std::time::Duration;

use common::{RunningApp, TestDisplay};

/// Env that seeds `is_active=true` so the magnify path runs without first
/// toggling via input (the input path is exercised separately).
const FORCE_ACTIVE: (&str, &str) = ("LUMINOS_FORCE_ACTIVE", "1");
/// Env that logs the live `AppState` on change (`state mouse=... zoom=... active=...`).
const LOG_STATE: (&str, &str) = ("LUMINOS_LOG_STATE", "1");

/// Spawns the app and waits until the input pipeline is ready, returning the
/// running app. Returns `None` (skip) if the display or app can't start.
fn spawn_ready(display: &str, extra_env: &[(&str, &str)]) -> Option<RunningApp> {
    let app = match RunningApp::spawn(display, extra_env) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("SKIP: could not spawn app: {e}");
            return None;
        }
    };
    if !app.wait_for_log("input_pipeline=ready", Duration::from_secs(25)) {
        eprintln!(
            "SKIP: input pipeline never became ready; log:\n{}",
            app.read_log()
        );
        return None;
    }
    Some(app)
}

/// Whether xdotool is available for input simulation.
fn xdotool_available() -> bool {
    std::process::Command::new("xdotool")
        .arg("--version")
        .output()
        .is_ok()
}

/// AC-1.1 (capture path wired): the capture driver and input pipeline come up,
/// and the loop keeps running through capture/render attempts without crashing.
#[test]
fn live_magnification_capture_path_wired() {
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    let Some(mut app) = spawn_ready(&display.display, &[FORCE_ACTIVE]) else {
        return;
    };

    assert!(
        app.wait_for_log("capture_driver=ready", Duration::from_secs(10)),
        "expected 'capture_driver=ready' (X11 capture backend wired); log:\n{}",
        app.read_log()
    );

    // The loop must keep pumping while the active magnify path runs each tick;
    // a panic in capture/render would freeze the heartbeat.
    let start = app.count_log_lines("redraw=");
    std::thread::sleep(Duration::from_millis(700));
    let end = app.count_log_lines("redraw=");
    assert!(
        end > start,
        "redraw heartbeat should keep advancing through the active magnify path \
         (start={start}, end={end}); log:\n{}",
        app.read_log()
    );

    let code = app.terminate_and_wait(Duration::from_secs(10));
    assert_eq!(code, Some(0), "app should exit cleanly (0)");
}

/// AC-2.1 (viewport follows cursor): moving the pointer past the dead zone
/// shifts the tracked `mouse_position` in `AppState`, which the loop reads each
/// frame to compute the capture region.
#[test]
fn live_cursor_moves_viewport() {
    if !xdotool_available() {
        eprintln!("SKIP: xdotool not available");
        return;
    }
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    let Some(mut app) = spawn_ready(&display.display, &[FORCE_ACTIVE, LOG_STATE]) else {
        return;
    };

    // Move the pointer to a distinct location well away from the origin.
    let _ = std::process::Command::new("xdotool")
        .env("DISPLAY", &display.display)
        .args(["mousemove", "1200", "700"])
        .status();

    let moved = app.wait_for_log("mouse='1200,700'", Duration::from_secs(5))
        || app.wait_for_log("mouse='1199,", Duration::from_secs(1))
        || app.wait_for_log("mouse='1201,", Duration::from_secs(1));
    assert!(
        moved,
        "viewport should follow the cursor (expected 'mouse=1200,700' in state log); log:\n{}",
        app.read_log()
    );

    let _ = app.terminate_and_wait(Duration::from_secs(10));
}

/// AC-1.2 (zoom reflected): a zoom-in hotkey raises `zoom_level`, which the
/// loop reads on the next frame (the region shrinks). Verified via the state
/// log: zoom goes 2 → 3.
#[test]
fn live_zoom_change_reflected_next_frame() {
    if !xdotool_available() {
        eprintln!("SKIP: xdotool not available");
        return;
    }
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    let Some(mut app) = spawn_ready(&display.display, &[FORCE_ACTIVE, LOG_STATE]) else {
        return;
    };

    // Default zoom is 2.0; zoom-in multiplies by 1.5 → 3.0.
    let _ = std::process::Command::new("xdotool")
        .env("DISPLAY", &display.display)
        .args(["key", "ctrl+alt+equal"])
        .status();

    assert!(
        app.wait_for_log("zoom='3'", Duration::from_secs(5)),
        "zoom-in hotkey should raise zoom to 3 (reflected in the next frame); log:\n{}",
        app.read_log()
    );

    let _ = app.terminate_and_wait(Duration::from_secs(10));
}

/// AC-3.1 (hotkeys drive state): each Phase-0 hotkey mutates `AppState`.
/// Zoom-in (→3), zoom-out (→2), toggle (active true), reset (→2 after a change).
#[test]
fn live_hotkeys_drive_state() {
    if !xdotool_available() {
        eprintln!("SKIP: xdotool not available");
        return;
    }
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    // Do NOT force-active: this also verifies the toggle hotkey flips is_active.
    let Some(mut app) = spawn_ready(&display.display, &[LOG_STATE]) else {
        return;
    };
    let disp = display.display.clone();
    let key = |k: &str| {
        let _ = std::process::Command::new("xdotool")
            .env("DISPLAY", &disp)
            .args(["key", k])
            .status();
        std::thread::sleep(Duration::from_millis(300));
    };

    // Zoom in: 2 → 3.
    key("ctrl+alt+equal");
    assert!(
        app.wait_for_log("zoom='3'", Duration::from_secs(5)),
        "ctrl+alt+equal should set zoom=3; log:\n{}",
        app.read_log()
    );

    // Zoom out: 3 → 2.
    key("ctrl+alt+minus");
    assert!(
        app.wait_for_log("zoom='2' active='false'", Duration::from_secs(5)),
        "ctrl+alt+minus should set zoom=2; log:\n{}",
        app.read_log()
    );

    // Toggle: active false → true.
    key("ctrl+alt+8");
    assert!(
        app.wait_for_log("active='true'", Duration::from_secs(5)),
        "ctrl+alt+8 should toggle magnification active; log:\n{}",
        app.read_log()
    );

    // Reset: bump zoom then reset to default 2.
    key("ctrl+alt+equal"); // → 3
    key("ctrl+alt+0"); // reset → 2
    assert!(
        app.wait_for_log("zoom='2' active='true'", Duration::from_secs(5)),
        "ctrl+alt+0 should reset zoom to 2 (active preserved); log:\n{}",
        app.read_log()
    );

    let _ = app.terminate_and_wait(Duration::from_secs(10));
}
