# Principal Product Manager - Agent Memory

## Project: Luminos
Open-source GPLv3 cross-platform screen magnification + TTS accessibility suite for low-vision users.

## Key Documents
- **Product Strategy:** `/Users/oliveren/Development/luminos/specs/PRODUCT_STRATEGY.md` (v1.3, GPLv3 + Linux-first pivot, 2026-03-15)
- **Tech Stack Evaluation:** `/Users/oliveren/Development/luminos/specs/TECH_STACK_EVALUATION.md` (FINAL, post-audit revision)
- **Project Instructions:** `/Users/oliveren/Development/luminos/CLAUDE.md`

## Strategic Decisions (v1.3)

### License: GPLv3
- Eliminates espeak-ng GPL isolation complexity entirely
- espeak-ng may still run as subprocess for crash isolation (engineering choice, not legal requirement)
- Copyleft prevents proprietary absorption, aligns with NVDA/GNOME/Linux community values
- Governance model: BDFL Year 1 -> registered non-profit foundation Year 2+

### Platform Priority: Linux-First
1. Linux X11 (Phase 0) -- zero professional magnification tools, simplest capture path
2. Linux Wayland (Phase 1) -- future-proofing, Wayland breaking existing X11 tools
3. macOS (Phase 2) -- good built-in Zoom but no open-source mag+TTS
4. OpenBSD (Phase 3) -- zero accessibility infrastructure, incremental from Linux X11
5. Windows (Phase 4) -- already has ZoomText, SuperNova, Fusion, Magnifier, VMG

Rationale: underserved-users-first, NOT market-share-driven. Linux has zero competition.

### Monetization: Non-Profit + Institutional Focus
Primary paying customers are INSTITUTIONS (universities, govt, enterprises) facing EAA/508/ADA compliance.
Revenue streams (priority order): Grants, Donations, Sponsorship, Support Contracts, Training, Consulting, Feature Sponsorship.
Year 1 target: $50K-$150K. Year 3+: $500K-$1M+.
Discarded: dual licensing, open core, selling binaries, SaaS, advertising.

## Document Alignment Status (v1.3)
PRODUCT_STRATEGY.md updated with:
- Sections 1, 7, 8.2, 8.4, 8.5, 8.7, 9.3, 10.1, 11, 12, 13, 14 modified
- Sections 2-6, 8.1, 8.3, 8.6, 9.1-9.2, 10.2-10.5 NOT modified
- Architecture diagram reordered (Linux first)
- Risk register: GPL contamination removed, 3 new risks + OpenBSD risk added
- Section 12 expanded from ~30 lines to ~120 lines (comprehensive monetization strategy)
- Section 14 (Version History) added
- Reference numbers 29+ renumbered; monetization references 29-36 added

## Architecture Decisions
- Dual-window: Tauri webview (control panel) + native winit+wgpu (magnification overlay)
- Platform order: Linux X11 first, then Wayland, then macOS, then OpenBSD, then Windows
- TTS pipeline: text -> espeak-ng (phonemes, linked directly under GPLv3) -> Kokoro ONNX inference (sherpa-rs) -> cpal audio

## Critical Cross-Document Dependencies
After v1.3, the following documents need updating to match:
- TECH_STACK_EVALUATION.md (license, platform order)
- CLAUDE.md (license, platform order, monetization references)
- specs/README.md (if any platform/license references)
