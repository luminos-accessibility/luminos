# Story 003 — Implementation Notes (lead briefing, 2026-06-05)

Verified against source at worktree HEAD (stories 001+002 DONE). SUPERSEDES stale DESIGN.md/SUBTASKS
parts — log each conflict in `SUBTASKS.md → Deviations from Design`. This story is **loop-glue only**:
wire existing E2/E3 modules into the run loop; write NO new capture/magnify/track/hotkey logic.

## ⚠️ CWD (every Bash block)
Bare shell starts in `/home/renatorro/Development/luminos` (branch main — 001/002 absent). Work in the
worktree: `cd /home/renatorro/Development/luminos/.claude/worktrees/epic+e04-control-panel`
(branch `worktree-epic+e04-control-panel`). Verify `git branch --show-current` before committing.

## 🔴 #1 correction: TrackingEngine lives in the RENDER LOOP, not the pipeline
`InputProcessingTask::dispatch_event` (`pipeline.rs:126-128`) only does
`state_manager.update_mouse_position(pos)` on `MouseMoved`. It NEVER constructs/updates `TrackingEngine`.
`TrackingEngine` (`tracking.rs`) is stateful (`&mut self`, `current_center`/`initialized`) and must be
updated exactly ONCE per rendered frame → the render loop (a new `CaptureDriver`) owns it. Do NOT move it
into the pipeline (would couple per-event to per-frame smoothing; out of scope). The DESIGN *diagram*
(lines 33-34) is misleading; the DESIGN's `CaptureDriver{ tracking }` text (line 83) is correct.

## A. Frame path (per DC-9 marshaled heartbeat → `present_if_ready`, app.rs:390)
Keep the existing cadence machinery. Replace the body of `present_if_ready`. Each `dirty`-gated tick:
1. Read state lock-free: `let s = app_state.load();` (capture an `Arc<ArcSwap<AppState>>` or `StateManager`
   into the loop closure at `Ready`). Extract `s.is_active`, `s.settings.magnification.zoom_level`,
   `s.mouse_position`.
2. **Inactive short-circuit:** if `!s.is_active` → `OverlayGpu::render_clear()` (now private) + return (T008).
3. **Viewport (loop-owned TrackingEngine):**
   `let center = tracking.update(s.mouse_position, viewport_size, screen_bounds, zoom);` then
   `let region = compute_source_region(center, zoom, viewport_size, screen_bounds);`
   - `TrackingEngine::update(&mut self, ScreenPoint, (u32,u32), ScreenRect, f32) -> ScreenPoint`
     (`tracking.rs:130`) — internally calls `smooth_viewport_position`; do NOT call that yourself.
   - `compute_source_region(ScreenPoint, f32, (u32,u32), ScreenRect) -> ScreenRect` (`viewport.rs:36`) —
     already clamps to `screen_bounds`. `viewport_size` = `OverlayGpu.config.{width,height}`;
     `screen_bounds` = magnified display rect (from `capture.list_displays()` `DisplayInfo.bounds`, or the
     overlay bounds 002 computed at `app.rs:324`).
4. **Capture (overlay excluded — set ONCE at Ready, NOT per frame):**
   `capture.capture_frame(display_id, Some(region)) -> Result<CaptureFrame, CaptureError>`
   (`screen_capture.rs:99`; `&self` — capture is immutable, only `set_excluded_windows` is `&mut`). The
   region is display-global; XcbCapture crops to monitor origin internally.
5. **Render:** `let is_bgra = matches!(frame.format, luminos_types::PixelFormat::Bgra8);`
   `renderer.render_frame(&surface, &frame, is_bgra)?;` (`renderer.rs:122`). render_frame records
   `FrameTimings` internally (FR-6, free).
6. **Capture failure (FR-7):** `renderer.handle_capture_failure()` (`renderer.rs:232`); reuse last texture /
   skip present; never panic.

### OverlayGpu ↔ Renderer ownership (DESIGN.md:63-69 is approximate — real struct has device,queue,config separate)
`wgpu::Device` IS `Clone` (Arc-backed; proof `renderer.rs:86` clones it). Refactor `OverlayGpu`: keep
`surface`, `_window`, `config`, a CLONED `device` (needed for `surface.configure` on resize/Lost/Outdated),
move `queue` into a `Renderer` field built ONCE in `OverlayGpu::new` via
`Renderer::new(device.clone(), queue, config.format, w, h, method)`. `OverlayGpu::frame_timing_summary()`
= `self.renderer.frame_timings().summary(60)`. No `Renderer::reconfigure_surface` needed.

