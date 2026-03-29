//! Shared types for the Luminos accessibility suite.
//!
//! This crate defines data types used across multiple workspace crates.
//! It has **zero** workspace crate dependencies (only `serde` from
//! external deps) to avoid circular dependency issues.
//!
//! # Modules
//!
//! - [`capture`] -- Screen capture data types (`CaptureFrame`, `PixelFormat`).
//! - [`display`] -- Display and geometry types (`ScreenRect`, `ScreenPoint`, `DisplayInfo`).
//! - [`overlay`] -- Overlay window types (`DockEdge`, `LensShape`, `OverlayMode`).
//! - [`state`] -- Runtime state enums (`MagnificationMode`, `TrackingMode`, etc.).
//! - [`gpu`] -- GPU configuration types (`PresentMode`, `GpuPreference`, `InterpolationMode`).

pub mod capture;
pub mod display;
pub mod gpu;
pub mod overlay;
pub mod state;

// Re-export all public types at crate root for convenience.
pub use capture::{CaptureFrame, PixelFormat};
pub use display::{DisplayInfo, ScreenPoint, ScreenRect};
pub use gpu::{GpuPreference, InterpolationMode, PresentMode};
pub use overlay::{DockEdge, LensShape, OverlayMode};
pub use state::{ColorFilterType, MagnificationMode, TrackingMode, TtsStatus};
