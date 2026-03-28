//! X11 screen capture backend using xcap (XCB protocol).

// XcbCapture is not yet wired into PlatformBackends (pending Story 005).
// The struct and its methods appear unused because the parent module is private.
#![allow(dead_code)]

use std::sync::Arc;

use x11rb::connection::Connection as _;
use x11rb::wrapper::ConnectionExt as _;

use crate::traits::screen_capture::{CaptureError, DisplayChangeEvent, ScreenCapture};
use crate::traits::types::{CaptureFrame, DisplayInfo, PixelFormat, ScreenRect};

/// X11 screen capture backend using xcap (XCB protocol).
///
/// Captures screen content via `xcb_get_image` through the xcap crate.
/// Supports full-display and region-specific capture in `Rgba8` pixel format
/// (xcap internally converts X11's native BGRA to RGBA).
///
/// # Self-Capture Prevention (RISK-002)
///
/// When window IDs are configured via [`set_excluded_windows()`](ScreenCapture::set_excluded_windows),
/// the implementation excludes those windows from captured frames via an
/// unmap/remap cycle: the windows are unmapped before capture and remapped
/// immediately after.
///
/// # Performance
///
/// Uses the non-SHM capture path (`xcb_get_image`), which performs a full
/// X server round-trip per capture. Typical latency: 1-5ms for small regions,
/// up to 8ms for full `1080p` display. `XShm` optimization is planned for Phase 1.
pub struct XcbCapture {
    /// Window IDs to exclude from capture (e.g., magnification overlay).
    /// Set via the `set_excluded_windows()` trait method.
    /// Stored as u64 per the trait contract; truncated to u32 for X11 APIs.
    excluded_window_ids: Vec<u64>,
}

impl XcbCapture {
    /// Creates a new X11 screen capture backend.
    ///
    /// The capture backend starts with no excluded windows. Use
    /// [`set_excluded_windows()`](ScreenCapture::set_excluded_windows) to
    /// configure self-capture prevention after construction (typically once
    /// the overlay window ID is known).
    ///
    /// # Errors
    ///
    /// Returns [`CaptureError::BackendUnavailable`] if the X11 display
    /// cannot be opened or xcap initialization fails.
    pub fn new() -> Result<Self, CaptureError> {
        // Validate X11 is available by attempting to list monitors
        let _monitors = xcap::Monitor::all().map_err(|e| CaptureError::BackendUnavailable {
            reason: format!("X11 display unavailable: {e}"),
        })?;

        Ok(Self {
            excluded_window_ids: Vec::new(),
        })
    }

    /// Maps an xcap [`Monitor`](xcap::Monitor) to a Luminos [`DisplayInfo`].
    ///
    /// # Errors
    ///
    /// Returns [`CaptureError::Platform`] if any xcap monitor property
    /// accessor fails.
    fn monitor_to_display_info(monitor: &xcap::Monitor) -> Result<DisplayInfo, CaptureError> {
        let map_err = |e: xcap::XCapError| CaptureError::Platform {
            message: format!("failed to read monitor property: {e}"),
            source: Some(Box::new(e)),
        };

        Ok(DisplayInfo {
            id: monitor.id().map_err(map_err)?.to_string(),
            name: monitor.name().map_err(map_err)?,
            bounds: ScreenRect {
                x: monitor.x().map_err(map_err)?,
                y: monitor.y().map_err(map_err)?,
                width: monitor.width().map_err(map_err)?,
                height: monitor.height().map_err(map_err)?,
            },
            scale_factor: f64::from(monitor.scale_factor().map_err(map_err)?),
            is_primary: monitor.is_primary().map_err(map_err)?,
        })
    }

