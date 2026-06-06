# Epic E04: Tauri Control Panel & Settings Persistence

**Status:** DONE (2026-06-05)
**Roadmap Ref:** [tech-strategy/09-implementation-roadmap.md Section 4.4](../tech-strategy/09-implementation-roadmap.md)
**Phase:** Phase 0 -- Foundation (Months 1-3)
**Started:** 2026-06-04
**Completed:** 2026-06-05
**Hard Dependencies:** E1 (workspace, core types, CI) -- DONE
**Soft Dependencies:** E3 (input pipeline, ArcSwap state) -- DONE [roadmap §3.2 soft-dep]. E2 (renderer) -- DONE [additive: E04 reuses the E2 `Renderer`; beyond the roadmap's stated soft-dep set, not a conflict]
**Primary Docs:** [05 -- Control Panel](../tech-strategy/05-control-panel.md) Sections 1-6; [01 -- System Architecture](../tech-strategy/01-system-architecture.md) Sections 3.3, 4.6-4.7, 5.4, 6.5, 9.4; [08 -- Build and Distribution](../tech-strategy/08-build-and-distribution.md) Section 5; [06 -- Cross-Cutting Concerns](../tech-strategy/06-cross-cutting-concerns.md) Section 3; [07 -- Testing Strategy](../tech-strategy/07-testing-strategy.md) Sections 3.2, 11

---

## Overview

E04 builds the Tauri 2.x control panel: a webview window with a React UI that drives the Rust magnification engine through typed IPC. It is the **first epic to produce a running Luminos application** -- prior epics (E1-E3) produced standalone, individually-tested modules (renderer, screen capture, input pipeline, lock-free state) but **no event loop wires them into a live, on-screen magnifier**. E04 stands up that unified runtime, opens the control panel alongside a live full-screen magnification overlay, exposes the Phase 0 IPC commands, and persists user settings to `config.toml`.

**User-perceivable value:** A user launches Luminos, sees their screen magnified, opens the control panel, drags a zoom slider and watches the magnification change in real time, switches magnification mode, reads the current frame time, minimizes to the system tray, and -- on next launch -- finds their settings preserved. This is the minimum viable, daily-dogfoodable magnifier that closes Phase 0.

> **Scope note (settings persistence pull-forward):** [05 -- Control Panel](../tech-strategy/05-control-panel.md) Section 1.3 assigns settings persistence to Phase 1. The roadmap (Section 4.4) deliberately pulls it into Phase 0 so dogfooders need not reconfigure after every restart. E04 honors the roadmap.

## Success Criteria

Copied from roadmap Section 4.4 (verified 2026-06-05 — see story-007 SUBTASKS T008 AC matrix for the full per-criterion test refs + tiers):

- [x] Control panel opens, hydrates from engine state, and renders without errors — 001 `app_boots_two_windows_and_exits_clean` (CI-Xvfb) + 006 `HydrationGate`/store Vitest (Node)
- [x] Zoom slider round-trips through IPC: UI → Rust → `ArcSwap` → render thread → next frame — 003 `live_zoom_change_reflected_next_frame` + 005 `set_zoom_level_*` + 006 slider Vitest + 007 E2E `D2` (CI-only). *Live present-on-screen = HW/manual (DC-10).*
- [x] Settings file written to `~/.config/luminos/config.toml` on save — 004 `config::manager::*` (atomic save) + 005 `save_settings_delegates_to_config`
- [x] Settings file read and applied on application startup — 004 `config_load_*` + `seeded_app_state`
- [x] TypeScript bindings match Rust command signatures (CI generation check) — 005 `bindings_export_smoke` + the CI `--export-bindings` diff gate
- [x] All frontend components pass `axe-core` with zero violations — 006 (~67 UI Vitest, 0 violations)
- [~] `tauri-driver` IPC integration tests — 007 `e2e/tests/ipc.e2e.ts` D2/D3/D4 in the `test-e2e` job. **Verification state (honest):** the suite is **authored + wired into the `test-e2e` CI job + `tsc`-typechecked locally; the first green CI run is still pending.** It cannot run on dev boxes (no `WebKitWebDriver`/`tauri-driver`), so the live driver assertions are confirmed only once CI executes the job. Epic remains DONE; this single criterion's CI-green run is the one outstanding verification item.

Additional epic-level acceptance (from deliverables D1, D6):

- [x] Tauri webview window opens alongside the magnification overlay (D1) — 001 `app_boots_two_windows_and_exits_clean`, `overlay_surface_is_created_from_owned_window`
- [x] System tray icon appears; minimize-to-tray works, degrading gracefully where no StatusNotifierItem host is present (D6) — 007 `tray_absent_host_degrades` (the AC-load-bearing degrade test, CI-Xvfb), `tray_init_reaches_definite_outcome_without_panic`, `minimize_to_tray_hides_window_keeps_running`. *Tray-icon VISIBLE-on-screen + icon-left-click restore = HW/manual (needs a real SNI host + visual check). On the dev box with a real D-Bus session the `tray=ready` + `tray_stashed=true` positive path WAS observed.*

---

## Story Breakdown

### Progress Summary

| # | Story | Status | Depends On | Notes |
|---|-------|--------|------------|-------|
| 001 | App Shell, Single Event Loop & wgpu Overlay Surface | DONE (2026-06-05) | --- | **RISK-001 RETIRED.** Single tao/Tauri loop; control-panel + overlay windows open; redraw cadence via a `run_on_main_thread`-marshaled ~60 Hz heartbeat (NOT bare `MainEventsCleared`); wgpu `Surface<'static>` from the owned overlay-window clone (`surface_created` proven); `LuminosHandle` + `AppNotifier`; graceful SIGTERM shutdown via `sigaction`. 23 tests (16 unit + 7 subprocess). See Shared Context → DC-9/DC-10/DC-11 + the 001 seam. |
| 002 | Overlay WindowManager (winit→tao) & Self-Capture | DONE (2026-06-05) | 001 | **winit REMOVED from `luminos-platform`** (`cargo tree` clean). `X11WindowManager` is now an x11rb struct binding the tao overlay by XID (no window creation, no event loop); geometry/visibility/`_NET_WM_STATE_ABOVE` via raw X11. XID bridge in `luminos-app::overlay_bridge` (rwh, not gdk) wired at `Ready`; manager stored on `LuminosHandle`; `overlay_window_id()` surfaced for story-003 self-capture. `raw_*_handle()`→`None` (AD-3). Lens/Docked → `Ok+warn` (deferred E05). See Shared Context → **the 002 seam**. |
| 003 | Live Full-Screen Magnification Integration | DONE (2026-06-05) | 002 | ScreenCapture + Renderer + InputProcessingTask wired into the loop. `CaptureDriver` (owns `XcbCapture`+`TrackingEngine`) drives per-frame capture; `OverlayGpu` hosts the E2 `Renderer`; input pipeline spawned at `Ready` over the same `ArcSwap`; `FrameTimingSummary` published to `LuminosHandle.frame_timings` for story 005. 44 app tests (30 unit + 14 subprocess). See Shared Context → **the 003 seam**. **DC-10 reality:** live magnify *present* + P99>0 need a surface-compatible GPU adapter (absent under headless Xvfb / CI software GL) → unobservable in CI; covered by offscreen shader unit tests; live assertion deferred to story 007 / real GPU. |
| 004 | ConfigManager & Settings Persistence | DONE (2026-06-04) | 001 | `config.toml` load/save in `~/.config/luminos/`, atomic write, startup seed. Pure Rust; runs parallel with 002/003. **T007 app-wiring handed off to 001 (001 not yet run) — see Shared Context → Integration Points → Story 004 seam.** |
| 005 | IPC Command Layer & tauri-specta Bindings | DONE (2026-06-05) | 001, 004 | 7 Phase-0 commands → `StateManager`/`ConfigManager` (+ 2 NEW `StateManager` methods); `zoom_changed`/`mode_changed` events emitted from the render loop on a delta; generated `bindings.ts` (swapped the 006 placeholder) — **70/70 story-006 Vitest tests pass against it (cross-language contract proven)**; capability extended to `core:default`+`core:event:default` (shell:allow-open deferred). 20 new Rust tests. See Shared Context → **the 005 seam**. |
| 006 | Frontend Control Panel UI | DONE — Node-only (2026-06-04) | 005 | pnpm/Vite/React/Zustand/Zod scaffolded; `App`→`ToastProvider`→`HydrationGate`→`Shell`→`MagnificationPage` with slider/mode-selector/frame-timing; hydration + zoom/mode event subscriptions; 70 Vitest tests, **0 axe violations**, build→`ui/dist` (83 kB gz). **Deferred:** generated-`bindings.ts` swap (005) + tauri-driver E2E (007). IPC contract 005 must honor recorded in Shared Context → Integration Points. |
| 007 | System Tray & tauri-driver CI E2E | DONE (2026-06-05) | 003, 005, 006 | **System tray (D6)** with Show/Hide + Quit menu + **minimize-to-tray** (`.on_window_event` `CloseRequested` on `control-panel` only; overlay never hidden); **graceful degrade** via `$DBUS_SESSION_BUS_ADDRESS` heuristic + Ok-on-`build()`-error → `tray=degraded` + panel stays visible + no panic (the AC-load-bearing automated test). TrayIcon stashed on a Linux-gated `LuminosHandle.tray`. **tauri-driver CI E2E** (`e2e/` WDIO9+Mocha, Rust `tauri-driver` v2.0.6 — NOT @crabnebula npm) asserting D2/D3/D4 engine state via `get_current_settings` round-trip; new `test-e2e` CI job (8th active job, mirrored to CLAUDE.md §9). 7 new tests (4 tray unit + 3 tray subprocess). **No IPC added → `bindings.ts` frozen.** See Shared Context → **DC-15 (the 007 seam)**. **Tier honesty:** tray-icon-visible + non-zero-P99 + live-present are HW/manual (DC-10/DC-13); E2E run is CI-only. |

**Total Stories:** 7 | **Done:** 7 (001-007) | **In Progress:** 0 | **Blocked:** 0 | **Epic: DONE (2026-06-05)**

**Parallelization:** After 001 completes, story 002 (overlay backend, `luminos-platform` + `luminos-app`) and story 004 (`luminos-core::config`, pure Rust) may proceed concurrently on disjoint files. 003 waits on 002 (needs the controllable, self-capture-safe overlay). 005 waits on 004 (its `save_settings`/`reset_settings` call `ConfigManager`). 006 waits on 005 (`bindings.ts`). 007 integrates everything.

### Deliverable Traceability

| Deliverable (roadmap §4.4) | Story / Stories | Verifying story |
|----------------------------|-----------------|-----------------|
| D1 -- webview window opens alongside overlay | 001 (both windows open) | 001, 007 |
| D2 -- zoom slider changes magnification real-time | 005 (`set_zoom_level`), 003 (engine applies), 006 (slider UI) | 007 (`tauri-driver`) |
| D3 -- mode selector switches mode | 005 (`set_magnification_mode`), 006 (`MagnificationModeSelector`) | 007 (`tauri-driver`) |
| D4 -- frame timing readout shows P99 | 003 (`FrameTimings` populated), 005 (`get_frame_timings`), 006 (`FrameTimingDisplay`) | 007 (`tauri-driver`) |
| D5 -- settings persist + reload | 004 (`ConfigManager`) | 004 (write/read unit test) |
| D6 -- system tray + minimize-to-tray | 007 | 007 (degrade + minimize CI-Xvfb; icon-visible HW/manual) |
| D7 -- `tauri-specta` valid TS bindings | 005 (`Builder` export) | 005 (CI build) |
| D8 -- components pass `axe-core` | 006 | 006 (Vitest + axe-core) |

| Success Criterion (roadmap §4.4) | Story / Stories |
|----------------------------------|-----------------|
| Control panel opens, hydrates, renders without errors | 001 (window), 006 (hydration) |
| Zoom slider round-trips UI→Rust→ArcSwap→render→frame | 003 + 005 + 006 (verified 007) |
| Settings file written on save | 004, 005 (`save_settings`) |
| Settings read + applied on startup | 004 (seed `AppState`) |
| TS bindings match Rust signatures (CI gen check) | 005 |
| All components pass `axe-core` zero violations | 006 |
| `tauri-driver` IPC integration tests pass in CI | 007 |

### Story Descriptions

#### 001 -- App Shell, Single Event Loop & wgpu Overlay Surface
**Scope:** Replace the empty `luminos-app/src/main.rs` with a single-process Tauri 2.x application running **one** tao/Tauri event loop (`tauri::Builder::build()?.run(|app, RunEvent| …)`) that hosts both the control-panel webview window and a native, transparent, always-on-top, click-through **overlay window** (opened via `WebviewWindowBuilder` in `setup`). Establish a redraw cadence (render inside `RunEvent::MainEventsCleared` gated on a shared `Arc<AtomicBool>` dirty flag — Tauri's `run` exposes **no** `ControlFlow`/`Poll`/`RedrawRequested` and `WebviewWindow` has **no** `request_redraw()`; see AD-1/AD-2) and render a clear-color wgpu frame into a `wgpu::Surface` built from an **owned clone** of the overlay window (the surface is `'static`). Introduce `LuminosHandle` managed state holding the real `Arc<ArcSwap<AppState>>`. Provide the tao/Tauri-backed `EventNotifier` impl (`AppNotifier`; sets the shared dirty flag — no `request_redraw`, which Tauri windows lack). Implement graceful shutdown.
**Key Deliverables:**
- `luminos-app` runs as a Tauri app; control-panel window + overlay window both open (D1)
- Validated per-frame redraw cadence on tao's GTK3 backend (dirty-flag-gated `MainEventsCleared`, with a ~60 Hz timer-thread fallback per AD-1)
- wgpu surface created from the overlay window's rwh-0.6 handle; clear frame renders under picom
- Overlay opens transparent + undecorated + always-on-top + click-through (`set_ignore_cursor_events`)
- `LuminosHandle` registered via `.manage(...)`; tao `EventNotifier` impl (`AppNotifier`) wakes the loop by setting the shared `Arc<AtomicBool>` dirty flag (AD-2)
- Graceful shutdown (`RequestExit`); **RISK-001 retired** via this spike
**Estimated Effort:** L (11-14 subtasks)
**Notes:** Highest-risk story. Validated approach in **Shared Context → AD-1/AD-2**, sourced from the RISK-001 research. The spike must explicitly confirm **three** things on X11 under Xvfb/picom before broader wiring: (1) a stable per-frame redraw cadence on the GTK3 backend, (2) a wgpu clear frame into the overlay surface, (3) transparency + click-through. This story opens the overlay window directly in `setup`; formalizing overlay control behind the `WindowManager` trait is story 002.

#### 002 -- Overlay WindowManager (winit→tao) & Self-Capture
**Scope:** Reimplement the `WindowManager` overlay backend over the tao/Tauri overlay window, replacing the winit-based `X11WindowManager` (which uses `EventLoop`/`with_override_redirect`/`WindowAttributesExtX11`). Satisfy the existing `WindowManager` trait surface (`create_overlay`, `set_overlay_bounds`, `set_overlay_mode`, `set_always_on_top`, `set_visible`, `raw_window_handle`) against the Tauri window. Re-validate the RISK-002 self-capture mitigation (overlay must not appear in its own captures) under the tao/GTK3 window, with a documented fallback if unmap/remap proves unstable (e.g. `_NET_WM_STATE`/input-shape exclusion, or root-region capture).
**Key Deliverables:**
- `WindowManager` overlay backend implemented over the tao/Tauri window; trait surface unchanged
- Overlay positioning/bounds/mode/visibility controllable through the trait
- `raw_window_handle()` ownership resolved for Tauri windows (cloned handle vs borrowed `&dyn HasWindowHandle` — see AD-3 / design)
- RISK-002 self-capture validated under GTK3; mitigation + fallback documented
**Estimated Effort:** M/L (9-12 subtasks)
**Notes:** The `WindowManager` trait shields the rest of the platform layer (AD-3). Coordinate with story 001's directly-opened overlay window: 002 wraps/controls that same window through the trait. Watch tao #7369 (stray label on transparent undecorated windows).

#### 003 -- Live Full-Screen Magnification Integration
**Scope:** Drive the existing engine modules from the E04 loop so the full-screen magnifier renders live. On each redraw: read latest `AppState` from `ArcSwap` (lock-free), call `ScreenCapture::capture_frame`, feed `Renderer::render_frame` against the overlay surface, present. Spawn the `InputProcessingTask` so cursor movement updates the viewport (via `TrackingEngine`) and hotkeys mutate state; the tao `EventNotifier` (from 001) wakes the loop. Collect `FrameTimings`.
**Key Deliverables:**
- Live full-screen magnification on X11 at the existing zoom range (1.5x-20x)
- Cursor tracking moves the magnified viewport smoothly (reuses E3 `TrackingEngine`)
- Phase-0 hotkeys (zoom in/out, toggle, reset) drive state and redraw
- `FrameTimings`/`FrameTimingSummary` populated and reachable for `get_frame_timings` (story 005)
**Estimated Effort:** L (10-14 subtasks)
**Notes:** Reuses E2 `Renderer` and E3 `InputProcessingTask`/`TrackingEngine`/`HotkeyMatcher`/`StateManager` **as-is**; do not reimplement them. New logic is only the per-frame drive loop and capture→render wiring inside `luminos-app`, on top of the controllable overlay from story 002.

#### 004 -- ConfigManager & Settings Persistence
**Scope:** Implement `ConfigManager` in `luminos-core::config`: load/save the existing `AppSettings` to `~/.config/luminos/config.toml` (XDG base-dir resolution), with atomic write (temp file + rename), default-on-missing, and tolerant handling of malformed/partial files (fall back to defaults, log, do not panic). Seed the initial `AppState` from the loaded settings at startup.
**Key Deliverables:**
- `ConfigManager` with `load()`/`save()`/`reset()` and resolved config path
- TOML (de)serialization of `AppSettings` (serde already derived); round-trip preserved
- Atomic write; corrupt-file recovery; XDG path resolution (`$XDG_CONFIG_HOME` → `~/.config`)
- Startup hook seeds `AppState.settings` from disk (D5)
**Estimated Effort:** M (7-10 subtasks)
**Notes:** Pure Rust, no Tauri dependency -- fully unit-testable with `tempfile`. `AppSettings` and all sub-structs already exist in `luminos-core::config::schema`; this story adds the I/O layer only.

#### 005 -- IPC Command Layer & tauri-specta Bindings
**Scope:** Implement the seven Phase-0 `#[tauri::command] #[specta::specta]` commands in `luminos-app`, wired to the **real** `StateManager` and `ConfigManager` via `LuminosHandle`: `get_current_settings`, `set_zoom_level`, `set_magnification_mode`, `toggle_magnification`, `get_frame_timings`, `save_settings`, `reset_settings`. Define `ZoomChangedEvent`/`ModeChangedEvent` (`#[tauri_specta::Event]`) emitted when hotkeys change state so the panel stays in sync. Configure the `tauri-specta` `Builder` to export `ui/src/ipc/bindings.ts` in debug builds. Author the minimal Tauri capability file.
**Key Deliverables:**
- 7 Phase-0 commands; each validates input (clamp/enum), writes via `StateManager`, wakes the loop via `EventNotifier`, returns serde types
- `ZoomChangedEvent(f32)` / `ModeChangedEvent(MagnificationMode)` emitted from the hotkey path
- `Builder` exports `ui/src/ipc/bindings.ts` (debug-only, `#[cfg(debug_assertions)]`) -- D7
- Capability file granting only `core:default`, `core:event:default`, `shell:allow-open`
**Estimated Effort:** M/L (10-13 subtasks)
**AC grouping (≤5):** (1) read commands return correct serde data; (2) mutation commands validate/clamp + write `StateManager` + wake loop; (3) persistence commands delegate to `ConfigManager`; (4) hotkey path emits `zoom_changed`/`mode_changed`; (5) `Builder` exports `bindings.ts` matching signatures + capability grants only the minimal set. Use parametrized tests across the command set.
**Notes:** **Reconcile to real code, not doc-05's illustrative snippets** -- see Shared Context → Integration Points. Zoom lives at `AppState.settings.magnification.zoom_level`; use `StateManager::update_zoom_level()` etc., not a flat `AppState { zoom_level }`. `LuminosEvent` has only `StateChanged`/`RequestExit` -- the engine wake stays `StateChanged`; the panel-facing `zoom_changed`/`mode_changed` are separate `tauri-specta` events. **`get_frame_timings` returns the `FrameTimingSummary` type (already in `luminos-gpu`); live values are populated only by story 003's render loop -- return a zeroed/last-known summary when the loop is not yet running; the end-to-end assertion lives in story 007.**

#### 006 -- Frontend Control Panel UI
**Scope:** Scaffold `ui/` (pnpm, Vite 6, React 19, TypeScript 5, Zustand, Zod, Vitest, RTL, axe-core). Build the Phase-0 component tree: `App` → `HydrationGate` → `Shell` (sidebar nav + outlet) → `MagnificationPage` containing `ZoomLevelSlider` and `MagnificationModeSelector`, plus a debug-only `FrameTimingDisplay`. Implement `useSettingsStore` (Zustand), hydration-on-startup (`get_current_settings`), and IPC event subscriptions (`zoom_changed`, `mode_changed`) using the generated `bindings.ts`. Optimistic updates with revert-on-error and accessible toasts.
**Key Deliverables:**
- `ui/` project building via `pnpm build` → `ui/dist` (wired to `tauri.conf.json` `frontendDist`)
- `useSettingsStore` + hydration gate; zoom slider and mode selector round-trip through IPC (D2, D3)
- Frame-timing readout in debug builds (D4, UI side)
- Vitest + RTL component tests; **zero** `axe-core` violations (D8); `eslint-plugin-jsx-a11y` clean
**Estimated Effort:** L (12-15 subtasks)
**AC grouping (≤5):** (1) project scaffolds + builds to `ui/dist` + consumes `bindings.ts`; (2) `HydrationGate` + `useSettingsStore` hydrate from `get_current_settings`, defaults-on-error; (3) `ZoomLevelSlider` round-trips with optimistic-update/revert; (4) `MagnificationModeSelector` round-trips + `FrameTimingDisplay` shows P99 (debug); (5) `zoom_changed`/`mode_changed` subscriptions update the store + zero `axe-core` violations across components.
**Notes:** TypeScript-only tooling per CLAUDE.md (no Python). Prefer Zod schemas + inferred types. `bindings.ts` is generated (story 005) and consumed here; treat it as the source of truth for command/event signatures.

#### 007 -- System Tray & tauri-driver CI E2E
**Scope:** Add a system-tray icon with minimize-to-tray / restore behavior, designed to **degrade gracefully** (log + keep window visible) where no StatusNotifierItem host runs (D6). Stand up the `tauri-driver` + WebKitWebDriver CI job and author the IPC integration tests verifying D2/D3/D4 end-to-end (zoom slider → engine state; mode switch; frame-timing readout). Finalize epic-level acceptance and the AC coverage matrix.
**Key Deliverables:**
- Tray icon + minimize-to-tray + restore; graceful no-SNI fallback (D6)
- `.github/workflows/ci.yml` gains a `test-e2e` job (Xvfb + picom + WebKitWebDriver + tauri-driver)
- IPC integration tests for D2/D3/D4 passing in CI
- Epic acceptance: all E04 success criteria verified; AC coverage matrix produced
**Estimated Effort:** M (8-11 subtasks)
**Notes:** `tauri-driver` is Linux+Windows only (no macOS WKWebView driver) -- fine for Linux-first CI. WebKitWebDriver ships with webkit2gtk; ensure the CI image installs it. Keep the tray's platform-specific glue behind the existing platform boundary where practical.

---

## Shared Context

### Architecture Decisions

- **AD-1 (RISK-001 resolution): Single tao/Tauri event loop; no separate winit `EventLoop`.**
  The overlay is a **second Tauri/tao window** (transparent, undecorated, always-on-top, click-through), not a winit-owned window. The wgpu surface is built from that window's `raw-window-handle` (rwh 0.6 -- shared by winit, wgpu, tao, and Tauri at the pinned versions; **the surface is `'static`, so it must be created from an _owned_ Tauri `WebviewWindow` clone**, not a borrowed reference). Rendering is driven from inside Tauri's `App::run(|app, RunEvent| …)` callback. **Render cadence (corrected -- Tauri's `run` API, not winit's):** Tauri's `run` callback exposes **no `ControlFlow`/`Poll`** and no `RedrawRequested`; `WebviewWindow` has **no `request_redraw()`**. The loop is observed via `RunEvent` variants (`Ready`, `MainEventsCleared`, `WindowEvent{ Resized, CloseRequested }`, `ExitRequested`). Drive rendering by **rendering inside `RunEvent::MainEventsCleared` gated on a shared dirty flag**, and -- because tao's GTK3 backend may not emit `MainEventsCleared` at a steady ~60 Hz (tao [#635](https://github.com/tauri-apps/tao/issues/635)) -- the story-001 spike empirically picks between (a) render-on-`MainEventsCleared` and (b) a ~60 Hz timer thread that flips the dirty flag. **Rationale for one loop:** macOS `NSApplication` permits exactly one main event loop, initialized once on the main thread; a second winit `EventLoop` panics ("winit requires control over the principal class", [winit #3772](https://github.com/rust-windowing/winit/issues/3772)). A single portable architecture is therefore mandatory. The well-documented wgpu+webview *single-window* flicker failure ([tauri #9220](https://github.com/tauri-apps/tauri/issues/9220), closed not-planned) is avoided by our **two-window** design. Confidence: HIGH on "one loop / no separate winit"; MEDIUM-HIGH on overlay-window mechanics (render cadence + transparency + click-through + self-capture under tao's GTK3 backend) -- hence the story-001 spike. Full research: `.claude/agent-memory/technical-research-analyst/risk001-dual-event-loop-research.md`.

- **AD-2: `EventNotifier` is the seam for the loop swap -- wake via a shared dirty flag.** `luminos-core::pipeline::EventNotifier` already abstracts the **wake-on-demand** mechanism (it currently impls for `winit::EventLoopProxy<LuminosEvent>`). E04 adds a tao/Tauri-backed impl (`AppNotifier`) holding a shared `Arc<std::sync::atomic::AtomicBool>` "render-dirty" flag: `notify_state_changed()` simply sets the flag (`Relaxed`/`Release`). The flag is `Send + Sync`, so input/IPC threads set it with **no main-thread marshaling and no `request_redraw`** (which doesn't exist on Tauri windows). The `App::run` callback reads-and-clears the flag each `MainEventsCleared` and renders when set; the steady cadence (AD-1) keeps `MainEventsCleared` arriving (or a timer flips the flag). Callers (input/hotkey/IPC threads) are unchanged -- they only see `EventNotifier`.

- **AD-3: `WindowManager` trait shields the overlay rewrite.** Only the X11 overlay backend (`luminos-platform::linux_x11::window::X11WindowManager`, currently winit + `with_override_redirect` + `WindowAttributesExtX11`) is reimplemented over a tao/Tauri window. The `WindowManager` trait surface is preserved; the rest of the platform layer is untouched. **Open design point (story 002):** the trait's `raw_window_handle()` currently returns a borrowed `&dyn HasWindowHandle`; Tauri windows are cloned handles, not borrowed values, so the backend may need to return an owned handle wrapper (or the trait signature may need a small adjustment) -- resolve in story 002's DESIGN.md.

- **AD-4: IPC writes go through `StateManager`, then wake the loop.** Commands clamp/validate input, call the existing `StateManager` RCU methods (`update_zoom_level`, `toggle_magnification`, ...) on `Arc<ArcSwap<AppState>>`, then `EventNotifier::notify_state_changed()` to request a redraw. The render path reads `ArcSwap` lock-free every frame. (doc-01 §6.5, doc-05 §4.1, §6.5.)

- **AD-5: Two distinct event channels -- do not conflate.** (a) **Engine wake:** `LuminosEvent::StateChanged` over the tao loop, to trigger a redraw. (b) **Panel sync:** `tauri-specta` events `zoom_changed`/`mode_changed` emitted to the webview when a hotkey (not the UI) changes state, so the Zustand store stays in sync. The UI→engine direction uses commands; the engine→UI direction uses these events.

### Key Type Definitions

New types introduced by E04 (full signatures finalized in each story's DESIGN.md):

```rust
// luminos-app — managed Tauri state (reconciled to real engine types)
pub(crate) struct LuminosHandle {
    pub app_state: std::sync::Arc<arc_swap::ArcSwap<luminos_core::AppState>>,
    pub config:    std::sync::Arc<std::sync::Mutex<Option<luminos_core::config::ConfigManager>>>,
    pub notifier:  AppNotifier,            // tao/Tauri-backed EventNotifier holding the dirty flag (AD-2)
    pub app:       tauri::AppHandle,
    // tts_tx: TtsSender — deferred to Phase 2 (Epic 11); omit or stub in Phase 0
}
// AppNotifier { dirty: Arc<AtomicBool>, ... } — notify_state_changed() sets `dirty`.
// `config` is Option because story 001 lands only a minimal `ConfigManager` stub
// (empty struct) so LuminosHandle compiles; story 004 fills in the real I/O and
// the app sets Some(ConfigManager::load()?) at startup.
// std::sync::Mutex (not parking_lot — not a workspace dep); config access is brief and off the render path.

// luminos-core::config — persistence layer (story 004, IMPLEMENTED 2026-06-04).
// Re-exported at crate root: luminos_core::{ConfigManager, ConfigError, seed_initial_state}.
pub struct ConfigManager { /* resolved path + cached AppSettings (Debug + Clone) */ }
impl ConfigManager {
    pub fn load() -> Result<Self, ConfigError>;          // default-on-missing, recover-on-corrupt
    pub fn config_path() -> Result<std::path::PathBuf, ConfigError>;       // XDG via `directories`
    pub fn path(&self) -> &std::path::Path;              // resolved config.toml path (#[must_use])
    pub fn settings(&self) -> &luminos_core::config::AppSettings;          // (#[must_use])
    pub fn save(&mut self, settings: &AppSettings) -> Result<(), ConfigError>; // atomic; updates cache
    pub fn reset(&mut self) -> Result<AppSettings, ConfigError>;           // defaults persisted + returned
    pub fn seeded_app_state(&self) -> luminos_core::AppState;              // AppState{settings, ..default}
}
// Startup seam (FR-7) — story 001 calls this in `setup`:
pub fn seed_initial_state() -> Result<(luminos_core::AppState, ConfigManager), ConfigError>;
// On Ok: wrap state in Arc<ArcSwap<AppState>> for StateManager::new(..); set LuminosHandle.config = Some(manager).
// On Err(NoConfigDir): log::warn! and fall back to AppState::default() + LuminosHandle.config = None.

// ConfigError (thiserror): Io { path: String, source: std::io::Error },
//                          Serialize(#[from] toml::ser::Error),
//                          NoConfigDir.  (NO Deserialize variant — load() recovers corrupt files.)
//
// On-disk format: TOML at $XDG_CONFIG_HOME/luminos/config.toml (else ~/.config/luminos/config.toml).
// File wrapper `ConfigFile { schema_version: u32 = 1, settings: AppSettings }` (schema_version is a
// FILE concern, NOT a field of AppSettings — AppSettings is unchanged). Corrupt files → defaults +
// best-effort backup to config.toml.bak + warn!. Atomic save = temp(same dir)+fsync+rename, 0600 on Unix.
// NEW workspace deps: directories =6.0.0 (config-dir resolution), tempfile =3.27.0 (dev-dep, tests).

// luminos-app::events — tauri-specta events (story 005). Deserialize REQUIRED for Event::listen.
#[derive(Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
pub struct ZoomChangedEvent(pub f32);
#[derive(Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
pub struct ModeChangedEvent(pub luminos_types::MagnificationMode);
```

### Integration Points

Reconciliation map -- **the docs' illustrative snippets diverge from the real E1-E3 code; follow the real code:**

| Concept | doc-05 illustrative | **Real code (authoritative)** | Path |
|---|---|---|---|
| Zoom field | `AppState { zoom_level }` | `AppState.settings.magnification.zoom_level` | `luminos-core/src/state.rs` |
| State write | `app_state.rcu(...)` inline | `StateManager::update_zoom_level()/toggle_magnification()/reset_zoom()` (clamps to [1.5,20]) | `luminos-core/src/state_manager.rs` |
| Loop wake | `EventLoopProxy<LuminosEvent>` + `LuminosEvent::ZoomChanged(f32)` | `EventNotifier::notify_state_changed()` → `LuminosEvent::StateChanged` (only `StateChanged`/`RequestExit` exist) | `luminos-core/src/{event,pipeline}.rs` |
| Settings type | new `AppSettings` Zod/serde | `AppSettings` already defined (serde-derived) | `luminos-core/src/config/schema.rs` |
| Renderer | -- | `Renderer::new(device,queue,format,w,h,method)`, `render_frame(&surface,&CaptureFrame,is_bgra)`, `frame_timings()` | `luminos-gpu/src/renderer.rs` |
| Input | -- | `InputProcessingTask::spawn(receiver, state_manager, hotkey_matcher, notifier)` | `luminos-core/src/pipeline.rs` |
| Timings | `FrameTimingSummary{averageMs,p99Ms,minMs,maxMs,targetFps}` | matches `FrameTimingSummary{average_ms,p99_ms,min_ms,max_ms,target_fps}` (serde rename to camelCase for IPC) | `luminos-gpu/src/frame_timings.rs` |

Phase-0 IPC surface (story 005): commands `get_current_settings`, `set_zoom_level`, `set_magnification_mode`, `toggle_magnification`, `get_frame_timings`, `save_settings`, `reset_settings`; events `zoom_changed`, `mode_changed`. Bindings exported to `ui/src/ipc/bindings.ts`. Capability: `core:default`, `core:event:default`, `shell:allow-open` only (doc-06 §3.4).

#### IPC contract assumed by story 006 — story 005 MUST honor this when generating bindings (verified against real Rust code 2026-06-04)

Story 006 hand-authored a placeholder `ui/src/ipc/bindings.ts` and Zod schemas against the EXACT wire format below. When story 005 generates the real `bindings.ts` via tauri-specta, these must match or the frontend's Zod validation will reject live payloads. The frontend depends only on the `ipc/{commands,events}.ts` wrappers, so the generated file is a one-file swap — provided the shapes below hold.

- **Command method names (camelCase) → Tauri command (snake_case) + args:**
  - `getCurrentSettings()` → `get_current_settings`, no args → `Result<AppSettings, string>`
  - `setZoomLevel(level: number)` → `set_zoom_level`, arg **`{ level: f32 }`** → `Result<null, string>`
  - `setMagnificationMode(mode)` → `set_magnification_mode`, arg **`{ mode: MagnificationMode }`** → `Result<null, string>`
  - `toggleMagnification()` → `toggle_magnification`, no args → `Result<bool, string>` (returns new enabled state)
  - `getFrameTimings()` → `get_frame_timings`, no args → `Result<FrameTimingSummary, string>`
  - `saveSettings()` → `save_settings`, no args → `Result<null, string>`
  - `resetSettings()` → `reset_settings`, no args → `Result<AppSettings, string>`
  - **Arg names matter:** the wrappers `invoke(cmd, { level }/{ mode })`. If story 005 names the Rust params differently, either keep these names or the swap requires touching `commands.ts`. Prefer params named `level` and `mode`.
- **`AppSettings` wire format = snake_case keys, PascalCase enum values, `null` for `Option::None`.** `AppSettings` + all sub-structs in `luminos-core::config::schema` carry **NO `#[serde(rename_all)]`** — do NOT add a camelCase rename. Keys: `magnification.{zoom_level,mode,tracking_mode,docked_edge,docked_size_percent,lens_width,lens_height,lens_shape,target_fps,present_mode,gpu_preference,interpolation,smooth_scrolling}`, `color_filter.{filter_type,brightness,contrast,color_matrix}`, `cursor.{...}`, `speech.{...}`, `keybindings` (sparse map of PascalCase `HotkeyAction` → `KeyBinding`|null), `start_on_login`, `minimize_to_tray`, `show_panel_on_start`.
- **Enum variants are bare PascalCase strings:** `MagnificationMode` ∈ {`FullScreen`,`Docked`,`Lens`}; `TrackingMode` ∈ {`Cursor`,`Focus`,`TextCaret`}; `ColorFilterType` ∈ {`None`,`Invert`,`SmartInvert`,`Grayscale`,`HighContrast`,`Custom`}; `InterpolationMode` ∈ {`Bilinear`,`Bicubic`}; `PresentMode`,`GpuPreference`,`DockEdge`,`LensShape`,`ModelVariant` likewise.
- **`FrameTimingSummary` is the ONE camelCase type** (per DC-5, story 005 adds `#[serde(rename_all="camelCase")]`): wire keys **`{ averageMs, p99Ms, minMs, maxMs, targetFps }`** (numbers; `targetFps` an integer). The Rust struct is `average_ms`/`p99_ms`/… today with no serde derive — story 005 adds both serde + the rename + `specta::Type`.
- **Events:** `zoom_changed` payload = bare `f32` (new zoom level); `mode_changed` payload = bare `MagnificationMode` string. tauri-specta exposes these as `events.zoomChangedEvent` / `events.modeChangedEvent` with `.listen(cb)` where `cb(e)` reads `e.payload`. If story 005 names the events/derives differently, update the two lines in `events.ts`.
- **Error envelope:** the wrappers assume tauri-specta's **default `Result` error-handling mode** (`{ status:"ok", data } | { status:"error", error }`). If story 005 configures `Throw` mode instead, simplify `bindings.ts`/`commands.ts` accordingly (the wrapper's `unwrap` becomes a passthrough) — still a localized change.

Frontend/build (story 006, doc-08 §5): `ui/` beside `luminos-app`; `tauri.conf.json` → `beforeDevCommand: "pnpm dev"`, `devUrl: http://localhost:1420`, `beforeBuildCommand: "pnpm build"`, `frontendDist: "../ui/dist"`. Node 20/22 LTS, pnpm 9.x, Vite 6.x.

### Discovered Constraints

- **DC-1:** No integrated event loop exists prior to E04 (`luminos-app/src/main.rs` is empty; `X11WindowManager` creates and drops an ephemeral winit `EventLoop`). E04 therefore absorbs the first construction of the unified runtime -- this is in-scope per the roadmap's "dual-window architecture (webview + native overlay)" inclusion and the "→ render thread → next frame" success criterion, even though no prior epic built the loop.
- **DC-2:** tao uses **GTK3** on Linux (winit talks X11/Wayland directly). Override-redirect and direct X11 property control are not first-class GTK concepts; the overlay backend reaches through GDK/X11 where needed. The **self-capture mitigation (RISK-002)** -- unmap/remap the overlay around capture -- must be re-validated under the GTK window (story 002; spike begins in 001). Watch [tao #7369](https://github.com/tauri-apps/tauri/issues/7369) (stray label on transparent undecorated windows).
- **DC-3:** Transparency on X11 requires a running compositor (picom -- already in CI). Click-through via `set_ignore_cursor_events`; tao/Tauri has **no per-region hit-testing** (irrelevant for an always-click-through magnification overlay).
- **DC-4:** `tauri-driver` is Linux + Windows only (no macOS WKWebView driver). Acceptable for Linux-first CI. Package is `@crabnebula/tauri-driver`.
- **DC-5 (specta prerequisite, story 005):** NO engine type implements `specta::Type` today, and `FrameTimingSummary` has neither serde nor specta. `#[specta::specta]` commands require it, so story 005 must add `specta` (pinned) + `#[derive(specta::Type)]` to all IPC-reachable types (`MagnificationMode`, `AppSettings` + sub-structs, `FrameTimingSummary` — plus serde + `rename_all="camelCase"` on the latter). Engine crates also lack crate-root `pub use` re-exports, so add them (`luminos_gpu::{Renderer,FrameTimingSummary,InterpolationMethod}`, `luminos_platform::ScreenCapture`) or use full module paths.
- **DC-6 (self-capture, stories 002/003):** `ScreenCapture::set_excluded_windows(&[u64])` is ALREADY implemented by the shipped `XcbCapture` (unmap/remap). Self-capture = pass the overlay XID to it; do NOT reinvent exclusion. Story 002 exposes `overlay_window_id()`; story 003 calls `set_excluded_windows(&[xid])` on the loop's capture instance.
- **DC-7 (FrameTimings vs Summary):** `FrameTimings` (ring buffer) ≠ `FrameTimingSummary`; call `.summary(target_fps)` to convert. `Renderer` bakes `InterpolationMethod` at `new()` (no runtime switch) → Phase 0 interpolation is fixed at startup. `settings.magnification.interpolation` is `InterpolationMode`; map it to `InterpolationMethod`.
- **DC-8 (capability-file timing across 001/005/007):** the full Tauri capability file (`core:default`, `core:event:default`, `shell:allow-open`) is authored in **story 005**, but story 001 already performs window operations (second-window create, `set_ignore_cursor_events`, transparency) and story 007 drives the tray/window show-hide. Rust-side window/tray calls via `AppHandle`/`WebviewWindow` are **not** gated by webview capabilities (capabilities gate the **webview's** access to `core:*`/plugin commands, not native Rust calls), so 001's spike and 007's tray work do **not** require the 005 capability file. **Action for story 001:** land a **minimal capability stub** (at least `core:default` for the control-panel webview) so the webview loads; story 005 extends it to the full set. **DONE (001):** `crates/luminos-app/capabilities/default.json` grants the `control-panel` window only `core:default`; no window op was blocked. Story 005 EXTENDS this same file (don't recreate it) and adds `"events"` to `tauri.conf.json` `security.capabilities` if it adds a second file.

- **DC-9 (redraw cadence on tao/GTK3 — story 001 RESOLVED): marshal the heartbeat onto the main thread.** AD-1/AD-2 anticipated "render on `MainEventsCleared` gated by a dirty flag, with a ~60 Hz timer fallback." Empirically (001 spike, Xvfb+picom): tao #635 is real AND a bare `run_on_main_thread(|| {})` does NOT reliably provoke `MainEventsCleared` (alternated 60/s vs ~1/s). **The stable mechanism is to marshal the heartbeat *closure itself*** — `AppHandle::run_on_main_thread(move || { dirty.store(true, Release); count.fetch_add(1); log "redraw=N"; })` — from the ~60 Hz timer thread; `run_on_main_thread` runs the closure reliably, yielding a rock-steady ~60 Hz. The GPU present then happens opportunistically in the resulting `MainEventsCleared`, gated on the dirty flag. **Story 003** must drive its real per-frame render through this same marshaled tick (or render directly inside the marshaled main-thread closure once `OverlayGpu` is reachable from it), NOT by relying on raw `MainEventsCleared` cadence. The `AppNotifier` wake (DC-11) feeds the same dirty flag.

- **DC-10 (headless test env — REQUIRED for any subprocess test of the app):** under a headless Xvfb the tao/GTK windows DO NOT realize and software GL does not bind unless the app child is spawned with **`GDK_BACKEND=x11` + `WEBKIT_DISABLE_COMPOSITING_MODE=1` + `WEBKIT_DISABLE_DMABUF_RENDERER=1` + `LIBGL_ALWAYS_SOFTWARE=1`** (`MESA_GL_VERSION_OVERRIDE=4.5` for GL 4.5). Stories 002/003/007 subprocess/E2E tests MUST set these per spawned child (the `tests/common::RunningApp` harness already does). Live GPU **present** fails under Xvfb (EGL "surfaceless platform" → no surface-compatible adapter) — that's a headless-software-GL limitation, not a coexistence failure; cover render logic with offscreen wgpu unit tests (`compatible_surface: None`). Window assertions: use **x11rb `query_tree`** (sees WM-less/override-redirect windows); `xdotool --name` does not, and `xwininfo` is not installed. always-on-top/skip-taskbar are WM-enforced and unobservable under a WM-less Xvfb.

- **DC-11 (the story-001 runtime seam — what 002/003/005 build on):**
  - **Entry point:** `luminos_app::app::run() -> Result<(), AppError>` owns the single `tauri::App::run` loop. `main.rs` is thin (logger + `run()`).
  - **Managed state:** `luminos_app::handle::LuminosHandle { app_state: Arc<ArcSwap<AppState>>, config: Arc<Mutex<Option<ConfigManager>>>, notifier: AppNotifier, app: AppHandle }` is `.manage`d. Stories 005 retrieve it with `app.state::<LuminosHandle>()` inside commands. The REAL `ConfigManager` (story 004) is wired (`Some` on Ok, `None` on `NoConfigDir`).
  - **Wake:** `luminos_app::notifier::AppNotifier` (Clone+Send+Sync) impls `luminos_core::pipeline::EventNotifier`; `notify_state_changed()` sets the shared `Arc<AtomicBool>` dirty flag the loop drains. The existing `EventLoopProxy<LuminosEvent>` blanket impl is UNTOUCHED. Stories 003 (input task) / 005 (commands) hold an `AppNotifier` and call `notify_state_changed()` after a `StateManager` mutation.
  - **Overlay window:** opened in `.setup` with label `"overlay"` (transparent/undecorated/always_on_top/skip_taskbar/focused(false) + `set_ignore_cursor_events(true)`), URL `overlay.html`. The control-panel is label `"control-panel"` (from `tauri.conf.json`). **Story 002** wraps THIS overlay window through the `WindowManager` trait — it must `app_handle.get_webview_window("overlay")` (owned, `'static`, rwh-0.6) rather than open a new one, and reconcile the trait's borrowed `raw_window_handle()` with the owned-`'static` surface model (AD-3).
  - **GPU surface:** `luminos_app::overlay_gpu::OverlayGpu::new(window: tauri::WebviewWindow, w, h)` builds `Surface<'static>` from the owned window. Story 003 replaces `render_clear` with `luminos_gpu::Renderer::render_frame` against the same surface (keep the owned window alive next to the surface).
  - **Shutdown:** SIGTERM/SIGINT handled via `signal::install_termination_handler()` (`sigaction`, NOT sigmask — sigmask breaks GTK) + the cadence thread calling `app.exit(0)`; the loop's `ExitRequested|Exit` arm joins threads + drops GPU once (compare_exchange guard).
  - **New pinned deps:** `pollster=0.4.0`, `libc=0.2.186` (both under the `tauri` feature; libc Linux-only). See PINNED_VERSIONS §1c.

- **DC-12 (the story-002 WindowManager seam — what 003 builds on):**
  - **Reaching the manager:** `app.state::<LuminosHandle>()` → `handle.window_manager: Arc<Mutex<Option<X11WindowManager>>>` (Linux-gated, concrete type — NOT `Box<dyn WindowManager>`, because the self-capture XID accessor is inherent to the backend, not the trait). `lock()` it off the render hot loop and call the `WindowManager` trait methods (`set_overlay_bounds`, `set_overlay_mode`, `set_always_on_top`, `set_visible`). It is bound at `RunEvent::Ready` by `app::init_window_manager` (after `init_overlay_gpu`), so it is `Some` by the time story 003's loop runs.
  - **`overlay_window_id()`:** `handle.overlay_window_id() -> Option<u64>` (Linux) returns the bound overlay XID (also `X11WindowManager::overlay_window_id()` directly). **Story 003 self-capture wiring point:** after constructing the render-loop `XcbCapture`, call `capture.set_excluded_windows(&[xid])` ONCE with this XID (the shipped unmap/remap does the rest — DC-6). The hook is already proven to run without panic (002's `LUMINOS_SELF_CAPTURE_PROBE` path); story 003 just moves the call from the probe into the real loop.
  - **`raw_*_handle()` → `None`:** the manager sources NO wgpu surface (AD-3). Story 003's surface stays `OverlayGpu`'s `Surface<'static>` from the owned overlay `WebviewWindow` (DC-11). Do NOT try to build a surface from the `WindowManager`.
  - **Lens/Docked:** `set_overlay_mode(Lens|Docked)` returns `Ok(()) + warn!` (deferred E05) — callers must not treat that as a hard error. Only `FullScreen` resizes (to the bound `display_bounds`).
  - **Bridge:** XID is extracted in `luminos-app::overlay_bridge` via `raw_window_handle::HasWindowHandle` (rwh-0.6 `Xlib`/`Xcb`), NOT gdk. `luminos-platform` has NO `tauri`/`winit` dep (`cargo tree -p luminos-platform` is clean) — keep it that way.
  - **⚠️ Per-frame-connect smell (record for 003 + a future cleanup):** `XcbCapture::{unmap,remap}_excluded_windows` open a FRESH `x11rb::connect(None)` PER CAPTURED FRAME (`capture.rs:171,203`) using ambient `$DISPLAY`. The story-002 `X11WindowManager` by contrast holds ONE persistent `RustConnection`. For the 60 fps loop, the capture's per-frame connect is a latency/correctness risk (handshake per frame; relies on `$DISPLAY` matching the overlay's display). Story 003 should either (a) reuse the manager's connection for exclusion, or (b) cache one connection in `XcbCapture`. Left as-is for 002; flagged here.
  - **RISK-002 finding (002):** the self-capture exclusion is unmap/remap around each capture (DC-6), so **visible flicker is the documented expected cost under tao/GTK3** (NFR-2) — flicker-free optimization is post-E04. (On this dev box the live frame-grab couldn't be observed because xcap 0.9.4 mis-selects the Wayland backend under headless Xvfb; CI's `xvfb-run`+picom X11 harness is the real check.)

- **DC-13 (the story-003 live-magnification seam — what 005/007 build on):**
  - **Frame-timing read seam (for story-005 `get_frame_timings`):** `LuminosHandle` now carries `frame_timings: Arc<Mutex<luminos_gpu::FrameTimingSummary>>`, initialized zeroed. The render loop calls `handle.set_frame_timings(gpu.frame_timing_summary())` after each presented `MainEventsCleared`. **Story 005's command reads it via `handle.frame_timings()`** (returns a clone; lock is uncontended, off the render path). It returns a zeroed summary until the loop has presented at least one frame. `FrameTimingSummary` fields are `average_ms`/`p99_ms`/`min_ms`/`max_ms`/`target_fps` (snake_case in Rust; story 005 adds `#[serde(rename_all="camelCase")]` + `specta::Type` per DC-5 → wire keys `averageMs`/`p99Ms`/`minMs`/`maxMs`/`targetFps`). **Live-data caveat (DC-10):** `FrameTimings::record` only runs inside a successful `Renderer::render_frame` present, which fails under headless software GL — so P99 stays 0 in CI; the live non-zero assertion is a real-GPU / story-007 concern (already noted in the Cross-Story edge below).
  - **State read/write path (the source of truth):** the render loop reads `AppState` lock-free every frame via `app_state.load()` (the SAME `Arc<ArcSwap<AppState>>` on `LuminosHandle.app_state`). All writes — from the input pipeline (cursor/hotkeys) AND from story-005 IPC commands — go through `StateManager` methods on that same Arc (`update_zoom_level`/`toggle_magnification`/`reset_zoom`/`update_mouse_position`), then `EventNotifier::notify_state_changed()` (the `AppNotifier` dirty flag) to wake the loop. Story 005's commands therefore: `app.state::<LuminosHandle>()` → build `StateManager::new(Arc::clone(&handle.app_state))` → mutate → `handle.notifier.notify_state_changed()`. The loop picks up the change on the next frame (proven live by `live_zoom_change_reflected_next_frame` / `live_hotkeys_drive_state`). `zoom_level` lives at `settings.magnification.zoom_level`, `mode` at `settings.magnification.mode`, `is_active` at the top level (the toggle).
  - **Interpolation is fixed at startup (Phase 0):** `OverlayGpu`/`Renderer` bake the `InterpolationMethod` at `Ready` from `settings.magnification.interpolation` (`luminos_app::capture_driver::interpolation_method_for`). Changing `settings.interpolation` via IPC will NOT re-bake the shader until restart — story-005 commands may persist it but must not promise a live interpolation switch.
  - **Self-capture / exclusion:** the loop's `CaptureDriver` sets `set_excluded_windows(&[overlay_xid])` ONCE at `Ready` (from `handle.overlay_window_id()`). `LUMINOS_NO_EXCLUDE=1` skips it (the per-frame-connect/flicker escape hatch). The per-frame-connect cost is RISK-004 (SUBTASKS B002), deferred to Phase 1.
  - **Test-only env hooks (never in production):** `LUMINOS_FORCE_ACTIVE=1` (seed `is_active=true`), `LUMINOS_LOG_STATE=1` (log `state mouse=... zoom=... active=...` on change), `LUMINOS_NO_EXCLUDE=1` (skip self-capture exclusion), `LUMINOS_FORCE_MINIMIZE_TO_TRAY=1|0` (story 007: force the `minimize_to_tray` seed so the minimize-to-tray subprocess test is deterministic regardless of the host config). Story 007's E2E may reuse these.

- **DC-14 (the story-005 IPC seam — what 007's tauri-driver E2E builds on):**
  - **The 7 commands** (`luminos_app::tauri_commands`, all `#[tauri::command] #[specta::specta]`, `State<'_, LuminosHandle>` LAST param): `get_current_settings() -> AppSettings`, `get_frame_timings() -> FrameTimingSummary`, `set_zoom_level(level: f32) -> ()`, `set_magnification_mode(mode: MagnificationMode) -> ()`, `toggle_magnification() -> bool`, `save_settings() -> ()`, `reset_settings() -> AppSettings`. All return `Result<T, String>`. They are registered via `ipc.invoke_handler()` on the `tauri::Builder` (in `app::run`). **For 007's E2E:** `invoke('get_current_settings')` etc. from the webview, or drive via the generated `commands` object; the wire command names are snake_case, the arg-object keys are `level`/`mode`.
  - **The 2 events** (`luminos_app::events`, `#[tauri_specta::Event]` with MANDATORY `#[tauri_specta(event_name = …)]`): `ZoomChangedEvent(f32)` → wire `zoom_changed`; `ModeChangedEvent(MagnificationMode)` → wire `mode_changed`. Mounted via `ipc.mount_events(app)` in `.setup`. **Emitted from the RENDER LOOP** (`app::emit_state_events` in the `MainEventsCleared` arm) on a `(zoom, mode)` delta — NOT the input thread (it has no `AppHandle`). **For 007's E2E:** subscribe via the generated `events.zoomChanged.listen(cb)` / `events.modeChanged.listen(cb)`; trigger `zoom_changed` with the `ctrl+alt+equal` hotkey (the proven path). **Caveat:** `mode_changed` has no Phase-0 hotkey trigger yet (`CycleMode` is a no-op) — it exists for the contract but won't fire from engine input; an E2E mode-change assertion must originate from the `set_magnification_mode` command's echo (the loop re-emits on the resulting delta).
  - **The 2 NEW `StateManager` methods** (`luminos-core::state_manager`): `set_magnification_mode(MagnificationMode)` and `replace_settings(&AppSettings)` (RCU writes; `replace_settings` preserves transient runtime fields). The other writes reuse E3's `update_zoom_level`/`toggle_magnification`.
  - **The capability** (`crates/luminos-app/capabilities/default.json`): `permissions: ["core:default", "core:event:default"]`, `windows: ["control-panel", "overlay"]`. `core:event:default` is what lets the webview `listen` for the events. **`shell:allow-open` is NOT granted** (deferred — see story-005 SUBTASKS Deviations); if 007 needs it, add `tauri-plugin-shell` + the permission together.
  - **Generated bindings + the contract gate:** `ui/src/ipc/bindings.ts` is now tauri-specta-generated (placeholder swapped). It is regenerated by the `--export-bindings` CLI seam (`cargo run -p luminos-app --features tauri -- --export-bindings`, windowless) and a CI step in the `test-app` job `git diff --exit-code`s it (mirrored into CLAUDE.md §8). The frontend wrappers (`ui/src/ipc/{commands,events}.ts`) consume `commands`/`events` from it; `events.ts` was edited to the generated `events.zoomChanged`/`events.modeChanged` keys, and `commands.ts` defines `Result` locally (the generated file inlines the envelope, exports no named `Result`). `FrameTimingSummary` is camelCase (`averageMs`/…); everything else stays snake_case; floats are `number` (Builder `enable_lossless_floats`). **All 70 story-006 Vitest tests pass against the regenerated file** — the cross-language round-trip is proven.

- **DC-15 (the story-007 tray + E2E seam — epic close-out):**
  - **Tray module:** `luminos_app::tray` (Linux, under the `tauri` feature). `init_tray(app: &tauri::App) -> Result<Option<TrayIcon<Wry>>, AppError>` is called from `app::run`'s `.setup` after `setup_overlay_window`; the returned `TrayIcon` is STASHED on `LuminosHandle.tray` (Linux-gated `Arc<Mutex<Option<TrayIcon<Wry>>>>`) — it is refcounted, so dropping it removes the icon. `toggle_control_panel(app)` is the reliable restore path (menu Show/Hide), NOT icon-click (SNI backends often deliver only menu events).
  - **Graceful degrade (FR-3):** two layers — (1) `$DBUS_SESSION_BUS_ADDRESS` unset/empty ⇒ provably no SNI host ⇒ skip the build + `Ok(None)`; (2) `build()` Err ⇒ `Ok(None)`. NEVER `?`-propagates out of setup. Structured markers `tray=ready` / `tray=degraded` (+ `tray_stashed=N`). Under every degrade path the control panel stays visible; no panic/`unwrap`/`expect`.
  - **Minimize-to-tray:** `tauri::Builder::on_window_event` (NOT `RunEvent::WindowEvent`, which is observation-only). On `CloseRequested` for the `control-panel` window ONLY: read `settings.minimize_to_tray` lock-free → `api.prevent_close()` + `window.hide()`. The overlay is NEVER hidden (hiding it kills magnification). Marker `minimize_to_tray=hidden`.
  - **Quit:** the predefined quit menu item calls `app.exit(0)` → the existing `ExitRequested|Exit` teardown (thread join + GPU drop, compare_exchange guard). FR-1 single-loop invariant intact — no winit `EventLoop`.
  - **Tray adds NO IPC** → `ui/src/ipc/bindings.ts` stays byte-frozen; the `test-app` bindings-diff gate stays green; the capability file (`core:default`+`core:event:default`) is unchanged (native window/tray calls are not webview-capability-gated).
  - **New test-only env hook:** `LUMINOS_FORCE_MINIMIZE_TO_TRAY=1|0` (forces the `minimize_to_tray` seed for the deterministic subprocess test). No new cargo dep (`tray-icon 0.23.1` is transitive via tauri's `tray-icon` feature; `libayatana-appindicator3` is a CI system lib).
  - **E2E (CI-only):** `e2e/` is a WebdriverIO 9 + Mocha (TypeScript via `tsx`) project at the repo root, driven by the **Rust `tauri-driver` v2.0.6** (NOT `@crabnebula/tauri-driver` npm). It drives the control-panel UI (slider/radios) and asserts engine state via the `get_current_settings` round-trip in the webview (`window.__TAURI_INTERNALS__.invoke`). D4 reads `getFrameTimings()` and asserts P99 PRESENCE (0 headless, DC-13). `tauri:options` v2.0.6 has only `application`/`args` (no `env`) — the DC-10 headless-WebKit env is injected into the `tauri-driver` process env. `switchToControlPanel()` handles the two-webview attach ambiguity. The `test-e2e` CI job (8th active job, `needs: [lint]`) installs `webkit2gtk-driver` + `libayatana-appindicator3-dev` + `tauri-driver` and runs under `xvfb-run` + picom. **Not locally runnable** without `WebKitWebDriver` (CI-authored + `tsc`-typechecked).

### Cross-Story Dependencies

- 002 and 004 depend on 001 (loop + overlay window + `LuminosHandle`). 002 ∥ 004 (disjoint files).
- 003 depends on 002 (needs the controllable, self-capture-safe overlay).
- 005 depends on 001 (handle) + 004 (`ConfigManager` for save/reset).
- 006 depends on 005 (`bindings.ts`).
- 007 depends on 003 (live engine to assert against) + 005 (commands) + 006 (UI).
- Runtime-only edge: 005's `get_frame_timings` yields live data only once 003's loop runs (return zeroed/last-known otherwise); the live assertion is in 007. Not a build dependency.

---

## Deviations from Tech Strategy Docs

| Item | Doc as written | E04 deviation | Rationale |
|------|----------------|---------------|-----------|
| Overlay windowing | doc-01 §3.3/§6.5, doc-05 §4.1, **and roadmap §4.4 (`LuminosHandle` lists `EventLoopProxy`)** specify a **winit** overlay window + `EventLoopProxy<LuminosEvent>` | Overlay is a **tao/Tauri** window under a single Tauri loop; wake via tao-backed `EventNotifier` (`AppNotifier`) setting a shared `Arc<AtomicBool>` dirty flag (AD-2). **RISK-001 RETIRED by story 001 (2026-06-05)** — the single-loop two-window model works end-to-end under Xvfb+picom (surface from the owned overlay window, steady ~60 Hz cadence, transparency/click-through, clean SIGTERM exit); the raw-wry+tao fallback was NOT needed. | RISK-001 research: a second winit `EventLoop` is impossible on macOS; one tao loop is the only portable design (AD-1). **Three docs carry the stale winit assumption (doc-01 §3.3/§6.5, doc-05 §4.1, roadmap §4.4). File a Phase-0-gate docs task to update all three + mark RISK-001 mitigated in the risk register now that story 001 has validated the spike.** |
| IPC state shape | doc-05 §4.1 shows flat `AppState { zoom_level }` + inline `.rcu()` | Use nested `AppState.settings.magnification.zoom_level` via `StateManager` methods | doc snippet is illustrative; real E3 code is authoritative (Integration Points table). |
| `LuminosHandle.tts_tx` | doc-05 §4.1 includes `tts_tx: TtsSender` | Omitted/stubbed in Phase 0 | TTS is Phase 2 (Epic 10/11); no TTS coordinator exists yet. |
| `WindowManager::raw_*_handle()` (story 002) | trait returns `Option<&dyn HasWindowHandle>` (winit backend returned `Some` after create, sourcing the wgpu surface) | X11 backend returns `None` always; the surface is sourced by `luminos-app`'s `OverlayGpu` from the owned overlay window (AD-3). Trait SIGNATURE unchanged (FR-6). | The overlay window is created by `luminos-app`, not the platform layer; duplicating handle ownership in the trait would force a `tauri` dep. **Flag a Phase-0-gate trait cleanup:** drop `raw_*_handle()` from `WindowManager` (now dead weight) or formalize surface-sourcing in the app layer. |

> Per CLAUDE.md governance rule 8, these deviations are recorded with rationale. The doc updates themselves are **out of scope for E04 implementation** but are logged here and should be raised as a docs task at the Phase 0 gate.

---

## Blockers & Issues Log

| ID | Date | Story | Description | Resolution | Status |
|----|------|-------|-------------|------------|--------|
| B001 | --- | --- | --- | --- | --- |

## Retrospective Notes

**Epic E04 closed 2026-06-05** — the first running Luminos application: a single tao/Tauri event loop hosting a transparent click-through wgpu overlay + a React control panel, typed IPC, settings persistence, a system tray, and a CI E2E job. 7 stories, all DONE.

**What went well:**
- **RISK-001 retired by story 001** — the single-loop two-window model (overlay = a second tao/Tauri window, surface from an owned `'static` clone) worked end-to-end under Xvfb+picom; the raw-wry+tao fallback was never needed.
- **The `IMPLEMENTATION_NOTES` source-verified briefings** (per story) caught several stale DESIGN assumptions BEFORE coding (winit→tao, flat vs nested `AppState`, `@crabnebula` vs Rust `tauri-driver`, the slider's dynamic `useId()`), avoiding rework.
- **The reconciliation-to-real-code rule** (Integration Points table) kept stories honest against the actual E1-E3 surface, not doc-05's illustrative snippets.
- **The bindings-diff CI gate** (005) made the cross-language contract self-enforcing; story 007 added a tray with zero IPC and the gate confirmed `bindings.ts` stayed frozen.

**What was hard / honest limits:**
- **Headless GPU reality (DC-10):** live magnification PRESENT and non-zero P99 cannot be observed under headless software GL (EGL surfaceless → no presentable surface). Render *logic* is covered by offscreen wgpu/shader unit tests; the live present is real-GPU/dogfood only. This is a genuine CI blind spot, recorded honestly (NOT papered over as "CI-verified").
- **Tray icon visibility** needs a real SNI host + visual inspection — only the **degrade path** + menu-driven show/hide + minimize-to-tray are automated. (On the dev box, which has a real D-Bus + SNI host, the `tray=ready` positive path WAS observed — stronger than anticipated.)
- **E2E is CI-only:** `WebKitWebDriver`/`tauri-driver`/`xvfb-run` are absent on dev boxes; the suite is authored + `tsc`-typechecked locally and runs live only in CI. First-run flake-soak is a CI-runtime verification item.
- **tauri-driver `tauri:options` env gap:** v2.0.6 supports only `application`/`args` — the headless env had to be threaded through the `tauri-driver` process env. Source-verifying the crate caught this before it would have silently dropped the env in CI.

**Close-out carry-forward (the lead / cross-cutting task #9 owns these — RECORDED here, not fixed in E04):**
1. **RISK-001 → Retired** in the risk register (`specs/tech-strategy/10-risk-register.md`) — story 001 validated the single-loop spike end-to-end.
2. **Tech-strategy doc updates (winit→tao):** doc-01 §3.3/§6.5, doc-05 §4.1, roadmap §4.4 (the `LuminosHandle` `EventLoopProxy` mention) all carry the stale winit overlay assumption + nested-`AppState` shape. File the Phase-0-gate docs task.
3. **`WindowManager::raw_*_handle()` cleanup:** the X11 backend always returns `None` (AD-3); drop `raw_*_handle()` from the trait (dead weight) or formalize surface-sourcing in the app layer.
4. **Code-polish backlog (recorded across stories):** filed as a dedicated, clearly-labeled Phase-1 backlog at [`PHASE1_BACKLOG.md`](./PHASE1_BACKLOG.md) — every item marked Phase-1, non-blocking. Covers: 002 `WindowError::PropertyFailed` routing; 003 typed `SurfaceErrorKind` discriminant; 005 pure `compute_emit_delta()` unit test; `WindowManager::raw_*_handle()` trait cleanup (always `None`); 007 `CONTROL_PANEL_LABEL` dedup + the dead `MENU_ID_QUIT` arm; the per-frame `x11rb::connect` in `XcbCapture::{unmap,remap}_excluded_windows` (now **RISK-039**, DC-12 — cache one connection for the 60 fps loop); and AD-5 origin-tagging for event emission. The 003 shutdown-detach concern is now **RISK-040**; the 003 BGRA-prose fix and the `deny.toml` prune were applied at this close-out.
5. **E04/007 specifics to carry forward:** confirm `test-e2e` is non-flaky on the first real CI runs; dogfood the tray icon visibility + icon-left-click restore on a real desktop; revisit live-present + non-zero-P99 on real GPU hardware; the `MENU_ID_QUIT` arm in `tray::handle_menu_event` is currently dead (the predefined quit item handles quit) — keep or wire a custom quit item.
6. **DC-4 driver-package correction:** the `@crabnebula/tauri-driver` references in older docs should be globally replaced with the Rust `tauri-driver` crate (recorded as a 007 Deviation).
