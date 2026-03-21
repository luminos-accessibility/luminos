# Rust Expert Agent Memory

## Project: Luminos

### Architecture Overview
- **Dual-window design:** Magnification overlay (winit + wgpu) + Control Panel (Tauri 2.0 + React)
- **Six platform traits:** `ScreenCapture`, `FocusTracker`, `TtsEngine`, `WindowManager`, `InputMonitor`, `AudioOutput`
- **Cargo workspace:** `luminos-core`, `luminos-platform`, `luminos-gpu`, `luminos-tts`, `luminos-app`
- **Platform order:** Linux X11 -> Wayland -> macOS -> OpenBSD -> Windows

### Key Files
- `specs/PRODUCT_STRATEGY.md` - Canonical product definition (v1.3)
- `specs/TECH_STACK_EVALUATION.md` - Tech stack validation (FINAL)
- `specs/tech-strategy/01-system-architecture.md` - System architecture
- `specs/tech-strategy/02-platform-abstraction.md` - Trait definitions (canonical Rust types)
- `specs/tech-strategy/03-rendering-pipeline.md` - GPU rendering pipeline (v1.1 post-audit)
- `specs/tech-strategy/04-tts-pipeline.md` - TTS pipeline (v1.1 post-audit)

### Tech Stack (pinned versions as of 2026-03-15)
- wgpu v28.0.0 (wgpu v28 uses `TexelCopyTextureInfo` not `ImageCopyTexture`)
- winit v0.30.13
- xcap v0.9.1 (screen capture)
- sherpa-rs v0.6.8 (TTS via sherpa-onnx)
- Rust 2024 edition, 1.85+

### Rendering Pipeline Patterns
- `CaptureFrame` has `Arc<[u8]>` data field, `PixelFormat` enum (Bgra8, Rgba8)
- BGRA->RGBA conversion done via shader swizzle (uniform flag), NOT CPU-side
- sRGB handled via `Rgba8UnormSrgb` texture format (automatic linearize on read)
- Phase 0 = bilinear interpolation; Phase 1 = bicubic (Catmull-Rom, a=-0.5)
- Ping-pong textures (`intermediate_texture_a`, `intermediate_texture_b`) for multi-pass
- `PresentMode::Mailbox` NOT available on macOS Metal, AMD/Intel X11 Vulkan, or OpenBSD

### TTS Pipeline Patterns (from 04-tts-pipeline.md authoring)
- espeak-ng CLI: `-q --ipa --stdin -v {lang}` (NOT --phoneme-only, NOT --language)
- Kokoro-82M q8 ONNX model is ~92MB (NOT ~165MB -- that's fp16)
- Kokoro v1.0 supports 9 language codes / ~8 language groups (NO Korean)
- cpal does NOT resample -- must use `rubato` crate for sample rate conversion
- OpenBSD does not mount /proc by default -- use sysctl/kvm_getprocs for process memory
- `AtomicF32` not in Rust std -- use `AtomicU32` + `f32::to_bits()/from_bits()`
- TTS pipeline pipelining: phonemization on Coordinator thread, synthesis on Inference thread

### Common Pitfalls (from audits)
- WGSL `smoothstep(edge0, edge1, x)` REQUIRES edge0 < edge1 (validation error otherwise)
- Catmull-Rom weights: |x|<=1: `1.5*|x|^3 - 2.5*|x|^2 + 1`, not custom polynomial
- wgpu v28 `DeviceDescriptor` has 6 fields; use `..Default::default()` for optional ones
- Contrast adjustment around 0.5 midpoint in linear space != perceptual midpoint (0.214)
- espeak-ng flags are short form (-q, -v) not GNU long form (--phoneme-only, --language)
- Model sizes from onnx-community HuggingFace may differ from sherpa-onnx distributions

### Development Conventions
- Product Strategy Phase 0 specifies bilinear; bicubic is Phase 1 "smooth text rendering"
- Cursor enhancement features are Phase 1, not Phase 0
- TTS is Phase 2; trait stubs exist from Phase 0 for boundary validation
- Performance budget: 2-8ms typical (excluding vsync), aligns with System Architecture 9.2
- Self-capture prevention is a cross-cutting concern between rendering pipeline and ScreenCapture
- Tech strategy documents use consistent structure: Overview, numbered sections, cross-references table, version history

### TDD Skill (`.claude/skills/rust-test-driven-development/`)
- **SKILL.md** (311 lines): Core TDD workflow — Red/Green/Refactor cycle, test patterns, anti-patterns, verification commands
- **references/test-patterns.md** (486 lines): Deep patterns — test doubles (fakes>stubs>mocks), async, proptest, rstest, insta, state machines, platform-gated tests
- Key patterns the skill enforces: factory closures for error injection, fakes over mockall, hierarchical test naming for nextest, generate_test_ prefix, Red-phase failure verification
- Eval results: 100% pass rate with skill vs 87% baseline; 32% fewer tokens, 19% faster
