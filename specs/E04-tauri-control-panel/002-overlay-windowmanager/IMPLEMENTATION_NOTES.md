# Story 002 — Implementation Notes (lead briefing, 2026-06-05)

Verified against real source at worktree HEAD `ed5cb17` (story 001 DONE: commits `fed6026`+`5361751`).
SUPERSEDES stale parts of DESIGN.md — log each conflict in `SUBTASKS.md → Deviations from Design`.

## ⚠️ CWD WARNING (do this first, every Bash block)
A bare subagent shell starts in `/home/renatorro/Development/luminos` (branch **main**, where story 001
does NOT exist). ALL work is in the worktree:
`/home/renatorro/Development/luminos/.claude/worktrees/epic+e04-control-panel` (branch `worktree-epic+e04-control-panel`).
Start every Bash with `cd <worktree>` (or use `git -C <worktree>` and absolute paths). Verify:
`git -C <worktree> branch --show-current` must print `worktree-epic+e04-control-panel`.

## A. Trait-reconciliation crux (the core of the story)
Story 001's overlay is a tao/Tauri `WebviewWindow` (label `"overlay"`, built at runtime in
`app.rs::setup_overlay_window`); `OverlayGpu` OWNS a `.clone()` of it for `Surface<'static>`.
Story 002 makes the `WindowManager` trait CONTROL that same window via x11rb, WITHOUT reopening it
and WITHOUT a `tauri` dep in `luminos-platform`:
1. **XID bridge lives in `luminos-app`** (correct dep direction). NEW `luminos-app/src/overlay_bridge.rs`:
   `WebviewWindow` already impls `raw_window_handle::HasWindowHandle`; on X11/tao 0.35 it returns
   `RawWindowHandle::Xlib { window: <XID> }` (or `Xcb`). Extract the XID from that — do NOT use the
   `gtk_window()→gdk→gdk_x11_window_get_xid` path (FR-8's literal prescription; main-thread-only, more
   unsafe FFI). Recommend rwh; note the deviation from FR-8. Extract at **`RunEvent::Ready`** (already
   on main thread, window realized — same spot as `init_overlay_gpu`).
2. **`X11WindowManager` becomes a struct holding `RustConnection` + bound XID (`u32`) + bounds + mode** —
   it NEVER creates a window. `create_overlay(display_id)` is repurposed to RESOLVE display bounds +
   confirm the XID is bound (must NOT create anything — FR-2). Geometry/visibility/stacking = raw X11
   requests against the bound XID.
3. **`raw_window_handle()`/`raw_display_handle()` return `None`** in this backend (AD-3) — surface is
   sourced by `OverlayGpu` in luminos-app, not the trait. This INVERTS the old winit behavior (returned
   `Some` after create). Flag trait-cleanup deviation (T007).
4. **Zero winit EventLoops (FR-1 stays intact).** DELETE the old `create_overlay`'s ephemeral
   `EventLoop::builder()` block (`window.rs:166-179`). x11rb `RustConnection` is a socket, not a loop.
   Verify: `cargo tree -p luminos-platform` shows NEITHER `winit` NOR `tauri` (T001).

## B. Self-capture (RISK-002) — mechanism already shipped
`XcbCapture::set_excluded_windows(&[u64])` (`linux_x11/capture.rs:298`) + per-frame unmap/remap
(`capture.rs:245-268`) is DC-6's shipped mechanism. Story 002 writes NO new exclusion code — it only
SURFACES the overlay XID via `overlay_window_id() -> Option<u64>`. The actual `set_excluded_windows(&[xid])`
call happens in **story 003** against the render-loop capture instance. For 002: exercise the hook
(construct `XcbCapture`, `set_excluded_windows(&[xid])`, capture a frame, no panic) + record any
unmap/remap flicker as a logged RISK-002 finding (NOT a blocker). GOTCHA to flag for 003:
`unmap/remap_excluded_windows` open a FRESH `x11rb::connect(None)` PER FRAME (`capture.rs:171,203`) using
ambient `$DISPLAY` — a latency/correctness smell; leave it for 002, record it.

## C. DESIGN.md staleness corrections (apply + log in Deviations)
1. Overlay **label is `"overlay"`** (`app.rs` `OVERLAY_LABEL`); title is `"Luminos Overlay"`; the test
   harness matches WM_NAME substring `"Overlay"`. Bridge retrieves via `get_webview_window("overlay")`.
2. **`u32` internally, `u64` at the boundary.** DESIGN uses `overlay_xid: u32`; keep that internally but
   `overlay_window_id() -> Option<u64>` to match `set_excluded_windows(&[u64])` + the old signature.
3. **Lens/Docked behavior change.** CURRENT code returns `Err(WindowError::PropertyFailed{… "deferred to
   E05"})` for Lens/Docked, with 5 tests asserting that (`window.rs:388-425`,
   `integration_overlay_mode.rs:61-84`). DESIGN chooses `Ok(()) + warn!`. ADOPT DESIGN (Ok+warn) and
   REWRITE the 5 broken tests — do not leave the contradiction.
4. **`create_overlay` no longer creates** — contract shifts to "resolve bounds + confirm XID". Update its
   doc comment + the `WindowManager` trait doc.
