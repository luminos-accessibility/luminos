//! Tauri managed-state handle shared with the run loop and (later) commands.
//!
//! [`LuminosHandle`] is registered via `tauri::Builder::manage` so every Tauri
//! command (stories 005+) and the run-loop closure can reach the live
//! application state, the persistence layer, and the wake mechanism.

use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use luminos_core::{AppState, ConfigManager};
#[cfg(target_os = "linux")]
use luminos_platform::linux_x11::X11WindowManager;

use crate::notifier::AppNotifier;

/// Shared application handle registered as Tauri managed state.
///
/// The render path reads `app_state` lock-free every redraw via
/// [`ArcSwap::load`]; mutations go through `luminos_core::StateManager`
/// (stories 003/005). `config` is `Option` because startup falls back to
/// defaults when no config directory is resolvable (correction #1: the real
/// `ConfigManager` is wired here, not a stub).
pub struct LuminosHandle {
    /// Live application state. Read lock-free on the render path.
    pub app_state: Arc<ArcSwap<AppState>>,
    /// Persistence layer; `None` when settings could not be seeded from disk.
    /// Behind a `Mutex` because config access is brief and off the render path.
    pub config: Arc<Mutex<Option<ConfigManager>>>,
    /// Wake mechanism handed to background threads (stories 003/005).
    pub notifier: AppNotifier,
    /// Tauri app handle for emitting events (story 005) and window control.
    pub app: tauri::AppHandle,
    /// Latest frame-timing summary, written by the render loop each presented
    /// frame (story 003) and read by the `get_frame_timings` IPC command
    /// (story 005). Behind a `Mutex` because it is updated off the lock-free
    /// state path; the summary is small and the lock is uncontended. Starts as
    /// a zeroed summary so the command returns last-known/zeroed data before
    /// the loop has rendered.
    pub frame_timings: Arc<Mutex<luminos_gpu::FrameTimingSummary>>,
    /// Platform overlay `WindowManager` bound to the tao/Tauri overlay window
    /// at `RunEvent::Ready` (story 002). `None` until the bridge runs (or when
    /// XID extraction fails). Story 003 reaches it here to drive
    /// geometry/visibility/stacking (via the `WindowManager` trait) and to query
    /// the self-capture XID (`overlay_window_id()`). Behind a `Mutex` because
    /// trait methods take `&self`/`&mut self` and are issued off the render hot
    /// loop. Stored as the concrete X11 backend (not `Box<dyn WindowManager>`)
    /// because the self-capture XID accessor is inherent to it, not on the trait
    /// (FR-6 keeps the trait surface unchanged). Linux-only (X11-specific).
    #[cfg(target_os = "linux")]
    pub window_manager: Arc<Mutex<Option<luminos_platform::linux_x11::X11WindowManager>>>,
    /// System tray icon, stashed so it outlives `setup` (story 007, D6). A
    /// `TrayIcon` is reference counted — dropping the last handle removes the
    /// icon from the tray — so the live icon MUST be retained here for the
    /// process lifetime. `None` when the tray degraded gracefully (no SNI host,
    /// FR-3) or before `setup` ran. Behind a `Mutex` because it is set once at
    /// `setup` and otherwise only read; Linux-only (tray is X11/SNI this epic).
    #[cfg(target_os = "linux")]
    pub tray: Arc<Mutex<Option<tauri::tray::TrayIcon<tauri::Wry>>>>,
}

/// A zeroed frame-timing summary used before the render loop has produced any
/// frames (story-005 `get_frame_timings` returns this until story 003's loop
/// runs). `FrameTimingSummary` has no `Default`, so it is built explicitly.
fn zeroed_frame_timing_summary() -> luminos_gpu::FrameTimingSummary {
    luminos_gpu::FrameTimingSummary {
        average_ms: 0.0,
        p99_ms: 0.0,
        min_ms: 0.0,
        max_ms: 0.0,
        target_fps: 60,
    }
}

impl LuminosHandle {
    /// Constructs the handle from its parts.
    #[must_use]
    pub fn new(
        app_state: Arc<ArcSwap<AppState>>,
        config: Option<ConfigManager>,
        notifier: AppNotifier,
        app: tauri::AppHandle,
    ) -> Self {
        Self {
            app_state,
            config: Arc::new(Mutex::new(config)),
            notifier,
            app,
            frame_timings: Arc::new(Mutex::new(zeroed_frame_timing_summary())),
            #[cfg(target_os = "linux")]
            window_manager: Arc::new(Mutex::new(None)),
            #[cfg(target_os = "linux")]
            tray: Arc::new(Mutex::new(None)),
        }
    }

    /// Stashes the live system tray icon so it outlives `setup` (story 007).
    /// Tolerates a poisoned lock by replacing the held icon. Passing `None`
    /// records the graceful-degrade outcome (no SNI host).
    #[cfg(target_os = "linux")]
    pub fn set_tray(&self, tray: Option<tauri::tray::TrayIcon<tauri::Wry>>) {
        match self.tray.lock() {
            Ok(mut guard) => *guard = tray,
            Err(poisoned) => {
                log::error!("tray mutex poisoned; replacing held tray icon");
                *poisoned.into_inner() = tray;
            }
        }
    }

