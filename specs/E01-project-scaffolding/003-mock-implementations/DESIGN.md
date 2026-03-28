# Design: Story E01/003 -- Mock Implementations & Test Utilities

**Story:** [STORY.md](./STORY.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** Principal Architect Agent
**Risk Refs:** [RISK-003](../../tech-strategy/10-risk-register.md#risk-003-platform-trait-surface-area-inadequacy) (trait surface area -- mocks are the first implementation of the traits and will surface any signature issues), [RISK-017](../../tech-strategy/10-risk-register.md#risk-017-screen-content-and-tts-text-leakage-via-logs-and-gpu-memory) (mocks must not log CaptureFrame pixel data)

---

## Overview

This design implements mock versions of all six platform abstraction traits defined in Story 002. Each mock follows the same pattern from [doc-02 Section 7.1](../../tech-strategy/02-platform-abstraction.md#71-mock-implementations):

1. A struct holding configurable state (return data, error factory)
2. A `generate_test_mock_<trait>()` constructor for easy test setup
3. A `with_error()` builder method accepting a closure factory (`Fn() -> XxxError + Send + Sync + 'static`)
4. A trait implementation that checks the error factory first, then returns configured data

Error injection uses **closure factories** rather than stored error values because error types are not `Clone` -- several contain `Box<dyn Error>` in their `Platform` variants. The factory is called on each method invocation to produce a fresh error value.

All mock code is gated behind `#[cfg(any(test, feature = "test_utils"))]`. The `test_utils` Cargo feature (defined in Story 001) enables downstream crates to import mocks in their `[dev-dependencies]`.

## Architecture

### Component Diagram

```
crates/luminos-platform/src/mock/
  |
  +-- mod.rs          # Re-exports all mock structs and constructors
  +-- capture.rs      # MockScreenCapture
  +-- focus.rs        # MockFocusTracker
  +-- tts.rs          # MockTtsEngine
  +-- window.rs       # MockWindowManager
  +-- input.rs        # MockInputMonitor
  +-- audio.rs        # MockAudioOutput
```

All files in `mock/` are compiled only when `cfg(any(test, feature = "test_utils"))` is active. The `mock` module is declared in `lib.rs` (Story 002) under that same gate.

**Downstream usage:**

```
luminos-core/Cargo.toml:
  [dev-dependencies]
  luminos-platform = { workspace = true, features = ["test_utils"] }

luminos-core/tests/some_test.rs:
  use luminos_platform::mock::MockScreenCapture;
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `mock::capture` | New | MockScreenCapture implementing ScreenCapture |
| `mock::focus` | New | MockFocusTracker implementing FocusTracker |
| `mock::tts` | New | MockTtsEngine implementing TtsEngine |
| `mock::window` | New | MockWindowManager implementing WindowManager |
| `mock::input` | New | MockInputMonitor implementing InputMonitor |
| `mock::audio` | New | MockAudioOutput implementing AudioOutput |
| `mock::mod` | New | Re-exports all mocks and constructors |

### Data Flow

Mocks do not perform real platform operations. The data flow is:

1. **Test setup:** Test code calls `generate_test_mock_<trait>(...)` with configured return data
2. **Optional error injection:** Test code chains `.with_error(|| SomeError::Variant { ... })`
3. **Method call:** Test code calls a trait method on the mock
4. **Error check:** Mock checks `self.error_factory` -- if `Some`, calls the factory and returns `Err`
5. **Happy path:** Mock returns the pre-configured data (cloned or freshly created)

---

## API Design

All mock structs follow a consistent pattern. Full type signatures are provided for every mock, its constructor, builder, and trait implementation.

### Common Pattern

Every mock struct has this shape:

```rust
#[cfg(any(test, feature = "test_utils"))]
pub struct Mock<Trait> {
    // Fields holding return data (specific to each trait)
    ...
    /// Error factory: called to produce an error when set.
    error_factory: Option<Box<dyn Fn() -> <Error> + Send + Sync>>,
}

impl Mock<Trait> {
    pub fn generate_test_mock_<trait>(...) -> Self { ... }
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> <Error> + Send + Sync + 'static
    { ... }
}

impl <Trait> for Mock<Trait> {
    // Each method checks error_factory first, then returns stored data
}
```

### `mock/capture.rs` -- MockScreenCapture

```rust
use crate::traits::{
    CaptureFrame, CaptureError, DisplayChangeEvent, DisplayInfo,
    ScreenCapture, ScreenRect,
};
use tokio::sync::mpsc;

/// Mock implementation of `ScreenCapture` for unit testing.
///
/// Returns pre-configured display lists and capture frames.
/// Supports error injection via `with_error()` builder.
///
/// # Example
///
/// ```rust
/// use luminos_platform::mock::MockScreenCapture;
/// use luminos_platform::traits::types::test_utils::*;
///
/// let displays = vec![generate_test_display_info("test-0", 1920, 1080, true)];
/// let frame = generate_test_capture_frame(64, 48, [0, 0, 255, 255]);
/// let capture = MockScreenCapture::generate_test_mock_screen_capture(
///     displays.clone(), frame,
/// );
/// assert_eq!(capture.list_displays().unwrap(), displays);
/// ```
#[cfg(any(test, feature = "test_utils"))]
pub struct MockScreenCapture {
    /// Display list returned by `list_displays()`.
    displays: Vec<DisplayInfo>,
    /// Frame returned by `capture_frame()` on success.
    frame: CaptureFrame,
    /// Error factory: called to produce an error when set.
    error_factory: Option<Box<dyn Fn() -> CaptureError + Send + Sync>>,
}

#[cfg(any(test, feature = "test_utils"))]
impl MockScreenCapture {
    /// Creates a mock that returns fixed display info and capture frames.
    pub fn generate_test_mock_screen_capture(
        displays: Vec<DisplayInfo>,
        frame: CaptureFrame,
    ) -> Self {
        Self {
            displays,
            frame,
            error_factory: None,
        }
    }

    /// Configures the mock to return an error on every method call
    /// that returns `Result<_, CaptureError>`.
    ///
    /// The factory is called each time to produce a fresh error value
    /// (error types are not `Clone`).
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
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
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
        // Validate display_id even in mock -- matches real backend behavior
        if !self.displays.iter().any(|d| d.id == display_id) {
            return Err(CaptureError::DisplayNotFound(display_id.to_string()));
        }
        Ok(self.frame.clone())
    }

    fn subscribe_display_changes(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<DisplayChangeEvent>, CaptureError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        // Return an empty channel -- no real display changes in mock mode
        let (_tx, rx) = mpsc::channel(buffer_size);
        Ok(rx)
    }
}
```

### `mock/focus.rs` -- MockFocusTracker

```rust
use crate::traits::{
    FocusChangedEvent, FocusError, FocusTracker, ScreenRect,
};
use tokio::sync::mpsc;

/// Mock implementation of `FocusTracker` for unit testing.
///
/// Returns a pre-configured focused element. Supports error injection.
#[cfg(any(test, feature = "test_utils"))]
pub struct MockFocusTracker {
    /// The focused element returned by `get_focused_element()`.
    focused_element: Option<FocusChangedEvent>,
    /// Error factory for error injection.
    error_factory: Option<Box<dyn Fn() -> FocusError + Send + Sync>>,
}

#[cfg(any(test, feature = "test_utils"))]
impl MockFocusTracker {
    /// Creates a mock with an optional pre-configured focused element.
    pub fn generate_test_mock_focus_tracker(
        focused_element: Option<FocusChangedEvent>,
    ) -> Self {
        Self {
            focused_element,
            error_factory: None,
        }
    }

    /// Configures the mock to return an error on every method call.
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> FocusError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }
}

#[cfg(any(test, feature = "test_utils"))]
impl FocusTracker for MockFocusTracker {
    fn subscribe_focus_changes(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<FocusChangedEvent>, FocusError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        let (_tx, rx) = mpsc::channel(buffer_size);
        Ok(rx)
    }

    fn get_focused_element(&self) -> Result<Option<FocusChangedEvent>, FocusError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(self.focused_element.clone())
    }

    fn get_element_bounds(
        &self,
        _element_id: &str,
    ) -> Result<Option<ScreenRect>, FocusError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        // Return the bounds of the focused element if it exists
        Ok(self.focused_element.as_ref().map(|e| e.bounds))
    }
}
```

### `mock/tts.rs` -- MockTtsEngine

```rust
use std::future::Future;
use std::pin::Pin;
use crate::traits::{TtsEngine, TtsError, Voice, TtsBackend};

