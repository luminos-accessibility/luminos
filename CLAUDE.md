# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## General rules

- ALWAYS ask at least 5 clarifying questions to the user using `AskUserQuestion` tool. Use this to refine your plan, ideas, assumptions and the tasks' ambiguities.
- ALWAYS add a code reviewer, a quality assurance engineer and a technical auditor to the Agent Team when executing coding tasks.
- ALWAYS add a technical auditor and quality reviewer to the Agent Team for non-coding tasks (like design, research, doc writing or producing specs).
- ONLY consider tasks done when reviewers, QA and auditors give explicit approval. Keep iterating until then.
- ALWAYS prefer using existing specialist agents in this project when spawning new teammates for the Agent Team. Create quality on-demand agent if a fitting specialist is not available.

## Language Policy

- **NEVER use Python** anywhere in this project. All scripts, tooling, and automation MUST use TypeScript executed with `npx tsx`.
- This applies to CI pipelines, build scripts, utility scripts, benchmarks, and any new tooling.
- The `.claude/skills/` directory is exempt from this policy as it contains third-party tools.

## Project Overview

**Luminos** is a GPLv3-licensed, cross-platform (Linux/macOS/OpenBSD/Windows) screen magnification + text-to-speech accessibility suite targeting low-vision users. The project has completed its **technical strategy phase** and **all of Phase 0 (Foundation)**: **Epic E01 (Project Scaffolding, Platform Traits & CI/CD)**, **Epic E02 (X11 Screen Capture & GPU Magnification)**, **Epic E03 (Input Tracking & Interactive Magnification)**, and **Epic E04 (Tauri Control Panel & Settings Persistence)** — the first running Luminos application. The repository contains the complete product strategy, technology evaluation, and technical strategy documents (10 documents covering architecture through risk management), plus the Rust codebase: 6 crates (`luminos-types`, `luminos-core`, `luminos-platform`, `luminos-gpu`, `luminos-tts`, `luminos-app`) with trait definitions, mock implementations, error hierarchy, core data types, X11 screen capture backend, GPU rendering pipeline (texture management, magnification shaders, frame pacing), X11 input monitoring (XInput2 mouse/keyboard capture), lock-free state management (ArcSwap), viewport tracking engine (dead zone, edge panning, smooth interpolation), global keyboard shortcuts (zoom in/out, toggle, reset), input processing pipeline integration, and a single-`tauri::App::run`-loop application shell (transparent click-through wgpu overlay + React control panel, x11rb overlay `WindowManager`, 7-command/2-event `tauri-specta` IPC, `config.toml` persistence, system tray), plus a `ui/` React project. Tests total ≈446 workspace + 67 `luminos-app` Rust + 70 UI Vitest, backed by a GitHub Actions CI pipeline (8 active jobs) with dedicated platform (X11/Xvfb), GPU (Mesa llvmpipe), Tauri app, and `tauri-driver` E2E test jobs.

## Repository Structure

- `specs/` — Spec-driven development artifacts (strategies, designs, implementation stories)
    - `PRODUCT_STRATEGY.md` — Product strategy & roadmap v1.3. The canonical product definition.
    - `TECH_STACK_EVALUATION.md` — Technology stack validation report. Contains the **revised** recommended stack (supersedes some choices in the strategy doc).
    - `README.md` — Spec-driven development (SDD) methodology guide with templates
    - `tech-strategy/` — **COMPLETE** technical strategy (10 documents). The canonical technical reference.
        - `README.md` — Strategy index, executive summary, conventions, risk register maintenance guide
        - `01-system-architecture.md` — Dual-window design, component model, threading, state management
        - `02-platform-abstraction.md` — 6 trait definitions, per-platform backends, conditional compilation
        - `03-rendering-pipeline.md` — GPU capture, shaders, frame pacing, zoom modes, font re-rendering research
        - `04-tts-pipeline.md` — espeak-ng subprocess, Kokoro/sherpa-onnx inference, ring buffer audio
        - `05-control-panel.md` — Tauri IPC, React UI, settings, profiles
        - `06-cross-cutting-concerns.md` — Performance budgets, security, licensing, a11y, observability, i18n
        - `07-testing-strategy.md` — Test pyramid, CI/CD pipeline (8 stages), quality gates, platform matrix
        - `08-build-and-distribution.md` — Cargo workspace, packaging (7 formats), signing, auto-update
        - `09-implementation-roadmap.md` — 20 epics across 5 phases, 20-month timeline
        - `10-risk-register.md` — 38 risks with mitigations, living document updated at phase gates
    - `ENN-epic-name/` — Engineering epic folders (one per roadmap epic E01-E20)
        - `HIGH_LEVEL_PLAN.md` — Epic-level plan, story breakdown, shared context
        - `NNN-story-name/` — Implementation story folders (STORY.md, DESIGN.md, SUBTASKS.md)
