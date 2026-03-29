//! Display and geometry types.
//!
//! These types form the coordinate vocabulary shared across platform
//! backends and the core engine.

use serde::{Deserialize, Serialize};

/// A rectangle in screen coordinates (pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScreenRect {
    /// X position of the top-left corner.
    pub x: i32,
    /// Y position of the top-left corner.
    pub y: i32,
    /// Width of the rectangle in pixels.
    pub width: u32,
    /// Height of the rectangle in pixels.
    pub height: u32,
}

/// A point in screen coordinates (pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScreenPoint {
    /// X coordinate.
    pub x: i32,
    /// Y coordinate.
    pub y: i32,
}

/// Information about a connected display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayInfo {
    /// Unique identifier for this display (platform-specific format).
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Display bounds in virtual screen coordinates.
    pub bounds: ScreenRect,
    /// Scale factor (e.g., 2.0 for HiDPI/Retina).
    pub scale_factor: f64,
    /// Whether this is the primary display.
    pub is_primary: bool,
}

#[cfg(any(test, feature = "test_utils"))]
pub mod test_utils {
    //! Test utilities for display and geometry types.

    use super::{DisplayInfo, ScreenRect};

    /// Generates a test [`DisplayInfo`] with configurable parameters.
    #[must_use]
    pub fn generate_test_display_info(
        id: &str,
        width: u32,
        height: u32,
        is_primary: bool,
    ) -> DisplayInfo {
        DisplayInfo {
            id: id.to_string(),
            name: format!("Test Display {id}"),
            bounds: ScreenRect {
                x: 0,
                y: 0,
                width,
                height,
            },
            scale_factor: 1.0,
            is_primary,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn screen_rect_fields_and_derives() {
        let rect = ScreenRect {
            x: 10,
            y: 20,
            width: 1920,
            height: 1080,
        };
        let debug = format!("{rect:?}");
        assert!(debug.contains("ScreenRect"));
        let copied = rect;
        assert_eq!(rect, copied);
        let mut set = HashSet::new();
        set.insert(rect);
        assert!(set.contains(&rect));
    }

    #[test]
    fn screen_rect_serde_roundtrip() {
        let rect = ScreenRect {
            x: 10,
            y: -20,
            width: 1920,
            height: 1080,
        };
        let json = serde_json::to_string(&rect).unwrap();
        let back: ScreenRect = serde_json::from_str(&json).unwrap();
        assert_eq!(rect, back);
    }

    #[test]
    fn screen_point_fields_and_derives() {
        let point = ScreenPoint { x: 100, y: 200 };
        let debug = format!("{point:?}");
        assert!(debug.contains("ScreenPoint"));
        let copied = point;
        assert_eq!(point, copied);
        let mut set = HashSet::new();
        set.insert(point);
        assert!(set.contains(&point));
    }

    #[test]
    fn screen_point_serde_roundtrip() {
        let point = ScreenPoint { x: -50, y: 300 };
        let json = serde_json::to_string(&point).unwrap();
        let back: ScreenPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(point, back);
    }

    #[test]
    fn display_info_fields_and_derives() {
        let info = DisplayInfo {
            id: "DP-1".to_string(),
            name: "Main Display".to_string(),
            bounds: ScreenRect {
                x: 0,
                y: 0,
                width: 2560,
                height: 1440,
            },
            scale_factor: 2.0,
            is_primary: true,
        };
        let debug = format!("{info:?}");
        assert!(debug.contains("DisplayInfo"));
        let cloned = info.clone();
        assert_eq!(info, cloned);
    }

    #[test]
    fn display_info_serde_roundtrip() {
        let info = DisplayInfo {
            id: "HDMI-1".to_string(),
            name: "External".to_string(),
            bounds: ScreenRect {
                x: 1920,
                y: 0,
                width: 2560,
                height: 1440,
            },
            scale_factor: 1.5,
            is_primary: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        let back: DisplayInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, back);
    }

    #[test]
    fn generate_test_display_info_correct_output() {
        let info = test_utils::generate_test_display_info("test-0", 1920, 1080, true);
        assert_eq!(info.id, "test-0");
        assert_eq!(info.bounds.width, 1920);
        assert_eq!(info.bounds.height, 1080);
        assert!(info.is_primary);
        assert!((info.scale_factor - 1.0).abs() < f64::EPSILON);
    }
}
