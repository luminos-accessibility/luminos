# Subtasks: Story E02/003 -- GPU Texture Pipeline

**Status:** DONE
**Started:** 2026-03-28
**Completed:** 2026-03-28
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 1 | 1 | 0 | 0 |
| 2. Core Implementation | 5 | 5 | 0 | 0 |
| 3. Integration | 3 | 3 | 0 | 0 |
| 4. Polish & Acceptance | 2 | 2 | 0 | 0 |
| **Total** | **11** | **11** | **0** | **0** |

---

## Phase 1: Setup

### T001 -- Create texture module structure

**Traces to:** FR-1
**Status:** DONE
**Files:** `crates/luminos-gpu/src/lib.rs`, `crates/luminos-gpu/src/texture.rs`

**Steps:**
1. Update `crates/luminos-gpu/src/lib.rs` to add `pub mod texture;`
2. Create `crates/luminos-gpu/src/texture.rs` with the `SourceTextureManager` struct skeleton (fields, constants, no methods yet):
   ```rust
   pub struct SourceTextureManager { /* fields from DESIGN.md */ }
   const STALE_FRAME_WARN_THRESHOLD: u32 = 60;
   const OVER_ALLOCATION_FACTOR: f32 = 1.5;
   const SOURCE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
   ```
3. Add necessary imports: `wgpu`, `luminos_platform::traits::types::CaptureFrame`, `log`
4. Verify `cargo build -p luminos-gpu` compiles

**Completion Notes:**
> Module created at `crates/luminos-gpu/src/texture.rs` with `pub mod texture;` in `lib.rs`. `SourceTextureManager` struct defined with all fields per DESIGN.md. Constants defined. Import uses `luminos_types::CaptureFrame` (via the `luminos-types` crate, not `luminos_platform::traits::types`). Module compiles clean.

---

**Checkpoint:** After completing Phase 1, verify:
- [x] `cargo build -p luminos-gpu` compiles
- [x] `SourceTextureManager` struct exists in `texture.rs`
- [x] Module is exported from `lib.rs`

---

## Phase 2: Core Implementation

### T002 -- Implement over_allocate helper and texture creation

**Traces to:** FR-2, FR-8, AC-2.1, AC-5.1
**Status:** DONE
**Files:** `crates/luminos-gpu/src/texture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `texture_over_allocate_small` -- Verify `over_allocate(100)` returns `150` (1.5x)
   - [ ] `texture_over_allocate_960` -- Verify `over_allocate(960)` returns `1440`
   - [ ] `texture_over_allocate_1920` -- Verify `over_allocate(1920)` returns `2880`
   - [ ] `texture_over_allocate_one` -- Verify `over_allocate(1)` returns at least `2` (ceiling)
   - [ ] `texture_source_format_is_srgb` -- Verify `SOURCE_TEXTURE_FORMAT == Rgba8UnormSrgb`
2. **Green** -- Implement:
   - [ ] Implement `over_allocate()` function: `(dimension as f32 * OVER_ALLOCATION_FACTOR).ceil() as u32`
   - [ ] Implement `create_source_texture()` function with `Rgba8UnormSrgb` format, `TEXTURE_BINDING | COPY_DST` usage
3. **Refactor** -- Clean up:
   - [x] Add doc-comments to helper functions

**Completion Notes:**
> Implemented `over_allocate()` using `f64` cast (not `f32`) to avoid precision issues with large dimensions. Added `#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]` for the float-to-int cast. `create_source_texture()` creates textures with `TEXTURE_BINDING | COPY_DST` usage flags and `Rgba8UnormSrgb` format. Six unit tests passing: `texture_over_allocate_small`, `texture_over_allocate_960`, `texture_over_allocate_1920`, `texture_over_allocate_one`, `texture_over_allocate_zero` (extra test), `texture_source_format_is_srgb`.

---

### T003 -- Implement SourceTextureManager constructor