/// Mock implementation of `TtsEngine` for unit testing.
///
/// The `speak` method returns a boxed future that resolves immediately.
/// Supports error injection and basic state tracking (`is_speaking`).
#[cfg(any(test, feature = "test_utils"))]
pub struct MockTtsEngine {
    /// Voices returned by `get_voices()`.
    voices: Vec<Voice>,
    /// Error factory for error injection.
    error_factory: Option<Box<dyn Fn() -> TtsError + Send + Sync>>,
}

#[cfg(any(test, feature = "test_utils"))]
impl MockTtsEngine {
    /// Creates a mock with a pre-configured voice list.
    pub fn generate_test_mock_tts_engine(voices: Vec<Voice>) -> Self {
        Self {
            voices,
            error_factory: None,
        }
    }

    /// Configures the mock to return an error on every method call.
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> TtsError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }
}

#[cfg(any(test, feature = "test_utils"))]
impl TtsEngine for MockTtsEngine {
    fn speak(
        &self,
        _text: &str,
        _interrupt: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), TtsError>> + Send + '_>> {
        if let Some(ref factory) = self.error_factory {
            let err = factory();
            return Box::pin(async move { Err(err) });
        }
        Box::pin(async { Ok(()) })
    }

    fn stop(&self) -> Result<(), TtsError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_voice(&self, _voice_id: &str) -> Result<(), TtsError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_rate(&self, _rate: f32) -> Result<(), TtsError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_pitch(&self, _pitch: f32) -> Result<(), TtsError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn get_voices(&self) -> Result<Vec<Voice>, TtsError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(self.voices.clone())
    }

    fn is_speaking(&self) -> bool {
        // Mock never speaks -- always returns false
        false
    }
}
```

### `mock/window.rs` -- MockWindowManager

```rust
use crate::traits::{
    OverlayMode, ScreenRect, WindowError, WindowManager,
};