    /// Whether a live tray icon is currently stashed (story 007). Returns
    /// `false` when the tray degraded or the lock is poisoned-and-empty.
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn has_tray(&self) -> bool {
        match self.tray.lock() {
            Ok(guard) => guard.is_some(),
            Err(poisoned) => poisoned.into_inner().is_some(),
        }
    }

    /// Writes the latest frame-timing summary (called by the render loop after
    /// each presented frame). Tolerates a poisoned lock by replacing the value.
    pub fn set_frame_timings(&self, summary: luminos_gpu::FrameTimingSummary) {
        match self.frame_timings.lock() {
            Ok(mut guard) => *guard = summary,
            Err(poisoned) => *poisoned.into_inner() = summary,
        }
    }

    /// Returns the latest frame-timing summary for the `get_frame_timings` IPC
    /// command (story 005). Returns a zeroed summary if the lock is poisoned.
    #[must_use]
    pub fn frame_timings(&self) -> luminos_gpu::FrameTimingSummary {
        match self.frame_timings.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// Stores the overlay `X11WindowManager` bound by the story-002 bridge.
    /// Replaces any prior manager.
    #[cfg(target_os = "linux")]
    pub fn set_window_manager(&self, manager: luminos_platform::linux_x11::X11WindowManager) {
        match self.window_manager.lock() {
            Ok(mut guard) => *guard = Some(manager),
            Err(poisoned) => {
                log::error!("window_manager mutex poisoned; replacing held manager");
                *poisoned.into_inner() = Some(manager);
            }
        }
    }

    /// Returns the overlay's X11 window id for the capture path's self-capture
    /// exclusion (story 003), if a `WindowManager` is bound.
    ///
    /// Returns `None` when no manager is bound yet or the lock is poisoned.
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn overlay_window_id(&self) -> Option<u64> {
        let guard = self.window_manager.lock().ok()?;
        guard.as_ref().and_then(X11WindowManager::overlay_window_id)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    use luminos_core::StateManager;

    use super::*;
    use crate::notifier::AppNotifier;

    #[test]
    fn handle_holds_real_app_state() {
        // Construct the state half of the handle without a Tauri runtime: the
        // `app: AppHandle` field needs a live app, so this test exercises the
        // `Arc<ArcSwap<AppState>>` + `config` seam directly (the full handle is
        // covered by the subprocess `managed_state` probe).
        let app_state = Arc::new(ArcSwap::from_pointee(AppState::default()));
        let config: Arc<Mutex<Option<ConfigManager>>> = Arc::new(Mutex::new(None));
        let notifier = AppNotifier::new(Arc::new(AtomicBool::new(false)));

        // The render path reads through ArcSwap; assert the seeded defaults.
        let manager = StateManager::new(Arc::clone(&app_state));
        let loaded = manager.load();
        let expected = AppState::default().settings.magnification.zoom_level;
        assert!(
            (loaded.settings.magnification.zoom_level - expected).abs() < f32::EPSILON,
            "expected seeded default zoom '{expected}'"
        );
        assert!(
            config.lock().unwrap().is_none(),
            "config should be None until story 004 seeds it (or NoConfigDir)"
        );
        let _ = notifier.dirty_flag();
    }

    #[test]
    fn handle_app_state_is_shared_with_state_manager() {
        // A write through StateManager is visible via the handle's ArcSwap,
        // proving they share one Arc (the render loop relies on this).
        let app_state = Arc::new(ArcSwap::from_pointee(AppState::default()));
        let manager = StateManager::new(Arc::clone(&app_state));

        manager.update_zoom_level(4.0);

        let via_handle = app_state.load();
        assert!(
            (via_handle.settings.magnification.zoom_level - 4.0).abs() < f32::EPSILON,
            "handle's ArcSwap should observe the StateManager write"
        );
    }

    #[test]
    fn handle_config_flag_round_trips() {
        let config: Arc<Mutex<Option<ConfigManager>>> = Arc::new(Mutex::new(None));
        assert!(config.lock().unwrap().is_none());
        let flag = Arc::new(AtomicBool::new(false));
        assert!(!flag.load(Ordering::Acquire));
        flag.store(true, Ordering::Release);
        assert!(flag.load(Ordering::Acquire));
    }

    // T009: the frame-timing slot the render loop writes and story 005 reads.

    #[test]
    fn handle_frame_timing_slot_starts_zeroed() {
        let zeroed = zeroed_frame_timing_summary();
        assert!(zeroed.p99_ms.abs() < f64::EPSILON);
        assert!(zeroed.average_ms.abs() < f64::EPSILON);
        assert_eq!(zeroed.target_fps, 60);
    }

    #[test]
    fn handle_frame_timing_slot_round_trips() {
        // The Arc<Mutex<FrameTimingSummary>> seam: a write through the slot is
        // observed by a reader (story 005's `get_frame_timings` reads the same
        // Arc the loop writes each frame).
        let slot: Arc<Mutex<luminos_gpu::FrameTimingSummary>> =
            Arc::new(Mutex::new(zeroed_frame_timing_summary()));

        let populated = luminos_gpu::FrameTimingSummary {
            average_ms: 8.0,
            p99_ms: 12.5,
            min_ms: 6.0,
            max_ms: 18.0,
            target_fps: 60,
        };
        *slot.lock().unwrap() = populated.clone();

        let read = slot.lock().unwrap().clone();
        assert_eq!(read, populated, "the slot must round-trip the summary");
        assert!(read.p99_ms > 0.0, "populated p99 should be non-zero");
    }
}
