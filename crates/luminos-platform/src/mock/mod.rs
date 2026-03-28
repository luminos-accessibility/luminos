//! Mock implementations of all six platform traits for unit testing.
//!
//! Gated behind `#[cfg(any(test, feature = "test_utils"))]`.
//!
//! # Usage
//!
//! In the same crate (unit tests):
//! ```rust,ignore
//! use crate::mock::MockScreenCapture;
//! ```
//!
//! In downstream crates (via `test_utils` feature):
//! ```rust,ignore
//! // Cargo.toml: luminos-platform = { workspace = true, features = ["test_utils"] }
//! use luminos_platform::mock::MockScreenCapture;
//! ```

pub mod audio;
pub mod capture;
pub mod focus;
pub mod input;
pub mod tts;
pub mod window;

pub use audio::MockAudioOutput;
pub use capture::MockScreenCapture;
pub use focus::MockFocusTracker;
pub use input::MockInputMonitor;
pub use tts::MockTtsEngine;
pub use window::MockWindowManager;

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that all six mock structs are re-exported from the mock module.
    /// This is a compile-time check -- if it compiles, all re-exports exist.
    #[test]
    fn mock_mod_reexports_all_six_structs() {
        let _capture = std::any::type_name::<MockScreenCapture>();
        let _focus = std::any::type_name::<MockFocusTracker>();
        let _tts = std::any::type_name::<MockTtsEngine>();
        let _window = std::any::type_name::<MockWindowManager>();
        let _input = std::any::type_name::<MockInputMonitor>();
        let _audio = std::any::type_name::<MockAudioOutput>();
    }
}