/// Mock implementation of `WindowManager` for unit testing.
///
/// All methods succeed by default. `raw_window_handle()` and
/// `raw_display_handle()` return `None` (no real window exists).
#[cfg(any(test, feature = "test_utils"))]
pub struct MockWindowManager {
    /// Whether `create_overlay` has been called.
    overlay_created: bool,
    /// Error factory for error injection.
    error_factory: Option<Box<dyn Fn() -> WindowError + Send + Sync>>,
}

#[cfg(any(test, feature = "test_utils"))]
impl MockWindowManager {
    /// Creates a mock window manager with default (success) behavior.
    pub fn generate_test_mock_window_manager() -> Self {
        Self {
            overlay_created: false,
            error_factory: None,
        }
    }

    /// Configures the mock to return an error on every method call
    /// that returns `Result<_, WindowError>`.
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> WindowError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }
}

#[cfg(any(test, feature = "test_utils"))]
impl WindowManager for MockWindowManager {
    fn create_overlay(&mut self, _display_id: &str) -> Result<(), WindowError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        self.overlay_created = true;
        Ok(())
    }

    fn set_overlay_bounds(&self, _bounds: ScreenRect) -> Result<(), WindowError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_overlay_mode(&mut self, _mode: OverlayMode) -> Result<(), WindowError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_always_on_top(&self, _always_on_top: bool) -> Result<(), WindowError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_visible(&self, _visible: bool) -> Result<(), WindowError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn raw_window_handle(&self) -> Option<&dyn raw_window_handle::HasWindowHandle> {
        // No real window exists in mock mode
        None
    }

    fn raw_display_handle(&self) -> Option<&dyn raw_window_handle::HasDisplayHandle> {
        // No real display exists in mock mode
        None
    }
}
```

### `mock/input.rs` -- MockInputMonitor

```rust
use crate::traits::{
    InputError, InputEvent, InputMonitor, ScreenPoint,
};
use tokio::sync::mpsc;