## B. Input + tracking + hotkeys (wire at `Ready`, after init_overlay_gpu/init_window_manager)
```
let monitor = X11InputMonitor::new()?;                       // input.rs:55
let rx = monitor.subscribe_input_events(256)?;               // input.rs:296 — SPAWNS the XI2 thread, returns tokio::sync::mpsc::Receiver
let sm = StateManager::new(Arc::clone(&handle.app_state));   // SAME ArcSwap as the loop (writes visible to load())
let task = InputProcessingTask::spawn(rx, sm, HotkeyMatcher::default(), handle.notifier.clone())?; // pipeline.rs:64 — spawns processor thread
```
- `InputProcessingTask::spawn<N: EventNotifier>(receiver: tokio::sync::mpsc::Receiver<InputEvent>,
  state_manager: StateManager, hotkey_matcher: HotkeyMatcher, notifier: N) -> Result<Self, io::Error>`.
  `blocking_recv()` needs NO tokio runtime.
- `HotkeyMatcher::default()` registers the 7 Phase-0 bindings (Ctrl+Alt + =/−/8/0). Pipeline already routes
  KeyEvent → dispatch_hotkey → StateManager mutation → notify. `dispatch_hotkey` is at
  `luminos_core::hotkeys::dispatch_hotkey` (NOT root). T007 is verification-only.
- `notifier.notify_state_changed()` sets the same `dirty` flag the loop drains → cursor moves + hotkeys wake
  the render (DC-11 wake path). No winit (FR-1 intact: monitor uses x11rb, task uses std::thread).

## C. Per-frame-connect smell (DC-12) — DOCUMENT & DEFER, do not fix here
`XcbCapture::{unmap,remap}_excluded_windows` open a FRESH `x11rb::connect(None)` per call (`capture.rs:171,203`),
run on every `capture_frame` with an exclusion set → ~120 connect/disconnect/sec at 60fps, in the frame
budget. Fixing = platform work (cache a connection + reconnect handling), out of this story's "glue-only"
mandate. RECORD as a Phase-1 perf follow-up (RISK-004) in SUBTASKS Blockers + agent-memory. EMPIRICAL
escape hatch: if the transparent/click-through overlay does NOT self-capture in the AC-1.1 screenshot, call
`set_excluded_windows(&[])` (empty → early-returns, skips unmap/remap, `capture.rs:167,199`) to kill the
flicker+perf cost. Decide from the screenshot; document the decision.

## D. DESIGN/SUBTASKS staleness corrections (apply + log)
1. **`main.rs` → `app.rs` (PERVASIVE).** The run loop is `luminos_app::app::run`/`run_event_loop` in
   `crates/luminos-app/src/app.rs`; `main.rs` is a thin shim. EVERY SUBTASK/DESIGN ref to `main.rs` means `app.rs`.
2. `OverlayGpu` real struct = `surface,_window,device,queue,config` (5 fields, device+queue separate) — see §A.
3. **is_bgra reality:** shipped `XcbCapture` hardcodes `PixelFormat::Rgba8` (`capture.rs:282`); source texture is
   `Rgba8UnormSrgb` (`texture.rs`). So `is_bgra=false` for X11 today. Plumbing still required (FR-2, future
   Windows). DESIGN/STORY prose "xcap yields BGRA" is WRONG — derive from `frame.format`, never assume.
4. **InterpolationMode → InterpolationMethod:** settings `luminos_types::InterpolationMode`
   (`schema.rs:111`, Bilinear/Bicubic) → `Renderer::new` wants `luminos_gpu::shaders::InterpolationMethod`.
   Map at construction. Renderer BAKES the method at build (no runtime switch) → interpolation fixed at
   startup for Phase 0.
5. **FrameTimings vs Summary:** `renderer.frame_timings() -> &FrameTimings` (`renderer.rs:250`); IPC type is
   `FrameTimingSummary` via `FrameTimings::summary(target_fps: u32)` (`frame_timings.rs:162`).
6. **No root re-exports** in luminos-gpu/platform: use full paths — `luminos_gpu::renderer::Renderer`,
   `luminos_gpu::frame_timings::{FrameTimings,FrameTimingSummary}`, `luminos_gpu::shaders::InterpolationMethod`,
   `luminos_gpu::viewport::compute_source_region`, `luminos_platform::traits::ScreenCapture`,
   `luminos_platform::linux_x11::{XcbCapture, X11InputMonitor}`. (Add convenience re-exports in T001 if preferred.)
