# Design: Story E01/005 -- CI/CD Pipeline

**Story:** [STORY.md](./STORY.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** Principal Architect Agent
**Risk Refs:** RISK-022 (GPLv3 dependency license compatibility), RISK-027 (CI pipeline performance and coverage gaps), RISK-034 (AI-agent development model unproven at this scale)

---

## Overview

This design implements the GitHub Actions CI workflow for Luminos, covering Stages 1-4 of the CI pipeline from doc-07 Section 4. The workflow runs on every push and pull request, enforcing code quality through three parallel-capable jobs: `lint`, `test-rust-unit`, and `security`. The `lint` job runs first as a gate; `test-rust-unit` and `security` run in parallel after `lint` passes. This structure provides granular failure feedback while minimizing total pipeline time.

The design uses `taiki-e/install-action` for tool installation (pre-built binaries, no compilation), `actions/cache` for Cargo registry and build artifact caching keyed on `Cargo.lock` hash, and environment variables (`CARGO_INCREMENTAL=0`, `RUSTFLAGS="--deny warnings"`) for deterministic CI builds. Placeholder jobs for shader tests (Stage 3) and integration tests (Stage 4) are included but disabled, ready for activation in E2 when GPU rendering and platform backends exist. An optional local pre-commit hook is also provided.

## Architecture

### Component Diagram

```
.github/
  workflows/
    ci.yml                    (main CI workflow)
.githooks/
  pre-commit                  (optional local pre-commit hook)

CI Job Dependency Chain:
+--------+
|  lint  |  (Stage 1: fmt, clippy, deny)
+--------+
    |
    +------> depends on lint passing
    |                    |
    v                    v
+----------------+  +-----------+
| test-rust-unit |  | security  |   (run in parallel)
| (Stage 2)      |  | (audit)   |
+----------------+  +-----------+
    |                    |
    v                    v
+-----------------+  +------------------+
| test-shaders    |  | test-integration |   (placeholders, disabled)
| (Stage 3, skip) |  | (Stage 4, skip)  |
+-----------------+  +------------------+
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `.github/workflows/ci.yml` | New | Main CI workflow file |
| `.githooks/pre-commit` | New | Optional local pre-commit script |

### Data Flow

```
Developer pushes code
    |
    v
GitHub webhook triggers ci.yml
    |
    v
[lint job] -- checkout -> cache restore -> rustup -> fmt check -> clippy -> deny check
    |                                                                         |
    | (pass)                                                                  | (fail: blocks merge)
    v                                                                         v
[test-rust-unit job] -- cache restore -> install nextest -> nextest run     STOP
    |
    v
[security job] -- cache restore -> install cargo-audit -> cargo audit
    |
    v
All jobs pass -> PR status check GREEN -> mergeable
```

## API Design

### GitHub Actions Workflow (.github/workflows/ci.yml)

```yaml
name: CI

on:
  push:
    branches: ["**"]
  pull_request:
    branches: [main]

env:
  CARGO_INCREMENTAL: "0"
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "--deny warnings"

jobs:
  lint:
    name: Lint & Format
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache Cargo registry and build artifacts
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: lint-${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            lint-${{ runner.os }}-cargo-

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run Clippy
        run: >
          cargo clippy
          --workspace --all-targets --all-features
          -- -D warnings
          -W clippy::unwrap_used
          -W clippy::expect_used
          -W clippy::pedantic
          -A clippy::module_name_repetitions

      - name: Install cargo-deny
        uses: taiki-e/install-action@cargo-deny

      - name: Check licenses and advisories
        run: cargo deny check licenses advisories

  test-rust-unit:
    name: Unit Tests (Rust)
    runs-on: ubuntu-latest
    needs: [lint]
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo registry and build artifacts
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: test-${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            test-${{ runner.os }}-cargo-

      - name: Install cargo-nextest
        uses: taiki-e/install-action@nextest

      - name: Run unit tests
        run: >
          cargo nextest run
          --profile ci
          --workspace
          --exclude luminos-app

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    needs: [lint]
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
          key: security-${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            security-${{ runner.os }}-cargo-

      - name: Install cargo-deny
        uses: taiki-e/install-action@cargo-deny

      - name: Check licenses and advisories
        run: cargo deny check licenses advisories

      - name: Install cargo-audit
        uses: taiki-e/install-action@cargo-audit

      - name: Run vulnerability audit
        run: cargo audit

  # --- Placeholder jobs (activated in E2+) ---

  test-shaders:
    name: Shader Tests (placeholder)
    runs-on: ubuntu-latest
    needs: [lint]
    if: false  # Disabled until E2 adds GPU rendering
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
      # TODO (E2): Install Mesa llvmpipe, set LIBGL_ALWAYS_SOFTWARE=1,
      # run cargo nextest --features ci_platform_tests -E 'test(~shader_)'

  test-integration:
    name: Integration Tests (placeholder)
    runs-on: ubuntu-latest
    needs: [test-rust-unit]
    if: false  # Disabled until E2+ adds platform backends
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
      # TODO (E2+): Install espeak-ng, Xvfb, Mesa, download Kokoro model,
      # run cargo nextest --features integration_tests,ci_platform_tests
```

### Pre-commit Hook (.githooks/pre-commit)

```bash
#!/usr/bin/env bash
# Optional pre-commit hook for local development.
# Install: git config core.hooksPath .githooks
#
# Runs fast checks only (fmt + clippy). Full lint suite runs in CI.

set -e

echo "==> Checking formatting..."
cargo fmt --all -- --check

echo "==> Running clippy..."
cargo clippy --workspace --all-targets --all-features -- \
    -D warnings \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -W clippy::pedantic \
    -A clippy::module_name_repetitions

echo "==> Pre-commit checks passed."
```

### Key Design Decisions

**Why `cargo deny check` runs in both `lint` and `security`:** The `lint` job catches license/advisory violations early as part of the fast-feedback loop. The `security` job provides a dedicated, named job for security concerns so that PR status checks clearly show whether a security issue exists. The duplication adds ~10s (the `deny` check is fast) but improves developer experience by making security status visible as a separate check.

**Why `lint` is a hard gate for `test-rust-unit` and `security`:** If code has formatting or linting errors, running tests is wasted compute. By gating on `lint`, we save 3-5 minutes of CI time on PRs with trivial formatting issues. The `lint` job itself takes ~1 minute, so the feedback loop remains fast.

**Why `--exclude luminos-app` in unit tests:** `luminos-app` depends on Tauri, which requires WebkitGTK system libraries not available by default on `ubuntu-latest`. Until E4 (Control Panel Foundation) adds Tauri CI setup, the app crate is excluded from unit tests. The app crate's `main.rs` is an empty `fn main() {}` in E1, so there is nothing to test.

**Why `taiki-e/install-action` instead of `cargo install`:** Pre-built binaries install in ~2s versus ~2-3 minutes for compilation from source. The action also handles version pinning and caching automatically.

## Error Handling

CI failures are communicated through GitHub Actions step exit codes and PR status checks:

- **Formatting failure:** `cargo fmt --check` exits non-zero; the `lint` job fails; the PR shows a red "Lint & Format" check with the specific diff in the step output.
- **Clippy warning:** `cargo clippy` with `-D warnings` exits non-zero; same failure path.
- **License violation:** `cargo deny check licenses` exits non-zero and outputs the offending crate name and license identifier.
- **Advisory violation:** `cargo deny check advisories` exits non-zero and outputs the advisory ID, crate name, and severity.
- **Test failure:** `cargo nextest` exits non-zero and outputs the failing test name, assertion message, and backtrace.
- **Vulnerability:** `cargo audit` exits non-zero and outputs the advisory ID, crate, and CVSS score.

The `lint` job uses sequential steps (fmt first, then clippy, then deny) so that the earliest failure is reported first. This is the "fail-fast within job" approach from NFR-2: if formatting fails, clippy does not run, giving the developer a clear first action.

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux (CI runner) | `ubuntu-latest` | All E1 CI jobs run on Ubuntu |
| macOS (CI runner) | Deferred to E2+ | macOS runners added when platform matrix stage is activated |
| Windows (CI runner) | Deferred to E2+ | Windows runners added when platform matrix stage is activated |
| OpenBSD (CI runner) | Deferred to E2+ | OpenBSD may require self-hosted runners (no GitHub-hosted OpenBSD) |

E1 CI runs on `ubuntu-latest` only. Platform matrix CI (running on macOS, Windows, and potentially OpenBSD) is a Stage 6 concern introduced in E2+ when platform backends exist to test.

## Testing Strategy

### Unit Tests

The CI pipeline itself is not unit-tested in the traditional sense. Its correctness is verified by execution: if the workflow runs and produces the expected pass/fail results, it is correct.

### Integration Tests

The CI pipeline is an integration test of the entire workspace. Its execution validates that:
- The workspace compiles
- Formatting rules are enforced
- Clippy lints are enforced
- License compliance is maintained
- Vulnerability scanning works
- Unit tests pass

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | CI execution | Open a PR against `main`; verify that `lint`, `test-rust-unit`, and `security` jobs appear in the GitHub Actions UI |
| AC-1.2 | CI execution (failure) | Introduce a formatting violation (e.g., extra whitespace), push to a PR branch; verify `lint` job fails with formatting diff in output |
| AC-1.3 | CI execution (failure) | Introduce a clippy warning (e.g., `let _ = vec.first().unwrap()`), push to a PR branch; verify `lint` job fails with clippy output |
| AC-1.4 | CI execution (success) | Push clean code to a PR branch; verify all three jobs pass and PR status check shows green |
| AC-2.1 | CI execution (success) | Verify `security` job passes on the initial workspace (all deps are in the allowlist) |
| AC-2.2 | CI execution (failure) | Add a dependency with an incompatible license (e.g., a test crate with SSPL); verify `security` job fails with crate name and license in output. Revert after verification. |
| AC-2.3 | CI execution (success) | Verify `cargo deny check advisories` step passes (no RustSec advisories on initial deps) |
| AC-3.1 | CI execution (success) | Verify `test-rust-unit` job runs `cargo nextest run --profile ci --workspace --exclude luminos-app` and succeeds |
| AC-3.2 | CI execution (failure) | Add a failing test to a library crate; push to PR branch; verify `test-rust-unit` job fails and output shows the test name |
| AC-3.3 | File inspection + CI | Verify `.config/nextest.toml` `ci` profile has `slow-timeout = { period = "60s", terminate-after = 3 }` and `retries = 2` |
| AC-4.1 | CI execution (success) | Verify `cargo audit` step in `security` job completes without failure |
| AC-4.2 | CI execution (advisory) | If a future dependency has a CVSS >= 7.0 advisory, verify the `security` job fails. (Not testable on initial workspace; documented as a future verification.) |
| AC-5.1 | CI execution (timing) | After second run on same branch (warm cache), measure total pipeline time; verify < 10 minutes |
| AC-5.2 | File inspection | Inspect `.github/workflows/ci.yml` job `needs` fields; verify `test-rust-unit` and `security` both `needs: [lint]` and have no dependency on each other |
| AC-5.3 | File inspection | Inspect `.github/workflows/ci.yml` cache configuration; verify `actions/cache@v4` caches `~/.cargo/registry`, `~/.cargo/git`, and `target/` with key containing `hashFiles('**/Cargo.lock')` |

**Testing failure scenarios:**

AC-1.2 and AC-1.3 require intentionally broken commits to verify CI catches violations. The implementing agent should:
1. Create a temporary branch
2. Introduce the violation
3. Push and verify the CI fails correctly
4. Force-push to remove the broken commit (or delete the branch)

AC-2.2 requires adding and then reverting an incompatible dependency. This should be done on a throw-away branch to avoid polluting `main`.

AC-4.2 cannot be directly tested on the initial workspace (which has no vulnerable deps). It is verified as a design property: `cargo audit` fails on any advisory by default. The `deny.toml` `[advisories]` section sets `vulnerability = "deny"`.

## Performance Targets

| Target | Source | Verification |
|--------|--------|-------------|
| Total pipeline < 10min (warm cache) | NFR-1 | Measured after second CI run on same branch |
| `lint` job < 3min | Derived from doc-07 Stage 1 (~1 min) | Observed in CI run timing |
| `test-rust-unit` job < 5min | Derived from doc-07 Stage 2 (~3 min) | Observed in CI run timing |
| `security` job < 3min | Derived from doc-07 | Observed in CI run timing |

**Cache effectiveness:** The `actions/cache` action with `Cargo.lock`-keyed caching typically reduces `cargo build` time from 3-5 minutes (cold) to 30-60 seconds (warm). This is the primary mechanism for achieving the 10-minute target.

## Security Considerations

- **RISK-022 (license compliance):** `cargo deny check licenses` in both `lint` and `security` jobs ensures no incompatible license enters the dependency tree. The allowlist-only approach (no `deny` list needed) means new licenses must be explicitly approved.
- **Advisory scanning:** `cargo deny check advisories` and `cargo audit` provide overlapping but complementary coverage. `cargo deny` checks advisories as part of its broader checks; `cargo audit` provides CVSS scoring and more detailed vulnerability information.
- **Supply chain:** The CI workflow pins GitHub Actions by major version (`@v4`). For maximum supply chain security, these could be pinned by SHA in a future hardening pass. The `taiki-e/install-action` downloads pre-built binaries from crate release assets, verified by the action's own integrity checks.
- **`CARGO_INCREMENTAL=0`:** Disabling incremental compilation in CI ensures deterministic builds and prevents stale incremental state from masking errors.

## Alternatives Considered

### Alternative: Single monolithic CI job

**Approach:** Run all checks (fmt, clippy, test, deny, audit) in a single job sequentially.

**Rejected because:**
- No granular feedback: if the job fails, the developer cannot tell from the PR status whether it was a formatting issue or a test failure without reading the full log.
- No parallelism: `test-rust-unit` and `security` are independent and can run in parallel after lint passes, saving 2-3 minutes versus sequential execution.
- The STORY.md specifically requires separate jobs with a defined dependency chain (AC-5.2).

### Alternative: cargo install instead of taiki-e/install-action

**Approach:** Install `cargo-nextest`, `cargo-deny`, and `cargo-audit` via `cargo install`.

**Rejected because:**
- `cargo install` compiles from source, adding 2-3 minutes per tool (6-9 minutes total).
- `taiki-e/install-action` downloads pre-built binaries in ~2s per tool, with automatic caching and version pinning.
- NFR-4 explicitly recommends `taiki-e/install-action` for speed and reproducibility.

### Alternative: Reusable workflow / composite actions

**Approach:** Extract common steps (checkout, toolchain install, cache) into reusable workflows or composite actions.

**Rejected because:**
- Premature abstraction for a three-job workflow. The duplication is minimal (checkout + toolchain + cache = 3 steps) and the readability benefit of self-contained job definitions outweighs the DRY violation.
- Reusable workflows add indirection that makes the CI harder to understand for new contributors and AI agents.
- Can be refactored in a future story if the workflow grows significantly (e.g., when platform matrix and TypeScript CI are added in E2/E4).
