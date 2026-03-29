---
name: E02/002 Implementation Decisions
description: Key decisions from Story E02/002 (X11 Overlay Window & GPU Surface) — luminos-types crate, winit event loop, wgpu v28 API changes
type: project
---

E02/002 (X11 Overlay Window & GPU Surface) completed 2026-03-28 with the following decisions:

- **luminos-types shared crate created (user-directed scope change):** User overrode the DESIGN.md approach (serde derives + re-export from luminos-platform). Instead, a new `luminos-types` crate was created with ZERO workspace deps (only serde). 15 types moved there from luminos-platform and luminos-core. All original locations re-export from luminos-types for backward compatibility. CaptureFrame skips Serialize/Deserialize (runtime GPU type with Arc<[u8]>).

- **winit event loop pattern:** Using deprecated `EventLoop::create_window()` for E02 since X11 windows survive event loop drop (X connection is refcounted). Will migrate to `ActiveEventLoop`-based creation in E05 render loop.

- **wgpu v28 API differences from DESIGN.md:** `request_adapter()` returns `Result<Adapter, RequestAdapterError>` (not `Option`), `request_device()` takes only `&DeviceDescriptor` (no trace_path), `Instance::new()` takes `&InstanceDescriptor`. All adapted in implementation.

- **Alpha mode fallback chain confirmed:** PreMultiplied → PostMultiplied → Opaque with log::warn on fallback. Extracted as testable `select_alpha_mode()` helper.

- **Testable GPU helpers pattern:** `select_alpha_mode()` and `select_texture_format()` extracted as public functions testable in isolation without GPU hardware.

- **Zero-dimension guard added:** `find_display_bounds()` returns `WindowError::CreationFailed` if xcap reports width=0 or height=0, preventing zero-size windows.

**Code review findings addressed:**
- Tautological assertion removed (was `x || !x`)
- Zero-dimension window guard added
- Empty test body removed

**Quality gates:** Code review APPROVED, QA APPROVED (181 tests), Technical audit APPROVED (0 critical, 0 high, 2 medium addressed)

**Why:** These decisions affect future stories: E05 (winit event loop migration), E04 (Tauri integration using luminos-types), Stories 003-005 (GPU pipeline building on device.rs/surface.rs).

**How to apply:** Implementation agents should use `luminos_types::` for shared types. The winit deprecation will need migration in E05. wgpu v28 API notes apply to all GPU stories.
