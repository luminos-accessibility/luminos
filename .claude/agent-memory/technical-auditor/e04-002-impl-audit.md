---
name: e04-002-impl-audit
description: E04 Story 002 (x11rb overlay WindowManager + self-capture XID) implementation audit, commit 7a8f984
metadata:
  type: project
---

# E04 Story 002 — Overlay WindowManager (winit→tao) & Self-Capture — AUDIT PASS (2026-06-05, commit 7a8f984)

Worktree branch `worktree-epic+e04-control-panel`. Verdict: **AUDIT PASS**, 0 blocking, 2 non-blocking (both cosmetic/pre-existing).

## Headline verdicts (all CONFIRMED true)
- **FR-1/FR-8 zero-winit/zero-tauri in luminos-platform:** `cargo tree -p luminos-platform` shows NEITHER. winit dep REMOVED from `crates/luminos-platform/Cargo.toml` in the story commit. Grep of platform src finds only doc-comment mentions, no EventLoop/winit/tauri code. The old ephemeral `EventLoop::builder()` in create_overlay (old window.rs:167) is GONE.
- **raw_window_handle()→None reconciliation HONEST:** `X11WindowManager::raw_window_handle()`/`raw_display_handle()` both `return None` (window.rs:287-295), documented AD-3 (surface sourced by luminos-app OverlayGpu). create_overlay creates nothing — resolves bounds + keeps bound XID (window.rs:141-154). Manager holds borrowed `overlay_xid: u32` field, NOT an owned window.
- **capture.rs GENUINELY UNTOUCHED:** `git diff HEAD` AND `git diff 7a8f984~1 7a8f984` both EMPTY for capture.rs. NOT in story commit's changed-files list. Shipped `set_excluded_windows` is at capture.rs:298 exactly; unmap/remap at 162-219, 245-268. Story 002 wrote NO new exclusion code.

## Self-capture probe (claim 3)
`app.rs::probe_self_capture` (app.rs:347, gated by `LUMINOS_SELF_CAPTURE_PROBE=1`) constructs a REAL `XcbCapture::new()`, calls shipped `set_excluded_windows(&[xid])` (capture.rs:298), then `capture_frame`. Handles Ok AND Err without panic (test asserts clean exit 0, not capture success). XID surfaced via `overlay_window_id() -> Option<u64>` on manager + LuminosHandle.
- RISK-002 flicker finding HONEST + non-masked: SUBTASKS.md:184, HLP:296. Live frame-grab failed because xcap 0.9.4 mis-selects Wayland (libwayshot) backend under headless Xvfb — GENUINE env limitation, cross-referenced to pre-existing capture integration tests that fail identically (same env, capture.rs byte-identical to HEAD). Flicker documented as expected cost of unmap/remap under tao/GTK3, optimization deferred post-E04. Not a masked failure.

## DESIGN corrections spot-check (claim 4) — all 3 CONFIRMED
- (a) Lens/Docked now `Ok+warn` (window.rs:188-194); 5 stale is_err tests REWRITTEN to assert Ok + no-resize (integration_overlay_mode.rs:93-127 docked+lens; window.rs:541 lens_docked_deferred). Not deleted-without-replacement.
- (b) integration_overlay_mode.rs compiles against `X11WindowManager::new(xid, bounds)` (line 51).
- (c) after-create `raw_window_handle().is_some()` (old window.rs:464) → now `is_none()` assertion (window.rs:572-584).

## EWMH correctness (claim 5) — VALID
window.rs:210-215: `ClientMessageEvent::new(32, overlay_xid, _NET_WM_STATE, [action, _NET_WM_STATE_ABOVE, 0, 1, 0])` sent to `root` with `SUBSTRUCTURE_NOTIFY|SUBSTRUCTURE_REDIRECT`, propagate=false. Textbook-correct EWMH _NET_WM_STATE: format 32, action add=1/remove=0, source-indication data[3]=1. Plus direct change_property32 fallback so WM-less Xvfb observes state. NOT a malformed no-op.

## Test-count delta (claim 6) — EXACT
- App suite 23→28 (SUBTASKS.md:26): +3 overlay_control subprocess, +2 in-process (overlay_bridge + app_error::bridge). Matches.
- Platform window tests 14/14 = window.rs (9: 1 const test + 8 Xvfb integration) + integration_overlay_mode.rs (5). Matches.
- Old window.rs had 18 tests → new 9. Removed tests (`*_no_window`×3, `*_before_create`×2, `new_default`, `default_display_bounds_none`, `set_overlay_mode_fullscreen_no_bounds`, Lens/Docked `*_rejected`×2) were GENUINELY obsolete — premised on a window-less default/"no window" state that no longer exists (manager binds XID at construction) and on Lens/Docked returning Err (now Ok). Replaced by real-X11-state Xvfb tests. Not silent coverage drop.

## GPU test rewrite (claim 6) — assertions NOT weakened
`integration_window_gpu.rs`: OLD went through (winit-based) X11WindowManager::create_overlay for the surface. Since manager no longer owns a window/surface, NEW creates its OWN throwaway winit window directly (create_test_window, line 35). Same 6 asserts preserved; GPU pipeline asserts (surface/device/configure/get_current_texture/present) unchanged. overlay_window_id self-capture test rewritten to bind via new(xid,bounds) against real x11rb window — assertions STRONGER (echoes exact XID + >0).
- NOTE: this test file uses winit (7 refs) but it's a TEST creating a throwaway window — FR-1 only forbids a second event loop in the SHIPPING path reachable from main. luminos-platform tree is winit-free; that's the FR-1/FR-8 scope.

## Non-blocking findings (cosmetic/pre-existing)
- NF-1 (LOW, pre-existing): `luminos-gpu/src/lib.rs:4` doc-comment still says "winit-based magnification overlay window" — now inaccurate (overlay is tao). Pre-existing, not introduced by 002.
- NF-2 (INFO): luminos-gpu carries winit as a NORMAL [dependencies] (line 20), pre-existing before 002. No winit code in gpu/src (only the doc mention). Does not violate FR-1/FR-8 (those are scoped to luminos-platform).

## Verification method notes
- `cargo check -p luminos-platform --all-features` compiles clean (31s) — confirms rust-analyzer E0308/unlinked/dead_code diagnostics were STALE as stated.
- xcap pinned `=0.9.4`; backend (X11 vs libwayshot) selected internally by xcap, so Wayland mis-detect under Xvfb is plausibly genuine, not a code defect.
