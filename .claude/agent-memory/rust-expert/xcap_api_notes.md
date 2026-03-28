---
name: xcap 0.9.3 API notes
description: Critical differences between xcap 0.9.3 actual API and what specs assumed
type: project
---

xcap 0.9.3 Monitor API returns `XCapResult<T>` (not bare values) for all property accessors: `id()`, `name()`, `x()`, `y()`, `width()`, `height()`, `scale_factor()`, `is_primary()`.

**Why:** The DESIGN.md for E02/001 assumed bare-value returns (e.g., `monitor.id()` -> `u32`), but the actual API wraps everything in `Result`. This required `monitor_to_display_info()` to return `Result<DisplayInfo, CaptureError>`.

**How to apply:** When writing code that calls xcap Monitor methods, always handle the `Result`. Use a local `map_err` closure to avoid repetition.

xcap has a native `capture_region(x, y, width, height)` method on Monitor, which is more efficient than full capture + image crop. We used this instead of the `image::imageops::crop_imm()` approach from DESIGN.md.

xcap's `wayland_detect()` checks `XDG_SESSION_TYPE` and `WAYLAND_DISPLAY` env vars. On Wayland sessions, xcap uses `libwayshot_xcap` for capture (requires `ZwlrScreencopy` protocol). On XWayland, `GetImage` on the root window fails with XCB Match error. Integration tests for the X11 backend only work on pure X11 (Xvfb in CI) not on XWayland.
