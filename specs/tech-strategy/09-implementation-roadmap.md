# 09 -- Implementation Roadmap

**Status:** DRAFT v1.0
**Date:** 2026-03-18
**Audience:** Everyone (engineers, product managers, contributors, AI agents)
**Source Documents:** [Product Strategy](../PRODUCT_STRATEGY.md) (v1.3, Sections 7-9), [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) (FINAL), [System Architecture](./01-system-architecture.md), [Platform Abstraction](./02-platform-abstraction.md), [Rendering Pipeline](./03-rendering-pipeline.md), [TTS Pipeline](./04-tts-pipeline.md), [Control Panel](./05-control-panel.md), [Cross-Cutting Concerns](./06-cross-cutting-concerns.md), [Testing Strategy](./07-testing-strategy.md), [Build and Distribution](./08-build-and-distribution.md)

---

## 1. Overview

### 1.1 Purpose

This document translates the architecture defined in docs 01-08 into a sequenced delivery plan: 20 engineering epics organized into 5 phases, spanning approximately 20 months. Each epic is a self-contained unit of work that a team can pick up, decompose into implementation stories (STORY.md / DESIGN.md / SUBTASKS.md per the [SDD methodology](../README.md)), and deliver in 2-6 weeks.

This document answers: **In what order do we build Luminos, and what does each increment deliver to users?**

### 1.2 Scope

This document covers:
- Epic definitions: scope, deliverables, dependencies, estimated duration, success criteria
- Dependency graph showing which epics can run in parallel vs. which are sequential
- Phase gates defining what must be true before advancing to the next phase
- Timeline overview with critical path analysis
- Staffing assumptions and parallelization opportunities

This document does NOT cover:
- Story-level task breakdowns (those are created per-epic by the team, following the SDD templates)
- Detailed technical design for individual features (see docs 01-08)
- Risk analysis and mitigation strategies (see [10 -- Risk Register](./10-risk-register.md) (planned))

### 1.3 Relationship to Product Phases

The [Product Strategy](../PRODUCT_STRATEGY.md) Section 7 defines five product phases. This roadmap maps engineering epics to those phases, but the mapping is not always one-to-one. Product phases define *what* ships to users; epics define *how* the engineering work is sequenced to build it. Some product features span multiple epics, and some epics deliver infrastructure that enables multiple product features.

| Product Phase | Months | Epics | Engineering Focus |
|---------------|--------|-------|-------------------|
| **Phase 0: Foundation** | 1-3 | E1-E4 | Workspace, X11 capture, GPU rendering, input, control panel |
| **Phase 1: Core Magnification** | 4-6 | E5-E9 | Zoom modes, visual enhancements, focus tracking, Wayland, Linux packages |
| **Phase 2: TTS + Cross-Platform** | 7-9 | E10-E12 | TTS pipeline, TTS UX, macOS full support |
| **Phase 3: Advanced + AI** | 10-14 | E13-E16 | Font re-rendering, OCR, OpenBSD, advanced magnification |
| **Phase 4: Platform & Ecosystem** | 15-20 | E17-E20 | Windows, plugins, enterprise, i18n |

### 1.4 How to Read This Document

Each epic in Sections 4-8 follows a consistent template:

- **Summary** -- One-paragraph description of what the epic delivers
- **Scope** -- Specific items included and excluded
- **Dependencies** -- Which epics must be complete (or substantially complete) before this one starts
- **Deliverables** -- Concrete, testable outputs
- **User-Perceivable Value** -- What a user can do after this epic ships that they could not do before
- **Key Risks** -- Highest-risk items within the epic
- **Estimated Duration** -- In weeks (1 sprint = 2 weeks)
- **Success Criteria** -- Measurable conditions that define "done"
- **Primary Docs** -- Which tech strategy documents define the detailed design

---

## 2. Epic Structure & Conventions

### 2.1 Sizing Rules

Each epic is scoped to be completable in **2-6 weeks** (1-3 two-week sprints) by the team working on it. If an epic exceeds 6 weeks during story decomposition, it must be split. If it is under 2 weeks, it should be merged into an adjacent epic.

### 2.2 From Epics to Stories

When the team picks up an epic, the workflow is:

1. **Create epic folder** -- Create `specs/ENN-epic-name/` and initialize `HIGH_LEVEL_PLAN.md` from the SDD template.
2. **Decompose** -- Break the epic into 3-8 implementation stories, documented in the HIGH_LEVEL_PLAN.md Progress Summary and Story Descriptions sections.
3. **Specify & Design** -- For each story, create a numbered subfolder (e.g., `001-story-name/`) containing STORY.md (requirements + acceptance criteria), DESIGN.md (technical design + test strategy), and SUBTASKS.md (TDD task breakdown).
4. **Implement** -- Execute each story using the TDD red-green-refactor cycle tracked in SUBTASKS.md. Update the epic's HIGH_LEVEL_PLAN.md Shared Context section as cross-story knowledge emerges.
5. **Validate** -- Verify all epic success criteria are met. Run the epic's acceptance tests. Update HIGH_LEVEL_PLAN.md status to DONE and fill in Retrospective Notes.

Stories live in `specs/ENN-epic-name/NNN-story-name/` folders, nested under their parent epic. Each epic folder contains a `HIGH_LEVEL_PLAN.md` that tracks the story breakdown and shared context. See [SDD Guide](../README.md) for templates and governance.

### 2.3 Dependency Notation

- **Hard dependency** (must complete first): shown as "Requires: E*N*"
- **Soft dependency** (benefits from but does not strictly require): shown as "After: E*N* (soft)"
- **No dependency**: the epic can start as soon as the team is available

### 2.4 Cross-Cutting Concerns

Performance engineering, security controls, observability, accessibility compliance, and licensing checks are **not** separate epics. They are embedded in every epic as part of the "done" definition. Specifically:

- Every epic must satisfy the relevant quality gates from [07 -- Testing Strategy](./07-testing-strategy.md) Section 5
- Every epic must comply with the security policies from [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Sections 3-4
- Every epic producing user-facing output must pass accessibility checks per [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 5
- Every epic adding dependencies must pass `cargo deny` license checks per [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 4

### 2.5 Test Infrastructure Progression

Test infrastructure is built incrementally alongside feature work, following the phase-by-phase rollout in [07 -- Testing Strategy](./07-testing-strategy.md) Section 13:

| Phase | Test Infrastructure Milestone |
|-------|------------------------------|
| 0 | GitHub Actions CI (Stages 1-4), Xvfb, Mesa llvmpipe, cargo nextest, cargo clippy + fmt, cargo deny + audit, Vitest + React Testing Library, axe-core, tauri-driver, absolute benchmark thresholds, coverage reporting (informational), manual keyboard navigation + Orca screen reader tests |
| 1 | Self-hosted benchmark runner, Criterion regression baselines, Wayland headless CI, SBOM generation, coverage dashboard |
| 2 | macOS CI jobs, TTS latency benchmarks, VoiceOver manual test protocol, performance profiling CI |
| 3+ | Windows CI, NVDA/JAWS coexistence testing, fuzz testing, reproducible builds, plugin security testing |

---

## 3. Dependency Map

### 3.1 Epic Dependency Graph

```
Phase 0                Phase 1                Phase 2              Phase 3              Phase 4
(Months 1-3)           (Months 4-6)           (Months 7-9)         (Months 10-14)       (Months 15-20)

E1 Scaffolding
 |
 +---> E2 X11 Capture + GPU
 |      |
 |      +---> E3 Input Tracking ---+---> E5 Lens/Docked ---+
 |      |                          |                        |
 |      +---> E6 Visual Pipeline --+                        +---> E9 Linux Packaging
 |      |                          |                        |
 |      +--------------------------)---> E8 Wayland --------+
 |                                 |                        |
 +---> E4 Control Panel -----------+---> E7 Focus/Keys -----+
                                   |
                                   +--------> E10 TTS Core -----> E11 TTS UX ---+
                                   |                                            |
                                   +------> E12 macOS --------------------------+
                                                                                |
                                              E13 Font Re-rendering <-----------+
                                              E14 OCR + AI <--- E11             |
                                              E15 OpenBSD <--- E9               |
                                              E16 Advanced Mag <--- E5, E6, E12 |
                                                                                |
                                              E17 Windows Core <--- E16 --------+
                                              E18 Windows Full <--- E17
                                              E19 Plugins <--- E18
                                              E20 Enterprise <--- E19
```

### 3.2 Dependency Table

| Epic | Hard Dependencies | Soft Dependencies | Earliest Start |
|------|-------------------|-------------------|----------------|
| E1 | None | -- | Month 1, Week 1 |
| E2 | E1 | -- | Month 1, Week 3 |
| E3 | E2 | -- | Month 2, Week 3 |
| E4 | E1 | E3 (soft: engine running helps integration) | Month 2, Week 1 |
| E5 | E3 | E4 (soft: UI controls) | Month 4, Week 1 |
| E6 | E2 | E4 (soft: UI controls) | Month 4, Week 1 |
| E7 | E4 | -- | Month 4, Week 1 |
| E8 | E2 | E5, E6 (soft: all X11 modes stable first) | Month 5, Week 1 |
| E9 | E8 | E5, E6, E7 (soft: all Linux features) | Month 6, Week 1 |
| E10 | E1 | E3 (soft: integration context) | Month 7, Week 1 |
| E11 | E10, E7 | E4 (soft: control panel) | Month 8, Week 1 |
| E12 | E9, E11 | -- | Month 8, Week 3 |
| E13 | E12 | -- | Month 10, Week 1 |
| E14 | E11 | -- | Month 10, Week 1 |
| E15 | E9 | -- | Month 10, Week 1 |
| E16 | E5, E6, E12 | -- | Month 12, Week 1 |
| E17 | E16 | -- | Month 15, Week 1 |
| E18 | E17 | -- | Month 16, Week 3 |
| E19 | E18 | -- | Month 18, Week 1 |
| E20 | E19 | -- | Month 19, Week 1 |

### 3.3 Critical Path

The longest sequential chain determines the minimum project duration:

```
E1 (3w) -> E2 (4w) -> E8 (4w) -> E9 (3w) -> E12 (5w) -> E16 (4w) -> E17 (5w) -> E18 (4w) -> E19 (4w) -> E20 (4w)
= 40 weeks (~10 months)
```

The product roadmap allocates 20 months, providing substantial schedule margin (~10 months). This margin absorbs:
- Parallel epics that share team members and cannot fully overlap
- Discovery work within epics (especially E13 font re-rendering, which has significant research risk)
- Platform-specific issues discovered during porting
- Community feedback integration between phases

---

## 4. Phase 0: Foundation (Months 1-3)

### 4.1 Epic 1 -- Project Scaffolding, Platform Traits & CI/CD

**Summary:** Bootstrap the entire development environment: Cargo workspace with five crate stubs, all six platform abstraction trait definitions with mock implementations, the shared error type hierarchy, core data types, and a fully operational CI/CD pipeline. This epic produces no user-facing functionality but is the prerequisite for every subsequent epic. It is the only "infrastructure-only" epic in the roadmap; its deliverable to the team is the ability to begin parallel feature work.

**Scope:**

*Included:*
- Cargo workspace root manifest with workspace-level dependency declarations ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 2)
- Five crate stubs: `luminos-core`, `luminos-platform`, `luminos-gpu`, `luminos-tts`, `luminos-app` ([01 -- System Architecture](./01-system-architecture.md) Section 7.1)
- Rust 2024 edition configuration, build profiles (dev, release, dist, CI) ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 4)
- Six platform abstraction trait definitions: `ScreenCapture`, `FocusTracker`, `TtsEngine`, `WindowManager`, `InputMonitor`, `AudioOutput` ([02 -- Platform Abstraction](./02-platform-abstraction.md) Sections 2-3)
- Mock implementations for all six traits in `luminos-platform/src/mock/` ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 7)
- `LuminosError` top-level hierarchy in `luminos-core/src/error.rs`; platform-specific error types in `luminos-platform/src/error.rs` ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 4)
- Core data types: `CaptureFrame`, `AppState`, `AppSettings` schema, `PixelFormat`, `MagnificationMode`, `TrackingMode` ([01 -- System Architecture](./01-system-architecture.md) Sections 4-5, [05 -- Control Panel](./05-control-panel.md) Section 3)
- GitHub Actions CI pipeline: build matrix, `cargo nextest`, `cargo clippy`, `cargo fmt`, `cargo deny`, `cargo audit` ([07 -- Testing Strategy](./07-testing-strategy.md) Section 4)
- Pre-commit hook configuration (optional, recommended)
- `LICENSE` (GPL-3.0-or-later), `CHANGELOG.md` initialization

*Excluded:*
- Any platform-specific backend implementation (those are Epic 2+)
- Frontend scaffolding (that is Epic 4)
- Tauri setup (that is Epic 4)

