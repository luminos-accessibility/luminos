# Luminos - Product Strategy & Roadmap

**Open-Source Cross-Platform Screen Magnification + Text-to-Speech Accessibility Suite**

**Document Status:** DRAFT v1.3 (GPLv3 licensing + Linux-first pivot)
**Date:** 2026-03-15
**Audience:** Founding team, contributors, design partners

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Market Analysis](#2-market-analysis)
3. [Competitive Landscape](#3-competitive-landscape)
4. [Gap Analysis](#4-gap-analysis)
5. [Product Definition](#5-product-definition)
6. [User Personas](#6-user-personas)
7. [Feature Roadmap](#7-feature-roadmap)
8. [Technical Strategy](#8-technical-strategy)
9. [Development Methodology](#9-development-methodology)
10. [Success Metrics](#10-success-metrics)
11. [Risk Register](#11-risk-register)
12. [Sustainability & Governance](#12-sustainability--governance)
13. [References](#13-references)
14. [Version History](#14-version-history)

---

## 1. Executive Summary

### The Opportunity

2.2 billion people worldwide have some form of visual impairment (WHO). Screen magnification users significantly outnumber screen reader users -- estimated at roughly 10:1 by Axess Lab, though this ratio includes casual zoom/browser users and the precise evidence base is thin. Regardless, the accessibility software ecosystem has invested disproportionately in screen readers. There is **no cross-platform, open-source, professional-grade screen magnification tool with integrated text-to-speech**. This gap affects millions of users across every operating system.

### The Solution

**Luminos** is a GPLv3-licensed, cross-platform (Linux, macOS, Windows, OpenBSD) accessibility suite that unifies GPU-accelerated screen magnification with neural text-to-speech in a single application. Development starts on **Linux**, the platform with zero professional-grade magnification tools, before expanding to macOS, OpenBSD, and Windows. It targets the massive underserved population of low-vision users who need more than built-in OS tools but cannot access or afford commercial alternatives like ZoomText ($905+ perpetual) or Fusion ($2,309+ perpetual).

### Why Now

- **Wayland transition** is breaking every standalone Linux magnifier (KMag, Magnus, xzoom, Compiz), creating urgent need
- **European Accessibility Act** (effective June 2025) is expanding compliance requirements, driving institutional demand
- **AI capabilities** (neural TTS, on-device OCR, image description) have matured enough for production use
- **Rust ecosystem** has reached critical mass for cross-platform screen capture, GPU rendering, and accessibility API integration
- **AI-assisted development** enables a small team to build and maintain cross-platform software at unprecedented velocity

### Product Name

**Luminos** -- "See clearly. Hear everything."

From Latin "lumen" (light). Evokes illumination and clarity. Works globally (pronounceable in English, Spanish, Portuguese, German, French, Japanese). Low trademark risk (formal USPTO search recommended). CLI-friendly: `luminos`.

Alternatives considered: Apertura ("Opening the world to everyone"), Prismara ("Every perspective, crystal clear").

---

## 2. Market Analysis

### 2.1 Global Visual Impairment Demographics

| Metric | 2020 Estimate | 2050 Projection |
|--------|---------------|-----------------|
| Total with vision impairment | ~1.1 billion (all categories) | ~1.7 billion (+55%) |
| Blind | 43.3 million | 61 million |
| Moderate-to-severe vision impairment | 295 million | 474 million |
| Near vision impairment (presbyopia) | 510 million+ | 866 million |

*Sources: IAPB Vision Atlas; Lancet Global Health; WHO Fact Sheet on Blindness and Visual Impairment.*
*Note: The 1.1B total includes all categories, of which ~510M are presbyopia (near-vision, often correctable with reading glasses). The addressable market for screen magnification software is primarily the ~295M with moderate-to-severe impairment plus a subset of mild/presbyopia users.*

- 90% of visually impaired people live in low- and middle-income countries (WHO)
- Approximately 7 million visually impaired adults in the United States (CDC)
- Leading causes: uncorrected refractive errors, cataracts, AMD, glaucoma, diabetic retinopathy
- Annual global productivity loss from vision impairment: US$411 billion (WHO)

### 2.2 Market Size

| Source | 2024-2025 Valuation | Projected Value | CAGR | Timeframe |
|--------|---------------------|-----------------|------|-----------|
| Kings Research | $4.22B | $11.21B | 13.22% | 2025-2032 |
| Mordor Intelligence | $6.34B (2025) | $11.20B | 12.05% | 2025-2030 |
| GM Insights (screen readers only) | $1.21B (2023) | ~$2.6B | 10% | 2024-2032 |

### 2.3 Usage Patterns

- **45.1%** of low-vision users prefer OS-level magnification settings (WebAIM Low Vision Survey #2, 2018)
- **68%** of low-vision respondents use 2 or more types of assistive technology (WebAIM Low Vision Survey #2, 2018)
- **23%** use both a screen reader and a screen magnifier (WebAIM Low Vision Survey #1, 2013)
- **17.9%** enlarge content to 400% or greater (WebAIM Low Vision Survey #2, 2018)
- NVDA (free) has reached near-parity with JAWS (commercial): 40.5% vs 37.7% primary usage; NVDA leads in "commonly used" at 65.6% vs 60.5% (WebAIM Screen Reader Survey #10, 2024)

### 2.4 Regulatory Drivers

| Regulation | Scope | Key Requirement |
|------------|-------|-----------------|
| ADA Title II (2024 update) | US state/local government digital services | WCAG 2.1 AA |
| Section 508 (2017 rule, effective Jan 2018) | US federal agencies and vendors | WCAG 2.0 AA, EN 301 549 |
| European Accessibility Act | EU public + private sectors | Effective June 2025, EU-wide |
| WCAG 2.2 (2023) | Technical standard referenced by all legislation | Low-vision specific criteria |

### 2.5 Distribution Channels

Users acquire accessibility tools through:
1. **Clinical referral** -- ophthalmologists/low-vision specialists recommend specific tools
2. **Vocational rehabilitation** -- state agencies fund AT for employment
3. **Education** -- schools provide AT under IDEA/Section 504
4. **Enterprise** -- employers provide accommodations via IT departments
5. **Direct purchase** -- individuals buy from vendors
6. **Peer recommendation** -- blind/low-vision orgs, forums, conferences (CSUN, ATIA)

AT specialists and rehabilitation counselors are key influencers -- marketing must target professionals, not just end users.

---

## 3. Competitive Landscape

### 3.1 Magnification Capability Matrix

| Tool | Platform | Max Zoom | Full Screen | Lens | Docked | Font Re-render | Color Inversion | Cost |
|------|----------|----------|-------------|------|--------|----------------|-----------------|------|
| Windows Magnifier | Windows | 16x | Yes | Yes | Yes | No | Yes | Free |
| macOS Zoom | macOS | 40x | Yes | PiP | Split | No | Via settings | Free |
| ZoomText | Windows | 36x (60x on legacy Win 8) | Yes | Yes | Yes (4 pos.) | **Yes (xFont)** | Yes + custom | $905+ |
| SuperNova | Windows | 64x | Yes | Yes | Yes | **Yes (TrueFonts)** | Yes + custom | ~$600+ |
| KMag | Linux (X11) | Variable | Yes | Follow mouse | No | No | Color sim | Free |
| Magnus | Linux (X11) | 5x | No | No | No | No | No | Free |
| GNOME Zoom | Linux | Config. | Yes | Limited | No | No | No | Free |
| KWin Zoom | Linux (KDE) | Config. | Yes | No | No | No | Separate | Free |
| VMG | Cross-platform | 32x | No | Overlay | No | No | No | Free |
| xzoom | Linux (X11) | Integer | No | No | No | No | No | Free |

### 3.2 TTS / Screen Reader Integration Matrix

| Tool | Built-in TTS | Screen Reader | Combined Mag+TTS | Platform |
|------|-------------|---------------|------------------|----------|
| ZoomText Mag/Reader | **Yes (full)** | Partial | **Yes** | Windows |
| Fusion (ZoomText+JAWS) | **Yes (full)** | **Yes (full)** | **Yes** | Windows |
| SuperNova Mag & Reader | **Yes (full)** | **Yes** | **Yes** | Windows |
| Windows Magnifier | Basic (Read aloud) | No | Partial | Windows |
| macOS Zoom + VoiceOver | Hover Text | VoiceOver (separate) | Partial | macOS |
| NVDA | **Yes (core)** | **Yes** | No magnification | Windows |
| JAWS | **Yes (core)** | **Yes** | Via Fusion | Windows |
| Orca | **Yes (core)** | **Yes** | No magnification | Linux |
| All Linux magnifiers | No | No | **No** | Linux |
| VMG | No | No | No | Cross |

### 3.3 Cross-Platform Availability

| Tool | Windows | macOS | Linux |
|------|---------|-------|-------|
| ZoomText | Yes | No | No |
| JAWS | Yes | No | No |
| SuperNova | Yes | No | No |
| NVDA | Yes | No | No |
| Orca | No | No | Yes |
| macOS Zoom/VoiceOver | No | Yes | No |
| Windows Magnifier | Yes | No | No |
| VMG | Yes | Dated | Yes |
| **Luminos (target)** | **Yes** | **Yes** | **Yes (+OpenBSD)** |

### 3.4 Pricing Comparison

| Product | Type | Perpetual (USD) | Annual (USD) |
|---------|------|-----------------|--------------|
| ZoomText Magnifier | Magnifier only | $905 | $362/yr |
| ZoomText Mag/Reader | Magnifier + TTS | $1,259 | $504/yr |
| JAWS Professional | Screen reader | $2,316 | $926/yr |
| Fusion Professional | Mag + Screen reader | ~$3,262 (unverified*) | -- |
| Fusion Home | Mag + Screen reader | ~$2,309 (unverified*) | -- |
| SuperNova Mag & Reader | Mag + Screen reader | ~$1,475 (est.) | ~$228/yr (SUP, est.) |
| NVDA | Screen reader | Free | Free |
| **Luminos (target)** | **Mag + TTS** | **Free** | **Free** |

*\* Fusion pricing could not be independently verified via Vispero eStore at time of writing (collection page returned 404). Figures sourced from AFB AccessWorld and reseller listings.*

---

## 4. Gap Analysis

### 4.1 Critical Feature Gaps

| Gap | Severity | Who It Affects | Current State |
|-----|----------|---------------|---------------|
| No cross-platform magnifier + TTS | **Critical** | All multi-OS low-vision users | Zero solutions exist |
| No open-source font re-rendering | **High** | All magnifier users above ~4x zoom | Only ZoomText (xFont) and SuperNova (TrueFonts) |
| No Linux magnifier + TTS integration | **High** | All Linux low-vision users | Must cobble compositor zoom + Orca |
| No Wayland-native standalone magnifier | **High** | Linux users on modern desktops | KMag, Magnus, xzoom all broken |
| No affordable combined mag + TTS on Windows | **Medium** | Users who can't afford $900+ | NVDA has no magnification |
| No open-source AI image description in magnifiers | **Medium** | All low-vision users viewing images | JAWS PictureSmart only (commercial) |
| No portable USB magnifier + TTS (open source) | **Medium** | Users on shared PCs | SuperNova USB mode being discontinued (Win 11 security restrictions) |
| Limited multi-monitor magnification | **Medium** | Professional users | Only ZoomText/SuperNova |

### 4.2 User Pain Points (from forums, surveys, community)

1. **ZoomText resource consumption** -- users widely report system freezes, cursor disappearing, and frequent restarts needed (multiple Reddit/forum threads)
2. **Cost barrier** -- commercial tools cost more than the computer; 90% of visually impaired in developing countries can't afford them
3. **"Magnification cliff"** -- beyond 4-5x zoom, users lose spatial orientation; no good bridge between magnification and screen reading
4. **Platform lock-in** -- switching OS means relearning entirely different tools
5. **Web design neglects magnification** -- hover tooltips close when magnified, content doesn't reflow, pop-ups appear outside viewport
6. **Innovation stagnation** -- ZoomText's core paradigm hasn't changed since 1988

### 4.3 The Core Opportunity

**NVDA proved that free, open-source accessibility software can achieve mainstream adoption**: near-parity with JAWS (37.7% vs 40.5% primary usage) with 250,000+ users in 175+ countries. But NVDA has no magnification. There is no equivalent success story for screen magnification. Luminos aims to fill that gap.

**Important structural differences from NVDA:** Screen reading is fundamentally software-only (process accessibility tree, output speech). Magnification requires real-time GPU-accelerated screen capture and rendering, which is more platform-dependent and performance-sensitive. NVDA targets only Windows; Luminos targets three platforms. And unlike JAWS ($2,316), Luminos also competes against free built-in OS magnifiers -- the cost argument alone is insufficient. The value must come from cross-platform consistency, combined mag+TTS integration, and features that built-in tools lack.

---

## 5. Product Definition

### 5.1 Vision Statement

> Every person with a visual impairment deserves a single tool that adapts to them -- magnifying what they need to see, reading what they need to hear, and learning how they work best. Luminos is that tool: open source, cross-platform, AI-enhanced, and designed from the first line of code to treat accessibility not as a feature, but as the entire point.

### 5.2 Core Value Propositions

| # | Value Proposition | vs. Built-in OS Tools | vs. Commercial (ZoomText/JAWS) | vs. Open Source (NVDA/Orca) |
|---|------------------|-----------------------|-------------------------------|---------------------------|
| 1 | **Unified by design** -- Magnification + TTS in one tool, sharing context | Separate tools, no integration | Fusion costs $2,309+; Windows only | NVDA: no magnification; Orca: no magnification |
| 2 | **Cross-platform reality** -- Same tool, same keybindings, Linux/macOS/Windows/OpenBSD | Single OS each | Windows only | NVDA: Windows only; Orca: Linux only |
| 3 | **AI-native** -- Intelligent tracking, neural TTS, on-device OCR, scene description | No AI features | Emerging (JAWS PictureSmart) | No AI features |
| 4 | **Zero cost, full capability** -- No "community edition" with missing features | Free but limited | $362-$3,262 | Free but single-function |
| 5 | **Platform, not product** -- Plugin architecture, extensible, community-driven | Closed | Closed | NVDA add-ons (screen reader only) |

### 5.3 Product Principles

1. **Accessibility is the product, not a feature** -- Every design decision evaluated against: "Does this make the tool more accessible?" The tool's own UI must be fully accessible at all times.
2. **Works on day one, grows with you** -- Meaningful value within 60 seconds of install, zero configuration. Deep customization available but never required.
3. **Performance is a feature** -- <16ms frame time (60fps) on integrated GPUs. <4GB RAM. <200ms TTS latency. Performance regressions block releases.
4. **One tool, every platform, same experience** -- Consistent keybindings, behaviors, and settings across OS. Platform divergence is invisible to the user.
5. **Open means open** -- Transparent decisions, inclusive governance, public roadmap, contributor-friendly.
6. **Privacy by design** -- No telemetry by default. Local AI inference. No data collection about what users view or read.
7. **Build for the margins, benefit the center** -- Design for extreme cases (very high magnification, low-spec hardware, RTL languages) first.

---

## 6. User Personas

### Margaret -- The Retiring Professional
- **Age:** 62 | **Location:** Portland, OR | **Condition:** Age-related macular degeneration (progressive)
- **Tech:** Moderate. Windows 11 laptop, Chromebook.
- **Pain:** Uses Windows Magnifier but finds it "jerky." Can't justify $500+ on fixed income. Never tried TTS.
- **Needs:** Zero-config first-run. Smooth magnification. TTS offered as optional enhancement. Progressive disclosure.

### David -- The Full-Time Knowledge Worker
- **Age:** 34 | **Location:** Lagos, Nigeria | **Condition:** Retinitis pigmentosa (tunnel vision, ~10 degrees)
- **Tech:** Expert. QA engineer. Ubuntu + Windows. Writes scripts to automate magnification.
- **Pain:** Different tools/keybindings on Linux vs Windows. No integrated magnifier+TTS on Linux. "Spends as much time configuring tools as using them."
- **Needs:** Cross-platform keybinding consistency. IDE integration. Spatial orientation. Config sync.

### Amara -- The Student
- **Age:** 19 | **Location:** Mumbai, India | **Condition:** Albinism with nystagmus and photophobia
- **Tech:** Moderate-high. Low-spec Windows laptop (4GB RAM), shared family computer.
- **Pain:** Bright screens cause pain. Can't afford ZoomText. Android accessibility is better than her laptop.
- **Needs:** Free. Low resource usage. Strong dark mode. Selective TTS ("read this paragraph"). Offline-first.

### Robert -- The IT Administrator
- **Age:** 45 | **Location:** Stockholm, Sweden | **Condition:** None. IT admin for 2,000-person municipal government.
- **Pain:** ZoomText/JAWS licenses cost EUR 25,000/year. No centralized deployment. Can't install ZoomText on Linux GIS workstations.
- **Needs:** Single tool replacing ZoomText + JAWS. MSI/GPO deployment. deb/rpm/snap packages. Config-as-code. Signed releases, SBOM.

### Dr. Fatima -- The AT Specialist
- **Age:** 38 | **Location:** Amman, Jordan | **Condition:** None. AT specialist at a rehabilitation center.
- **Pain:** "Nothing equivalent to ZoomText for low-vision clients." ZoomText has no Arabic UI. Trains clients on multiple non-integrated tools.
- **Needs:** Exportable/importable user profiles. Arabic UI with RTL support. Condition-based setup wizard. Portable no-install mode.

---

## 7. Feature Roadmap

### 7.1 Phase 0: Foundation (Months 1-3)

**Goal:** Core magnification engine working on **Linux X11**, proving the architecture on the most underserved platform.

| Feature | Priority | Description |
|---------|----------|-------------|
| Screen capture engine | P0 | Screen capture via `xcap` crate (XCB path on Linux X11) |
| GPU-accelerated magnification | P0 | `wgpu`-based rendering (Vulkan backend) with bilinear interpolation, transparent overlay window |
| Basic magnification modes | P0 | Full-screen zoom (1.5x-20x) with mouse-follow tracking |
| Keyboard shortcuts | P0 | Zoom in/out, toggle, reset. Configurable. |
| Smooth scrolling/panning | P0 | 60fps panning when cursor reaches magnification window edges |
| Tauri control panel shell | P0 | Basic settings window (zoom level slider, mode selection) |
| CI/CD pipeline | P0 | Build + test on Linux, automated releases |

### 7.2 Phase 1: Core Magnification (Months 4-6)

**Goal:** Full magnification feature set on **Linux X11 + Wayland** support. Functional baseline on the Linux platform.

| Feature | Priority | Description |
|---------|----------|-------------|
| Wayland screen capture | P0 | PipeWire + XDG Desktop Portal for Wayland compositors (GNOME, KDE, Sway) |
| Lens magnification mode | P0 | Movable lens/loupe following cursor |
| Docked magnification mode | P0 | Split-screen with magnified region top/bottom/left/right |
| Cursor enhancement | P0 | Enlarged cursor, crosshairs, halo, locator animation |
| Focus tracking | P0 | Magnification follows keyboard focus, text caret, mouse pointer (AT-SPI2) |
| Color inversion / filters | P1 | Full inversion, smart inversion, custom color schemes, brightness/contrast |
| High-contrast color schemes | P1 | Preset schemes (white-on-black, yellow-on-blue, green-on-black) |
| Smooth text rendering | P1 | Anti-aliased text at high magnification (shader-based smoothing) |
| Settings persistence | P1 | Save/load user configuration profiles |
| Linux packages | P1 | deb, rpm, snap, AppImage, Flatpak |
| XShm capture optimization | P1 | Direct x11rb-based capture with XShm for improved X11 performance at low zoom levels |

### 7.3 Phase 2: TTS + Cross-Platform (Months 7-9)

**Goal:** Integrated text-to-speech on Linux + **macOS support**. Port magnification engine to macOS.

| Feature | Priority | Description |
|---------|----------|-------------|
| Neural TTS engine integration (Kokoro via sherpa-onnx) | P0 | Embedded neural TTS via sherpa-onnx runtime (sherpa-rs Rust bindings), Kokoro-82M as primary model for near-commercial quality, Piper VITS models as language breadth fallback via same sherpa-onnx runtime; 9 language codes / ~8 unique languages (Kokoro primary), 30+ via Piper fallback |
| "Read what I see" mode | P0 | TTS reads text under magnification focus (via accessibility APIs) |
| Selective TTS | P0 | Select text region, trigger speech ("read this paragraph") |
| macOS screen capture | P0 | ScreenCaptureKit via `xcap` / `screencapturekit` crate |
| macOS full support | P0 | Port all Phase 0-1 magnification features to macOS (Metal via wgpu, AXUIElement focus tracking) |
| Reading speed / voice control | P1 | Adjustable rate, pitch, voice selection |
| Platform-native TTS fallback | P1 | AVSpeech (macOS), speech-dispatcher (Linux) |
| Read aloud with word highlighting | P1 | Synchronized visual highlight of current word being spoken |
| macOS installer | P1 | macOS .dmg package |

### 7.4 Phase 3: Advanced Magnification + AI (Months 10-14)

**Goal:** Feature parity with mid-tier commercial tools. AI-powered capabilities. **OpenBSD support**.

| Feature | Priority | Description |
|---------|----------|-------------|
| Font re-rendering engine | P0 | Re-render text at magnified size using system fonts (like xFont/TrueFonts). Key competitive differentiator. |
| On-device OCR | P0 | Vision framework (macOS), Tesseract (Linux/OpenBSD). For apps without accessibility API support. |
| OCR-to-TTS pipeline | P0 | Automatic text extraction from images/scanned docs, fed to TTS |
| OpenBSD X11 support | P1 | Port Linux X11 backend to OpenBSD (X11/XCB capture, Vulkan via wgpu). Incremental -- most X11 code shared with Linux. |
| Multi-monitor support | P1 | Independent magnification per monitor |
| AI image description | P1 | On-device model describes images/charts/diagrams via TTS |
| Split-screen view | P1 | Original + magnified side by side |
| Mini-map navigator | P1 | Overview of full screen with viewport indicator for spatial orientation |
| Mouse pointer customization | P1 | Size, color, shape, animation for visibility |
| Condition-based profiles | P2 | Preset configurations optimized for AMD, glaucoma, diabetic retinopathy, etc. |
| Setup wizard | P2 | First-run wizard: "What kind of vision difficulty do you have?" |

### 7.5 Phase 4: Platform & Ecosystem (Months 15-20)

**Goal:** **Windows support** + plugin ecosystem + enterprise features. Windows is last because it already has 5+ magnification options (ZoomText, SuperNova, Fusion, Windows Magnifier, VMG).

| Feature | Priority | Description |
|---------|----------|-------------|
| Windows screen capture | P0 | Windows.Graphics.Capture via `windows-capture` crate (DXGI performance path); `xcap` as cross-platform primary with `windows-capture` as Windows-specific fallback for DXGI Desktop Duplication performance |
| Windows full support | P0 | Port all magnification + TTS features to Windows (DX12 via wgpu, UI Automation, SAPI fallback). Must coexist with NVDA/JAWS. |
| Windows installer | P0 | MSI installer with GPO support for enterprise deployment |
| Plugin architecture | P0 | Rust trait-based backend plugins + Tauri frontend extensions |
| Enterprise deployment | P1 | GPO/MDM configuration, silent install, centralized config |
| Configuration sync | P1 | Cross-device settings sync via file export/import (Git-friendly JSON) |
| USB portable mode | P2 | Run from USB drive without installation. Note: SuperNova discontinued this due to Windows 11 security restrictions -- technical feasibility must be validated. |
| Braille display output | P2 | Basic braille support via platform APIs |
| Application-specific profiles | P2 | Auto-switch magnification settings per application |
| Touch/trackpad gestures | P2 | Pinch-to-zoom, two-finger pan on supported devices |
| i18n: UI localization | P2 | 10+ languages for the application UI, starting with top WHO-impacted regions |
| RTL layout support | P2 | Full right-to-left support for Arabic, Hebrew, Persian |
| Community plugin registry | P2 | Curated repository for community-built plugins |

### 7.6 Phase 5: Vision (Months 21+)

| Feature | Description |
|---------|-------------|
| AI-powered content-aware magnification | Intelligent zoom that follows semantic content (paragraphs, UI elements) not just cursor |
| Context-aware TTS | Neural voices adjust pacing/emphasis based on content type (code vs prose vs navigation) |
| Scene understanding | AI describes complex visual layouts, data visualizations, spatial relationships |
| Adaptive learning profiles | System learns user's browsing patterns and pre-adjusts magnification |
| AR/VR integration research | Explore wearable magnification (Apple Vision Pro, smart glasses) |
| Screen reader feature parity | Graduate from magnifier+TTS to full screen reader capability |

### 7.7 Feature Priority Legend

- **P0** = Must-have for the phase. Blocks release.
- **P1** = Important. Included in phase if feasible, may slip to next.
- **P2** = Nice-to-have. Scheduled but deprioritized if needed.

---

## 8. Technical Strategy

### 8.1 Architecture Overview

```
+----------------------------------------------------------+
|                 Tauri Control Panel                       |
|  (TypeScript/React: settings, status, voice selection)    |
+----------------------------------------------------------+
            |  Tauri IPC Commands (typed)  |
+----------------------------------------------------------+
|                   Rust Core Engine                        |
|  +----------------------------------------------------+  |
|  |  Platform Abstraction Layer (Rust traits)           |  |
|  |  - ScreenCapture trait                              |  |
|  |  - FocusTracker trait                               |  |
|  |  - TtsEngine trait                                  |  |
|  |  - WindowManager trait                              |  |
|  |  - InputMonitor trait                               |  |
|  |  - AudioOutput trait                                |  |
|  +----------------------------------------------------+  |
|  |  Plugin System (trait objects + Tauri plugins)      |  |
|  +----------------------------------------------------+  |
+----------------------------------------------------------+
            |  Platform-specific backends  |
+----------------------------------------------------------+
|  Linux Backend    | macOS Backend      | Windows Backend  |
|  xcap (XCB)       | xcap (SCKit)       | xcap / win-capture|
|  AT-SPI2          | AXUIElement        | UI Automation    |
|  speech-dispatcher| AVSpeech           | SAPI             |
|  Vulkan (via wgpu)| Metal (via wgpu)   | DX12 (via wgpu)  |
+----------------------------------------------------------+
```

### 8.2 Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Application framework** | Tauri 2.0 | Lightweight (2-15MB base vs Electron's 85-100MB+), 58% less RAM; note: final app will be larger due to bundled TTS models and OCR. Proven by screenpipe (~17K stars). |
| **Backend language** | Rust | Memory-safe, zero-cost abstractions, compiler-as-reviewer for AI-generated code, excellent cross-platform crate ecosystem |
| **Frontend** | TypeScript + React | Largest LLM training corpus, optimal for AI-assisted UI development, declarative component model |
| **Screen capture** | `xcap` crate (v0.9.1, Apache 2.0) | Cross-platform capture starting with Linux X11 via XCB (simplest path -- no permissions/entitlements), then ScreenCaptureKit on macOS, Windows 8.1+ |
| **GPU rendering** | `wgpu` | Cross-platform (Metal/DX12/Vulkan), transparent overlay window, GPU-accelerated magnification transforms |
| **Window management** | `winit` (v0.30.13, Apache 2.0) | Cross-platform window creation for the magnification overlay: transparent, borderless, always-on-top windows integrated with wgpu via raw-window-handle |
| **TTS engine** | Kokoro-82M via sherpa-onnx (`sherpa-rs` Rust bindings) | Near-commercial quality neural TTS, Apache 2.0 model weights, 9 language codes (~8 unique languages); Piper VITS models available as fallback for 30+ languages via same runtime |
| **TTS fallback** | Platform-native | AVSpeechSynthesizer (macOS), SAPI (Windows), speech-dispatcher (Linux) |
| **Audio output** | `cpal` (Apache 2.0) | Cross-platform audio output for TTS playback |
| **Clipboard** | `arboard` (MIT/Apache 2.0) | Cross-platform clipboard access for "read selected text" workflows |
| **OCR** | Platform-native + Tesseract | macOS Vision, Windows OCR API, Tesseract 5.x as cross-platform fallback |
| **Accessibility APIs** | Platform-native | AXUIElement (macOS), UI Automation (Windows), AT-SPI2 (Linux) |
| **Text extraction** | Accessibility APIs + OCR (co-primary) | Accessibility APIs for apps with AT support; OCR for legacy/custom-rendered/Electron apps. Coverage is variable -- many apps expose minimal accessibility tree data. |

### 8.3 Dual-Window Architecture

The application uses two window types:

1. **Control Panel** (Tauri webview): Settings, preferences, voice selection, magnification controls. Standard web UI. Performance non-critical.

2. **Magnification Overlay** (native Rust + wgpu): Transparent, always-on-top, GPU-accelerated. Captures screen content, applies magnification transforms via GPU shaders, composites as overlay. This bypasses WebkitGTK entirely, mitigating Tauri's known Linux rendering concerns.

### 8.4 Key Technical Decisions

| Decision | Choice | Status |
|----------|--------|--------|
| Open-source license | **GPLv3** | Decided. Rationale: (a) espeak-ng dependency (GPL-3.0, used for G2P by Kokoro and Piper) makes GPL propagation unavoidable without complex subprocess isolation -- adopting GPLv3 eliminates this architectural constraint entirely, (b) copyleft prevents proprietary competitors from absorbing the codebase without contributing back, (c) aligns with NVDA/GNOME/Linux community values and culture, (d) simplifies architecture by not requiring subprocess isolation for legal reasons (though subprocess isolation may still be preferred for crash isolation engineering reasons). Note: espeak-ng may still be run as a subprocess for crash isolation, but this is an engineering decision, not a legal requirement. |
| Core language | Rust | Decided |
| UI framework | Tauri 2.0 + React | Decided |
| GPU rendering | wgpu (Vulkan/Metal/DX12) | Decided |
| TTS engine | Kokoro via sherpa-onnx (primary), Piper VITS via sherpa-onnx (language fallback), platform-native (system fallback) | Decided |
| Font re-rendering approach | TBD -- research required | Phase 3 |
| AI inference | Local-first, cloud-optional | Decided |
| Plugin architecture | Rust traits (backend) + Tauri plugins (frontend) | Decided |
| Governance model | BDFL -> Non-profit Foundation (see Section 12.2) | Decided: BDFL in Year 1, transition to registered non-profit foundation in Year 2+ |

### 8.5 Key Rust Crates

| Crate | Purpose | Maturity |
|-------|---------|----------|
| `xcap` | Cross-platform screen capture (v0.9.1, Apache 2.0) | Stable, 85K monthly downloads |
| `wgpu` | GPU rendering (Vulkan/Metal/DX12) | Mature, widely used |
| `winit` | Window creation for magnification overlay (v0.30.13, Apache 2.0) | Mature, 34.3M total downloads |
| `sherpa-rs` | TTS runtime (Kokoro, Piper via sherpa-onnx) (v0.6.8, MIT) | Active |
| `cpal` | Cross-platform audio output (Apache 2.0) | Mature |
| `arboard` | Cross-platform clipboard (MIT/Apache 2.0) | Stable |
| `tauri` | Application framework | Mature, v2 stable |
| `atspi` | AT-SPI2 D-Bus bindings (Linux) | Active |
| `leptess` | Tesseract OCR bindings | Stable but potentially unmaintained (~3 years since last update); evaluate `tesseract-rs` as alternative |
| `screencapturekit` | macOS ScreenCaptureKit bindings | Active, mature |
| `objc2` | macOS Objective-C bindings | Mature |
| `windows-capture` | Windows.Graphics.Capture / DXGI bindings | Active, mature |
| `windows` | Windows API bindings (UIA, OCR, etc.) | Mature (Microsoft-maintained) |

### 8.6 Performance Requirements

| Metric | Target | Rationale |
|--------|--------|-----------|
| Frame rate | 60fps (16ms frame time) | Magnified view is the user's primary view; stuttering = inaccessible |
| RAM usage | <4GB under all conditions | Must work alongside user's applications on 8GB machines |
| TTS latency | <200ms from trigger to first audio | Must feel responsive |
| App binary size | <50MB (excluding voice models) | Reasonable download, especially in low-bandwidth regions |
| Startup time | <2 seconds to usable magnification | User depends on the tool from login |
| GPU | Must work on integrated GPUs | Dedicated GPU cannot be a requirement |

### 8.7 Platform-Specific Considerations

**Linux (starting platform):**
- X11: xcap via XCB (xcb_get_image); XShm optimization planned for Phase 1. No permission dialogs required -- simplest capture path across all platforms.
- Wayland: must use PipeWire + XDG Desktop Portal (user consent dialog). Phase 1.
- AT-SPI2 works over D-Bus (unaffected by Wayland transition)
- Must work on GNOME, KDE, and standalone WMs (Sway, Hyprland, i3)
- Package formats: deb, rpm, snap, AppImage, Flatpak
- Vulkan is the primary GPU backend (well-supported on Mesa drivers)

**macOS (Phase 2):**
- ScreenCaptureKit requires Screen Recording permission (user authorization dialog)
- Accessibility API requires Accessibility permission
- ScreenCaptureKit is mandatory from macOS 15 (CGWindowListCreateImage deprecated)
- Metal GPU backend via wgpu

**OpenBSD (Phase 3):**
- X11 only (no Wayland compositor in base). Most X11/XCB code shared with Linux backend.
- Vulkan support via Mesa (limited but improving). Software rendering fallback may be needed.
- No AT-SPI2; accessibility API integration deferred. Focus on core magnification first.
- Package via OpenBSD ports system
- Smallest user base but essentially zero accessibility infrastructure -- high impact per user

**Windows (Phase 4):**
- Windows.Graphics.Capture recommended over legacy Magnification API (per Microsoft)
- Requires Windows 10 1803+
- MSI installer with GPO support for enterprise deployment
- Must coexist with NVDA/JAWS (many users run both magnification + screen reader)
- DX12 GPU backend via wgpu
- Last platform because Windows already has ZoomText, SuperNova, Fusion, Windows Magnifier, and VMG

---

## 9. Development Methodology

### 9.1 AI-Agent Driven Development

This project is designed to be built primarily with AI agent assistance. Technology choices directly support this:

**TypeScript + Rust synergy for AI development:**
- TypeScript has the largest LLM training corpus of any typed language -- AI agents generate high-quality React UI code
- Rust's strict compiler acts as an automated reviewer -- type/memory/concurrency errors in AI-generated code are caught at compile time, not runtime
- The `screenpipe` project (17K+ stars) validates this exact stack for screen capture + OCR + accessibility
- Christopher Chedeau demonstrated porting 100K lines of TypeScript to Rust via Claude Code in ~4 weeks

**AI-first development practices:**
- Comprehensive type definitions as "contracts" for AI-generated code
- Trait-based abstractions enable AI agents to implement platform backends independently
- Automated test suites validate AI-generated implementations
- CI/CD pipeline catches regressions from AI-generated commits

### 9.2 Development Process

| Practice | Approach |
|----------|----------|
| Version control | Git, GitHub, trunk-based development with feature branches |
| Code review | AI-generated code reviewed by AI auditor agents + human maintainers |
| Testing | Unit tests (Rust `#[test]`), integration tests, accessibility regression tests |
| CI/CD | GitHub Actions: build all platforms, run tests, create release artifacts |
| Release cadence | Monthly releases during active development; quarterly stable releases |
| Documentation | In-repo docs, auto-generated API docs, user guides with audio descriptions |
| Community | GitHub Discussions for RFCs, Issues for bugs, public roadmap |

### 9.3 Development Phasing Strategy

**Start where you are needed most, expand to where alternatives already exist:**
1. **Linux X11 first** -- zero professional magnification tools, zero magnifier+TTS integrations, most underserved users
2. **Linux Wayland** -- future-proof the Linux platform; Wayland transition is actively breaking existing X11 tools (KMag, Magnus, xzoom)
3. **macOS** -- good built-in Zoom but no open-source alternative with TTS integration
4. **OpenBSD** -- essentially no accessibility infrastructure; incremental from Linux X11 backend
5. **Windows last** -- already has ZoomText, SuperNova, Fusion, Windows Magnifier, VMG; least urgent need

**Platform sequencing rationale:**

The Linux-first strategy is driven by where the project can deliver the most impact per unit of effort:

- **(a) Linux users are most underserved.** There are zero professional-grade magnification tools on Linux. KMag (basic, X11-only), Magnus (5x max, unmaintained), and xzoom (integer zoom only) are the only standalone options. None integrate with TTS. GNOME/KDE compositor zoom exists but offers no lens modes, no color filters, no cursor enhancement, and no TTS.
- **(b) X11 screen capture is technically simplest.** No permission dialogs, no entitlements, no sandbox restrictions. XCB capture works immediately. This accelerates Phase 0 architecture validation.
- **(c) Wayland transition creates urgency.** Every existing Linux magnifier is X11-only and is actively breaking as distributions default to Wayland. There is a shrinking window to serve these users before they are forced onto compositor-level zoom with no feature depth.
- **(d) Zero competition on Linux vs. mature competition on Windows/macOS.** On Windows, users have 5+ options across a $0-$3,262 price range. On macOS, built-in Zoom is capable (40x, PiP, split). On Linux, there is nothing. Building where there is nothing is strategically superior to entering a crowded market.
- **(e) Open-source community alignment.** Linux users are disproportionately open-source contributors. A Linux-first launch maximizes community engagement, bug reports, and contributions during the critical early development period.
- **(f) Vulkan well-supported for GPU rendering.** Mesa drivers provide strong Vulkan support on Linux, validating the wgpu/Vulkan rendering pipeline.

**Platform sequencing trade-off (explicit):** >90% of AT users are on Windows (WebAIM data). Starting on Linux means the largest user base does not receive Luminos until Phase 4. This is a deliberate choice: Windows users already have multiple options (including the free Windows Magnifier), while Linux users have effectively none. The platform abstraction layer (Rust traits with per-platform backends) ensures that platform backends are independent -- if the team grows or receives contributions, platform development can proceed in parallel. Windows is not deprioritized because it is unimportant; it is sequenced last because its users are least underserved.

**Validate architecture before expanding features:**
- Phase 0 proves the Tauri + Rust + wgpu architecture on Linux X11
- Phase 1 proves the X11-to-Wayland abstraction and full Linux feature set
- Phase 2 proves cross-platform portability (macOS) and magnification + TTS integration
- Phase 3 proves BSD portability and advanced AI features
- Phase 4 proves Windows platform support and enterprise readiness

---

## 10. Success Metrics

### 10.1 Adoption

| Metric | Year 1 | Year 3 |
|--------|--------|--------|
| Total downloads (all platforms) | 50,000 | 500,000 |
| Monthly active users (opt-in) | 5,000 | 75,000 |
| Platform distribution | Linux 60%, macOS 25%, Windows 15% | Linux 40%, macOS 25%, Windows 25%, OpenBSD/other 10% |
| Institutional deployments (>10 seats) | 20 | 200 |
| Geographic diversity (countries >100 users) | 15 | 50 |

### 10.2 Community Health

| Metric | Year 1 | Year 3 |
|--------|--------|--------|
| GitHub stars | 2,000 | 15,000 |
| Active contributors (>1 merged PR / 90 days) | 25 | 100 |
| First-time contributor PRs merged / quarter | 15 | 50 |
| Avg time to first response on issues | <48 hours | <24 hours |
| Plugin ecosystem size | 10 | 75 |

### 10.3 Feature Coverage

| Metric | v1.0 | v3.0 |
|--------|------|------|
| ZoomText feature parity | 60% | 90% |
| Supported TTS languages | 10 | 30+ |
| WCAG 2.2 AA compliance (own UI) | 100% | 100% |
| Magnification modes | 4 | 6+ |

### 10.4 User Satisfaction

| Metric | Year 1 | Year 3 |
|--------|--------|--------|
| Net Promoter Score (NPS) | >40 | >60 |
| Time-to-productivity (install to useful magnification) | <3 min | <1 min |
| AT specialist recommendation rate | >50% | >75% |

### 10.5 Anti-Metrics (What We Do NOT Optimize For)

- **Total feature count** -- depth and quality over breadth
- **Release frequency** -- ship when ready, not on a schedule
- **Revenue (Year 1-2)** -- building a public good first

---

## 11. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **GPLv3 license may limit corporate adoption** | **Medium** | **High** | Some corporate legal teams are cautious about GPLv3. Mitigation: (a) institutional support contracts provide a "who do we call?" relationship that satisfies procurement, (b) clear documentation that GPLv3 allows unrestricted internal use without redistribution obligations -- only modifications distributed externally trigger copyleft, (c) precedent: NVDA (GPLv2) is widely deployed in enterprises, government agencies, and universities despite GPL license, (d) for institutions, the alternative is $905+/seat for ZoomText -- GPL concerns are secondary to cost savings. |
| **Linux-first development delays reaching largest user base (Windows)** | **Medium** | **Medium** | >90% of AT users are on Windows. Delaying Windows support to Phase 4 means the largest potential user base waits 15+ months. Mitigation: (a) Windows users already have multiple alternatives (ZoomText, SuperNova, Fusion, Windows Magnifier), (b) platform abstraction layer enables parallel development -- Windows backend work can begin earlier if contributors or funding allow, (c) Linux-first builds community and contributor base that accelerates later platform work, (d) early Linux release generates press coverage and awareness before Windows launch. |
| **Monetization of GPLv3 project insufficient for sustainability** | **Medium** | **High** | Open-source accessibility tools historically struggle with sustainable funding. Mitigation: diversified non-profit funding model (see Section 12) combining grants, institutional sponsorship, support contracts, and donations. Target institutions (universities, government, enterprises) as primary paying customers, not individuals. Precedent: NV Access sustains NVDA via this model; Blender Foundation generates millions/year from corporate sponsors. Apply for grants before launch, not after. |
| Font re-rendering is extremely hard | High | High | ZoomText's xFont and SuperNova's TrueFonts represent decades of proprietary engineering. Requires deep integration with FreeType (Linux/OpenBSD), Core Text (macOS), DirectWrite (Win). Phase 3 timing allows research, but this may define whether Luminos can compete above ~4x zoom. Consider as potential multi-phase effort. |
| Wayland consent dialog chicken-and-egg | Certain | High | XDG Portal screen capture on Wayland requires a system dialog to grant permission. Low-vision users may need magnification to read the permission dialog. Mitigate with: session restoration tokens, clear documentation, OS-level accessibility for the dialog itself. |
| Tauri WebkitGTK rendering issues on Linux | High | Medium | Control panel uses simple forms; magnification overlay bypasses webview; CEF alternative in development |
| Platform API deprecation (especially macOS annual cycles) | Medium | High | Abstract behind traits; monitor deprecation cycles; budget for annual platform adaptation |
| Kokoro/Piper TTS quality insufficient for some languages | Medium | Low | Kokoro covers ~8 unique languages (9 language codes) at near-commercial quality; Piper VITS models extend coverage to 30+ via same sherpa-onnx runtime. Fallback to platform-native TTS for unsupported languages. |
| **xcap X11 capture performance at low zoom levels** | **Medium** | **Medium** | xcap uses non-SHM X11 capture (xcb_get_image path), which requires a full X server round-trip per capture. Adequate for small source regions at high zoom, but may exceed frame budget at low zoom levels (1.5-3x) with large capture areas on high-resolution displays. Mitigation: implement direct x11rb-based capture backend with XShm support as Phase 1 optimization. OBS achieves 60fps+ X11 capture via XShm. |
| Accessibility API coverage gaps | Medium | Medium | Many apps (legacy Win32, Electron, games, CAD, PDF viewers) expose minimal accessibility tree. OCR must be treated as co-primary strategy, not just fallback. |
| Low adoption despite technical quality | Medium | High | Engage AT specialists and rehab centers early; partner with NVDA community; attend CSUN/ATIA. Linux-first strategy targets a community with zero alternatives, improving early adoption odds. |
| Contributor burnout (common in accessibility OSS) | Medium | High | Establish sustainable governance; seek grant funding (Sovereign Tech Fund, NLnet, Microsoft AI for Accessibility). Non-profit structure enables diversified funding. |
| Commercial vendor response (Vispero price cuts) | Low | Low | Our value is cross-platform + free; price cuts validate the market. Linux-first strategy means we face zero commercial competition initially. |
| USB portable mode may be infeasible on Windows 11 | Medium | Low | SuperNova discontinued USB mode due to Win 11 security restrictions. Evaluate technical feasibility before committing to Phase 4. |
| OpenBSD platform support may have limited user impact | Medium | Low | OpenBSD has a small user base. Mitigation: incremental effort (most X11 code shared with Linux), high per-user impact in a community with essentially zero accessibility tools, and aligns with project values of serving the most underserved. |

---

## 12. Sustainability & Governance

For an accessibility tool that vulnerable users depend on daily, sustainability is a first-order concern, not an afterthought. Users who adopt Luminos cannot easily switch if the project is abandoned. The GPLv3 license ensures the codebase remains permanently open, but ongoing development requires sustainable funding.

### 12.1 Monetization Strategy

**Core insight:** The primary paying customers for Luminos are **institutions** (universities, government agencies, enterprises) facing accessibility compliance mandates (EAA, Section 508, ADA Title II), not individual users. A free GPLv3 tool with paid institutional support is dramatically cheaper than $905+/seat for ZoomText -- a university with 200 low-vision students saves $160,000+/year by deploying Luminos with a $5,000-$15,000 support contract.

**Non-profit structure:** Luminos will be established as a registered non-profit organization, modeled on NV Access (NVDA) and the Blender Foundation. This enables grant eligibility, tax-deductible donations in most jurisdictions, and institutional procurement compatibility.

#### 12.1.1 Revenue Streams (in priority order)

**1. Foundation and Government Grants (Year 1 onward) -- Primary early revenue**

| Grant Source | Typical Range | Eligibility | Notes |
|-------------|---------------|-------------|-------|
| Sovereign Tech Fund (Germany/EU) | EUR 150K-1M | Open-source infrastructure projects | GNOME received EUR 1M (2024). Luminos qualifies as accessibility infrastructure. |
| NLnet Foundation (EU) | EUR 5K-50K | Open internet/open-source projects | Funds specific milestones. Multiple applications possible. |
| Microsoft AI for Accessibility | $25K-500K | AI-powered accessibility projects | Kokoro TTS + AI image description directly qualify. |
| NIDILRR (US federal) | $50K-500K | Assistive technology R&D | US National Institute on Disability, Independent Living, and Rehabilitation Research. |
| Mozilla MOSS | $10K-250K | Open-source projects supporting internet health | Previously funded accessibility tools. |

**2. Community Donations (Day 1 onward)**

- GitHub Sponsors + Open Collective (dual platform for maximum reach)
- In-product donation prompt (non-intrusive, shown once after 30 days of active use)
- Annual fundraising campaigns tied to Global Accessibility Awareness Day (third Thursday of May) and accessibility awareness months
- Target: $10,000-$80,000/year scaling with active user base

**3. Tiered Corporate/Institutional Sponsorship (Year 2 onward)**

| Tier | Annual | Benefits |
|------|--------|----------|
| Platinum | $25,000 | Logo on website/README, priority feature input, quarterly roadmap briefing, named acknowledgment in release notes |
| Gold | $10,000 | Logo on website/README, annual roadmap briefing |
| Silver | $5,000 | Logo on sponsors page, acknowledgment in release notes |
| Bronze | $1,000 | Name on sponsors page |

Target companies with EAA/Section 508 compliance obligations. Precedent: Blender Development Fund generates millions per year from Apple, Google, NVIDIA, AMD, Intel, Meta, and others.

**4. Institutional Support Contracts (Year 2-3)**

| Tier | Annual | Includes |
|------|--------|---------|
| Basic | $2,000/yr | Email support (48h SLA), deployment documentation, quarterly security advisories |
| Standard | $5,000/yr | Email + video support (24h SLA), deployment assistance, priority bug fixes, custom configuration guidance |
| Enterprise | $15,000/yr | Dedicated support contact (8h SLA), on-site/remote deployment, GPO/MDM integration support, custom packaging, SLA-backed uptime for support services |

IT departments need "who do we call when it breaks?" -- institutional support contracts answer that question. This is the proven model for enterprise open-source adoption (Red Hat, Canonical, NV Access).

**5. Training and Certification (Year 2-3)**

- **Luminos AT Professional** certification exam: $200/exam. Validates proficiency in deploying, configuring, and supporting Luminos for end users.
- Online self-paced courses: $50-$150 per module (deployment, advanced configuration, TTS customization)
- Institutional training packages: $2,000-$10,000 for on-site/remote group training
- Precedent: NV Access offers NVDA Certified Expert exams (AUD $95-120/exam)

**6. Consulting and Customization (Year 3+)**

- Custom deployment and integration services: $5,000-$50,000 per engagement
- Accessibility auditing for organizations deploying Luminos alongside other AT
- Custom plugin development for institutional needs
- Integration with institutional IT systems (LDAP, SSO, MDM)

**7. Feature Sponsorship (Year 2 onward)**

- Institutions fund specific features on the public roadmap
- Minimum $5,000 per sponsored feature
- Sponsor acknowledged in feature documentation and release notes
- Feature remains GPLv3 -- sponsorship accelerates development, does not create proprietary forks

#### 12.1.2 Revenue Targets

| Timeframe | Target Range | Primary Sources |
|-----------|-------------|-----------------|
| Year 1 | $50,000-$150,000 | Grants + community donations |
| Year 2-3 | $200,000-$500,000/yr | Sponsorship + support contracts + grants |
| Year 3+ | $500,000-$1,000,000+/yr | Diversified across all streams |

**Revenue diversification target (Year 3+):** Grants 15-25%, Sponsorship 25-35%, Support contracts 20-30%, Donations 10-15%, Training 5-10%, Consulting 5-15%. No single source should exceed 35% of total revenue to ensure resilience.

#### 12.1.3 Discarded Monetization Approaches

| Approach | Why Discarded |
|----------|--------------|
| Dual licensing (GPLv3 + commercial) | Requires Contributor License Agreement (CLA), which discourages open-source contributions. Limited market for commercially embedding a screen magnifier. Incompatible with community values. |
| Open core / premium features | Ethically problematic for an accessibility tool -- withholding features from disabled users to drive revenue contradicts the project's mission. |
| Selling pre-built binaries | Users who most need pre-built binaries (non-technical low-vision users) are least able to compile from source. Charging for binaries creates a de facto paywall. Flatpak/snap/AppImage solve the distribution problem. |
| SaaS / cloud-hosted | Desktop screen magnification cannot meaningfully be cloud-hosted. Latency requirements (16ms frame time) are fundamentally incompatible with network round-trips. |
| Advertising | Hostile to accessibility users. Ads create visual noise and cognitive load that directly conflict with the purpose of a magnification tool. |

### 12.2 Governance

**Phase 1 (Year 1): BDFL + Core Team**
- Project founder serves as BDFL during initial development
- Core team of 3-5 maintainers with commit rights, selected based on sustained contribution
- All major decisions documented publicly (GitHub Discussions RFCs)
- Code of Conduct enforced from day one

**Phase 2 (Year 2+): Non-Profit Foundation**
- Transition to a registered non-profit (modeled on NV Access / Blender Foundation)
- Elected board of directors including: maintainers, user representatives (low-vision community), institutional stakeholders
- BDFL transitions to Technical Lead role within foundation structure
- Foundation holds intellectual property, manages finances, employs core maintainers
- Annual public financial reports

**Governance principles:**
- Decisions affecting users require user community input (accessibility advisory board)
- Technical decisions follow the RFC process with public comment periods
- No single organization may hold more than 2 of 5+ board seats
- All governance documents are public and version-controlled

### 12.3 Sustainability Principles

- Never create user dependency without a maintenance plan -- users who adopt Luminos must be confident in its longevity
- Seek grant funding **before** the project launches, not after -- applications should be submitted during Phase 0
- Establish a contributor pipeline: documentation -> triage -> code review -> core maintainer
- Target NV Access and Blender Foundation as organizational models
- Maintain a financial reserve of 6+ months of operating expenses
- GPLv3 ensures that even if the organization fails, the codebase remains available for community continuation
- Prioritize institutional revenue over individual donations for financial stability
- Never compromise user accessibility for monetization (no paywalled features, no ads, no telemetry-for-revenue)

---

## 13. References

### Market & Demographics
1. WHO. "Vision impairment and blindness." Fact Sheet. https://www.who.int/news-room/fact-sheets/detail/blindness-and-visual-impairment
2. WHO. "World report on vision." 2019.
3. IAPB Vision Atlas. https://visionatlas.iapb.org/
4. Kings Research. "Assistive Technologies for Visually Impaired Market Size, 2032."
5. Mordor Intelligence. "Assistive Technologies for Visually Impaired Market."
6. GM Insights. "Screen Readers Software Market 2024-2032."

### User Surveys
7. WebAIM. "Screen Reader User Survey #10." 2024. https://webaim.org/projects/screenreadersurvey10/
8. WebAIM. "Survey of Users with Low Vision #2." 2018. https://webaim.org/projects/lowvisionsurvey2/
8b. WebAIM. "Survey of Users with Low Vision #1." 2013. https://webaim.org/projects/lowvisionsurvey/
9. Axess Lab. "How to make your site accessible for screen magnifiers." (Note: the 10:1 ratio cited is an unsourced assertion; use with caution.)

### Commercial Products
10. Freedom Scientific ZoomText. https://www.freedomscientific.com/products/software/zoomtext/
11. Freedom Scientific JAWS. https://www.freedomscientific.com/products/software/jaws/
12. Vispero eStore pricing. https://store.vispero.com/
13. Dolphin SuperNova. https://yourdolphin.com/SuperNova

### Open Source Tools
14. NVDA. https://www.nvaccess.org/ | https://github.com/nvaccess/nvda
15. Orca. https://orca.gnome.org/ | https://gitlab.gnome.org/GNOME/orca
16. KMag. https://apps.kde.org/kmag/ | https://github.com/KDE/kmag
17. Magnus. https://github.com/stuartlangridge/magnus
18. Virtual Magnifying Glass. https://magnifier.sourceforge.net/

### Technical References
19. screenpipe project. https://github.com/screenpipe/screenpipe (Tauri+Rust architecture validation)
20. xcap crate. https://github.com/nashaofu/xcap
21. Piper TTS (archived Oct 2025). https://github.com/rhasspy/piper | GPL fork: https://github.com/OHF-Voice/piper1-gpl
21b. Kokoro TTS model. https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX (Apache 2.0)
21c. sherpa-onnx. https://github.com/k2-fsa/sherpa-onnx (Apache 2.0, 10.8K stars)
21d. sherpa-rs. https://crates.io/crates/sherpa-rs (v0.6.8, MIT)
22. wgpu. https://github.com/gfx-rs/wgpu
23. Apple ScreenCaptureKit. https://developer.apple.com/documentation/screencapturekit/
24. Microsoft Windows.Graphics.Capture. https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture
25. XDG Desktop Portal ScreenCast. https://flatpak.github.io/xdg-desktop-portal/

### Regulatory
26. European Accessibility Act. https://commission.europa.eu/strategy-and-policy/policies/justice-and-fundamental-rights/disability/european-accessibility-act-eaa_en
27. ADA Title II (2024 update, WCAG 2.1 AA for state/local government).
28. Section 508 (2017 rule, effective Jan 2018; aligned with WCAG 2.0 AA and EN 301 549).

### Monetization & Sustainability Models
29. NV Access (NVDA). https://www.nvaccess.org/ -- Non-profit model for accessibility OSS; sustains team via donations, grants, and corporate partnerships.
30. Blender Development Fund. https://fund.blender.org/ -- Corporate sponsorship model generating millions/year from Apple, Google, NVIDIA, AMD, Intel, Meta, etc.
31. Sovereign Tech Fund. https://www.sovereigntechfund.de/ -- German/EU fund for open-source infrastructure. GNOME received EUR 1M (2024).
32. NLnet Foundation. https://nlnet.nl/ -- EU foundation funding open internet and open-source projects.
33. Microsoft AI for Accessibility. https://www.microsoft.com/en-us/ai/ai-for-accessibility -- Grant program ($25K-$500K) for AI-powered accessibility projects.
34. NIDILRR (US). https://acl.gov/about-acl/about-national-institute-disability-independent-living-and-rehabilitation-research -- US federal AT R&D funding.
35. European Accessibility Act -- monetization relevance: EAA compliance mandates (effective June 2025) create institutional demand for accessibility tools and drive sponsorship/support contract revenue. See also reference 26.
36. NV Access NVDA Certified Expert. https://certification.nvaccess.org/ -- Precedent for accessibility tool certification program.

### AI-Assisted Development
37. Chalyi. "Rust Is Winning the AI Code Generation Race" (2026).
38. Strand-Rust-Coder-v1 Technical Report. https://huggingface.co/blog/Fortytwo-Network/strand-rust-coder-tech-report

---

## 14. Version History

| Version | Date | Changes |
|---------|------|---------|
| v1.0 | 2026-03-13 | Initial product strategy document |
| v1.1 | 2026-03-13 | Competitive landscape corrections, citation improvements |
| v1.2 | 2026-03-14 | Tech stack alignment: xcap replaces scap, Kokoro replaces Piper as primary TTS, winit/cpal/arboard added, GPL risk reframed around espeak-ng |
| v1.3 | 2026-03-15 | **Major strategic pivots:** (1) GPLv3 licensing adopted -- eliminates espeak-ng GPL isolation complexity, aligns with Linux/FOSS community values. (2) Platform priority reversed to Linux-first (X11 -> Wayland -> macOS -> OpenBSD -> Windows) based on underserved-user-first strategy. (3) Comprehensive monetization strategy replacing preliminary funding model -- non-profit structure, grants, institutional sponsorship, support contracts, training/certification. (4) OpenBSD added as Phase 3 platform. (5) Risk register updated: GPL contamination risk removed, GPLv3 corporate adoption / Linux-first delay / monetization sustainability risks added. (6) Feature roadmap restructured across all phases to reflect new platform order. |

---

*This document synthesizes research from four parallel analysis tracks: competitive tools deep-dive (18 tools analyzed), technical feasibility assessment, market and regulatory analysis, and product strategy development. All claims are sourced or explicitly marked as hypotheses. Document has been reviewed by technical evaluation (see TECH_STACK_EVALUATION.md) and corrected accordingly.*
