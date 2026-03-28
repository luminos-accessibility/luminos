# Subtasks: Story E02/002 -- X11 Overlay Window & GPU Surface

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
| 3. Integration | 3 | 0 | 0 | 3 |
| 4. Polish & Acceptance | 2 | 0 | 0 | 2 |
| **Total** | **14** | **0** | **0** | **14** |

---

## Phase 1: Setup

### T001 [P] -- Unify DockEdge/LensShape types with serde derives

**Traces to:** FR-6, AC-4.1, AC-4.2, AC-4.3, AC-4.4
**Status:** TODO
**Files:** `crates/luminos-platform/Cargo.toml`, `crates/luminos-platform/src/traits/window_manager.rs`, `crates/luminos-core/src/config/schema.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `dock_edge_serde_roundtrip` -- Serialize each `DockEdge` variant to JSON and deserialize back, verify equality
   - [ ] `lens_shape_serde_roundtrip` -- Serialize each `LensShape` variant to JSON and deserialize back, verify equality
   - [ ] `overlay_mode_serde_roundtrip` -- Serialize each `OverlayMode` variant to JSON and deserialize back, verify equality
2. **Green** -- Implement minimum code to pass:
   - [ ] Add `serde = { workspace = true, features = ["derive"] }` to `luminos-platform/Cargo.toml`
   - [ ] Add `Serialize, Deserialize, Hash` derives to `DockEdge`, `LensShape`, `OverlayMode` in `window_manager.rs`
   - [ ] Replace `DockEdge` and `LensShape` definitions in `luminos-core/src/config/schema.rs` with `pub use luminos_platform::traits::{DockEdge, LensShape};`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Verify all existing tests in both crates still pass after the re-export change
   - [ ] Remove any now-unused imports in `schema.rs`

**Completion Notes:**
>

---

### T002 [P] -- Define RenderError enum in luminos-gpu

**Traces to:** FR-8, AC-2.4
**Status:** TODO
**Files:** `crates/luminos-gpu/src/error.rs`, `crates/luminos-gpu/src/lib.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `render_error_display_no_adapter` -- Verify `RenderError::NoAdapter` displays "no compatible GPU adapter found"
   - [ ] `render_error_display_device_creation` -- Verify `RenderError::DeviceCreation` includes the message
   - [ ] `render_error_display_surface_configuration` -- Verify `RenderError::SurfaceConfiguration` includes the message
   - [ ] `render_error_display_surface_texture` -- Verify `RenderError::SurfaceTexture` includes the message
   - [ ] `render_error_display_shader_compilation` -- Verify `RenderError::ShaderCompilation` includes the message
   - [ ] `render_error_display_render_failed` -- Verify `RenderError::RenderFailed` includes the message
2. **Green** -- Implement minimum code to pass:
   - [ ] Create `crates/luminos-gpu/src/error.rs` with `RenderError` enum per DESIGN.md
   - [ ] Add `pub mod error;` to `crates/luminos-gpu/src/lib.rs`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to all variants

**Completion Notes:**
>

---

### T003 [P] -- Create linux_x11/window.rs module and X11WindowManager struct skeleton

