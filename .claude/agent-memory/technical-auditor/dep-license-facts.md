# Dependency / License Facts (Luminos workspace)

Source of truth: `cargo metadata --format-version 1 --manifest-path crates/luminos-app/Cargo.toml`,
field `packages[].license` (SPDX). Verified 2026-06-06.

## SPDX licenses (resolved graph)
- tao 0.35.3 — `Apache-2.0` (NOT dual-licensed; Apache only)
- x11rb 0.13.2 — `MIT OR Apache-2.0`
- winit 0.30.13 — `Apache-2.0` (Apache only)
- tauri 2.11.2 — `Apache-2.0 OR MIT`
- wgpu 29.0.3 — `MIT OR Apache-2.0`
- xcap 0.9.4 — `Apache-2.0`
- cpal 0.17.3 — `Apache-2.0`

NOTE: tao and winit are Apache-ONLY (single license). README "Window management"
row "Apache 2.0 / MIT" is a fair COMBINED representation of tao(Apache-2.0)+x11rb(MIT OR Apache-2.0):
the slash means "the combo spans Apache-2.0 and MIT", not "each is dual". Acceptable but mildly loose
(tao alone is not MIT-available). LOW at most.

## tao is TRANSITIVE only (single version 0.35.3)
Direct dependents of tao in graph: muda, tauri-runtime-wry, tray-icon, window-vibrancy, wry.
NO luminos crate depends on tao directly. "tao (via Tauri)" is accurate.

## winit dependency vs USAGE nuance (re-confirmed 2026-06-06)
- workspace dep `winit = "=0.30.13"`, declared by luminos-gpu AND luminos-core manifests.
- luminos-core/src USES winit: `EventNotifier` impl on `winit::event_loop::EventLoopProxy<LuminosEvent>`
  (pipeline.rs) + event.rs doc/type references.
- luminos-gpu/src has ZERO winit usage (grep empty) — dep is declared but unused in src (pre-existing).
- luminos-platform: zero winit/tauri in manifest AND src (only doc comments referencing the RETIRED winit backend).
  => Overlay re-description edits (winit→tao/x11rb) must NOT say "winit removed entirely"; it remains a
  core/gpu dep. The E04 edits correctly scope the change to the OVERLAY mechanism only.

## Overlay mechanism ground truth (E04)
- Overlay = a SECOND tao/Tauri `WebviewWindow` labeled "overlay" (app.rs OVERLAY_LABEL="overlay",
  setup_overlay_window: transparent(true), decorations(false), always_on_top(true), set_ignore_cursor_events(true)).
- wgpu `Surface<'static>` built from the owned overlay WebviewWindow handle (overlay_gpu.rs OverlayGpu::new).
- Geometry/visibility/stacking via x11rb: overlay_bridge.rs extracts XID via raw_window_handle →
  X11WindowManager::new (luminos-platform/linux_x11/window.rs, RustConnection, no winit, no window creation).
- run_event_loop has explicit FR-1 INVARIANT comment: ONE tauri::App::run loop, never a winit::EventLoop.
  X11WindowManager::create_overlay is NOT called from the app path (that's the retired winit overlay-creation seam).
