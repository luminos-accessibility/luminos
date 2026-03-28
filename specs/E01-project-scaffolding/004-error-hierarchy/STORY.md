# Story E01/004: Error Hierarchy & Core Data Types

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** 002 (subsystem error types in `luminos-platform` must exist for `From` conversions)

---

## Problem Statement

The Luminos core engine must propagate errors from six different platform subsystems (capture, focus, TTS, window, input, audio) through a unified error type. Without a well-defined `LuminosError` hierarchy with automatic `From` conversions, every call site that crosses a subsystem boundary would require verbose manual error mapping, violating the project's error handling conventions (prefer `?` propagation, no `unwrap()`/`expect()` in production code).

Additionally, the core application data types -- `AppState`, `AppSettings`, `MagnificationMode`, `TrackingMode`, `ColorFilterType` -- must be defined in `luminos-core` so that the render thread, IPC layer, and configuration manager all share a single source of truth for runtime and persisted state. These types are referenced by virtually every subsequent epic.

This story defines the `LuminosError` enum with `#[from]` conversions for all six subsystem error types, the core application state and settings types, and unit tests verifying error propagation and serialization roundtrips.

## User Scenarios

### US-1: Error Propagation Across Subsystem Boundaries

As a **core engine developer**, I want `LuminosError` to automatically convert from any subsystem error via the `?` operator so that error handling in cross-subsystem code is concise and consistent.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given a function returning `Result<(), LuminosError>`, when a `CaptureError` is propagated with `?`, then the error is automatically converted to `LuminosError::Capture(CaptureError)` without explicit mapping.
- **AC-1.2:** Given a function returning `Result<(), LuminosError>`, when a `FocusError` is propagated with `?`, then the error is automatically converted to `LuminosError::Focus(FocusError)`.
- **AC-1.3:** Given a function returning `Result<(), LuminosError>`, when a `TtsError` is propagated with `?`, then the error is automatically converted to `LuminosError::Tts(TtsError)`.
- **AC-1.4:** Given a function returning `Result<(), LuminosError>`, when a `WindowError`, `InputError`, or `AudioError` is propagated with `?`, then each is automatically converted to its corresponding `LuminosError` variant.
- **AC-1.5:** Given any `LuminosError` variant, when `Display` is called (via `format!("{}", error)`), then the output is a human-readable message that identifies the subsystem and the underlying error detail.

### US-2: Configuration and Application State Types

As a **control panel developer** (E4), I want `AppSettings` to be serializable and deserializable with serde so that settings can be persisted to `config.toml` and hydrated back without data loss.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given an `AppSettings` instance with all fields set to non-default values, when serialized to TOML via `toml::to_string()` and deserialized back via `toml::from_str()`, then the deserialized value equals the original.
- **AC-2.2:** Given an `AppSettings` instance with all fields set to non-default values, when serialized to JSON via `serde_json::to_string()` and deserialized back via `serde_json::from_str()`, then the deserialized value equals the original.
- **AC-2.3:** Given `AppSettings::default()`, when each field is inspected, then the values match the compiled-in defaults specified in doc-01 Section 4.6 and doc-05 Section 3 (zoom level 2.0, mode FullScreen, tracking mode Cursor, color filter None, etc.).

### US-3: Runtime State Enums Have Correct Variants

As a **rendering pipeline developer** (E2), I want `MagnificationMode`, `TrackingMode`, and `ColorFilterType` enums to have exactly the variants specified in the tech strategy so that downstream code can exhaustively match against them.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given the `MagnificationMode` enum, when its variants are listed, then it contains exactly `FullScreen`, `Docked`, and `Lens`.
- **AC-3.2:** Given the `TrackingMode` enum, when its variants are listed, then it contains exactly `Cursor`, `Focus`, and `TextCaret`.
- **AC-3.3:** Given the `ColorFilterType` enum, when its variants are listed, then it contains exactly `None`, `Invert`, `SmartInvert`, `Grayscale`, `HighContrast`, and `Custom`.
- **AC-3.4:** Given any value of `MagnificationMode`, `TrackingMode`, or `ColorFilterType`, when serialized with serde, then the output uses PascalCase variant names (e.g., `"FullScreen"`, not `"full_screen"`).

### US-4: AppState Default Construction

As a **core engine developer**, I want `AppState` to have a sensible `Default` implementation so that the application can start with a valid initial state before any user configuration is loaded.

**Priority:** P1
**Acceptance Criteria:**

- **AC-4.1:** Given `AppState::default()`, when the embedded settings are inspected, then they match `AppSettings::default()`.
- **AC-4.2:** Given `AppState::default()`, when the runtime transient fields (viewport position, TTS status, etc.) are inspected, then they have reasonable initial values (viewport at origin, TTS idle, etc.).

