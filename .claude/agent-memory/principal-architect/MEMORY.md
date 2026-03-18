# Principal Architect Memory

## Project: Luminos
- GPLv3 cross-platform screen magnification + TTS accessibility suite
- Pre-development phase: strategy docs written, no code yet

## Architecture Decisions (Confirmed)
- Dual-window: winit+wgpu overlay (render) + Tauri 2.0 webview (control panel)
- 6 platform traits: ScreenCapture, FocusTracker, TtsEngine, WindowManager, InputMonitor, AudioOutput
- 5 workspace crates: luminos-core, luminos-gpu, luminos-tts, luminos-platform, luminos-app
- Runtime state: `ArcSwap<AppState>` for lock-free render thread reads (doc-05 definitive)
- Platform order: Linux X11 → Wayland → macOS → OpenBSD → Windows

## Key Type Names
- `AppState` = runtime shared state (in ArcSwap)
- `AppSettings` = IPC-facing settings schema (Zod + serde)
- `CaptureFrame` = captured pixels (canonical def in doc-02: data: Arc<[u8]>, width, height, stride, format)
- `FrameTimings` = internal perf tracker (doc-03); `FrameTimingSummary` = IPC response (doc-05)
- `LuminosHandle` = Tauri managed state containing ArcSwap, ConfigManager, EventLoopProxy, TtsSender

## Issues Fixed (2026-03-16 review)
- Doc-01: q8 model size corrected 165→92MB, memory total recalculated
- Doc-01: ArcSwap<AppState> committed (hedging removed)
- Kokoro language count unified to 9 codes / ~8 unique languages (Korean NOT confirmed in v1.0 model; misaki G2P supports Korean separately)
- Doc-01: CaptureFrame fields fixed to match doc-02 canonical (data, stride, format)
- Doc-01: "four-stage" → "five-stage", MagMode → MagnificationMode, SpeechHandle removed
- Doc-05: version header updated v1.0 → v1.1, VoiceInfo import path fixed
- Doc-03: FrameTimings gained min/max/summary methods, 33ms threshold added
- Forward refs to docs 06-09 marked as (planned) across all docs
- Bilinear/bicubic phase progression noted in doc-01 and Tech Stack Eval
- Doc-06 written and audited (cross-cutting concerns: perf, security, licensing, a11y, observability, errors, i18n)
- Doc-07 written and audited (testing strategy: CI/CD pipeline, quality gates, release checklist, benchmarks)
- Docs 08-09 still needed (build/distribution, implementation roadmap)
- Doc-10 still needed (risk register)

## Performance Targets
- 60fps / 16.67ms frame budget (P99 < 20ms)
- <4GB RAM, <200ms TTS latency, <50MB binary, <2s startup

## Tech Stack (Canonical from TECH_STACK_EVALUATION.md)
- xcap 0.9.1 (screen capture), winit 0.30.13 (window), wgpu 28.0.0 (GPU)
- sherpa-rs 0.6.8 / sherpa-onnx (TTS runtime), Kokoro-82M (primary TTS model)
- espeak-ng subprocess (phonemizer, crash isolation), cpal (audio), arboard (clipboard)
- tauri-specta v2 + specta-typescript 0.0.9 (IPC type generation)

## Important Audit Learnings
- Tauri 2.0 capabilities: no top-level `deny` field; omission = denial. Only `permissions` array.
- cargo-deny: `copyleft` and `deny` fields REMOVED. Use `allow` list only (auto-deny rest).
- SPDX: use `GPL-3.0-only`/`GPL-3.0-or-later`, NOT deprecated `GPL-3.0`
- LGPL is compatible with GPLv3 (LGPL-2.1 §3 allows upgrade; LGPL-3.0 explicit compat)
- WCAG 2.3.1 = "Three Flashes" (not reduced motion). 2.3.3 = "Animation from Interactions" (AAA)
- `env_logger` does NOT provide compile-time filtering; `log` crate features do
- LuminosError canonical def is in doc-02 §4.1 (luminos-platform/src/error.rs)
- NVDA license is GPL-2.0-or-later (not GPL-2.0-only)
- Criterion.toml only supports output_format, plotting_backend, colors. Statistical params are Rust API only.
- tauri-driver: Linux + Windows only. macOS has no WKWebView driver tool.
- GitHub Actions macOS runners do NOT auto-grant Screen Recording permission (actions/runner-images#8951)
- espeak-ng on Windows: use MSI from GitHub releases, NOT choco (may not exist)
- doc-06 only defines fail thresholds for frame time (20ms) and memory (1GB); warn thresholds (16.67ms, 800MB) are new in doc-07

## Document Details → [tech-strategy-review.md](./tech-strategy-review.md)
