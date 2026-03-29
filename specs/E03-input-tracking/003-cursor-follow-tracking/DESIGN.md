# Design: Story E03/003 -- Cursor-Follow Viewport Tracking

**Story:** [STORY.md](STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** spec-writer-2
**Risk Refs:** RISK-004 (render thread starvation -- tracking engine must be sub-microsecond), RISK-006 (multi-display coordinate inconsistencies -- clamping to screen bounds)

---

## Overview

Implement the `TrackingEngine` in `luminos-core` that bridges mouse position input to viewport output. Each frame, the render loop reads the current mouse position from `ArcSwap<AppState>` (Story 002's `StateManager`), passes it to `TrackingEngine::update()`, which computes a new smoothed viewport center using three mechanisms: (1) dead zone suppression for micro-movements, (2) smooth interpolation via `smooth_viewport_position()` from `luminos-gpu::viewport`, and (3) edge panning when the cursor approaches viewport margins. The resulting viewport center is then passed to `compute_source_region()` (also from `luminos-gpu::viewport`) to produce the final `ScreenRect` clamped to screen bounds.

The `TrackingEngine` is a pure-logic component: no I/O, no GPU dependency, no allocation on the hot path. It is fully deterministic given the same inputs, making it exhaustively unit-testable. The engine lives in `luminos-core` (not `luminos-gpu`) because it is application logic -- it orchestrates viewport math functions, it does not perform rendering.

## Architecture

### Component Diagram

```
luminos-core/src/
  lib.rs                    [Modified] Add `pub mod tracking;`
  tracking.rs               [New]      TrackingEngine, TrackingConfig
  state.rs                  [Existing] AppState (with mouse_position from Story 002)
  state_manager.rs          [From Story 002] StateManager
  config/
    schema.rs               [Existing] MagnificationSettings (smooth_scrolling)

luminos-gpu/src/
  viewport.rs               [Existing, unchanged] compute_source_region(), smooth_viewport_position()
```

```
Per-frame data flow (in render loop, wired by Story 005):

  ArcSwap<AppState>
       |
       | load() -> AppState { mouse_position, settings.magnification.zoom_level, ... }
       v
  +-------------------+
  | TrackingEngine    |
  | .update()         |
  |                   |
  | 1. Dead zone check|-----> No panning (return current center)
  | 2. Edge panning   |-----> Compute pan velocity from margin depth
  | 3. Smooth interp  |-----> smooth_viewport_position(current, target, factor)
  |                   |
  +--------+----------+
           |
           v  ScreenPoint (smoothed viewport center)
  +-------------------+
  | compute_source_   |
  | region()          |
  | (luminos-gpu)     |
  +--------+----------+
           |
           v  ScreenRect (clamped to screen bounds)
  +-------------------+
  | AppState.viewport |
  | (updated via      |
  |  StateManager)    |
  +-------------------+
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `luminos-core::tracking` | New | `TrackingEngine`, `TrackingConfig` structs |
| `luminos-core::lib.rs` | Modified | Add `pub mod tracking;` and re-export `TrackingEngine`, `TrackingConfig` |
| `luminos-gpu::viewport` | Unchanged | `compute_source_region()`, `smooth_viewport_position()` used as-is |
| `luminos-core::state` | Unchanged | `AppState` already extended with `mouse_position` by Story 002 |

### Data Flow

1. **Initialization:** Application creates `TrackingEngine::new(TrackingConfig::default())`. The engine stores the initial viewport center at `ScreenPoint { x: 0, y: 0 }` (will be set to the cursor position on the first frame).

2. **Per-frame update (called by Story 005's render loop):**
   a. Read `AppState` from `ArcSwap` via `StateManager::load()`.
   b. Extract `mouse_position`, `settings.magnification.zoom_level`, `settings.magnification.smooth_scrolling`, viewport size, and screen bounds.
   c. Call `tracking_engine.update(mouse_position, viewport_size, screen_bounds, zoom_level)`.
   d. The engine internally:
      - Computes the cursor position relative to the current viewport center.
      - Checks if the relative position falls within the dead zone -> if yes, returns current center (no panning).
      - If outside dead zone, computes the panning target considering edge panning adjustments.
      - Applies `smooth_viewport_position(current_center, target, smoothing_factor)` to get the new smoothed center.
      - Stores the new center internally for the next frame.
   e. Returns the new smoothed center as `ScreenPoint`.
   f. The render loop calls `compute_source_region(smoothed_center, zoom_level, viewport_size, screen_bounds)` to get the final `ScreenRect`.

## API Design

### TrackingConfig

```rust
/// Configuration for the viewport tracking engine.
///
/// Controls dead zone size, edge panning margins, and smooth
/// interpolation behavior. Defaults are tuned for comfortable
/// magnification navigation.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackingConfig {
    /// Smoothing factor for viewport interpolation (0.05 to 1.0).
    ///
    /// Lower values produce smoother but slower panning.
    /// Higher values produce more responsive but potentially jerky panning.
    /// - 1.0 = instant (no smoothing, viewport jumps to target)
    /// - 0.2 = default (smooth, comfortable panning over 3-5 frames)
    /// - 0.05 = very smooth (slow convergence, may feel sluggish)
    pub smoothing_factor: f32,

    /// Dead zone as a fraction of viewport dimensions (0.0 to 0.5).
    ///
    /// When the cursor's offset from the viewport center is within
    /// `dead_zone_percent * viewport_dimension / 2` pixels in each axis,
    /// no panning occurs. This prevents jitter from small cursor movements
    /// while the user is reading.
    /// - 0.0 = no dead zone (any movement pans)
    /// - 0.2 = default (20% of viewport is dead zone)
    /// - 0.5 = maximum (50% dead zone, very stable but less responsive)
    pub dead_zone_percent: f32,

    /// Edge panning margin as a fraction of viewport dimensions (0.0 to 0.3).
    ///
    /// When the cursor is within `edge_margin_percent * viewport_dimension`
    /// pixels of the viewport edge, the viewport pans proportionally to
    /// the cursor's depth into the margin. Panning speed increases as the
    /// cursor moves deeper into the margin.
    /// - 0.0 = no edge panning
    /// - 0.15 = default (15% of viewport width/height is edge margin)
    /// - 0.3 = maximum edge margin
    pub edge_margin_percent: f32,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            smoothing_factor: 0.2,
            dead_zone_percent: 0.2,
            edge_margin_percent: 0.15,
        }
    }
}
```

### TrackingEngine

```rust
use luminos_types::ScreenPoint;
use luminos_gpu::viewport::smooth_viewport_position;

