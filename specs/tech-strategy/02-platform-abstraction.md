# 02 -- Platform Abstraction

**Status:** DRAFT v1.2 (post audit review)
**Date:** 2026-03-15
**Audience:** Engineers, AI agents implementing platform backends
**Source Documents:** [Product Strategy](../PRODUCT_STRATEGY.md) (v1.3, Sections 7-8), [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) (FINAL, Sections 3-5), [System Architecture](./01-system-architecture.md)

---

## 1. Overview

### 1.1 Purpose

The platform abstraction layer is the boundary between Luminos's platform-independent core logic and the platform-specific code that makes it work on Linux X11, Linux Wayland, macOS, OpenBSD, and Windows. It defines six Rust traits -- `ScreenCapture`, `FocusTracker`, `TtsEngine`, `WindowManager`, `InputMonitor`, and `AudioOutput` -- that encode the behaviors every platform must provide. The core engine programs exclusively against these traits, never against platform APIs directly.

### 1.2 Why Traits

Rust traits serve three roles in this architecture:

1. **Compiler-enforced contracts.** An AI agent implementing the macOS `ScreenCapture` backend cannot forget a method or return the wrong type. The Rust compiler rejects non-conforming implementations at compile time.

2. **Parallel development.** Each platform backend is an independent module implementing shared traits. Multiple AI agents (or human contributors) can work on Linux, macOS, OpenBSD, and Windows backends simultaneously without merge conflicts. The trait definition is the coordination point, not the implementation.

3. **Testability.** Mock implementations of every trait can be injected into the core engine for unit testing. The core magnification pipeline, TTS pipeline, and control panel logic can all be tested without a display server, GPU, or audio device.

### 1.3 Relationship to Conditional Compilation

Trait definitions live in a single, platform-independent module (`platform::traits`). Platform backends live in per-platform modules gated by `#[cfg(target_os = "...")]` attributes. On Linux, X11 and Wayland backends are both compiled into the binary and selected at runtime based on the active session type. On all other platforms, backend selection is a compile-time decision.

---

## 2. Design Principles

### 2.1 Trait-First Design

Define behavior, not implementation. Every public interaction with a platform capability goes through a trait. Platform backends are private implementation details -- consuming code never imports `linux_x11::XcbCapture` directly; it receives a `Box<dyn ScreenCapture>` or a generic `T: ScreenCapture`.

### 2.2 Platform Backends Are Independent and Substitutable

Each backend module depends only on the trait definitions and the platform's native APIs. No backend depends on another backend. OpenBSD shares source code with Linux X11 (via `pub use` re-exports or shared utility modules), but the dependency is on shared utility code, not on the Linux backend itself.

### 2.3 Async Where Needed, Sync by Default

Following the project's async discipline (CLAUDE.md): synchronous operations remain synchronous. Async is reserved for operations that genuinely perform I/O, wait on external events, or run in background loops:

- **Async:** `FocusTracker::subscribe_focus_changes` (D-Bus event stream), `InputMonitor::subscribe_input_events` (event loop), `TtsEngine::speak` (I/O-bound inference + audio playback)
- **Sync:** `ScreenCapture::capture_frame` (fast, blocking capture per frame), `WindowManager::set_overlay_bounds` (immediate window property change), `AudioOutput::set_volume` (single syscall)

Methods that return a stream of events use `tokio::sync::mpsc` channels rather than trait-level async iterators, keeping trait definitions compatible with both async and sync callers.

### 2.4 Error Types Use `From` Trait for Conversion

Each subsystem defines its own error enum. All subsystem errors implement `From` conversions to the top-level `LuminosError`, enabling `?` propagation across subsystem boundaries without explicit matching.

### 2.5 Test Generators Co-Located with Types

Mock implementations and test fixture generators (`generate_test_*` functions) live in `#[cfg(test)]` blocks within the same module that defines the type. Public test utilities shared across crates are gated behind `#[cfg(feature = "test_utils")]`.

---

## 3. Trait Definitions

All trait definitions live in `crates/luminos-platform/src/traits.rs` (or `traits/` as a module directory if individual trait files are cleaner). The types below are the canonical, implementable definitions. An AI agent should be able to implement a complete platform backend from these signatures alone.

**Lifecycle convention:** All platform backends are responsible for resource cleanup via their `Drop` implementation. When a trait object (`Box<dyn ScreenCapture>`, etc.) is dropped, the implementation must release platform resources (X11 connections, D-Bus subscriptions, PipeWire streams, event loop handles). No explicit `shutdown()` method is defined on the traits -- `Drop` is the cleanup mechanism. Implementations that spawn background tasks must ensure those tasks are signaled to stop and joined (or detached) in `Drop`.

### 3.1 Common Types

```rust
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
/// gamma-encoded). The GPU rendering pipeline (see [03 -- Rendering
/// Pipeline]) performs gamma-correct resampling by converting to linear
/// space before interpolation and back to sRGB for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// Blue, Green, Red, Alpha (8 bits each). Native format on X11 and Windows.
    Bgra8,
    /// Red, Green, Blue, Alpha (8 bits each). Native format on macOS (ScreenCaptureKit).
    Rgba8,
}
```

### 3.2 ScreenCapture

Captures screen content as CPU pixel buffers. The magnification pipeline uploads these buffers to GPU textures via `wgpu::Queue::write_texture()`.

