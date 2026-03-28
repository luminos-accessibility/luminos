# Subtasks: Story E01/005 -- CI/CD Pipeline

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
| 2. Core Implementation | 4 | 0 | 0 | 4 |
| 3. Integration | 2 | 0 | 0 | 2 |
| 4. Polish & Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **8** | **0** | **0** | **8** |

---

## Phase 1: Setup

### T001 -- Create workflow directory and pre-commit hook

**Traces to:** FR-1, FR-10, AC-1.1
**Status:** TODO
**Files:** `.github/workflows/` (directory), `.githooks/pre-commit`

**Steps:**
1. Create `.github/workflows/` directory
2. Create `.githooks/pre-commit` script with:
   - Shebang: `#!/usr/bin/env bash`
   - Comment: instructions for install (`git config core.hooksPath .githooks`)
   - `set -e`
   - Run `cargo fmt --all -- --check`
   - Run `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions`
   - Echo success message
3. Make `.githooks/pre-commit` executable (`chmod +x`)

**Verification:** `.github/workflows/` directory exists. `.githooks/pre-commit` is executable and contains correct commands.

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 1, verify:
- [ ] Directory structure exists
- [ ] Pre-commit hook is executable

---

## Phase 2: Core Implementation

### T002 -- Implement lint job in ci.yml

**Traces to:** FR-1, FR-2, FR-6, FR-7, FR-8, AC-1.1, AC-1.2, AC-1.3, AC-2.1, AC-5.2, AC-5.3
**Status:** TODO
**Files:** `.github/workflows/ci.yml`

**Steps:**
1. Create `.github/workflows/ci.yml` with:
   - `name: CI`
   - `on:` triggers for `push` (all branches) and `pull_request` (main)
   - Global `env:` with `CARGO_INCREMENTAL: "0"`, `CARGO_TERM_COLOR: always`, `RUSTFLAGS: "--deny warnings"`
2. Define `lint` job on `ubuntu-latest` with steps:
   - `actions/checkout@v4`
   - `dtolnay/rust-toolchain@stable` with `components: rustfmt, clippy`
   - `actions/cache@v4` caching `~/.cargo/registry`, `~/.cargo/git`, `target` with key `lint-${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
   - `cargo fmt --all -- --check` (named "Check formatting")
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` (named "Run Clippy")
   - Install `cargo-deny` via `taiki-e/install-action@cargo-deny`
   - `cargo deny check licenses advisories` (named "Check licenses and advisories")

**Note:** Create the `ci.yml` file with ONLY the lint job in this task. Subsequent jobs (`test-rust-unit`, `security`, placeholders) are added incrementally in T003-T005.

**Verification:** YAML is syntactically valid. Lint job has all required steps in order (fmt -> clippy -> deny).

**Completion Notes:**
>

---

### T003 -- Implement test-rust-unit job in ci.yml

**Traces to:** FR-4, FR-5, FR-6, FR-7, AC-3.1, AC-3.2, AC-3.3, AC-5.2, AC-5.3
**Status:** TODO
**Files:** `.github/workflows/ci.yml`