**Dependencies:** None (first epic).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Compiling Cargo workspace (`cargo build` passes on all five crates) | CI green |
| D2 | All six trait definitions with doc-comments matching [02 -- Platform Abstraction](./02-platform-abstraction.md) | Code review |
| D3 | Mock implementations pass unit tests for every trait method | `cargo nextest run -p luminos-platform` |
| D4 | `LuminosError` top-level hierarchy in `luminos-core/src/error.rs` and platform-specific error types in `luminos-platform/src/error.rs` compile with `From` conversions between them | Unit tests |
| D5 | CI pipeline runs on every PR: build, test, lint, license check, vulnerability scan | GitHub Actions workflow |
| D6 | Build profiles produce correct optimization levels | `cargo build --profile dist` succeeds |

**User-Perceivable Value:** None directly. This epic enables the team to begin working on all subsequent epics in parallel.

**Key Risks:**
- Trait definitions may need revision once real platform backends are implemented. Mitigation: traits are designed from validated research ([Tech Stack Evaluation](../TECH_STACK_EVALUATION.md)); revision is expected and acceptable as long as the mock implementations are updated in sync.
- CI runner configuration may require iteration (especially Xvfb and Mesa llvmpipe for future GPU tests). Mitigation: start with build+lint+unit-test only; add GPU-dependent CI stages in Epic 2.

**Estimated Duration:** 3 weeks (1.5 sprints)

