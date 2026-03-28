# Design: Story E01/004 -- Error Hierarchy & Core Data Types

**Story:** [STORY.md](./STORY.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** Principal Architect Agent
**Risk Refs:** RISK-003 (platform trait surface area inadequacy), RISK-017 (screen content leakage via logs)

---

## Overview

This design defines the top-level `LuminosError` enum, the core application state types (`AppState`, `AppSettings`), and the configuration enums (`MagnificationMode`, `TrackingMode`, `ColorFilterType`) in the `luminos-core` crate. The `LuminosError` enum uses `thiserror` with `#[from]` attributes to provide automatic `From` conversions from all six subsystem error types defined in Story 002's `luminos-platform` crate, enabling `?` propagation across subsystem boundaries per CLAUDE.md conventions.

The `AppSettings` struct mirrors the Zod schema from doc-05 Section 3.2, with nested sub-structs for magnification, color filter, cursor, speech, and keybinding settings. It derives `Serialize`, `Deserialize`, `Clone`, `Debug`, and `PartialEq` for TOML/JSON roundtripping. `AppState` wraps `AppSettings` plus transient runtime fields (viewport position, TTS status, active display ID) and also implements `Default`. These types become the shared source of truth for the render thread, IPC layer, and future configuration manager.

## Architecture

### Component Diagram

```
crates/luminos-core/
  src/
    lib.rs                      # Re-exports: error, state, config
    error.rs                    # LuminosError enum
    state.rs                    # AppState, MagnificationMode, TrackingMode,
    │                           # ColorFilterType, TtsStatus
    config/
      mod.rs                    # Re-exports schema module
      schema.rs                 # AppSettings + sub-structs + Default impl
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `luminos-core/src/error.rs` | New | `LuminosError` enum with `#[from]` conversions |
| `luminos-core/src/state.rs` | New | `AppState`, `MagnificationMode`, `TrackingMode`, `ColorFilterType`, `TtsStatus` |
| `luminos-core/src/config/schema.rs` | New | `AppSettings` and nested config sub-structs |
| `luminos-core/src/config/mod.rs` | New | Module re-exports |
| `luminos-core/src/lib.rs` | Modified | Module declarations and public re-exports |

### Data Flow

```
luminos-platform (Story 002)       luminos-core (this story)
+---------------------------+      +--------------------------------+
| CaptureError              |      | LuminosError                   |
| FocusError                | ---> |   ::Capture(CaptureError)      |
| TtsError                  |  via |   ::Focus(FocusError)          |
| WindowError               | From |   ::Tts(TtsError)              |
| InputError                |      |   ::Window(WindowError)        |
| AudioError                |      |   ::Input(InputError)          |
+---------------------------+      |   ::Audio(AudioError)          |
                                   |   ::Config { message }         |
                                   |   ::Internal { message }       |
                                   +--------------------------------+

AppSettings (persisted)   --embedded-in-->   AppState (runtime)
  .magnification                               .settings: AppSettings
  .color_filter                                .viewport: ScreenRect
  .cursor                                      .tts_status: TtsStatus
  .speech                                      .active_display_id: Option<String>
  .keybindings                                 .is_active: bool
  .start_on_login
  .minimize_to_tray
  .show_panel_on_start
```

## API Design

### LuminosError (crates/luminos-core/src/error.rs)

```rust
use luminos_platform::{
    AudioError, CaptureError, FocusError, InputError, TtsError, WindowError,
};

/// Top-level application error.
///
/// Each variant wraps a subsystem-specific error type. The `From` trait
/// is implemented for each subsystem error via `#[from]`, enabling `?`
/// propagation across subsystem boundaries without explicit matching.
#[derive(Debug, thiserror::Error)]
pub enum LuminosError {
    /// Screen capture subsystem error.
    #[error("screen capture: {0}")]
    Capture(#[from] CaptureError),

    /// Focus tracking subsystem error.
    #[error("focus tracking: {0}")]
    Focus(#[from] FocusError),

    /// Text-to-speech subsystem error.
    #[error("text-to-speech: {0}")]
    Tts(#[from] TtsError),

    /// Window management subsystem error.
    #[error("window management: {0}")]
    Window(#[from] WindowError),

    /// Input monitoring subsystem error.
    #[error("input monitoring: {0}")]
    Input(#[from] InputError),

    /// Audio output subsystem error.
    #[error("audio output: {0}")]
    Audio(#[from] AudioError),

    /// Configuration error (invalid settings, parse failure, I/O).
    #[error("configuration: {message}")]
    Config { message: String },

    /// Internal logic error (invariant violation, unexpected state).
    #[error("internal error: {message}")]
    Internal { message: String },
}
```

### Configuration Enums (crates/luminos-core/src/state.rs)

```rust
use serde::{Deserialize, Serialize};

/// Screen magnification display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MagnificationMode {
    /// Entire screen is magnified; overlay covers full display.
    FullScreen,
    /// Magnified view docked to a screen edge.
    Docked,
    /// Magnifying glass lens follows cursor.
    Lens,
}

/// Which element the magnification viewport tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrackingMode {
    /// Viewport follows the mouse cursor.
    Cursor,
    /// Viewport follows the keyboard focus element.
    Focus,
    /// Viewport follows the text insertion caret.
    TextCaret,
}

/// Color filter applied to the magnified view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
```

### AppSettings (crates/luminos-core/src/config/schema.rs)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::state::{
    ColorFilterType, MagnificationMode, TrackingMode,
};

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

/// Screen edge for docked mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DockEdge {
    Top,
    Bottom,
    Left,
    Right,
}

/// Lens overlay shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LensShape {
    Rectangle,
    Ellipse,
}

/// VSync / frame presentation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PresentMode {
    /// VSync-locked (Fifo), smooth, no tearing.
    Quality,
    /// Mailbox, lowest input latency with GPU overhead.
    LowLatency,
    /// Immediate, uncapped FPS for diagnostics.
    Performance,
}

/// GPU device preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GpuPreference {
    /// Integrated GPU (lower power, default).
    LowPower,
    /// Discrete GPU (higher performance).
    HighPerformance,
}

/// Scaling interpolation algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterpolationMode {
    /// Bilinear filtering (Phase 0 default).
    Bilinear,
    /// Bicubic filtering (Phase 1+, higher quality).
    Bicubic,
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

/// Kokoro ONNX model quantization variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelVariant {
    Q4,
    Q8,
    Fp16,
    Fp32,
}

/// Hotkey action identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HotkeyAction {
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleMagnification,
    CycleMode,
    ReadWhatISee,
    ReadSelection,
    StopSpeech,
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
    Ctrl,
    Shift,
    Alt,
    Super,
    Meta,
}

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
```

### Default Implementations

```rust
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
```

### AppState (crates/luminos-core/src/state.rs)

```rust
use luminos_platform::ScreenRect;
use crate::config::schema::AppSettings;

/// Runtime application state.
///
/// Contains both persisted settings (`AppSettings`) and transient
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
        }
    }
}
```

### Module Re-exports (crates/luminos-core/src/lib.rs)

```rust
//! Luminos core engine library.
//!
//! Provides the application state types, error hierarchy, and
//! settings schema shared by the render thread, TTS pipeline,
//! and control panel IPC layer.

pub mod config;
pub mod error;
pub mod state;

pub use error::LuminosError;
pub use state::{
    AppState, ColorFilterType, MagnificationMode, TrackingMode, TtsStatus,
};
pub use config::schema::AppSettings;
```

## Error Handling

All error handling follows CLAUDE.md conventions:

- **`?` propagation** is the primary mechanism. `LuminosError` implements `From` for all six subsystem error types via `thiserror` `#[from]`, so any function returning `Result<T, LuminosError>` can propagate subsystem errors with `?`.
- **No `unwrap()` or `expect()`** in any production code. Only in `#[cfg(test)]` blocks.
- **`Config` and `Internal` variants** use `{ message: String }` instead of `#[from]` because they don't wrap a specific error type -- they are constructed directly where the error is detected.
- **`Display` formatting** includes the subsystem name as a prefix (e.g., "screen capture: display not found: 'HDMI-1'") so log output is self-describing.

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | N/A | Error types are platform-independent |
| Linux Wayland | N/A | Error types are platform-independent |
| macOS | N/A | Error types are platform-independent |
| OpenBSD | N/A | Error types are platform-independent |
| Windows | N/A | Error types are platform-independent |

All types defined in this story are platform-independent. Platform-specific error variants are defined in `luminos-platform` (Story 002). This story only defines the `From` conversions that bridge them to `LuminosError`.

## Testing Strategy

### Unit Tests

Tests are organized in `#[cfg(test)]` modules within each source file. Test names use hierarchical prefixes for granular `cargo nextest` filtering.

**error.rs tests:**
- `luminos_error_from_capture_error` -- `?` propagation from `CaptureError`
- `luminos_error_from_focus_error` -- `?` propagation from `FocusError`
- `luminos_error_from_tts_error` -- `?` propagation from `TtsError`
- `luminos_error_from_window_error` -- `?` propagation from `WindowError`
- `luminos_error_from_input_error` -- `?` propagation from `InputError`
- `luminos_error_from_audio_error` -- `?` propagation from `AudioError`
- `luminos_error_display_capture` -- `Display` output includes "screen capture"
- `luminos_error_display_config` -- `Display` output includes "configuration"
- `luminos_error_display_internal` -- `Display` output includes "internal error"

**state.rs tests:**
- `magnification_mode_serde_roundtrip` -- PascalCase serialization
- `tracking_mode_serde_roundtrip` -- PascalCase serialization
- `color_filter_type_serde_roundtrip` -- PascalCase serialization
- `tts_status_serde_roundtrip` -- PascalCase serialization
- `app_state_default_settings_match` -- `AppState::default().settings == AppSettings::default()`
- `app_state_default_viewport_at_origin` -- viewport x=0, y=0
- `app_state_default_tts_idle` -- `tts_status == TtsStatus::Idle`
- `app_state_default_not_active` -- `is_active == false`

**config/schema.rs tests:**
- `app_settings_toml_roundtrip` -- serialize to TOML, deserialize back, assert equal
- `app_settings_json_roundtrip` -- serialize to JSON, deserialize back, assert equal
- `app_settings_default_zoom_level` -- default zoom is 2.0
- `app_settings_default_mode_fullscreen` -- default mode is FullScreen
- `app_settings_default_tracking_cursor` -- default tracking is Cursor
- `app_settings_default_color_filter_none` -- default filter is None
- `app_settings_default_speech_disabled` -- default speech.enabled is false
- `app_settings_nondefault_toml_roundtrip` -- set all fields to non-default values, roundtrip

### Integration Tests

No integration tests needed for this story. All types are tested via unit tests.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Unit | `luminos_error_from_capture_error`: call a function returning `Result<(), LuminosError>` that propagates a `CaptureError` via `?`; assert the result is `LuminosError::Capture(_)` |
| AC-1.2 | Unit | `luminos_error_from_focus_error`: same pattern with `FocusError` |
| AC-1.3 | Unit | `luminos_error_from_tts_error`: same pattern with `TtsError` |
| AC-1.4 | Unit | `luminos_error_from_window_error`, `luminos_error_from_input_error`, `luminos_error_from_audio_error`: same pattern for each |
| AC-1.5 | Unit | `luminos_error_display_capture`, `_config`, `_internal`: format with `"{}"` and assert output contains subsystem name and detail |
| AC-2.1 | Unit | `app_settings_toml_roundtrip`: construct `AppSettings` with non-default values, `toml::to_string()`, `toml::from_str()`, assert equal |
| AC-2.2 | Unit | `app_settings_json_roundtrip`: same with `serde_json` |
| AC-2.3 | Unit | `app_settings_default_*` tests: assert each default field matches spec (zoom 2.0, FullScreen, Cursor, None, etc.) |
| AC-3.1 | Unit | `magnification_mode_serde_roundtrip`: serialize each variant, assert PascalCase string, deserialize back |
| AC-3.2 | Unit | `tracking_mode_serde_roundtrip`: same pattern |
| AC-3.3 | Unit | `color_filter_type_serde_roundtrip`: same pattern for all six variants |
| AC-3.4 | Unit | Covered by serde roundtrip tests: assert serialized output is `"FullScreen"` not `"full_screen"` |
| AC-4.1 | Unit | `app_state_default_settings_match`: `AppState::default().settings == AppSettings::default()` |
| AC-4.2 | Unit | `app_state_default_viewport_at_origin` + `app_state_default_tts_idle` + `app_state_default_not_active` |

## Performance Targets

No runtime performance targets apply to this story. These are compile-time data types and static configuration. The only relevant constraint is NFR-1 (zero `unwrap()`/`expect()` in production code), which is enforced by clippy in Story 005 CI.

## Security Considerations

- **RISK-017 (screen content leakage):** `AppState` does not contain pixel data. The `CaptureFrame` type (in `luminos-platform`) has a custom `Debug` implementation that omits the `data` field -- that concern is addressed in Story 002, not here.
- **No sensitive data in settings:** `AppSettings` contains user preferences, not credentials or personal data. TOML serialization is safe for on-disk storage.

## Alternatives Considered

### Alternative: Define error types in luminos-core instead of luminos-platform

**Approach:** Put all error enums (`CaptureError`, `FocusError`, etc.) in `luminos-core` alongside `LuminosError`.

**Rejected because:**
- `luminos-platform` is the foundation crate with zero internal deps. If error types lived in `luminos-core`, then `luminos-platform` would need to depend on `luminos-core`, creating a circular dependency.
- Subsystem errors belong with their subsystem traits. `CaptureError` is semantically part of the `ScreenCapture` trait definition.
- The doc-02 Section 4 architecture places subsystem errors in `luminos-platform` and only the top-level `LuminosError` in `luminos-core`.

### Alternative: Use anyhow instead of thiserror for LuminosError

**Approach:** Use `anyhow::Error` as the top-level error type for simpler error handling.

**Rejected because:**
- `anyhow` erases error types, making it impossible to match on specific variants for recovery logic (e.g., "if capture permission denied, show permission dialog").
- `thiserror` provides `#[from]` for automatic `From` implementations, which is the exact pattern needed for `?` propagation from six distinct error types.
- The project error handling conventions (CLAUDE.md) require typed errors with `From` conversions, not type-erased errors.

### Alternative: Flat AppSettings (no nested sub-structs)

**Approach:** Put all settings fields directly on `AppSettings` instead of nesting into `MagnificationSettings`, `ColorFilterConfig`, etc.

**Rejected because:**
- A flat struct with 30+ fields is harder to read, document, and extend.
- The Zod schema in doc-05 Section 3.2 uses nested objects. The Rust struct should mirror it for type safety across the IPC boundary.
- Nested structs allow per-section `Default` implementations, which are cleaner and more maintainable.
