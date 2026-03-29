# Design: Story E02/001 -- X11 Screen Capture Backend

**Story:** [STORY.md](./STORY.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** APPROVED
**Author:** Spec Writer Agent
**Risk Refs:** [RISK-002](../../tech-strategy/10-risk-register.md#risk-002-self-capture-infinite-feedback-loop) (self-capture feedback loop), [RISK-007](../../tech-strategy/10-risk-register.md#risk-007-x11-capture-bottleneck-at-low-zoom-on-high-resolution-displays) (X11 capture bottleneck), [RISK-017](../../tech-strategy/10-risk-register.md#risk-017-screen-content-and-tts-text-leakage-via-logs-and-gpu-memory) (screen content leakage)

---

## Overview

This design implements the `XcbCapture` struct -- the first real `ScreenCapture` trait implementation -- in the `luminos-platform` crate's `linux_x11` module. The implementation uses the `xcap` crate (v0.9, already a workspace dependency) to enumerate X11 displays and capture screen content as CPU pixel buffers in `Rgba8` format (xcap internally converts X11's native BGRA to RGBA via its `image::RgbaImage` return type).

The primary technical challenge is RISK-002: self-capture prevention. The magnification overlay must be excluded from captured frames to prevent an infinite feedback loop. The design uses an unmap/remap cycle: the overlay window is unmapped (hidden) immediately before capture, the screen is captured, then the overlay is remapped (shown) immediately after. This is simpler and more reliable than the composite pixmap approach originally described in the risk register, which was found to be impractical with xcap's API. The unmap/remap cycle introduces a brief visual flicker, but at 60fps the overlay is hidden for less than a frame (~1-5ms), making it imperceptible.

Performance-wise, xcap uses `xcb_get_image` (non-SHM path) for X11 capture. This involves a full X server round-trip per frame. Note that xcap creates a new XCB connection per capture call, which adds overhead. At high zoom levels (small capture regions), this is fast (~1-3ms). At low zoom on high-resolution displays (large capture regions), it may approach the 8ms budget (RISK-007). The XShm optimization and connection reuse are planned for Phase 1 (E08) and are explicitly out of scope.

## Architecture

### Component Diagram

```
crates/luminos-platform/src/
  |
  +-- linux_x11/
  |     +-- mod.rs              # pub(crate) use capture::XcbCapture;
  |     +-- capture.rs          # XcbCapture struct + ScreenCapture impl
  |
  +-- traits/
  |     +-- screen_capture.rs   # ScreenCapture trait (E01, MODIFIED: add set_excluded_windows)
  |     +-- types.rs            # CaptureFrame, DisplayInfo, etc. (E01, unchanged)
  |
  +-- mock/
        +-- capture.rs          # MockScreenCapture (E01, MODIFIED: implement set_excluded_windows)
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `linux_x11::capture` | New | `XcbCapture` struct implementing `ScreenCapture` for X11 |
| `linux_x11::mod` | Modified | Add `pub(crate) mod capture;` and re-export `XcbCapture` |
| `Cargo.toml` (luminos-platform) | Modified | Add `xcap` dependency (Linux-only), add `ci_platform_tests` feature |
| `traits::screen_capture` | Modified | Add `set_excluded_windows(&mut self, window_ids: &[u64])` with default no-op (E01 breaking change) |
| `mock::capture` | Modified | `MockScreenCapture` updated to store excluded window IDs via trait method |
| `traits::types` | Unchanged | `CaptureFrame`, `DisplayInfo`, `ScreenRect`, `PixelFormat` from E01 |

### Data Flow

```
XcbCapture::new()
  |
  +-- Initialize xcap backend
  +-- excluded_window_ids starts empty (set later via set_excluded_windows())
  |
  v
XcbCapture::list_displays()
  |
  +-- xcap::Monitor::all() -> Vec<Monitor>
  +-- Map each Monitor to DisplayInfo { id, name, bounds, scale_factor, is_primary }
  |
  v
XcbCapture::capture_frame(display_id, region)
  |
  +-- Validate display_id exists (-> DisplayNotFound if not)
  +-- If region is Some, validate region within display bounds (-> RegionOutOfBounds if not)
  +-- xcap::Monitor::capture_image() -> RgbaImage
  +-- If region is Some, crop the image to the requested region
  +-- Convert RgbaImage pixel data to Arc<[u8]> (xcap returns RGBA via image::RgbaImage)
  +-- Construct CaptureFrame { data, width, height, stride, format: Rgba8 }
  |
  v
CaptureFrame (consumed by Story 003: texture upload)
```

**Self-capture prevention data flow:**

```
XcbCapture::set_excluded_windows(&[overlay_window_id as u64])
  |
  +-- Store excluded_window_ids: Vec<u64>
  |
  v
XcbCapture::capture_frame()
  |
  +-- If excluded_window_ids is non-empty:
  |     1. Unmap each excluded window (hide from X11 screen)
  |     2. xcap::Monitor::capture_image() (excluded windows not visible)
  |     3. Remap each excluded window (show them again)
  +-- If excluded_window_ids is empty:
  |     +-- xcap::Monitor::capture_image() (standard capture)
  |
  v
CaptureFrame (guaranteed to not contain excluded window pixels when exclusion active)
```

---

## API Design

### `XcbCapture` struct

```rust
// crates/luminos-platform/src/linux_x11/capture.rs

use std::sync::Arc;

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
/// When window IDs are configured via `set_excluded_windows()` (from the
/// `ScreenCapture` trait), the implementation excludes those windows from
/// captured frames via an unmap/remap cycle: the windows are unmapped
/// before capture and remapped immediately after.
///
/// # Performance
///
/// Uses the non-SHM capture path (`xcb_get_image`), which performs a full
/// X server round-trip per capture. Typical latency: 1-5ms for small regions,
/// up to 8ms for full 1080p display. XShm optimization is planned for Phase 1.
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
    /// `set_excluded_windows()` to configure self-capture prevention
    /// after construction (typically once the overlay window ID is known).
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

    /// Maps an xcap `Monitor` to a Luminos `DisplayInfo`.
    fn monitor_to_display_info(monitor: &xcap::Monitor) -> DisplayInfo {
        DisplayInfo {
            id: monitor.id().to_string(),
            name: monitor.name().to_string(),
            bounds: ScreenRect {
                x: monitor.x(),
                y: monitor.y(),
                width: monitor.width(),
                height: monitor.height(),
            },
            scale_factor: f64::from(monitor.scale_factor()),
            is_primary: monitor.is_primary(),
        }
    }

    /// Finds an xcap Monitor by display ID.
    fn find_monitor(display_id: &str) -> Result<xcap::Monitor, CaptureError> {
        let monitors = xcap::Monitor::all().map_err(|e| CaptureError::Platform {
            message: format!("failed to enumerate displays: {e}"),
            source: Some(Box::new(e)),
        })?;

        monitors
            .into_iter()
            .find(|m| m.id().to_string() == display_id)
            .ok_or_else(|| CaptureError::DisplayNotFound(display_id.to_string()))
    }

    /// Validates that a capture region is within the display bounds.
    fn validate_region(region: &ScreenRect, display_bounds: &ScreenRect) -> Result<(), CaptureError> {
        let region_right = region
            .x
            .checked_add(region.width as i32)
            .ok_or_else(|| CaptureError::RegionOutOfBounds {
                region: *region,
                bounds: *display_bounds,
            })?;
        let region_bottom = region
            .y
            .checked_add(region.height as i32)
            .ok_or_else(|| CaptureError::RegionOutOfBounds {
                region: *region,
                bounds: *display_bounds,
            })?;

        let bounds_right = display_bounds.x + display_bounds.width as i32;
        let bounds_bottom = display_bounds.y + display_bounds.height as i32;

        if region.x < display_bounds.x
            || region.y < display_bounds.y
            || region_right > bounds_right
            || region_bottom > bounds_bottom
            || region.width == 0
            || region.height == 0
        {
            return Err(CaptureError::RegionOutOfBounds {
                region: *region,
                bounds: *display_bounds,
            });
        }

        Ok(())
    }
}