**Traces to:** FR-2, AC-2.1
**Status:** DONE
**Files:** `crates/luminos-gpu/src/texture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `texture_manager_new_capacity_over_allocated` -- Create `SourceTextureManager::new(device, 960, 540)` on Mesa llvmpipe, verify `capacity_width >= 1440` and `capacity_height >= 810` (integration test)
   - [ ] `texture_manager_new_initial_dimensions` -- Verify `current_dimensions()` returns `(960, 540)` after construction
2. **Green** -- Implement:
   - [ ] Implement `SourceTextureManager::new()` constructor per DESIGN.md
   - [ ] Create over-allocated texture and texture view
3. **Refactor** -- Clean up:
   - [x] Ensure constructor does not use `unwrap()` (all wgpu creation is infallible in this API)

**Completion Notes:**
> Constructor implemented as `SourceTextureManager::new(device, initial_width, initial_height)` per DESIGN.md. Takes ownership of `wgpu::Device` (needed for reallocation). Creates over-allocated texture and view. Three async integration tests passing on Mesa llvmpipe: `texture_manager_new_initial_dimensions`, `texture_manager_new_capacity_over_allocated`, `texture_manager_new_stale_count_zero`. No `unwrap()` in production code. `#[must_use]` annotation added.

---

### T004 -- Implement upload method

**Traces to:** FR-3, FR-4, AC-1.1, AC-1.3, AC-2.2, AC-2.3, AC-2.4
**Status:** DONE
**Files:** `crates/luminos-gpu/src/texture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `texture_manager_upload_updates_dimensions` -- Upload a 640x480 frame, verify `current_dimensions() == (640, 480)`
   - [ ] `texture_manager_upload_within_capacity_no_realloc` -- Upload a frame within capacity, verify capacity unchanged (unit test using dimension tracking)
   - [ ] `texture_manager_upload_exceeds_capacity_realloc` -- Upload a frame exceeding capacity, verify capacity increases to 1.5x new dimensions
   - [ ] `texture_manager_upload_resets_stale_count` -- Set stale count to 30, upload a frame, verify `stale_frame_count() == 0`
2. **Green** -- Implement:
   - [ ] Implement `upload()` method with reallocation check and `write_texture()` call
   - [ ] Implement `reallocate()` private method
   - [ ] Handle stride via `TexelCopyBufferLayout::bytes_per_row`
3. **Refactor** -- Clean up:
   - [x] Add logging for reallocation events at `info` level

**Completion Notes:**
> `upload()` method implemented with reallocation check, `Queue::write_texture()` call with proper `TexelCopyBufferLayout` for stride handling, dimension tracking, and stale count reset. Private `reallocate()` method logs at `info` level with old and new dimensions. Four async tests passing: `texture_manager_upload_updates_dimensions`, `texture_manager_upload_within_capacity_no_realloc`, `texture_manager_upload_exceeds_capacity_realloc`, `texture_manager_upload_resets_stale_count`.

---

### T005 -- Implement stale frame tracking

**Traces to:** FR-5, AC-3.1, AC-3.2, AC-3.3
**Status:** DONE
**Files:** `crates/luminos-gpu/src/texture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `texture_manager_stale_count_increments` -- Call `record_capture_failure()` 5 times, verify `stale_frame_count() == 5`
   - [ ] `texture_manager_stale_threshold_warning` -- Call `record_capture_failure()` 60 times, verify stale count reaches threshold (log output tested via counter, not log capture)
   - [ ] `texture_manager_stale_count_resets_on_upload` -- Call `record_capture_failure()` 30 times, then `upload()`, verify `stale_frame_count() == 0`
   - [ ] `texture_manager_stale_preserves_texture_view` -- After failures, verify `texture_view()` still returns a reference (not panicking or null)
2. **Green** -- Implement:
   - [ ] Implement `record_capture_failure()` with counter increment and threshold logging
   - [ ] Implement `stale_frame_count()` getter
3. **Refactor** -- Clean up:
   - [x] Verify log message format follows convention: `log::warn!("Capture stale for '{}' consecutive frames", count)`

**Completion Notes:**
> `record_capture_failure()` increments stale counter and logs `warn!` at exactly 60 consecutive failures (includes seconds calculation). `stale_frame_count()` getter returns counter value. Four async tests passing: `texture_manager_stale_count_increments`, `texture_manager_stale_threshold_warning`, `texture_manager_stale_count_resets_on_upload`, `texture_manager_stale_preserves_texture_view`.

---

### T006 -- Implement texture_view and current_dimensions accessors