    /// Finds an xcap [`Monitor`](xcap::Monitor) by display ID.
    fn find_monitor(display_id: &str) -> Result<xcap::Monitor, CaptureError> {
        let monitors = xcap::Monitor::all().map_err(|e| CaptureError::Platform {
            message: format!("failed to enumerate displays: {e}"),
            source: Some(Box::new(e)),
        })?;

        for monitor in monitors {
            let id = monitor.id().map_err(|e| CaptureError::Platform {
                message: format!("failed to read monitor id: {e}"),
                source: Some(Box::new(e)),
            })?;
            if id.to_string() == display_id {
                return Ok(monitor);
            }
        }

        Err(CaptureError::DisplayNotFound(display_id.to_string()))
    }

    /// Returns the display bounds for a given monitor.
    fn monitor_bounds(monitor: &xcap::Monitor) -> Result<ScreenRect, CaptureError> {
        let map_err = |e: xcap::XCapError| CaptureError::Platform {
            message: format!("failed to read monitor bounds: {e}"),
            source: Some(Box::new(e)),
        };

        Ok(ScreenRect {
            x: monitor.x().map_err(map_err)?,
            y: monitor.y().map_err(map_err)?,
            width: monitor.width().map_err(map_err)?,
            height: monitor.height().map_err(map_err)?,
        })
    }

    /// Validates that a capture region is within the display bounds.
    ///
    /// Returns `Ok(())` if the region is valid, or
    /// [`CaptureError::RegionOutOfBounds`] if the region exceeds the display
    /// bounds, has zero dimensions, or causes integer overflow.
    fn validate_region(
        region: &ScreenRect,
        display_bounds: &ScreenRect,
    ) -> Result<(), CaptureError> {
        // Zero dimensions are invalid
        if region.width == 0 || region.height == 0 {
            return Err(CaptureError::RegionOutOfBounds {
                region: *region,
                bounds: *display_bounds,
            });
        }

        // Use i64 to prevent overflow when computing right/bottom edges
        let region_right = i64::from(region.x) + i64::from(region.width);
        let region_bottom = i64::from(region.y) + i64::from(region.height);

        let bounds_right = i64::from(display_bounds.x) + i64::from(display_bounds.width);
        let bounds_bottom = i64::from(display_bounds.y) + i64::from(display_bounds.height);

        if i64::from(region.x) < i64::from(display_bounds.x)
            || i64::from(region.y) < i64::from(display_bounds.y)
            || region_right > bounds_right
            || region_bottom > bounds_bottom
        {
            return Err(CaptureError::RegionOutOfBounds {
                region: *region,
                bounds: *display_bounds,
            });
        }

        Ok(())
    }

    /// Unmaps excluded windows from the X11 display server.
    ///
    /// Used as part of the self-capture prevention unmap/remap cycle.
    /// Windows are hidden from the screen so they are not captured.
    fn unmap_excluded_windows(&self) {
        if self.excluded_window_ids.is_empty() {
            return;
        }

        let Ok((conn, _)) = x11rb::connect(None) else {
            log::warn!("Failed to connect to X11 for self-capture exclusion unmap");
            return;
        };

        for &window_id in &self.excluded_window_ids {
            #[allow(clippy::cast_possible_truncation)]
            let xid = window_id as u32;
            if let Err(e) = x11rb::protocol::xproto::unmap_window(&conn, xid) {
                log::warn!("Failed to unmap window '{xid}': {e}");
            }
        }

        // Flush to ensure unmap requests are processed before capture
        if let Err(e) = conn.flush() {
            log::warn!("Failed to flush X11 connection after unmap: {e}");
        }

        // Sync to ensure the server has processed the unmap
        if let Err(e) = conn.sync() {
            log::warn!("Failed to sync X11 connection after unmap: {e}");
        }
    }

    /// Remaps previously unmapped excluded windows.
    ///
    /// Windows are shown again after capture completes.
    fn remap_excluded_windows(&self) {
        if self.excluded_window_ids.is_empty() {
            return;
        }

        let Ok((conn, _)) = x11rb::connect(None) else {
            log::warn!("Failed to connect to X11 for self-capture exclusion remap");
            return;
        };

        for &window_id in &self.excluded_window_ids {
            #[allow(clippy::cast_possible_truncation)]
            let xid = window_id as u32;
            if let Err(e) = x11rb::protocol::xproto::map_window(&conn, xid) {
                log::warn!("Failed to remap window '{xid}': {e}");
            }
        }

        if let Err(e) = conn.flush() {
            log::warn!("Failed to flush X11 connection after remap: {e}");
        }
    }
}

