//! Screen capture data types.
//!
//! Contains [`CaptureFrame`] and [`PixelFormat`] used by the screen
//! capture trait and the GPU rendering pipeline.

use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Pixel format of captured frame data.
///
/// All pixel data is assumed to be in **sRGB color space** (nonlinear,
/// gamma-encoded). The GPU rendering pipeline performs gamma-correct
/// resampling by converting to linear space before interpolation and
/// back to sRGB for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PixelFormat {
    /// Blue, Green, Red, Alpha (8 bits each). Native format on X11 and Windows.
    Bgra8,
    /// Red, Green, Blue, Alpha (8 bits each). Native format on macOS (`ScreenCaptureKit`).
    Rgba8,
}

/// A captured frame of screen content.
///
/// **Privacy:** This struct contains raw screen pixels that may include
/// sensitive content (passwords, banking, medical records). The custom
/// `Debug` implementation intentionally omits the `data` field to prevent
/// accidental leakage in log output. See RISK-017.
///
/// **Note:** `CaptureFrame` does not implement `Serialize`/`Deserialize`
/// because it contains `Arc<[u8]>` pixel data intended for GPU upload,
/// not for config persistence or IPC serialization.
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

/// Custom Debug impl for `CaptureFrame` that omits pixel data (RISK-017).
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

#[cfg(any(test, feature = "test_utils"))]
pub mod test_utils {
    //! Test utilities for capture types.

    use super::{CaptureFrame, PixelFormat};

    /// Generates a test [`CaptureFrame`] with solid-color BGRA pixel data.
    ///
    /// # Parameters
    /// - `width`: Frame width in pixels.
    /// - `height`: Frame height in pixels.
    /// - `color`: BGRA color value `[b, g, r, a]` for every pixel.
    #[must_use]
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn pixel_format_derives() {
        let bgra = PixelFormat::Bgra8;
        let rgba = PixelFormat::Rgba8;
        assert!(format!("{bgra:?}").contains("Bgra8"));
        assert!(format!("{rgba:?}").contains("Rgba8"));
        let copied = bgra;
        assert_eq!(bgra, copied);
        let mut set = HashSet::new();
        set.insert(bgra);
        set.insert(rgba);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn pixel_format_serde_roundtrip() {
        let variants = [
            (PixelFormat::Bgra8, "\"Bgra8\""),
            (PixelFormat::Rgba8, "\"Rgba8\""),
        ];
        for (variant, expected_json) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json, "serialization of {variant:?}");
            let back: PixelFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant, "deserialization of {variant:?}");
        }
    }

    #[test]
    fn capture_frame_fields() {
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
    fn capture_frame_debug_omits_data() {
        let frame = test_utils::generate_test_capture_frame(4, 2, [255, 0, 0, 255]);
        let debug = format!("{frame:?}");
        assert!(
            debug.contains("bytes"),
            "debug output must contain 'bytes' placeholder"
        );
        assert!(
            !debug.contains("[255, 0, 0, 255"),
            "debug output must NOT contain raw pixel data"
        );
    }

    #[test]
    fn generate_test_capture_frame_correct_output() {
        let frame = test_utils::generate_test_capture_frame(64, 48, [0, 0, 255, 255]);
        assert_eq!(frame.width, 64);
        assert_eq!(frame.height, 48);
        assert_eq!(frame.stride, 256);
        assert_eq!(frame.format, PixelFormat::Bgra8);
        assert_eq!(frame.data.len(), 12288);
    }
}
