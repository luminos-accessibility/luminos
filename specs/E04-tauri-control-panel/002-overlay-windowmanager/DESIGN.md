# Design: Story E04/002 -- Overlay WindowManager (winit→tao) & Self-Capture

**Story:** [STORY.md](./STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** principal-architect
**Risk Refs:** RISK-002 (self-capture — mitigated here), RISK-001 (winit removal completes the AD-1 migration)

---

## Overview

Reimplement `luminos-platform::linux_x11::window::X11WindowManager` over **x11rb** so it controls an **existing** overlay window by its **X11 window id**, removing all winit usage. `luminos-platform` stays free of any `tauri` dependency (correct dependency direction). The bridge lives in `luminos-app`: it extracts the overlay's XID from the Tauri/GTK window and constructs the manager. The wgpu surface is **not** sourced from this manager (story 001's `OverlayGpu` owns the window clone for that); the trait's `raw_window_handle`/`raw_display_handle` therefore return `None` in this backend, with a doc note and a flagged trait-cleanup deviation.

Self-capture (RISK-002): because the full-screen overlay shows magnified desktop content, capturing the screen would recapture the overlay → feedback. The manager exposes the overlay XID and implements an exclusion strategy validated under tao/GTK3, with a documented fallback.

## Architecture

### Component Diagram

```
  luminos-app (Linux bridge)                         luminos-platform (tauri-free)
  ┌───────────────────────────────┐                 ┌───────────────────────────────────────┐
  │ overlay: tauri::WebviewWindow  │                 │ X11WindowManager (x11rb)              │
  │  .gtk_window()? → gdk window   │   xid: u32      │  conn: RustConnection                 │
  │  → gdk_x11 XID  ───────────────┼────────────────▶│  overlay_xid: Option<u32>             │
  │ OverlayGpu owns window clone   │  create_overlay │  display_bounds, current_mode         │
  │  (surface; story 001)          │                 │  set_overlay_bounds/mode/visible/...   │
  └───────────────────────────────┘                 │  overlay_window_id() → Option<u32>     │
        capture path (story 003) ◀─── exclude xid ───┤  self-capture exclusion / unmap-remap │
                                                      └───────────────────────────────────────┘
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `luminos-platform/src/linux_x11/window.rs` | Rewrite | winit `EventLoop`/`with_override_redirect` removed; x11rb control over a bound XID. |
| `luminos-platform` `WindowManager` trait | Unchanged signature | Doc comment updated (remove "winit-based"); `raw_window_handle`/`raw_display_handle` return `None` in this backend (documented). |
| `luminos-platform/src/linux_x11/mod.rs` | Modified | Export the new manager API (`new`, `overlay_window_id`). |
| `luminos-platform/Cargo.toml` | Modified | Drop winit (if only used here); keep x11rb (`randr`,`shm`,`xinput` already). |
| `luminos-app/src/overlay_bridge.rs` | New | `extract_overlay_xid(window: &tauri::WebviewWindow) -> Result<u32, AppError>` via `gtk_window()` → gdk → XID; constructs `X11WindowManager`. |
| `luminos-core::pipeline`/`event` doc comments | Deferred (story 003) | "winit event loop" comment cleanup tracked, not done here. |

### Data Flow

1. `luminos-app` (after story 001 opened the overlay) calls `overlay.gtk_window()` (Tauri 2.x, Linux) → `gtk::ApplicationWindow` → `.window()` (gdk) → `gdk_x11::X11Window` XID (`u32`).
2. Constructs `X11WindowManager::new(conn, overlay_xid, display_bounds)` (or `create_overlay(display_id)` binds the XID + resolves bounds from `DisplayInfo`).
3. Engine code calls trait methods → manager issues x11rb requests: `ConfigureWindow` (bounds), `MapWindow`/`UnmapWindow` (visible), `_NET_WM_STATE_ABOVE` via `ChangeProperty`/client message (always-on-top).
4. Capture path (story 003) queries `overlay_window_id()` and applies the chosen self-capture exclusion before/around `capture_frame`.

### Self-capture (RISK-002) — use the SHIPPED mechanism first

**Important correction:** the X11 capture backend (`luminos-platform::linux_x11::capture::XcbCapture`) **already implements self-capture exclusion** via the existing trait method `ScreenCapture::set_excluded_windows(&mut self, ids: &[u64])` (it unmaps/remaps the excluded windows around each x11rb capture — it is NOT a raw per-window `xcap` grab). So the primary path is **not new**: pass the overlay XID to `set_excluded_windows(&[overlay_xid])`, and the existing backend handles it. Story 002's job is to surface the overlay XID (`overlay_window_id()`) and ensure story 003 wires it into the capture instance the render loop uses.

| Path | Mechanism | Status |
|------|-----------|--------|
| **Primary** | `ScreenCapture::set_excluded_windows(&[overlay_xid])` — shipped `XcbCapture` unmap/remap | **Already implemented; just wire the XID** |
| Future opt | Per-window/region exclusion without unmap (avoid any flicker) | Optimization, post-E04 |
| Future opt | XComposite capture excluding overlay | Optimization, compositor-dependent |
| D (lens/docked) | Overlay doesn't cover captured region | E5 (non-fullscreen) |

If the shipped unmap/remap introduces visible flicker at 60fps (NFR-2), that is a **logged RISK-002 finding** feeding the future optimizations — not a blocker for E04. Record the observed behavior in completion notes + epic Shared Context.

## API Design

```rust
// luminos-platform/src/linux_x11/window.rs (reimplemented; trait surface unchanged)
pub struct X11WindowManager {
    conn: x11rb::rust_connection::RustConnection,
    screen_num: usize,
    overlay_xid: Option<u32>,
    display_bounds: ScreenRect,
    current_mode: OverlayMode,
}

