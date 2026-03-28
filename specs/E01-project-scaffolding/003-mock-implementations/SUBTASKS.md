# Subtasks: Story E01/003 -- Mock Implementations & Test Utilities

**Status:** NOT STARTED
**Started:** ---
**Completed:** ---
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 2 | 0 | 0 | 2 |
| 2. Core Implementation | 6 | 0 | 0 | 6 |
| 3. Integration | 2 | 0 | 0 | 2 |
| 4. Polish & Acceptance | 2 | 0 | 0 | 2 |
| **Total** | **12** | **0** | **0** | **12** |

---

## Phase 1: Setup

### T001 -- Create mock module directory and file stubs

**Traces to:** FR-7, FR-8, AC-4.2, AC-4.3
**Status:** TODO
**Files:** `crates/luminos-platform/src/mock/mod.rs`, `crates/luminos-platform/src/mock/capture.rs`, `crates/luminos-platform/src/mock/focus.rs`, `crates/luminos-platform/src/mock/tts.rs`, `crates/luminos-platform/src/mock/window.rs`, `crates/luminos-platform/src/mock/input.rs`, `crates/luminos-platform/src/mock/audio.rs`

**Steps (no TDD -- scaffolding only):**
- [ ] Create `crates/luminos-platform/src/mock/` directory
- [ ] Create empty `capture.rs`, `focus.rs`, `tts.rs`, `window.rs`, `input.rs`, `audio.rs` files (with module-level doc-comments only)
- [ ] Create `mod.rs` with `pub mod` declarations for all six sub-modules (no re-exports yet -- those come in T009)
- [ ] Verify `#[cfg(any(test, feature = "test_utils"))]` gate exists on the `mock` module declaration in `lib.rs` (from Story 002). If not present, add it.
- [ ] Verify `cargo build -p luminos-platform` compiles with the new empty modules

**Completion Notes:**
>

---

### T002 -- Wire test_utils feature flag in Cargo.toml

**Traces to:** FR-7, AC-4.1, AC-4.2
**Status:** TODO
**Files:** `crates/luminos-platform/Cargo.toml`

**Steps (no TDD -- configuration only):**
- [ ] Verify `test_utils` feature is declared in `crates/luminos-platform/Cargo.toml` under `[features]` (from Story 001). If not present, add: `test_utils = []`
- [ ] Add `tokio` as a dev-dependency if not already present: `tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }` (needed for `#[tokio::test]` in MockTtsEngine tests)
- [ ] Verify `cargo build -p luminos-platform --features test_utils` compiles
- [ ] Verify `cargo build -p luminos-platform` (without feature) compiles and does NOT include mock module (outside of test context)

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 1, verify:
- [ ] `cargo build -p luminos-platform` compiles with zero errors
- [ ] `cargo build -p luminos-platform --features test_utils` compiles with zero errors
- [ ] The `mock/` directory exists with six empty module files and a `mod.rs`

---

## Phase 2: Core Implementation

Each task below implements one mock struct following the pattern from DESIGN.md: struct definition, constructor (`generate_test_mock_<trait>()`), `with_error()` builder, trait implementation, and unit tests (success + error paths). All six tasks are independent and can run in parallel.

### T003 [P] -- Implement MockScreenCapture with tests

**Traces to:** FR-1, FR-10, AC-1.1, AC-2.1, AC-3.1, AC-3.2, AC-3.3, AC-5.4
**Status:** TODO
**Files:** `crates/luminos-platform/src/mock/capture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `mock_screen_capture_list_displays_returns_configured_displays` -- Construct with 2 displays, call `list_displays()`, assert `Ok(displays)` with matching IDs and count (AC-3.1)
   - [ ] `mock_screen_capture_capture_frame_returns_configured_frame` -- Construct with display "test-0" and a frame, call `capture_frame("test-0", None)`, assert returned frame matches (AC-3.2)
   - [ ] `mock_screen_capture_capture_frame_unknown_display_returns_not_found` -- Call `capture_frame("nonexistent", None)` on mock without error factory, assert `Err(CaptureError::DisplayNotFound(...))` (AC-3.3)
   - [ ] `mock_screen_capture_capture_frame_with_error_returns_injected_error` -- Construct with `.with_error(|| CaptureError::PermissionDenied)`, call `capture_frame(...)`, assert `Err(CaptureError::PermissionDenied)` (AC-2.1)
   - [ ] `mock_screen_capture_list_displays_with_error_returns_injected_error` -- Same error factory, call `list_displays()`, assert error
   - [ ] `mock_screen_capture_subscribe_display_changes_returns_channel` -- Call `subscribe_display_changes(16)`, assert `Ok(rx)` where `rx` is a valid receiver (AC-5.4)
   - [ ] `mock_screen_capture_subscribe_display_changes_with_error` -- With error factory, call `subscribe_display_changes(16)`, assert error
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `MockScreenCapture` struct with `displays: Vec<DisplayInfo>`, `frame: CaptureFrame`, `error_factory: Option<Box<dyn Fn() -> CaptureError + Send + Sync>>`
   - [ ] Implement `generate_test_mock_screen_capture(displays, frame) -> Self` constructor
   - [ ] Implement `with_error<F>(self, factory: F) -> Self` builder
   - [ ] Implement `ScreenCapture` for `MockScreenCapture` (all 3 methods: `list_displays`, `capture_frame`, `subscribe_display_changes`)
   - [ ] Gate all code with `#[cfg(any(test, feature = "test_utils"))]`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to struct, constructor, builder, and trait methods per DESIGN.md
   - [ ] Verify no `unwrap()` in impl block (only in tests)

