//! GPU configuration types.
//!
//! Defines presentation mode, GPU preference, and interpolation mode
//! enums used by the rendering pipeline and settings schema.

use serde::{Deserialize, Serialize};

/// `VSync` / frame presentation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum PresentMode {
    /// VSync-locked (Fifo), smooth, no tearing.
    Quality,
    /// Mailbox, lowest input latency with GPU overhead.
    LowLatency,
    /// Immediate, uncapped FPS for diagnostics.
    Performance,
}

/// GPU device preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum GpuPreference {
    /// Integrated GPU (lower power, default).
    LowPower,
    /// Discrete GPU (higher performance).
    HighPerformance,
}

/// Scaling interpolation algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum InterpolationMode {
    /// Bilinear filtering (Phase 0 default).
    Bilinear,
    /// Bicubic filtering (Phase 1+, higher quality).
    Bicubic,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn present_mode_serde_roundtrip() {
        let variants = [
            (PresentMode::Quality, "\"Quality\""),
            (PresentMode::LowLatency, "\"LowLatency\""),
            (PresentMode::Performance, "\"Performance\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: PresentMode = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }

    #[test]
    fn gpu_preference_serde_roundtrip() {
        let variants = [
            (GpuPreference::LowPower, "\"LowPower\""),
            (GpuPreference::HighPerformance, "\"HighPerformance\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: GpuPreference = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }

    #[test]
    fn interpolation_mode_serde_roundtrip() {
        let variants = [
            (InterpolationMode::Bilinear, "\"Bilinear\""),
            (InterpolationMode::Bicubic, "\"Bicubic\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: InterpolationMode = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }
}
