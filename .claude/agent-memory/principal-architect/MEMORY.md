# Principal Architect Memory

## Project: Luminos

### Key Architectural Decisions
- **License:** GPLv3 (decided v1.3, 2026-03-15). Eliminates espeak-ng GPL isolation concern.
- **Platform order:** Linux X11 -> Linux Wayland -> macOS -> OpenBSD -> Windows
- **Phase mapping:** Phase 0 = Linux X11, Phase 1 = Wayland, Phase 2 = macOS + TTS, Phase 3 = OpenBSD + AI, Phase 4 = Windows
- **espeak-ng subprocess:** Recommended for engineering reasons (crash isolation, testability), NOT legal necessity under GPLv3.

### Document Relationships
- `specs/PRODUCT_STRATEGY.md` — Canonical product definition (v1.3 as of 2026-03-15)
- `specs/TECH_STACK_EVALUATION.md` — Technical stack validation (post-audit, revised 2026-03-15)
- TECH_STACK_EVALUATION references PRODUCT_STRATEGY v1.3 Section 8.4 for GPLv3 rationale

### Technology Stack (Validated)
- Screen capture: `xcap` v0.9.1 (replaces `scap`)
- TTS: Kokoro-82M via `sherpa-rs`/sherpa-onnx (replaces Piper)
- Window mgmt: `winit` v0.30.13 (overlay window)
- GPU: `wgpu` v28.0.0
- App framework: Tauri 2.0 (control panel only)
- Phonemizer: espeak-ng as subprocess
- Audio: `cpal`
- Clipboard: `arboard`

### OpenBSD Notes
- X11 via xenocara; same XCB capture path as Linux
- Vulkan via Mesa (limited, may need software rendering fallback)
- No AT-SPI2 in base; accessibility API deferred
- winit X11 backend expected to work; build validation needed Phase 3
