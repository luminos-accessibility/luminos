# Epic E04: Tauri Control Panel & Settings Persistence

**Status:** IN PROGRESS
**Roadmap Ref:** [tech-strategy/09-implementation-roadmap.md Section 4.4](../tech-strategy/09-implementation-roadmap.md)
**Phase:** Phase 0 -- Foundation (Months 1-3)
**Started:** 2026-06-04
**Completed:** ---
**Hard Dependencies:** E1 (workspace, core types, CI) -- DONE
**Soft Dependencies:** E3 (input pipeline, ArcSwap state) -- DONE [roadmap §3.2 soft-dep]. E2 (renderer) -- DONE [additive: E04 reuses the E2 `Renderer`; beyond the roadmap's stated soft-dep set, not a conflict]
**Primary Docs:** [05 -- Control Panel](../tech-strategy/05-control-panel.md) Sections 1-6; [01 -- System Architecture](../tech-strategy/01-system-architecture.md) Sections 3.3, 4.6-4.7, 5.4, 6.5, 9.4; [08 -- Build and Distribution](../tech-strategy/08-build-and-distribution.md) Section 5; [06 -- Cross-Cutting Concerns](../tech-strategy/06-cross-cutting-concerns.md) Section 3; [07 -- Testing Strategy](../tech-strategy/07-testing-strategy.md) Sections 3.2, 11

---

## Overview

E04 builds the Tauri 2.x control panel: a webview window with a React UI that drives the Rust magnification engine through typed IPC. It is the **first epic to produce a running Luminos application** -- prior epics (E1-E3) produced standalone, individually-tested modules (renderer, screen capture, input pipeline, lock-free state) but **no event loop wires them into a live, on-screen magnifier**. E04 stands up that unified runtime, opens the control panel alongside a live full-screen magnification overlay, exposes the Phase 0 IPC commands, and persists user settings to `config.toml`.

**User-perceivable value:** A user launches Luminos, sees their screen magnified, opens the control panel, drags a zoom slider and watches the magnification change in real time, switches magnification mode, reads the current frame time, minimizes to the system tray, and -- on next launch -- finds their settings preserved. This is the minimum viable, daily-dogfoodable magnifier that closes Phase 0.

> **Scope note (settings persistence pull-forward):** [05 -- Control Panel](../tech-strategy/05-control-panel.md) Section 1.3 assigns settings persistence to Phase 1. The roadmap (Section 4.4) deliberately pulls it into Phase 0 so dogfooders need not reconfigure after every restart. E04 honors the roadmap.

## Success Criteria

Copied from roadmap Section 4.4:

- [ ] Control panel opens, hydrates from engine state, and renders without errors
- [ ] Zoom slider round-trips through IPC: UI → Rust → `ArcSwap` → render thread → next frame
- [ ] Settings file written to `~/.config/luminos/config.toml` on save
- [ ] Settings file read and applied on application startup
- [ ] TypeScript bindings match Rust command signatures (CI generation check)
- [ ] All frontend components pass `axe-core` with zero violations
- [ ] `tauri-driver` IPC integration tests pass in CI

Additional epic-level acceptance (from deliverables D1, D6):

- [ ] Tauri webview window opens alongside the magnification overlay (D1)
- [ ] System tray icon appears; minimize-to-tray works, degrading gracefully where no StatusNotifierItem host is present (D6)

---

## Story Breakdown

### Progress Summary

| # | Story | Status | Depends On | Notes |
|---|-------|--------|------------|-------|
| 001 | App Shell, Single Event Loop & wgpu Overlay Surface | NOT STARTED | --- | RISK-001 linchpin. Single tao/Tauri loop; control-panel + overlay windows open; redraw cadence (dirty-flag-gated `MainEventsCleared`, AD-2); wgpu surface from an owned overlay-window clone renders a clear frame; `LuminosHandle` + tao `EventNotifier`; graceful shutdown. Retires RISK-001 via spike. |
| 002 | Overlay WindowManager (winit→tao) & Self-Capture | NOT STARTED | 001 | Reimplement the `WindowManager` overlay backend over the tao/Tauri window (replace winit `X11WindowManager`); transparency/always-on-top/click-through; positioning/mode/visibility; re-validate RISK-002 self-capture under GTK3. |
| 003 | Live Full-Screen Magnification Integration | NOT STARTED | 002 | Wire ScreenCapture + Renderer + InputProcessingTask into the loop. Live magnifier + cursor tracking + frame timings. |
| 004 | ConfigManager & Settings Persistence | NOT STARTED | 001 | `config.toml` load/save in `~/.config/luminos/`, atomic write, startup seed. Pure Rust; runs parallel with 002/003. |
| 005 | IPC Command Layer & tauri-specta Bindings | NOT STARTED | 001, 004 | 7 Phase-0 commands → `StateManager`/`ConfigManager`; `ZoomChanged`/`ModeChanged` events; `bindings.ts` export; capability file. AC-grouped to ≤5 (see notes). |
| 006 | Frontend Control Panel UI | NOT STARTED | 005 | pnpm/Vite/React/Zustand/Zod; `App`→`HydrationGate`→`Shell`→`MagnificationPage`; hydration + event subscriptions; Vitest + RTL + axe-core. AC-grouped to ≤5. |
| 007 | System Tray & tauri-driver CI E2E | NOT STARTED | 003, 005, 006 | Tray + minimize-to-tray (graceful degrade); `tauri-driver` CI job; IPC integration tests; epic acceptance. |

**Total Stories:** 7 | **Done:** 0 | **In Progress:** 0 | **Blocked:** 0

**Parallelization:** After 001 completes, story 002 (overlay backend, `luminos-platform` + `luminos-app`) and story 004 (`luminos-core::config`, pure Rust) may proceed concurrently on disjoint files. 003 waits on 002 (needs the controllable, self-capture-safe overlay). 005 waits on 004 (its `save_settings`/`reset_settings` call `ConfigManager`). 006 waits on 005 (`bindings.ts`). 007 integrates everything.

### Deliverable Traceability

| Deliverable (roadmap §4.4) | Story / Stories | Verifying story |
|----------------------------|-----------------|-----------------|
| D1 -- webview window opens alongside overlay | 001 (both windows open) | 001, 007 |
| D2 -- zoom slider changes magnification real-time | 005 (`set_zoom_level`), 003 (engine applies), 006 (slider UI) | 007 (`tauri-driver`) |
| D3 -- mode selector switches mode | 005 (`set_magnification_mode`), 006 (`MagnificationModeSelector`) | 007 (`tauri-driver`) |
| D4 -- frame timing readout shows P99 | 003 (`FrameTimings` populated), 005 (`get_frame_timings`), 006 (`FrameTimingDisplay`) | 007 (`tauri-driver`) |
| D5 -- settings persist + reload | 004 (`ConfigManager`) | 004 (write/read unit test) |
| D6 -- system tray + minimize-to-tray | 007 | 007 (manual) |
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

// luminos-core::config — new persistence layer (story 004)
pub struct ConfigManager { /* resolved path + cached AppSettings */ }
impl ConfigManager {
    pub fn load() -> Result<Self, ConfigError>;          // default-on-missing
    pub fn settings(&self) -> &luminos_core::config::AppSettings;
    pub fn save(&mut self, settings: &AppSettings) -> Result<(), ConfigError>; // atomic; updates cached settings
    pub fn reset(&mut self) -> Result<AppSettings, ConfigError>;
    pub fn config_path() -> Result<std::path::PathBuf, ConfigError>;       // XDG
}

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

Frontend/build (story 006, doc-08 §5): `ui/` beside `luminos-app`; `tauri.conf.json` → `beforeDevCommand: "pnpm dev"`, `devUrl: http://localhost:1420`, `beforeBuildCommand: "pnpm build"`, `frontendDist: "../ui/dist"`. Node 20/22 LTS, pnpm 9.x, Vite 6.x.

### Discovered Constraints

- **DC-1:** No integrated event loop exists prior to E04 (`luminos-app/src/main.rs` is empty; `X11WindowManager` creates and drops an ephemeral winit `EventLoop`). E04 therefore absorbs the first construction of the unified runtime -- this is in-scope per the roadmap's "dual-window architecture (webview + native overlay)" inclusion and the "→ render thread → next frame" success criterion, even though no prior epic built the loop.
- **DC-2:** tao uses **GTK3** on Linux (winit talks X11/Wayland directly). Override-redirect and direct X11 property control are not first-class GTK concepts; the overlay backend reaches through GDK/X11 where needed. The **self-capture mitigation (RISK-002)** -- unmap/remap the overlay around capture -- must be re-validated under the GTK window (story 002; spike begins in 001). Watch [tao #7369](https://github.com/tauri-apps/tauri/issues/7369) (stray label on transparent undecorated windows).
- **DC-3:** Transparency on X11 requires a running compositor (picom -- already in CI). Click-through via `set_ignore_cursor_events`; tao/Tauri has **no per-region hit-testing** (irrelevant for an always-click-through magnification overlay).
- **DC-4:** `tauri-driver` is Linux + Windows only (no macOS WKWebView driver). Acceptable for Linux-first CI. Package is `@crabnebula/tauri-driver`.
- **DC-5 (specta prerequisite, story 005):** NO engine type implements `specta::Type` today, and `FrameTimingSummary` has neither serde nor specta. `#[specta::specta]` commands require it, so story 005 must add `specta` (pinned) + `#[derive(specta::Type)]` to all IPC-reachable types (`MagnificationMode`, `AppSettings` + sub-structs, `FrameTimingSummary` — plus serde + `rename_all="camelCase"` on the latter). Engine crates also lack crate-root `pub use` re-exports, so add them (`luminos_gpu::{Renderer,FrameTimingSummary,InterpolationMethod}`, `luminos_platform::ScreenCapture`) or use full module paths.
- **DC-6 (self-capture, stories 002/003):** `ScreenCapture::set_excluded_windows(&[u64])` is ALREADY implemented by the shipped `XcbCapture` (unmap/remap). Self-capture = pass the overlay XID to it; do NOT reinvent exclusion. Story 002 exposes `overlay_window_id()`; story 003 calls `set_excluded_windows(&[xid])` on the loop's capture instance.
- **DC-7 (FrameTimings vs Summary):** `FrameTimings` (ring buffer) ≠ `FrameTimingSummary`; call `.summary(target_fps)` to convert. `Renderer` bakes `InterpolationMethod` at `new()` (no runtime switch) → Phase 0 interpolation is fixed at startup. `settings.magnification.interpolation` is `InterpolationMode`; map it to `InterpolationMethod`.
- **DC-8 (capability-file timing across 001/005/007):** the full Tauri capability file (`core:default`, `core:event:default`, `shell:allow-open`) is authored in **story 005**, but story 001 already performs window operations (second-window create, `set_ignore_cursor_events`, transparency) and story 007 drives the tray/window show-hide. Rust-side window/tray calls via `AppHandle`/`WebviewWindow` are **not** gated by webview capabilities (capabilities gate the **webview's** access to `core:*`/plugin commands, not native Rust calls), so 001's spike and 007's tray work do **not** require the 005 capability file. **Action for story 001:** land a **minimal capability stub** (at least `core:default` for the control-panel webview) so the webview loads; story 005 extends it to the full set. Confirm during the 001 spike that no window op it performs is blocked; if any is, fold the needed permission into the 001 stub rather than waiting on 005.

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
| Overlay windowing | doc-01 §3.3/§6.5, doc-05 §4.1, **and roadmap §4.4 (`LuminosHandle` lists `EventLoopProxy`)** specify a **winit** overlay window + `EventLoopProxy<LuminosEvent>` | Overlay is a **tao/Tauri** window under a single Tauri loop; wake via tao-backed `EventNotifier` (`AppNotifier`) setting a shared `Arc<AtomicBool>` dirty flag (AD-2) | RISK-001 research: a second winit `EventLoop` is impossible on macOS; one tao loop is the only portable design (AD-1). **Three docs carry the stale winit assumption (doc-01 §3.3/§6.5, doc-05 §4.1, roadmap §4.4). File a Phase-0-gate docs task to update all three + RISK-001 status after story 001 validates the spike** (see gate note below). |
| IPC state shape | doc-05 §4.1 shows flat `AppState { zoom_level }` + inline `.rcu()` | Use nested `AppState.settings.magnification.zoom_level` via `StateManager` methods | doc snippet is illustrative; real E3 code is authoritative (Integration Points table). |
| `LuminosHandle.tts_tx` | doc-05 §4.1 includes `tts_tx: TtsSender` | Omitted/stubbed in Phase 0 | TTS is Phase 2 (Epic 10/11); no TTS coordinator exists yet. |

> Per CLAUDE.md governance rule 8, these deviations are recorded with rationale. The doc updates themselves are **out of scope for E04 implementation** but are logged here and should be raised as a docs task at the Phase 0 gate.

---

## Blockers & Issues Log

| ID | Date | Story | Description | Resolution | Status |
|----|------|-------|-------------|------------|--------|
| B001 | --- | --- | --- | --- | --- |

## Retrospective Notes

[Filled in when the epic is DONE.]
