# Technical Auditor Memory

## Project: Luminos

### Key Source-of-Truth Files
- `specs/PRODUCT_STRATEGY.md` (v1.3) - Canonical product definition, feature roadmap by phase
- `specs/TECH_STACK_EVALUATION.md` (FINAL) - Revised tech stack (supersedes some strategy choices)
- `specs/tech-strategy/01-system-architecture.md` - System architecture, performance targets (Section 9)
- `specs/tech-strategy/02-platform-abstraction.md` - Trait definitions, conditional compilation (Section 5)
- `specs/tech-strategy/03-rendering-pipeline.md` - GPU rendering pipeline
- `specs/tech-strategy/04-tts-pipeline.md` - TTS pipeline (audited 2026-03-15)
- `specs/tech-strategy/05-control-panel.md` - Control panel IPC/UI strategy (audited 2026-03-16)
- `specs/tech-strategy/06-cross-cutting-concerns.md` - Cross-cutting concerns (audited 2026-03-17)
- `specs/tech-strategy/07-testing-strategy.md` - Testing strategy (audited 2026-03-17)
- `specs/tech-strategy/08-build-and-distribution.md` - Build and distribution (audited 2026-03-17)
- `specs/tech-strategy/09-implementation-roadmap.md` - Implementation roadmap (audited 2026-03-18)

### Document Section Number Maps (Verified 2026-03-18)
See [section-maps.md](section-maps.md) for per-document section structures.

### Known Discrepancies in Source Documents (as of 2026-03-18 audit)
- 01-system-architecture.md Section 4.4 says "four-stage loop" but lists 5 stages
- 01-system-architecture.md Section 5.1 uses `pixels: Vec<u8>` for CaptureFrame but canonical def in 02 uses `data: Arc<[u8]>`
- 01-system-architecture.md Section 9.3 and 11.1 say Kokoro q8 = ~165MB; docs 04 and 05 correctly say ~92MB (verified)
- Product Strategy Phase 0 says "bilinear interpolation"; Tech Stack Eval says "bicubic interpolation"
- 08-build-and-distribution.md fabricates `x11` feature flag; doc-02 only defines `wayland` and `xshm` features
- 08-build-and-distribution.md puts .deb/.rpm/AppImage in Phase 0; Product Strategy puts all 5 Linux packages in Phase 1
- 09-implementation-roadmap.md has SYSTEMATIC section number errors for doc-02, doc-03, doc-08
- 09-implementation-roadmap.md critical path includes E5->E9 but E9's hard dep is E8, not E5

### Tauri 2.0 Configuration Facts (Verified 2026-03-17)
- DebConfig: depends, section, priority, desktopTemplate, files, changelog, conflicts -- all valid
- RpmConfig: depends, release, epoch, conflicts, provides, obsoletes, files, desktopTemplate, compression -- NO `license` field
- NsisConfig: installMode (currentUser/perMachine/both), displayLanguageSelector -- valid
- WixConfig: fipsCompliant -- valid
- `cargo tauri build` does NOT have `--profile` flag; has `--bundles`, `--no-sign` (NOT --skip-signing)
- macOS env vars: APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD, APPLE_SIGNING_IDENTITY, APPLE_API_ISSUER/KEY/KEY_PATH
- AppImage signing: SIGN, SIGN_KEY, APPIMAGETOOL_SIGN_PASSPHRASE, APPIMAGETOOL_FORCE_SIGN
- RPM signing: TAURI_SIGNING_RPM_KEY, TAURI_SIGNING_RPM_KEY_PASSPHRASE
- Updater: TAURI_SIGNING_PRIVATE_KEY; template vars {{target}}, {{arch}}, {{current_version}}
- createUpdaterArtifacts type: Updater (accepts true, false, "v1Compatible")
- cpal latest version: 0.17.3 (not 0.15)

### Other Verified Facts
- Rust edition 2024 requires rustc 1.85+ (confirmed)
- NSIS cross-compilation from Linux/macOS: possible but experimental with caveats
- cargo-cyclonedx flags: -f json, --spec-version 1.5, --manifest-path all valid

### Common Patterns of Imprecision Found
- Phase attribution errors (features attributed to wrong phase)
- Illustrative code examples contradicting canonical definitions in earlier docs
- Fabricated config fields (Criterion.toml, cargo-deny, Tauri RPM license)
- CLI flags that don't exist (--skip-signing, --profile for cargo tauri)
- Cross-reference section numbers off by one or more (SYSTEMATIC in doc-09)
- Tool/library configurations fabricated with plausible-looking but invalid fields
- Doc-08 section numbers are offset by 2 from what doc-09 cites (doc-09 "Section 6" = doc-08 "Section 8")
- Doc-02 per-platform specs are in Sections 2-3 (trait docs) and 6 (matrix), NOT Section 5.x
