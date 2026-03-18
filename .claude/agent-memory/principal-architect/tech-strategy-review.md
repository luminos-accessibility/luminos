# Tech Strategy Review Notes (2026-03-17)

## Documents Reviewed
- 01-system-architecture.md (v1.0) - 1032 lines
- 02-platform-abstraction.md (v1.2) - ~1600 lines
- 03-rendering-pipeline.md (v1.1) - ~1300 lines
- 04-tts-pipeline.md (v1.1) - ~1070 lines
- 05-control-panel.md (v1.1) - ~1990 lines
- 06-cross-cutting-concerns.md (v1.1) - ~800 lines (written + audited 2026-03-16)
- 07-testing-strategy.md (v1.1) - ~870 lines (written + audited 2026-03-17)

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

## Doc-06 Audit Findings (2026-03-16) — All Fixed
- F-001 (HIGH): LuminosError hierarchy diverged from doc-02 canonical. Fixed: aligned with doc-02 §4.1.
- F-002 (HIGH): Tauri capability JSON had non-existent `deny` field. Fixed: removed, using omission-based denial.
- F-003 (HIGH): cargo-deny config had deprecated `copyleft`/`deny` fields. Fixed: `allow` list only.
- F-004 (MEDIUM): WCAG 2.3.1 mapped to reduced motion (wrong). Fixed: corrected to 2.3.3.
- F-005 (MEDIUM): LGPL incorrectly described as incompatible with GPLv3. Fixed: added to allowlist.
- F-006 (MEDIUM): get_system_info listed as Phase 0 (should be Phase 3 per doc-05). Fixed.
- F-007 (MEDIUM): Flicker "3Hz to 50Hz" is WCAG 1.0, not 2.x. Fixed: "three flashes per second."
- F-008 (LOW): NVDA "GPL-2.0" → "GPL-2.0-or-later." Fixed.
- F-009 (LOW): Zero-cost logging attributed to env_logger. Fixed: attributed to log crate features.
- F-010 (LOW): LUMINOS_LOG env var needs custom setup. Fixed: documented Builder::from_env.
- P-001: Memory budget discrepancy. Fixed: added supersession note.
- P-002: Signing key gap pre-foundation. Fixed: added interim Year 1 key management.
- P-003: Piper post-fork GPL-3.0 caveat. Fixed: added note.
- P-004: SPDX identifiers. Fixed: using current GPL-3.0-only/or-later format.
- P-005: TtsTimings/get_tts_timings new types. Fixed: marked as proposed.

## Doc-07 Audit Findings (2026-03-17) — All Fixed
- F-001 (HIGH): Criterion.toml config block used invalid fields (significance_level, noise_threshold, etc.). These are Rust API params, not TOML config. Fixed: replaced with Rust criterion_group! macro code.
- F-002 (MEDIUM): macOS Screen Recording claim incorrect — GitHub Actions macOS runners do NOT auto-grant this. Fixed: documented limitation and workarounds.
- F-003 (MEDIUM): Frame time warn (16.67ms) and memory warn (800MB) thresholds claimed as "consolidated from doc-06" but are new. Doc-06 only defines fail thresholds. Fixed: marked with * as new additions.
- F-004 (MEDIUM): tauri-driver not supported on macOS (no WKWebView driver). Fixed: noted in Sections 3.3 and 4.7.
- F-005 (LOW): Memory warn check (800MB) missing from benchmark script. Fixed: added.
- F-006 (LOW): New test tools not clearly distinguished from doc-05 base toolchain. Fixed: added * markers and attribution note.
- F-007 (LOW): Wayland CI terminology mismatch with doc-02 "wlheadless". Fixed: added clarifying note.
- Additional fixes: replaced choco espeak-ng with MSI download; added --benchmark-mode explanation; removed misleading Vitest alias; added E2E flakiness management policy (P-002); added coverage trend monitoring (P-003).

