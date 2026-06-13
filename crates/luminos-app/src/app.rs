//! Single tao/Tauri event loop hosting the control panel and the GPU overlay.
//!
//! This is the RISK-001 linchpin: ONE event loop (`tauri::App::run`) drives both
//! the control-panel webview and a transparent, click-through magnification
//! overlay whose wgpu surface is built from the overlay window's owned handle.
//!
//! Redraw cadence (FR-5/AC-2.3): Tauri's `run` exposes no `ControlFlow`/`Poll`
//! and `WebviewWindow` has no `request_redraw()`. tao's GTK3 backend does not
//! emit `RunEvent::MainEventsCleared` at a steady rate on its own (tao #635),
//! and a bare `run_on_main_thread(|| {})` does not reliably provoke one. The
//! spike's chosen, empirically-stable mechanism: a ~60 Hz timer thread marshals
//! the *heartbeat itself* onto the main thread via
//! `AppHandle::run_on_main_thread` (which runs the closure reliably), where it
//! sets the shared `Arc<AtomicBool>` dirty flag and advances the `redraw=N`
//! counter. The opportunistic GPU present then happens in whatever
//! `MainEventsCleared` that main-thread task provokes, gated on the dirty flag.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use arc_swap::ArcSwap;
use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};

use luminos_core::{AppState, EventNotifier};

use crate::app_error::AppError;
use crate::compositor::compositor_running;
use crate::handle::LuminosHandle;
use crate::notifier::AppNotifier;
use crate::overlay_gpu::OverlayGpu;

/// Window label of the overlay (the control panel is `control-panel`, declared
/// in `tauri.conf.json`).
const OVERLAY_LABEL: &str = "overlay";

/// Target redraw period for the cadence timer (~60 Hz).
const CADENCE_PERIOD: Duration = Duration::from_micros(16_666);

/// Fallback overlay size used only when no monitor can be queried.
const FALLBACK_SIZE: (u32, u32) = (1920, 1080);

/// Builds the Tauri app, opens both windows, and runs the single event loop.
///
/// Never returns on success until the loop exits (then returns `Ok(())`).
///
/// # Errors
///
/// Returns [`AppError`] if the Tauri context fails to build or a window cannot
/// be created during `setup`.
pub fn run() -> Result<(), AppError> {
    // Install an async-signal-safe SIGTERM/SIGINT handler. It only sets an
    // atomic; the cadence thread polls it and asks the loop to exit cleanly.
    // (We deliberately do NOT block these signals: that would break GTK window
    // realization, which relies on GLib's own signal handling.)
    crate::signal::install_termination_handler();

    // Shared dirty flag: the notifier sets it, the run loop drains it.
    let dirty = Arc::new(AtomicBool::new(true));
    let notifier = AppNotifier::new(Arc::clone(&dirty));

    // Seed the real application state from disk (correction #1: ConfigManager
    // is real). On failure, warn and fall back to defaults.
    let (mut state, config) = match luminos_core::seed_initial_state() {
        Ok((state, manager)) => (state, Some(manager)),
        Err(e) => {
            log::warn!("config seed failed: '{e}'; using defaults");
            (AppState::default(), None)
        }
    };

    // Test-only hook: start with magnification active so subprocess tests can
    // exercise the live magnify path without first toggling via input (the
    // input pipeline is covered separately). Gated by an env var so it never
    // affects production, mirroring `LUMINOS_SELF_CAPTURE_PROBE`/`LUMINOS_DEBUG_NOTIFY`.
    if std::env::var("LUMINOS_FORCE_ACTIVE").as_deref() == Ok("1") {
        state.is_active = true;
        log::info!("LUMINOS_FORCE_ACTIVE=1: seeding is_active=true for the live magnify path");
    }

    // Test-only hook (story 007): force `minimize_to_tray` so the minimize-to-tray
    // subprocess test is deterministic regardless of the seeded config on the
    // host (the default is environment-dependent once a real config.toml exists).
    // Gated by an env var so it never affects production, mirroring
    // `LUMINOS_FORCE_ACTIVE`.
    match std::env::var("LUMINOS_FORCE_MINIMIZE_TO_TRAY").as_deref() {
        Ok("1") => {
            state.settings.minimize_to_tray = true;
            log::info!("LUMINOS_FORCE_MINIMIZE_TO_TRAY=1: seeding minimize_to_tray=true");
        }
        Ok("0") => {
            state.settings.minimize_to_tray = false;
            log::info!("LUMINOS_FORCE_MINIMIZE_TO_TRAY=0: seeding minimize_to_tray=false");
        }
        _ => {}
    }

    let app_state = Arc::new(ArcSwap::from_pointee(state));

    let builder_state = Arc::clone(&app_state);
    let builder_notifier = notifier.clone();

    // The tauri-specta IPC handler owns the command + event surface (story 005).
    // In debug builds it also regenerates `ui/src/ipc/bindings.ts`; the CI
    // bindings-up-to-date check diffs the committed file against a fresh export.
    let ipc = crate::ipc::build_ipc_handler();
    #[cfg(debug_assertions)]
    if let Err(e) = crate::ipc::export_bindings(&ipc) {
        // Non-fatal: a failed export must not block app startup (e.g. read-only
        // checkout). The committed bindings remain authoritative at runtime.
        log::warn!("bindings export skipped: '{e}'");
    }

    let app = tauri::Builder::default()
        .invoke_handler(ipc.invoke_handler())
        .setup(move |app| {
            // Register the tauri-specta events so `Event::emit(app)` reaches the
            // webview's listeners (story 005 panel-sync channel, AD-5).
            ipc.mount_events(app);

            // Register managed state holding the real ArcSwap (FR-6).
            let handle = LuminosHandle::new(
                Arc::clone(&builder_state),
                config,
                builder_notifier.clone(),
                app.handle().clone(),
            );
            app.manage(handle);

            // AC-3.1 (state half): prove managed state is retrievable from a
            // Tauri command context and reads the live `AppState`.
            probe_managed_state(app.handle());

            setup_overlay_window(app)?;

            // Story 007 (D6): create the system tray. Linux-only this epic. The
            // returned icon is STASHED on the handle so it outlives `setup`
            // (dropping a `TrayIcon` removes it). On no SNI host it degrades to
            // `None` (warn logged inside `init_tray`); never aborts startup.
            #[cfg(target_os = "linux")]
            init_tray_into_handle(app);

            Ok(())
        })
        .on_window_event(handle_close_to_tray)
        .build(tauri::generate_context!())
        .map_err(AppError::Tauri)?;

    let exit_code = run_event_loop(app, dirty, notifier, app_state);
    if exit_code != 0 {
        log::warn!("event loop exited with non-zero status '{exit_code}'");
    }
    Ok(())
}