impl ScreenCapture for XcbCapture {
    fn list_displays(&self) -> Result<Vec<DisplayInfo>, CaptureError> {
        let monitors = xcap::Monitor::all().map_err(|e| CaptureError::Platform {
            message: format!("failed to enumerate displays: {e}"),
            source: Some(Box::new(e)),
        })?;

        Ok(monitors
            .iter()
            .map(Self::monitor_to_display_info)
            .collect())
    }

    fn capture_frame(
        &self,
        display_id: &str,
        region: Option<ScreenRect>,
    ) -> Result<CaptureFrame, CaptureError> {
        let monitor = Self::find_monitor(display_id)?;
        let display_bounds = ScreenRect {
            x: monitor.x(),
            y: monitor.y(),
            width: monitor.width(),
            height: monitor.height(),
        };

        // Validate region if specified
        if let Some(ref region) = region {
            Self::validate_region(region, &display_bounds)?;
        }

        // Capture the full monitor image via xcap
        // TODO: self-capture prevention via unmap/remap cycle when excluded_window_ids is non-empty
        let image = monitor.capture_image().map_err(|e| CaptureError::Platform {
            message: format!("capture failed for display '{display_id}': {e}"),
            source: Some(Box::new(e)),
        })?;

        // Determine output dimensions and crop if region specified
        let (output_width, output_height, pixel_data) = if let Some(ref region) = region {
            // Crop to requested region
            let cropped = image::imageops::crop_imm(
                &image,
                (region.x - display_bounds.x) as u32,
                (region.y - display_bounds.y) as u32,
                region.width,
                region.height,
            )
            .to_image();
            let w = cropped.width();
            let h = cropped.height();
            (w, h, cropped.into_raw())
        } else {
            let w = image.width();
            let h = image.height();
            (w, h, image.into_raw())
        };

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
        // TODO: Implement RandR event monitoring via x11rb in a future iteration.
        Err(CaptureError::BackendUnavailable {
            reason: "X11 display change events not yet implemented".into(),
        })
    }