**Steps:**
1. Add `test-rust-unit` job to `ci.yml` on `ubuntu-latest` with `needs: [lint]`:
   - `actions/checkout@v4`
   - `dtolnay/rust-toolchain@stable`
   - `actions/cache@v4` caching same paths with key `test-${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
   - Install `cargo-nextest` via `taiki-e/install-action@nextest`
   - `cargo nextest run --profile ci --workspace --exclude luminos-app` (named "Run unit tests")

**Verification:** Job depends on `lint`. Uses `--profile ci` and `--exclude luminos-app`.

**Completion Notes:**
>

---

### T004 -- Implement security job in ci.yml

**Traces to:** FR-3, FR-5, FR-6, FR-7, AC-2.1, AC-2.2, AC-2.3, AC-4.1, AC-4.2, AC-5.2, AC-5.3
**Status:** TODO
**Files:** `.github/workflows/ci.yml`

**Steps:**
1. Add `security` job to `ci.yml` on `ubuntu-latest` with `needs: [lint]`:
   - `actions/checkout@v4`
   - `dtolnay/rust-toolchain@stable`
   - `actions/cache@v4` caching `~/.cargo/registry` and `~/.cargo/git` (no `target` needed) with key `security-${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
   - Install `cargo-deny` via `taiki-e/install-action@cargo-deny`
   - `cargo deny check licenses advisories` (named "Check licenses and advisories")
   - Install `cargo-audit` via `taiki-e/install-action@cargo-audit`
   - `cargo audit` (named "Run vulnerability audit")
2. Verify `security` runs in parallel with `test-rust-unit` (both depend on `lint`, not on each other)

**Verification:** Job depends only on `lint`. Contains both `cargo deny check` and `cargo audit` steps.

**Completion Notes:**
>

---

### T005 -- Add placeholder jobs for Stages 3 and 4

**Traces to:** FR-9, AC-1.4
**Status:** TODO
**Files:** `.github/workflows/ci.yml`

**Steps:**
1. Add `test-shaders` placeholder job:
   - `runs-on: ubuntu-latest`, `needs: [lint]`
   - `if: false` (disabled until E2)
   - Single checkout step with TODO comment explaining what will be added in E2 (Mesa llvmpipe, shader tests)
2. Add `test-integration` placeholder job:
   - `runs-on: ubuntu-latest`, `needs: [test-rust-unit]`
   - `if: false` (disabled until E2+)
   - Single checkout step with TODO comment explaining what will be added (espeak-ng, Xvfb, Kokoro model)

**Verification:** Both jobs have `if: false` and clear TODO comments. `test-shaders` depends on `lint`; `test-integration` depends on `test-rust-unit`.

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 2, verify:
- [ ] `ci.yml` is valid YAML (can be validated with `python -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` or similar)
- [ ] Job dependency chain: lint -> (test-rust-unit, security) in parallel
- [ ] Placeholder jobs are disabled with `if: false`
- [ ] All tool installations use `taiki-e/install-action`
- [ ] Environment variables set at workflow level (`CARGO_INCREMENTAL=0`, `RUSTFLAGS`)

---

## Phase 3: Integration

### T006 -- Validate workflow YAML and cache configuration

**Traces to:** FR-6, AC-5.1, AC-5.3, NFR-1, NFR-3, NFR-4
**Status:** TODO
**Files:** `.github/workflows/ci.yml`

**Steps:**
1. Validate `ci.yml` is syntactically correct YAML
2. Verify all `actions/cache@v4` steps:
   - Cache paths include `~/.cargo/registry`, `~/.cargo/git`, and `target/` (for lint and test jobs)
   - Cache keys contain `${{ hashFiles('**/Cargo.lock') }}`
   - Restore keys provide fallback for partial cache hits
3. Verify job dependency chain:
   - `test-rust-unit` has `needs: [lint]`
   - `security` has `needs: [lint]`
   - `test-rust-unit` and `security` do NOT depend on each other (parallel execution)
4. Verify all step names are descriptive (NFR-3)
5. Verify tool installations use `taiki-e/install-action` (NFR-4):
   - `cargo-deny` via `taiki-e/install-action@cargo-deny`
   - `cargo-nextest` via `taiki-e/install-action@nextest`
   - `cargo-audit` via `taiki-e/install-action@cargo-audit`

**Verification:** All checks pass. YAML validated. Cache keys are Cargo.lock-based.

**Completion Notes:**
>

---

### T007 -- Push to branch and verify CI execution

**Traces to:** AC-1.1, AC-1.4, AC-2.1, AC-2.3, AC-3.1, AC-4.1, AC-5.1, AC-5.2
**Status:** TODO
**Files:** None (verification only)

**Steps:**
1. Commit the CI workflow and pre-commit hook to a feature branch
2. Push the branch to trigger the CI workflow
3. Observe GitHub Actions UI:
   - Verify `lint`, `test-rust-unit`, and `security` jobs appear (AC-1.1)
   - Verify `lint` runs first, then `test-rust-unit` and `security` in parallel (AC-5.2)
   - Verify all three jobs pass with green status (AC-1.4)
4. Verify failure detection (AC-1.2, AC-1.3):
   - Create a temporary commit with a formatting violation (e.g., missing trailing newline or extra whitespace), push, verify `lint` job fails with formatting diff in output
   - Create a temporary commit with `let _ = vec![1].first().unwrap();` in a lib.rs, push, verify `lint` job fails with clippy output
   - Revert both temporary commits after verification
5. Note total pipeline time for warm cache runs (target: < 10 minutes, AC-5.1)
6. Record timing observations in Completion Notes

**Verification:** All CI jobs pass on clean code. Failure scenarios verified for fmt and clippy. Pipeline time observed and recorded.

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 3, verify:
- [ ] CI workflow runs successfully on GitHub Actions
- [ ] All three active jobs pass
- [ ] Job dependency chain is correct (lint gates test + security)
- [ ] Caching is working (second run faster than first)

---

## Phase 4: Polish & Acceptance

### T008 -- Final acceptance verification

**Traces to:** All ACs, All FRs, All NFRs
**Status:** TODO

**Verification Checklist:**

*US-1: CI Enforces Code Quality on Every PR*
- [ ] AC-1.1: CI workflow triggers on PR against `main` and runs `lint`, `test-rust-unit`, `security` jobs
- [ ] AC-1.2: Formatting violations cause `lint` job to fail (verified by inspection or temporary breakage)
- [ ] AC-1.3: Clippy warnings cause `lint` job to fail (verified by inspection or temporary breakage)
- [ ] AC-1.4: All stages passing results in green PR status check

*US-2: CI Catches License Violations*
- [ ] AC-2.1: `cargo deny check licenses` passes on current workspace
- [ ] AC-2.2: Design ensures incompatible licenses would cause failure (verified by `deny.toml` allowlist-only approach)
- [ ] AC-2.3: `cargo deny check advisories` passes on current workspace

*US-3: CI Runs Unit Tests*
- [ ] AC-3.1: `cargo nextest run --profile ci --workspace --exclude luminos-app` runs and succeeds
- [ ] AC-3.2: Failing tests cause `test-rust-unit` job to fail (verified by design)
- [ ] AC-3.3: `ci` nextest profile has `slow-timeout = { period = "60s", terminate-after = 3 }` and `retries = 2`

*US-4: CI Catches Vulnerability Advisories*
- [ ] AC-4.1: `cargo audit` completes without failure on current workspace
- [ ] AC-4.2: Design ensures CVSS >= 7.0 advisories would cause failure (verified by `cargo audit` default behavior)

*US-5: CI Pipeline Runs Efficiently*
- [ ] AC-5.1: Warm-cache pipeline time < 10 minutes (observed in T007)
- [ ] AC-5.2: `test-rust-unit` and `security` both `needs: [lint]` only (parallel after lint)
- [ ] AC-5.3: `actions/cache@v4` caches `~/.cargo/registry`, `~/.cargo/git`, `target/` with `Cargo.lock` hash key

*NFRs*
- [ ] NFR-1: Total pipeline time < 10 minutes (warm cache)
- [ ] NFR-2: `lint` job steps are sequential (fmt -> clippy -> deny); failure stops subsequent steps
- [ ] NFR-3: All steps have descriptive names
- [ ] NFR-4: All tools installed via `taiki-e/install-action`

*FRs*
- [ ] FR-1: Workflow triggers on push (all branches) and pull_request (main)
- [ ] FR-2: Lint job runs fmt + clippy
- [ ] FR-3: Security job runs `cargo deny check licenses advisories`
- [ ] FR-4: Test job runs `cargo nextest run --profile ci --workspace --exclude luminos-app`
- [ ] FR-5: Job dependency chain correct
- [ ] FR-6: Caching configured with `actions/cache@v4`
- [ ] FR-7: Tools installed via `taiki-e/install-action`
- [ ] FR-8: `CARGO_INCREMENTAL=0` and `RUSTFLAGS="--deny warnings"` set
- [ ] FR-9: Placeholder jobs exist with `if: false`
- [ ] FR-10: `.githooks/pre-commit` exists and is executable

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