/// Retrieves the managed [`LuminosHandle`] the way a Tauri command would and
/// logs proof that it is reachable and reads the live `AppState` (AC-3.1).
fn probe_managed_state(app: &tauri::AppHandle) {
    let Some(handle) = app.try_state::<LuminosHandle>() else {
        log::error!("managed_state probe failed: LuminosHandle not registered");
        return;
    };
    let zoom = handle.app_state.load().settings.magnification.zoom_level;
    // Handle a poisoned config mutex explicitly rather than masking it as
    // `false`: a poisoned lock means a holder panicked, which is worth a log.
    let config_present = match handle.config.lock() {
        Ok(guard) => guard.is_some(),
        Err(poisoned) => {
            log::error!("config mutex poisoned during managed_state probe; treating as absent");
            poisoned.into_inner().is_some()
        }
    };
    log::info!("managed_state_ok: zoom='{zoom}' config_present='{config_present}'");
}

/// Opens the transparent, click-through overlay window sized to the primary
/// monitor (FR-2, FR-3, AC-2.2).
fn setup_overlay_window(app: &tauri::App) -> Result<(), AppError> {
    if !compositor_running() {
        // NFR-3: warn and continue opaque rather than panic. The typed
        // `AppError::NoCompositor` is the single source of the message; it is
        // logged (not returned) because absence of a compositor is non-fatal.
        log::warn!(
            "NoCompositor: {} -- rendering opaque",
            AppError::NoCompositor
        );
    }

    let (width, height, x, y) = primary_monitor_bounds(app);

    let overlay =
        WebviewWindowBuilder::new(app, OVERLAY_LABEL, WebviewUrl::App("overlay.html".into()))
            .title("Luminos Overlay")
            .transparent(true)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .focused(false)
            .inner_size(f64::from(width), f64::from(height))
            .position(f64::from(x), f64::from(y))
            .build()
            .map_err(AppError::Tauri)?;

    overlay
        .set_ignore_cursor_events(true)
        .map_err(AppError::Tauri)?;
    log::info!(
        "overlay window '{OVERLAY_LABEL}' opened: '{width}x{height}' at '{x},{y}' \
         ignore_cursor_events=true"
    );
    Ok(())
}

/// Creates the system tray (story 007, D6) and stashes the live icon on the
/// managed [`LuminosHandle`] so it outlives `setup`.
///
/// Non-fatal end to end (FR-3): `init_tray` returns `Ok(None)` where no SNI
/// host exists, and any unexpected error is logged and swallowed here rather
/// than aborting startup — the control panel stays visible regardless.
#[cfg(target_os = "linux")]
fn init_tray_into_handle(app: &tauri::App) {
    use tauri::Manager as _;

    let tray = match crate::tray::init_tray(app) {
        Ok(tray) => tray,
        Err(e) => {
            // Per FR-3 the tray must never abort startup; an error here is the
            // belt-and-braces guard for a future hard-failure mode.
            log::warn!("tray init error: '{e}'; keeping control panel visible");
            None
        }
    };

    let present = tray.is_some();
    if let Some(handle) = app.try_state::<LuminosHandle>() {
        handle.set_tray(tray);
        log::info!("tray_stashed={present}");
    } else {
        log::error!("tray init: LuminosHandle not registered; cannot stash tray icon");
    }
}

