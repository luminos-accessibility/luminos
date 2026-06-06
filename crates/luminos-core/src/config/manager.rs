//! On-disk configuration persistence.
//!
//! [`ConfigManager`] loads and saves [`AppSettings`] to
//! `$XDG_CONFIG_HOME/luminos/config.toml` (falling back to
//! `~/.config/luminos/config.toml`). Writes are **atomic** (temp file in the
//! same directory, `fsync`, then `rename` over the target) so a crash mid-write
//! cannot corrupt the live config. A missing file yields
//! [`AppSettings::default()`]; a corrupt file is recovered to defaults and
//! backed up to `config.toml.bak` (or, if that already exists, a numbered
//! `config.toml.bak.<n>` so a prior backup is never overwritten) rather than
//! propagating a hard error.

use std::path::{Path, PathBuf};

use crate::config::AppSettings;
use crate::config::error::ConfigError;
use crate::state::AppState;

/// Current on-disk schema version. Bumped only when the file format changes in
/// a way that needs migration logic (migration itself is out of scope here).
const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Application sub-directory under the platform config directory.
const APP_DIR: &str = "luminos";
/// Config file name within the application config directory.
const CONFIG_FILE_NAME: &str = "config.toml";
/// Backup file name used when a corrupt config is recovered.
const BACKUP_FILE_NAME: &str = "config.toml.bak";

/// On-disk file format wrapper.
///
/// `schema_version` is a *file* concern, deliberately kept out of
/// [`AppSettings`] so the settings struct (and its existing tests) stay
/// unchanged. The wrapper is what is (de)serialized to/from `config.toml`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct ConfigFile {
    /// File-format version; defaults to the current version when absent so
    /// older files (or hand-written ones omitting the key) still parse.
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    /// The persisted application settings.
    settings: AppSettings,
}

/// serde default for [`ConfigFile::schema_version`].
fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

/// Borrowing serialize-only view of [`ConfigFile`].
///
/// Lets [`ConfigManager::save`] serialize the current-version envelope without
/// cloning the settings into an owned [`ConfigFile`] first (the only clone left
/// in `save` is the cache update). Serialization is structurally identical to
/// [`ConfigFile`], so the on-disk format is unchanged.
#[derive(serde::Serialize)]
struct ConfigFileRef<'a> {
    schema_version: u32,
    settings: &'a AppSettings,
}

impl<'a> ConfigFileRef<'a> {
    /// Borrow settings into the current-version file envelope.
    fn wrap(settings: &'a AppSettings) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            settings,
        }
    }
}

#[cfg(test)]
impl ConfigFile {
    /// Wrap settings in an owned current-version file envelope.
    ///
    /// Test-only: production [`ConfigManager::save`] serializes via the
    /// borrowing [`ConfigFileRef`]; tests use this when they need an owned,
    /// deserializable [`ConfigFile`] (e.g. round-trip assertions).
    fn wrap(settings: AppSettings) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            settings,
        }
    }
}

/// Persistence layer for [`AppSettings`].
///
/// Holds the resolved `config.toml` path and a cache of the most recently
/// loaded/saved settings. Construct with [`ConfigManager::load`], which is
/// default-on-missing and recover-on-corrupt.
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Resolved absolute path to `config.toml`.
    path: PathBuf,
    /// Cached current settings (the unwrapped [`ConfigFile::settings`]).
    settings: AppSettings,
}