```rust
use std::sync::Arc;

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

/// A captured frame of screen content.
#[derive(Debug, Clone)]
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
    ) -> Result<tokio::sync::mpsc::Receiver<DisplayChangeEvent>, CaptureError>;
}

#[cfg(test)]
pub mod screen_capture_test_utils {
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

### 3.3 FocusTracker

Monitors keyboard focus changes via platform accessibility APIs. Not all applications expose focus information -- mouse-follow mode is the reliable universal fallback.

```rust
use tokio::sync::mpsc;

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
/// | Platform | Struct | Module | Mechanism |
/// |----------|--------|--------|-----------|
/// | Linux X11 | `AtSpiTracker` | `common::atspi_common` | AT-SPI2 via D-Bus (`atspi` crate) |
/// | Linux Wayland | `AtSpiTracker` | `common::atspi_common` | Same (AT-SPI2 is display-protocol-independent) |
/// | macOS | `AxTracker` | `macos::focus` | AXUIElement + AXObserver (`objc2` crate) |
/// | OpenBSD | `MouseFallbackTracker` | `openbsd` | No AT-SPI2 in base; mouse position only |
/// | Windows | `UiaTracker` | `windows::focus` | UI Automation (`windows` crate) |
///
/// # Coverage Limitations
///
/// Not all applications expose complete accessibility trees. Electron apps,
/// games, custom-rendered UIs, and legacy Win32 applications may not report
/// focus changes. The core engine must always fall back gracefully to
/// mouse-follow mode when no focus events arrive.
pub trait FocusTracker: Send + Sync {
    /// Begins monitoring focus changes and returns a receiver for events.
    ///
    /// The implementation spawns an internal task that listens for
    /// accessibility events and sends `FocusChangedEvent` values to the
    /// returned channel. The channel is bounded (`buffer_size` capacity).
    ///
    /// **Lossy semantics:** For focus tracking, only the latest position
    /// matters. Implementations must use `try_send()` and silently drop
    /// events when the channel is full, rather than awaiting send capacity.
    /// This prevents the accessibility event loop from blocking when the
    /// consumer falls behind. A `log::trace!` on drop is acceptable.
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
    /// cannot determine the focused element. This is a best-effort query,
    /// not a guaranteed result.
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

### 3.4 TtsEngine

Manages text-to-speech synthesis. The primary engine (Kokoro via sherpa-onnx) is cross-platform, so unlike other traits, the implementation is largely shared. Platform-specific native TTS engines serve as fallbacks for unsupported languages.

```rust
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
/// playback begins (not when it completes), enabling the caller to
/// proceed while speech plays.
///
/// # Object Safety
///
/// This trait is **object-safe** (`dyn TtsEngine` is supported). This is
/// required because the core engine may select between `SherpaEngine` and
/// `NativeTtsEngine` at runtime based on language availability and user
/// preference. The `speak` method returns a boxed future rather than
/// using RPITIT (`-> impl Future`) to preserve object safety.
///
/// # Implementations
///
/// | Struct | Engine | Notes |
/// |--------|--------|-------|
/// | `SherpaEngine` | Kokoro-82M via sherpa-onnx | Primary. Cross-platform. Near-commercial quality. |
/// | `SherpaEngine` | Piper VITS via sherpa-onnx | Language fallback. Same runtime, different models. |
/// | `NativeTtsEngine` | AVSpeech / SAPI / speech-dispatcher | System fallback per platform. |
///
/// # Architecture
///
/// ```text
/// text -> espeak-ng subprocess (phonemes) -> sherpa-onnx inference -> cpal audio
/// ```
///
/// espeak-ng is run as a long-lived subprocess for crash isolation.
/// The subprocess is spawned on first `speak` call and kept warm.
pub trait TtsEngine: Send + Sync {
    /// Speaks the given text asynchronously.
    ///
    /// Returns when audio playback begins (first audio samples are
    /// queued to the audio device). Target: <200ms from call to first audio.
    ///
    /// If speech is already in progress, the behavior depends on the
    /// `interrupt` flag:
    /// - `true`: Stop current speech immediately, begin new speech.
    /// - `false`: Queue this text after the current speech completes.
    ///
    /// Returns a boxed future (not RPITIT) to maintain object safety,
    /// allowing runtime dispatch between `SherpaEngine` and `NativeTtsEngine`.
    fn speak(
        &self,
        text: &str,
        interrupt: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), TtsError>> + Send + '_>>;

    /// Stops any speech currently in progress or queued.
    fn stop(&self) -> Result<(), TtsError>;

    /// Sets the active voice by its ID.
    ///
    /// The voice ID must match a `Voice::id` from `get_voices()`.
    fn set_voice(&self, voice_id: &str) -> Result<(), TtsError>;

    /// Sets the speech rate.
    ///
    /// `rate` is a multiplier: 1.0 = normal speed, 0.5 = half speed,
    /// 2.0 = double speed. Clamped to `[0.25, 4.0]` by the implementation.
    fn set_rate(&self, rate: f32) -> Result<(), TtsError>;

    /// Sets the speech pitch.
    ///
    /// `pitch` is a multiplier: 1.0 = normal pitch, 0.5 = lower,
    /// 2.0 = higher. Clamped to `[0.5, 2.0]` by the implementation.
    fn set_pitch(&self, pitch: f32) -> Result<(), TtsError>;

    /// Returns all available voices across all engines.
    fn get_voices(&self) -> Result<Vec<Voice>, TtsError>;

    /// Returns `true` if speech is currently being played.
    fn is_speaking(&self) -> bool;
}
```

### 3.5 WindowManager

Manages the magnification overlay window. This is the most platform-divergent trait due to docked mode's reliance on platform-specific window manager protocols.

```rust
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
    /// Other windows are prevented from overlapping (on platforms that
    /// support screen reservation: Linux X11, Linux Wayland with
    /// wlr-layer-shell compositors, OpenBSD, Windows).
    /// macOS and GNOME Wayland fall back to floating always-on-top.
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

    /// A window property could not be set (e.g., always-on-top denied).
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
/// | Linux X11 | `X11WindowManager` | EWMH `_NET_WM_STRUT_PARTIAL` via x11rb |
/// | Linux Wayland | `WaylandWindowManager` | Layer-shell protocol (wlr-layer-shell) |
/// | macOS | `CocoaWindowManager` | Floating `NSPanel` (overlay, not reservation) |
/// | OpenBSD | `X11WindowManager` | Shared with Linux X11 (EWMH) |
/// | Windows | `Win32WindowManager` | `SHAppBarMessage` / AppBar API |
///
/// # Docked Mode Platform Differences
///
/// Linux X11 and OpenBSD: EWMH struts reserve screen space. Other windows
/// respect the reservation and will not maximize behind the overlay.
///
/// macOS: No public API for third-party screen reservation. The overlay
/// floats on top (NSPanel with floating level) but maximized windows may
/// extend behind it. Documented as a known macOS limitation.
///
/// Windows: AppBar API reserves desktop space identically to the taskbar.
pub trait WindowManager: Send + Sync {
    /// Creates the magnification overlay window on the specified display.
    ///
    /// Returns a handle that can be used with wgpu to create a rendering
    /// surface. The window starts hidden; call `set_visible(true)` to show it.
    ///
    /// This method must be called from the main thread on macOS.
    fn create_overlay(&mut self, display_id: &str) -> Result<(), WindowError>;

