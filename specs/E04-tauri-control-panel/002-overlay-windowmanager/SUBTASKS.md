# Subtasks: Story E04/002 -- Overlay WindowManager (winit→tao) & Self-Capture

**Status:** NOT STARTED
**Started:** ---
**Completed:** ---
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 2 | 0 | 0 | 2 |
| 2. Core Implementation (x11rb backend) | 5 | 0 | 0 | 5 |
| 3. Integration (bridge + self-capture) | 3 | 0 | 0 | 3 |
| 4. Polish & Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **11** | **0** | **0** | **11** |

> Self-capture mechanism is selected at implementation time (DESIGN candidates A→C→B); record the choice + fallback in completion notes + epic Shared Context.

---

## Phase 1: Setup

### T001 -- Strip winit from the overlay backend; module scaffold
**Traces to:** FR-1, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`, `crates/luminos-platform/Cargo.toml`

**TDD Cycle:** (setup)
1. **Green:**
   - [ ] Remove the winit `EventLoop`/`with_override_redirect` code path; reduce `X11WindowManager` to an x11rb-backed struct skeleton per DESIGN.
   - [ ] Remove the `winit` dependency from `luminos-platform/Cargo.toml` if unused elsewhere; confirm via `cargo tree -p luminos-platform` that neither `winit` nor `tauri` is a dependency.
2. **Refactor:**
   - [ ] Update the `WindowManager` trait doc comment (drop "winit-based").

**Completion Notes:**
>

---

### T002 -- x11rb connection + bind to overlay XID
**Traces to:** FR-2
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `x11_window_manager_new_binds_xid` (Xvfb) -- create a test X11 window, `X11WindowManager::new(xid, bounds)` succeeds; `overlay_window_id()` == that xid.
2. **Green:**
   - [ ] Implement `new`/`create_overlay`/`overlay_window_id` with a `RustConnection`.
3. **Refactor:**
   - [ ] Map x11rb errors → `WindowError`.

**Completion Notes:**
>

---

## Phase 2: Core Implementation (x11rb backend)

### T003 -- `set_overlay_bounds` via ConfigureWindow
**Traces to:** FR-2, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red:** `x11_window_manager_set_bounds_applies` (Xvfb) -- after `set_overlay_bounds(rect)`, `GetGeometry` == rect.
2. **Green:** Implement `ConfigureWindow` (x/y/width/height).
3. **Refactor:** Clamp/validate rect; single-quote logs.

**Completion Notes:**
>

---

### T004 -- `set_always_on_top` via `_NET_WM_STATE_ABOVE`
**Traces to:** FR-4, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red:** `x11_window_manager_always_on_top_sets_state` -- assert `_NET_WM_STATE_ABOVE` present after `set_always_on_top(true)`, absent after `false`.
2. **Green:** Send EWMH client message / `ChangeProperty` for `_NET_WM_STATE_ABOVE`.
3. **Refactor:** Atom interning helper.

**Completion Notes:**
>

---

### T005 -- `set_visible` via Map/Unmap
**Traces to:** FR-4, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red:** `x11_window_manager_visible_maps_unmaps` -- map-state toggles per `set_visible`.
2. **Green:** `MapWindow`/`UnmapWindow` + flush.
3. **Refactor:** —

**Completion Notes:**
>

---

### T006 -- `set_overlay_mode(FullScreen)` + Lens/Docked deferral
**Traces to:** FR-3, AC-1.2
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `x11_window_manager_fullscreen_sizes_to_display` -- geometry == display bounds.
   - [ ] `x11_window_manager_lens_docked_deferred` -- `Lens`/`Docked` return `Ok` + warn.
2. **Green:** FullScreen → `set_overlay_bounds(display_bounds)`; Lens/Docked → `Ok(()) + warn!("... deferred to E5")`.
3. **Refactor:** —

**Completion Notes:**
>

---

### T007 -- Handle methods return None; trait unchanged
**Traces to:** FR-6, AC-1.2
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`, `crates/luminos-platform/src/traits/window_manager.rs` (doc only)

