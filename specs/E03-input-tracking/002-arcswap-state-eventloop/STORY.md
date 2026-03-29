# Story E03/002: ArcSwap State Management & EventLoopProxy

**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Depends On:** None

---

## Problem Statement

The E02 render loop reads magnification settings directly and uses a static viewport position. For E03's interactive magnification to work, the input monitoring thread (Story 001) and hotkey handler (Story 004) must be able to write state changes (mouse position, zoom level, magnification toggle) that the render thread picks up on the very next frame -- without locking, blocking, or introducing latency on the 60fps render hot path.

This story establishes the `ArcSwap<AppState>` state distribution infrastructure and `EventLoopProxy` integration that connect the input/control threads to the winit render loop. It introduces the `StateManager` convenience wrapper for thread-safe state reads and writes, and the `LuminosEvent` custom event type for cross-thread render loop wake-up. These are foundational building blocks that all subsequent epics (E04 control panel IPC, E07 focus tracking, E11 TTS triggers) build upon.

## User Scenarios

### US-1: Lock-Free Render Thread State Access

As a render thread, I need to read the current application state (zoom level, magnification mode, mouse position, active state) every frame without any lock acquisition so that I maintain consistent 60fps rendering with no frame drops caused by lock contention.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given a `StateManager` wrapping `Arc<ArcSwap<AppState>>`, when `load()` is called from the render thread, then a `Guard` reference to the current `AppState` is returned without acquiring any mutex or RwLock.
- **AC-1.2:** Given a `StateManager`, when `load()` is called in a tight loop (benchmark), then the average latency per call is less than 100 nanoseconds.
- **AC-1.3:** Given a writer thread calling `update_mouse_position()` and a reader thread calling `load()` concurrently, when the writer updates the mouse position, then the reader observes the new position on the next `load()` call (within one ArcSwap generation).

### US-2: State Mutation from Input Thread

As the input processing thread, I need to write mouse position and hotkey-triggered state changes to the shared application state so that the render thread can read the updated values on the next frame.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1:** Given a `StateManager`, when `update_mouse_position(ScreenPoint { x: 500, y: 300 })` is called, then the `AppState.mouse_position` field is updated to `ScreenPoint { x: 500, y: 300 }` via `rcu()` (read-copy-update).
- **AC-2.2:** Given a `StateManager`, when `update_zoom_level(5.0)` is called, then `AppState.settings.magnification.zoom_level` is updated to `5.0` via `rcu()`.
- **AC-2.3:** Given a `StateManager` with `AppState.is_active == true`, when `toggle_magnification()` is called, then `AppState.is_active` becomes `false`.
- **AC-2.4:** Given a `StateManager`, when `reset_zoom()` is called, then `AppState.settings.magnification.zoom_level` is set to the default value (2.0).
- **AC-2.5:** Given two threads calling `update_mouse_position()` and `update_zoom_level()` concurrently, when both updates complete, then neither update is lost -- `rcu()` retries on contention.

### US-3: EventLoopProxy Render Loop Wake-Up

As the input processing thread, I need to wake the winit render loop immediately after a state change so that the updated magnification is rendered on the next frame without waiting for the next vsync tick or timer event.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1:** Given a `LuminosEvent` enum, when the variant `StateChanged` is sent via `EventLoopProxy::send_event()`, then the winit event loop receives a `Event::UserEvent(LuminosEvent::StateChanged)` and can trigger a `request_redraw()`.
- **AC-3.2:** Given a `LuminosEvent` enum, when the variant `RequestExit` is sent via `EventLoopProxy`, then the winit event loop receives it and can initiate graceful shutdown.
- **AC-3.3:** Given an `EventLoopProxy<LuminosEvent>`, when `clone()` is called, then the clone can send events from a different thread and both the original and clone deliver events to the same event loop.

### US-4: AppState Extension

As a developer, I need the `AppState` struct to carry the current mouse position so that the tracking engine (Story 003) can read it from ArcSwap each frame to compute the viewport source region.

**Priority:** P0
**Acceptance Criteria:**

- **AC-4.1:** Given the `AppState` struct, when a new `AppState` is constructed with `Default::default()`, then `mouse_position` is `ScreenPoint { x: 0, y: 0 }`.
- **AC-4.2:** Given an existing `AppState` with `mouse_position: ScreenPoint { x: 100, y: 200 }`, when it is cloned, then the clone has `mouse_position: ScreenPoint { x: 100, y: 200 }`.

## Functional Requirements

- **FR-1:** Create a `StateManager` struct in `luminos-core::state_manager` that wraps `Arc<ArcSwap<AppState>>` and provides typed read/write methods for all state fields used in E03.
- **FR-2:** `StateManager::load()` returns an `arc_swap::Guard` reference to the current `AppState` for lock-free, zero-copy reads.
- **FR-3:** `StateManager::update_mouse_position()`, `update_zoom_level()`, `toggle_magnification()`, and `reset_zoom()` mutate state via `rcu()` (read-copy-update pattern), ensuring no updates are lost under contention.
- **FR-4:** Define a `LuminosEvent` enum in `luminos-core` with variants `StateChanged` and `RequestExit`, suitable as the generic parameter for `winit::event_loop::EventLoopProxy<LuminosEvent>`.
- **FR-5:** Extend `AppState` with a `mouse_position: ScreenPoint` field, defaulting to `(0, 0)`.
- **FR-6:** `StateManager` constructor accepts an `Arc<ArcSwap<AppState>>` (not constructing it internally), allowing the caller to share the same ArcSwap instance with both the StateManager and the render thread.

## Non-Functional Requirements

- **NFR-1:** `ArcSwap::load()` average latency must be less than 100 nanoseconds per call (SC4, D4). Verified by benchmark test.
- **NFR-2:** State writes via `rcu()` must be wait-free for the reader -- a slow writer must never block the render thread's `load()` call.
- **NFR-3:** The `StateManager` must be `Send + Sync` for safe sharing between threads via `Arc<StateManager>`.
- **NFR-4:** `LuminosEvent` must be `Send` to satisfy `EventLoopProxy<T: Send>` requirements.

## Out of Scope

- Tauri IPC integration for state changes (E04)
- Settings persistence to `config.toml` (E04)
- Focus tracking state (E07)
- TTS status state management (E11)
- Profile management and multiple state snapshots (E09)
- Configurable default zoom level (E07) -- this story uses the hardcoded default from `AppSettings::default()`
- The actual winit event loop setup -- this story provides the `LuminosEvent` type and verifies it works with `EventLoopProxy`; Story 005 wires the event loop

## Open Questions

- [x] Should `StateManager` own the `EventLoopProxy` and call `send_event()` after each mutation? **Decision: No.** The `StateManager` is a state container only. The caller (input processing task, hotkey dispatcher) is responsible for calling `event_loop_proxy.send_event()` after state mutations. This keeps `StateManager` decoupled from winit and testable without a live event loop.
- [x] Should `mouse_position` live in `AppState` or as a separate `AtomicI32` pair? **Decision: In `AppState`.** The ArcSwap read is <100ns, well within the 16.67ms frame budget. The atomic pair optimization is premature and adds complexity. See HIGH_LEVEL_PLAN.md Architecture Decisions.
- [x] Where should `LuminosEvent` be defined? **Decision: In `luminos-core`.** It couples application state to the winit event loop but must NOT be in `luminos-platform` (which has no winit dependency). `luminos-core` is the correct location as it already owns `AppState`.
