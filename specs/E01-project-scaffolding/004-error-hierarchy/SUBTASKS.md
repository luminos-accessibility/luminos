# Subtasks: Story E01/004 -- Error Hierarchy & Core Data Types

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
| 1. Setup | 1 | 0 | 0 | 1 |
| 2. Core Implementation | 6 | 0 | 0 | 6 |
| 3. Integration | 1 | 0 | 0 | 1 |
| 4. Polish & Acceptance | 2 | 0 | 0 | 2 |
| **Total** | **10** | **0** | **0** | **10** |

---

## Phase 1: Setup

### T001 -- Create luminos-core module structure

**Traces to:** FR-1, FR-4, FR-5, FR-7
**Status:** TODO
**Files:** `crates/luminos-core/src/lib.rs`, `crates/luminos-core/src/error.rs`, `crates/luminos-core/src/state.rs`, `crates/luminos-core/src/config/mod.rs`, `crates/luminos-core/src/config/schema.rs`, `crates/luminos-core/Cargo.toml`

**Steps (no TDD -- scaffolding only):**
- [ ] Create `crates/luminos-core/src/error.rs` (empty, with module-level doc-comment)
- [ ] Create `crates/luminos-core/src/state.rs` (empty, with module-level doc-comment)
- [ ] Create `crates/luminos-core/src/config/` directory
- [ ] Create `crates/luminos-core/src/config/mod.rs` with `pub mod schema;` declaration
- [ ] Create `crates/luminos-core/src/config/schema.rs` (empty, with module-level doc-comment)
- [ ] Update `crates/luminos-core/src/lib.rs` to declare modules: `pub mod error;`, `pub mod state;`, `pub mod config;`
- [ ] Verify `luminos-core/Cargo.toml` has required dependencies:
  - `luminos-platform = { workspace = true }` (for subsystem error types)
  - `thiserror = { workspace = true }` (for `LuminosError` derive)
  - `serde = { workspace = true, features = ["derive"] }` (for `AppSettings` serialization)
  - `serde_json = { workspace = true }` in `[dev-dependencies]` (for JSON roundtrip tests)
  - `toml = { workspace = true }` in `[dev-dependencies]` (for TOML roundtrip tests)
  - **Note:** Check if `serde_json` and `toml` are already declared in `[dependencies]` from Story 001's workspace setup. If so, they don't need separate `[dev-dependencies]` entries -- regular dependencies are already available in tests.
- [ ] Verify `cargo build -p luminos-core` compiles with the new empty modules

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 1, verify:
- [ ] `cargo build -p luminos-core` compiles with zero errors
- [ ] Module structure matches DESIGN.md component diagram

---

## Phase 2: Core Implementation

### T002 -- Implement LuminosError enum with From conversions

