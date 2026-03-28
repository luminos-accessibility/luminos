# Story E01/005: CI/CD Pipeline

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** 001 (workspace must exist for CI to build and lint against)

---

## Problem Statement

Without an automated CI/CD pipeline, code quality in Luminos depends entirely on individual contributors remembering to run formatting, linting, license, and vulnerability checks before pushing. This is especially critical for an AI-agent-driven development model where multiple agents may be writing code in parallel across different crates. A single PR with a license-incompatible dependency or a `clippy::unwrap_used` violation could propagate through the codebase undetected.

This story creates a GitHub Actions workflow implementing Stages 1-4 of the CI pipeline from doc-07 Section 4. The pipeline enforces formatting, linting, license compliance, vulnerability scanning, and unit testing on every push and PR. It also provides an optional local pre-commit hook for developers who want fast feedback before pushing.

## User Scenarios

### US-1: CI Enforces Code Quality on Every PR

As a **project maintainer**, I want the CI pipeline to run automatically on every pull request so that no code merges without passing formatting, linting, and testing checks.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given a pull request is opened against `main`, when GitHub Actions triggers, then the CI workflow runs the `lint`, `test-rust-unit`, and `security` jobs.
- **AC-1.2:** Given the CI workflow runs, when `cargo fmt --all -- --check` detects formatting differences, then the `lint` job fails with a non-zero exit code and the PR status check shows failure.
- **AC-1.3:** Given the CI workflow runs, when `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` reports any warning, then the `lint` job fails.
- **AC-1.4:** Given the CI workflow runs, when all stages pass, then the PR status check shows success and the PR is mergeable.

### US-2: CI Catches License Violations

As a **legal compliance stakeholder**, I want the CI pipeline to reject any dependency with a license not on the GPLv3-compatible allowlist so that Luminos never ships code that violates its GPL-3.0-only license.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given the CI workflow runs, when `cargo deny check licenses` is executed and all dependencies have licenses in the `deny.toml` allowlist, then the `security` job passes.
- **AC-2.2:** Given a PR adds a dependency with a license not in the `deny.toml` allowlist (e.g., a proprietary or SSPL-licensed crate), when `cargo deny check licenses` is executed, then the `security` job fails and the output identifies the offending crate and its license.
- **AC-2.3:** Given the CI workflow runs, when `cargo deny check advisories` is executed and no dependencies have active RustSec advisories, then the `security` job passes.

### US-3: CI Runs Unit Tests

As a **developer**, I want the CI pipeline to run all workspace unit tests so that test regressions are caught before merge.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given the CI workflow runs, when `cargo nextest run --profile ci --workspace --exclude luminos-app` is executed and all tests pass, then the `test-rust-unit` job succeeds.
- **AC-3.2:** Given the CI workflow runs, when any unit test fails, then the `test-rust-unit` job fails with a non-zero exit code and the test output identifies the failing test by name.
- **AC-3.3:** Given the CI workflow uses the `ci` nextest profile, when a test is slow (exceeds 60s), then nextest terminates it after 3 retry attempts as configured in `.config/nextest.toml`.

### US-4: CI Catches Vulnerability Advisories

As a **security-conscious maintainer**, I want the CI pipeline to run `cargo audit` so that known vulnerabilities in dependencies are detected before merge.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given the CI workflow runs, when `cargo audit` is executed and no dependencies have CVSS >= 7.0 advisories in the RustSec database, then the `security` job passes.
- **AC-4.2:** Given a dependency has a known vulnerability with CVSS >= 7.0, when `cargo audit` is executed, then the `security` job fails and the output identifies the vulnerable crate, advisory ID, and severity.

### US-5: CI Pipeline Runs Efficiently

As a **developer**, I want the CI pipeline to complete quickly and use caching so that feedback is fast and CI costs are controlled.

**Priority:** P1
**Acceptance Criteria:**

- **AC-5.1:** Given the CI workflow runs, when the Rust toolchain and Cargo registry/build caches are warm (second run on same branch), then the total pipeline time for lint + test + security is under 10 minutes.
- **AC-5.2:** Given the CI workflow, when the job dependency chain is inspected, then `lint` runs first, `test-rust-unit` runs after `lint` passes, and `security` runs in parallel with `test-rust-unit` (both depend on `lint` only).
- **AC-5.3:** Given the CI workflow, when Cargo build artifacts are inspected across jobs, then the workflow uses `actions/cache` to cache `~/.cargo/registry`, `~/.cargo/git`, and `target/` directories keyed on `Cargo.lock` hash.

## Functional Requirements

