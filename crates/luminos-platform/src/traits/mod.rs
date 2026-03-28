//! Platform abstraction trait definitions and associated types.
//!
//! Each sub-module defines one platform trait, its error type, and any
//! associated types specific to that subsystem. The `types` module contains
//! types shared across multiple traits.

pub mod types;
pub mod screen_capture;
pub mod focus_tracker;
pub mod tts_engine;
pub mod window_manager;
pub mod input_monitor;
pub mod audio_output;

// Re-export all public items for convenient `use luminos_platform::traits::*`
pub use types::*;
pub use screen_capture::*;
pub use focus_tracker::*;
pub use tts_engine::*;
pub use window_manager::*;
pub use input_monitor::*;
pub use audio_output::*;
