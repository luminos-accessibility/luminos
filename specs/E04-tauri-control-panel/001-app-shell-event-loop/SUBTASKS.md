# Subtasks: Story E04/001 -- App Shell, Single Event Loop & wgpu Overlay Surface

**Status:** NOT STARTED
**Started:** ---
**Completed:** ---
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 3 | 0 | 0 | 3 |
| 2. Spike — RISK-001 validation (GATE) | 3 | 0 | 0 | 3 |
| 3. Core Implementation | 4 | 0 | 0 | 4 |
| 4. Integration | 3 | 0 | 0 | 3 |
| 5. Polish & Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **14** | **0** | **0** | **14** |

> **GATE:** Phase 2 (spike) MUST pass before Phase 3. If the two-window Tauri model fails the spike (no stable cadence, surface/transparency unworkable), STOP and escalate to the raw-wry+tao fallback (DESIGN Alternatives #3) — do not proceed on the Tauri path.
>
> **Harness note:** `tauri::App::run` never returns and owns the main thread, so `run()`-driven behavior is tested via a **subprocess harness** (spawn the binary under Xvfb+picom; assert via `xprop`/`xwininfo`, `redraw=N`/`shutdown=clean` log lines, and exit code). Pure logic uses in-process seam tests. See DESIGN → Testing Strategy.

---

## Phase 1: Setup

### T001 -- Enable the Tauri app build (Cargo feature, build.rs, tauri.conf.json)
**Traces to:** FR-1, FR-2
**Status:** TODO
**Files:** `crates/luminos-app/Cargo.toml`, `crates/luminos-app/build.rs`, `crates/luminos-app/tauri.conf.json`, `crates/luminos-app/capabilities/` (placeholder)

**TDD Cycle:** (setup — no Red)
1. **Green:**
   - [ ] **Keep the `tauri` feature gate**; set `default = ["tauri"]` for `luminos-app` and put `tauri`, `tauri-build`, `wgpu`, `raw-window-handle` under it (workspace pins: `wgpu=29.0.3`, `tauri=2.11.2`, `tauri-build=2.6.2`, `raw-window-handle=0.6.2`). Do NOT make webkit2gtk a hard requirement for `cargo build` of unrelated crates.
   - [ ] Add `build.rs` calling `tauri_build::build()` (gated on the `tauri` feature).
   - [ ] Author minimal `tauri.conf.json`: identifier `gg.luminos.app` (confirm), product name, two windows ("main" placeholder page + "overlay" `transparent`/`decorations:false`/`alwaysOnTop:true`/`skipTaskbar:true`), `bundle.license = "GPL-3.0-only"` (matches workspace `license`), `frontendDist: "../ui/dist"` with a placeholder `index.html`.
   - [ ] Author a **minimal capability stub** `capabilities/default.json` granting `core:default` to the `main` webview so it loads (HLP DC-8). Story 005 extends this same file to `core:default` + `core:event:default` + `shell:allow-open`. Native Rust window ops in the spike are not capability-gated; if any is found blocked, add the needed permission here rather than waiting on 005.
2. **Refactor:**
   - [ ] Document required system libs (webkit2gtk-4.1, libsoup-3.0) in a build comment + completion note; confirm the `--exclude luminos-app` CLAUDE.md convention still holds for lib-less environments.

**Completion Notes:**
>

---

### T002 -- Top-level error type and module skeleton
**Traces to:** NFR-4
**Status:** TODO
**Files:** `crates/luminos-app/src/app_error.rs`, `crates/luminos-app/src/{handle,notifier,overlay_gpu}.rs` (stubs), `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `app_error_from_tauri_error_maps` -- `AppError: From<tauri::Error>` compiles and maps variant.
2. **Green:**
   - [ ] Define `AppError` per DESIGN; create module stubs (`handle`, `notifier`, `overlay_gpu`) with signatures only.
3. **Refactor:**
   - [ ] `clippy::pedantic` clean; no `unwrap`/`expect`.

**Completion Notes:**
>

---

### T003 -- `ConfigManager` stub + `LuminosHandle` managed-state struct
**Traces to:** FR-6, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-core/src/config/{mod,manager}.rs`, `crates/luminos-app/src/handle.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `config_manager_stub_default` -- minimal empty `ConfigManager` constructs via `Default` (no I/O; story 004 fills it in).
   - [ ] `handle_holds_real_app_state` -- construct `LuminosHandle` with `Arc<ArcSwap<AppState>>`; assert `app_state.load()` returns seeded `AppState::default()`; `config` is `Arc<Mutex<Option<ConfigManager>>>` = `None`.
2. **Green:**
   - [ ] Add the `ConfigManager` stub in `luminos-core::config`; implement `LuminosHandle` fields per DESIGN.
3. **Refactor:**
   - [ ] Document thread-safety; ensure no `Send+Sync` violation.

**Completion Notes:**
>

---

## Phase 2: Spike — RISK-001 validation (GATE)

### T004 [P] -- Spike: redraw cadence inside Tauri's `run` callback
**Traces to:** FR-5, AC-2.3
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs`, `crates/luminos-app/tests/redraw_cadence.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `redraw_cadence` (subprocess, Xvfb) -- spawn the binary; parse `redraw=N` heartbeat log lines; assert N advances by ≥ 30 within 1.0 s wall-clock.
2. **Green:**
   - [ ] In `run(|app, RunEvent|…)`, render on `RunEvent::MainEventsCleared` (gated by the dirty flag + a steady-cadence path). **No winit `Poll`/`request_redraw`** (Tauri exposes neither). If `MainEventsCleared` is too sparse on GTK3 (tao #635), add a ~60 Hz timer thread that flips the dirty flag. Emit `redraw=N` heartbeat.
3. **Refactor:**
   - [ ] Record the chosen cadence mechanism in completion notes + epic Shared Context (Discovered Constraints).

**Completion Notes:**
>

---

### T005 [P] -- Spike: wgpu surface from the OWNED overlay window + clear frame
**Traces to:** FR-4, AC-2.1
**Status:** TODO
**Files:** `crates/luminos-app/src/overlay_gpu.rs`, `crates/luminos-app/tests/overlay_surface.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `overlay_gpu_offscreen_render_clear` (GPU unit, Mesa llvmpipe) -- clear-frame logic against a headless device + offscreen `TextureView` (no window surface needed); assert submit succeeds.
   - [ ] `overlay_surface_presents` (subprocess, Mesa) -- binary logs `surface_ok` + `frame_presented` from the real overlay surface.
2. **Green:**
   - [ ] Implement `OverlayGpu::new(window: tauri::WebviewWindow, w, h)` taking an **owned** window (Arc-backed, `'static`) → `Instance::create_surface(window.clone())` → `Surface<'static>` → adapter/device/queue → configure `Bgra8UnormSrgb` + alpha. Implement `render_clear`/`resize`.
3. **Refactor:**
   - [ ] Map wgpu errors to `AppError::Gpu(..)`; pick an `AlphaMode` honoring transparency (fallback if unsupported).

**Completion Notes:**
>

---

### T006 -- Spike: overlay transparency + click-through under compositor
**Traces to:** FR-3, AC-2.2, NFR-3
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs`, `crates/luminos-app/tests/overlay_attrs.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `overlay_window_is_transparent_clickthrough` (subprocess, Xvfb+picom) -- `xprop` shows `_NET_WM_STATE_ABOVE`, undecorated, skip-taskbar; `ignore_cursor_events` logged; pointer-passthrough probe.
   - [ ] `overlay_no_compositor_logs_warn` -- with no compositor, binary logs `NoCompositor` warn and continues (no panic, no crash).
2. **Green:**
   - [ ] Build overlay transparent/undecorated/always_on_top/skip_taskbar; `set_ignore_cursor_events(true)`; add compositor detection (`_NET_WM_CM_S0` selection owner).
3. **Refactor:**
   - [ ] Note tao #7369 (stray label) status; document any GTK quirk.

**Checkpoint (GATE):** T004-T006 pass under Xvfb+picom / Mesa llvmpipe. If a fundamental coexistence failure is found, STOP → raw-wry+tao fallback before Phase 3.

**Completion Notes:**
>

---

## Phase 3: Core Implementation

### T007 -- `AppNotifier` (dirty-flag EventNotifier)
**Traces to:** FR-7, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-app/src/notifier.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `app_notifier_sets_dirty_flag` (unit, no runtime) -- `notify_state_changed()` flips the shared `Arc<AtomicBool>` to `true`.
2. **Green:**
   - [ ] Implement `AppNotifier { dirty: Arc<AtomicBool> }` + `impl EventNotifier`; `dirty_flag()` accessor for the run loop.
3. **Refactor:**
   - [ ] Ensure `AppNotifier: Clone + Send + Sync` so worker threads (003/005) can hold it.

**Completion Notes:**
>

---

### T008 -- App bootstrap: build + manage + setup (two windows)
**Traces to:** FR-1, FR-2, FR-6, AC-1.1, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `app_boots_two_windows` (subprocess) -- after launch, `xwininfo` shows both "main" and "overlay" windows; binary logs `managed_state_ok` (probe retrieves `State<LuminosHandle>` and reads `app_state`).
2. **Green:**
   - [ ] Wire `Builder::default().manage(handle).setup(open both windows + set_ignore_cursor_events).build(generate_context!())`. Create the shared `Arc<AtomicBool>` dirty flag; build `AppNotifier` from it; store overlay `WebviewWindow` for the run loop.
3. **Refactor:**
   - [ ] Extract `setup_windows(app)`; keep `main` thin.

**Completion Notes:**
>

---

### T009 -- Run loop: init surface on Ready, render on MainEventsCleared, handle resize
**Traces to:** FR-4, FR-5, AC-2.1
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `app_presents_frames_after_ready` (subprocess, Mesa) -- after `Ready`, `frame_presented` heartbeats continue; resizing the overlay logs `resized=WxH` with no surface error.
2. **Green:**
   - [ ] In `.run(|app,event|…)`: on `RunEvent::Ready` construct `OverlayGpu` from the OWNED overlay window; on `RunEvent::MainEventsCleared` render (dirty-gated + cadence); on `RunEvent::WindowEvent{ event: WindowEvent::Resized(size), .. }` call `OverlayGpu::resize`. Read `app_state.load()` on the render path (lock-free).
3. **Refactor:**
   - [ ] `SurfaceError::Lost/Outdated` → reconfigure.

**Completion Notes:**
>

---

### T010 -- Graceful shutdown
**Traces to:** FR-8, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `app_shuts_down_cleanly` (subprocess, timeout-guarded) -- `SIGTERM`/window close → `shutdown=clean` logged, threads joined, exit code 0, no hang.
2. **Green:**
   - [ ] On `RunEvent::ExitRequested`/`WindowEvent::CloseRequested`, set shutdown flag, join the cadence/timer thread, drop `OverlayGpu`, allow exit.
3. **Refactor:**
   - [ ] No `wgpu` resource-leak warnings on drop.

**Checkpoint:** All Phase 1-3 subprocess + unit tests pass; app boots, renders a clear overlay frame, exits cleanly.

**Completion Notes:**
>

---

## Phase 4: Integration

### T011 -- Stabilize overlay-attribute subprocess harness
**Traces to:** AC-2.2
**Status:** TODO
**Files:** `crates/luminos-app/tests/overlay_attrs.rs`, `crates/luminos-app/tests/common/`

**TDD Cycle:**
1. **Red:**
   - [ ] Promote T006's assertions into a stable harness; gracefully skip if `xprop`/`xwininfo`/`xdotool` absent (mirroring E03 platform-test pattern). CI MUST install them.
2. **Green:**
   - [ ] Implement shared `tests/common/` util to launch the app under Xvfb and parse logs.
3. **Refactor:**
   - [ ] Dedupe the spawn/parse helpers used by T004/T006/T008-T010.

**Completion Notes:**
>

---

### T012 -- Notifier→render end-to-end
**Traces to:** AC-3.1
**Status:** TODO
**Files:** `crates/luminos-app/tests/notifier_redraw.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `notify_triggers_render` (subprocess) -- an env-gated debug thread in the binary calls `notify_state_changed()` after an idle period; assert the heartbeat shows a `dirty_render` tick / rate increase within a timeout.
2. **Green:**
   - [ ] Wire the env-gated debug trigger thread holding `AppNotifier`.
3. **Refactor:**
   - [ ] De-flake with retry/relaxed timeout per `ci` nextest profile.

**Completion Notes:**
>

---

### T013 -- CI: build the Tauri app + run subprocess/GPU tests
**Traces to:** FR-1, AC-1.1, AC-2.1
**Status:** TODO
**Files:** `.github/workflows/ci.yml`, `CLAUDE.md` (CI command section if changed)

**TDD Cycle:** (CI wiring)
1. **Green:**
   - [ ] Ensure CI installs webkit2gtk-4.1/libsoup-3.0 + `xprop`/`xwininfo`/`xdotool` before building/testing `luminos-app`; build with `--features tauri`; run the new subprocess tests under Xvfb+picom and GPU tests under Mesa llvmpipe (extend existing `test-platform`/`test-gpu` patterns to `-p luminos-app --features ci_platform_tests`).
   - [ ] Confirm the workspace lint job still builds (other crates) without webkit2gtk, or installs it; reconcile `--exclude luminos-app` usage. Update the `CLAUDE.md` CI section if commands change (source of truth).
2. **Refactor:**
   - [ ] Cache webkit/system deps where possible.

**Checkpoint:** CI green; subprocess + GPU tests run under Xvfb+picom / Mesa.

**Completion Notes:**
>

---

## Phase 5: Polish & Acceptance

### T014 -- Acceptance verification + AC coverage matrix
**Traces to:** All ACs
**Status:** TODO
**Files:** `crates/luminos-app/**`, story docs

**Verification Checklist:**
- [ ] AC-1.1 lifecycle: two windows / single process / graceful exit 0
- [ ] AC-2.1 wgpu surface from owned overlay window + clear frame (offscreen unit + subprocess)
- [ ] AC-2.2 transparent/undecorated/always-on-top/skip-taskbar/click-through
- [ ] AC-2.3 redraw cadence on GTK3 (chosen mechanism recorded)
- [ ] AC-3.1 `LuminosHandle` managed state retrievable + dirty-flag wake triggers render
- [ ] `cargo fmt --all -- --check` clean
- [ ] Clippy clean with workspace flags (`-D warnings`, pedantic, no `unwrap_used`/`expect_used`)
- [ ] No `unwrap()`/`expect()` in production paths
- [ ] Chosen redraw-cadence mechanism recorded in epic Shared Context (Discovered Constraints)
- [ ] RISK-001 status note for the Phase-0 gate docs task (HIGH_LEVEL_PLAN Deviations)

**Completion Notes:**
>

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| --- | --- | --- |
