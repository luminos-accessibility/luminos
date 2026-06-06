# Story E04/001: App Shell, Single Event Loop & wgpu Overlay Surface

**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** APPROVED (approved-as-authoritative per E04 execution, 2026-06-04)
**Depends On:** None (hard-dep E01 DONE; reuses E02 `luminos-gpu`, E03 `luminos-core` state — both DONE)

---

## Problem Statement

Luminos has, after epics E1-E3, a set of individually-tested engine modules — a GPU renderer, an X11 screen-capture backend, an input pipeline, and lock-free shared state — but **nothing wires them into a running program**. `luminos-app/src/main.rs` is an empty `fn main() {}`, and the only window-creation code (`X11WindowManager`) spins up an ephemeral winit `EventLoop` and immediately drops it. There is no process that opens a window, owns an event loop, or presents a frame.

This story builds that missing foundation: a single-process Tauri 2.x application that runs **one** event loop (tao/Tauri's — see RISK-001), opens both the control-panel webview window and a transparent, always-on-top, click-through native **overlay window**, and proves the rendering path by drawing a wgpu clear-color frame into a surface created from the overlay window's handle. It is the **highest-risk story in the epic** because the in-process coexistence of a webview event loop and a GPU overlay has no widely-adopted reference implementation; the first 2-3 subtasks are an explicit spike to retire RISK-001 before broader wiring.

Everything else in E04 — live magnification (003), IPC (005), the control-panel UI (006) — sits on top of the runtime this story establishes.

## User Scenarios

> **AC count = 5** (governance rule 9). Boot+shutdown are one lifecycle AC; managed-state + wake are one wiring AC.

### US-1: Application launches with both windows and shuts down cleanly
As a low-vision user, I want to launch Luminos and see both its control panel and a magnification overlay window appear (and have it exit cleanly when I close it), so that I have a stable running application.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1 (lifecycle):** Given a Linux X11 session with a running compositor (picom), when the `luminos-app` binary is launched and subsequently sent a close/exit request, then exactly one OS process starts with both a control-panel webview window and a separate full-screen overlay window open (no panic, no second event loop created); and on the exit request both windows close, background threads join, and the process exits with status 0 without hanging.

### US-2: Overlay renders a GPU frame and is non-intrusive
As a user, I want the overlay to be a transparent, click-through, always-on-top surface that the GPU can draw into, so that magnified content can later be composited over my desktop without blocking my interaction with the apps beneath it.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1 (surface + frame):** Given the overlay window is open under a compositor, when the app initializes wgpu from the overlay window's **owned** `WebviewWindow` handle (rwh 0.6, yielding a `Surface<'static>`), then a `wgpu::Surface` is successfully created and a clear-color frame is presented at the surface resolution, confirmed by an automated headless GPU test (Mesa llvmpipe) for the surface path and a subprocess smoke test for the live overlay.
- **AC-2.2 (attributes):** Given the overlay window, when it is created, then it is transparent (alpha-capable), undecorated, always-on-top, skips the taskbar, and is click-through (`set_ignore_cursor_events(true)`) — verified by inspecting window state via `xprop`/`xwininfo`.
- **AC-2.3 (cadence):** Given the app renders inside Tauri's `App::run` callback on `RunEvent::MainEventsCleared` gated by a shared dirty flag (mechanism empirically selected by the story spike — `MainEventsCleared` or a ~60 Hz timer flipping the flag; NO winit `Poll`/`request_redraw`, which Tauri does not expose), when the app runs, then a redraw counter (emitted as a `redraw=N` log heartbeat) advances by ≥ a threshold over a fixed wall-clock window, asserted by a subprocess test.

### US-3: Shared state and wake mechanism are wired
As a developer building later stories, I want the running app to expose the real lock-free `AppState` and a working wake mechanism, so that IPC commands and the input pipeline can mutate state and trigger redraws.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1 (managed state + wake):** Given the running app, when it starts, then a `LuminosHandle` holding the real `Arc<ArcSwap<AppState>>` is registered as Tauri managed state and is retrievable from a Tauri command context; and given a background thread holding the tao-backed `AppNotifier`, when it calls `notify_state_changed()`, then the shared dirty flag is set (no main-thread marshaling, no `request_redraw`) and the next render observes the latest `ArcSwap` state (a redraw occurs beyond the idle cadence).

## Functional Requirements

- **FR-1:** `luminos-app` MUST run as a Tauri 2.x application driven by a single tao/Tauri event loop via `tauri::Builder::…build()?.run(|app, RunEvent| …)`. No separate `winit::EventLoop` is instantiated in the shipping binary. *(Traced by AC-1.1)*
- **FR-2:** The app MUST open a control-panel webview window and a separate native overlay window during `setup`. *(Traced by AC-1.1)*
- **FR-3:** The overlay window MUST be transparent, undecorated, always-on-top, taskbar-skipping, and click-through. *(Traced by AC-2.2)*
- **FR-4:** The app MUST create a `wgpu::Surface<'static>` from an **owned** overlay `WebviewWindow` (rwh-0.6) and present a clear-color frame each render. *(Traced by AC-2.1)*
- **FR-5:** The app MUST establish a steady redraw cadence by rendering inside the `App::run` callback gated on a shared dirty flag (mechanism validated on tao GTK3), without relying on winit-style `Poll`/`request_redraw`. *(Traced by AC-2.3)*
- **FR-6:** The app MUST register a `LuminosHandle { app_state, config, notifier, app }` as Tauri managed state, where `app_state` is the real `Arc<ArcSwap<AppState>>` from `luminos-core`. *(Traced by AC-3.1)*
- **FR-7:** The app MUST provide a tao/Tauri-backed `EventNotifier` impl (`AppNotifier`) whose `notify_state_changed()` sets the shared `Arc<AtomicBool>` dirty flag. *(Traced by AC-3.1)*
- **FR-8:** The app MUST shut down gracefully on `RunEvent::ExitRequested`/window-close: stop threads, drop GPU resources, exit cleanly. *(Traced by AC-1.1)*

## Non-Functional Requirements

- **NFR-1:** Startup to first presented overlay frame MUST be < 2 s on the reference dev machine (Phase 0 target, doc-06).
- **NFR-2:** The redraw path MUST read `AppState` lock-free via `ArcSwap` (no mutex on the render path); per doc-01 §6.3 the render work budget is < 8 ms (clear-frame stub trivially satisfies this; the budget is asserted for real in story 003).
- **NFR-3:** Transparency requires a running X11 compositor; the app MUST detect absence of a compositor and log a clear warning rather than panicking (overlay may render opaque in that case).
- **NFR-4:** No `unwrap()`/`expect()` in production paths (CLAUDE.md); GPU/window init failures surface as typed errors with actionable log messages.
- **NFR-5:** Story 001 stubs a minimal `core:default` capability (HLP DC-8); the full capability set is extended in story 005. Story 001 MUST NOT broaden permissions beyond the `core:default` webview stub plus whatever Tauri defaults are needed to open windows.

## Out of Scope

- Screen capture, magnification shaders, cursor tracking, hotkeys → **story 003** (this story renders only a clear color).
- The `WindowManager` trait re-implementation over tao and self-capture (RISK-002) → **story 002** (this story opens the overlay directly in `setup`; it does not route overlay control through the platform trait).
- IPC commands, `tauri-specta` bindings, events, and the full capability set (story 001 lands only the `core:default` stub) → **story 005**.
- `ConfigManager`/persistence (the `config` field may be a placeholder/`None`-equivalent until story 004) → **story 004**.
- React/frontend UI (the control-panel window may load a placeholder page) → **story 006**.
- System tray → **story 007**.
- macOS/Windows/Wayland specifics (Linux X11 only this story); document the macOS `NSPanel` follow-up but do not implement it.

## Open Questions

- [x] Should the overlay be a `WebviewWindow` (empty page) or a plain native window? — **Resolved:** Use a Tauri window that yields a valid rwh-0.6 handle for the wgpu surface; an attached-but-empty webview is acceptable. Do NOT composite wgpu *under* a visible webview in the same window (tauri #9220 flicker). Finalize the exact builder in DESIGN.
- [x] How is the redraw cadence driven on tao GTK3? — **Resolved (corrected):** Tauri's `App::run` exposes no `ControlFlow`/`Poll`/`RedrawRequested` and `WebviewWindow` has no `request_redraw()`. Render inside the `run` callback on `RunEvent::MainEventsCleared` gated by a shared dirty flag; the spike (AC-2.3) picks between rendering directly on `MainEventsCleared` vs a ~60 Hz timer thread that flips the flag (tao #635 means `MainEventsCleared` cadence on GTK3 is not guaranteed).
- [x] `LuminosHandle.config` references `ConfigManager`, which story 004 builds — how does 001 compile? — **Resolved (superseded at implementation time):** story 004 SHIPPED before 001 was executed, so NO stub was created. Story 001 uses the **real** `luminos_core::seed_initial_state() -> Result<(AppState, ConfigManager), ConfigError>` startup seam: on `Ok` it stores `Some(manager)` in `LuminosHandle.config` (typed `Arc<Mutex<Option<ConfigManager>>>`); on `Err` (e.g. `NoConfigDir`) it `log::warn!`s and falls back to `AppState::default()` + `None`. `AppError` gained `From<ConfigError>`. (The original "minimal empty stub" plan is obsolete — see SUBTASKS Deviations.)
- [x] Does the spike block the rest of the epic? — **Resolved:** Yes by design; story 001 is the critical-path foundation. If the two-window in-process model fails the spike, escalate to the raw wry+tao fallback (HIGH_LEVEL_PLAN AD-1 / research option d) before proceeding.
