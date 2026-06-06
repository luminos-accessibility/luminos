//! The seven Phase-0 Tauri IPC commands (story E04/005).
//!
//! Each `#[tauri::command] #[specta::specta]` entry point is a thin async
//! wrapper that retrieves the per-field seams from the managed
//! [`LuminosHandle`] and delegates to a synchronous, runtime-free inner `fn`
//! (the `*_inner` functions). Splitting the logic out lets the command bodies be
//! unit-tested in-process without spinning up a full Tauri runtime (constructing
//! a `tauri::State<LuminosHandle>` requires a live `AppHandle`), per the DESIGN
//! testing strategy.
//!
//! Wiring (real engine types, reconciled per HLP Integration Points):
//! - reads go straight off `app_state.load()` / the frame-timing slot;
//! - mutations go through `luminos_core::StateManager` (RCU on the shared
//!   `Arc<ArcSwap<AppState>>`), then wake the loop via the `AppNotifier`
//!   dirty flag (AD-4);
//! - persistence locks the `config` mutex and delegates to `ConfigManager`.
//!
//! Commands return `Result<T, String>` (Tauri convention); no `unwrap`/`expect`.

use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use tauri::State;

use luminos_core::config::ConfigManager;
use luminos_core::{AppSettings, AppState, EventNotifier, StateManager};
use luminos_gpu::FrameTimingSummary;
use luminos_types::MagnificationMode;

use crate::handle::LuminosHandle;
use crate::notifier::AppNotifier;

/// Message returned when a persistence command runs but no `ConfigManager` was
/// seeded (startup hit `NoConfigDir`). Matches the story-006 contract.
const CONFIG_UNAVAILABLE: &str = "config unavailable";

// ---------------------------------------------------------------------------
// Read commands (FR-1, FR-2)
// ---------------------------------------------------------------------------

/// FR-1: returns the engine's current settings (a clone of
/// `AppState.settings`). No wake (a read does not mutate state).
fn get_current_settings_inner(app_state: &ArcSwap<AppState>) -> AppSettings {
    app_state.load().settings.clone()
}