**Completion Notes:**
>

---

### T004 [P] -- Implement MockFocusTracker with tests

**Traces to:** FR-2, FR-10, AC-1.2, AC-2.2, AC-3.4
**Status:** TODO
**Files:** `crates/luminos-platform/src/mock/focus.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `mock_focus_tracker_get_focused_element_returns_configured_element` -- Construct with `Some(event)`, call `get_focused_element()`, assert `Ok(Some(event))` (AC-3.4)
   - [ ] `mock_focus_tracker_get_focused_element_returns_none_when_unconfigured` -- Construct with `None`, call `get_focused_element()`, assert `Ok(None)`
   - [ ] `mock_focus_tracker_subscribe_focus_changes_returns_channel` -- Call `subscribe_focus_changes(16)`, assert `Ok(rx)`
   - [ ] `mock_focus_tracker_subscribe_with_error_returns_injected_error` -- Construct with `.with_error(|| FocusError::ApiUnavailable { reason: "test".into() })`, call `subscribe_focus_changes(16)`, assert error reason contains "test" (AC-2.2)
   - [ ] `mock_focus_tracker_get_element_bounds_returns_configured_bounds` -- Construct with focused element, call `get_element_bounds(element_id)`, assert `Ok(Some(bounds))`. Note: pass any element_id string -- the mock does not validate IDs. It returns bounds from the configured focused_element regardless of the ID passed.
   - [ ] `mock_focus_tracker_get_element_bounds_with_error` -- With error factory, call `get_element_bounds(...)`, assert error
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `MockFocusTracker` struct with `focused_element: Option<FocusChangedEvent>`, `error_factory` field
   - [ ] Implement `generate_test_mock_focus_tracker(focused_element) -> Self` constructor
   - [ ] Implement `with_error` builder
   - [ ] Implement `FocusTracker` for `MockFocusTracker` (all 3 methods)
   - [ ] Gate all code with `#[cfg(any(test, feature = "test_utils"))]`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments per DESIGN.md

**Completion Notes:**
>

---

### T005 [P] -- Implement MockTtsEngine with async tests

**Traces to:** FR-3, FR-10, AC-1.3, AC-2.3, AC-5.3
**Status:** TODO
**Files:** `crates/luminos-platform/src/mock/tts.rs`

**TDD Cycle:**
1. **Red** -- Write test(s) (use `#[tokio::test]` for async `speak` method):
   - [ ] `mock_tts_engine_speak_success` -- Construct with voices, await `speak("hello", false)`, assert `Ok(())` (AC-5.3)
   - [ ] `mock_tts_engine_speak_with_error_returns_injected_error` -- Construct with `.with_error(|| TtsError::VoiceNotFound("missing".into()))`, await `speak("hello", false)`, assert error matches with id "missing" (AC-2.3)
   - [ ] `mock_tts_engine_stop_success` -- Call `stop()`, assert `Ok(())`
   - [ ] `mock_tts_engine_stop_with_error` -- With error factory, call `stop()`, assert error
   - [ ] `mock_tts_engine_set_voice_success` -- Call `set_voice("kokoro-af_heart")`, assert `Ok(())`
   - [ ] `mock_tts_engine_set_rate_success` -- Call `set_rate(1.5)`, assert `Ok(())`
   - [ ] `mock_tts_engine_set_pitch_success` -- Call `set_pitch(1.0)`, assert `Ok(())`
   - [ ] `mock_tts_engine_get_voices_returns_configured_voices` -- Construct with voice list, call `get_voices()`, assert `Ok(voices)` with matching entries
   - [ ] `mock_tts_engine_get_voices_with_error` -- With error factory, call `get_voices()`, assert error
   - [ ] `mock_tts_engine_is_speaking_returns_false` -- Call `is_speaking()`, assert `false`
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `MockTtsEngine` struct with `voices: Vec<Voice>`, `error_factory` field
   - [ ] Implement `generate_test_mock_tts_engine(voices) -> Self` constructor
   - [ ] Implement `with_error` builder
   - [ ] Implement `TtsEngine` for `MockTtsEngine` (all 7 methods, with `speak` returning `Pin<Box<dyn Future<...>>>`)
   - [ ] Gate all code with `#[cfg(any(test, feature = "test_utils"))]`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments per DESIGN.md
   - [ ] Verify the boxed future resolves immediately (no unnecessary async runtime overhead)

