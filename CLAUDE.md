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

**Luminos** is a GPLv3-licensed, cross-platform (Linux/macOS/OpenBSD/Windows) screen magnification + text-to-speech accessibility suite targeting low-vision users. The project has completed its **technical strategy phase** and **Epic E01 (Project Scaffolding, Platform Traits & CI/CD)**. The repository contains the complete product strategy, technology evaluation, and technical strategy documents (10 documents covering architecture through risk management), plus the foundational Rust codebase: 5 crates with trait definitions, mock implementations, error hierarchy, core data types, and 114 unit tests, backed by a GitHub Actions CI pipeline. No user-facing functionality exists yet (that starts in E02).

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

**Dual-window design:**

1. **Control Panel** — Tauri 2.0 webview (TypeScript/React). Settings UI. Not performance-critical.
2. **Magnification Overlay** — Native Rust window via winit + wgpu. GPU-accelerated, transparent, always-on-top. Bypasses webview entirely.

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

## Current Project Phase

**Technical strategy is COMPLETE.** The project is in **Phase 0: Foundation** (Months 1-3).

Phase 0 epics (from `specs/tech-strategy/09-implementation-roadmap.md`):

- **E1:** Project Scaffolding, Platform Traits & CI/CD -- **COMPLETE** (2026-03-28). 5 stories, 53 subtasks, 114 tests passing, clippy clean, fmt clean.
- **E2:** X11 Screen Capture + GPU Rendering (proof-of-concept magnification)
- **E3:** Focus Tracking + Input Monitoring (AT-SPI2, rdev)
- **E4:** Control Panel Foundation (Tauri IPC, React settings UI)

**Next step:** Begin Epic E02 (X11 Screen Capture & GPU Magnification) -- decompose into stories and create STORY.md/DESIGN.md/SUBTASKS.md for each story, referencing the tech strategy docs and risk register.

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
