# Subtasks: Story E01/002 -- Platform Trait Definitions & Common Types

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
| 2. Core Implementation | 8 | 0 | 0 | 8 |
| 3. Integration | 2 | 0 | 0 | 2 |
| 4. Polish & Acceptance | 2 | 0 | 0 | 2 |
| **Total** | **14** | **0** | **0** | **14** |

---

## Phase 1: Setup

### T001 -- Create module directory structure and lib.rs skeleton

**Traces to:** FR-13, AC-4.1, AC-4.2
**Status:** TODO
**Files:** `crates/luminos-platform/src/lib.rs`, `crates/luminos-platform/src/traits/mod.rs`, `crates/luminos-platform/src/error.rs`, `crates/luminos-platform/src/mock/mod.rs`, `crates/luminos-platform/src/common/mod.rs`, `crates/luminos-platform/src/linux_x11/mod.rs`, `crates/luminos-platform/src/linux_wayland/mod.rs`, `crates/luminos-platform/src/macos/mod.rs`, `crates/luminos-platform/src/openbsd/mod.rs`, `crates/luminos-platform/src/windows/mod.rs`

**Steps:**
1. Replace the stub `lib.rs` with the module structure from DESIGN.md:
   - `pub mod traits;` and `pub mod error;` unconditionally
   - `pub mod mock;` gated by `#[cfg(any(test, feature = "test_utils"))]`
   - `pub(crate) mod common;` gated by `#[cfg(any(target_os = "linux", target_os = "openbsd"))]`
   - Platform backend modules (`linux_x11`, `linux_wayland`, `macos`, `openbsd`, `windows`) each gated by their respective `#[cfg(target_os = "...")]`
   - Note: Both `linux_x11` and `linux_wayland` are compiled on `target_os = "linux"` unconditionally (no feature gate)
2. Create `crates/luminos-platform/src/traits/mod.rs` with sub-module declarations (empty for now)
3. Create empty stub files for `error.rs`, `mock/mod.rs`, `common/mod.rs`, and all platform backend `mod.rs` files
4. Verify `cargo build -p luminos-platform` compiles (may need temporary empty content)

**Verification:** `cargo build -p luminos-platform` compiles. Module structure matches DESIGN.md component diagram.

**Completion Notes:**
>

---

### T002 -- Add required dependencies to luminos-platform Cargo.toml

**Traces to:** FR-1, FR-3, FR-4, FR-5, FR-6, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-platform/Cargo.toml`, workspace root `Cargo.toml`

**Steps:**
1. Ensure `luminos-platform/Cargo.toml` has the following in `[dependencies]`:
   - `thiserror = { workspace = true }`
   - `log = { workspace = true }`
2. Add `tokio` to workspace dependencies (needed for `tokio::sync::mpsc` in trait signatures):
   - In workspace root: `tokio = { version = "1", features = ["sync"] }`
   - In `luminos-platform`: `tokio = { workspace = true }`
3. Add `raw-window-handle` to workspace dependencies (needed for `WindowManager` trait):
   - In workspace root: `raw-window-handle = "0.6"`
   - In `luminos-platform`: `raw-window-handle = { workspace = true }`
4. Verify `cargo build -p luminos-platform` compiles after dependency additions

**Verification:** `cargo build -p luminos-platform` compiles. No duplicate dependency warnings.

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 1, verify:
- [ ] `cargo build -p luminos-platform` compiles with empty module stubs
- [ ] Module structure in `lib.rs` matches DESIGN.md (AC-4.1)
- [ ] All platform backend stubs exist as empty modules (AC-4.2)

---

## Phase 2: Core Implementation

### T003 -- Implement common types (types.rs)

**Traces to:** FR-7, FR-10, FR-12, FR-15, AC-2.1, AC-2.2, AC-2.3, AC-2.4, AC-5.1, AC-5.3, AC-5.4
**Status:** TODO
**Files:** `crates/luminos-platform/src/traits/types.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `types_screen_rect_fields_and_derives` -- Construct `ScreenRect`, verify `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash` via trait method calls
   - [ ] `types_screen_point_fields_and_derives` -- Construct `ScreenPoint`, verify all derives
   - [ ] `types_display_info_fields_and_derives` -- Construct `DisplayInfo`, verify `Debug`, `Clone`, `PartialEq`
   - [ ] `types_pixel_format_derives` -- Verify `PixelFormat::Bgra8` and `Rgba8` have `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`
   - [ ] `types_capture_frame_fields` -- Construct `CaptureFrame`, verify fields `data`, `width`, `height`, `stride`, `format`
   - [ ] `types_capture_frame_debug_omits_data` -- Format `CaptureFrame` with `{:?}`, assert output contains `"bytes"` placeholder, does NOT contain raw pixel data (RISK-017 mitigation)
   - [ ] `types_generate_test_capture_frame_correct_output` -- Call `generate_test_capture_frame(64, 48, [0, 0, 255, 255])`, verify `width == 64`, `height == 48`, `stride == 256`, `format == PixelFormat::Bgra8`, `data.len() == 12288`
   - [ ] `types_generate_test_display_info_correct_output` -- Call `generate_test_display_info("test-0", 1920, 1080, true)`, verify all fields