/// FR-2: returns the latest frame-timing summary. Never errors — a poisoned
/// lock yields the last-known value rather than propagating (AC-1.1). Returns a
/// zeroed/last-known summary until story 003's loop has presented a frame.
fn get_frame_timings_inner(slot: &Mutex<FrameTimingSummary>) -> FrameTimingSummary {
    match slot.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

// ---------------------------------------------------------------------------
// Mutation commands (FR-3, FR-4)
// ---------------------------------------------------------------------------

/// FR-3: clamps `level` to `[1.5, 20]` via `StateManager::update_zoom_level`,
/// then wakes the loop. Rejects `NaN` explicitly (clamp would silently pass it
/// through) — NFR-2 server-side validation.
fn set_zoom_level_inner(
    app_state: &Arc<ArcSwap<AppState>>,
    notifier: &AppNotifier,
    level: f32,
) -> Result<(), String> {
    if level.is_nan() {
        return Err("zoom level must be a number (got NaN)".to_string());
    }
    StateManager::new(Arc::clone(app_state)).update_zoom_level(level);
    notifier.notify_state_changed();
    Ok(())
}

/// FR-4: writes the magnification mode via `StateManager` (RCU on
/// `settings.magnification.mode`), then wakes the loop. The enum is validated by
/// deserialization at the IPC boundary.
fn set_magnification_mode_inner(
    app_state: &Arc<ArcSwap<AppState>>,
    notifier: &AppNotifier,
    mode: MagnificationMode,
) {
    StateManager::new(Arc::clone(app_state)).set_magnification_mode(mode);
    notifier.notify_state_changed();
}

/// FR-4: flips `is_active` via `StateManager`, wakes the loop, and returns the
/// new active state read back from `AppState`.
fn toggle_magnification_inner(app_state: &Arc<ArcSwap<AppState>>, notifier: &AppNotifier) -> bool {
    StateManager::new(Arc::clone(app_state)).toggle_magnification();
    notifier.notify_state_changed();
    app_state.load().is_active
}

// ---------------------------------------------------------------------------
// Persistence commands (FR-5)
// ---------------------------------------------------------------------------

/// FR-5: persists the current `AppState.settings` via `ConfigManager::save`.
/// Returns `Err("config unavailable")` when no manager was seeded (no wake — a
/// save does not change `AppState`).
fn save_settings_inner(
    app_state: &ArcSwap<AppState>,
    config: &Mutex<Option<ConfigManager>>,
) -> Result<(), String> {
    let settings = app_state.load().settings.clone();
    let mut guard = config.lock().map_err(|e| e.to_string())?;
    let manager = guard
        .as_mut()
        .ok_or_else(|| CONFIG_UNAVAILABLE.to_string())?;
    manager.save(&settings).map_err(|e| e.to_string())
}

/// FR-5: resets settings to defaults via `ConfigManager::reset`, applies the
/// defaults to the live `AppState` (so the render loop and a subsequent
/// `get_current_settings` agree), wakes the loop, and returns the defaults.
fn reset_settings_inner(
    app_state: &Arc<ArcSwap<AppState>>,
    config: &Mutex<Option<ConfigManager>>,
    notifier: &AppNotifier,
) -> Result<AppSettings, String> {
    let defaults = {
        let mut guard = config.lock().map_err(|e| e.to_string())?;
        let manager = guard
            .as_mut()
            .ok_or_else(|| CONFIG_UNAVAILABLE.to_string())?;
        manager.reset().map_err(|e| e.to_string())?
    };
    StateManager::new(Arc::clone(app_state)).replace_settings(&defaults);
    notifier.notify_state_changed();
    Ok(defaults)
}

// ---------------------------------------------------------------------------
// Tauri command entry points
// ---------------------------------------------------------------------------

/// Returns the engine's current [`AppSettings`].
#[tauri::command]
#[specta::specta]
pub(crate) async fn get_current_settings(
    h: State<'_, LuminosHandle>,
) -> Result<AppSettings, String> {
    Ok(get_current_settings_inner(&h.app_state))
}

/// Returns the latest [`FrameTimingSummary`] (zeroed/last-known before the loop
/// runs; never errors).
#[tauri::command]
#[specta::specta]
pub(crate) async fn get_frame_timings(
    h: State<'_, LuminosHandle>,
) -> Result<FrameTimingSummary, String> {
    Ok(get_frame_timings_inner(&h.frame_timings))
}

/// Sets the zoom level (clamped to `[1.5, 20]`; `NaN` rejected) and wakes the
/// loop.
#[tauri::command]
#[specta::specta]
pub(crate) async fn set_zoom_level(level: f32, h: State<'_, LuminosHandle>) -> Result<(), String> {
    set_zoom_level_inner(&h.app_state, &h.notifier, level)
}

/// Sets the magnification mode and wakes the loop.
#[tauri::command]
#[specta::specta]
pub(crate) async fn set_magnification_mode(
    mode: MagnificationMode,
    h: State<'_, LuminosHandle>,
) -> Result<(), String> {
    set_magnification_mode_inner(&h.app_state, &h.notifier, mode);
    Ok(())
}

/// Toggles magnification, wakes the loop, and returns the new active state.
#[tauri::command]
#[specta::specta]
pub(crate) async fn toggle_magnification(h: State<'_, LuminosHandle>) -> Result<bool, String> {
    Ok(toggle_magnification_inner(&h.app_state, &h.notifier))
}

/// Persists the current settings to `config.toml`.
#[tauri::command]
#[specta::specta]
pub(crate) async fn save_settings(h: State<'_, LuminosHandle>) -> Result<(), String> {
    save_settings_inner(&h.app_state, &h.config)
}

/// Resets settings to defaults, applies them to the live state, and returns the
/// defaults.
#[tauri::command]
#[specta::specta]
pub(crate) async fn reset_settings(h: State<'_, LuminosHandle>) -> Result<AppSettings, String> {
    reset_settings_inner(&h.app_state, &h.config, &h.notifier)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::*;
    use crate::notifier::tests::generate_test_notifier;
    use luminos_core::config::ConfigManager;

    // ---- shared fixtures -------------------------------------------------

    /// A fresh shared `AppState` Arc seeded with defaults.
    fn generate_test_app_state() -> Arc<ArcSwap<AppState>> {
        Arc::new(ArcSwap::from_pointee(AppState::default()))
    }

    /// A `ConfigManager` rooted at a throwaway temp `config.toml` (no file yet,
    /// so it holds defaults), wrapped the way the handle holds it. Returns the
    /// dir guard so it outlives the test.
    fn generate_test_config() -> (tempfile::TempDir, Arc<Mutex<Option<ConfigManager>>>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("luminos").join("config.toml");
        let manager = ConfigManager::load_from(&path).unwrap();
        (dir, Arc::new(Mutex::new(Some(manager))))
    }

    // ---- T003: read commands --------------------------------------------

    #[test]
    fn get_current_settings_returns_state() {
        let app_state = generate_test_app_state();
        StateManager::new(Arc::clone(&app_state)).update_zoom_level(6.0);
        let settings = get_current_settings_inner(&app_state);
        assert!((settings.magnification.zoom_level - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn get_frame_timings_zeroed_before_loop() {
        let slot = Mutex::new(FrameTimingSummary {
            average_ms: 0.0,
            p99_ms: 0.0,
            min_ms: 0.0,
            max_ms: 0.0,
            target_fps: 60,
        });
        let summary = get_frame_timings_inner(&slot);
        assert!(summary.p99_ms.abs() < f64::EPSILON);
        assert_eq!(summary.target_fps, 60);
    }

    // ---- T004: mutation commands ----------------------------------------

    #[test]
    fn set_zoom_level_clamps() {
        let app_state = generate_test_app_state();
        let notifier = generate_test_notifier();
        set_zoom_level_inner(&app_state, &notifier, 0.5).unwrap();
        assert!((app_state.load().settings.magnification.zoom_level - 1.5).abs() < f32::EPSILON);
        set_zoom_level_inner(&app_state, &notifier, 50.0).unwrap();
        assert!((app_state.load().settings.magnification.zoom_level - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn set_zoom_level_rejects_nan() {
        let app_state = generate_test_app_state();
        let notifier = generate_test_notifier();
        let result = set_zoom_level_inner(&app_state, &notifier, f32::NAN);
        assert!(result.is_err(), "NaN zoom must be rejected");
    }

    #[test]
    fn set_zoom_level_wakes_loop() {
        let app_state = generate_test_app_state();
        let notifier = generate_test_notifier();
        let flag = notifier.dirty_flag();
        flag.store(false, Ordering::Release);
        set_zoom_level_inner(&app_state, &notifier, 5.0).unwrap();
        assert!(
            flag.load(Ordering::Acquire),
            "set_zoom_level must wake the loop"
        );
    }

    #[test]
    fn set_magnification_mode_writes_state() {
        let app_state = generate_test_app_state();
        let notifier = generate_test_notifier();
        set_magnification_mode_inner(&app_state, &notifier, MagnificationMode::Lens);
        assert_eq!(
            app_state.load().settings.magnification.mode,
            MagnificationMode::Lens
        );
    }

    #[test]
    fn set_magnification_mode_wakes_loop() {
        let app_state = generate_test_app_state();
        let notifier = generate_test_notifier();
        let flag = notifier.dirty_flag();
        flag.store(false, Ordering::Release);
        set_magnification_mode_inner(&app_state, &notifier, MagnificationMode::Docked);
        assert!(flag.load(Ordering::Acquire));
    }

    #[test]
    fn toggle_magnification_returns_new_state() {
        let app_state = generate_test_app_state();
        let notifier = generate_test_notifier();
        assert!(!app_state.load().is_active, "default inactive");
        let after_first = toggle_magnification_inner(&app_state, &notifier);
        assert!(after_first, "first toggle returns true");
        let after_second = toggle_magnification_inner(&app_state, &notifier);
        assert!(!after_second, "second toggle returns false");
    }

    // ---- T005: persistence commands -------------------------------------

    #[test]
    fn save_settings_delegates_to_config() {
        let app_state = generate_test_app_state();
        StateManager::new(Arc::clone(&app_state)).update_zoom_level(7.0);
        let (_dir, config) = generate_test_config();
        save_settings_inner(&app_state, &config).unwrap();
        // The manager's cache now holds the saved settings.
        let guard = config.lock().unwrap();
        let cached = guard.as_ref().unwrap().settings();
        assert!((cached.magnification.zoom_level - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reset_settings_returns_defaults() {
        let app_state = generate_test_app_state();
        StateManager::new(Arc::clone(&app_state)).update_zoom_level(9.0);
        let (_dir, config) = generate_test_config();
        let notifier = generate_test_notifier();
        let returned = reset_settings_inner(&app_state, &config, &notifier).unwrap();
        assert_eq!(returned, AppSettings::default(), "reset returns defaults");
        // Defaults are applied to the live state too.
        assert_eq!(app_state.load().settings, AppSettings::default());
    }

    #[test]
    fn reset_settings_wakes_loop() {
        let app_state = generate_test_app_state();
        let (_dir, config) = generate_test_config();
        let notifier = generate_test_notifier();
        let flag = notifier.dirty_flag();
        flag.store(false, Ordering::Release);
        reset_settings_inner(&app_state, &config, &notifier).unwrap();
        assert!(
            flag.load(Ordering::Acquire),
            "reset_settings must wake the loop"
        );
    }

    #[test]
    fn save_settings_config_none_errors() {
        let app_state = generate_test_app_state();
        let config: Arc<Mutex<Option<ConfigManager>>> = Arc::new(Mutex::new(None));
        let err = save_settings_inner(&app_state, &config).unwrap_err();
        assert_eq!(err, CONFIG_UNAVAILABLE);
    }

    #[test]
    fn reset_settings_config_none_errors() {
        let app_state = generate_test_app_state();
        let config: Arc<Mutex<Option<ConfigManager>>> = Arc::new(Mutex::new(None));
        let notifier = generate_test_notifier();
        let err = reset_settings_inner(&app_state, &config, &notifier).unwrap_err();
        assert_eq!(err, CONFIG_UNAVAILABLE);
    }
}
