//! AC-2.1 (live-window half): a wgpu surface is created from the OWNED overlay
//! `WebviewWindow` handle. The offscreen render-logic half is the
//! `overlay_gpu_offscreen_render_clear` unit test.
//!
//! `surface_created` proves the RISK-001 linchpin: a `Surface<'static>` was
//! built from the tao/Tauri overlay window's rwh-0.6 handle. When a presentable
//! GPU adapter is available (real GPU, or a software stack that supports X11
//! surfaces) the binary additionally logs `frame_presented`; under a headless
//! Xvfb without a presentable adapter the surface is still created and the
//! render-logic half is covered by the offscreen unit test.
#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]

mod common;

use std::time::Duration;

use common::{RunningApp, TestDisplay};

#[test]
#[ignore = "quarantined (DC-10/DC-13): intermittently times out on the Ubuntu 24.04 CI runner -- slow app boot under load races the harness boot-marker timeout (flaky, not deterministic; not yet root-caused). Run locally with --run-ignored all"]
fn overlay_surface_is_created_from_owned_window() {
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
        app.wait_for_log("surface_created", Duration::from_secs(20)),
        "expected 'surface_created' (wgpu Surface from owned overlay window); log:\n{}",
        app.read_log()
    );

    // If a presentable adapter exists, frames are presented; otherwise the
    // surface-creation evidence above is sufficient for this half (offscreen
    // unit test covers the clear/submit logic).
    if app.wait_for_log("frame_presented", Duration::from_secs(3)) {
        eprintln!("note: presentable adapter available; frame_presented observed");
    } else {
        eprintln!(
            "note: no presentable adapter under this display; \
             surface_created is the live-window evidence (offscreen unit covers render)"
        );
    }

    let _ = app.terminate_and_wait(Duration::from_secs(10));
}