**Traces to:** FR-6, FR-7, AC-4.1, AC-4.2
**Status:** DONE
**Files:** `crates/luminos-gpu/src/texture.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `texture_manager_texture_view_returns_reference` -- Verify `texture_view()` returns a `&TextureView` (compilation test, the borrow check is sufficient)
   - [ ] `texture_manager_current_dimensions_after_upload` -- Upload 800x600 frame, verify `current_dimensions() == (800, 600)`
   - [ ] `texture_manager_current_dimensions_after_realloc` -- Upload frame that triggers realloc, verify `current_dimensions()` reflects the frame dimensions (not the over-allocated capacity)
2. **Green** -- Implement:
   - [ ] Implement `texture_view()` returning `&self.view`
   - [ ] Implement `current_dimensions()` returning `(self.current_width, self.current_height)`
3. **Refactor** -- Clean up:
   - [x] Add doc-comments referencing usage in `MagnifyUniforms.source_size`

**Completion Notes:**
> `texture_view()` returns `&wgpu::TextureView` with `#[must_use]`. `current_dimensions()` returns `(u32, u32)` tuple of frame dimensions (not capacity). Added `capacity()` accessor (not in original spec but useful for testing). Three async tests passing: `texture_manager_texture_view_returns_reference`, `texture_manager_current_dimensions_after_upload`, `texture_manager_current_dimensions_after_realloc`.

---

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [x] All unit tests pass: `cargo nextest run -p luminos-gpu -E 'test(~texture_)'`
- [x] `cargo clippy -p luminos-gpu -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` passes

---

## Phase 3: Integration

### T007 -- GPU texture creation and upload integration test

**Traces to:** AC-1.1, AC-1.2, AC-5.1, AC-5.2
**Status:** DONE
**Files:** `crates/luminos-gpu/src/texture.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `texture_integration_create_and_upload` -- On Mesa llvmpipe: create `SourceTextureManager`, upload a `generate_test_capture_frame(256, 256, [0, 0, 255, 255])`, verify texture is `Rgba8UnormSrgb` format. Read back texture data via a staging buffer and `map_async`, verify pixel values are present.
   - [ ] `texture_integration_stride_padding` -- Upload a frame with artificial stride padding (stride = width * 4 + 32), verify readback is correct (no row misalignment)
2. **Green** -- Tests should pass with Phase 2 implementation
3. **Refactor** -- Clean up:
   - [x] Extract wgpu test device creation into a helper: `generate_test_gpu_device()` for reuse across GPU tests

**Completion Notes:**
> Integration tests `texture_integration_create_and_upload` and `texture_integration_stride_padding` passing on Mesa llvmpipe. `generate_test_gpu_device()` helper extracted as async function returning `Option<(wgpu::Device, wgpu::Queue)>` using GL+Vulkan backends with graceful skip when no adapter is available. `generate_test_capture_frame_rgba()` and `generate_test_capture_frame_with_stride()` helpers created for test frame generation.

---

### T008 -- Reallocation integration test

**Traces to:** AC-2.3, AC-2.4
**Status:** DONE
**Files:** `crates/luminos-gpu/src/texture.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `texture_integration_reallocation` -- Create manager with 256x256 initial, upload 512x512 frame (exceeds 384x384 capacity), verify reallocation occurred (capacity >= 768x768), upload succeeds, readback correct
   - [ ] `texture_integration_reallocation_preserves_view` -- After reallocation, verify `texture_view()` returns a new valid view (can be bound to a bind group)
2. **Green** -- Tests should pass with Phase 2 implementation
3. **Refactor** -- Clean up:
   - [x] Add assertion messages for CI log diagnosis

**Completion Notes:**
> Integration tests `texture_integration_reallocation` and `texture_integration_reallocation_preserves_view` passing. Reallocation test verifies initial capacity (384x384), triggers realloc with 512x512 frame, confirms new capacity >= 768x768, and verifies subsequent frames within new capacity don't trigger further realloc. View preservation test confirms `texture_view()` returns valid reference after reallocation.

---

### T009 -- Upload performance benchmark

**Traces to:** NFR-1
**Status:** DONE
**Files:** `crates/luminos-gpu/src/texture.rs` (test module)

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `texture_benchmark_upload_small_region` -- Upload 96x54 frame 100 times, report average time via `eprintln!`
   - [ ] `texture_benchmark_upload_medium_region` -- Upload 960x540 frame 100 times, report average time
   - [ ] `texture_benchmark_upload_large_region` -- Upload 1280x720 frame 100 times, report average time
