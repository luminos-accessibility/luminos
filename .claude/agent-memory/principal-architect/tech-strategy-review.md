# Tech Strategy Review Notes (2026-03-16)

## Documents Reviewed
- 01-system-architecture.md (v1.0) - 1032 lines
- 02-platform-abstraction.md (v1.2) - ~1600 lines
- 03-rendering-pipeline.md (v1.1) - ~1300 lines
- 04-tts-pipeline.md (v1.1) - ~1070 lines
- 05-control-panel.md (v1.1) - ~1990 lines

## Critical Data Corrections Needed
1. Doc-01 Section 9.3: q8 model = ~92MB (not ~165MB). Total budget should be ~302-392MB.
2. Kokoro language count: Tech Stack Eval lists Korean, doc-04 omits it. Verify against HuggingFace model card.

## Naming Consistency Issues
- Doc-01 uses `Config` where `AppState` or `AppSettings` is canonical
- Doc-01 `CaptureFrame` flow diagram uses `pixels: Vec<u8>`, canonical is `data: Arc<[u8]>`
- Doc-01 uses `pixel_format`, canonical is `format: PixelFormat`
- Doc-01 `CaptureFrame` missing `stride: u32` field

## Decision Commitment Issues
- Doc-01 Section 4.6: "likely Arc<parking_lot::RwLock<Config>> or Arc<ArcSwap<Config>>"
  → Doc-05 committed to: `Arc<ArcSwap<AppState>>`
- Doc-01 Section 6.4: "ArcSwap<AppState> or atomic fields"
  → Doc-05 committed to: ArcSwap<AppState>
- Doc-01 AD-08: "ArcSwap or equivalent"
  → Should be definitive: ArcSwap<AppState>

## Cross-Reference Gaps
- FrameTimings (doc-03) missing min()/max() methods that doc-05's FrameTimingSummary needs
- Forward refs to docs 06-09 not marked as planned/TBD
- Doc-04 Section 15 has TBD for doc-05 cross-reference section numbers

## Phase Attribution
- Doc-05 "Enable TTS" toggle listed as Phase 0 but TTS is Phase 2 (schema exists early, functional later)
- Built-in profiles ship Phase 1 (doc-05), condition-based profiles Phase 3 (consistent with product strategy)

## Additional Findings from Agent Review (2026-03-16)
- Doc-01 Section 4.4: says "four-stage loop" but lists 5 stages. Doc-03 correctly says "five-stage."
- Doc-05 version header says v1.0, but version history shows v1.1 post-audit revision
- Doc-05 commands.ts example imports VoiceInfo from '../types/settings' but it's defined in '../types/tts'
- Doc-01 Section 4.3: illustrative `speak(text, voice, options) -> SpeechHandle` — SpeechHandle doesn't exist; canonical API is `speak(&self, text, interrupt) -> Result<(), TtsError>`, voice set separately
- Doc-01 uses `MagMode` type name; canonical is `MagnificationMode`
- Product Strategy and Tech Stack Eval disagree on interpolation: bilinear (PS Phase 0) vs bicubic (TSE). Doc-03 resolves: bilinear Phase 0, bicubic Phase 1
- Doc-05 cites doc-03 Section 8.3 for 33ms/30fps red threshold, but doc-03 only defines 20ms threshold
- Plugin architecture (Phase 4 P0, core value proposition) has zero technical spec — expected, but notable
- CI/CD pipeline (Phase 0 P0) has no spec — docs 07-08 needed

## Validated Claims (confirmed by auditor)
- All crate versions verified: xcap 0.9.1, winit 0.30.13, wgpu 28.0.0, sherpa-rs 0.6.8, Zustand 5.x, React Router 7
- All license claims correct
- tauri-specta v2 usage patterns correct (Builder as invoke handler, dependencies not dev-dependencies)
- Kokoro model sizes verified: fp32 ~327MB (326MB actual), fp16 ~163MB, q4 ~80MB — correct in docs 04 and 05
- 12+ cross-references spot-checked — all target sections exist and cover claimed topics
- Zero broken internal cross-references (per exhaustive xref checker)
