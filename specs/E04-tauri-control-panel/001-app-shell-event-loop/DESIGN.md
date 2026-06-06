# Design: Story E04/001 -- App Shell, Single Event Loop & wgpu Overlay Surface

**Story:** [STORY.md](./STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** APPROVED (approved-as-authoritative per E04 execution, 2026-06-04)
**Author:** principal-architect
**Risk Refs:** RISK-001 (dual event loop coexistence — this story retires it), RISK-002 (self-capture — characterized here, mitigated in 002), RISK-016 (wgpu backend compat), RISK-030 (wgpu/winit/Tauri upgrade cascade)

---

## Overview

This story replaces the empty `luminos-app` binary with a Tauri 2.x application that runs a **single** tao/Tauri event loop and hosts two windows: the control-panel webview and a native, transparent, click-through **overlay**. It builds a `wgpu::Surface` from the overlay window's `raw-window-handle` and presents a clear-color frame on a validated redraw cadence. It introduces `LuminosHandle` (real `Arc<ArcSwap<AppState>>`) as Tauri managed state and a tao-backed `EventNotifier`.

The approach is mandated by the RISK-001 research (HIGH_LEVEL_PLAN AD-1): a second `winit::EventLoop` is impossible alongside Tauri on macOS (single `NSApplication` principal class), so we run one tao loop and treat the overlay as a second tao window. We deliberately keep the GPU and the webview in **separate windows** to avoid the documented single-window wgpu/webview flicker (tauri #9220). The first phase is a spike that empirically validates the three uncertain mechanics on tao's GTK3 backend: redraw cadence, wgpu-surface-from-overlay, and transparency/click-through.

## Architecture

### Component Diagram

```
                          luminos-app process (single tao/Tauri event loop, main thread)
  ┌───────────────────────────────────────────────────────────────────────────────────┐
  │  tauri::Builder                                                                      │
  │    .manage(LuminosHandle{ app_state: Arc<ArcSwap<AppState>>, config, notifier, app })│
  │    .setup(|app| {                                                                    │
  │        control_panel = WebviewWindowBuilder("main", ...placeholder page...)          │
  │        overlay       = WebviewWindowBuilder("overlay", transparent/undecorated/      │
  │                          always_on_top/skip_taskbar).build()?                         │
  │        overlay.set_ignore_cursor_events(true)   // click-through                      │
  │     })                                                                                │
  │    .build(generate_context!())?                                                       │
  │    .run(|app_handle, RunEvent| match event {                                          │
  │        Ready            => init Renderer-surface from overlay rwh (OverlayGpu)         │
  │        <redraw cadence> => OverlayGpu.render_clear()   // reads ArcSwap (lock-free)    │
  │        ExitRequested    => graceful shutdown                                          │
  │     })                                                                                 │
  │                                                                                       │
  │   ┌─────────────┐   sets dirty flag (Arc<AtomicBool>)  ┌───────────────────────────┐  │
  │   │ AppNotifier │ ────────────────────────────────────▶│ MainEventsCleared reads &  │  │
  │   │(EventNotifier)│  no request_redraw (Tauri lacks it)│ clears flag → renders      │  │
  │   └─────▲───────┘   Send+Sync, no main-thread marshal  │  overlay wgpu::Surface     │  │
  │         │ notify_state_changed()                       └───────────────────────────┘  │
  │   (held by future input/IPC threads — stories 003/005)                                │
  └───────────────────────────────────────────────────────────────────────────────────┘
        reads ▲ lock-free                              │ writes (stories 003/005)
              └────────── Arc<ArcSwap<AppState>> ──────┘   (luminos-core)
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `luminos-app/src/main.rs` | Rewrite | Empty stub → Tauri app with single tao loop, two windows, run-loop match. |
| `luminos-app/src/handle.rs` | New | `LuminosHandle` managed-state struct. |
| `luminos-app/src/notifier.rs` | New | `AppNotifier` implementing `luminos_core::pipeline::EventNotifier` over Tauri `AppHandle`. |
| `luminos-app/src/overlay_gpu.rs` | New | `OverlayGpu` — owns wgpu `Instance`/`Surface`/`Device`/`Queue` bound to the overlay window; `render_clear()`, `resize()`. (Story 003 swaps the clear for the real `Renderer`.) |
| `luminos-app/src/app_error.rs` | New | `AppError` top-level error enum (`thiserror`). |
| `luminos-app/Cargo.toml` | Modified | **Keep the `tauri` feature gate** (webkit2gtk-4.1/libsoup-3.0 must not become a hard requirement for `cargo build` of unrelated crates — see E01 known deviation). Set `default = ["tauri"]` for this crate, and ensure CI installs webkit2gtk before building/linting `luminos-app`; the existing `--exclude luminos-app` convention in CLAUDE.md stays for environments lacking the libs. Add `wgpu`, `raw-window-handle` (and `arc-swap`, present) under the `tauri` feature. |
| `luminos-core/src/config/{mod,manager}.rs` | New (stub) | Minimal empty `ConfigManager` struct + `Default` so `LuminosHandle` compiles; real I/O is story 004. |
| `luminos-app/build.rs` | New | `tauri_build::build()`. |
| `luminos-app/tauri.conf.json` | New | App identifier, two-window config, `frontendDist` placeholder, bundle metadata (GPL-3.0-only). |
| `luminos-core` | Reused (no change) | `AppState`, `StateManager`, `LuminosEvent`, `EventNotifier` consumed as-is. |
| `luminos-gpu` | Reused (no change) | wgpu device/surface helpers reused; clear-frame logic local to `OverlayGpu` for this story. |

> `EventNotifier` already has a blanket impl for `winit::EventLoopProxy<LuminosEvent>` in `luminos-core::pipeline`. We add a **second** impl (`AppNotifier`) in `luminos-app`; the winit impl stays for unit tests and is untouched. Doc comments in `event.rs`/`pipeline.rs` that say "winit event loop" should be updated in story 003 when the input pipeline is wired (logged as a docs follow-up; not edited here to keep 001 contained).

### Data Flow (primary scenario: launch → first frame)

1. `main()` builds `LuminosHandle` with `app_state = Arc::new(ArcSwap::from_pointee(AppState::default()))` (settings seeded from disk in story 004; default here), a placeholder `config`, and the `AppHandle` (filled in `setup`).
2. `tauri::Builder::default().manage(handle).setup(...)` opens the control-panel window (`"main"`, placeholder page) and the overlay window (`"overlay"`, transparent/undecorated/always-on-top/skip-taskbar), then `overlay.set_ignore_cursor_events(true)`.
3. `.build(generate_context!())?` returns the `App`; `.run(closure)` enters the single loop.
4. On `RunEvent::Ready`, `OverlayGpu::new(&overlay_window)` creates `wgpu::Instance` → `create_surface(&overlay_window)` (valid because Tauri `WebviewWindow: HasWindowHandle + HasDisplayHandle`, rwh 0.6) → request adapter/device/queue → configure surface (`Bgra8UnormSrgb`, `AlphaMode` honoring transparency).
5. Each `RunEvent::MainEventsCleared`, the run callback does `dirty.swap(false, Acquire)` (and/or the steady-cadence path) and calls `overlay_gpu.render_clear()` — reads `app_state.load()` (lock-free), acquires the next swapchain texture, clears to a transparent color, submits, presents. A `redraw=N` heartbeat is logged for the cadence test.
6. A background thread (test harness in this story; real input/IPC threads in 003/005) calls `notifier.notify_state_changed()`, which sets the shared `Arc<AtomicBool>` dirty flag — no main-thread marshaling, since the flag is `Send + Sync`. The next `MainEventsCleared` observes it and renders.
7. Resize arrives as `RunEvent::WindowEvent { label: "overlay", event: WindowEvent::Resized(size) }` → `OverlayGpu::resize`. On `RunEvent::ExitRequested`/`WindowEvent::CloseRequested`, the callback flips a shutdown flag, joins any spawned threads, drops `OverlayGpu`, and allows the loop to exit.

## API Design

```rust
// luminos-app/src/handle.rs
use std::sync::{Arc, Mutex};
use arc_swap::ArcSwap;
use luminos_core::AppState;

/// Tauri managed state shared with every command (stories 005+) and the run loop.
pub(crate) struct LuminosHandle {
    /// Live application state. The render path reads this lock-free every redraw.
    pub app_state: Arc<ArcSwap<AppState>>,
    /// Persistence layer. Story 001 lands only a minimal `ConfigManager` stub
    /// (empty struct + Default) so this compiles; `None` until story 004 wires real I/O.
    pub config: Arc<Mutex<Option<luminos_core::config::ConfigManager>>>,
    /// Wake mechanism handed to background threads (stories 003/005).
    pub notifier: AppNotifier,
    /// Tauri app handle for emitting events (story 005) and main-thread marshaling.
    pub app: tauri::AppHandle,
}

// luminos-app/src/notifier.rs
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use luminos_core::pipeline::EventNotifier;

/// tao/Tauri-backed EventNotifier. Tauri's `WebviewWindow` has NO `request_redraw()`
/// and `App::run` exposes no control-flow, so we do NOT marshal a redraw. Instead
/// `notify_state_changed()` sets a shared `Arc<AtomicBool>` "render-dirty" flag that
/// the `App::run` callback reads-and-clears each `MainEventsCleared`. The flag is
/// `Send + Sync`, so worker threads set it with no main-thread affinity.
#[derive(Clone)]
pub(crate) struct AppNotifier {
    dirty: Arc<AtomicBool>,
}

impl AppNotifier {
    pub fn new(dirty: Arc<AtomicBool>) -> Self { Self { dirty } }
    /// Shared flag handed to the run loop; loop does `swap(false, Acquire)`.
    pub fn dirty_flag(&self) -> Arc<AtomicBool> { self.dirty.clone() }
}

impl EventNotifier for AppNotifier {
    fn notify_state_changed(&self) {
        self.dirty.store(true, Ordering::Release);
    }
}

// luminos-app/src/overlay_gpu.rs
/// Owns the wgpu objects bound to the overlay window. Story 003 replaces
/// `render_clear` with `luminos_gpu::Renderer`-driven frames against this surface.
///
/// `Surface<'static>` requires the surface target to outlive the surface, so we
/// store an OWNED `WebviewWindow` (Arc-backed, `'static`, impls HasWindowHandle +
/// HasDisplayHandle) next to it — NOT a borrowed `&W` (which yields `Surface<'window>`).
pub(crate) struct OverlayGpu {
    surface: wgpu::Surface<'static>,
    _window: tauri::WebviewWindow, // keeps the surface target alive
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl OverlayGpu {
    /// Create instance/surface/device/queue from an OWNED overlay window.
    /// `Instance::create_surface(window.clone())` over an owned `'static` target
    /// yields `Surface<'static>`.
    pub fn new(window: tauri::WebviewWindow, width: u32, height: u32) -> Result<Self, AppError>;

    /// Present one transparent clear frame. Reads nothing yet beyond size.
    pub fn render_clear(&mut self) -> Result<(), AppError>;

    /// Reconfigure the surface after a resize.
    pub fn resize(&mut self, width: u32, height: u32);
}

// luminos-app/src/main.rs
fn main() -> Result<(), AppError>; // builds, runs the single tao/Tauri loop
```

**Redraw cadence (FR-5, AC-2.3) — Tauri's `run` API, not winit's.** Tauri exposes the loop via `App::run(|app_handle, RunEvent| …)`, which **never returns**, owns the main thread, and exposes **no `ControlFlow`/`Poll`** and **no `RedrawRequested`**; `WebviewWindow` has **no `request_redraw()`**. So the loop is observed only through `RunEvent` variants. We render inside `RunEvent::MainEventsCleared`, gated on the shared `Arc<AtomicBool>` dirty flag (`swap(false, Acquire)` → render if it was set, plus an always-render path for the steady cadence). Because tao's GTK3 backend may not emit `MainEventsCleared` at a steady ~60 Hz (tao #635), the **spike (Phase 2) measures the actual cadence and selects** between (a) rendering on every `MainEventsCleared`, or (b) a ~60 Hz timer thread that flips the dirty flag to force redraws. The chosen mechanism is recorded in SUBTASKS completion notes and the epic Shared Context. If neither yields a stable cadence under Tauri's abstraction, escalate to the raw-wry+tao fallback (Alternatives #3), which *does* expose tao's `ControlFlow`/user-events.

## Error Handling

`AppError` (in `luminos-app/src/app_error.rs`, `thiserror`) aggregates startup/runtime failures; `main` returns `Result<(), AppError>` so failures print a typed message and exit non-zero rather than panicking.

```rust
#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
    #[error("Tauri runtime error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("GPU init failed: {0}")]
    Gpu(String),                       // wraps wgpu CreateSurfaceError/RequestDeviceError (no From — they vary by wgpu)
    #[error("overlay window '{0}' not found")]
    OverlayMissing(String),
    #[error("no compositor detected; transparency unavailable")]
    NoCompositor,                      // non-fatal: logged as warn, app continues opaque
}
```

Conventions (CLAUDE.md): `?` propagation with `From` for `tauri::Error`; wgpu errors are mapped via `.map_err(|e| AppError::Gpu(e.to_string()))?` because wgpu error types differ per call. No `unwrap`/`expect` in production paths; `NoCompositor` is a `warn!` + continue, not a hard error. All dynamic values single-quoted in logs.

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | Tauri/tao GTK3 windows; transparency needs compositor (picom); click-through via `set_ignore_cursor_events`; surface from rwh. | **The only platform implemented this story.** Override-redirect is a story-002 concern (via WindowManager). |
| Linux Wayland | Deferred (E8). tao supports Wayland; transparency/always-on-top differ. | Not built here. |
| macOS | Single `NSApplication` loop makes this architecture mandatory. Overlay-above-fullscreen needs `NSPanel` + `macOSPrivateApi` + `ActivationPolicy::Accessory`. | Documented as a follow-up (E12); not implemented. |
| OpenBSD | Deferred (E15). | — |
| Windows | Most permissive; same single-loop design kept for portability; `WS_EX_NOACTIVATE` to avoid focus theft; WebView2 bootstrapper at packaging. | Deferred (E17/E18). |

## Testing Strategy

**Harness constraint (important).** `tauri::App::run` never returns and must own the main thread; `cargo nextest` runs `#[test]`s on worker threads. Therefore **`run()`-driven behavior cannot be asserted in-process.** Two harnesses are used:

- **In-process seam/unit tests** (worker thread, no `run()`): pure logic that doesn't need the loop.
- **Subprocess tests**: spawn the real `luminos-app` binary under `Xvfb`+`picom`, drive it externally (`xdotool`, `SIGTERM`), and assert via window inspection (`xprop`/`xwininfo`), structured stdout/log lines (`redraw=N` heartbeat; `shutdown=clean`), and the process exit code. These gracefully skip if `xprop`/`xdotool` are unavailable (mirroring the E03 platform-test pattern), but CI MUST have them.

### Unit / In-process seam tests
- `app_notifier_sets_dirty_flag` — `notify_state_changed()` flips the shared `Arc<AtomicBool>` to `true` (pure, no runtime).
- `overlay_gpu_offscreen_render_clear` — the clear-frame render logic against a **headless wgpu device + offscreen `TextureView`** (Mesa llvmpipe, `ci_platform_tests`) — exercises device/queue/clear/submit without needing a window surface.
- `app_error_from_tauri_error_maps` — `AppError: From<tauri::Error>` compiles and maps.

### Subprocess integration tests (Linux, `ci_platform_tests`, under Xvfb+picom)
- `app_boots_two_windows_and_exits_clean` — spawn binary; assert two windows via `xwininfo`/`xprop`, single process, then `SIGTERM`/close → exit code 0 within a timeout, `shutdown=clean` logged (AC-1.1).
- `overlay_surface_presents` — binary logs `surface_ok` + `frame_presented` from the real overlay surface path under Mesa llvmpipe (AC-2.1, live-window half; the offscreen unit test covers the render logic).
- `overlay_attributes` — `xprop` shows `_NET_WM_STATE_ABOVE`, undecorated, skip-taskbar; click-through asserted via the `ignore_cursor_events` log + a pointer-event probe (AC-2.2).
- `redraw_cadence` — parse `redraw=N` heartbeats over a fixed wall-clock window; assert N ≥ threshold (AC-2.3).
- `notify_triggers_redraw` — the binary exposes a debug trigger (env-gated) that sets the dirty flag from a spawned thread; assert the heartbeat rate increases / a `dirty_render` line appears (AC-3.1 wake half).
- `managed_state_probe` — an env-gated debug command path logs the `app_state` pointer/zoom to prove `State<LuminosHandle>` retrieval (AC-3.1 state half).

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|---------------------|
| AC-1.1 (lifecycle) | Subprocess | Spawn binary under Xvfb+picom; assert two windows + single process; `SIGTERM`/close → exit 0, `shutdown=clean`, no hang (timeout-guarded). |
| AC-2.1 (surface + frame) | Unit (offscreen, Mesa) + Subprocess | Offscreen device clear-render unit test for logic; subprocess `surface_ok`/`frame_presented` for the live overlay surface. |
| AC-2.2 (attributes) | Subprocess | `xprop`/`xwininfo` flags + `ignore_cursor_events` log + pointer-passthrough probe. |
| AC-2.3 (cadence) | Subprocess | `redraw=N` heartbeat advances ≥ threshold over fixed window on GTK3. |
| AC-3.1 (state + wake) | Unit + Subprocess | Unit: `notify_state_changed` sets flag. Subprocess: managed-state probe logs handle; debug wake trigger increases redraw rate. |

## Performance Targets

- Startup → first overlay frame < 2 s (NFR-1).
- Clear-frame render trivially < 8 ms; the real render-budget assertion lands in story 003 (NFR-2).

## Security Considerations

- Native Rust window ops (second-window create, `set_ignore_cursor_events`, transparency) are **not** gated by webview capabilities, so the spike needs no command permissions. This story still lands a **minimal capability stub** (`core:default` for the control-panel webview, so it loads); story 005 **extends** that same file to the full set (`core:default`, `core:event:default`, `shell:allow-open`). See HLP **DC-8**. If the spike finds any window op blocked, fold the needed permission into this stub rather than deferring to 005.
- `macOSPrivateApi`/private APIs not enabled on Linux; deferred to macOS epic.
- Overlay loads no remote content; default CSP retained (doc-06 §3.4).

## Alternatives Considered

1. **Second `winit::EventLoop` alongside Tauri (research option b).** Rejected: impossible on macOS (winit #3772, single `NSApplication` principal class). This is the decisive constraint behind AD-1.
2. **wgpu + webview composited in one window (tauri #9220).** Rejected: documented surface-contention flicker; closed not-planned upstream. The two-window split avoids it.
3. **Raw `wry` + `tao`, bypassing the Tauri app abstraction (research option d).** Rejected as the primary path — loses Tauri IPC, `tauri-specta`, plugins, packaging. **Retained as the fallback** if the Tauri two-window spike fails (escalation path noted in STORY Open Questions).
4. **Keep winit for the overlay, drive Tauri separately.** Rejected: degenerates into the two-event-loop problem; not portable.