    /// Sets the overlay's position and size in screen coordinates.
    fn set_overlay_bounds(&self, bounds: ScreenRect) -> Result<(), WindowError>;

    /// Switches the overlay to the specified display mode.
    ///
    /// In `Docked` mode, this also sets up platform-specific screen
    /// reservation (EWMH struts, AppBar, etc.).
    ///
    /// In `Lens` mode, this enables click-through on non-magnified areas
    /// and sets up cursor tracking.
    fn set_overlay_mode(
        &mut self,
        mode: OverlayMode,
    ) -> Result<(), WindowError>;

    /// Sets whether the overlay is always above other windows.
    ///
    /// This should default to `true` for all magnification modes.
    fn set_always_on_top(&self, always_on_top: bool) -> Result<(), WindowError>;

    /// Shows or hides the overlay window.
    fn set_visible(&self, visible: bool) -> Result<(), WindowError>;

    /// Returns the raw window handle for wgpu surface creation.
    ///
    /// The returned handle implements `raw_window_handle::HasWindowHandle`
    /// and `raw_window_handle::HasDisplayHandle`. The caller uses these
    /// to create a `wgpu::Surface`.
    ///
    /// Returns `None` if the overlay has not been created yet.
    fn raw_window_handle(
        &self,
    ) -> Option<&dyn raw_window_handle::HasWindowHandle>;

