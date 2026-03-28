//! luminos-platform: Platform abstraction layer for Luminos.
//!
//! Defines six platform traits and their associated types. Platform-specific
//! backends implement these traits; the core engine programs against the
//! trait interfaces exclusively.

pub mod error;
pub mod traits;

#[cfg(any(test, feature = "test_utils"))]
pub mod mock;

// Shared code used by multiple platforms (Linux + OpenBSD).
#[cfg(any(target_os = "linux", target_os = "openbsd"))]
pub(crate) mod common;

// Platform backends -- only the relevant platform is compiled.
// On Linux, BOTH x11 and wayland modules are compiled; runtime selection
// chooses the active backend (see doc-02 Section 5.3).

#[cfg(target_os = "linux")]
mod linux_x11;

#[cfg(target_os = "linux")]
mod linux_wayland;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "openbsd")]
mod openbsd;

#[cfg(target_os = "windows")]
mod windows;

use traits::{AudioOutput, FocusTracker, InputMonitor, ScreenCapture, WindowManager};

/// Bundle of all platform-specific trait implementations.
///
/// Created once at application startup by the platform factory function.
/// The core engine receives this and programs against the trait interfaces.
///
/// `TtsEngine` is excluded because it is constructed separately by the
/// `luminos-tts` crate (it depends on `AudioOutput` + espeak-ng subprocess,
/// not on platform APIs directly).
pub struct PlatformBackends {
    /// Screen capture backend.
    pub capture: Box<dyn ScreenCapture>,
    /// Accessibility focus tracking backend.
    pub focus_tracker: Box<dyn FocusTracker>,
    /// Magnification overlay window manager.
    pub window_mgr: Box<dyn WindowManager>,
    /// Global input event monitor.
    pub input_monitor: Box<dyn InputMonitor>,
    /// Audio output for TTS playback.
    pub audio_output: Box<dyn AudioOutput>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that `PlatformBackends` has five fields with correct types.
    /// This is a compile-only test -- if it compiles, the struct shape is correct.
    /// Full construction requires mock implementations from Story 003.
    #[allow(dead_code)]
    fn platform_backends_struct_has_five_fields(backends: PlatformBackends) {
        let _: Box<dyn ScreenCapture> = backends.capture;
        let _: Box<dyn FocusTracker> = backends.focus_tracker;
        let _: Box<dyn WindowManager> = backends.window_mgr;
        let _: Box<dyn InputMonitor> = backends.input_monitor;
        let _: Box<dyn AudioOutput> = backends.audio_output;
    }
}