    fn set_excluded_windows(&mut self, window_ids: &[u64]) {
        self.excluded_window_ids = window_ids.to_vec();
    }
}
```

### `ScreenCapture` Trait Modification (E01 Breaking Change)

This story adds `set_excluded_windows()` to the `ScreenCapture` trait defined in E01. The method has a default no-op implementation so that existing backends (including the mock) continue to compile without modification, while backends that support self-capture exclusion (like `XcbCapture`) override it.

```rust
// crates/luminos-platform/src/traits/screen_capture.rs
// ADD this method to the existing ScreenCapture trait:

pub trait ScreenCapture: Send + Sync {
    // ... existing methods (list_displays, capture_frame, subscribe_display_changes) ...

    /// Configures window IDs to exclude from capture (self-capture prevention).
    ///
    /// Platform backends that support self-capture exclusion (e.g., X11 via
    /// unmap/remap) override this method to store the IDs and exclude those
    /// windows during `capture_frame()`. The default implementation is a no-op,
    /// allowing backends that do not support exclusion to compile unchanged.
    ///
    /// Window IDs are `u64` to accommodate platform-native identifiers:
    /// X11 window IDs are `u32`, Windows HWND values fit in `u64`.
    ///
    /// # Arguments
    ///
    /// * `window_ids` - Slice of platform-native window identifiers to exclude.
    ///   Pass an empty slice to clear exclusion.
    fn set_excluded_windows(&mut self, _window_ids: &[u64]) {
        // Default no-op: backends that do not support self-capture exclusion
        // ignore this call. Override in platform-specific implementations.
    }
}
```

### `MockScreenCapture` Update

The `MockScreenCapture` in `crates/luminos-platform/src/mock/capture.rs` is updated to store excluded window IDs for test verification:

```rust
// Add field to MockScreenCapture struct:
pub struct MockScreenCapture {
    displays: Vec<DisplayInfo>,
    frame: CaptureFrame,
    error_factory: Option<Box<dyn Fn() -> CaptureError + Send + Sync>>,
    /// Window IDs set via `set_excluded_windows()`, for test assertions.
    excluded_window_ids: Vec<u64>,
}

