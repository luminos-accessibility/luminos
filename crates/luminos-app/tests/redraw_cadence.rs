//! AC-2.3: the redraw cadence advances by at least the threshold over a fixed
//! wall-clock window, driven by the ~60 Hz timer thread (the spike's chosen
//! mechanism; tao's GTK3 backend does not emit `MainEventsCleared` steadily on
//! its own — tao #635).
#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]

mod common;

use std::time::{Duration, Instant};

use common::{RunningApp, TestDisplay};

/// Minimum redraws required over the measurement window (>= 30 per second).
const MIN_REDRAWS: usize = 30;

/// Heartbeats observed before opening the measurement window, so the 1.0s
/// sample reflects steady-state cadence rather than the GTK/webkit warmup
/// transient (the first ~second after the first heartbeat can be sluggish
/// while the webview finishes initializing).
const WARMUP_REDRAWS: usize = 30;

#[test]
fn redraw_cadence_advances_over_one_second() {
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

    // Wait until the loop is pumping (first heartbeat).
    assert!(
        app.wait_for_log("redraw=", Duration::from_secs(20)),
        "expected at least one redraw heartbeat; log:\n{}",
        app.read_log()
    );

    // Let the cadence reach steady state (warmup past) before measuring, so we
    // sample a fixed 1.0s window of the steady cadence (AC-2.3), not the warmup.
    let warmup_deadline = Instant::now() + Duration::from_secs(20);
    while app.count_log_lines("redraw=") < WARMUP_REDRAWS {
        assert!(
            Instant::now() < warmup_deadline,
            "cadence never reached {WARMUP_REDRAWS} warmup redraws; log:\n{}",
            app.read_log()
        );
        std::thread::sleep(Duration::from_millis(50));
    }

    let start = app.count_log_lines("redraw=");
    std::thread::sleep(Duration::from_secs(1));
    let end = app.count_log_lines("redraw=");
    let delta = end.saturating_sub(start);

    let _ = app.terminate_and_wait(Duration::from_secs(10));

    assert!(
        delta >= MIN_REDRAWS,
        "redraw cadence too low: {delta} redraws in 1.0s (need >= {MIN_REDRAWS})"
    );
}
