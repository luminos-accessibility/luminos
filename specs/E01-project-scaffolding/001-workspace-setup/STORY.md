# Story E01/001: Cargo Workspace & Build Profiles

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** None (first story in epic)

---

## Problem Statement

Luminos cannot begin development until a well-structured Cargo workspace exists. Every subsequent story in E1 -- and every subsequent epic in the project -- depends on the workspace skeleton compiling cleanly. Without centralized dependency management, build profiles, and project-level configuration files, contributors (human and AI agent) cannot build, lint, or test any Luminos code.

This story creates the foundational project scaffolding: the workspace root manifest, five crate stubs matching the architecture in doc-01 Section 7, build profiles from doc-08 Section 4, Rust toolchain configuration, linting/formatting configuration, license compliance tooling (`deny.toml`), test runner configuration (`nextest.toml`), and project-level metadata files. The deliverable is a repository that passes `cargo build --workspace`, `cargo clippy`, `cargo fmt --check`, and `cargo deny check` with zero warnings and zero errors.

## User Scenarios

### US-1: New Contributor Clones and Builds

As a **new contributor** (human or AI agent), I want to clone the Luminos repository and run `cargo build --workspace` so that I get a clean build with zero warnings and zero errors, confirming the development environment is ready.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given a fresh clone of the repository with rustc 1.85+ installed, when the contributor runs `cargo build --workspace`, then the build succeeds with exit code 0 and produces zero compiler warnings.
- **AC-1.2:** Given the workspace is built, when the contributor runs `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions`, then clippy passes with zero warnings and zero errors.
- **AC-1.3:** Given the workspace is built, when the contributor runs `cargo fmt --all -- --check`, then the formatting check passes with no differences reported.

### US-2: Workspace Crate Structure Matches Architecture

As the **system architect**, I want the workspace to contain exactly five crates with the correct dependency graph so that the crate boundaries enforce the architectural layering defined in doc-01 Section 7.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given the workspace root `Cargo.toml`, when the `[workspace]` members are inspected, then the members list contains exactly: `crates/luminos-core`, `crates/luminos-platform`, `crates/luminos-gpu`, `crates/luminos-tts`, `crates/luminos-app`.
- **AC-2.2:** Given the workspace compiles, when `cargo tree -p luminos-app` is run, then `luminos-app` depends on `luminos-core`, `luminos-platform`, `luminos-gpu`, and `luminos-tts`; `luminos-core` depends on `luminos-platform` and `luminos-tts`; `luminos-gpu` depends on `luminos-platform`; `luminos-tts` depends on `luminos-platform`; and `luminos-platform` has no internal crate dependencies.
- **AC-2.3:** Given each crate's `Cargo.toml`, when the `[package]` section is inspected, then `version`, `edition`, `license`, and `rust-version` all use `{ workspace = true }` inheritance from the workspace root.
- **AC-2.4:** Given the workspace root `[workspace.package]`, when its fields are inspected, then `edition` is `"2024"`, `license` is `"GPL-3.0-only"`, `rust-version` is `"1.85"`, and `version` is `"0.1.0"`.

### US-3: Build Profiles Produce Correct Optimization Levels

As a **developer**, I want `dev`, `release`, and `dist` build profiles configured so that I can iterate quickly in development, benchmark realistically with `release`, and produce size-optimized binaries for distribution with `dist`.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given the workspace root `Cargo.toml`, when the `[profile.dev]` section is inspected, then `opt-level` is `0`, `debug` is `true`, and `incremental` is `true`.
- **AC-3.2:** Given the workspace root `Cargo.toml`, when the `[profile.release]` section is inspected, then `opt-level` is `3`, `lto` is `"thin"`, `codegen-units` is `16`, and `strip` is `"debuginfo"`.
- **AC-3.3:** Given the workspace root `Cargo.toml`, when the `[profile.dist]` section is inspected, then it inherits from `release`, sets `opt-level` to `"z"`, `lto` to `"fat"`, `codegen-units` to `1`, `panic` to `"abort"`, and `strip` to `"symbols"`.
- **AC-3.4:** Given the workspace compiles, when `cargo build --profile dist -p luminos-app` is run, then the build succeeds and produces a binary at `target/dist/luminos-app` (or platform equivalent).