    /// Returns the raw display handle for wgpu surface creation.
    fn raw_display_handle(
        &self,
    ) -> Option<&dyn raw_window_handle::HasDisplayHandle>;
}
```

### 3.6 InputMonitor

Monitors global mouse and keyboard input across all applications. "Global" means the input is captured even when the Luminos window does not have focus -- this is essential for a magnification tool that must track the cursor at all times.

```rust
use tokio::sync::mpsc;

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
/// This is a simplified subset covering keys used for Luminos shortcuts.
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
/// The `rdev` crate provides cross-platform global input monitoring and
/// is the recommended starting point. Platform-native APIs may be used
/// for additional precision or lower latency:
///
/// | Platform | Primary | Fallback / Native |
/// |----------|---------|-------------------|
/// | Linux X11 | rdev | XInput2 / XRecord extension |
/// | Linux Wayland | rdev | libinput (requires permissions) |
/// | macOS | rdev | CGEvent tap (requires Accessibility permission) |
/// | OpenBSD | rdev | XInput2 / XRecord (same as Linux X11) |
/// | Windows | rdev | Raw Input / Low-level hooks |
pub trait InputMonitor: Send + Sync {
    /// Begins monitoring input events and returns a receiver.
    ///
    /// The implementation spawns an internal event loop that captures
    /// global input events and sends them to the returned channel.
    /// The channel is bounded (`buffer_size` capacity).
    ///
    /// **Lossy semantics for mouse moves:** Implementations must use
    /// `try_send()` for `MouseMoved` events and silently drop when the
    /// channel is full (only the latest position matters). Key events
    /// and button events should use `blocking_send()` (since the event
    /// callback runs outside the tokio runtime) to avoid dropped
    /// hotkeys. Mouse move coalescing (replacing the pending move with
    /// the latest position) is encouraged for efficiency.
    fn subscribe_input_events(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<InputEvent>, InputError>;

    /// Returns the current mouse pointer position.
    ///
    /// This is a synchronous query for the instantaneous pointer location.
    /// Useful for initialization or when the event stream has not yet
    /// delivered a position.
    fn get_mouse_position(&self) -> Result<ScreenPoint, InputError>;
}
```

### 3.7 AudioOutput

Manages audio playback for TTS output. This trait wraps `cpal`, which already provides cross-platform audio. The trait exists to decouple the TTS engine from the specific audio backend and to enable mock audio output in tests.

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
/// and playing audio samples generated by the TTS engine. The trait
/// abstraction enables mock audio output in tests and potential future
/// backends (e.g., PulseAudio directly, JACK).
///
/// # Platform Backends
///
/// | Platform | cpal Backend |
/// |----------|-------------|
/// | Linux X11/Wayland | ALSA or PulseAudio (cpal auto-selects) |
/// | macOS | CoreAudio |
/// | OpenBSD | sndio |
/// | Windows | WASAPI |
///
/// `cpal` handles backend selection automatically based on the platform.
/// The `AudioOutput` trait adds volume control, stop/interrupt semantics,
/// and a clean interface for the TTS pipeline.
pub trait AudioOutput: Send + Sync {
    /// Plays the given audio sample.
    ///
    /// Returns when playback begins (audio is queued to the device).
    /// If audio is already playing and `interrupt` is `true`, stops
    /// current playback and begins the new sample immediately.
    fn play_audio(
        &self,
        sample: AudioSample,
        interrupt: bool,
    ) -> Result<(), AudioError>;

    /// Stops any audio currently playing.
    fn stop_audio(&self) -> Result<(), AudioError>;

    /// Sets the output volume.
    ///
    /// `volume` is a linear scale from 0.0 (silent) to 1.0 (full).
    /// Values outside this range are clamped.
    fn set_volume(&self, volume: f32) -> Result<(), AudioError>;

    /// Returns the name of the default audio output device, if available.
    fn get_default_device_name(&self) -> Result<Option<String>, AudioError>;
}
```

---

## 4. Error Type Architecture

### 4.1 Top-Level Error Enum

All subsystem errors funnel into a single `LuminosError` type at the application boundary. This enables uniform error handling in the core engine and Tauri IPC layer.

```rust
/// Top-level application error.
///
/// Each variant wraps a subsystem-specific error type. The `From` trait
/// is implemented for each subsystem error, enabling `?` propagation
/// across subsystem boundaries.
#[derive(Debug, thiserror::Error)]
pub enum LuminosError {
    #[error("screen capture: {0}")]
    Capture(#[from] CaptureError),

    #[error("focus tracking: {0}")]
    Focus(#[from] FocusError),

    #[error("text-to-speech: {0}")]
    Tts(#[from] TtsError),

    #[error("window management: {0}")]
    Window(#[from] WindowError),

    #[error("input monitoring: {0}")]
    Input(#[from] InputError),

    #[error("audio output: {0}")]
    Audio(#[from] AudioError),

    #[error("configuration: {message}")]
    Config { message: String },

    #[error("internal error: {message}")]
    Internal { message: String },
}
```

### 4.2 Error Propagation Example

The following example shows how `?` propagation works across subsystem boundaries without explicit `match` statements:

```rust
/// Initializes the magnification pipeline on the primary display.
fn init_magnification(
    capture: &dyn ScreenCapture,
    window_mgr: &mut dyn WindowManager,
) -> Result<(), LuminosError> {
    // CaptureError -> LuminosError via From trait
    let displays = capture.list_displays()?;

    let primary = displays
        .iter()
        .find(|d| d.is_primary)
        .ok_or_else(|| CaptureError::DisplayNotFound("primary".to_string()))?;

    // WindowError -> LuminosError via From trait
    window_mgr.create_overlay(&primary.id)?;
    window_mgr.set_overlay_mode(OverlayMode::FullScreen)?;
    window_mgr.set_always_on_top(true)?;
    window_mgr.set_visible(true)?;

    log::info!("Magnification initialized on display '{}'", primary.name);
    Ok(())
}
```

### 4.3 Per-Subsystem Error Design Rules

Each subsystem error enum follows a consistent structure:

1. **Common variants** cover errors that can occur on any platform (e.g., `PermissionDenied`, `VoiceNotFound`).
2. **A `Platform` variant** with a `message: String` field captures platform-specific errors that do not map to common variants. This prevents the need to add variants for every possible OS-level error code.
3. **All variants include enough context** to produce an actionable log message without requiring the caller to add context.
4. **`thiserror::Error`** is used for all error types to auto-derive `Display` and `Error` implementations.

---

## 5. Conditional Compilation Strategy

### 5.1 Module Organization

The platform abstraction layer lives in its own crate (`luminos-platform`) within the Cargo workspace defined by [01 -- System Architecture](./01-system-architecture.md) Section 7.1. It has **no dependencies on other Luminos crates** -- it is the foundation of the dependency graph. Core engine logic (`magnification.rs`, `tts_pipeline.rs`) lives in the separate `luminos-core` crate.

```
crates/luminos-platform/
  src/
    lib.rs                  # Re-exports: traits + active backend factory
    traits.rs               # All six trait definitions (platform-independent)
    error.rs                # LuminosError top-level enum + per-subsystem errors
    common/
      mod.rs                # Shared utilities (e.g., X11 helpers used by Linux + OpenBSD)
      x11_common.rs         # XCB/EWMH helpers shared between Linux X11 and OpenBSD
      atspi_common.rs       # AT-SPI2 FocusTracker shared between Linux X11 and Wayland
    linux_x11/
      mod.rs                # XcbCapture, X11WindowManager, etc.
      capture.rs
      window.rs
      input.rs
    linux_wayland/
      mod.rs                # PipeWireCapture, WaylandWindowManager
      capture.rs
      window.rs
    macos/
      mod.rs                # SCKitCapture, AxTracker, CocoaWindowManager, etc.
      capture.rs
      focus.rs
      window.rs
      input.rs
    openbsd/
      mod.rs                # Re-exports from common/x11_common + OpenBSD-specific overrides
      capture.rs            # Thin wrapper delegating to common::x11_common
      window.rs             # Thin wrapper delegating to common::x11_common
    windows/
      mod.rs                # DxgiCapture, UiaTracker, Win32WindowManager, etc.
      capture.rs
      focus.rs
      window.rs
      input.rs
    mock/
      mod.rs                # MockScreenCapture, MockFocusTracker, etc.
      capture.rs            # #[cfg(any(test, feature = "test_utils"))]
      focus.rs
      tts.rs
      window.rs
      input.rs
      audio.rs
```

**Key structural notes:**
- `common/atspi_common.rs` houses the `AtSpiTracker` struct used by **both** Linux X11 and Linux Wayland, because AT-SPI2 operates over D-Bus (display-protocol-independent). Neither `linux_x11/` nor `linux_wayland/` owns this code.
- `linux_x11/` does **not** contain a `focus.rs` -- it re-exports `AtSpiTracker` from `common/atspi_common`.
- `linux_wayland/` also does **not** contain its own focus tracker -- it re-exports the same `AtSpiTracker`.
- Engine code (`magnification.rs`, `tts_pipeline.rs`, etc.) lives in the `luminos-core` crate, not here. See [01 -- System Architecture](./01-system-architecture.md) Section 7.1.

### 5.2 cfg Patterns

**Compile-time platform selection** for the backend modules:

```rust
// crates/luminos-platform/src/lib.rs

pub mod traits;
pub mod error;

#[cfg(any(test, feature = "test_utils"))]
pub mod mock;

// Shared code used by multiple platforms (must be declared before consumers).
#[cfg(any(target_os = "linux", target_os = "openbsd"))]
pub(crate) mod common;

// Platform backends -- only the relevant platform is compiled.
// On Linux, BOTH x11 and wayland modules are compiled; runtime selection
// chooses the active backend (see Section 5.3).

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
```

**Feature flags** for optional capabilities:

```toml
# crates/luminos-platform/Cargo.toml
[features]
default = ["wayland"]  # Wayland support is compiled by default on Linux
test_utils = []        # Enables mock backends for downstream crate testing
wayland = ["dep:ashpd"]          # Wayland capture support (Linux only, requires XDG Portal + PipeWire)
xshm = ["dep:x11rb"]             # XShm optimized capture (Phase 1)
```

**Note on the `wayland` feature flag:** On Linux, both X11 and Wayland backends are compiled into the same binary by default (the `wayland` feature is in `default`). The feature exists so that distribution packagers who target X11-only environments (e.g., legacy systems without PipeWire) can build without Wayland dependencies. Runtime backend selection still uses `XDG_SESSION_TYPE` (see Section 5.3); the feature flag only controls whether the Wayland code path is **available** at runtime.

### 5.3 Compile-Time vs Runtime Platform Selection

Most platform choices are compile-time: there is exactly one `ScreenCapture` implementation compiled for macOS, one for Windows, one for OpenBSD. Linux is the exception.

**The `PlatformBackends` bundle:**

Each platform factory returns a `PlatformBackends` struct that bundles all trait objects for the active platform:

```rust
/// Bundle of all platform-specific trait implementations.
///
/// Created once at application startup by the platform factory function.
/// The core engine receives this and programs against the trait interfaces.
pub struct PlatformBackends {
    pub capture: Box<dyn ScreenCapture>,
    pub focus_tracker: Box<dyn FocusTracker>,
    pub window_mgr: Box<dyn WindowManager>,
    pub input_monitor: Box<dyn InputMonitor>,
    pub audio_output: Box<dyn AudioOutput>,
    // TtsEngine is constructed separately by luminos-tts crate,
    // because it depends on AudioOutput + espeak-ng subprocess,
    // not on platform APIs directly.
}
```

**Linux: runtime X11/Wayland selection.**

On Linux, both X11 and Wayland backends are compiled into the same binary. The active backend is selected at runtime based on the user's session:

```rust
#[cfg(target_os = "linux")]
pub fn create_platform_backends() -> Result<PlatformBackends, LuminosError> {
    let session_type = detect_session_type();

    let (capture, window_mgr): (
        Box<dyn ScreenCapture>,
        Box<dyn WindowManager>,
    ) = match session_type {
        SessionType::Wayland => {
            log::info!("Detected Wayland session, using PipeWire capture");
            let capture = linux_wayland::PipeWireCapture::new()?;
            let window_mgr = linux_wayland::WaylandWindowManager::new()?;
            (Box::new(capture), Box::new(window_mgr))
        }
        SessionType::X11 => {
            log::info!("Detected X11 session, using XCB capture");
            let capture = linux_x11::XcbCapture::new()?;
            let window_mgr = linux_x11::X11WindowManager::new()?;
            (Box::new(capture), Box::new(window_mgr))
        }
    };

    // AT-SPI2 focus tracking works on both X11 and Wayland (D-Bus, not
    // display protocol). Same implementation used for both.
    // AtSpiTracker lives in common::atspi_common, not in linux_x11.
    let focus_tracker = common::atspi_common::AtSpiTracker::new()?;

    // rdev and cpal are cross-platform; same structs on both sessions.
    let input_monitor = linux_x11::RdevInputMonitor::new()?;
    let audio_output = CpalAudioOutput::new()?;

    Ok(PlatformBackends {
        capture,
        focus_tracker: Box::new(focus_tracker),
        window_mgr,
        input_monitor: Box::new(input_monitor),
        audio_output: Box::new(audio_output),
    })
}
```

**OpenBSD shares X11 code with Linux:**

```rust
// crates/luminos-platform/src/openbsd/capture.rs
use crate::common::x11_common;

pub struct XcbCapture {
    inner: x11_common::XcbCaptureImpl,
}

impl ScreenCapture for XcbCapture {
    fn list_displays(&self) -> Result<Vec<DisplayInfo>, CaptureError> {
        self.inner.list_displays()
    }

    fn capture_frame(
        &self,
        display_id: &str,
        region: Option<ScreenRect>,
    ) -> Result<CaptureFrame, CaptureError> {
        self.inner.capture_frame(display_id, region)
    }

    fn subscribe_display_changes(
        &self,
        buffer_size: usize,
    ) -> Result<tokio::sync::mpsc::Receiver<DisplayChangeEvent>, CaptureError> {
        // OpenBSD does not support display change notifications via X11.
        // The core engine falls back to periodic list_displays() polling.
        Err(CaptureError::BackendUnavailable {
            reason: "display change events not supported on OpenBSD".to_string(),
        })
    }
}
```

---

## 6. Platform Implementation Matrix

This matrix maps each trait to its concrete implementation per platform, including the underlying technology.

| Trait | Linux X11 | Linux Wayland | macOS | OpenBSD | Windows |
|-------|-----------|---------------|-------|---------|---------|
| **ScreenCapture** | `XcbCapture` -- xcap (XCB `xcb_get_image`); XShm via x11rb planned Phase 1 | `PipeWireCapture` -- PipeWire + XDG Desktop Portal (`ashpd` crate) | `SCKitCapture` -- xcap (ScreenCaptureKit) | `XcbCapture` -- shared with Linux X11 (xcap, XCB) | `DxgiCapture` -- windows-capture (DXGI Desktop Duplication) |
| **FocusTracker** | `AtSpiTracker` -- AT-SPI2 via D-Bus (`atspi` crate) | `AtSpiTracker` -- same (D-Bus is display-protocol-independent) | `AxTracker` -- AXUIElement + AXObserver (`objc2` crate) | `MouseFallbackTracker` -- mouse position only (no AT-SPI2 in base) | `UiaTracker` -- UI Automation (`windows` crate) |
| **TtsEngine** | `SherpaEngine` -- Kokoro via sherpa-onnx (`sherpa-rs`); espeak-ng subprocess | `SherpaEngine` -- same | `SherpaEngine` -- same | `SherpaEngine` -- same | `SherpaEngine` -- same |
| **WindowManager** | `X11WindowManager` -- winit + x11rb (EWMH struts) | `WaylandWindowManager` -- winit + wlr-layer-shell | `CocoaWindowManager` -- winit + NSPanel (floating overlay) | `X11WindowManager` -- shared with Linux X11 (winit + x11rb) | `Win32WindowManager` -- winit + AppBar API (`windows` crate) |
| **InputMonitor** | `RdevInputMonitor` -- `rdev` crate (X11 backend) | `RdevInputMonitor` -- `rdev` crate (evdev via `grab`; `listen` is X11-only) | `RdevInputMonitor` -- `rdev` crate (CGEvent tap) | `RdevInputMonitor` -- `rdev` crate (X11 backend) | `RdevInputMonitor` -- `rdev` crate (low-level hooks) |
| **AudioOutput** | `CpalAudioOutput` -- `cpal` (ALSA or PulseAudio) | `CpalAudioOutput` -- `cpal` (PipeWire or PulseAudio) | `CpalAudioOutput` -- `cpal` (CoreAudio) | `CpalAudioOutput` -- `cpal` (sndio; pending upstream PR #493) | `CpalAudioOutput` -- `cpal` (WASAPI) |

**Key observations:**

- **TtsEngine** and **AudioOutput** are effectively platform-agnostic. The same `SherpaEngine` and `CpalAudioOutput` structs are used everywhere. The trait abstraction still exists for testability (mock injection) and to support the platform-native TTS fallback path.
- **InputMonitor** uses `rdev` on all platforms. Platform-native alternatives may be substituted later for lower latency.
- **FocusTracker** on Linux Wayland uses the same `AtSpiTracker` as X11, because AT-SPI2 operates over D-Bus, not the display protocol. `AtSpiTracker` lives in `common::atspi_common`, shared by both Linux backends.
- **OpenBSD** shares four of six implementations with Linux X11 (via `common/`). Only `FocusTracker` (no AT-SPI2; falls back to mouse tracking) and potentially `AudioOutput` (sndio vs ALSA, handled transparently by cpal) differ.
- **ScreenCapture** now includes a `subscribe_display_changes()` method for hot-plug notification. Platforms that do not support display change events (e.g., OpenBSD) may return `Err`; the core engine degrades to periodic `list_displays()` polling.

---

## 7. Testing Strategy for Platform Code

### 7.1 Mock Implementations

Every trait has a corresponding mock implementation in `crates/luminos-platform/src/mock/`. Mocks are gated behind `#[cfg(any(test, feature = "test_utils"))]` and are parameterizable.

```rust
// crates/luminos-platform/src/mock/capture.rs
#[cfg(any(test, feature = "test_utils"))]
pub struct MockScreenCapture {
    displays: Vec<DisplayInfo>,
    frame: CaptureFrame,
    /// Error factory: called to produce an error when set. Using a factory
    /// function instead of storing a `CaptureError` directly because
    /// `CaptureError` is not `Clone` (its `Platform` variant contains
    /// `Option<Box<dyn Error>>`).
    error_factory: Option<Box<dyn Fn() -> CaptureError + Send + Sync>>,
}

#[cfg(any(test, feature = "test_utils"))]
impl MockScreenCapture {
    /// Creates a mock that returns a fixed frame for any capture request.
    pub fn generate_test_mock_screen_capture(
        displays: Vec<DisplayInfo>,
        frame: CaptureFrame,
    ) -> Self {
        Self { displays, frame, error_factory: None }
    }

    /// Configures the mock to return an error on every capture call.
    ///
    /// The factory is called each time to produce a fresh error value.
    /// This preserves the exact error variant (not always `Platform`).
    ///
    /// # Example
    /// ```ignore
    /// let mock = MockScreenCapture::generate_test_mock_screen_capture(displays, frame)
    ///     .with_error(|| CaptureError::PermissionDenied);
    /// ```
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> CaptureError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }
}

#[cfg(any(test, feature = "test_utils"))]
impl ScreenCapture for MockScreenCapture {
    fn list_displays(&self) -> Result<Vec<DisplayInfo>, CaptureError> {
        Ok(self.displays.clone())
    }

