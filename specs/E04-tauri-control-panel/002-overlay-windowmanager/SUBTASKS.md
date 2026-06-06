# Subtasks: Story E04/002 -- Overlay WindowManager (winit→tao) & Self-Capture

**Status:** DONE
**Started:** 2026-06-05
**Completed:** 2026-06-05
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 2 | 2 | 0 | 0 |
| 2. Core Implementation (x11rb backend) | 5 | 5 | 0 | 0 |
| 3. Integration (bridge + self-capture) | 3 | 3 | 0 | 0 |
| 4. Polish & Acceptance | 1 | 1 | 0 | 0 |
| **Total** | **11** | **11** | **0** | **0** |

> **Self-capture mechanism chosen:** the SHIPPED `XcbCapture::set_excluded_windows(&[overlay_xid])` unmap/remap (DC-6). Story 002 only surfaces the XID (`overlay_window_id()` on the manager + `LuminosHandle`); story 003 wires it into the render-loop capture instance. **Fallback (AC-2.2):** if unmap/remap flickers under tao/GTK3, log a RISK-002 finding (optimization deferred post-E04) — no second mechanism in E04.

### Test counts (delta vs the ~470 baseline)
- Workspace ex-app (ci profile, no display): **436 passed, 3 skipped**.
- `luminos-app` (ci profile, -j2, Xvfb): **28 passed** (was 23 in story 001; +3 `overlay_control` subprocess, +2 in-process `overlay_bridge`/`app_error::bridge`).
- `luminos-platform` Xvfb window tests (`:88`): **14 passed**.
- **Net rewrites (no net loss):** removed 5 stale in-module winit tests (`*_no_window` ×3, `set_overlay_mode_fullscreen_no_bounds`, `new_default`/`raw_display_handle_before_create`/`overlay_window_id_before_create` premised on a window-less default and `is_some()`-after-create) — they tested winit-specific preconditions that no longer exist (the manager binds an XID at construction; there is no "no window" state). Replaced by 8 in-module Xvfb integration tests + 6 rewritten `integration_overlay_mode.rs` tests asserting real X11 state.
- **Capture integration tests** (`xcb_capture_*`) and one app `overlay_no_compositor` test fail ONLY under this dev box's high-parallelism software-Xvfb (xcap mis-selects the Wayland backend; webkit spawn contention) — all pass in isolation / under the `ci` profile's retries, and `capture.rs` is byte-identical to HEAD (not touched by this story). Not a regression.

---

## Phase 1: Setup

