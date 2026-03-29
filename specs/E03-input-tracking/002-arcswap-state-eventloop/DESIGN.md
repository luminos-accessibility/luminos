# Design: Story E03/002 -- ArcSwap State Management & EventLoopProxy

**Story:** [STORY.md](STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** spec-writer-1
**Risk Refs:** RISK-001 (dual event loop coexistence -- EventLoopProxy is the validated bridge pattern)

---

## Overview

Establish the `ArcSwap<AppState>` state distribution infrastructure and `EventLoopProxy<LuminosEvent>` integration that connect input/control threads to the winit render loop. The `StateManager` struct wraps `Arc<ArcSwap<AppState>>` with typed methods for reading and mutating application state. The `LuminosEvent` enum provides the custom event type for waking the winit event loop from other threads.

This is a foundational story: every subsequent epic that mutates render-visible state (E04 control panel, E07 focus tracking, E11 TTS) uses the `StateManager` and `LuminosEvent` patterns established here.

## Architecture

### Component Diagram

```
luminos-core/src/
  lib.rs                    [Modified] Add `pub mod state_manager; pub mod event;`
  state.rs                  [Modified] Add `mouse_position: ScreenPoint` to AppState
  state_manager.rs          [New]      StateManager wrapping Arc<ArcSwap<AppState>>
  event.rs                  [New]      LuminosEvent enum
  config/
    schema.rs               [Existing] Unchanged
  error.rs                  [Existing] Unchanged
```

```
  Input Thread                  StateManager                    Render Thread
  ============                  ============                    =============
       |                             |                               |
       | update_mouse_position()     |                               |
       +---------------------------->|                               |
       |                             | rcu() write to                |
       |                             | ArcSwap<AppState>             |
       |                             |                               |
       | EventLoopProxy.send_event() |                               |
       +------------------------------------+                        |
       |                             |      |  Event::UserEvent      |
       |                             |      +----------------------->|
       |                             |                               |
       |                             |            load() -> Guard    |
       |                             |<------------------------------+
       |                             |                               |
       |                             | Returns current AppState      |
       |                             +------------------------------>|
       |                             |                     (render frame)
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `luminos-core::state` | Modified | Add `mouse_position: ScreenPoint` field to `AppState` |
| `luminos-core::state_manager` | New | `StateManager` struct with typed read/write methods |
| `luminos-core::event` | New | `LuminosEvent` enum for `EventLoopProxy<LuminosEvent>` |
| `luminos-core::lib.rs` | Modified | Add `pub mod state_manager; pub mod event;` and re-exports |

### Data Flow

1. **Initialization:** Application creates `Arc<ArcSwap<AppState>>` with `AppState::default()`. Passes the `Arc` to `StateManager::new()`. Creates `EventLoop<LuminosEvent>` and gets `EventLoopProxy` from `event_loop.create_proxy()`. Clones the `StateManager` (via `Arc`) and `EventLoopProxy` to input/control threads.

2. **Write path (input thread):** Calls `state_manager.update_mouse_position(point)`. Internally: `self.state.rcu(|current| { let mut new = (**current).clone(); new.mouse_position = point; new })`. After the write, the caller sends `event_loop_proxy.send_event(LuminosEvent::StateChanged)` to wake the render loop.

3. **Read path (render thread):** Each frame, calls `state_manager.load()`. Internally: `self.state.load()` returns an `arc_swap::Guard<Arc<AppState>>`. The guard dereferences to `&AppState`. No mutex, no RwLock, no blocking. The read sees the latest completed `rcu()` write.

4. **Event loop integration:** The winit event loop receives `Event::UserEvent(LuminosEvent::StateChanged)` and calls `window.request_redraw()`. On `Event::WindowEvent { event: RedrawRequested, .. }`, the render loop executes the frame pipeline using the state loaded from ArcSwap.

## API Design

### StateManager

```rust
use std::sync::Arc;
use arc_swap::{ArcSwap, Guard};
use luminos_types::ScreenPoint;
use crate::state::AppState;

/// Thread-safe application state manager.
///
/// Wraps `Arc<ArcSwap<AppState>>` with typed methods for lock-free reads
/// and `rcu()` (read-copy-update) writes. The render thread reads via
/// `load()` every frame; input/control threads write via `update_*()`.
///
/// The `StateManager` does NOT own or call `EventLoopProxy`. The caller
/// is responsible for sending wake events after state mutations.
#[derive(Clone)]
pub struct StateManager {
    state: Arc<ArcSwap<AppState>>,
}

impl StateManager {
    /// Creates a new state manager wrapping the given shared state.
    pub fn new(state: Arc<ArcSwap<AppState>>) -> Self {
        Self { state }
    }

    /// Returns a lock-free guard to the current application state.
    ///
    /// The returned `Guard` dereferences to `&AppState`. This is the
    /// render thread's primary state access method (< 100ns per call).
    pub fn load(&self) -> Guard<Arc<AppState>> {
        self.state.load()
    }

    /// Returns a clone of the inner `Arc<ArcSwap<AppState>>` for sharing.
    pub fn inner(&self) -> Arc<ArcSwap<AppState>> {
        Arc::clone(&self.state)
    }

    /// Updates the current mouse position via read-copy-update.
    ///
    /// The render thread will see the new position on the next `load()` call.
    pub fn update_mouse_position(&self, position: ScreenPoint) {
        self.state.rcu(|current| {
            let mut new_state = (**current).clone();
            new_state.mouse_position = position;
            new_state
        });
    }

    /// Updates the zoom level via read-copy-update.
    ///
    /// Clamps the value to the valid range [1.5, 20.0].
    pub fn update_zoom_level(&self, level: f32) {
        let clamped = level.clamp(1.5, 20.0);
        self.state.rcu(|current| {
            let mut new_state = (**current).clone();
            new_state.settings.magnification.zoom_level = clamped;
            new_state
        });
    }

    /// Toggles magnification on/off via read-copy-update.
    pub fn toggle_magnification(&self) {
        self.state.rcu(|current| {
            let mut new_state = (**current).clone();
            new_state.is_active = !new_state.is_active;
            new_state
        });
    }

    /// Resets zoom level to the default (2.0) via read-copy-update.
    pub fn reset_zoom(&self) {
        self.state.rcu(|current| {
            let mut new_state = (**current).clone();
            new_state.settings.magnification.zoom_level = 2.0;
            new_state
        });
    }
}
```

### LuminosEvent

```rust
/// Custom event type for inter-thread communication with the winit event loop.
///
/// Sent via `EventLoopProxy<LuminosEvent>` from input/control threads to
/// wake the render loop. The winit event loop receives these as
/// `Event::UserEvent(LuminosEvent)`.
#[derive(Debug, Clone)]
pub enum LuminosEvent {
    /// Application state was updated (mouse position, zoom, mode, etc.).
    ///
    /// The render loop should call `window.request_redraw()` to render
    /// the next frame with the updated state.
    StateChanged,

    /// Graceful shutdown requested.
    ///
    /// The render loop should stop the input monitor, clean up resources,
    /// and exit the event loop.
    RequestExit,
}
```

### AppState Extension

```rust
// In luminos-core/src/state.rs -- add mouse_position field:

use luminos_types::ScreenPoint;

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub settings: AppSettings,
    pub viewport: ScreenRect,
    pub tts_status: TtsStatus,
    pub active_display_id: Option<String>,
    pub is_active: bool,
    /// Current mouse cursor position in screen coordinates.
    /// Updated by the input monitoring thread via `StateManager::update_mouse_position()`.
    pub mouse_position: ScreenPoint,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: AppSettings::default(),
            viewport: ScreenRect { x: 0, y: 0, width: 0, height: 0 },
            tts_status: TtsStatus::Idle,
            active_display_id: None,
            is_active: false,
            mouse_position: ScreenPoint { x: 0, y: 0 },
        }
    }
}
```

## Error Handling

This story introduces no new error types. The `StateManager` methods are infallible:
- `load()` always succeeds (ArcSwap is lock-free, no failure mode).
- `rcu()` retries internally on contention -- it never fails, only retries.
- `EventLoopProxy::send_event()` returns `Err` only if the `EventLoop` has been dropped. The caller should handle this (e.g., log a warning and stop sending), but this is wired in Story 005, not here.

`LuminosEvent` has no error variants. The `RequestExit` variant is a signal, not an error.

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| All platforms | `arc-swap` crate (pure Rust) | Platform-independent. No OS-specific code. |
| All platforms | `winit::event_loop::EventLoopProxy` | winit handles platform-specific event loop wake-up internally. |

This story is entirely platform-independent. `ArcSwap` and `EventLoopProxy` are pure Rust constructs that work identically on all platforms. No conditional compilation is needed.

## Testing Strategy

### Unit Tests

- **StateManager read/write:** Create `StateManager`, write via `update_mouse_position()`, read via `load()`, verify value.
- **StateManager toggle:** Toggle magnification, verify `is_active` flips.
- **StateManager zoom update:** Update zoom level, verify it's clamped to valid range.
- **StateManager reset zoom:** Reset zoom, verify it returns to 2.0 default.
- **StateManager concurrent write:** Spawn two threads calling `update_mouse_position()` and `update_zoom_level()` concurrently, verify both updates are visible (neither lost).
- **AppState default mouse_position:** Verify `AppState::default().mouse_position` is `(0, 0)`.
- **AppState clone preserves mouse_position:** Clone an AppState with non-default mouse_position, verify clone matches.
- **LuminosEvent is Send:** Static assertion that `LuminosEvent: Send` (required for `EventLoopProxy<T: Send>`).

### Integration Tests

- **ArcSwap benchmark:** Measure `load()` latency over 1M iterations, verify average < 100ns (NFR-1).
- **Cross-thread visibility:** Spawn writer thread, write value, signal reader thread, verify reader sees new value on next `load()`.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Unit | `StateManager::load()` returns `Guard` reference, deref to `&AppState` succeeds |
| AC-1.2 | Integration (benchmark) | Loop 1M `load()` calls, measure avg < 100ns |
| AC-1.3 | Integration | Spawn writer + reader threads, verify convergence within one generation |
| AC-2.1 | Unit | `update_mouse_position(ScreenPoint { x: 500, y: 300 })`, `load()`, assert `mouse_position == (500, 300)` |
| AC-2.2 | Unit | `update_zoom_level(5.0)`, `load()`, assert `settings.magnification.zoom_level == 5.0` |
| AC-2.3 | Unit | Set `is_active = true`, `toggle_magnification()`, assert `is_active == false` |
| AC-2.4 | Unit | `reset_zoom()`, assert `settings.magnification.zoom_level == 2.0` |
| AC-2.5 | Integration | Two threads writing concurrently, verify both updates present in final state |
| AC-3.1 | Unit | Static type assertion: `LuminosEvent: Send`. Construct `LuminosEvent::StateChanged`, verify Debug format |
| AC-3.2 | Unit | Construct `LuminosEvent::RequestExit`, verify Debug format and pattern matching works |
| AC-3.3 | Unit | Verify `EventLoopProxy<LuminosEvent>: Clone + Send` (compile-time assertion) |
| AC-4.1 | Unit | `AppState::default().mouse_position == ScreenPoint { x: 0, y: 0 }` |
| AC-4.2 | Unit | Clone AppState with custom mouse_position, verify clone preserves value |

## Performance Targets

| Metric | Target | Source |
|--------|--------|--------|
| `ArcSwap::load()` average latency | < 100ns | NFR-1, SC4, D4 |
| `rcu()` write (no contention) | < 1us | Typical ArcSwap performance |
| `rcu()` write (with contention) | Retries without blocking reader | NFR-2 |
| `StateManager` memory overhead | ~64 bytes (Arc + ArcSwap internal) | Negligible |

## Security Considerations

- **No sensitive data in AppState:** The `AppState` contains configuration values (zoom level, mode) and transient UI state (mouse position, TTS status). No passwords, tokens, or user content is stored.
- **Mouse position privacy:** The `mouse_position` field records the cursor's screen coordinates, which is inherent to the magnification use case. This data is not logged, persisted, or transmitted.

## Alternatives Considered

### Alternative 1: `RwLock<AppState>` (rejected)

Standard `Arc<RwLock<AppState>>` would work for shared state but introduces lock contention risk. The render thread reads state every frame (60 reads/sec). If a write is in progress, the read blocks -- potentially causing a frame drop. `ArcSwap` eliminates this: reads are always lock-free, writes create a new `Arc` that readers pick up on the next `load()`. Doc-01 AD-08 explicitly chose ArcSwap for this reason.

### Alternative 2: Atomic fields for hot-path values (deferred)

Doc-01 Section 6.4 mentions `AtomicI32` pairs for viewport position. This would eliminate the `Arc` clone overhead in `rcu()` for frequently-updated values (mouse position changes every frame). Deferred because: (1) ArcSwap's `load()` is already < 100ns, well within the 16.67ms frame budget; (2) atomic fields fragment state access across multiple locations; (3) ArcSwap provides a consistent snapshot of all state fields in one `load()` call. Can be revisited if profiling shows contention.

### Alternative 3: StateManager owns EventLoopProxy (rejected)

An earlier design had `StateManager` call `event_loop_proxy.send_event()` automatically after each `rcu()` write. Rejected because: (1) it couples `StateManager` to winit, making it untestable without a live event loop; (2) some callers may batch multiple state changes before waking the event loop; (3) the caller has better context about when to wake (e.g., mouse moves may not need immediate wake if the render loop is already running on `AboutToWait`).
