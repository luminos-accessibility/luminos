# Story 001 — Implementation Notes (lead briefing, 2026-06-04)

Verified against real source: Tauri 2.11.2 (`~/.cargo/registry/.../tauri-2.11.2/`), the in-repo
engine crates, and `Cargo.lock`. Resolved transitive stack: **tao 0.35.3, wry 0.55.1, gtk 0.18.2,
webkit2gtk 2.0.2**. These notes SUPERSEDE the stale parts of DESIGN.md — read both, prefer these
where they conflict, and log each conflict in `SUBTASKS.md → Deviations from Design`.

## CRITICAL CORRECTIONS to DESIGN.md (must apply)

1. **ConfigManager is REAL — do NOT create a stub.** DESIGN.md §60/§80–97 + SUBTASKS T003 say to
   land an empty `ConfigManager` stub and init `config = None`. Story 004 is DONE: `luminos_core::{ConfigManager, ConfigError, seed_initial_state}` are fully implemented and crate-root re-exported.
   T003 MUST call the real startup seam:
   ```rust
   let (state, config) = match luminos_core::seed_initial_state() {
       Ok((s, m)) => (s, Some(m)),
       Err(e) => { log::warn!("config seed failed: '{e}'; using defaults"); (luminos_core::AppState::default(), None) }
   };
   let app_state = std::sync::Arc::new(arc_swap::ArcSwap::from_pointee(state));
   // LuminosHandle.config = Arc<Mutex<Option<ConfigManager>>> = Arc::new(Mutex::new(config))
   ```
   `AppError` should gain `From<ConfigError>`.

2. **`create_gpu_device` is ASYNC; there is no runtime on the loop thread.** `luminos_gpu::device::create_gpu_device(&Instance, &Surface) -> impl Future<Output=Result<(Adapter,Device,Queue), RenderError>>`. Bridge with a blocking call. FIRST check how `luminos-gpu` already drives it (it may already pull `pollster`/`futures` as a dep or dev-dep — reuse that). If a NEW dependency is required, pin it per the supply-chain rule (latest version published ≤2026-05-21, advisory-free, not yanked) and ADD it to `PINNED_VERSIONS.md` before using it. Likely `pollster::block_on(...)` under the `tauri` feature.

3. **Reuse existing GPU helpers — do not hand-roll.** `luminos_gpu::device::create_wgpu_instance()`, `device::create_gpu_device(...)`, `surface::configure_surface(&Surface,&Adapter,&Device,w,h,PresentMode)`, `surface::select_alpha_mode(...)` (already degrades PreMultiplied→PostMultiplied→Opaque with warnings). Confirm they're reachable from `luminos-app` (crate-root re-export if needed).

## A. Integration seams (real signatures)

- `EventNotifier` (`luminos-core/src/pipeline.rs:24`): `trait EventNotifier: Send + 'static { fn notify_state_changed(&self); }`. `AppNotifier { dirty: Arc<AtomicBool> }` impls it (Clone+Send). Leave the existing blanket impl for `winit::EventLoopProxy<LuminosEvent>` (`pipeline.rs:29`) UNTOUCHED — it backs 418 existing tests.
- `StateManager` (`state_manager.rs:39`): `new(Arc<ArcSwap<AppState>>)`, `load() -> Guard<Arc<AppState>>` (lock-free render read), `inner() -> Arc<ArcSwap<AppState>>`. Story 001 only constructs + reads; mutators (`update_zoom_level` clamps [1.5,20], `toggle_magnification`, `reset_zoom`, `update_mouse_position`) belong to stories 003/005.
- ConfigManager seam: `seed_initial_state() -> Result<(AppState, ConfigManager), ConfigError>` (`manager.rs:326`); `ConfigError::{Io{path,source}, Serialize(#[from] toml::ser::Error), NoConfigDir}`.
- `Renderer` is story 003, NOT 001 (001 does clear-frame only). For reference: `Renderer::new(device,queue,format,w,h,method)`, `render_frame(&mut self,&Surface,&CaptureFrame,is_bgra)`, `frame_timings()`.
- `WindowManager` trait is story 002 — story 001 does NOT touch it. Its `raw_window_handle() -> Option<&dyn HasWindowHandle>` (borrowed) is incompatible with the owned-`'static` surface model; that reconciliation is 002's job. Story 001 opens the overlay directly via `WebviewWindowBuilder` in `setup`.

## B. Single-event-loop construction