### T001 -- Strip winit from the overlay backend; module scaffold
**Traces to:** FR-1, AC-3.1
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`, `crates/luminos-platform/Cargo.toml`

**TDD Cycle:** (setup)
1. **Green:**
   - [x] Removed the winit `EventLoop`/`with_override_redirect` code path; `X11WindowManager` is now an x11rb-backed struct (`RustConnection` + `root` + `overlay_xid: u32` + `display_bounds` + `current_mode`).
   - [x] Removed `winit` from `luminos-platform/Cargo.toml` (it was only used here). `cargo tree -p luminos-platform` shows 0 `winit` and 0 `tauri`.
2. **Refactor:**
   - [x] Updated the `WindowManager` trait doc (dropped "winit-based"; documents the x11rb-over-bound-XID model + `raw_*_handle()==None`).

**Completion Notes:**
> winit fully removed from `luminos-platform` (verified `cargo tree -p luminos-platform | grep -ci winit` == 0; same for tauri). The struct holds a `RustConnection` (a socket, not a loop) so FR-1 holds. Trait surface unchanged. See Deviations #4 (create_overlay contract shift).

---

### T002 -- x11rb connection + bind to overlay XID
**Traces to:** FR-2
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red:**
   - [x] `x11_window_manager_new_binds_xid` (Xvfb) -- creates a throwaway x11rb window, `X11WindowManager::new(xid, bounds)` succeeds; `overlay_window_id()` == `Some(u64::from(xid))`.
   - [x] `x11_window_manager_create_overlay_resolves_bounds` (Xvfb) -- `create_overlay(display_id)` resolves a real monitor's bounds and leaves the bound XID unchanged (creates nothing, FR-2).
2. **Green:**
   - [x] Implemented `new(xid, bounds)` (opens `RustConnection`, records root+xid+bounds), `create_overlay` (resolves bounds only), `overlay_window_id() -> Option<u64>`.
3. **Refactor:**
   - [x] All x11rb errors mapped → `WindowError::Platform { message }` via `.map_err`; `intern`/`flush` helpers added.

**Completion Notes:**
> `overlay_window_id()` returns `u64` (Deviation #2) to match `set_excluded_windows(&[u64])`. `new()` connects via ambient `$DISPLAY` — same per-process connection reused for all control requests (contrast: capture's per-frame connect smell, recorded for story 003).

---

## Phase 2: Core Implementation (x11rb backend)

### T003 -- `set_overlay_bounds` via ConfigureWindow
**Traces to:** FR-2, AC-1.1
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red:** [x] `x11_window_manager_set_bounds_applies` (Xvfb) -- after `set_overlay_bounds(rect)`, `GetGeometry` reports x/y/width/height == rect.
2. **Green:** [x] `configure_window` with `ConfigureWindowAux::new().x().y().width().height()` + flush.
3. **Refactor:** [x] Single-quoted dynamic log values; error mapped to `Platform`.

**Completion Notes:**
> Geometry verified end-to-end via `GetGeometry` against a throwaway window under Xvfb. x/y are root-relative (no reparenting WM in CI), matching the bound rect exactly.

---

### T004 -- `set_always_on_top` via `_NET_WM_STATE_ABOVE`
**Traces to:** FR-4, AC-1.1
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red:** [x] `x11_window_manager_always_on_top_sets_state` -- `_NET_WM_STATE_ABOVE` member present after `set_always_on_top(true)`, absent after `false` (read via `GetProperty`).
2. **Green:** [x] Sends EWMH `_NET_WM_STATE` ClientMessage (action ADD/REMOVE, atom `_NET_WM_STATE_ABOVE`, source=1) to root with `SUBSTRUCTURE_NOTIFY|SUBSTRUCTURE_REDIRECT`, AND directly sets/clears the `_NET_WM_STATE` property so a WM-less Xvfb is observable (per §E).
3. **Refactor:** [x] `intern()` atom helper; flush after mutation.

**Completion Notes:**
> Under WM-less Xvfb no WM enforces stacking, so the test asserts the PROPERTY membership (per §E), not actual restack. Both the EWMH ClientMessage (authoritative under a real WM) and the direct property write are issued.

---

### T005 -- `set_visible` via Map/Unmap
**Traces to:** FR-4, AC-1.1
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`

**TDD Cycle:**
1. **Red:** [x] `x11_window_manager_visible_maps_unmaps` -- `GetWindowAttributes.map_state` == `VIEWABLE` after `set_visible(true)`, `UNMAPPED` after `set_visible(false)`.
2. **Green:** [x] `map_window`/`unmap_window` + flush.
3. **Refactor:** [x] Errors mapped to `Platform`; single-quoted logs.

**Completion Notes:**
> Straightforward Map/Unmap on the bound XID, verified via map-state under Xvfb.

---