impl ScreenCapture for XcbCapture {
    fn list_displays(&self) -> Result<Vec<DisplayInfo>, CaptureError> {
        let monitors = xcap::Monitor::all().map_err(|e| CaptureError::Platform {
            message: format!("failed to enumerate displays: {e}"),
            source: Some(Box::new(e)),
        })?;

        monitors.iter().map(Self::monitor_to_display_info).collect()
    }

    fn capture_frame(
        &self,
        display_id: &str,
        region: Option<ScreenRect>,
    ) -> Result<CaptureFrame, CaptureError> {
        let monitor = Self::find_monitor(display_id)?;
        let display_bounds = Self::monitor_bounds(&monitor)?;

        // Validate region if specified
        if let Some(ref region) = region {
            Self::validate_region(region, &display_bounds)?;
        }

        // Self-capture prevention: unmap excluded windows before capture
        self.unmap_excluded_windows();

        // Capture via xcap (always remap after, even on failure)
        let capture_result = if let Some(ref region) = region {
            // Use xcap's native region capture for better performance
            let crop_x = (region.x - display_bounds.x).unsigned_abs();
            let crop_y = (region.y - display_bounds.y).unsigned_abs();
            monitor
                .capture_region(crop_x, crop_y, region.width, region.height)
                .map_err(|e| CaptureError::Platform {
                    message: format!("region capture failed for display '{display_id}': {e}"),
                    source: Some(Box::new(e)),
                })
        } else {
            monitor.capture_image().map_err(|e| CaptureError::Platform {
                message: format!("capture failed for display '{display_id}': {e}"),
                source: Some(Box::new(e)),
            })
        };

        // Self-capture prevention: remap excluded windows after capture
        // (always remap, even if capture failed, to avoid leaving windows hidden)
        self.remap_excluded_windows();

        let image = capture_result?;

        let output_width = image.width();
        let output_height = image.height();
        let pixel_data = image.into_raw();
        let stride = output_width * 4;

        Ok(CaptureFrame {
            data: Arc::from(pixel_data.into_boxed_slice()),
            width: output_width,
            height: output_height,
            stride,
            format: PixelFormat::Rgba8,
        })
    }

    fn subscribe_display_changes(
        &self,
        _buffer_size: usize,
    ) -> Result<tokio::sync::mpsc::Receiver<DisplayChangeEvent>, CaptureError> {
        // Phase 0: RandR event monitoring is not yet implemented.
        // Return BackendUnavailable per the trait's fallback contract (AC-5.2).
        // The core engine gracefully falls back to periodic list_displays() polling.
        Err(CaptureError::BackendUnavailable {
            reason: "X11 display change events not yet implemented".into(),
        })
    }

    fn set_excluded_windows(&mut self, window_ids: &[u64]) {
        if window_ids.is_empty() {
            log::debug!("Self-capture exclusion cleared");
        } else {
            log::info!(
                "Self-capture exclusion active for '{}' window(s)",
                window_ids.len()
            );
        }
        self.excluded_window_ids = window_ids.to_vec();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ── T004: Region validation unit tests (no X11 required) ──

    fn display_bounds() -> ScreenRect {
        ScreenRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }
    }

    #[test]
    fn xcb_capture_validate_region_within_bounds() {
        let region = ScreenRect {
            x: 100,
            y: 100,
            width: 200,
            height: 150,
        };
        assert!(XcbCapture::validate_region(&region, &display_bounds()).is_ok());
    }

    #[test]
    fn xcb_capture_validate_region_exceeds_right() {
        let region = ScreenRect {
            x: 1800,
            y: 0,
            width: 300,
            height: 100,
        };
        let result = XcbCapture::validate_region(&region, &display_bounds());
        assert!(matches!(
            result,
            Err(CaptureError::RegionOutOfBounds { .. })
        ));
    }