**Traces to:** FR-1, FR-2, FR-3, AC-1.1, AC-1.2, AC-1.3, AC-1.4, AC-1.5
**Status:** TODO
**Files:** `crates/luminos-core/src/error.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `luminos_error_from_capture_error` -- Create a `CaptureError::PermissionDenied`, propagate via `?` in a function returning `Result<(), LuminosError>`, assert result is `Err(LuminosError::Capture(_))` (AC-1.1)
   - [ ] `luminos_error_from_focus_error` -- Same pattern with `FocusError::PermissionDenied`, assert `Err(LuminosError::Focus(_))` (AC-1.2)
   - [ ] `luminos_error_from_tts_error` -- Same pattern with `TtsError::VoiceNotFound("test".into())`, assert `Err(LuminosError::Tts(_))` (AC-1.3)
   - [ ] `luminos_error_from_window_error` -- Same pattern with `WindowError::CreationFailed { message: "test".into() }`, assert `Err(LuminosError::Window(_))` (AC-1.4)
   - [ ] `luminos_error_from_input_error` -- Same pattern with `InputError::Unavailable { reason: "test".into() }`, assert `Err(LuminosError::Input(_))` (AC-1.4)
   - [ ] `luminos_error_from_audio_error` -- Same pattern with `AudioError::NoDevice`, assert `Err(LuminosError::Audio(_))` (AC-1.4)
   - [ ] `luminos_error_display_capture` -- Format `LuminosError::Capture(CaptureError::PermissionDenied)` with `"{}"`, assert output contains "screen capture" (AC-1.5)
   - [ ] `luminos_error_display_config` -- Format `LuminosError::Config { message: "bad value".into() }` with `"{}"`, assert output contains "configuration" and "bad value" (AC-1.5)
   - [ ] `luminos_error_display_internal` -- Format `LuminosError::Internal { message: "unexpected".into() }` with `"{}"`, assert output contains "internal error" and "unexpected" (AC-1.5)
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `LuminosError` enum with 8 variants: `Capture(#[from] CaptureError)`, `Focus(#[from] FocusError)`, `Tts(#[from] TtsError)`, `Window(#[from] WindowError)`, `Input(#[from] InputError)`, `Audio(#[from] AudioError)`, `Config { message: String }`, `Internal { message: String }`
   - [ ] Derive `Debug` and `thiserror::Error` with `#[error(...)]` attributes per DESIGN.md
   - [ ] Import subsystem error types from `luminos_platform`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to enum and each variant per DESIGN.md
   - [ ] Verify Display output format is consistent across all variants

**Completion Notes:**
>

---

### T003 [P] -- Implement configuration enums (MagnificationMode, TrackingMode, ColorFilterType, TtsStatus)

**Traces to:** FR-4, AC-3.1, AC-3.2, AC-3.3, AC-3.4
**Status:** TODO
**Files:** `crates/luminos-core/src/state.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `magnification_mode_serde_roundtrip` -- Serialize each variant (`FullScreen`, `Docked`, `Lens`) to JSON, assert PascalCase strings (e.g., `"FullScreen"`), deserialize back, assert equality (AC-3.1, AC-3.4)
   - [ ] `tracking_mode_serde_roundtrip` -- Same pattern for `Cursor`, `Focus`, `TextCaret` (AC-3.2, AC-3.4)
   - [ ] `color_filter_type_serde_roundtrip` -- Same pattern for all six variants: `None`, `Invert`, `SmartInvert`, `Grayscale`, `HighContrast`, `Custom` (AC-3.3, AC-3.4)
   - [ ] `tts_status_serde_roundtrip` -- Same pattern for `Idle`, `Loading`, `Speaking`, `Draining`, `Error`
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `MagnificationMode` enum: `FullScreen`, `Docked`, `Lens` -- derive `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize`
   - [ ] Define `TrackingMode` enum: `Cursor`, `Focus`, `TextCaret` -- same derives
   - [ ] Define `ColorFilterType` enum: `None`, `Invert`, `SmartInvert`, `Grayscale`, `HighContrast`, `Custom` -- same derives
   - [ ] Define `TtsStatus` enum: `Idle`, `Loading`, `Speaking`, `Draining`, `Error` -- same derives
   - [ ] Add `use serde::{Deserialize, Serialize};` import
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to each enum and variant per DESIGN.md
   - [ ] Verify all enums use default serde representation (PascalCase variant names) -- no `rename_all` attribute needed since PascalCase is the serde default for enum variants

**Completion Notes:**
>

---

### T004a [P] -- Define supporting enums and config sub-structs with Default impls

**Traces to:** FR-5, FR-6, AC-2.3
**Status:** TODO
**Files:** `crates/luminos-core/src/config/schema.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `magnification_settings_default_zoom_level` -- Assert `MagnificationSettings::default().zoom_level == 2.0`
   - [ ] `magnification_settings_default_mode` -- Assert `MagnificationSettings::default().mode == MagnificationMode::FullScreen`
   - [ ] `magnification_settings_default_tracking` -- Assert `MagnificationSettings::default().tracking_mode == TrackingMode::Cursor`
   - [ ] `color_filter_config_default_filter_none` -- Assert `ColorFilterConfig::default().filter_type == ColorFilterType::None`
   - [ ] `speech_settings_default_disabled` -- Assert `SpeechSettings::default().enabled == false`
2. **Green** -- Implement minimum code to pass:
   - [ ] Define supporting enums: `DockEdge` (`Top`, `Bottom`, `Left`, `Right`), `LensShape` (`Rectangle`, `Ellipse`), `PresentMode` (`Quality`, `LowLatency`, `Performance`), `GpuPreference` (`LowPower`, `HighPerformance`), `InterpolationMode` (`Bilinear`, `Bicubic`), `ModelVariant` (`Q4`, `Q8`, `Fp16`, `Fp32`), `HotkeyAction` (9 variants), `ModifierKey` (`Ctrl`, `Shift`, `Alt`, `Super`, `Meta`), `KeyBinding` struct -- all with appropriate serde derives
   - [ ] Define `MagnificationSettings` struct with all fields per DESIGN.md, derive `Debug, Clone, PartialEq, Serialize, Deserialize`, implement `Default`
   - [ ] Define `ColorFilterConfig` struct, derive, implement `Default`
   - [ ] Define `CursorConfig` struct, derive, implement `Default`
   - [ ] Define `SpeechSettings` struct, derive, implement `Default`
   - [ ] Import `HashMap` for keybindings, import enums from `crate::state`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to all supporting enums, structs, and fields per DESIGN.md
   - [ ] Verify default values match doc-01 Section 4.6 and doc-05 Section 3

**Note on DockEdge/LensShape duplication:** `DockEdge` and `LensShape` are also defined in `luminos-platform` (Story 002) as part of `WindowManager` associated types. The definitions here in `luminos-core` are independent for the settings schema. A future reconciliation may unify them, but for E1 they remain separate. Record this in completion notes for the HIGH_LEVEL_PLAN.md Shared Context.

**Completion Notes:**
>

---

### T004b [P] -- Define AppSettings root struct with Default impl and serde roundtrip tests

**Traces to:** FR-5, FR-6, AC-2.1, AC-2.2, AC-2.3
**Status:** TODO
**Files:** `crates/luminos-core/src/config/schema.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `app_settings_default_zoom_level` -- Assert `AppSettings::default().magnification.zoom_level == 2.0` (AC-2.3)
   - [ ] `app_settings_default_mode_fullscreen` -- Assert `AppSettings::default().magnification.mode == MagnificationMode::FullScreen` (AC-2.3)
   - [ ] `app_settings_default_tracking_cursor` -- Assert `AppSettings::default().magnification.tracking_mode == TrackingMode::Cursor` (AC-2.3)
   - [ ] `app_settings_default_color_filter_none` -- Assert `AppSettings::default().color_filter.filter_type == ColorFilterType::None` (AC-2.3)
   - [ ] `app_settings_default_speech_disabled` -- Assert `AppSettings::default().speech.enabled == false` (AC-2.3)
   - [ ] `app_settings_toml_roundtrip` -- Serialize `AppSettings::default()` to TOML string, deserialize back, assert equality (AC-2.1)
   - [ ] `app_settings_json_roundtrip` -- Serialize `AppSettings::default()` to JSON string, deserialize back, assert equality (AC-2.2)
   - [ ] `app_settings_nondefault_toml_roundtrip` -- Construct `AppSettings` with ALL fields set to non-default values, serialize to TOML, deserialize back, assert equality (AC-2.1)
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `AppSettings` struct with all fields per DESIGN.md (references sub-structs from T004a), derive `Debug, Clone, PartialEq, Serialize, Deserialize`
   - [ ] Implement `Default` for `AppSettings` using sub-struct defaults and top-level defaults (`start_on_login: false`, `minimize_to_tray: true`, `show_panel_on_start: true`, `keybindings: HashMap::new()`)
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to `AppSettings` struct and all fields per DESIGN.md
   - [ ] Verify default values match doc-01 Section 4.6 and doc-05 Section 3 (zoom 2.0, FullScreen, Cursor, etc.)

**Completion Notes:**
>

---

### T005 -- Implement AppState struct with Default

**Traces to:** FR-7, FR-8, AC-4.1, AC-4.2
**Status:** TODO
**Files:** `crates/luminos-core/src/state.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `app_state_default_settings_match` -- Assert `AppState::default().settings == AppSettings::default()` (AC-4.1)
   - [ ] `app_state_default_viewport_at_origin` -- Assert `AppState::default().viewport == ScreenRect { x: 0, y: 0, width: 0, height: 0 }` (AC-4.2)
   - [ ] `app_state_default_tts_idle` -- Assert `AppState::default().tts_status == TtsStatus::Idle` (AC-4.2)
   - [ ] `app_state_default_not_active` -- Assert `AppState::default().is_active == false` (AC-4.2)
   - [ ] `app_state_default_no_active_display` -- Assert `AppState::default().active_display_id.is_none()`
2. **Green** -- Implement minimum code to pass:
   - [ ] Define `AppState` struct with fields: `settings: AppSettings`, `viewport: ScreenRect`, `tts_status: TtsStatus`, `active_display_id: Option<String>`, `is_active: bool` -- derive `Debug, Clone, PartialEq`
   - [ ] Implement `Default` for `AppState`: settings = `AppSettings::default()`, viewport at origin (0,0,0,0), tts_status = `Idle`, active_display_id = `None`, is_active = `false`
   - [ ] Import `ScreenRect` from `luminos_platform`, `AppSettings` from `crate::config::schema`, `TtsStatus` from same module
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to struct and all fields per DESIGN.md
   - [ ] Note in doc-comment that `AppState` is wrapped in `Arc<ArcSwap<AppState>>` at the application level (not in this story)

**Completion Notes:**
>

---

### T006 -- Wire lib.rs re-exports

**Traces to:** FR-1, FR-4, FR-5, FR-7
**Status:** TODO
**Files:** `crates/luminos-core/src/lib.rs`

**Steps (no TDD -- wiring only; compile-only import verification has no meaningful Red phase):**
- [ ] Add `pub use error::LuminosError;` to `lib.rs`
- [ ] Add `pub use state::{AppState, ColorFilterType, MagnificationMode, TrackingMode, TtsStatus};` to `lib.rs`
- [ ] Add `pub use config::schema::AppSettings;` to `lib.rs`
- [ ] Add module-level doc-comment to `lib.rs` per DESIGN.md
- [ ] Verify `cargo build -p luminos-core` compiles with the re-exports
- [ ] Verify all re-exported types are accessible from downstream crates (confirmed in T007 workspace build)

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [ ] `cargo nextest run -p luminos-core` passes all tests
- [ ] All `From` conversions compile and work (6 subsystem errors → `LuminosError`)
- [ ] `AppSettings` default values match specification
- [ ] `AppState::default()` produces sensible initial state

---

## Phase 3: Integration

### T007 -- Verify cross-crate dependency and build

**Traces to:** FR-1, FR-2, AC-1.1, AC-4.1
**Status:** TODO
**Files:** `crates/luminos-core/Cargo.toml`

**Steps:**
- [ ] Run `cargo build --workspace` and verify zero warnings
- [ ] Run `cargo nextest run --workspace` and verify all tests pass (both `luminos-platform` and `luminos-core`)
- [ ] Verify `luminos-core` depends on `luminos-platform` correctly (check `Cargo.toml`)
- [ ] Run `cargo tree -p luminos-core` and verify dependency graph looks correct (luminos-core → luminos-platform, thiserror, serde)
- [ ] If any circular dependencies or version conflicts are detected, fix them

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 3, verify:
- [ ] `cargo build --workspace` compiles with zero warnings
- [ ] `cargo nextest run --workspace` passes all tests

---

## Phase 4: Polish & Acceptance

### T008 -- Clippy, formatting, and doc-comment audit

**Traces to:** NFR-1, NFR-2, NFR-4
**Status:** TODO
**Files:** All `crates/luminos-core/src/**/*.rs` files

**Steps:**
- [ ] Run `cargo clippy -p luminos-core -- -D warnings` and fix any warnings
- [ ] Run `cargo clippy -p luminos-core -- -W clippy::unwrap_used -W clippy::expect_used` and verify zero warnings in production code (only `#[cfg(test)]` blocks may use `unwrap()`)
- [ ] Run `cargo fmt --all -- --check` and fix any formatting issues
- [ ] Run `cargo doc -p luminos-core --no-deps` and verify zero warnings
- [ ] Verify every public struct, enum, method, and field has a `///` doc-comment
- [ ] Verify `LuminosError` Display output for each variant includes subsystem name and detail (NFR-2)

**Completion Notes:**
>

---

### T009 -- Full acceptance test verification

**Traces to:** All ACs (AC-1.1 through AC-4.2)
**Status:** TODO
**Files:** None (verification only)

**Verification Checklist:**
- [ ] AC-1.1: `luminos_error_from_capture_error` test passes -- `CaptureError` propagates to `LuminosError::Capture` via `?` (T002)
- [ ] AC-1.2: `luminos_error_from_focus_error` test passes (T002)
- [ ] AC-1.3: `luminos_error_from_tts_error` test passes (T002)
- [ ] AC-1.4: `luminos_error_from_window_error`, `_input_error`, `_audio_error` tests pass (T002)
- [ ] AC-1.5: `luminos_error_display_capture`, `_config`, `_internal` tests pass -- Display output includes subsystem name (T002)
- [ ] AC-2.1: `app_settings_toml_roundtrip` and `app_settings_nondefault_toml_roundtrip` tests pass (T004b)
- [ ] AC-2.2: `app_settings_json_roundtrip` test passes (T004b)
- [ ] AC-2.3: `app_settings_default_zoom_level`, `_mode_fullscreen`, `_tracking_cursor`, `_color_filter_none`, `_speech_disabled` tests pass (T004a, T004b)
- [ ] AC-3.1: `magnification_mode_serde_roundtrip` test passes -- exactly 3 variants (T003)
- [ ] AC-3.2: `tracking_mode_serde_roundtrip` test passes -- exactly 3 variants (T003)
- [ ] AC-3.3: `color_filter_type_serde_roundtrip` test passes -- exactly 6 variants (T003)
- [ ] AC-3.4: Serde roundtrip tests verify PascalCase output (T003)
- [ ] AC-4.1: `app_state_default_settings_match` test passes (T005)
- [ ] AC-4.2: `app_state_default_viewport_at_origin`, `_tts_idle`, `_not_active` tests pass (T005)
- [ ] NFR-1: Zero `unwrap()`/`expect()` in production code
- [ ] NFR-2: All `LuminosError` Display outputs are human-readable with subsystem identification
- [ ] NFR-3: TOML and JSON roundtrips are lossless
- [ ] NFR-4: All public types have doc-comments
- [ ] Run final: `cargo nextest run -p luminos-core` exits 0

**Note on DockEdge/LensShape duplication:** Record in HIGH_LEVEL_PLAN.md Shared Context > Discovered Constraints that `DockEdge` and `LensShape` are defined independently in both `luminos-platform` (Story 002, as part of `WindowManager` types) and `luminos-core` (Story 004, as part of `AppSettings` schema). Future reconciliation may unify them via a re-export or shared definition in `luminos-platform`.

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
