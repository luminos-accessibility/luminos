# Luminos - Product Strategy & Roadmap

**Open-Source Cross-Platform Screen Magnification + Text-to-Speech Accessibility Suite**

**Document Status:** DRAFT v1.1 (post-audit revision)
**Date:** 2026-03-13
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
12. [Sustainability & Governance](#12-sustainability--governance-preliminary)
13. [References](#13-references)

---

## 1. Executive Summary

### The Opportunity

2.2 billion people worldwide have some form of visual impairment (WHO). Screen magnification users significantly outnumber screen reader users -- estimated at roughly 10:1 by Axess Lab, though this ratio includes casual zoom/browser users and the precise evidence base is thin. Regardless, the accessibility software ecosystem has invested disproportionately in screen readers. There is **no cross-platform, open-source, professional-grade screen magnification tool with integrated text-to-speech**. This gap affects millions of users across every operating system.

### The Solution

**Luminos** is an open-source, cross-platform (Windows, macOS, Linux) accessibility suite that unifies GPU-accelerated screen magnification with neural text-to-speech in a single application. It targets the massive underserved population of low-vision users who need more than built-in OS tools but cannot access or afford commercial alternatives like ZoomText ($905+ perpetual) or Fusion ($2,309+ perpetual).

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
| **Luminos (target)** | **Yes** | **Yes** | **Yes** |

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
| 2 | **Cross-platform reality** -- Same tool, same keybindings, Windows/macOS/Linux | Single OS each | Windows only | NVDA: Windows only; Orca: Linux only |
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

**Goal:** Core magnification engine working on one platform (macOS), proving the architecture.

| Feature | Priority | Description |
|---------|----------|-------------|
| Screen capture engine | P0 | Platform-native screen capture via `scap` crate (ScreenCaptureKit on macOS) |
| GPU-accelerated magnification | P0 | `wgpu`-based rendering with bilinear interpolation, transparent overlay window |
| Basic magnification modes | P0 | Full-screen zoom (2x-16x) with mouse-follow tracking |
| Keyboard shortcuts | P0 | Zoom in/out, toggle, reset. Configurable. |
| Smooth scrolling/panning | P0 | 60fps panning when cursor reaches magnification window edges |
| Tauri control panel shell | P0 | Basic settings window (zoom level slider, mode selection) |
| CI/CD pipeline | P0 | Build + test on macOS, automated releases |

### 7.2 Phase 1: Core Magnification (Months 4-6)

**Goal:** Full magnification feature set on macOS + Windows. Functional baseline.

| Feature | Priority | Description |
|---------|----------|-------------|
| Windows screen capture | P0 | Windows.Graphics.Capture via `windows-capture` crate |
| Lens magnification mode | P0 | Movable lens/loupe following cursor |
| Docked magnification mode | P0 | Split-screen with magnified region top/bottom/left/right |
| Cursor enhancement | P0 | Enlarged cursor, crosshairs, halo, locator animation |
| Focus tracking | P0 | Magnification follows keyboard focus, text caret, mouse pointer |
| Color inversion / filters | P1 | Full inversion, smart inversion, custom color schemes, brightness/contrast |
| High-contrast color schemes | P1 | Preset schemes (white-on-black, yellow-on-blue, green-on-black) |
| Smooth text rendering | P1 | Anti-aliased text at high magnification (shader-based smoothing) |
| Settings persistence | P1 | Save/load user configuration profiles |
| Installer packages | P1 | macOS .dmg, Windows MSI |

### 7.3 Phase 2: TTS Integration (Months 7-9)

**Goal:** Integrated text-to-speech working alongside magnification. Linux support.

| Feature | Priority | Description |
|---------|----------|-------------|
| Piper TTS engine integration | P0 | Embedded neural TTS via ONNX Runtime, 10+ language voices |
| "Read what I see" mode | P0 | TTS reads text under magnification focus (via accessibility APIs) |
| Selective TTS | P0 | Select text region, trigger speech ("read this paragraph") |
| Platform accessibility API integration | P0 | AXUIElement (macOS), UI Automation (Windows), AT-SPI2 (Linux) |
| Linux screen capture | P0 | PipeWire + XDG Portal (Wayland), X11 fallback |
| Linux full support | P0 | All Phase 0-1 features on Linux |
| Reading speed / voice control | P1 | Adjustable rate, pitch, voice selection |
| Platform-native TTS fallback | P1 | AVSpeech (macOS), SAPI (Windows), speech-dispatcher (Linux) |
| Read aloud with word highlighting | P1 | Synchronized visual highlight of current word being spoken |
| Linux packages | P1 | deb, rpm, snap, AppImage, Flatpak |

### 7.4 Phase 3: Advanced Magnification + AI (Months 10-14)

**Goal:** Feature parity with mid-tier commercial tools. AI-powered capabilities.

| Feature | Priority | Description |
|---------|----------|-------------|
| Font re-rendering engine | P0 | Re-render text at magnified size using system fonts (like xFont/TrueFonts). Key competitive differentiator. |
| On-device OCR | P0 | Vision framework (macOS), Windows OCR API, Tesseract (Linux). For apps without accessibility API support. |
| OCR-to-TTS pipeline | P0 | Automatic text extraction from images/scanned docs, fed to TTS |
| Multi-monitor support | P1 | Independent magnification per monitor |
| AI image description | P1 | On-device model describes images/charts/diagrams via TTS |
| Split-screen view | P1 | Original + magnified side by side |
| Mini-map navigator | P1 | Overview of full screen with viewport indicator for spatial orientation |
| Mouse pointer customization | P1 | Size, color, shape, animation for visibility |
| Condition-based profiles | P2 | Preset configurations optimized for AMD, glaucoma, diabetic retinopathy, etc. |
| Setup wizard | P2 | First-run wizard: "What kind of vision difficulty do you have?" |

### 7.5 Phase 4: Platform & Ecosystem (Months 15-20)

**Goal:** Plugin ecosystem, enterprise features, community growth.

| Feature | Priority | Description |
|---------|----------|-------------|
| Plugin architecture | P0 | Rust trait-based backend plugins + Tauri frontend extensions |
| Configuration sync | P1 | Cross-device settings sync via file export/import (Git-friendly JSON) |
| Enterprise deployment | P1 | GPO/MDM configuration, silent install, centralized config |
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
|  |  - AccessibilityReader trait                        |  |
|  |  - OcrEngine trait                                  |  |
|  |  - TtsEngine trait                                  |  |
|  |  - MagnificationRenderer trait                      |  |
|  +----------------------------------------------------+  |
|  |  Plugin System (trait objects + Tauri plugins)      |  |
|  +----------------------------------------------------+  |
+----------------------------------------------------------+
            |  Platform-specific backends  |
+----------------------------------------------------------+
|  macOS Backend    | Windows Backend    | Linux Backend    |
|  ScreenCaptureKit | WGC / DXGI         | PipeWire / X11   |
|  AXUIElement      | UI Automation      | AT-SPI2          |
|  Vision OCR       | Windows.Ocr        | Tesseract        |
|  AVSpeech         | SAPI               | speech-dispatcher|
|  Metal (via wgpu) | DX12 (via wgpu)    | Vulkan (via wgpu)|
+----------------------------------------------------------+
```

### 8.2 Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Application framework** | Tauri 2.0 | Lightweight (2-15MB base vs Electron's 85-100MB+), 58% less RAM; note: final app will be larger due to bundled TTS models and OCR. Proven by screenpipe (~17K stars). |
| **Backend language** | Rust | Memory-safe, zero-cost abstractions, compiler-as-reviewer for AI-generated code, excellent cross-platform crate ecosystem |
| **Frontend** | TypeScript + React | Largest LLM training corpus, optimal for AI-assisted UI development, declarative component model |
| **Screen capture** | `scap` crate | Unified API wrapping ScreenCaptureKit (macOS), WGC (Windows), PipeWire/X11 (Linux) |
| **GPU rendering** | `wgpu` | Cross-platform (Metal/DX12/Vulkan), transparent overlay window, GPU-accelerated magnification transforms |
| **TTS engine** | Piper TTS (ONNX) | Natural-sounding, CPU-efficient, **GPL-3.0 license** (due to espeak-ng dependency), 30+ languages, runs on Raspberry Pi |
| **TTS fallback** | Platform-native | AVSpeechSynthesizer (macOS), SAPI (Windows), speech-dispatcher (Linux) |
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
| Open-source license | TBD -- **GPL-3.0 likely required** if linking Piper (which is GPL due to espeak-ng); alternatives: run Piper as subprocess, or use Apache-2.0/MIT for core with GPL TTS module | **CRITICAL**: Requires immediate legal analysis |
| Core language | Rust | Decided |
| UI framework | Tauri 2.0 + React | Decided |
| GPU rendering | wgpu (Vulkan/Metal/DX12) | Decided |
| TTS engine | Piper TTS (primary), platform-native (fallback) | Decided |
| Font re-rendering approach | TBD -- research required | Phase 3 |
| AI inference | Local-first, cloud-optional | Decided |
| Plugin architecture | Rust traits (backend) + Tauri plugins (frontend) | Decided |
| Governance model | TBD (BDFL, Foundation, Core team) | Requires separate governance document |

### 8.5 Key Rust Crates

| Crate | Purpose | Maturity |
|-------|---------|----------|
| `scap` | Unified screen capture | Active, v0.1.0-beta.1 on GitHub (v0.0.8 on crates.io); docs.rs build failed on v0.0.8 -- monitor closely |
| `screencapturekit` | macOS ScreenCaptureKit bindings | Active, mature |
| `windows-capture` | Windows.Graphics.Capture bindings | Active, mature |
| `wgpu` | GPU rendering (Vulkan/Metal/DX12) | Mature, widely used |
| `ort` | ONNX Runtime bindings (for Piper TTS) | Active, mature |
| `objc2` | macOS Objective-C bindings | Mature |
| `windows` | Windows API bindings (UIA, OCR, etc.) | Mature (Microsoft-maintained) |
| `atspi` | AT-SPI2 D-Bus bindings (Linux) | Active |
| `leptess` | Tesseract OCR bindings | Stable but potentially unmaintained (~3 years since last update); evaluate `tesseract-rs` as alternative |
| `tauri` | Application framework | Mature, v2 stable |
| `winit` | Window creation | Mature |

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

**macOS:**
- ScreenCaptureKit requires Screen Recording permission (user authorization dialog)
- Accessibility API requires Accessibility permission
- ScreenCaptureKit is mandatory from macOS 15 (CGWindowListCreateImage deprecated)
- Starting platform for development (most mature Rust bindings)

**Windows:**
- Windows.Graphics.Capture recommended over legacy Magnification API (per Microsoft)
- Requires Windows 10 1803+
- MSI installer with GPO support for enterprise deployment
- Must coexist with NVDA/JAWS (many users run both magnification + screen reader)

**Linux:**
- Wayland: must use PipeWire + XDG Desktop Portal (user consent dialog)
- X11: XComposite + XGetImage fallback
- AT-SPI2 works over D-Bus (unaffected by Wayland transition)
- Must work on GNOME, KDE, and standalone WMs
- Package formats: deb, rpm, snap, AppImage, Flatpak

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

**Start narrow, expand wide:**
1. macOS first (most mature Rust bindings for ScreenCaptureKit + accessibility)
2. Add Windows (largest user base, most commercial competition)
3. Add Linux (urgent need due to Wayland transition, strong open-source community)

**Platform sequencing trade-off (explicit):** The macOS-first choice is technically motivated (best Rust bindings, cleanest APIs). However, >90% of AT users are on Windows (WebAIM data), and the strongest competitive gaps are there. Starting on macOS risks missing the primary user base for early adoption and AT specialist engagement. **Alternative considered:** Windows-first would maximize early user impact but has more complex APIs. **Decision rationale:** macOS provides the cleanest architecture validation; Windows follows in Phase 1 (months 4-6), only 3 months later. The cross-platform abstraction layer is designed so platform backends are independent -- if the team grows, Windows and macOS development can proceed in parallel.

**Validate architecture before expanding features:**
- Phase 0 proves the Tauri + Rust + wgpu architecture on a single platform
- Phase 1 proves cross-platform abstraction works
- Phase 2 proves magnification + TTS integration
- Phase 3+ builds on validated foundation

---

## 10. Success Metrics

### 10.1 Adoption

| Metric | Year 1 | Year 3 |
|--------|--------|--------|
| Total downloads (all platforms) | 50,000 | 500,000 |
| Monthly active users (opt-in) | 5,000 | 75,000 |
| Platform distribution | Win 60%, Linux 25%, macOS 15% | Win 50%, Linux 30%, macOS 20% |
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
| **Piper TTS is GPL-3.0 -- constrains project license** | **Certain** | **Critical** | Piper moved to GPL (OHF-Voice/piper1-gpl) due to espeak-ng. Options: (a) license Luminos as GPL-3.0, (b) run Piper as a separate subprocess to avoid linking, (c) use platform-native TTS only and drop Piper. **Requires immediate legal counsel.** |
| Font re-rendering is extremely hard | High | High | ZoomText's xFont and SuperNova's TrueFonts represent decades of proprietary engineering. Requires deep integration with DirectWrite (Win), Core Text (macOS), FreeType (Linux). Phase 3 timing allows research, but this may define whether Luminos can compete above ~4x zoom. Consider as potential multi-phase effort. |
| Wayland consent dialog chicken-and-egg | Certain | High | XDG Portal screen capture on Wayland requires a system dialog to grant permission. Low-vision users may need magnification to read the permission dialog. Mitigate with: session restoration tokens, clear documentation, OS-level accessibility for the dialog itself. |
| Tauri WebkitGTK rendering issues on Linux | High | Medium | Control panel uses simple forms; magnification overlay bypasses webview; CEF alternative in development |
| Platform API deprecation (especially macOS annual cycles) | Medium | High | Abstract behind traits; monitor deprecation cycles; budget for annual platform adaptation |
| Piper TTS quality insufficient for some languages | Medium | Low | Fallback to platform-native TTS; espeak-ng for unsupported languages |
| `scap` crate immaturity (beta) | Medium | Medium | v0.1.0-beta.1 on GitHub, docs.rs build failed. Maintain fallback to platform-specific crates (`screencapturekit`, `windows-capture`); contribute upstream fixes |
| Accessibility API coverage gaps | Medium | Medium | Many apps (legacy Win32, Electron, games, CAD, PDF viewers) expose minimal accessibility tree. OCR must be treated as co-primary strategy, not just fallback. |
| Low adoption despite technical quality | Medium | High | Engage AT specialists and rehab centers early; partner with NVDA community; attend CSUN/ATIA |
| Contributor burnout (common in accessibility OSS) | Medium | High | Establish sustainable governance; seek grant funding (Sovereign Tech Fund, Mozilla MOSS) |
| Commercial vendor response (Vispero price cuts) | Low | Low | Our value is cross-platform + free; price cuts validate the market |
| USB portable mode may be infeasible on Windows 11 | Medium | Low | SuperNova discontinued USB mode due to Win 11 security restrictions. Evaluate technical feasibility before committing to Phase 4. |

---

## 12. Sustainability & Governance (Preliminary)

For an accessibility tool that vulnerable users depend on daily, sustainability is a first-order concern, not an afterthought. Users who adopt Luminos cannot easily switch if the project is abandoned.

### 12.1 Funding Model Options

| Model | Precedent | Viability | Notes |
|-------|-----------|-----------|-------|
| Non-profit charity + donations | NV Access (NVDA) | Proven | NVDA sustains a small team via donations + Mozilla/Microsoft grants |
| Government grants | GNOME (Sovereign Tech Fund, EUR 1M) | Strong | EU Sovereign Tech Fund, Mozilla MOSS, NLnet Foundation |
| Corporate sponsorship | Linux Foundation projects | Medium | Companies with accessibility compliance needs (Microsoft, Google, Red Hat) |
| Paid enterprise support | Red Hat model | Medium | Free product, paid deployment/configuration/SLA for institutions |
| Bounty/feature sponsorship | Bountysource, Open Collective | Supplementary | Institutions fund specific features they need |

### 12.2 Governance

To be defined in a separate governance document. Options under consideration:
- **BDFL** (Benevolent Dictator for Life) -- simplest, but single point of failure
- **Core team** -- small elected maintainer group, most common in mid-size OSS
- **Foundation** -- appropriate if/when the project reaches significant adoption

### 12.3 Sustainability Principles

- Never create user dependency without a maintenance plan
- Seek grant funding before the project launches, not after
- Establish a contributor pipeline: documentation -> triage -> code review -> core
- Target NV Access / GNOME Foundation as organizational models

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
20. scap crate. https://github.com/CapSoftware/scap
21. Piper TTS. https://github.com/rhasspy/piper
22. wgpu. https://github.com/gfx-rs/wgpu
23. Apple ScreenCaptureKit. https://developer.apple.com/documentation/screencapturekit/
24. Microsoft Windows.Graphics.Capture. https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture
25. XDG Desktop Portal ScreenCast. https://flatpak.github.io/xdg-desktop-portal/

### Regulatory
26. European Accessibility Act. https://commission.europa.eu/strategy-and-policy/policies/justice-and-fundamental-rights/disability/european-accessibility-act-eaa_en
27. ADA Title II (2024 update, WCAG 2.1 AA for state/local government).
28. Section 508 (2017 rule, effective Jan 2018; aligned with WCAG 2.0 AA and EN 301 549).

### AI-Assisted Development
29. Chalyi. "Rust Is Winning the AI Code Generation Race" (2026).
30. Strand-Rust-Coder-v1 Technical Report. https://huggingface.co/blog/Fortytwo-Network/strand-rust-coder-tech-report

---

*This document synthesizes research from four parallel analysis tracks: competitive tools deep-dive (18 tools analyzed), technical feasibility assessment, market and regulatory analysis, and product strategy development. All claims are sourced or explicitly marked as hypotheses. Document has been reviewed by technical audit (see AUDIT_REPORT.md) and corrected accordingly.*