**Success Criteria:**
- [ ] `cargo build --workspace` compiles with zero warnings
- [ ] `cargo nextest run --workspace` passes all mock-based unit tests
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo deny check` passes (all dependencies GPLv3-compatible)
- [ ] CI pipeline runs end-to-end on a PR and reports status
- [ ] A new contributor can clone the repo, run `cargo build`, and get a clean result

**Primary Docs:** [01 -- System Architecture](./01-system-architecture.md) Section 7, [02 -- Platform Abstraction](./02-platform-abstraction.md) Sections 2-4 and 7, [07 -- Testing Strategy](./07-testing-strategy.md) Sections 4.1-4.4, [08 -- Build and Distribution](./08-build-and-distribution.md) Sections 2 and 4

---

### 4.2 Epic 2 -- X11 Screen Capture & GPU Magnification

**Summary:** Implement the end-to-end rendering pipeline on Linux X11: capture screen content via `xcap`, upload it to a GPU texture, apply bilinear magnification via a WGSL shader, and present the result in a transparent always-on-top winit overlay window at 60fps. This is the first visual proof that the architecture works -- a user sees magnified screen content rendered in real-time.

**Scope:**

*Included:*
- `ScreenCapture` trait implementation for Linux X11 via `xcap` ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.1)
- `CaptureFrame` data pipeline: capture region calculation, pixel format handling (`Bgra8` primary) ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 2)
- `WindowManager` trait implementation for Linux X11 via `winit`: transparent, always-on-top, borderless overlay window ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.1)
- wgpu device and surface initialization (Vulkan backend on Linux) ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 3)
- Double-buffered GPU texture upload from `CaptureFrame` ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 4)
- WGSL bilinear magnification shader ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 6)
- Full-screen zoom mode rendering (1.5x-20x) ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 7)
- Frame pacing with vsync (Fifo present mode default) ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 8)
- `FrameTimings` ring buffer for performance measurement ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 8.3)
- CI additions: Xvfb for headless X11, Mesa llvmpipe for shader compilation tests ([07 -- Testing Strategy](./07-testing-strategy.md) Section 4.5)
- Integration test: capture → render → verify output frame properties

*Excluded:*
- Input handling and cursor tracking (Epic 3)
- Lens and docked modes (Epic 5)
- Color filters and cursor enhancement (Epic 6)
- Bicubic interpolation (Epic 6)
- XShm capture optimization (Epic 8 or Phase 1 enhancement)

**Dependencies:** Requires E1 (trait definitions, workspace, CI).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | `ScreenCapture` impl captures full-screen content on X11 | Integration test on Xvfb |
| D2 | Winit overlay window renders transparently on top of other windows | Manual verification + screenshot test |
| D3 | Captured content displayed magnified at configurable zoom level (1.5x-20x) | Visual inspection + shader unit tests |
| D4 | Frame rate sustains 60fps at 2x zoom on integrated GPU | `FrameTimings` P99 < 20ms in CI benchmark |
| D5 | Double-buffered texture upload prevents visible tearing | Visual inspection |

**User-Perceivable Value:** A user launches Luminos on a Linux X11 desktop and sees their screen magnified at a chosen zoom level, rendered smoothly at 60fps in a transparent overlay. This is the core value proposition made visible for the first time.

**Key Risks:**
- `xcap` X11 capture performance at high resolutions (non-SHM path). Mitigation: benchmark early; XShm optimization is planned for Phase 1. At high zoom levels, the capture region is smaller, mitigating the bottleneck.
- wgpu Vulkan backend issues on specific Linux GPU drivers. Mitigation: Mesa llvmpipe as CI fallback; test on Intel, AMD, and NVIDIA drivers during development.
- Overlay window transparency may behave differently across X11 window managers. Mitigation: test on GNOME (Mutter), KDE (KWin), and i3/Sway X11 mode.

**Estimated Duration:** 4 weeks (2 sprints)

**Success Criteria:**
- [ ] `cargo nextest run -p luminos-platform --test x11_capture` passes on Xvfb
- [ ] `cargo nextest run -p luminos-gpu` passes (shader compilation, texture upload)
- [ ] Frame time P99 < 20ms at 2x zoom on CI runner (Mesa llvmpipe)
- [ ] Overlay window is transparent and always-on-top on at least two X11 WMs
- [ ] Zoom levels 1.5x, 5x, 10x, 20x all render without artifacts

**Primary Docs:** [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.1, [03 -- Rendering Pipeline](./03-rendering-pipeline.md) Sections 2-8, [07 -- Testing Strategy](./07-testing-strategy.md) Sections 4.5 and 10

---

### 4.3 Epic 3 -- Input Tracking & Interactive Magnification

**Summary:** Make the magnifier interactive. Implement global input monitoring (mouse position, keyboard events), viewport tracking that follows the cursor with smooth panning, and keyboard shortcuts for zoom control. After this epic, a user can actually *use* the magnifier -- moving around the screen, zooming in and out, and toggling magnification on/off.

**Scope:**

*Included:*
- `InputMonitor` trait implementation for Linux X11 via `rdev` ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.1)
- Viewport tracking engine: cursor-follow mode with configurable dead zone ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 7, [01 -- System Architecture](./01-system-architecture.md) Section 4.4)
- Smooth panning when cursor approaches viewport edges ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 7)
- `ArcSwap<AppState>` integration: render thread reads state lock-free every frame ([01 -- System Architecture](./01-system-architecture.md) Section 6.4)
- Global keyboard shortcuts: zoom in, zoom out, toggle magnification, reset zoom ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.1)
- `EventLoopProxy` integration: input events wake the winit render loop ([01 -- System Architecture](./01-system-architecture.md) Section 6.5)

*Excluded:*
- Focus tracking via AT-SPI2 (Epic 7)
- Text caret tracking (Epic 7)
- Configurable keybindings (Epic 7) -- Note: the [Product Strategy](../PRODUCT_STRATEGY.md) Section 7.1 lists keyboard shortcuts as 'Configurable' in Phase 0 (P0). This roadmap delivers hardcoded defaults in Phase 0 and defers configurability to Phase 1 (Epic 7) to reduce Phase 0 scope to the minimum viable magnifier.
- Lens/docked viewport behavior (Epic 5)

**Dependencies:** Requires E2 (overlay window, render loop, GPU pipeline).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Mouse position continuously tracked and fed to viewport engine | Unit test + integration test with simulated mouse events |
| D2 | Magnification viewport follows cursor smoothly at 60fps | Visual inspection + panning latency benchmark |
| D3 | Keyboard shortcuts change zoom level and toggle magnification | Integration test via `xdotool` on Xvfb |
| D4 | `ArcSwap<AppState>` reads are lock-free on the render thread | Benchmark: state read < 100ns |

**User-Perceivable Value:** A user launches Luminos, moves their mouse, and the magnified view follows smoothly. They press a hotkey to zoom in or out, and the magnification level changes instantly. They can toggle magnification on/off. The magnifier is now usable as a daily tool.

**Key Risks:**
- `rdev` may have latency or missed events on certain X11 configurations. Mitigation: benchmark input latency; prepare fallback to raw X11 event monitoring via `x11rb` if needed.
- Smooth panning algorithm may feel sluggish or jerky. Mitigation: parameterize the dead zone and acceleration curve; plan for user-configurable values in Phase 1.

**Estimated Duration:** 3 weeks (1.5 sprints)

**Success Criteria:**
- [ ] Mouse movement updates viewport position within 1 frame (< 16.67ms)
- [ ] Panning is visually smooth (no jitter or snapping) at all zoom levels
- [ ] All four keyboard shortcuts work on X11 (zoom in, zoom out, toggle, reset)
- [ ] `ArcSwap` state update from input thread is visible to render thread on the next frame
- [ ] No dropped frames during rapid mouse movement (P99 frame time < 20ms)

**Primary Docs:** [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.1 (InputMonitor), [03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 7, [01 -- System Architecture](./01-system-architecture.md) Sections 4.4, 6.4-6.5

---

### 4.4 Epic 4 -- Tauri Control Panel & Settings Persistence

**Summary:** Build the Tauri 2.0 control panel shell: a webview window with React UI that communicates with the Rust engine via typed IPC. Implement the Phase 0 IPC commands, a minimal settings UI (zoom slider, mode selector, frame timing readout), system tray integration, and settings persistence to `config.toml`. After this epic, users have a graphical interface for controlling their magnifier and their preferences survive application restarts.

**Scope:**

**Phase attribution note:** [05 -- Control Panel](./05-control-panel.md) Section 1.3 assigns settings persistence to Phase 1. This roadmap pulls it into Phase 0 because settings persistence is essential for daily dogfooding -- users cannot be expected to reconfigure the magnifier after every restart during Phase 0 validation.

*Included:*
- Tauri 2.0 application setup with dual-window architecture (webview + native overlay) ([01 -- System Architecture](./01-system-architecture.md) Section 3.3, [05 -- Control Panel](./05-control-panel.md) Section 1)
- `tauri-specta` v2 IPC type generation: `Builder`, `collect_commands!`, TypeScript bindings export ([05 -- Control Panel](./05-control-panel.md) Section 2.2)
- `LuminosHandle` managed state: `ArcSwap<AppState>`, `ConfigManager`, `EventLoopProxy`, `TtsSender` (placeholder) ([05 -- Control Panel](./05-control-panel.md) Section 4.1)
- Phase 0 IPC commands: `get_current_settings`, `set_zoom_level`, `set_magnification_mode`, `toggle_magnification`, `get_frame_timings` ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- TypeScript/React project scaffolding: Vite, Zod schemas, Zustand stores ([05 -- Control Panel](./05-control-panel.md) Sections 3 and 5)
- React shell: `App`, `HydrationGate`, `Shell` with sidebar navigation, `MagnificationPage` with zoom slider and mode selector ([05 -- Control Panel](./05-control-panel.md) Section 6)
- Settings hydration on startup (fetch `get_current_settings`, populate Zustand store) ([05 -- Control Panel](./05-control-panel.md) Section 5.4)
- IPC event subscriptions: `zoom_changed`, `mode_changed` (hotkey-originated) ([05 -- Control Panel](./05-control-panel.md) Section 2.4)
- System tray icon with minimize-to-tray behavior ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.1)
- `ConfigManager`: load/save `config.toml` in `~/.config/luminos/` ([01 -- System Architecture](./01-system-architecture.md) Section 5.4, [05 -- Control Panel](./05-control-panel.md) Section 2.3)
- `save_settings`, `reset_settings` IPC commands ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- Frontend unit tests (Vitest + React Testing Library), `axe-core` accessibility check ([07 -- Testing Strategy](./07-testing-strategy.md) Sections 3.2 and 11)
- Tauri capability file (minimal permissions) ([06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 3)

*Excluded:*
- Color filter, cursor enhancement, display controls (Epic 6)
- Speech controls (Epic 11)
- Profile management (Epic 9)
- Keybinding configuration page (Epic 7)
- Diagnostics page with historical chart (Epic 16)

**Dependencies:** Requires E1 (workspace, core types). Soft dependency on E3 (a running engine makes integration testing more meaningful, but the control panel can be developed against mock backends).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Tauri webview window opens alongside the magnification overlay | Manual verification |
| D2 | Zoom slider changes magnification level in real-time | IPC integration test via `tauri-driver` |
| D3 | Mode selector switches between full-screen (and placeholder lens/docked) | IPC integration test |
| D4 | Frame timing readout displays current P99 frame time | IPC integration test |
| D5 | Settings persist to `config.toml` and reload on next launch | Unit test (write/read cycle) |
| D6 | System tray icon appears; minimize-to-tray works | Manual verification |
| D7 | `tauri-specta` generates valid TypeScript bindings | CI build succeeds |
| D8 | All React components pass `axe-core` accessibility audit (zero violations) | Vitest + axe-core |

**User-Perceivable Value:** A user opens the Luminos control panel, adjusts their zoom level with a slider, and sees the magnification change instantly. They close and reopen the app, and their settings are preserved. They can minimize to the system tray to keep Luminos running unobtrusively.

**Key Risks:**
- Tauri 2.0 dual-window setup (webview + native winit window in the same process) may have coordination issues. Mitigation: the architecture is validated in the tech stack evaluation; this is the first real integration test.
- `tauri-specta` v2 may not support all Rust types used in IPC. Mitigation: manual TypeScript types as fallback, validated in integration tests ([05 -- Control Panel](./05-control-panel.md) Section 2.2).

**Estimated Duration:** 3 weeks (1.5 sprints)

**Success Criteria:**
- [ ] Control panel opens, hydrates from engine state, and renders without errors
- [ ] Zoom slider round-trips through IPC: UI → Rust → `ArcSwap` → render thread → next frame
- [ ] Settings file written to `~/.config/luminos/config.toml` on save
- [ ] Settings file read and applied on application startup
- [ ] TypeScript bindings match Rust command signatures (CI generation check)
- [ ] All frontend components pass `axe-core` with zero violations
- [ ] `tauri-driver` IPC integration tests pass in CI

**Primary Docs:** [05 -- Control Panel](./05-control-panel.md) Sections 1-6, [01 -- System Architecture](./01-system-architecture.md) Sections 3.3, 4.6-4.7, 5.4, 6.5, 9.4, [08 -- Build and Distribution](./08-build-and-distribution.md) Section 5

---

### 4.5 Phase 0 Summary

**Phase 0 delivers a functional screen magnifier on Linux X11** -- a user can launch Luminos, see their screen magnified at up to 20x zoom, move around the screen with smooth cursor tracking, adjust settings via a graphical control panel, and have their preferences persist across sessions.

| Metric | Target | Epic |
|--------|--------|------|
| Frame rate | 60fps (P99 < 20ms) | E2 |
| Zoom range | 1.5x - 20x full-screen | E2 |
| Input latency | < 1 frame (16.67ms) | E3 |
| Settings persistence | Survives restart | E4 |
| CI pipeline | Build + test + lint + license | E1 |

**Phase 0 total estimated duration:** 13 weeks (~3.25 months). Epics E2 and E4 can overlap partially (E4 starts after E1, not E2), reducing the wall-clock time.

**Note:** The [Product Strategy](../PRODUCT_STRATEGY.md) Section 7.1 includes 'automated releases' as part of the Phase 0 CI/CD P0 requirement. This roadmap delivers CI build+test+lint in Phase 0 (Epic 1) and defers release automation and packaging to Phase 1 (Epic 9). Developers building from source can use Luminos during Phase 0; packaged distribution begins at the Phase 1 gate.

**Phase 0 exit criteria:** All E1-E4 success criteria met. A user can install from source on Linux X11, use the magnifier daily, and configure it via the control panel. This is the minimum viable product for internal dogfooding.

---

## 5. Phase 1: Core Magnification (Months 4-6)

### 5.1 Epic 5 -- Lens & Docked Magnification Modes

**Summary:** Implement the two remaining magnification modes: lens (a movable magnifying glass that follows the cursor) and docked (a split-screen view with the magnified region pinned to one edge). This epic extends the rendering pipeline to support multiple viewport geometries and adds mode switching through both the control panel and keyboard shortcuts.

**Scope:**

*Included:*
- Lens mode rendering: movable rectangular or elliptical magnification region following the cursor ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 7)
- Configurable lens dimensions (width, height) and shape (rectangle, ellipse) ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- Docked mode rendering: magnified content pinned to a configurable edge (top, bottom, left, right) with adjustable size percentage (10-90%) ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 7)
- EWMH strut reservation for docked mode (prevents other windows from overlapping the docked region) via `x11rb` ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.1)
- Mode switching IPC: `set_magnification_mode`, `set_dock_edge`, `set_lens_size`, `set_lens_shape` commands ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- Control panel updates: docked mode controls (edge selector, size slider), lens mode controls (dimension sliders, shape selector) -- shown conditionally based on active mode ([05 -- Control Panel](./05-control-panel.md) Section 6.1)
- Keyboard shortcut: cycle mode (full-screen → lens → docked) ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.2)

*Excluded:*
- Split-screen view with original + magnified side-by-side (Epic 16)
- Multi-monitor considerations (Epic 16)

**Dependencies:** Requires E3 (input tracking, viewport engine). Soft dependency on E4 (UI controls for mode switching).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Lens mode follows cursor with configurable rectangle/ellipse shape | Visual inspection + integration test |
| D2 | Docked mode renders magnified content on all four edges | Integration test: set edge → verify window geometry |
| D3 | EWMH struts reserve screen space in docked mode | Verify via `xprop` that strut hints are set correctly |
| D4 | Mode cycling hotkey rotates through all three modes | Integration test via `xdotool` |
| D5 | Control panel mode-specific controls show/hide correctly | Frontend test (RTL) |

**User-Perceivable Value:** A user chooses how they want to magnify: full-screen immersion, a floating lens they can move around, or a docked strip along one edge. They switch modes with a single keypress.

**Key Risks:**
- Lens ellipse clipping requires a specialized shader or stencil buffer. Mitigation: start with rectangular lens; ellipse can be implemented as a shader mask.
- Docked mode strut reservation may not work on all X11 window managers. Mitigation: test on GNOME, KDE, and tiling WMs; fall back to window positioning without struts if needed.

**Estimated Duration:** 3 weeks (1.5 sprints)

**Success Criteria:**
- [ ] All three magnification modes render correctly at zoom levels 1.5x, 5x, 10x, 20x
- [ ] Mode switching completes within 1 frame (no visible flicker)
- [ ] Lens mode tracks cursor smoothly (same P99 < 20ms target as full-screen)
- [ ] Docked mode resizes correctly when edge or size percentage changes
- [ ] Control panel UI updates reactively when mode changes via hotkey

**Primary Docs:** [03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 7, [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.1 (EWMH struts), [05 -- Control Panel](./05-control-panel.md) Section 2.3

---

### 5.2 Epic 6 -- Visual Enhancement Pipeline

**Summary:** Implement the full color filter pipeline and cursor enhancement features that make magnification usable for a wide range of vision conditions. This includes color inversion, smart inversion, grayscale, high-contrast presets, brightness/contrast adjustment, enlarged cursor, crosshairs, halo, and the find-cursor locator animation. Also upgrades the interpolation shader from bilinear to bicubic for crisper magnified content.

**Scope:**

*Included:*
- WGSL color filter shader pipeline: inversion, smart inversion, grayscale, high-contrast schemes (white-on-black, yellow-on-blue, green-on-black), brightness/contrast adjustments, custom 4x4 color matrix ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 6.3)
- Cursor enhancement: enlarged cursor overlay, crosshairs, halo/spotlight effect ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 6.4)
- Find-cursor locator animation (triggered by hotkey) ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.2)
- WGSL cursor overlay shader ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 6.4)
- Bicubic interpolation shader (upgrade from bilinear) ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 6.2)
- Anti-aliased rendering at high zoom levels ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.2)
- IPC commands: `set_color_filter`, `set_cursor_config`, `set_interpolation_mode` ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- Control panel `DisplayPage`: color filter panel (filter type, brightness, contrast sliders), cursor enhancement panel, rendering controls (interpolation mode) ([05 -- Control Panel](./05-control-panel.md) Section 6.1)

*Excluded:*
- Font re-rendering (Epic 13)
- Present mode and frame rate target controls (included as simple pass-through in this epic; full diagnostics in Epic 16)

**Dependencies:** Requires E2 (GPU pipeline, shader infrastructure). Soft dependency on E4 (UI controls).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | All six color filter types render correctly | Shader unit tests with reference images |
| D2 | Brightness/contrast sliders affect rendered output | Visual inspection + shader tests |
| D3 | Enlarged cursor, crosshairs, and halo render on top of magnified content | Visual inspection |
| D4 | Find-cursor animation plays on hotkey press | Integration test |
| D5 | Bicubic interpolation visibly improves text clarity over bilinear | Visual comparison (documented) |
| D6 | Control panel display page controls all visual enhancements | IPC integration tests |

**User-Perceivable Value:** A user with glaucoma selects a yellow-on-blue high-contrast scheme and sees content transformed to their preferred palette. A user with AMD enables cursor crosshairs to locate their pointer. Text appears sharper at high zoom thanks to bicubic interpolation.

**Key Risks:**
- Smart inversion (preserving images while inverting UI) is algorithmically complex. Mitigation: start with a simple heuristic (invert if pixel brightness > threshold); refine in later phases.
- Multi-pass shader pipeline may impact frame time. Mitigation: benchmark each filter individually; combine passes where possible.

**Estimated Duration:** 3 weeks (1.5 sprints)

**Success Criteria:**
- [ ] Color filter rendering matches reference images for all six types
- [ ] Cursor enhancements (crosshairs, halo) are visible at all zoom levels
- [ ] Bicubic interpolation passes visual quality comparison against bilinear
- [ ] Frame time P99 < 20ms with color filter + cursor enhancement both active
- [ ] All display controls round-trip through IPC correctly

**Primary Docs:** [03 -- Rendering Pipeline](./03-rendering-pipeline.md) Sections 6.2-6.4, [05 -- Control Panel](./05-control-panel.md) Sections 2.3 and 6.1

---

### 5.3 Epic 7 -- Focus Tracking & Keybinding Configuration

**Summary:** Implement accessibility-aware focus tracking so the magnifier follows keyboard focus and text caret in addition to the mouse cursor. Add a configurable keybinding system so users can customize all keyboard shortcuts. This epic bridges magnification and accessibility -- the magnifier becomes usable for keyboard-only users and integrates with Linux accessibility infrastructure.

**Scope:**

**Phase attribution note:** [05 -- Control Panel](./05-control-panel.md) Section 1.3 assigns the keybinding configuration page to Phase 2. This roadmap pulls it forward to Phase 1 because configurable keybindings are integral to the keyboard-accessible experience that focus tracking (also Phase 1) requires. The Phase 1 delivery includes the configuration UI; Phase 2 adds the TTS-related hotkey actions.

*Included:*
- `FocusTracker` trait implementation for Linux via AT-SPI2 (`atspi` crate) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.1)
- Keyboard focus tracking: magnification follows the currently focused UI element ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.2)
- Text caret tracking: magnification follows the cursor position within text fields ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.2)
- Tracking mode switching: cursor, focus, text caret (via IPC and hotkey) ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- `set_tracking_mode` IPC command ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- Configurable keybinding system: in-memory keybinding map, `get_keybindings`, `set_keybinding`, `reset_keybindings` IPC commands ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- Control panel `KeybindingsPage`: table of all actions with key capture input ([05 -- Control Panel](./05-control-panel.md) Section 6.1)
- Keybinding persistence in `config.toml` ([01 -- System Architecture](./01-system-architecture.md) Section 5.4)

*Excluded:*
- macOS focus tracking via AXUIElement (Epic 12)
- Windows focus tracking via UI Automation (Epic 18)
- Application-specific profiles (Epic 20)

**Dependencies:** Requires E4 (control panel, settings persistence).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Focus tracking follows keyboard navigation in GTK and Qt applications | Manual test with Firefox, GNOME Terminal, KDE apps |
| D2 | Text caret tracking follows cursor in text editors | Manual test with gedit, Kate |
| D3 | Tracking mode switches smoothly between cursor/focus/caret | Integration test |
| D4 | Keybinding configuration page captures key combinations | Frontend test + IPC integration test |
| D5 | Custom keybindings persist across restarts | Settings round-trip test |

**User-Perceivable Value:** A keyboard-only user tabs through a web form and the magnifier follows their focus automatically. A user customizes their zoom-in shortcut from the default to a key combination that doesn't conflict with their other tools.

**Key Risks:**
- AT-SPI2 D-Bus interface may have inconsistent implementations across toolkits (GTK 3 vs GTK 4 vs Qt). Mitigation: test against multiple toolkits; implement graceful degradation (fall back to cursor tracking if focus events are unreliable).
- Global keybinding registration on X11 may conflict with desktop environment shortcuts. Mitigation: document conflicts; allow unbinding.

**Estimated Duration:** 3 weeks (1.5 sprints)

**Success Criteria:**
- [ ] Focus tracking works with GTK 3, GTK 4, and Qt 5 applications
- [ ] Text caret tracking updates viewport within 100ms of caret movement
- [ ] All nine hotkey actions are configurable via the keybinding page
- [ ] Key capture input correctly identifies modifier combinations (Ctrl+Shift+Plus, etc.)
- [ ] Custom keybindings survive application restart (loaded from config.toml)

**Primary Docs:** [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.1 (FocusTracker), [05 -- Control Panel](./05-control-panel.md) Sections 2.3 and 6.1, [01 -- System Architecture](./01-system-architecture.md) Section 5.4

---

### 5.4 Epic 8 -- Wayland Display Support

**Summary:** Port the screen capture and window management layers to Linux Wayland, the modern display server replacing X11 on most Linux desktops. This epic implements the Wayland `ScreenCapture` backend via PipeWire and XDG Desktop Portal, adapts the overlay window for Wayland compositors, and validates all existing magnification features (full-screen, lens, docked, color filters, cursor enhancement) on Wayland. After this epic, Luminos works on both X11 and Wayland -- covering essentially all Linux desktop configurations.

**Scope:**

*Included:*
- `ScreenCapture` trait implementation for Linux Wayland via `ashpd` (XDG Desktop Portal) + PipeWire ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.2)
- `WindowManager` trait implementation for Wayland: layer-shell or compositor-specific overlay window ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.2)
- Runtime display server detection: `XDG_SESSION_TYPE`, `WAYLAND_DISPLAY`, `DISPLAY` environment variable inspection ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 1.3)
- `InputMonitor` validation/adaptation for Wayland (rdev Wayland support) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.2)
- All Phase 0-1 magnification features validated on Wayland: full-screen, lens, docked, color filters, cursor enhancement, focus tracking, keybindings
- CI addition: Wayland headless compositor (`weston --headless` or `cage`) ([07 -- Testing Strategy](./07-testing-strategy.md) Section 13.2)
- XShm capture optimization for X11 path (opportunistic inclusion -- improves X11 capture performance at low zoom levels) ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.2)

*Excluded:*
- macOS or Windows display support (Epics 12, 17)
- OpenBSD display support (Epic 15)

**Dependencies:** Requires E2 (X11 capture and GPU pipeline -- same GPU pipeline is reused). Soft dependency on E5 and E6 (testing all modes on Wayland is more meaningful after they exist).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Screen capture works on Wayland via PipeWire | Integration test on headless Wayland |
| D2 | Overlay window renders correctly on GNOME (Mutter) Wayland | Manual verification |
| D3 | Overlay window renders correctly on KDE (KWin) Wayland | Manual verification |
| D4 | All three magnification modes work on Wayland | Feature matrix test |
| D5 | Runtime display server detection selects correct backend | Unit test with mock environment variables |
| D6 | XShm capture optimization improves X11 performance (if included) | Before/after benchmark |

**User-Perceivable Value:** A user on a modern Linux desktop (GNOME 46, KDE Plasma 6, Sway) can use Luminos without being forced to run X11 or Xwayland. This is critical -- Wayland is the default on most Linux distributions as of 2025-2026, and all existing standalone Linux magnifiers (KMag, Magnus, xzoom) are X11-only and broken on Wayland.

**Key Risks:**
- PipeWire screen capture requires user permission (portal dialog). Mitigation: document the permission flow; ensure the permission dialog is accessible.
- Wayland overlay window behavior varies between compositors (Mutter, KWin, wlroots). Mitigation: test on all three compositor families; document known limitations.
- `rdev` Wayland support may be incomplete. Mitigation: verify capability before the epic starts; prepare `libinput`-based fallback if needed.

**Estimated Duration:** 4 weeks (2 sprints)

**Success Criteria:**
- [ ] Screen capture delivers frames at >= 30fps on Wayland (PipeWire path)
- [ ] Overlay window is transparent and above other windows on GNOME and KDE Wayland
- [ ] Docked mode strut reservation or equivalent works on Wayland
- [ ] Focus tracking via AT-SPI2 works identically on X11 and Wayland
- [ ] Runtime detection correctly selects Wayland backend when `WAYLAND_DISPLAY` is set
- [ ] CI includes Wayland headless tests

**Primary Docs:** [02 -- Platform Abstraction](./02-platform-abstraction.md) Sections 1.3, 8.2, [07 -- Testing Strategy](./07-testing-strategy.md) Section 13.2, [03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 10.2 (XShm)

---

### 5.5 Epic 9 -- Linux Packaging & Release Automation

**Summary:** Package Luminos for distribution on Linux: .deb (Debian/Ubuntu), .rpm (Fedora/RHEL), and AppImage (universal). Set up the release automation pipeline in GitHub Actions, including artifact building, espeak-ng bundling, voice model download mechanisms, and GPG signing. After this epic, a Linux user can install Luminos from a downloaded package without building from source.

**Scope:**

*Included:*
- .deb package configuration (Tauri bundler) ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 8)
- .rpm package configuration (Tauri bundler) ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 8)
- AppImage configuration ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 8)
- espeak-ng binary and data file bundling per package format ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 6)
- Voice model on-demand download mechanism: first-run download of Kokoro Q8 model, verification via SHA-256 ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 8, [08 -- Build and Distribution](./08-build-and-distribution.md) Section 7)
- Desktop entry file, application icon, MIME type registration ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 8)
- GitHub Actions release workflow: build matrix, artifact upload, GitHub Releases publishing ([07 -- Testing Strategy](./07-testing-strategy.md) Section 5.3)
- GPG signing for Linux artifacts ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 9)
- SBOM generation via `cargo-cyclonedx` ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 12)
- Basic profile save/load (user-created profiles) as part of settings finalization ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- `list_profiles`, `save_profile`, `load_profile`, `delete_profile` IPC commands ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- Control panel profiles page (basic: save current, load, delete) ([05 -- Control Panel](./05-control-panel.md) Section 6.1)

*Excluded:*
- Flatpak and snap packaging -- The [Product Strategy](../PRODUCT_STRATEGY.md) Section 7.2 lists these as Phase 1 P1. They are deferred because Tauri's native bundler does not produce Flatpak or snap packages ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 8). Custom packaging scripts are required and will be contributed as a follow-up to Epic 9, tracked separately. The three formats delivered (.deb, .rpm, AppImage) cover the majority of Linux desktop users.
- macOS .dmg packaging (Epic 12)
- Windows .msi / NSIS packaging (Epic 18)
- OpenBSD ports (Epic 15)
- Built-in condition-based profile presets (Epic 16)
- Profile import/export (Epic 20)

**Dependencies:** Requires E8 (Wayland support must be in place for Linux packages to cover both display servers). Soft dependency on E5, E6, E7 (all Linux features should be stable before packaging).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | .deb installs cleanly on Ubuntu 24.04+ | Manual install test |
| D2 | .rpm installs cleanly on Fedora 40+ | Manual install test |
| D3 | AppImage runs on Ubuntu, Fedora, and Arch without installation | Manual run test |
| D4 | Desktop entry appears in application menu after installation | Manual verification |
| D5 | Voice model downloads on first TTS use (placeholder until Epic 10 integrates TTS) | Unit test for download/verify mechanism |
| D6 | Release workflow produces signed artifacts on GitHub Releases | CI workflow run |
| D7 | SBOM attached to release artifacts | CI verification |
| D8 | User can save, load, and delete profiles | IPC integration test |

**User-Perceivable Value:** A Linux user downloads a .deb or AppImage from the GitHub Releases page, installs it, and launches Luminos from their application menu. No build tools required. They can save their current settings as a named profile and switch between profiles.

**Key Risks:**
- AppImage bundling with Tauri may require custom scripts for espeak-ng data files. Mitigation: test early; Tauri's native bundler handles .deb and .rpm well but AppImage may need extra work.
- GPG key management for signing. Mitigation: follow the interim signing key approach from [08 -- Build and Distribution](./08-build-and-distribution.md) Section 9.

**Estimated Duration:** 3 weeks (1.5 sprints)

**Success Criteria:**
- [ ] All three package formats install and run on fresh OS installations
- [ ] Application binary size < 50MB (excluding voice models)
- [ ] espeak-ng binary and data files are correctly bundled and accessible at runtime
- [ ] Release workflow completes end-to-end (build → test → package → sign → publish)
- [ ] Profile save/load persists to `~/.config/luminos/profiles/`
- [ ] SBOM generated in CycloneDX JSON format

**Primary Docs:** [08 -- Build and Distribution](./08-build-and-distribution.md) Sections 6-9, [07 -- Testing Strategy](./07-testing-strategy.md) Section 5.3, [05 -- Control Panel](./05-control-panel.md) Sections 2.3 (profile commands) and 6.1 (profiles page)

---

### 5.6 Phase 1 Summary

**Phase 1 delivers a complete magnification solution for Linux** -- full-screen, lens, and docked modes with color filters, cursor enhancement, focus tracking, configurable keybindings, and installable packages for both X11 and Wayland.

| Metric | Target | Epic |
|--------|--------|------|
| Magnification modes | 3 (full-screen, lens, docked) | E5 |
| Color filter types | 6 (invert, smart invert, grayscale, 3 high-contrast) | E6 |
| Interpolation | Bicubic | E6 |
| Focus tracking | Cursor + keyboard focus + text caret | E7 |
| Wayland support | GNOME, KDE, wlroots compositors | E8 |
| Package formats | .deb, .rpm, AppImage | E9 |

**Phase 1 total estimated duration:** 16 weeks (~4 months). Epics E5, E6, and E7 can run in parallel (different subsystems, shared dependency on E3/E4). E8 follows after E5/E6 for complete feature testing. E9 follows E8.

**Phase 1 exit criteria:** All E5-E9 success criteria met. Luminos is installable from packages on Linux X11 and Wayland, with all magnification modes, visual enhancements, and focus tracking working. This is the first externally distributable release (v0.1.0).

---

## 6. Phase 2: TTS + Cross-Platform (Months 7-9)

### 6.1 Epic 10 -- TTS Core Pipeline

**Summary:** Build the text-to-speech engine from input text to audio output: espeak-ng subprocess management (spawn, crash recovery, phoneme generation), sherpa-onnx neural inference via `sherpa-rs` for Kokoro-82M synthesis, cpal audio output with ring buffer, and speech queue management (interrupt, queue, priority). After this epic, the Rust backend can convert arbitrary text to natural-sounding spoken audio at under 200ms latency.

**Scope:**

*Included:*
- `TtsEngine` trait implementation: Kokoro primary backend, Piper fallback backend ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 3)
- `AudioOutput` trait implementation for Linux via cpal ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 3)
- espeak-ng subprocess protocol: spawn, stdio-based communication, phoneme generation, crash detection and automatic restart, graceful shutdown ([04 -- TTS Pipeline](./04-tts-pipeline.md) Sections 3-4)
- Text preprocessing: sentence segmentation, abbreviation expansion, number normalization ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 2)
- sherpa-onnx integration via `sherpa-rs`: model loading, Kokoro-82M inference, audio sample generation ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 5)
- Audio output via cpal: ring buffer, sample rate conversion, device selection ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 6)
- Speech queue: interrupt (stop current, play new), enqueue (queue after current), priority levels ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 7)
- TTS Coordinator thread: orchestrates the pipeline stages, manages back-pressure ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 9)
- `TtsSender` channel for submitting speech requests from any thread ([01 -- System Architecture](./01-system-architecture.md) Section 5.3)
- End-to-end latency measurement and validation against < 200ms target ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 10)
- Voice model loading: Kokoro Q8 ONNX model discovery in `~/.local/share/luminos/voices/` or bundled path ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 8)

*Excluded:*
- Control panel speech UI (Epic 11)
- "Read what I see" and selective TTS features (Epic 11)
- Word-level highlighting (Epic 11)
- Voice model download UI (Epic 11)
- Platform-native TTS fallback (Epic 11)
- macOS and Windows audio backends (Epics 12, 18)

**Dependencies:** Requires E1 (trait definitions). Soft dependency on E3 (integration context -- TTS needs to interact with the magnification engine, but the core pipeline can be developed independently and integrated later).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | espeak-ng subprocess spawns, generates phonemes, and recovers from crashes | Unit test with mock espeak-ng + integration test with real binary |
| D2 | Kokoro-82M model loads and synthesizes audio from phonemes | Integration test with real model file |
| D3 | Audio plays through system speakers via cpal | Manual verification + integration test |
| D4 | `speak("Hello world", interrupt=true)` produces audible speech within 200ms | Latency benchmark |
| D5 | Speech queue correctly interrupts, enqueues, and drains | Unit tests for all queue operations |
| D6 | espeak-ng crash does not crash the main application | Fault injection test (kill espeak-ng process) |

**User-Perceivable Value:** When integrated with Epic 11, users hear their screen content spoken aloud in a natural voice. This epic's standalone value is heard through the `speak_text` IPC command -- text submitted programmatically is spoken.

**Key Risks:**
- sherpa-onnx model loading time may exceed expectations on modest hardware. Mitigation: lazy loading on first speech request; measure and report load time.
- espeak-ng subprocess latency may vary by platform and system load. Mitigation: pipeline phoneme generation ahead of inference; budget 50ms for espeak-ng in the 200ms latency target.
- Kokoro Q8 model quality may not meet expectations for all languages. Mitigation: Piper fallback models are available for 30+ languages with acceptable quality.

**Estimated Duration:** 4 weeks (2 sprints)

**Success Criteria:**
- [ ] End-to-end latency (text in → first audio out) < 200ms for sentences under 50 words
- [ ] espeak-ng crash recovery completes within 500ms (re-spawn + re-initialize)
- [ ] Audio output plays without clicks, pops, or gaps between sentences
- [ ] Speech queue handles rapid interrupt/enqueue sequences without deadlocks
- [ ] Memory usage with model loaded < 600MB (Q8 model ~92MB + inference overhead)
- [ ] All TTS unit tests pass with mock backends (no real espeak-ng or model required)

**Primary Docs:** [04 -- TTS Pipeline](./04-tts-pipeline.md) Sections 2-10, [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 3 (TtsEngine, AudioOutput traits), [01 -- System Architecture](./01-system-architecture.md) Section 5.3

---

### 6.2 Epic 11 -- TTS User Experience & Control Panel Integration

**Summary:** Connect the TTS pipeline to the magnification engine and build the user-facing TTS features: "read what I see" (speak text under magnification focus), selective reading (speak highlighted text), voice selection, speech rate/volume controls, and the complete control panel Speech page. This epic transforms TTS from a backend capability into a user experience -- users interact with speech through intuitive controls and natural workflows.

**Scope:**

*Included:*
- "Read what I see" mode: extract text at the current magnification focus via accessibility APIs (AT-SPI2 `GetText` on focused element) and send to TTS ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.3)
- Selective TTS: user selects text region, triggers "read this" via hotkey, text extracted via clipboard (`arboard`) and spoken ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.3)
- Voice selection and model management: voice list, model discovery, on-demand download with progress indicator ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 8, [05 -- Control Panel](./05-control-panel.md) Section 2.3)
- IPC commands: `list_voices`, `set_voice`, `set_speech_rate`, `set_speech_volume`, `set_model_variant`, `speak_text`, `stop_speech`, `get_tts_status`, `check_espeak_available` ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- IPC events: `tts_status_changed`, `voice_model_loading`, `espeak_status_changed` ([05 -- Control Panel](./05-control-panel.md) Section 2.4)
- Control panel `SpeechPage`: voice selector, speech rate/volume sliders, model variant selector, TTS status indicator, espeak-ng warning banner ([05 -- Control Panel](./05-control-panel.md) Section 6.1)
- Word-level highlighting: synchronize TTS word position with visual highlight in the magnification overlay ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 11, [Product Strategy](../PRODUCT_STRATEGY.md) Section 7.3)
- Hotkeys: "read what I see", "read selection", "stop speech" ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.3)
- Platform-native TTS fallback: `speech-dispatcher` on Linux for when Kokoro model is unavailable ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 12)

*Excluded:*
- TTS core pipeline internals (Epic 10)
- macOS AVSpeech fallback (Epic 12)
- Windows SAPI fallback (Epic 18)

**Dependencies:** Requires E10 (TTS core pipeline). Requires E7 (focus tracking -- needed for "read what I see"). Soft dependency on E4 (control panel for speech page).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | "Read what I see" hotkey speaks text from focused element | Integration test: focus text field → hotkey → verify speech output |
| D2 | "Read selection" hotkey speaks clipboard text | Integration test: select text → hotkey → verify speech output |
| D3 | Voice selector lists available Kokoro speakers and Piper models | IPC integration test |
| D4 | Model download shows progress in control panel | IPC event test |
| D5 | Speech rate and volume sliders affect playback in real-time | Manual verification |
| D6 | TTS status indicator shows idle/speaking/loading/error states | IPC event test + frontend test |
| D7 | Word-level highlighting tracks spoken word in overlay | Visual inspection |
| D8 | speech-dispatcher fallback works when Kokoro model is absent | Integration test on system without model |

**User-Perceivable Value:** A user hovers over a paragraph and presses a hotkey -- the text is read aloud in a natural voice. They select a block of text in a document and trigger "read selection." They choose between different voices and adjust speaking speed. When TTS encounters an error, the status indicator tells them what happened. This is the integration of magnification and speech that no existing Linux tool provides.

**Key Risks:**
- AT-SPI2 `GetText` may not return meaningful text from all applications (especially Electron apps, games). Mitigation: fallback to clipboard-based extraction; document known limitations.
- Word-level highlighting requires precise timing alignment between TTS output and visual rendering. Mitigation: start with sentence-level highlighting; word-level is a progressive enhancement.

**Estimated Duration:** 3 weeks (1.5 sprints)

**Success Criteria:**
- [ ] "Read what I see" correctly extracts and speaks text from GTK and Qt applications
- [ ] "Read selection" works with text selected in any application (clipboard path)
- [ ] Voice switching completes within 2 seconds (model swap)
- [ ] Control panel speech page passes `axe-core` accessibility audit
- [ ] espeak-ng absence produces a user-visible warning (not a crash)
- [ ] Word-level highlighting is visually synchronized with speech (within 100ms)

**Primary Docs:** [04 -- TTS Pipeline](./04-tts-pipeline.md) Sections 8, 11-12, [05 -- Control Panel](./05-control-panel.md) Sections 2.3-2.4 and 6.1, [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.1 (FocusTracker `GetText`)

---

### 6.3 Epic 12 -- macOS Platform Support

**Summary:** Port all Luminos functionality to macOS: screen capture via ScreenCaptureKit, window management via NSWindow, focus tracking via AXUIElement, input monitoring, audio output validation, and wgpu Metal backend validation. This epic also includes macOS-specific TTS integration (espeak-ng on macOS, AVSpeech fallback), .dmg packaging, and Apple Developer ID code signing. After this epic, Luminos delivers the full magnification + TTS experience on macOS.

**Scope:**

*Included:*
- `ScreenCapture` implementation for macOS via `xcap` (ScreenCaptureKit path) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.3)
- `WindowManager` implementation for macOS: NSWindow with transparent, always-on-top, borderless, click-through behavior ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.3)
- `FocusTracker` implementation for macOS via AXUIElement (`objc2` bindings) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.3)
- `InputMonitor` validation for macOS (rdev macOS path) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.3)
- `AudioOutput` validation for macOS (cpal CoreAudio backend) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.3)
- wgpu Metal backend validation: all shaders (magnification, color filter, cursor) compile and render correctly ([03 -- Rendering Pipeline](./03-rendering-pipeline.md))
- All magnification modes (full-screen, lens, docked) working on macOS
- All color filters and cursor enhancements working on macOS
- All TTS features working on macOS: espeak-ng subprocess, Kokoro inference, cpal audio
- AVSpeech TTS fallback for macOS ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 12)
- macOS .dmg packaging via Tauri bundler ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 8)
- Apple Developer ID code signing ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 9)
- macOS CI job addition to GitHub Actions ([07 -- Testing Strategy](./07-testing-strategy.md) Section 13.3)
- macOS Screen Recording permission handling (user must grant manually) ([07 -- Testing Strategy](./07-testing-strategy.md) Section 4.6)

*Excluded:*
- macOS-specific magnification features beyond what Linux supports (no macOS-only enhancements in this epic)
- VoiceOver coexistence testing depth (basic verification only; comprehensive testing is ongoing)
- macOS App Store distribution (not planned for initial release)

**Dependencies:** Requires E9 (Linux fully packaged -- ensures all features are stable before porting). Requires E11 (TTS user experience -- macOS port includes TTS).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Screen capture works on macOS via ScreenCaptureKit | Integration test on macOS CI runner |
| D2 | Overlay window renders transparently on macOS (Metal backend) | Manual verification |
| D3 | All three magnification modes work on macOS | Feature matrix test |
| D4 | Focus tracking works with macOS Cocoa applications | Manual test with Safari, TextEdit, Xcode |
| D5 | TTS speaks text on macOS with Kokoro voice | Manual verification |
| D6 | AVSpeech fallback produces speech when Kokoro model is absent | Integration test |
| D7 | .dmg installer produces signed, distributable package | Manual install on clean macOS |
| D8 | macOS CI job builds and tests successfully | GitHub Actions green |

**User-Perceivable Value:** A macOS user downloads a .dmg, installs Luminos, and gets the same magnification + TTS experience as Linux users. They grant Screen Recording permission once and the magnifier works across all applications. This is the first time a cross-platform magnification + TTS tool exists for macOS that is not tied to the built-in Zoom + VoiceOver.

**Key Risks:**
- macOS Screen Recording permission cannot be granted programmatically. Mitigation: clear first-run instructions; detect missing permission and display a user-friendly error.
- AXUIElement focus tracking requires Accessibility permission. Mitigation: same approach as Screen Recording -- detect and guide user.
- Apple code signing requires an Apple Developer account ($99/year). Mitigation: budget for this; unsigned builds are available as fallback.
- GitHub Actions macOS runners do NOT auto-grant Screen Recording permission ([07 -- Testing Strategy](./07-testing-strategy.md) Section 4.6 note). Mitigation: CI tests mock the capture layer; real capture is tested manually.

**Estimated Duration:** 5 weeks (2.5 sprints)

**Success Criteria:**
- [ ] All Phase 0-2 features pass on macOS (full-screen, lens, docked, color filters, cursor enhancement, focus tracking, TTS)
- [ ] Frame rate sustains 60fps on Apple M-series integrated GPU
- [ ] TTS latency < 200ms on macOS
- [ ] .dmg installs cleanly on macOS 14+ (Sonoma or Tahoe)
- [ ] Code signing passes Apple's notarization (or at minimum, Gatekeeper allows execution with user approval)
- [ ] macOS CI job runs unit tests and build validation

**Primary Docs:** [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.3, [03 -- Rendering Pipeline](./03-rendering-pipeline.md) (Metal validation), [04 -- TTS Pipeline](./04-tts-pipeline.md) Section 12 (AVSpeech), [08 -- Build and Distribution](./08-build-and-distribution.md) Sections 8-9

---

### 6.4 Phase 2 Summary

**Phase 2 delivers integrated magnification + text-to-speech on Linux and macOS** -- the core product promise. For the first time, users have a single open-source tool that magnifies and speaks on two platforms.

| Metric | Target | Epic |
|--------|--------|------|
| TTS latency | < 200ms trigger-to-audio | E10 |
| TTS voices | Kokoro (8 languages) + Piper fallback (30+) | E10 |
| "Read what I see" | Text under focus spoken on hotkey | E11 |
| macOS support | Full feature parity with Linux | E12 |
| Platforms | Linux X11, Linux Wayland, macOS | E12 |

**Phase 2 total estimated duration:** 12 weeks (~3 months). Epics E10 and E12 can partially overlap (E10 builds the TTS engine while macOS platform work begins on capture/rendering). E11 depends on E10 completion.

**Phase 2 exit criteria:** All E10-E12 success criteria met. Luminos is distributable on Linux (.deb, .rpm, AppImage) and macOS (.dmg) with magnification + TTS. This is the first publicly announced release (v0.2.0).

---

## 7. Phase 3: Advanced Magnification + AI (Months 10-14)

### 7.1 Epic 13 -- Font Re-Rendering Engine

**Summary:** Research and implement font re-rendering: detect text regions in captured screen content, match the original font, and re-render text at the magnified resolution using system font APIs. This is the key competitive differentiator -- commercial tools like ZoomText (xFont) and SuperNova (TrueFonts) charge $900+ for this capability, and no open-source tool has ever implemented it. This epic has significant research risk; the scope may be reduced to a single platform or a subset of applications based on feasibility findings.

**Scope:**

*Included:*
- Text region detection in captured content: identify areas containing rendered text ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 11)
- Platform font API integration: FreeType/fontconfig on Linux, CoreText on macOS ([03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 11)
- Font matching algorithm: infer font family, weight, size, and color from captured pixels
- Text extraction from accessibility API metadata (AT-SPI2 on Linux, AXUIElement on macOS) as a higher-fidelity alternative to pixel-based detection
- Re-rendering text at magnified resolution with proper hinting and anti-aliasing
- Shader integration: composite re-rendered text over the magnified pixel content
- Quality validation: side-by-side comparison of pixel magnification vs. font re-rendering
- Configuration: enable/disable font re-rendering, confidence threshold for auto-detection

*Excluded:*
- OpenBSD font re-rendering (deferred -- depends on FreeType availability)
- Windows font re-rendering (deferred to Phase 4 -- different font stack)
- Re-rendering for non-Latin scripts (initial implementation targets Latin scripts)

**Dependencies:** Requires E12 (multi-platform -- font re-rendering benefits from running on both Linux and macOS for validation).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Text region detection identifies text areas in captured content | Benchmark: precision/recall on test screenshots |
| D2 | Font matching correctly identifies font family and size for common system fonts | Test against known font rendering in GTK, Qt, and Cocoa applications |
| D3 | Re-rendered text is visually sharper than pixel magnification at 5x+ zoom | Visual comparison (documented, reviewed by low-vision testers if available) |
| D4 | Font re-rendering toggle works via control panel | IPC integration test |
| D5 | Re-rendering does not degrade frame rate below 30fps when active | Performance benchmark |

**User-Perceivable Value:** A user zooming to 8x sees crisp, readable text instead of blurry pixel magnification. Headings, body text, and UI labels are sharp and clear. This is the feature that makes Luminos competitive with $900+ commercial tools.

**Key Risks:**
- **High research risk.** Font matching from pixel data is an unsolved problem in the open-source space. Accuracy may be low for unusual fonts or complex layouts. Mitigation: prioritize accessibility API text extraction (which provides font metadata) over pure pixel analysis; accept partial coverage (common fonts in common toolkits).
- Frame time impact: re-rendering text at magnified size adds CPU/GPU work per frame. Mitigation: only re-render when zoom level exceeds a threshold (e.g., 4x); cache re-rendered glyphs; allow users to disable the feature.
- Platform font API complexity varies significantly. Mitigation: start with Linux (FreeType is well-documented); macOS CoreText integration follows.

**Estimated Duration:** 5 weeks (2.5 sprints) -- includes research spike in the first week

**Success Criteria:**
- [ ] Font re-rendering produces measurably sharper text than pixel magnification (validated by image comparison metric)
- [ ] Re-rendering works for at least 3 common font families (system sans, serif, monospace) on Linux and macOS
- [ ] Frame time with re-rendering active: P99 < 33ms (30fps acceptable for this feature; users can disable for 60fps)
- [ ] Feature can be toggled on/off without restart
- [ ] Accessibility API text extraction path works for GTK, Qt, and Cocoa applications

**Primary Docs:** [03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 11, [Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4

---

### 7.2 Epic 14 -- OCR & AI-Powered Accessibility

**Summary:** Integrate optical character recognition (OCR) to extract text from images, scanned documents, and applications that lack accessibility API support. Connect the OCR output to the TTS pipeline so users can hear text that would otherwise be invisible to the magnifier. Optionally integrate an on-device AI model for image description (describing charts, photos, and diagrams via TTS).

**Scope:**

*Included:*
- OCR integration: Tesseract on Linux/OpenBSD, Vision framework on macOS ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4)
- OCR-to-TTS pipeline: captured image region → OCR text extraction → TTS speech ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4)
- User-triggered OCR: hotkey or control panel action to OCR the current magnification viewport
- OCR language selection (match TTS language)
- AI image description: on-device model (small vision-language model) describes visual content ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4)
- Image description triggered by hotkey or automatic when focus is on an image element
- Control panel UI for OCR language and AI description settings

*Excluded:*
- Windows OCR backend (deferred to Phase 4)
- Real-time continuous OCR (too expensive computationally; OCR is user-triggered)
- Training custom OCR models

**Dependencies:** Requires E11 (TTS user experience -- OCR output feeds into TTS).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Tesseract extracts text from captured screen region on Linux | Integration test with known text image |
| D2 | Vision framework extracts text on macOS | Integration test |
| D3 | OCR text is spoken via TTS pipeline on hotkey | End-to-end test: capture → OCR → TTS |
| D4 | AI image description produces meaningful descriptions of common image types | Manual evaluation with test images |
| D5 | OCR language selection matches TTS voice language | Integration test |

**User-Perceivable Value:** A user encounters a scanned PDF or an image with text. They press a hotkey and hear the text read aloud. They hover over a chart or photo and hear a description of what it contains. This extends Luminos's accessibility reach beyond text-based applications to visual content.

**Key Risks:**
- Tesseract OCR accuracy varies significantly by image quality and font. Mitigation: pre-process captured region (contrast enhancement, deskew) before OCR.
- On-device vision-language models are large (1-4GB) and may not fit within the 4GB RAM budget alongside the TTS model. Mitigation: OCR and image description use the GPU when available; model is optional and downloaded on demand; smaller quantized models preferred.
- AI image description quality may not meet user expectations. Mitigation: clearly label as "AI-generated description"; allow users to disable.

**Estimated Duration:** 4 weeks (2 sprints)

**Success Criteria:**
- [ ] OCR accuracy > 90% on clean printed text (standard fonts, sufficient contrast)
- [ ] OCR-to-TTS latency < 2 seconds for a viewport-sized region
- [ ] AI image description produces reasonable output for photos, charts, and UI screenshots
- [ ] OCR does not crash on malformed or low-quality input (graceful degradation)
- [ ] Memory usage with OCR + TTS models loaded stays within 4GB budget

**Primary Docs:** [Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4, [04 -- TTS Pipeline](./04-tts-pipeline.md) (integration point)

---

### 7.3 Epic 15 -- OpenBSD Platform Support

**Summary:** Port Luminos to OpenBSD by adapting the Linux X11 backends. OpenBSD uses X11 via xenocara (a fork of X.Org), so most X11 capture and window management code is shared with the Linux X11 path. This epic validates the shared code, implements any OpenBSD-specific adjustments, and produces an OpenBSD ports package. This is the lowest-effort platform port due to high code reuse.

**Scope:**

*Included:*
- `ScreenCapture` validation/adaptation for OpenBSD X11 (shared `linux_x11` code via `common/` module) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.4)
- `WindowManager` validation for OpenBSD X11 ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.4)
- `InputMonitor` validation for OpenBSD (rdev + X11 events) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.4)
- `FocusTracker` for OpenBSD: AT-SPI2 if available, fallback to cursor-only tracking ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.4)
- `AudioOutput` validation on OpenBSD (cpal sndio backend) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.4)
- espeak-ng availability and subprocess operation on OpenBSD
- wgpu Vulkan backend validation on OpenBSD
- OpenBSD ports-style packaging ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 8)
- Self-hosted CI runner for OpenBSD (no GitHub-hosted OpenBSD runners exist) ([07 -- Testing Strategy](./07-testing-strategy.md) Section 13.2) (note: [07 -- Testing Strategy](./07-testing-strategy.md) Section 13.2 lists this as Phase 1 infrastructure, but it is deferred to Phase 3 when OpenBSD code is first implemented)

*Excluded:*
- OpenBSD Wayland (does not exist; OpenBSD uses X11)
- OpenBSD-specific magnification features
- Font re-rendering on OpenBSD (deferred)

**Dependencies:** Requires E9 (Linux packaging -- confirms the build/package pipeline works).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Luminos compiles on OpenBSD without modification to shared X11 code | CI build on self-hosted runner |
| D2 | Screen capture works on OpenBSD X11 | Integration test |
| D3 | All magnification modes render correctly | Manual verification |
| D4 | TTS pipeline operates (espeak-ng + Kokoro) | Manual verification |
| D5 | OpenBSD ports package installs cleanly | Manual install test |

**User-Perceivable Value:** An OpenBSD user -- a community with essentially zero dedicated accessibility tools -- can install Luminos and get the same magnification + TTS experience as Linux users. This fulfills the product strategy's commitment to serving underserved platforms.

**Key Risks:**
- cpal sndio backend may have quirks on OpenBSD. Mitigation: test early; OpenAL fallback if needed.
- Self-hosted CI runner setup requires a physical or virtual OpenBSD machine. Mitigation: use a VM on existing CI infrastructure.
- Some Linux-specific dependencies (e.g., atspi for focus tracking) may not be available on OpenBSD. Mitigation: graceful degradation to cursor-only tracking.

**Estimated Duration:** 3 weeks (1.5 sprints)

**Success Criteria:**
- [ ] `cargo build --workspace` compiles on OpenBSD without platform-specific workarounds beyond `#[cfg]` blocks
- [ ] Full-screen, lens, and docked modes render at 60fps on OpenBSD X11
- [ ] TTS pipeline produces audio on OpenBSD
- [ ] OpenBSD self-hosted CI runs build + test on every PR
- [ ] Ports package installs and runs on OpenBSD -current