2. **Green** -- Implement:
   - [ ] Define `ScreenRect`, `ScreenPoint`, `DisplayInfo`, `PixelFormat`, `CaptureFrame` with all fields and derive attributes per DESIGN.md
   - [ ] Implement custom `Debug` for `CaptureFrame` that prints `data: [<{len} bytes>]`
   - [ ] Implement `generate_test_capture_frame` and `generate_test_display_info` in `#[cfg(test)] pub mod test_utils`
3. **Refactor** -- Clean up:
   - [ ] Ensure doc-comments on every public item match doc-02 Section 3.1 descriptions
   - [ ] Verify no clippy warnings

**Completion Notes:**
>

---

### T004 [P] -- Implement ScreenCapture trait and CaptureError (screen_capture.rs)

**Traces to:** FR-1, FR-9, FR-11, AC-1.2, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-platform/src/traits/screen_capture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `error_capture_error_display_not_found` -- Verify `CaptureError::DisplayNotFound("HDMI-1".into())` displays `"display not found: 'HDMI-1'"`
   - [ ] `error_capture_error_display_region_out_of_bounds` -- Verify `RegionOutOfBounds` variant display contains region and bounds
   - [ ] `error_capture_error_display_permission_denied` -- Verify `PermissionDenied` display message
   - [ ] `error_capture_error_display_backend_unavailable` -- Verify `BackendUnavailable` display contains reason
   - [ ] `error_capture_error_display_platform` -- Verify `Platform` variant display contains message
2. **Green** -- Implement:
   - [ ] Define `DisplayChangeEvent` enum with `Connected(DisplayInfo)`, `Disconnected(String)`, `Reconfigured(DisplayInfo)` variants
   - [ ] Define `CaptureError` enum with all 5 variants, `#[derive(Debug, thiserror::Error)]`
   - [ ] Define `ScreenCapture` trait bounded by `Send + Sync` with 3 methods: `list_displays`, `capture_frame`, `subscribe_display_changes`
3. **Refactor** -- Clean up:
   - [ ] Add doc-comments matching doc-02 Section 3.2 descriptions
   - [ ] Verify clippy clean

**Completion Notes:**
>

---

### T005 [P] -- Implement FocusTracker trait and FocusError (focus_tracker.rs)

**Traces to:** FR-2, FR-9, FR-11, AC-1.3, AC-3.2
**Status:** TODO
**Files:** `crates/luminos-platform/src/traits/focus_tracker.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `error_focus_error_display_api_unavailable` -- Verify `ApiUnavailable` variant display
   - [ ] `error_focus_error_display_permission_denied` -- Verify `PermissionDenied` display
   - [ ] `error_focus_error_display_query_failed` -- Verify `QueryFailed` display contains message
   - [ ] `error_focus_error_display_disconnected` -- Verify `Disconnected` display
   - [ ] `error_focus_error_display_platform` -- Verify `Platform` variant display
2. **Green** -- Implement:
   - [ ] Define `ElementType` enum with 6 variants (`TextInput`, `Control`, `Menu`, `ListItem`, `Link`, `Other(String)`)
   - [ ] Define `FocusChangedEvent` struct with fields: `element_id`, `bounds`, `element_type`, `label`, `pid`
   - [ ] Define `FocusError` enum with all 5 variants, `#[derive(Debug, thiserror::Error)]`
   - [ ] Define `FocusTracker` trait bounded by `Send + Sync` with 3 methods: `subscribe_focus_changes`, `get_focused_element`, `get_element_bounds`
3. **Refactor** -- Clean up:
   - [ ] Add doc-comments matching doc-02 Section 3.3 descriptions
   - [ ] Verify clippy clean

**Completion Notes:**
>

---

### T006 [P] -- Implement TtsEngine trait and TtsError (tts_engine.rs)

