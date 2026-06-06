# CLAUDE.md "Current Project Phase" status line audits

Root CLAUDE.md status maintenance (audited 2026-06-06, commit ee0d507 working tree):

## Verified facts (source-of-truth locations)
- CI active job count: `grep -E "^  [a-z].*:" .github/workflows/ci.yml` under `jobs:` →
  8 active (lint, test-rust-unit, security, coverage, test-platform, test-gpu, test-app, test-e2e)
  + 2 `if: false` placeholders (test-shaders @ ~500, test-integration @ ~511). Always exclude the if:false two.
- Crate count: 6, in Cargo.toml `members` (crates/luminos-{types,core,platform,gpu,tts,app}).
- E2E suite is ONE spec file: e2e/tests/ipc.e2e.ts (CI-only, tauri-driver+WebKitWebDriver).
- Phase 1 = "Core Magnification", Months 4-6, epics E5-E9 — per 09-implementation-roadmap.md line 39.
  Next epic after E04 is E05 (roadmap dep table: E5 starts Month 4 Week 1).
- E04 closed 2026-06-05 (7 stories). Test totals phrasing: "≈446 workspace + 67 luminos-app Rust + 70 UI Vitest".

## Known PRE-EXISTING (do NOT flag as new staleness/over-correction)
- CLAUDE.md line ~195 says "PINNED_VERSIONS.md §3" (path shorthand). File actually lives at
  specs/E04-tauri-control-panel/PINNED_VERSIONS.md — exists, just not at repo root. Pre-existing in HEAD.
- Convention mix: status line uses zero-padded E05-E09; body epic list uses non-padded E1-E4; roadmap uses E5-E9.
  Harmless, pre-existing throughout the file. Not a defect.

## E04-phase edit verdict (2026-06-06): ACCURATE, zero issues, zero over-corrections.
Only line 216 changed (Phase 0 in-progress → Phase 0 COMPLETE / entering Phase 1). Surgical, correct.
