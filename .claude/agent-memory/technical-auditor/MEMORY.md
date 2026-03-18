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
- `specs/tech-strategy/06-cross-cutting-concerns.md` - Cross-cutting concerns (audited 2026-03-17)
- `specs/tech-strategy/07-testing-strategy.md` - Testing strategy (audited 2026-03-17)

### Known Discrepancies in Source Documents (as of 2026-03-17 audit)
- 01-system-architecture.md Section 4.4 says "four-stage loop" but lists 5 stages
- 01-system-architecture.md Section 5.1 uses `pixels: Vec<u8>` for CaptureFrame but canonical def in 02 uses `data: Arc<[u8]>`
- 01-system-architecture.md Section 9.3 and 11.1 say Kokoro q8 = ~165MB; docs 04 and 05 correctly say ~92MB (verified)
- Product Strategy Phase 0 says "bilinear interpolation"; Tech Stack Eval says "bicubic interpolation"

### Criterion.rs Configuration Facts (Verified 2026-03-17)
- Criterion.toml supports ONLY: criterion_home, output_format, plotting_backend, [colors]
- significance_level, noise_threshold, confidence_level, warm_up_time, measurement_time, sample_size are set via Rust Criterion builder API, NOT via Criterion.toml
- warm_up_time takes Duration, not ms integer
- Default: warm_up=3s, measurement=5s, sample_size=100, significance=0.05, noise=0.02, confidence=0.95

### cargo-nextest Filter Syntax (Verified 2026-03-17)
- `test(~string)` = contains matcher (substring match)
- `test(=string)` = exact match
- `test(/regex/)` = regex match

### tauri-driver Limitations (Verified 2026-03-17)
- Desktop: only Windows and Linux supported
- macOS NOT supported (no WKWebView driver tool)

### GitHub Actions macOS Runner Facts (Verified 2026-03-17)
- Screen Recording permission is NOT granted by default
- Users report `UnableToAccessScreenRecordingAPIError` on macOS runners

### Verified wgpu v28 API Facts
- `wgpu::Backends::GL` is valid for GL backend reference
- `TexelCopyTextureInfo` and `TexelCopyBufferLayout` are correct type names
- `PresentMode::Mailbox` only supported on DX12 Win10, NVidia Vulkan, Wayland Vulkan (NOT macOS/Metal)

### espeak-ng CLI Facts (Verified 2026-03-15)
- `--phoneme-only` does NOT exist. Correct approach: `-q --ipa`
- `--language` does NOT exist. Correct flag: `-v <voice/language>`

### Kokoro-82M Model Facts (Verified 2026-03-15)
- q4 (int4) variant is ~80MB per 04-tts-pipeline.md CI fixture reference
- Sample rate: 24000 Hz; License: Apache 2.0

### cargo-deny Configuration Facts (Verified 2026-03-17)
- `copyleft` and `deny` fields REMOVED from modern cargo-deny
- All licenses are denied unless explicitly in `allow` list

### Common Patterns of Imprecision Found
- Performance budget numbers vary slightly across documents
- Phase attribution errors (features attributed to wrong phase)
- Illustrative code examples using wrong type names vs canonical definitions
- Criterion.toml config using fields that only exist in Rust builder API (not TOML format)
- macOS CI runner permissions claimed as default when they are NOT
- Tool/library configurations fabricated with plausible-looking but invalid fields
