//! Luminos application shell library.
//!
//! Exposes the building blocks of the single-event-loop Tauri application so
//! they can be unit-tested in-process (the binary `main` wires them into the
//! `tauri::Builder` and the `App::run` callback). See
//! `specs/E04-tauri-control-panel/001-app-shell-event-loop/` for the design.

#[cfg(feature = "tauri")]
pub mod app;
#[cfg(feature = "tauri")]
pub mod app_error;
#[cfg(feature = "tauri")]
pub mod capture_driver;
#[cfg(feature = "tauri")]
pub mod compositor;
#[cfg(feature = "tauri")]
pub mod events;
#[cfg(feature = "tauri")]
pub mod handle;
#[cfg(feature = "tauri")]
pub mod ipc;
#[cfg(feature = "tauri")]
pub mod notifier;
#[cfg(all(feature = "tauri", target_os = "linux"))]
pub mod overlay_bridge;
#[cfg(feature = "tauri")]
pub mod overlay_gpu;
pub mod platform_env;
#[cfg(feature = "tauri")]
pub mod signal;
#[cfg(feature = "tauri")]
pub mod tauri_commands;
#[cfg(feature = "tauri")]
pub mod tray;

#[cfg(feature = "tauri")]
pub use app_error::AppError;
#[cfg(feature = "tauri")]
pub use handle::LuminosHandle;
#[cfg(feature = "tauri")]
pub use notifier::AppNotifier;