/// `.on_window_event` handler implementing minimize-to-tray (story 007, FR-2).
///
/// Intercepts `CloseRequested` on the `control-panel` window ONLY: when
/// `minimize_to_tray` is enabled it prevents the close and hides the window
/// (the app keeps running, restorable from the tray menu). The overlay is NEVER
/// hidden here (hiding it kills magnification). The setting is read lock-free
/// from the `ArcSwap` `AppState`; a missing handle defaults to NOT intercepting
/// (the window closes normally).
fn handle_close_to_tray(window: &tauri::Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    if window.label() != "control-panel" {
        return;
    }

    let minimize = window
        .try_state::<LuminosHandle>()
        .is_some_and(|h| h.app_state.load().settings.minimize_to_tray);
    if !minimize {
        // Closing the control panel must quit the whole app. The overlay is a
        // second tao window that keeps the single event loop alive, so letting
        // the control-panel window close on its own leaves a headless process
        // running (the user has to Ctrl+C). Trigger a clean shutdown: `exit`
        // fires `ExitRequested`/`Exit`, which runs the teardown in the run loop.
        log::info!("control-panel close: minimize_to_tray=false; exiting app");
        window.app_handle().exit(0);
        return;
    }

    api.prevent_close();
    match window.get_webview_window("control-panel") {
        Some(panel) => {
            if let Err(e) = panel.hide() {
                log::warn!("minimize_to_tray: hide failed: '{e}'");
            } else {
                log::info!("minimize_to_tray=hidden control-panel");
            }
        }
        None => log::warn!("minimize_to_tray: control-panel window not retrievable to hide"),
    }
}

/// Resolves the primary monitor's bounds, falling back to a default size.
fn primary_monitor_bounds(app: &tauri::App) -> (u32, u32, i32, i32) {
    match app.primary_monitor() {
        Ok(Some(monitor)) => {
            let size = monitor.size();
            let pos = monitor.position();
            (size.width, size.height, pos.x, pos.y)
        }
        Ok(None) => {
            log::warn!("no primary monitor reported; using fallback overlay size");
            (FALLBACK_SIZE.0, FALLBACK_SIZE.1, 0, 0)
        }
        Err(e) => {
            log::warn!("primary monitor query failed: '{e}'; using fallback size");
            (FALLBACK_SIZE.0, FALLBACK_SIZE.1, 0, 0)
        }
    }
}

