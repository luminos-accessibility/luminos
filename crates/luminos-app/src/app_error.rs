//! Top-level error type for the Luminos application shell.
//!
//! [`AppError`] aggregates startup and runtime failures so that `main` returns
//! `Result<(), AppError>`: a failure prints a typed, actionable message and
//! exits non-zero instead of panicking (CLAUDE.md: no `unwrap`/`expect` in
//! production paths). GPU errors are stringified at the boundary because wgpu's
//! error types differ per call site and carry no stable conversion.

use luminos_core::ConfigError;

/// Errors surfaced by the application shell during startup or the run loop.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A Tauri runtime/builder error (window creation, context build, run).
    #[error("Tauri runtime error: {0}")]
    Tauri(#[from] tauri::Error),

    /// Settings persistence failed while seeding the initial state. Non-fatal
    /// at startup (the app falls back to [`luminos_core::AppState::default`]),
    /// but retained as a typed variant for callers that choose to surface it.
    #[error("config error: {0}")]
    Config(#[from] ConfigError),

    /// wgpu device/surface/render failure. Stringified because wgpu error
    /// types vary per call and have no shared `From` target.
    #[error("GPU init failed: {0}")]
    Gpu(String),

    /// The named overlay window could not be retrieved from the app handle.
    #[error("overlay window '{0}' not found")]
    OverlayMissing(String),

    /// No X11 compositor is running, so the overlay cannot be transparent.
    /// Non-fatal: logged as a warning and the overlay renders opaque.
    #[error("no compositor detected; transparency unavailable")]
    NoCompositor,

    /// The Tauri overlay window could not be bridged to the platform
    /// `WindowManager` (XID extraction or X server connection failed). Story
    /// 002's `overlay_bridge`; non-fatal (logged, the overlay still renders).
    #[error("overlay bridge error: {0}")]
    Bridge(String),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn app_error_from_tauri_error_maps() {
        // A `tauri::Error` value mapped via `?`/`From` lands in `AppError::Tauri`
        // and renders with the expected prefix.
        let tauri_err = tauri::Error::WebviewNotFound;
        let app_err: AppError = tauri_err.into();
        assert!(
            matches!(app_err, AppError::Tauri(_)),
            "expected AppError::Tauri, got {app_err:?}"
        );
        assert!(
            app_err.to_string().starts_with("Tauri runtime error:"),
            "unexpected display: '{app_err}'"
        );
    }

    #[test]
    fn app_error_from_config_error_maps() {
        // `ConfigError` mapped via `From` lands in `AppError::Config` so the
        // startup seam can use `?` (correction #1, ConfigManager is real).
        let cfg_err = ConfigError::NoConfigDir;
        let app_err: AppError = cfg_err.into();
        assert!(
            matches!(app_err, AppError::Config(_)),
            "expected AppError::Config, got {app_err:?}"
        );
    }

    #[test]
    fn app_error_gpu_carries_message() {
        let app_err = AppError::Gpu("no adapter".to_string());
        assert_eq!(app_err.to_string(), "GPU init failed: no adapter");
    }

    #[test]
    fn app_error_overlay_missing_names_window() {
        let app_err = AppError::OverlayMissing("overlay".to_string());
        assert_eq!(app_err.to_string(), "overlay window 'overlay' not found");
    }

    #[test]
    fn app_error_bridge_carries_message() {
        let app_err = AppError::Bridge("no X11 handle".to_string());
        assert_eq!(
            app_err.to_string(),
            "overlay bridge error: no X11 handle",
            "Bridge variant must surface the underlying cause"
        );
    }
}
