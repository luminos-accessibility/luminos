# Luminos Technical Strategy

**Status:** DRAFT v1.1
**Date:** 2026-03-15
**Audience:** Engineering team, AI agents, technical auditors, contributors
**Source Documents:** [Product Strategy](../PRODUCT_STRATEGY.md) (v1.3), [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) (FINAL)

---

## Purpose

This technical strategy translates the Luminos product vision into a comprehensive, actionable engineering plan. It defines the system architecture, technology choices, implementation patterns, and delivery roadmap that will guide all development from Phase 0 through Phase 4.

This document set is the **canonical technical reference** for the project. All implementation stories (STORY.md/DESIGN.md/SUBTASKS.md) must conform to the architecture and patterns defined here. Deviations require documented rationale and strategy-level review.

## How to Read This Strategy

The strategy is organized into focused, interconnected documents. Read them in order for a complete picture, or jump to specific sections as needed:

| # | Document | Scope | Audience |
|---|----------|-------|----------|
| 01 | [System Architecture](./01-system-architecture.md) | Overall architecture, dual-window design, component model, data flow | Everyone |
| 02 | [Platform Abstraction](./02-platform-abstraction.md) | Trait definitions, per-platform backends, conditional compilation | Engineers |
| 03 | [Rendering Pipeline](./03-rendering-pipeline.md) | GPU magnification: capture, shaders, frame pacing, zoom modes | Engineers |
| 04 | [TTS Pipeline](./04-tts-pipeline.md) | Text-to-speech architecture, subprocess isolation, voice model management | Engineers |
| 05 | [Control Panel](./05-control-panel.md) | Tauri IPC, React UI, settings, profiles | Frontend engineers |
| 06 | [Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Performance, security, licensing, accessibility, observability, error handling, i18n | Everyone |
| 07 | [Testing Strategy](./07-testing-strategy.md) | Test architecture, CI/CD pipeline, quality gates | Engineers |
| 08 | [Build and Distribution](./08-build-and-distribution.md) | Cargo workspace, packaging, signing, release engineering | Engineers, DevOps |
| 09 | [Implementation Roadmap](./09-implementation-roadmap.md) | Phased milestones, story breakdown, delivery timeline | Everyone |
| 10 | [Risk Register](./10-risk-register.md) | Technical risks, mitigations, monitoring | Everyone |

## Executive Summary

### The Technical Challenge

Luminos must deliver GPU-accelerated screen magnification at 60fps with integrated neural text-to-speech across four platforms (Linux, macOS, OpenBSD, Windows), on hardware as modest as integrated GPUs with 4GB total system RAM. It must coexist with existing screen readers (NVDA, JAWS), respect platform accessibility APIs, and ship under the GPLv3 open-source license.

### The Architecture

Luminos uses a **dual-window architecture** that separates concerns by performance criticality:

1. **Magnification Overlay** -- A native Rust window (winit + wgpu) that captures screen content, applies GPU shader transforms, and renders magnified output at 60fps. This is the performance-critical path and bypasses any web rendering layer entirely.

2. **Control Panel** -- A Tauri 2.0 webview (TypeScript/React) for settings, preferences, and voice selection. Not performance-critical. Uses Tauri's typed IPC to communicate with the Rust backend.

Both windows share a single Rust core engine organized around six platform-abstraction traits (`ScreenCapture`, `FocusTracker`, `TtsEngine`, `WindowManager`, `InputMonitor`, `AudioOutput`), each with per-platform backend implementations selected at compile time.

### Key Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Core language | Rust (2024 edition) | Memory safety, zero-cost abstractions, compiler catches AI-generated code errors |
| GPU rendering | wgpu (v28.0.0) | Cross-platform Metal/DX12/Vulkan, mature (17.9M downloads) |
| Screen capture | xcap (v0.9.1) | Direct X11 support, stable, Apache 2.0 |
| Window management | winit (v0.30.13) | Cross-platform transparent/borderless windows, wgpu integration |
| TTS engine | Kokoro-82M via sherpa-onnx | Near-commercial quality, Apache 2.0 model weights |
| Phonemizer | espeak-ng (subprocess) | GPL-3.0 (project is GPLv3; subprocess retained for crash isolation) |
| Control panel | Tauri 2.0 + React | Lightweight, TypeScript for AI-friendly UI generation |
| Audio output | cpal | Cross-platform, mature |

### Platform Order

Linux X11 first (most underserved audience, simplest capture APIs, zero competition) --> Linux Wayland (future-proof) --> macOS (good built-in Zoom, but users want cross-platform consistency) --> OpenBSD (incremental from Linux X11 work) --> Windows (last -- already has ZoomText, Magnifier, SuperNova).

### Performance Targets

| Metric | Target | Constraint |
|--------|--------|------------|
| Frame rate | 60fps (16ms frame budget) | Integrated GPUs (Intel UHD, Apple M-series, AMD Vega) |
| RAM | < 4GB | Must coexist with user applications on 8GB systems |
| TTS latency | < 200ms trigger-to-audio | Responsive read-aloud experience |
| Binary size | < 50MB (excluding voice models) | Low-bandwidth regions |
| Startup | < 2s to usable magnification | Users depend on the tool from login |

### Critical Path Items

1. **espeak-ng subprocess protocol** -- GPLv3 licensing resolves the legal concern. Subprocess isolation is retained for crash isolation and testability (espeak-ng crashes must not bring down the main process). Misaki G2P remains a nice-to-have for reducing the external dependency.
2. **X11 capture performance** -- Linux X11 is now the starting platform (Phase 0). xcap's non-SHM capture path may bottleneck at low zoom levels and must be validated immediately. XShm backend planned as Phase 1 optimization.
3. **Font re-rendering** -- The key commercial differentiator (Phase 3) requires deep platform font API integration. Research must begin in Phase 1.

## Relationship to Other Documents

```
PRODUCT_STRATEGY.md          -- WHAT to build, WHY, for WHOM
    |
    v
TECH_STACK_EVALUATION.md     -- Technology validation and selection
    |
    v
tech-strategy/ (this)        -- HOW to build it: architecture, patterns, roadmap
    |
    v
specs/NNN-story/STORY.md     -- Individual feature specifications
specs/NNN-story/DESIGN.md    -- Feature-level technical design
specs/NNN-story/SUBTASKS.md  -- TDD execution plan
```

The tech strategy is informed by the product strategy and tech stack evaluation, and constrains all downstream implementation stories. Changes to this strategy require review and versioning.

## Conventions

- **Crate versions** are pinned as of 2026-03-15. Verify against crates.io before implementing.
- **Platform-specific code** uses `#[cfg(target_os = "...")]` conditional compilation.
- **Trait names** match the architecture exactly: `ScreenCapture`, `FocusTracker`, `TtsEngine`, `WindowManager`, `InputMonitor`, `AudioOutput`.
- **Performance claims** cite hardware context. Desktop CPU/GPU benchmarks are not comparable to Raspberry Pi benchmarks.
- **License claims** distinguish code license from model/data license.

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-15 | Initial technical strategy |
| 1.1 | 2026-03-15 | GPLv3 licensing decision; Linux-first platform pivot; OpenBSD added |