/// Drives the single event loop. Returns the process exit code.
///
/// FR-1 INVARIANT (RISK-001): this `tauri::App::run` is the ONE AND ONLY event
/// loop in the shipping binary. Do NOT instantiate a `winit::EventLoop` here or
/// anywhere reachable from `main` — in particular never call the
/// `luminos_platform` overlay-creation paths (e.g. `X11WindowManager::create_overlay`,
/// which spins up an ephemeral winit `EventLoop`). A second event loop is
/// impossible alongside Tauri on macOS and silently breaks the cross-platform
/// design. The overlay is a second tao/Tauri window (see `setup_overlay_window`).
fn run_event_loop(
    app: tauri::App,
    dirty: Arc<AtomicBool>,
    notifier: AppNotifier,
    app_state: Arc<ArcSwap<AppState>>,
) -> i32 {
    let mut overlay_gpu: Option<OverlayGpu> = None;
    // Capture driver owns the X11 capture backend + the per-frame TrackingEngine
    // (story-003 §1: tracking lives in the render loop, not the input pipeline).
    #[cfg(target_os = "linux")]
    let mut capture_driver: Option<crate::capture_driver::CaptureDriver> = None;
    let redraw_count = Arc::new(AtomicU64::new(0));
    let shutdown = Arc::new(AtomicBool::new(false));

    // ~60 Hz cadence timer. tao's GTK3 backend does not emit `MainEventsCleared`
    // at a steady rate on its own (tao #635), and `run_on_main_thread(|| {})`
    // alone does not reliably provoke one. So the timer marshals the *heartbeat*
    // itself onto the main thread (which `run_on_main_thread` runs reliably),
    // flagging `dirty`; the GPU present is then performed opportunistically in
    // whatever `MainEventsCleared` the marshaled task provokes. The heartbeat
    // (`redraw=N`) therefore reflects the real, steady main-thread cadence.
    let cadence_handle = spawn_cadence_timer(
        app.handle().clone(),
        Arc::clone(&dirty),
        Arc::clone(&shutdown),
        Arc::clone(&redraw_count),
    );

    // Optional debug wake thread (T012): after an idle delay, set the flag via
    // the notifier to prove the wake path end-to-end.
    let debug_handle = maybe_spawn_debug_notifier(notifier.clone(), Arc::clone(&shutdown));

    let cadence_handle = std::sync::Mutex::new(Some(cadence_handle));
    let debug_handle = std::sync::Mutex::new(debug_handle);
    // Input pipeline slot (story 003 / T006). Linux-only; the X11 input monitor
    // + processor thread are stored so they outlive `Ready` and can be retired
    // on shutdown.
    #[cfg(target_os = "linux")]
    let input_pipeline = std::sync::Mutex::new(None);

    // De-dup key for the state-observation probe (`LUMINOS_LOG_STATE=1`): the
    // last `(mouse_x, mouse_y, zoom_bits, is_active)` logged, so it logs only
    // on change rather than every 60 Hz tick.
    let mut state_log_last: Option<(i32, i32, u32, bool)> = None;

    // Last `(zoom_bits, mode)` emitted to the webview (story 005, FR-6). The
    // loop emits `ZoomChangedEvent`/`ModeChangedEvent` only on a delta so the
    // panel's Zustand store stays in sync after an engine-origin (hotkey)
    // change. It is origin-agnostic: a UI-origin command also moves through the
    // SAME ArcSwap, so the panel may receive a redundant echo of a value it just
    // set — idempotent and harmless (AD-5 deviation; see SUBTASKS). `None` until
    // the first observation so the very first frame does not spuriously emit.
    let mut last_emit: Option<(u32, luminos_types::MagnificationMode)> = None;

    app.run(move |app_handle, event| match event {
        RunEvent::Ready => {
            log::info!("event loop ready; initializing overlay GPU surface");
            // Cannot propagate from the run closure; log the typed error and
            // continue (the overlay simply won't present).
            overlay_gpu = match init_overlay_gpu(app_handle) {
                Ok(gpu) => Some(gpu),
                Err(e) => {
                    log::error!("overlay GPU init failed: '{e}'");
                    None
                }
            };
            // Story 002: bind the platform WindowManager to the overlay's X11
            // window id (same realized overlay window, no re-open). Errors are
            // logged, not fatal (the overlay still renders).
            #[cfg(target_os = "linux")]
            {
                init_window_manager(app_handle);
                // Story 003: build the capture driver (with self-capture
                // exclusion) AFTER the manager binds the overlay XID, then spawn
                // the input pipeline so cursor/hotkeys drive state.
                capture_driver = init_capture_driver(app_handle);
                if let Some(pipeline) = init_input_pipeline(app_handle, &notifier)
                    && let Ok(mut slot) = input_pipeline.lock()
                {
                    *slot = Some(pipeline);
                }
            }
        }
        RunEvent::MainEventsCleared => {
            if dirty.swap(false, Ordering::Acquire) {
                // State-observation probe (test-only, `LUMINOS_LOG_STATE=1`):
                // logs the live AppState each tick so subprocess tests can
                // assert that the input pipeline's cursor/hotkey writes landed,
                // even where no surface-compatible GPU adapter exists under
                // headless Xvfb (DC-10) so the magnify path cannot run.
                log_state_if_enabled(&app_state, &mut state_log_last);

                // Story 005 (FR-6): emit zoom/mode events to the webview on a
                // delta so the panel stays in sync after a hotkey change. The
                // input thread has no AppHandle, so emission lives here in the
                // render loop (AD-5 deviation; see SUBTASKS).
                emit_state_events(app_handle, &app_state, &mut last_emit);

                #[cfg(target_os = "linux")]
                present_if_ready(overlay_gpu.as_mut(), capture_driver.as_mut(), &app_state);
                #[cfg(not(target_os = "linux"))]
                present_if_ready(overlay_gpu.as_mut(), &app_state);

                // Publish the latest frame-timing summary to the handle so the
                // story-005 `get_frame_timings` command reads live data (FR-6).
                if let Some(gpu) = overlay_gpu.as_ref()
                    && let Some(handle) = app_handle.try_state::<LuminosHandle>()
                {
                    handle.set_frame_timings(gpu.frame_timing_summary());
                }
            }
        }
        RunEvent::WindowEvent { label, event, .. } => {
            handle_window_event(&label, &event, overlay_gpu.as_mut(), &dirty);
        }
        // `ExitRequested` then `Exit` both fire; the guard's `compare_exchange`
        // runs teardown exactly once (a later fire fails the swap, so the arm
        // is skipped and falls through to the `_` no-op).
        RunEvent::ExitRequested { .. } | RunEvent::Exit
            if shutdown
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok() =>
        {
            log::info!("shutdown=requested; joining background threads");
            join_thread_slot(&cadence_handle);
            join_thread_slot(&debug_handle);
            // Retire the input pipeline. We DROP (detach) rather than join:
            // the X11 XI2 monitor thread owns the channel Sender and only
            // releases it on a connection error or on the next event after
            // its Receiver closes — and the Receiver is owned by the
            // processor thread, which only exits once the Sender drops. That
            // circular ownership means a blocking `join()` can hang at
            // shutdown, so we detach both daemon threads and let process
            // exit reap them (story-003 Deviations / RISK note).
            #[cfg(target_os = "linux")]
            if let Ok(mut slot) = input_pipeline.lock()
                && slot.take().is_some()
            {
                log::info!("input_pipeline=detached");
            }
            overlay_gpu = None; // drop GPU resources before exit
            log::info!("shutdown=clean");
        }
        _ => {}
    });

    0
}

