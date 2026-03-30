//! Viewport tracking engine for cursor-follow magnification.
//!
//! Provides [`TrackingEngine`] and [`TrackingConfig`] for computing
//! smooth viewport position updates based on mouse cursor movement.
//! The engine applies dead zone suppression, edge panning, and smooth
//! interpolation to produce fluid, comfortable viewport behavior.
//!
//! # Architecture
//!
//! `TrackingEngine` is a pure-logic component: no I/O, no GPU dependency,
//! no allocation on the hot path. It consumes [`ScreenPoint`] values from
//! the platform input monitor and produces [`ScreenPoint`] values consumed
//! by [`compute_source_region()`](luminos_gpu::viewport::compute_source_region).

use luminos_gpu::viewport::smooth_viewport_position;
use luminos_types::{ScreenPoint, ScreenRect};

/// Configuration for the viewport tracking engine.
///
/// Controls dead zone size, edge panning margins, and smooth
/// interpolation behavior. Defaults are tuned for comfortable
/// magnification navigation.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackingConfig {
    /// Smoothing factor for viewport interpolation (0.05 to 1.0).
    ///
    /// Lower values produce smoother but slower panning.
    /// Higher values produce more responsive but potentially jerky panning.
    /// - `1.0` = instant (no smoothing, viewport jumps to target)
    /// - `0.2` = default (smooth, comfortable panning over 3-5 frames)
    /// - `0.05` = very smooth (slow convergence, may feel sluggish)
    pub smoothing_factor: f32,

    /// Dead zone as a fraction of viewport dimensions (0.0 to 0.5).
    ///
    /// When the cursor's offset from the viewport center is within
    /// `dead_zone_percent * viewport_dimension / (2 * zoom)` pixels in each axis,
    /// no panning occurs. This prevents jitter from small cursor movements
    /// while the user is reading.
    /// - `0.0` = no dead zone (any movement pans)
    /// - `0.2` = default (20% of source region is dead zone)
    /// - `0.5` = maximum (50% dead zone, very stable but less responsive)
    pub dead_zone_percent: f32,

    /// Edge panning margin as a fraction of source region dimensions (0.0 to 0.3).
    ///
    /// When the cursor is within `edge_margin_percent * source_dimension`
    /// pixels of the source region edge, the viewport pans proportionally
    /// to the cursor's depth into the margin. Panning speed increases as
    /// the cursor moves deeper into the margin.
    /// - `0.0` = no edge panning
    /// - `0.15` = default (15% of source region width/height is edge margin)
    /// - `0.3` = maximum edge margin
    pub edge_margin_percent: f32,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            smoothing_factor: 0.2,
            dead_zone_percent: 0.2,
            edge_margin_percent: 0.15,
        }
    }
}

/// Viewport tracking engine for cursor-follow magnification.
///
/// Computes the smoothed viewport center each frame based on the
/// current mouse position, applying dead zone suppression, edge
/// panning, and smooth interpolation. The engine is a pure-logic
/// component with no I/O, GPU, or allocation on the hot path.
///
/// # Usage
///
/// ```ignore
/// let mut engine = TrackingEngine::new(TrackingConfig::default());
/// // Per frame:
/// let center = engine.update(mouse_pos, viewport_size, screen_bounds, zoom_level);
/// let source_region = compute_source_region(center, zoom_level, viewport_size, screen_bounds);
/// ```
pub struct TrackingEngine {
    /// Configuration (dead zone, edge margin, smoothing).
    config: TrackingConfig,
    /// Current smoothed viewport center in screen coordinates.
    current_center: ScreenPoint,
    /// Whether the engine has received its first update.
    initialized: bool,
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]
impl TrackingEngine {
    /// Creates a new tracking engine with the given configuration.
    ///
    /// The viewport center starts at `(0, 0)` and will snap to the
    /// cursor position on the first [`update()`](Self::update) call.
    #[must_use]
    pub fn new(config: TrackingConfig) -> Self {
        Self {
            config,
            current_center: ScreenPoint { x: 0, y: 0 },
            initialized: false,
        }
    }