/// Mock implementation of `InputMonitor` for unit testing.
///
/// Returns a pre-configured mouse position. The event subscription
/// returns an empty channel (no real input events in mock mode).
#[cfg(any(test, feature = "test_utils"))]
pub struct MockInputMonitor {
    /// Mouse position returned by `get_mouse_position()`.
    mouse_position: ScreenPoint,
    /// Error factory for error injection.
    error_factory: Option<Box<dyn Fn() -> InputError + Send + Sync>>,
}

#[cfg(any(test, feature = "test_utils"))]
impl MockInputMonitor {
    /// Creates a mock with a pre-configured mouse position.
    pub fn generate_test_mock_input_monitor(mouse_position: ScreenPoint) -> Self {
        Self {
            mouse_position,
            error_factory: None,
        }
    }

    /// Configures the mock to return an error on every method call.
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> InputError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }
}

#[cfg(any(test, feature = "test_utils"))]
impl InputMonitor for MockInputMonitor {
    fn subscribe_input_events(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<InputEvent>, InputError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        let (_tx, rx) = mpsc::channel(buffer_size);
        Ok(rx)
    }

    fn get_mouse_position(&self) -> Result<ScreenPoint, InputError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(self.mouse_position)
    }
}
```

### `mock/audio.rs` -- MockAudioOutput

```rust
use crate::traits::{AudioError, AudioOutput, AudioSample};

/// Mock implementation of `AudioOutput` for unit testing.
///
/// All methods succeed by default. Does not actually play audio.
#[cfg(any(test, feature = "test_utils"))]
pub struct MockAudioOutput {
    /// Device name returned by `get_default_device_name()`.
    device_name: Option<String>,
    /// Error factory for error injection.
    error_factory: Option<Box<dyn Fn() -> AudioError + Send + Sync>>,
}

#[cfg(any(test, feature = "test_utils"))]
impl MockAudioOutput {
    /// Creates a mock with default (success) behavior.
    pub fn generate_test_mock_audio_output() -> Self {
        Self {
            device_name: Some("Mock Audio Device".to_string()),
            error_factory: None,
        }
    }

    /// Configures the mock to return an error on every method call.
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> AudioError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }
}

#[cfg(any(test, feature = "test_utils"))]
impl AudioOutput for MockAudioOutput {
    fn play_audio(
        &self,
        _sample: AudioSample,
        _interrupt: bool,
    ) -> Result<(), AudioError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn stop_audio(&self) -> Result<(), AudioError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_volume(&self, _volume: f32) -> Result<(), AudioError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn get_default_device_name(&self) -> Result<Option<String>, AudioError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(self.device_name.clone())
    }
}
```

### `mock/mod.rs` -- Module Re-exports

```rust
//! Mock implementations of all six platform traits for unit testing.
//!
//! Gated behind `#[cfg(any(test, feature = "test_utils"))]`.
//!
//! # Usage
//!
//! In the same crate (unit tests):
//! ```rust
//! use crate::mock::MockScreenCapture;
//! ```
//!
//! In downstream crates (via `test_utils` feature):
//! ```rust
//! // Cargo.toml: luminos-platform = { workspace = true, features = ["test_utils"] }
//! use luminos_platform::mock::MockScreenCapture;
//! ```

pub mod capture;
pub mod focus;
pub mod tts;
pub mod window;
pub mod input;
pub mod audio;