/// Builds the capture driver at `Ready`, excluding the overlay window from
/// capture once (self-capture prevention, DC-6). Non-fatal: on failure it logs
/// the typed error and returns `None` (the overlay then presents clear frames).
///
/// Empirical self-capture decision (story-003 §C): when
/// `LUMINOS_NO_EXCLUDE=1` is set, the overlay XID is NOT passed to
/// `set_excluded_windows`, skipping the per-frame unmap/remap flicker + perf
/// cost — use only after confirming the transparent overlay does not
/// self-capture in the AC-1.1 screenshot.
#[cfg(target_os = "linux")]
fn init_capture_driver(
    app_handle: &tauri::AppHandle,
) -> Option<crate::capture_driver::CaptureDriver> {
    let Some(handle) = app_handle.try_state::<LuminosHandle>() else {
        log::error!("capture_driver init: LuminosHandle not registered");
        return None;
    };

    let bounds = capture_screen_bounds(app_handle);
    let exclude = std::env::var("LUMINOS_NO_EXCLUDE").as_deref() != Ok("1");
    let overlay_xid = if exclude {
        handle.overlay_window_id()
    } else {
        log::info!("LUMINOS_NO_EXCLUDE=1: skipping overlay self-capture exclusion");
        None
    };

    match crate::capture_driver::CaptureDriver::new(overlay_xid, bounds) {
        Ok(driver) => {
            log::info!("capture_driver=ready");
            Some(driver)
        }
        Err(e) => {
            log::error!("capture_driver init failed: '{e}'");
            None
        }
    }
}

/// Resolves the bounds of the display being magnified, from the overlay's
/// current geometry (the overlay is sized to the primary monitor).
#[cfg(target_os = "linux")]
fn capture_screen_bounds(app_handle: &tauri::AppHandle) -> luminos_types::ScreenRect {
    let Some(overlay) = app_handle.get_webview_window(OVERLAY_LABEL) else {
        return luminos_types::ScreenRect {
            x: 0,
            y: 0,
            width: FALLBACK_SIZE.0,
            height: FALLBACK_SIZE.1,
        };
    };
    let (width, height) = overlay
        .inner_size()
        .map_or(FALLBACK_SIZE, |s| (s.width, s.height));
    let (x, y) = overlay.outer_position().map_or((0, 0), |p| (p.x, p.y));
    luminos_types::ScreenRect {
        x,
        y,
        width,
        height,
    }
}

/// Spawns the X11 input pipeline at `Ready`: an `X11InputMonitor` feeding an
/// `InputProcessingTask` that mutates the SAME `AppState` `ArcSwap` the loop
/// reads, waking the loop via the `AppNotifier` (FR-4/FR-5). Non-fatal: on
/// failure it logs and returns `None` (magnification still renders; cursor
/// tracking/hotkeys are simply inert).
#[cfg(target_os = "linux")]
fn init_input_pipeline(
    app_handle: &tauri::AppHandle,
    notifier: &AppNotifier,
) -> Option<luminos_core::InputProcessingTask> {
    use luminos_platform::traits::input_monitor::InputMonitor as _;

    let Some(handle) = app_handle.try_state::<LuminosHandle>() else {
        log::error!("input_pipeline init: LuminosHandle not registered");
        return None;
    };

    let monitor = match luminos_platform::linux_x11::X11InputMonitor::new() {
        Ok(m) => m,
        Err(e) => {
            log::error!("input_pipeline: X11InputMonitor unavailable: '{e}'");
            return None;
        }
    };
    let rx = match monitor.subscribe_input_events(256) {
        Ok(rx) => rx,
        Err(e) => {
            log::error!("input_pipeline: subscribe_input_events failed: '{e}'");
            return None;
        }
    };

    // The StateManager MUST wrap the SAME Arc<ArcSwap<AppState>> as the loop so
    // input writes are visible to the render's lock-free load() (story-003 §B).
    let state_manager = luminos_core::StateManager::new(Arc::clone(&handle.app_state));
    let task = match luminos_core::InputProcessingTask::spawn(
        rx,
        state_manager,
        luminos_core::HotkeyMatcher::default(),
        notifier.clone(),
    ) {
        Ok(task) => task,
        Err(e) => {
            log::error!("input_pipeline: failed to spawn processor: '{e}'");
            return None;
        }
    };

    log::info!("input_pipeline=ready (X11 monitor + processor spawned)");
    Some(task)
}

/// Initializes the overlay GPU surface from the owned overlay window.
///
/// # Errors
///
/// Returns [`AppError::OverlayMissing`] if the overlay window is not registered
/// at `Ready`, or [`AppError::Gpu`] if surface/device creation fails.
fn init_overlay_gpu(app_handle: &tauri::AppHandle) -> Result<OverlayGpu, AppError> {
    let overlay = app_handle
        .get_webview_window(OVERLAY_LABEL)
        .ok_or_else(|| AppError::OverlayMissing(OVERLAY_LABEL.to_string()))?;
    let (width, height) = overlay
        .inner_size()
        .map_or(FALLBACK_SIZE, |s| (s.width, s.height));

    // Bake the interpolation method + target fps from the seeded settings.
    // Phase 0 fixes interpolation at startup (Renderer has no runtime switch).
    let (method, target_fps) = if let Some(handle) = app_handle.try_state::<LuminosHandle>() {
        let s = handle.app_state.load();
        (
            crate::capture_driver::interpolation_method_for(s.settings.magnification.interpolation),
            s.settings.magnification.target_fps,
        )
    } else {
        log::warn!("LuminosHandle unavailable at GPU init; using default interpolation/fps");
        (luminos_gpu::InterpolationMethod::Bilinear, 60)
    };

    let gpu = OverlayGpu::new(overlay, width.max(1), height.max(1), method, target_fps)?;
    log::info!(
        "surface_ok: overlay GPU initialized at '{width}x{height}' method '{method:?}' \
         target_fps '{target_fps}'"
    );
    Ok(gpu)
}

