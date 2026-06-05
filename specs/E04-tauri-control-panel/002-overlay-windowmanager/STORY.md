# Story E04/002: Overlay WindowManager (winit→tao) & Self-Capture

**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** 001 (the running app + the Tauri overlay window it opens)

---

## Problem Statement

Story 001 opens the overlay as a Tauri/tao window and renders into it directly from `luminos-app`. But the platform abstraction's `WindowManager` trait — the contract the rest of the engine uses to position, size, show/hide, and stack the overlay — is still implemented (in `luminos-platform::linux_x11::window::X11WindowManager`) on top of **winit**, which is incompatible with the single-tao-loop architecture (RISK-001/AD-1) and is currently dead code (`#[allow(dead_code)]`, awaiting integration).

This story reimplements the X11 overlay backend so it controls the **already-created tao/Tauri overlay window** via **x11rb** (the X11 protocol crate already used for capture/input), with **no winit and no `tauri` dependency in `luminos-platform`**. `luminos-app` bridges the two layers: it extracts the overlay's X11 window id from the Tauri/GTK window and hands that id to the platform `X11WindowManager`, which then drives geometry/visibility/stacking through raw X11 messages.

It also retires the second half of RISK-002 under the new backend: the magnification overlay covers the screen and shows magnified content, so a naive screen capture would re-capture the overlay's own output (a "hall of mirrors" feedback loop). This story implements and validates a **self-capture exclusion** strategy under tao's GTK3 window, with a documented fallback.

## User Scenarios

> **AC count = 5.**

### US-1: The overlay is controllable through the platform trait
As an engine developer, I want to drive the overlay's position, size, visibility, and stacking through the existing `WindowManager` trait, so that later stories (live magnification, lens/docked modes) use one stable platform contract regardless of the underlying windowing toolkit.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1 (geometry/visibility/stacking):** Given the tao-backed `X11WindowManager` bound to the overlay's X11 window id, when `set_overlay_bounds(rect)`, `set_visible(bool)`, and `set_always_on_top(bool)` are called, then the overlay's geometry, mapped-state, and `_NET_WM_STATE_ABOVE` change accordingly, verified via `xprop`/`xwininfo`. *(FR-1, FR-2, FR-4)*
- **AC-1.2 (full-screen mode + trait surface unchanged):** Given the manager, when `create_overlay(display_id)` then `set_overlay_mode(OverlayMode::FullScreen)` are called for a known display, then the overlay is sized to that display's bounds; and the public `WindowManager` trait signature is unchanged so all existing trait unit tests still pass (Lens/Docked modes return a documented "not-yet-implemented in E04" path, deferred to E5). *(FR-2, FR-3, FR-6)*

### US-2: No self-capture feedback
As a user, I want the magnified overlay to show my actual desktop, not a recursive image of the magnifier itself, so that magnification is usable.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1 (exclusion works):** Given full-screen magnification with the overlay visible and rendering, when the capture path acquires the next frame, then the overlay's own output is excluded from the capture (no recursive feedback), verified by the chosen mechanism (overlay X11 window id known to the capture/exclusion logic; or unmap/remap around capture). *(FR-5)*
- **AC-2.2 (documented fallback):** Given the primary self-capture mitigation is unavailable or unstable under tao/GTK3, when the manager initializes, then it selects and logs a documented fallback (e.g. unmap/remap vs. window-id exclusion vs. capturing a non-overlay region) without panicking. *(FR-5, FR-7)*

### US-3: winit removed from the overlay path
As a maintainer, I want winit gone from the shipping overlay path, so that the single-tao-loop architecture holds and there is no second windowing stack.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1 (winit-free):** Given the platform layer builds, when `X11WindowManager` is compiled, then it instantiates no `winit::EventLoop` and creates no winit window (overlay control is x11rb-only over the bound window id), and `luminos-platform` does not depend on `tauri`. *(FR-1, FR-8)*

## Functional Requirements

- **FR-1:** `X11WindowManager` MUST control the overlay via **x11rb** over a bound X11 window id; it MUST NOT use winit and MUST NOT depend on `tauri`. *(Traced by AC-1.1, AC-3.1)*
- **FR-2:** `X11WindowManager` MUST accept the overlay's X11 window id at construction/`create_overlay` and apply geometry via `ConfigureWindow`. *(Traced by AC-1.1, AC-1.2)*
- **FR-3:** `set_overlay_mode(FullScreen)` MUST size the overlay to the target display's bounds (from `DisplayInfo`). Lens/Docked MUST return a documented deferral (E5). *(Traced by AC-1.2)*
- **FR-4:** `set_always_on_top` MUST toggle `_NET_WM_STATE_ABOVE`; `set_visible` MUST map/unmap the window. *(Traced by AC-1.1)*
- **FR-5:** The overlay MUST be excluded from screen capture so no self-capture feedback occurs, using the **shipped** `ScreenCapture::set_excluded_windows(&[overlay_xid])` mechanism (`XcbCapture` already implements unmap/remap exclusion); the manager MUST expose the overlay X11 window id (`overlay_window_id()`) so story 003 wires it into the capture instance. *(Traced by AC-2.1)*
- **FR-6:** The `WindowManager` trait public signature MUST remain unchanged (existing trait tests pass). *(Traced by AC-1.2)*
- **FR-7:** The self-capture mitigation MUST degrade gracefully with a logged fallback. *(Traced by AC-2.2)*
- **FR-8:** `luminos-app` MUST extract the overlay's X11 window id from the Tauri overlay window (`WebviewWindow::gtk_window()` → gdk → XID) and pass it to `X11WindowManager`. *(Traced by AC-3.1)*

## Non-Functional Requirements

- **NFR-1:** Geometry/visibility/stacking changes MUST apply within one frame (no visible lag) and MUST NOT block the render path (x11rb calls happen off the render hot loop or are sub-millisecond).
- **NFR-2:** The self-capture mitigation MUST NOT introduce visible flicker at the 60fps target; if unmap/remap is the only viable approach and flickers, that is a logged finding feeding RISK-002 (not a silent regression).
- **NFR-3:** No `unwrap()`/`expect()` in production paths; X11 errors map to `WindowError` variants.

## Out of Scope

- Lens and Docked mode rendering/geometry and EWMH struts → **Epic 5** (this story returns a documented deferral for those modes).
- The actual capture→render→present loop and `Renderer` wiring → **story 003** (this story only ensures the overlay is excludable and controllable).
- Wayland/macOS/Windows window managers → later epics (E8/E12/E17).
- Multi-monitor overlay management → Epic 16.

## Open Questions

- [x] Does `luminos-platform` take a `tauri` dependency? — **Resolved: No.** It controls the overlay by X11 window id via x11rb; `luminos-app` does the Tauri→XID bridge (FR-8). Keeps the dependency direction correct.
- [x] What replaces the winit-based `X11WindowManager`? — **Resolved:** an x11rb reimplementation that binds to an existing window id; the old winit creation code is removed.
- [x] Which self-capture mechanism wins under tao/GTK3 (window-id exclusion vs. unmap/remap vs. region capture)? — **Resolved as a decision procedure** (RISK-002, partly empirical): the DESIGN enumerates the candidates and the selection criteria; implementation picks the first that works under tao/GTK3 without flicker and records it (+ fallback) in completion notes and epic Shared Context. This mirrors story 001's spike resolution — a documented procedure, not an open blocker.
