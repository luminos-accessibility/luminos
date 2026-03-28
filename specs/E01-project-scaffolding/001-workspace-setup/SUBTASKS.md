# Subtasks: Story E01/001 -- Cargo Workspace & Build Profiles

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
| 1. Setup | 3 | 0 | 0 | 3 |
| 2. Core Implementation | 3 | 0 | 0 | 3 |
| 3. Integration | 2 | 0 | 0 | 2 |
| 4. Polish & Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **9** | **0** | **0** | **9** |

---

## Phase 1: Setup

### T001 -- Create workspace root Cargo.toml

**Traces to:** FR-1, FR-4, AC-2.1, AC-2.4, AC-3.1, AC-3.2, AC-3.3
**Status:** TODO
**Files:** `Cargo.toml`

**Steps:**
1. Create the workspace root `Cargo.toml` with:
   - `[workspace]` members listing all five crate paths: `crates/luminos-core`, `crates/luminos-platform`, `crates/luminos-gpu`, `crates/luminos-tts`, `crates/luminos-app`
   - `[workspace.package]` metadata: `version = "0.1.0"`, `edition = "2024"`, `license = "GPL-3.0-only"`, `repository`, `homepage`, `authors`, `rust-version = "1.85"`
   - `[workspace.dependencies]` declaring all external and internal crate dependencies per DESIGN.md
   - `[profile.dev]` with `opt-level = 0`, `debug = true`, `incremental = true`
   - `[profile.release]` with `opt-level = 3`, `debug = false`, `lto = "thin"`, `codegen-units = 16`, `strip = "debuginfo"`
   - `[profile.dist]` inheriting from `release` with `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`
2. No explicit `resolver` field (edition 2024 defaults to resolver 3)

**Verification:** File parses as valid TOML. Profile fields match AC-3.1, AC-3.2, AC-3.3.

**Completion Notes:**
>

---

### T002 [P] -- Create project configuration files

**Traces to:** FR-5, FR-6, FR-7, FR-8, FR-9, FR-10, FR-11, AC-4.1, AC-4.2, AC-4.3, AC-4.5
**Status:** TODO
**Files:** `rust-toolchain.toml`, `.clippy.toml`, `rustfmt.toml`, `deny.toml`, `.config/nextest.toml`, `LICENSE`, `CHANGELOG.md`

