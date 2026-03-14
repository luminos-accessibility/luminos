# Luminos

**See clearly. Hear everything.**

[![Project Status: Research](https://img.shields.io/badge/status-research%20phase-yellow)](docs/PRODUCT_STRATEGY.md)
[![License: TBD](https://img.shields.io/badge/license-TBD-lightgrey)](#license)
[![Platforms: macOS, Windows, Linux](https://img.shields.io/badge/platforms-macOS%20%7C%20Windows%20%7C%20Linux-blue)](#platform-support)

Luminos is an open-source, cross-platform screen magnification and text-to-speech accessibility suite for low-vision users. It combines GPU-accelerated magnification with neural TTS in a single application that works the same way on macOS, Windows, and Linux.

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

**There is no cross-platform, open-source, professional-grade screen magnification tool with integrated text-to-speech.**

## The Solution

Luminos fills that gap with:

- **GPU-accelerated magnification** -- 60fps on integrated GPUs, up to 20x zoom with anti-aliased rendering
- **Neural text-to-speech** -- On-device Kokoro TTS (8 languages, with Piper fallback for 30+), <200ms latency
- **Cross-platform consistency** -- Same tool, same keybindings, same experience on macOS, Windows, and Linux
- **Zero cost, full capability** -- No "community edition" with missing features
- **Privacy by design** -- All AI inference on-device, no telemetry by default

## Feature Roadmap

Development is organized into 5 phases:

| Phase | Focus | Key Features |
|-------|-------|-------------|
| **0: Foundation** | Architecture proof on macOS | Screen capture, GPU magnification, basic zoom modes, control panel |
| **1: Core Magnification** | Full magnification on macOS + Windows | Lens mode, docked mode, cursor enhancement, focus tracking, color filters |
| **2: TTS Integration** | Speech + Linux support | Neural TTS, "read what I see", selective reading, Linux full support |
| **3: Advanced + AI** | Commercial feature parity | Font re-rendering, on-device OCR, AI image description, multi-monitor |
| **4: Ecosystem** | Extensibility + enterprise | Plugin architecture, config sync, enterprise deployment, i18n |

See [Product Strategy](docs/PRODUCT_STRATEGY.md) for the complete feature breakdown with priorities.

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
┌──────────┬───────────┴──────┬────────────────────┐
│  macOS   │    Windows       │   Linux            │
│  xcap    │  windows-capture │  xcap (X11)        │
│  Metal   │  DX12            │  Vulkan            │
│ AXUIElem │  UI Automation   │  AT-SPI2           │
└──────────┴──────────────────┴────────────────────┘
```

**Key architectural decisions:**
- The **magnification overlay** is a native Rust window (winit + wgpu), bypassing the webview entirely for GPU-level performance
- **Trait-based platform abstraction** keeps platform backends independent and testable
- **espeak-ng runs as a subprocess** to isolate its GPL-3.0 license from the main binary

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
| Phonemizer | espeak-ng (subprocess) | GPL-3.0 (isolated) |
| Audio output | cpal | Apache 2.0 |

See [Tech Stack Evaluation](docs/TECH_STACK_EVALUATION.md) for the full validation report with version pinning, alternatives considered, and audit findings.

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| macOS (Tahoe+) | Planned (Phase 0) | Development starts here |
| Windows 11 | Planned (Phase 1) | Must coexist with NVDA/JAWS |
| Linux X11 | Planned (Phase 2) | GNOME, KDE, standalone WMs |
| Linux Wayland | Planned (Phase 2) | PipeWire + XDG Portal |

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
├── docs/
│   ├── README.md              # Spec-driven development guide
│   ├── PRODUCT_STRATEGY.md    # Product strategy & roadmap v1.1
│   ├── TECH_STACK_EVALUATION.md  # Technology stack validation report
│   └── NNN-story-name/        # Implementation specs (when development begins)
│       ├── STORY.md            #   Requirements specification
│       ├── DESIGN.md           #   Technical design document
│       └── SUBTASKS.md         #   TDD task breakdown + progress tracking
└── src/                       # Application source (not yet created)
```

## Development Methodology

Luminos uses **spec-driven development** (SDD) with integrated **test-driven development** (TDD):

1. **Specify** -- Write requirements with Given-When-Then acceptance criteria (STORY.md)
2. **Design** -- Translate to architecture with test strategy mapped to every acceptance criterion (DESIGN.md)
3. **Implement** -- Break into atomic tasks, each following the TDD red-green-refactor cycle (SUBTASKS.md)
4. **Review & Close** -- Verify all acceptance criteria, update progress tracking

SUBTASKS.md serves as the **execution memory file** -- it tracks what was done, what's blocked, and what's next, enabling seamless handoffs between AI agents and developers across sessions.

See [Spec-Driven Development Guide](docs/README.md) for the full methodology, templates, and governance rules.

### AI-Agent Driven Development

This project is designed to be built primarily with AI agent assistance:

- **TypeScript** has the largest LLM training corpus -- AI agents generate high-quality React UI code
- **Rust's compiler** catches type, memory, and concurrency errors in AI-generated code at compile time
- **Trait-based abstractions** define clear implementation contracts for AI agents to target
- **Spec-driven development** provides structured context that prevents inconsistent AI output

## Contributing

Luminos is in pre-development. The most valuable contributions right now are:

- **Research** -- Validate assumptions in the [product strategy](docs/PRODUCT_STRATEGY.md), especially around AT user needs
- **Design** -- Help define the first implementation stories using the [SDD methodology](docs/README.md)
- **Accessibility expertise** -- Review our approach from the perspective of low-vision users and AT specialists
- **Technical review** -- Audit the [tech stack evaluation](docs/TECH_STACK_EVALUATION.md) against your platform experience

Once development begins, contributions will follow the spec-driven workflow: every feature starts as a specification before any code is written.

### Getting Started (Contributors)

```bash
# Clone the repository
# Replace with actual repository URL
git clone https://github.com/<your-username>/luminos.git
cd luminos

# Read the project context
cat CLAUDE.md                       # Architecture, coding rules, constraints
cat docs/PRODUCT_STRATEGY.md        # What we're building and why
cat docs/TECH_STACK_EVALUATION.md   # Validated technology choices
cat docs/README.md                  # How we develop (SDD + TDD methodology)
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

License is **to be determined**. The core application targets a permissive license (MIT or Apache 2.0). The espeak-ng phonemizer (GPL-3.0) is isolated as a subprocess to avoid license propagation. This strategy requires legal review before development begins.

See the [Risk Register](docs/PRODUCT_STRATEGY.md#11-risk-register) for details on the licensing analysis.

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
