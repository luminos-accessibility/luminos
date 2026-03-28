# Story E01/002: Platform Trait Definitions & Common Types

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DONE
**Depends On:** 001

---

## Problem Statement

The Luminos architecture relies on six platform abstraction traits as the central coordination contract between the core engine and every platform backend. Until these traits are defined in compilable Rust code with correct signatures, doc-comments, and associated types, no subsequent work can proceed: mock implementations (Story 003), error hierarchy (Story 004), and every epic from E2 onward depend on these definitions existing.

This story translates the canonical trait specifications from [doc-02 Sections 3.1-3.7](../../tech-strategy/02-platform-abstraction.md#3-trait-definitions) into compilable Rust code in the `luminos-platform` crate. It also establishes the module structure (lib.rs with `#[cfg]`-gated backend stubs), the `PlatformBackends` bundle struct, and co-located test generators for common types.

## User Scenarios

### US-1: All Six Traits Compile with Correct Method Signatures

As a contributing developer, I want all six platform abstraction traits defined with the exact method signatures from doc-02 so that I can implement platform backends and mock implementations against a compiler-enforced contract.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given the `luminos-platform` crate, when `cargo build -p luminos-platform` is run, then it compiles with zero errors and zero warnings.
- **AC-1.2:** Given the `ScreenCapture` trait in `crates/luminos-platform/src/traits.rs` (or `traits/` module), when inspected, then it declares exactly three methods: `fn list_displays(&self) -> Result<Vec<DisplayInfo>, CaptureError>`, `fn capture_frame(&self, display_id: &str, region: Option<ScreenRect>) -> Result<CaptureFrame, CaptureError>`, and `fn subscribe_display_changes(&self, buffer_size: usize) -> Result<tokio::sync::mpsc::Receiver<DisplayChangeEvent>, CaptureError>`, and the trait is bounded by `Send + Sync`.
- **AC-1.3:** Given the `FocusTracker` trait, when inspected, then it declares exactly three methods: `fn subscribe_focus_changes(&self, buffer_size: usize) -> Result<mpsc::Receiver<FocusChangedEvent>, FocusError>`, `fn get_focused_element(&self) -> Result<Option<FocusChangedEvent>, FocusError>`, and `fn get_element_bounds(&self, element_id: &str) -> Result<Option<ScreenRect>, FocusError>`, and the trait is bounded by `Send + Sync`.
- **AC-1.4:** Given the `TtsEngine` trait, when inspected, then it declares exactly seven methods: `speak(&self, text: &str, interrupt: bool) -> Pin<Box<dyn Future<Output = Result<(), TtsError>> + Send + '_>>`, `stop(&self) -> Result<(), TtsError>`, `set_voice(&self, voice_id: &str) -> Result<(), TtsError>`, `set_rate(&self, rate: f32) -> Result<(), TtsError>`, `set_pitch(&self, pitch: f32) -> Result<(), TtsError>`, `get_voices(&self) -> Result<Vec<Voice>, TtsError>`, and `is_speaking(&self) -> bool`, and the trait is bounded by `Send + Sync`.
- **AC-1.5:** Given the `WindowManager` trait, when inspected, then it declares exactly seven methods: `create_overlay(&mut self, display_id: &str) -> Result<(), WindowError>`, `set_overlay_bounds(&self, bounds: ScreenRect) -> Result<(), WindowError>`, `set_overlay_mode(&mut self, mode: OverlayMode) -> Result<(), WindowError>`, `set_always_on_top(&self, always_on_top: bool) -> Result<(), WindowError>`, `set_visible(&self, visible: bool) -> Result<(), WindowError>`, `raw_window_handle(&self) -> Option<&dyn raw_window_handle::HasWindowHandle>`, and `raw_display_handle(&self) -> Option<&dyn raw_window_handle::HasDisplayHandle>`, and the trait is bounded by `Send + Sync`.

### US-2: All Common Types Have Correct Fields and Derive Traits

As a contributing developer, I want the shared types (`ScreenRect`, `ScreenPoint`, `DisplayInfo`, `PixelFormat`, `CaptureFrame`, `AudioSample`, `Voice`) defined with the exact fields and derive attributes from doc-02 so that all crates referencing these types get correct, consistent definitions.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given the `ScreenRect` struct, when inspected, then it has fields `x: i32`, `y: i32`, `width: u32`, `height: u32` and derives `Debug, Clone, Copy, PartialEq, Eq, Hash`.
- **AC-2.2:** Given the `ScreenPoint` struct, when inspected, then it has fields `x: i32`, `y: i32` and derives `Debug, Clone, Copy, PartialEq, Eq, Hash`.
- **AC-2.3:** Given the `DisplayInfo` struct, when inspected, then it has fields `id: String`, `name: String`, `bounds: ScreenRect`, `scale_factor: f64`, `is_primary: bool` and derives `Debug, Clone, PartialEq`.
- **AC-2.4:** Given the `CaptureFrame` struct, when inspected, then it has fields `data: Arc<[u8]>`, `width: u32`, `height: u32`, `stride: u32`, `format: PixelFormat`, and derives `Clone` but NOT the standard `Debug` derive (see AC-5.1 for custom Debug).
- **AC-2.5:** Given the `AudioSample` struct, when inspected, then it has fields `data: Vec<f32>`, `sample_rate: u32`, `channels: u16` and derives `Debug, Clone`.
- **AC-2.6:** Given the `Voice` struct, when inspected, then it has fields `id: String`, `name: String`, `language: String`, `requires_download: bool`, `engine: TtsBackend` and derives `Debug, Clone`. The `TtsBackend` enum has exactly three variants: `Kokoro`, `Piper`, `Native`, and derives `Debug, Clone, PartialEq, Eq`.

### US-3: All Error Enums Derive thiserror::Error with Correct Variants

As a contributing developer, I want all six subsystem error enums defined with every variant from doc-02 and `thiserror::Error` derivation so that errors produce actionable display messages and support `?` propagation.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given the `CaptureError` enum, when inspected, then it has exactly five variants: `DisplayNotFound(String)`, `RegionOutOfBounds { region: ScreenRect, bounds: ScreenRect }`, `PermissionDenied`, `BackendUnavailable { reason: String }`, and `Platform { message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> }`, and derives `Debug` and `thiserror::Error`.
- **AC-3.2:** Given the `FocusError` enum, when inspected, then it has exactly five variants: `ApiUnavailable { reason: String }`, `PermissionDenied`, `QueryFailed { message: String }`, `Disconnected { message: String }`, and `Platform { message: String }`, and derives `Debug` and `thiserror::Error`.
- **AC-3.3:** Given the `TtsError` enum, when inspected, then it has exactly six variants: `VoiceNotFound(String)`, `ModelLoadFailed { message: String }`, `PhonemizerFailed { message: String }`, `InferenceFailed { message: String }`, `AudioUnavailable { message: String }`, and `Platform { message: String }`, and derives `Debug` and `thiserror::Error`.
- **AC-3.4:** Given the `WindowError` enum, when inspected, then it has exactly five variants: `CreationFailed { message: String }`, `PropertyFailed { property: String, message: String }`, `DisplayNotFound(String)`, `DockFailed { message: String }`, and `Platform { message: String }`, and derives `Debug` and `thiserror::Error`.
- **AC-3.5:** Given the `InputError` enum, when inspected, then it has exactly three variants: `Unavailable { reason: String }`, `Disconnected { message: String }`, and `Platform { message: String }`, and derives `Debug` and `thiserror::Error`.
- **AC-3.6:** Given the `AudioError` enum, when inspected, then it has exactly five variants: `NoDevice`, `DeviceFailed { message: String }`, `PlaybackInterrupted { message: String }`, `UnsupportedFormat { message: String }`, and `Platform { message: String }`, and derives `Debug` and `thiserror::Error`.

### US-4: Module Structure and Platform Backend Stubs

As a contributing developer, I want the `luminos-platform` crate's `lib.rs` to declare all modules with correct `#[cfg]` gates matching doc-02 Section 5.2 so that platform backends, mocks, and common utilities are compiled only for the appropriate targets.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given the `crates/luminos-platform/src/lib.rs` file, when inspected, then it declares `pub mod traits` and `pub mod error` unconditionally, `pub mod mock` gated by `#[cfg(any(test, feature = "test_utils"))]`, `pub(crate) mod common` gated by `#[cfg(any(target_os = "linux", target_os = "openbsd"))]`, and platform backend modules (`linux_x11`, `linux_wayland`, `macos`, `openbsd`, `windows`) each gated by their respective `#[cfg(target_os = "...")]`.
- **AC-4.2:** Given the platform backend modules (`linux_x11`, `linux_wayland`, `macos`, `openbsd`, `windows`) and the `common` module, when inspected, then each is an empty stub (module directory or file with no impl code), compiling without error.
- **AC-4.3:** Given the `PlatformBackends` struct, when inspected, then it has exactly five fields: `capture: Box<dyn ScreenCapture>`, `focus_tracker: Box<dyn FocusTracker>`, `window_mgr: Box<dyn WindowManager>`, `input_monitor: Box<dyn InputMonitor>`, `audio_output: Box<dyn AudioOutput>`, and is declared in `luminos-platform` (not in a `#[cfg]`-gated module).
- **AC-4.4:** Given the `InputMonitor` trait, when inspected, then it declares exactly two methods: `fn subscribe_input_events(&self, buffer_size: usize) -> Result<mpsc::Receiver<InputEvent>, InputError>` and `fn get_mouse_position(&self) -> Result<ScreenPoint, InputError>`, and the trait is bounded by `Send + Sync`.
- **AC-4.5:** Given the `AudioOutput` trait, when inspected, then it declares exactly four methods: `fn play_audio(&self, sample: AudioSample, interrupt: bool) -> Result<(), AudioError>`, `fn stop_audio(&self) -> Result<(), AudioError>`, `fn set_volume(&self, volume: f32) -> Result<(), AudioError>`, and `fn get_default_device_name(&self) -> Result<Option<String>, AudioError>`, and the trait is bounded by `Send + Sync`.

### US-5: Doc-Comments, Privacy Mitigation & Test Generators

As a contributing developer, I want every public item to have doc-comments matching doc-02 descriptions, `CaptureFrame` to have a custom `Debug` impl that omits pixel data (RISK-017), and co-located test generators for common types so that the crate is self-documenting, safe from data leakage, and testable.

**Priority:** P0
**Acceptance Criteria:**

- **AC-5.1:** Given the `CaptureFrame` struct, when formatted with `{:?}`, then the output includes `width`, `height`, `stride`, and `format` fields but does NOT include the raw pixel `data` content (must use a custom `Debug` implementation that omits or redacts the data field, e.g., `data: [<{len} bytes>]`). This mitigates RISK-017 (screen content leakage via logs).
- **AC-5.2:** Given every public trait, struct, enum, method, and variant in the `traits` module, when inspected, then each has a `///` doc-comment that describes its purpose, matching the descriptions in doc-02 Sections 3.1-3.7.
- **AC-5.3:** Given the `generate_test_capture_frame(width: u32, height: u32, color: [u8; 4]) -> CaptureFrame` function in a `#[cfg(test)]` block co-located with `CaptureFrame`, when called with `(64, 48, [0, 0, 255, 255])`, then it returns a `CaptureFrame` with `width == 64`, `height == 48`, `stride == 256` (64 * 4), `format == PixelFormat::Bgra8`, and `data.len() == 12288` (256 * 48).
- **AC-5.4:** Given the `generate_test_display_info(id: &str, width: u32, height: u32, is_primary: bool) -> DisplayInfo` function in a `#[cfg(test)]` block co-located with `DisplayInfo`, when called with `("test-0", 1920, 1080, true)`, then it returns a `DisplayInfo` with `id == "test-0"`, `bounds.width == 1920`, `bounds.height == 1080`, `is_primary == true`, and `scale_factor == 1.0`.

## Functional Requirements

- **FR-1:** Define the `ScreenCapture` trait with all methods and associated types per doc-02 Section 3.2. *(Traced by AC-1.2)*
- **FR-2:** Define the `FocusTracker` trait with all methods and associated types per doc-02 Section 3.3. *(Traced by AC-1.3)*
- **FR-3:** Define the `TtsEngine` trait with all methods and associated types per doc-02 Section 3.4, using `Pin<Box<dyn Future>>` for `speak()` return type to preserve object safety. *(Traced by AC-1.4)*
- **FR-4:** Define the `WindowManager` trait with all methods and associated types per doc-02 Section 3.5, including `raw_window_handle` and `raw_display_handle` methods returning trait objects from the `raw-window-handle` crate. *(Traced by AC-1.5)*
- **FR-5:** Define the `InputMonitor` trait with all methods and associated types per doc-02 Section 3.6. *(Traced by AC-4.4)*
- **FR-6:** Define the `AudioOutput` trait with all methods and associated types per doc-02 Section 3.7. *(Traced by AC-4.5)*
- **FR-7:** Define common types (`ScreenRect`, `ScreenPoint`, `DisplayInfo`, `PixelFormat`, `CaptureFrame`) per doc-02 Section 3.1 with specified derive attributes. *(Traced by AC-2.1, AC-2.2, AC-2.3, AC-2.4)*
- **FR-8:** Define `AudioSample` and `Voice` structs per doc-02 Sections 3.7 and 3.4 respectively. *(Traced by AC-2.5, AC-2.6)*
- **FR-9:** Define all six subsystem error enums (`CaptureError`, `FocusError`, `TtsError`, `WindowError`, `InputError`, `AudioError`) with `thiserror::Error` derivation per doc-02 Sections 3.2-3.7. *(Traced by AC-3.1 through AC-3.6)*
- **FR-10:** Define the `PixelFormat` enum with variants `Bgra8` and `Rgba8`, deriving `Debug, Clone, Copy, PartialEq, Eq, Hash`. *(Traced by AC-2.4)*
- **FR-11:** Define enums `DisplayChangeEvent` (3 variants), `ElementType` (6 variants), `TtsBackend` (3 variants), `OverlayMode` (3 variants), `DockEdge` (4 variants), `LensShape` (2 variants), `InputEvent` (4 variants), `MouseButton` (4 variants), `KeyCode` (full variant list per doc-02 Section 3.6), and `Modifiers` struct per doc-02. *(Traced by AC-1.2 through AC-4.5)*
- **FR-12:** Implement custom `Debug` for `CaptureFrame` that omits the `data` field, printing only metadata. *(Traced by AC-5.1)*
- **FR-13:** Establish `lib.rs` module structure with `#[cfg]` gates matching doc-02 Section 5.2 and empty platform backend stubs. *(Traced by AC-4.1, AC-4.2)*
- **FR-14:** Define the `PlatformBackends` bundle struct with five trait object fields per doc-02 Section 5.3 (`TtsEngine` excluded). *(Traced by AC-4.3)*
- **FR-15:** Provide `generate_test_capture_frame` and `generate_test_display_info` test generator functions co-located with their types in `#[cfg(test)]` blocks. *(Traced by AC-5.3, AC-5.4)*
- **FR-16:** Add `///` doc-comments to every public trait, method, struct, enum, and variant matching doc-02 descriptions. *(Traced by AC-5.2)*

## Non-Functional Requirements

- **NFR-1:** All six traits must be bounded by `Send + Sync` to support multi-threaded access (render thread, IPC thread, input monitoring thread).
- **NFR-2:** No `unwrap()` or `expect()` in any production code path. Exception: `unwrap()` is acceptable in `#[cfg(test)]` blocks.
- **NFR-3:** Every public item (trait, method, struct, field, enum, variant) must have a `///` doc-comment. `cargo doc -p luminos-platform --no-deps` must produce documentation without warnings.
- **NFR-4:** `cargo clippy -p luminos-platform -- -D warnings` must pass with zero warnings.
- **NFR-5:** All error enums must implement `std::error::Error` (via `thiserror`) and `Debug`, enabling `?` propagation and structured logging.

## Out of Scope

- Mock implementations of the six traits (Story 003).
- `LuminosError` top-level error enum in `luminos-core/src/error.rs` and `From` conversions (Story 004).
- Core application types (`AppState`, `AppSettings`, `MagnificationMode`, `TrackingMode`, `ColorFilterType`) (Story 004).
- Real platform backend implementations in `linux_x11/`, `linux_wayland/`, `macos/`, `openbsd/`, `windows/` (E2+).
- CI pipeline setup (Story 005).
- `TtsEngine::speak` method using RPITIT — must use `Pin<Box<dyn Future>>` for object safety.
- Feature flag definitions in `Cargo.toml` (`wayland`, `xshm`, `test_utils`) (Story 001).

## Open Questions

*None — all questions resolved during epic planning.*
