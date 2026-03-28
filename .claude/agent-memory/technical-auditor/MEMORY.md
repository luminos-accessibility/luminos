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
- `specs/tech-strategy/10-risk-register.md` - Risk register (audited 2026-03-18)
- `specs/README.md` - SDD methodology (audited 2026-03-21, epic-nested layout)
- `.claude/skills/spec-driven-development/` - SDD skill (audited 2026-03-21)

### Document Section Number Maps (Verified 2026-03-18)
See [section-maps.md](section-maps.md) for per-document section structures.

### Known Discrepancies in Source Documents (as of 2026-03-21 audit)
- 01-system-architecture.md Section 4.4 says "four-stage loop" but lists 5 stages
- 01-system-architecture.md Section 5.1 uses `pixels: Vec<u8>` for CaptureFrame but canonical def in 02 uses `data: Arc<[u8]>`
- 01-system-architecture.md Section 9.3 and 11.1 say Kokoro q8 = ~165MB; docs 04 and 05 correctly say ~92MB (verified)
- Product Strategy Phase 0 says "bilinear interpolation"; Tech Stack Eval says "bicubic interpolation"
- 08-build-and-distribution.md fabricates `x11` feature flag; doc-02 only defines `wayland` and `xshm` features
- 08-build-and-distribution.md puts .deb/.rpm/AppImage in Phase 0; Product Strategy puts all 5 Linux packages in Phase 1
- 09-implementation-roadmap.md has SYSTEMATIC section number errors for doc-02, doc-03, doc-08
- 10-risk-register.md score distribution summary on line 130 is WRONG (uses fabricated categories)
- 10-risk-register.md title uses period separator ("10.") instead of double-dash ("10 --")
- 06-cross-cutting-concerns.md says LuminosError in luminos-platform/src/error.rs; doc-09 says luminos-core/src/error.rs
- specs/README.md uses "shared memory" and "Shared Context" interchangeably (template section is "Shared Context")
- Root README.md Project Structure still shows old flat NNN-story-name/ layout (not updated for epic nesting)

### SDD Methodology Structure (Verified 2026-03-21)
- Epic-nested layout: `specs/ENN-epic-name/NNN-story-name/`
- Four artifacts: HIGH_LEVEL_PLAN.md (epic), STORY.md, DESIGN.md, SUBTASKS.md (story)
- Epic folder naming: zero-padded (E01-E20), but doc-09 and CLAUDE.md prose uses non-padded (E1-E20)
- Story status values: STORY.md (DRAFT|APPROVED|IN PROGRESS|DONE|CANCELLED), SUBTASKS.md (NOT STARTED|IN PROGRESS|BLOCKED|DONE)
- Task status values: TODO|IN PROGRESS|DONE|BLOCKED
- DESIGN.md template DOES include Status field (line 281 of specs/README.md): DRAFT|IN PROGRESS|APPROVED|REVISION NEEDED

### SDD Skill Audit Findings (2026-03-21)
See [sdd-skill-audit.md](sdd-skill-audit.md) for full details.
Key gaps: DESIGN.md lifecycle oversimplified, missing TypeScript TDD exception, missing cross-epic blocker rule, AC-count splitting rule omitted from SKILL.md governance, "2-5 ACs" target added without canonical basis.

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
- WebAIM Screen Reader Survey #10: ~86% Windows (not ">90%" as sometimes cited)

### E01 HIGH_LEVEL_PLAN.md Audit (2026-03-27, re-audited 2026-03-28)
- Verdict: APPROVED (after revisions; 3 MEDIUM resolved, 1 LOW unresolved)
- F-001 RESOLVED: TrackingMode now Cursor/Focus/TextCaret per doc-05
- F-002 RESOLVED: Now ColorFilterType with SmartInvert included
- F-003 RESOLVED: PlatformBackends now correctly says "five of the six"
- F-004 RESOLVED: Scope expansion acknowledged with rationale
- F-005 NOT RESOLVED (LOW): RISK-001, RISK-017, RISK-024 still missing from Relevant Risks table
- Pre-existing discrepancy confirmed: doc-09 says GPL-3.0-or-later, doc-08 says GPL-3.0-only
- Note: RISK-017 custom Debug for CaptureFrame is a Story 002 implementation requirement

