//! Overlay window configuration types.
//!
//! Defines [`DockEdge`], [`LensShape`], and [`OverlayMode`] used by the
//! window manager trait and configuration schema.

use serde::{Deserialize, Serialize};

/// The edge of the screen where a docked overlay attaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum DockEdge {
    /// Top edge of the screen.
    Top,
    /// Bottom edge of the screen.
    Bottom,
    /// Left edge of the screen.
    Left,
    /// Right edge of the screen.
    Right,
}

/// The shape of a lens-mode overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum LensShape {
    /// Rectangular lens boundary.
    Rectangle,
    /// Elliptical lens boundary.
    Ellipse,
}

/// The magnification overlay display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayMode {
    /// The overlay covers the entire display.
    FullScreen,
    /// A movable lens that follows the cursor.
    Lens {
        /// Width of the lens in pixels.
        width: u32,
        /// Height of the lens in pixels.
        height: u32,
        /// Shape of the lens boundary.
        shape: LensShape,
    },
    /// The overlay is docked to one edge of the screen.
    Docked {
        /// Which screen edge to dock against.
        edge: DockEdge,
        /// Size of the docked region in pixels (perpendicular to the edge).
        size_px: u32,
    },
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn dock_edge_serde_roundtrip() {
        let variants = [
            (DockEdge::Top, "\"Top\""),
            (DockEdge::Bottom, "\"Bottom\""),
            (DockEdge::Left, "\"Left\""),
            (DockEdge::Right, "\"Right\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: DockEdge = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }

    #[test]
    fn lens_shape_serde_roundtrip() {
        let variants = [
            (LensShape::Rectangle, "\"Rectangle\""),
            (LensShape::Ellipse, "\"Ellipse\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: LensShape = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }

    #[test]
    fn overlay_mode_serde_roundtrip() {
        let variants_json = [
            (OverlayMode::FullScreen, "\"FullScreen\""),
            (
                OverlayMode::Lens {
                    width: 400,
                    height: 300,
                    shape: LensShape::Ellipse,
                },
                "{\"Lens\":{\"width\":400,\"height\":300,\"shape\":\"Ellipse\"}}",
            ),
            (
                OverlayMode::Docked {
                    edge: DockEdge::Bottom,
                    size_px: 540,
                },
                "{\"Docked\":{\"edge\":\"Bottom\",\"size_px\":540}}",
            ),
        ];
        for (variant, expected_json) in &variants_json {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: OverlayMode = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }
}
