//! Common types shared across platform abstraction traits.
//!
//! These types form the data exchange vocabulary between the core engine
//! and all platform backends. They are defined here to avoid circular
//! dependencies between trait modules.

use std::fmt;
use std::sync::Arc;

/// A rectangle in screen coordinates (pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenPoint {
    /// X coordinate.
    pub x: i32,
    /// Y coordinate.
    pub y: i32,
}

/// Information about a connected display.
#[derive(Debug, Clone, PartialEq)]
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

/// Pixel format of captured frame data.
///
/// All pixel data is assumed to be in **sRGB color space** (nonlinear,
/// gamma-encoded). The GPU rendering pipeline performs gamma-correct
/// resampling by converting to linear space before interpolation and
/// back to sRGB for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// Blue, Green, Red, Alpha (8 bits each). Native format on X11 and Windows.
    Bgra8,
    /// Red, Green, Blue, Alpha (8 bits each). Native format on macOS (ScreenCaptureKit).
    Rgba8,
}

/// A captured frame of screen content.
///
/// **Privacy:** This struct contains raw screen pixels that may include
/// sensitive content (passwords, banking, medical records). The custom
/// `Debug` implementation intentionally omits the `data` field to prevent
/// accidental leakage in log output. See RISK-017.
#[derive(Clone)]
pub struct CaptureFrame {
    /// Raw pixel data in row-major order, top-left origin.
    pub data: Arc<[u8]>,
    /// Width of the captured frame in pixels.
    pub width: u32,
    /// Height of the captured frame in pixels.
    pub height: u32,
    /// Bytes per row (may include padding).
    pub stride: u32,
    /// Pixel format of the data.
    pub format: PixelFormat,
}

/// Custom Debug impl for CaptureFrame that omits pixel data (RISK-017).
///
/// Prints metadata only: width, height, stride, format, and the byte
/// length of the data buffer. Never prints raw pixel content.
impl fmt::Debug for CaptureFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CaptureFrame")
            .field("data", &format_args!("[<{} bytes>]", self.data.len()))
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .field("format", &self.format)
            .finish()
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    /// Generates a test `CaptureFrame` with solid-color BGRA pixel data.
    ///
    /// # Parameters
    /// - `width`: Frame width in pixels.
    /// - `height`: Frame height in pixels.
    /// - `color`: BGRA color value `[b, g, r, a]` for every pixel.
    pub fn generate_test_capture_frame(width: u32, height: u32, color: [u8; 4]) -> CaptureFrame {
        let stride = width * 4;
        let data: Vec<u8> = color
            .iter()
            .cycle()
            .take((stride * height) as usize)
            .copied()
            .collect();
        CaptureFrame {
            data: data.into(),
            width,
            height,
            stride,
            format: PixelFormat::Bgra8,
        }
    }

    /// Generates a test `DisplayInfo` with configurable parameters.
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
mod tests {
    use super::*;
    use std::collections::HashSet;

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
        let cloned = rect.clone();
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