**Traces to:** FR-3, FR-8, FR-9, FR-11, AC-1.4, AC-2.6, AC-3.3
**Status:** TODO
**Files:** `crates/luminos-platform/src/traits/tts_engine.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `types_voice_fields_and_tts_backend_variants` -- Construct `Voice`, verify fields; verify `TtsBackend` has `Kokoro`, `Piper`, `Native` variants with `Debug`, `Clone`, `PartialEq`, `Eq`
   - [ ] `error_tts_error_display_voice_not_found` -- Verify `VoiceNotFound` display
   - [ ] `error_tts_error_display_model_load_failed` -- Verify `ModelLoadFailed` display
   - [ ] `error_tts_error_display_phonemizer_failed` -- Verify `PhonemizerFailed` display
   - [ ] `error_tts_error_display_inference_failed` -- Verify `InferenceFailed` display
   - [ ] `error_tts_error_display_audio_unavailable` -- Verify `AudioUnavailable` display
   - [ ] `error_tts_error_display_platform` -- Verify `Platform` variant display
2. **Green** -- Implement:
   - [ ] Define `Voice` struct with fields: `id`, `name`, `language`, `requires_download`, `engine`
   - [ ] Define `TtsBackend` enum with `Kokoro`, `Piper`, `Native` variants
   - [ ] Define `TtsError` enum with all 6 variants, `#[derive(Debug, thiserror::Error)]`
   - [ ] Define `TtsEngine` trait bounded by `Send + Sync` with 7 methods (using `Pin<Box<dyn Future>>` for `speak`)
3. **Refactor** -- Clean up:
   - [ ] Add doc-comments matching doc-02 Section 3.4 descriptions
   - [ ] Verify clippy clean

**Completion Notes:**
>

---

### T007 [P] -- Implement WindowManager trait and WindowError (window_manager.rs)

**Traces to:** FR-4, FR-9, FR-11, AC-1.5, AC-3.4
**Status:** TODO
**Files:** `crates/luminos-platform/src/traits/window_manager.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `error_window_error_display_creation_failed` -- Verify `CreationFailed` display
   - [ ] `error_window_error_display_property_failed` -- Verify `PropertyFailed` display includes property name and message
   - [ ] `error_window_error_display_display_not_found` -- Verify `DisplayNotFound` display
   - [ ] `error_window_error_display_dock_failed` -- Verify `DockFailed` display
   - [ ] `error_window_error_display_platform` -- Verify `Platform` display
2. **Green** -- Implement:
   - [ ] Define `DockEdge` enum with 4 variants (`Top`, `Bottom`, `Left`, `Right`)
   - [ ] Define `LensShape` enum with 2 variants (`Rectangle`, `Ellipse`)
   - [ ] Define `OverlayMode` enum with 3 variants (`FullScreen`, `Lens { width, height, shape }`, `Docked { edge, size_px }`)
   - [ ] Define `WindowError` enum with all 5 variants, `#[derive(Debug, thiserror::Error)]`
   - [ ] Define `WindowManager` trait bounded by `Send + Sync` with 7 methods (including `raw_window_handle`, `raw_display_handle`)
3. **Refactor** -- Clean up:
   - [ ] Add doc-comments matching doc-02 Section 3.5 descriptions
   - [ ] Verify clippy clean

**Completion Notes:**
>

---

### T008 [P] -- Implement InputMonitor trait and InputError (input_monitor.rs)

**Traces to:** FR-5, FR-9, FR-11, AC-4.4, AC-3.5
**Status:** TODO
**Files:** `crates/luminos-platform/src/traits/input_monitor.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `error_input_error_display_unavailable` -- Verify `Unavailable` variant display contains reason
   - [ ] `error_input_error_display_disconnected` -- Verify `Disconnected` display
   - [ ] `error_input_error_display_platform` -- Verify `Platform` display
2. **Green** -- Implement:
   - [ ] Define `Modifiers` struct with `shift`, `ctrl`, `alt`, `meta` bool fields, `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]`
   - [ ] Define `InputEvent` enum with 4 variants (`MouseMoved`, `MouseButton`, `Scroll`, `KeyEvent`) per DESIGN.md
   - [ ] Define `MouseButton` enum with 4 variants (`Left`, `Right`, `Middle`, `Other(u16)`)
   - [ ] Define `KeyCode` enum with full variant list per DESIGN.md
   - [ ] Define `InputError` enum with all 3 variants, `#[derive(Debug, thiserror::Error)]`
   - [ ] Define `InputMonitor` trait bounded by `Send + Sync` with 2 methods: `subscribe_input_events`, `get_mouse_position`
3. **Refactor** -- Clean up:
   - [ ] Add doc-comments matching doc-02 Section 3.6 descriptions
   - [ ] Verify clippy clean

**Completion Notes:**
>

---

### T009 [P] -- Implement AudioOutput trait and AudioError (audio_output.rs)