/// Resolve the `config.toml` path from explicit `XDG_CONFIG_HOME` / `HOME`
/// values, applying XDG base-directory rules.
///
/// Resolution order (per FR-1):
/// 1. `$XDG_CONFIG_HOME/luminos/config.toml` when it is an **absolute** path.
/// 2. `$HOME/.config/luminos/config.toml` otherwise.
/// 3. [`ConfigError::NoConfigDir`] when neither is usable.
///
/// This is a pure function (no environment access) so the resolution logic is
/// deterministically unit-testable; [`ConfigManager::config_path`] feeds it the
/// live environment.
fn resolve_config_path(
    xdg_config_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Result<PathBuf, ConfigError> {
    let config_dir = xdg_config_home
        .filter(|p| p.is_absolute())
        .or_else(|| home.map(|h| h.join(".config")))
        .ok_or(ConfigError::NoConfigDir)?;
    Ok(config_dir.join(APP_DIR).join(CONFIG_FILE_NAME))
}

/// Read an environment variable as a non-empty [`PathBuf`].
///
/// An unset or empty variable yields `None` so that, e.g., `XDG_CONFIG_HOME=""`
/// is treated as absent rather than resolving to the current directory.
fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

impl ConfigManager {
    /// Resolve `$XDG_CONFIG_HOME/luminos/config.toml` (falling back to
    /// `~/.config/luminos/config.toml`).
    ///
    /// Uses the [`directories`] crate (`ProjectDirs`) for the platform-correct
    /// config directory — on Linux/OpenBSD this honours `$XDG_CONFIG_HOME` then
    /// `$HOME/.config`, and on macOS/Windows it will yield the native location
    /// once those branches are wired (E12/E17). When `ProjectDirs` cannot
    /// resolve a base directory (no `HOME`/`XDG_CONFIG_HOME`), it falls back to
    /// the explicit XDG resolver, which returns [`ConfigError::NoConfigDir`].
    ///
    /// # Errors
    /// Returns [`ConfigError::NoConfigDir`] when no config directory can be
    /// resolved.
    pub fn config_path() -> Result<PathBuf, ConfigError> {
        // The application component is intentionally the lowercase `"luminos"`
        // (`APP_DIR`): on Linux `directories` lowercases/strips the app name to
        // build the project path, so `ProjectDirs::from("dev", "luminos",
        // "luminos").config_dir()` yields the spec-mandated
        // `$XDG_CONFIG_HOME/luminos` (else `~/.config/luminos`). A capitalized
        // "Luminos" would resolve identically on Linux but is avoided so the
        // literal matches the documented path exactly.
        if let Some(dirs) = directories::ProjectDirs::from("dev", APP_DIR, APP_DIR) {
            return Ok(dirs.config_dir().join(CONFIG_FILE_NAME));
        }
        resolve_config_path(env_path("XDG_CONFIG_HOME"), env_path("HOME"))
    }

    /// Load the configuration from the resolved `config.toml` path.
    ///
    /// Default-on-missing and recover-on-corrupt: a missing file yields
    /// [`AppSettings::default()`]; a malformed file is recovered to defaults and
    /// backed up (see [`ConfigManager::load_from`]).
    ///
    /// # Errors
    /// Returns [`ConfigError::NoConfigDir`] when the config path cannot be
    /// resolved, or [`ConfigError::Io`] on an unexpected filesystem error.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path()?;
        Self::load_from(&path)
    }

    /// Load from an explicit `config.toml` path (bypassing XDG resolution).
    ///
    /// Same default-on-missing / recover-on-corrupt semantics as [`load`]: a
    /// missing file yields [`AppSettings::default()`]; a malformed file is
    /// recovered to defaults and backed up. Used by [`load`] (which resolves the
    /// path from the environment), by the startup seam, and by callers that need
    /// a manager rooted at an explicit path (e.g. tests against a temp dir, or
    /// platform branches that resolve the path differently).
    ///
    /// # Errors
    /// Returns [`ConfigError::Io`] on an unexpected filesystem error while
    /// reading the file (a missing file is not an error).
    ///
    /// [`load`]: ConfigManager::load
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let raw = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                log::info!(
                    "no config file at '{}'; using default settings",
                    path.display()
                );
                return Ok(Self::with_defaults(path));
            }
            Err(err) => {
                return Err(ConfigError::Io {
                    path: path.display().to_string(),
                    source: err,
                });
            }
        };

        match toml::from_str::<ConfigFile>(&raw) {
            Ok(file) => Ok(Self {
                path: path.to_path_buf(),
                settings: file.settings,
            }),
            Err(err) => {
                Self::recover_corrupt(path, &err);
                Ok(Self::with_defaults(path))
            }
        }
    }

    /// Recover from a corrupt/unparseable config: log a warning and move the
    /// bad file aside (best-effort) so the user can inspect any hand-edits, then
    /// the caller falls back to defaults (FR-5).
    ///
    /// Backup naming is **non-destructive**: the common case uses
    /// `config.toml.bak`, but if that already exists (e.g. a prior recovery),
    /// the bad file is moved to the first free numbered fallback
    /// (`config.toml.bak.1`, `.2`, …) so an earlier backup is never overwritten.
    ///
    /// Never panics or propagates: a failed backup is logged and ignored — the
    /// priority is keeping the user out of a locked-out state.
    fn recover_corrupt(path: &Path, err: &toml::de::Error) {
        let backup = pick_backup_path(path);
        log::warn!(
            "config file '{}' is invalid ({}); using default settings, backing up to '{}'",
            path.display(),
            err,
            backup.display()
        );
        if backup.file_name() != Some(std::ffi::OsStr::new(BACKUP_FILE_NAME)) {
            log::warn!(
                "existing backup '{}' preserved; corrupt config saved to numbered backup '{}'",
                path.with_file_name(BACKUP_FILE_NAME).display(),
                backup.display()
            );
        }
        if let Err(rename_err) = std::fs::rename(path, &backup) {
            log::warn!(
                "could not back up invalid config '{}' to '{}': {}",
                path.display(),
                backup.display(),
                rename_err
            );
        }
    }

    /// Construct a manager holding default settings for the given path.
    fn with_defaults(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            settings: AppSettings::default(),
        }
    }

    /// The resolved `config.toml` path this manager reads from and writes to.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The current cached settings.
    #[must_use]
    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    /// Persist `settings` to `config.toml` atomically and update the cache.
    ///
    /// Serializes the settings inside a [`ConfigFile`] envelope, then writes via
    /// temp-file + `fsync` + `rename` so a crash mid-write cannot leave a torn
    /// file. Creates the config directory if absent and, on Unix, restricts the
    /// file to mode `0600`.
    ///
    /// # Errors
    /// [`ConfigError::Serialize`] on TOML serialization failure, or
    /// [`ConfigError::Io`] on any filesystem failure (dir create, write, sync,
    /// permissions, rename).
    pub fn save(&mut self, settings: &AppSettings) -> Result<(), ConfigError> {
        let body = toml::to_string_pretty(&ConfigFileRef::wrap(settings))?;
        atomic_write(&self.path, body.as_bytes())?;
        log::info!("saved settings to '{}'", self.path.display());
        self.settings = settings.clone();
        Ok(())
    }

    /// Reset settings to [`AppSettings::default()`], persist them atomically,
    /// and return the defaults.
    ///
    /// # Errors
    /// Propagates any [`ConfigError`] from the underlying [`save`].
    ///
    /// [`save`]: ConfigManager::save
    pub fn reset(&mut self) -> Result<AppSettings, ConfigError> {
        let defaults = AppSettings::default();
        self.save(&defaults)?;
        log::info!("reset settings to defaults at '{}'", self.path.display());
        Ok(defaults)
    }

    /// Build the initial [`AppState`] seeded from the cached settings.
    ///
    /// All transient runtime fields (viewport, TTS status, mouse position, …)
    /// take their defaults; only [`AppState::settings`] is replaced. This is the
    /// startup seam (FR-7/AC-3.1): story 001 wraps the result in
    /// `Arc<ArcSwap<AppState>>` for [`StateManager`].
    ///
    /// [`StateManager`]: crate::state_manager::StateManager
    #[must_use]
    pub fn seeded_app_state(&self) -> AppState {
        AppState {
            settings: self.settings.clone(),
            ..AppState::default()
        }
    }
}