### T006 -- `set_overlay_mode(FullScreen)` + Lens/Docked deferral
**Traces to:** FR-3, AC-1.2
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`, `crates/luminos-platform/tests/integration_overlay_mode.rs`

**TDD Cycle:**
1. **Red:**
   - [x] `x11_window_manager_fullscreen_sizes_to_display` -- geometry == display bounds.
   - [x] `x11_window_manager_lens_docked_deferred` -- `Lens`/`Docked` return `Ok` + warn AND do not resize the window.
2. **Green:** [x] FullScreen → `set_overlay_bounds(display_bounds)`; Lens/Docked → `Ok(()) + warn!("... deferred to E05")`.
3. **Refactor:** [x] Combined Lens/Docked arms; rewrote the 5 stale `is_err` tests (in-module + `integration_overlay_mode.rs`) to assert `Ok` + no-resize.

**Completion Notes:**
> **Deviation #3 (DESIGN-mandated):** ADOPTED `Ok(()) + warn!`. REWROTE the 5 tests that previously asserted `Err`: in-module `x11_window_manager_set_overlay_mode_{docked,lens}_rejected` → deleted/folded into `x11_window_manager_lens_docked_deferred`; `integration_overlay_mode.rs` `*_docked_rejected`/`*_lens_rejected` → `*_docked_deferred`/`*_lens_deferred`. The pure-logic `set_overlay_mode_fullscreen_no_bounds` test (which relied on the old `Option<display_bounds>`) is removed: `display_bounds` is now a required non-Option field set at construction.

---

### T007 -- Handle methods return None; trait unchanged
**Traces to:** FR-6, AC-1.2
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`, `crates/luminos-platform/src/traits/window_manager.rs` (doc only)

**TDD Cycle:**
1. **Red:**
   - [x] `x11_window_manager_handles_return_none` -- `raw_window_handle()`/`raw_display_handle()` == `None` (AD-3).
   - [x] Existing `window_manager.rs` trait tests still pass unchanged (trait surface preserved, FR-6).
