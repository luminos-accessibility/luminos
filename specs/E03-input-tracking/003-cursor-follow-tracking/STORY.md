# Story E03/003: Cursor-Follow Viewport Tracking

**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** 001 (X11 Global Input Monitoring), 002 (ArcSwap State Management & EventLoopProxy)

---

## Problem Statement

After E02, the magnification overlay renders a static magnified view -- it does not respond to the user's mouse movement. After E03 stories 001 and 002, mouse position data flows from the X11 input monitor through a bounded channel into `ArcSwap<AppState>`, but nothing consumes that position to update *where* on the screen the magnifier is looking. Without a tracking engine, the magnified view remains locked to a fixed position regardless of where the user moves their cursor.

This story implements the `TrackingEngine` in `luminos-core` that bridges mouse position input to viewport output. Each frame, the tracking engine reads the current mouse position, applies smooth interpolation (using the existing `smooth_viewport_position()` from `luminos-gpu::viewport`), enforces a dead zone that prevents jitter from micro-movements, and implements edge panning that shifts the viewport when the cursor approaches the magnified view's boundaries. The result is a viewport that follows the user's cursor fluidly and predictably, which is the fundamental user experience of a screen magnifier.

## User Scenarios

### US-1: Smooth Cursor-Follow Panning

As a low-vision user, I want the magnified view to smoothly follow my mouse cursor so that I can navigate the screen without disorienting jumps or visual stuttering.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given the `TrackingEngine` initialized with default settings (smoothing_factor=0.2) and the cursor at position (960, 540), when the cursor moves to (1200, 600) over a sequence of frames, then the smoothed viewport center converges toward (1200, 600) over 3-5 frames without jumping directly to the target.
- **AC-1.2:** Given the tracking engine with smoothing enabled, when the cursor moves continuously at 60fps, then the viewport position changes smoothly on each frame (no frame-to-frame delta exceeds `(target - current) * smoothing_factor` by more than 1 pixel rounding).
- **AC-1.3:** Given the tracking engine with smoothing disabled (`smoothing_factor=1.0`), when the cursor moves to a new position, then the viewport center immediately matches the cursor position on the next frame.

### US-2: Dead Zone Suppresses Micro-Movement Jitter

As a low-vision user, I want the magnified view to remain still when my cursor makes small movements near the center of the view so that I can read text without the view shifting under me.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given the tracking engine with a dead zone of 20% of viewport dimensions (default) and the viewport centered on (960, 540), when the cursor moves within the dead zone boundaries (e.g., from (960, 540) to (980, 550) on a 1920x1080 viewport where dead zone half-width is 192px), then the tracking engine returns the same viewport center -- no panning occurs.
- **AC-2.2:** Given the cursor at the edge of the dead zone, when the cursor moves beyond the dead zone boundary, then the tracking engine begins panning the viewport toward the cursor.
- **AC-2.3:** Given a dead zone percentage of 0% (dead zone disabled), when any cursor movement occurs, then the viewport pans immediately (subject to smoothing).

### US-3: Edge Panning at Viewport Margins

As a low-vision user, I want the magnified view to pan when my cursor approaches the edge of the magnified area so that I can see content beyond the current viewport without moving my cursor all the way to the center of the new content.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given the tracking engine with an edge margin of 15% of viewport width/height (default) and the cursor at the right edge margin of the magnified viewport, when the cursor is at 90% of the viewport width from the left, then the viewport pans rightward proportionally to how far into the margin the cursor has moved.
- **AC-3.2:** Given the cursor deep in the edge margin (e.g., at 98% of viewport width), when compared to the cursor at 86% of viewport width (just inside the margin), then the panning speed at 98% is faster than at 86% (proportional panning).
- **AC-3.3:** Given the cursor within the content area (between the dead zone and the edge margin), when the cursor moves, then no edge panning occurs -- only standard smooth tracking applies.

### US-4: Viewport Clamped to Screen Bounds

As a low-vision user, I want the magnified view to never show content outside my screen boundaries so that I always see valid screen content regardless of where my cursor moves.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given a 1920x1080 screen and 2x zoom (source region 960x540), when the cursor moves to the top-left corner (0, 0), then the computed source region has `x >= 0` and `y >= 0`.
- **AC-4.2:** Given a 1920x1080 screen and 2x zoom, when the cursor moves to the bottom-right corner (1920, 1080), then the computed source region has `x + width <= 1920` and `y + height <= 1080`.
- **AC-4.3:** Given a multi-monitor setup where the active display has bounds `ScreenRect { x: 1920, y: 0, width: 1920, height: 1080 }`, when the cursor is at position (2880, 540), then the source region is clamped within the active display bounds.

