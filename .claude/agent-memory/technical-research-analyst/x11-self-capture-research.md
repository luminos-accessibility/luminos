---
name: X11 Self-Capture Prevention Research
description: Findings on RISK-002 self-capture prevention for the Luminos magnification overlay on X11
type: project
---

## X11 Self-Capture Prevention (RISK-002)

xcap 0.9.3 captures from root window via `xcb_get_image`. On composited desktops, ALL visible windows (including override-redirect) appear in captures.

**Why:** The risk register's mitigation #2 (xcb_composite_redirect_window excludes override-redirect) is INCORRECT. Compositors re-composite override-redirect windows onto the root.

**How to apply:**
- Primary solution: Unmap/remap cycle around each capture call using x11rb
- Fallback: Watermark detection (Strategy C from risk register contingency)
- Rejected: Custom compositor approach (Strategy B) -- too complex, conflicts with running compositor
- New trait method needed: `set_excluded_windows(&mut self, window_ids: &[u64])`

## xcap 0.9.3 Key Details
- Returns `image::RgbaImage` (RGBA format, not BGRA)
- X11 path: `xorg_capture(screen_buf.root(), ...)` -- always root window
- Uses `xcb` crate internally (not x11rb)
- No XShm support (deferred to Phase 1)
- No window exclusion API
- Creates new XCB connection per call

## winit 0.30.13 Overlay Configuration
- `with_transparent(true)` + `with_override_redirect(true)` + `with_decorations(false)` + `with_window_level(AlwaysOnTop)`
- X11 window ID available via `raw_window_handle()` -> `XlibWindowHandle::window`
- `WindowAttributesExtX11` trait provides X11-specific methods

## wgpu Transparency
- `CompositeAlphaMode::PreMultiplied` preferred; support varies by driver
- Query `surface.get_capabilities()` at runtime
- Shaders must output premultiplied alpha

## CI Testing
- Xvfb + lavapipe (mesa-vulkan-drivers) for headless Vulkan
- Xvfb has no compositor by default; unmap/remap works identically
- Consider picom in CI for compositor-realistic testing
