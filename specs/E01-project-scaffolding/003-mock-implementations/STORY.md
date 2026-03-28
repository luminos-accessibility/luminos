# Story E01/003: Mock Implementations & Test Utilities

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** 002

---

## Problem Statement

The Luminos core engine, rendering pipeline, and TTS pipeline are all developed and tested against the six platform abstraction traits. Without mock implementations of these traits, no unit testing of core logic is possible — every test would require a live display server, GPU, audio device, and accessibility API. This blocks all subsequent epics (E2-E20) from writing testable code.

This story creates mock implementations of all six platform traits in `crates/luminos-platform/src/mock/`, gated behind `#[cfg(any(test, feature = "test_utils"))]`. Each mock is parameterizable via builder methods, supports error injection through closure factories (because error types are not `Clone`), and includes comprehensive unit tests covering both success and error paths. The `test_utils` feature flag enables downstream crates (`luminos-core`, `luminos-gpu`, `luminos-tts`) to import these mocks in their dev-dependencies.

## User Scenarios

### US-1: Mock Structs Compile and Implement Their Traits

As a contributing developer, I want mock implementations of all six platform traits that compile correctly and satisfy the trait contracts so that I can write unit tests for core engine logic without platform dependencies.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given the `MockScreenCapture` struct in `crates/luminos-platform/src/mock/capture.rs`, when `cargo build -p luminos-platform --features test_utils` is run, then it compiles with zero errors and `MockScreenCapture` implements the `ScreenCapture` trait (all three methods: `list_displays`, `capture_frame`, `subscribe_display_changes`).
- **AC-1.2:** Given the `MockFocusTracker` struct in `crates/luminos-platform/src/mock/focus.rs`, when `cargo build -p luminos-platform --features test_utils` is run, then it compiles with zero errors and `MockFocusTracker` implements the `FocusTracker` trait (all three methods: `subscribe_focus_changes`, `get_focused_element`, `get_element_bounds`).
- **AC-1.3:** Given the `MockTtsEngine` struct in `crates/luminos-platform/src/mock/tts.rs`, when `cargo build -p luminos-platform --features test_utils` is run, then it compiles with zero errors and `MockTtsEngine` implements the `TtsEngine` trait (all seven methods: `speak`, `stop`, `set_voice`, `set_rate`, `set_pitch`, `get_voices`, `is_speaking`).
- **AC-1.4:** Given the `MockWindowManager` struct in `crates/luminos-platform/src/mock/window.rs`, when `cargo build -p luminos-platform --features test_utils` is run, then it compiles with zero errors and `MockWindowManager` implements the `WindowManager` trait (all seven methods: `create_overlay`, `set_overlay_bounds`, `set_overlay_mode`, `set_always_on_top`, `set_visible`, `raw_window_handle`, `raw_display_handle`).
- **AC-1.5:** Given the `MockInputMonitor` struct in `crates/luminos-platform/src/mock/input.rs`, when `cargo build -p luminos-platform --features test_utils` is run, then it compiles with zero errors and `MockInputMonitor` implements the `InputMonitor` trait (both methods: `subscribe_input_events`, `get_mouse_position`).
- **AC-1.6:** Given the `MockAudioOutput` struct in `crates/luminos-platform/src/mock/audio.rs`, when `cargo build -p luminos-platform --features test_utils` is run, then it compiles with zero errors and `MockAudioOutput` implements the `AudioOutput` trait (all four methods: `play_audio`, `stop_audio`, `set_volume`, `get_default_device_name`).

### US-2: Error Injection via Builder Pattern

As a contributing developer, I want to configure mocks to return specific error variants via a `with_error()` builder method so that I can test error handling paths in core engine logic without triggering real platform failures.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given a `MockScreenCapture` constructed via `generate_test_mock_screen_capture(displays, frame).with_error(|| CaptureError::PermissionDenied)`, when `capture_frame("any-id", None)` is called, then it returns `Err(CaptureError::PermissionDenied)`.
- **AC-2.2:** Given a `MockFocusTracker` constructed with `.with_error(|| FocusError::ApiUnavailable { reason: "test".into() })`, when `subscribe_focus_changes(16)` is called, then it returns `Err(FocusError::ApiUnavailable { reason })` where `reason` contains `"test"`.
- **AC-2.3:** Given a `MockTtsEngine` constructed with `.with_error(|| TtsError::VoiceNotFound("missing".into()))`, when `speak("hello", false).await` is called, then it returns `Err(TtsError::VoiceNotFound(id))` where `id` is `"missing"`.
- **AC-2.4:** Given a `MockWindowManager` constructed with `.with_error(|| WindowError::CreationFailed { message: "test".into() })`, when `create_overlay("display-0")` is called, then it returns `Err(WindowError::CreationFailed { message })` where `message` contains `"test"`.
- **AC-2.5:** Given a `MockInputMonitor` constructed with `.with_error(|| InputError::Unavailable { reason: "denied".into() })`, when `subscribe_input_events(32)` is called, then it returns `Err(InputError::Unavailable { reason })` where `reason` contains `"denied"`.