- `docs/` — Product documentation and user manuals (created when development begins)
- `.claude/agent-memory/` — Persistent memory for specialized agents

## Architecture (Decided — see `specs/tech-strategy/` for full details)

**Cargo workspace crates:**
- `luminos-types` — Shared data types (zero workspace deps, only serde). Canonical definitions for `ScreenRect`, `DisplayInfo`, `CaptureFrame`, `DockEdge`, `LensShape`, `OverlayMode`, etc.
- `luminos-platform` — Platform abstraction: 6 traits + per-platform backends. Re-exports types from `luminos-types`.
- `luminos-core` — Application state, settings schema. Re-exports types from `luminos-types`.
- `luminos-gpu` — GPU rendering pipeline (wgpu shaders, device/surface management, frame pacing).
- `luminos-tts` — Text-to-speech pipeline (espeak-ng + Kokoro ONNX).
- `luminos-app` — Tauri application shell.

**Dual-window design:**

1. **Control Panel** — Tauri 2.0 webview (TypeScript/React). Settings UI. Not performance-critical.
2. **Magnification Overlay** — Transparent, click-through wgpu surface on the Tauri (tao) window, controlled via x11rb. GPU-accelerated, always-on-top. Bypasses the webview entirely.

**Rendering pipeline:** screen capture → GPU texture → wgpu shader transform → anti-alias → composite → present

**TTS pipeline:** text → espeak-ng subprocess (phonemes, crash-isolated) → Kokoro ONNX inference (via sherpa-rs) → cpal audio output

## Platform Development Order

Linux X11 first → Linux Wayland → macOS → OpenBSD → Windows

## Licensing

Luminos is licensed under **GPLv3**. espeak-ng (GPL-3.0) is used for phonemization by both Kokoro and Piper TTS engines. Since the project itself is GPLv3, there is no license propagation concern. espeak-ng is still run as a **subprocess** for engineering reasons (crash isolation, resource management, testability), not for legal isolation. Medium-term: evaluate misaki transformer-based G2P as a way to reduce external dependencies.

## Key Constraints & Performance Targets

- 60fps (16ms frame time) on integrated GPUs
- <4GB RAM
- <200ms TTS latency
- <50MB binary (excluding voice models)
- <2s startup to usable magnification
- Must work alongside screen readers (NVDA/JAWS) on Windows

## Development Philosophy

- **AI-agent driven development** — TypeScript for AI-friendly UI generation, Rust compiler as automated reviewer for AI-generated code
- **Trait-based platform abstraction** — `ScreenCapture`, `FocusTracker`, `TtsEngine`, `WindowManager`, `InputMonitor`, `AudioOutput` traits with per-platform backends
- **Local-first, privacy by design** — All AI inference on-device, no telemetry by default

## CI / Quality Assurance Commands

These commands mirror the GitHub Actions CI pipeline (`.github/workflows/ci.yml`). **Quality assurance agents MUST run ALL of these checks before considering work complete.** CI sets `RUSTFLAGS="--deny warnings"` — replicate this to catch warning-as-error failures locally.

**If the CI pipeline is modified, this section MUST be updated to match.** The source of truth is `.github/workflows/ci.yml`.

The CI pipeline has 8 active jobs. All test/security/coverage jobs depend on lint passing first.

### QA Agent Minimum Checks

**QA agents on implementation teams MUST run at least checks 1-4 below** before approving any implementation work. QA agents MUST NOT edit source files — only validate. If checks fail, report the failure to the implementor for fixing.

