# Subtasks: Story E02/001 -- X11 Screen Capture Backend

**Status:** NOT STARTED
**Started:** ---
**Completed:** ---
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 3 | 0 | 0 | 3 |
| 2. Core Implementation | 6 | 0 | 0 | 6 |
| 3. Integration | 4 | 0 | 0 | 4 |
| 4. Polish & Acceptance | 2 | 0 | 0 | 2 |
| **Total** | **15** | **0** | **0** | **15** |

---

## Phase 1: Setup

### T000 -- Modify ScreenCapture trait and update MockScreenCapture (E01 breaking change)

**Traces to:** FR-11, AC-4.4, AC-4.5
**Status:** TODO
**Files:** `crates/luminos-platform/src/traits/screen_capture.rs`, `crates/luminos-platform/src/mock/capture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `screen_capture_trait_set_excluded_windows_default_noop` -- Verify the default implementation of `set_excluded_windows()` compiles and is callable without panicking (use a minimal test struct implementing `ScreenCapture` with only required methods)
   - [ ] `mock_screen_capture_set_excluded_windows_stores_ids` -- Verify `MockScreenCapture::set_excluded_windows(&[42, 99])` stores the IDs and `excluded_window_ids()` returns `&[42, 99]`
   - [ ] `mock_screen_capture_set_excluded_windows_clear` -- Verify `set_excluded_windows(&[])` clears previously stored IDs
2. **Green** -- Implement:
   - [ ] Add `set_excluded_windows(&mut self, _window_ids: &[u64])` method with default no-op body to the `ScreenCapture` trait in `screen_capture.rs`
   - [ ] Add `excluded_window_ids: Vec<u64>` field to `MockScreenCapture` struct
   - [ ] Initialize `excluded_window_ids: Vec::new()` in `generate_test_mock_screen_capture()`
   - [ ] Override `set_excluded_windows()` in `MockScreenCapture`'s `ScreenCapture` impl to store the IDs
   - [ ] Add `pub fn excluded_window_ids(&self) -> &[u64]` accessor to `MockScreenCapture` for test assertions
3. **Refactor** -- Clean up:
   - [ ] Add `///` doc-comments to the new trait method and mock accessor
   - [ ] Verify `cargo build --workspace` compiles (this is a breaking change to an E01 trait)

**Completion Notes:**
>

---

### T001 -- Add xcap dependency and verify ci_platform_tests feature flag

**Traces to:** FR-9, FR-10
**Status:** TODO
**Files:** `crates/luminos-platform/Cargo.toml`, `crates/luminos-platform/src/linux_x11/mod.rs`

**Steps:**
1. Verify that `ci_platform_tests = []` already exists under `[features]` in `crates/luminos-platform/Cargo.toml` (added in E01).
2. In `crates/luminos-platform/Cargo.toml`:
   - Add platform-gated xcap dependency: `[target.'cfg(target_os = "linux")'.dependencies]` section with `xcap = { workspace = true }`
   - Add `image` as a workspace dependency if not already present (xcap returns `image::RgbaImage`, and we need `image::imageops::crop_imm` for region capture)
3. In workspace root `Cargo.toml`, add `image` to `[workspace.dependencies]` if not already present: `image = "0.25"`
4. Verify `cargo build -p luminos-platform` compiles on Linux

**Completion Notes:**
>

---

### T002 -- Create capture module structure

