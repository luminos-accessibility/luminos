# Design: Story E04/004 -- ConfigManager & Settings Persistence

**Story:** [STORY.md](./STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** principal-architect
**Risk Refs:** RISK-017 (sensitive data in logs — config has no secrets, but keep paths/values appropriately logged), none persistence-specific in the register

---

## Overview

Flesh out the `ConfigManager` stub (from story 001) in `luminos-core::config` into a real persistence layer for the existing `AppSettings`, using the `toml` crate (workspace pin `=1.1.2`) + serde. Atomic writes via temp-file + rename. Corrupt-file recovery to defaults with a `.bak` backup. Pure Rust, no Tauri — unit-tested with `tempfile`. The app seeds `AppState` from disk at startup.

## Architecture

### Affected Traits / Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| `luminos-core/src/config/manager.rs` | Extended (from 001 stub) | Real `ConfigManager` with load/save/reset/path. |
| `luminos-core/src/config/error.rs` | New | `ConfigError` (`thiserror`). |
| `luminos-core/src/config/schema.rs` | Unchanged | `AppSettings` already derives `Default` (verified). **Do NOT add `schema_version` here** — adding a field breaks existing struct-literal tests + serde round-trip. |
| `luminos-core/src/config/manager.rs` | New | `ConfigFile { schema_version: u32, settings: AppSettings }` file-format wrapper — version is a *file* concern, not an app setting. The wrapper is what is (de)serialized to/from `config.toml`. |
| `luminos-core/src/config/mod.rs` | Modified | Re-export `ConfigManager`, `ConfigError`. |
| `luminos-core/Cargo.toml` | Modified | Add `toml` (workspace), `dirs`/XDG resolution (or `std::env` + `$HOME`), `tempfile` (dev-dependency for tests). |
| `luminos-app/src/main.rs` | Modified | Seed `AppState.settings` from `ConfigManager::load()`; store `Some(ConfigManager)` in `LuminosHandle`. |

> **XDG resolution dependency:** prefer a tiny, well-audited approach. Options: (a) `std::env::var("XDG_CONFIG_HOME")` else `$HOME/.config` (no new dep — **preferred**, avoids adding a crate); (b) the `dirs`/`directories` crate (pin if added). Choose (a) to minimize the dependency surface (supply-chain rule).

### Data Flow
- **Startup:** `ConfigManager::load()` → settings → `AppState { settings, ..default }` → `ArcSwap::from_pointee`; `LuminosHandle.config = Some(manager)`.
- **Save (story 005 `save_settings`):** read current `AppState.settings`, `config.lock().save(&settings)`.
- **Reset (story 005 `reset_settings`):** `config.lock().reset()` → defaults → also pushed into `AppState` + wake loop.

## API Design

```rust
// luminos-core/src/config/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config I/O error at '{path}': {source}")]
    Io { path: String, #[source] source: std::io::Error },
    #[error("config serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("could not resolve config directory (no XDG_CONFIG_HOME or HOME)")]
    NoConfigDir,
}
// NOTE: deserialize errors are NOT surfaced as hard errors — load() recovers to defaults (FR-5).

// luminos-core/src/config/manager.rs
/// On-disk file format wrapper — `schema_version` is a file concern, NOT part of
/// AppSettings (so AppSettings stays unchanged and its existing tests keep compiling).
#[derive(serde::Serialize, serde::Deserialize)]
struct ConfigFile {
    #[serde(default = "default_schema_version")]
    schema_version: u32,             // current = 1
    settings: AppSettings,
}
fn default_schema_version() -> u32 { 1 }

pub struct ConfigManager {
    path: std::path::PathBuf,        // resolved config.toml
    settings: AppSettings,           // cached current settings (unwrapped)
}
// load() parses ConfigFile then keeps `.settings`; save() wraps current settings in
// ConfigFile { schema_version: 1, settings } before serializing. Unknown future
// versions are tolerated (forward-compat) — log + use settings as-is for v1.

impl ConfigManager {
    /// Resolve path, load existing config (default-on-missing, recover-on-corrupt).
    pub fn load() -> Result<Self, ConfigError>;
    /// Resolve $XDG_CONFIG_HOME/luminos/config.toml or ~/.config/luminos/config.toml.
    pub fn config_path() -> Result<std::path::PathBuf, ConfigError>;
    /// Current cached settings.
    pub fn settings(&self) -> &AppSettings;
    /// Persist atomically (temp + fsync + rename); updates the cache.
    pub fn save(&mut self, settings: &AppSettings) -> Result<(), ConfigError>;
    /// Reset to defaults and persist; returns the defaults.
    pub fn reset(&mut self) -> Result<AppSettings, ConfigError>;
}
```

**Atomic write (FR-4):**
```text
1. dir = path.parent(); create_dir_all(dir)
2. tmp = dir/.config.toml.<pid-or-counter>.tmp   // same filesystem → rename is atomic
3. write TOML to tmp; file.flush()?; file.sync_all()?   // fsync before rename
4. set permissions 0600 on Unix (NFR-3)
5. fs::rename(tmp, path)   // atomic replace
6. on any error: remove tmp, return ConfigError::Io
```
(No `Math.random`/pid via std is fine in production; in tests inject a deterministic temp name. Note: scripts/agents avoid `Date::now` only in the *workflow runtime*, not in production Rust — this is normal Rust code.)

**Corrupt recovery (FR-5):** `load()` reads the file; on `toml::from_str` error → `warn!("config.toml invalid: {e}; using defaults, backed up to config.toml.bak")`, `fs::rename(path, path.with_extension("toml.bak"))` (best-effort), return defaults.

## Error Handling
- I/O failures → `ConfigError::Io { path, source }` via `.map_err`. Serialize → `#[from] toml::ser::Error`.
- **Deserialize failures do NOT propagate** — they are recovered (defaults + `.bak`), per FR-5.
- `?` propagation; no `unwrap`/`expect` (tests may `unwrap`).
- Path resolution failure (no `XDG_CONFIG_HOME` and no `HOME`) → `ConfigError::NoConfigDir`; app logs and falls back to in-memory defaults (does not crash).

## Platform Considerations

| Platform | Config path | Notes |
|----------|-------------|-------|
| Linux/OpenBSD | `$XDG_CONFIG_HOME/luminos/config.toml` → `~/.config/luminos/config.toml` | This story's target; XDG. |
| macOS | `~/Library/Application Support/luminos/config.toml` (or XDG if set) | Path branch added in E12; structure identical. Document the macOS path now; implement later. |
| Windows | `%APPDATA%\luminos\config.toml` | E17/E18. Document now. |

(The path-resolution function is the only platform-variant point; the load/save/atomic logic is portable.)

## Testing Strategy

### Unit tests (`tempfile` temp dirs; override XDG_CONFIG_HOME to the temp dir)
- `config_load_missing_returns_default` (AC-1.1) — no file → defaults; path resolved.
- `config_save_load_roundtrip` (AC-1.1, AC-1.2) — save non-default settings, reload, assert equal.
- `config_save_is_atomic` (AC-1.2) — assert no `.tmp` left behind; target replaced; (optionally simulate write failure → original intact).
- `config_save_creates_dir` (AC-1.2) — missing dir created.
- `config_load_corrupt_recovers` (AC-2.1) — write garbage → defaults + `.bak` exists + no panic.
- `config_load_partial_invalid_recovers` (AC-2.1) — valid TOML, wrong types → recover.
- `config_reset_restores_defaults` (AC-2.2) — reset → defaults persisted.
- `config_path_xdg_then_home` (FR-1) — `XDG_CONFIG_HOME` honored; fallback to `$HOME/.config`.
- `config_unix_permissions_0600` (NFR-3, Unix-gated) — saved file mode is 0600.

### Integration
- `app_seeds_state_from_config` (AC-3.1, subprocess) — pre-write a config with zoom=5.0; start app; assert first-frame/state zoom == 5.0 and `config = Some`.

### Acceptance Tests

| AC | Test Type | Verification |
|----|-----------|--------------|
| AC-1.1 | Unit | Default-on-missing + path resolution + TOML round-trip. |
| AC-1.2 | Unit | Atomic save (temp+rename, no torn file) + reload identical + dir created. |
| AC-2.1 | Unit | Corrupt → defaults + `.bak` + no panic. |
| AC-2.2 | Unit | reset → defaults persisted. |
| AC-3.1 | Subprocess | Pre-seeded config reflected in startup state. |

## Performance Targets
- load/save are startup/explicit-action operations, not hot-loop; no specific latency target beyond "not perceptible" (NFR-1).

## Security Considerations
- `config.toml` 0600 on Unix (NFR-3). No secrets stored. Log the path, not full content, at info level.

## Alternatives Considered
1. **JSON on disk.** Rejected — roadmap/doc-01 specify TOML for human-editability; JSON reserved for IPC.
2. **Write-in-place (non-atomic).** Rejected — a crash mid-write corrupts config; temp+rename is the standard safe pattern.
3. **`dirs`/`directories` crate for paths.** Deferred — `std::env` XDG resolution avoids a new dependency (supply-chain minimization); revisit only when macOS/Windows path branches need it.
4. **Propagate deserialize errors.** Rejected — a corrupt file must never lock the user out (FR-5); recover to defaults.
