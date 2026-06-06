# README.md Outward-Facing Status Audit

## 2026-06-06 (E04-complete update, commit 5a006ab)
Verdict: ACCURATE for current-status claims; one PRE-EXISTING nit; one LOW framing pointer.

### Verified against ground truth + repo
- Phase 0 COMPLETE / next Phase 1 (E05) — matches roadmap §3.2 (E5-E9 = Phase 1; E5 = Lens/Docked first Phase 1 epic). E04 HLP Status: DONE (2026-06-05).
- 6 crates: workspace members in /Cargo.toml lists exactly luminos-{types,core,platform,gpu,tts,app}.
- 8 active CI jobs: ci.yml has lint, test-rust-unit, security, coverage, test-platform, test-gpu, test-app, test-e2e. Plus 2 `if: false` placeholders (test-shaders L500, test-integration L511) — correctly NOT counted.
- IPC 7 commands + 2 events: ipc.rs collect_commands![] has 7 (get_current_settings, get_frame_timings, set_zoom_level, set_magnification_mode, toggle_magnification, save_settings, reset_settings); collect_events![ZoomChangedEvent, ModeChangedEvent] = 2.
- "first running Luminos application" + "live full-screen magnification" — verbatim from E04 HLP line 16/109. Lens/Docked deferred to E05 (Ok+warn), so "full-screen" framing is honest.
- Test counts ~446 wksp + 67 luminos-app Rust + 70 UI Vitest — match ground truth/CLAUDE.md phrasing.
- Tables well-formed (roadmap row 4 cols, platform row 3 cols). No leftover "In Progress"/"in Phase 0"/"next up: E04"/orphan 418.

### Pre-existing nit (NOT introduced by this edit)
- Project Structure tree (README L192-197) lists only 5 crates, OMITS luminos-types, while body says "6 crates". Diff confirms luminos-types was already absent before this edit; only luminos-app's comment line changed. Real inconsistency but pre-existing + outside the status-update scope.

### LOW framing pointer (defensible, not an error)
- Status badge (L13) changed Phase 0→"Phase 1 Core Magnification" (brightgreen). Phase 1 work (E05-E09) has not started; badge picks forward-looking framing while body says "Next up: Phase 1... starting with E05". Not factually false (project IS entering Phase 1) and body clarifies immediately. Prior convention: badge tracked the active phase.

### UI Vitest static-count nuance (for future audits)
- Static grep: 67 `test(` decls, 0 `it(`. One is `test.each(VALID_MODES)` with 3 modes → 66 plain + 3 = 69 runtime cases by my static count. Ground truth/CLAUDE.md assert 70. Could not run vitest (no ui/node_modules). README's "70" matches the authoritative ground truth, so NOT flagged — but the static recount lands at 69; re-verify by running `pnpm --dir ui test` if the exact UI count ever becomes load-bearing.