    /// Updates the viewport center based on the current mouse position.
    ///
    /// Call this once per frame. Returns the new smoothed viewport center.
    ///
    /// On the first call, sets the viewport center directly to the mouse
    /// position (no smoothing on initialization).
    ///
    /// # Arguments
    ///
    /// * `mouse_position` -- Current cursor position in screen coordinates.
    /// * `viewport_size` -- Overlay viewport dimensions `(width, height)` in pixels.
    /// * `screen_bounds` -- Active display bounds (for edge panning reference).
    /// * `zoom_level` -- Current magnification factor (affects source region size).
    ///
    /// # Returns
    ///
    /// The new smoothed viewport center as a [`ScreenPoint`]. Pass this to
    /// [`compute_source_region()`](luminos_gpu::viewport::compute_source_region)
    /// to get the capture region.
    #[must_use]
    pub fn update(
        &mut self,
        mouse_position: ScreenPoint,
        viewport_size: (u32, u32),
        _screen_bounds: ScreenRect,
        zoom_level: f32,
    ) -> ScreenPoint {
        // First frame: snap to mouse position (no smoothing).
        if !self.initialized {
            self.current_center = mouse_position;
            self.initialized = true;
            return self.current_center;
        }

        // Step 1: Dead zone check.
        // Compute dead zone half-extents in screen pixels based on source region.
        let half_source_width = viewport_size.0 as f32 / (2.0 * zoom_level);
        let half_source_height = viewport_size.1 as f32 / (2.0 * zoom_level);
        let dead_half_x = half_source_width * self.config.dead_zone_percent;
        let dead_half_y = half_source_height * self.config.dead_zone_percent;

        let dx = (mouse_position.x - self.current_center.x) as f32;
        let dy = (mouse_position.y - self.current_center.y) as f32;

        if dx.abs() <= dead_half_x && dy.abs() <= dead_half_y {
            return self.current_center;
        }

        // Step 2: Start with mouse position as the base panning target.
        let mut target = mouse_position;

        // Step 3: Edge panning adjustment.
        // Compute source region dimensions at the current zoom level.
        let source_w = viewport_size.0 as f32 / zoom_level;
        let source_h = viewport_size.1 as f32 / zoom_level;
        let edge_margin_x = source_w * self.config.edge_margin_percent;
        let edge_margin_y = source_h * self.config.edge_margin_percent;

        // Source region bounds centered on current viewport center.
        let source_left = self.current_center.x as f32 - source_w / 2.0;
        let source_right = source_left + source_w;
        let source_top = self.current_center.y as f32 - source_h / 2.0;
        let source_bottom = source_top + source_h;

        let mx = mouse_position.x as f32;
        let my = mouse_position.y as f32;

        // Proportional panning: velocity scales with depth into margin.
        if mx < source_left + edge_margin_x && edge_margin_x > 0.0 {
            let depth = (source_left + edge_margin_x - mx) / edge_margin_x;
            target.x -= (depth * edge_margin_x) as i32;
        } else if mx > source_right - edge_margin_x && edge_margin_x > 0.0 {
            let depth = (mx - (source_right - edge_margin_x)) / edge_margin_x;
            target.x += (depth * edge_margin_x) as i32;
        }

        if my < source_top + edge_margin_y && edge_margin_y > 0.0 {
            let depth = (source_top + edge_margin_y - my) / edge_margin_y;
            target.y -= (depth * edge_margin_y) as i32;
        } else if my > source_bottom - edge_margin_y && edge_margin_y > 0.0 {
            let depth = (my - (source_bottom - edge_margin_y)) / edge_margin_y;
            target.y += (depth * edge_margin_y) as i32;
        }

        // Step 4: Smooth interpolation toward target.
        self.current_center =
            smooth_viewport_position(self.current_center, target, self.config.smoothing_factor);

        self.current_center
    }

    /// Returns the current smoothed viewport center.
    #[must_use]
    pub fn current_center(&self) -> ScreenPoint {
        self.current_center
    }

    /// Returns a reference to the tracking configuration.
    #[must_use]
    pub fn config(&self) -> &TrackingConfig {
        &self.config
    }

