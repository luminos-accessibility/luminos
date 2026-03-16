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
- Docs 06-09 still needed (CI/CD, testing, build/distribution, roadmap)

## Performance Targets
- 60fps / 16.67ms frame budget (P99 < 20ms)
- <4GB RAM, <200ms TTS latency, <50MB binary, <2s startup

## Tech Stack (Canonical from TECH_STACK_EVALUATION.md)
- xcap 0.9.1 (screen capture), winit 0.30.13 (window), wgpu 28.0.0 (GPU)
- sherpa-rs 0.6.8 / sherpa-onnx (TTS runtime), Kokoro-82M (primary TTS model)
- espeak-ng subprocess (phonemizer, crash isolation), cpal (audio), arboard (clipboard)
- tauri-specta v2 + specta-typescript 0.0.9 (IPC type generation)

## Document Details → [tech-strategy-review.md](./tech-strategy-review.md)