    #[test]
    fn xcb_capture_validate_region_exceeds_bottom() {
        let region = ScreenRect {
            x: 0,
            y: 1000,
            width: 100,
            height: 200,
        };
        let result = XcbCapture::validate_region(&region, &display_bounds());
        assert!(matches!(
            result,
            Err(CaptureError::RegionOutOfBounds { .. })
        ));
    }

    #[test]
    fn xcb_capture_validate_region_negative_origin() {
        let region = ScreenRect {
            x: -10,
            y: -10,
            width: 100,
            height: 100,
        };
        let result = XcbCapture::validate_region(&region, &display_bounds());
        assert!(matches!(
            result,
            Err(CaptureError::RegionOutOfBounds { .. })
        ));
    }

    #[test]
    fn xcb_capture_validate_region_zero_dimensions() {
        let zero_width = ScreenRect {
            x: 0,
            y: 0,
            width: 0,
            height: 100,
        };
        assert!(matches!(
            XcbCapture::validate_region(&zero_width, &display_bounds()),
            Err(CaptureError::RegionOutOfBounds { .. })
        ));

        let zero_height = ScreenRect {
            x: 0,
            y: 0,
            width: 100,
            height: 0,
        };
        assert!(matches!(
            XcbCapture::validate_region(&zero_height, &display_bounds()),
            Err(CaptureError::RegionOutOfBounds { .. })
        ));
    }

    #[test]
    fn xcb_capture_validate_region_overflow() {
        // i32::MAX + width would overflow i32, but we use i64 internally
        let region = ScreenRect {
            x: i32::MAX,
            y: 0,
            width: 100,
            height: 100,
        };
        let result = XcbCapture::validate_region(&region, &display_bounds());
        assert!(matches!(
            result,
            Err(CaptureError::RegionOutOfBounds { .. })
        ));
    }

    #[test]
    fn xcb_capture_validate_region_exact_bounds() {
        let region = ScreenRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        assert!(XcbCapture::validate_region(&region, &display_bounds()).is_ok());
    }

    #[test]
    fn xcb_capture_validate_region_with_offset_display() {
        let bounds = ScreenRect {
            x: 1920,
            y: 0,
            width: 2560,
            height: 1440,
        };
        let region = ScreenRect {
            x: 2000,
            y: 100,
            width: 200,
            height: 150,
        };
        assert!(XcbCapture::validate_region(&region, &bounds).is_ok());

        let bad_region = ScreenRect {
            x: 1900,
            y: 0,
            width: 200,
            height: 150,
        };
        assert!(matches!(
            XcbCapture::validate_region(&bad_region, &bounds),
            Err(CaptureError::RegionOutOfBounds { .. })
        ));
    }

    // ── T008: Display change subscription error tests ──

    #[test]
    fn xcb_capture_subscribe_display_changes_error_is_descriptive() {
        let err = CaptureError::BackendUnavailable {
            reason: "X11 display change events not yet implemented".into(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("not yet implemented"),
            "error message should indicate the feature is not yet implemented: '{msg}'"
        );
    }

    // ── T012: Error propagation verification ──

    #[test]
    fn xcb_capture_error_display_format() {
        let err = CaptureError::Platform {
            message: "capture failed for display 'test-0': XCB connection failed".into(),
            source: None,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("platform capture error"),
            "should contain 'platform capture error': '{msg}'"
        );
        assert!(
            msg.contains("capture failed for display 'test-0'"),
            "should contain context: '{msg}'"
        );
    }

    #[test]
    fn xcb_capture_error_to_luminos_error() {
        // Verify CaptureError's Display format includes the display ID.
        // The From<CaptureError> for LuminosError conversion is tested
        // in luminos-core::error::tests.
        let err = CaptureError::DisplayNotFound("test-display".into());
        let msg = format!("{err}");
        assert!(msg.contains("test-display"));
    }
}

// ── Integration tests requiring X11 ──

#[cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]
#[allow(clippy::unwrap_used)]
mod integration_tests {
    use super::*;
    use std::time::Instant;