    fn capture_frame(
        &self,
        display_id: &str,
        _region: Option<ScreenRect>,
    ) -> Result<CaptureFrame, CaptureError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        if !self.displays.iter().any(|d| d.id == display_id) {
            return Err(CaptureError::DisplayNotFound(display_id.to_string()));
        }
        Ok(self.frame.clone())
    }

    fn subscribe_display_changes(
        &self,
        buffer_size: usize,
    ) -> Result<tokio::sync::mpsc::Receiver<DisplayChangeEvent>, CaptureError> {
        let (_tx, rx) = tokio::sync::mpsc::channel(buffer_size);
        Ok(rx)
    }
}
```

Similar mock structs exist for `MockFocusTracker`, `MockTtsEngine`, `MockWindowManager`, `MockInputMonitor`, and `MockAudioOutput`. Each follows the pattern:

- Constructor named `generate_test_mock_<trait_name>`
- Builder methods for configuring error injection via **closures** (error factories), not stored error values -- because most error enums are not `Clone` (they may contain `Box<dyn Error>` source chains)
- Sensible defaults that succeed for the happy path

### 7.2 Test Naming Conventions

Tests follow hierarchical naming for granular selection via `cargo nextest run`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_capture_list_displays_returns_primary() {
        // ...
    }

    #[test]
    fn screen_capture_list_displays_empty_when_no_displays() {
        // ...
    }

    #[test]
    fn screen_capture_capture_frame_region_out_of_bounds() {
        // ...
    }

    #[test]
    fn focus_tracker_subscribe_receives_events() {
        // ...
    }

    #[test]
    fn focus_tracker_get_focused_element_returns_none_when_unfocused() {
        // ...
    }
}
```