impl X11WindowManager {
    /// Bind to an externally-created overlay window (its X11 id) on `screen`.
    pub fn new(overlay_xid: u32, display_bounds: ScreenRect) -> Result<Self, WindowError>;
    /// Overlay X11 window id for the capture path's self-capture exclusion.
    pub fn overlay_window_id(&self) -> Option<u32>;
}

impl WindowManager for X11WindowManager {
    fn create_overlay(&mut self, display_id: &str) -> Result<(), WindowError>; // resolve bounds, confirm XID bound
    fn set_overlay_bounds(&self, bounds: ScreenRect) -> Result<(), WindowError>; // ConfigureWindow
    fn set_overlay_mode(&mut self, mode: OverlayMode) -> Result<(), WindowError>; // FullScreen now; Lens/Docked => WindowError? No — Ok + log deferral
    fn set_always_on_top(&self, on: bool) -> Result<(), WindowError>;            // _NET_WM_STATE_ABOVE
    fn set_visible(&self, visible: bool) -> Result<(), WindowError>;             // Map/Unmap
    fn raw_window_handle(&self) -> Option<&dyn raw_window_handle::HasWindowHandle> { None }   // surface sourced in luminos-app
    fn raw_display_handle(&self) -> Option<&dyn raw_window_handle::HasDisplayHandle> { None }
}