/// Binds the platform `WindowManager` to the overlay's X11 window id (story
/// 002) and stores it on the managed [`LuminosHandle`] for story 003.
///
/// Non-fatal: on any failure it logs the typed error and returns; the overlay
/// still renders (the bridge only adds trait-level control, not the surface).
/// When `LUMINOS_SELF_CAPTURE_PROBE=1`, it also exercises the shipped
/// `set_excluded_windows` hook against a real capture (T009 / RISK-002).
#[cfg(target_os = "linux")]
fn init_window_manager(app_handle: &tauri::AppHandle) {
    use tauri::Manager as _;

    let Some(overlay) = app_handle.get_webview_window(OVERLAY_LABEL) else {
        log::error!("windowmanager bridge: overlay window '{OVERLAY_LABEL}' not found");
        return;
    };
    let Some(handle) = app_handle.try_state::<LuminosHandle>() else {
        log::error!("windowmanager bridge: LuminosHandle not registered");
        return;
    };

    let bounds = overlay_display_bounds(&overlay);
    let manager = match crate::overlay_bridge::build_window_manager(&overlay, bounds) {
        Ok(manager) => manager,
        Err(e) => {
            log::error!("windowmanager bridge failed: '{e}'");
            return;
        }
    };

    // Make the overlay genuinely click-through. tao's
    // `set_ignore_cursor_events` (setup_overlay_window) only shapes the overlay
    // toplevel; the embedded WebKitWebView child window still grabs pointer
    // events across the full-screen overlay and ate clicks on the control panel
    // (e.g. its close button). Empty the X11 input region of the overlay AND its
    // descendants now that the webview child is realized. Non-fatal: log on
    // failure (the overlay still renders, just not click-through).
    if let Err(e) = manager.set_input_passthrough(true) {
        log::error!("overlay input passthrough (click-through) failed: '{e}'");
    }

    let xid = manager.overlay_window_id();
    handle.set_window_manager(manager);

    if let Some(xid) = handle.overlay_window_id() {
        log::info!("overlay_window_id_exposed={xid} (reachable via LuminosHandle for story 003)");
    }

    if std::env::var("LUMINOS_SELF_CAPTURE_PROBE").as_deref() == Ok("1") {
        probe_self_capture(xid);
    }
}

/// Resolves the overlay's target display rectangle from its current geometry.
#[cfg(target_os = "linux")]
fn overlay_display_bounds(overlay: &tauri::WebviewWindow) -> luminos_platform::traits::ScreenRect {
    let (width, height) = overlay
        .inner_size()
        .map_or(FALLBACK_SIZE, |s| (s.width, s.height));
    let (x, y) = overlay.outer_position().map_or((0, 0), |p| (p.x, p.y));
    luminos_platform::traits::ScreenRect {
        x,
        y,
        width,
        height,
    }
}

/// T009 / RISK-002: exercises the shipped `set_excluded_windows(&[overlay_xid])`
/// hook and captures one frame, logging a `self_capture_probe=` finding. This is
/// the seam story 003 wires into the render loop; here it only proves the hook
/// runs without panicking and records observed flicker behavior. Test-gated by
/// `LUMINOS_SELF_CAPTURE_PROBE` so it never runs in production.
#[cfg(target_os = "linux")]
fn probe_self_capture(overlay_xid: Option<u64>) {
    use luminos_platform::traits::ScreenCapture as _;

    let Some(xid) = overlay_xid else {
        log::warn!("self_capture_probe=skipped (no overlay xid)");
        return;
    };

    let mut capture = match luminos_platform::linux_x11::XcbCapture::new() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("self_capture_probe=capture_unavailable ('{e}')");
            return;
        }
    };
    capture.set_excluded_windows(&[xid]);

    let displays = capture.list_displays().unwrap_or_default();
    let Some(display) = displays.first() else {
        log::warn!("self_capture_probe=no_display");
        return;
    };

    match capture.capture_frame(&display.id, None) {
        Ok(frame) => log::info!(
            "self_capture_probe=ok excluded_xid='{xid}' frame='{}x{}' \
             (RISK-002 finding: unmap/remap exclusion ran; flicker is the \
             documented cost under tao/GTK3 — see Shared Context)",
            frame.width,
            frame.height
        ),
        Err(e) => log::warn!(
            "self_capture_probe=capture_failed excluded_xid='{xid}' err='{e}' \
             (hook exercised; capture backend unavailable in this env)"
        ),
    }
}

