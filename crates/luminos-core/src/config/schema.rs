//! Application settings schema and configuration types.
//!
//! Defines [`AppSettings`] and its nested sub-structs:
//! [`MagnificationSettings`], [`ColorFilterConfig`], [`CursorConfig`],
//! and [`SpeechSettings`]. These types mirror the TypeScript Zod schema
//! (doc-05 Section 3.2) and are serialized to TOML for persistence and
//! JSON for IPC.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::state::{ColorFilterType, MagnificationMode, TrackingMode};

// ---------------------------------------------------------------------------
// Supporting enums (re-exported from luminos-types)
// ---------------------------------------------------------------------------

pub use luminos_types::{DockEdge, GpuPreference, InterpolationMode, LensShape, PresentMode};

/// Kokoro ONNX model quantization variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelVariant {
    /// 4-bit quantized model.
    Q4,
    /// 8-bit quantized model (~92MB).
    Q8,
    /// 16-bit floating point model.
    Fp16,
    /// 32-bit floating point model.
    Fp32,
}

/// Hotkey action identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HotkeyAction {
    /// Increase zoom level.
    ZoomIn,
    /// Decrease zoom level.
    ZoomOut,
    /// Reset zoom to default.
    ZoomReset,
    /// Toggle magnification on/off.
    ToggleMagnification,
    /// Cycle through magnification modes.
    CycleMode,
    /// Read aloud what is visible on screen.
    ReadWhatISee,
    /// Read aloud the current text selection.
    ReadSelection,
    /// Stop any active speech.
    StopSpeech,
    /// Visually locate the cursor.
    FindCursor,
}

/// A keyboard shortcut binding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    /// Key name (e.g., "Equal", "Minus", "F1").
    pub key: String,
    /// Modifier keys required.
    pub modifiers: Vec<ModifierKey>,
}

/// Modifier key names for keybindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierKey {
    /// Control key.
    Ctrl,
    /// Shift key.
    Shift,
    /// Alt key.
    Alt,
    /// Super/Windows key (Linux/Windows).
    Super,
    /// Meta key (macOS Command).
    Meta,
}

// ---------------------------------------------------------------------------
// Configuration sub-structs
// ---------------------------------------------------------------------------

/// Magnification-related settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MagnificationSettings {
    /// Zoom level multiplier (1.5 to 20.0).
    pub zoom_level: f32,
    /// Active magnification display mode.
    pub mode: MagnificationMode,
    /// What the viewport tracks.
    pub tracking_mode: TrackingMode,
    /// Docked mode: which screen edge (if docked).
    pub docked_edge: Option<DockEdge>,
    /// Docked mode: percentage of screen reserved (10-90).
    pub docked_size_percent: Option<u32>,
    /// Lens mode: width in pixels.
    pub lens_width: Option<u32>,
    /// Lens mode: height in pixels.
    pub lens_height: Option<u32>,
    /// Lens mode: shape.
    pub lens_shape: Option<LensShape>,
    /// Target frames per second (15-144).
    pub target_fps: u32,
    /// VSync/present strategy.
    pub present_mode: PresentMode,
    /// GPU selection preference.
    pub gpu_preference: GpuPreference,
    /// Scaling interpolation algorithm.
    pub interpolation: InterpolationMode,
    /// Smooth scrolling animation.
    pub smooth_scrolling: bool,
}

impl Default for MagnificationSettings {
    fn default() -> Self {
        Self {
            zoom_level: 2.0,
            mode: MagnificationMode::FullScreen,
            tracking_mode: TrackingMode::Cursor,
            docked_edge: None,
            docked_size_percent: None,
            lens_width: None,
            lens_height: None,
            lens_shape: None,
            target_fps: 60,
            present_mode: PresentMode::Quality,
            gpu_preference: GpuPreference::LowPower,
            interpolation: InterpolationMode::Bilinear,
            smooth_scrolling: true,
        }
    }
}

/// Color filter configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorFilterConfig {
    /// Active filter type.
    pub filter_type: ColorFilterType,
    /// Brightness adjustment (-1.0 to 1.0).
    pub brightness: f32,
    /// Contrast multiplier (0.0 to 3.0).
    pub contrast: f32,
    /// Custom 4x4 color matrix (row-major, 16 elements).
    /// Present only when `filter_type` is `Custom`.
    pub color_matrix: Option<[f32; 16]>,
}