### US-3: Happy-Path Mock Behavior Returns Valid Data

As a contributing developer, I want mocks constructed without error injection to return sensible default data so that I can write happy-path tests without extensive setup.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given a `MockScreenCapture` constructed with a list of `DisplayInfo` and a `CaptureFrame` via `generate_test_mock_screen_capture(displays, frame)`, when `list_displays()` is called, then it returns `Ok(displays)` matching the provided display list.
- **AC-3.2:** Given a `MockScreenCapture` constructed with a display with id `"test-0"` and a `CaptureFrame`, when `capture_frame("test-0", None)` is called, then it returns `Ok(frame)` matching the provided frame.
- **AC-3.3:** Given a `MockScreenCapture` constructed with a display with id `"test-0"`, when `capture_frame("nonexistent", None)` is called, then it returns `Err(CaptureError::DisplayNotFound("nonexistent".into()))` — validating that the mock checks the display ID even in the happy path.
- **AC-3.4:** Given a `MockFocusTracker` constructed with default parameters, when `get_focused_element()` is called, then it returns `Ok(Some(event))` or `Ok(None)` depending on configuration — not a panic or unimplemented error.
- **AC-3.5:** Given a `MockAudioOutput` constructed with default parameters, when `play_audio(sample, false)` is called with a valid `AudioSample`, then it returns `Ok(())`.

### US-4: Feature Gate Exports Mocks to Downstream Crates

As a developer working on `luminos-core` or `luminos-gpu`, I want to import mock implementations from `luminos-platform` via a `test_utils` feature flag in my `[dev-dependencies]` so that I can use them in my crate's unit tests.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given a downstream crate's `Cargo.toml` with `luminos-platform = { workspace = true, features = ["test_utils"] }` in `[dev-dependencies]`, when a test file imports `use luminos_platform::mock::MockScreenCapture`, then it compiles successfully.
- **AC-4.2:** Given the `luminos-platform` crate built without the `test_utils` feature and outside of `#[cfg(test)]`, when the `mock` module is referenced, then the compiler reports an error (the module does not exist in non-test, non-feature builds).
- **AC-4.3:** Given the `crates/luminos-platform/src/mock/mod.rs` file, when inspected, then it re-exports all six mock structs (`MockScreenCapture`, `MockFocusTracker`, `MockTtsEngine`, `MockWindowManager`, `MockInputMonitor`, `MockAudioOutput`) and their constructor functions.

### US-5: Unit Tests Pass for Every Mock Method

As a contributing developer, I want comprehensive unit tests covering every mock method in both success and error paths so that I can trust the mocks behave correctly when used in downstream tests.

**Priority:** P0
**Acceptance Criteria:**

- **AC-5.1:** Given the `luminos-platform` crate, when `cargo nextest run -p luminos-platform` is run, then all mock-related unit tests pass with zero failures.
- **AC-5.2:** Given the test suite, when inspected, then there exists at least one success-path test and one error-path test for each of the six mock structs, using hierarchical test names: `mock_screen_capture_*`, `mock_focus_tracker_*`, `mock_tts_engine_*`, `mock_window_manager_*`, `mock_input_monitor_*`, `mock_audio_output_*`.
- **AC-5.3:** Given the `MockTtsEngine` mock, when its `speak` method is tested in an async context (e.g., `#[tokio::test]`), then the returned future resolves to `Ok(())` in the success path and to the configured error in the error path.
- **AC-5.4:** Given the `MockScreenCapture` mock constructed via `generate_test_mock_screen_capture(displays, frame)`, when `subscribe_display_changes(16)` is called, then it returns `Ok(rx)` where `rx` is a valid `mpsc::Receiver` (the channel exists but produces no events, since no real display changes occur in mock mode).
- **AC-5.5:** Given the `MockAudioOutput` mock constructed with `.with_error(|| AudioError::NoDevice)`, when `play_audio(sample, false)` is called, then it returns `Err(AudioError::NoDevice)`.