### E01 DESIGN.md Audit (2026-03-28)
- Verdict: ALL 5 DESIGNS APPROVED (PASS WITH FINDINGS, no revisions needed)
- 4 LOW findings, 5 pointers for consideration, 0 blocking issues
- Story 002 critical check: all 6 traits, all common types, all 6 error enums match doc-02 EXACTLY
- CaptureFrame custom Debug, PlatformBackends 5 fields (no TtsEngine), module structure all correct
- F-001: default=[] vs default=["wayland"] pre-existing doc-02/doc-08 discrepancy (designs follow doc-08)
- F-002: error.rs in luminos-platform is re-exports only (design more correct than doc-02 Section 5.1)
- F-003: DockEdge/LensShape duplicated between Story 002 (luminos-platform) and Story 004 (luminos-core)
- F-004: #[serde(rename_all = "PascalCase")] not shown on enums (default behavior correct but FR-4 calls for it)
- P-002 ACTIONABLE: tokio and raw-window-handle missing from workspace deps (needed by Story 002 traits)
- Cross-story consistency verified: types, features, CI commands all consistent
- NEW PATTERN: Type duplication across crate boundaries (DockEdge/LensShape in both platform and core)
- NEW PATTERN: Workspace dependency gaps when trait signatures reference external crates not yet declared

### E01 STORY.md Audit (2026-03-28, final)
- Verdict: ALL 5 STORIES APPROVED (after one revision cycle)
- Initial: 8 findings (1 HIGH, 5 MEDIUM, 2 LOW), 3 pointers
- All 6 MEDIUM+ findings resolved in revision cycle
- 103 total ACs across 5 stories, all Given-When-Then format
- D1-D6 and SC1-SC6 fully covered and testable
- 2 LOW findings accepted (F-007 nextest wording, F-008 associated type field-level ACs)
- NEW PATTERN: AudioOutput consistently omitted from enumerated lists (both 002 and 003 initially skipped it)

### Common Patterns of Imprecision Found
- Phase attribution errors (features attributed to wrong phase)
- Illustrative code examples contradicting canonical definitions in earlier docs
- Fabricated config fields (Criterion.toml, cargo-deny, Tauri RPM license)
- CLI flags that don't exist (--skip-signing, --profile for cargo tauri)
- Cross-reference section numbers off by one or more (SYSTEMATIC in doc-09)
- Tool/library configurations fabricated with plausible-looking but invalid fields
- Doc-08 section numbers are offset by 2 from what doc-09 cites (doc-09 "Section 6" = doc-08 "Section 8")
- Doc-02 per-platform specs are in Sections 2-3 (trait docs) and 6 (matrix), NOT Section 5.x
- Summary/distribution counts in tables often wrong (doc-10 score distribution)
- Terminology drift between "shared memory" and "Shared Context" in SDD docs
- Skills/summaries tend to oversimplify lifecycle transitions (dropping intermediate states)
- Enum variant names invented instead of using canonical names from doc-05 (TrackingMode, ColorFilterType)
- Trait object counts stated incorrectly (PlatformBackends has 5, not 6)
- Risk references omit risks with explicit E1 mitigation actions (check doc-10 mitigations for "in E1" text)
- AudioOutput/AudioError consistently dropped from enumerated lists when authors list "all six" traits/errors
- FR traceability claims broader AC coverage than actually exists (FR-8 claims AC-2.5 covers Voice, but it only covers AudioSample)
- "at minimum" framing in ACs allows canonical list items to be omitted (deny.toml LGPL variants)
- nextest slow-timeout and retries are commonly conflated in descriptions
- Type duplication across crate boundaries without noting re-export vs independent definition
- Workspace dependency lists incomplete when trait signatures reference crates not yet declared (tokio, raw-window-handle)
- DESIGN.md quality generally high when copying directly from canonical doc-02 signatures (low invention risk)