This enables running all screen capture tests with:

```bash
cargo nextest run screen_capture_
```

### 7.3 Platform-Specific Testing

**CI matrix:** GitHub Actions runs platform-specific tests on each target:

| Platform | CI Runner | Test Scope | Notes |
|----------|-----------|------------|-------|
| Linux X11 | `ubuntu-latest` + Xvfb | Full unit + integration | Xvfb provides a headless X11 server |
| Linux Wayland | `ubuntu-latest` + wlheadless | Wayland-specific tests | PipeWire mock or headless Wayland compositor |
| macOS | `macos-latest` | Full unit + integration | ScreenCaptureKit tests require Screen Recording permission (CI grant) |
| OpenBSD | Self-hosted runner | X11 tests only | OpenBSD GitHub-hosted runners do not exist |
| Windows | `windows-latest` | Full unit + integration | DXGI tests require a display adapter (virtual display) |

**Headless testing:** Platform-specific tests that require a display server, GPU, or audio device are gated behind a `ci_platform_tests` feature flag. Unit tests using mock backends run everywhere. Integration tests that exercise real platform APIs only run on their target platform's CI runner.

```rust
// crates/luminos-platform/src/linux_x11/capture.rs
#[cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]
mod platform_integration_tests {
    #[test]
    fn screen_capture_xcb_captures_xvfb_display() {
        // Only runs on Linux CI with Xvfb
    }
}
```

### 7.4 Integration Testing with Mock Backends

The full magnification pipeline can be tested end-to-end using mock backends:

```rust
#[cfg(test)]
mod pipeline_integration_tests {
    use super::*;
    use crate::platform::mock::*;

    #[test]
    fn magnification_pipeline_init_to_first_frame() {
        let displays = vec![
            generate_test_display_info("test-0", 1920, 1080, true),
        ];
        let frame = generate_test_capture_frame(96, 54, [0, 0, 0, 255]);
        let capture = MockScreenCapture::generate_test_mock_screen_capture(
            displays,
            frame,
        );

        // Exercise the pipeline with mock capture
        // Verify that the pipeline produces a valid output texture
    }
}
```

---

## 8. Platform-Specific Considerations

