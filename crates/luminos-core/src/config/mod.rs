//! Configuration schema, persistence, and settings types.
//!
//! The [`schema`] module defines [`schema::AppSettings`] and all nested
//! configuration sub-structs. The [`manager`] module provides
//! [`ConfigManager`], the on-disk persistence layer that loads and saves
//! [`schema::AppSettings`] to `config.toml`, and [`error::ConfigError`],
//! the error type those operations return.

pub mod error;
pub mod manager;
pub mod schema;

pub use error::ConfigError;
pub use manager::{ConfigManager, seed_initial_state};
pub use schema::AppSettings;