/// Viewport tracking engine for cursor-follow magnification.
///
/// Computes the smoothed viewport center each frame based on the
/// current mouse position, applying dead zone suppression, edge
/// panning, and smooth interpolation. The engine is a pure-logic
/// component with no I/O, GPU, or allocation on the hot path.
///
/// # Usage
///
/// ```ignore
/// let mut engine = TrackingEngine::new(TrackingConfig::default());
/// // Per frame:
/// let center = engine.update(mouse_pos, viewport_size, screen_bounds, zoom_level);
/// let source_region = compute_source_region(center, zoom_level, viewport_size, screen_bounds);
/// ```
pub struct TrackingEngine {
    /// Configuration (dead zone, edge margin, smoothing).
    config: TrackingConfig,
    /// Current smoothed viewport center in screen coordinates.
    current_center: ScreenPoint,
    /// Whether the engine has received its first update.
    initialized: bool,
}

impl TrackingEngine {
    /// Creates a new tracking engine with the given configuration.
    pub fn new(config: TrackingConfig) -> Self {
        Self {
            config,
            current_center: ScreenPoint { x: 0, y: 0 },
            initialized: false,
        }
    }

    /// Updates the viewport center based on the current mouse position.
    ///
    /// Call this once per frame. Returns the new smoothed viewport center.
    ///
    /// On the first call, sets the viewport center directly to the mouse
    /// position (no smoothing on initialization).
    ///
    /// # Arguments
    ///
    /// * `mouse_position` -- Current cursor position in screen coordinates.
    /// * `viewport_size` -- Overlay viewport dimensions `(width, height)` in pixels.
    /// * `screen_bounds` -- Active display bounds (for edge panning reference).
    /// * `zoom_level` -- Current magnification factor (affects source region size but
    ///   not tracking behavior directly).
    ///
    /// # Returns
    ///
    /// The new smoothed viewport center as a `ScreenPoint`. Pass this to
    /// `compute_source_region()` to get the capture region.
    #[must_use]
    pub fn update(
        &mut self,
        mouse_position: ScreenPoint,
        viewport_size: (u32, u32),
        screen_bounds: luminos_types::ScreenRect,
        zoom_level: f32,
    ) -> ScreenPoint {
        // First frame: snap to mouse position (no smoothing).
        if !self.initialized {
            self.current_center = mouse_position;
            self.initialized = true;
            return self.current_center;
        }

        // Step 1: Dead zone check.
        // Compute cursor offset from current center in "viewport fraction" space.
        let half_vw = viewport_size.0 as f32 / (2.0 * zoom_level);
        let half_vh = viewport_size.1 as f32 / (2.0 * zoom_level);
        let dead_half_x = half_vw * self.config.dead_zone_percent;
        let dead_half_y = half_vh * self.config.dead_zone_percent;

        let dx = (mouse_position.x - self.current_center.x) as f32;
        let dy = (mouse_position.y - self.current_center.y) as f32;

        let in_dead_zone = dx.abs() <= dead_half_x && dy.abs() <= dead_half_y;
        if in_dead_zone {
            return self.current_center;
        }

        // Step 2: Compute panning target.
        // Start with mouse position as the base target.
        let mut target = mouse_position;

        // Step 3: Edge panning adjustment.
        // Compute source region dimensions at the current zoom level.
        let source_w = viewport_size.0 as f32 / zoom_level;
        let source_h = viewport_size.1 as f32 / zoom_level;
        let edge_margin_x = source_w * self.config.edge_margin_percent;
        let edge_margin_y = source_h * self.config.edge_margin_percent;

        // Check if cursor is in the edge margin of the current viewport.
        // Edge margin is measured from the edge of the source region.
        let source_left = self.current_center.x as f32 - source_w / 2.0;
        let source_right = source_left + source_w;
        let source_top = self.current_center.y as f32 - source_h / 2.0;
        let source_bottom = source_top + source_h;

        let mx = mouse_position.x as f32;
        let my = mouse_position.y as f32;

        // Proportional panning: velocity scales with depth into margin.
        // Max pan velocity = edge_margin pixels per frame (at the very edge).
        if mx < source_left + edge_margin_x && edge_margin_x > 0.0 {
            let depth = (source_left + edge_margin_x - mx) / edge_margin_x;
            target.x -= (depth * edge_margin_x) as i32;
        } else if mx > source_right - edge_margin_x && edge_margin_x > 0.0 {
            let depth = (mx - (source_right - edge_margin_x)) / edge_margin_x;
            target.x += (depth * edge_margin_x) as i32;
        }

        if my < source_top + edge_margin_y && edge_margin_y > 0.0 {
            let depth = (source_top + edge_margin_y - my) / edge_margin_y;
            target.y -= (depth * edge_margin_y) as i32;
        } else if my > source_bottom - edge_margin_y && edge_margin_y > 0.0 {
            let depth = (my - (source_bottom - edge_margin_y)) / edge_margin_y;
            target.y += (depth * edge_margin_y) as i32;
        }

        // Step 4: Smooth interpolation toward target.
        self.current_center = smooth_viewport_position(
            self.current_center,
            target,
            self.config.smoothing_factor,
        );

        self.current_center
    }