**Traces to:** FR-1, FR-10
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/mod.rs`, `crates/luminos-platform/src/linux_x11/capture.rs`

**Steps:**
1. Update `crates/luminos-platform/src/linux_x11/mod.rs` to declare `pub(crate) mod capture;` and add `pub use capture::XcbCapture;`
2. Create `crates/luminos-platform/src/linux_x11/capture.rs` with the `XcbCapture` struct skeleton (fields only, no trait impl yet):
   ```rust
   pub struct XcbCapture {
       excluded_window_ids: Vec<u64>,
   }
   ```
3. Add `use` imports for all required types from `traits::screen_capture` and `traits::types`
4. Verify `cargo build -p luminos-platform` compiles

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 1, verify:
- [ ] `ScreenCapture` trait has `set_excluded_windows()` method with default no-op
- [ ] `MockScreenCapture` stores excluded window IDs and passes new tests
- [ ] `cargo build --workspace` compiles (E01 trait modification is not breaking)
- [ ] `cargo build -p luminos-platform` compiles with xcap dependency
- [ ] `ci_platform_tests` feature flag exists in Cargo.toml
- [ ] `XcbCapture` struct exists in `linux_x11/capture.rs`

---

## Phase 2: Core Implementation

### T003 -- Implement XcbCapture constructor and display enumeration

**Traces to:** FR-2, FR-3, AC-1.1, AC-1.2, AC-1.3
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/capture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `xcb_capture_new_succeeds` -- Verify `XcbCapture::new()` returns `Ok` on X11 (integration test, gated)
   - [ ] `xcb_capture_list_displays_returns_non_empty` -- Verify `list_displays()` returns at least one display on Xvfb (integration test, gated)
   - [ ] `xcb_capture_list_displays_has_primary` -- Verify exactly one display has `is_primary == true` (integration test, gated)
   - [ ] `xcb_capture_list_displays_valid_fields` -- Verify each `DisplayInfo` has non-empty `id`, non-empty `name`, `width > 0`, `height > 0`, `scale_factor > 0.0` (integration test, gated)
2. **Green** -- Implement:
   - [ ] Implement `XcbCapture::new()` constructor (no parameters) with xcap availability check, initializing `excluded_window_ids: Vec::new()`
   - [ ] Implement `monitor_to_display_info()` helper mapping xcap `Monitor` to `DisplayInfo`
   - [ ] Implement `ScreenCapture::list_displays()` via `xcap::Monitor::all()`
   - [ ] Implement `find_monitor()` helper for display ID lookup
3. **Refactor** -- Clean up:
   - [ ] Extract common xcap error mapping to a helper function if pattern repeats

**Completion Notes:**
>

---

### T004 -- Implement region validation

**Traces to:** FR-5, AC-3.2
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/capture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `xcb_capture_validate_region_within_bounds` -- Verify `validate_region()` returns `Ok` for a region fully within bounds
   - [ ] `xcb_capture_validate_region_exceeds_right` -- Verify `RegionOutOfBounds` when region extends past right edge
   - [ ] `xcb_capture_validate_region_exceeds_bottom` -- Verify `RegionOutOfBounds` when region extends past bottom edge
   - [ ] `xcb_capture_validate_region_negative_origin` -- Verify `RegionOutOfBounds` when region starts before display origin
   - [ ] `xcb_capture_validate_region_zero_dimensions` -- Verify `RegionOutOfBounds` for zero-width or zero-height region
   - [ ] `xcb_capture_validate_region_overflow` -- Verify `RegionOutOfBounds` when x + width overflows i32
2. **Green** -- Implement:
   - [ ] Implement `validate_region()` with bounds checking and overflow protection
3. **Refactor** -- Clean up:
   - [ ] Ensure error messages in `RegionOutOfBounds` are descriptive

**Completion Notes:**
>

---

### T005 -- Implement full-display capture

**Traces to:** FR-4, FR-6, AC-2.1, AC-2.2, AC-2.3
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/capture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `xcb_capture_full_display_correct_dimensions` -- Verify full capture returns `CaptureFrame` with `width == display_width`, `height == display_height` (integration test, gated)
   - [ ] `xcb_capture_full_display_rgba8_format` -- Verify `format == PixelFormat::Rgba8` (integration test, gated; xcap returns RGBA)
   - [ ] `xcb_capture_full_display_valid_stride` -- Verify `stride >= width * 4` (integration test, gated)
   - [ ] `xcb_capture_full_display_valid_data_length` -- Verify `data.len() >= (stride * height) as usize` (integration test, gated)
   - [ ] `xcb_capture_full_display_non_zero_pixels` -- Verify pixel data contains non-zero values on Xvfb (integration test, gated)
   - [ ] `xcb_capture_invalid_display_id_returns_not_found` -- Verify `DisplayNotFound` for bogus display ID (integration test, gated)
2. **Green** -- Implement:
   - [ ] Implement `ScreenCapture::capture_frame()` for the `region: None` path
   - [ ] Convert xcap `RgbaImage` to `CaptureFrame` with correct pixel format mapping
   - [ ] Handle display ID validation via `find_monitor()`
3. **Refactor** -- Clean up:
   - [ ] Verify pixel format is correct (xcap v0.9 returns RGBA via `image::RgbaImage` -- X11's native BGRA is converted internally by xcap)

**Completion Notes:**
>

---

### T006 -- Implement region-specific capture

**Traces to:** FR-4, FR-5, AC-3.1, AC-3.3
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/capture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `xcb_capture_region_correct_dimensions` -- Verify region capture returns `CaptureFrame` with `width == region.width`, `height == region.height` (integration test, gated)
   - [ ] `xcb_capture_region_out_of_bounds_error` -- Verify `RegionOutOfBounds` for region exceeding display bounds (integration test, gated)
   - [ ] `xcb_capture_region_small_source_performance` -- Benchmark 96x54 region capture; assert < 8ms (integration test, gated, relaxed for CI)
2. **Green** -- Implement:
   - [ ] Implement `capture_frame()` for the `region: Some(rect)` path using `image::imageops::crop_imm()`
   - [ ] Convert display-local coordinates to image-local coordinates for cropping
3. **Refactor** -- Clean up:
   - [ ] Consider whether xcap supports native region capture (avoiding full capture + crop) for performance

**Completion Notes:**
>

---

### T007 -- Implement self-capture prevention (RISK-002)

**Traces to:** FR-7, AC-4.1, AC-4.2, AC-4.3
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/capture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `xcb_capture_without_exclusion_returns_valid_data` -- Verify `XcbCapture::new()` with no excluded windows captures normally (integration test, gated)
   - [ ] `xcb_capture_set_excluded_windows_stores_ids` -- Verify `set_excluded_windows(&[42])` stores the ID and is used during capture (unit test, may verify via capture behavior on X11)
   - [ ] `xcb_capture_with_exclusion_excludes_overlay` -- Verify captured frame does not contain overlay pixels when exclusion is active (integration test, gated; requires overlay window from Story 002 -- may be deferred)
2. **Green** -- Implement:
   - [ ] Implement `set_excluded_windows()` override in `XcbCapture`'s `ScreenCapture` impl (stores `window_ids.to_vec()` in `self.excluded_window_ids`)
   - [ ] Implement self-capture exclusion logic in `capture_frame()` via unmap/remap cycle:
     - When `excluded_window_ids` is non-empty, unmap each excluded window via `x11rb` before capture
     - Call `xcap::Monitor::capture_image()` (excluded windows are hidden, not captured)
     - Remap each excluded window via `x11rb` immediately after capture
     - The brief unmap (~1-5ms per frame) is imperceptible at 60fps
3. **Refactor** -- Clean up:
   - [ ] Add logging at `info` level when self-capture exclusion is active: `log::info!("Self-capture exclusion active for '{}' window(s)", self.excluded_window_ids.len())`

**Completion Notes:**
>

---

### T008 -- Implement display change subscription

**Traces to:** FR-8, AC-5.1, AC-5.2
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/capture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `xcb_capture_subscribe_display_changes_returns_backend_unavailable` -- Verify `subscribe_display_changes(16)` returns `Err(CaptureError::BackendUnavailable { .. })` with reason containing "not yet implemented" (integration test, gated)
   - [ ] `xcb_capture_subscribe_display_changes_error_is_descriptive` -- Verify the `BackendUnavailable` reason string enables the caller to understand the limitation and fall back to polling (unit test)
2. **Green** -- Implement:
   - [ ] Implement `subscribe_display_changes()` returning `Err(CaptureError::BackendUnavailable { reason: "X11 display change events not yet implemented".into() })`. This is the correct Phase 0 behavior per the trait's fallback contract (AC-5.2): the caller falls back to periodic `list_displays()` polling.
3. **Refactor** -- Clean up:
   - [ ] Add doc-comment noting the Phase 0 limitation and that RandR event monitoring via `x11rb` is planned for a future iteration

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [ ] All Phase 1 + Phase 2 unit tests pass: `cargo nextest run -p luminos-platform -E 'test(~xcb_capture_validate)'`
- [ ] All integration tests pass on X11: `cargo nextest run -p luminos-platform --features ci_platform_tests -E 'test(~xcb_capture)'`
- [ ] `cargo clippy -p luminos-platform -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` passes

---

## Phase 3: Integration

### T009 -- Full capture pipeline integration test on Xvfb

**Traces to:** AC-1.3, AC-2.1, AC-2.2, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/capture.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `xcb_capture_integration_full_pipeline` -- End-to-end test: create `XcbCapture`, list displays, capture full frame, capture region, verify all properties match expectations on Xvfb with 1920x1080 screen
2. **Green** -- Test should pass with existing implementation from Phase 2
3. **Refactor** -- Clean up:
   - [ ] Add descriptive assertion messages for CI failure diagnosis

**Completion Notes:**
>

---

### T010 [P] -- Self-capture prevention integration test

**Traces to:** AC-4.1, AC-4.2
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/capture.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `xcb_capture_integration_self_capture_prevention` -- Create a magenta (#FF00FF) overlay window via `x11rb`, set exclusion via `set_excluded_windows(&[window_id as u64])`, capture frame, assert no magenta pixels present in the captured frame. This test may need to be deferred to Story 005 if creating a test overlay requires `WindowManager` from Story 002.
2. **Green** -- Implement any remaining self-capture prevention logic
3. **Refactor** -- Clean up:
   - [ ] Document whether this test fully validates RISK-002 or if additional validation is needed in Story 005

**Completion Notes:**
>

---

### T011 [P] -- Capture performance benchmarks

**Traces to:** AC-3.3, NFR-1
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/capture.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `xcb_capture_benchmark_small_region` -- Capture 96x54 region 10 times, assert average < 8ms (relaxed to < 50ms for CI)
   - [ ] `xcb_capture_benchmark_medium_region` -- Capture 960x540 region 10 times, assert average < 8ms (relaxed for CI)
   - [ ] `xcb_capture_benchmark_full_display` -- Capture full 1920x1080, assert average < 8ms (relaxed for CI)
2. **Green** -- Tests should pass with existing implementation
3. **Refactor** -- Clean up:
   - [ ] Add timing output via `eprintln!` for CI log visibility

**Completion Notes:**
>

---

### T012 -- Error propagation verification

**Traces to:** AC-6.1, AC-6.2
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/capture.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `xcb_capture_error_display_format` -- Verify `CaptureError::Platform` display output includes context and detail
   - [ ] `xcb_capture_error_to_luminos_error` -- Verify `From<CaptureError> for LuminosError` conversion (may already be covered by E01 tests; confirm)
2. **Green** -- Should pass with existing error types from E01
3. **Refactor** -- Clean up:
   - [ ] Ensure all error messages follow the logging convention: static text + single-quoted dynamic values

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 3, run full test suite and verify:
- [ ] All unit tests pass: `cargo nextest run -p luminos-platform`
- [ ] All integration tests pass on X11: `cargo nextest run -p luminos-platform --features ci_platform_tests`
- [ ] Performance benchmarks produce reasonable timings
- [ ] Self-capture prevention test passes (or is documented as deferred to Story 005)

---

## Phase 4: Polish & Acceptance

### T013 -- Documentation and clippy compliance

**Traces to:** NFR-3, NFR-4, NFR-5
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/capture.rs`