**Completion Notes:**
>

---

### T006 [P] -- Implement MockWindowManager with tests

**Traces to:** FR-4, FR-10, AC-1.4, AC-2.4
**Status:** TODO
**Files:** `crates/luminos-platform/src/mock/window.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `mock_window_manager_create_overlay_success` -- Construct default, call `create_overlay("display-0")`, assert `Ok(())`
   - [ ] `mock_window_manager_create_overlay_with_error_returns_injected_error` -- Construct with `.with_error(|| WindowError::CreationFailed { message: "test".into() })`, call `create_overlay("display-0")`, assert error message contains "test" (AC-2.4)
   - [ ] `mock_window_manager_set_overlay_bounds_success` -- Call `set_overlay_bounds(rect)`, assert `Ok(())`
   - [ ] `mock_window_manager_set_overlay_mode_success` -- Call `set_overlay_mode(OverlayMode::FullScreen)`, assert `Ok(())`
   - [ ] `mock_window_manager_set_always_on_top_success` -- Call `set_always_on_top(true)`, assert `Ok(())`
   - [ ] `mock_window_manager_set_visible_success` -- Call `set_visible(true)`, assert `Ok(())`
   - [ ] `mock_window_manager_raw_window_handle_returns_none` -- Call `raw_window_handle()`, assert `None` (FR-4)
   - [ ] `mock_window_manager_raw_display_handle_returns_none` -- Call `raw_display_handle()`, assert `None` (FR-4)
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `MockWindowManager` struct with `overlay_created: bool`, `error_factory` field
   - [ ] Implement `generate_test_mock_window_manager() -> Self` constructor
   - [ ] Implement `with_error` builder
   - [ ] Implement `WindowManager` for `MockWindowManager` (all 7 methods; `raw_window_handle()` and `raw_display_handle()` return `None`)
   - [ ] Gate all code with `#[cfg(any(test, feature = "test_utils"))]`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments per DESIGN.md

**Completion Notes:**
>

---

### T007 [P] -- Implement MockInputMonitor with tests

**Traces to:** FR-5, FR-10, AC-1.5, AC-2.5
**Status:** TODO
**Files:** `crates/luminos-platform/src/mock/input.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `mock_input_monitor_get_mouse_position_returns_configured_position` -- Construct with `ScreenPoint { x: 100, y: 200 }`, call `get_mouse_position()`, assert `Ok(ScreenPoint { x: 100, y: 200 })`
   - [ ] `mock_input_monitor_subscribe_input_events_returns_channel` -- Call `subscribe_input_events(32)`, assert `Ok(rx)` where `rx` is a valid receiver
   - [ ] `mock_input_monitor_subscribe_with_error_returns_injected_error` -- Construct with `.with_error(|| InputError::Unavailable { reason: "denied".into() })`, call `subscribe_input_events(32)`, assert error reason contains "denied" (AC-2.5)
   - [ ] `mock_input_monitor_get_mouse_position_with_error` -- With error factory, call `get_mouse_position()`, assert error
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `MockInputMonitor` struct with `mouse_position: ScreenPoint`, `error_factory` field
   - [ ] Implement `generate_test_mock_input_monitor(mouse_position) -> Self` constructor
   - [ ] Implement `with_error` builder
   - [ ] Implement `InputMonitor` for `MockInputMonitor` (both methods)
   - [ ] Gate all code with `#[cfg(any(test, feature = "test_utils"))]`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments per DESIGN.md

**Completion Notes:**
>

---

### T008 [P] -- Implement MockAudioOutput with tests