/// Load persisted settings and return the seeded [`AppState`] together with the
/// [`ConfigManager`] that owns the resolved path.
///
/// This is the application startup entry point (FR-7): story 001 calls it,
/// wraps the [`AppState`] in `Arc<ArcSwap<AppState>>`, and stores the returned
/// [`ConfigManager`] in `LuminosHandle.config`. Default-on-missing and
/// recover-on-corrupt behaviour is inherited from [`ConfigManager::load`].
///
/// # Errors
/// Returns [`ConfigError::NoConfigDir`] when no config path can be resolved, or
/// [`ConfigError::Io`] on an unexpected filesystem error. Callers should log and
/// fall back to in-memory defaults rather than aborting startup.
pub fn seed_initial_state() -> Result<(AppState, ConfigManager), ConfigError> {
    let path = ConfigManager::config_path()?;
    seed_initial_state_from(&path)
}

/// Path-explicit variant of [`seed_initial_state`], used by [`seed_initial_state`]
/// and by unit tests against temporary directories.
fn seed_initial_state_from(path: &Path) -> Result<(AppState, ConfigManager), ConfigError> {
    let manager = ConfigManager::load_from(path)?;
    let state = manager.seeded_app_state();
    Ok((state, manager))
}

/// Write `bytes` to `path` atomically.
///
/// Steps (FR-4): create the parent directory, write to a uniquely-named temp
/// file in the **same** directory (so `rename` stays on one filesystem and is
/// atomic), flush + `sync_all`, set mode `0600` on Unix, then `rename` over the
/// target. On any error the temp file is removed (best-effort) before returning.
/// After a successful rename, the parent **directory** is fsync'd (best-effort,
/// Unix) so the directory-entry change is itself durable (see [`sync_parent_dir`]).
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ConfigError> {
    let parent = path.parent().ok_or_else(|| ConfigError::Io {
        path: path.display().to_string(),
        source: std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "config path has no parent directory",
        ),
    })?;

    let io_err = |p: &Path, source: std::io::Error| ConfigError::Io {
        path: p.display().to_string(),
        source,
    };

    std::fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;

    let tmp = temp_path_for(path);
    if let Err(err) = write_synced(&tmp, bytes) {
        let _ = std::fs::remove_file(&tmp);
        return Err(io_err(&tmp, err));
    }

    if let Err(err) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(io_err(path, err));
    }

    // Durability of the rename itself: on ext4/xfs the directory-entry change is
    // not on stable storage until the directory is fsync'd. Best-effort — the
    // data write already succeeded; a failed dir-sync should not fail the save.
    sync_parent_dir(parent);
    Ok(())
}