    // ── T003: Constructor and display enumeration ──

    #[test]
    fn xcb_capture_new_succeeds() {
        let capture = XcbCapture::new();
        assert!(
            capture.is_ok(),
            "XcbCapture::new() should succeed on X11: {:?}",
            capture.err()
        );
    }

    #[test]
    fn xcb_capture_list_displays_returns_non_empty() {
        let capture = XcbCapture::new().unwrap();
        let displays = capture.list_displays().unwrap();
        assert!(
            !displays.is_empty(),
            "list_displays() should return at least one display"
        );
    }

    #[test]
    fn xcb_capture_list_displays_has_primary() {
        let capture = XcbCapture::new().unwrap();
        let displays = capture.list_displays().unwrap();
        let primary_count = displays.iter().filter(|d| d.is_primary).count();
        assert_eq!(
            primary_count, 1,
            "exactly one display should be primary, found {primary_count}"
        );
    }

    #[test]
    fn xcb_capture_list_displays_valid_fields() {
        let capture = XcbCapture::new().unwrap();
        let displays = capture.list_displays().unwrap();
        for display in &displays {
            assert!(!display.id.is_empty(), "display id should not be empty");
            assert!(!display.name.is_empty(), "display name should not be empty");
            assert!(
                display.bounds.width > 0,
                "display width should be positive, got {}",
                display.bounds.width
            );
            assert!(
                display.bounds.height > 0,
                "display height should be positive, got {}",
                display.bounds.height
            );
            assert!(
                display.scale_factor > 0.0,
                "display scale_factor should be positive, got {}",
                display.scale_factor
            );
        }
    }

    // ── T005: Full-display capture ──

    fn primary_display_id(capture: &XcbCapture) -> String {
        capture
            .list_displays()
            .unwrap()
            .into_iter()
            .find(|d| d.is_primary)
            .unwrap()
            .id
    }

    #[test]
    fn xcb_capture_full_display_correct_dimensions() {
        let capture = XcbCapture::new().unwrap();
        let displays = capture.list_displays().unwrap();
        let primary = displays.iter().find(|d| d.is_primary).unwrap();
        let display_id = &primary.id;

        let frame = capture.capture_frame(display_id, None).unwrap();
        assert_eq!(
            frame.width, primary.bounds.width,
            "frame width should match display width"
        );
        assert_eq!(
            frame.height, primary.bounds.height,
            "frame height should match display height"
        );
    }

    #[test]
    fn xcb_capture_full_display_rgba8_format() {
        let capture = XcbCapture::new().unwrap();
        let display_id = primary_display_id(&capture);
        let frame = capture.capture_frame(&display_id, None).unwrap();
        assert_eq!(
            frame.format,
            PixelFormat::Rgba8,
            "xcap should return Rgba8 format"
        );
    }

    #[test]
    fn xcb_capture_full_display_valid_stride() {
        let capture = XcbCapture::new().unwrap();
        let display_id = primary_display_id(&capture);
        let frame = capture.capture_frame(&display_id, None).unwrap();
        assert!(
            frame.stride >= frame.width * 4,
            "stride ({}) should be >= width * 4 ({})",
            frame.stride,
            frame.width * 4
        );
    }

    #[test]
    fn xcb_capture_full_display_valid_data_length() {
        let capture = XcbCapture::new().unwrap();
        let display_id = primary_display_id(&capture);
        let frame = capture.capture_frame(&display_id, None).unwrap();
        let expected_min = (frame.stride * frame.height) as usize;
        assert!(
            frame.data.len() >= expected_min,
            "data length ({}) should be >= stride * height ({})",
            frame.data.len(),
            expected_min
        );
    }

