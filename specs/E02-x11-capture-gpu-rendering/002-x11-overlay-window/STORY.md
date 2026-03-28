# Story E02/002: X11 Overlay Window & GPU Surface

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** None (parallel with 001)

---

## Problem Statement

The Luminos magnification overlay requires a native window on Linux X11 that is transparent, borderless, always-on-top, and capable of receiving GPU-rendered content via wgpu. Without this window, captured screen content has nowhere to be displayed -- it is the user-visible surface of the magnification pipeline. This story implements the `WindowManager` trait for X11 (FullScreen mode only -- docked and lens modes are Epic 5), initializes the wgpu device and surface on a Vulkan backend, and resolves the `DockEdge`/`LensShape` type duplication discovered in E01 so that settings flow cleanly from the control panel to overlay mode changes.

The overlay window is the architectural bridge between screen capture (Story 001) and GPU rendering (Stories 003-005). It must exist and be correctly configured before any magnified pixels can appear on screen.

## User Scenarios

### US-1: Overlay Window Creation and Visibility

As a low-vision user launching Luminos on X11, I want a transparent overlay window to appear on my primary display so that magnified content can be rendered on top of my desktop without obscuring unmagnified areas outside the viewport.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given a running X11 display server, when `X11WindowManager::create_overlay(display_id)` is called with a valid display ID, then an overlay window is created that is transparent, borderless, always-on-top, and uses override-redirect to bypass the window manager.
- **AC-1.2:** Given an overlay window created on X11, when `set_visible(true)` is called, then the window becomes visible on the target display; when `set_visible(false)` is called, then the window is hidden.
- **AC-1.3:** Given an overlay window on X11, when `set_always_on_top(true)` is called, then the window remains above all other non-override-redirect windows on the display.
- **AC-1.4:** Given an overlay window on X11, when `set_overlay_bounds(bounds)` is called with a valid `ScreenRect`, then the window is repositioned and resized to match the specified bounds.

### US-2: wgpu Device and Surface Initialization

As a rendering pipeline component, I want a wgpu device and surface bound to the overlay window so that GPU shaders can render magnified content directly to the window's swap chain.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given an overlay window with valid `HasWindowHandle` and `HasDisplayHandle`, when `create_gpu_device()` is called, then a wgpu `Device` and `Queue` are returned using the `LowPower` adapter preference.
- **AC-2.2:** Given a wgpu device and the overlay window surface, when `configure_surface()` is called, then the surface is configured with an sRGB-compatible format, `PreMultiplied` alpha mode (with fallback), and `PresentMode::Fifo`.
- **AC-2.3:** Given a configured wgpu surface, when `surface.get_current_texture()` is called, then a valid `SurfaceTexture` is returned for rendering.
- **AC-2.4:** Given no compatible GPU adapter available, when `create_gpu_device()` is called, then a `RenderError::NoAdapter` error is returned.

### US-3: Overlay Mode Configuration (FullScreen Only)

As a user launching Luminos in full-screen magnification mode, I want the overlay window to cover the entire display so that all screen content is magnified.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given an overlay window on X11, when `set_overlay_mode(OverlayMode::FullScreen)` is called, then the overlay covers the entire display.

### US-4: DockEdge/LensShape Type Unification

As a developer bridging control panel settings to overlay mode changes, I want a single canonical definition of `DockEdge` and `LensShape` with serde support so that settings deserialized from config files can be used directly with the `WindowManager` trait without manual conversion.

**Priority:** P1
**Acceptance Criteria:**

- **AC-4.1:** Given the `DockEdge` enum, when inspected in both `luminos-platform` and `luminos-core`, then a single definition exists (in `luminos-platform`) with `Serialize`/`Deserialize` derives, and `luminos-core` re-exports it.
- **AC-4.2:** Given the `LensShape` enum, when inspected in both `luminos-platform` and `luminos-core`, then a single definition exists (in `luminos-platform`) with `Serialize`/`Deserialize` derives, and `luminos-core` re-exports it.
- **AC-4.3:** Given the existing unit tests in both crates that reference `DockEdge` and `LensShape`, when the test suite runs after unification, then all tests pass without modification (beyond import path changes).
- **AC-4.4:** Given the `OverlayMode` enum in `luminos-platform`, when inspected, then it also derives `Serialize` and `Deserialize` to support IPC serialization.

### US-5: Raw Window Handle Integration

As a rendering pipeline initializer, I want the `X11WindowManager` to provide `HasWindowHandle` and `HasDisplayHandle` trait objects so that wgpu can create a surface bound to the overlay window.

**Priority:** P0
**Acceptance Criteria:**