/// fsync the directory `parent` so a preceding `rename` into it is durable.
///
/// Best-effort and Unix-only: opens the directory as a read handle and
/// `sync_all()`s it. Errors are ignored (the standard atomic-write idiom) —
/// the file data is already synced; only crash-window durability of the
/// directory entry is at stake.
#[cfg(unix)]
fn sync_parent_dir(parent: &Path) {
    if let Ok(dir) = std::fs::File::open(parent) {
        let _ = dir.sync_all();
    }
}

/// Non-Unix no-op for [`sync_parent_dir`] (directory fsync semantics differ on
/// Windows/macOS; handled in their platform branches).
#[cfg(not(unix))]
fn sync_parent_dir(_parent: &Path) {}

/// Write `bytes` to `tmp`, flush, `fsync`, and (on Unix) chmod 0600.
///
/// Returns the raw [`std::io::Error`] so the caller can attach the offending
/// path. Kept separate from [`atomic_write`] to keep its `?` chain flat.
fn write_synced(tmp: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(tmp)?;
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()?;
    set_owner_only_permissions(&file)?;
    Ok(())
}

/// Restrict a file to owner read/write (`0600`) on Unix. No-op elsewhere
/// (Windows/macOS ACL handling is deferred to their platform branches).
#[cfg(unix)]
fn set_owner_only_permissions(file: &std::fs::File) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(std::fs::Permissions::from_mode(0o600))
}

/// Non-Unix no-op for [`set_owner_only_permissions`].
#[cfg(not(unix))]
fn set_owner_only_permissions(_file: &std::fs::File) -> std::io::Result<()> {
    Ok(())
}

/// Build a unique sibling temp path for `path` (same directory, so `rename` is
/// atomic). Uniqueness comes from the process id plus a monotonic counter, so
/// concurrent saves within or across processes don't collide.
fn temp_path_for(path: &Path) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let stem = path.file_name().map_or_else(
        || CONFIG_FILE_NAME.to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    let tmp_name = format!(".{stem}.{pid}.{seq}.tmp");
    match path.parent() {
        Some(parent) => parent.join(tmp_name),
        None => PathBuf::from(tmp_name),
    }
}

