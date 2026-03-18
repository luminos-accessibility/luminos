# Luminos

**See clearly. Hear everything.**

[![Project Status: Research](https://img.shields.io/badge/status-research%20phase-yellow)](specs/PRODUCT_STRATEGY.md)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)
[![Platforms: Linux, macOS, OpenBSD, Windows](https://img.shields.io/badge/platforms-Linux%20%7C%20macOS%20%7C%20OpenBSD%20%7C%20Windows-blue)](#platform-support)

Luminos is a GPLv3-licensed, cross-platform screen magnification and text-to-speech accessibility suite for low-vision users. It combines GPU-accelerated magnification with neural TTS in a single application that works the same way on Linux, macOS, Windows, and OpenBSD.

> **Project Status:** Luminos is in the **pre-development research phase**. Product strategy, technical architecture, and development methodology are defined. No application code exists yet. Contributions to research, design, and planning are welcome.

---

## The Problem

2.2 billion people worldwide have some form of visual impairment ([WHO](https://www.who.int/news-room/fact-sheets/detail/blindness-and-visual-impairment)). Screen magnification users significantly outnumber screen reader users, yet the accessibility software ecosystem has invested disproportionately in screen readers.

Today, a low-vision user faces these choices:

| Option | Cost | Limitation |
|--------|------|------------|
| ZoomText | $905+ | Windows only |
| ZoomText + JAWS (Fusion) | $2,309+ | Windows only |
| Built-in OS magnifiers | Free | No TTS, no cross-platform consistency, limited features |
| NVDA | Free | Screen reader only, no magnification, Windows only |
| Linux magnifiers (KMag, Magnus) | Free | Breaking under Wayland transition, no TTS |
| OpenBSD magnifiers | Free | No dedicated accessibility tools exist |

**There is no cross-platform, open-source, professional-grade screen magnification tool with integrated text-to-speech.**

## The Solution

Luminos fills that gap with:

- **GPU-accelerated magnification** -- 60fps on integrated GPUs, up to 20x zoom with anti-aliased rendering
- **Neural text-to-speech** -- On-device Kokoro TTS (8 languages, with Piper fallback for 30+), <200ms latency
- **Cross-platform consistency** -- Same tool, same keybindings, same experience on Linux, macOS, Windows, and OpenBSD
- **Zero cost, full capability** -- No "community edition" with missing features
- **Licensed under GPLv3** -- Your freedom to use, study, modify, and share is guaranteed
- **Privacy by design** -- All AI inference on-device, no telemetry by default

## Feature Roadmap

Development is organized into 5 phases spanning approximately 20 months:

| Phase | Focus | Key Features |
|-------|-------|-------------|
| **0: Foundation** | Architecture proof on Linux X11 | Screen capture, GPU magnification, basic zoom modes, control panel |
| **1: Core Magnification** | Full magnification on Linux X11 + Wayland | Lens mode, docked mode, cursor enhancement, focus tracking, color filters, Linux packages |
| **2: TTS + Cross-Platform** | Speech + macOS support | Neural TTS, "read what I see", selective reading, macOS full support |
| **3: Advanced + AI** | Commercial feature parity + OpenBSD | Font re-rendering, on-device OCR, AI image description, multi-monitor, OpenBSD support |
| **4: Platform & Ecosystem** | Windows support + extensibility | Windows full support, plugin architecture, config sync, enterprise deployment, i18n |

See [Product Strategy](specs/PRODUCT_STRATEGY.md) for the complete feature breakdown with priorities.

### Delivery Roadmap (20 Engineering Epics)

Each phase is decomposed into self-contained engineering epics (2-6 weeks each). Epics are the unit of work picked up by the team and broken into implementation stories.

| Epic | Name | Duration | Deliverable |
|------|------|----------|-------------|
| | **Phase 0: Foundation (Months 1-3)** | | |
| E1 | Project Scaffolding, Platform Traits & CI/CD | 3 weeks | Compiling workspace, 6 trait definitions, CI pipeline |
| E2 | X11 Screen Capture & GPU Magnification | 4 weeks | Magnified screen content at 60fps on Linux X11 |
| E3 | Input Tracking & Interactive Magnification | 3 weeks | Cursor-following magnifier with keyboard shortcuts |
| E4 | Tauri Control Panel & Settings Persistence | 3 weeks | Settings UI, IPC, config persistence, system tray |
| | **Phase 1: Core Magnification (Months 4-6)** | | |
| E5 | Lens & Docked Magnification Modes | 3 weeks | Three distinct magnification modes |
| E6 | Visual Enhancement Pipeline | 3 weeks | Color filters, cursor enhancement, bicubic interpolation |
| E7 | Focus Tracking & Keybinding Configuration | 3 weeks | AT-SPI2 focus tracking, configurable hotkeys |
| E8 | Wayland Display Support | 4 weeks | Full Wayland support (GNOME, KDE, wlroots) |
| E9 | Linux Packaging & Release Automation | 3 weeks | .deb, .rpm, AppImage packages; release pipeline |
| | **Phase 2: TTS + Cross-Platform (Months 7-9)** | | |
| E10 | TTS Core Pipeline | 4 weeks | espeak-ng + Kokoro neural TTS at <200ms latency |
| E11 | TTS User Experience | 3 weeks | "Read what I see", voice selection, word highlighting |
| E12 | macOS Platform Support | 5 weeks | Full magnification + TTS on macOS with .dmg |
| | **Phase 3: Advanced + AI (Months 10-14)** | | |
| E13 | Font Re-Rendering Engine | 5 weeks | Crisp text at high zoom (key competitive differentiator) |
| E14 | OCR & AI-Powered Accessibility | 4 weeks | Text extraction from images, AI image descriptions |
| E15 | OpenBSD Platform Support | 3 weeks | Full support on OpenBSD via shared X11 code |
| E16 | Advanced Magnification Features | 4 weeks | Multi-monitor, split-screen, mini-map, condition presets |
| | **Phase 4: Platform & Ecosystem (Months 15-20)** | | |
| E17 | Windows Core Platform Support | 5 weeks | Magnification on Windows via DXGI/DX12 |
| E18 | Windows Full Integration & Packaging | 4 weeks | TTS, NVDA/JAWS coexistence, .msi installer |
| E19 | Plugin Architecture | 4 weeks | Extensible plugin system with example plugin |
| E20 | Enterprise, i18n & Ecosystem | 4 weeks | GPO/MDM deployment, 10+ languages, RTL support |

See [Implementation Roadmap](specs/tech-strategy/09-implementation-roadmap.md) for epic details, dependencies, success criteria, and risk analysis.

## Architecture

Luminos uses a **dual-window design** to combine a rich settings UI with native GPU performance:

```
┌──────────────────────────────────────────────────┐
│            Control Panel (Tauri 2.0)              │
│     TypeScript/React -- settings, preferences     │
│            Not performance-critical               │
└──────────────────────┬───────────────────────────┘
                       │ Tauri IPC
┌──────────────────────┴───────────────────────────┐
│               Rust Core Engine                    │
│  ┌────────────────────────────────────────────┐  │
│  │     Platform Abstraction Layer (traits)     │  │
│  │  ScreenCapture | FocusTracker | TtsEngine   │  │
│  │  WindowManager | InputMonitor | AudioOutput │  │
│  └────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────┐  │
│  │       Rendering Pipeline (wgpu)            │  │
│  │  capture → GPU texture → shader transform  │  │
│  │  → anti-alias → composite → present        │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────┬───────────────────────────┘
                       │ Platform backends
┌──────────┬──────────┬────────────────┬──────────┐
│  Linux   │  macOS   │    Windows     │ OpenBSD  │
│  xcap    │  xcap    │ windows-capture│ xcap     │
│  Vulkan  │  Metal   │  DX12          │ Vulkan   │
│ AT-SPI2  │ AXUIElem │ UI Automation  │  (none)  │
└──────────┴──────────┴────────────────┴──────────┘
```

**Key architectural decisions:**
- The **magnification overlay** is a native Rust window (winit + wgpu), bypassing the webview entirely for GPU-level performance
- **Trait-based platform abstraction** keeps platform backends independent and testable
- **espeak-ng runs as a subprocess** for crash isolation and maintainability (the project is GPLv3-licensed, so GPL propagation is not a concern)

## Tech Stack

| Component | Technology | License |
|-----------|-----------|---------|
| Core language | Rust (2024 edition) | MIT/Apache 2.0 |
| Application framework | Tauri 2.0 | MIT/Apache 2.0 |
| Frontend | TypeScript + React | MIT |
| GPU rendering | wgpu | MIT/Apache 2.0 |
| Window management | winit | Apache 2.0 |
| Screen capture | xcap | Apache 2.0 |
| TTS runtime | sherpa-onnx via sherpa-rs | MIT/Apache 2.0 |
| TTS model | Kokoro-82M ONNX | Apache 2.0 |
| Phonemizer | espeak-ng (subprocess) | GPL-3.0 (compatible) |
| Audio output | cpal | Apache 2.0 |

See [Tech Stack Evaluation](specs/TECH_STACK_EVALUATION.md) for the full validation report with version pinning, alternatives considered, and audit findings.

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux X11 | Planned (Phase 0) | Development starts here -- GNOME, KDE, standalone WMs |
| Linux Wayland | Planned (Phase 1) | PipeWire + XDG Portal |
| macOS (Tahoe+) | Planned (Phase 2) | Full feature port |
| OpenBSD | Planned (Phase 3) | X11 via xenocara, incremental from Linux X11 |
| Windows 11 | Planned (Phase 4) | Must coexist with NVDA/JAWS |

## Performance Targets

| Metric | Target |
|--------|--------|
| Frame rate | 60fps (16ms frame time) on integrated GPUs |
| RAM | <4GB under all conditions |
| TTS latency | <200ms trigger to first audio |
| Binary size | <50MB (excluding voice models) |
| Startup | <2s to usable magnification |

## Project Structure

```
luminos/
├── CLAUDE.md                  # AI agent instructions + coding conventions
├── README.md                  # This file
├── specs/
│   ├── README.md              # Spec-driven development guide
│   ├── PRODUCT_STRATEGY.md    # Product strategy & roadmap v1.3
│   ├── TECH_STACK_EVALUATION.md  # Technology stack validation report
│   ├── tech-strategy/         # Technical strategy (architecture, pipelines, roadmap)
│   │   └── README.md          #   Tech strategy overview + document index
│   └── NNN-story-name/        # Implementation specs (when development begins)
│       ├── STORY.md            #   Requirements specification
│       ├── DESIGN.md           #   Technical design document
│       └── SUBTASKS.md         #   TDD task breakdown + progress tracking
├── docs/                      # Product documentation + user manuals (future)
└── src/                       # Application source (not yet created)
```

## Development Methodology

Luminos uses **spec-driven development** (SDD) with integrated **test-driven development** (TDD):

1. **Specify** -- Write requirements with Given-When-Then acceptance criteria (STORY.md)
2. **Design** -- Translate to architecture with test strategy mapped to every acceptance criterion (DESIGN.md)
3. **Implement** -- Break into atomic tasks, each following the TDD red-green-refactor cycle (SUBTASKS.md)
4. **Review & Close** -- Verify all acceptance criteria, update progress tracking

SUBTASKS.md serves as the **execution memory file** -- it tracks what was done, what's blocked, and what's next, enabling seamless handoffs between AI agents and developers across sessions.

See [Spec-Driven Development Guide](specs/README.md) for the full methodology, templates, and governance rules.

### AI-Agent Driven Development

This project is designed to be built primarily with AI agent assistance:

- **TypeScript** has the largest LLM training corpus -- AI agents generate high-quality React UI code
- **Rust's compiler** catches type, memory, and concurrency errors in AI-generated code at compile time
- **Trait-based abstractions** define clear implementation contracts for AI agents to target
- **Spec-driven development** provides structured context that prevents inconsistent AI output

## Contributing

Luminos is in pre-development. The most valuable contributions right now are:

- **Research** -- Validate assumptions in the [product strategy](specs/PRODUCT_STRATEGY.md), especially around AT user needs
- **Design** -- Help define the first implementation stories using the [SDD methodology](specs/README.md)
- **Accessibility expertise** -- Review our approach from the perspective of low-vision users and AT specialists
- **Technical review** -- Audit the [tech stack evaluation](specs/TECH_STACK_EVALUATION.md) against your platform experience

Once development begins, contributions will follow the spec-driven workflow: every feature starts as a specification before any code is written.

### Getting Started (Contributors)

```bash
# Clone the repository
# Replace with actual repository URL
git clone https://github.com/<your-username>/luminos.git
cd luminos

# Read the project context
cat CLAUDE.md                       # Architecture, coding rules, constraints
cat specs/PRODUCT_STRATEGY.md        # What we're building and why
cat specs/TECH_STACK_EVALUATION.md   # Validated technology choices
cat specs/README.md                  # How we develop (SDD + TDD methodology)
```

### Reporting Issues

Use [GitHub Issues](../../issues) for:
- Bug reports (once development begins)
- Feature requests and user stories
- Research questions and AT user feedback

Use [GitHub Discussions](../../discussions) for:
- Architecture proposals and RFCs
- General questions and community conversation

## Why "Luminos"?

From Latin *lumen* (light). Evokes illumination and clarity. Works globally -- pronounceable in English, Spanish, Portuguese, German, French, and Japanese. CLI-friendly: `luminos`.

## Why Now?

- **Wayland transition** is breaking every standalone Linux magnifier (KMag, Magnus, xzoom)
- **European Accessibility Act** (effective June 2025) is expanding compliance requirements
- **Neural TTS and on-device AI** have matured enough for production accessibility tools
- **Rust ecosystem** has reached critical mass for cross-platform screen capture and GPU rendering
- **NVDA proved the model** -- free, open-source accessibility software can reach mainstream adoption (near-parity with commercial JAWS at 40.5% vs 37.7% primary usage)

## License

Luminos is licensed under the **GNU General Public License v3.0** (GPL-3.0). You are free to use, study, modify, and redistribute this software under the terms of the GPL.

The GPLv3 license ensures that Luminos and all derivative works remain free and open-source. This aligns with the project's mission: accessibility software should be a public good, not a commodity.

See the [LICENSE](LICENSE) file for the full license text.

## Acknowledgments

Luminos draws inspiration from these projects and communities:

- [NVDA](https://www.nvaccess.org/) -- Proved that free, open-source accessibility software can achieve mainstream adoption
- [screenpipe](https://github.com/screenpipe/screenpipe) -- Validated the Tauri + Rust architecture for screen capture and accessibility
- [wgpu](https://github.com/gfx-rs/wgpu) -- Cross-platform GPU rendering foundation
- [Kokoro](https://github.com/hexgrad/kokoro) -- High-quality open-source neural TTS
- [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) -- On-device speech processing runtime

---

<p align="center">
  <em>Every person with a visual impairment deserves a single tool that adapts to them -- magnifying what they need to see, reading what they need to hear, and learning how they work best.</em>
</p>