7. `CaptureDriver` (new, in luminos-app) holds `XcbCapture` (excluded once) + `TrackingEngine`; `set_excluded_windows`
   is `&mut` (init only), `capture_frame` is `&self` (per frame). Types `ScreenPoint/ScreenRect/CaptureFrame/
   PixelFormat` all originate in `luminos-types`, re-exported identically through platform/core — no type-identity conflict.

## E. Subtask order (file paths corrected to app.rs) + test tiers
Tiers: (I) pure unit; (II) Mesa-llvmpipe offscreen GPU unit (`compatible_surface: None`, feature
`ci_platform_tests`); (III) subprocess-under-Xvfb+picom (`tests/common::RunningApp`).
- T001 scaffold + deps + mapping helpers + new `capture_driver.rs` (compile).
- T004 CaptureDriver: region from state+TrackingEngine (I: region == compute_source_region).
- T003 is_bgra from format (I: Bgra8→true, Rgba8→false).
- T002 OverlayGpu hosts Renderer; `render(&CaptureFrame)`; `frame_timing_summary()` (II offscreen).
- T005 wire capture→render into app.rs loop + Ready-time CaptureDriver/TrackingEngine/state (III).
- T006 spawn InputProcessingTask + X11InputMonitor + AppNotifier; join on shutdown (III + xdotool).
- T007 Phase-0 hotkeys verification (III + xdotool `ctrl+alt+equal/minus/8/0`).
- T008 toggle-off (inactive→clear) + capture-failure resilience (I + III).
- T009 frame-timing exposure probe (FrameTimingSummary via handle/debug for story 005) (III; P99 non-zero).
- T010 acceptance + perf (P99<20ms target; expect llvmpipe variance — record) + AC matrix.

**GPU-test caveat (T002/AC-1.1):** `render_frame` calls `output.present()` which FAILS under headless Xvfb
(EGL surfaceless, DC-10). Cover the magnify SHADER pipeline with an OFFSCREEN `TextureView` + readback (like
`crates/luminos-gpu/tests/shader_output.rs`, `compatible_surface: None`); assert present-without-panic via
subprocess LOG markers, not surface readback. Mirror `overlay_gpu_offscreen_render_clear` (`overlay_gpu.rs:208`).
**DC-10 child env (tier III):** `GDK_BACKEND=x11 WEBKIT_DISABLE_COMPOSITING_MODE=1
WEBKIT_DISABLE_DMABUF_RENDERER=1 LIBGL_ALWAYS_SOFTWARE=1 MESA_GL_VERSION_OVERRIDE=4.5`. Window
stacking/always-on-top unobservable WM-less — don't assert them; use x11rb `query_tree`.

## F. Risks/gotchas
- Frame pacing (NFR-1/RISK-004): per-frame connect (§C) + non-SHM `xcb_get_image` (~8ms at 1080p,
  `capture.rs:30`) can blow 20ms at low zoom; small regions at high zoom are cheap. Capture inline, never
  behind a lock on the state path. FrameTimings already warns at P99>20ms×300 frames.
- BGRA/RGBA (§D.3). Self-capture flicker (§C). Software-GL present unavailable (§E caveat). FR-1 intact.
- **Thread shutdown:** retire 2 new threads in `RunEvent::ExitRequested|Exit` teardown (`app.rs:242`): the XI2
  monitor thread exits when its `Receiver` is dropped; the processor exits when its `Sender` is dropped →
  `blocking_recv()` returns `None`. Order: drop the monitor (→ processor's recv returns None → processor
  exits), then `InputProcessingTask::join()` (`pipeline.rs:85`). Store the task + a monitor-drop handle in
  slots alongside `cadence_handle`/`debug_handle` under the existing once-guard.
- `tokio::sync::mpsc` `blocking_recv()` needs NO runtime (proven by existing pipeline tests).

## Carry-forward Minor items from story 002 (opportunistic; log if applied)
- Route `window.rs` property-mutation failures (`set_overlay_bounds`/`set_always_on_top`/`set_visible`) through
  `WindowError::PropertyFailed{property,message}` instead of generic `Platform` — only if you touch window.rs.
- Fix stale `luminos-gpu/src/lib.rs:4` doc comment: "winit-based magnification overlay window" → tao/Tauri.
