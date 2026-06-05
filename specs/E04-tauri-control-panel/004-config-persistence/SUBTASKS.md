# Subtasks: Story E04/004 -- ConfigManager & Settings Persistence

**Status:** NOT STARTED
**Started:** ---
**Completed:** ---
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 2 | 0 | 0 | 2 |
| 2. Core (load/save/atomic/recovery) | 4 | 0 | 0 | 4 |
| 3. Integration (startup seed) | 1 | 0 | 0 | 1 |
| 4. Polish & Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **8** | **0** | **0** | **8** |

---

## Phase 1: Setup

### T001 -- Deps, `ConfigError`, `ConfigFile` wrapper
**Traces to:** FR-2, NFR-2
**Status:** TODO
**Files:** `crates/luminos-core/Cargo.toml`, `crates/luminos-core/src/config/error.rs`, `crates/luminos-core/src/config/manager.rs`, `crates/luminos-core/src/config/mod.rs`, root `Cargo.toml` (workspace dep pin)

**TDD Cycle:**
1. **Red:** `config_error_display` -- `ConfigError` variants format; `app_settings_default_holds` -- `AppSettings::default()` zoom == 2.0, mode FullScreen, tracking Cursor (**already exists** — just assert, do NOT re-derive).
2. **Green:** Add `toml` (workspace pin, already present) and pin `tempfile` in `[workspace.dependencies]` (exact version, dev-dep in core). Define `ConfigError`. Define the `ConfigFile { schema_version, settings }` wrapper in `manager.rs` (do NOT modify `AppSettings`/`schema.rs`). Re-export from `mod.rs`.
3. **Refactor:** Confirm existing `schema.rs` tests still compile (no AppSettings field change).

**Completion Notes:**
>

---

### T002 -- `config_path()` (XDG → HOME)
**Traces to:** FR-1, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-core/src/config/manager.rs`

**TDD Cycle:**
1. **Red:** `config_path_xdg_then_home` -- with `XDG_CONFIG_HOME` set → `<xdg>/luminos/config.toml`; unset (HOME only) → `$HOME/.config/luminos/config.toml`; neither → `ConfigError::NoConfigDir`.
2. **Green:** Implement via `std::env` (no new crate).
3. **Refactor:** —

**Completion Notes:**
>

---

## Phase 2: Core (load/save/atomic/recovery)

### T003 -- `load()` default-on-missing + round-trip
**Traces to:** FR-2, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-core/src/config/manager.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `config_load_missing_returns_default` -- no file → defaults; path cached.
   - [ ] `config_save_load_roundtrip` -- write non-default, reload, equal.
2. **Green:** `load()` reads + `toml::from_str`, defaults if absent; `settings()` accessor.
3. **Refactor:** —

**Completion Notes:**
>

---

### T004 -- `save()` atomic (temp + fsync + rename) + dir create + 0600
**Traces to:** FR-3, FR-4, NFR-3, AC-1.2
**Status:** TODO
**Files:** `crates/luminos-core/src/config/manager.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `config_save_is_atomic` -- no `.tmp` remains; target replaced.
   - [ ] `config_save_creates_dir` -- missing dir created.
   - [ ] `config_unix_permissions_0600` (Unix-gated).
2. **Green:** temp file in same dir → flush + `sync_all` → set 0600 (Unix) → `fs::rename`; cleanup temp on error.
3. **Refactor:** Extract `atomic_write(path, bytes)` helper.

**Completion Notes:**
>

---

### T005 -- Corrupt recovery + `.bak` backup
**Traces to:** FR-5, AC-2.1
**Status:** TODO
**Files:** `crates/luminos-core/src/config/manager.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `config_load_corrupt_recovers` -- garbage → defaults + `config.toml.bak` exists + no panic.
   - [ ] `config_load_partial_invalid_recovers` -- wrong types → recover.
2. **Green:** On parse error → `warn!`, best-effort rename to `.bak`, return defaults.
3. **Refactor:** —

**Completion Notes:**
>

---

### T006 -- `reset()`
**Traces to:** FR-6, AC-2.2
**Status:** TODO
**Files:** `crates/luminos-core/src/config/manager.rs`

**TDD Cycle:**
1. **Red:** `config_reset_restores_defaults` -- reset → defaults persisted + returned.
2. **Green:** Set cache to defaults, `save`, return.
3. **Refactor:** —

**Checkpoint:** ConfigManager fully unit-tested (load/save/atomic/recover/reset).

**Completion Notes:**
>

---

## Phase 3: Integration (startup seed)

### T007 -- App seeds `AppState` from config; `LuminosHandle.config = Some(..)`
**Traces to:** FR-7, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:** `app_seeds_state_from_config` (subprocess) -- pre-write config zoom=5.0; start app; assert startup state zoom == 5.0; `config` is `Some`.
2. **Green:** At startup `ConfigManager::load()` → seed `AppState.settings` into the `ArcSwap`; store `Some(manager)` in `LuminosHandle`. On `NoConfigDir`, log + in-memory defaults.
3. **Refactor:** —

**Checkpoint:** Settings persist across restarts (D5) and apply at startup.

**Completion Notes:**
>

---

## Phase 4: Polish & Acceptance

### T008 -- Acceptance + AC matrix
**Traces to:** All ACs
**Status:** TODO
**Files:** story docs

**Verification Checklist:**
- [ ] AC-1.1 default-on-missing + path + round-trip
- [ ] AC-1.2 atomic save + reload identical + dir create
- [ ] AC-2.1 corrupt → defaults + `.bak` + no panic
- [ ] AC-2.2 reset
- [ ] AC-3.1 startup seed reflected in state
- [ ] D5 (persist + reload) demonstrated
- [ ] `cargo fmt`/clippy clean; no `unwrap`/`expect`; no new non-pinned deps
- [ ] macOS/Windows path branches documented (not implemented)

**Completion Notes:**
>

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| --- | --- | --- |