**Primary Docs:** [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.4, [08 -- Build and Distribution](./08-build-and-distribution.md) Section 8 (OpenBSD ports), [07 -- Testing Strategy](./07-testing-strategy.md) Section 13.2

---

### 7.4 Epic 16 -- Advanced Magnification Features

**Summary:** Implement the advanced magnification features that bring Luminos to feature parity with mid-tier commercial tools: multi-monitor support, split-screen view (original + magnified side by side), mini-map navigator for spatial orientation, and condition-based profile presets. Also completes the diagnostics page in the control panel with historical frame timing charts and system information.

**Scope:**

*Included:*
- Multi-monitor support: display enumeration, per-monitor capture, cross-monitor panning ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4)
- Split-screen view: original content on one side, magnified on the other ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4)
- Mini-map navigator: small overview showing full screen with viewport indicator for spatial orientation ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4)
- Condition-based profile presets: AMD (high-contrast, warm colors), glaucoma (enhanced peripheral, reduced brightness), diabetic retinopathy (contrast + magnification), cataracts (brightness + contrast) ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4)
- Setup wizard: first-run experience that asks about vision condition and suggests a profile ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4)
- Control panel `DiagnosticsPage`: historical frame timing chart (polling `get_frame_timings`), `get_system_info` command, system info panel ([05 -- Control Panel](./05-control-panel.md) Section 6.1)
- GPU preference selector (`set_gpu_preference` IPC) ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- Built-in profile presets shipped with the application ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- Mouse pointer customization: size, color, shape ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4)

