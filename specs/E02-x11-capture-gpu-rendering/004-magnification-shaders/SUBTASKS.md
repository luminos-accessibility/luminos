# Subtasks: Story E02/004 -- Magnification Shaders & Viewport

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
| 1. Setup | 2 | 0 | 0 | 2 |
| 2. Core Implementation | 6 | 0 | 0 | 6 |
| 3. Integration | 4 | 0 | 0 | 4 |
| 4. Polish & Acceptance | 2 | 0 | 0 | 2 |
| **Total** | **14** | **0** | **0** | **14** |

---

## Phase 1: Setup

### T001 [P] -- Create viewport module with compute_source_region()

**Traces to:** FR-8, FR-9, AC-1.3, AC-1.4
**Status:** TODO
**Files:** `crates/luminos-gpu/src/viewport.rs`, `crates/luminos-gpu/src/lib.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `viewport_source_region_2x_zoom` -- 1920x1080 viewport at 2x zoom, target (960, 540) -> source 960x540 centered at (480, 270)
   - [ ] `viewport_source_region_5x_zoom` -- 1920x1080 viewport at 5x zoom -> source 384x216
   - [ ] `viewport_source_region_10x_zoom` -- 1920x1080 viewport at 10x zoom -> source 192x108
   - [ ] `viewport_source_region_20x_zoom` -- 1920x1080 viewport at 20x zoom -> source 96x54
   - [ ] `viewport_source_region_1_5x_zoom` -- 1920x1080 viewport at 1.5x zoom -> source 1280x720
   - [ ] `viewport_source_region_clamp_left` -- Target at (0, 540) with 2x zoom clamps x to 0
   - [ ] `viewport_source_region_clamp_top` -- Target at (960, 0) with 2x zoom clamps y to 0
   - [ ] `viewport_source_region_clamp_right` -- Target at (1920, 540) with 2x zoom clamps x to max
   - [ ] `viewport_source_region_clamp_bottom` -- Target at (960, 1080) with 2x zoom clamps y to max
   - [ ] `viewport_source_region_clamp_corner` -- Target at (0, 0) with 2x zoom clamps both
   - [ ] `viewport_source_region_zero_zoom` -- zoom = 0.0 returns zero-size region
2. **Green** -- Implement minimum code to pass:
   - [ ] Create `crates/luminos-gpu/src/viewport.rs` with `compute_source_region()` per DESIGN.md
   - [ ] Add `pub mod viewport;` to `crates/luminos-gpu/src/lib.rs`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments with examples

**Completion Notes:**
>

---

### T002 [P] -- Create smooth_viewport_position() function

**Traces to:** FR-10, AC-1.3
**Status:** TODO
**Files:** `crates/luminos-gpu/src/viewport.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `smooth_viewport_factor_1_0` -- Factor 1.0: current (0, 0), target (100, 200) -> result (100, 200) (instant)
   - [ ] `smooth_viewport_factor_0_0` -- Factor 0.0: current (100, 200), target (300, 400) -> result (100, 200) (no movement)
   - [ ] `smooth_viewport_factor_0_5` -- Factor 0.5: current (0, 0), target (100, 200) -> result (50, 100) (halfway)
   - [ ] `smooth_viewport_clamp_factor` -- Factor 1.5: result equals target (clamped to 1.0)
2. **Green** -- Implement minimum code to pass:
   - [ ] Add `smooth_viewport_position()` to `viewport.rs` per DESIGN.md
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Verify doc-comment matches behavior

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 1, verify:
- [ ] `cargo build -p luminos-gpu` compiles
- [ ] All viewport unit tests pass
- [ ] `compute_source_region()` returns correct dimensions at all zoom levels
- [ ] Edge clamping works correctly in all corner cases

---

## Phase 2: Core Implementation

### T003 -- Define MagnifyUniforms struct with bytemuck derives

