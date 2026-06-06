//! AC-3.1 (wake half): a background thread holding the tao-backed `AppNotifier`
//! sets the shared dirty flag via `notify_state_changed()`, and the run loop
//! observes it and renders (no `request_redraw`, no main-thread marshaling of
//! the flag itself).
//!
//! The binary exposes an env-gated debug thread (`LUMINOS_DEBUG_NOTIFY=1`) that,
//! after an idle delay, calls `notify_state_changed()` and logs `dirty_render`.
#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]

mod common;

use std::time::Duration;

use common::{RunningApp, TestDisplay};

#[test]
fn notify_state_changed_triggers_render() {
    let Some(display) = TestDisplay::launch() else {
        return;
    };
    let mut app = match RunningApp::spawn(&display.display, &[("LUMINOS_DEBUG_NOTIFY", "1")]) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("SKIP: could not spawn app: {e}");
            return;
        }
    };

    // The debug thread fires `dirty_render` once it has set the flag through the
    // EventNotifier path; the loop renders the resulting tick.
    assert!(
        app.wait_for_log("dirty_render", Duration::from_secs(20)),
        "expected 'dirty_render' from the AppNotifier wake path; log:\n{}",
        app.read_log()
    );
    // A redraw must follow the wake (the loop observed the flag).
    assert!(
        app.wait_for_log("redraw=", Duration::from_secs(5)),
        "expected a redraw after the wake; log:\n{}",
        app.read_log()
    );

    let _ = app.terminate_and_wait(Duration::from_secs(10));
}