/// Presents one frame when a GPU surface exists (Linux: live magnification).
///
/// Reads `AppState` lock-free (`ArcSwap`). If magnification is inactive, presents
/// a transparent clear frame (`inactive_clear`). Otherwise captures the region
/// around the tracked viewport at the current zoom and magnifies it
/// (`magnify_present`); on capture failure it reuses the last frame
/// (`capture_failed`, FR-7) and never panics. Where no presentable adapter
/// exists (e.g. Xvfb), the present is a no-op (`present_skipped`) and the render
/// logic is covered by the offscreen unit test. The loop *cadence* is measured
/// separately by the marshaled heartbeat (see `spawn_cadence_timer`).
#[cfg(target_os = "linux")]
fn present_if_ready(
    gpu: Option<&mut OverlayGpu>,
    capture_driver: Option<&mut crate::capture_driver::CaptureDriver>,
    app_state: &ArcSwap<AppState>,
) {
    let Some(gpu) = gpu else {
        return;
    };

    let state = app_state.load();

    // Inactive / toggle-off: present a transparent clear frame (T008).
    if !state.is_active {
        present_clear(gpu);
        return;
    }

    // Active but no capture driver (capture backend unavailable): clear.
    let Some(capture_driver) = capture_driver else {
        present_clear(gpu);
        return;
    };

    let viewport = gpu.viewport_size();
    let zoom = state.settings.magnification.zoom_level;
    let mouse = state.mouse_position;

    // Compute the region first (advances tracking) so its origin is observable
    // even when the present itself fails under headless software GL (DC-10).
    let region = capture_driver.region_for_state(mouse, viewport, zoom);
    log::debug!(
        "magnify_region zoom='{zoom}' mouse='{},{}' region='{}x{}@{},{}'",
        mouse.x,
        mouse.y,
        region.width,
        region.height,
        region.x,
        region.y,
    );

    match capture_driver.capture_region(region) {
        Ok(frame) => {
            // `magnify_capture` proves capture→frame succeeded; the present may
            // still fail under headless Xvfb (EGL surfaceless, DC-10), in which
            // case `render` logs the error but the loop must not panic (FR-7).
            log::debug!("magnify_capture frame='{}x{}'", frame.width, frame.height);
            match gpu.render(&frame) {
                Ok(()) => log::debug!("magnify_present zoom='{zoom}'"),
                Err(e) => log::warn!("magnify_present_skipped: '{e}'"),
            }
        }
        Err(e) => {
            // FR-7: reuse the last source texture; never panic.
            gpu.handle_capture_failure();
            log::warn!("capture_failed: '{e}' (reusing last frame)");
        }
    }
}

/// Presents one frame when a GPU surface exists (non-Linux: clear only — no
/// capture backend is wired on other platforms yet).
#[cfg(not(target_os = "linux"))]
fn present_if_ready(gpu: Option<&mut OverlayGpu>, app_state: &ArcSwap<AppState>) {
    let _ = app_state;
    let Some(gpu) = gpu else {
        return;
    };
    present_clear(gpu);
}

/// Logs the live `AppState` (`state mouse='x,y' zoom='Z' active='B'`) when it
/// changes, gated by `LUMINOS_LOG_STATE=1`. Test-only observability: lets the
/// subprocess tests assert the input pipeline's cursor/hotkey writes landed,
/// independent of the GPU path (which cannot run under headless Xvfb, DC-10).
/// `last` de-dups so it logs on change, not every tick.
fn log_state_if_enabled(app_state: &ArcSwap<AppState>, last: &mut Option<(i32, i32, u32, bool)>) {
    if std::env::var("LUMINOS_LOG_STATE").as_deref() != Ok("1") {
        return;
    }
    let state = app_state.load();
    let zoom = state.settings.magnification.zoom_level;
    let key = (
        state.mouse_position.x,
        state.mouse_position.y,
        zoom.to_bits(),
        state.is_active,
    );
    if *last == Some(key) {
        return;
    }
    *last = Some(key);
    log::info!(
        "state mouse='{},{}' zoom='{zoom}' active='{}'",
        state.mouse_position.x,
        state.mouse_position.y,
        state.is_active,
    );
}

/// Emits `ZoomChangedEvent`/`ModeChangedEvent` to the webview when the live
/// `(zoom, mode)` changes (story 005, FR-6 / AC-2.2).
///
/// `last` de-dups so an event fires only on a real delta, not every 60 Hz tick.
/// Emission is origin-agnostic — it reads `AppState` and cannot tell a
/// hotkey-origin change from a UI-origin one, so the panel may receive a
/// redundant echo of a value it just set via a command (idempotent: the store
/// already holds that value). This is the AD-5 deviation recorded in SUBTASKS;
/// the alternative (origin tagging) is deferred. The very first observation
/// seeds `last` without emitting, so startup does not spuriously notify.
fn emit_state_events(
    app_handle: &tauri::AppHandle,
    app_state: &ArcSwap<AppState>,
    last: &mut Option<(u32, luminos_types::MagnificationMode)>,
) {
    use tauri_specta::Event as _;

    let state = app_state.load();
    let zoom = state.settings.magnification.zoom_level;
    let mode = state.settings.magnification.mode;
    let key = (zoom.to_bits(), mode);

    let Some((last_zoom_bits, last_mode)) = *last else {
        // First observation: seed without emitting (avoid a startup echo).
        *last = Some(key);
        return;
    };
    if key == (last_zoom_bits, last_mode) {
        return;
    }
    *last = Some(key);

    if zoom.to_bits() != last_zoom_bits {
        if let Err(e) = crate::events::ZoomChangedEvent(zoom).emit(app_handle) {
            log::warn!("emit zoom_changed failed: '{e}'");
        } else {
            log::info!("emit zoom_changed={zoom}");
        }
    }
    if mode != last_mode {
        if let Err(e) = crate::events::ModeChangedEvent(mode).emit(app_handle) {
            log::warn!("emit mode_changed failed: '{e}'");
        } else {
            log::info!("emit mode_changed={mode:?}");
        }
    }
}

