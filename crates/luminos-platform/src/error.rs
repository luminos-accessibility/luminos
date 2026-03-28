//! Convenience re-exports for all platform error types.
//!
//! Consumers can import errors via `use luminos_platform::error::*`
//! instead of navigating individual trait modules.

pub use crate::traits::audio_output::AudioError;
pub use crate::traits::focus_tracker::FocusError;
pub use crate::traits::input_monitor::InputError;
pub use crate::traits::screen_capture::CaptureError;
pub use crate::traits::tts_engine::TtsError;
pub use crate::traits::window_manager::WindowError;