    /// Returns the current smoothed viewport center.
    #[must_use]
    pub fn current_center(&self) -> ScreenPoint {
        self.current_center
    }

    /// Returns a reference to the tracking configuration.
    #[must_use]
    pub fn config(&self) -> &TrackingConfig {
        &self.config
    }

    /// Updates the tracking configuration.
    ///
    /// Takes effect on the next `update()` call.
    pub fn set_config(&mut self, config: TrackingConfig) {
        self.config = config;
    }
}
```

## Error Handling

This story introduces no new error types. All `TrackingEngine` methods are infallible:
- `new()` is a simple struct construction.
- `update()` performs pure arithmetic -- no I/O, no allocation, no fallible operations.
- `current_center()` and `config()` are simple field accessors.

The `smooth_viewport_position()` function (from `luminos-gpu::viewport`) is also infallible -- it clamps the smoothing factor to [0.0, 1.0] and performs pure arithmetic.

Division by zero is prevented by checking `edge_margin_x > 0.0` and `edge_margin_y > 0.0` before computing proportional depth. If `zoom_level <= 0.0`, `compute_source_region()` (called by the render loop, not by `TrackingEngine`) already handles this by returning a zero-size region.

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| All platforms | Pure Rust arithmetic | No platform-specific code. `TrackingEngine` is fully portable. |

The tracking engine is platform-independent. It consumes `ScreenPoint` values (from the platform-specific `InputMonitor`) and produces `ScreenPoint` values (consumed by the platform-independent `compute_source_region()`). Platform-specific coordinate systems are handled by the `InputMonitor` implementation (Story 001) before the data reaches the tracking engine.

## Testing Strategy

### Unit Tests

- **TrackingConfig defaults:** Verify default values (smoothing_factor=0.2, dead_zone_percent=0.2, edge_margin_percent=0.15).
- **First frame initialization:** On the first `update()` call, the returned center matches the mouse position exactly (no smoothing).
- **Dead zone suppression:** Cursor moves within dead zone -> center does not change.
- **Dead zone boundary:** Cursor exactly at dead zone edge -> no panning. Cursor just outside -> panning begins.
- **Dead zone disabled (0%):** Any cursor movement causes panning.
- **Smooth interpolation:** Multiple `update()` calls with same target -> center converges over frames.
- **Instant tracking (factor=1.0):** Center immediately matches target on each frame.
- **Edge panning activation:** Cursor in edge margin -> viewport pans proportionally.
- **Edge panning proportional speed:** Cursor deeper in margin -> faster panning.
- **Edge panning inactive in content area:** Cursor between dead zone and edge margin -> no edge panning, only smooth tracking.
- **Multiple zoom levels:** Verify dead zone and edge margin scale correctly at 1.5x, 2x, 5x, 10x, 20x zoom.
- **Screen boundary clamping:** Tested via `compute_source_region()` in integration, but tracking engine output (viewport center) should remain within reasonable bounds.

### Integration Tests

- **TrackingEngine + compute_source_region round-trip:** Call `update()` then `compute_source_region()`, verify the resulting `ScreenRect` is within screen bounds.
- **Performance micro-benchmark:** Measure `update()` latency over 10K iterations, verify average < 10us (0.01ms target from NFR-1).

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Unit | Initialize with default config, call `update()` 5 times moving cursor (960,540)->(1200,600), verify center converges over 3-5 frames without jumping |
| AC-1.2 | Unit | Call `update()` 60 times with continuous cursor movement, verify each frame-to-frame delta <= `(target-current) * factor` + 1 pixel |
| AC-1.3 | Unit | Set `smoothing_factor=1.0`, call `update()`, verify center == mouse_position |
| AC-2.1 | Unit | Set dead_zone_percent=0.2, viewport 1920x1080, move cursor within dead zone, verify center unchanged |
| AC-2.2 | Unit | Move cursor from within dead zone to just outside, verify center begins changing |
| AC-2.3 | Unit | Set dead_zone_percent=0.0, move cursor by 1 pixel, verify center changes |
| AC-3.1 | Unit | Position cursor at 90% viewport width, verify `update()` shifts center rightward |
| AC-3.2 | Unit | Compare panning at 86% vs 98% viewport width, verify deeper margin produces larger shift |
| AC-3.3 | Unit | Position cursor in content area (between dead zone and edge margin), verify no edge panning component |
| AC-4.1 | Unit + Integration | 1920x1080 screen, 2x zoom, cursor at (0,0), call `update()` then `compute_source_region()`, verify x >= 0 and y >= 0 |
| AC-4.2 | Unit + Integration | Same setup, cursor at (1920,1080), verify x+width <= 1920 and y+height <= 1080 |
| AC-4.3 | Unit + Integration | Multi-monitor bounds `{x:1920, y:0, w:1920, h:1080}`, cursor at (2880,540), verify source region within bounds |
| AC-5.1 | Unit | Set zoom=1.5, verify dead zone and edge margin scale to source region dimensions and tracking works |
| AC-5.2 | Unit | Set zoom=20, verify tracking works with very small source region |

## Performance Targets

| Metric | Target | Source |
|--------|--------|--------|
| `TrackingEngine::update()` latency | < 0.01ms (10us) | NFR-1, doc-03 Section 2.3 |
| Allocations per `update()` call | 0 | Pure arithmetic on stack |
| CPU cache footprint | ~128 bytes (TrackingEngine struct) | Fits in L1 cache line |

## Security Considerations

No security implications. The `TrackingEngine` processes screen coordinates (non-sensitive public information) and performs pure arithmetic. No I/O, no network, no file access, no user data beyond cursor position.

## Alternatives Considered

### Alternative 1: Tracking engine in luminos-gpu (rejected)

The tracking engine could live alongside `compute_source_region()` in `luminos-gpu::viewport`. Rejected because: (1) the tracking engine is application logic (state management, configuration), not GPU code; (2) placing it in `luminos-gpu` would create a circular dependency if it needs to read `AppState` from `luminos-core`; (3) `luminos-core` is the correct home for application-level orchestration logic that consumes types from `luminos-types` and functions from `luminos-gpu`.

### Alternative 2: Delta-time-based smoothing (considered, deferred)

The smoothing factor could be scaled by delta time (`dt`) to make panning speed frame-rate-independent: `effective_factor = 1.0 - (1.0 - factor).powf(dt * 60.0)`. This ensures consistent panning speed whether running at 30fps or 120fps. The current design uses a fixed factor per frame, which is acceptable at the constant 60fps target. Frame-rate-independent smoothing can be added in a future story if variable frame rates are introduced (E05 adaptive frame rate).

### Alternative 3: Separate dead zone and tracking target (rejected)

A more complex design would have the dead zone centered on the tracking target (not the viewport center), with the tracking target updating only when the cursor exits the dead zone. Rejected in favor of the simpler model where the dead zone is centered on the current viewport center and the cursor position is the panning target. The simpler model is easier to reason about and test, and matches the behavior described in doc-03 Section 3.4.
