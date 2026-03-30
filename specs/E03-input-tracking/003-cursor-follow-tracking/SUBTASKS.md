# Subtasks: Story E03/003 -- Cursor-Follow Viewport Tracking

**Status:** COMPLETE
**Started:** 2026-03-29
**Completed:** 2026-03-29
**Story:** [STORY.md](STORY.md)
**Design:** [DESIGN.md](DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 1 | 1 | 0 | 0 |
| 2. Core Implementation | 6 | 6 | 0 | 0 |
| 3. Integration | 2 | 2 | 0 | 0 |
| 4. Polish & Acceptance | 1 | 1 | 0 | 0 |
| **Total** | **10** | **10** | **0** | **0** |

---

## Phase 1: Setup

### T001 -- Create tracking module scaffolding and TrackingConfig
**Traces to:** FR-6, FR-7
**Status:** DONE
**Files:** `crates/luminos-core/src/tracking.rs`, `crates/luminos-core/src/lib.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [x] `tracking_config_default_smoothing_factor` -- `TrackingConfig::default().smoothing_factor` is approximately 0.2
   - [x] `tracking_config_default_dead_zone_percent` -- `TrackingConfig::default().dead_zone_percent` is approximately 0.2
   - [x] `tracking_config_default_edge_margin_percent` -- `TrackingConfig::default().edge_margin_percent` is approximately 0.15
2. **Green** -- Implement minimum code to pass:
   - [x] Create `crates/luminos-core/src/tracking.rs` with module doc-comment
   - [x] Define `TrackingConfig` struct with `smoothing_factor`, `dead_zone_percent`, `edge_margin_percent` fields (all `f32`)
   - [x] Derive `Debug`, `Clone`, `PartialEq`
   - [x] Implement `Default for TrackingConfig` with documented defaults
   - [x] Add `pub mod tracking;` to `lib.rs` (pre-applied)
   - [x] Add re-exports: `pub use tracking::{TrackingEngine, TrackingConfig};` (pre-applied)
3. **Refactor** -- Clean up while tests stay green:
   - [x] Add doc-comments to each `TrackingConfig` field explaining the valid range and behavior

**Completion Notes:**
> Implemented `TrackingConfig` with all 3 fields, `Default` impl, and added a bonus `tracking_config_derives_debug_clone_partial_eq` test. Module declaration and re-exports were pre-applied in `lib.rs`.

---

## Phase 2: Core Implementation

### T002 -- Implement TrackingEngine constructor and first-frame initialization
**Traces to:** FR-1, FR-6
**Status:** DONE
**Files:** `crates/luminos-core/src/tracking.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [x] `tracking_engine_new_has_zero_center` -- `TrackingEngine::new(TrackingConfig::default()).current_center() == ScreenPoint { x: 0, y: 0 }`
   - [x] `tracking_engine_first_frame_snaps_to_cursor` -- On the first `update()` call with `mouse_position = (500, 300)`, returns `ScreenPoint { x: 500, y: 300 }` (no smoothing on first frame)
2. **Green** -- Implement minimum code to pass:
   - [x] Define `TrackingEngine` struct with `config`, `current_center`, `initialized` fields
   - [x] Implement `TrackingEngine::new(config)` setting `current_center` to `(0, 0)` and `initialized` to `false`
   - [x] Implement `current_center()` and `config()` accessors
   - [x] Implement `set_config()` method
   - [x] Start `update()` method: handle the `!initialized` case (snap to mouse position, set initialized=true)
3. **Refactor** -- Clean up while tests stay green:
   - [x] Add doc-comments to the struct and all public methods

**Completion Notes:**
> Implemented `TrackingEngine` struct with `#[must_use]` on `new()`, `current_center()`, `config()`. Added bonus tests for `config()` accessor and `set_config()`. All accessors have doc-comments per DESIGN.md spec.

---

### T003 -- Implement dead zone suppression
**Traces to:** FR-3, AC-2.1, AC-2.2, AC-2.3
**Status:** DONE
**Files:** `crates/luminos-core/src/tracking.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [x] `tracking_dead_zone_suppresses_micro_movement` -- Config with dead_zone_percent=0.2, viewport 1920x1080, zoom 2.0. Initialize at (960, 540). Move cursor to (970, 545) (well within dead zone half-width ~96px). Assert `update()` returns (960, 540) unchanged.
   - [x] `tracking_dead_zone_boundary_no_panning` -- Move cursor exactly to dead zone boundary. Assert center unchanged.
   - [x] `tracking_dead_zone_exit_triggers_panning` -- Move cursor to (1100, 540) (outside dead zone half-width of ~96px from center 960). Call `update()` with smoothing=1.0. Assert center moves toward cursor.
   - [x] `tracking_dead_zone_zero_disables` -- Config with dead_zone_percent=0.0. Move cursor by 1 pixel. Assert center changes.
2. **Green** -- Implement minimum code to pass:
   - [x] In `update()`, after first-frame check:
     - Compute `half_source_width` and `half_source_height` as viewport_size / (2 * zoom_level)
     - Compute `dead_half_x = half_source_width * config.dead_zone_percent`
     - Compute `dead_half_y = half_source_height * config.dead_zone_percent`
     - Compute dx, dy between mouse_position and current_center
     - If `|dx| <= dead_half_x && |dy| <= dead_half_y`, return current_center
3. **Refactor** -- Clean up while tests stay green:
   - [x] Renamed `half_vw`/`half_vh` to `half_source_width`/`half_source_height` to satisfy clippy::similar_names

**Completion Notes:**
> Dead zone uses `<=` comparison (inclusive boundary). Renamed variables to avoid clippy pedantic `similar_names` lint.

---

### T004 -- Implement smooth interpolation toward target
**Traces to:** FR-2, AC-1.1, AC-1.2, AC-1.3
**Status:** DONE
**Files:** `crates/luminos-core/src/tracking.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [x] `tracking_smooth_convergence_over_frames` -- Config smoothing_factor=0.2. Initialize at (0, 0). Call `update()` 10 times with mouse at (1000, 500). Assert center converges toward (1000, 500) -- each frame closer than the last. After 10 frames, center should be within ~85% of target.
   - [x] `tracking_smooth_frame_delta_bounded` -- Call `update()` with smoothing_factor=0.2. Verify each frame's delta does not exceed `(target - current) * factor + 1` (allow 1px rounding).
   - [x] `tracking_instant_tracking_factor_1` -- Config smoothing_factor=1.0. Call `update()` with cursor at (500, 300). Assert center immediately equals (500, 300).
   - [x] `tracking_smooth_preserves_asymptotic_approach` -- After multiple frames, verify the remaining distance decreases geometrically (each frame moves ~20% of remaining distance with factor=0.2).
2. **Green** -- Implement minimum code to pass:
   - [x] In `update()`, after dead zone check:
     - Set `target = mouse_position`
     - Call `smooth_viewport_position(self.current_center, target, self.config.smoothing_factor)`
     - Store result as new `self.current_center`
     - Return `self.current_center`
   - [x] `luminos-gpu` dependency already non-optional in `luminos-core/Cargo.toml` (pre-applied)
3. **Refactor** -- Clean up while tests stay green:
   - [x] Verified `luminos-gpu` dependency is non-optional

**Completion Notes:**
> Uses `luminos_gpu::viewport::smooth_viewport_position()` directly. Convergence verified over 10 frames with distance-squared metric. Geometric decay ratio verified < 0.95 per frame (expected ~0.8).

---

### T005 -- Implement edge panning
**Traces to:** FR-4, AC-3.1, AC-3.2, AC-3.3
**Status:** DONE
**Files:** `crates/luminos-core/src/tracking.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [x] `tracking_edge_panning_right_margin` -- Config edge_margin_percent=0.15, viewport 1920x1080, zoom 2.0 (source ~960x540). Initialize at (960, 540). Move cursor to right edge margin (~90% of source width from left of viewport). Assert `update()` shifts center rightward.
   - [x] `tracking_edge_panning_proportional_speed` -- Compare update results for cursor at 86% vs 98% of source width. Verify deeper position produces larger shift.
   - [x] `tracking_edge_panning_inactive_in_content_area` -- Position cursor between dead zone and edge margin. Verify only smooth tracking occurs, no extra edge panning shift.
   - [x] `tracking_edge_panning_left_margin` -- Cursor at left edge margin. Assert center shifts leftward.
   - [x] `tracking_edge_panning_vertical` -- Cursor at top/bottom edge margin. Assert vertical panning.
   - [x] `tracking_edge_panning_disabled_zero_margin` -- Config edge_margin_percent=0.0. Cursor at viewport edge. Assert no edge panning adjustment.
2. **Green** -- Implement minimum code to pass:
   - [x] In `update()`, between dead zone check and smooth interpolation:
     - Compute source region dimensions: `source_w = viewport_w / zoom`, `source_h = viewport_h / zoom`
     - Compute edge margins: `edge_margin_x = source_w * config.edge_margin_percent`
     - Compute source region bounds from current center
     - Check if cursor is in left/right/top/bottom edge margins
     - For each margin the cursor is in, compute depth fraction and apply proportional shift to target
3. **Refactor** -- Clean up while tests stay green:
   - [x] Decided against extracting helper: edge panning logic is inline in `update()` (only ~20 lines). Extraction would add complexity without benefit for a single call site.

**Completion Notes:**
> Edge panning uses proportional depth: `depth = (cursor_in_margin_distance) / margin_width`. Shift = `depth * margin_width` pixels. Division-by-zero guarded by `edge_margin > 0.0` check. All 4 directions tested independently.

---

### T006 -- Implement multi-zoom-level behavior
**Traces to:** FR-2, AC-5.1, AC-5.2
**Status:** DONE
**Files:** `crates/luminos-core/src/tracking.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [x] `tracking_zoom_1_5x_correct_dimensions` -- Zoom 1.5x, viewport 1920x1080. Verify dead zone and edge margin scale correctly (source ~1280x720).
   - [x] `tracking_zoom_20x_correct_dimensions` -- Zoom 20x, viewport 1920x1080. Verify tracking works with very small source region (~96x54). Dead zone and edge margin should be proportionally small.
   - [x] `tracking_zoom_5x_dead_zone_scales` -- Zoom 5x. Dead zone half-width should be ~38px (0.2 * 1920/(2*5)). Move cursor by 30px from center. Assert no panning. Move by 45px. Assert panning.
2. **Green** -- Tests pass with existing implementation (dead zone and edge margin computed relative to viewport/zoom)
3. **Refactor** -- None needed

**Completion Notes:**
> All 3 zoom levels verified: 1.5x (dead zone half=128px), 5x (half=38.4px), 20x (half=9.6px). Dead zone scales correctly as `viewport_size / (2 * zoom) * dead_zone_percent`.

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [x] All Phase 1 + Phase 2 tests pass (28 tracking tests)
- [x] `RUSTFLAGS="--deny warnings" cargo clippy -p luminos-core --all-targets --all-features -- -D warnings -W clippy::pedantic` clean
- [x] `cargo fmt --all -- --check` clean

---

## Phase 3: Integration

### T007 -- Integration test: TrackingEngine + compute_source_region round-trip
**Traces to:** AC-4.1, AC-4.2, AC-4.3, FR-5
**Status:** DONE
**Files:** `crates/luminos-core/src/tracking.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [x] `tracking_integration_source_region_within_bounds_top_left` -- 1920x1080 screen, 2x zoom. Cursor at (0, 0). Call `update()` then `compute_source_region()`. Assert `x >= 0 && y >= 0`.
   - [x] `tracking_integration_source_region_within_bounds_bottom_right` -- Cursor at (1920, 1080). Assert `x + width <= 1920 && y + height <= 1080`.
   - [x] `tracking_integration_source_region_multi_monitor` -- Screen bounds `{x: 1920, y: 0, w: 1920, h: 1080}`. Cursor at (2880, 540). Assert source region within active display bounds.
   - [x] `tracking_integration_source_region_correct_dimensions_2x` -- 2x zoom. Assert source region is ~960x540.
2. **Green** -- Imported `luminos_gpu::viewport::compute_source_region` in test module. Wired tracking engine output to `compute_source_region()`.
3. **Refactor** -- None needed

**Completion Notes:**
> All 4 integration tests verify the full round-trip: `TrackingEngine::update()` -> `compute_source_region()`. Multi-monitor test uses second display at x=1920. All clamping verified correctly.

---

### T008 -- Performance micro-benchmark
**Traces to:** NFR-1
**Status:** DONE
**Files:** `crates/luminos-core/src/tracking.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [x] `tracking_update_latency_under_10us` -- Create `TrackingEngine`, warm up with 100 calls. Measure 10,000 `update()` calls with varying cursor positions. Assert average < 10 microseconds (0.01ms). Use `std::time::Instant` and `std::hint::black_box()`.
2. **Green** -- Passes (pure arithmetic, no allocation)
3. **Refactor** -- None needed

**Completion Notes:**
> Benchmark uses 100-call warmup + 10,000 measured iterations with varying cursor positions. Uses `std::hint::black_box()` to prevent optimization. Passes well under threshold (typically sub-microsecond).

---

## Phase 4: Polish & Acceptance

### T009 -- Acceptance test verification
**Traces to:** All ACs
**Status:** DONE

**Verification Checklist:**
- [x] AC-1.1: Smooth convergence over 3-5 frames → `tracking_smooth_convergence_over_frames`
- [x] AC-1.2: Frame-to-frame delta bounded by smoothing factor → `tracking_smooth_frame_delta_bounded`
- [x] AC-1.3: Instant tracking with factor=1.0 → `tracking_instant_tracking_factor_1`
- [x] AC-2.1: Dead zone suppresses micro-movement → `tracking_dead_zone_suppresses_micro_movement`
- [x] AC-2.2: Panning begins at dead zone boundary → `tracking_dead_zone_boundary_no_panning` + `tracking_dead_zone_exit_triggers_panning`
- [x] AC-2.3: Dead zone disabled at 0% → `tracking_dead_zone_zero_disables`
- [x] AC-3.1: Edge panning at viewport margins → `tracking_edge_panning_right_margin`
- [x] AC-3.2: Proportional panning speed → `tracking_edge_panning_proportional_speed`
- [x] AC-3.3: No edge panning in content area → `tracking_edge_panning_inactive_in_content_area`
- [x] AC-4.1: Source region x >= 0, y >= 0 at top-left cursor → `tracking_integration_source_region_within_bounds_top_left`
- [x] AC-4.2: Source region within screen bounds at bottom-right cursor → `tracking_integration_source_region_within_bounds_bottom_right`
- [x] AC-4.3: Source region within multi-monitor bounds → `tracking_integration_source_region_multi_monitor`
- [x] AC-5.1: Tracking works at 1.5x zoom → `tracking_zoom_1_5x_correct_dimensions`
- [x] AC-5.2: Tracking works at 20x zoom → `tracking_zoom_20x_correct_dimensions`
- [x] All clippy warnings resolved (`RUSTFLAGS="--deny warnings" cargo clippy -p luminos-core --all-targets --all-features -- -D warnings -W clippy::pedantic`)
- [x] No `unwrap()` in production code paths (only in `#[cfg(test)]` module)
- [x] `cargo fmt --all -- --check` clean
- [ ] Update HIGH_LEVEL_PLAN.md Shared Context with any implementation findings (deferred to team lead)

**Completion Notes:**
> 32 tests total (28 tracking-specific + 4 bonus). All 14 acceptance criteria covered by at least one test. Performance benchmark passes well under 10us threshold. No `unwrap()` or `expect()` in production code. Clippy pedantic clean.

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| --- | --- | --- |