- ONE `tauri::Builder` → `.setup(|app| { build "main" + "overlay" WebviewWindows })` → `.build(generate_context!())` → `app.run(|app_handle, event| { match event {...} })`.
- Overlay window: `WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))` with `.transparent(true).decorations(false).always_on_top(true).skip_taskbar(true).focused(false).inner_size(w,h).position(x,y)`, then `overlay.set_ignore_cursor_events(true)?` (click-through, `webview_window.rs:2121`). Needs a minimal transparent `overlay.html` in the frontend dist (coordinate with `ui/` from story 006; an attached-but-empty webview is acceptable per STORY Open Q — do NOT composite wgpu under a VISIBLE webview, tauri #9220).
- `App::run` closure gets `&AppHandle` (NOT `&App`). Fetch the overlay INSIDE the loop: `app_handle.get_webview_window("overlay") -> Option<WebviewWindow>` returns an **owned, Clone, `'static`** value (`webview_window.rs:1452`, impls `HasWindowHandle+HasDisplayHandle` rwh 0.6 at :1469/:1479). Build the surface from `overlay.clone()` → wgpu 29 yields `Surface<'static>`. Keep the original `overlay` in `OverlayGpu._window` so the target outlives the surface. `tauri::WebviewWindow` aliases `WebviewWindow<Wry>`.
- `RunEvent::MainEventsCleared` EXISTS (`app.rs:256`); `RunEvent` is `#[non_exhaustive]` → add `_ => {}`. Render inside it, gated by `dirty.swap(false, Ordering::Acquire)`. `OverlayGpu` lives as an `Option<_>` captured in the `move` closure, `None` until `RunEvent::Ready` constructs it.
- `surface.get_current_texture()` in wgpu 29 returns the `CurrentSurfaceTexture` enum: handle `Success(t) | Suboptimal(t) => t`, map others to error (pattern at `renderer.rs:137–145`).

## C. Empirical spikes (Phase-2 GATE before Phase 3)

- **T004 cadence:** measure `RunEvent::MainEventsCleared` rate under Xvfb+picom; need ≥30 redraws/1.0s. If sparse (tao #635) → ~60 Hz timer thread that sets `dirty` + wakes the loop (`AppHandle::run_on_main_thread`). RECORD the chosen mechanism in SUBTASKS T004 notes + HLP Discovered Constraints (T014 checklist).
- **T005 surface:** offscreen Mesa-llvmpipe unit (`overlay_gpu_offscreen_render_clear`) de-risks clear/submit logic headlessly; subprocess (`overlay_surface_presents`) proves the live GTK-window surface. Ensure `LIBGL_ALWAYS_SOFTWARE=1` + GL backend (device.rs:21).

## D. RISK-001 fallback (raw wry 0.55 + tao 0.35)

If a FUNDAMENTAL coexistence failure is proven in T004–T006 (surface can NEVER be created from the GTK window, OR no cadence mechanism is stable, OR transparency/click-through impossible) → **autonomously** switch to driving `tao::EventLoop::<LuminosEvent>::with_user_event()` + `wry::WebViewBuilder` directly (exact versions already in the lock; no new majors), reverting `EventNotifier` to the existing `EventLoopProxy<LuminosEvent>` blanket impl. Cost: loses Tauri IPC / tauri-specta / bundler → stories 005/006/007 rework. If the failure is AMBIGUOUS/PARTIAL (flicker, jitter, intermittent click-through = tuning problems) → **escalate to lead** with logs/measurements; do NOT rewrite. (Per the epic decision: implement the documented fallback autonomously, escalate only if the fallback ALSO fails.)

## E. Toolchain gate & verification

Almost everything needs the crate to COMPILE under `--features tauri` (LuminosHandle/AppError reference `tauri::*`), which needs `webkit2gtk-4.1`+`libsoup-3.0`. Pure-logic TEST BODIES (T002 AppError mapping, T003 handle, T007 AppNotifier) run in-process on worker threads; `run()`-driven behavior (T004/5/6/8/9/10/12) needs subprocess-under-Xvfb(+picom) with `xprop`/`xwininfo` window assertions. T007 (`AppNotifier`, pure `Arc<AtomicBool>`) is the most isolated — good first headless task once the crate compiles.

## F. Gotchas

- tao = GTK3: use EWMH via `always_on_top`/`skip_taskbar`; do NOT use winit `with_override_redirect` (that's 002/winit-only). The shipping `luminos-app` binary must instantiate ZERO winit `EventLoop`s (FR-1/AC-1.1) — never call `X11WindowManager::create_overlay` (it builds an ephemeral winit EventLoop, `window.rs:167`).
- Transparency needs a compositor (picom): detect `_NET_WM_CM_S0` owner absence → `log::warn!` NoCompositor + continue opaque, never panic (NFR-3).
- Capability stub: ship only `core:default` for the `main` webview (DC-8/NFR-5). Native window ops (2nd window, ignore-cursor-events, transparency) are NOT capability-gated. Fold any blocked op into this stub — do not defer to 005.
- `tauri.conf.json`: set identifier (e.g. `dev.luminos.app`), `bundle.license = "GPL-3.0-only"`, `frontendDist = "../ui/dist"` (story 006 produces `ui/dist`). `T001` may need a placeholder if `ui/dist` isn't built yet at compile time.
- Enable `default = ["tauri"]` in `luminos-app/Cargo.toml`; add `wgpu`, `raw-window-handle`, and the async-bridge crate under the `tauri` feature.