    /// Updates the tracking configuration.
    ///
    /// Takes effect on the next [`update()`](Self::update) call.
    pub fn set_config(&mut self, config: TrackingConfig) {
        self.config = config;
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
mod tests {
    use super::*;
    use luminos_gpu::viewport::compute_source_region;

    // --- Helper: standard 1080p screen bounds ---

    fn screen_1080p() -> ScreenRect {
        ScreenRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }
    }

    // =====================================================================
    // T001: TrackingConfig defaults
    // =====================================================================

    #[test]
    fn tracking_config_default_smoothing_factor() {
        let config = TrackingConfig::default();
        assert!(
            (config.smoothing_factor - 0.2).abs() < f32::EPSILON,
            "default smoothing_factor should be 0.2, got {}",
            config.smoothing_factor
        );
    }

    #[test]
    fn tracking_config_default_dead_zone_percent() {
        let config = TrackingConfig::default();
        assert!(
            (config.dead_zone_percent - 0.2).abs() < f32::EPSILON,
            "default dead_zone_percent should be 0.2, got {}",
            config.dead_zone_percent
        );
    }

    #[test]
    fn tracking_config_default_edge_margin_percent() {
        let config = TrackingConfig::default();
        assert!(
            (config.edge_margin_percent - 0.15).abs() < f32::EPSILON,
            "default edge_margin_percent should be 0.15, got {}",
            config.edge_margin_percent
        );
    }

    #[test]
    fn tracking_config_derives_debug_clone_partial_eq() {
        let config = TrackingConfig::default();
        let cloned = config.clone();
        assert_eq!(config, cloned);
        // Debug derive check: format should not panic.
        let _debug = format!("{config:?}");
    }

    // =====================================================================
    // T002: TrackingEngine constructor and first-frame initialization
    // =====================================================================

    #[test]
    fn tracking_engine_new_has_zero_center() {
        let engine = TrackingEngine::new(TrackingConfig::default());
        assert_eq!(engine.current_center(), ScreenPoint { x: 0, y: 0 });
    }

    #[test]
    fn tracking_engine_first_frame_snaps_to_cursor() {
        let mut engine = TrackingEngine::new(TrackingConfig::default());
        let result = engine.update(
            ScreenPoint { x: 500, y: 300 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert_eq!(result, ScreenPoint { x: 500, y: 300 });
        assert_eq!(engine.current_center(), ScreenPoint { x: 500, y: 300 });
    }

    #[test]
    fn tracking_engine_config_accessor() {
        let config = TrackingConfig {
            smoothing_factor: 0.5,
            dead_zone_percent: 0.3,
            edge_margin_percent: 0.1,
        };
        let engine = TrackingEngine::new(config.clone());
        assert_eq!(*engine.config(), config);
    }

    #[test]
    fn tracking_engine_set_config() {
        let mut engine = TrackingEngine::new(TrackingConfig::default());
        let new_config = TrackingConfig {
            smoothing_factor: 0.5,
            dead_zone_percent: 0.1,
            edge_margin_percent: 0.25,
        };
        engine.set_config(new_config.clone());
        assert_eq!(*engine.config(), new_config);
    }

    // =====================================================================
    // T003: Dead zone suppression
    // =====================================================================

    #[test]
    fn tracking_dead_zone_suppresses_micro_movement() {
        // dead_zone_percent=0.2, viewport 1920x1080, zoom 2.0
        // source region = 960x540, half = 480x270
        // dead zone half-width = 480 * 0.2 = 96px, half-height = 270 * 0.2 = 54px
        let mut engine = TrackingEngine::new(TrackingConfig::default());
        // First frame: snap to center.
        let _ = engine.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        // Move cursor 10px right, 5px down -- well within dead zone.
        let result = engine.update(
            ScreenPoint { x: 970, y: 545 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert_eq!(
            result,
            ScreenPoint { x: 960, y: 540 },
            "cursor within dead zone should not cause panning"
        );
    }

    #[test]
    fn tracking_dead_zone_boundary_no_panning() {
        // Dead zone half-width = 96px at zoom 2.0 on 1920 viewport.
        // Move cursor exactly to boundary (960 + 96 = 1056).
        let mut engine = TrackingEngine::new(TrackingConfig::default());
        let _ = engine.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        let result = engine.update(
            ScreenPoint { x: 1056, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert_eq!(
            result,
            ScreenPoint { x: 960, y: 540 },
            "cursor at dead zone boundary should not cause panning"
        );
    }

    #[test]
    fn tracking_dead_zone_exit_triggers_panning() {
        // Move cursor well outside dead zone half-width of 96px.
        let config = TrackingConfig {
            smoothing_factor: 1.0, // instant so we can verify target
            ..TrackingConfig::default()
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        // Move cursor 140px right (outside dead zone of 96px).
        let result = engine.update(
            ScreenPoint { x: 1100, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert_ne!(
            result,
            ScreenPoint { x: 960, y: 540 },
            "cursor outside dead zone should trigger panning"
        );
    }

    #[test]
    fn tracking_dead_zone_zero_disables() {
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        // Move cursor by just 1 pixel.
        let result = engine.update(
            ScreenPoint { x: 961, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert_eq!(
            result,
            ScreenPoint { x: 961, y: 540 },
            "with dead zone 0% and smoothing 1.0, 1px movement should pan instantly"
        );
    }

    // =====================================================================
    // T004: Smooth interpolation toward target
    // =====================================================================

    #[test]
    fn tracking_smooth_convergence_over_frames() {
        let config = TrackingConfig {
            smoothing_factor: 0.2,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        // First frame: snap to (0, 0).
        let _ = engine.update(
            ScreenPoint { x: 0, y: 0 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        let target = ScreenPoint { x: 1000, y: 500 };
        let mut prev_dist_sq = i64::MAX;

        for frame in 0..10 {
            let center = engine.update(target, (1920, 1080), screen_1080p(), 2.0);
            let dist_sq = (center.x as i64 - target.x as i64).pow(2)
                + (center.y as i64 - target.y as i64).pow(2);
            assert!(
                dist_sq < prev_dist_sq,
                "frame {frame}: distance should decrease each frame (was {prev_dist_sq}, now {dist_sq})"
            );
            prev_dist_sq = dist_sq;
        }

        // After 10 frames with factor 0.2, should be within ~85% of target.
        // Remaining distance = (1 - 0.2)^10 = 0.8^10 ≈ 0.107 of original.
        let final_center = engine.current_center();
        let remaining_x = (target.x - final_center.x).unsigned_abs();
        let remaining_y = (target.y - final_center.y).unsigned_abs();
        assert!(
            remaining_x < 150,
            "x should converge within ~85% after 10 frames, remaining: {remaining_x}"
        );
        assert!(
            remaining_y < 75,
            "y should converge within ~85% after 10 frames, remaining: {remaining_y}"
        );
    }

    #[test]
    fn tracking_smooth_frame_delta_bounded() {
        let config = TrackingConfig {
            smoothing_factor: 0.2,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 0, y: 0 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        let target = ScreenPoint { x: 1000, y: 500 };
        let mut prev_center = engine.current_center();

        for frame in 0..20 {
            let center = engine.update(target, (1920, 1080), screen_1080p(), 2.0);
            let delta_x = (center.x - prev_center.x).abs();
            let delta_y = (center.y - prev_center.y).abs();
            let max_delta_x = ((target.x - prev_center.x) as f32 * 0.2).abs() as i32 + 1;
            let max_delta_y = ((target.y - prev_center.y) as f32 * 0.2).abs() as i32 + 1;
            assert!(
                delta_x <= max_delta_x,
                "frame {frame}: x delta {delta_x} exceeds max {max_delta_x}"
            );
            assert!(
                delta_y <= max_delta_y,
                "frame {frame}: y delta {delta_y} exceeds max {max_delta_y}"
            );
            prev_center = center;
        }
    }

    #[test]
    fn tracking_instant_tracking_factor_1() {
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 0, y: 0 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        let result = engine.update(
            ScreenPoint { x: 500, y: 300 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert_eq!(
            result,
            ScreenPoint { x: 500, y: 300 },
            "smoothing_factor=1.0 should produce instant tracking"
        );
    }

    #[test]
    fn tracking_smooth_preserves_asymptotic_approach() {
        let config = TrackingConfig {
            smoothing_factor: 0.2,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 0, y: 0 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        let target = ScreenPoint { x: 1000, y: 0 };
        let mut distances = Vec::new();

        for _ in 0..8 {
            let center = engine.update(target, (1920, 1080), screen_1080p(), 2.0);
            let dist = (target.x - center.x).abs();
            distances.push(dist);
        }

        // Each remaining distance should be roughly 80% of the previous
        // (geometric decay with factor 0.2 -> 80% remaining each frame).
        for i in 1..distances.len() {
            if distances[i - 1] == 0 {
                break;
            }
            let ratio = distances[i] as f32 / distances[i - 1] as f32;
            // Allow generous tolerance for integer rounding.
            assert!(
                ratio < 0.95,
                "frame {i}: ratio {ratio} should show geometric decay (< 0.95)"
            );
        }
    }

    // =====================================================================
    // T005: Edge panning
    // =====================================================================

    #[test]
    fn tracking_edge_panning_right_margin() {
        // edge_margin_percent=0.15, viewport 1920x1080, zoom 2.0
        // source region = 960x540
        // edge_margin_x = 960 * 0.15 = 144px
        // right edge margin starts at: center_x + source_w/2 - edge_margin_x
        // = 960 + 480 - 144 = 1296
        let config = TrackingConfig {
            smoothing_factor: 1.0, // instant for testing
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.15,
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        // Place cursor in the right edge margin.
        let result = engine.update(
            ScreenPoint { x: 1400, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert!(
            result.x > 1400,
            "edge panning should shift target rightward beyond cursor, got x={}",
            result.x
        );
    }

    #[test]
    fn tracking_edge_panning_proportional_speed() {
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.15,
        };

        // Test at 86% of source width from source left.
        let mut engine_shallow = TrackingEngine::new(config.clone());
        let _ = engine_shallow.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        // Source left = 960 - 480 = 480. 86% of 960 = 825.6. Cursor at 480 + 826 = 1306.
        let result_shallow = engine_shallow.update(
            ScreenPoint { x: 1306, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        // Test at 98% of source width from source left.
        let mut engine_deep = TrackingEngine::new(config);
        let _ = engine_deep.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        // 98% of 960 = 940.8. Cursor at 480 + 941 = 1421.
        let result_deep = engine_deep.update(
            ScreenPoint { x: 1421, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        // The deep-margin cursor should produce a larger rightward shift.
        let shift_shallow = result_shallow.x - 1306;
        let shift_deep = result_deep.x - 1421;
        assert!(
            shift_deep > shift_shallow,
            "deeper margin should produce larger shift: deep={shift_deep}, shallow={shift_shallow}"
        );
    }

    #[test]
    fn tracking_edge_panning_inactive_in_content_area() {
        // Cursor between dead zone and edge margin -> no edge panning.
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.15,
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        // Place cursor in the content area (not near edges of source region).
        // Source region: 480..1440 x 270..810.
        // Edge margin x = 144px. Content area starts at 480+144=624, ends at 1440-144=1296.
        // Cursor at (1000, 540) is in content area.
        let result = engine.update(
            ScreenPoint { x: 1000, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert_eq!(
            result,
            ScreenPoint { x: 1000, y: 540 },
            "cursor in content area with instant tracking should match cursor position exactly"
        );
    }

    #[test]
    fn tracking_edge_panning_left_margin() {
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.15,
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        // Source left = 480. Edge margin starts at 480 + 144 = 624.
        // Place cursor at 500 (in left edge margin).
        let result = engine.update(
            ScreenPoint { x: 500, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert!(
            result.x < 500,
            "edge panning should shift target leftward, got x={}",
            result.x
        );
    }

    #[test]
    fn tracking_edge_panning_vertical() {
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.15,
        };
        let mut engine_top = TrackingEngine::new(config.clone());
        let _ = engine_top.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        // Source top = 270. Edge margin = 540 * 0.15 = 81px. Margin boundary = 270 + 81 = 351.
        // Cursor at y=280 (in top edge margin).
        let result_top = engine_top.update(
            ScreenPoint { x: 960, y: 280 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert!(
            result_top.y < 280,
            "top edge panning should shift upward, got y={}",
            result_top.y
        );

        // Bottom edge margin.
        let mut engine_bottom = TrackingEngine::new(config);
        let _ = engine_bottom.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        // Source bottom = 810. Bottom margin starts at 810 - 81 = 729.
        // Cursor at y=800 (in bottom edge margin).
        let result_bottom = engine_bottom.update(
            ScreenPoint { x: 960, y: 800 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert!(
            result_bottom.y > 800,
            "bottom edge panning should shift downward, got y={}",
            result_bottom.y
        );
    }

    #[test]
    fn tracking_edge_panning_disabled_zero_margin() {
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );

        // Cursor at extreme right edge.
        let result = engine.update(
            ScreenPoint { x: 1430, y: 540 },
            (1920, 1080),
            screen_1080p(),
            2.0,
        );
        assert_eq!(
            result,
            ScreenPoint { x: 1430, y: 540 },
            "edge_margin_percent=0.0 should produce no edge panning"
        );
    }

    // =====================================================================
    // T006: Multi-zoom-level behavior
    // =====================================================================

    #[test]
    fn tracking_zoom_1_5x_correct_dimensions() {
        // zoom 1.5, viewport 1920x1080 -> source ~1280x720
        // dead zone half-width = (1920 / (2*1.5)) * 0.2 = 640 * 0.2 = 128px
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.2,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            1.5,
        );

        // Move within dead zone (120px < 128px).
        let result = engine.update(
            ScreenPoint { x: 1080, y: 540 },
            (1920, 1080),
            screen_1080p(),
            1.5,
        );
        assert_eq!(
            result,
            ScreenPoint { x: 960, y: 540 },
            "at 1.5x zoom, 120px movement should be in dead zone (half-width=128px)"
        );

        // Move outside dead zone (140px > 128px).
        let result = engine.update(
            ScreenPoint { x: 1100, y: 540 },
            (1920, 1080),
            screen_1080p(),
            1.5,
        );
        assert_ne!(
            result,
            ScreenPoint { x: 960, y: 540 },
            "at 1.5x zoom, 140px movement should exit dead zone"
        );
    }

    #[test]
    fn tracking_zoom_20x_correct_dimensions() {
        // zoom 20, viewport 1920x1080 -> source 96x54
        // dead zone half-width = (1920 / (2*20)) * 0.2 = 48 * 0.2 = 9.6px
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.2,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            20.0,
        );

        // Move within dead zone (5px < 9.6px).
        let result = engine.update(
            ScreenPoint { x: 965, y: 540 },
            (1920, 1080),
            screen_1080p(),
            20.0,
        );
        assert_eq!(
            result,
            ScreenPoint { x: 960, y: 540 },
            "at 20x zoom, 5px movement should be in dead zone"
        );

        // Move outside dead zone (15px > 9.6px).
        let result = engine.update(
            ScreenPoint { x: 975, y: 540 },
            (1920, 1080),
            screen_1080p(),
            20.0,
        );
        assert_ne!(
            result,
            ScreenPoint { x: 960, y: 540 },
            "at 20x zoom, 15px movement should exit dead zone"
        );
    }

    #[test]
    fn tracking_zoom_5x_dead_zone_scales() {
        // zoom 5, viewport 1920x1080 -> source 384x216
        // dead zone half-width = (1920 / (2*5)) * 0.2 = 192 * 0.2 = 38.4px
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.2,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        let _ = engine.update(
            ScreenPoint { x: 960, y: 540 },
            (1920, 1080),
            screen_1080p(),
            5.0,
        );

        // Move 30px (within dead zone of 38.4px).
        let result = engine.update(
            ScreenPoint { x: 990, y: 540 },
            (1920, 1080),
            screen_1080p(),
            5.0,
        );
        assert_eq!(
            result,
            ScreenPoint { x: 960, y: 540 },
            "at 5x zoom, 30px movement should be in dead zone (half-width ≈ 38px)"
        );

        // Move 45px (outside dead zone).
        let result = engine.update(
            ScreenPoint { x: 1005, y: 540 },
            (1920, 1080),
            screen_1080p(),
            5.0,
        );
        assert_ne!(
            result,
            ScreenPoint { x: 960, y: 540 },
            "at 5x zoom, 45px movement should exit dead zone"
        );
    }

    // =====================================================================
    // T007: Integration tests with compute_source_region
    // =====================================================================

    #[test]
    fn tracking_integration_source_region_within_bounds_top_left() {
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        let bounds = screen_1080p();
        let center = engine.update(ScreenPoint { x: 0, y: 0 }, (1920, 1080), bounds, 2.0);
        let region = compute_source_region(center, 2.0, (1920, 1080), bounds);
        assert!(
            region.x >= 0,
            "source region x should be >= 0, got {}",
            region.x
        );
        assert!(
            region.y >= 0,
            "source region y should be >= 0, got {}",
            region.y
        );
    }

    #[test]
    fn tracking_integration_source_region_within_bounds_bottom_right() {
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        let bounds = screen_1080p();
        let center = engine.update(ScreenPoint { x: 1920, y: 1080 }, (1920, 1080), bounds, 2.0);
        let region = compute_source_region(center, 2.0, (1920, 1080), bounds);
        assert!(
            region.x + region.width as i32 <= 1920,
            "source region right edge {} should be <= 1920",
            region.x + region.width as i32
        );
        assert!(
            region.y + region.height as i32 <= 1080,
            "source region bottom edge {} should be <= 1080",
            region.y + region.height as i32
        );
    }

    #[test]
    fn tracking_integration_source_region_multi_monitor() {
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        // Second monitor at x=1920.
        let bounds = ScreenRect {
            x: 1920,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let center = engine.update(ScreenPoint { x: 2880, y: 540 }, (1920, 1080), bounds, 2.0);
        let region = compute_source_region(center, 2.0, (1920, 1080), bounds);
        assert!(
            region.x >= bounds.x,
            "multi-monitor: region x {} should be >= {}",
            region.x,
            bounds.x
        );
        assert!(
            region.x + region.width as i32 <= bounds.x + bounds.width as i32,
            "multi-monitor: region right {} should be <= {}",
            region.x + region.width as i32,
            bounds.x + bounds.width as i32
        );
    }

    #[test]
    fn tracking_integration_source_region_correct_dimensions_2x() {
        let config = TrackingConfig {
            smoothing_factor: 1.0,
            dead_zone_percent: 0.0,
            edge_margin_percent: 0.0,
        };
        let mut engine = TrackingEngine::new(config);
        let bounds = screen_1080p();
        let center = engine.update(ScreenPoint { x: 960, y: 540 }, (1920, 1080), bounds, 2.0);
        let region = compute_source_region(center, 2.0, (1920, 1080), bounds);
        assert_eq!(region.width, 960, "2x zoom source width should be 960");
        assert_eq!(region.height, 540, "2x zoom source height should be 540");
    }

    // =====================================================================
    // T008: Performance micro-benchmark
    // =====================================================================

    #[test]
    fn tracking_update_latency_under_10us() {
        let config = TrackingConfig::default();
        let mut engine = TrackingEngine::new(config);
        let bounds = screen_1080p();

        // Warm up with 100 calls.
        for i in 0..100 {
            std::hint::black_box(engine.update(
                ScreenPoint {
                    x: 960 + (i % 50),
                    y: 540 + (i % 30),
                },
                (1920, 1080),
                bounds,
                2.0,
            ));
        }

        // Measure 10,000 calls.
        let iterations = 10_000;
        let start = std::time::Instant::now();
        for i in 0..iterations {
            std::hint::black_box(engine.update(
                ScreenPoint {
                    x: 500 + (i % 1000),
                    y: 300 + (i % 600),
                },
                (1920, 1080),
                bounds,
                2.0,
            ));
        }
        let elapsed = start.elapsed();
        let avg_ns = elapsed.as_nanos() / iterations as u128;

        assert!(
            avg_ns < 10_000,
            "average update() latency should be < 10us (10,000ns), got {avg_ns}ns"
        );
    }
}