- **AC-5.1:** Given an `X11WindowManager` with an active overlay, when `raw_window_handle()` is called, then it returns `Some(&dyn HasWindowHandle)` containing the X11 window ID.
- **AC-5.2:** Given an `X11WindowManager` with an active overlay, when `raw_display_handle()` is called, then it returns `Some(&dyn HasDisplayHandle)` containing the X11 display connection.
- **AC-5.3:** Given an `X11WindowManager` without an active overlay (before `create_overlay`), when `raw_window_handle()` or `raw_display_handle()` is called, then `None` is returned.

### US-6: Overlay Window ID for Self-Capture Exclusion

As the screen capture backend (Story 001), I need the X11 window ID of the overlay window so that I can exclude it from captured frames, preventing infinite self-capture feedback (RISK-002).

**Priority:** P0
**Acceptance Criteria:**

- **AC-6.1:** Given an `X11WindowManager` with an active overlay, when `overlay_window_id()` is called, then it returns `Some(u64)` containing the X11 window ID extracted from the raw window handle (`RawWindowHandle::Xlib(handle) -> handle.window as u64`).
- **AC-6.2:** Given an `X11WindowManager` without an active overlay, when `overlay_window_id()` is called, then it returns `None`.

## Functional Requirements

- **FR-1:** Implement `X11WindowManager` struct in `crates/luminos-platform/src/linux_x11/window.rs` that implements the `WindowManager` trait using `winit`. *(Traced by AC-1.1 through AC-1.4)*
- **FR-2:** Create wgpu device initialization in `crates/luminos-gpu/src/device.rs` with `LowPower` adapter preference and `downlevel_webgl2_defaults` limits. *(Traced by AC-2.1, AC-2.4)*
- **FR-3:** Create wgpu surface configuration in `crates/luminos-gpu/src/surface.rs` with sRGB format preference, `PreMultiplied` alpha mode (with fallback), and configurable `PresentMode`. *(Traced by AC-2.2, AC-2.3)*
- **FR-4:** Set X11 window properties via winit: transparent, borderless (no decorations), always-on-top, and override-redirect (`with_override_redirect(true)`) to bypass the window manager. *(Traced by AC-1.1, AC-1.3)*
- **FR-5:** Implement `set_overlay_mode(OverlayMode::FullScreen)` to resize the overlay to cover the entire target display. *(Traced by AC-3.1)*
- **FR-6:** Unify `DockEdge` and `LensShape` definitions -- add `Serialize`/`Deserialize` to `luminos-platform` definitions, re-export from `luminos-core`. *(Traced by AC-4.1, AC-4.2, AC-4.3, AC-4.4)*
- **FR-7:** Implement `raw_window_handle()` and `raw_display_handle()` on `X11WindowManager` returning winit's raw handles. *(Traced by AC-5.1, AC-5.2, AC-5.3)*
- **FR-8:** Define `RenderError` enum in `crates/luminos-gpu/src/error.rs` with variants for GPU initialization failures. *(Traced by AC-2.4)*
- **FR-9:** Implement `overlay_window_id()` method on `X11WindowManager` that extracts the X11 window ID (`u64`) from the raw window handle for self-capture exclusion (RISK-002). *(Traced by AC-6.1, AC-6.2)*

## Non-Functional Requirements

- **NFR-1:** wgpu device creation must complete within 200ms on integrated GPUs (within the ~400ms startup-to-first-frame budget from doc-03 Section 1.3).
- **NFR-2:** Overlay window transparency must work on compositing X11 window managers (Mutter, KWin, Picom). Non-compositing WMs may fall back to opaque.
- **NFR-3:** No `unwrap()` or `expect()` in production code paths. `unwrap()` is acceptable in `#[cfg(test)]` blocks.
- **NFR-4:** All public APIs must have `///` doc-comments.
- **NFR-5:** `cargo clippy -p luminos-platform -p luminos-gpu -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions` must pass.

## Out of Scope

- Screen capture implementation (Story 001).
- GPU texture upload and double buffering (Story 003).
- Magnification shaders and viewport calculation (Story 004).
- Render loop assembly and frame pacing (Story 005).
- Docked and Lens overlay modes (Epic 5) -- E02 is scoped to FullScreen mode only. Docked mode requires EWMH `_NET_WM_STRUT_PARTIAL` which is incompatible with override-redirect windows; resolving that tension is E05 work.
- Lens and docked mode rendering logic (Epic 5) -- shader-level lens/docked rendering is later.
- Wayland, macOS, OpenBSD, and Windows `WindowManager` implementations.
- Color filter shaders and cursor overlay (Epic 6).
- Click-through behavior for lens mode (Epic 5).

## Open Questions

*None -- all questions resolved by tech strategy documents and E01 Shared Context.*
