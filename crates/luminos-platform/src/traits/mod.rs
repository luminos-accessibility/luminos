//! Platform abstraction trait definitions and associated types.
//!
//! Each sub-module defines one platform trait, its error type, and any
//! associated types specific to that subsystem. The `types` module contains
//! types shared across multiple traits.

pub mod audio_output;
pub mod focus_tracker;
pub mod input_monitor;
pub mod screen_capture;
pub mod tts_engine;
pub mod types;
pub mod window_manager;

// Re-export all public items for convenient `use luminos_platform::traits::*`
pub use audio_output::*;
pub use focus_tracker::*;
pub use input_monitor::*;
pub use screen_capture::*;
pub use tts_engine::*;
pub use types::*;
pub use window_manager::*;