pub use capture::MockScreenCapture;
pub use focus::MockFocusTracker;
pub use tts::MockTtsEngine;
pub use window::MockWindowManager;
pub use input::MockInputMonitor;
pub use audio::MockAudioOutput;
```

### Example Test Code

**Happy-path test:**

```rust
#[cfg(test)]
mod tests {
    use crate::traits::types::test_utils::*;
    use crate::mock::MockScreenCapture;

    #[test]
    fn mock_screen_capture_list_displays_returns_configured_displays() {
        let displays = vec![
            generate_test_display_info("test-0", 1920, 1080, true),
            generate_test_display_info("test-1", 2560, 1440, false),
        ];
        let frame = generate_test_capture_frame(64, 48, [0, 0, 255, 255]);
        let capture = MockScreenCapture::generate_test_mock_screen_capture(
            displays.clone(),
            frame,
        );

        let result = capture.list_displays().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "test-0");
        assert!(result[0].is_primary);
    }
}
```

**Error injection test:**

```rust
#[cfg(test)]
mod tests {
    use crate::traits::{CaptureError, ScreenCapture};
    use crate::traits::types::test_utils::*;
    use crate::mock::MockScreenCapture;

    #[test]
    fn mock_screen_capture_capture_frame_with_error_returns_injected_error() {
        let displays = vec![generate_test_display_info("test-0", 1920, 1080, true)];
        let frame = generate_test_capture_frame(64, 48, [0, 0, 0, 255]);
        let capture = MockScreenCapture::generate_test_mock_screen_capture(
            displays, frame,
        )
        .with_error(|| CaptureError::PermissionDenied);

        let result = capture.capture_frame("test-0", None);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), CaptureError::PermissionDenied)
        );
    }
}
```

**Async TTS test:**

```rust
#[cfg(test)]
mod tests {
    use crate::traits::{TtsEngine, TtsError, Voice, TtsBackend};
    use crate::mock::MockTtsEngine;

    #[tokio::test]
    async fn mock_tts_engine_speak_success() {
        let voices = vec![Voice {
            id: "kokoro-af_heart".to_string(),
            name: "Heart".to_string(),
            language: "en-US".to_string(),
            requires_download: false,
            engine: TtsBackend::Kokoro,
        }];
        let tts = MockTtsEngine::generate_test_mock_tts_engine(voices);

        let result = tts.speak("hello world", false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn mock_tts_engine_speak_with_error_returns_injected_error() {
        let tts = MockTtsEngine::generate_test_mock_tts_engine(vec![])
            .with_error(|| TtsError::VoiceNotFound("missing".into()));

        let result = tts.speak("hello", false).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TtsError::VoiceNotFound(id) if id == "missing"
        ));
    }
}
```

---

## Error Handling

Mock implementations follow the project's error handling conventions:

1. **No `unwrap()` or `expect()` in mock impl blocks** (NFR-4). Error factories may fail to produce errors only if the caller passes a panicking closure -- that is the caller's responsibility.
2. **`unwrap()` is acceptable in `#[cfg(test)]` test functions** -- this is the standard Rust test convention.
3. **Error factory uses `Fn()` (not `FnOnce()`)** -- the factory is called on every method invocation, so it must be repeatable.
4. **Error factory is `Send + Sync + 'static`** -- required because the mock structs must be `Send + Sync` (trait bounds demand it).

Error propagation in mock consumers follows the standard `?` pattern:

```rust
fn init_with_mock(capture: &dyn ScreenCapture) -> Result<(), LuminosError> {
    let displays = capture.list_displays()?; // CaptureError -> LuminosError via From
    Ok(())
}
```

---

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| All platforms | Mocks are platform-independent | No `#[cfg(target_os)]` gates on mock code |
| Linux X11 | Mocks replace XcbCapture, AtSpiTracker, etc. in tests | Same mock interface |
| Linux Wayland | Same mocks as X11 | PipeWireCapture differences are invisible at trait level |
| macOS | Same mocks | SCKitCapture permission issues are bypassed |
| OpenBSD | Same mocks | No AT-SPI2 concern in mock mode |
| Windows | Same mocks | No DXGI/UIA dependency in mock mode |