## Doc-08 Audit Findings (2026-03-17) — All Fixed
- F-001 (CRITICAL): `cargo tauri build --profile dist` — Tauri CLI has no `--profile` flag. Fixed: split into `cargo build --profile dist` + `cargo tauri build --bundles`.
- F-002 (HIGH): Fabricated `x11` Cargo feature gating `linux_x11` module. Doc-02 §5.2 defines no such feature. Fixed: aligned with doc-02 canonical (both modules compile unconditionally on linux).
- F-003 (HIGH): Fabricated `license` field in Tauri RpmConfig. Fixed: removed, noted RPM license from `bundle.license`.
- F-004 (HIGH): `--skip-signing` flag incorrect. Correct Tauri flag is `--no-sign`. Fixed.
- F-005 (MEDIUM): Doc-07 xref Section 7.2 → should be 7.3 (shader reference images). Fixed.
- F-006 (MEDIUM): Doc-06 xref Section 2.5 → should be 2.4 (profiling strategy). Fixed.
- F-007 (MEDIUM): Doc-04 xref Section 3.2 incorrect for espeak-ng path resolution. Fixed: updated to Section 5.
- F-008 (MEDIUM): Phase 0 packaging inconsistent with Product Strategy (deb/rpm/AppImage listed as Phase 1 P1). Fixed: added justification footnote.
- F-009 (MEDIUM): cpal version 0.15 outdated (latest is 0.17.x). Fixed: updated to 0.17.
- F-010 (LOW): `resolver = "2"` incorrect for edition 2024 (defaults to resolver 3). Fixed: removed explicit resolver.
- F-011 (LOW): Binary size check profile inconsistency with doc-07. Fixed: added clarifying note about Stage 5 vs Stage 8.
- Additional: clarified Windows signing env vars as CI workflow vars (P-002), added NSIS cross-compilation caveat (P-003), added ARM64 Linux note (P-005), fixed all `target/dist/bundle` paths to `target/release/bundle`.

## Doc-09 Audit Findings (2026-03-18) — All Fixed
- F-001 (CRITICAL): Critical path calculation had invalid E5→E9 link. Fixed: E1→E2→E8→E9→E12→E16→E17→E18→E19→E20 = 40 weeks.
- F-002 (CRITICAL): Systematic doc-02 section errors — "Section 5.1-5.5" for per-platform backends. Doc-02 has no §5.1-5.5. Fixed: all changed to §8.1-8.5.
- F-003 (CRITICAL): Systematic doc-08 section errors — off by +2 throughout. Fixed: packaging→§8, signing→§9, espeak-ng→§6, voice models→§7.
- F-004 (HIGH): Keybinding config pulled from Phase 2 to Phase 1 without note. Fixed: added phase attribution note.
- F-005 (HIGH): "Configurable" keyboard shortcuts P0 requirement partially deferred. Fixed: added explicit scope reduction note.
- F-006 (HIGH): Settings persistence pulled from Phase 1 to Phase 0 without note. Fixed: added phase attribution note.
- F-007 (HIGH): Flatpak/snap dropped from all epics (PS lists as P1). Fixed: added rationale (Tauri doesn't support them natively).
- F-008 (HIGH): doc-03 §9/§10 for color/cursor shaders wrong. Fixed: §6.3/§6.4.
- F-009 (MEDIUM): Multiple doc-03 refs off by 1. Fixed systematically.
- F-010 (MEDIUM): LuminosError location wrong (luminos-platform vs luminos-core). Fixed: luminos-core/src/error.rs.
- F-011 (MEDIUM): "Automated releases" from PS Phase 0 not covered in Phase 0. Fixed: added gap note.
- F-012 (MEDIUM): Touch/trackpad gestures missing. Fixed: added to E20 excluded list.
- F-013 (MEDIUM): E3 earliest start inconsistent between dep table and timeline. Minor visual issue.
- F-014 (MEDIUM): Test infra table incomplete vs doc-07 §13.1. Fixed: expanded Phase 0 row.
- F-015 (MEDIUM): OpenBSD CI runner phase conflict with doc-07. Fixed: added discrepancy note.
- 8 positive findings confirmed (correct type/trait names, performance targets, tauri-specta patterns, etc.)