## Functional Requirements

- **FR-1:** Implement `MockScreenCapture` struct with `generate_test_mock_screen_capture(displays: Vec<DisplayInfo>, frame: CaptureFrame) -> Self` constructor and `with_error<F: Fn() -> CaptureError + Send + Sync + 'static>(self, factory: F) -> Self` builder, implementing the `ScreenCapture` trait. *(Traced by AC-1.1, AC-2.1, AC-3.1, AC-3.2, AC-3.3)*
- **FR-2:** Implement `MockFocusTracker` struct with `generate_test_mock_focus_tracker(...) -> Self` constructor and `with_error` builder, implementing the `FocusTracker` trait. *(Traced by AC-1.2, AC-2.2, AC-3.4)*
- **FR-3:** Implement `MockTtsEngine` struct with `generate_test_mock_tts_engine(...) -> Self` constructor and `with_error` builder, implementing the `TtsEngine` trait. The `speak` method must return a boxed future compatible with the trait's `Pin<Box<dyn Future>>` signature. *(Traced by AC-1.3, AC-2.3, AC-5.3)*
- **FR-4:** Implement `MockWindowManager` struct with `generate_test_mock_window_manager() -> Self` constructor and `with_error` builder, implementing the `WindowManager` trait. The `raw_window_handle()` and `raw_display_handle()` methods must return `None` (no real window exists). *(Traced by AC-1.4, AC-2.4)*
- **FR-5:** Implement `MockInputMonitor` struct with `generate_test_mock_input_monitor(...) -> Self` constructor and `with_error` builder, implementing the `InputMonitor` trait. *(Traced by AC-1.5, AC-2.5)*
- **FR-6:** Implement `MockAudioOutput` struct with `generate_test_mock_audio_output() -> Self` constructor and `with_error` builder, implementing the `AudioOutput` trait. *(Traced by AC-1.6, AC-3.5, AC-5.5)*
- **FR-7:** Gate all mock code behind `#[cfg(any(test, feature = "test_utils"))]`. *(Traced by AC-4.2)*
- **FR-8:** Create `crates/luminos-platform/src/mock/mod.rs` re-exporting all six mock structs and their constructor functions. *(Traced by AC-4.3)*
- **FR-9:** Write unit tests for every mock method in both success and error paths, using hierarchical test naming (`mock_<trait_name>_<method>_<scenario>`). *(Traced by AC-5.1, AC-5.2)*
- **FR-10:** Error injection uses closure factories (`Box<dyn Fn() -> XxxError + Send + Sync>`) because error types are not `Clone` (they may contain `Box<dyn Error>`). Each call to a mock method that checks the error factory calls the closure to produce a fresh error value. *(Traced by AC-2.1 through AC-2.5)*

## Non-Functional Requirements

- **NFR-1:** All six mock structs must be `Send + Sync` (required by the trait bounds). This is automatically satisfied if all stored fields are `Send + Sync`, which they are since error factories use `Box<dyn Fn() -> E + Send + Sync>`.
- **NFR-2:** `cargo nextest run -p luminos-platform` must pass all mock tests with zero failures.
- **NFR-3:** `cargo clippy -p luminos-platform --features test_utils -- -D warnings` must pass with zero warnings.
- **NFR-4:** No `unwrap()` or `expect()` in mock production code (the impl blocks). `unwrap()` is acceptable in `#[cfg(test)]` test functions.
- **NFR-5:** All mock structs and their public methods must have `///` doc-comments explaining their purpose and usage patterns.

## Out of Scope

- Integration tests with real platform APIs (E2+).
- Core engine testing using these mocks (later epics use the mocks; this story only creates and unit-tests them).
- The `LuminosError` hierarchy and `From` conversions (Story 004).
- Pipeline integration tests (doc-02 Section 7.4 — deferred to E2 when the render pipeline exists).
- Mock implementations for types outside `luminos-platform` (e.g., `MockEspeakSubprocess` in `luminos-tts` — deferred to E10).
- Parameterizable `subscribe_display_changes` behavior (the mock returns an empty channel; real display change simulation is deferred to integration tests).

## Open Questions

*None — all questions resolved during epic planning.*