The entire point of the mock layer is platform independence. Mocks compile and run identically on every platform, enabling CI to test core engine logic on `ubuntu-latest` even though the production capture backend is platform-specific.

---

## Testing Strategy

### Unit Tests

Each mock struct has at least two tests per trait method:
1. **Success path:** Verify the mock returns configured data when no error factory is set
2. **Error path:** Verify the mock returns the injected error when `with_error()` is used

Tests use hierarchical naming: `mock_<trait_name>_<method>_<scenario>`.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Build | `cargo build -p luminos-platform --features test_utils` compiles MockScreenCapture implementing ScreenCapture |
| AC-1.2 | Build | Same build verifies MockFocusTracker compiles |
| AC-1.3 | Build | Same build verifies MockTtsEngine compiles |
| AC-1.4 | Build | Same build verifies MockWindowManager compiles |
| AC-1.5 | Build | Same build verifies MockInputMonitor compiles |
| AC-1.6 | Build | Same build verifies MockAudioOutput compiles |
| AC-2.1 | Unit | `mock_screen_capture_capture_frame_with_error_returns_injected_error` -- construct with `with_error(|| CaptureError::PermissionDenied)`, call `capture_frame`, assert `Err(PermissionDenied)` |
| AC-2.2 | Unit | `mock_focus_tracker_subscribe_with_error_returns_injected_error` -- construct with `with_error(|| FocusError::ApiUnavailable { reason: "test".into() })`, call `subscribe_focus_changes(16)`, assert error reason contains "test" |
| AC-2.3 | Unit + Async | `mock_tts_engine_speak_with_error_returns_injected_error` -- `#[tokio::test]`, construct with `with_error(|| TtsError::VoiceNotFound("missing".into()))`, await `speak`, assert error |
| AC-2.4 | Unit | `mock_window_manager_create_overlay_with_error_returns_injected_error` -- construct with `with_error(|| WindowError::CreationFailed { message: "test".into() })`, call `create_overlay`, assert error |
| AC-2.5 | Unit | `mock_input_monitor_subscribe_with_error_returns_injected_error` -- construct with `with_error(|| InputError::Unavailable { reason: "denied".into() })`, call `subscribe_input_events(32)`, assert error |
| AC-3.1 | Unit | `mock_screen_capture_list_displays_returns_configured_displays` -- construct with display list, call `list_displays`, assert matching |
| AC-3.2 | Unit | `mock_screen_capture_capture_frame_returns_configured_frame` -- construct with display "test-0" and frame, call `capture_frame("test-0", None)`, assert matching frame |
| AC-3.3 | Unit | `mock_screen_capture_capture_frame_unknown_display_returns_not_found` -- call `capture_frame("nonexistent", None)` on mock without error factory, assert `DisplayNotFound` |
| AC-3.4 | Unit | `mock_focus_tracker_get_focused_element_returns_configured_element` -- construct with `Some(event)`, call `get_focused_element`, assert `Ok(Some(event))` |
| AC-3.5 | Unit | `mock_audio_output_play_audio_success` -- construct default, call `play_audio(sample, false)`, assert `Ok(())` |
| AC-4.1 | Build | Downstream crate test with `luminos-platform = { workspace = true, features = ["test_utils"] }` in dev-deps; `use luminos_platform::mock::MockScreenCapture` compiles (verified in Story 004 or E2) |
| AC-4.2 | Build | `cargo build -p luminos-platform` (without `--features test_utils`, outside test mode) succeeds; `mock` module not compiled -- verified by absence from `cargo doc` output |
| AC-4.3 | Inspection | `mock/mod.rs` re-exports all 6 mock structs via `pub use` |
| AC-5.1 | Test run | `cargo nextest run -p luminos-platform` exits 0 with all mock tests passing |
| AC-5.2 | Inspection | Test module contains at least one `mock_screen_capture_*`, `mock_focus_tracker_*`, `mock_tts_engine_*`, `mock_window_manager_*`, `mock_input_monitor_*`, `mock_audio_output_*` test each for success and error paths |
| AC-5.3 | Unit + Async | `mock_tts_engine_speak_success` -- `#[tokio::test]`, construct without error, await `speak`, assert `Ok(())` |
| AC-5.4 | Unit | `mock_screen_capture_subscribe_display_changes_returns_channel` -- call `subscribe_display_changes(16)`, assert `Ok(rx)` where `rx` is a valid receiver |
| AC-5.5 | Unit | `mock_audio_output_play_audio_with_error_returns_no_device` -- construct with `with_error(|| AudioError::NoDevice)`, call `play_audio`, assert `Err(AudioError::NoDevice)` |

