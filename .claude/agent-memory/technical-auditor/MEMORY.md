# Technical Auditor Memory

## Project: Luminos

### Dependency / License Facts
- SPDX licenses + winit-usage nuance + overlay-mechanism ground truth: see [dep-license-facts.md](dep-license-facts.md). tao=Apache-2.0 (single), x11rb=MIT OR Apache-2.0, winit=Apache-2.0 (single). tao is transitive-only via tauri. winit still a luminos-core/gpu dep (used in core, unused in gpu/src).

### Key Source-of-Truth Files
- README.md outward-facing E04-complete status update audit (2026-06-06, commit 5a006ab): ACCURATE. See [readme-status-audit.md](readme-status-audit.md). All current-status claims verified vs repo+GT. Pre-existing nit: Project Structure tree omits luminos-types (5 shown vs "6 crates" body), out of scope. LOW pointer: status badge bumped to "Phase 1" while body says Phase 1 is "next up". UI Vitest static recount=69 vs GT 70; README matches GT so not flagged.
- E04 dep manifest audit (2026-06-04): APPROVED. See [e04-pinned-versions.md](e04-pinned-versions.md) for verified dates/peers/advisories + registry query method.
- E04 Story 004 ConfigManager impl audit (2026-06-04, commit 59cd869): AUDIT PASS. See [e04-004-impl-audit.md](e04-004-impl-audit.md). FR-7 deferral legitimate (main.rs still `fn main(){}`); seed_initial_state seam tested; 27 new tests exact; on-disk safety genuinely test-backed.
- E04 Story 002 Overlay WindowManager (x11rb over tao) + self-capture XID audit (2026-06-05, commit 7a8f984): AUDIT PASS, 0 blocking. See [e04-002-impl-audit.md](e04-002-impl-audit.md). luminos-platform tree zero-winit/zero-tauri (winit dep removed); raw_*_handle()→None (AD-3) honest; capture.rs byte-identical to HEAD (shipped set_excluded_windows@298 untouched); RISK-002 flicker/xcap-Wayland-misdetect finding genuine not masked; EWMH _NET_WM_STATE msg valid; test deltas exact (app 23→28, platform 14/14). winit in luminos-gpu is normal-dep but pre-existing + unused in gpu/src.
- E04 Story 006 Frontend UI / IPC-contract audit (2026-06-04, commit 25f8359): AUDIT PASS. See [e04-006-impl-audit.md](e04-006-impl-audit.md). Wire format 006 assumed is CORRECT vs real Rust: AppSettings snake_case (NO rename_all), enums bare-PascalCase serde-default, FrameTimingSummary camelCase is a story-005 obligation (DC-5, honestly flagged). DEFAULT_SETTINGS exact match. 70 tests exact (test.each reconciles). globals@16.5.0 used+safe. Deferrals legit.
- E04 Story 007 tray + tauri-driver E2E + EPIC ACCEPTANCE audit (2026-06-05, commit 8a4a4f3): AUDIT PASS, 0 blocking. Epic E04 honestly DONE. See [e04-007-impl-audit.md](e04-007-impl-audit.md). NO overclaim: all 8 D + 7 SC genuinely-verified OR honestly tiered CI-only/HW-manual. init_tray provably no-panic (every path Ok; 3 ?-ops confined to build_menu→match→warn+Ok(None)); degrade DETERMINISTIC (empty DBUS env forces pre-check even on bus-present box); BOTH paths verified live here. bindings.ts + Cargo.lock ZERO diff (tray-icon 0.23.1 already transitive). 446 wksp + 67 app (+7 tray). e2e npm pins registry-verified ≤2026-05-21 advisory-free. "zoom→render→frame" SC is multi-test CHAIN (hotkey region-log proves →render; pixel-present HW/manual) not one slider→pixel test. 8 active CI jobs. test-e2e authored+CI-only-never-green (carry-forward #5). 6-item carry-forward recorded.
- E04 Story 005 IPC + tauri-specta bindings audit (2026-06-05, commit 66a0ca5): AUDIT PASS, 0 blocking. See [e04-005-impl-audit.md](e04-005-impl-audit.md). CLOSES the 006 P-001 cross-language contract gap. bindings.ts == Zod == real Rust (3 legs agree, not a self-consistent false-green); BYTE-EXACT idempotent (fresh export sha256 identical, git diff --exit-code=0). FrameTimingSummary now has serde+rename_all camelCase+specta::Type (DC-5 fulfilled). ZERO specta(rename) → derive inert. specta single rc.25, NON-optional non-feature-gated on engine crates (prompt's "tauri-feature-gated" is imprecise). lossless_floats real. shell:allow-open dropped (plugin not a dep). AD-5 loop-delta-emit + CycleMode no-op TRUE. 446 wksp + 60 app (41 nextest + 19 Xvfb subprocess; 1 flaky is pre-existing story-003/DC-12). CI test-app diff step + CLAUDE.md mirror added. DC-14 accurate.
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

### E01 Story 002 Implementation Audit (2026-03-28)
- Verdict: PASS WITH FINDINGS (0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW)
- All 6 traits, all common types, all 6 error enums, module structure, PlatformBackends MATCH doc-02
- RISK-017 CaptureFrame custom Debug verified: derives Clone only, custom Debug prints [<N bytes>]
- 39/39 tests pass, clippy clean, doc clean, no unwrap/expect in production
- F-001 MEDIUM: `log` dependency in Cargo.toml but unused (pre-existing from Story 001, for E2+ backends)
- F-002 MEDIUM: `#[allow(missing_docs)]` on KeyCode suppresses NFR-3 (pragmatic, 63 self-explanatory variants)
- F-003 LOW: CaptureError::Platform source field uses thiserror implicit naming convention (correct but implicit)
- F-004 LOW: PlatformBackends test is compile-only dead_code fn, not a #[test] (needs mocks from Story 003)
- P-001: Only CaptureError::Platform has error source chain; other Platform variants may need it in E2+ (RISK-003)
- P-002: tokio "sync" feature minimal; E2+ will need rt/macros/time/process features
- Dependencies verified: tokio 1.x sync, raw-window-handle 0.6, thiserror workspace -- all appropriate
- cargo deny check: PASS (same ecosystem duplicates as Story 001)

### E01 Story 001 Implementation Audit (2026-03-28)
- Verdict: PASS WITH FINDINGS (1 CRITICAL, 1 HIGH, 2 MEDIUM, 3 INFO)
- F-001 CRITICAL: Cargo.lock NOT tracked in git (violates FR-13, doc-08 Section 2.2)
- F-002 HIGH: deny.toml has invalid fields for cargo-deny v0.19 (vulnerability/unmaintained/yanked/notice fields removed)
- F-003 MEDIUM: RUSTSEC-2026-0009 ignore reason says "quick-xml" but actual path is tauri->tauri-codegen->time
- F-004 MEDIUM: RUSTSEC-2024-0429 (glib unsoundness) missing from deny.toml ignore list
- F-005 INFO: Repository URL is luminos-accessibility/luminos, spec says luminos-app/luminos
- F-006 INFO: Many duplicate transitive deps (bindgen, bitflags, nix, rand, rustix, thiserror, zbus) -- all from ecosystem
- F-007 INFO: clippy --all-features fails without webkit2gtk system libs (known deviation #2)
- Binary size baseline: 408KB (release profile, stub binary)
- dist profile build fails due to sherpa-rs-sys bug with custom profiles (known deviation #7)
- All 7 known deviations confirmed reasonable
- 24 verification checks passed, all ACs verified except AC-3.4 (dist) and AC-4.4 (cargo-deny)
- Cargo audit: 1 vulnerability (time 0.3.45 via Tauri), 1 unsound (glib via Tauri), 19 unmaintained (all Tauri GTK3)

### cargo-deny v0.19 Configuration Facts (Verified 2026-03-28)
- [advisories] section NO LONGER supports: vulnerability, unmaintained, yanked, notice as "deny"/"warn" strings
- Expected values for those fields are now: "all", "workspace", "transitive", "none" (scope-based)
- Defaults in v0.19: vulnerabilities denied, unmaintained/yanked/notice warned (removing old fields uses correct defaults)
- ignore list format unchanged: { id = "RUSTSEC-...", reason = "..." } still works
- [bans] section unchanged: multiple-versions = "warn", wildcards = "deny" still valid

### E02 Spec Artifacts Audit (2026-03-28, FINAL)
- Verdict: PASS WITH FINDINGS -- all 3 Major RESOLVED, 3 Minor pending (non-blocking)
- 16 files reviewed: HIGH_LEVEL_PLAN.md + 5 stories x 3 files; 10 findings total
- F-001 MAJOR RESOLVED: D5 explained via sequential pipeline (no concurrent read/write hazard)
- F-002 MAJOR RESOLVED: Docked/lens ACs removed from Story 002; FullScreen only in E02
- F-009 MAJOR RESOLVED: `set_excluded_windows(&mut self, window_ids: &[u64])` now consistent across HLP, STORY.md, DESIGN.md, SUBTASKS.md
- F-003 MINOR pending: ci_platform_tests wording (cosmetic)
- F-004 MINOR handled: SUBTASKS.md T001 adds `image` workspace dep
- F-005 MINOR RESOLVED: anchor fixed to #65-event-loop-integration
- F-006 MINOR RESOLVED: bicubic scope change documented in Discovered Constraints
- F-007 MINOR pending: dead receiver vs BackendUnavailable (can fix during impl)
- F-008 MINOR pending: f32::from(u32) compile error (can fix during impl)
- F-010 MINOR deferred: xcap scale_factor return type verification
- P-001 RESOLVED: RISK-001 added to HLP with deferral note
- P-002 RESOLVED by F-002: docked mode removed from E02
- Total subtasks: 67 across 5 stories (11-14 per story)
- wgpu 28.0 API usage verified correct
- Research findings all integrated after supplementary audit
- VERIFIED: xcap 0.9.3 returns RGBA on X11 (not BGRA); doc-03 Section 4.3 is wrong for xcap path
- VERIFIED: composite pixmap self-capture prevention is INCORRECT; unmap/remap cycle is correct
- NEW PATTERN: Deliverable-to-story mapping can be inconsistent when scope is narrowed in STORY.md
- NEW PATTERN: HLP story descriptions and STORY.md scope can diverge (HLP is less detailed)
- NEW PATTERN: f32::from(u32) is NOT valid Rust -- common in AI-generated code; always verify From trait availability
- NEW PATTERN: HLP-level research corrections may not propagate to story-level specs (check all 3 files per story)
- NEW PATTERN: API signature drift between HLP and DESIGN.md -- check method names, parameter types, trait vs struct level

### SemVer Versioning Conflict Audit (2026-03-28, updated with cross-check)
- New versioning decisions: lockstep SemVer, 1.0.0 = end of Phase 1, pre-1.0 covers Phases 0-1, post-1.0 features as 1.x.y
- Source of truth: CLAUDE.md lines 300-319, README.md lines 329-339 (Versioning + Milestone Versions sections)
- CRITICAL: doc-09 Section 9.2 release schedule table says v1.0.0 = Phase 4 (Month 20); new decision says Phase 1
- CRITICAL: doc-09 Phase 4 exit criteria (line 1406) says "This is the v1.0 release"
- HIGH: doc-09 phase exit criteria: Phase 1 = v0.1.0 (line 728), Phase 2 = v0.2.0 (line 930), Phase 3 = v0.3.0 (line 1164)
- HIGH: doc-09 "Phase 4 → v1.0 Gate" checklist title (line 1559)
- MEDIUM: doc-08 line 1334 says "full Linux support" instead of "production-ready X11 magnification"
- MEDIUM: Product Strategy Section 10.3 uses v1.0/v3.0 as milestone column headers with feature counts that exceed Phase 1 scope (TTS, 4 modes)
- MEDIUM: README internal contradiction: feature roadmap (line 52) puts Wayland in Phase 1, milestone table (line 337) puts it in post-1.0 1.x.y
- MEDIUM: doc-09 Gantt chart (line 1450) shows v1.0 at Month 20
- v1.0.0 scope defined differently in 3+ documents (doc-08=Phase1, doc-09=Phase4, doc-10=Phase2, Product Strategy=ambiguous)
- Wayland scope ambiguity: Phase 1 epic (E8) in all roadmap docs, but README milestone table explicitly puts Wayland post-1.0

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
- Implementation quality high when DESIGN.md provides exact code blocks to copy (Story 002 was nearly 1:1 with design)
- Pre-provisioned dependencies (log in luminos-platform) are a recurring minor finding -- teams add deps for future use
- #[allow(missing_docs)] is pragmatic for large enums with self-explanatory variants (KeyCode, KeyModifiers)
- STORY.md ACs not updated when implementation deviates (AC-4.1/4.2 still say "luminos-platform" after luminos-types crate created)
- Tautological assertions in integration tests (e.g. `x || !x`) -- check test assertions for actual verification value
- DESIGN.md status field not updated after implementation begins (stays DRAFT)
- RustConnection (x11rb) is Send+Sync -- claims otherwise are wrong (verified 2026-03-28)
- "ZoomText conventions" attribution is imprecise -- ZoomText uses Caps Lock, not Ctrl+Alt
- Dependency direction issues when core crate imports from GPU crate for trivial math functions

### Supply-Chain Pin Audit (2026-06-03) -- see [supply-chain-pins.md](supply-chain-pins.md)
- Verdict PASS: all 25 exact `=x.y.z` workspace pins eligible/correct vs crates.io + OSV.dev
- DISCREPANCY: Cargo.toml lines 19-20 claim wgpu 29 + ashpd 0.13 need MSRV 1.92; crates.io rust_version says wgpu=1.87, ashpd=1.87, image=1.88. 1.92 is team toolchain, not these crates.
- rdev removed as dep but still named as future backend in input_monitor.rs doc table (lines 217-221)
- crates.io API gotcha: `newest_version` field unreliable (ashpd showed 0.9.3); use max_stable_version/versions list
- Verified clears: crossbeam 0.5.15>RUSTSEC-2025-0024, tauri 2.11.2>CVE-2026-42184, tokio 1.52.3>RUSTSEC-2025-0023, arc-swap 1.9.1>RUSTSEC-2020-0091

### E03 Spec Artifacts Audit (2026-03-29, FINAL)
- Verdict: APPROVED WITH FINDINGS (0 BLOCKING, 3 ADVISORY, 4 INFO, 6 pointers)
- See [e03-audit.md](e03-audit.md) for full details
- All 16 files reviewed: HLP + 5 stories x 3 files each
- F-001 ADVISORY: "ZoomText convention" claim MISLEADING (ZoomText uses Caps Lock, not Ctrl+Alt)
- F-002 ADVISORY: Story 004 SUBTASKS T002 says Ctrl+Alt+M, should be Ctrl+Alt+F1 (matches DESIGN)
- F-003 ADVISORY: Story 005 SUBTASKS T003 fabricates `modifiers` field on MouseButton/Scroll events
- F-004 INFO: luminos-core->luminos-gpu dep issue for TrackingEngine (gpu is optional dep)
- F-005 INFO: x11rb 0.13 XIEventMask is newtype with UPPER_SNAKE_CASE, Device::ALL_MASTER is type-safe API
- F-007 INFO: HLP says prefer try_recv() but DESIGN correctly uses blocking_recv() for dedicated thread
- P-001: Ctrl+Alt+F1 conflicts with Linux VT switching
- P-005: GetKeyboardMapping insufficient for non-Latin layouts in E07
- Modifiers needs Hash derive added (correctly identified in DESIGN 004)
- Cross-story consistency VERIFIED: all 5 stories consume/produce types consistently
- File ownership: no conflicts between stories

### E02 Story 002 Implementation Audit (2026-03-28)
See [e02-002-impl-audit.md](e02-002-impl-audit.md) for full details.
- Verdict: APPROVED (0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW)
- luminos-types crate: zero workspace deps, all shared types moved, backward-compatible re-exports
- 181/181 tests pass, clippy clean, fmt clean, cargo deny PASS
- RISK-017 verified: CaptureFrame custom Debug preserved in luminos-types, no pixel data in logs
- RISK-002 verified: overlay_window_id() extracts X11 ID from both Xlib and Xcb handles
- wgpu v28 API adaptations correct: request_adapter returns Result, request_device has no trace_path
- All deviations well-documented in SUBTASKS.md deviations table (5 entries)