2. **Green:** [x] Both return `None`; doc explains the surface is sourced by `luminos-app`'s `OverlayGpu`.
3. **Refactor:** [x] Trait-cleanup deviation logged (Deviation #6).

**Checkpoint:** PASSED -- manager controls a bound X11 window fully via x11rb; no winit; no tauri dep in luminos-platform; 14 X11 window tests pass under Xvfb; 436 workspace tests (ex-app) pass.

**Completion Notes:**
> **Deviation #6:** This INVERTS the old winit behavior (`Some` after create). Now ALWAYS `None`. The trait still declares `Option<&dyn HasWindowHandle>` (FR-6: unchanged signature); a future trait cleanup (drop these methods or move surface-sourcing fully to the app) is flagged for the Phase-0 gate. Also rewrote the old in-module tests that asserted `raw_window_handle().is_some()` after create (`window.rs:464-465`).

---

## Phase 3: Integration (bridge + self-capture)

### T008 -- `luminos-app` overlay XID bridge
**Traces to:** FR-8, AC-3.1
**Status:** DONE
**Files:** `crates/luminos-app/src/overlay_bridge.rs`, `crates/luminos-app/src/app.rs`, `crates/luminos-app/src/app_error.rs`, `crates/luminos-app/src/handle.rs`, `crates/luminos-app/src/lib.rs`, `crates/luminos-app/tests/overlay_control.rs`

**TDD Cycle:**
1. **Red:** [x] `app_logs_overlay_xid` (subprocess, X11) -- app logs a non-zero `overlay_xid=…` AND `windowmanager_bound` at startup. Also `overlay_bridge_error_display` + `app_error_bridge_carries_message` (in-process).
2. **Green:** [x] `extract_overlay_xid(&WebviewWindow)` via `raw_window_handle::HasWindowHandle` → `RawWindowHandle::Xlib{window}`/`Xcb` (NOT gtk_window/gdk — Deviation #1); `build_window_manager` constructs `X11WindowManager::new(xid, bounds)`; wired at `RunEvent::Ready` (`init_window_manager`); stored on `LuminosHandle.window_manager`.
3. **Refactor:** [x] Added `AppError::Bridge(String)`; XID/connection failures map to it.

**Completion Notes:**
> Verified live: `overlay_xid=4194329 (extracted from overlay raw-window-handle)`, `windowmanager_bound`, `overlay_window_id_exposed=4194329 (reachable via LuminosHandle for story 003)`. **Deviation #1:** chose the rwh path over FR-8's literal `gtk_window()→gdk→gdk_x11_window_get_xid` (main-thread-only, extra unsafe FFI; rwh reuses the same handle wgpu consumes). **Deviation (handle):** `LuminosHandle` stores the concrete `X11WindowManager` (Linux-gated, behind `Mutex`), not `Box<dyn WindowManager>`, because `overlay_window_id()` is inherent to the backend, not the trait (FR-6 keeps the trait unchanged).

---

### T009 -- Self-capture via shipped `set_excluded_windows` + expose XID
**Traces to:** FR-5, FR-7, AC-2.1, AC-2.2
**Status:** DONE
**Files:** `crates/luminos-platform/src/linux_x11/window.rs`, `crates/luminos-app/src/app.rs`, `crates/luminos-app/src/handle.rs`, `crates/luminos-app/tests/overlay_control.rs`

**TDD Cycle:**
1. **Red:**
   - [x] `x11_window_manager_new_binds_xid` + `LuminosHandle::overlay_window_id()` -- the bound XID is reachable for the capture path (via the manager and via the handle).
   - [x] `app_self_capture_hook_runs_without_panic` (subprocess) -- with `LUMINOS_SELF_CAPTURE_PROBE=1`, the app calls `set_excluded_windows(&[overlay_xid])` on a real `XcbCapture` and attempts a capture; logs `self_capture_probe=`; exits cleanly (no panic).
2. **Green:** [x] Surfaced `overlay_window_id() -> Option<u64>` on the manager and on `LuminosHandle`; **no exclusion code written** — exercised the shipped `XcbCapture::set_excluded_windows` hook in `probe_self_capture`. The real render-loop wiring is story 003.
3. **Refactor:** [x] RISK-002 finding recorded (below + epic Shared Context).

**Completion Notes:**
> **RISK-002 finding (NFR-2):** the hook ran end-to-end without panic. Observed live (this dev box): `Self-capture exclusion active for '1' window(s)` then `self_capture_probe=capture_failed … 'Cannot find required wayland protocol'` — xcap 0.9.4 mis-selects the Wayland (libwayshot) backend under headless Xvfb, so the actual frame grab + flicker observation could NOT be made here (same env limitation as the pre-existing `capture` integration tests; they pass in CI's `xvfb-run`+picom X11 harness, not this dev box). The exclusion mechanism is unmap/remap around capture (DC-6), so **flicker is the documented expected cost under tao/GTK3** — flicker-free optimization is post-E04. AC-2.2 fallback = log the finding, no panic (satisfied).

> Note: AC-2.2 ("documented fallback") here means: if the shipped unmap/remap is unstable/flickers, that is logged as a RISK-002 finding (the optimization is deferred) — there is no second exclusion mechanism to implement in E04.

---

### T010 -- Geometry/visibility/stacking subprocess verification
**Traces to:** AC-1.1
**Status:** DONE
**Files:** `crates/luminos-app/tests/overlay_control.rs`

**TDD Cycle:**
1. **Red:** [x] `app_overlay_window_is_bound_and_mapped` (subprocess) -- the bound overlay XID (from the `overlay_xid=` log) is a real mapped window in the X11 tree.
2. **Green:** [x] The bridge binds the manager at `Ready`; the X11 query harness confirms the bound XID == a mapped "Overlay" window.
3. **Refactor:** [x] Reused the story-001 `tests/common/` x11rb query harness (`find_windows`/`query_tree`) — **NOT** `xprop`/`xwininfo` (Deviation #7; both absent on the box, per §E).

**Checkpoint:** PASSED -- the overlay is controllable end-to-end through the trait from the running app (XID extracted, manager bound, reachable via `LuminosHandle`), and the self-capture hook is exercised. The geometry/visibility/stacking trait METHODS are verified by the platform-crate Xvfb unit tests (T003-T006) against the exact same x11rb requests; story 003 drives them on the live overlay per redraw.

**Completion Notes:**
> **Deviation #5/#7:** the geometry/stacking *assertions* live in the platform crate's Xvfb unit tests (`window.rs` integration + `integration_overlay_mode.rs`), which exercise the identical `configure_window`/`_NET_WM_STATE`/map requests against a throwaway window with `GetGeometry`/`GetProperty` — stronger and faster than driving them through a subprocess. The subprocess test proves the bridge binds the *real* overlay window (the seam story 003 needs); it does not re-drive every trait method through the app because no debug command path exists yet (story 005 adds IPC). `xprop`/`xwininfo` were NOT used (DESIGN named them but both are absent; the x11rb harness supersedes — Deviation #7).

---

## Phase 4: Polish & Acceptance

### T011 -- Acceptance verification + AC matrix
**Traces to:** All ACs
**Status:** DONE
**Files:** story docs, `../HIGH_LEVEL_PLAN.md`

**Verification Checklist:**
- [x] AC-1.1 geometry/visibility/stacking via x11rb -- `x11_window_manager_set_bounds_applies`, `_always_on_top_sets_state`, `_visible_maps_unmaps`, `integration_overlay_geometry_stacking_visibility` (GetGeometry/GetProperty/map-state).
- [x] AC-1.2 FullScreen sizing + trait unchanged + Lens/Docked deferral -- `x11_window_manager_fullscreen_sizes_to_display`, `_lens_docked_deferred`, `integration_overlay_mode_*`; all `window_manager.rs` trait tests pass unchanged.
- [x] AC-2.1 self-capture XID surfaced + hook exercised -- `overlay_window_id()` on manager + `LuminosHandle`; `app_self_capture_hook_runs_without_panic`.
- [x] AC-2.2 fallback logged, no panic -- `self_capture_probe=` finding logged; clean exit.
- [x] AC-3.1 winit-free + no `tauri` dep (`cargo tree -p luminos-platform` → 0 winit, 0 tauri) + XID bridge works (`app_logs_overlay_xid`, `_handles_return_none`).
- [x] `cargo fmt`/clippy clean (workspace ex-app + app, both feature sets); no `unwrap`/`expect` in production.
- [x] Self-capture mechanism + fallback recorded in epic Shared Context (P-002 below).
- [x] `raw_window_handle` trait-cleanup logged in epic Deviations + this story's Deviation #6.

**AC → Test Matrix:**

| AC | Tests |
|----|-------|
| AC-1.1 | `window.rs::integration::x11_window_manager_set_bounds_applies`, `_always_on_top_sets_state`, `_visible_maps_unmaps`; `integration_overlay_mode::integration_overlay_geometry_stacking_visibility`, `_fullscreen_then_visible`; `overlay_control::app_overlay_window_is_bound_and_mapped` |
| AC-1.2 | `window.rs::integration::x11_window_manager_fullscreen_sizes_to_display`, `_lens_docked_deferred`, `_create_overlay_resolves_bounds`; `integration_overlay_mode::integration_overlay_mode_fullscreen_sizes_to_display`, `_docked_deferred`, `_lens_deferred`; all `traits::window_manager::tests::*` (unchanged) |
| AC-2.1 | `window.rs::integration::x11_window_manager_new_binds_xid`; `handle::overlay_window_id`; `overlay_control::app_self_capture_hook_runs_without_panic` |
| AC-2.2 | `overlay_control::app_self_capture_hook_runs_without_panic` (logs `self_capture_probe=`, clean exit) |
| AC-3.1 | `window.rs::integration::x11_window_manager_handles_return_none`; `overlay_bridge::tests::overlay_bridge_error_display`; `app_error::tests::app_error_bridge_carries_message`; `overlay_control::app_logs_overlay_xid`; `cargo tree -p luminos-platform` (build gate) |

**Completion Notes:**
> All 5 ACs covered by ≥1 passing test. Test counts: platform window tests 14/14 (Xvfb :88); app suite 28/28 (ci profile, -j2); workspace ex-app 436/436. Quality gates: fmt clean, clippy clean (both invocations), `cargo deny`/`cargo audit` green, `cargo tree` shows no winit/no tauri in luminos-platform. Test-count delta vs the 470 baseline explained in Progress Summary.

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

All 7 corrections from IMPLEMENTATION_NOTES §C applied + 2 mechanical consequences logged.

| Task | Deviation | Rationale |
|------|-----------|-----------|
| T008 (FR-8) | XID extracted via `raw_window_handle::HasWindowHandle` → `RawWindowHandle::Xlib{window}`/`Xcb`, **not** FR-8's literal `gtk_window()→gdk→gdk_x11_window_get_xid`. | The rwh path is main-thread-safe, needs no extra unsafe GTK FFI, and reuses the exact handle wgpu already consumes for the surface. (§A.1, §C.7-adjacent) |
| T002 (correction #1) | Overlay **label = `"overlay"`**, title `"Luminos Overlay"`; bridge uses `get_webview_window("overlay")`; the X11 harness matches WM_NAME substring `"Overlay"`. | Matches the real `app.rs` `OVERLAY_LABEL`. (§C.1) |
| T002 (correction #2) | `u32` internally; `overlay_window_id() -> Option<u64>` at the boundary (manager + `LuminosHandle`). | Matches `set_excluded_windows(&[u64])` + the old signature. (§C.2) |
| T006 (correction #3) | Lens/Docked → `Ok(()) + warn!("… deferred to E05")` (was `Err(PropertyFailed)`). **Rewrote 5 tests** that asserted `Err`. | DESIGN-chosen; avoids spurious errors during E04 (FullScreen is the only E04 mode). (§C.3) |
| T001/T002 (correction #4) | `create_overlay` no longer creates a window — it resolves display bounds + the bound XID is preserved. Trait + impl docs updated. | The overlay is opened by `luminos-app` (story 001); FR-2. (§C.4) |
| T002-T006 (correction #5) | **Rewrote** `integration_overlay_mode.rs` + the in-module Xvfb tests for `new(xid, bounds)`: create a throwaway x11rb window, bind, assert via `GetGeometry`/`GetWindowAttributes`/`GetProperty`. (~150 LOC churn, largest task.) | The old tests called `X11WindowManager::new()` (no args) and would not compile. (§C.5) |
| T007 (correction #6) | `raw_window_handle()`/`raw_display_handle()` now ALWAYS `None` (was `Some` after create). Rewrote the `is_some()`-after-create assertions. Trait signature UNCHANGED (FR-6). | Surface sourced by `luminos-app`'s `OverlayGpu` (AD-3). Future trait cleanup flagged for the Phase-0 gate. (§C.6) |
| (correction #7) | `OverlayGpu`/surface coupling matches DESIGN Alternative #3 — no change needed. | Reality already matched. (§C.7) |
| T008 (handle storage) | `LuminosHandle` stores the **concrete** `X11WindowManager` (Linux-gated, `Mutex`), not `Box<dyn WindowManager>`. | `overlay_window_id()` is inherent to the backend, not on the trait (FR-6 keeps the trait unchanged); a `Box<dyn>` couldn't expose it. |
| Cross-crate (gpu tests) | Rewrote `luminos-gpu/tests/{integration_window_gpu.rs, integration.rs}` to source their wgpu surface from a throwaway **winit** window created in-test (not from `X11WindowManager.window()`, which no longer exists), and the binding test to `new(xid, bounds)`. Added `x11rb` to gpu dev-deps. | The E02 GPU integration tests built the surface from the winit overlay window; that path moved to `luminos-app` (story 001). winit in a *test* is fine — FR-1 only forbids a second event loop in the shipping path. The GPU device/surface/texture coverage is preserved. |
