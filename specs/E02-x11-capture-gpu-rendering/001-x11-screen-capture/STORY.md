# Story E02/001: X11 Screen Capture Backend

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DONE
**Depends On:** None (can start immediately; soft dependency on E02/002 for overlay window ID used in self-capture prevention)

---

## Problem Statement

The magnification pipeline cannot begin without a source of screen pixels. The `ScreenCapture` trait was defined in E01 with three methods (`list_displays`, `capture_frame`, `subscribe_display_changes`), but no real platform backend exists -- only a mock implementation that returns synthetic test data. Until a working X11 backend captures actual screen content and delivers it as `CaptureFrame` values, the GPU rendering pipeline (Stories 003-005) has no real data to process.

This story implements `XcbCapture`, the first real `ScreenCapture` backend, targeting Linux X11 via the `xcap` crate. It delivers display enumeration, full-screen and region-specific capture in `Rgba8` pixel format (xcap v0.9 internally converts X11's native BGRA to RGBA), and self-capture prevention (RISK-002) by excluding the magnification overlay window from captured frames. The implementation runs on Xvfb in CI, enabling automated testing without a physical display.

## User Scenarios

### US-1: Display Enumeration

As a low-vision user launching Luminos on a Linux X11 desktop, I want the application to detect my connected display(s) so that it knows which screen to magnify.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given a Linux X11 environment with at least one connected display, when `XcbCapture::list_displays()` is called, then it returns a non-empty `Vec<DisplayInfo>` where each entry has a non-empty `id`, a non-empty `name`, `bounds` with `width > 0` and `height > 0`, and `scale_factor > 0.0`.
- **AC-1.2:** Given a Linux X11 environment with one primary display, when `list_displays()` is called, then exactly one entry in the returned vector has `is_primary == true`.
- **AC-1.3:** Given a display with known resolution (e.g., Xvfb at 1920x1080), when `list_displays()` is called, then the primary display's `bounds.width` and `bounds.height` match the known resolution.

### US-2: Full-Display Capture

As the rendering pipeline, I want to capture the entire content of a display as a `CaptureFrame` so that I can upload it to a GPU texture for magnification.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given a valid display ID from `list_displays()`, when `capture_frame(display_id, None)` is called, then it returns `Ok(CaptureFrame)` with `width` and `height` matching the display's `bounds.width` and `bounds.height`, `format == PixelFormat::Rgba8` (xcap converts X11's native BGRA to RGBA internally), `stride >= width * 4`, and `data.len() >= (stride * height) as usize`.
- **AC-2.2:** Given a valid display ID, when `capture_frame(display_id, None)` is called on an Xvfb display with a non-black background, then the returned `CaptureFrame::data` contains non-zero pixel values (the capture is not blank).
- **AC-2.3:** Given an invalid display ID (e.g., `"nonexistent-0"`), when `capture_frame(display_id, None)` is called, then it returns `Err(CaptureError::DisplayNotFound("nonexistent-0"))`.

### US-3: Region Capture

As the rendering pipeline, I want to capture only a specific rectangular region of the display (the viewport source region) so that high-zoom captures are fast and bandwidth-efficient.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given a valid display ID and a region `ScreenRect { x: 100, y: 100, width: 200, height: 150 }` within display bounds, when `capture_frame(display_id, Some(region))` is called, then it returns `Ok(CaptureFrame)` with `width == 200` and `height == 150`.
- **AC-3.2:** Given a region that exceeds the display bounds (e.g., `x: 1800, width: 300` on a 1920-wide display), when `capture_frame(display_id, Some(region))` is called, then it returns `Err(CaptureError::RegionOutOfBounds { region, bounds })` where `bounds` matches the display's actual bounds.
- **AC-3.3:** Given a valid display and a small region (e.g., 96x54 pixels simulating 20x zoom on 1080p), when `capture_frame()` is called, then it completes in under 8ms (the per-stage capture budget from doc-03 Section 2.3).

### US-4: Self-Capture Prevention (RISK-002)

As a low-vision user, I want the magnified overlay to NOT capture itself so that I see real screen content instead of an infinite feedback loop of magnified views.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given an `XcbCapture` instance with an overlay window ID configured via `set_excluded_windows(&[overlay_window_id as u64])`, when `capture_frame()` is called, then the returned `CaptureFrame` does not contain pixels from the overlay window.
- **AC-4.2:** Given an overlay window filled with a known solid color (e.g., `#FF00FF` magenta) and exclusion configured via `set_excluded_windows()`, when a frame is captured, then the captured frame's pixel data does NOT contain the magenta color at the overlay window's position.
- **AC-4.3:** Given an `XcbCapture` instance with no excluded windows set (default after `XcbCapture::new()`), when `capture_frame()` is called, then it still returns valid capture data (self-capture prevention is optional, not required for basic capture).
- **AC-4.4:** Given the `ScreenCapture` trait, when inspected, then it contains a `set_excluded_windows(&mut self, window_ids: &[u64])` method with a default no-op implementation, so that existing trait implementations continue to compile without modification.
- **AC-4.5:** Given a `MockScreenCapture` instance, when `set_excluded_windows(&mut self, &[42, 99])` is called and then the excluded window IDs are inspected, then the mock stores the provided IDs (verifying the trait method is callable on mock implementations).

### US-5: Display Change Notifications

As the rendering pipeline, I want to be notified when displays are connected, disconnected, or reconfigured so that the magnification viewport can adapt to display changes.

**Priority:** P1
**Acceptance Criteria:**