2. **Green** -- Tests should pass (benchmark assertions are informational, not hard failures on CI)
3. **Refactor** -- Clean up:
   - [x] Consider adding `#[ignore]` attribute for CI performance benchmarks to prevent test suite slowdown; run explicitly via `--ignored`

**Completion Notes:**
> Three benchmark tests created: `texture_benchmark_upload_small_region` (96x54), `texture_benchmark_upload_medium_region` (960x540), `texture_benchmark_upload_large_region` (1280x720). All marked with `#[ignore]` to exclude from regular CI runs. Each runs 100 iterations and reports average time via `eprintln!`.

---

**Checkpoint:** After completing Phase 3, run full test suite and verify:
- [x] All unit tests pass: `cargo nextest run -p luminos-gpu`
- [x] All integration tests pass on Mesa llvmpipe: `xvfb-run cargo nextest run -p luminos-gpu`
- [x] Performance benchmarks produce reasonable timings

---

## Phase 4: Polish & Acceptance

### T010 -- Documentation and clippy compliance

**Traces to:** NFR-2, NFR-4, NFR-5
**Status:** DONE
**Files:** `crates/luminos-gpu/src/texture.rs`

**Steps:**
1. Verify all public items have `///` doc-comments
2. Run `cargo doc -p luminos-gpu --no-deps` and verify no documentation warnings
3. Run full clippy: `cargo clippy -p luminos-gpu -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions`
4. Fix any clippy warnings
5. Verify no `unwrap()` or `expect()` in production code paths

**Completion Notes:**
> All public items have `///` doc-comments including module-level `//!` documentation. `cargo doc -p luminos-gpu --no-deps` produces no warnings. Full clippy clean with pedantic lints. No `unwrap()` or `expect()` in production code paths. `cargo fmt --all -- --check` passes.

---

### T011 -- Acceptance test verification

**Traces to:** All ACs
**Status:** DONE

**Verification Checklist:**
- [x] AC-1.1: Upload succeeds and texture data is available for shader sampling
- [x] AC-1.2: Source texture format is `Rgba8UnormSrgb`
- [x] AC-1.3: Stride padding is correctly handled in `bytes_per_row`
- [x] AC-2.1: Initial texture capacity is 1.5x of requested dimensions
- [x] AC-2.2: Upload within capacity does not trigger reallocation
- [x] AC-2.3: Upload exceeding capacity triggers reallocation to 1.5x new dimensions
- [x] AC-2.4: After reallocation, texture is immediately usable
- [x] AC-3.1: Last uploaded texture persists through capture failures
- [x] AC-3.2: Warning logged at 60 consecutive stale frames
- [x] AC-3.3: Stale counter resets on successful upload
- [x] AC-4.1: `texture_view()` returns valid `TextureView`
- [x] AC-4.2: `current_dimensions()` returns frame dimensions (not capacity)
- [x] AC-5.1: Texture format is `Rgba8UnormSrgb`
- [x] AC-5.2: sRGB decode happens automatically on shader sample
- [x] All clippy warnings resolved
- [x] No `unwrap()` in production code paths
- [x] Doc-comments on all public items
- [x] `cargo fmt --all -- --check` passes

**Completion Notes:**
> All acceptance criteria verified. 243 tests passing across workspace (excluding luminos-app). Story 003 contributes approximately 25 tests (unit + integration). All QA, code review, and technical audit quality gates passed.

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | GPU integration tests require wgpu device (Story 002 dependency) | Use `wgpu::Instance::new()` directly in tests with `Backends::GL` for Mesa llvmpipe | Open |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| T001 | Import uses `luminos_types::CaptureFrame` instead of `luminos_platform::traits::types::CaptureFrame` | `luminos-types` crate is the canonical source for shared types since Story 002 type unification |
| T002 | `over_allocate()` uses `f64` cast instead of `f32` | Avoids precision loss for large dimensions (>16 million pixels); `f32` has only 24 bits of mantissa |
| T006 | Added `capacity()` accessor not in original DESIGN.md | Useful for integration tests to verify over-allocation behavior without accessing private fields |