**Steps:**
1. Create `rust-toolchain.toml` with `channel = "stable"` and `components = ["rustfmt", "clippy"]`
2. Create `.clippy.toml` with `cognitive-complexity-threshold = 25`, `too-many-arguments-threshold = 7`, `type-complexity-threshold = 250`
3. Create `rustfmt.toml` with `edition = "2024"`
4. Create `deny.toml` with:
   - `[graph]`: `targets = []`, `all-features = true`
   - `[licenses]`: `allow` list containing `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `MPL-2.0`, `GPL-3.0-only`, `GPL-3.0-or-later`, `LGPL-2.1-only`, `LGPL-2.1-or-later`, `LGPL-3.0-only`, `LGPL-3.0-or-later`, `Unicode-3.0`, `Unicode-DFS-2016`; `confidence-threshold = 0.8`
   - `[advisories]`: `vulnerability = "deny"`, `unmaintained = "warn"`, `yanked = "warn"`, `notice = "warn"`
   - `[bans]`: `multiple-versions = "warn"`, `wildcards = "deny"`
5. Create `.config/nextest.toml` with `[profile.default]` (`fail-fast = true`, `retries = 0`, `slow-timeout`) and `[profile.ci]` (`fail-fast = false`, `retries = 2`, `slow-timeout`) per DESIGN.md
6. Create `LICENSE` file with full GPL-3.0-only text
7. Create `CHANGELOG.md` with Keep a Changelog format header (empty)

**Verification:** Each file is syntactically valid. `deny.toml` allow list matches AC-4.3. `nextest.toml` profiles match AC-4.5.

**Completion Notes:**
>

---

### T003 [P] -- Create five crate stubs with minimal source files

**Traces to:** FR-2, AC-1.1, AC-2.1, AC-2.3
**Status:** TODO
**Files:** `crates/luminos-platform/Cargo.toml`, `crates/luminos-platform/src/lib.rs`, `crates/luminos-gpu/Cargo.toml`, `crates/luminos-gpu/src/lib.rs`, `crates/luminos-tts/Cargo.toml`, `crates/luminos-tts/src/lib.rs`, `crates/luminos-core/Cargo.toml`, `crates/luminos-core/src/lib.rs`, `crates/luminos-app/Cargo.toml`, `crates/luminos-app/src/main.rs`

**Steps:**
1. Create directory structure: `crates/luminos-platform/src/`, `crates/luminos-gpu/src/`, `crates/luminos-tts/src/`, `crates/luminos-core/src/`, `crates/luminos-app/src/`
2. Create each crate's `Cargo.toml` with:
   - `[package]` inheriting `version`, `edition`, `license`, `rust-version` from workspace (`.workspace = true`)
   - Dependencies declared per DESIGN.md (using `{ workspace = true }` inheritance)
   - `[features]` sections defined per FR-12 (detailed in T005)
3. Create `src/lib.rs` for library crates with doc-comment only (no code):
   - `luminos-platform`: "Platform abstraction layer for Luminos."
   - `luminos-gpu`: "GPU-accelerated rendering pipeline for Luminos."
   - `luminos-tts`: "Text-to-speech pipeline for Luminos."
   - `luminos-core`: "Core engine and state management for Luminos."
4. Create `crates/luminos-app/src/main.rs` with `fn main() {}` and doc-comment
5. Create `crates/luminos-app/build.rs` with `fn main() {}` only. Do NOT call `tauri_build::build()` -- Tauri setup is deferred to E4.

**Verification:** All five crate directories exist with correct `Cargo.toml` and source files.

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 1, verify:
- [ ] All files exist in the expected locations
- [ ] `Cargo.toml` workspace root parses without errors
- [ ] Each crate `Cargo.toml` parses without errors

---

## Phase 2: Core Implementation

### T004 -- Configure inter-crate dependency graph

**Traces to:** FR-3, AC-2.2
**Status:** TODO
**Files:** `crates/luminos-platform/Cargo.toml`, `crates/luminos-gpu/Cargo.toml`, `crates/luminos-tts/Cargo.toml`, `crates/luminos-core/Cargo.toml`, `crates/luminos-app/Cargo.toml`

**Steps:**
1. Ensure `luminos-platform` has NO internal crate dependencies (foundation crate)
2. Add to `luminos-gpu/Cargo.toml` `[dependencies]`: `luminos-platform = { workspace = true }`, `wgpu`, `winit`, `log`
3. Add to `luminos-tts/Cargo.toml` `[dependencies]`: `luminos-platform = { workspace = true }`, `sherpa-rs`, `cpal`, `log`
4. Add to `luminos-core/Cargo.toml` `[dependencies]`: `luminos-platform = { workspace = true }`, `luminos-tts = { workspace = true }`, `serde`, `serde_json`, `toml`, `arc-swap`, `crossbeam-channel`, `log`, `thiserror`, `arboard`
5. Add to `luminos-app/Cargo.toml` `[dependencies]`: `luminos-core`, `luminos-platform`, `luminos-gpu`, `luminos-tts` (all `{ workspace = true }`), plus `tauri`, `tauri-specta`, `specta-typescript`, `log`, `env_logger`, `arc-swap`, `serde`, `serde_json`
6. Add `luminos-app` `[build-dependencies]`: `tauri-build = { workspace = true }`
7. Add platform-specific deps for `luminos-platform` under `[target.'cfg(...)'.dependencies]` per DESIGN.md

**Verification:** `cargo tree -p luminos-app` shows correct dependency edges per AC-2.2. No circular dependencies.

**Completion Notes:**
>

---

### T005 -- Define Cargo feature flags per crate

**Traces to:** FR-12, AC-5.1, AC-5.2, AC-5.3, AC-5.4, AC-5.5
**Status:** TODO
**Files:** `crates/luminos-platform/Cargo.toml`, `crates/luminos-gpu/Cargo.toml`, `crates/luminos-tts/Cargo.toml`, `crates/luminos-core/Cargo.toml`, `crates/luminos-app/Cargo.toml`

**Steps:**
1. `luminos-platform` features: `default = []`, `wayland = ["ashpd"]`, `xshm = ["x11rb/shm"]`, `test_utils = []`, `ci_platform_tests = []`
2. `luminos-gpu` features: `default = []`, `test_utils = []`, `update_refs = []`, `profiling = []`
3. `luminos-tts` features: `default = []`, `test_utils = []`
4. `luminos-core` features: `default = []`, `test_utils = ["luminos-platform/test_utils", "luminos-gpu/test_utils", "luminos-tts/test_utils"]`
5. `luminos-app` features: `default = []`, `integration_tests = ["luminos-core/test_utils"]`, `ci_platform_tests = ["luminos-platform/ci_platform_tests"]`, `profiling = ["luminos-gpu/profiling"]`
6. Verify `ashpd` optional dep is declared in `luminos-platform` under `[target.'cfg(target_os = "linux")'.dependencies.ashpd]`

**Verification:** Each crate's `[features]` section matches STORY.md AC-5.1 through AC-5.5.

**Completion Notes:**
>

---

### T006 -- Commit Cargo.lock and run initial dependency resolution

**Traces to:** FR-13, AC-1.1, AC-4.4
**Status:** TODO
**Files:** `Cargo.lock`

**Steps:**
1. Run `cargo generate-lockfile` (or `cargo build --workspace` which generates it) to create `Cargo.lock`
2. Verify `Cargo.lock` is committed to the repository (not `.gitignore`d)
3. Run `cargo tree -d` to check for duplicate transitive dependencies (RISK-001, RISK-030)
4. Document any duplicates found in Completion Notes

**Verification:** `Cargo.lock` exists and is tracked by git. `cargo tree -d` output reviewed.

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 2, verify:
- [ ] `cargo build --workspace` compiles successfully (may have warnings at this point)
- [ ] `cargo tree -p luminos-app` shows correct dependency graph per AC-2.2
- [ ] All feature flags are defined correctly

---

## Phase 3: Integration

### T007 -- Verify workspace builds and passes linting

**Traces to:** AC-1.1, AC-1.2, AC-1.3, NFR-2
**Status:** TODO
**Files:** All crate source files (may need adjustments to pass clippy/fmt)

**Steps:**
1. Run `cargo build --workspace` -- must succeed with zero errors and zero warnings
2. Run `cargo fmt --all -- --check` -- must report no differences
3. Run `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` -- must pass with zero warnings
4. Fix any issues found (add `#![allow(...)]` only as a last resort; prefer fixing the code)
5. Run `RUSTFLAGS="--deny warnings" cargo build --workspace` to verify NFR-2