### 8.1 Linux X11

**Starting platform (Phase 0).** Linux X11 is the simplest capture path and the most underserved user base.

**Screen capture:**
- xcap uses XCB protocol (`xcb_get_image`) for screen capture. This performs a full X server round-trip per capture frame.
- No permission dialogs required -- XCB screen capture works immediately on X11 sessions.
- **XShm optimization (Phase 1):** xcap's XCB dependency does not enable the `shm` feature, meaning it uses the standard (non-shared-memory) capture path. At low zoom levels (1.5-3x) with large capture regions on high-resolution displays, this may exceed the 8ms capture budget. An `x11rb`-based capture backend with XShm (shared memory) support is planned for Phase 1. OBS Studio achieves 60fps+ X11 capture via XShm, validating this approach.

**Window management:**
- EWMH (Extended Window Manager Hints) provides `_NET_WM_STRUT_PARTIAL` for screen space reservation in docked mode.
- Both KWin (KDE) and Mutter (GNOME) respect EWMH struts.
- Implementation requires setting via raw X11 calls using `x11rb` on the window handle obtained from winit. The properties to set:
  1. `_NET_WM_WINDOW_TYPE` = `_NET_WM_WINDOW_TYPE_DOCK`
  2. `_NET_WM_STRUT_PARTIAL` with 12 cardinal values
  3. `_NET_WM_STATE_STICKY` for all-workspace visibility
  4. `_NET_WM_STATE_ABOVE` for always-on-top (also set via winit)

**Focus tracking:**
- AT-SPI2 over D-Bus via the `atspi` crate (from the Odilia screen reader project).
- `AtSpiTracker` lives in `common::atspi_common` (not `linux_x11`) because it is shared with the Wayland backend.
- Register for `focus:` events on the AT-SPI bus; query component screen extents on focus change.
- Coverage is application-dependent: GTK and Qt apps expose complete accessibility trees; games, Electron apps, and custom-rendered UIs may not.

**GPU rendering:**
- Vulkan backend via wgpu. Mesa drivers provide strong Vulkan support across Intel, AMD, and NVIDIA hardware.
- wgpu's `Backends::GL` fallback is available for very old hardware without Vulkan support.

### 8.2 Linux Wayland

**Phase 1.** The Wayland transition is actively breaking existing X11-only magnifiers (KMag, Magnus, xzoom), creating urgency for Wayland support.

**Screen capture:**
- Direct screen capture is not possible on Wayland by design (security model prevents arbitrary screen reading).
- **XDG Desktop Portal** provides screen capture via a user consent dialog. The user selects which display or window to share.
- **PipeWire** streams the captured frames. The `ashpd` crate (Rust bindings for XDG Desktop Portal) and `pipewire` crate handle the integration.
- **Session restore tokens:** After the user grants permission once, a restore token can be persisted to skip the consent dialog on subsequent launches. This is critical for usability -- a low-vision user who needs magnification to read the consent dialog faces a chicken-and-egg problem.

**Runtime detection of X11 vs Wayland:**

This function is the single source of truth for session detection. It is used by the `create_platform_backends()` factory in Section 5.3.

```rust
/// Determines the active display server session type.
///
/// Checks `XDG_SESSION_TYPE` first (the standard), then falls back to
/// probing `WAYLAND_DISPLAY` (set by compositors). Defaults to X11
/// when detection is ambiguous (e.g., TTY login, empty env).
pub(crate) fn detect_session_type() -> SessionType {
    match std::env::var("XDG_SESSION_TYPE").as_deref() {
        Ok("wayland") => SessionType::Wayland,
        Ok("x11") => SessionType::X11,
        _ => {
            // Fallback: check for WAYLAND_DISPLAY env var
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                log::warn!(
                    concat!(
                        "XDG_SESSION_TYPE not set but WAYLAND_DISPLAY exists; ",
                        "assuming Wayland session"
                    )
                );
                SessionType::Wayland
            } else {
                log::info!("Defaulting to X11 session (XDG_SESSION_TYPE not set)");
                SessionType::X11
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionType {
    X11,
    Wayland,
}
```

**Focus tracking:**
- AT-SPI2 works unchanged on Wayland -- it communicates via D-Bus, not the display protocol. The same `AtSpiTracker` struct from the X11 backend is reused directly.

**Window management:**
- Wayland does not support EWMH struts. For docked mode, the overlay uses the `wlr-layer-shell` protocol (supported by Sway, Hyprland, and other wlroots-based compositors) or the `ext-layer-shell` protocol (emerging standard).
- GNOME's Mutter does not support `wlr-layer-shell`. On GNOME Wayland, docked mode falls back to a floating always-on-top window (similar to the macOS behavior). This is a known limitation documented for users.

**Input monitoring:**
- `rdev`'s `listen()` function uses X11 APIs and does **not** work on Wayland. The `grab()` function (requires `unstable_grab` feature) uses **evdev** and works on both X11 and Wayland. This requires the user to be in the `input` group (or `plugdev` on some distros). Note: `grab()` intercepts events (preventing their delivery to other applications), which differs from passive monitoring. Evaluate whether this semantic difference is acceptable for magnification input tracking, or whether direct evdev or libinput integration is needed.

### 8.3 macOS

**Phase 2.** macOS has a capable built-in Zoom, but no open-source alternative with TTS integration.

**Screen capture:**
- ScreenCaptureKit is mandatory from macOS 15 (the legacy `CGWindowListCreateImage` is deprecated).
- **Requires Screen Recording permission.** The OS presents a system dialog on first use. The user must grant permission in System Settings > Privacy & Security > Screen Recording.
- xcap uses ScreenCaptureKit internally on macOS.

**Focus tracking:**
- AXUIElement API via `objc2` crate.
- Register `AXObserver` callbacks with `kAXFocusedUIElementChangedNotification`.
- Retrieve element bounds via `kAXPositionAttribute` and `kAXSizeAttribute`.
- **Requires Accessibility permission.** The OS presents a system dialog. The user must grant permission in System Settings > Privacy & Security > Accessibility.

**Window management:**
- winit creates the overlay window. macOS-specific configuration via `objc2`:
  - `NSPanel` with `NSWindowLevel.floating` for always-on-top behavior.
  - `NSWindowCollectionBehavior.canJoinAllSpaces` for multi-desktop visibility.