*Excluded:*
- Application-specific auto-switching profiles (Epic 20)
- Content-aware AI magnification (Phase 5 -- future vision)

**Dependencies:** Requires E5 and E6 (magnification modes and visual enhancements must be complete). Requires E12 (multi-platform validation -- features should work on both Linux and macOS).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Multi-monitor: magnification follows cursor across monitors | Integration test with dual-monitor setup |
| D2 | Split-screen view renders original + magnified content | Visual inspection |
| D3 | Mini-map shows current viewport position relative to full screen | Visual inspection |
| D4 | At least 4 condition-based profile presets shipped | Preset file validation |
| D5 | Setup wizard runs on first launch and configures initial profile | Manual verification |
| D6 | Diagnostics page displays live frame timing chart | IPC polling test |
| D7 | GPU preference selector switches between integrated/discrete GPU | Restart-triggered test |

**User-Perceivable Value:** A professional user with multiple monitors can magnify content across their entire workspace. A user with AMD selects the AMD-optimized profile and gets tuned color/contrast settings. A new user launches Luminos for the first time and a wizard helps them choose the right configuration for their vision condition.

**Key Risks:**
- Multi-monitor capture may have performance implications (capturing from multiple displays). Mitigation: only capture the display containing the cursor/focus; pan between displays on demand.
- Setup wizard UX requires careful design for accessibility (must be usable before magnification is configured). Mitigation: use the control panel webview with large, high-contrast defaults.

