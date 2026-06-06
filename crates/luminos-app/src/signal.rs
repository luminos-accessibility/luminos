//! Graceful shutdown on POSIX termination signals.
//!
//! tao's GTK3 backend does not convert `SIGTERM`/`SIGINT` into a
//! `RunEvent::ExitRequested`, so a bare signal terminates the process abruptly
//! (no thread join, no GPU teardown).
//!
//! We must NOT block these signals process-wide: GTK/GLib install their own
//! signal handling during initialization and blocking `SIGTERM`/`SIGINT` before
//! GTK starts up prevents window realization. Instead we install an
//! async-signal-safe `sigaction` handler that merely sets an atomic flag; the
//! cadence thread polls [`shutdown_requested`] and asks the loop to exit
//! cleanly on the main thread. The loop's `ExitRequested`/`Exit` handler then
//! joins threads and drops GPU resources.

use std::sync::atomic::{AtomicBool, Ordering};

/// Set by the signal handler; polled by the cadence thread.
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Returns whether a termination signal has been received since startup.
#[must_use]
pub fn shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::Acquire)
}

/// Async-signal-safe handler: only touches an atomic (no allocation, no I/O).
#[cfg(unix)]
extern "C" fn handle_signal(_sig: libc::c_int) {
    SHUTDOWN_REQUESTED.store(true, Ordering::Release);
}

/// Installs the `SIGTERM`/`SIGINT` handler. Safe to call before GTK init: a
/// plain `sigaction` handler does not interfere with GTK's main loop the way a
/// process-wide signal block would.
#[cfg(unix)]
pub fn install_termination_handler() {
    // SAFETY: `action` is fully initialized before `sigaction`; the handler is
    // async-signal-safe (sets one atomic). `sa_sigaction` aliases `sa_handler`.
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = handle_signal as *const () as usize;
        libc::sigemptyset(&raw mut action.sa_mask);
        action.sa_flags = libc::SA_RESTART;
        libc::sigaction(libc::SIGTERM, &raw const action, std::ptr::null_mut());
        libc::sigaction(libc::SIGINT, &raw const action, std::ptr::null_mut());
    }
}

/// No-op on non-Unix platforms (Windows shuts down via window close).
#[cfg(not(unix))]
pub fn install_termination_handler() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_shutdown_flag_starts_clear() {
        // Fresh process: no signal delivered yet. (Other tests in this binary
        // do not raise signals, so this observes the initial state.)
        assert!(!shutdown_requested());
    }
}
