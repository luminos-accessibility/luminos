# Subtasks: Story E04/001 -- App Shell, Single Event Loop & wgpu Overlay Surface

**Status:** DONE
**Started:** 2026-06-05
**Completed:** 2026-06-05
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 3 | 3 | 0 | 0 |
| 2. Spike — RISK-001 validation (GATE) | 3 | 3 | 0 | 0 |
| 3. Core Implementation | 4 | 4 | 0 | 0 |
| 4. Integration | 3 | 3 | 0 | 0 |
| 5. Polish & Acceptance | 1 | 1 | 0 | 0 |
| **Total** | **14** | **14** | **0** | **0** |

> **RISK-001 RETIRED (2026-06-05).** The single tao/Tauri loop + two-window model
> works end-to-end on X11 under Xvfb+picom: overlay opens transparent/undecorated/
> click-through, a `wgpu::Surface<'static>` is built from the owned overlay
> `WebviewWindow` handle (`surface_created`), the redraw cadence holds a steady
> ~60 Hz, `LuminosHandle` managed state is retrievable, the `AppNotifier` wake
> path triggers a render, and SIGTERM shuts down cleanly (exit 0). The
> raw-wry+tao fallback was NOT needed. 23 luminos-app tests pass (16 unit + 7
> subprocess), prior 447 workspace tests unaffected.