**Estimated Duration:** 4 weeks (2 sprints)

**Success Criteria:**
- [ ] Multi-monitor magnification transitions smoothly between displays (< 100ms transition time)
- [ ] Split-screen view maintains 60fps for both original and magnified panels
- [ ] Mini-map viewport indicator updates in real-time as user pans
- [ ] All condition-based presets produce visually distinct, appropriate configurations
- [ ] Setup wizard is keyboard-accessible and passes `axe-core` audit
- [ ] Diagnostics page chart updates at 1Hz with accurate frame timing data

**Primary Docs:** [Product Strategy](../PRODUCT_STRATEGY.md) Section 7.4, [05 -- Control Panel](./05-control-panel.md) Section 6.1, [03 -- Rendering Pipeline](./03-rendering-pipeline.md) (multi-monitor, split-screen rendering)

---

### 7.5 Phase 3 Summary

**Phase 3 delivers advanced features and expands to OpenBSD**, achieving feature parity with mid-tier commercial tools. Font re-rendering is the signature competitive differentiator; OCR extends accessibility to visual content; and multi-monitor/split-screen support serves professional users.

| Metric | Target | Epic |
|--------|--------|------|
| Font re-rendering | Common fonts on Linux + macOS | E13 |
| OCR accuracy | > 90% on clean text | E14 |
| Platforms | Linux, macOS, OpenBSD | E15 |
| Multi-monitor | Cross-display panning | E16 |
| Condition presets | 4+ profiles (AMD, glaucoma, etc.) | E16 |