- **Docked mode limitation:** macOS does not provide a public API for third-party applications to reserve screen space the way the Dock does. The docked overlay floats on top but maximized windows may extend behind it. Accepted trade-off documented in the Tech Stack Evaluation.
- Metal GPU backend via wgpu (the only option on macOS).

**Audio:**
- CoreAudio via `cpal`. Mature and well-supported.

### 8.4 OpenBSD

**Phase 3.** Essentially zero accessibility infrastructure on OpenBSD -- high impact per user despite the small user base. Most code is shared with Linux X11.

**Screen capture:**
- OpenBSD's xenocara provides standard X11/XCB libraries. The same `XcbCapture` code (via `common::x11_common`) used on Linux X11 works.
- xcap has no explicit OpenBSD CI, but the underlying XCB protocol is platform-agnostic. Build validation is required.

**Focus tracking:**
- AT-SPI2 is not available in the OpenBSD base system. D-Bus is available as a package, and AT-SPI2 could theoretically be ported, but this is not a Phase 3 priority.
- `MouseFallbackTracker` is the default: tracks the mouse pointer position. No accessibility-API-driven focus tracking.

**Window management:**
- EWMH strut mechanism works identically to Linux X11 -- it is standard X11 protocol. Window managers that support EWMH (FVWM, ported KWin/GNOME components, or cwm with EWMH patches) will respect `_NET_WM_STRUT_PARTIAL`.
- Shared `X11WindowManager` from `common::x11_common`.

**GPU rendering:**
- Vulkan support via Mesa is limited but improving on OpenBSD.
- **GL fallback may be needed.** wgpu's `Backends::GL` provides an OpenGL ES fallback. If Vulkan drivers are unavailable, the rendering pipeline falls back to GL. Performance at 60fps may not be achievable on all hardware with the GL backend; testing is required.

**Audio:**
- sndio is the native OpenBSD audio system. `cpal` has a pending sndio support PR (#493, submitted 2020, not yet merged). Phase 3 planning must account for this gap: options include contributing to the upstream PR merge, maintaining a patched cpal fork, using `sndio-sys` directly, or using PulseAudio on OpenBSD (available as a package).

### 8.5 Windows

**Phase 4.** Windows is sequenced last because it already has multiple magnification options (ZoomText, SuperNova, Fusion, Windows Magnifier, VMG). Windows is still important -- over 90% of AT users are on Windows (WebAIM data) -- but those users are less underserved.

**Screen capture:**
- DXGI Desktop Duplication via the `windows-capture` crate provides high-performance capture with dirty-rectangle metadata and GPU-texture output.
- No yellow border (unlike Windows.Graphics.Capture on older Windows versions).
- DXGI can output D3D11 textures, which may be shareable with wgpu's DX12 backend via cross-API texture sharing (zero-copy optimization for later phases).

**Focus tracking:**
- UI Automation via the `windows` crate (Microsoft-maintained). Register `IUIAutomationFocusChangedEventHandler` for focus change notifications.
- Call `get_CurrentBoundingRectangle()` on the focused element for screen coordinates.
- UIA is the most reliable method; MSAA is deprecated and has known gaps.

**Window management:**
- AppBar API (`SHAppBarMessage` + `ABM_NEW`) via the `windows` crate reserves desktop space identically to the taskbar.
- Other windows respect the AppBar reservation when maximizing.
- DX12 GPU backend via wgpu.

**Coexistence with screen readers:**
- Luminos must not interfere with NVDA or JAWS. Many low-vision users run both a magnifier and a screen reader simultaneously.
- NVDA uses UI Automation and MSAA. Luminos's UIA focus tracking is read-only (subscribes to events, does not modify the accessibility tree) and should not conflict.
- The magnification overlay (winit + wgpu) is a standard window and should not interfere with screen reader window enumeration.
- Testing with NVDA and JAWS running simultaneously is a Phase 4 CI requirement.

---

## 9. Cross-References

| Topic | Document | Section |
|-------|----------|---------|
| Dual-window architecture (overlay vs. control panel) | [01 - System Architecture](./01-system-architecture.md) | Architecture overview |
| Capture-to-GPU rendering pipeline | [03 - Rendering Pipeline](./03-rendering-pipeline.md) | Full pipeline design |
| TTS pipeline (espeak-ng subprocess, Kokoro inference, cpal output) | [04 - TTS Pipeline](./04-tts-pipeline.md) | Pipeline architecture |
| Tauri IPC between control panel and Rust backend | [05 - Control Panel](./05-control-panel.md) | IPC design |
| Performance budgets and profiling strategy | [06 - Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Performance |
| CI/CD matrix and platform-specific test infrastructure | [07 - Testing Strategy](./07-testing-strategy.md) | CI pipeline |
| Cargo workspace structure and conditional compilation | [08 - Build and Distribution](./08-build-and-distribution.md) | Workspace layout |
| Phase 0-4 platform sequencing and milestones | [09 - Implementation Roadmap](./09-implementation-roadmap.md) (planned) | Phased delivery |
| Platform API deprecation and driver compatibility risks | [10 - Risk Register](./10-risk-register.md) | Platform risks |
| Technology stack validation and crate selection | [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) | Sections 3-5 |
| Feature roadmap and platform development order | [Product Strategy](../PRODUCT_STRATEGY.md) | Sections 7-8 |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-15 | Initial platform abstraction strategy |
| 1.1 | 2026-03-15 | Cross-analysis review: aligned module paths with Section 01 workspace; fixed TtsEngine::speak to use boxed future for object safety; added PlatformBackends struct definition; moved AtSpiTracker to common/atspi_common; fixed Wayland feature flag default; fixed mock error injection pattern; added display change events to ScreenCapture; added color space docs to PixelFormat; added lifecycle/Drop convention; clarified channel lossy semantics |
| 1.2 | 2026-03-15 | Audit review: added subscribe_display_changes to OpenBSD XcbCapture and MockScreenCapture impl blocks; corrected rdev Wayland backend from "libinput" to "evdev via grab" with usage notes; corrected cpal sndio claim to note pending PR #493; fixed mock path references from src/platform/mock/ to crates/luminos-platform/src/mock/; corrected InputMonitor send() to blocking_send(); added Eq to OverlayMode; added PartialEq to DisplayInfo; expanded Docked mode reservation comment to include Wayland layer-shell |
