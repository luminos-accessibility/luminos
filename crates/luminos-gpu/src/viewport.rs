//! Viewport calculation for the magnification pipeline.
//!
//! Pure arithmetic functions that determine which region of the screen
//! to capture based on a tracking target (e.g., cursor position) and
//! zoom level. These functions contain no GPU, I/O, or allocation
//! operations, enabling sub-microsecond execution.

use luminos_types::{ScreenPoint, ScreenRect};

/// Computes the source region of the screen to capture for magnification.
///
/// The source region is the unmagnified rectangle of screen content that,
/// when scaled by `zoom_level`, fills the overlay viewport.
///
/// The region is centered on `tracking_target` and clamped to `screen_bounds`
/// to prevent capturing outside the display.
///
/// # Arguments
///
/// * `tracking_target` -- The point to center the magnified view on (cursor position).
/// * `zoom_level` -- The magnification factor (1.5 to 20.0).
/// * `viewport_size` -- The overlay viewport dimensions (width, height) in pixels.
/// * `screen_bounds` -- The display bounds in physical pixel coordinates.
///
/// # Returns
///
/// A [`ScreenRect`] representing the region to capture. Returns a zero-size
/// region at the tracking target if `zoom_level` is zero or negative.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub fn compute_source_region(
    tracking_target: ScreenPoint,
    zoom_level: f32,
    viewport_size: (u32, u32),
    screen_bounds: ScreenRect,
) -> ScreenRect {
    if zoom_level <= 0.0 {
        return ScreenRect {
            x: tracking_target.x,
            y: tracking_target.y,
            width: 0,
            height: 0,
        };
    }

    let source_width = (viewport_size.0 as f32 / zoom_level).ceil() as i32;
    let source_height = (viewport_size.1 as f32 / zoom_level).ceil() as i32;

    // Center the source region on the tracking target.
    let mut x = tracking_target.x - source_width / 2;
    let mut y = tracking_target.y - source_height / 2;

    // Clamp to screen bounds (prevent capturing outside the display).
    let max_x = screen_bounds.x + screen_bounds.width as i32 - source_width;
    let max_y = screen_bounds.y + screen_bounds.height as i32 - source_height;

    x = x.clamp(screen_bounds.x, max_x.max(screen_bounds.x));
    y = y.clamp(screen_bounds.y, max_y.max(screen_bounds.y));

    ScreenRect {
        x,
        y,
        width: source_width.max(0) as u32,
        height: source_height.max(0) as u32,
    }
}