- **AC-5.1:** Given an `XcbCapture` instance, when `subscribe_display_changes(16)` is called, then it returns `Ok(Receiver<DisplayChangeEvent>)` (the subscription is established).
- **AC-5.2:** Given a platform where display change events are not available (e.g., Xvfb without RandR support), when `subscribe_display_changes()` is called, then it returns `Err(CaptureError::BackendUnavailable { reason })` with a descriptive reason, enabling graceful fallback to polling `list_displays()`.

### US-6: Error Handling and Graceful Degradation

As the rendering pipeline, I want capture errors to be well-typed and recoverable so that transient failures do not crash the application.

**Priority:** P0
**Acceptance Criteria:**

- **AC-6.1:** Given a `CaptureError::Platform` variant, when formatted with `Display`, then the message includes both the high-level context and the platform-specific detail (e.g., `"platform capture error: XCB connection failed"`).
- **AC-6.2:** Given a `CaptureError`, when propagated via `?` to a `LuminosError`, then the conversion succeeds via the existing `From<CaptureError>` implementation from E01.

## Functional Requirements

- **FR-1:** Implement the `XcbCapture` struct in `crates/luminos-platform/src/linux_x11/capture.rs` that implements the `ScreenCapture` trait. *(Traced by US-1, US-2, US-3, US-4, US-5)*
- **FR-2:** Implement `XcbCapture::new() -> Result<Self, CaptureError>` constructor that initializes the xcap backend. Self-capture exclusion is configured separately via the `set_excluded_windows()` trait method (FR-11). *(Traced by AC-4.3)*
- **FR-3:** Implement `list_displays()` returning `Vec<DisplayInfo>` populated from X11 screen information via xcap. Map xcap monitor data to `DisplayInfo` fields. *(Traced by AC-1.1, AC-1.2, AC-1.3)*
- **FR-4:** Implement `capture_frame()` supporting both full-display (`region: None`) and region-specific (`region: Some(ScreenRect)`) capture. Output pixel format is `PixelFormat::Rgba8` (xcap v0.9 returns RGBA despite X11's native BGRA format). *(Traced by AC-2.1, AC-3.1)*
- **FR-5:** Validate capture region bounds before capture. Return `CaptureError::RegionOutOfBounds` if the requested region exceeds display bounds. *(Traced by AC-3.2)*
- **FR-6:** Validate display ID before capture. Return `CaptureError::DisplayNotFound` if the display ID does not match any connected display. *(Traced by AC-2.3)*
- **FR-7:** Implement self-capture prevention: when overlay window IDs are set via `set_excluded_windows()`, exclude those windows from captured frames. Primary mechanism: unmap/remap cycle (unmap the excluded windows before capture, capture, then remap immediately). This approach is simpler and more reliable than composite pixmap capture. *(Traced by AC-4.1, AC-4.2)*
- **FR-8:** Implement `subscribe_display_changes()`. For Phase 0, return `CaptureError::BackendUnavailable` with a descriptive reason (RandR event monitoring not yet implemented). The caller falls back to periodic `list_displays()` polling. Full RandR event monitoring via `x11rb` is deferred to a future iteration. *(Traced by AC-5.2)*
- **FR-9:** Verify that the `ci_platform_tests` feature flag exists in `crates/luminos-platform/Cargo.toml` (added in E01) for gating integration tests that require X11. *(Traced by integration test infrastructure)*
- **FR-10:** Update `crates/luminos-platform/src/linux_x11/mod.rs` to export `XcbCapture` publicly within the crate. *(Traced by module structure)*
- **FR-11:** Modify the `ScreenCapture` trait in `crates/luminos-platform/src/traits/screen_capture.rs` (E01 breaking change) to add `set_excluded_windows(&mut self, window_ids: &[u64])` with a default no-op implementation. Update `MockScreenCapture` to store excluded window IDs. The `XcbCapture` implementation stores the IDs and uses them for the unmap/remap self-capture exclusion cycle during `capture_frame()`. The `u64` type accommodates X11 window IDs (u32) and Windows HWND values across platforms. *(Traced by AC-4.1, AC-4.4, AC-4.5)*

## Non-Functional Requirements

- **NFR-1:** `capture_frame()` must complete in under 8ms for typical magnification source regions (up to 1280x720 at 1.5x zoom on 1080p) per doc-03 Section 2.3. *(Traced by AC-3.3)*
- **NFR-2:** No `unwrap()` or `expect()` in production code paths. All error handling via `?` propagation and `From` conversions. `unwrap()` is acceptable in `#[cfg(test)]` blocks.
- **NFR-3:** `CaptureFrame` debug output must not include pixel data (RISK-017 mitigation, enforced by E01's custom `Debug` impl).
- **NFR-4:** All public items must have `///` doc-comments. `cargo doc -p luminos-platform --no-deps` must produce documentation without warnings.
- **NFR-5:** `cargo clippy -p luminos-platform -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` must pass with zero warnings.
- **NFR-6:** Integration tests must be gated behind `#[cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]` so they do not run on non-Linux platforms or during default `cargo nextest run`.

## Out of Scope

- XShm (shared memory) capture optimization (Phase 1, E08 or later).
- Linux Wayland capture via PipeWire (separate story in a future epic).
- macOS, OpenBSD, or Windows capture backends (future epics).
- Multi-display capture coordination (single primary display only in E02; multi-display is E05+).
- HiDPI scaling logic beyond reporting `scale_factor` in `DisplayInfo` (coordinate scaling is a rendering concern in Story 004).
- GPU texture upload from `CaptureFrame` (Story 003).
- Render loop integration (Story 005).

## Open Questions

*None -- all questions resolved during epic planning. Self-capture prevention mechanism confirmed as unmap/remap cycle approach per researcher investigation of xcap and X11 APIs. The risk register's composite pixmap mitigation was superseded based on practical testing.*
