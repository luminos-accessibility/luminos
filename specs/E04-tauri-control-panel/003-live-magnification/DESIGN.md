# Design: Story E04/003 -- Live Full-Screen Magnification Integration

**Story:** [STORY.md](./STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** principal-architect
**Risk Refs:** RISK-002 (self-capture — consumes story 002's exclusion), RISK-004 (capture/render pipeline stalls), RISK-016 (wgpu backend compat)

---

## Overview

Wire the existing engine modules into the `luminos-app` run loop established by story 001. This story adds **no** new capture/magnify/track logic — it reuses `luminos_gpu::Renderer` (E2), the X11 `ScreenCapture` + `InputMonitor` backends (E2/E3), and `luminos_core::{StateManager, TrackingEngine, HotkeyMatcher, pipeline::InputProcessingTask}` (E3). The new code is: (1) refactor story-001 `OverlayGpu` to host a `Renderer` and call `render_frame` each redraw with a freshly captured frame, and (2) spawn the input pipeline so cursor/hotkeys mutate `AppState`, waking the loop via the story-001 `AppNotifier`.

## Architecture

### Component Diagram

```
  luminos-app run loop (single tao loop, story 001)
  ┌───────────────────────────────────────────────────────────────────────┐
  │ RunEvent::MainEventsCleared (dirty-gated + cadence):                    │
  │   state = app_state.load()                       // lock-free (ArcSwap) │
  │   frame = screen_capture.capture_frame(region)   // xcap, excl. overlay │
  │   overlay_gpu.render(&frame, &state)             // → Renderer::render_frame
  │     └─ Renderer (owns device/queue, E2) renders magnified region        │
  │   (FrameTimings recorded inside render_frame)                            │
  └───────────────────────────────────────────────────────────────────────┘
        ▲ writes (StateManager)                         │ exposes timings
  ┌─────┴───────────────────────────────────────────────▼──────────────┐
  │ InputProcessingTask (E3, own thread)                                 │
  │   X11InputMonitor → mpsc::Receiver<InputEvent>                       │
  │   MouseMoved → StateManager.update_mouse_position + TrackingEngine   │
  │   KeyEvent → HotkeyMatcher → StateManager.update_zoom/toggle/reset   │
  │   → AppNotifier.notify_state_changed() (sets dirty flag, story 001)  │
  └──────────────────────────────────────────────────────────────────────┘
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `luminos-app/src/overlay_gpu.rs` | Extended | Host a `luminos_gpu::Renderer`; replace `render_clear` with `render(&CaptureFrame, &AppState)`. |
| `luminos-app/src/main.rs` | Modified | Per-redraw: capture → render; spawn `InputProcessingTask`; pass self-capture exclusion (story 002). |
| `luminos-app/src/capture_driver.rs` | New | Thin glue: owns the `ScreenCapture` backend + viewport/region computation calling the E3 `TrackingEngine`. |
| `luminos_gpu::Renderer` | Reused (no change) | `new`, `render_frame`, `handle_capture_failure`, `resize`, `frame_timings`. |
| `luminos_core` (StateManager, TrackingEngine, HotkeyMatcher, pipeline) | Reused (no change) | Driven, not modified. |
| `luminos-platform` X11 capture + input | Reused (no change) | `ScreenCapture::capture_frame`, `X11InputMonitor::subscribe_input_events`. |

### Data Flow (per frame)
1. `state = app_state.load()` (lock-free). Extract `zoom_level`, viewport center, `is_active`.
2. If `!is_active`, present a transparent/empty frame (toggle-off path) and continue.
3. `region = compute_source_region(viewport_center, zoom, display_bounds)` (E2 `luminos_gpu::viewport::compute_source_region`).
4. `frame = screen_capture.capture_frame(region)` with overlay excluded (story-002 mechanism). On `Err`, `renderer.handle_capture_failure()` and reuse last frame.
5. `is_bgra = matches!(frame.format, PixelFormat::Bgra8)`.
6. `renderer.render_frame(&surface, &frame, is_bgra)` — magnifies + presents; records `FrameTimings`.

Input path (separate thread, E3): `MouseMoved` → `StateManager.update_mouse_position` + `TrackingEngine.update(...)` → viewport in state; `KeyEvent` → `HotkeyMatcher.match_event` → `dispatch_hotkey` → `StateManager.update_zoom_level/toggle_magnification/reset_zoom`; then `AppNotifier.notify_state_changed()` sets the dirty flag.

## API Design

```rust
// luminos-app/src/overlay_gpu.rs (extended)
pub(crate) struct OverlayGpu {
    surface: wgpu::Surface<'static>,
    _window: tauri::WebviewWindow,
    renderer: luminos_gpu::Renderer,   // owns device/queue (E2)
    config: wgpu::SurfaceConfiguration,
}
impl OverlayGpu {
    pub fn new(window: tauri::WebviewWindow, w: u32, h: u32,
               method: luminos_gpu::InterpolationMethod) -> Result<Self, AppError>;
    /// Capture-driven frame: magnify `frame` and present.
    pub fn render(&mut self, frame: &luminos_types::CaptureFrame) -> Result<(), AppError>;
    pub fn handle_capture_failure(&mut self);
    pub fn resize(&mut self, w: u32, h: u32);
    pub fn frame_timing_summary(&self) -> luminos_gpu::FrameTimingSummary;
}

// luminos-app/src/capture_driver.rs (new)
pub(crate) struct CaptureDriver {
    capture: Box<dyn luminos_platform::ScreenCapture>, // X11 backend
    tracking: luminos_core::TrackingEngine,
}
impl CaptureDriver {
    pub fn new(/* display_id, overlay_xid for exclusion */) -> Result<Self, AppError>;
    /// Compute region from state + capture (excluding overlay).
    pub fn capture(&mut self, state: &luminos_core::AppState)
        -> Result<luminos_types::CaptureFrame, AppError>;
}
```

> **Device/queue/surface ownership.** `Renderer::new` consumes `device`/`queue`. `OverlayGpu::new` creates instance/adapter/device/queue, configures the surface, then moves `device`/`queue` into `Renderer`. Surface (re)configuration on resize needs a device reference — confirm whether `wgpu::Device` is cloneable in 29.0.3 (it is `Clone` via internal `Arc` in recent wgpu) or expose a `Renderer::reconfigure_surface(&surface, &config)` helper. **This is the one implementation-time wgpu API confirmation for this story** (flagged so it isn't asserted blindly).

## Error Handling
- Capture errors → `renderer.handle_capture_failure()` + reuse last frame (no panic). `CaptureError` mapped to a logged `warn!`; sustained failure escalates to `error!`.
- Render errors (`RenderError`) → mapped to `AppError::Gpu`; `SurfaceError::Lost/Outdated` → reconfigure.
- Input thread errors (channel closed on shutdown) → graceful task exit.
- `?` propagation; no `unwrap`/`expect`.

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | xcap capture + XInput2 monitor (E2/E3); self-capture exclusion (story 002). | Only platform this story. |
| Others | Deferred (capture/input backends are per-platform; the loop wiring is portable). | — |

## Testing Strategy

### In-process / unit
- `capture_driver_computes_region_from_state` — region == `compute_source_region(center, zoom, bounds)` for sample states (pure).
- `overlay_gpu_render_magnifies` (GPU, Mesa) — feed a known `CaptureFrame`, render to an offscreen target, assert the output is the source scaled by zoom (sample pixels). Reuses E2 renderer tests' approach.
- `is_bgra_derived_from_format` — `Bgra8`→true, `Rgba8`→false.

### Subprocess integration (Xvfb+picom, Mesa)
- `live_magnification_presents` (AC-1.1) — display a known pattern; assert the overlay shows it magnified (screenshot diff / logged sample).
- `zoom_change_reflected` (AC-1.2) — change zoom in state (debug trigger); assert next-frame scale changes.
- `cursor_moves_viewport` (AC-2.1) — `xdotool mousemove`; assert viewport center in state/log shifts past the dead zone.
- `hotkeys_drive_state` (AC-3.1) — `xdotool key ctrl+alt+equal` etc.; assert zoom/toggle/reset in state/log.
- `frame_timings_populated` (AC-3.2) — after a render window, a debug log / probe reports non-zero P99.

### Acceptance Tests

| AC | Test Type | Verification |
|----|-----------|--------------|
| AC-1.1 | Subprocess + GPU unit | Magnified pattern shown; offscreen render scales source by zoom. |
| AC-1.2 | Subprocess | Zoom change → next-frame scale change. |
| AC-2.1 | Subprocess (xdotool) | Viewport center follows pointer past dead zone. |
| AC-3.1 | Subprocess (xdotool) | Each Phase-0 hotkey mutates state correctly. |
| AC-3.2 | Subprocess/probe | `FrameTimingSummary` P99 non-zero. |

## Performance Targets
- P99 frame time < 20 ms (NFR-1); warn beyond threshold (doc-07).
- Input→viewport < 1 frame (NFR-2, inherited from E3).

## Security Considerations
- Screen capture reads desktop pixels (already in E2 threat model, RISK-017: `CaptureFrame` Debug omits pixel data). No new surface.

## Alternatives Considered
1. **Re-implement capture/magnify in the loop.** Rejected — E2/E3 modules are tested; reuse them.
2. **Capture on a dedicated thread, double-buffer to the render loop.** Deferred — full pipelining is RISK-004 perf work; this story captures inline but must not stall (NFR-3). Revisit if P99 misses 20 ms.
3. **Drive zoom directly from hotkey thread into the renderer.** Rejected — all state flows through `StateManager`/`ArcSwap`; the render loop reads it. Single source of truth (AD-4).