**Steps:**
1. Verify all public items (`XcbCapture`, `new()`, `set_excluded_windows()` trait method) have `///` doc-comments
2. Run `cargo doc -p luminos-platform --no-deps` and verify no documentation warnings
3. Run full clippy: `cargo clippy -p luminos-platform -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions`
4. Fix any clippy warnings
5. Verify no `unwrap()` or `expect()` in production code paths (only in `#[cfg(test)]` blocks)

**Completion Notes:**
>

---

### T014 -- Acceptance test verification

**Traces to:** All ACs
**Status:** TODO

**Verification Checklist:**
- [ ] AC-1.1: `list_displays()` returns non-empty vec with valid `DisplayInfo` fields
- [ ] AC-1.2: Exactly one display has `is_primary == true`
- [ ] AC-1.3: Primary display bounds match Xvfb 1920x1080 resolution
- [ ] AC-2.1: Full capture returns correct dimensions, format, stride, data length
- [ ] AC-2.2: Captured pixel data contains non-zero values
- [ ] AC-2.3: Invalid display ID returns `DisplayNotFound`
- [ ] AC-3.1: Region capture returns correct cropped dimensions
- [ ] AC-3.2: Region exceeding bounds returns `RegionOutOfBounds`
- [ ] AC-3.3: Small region capture completes in < 8ms (real hardware) / < 50ms (CI)
- [ ] AC-4.1: Capture with exclusion does not contain overlay pixels (or deferred)
- [ ] AC-4.2: Known-color overlay absent from captured frame (or deferred)
- [ ] AC-4.3: Capture without exclusion returns valid data
- [ ] AC-4.4: `ScreenCapture` trait has `set_excluded_windows()` with default no-op
- [ ] AC-4.5: `MockScreenCapture::set_excluded_windows()` stores IDs, verified via accessor
- [ ] AC-5.1: `subscribe_display_changes()` returns `Err(CaptureError::BackendUnavailable)` in Phase 0 (RandR not yet implemented)
- [ ] AC-5.2: `BackendUnavailable` reason is descriptive, enabling graceful fallback to polling `list_displays()`
- [ ] AC-6.1: `CaptureError::Platform` formats with context + detail
- [ ] AC-6.2: `From<CaptureError> for LuminosError` conversion works
- [ ] All clippy warnings resolved
- [ ] No `unwrap()` in production code paths
- [ ] Doc-comments on all public items
- [ ] `cargo fmt --all -- --check` passes

**Completion Notes:**
>

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | Self-capture prevention integration test (T010) may require overlay window from Story 002 | May defer full RISK-002 validation to Story 005 when all components are available | Open |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| --- | --- | --- |