**Phase 3 total estimated duration:** 16 weeks (~4 months). Epics E13, E14, and E15 can run in parallel (independent subsystems with different dependencies). E16 depends on E5, E6, and E12 but not on E13-E15.

**Phase 3 exit criteria:** All E13-E16 success criteria met. Luminos runs on Linux, macOS, and OpenBSD with advanced magnification features, font re-rendering, OCR, and condition-based profiles. This is the feature-complete release for non-Windows platforms (v0.3.0).

---

## 8. Phase 4: Platform & Ecosystem (Months 15-20)

### 8.1 Epic 17 -- Windows Core Platform Support

**Summary:** Implement the core platform backends for Windows: screen capture via `windows-capture` (DXGI Desktop Duplication) with `xcap` as cross-platform fallback, window management via Win32 APIs, input monitoring, and wgpu DX12 backend validation. This epic brings the magnification overlay to Windows -- a user sees magnified screen content rendered at 60fps. TTS and full integration follow in Epic 18.

**Scope:**

*Included:*
- `ScreenCapture` implementation for Windows: `windows-capture` (DXGI Desktop Duplication) as primary, `xcap` as fallback ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.5, [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) Section 4.2)
- `WindowManager` implementation for Windows: Win32 overlay window (transparent, always-on-top, borderless, click-through) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.5)
- `InputMonitor` implementation for Windows via rdev ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.5)
- wgpu DX12 backend validation: all shaders compile and render correctly ([03 -- Rendering Pipeline](./03-rendering-pipeline.md))
- All magnification modes (full-screen, lens, docked) working on Windows
- All color filters, cursor enhancements, and interpolation working on Windows
- Multi-monitor support on Windows
- Windows CI job addition to GitHub Actions ([07 -- Testing Strategy](./07-testing-strategy.md) Section 13.4)

*Excluded:*
- Focus tracking via UI Automation (Epic 18)
- TTS on Windows (Epic 18)
- Screen reader coexistence (Epic 18)
- Windows packaging (Epic 18)

**Dependencies:** Requires E16 (advanced features complete -- ensures all features exist before porting).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Screen capture works on Windows via DXGI Desktop Duplication | Integration test on Windows CI |
| D2 | Overlay window renders transparently on Windows 11 | Manual verification |
| D3 | All magnification modes work on Windows | Feature matrix test |
| D4 | DX12 shader pipeline renders correctly (magnification, color filters, cursor) | Shader tests on Windows CI |
| D5 | Multi-monitor capture works on Windows | Multi-monitor test |

**User-Perceivable Value:** A Windows user runs Luminos and sees magnified screen content. They can use full-screen, lens, and docked modes with color filters and cursor enhancement. The magnifier is fast and responsive on Windows hardware.

**Key Risks:**
- DXGI Desktop Duplication may require elevated permissions or fail on certain Windows configurations (e.g., Secure Desktop). Mitigation: `xcap` as fallback capture path.
- Win32 transparent overlay window behavior is complex (layered windows, click-through). Mitigation: research prior art in screen annotation tools.
- DX12 shader compilation may produce different results than Vulkan/Metal. Mitigation: reference image tests in CI.

**Estimated Duration:** 5 weeks (2.5 sprints)

**Success Criteria:**
- [ ] Screen capture delivers frames at 60fps on Windows 11 with integrated Intel/AMD GPU
- [ ] Overlay window is transparent and always-on-top on Windows 11
- [ ] All three magnification modes render at P99 < 20ms
- [ ] Color filters and cursor enhancements render identically to Linux/macOS (reference images)
- [ ] Windows CI job builds and tests successfully

**Primary Docs:** [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.5, [03 -- Rendering Pipeline](./03-rendering-pipeline.md) (DX12 validation), [07 -- Testing Strategy](./07-testing-strategy.md) Section 13.4

---

### 8.2 Epic 18 -- Windows Full Integration & Packaging

**Summary:** Complete the Windows port: focus tracking via UI Automation, TTS integration (espeak-ng MSI, SAPI fallback), screen reader coexistence (NVDA, JAWS), and Windows packaging (.msi, NSIS installer). This epic is critical for the Windows ecosystem -- Luminos must not interfere with existing screen readers that Windows users depend on.

**Scope:**

*Included:*
- `FocusTracker` implementation for Windows via UI Automation (`windows` crate) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.5)
- `AudioOutput` validation for Windows (cpal WASAPI backend) ([02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.5)
- TTS on Windows: espeak-ng via MSI installation from GitHub releases (not Chocolatey), Kokoro inference, cpal audio ([04 -- TTS Pipeline](./04-tts-pipeline.md), [08 -- Build and Distribution](./08-build-and-distribution.md) Section 6)
- SAPI (Windows native) TTS fallback ([04 -- TTS Pipeline](./04-tts-pipeline.md) Section 12)
- Screen reader coexistence testing: Luminos must not interfere with NVDA or JAWS operation ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5, [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 5)
- .msi installer via Tauri bundler (WiX) ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 8)
- NSIS installer as alternative ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 8)
- Windows code signing (Authenticode) ([08 -- Build and Distribution](./08-build-and-distribution.md) Section 9)
- GPO support for enterprise deployment (silent install, configuration) ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5)
- NVDA and JAWS coexistence manual test protocol ([07 -- Testing Strategy](./07-testing-strategy.md) Section 11)
- All TTS features (read what I see, selective reading, voice selection) working on Windows

*Excluded:*
- Windows Store distribution (not planned for initial release)
- NSIS cross-compilation from Linux/macOS (not reliable per Tauri docs)

**Dependencies:** Requires E17 (Windows core capture and rendering).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | Focus tracking works with Windows applications (UWP, Win32, Electron) | Manual test with Edge, Notepad, VS Code |
| D2 | TTS produces speech on Windows via Kokoro | Manual verification |
| D3 | SAPI fallback works when Kokoro model is absent | Integration test |
| D4 | Luminos does not interfere with NVDA operation | NVDA coexistence test protocol |
| D5 | Luminos does not interfere with JAWS operation | JAWS coexistence test protocol |
| D6 | .msi installer installs and uninstalls cleanly on Windows 11 | Manual install/uninstall test |
| D7 | Application starts after installation and magnifies correctly | Manual verification |
| D8 | Windows code signing passes SmartScreen (or produces acceptable UX for unsigned) | Manual verification |

**User-Perceivable Value:** A Windows user installs Luminos via a standard .msi installer and gets the full magnification + TTS experience. Their existing NVDA or JAWS screen reader continues to work alongside Luminos. Enterprise IT departments can deploy Luminos via GPO.

**Key Risks:**
- Screen reader coexistence is the highest-risk item in the entire project. Luminos's overlay window may interfere with NVDA/JAWS screen reading. Mitigation: mark the overlay window with `WS_EX_NOACTIVATE` and appropriate UI Automation properties; test extensively with real screen readers.
- UI Automation focus tracking may behave differently from AT-SPI2. Mitigation: budget extra testing time; accept reduced coverage for non-standard applications.
- Windows code signing requires a certificate ($200-400/year). Mitigation: budget for this; unsigned builds trigger SmartScreen warnings but are functional.

**Estimated Duration:** 4 weeks (2 sprints)

**Success Criteria:**
- [ ] All Phase 0-3 features pass on Windows 11 (magnification, TTS, focus tracking, color filters, etc.)
- [ ] NVDA continues to read screen content correctly while Luminos overlay is active
- [ ] JAWS continues to function correctly while Luminos is running
- [ ] .msi installer handles upgrade, clean install, and uninstall correctly
- [ ] TTS latency < 200ms on Windows
- [ ] Enterprise silent install (`msiexec /quiet`) works

