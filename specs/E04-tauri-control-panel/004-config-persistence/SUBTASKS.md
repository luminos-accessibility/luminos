# Subtasks: Story E04/004 -- ConfigManager & Settings Persistence

**Status:** DONE (T007 app-wiring handed off to story 001 — see B001/deviations)
**Started:** 2026-06-04
**Completed:** 2026-06-04
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 2 | 2 | 0 | 0 |
| 2. Core (load/save/atomic/recovery) | 4 | 4 | 0 | 0 |
| 3. Integration (startup seed) | 1 | 1* | 0 | 0 |
| 4. Polish & Acceptance | 1 | 1 | 0 | 0 |
| **Total** | **8** | **8** | **0** | **0** |

\* T007 seam complete in `luminos-core`; the `luminos-app` wiring is handed off to story 001 (B001). 29 new tests (incl. 2 from code-review hardening — I-2 backup-preservation + M-5 dir-create-failure); full workspace suite 447 passed, 3 skipped.

---

## Phase 1: Setup

### T001 -- Deps, `ConfigError`, `ConfigFile` wrapper
**Traces to:** FR-2, NFR-2
**Status:** DONE
**Files:** `crates/luminos-core/Cargo.toml`, `crates/luminos-core/src/config/error.rs`, `crates/luminos-core/src/config/manager.rs`, `crates/luminos-core/src/config/mod.rs`, root `Cargo.toml` (workspace dep pin)

**TDD Cycle:**
1. **Red:** `config_error_display` -- `ConfigError` variants format; `app_settings_default_holds` -- `AppSettings::default()` zoom == 2.0, mode FullScreen, tracking Cursor (**already exists** — just assert, do NOT re-derive).
2. **Green:** Add `toml` (workspace pin, already present) and pin `tempfile` in `[workspace.dependencies]` (exact version, dev-dep in core). Define `ConfigError`. Define the `ConfigFile { schema_version, settings }` wrapper in `manager.rs` (do NOT modify `AppSettings`/`schema.rs`). Re-export from `mod.rs`.
3. **Refactor:** Confirm existing `schema.rs` tests still compile (no AppSettings field change).

**Completion Notes:**
> Added `directories = "=6.0.0"` and `tempfile = "=3.27.0"` to `[workspace.dependencies]`; `directories` as a `luminos-core` dependency, `tempfile` as a `[dev-dependencies]`. `toml`/`serde`/`thiserror` were already present. Created `config/error.rs` with `ConfigError` (`thiserror`) — variants `Io { path, source }`, `Serialize(#[from] toml::ser::Error)`, `NoConfigDir`. Deliberately NO deserialize variant (FR-5 recovers, never propagates). Created `config/manager.rs` with the `ConfigFile { schema_version, settings }` file-format wrapper (`#[serde(default = "default_schema_version")]`, current version = 1) — `AppSettings`/`schema.rs` untouched, all 17 existing schema/state tests still pass. `config/mod.rs` re-exports `ConfigError`, `ConfigManager`, `AppSettings`, `seed_initial_state`; crate root `lib.rs` re-exports `ConfigManager`, `ConfigError`, `seed_initial_state`.
> Red verified: `ConfigError` undeclared (E0433). Fixed test fixtures: forcing a `toml::ser::Error` needs a bare value at document root (`toml::to_string(&42i32)` errors) — a `HashMap<bool,u8>` serializes fine in `toml` 1.1.2.
> Tests (6): `config_error_display_io`, `config_error_display_serialize`, `config_error_display_no_config_dir`, `config_error_from_toml_ser_error`, `config_file_default_schema_version_is_current`, `config_file_wrap_carries_current_version`, `config_file_missing_version_defaults_on_parse`, `config_file_toml_roundtrip`, plus the `app_settings_default_holds` assertion.

---

### T002 -- `config_path()` (XDG → HOME)
**Traces to:** FR-1, AC-1.1
**Status:** DONE
**Files:** `crates/luminos-core/src/config/manager.rs`

**TDD Cycle:**
1. **Red:** `config_path_xdg_then_home` -- with `XDG_CONFIG_HOME` set → `<xdg>/luminos/config.toml`; unset (HOME only) → `$HOME/.config/luminos/config.toml`; neither → `ConfigError::NoConfigDir`.
2. **Green:** Implement via `std::env` (no new crate).
3. **Refactor:** —