/// Smoothly interpolates the viewport center toward the tracking target.
///
/// Uses linear interpolation with a configurable smoothing factor to
/// prevent disorienting viewport jumps when the tracking target moves.
///
/// # Arguments
///
/// * `current` -- The current viewport center position.
/// * `target` -- The desired viewport center (tracking target).
/// * `smoothing_factor` -- Interpolation speed (0.0 = no movement, 1.0 = instant).
///   Typical range: 0.1-0.3 for comfortable panning.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn smooth_viewport_position(
    current: ScreenPoint,
    target: ScreenPoint,
    smoothing_factor: f32,
) -> ScreenPoint {
    let factor = smoothing_factor.clamp(0.0, 1.0);
    ScreenPoint {
        x: current.x + ((target.x - current.x) as f32 * factor) as i32,
        y: current.y + ((target.y - current.y) as f32 * factor) as i32,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Standard 1080p screen bounds for tests.
    fn screen_1080p() -> ScreenRect {
        ScreenRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }
    }

    // --- compute_source_region: zoom level dimension tests ---

    #[test]
    fn viewport_source_region_2x_zoom() {
        let result = compute_source_region(
            ScreenPoint { x: 960, y: 540 },
            2.0,
            (1920, 1080),
            screen_1080p(),
        );
        assert_eq!(result.width, 960);
        assert_eq!(result.height, 540);
    }

    #[test]
    fn viewport_source_region_5x_zoom() {
        let result = compute_source_region(
            ScreenPoint { x: 960, y: 540 },
            5.0,
            (1920, 1080),
            screen_1080p(),
        );
        assert_eq!(result.width, 384);
        assert_eq!(result.height, 216);
    }

    #[test]
    fn viewport_source_region_10x_zoom() {
        let result = compute_source_region(
            ScreenPoint { x: 960, y: 540 },
            10.0,
            (1920, 1080),
            screen_1080p(),
        );
        assert_eq!(result.width, 192);
        assert_eq!(result.height, 108);
    }

    #[test]
    fn viewport_source_region_20x_zoom() {
        let result = compute_source_region(
            ScreenPoint { x: 960, y: 540 },
            20.0,
            (1920, 1080),
            screen_1080p(),
        );
        assert_eq!(result.width, 96);
        assert_eq!(result.height, 54);
    }

    #[test]
    fn viewport_source_region_1_5x_zoom() {
        let result = compute_source_region(
            ScreenPoint { x: 960, y: 540 },
            1.5,
            (1920, 1080),
            screen_1080p(),
        );
        assert_eq!(result.width, 1280);
        assert_eq!(result.height, 720);
    }

    #[test]
    fn viewport_source_region_centered() {
        let result = compute_source_region(
            ScreenPoint { x: 960, y: 540 },
            2.0,
            (1920, 1080),
            screen_1080p(),
        );
        // Source is 960x540, centered on (960, 540) -> top-left at (480, 270).
        assert_eq!(result.x, 480);
        assert_eq!(result.y, 270);
    }

    // --- compute_source_region: edge clamping tests ---

    #[test]
    fn viewport_source_region_clamp_left() {
        let result = compute_source_region(
            ScreenPoint { x: 0, y: 540 },
            2.0,
            (1920, 1080),
            screen_1080p(),
        );
        assert_eq!(result.x, 0, "x should be clamped to screen left edge");
    }

    #[test]
    fn viewport_source_region_clamp_top() {
        let result = compute_source_region(
            ScreenPoint { x: 960, y: 0 },
            2.0,
            (1920, 1080),
            screen_1080p(),
        );
        assert_eq!(result.y, 0, "y should be clamped to screen top edge");
    }

    #[test]
    fn viewport_source_region_clamp_right() {
        let result = compute_source_region(
            ScreenPoint { x: 1920, y: 540 },
            2.0,
            (1920, 1080),
            screen_1080p(),
        );
        // Source is 960 wide, max x = 1920 - 960 = 960.
        assert_eq!(result.x, 960, "x should be clamped to screen right edge");
    }

    #[test]
    fn viewport_source_region_clamp_bottom() {
        let result = compute_source_region(
            ScreenPoint { x: 960, y: 1080 },
            2.0,
            (1920, 1080),
            screen_1080p(),
        );
        // Source is 540 tall, max y = 1080 - 540 = 540.
        assert_eq!(result.y, 540, "y should be clamped to screen bottom edge");
    }

    #[test]
    fn viewport_source_region_clamp_corner() {
        let result = compute_source_region(
            ScreenPoint { x: 0, y: 0 },
            2.0,
            (1920, 1080),
            screen_1080p(),
        );
        assert_eq!(result.x, 0, "x should be clamped to 0");
        assert_eq!(result.y, 0, "y should be clamped to 0");
    }

    #[test]
    fn viewport_source_region_zero_zoom() {
        let result = compute_source_region(
            ScreenPoint { x: 960, y: 540 },
            0.0,
            (1920, 1080),
            screen_1080p(),
        );
        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
    }

    // --- smooth_viewport_position tests ---

    #[test]
    fn smooth_viewport_factor_1_0() {
        let result = smooth_viewport_position(
            ScreenPoint { x: 0, y: 0 },
            ScreenPoint { x: 100, y: 200 },
            1.0,
        );
        assert_eq!(result, ScreenPoint { x: 100, y: 200 });
    }

    #[test]
    fn smooth_viewport_factor_0_0() {
        let result = smooth_viewport_position(
            ScreenPoint { x: 100, y: 200 },
            ScreenPoint { x: 300, y: 400 },
            0.0,
        );
        assert_eq!(result, ScreenPoint { x: 100, y: 200 });
    }

    #[test]
    fn smooth_viewport_factor_0_5() {
        let result = smooth_viewport_position(
            ScreenPoint { x: 0, y: 0 },
            ScreenPoint { x: 100, y: 200 },
            0.5,
        );
        assert_eq!(result, ScreenPoint { x: 50, y: 100 });
    }

    #[test]
    fn smooth_viewport_clamp_factor() {
        // Factor > 1.0 should be clamped to 1.0 (instant).
        let result = smooth_viewport_position(
            ScreenPoint { x: 0, y: 0 },
            ScreenPoint { x: 100, y: 200 },
            1.5,
        );
        assert_eq!(result, ScreenPoint { x: 100, y: 200 });
    }
}
