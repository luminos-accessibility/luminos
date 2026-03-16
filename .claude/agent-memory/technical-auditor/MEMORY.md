# Technical Auditor Memory

## Project: Luminos

### Key Source-of-Truth Files
- `specs/PRODUCT_STRATEGY.md` (v1.3) - Canonical product definition, feature roadmap by phase
- `specs/TECH_STACK_EVALUATION.md` (FINAL) - Revised tech stack (supersedes some strategy choices)
- `specs/tech-strategy/01-system-architecture.md` - System architecture, performance targets (Section 9)
- `specs/tech-strategy/02-platform-abstraction.md` - Trait definitions (ScreenCapture, WindowManager, TtsEngine, AudioOutput, etc.)
- `specs/tech-strategy/03-rendering-pipeline.md` - GPU rendering pipeline
- `specs/tech-strategy/04-tts-pipeline.md` - TTS pipeline (audited 2026-03-15)
- `specs/tech-strategy/05-control-panel.md` - Control panel IPC/UI strategy (audited 2026-03-16)

### Known Discrepancies in Source Documents (as of 2026-03-16 audit)
- 01-system-architecture.md Section 4.4 says "four-stage loop" but lists 5 stages
- 01-system-architecture.md Section 5.1 uses `pixels: Vec<u8>` for CaptureFrame but canonical def in 02 uses `data: Arc<[u8]>`
- 01-system-architecture.md Section 9.3 and 11.1 say Kokoro q8 = ~165MB; docs 04 and 05 correctly say ~92MB (verified)
- 01-system-architecture.md Section 4.7 uses `MagMode` type; doc-05 uses `MagnificationMode` (canonical)
- 01-system-architecture.md Section 4.7 uses `SpeechHandle` return type for TtsEngine::speak; doc-02 returns Result<(),TtsError> future
- Product Strategy Phase 0 says "bilinear interpolation"; Tech Stack Eval says "bicubic interpolation"
- 01-system-architecture.md Section 4.5 says TTS has "three stages"; 04-tts-pipeline.md says five stages (doc-04 acknowledges the difference)
- 05-control-panel.md status header says "DRAFT v1.0" but version history shows v1.1 applied
- 05-control-panel.md line 621 imports VoiceInfo from '../types/settings' but it's defined in '../types/tts'
- 01-system-architecture.md memory budget total (~375-465MB) based on wrong q8 size; actual total is lower

### Verified wgpu v28 API Facts
- `DeviceDescriptor` has 6 fields: label, required_features, required_limits, experimental_features, memory_hints, trace
- `TexelCopyTextureInfo` and `TexelCopyBufferLayout` are correct type names for write_texture
- `Queue::write_texture` signature: (TexelCopyTextureInfo, &[u8], TexelCopyBufferLayout, Extent3d)
- `surface.get_capabilities()` takes `&Adapter` (not `&Device`)
- `Limits::downlevel_webgl2_defaults()` has max_texture_dimension_2d: 2048
- `PresentMode::Mailbox` only supported on DX12 Win10, NVidia Vulkan, Wayland Vulkan (NOT macOS/Metal)

### WGSL Verified Facts
- `smoothstep(edge0, edge1, x)` requires edge0 < edge1 in WGSL (unlike GLSL)
- `select(falseExpr, trueExpr, condition)` - false first, true second
- `sign(0.0)` returns 0 in WGSL
- `textureSampleLevel` is correct for non-fragment contexts and avoids uniformity requirements
- for loop syntax: `for (var i = 0; i < N; i++)` is valid

### Catmull-Rom Bicubic (a=-0.5) Standard Formula
- |x| <= 1: 1.5|x|^3 - 2.5|x|^2 + 1
- 1 < |x| < 2: -0.5|x|^3 + 2.5|x|^2 - 4|x| + 2

### espeak-ng CLI Facts (Verified 2026-03-15)
- `--phoneme-only` does NOT exist. Correct approach: `-q --ipa`
- `--language` does NOT exist. Correct flag: `-v <voice/language>`
- `--stdin` IS valid; `--ipa` IS valid; `-q` suppresses audio output

### Kokoro-82M Model Facts (Verified 2026-03-15)
- v1.0 languages: en-US, en-GB, es, fr, hi, it, ja, pt-BR, zh (9 codes, 8 language groups)
- Korean (ko) NOT in Kokoro v1.0
- ONNX sizes (onnx-community): fp32=326MB, fp16=163MB, q8=92.4MB, q4f16=154MB
- q4 (int4) variant is ~80MB per 04-tts-pipeline.md CI fixture reference (NOT ~50MB)
- sherpa-onnx multi-lang v1.0 model.onnx: 310MB (fp32)
- Sample rate: 24000 Hz; License: Apache 2.0

### cpal Facts (Verified 2026-03-15)
- cpal does NOT do automatic sample rate conversion
- sndio PR #493 submitted Oct 30 2020, still not merged

### tauri-specta v2 Facts (Verified 2026-03-16)
- Latest stable RC: 2.0.0-rc.21; specta-typescript latest: 0.0.9 (not 0.0.7)
- tauri-specta MUST be in [dependencies], NOT [dev-dependencies]
- v2 API: Builder::<tauri::Wry>::new().commands(...).events(...) pattern
- The `export()` call should be in `#[cfg(debug_assertions)]` block only
- specta-typescript crate name is hyphenated in Cargo.toml: `specta-typescript = "0.0.9"`

### Tauri 2.0 JS API Facts (Verified 2026-03-16)
- `listen` import: `import { listen } from '@tauri-apps/api/event'`
- `mockIPC`, `clearMocks` import: `import { mockIPC, clearMocks } from '@tauri-apps/api/mocks'`
- React Router v7: imports come from `'react-router'` (not `'react-router-dom'`)

### Crate Version Facts (Verified 2026-03-16)
- xcap: v0.9.1 confirmed via docs.rs (released 2026-03-10). Apache 2.0.
- winit: v0.30.13 latest stable. Apache 2.0.
- wgpu: v28.0.0 confirmed. MIT/Apache 2.0.
- sherpa-rs: v0.6.8 confirmed (released 2025-10-05). MIT.
- Zustand: v5.0.11 latest. MIT. Import: `zustand/middleware/immer` is correct.

### Common Patterns of Imprecision Found
- Performance budget numbers vary slightly across documents
- Phase attribution errors (features attributed to wrong phase)
- Data sizes quoted inconsistently (especially q8 model size in doc-01 vs doc-04/05)
- Stage count inconsistencies across docs (same pipeline, different granularity)
- CLI flags fabricated for espeak-ng (common AI hallucination pattern)
- Model sizes confused between quantization variants
- Illustrative code examples using wrong type names vs canonical definitions
- Version headers not updated after revisions applied