impl Default for ColorFilterConfig {
    fn default() -> Self {
        Self {
            filter_type: ColorFilterType::None,
            brightness: 0.0,
            contrast: 1.0,
            color_matrix: None,
        }
    }
}

/// Cursor enhancement configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CursorConfig {
    /// Whether the cursor is enlarged in the magnified view.
    pub enlarged_cursor: bool,
    /// Cursor scale factor (1.0 to 4.0).
    pub cursor_scale: f32,
    /// Whether crosshairs are drawn at the cursor position.
    pub crosshairs_enabled: bool,
    /// Crosshair line width in pixels (1-10).
    pub crosshair_width: u32,
    /// Crosshair color as CSS hex string (e.g., "#ff0000").
    pub crosshair_color: String,
    /// Whether a halo circle surrounds the cursor.
    pub halo_enabled: bool,
    /// Halo radius in pixels (10-200).
    pub halo_radius: u32,
    /// Halo color as CSS hex string.
    pub halo_color: String,
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self {
            enlarged_cursor: false,
            cursor_scale: 1.0,
            crosshairs_enabled: false,
            crosshair_width: 2,
            crosshair_color: "#ff0000".to_string(),
            halo_enabled: false,
            halo_radius: 50,
            halo_color: "#ffff0080".to_string(),
        }
    }
}

/// Speech / TTS configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeechSettings {
    /// Whether TTS is enabled.
    pub enabled: bool,
    /// Active voice identifier.
    pub voice_id: String,
    /// Speech rate multiplier (0.5 to 3.0).
    pub speech_rate: f32,
    /// Speech volume (0.0 to 1.0).
    pub speech_volume: f32,
    /// Kokoro quantization variant.
    pub model_variant: ModelVariant,
}

impl Default for SpeechSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            voice_id: String::new(),
            speech_rate: 1.0,
            speech_volume: 1.0,
            model_variant: ModelVariant::Q8,
        }
    }
}

// ---------------------------------------------------------------------------
// Root settings struct
// ---------------------------------------------------------------------------