**Completion Notes:**
> **DEVIATION (authoritative pin overrides DESIGN):** DESIGN.md "Alternatives Considered" preferred `std::env` (no new crate); PINNED_VERSIONS.md + the execution prompt mandate `directories = "=6.0.0"`. Resolved both: public `ConfigManager::config_path()` uses `directories::ProjectDirs::from("dev", "luminos", "luminos").config_dir()` — the application component is the **lowercase** `"luminos"` (the `APP_DIR` constant), which on Linux yields exactly `$XDG_CONFIG_HOME/luminos` (else `~/.config/luminos`), matching FR-1. (A code comment at the call site documents this lowercase intent — F-002.) It falls back to a **pure** `resolve_config_path(xdg, home)` helper that returns `ConfigError::NoConfigDir` when `ProjectDirs` yields `None`.
> The XDG→HOME→NoConfigDir branch logic lives in the pure helper so it is **deterministically unit-testable without mutating process env** (env mutation is `unsafe`/racy under nextest's parallelism). Tests (5): `config_path_xdg_then_home`, `config_path_home_fallback`, `config_path_neither_is_no_config_dir`, `config_path_ignores_relative_xdg` (XDG must be absolute per spec), `config_path_public_resolves_under_xdg_or_home` (exercises the live `directories` path).
> Red verified: `resolve_config_path`/`ConfigManager` undeclared (E0425/E0433).

---

## Phase 2: Core (load/save/atomic/recovery)

### T003 -- `load()` default-on-missing + round-trip
**Traces to:** FR-2, AC-1.1
**Status:** DONE
**Files:** `crates/luminos-core/src/config/manager.rs`

**TDD Cycle:**
1. **Red:**
   - [x] `config_load_missing_returns_default` -- no file → defaults; path cached.
   - [x] `config_save_load_roundtrip` -- write non-default, reload, equal.
2. **Green:** `load()` reads + `toml::from_str`, defaults if absent; `settings()` accessor.
3. **Refactor:** —

**Completion Notes:**
> `ConfigManager { path, settings }` with `load()` (resolves real path → `load_from`) and a private `load_from(&Path)` for temp-dir testing. Missing file (`ErrorKind::NotFound`) → defaults + `info!`; other I/O errors → `ConfigError::Io`; present file → parse `ConfigFile`, keep `.settings`. Accessors `path()` + `settings()` (both `#[must_use]`). Added `generate_test_settings()` in-module fixture (zoom 5.0, Docked, brightness 0.25, enlarged cursor, start_on_login) and a `temp_config_path()` helper (tempfile). Tests (2): `config_load_missing_returns_default`, `config_save_load_roundtrip`. Red verified: `load_from` not found (E0599).

---

### T004 -- `save()` atomic (temp + fsync + rename) + dir create + 0600
**Traces to:** FR-3, FR-4, NFR-3, AC-1.2
**Status:** DONE
**Files:** `crates/luminos-core/src/config/manager.rs`

**TDD Cycle:**
1. **Red:**
   - [x] `config_save_is_atomic` -- no `.tmp` remains; target replaced.
   - [x] `config_save_creates_dir` -- missing dir created.
   - [x] `config_unix_permissions_0600` (Unix-gated).
2. **Green:** temp file in same dir → flush + `sync_all` → set 0600 (Unix) → `fs::rename`; cleanup temp on error.
3. **Refactor:** Extract `atomic_write(path, bytes)` helper.

**Completion Notes:**
> `save(&mut self, &AppSettings)` serializes the current-version envelope via `toml::to_string_pretty` (human-readable, NFR-4), then `atomic_write(path, bytes)`, then updates the in-memory cache. Refactored the atomic logic into a free `atomic_write` helper: `create_dir_all(parent)` → unique sibling temp via `temp_path_for` (`.<name>.<pid>.<counter>.tmp`, same dir so `rename` is atomic) → `write_synced` (`write_all` + `flush` + `sync_all` + chmod 0600 on Unix) → `fs::rename` over target → remove temp on any error. `set_owner_only_permissions` is `#[cfg(unix)]`/no-op elsewhere. Tests (4): `config_save_is_atomic_no_temp_left`, `config_save_updates_cache`, `config_save_creates_dir`, `config_save_unix_permissions_0600` (`#[cfg(unix)]`). Red verified: `save` not found (E0599).
> **Review follow-up (I-1 durability):** after a successful `fs::rename`, `atomic_write` now fsyncs the **parent directory** via a best-effort, `#[cfg(unix)]`-gated `sync_parent_dir` (opens the dir as a `File`, `sync_all()`, errors ignored; no-op on non-Unix). On ext4/xfs the directory-entry change is not durable until the dir is fsync'd, so a power loss in that window could otherwise lose the rename. The temp-file data fsync is unchanged.
> **Review follow-up (M-2):** `save` now serializes a borrowing `ConfigFileRef<'a> { schema_version, settings: &AppSettings }` instead of `ConfigFile::wrap(settings.clone())`, removing the redundant pre-serialize clone; the only remaining clone is the cache update. On-disk format unchanged (structurally identical serialize). The owned `ConfigFile::wrap` is now `#[cfg(test)]`-only (round-trip tests need an owned, deserializable value).
> **Review follow-up (M-5):** added `config_save_parent_is_file_returns_io_error_no_temp_leak` — points `save` at a path whose parent is a regular file (so `create_dir_all` fails `NotADirectory`); asserts `ConfigError::Io { .. }` and that no `.tmp` is leaked.

---

### T005 -- Corrupt recovery + `.bak` backup
**Traces to:** FR-5, AC-2.1
**Status:** DONE
**Files:** `crates/luminos-core/src/config/manager.rs`

**TDD Cycle:**
1. **Red:**
   - [x] `config_load_corrupt_recovers` -- garbage → defaults + `config.toml.bak` exists + no panic.
   - [x] `config_load_partial_invalid_recovers` -- wrong types → recover.
2. **Green:** On parse error → `warn!`, best-effort rename to `.bak`, return defaults.
3. **Refactor:** —

**Completion Notes:**
> Added `recover_corrupt(path, &toml::de::Error)` to the `load_from` parse-error arm: `warn!` (with the parse error and target paths, dynamic values single-quoted), then best-effort rename to a backup, then caller returns `with_defaults`. A failed backup is logged, never propagated — never panics (NFR-2/FR-5). Tests (3): `config_load_corrupt_recovers_to_defaults_with_bak` (garbage → defaults + `.bak` preserves original bytes), `config_load_partial_invalid_recovers` (well-formed TOML, `zoom_level="not a number"` → recover + `.bak`), `config_load_recovery_does_not_overwrite_target`. Red verified: tests panicked on the missing `.bak`.
> **Review follow-up (I-2 data safety):** backup is now **non-destructive**. New `pick_backup_path(path)` returns `config.toml.bak` when free, else the first unused `config.toml.bak.<n>` (1..=1000), so an existing backup is never clobbered; `recover_corrupt` `warn!`s when it falls back to a numbered name. Added `config_load_corrupt_preserves_existing_bak` — pre-creates `config.toml.bak` with sentinel content, writes a corrupt `config.toml`, loads, and asserts (a) the original `.bak` sentinel is intact, (b) `config.toml.bak.1` holds the new corrupt bytes, (c) load returns defaults without panic.
> **Review follow-up (F-001):** rewrote `config_load_recovery_does_not_overwrite_target` to assert the real invariant (was vacuous — `unwrap_or_default()` always read `""`). It now asserts (a) the corrupt bytes are not at the live path (file moved off, `!path.exists()`), (b) the backup holds the original corrupt bytes verbatim, and (c) an unrelated pre-existing sibling file (`unrelated.txt`) is untouched.

---

### T006 -- `reset()`
**Traces to:** FR-6, AC-2.2
**Status:** DONE
**Files:** `crates/luminos-core/src/config/manager.rs`

**TDD Cycle:**
1. **Red:** `config_reset_restores_defaults` -- reset → defaults persisted + returned.
2. **Green:** Set cache to defaults, `save`, return.
3. **Refactor:** —

**Checkpoint:** ConfigManager fully unit-tested (load/save/atomic/recover/reset). **Reached — after code-review hardening: 25 manager unit tests green; full suite 447/447.**

**Completion Notes:**
> `reset(&mut self) -> Result<AppSettings, ConfigError>`: builds `AppSettings::default()`, delegates to `save()` (so the atomic write + cache update + 0600 are reused), `info!`, returns the defaults. Test (1): `config_reset_restores_defaults` — start from non-default on-disk state, reset → returned == defaults, cache == defaults, and a fresh `load_from` confirms defaults persisted to disk. Red verified: `reset` not found (E0599).

---

## Phase 3: Integration (startup seed)

### T007 -- App seeds `AppState` from config; `LuminosHandle.config = Some(..)`
**Traces to:** FR-7, AC-3.1
**Status:** DONE (seam implemented in `luminos-core`; `main.rs` wiring deferred to story 001 — see deviation)
**Files:** `crates/luminos-core/src/config/manager.rs` (instead of `crates/luminos-app/src/main.rs` — see deviation)

**TDD Cycle:**
1. **Red:** `app_seeds_state_from_config` (subprocess) -- pre-write config zoom=5.0; start app; assert startup state zoom == 5.0; `config` is `Some`.
2. **Green:** At startup `ConfigManager::load()` → seed `AppState.settings` into the `ArcSwap`; store `Some(manager)` in `LuminosHandle`. On `NoConfigDir`, log + in-memory defaults.
3. **Refactor:** —

**Checkpoint:** Settings persist across restarts (D5) and apply at startup.

**Completion Notes:**
> **DEVIATION — story-001 prerequisite missing.** The DESIGN/SUBTASKS assume story 001 already landed `LuminosHandle`, the single Tauri event loop, and a non-empty `luminos-app/src/main.rs` (the DESIGN's "Affected Modules" lists `main.rs` as *Modified, from 001*). Story 001 is **NOT STARTED**: `luminos-app/src/main.rs` is still `fn main() {}`, there is no `LuminosHandle`, no `ArcSwap<AppState>` at app level, and the execution prompt forbids pulling `tauri`/webkit into the default build (those system libs are not installed). A subprocess test that "starts the app" therefore has no app loop to start.
> **What I built instead (the actual load-bearing FR-7 logic, fully unit-tested, Tauri-free):** the reusable seeding seam in `luminos-core::config`:
> - `ConfigManager::seeded_app_state(&self) -> AppState` — `AppState { settings: cached, ..default() }`.
> - `pub fn seed_initial_state() -> Result<(AppState, ConfigManager), ConfigError>` — resolves the real path, loads (default-on-missing / recover-on-corrupt), returns the seeded `AppState` **and** the `ConfigManager` story 001 stores as `LuminosHandle.config = Some(..)`. Re-exported at crate root as `luminos_core::seed_initial_state`.
> - private path-explicit `seed_initial_state_from(&Path)` for temp-dir testing.
> Tests (3, unit, no subprocess): `config_seeded_app_state_carries_loaded_settings` (non-default settings → seeded state, transient fields default), `config_seed_initial_state_from_path_returns_state_and_manager` (zoom=5.0 on disk → seeded state zoom 5.0, manager returned with that path), `config_seed_initial_state_missing_file_yields_defaults`. Red verified: `seeded_app_state`/`seed_initial_state_from` undeclared (E0599/E0425).
> **Hand-off for story 001 (and 005):** in `setup`, call `let (state, config) = luminos_core::seed_initial_state().unwrap_or_else(|e| { log::warn!(..); (AppState::default(), <skip persistence>) });` — on `NoConfigDir`, fall back to `AppState::default()` and `LuminosHandle.config = None`; wrap `state` in `Arc::new(ArcSwap::from_pointee(state))` for `StateManager::new(..)`. The AC-3.1 end-to-end "first frame reflects disk" assertion belongs in story 001/007 once a loop exists. Logged as B001 below.

---

## Phase 4: Polish & Acceptance

### T008 -- Acceptance + AC matrix
**Traces to:** All ACs
**Status:** DONE
**Files:** story docs

**Verification Checklist:**
- [x] AC-1.1 default-on-missing + path + round-trip
- [x] AC-1.2 atomic save + reload identical + dir create
- [x] AC-2.1 corrupt → defaults + `.bak` + no panic
- [x] AC-2.2 reset
- [x] AC-3.1 startup seed reflected in state — covered by the `luminos-core` seeding seam (`seed_initial_state`); the live app-loop assertion is deferred to story 001/007 (no loop exists yet; see T007 deviation/B001)
- [x] D5 (persist + reload) demonstrated — `config_save_load_roundtrip` + `config_reset_restores_defaults` (save→reload identical)
- [x] `cargo fmt`/clippy clean; no `unwrap`/`expect` in production; no new non-pinned deps (`directories =6.0.0`, `tempfile =3.27.0` both per PINNED_VERSIONS.md)
- [x] macOS/Windows path branches documented (not implemented) — see `config_path` doc-comment + DESIGN Platform Considerations table

**AC → Test coverage matrix:**

| AC | Tests (all in `luminos-core::config`) |
|----|----------------------------------------|
| AC-1.1 | `config_load_missing_returns_default`, `config_save_load_roundtrip`, `config_path_xdg_then_home`, `config_path_home_fallback`, `config_path_neither_is_no_config_dir`, `config_path_ignores_relative_xdg`, `config_path_public_resolves_under_xdg_or_home`, `app_settings_default_holds` |
| AC-1.2 | `config_save_is_atomic_no_temp_left`, `config_save_updates_cache`, `config_save_creates_dir`, `config_save_unix_permissions_0600`, `config_save_load_roundtrip`, `config_save_parent_is_file_returns_io_error_no_temp_leak` (M-5, error path) |
| AC-2.1 | `config_load_corrupt_recovers_to_defaults_with_bak`, `config_load_partial_invalid_recovers`, `config_load_recovery_does_not_overwrite_target`, `config_load_corrupt_preserves_existing_bak` (I-2, non-destructive backup) |
| AC-2.2 | `config_reset_restores_defaults` |
| AC-3.1 | `config_seeded_app_state_carries_loaded_settings`, `config_seed_initial_state_from_path_returns_state_and_manager`, `config_seed_initial_state_missing_file_yields_defaults` (seam; live-loop assertion → story 001/007) |

**Completion Notes:**
> All 5 ACs covered by ≥1 passing unit test (27 new tests total). CI-mirror checks: `cargo fmt --all -- --check` clean; strict clippy (pedantic + unwrap_used + expect_used) clean on `--workspace --exclude luminos-app`; `cargo nextest run --profile ci --workspace --exclude luminos-app` → **445 passed, 3 skipped**; `cargo deny check licenses advisories` and `cargo audit` both exit 0 (new crates introduce no advisory/license failures). `luminos-app` default build (no `tauri` feature) still compiles. `luminos-app` is excluded from clippy/test because its `--all-features` set pulls webkit2gtk/javascriptcoregtk system libs that are not installed in this environment (pre-existing constraint, matches CLAUDE.md's `--exclude luminos-app` test command).

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | 2026-06-04 | T007's literal target (`luminos-app/src/main.rs` startup seed + `LuminosHandle.config = Some(..)` + subprocess test) requires story 001's `LuminosHandle`/Tauri event loop, which does not exist yet (001 NOT STARTED; `main.rs` is `fn main() {}`). Story 004 must not pull in Tauri (webkit/GTK not installed). | Implemented the reusable, Tauri-free seeding seam in `luminos-core` (`seed_initial_state`, `ConfigManager::seeded_app_state`) and unit-tested it. Story 001 calls it from `setup` and stores the returned manager in `LuminosHandle.config`; AC-3.1 live-loop assertion deferred to story 001/007. | RESOLVED (seam) / HANDED OFF (app wiring) |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| T002 | Used `directories` crate for `config_path()` rather than DESIGN's preferred `std::env`-only resolution; application component is lowercase `"luminos"` | PINNED_VERSIONS.md (authoritative) + execution prompt mandate `directories =6.0.0`. Kept a pure `resolve_config_path` helper for deterministic XDG→HOME→NoConfigDir unit testing; live path uses `ProjectDirs`. Same on-disk paths (FR-1 satisfied). |
| T007 | Implemented the FR-7 seam in `luminos-core::config` (`seed_initial_state`) instead of wiring `luminos-app/src/main.rs`; replaced the subprocess test with 3 unit tests | Story 001 (owner of `main.rs`/`LuminosHandle`/event loop) has not run; the app loop the subprocess test needs does not exist, and Tauri cannot be added in this environment. The seam is the actual reusable logic story 001 will call. See B001. |
| T004 (review) | Beyond DESIGN's atomic-write steps, `atomic_write` also fsyncs the **parent directory** after `rename` (I-1) | Durability: on ext4/xfs the rename's directory-entry change is not on stable storage until the dir is fsync'd. Best-effort, `#[cfg(unix)]`. DESIGN's step list pre-dated this hardening. |
| T005 (review) | DESIGN/FR-5 specify a single `config.toml.bak`; implementation falls back to numbered `config.toml.bak.<n>` when `.bak` already exists (I-2) | Data safety: never clobber an earlier backup. The common (no-prior-backup) case still uses the spec's `config.toml.bak`, so the named backup is unchanged. |
