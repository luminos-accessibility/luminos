# Design: Story E01/002 -- Platform Trait Definitions & Common Types

**Story:** [STORY.md](./STORY.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** Principal Architect Agent
**Risk Refs:** [RISK-003](../../tech-strategy/10-risk-register.md#risk-003-platform-trait-surface-area-inadequacy) (trait surface area inadequacy), [RISK-017](../../tech-strategy/10-risk-register.md#risk-017-screen-content-and-tts-text-leakage-via-logs-and-gpu-memory) (screen content leakage)

---

## Overview

This design translates the canonical platform abstraction trait specifications from [doc-02](../../tech-strategy/02-platform-abstraction.md) into compilable Rust code in the `luminos-platform` crate. The approach is straightforward: copy the exact type signatures from doc-02 Sections 3.1--3.7 into source files organized by concern (types, traits, errors), establish the `lib.rs` module structure with `#[cfg]`-gated platform stubs, and add co-located test generators for common types.

This is the most critical story in E1 because every subsequent story (003 mocks, 004 core types, and all epics E2--E20) depends on these trait definitions compiling correctly. The design prioritizes **exact fidelity to doc-02 signatures** over any local optimization. Deviations from doc-02 require explicit justification.

**RISK-017 mitigation:** `CaptureFrame` receives a custom `Debug` implementation that omits raw pixel data, printing only metadata (width, height, stride, format, data length). This prevents accidental screen content leakage in log output.

**RISK-003 awareness:** These trait definitions are designed from research, not implementation. Per the risk mitigation strategy, they are treated as living contracts that may be revised when backends are implemented in E2+. The current definitions are the baseline.

## Architecture

### Component Diagram

```
crates/luminos-platform/src/
  |
  +-- lib.rs                     # Module declarations, re-exports, PlatformBackends
  |
  +-- traits/
  |     +-- mod.rs               # Re-exports all sub-modules
  |     +-- types.rs             # ScreenRect, ScreenPoint, DisplayInfo, PixelFormat, CaptureFrame
  |     +-- screen_capture.rs    # ScreenCapture trait, DisplayChangeEvent, CaptureError
  |     +-- focus_tracker.rs     # FocusTracker trait, FocusChangedEvent, ElementType, FocusError
  |     +-- tts_engine.rs        # TtsEngine trait, Voice, TtsBackend, TtsError
  |     +-- window_manager.rs    # WindowManager trait, OverlayMode, DockEdge, LensShape, WindowError
  |     +-- input_monitor.rs     # InputMonitor trait, InputEvent, MouseButton, KeyCode, Modifiers, InputError
  |     +-- audio_output.rs      # AudioOutput trait, AudioSample, AudioError
  |
  +-- error.rs                   # Re-exports all error types from traits/ sub-modules
  |
  +-- mock/                      # (Story 003 -- empty stub for now)
  |     +-- mod.rs
  |
  +-- common/                    # #[cfg(any(target_os = "linux", target_os = "openbsd"))]
  |     +-- mod.rs               # Empty stub
  |
  +-- linux_x11/mod.rs           # Empty stub, #[cfg(target_os = "linux")]
  +-- linux_wayland/mod.rs       # Empty stub, #[cfg(target_os = "linux")]
  +-- macos/mod.rs               # Empty stub, #[cfg(target_os = "macos")]
  +-- openbsd/mod.rs             # Empty stub, #[cfg(target_os = "openbsd")]
  +-- windows/mod.rs             # Empty stub, #[cfg(target_os = "windows")]
```

**Rationale for `traits/` as a module directory** (vs a single `traits.rs` file): The combined trait definitions, associated types, error enums, and doc-comments exceed 800 lines. A module directory with one file per trait improves navigability and enables parallel agent work on individual trait files. Doc-02 Section 3 already groups definitions by trait, making the mapping natural.

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `traits::types` | New | Common types: ScreenRect, ScreenPoint, DisplayInfo, PixelFormat, CaptureFrame |
| `traits::screen_capture` | New | ScreenCapture trait + DisplayChangeEvent + CaptureError |
| `traits::focus_tracker` | New | FocusTracker trait + FocusChangedEvent + ElementType + FocusError |
| `traits::tts_engine` | New | TtsEngine trait + Voice + TtsBackend + TtsError |
| `traits::window_manager` | New | WindowManager trait + OverlayMode + DockEdge + LensShape + WindowError |
| `traits::input_monitor` | New | InputMonitor trait + InputEvent + MouseButton + KeyCode + Modifiers + InputError |
| `traits::audio_output` | New | AudioOutput trait + AudioSample + AudioError |
| `error` | New | Re-exports all error types for convenience |
| `lib.rs` | New | Module structure with #[cfg] gates, PlatformBackends struct |

### Data Flow

No runtime data flow exists in this story -- it defines only types and trait signatures. The data flow described in doc-02 Section 5.3 (capture -> GPU -> render) is enabled by these trait definitions but implemented in E2+.

The compile-time flow is:
1. `traits/types.rs` defines shared types (ScreenRect, etc.) used by all trait files
2. Each `traits/<trait>.rs` file imports from `types.rs` and defines its trait + error + associated types
3. `traits/mod.rs` re-exports everything publicly
4. `error.rs` re-exports all error types for convenience imports
5. `lib.rs` re-exports `traits` and `error`, declares `#[cfg]`-gated platform stubs
6. `PlatformBackends` in `lib.rs` bundles five trait objects

---

## API Design

All type signatures below are copied from [doc-02 Sections 3.1--3.7](../../tech-strategy/02-platform-abstraction.md#3-trait-definitions). An implementing agent should write code directly from these signatures.

### `traits/types.rs` -- Common Types

```rust
use std::fmt;
use std::sync::Arc;

/// A rectangle in screen coordinates (pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// A point in screen coordinates (pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}

/// Information about a connected display.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfo {
    /// Unique identifier for this display (platform-specific format).
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Display bounds in virtual screen coordinates.
    pub bounds: ScreenRect,
    /// Scale factor (e.g., 2.0 for HiDPI/Retina).
    pub scale_factor: f64,
    /// Whether this is the primary display.
    pub is_primary: bool,
}

/// Pixel format of captured frame data.
///
/// All pixel data is assumed to be in **sRGB color space** (nonlinear,
/// gamma-encoded). The GPU rendering pipeline performs gamma-correct
/// resampling by converting to linear space before interpolation and
/// back to sRGB for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// Blue, Green, Red, Alpha (8 bits each). Native format on X11 and Windows.
    Bgra8,
    /// Red, Green, Blue, Alpha (8 bits each). Native format on macOS (ScreenCaptureKit).
    Rgba8,
}

/// A captured frame of screen content.
///
/// **Privacy:** This struct contains raw screen pixels that may include
/// sensitive content (passwords, banking, medical records). The custom
/// `Debug` implementation intentionally omits the `data` field to prevent
/// accidental leakage in log output. See RISK-017.
#[derive(Clone)]
pub struct CaptureFrame {
    /// Raw pixel data in row-major order, top-left origin.
    pub data: Arc<[u8]>,
    /// Width of the captured frame in pixels.
    pub width: u32,
    /// Height of the captured frame in pixels.
    pub height: u32,
    /// Bytes per row (may include padding).
    pub stride: u32,
    /// Pixel format of the data.
    pub format: PixelFormat,
}

/// Custom Debug impl for CaptureFrame that omits pixel data (RISK-017).
///
/// Prints metadata only: width, height, stride, format, and the byte
/// length of the data buffer. Never prints raw pixel content.
impl fmt::Debug for CaptureFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CaptureFrame")
            .field("data", &format_args!("[<{} bytes>]", self.data.len()))
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .field("format", &self.format)
            .finish()
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    /// Generates a test `CaptureFrame` with solid-color BGRA pixel data.
    ///
    /// # Parameters
    /// - `width`: Frame width in pixels.
    /// - `height`: Frame height in pixels.
    /// - `color`: BGRA color value `[b, g, r, a]` for every pixel.
    pub fn generate_test_capture_frame(
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> CaptureFrame {
        let stride = width * 4;
        let data: Vec<u8> = color
            .iter()
            .cycle()
            .take((stride * height) as usize)
            .copied()
            .collect();
        CaptureFrame {
            data: data.into(),
            width,
            height,
            stride,
            format: PixelFormat::Bgra8,
        }
    }

    /// Generates a test `DisplayInfo` with configurable parameters.
    pub fn generate_test_display_info(
        id: &str,
        width: u32,
        height: u32,
        is_primary: bool,
    ) -> DisplayInfo {
        DisplayInfo {
            id: id.to_string(),
            name: format!("Test Display {id}"),
            bounds: ScreenRect { x: 0, y: 0, width, height },
            scale_factor: 1.0,
            is_primary,
        }
    }
}
```

### `traits/screen_capture.rs` -- ScreenCapture Trait

```rust
use tokio::sync::mpsc;
use super::types::{CaptureFrame, DisplayInfo, ScreenRect};

/// A display configuration change event.
#[derive(Debug, Clone)]
pub enum DisplayChangeEvent {
    /// A new display was connected.
    Connected(DisplayInfo),
    /// A display was disconnected. Contains the display ID.
    Disconnected(String),
    /// A display's configuration changed (resolution, scale, position).
    Reconfigured(DisplayInfo),
}

/// Errors that can occur during screen capture.
#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    /// The requested display was not found.
    #[error("display not found: '{0}'")]
    DisplayNotFound(String),

    /// The requested region is outside the display bounds.
    #[error("capture region {region:?} exceeds display bounds {bounds:?}")]
    RegionOutOfBounds {
        region: ScreenRect,
        bounds: ScreenRect,
    },

    /// The user denied the required screen capture permission.
    #[error("screen capture permission denied")]
    PermissionDenied,

    /// The capture backend is not available on this system.
    #[error("capture backend unavailable: {reason}")]
    BackendUnavailable { reason: String },

    /// A platform-specific error occurred.
    #[error("platform capture error: {message}")]
    Platform {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Screen capture abstraction.
///
/// Implementations capture screen content as CPU pixel buffers.
/// The capture is synchronous and blocking -- it completes within a single
/// frame budget (target: <8ms). The caller (the rendering pipeline) drives
/// the capture cadence.
///
/// # Platform Implementations
///
/// | Platform | Struct | Mechanism |
/// |----------|--------|-----------|
/// | Linux X11 | `XcbCapture` | xcap via XCB (`xcb_get_image`); XShm planned Phase 1 |
/// | Linux Wayland | `PipeWireCapture` | PipeWire + XDG Desktop Portal |
/// | macOS | `SCKitCapture` | ScreenCaptureKit via xcap |
/// | OpenBSD | `XcbCapture` | Shared with Linux X11 (xenocara) |
/// | Windows | `DxgiCapture` | DXGI Desktop Duplication via windows-capture |
pub trait ScreenCapture: Send + Sync {
    /// Lists all connected displays.
    ///
    /// Returns display metadata including bounds and scale factor.
    /// Used during initialization and when displays are added/removed.
    fn list_displays(&self) -> Result<Vec<DisplayInfo>, CaptureError>;

    /// Captures a rectangular region of the specified display.
    ///
    /// The `region` is in display-local coordinates (relative to the display's
    /// top-left corner). If `region` is `None`, captures the entire display.
    ///
    /// This is the hot-path method called every frame (up to 60fps).
    /// Implementations must target <8ms for the source region sizes typical
    /// in magnification (small regions at high zoom).
    fn capture_frame(
        &self,
        display_id: &str,
        region: Option<ScreenRect>,
    ) -> Result<CaptureFrame, CaptureError>;

    /// Subscribes to display configuration change events (hot-plug,
    /// resolution change, scale factor change).
    ///
    /// Returns a receiver that emits events when displays are connected,
    /// disconnected, or reconfigured. The core engine uses these events
    /// to refresh the display list and reposition the overlay.
    ///
    /// Returns `Err` if the platform does not support display change
    /// notifications (graceful degradation: the engine can poll
    /// `list_displays()` periodically as a fallback).
    fn subscribe_display_changes(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<DisplayChangeEvent>, CaptureError>;
}
```

### `traits/focus_tracker.rs` -- FocusTracker Trait

```rust
use tokio::sync::mpsc;
use super::types::ScreenRect;

/// The type of UI element that received focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementType {
    /// A text input field or text area.
    TextInput,
    /// A button, checkbox, radio button, or similar control.
    Control,
    /// A menu or menu item.
    Menu,
    /// A list item, tree item, or table cell.
    ListItem,
    /// A hyperlink.
    Link,
    /// An element type not specifically categorized.
    Other(String),
}

/// A focus change event from the accessibility API.
#[derive(Debug, Clone)]
pub struct FocusChangedEvent {
    /// Platform-specific identifier for the focused element.
    pub element_id: String,
    /// Screen-coordinate bounds of the focused element.
    pub bounds: ScreenRect,
    /// The semantic type of the focused element.
    pub element_type: ElementType,
    /// The accessible name/label of the element, if available.
    pub label: Option<String>,
    /// The PID of the application owning the focused element.
    pub pid: Option<u32>,
}

/// Errors that can occur during focus tracking.
#[derive(Debug, thiserror::Error)]
pub enum FocusError {
    /// The accessibility API is not available on this platform.
    #[error("accessibility API unavailable: {reason}")]
    ApiUnavailable { reason: String },

    /// The required accessibility permission was not granted.
    #[error("accessibility permission denied")]
    PermissionDenied,

    /// The focused element could not be queried (e.g., application crashed).
    #[error("failed to query focused element: {message}")]
    QueryFailed { message: String },

    /// The accessibility bus or service disconnected.
    #[error("accessibility service disconnected: {message}")]
    Disconnected { message: String },

    /// A platform-specific error occurred.
    #[error("platform focus error: {message}")]
    Platform { message: String },
}

/// Keyboard focus tracking via platform accessibility APIs.
///
/// Focus tracking is inherently event-driven and asynchronous (events arrive
/// from D-Bus, the Accessibility API, or UI Automation at unpredictable times).
/// The `subscribe_focus_changes` method returns a channel receiver; the
/// implementation runs an event loop internally.
///
/// # Platform Implementations
///
/// | Platform | Struct | Mechanism |
/// |----------|--------|-----------|
/// | Linux X11 | `AtSpiTracker` | AT-SPI2 via D-Bus (`atspi` crate) |
/// | Linux Wayland | `AtSpiTracker` | Same (AT-SPI2 is display-protocol-independent) |
/// | macOS | `AxTracker` | AXUIElement + AXObserver (`objc2` crate) |
/// | OpenBSD | `MouseFallbackTracker` | No AT-SPI2 in base; mouse position only |
/// | Windows | `UiaTracker` | UI Automation (`windows` crate) |
pub trait FocusTracker: Send + Sync {
    /// Begins monitoring focus changes and returns a receiver for events.
    ///
    /// The implementation spawns an internal task that listens for
    /// accessibility events and sends `FocusChangedEvent` values to the
    /// returned channel. The channel is bounded (`buffer_size` capacity).
    ///
    /// Calling this method multiple times is idempotent; subsequent calls
    /// return new receivers attached to the same internal event source.
    fn subscribe_focus_changes(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<FocusChangedEvent>, FocusError>;

    /// Queries the currently focused element synchronously.
    ///
    /// Returns `None` if no element has focus or if the accessibility API
    /// cannot determine the focused element.
    fn get_focused_element(&self) -> Result<Option<FocusChangedEvent>, FocusError>;

    /// Returns the screen-coordinate bounds of a previously identified element.
    ///
    /// The `element_id` is the platform-specific identifier from a prior
    /// `FocusChangedEvent`. Returns `None` if the element no longer exists
    /// or its bounds cannot be determined.
    fn get_element_bounds(
        &self,
        element_id: &str,
    ) -> Result<Option<ScreenRect>, FocusError>;
}
```

### `traits/tts_engine.rs` -- TtsEngine Trait

```rust
use std::future::Future;
use std::pin::Pin;

/// Metadata about an available TTS voice.
#[derive(Debug, Clone)]
pub struct Voice {
    /// Unique identifier for this voice (e.g., "kokoro-af_heart", "system-en-us-jenny").
    pub id: String,
    /// Human-readable name (e.g., "Heart (American English)").
    pub name: String,
    /// BCP 47 language tag (e.g., "en-US", "ja-JP").
    pub language: String,
    /// Whether this voice requires a model download before use.
    pub requires_download: bool,
    /// The engine providing this voice.
    pub engine: TtsBackend,
}

/// Identifies which TTS engine provides a voice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TtsBackend {
    /// Kokoro model via sherpa-onnx runtime.
    Kokoro,
    /// Piper VITS model via sherpa-onnx runtime (language breadth fallback).
    Piper,
    /// Platform-native TTS (AVSpeech, SAPI, speech-dispatcher).
    Native,
}

/// Errors that can occur during TTS operations.
#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    /// The requested voice is not installed or available.
    #[error("voice not found: '{0}'")]
    VoiceNotFound(String),

    /// The TTS model failed to load (corrupted, missing, incompatible).
    #[error("model load failed: {message}")]
    ModelLoadFailed { message: String },

    /// The phonemizer (espeak-ng subprocess) failed.
    #[error("phonemizer error: {message}")]
    PhonemizerFailed { message: String },

    /// The inference engine returned an error.
    #[error("inference error: {message}")]
    InferenceFailed { message: String },

    /// The audio output device is unavailable.
    #[error("audio output unavailable: {message}")]
    AudioUnavailable { message: String },

    /// A platform-specific error occurred.
    #[error("platform TTS error: {message}")]
    Platform { message: String },
}

/// Text-to-speech engine abstraction.
///
/// The `speak` method is async because TTS involves I/O-bound work:
/// subprocess communication with espeak-ng for phonemization, ONNX model
/// inference, and audio buffer playback. The method returns when audio
/// playback begins (not when it completes).
///
/// # Object Safety
///
/// This trait is **object-safe** (`dyn TtsEngine` is supported). The
/// `speak` method returns a boxed future rather than using RPITIT
/// (`-> impl Future`) to preserve object safety.
pub trait TtsEngine: Send + Sync {
    /// Speaks the given text asynchronously.
    ///
    /// Returns when audio playback begins. Target: <200ms from call to first audio.
    ///
    /// If `interrupt` is `true`, stops current speech and begins new speech.
    /// If `false`, queues after current speech completes.
    fn speak(
        &self,
        text: &str,
        interrupt: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), TtsError>> + Send + '_>>;

    /// Stops any speech currently in progress or queued.
    fn stop(&self) -> Result<(), TtsError>;

    /// Sets the active voice by its ID.
    fn set_voice(&self, voice_id: &str) -> Result<(), TtsError>;

    /// Sets the speech rate. 1.0 = normal, clamped to [0.25, 4.0].
    fn set_rate(&self, rate: f32) -> Result<(), TtsError>;

    /// Sets the speech pitch. 1.0 = normal, clamped to [0.5, 2.0].
    fn set_pitch(&self, pitch: f32) -> Result<(), TtsError>;

    /// Returns all available voices across all engines.
    fn get_voices(&self) -> Result<Vec<Voice>, TtsError>;

    /// Returns `true` if speech is currently being played.
    fn is_speaking(&self) -> bool;
}
```

### `traits/window_manager.rs` -- WindowManager Trait

```rust
use super::types::ScreenRect;

/// The edge of the screen where a docked overlay attaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockEdge {
    Top,
    Bottom,
    Left,
    Right,
}

/// The shape of a lens-mode overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LensShape {
    Rectangle,
    Ellipse,
}

/// The magnification overlay display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayMode {
    /// The overlay covers the entire display.
    FullScreen,
    /// A movable lens that follows the cursor.
    Lens {
        /// Width of the lens in pixels.
        width: u32,
        /// Height of the lens in pixels.
        height: u32,
        /// Shape of the lens boundary.
        shape: LensShape,
    },
    /// The overlay is docked to one edge of the screen.
    Docked {
        /// Which screen edge to dock against.
        edge: DockEdge,
        /// Size of the docked region in pixels (perpendicular to the edge).
        size_px: u32,
    },
}

/// Errors that can occur during window management.
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    /// The window could not be created.
    #[error("window creation failed: {message}")]
    CreationFailed { message: String },

    /// A window property could not be set.
    #[error("failed to set window property '{property}': {message}")]
    PropertyFailed { property: String, message: String },

    /// The requested display for the overlay was not found.
    #[error("target display not found: '{0}'")]
    DisplayNotFound(String),

    /// Platform-specific dock/strut reservation failed.
    #[error("dock reservation failed: {message}")]
    DockFailed { message: String },

    /// A platform-specific error occurred.
    #[error("platform window error: {message}")]
    Platform { message: String },
}

/// Magnification overlay window management.
///
/// This trait controls the winit-based magnification overlay window.
/// The overlay is independent of the Tauri control panel -- it is a native
/// window with wgpu rendering, transparent, borderless, and always-on-top.
///
/// # Platform Implementations
///
/// | Platform | Struct | Dock Mechanism |
/// |----------|--------|----------------|
/// | Linux X11 | `X11WindowManager` | EWMH `_NET_WM_STRUT_PARTIAL` |
/// | Linux Wayland | `WaylandWindowManager` | Layer-shell protocol |
/// | macOS | `CocoaWindowManager` | Floating NSPanel (no reservation) |
/// | OpenBSD | `X11WindowManager` | Shared with Linux X11 (EWMH) |
/// | Windows | `Win32WindowManager` | `SHAppBarMessage` / AppBar API |
pub trait WindowManager: Send + Sync {
    /// Creates the magnification overlay window on the specified display.
    fn create_overlay(&mut self, display_id: &str) -> Result<(), WindowError>;

    /// Sets the overlay's position and size in screen coordinates.
    fn set_overlay_bounds(&self, bounds: ScreenRect) -> Result<(), WindowError>;

    /// Switches the overlay to the specified display mode.
    fn set_overlay_mode(&mut self, mode: OverlayMode) -> Result<(), WindowError>;

    /// Sets whether the overlay is always above other windows.
    fn set_always_on_top(&self, always_on_top: bool) -> Result<(), WindowError>;

    /// Shows or hides the overlay window.
    fn set_visible(&self, visible: bool) -> Result<(), WindowError>;

    /// Returns the raw window handle for wgpu surface creation.
    /// Returns `None` if the overlay has not been created yet.
    fn raw_window_handle(&self) -> Option<&dyn raw_window_handle::HasWindowHandle>;

    /// Returns the raw display handle for wgpu surface creation.
    fn raw_display_handle(&self) -> Option<&dyn raw_window_handle::HasDisplayHandle>;
}
```

### `traits/input_monitor.rs` -- InputMonitor Trait

```rust
use tokio::sync::mpsc;
use super::types::ScreenPoint;

/// Keyboard modifier state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    /// "Super" on Linux, "Cmd" on macOS, "Win" on Windows.
    pub meta: bool,
}

/// A global input event.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// The mouse pointer moved to a new position.
    MouseMoved {
        /// Absolute screen-coordinate position.
        position: ScreenPoint,
    },
    /// A mouse button was pressed or released.
    MouseButton {
        /// The button that changed state.
        button: MouseButton,
        /// `true` if pressed, `false` if released.
        pressed: bool,
        /// Current pointer position.
        position: ScreenPoint,
    },
    /// The scroll wheel was moved.
    Scroll {
        /// Horizontal scroll delta (positive = right).
        delta_x: f64,
        /// Vertical scroll delta (positive = down).
        delta_y: f64,
        /// Current pointer position.
        position: ScreenPoint,
    },
    /// A keyboard key was pressed or released.
    KeyEvent {
        /// Platform-independent key code.
        code: KeyCode,
        /// `true` if pressed, `false` if released.
        pressed: bool,
        /// Active modifier keys.
        modifiers: Modifiers,
    },
}

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

/// Platform-independent key code.
///
/// A simplified subset covering keys used for Luminos shortcuts.
/// A full keycode mapping is deferred to the input backend implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Alphanumeric
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Navigation
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown,
    // Modifiers (as standalone key events)
    ShiftLeft, ShiftRight,
    CtrlLeft, CtrlRight,
    AltLeft, AltRight,
    MetaLeft, MetaRight,
    // Common
    Space, Enter, Escape, Tab, Backspace, Delete,
    // Punctuation used in shortcuts
    Plus, Minus, Equal,
    BracketLeft, BracketRight,
    // Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide,
    // Catch-all for keys not in this enum
    Unknown(u32),
}

/// Errors that can occur during input monitoring.
#[derive(Debug, thiserror::Error)]
pub enum InputError {
    /// Global input monitoring is not available (e.g., permission denied).
    #[error("input monitoring unavailable: {reason}")]
    Unavailable { reason: String },

    /// The input monitoring backend disconnected unexpectedly.
    #[error("input monitor disconnected: {message}")]
    Disconnected { message: String },

    /// A platform-specific error occurred.
    #[error("platform input error: {message}")]
    Platform { message: String },
}

/// Global input event monitoring.
///
/// Monitors mouse movement, clicks, scroll events, and keyboard events
/// globally (across all applications, not just when Luminos has focus).
/// This is essential for cursor-follow magnification.
///
/// # Platform Considerations
///
/// | Platform | Primary | Fallback |
/// |----------|---------|----------|
/// | Linux X11 | rdev | XInput2 / XRecord extension |
/// | Linux Wayland | rdev (evdev) | libinput (requires permissions) |
/// | macOS | rdev | CGEvent tap (requires Accessibility permission) |
/// | OpenBSD | rdev | XInput2 / XRecord |
/// | Windows | rdev | Raw Input / Low-level hooks |
pub trait InputMonitor: Send + Sync {
    /// Begins monitoring input events and returns a receiver.
    ///
    /// The implementation spawns an internal event loop that captures
    /// global input events and sends them to the returned channel.
    /// The channel is bounded (`buffer_size` capacity).
    fn subscribe_input_events(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<InputEvent>, InputError>;

    /// Returns the current mouse pointer position.
    fn get_mouse_position(&self) -> Result<ScreenPoint, InputError>;
}
```

### `traits/audio_output.rs` -- AudioOutput Trait

```rust
/// An audio sample buffer ready for playback.
#[derive(Debug, Clone)]
pub struct AudioSample {
    /// PCM audio data (f32 samples, mono or interleaved stereo).
    pub data: Vec<f32>,
    /// Sample rate in Hz (e.g., 24000 for Kokoro output).
    pub sample_rate: u32,
    /// Number of audio channels (1 = mono, 2 = stereo).
    pub channels: u16,
}

/// Errors that can occur during audio output.
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    /// No audio output device was found.
    #[error("no audio output device found")]
    NoDevice,

    /// The audio device failed to open or initialize.
    #[error("audio device error: {message}")]
    DeviceFailed { message: String },

    /// Audio playback was interrupted by a device change or error.
    #[error("playback interrupted: {message}")]
    PlaybackInterrupted { message: String },

    /// The sample format is unsupported by the output device.
    #[error("unsupported audio format: {message}")]
    UnsupportedFormat { message: String },

    /// A platform-specific error occurred.
    #[error("platform audio error: {message}")]
    Platform { message: String },
}

/// Audio output for TTS playback.
///
/// Wraps the `cpal` crate to provide a simplified interface for queuing
/// and playing audio samples generated by the TTS engine.
///
/// # Platform Backends
///
/// | Platform | cpal Backend |
/// |----------|-------------|
/// | Linux | ALSA or PulseAudio (cpal auto-selects) |
/// | macOS | CoreAudio |
/// | OpenBSD | sndio |
/// | Windows | WASAPI |
pub trait AudioOutput: Send + Sync {
    /// Plays the given audio sample.
    ///
    /// If `interrupt` is `true`, stops current playback first.
    fn play_audio(
        &self,
        sample: AudioSample,
        interrupt: bool,
    ) -> Result<(), AudioError>;

    /// Stops any audio currently playing.
    fn stop_audio(&self) -> Result<(), AudioError>;

    /// Sets the output volume. Linear scale 0.0 (silent) to 1.0 (full).
    fn set_volume(&self, volume: f32) -> Result<(), AudioError>;

    /// Returns the name of the default audio output device, if available.
    fn get_default_device_name(&self) -> Result<Option<String>, AudioError>;
}
```

### `traits/mod.rs` -- Re-exports

```rust
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
```

### `error.rs` -- Error Re-exports

```rust
//! Convenience re-exports for all platform error types.
//!
//! Consumers can import errors via `use luminos_platform::error::*`
//! instead of navigating individual trait modules.

pub use crate::traits::screen_capture::CaptureError;
pub use crate::traits::focus_tracker::FocusError;
pub use crate::traits::tts_engine::TtsError;
pub use crate::traits::window_manager::WindowError;
pub use crate::traits::input_monitor::InputError;
pub use crate::traits::audio_output::AudioError;
```

### `lib.rs` -- Module Structure and PlatformBackends

```rust
//! luminos-platform: Platform abstraction layer for Luminos.
//!
//! Defines six platform traits and their associated types. Platform-specific
//! backends implement these traits; the core engine programs against the
//! trait interfaces exclusively.

pub mod traits;
pub mod error;

#[cfg(any(test, feature = "test_utils"))]
pub mod mock;

// Shared code used by multiple platforms (Linux + OpenBSD).
#[cfg(any(target_os = "linux", target_os = "openbsd"))]
pub(crate) mod common;

// Platform backends -- only the relevant platform is compiled.
// On Linux, BOTH x11 and wayland modules are compiled; runtime selection
// chooses the active backend (see doc-02 Section 5.3).

#[cfg(target_os = "linux")]
mod linux_x11;

#[cfg(target_os = "linux")]
mod linux_wayland;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "openbsd")]
mod openbsd;

#[cfg(target_os = "windows")]
mod windows;

use traits::{AudioOutput, FocusTracker, InputMonitor, ScreenCapture, WindowManager};

/// Bundle of all platform-specific trait implementations.
///
/// Created once at application startup by the platform factory function.
/// The core engine receives this and programs against the trait interfaces.
///
/// `TtsEngine` is excluded because it is constructed separately by the
/// `luminos-tts` crate (it depends on `AudioOutput` + espeak-ng subprocess,
/// not on platform APIs directly).
pub struct PlatformBackends {
    pub capture: Box<dyn ScreenCapture>,
    pub focus_tracker: Box<dyn FocusTracker>,
    pub window_mgr: Box<dyn WindowManager>,
    pub input_monitor: Box<dyn InputMonitor>,
    pub audio_output: Box<dyn AudioOutput>,
}
```

---

## Error Handling

All six error enums derive `thiserror::Error` for automatic `Display` and `std::error::Error` implementations. This enables `?` propagation to the top-level `LuminosError` (defined in Story 004) via `From` trait conversions.

**Design rules applied** (from doc-02 Section 4.3):
1. Common variants cover platform-agnostic errors (e.g., `PermissionDenied`, `VoiceNotFound`)
2. Every enum has a `Platform { message: String }` variant as an escape hatch for OS-specific errors
3. `CaptureError::Platform` additionally carries an optional `source` for error chain support
4. All variants include enough context to produce actionable log messages

**No `unwrap()` or `expect()` in production code** (NFR-2). The only exception is inside `#[cfg(test)]` blocks.

---

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | Primary dev target | Empty `linux_x11/mod.rs` stub compiled via `#[cfg(target_os = "linux")]` |
| Linux Wayland | Compiled alongside X11 on Linux | Empty `linux_wayland/mod.rs` stub; `wayland` feature controls `ashpd` dep (Story 001) |
| macOS | Empty stub | `macos/mod.rs` compiled via `#[cfg(target_os = "macos")]` |
| OpenBSD | Shares `common/` with Linux | `openbsd/mod.rs` + `common/mod.rs` both compiled via `#[cfg(target_os = "openbsd")]` / `#[cfg(any(target_os = "linux", target_os = "openbsd"))]` |
| Windows | Empty stub | `windows/mod.rs` compiled via `#[cfg(target_os = "windows")]`; note: module name may shadow `std::os::windows` -- use fully qualified paths if needed |
| All platforms | Trait definitions are unconditional | `traits/` module compiles on every target; only backend modules are gated |

**Important:** Both `linux_x11` and `linux_wayland` modules compile unconditionally on `target_os = "linux"`. There is no Cargo feature gating which module compiles -- only the `wayland` feature controls the `ashpd` dependency. Runtime selection via `XDG_SESSION_TYPE` determines which backend is active (doc-02 Section 5.3).

---

## Testing Strategy

### Unit Tests

Unit tests for this story verify:
1. Type construction and derive attribute correctness
2. `CaptureFrame` custom Debug output (RISK-017)
3. Test generator functions produce correct values
4. Error Display formatting matches `#[error(...)]` attributes
5. All code compiles with zero warnings (`cargo build`, `cargo clippy`)

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Build | `cargo build -p luminos-platform` exits 0 with no errors or warnings |
| AC-1.2 | Inspection + compile | ScreenCapture trait has exactly 3 methods with correct signatures; verified by mock impl in Story 003 compiling against it |
| AC-1.3 | Inspection + compile | FocusTracker trait has exactly 3 methods; verified by mock impl |
| AC-1.4 | Inspection + compile | TtsEngine trait has exactly 7 methods; verified by mock impl |
| AC-1.5 | Inspection + compile | WindowManager trait has exactly 7 methods; verified by mock impl |
| AC-2.1 | Unit | `types_screen_rect_fields_and_derives` -- construct ScreenRect, verify Debug, Clone, Copy, PartialEq, Eq, Hash |
| AC-2.2 | Unit | `types_screen_point_fields_and_derives` -- construct ScreenPoint, verify derives |
| AC-2.3 | Unit | `types_display_info_fields_and_derives` -- construct DisplayInfo, verify Debug, Clone, PartialEq |
| AC-2.4 | Unit | `types_capture_frame_fields` -- construct CaptureFrame, verify fields; `types_pixel_format_derives` -- verify PixelFormat derives |
| AC-2.5 | Unit | `types_audio_sample_fields_and_derives` -- construct AudioSample, verify Debug, Clone |
| AC-2.6 | Unit | `types_voice_fields_and_tts_backend_variants` -- construct Voice, verify TtsBackend variants |
| AC-3.1 | Unit | `error_capture_error_display_*` -- one test per variant verifying Display output |
| AC-3.2 | Unit | `error_focus_error_display_*` -- one test per variant |
| AC-3.3 | Unit | `error_tts_error_display_*` -- one test per variant |
| AC-3.4 | Unit | `error_window_error_display_*` -- one test per variant |
| AC-3.5 | Unit | `error_input_error_display_*` -- one test per variant |
| AC-3.6 | Unit | `error_audio_error_display_*` -- one test per variant |
| AC-4.1 | Inspection | `lib.rs` file review confirms module declarations match doc-02 Section 5.2 |
| AC-4.2 | Build | `cargo build -p luminos-platform` succeeds with empty stubs on current platform |
| AC-4.3 | Unit | `platform_backends_struct_fields` -- construct PlatformBackends with mock objects (requires Story 003; can be a compile-only test) |
| AC-4.4 | Inspection + compile | InputMonitor trait has exactly 2 methods; verified by mock impl |
| AC-4.5 | Inspection + compile | AudioOutput trait has exactly 4 methods; verified by mock impl |
| AC-5.1 | Unit | `types_capture_frame_debug_omits_data` -- format CaptureFrame with `{:?}`, assert output contains "bytes" placeholder, does NOT contain raw pixel data |
| AC-5.2 | Build | `cargo doc -p luminos-platform --no-deps` produces docs without warnings |
| AC-5.3 | Unit | `types_generate_test_capture_frame_correct_output` -- call with (64, 48, [0,0,255,255]), verify width=64, height=48, stride=256, format=Bgra8, data.len()=12288 |
| AC-5.4 | Unit | `types_generate_test_display_info_correct_output` -- call with ("test-0", 1920, 1080, true), verify all fields |

### NFR Verification

| NFR | Verification |
|-----|-------------|
| NFR-1 (Send + Sync) | Compile-time: trait bounds enforce this; mock impls in Story 003 prove it |
| NFR-2 (No unwrap in prod) | `cargo clippy -- -W clippy::unwrap_used -W clippy::expect_used`; manual review |
| NFR-3 (Doc-comments) | `cargo doc -p luminos-platform --no-deps` with zero warnings |
| NFR-4 (Clippy clean) | `cargo clippy -p luminos-platform -- -D warnings` exits 0 |
| NFR-5 (thiserror) | Compile-time: `#[derive(thiserror::Error)]` on all error enums |

---

## Performance Targets

This story defines only types and trait signatures -- no runtime code. There are no performance targets. The traits declare performance expectations in their doc-comments (e.g., `capture_frame` target <8ms, `speak` target <200ms) that will be enforced when backends are implemented in E2+.

---

## Security Considerations

**RISK-017 (Screen content leakage via logs):**
- `CaptureFrame` has a custom `Debug` implementation that prints `data: [<{N} bytes>]` instead of the raw pixel content. This is the primary mitigation for this story.
- The custom Debug impl is tested explicitly (AC-5.1) to ensure it cannot regress.
- `FocusChangedEvent.label` contains accessible names from applications. While not redacted in this story (it is a user-facing label, not raw screen content), the RISK-017 mitigation strategy in doc-10 notes that TTS text redaction will be implemented separately in E10.

---

## Alternatives Considered

### Alternative 1: Single `traits.rs` file vs `traits/` module directory

**Rejected approach:** Put all trait definitions in a single `crates/luminos-platform/src/traits.rs` file.

**Rationale for rejection:** The combined definitions, doc-comments, associated types, and error enums exceed 800 lines. A single file would be unwieldy for navigation, parallel editing, and code review. The module directory approach maps naturally to doc-02's per-section organization (3.1 types, 3.2 ScreenCapture, etc.) and allows agents to work on individual trait files without merge conflicts.

**Trade-off:** The module directory approach adds `mod.rs` boilerplate and requires `use super::types::*` imports in each trait file. This is a minor cost for significantly better organization.

### Alternative 2: Separate error crate vs co-located errors

**Rejected approach:** Define all error types in a separate `luminos-error` crate.

**Rationale for rejection:** Error types are tightly coupled to their trait definitions (e.g., `CaptureError` is the error type of `ScreenCapture` methods). Separating them into a different crate would add a dependency edge and make it harder to maintain coherence between trait signatures and error variants. Doc-02 co-locates errors with their traits. `error.rs` provides convenience re-exports for consumers who want all errors in one import.

### Alternative 3: Generic error type vs per-subsystem enums

**Rejected approach:** Use a single `PlatformError` enum for all six subsystems.

**Rationale for rejection:** A unified error type would require callers to `match` against variants from unrelated subsystems. Per-subsystem enums provide type-safe, narrowly-scoped error handling. The `LuminosError` in `luminos-core` (Story 004) provides the unified type at the application boundary via `From` conversions.
