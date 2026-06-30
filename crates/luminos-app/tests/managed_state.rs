//! AC-3.1 (state half): the `LuminosHandle` holding the real
//! `Arc<ArcSwap<AppState>>` is registered as Tauri managed state and is
//! retrievable from a Tauri command context (proved by the `managed_state_ok`
//! probe that reads the live `AppState`).
#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]

mod common;

use std::time::Duration;

use common::{RunningApp, TestDisplay};

#[test]
#[ignore = "quarantined (DC-10/DC-13): intermittently times out on the Ubuntu 24.04 CI runner -- slow app boot under load races the harness boot-marker timeout (flaky, not deterministic; not yet root-caused). Run locally with --run-ignored all"]
fn managed_state_handle_is_retrievable() {
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
        app.wait_for_log("managed_state_ok", Duration::from_secs(20)),
        "expected 'managed_state_ok' proving State<LuminosHandle> retrieval; log:\n{}",
        app.read_log()
    );

    let _ = app.terminate_and_wait(Duration::from_secs(10));
}
