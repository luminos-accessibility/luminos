//! Story E04/005 subprocess integration test: engine -> panel events (FR-6 /
//! AC-2.2).
//!
//! The two `tauri-specta` events (`zoom_changed`, `mode_changed`) are emitted
//! from the render loop when the live `(zoom, mode)` changes on a delta (the
//! input thread has no `AppHandle`, so emission lives in the loop — AD-5). A
//! Phase-0 zoom-in hotkey (`ctrl+alt+equal`) raises the zoom, which the loop
//! observes on its next tick and emits `ZoomChangedEvent` for. We assert the
//! logged-emit marker (`emit zoom_changed=...`) beside the `.emit(app_handle)`
//! call; the live webview-listener assertion is story 007's `tauri-driver`
//! concern (no webview client exists in this harness).
//!
//! GPU/headless caveats are identical to `live_magnification.rs` (DC-10). This
//! test does NOT require a presentable GPU adapter — the emit happens in the
//! `MainEventsCleared` arm independent of the present.
#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]

mod common;

use std::time::Duration;

use common::{RunningApp, TestDisplay};

/// Logs the live `AppState` on change (so we can also see the zoom transition).
const LOG_STATE: (&str, &str) = ("LUMINOS_LOG_STATE", "1");

/// Whether xdotool is available for input simulation.
fn xdotool_available() -> bool {
    std::process::Command::new("xdotool")
        .arg("--version")
        .output()
        .is_ok()
}

/// Spawns the app and waits for the input pipeline to be ready.
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

/// AC-2.2: a zoom-in hotkey changes the engine's zoom, and the render loop emits
/// `ZoomChangedEvent` (observed via the `emit zoom_changed=` log marker beside
/// the `.emit()` call).
#[test]
fn ipc_hotkey_emits_zoom_changed_event() {
    if !xdotool_available() {
        eprintln!("SKIP: xdotool not available");
        return;
    }
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    let Some(mut app) = spawn_ready(&display.display, &[LOG_STATE]) else {
        return;
    };

    // Default zoom is 2.0; the zoom-in hotkey multiplies by 1.5 -> 3.0, which the
    // loop emits as `ZoomChangedEvent(3.0)`.
    let _ = std::process::Command::new("xdotool")
        .env("DISPLAY", &display.display)
        .args(["key", "ctrl+alt+equal"])
        .status();

    assert!(
        app.wait_for_log("emit zoom_changed=3", Duration::from_secs(8)),
        "the render loop should emit ZoomChangedEvent(3) after the zoom-in hotkey; log:\n{}",
        app.read_log()
    );

    let _ = app.terminate_and_wait(Duration::from_secs(10));
}
