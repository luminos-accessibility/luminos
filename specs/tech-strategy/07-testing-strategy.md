# 07 -- Testing Strategy

**Status:** DRAFT v1.1 (post audit review)
**Date:** 2026-03-17
**Audience:** Engineers, AI agents, CI/CD maintainers, contributors
**Source Documents:** [Product Strategy](../PRODUCT_STRATEGY.md) (v1.3, Section 9), [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) (FINAL), [System Architecture](./01-system-architecture.md) (Sections 7, 9, 10), [Platform Abstraction](./02-platform-abstraction.md) (Section 7), [Rendering Pipeline](./03-rendering-pipeline.md) (Section 12), [TTS Pipeline](./04-tts-pipeline.md) (Section 14), [Control Panel](./05-control-panel.md) (Section 13), [Cross-Cutting Concerns](./06-cross-cutting-concerns.md) (Sections 2, 5, 7)

---

## 1. Overview

### 1.1 Purpose

This document defines the testing strategy for Luminos: the test architecture, CI/CD pipeline, quality gates, and release verification process. It consolidates the per-subsystem testing approaches defined in docs 02-06 into a unified strategy and adds the infrastructure, tooling, and process layers that span all subsystems.

This document answers: **How do we verify that Luminos works correctly, performs well, and remains accessible -- continuously and across all platforms?**

### 1.2 Scope

This document covers:
- Test pyramid: classification of all test types, their roles, and where they run
- CI/CD pipeline architecture (GitHub Actions workflows, job structure, caching)
- Quality gates: what must pass before code merges and before releases ship
- CI hardware specifications and baseline hardware profile for benchmarks
- Performance regression detection methodology
- Test data and fixture management
- Local development testing workflow (pre-commit checks)
- Release checklist and manual verification steps
- Phase-by-phase test infrastructure rollout