### US-4: Project Configuration Files Are Valid

As a **CI pipeline** (Story 005), I want all project configuration files to exist and be syntactically valid so that CI stages can reference them without setup failures.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given the repository root, when `rust-toolchain.toml` is inspected, then it specifies the `stable` toolchain channel with Rust edition 2024 compatibility (rustc 1.85+).
- **AC-4.2:** Given the repository root, when `.clippy.toml` is inspected, then it contains `cognitive-complexity-threshold = 25`, `too-many-arguments-threshold = 7`, and `type-complexity-threshold = 250`.
- **AC-4.3:** Given the repository root, when `deny.toml` is inspected, then the `[licenses]` section contains an `allow` list that includes at minimum: `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `MPL-2.0`, `GPL-3.0-only`, `GPL-3.0-or-later`, `LGPL-2.1-only`, `LGPL-2.1-or-later`, `LGPL-3.0-only`, `LGPL-3.0-or-later`, `Unicode-3.0`, `Unicode-DFS-2016`, and the `confidence-threshold` is `0.8`.
- **AC-4.4:** Given the repository root, when `cargo deny check licenses advisories` is run against the workspace, then the check passes with zero violations.
- **AC-4.5:** Given the repository root, when `.config/nextest.toml` is inspected, then it defines a `default` profile with `fail-fast = true` and a `ci` profile with `retries = 2` and `fail-fast = false`.

### US-5: Cargo Feature Definitions Match Architecture

As a **platform backend developer** (E2+), I want Cargo feature flags pre-defined on each crate so that conditional compilation for `test_utils`, `wayland`, `xshm`, `profiling`, and other optional capabilities is available without modifying crate manifests later.

**Priority:** P1
**Acceptance Criteria:**

- **AC-5.1:** Given `crates/luminos-platform/Cargo.toml`, when the `[features]` section is inspected, then it defines `default = []`, `wayland`, `xshm`, `test_utils`, and `ci_platform_tests` features as specified in doc-08 Section 3.2.
- **AC-5.2:** Given `crates/luminos-gpu/Cargo.toml`, when the `[features]` section is inspected, then it defines `default = []`, `test_utils`, `update_refs`, and `profiling` features.
- **AC-5.3:** Given `crates/luminos-core/Cargo.toml`, when the `[features]` section is inspected, then it defines `test_utils` which transitively enables `luminos-platform/test_utils`, `luminos-gpu/test_utils`, and `luminos-tts/test_utils`.
- **AC-5.4:** Given `crates/luminos-tts/Cargo.toml`, when the `[features]` section is inspected, then it defines `default = []` and `test_utils`.
- **AC-5.5:** Given `crates/luminos-app/Cargo.toml`, when the `[features]` section is inspected, then it defines `default = []`, `integration_tests` (enabling `luminos-core/test_utils`), `ci_platform_tests` (enabling `luminos-platform/ci_platform_tests`), and `profiling` (enabling `luminos-gpu/profiling`) as specified in doc-08 Section 3.2.

## Functional Requirements

- **FR-1:** Create `Cargo.toml` workspace root with `[workspace]` members, `[workspace.package]` metadata, and `[workspace.dependencies]` as specified in doc-08 Section 2.1-2.2. *(Traces to AC-1.1, AC-2.1, AC-2.4)*
- **FR-2:** Create five crate stubs (`luminos-core`, `luminos-platform`, `luminos-gpu`, `luminos-tts`, `luminos-app`) under `crates/`, each with `Cargo.toml` inheriting workspace metadata and a minimal `src/lib.rs` (or `src/main.rs` for `luminos-app`). *(Traces to AC-1.1, AC-2.1, AC-2.3)*
- **FR-3:** Configure the inter-crate dependency graph matching doc-01 Section 7.2. *(Traces to AC-2.2)*
- **FR-4:** Define `dev`, `release`, and `dist` build profiles in the workspace root as specified in doc-08 Section 4. *(Traces to AC-3.1, AC-3.2, AC-3.3, AC-3.4)*
- **FR-5:** Create `rust-toolchain.toml` specifying Rust 2024 edition compatibility. *(Traces to AC-4.1)*
- **FR-6:** Create `.clippy.toml` with the thresholds specified in doc-07 Section 4.2. *(Traces to AC-4.2)*
- **FR-7:** Create `rustfmt.toml` for consistent formatting. *(Traces to AC-1.3)*
- **FR-8:** Create `deny.toml` with GPLv3-compatible license allowlist per doc-07 Section 4.2. *(Traces to AC-4.3, AC-4.4)*
- **FR-9:** Create `.config/nextest.toml` with `default` and `ci` profiles per doc-07 Section 3.1. *(Traces to AC-4.5)*
- **FR-10:** Create `LICENSE` file containing the full GPL-3.0-only license text. *(Traces to AC-2.4)*
- **FR-11:** Create `CHANGELOG.md` with Keep a Changelog format header (empty, initialized). *(Traces to AC-1.1)*
- **FR-12:** Define Cargo feature flags per crate matching doc-08 Section 3.2. *(Traces to AC-5.1, AC-5.2, AC-5.3, AC-5.4, AC-5.5)*
- **FR-13:** Commit `Cargo.lock` to the repository for build reproducibility. *(Traces to AC-1.1)*

## Non-Functional Requirements

- **NFR-1:** Clean build (`cargo build --workspace`) must complete in under 5 minutes on a GitHub Actions `ubuntu-latest` runner with cold Cargo cache, or under 2 minutes with warm cache.
- **NFR-2:** `cargo build --workspace` must produce zero warnings with `RUSTFLAGS="--deny warnings"`.
- **NFR-3:** No `unwrap()` or `expect()` calls in any production source file (crate `src/` directories). Exception: unit tests.
- **NFR-4:** All workspace dependencies must use `{ workspace = true }` inheritance -- no crate may declare a dependency version directly if that dependency is declared in `[workspace.dependencies]`.

## Out of Scope

- **Trait definitions and type implementations** -- handled by Story 002 (Platform Trait Definitions & Common Types).
- **Mock implementations** -- handled by Story 003 (Mock Implementations & Test Utilities).
- **Error hierarchy and core data types** -- handled by Story 004 (Error Hierarchy & Core Data Types).
- **CI/CD pipeline** -- handled by Story 005 (CI/CD Pipeline). This story creates configuration files CI depends on (`deny.toml`, `.config/nextest.toml`, `.clippy.toml`) but does not create the GitHub Actions workflow.
- **Tauri setup** -- deferred to Epic 4 (Control Panel Foundation). `luminos-app` has a minimal `fn main() {}` only.
- **TypeScript/frontend scaffolding** -- deferred to Epic 4. No `ui/` directory is created.
- **Platform backend module stubs** -- created in Story 002 when `luminos-platform/src/lib.rs` is populated with `#[cfg]`-gated module declarations.
- **External dependency usage** -- workspace dependencies (wgpu, winit, tauri, etc.) are declared but not imported in crate source files. Actual usage begins in later stories and epics.

## Open Questions

- [x] Should `luminos-app` include Tauri as a dependency in E1, or defer it to E4? **Answer:** Declare `tauri` in workspace dependencies and in `luminos-app/Cargo.toml`, but do not initialize Tauri in `main.rs`. This validates dependency resolution and license compliance in E1. The `main.rs` contains only `fn main() {}`.
- [x] Should `opt-level = "z"` or `"s"` be used for the dist profile? **Answer:** Start with `"z"` as specified in doc-08 Section 4.3. Benchmark both during Phase 0 and adopt the smaller result. The dist profile is not used until CI Stage 8 (release builds).
- [x] Should `Cargo.lock` be committed? **Answer:** Yes. Committed for reproducibility per doc-08 Section 2.2 version pinning strategy.
