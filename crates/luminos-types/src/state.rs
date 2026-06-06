//! Runtime state enumerations.
//!
//! Defines display mode, tracking mode, color filter, and TTS status
//! enums used throughout the application.

use serde::{Deserialize, Serialize};

/// Screen magnification display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum MagnificationMode {
    /// Entire screen is magnified; overlay covers full display.
    FullScreen,
    /// Magnified view docked to a screen edge.
    Docked,
    /// Magnifying glass lens follows cursor.
    Lens,
}

/// Which element the magnification viewport tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum TrackingMode {
    /// Viewport follows the mouse cursor.
    Cursor,
    /// Viewport follows the keyboard focus element.
    Focus,
    /// Viewport follows the text insertion caret.
    TextCaret,
}

/// Color filter applied to the magnified view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum ColorFilterType {
    /// No color filter (passthrough).
    None,
    /// Full color inversion.
    Invert,
    /// Inversion that preserves image hues (inverts lightness only).
    SmartInvert,
    /// Converts to grayscale.
    Grayscale,
    /// High-contrast black/white with configurable threshold.
    HighContrast,
    /// User-defined 4x4 color transformation matrix.
    Custom,
}

/// TTS pipeline runtime status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TtsStatus {
    /// No speech in progress, ready for new requests.
    Idle,
    /// Loading a voice model.
    Loading,
    /// Currently synthesizing and playing speech.
    Speaking,
    /// Speech finished synthesizing, audio buffer draining.
    Draining,
    /// An error occurred in the TTS pipeline.
    Error,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn magnification_mode_serde_roundtrip() {
        let variants = [
            (MagnificationMode::FullScreen, "\"FullScreen\""),
            (MagnificationMode::Docked, "\"Docked\""),
            (MagnificationMode::Lens, "\"Lens\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: MagnificationMode = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }

    #[test]
    fn tracking_mode_serde_roundtrip() {
        let variants = [
            (TrackingMode::Cursor, "\"Cursor\""),
            (TrackingMode::Focus, "\"Focus\""),
            (TrackingMode::TextCaret, "\"TextCaret\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: TrackingMode = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }

    #[test]
    fn color_filter_type_serde_roundtrip() {
        let variants = [
            (ColorFilterType::None, "\"None\""),
            (ColorFilterType::Invert, "\"Invert\""),
            (ColorFilterType::SmartInvert, "\"SmartInvert\""),
            (ColorFilterType::Grayscale, "\"Grayscale\""),
            (ColorFilterType::HighContrast, "\"HighContrast\""),
            (ColorFilterType::Custom, "\"Custom\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: ColorFilterType = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }

    #[test]
    fn tts_status_serde_roundtrip() {
        let variants = [
            (TtsStatus::Idle, "\"Idle\""),
            (TtsStatus::Loading, "\"Loading\""),
            (TtsStatus::Speaking, "\"Speaking\""),
            (TtsStatus::Draining, "\"Draining\""),
            (TtsStatus::Error, "\"Error\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: TtsStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }

    // E04/005 T002: the IPC-reachable enums must derive `specta::Type` (DC-5).
    // TtsStatus is intentionally NOT covered — it is not part of `AppSettings`.

    #[test]
    fn state_enums_implement_specta_type() {
        fn assert_specta_type<T: specta::Type>() {}
        assert_specta_type::<MagnificationMode>();
        assert_specta_type::<TrackingMode>();
        assert_specta_type::<ColorFilterType>();
    }
}