## Functional Requirements

- **FR-1:** Define `LuminosError` enum in `crates/luminos-core/src/error.rs` with variants: `Capture(CaptureError)`, `Focus(FocusError)`, `Tts(TtsError)`, `Window(WindowError)`, `Input(InputError)`, `Audio(AudioError)`, `Config { message: String }`, `Internal { message: String }`. *(Traces to AC-1.1, AC-1.2, AC-1.3, AC-1.4)*
- **FR-2:** Derive `thiserror::Error` on `LuminosError` and use `#[from]` attributes for automatic `From` conversions from all six subsystem error types. *(Traces to AC-1.1, AC-1.2, AC-1.3, AC-1.4)*
- **FR-3:** Implement `Display` formatting on `LuminosError` via `thiserror` `#[error(...)]` attributes that produce human-readable messages identifying the subsystem. *(Traces to AC-1.5)*
- **FR-4:** Define `MagnificationMode`, `TrackingMode`, and `ColorFilterType` enums in `crates/luminos-core/src/state.rs` with variants matching doc-01/doc-02/doc-05 Section 3. All enums derive `serde::Serialize`, `serde::Deserialize` with `rename_all = "PascalCase"`. *(Traces to AC-3.1, AC-3.2, AC-3.3, AC-3.4)*
- **FR-5:** Define `AppSettings` struct in `crates/luminos-core/src/config/schema.rs` with fields matching the Zod schema in doc-05 Section 3.2, deriving `serde::Serialize`, `serde::Deserialize`, `Clone`, `Debug`, `PartialEq`. *(Traces to AC-2.1, AC-2.2)*
- **FR-6:** Implement `Default` for `AppSettings` with compiled-in defaults matching doc-01 Section 4.6 and doc-05 Section 3. *(Traces to AC-2.3)*
- **FR-7:** Define `AppState` struct in `crates/luminos-core/src/state.rs` containing `AppSettings` plus runtime transient fields (viewport position, TTS status, active display ID). *(Traces to AC-4.1, AC-4.2)*
- **FR-8:** Implement `Default` for `AppState` that uses `AppSettings::default()` for settings and reasonable initial values for transient state. *(Traces to AC-4.1, AC-4.2)*

## Non-Functional Requirements

- **NFR-1:** Zero `unwrap()` or `expect()` calls in any production code within `luminos-core/src/`. Exception: unit tests in `#[cfg(test)]` blocks.
- **NFR-2:** All `LuminosError` variants produce a `Display` output that is understandable to a developer reading logs -- the subsystem name and specific failure reason must be present.
- **NFR-3:** `AppSettings` serialization/deserialization roundtrip must be lossless for both TOML and JSON formats.
- **NFR-4:** All public types must have doc-comments explaining their purpose and usage context.

## Out of Scope

- **Platform backend error construction** -- the subsystem error types (`CaptureError`, `FocusError`, etc.) are defined in Story 002 within `luminos-platform`. This story only defines conversions *from* them.
- **`Arc<ArcSwap<AppState>>` wrapper** -- the concurrency wrapper is constructed at the application level in E2+. This story defines `AppState` as a plain struct.
- **Full AppSettings field richness** -- fields are defined with the structure and defaults from the tech strategy, but richer validation (e.g., zoom level clamping, keybinding conflict detection) comes in later epics (E4, E5).
- **Platform backends** -- no real platform-specific code. All error types are consumed as abstract `From` conversions.
- **Settings UI** -- deferred to Epic 4 (Control Panel Foundation).
- **Profile management** -- deferred to Epic 4+. `AppSettings` has no profile wrapper in E1.
- **Settings persistence (file I/O)** -- deferred to E4. This story verifies in-memory serialization roundtrips only.

## Open Questions

- [x] Should `AppSettings` include all fields from doc-05 Section 3.2 or just the Phase 0 subset? **Answer:** Include the full schema structure (magnification, color filter, cursor, speech, keybindings, top-level booleans) with all fields. Fields for Phase 1+ features (speech settings, keybinding config, etc.) have sensible defaults. This avoids schema migrations later and enables full serialization roundtrip testing from day one.
- [x] Should `AppState` include a `tts_status` field in E1 given TTS is Phase 2? **Answer:** Yes. Define the field (defaulting to `Idle`) to validate the struct layout. The TTS pipeline will populate it in E2/E3. Keeping the field from the start avoids structural changes when TTS is integrated.
- [x] Should `ColorFilterType` be defined here or in `luminos-gpu`? **Answer:** Define in `luminos-core` because it is part of `AppSettings` and `AppState`, which are core types. The GPU crate reads `ColorFilterType` from `AppState` via the `luminos-core` dependency.
