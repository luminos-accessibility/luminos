//! Configuration persistence errors.
//!
//! Defines [`ConfigError`], the error type returned by [`ConfigManager`]
//! load/save/reset operations.
//!
//! [`ConfigManager`]: crate::config::ConfigManager

/// Errors raised while loading, saving, or resetting persisted configuration.
///
/// Deserialize/parse failures are deliberately **not** represented here:
/// [`ConfigManager::load`] recovers a corrupt file to defaults rather than
/// surfacing a hard error (see story FR-5).
///
/// [`ConfigManager::load`]: crate::config::ConfigManager::load
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A filesystem operation (read, write, rename, sync, permissions) failed.
    #[error("config I/O error at '{path}': {source}")]
    Io {
        /// The path the operation targeted.
        path: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Serializing [`AppSettings`] to TOML failed.
    ///
    /// [`AppSettings`]: crate::config::AppSettings
    #[error("config serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),

    /// Neither `$XDG_CONFIG_HOME` nor `$HOME` could be resolved, so the
    /// platform config directory is unknown.
    #[error("could not resolve config directory (no XDG_CONFIG_HOME or HOME)")]
    NoConfigDir,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn config_error_display_io() {
        let err = ConfigError::Io {
            path: "/tmp/luminos/config.toml".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        };
        let msg = format!("{err}");
        assert!(
            msg.contains("/tmp/luminos/config.toml"),
            "expected path in '{msg}'"
        );
        assert!(msg.contains("denied"), "expected source detail in '{msg}'");
    }

    #[test]
    fn config_error_display_serialize() {
        // Force a real toml serialize error: a bare integer is not a valid TOML
        // document (values must live inside a table), so serializing one fails.
        let toml_err = toml::to_string(&42i32).expect_err("expected serialize failure");
        let err = ConfigError::Serialize(toml_err);
        let msg = format!("{err}");
        assert!(msg.contains("serialize"), "expected 'serialize' in '{msg}'");
    }

    #[test]
    fn config_error_display_no_config_dir() {
        let err = ConfigError::NoConfigDir;
        let msg = format!("{err}");
        assert!(
            msg.contains("config directory"),
            "expected 'config directory' in '{msg}'"
        );
    }

    #[test]
    fn config_error_from_toml_ser_error() {
        // The `#[from]` impl must let `?` convert a toml serialize error.
        fn convert() -> Result<(), ConfigError> {
            let _ = toml::to_string(&42i32)?;
            Ok(())
        }
        assert!(matches!(convert(), Err(ConfigError::Serialize(_))));
    }
}