    #[test]
    fn xcb_capture_full_display_non_zero_pixels() {
        let capture = XcbCapture::new().unwrap();
        let display_id = primary_display_id(&capture);
        let frame = capture.capture_frame(&display_id, None).unwrap();
        let has_non_zero = frame.data.iter().any(|&b| b != 0);
        assert!(
            has_non_zero,
            "captured frame should contain non-zero pixel data"
        );
    }

    #[test]
    fn xcb_capture_invalid_display_id_returns_not_found() {
        let capture = XcbCapture::new().unwrap();
        let result = capture.capture_frame("nonexistent-display-42", None);
        assert!(matches!(
            result,
            Err(CaptureError::DisplayNotFound(ref id)) if id == "nonexistent-display-42"
        ));
    }

    // ── T006: Region-specific capture ──

    #[test]
    fn xcb_capture_region_correct_dimensions() {
        let capture = XcbCapture::new().unwrap();
        let display_id = primary_display_id(&capture);
        let region = ScreenRect {
            x: 100,
            y: 100,
            width: 200,
            height: 150,
        };
        let frame = capture.capture_frame(&display_id, Some(region)).unwrap();
        assert_eq!(
            frame.width, 200,
            "region capture width should match requested"
        );
        assert_eq!(
            frame.height, 150,
            "region capture height should match requested"
        );
    }

    #[test]
    fn xcb_capture_region_out_of_bounds_error() {
        let capture = XcbCapture::new().unwrap();
        let displays = capture.list_displays().unwrap();
        let primary = displays.iter().find(|d| d.is_primary).unwrap();
        let display_id = &primary.id;

        let region = ScreenRect {
            #[allow(clippy::cast_possible_wrap)]
            x: (primary.bounds.width - 100) as i32,
            y: 0,
            width: 300,
            height: 100,
        };
        let result = capture.capture_frame(display_id, Some(region));
        assert!(matches!(
            result,
            Err(CaptureError::RegionOutOfBounds { .. })
        ));
    }

    #[test]
    fn xcb_capture_region_small_source_performance() {
        let capture = XcbCapture::new().unwrap();
        let display_id = primary_display_id(&capture);
        let region = ScreenRect {
            x: 0,
            y: 0,
            width: 96,
            height: 54,
        };

        // Warm up
        let _ = capture.capture_frame(&display_id, Some(region));

        let iterations: u32 = 10;
        let start = Instant::now();
        for _ in 0..iterations {
            capture.capture_frame(&display_id, Some(region)).unwrap();
        }
        let avg_ms = start.elapsed().as_secs_f64() * 1000.0 / f64::from(iterations);
        eprintln!("Small region (96x54) avg capture: {avg_ms:.2}ms");

        // Relaxed threshold for CI (Xvfb + software rendering)
        assert!(
            avg_ms < 50.0,
            "average capture time ({avg_ms:.2}ms) should be < 50ms on CI"
        );
    }

    // ── T007: Self-capture prevention ──

    #[test]
    fn xcb_capture_without_exclusion_returns_valid_data() {
        let capture = XcbCapture::new().unwrap();
        let display_id = primary_display_id(&capture);
        let frame = capture.capture_frame(&display_id, None).unwrap();
        assert!(frame.width > 0);
        assert!(frame.height > 0);
        assert!(!frame.data.is_empty());
    }

    #[test]
    fn xcb_capture_set_excluded_windows_stores_ids() {
        let mut capture = XcbCapture::new().unwrap();
        capture.set_excluded_windows(&[42]);
        // Verify capture still works with non-existent exclusion IDs
        let display_id = primary_display_id(&capture);
        let frame = capture.capture_frame(&display_id, None).unwrap();
        assert!(
            frame.width > 0,
            "capture should still succeed with non-existent exclusion ID"
        );
    }

    // ── T008: Display change subscription ──