- **For changes to `luminos-platform`:** Also run check 6 (Platform Tests) if an X11 display is available.
- **For changes to `luminos-gpu`:** Also run check 7 (GPU Tests) if Mesa llvmpipe is available.
- **For all changes:** Verify that every acceptance criterion from the story's STORY.md has at least one passing test covering it. Produce an AC coverage matrix in the QA report.

### 1. Formatting

```bash
cargo fmt --all -- --check
```

### 2. Linting (Clippy)

```bash
RUSTFLAGS="--deny warnings" cargo clippy --workspace --all-targets --all-features \
  -- -D warnings \
  -W clippy::unwrap_used \
  -W clippy::expect_used \
  -W clippy::pedantic \
  -A clippy::module_name_repetitions
```

### 3. Unit Tests

Requires [cargo-nextest](https://nexte.st/). The `ci` profile enables retries and relaxed timeouts (see `.config/nextest.toml`).

```bash
RUSTFLAGS="--deny warnings" cargo nextest run --profile ci --workspace --exclude luminos-app
```

### 4. Security Audit

Requires [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) and [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit).

```bash
cargo deny check licenses advisories
cargo audit
```

### 5. Test Coverage

Requires [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov).

```bash
RUSTFLAGS="--deny warnings" cargo llvm-cov --workspace --exclude luminos-app --lcov --output-path lcov.info
```

### 6. Platform Tests (X11/Xvfb)

Runs `luminos-platform` tests that require a live X11 display. Requires Xvfb, picom, xdotool, and X11 dev libraries. CI runs these under `xvfb-run` with a virtual 1920x1080 screen and picom as compositor.

```bash
xvfb-run -s "-screen 0 1920x1080x24" bash -c \
  "picom --backend xrender --daemon && \
   RUSTFLAGS='--deny warnings' cargo nextest run --profile ci \
   -p luminos-platform --features ci_platform_tests"
```

**Important notes for platform tests:**
- `xdotool` is required for input monitoring integration tests (mouse move, key press simulation). Tests gracefully skip if `xdotool` is not installed, but CI MUST have it.
- Tests that set `DISPLAY` to an invalid value use `:54321` (not `:99`). The `xvfb-run` default display is `:99`, so using it as an "invalid" display causes false passes.
- Integration tests are gated behind `#[cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]` — they only compile and run when the `ci_platform_tests` feature is enabled.

### 7. GPU Tests (Mesa llvmpipe)

Runs `luminos-gpu` tests that require a GPU context. Uses Mesa llvmpipe software renderer. Requires Xvfb, picom, Mesa drivers, and the same X11 dev libraries as platform tests.

```bash
MESA_GL_VERSION_OVERRIDE="4.5" LIBGL_ALWAYS_SOFTWARE="1" \
  xvfb-run -s "-screen 0 1920x1080x24" bash -c \
  "picom --backend xrender --daemon && \
   RUSTFLAGS='--deny warnings' cargo nextest run --profile ci \
   -p luminos-gpu --features ci_platform_tests"
```

### 8. App Shell Tests (Tauri + Xvfb)

Runs `luminos-app` lib unit tests (incl. the offscreen Mesa-llvmpipe clear test) and the subprocess integration tests that spawn the real Tauri binary. Requires webkit2gtk-4.1 + libsoup-3.0 (to build the app), Xvfb, picom, `x11-utils` (`xprop`), `xdotool`, and Mesa drivers, plus a built `ui/dist` (run `pnpm --dir ui build` first so `overlay.html`/`index.html` exist). The harness launches its OWN Xvfb + picom per test (no outer `xvfb-run`) and sets `GDK_BACKEND=x11` + `WEBKIT_DISABLE_COMPOSITING_MODE=1` + `WEBKIT_DISABLE_DMABUF_RENDERER=1` + `NO_AT_BRIDGE=1` per spawned child (the first three are required for GTK window realization + software GL under a headless Xvfb; `NO_AT_BRIDGE=1` disables the GTK AT-SPI accessibility bridge, whose atk-bridge module load during GTK init otherwise blocks ~9s trying to reach a non-existent session/a11y D-Bus on a CI runner — long enough on a slow runner to blow the 20s overlay-marker timeout and fail the boot-dependent subprocess tests). Run single-threaded so each test's dedicated display does not contend.

```bash
MESA_GL_VERSION_OVERRIDE="4.5" LIBGL_ALWAYS_SOFTWARE="1" \
  cargo nextest run --profile ci \
  -p luminos-app --features "tauri custom-protocol ci_platform_tests" --test-threads 1
```

**`custom-protocol` is now a DEFAULT feature** (`default = ["tauri", "custom-protocol"]`) so a plain `cargo run -p luminos-app` serves the embedded `frontendDist` and the control panel works out of the box. Without it Tauri runs in dev mode and the control-panel webview loads `build.devUrl` (`http://localhost:1420`) instead of the embedded `frontendDist`, so a binary with no dev server shows "Could not connect to localhost: Connection refused" and IPC never initializes. Because it is a default, every build that doesn't pass `--no-default-features` already has it — the explicit `--features "tauri custom-protocol"` in the CI app/e2e jobs is now redundant-but-harmless and kept for clarity. **The ONLY build that must turn it OFF is the hot-reload dev workflow** (see below).

### Running the App (manual testing)

Build the frontend once, then run the binary — `custom-protocol` is a default feature so the embedded UI loads with no extra flags. On Linux the binary auto-pins the X11/XWayland backend at startup (`platform_env::force_x11_backend`), so it runs on both X11 and Wayland sessions (under XWayland on the latter, with a logged notice — native Wayland is Epic E08). Closing the control-panel window quits the app unless `minimize_to_tray` is enabled (then it hides to the tray).

```bash
pnpm --dir ui build                                      # produce ui/dist (embedded at compile time)
cargo run -p luminos-app                                 # custom-protocol is default — control panel just works
```

For the hot-reload dev workflow instead, use the Tauri CLI with default features disabled so it serves the live Vite dev server (`build.devUrl`, `:1420`) rather than the stale embedded `frontendDist`:

```bash
cargo tauri dev --no-default-features --features tauri
```

This is the **only** mode that should rely on `build.devUrl`; everything else uses the embedded assets via the default `custom-protocol`.

The same job also runs the **tauri-specta bindings-up-to-date check** (story E04/005, D7): it regenerates `ui/src/ipc/bindings.ts` from the Rust IPC surface via the app's `--export-bindings` seam (which exports the bindings and exits — no Xvfb/webview needed) and fails if the committed file drifted. **If you change a `#[tauri::command]`/`#[tauri_specta::Event]` definition or any `specta::Type` it touches, regenerate and commit `ui/src/ipc/bindings.ts`.**

```bash
# Regenerate the committed bindings, then fail on drift (CI step):
cargo run -p luminos-app --features "tauri" -- --export-bindings
git diff --exit-code ui/src/ipc/bindings.ts
```

### 9. E2E Tests (tauri-driver + WebKitWebDriver)

Runs the WebdriverIO IPC integration suite in `e2e/` (story E04/007, D2/D3/D4) against the **built** `luminos-app` binary, driving the real control-panel webview via the Rust `tauri-driver` proxy + `WebKitWebDriver`. The suite drives the UI (zoom slider, mode radios) and asserts the **real engine state** through a `get_current_settings` round-trip (not just the React store), verifying the IPC contract end-to-end. Requires `webkit2gtk-driver` (ships `WebKitWebDriver`), `libayatana-appindicator3-dev` (the tray SNI host), `xvfb`, `picom`, the app build deps, and the Rust `tauri-driver` binary (`cargo install tauri-driver --version 2.0.6 --locked`, pinned per PINNED_VERSIONS.md §3). The job builds `ui/dist` + the debug app (`cargo build -p luminos-app --features "tauri custom-protocol"` — `custom-protocol` is mandatory so the webview serves the embedded `frontendDist` instead of the absent dev server), then runs the suite under `xvfb-run` + picom with the headless-WebKit env (`GDK_BACKEND=x11`, `WEBKIT_DISABLE_COMPOSITING_MODE=1`, `WEBKIT_DISABLE_DMABUF_RENDERER=1`, `MESA_GL_VERSION_OVERRIDE=4.5`, `LIBGL_ALWAYS_SOFTWARE=1`).

```bash
# CI-only (needs WebKitWebDriver + tauri-driver, NOT typically on a dev box):
cargo install tauri-driver --version 2.0.6 --locked
pnpm --dir ui install --frozen-lockfile && pnpm --dir ui build
cargo build -p luminos-app --features "tauri custom-protocol"
pnpm --dir e2e install --frozen-lockfile
MESA_GL_VERSION_OVERRIDE="4.5" LIBGL_ALWAYS_SOFTWARE="1" \
  GDK_BACKEND="x11" WEBKIT_DISABLE_COMPOSITING_MODE="1" WEBKIT_DISABLE_DMABUF_RENDERER="1" \
  xvfb-run -s "-screen 0 1920x1080x24" bash -c \
  "picom --backend xrender --daemon && pnpm --dir e2e test"

# Typecheck the E2E specs/config without running the driver (runnable anywhere):
pnpm --dir e2e install --frozen-lockfile && pnpm --dir e2e exec tsc --noEmit
```

**Local-run note:** the full E2E run is **deferred to CI** — `WebKitWebDriver` and `xvfb-run` are commonly absent on dev boxes (and macOS has no WKWebView driver at all, NFR-2). Locally, run only the `tsc --noEmit` typecheck; the live driver assertions (D2/D3/D4) are verified in the `test-e2e` CI job.

## Current Project Phase

**Technical strategy is COMPLETE.** **Phase 0 (Foundation) is COMPLETE** (Months 1-3; all four epics E01-E04 closed). The project is now entering **Phase 1: Core Magnification** (Months 4-6, epics E05-E09).

Phase 0 epics (from `specs/tech-strategy/09-implementation-roadmap.md`):

- **E1:** Project Scaffolding, Platform Traits & CI/CD -- **COMPLETE** (2026-03-28). 5 stories, 53 subtasks, 114 tests passing, clippy clean, fmt clean.
- **E2:** X11 Screen Capture + GPU Rendering -- **COMPLETE** (2026-03-28). 5 stories, 14 subtasks in final story, 275 tests passing, clippy clean, fmt clean. New modules in `luminos-gpu`: `texture.rs` (`SourceTextureManager`), `viewport.rs` (`compute_source_region`, `smooth_viewport_position`), `shaders/` (bilinear + bicubic WGSL magnification shaders, `MagnifyUniforms`, `MagnifyPipeline`, `InterpolationMethod`), `frame_timings.rs` (`FrameTimings` ring buffer, `FrameTimingSummary`, performance threshold detection), `renderer.rs` (`Renderer` struct orchestrating capture-upload-render-present pipeline). CI additions: `test-platform` (X11/Xvfb) and `test-gpu` (Mesa llvmpipe) jobs.
- **E3:** Input Tracking & Interactive Magnification -- **COMPLETE** (2026-03-29). 5 stories, ~70 subtasks, 418 tests passing, clippy clean, fmt clean. New modules: `luminos-platform::linux_x11::input` (`X11InputMonitor` with XInput2), `luminos-platform::linux_x11::keymap` (89-keysym mapping), `luminos-core::state_manager` (`StateManager` wrapping `ArcSwap<AppState>`), `luminos-core::event` (`LuminosEvent`), `luminos-core::tracking` (`TrackingEngine` with dead zone, edge panning, smooth interpolation), `luminos-core::hotkeys` (`HotkeyMatcher`, `dispatch_hotkey`), `luminos-core::pipeline` (`EventNotifier` trait, `InputProcessingTask`). Key decisions: x11rb over rdev for X11 input, EventNotifier trait for testability, ArcSwap for lock-free render thread state reads.
- **E4:** Control Panel Foundation (Tauri IPC, React settings UI) -- **COMPLETE** (2026-06-05). 7 stories, ≈446 workspace + 67 `luminos-app` Rust + 70 UI Vitest tests passing, clippy clean, fmt clean. The first running Luminos application: a single tao/Tauri event loop (RISK-001 retired) hosting a transparent click-through wgpu overlay + a React control panel, live full-screen magnification, x11rb overlay `WindowManager` (no winit/tauri dep in `luminos-platform`), `ConfigManager` settings persistence, typed `tauri-specta` IPC (7 commands + 2 events, frozen `bindings.ts` + CI diff gate), a system tray (Show/Hide + Quit + minimize-to-tray, graceful no-SNI degrade), and a `tauri-driver` E2E CI job. New `luminos-app` crate (`app`/`handle`/`notifier`/`ipc`/`tauri_commands`/`events`/`overlay_gpu`/`overlay_bridge`/`capture_driver`/`tray`/`signal`/`compositor`); `luminos-core::config` (`ConfigManager`); `e2e/` WDIO project. CI additions: `test-app` (Tauri+Xvfb) and `test-e2e` (tauri-driver+WebKitWebDriver) jobs (8 active jobs total). Key decisions: single `tauri::App::run` loop over a second winit `EventLoop` (RISK-001); `AppNotifier` dirty-flag wake over `EventLoopProxy`; x11rb-over-tao-window overlay `WindowManager` (keeps `luminos-platform` tauri/winit-free). **Honest CI blind spots (DC-10/DC-13):** live magnify present + non-zero P99 + tray-icon-visible are HW/manual-only; the `test-e2e` job is authored/wired/typechecked with its first green CI run pending.

**Current work:** Epics E01, E02, E03, and E04 are complete — **Phase 0 (Foundation) deliverables are done**. Next epic is E05 (per the roadmap).

## When Editing Strategy Documents

- All market claims must cite specific sources with dates
- Distinguish between WebAIM Low Vision Survey #1 (2013) and #2 (2018) — they have different stats
- ZoomText max zoom is 36x on current versions (60x was legacy Win 8 only)
- Section 508 references WCAG 2.0 AA, not 2.1
- Crate versions change quickly — always verify against crates.io before citing

## Development Methodology

**Spec-Driven Development (SDD)** with integrated TDD. Work is organized as **epics** (from the roadmap) containing **stories**:

1. **HIGH_LEVEL_PLAN.md** — Epic-level decomposition into stories, shared context, progress tracking
2. **STORY.md** — Requirements with Given-When-Then acceptance criteria
3. **DESIGN.md** — Architecture, API design, testing strategy mapped to every AC
4. **SUBTASKS.md** — TDD task breakdown (red-green-refactor) + progress tracking (the execution memory file)

Epics live in `specs/ENN-epic-name/` folders. Stories live in `specs/ENN-epic-name/NNN-story-name/` folders. See `specs/README.md` for full methodology, templates, and governance rules.

**Key SDD rules:**

- No implementation without an approved STORY.md and DESIGN.md
- Every test traces to an acceptance criterion
- SUBTASKS.md completion notes are mandatory (they are the memory for agent handoffs)
- HIGH_LEVEL_PLAN.md shared context updates are mandatory when a story produces cross-story knowledge
- Stories target 5-15 subtasks; split if exceeding 20
- Epics target 3-8 stories; split if exceeding 8
- DESIGN.md must reference relevant risks from the risk register (`specs/tech-strategy/10-risk-register.md`)
- Agents working on a story read ONLY the epic's HIGH_LEVEL_PLAN.md and their story's three artifacts (not other stories)

## General writing rules

- When handling memory files, NEVER use absolute paths, always reference paths relative to the project root.
- When using a library or API, always fetch documentation from context7 or web search tools.
- When working on writing large files (1,000 lines or more), write a skeleton of the file first, then write each section/method/function in a separate write tool call.
- You have a team of specialists that you can delegate subtasks to, make use of them when needed.

## Rust Coding Rules

### Naming & Style

- Follow standard Rust naming conventions per [RFC 430](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md)
- Reuse names from the architecture (trait names, module names) rather than inventing new ones
- Name unit tests with hierarchical prefixes for granular selection via `cargo nextest run` (e.g., `screen_capture_init_success`, `screen_capture_init_missing_permission`)

### Error Handling

- **Prefer `?` propagation** over exhaustive `match`/`if-let` chains. Implement `From` trait conversions when error types mismatch.
- **No `unwrap()` or `expect()` in production code.** Use `match`, `if let`, or `.unwrap_or_else()` with sensible defaults. Exception: `unwrap()` is acceptable in unit tests to keep them concise.
- Convert between `Result<T,E>` and `Option<T>` with `.ok()?` rather than match statements

### Control Flow & Idioms

- Limit nesting to 1-2 levels. Use early-return guard clauses to separate error handling from core logic.
- Prefer `while let` over `loop { match ... break }` patterns
- Prefer lazily evaluated combinators (`and_then()`, `or_else()`, `unwrap_or_else()`) over eager variants (`and()`, `or()`, `unwrap_or()`) when the alternative involves allocation or computation
- Prefer iterator chains (`.filter()`, `.map()`, `.collect()`) over manual loop-and-accumulate patterns

### Async Discipline

- **Don't make sync code async.** Async is for I/O-intensive, network, or background tasks. Synchronous operations that are only called synchronously must remain sync.
- **Don't mix sync and async without justification.** Mixing can cause deadlocks and performance issues.

### Logging

- Surround dynamic values in single quotes to distinguish from static text: `log::info!("Capturing display '{}'", display.name)`
- Use `concat!` for multiline log messages
- Severity levels: **trace** (granular diagnostics), **debug** (developer-focused), **info** (important state changes), **warn** (unexpected but non-fatal), **error** (failures that may panic)

### Testing

- Place test mock/fixture generation in the same module where the type is defined
- Prefix all test object generators with `generate_test_` (e.g., `generate_test_capture_config()`)
- Gate test-only code with `#[cfg(test)]` or `#[cfg(feature = "test_utils")]`
- Make test generators public and parametrizable for reuse across modules

## TypeScript Coding Rules

- Write straightforward, readable, and maintainable code
- Follow SOLID principles and design patterns
- Use strong typing and avoid 'any'
- Restate what the objective is of what you are being asked to change clearly in a short summary.
- Utilize Lodash, 'Promise.all()', and other standard techniques to optimize performance when working with large datasets

### Naming Conventions

- Classes: PascalCase
- Variables, functions, methods: camelCase
- Files, directories: kebab-case
- Constants, env variables: UPPER_SNAKE_CASE

### Functions

- Use descriptive names: verbs & nouns (e.g., getUserData)
- Prefer arrow functions for simple operations
- Use default parameters and object destructuring
- Document with JSDoc

### Types and Interfaces

- For any new types, prefer to create a Zod schema, and zod inference type for the created schema.
- Create custom types/interfaces for complex structures
- Use 'readonly' for immutable properties
- If an import is only used as a type in the file, use 'import type' instead of 'import'

## Commit Messages

Follow Conventional Commits v1.0.0 (https://www.conventionalcommits.org/en/v1.0.0/).

### Format

```
<type>[(scope)][!]: <description>

[body]

[footer(s)]
```

### Types

| Type       | When to use                                      |
|------------|--------------------------------------------------|
| feat       | New feature (→ SemVer MINOR)                     |
| fix        | Bug fix (→ SemVer PATCH)                         |
| docs       | Documentation only                               |
| style      | Formatting, whitespace — no logic change         |
| refactor   | Code change that neither fixes nor adds features |
| perf       | Performance improvement                          |
| test       | Adding or correcting tests                       |
| build      | Build system or external dependency changes      |
| ci         | CI configuration and scripts                     |
| chore      | Maintenance tasks that don't modify src or tests |
| revert     | Reverts a previous commit                        |

### Rules

- The description MUST immediately follow <type>[(scope)]: with a single space.
- Use imperative, present tense ("add", not "added" or "adds").
- Do NOT capitalize the first letter of the description.
- Do NOT end the description with a period.
- Scope is optional — use a noun describing the affected area: feat(parser):, fix(api):.
- Breaking changes MUST append ! before the colon OR include a BREAKING CHANGE: footer (or both).
- A BREAKING CHANGE footer triggers a SemVer MAJOR bump regardless of type.
- Body and footers are separated from the description by a blank line.
- Keep the subject line under 72 characters.
- One logical change per commit — if a commit spans multiple types, split it.

## Versioning

See [VERSIONING.md](VERSIONING.md) for the complete versioning policy (SemVer 2.0.0, lockstep workspace versioning, bump rules, milestones).