**Traces to:** FR-6, FR-8, FR-9, FR-11, AC-2.5, AC-4.5, AC-3.6
**Status:** TODO
**Files:** `crates/luminos-platform/src/traits/audio_output.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `types_audio_sample_fields_and_derives` -- Construct `AudioSample`, verify `Debug`, `Clone`, fields `data`, `sample_rate`, `channels`
   - [ ] `error_audio_error_display_no_device` -- Verify `NoDevice` display
   - [ ] `error_audio_error_display_device_failed` -- Verify `DeviceFailed` display
   - [ ] `error_audio_error_display_playback_interrupted` -- Verify `PlaybackInterrupted` display
   - [ ] `error_audio_error_display_unsupported_format` -- Verify `UnsupportedFormat` display
   - [ ] `error_audio_error_display_platform` -- Verify `Platform` display
2. **Green** -- Implement:
   - [ ] Define `AudioSample` struct with fields: `data: Vec<f32>`, `sample_rate: u32`, `channels: u16`, `#[derive(Debug, Clone)]`
   - [ ] Define `AudioError` enum with all 5 variants, `#[derive(Debug, thiserror::Error)]`
   - [ ] Define `AudioOutput` trait bounded by `Send + Sync` with 4 methods: `play_audio`, `stop_audio`, `set_volume`, `get_default_device_name`
3. **Refactor** -- Clean up:
   - [ ] Add doc-comments matching doc-02 Section 3.7 descriptions
   - [ ] Verify clippy clean

**Completion Notes:**
>

---

### T010 -- Wire up traits/mod.rs re-exports

**Traces to:** FR-1 through FR-11, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-platform/src/traits/mod.rs`

**Steps:**
1. Add sub-module declarations for all 6 trait files + types:
   ```
   pub mod types;
   pub mod screen_capture;
   pub mod focus_tracker;
   pub mod tts_engine;
   pub mod window_manager;
   pub mod input_monitor;
   pub mod audio_output;
   ```
2. Add `pub use` re-exports for all public items from each sub-module
3. Verify `cargo build -p luminos-platform` compiles with all modules wired

**Verification:** `cargo build -p luminos-platform` succeeds. All types accessible via `luminos_platform::traits::*`.

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 2, verify:
- [ ] `cargo build -p luminos-platform` compiles with zero warnings
- [ ] `cargo nextest run -p luminos-platform` -- all unit tests pass
- [ ] All 6 traits have correct method counts and signatures
- [ ] All 6 error enums have correct variants
- [ ] CaptureFrame custom Debug output omits data (RISK-017)

---

## Phase 3: Integration

### T011 -- Implement error.rs re-exports and PlatformBackends struct

**Traces to:** FR-14, AC-4.3
**Status:** TODO
**Files:** `crates/luminos-platform/src/error.rs`, `crates/luminos-platform/src/lib.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `platform_backends_struct_has_five_fields` -- Write a function that accepts `PlatformBackends` and accesses each field by name (`capture`, `focus_tracker`, `window_mgr`, `input_monitor`, `audio_output`) using `let _ = ...` assignments. If it compiles, the struct definition is correct. Use `#[allow(dead_code)]` on the test helper function. Note: full construction test requires mock implementations from Story 003; this task verifies the struct definition compiles.
2. **Green** -- Implement:
   - [ ] Populate `error.rs` with `pub use` re-exports for all 6 error types from `traits/` sub-modules
   - [ ] Add `PlatformBackends` struct to `lib.rs` with 5 trait object fields: `capture: Box<dyn ScreenCapture>`, `focus_tracker: Box<dyn FocusTracker>`, `window_mgr: Box<dyn WindowManager>`, `input_monitor: Box<dyn InputMonitor>`, `audio_output: Box<dyn AudioOutput>`
   - [ ] Add doc-comment for `PlatformBackends` explaining TtsEngine exclusion
   - [ ] Add necessary `use` imports in `lib.rs`
3. **Refactor** -- Clean up:
   - [ ] Verify `error.rs` allows `use luminos_platform::error::*` to import all error types
   - [ ] Verify clippy clean

**Completion Notes:**
>

---

### T012 -- Cross-crate compilation verification

**Traces to:** AC-1.1, NFR-1, NFR-4
**Status:** TODO
**Files:** None (verification only; may fix issues in any crate file)

**Steps:**
1. Run `cargo build --workspace` -- must compile with zero warnings
2. Run `cargo clippy -p luminos-platform -- -D warnings` -- must pass
3. Verify downstream crates can reference `luminos-platform` types:
   - Temporarily add a `use luminos_platform::traits::ScreenCapture;` to `luminos-gpu/src/lib.rs`, verify it compiles, then remove it