**Verification:** All three commands exit with code 0. Zero warnings, zero errors.

**Completion Notes:**
>

---

### T008 -- Verify license compliance and dist profile

**Traces to:** FR-8, AC-3.4, AC-4.4
**Status:** TODO
**Files:** `deny.toml` (may need adjustments), `Cargo.toml` (profiles)

**Steps:**
1. Run `cargo deny check licenses advisories` -- must pass with zero violations (AC-4.4)
2. If violations are found, update `deny.toml` allow list only for GPLv3-compatible licenses
3. Run `cargo build --profile dist -p luminos-app` -- must succeed and produce binary at `target/dist/luminos-app` (AC-3.4)
4. Verify the dist binary exists at the expected path

**Verification:** `cargo deny check` exits 0. Dist binary exists at `target/dist/luminos-app`.

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 3, verify:
- [ ] `cargo build --workspace` -- zero warnings, zero errors
- [ ] `cargo clippy` -- clean
- [ ] `cargo fmt --check` -- clean
- [ ] `cargo deny check` -- passes
- [ ] `cargo build --profile dist -p luminos-app` -- produces binary

---

## Phase 4: Polish & Acceptance

### T009 -- Final acceptance verification

**Traces to:** All ACs, All FRs
**Status:** TODO

**Verification Checklist:**

*US-1: New Contributor Clones and Builds*
- [ ] AC-1.1: `cargo build --workspace` succeeds with exit code 0 and zero warnings
- [ ] AC-1.2: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` passes
- [ ] AC-1.3: `cargo fmt --all -- --check` passes with no differences

*US-2: Workspace Crate Structure*
- [ ] AC-2.1: `[workspace]` members contains exactly five crate paths
- [ ] AC-2.2: `cargo tree -p luminos-app` shows correct dependency graph
- [ ] AC-2.3: Each crate inherits `version`, `edition`, `license`, `rust-version` from workspace
- [ ] AC-2.4: `[workspace.package]` has `edition = "2024"`, `license = "GPL-3.0-only"`, `rust-version = "1.85"`, `version = "0.1.0"`

*US-3: Build Profiles*
- [ ] AC-3.1: `[profile.dev]` matches specified values
- [ ] AC-3.2: `[profile.release]` matches specified values
- [ ] AC-3.3: `[profile.dist]` matches specified values (inherits release, `opt-level = "z"`, etc.)
- [ ] AC-3.4: `cargo build --profile dist -p luminos-app` succeeds and binary exists

*US-4: Configuration Files*
- [ ] AC-4.1: `rust-toolchain.toml` specifies stable channel with rustfmt + clippy components
- [ ] AC-4.2: `.clippy.toml` thresholds match specified values
- [ ] AC-4.3: `deny.toml` allow list contains all specified licenses, `confidence-threshold = 0.8`
- [ ] AC-4.4: `cargo deny check licenses advisories` passes
- [ ] AC-4.5: `.config/nextest.toml` has correct default and ci profiles

*US-5: Cargo Features*
- [ ] AC-5.1: `luminos-platform` features match spec
- [ ] AC-5.2: `luminos-gpu` features match spec
- [ ] AC-5.3: `luminos-core` `test_utils` transitively enables sub-crate `test_utils`
- [ ] AC-5.4: `luminos-tts` features match spec
- [ ] AC-5.5: `luminos-app` features match spec

*NFRs*
- [ ] NFR-1: Build time is reasonable (measured in CI, Story 005)
- [ ] NFR-2: Zero warnings with `RUSTFLAGS="--deny warnings"`
- [ ] NFR-3: No `unwrap()` or `expect()` in production source files
- [ ] NFR-4: All workspace dependencies use `{ workspace = true }` inheritance

*General*
- [ ] All clippy warnings resolved
- [ ] `Cargo.lock` is committed
- [ ] `LICENSE` file exists with GPL-3.0-only text
- [ ] `CHANGELOG.md` exists with Keep a Changelog header

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