/// Choose a non-destructive backup path for the corrupt file at `path`.
///
/// Returns `config.toml.bak` when free, otherwise the first unused
/// `config.toml.bak.<n>` (`n` = 1, 2, …). A small upper bound caps the search;
/// if every candidate is taken the bounded fallback is returned (an existing
/// file there would be overwritten only in that pathological case).
fn pick_backup_path(path: &Path) -> PathBuf {
    /// Cap on numbered backups probed before giving up.
    const MAX_NUMBERED_BACKUPS: u32 = 1000;

    let primary = path.with_file_name(BACKUP_FILE_NAME);
    if !primary.exists() {
        return primary;
    }
    (1..=MAX_NUMBERED_BACKUPS)
        .map(|n| path.with_file_name(format!("{BACKUP_FILE_NAME}.{n}")))
        .find(|candidate| !candidate.exists())
        .unwrap_or_else(|| {
            path.with_file_name(format!("{BACKUP_FILE_NAME}.{MAX_NUMBERED_BACKUPS}"))
        })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::path::{Path, PathBuf};

    // ---- shared test fixtures --------------------------------------------

    /// Non-default settings used across persistence tests. Parametrizable so
    /// individual tests can tweak a field without rebuilding the whole struct.
    pub(crate) fn generate_test_settings() -> AppSettings {
        let mut settings = AppSettings::default();
        settings.magnification.zoom_level = 5.0;
        settings.magnification.mode = crate::state::MagnificationMode::Docked;
        settings.color_filter.brightness = 0.25;
        settings.cursor.enlarged_cursor = true;
        settings.start_on_login = true;
        settings
    }

    /// A throwaway temp directory + the `config.toml` path beneath it.
    fn temp_config_path() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(APP_DIR).join(CONFIG_FILE_NAME);
        (dir, path)
    }

    // ---- T002: path resolution -------------------------------------------

    #[test]
    fn config_path_xdg_then_home() {
        // XDG_CONFIG_HOME set → <xdg>/luminos/config.toml
        let xdg = Path::new("/tmp/xdg-test");
        let resolved = resolve_config_path(Some(xdg.to_path_buf()), Some(PathBuf::from("/home/u")))
            .expect("xdg present resolves");
        assert_eq!(resolved, xdg.join("luminos").join("config.toml"));
    }

    #[test]
    fn config_path_home_fallback() {
        // No XDG, HOME present → $HOME/.config/luminos/config.toml
        let home = Path::new("/home/u");
        let resolved =
            resolve_config_path(None, Some(home.to_path_buf())).expect("home present resolves");
        assert_eq!(
            resolved,
            home.join(".config").join("luminos").join("config.toml")
        );
    }

    #[test]
    fn config_path_neither_is_no_config_dir() {
        // Neither XDG nor HOME → NoConfigDir.
        let err = resolve_config_path(None, None).expect_err("no dirs → error");
        assert!(matches!(err, ConfigError::NoConfigDir));
    }

    #[test]
    fn config_path_ignores_relative_xdg() {
        // XDG_CONFIG_HOME must be an absolute path per the XDG spec; a relative
        // value is ignored in favour of the HOME fallback.
        let resolved = resolve_config_path(
            Some(PathBuf::from("relative/dir")),
            Some(PathBuf::from("/home/u")),
        )
        .expect("relative xdg falls back to home");
        assert_eq!(
            resolved,
            Path::new("/home/u")
                .join(".config")
                .join("luminos")
                .join("config.toml")
        );
    }

    #[test]
    fn config_path_public_resolves_under_xdg_or_home() {
        // The public entry point resolves to a luminos/config.toml under
        // whichever base dir the live environment provides (CI always has HOME).
        let path = ConfigManager::config_path().expect("a config dir is resolvable in this env");
        assert!(
            path.ends_with(Path::new("luminos").join("config.toml")),
            "expected .../luminos/config.toml, got {}",
            path.display()
        );
    }

    // ---- T003: load default-on-missing + round-trip ----------------------

    #[test]
    fn config_load_missing_returns_default() {
        // No file at the resolved path → defaults, and the path is cached.
        let (_dir, path) = temp_config_path();
        let manager = ConfigManager::load_from(&path).expect("missing file loads defaults");
        assert_eq!(manager.settings(), &AppSettings::default());
        assert_eq!(manager.path(), path.as_path());
    }

    #[test]
    fn config_save_load_roundtrip() {
        // Write a ConfigFile envelope by hand, then load → equal settings.
        let (_dir, path) = temp_config_path();
        let settings = generate_test_settings();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let body = toml::to_string(&ConfigFile::wrap(settings.clone())).unwrap();
        std::fs::write(&path, body).unwrap();

        let manager = ConfigManager::load_from(&path).expect("valid file loads");
        assert_eq!(manager.settings(), &settings);
    }

    // ---- T007: startup seed (pure-Rust seam for story 001) ---------------

    #[test]
    fn config_seeded_app_state_carries_loaded_settings() {
        // FR-7/AC-3.1 seam: a manager holding non-default settings produces an
        // AppState seeded with exactly those settings (other fields default).
        let (_dir, path) = temp_config_path();
        let mut manager = ConfigManager::load_from(&path).unwrap();
        let settings = generate_test_settings();
        manager.save(&settings).unwrap();

        let state = manager.seeded_app_state();
        assert_eq!(state.settings, settings, "settings seeded from config");
        // Transient runtime fields stay at their defaults.
        assert_eq!(
            state,
            AppState {
                settings,
                ..AppState::default()
            }
        );
    }

    #[test]
    fn config_seed_initial_state_from_path_returns_state_and_manager() {
        // The startup entry point loads from disk and hands back both the seeded
        // state and the manager (which story 001 stores in LuminosHandle.config).
        let (_dir, path) = temp_config_path();
        let settings = generate_test_settings();
        {
            let mut writer = ConfigManager::load_from(&path).unwrap();
            writer.save(&settings).unwrap();
        }

        let (state, manager) =
            seed_initial_state_from(&path).expect("seed from a valid config path");
        assert!(
            (state.settings.magnification.zoom_level - 5.0).abs() < f32::EPSILON,
            "zoom seeded from disk"
        );
        assert_eq!(manager.settings(), &settings);
        assert_eq!(manager.path(), path.as_path());
    }

    #[test]
    fn config_seed_initial_state_missing_file_yields_defaults() {
        // No file on disk → defaults seeded, manager still returned (Some).
        let (_dir, path) = temp_config_path();
        let (state, manager) =
            seed_initial_state_from(&path).expect("seed with no file yields defaults");
        assert_eq!(state.settings, AppSettings::default());
        assert_eq!(manager.settings(), &AppSettings::default());
    }

    // ---- T006: reset -----------------------------------------------------

    #[test]
    fn config_reset_restores_defaults() {
        // Start from non-default on-disk state, reset → defaults persisted,
        // cached, and returned.
        let (_dir, path) = temp_config_path();
        let mut manager = ConfigManager::load_from(&path).unwrap();
        manager.save(&generate_test_settings()).unwrap();
        assert_ne!(manager.settings(), &AppSettings::default());

        let returned = manager.reset().expect("reset succeeds");
        assert_eq!(returned, AppSettings::default(), "reset returns defaults");
        assert_eq!(
            manager.settings(),
            &AppSettings::default(),
            "cache holds defaults"
        );

        let reloaded = ConfigManager::load_from(&path).unwrap();
        assert_eq!(
            reloaded.settings(),
            &AppSettings::default(),
            "defaults persisted to disk"
        );
    }

    // ---- T005: corrupt recovery + .bak backup ----------------------------

    /// Write arbitrary bytes as the config file at a fresh temp path.
    fn write_raw_config(contents: &str) -> (tempfile::TempDir, PathBuf) {
        let (dir, path) = temp_config_path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, contents).unwrap();
        (dir, path)
    }

    #[test]
    fn config_load_corrupt_recovers_to_defaults_with_bak() {
        // Total garbage → defaults, original preserved as config.toml.bak,
        // no panic.
        let (_dir, path) = write_raw_config("this is not valid toml @@@ {{{");
        let manager = ConfigManager::load_from(&path).expect("corrupt file recovers");
        assert_eq!(manager.settings(), &AppSettings::default());

        let bak = path.with_file_name(BACKUP_FILE_NAME);
        assert!(bak.exists(), "corrupt file should be backed up to .bak");
        let backed_up = std::fs::read_to_string(&bak).unwrap();
        assert!(
            backed_up.contains("not valid toml"),
            "backup must preserve the original bad contents"
        );
    }

    #[test]
    fn config_load_partial_invalid_recovers() {
        // Well-formed TOML but wrong types (zoom_level as a string) → recover.
        let bad = r#"
schema_version = 1
[settings.magnification]
zoom_level = "not a number"
"#;
        let (_dir, path) = write_raw_config(bad);
        let manager = ConfigManager::load_from(&path).expect("type-mismatched file recovers");
        assert_eq!(manager.settings(), &AppSettings::default());
        assert!(
            path.with_file_name(BACKUP_FILE_NAME).exists(),
            "partial-invalid file is also backed up"
        );
    }

    #[test]
    fn config_load_recovery_does_not_overwrite_target() {
        // Recovery invariants (F-001 — assert the real contract, not a vacuous
        // empty-string read):
        //   (a) the corrupt bytes are NOT at the live config.toml path,
        //   (b) the corrupt bytes ARE preserved in the backup,
        //   (c) an unrelated pre-existing sibling file is never touched.
        let (_dir, path) = write_raw_config("garbage-corrupt-bytes");

        // Drop an unrelated neighbour in the same directory.
        let bystander = path.with_file_name("unrelated.txt");
        std::fs::write(&bystander, "do-not-touch").unwrap();

        let manager = ConfigManager::load_from(&path).expect("corrupt file recovers");
        assert_eq!(manager.settings(), &AppSettings::default());

        // (a) the live path no longer holds the corrupt bytes. It was renamed
        //     away, so it should not exist (and certainly not contain garbage).
        assert!(
            !path.exists(),
            "corrupt file should be moved off the live path, not left in place"
        );

        // (b) the backup holds exactly the original corrupt bytes.
        let bak = path.with_file_name(BACKUP_FILE_NAME);
        assert_eq!(
            std::fs::read_to_string(&bak).unwrap(),
            "garbage-corrupt-bytes",
            "backup must preserve the original corrupt bytes verbatim"
        );

        // (c) the unrelated neighbour is untouched.
        assert_eq!(
            std::fs::read_to_string(&bystander).unwrap(),
            "do-not-touch",
            "recovery must not touch unrelated files in the directory"
        );
    }

    #[test]
    fn config_load_corrupt_preserves_existing_bak() {
        // I-2: a pre-existing config.toml.bak must NOT be clobbered. The new
        // corrupt file goes to the first free numbered fallback instead.
        let (_dir, path) = write_raw_config("new-corrupt-content");

        // Pre-create config.toml.bak with sentinel content (a prior recovery).
        let bak = path.with_file_name(BACKUP_FILE_NAME);
        std::fs::write(&bak, "OLD-SENTINEL-BACKUP").unwrap();

        let manager = ConfigManager::load_from(&path).expect("corrupt file recovers, no panic");
        // (c) load returns defaults without panicking.
        assert_eq!(manager.settings(), &AppSettings::default());

        // (a) the original .bak sentinel is still intact.
        assert_eq!(
            std::fs::read_to_string(&bak).unwrap(),
            "OLD-SENTINEL-BACKUP",
            "existing config.toml.bak must not be overwritten"
        );

        // (b) a numbered backup now holds the new corrupt bytes.
        let numbered = path.with_file_name(format!("{BACKUP_FILE_NAME}.1"));
        assert!(
            numbered.exists(),
            "numbered fallback backup should be created"
        );
        assert_eq!(
            std::fs::read_to_string(&numbered).unwrap(),
            "new-corrupt-content",
            "numbered backup must hold the new corrupt bytes"
        );
    }

    // ---- T004: atomic save + dir create + 0600 ---------------------------

    #[test]
    fn config_save_is_atomic_no_temp_left() {
        // After save, the target exists, reloads identically, and no stray
        // temp file remains alongside it.
        let (_dir, path) = temp_config_path();
        let mut manager = ConfigManager::load_from(&path).unwrap();
        let settings = generate_test_settings();
        manager.save(&settings).expect("save succeeds");

        assert!(path.exists(), "target config.toml must exist after save");
        let reloaded = ConfigManager::load_from(&path).unwrap();
        assert_eq!(reloaded.settings(), &settings, "saved settings reload");

        let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "no temp file should remain, found {leftovers:?}"
        );
    }

    #[test]
    fn config_save_updates_cache() {
        // save() must refresh the in-memory cache so settings() reflects it.
        let (_dir, path) = temp_config_path();
        let mut manager = ConfigManager::load_from(&path).unwrap();
        let settings = generate_test_settings();
        manager.save(&settings).unwrap();
        assert_eq!(manager.settings(), &settings);
    }

    #[test]
    fn config_save_creates_dir() {
        // Parent directory absent → save creates it.
        let dir = tempfile::tempdir().unwrap();
        let path = dir
            .path()
            .join("nested")
            .join("deeper")
            .join(APP_DIR)
            .join(CONFIG_FILE_NAME);
        assert!(!path.parent().unwrap().exists());
        let mut manager = ConfigManager::load_from(&path).unwrap();
        manager
            .save(&AppSettings::default())
            .expect("save creates dir");
        assert!(path.exists(), "config.toml created with its directory tree");
    }

    #[test]
    #[cfg(unix)]
    fn config_save_unix_permissions_0600() {
        use std::os::unix::fs::PermissionsExt;
        let (_dir, path) = temp_config_path();
        let mut manager = ConfigManager::load_from(&path).unwrap();
        manager.save(&AppSettings::default()).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(
            mode & 0o777,
            0o600,
            "config.toml should be user-only (0600), got {:o}",
            mode & 0o777
        );
    }

    #[test]
    fn config_save_parent_is_file_returns_io_error_no_temp_leak() {
        // M-5: if the target's parent path is a regular file, create_dir_all
        // fails (NotADirectory). save() must surface ConfigError::Io and leak
        // no .tmp file.
        let dir = tempfile::tempdir().unwrap();
        // `blocker` is a regular file; treat it as if it were a directory.
        let blocker = dir.path().join("blocker");
        std::fs::write(&blocker, "i am a file, not a dir").unwrap();
        let path = blocker.join(CONFIG_FILE_NAME); // parent == blocker (a file)

        // Build a manager pointed at the bad path directly (going through
        // load_from would itself fail to read, which isn't what we're testing).
        let mut manager = ConfigManager::with_defaults(&path);
        let err = manager
            .save(&AppSettings::default())
            .expect_err("save under a file-parent must fail");
        assert!(
            matches!(err, ConfigError::Io { .. }),
            "expected ConfigError::Io, got {err:?}"
        );

        // No temp file leaked into the temp dir root.
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "no temp file should leak on failure, found {leftovers:?}"
        );
    }

    #[test]
    fn app_settings_default_holds() {
        // AC-1.1 anchor: the default settings the persistence layer falls back
        // to must match the documented Phase 0 defaults. (Asserting existing
        // behavior — schema.rs owns the Default impl; do NOT re-derive it.)
        let s = AppSettings::default();
        assert!(
            (s.magnification.zoom_level - 2.0).abs() < f32::EPSILON,
            "expected default zoom 2.0, got {}",
            s.magnification.zoom_level
        );
        assert_eq!(
            s.magnification.mode,
            crate::state::MagnificationMode::FullScreen
        );
        assert_eq!(
            s.magnification.tracking_mode,
            crate::state::TrackingMode::Cursor
        );
    }

    #[test]
    fn config_file_default_schema_version_is_current() {
        assert_eq!(default_schema_version(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn config_file_wrap_carries_current_version() {
        let wrapped = ConfigFile::wrap(AppSettings::default());
        assert_eq!(wrapped.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(wrapped.settings, AppSettings::default());
    }

    #[test]
    fn config_file_missing_version_defaults_on_parse() {
        // A file that omits `schema_version` (e.g. hand-written) must still
        // parse, taking the default version via `#[serde(default)]`.
        let body = toml::to_string(&AppSettings::default()).unwrap();
        let doc = format!("[settings]\n{}", indent_under_settings(&body));
        let parsed: ConfigFile = toml::from_str(&doc).unwrap();
        assert_eq!(parsed.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(parsed.settings, AppSettings::default());
    }

    #[test]
    fn config_file_toml_roundtrip() {
        let original = ConfigFile::wrap(AppSettings::default());
        let toml_str = toml::to_string(&original).unwrap();
        let back: ConfigFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(original, back);
    }

    /// Re-emit a flat `AppSettings` TOML body nested one level under a
    /// `[settings]` table. `toml::to_string` already produces section headers
    /// like `[magnification]`; rewrite them to `[settings.magnification]`.
    fn indent_under_settings(body: &str) -> String {
        body.lines()
            .map(|line| {
                if let Some(rest) = line.strip_prefix('[') {
                    format!("[settings.{rest}")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