5. **`integration_overlay_mode.rs` won't compile** — it calls `X11WindowManager::new()` (no args); new
   signature is `new(overlay_xid: u32, display_bounds: ScreenRect)`. Rewrite it (+ the in-module Xvfb
   tests `window.rs:442-542`) to create a throwaway x11rb test window, bind its XID, assert via
   `GetGeometry`/`GetWindowAttributes`/`GetProperty`. This is the single biggest mechanical task (~150 LOC churn).
6. **`raw_window_handle().is_some()` after-create assertions break** (`window.rs:464-465`) — now `None`. Rewrite.
7. `OverlayGpu`/surface coupling — DESIGN's Alternative #3 (manager doesn't source the surface) matches
   reality; no change.

## D. Integration seams (real signatures)
- `WindowManager` trait (`luminos-platform/src/traits/window_manager.rs:64-106`, surface UNCHANGED):
  `create_overlay(&mut self, &str)->Result<(),WindowError>`, `set_overlay_bounds(&self, ScreenRect)`,
  `set_overlay_mode(&mut self, OverlayMode)`, `set_always_on_top(&self, bool)`, `set_visible(&self, bool)`,
  `raw_window_handle()->Option<&dyn HasWindowHandle>` (→None), `raw_display_handle()` (→None).
  `WindowError::{CreationFailed,PropertyFailed{property,message},DisplayNotFound,DockFailed,Platform{message}}`.
  Map x11rb errors → `WindowError::Platform{message}` (no unwrap/expect).
- `ScreenRect { x:i32, y:i32, width:u32, height:u32 }`; `OverlayMode::{FullScreen, Lens{..}, Docked{..}}`.
- `ScreenCapture::set_excluded_windows(&mut self, &[u64])`; `XcbCapture` overrides it (`capture.rs:298`).
- x11rb 0.13.2 (features randr,shm,xinput on): `connect(None)->(RustConnection, screen_num)`;
  geometry `configure_window(xid, &ConfigureWindowAux::new().x().y().width().height())`;
  visibility `map_window`/`unmap_window`; always-on-top via `intern_atom`+EWMH `_NET_WM_STATE` ClientMessage
  (`_NET_WM_STATE_ADD=1`/`_REMOVE=0`, atom `_NET_WM_STATE_ABOVE`) `send_event(false, root,
  SUBSTRUCTURE_NOTIFY|SUBSTRUCTURE_REDIRECT, msg)`; always `conn.flush()` after mutations.
  Test assertions: `get_geometry(xid).reply()`, `get_window_attributes(xid).reply().map_state`,
  `get_property(... _NET_WM_STATE ...)` membership.
- `LuminosHandle` (`luminos-app/src/handle.rs:21`): add a field to STORE the constructed `WindowManager`
  (`Mutex<Option<Box<dyn WindowManager>>>` or concrete `X11WindowManager`) so story 003 can reach it;
  construct it at `Ready` after XID extraction.
- `AppError` (`luminos-app/src/app_error.rs`): add `Bridge(String)` for rwh/gdk failures.

## E. Subtask sequencing (11 tasks)
T001 strip winit + x11rb skeleton + drop winit from platform Cargo.toml + trait doc (build; `cargo tree`
shows no winit/tauri). T002 RustConnection + bind XID + `new(xid,bounds)`/`create_overlay`/`overlay_window_id`
(Xvfb unit: create test window, GetGeometry). T003 `set_overlay_bounds` (ConfigureWindow). T004
`set_always_on_top` (_NET_WM_STATE_ABOVE — assert PROPERTY set, since WM-less Xvfb won't enforce stacking).
T005 `set_visible` (Map/Unmap). T006 `set_overlay_mode(FullScreen)` + Lens/Docked **Ok+warn** (rewrite 5
tests). T007 handles→None + flag deviation. **Checkpoint.** T008 `overlay_bridge.rs` + wire at `Ready`
(subprocess: assert `overlay_xid=…` non-zero). T009 expose XID + exercise `set_excluded_windows` hook +
record flicker. T010 geometry/visibility/stacking subprocess verification via **x11rb query harness**
(`tests/common/mod.rs` `find_windows`/`query_tree` — NOT xwininfo/xprop, both absent; extend `XWindow`
with `_NET_WM_STATE` if needed). T011 AC matrix, fmt/clippy, Shared Context + Deviations.

## F. Risks/gotchas
- tao GTK3 overlay is a GTK `ApplicationWindow`, NOT override-redirect (old winit used
  `with_override_redirect(true)` — gone). Raw `ConfigureWindow`/`_NET_WM_STATE` on a GTK window may race
  with GDK and, under a real WM, may need `_NET_MOVERESIZE_WINDOW`. Fine under WM-less Xvfb for E04; record.
- No `tauri` dep in `luminos-platform` (hard). XID extraction stays entirely in `luminos-app/overlay_bridge.rs`
  (luminos-app already has x11rb behind the `tauri` feature). Verify `cargo tree -p luminos-platform`.
- Keep FR-1: nothing reachable from `main` calls the OLD creating `create_overlay`.
- Test breakage (~150 LOC) is the hidden cost: rewrite `integration_overlay_mode.rs`, the in-module Xvfb
  tests, and the Lens/Docked `is_err` tests. Budget for it.