**Traces to:** FR-6, FR-10, AC-1.6, AC-3.5, AC-5.5
**Status:** TODO
**Files:** `crates/luminos-platform/src/mock/audio.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `mock_audio_output_play_audio_success` -- Construct default, call `play_audio(sample, false)` with valid `AudioSample`, assert `Ok(())` (AC-3.5)
   - [ ] `mock_audio_output_play_audio_with_error_returns_no_device` -- Construct with `.with_error(|| AudioError::NoDevice)`, call `play_audio(sample, false)`, assert `Err(AudioError::NoDevice)` (AC-5.5)
   - [ ] `mock_audio_output_stop_audio_success` -- Call `stop_audio()`, assert `Ok(())`
   - [ ] `mock_audio_output_stop_audio_with_error` -- With error factory, call `stop_audio()`, assert error
   - [ ] `mock_audio_output_set_volume_success` -- Call `set_volume(0.5)`, assert `Ok(())`
   - [ ] `mock_audio_output_set_volume_with_error` -- With error factory, call `set_volume(0.5)`, assert error
   - [ ] `mock_audio_output_get_default_device_name_returns_configured_name` -- Construct default, call `get_default_device_name()`, assert `Ok(Some("Mock Audio Device".to_string()))`
   - [ ] `mock_audio_output_get_default_device_name_with_error` -- With error factory, call `get_default_device_name()`, assert error
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `MockAudioOutput` struct with `device_name: Option<String>`, `error_factory` field
   - [ ] Implement `generate_test_mock_audio_output() -> Self` constructor (defaults `device_name` to `Some("Mock Audio Device".to_string())`)
   - [ ] Implement `with_error` builder
   - [ ] Implement `AudioOutput` for `MockAudioOutput` (all 4 methods)
   - [ ] Gate all code with `#[cfg(any(test, feature = "test_utils"))]`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments per DESIGN.md

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [ ] All Phase 2 tests pass: `cargo nextest run -p luminos-platform`
- [ ] Each mock struct compiles and satisfies its trait bounds (`Send + Sync`)
- [ ] At least one success-path and one error-path test exists for each of the six mocks

---

## Phase 3: Integration

### T009 -- Wire mod.rs re-exports and verify public API

**Traces to:** FR-8, AC-4.3
**Status:** TODO
**Files:** `crates/luminos-platform/src/mock/mod.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `mock_mod_reexports_all_six_structs` -- In a test using `crate::mock::*`, verify all six mock types are accessible: `MockScreenCapture`, `MockFocusTracker`, `MockTtsEngine`, `MockWindowManager`, `MockInputMonitor`, `MockAudioOutput`
2. **Green** -- Implement:
   - [ ] Update `mock/mod.rs` to add `pub use` re-exports for all six mock structs from their sub-modules (per DESIGN.md `mock/mod.rs` listing)
   - [ ] Add module-level doc-comment to `mod.rs` explaining usage pattern (in-crate vs downstream)
3. **Refactor** -- Clean up:
   - [ ] Verify import paths work from both `crate::mock::MockScreenCapture` and `crate::mock::capture::MockScreenCapture`

**Completion Notes:**
>

---

### T010 -- Verify feature gate isolates mock module

**Traces to:** FR-7, AC-4.1, AC-4.2
**Status:** TODO
**Files:** `crates/luminos-platform/src/lib.rs`

**TDD Cycle:**
1. **Red** -- Write verification steps:
   - [ ] `cargo build -p luminos-platform` (no features, not in test mode) succeeds and the `mock` module is NOT compiled (verify by checking no mock-related symbols in output or by confirming the `#[cfg]` gate on the `pub mod mock;` declaration in `lib.rs`)
   - [ ] `cargo build -p luminos-platform --features test_utils` succeeds and the `mock` module IS compiled
2. **Green** -- Verify/fix:
   - [ ] Confirm `lib.rs` has `#[cfg(any(test, feature = "test_utils"))] pub mod mock;`
   - [ ] If a downstream crate (e.g., `luminos-core`) needs mocks in dev-deps, verify the import path works: add a temporary compile-only test in `luminos-core` that does `use luminos_platform::mock::MockScreenCapture;` with `luminos-platform` as a `[dev-dependencies]` with `features = ["test_utils"]`