> **GATE:** Phase 2 (spike) MUST pass before Phase 3. If the two-window Tauri model fails the spike (no stable cadence, surface/transparency unworkable), STOP and escalate to the raw-wry+tao fallback (DESIGN Alternatives #3) — do not proceed on the Tauri path.
>
> **Harness note:** `tauri::App::run` never returns and owns the main thread, so `run()`-driven behavior is tested via a **subprocess harness** (spawn the binary under Xvfb+picom; assert via `xprop`/`xwininfo`, `redraw=N`/`shutdown=clean` log lines, and exit code). Pure logic uses in-process seam tests. See DESIGN → Testing Strategy.

---

## Phase 1: Setup

### T001 -- Enable the Tauri app build (Cargo feature, build.rs, tauri.conf.json)
**Traces to:** FR-1, FR-2
**Status:** DONE
**Files:** `crates/luminos-app/Cargo.toml`, `crates/luminos-app/build.rs`, `crates/luminos-app/tauri.conf.json`, `crates/luminos-app/capabilities/` (placeholder)

**TDD Cycle:** (setup — no Red)
1. **Green:**
   - [ ] **Keep the `tauri` feature gate**; set `default = ["tauri"]` for `luminos-app` and put `tauri`, `tauri-build`, `wgpu`, `raw-window-handle` under it (workspace pins: `wgpu=29.0.3`, `tauri=2.11.2`, `tauri-build=2.6.2`, `raw-window-handle=0.6.2`). Do NOT make webkit2gtk a hard requirement for `cargo build` of unrelated crates.
   - [ ] Add `build.rs` calling `tauri_build::build()` (gated on the `tauri` feature).
   - [ ] Author minimal `tauri.conf.json`: identifier `gg.luminos.app` (confirm), product name, two windows ("main" placeholder page + "overlay" `transparent`/`decorations:false`/`alwaysOnTop:true`/`skipTaskbar:true`), `bundle.license = "GPL-3.0-only"` (matches workspace `license`), `frontendDist: "../ui/dist"` with a placeholder `index.html`.
   - [ ] Author a **minimal capability stub** `capabilities/default.json` granting `core:default` to the `main` webview so it loads (HLP DC-8). Story 005 extends this same file to `core:default` + `core:event:default` + `shell:allow-open`. Native Rust window ops in the spike are not capability-gated; if any is found blocked, add the needed permission here rather than waiting on 005.
2. **Refactor:**
   - [x] Document required system libs (webkit2gtk-4.1, libsoup-3.0) in a build comment + completion note; confirm the `--exclude luminos-app` CLAUDE.md convention still holds for lib-less environments.

**Completion Notes:**
> `luminos-app/Cargo.toml`: `default = ["tauri"]`; the `tauri` feature now pulls
> `wgpu`, `raw-window-handle`, `pollster`, plus Linux-only `x11rb` + `libc`.
> `build.rs` calls `tauri_build::build()` under the feature. `tauri.conf.json`
> EXTENDED (not clobbered — story 006 owns the frontend wiring): identifier
> `dev.luminos.app`, `bundle.license = "GPL-3.0-only"`, `bundle.icon`, the
> existing `control-panel` window + `frontendDist: ../../ui/dist` kept, and
> `security.capabilities = ["default"]`. Minimal capability stub at
> `capabilities/default.json` grants the `control-panel` webview only
> `core:default` (DC-8/NFR-5). Native window ops (overlay create, transparency,
> ignore-cursor-events) are NOT webview-capability-gated and need nothing here.
> Generated a valid 512x512 RGBA `icons/icon.png` via Node (zlib + manual PNG
> chunks — no Python). Added a transparent `ui/public/overlay.html` so the
> overlay webview yields a valid rwh without compositing DOM over the GPU
> (tauri #9220); it lands in `ui/dist/overlay.html` via `pnpm build`. The
> `--exclude luminos-app` convention still holds for lib-less envs.

---

### T002 -- Top-level error type and module skeleton
**Traces to:** NFR-4
**Status:** DONE
**Files:** `crates/luminos-app/src/app_error.rs`, `crates/luminos-app/src/{handle,notifier,overlay_gpu}.rs` (stubs), `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `app_error_from_tauri_error_maps` -- `AppError: From<tauri::Error>` compiles and maps variant.
2. **Green:**
   - [x] Define `AppError` per DESIGN; create module stubs (`handle`, `notifier`, `overlay_gpu`) with signatures only.
3. **Refactor:**
   - [x] `clippy::pedantic` clean; no `unwrap`/`expect`.

**Completion Notes:**
> `app_error.rs`: `AppError` (thiserror) with `Tauri(#[from] tauri::Error)`,
> **`Config(#[from] ConfigError)`** (DESIGN correction #1 — ConfigManager is real,
> so AppError gains the From), `Gpu(String)`, `OverlayMissing(String)`,
> `NoCompositor`. Crate restructured into `lib.rs` + thin `main.rs` so modules
> are unit-testable in-process. Tests: `app_error_from_tauri_error_maps`,
> `app_error_from_config_error_maps`, `app_error_gpu_carries_message`,
> `app_error_overlay_missing_names_window` — all green.

---

### T003 -- `ConfigManager` stub + `LuminosHandle` managed-state struct
**Traces to:** FR-6, AC-3.1
**Status:** DONE
**Files:** `crates/luminos-core/src/config/{mod,manager}.rs`, `crates/luminos-app/src/handle.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `config_manager_stub_default` -- minimal empty `ConfigManager` constructs via `Default` (no I/O; story 004 fills it in).
   - [ ] `handle_holds_real_app_state` -- construct `LuminosHandle` with `Arc<ArcSwap<AppState>>`; assert `app_state.load()` returns seeded `AppState::default()`; `config` is `Arc<Mutex<Option<ConfigManager>>>` = `None`.
2. **Green:**
   - [x] ~~Add the `ConfigManager` stub~~ → used the REAL `ConfigManager` (DESIGN correction #1; story 004 DONE). `run()` calls `seed_initial_state()`; implemented `LuminosHandle` per DESIGN.
3. **Refactor:**
   - [x] Documented thread-safety; `config` behind `Mutex`, off the render path.

**Completion Notes:**
> NO ConfigManager stub created — the real `luminos_core::{ConfigManager,
> ConfigError, seed_initial_state}` (story 004) is wired directly.
> `handle.rs`: `LuminosHandle { app_state: Arc<ArcSwap<AppState>>, config:
> Arc<Mutex<Option<ConfigManager>>>, notifier: AppNotifier, app: AppHandle }` +
> `new(...)`. `run()` seeds via `seed_initial_state()` → `Some(manager)` on Ok,
> `warn!` + `AppState::default()` + `None` on Err (NoConfigDir). Tests
> `handle_holds_real_app_state`, `handle_app_state_is_shared_with_state_manager`,
> `handle_config_flag_round_trips` cover the seam in-process; the live half is the
> `managed_state_ok` subprocess probe.

---

## Phase 2: Spike — RISK-001 validation (GATE)

### T004 [P] -- Spike: redraw cadence inside Tauri's `run` callback
**Traces to:** FR-5, AC-2.3
**Status:** DONE
**Files:** `crates/luminos-app/src/main.rs`, `crates/luminos-app/tests/redraw_cadence.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `redraw_cadence` (subprocess, Xvfb) -- spawn the binary; parse `redraw=N` heartbeat log lines; assert N advances by ≥ 30 within 1.0 s wall-clock.
2. **Green:**
   - [x] Implemented in `run_event_loop`. **Chosen mechanism (empirically validated):** a ~60 Hz timer thread marshals the heartbeat ITSELF onto the main thread via `AppHandle::run_on_main_thread`, which sets `dirty`, advances the counter and logs `redraw=N`. The GPU present is opportunistic in the resulting `MainEventsCleared`. No winit Poll/request_redraw.
3. **Refactor:**
   - [x] Recorded the cadence mechanism here + in HLP Discovered Constraints (DC-9).

**Completion Notes:**
> **tao #635 is real AND a bare `run_on_main_thread(|| {})` does NOT reliably
> provoke `MainEventsCleared`** — measured alternating 60/s vs ~1/s with that
> approach. The stable fix: marshal the *heartbeat closure itself*
> (`dirty.store(true); count++; log "redraw=N"`) via `run_on_main_thread`, which
> runs the closure RELIABLY. Result: rock-solid 60–61 redraws/sec across repeated
> runs under Xvfb+picom. `redraw=N` (cadence) and `frame_presented` (GPU present)
> are SEPARATE markers so cadence is measurable even where no presentable adapter
> exists. Test `redraw_cadence_advances_over_one_second` asserts ≥30/1.0s (passes
> at ~60). NOTE: env_logger block-buffers a redirected stderr; routed it through a
> per-write-flushing `Target::Pipe` adapter in `main.rs` so the subprocess log is
> real-time.

---

### T005 [P] -- Spike: wgpu surface from the OWNED overlay window + clear frame
**Traces to:** FR-4, AC-2.1
**Status:** DONE
**Files:** `crates/luminos-app/src/overlay_gpu.rs`, `crates/luminos-app/tests/overlay_surface.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `overlay_gpu_offscreen_render_clear` (GPU unit, Mesa llvmpipe) -- clear-frame logic against a headless device + offscreen `TextureView` (no window surface needed); assert submit succeeds.
   - [ ] `overlay_surface_presents` (subprocess, Mesa) -- binary logs `surface_ok` + `frame_presented` from the real overlay surface.
2. **Green:**
   - [ ] Implement `OverlayGpu::new(window: tauri::WebviewWindow, w, h)` taking an **owned** window (Arc-backed, `'static`) → `Instance::create_surface(window.clone())` → `Surface<'static>` → adapter/device/queue → configure `Bgra8UnormSrgb` + alpha. Implement `render_clear`/`resize`.
3. **Refactor:**
   - [x] Map wgpu errors to `AppError::Gpu(..)`; reuse `luminos_gpu::surface::select_alpha_mode` (PreMultiplied→PostMultiplied→Opaque).

**Completion Notes:**
> `overlay_gpu.rs`: `OverlayGpu::new(window: tauri::WebviewWindow, w, h)` takes the
> OWNED window → `create_surface(window.clone())` yields `Surface<'static>`,
> original kept in `_window`. Reuses `luminos_gpu::device::{create_wgpu_instance,
> create_gpu_device}` (the latter async → `pollster::block_on`) +
> `surface::{configure_surface, select_alpha_mode}`. `render_clear` handles the
> wgpu 29 `CurrentSurfaceTexture` enum (Success|Suboptimal usable;
> Lost/Outdated → reconfigure + retry once). Clear/submit extracted into
> `encode_clear` so it's unit-testable headlessly. Test
> `overlay_gpu_offscreen_render_clear` (Mesa llvmpipe, `compatible_surface:None`)
> PASSES — covers AC-2.1 render logic. Subprocess `overlay_surface_*` asserts
> `surface_created` (the RISK-001 linchpin: a valid `Surface<'static>` from the
> tao window's rwh-0.6 handle). DEVIATION: under a headless Xvfb no GPU adapter is
> *presentable* (EGL "surfaceless platform" / "no compatible adapter") so
> `frame_presented` is conditional there — this is an Xvfb+software-GL limitation,
> NOT a coexistence failure; `surface_created` is the live-window evidence and the
> offscreen unit covers the render. On real GPU / a presentable software stack,
> `frame_presented` fires.

---

### T006 -- Spike: overlay transparency + click-through under compositor
**Traces to:** FR-3, AC-2.2, NFR-3
**Status:** DONE
**Files:** `crates/luminos-app/src/main.rs`, `crates/luminos-app/tests/overlay_attrs.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `overlay_window_is_transparent_clickthrough` (subprocess, Xvfb+picom) -- `xprop` shows `_NET_WM_STATE_ABOVE`, undecorated, skip-taskbar; `ignore_cursor_events` logged; pointer-passthrough probe.
   - [ ] `overlay_no_compositor_logs_warn` -- with no compositor, binary logs `NoCompositor` warn and continues (no panic, no crash).
2. **Green:**
   - [ ] Build overlay transparent/undecorated/always_on_top/skip_taskbar; `set_ignore_cursor_events(true)`; add compositor detection (`_NET_WM_CM_S0` selection owner).
3. **Refactor:**
   - [x] Documented the GTK/Xvfb quirks (GDK_BACKEND + WEBKIT env) in the harness + HLP DC-9/DC-10.

**Checkpoint (GATE):** ✅ T004-T006 PASS under Xvfb+picom / Mesa llvmpipe. **No fundamental coexistence failure — RISK-001 RETIRED.** Proceeded to Phase 3 on the Tauri path; raw-wry+tao fallback NOT triggered.

**Completion Notes:**
> Overlay built transparent/undecorated/always_on_top/skip_taskbar/focused(false)
> via `WebviewWindowBuilder` in `setup`, then `set_ignore_cursor_events(true)`
> (logged `ignore_cursor_events=true`). Compositor detection in `compositor.rs`
> via the `_NET_WM_CM_S<screen>` selection owner (x11rb); absent → `warn!`
> `NoCompositor` + continue opaque (NFR-3), never panic. Verified live:
> `overlay_window_is_undecorated_and_clickthrough` (x11rb sees both windows
> VIEWABLE; overlay `_MOTIF_WM_HINTS` decorations=0, full-screen size) and
> `overlay_no_compositor_logs_warn_and_continues` (NoCompositor + keeps rendering)
> — both PASS. WINDOW-INSPECTION DEVIATION: `xdotool --name` does NOT see the
> WM-less GTK windows and `xwininfo` is not installed; the harness walks the X
> tree with **x11rb `query_tree`** instead (sees override-redirect/WM-less
> windows). always_on_top/skip_taskbar are WM-enforced EWMH hints not observable
> under a WM-less Xvfb, so they're asserted via the builder call + log, not xprop.

---

## Phase 3: Core Implementation

### T007 -- `AppNotifier` (dirty-flag EventNotifier)
**Traces to:** FR-7, AC-3.1
**Status:** DONE
**Files:** `crates/luminos-app/src/notifier.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `app_notifier_sets_dirty_flag` (unit, no runtime) -- `notify_state_changed()` flips the shared `Arc<AtomicBool>` to `true`.
2. **Green:**
   - [x] Implemented `AppNotifier { dirty: Arc<AtomicBool> }` + `impl EventNotifier`; `dirty_flag()` accessor.
3. **Refactor:**
   - [x] `AppNotifier: Clone + Send + Sync` (asserted by `app_notifier_is_clone_send_sync`).

**Completion Notes:**
> `notifier.rs`: `AppNotifier::notify_state_changed()` stores `true` (Release) in
> the shared `Arc<AtomicBool>`; the loop drains with `swap(false, Acquire)`. The
> existing blanket `impl EventNotifier for winit::EventLoopProxy<LuminosEvent>`
> (backing the 418 prior tests) is UNTOUCHED — `AppNotifier` is a second impl.
> Tests: `app_notifier_sets_dirty_flag`, `app_notifier_dirty_flag_is_shared`,
> `app_notifier_is_clone_send_sync`, `app_notifier_usable_as_dyn_event_notifier`
> — all green. Note `EventNotifier: Send + 'static` (not `Sync`), but `AppNotifier`
> is independently `Send + Sync` so 003/005 worker threads can hold it.

---

### T008 -- App bootstrap: build + manage + setup (two windows)
**Traces to:** FR-1, FR-2, FR-6, AC-1.1, AC-3.1
**Status:** DONE
**Files:** `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `app_boots_two_windows` (subprocess) -- after launch, `xwininfo` shows both "main" and "overlay" windows; binary logs `managed_state_ok` (probe retrieves `State<LuminosHandle>` and reads `app_state`).
2. **Green:**
   - [x] Wired `Builder::default().setup(manage(handle) + open overlay + set_ignore_cursor_events).build(generate_context!()).run(...)`. Shared dirty flag built in `run()`; `AppNotifier` from it; control-panel window comes from `tauri.conf.json`, overlay opened in setup.
3. **Refactor:**
   - [x] Extracted `setup_overlay_window`, `primary_monitor_bounds`, `probe_managed_state`; `main` is thin (logger init + `app::run()`).

**Completion Notes:**
> `app::run()` builds `LuminosHandle` (real ArcSwap + seeded config), `.manage`s
> it, opens the overlay sized to the primary monitor (fallback 1920x1080), and
> runs the single `App::run` loop. Subprocess `app_boots_two_windows_and_exits_clean`
> asserts via x11rb that BOTH windows ("Luminos Control Panel" + "Luminos Overlay")
> are VIEWABLE as ONE process (DEVIATION from the SUBTASKS text: `xwininfo` is not
> installed and the windows aren't `main`/`overlay`-titled — the control panel is
> `control-panel` per conf.json, titled "Luminos Control Panel"). `managed_state_ok`
> probe proves `State<LuminosHandle>` retrieval + lock-free `AppState` read (AC-3.1
> state half).

---

### T009 -- Run loop: init surface on Ready, render on MainEventsCleared, handle resize
**Traces to:** FR-4, FR-5, AC-2.1
**Status:** DONE
**Files:** `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `app_presents_frames_after_ready` (subprocess, Mesa) -- after `Ready`, `frame_presented` heartbeats continue; resizing the overlay logs `resized=WxH` with no surface error.
2. **Green:**
   - [x] In `.run`: `RunEvent::Ready` → `OverlayGpu::new(get_webview_window("overlay"))`; `MainEventsCleared` → dirty-gated `present_if_ready`; `WindowEvent::Resized` → `OverlayGpu::resize` (logs `resized=WxH`).
3. **Refactor:**
   - [x] `CurrentSurfaceTexture::Lost/Outdated` → reconfigure + retry once.

**Completion Notes:**
> `RunEvent::Ready` initializes `OverlayGpu` from the owned overlay window
> (`init_overlay_gpu`); `surface_created` + `surface_ok` logged when it succeeds.
> Present happens in `MainEventsCleared` when dirty. Resize reconfigures the
> surface. Covered by `overlay_surface_is_created_from_owned_window` and the
> cadence/lifecycle tests. (No standalone `resized` subprocess test under a
> WM-less Xvfb where the window isn't resized externally; `resize()` is exercised
> by `OverlayGpu` + the `WindowEvent::Resized` arm, and visually unblocked for
> story 003.)

---

### T010 -- Graceful shutdown
**Traces to:** FR-8, AC-1.1
**Status:** DONE
**Files:** `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `app_shuts_down_cleanly` (subprocess, timeout-guarded) -- `SIGTERM`/window close → `shutdown=clean` logged, threads joined, exit code 0, no hang.
2. **Green:**
   - [x] On `RunEvent::ExitRequested|Exit` (idempotent via `compare_exchange`): join cadence/debug threads, drop `OverlayGpu`, exit. SIGTERM/SIGINT drive this via a `sigaction` handler (see refactor) since tao does NOT map signals to `ExitRequested`.
3. **Refactor:**
   - [x] Clean drop order (GPU dropped before exit); no leak warnings observed.

**Checkpoint:** ✅ All Phase 1-3 unit + subprocess tests pass; app boots, creates the overlay surface, holds ~60 Hz cadence, exits cleanly (exit 0).

**Completion Notes:**
> CRITICAL GTK INTERACTION: tao/GTK3 does NOT convert SIGTERM/SIGINT into
> `RunEvent::ExitRequested`. First attempt — `pthread_sigmask(SIG_BLOCK, …)` +
> a `sigwait` thread — BROKE GTK window realization (windows never mapped). The
> working design (`signal.rs`): an async-signal-safe `sigaction` handler that only
> sets a `static AtomicBool`; the cadence thread polls `shutdown_requested()` and
> calls `app.exit(0)` via `run_on_main_thread`, which makes the loop emit
> `ExitRequested`/`Exit`. Teardown runs once (compare_exchange guard). Verified:
> `app_boots_two_windows_and_exits_clean` — SIGTERM → `signal=15 received` →
> `shutdown=requested` → `shutdown=clean` → **exit code 0**, no hang. `libc=0.2.186`
> pinned for `sigaction`.

---

## Phase 4: Integration

### T011 -- Stabilize overlay-attribute subprocess harness
**Traces to:** AC-2.2
**Status:** DONE
**Files:** `crates/luminos-app/tests/overlay_attrs.rs`, `crates/luminos-app/tests/common/`

**TDD Cycle:**
1. **Red:**
   - [ ] Promote T006's assertions into a stable harness; gracefully skip if `xprop`/`xwininfo`/`xdotool` absent (mirroring E03 platform-test pattern). CI MUST install them.
2. **Green:**
   - [x] `tests/common/mod.rs`: `TestDisplay` (launches Xvfb+picom, drops them), `RunningApp` (spawns the binary in its own process group with the headless env, parses log markers, SIGTERM+wait), `find_windows` (x11rb tree walk + `_MOTIF_WM_HINTS`).
3. **Refactor:**
   - [x] All 6 integration test files share `tests/common`; graceful skip if Xvfb absent.

**Completion Notes:**
> Harness gated `#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]`,
> mirroring the E03 pattern. Uses x11rb (NOT xdotool/xwininfo) for window
> assertions. HARNESS GOTCHAS captured: (1) `pkill -f Xvfb` self-kills the test
> shell — use exact-name `pkill -x`; (2) `Command::process_group(0)` so SIGTERM to
> the app can't reach the nextest runner; (3) per-test dedicated display
> (`next_display()` from :180) + run `--test-threads 1`.

---

### T012 -- Notifier→render end-to-end
**Traces to:** AC-3.1
**Status:** DONE
**Files:** `crates/luminos-app/tests/notifier_redraw.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `notify_triggers_render` (subprocess) -- an env-gated debug thread in the binary calls `notify_state_changed()` after an idle period; assert the heartbeat shows a `dirty_render` tick / rate increase within a timeout.
2. **Green:**
   - [x] `maybe_spawn_debug_notifier` (env-gated `LUMINOS_DEBUG_NOTIFY=1`): a thread holds `AppNotifier`, waits past an idle window, calls `notify_state_changed()`, logs `dirty_render`.
3. **Refactor:**
   - [x] Runs under the `ci` nextest profile (retries/relaxed timeouts).

**Completion Notes:**
> `notify_state_changed_triggers_render` (subprocess) spawns the app with
> `LUMINOS_DEBUG_NOTIFY=1`, asserts `dirty_render` then a following `redraw=`.
> PASSES — proves the `AppNotifier` → shared dirty flag → loop-render wake path
> end-to-end (no `request_redraw`, no main-thread marshaling of the flag itself).

---

### T013 -- CI: build the Tauri app + run subprocess/GPU tests
**Traces to:** FR-1, AC-1.1, AC-2.1
**Status:** DONE
**Files:** `.github/workflows/ci.yml`, `CLAUDE.md` (CI command section if changed)

**TDD Cycle:** (CI wiring)
1. **Green:**
   - [x] Added a `test-app` job: installs webkit2gtk-4.1 + libsoup-3.0 + javascriptcoregtk-4.1 + `x11-utils`(xprop) + xdotool + Mesa, sets up Node + builds `ui/dist`, runs clippy + `cargo nextest -p luminos-app --features "tauri ci_platform_tests" --test-threads 1`. (No outer `xvfb-run` — the harness launches its own Xvfb per test.)
   - [x] Excluded `luminos-app` from the workspace lint clippy (needs `ui/dist` for `generate_context!`) and lint it in `test-app` instead. Updated CLAUDE.md CI section (check 8 + 7-jobs count).
2. **Refactor:**
   - [x] `test-app` caches the Cargo registry/target.

**Checkpoint:** CI job authored; locally the equivalent commands are green (see T014).

**Completion Notes:**
> `.github/workflows/ci.yml` gains `test-app`. `xwininfo` is NOT installed
> anywhere (it isn't needed — x11rb is used), so it is NOT added to CI; `x11-utils`
> (xprop) + xdotool are. NOTE: the lint job's clippy was changed to
> `--workspace --exclude luminos-app` because `--all-targets` on luminos-app needs
> a built `ui/dist`.

---

## Phase 5: Polish & Acceptance

### T014 -- Acceptance verification + AC coverage matrix
**Traces to:** All ACs
**Status:** DONE
**Files:** `crates/luminos-app/**`, story docs

**Verification Checklist:**
- [x] AC-1.1 lifecycle: two windows / single process / graceful exit 0 — `app_boots_two_windows_and_exits_clean`
- [x] AC-2.1 wgpu surface from owned overlay window + clear frame — `overlay_gpu_offscreen_render_clear` (unit) + `overlay_surface_is_created_from_owned_window` (subprocess `surface_created`)
- [x] AC-2.2 transparent/undecorated/click-through — `overlay_window_is_undecorated_and_clickthrough` + `overlay_no_compositor_logs_warn_and_continues` (always-on-top/skip-taskbar requested via builder+log; WM-enforced, not observable headless)
- [x] AC-2.3 redraw cadence on GTK3 (marshaled-heartbeat mechanism recorded) — `redraw_cadence_advances_over_one_second`
- [x] AC-3.1 managed state retrievable + dirty-flag wake — `managed_state_handle_is_retrievable` + `notify_state_changed_triggers_render` (+ in-process handle/notifier units)
- [x] `cargo fmt --all -- --check` clean
- [x] Clippy clean (luminos-app `tauri+ci_platform_tests --all-targets` AND `--workspace --exclude luminos-app --all-features`)
- [x] No `unwrap()`/`expect()` in production paths (clippy `unwrap_used`/`expect_used` clean)
- [x] Cadence mechanism recorded in HLP Shared Context (DC-9)
- [x] RISK-001 status recorded in HLP Deviations + Shared Context (RETIRED)

**AC → Test Matrix:**

| AC | Tests (unit `u` / subprocess `s`) | Result |
|----|-----------------------------------|--------|
| AC-1.1 | `app_boots_two_windows_and_exits_clean` (s) | PASS |
| AC-2.1 | `overlay_gpu_offscreen_render_clear` (u, Mesa) + `overlay_surface_is_created_from_owned_window` (s) | PASS |
| AC-2.2 | `overlay_window_is_undecorated_and_clickthrough` (s) + `overlay_no_compositor_logs_warn_and_continues` (s) | PASS |
| AC-2.3 | `redraw_cadence_advances_over_one_second` (s) | PASS |
| AC-3.1 | `managed_state_handle_is_retrievable` (s) + `notify_state_changed_triggers_render` (s) + `handle_*`/`app_notifier_*` (u) | PASS |
| NFR-4 | `app_error_*` (u) — no unwrap/expect, typed errors | PASS |

**Completion Notes:**
> All 5 ACs covered by ≥1 passing test. luminos-app: 23 tests (16 unit + 7
> subprocess) PASS; workspace: prior 447 PASS. fmt/clippy/deny/audit all green.
> RISK-001 RETIRED on the Tauri two-window path (no fallback). The cadence
> mechanism (run_on_main_thread-marshaled heartbeat) and discovered constraints
> are in HLP Shared Context.
>
> **Review follow-up (2026-06-05), all green after, no behavior change to ACs:**
> - I-1: `configure_surface` now returns the applied `wgpu::SurfaceConfiguration`;
>   `OverlayGpu` stores THAT as the single source of truth for `resize`/`Lost`/
>   `Outdated` recovery (removed the drift-prone hand-rebuilt copy). luminos-gpu
>   test callers updated to read `.format`; luminos-gpu tests stay green.
> - Hardened: cadence-thread spawn failure now logs the consequence ("overlay
>   will NOT redraw"); poisoned config mutex in the managed-state probe handled
>   explicitly (log + recover) instead of masking as `false`.
> - P-2: `AppError::{OverlayMissing, NoCompositor}` are now actually constructed
>   (`init_overlay_gpu` returns `OverlayMissing` when the overlay window is absent;
>   the no-compositor warn path uses `NoCompositor`'s Display) — no dead variants.
> - P-1: FR-1 zero-winit invariant documented on `run_event_loop`.
> - De-flaked `redraw_cadence_advances_over_one_second`: it now waits for the
>   cadence to reach steady state (30 warmup heartbeats) before opening the 1.0s
>   sample window, so the GTK/webkit warmup transient no longer skews the first
>   second. Verified 4/4 clean direct runs (no-retry profile) + a clean `ci`
>   full-suite run (was: 1 flaky retry).

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| T003 | Used the REAL `ConfigManager`/`seed_initial_state` instead of landing an empty stub | IMPLEMENTATION_NOTES correction #1: story 004 is DONE and crate-root re-exports the real type. `AppError` gains `From<ConfigError>`. |
| T004 | Cadence driven by a `run_on_main_thread`-marshaled HEARTBEAT, not by gating render on a hoped-for `MainEventsCleared` | Empirically, a bare `run_on_main_thread(\|\| {})` does NOT reliably provoke `MainEventsCleared` on GTK3 (tao #635) — alternated 60/s vs ~1/s. Marshaling the heartbeat closure itself runs reliably (steady ~60 Hz). |
| T005 | Live `frame_presented` is conditional under headless Xvfb (no presentable GPU adapter); `surface_created` is the live-window evidence | wgpu GL/EGL on Xvfb selects a "surfaceless platform" → no surface-compatible adapter. This is an Xvfb+software-GL limitation, NOT a coexistence failure. The offscreen unit test covers the clear/submit render logic; on real GPU/presentable software stacks `frame_presented` fires. |
| T006/T011 | Window attributes asserted via **x11rb `query_tree`**, not `xprop`/`xwininfo`; always-on-top/skip-taskbar via builder+log | `xdotool --name` does not see WM-less GTK windows; `xwininfo` is not installed. EWMH always-on-top/skip-taskbar are WM-enforced and unobservable under a WM-less Xvfb, so they are asserted from the builder call + log line. |
| T010 | Graceful shutdown via a `sigaction` handler (+ cadence-thread poll), NOT `pthread_sigmask`/`sigwait` | Blocking SIGTERM/SIGINT before GTK init breaks GTK window realization. A `sigaction` handler that only sets an atomic does not. New pinned dep `libc=0.2.186`. |
| T001/T008 | Control-panel window keeps story-006's `control-panel` label (titled "Luminos Control Panel"), not DESIGN's `main` | EXTEND, don't clobber, story 006's existing `tauri.conf.json` frontend wiring (IMPLEMENTATION_NOTES §F). The capability + window assertions target `control-panel`. |
| (deps) | New pinned deps added: `pollster=0.4.0`, `libc=0.2.186` (both under `tauri` feature, Linux-only for libc) | `pollster` bridges the async `create_gpu_device` on the runtime-less loop thread; `libc` for `sigaction`. Both recorded in PINNED_VERSIONS §1c, advisory-free, ≤2026-05-21. |
| T002/T008 | Entry point is a thin `main.rs` + `pub fn run()` in `app.rs`, not DESIGN's `fn main() -> Result<(), AppError>` | Testability: the loop logic lives in `luminos_app::app::run()` (lib) so the modules unit-test in-process; `main.rs` only inits the logger and calls `run()`. The DESIGN signature was an illustration of the error-returning contract, which `run() -> Result<(), AppError>` preserves. |
| T005 (review) | `luminos_gpu::surface::configure_surface` now RETURNS the applied `wgpu::SurfaceConfiguration` (was: just `TextureFormat`) | Single source of truth (review I-1): `OverlayGpu` stored a hand-rebuilt config copy that could drift from what was actually applied (used by `resize`/`Lost`/`Outdated` recovery). The helper now returns the exact struct it configured; `OverlayGpu` stores THAT. luminos-gpu test callers read `.format`. luminos-gpu tests stay green. |