4. Run `cargo doc -p luminos-platform --no-deps` -- must produce documentation without warnings (NFR-3)
5. Fix any issues discovered

**Verification:** Full workspace builds. Downstream crates can import luminos-platform types. Docs build cleanly.

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 3, verify:
- [ ] `cargo build --workspace` -- zero warnings
- [ ] `cargo clippy -p luminos-platform -- -D warnings` -- passes
- [ ] `cargo doc -p luminos-platform --no-deps` -- no warnings
- [ ] `PlatformBackends` struct compiles with correct field types

---

## Phase 4: Polish & Acceptance

### T013 -- Verify doc-comments on all public items

**Traces to:** FR-16, AC-5.2, NFR-3
**Status:** TODO
**Files:** All files in `crates/luminos-platform/src/traits/`

**Steps:**
1. Review every public trait, method, struct, field, enum, and variant in the `traits/` module
2. Verify each has a `///` doc-comment matching doc-02 Sections 3.1-3.7 descriptions
3. Run `cargo doc -p luminos-platform --no-deps` and verify zero warnings
4. Fix any missing or incorrect doc-comments

**Verification:** `cargo doc` produces zero warnings. Visual inspection confirms doc-comments exist on all public items.

**Completion Notes:**
>

---

### T014 -- Final acceptance verification

**Traces to:** All ACs, All FRs, All NFRs
**Status:** TODO

**Verification Checklist:**

*US-1: All Six Traits Compile*
- [ ] AC-1.1: `cargo build -p luminos-platform` succeeds with zero errors and warnings
- [ ] AC-1.2: `ScreenCapture` has exactly 3 methods with correct signatures
- [ ] AC-1.3: `FocusTracker` has exactly 3 methods with correct signatures
- [ ] AC-1.4: `TtsEngine` has exactly 7 methods, `speak` returns `Pin<Box<dyn Future>>`
- [ ] AC-1.5: `WindowManager` has exactly 7 methods, including `raw_window_handle` and `raw_display_handle`

*US-2: Common Types*
- [ ] AC-2.1: `ScreenRect` fields and derives correct
- [ ] AC-2.2: `ScreenPoint` fields and derives correct
- [ ] AC-2.3: `DisplayInfo` fields and derives correct
- [ ] AC-2.4: `CaptureFrame` fields correct, does NOT derive standard `Debug`
- [ ] AC-2.5: `AudioSample` fields and derives correct
- [ ] AC-2.6: `Voice` fields correct, `TtsBackend` has 3 variants

*US-3: Error Enums*
- [ ] AC-3.1: `CaptureError` has exactly 5 variants with correct Display messages
- [ ] AC-3.2: `FocusError` has exactly 5 variants with correct Display messages
- [ ] AC-3.3: `TtsError` has exactly 6 variants with correct Display messages
- [ ] AC-3.4: `WindowError` has exactly 5 variants with correct Display messages
- [ ] AC-3.5: `InputError` has exactly 3 variants with correct Display messages
- [ ] AC-3.6: `AudioError` has exactly 5 variants with correct Display messages

*US-4: Module Structure*
- [ ] AC-4.1: `lib.rs` module declarations match doc-02 Section 5.2 (correct `#[cfg]` gates)
- [ ] AC-4.2: All platform backend stubs are empty modules that compile
- [ ] AC-4.3: `PlatformBackends` has 5 trait object fields (TtsEngine excluded)
- [ ] AC-4.4: `InputMonitor` has exactly 2 methods
- [ ] AC-4.5: `AudioOutput` has exactly 4 methods

*US-5: Docs, Privacy, Test Generators*
- [ ] AC-5.1: `CaptureFrame` `{:?}` omits pixel data, shows `[<N bytes>]` (RISK-017)
- [ ] AC-5.2: All public items have `///` doc-comments
- [ ] AC-5.3: `generate_test_capture_frame(64, 48, [0,0,255,255])` returns correct values
- [ ] AC-5.4: `generate_test_display_info("test-0", 1920, 1080, true)` returns correct values

*NFRs*
- [ ] NFR-1: All traits bounded by `Send + Sync`
- [ ] NFR-2: No `unwrap()` or `expect()` in production code
- [ ] NFR-3: `cargo doc -p luminos-platform --no-deps` -- zero warnings
- [ ] NFR-4: `cargo clippy -p luminos-platform -- -D warnings` -- passes
- [ ] NFR-5: All error enums derive `thiserror::Error` and `Debug`

*Full test run*
- [ ] `cargo nextest run -p luminos-platform` -- all tests pass

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
