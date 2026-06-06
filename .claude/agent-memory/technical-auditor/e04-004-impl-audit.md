---
name: e04-004-impl-audit
description: E04 Story 004 ConfigManager implementation audit (2026-06-04) — verdict, deferral legitimacy, test-backing facts
metadata:
  type: project
---

# E04 Story 004 (ConfigManager) Implementation Audit — 2026-06-04 (commit 59cd869)

**Verdict: AUDIT PASS** (3 non-blocking findings, 2 pointers). DONE_WITH_CONCERNS honestly scoped.

**Why:** Audited whether the FR-7 deferral was a hidden gap and whether on-disk-safety claims were truly test-backed.
**How to apply:** Reuse these verified facts when auditing E04 stories 001/005 (which wire ConfigManager) and any future config work.

## Deferral (FR-7) is LEGITIMATE
- `crates/luminos-app/src/main.rs` at 59cd869 is exactly `fn main() {}` (verified via git show).
- DESIGN.md:27 lists main.rs as "Modified (from 001)" — design presupposes story 001's LuminosHandle/event loop, which is NOT STARTED.
- Load-bearing FR-7 logic IS implemented + unit-tested in luminos-core: `seed_initial_state() -> Result<(AppState, ConfigManager), ConfigError>` (manager.rs:278) + `seeded_app_state()` (manager.rs:258). Only the main.rs wiring + live-loop AC-3.1 assertion deferred to story 001/007.

## On-disk safety — genuinely test-backed (read bodies, not names)
- Atomic write: production renames temp→target (manager.rs:319, `fs::rename`), not truncate-in-place. Test `config_save_is_atomic_no_temp_left` scans parent dir for `.tmp` and asserts empty.
- Corrupt recovery: `recover_corrupt` (manager.rs:178) does `fs::rename(path, config.toml.bak)` best-effort, failure logged not propagated. Test asserts defaults returned + `.bak` exists + `.bak` CONTAINS original garbage bytes (content preserved, not destroyed) + no panic.
- 0600: test reads `metadata().permissions().mode()`, asserts `& 0o777 == 0o600`. Production sets via `Permissions::from_mode(0o600)` on temp before rename.
- NoConfigDir branch IS covered (`config_path_neither_is_no_config_dir`, relative-XDG fallback test).

## Test count: 27 new is EXACT
- manager.rs (23 #[test]) + error.rs (4 #[test]), BOTH brand-new files in commit (git cat-file -e 59cd869~1 confirms neither existed before). 23+4=27.
- `cargo test -p luminos-core config::` → 40 passed (27 new + 13 pre-existing schema). Real and green.
- 445 full-suite = 418 (E03 baseline per CLAUDE.md) + 27. Arithmetic coherent; full suite delegated to QA.

## Findings (all non-blocking)
- F-001: `config_load_recovery_does_not_overwrite_target` (manager.rs:612) reads live path with `unwrap_or_default()` — after rename-away the file is gone, so `""` passes trivially. Test name oversells; the real recovery guarantees are in the OTHER two corrupt tests.
- F-002: code calls `ProjectDirs::from("dev","luminos","luminos")` (3rd arg = APP_DIR = "luminos") but commit msg + SUBTASKS.md T002 claim `"Luminos"`. Linux path correct either way (only app component used); matters for macOS/Windows E12/E17.
- F-003: DESIGN.md:95 says `with_extension("toml.bak")`; impl uses `with_file_name("config.toml.bak")` (correct, design text imprecise).

## Pointers
- P-001: single fixed `config.toml.bak` overwritten on repeated corruption (matches spec, single-deep salvage).
- P-002: if recovery rename fails, garbage stays at live path; next load re-recovers (idempotent, never locks out) but warns every launch until a save overwrites.

## HLP hand-off (B001) — COMPLETE
- HLP Shared Context updated with full ConfigManager API, `seed_initial_state` seam + story-001 wiring instructions, ConfigError variants (Io/Serialize/NoConfigDir, explicit "NO Deserialize variant"), ConfigFile wrapper, new pinned deps (directories =6.0.0, tempfile =3.27.0).
- B001 blocker entry present in SUBTASKS.md:194. Story 001/005 can wire without re-discovery.

## Architecture facts (verified)
- ConfigError (thiserror): Io{path:String, source:io::Error}, Serialize(#[from] toml::ser::Error), NoConfigDir. NO Deserialize variant by design (corrupt→recover, never propagate).
- On-disk wrapper `ConfigFile { schema_version: u32 (=1, serde default), settings: AppSettings }`. schema_version is FILE concern; AppSettings unchanged.
- `resolve_config_path(xdg, home)` is a PURE helper (no env access) for deterministic testing; public `config_path()` uses `directories::ProjectDirs` then falls back to the pure helper.
- AppSettings::default(): zoom 2.0, FullScreen, Cursor tracking (asserted in app_settings_default_holds).