/// Root application settings schema.
///
/// This struct represents all user-configurable settings. It is
/// serialized to `config.toml` for persistence and to JSON for IPC.
/// The TypeScript `AppSettings` Zod schema (doc-05 Section 3.2)
/// mirrors this struct field-for-field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Magnification display settings.
    pub magnification: MagnificationSettings,
    /// Color filter settings.
    pub color_filter: ColorFilterConfig,
    /// Cursor enhancement settings.
    pub cursor: CursorConfig,
    /// Speech / TTS settings.
    pub speech: SpeechSettings,
    /// Keyboard shortcut bindings (action -> binding or null).
    pub keybindings: HashMap<HotkeyAction, Option<KeyBinding>>,
    /// Whether Luminos starts automatically on login.
    pub start_on_login: bool,
    /// Whether closing the control panel minimizes to tray.
    pub minimize_to_tray: bool,
    /// Whether the control panel is shown on application start.
    pub show_panel_on_start: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            magnification: MagnificationSettings::default(),
            color_filter: ColorFilterConfig::default(),
            cursor: CursorConfig::default(),
            speech: SpeechSettings::default(),
            keybindings: HashMap::new(),
            start_on_login: false,
            minimize_to_tray: true,
            show_panel_on_start: true,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // T004a tests -- sub-struct defaults

    #[test]
    fn magnification_settings_default_zoom_level() {
        let m = MagnificationSettings::default();
        assert!(
            (m.zoom_level - 2.0).abs() < f32::EPSILON,
            "expected zoom_level 2.0, got {}",
            m.zoom_level
        );
    }

    #[test]
    fn magnification_settings_default_mode() {
        assert_eq!(
            MagnificationSettings::default().mode,
            MagnificationMode::FullScreen
        );
    }

    #[test]
    fn magnification_settings_default_tracking() {
        assert_eq!(
            MagnificationSettings::default().tracking_mode,
            TrackingMode::Cursor
        );
    }

    #[test]
    fn color_filter_config_default_filter_none() {
        assert_eq!(
            ColorFilterConfig::default().filter_type,
            ColorFilterType::None
        );
    }

    #[test]
    fn speech_settings_default_disabled() {
        assert!(!SpeechSettings::default().enabled);
    }

    // T004b tests -- AppSettings defaults

    #[test]
    fn app_settings_default_zoom_level() {
        let s = AppSettings::default();
        assert!(
            (s.magnification.zoom_level - 2.0).abs() < f32::EPSILON,
            "expected zoom_level 2.0, got {}",
            s.magnification.zoom_level
        );
    }

    #[test]
    fn app_settings_default_mode_fullscreen() {
        assert_eq!(
            AppSettings::default().magnification.mode,
            MagnificationMode::FullScreen
        );
    }

    #[test]
    fn app_settings_default_tracking_cursor() {
        assert_eq!(
            AppSettings::default().magnification.tracking_mode,
            TrackingMode::Cursor
        );
    }

    #[test]
    fn app_settings_default_color_filter_none() {
        assert_eq!(
            AppSettings::default().color_filter.filter_type,
            ColorFilterType::None
        );
    }

    #[test]
    fn app_settings_default_speech_disabled() {
        assert!(!AppSettings::default().speech.enabled);
    }

    // T004b tests -- serde roundtrips

    #[test]
    fn app_settings_toml_roundtrip() {
        let original = AppSettings::default();
        let toml_str = toml::to_string(&original).unwrap();
        let back: AppSettings = toml::from_str(&toml_str).unwrap();
        assert_eq!(original, back);
    }

    #[test]
    fn app_settings_json_roundtrip() {
        let original = AppSettings::default();
        let json_str = serde_json::to_string(&original).unwrap();
        let back: AppSettings = serde_json::from_str(&json_str).unwrap();
        assert_eq!(original, back);
    }

    #[test]
    fn app_settings_nondefault_toml_roundtrip() {
        let mut keybindings = HashMap::new();
        keybindings.insert(
            HotkeyAction::ZoomIn,
            Some(KeyBinding {
                key: "Equal".to_string(),
                modifiers: vec![ModifierKey::Ctrl],
            }),
        );
        keybindings.insert(
            HotkeyAction::StopSpeech,
            Some(KeyBinding {
                key: "Escape".to_string(),
                modifiers: vec![ModifierKey::Ctrl, ModifierKey::Shift],
            }),
        );

        let settings = AppSettings {
            magnification: MagnificationSettings {
                zoom_level: 5.0,
                mode: MagnificationMode::Docked,
                tracking_mode: TrackingMode::Focus,
                docked_edge: Some(DockEdge::Bottom),
                docked_size_percent: Some(40),
                lens_width: Some(400),
                lens_height: Some(300),
                lens_shape: Some(LensShape::Ellipse),
                target_fps: 30,
                present_mode: PresentMode::LowLatency,
                gpu_preference: GpuPreference::HighPerformance,
                interpolation: InterpolationMode::Bicubic,
                smooth_scrolling: false,
            },
            color_filter: ColorFilterConfig {
                filter_type: ColorFilterType::SmartInvert,
                brightness: 0.3,
                contrast: 1.5,
                color_matrix: Some([
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ]),
            },
            cursor: CursorConfig {
                enlarged_cursor: true,
                cursor_scale: 3.0,
                crosshairs_enabled: true,
                crosshair_width: 5,
                crosshair_color: "#00ff00".to_string(),
                halo_enabled: true,
                halo_radius: 100,
                halo_color: "#0000ff80".to_string(),
            },
            speech: SpeechSettings {
                enabled: true,
                voice_id: "kokoro-af_heart".to_string(),
                speech_rate: 1.5,
                speech_volume: 0.8,
                model_variant: ModelVariant::Fp16,
            },
            keybindings,
            start_on_login: true,
            minimize_to_tray: false,
            show_panel_on_start: false,
        };

        let toml_str = toml::to_string(&settings).unwrap();
        let back: AppSettings = toml::from_str(&toml_str).unwrap();
        assert_eq!(settings, back);
    }
}