- **FR-1:** Create `.github/workflows/ci.yml` implementing a GitHub Actions workflow that triggers on `push` to all branches and on `pull_request` to `main`. *(Traces to AC-1.1)*
- **FR-2:** Implement a `lint` job on `ubuntu-latest` that runs: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions`. *(Traces to AC-1.2, AC-1.3)*
- **FR-3:** Implement a `security` job on `ubuntu-latest` that runs `cargo deny check licenses advisories` and `cargo audit`. *(Traces to AC-2.1, AC-2.2, AC-2.3, AC-4.1, AC-4.2)*
- **FR-4:** Implement a `test-rust-unit` job on `ubuntu-latest` that runs `cargo nextest run --profile ci --workspace --exclude luminos-app`. *(Traces to AC-3.1, AC-3.2, AC-3.3)*
- **FR-5:** Configure job dependency chain: `lint` runs first; `test-rust-unit` and `security` depend on `lint` succeeding; `security` runs in parallel with `test-rust-unit`. *(Traces to AC-5.2)*
- **FR-6:** Implement Cargo caching using `actions/cache` with keys based on `Cargo.lock` hash, caching `~/.cargo/registry`, `~/.cargo/git`, and `target/` directories. *(Traces to AC-5.1, AC-5.3)*
- **FR-7:** Install required CI tools (`cargo-nextest`, `cargo-deny`, `cargo-audit`) via `cargo install` or `taiki-e/install-action`. *(Traces to AC-1.1, AC-2.1, AC-4.1)*
- **FR-8:** Set CI environment variables: `CARGO_INCREMENTAL=0` (deterministic builds), `RUSTFLAGS="--deny warnings"` (treat warnings as errors). *(Traces to AC-1.2, AC-1.3)*
- **FR-9:** Add placeholder jobs for `test-shaders` (Stage 3) and `test-integration` (Stage 4) that are configured but contain only a comment noting they will be activated in E2. *(Traces to AC-1.4)*
- **FR-10:** Create optional `.githooks/pre-commit` script that runs `cargo fmt --check` and `cargo clippy` locally. *(Traces to AC-1.2, AC-1.3)*

## Non-Functional Requirements

- **NFR-1:** Total CI pipeline time (lint + test + security) must be under 10 minutes with warm Cargo cache on a GitHub Actions `ubuntu-latest` runner.
- **NFR-2:** The `lint` job must fail fast -- if `cargo fmt --check` fails, subsequent lint steps should not run (fail-fast within the job).
- **NFR-3:** CI pipeline configuration must be readable and maintainable -- use named steps with descriptive names, not opaque one-liners.
- **NFR-4:** Cargo tool installation must be version-pinned or use `latest` with the `taiki-e/install-action` action for reproducibility and speed (pre-built binaries, no compilation needed).

## Out of Scope

- **TypeScript CI stages** (ESLint, Vitest, TypeScript type-checking) -- deferred to Epic 4 (Control Panel Foundation) when the `ui/` directory and frontend scaffolding exist.
- **Shader tests (Stage 3)** -- placeholder only. Real shader tests require wgpu and Mesa llvmpipe setup, introduced in E2 when the rendering pipeline exists.
- **Integration tests (Stage 4)** -- placeholder only. Real integration tests require platform APIs (Xvfb, espeak-ng), introduced in E2+ when backends exist.
- **Performance benchmarks (Stage 5)** -- introduced in E2 when there is code to benchmark.
- **Platform matrix (Stage 6)** -- introduced in E2+ when platform backends exist. E1 CI runs on `ubuntu-latest` only.
- **E2E tests (Stage 7)** -- introduced in E2+ when the application can start.
- **Release builds (Stage 8)** -- introduced when the first release is tagged.
- **Branch protection rule configuration** -- documented in this story but configured manually by the project maintainer in GitHub repository settings (not automatable via Actions).
- **macOS and Windows CI runners** -- deferred to E2+ (platform matrix stage).

## Open Questions

- [x] Should `cargo audit` block PRs or only warn? **Answer:** Block. A CVSS >= 7.0 advisory is a security risk that must be addressed before merge. Lower-severity advisories can be tracked but should not block. `cargo audit` defaults to failing on any advisory; use `--ignore` for known-acceptable advisories documented in `deny.toml` advisories section.
- [x] Should CI install tools via `cargo install` or `taiki-e/install-action`? **Answer:** Use `taiki-e/install-action` for `cargo-nextest`, `cargo-deny`, and `cargo-audit`. It provides pre-built binaries (no compilation), version pinning, and caching, reducing CI time by 2-3 minutes versus `cargo install`.
- [x] Should the workflow use a single job or multiple jobs? **Answer:** Multiple jobs (`lint`, `test-rust-unit`, `security`) with dependencies. This provides granular feedback (developers see which stage failed), enables parallel execution (test + security run in parallel after lint), and avoids re-running all stages when only one fails.
- [x] Should `luminos-app` be included in unit tests? **Answer:** No. `luminos-app` is the binary crate that depends on Tauri, which requires system libraries (WebkitGTK) not available by default on CI runners. Exclude it with `--exclude luminos-app` until E4 when Tauri CI setup is added.
