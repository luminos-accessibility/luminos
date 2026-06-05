# Story E04/004: ConfigManager & Settings Persistence

**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** 001 (the `ConfigManager` stub + `LuminosHandle.config` slot)

---

## Problem Statement

`AppSettings` (and all sub-structs: magnification, color filter, cursor, speech, keybindings, flags) already exist, fully serde-derived, in `luminos-core::config::schema`. Story 001 landed a minimal empty `ConfigManager` stub so `LuminosHandle` compiles. But there is **no persistence**: settings do not survive a restart, which makes the Phase 0 magnifier unusable for daily dogfooding (the roadmap pulled persistence into Phase 0 for exactly this reason).

This story implements `ConfigManager`: load/save `AppSettings` to `~/.config/luminos/config.toml` (XDG base-dir resolution), with **atomic writes** (temp file + rename), graceful default-on-missing and recovery-on-corrupt behavior, and a `reset()` to defaults. The app seeds the initial `AppState.settings` from disk at startup. This is pure Rust with no Tauri dependency, fully unit-testable.

## User Scenarios

> **AC count = 5.**

### US-1: Settings load and save
As a user, I want my settings saved to a file and reloaded next launch, so that I don't reconfigure the magnifier every time.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1 (load/default + round-trip):** Given no `config.toml` exists, when `ConfigManager::load()` is called, then it returns `AppSettings::default()` and resolves the path to `$XDG_CONFIG_HOME/luminos/config.toml` (falling back to `~/.config/luminos/config.toml`); and given a valid `config.toml`, when loaded, then the parsed `AppSettings` equals the written settings (TOML round-trip). *(FR-1, FR-2)*
- **AC-1.2 (atomic save):** Given an `AppSettings`, when `save(&settings)` is called, then `config.toml` is written **atomically** (write temp file in the same dir, `fsync`, rename over target), creating the directory if absent; and a subsequent `load()` yields identical settings. *(FR-3, FR-4)* — **D5**

### US-2: Robust to corruption and resettable
As a user, I want a corrupt config to not crash the app, and a way to reset to defaults, so that I'm never locked out.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1 (corrupt recovery):** Given a malformed or partially-invalid `config.toml`, when `load()` is called, then it returns `AppSettings::default()`, logs a `warn!`, backs up the bad file to `config.toml.bak`, and does NOT panic or propagate a hard error. *(FR-5)*
- **AC-2.2 (reset):** Given any current config, when `reset()` is called, then the in-memory settings become `AppSettings::default()` and are persisted to `config.toml` (atomic), returning the defaults. *(FR-6)*

### US-3: Applied at startup
As a user, I want my saved settings active the moment the app starts, so that my magnifier opens the way I left it.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1 (startup seed):** Given a `config.toml` with non-default settings, when the app starts, then `AppState.settings` is seeded from the loaded settings (the render loop and overlay reflect them on the first frame), and `LuminosHandle.config` holds `Some(ConfigManager)`. *(FR-7)*

## Functional Requirements

- **FR-1:** `ConfigManager::config_path()` MUST resolve `$XDG_CONFIG_HOME/luminos/config.toml`, falling back to `~/.config/luminos/config.toml`. *(AC-1.1)*
- **FR-2:** `load()` MUST parse existing `config.toml` into `AppSettings` (TOML via `toml` crate + serde), or return `AppSettings::default()` if absent. *(AC-1.1)*
- **FR-3:** `save(&AppSettings)` MUST serialize to TOML and create the config directory if absent. *(AC-1.2)*
- **FR-4:** `save` MUST be atomic: write to a temp file in the same directory, flush/fsync, then `rename` over the target (no torn writes). *(AC-1.2)*
- **FR-5:** `load()` on malformed/invalid TOML MUST recover: return defaults, `warn!`, back up the bad file to `config.toml.bak`; never panic. *(AC-2.1)*
- **FR-6:** `reset()` MUST set settings to defaults and persist them, returning the defaults. *(AC-2.2)*
- **FR-7:** The app startup MUST seed `AppState.settings` from `ConfigManager::load()` and store `Some(ConfigManager)` in `LuminosHandle.config`. *(AC-3.1)*

## Non-Functional Requirements

- **NFR-1:** `load`/`save` MUST be off the render hot loop (called at startup and on explicit save/reset, story 005); `LuminosHandle.config` is behind `std::sync::Mutex` (brief critical sections).
- **NFR-2:** No `unwrap()`/`expect()` in production paths; all I/O and parse failures map to `ConfigError`.
- **NFR-3:** File permissions for `config.toml` SHOULD be user-only where the platform supports it (0600 on Unix) — config may contain user preferences; not secrets, but principle of least exposure.
- **NFR-4:** TOML output SHOULD be human-readable/editable (stable key ordering, comments not required).

## Out of Scope

- Profiles (built-in + user, condition-based overrides) → Epic 9 (this story is the single `config.toml` only).
- Settings migration/versioning across schema changes → a `schema_version` key is added to the on-disk **`ConfigFile` wrapper** (NOT to `AppSettings`, which stays unchanged) to ease future migration; full migration logic is later.
- The IPC `save_settings`/`reset_settings` commands that call this → story 005 (this story exposes the `ConfigManager` API they use).
- Watching the file for external edits / hot reload → out of scope.

## Open Questions

- [x] TOML or JSON for the on-disk format? — **Resolved: TOML** (`config.toml`, per roadmap/doc-01 §5.4; human-editable). JSON is used only for IPC payloads.
- [x] Where does `AppSettings::default()` come from? — **Resolved:** it already exists (serde-derived struct in `luminos-core::config::schema`); confirm a `Default` impl exists or add one (defaults: zoom 2.0, FullScreen, Cursor tracking, etc., matching `StateManager` constants).
- [x] Should a corrupt file be deleted or preserved? — **Resolved:** preserved as `config.toml.bak` (FR-5) so users/devs can recover manual edits.
