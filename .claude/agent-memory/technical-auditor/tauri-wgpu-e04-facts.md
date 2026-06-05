---
name: tauri-wgpu-e04-facts
description: Verified Tauri 2.11.2 / wgpu 29.0.3 API facts for auditing E04 (control panel) specs
metadata:
  type: reference
---

# Tauri 2.11.2 + wgpu 29.0.3 API Facts (verified 2026-06-04)

## Tauri 2.11.2 (docs.rs)
- `WebviewWindow` does NOT have `request_redraw()` — neither Rust API nor JS API. Any spec driving redraw via `WebviewWindow::request_redraw()` is a bug. Alternatives: emit a tao user event the run-loop turns into a render, or a shared AtomicBool the loop polls each MainEventsCleared tick.
- `WebviewWindow::set_ignore_cursor_events(&self, ignore: bool) -> Result<()>` EXISTS (click-through). Correct.
- `WebviewWindow` implements `HasWindowHandle` + `HasDisplayHandle` (rwh 0.6). Correct.
- `AppHandle::run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()>` EXISTS. Correct API for main-thread marshaling.
- `App::run<F: FnMut(&AppHandle<R>, RunEvent) + 'static>(self, callback: F)` — "never returns". NO ControlFlow / Poll-vs-Wait exposure. The callback is an OBSERVER; you cannot `set_control_flow(Poll)` and cannot `request_redraw` from it. The winit "Poll + request_redraw" framing does NOT map to Tauri's run() API.
- `RunEvent` variants: Exit, ExitRequested, WindowEvent{label,event}, WebviewEvent, Ready, Resumed, MainEventsCleared, MenuEvent, TrayIconEvent. NO top-level `Resized` (it's inside WindowEvent). NO `RedrawRequested`.
- tauri.conf.json: `app.windows[].{transparent,decorations,alwaysOnTop,skipTaskbar}` all valid. `app.macOSPrivateApi` is the macOS gate for transparent bg (macOS only; Linux/X11 transparency just needs a compositor). `bundle.license` IS a valid field.

## wgpu 29.0.3
- `Instance::create_surface<'window>(&self, target: impl Into<SurfaceTarget<'window>>) -> Result<Surface<'window>, CreateSurfaceError>`.
- A BORROWED `&W` yields `Surface<'window>` tied to the borrow, NOT `Surface<'static>`. `OverlayGpu::new<W>(window: &W) -> {surface: Surface<'static>}` is UNSOUND as written.
- To get `Surface<'static>`: pass an OWNED `'static` window (e.g. `Arc<Window>`, the wry wgpu example pattern) or use `create_surface_unsafe(SurfaceTargetUnsafe::from_window(&w))` with a manually-upheld outlives invariant. Tauri `WebviewWindow` is Clone/Arc-backed and 'static — store an owned clone alongside the surface so the surface can be 'static.

## luminos-core reality (E04 relevant)
- `EventNotifier: Send + 'static` (pipeline.rs:24), method `notify_state_changed(&self)`. Blanket impl for `winit::EventLoopProxy<LuminosEvent>`. Second impl (AppNotifier, local type) is orphan-rule-fine.
- `LuminosEvent` has ONLY `StateChanged` + `RequestExit` (no ZoomChanged).
- `AppState { settings: AppSettings, ... }`; zoom at `settings.magnification.zoom_level`. StateManager methods: update_zoom_level/toggle_magnification/reset_zoom/update_mouse_position.
- `luminos-core::config` has ONLY mod.rs + schema.rs — NO `ConfigManager` exists yet (story 004 creates it). DESIGN/HLP referencing `luminos_core::config::ConfigManager` as a managed-state field type won't compile until 004.
- `luminos-app/Cargo.toml`: tauri gated behind `tauri` feature (optional deps). Making non-optional means whole-workspace `cargo build` needs webkit2gtk-4.1/libsoup-3.0.