### US-5: Tracking Works Across Zoom Levels

As a low-vision user, I want the cursor-follow behavior to work consistently whether I am at 1.5x or 20x zoom so that changing zoom level does not degrade the tracking experience.

**Priority:** P1
**Acceptance Criteria:**

- **AC-5.1:** Given zoom level 1.5x (source region ~1280x720), when the cursor moves and the tracking engine updates, then the viewport source region has correct dimensions and follows the cursor smoothly.
- **AC-5.2:** Given zoom level 20x (source region ~96x54), when the cursor moves, then the viewport source region has correct dimensions and follows the cursor with the same smoothing behavior as at lower zoom levels.

## Functional Requirements

- **FR-1:** Implement `TrackingEngine` struct in `crates/luminos-core/src/tracking.rs` that holds the current smoothed viewport center, dead zone configuration, edge panning margin configuration, and smoothing factor. *(Traced by US-1, US-2, US-3)*
- **FR-2:** Implement `TrackingEngine::update(mouse_position, viewport_size, screen_bounds, zoom_level, dt) -> ScreenPoint` that computes the new smoothed viewport center each frame, applying dead zone suppression, edge panning, and smooth interpolation via `smooth_viewport_position()`. *(Traced by AC-1.1, AC-1.2, AC-2.1, AC-3.1)*
- **FR-3:** Implement dead zone logic: if the cursor position relative to the current viewport center falls within the dead zone rectangle (configurable as percentage of viewport dimensions, default 20%), suppress panning and return the current viewport center. *(Traced by AC-2.1, AC-2.2, AC-2.3)*
- **FR-4:** Implement edge panning logic: when the cursor is within the edge margin (configurable as percentage of viewport dimensions, default 15%), apply an additional panning velocity proportional to the cursor's depth into the margin. *(Traced by AC-3.1, AC-3.2, AC-3.3)*
- **FR-5:** After computing the smoothed viewport center, call `compute_source_region()` (from `luminos-gpu::viewport`) to produce the final `ScreenRect` clamped to screen bounds. *(Traced by AC-4.1, AC-4.2, AC-4.3)*
- **FR-6:** Implement `TrackingEngine::new(config: TrackingConfig)` constructor accepting configurable parameters (smoothing_factor, dead_zone_percent, edge_margin_percent). *(Traced by FR-1)*
- **FR-7:** Implement `TrackingConfig` struct with validated defaults: `smoothing_factor: 0.2` (range 0.05-1.0), `dead_zone_percent: 0.2` (range 0.0-0.5), `edge_margin_percent: 0.15` (range 0.0-0.3). *(Traced by FR-6)*

## Non-Functional Requirements

- **NFR-1:** `TrackingEngine::update()` must complete in under 0.01ms (10 microseconds) -- it is pure arithmetic with no allocation, I/O, or GPU dependency. *(Source: doc-03 Section 2.3 viewport calculation budget)*
- **NFR-2:** Panning must be visually smooth at all zoom levels from 1.5x to 20x, with no visible jitter or frame-to-frame snapping. *(Source: E03 SC2)*
- **NFR-3:** Mouse movement must update the viewport position within 1 frame (< 16.67ms end-to-end from mouse event to rendered frame). *(Source: E03 SC1)*
- **NFR-4:** No `unwrap()` or `expect()` in production code. Use `?` propagation where fallible, return defaults where infallible.
- **NFR-5:** All public items in `tracking.rs` must have `///` doc-comments.

## Out of Scope

- Focus-follow tracking mode (AT-SPI2 focused element tracking) -- deferred to E07.
- Text caret tracking -- deferred to E07.
- Hybrid tracking mode (switch between mouse-follow and focus-follow based on last input type) -- deferred to E07.
- Lens mode and docked mode viewport behavior -- deferred to E05.
- User-configurable dead zone and edge margin via control panel UI -- deferred to E07. This story uses hardcoded defaults.
- Integration with the render loop (wiring `TrackingEngine` into the event-driven loop) -- handled by Story 005.
- Input event consumption (reading from the mpsc channel) -- handled by Story 005.

## Open Questions

*None -- all design decisions resolved via HIGH_LEVEL_PLAN.md architecture decisions and doc-03 Sections 3.1-3.4.*