**Traces to:** FR-1, FR-9, AC-5.3, AC-6.2
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/mod.rs`, `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_window_manager_new_default` -- Verify `X11WindowManager::new()` returns an instance with `raw_window_handle()` returning `None`
   - [ ] `x11_window_manager_raw_display_handle_before_create` -- Verify `raw_display_handle()` returns `None` before `create_overlay`
   - [ ] `x11_window_manager_overlay_window_id_before_create` -- Verify `overlay_window_id()` returns `None` before `create_overlay`
2. **Green** -- Implement minimum code to pass:
   - [ ] Create `crates/luminos-platform/src/linux_x11/window.rs` with `X11WindowManager` struct (window: `Option<Window>`, current_mode, display_bounds)
   - [ ] Implement `X11WindowManager::new()` returning default state
   - [ ] Implement `raw_window_handle()` and `raw_display_handle()` returning `None` when window is absent
   - [ ] Implement `overlay_window_id()` extracting X11 window ID from `RawWindowHandle::Xlib` or `RawWindowHandle::Xcb` (returns `None` when window is absent)
   - [ ] Add `pub mod window;` to `linux_x11/mod.rs`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments to struct and all methods

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 1, verify:
- [ ] `cargo build --workspace` compiles
- [ ] `DockEdge`/`LensShape` serde roundtrip tests pass
- [ ] All existing E01 tests still pass
- [ ] `RenderError` display message tests pass
- [ ] `X11WindowManager::new()` tests pass

---

## Phase 2: Core Implementation

### T004 -- Implement create_gpu_device() in luminos-gpu

**Traces to:** FR-2, AC-2.1, AC-2.4
**Status:** TODO
**Files:** `crates/luminos-gpu/src/device.rs`, `crates/luminos-gpu/src/lib.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `gpu_device_create_success` -- On wgpu GL backend (headless), verify `create_gpu_device()` returns Ok with device and queue
   - [ ] `gpu_device_adapter_low_power` -- Verify the adapter request uses `LowPower` preference (test by inspecting the function's behavior with available backends)
2. **Green** -- Implement minimum code to pass:
   - [ ] Create `crates/luminos-gpu/src/device.rs` with `create_gpu_device()` per DESIGN.md
   - [ ] Add `pub mod device;` to `lib.rs`
   - [ ] Add required dependencies (`wgpu`, `log`) to `luminos-gpu/Cargo.toml` if not already present
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add comprehensive doc-comments with `# Errors` section

**Completion Notes:**
>

---

### T005 -- Implement configure_surface() in luminos-gpu

**Traces to:** FR-3, AC-2.2, AC-2.3
**Status:** TODO
**Files:** `crates/luminos-gpu/src/surface.rs`, `crates/luminos-gpu/src/lib.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `surface_configure_srgb_preferred` -- On GL backend headless, verify `configure_surface()` returns an sRGB format when available
   - [ ] `surface_configure_alpha_mode_fallback` -- Verify fallback logic when `PreMultiplied` is unavailable (unit test with mocked capabilities)
   - [ ] `surface_configure_minimum_dimensions` -- Verify width/height are clamped to at least 1 (zero-size surfaces are invalid)
2. **Green** -- Implement minimum code to pass:
   - [ ] Create `crates/luminos-gpu/src/surface.rs` with `configure_surface()` per DESIGN.md
   - [ ] Add `pub mod surface;` to `lib.rs`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Extract alpha mode selection into a helper function for testability

**Completion Notes:**
>

---

### T006 -- Implement X11WindowManager::create_overlay()

**Traces to:** FR-1, FR-4, FR-9, AC-1.1, AC-6.1
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_window_manager_create_overlay_invalid_display` -- Verify `create_overlay("nonexistent")` returns `WindowError::DisplayNotFound`
   - [ ] `x11_window_manager_create_overlay_success` -- (Integration, requires Xvfb) Verify `create_overlay(valid_id)` succeeds and `raw_window_handle()` returns `Some`
   - [ ] `x11_window_manager_overlay_window_id_after_create` -- (Integration, requires Xvfb) Verify `overlay_window_id()` returns `Some(id)` with non-zero value after `create_overlay`
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `create_overlay`: create winit window with transparent, borderless, always-on-top, and `with_override_redirect(true)` attributes
   - [ ] Store the window in `self.window`
   - [ ] Store display bounds in `self.display_bounds`
   - [ ] Handle winit event loop integration (may require `ActiveEventLoop` reference at construction or a factory pattern)
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Extract window attribute configuration into a helper function

**Completion Notes:**
>

---

### T007 -- Implement set_overlay_bounds, set_visible, set_always_on_top

**Traces to:** FR-4, AC-1.2, AC-1.3, AC-1.4
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_window_manager_set_bounds_no_window` -- Verify `set_overlay_bounds` returns error when no overlay exists
   - [ ] `x11_window_manager_set_visible_no_window` -- Verify `set_visible` returns error when no overlay exists
   - [ ] `x11_window_manager_set_always_on_top_success` -- (Integration) Verify `set_always_on_top(true)` succeeds after `create_overlay`
   - [ ] `x11_window_manager_set_visible_success` -- (Integration) Verify `set_visible(true/false)` succeeds after `create_overlay`
   - [ ] `x11_window_manager_set_bounds_success` -- (Integration) Verify `set_overlay_bounds` succeeds after `create_overlay`
2. **Green** -- Implement minimum code to pass:
   - [ ] `set_overlay_bounds`: use `window.request_inner_size()` and `window.set_outer_position()`
   - [ ] `set_visible`: use `window.set_visible()`
   - [ ] `set_always_on_top`: use `window.set_window_level()`
   - [ ] All methods return `WindowError::PropertyFailed` when `self.window` is `None`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Extract "get window or error" helper to reduce boilerplate

**Completion Notes:**
>

---

### T008 -- Implement set_overlay_mode() (FullScreen only, E02 scope)

**Traces to:** FR-5, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `x11_window_manager_set_mode_fullscreen` -- (Integration) Verify FullScreen mode sets window to full display size
   - [ ] `x11_window_manager_set_mode_no_window` -- Verify `set_overlay_mode` returns error when no overlay exists
   - [ ] `x11_window_manager_set_mode_non_fullscreen_rejected` -- Verify Docked/Lens modes return `WindowError::PropertyFailed` (deferred to E05)
2. **Green** -- Implement minimum code to pass:
   - [ ] FullScreen: set window bounds to full display
   - [ ] Docked/Lens: return `WindowError::PropertyFailed` with message indicating E05 scope
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Verify mode state is updated in `self.current_mode` only on success

**Completion Notes:**
>

---

### T009 -- Implement wgpu instance creation helper

**Traces to:** FR-2, AC-2.1
**Status:** TODO
**Files:** `crates/luminos-gpu/src/device.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `gpu_instance_create_default` -- Verify `create_wgpu_instance()` returns a valid instance (does not panic)
   - [ ] `gpu_instance_vulkan_or_gl` -- Verify the instance is created with Vulkan primary and GL fallback backends
2. **Green** -- Implement minimum code to pass:
   - [ ] Add `create_wgpu_instance()` function that creates `wgpu::Instance` with `wgpu::InstanceDescriptor` using `Backends::VULKAN | Backends::GL` on Linux
   - [ ] Use `wgpu::Dx12Compiler::default()` and `wgpu::Gles3MinorVersion::default()`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Add doc-comments explaining backend selection rationale

**Completion Notes:**
>

---

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [ ] All Phase 1 + Phase 2 tests pass
- [ ] `cargo clippy -p luminos-platform -p luminos-gpu -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` passes
- [ ] `cargo build --workspace` compiles cleanly
- [ ] No `unwrap()` or `expect()` in production code paths

---

## Phase 3: Integration

### T010 -- Integration test: window creation + wgpu device + surface on Xvfb

**Traces to:** AC-1.1, AC-2.1, AC-2.2, AC-2.3, AC-5.1, AC-5.2
**Status:** TODO
**Files:** `crates/luminos-gpu/tests/integration_window_gpu.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `integration_overlay_window_with_gpu_surface` -- Create X11WindowManager, create_overlay, get raw handles, create wgpu instance + device + surface, configure surface, acquire texture -- verify each step succeeds
2. **Green** -- Implement minimum code to pass:
   - [ ] Wire together all components: `X11WindowManager::new()` -> `create_overlay()` -> `raw_window_handle/display_handle` -> `wgpu::Instance::create_surface()` -> `create_gpu_device()` -> `configure_surface()`
   - [ ] Test requires `#[cfg(target_os = "linux")]` gate and Xvfb + Mesa llvmpipe in CI
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Extract integration test helpers into a shared test utilities module

**Completion Notes:**
>

---

### T011 -- Integration test: FullScreen overlay mode on Xvfb

**Traces to:** AC-3.1
**Status:** TODO
**Files:** `crates/luminos-platform/tests/integration_overlay_mode.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `integration_overlay_mode_fullscreen` -- Create overlay, set FullScreen mode, verify no error and window covers display
2. **Green** -- Implement minimum code to pass:
   - [ ] Tests exercise the `X11WindowManager` FullScreen mode on Xvfb
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Factor common Xvfb test setup into a shared helper

**Completion Notes:**
>

---

### T012 -- Verify type unification does not break existing tests

**Traces to:** AC-4.3
**Status:** TODO
**Files:** (no new files -- runs existing test suite)

**Steps:**
1. [ ] Run `cargo nextest run --workspace --exclude luminos-app` and verify all tests pass
2. [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` and verify clean
3. [ ] Run `cargo fmt --all -- --check` and verify clean
4. [ ] Run `cargo deny check licenses advisories` and verify clean (serde addition should not introduce new license concerns)

**Completion Notes:**
>

---

## Phase 4: Polish & Acceptance

### T013 -- Doc-comments and clippy clean pass

**Traces to:** NFR-3, NFR-4, NFR-5
**Status:** TODO
**Files:** All files created/modified in this story

**Steps:**
1. [ ] Verify all public items in `linux_x11/window.rs`, `luminos-gpu/error.rs`, `luminos-gpu/device.rs`, `luminos-gpu/surface.rs` have `///` doc-comments
2. [ ] Run `cargo doc -p luminos-platform -p luminos-gpu --no-deps` and verify no warnings
3. [ ] Run full clippy command and fix any remaining warnings
4. [ ] Verify no `unwrap()` or `expect()` in production code (search with `grep -rn "unwrap()\|expect(" crates/luminos-platform/src/linux_x11/ crates/luminos-gpu/src/ --include="*.rs"` excluding test modules)

**Completion Notes:**
>

---

### T014 -- Acceptance test verification

**Traces to:** All ACs
**Status:** TODO

**Verification Checklist:**
- [ ] AC-1.1: Overlay window created on Xvfb (transparent, borderless, always-on-top)
- [ ] AC-1.2: `set_visible(true/false)` works without error
- [ ] AC-1.3: `set_always_on_top(true)` works without error
- [ ] AC-1.4: `set_overlay_bounds` repositions and resizes the window
- [ ] AC-2.1: `create_gpu_device` returns device and queue on llvmpipe
- [ ] AC-2.2: `configure_surface` returns sRGB format with PreMultiplied alpha (or fallback)
- [ ] AC-2.3: `get_current_texture` returns valid texture
- [ ] AC-2.4: `RenderError::NoAdapter` returned when no GPU available
- [ ] AC-3.1: FullScreen mode covers entire display
- [ ] AC-4.1: `DockEdge` unified with serde, re-exported from luminos-core
- [ ] AC-4.2: `LensShape` unified with serde, re-exported from luminos-core
- [ ] AC-4.3: All existing tests pass after unification
- [ ] AC-4.4: `OverlayMode` serializes/deserializes correctly
- [ ] AC-5.1: `raw_window_handle` returns Some after create_overlay
- [ ] AC-5.2: `raw_display_handle` returns Some after create_overlay
- [ ] AC-5.3: Both handles return None before create_overlay
- [ ] AC-6.1: `overlay_window_id()` returns Some(u64) with non-zero value after create_overlay
- [ ] AC-6.2: `overlay_window_id()` returns None before create_overlay
- [ ] All clippy warnings resolved
- [ ] No `unwrap()` in production code paths
- [ ] `cargo fmt --all -- --check` passes

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
