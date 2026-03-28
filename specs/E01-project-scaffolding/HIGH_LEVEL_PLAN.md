# Epic E01: Project Scaffolding, Platform Traits & CI/CD

**Status:** NOT STARTED
**Roadmap Ref:** [tech-strategy/09-implementation-roadmap.md Section 4.1](../tech-strategy/09-implementation-roadmap.md#41-epic-1----project-scaffolding-platform-traits--cicd)
**Phase:** Phase 0: Foundation (Months 1-3)
**Started:** ---
**Completed:** ---
**Hard Dependencies:** None (first epic)
**Soft Dependencies:** None
**Primary Docs:** [01 -- System Architecture](../tech-strategy/01-system-architecture.md) Section 7, [02 -- Platform Abstraction](../tech-strategy/02-platform-abstraction.md) Sections 2-4 and 7, [07 -- Testing Strategy](../tech-strategy/07-testing-strategy.md) Sections 4.1-4.4, [08 -- Build and Distribution](../tech-strategy/08-build-and-distribution.md) Sections 2 and 4

---

## Overview

Bootstrap the entire development environment: Cargo workspace with five crate stubs (`luminos-core`, `luminos-platform`, `luminos-gpu`, `luminos-tts`, `luminos-app`), all six platform abstraction trait definitions with mock implementations, the shared error type hierarchy, core data types, and a fully operational CI/CD pipeline. This epic produces no user-facing functionality but is the prerequisite for every subsequent epic. Its deliverable to the team is the ability to begin parallel feature work on E2 (X11 Capture + GPU), E3 (Input Tracking), and E4 (Control Panel).

This is the only "infrastructure-only" epic in the roadmap. Every subsequent epic delivers user-perceivable value. The architectural decisions embedded here -- workspace structure, trait signatures, error hierarchy, build profiles, and CI quality gates -- form the foundation that all 19 remaining epics build upon.

## Success Criteria

Copied verbatim from [doc-09 Section 4.1](../tech-strategy/09-implementation-roadmap.md#41-epic-1----project-scaffolding-platform-traits--cicd):

- [ ] `cargo build --workspace` compiles with zero warnings
- [ ] `cargo nextest run --workspace` passes all mock-based unit tests
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo deny check` passes (all dependencies GPLv3-compatible)
- [ ] CI pipeline runs end-to-end on a PR and reports status
- [ ] A new contributor can clone the repo, run `cargo build`, and get a clean result

---

## Story Breakdown

### Progress Summary

| # | Story | Status | Depends On | Est. Effort | Notes |
|---|-------|--------|------------|-------------|-------|
| 001 | Cargo Workspace & Build Profiles | NOT STARTED | --- | M (6-10 subtasks) | Foundation; unblocks all other stories |
| 002 | Platform Trait Definitions & Common Types | NOT STARTED | 001 | L (11-15 subtasks) | All 6 traits + associated types from doc-02 |
| 003 | Mock Implementations & Test Utilities | NOT STARTED | 002 | L (11-15 subtasks) | Can run parallel with 004 after 002 |
| 004 | Error Hierarchy & Core Data Types | NOT STARTED | 002 | M (6-10 subtasks) | Can run parallel with 003 after 002 |
| 005 | CI/CD Pipeline | NOT STARTED | 001 | M (6-10 subtasks) | Can run parallel with 002, 003, 004 |

**Total Stories:** 5 | **Done:** 0 | **In Progress:** 0 | **Blocked:** 0

**Dependency graph:**

```
001 Workspace ──┬──> 002 Traits ──┬──> 003 Mocks
               │                 │
               │                 └──> 004 Errors & Core Types
               │
               └──> 005 CI/CD
```

Stories 003 and 004 can execute in parallel once 002 is complete. Story 005 can execute in parallel with 002/003/004 once 001 is complete.

### Deliverable Traceability

Every roadmap deliverable (D1-D6) and success criterion (SC1-SC6) maps to at least one story:

| Deliverable | Description | Story |
|-------------|-------------|-------|
| D1 | Compiling Cargo workspace (`cargo build` passes on all five crates) | 001, 002, 003, 004 |
| D2 | All six trait definitions with doc-comments matching doc-02 | 002 |
| D3 | Mock implementations pass unit tests for every trait method | 003 |
| D4 | `LuminosError` hierarchy + platform errors + `From` conversions | 004 |
| D5 | CI pipeline: build, test, lint, license check, vulnerability scan | 005 |
| D6 | Build profiles produce correct optimization levels | 001 |

| Success Criterion | Story |
|-------------------|-------|
| SC1: `cargo build --workspace` zero warnings | 001, 002, 003, 004 |
| SC2: `cargo nextest run --workspace` passes | 003, 004 |
| SC3: `cargo clippy --workspace -- -D warnings` passes | All (enforced in 005) |
| SC4: `cargo deny check` passes | 001 (deny.toml), 005 (CI enforcement) |
| SC5: CI pipeline runs end-to-end on PR | 005 |
| SC6: New contributor clean build | 001, 005 |

### Story Descriptions

#### 001 -- Cargo Workspace & Build Profiles

**Scope:** Create the Cargo workspace root manifest, five crate stubs that compile, workspace-level dependency declarations, build profiles, Rust toolchain configuration, and project-level configuration files. This story produces the skeleton that all subsequent stories fill in.

**Key Deliverables:**
- `Cargo.toml` workspace root with `[workspace]` members, `[workspace.package]` metadata, and `[workspace.dependencies]` as specified in [doc-08 Section 2](../tech-strategy/08-build-and-distribution.md#2-cargo-workspace-configuration)
- Five crate stubs: `crates/luminos-core/`, `crates/luminos-platform/`, `crates/luminos-gpu/`, `crates/luminos-tts/`, `crates/luminos-app/` -- each with `Cargo.toml` inheriting workspace metadata and a minimal `src/lib.rs` (or `src/main.rs` for `luminos-app`)
- Crate dependency graph matching [doc-01 Section 7.2](../tech-strategy/01-system-architecture.md#72-crate-dependency-graph): `luminos-platform` has no internal deps; `luminos-core` depends on `luminos-platform` and `luminos-tts`; `luminos-gpu` depends on `luminos-platform`; `luminos-tts` depends on `luminos-platform`; `luminos-app` depends on all four
- Build profiles: `dev`, `release`, `dist` as defined in [doc-08 Section 4](../tech-strategy/08-build-and-distribution.md#4-build-profiles)
- `rust-toolchain.toml` (Rust 2024 edition, rustc 1.85+)
- `.clippy.toml` with `cognitive-complexity-threshold = 25`, `too-many-arguments-threshold = 7`, `type-complexity-threshold = 250` per [doc-07 Section 4.2](../tech-strategy/07-testing-strategy.md#42-stage-1-lint--format)
- `rustfmt.toml` for consistent formatting
- `deny.toml` with GPLv3-compatible license allowlist per [doc-07 Section 4.2](../tech-strategy/07-testing-strategy.md#42-stage-1-lint--format)
- `.config/nextest.toml` with default and CI profiles per [doc-07 Section 3.1](../tech-strategy/07-testing-strategy.md#31-rust-test-tools)
- `LICENSE` file (GPL-3.0-only full text)
- `CHANGELOG.md` initialized (empty, with Keep a Changelog format header)
- Cargo feature definitions per crate matching [doc-08 Section 3.2](../tech-strategy/08-build-and-distribution.md#32-feature-definitions-per-crate)

**Estimated Effort:** M (6-10 subtasks)

**Notes:** This story has no dependencies and must complete before any other story can begin. The workspace must produce `cargo build --workspace` with zero warnings and `cargo deny check` must pass. External dependencies (wgpu, winit, tauri, etc.) are declared in workspace dependencies but only used by crate stubs as empty imports -- actual usage comes in later stories and epics.

---

#### 002 -- Platform Trait Definitions & Common Types

**Scope:** Define all six platform abstraction traits and their associated types in `luminos-platform/src/traits.rs` (or `traits/` module directory), with full doc-comments matching the canonical definitions in [doc-02 Sections 3.1-3.7](../tech-strategy/02-platform-abstraction.md#3-trait-definitions). This story also defines the common types shared across traits (`ScreenRect`, `ScreenPoint`, `DisplayInfo`, `PixelFormat`, `CaptureFrame`, etc.) and the per-subsystem error enums (`CaptureError`, `FocusError`, `TtsError`, `WindowError`, `InputError`, `AudioError`).

**Key Deliverables:**
- `crates/luminos-platform/src/traits.rs` (or `traits/` module dir) containing:
  - Common types: `ScreenRect`, `ScreenPoint`, `DisplayInfo`, `PixelFormat`, `CaptureFrame` (doc-02 Section 3.1)
  - `ScreenCapture` trait + `DisplayChangeEvent` enum + `CaptureError` enum (doc-02 Section 3.2)
  - `FocusTracker` trait + `FocusChangedEvent`, `ElementType` + `FocusError` enum (doc-02 Section 3.3)
  - `TtsEngine` trait + `Voice`, `TtsBackend` + `TtsError` enum (doc-02 Section 3.4)
  - `WindowManager` trait + `OverlayMode`, `DockEdge`, `LensShape` + `WindowError` enum (doc-02 Section 3.5)
  - `InputMonitor` trait + `InputEvent`, `MouseButton`, `KeyCode`, `Modifiers` + `InputError` enum (doc-02 Section 3.6)
  - `AudioOutput` trait + `AudioSample` + `AudioError` enum (doc-02 Section 3.7)
- `crates/luminos-platform/src/error.rs` containing platform-specific error re-exports
- Doc-comments on every trait, method, type, and variant matching doc-02 descriptions
- `generate_test_*` functions co-located with types in `#[cfg(test)]` blocks (e.g., `generate_test_capture_frame`, `generate_test_display_info`) per doc-02 Section 3.2
- `lib.rs` module structure matching [doc-02 Section 5.2](../tech-strategy/02-platform-abstraction.md#52-cfg-patterns) with `#[cfg]`-gated backend module declarations (empty modules for now -- backends are implemented in later epics)

**Estimated Effort:** L (11-15 subtasks)

**Notes:** Depends on Story 001 (workspace must compile). The trait definitions are the central coordination point for the entire project -- they must match doc-02 exactly. The `TtsEngine::speak` method uses `Pin<Box<dyn Future>>` return type for object safety, not RPITIT. All error types derive `thiserror::Error`. The `PlatformBackends` bundle struct should also be defined here (doc-02 Section 5.3). Platform backend modules are declared with `#[cfg]` gates but left empty (stub `mod` declarations only).

---

#### 003 -- Mock Implementations & Test Utilities

**Scope:** Implement mock versions of all six traits in `crates/luminos-platform/src/mock/`, gated behind `#[cfg(any(test, feature = "test_utils"))]`. Each mock is parameterizable with builder methods for error injection. Write unit tests that verify every mock method works correctly for both success and error paths.

**Key Deliverables:**
- `crates/luminos-platform/src/mock/` directory with:
  - `mod.rs` -- re-exports all mock structs
  - `capture.rs` -- `MockScreenCapture` with `generate_test_mock_screen_capture()` constructor and `with_error()` builder (doc-02 Section 7.1)
  - `focus.rs` -- `MockFocusTracker` with `generate_test_mock_focus_tracker()` constructor
  - `tts.rs` -- `MockTtsEngine` with `generate_test_mock_tts_engine()` constructor
  - `window.rs` -- `MockWindowManager` with `generate_test_mock_window_manager()` constructor
  - `input.rs` -- `MockInputMonitor` with `generate_test_mock_input_monitor()` constructor
  - `audio.rs` -- `MockAudioOutput` with `generate_test_mock_audio_output()` constructor
- `test_utils` feature flag wired in `luminos-platform/Cargo.toml` to export mocks to downstream crates
- Unit tests for every mock method (success path + error injection):
  - Tests use hierarchical naming: `mock_screen_capture_*`, `mock_focus_tracker_*`, etc.
  - Happy-path tests verify correct return values
  - Error-path tests verify `with_error()` factory produces expected error variants
  - All tests pass via `cargo nextest run -p luminos-platform`

**Estimated Effort:** L (11-15 subtasks)

**Notes:** Depends on Story 002 (trait definitions must exist to implement them). Can run in parallel with Story 004 once 002 is complete. Error factories use closures (`Box<dyn Fn() -> XxxError + Send + Sync>`) because error types are not `Clone` (they may contain `Box<dyn Error>` in Platform variants). The mock pattern is documented in doc-02 Section 7.1.

---

#### 004 -- Error Hierarchy & Core Data Types

**Scope:** Define the top-level `LuminosError` enum in `luminos-core/src/error.rs` with `From` trait conversions from all six subsystem error types. Define core application data types (`AppState`, `AppSettings`, `MagnificationMode`, `TrackingMode`, `ColorFilterType`) in `luminos-core` with field definitions matching the tech strategy documents. Note: `ColorFilterType` is an addition beyond the doc-09 E1 scope (which lists `AppState`, `AppSettings`, `MagnificationMode`, `TrackingMode`) but is included here because it is part of the `AppSettings` schema (doc-05 Section 3) and is needed for settings serialization roundtrip tests.

**Key Deliverables:**
- `crates/luminos-core/src/error.rs`:
  - `LuminosError` enum with variants: `Capture(CaptureError)`, `Focus(FocusError)`, `Tts(TtsError)`, `Window(WindowError)`, `Input(InputError)`, `Audio(AudioError)`, `Config { message: String }`, `Internal { message: String }` (doc-02 Section 4.1)
  - All `#[from]` derives via `thiserror` for automatic `From` conversions
  - Unit tests verifying `?` propagation from each subsystem error to `LuminosError`
- `crates/luminos-core/src/state.rs`:
  - `AppState` struct -- runtime shared state read by render thread via `ArcSwap` (doc-01 Section 4.6)
  - `MagnificationMode` enum (`FullScreen`, `Docked`, `Lens`) matching doc-01/doc-02
  - `TrackingMode` enum (`Cursor`, `Focus`, `TextCaret`) matching doc-05 Section 3
  - `ColorFilterType` enum (`None`, `Invert`, `SmartInvert`, `Grayscale`, `HighContrast`, `Custom`) matching doc-05 Section 3
- `crates/luminos-core/src/config/schema.rs`:
  - `AppSettings` struct -- the persisted settings schema (doc-05 Section 3)
  - Default implementation matching compiled-in defaults from doc-01 Section 4.6
- Unit tests:
  - `LuminosError` `Display` formatting for every variant
  - `From` conversion for every subsystem error type
  - `AppState` default construction
  - `AppSettings` serialization/deserialization roundtrip via serde

**Estimated Effort:** M (6-10 subtasks)

**Notes:** Depends on Story 002 (subsystem error types in `luminos-platform` must exist for `From` conversions to compile). Can run in parallel with Story 003 once 002 is complete. The `AppState` struct wraps settings + runtime transient state (viewport position, TTS status, etc.). The `AppSettings` struct is the subset that gets persisted to `config.toml`. Fields should be defined with sensible defaults but full richness comes in later epics -- define the structure now, populate it incrementally.

---

#### 005 -- CI/CD Pipeline

**Scope:** Create a GitHub Actions workflow that runs on every push and PR, implementing Stages 1-4 of the CI pipeline from [doc-07 Section 4](../tech-strategy/07-testing-strategy.md#4-cicd-pipeline-architecture). The pipeline enforces code quality via linting, formatting, unit testing, license compliance, and vulnerability scanning.

**Key Deliverables:**
- `.github/workflows/ci.yml` implementing:
  - **Stage 1 -- Lint & Format** (doc-07 Section 4.2):
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions`
    - `cargo deny check licenses advisories`
  - **Stage 2 -- Unit Tests** (doc-07 Section 4.3):
    - `cargo nextest run --profile ci --workspace --exclude luminos-app`
  - **Stage 3 -- Component Tests** (doc-07 Section 4.4):
    - Placeholder for shader tests (actual shader tests added in E2)
  - **Stage 4 -- Integration Tests** (doc-07 Section 4.5):
    - Placeholder structure (actual integration tests added in E2+)
  - `cargo audit` for vulnerability scanning
- Rust toolchain caching (rustup + cargo cache via `actions/cache`)
- Job dependency chain: lint -> test -> (future stages)
- Branch protection rule documentation (require CI pass on PR)
- Optional: pre-commit hook script (`.githooks/pre-commit`) running `cargo fmt --check` and `cargo clippy`

**Estimated Effort:** M (6-10 subtasks)

**Notes:** Depends on Story 001 (workspace must exist for CI to run against). Can run in parallel with Stories 002, 003, and 004 once 001 is complete. Stage 3 (shader tests) and Stage 4 (integration tests) are placeholder jobs in E1 -- they become real in E2 when GPU rendering and platform backends are implemented. The CI pipeline should use `ubuntu-latest` runners. TypeScript CI stages (ESLint, Vitest) are deferred to E4 (Control Panel Foundation) when the frontend scaffolding exists.

---

## Shared Context

This section contains cross-cutting knowledge that applies to all stories in this epic. Agents working on any story should read this section. Update it as stories are completed and new knowledge emerges.

### Architecture Decisions

These decisions are drawn from the tech strategy and apply across all E1 stories:

- **Workspace structure with 5 crates:** `luminos-core`, `luminos-platform`, `luminos-gpu`, `luminos-tts`, `luminos-app`. Dependency direction flows from `luminos-app` (binary) down to `luminos-platform` (foundation). No circular dependencies. See [doc-01 Section 7.1](../tech-strategy/01-system-architecture.md#71-cargo-workspace-structure).

- **Rust 2024 edition (rustc 1.85+):** Uses resolver version 3 (default for edition 2024, no explicit `resolver` field needed). MSRV is 1.85. See [doc-08 Section 2.1](../tech-strategy/08-build-and-distribution.md#21-workspace-root-cargotoml).

- **Platform dispatch via `#[cfg(target_os)]`, not Cargo features:** Platform-specific backend selection is automatic via `target_os` conditional compilation. Cargo features control optional capabilities (`test_utils`, `wayland`, `xshm`, `profiling`), not platform selection. See [doc-08 Section 3.1](../tech-strategy/08-build-and-distribution.md#31-platform-feature-strategy).

- **`Arc<ArcSwap<AppState>>` for lock-free render thread reads:** The runtime shared state is wrapped in `Arc<ArcSwap<AppState>>` so the render thread (60fps hot path) can read settings without locking. IPC and hotkey handlers write via `rcu()`. See [doc-01 Section 4.6](../tech-strategy/01-system-architecture.md#46-configuration-manager).

- **Trait lifecycle via `Drop`, not explicit `shutdown()`:** All platform backends clean up resources in their `Drop` implementation. No explicit shutdown methods on traits. See [doc-02 Section 3 preamble](../tech-strategy/02-platform-abstraction.md#3-trait-definitions).

- **Error factories in mocks, not stored errors:** Mock `with_error()` methods take `Fn() -> XxxError` closures because error types are not `Clone` (they contain `Box<dyn Error>`). See [doc-02 Section 7.1](../tech-strategy/02-platform-abstraction.md#71-mock-implementations).

- **License: GPL-3.0-only (SPDX):** Use `GPL-3.0-only` SPDX identifier, not the deprecated `GPL-3.0`. See [doc-08 Section 2.1](../tech-strategy/08-build-and-distribution.md#21-workspace-root-cargotoml).

### Key Type Definitions

The following types are defined across Stories 002 and 004. Later stories reference these -- agents implementing Stories 003 and 004 should use these signatures.

**Common types (Story 002, in `luminos-platform`):**
- `ScreenRect { x: i32, y: i32, width: u32, height: u32 }` -- screen-coordinate rectangle
- `ScreenPoint { x: i32, y: i32 }` -- screen-coordinate point
- `DisplayInfo { id: String, name: String, bounds: ScreenRect, scale_factor: f64, is_primary: bool }`
- `PixelFormat` enum: `Bgra8`, `Rgba8`
- `CaptureFrame { data: Arc<[u8]>, width: u32, height: u32, stride: u32, format: PixelFormat }`
- `AudioSample { data: Vec<f32>, sample_rate: u32, channels: u16 }`
- `Voice { id: String, name: String, language: String, requires_download: bool, engine: TtsBackend }`

**Error types (Story 002, in `luminos-platform`):**
- `CaptureError`, `FocusError`, `TtsError`, `WindowError`, `InputError`, `AudioError` -- all derive `thiserror::Error`

**Core types (Story 004, in `luminos-core`):**
- `LuminosError` -- top-level error with `#[from]` conversions for all subsystem errors
- `AppState` -- runtime shared state (wrapped in `Arc<ArcSwap<AppState>>` at higher levels)
- `AppSettings` -- persisted settings schema (serde-serializable)
- `MagnificationMode`, `TrackingMode`, `ColorFilterType` -- configuration enums (variants per doc-05 Section 3)

### Integration Points

- **`luminos-platform` is the foundation crate:** It has zero dependencies on other luminos crates. All other crates depend on it (directly or transitively). This is enforced by the workspace dependency graph.
- **`luminos-core` depends on `luminos-platform`:** For trait types (used in `LuminosError` `From` conversions) and for `AppState` field types (which reference `MagnificationMode` etc.).
- **Mock exports via `test_utils` feature:** Downstream crates import mocks by depending on `luminos-platform = { workspace = true, features = ["test_utils"] }` in their `[dev-dependencies]`.
- **`PlatformBackends` struct:** Defined in `luminos-platform`, bundles five of the six platform trait objects (`ScreenCapture`, `FocusTracker`, `WindowManager`, `InputMonitor`, `AudioOutput`) into a single struct for startup initialization. `TtsEngine` is excluded because it is constructed separately by `luminos-tts` (it depends on `AudioOutput` + espeak-ng subprocess, not on platform APIs directly). See doc-02 Section 5.3.

### Discovered Constraints

_To be updated as stories are implemented. Pre-populated with known constraints:_

- **Tauri is not set up in E1:** `luminos-app` is a binary crate but does NOT include Tauri setup in this epic. It has a minimal `fn main()` that compiles. Tauri initialization is Epic 4 (Control Panel Foundation).
- **No platform backends in E1:** All `linux_x11/`, `linux_wayland/`, `macos/`, `openbsd/`, `windows/` modules are empty stubs (declared with `#[cfg]` gates but containing no code). Real backends start in E2.
- **TypeScript/frontend deferred to E4:** The `ui/` directory is not created in E1. TypeScript CI stages are added in E4.
- **GitHub Actions macOS runners do NOT auto-grant Screen Recording permission** (actions/runner-images#8951) -- not relevant to E1 (Linux-only CI) but worth noting for E2+.

### Cross-Story Dependencies

| Dependency | Source Story | Target Story | Nature |
|------------|-------------|--------------|--------|
| Workspace compiles | 001 | 002, 003, 004, 005 | Hard: crate stubs must exist |
| Trait definitions exist | 002 | 003 | Hard: mocks implement traits |
| Subsystem error types exist | 002 | 004 | Hard: `LuminosError` wraps them via `From` |
| Common types defined | 002 | 003, 004 | Hard: mocks and core types reference them |
| `deny.toml` exists | 001 | 005 | Hard: CI runs `cargo deny check` |
| `.config/nextest.toml` exists | 001 | 005 | Hard: CI uses `--profile ci` |

### Relevant Risks

The following risks from the [Risk Register](../tech-strategy/10-risk-register.md) are relevant to E1 work:

| Risk ID | Title | Score | Relevance to E1 |
|---------|-------|-------|------------------|
| RISK-001 | Dual event loop coexistence (winit + Tauri) | 8 (Mitigate) | In Story 001, run `cargo tree -d` to detect duplicate transitive dependencies between Tauri and winit/wgpu. Workspace dependency deduplication where conflicts arise. Full PoC validation is E2/E4, but dependency conflicts surface in E1. |
| RISK-003 | Platform trait surface area inadequacy | 6 (Monitor) | Trait definitions in Story 002 are designed from research, not implementation. Expect revisions when E2+ backends are built. Mitigation: traits are the coordination contract; revisions are acceptable if mocks are updated in sync. |
| RISK-017 | Screen content leakage via logs and GPU memory | 6 (Monitor) | Story 002: implement custom `Debug` for `CaptureFrame` that prints metadata only (width, height, stride, format), omitting the `data` field. This prevents accidental pixel data leakage in log output. |
| RISK-022 | GPLv3 dependency license compatibility | 6 (Monitor) | `deny.toml` in Story 001 and `cargo deny check` in Story 005 enforce license compliance from day one. |
| RISK-024 | Binary size budget with ONNX Runtime | 9 (Mitigate) | Story 001/005: benchmark stripped binary sizes with the initial workspace dependencies to establish a baseline before ONNX integration in E10. Track binary size in CI from the start. |
| RISK-027 | CI pipeline performance and coverage gaps | 6 (Monitor) | Story 005 establishes the CI pipeline. Initial coverage is build+lint+unit-test only; GPU-dependent and platform-specific stages are added in E2+. |
| RISK-030 | wgpu/winit/Tauri major version upgrade cascade | 9 (Mitigate) | Workspace-level dependency pinning in Story 001 centralizes version management. `Cargo.lock` committed for reproducibility. Run `cargo tree -d` to detect duplicate transitive dependencies early. |
| RISK-034 | AI-agent development model unproven at this scale | 9 (Mitigate) | E1 is the first real test of the AI-agent-driven SDD methodology. The trait-based architecture and crate boundaries are designed to enable parallel AI agent work. E1 validates this model. |

---

## Blockers & Issues Log

| ID | Date | Story | Description | Resolution | Status |
|----|------|-------|-------------|------------|--------|
| B001 | --- | --- | --- | --- | --- |

## Retrospective Notes

_Filled in when the epic is DONE. What went well, what didn't, what to carry forward to future epics._