3. **Refactor** -- Clean up:
   - [ ] Remove any temporary tests added for verification (or keep as a permanent downstream import smoke test if appropriate for Story 004)

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 3, verify:
- [ ] `crate::mock::MockScreenCapture` (and all five others) are importable from `mock/mod.rs`
- [ ] Building without `test_utils` feature (and outside `#[cfg(test)]`) excludes the `mock` module
- [ ] Building with `test_utils` feature includes the `mock` module

---

## Phase 4: Polish & Acceptance

### T011 -- Clippy, formatting, and doc-comment audit

**Traces to:** NFR-3, NFR-4, NFR-5
**Status:** TODO
**Files:** All `crates/luminos-platform/src/mock/*.rs` files

**Steps:**
- [ ] Run `cargo clippy -p luminos-platform --features test_utils -- -D warnings` and fix any warnings
- [ ] Run `cargo clippy -p luminos-platform --features test_utils -- -W clippy::unwrap_used -W clippy::expect_used` and verify zero warnings in `mock/` impl blocks (only in `#[cfg(test)]` blocks is `unwrap()` allowed)
- [ ] Run `cargo fmt --all -- --check` and fix any formatting issues
- [ ] Run `cargo doc -p luminos-platform --features test_utils --no-deps` and verify zero warnings for mock module docs
- [ ] Verify every public struct, method, and constructor in `mock/` has a `///` doc-comment

**Completion Notes:**
>

---

### T012 -- Full acceptance test verification

**Traces to:** All ACs (AC-1.1 through AC-5.5)
**Status:** TODO
**Files:** None (verification only)

**Verification Checklist:**
- [ ] AC-1.1: `MockScreenCapture` implements `ScreenCapture` (verified by `cargo build -p luminos-platform --features test_utils`)
- [ ] AC-1.2: `MockFocusTracker` implements `FocusTracker` (verified by same build)
- [ ] AC-1.3: `MockTtsEngine` implements `TtsEngine` (verified by same build)
- [ ] AC-1.4: `MockWindowManager` implements `WindowManager` (verified by same build)
- [ ] AC-1.5: `MockInputMonitor` implements `InputMonitor` (verified by same build)
- [ ] AC-1.6: `MockAudioOutput` implements `AudioOutput` (verified by same build)
- [ ] AC-2.1: `mock_screen_capture_capture_frame_with_error_returns_injected_error` passes (T003)
- [ ] AC-2.2: `mock_focus_tracker_subscribe_with_error_returns_injected_error` passes (T004)
- [ ] AC-2.3: `mock_tts_engine_speak_with_error_returns_injected_error` passes (T005)
- [ ] AC-2.4: `mock_window_manager_create_overlay_with_error_returns_injected_error` passes (T006)
- [ ] AC-2.5: `mock_input_monitor_subscribe_with_error_returns_injected_error` passes (T007)
- [ ] AC-3.1: `mock_screen_capture_list_displays_returns_configured_displays` passes (T003)
- [ ] AC-3.2: `mock_screen_capture_capture_frame_returns_configured_frame` passes (T003)
- [ ] AC-3.3: `mock_screen_capture_capture_frame_unknown_display_returns_not_found` passes (T003)
- [ ] AC-3.4: `mock_focus_tracker_get_focused_element_returns_configured_element` passes (T004)
- [ ] AC-3.5: `mock_audio_output_play_audio_success` passes (T008)
- [ ] AC-4.1: Downstream crate can import mocks via `test_utils` feature (T010)
- [ ] AC-4.2: Mock module excluded from non-test, non-feature builds (T010)
- [ ] AC-4.3: `mock/mod.rs` re-exports all six mock structs (T009)
- [ ] AC-5.1: `cargo nextest run -p luminos-platform` exits 0 with all mock tests passing
- [ ] AC-5.2: Test suite contains hierarchically-named tests for each mock (success + error paths)
- [ ] AC-5.3: `mock_tts_engine_speak_success` passes with `#[tokio::test]` (T005)
- [ ] AC-5.4: `mock_screen_capture_subscribe_display_changes_returns_channel` passes (T003)
- [ ] AC-5.5: `mock_audio_output_play_audio_with_error_returns_no_device` passes (T008)
- [ ] NFR-1: All mock structs are `Send + Sync` (verified by trait bounds compiling)
- [ ] NFR-2: `cargo nextest run -p luminos-platform` exits 0
- [ ] NFR-3: `cargo clippy -p luminos-platform --features test_utils -- -D warnings` exits 0
- [ ] NFR-4: No `unwrap()` in mock impl blocks (only in `#[cfg(test)]`)
- [ ] NFR-5: All public items have doc-comments

**Completion Notes:**
>

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| --- | --- | --- |