**Primary Docs:** [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.5, [04 -- TTS Pipeline](./04-tts-pipeline.md) Section 12 (SAPI), [08 -- Build and Distribution](./08-build-and-distribution.md) Sections 8-9, [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 5

---

### 8.3 Epic 19 -- Plugin Architecture

**Summary:** Design and implement the plugin system that allows community-developed extensions to Luminos: Rust trait-based backend plugins (new capture methods, new TTS engines, new color filters) and Tauri frontend extensions (custom settings panels, new UI features). This epic defines the plugin API, implements the plugin discovery and loading mechanism, documents the API for contributors, and ships an example plugin.

**Scope:**

*Included:*
- Plugin trait system: define `LuminosPlugin` trait with lifecycle hooks (init, shutdown, capabilities) ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5)
- Plugin discovery: scan plugin directory, validate plugin manifest, load shared libraries ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5)
- Plugin sandboxing: capability-based permissions (a color filter plugin cannot access the network)
- Frontend extension points: Tauri-side plugin API for adding settings panels and UI components
- Plugin API documentation (rustdoc + TypeScript docs)
- Example plugin: a custom color filter plugin demonstrating the full plugin lifecycle
- Plugin security model: signing, capability validation ([06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 3)

*Excluded:*
- Community plugin registry (Epic 20)
- Plugin package manager (future)
- Specific community plugins (community-driven post-release)

**Dependencies:** Requires E18 (all platforms supported -- plugin API should cover all platform backends).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | `LuminosPlugin` trait defined with rustdoc documentation | Code review |
| D2 | Plugin discovery loads plugins from `~/.config/luminos/plugins/` | Integration test with example plugin |
| D3 | Example color filter plugin compiles, loads, and applies a custom filter | End-to-end test |
| D4 | Frontend extension point allows adding a custom settings panel | Manual verification |
| D5 | Plugin capability system prevents unauthorized API access | Security test |
| D6 | Plugin API documentation published | Documentation review |

**User-Perceivable Value:** A developer or advanced user can extend Luminos with custom functionality: a new color filter, a specialized TTS engine, or a custom settings panel. The community can contribute plugins without modifying the core codebase.

**Key Risks:**
- Plugin API stability: the API must be stable enough for external developers but flexible enough for future evolution. Mitigation: version the plugin API; start with a narrow surface area.
- Security: loading external code (shared libraries) introduces attack surface. Mitigation: capability-based sandboxing; plugin signing for verified plugins.

**Estimated Duration:** 4 weeks (2 sprints)

**Success Criteria:**
- [ ] Example plugin compiles independently from the Luminos workspace
- [ ] Plugin loads dynamically at runtime without recompilation
- [ ] Capability system blocks unauthorized API calls (test: plugin requests network → denied)
- [ ] Plugin API documentation is sufficient for an external developer to write a plugin
- [ ] Plugin unloading cleans up resources without leaks

**Primary Docs:** [Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5, [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 3

---

### 8.4 Epic 20 -- Enterprise, i18n & Ecosystem

**Summary:** Add enterprise deployment features, internationalization infrastructure, and ecosystem tools: GPO/MDM centralized configuration, application-specific profiles (auto-switch settings per application), configuration import/export, UI localization for 10+ languages, RTL layout support, and a basic community plugin registry. This epic transforms Luminos from a developer tool into an enterprise-ready, globally accessible application.

**Scope:**

*Included:*
- Enterprise deployment: GPO/MDM configuration templates, silent install profiles, centralized configuration management ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5)
- Application-specific profiles: auto-detect active application and switch magnification profile (e.g., higher zoom for code editors, lower for browsers) ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5)
- Configuration import/export: Git-friendly JSON format for sharing settings between machines ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5)
- i18n infrastructure: react-intl (or equivalent) integration, translation file format, contributor translation workflow ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5)
- UI localization: initial 10+ languages targeting top WHO-impacted regions ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5)
- RTL layout support: Arabic, Hebrew, Persian ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5)
- Community plugin registry: curated listing of verified plugins (basic web page or GitHub-based) ([Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5)
- `export_profile`, `import_profile` IPC commands ([05 -- Control Panel](./05-control-panel.md) Section 2.3)
- Control panel import/export controls and i18n language selector

*Excluded:*
- Braille display output (future research)
- USB portable mode (technical feasibility unconfirmed per [Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5)
- Touch/trackpad gestures (P2) -- deferred to a post-v1.0 enhancement or community contribution

**Dependencies:** Requires E19 (plugin architecture -- enterprise features and i18n benefit from the extension system).

**Deliverables:**

| # | Deliverable | Verification |
|---|-------------|--------------|
| D1 | GPO template deploys Luminos with pre-configured settings | Enterprise deployment test |
| D2 | Application-specific profiles auto-switch on app focus change | Integration test: switch between two apps → verify profile change |
| D3 | Configuration export produces valid JSON; import restores settings | Round-trip test |
| D4 | Control panel displays correctly in at least 10 languages | Visual verification per language |
| D5 | RTL layout renders correctly for Arabic | Visual verification |
| D6 | Plugin registry lists available plugins with install instructions | Manual verification |

**User-Perceivable Value:** An enterprise IT department deploys Luminos to 500 workstations with pre-configured profiles via GPO. A developer's Luminos automatically switches to high-zoom + dark mode when VS Code gains focus, and back to moderate-zoom when switching to a browser. A user in Egypt sees the Luminos control panel in Arabic with correct RTL layout. A user in Japan sees it in Japanese.

**Key Risks:**
- GPO/MDM integration requires platform-specific registry or plist management. Mitigation: start with a config file-based approach; GPO template points to a network-accessible config file.
- Application detection varies by platform (procfs on Linux, NSWorkspace on macOS, foreground window title on Windows). Mitigation: implement the simplest reliable method per platform.
- Translation quality for 10+ languages. Mitigation: use professional translators or verified community contributors; machine translation is acceptable for initial draft with human review.

**Estimated Duration:** 4 weeks (2 sprints)

**Success Criteria:**
- [ ] Enterprise deployment: Luminos installs silently with pre-configured settings via GPO
- [ ] Application-specific profiles switch within 500ms of app focus change
- [ ] Configuration export/import round-trips without data loss
- [ ] All UI strings are externalized (no hard-coded English text)
- [ ] RTL layout passes visual review for Arabic
- [ ] At least 10 language translations are shipped

**Primary Docs:** [Product Strategy](../PRODUCT_STRATEGY.md) Section 7.5, [05 -- Control Panel](./05-control-panel.md) Sections 2.3 and 6.1, [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 8 (i18n)

---

### 8.5 Phase 4 Summary

**Phase 4 delivers Windows support, extensibility, and enterprise readiness** -- completing the cross-platform vision and making Luminos deployable in institutional settings.

| Metric | Target | Epic |
|--------|--------|------|
| Windows support | Full feature parity | E17, E18 |
| Screen reader coexistence | NVDA + JAWS verified | E18 |
| Plugin API | Documented, with example plugin | E19 |
| Languages | 10+ UI translations | E20 |
| Enterprise | GPO/MDM deployment | E20 |
| Platforms | Linux, macOS, OpenBSD, Windows | All |

**Phase 4 total estimated duration:** 17 weeks (~4.25 months). Epics are sequential (E17 → E18 → E19 → E20), but E19 and E20 could partially overlap if staffed independently.

**Phase 4 exit criteria:** All E17-E20 success criteria met. Luminos runs on all four target platforms with magnification, TTS, plugins, enterprise deployment, and localized UI. This is the v1.0 release.

---

## 9. Timeline Overview

### 9.1 Month-by-Month Timeline

```
Month  1 |  2 |  3 |  4 |  5 |  6 |  7 |  8 |  9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20
       |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E1 ===|    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E2    |====|====|    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E4    |  ==|==  |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E3    |    |====|==  |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
       |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
       |    |    | ** Phase 0 Gate **   |    |    |    |    |    |    |    |    |    |    |    |
       |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E5    |    |    |  ==|==  |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E6    |    |    |  ==|==  |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E7    |    |    |  ==|==  |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E8    |    |    |    |  ==|==  |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E9    |    |    |    |    |  ==|==  |    |    |    |    |    |    |    |    |    |    |    |    |
       |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
       |    |    |    |    |    | ** Phase 1 Gate **   |    |    |    |    |    |    |    |    |
       |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E10   |    |    |    |    |    |  ==|==  |    |    |    |    |    |    |    |    |    |    |    |
 E11   |    |    |    |    |    |    |  ==|==  |    |    |    |    |    |    |    |    |    |    |
 E12   |    |    |    |    |    |    |    |====|==  |    |    |    |    |    |    |    |    |    |
       |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
       |    |    |    |    |    |    |    |    | ** Phase 2 Gate **   |    |    |    |    |    |
       |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E13   |    |    |    |    |    |    |    |    |  ==|====|    |    |    |    |    |    |    |    |
 E14   |    |    |    |    |    |    |    |    |  ==|==  |    |    |    |    |    |    |    |    |
 E15   |    |    |    |    |    |    |    |    |  ==|==  |    |    |    |    |    |    |    |    |
 E16   |    |    |    |    |    |    |    |    |    |    |  ==|==  |    |    |    |    |    |    |
       |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
       |    |    |    |    |    |    |    |    |    |    |    |    | ** Phase 3 Gate **   |    |
       |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
 E17   |    |    |    |    |    |    |    |    |    |    |    |    |    |====|====|    |    |    |
 E18   |    |    |    |    |    |    |    |    |    |    |    |    |    |    |  ==|==  |    |    |
 E19   |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |  ==|==  |    |
 E20   |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |  ==|==  |
       |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |
       |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    |    | ** v1.0 **
       Phase 0  |  Phase 1  |  Phase 2  |       Phase 3           |       Phase 4           |
```

Legend: `===` = active epic work. Each `=` segment is approximately one week. `**` = phase gate.

### 9.2 Release Schedule

| Release | Phase Gate | Estimated Month | Milestone |
|---------|-----------|-----------------|-----------|
| Internal dogfood | Phase 0 complete | Month 3 | Magnifier usable on X11 by the team |
| v0.1.0-alpha | Phase 1 complete | Month 6 | First external distribution (Linux packages) |
| v0.2.0-beta | Phase 2 complete | Month 9 | Linux + macOS with TTS. First public announcement. |
| v0.3.0-rc | Phase 3 complete | Month 14 | Feature-complete for Linux/macOS/OpenBSD |
| v1.0.0 | Phase 4 complete | Month 20 | All four platforms, plugins, enterprise, i18n |

### 9.3 Schedule Risk

The timeline has approximately 10 months of margin between the critical path (~10 months) and the allocated schedule (20 months). This margin absorbs:

| Risk Factor | Estimated Impact | Likelihood |
|-------------|-----------------|------------|
| Font re-rendering research (E13) takes longer than planned | +2-4 weeks | High |
| Wayland compositor compatibility issues (E8) | +1-2 weeks | Medium |
| Windows screen reader coexistence (E18) requires redesign | +2-3 weeks | Medium |
| macOS permission UX requires multiple iterations | +1-2 weeks | Medium |
| Platform-specific GPU driver issues | +1-2 weeks per platform | Low-Medium |
| Dependency breaking changes (wgpu, Tauri major updates) | +1-3 weeks | Low |

---

## 10. Staffing & Parallel Work

### 10.1 Parallelization Opportunities

The dependency graph reveals several parallelization windows where independent epics can be worked simultaneously:

| Time Window | Parallel Epics | Independence Rationale |
|-------------|---------------|----------------------|
| Month 2-3 | E2 + E4 | E2 (GPU/capture) and E4 (Tauri/UI) touch different crates and subsystems |
| Month 4-5 | E5 + E6 + E7 | E5 (modes), E6 (visual), E7 (focus/keys) modify different pipeline stages |
| Month 10-11 | E13 + E14 + E15 | E13 (fonts), E14 (OCR), E15 (OpenBSD) are entirely independent subsystems |

### 10.2 Team Structure Assumptions

This roadmap assumes the following AI-agent-driven team model, consistent with the [Product Strategy](../PRODUCT_STRATEGY.md) Section 9:

| Role | Count | Responsibilities |
|------|-------|-----------------|
| AI agent (Rust backend) | 2-3 concurrent | Platform backends, GPU pipeline, TTS engine, core engine |
| AI agent (TypeScript frontend) | 1-2 concurrent | React components, Zustand stores, IPC wrappers |
| Human tech lead | 1 | Story decomposition, code review, architectural decisions |
| Human QA/accessibility | 1 (part-time) | Manual accessibility testing, screen reader validation, user testing |

AI agents operate within the SDD framework: they receive STORY.md + DESIGN.md + SUBTASKS.md and implement via TDD. The Rust compiler serves as the primary automated code reviewer. Human review focuses on architectural conformance and accessibility.

### 10.3 Single-Team vs. Multi-Team Execution

The 20-epic roadmap is designed for a **single team** with 2-3 backend and 1-2 frontend AI agents. In this model:
- Phases execute mostly sequentially, with parallel epics within each phase
- Phase 0-1 can run 2-3 epics in parallel
- Phase 3 has the highest parallelization potential (3 independent epics)

If a **second team** is added, the roadmap can be accelerated:
- Team A focuses on the magnification pipeline (E2, E3, E5, E6, E8, E13, E16)
- Team B focuses on integration and platform (E4, E7, E9, E10, E11, E12, E15, E17-E20)
- With two teams, the total timeline could compress to approximately 14-16 months

---

## 11. Phase Gates & Release Criteria

### 11.1 Phase Gate Process

Before advancing to the next phase, the following must be verified:

1. **All epic success criteria met.** Every checkbox in every epic of the current phase is checked.
2. **CI pipeline green.** All automated tests pass on all targeted platforms for the phase.
3. **Performance targets met.** Frame time, memory, TTS latency within documented budgets ([06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 2).
4. **Accessibility audit passed.** Manual screen reader test and keyboard navigation audit completed ([07 -- Testing Strategy](./07-testing-strategy.md) Section 11).
5. **No P0 bugs open.** All critical bugs discovered during the phase are resolved.
6. **Technical debt documented.** Any shortcuts taken are recorded in the risk register with remediation plans.

### 11.2 Phase-Specific Criteria

**Phase 0 → Phase 1 Gate:**
- [ ] Full-screen magnification works on Linux X11 at 60fps (P99 < 20ms)
- [ ] Control panel connects to engine, settings persist
- [ ] CI pipeline runs build + test + lint + license check
- [ ] Binary compiles and runs from source on a fresh Ubuntu 24.04 installation

**Phase 1 → Phase 2 Gate:**
- [ ] All three magnification modes work on X11 and Wayland
- [ ] Color filters, cursor enhancement, focus tracking functional
- [ ] .deb, .rpm, and AppImage packages install and run correctly
- [ ] Manual Orca screen reader coexistence test passes (Linux)

**Phase 2 → Phase 3 Gate:**
- [ ] TTS produces speech at < 200ms latency on Linux and macOS
- [ ] "Read what I see" and selective TTS work on both platforms
- [ ] macOS .dmg installs and runs on macOS 14+
- [ ] VoiceOver coexistence basic test passes (macOS)

**Phase 3 → Phase 4 Gate:**
- [ ] Font re-rendering produces visibly sharper text on Linux and macOS (documented comparison)
- [ ] OCR extracts text from images with > 90% accuracy on clean text
- [ ] OpenBSD build and test passes on self-hosted CI
- [ ] Multi-monitor magnification works on Linux and macOS

**Phase 4 → v1.0 Gate:**
- [ ] All four platforms pass the full feature matrix
- [ ] NVDA and JAWS coexistence tests pass (Windows)
- [ ] Plugin API documented and example plugin functional
- [ ] UI localized in 10+ languages
- [ ] Binary size < 50MB (excluding voice models) on all platforms
- [ ] RAM usage < 4GB under worst-case conditions on all platforms

---

## 12. Cross-References

| Topic | Document | Section |
|-------|----------|---------|
| Product phase definitions | [Product Strategy](../PRODUCT_STRATEGY.md) | Section 7 |
| System architecture and crate structure | [01 -- System Architecture](./01-system-architecture.md) | Sections 3, 7 |
| Platform trait definitions | [02 -- Platform Abstraction](./02-platform-abstraction.md) | Sections 2-3 |
| Per-platform backend specs | [02 -- Platform Abstraction](./02-platform-abstraction.md) | Section 8 |
| GPU rendering pipeline | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | Sections 2-11 |
| TTS pipeline architecture | [04 -- TTS Pipeline](./04-tts-pipeline.md) | Sections 2-12 |
| IPC command catalogue | [05 -- Control Panel](./05-control-panel.md) | Section 2.3 |
| React component architecture | [05 -- Control Panel](./05-control-panel.md) | Section 6 |
| Performance targets and budgets | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 2 |
| Security and privacy | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 3 |
| Licensing compliance | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 4 |
| Accessibility compliance | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 5 |
| CI/CD pipeline | [07 -- Testing Strategy](./07-testing-strategy.md) | Section 4 |
| Quality gates | [07 -- Testing Strategy](./07-testing-strategy.md) | Section 5 |
| Test infrastructure rollout | [07 -- Testing Strategy](./07-testing-strategy.md) | Section 13 |
| Screen reader test protocol | [07 -- Testing Strategy](./07-testing-strategy.md) | Section 11 |
| Cargo workspace configuration | [08 -- Build and Distribution](./08-build-and-distribution.md) | Section 2 |
| Build profiles | [08 -- Build and Distribution](./08-build-and-distribution.md) | Section 4 |
| Platform packaging | [08 -- Build and Distribution](./08-build-and-distribution.md) | Section 8 |
| Code signing | [08 -- Build and Distribution](./08-build-and-distribution.md) | Section 9 |
| Risk analysis | [10 -- Risk Register](./10-risk-register.md) | (planned) |

---

## 13. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-18 | Initial implementation roadmap with 20 epics across 5 phases |