// luminos-app/src/overlay_bridge.rs
pub(crate) fn extract_overlay_xid(window: &tauri::WebviewWindow) -> Result<u32, AppError>;
```

> **Lens/Docked handling:** `set_overlay_mode` for `Lens`/`Docked` returns `Ok(())` after a `warn!` that the mode is deferred to E5 (so callers don't break), OR a dedicated `WindowError::Platform { message: "lens/docked deferred to E5" }` — **chosen: `Ok(()) + warn!`** to avoid spurious errors during E04 (full-screen is the only E04 mode). Documented.

## Error Handling

x11rb `ConnectionError`/`ReplyError`/`ReplyOrIdError` map into `WindowError::Platform { message }` (or `PropertyFailed`/`DockFailed` where specific) via `From`/`.map_err`. `?` propagation throughout; no `unwrap`/`expect`. The `extract_overlay_xid` bridge maps GTK/gdk failures to `AppError::OverlayMissing`/a new `AppError::Bridge(String)`.

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | x11rb control by XID; `_NET_WM_STATE_ABOVE`; Map/Unmap; self-capture via the shipped `set_excluded_windows` (unmap/remap). | Only platform this story. |
| Linux Wayland | Deferred E8. No XID; uses layer-shell + a different self-capture story. | — |
| macOS | Deferred E12. `CocoaWindowManager`/NSPanel; self-capture via `CGWindowListCreateImage` exclusion. | — |
| OpenBSD | Deferred E15. Shares X11 path. | — |
| Windows | Deferred E17/E18. `Win32WindowManager`; exclude via `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`. | — |

## Testing Strategy

### Unit tests (x11rb against Xvfb display, `ci_platform_tests`)
- `x11_window_manager_set_bounds_applies` — create a test window, bind its XID, `set_overlay_bounds`, assert geometry via x11rb `GetGeometry` (AC-1.1).
- `x11_window_manager_always_on_top_sets_state` — assert `_NET_WM_STATE_ABOVE` present after `set_always_on_top(true)` (AC-1.1).
- `x11_window_manager_visible_maps_unmaps` — assert map state toggles (AC-1.1).
- `x11_window_manager_fullscreen_sizes_to_display` — `set_overlay_mode(FullScreen)` → geometry == display bounds (AC-1.2).
- `x11_window_manager_lens_docked_deferred` — returns `Ok` + logs (AC-1.2).
- `x11_window_manager_handles_return_none` — `raw_window_handle()`/`raw_display_handle()` == `None` (documented behavior).
- Existing `window_manager.rs` trait tests still compile/pass (AC-1.2, trait unchanged).

### Integration tests
- Self-capture (AC-2.1/2.2): with the real app overlay (subprocess, story-001 harness) showing a known pattern, capture a frame via the story-003-adjacent capture path and assert the overlay's pattern is absent (or that the chosen exclusion is active + logged). Because full capture+render is story 003, this story's integration test may stub the capture call and assert the exclusion hook (`overlay_window_id()` consulted / unmap-remap invoked) is exercised.
- `extract_overlay_xid` (AC-3.1, FR-8): subprocess test asserts the app logs a non-zero overlay XID at startup on X11.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|---------------------|
| AC-1.1 | Unit (x11rb/Xvfb) | GetGeometry + `_NET_WM_STATE_ABOVE` + map-state assertions. |
| AC-1.2 | Unit | FullScreen sizing == display bounds; existing trait tests pass; Lens/Docked deferral. |
| AC-2.1 | Integration | Overlay pattern absent from capture (or exclusion hook exercised + logged). |
| AC-2.2 | Unit/Integration | Forced-fallback path logs the chosen fallback, no panic. |
| AC-3.1 | Build + Subprocess | No winit in `X11WindowManager`; `luminos-platform` has no `tauri` dep (grep/`cargo tree`); app logs non-zero XID. |

## Performance Targets
- Geometry/visibility changes < 1 frame; x11rb calls off the render hot loop (NFR-1).
- Self-capture mitigation introduces no 60fps-visible flicker, or the flicker is a logged RISK-002 finding (NFR-2).

## Security Considerations
- No new capabilities; X11 control is local. Self-capture exclusion prevents leaking the overlay's content into captures it shouldn't be in (minor).

## Alternatives Considered
1. **Keep `X11WindowManager` on winit, drive Tauri separately.** Rejected — violates AD-1 (two windowing stacks / two loops).
2. **Add a `tauri` dependency to `luminos-platform` and pass the `WebviewWindow`.** Rejected — wrong dependency direction; couples the platform abstraction to the app shell. The XID bridge keeps layers clean.
3. **Source the wgpu surface from the `WindowManager` (return owned handle).** Rejected — story 001's `OverlayGpu` already owns the window clone for the surface; duplicating handle ownership in the platform layer adds no value and would force a `tauri` dep. Hence `raw_window_handle` returns `None` here (flag a future trait cleanup at the Phase-0 gate).
4. **Reinventing self-capture exclusion in this story.** Rejected — the shipped `XcbCapture` already implements `ScreenCapture::set_excluded_windows(&[u64])` via unmap/remap (verified in `linux_x11/capture.rs`). This story's only job is to surface the overlay XID (`overlay_window_id()`) for story 003 to pass in; no new exclusion mechanism is written. If the shipped unmap/remap shows 60fps flicker, that is a logged RISK-002 finding feeding the post-E04 flicker-free optimizations (per the table above), not a story-002 deliverable.
