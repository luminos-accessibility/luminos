# Principal Product Manager - Agent Memory

## Project: Luminos
Open-source cross-platform screen magnification + TTS accessibility suite for low-vision users.

## Key Documents
- **Product Strategy:** `/Users/oliveren/Development/luminos/specs/PRODUCT_STRATEGY.md` (v1.2, tech stack alignment, 2026-03-14)
- **Tech Stack Evaluation:** `/Users/oliveren/Development/luminos/specs/TECH_STACK_EVALUATION.md` (FINAL, post-audit revision)
- **Project Instructions:** `/Users/oliveren/Development/luminos/CLAUDE.md`

## Document Alignment Status (v1.2)
PRODUCT_STRATEGY.md was updated to align with TECH_STACK_EVALUATION.md decisions:
- `scap` replaced with `xcap` (v0.9.1, Apache 2.0) for screen capture
- Piper TTS replaced with Kokoro-82M via sherpa-onnx (sherpa-rs bindings) as primary TTS
- `winit`, `cpal`, `arboard` explicitly added to tech stack tables
- GPL risk reframed: espeak-ng is the root cause (affects both Kokoro and Piper), not Piper itself
- Piper archived Oct 2025; retained as language fallback via same sherpa-onnx runtime
- Zoom range updated to 1.5x-20x (was 2x-16x)
- Risk register: scap immaturity row replaced with xcap X11 performance row
- Footer: AUDIT_REPORT.md reference corrected to TECH_STACK_EVALUATION.md
- Sections 1-6, 10, 12 were NOT modified (product/market/competitive content preserved)

## Critical Legal Issue
espeak-ng GPL-3.0 affects ALL offline TTS (Kokoro and Piper both need it for G2P). Subprocess isolation is recommended short-term; misaki G2P is the medium-term GPL elimination path. Legal counsel required before TTS integration work.

## Architecture Decisions
- Dual-window: Tauri webview (control panel) + native winit+wgpu (magnification overlay)
- Platform order: macOS first, then Windows, then Linux (X11 first, Wayland later)
- TTS pipeline: text -> espeak-ng subprocess (phonemes) -> Kokoro ONNX inference (sherpa-rs) -> cpal audio