**Traces to:** FR-3, FR-6, AC-4.4
**Status:** TODO
**Files:** `crates/luminos-gpu/src/shaders/mod.rs`, `crates/luminos-gpu/Cargo.toml`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `magnify_uniforms_size_32_bytes` -- Verify `size_of::<MagnifyUniforms>()` == 32
   - [ ] `magnify_uniforms_bytemuck_cast` -- Verify `bytemuck::bytes_of(&uniforms)` returns 32-byte slice
   - [ ] `magnify_uniforms_default_values` -- Verify `MagnifyUniforms::zeroed()` has all fields at 0.0
2. **Green** -- Implement minimum code to pass:
   - [ ] Create `crates/luminos-gpu/src/shaders/mod.rs` with `MagnifyUniforms` struct per DESIGN.md
   - [ ] Add `bytemuck = { version = "1", features = ["derive"] }` to `luminos-gpu/Cargo.toml` workspace deps if needed
   - [ ] Create `crates/luminos-gpu/src/shaders/` directory
   - [ ] Add `pub mod shaders;` to `lib.rs`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments explaining alignment requirements

**Completion Notes:**
>

---

### T004 -- Define InterpolationMethod enum and bind group layout

**Traces to:** FR-5, FR-7, AC-4.3
**Status:** TODO
**Files:** `crates/luminos-gpu/src/shaders/mod.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `interpolation_method_bilinear_ne_bicubic` -- Verify `InterpolationMethod::Bilinear != InterpolationMethod::Bicubic`
   - [ ] `bind_group_layout_creates` -- (Integration, requires wgpu device) Verify `create_magnify_bind_group_layout()` returns a valid layout
2. **Green** -- Implement minimum code to pass:
   - [ ] Add `InterpolationMethod` enum to `shaders/mod.rs`
   - [ ] Add `create_magnify_bind_group_layout()` function per DESIGN.md
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to layout entries explaining each binding

**Completion Notes:**
>

---

### T005 -- Create bilinear magnification shader (magnify_bilinear.wgsl)

**Traces to:** FR-1, FR-4, AC-1.1, AC-3.1, AC-3.2, AC-5.1, AC-5.2
**Status:** TODO
**Files:** `crates/luminos-gpu/src/shaders/magnify_bilinear.wgsl`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `shader_bilinear_compiles` -- (Integration) Compile `magnify_bilinear.wgsl` with `device.create_shader_module()`, verify no error
   - [ ] `pipeline_bilinear_creates` -- (Integration) Create full `RenderPipeline` with bilinear shader, verify success
2. **Green** -- Implement minimum code to pass:
   - [ ] Create `magnify_bilinear.wgsl` with vertex shader (`vs_main`) and fragment shader (`fs_main`) per DESIGN.md
   - [ ] Vertex shader: full-screen triangle from `vertex_index` (no vertex buffer)
   - [ ] Fragment shader: single `textureSampleLevel` call + BGRA swizzle
   - [ ] Add `create_magnify_pipeline()` function to `shaders/mod.rs` per DESIGN.md
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add WGSL comments explaining the full-screen triangle technique

**Completion Notes:**
>

---

### T006 -- Create bicubic magnification shader (magnify_bicubic.wgsl)

**Traces to:** FR-2, FR-4, AC-2.1, AC-2.2, AC-2.3
**Status:** TODO
**Files:** `crates/luminos-gpu/src/shaders/magnify_bicubic.wgsl`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `shader_bicubic_compiles` -- (Integration) Compile `magnify_bicubic.wgsl` with `device.create_shader_module()`, verify no error
   - [ ] `pipeline_bicubic_creates` -- (Integration) Create full `RenderPipeline` with bicubic shader, verify success
2. **Green** -- Implement minimum code to pass:
   - [ ] Create `magnify_bicubic.wgsl` with shared vertex shader and bicubic fragment shader per DESIGN.md
   - [ ] Fragment shader: `cubic_weight()` function + `sample_bicubic()` 4x4 loop + BGRA swizzle
   - [ ] Verify both pipelines use the same bind group layout (swap test: create bind group with either pipeline's layout)
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add WGSL comments explaining Catmull-Rom weight derivation

**Completion Notes:**
>

---

### T007 -- Implement create_magnify_bind_group() helper

**Traces to:** FR-7, AC-4.3
**Status:** TODO
**Files:** `crates/luminos-gpu/src/shaders/mod.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `bind_group_creates_with_texture` -- (Integration) Create a test texture, sampler, and uniform buffer, verify `create_magnify_bind_group()` returns a valid bind group
2. **Green** -- Implement minimum code to pass:
   - [ ] Add `create_magnify_bind_group()` function per DESIGN.md
   - [ ] Add sampler creation helper (linear filtering for bilinear, point for bicubic is handled by the shader's manual sampling)
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments explaining when bind groups need recreation (texture change)

**Completion Notes:**
>

---

### T008 -- Create test texture sampler helper

**Traces to:** AC-4.3
**Status:** TODO
**Files:** `crates/luminos-gpu/src/shaders/mod.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `sampler_linear_filtering` -- (Integration) Create sampler, verify it can be bound to the magnify bind group
2. **Green** -- Implement minimum code to pass:
   - [ ] Add `create_magnify_sampler()` function that creates a `wgpu::Sampler` with `FilterMode::Linear` for magnification
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments explaining filtering mode choice

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [ ] All Phase 1 + Phase 2 tests pass
- [ ] Both shader variants compile without errors
- [ ] Both render pipelines create successfully
- [ ] `MagnifyUniforms` has correct size and alignment
- [ ] `cargo clippy -p luminos-gpu -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` passes
- [ ] No `unwrap()` or `expect()` in production code paths

---

## Phase 3: Integration

### T009 -- Shader output test: bilinear solid color rendering

**Traces to:** AC-1.1, AC-1.2, AC-3.1, AC-3.2, AC-5.1
**Status:** TODO
**Files:** `crates/luminos-gpu/tests/shader_output.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `shader_bilinear_solid_red_rgba` -- Create a 64x64 solid red RGBA source texture, render through bilinear shader with `is_bgra = 0.0` to a 128x128 output texture (2x zoom), read back output pixels, verify all are red (R=255, G=0, B=0, A=255)
   - [ ] `shader_bilinear_solid_blue_bgra_swizzle` -- Create a 64x64 source with blue in R channel (BGRA format), render with `is_bgra = 1.0`, verify output shows blue correctly
   - [ ] `shader_bilinear_1_5x_zoom_no_artifacts` -- Render at 1.5x zoom, verify no black pixels in output
   - [ ] `shader_bilinear_20x_zoom_no_artifacts` -- Render at 20x zoom from a small source, verify output is filled
2. **Green** -- Implement minimum code to pass:
   - [ ] Create headless wgpu test harness (GL backend, no window)
   - [ ] Create source texture with known pixel data
   - [ ] Execute render pass writing to a storage texture
   - [ ] Read back pixels via `buffer.map_async()` and verify values
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Extract headless GPU test harness into a reusable `#[cfg(test)]` helper in `luminos-gpu`

**Completion Notes:**
>

---

### T010 -- Shader output test: bicubic solid color and quality comparison

**Traces to:** AC-2.1, AC-2.2, AC-2.3
**Status:** TODO
**Files:** `crates/luminos-gpu/tests/shader_output.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `shader_bicubic_solid_red_rgba` -- Same as bilinear solid red test but with bicubic shader, verify output is red
   - [ ] `shader_bicubic_solid_blue_bgra_swizzle` -- Same as bilinear BGRA test but with bicubic shader
   - [ ] `shader_bicubic_edge_quality` -- Create a source with a sharp black-white edge, render at 10x with both bilinear and bicubic, verify bicubic output has more intermediate gray values at the edge (smoother transition)
2. **Green** -- Implement minimum code to pass:
   - [ ] Reuse headless GPU test harness from T009
   - [ ] Create edge test texture (left half black, right half white)
   - [ ] Compare edge pixel values between bilinear and bicubic output
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Extract texture creation helpers into shared test utilities

**Completion Notes:**
>

---

### T011 -- Integration test: pipeline creation with both variants

**Traces to:** AC-4.1, AC-4.2, AC-4.3
**Status:** TODO
**Files:** `crates/luminos-gpu/tests/pipeline_creation.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `pipeline_both_variants_compile` -- Create a wgpu device (GL backend), compile both bilinear and bicubic shaders, create both render pipelines, verify both succeed
   - [ ] `pipeline_variant_swap` -- Create both pipelines with the same bind group layout, verify the same bind group can be used with either pipeline (proves layout compatibility)
   - [ ] `pipeline_uniform_buffer_write` -- Write `MagnifyUniforms` to the uniform buffer, verify `queue.write_buffer()` does not error
2. **Green** -- Implement minimum code to pass:
   - [ ] Wire together device creation, shader compilation, pipeline creation
   - [ ] Create bind group with a test texture and verify it works with both pipelines
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Consolidate GPU test setup code

**Completion Notes:**
>

---

### T012 -- Integration test: full-screen triangle UV coverage

**Traces to:** AC-5.1, AC-5.2
**Status:** TODO
**Files:** `crates/luminos-gpu/tests/shader_output.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `fullscreen_triangle_covers_viewport` -- Create a gradient source texture (UV-encoded: R = U * 255, G = V * 255), render through bilinear shader to an output texture, read back pixels at corners (0,0), (W-1,0), (0,H-1), (W-1,H-1), verify UV values are approximately (0,0), (255,0), (0,255), (255,255)
2. **Green** -- Implement minimum code to pass:
   - [ ] Create UV-gradient source texture
   - [ ] Render and read back corner pixels
   - [ ] Allow small tolerance (+-2) for interpolation rounding
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add comments explaining the UV gradient verification technique

**Completion Notes:**
>

---

## Phase 4: Polish & Acceptance

### T013 -- Doc-comments and clippy clean pass

**Traces to:** NFR-3, NFR-4, NFR-5
**Status:** TODO
**Files:** All files created/modified in this story

**Steps:**
1. [ ] Verify all public items in `viewport.rs`, `shaders/mod.rs` have `///` doc-comments
2. [ ] Verify WGSL shaders have descriptive comments
3. [ ] Run `cargo doc -p luminos-gpu --no-deps` and verify no warnings
4. [ ] Run full clippy command and fix any remaining warnings
5. [ ] Verify no `unwrap()` or `expect()` in production code (search `crates/luminos-gpu/src/` excluding test modules)
6. [ ] Run `cargo fmt --all -- --check` and verify clean

**Completion Notes:**
>

---

### T014 -- Acceptance test verification

**Traces to:** All ACs
**Status:** TODO

**Verification Checklist:**
- [ ] AC-1.1: Bilinear shader renders 2x magnified output correctly
- [ ] AC-1.2: Magnification at 1.5x, 5x, 10x, 20x renders without artifacts
- [ ] AC-1.3: `compute_source_region` returns correct dimensions at all zoom levels
- [ ] AC-1.4: Source region clamped to screen bounds at all edges
- [ ] AC-2.1: Bicubic shader produces smoother edges than bilinear at 10x
- [ ] AC-2.2: Bicubic shader uses 4x4 tap pattern (verified by code review)
- [ ] AC-2.3: Both shader variants compile and create render pipelines
- [ ] AC-3.1: BGRA swizzle produces correct colors with `is_bgra = 1.0`
- [ ] AC-3.2: RGBA passthrough produces correct colors with `is_bgra = 0.0`
- [ ] AC-4.1: Bilinear shader compiles on Vulkan and GL backends
- [ ] AC-4.2: Bicubic shader compiles on Vulkan and GL backends
- [ ] AC-4.3: Render pipelines create successfully for both variants
- [ ] AC-4.4: `MagnifyUniforms` is 32 bytes with correct alignment
- [ ] AC-5.1: Full-screen triangle covers entire viewport
- [ ] AC-5.2: UV coordinates map correctly (gradient test)
- [ ] All clippy warnings resolved
- [ ] No `unwrap()` in production code paths
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo nextest run -p luminos-gpu` passes

**Completion Notes:**
>

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| --- | --- | --- |