This document does NOT cover:
- Per-subsystem test case inventories (see [02](./02-platform-abstraction.md) Section 7, [03](./03-rendering-pipeline.md) Section 12, [04](./04-tts-pipeline.md) Section 14, [05](./05-control-panel.md) Section 13)
- Build and packaging specifics (see [08 -- Build and Distribution](./08-build-and-distribution.md))
- Error handling strategy (see [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 7)

### 1.3 Design Principles

1. **Fast feedback first.** The most common tests (unit, lint) run in seconds. Developers get signal before pushing. Slow tests (integration, benchmarks) run in CI but never block the inner development loop.
2. **Mock by default, integrate on purpose.** Unit tests use mock backends for all six platform traits. Integration tests that require real platform APIs are explicit opt-in and run only on their target platform's CI runner.
3. **Every test traces to a requirement.** Spec-Driven Development (SDD) requires every test to trace to an acceptance criterion in a STORY.md. This document defines the infrastructure; individual stories define which tests exist.
4. **AI agents can run the full test suite.** Test commands are deterministic, require no interactive input, and produce machine-parseable output. AI agents execute `cargo nextest run` and `pnpm test` as part of their TDD workflow.
5. **No flaky tests.** Tests that depend on timing, network, or display state are either deterministic (mocked) or explicitly isolated. A flaky test is a bug with the same priority as a code bug.

### 1.4 Relationship to SDD Methodology

The Spec-Driven Development methodology ([specs/README.md](../README.md)) requires:
- Every acceptance criterion in STORY.md maps to at least one test
- DESIGN.md includes a testing strategy section mapping ACs to test types
- SUBTASKS.md follows TDD: red (failing test) -> green (implementation) -> refactor

This testing strategy document defines the test infrastructure that stories execute against. It does not define individual test cases -- those are defined in each story's DESIGN.md.

---

## 2. Test Pyramid

### 2.1 Classification

Luminos tests are organized in four tiers, from fastest/most numerous to slowest/fewest:

```
                    /\
                   /  \
                  / E2E \          Tier 4: End-to-End        (~5 tests)
                 /--------\
                / Integr.   \      Tier 3: Integration       (~50 tests)
               /--------------\
              /   Component     \  Tier 2: Component/Shader  (~100 tests)
             /--------------------\
            /       Unit            \  Tier 1: Unit           (~500+ tests)
           /--------------------------\
```

| Tier | Type | Language | Runner | Typical Duration | Dependencies |
|------|------|----------|--------|-----------------|--------------|
| 1 | Unit | Rust | `cargo nextest` | < 0.1s each | None (mocked) |
| 1 | Unit | TypeScript | `vitest` | < 0.1s each | JSDOM (mocked IPC) |
| 2 | Component | TypeScript | `vitest` + React Testing Library | < 0.5s each | JSDOM, mocked IPC |
| 2 | Shader | Rust | `cargo nextest` | < 2s each | wgpu headless (GL backend) |
| 3 | Integration | Rust | `cargo nextest --features integration_tests` | < 10s each | Platform APIs, espeak-ng, model files |
| 3 | IPC Integration | TypeScript | `vitest` + `tauri-driver` | < 5s each | Full Tauri process |
| 4 | End-to-End | Rust + TypeScript | Custom harness | < 30s each | Full application, display server, GPU |

### 2.2 Tier 1: Unit Tests

**Rust unit tests** use mock implementations of all six platform traits (defined in [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 7.1). They test:
- Pure logic: viewport calculation, smooth panning, frame limiting, sentence segmentation, text preprocessing, number normalization, settings validation, profile serialization, keybinding conflict detection
- State machines: TTS Coordinator state transitions, degradation level transitions
- Data structures: `FrameTimings` circular buffer, ring buffer, model manifest parsing
- Error paths: every `LuminosError` variant construction and `Display` formatting

**TypeScript unit tests** use Vitest ([05 -- Control Panel](./05-control-panel.md) Section 13.1) and test:
- Zod schema validation (accepts valid data, rejects invalid data for every type)
- Zustand store logic (hydration, optimistic updates, reverts)
- IPC command wrapper functions (correct delegation to `tauri-specta` bindings)
- Default settings validity (compiled-in defaults pass schema validation)

### 2.3 Tier 2: Component and Shader Tests

**React component tests** use React Testing Library with `@tauri-apps/api/mocks` to mock IPC calls. They test:
- Component rendering with given state
- User interaction (slider drag, button click, keyboard navigation)
- Optimistic update + IPC call + error revert pattern
- Accessibility: every component page runs through `axe-core` for automated a11y checks

**WGSL shader tests** use wgpu headless rendering ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 12.2). They test:
- Known-input-to-known-output pixel comparisons (screenshot comparison with tolerance)
- Identity transforms (1x zoom = pixel-exact copy)
- Color filter correctness (inversion, grayscale, high-contrast)
- Cursor overlay positioning

Shader tests require a wgpu-compatible backend. In CI, they use `wgpu::Backends::GL` (software rendering via Mesa llvmpipe or swiftshader). Locally, they use the system's native GPU backend.

### 2.4 Tier 3: Integration Tests

Integration tests exercise real subsystem boundaries:

| Test Suite | Feature Gate | Dependencies | What It Tests |
|------------|-------------|--------------|---------------|
| Platform capture | `ci_platform_tests` | Display server (Xvfb, real display) | xcap captures real screen pixels |
| espeak-ng protocol | `integration_tests` | espeak-ng binary installed | Subprocess spawn, phonemize, crash recovery |
| TTS full pipeline | `integration_tests` | espeak-ng + Kokoro q4 model (~80MB) | Text -> phonemes -> inference -> audio samples |
| IPC roundtrip | `integration_tests` | Full Tauri process (`tauri-driver`) | TypeScript command -> Rust handler -> response -> Zod validation |
| IPC type compatibility | `integration_tests` | Full Tauri process (`tauri-driver`) | Every Zod schema accepts actual engine response |
| Settings persistence | `integration_tests` | File system (temp dir) | Save -> reload -> compare settings roundtrip |
| Profile import/export | `integration_tests` | File system (temp dir) | Export -> import -> compare profile roundtrip |
| Render pipeline | `ci_platform_tests` | wgpu (headless GL) | Mock capture -> GPU upload -> shader -> readback -> pixel comparison |

**Feature gates** prevent integration tests from running during `cargo nextest run` (the default invocation). Developers run them explicitly:

```bash
# Run all integration tests (requires espeak-ng, model files, display server)
cargo nextest run --features integration_tests,ci_platform_tests

# Run only TTS integration tests
cargo nextest run --features integration_tests tts_pipeline_integration_
```

### 2.5 Tier 4: End-to-End Tests

End-to-end tests verify the full application from a user's perspective. They are few in number and run only in CI on platform-specific runners:

| Test | Platform | Method | Verifies |
|------|----------|--------|----------|
| Application starts and magnifies | Linux X11 | Launch binary on Xvfb; assert overlay window appears | Startup sequence completes in < 2s |
| Control panel opens and hydrates | Linux X11 | Launch binary; WebDriver connects to Tauri webview; assert zoom slider rendered | IPC hydration works end-to-end |
| Zoom in/out via hotkey | Linux X11 | Launch binary; simulate keypress via `xdotool`; assert zoom level changed | Input monitoring -> state change -> render thread |
| TTS speaks text | Linux X11 | Launch binary; trigger speak via IPC; assert audio samples produced | Full TTS pipeline produces output |
| Settings persist across restart | Linux X11 | Launch, change setting, quit, relaunch, assert setting persists | Config save/load roundtrip |

E2E tests are expensive (~30s each) and fragile (timing-dependent). They are run:
- On every push to `main`
- On every PR that modifies startup, IPC, or platform code
- Not on every commit to feature branches (too slow)

---

## 3. Test Toolchain

### 3.1 Rust Test Tools

| Tool | Purpose | Version | Notes |
|------|---------|---------|-------|
| `cargo-nextest` | Test runner | Latest | Parallel execution, structured output, per-test timeout, retry support |
| `cargo-clippy` | Linter | Bundled with rustup | Custom lint configuration (Section 5.3) |
| `cargo-audit` | Vulnerability scanner | Latest | CVSS >= 7.0 fails CI |
| `cargo-deny` | License + advisory checker | Latest | License allowlist enforcement |
| `cargo-llvm-cov` | Code coverage | Latest | LLVM source-based coverage; lcov output for CI |
| `cargo-bench` (built-in) | Benchmark runner | Stable | Performance regression benchmarks |
| `criterion` | Statistical benchmarks | Latest | Detailed statistical analysis with regression detection |

**Why `cargo-nextest` over `cargo test`:**
- Per-test process isolation prevents shared mutable state leaks between tests
- Structured JSON output for CI parsing
- Built-in retry for transient failures (useful for platform integration tests)
- Configurable per-test timeouts (important for subprocess tests that might hang)
- Parallel test execution with configurable concurrency

**`nextest` configuration** (`.config/nextest.toml`):

```toml
[store]
dir = "target/nextest"

[profile.default]
retries = 0
slow-timeout = { period = "30s", terminate-after = 2 }
fail-fast = true

[profile.ci]
retries = 2
slow-timeout = { period = "60s", terminate-after = 3 }
fail-fast = false

# Platform integration tests get longer timeouts
[[profile.ci.overrides]]
filter = "test(~platform_integration_)"
slow-timeout = { period = "120s", terminate-after = 2 }

# TTS integration tests need even more time (model loading)
[[profile.ci.overrides]]
filter = "test(~tts_pipeline_integration_)"
slow-timeout = { period = "180s", terminate-after = 2 }
```

### 3.2 TypeScript Test Tools

The base test toolchain (Vitest, React Testing Library, `@tauri-apps/api/mocks`, zod) is defined in [05 -- Control Panel](./05-control-panel.md) Section 13.1. The following table includes those tools plus additional tools introduced in this document (marked with *):

| Tool | Purpose | Version | Notes |
|------|---------|---------|-------|
| Vitest | Test runner | Latest | Fast, native ESM, Vite-integrated |
| React Testing Library | Component test utilities | Latest | DOM-based component testing |
| `@testing-library/user-event` * | Interaction simulation | Latest | Realistic user event simulation |
| `@tauri-apps/api/mocks` | IPC mocking | Latest (matches Tauri 2.0) | Mock `invoke()` and `listen()` in tests |
| `axe-core` | Accessibility checker | Latest | Automated WCAG 2.1 AA violation detection |
| `vitest-axe` * | axe + Vitest integration | Latest | `expect(container).toHaveNoViolations()` matcher |
| `c8` / `v8` coverage * | Code coverage | Bundled with Vitest | v8 provider for coverage reporting |
| `eslint-plugin-jsx-a11y` * | JSX accessibility linting | Latest | Static analysis for WCAG + ARIA best practices |

**Vitest configuration** (`ui/vitest.config.ts`):

```typescript
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: ['src/ipc/bindings.ts', 'src/test/**'],  // Exclude auto-generated files
    },
    // Tauri-specific: @tauri-apps/api/mocks patches invoke/listen.
    // The setup file (src/test/setup.ts) imports and initializes the mock layer.
  },
});
```

### 3.3 Shared Infrastructure

| Tool | Purpose | Phase |
|------|---------|-------|
| GitHub Actions | CI/CD orchestration | Phase 0 |
| `tauri-driver` | WebDriver bridge for Tauri integration/E2E tests (Linux + Windows only; macOS not supported due to missing WKWebView driver) | Phase 0 |
| Xvfb | Headless X11 display server for Linux CI | Phase 0 |
| Mesa llvmpipe | Software OpenGL renderer for shader tests in CI | Phase 0 |
| `xdotool` | Keyboard/mouse simulation for E2E tests on X11 | Phase 0 |

---

## 4. CI/CD Pipeline Architecture

### 4.1 Pipeline Overview

The CI/CD pipeline runs on GitHub Actions. It is structured as a sequence of stages with increasing cost and decreasing frequency. Early stages provide fast feedback; later stages provide deeper validation.

```
Push / PR
  |
  v
[Stage 1: Lint & Format]  ~1 min        -- Every push, every PR
  |
  v
[Stage 2: Unit Tests]     ~3 min        -- Every push, every PR
  |
  v
[Stage 3: Component Tests] ~3 min       -- Every push, every PR
  |
  v
[Stage 4: Integration]    ~10 min       -- Every PR, every push to main
  |
  v
[Stage 5: Benchmarks]     ~5 min        -- Every push to main, nightly
  |
  v
[Stage 6: Platform Matrix] ~15 min      -- Every PR, every push to main
  |
  v
[Stage 7: E2E Tests]      ~5 min        -- Every push to main, PRs touching startup/IPC/platform
  |
  v
[Stage 8: Release Build]  ~20 min       -- Tags only (vX.Y.Z)
```

Stages 1-3 run on every push (including feature branch commits). Stages 4-7 run on PRs and main. Stage 8 runs only on release tags.

### 4.2 Stage 1: Lint & Format

**Job: `lint`** (runs on `ubuntu-latest`)

```yaml
steps:
  # Rust checks
  - cargo fmt --all -- --check
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
    -W clippy::unwrap_used -W clippy::expect_used
    -W clippy::pedantic -A clippy::module_name_repetitions
  - cargo deny check licenses advisories

  # TypeScript checks
  - pnpm --dir ui lint          # ESLint + Prettier check
  - pnpm --dir ui typecheck     # tsc --noEmit
```

**Clippy configuration** (`.clippy.toml`):

```toml
cognitive-complexity-threshold = 25
too-many-arguments-threshold = 7
type-complexity-threshold = 250
```

**Key clippy lints enforced:**
- `clippy::unwrap_used` and `clippy::expect_used` -- no panicking in production code (see [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 7.4)
- `clippy::pedantic` -- catches common anti-patterns
- `clippy::module_name_repetitions` -- allowed (common in trait-per-file layouts)

**`cargo deny` configuration** (`deny.toml`):

```toml
[licenses]
allow = [
  "MIT",
  "Apache-2.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "Zlib",
  "MPL-2.0",
  "GPL-3.0-only",
  "GPL-3.0-or-later",
  "LGPL-2.1-only",
  "LGPL-2.1-or-later",
  "LGPL-3.0-only",
  "LGPL-3.0-or-later",
  "Unicode-3.0",
  "Unicode-DFS-2016",
]
confidence-threshold = 0.8
```

Any license not in the allowlist is automatically denied. There is no separate deny list -- omission from the allowlist is denial (see [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 4.2).

### 4.3 Stage 2: Unit Tests

**Job: `test-rust-unit`** (runs on `ubuntu-latest`)

```yaml
steps:
  - cargo nextest run --profile ci --workspace
    --exclude luminos-app  # App crate requires Tauri setup; tested in integration
```

**Job: `test-ts-unit`** (runs on `ubuntu-latest`)

```yaml
steps:
  - pnpm --dir ui test -- --reporter=verbose
```

Unit tests use no feature flags -- they run against mock backends only.

### 4.4 Stage 3: Component and Shader Tests

**Job: `test-ts-components`** (runs on `ubuntu-latest`)

```yaml
steps:
  - pnpm --dir ui test -- --reporter=verbose
    # Component tests run alongside unit tests; Vitest discovers all *.test.tsx files
```

**Job: `test-shaders`** (runs on `ubuntu-latest`)

```yaml
env:
  MESA_GL_VERSION_OVERRIDE: "4.5"
  LIBGL_ALWAYS_SOFTWARE: "1"

steps:
  - sudo apt-get install -y mesa-utils libegl1-mesa-dev libgl1-mesa-dri
  - cargo nextest run --profile ci --features ci_platform_tests
    -E 'test(~shader_)'
```

Shader tests use Mesa llvmpipe (software OpenGL) in CI. The `LIBGL_ALWAYS_SOFTWARE=1` environment variable forces software rendering, and `MESA_GL_VERSION_OVERRIDE=4.5` ensures sufficient GL version for wgpu's GL backend. This produces deterministic pixel output independent of GPU hardware.

### 4.5 Stage 4: Integration Tests

**Job: `test-integration`** (runs on `ubuntu-latest`)

```yaml
services:
  xvfb:
    # Xvfb is started before tests

env:
  DISPLAY: ":99"

steps:
  - sudo apt-get install -y espeak-ng xvfb mesa-utils libegl1-mesa-dev
  - Xvfb :99 -screen 0 1920x1080x24 &

  # Download Kokoro q4 model for TTS integration tests
  - name: Cache Kokoro model
    uses: actions/cache@v4
    with:
      path: test-fixtures/models/kokoro-q4.onnx
      key: kokoro-q4-v1

  - cargo nextest run --profile ci
    --features integration_tests,ci_platform_tests
    -E 'test(~integration_)'
```

**Job: `test-ipc-integration`** (runs on `ubuntu-latest`)

```yaml
steps:
  - sudo apt-get install -y webkit2gtk-4.1 xvfb espeak-ng
  - Xvfb :99 -screen 0 1920x1080x24 &
  - cargo install tauri-driver
  - cargo build --release -p luminos-app

  # Run IPC integration tests against the built binary
  - pnpm --dir ui test:integration
```

### 4.6 Stage 5: Performance Benchmarks

**Job: `benchmark`** (runs on dedicated `self-hosted` runner or `ubuntu-latest`)

```yaml
steps:
  - sudo apt-get install -y espeak-ng xvfb mesa-utils libegl1-mesa-dev
  - Xvfb :99 -screen 0 1920x1080x24 &

  - cargo bench --workspace -- --output-format bencher
    | tee target/benchmark-results.txt

  # Parse results and compare against baseline
  - name: Check benchmark regression
    run: npx tsx scripts/check-benchmarks.ts target/benchmark-results.txt

  # Additional non-bench checks
  - name: Check binary size
    run: |
      cargo build --release -p luminos-app
      SIZE=$(stat -f%z target/release/luminos 2>/dev/null || stat -c%s target/release/luminos)
      echo "Binary size: $SIZE bytes"
      if [ "$SIZE" -gt 62914560 ]; then echo "FAIL: binary > 60MB" && exit 1; fi
      if [ "$SIZE" -gt 52428800 ]; then echo "WARN: binary > 50MB"; fi

  - name: Check memory high-water mark
    run: |
      cargo build --release -p luminos-app --features integration_tests
      # --benchmark-mode: starts Luminos in headless mode with mock capture,
      # renders N frames, then exits. Defined in a future implementation story.
      target/release/luminos --benchmark-mode &
      PID=$!
      sleep 10
      PEAK=$(grep VmPeak /proc/$PID/status | awk '{print $2}')
      kill $PID
      echo "Peak RSS: ${PEAK}KB"
      if [ "$PEAK" -gt 1048576 ]; then echo "FAIL: peak RSS > 1GB" && exit 1; fi
      if [ "$PEAK" -gt 819200 ]; then echo "WARN: peak RSS > 800MB"; fi
```

**Benchmark thresholds** (fail thresholds from [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 2.4; warn thresholds marked with * are new in this document):

| Benchmark | Metric | Warn | Fail |
|-----------|--------|------|------|
| Frame time | P99 | > 16.67ms * | > 20ms |
| Memory high-water mark | Peak RSS | > 800MB * | > 1GB |
| Binary size | Release binary (excl. models) | > 50MB | > 60MB |
| Startup time | Cold start to first frame | > 2s | > 3s |
| TTS latency | Trigger-to-first-audio P99 | > 200ms | > 300ms |

*\* Frame time warn at 16.67ms (the 60fps budget) and memory warn at 800MB are early-warning thresholds introduced here. Doc-06 defines only the fail thresholds for these metrics.*

**Regression detection:** Benchmark results from main are stored as JSON artifacts. Each PR compares against the last main baseline. A > 10% regression on any metric triggers a warning annotation on the PR. A > 20% regression or a threshold violation fails the check.

### 4.7 Stage 6: Platform Matrix

**Job: `platform-tests`** (matrix strategy)

```yaml
strategy:
  fail-fast: false
  matrix:
    include:
      - os: ubuntu-latest
        platform: linux-x11
        features: "integration_tests,ci_platform_tests"
        setup: |
          sudo apt-get install -y espeak-ng xvfb mesa-utils libegl1-mesa-dev libgl1-mesa-dri webkit2gtk-4.1
          Xvfb :99 -screen 0 1920x1080x24 &
        env:
          DISPLAY: ":99"
          LIBGL_ALWAYS_SOFTWARE: "1"

      - os: macos-latest
        platform: macos
        features: "integration_tests,ci_platform_tests"
        setup: |
          brew install espeak-ng
        env: {}

      - os: windows-latest
        platform: windows
        features: "integration_tests,ci_platform_tests"
        setup: |
          # espeak-ng Windows install: use MSI installer from GitHub releases.
          # Chocolatey package may not exist; fall back to direct download.
          curl -L -o espeak-ng.msi https://github.com/espeak-ng/espeak-ng/releases/download/1.51/espeak-ng-X64.msi
          msiexec /i espeak-ng.msi /quiet
        env: {}

      # OpenBSD: self-hosted runner (Phase 1+)
      # - os: self-hosted-openbsd
      #   platform: openbsd
      #   features: "integration_tests,ci_platform_tests"

steps:
  - name: Platform setup
    run: ${{ matrix.setup }}

  - name: Run platform tests
    env: ${{ matrix.env }}
    run: cargo nextest run --profile ci --features ${{ matrix.features }}
```

**Platform CI notes:**
- **Linux X11**: Full test suite runs under Xvfb. Shader tests use Mesa llvmpipe.
- **Linux Wayland**: Deferred to Phase 1 when Wayland backend is implemented. Will require a headless Wayland compositor (`weston --headless` or `cage`). Note: doc-02 Section 7.3 uses the shorthand "wlheadless" for this setup; the actual tools are `weston --headless` or `cage`.
- **macOS**: Integration tests that require screen capture (xcap/ScreenCaptureKit) need Screen Recording permission, which is NOT automatically granted on GitHub Actions macOS runners (known limitation; see GitHub issue actions/runner-images#8951). Options: (a) skip capture-dependent integration tests on macOS CI, running only unit tests with mock backends (Phase 0 approach); (b) use a self-hosted macOS runner with pre-granted permissions (Phase 2+); (c) accept that macOS capture tests are manual-only until a CI solution exists. `tauri-driver` is also not supported on macOS (no WKWebView driver tool), so IPC integration tests using WebDriver run only on Linux and Windows CI.
- **OpenBSD**: No GitHub-hosted runners exist. A self-hosted runner is provisioned in Phase 1. Until then, OpenBSD builds are validated manually.
- **Windows**: DXGI capture tests require a virtual display adapter. GitHub Actions Windows runners provide a software display.

### 4.8 Stage 7: End-to-End Tests

**Job: `e2e-tests`** (runs on `ubuntu-latest` with Xvfb)

```yaml
steps:
  - sudo apt-get install -y espeak-ng xvfb mesa-utils webkit2gtk-4.1 xdotool
  - Xvfb :99 -screen 0 1920x1080x24 &
  - cargo build --release -p luminos-app

  - name: Run E2E tests
    env:
      DISPLAY: ":99"
      LUMINOS_E2E: "1"
    run: cargo nextest run --profile ci --features e2e_tests
      -E 'test(~e2e_)'
```

E2E tests launch the full Luminos binary, interact with it via hotkeys (`xdotool`), WebDriver (for the control panel), and IPC, then assert outcomes. They are gated behind the `e2e_tests` feature flag.

**E2E test timeout policy:** Each E2E test has a 60-second hard timeout. If Luminos fails to start, produce a frame, or respond to IPC within this window, the test fails.

**E2E flakiness management:** E2E tests are inherently more fragile than unit tests due to dependence on timing, display servers, and full application startup. To manage this:
- E2E tests in CI use `nextest` retries (2 retries in the `ci` profile) to absorb transient failures.
- A test that fails intermittently (> 1 failure per 10 runs) is quarantined: moved behind a `flaky_e2e` feature flag and investigated within one sprint. Quarantined tests do not block merges.
- A quarantined test that is not fixed within two sprints is either redesigned to be deterministic or deleted.

### 4.9 Stage 8: Release Build

Release builds run only on version tags (`v*.*.*`). They produce signed release artifacts for all platforms. The full specification is in [08 -- Build and Distribution](./08-build-and-distribution.md). From a testing perspective, the release stage:

1. Runs all Stage 1-7 checks (full validation)
2. Builds optimized release binaries for each platform
3. Runs the E2E smoke test suite against the release binary (not the debug binary)
4. Signs binaries (see [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 3.7)
5. Generates SBOM in CycloneDX format (Phase 1)
6. Publishes release artifacts to GitHub Releases

---

## 5. Quality Gates

### 5.1 Pull Request Gate (Required to Merge)

Every PR must pass all of the following before merging:

| Check | Tool | Threshold | Rationale |
|-------|------|-----------|-----------|
| Rust formatting | `cargo fmt` | Zero diff | Consistent style; eliminates formatting debates |
| Rust lints | `cargo clippy` | Zero warnings (with `-D warnings`) | Catches anti-patterns; enforces no-unwrap policy |
| License compliance | `cargo deny` | Zero violations | GPLv3 compatibility; no surprise license obligations |
| Vulnerability scan | `cargo audit` | Zero CVSS >= 7.0 | No known high-severity vulnerabilities |
| Rust unit tests | `cargo nextest` | 100% pass | Core logic correctness |
| TypeScript lint | `eslint` + `prettier` | Zero errors | Consistent frontend style |
| TypeScript typecheck | `tsc --noEmit` | Zero errors | Type safety |
| TypeScript unit tests | `vitest` | 100% pass | Frontend logic correctness |
| Component tests | `vitest` + RTL | 100% pass | UI interaction correctness |
| Accessibility checks | `axe-core` | Zero violations | WCAG 2.1 AA compliance for control panel |
| Shader tests | `cargo nextest` (GL) | 100% pass | GPU pipeline correctness |
| Platform matrix | `cargo nextest` per platform | 100% pass on all active platforms | Cross-platform correctness |
| npm audit | `pnpm audit` | Zero high-severity | Frontend supply chain security |

### 5.2 Main Branch Gate (Post-Merge)

After merging to main, additional checks run:

| Check | Tool | Threshold |
|-------|------|-----------|
| Integration tests | `cargo nextest --features integration_tests` | 100% pass |
| E2E smoke tests | Custom harness | 100% pass |
| Performance benchmarks | `cargo bench` + custom script | No threshold violations (Section 4.6) |

If any post-merge check fails, the team is notified immediately. The failing commit is investigated within 24 hours. If the regression is confirmed, it is either fixed forward (preferred) or reverted.

### 5.3 Release Gate (Required to Ship)

Before any release (including pre-releases), all of the following must be verified:

| Check | Type | Verification |
|-------|------|-------------|
| All CI stages green | Automated | Stages 1-7 pass on the release tag |
| Release binary smoke test | Automated | E2E tests pass against the release-optimized binary |
| Binary size check | Automated | < 50MB (warn), < 60MB (fail) |
| Manual keyboard navigation | Manual | Every control panel page navigable via keyboard only |
| Manual screen reader test (Orca) | Manual | Control panel usable with Orca on Linux (Section 8) |
| Performance spot-check | Manual | 60fps on reference hardware with default settings |
| Release notes reviewed | Manual | Changelog lists all user-visible changes |
| SBOM generated | Automated (Phase 1) | CycloneDX SBOM attached to release |
| Signatures valid | Automated | `gpg --verify` passes on all Linux artifacts |

### 5.4 Coverage Policy

Code coverage is tracked but not gated. The project uses coverage as a **diagnostic tool**, not a quality metric.

| Metric | Target | Enforcement |
|--------|--------|-------------|
| Rust line coverage | > 70% for `luminos-core`, `luminos-platform` | Tracked; not blocking |
| Rust line coverage | > 50% for `luminos-gpu` (GPU code is hard to cover) | Tracked; not blocking |
| TypeScript line coverage | > 80% for `ui/src/` (excluding `bindings.ts`) | Tracked; not blocking |
| Branch coverage | Informational | Not tracked as gate |

**Why no coverage gate:** Coverage numbers are easy to game and do not correlate well with test quality for an application of this nature. GPU shader tests cover few "lines" but validate critical behavior. A 90% coverage number with poor test assertions is worse than 60% with meaningful property checks. The SDD methodology (every AC maps to a test) provides better quality assurance than a coverage percentage.

**Coverage trend monitoring:** A > 10% drop in coverage on any crate between consecutive releases is flagged for review, even though coverage is not a merge gate. This catches large swaths of untested code being added without corresponding tests.

Coverage reports are generated in CI using `cargo-llvm-cov` (Rust) and Vitest's v8 provider (TypeScript). Reports are uploaded as CI artifacts for developer review and published to a coverage dashboard (Phase 1).

```bash
# Generate Rust coverage report
cargo llvm-cov nextest --workspace --lcov --output-path target/lcov.info

# Generate TypeScript coverage report
pnpm --dir ui test -- --coverage
```

---

## 6. CI Hardware and Baseline Profile

### 6.1 CI Runner Specifications

| Runner | Provider | CPU | RAM | GPU | Display | Use |
|--------|----------|-----|-----|-----|---------|-----|
| `ubuntu-latest` | GitHub-hosted | 4-core x86_64 | 16GB | Mesa llvmpipe (software) | Xvfb 1920x1080x24 | Stages 1-5, 7 |
| `macos-latest` | GitHub-hosted | Apple Silicon (M1+) | 7GB | Apple GPU (Metal) | macOS display | Stage 6 (macOS) |
| `windows-latest` | GitHub-hosted | 4-core x86_64 | 16GB | Software adapter | Virtual display | Stage 6 (Windows) |
| `self-hosted-benchmark` | Self-hosted (Phase 1) | Defined in Section 6.2 | 16GB | Intel UHD 770 | Xvfb or real | Stage 5 (production benchmarks) |
| `self-hosted-openbsd` | Self-hosted (Phase 1) | x86_64 | 8GB | Intel (X11) | X11 | Stage 6 (OpenBSD) |

### 6.2 Benchmark Baseline Hardware

Performance benchmarks must run on consistent hardware to detect regressions reliably. GitHub-hosted runners have variable performance (shared infrastructure), so absolute benchmark results on hosted runners are used only for threshold checks (fail/pass), not for regression trending.

**Phase 0 (GitHub-hosted runners only):** Benchmarks run on `ubuntu-latest`. Absolute thresholds (P99 < 20ms, memory < 1GB) are enforced. Regression trending is not reliable due to runner variability.

**Phase 1+ (self-hosted benchmark runner):**

| Component | Specification | Rationale |
|-----------|--------------|-----------|
| CPU | Intel Core i5-12400 (6P cores) or equivalent | Mid-range desktop; representative of target users |
| GPU | Intel UHD Graphics 770 (integrated) | Worst-case target GPU; if benchmarks pass here, they pass on discrete GPUs |
| RAM | 16GB DDR4-3200 | 8GB used by system; 8GB available for Luminos + test harness |
| Storage | NVMe SSD | Eliminates I/O variance in startup benchmarks |
| OS | Ubuntu 24.04 LTS + X11 session | Primary target platform |
| Display | Xvfb at 1920x1080x24 | Consistent resolution for capture benchmarks |

This hardware profile is deliberately modest -- it represents the lower end of the target user base. If Luminos achieves 60fps on integrated Intel UHD graphics, it will perform well on any modern GPU.

### 6.3 Benchmark Data Management

Benchmark results are stored as JSON artifacts attached to each CI run. A TypeScript script (`scripts/check-benchmarks.ts`) run via `npx tsx` compares the current run against the last main baseline:

```json
{
  "timestamp": "2026-03-17T12:00:00Z",
  "commit": "abc123",
  "runner": "ubuntu-latest",
  "benchmarks": {
    "frame_time_p99_ms": 8.5,
    "memory_peak_kb": 512000,
    "binary_size_bytes": 42000000,
    "startup_time_ms": 1200,
    "tts_latency_p99_ms": 150
  }
}
```

**Retention:** Benchmark artifacts are retained for 90 days. Historical data is aggregated into a trend file (`benchmarks/history.json`) committed to the repository monthly.

---

## 7. Test Data and Fixture Management

### 7.1 Fixture Strategy

Test fixtures are managed differently based on size and sensitivity:

| Fixture Type | Location | Size | VCS | Example |
|-------------|----------|------|-----|---------|
| Rust test generators | Same module as type definition | Tiny | Committed | `generate_test_capture_frame()` |
| Small reference images | `tests/fixtures/images/` | < 1MB total | Committed | Shader comparison reference PNGs |
| Kokoro model (q4) | Downloaded in CI, cached | ~80MB | Not committed; CI-cached | TTS integration tests |
| Large test datasets | N/A | N/A | N/A | Not needed for Phase 0-2 |

### 7.2 Rust Test Generators

All Rust test data generators follow the conventions defined in CLAUDE.md and [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 7.1:

- **Prefix:** `generate_test_` (e.g., `generate_test_capture_frame`, `generate_test_display_info`, `generate_test_espeak_mock`)
- **Location:** Co-located with the type they generate, in the same module
- **Gate:** `#[cfg(any(test, feature = "test_utils"))]`
- **Visibility:** `pub` for cross-crate reuse
- **Parameterizable:** Accept arguments for customization; return sensible defaults when called with minimal args
- **Error injection:** Mock structs use error factory closures (`Box<dyn Fn() -> ErrorType + Send + Sync>`) rather than stored error values, because most error types are not `Clone`

**Canonical generator inventory** (from docs 02-04):

| Generator | Crate | Produces |
|-----------|-------|----------|
| `generate_test_display_info` | luminos-platform | `DisplayInfo` with given ID, dimensions, primary flag |
| `generate_test_capture_frame` | luminos-platform | `CaptureFrame` with solid-color pixels |
| `generate_test_checkerboard_frame` | luminos-gpu | `CaptureFrame` with checkerboard pattern |
| `generate_test_mock_screen_capture` | luminos-platform | `MockScreenCapture` with fixed frame response |
| `generate_test_mock_focus_tracker` | luminos-platform | `MockFocusTracker` |
| `generate_test_mock_tts_engine` | luminos-platform | `MockTtsEngine` |
| `generate_test_mock_window_manager` | luminos-platform | `MockWindowManager` |
| `generate_test_mock_input_monitor` | luminos-platform | `MockInputMonitor` |
| `generate_test_mock_audio_output` | luminos-platform | `MockAudioOutput` |
| `generate_test_espeak_mock` | luminos-tts | `MockEspeakSubprocess` with predetermined phonemes |
| `generate_test_inference_mock` | luminos-tts | `MockSherpaInference` that returns silence |

### 7.3 Reference Image Management

Shader screenshot comparison tests ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 12.2) compare GPU output against reference images. These images must be deterministic:

- **Generated on Mesa llvmpipe** (software rendering) to ensure reproducibility
- **Stored as PNG** in `tests/fixtures/images/shader_refs/`
- **Tolerance threshold:** Maximum per-channel difference of 2/255 to accommodate floating-point variance between GPU implementations
- **Update workflow:** When a shader changes, run `cargo nextest run --features update_refs shader_` to regenerate reference images, then review the diff before committing

```rust
#[cfg(all(test, feature = "ci_platform_tests"))]
fn assert_image_matches_reference(
    actual: &[u8],
    reference_path: &str,
    tolerance: u8,
) {
    let reference = image::open(reference_path)
        .expect("reference image not found")
        .to_rgba8();

    assert_eq!(actual.len(), reference.as_raw().len(), "image dimensions mismatch");

    for (i, (a, r)) in actual.iter().zip(reference.as_raw().iter()).enumerate() {
        let diff = (*a as i16 - *r as i16).unsigned_abs() as u8;
        assert!(
            diff <= tolerance,
            "pixel mismatch at byte {i}: actual={a}, reference={r}, diff={diff} > tolerance={tolerance}"
        );
    }
}
```

### 7.4 Model File Caching

The Kokoro q4 model (~80MB) is required for TTS integration tests. It is not committed to the repository. In CI, it is downloaded once and cached:

```yaml
- name: Cache Kokoro q4 model
  uses: actions/cache@v4
  with:
    path: test-fixtures/models/
    key: kokoro-q4-${{ hashFiles('scripts/download-test-models.sh') }}

- name: Download test models
  if: steps.cache.outputs.cache-hit != 'true'
  run: scripts/download-test-models.sh
```

Locally, developers download the model once with `scripts/download-test-models.sh`. The `test-fixtures/` directory is in `.gitignore`.

---

## 8. Release Checklist

The release checklist is a manual verification process that must be completed before publishing any release. It supplements the automated CI gates (Section 5.3).

### 8.1 Pre-Release Checklist

```markdown
## Release v{X.Y.Z} Checklist

### Automated (CI must be green)
- [ ] All CI stages pass on the release tag
- [ ] Binary size < 50MB (excluding voice models)
- [ ] Performance benchmarks within thresholds
- [ ] Zero `cargo audit` high-severity findings
- [ ] Zero `cargo deny` license violations
- [ ] Zero `npm audit` high-severity findings

### Manual Verification (Linux X11 -- primary platform)
- [ ] Fresh install: application launches from cold start in < 2s
- [ ] Magnification: full-screen mode at 2x, 5x, 10x, 20x -- smooth at 60fps
- [ ] Magnification: lens mode -- follows cursor, no visual artifacts
- [ ] Magnification: docked mode -- top, bottom, left, right edges work
- [ ] Color filters: inversion, grayscale, high-contrast -- correct rendering
- [ ] Cursor enhancement: enlarged cursor, crosshairs, halo -- visible and positioned correctly
- [ ] Control panel: opens, settings hydrate correctly
- [ ] Control panel: zoom slider, mode selector, all controls responsive
- [ ] Settings: change setting -> save -> restart -> setting persists
- [ ] Profiles: save, load, delete, export, import -- all work
- [ ] Hotkeys: zoom in, zoom out, toggle magnification -- all work

### Manual Verification (TTS -- when TTS features are in-scope)
- [ ] TTS: speak text produces audible output
- [ ] TTS: speech rate and volume controls work
- [ ] TTS: voice selection changes voice
- [ ] TTS: interrupt stops speech
- [ ] TTS: espeak-ng missing shows warning banner

### Keyboard Navigation
- [ ] Tab through every control panel page -- all controls reachable
- [ ] Focus indicators visible on every interactive element
- [ ] Slider values adjustable via arrow keys
- [ ] No keyboard traps

### Screen Reader (Orca on Linux)
- [ ] Orca reads all control labels and values
- [ ] Orca announces state changes (zoom level, mode switches)
- [ ] Live regions (`aria-live`) announce errors and warnings
- [ ] Magnification overlay does not interfere with Orca operation
- [ ] Orca + Luminos simultaneously: no audio conflicts, no focus stealing

### Platform-Specific (when releasing for additional platforms)
- [ ] macOS: VoiceOver reads control panel
- [ ] macOS: magnification overlay respects full-screen apps
- [ ] Windows: NVDA reads control panel (Phase 4+)
- [ ] Windows: JAWS coexistence tested (Phase 4+)
- [ ] OpenBSD: application launches and magnifies on X11

### Release Artifacts
- [ ] Release notes written and reviewed
- [ ] CHANGELOG.md updated
- [ ] Binary signatures valid (`gpg --verify`)
- [ ] SBOM generated (Phase 1+)
```

### 8.2 Release Frequency

From [Product Strategy](../PRODUCT_STRATEGY.md) Section 9.2:

| Track | Frequency | Testing |
|-------|-----------|---------|
| Development | Monthly | Full automated + abbreviated manual checklist |
| Stable | Quarterly | Full automated + complete manual checklist |
| Hotfix | As needed | Full automated + targeted manual verification of the fix |

---

## 9. Local Development Testing Workflow

### 9.1 Pre-Push Checks

Developers (human or AI agent) should run the following before pushing:

```bash
# Quick check (~30 seconds)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace --exclude luminos-app

# Frontend check (~15 seconds)
cd ui && pnpm lint && pnpm typecheck && pnpm test
```

This matches Stages 1-3 of the CI pipeline and catches most issues before CI runs.

### 9.2 Recommended Development Loop

For TDD as prescribed by SDD methodology:

```bash
# 1. Write a failing test (Red)
cargo nextest run -E 'test(~my_new_test_name)'  # Should fail

# 2. Implement the feature (Green)
cargo nextest run -E 'test(~my_new_test_name)'  # Should pass

# 3. Run the full suite to check for regressions
cargo nextest run --workspace --exclude luminos-app

# 4. Run clippy to catch anti-patterns
cargo clippy --workspace --all-targets -- -D warnings
```

### 9.3 Running Integration Tests Locally

Integration tests require additional setup:

```bash
# Install espeak-ng
sudo apt install espeak-ng       # Ubuntu/Debian
brew install espeak-ng            # macOS

# Download test model (one-time)
./scripts/download-test-models.sh

# Run integration tests
cargo nextest run --features integration_tests,ci_platform_tests
```

### 9.4 Running Shader Tests Locally

Shader tests run against the local GPU by default. To match CI behavior (software rendering):

```bash
# Software rendering (matches CI)
LIBGL_ALWAYS_SOFTWARE=1 cargo nextest run --features ci_platform_tests -E 'test(~shader_)'

# Native GPU (faster, may have slight pixel differences)
cargo nextest run --features ci_platform_tests -E 'test(~shader_)'
```

---

## 10. Test Naming Conventions

### 10.1 Hierarchical Naming

All tests use hierarchical prefixes for granular selection via `cargo nextest run -E 'test(~prefix_)'`. The convention is established in CLAUDE.md and [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 7.2:

**Rust pattern:** `{subsystem}_{component}_{behavior}_{condition}`

```
screen_capture_list_displays_returns_primary
screen_capture_capture_frame_region_out_of_bounds
focus_tracker_subscribe_receives_events
tts_preprocessor_sentence_split_basic
tts_espeak_subprocess_crash_recovery
render_viewport_calc_centered_on_cursor
render_shader_magnify_identity_transform
ipc_set_zoom_level_clamps_to_bounds
settings_persistence_save_load_roundtrip
```

**TypeScript pattern:** `{module}_{behavior}_{condition}`

```
settings_schema_rejects_zoom_below_minimum
settings_store_hydrate_sets_is_hydrating_false
zoom_slider_renders_current_zoom_level
zoom_slider_reverts_on_ipc_error
voice_selector_groups_voices_by_language
```

### 10.2 Filtering Examples

```bash
# All screen capture tests
cargo nextest run -E 'test(~screen_capture_)'

# All TTS tests
cargo nextest run -E 'test(~tts_)'

# All shader tests
cargo nextest run -E 'test(~shader_)'

# All IPC tests
cargo nextest run -E 'test(~ipc_)'

# All integration tests
cargo nextest run --features integration_tests -E 'test(~integration_)'

# All E2E tests
cargo nextest run --features e2e_tests -E 'test(~e2e_)'
```

---

## 11. Accessibility Testing

### 11.1 Automated Accessibility Checks

Automated checks run in CI on every PR as part of the component test suite:

| Tool | Scope | Standard | Integration |
|------|-------|----------|-------------|
| `axe-core` via `vitest-axe` | Every React page component | WCAG 2.1 AA | Component tests assert zero violations |
| `eslint-plugin-jsx-a11y` | All JSX in `ui/src/` | WCAG + ARIA best practices | ESLint check in Stage 1 |

Automated checks catch: missing labels, insufficient color contrast (4.5:1 for text, 3:1 for large text/UI components), keyboard traps, missing ARIA roles, invalid ARIA attributes, missing alt text.

Automated checks do NOT catch: screen reader compatibility, logical focus order, meaningful content structure, or usability by real users with disabilities.

### 11.2 Manual Accessibility Testing

Manual testing is required for each release, following the frequencies defined in [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 5.5:

| Method | Scope | Frequency | Phase |
|--------|-------|-----------|-------|
| Keyboard navigation audit | Control panel, all pages | Every release | Phase 0 |
| Orca screen reader testing | Control panel + overlay coexistence on Linux | Every release | Phase 0 |
| VoiceOver screen reader testing | Control panel + overlay coexistence on macOS | Every macOS release | Phase 2 |
| NVDA screen reader testing | Control panel + overlay coexistence on Windows | Every Windows release | Phase 4 |
| User testing with low-vision testers | Full application | Quarterly (when resources available) | Phase 1 |

### 11.3 Screen Reader Test Protocol

The manual screen reader test protocol verifies:

1. **Page navigation:** Orca/NVDA announces each page when navigating via sidebar
2. **Control labels:** Every interactive control has a readable label
3. **State changes:** Toggling a switch, moving a slider announces the new value
4. **Error announcements:** Errors are announced via `aria-live="assertive"` regions
5. **Status updates:** Non-critical status changes use `aria-live="polite"`
6. **Focus management:** Focus moves logically; no focus traps; focus returns to trigger after dialogs close
7. **Overlay coexistence:** The magnification overlay does not interfere with screen reader cursor or narration

Results are recorded in a Markdown checklist and attached to the release PR.

---

## 12. Performance Testing

### 12.1 Benchmark Suite

Performance benchmarks are implemented using `criterion` for statistical rigor:

```rust
// benches/frame_pipeline.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

fn bench_viewport_calculation(c: &mut Criterion) {
    let state = generate_test_app_state(/* zoom: 5.0, cursor: (960, 540) */);
    c.bench_function("viewport_calc_5x_1080p", |b| {
        b.iter(|| viewport::calculate(black_box(&state)))
    });
}

fn bench_frame_pipeline_mock_capture(c: &mut Criterion) {
    // Uses mock capture + real GPU (headless) to benchmark the render pipeline
    let mut group = c.benchmark_group("frame_pipeline");
    for zoom in [2.0, 5.0, 10.0, 20.0] {
        group.bench_function(format!("zoom_{zoom}x"), |b| {
            b.iter(|| render_one_frame(black_box(zoom)))
        });
    }
    group.finish();
}

fn bench_tts_preprocessing(c: &mut Criterion) {
    let text = "The quick brown fox jumps over the lazy dog. It was a sunny day.";
    c.bench_function("tts_preprocess_two_sentences", |b| {
        b.iter(|| preprocess(black_box(text)))
    });
}

// Statistical parameters are configured via the criterion_group! macro,
// not via Criterion.toml (which only supports output format and plotting options).
criterion_group!{
    name = benches;
    config = Criterion::default()
        .significance_level(0.05)
        .noise_threshold(0.02)
        .confidence_level(0.95)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .sample_size(100);
    targets = bench_viewport_calculation, bench_frame_pipeline_mock_capture, bench_tts_preprocessing
}
criterion_main!(benches);
```

### 12.2 Regression Detection

On the self-hosted benchmark runner (Phase 1+), Criterion stores historical baselines in `target/criterion/`. A regression is detected when:
- The new result is statistically worse than the baseline (p < 0.05)
- The magnitude exceeds the noise threshold (> 2%)

On GitHub-hosted runners (Phase 0), Criterion baselines are not reliable due to runner variability. Only absolute thresholds are enforced.

### 12.3 Profiling Integration

For deep performance investigation, the `profiling` crate integrates with Tracy ([06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 2.4):

```bash
# Build with profiling instrumentation
cargo build --release --features profiling

# Connect Tracy profiler to the running application
tracy-capture -o trace.tracy
```

Profiling is never enabled in CI. It is a developer tool for investigating performance regressions after CI detects them.

---

## 13. Phase Rollout

### 13.1 Phase 0: Foundation

| Capability | Status |
|-----------|--------|
| `cargo nextest` as test runner | Required |
| `cargo clippy` + `cargo fmt` lint gate | Required |
| `cargo deny` license check | Required |
| `cargo audit` vulnerability scan | Required |
| Rust unit tests with mock backends | Required |
| Vitest + React Testing Library for frontend | Required |
| `axe-core` accessibility checks | Required |
| GitHub Actions CI (Stages 1-4) | Required |
| Xvfb for headless Linux X11 CI | Required |
| Mesa llvmpipe for shader tests | Required |
| `tauri-driver` for IPC integration tests | Required |
| Benchmark thresholds (absolute, on hosted runners) | Required |
| Coverage reporting (informational, not gated) | Required |
| Manual keyboard navigation test per release | Required |
| Manual Orca screen reader test per release | Required |

### 13.2 Phase 1: Hardening

| Capability | Status |
|-----------|--------|
| Self-hosted benchmark runner (Intel UHD 770) | New |
| Criterion regression detection with stable baselines | New |
| Benchmark trend tracking (history.json) | New |
| OpenBSD self-hosted CI runner | New |
| Wayland headless CI (weston or cage) | New |
| SBOM generation in release pipeline | New |
| Coverage dashboard | New |
| Reference image update workflow for shaders | New |

### 13.3 Phase 2: Platform Expansion

| Capability | Status |
|-----------|--------|
| macOS VoiceOver manual test per release | New |
| GPU texture sharing tests (platform-specific) | New |
| TTS latency benchmarks with real model inference | New |
| `TtsTimings` / `get_tts_timings` diagnostics | New |
| Performance profiling CI job (nightly, Tracy traces) | New |

### 13.4 Phase 3+: Maturity

| Capability | Status |
|-----------|--------|
| Windows NVDA manual test per release | New (Phase 4) |
| JAWS coexistence testing | New (Phase 4) |
| Plugin security testing framework | New (Phase 4) |
| Fuzz testing for IPC boundary | New (Phase 3) |
| Reproducible build verification | New (Phase 3) |
| User testing with low-vision testers (quarterly) | Ongoing from Phase 1 |

---

## 14. Cross-References

| Topic | Document | Section |
|-------|----------|---------|
| Platform trait mock implementations | [02 -- Platform Abstraction](./02-platform-abstraction.md) | 7.1 |
| Platform test naming conventions | [02 -- Platform Abstraction](./02-platform-abstraction.md) | 7.2 |
| Platform-specific CI matrix (first defined) | [02 -- Platform Abstraction](./02-platform-abstraction.md) | 7.3 |
| Pipeline integration tests with mock backends | [02 -- Platform Abstraction](./02-platform-abstraction.md) | 7.4 |
| Rendering pipeline unit tests | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 12.1 |
| Shader test methodology | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 12.2 |
| Rendering integration tests | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 12.3 |
| Rendering test fixtures | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 12.4 |
| TTS test approach and mock strategy | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 14.1, 14.2 |
| TTS test naming convention | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 14.3 |
| TTS CI considerations | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 14.4 |
| Control panel test toolchain | [05 -- Control Panel](./05-control-panel.md) | 13.1 |
| TypeScript unit test inventory | [05 -- Control Panel](./05-control-panel.md) | 13.2 |
| Component test examples | [05 -- Control Panel](./05-control-panel.md) | 13.3 |
| IPC integration test specification | [05 -- Control Panel](./05-control-panel.md) | 13.4 |
| Control panel accessibility tests | [05 -- Control Panel](./05-control-panel.md) | 13.5 |
| Performance targets and budgets | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | 2.1-2.3 |
| Profiling strategy | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | 2.4 |
| CI benchmark suite (first defined) | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | 2.4 |
| Degradation strategy | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | 2.5 |
| Supply chain security (cargo audit, cargo deny) | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | 3.6 |
| Accessibility testing frequencies | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | 5.5 |
| Panic policy and clippy enforcement | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | 7.4 |
| Observability and diagnostics | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | 6 |
| SDD methodology and templates | [specs/README.md](../README.md) | Testing Strategy, TDD Workflow |
| Development process and CI/CD | [Product Strategy](../PRODUCT_STRATEGY.md) | 9.2 |
| Build and distribution | [08 -- Build and Distribution](./08-build-and-distribution.md) | Sections 4, 8, 9, 11 |

---

## 15. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-17 | Initial testing strategy document |
| 1.1 | 2026-03-17 | Post-audit revision: replaced invalid Criterion.toml config with Rust API code (F-001); corrected macOS Screen Recording permission claim -- not auto-granted on GitHub Actions (F-002); clarified that frame time 16.67ms and memory 800MB warn thresholds are new, not consolidated from doc-06 (F-003); added tauri-driver macOS limitation (no WKWebView driver) (F-004); added missing memory warn threshold to benchmark script (F-005); clarified new test tools vs. doc-05 base toolchain (F-006); aligned Wayland CI terminology with doc-02 (F-007); replaced choco espeak-ng install with MSI download; added --benchmark-mode explanation; removed misleading Vitest alias; added E2E flakiness management policy (P-002); added coverage trend monitoring (P-003) |
