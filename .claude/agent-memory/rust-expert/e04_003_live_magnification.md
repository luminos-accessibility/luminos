# E04/003 Live Full-Screen Magnification (loop-glue, done 2026-06-05)

The first epic to drive the engine modules from a real run loop. Loop-glue only —
no new capture/magnify/track/hotkey logic; existing E2/E3 modules driven as-is.

## Frame path (luminos-app)
- Run loop in `app.rs` (`run`/`run_event_loop`/`present_if_ready`), NOT `main.rs` (thin shim).
- Marshaled ~60 Hz heartbeat (DC-9) wakes `MainEventsCleared` → `present_if_ready` loads
  `AppState` lock-free → inactive→`render_clear`; active→`CaptureDriver.region_for_state`
  (advances `TrackingEngine` ONCE) → `capture_region` → `OverlayGpu::render(&CaptureFrame)`.
- `CaptureDriver` (new, `capture_driver.rs`, Linux) owns `XcbCapture` (excluded once at Ready)
  + the loop-owned `TrackingEngine`. TrackingEngine lives in the RENDER LOOP, not the input
  pipeline (it's stateful, advance exactly once/frame). Split region/capture so tracking isn't
  double-advanced.
- `OverlayGpu` hosts the E2 `Renderer` (built once in `new`). `wgpu::Device` AND `wgpu::Queue`
  are both `#[derive(Clone)]` in wgpu 29.0.3 (Arc-backed) → OverlayGpu keeps cloned device+queue
  for surface (re)config + the inactive clear path; Renderer owns its own clones.
- `is_bgra` derived from `frame.format` via `is_bgra_format()`, NEVER assumed. Shipped X11
  `XcbCapture` hardcodes `PixelFormat::Rgba8` → `is_bgra=false` today (DESIGN's "xcap yields BGRA"
  prose is WRONG). Plumbing kept for future Windows DXGI (`Bgra8`).

## Input pipeline (at Ready, Linux)
`X11InputMonitor::new()?` → `subscribe_input_events(256)?` (spawns XI2 thread, returns
`tokio::sync::mpsc::Receiver`; `blocking_recv` needs NO tokio runtime) →
`InputProcessingTask::spawn(rx, StateManager::new(Arc::clone(&handle.app_state)),
HotkeyMatcher::default(), notifier.clone())?`. StateManager wraps the SAME ArcSwap as the loop.

### Shutdown gotcha (important)
`InputProcessingTask` is DROPPED (detached) on shutdown, NOT joined. The X11 XI2 monitor thread
owns the channel Sender (its own connection, independent of the X11InputMonitor struct) and only
releases it on a connection error OR the next event after its Receiver closes; the processor
thread (owns the Receiver) only exits once the Sender drops → CIRCULAR ownership → a blocking
`join()` can HANG shutdown. Detach + process-exit reap is correct. (IMPLEMENTATION_NOTES #8 said
join; that would hang.)

## Story-005 seam (DC-13)
- Frame timings: `LuminosHandle.frame_timings: Arc<Mutex<luminos_gpu::FrameTimingSummary>>`
  (zeroed init). Loop calls `handle.set_frame_timings(gpu.frame_timing_summary())` each presented
  frame; story 005 reads `handle.frame_timings()`. Fields snake_case (`average_ms`/`p99_ms`/...);
  005 adds `#[serde(rename_all="camelCase")]` + `specta::Type`.
- State writes (IPC commands): `StateManager::new(Arc::clone(&handle.app_state))` → mutate →
  `handle.notifier.notify_state_changed()`. Loop reads next frame. zoom at
  `settings.magnification.zoom_level`, mode at `.mode`, `is_active` top-level.
- Interpolation BAKED at startup (`interpolation_method_for(InterpolationMode)` → no runtime switch).

## DC-10 / env reality (CRITICAL for testing)
- Headless Xvfb has NO surface-compatible wgpu adapter: `create_gpu_device(&instance, &surface)`
  requests `compatible_surface: Some(&surface)` → EGL "surfaceless"/DRI2 fail → `OverlayGpu::new`
  errors `NoAdapter`. So the LIVE magnify present + P99>0 are UNOBSERVABLE under headless Xvfb AND
  CI software GL. Covered structurally: offscreen `Renderer` unit test (`compatible_surface: None`)
  + `luminos-gpu::shader_output` (magnify shader correctness). Live assertion → story 007/real GPU.
- The 2 `luminos-gpu::integration::render_pipeline_*_shader_renders` tests FAIL on a dev box
  without EGL/DRI2/software-Vulkan (`NoAdapter`) — pre-existing env flakiness, pass on CI llvmpipe.
- Offscreen wgpu (`compatible_surface: None`, GL/swrast) DOES work on this box.

## xcap Wayland leak (dev box)
xcap auto-selects Wayland when `WAYLAND_DISPLAY`/`XDG_SESSION_TYPE=wayland` are set, even under
X11 `DISPLAY` → "Cannot find required wayland protocol". For X11 capture tests on a dev box with a
live Wayland session: `remove_var("WAYLAND_DISPLAY"); set_var("XDG_SESSION_TYPE","x11")` (safe under
nextest process-per-test). CI's pure-X11 `xvfb-run` has no leak. The actual xcap capture round-trip
also fails on this box's software Xvfb ("Connection error") — env flakiness, passes on CI.

## Test-only env hooks (gated, never production)
`LUMINOS_FORCE_ACTIVE=1` (seed is_active=true), `LUMINOS_LOG_STATE=1` (log `state mouse/zoom/active`
on change), `LUMINOS_NO_EXCLUDE=1` (skip self-capture exclusion). Subprocess tests assert via these
+ `tests/common::RunningApp` (own Xvfb :180+ per test) + xdotool. Markers: `capture_driver=ready`,
`input_pipeline=ready`, `magnify_region`/`magnify_capture`/`magnify_present`, `state ...`.

## RISK-004 follow-up (deferred, SUBTASKS B002)
`XcbCapture::{unmap,remap}_excluded_windows` open a FRESH `x11rb::connect(None)` per captured frame
when an exclusion set is active (~120 connect/sec at 60fps). Phase-1 fix: cache a connection in
XcbCapture or reuse X11WindowManager's persistent RustConnection. Escape hatch: `LUMINOS_NO_EXCLUDE=1`.

## Tooling note
`cargo-nextest` IS installed but only on `~/.cargo/bin` — non-interactive shells need
`export PATH="$HOME/.cargo/bin:$PATH"`. `luminos-app` builds with the `tauri` feature here
(webkit2gtk-4.1/jsc-4.1/libsoup-3.0 all present).
