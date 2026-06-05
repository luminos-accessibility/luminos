# E04 Tauri Control Panel — Key Findings

Epic E04 (Tauri control panel + settings persistence) spec'd 2026-06. 7 stories.
Specs: `specs/E04-tauri-control-panel/` (HIGH_LEVEL_PLAN.md + 001..007 story folders).
Full RISK-001 research: `.claude/agent-memory/technical-research-analyst/risk001-dual-event-loop-research.md`.

## RISK-001 resolution (single event loop) — VALIDATED
- ONE tao/Tauri loop via `tauri::Builder::…build()?.run(|app, RunEvent| …)`. NO separate `winit::EventLoop` in the shipping binary (macOS NSApplication single-principal-class constraint; winit #3772).
- Overlay = a SECOND Tauri/tao window (`WebviewWindowBuilder`, transparent/decorations:false/alwaysOnTop/skipTaskbar). Keep wgpu and webview in SEPARATE windows (single-window wgpu+webview flicker = tauri #9220, closed not-planned).
- tao uses GTK3 on Linux (winit uses X11/Wayland directly). Override-redirect / direct X11 props are not first-class in GTK.

## Tauri 2.11 API facts confirmed during audit (avoid these mistakes)
- `WebviewWindow` has **NO `request_redraw()`**. `App::run` callback exposes **NO `ControlFlow`/`Poll`** and no `RedrawRequested`. Drive rendering inside `RunEvent::MainEventsCleared`.
- Redraw cadence on tao GTK3 is not guaranteed steady (tao #635) → gate render on a shared `Arc<AtomicBool>` dirty flag and/or a ~60 Hz timer thread that flips it. (Empirically chosen in story-001 spike.)
- Wake mechanism: `EventNotifier::notify_state_changed()` just `store(true)` on the shared `Arc<AtomicBool>` — Send+Sync, no main-thread marshaling, no `run_on_main_thread` needed for redraw.
- `wgpu::Surface<'static>` (wgpu 29.0.3) requires an OWNED `'static` window target. Build from `Instance::create_surface(window.clone())` on an owned Tauri `WebviewWindow` (Arc-backed). A borrowed `&W` yields `Surface<'window>`, NOT `'static` — won't compile if you declare `Surface<'static>`.
- `RunEvent` has no top-level `Resized`; resize arrives as `RunEvent::WindowEvent { event: WindowEvent::Resized(size), .. }`.
- Valid tauri.conf.json window keys: `transparent`, `decorations`, `alwaysOnTop`, `skipTaskbar`. `set_ignore_cursor_events(bool)` = click-through. `macOSPrivateApi` is macOS-only (Linux transparency just needs a compositor). `bundle.license` valid; workspace `license = "GPL-3.0-only"` (authoritative; doc-09's "or-later" is stale).
- Capability perms `core:default`, `core:event:default`, `shell:allow-open` are valid Tauri 2.x identifiers.

## Testing constraint
- `tauri::App::run` never returns + owns the main thread; nextest tests run on worker threads → you CANNOT boot a Tauri app in-process in a `#[test]`. Use a SUBPROCESS harness: spawn the binary under Xvfb+picom, assert via `xprop`/`xwininfo`, structured log heartbeats (`redraw=N`, `shutdown=clean`), and exit code. Pure logic (notifier, offscreen wgpu clear) stays in-process.
- Keep the `tauri` feature gate on `luminos-app` (webkit2gtk-4.1/libsoup-3.0 must NOT become a hard `cargo build` requirement for unrelated crates; E01 known deviation). CI installs webkit2gtk + xprop/xwininfo/xdotool for luminos-app jobs.
- `tauri-driver` (E2E, story 007) is Linux+Windows only (no macOS WKWebView driver).

## Compile-blockers found in audit (real-code gaps to schedule)
- **specta::Type is NOT implemented by ANY engine type.** `#[specta::specta]` commands need it. `FrameTimingSummary` has NEITHER serde NOR specta. Story 005 must add `specta` (pinned) + `#[derive(specta::Type)]` to MagnificationMode, AppSettings + all sub-structs, FrameTimingSummary (+ serde + rename_all="camelCase"). Alternative: local DTOs in luminos-app.
- **No crate-root `pub use` re-exports** in luminos-gpu/luminos-platform. `luminos_gpu::Renderer` does NOT resolve — use `luminos_gpu::renderer::Renderer`, `::shaders::InterpolationMethod`, `::frame_timings::FrameTimingSummary`, `luminos_platform::traits::ScreenCapture`, OR add re-exports. (luminos-core DOES re-export.)
- **Self-capture is ALREADY shipped:** `ScreenCapture::set_excluded_windows(&mut self, &[u64])` on `XcbCapture` (unmap/remap). Don't reinvent — pass overlay XID to it.
- **tauri-specta events need `serde::Deserialize`** (for Event::listen), not just Serialize+Type.
- **AppSettings already derives Default** (schema.rs). Don't add fields to it (breaks struct-literal tests) — use a `ConfigFile { schema_version, settings }` wrapper for on-disk versioning.
- `wgpu::Device` IS `Clone` in 29.0.3. `FrameTimings` (ring buffer) ≠ `FrameTimingSummary` (call `.summary(target_fps)`). `Renderer` bakes `InterpolationMethod` at new() — no runtime switch (Phase 0 fixed-at-startup). `InterpolationMode` (types/settings) vs `InterpolationMethod` (gpu) — map between.
- `tauri::WebviewWindow::gtk_window()` exists (Linux) → gdk → X11 XID. Tray needs libayatana-appindicator. `@crabnebula/tauri-driver` is the package. `@tauri-apps/api >= 2.7.0` for event mocking. `api.prevent_close()` is on the CloseRequested event's `api`, not the window.

## Story map (7)
001 App shell+single loop+wgpu overlay surface (RISK-001 spike) → 002 Overlay WindowManager winit→tao + self-capture (RISK-002) → 003 Live magnification (wire E2/E3 modules) → 004 ConfigManager+persistence → 005 IPC layer+tauri-specta bindings → 006 Frontend UI → 007 Tray+tauri-driver CI E2E.
Deps: 001→{002,004}; 003←002; 005←{001,004}; 006←005; 007←{003,005,006}.

## Deviations to reconcile at Phase-0 gate
- doc-01 §3.3/§6.5, doc-05 §4.1, roadmap §4.4 all still say winit overlay + EventLoopProxy → update to tao/AppNotifier after story 001 validates the spike. Update RISK-001 status.
- E04 absorbs the FIRST construction of the main event loop (no prior epic built it; luminos-app/main.rs was empty) — in-scope per roadmap's dual-window inclusion.
