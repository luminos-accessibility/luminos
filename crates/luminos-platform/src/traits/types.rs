//! Common types shared across platform abstraction traits.
//!
//! These types form the data exchange vocabulary between the core engine
//! and all platform backends. They are re-exported from [`luminos_types`]
//! to maintain backwards compatibility with existing import paths.

pub use luminos_types::{CaptureFrame, DisplayInfo, PixelFormat, ScreenPoint, ScreenRect};

#[cfg(any(test, feature = "test_utils"))]
pub mod test_utils {
    //! Test utilities for common types.
    //!
    //! Re-exported from [`luminos_types`] capture and display test utilities.

    pub use luminos_types::capture::test_utils::generate_test_capture_frame;
    pub use luminos_types::display::test_utils::generate_test_display_info;
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;

    #[test]
    fn types_screen_rect_fields_and_derives() {
        let rect = ScreenRect {
            x: 10,
            y: 20,
            width: 1920,
            height: 1080,
        };
        // Debug
        let debug = format!("{rect:?}");
        assert!(debug.contains("ScreenRect"));
        // Clone + Copy
        let copied = rect;
        assert_eq!(rect, copied);
        let cloned = rect;
        assert_eq!(rect, cloned);
        // Eq + Hash
        let mut set = HashSet::new();
        set.insert(rect);
        assert!(set.contains(&rect));
    }

    #[test]
    fn types_screen_point_fields_and_derives() {
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
    fn types_display_info_fields_and_derives() {
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
    fn types_pixel_format_derives() {
        let bgra = PixelFormat::Bgra8;
        let rgba = PixelFormat::Rgba8;
        // Debug
        assert!(format!("{bgra:?}").contains("Bgra8"));
        assert!(format!("{rgba:?}").contains("Rgba8"));
        // Clone + Copy
        let copied = bgra;
        assert_eq!(bgra, copied);
        // Eq + Hash
        let mut set = HashSet::new();
        set.insert(bgra);
        set.insert(rgba);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn types_capture_frame_fields() {
        let data: Arc<[u8]> = vec![0u8; 100].into();
        let frame = CaptureFrame {
            data: data.clone(),
            width: 10,
            height: 10,
            stride: 40,
            format: PixelFormat::Bgra8,
        };
        assert_eq!(frame.width, 10);
        assert_eq!(frame.height, 10);
        assert_eq!(frame.stride, 40);
        assert_eq!(frame.format, PixelFormat::Bgra8);
        assert_eq!(frame.data.len(), 100);
    }

    #[test]
    fn types_capture_frame_debug_omits_data() {
        let frame = test_utils::generate_test_capture_frame(4, 2, [255, 0, 0, 255]);
        let debug = format!("{frame:?}");
        // Must contain the "bytes" placeholder
        assert!(
            debug.contains("bytes"),
            "debug output must contain 'bytes' placeholder"
        );
        // Must NOT contain raw pixel data patterns
        assert!(
            !debug.contains("[255, 0, 0, 255"),
            "debug output must NOT contain raw pixel data"
        );
    }

    #[test]
    fn types_generate_test_capture_frame_correct_output() {
        let frame = test_utils::generate_test_capture_frame(64, 48, [0, 0, 255, 255]);
        assert_eq!(frame.width, 64);
        assert_eq!(frame.height, 48);
        assert_eq!(frame.stride, 256); // 64 * 4
        assert_eq!(frame.format, PixelFormat::Bgra8);
        assert_eq!(frame.data.len(), 12288); // 256 * 48
    }

    #[test]
    fn types_generate_test_display_info_correct_output() {
        let info = test_utils::generate_test_display_info("test-0", 1920, 1080, true);
        assert_eq!(info.id, "test-0");
        assert_eq!(info.bounds.width, 1920);
        assert_eq!(info.bounds.height, 1080);
        assert!(info.is_primary);
        assert!((info.scale_factor - 1.0).abs() < f64::EPSILON);
    }
}