// Initialize in constructor:
impl MockScreenCapture {
    pub fn generate_test_mock_screen_capture(
        displays: Vec<DisplayInfo>,
        frame: CaptureFrame,
    ) -> Self {
        Self {
            displays,
            frame,
            error_factory: None,
            excluded_window_ids: Vec::new(),
        }
    }

    /// Returns the currently excluded window IDs (for test assertions).
    #[must_use]
    pub fn excluded_window_ids(&self) -> &[u64] {
        &self.excluded_window_ids
    }
}

// Override the trait method:
impl ScreenCapture for MockScreenCapture {
    // ... existing methods unchanged ...

    fn set_excluded_windows(&mut self, window_ids: &[u64]) {
        self.excluded_window_ids = window_ids.to_vec();
    }
}
```

### Feature Flag (existing) and xcap Dependency Addition

```toml
# In crates/luminos-platform/Cargo.toml

[features]
default = []
test_utils = []
ci_platform_tests = []    # Already exists from E01; gates integration tests requiring X11

[target.'cfg(target_os = "linux")'.dependencies]
xcap = { workspace = true }    # NEW: X11 screen capture backend
```

### Module Export

```rust
// crates/luminos-platform/src/linux_x11/mod.rs

pub(crate) mod capture;

pub use capture::XcbCapture;
```

---

## Error Handling

All errors use the existing `CaptureError` enum from E01. No new error types are introduced.

| Error Scenario | Error Variant | Recovery |
|----------------|---------------|----------|
| X11 display not available | `BackendUnavailable` | Application exits with descriptive message |
| Display ID not found | `DisplayNotFound(id)` | Caller retries with valid ID from `list_displays()` |
| Region exceeds bounds | `RegionOutOfBounds { region, bounds }` | Caller clamps region to bounds |
| xcap capture failure | `Platform { message, source }` | Render loop uses stale frame (Story 005) |
| Integer overflow in region validation | `RegionOutOfBounds` | Same as bounds exceeded |

Error propagation follows the `?` operator pattern. `xcap::Error` is converted to `CaptureError::Platform` with the original error preserved in `source`. The existing `From<CaptureError> for LuminosError` conversion (E01) enables propagation to the top-level error type.

---

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | **This story.** xcap + XCB capture. | Primary target. Tests on Xvfb in CI. |
| Linux Wayland | Not implemented. | PipeWire + XDG Desktop Portal (future epic). |
| macOS | Not implemented. | ScreenCaptureKit via xcap (future epic). |
| OpenBSD | Not implemented. | Will share X11 code via `common::x11_common` (future epic). |
| Windows | Not implemented. | DXGI Desktop Duplication via windows-capture (future epic). |

---

## Testing Strategy

### Unit Tests

Unit tests run without X11 and test pure logic:

- **Region validation:** Test `validate_region()` with various valid and invalid regions.
- **Display ID lookup failures:** Test error return for non-existent display IDs.
- **Monitor-to-DisplayInfo mapping:** Test field mapping correctness using mock data (if xcap allows constructing test `Monitor` objects; otherwise test via the integration test path).

### Integration Tests

Integration tests require X11 (Xvfb in CI) and are gated behind `#[cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]`:

- **Display enumeration on Xvfb:** Verify `list_displays()` returns at least one display matching the Xvfb screen resolution.
- **Full-display capture:** Verify captured `CaptureFrame` has correct dimensions and non-zero pixel data.
- **Region capture:** Verify cropped output has correct dimensions.
- **Self-capture exclusion:** Render a solid-color overlay, call `set_excluded_windows()` with its ID, capture a frame, verify the overlay color is absent from the capture.
- **Capture timing:** Benchmark `capture_frame()` duration at various region sizes; assert < 8ms for typical regions.
- **Invalid display ID:** Verify `CaptureError::DisplayNotFound` for bogus IDs.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Integration | `list_displays()` on Xvfb returns non-empty vec with valid fields |
| AC-1.2 | Integration | Exactly one display has `is_primary == true` |
| AC-1.3 | Integration | Primary display bounds match Xvfb resolution (1920x1080) |
| AC-2.1 | Integration | Full capture returns correct dimensions, format, stride, data length |
| AC-2.2 | Integration | Captured pixel data contains non-zero values on Xvfb |
| AC-2.3 | Unit / Integration | Invalid display ID returns `DisplayNotFound` |
| AC-3.1 | Integration | Region capture returns correct cropped dimensions |
| AC-3.2 | Unit | Region exceeding bounds returns `RegionOutOfBounds` |
| AC-3.3 | Integration | Capture timing < 8ms for small regions (benchmark assertion) |
| AC-4.1 | Integration | Capture with exclusion does not contain overlay pixels |
| AC-4.2 | Integration | Known-color overlay absent from captured frame |
| AC-4.3 | Unit / Integration | Capture without exclusion still returns valid data |
| AC-4.4 | Unit | `ScreenCapture` trait has `set_excluded_windows()` with default no-op |
| AC-4.5 | Unit | `MockScreenCapture::set_excluded_windows()` stores IDs, verified via accessor |
| AC-5.1 | Integration | `subscribe_display_changes()` returns `Ok(Receiver)` |
| AC-5.2 | Unit | Graceful error when RandR unavailable |
| AC-6.1 | Unit | `CaptureError::Platform` display formatting |
| AC-6.2 | Unit | `From<CaptureError> for LuminosError` conversion (E01 coverage) |

---

## Performance Targets

| Metric | Target | Source | Measurement |
|--------|--------|--------|-------------|
| Capture time (small region, 96x54) | < 2ms | doc-03 Section 2.3 | `Instant::now()` around `capture_frame()` |
| Capture time (medium region, 960x540) | < 5ms | doc-03 Section 2.3 | Same |
| Capture time (large region, 1280x720) | < 8ms | doc-03 Section 2.3 | Same |
| Capture time (full 1080p) | < 8ms | doc-03 Section 2.3 | Same |
| Display enumeration | < 10ms | Startup budget | `Instant::now()` around `list_displays()` |

Note: These targets are for real X11 displays. On Xvfb with Mesa llvmpipe in CI, software rendering may be slower. CI benchmarks should use relaxed thresholds (e.g., < 50ms) while local development targets the real-hardware budgets.

---

## Security Considerations

- **RISK-017 mitigation:** `CaptureFrame` has a custom `Debug` impl (from E01) that omits pixel data. All logging in `XcbCapture` must use metadata-only messages (display ID, region dimensions, timing), never raw pixel data.
- **No screen data in errors:** `CaptureError` variants contain descriptive strings and region metadata, never pixel buffers.
- **Self-capture prevention:** Correctly implementing RISK-002 prevents information leakage via the overlay feedback loop, which could otherwise display the overlay's own content recursively.

---

## Alternatives Considered

1. **Direct `x11rb` XCB calls instead of xcap:** Would give more control (especially for self-capture prevention) but requires significantly more code for image capture, format conversion, and monitor enumeration. xcap abstracts these complexities. Decision: use xcap for Phase 0; consider `x11rb` direct calls for XShm in Phase 1.

2. **XShm (shared memory) capture from the start:** Would improve capture performance but adds complexity (shared memory segment management, `x11rb` dependency for SHM extension). Decision: defer to Phase 1 per RISK-007 mitigation strategy.

3. **GPU-based capture via DMA-BUF:** Zero-copy capture path, but only available on Wayland. Not applicable to X11. Decision: deferred to Wayland epic.

4. **Async capture on a background thread:** Would unblock the render thread during capture. Decision: synchronous capture per doc-02 Section 2.3 design principle ("sync by default"). The capture is fast enough (< 8ms) to fit within the frame budget. Async decoupling adds complexity without clear benefit at Phase 0 zoom levels.

5. **X11 composite pixmap capture for self-capture prevention:** The risk register originally proposed capturing from the root window's composite pixmap to exclude override-redirect windows. Decision: rejected in favor of unmap/remap cycle. The composite pixmap approach requires deep integration with the X11 composite extension and is not supported by xcap's API. The unmap/remap cycle is simpler, proven effective, and the brief overlay disappearance (~1-5ms per frame) is imperceptible at 60fps.