### NFR Verification

| NFR | Verification |
|-----|-------------|
| NFR-1 (Send + Sync) | Compile-time: mock structs must satisfy trait bounds `Send + Sync`; `Box<dyn Fn() -> E + Send + Sync>` fields ensure this |
| NFR-2 (nextest passes) | `cargo nextest run -p luminos-platform` exits 0 |
| NFR-3 (Clippy clean) | `cargo clippy -p luminos-platform --features test_utils -- -D warnings` exits 0 |
| NFR-4 (No unwrap in impl) | Manual review + `cargo clippy -- -W clippy::unwrap_used`; `unwrap()` only in `#[cfg(test)]` functions |
| NFR-5 (Doc-comments) | `cargo doc -p luminos-platform --no-deps` with zero warnings |

---

## Performance Targets

Mocks are test-only code with no performance requirements. They return pre-configured data synchronously (or via an immediately-resolving future for `TtsEngine::speak`). Mock method calls are O(1) -- no I/O, no allocation beyond cloning return data.

---

## Security Considerations

**RISK-017 (Screen content leakage):** Mocks handle `CaptureFrame` objects. The custom `Debug` impl (from Story 002) ensures that even in test output, raw pixel data is not printed. Mock constructors accept `CaptureFrame` values created by `generate_test_capture_frame()` which contains synthetic (non-sensitive) pixel data.

**No other security concerns:** Mocks do not interact with platform APIs, network, filesystem, or external processes.

---

## Alternatives Considered

### Alternative 1: Trait-level `mockall` macro vs hand-written mocks

**Rejected approach:** Use the `mockall` crate to auto-generate mock structs from trait definitions.

**Rationale for rejection:**
- `mockall` generates complex types that are hard to debug and produce opaque compiler errors
- `mockall` does not support custom builder patterns like our `with_error()` closure factory approach (it uses `.returning()` which requires cloneable return types or panics)
- Our error types are not `Clone` (they contain `Box<dyn Error>`), which is incompatible with `mockall`'s expectations API
- Hand-written mocks are simple, explicit, and match the doc-02 Section 7.1 pattern exactly
- The six mocks are small (20-40 lines each) -- the cost of hand-writing them is low

**Trade-off:** Hand-written mocks require updating when trait signatures change (RISK-003). This is acceptable because trait changes are compile errors -- if the trait signature changes, the mock fails to compile, and the implementing agent must update it. This is actually a feature, not a cost.

### Alternative 2: Stored error values vs closure factories

**Rejected approach:** Store a `CaptureError` value in the mock struct and `.clone()` it on each call.

**Rationale for rejection:** `CaptureError::Platform { source: Option<Box<dyn Error + Send + Sync>> }` is not `Clone` because `Box<dyn Error>` is not `Clone`. The closure factory pattern (`Box<dyn Fn() -> CaptureError + Send + Sync>`) produces a fresh error on each call, avoiding the clone requirement entirely. This matches doc-02 Section 7.1.

### Alternative 3: Global mock state vs per-instance configuration

**Rejected approach:** Use thread-local storage or global state to configure mock behavior (like some C/C++ mock frameworks).

**Rationale for rejection:** Global state creates test coupling and prevents parallel test execution. Per-instance configuration via constructors and builders is idiomatic Rust, thread-safe, and compatible with `cargo nextest` (which runs tests in parallel by default).
