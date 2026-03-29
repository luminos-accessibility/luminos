//! Core application state and runtime enumerations.
//!
//! Defines the display mode, tracking mode, color filter, and TTS status
//! enums used throughout the application. Also defines [`AppState`], the
//! runtime state container that wraps [`AppSettings`] with transient
//! runtime fields.

pub use luminos_types::{
    ColorFilterType, MagnificationMode, ScreenPoint, ScreenRect, TrackingMode, TtsStatus,
};

use crate::config::schema::AppSettings;

/// Runtime application state.
///
/// Contains both persisted settings ([`AppSettings`]) and transient
/// runtime state (viewport position, TTS status, etc.). Wrapped in
/// `Arc<ArcSwap<AppState>>` at the application level for lock-free
/// reads from the render thread and `rcu()` writes from the IPC thread.
#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    /// User-configurable settings (persisted to config.toml).
    pub settings: AppSettings,
    /// Current viewport source rectangle in screen coordinates.
    pub viewport: ScreenRect,
    /// Current TTS pipeline status.
    pub tts_status: TtsStatus,
    /// ID of the active display being magnified.
    pub active_display_id: Option<String>,
    /// Whether magnification is currently active.
    pub is_active: bool,
    /// Current mouse cursor position in screen coordinates.
    ///
    /// Updated by the input monitoring thread via
    /// [`StateManager::update_mouse_position()`](crate::state_manager::StateManager::update_mouse_position).
    pub mouse_position: ScreenPoint,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: AppSettings::default(),
            viewport: ScreenRect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            tts_status: TtsStatus::Idle,
            active_display_id: None,
            is_active: false,
            mouse_position: ScreenPoint { x: 0, y: 0 },
        }
    }
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

    // T005 tests -- AppState defaults

    #[test]
    fn app_state_default_settings_match() {
        assert_eq!(AppState::default().settings, AppSettings::default());
    }

    #[test]
    fn app_state_default_viewport_at_origin() {
        let viewport = AppState::default().viewport;
        assert_eq!(
            viewport,
            ScreenRect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            }
        );
    }

    #[test]
    fn app_state_default_tts_idle() {
        assert_eq!(AppState::default().tts_status, TtsStatus::Idle);
    }

    #[test]
    fn app_state_default_not_active() {
        assert!(!AppState::default().is_active);
    }

    #[test]
    fn app_state_default_no_active_display() {
        assert!(AppState::default().active_display_id.is_none());
    }

    // E03/002 T001 tests

    #[test]
    fn app_state_default_mouse_position_at_origin() {
        let state = AppState::default();
        assert_eq!(state.mouse_position, ScreenPoint { x: 0, y: 0 });
    }

    #[test]
    fn app_state_clone_preserves_mouse_position() {
        let state = AppState {
            mouse_position: ScreenPoint { x: 100, y: 200 },
            ..AppState::default()
        };
        let cloned = state.clone();
        assert_eq!(cloned.mouse_position, ScreenPoint { x: 100, y: 200 });
    }
}
