# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Luminos** is an open-source, cross-platform (macOS/Windows/Linux) screen magnification + text-to-speech accessibility suite targeting low-vision users. The project is currently in the **pre-development research phase** — no application code exists yet. The repository contains product strategy and technical evaluation documents.

## Repository Structure

- `PRODUCT_STRATEGY.md` — Product strategy & roadmap v1.1 (post-audit). The canonical product definition.
- `TECH_STACK_EVALUATION.md` — Technology stack validation report. Contains the **revised** recommended stack (supersedes some choices in the strategy doc).
- `.claude/agent-memory/` — Persistent memory for specialized agents (technical-auditor, technical-research-analyst)

## Architecture (Decided)

**Dual-window design:**
1. **Control Panel** — Tauri 2.0 webview (TypeScript/React). Settings UI. Not performance-critical.
2. **Magnification Overlay** — Native Rust window via winit + wgpu. GPU-accelerated, transparent, always-on-top. Bypasses webview entirely.

**Rendering pipeline:** screen capture → GPU texture → wgpu shader transform → anti-alias → composite → present

**TTS pipeline:** text → espeak-ng subprocess (phonemes, GPL-isolated) → Kokoro ONNX inference (via sherpa-rs) → cpal audio output

## Revised Technology Stack (from TECH_STACK_EVALUATION.md)

Key changes from the original strategy doc:
- **Screen capture:** `xcap` crate (v0.9.1, Apache 2.0) replaces `scap` — provides direct X11 support
- **TTS:** Kokoro-82M via `sherpa-rs`/sherpa-onnx replaces Piper (archived, GPL)
- **Window management:** `winit` explicitly adopted for the magnification overlay
- **Phonemizer:** espeak-ng run as **subprocess** to isolate GPL-3.0 from the main binary
- **Audio output:** `cpal` crate
- **Clipboard:** `arboard` crate

Core unchanged: Rust backend, Tauri 2.0 (control panel), wgpu (GPU), TypeScript+React (UI)

## Platform Development Order

macOS first → Windows → Linux (X11 first, Wayland later)

## Critical Legal Issue

Piper TTS is GPL-3.0 (via espeak-ng dependency). The project's licensing strategy depends on subprocess isolation of espeak-ng. This requires legal review before any TTS integration work.

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

## When Editing Strategy Documents

- All market claims must cite specific sources with dates
- Distinguish between WebAIM Low Vision Survey #1 (2013) and #2 (2018) — they have different stats
- ZoomText max zoom is 36x on current versions (60x was legacy Win 8 only)
- Section 508 references WCAG 2.0 AA, not 2.1
- Crate versions change quickly — always verify against crates.io before citing