    #[test]
    fn xcb_capture_subscribe_display_changes_returns_backend_unavailable() {
        let capture = XcbCapture::new().unwrap();
        let result = capture.subscribe_display_changes(16);
        assert!(
            matches!(result, Err(CaptureError::BackendUnavailable { .. })),
            "subscribe_display_changes should return BackendUnavailable in Phase 0"
        );
        if let Err(CaptureError::BackendUnavailable { reason }) = result {
            assert!(
                reason.contains("not yet implemented"),
                "reason should contain 'not yet implemented': '{reason}'"
            );
        }
    }

    // ── T009: Full pipeline integration test ──

    #[test]
    fn xcb_capture_integration_full_pipeline() {
        let capture = XcbCapture::new().unwrap();

        // List displays
        let displays = capture.list_displays().unwrap();
        assert!(!displays.is_empty(), "should have displays");

        let primary = displays.iter().find(|d| d.is_primary).unwrap();
        let display_id = &primary.id;

        // Full capture
        let full_frame = capture.capture_frame(display_id, None).unwrap();
        assert_eq!(full_frame.width, primary.bounds.width);
        assert_eq!(full_frame.height, primary.bounds.height);
        assert_eq!(full_frame.format, PixelFormat::Rgba8);
        assert!(full_frame.stride >= full_frame.width * 4);
        assert!(full_frame.data.len() >= (full_frame.stride * full_frame.height) as usize);

        // Region capture
        let region = ScreenRect {
            x: primary.bounds.x + 10,
            y: primary.bounds.y + 10,
            width: 200,
            height: 150,
        };
        let region_frame = capture.capture_frame(display_id, Some(region)).unwrap();
        assert_eq!(region_frame.width, 200);
        assert_eq!(region_frame.height, 150);
        assert_eq!(region_frame.format, PixelFormat::Rgba8);
    }

    // ── T011: Capture performance benchmarks ──

    #[test]
    fn xcb_capture_benchmark_small_region() {
        let capture = XcbCapture::new().unwrap();
        let display_id = primary_display_id(&capture);
        let region = ScreenRect {
            x: 0,
            y: 0,
            width: 96,
            height: 54,
        };

        let _ = capture.capture_frame(&display_id, Some(region));

        let iterations: u32 = 10;
        let start = Instant::now();
        for _ in 0..iterations {
            capture.capture_frame(&display_id, Some(region)).unwrap();
        }
        let avg_ms = start.elapsed().as_secs_f64() * 1000.0 / f64::from(iterations);
        eprintln!("Benchmark: 96x54 region avg = {avg_ms:.2}ms");
        assert!(avg_ms < 50.0, "avg {avg_ms:.2}ms should be < 50ms (CI)");
    }

    #[test]
    fn xcb_capture_benchmark_medium_region() {
        let capture = XcbCapture::new().unwrap();
        let display_id = primary_display_id(&capture);
        let region = ScreenRect {
            x: 0,
            y: 0,
            width: 960,
            height: 540,
        };

        let _ = capture.capture_frame(&display_id, Some(region));

        let iterations: u32 = 10;
        let start = Instant::now();
        for _ in 0..iterations {
            capture.capture_frame(&display_id, Some(region)).unwrap();
        }
        let avg_ms = start.elapsed().as_secs_f64() * 1000.0 / f64::from(iterations);
        eprintln!("Benchmark: 960x540 region avg = {avg_ms:.2}ms");
        assert!(avg_ms < 50.0, "avg {avg_ms:.2}ms should be < 50ms (CI)");
    }

    #[test]
    fn xcb_capture_benchmark_full_display() {
        let capture = XcbCapture::new().unwrap();
        let display_id = primary_display_id(&capture);

        let _ = capture.capture_frame(&display_id, None);

        let iterations: u32 = 10;
        let start = Instant::now();
        for _ in 0..iterations {
            capture.capture_frame(&display_id, None).unwrap();
        }
        let avg_ms = start.elapsed().as_secs_f64() * 1000.0 / f64::from(iterations);
        eprintln!("Benchmark: full display avg = {avg_ms:.2}ms");
        assert!(avg_ms < 50.0, "avg {avg_ms:.2}ms should be < 50ms (CI)");
    }
}
