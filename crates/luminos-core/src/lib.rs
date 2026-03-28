//! Luminos core engine library.
//!
//! Provides the application state types, error hierarchy, and
//! settings schema shared by the render thread, TTS pipeline,
//! and control panel IPC layer.

pub mod config;
pub mod error;
pub mod state;

pub use config::schema::AppSettings;
pub use error::LuminosError;
pub use state::{AppState, ColorFilterType, MagnificationMode, TrackingMode, TtsStatus};