**TDD Cycle:**
1. **Red:**
   - [ ] `x11_window_manager_handles_return_none` -- `raw_window_handle()`/`raw_display_handle()` == `None`.
   - [ ] Existing `window_manager.rs` trait tests still pass (run them).
2. **Green:** Return `None`; document why (surface sourced in luminos-app).
3. **Refactor:** Flag trait-cleanup deviation in epic Deviations.

**Checkpoint:** Phases 1-2 pass; manager controls a bound X11 window fully via x11rb; no winit; no tauri dep in luminos-platform.

**Completion Notes:**
>

---

## Phase 3: Integration (bridge + self-capture)

### T008 -- `luminos-app` overlay XID bridge
**Traces to:** FR-8, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-app/src/overlay_bridge.rs`, `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:** `app_logs_overlay_xid` (subprocess, X11) -- app logs a non-zero `overlay_xid=…` at startup.
2. **Green:** `extract_overlay_xid(&WebviewWindow)` via `gtk_window()` → gdk → XID; construct `X11WindowManager`; store in app state for later stories.
3. **Refactor:** Map GTK/gdk failures → `AppError`.

**Completion Notes:**
>

---

### T009 -- Self-capture via shipped `set_excluded_windows` + expose XID
**Traces to:** FR-5, FR-7, AC-2.1, AC-2.2
**Status:** TODO
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`, `crates/luminos-app/**`

**TDD Cycle:**
1. **Red:**
   - [ ] `overlay_window_id_exposed` -- `overlay_window_id()` returns the bound XID for the capture path.
   - [ ] `self_capture_excludes_overlay` (integration) -- with `ScreenCapture::set_excluded_windows(&[overlay_xid])` applied (shipped `XcbCapture` unmap/remap), the overlay's known pattern is absent from a captured frame.
2. **Green:** Surface `overlay_window_id()`; **do not reimplement exclusion** — the existing `XcbCapture::set_excluded_windows` handles it. The actual `set_excluded_windows(&[xid])` call happens in story 003 against the render loop's capture instance (see P-002 / epic Shared Context).
3. **Refactor:** Record observed flicker behavior (NFR-2) as a RISK-002 finding in completion notes + epic Shared Context; future no-flicker optimizations are post-E04.

**Completion Notes:**
>

> Note: AC-2.2 ("documented fallback") here means: if the shipped unmap/remap is unstable/flickers, that is logged as a RISK-002 finding (the optimization is deferred) — there is no second exclusion mechanism to implement in E04.

**Completion Notes:**
>

---

### T010 -- Geometry/visibility/stacking subprocess verification
**Traces to:** AC-1.1
**Status:** TODO
**Files:** `crates/luminos-app/tests/overlay_control.rs`

**TDD Cycle:**
1. **Red:** Drive trait methods on the real overlay (subprocess); assert via `xprop`/`xwininfo` (bounds, `_NET_WM_STATE_ABOVE`, map state). Skip gracefully if tools absent.
2. **Green:** Wire a debug command path to invoke the manager.
3. **Refactor:** Reuse story-001 `tests/common/` harness.

**Checkpoint:** Overlay controllable end-to-end through the trait from the running app; self-capture mitigation active.

**Completion Notes:**
>

---

## Phase 4: Polish & Acceptance

### T011 -- Acceptance verification + AC matrix
**Traces to:** All ACs
**Status:** TODO
**Files:** story docs

**Verification Checklist:**
- [ ] AC-1.1 geometry/visibility/stacking via x11rb
- [ ] AC-1.2 FullScreen sizing + trait unchanged (existing tests pass) + Lens/Docked deferral
- [ ] AC-2.1 no self-capture feedback (chosen mechanism)
- [ ] AC-2.2 fallback logged, no panic
- [ ] AC-3.1 winit-free + no `tauri` dep in luminos-platform (`cargo tree`) + XID bridge works
- [ ] `cargo fmt`/clippy clean; no `unwrap`/`expect`
- [ ] Self-capture mechanism + fallback recorded in epic Shared Context
- [ ] `raw_window_handle` trait-cleanup logged in epic Deviations

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