/// Presents a transparent clear frame, logging the outcome (`inactive_clear` on
/// success, `present_skipped` where no presentable adapter exists).
fn present_clear(gpu: &mut OverlayGpu) {
    if let Err(e) = gpu.render_clear() {
        // No presentable adapter (Xvfb) surfaces here; it is expected headless.
        log::warn!("present_skipped: '{e}'");
    } else {
        log::debug!("inactive_clear");
    }
}

/// Handles per-window events: resize reconfigures the surface; close requests
/// flag a redraw so the loop keeps pumping toward shutdown.
fn handle_window_event(
    label: &str,
    event: &WindowEvent,
    gpu: Option<&mut OverlayGpu>,
    dirty: &AtomicBool,
) {
    match event {
        WindowEvent::Resized(size) if label == OVERLAY_LABEL => {
            if let Some(gpu) = gpu {
                gpu.resize(size.width, size.height);
                log::info!("resized={}x{}", size.width, size.height);
                dirty.store(true, Ordering::Release);
            }
        }
        WindowEvent::CloseRequested { .. } => {
            log::info!("close_requested window='{label}'");
            dirty.store(true, Ordering::Release);
        }
        _ => {}
    }
}

/// Spawns the ~60 Hz cadence timer thread.
///
/// Each tick it marshals a closure onto the main thread (which
/// `run_on_main_thread` runs reliably, unlike a bare `MainEventsCleared`):
/// the closure flags `dirty`, advances the redraw heartbeat, and logs
/// `redraw=N`. Running on the main thread also drives the GTK loop to emit a
/// `MainEventsCleared` where the opportunistic GPU present happens. The timer
/// also polls the termination-signal flag and asks the loop to `exit(0)` on a
/// received SIGTERM/SIGINT.
fn spawn_cadence_timer(
    app_handle: tauri::AppHandle,
    dirty: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
    redraw_count: Arc<AtomicU64>,
) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("luminos-cadence".into())
        .spawn(move || {
            while !shutdown.load(Ordering::Acquire) {
                std::thread::sleep(CADENCE_PERIOD);

                if crate::signal::shutdown_requested() {
                    let exit_app = app_handle.clone();
                    let _ = app_handle.run_on_main_thread(move || exit_app.exit(0));
                    break;
                }

                let tick_dirty = Arc::clone(&dirty);
                let tick_count = Arc::clone(&redraw_count);
                let marshaled = app_handle.run_on_main_thread(move || {
                    tick_dirty.store(true, Ordering::Release);
                    let n = tick_count.fetch_add(1, Ordering::Relaxed) + 1;
                    log::debug!("redraw={n}");
                });
                if marshaled.is_err() {
                    // `run_on_main_thread` only fails once the event loop has
                    // exited (e.g. during shutdown); stop ticking. This is NOT a
                    // shutdown trigger — it is the reaction to one already in
                    // progress.
                    break;
                }
            }
        })
        .unwrap_or_else(|e| {
            // A failed cadence-thread spawn means the overlay never redraws (a
            // frozen magnifier), so surface it loudly. The placeholder thread
            // exists only so the caller still gets a joinable handle.
            log::error!(
                "failed to spawn cadence thread: '{e}'; overlay will NOT redraw (no cadence)"
            );
            std::thread::spawn(|| {})
        })
}

/// Spawns the env-gated debug notifier thread (`LUMINOS_DEBUG_NOTIFY=1`).
fn maybe_spawn_debug_notifier(
    notifier: AppNotifier,
    shutdown: Arc<AtomicBool>,
) -> Option<std::thread::JoinHandle<()>> {
    if std::env::var("LUMINOS_DEBUG_NOTIFY").as_deref() != Ok("1") {
        return None;
    }
    std::thread::Builder::new()
        .name("luminos-debug-notify".into())
        .spawn(move || {
            // Wait past the idle window so the test can distinguish the wake.
            std::thread::sleep(Duration::from_millis(500));
            if shutdown.load(Ordering::Acquire) {
                return;
            }
            notifier.notify_state_changed();
            log::info!("dirty_render: debug notifier set the dirty flag");
        })
        .ok()
}

/// Joins a background thread held in a shared slot, if present.
fn join_thread_slot(slot: &std::sync::Mutex<Option<std::thread::JoinHandle<()>>>) {
    if let Ok(mut guard) = slot.lock()
        && let Some(handle) = guard.take()
    {
        let _ = handle.join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_overlay_label_is_overlay() {
        assert_eq!(OVERLAY_LABEL, "overlay");
    }

    #[test]
    fn app_cadence_period_is_about_60hz() {
        // ~60 Hz → ~16.6 ms period; assert it is in a sane bound.
        assert!(CADENCE_PERIOD.as_micros() >= 16_000);
        assert!(CADENCE_PERIOD.as_micros() <= 17_000);
    }
}
