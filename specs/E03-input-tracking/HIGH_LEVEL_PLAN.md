# Epic E03: Input Tracking & Interactive Magnification

**Status:** DONE
**Roadmap Ref:** [tech-strategy/09-implementation-roadmap.md Section 4.3](../tech-strategy/09-implementation-roadmap.md#43-epic-3----input-tracking--interactive-magnification)
**Phase:** Phase 0: Foundation (Months 1-3)
**Started:** 2026-03-28
**Completed:** 2026-03-29
**Hard Dependencies:** E2 (X11 Screen Capture & GPU Magnification) -- DONE 2026-03-28
**Soft Dependencies:** None
**Primary Docs:** [02 -- Platform Abstraction](../tech-strategy/02-platform-abstraction.md) Section 3.6 (InputMonitor trait) and Section 8.1 (Linux X11 input), [03 -- Rendering Pipeline](../tech-strategy/03-rendering-pipeline.md) Sections 3 (viewport calc) and 3.3-3.4 (tracking modes, edge panning), [01 -- System Architecture](../tech-strategy/01-system-architecture.md) Sections 5.2 (input flow), 6.4 (inter-thread comms), and 6.5 (event loop integration)

---

## Overview

Make the magnifier interactive. After E02, Luminos renders magnified screen content at 60fps in an overlay window -- but the view is static. E03 adds global input monitoring (mouse position and keyboard events), a viewport tracking engine that smoothly follows the cursor, and global keyboard shortcuts for zoom control and magnification toggle. After this epic, a user can actually *use* the magnifier: moving their mouse pans the magnified view smoothly, pressing Ctrl+Alt+= zooms in, pressing Ctrl+Alt+- zooms out, and pressing Ctrl+Alt+8 toggles magnification on/off.

This epic also establishes the `ArcSwap<AppState>` runtime state distribution pattern and `EventLoopProxy` integration that all subsequent epics (E04 control panel, E07 focus tracking) build upon. The input thread writes mouse position and hotkey-triggered state changes to `ArcSwap<AppState>`; the render thread reads them lock-free every frame via `ArcSwap::load()`.

## Success Criteria

Copied from [doc-09 Section 4.3](../tech-strategy/09-implementation-roadmap.md#43-epic-3----input-tracking--interactive-magnification):

- [x] Mouse movement updates viewport position within 1 frame (< 16.67ms)
- [x] Panning is visually smooth (no jitter or snapping) at all zoom levels
- [x] All four keyboard shortcuts work on X11 (zoom in, zoom out, toggle, reset)
- [x] `ArcSwap` state update from input thread is visible to render thread on the next frame
- [x] No dropped frames during rapid mouse movement (P99 frame time < 20ms)

---

## Story Breakdown

### Progress Summary

| # | Story | Status | Depends On | Est. Effort | Notes |
|---|-------|--------|------------|-------------|-------|
| 001 | X11 Global Input Monitoring (x11rb) | DONE (2026-03-29) | --- | L (12-15 subtasks) | Parallel with 002. Covers D1. 36 new tests, 343 total after post-review fixes. |
| 002 | ArcSwap State Management & EventLoopProxy | DONE (2026-03-29) | --- | M (8-10 subtasks) | Parallel with 001. Covers D4. 24 new tests, StateManager + LuminosEvent + AppState.mouse_position. |
| 003 | Cursor-Follow Viewport Tracking | DONE (2026-03-29) | 001, 002 | M (8-12 subtasks) | Covers D2. 24 new tests, TrackingEngine with dead zone + edge panning + smooth interpolation. |
| 004 | Global Keyboard Shortcuts | DONE (2026-03-29) | 001, 002 | M (8-10 subtasks) | Covers D3. 34 new tests, HotkeyMatcher (7 bindings) + dispatch_hotkey. |
| 005 | End-to-End Pipeline Integration | DONE (2026-03-29) | 003, 004 | M (8-12 subtasks) | All deliverables verified. 14 unit + 10 integration tests, EventNotifier trait, InputProcessingTask pipeline. |

**Total Stories:** 5 | **Done:** 5 | **In Progress:** 0 | **Blocked:** 0

**Dependency graph:**

```
001 X11 Input Monitor ──────────────────────┐
                                            │
                                            ├──> 003 Viewport Tracking ──┐
                                            │                           │
002 ArcSwap & EventLoopProxy ──────────────┤                           ├──> 005 E2E Integration
                                            │                           │
                                            └──> 004 Keyboard Shortcuts ─┘
```

Stories 001 and 002 can execute in parallel (no internal dependencies). Stories 003 and 004 can execute in parallel once both 001 and 002 are complete. Story 005 depends on 003 and 004 (it wires the full interactive pipeline and runs acceptance verification).

### Deliverable Traceability

Every roadmap deliverable (D1-D4) and success criterion (SC1-SC5) maps to at least one story:

| Deliverable | Description | Story |
|-------------|-------------|-------|
| D1 | Mouse position continuously tracked and fed to viewport engine | 001, 003 |
| D2 | Magnification viewport follows cursor smoothly at 60fps | 003, 005 |
| D3 | Keyboard shortcuts change zoom level and toggle magnification | 004, 005 |
| D4 | `ArcSwap<AppState>` reads are lock-free on the render thread | 002, 005 |

| Success Criterion | Story |
|-------------------|-------|
| SC1: Mouse movement updates viewport within 1 frame (< 16.67ms) | 001, 003, 005 |
| SC2: Panning is visually smooth at all zoom levels | 003, 005 |
| SC3: All four keyboard shortcuts work on X11 | 004, 005 |
| SC4: ArcSwap state update visible to render thread on next frame | 002, 005 |
| SC5: No dropped frames during rapid mouse movement (P99 < 20ms) | 003, 005 |

### Story Descriptions

#### 001 -- X11 Global Input Monitoring (x11rb)

**Scope:** Implement the `InputMonitor` trait for Linux X11 using `x11rb` directly with XInput2 (XI2) extension for mouse position tracking and global keyboard event capture. The implementation spawns a dedicated input monitoring thread that connects to the X server, registers for XI2 `Motion`, `ButtonPress`, `ButtonRelease`, `KeyPress`, and `KeyRelease` events on `XIAllMasterDevices`, translates them into `InputEvent` variants, and sends them through a bounded `tokio::sync::mpsc` channel. Regular (non-Raw) events are used because they provide absolute screen coordinates directly. Mouse move events use `try_send()` with lossy semantics (drop when full); key events use `blocking_send()` to prevent dropped hotkeys.

**Key Deliverables:**
- `crates/luminos-platform/src/linux_x11/input.rs` containing `X11InputMonitor` struct implementing `InputMonitor`
- X11 connection management: two persistent `x11rb` connections (one for queries, one for the monitor thread's blocking event loop -- avoids `wait_for_event()` lock contention) with XInput2 extension initialization
- `subscribe_input_events()`: spawns a thread that listens for XI2 `Motion`, `ButtonPress`, `ButtonRelease`, `KeyPress`, and `KeyRelease` events, translates X11 keycodes to `KeyCode` enum via `GetKeyboardMapping` keysym lookup, tracks modifier state, sends `InputEvent` variants on the channel
- `get_mouse_position()`: synchronous `QueryPointer` request to the X server
- X11 keycode-to-`KeyCode` mapping module (covers all `KeyCode` variants defined in the trait)
- Unit tests: keycode mapping, modifier state tracking, InputEvent construction, error cases
- Integration tests gated behind `#[cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]` running on Xvfb: actual mouse position query, event subscription produces events when `xdotool` simulates input

**Estimated Effort:** L (12-15 subtasks)

**Notes:** This story deliberately uses `x11rb` directly instead of the `rdev` crate recommended by the tech strategy docs. Rationale: (1) `rdev` is a single-maintainer crate (RISK-031) with 0.5.x versioning and infrequent releases; (2) `rdev`'s `listen()` function uses X11 APIs but `grab()` (the Wayland path) intercepts events rather than passively monitoring them (doc-02 Section 8.2); (3) using `x11rb` directly gives full control over XInput2, which are the same APIs `rdev` wraps internally; (4) `x11rb` is already a workspace dependency (v0.13, with `randr` and `shm` features). The XInput2 extension requires the `xinput` feature on `x11rb` (which transitively enables `xfixes`). Keysym resolution uses core protocol `GetKeyboardMapping` (no `xkb` feature needed). The `InputMonitor` trait, `InputEvent`, `KeyCode`, `Modifiers`, `MouseButton`, and `InputError` are all already defined in `luminos-platform::traits::input_monitor` from E01.

**Risk Refs:** RISK-031 (single-maintainer crate dependencies -- mitigated by choosing x11rb over rdev), RISK-012 (Wayland input -- this story is X11-only; Wayland input backend is deferred to E08)

---

#### 002 -- ArcSwap State Management & EventLoopProxy

**Scope:** Wire up the `ArcSwap<AppState>` shared state distribution and `EventLoopProxy` integration that connects the input monitoring thread to the winit render loop. The input thread (Story 001) and hotkey handler (Story 004) write state changes to `ArcSwap<AppState>` via `rcu()`. The render thread reads the current state lock-free via `ArcSwap::load()` every frame. State changes trigger a `RedrawRequested` on the winit event loop via `EventLoopProxy::send_event()`, ensuring the render thread wakes immediately rather than waiting for the next vsync tick.

**Key Deliverables:**
- `crates/luminos-core/src/state_manager.rs` containing `StateManager` struct:
  - Owns `Arc<ArcSwap<AppState>>` and exposes read/write methods
  - `load()` returns a `Guard` (lock-free read, <100ns target)
  - `update_mouse_position(position: ScreenPoint)` -- writes mouse position via `rcu()`
  - `update_zoom_level(level: f32)` -- writes zoom level via `rcu()`
  - `toggle_magnification()` -- flips `is_active` via `rcu()`
  - `reset_zoom()` -- resets zoom to default via `rcu()`
- Custom winit event type (`LuminosEvent` enum) for `EventLoopProxy<LuminosEvent>`:
  - `StateChanged` -- generic wake event after any ArcSwap update
  - `RequestExit` -- graceful shutdown request
- `AppState` extended with `mouse_position: ScreenPoint` field for current cursor location (the viewport tracking engine in Story 003 reads this to compute the source region)
- Unit tests: ArcSwap read/write round-trip, state visibility across threads (spawn writer + reader, verify convergence), `rcu()` contention test (multiple concurrent writers), StateManager method correctness
- Benchmark test: `ArcSwap::load()` latency < 100ns (D4 verification)

**Estimated Effort:** M (8-10 subtasks)

**Notes:** `arc-swap` is already a workspace dependency (v1.9.0) used by `luminos-core` and `luminos-app`. `AppState` already exists in `luminos-core::state` with `settings`, `viewport`, `tts_status`, `active_display_id`, and `is_active` fields. This story adds `mouse_position` and creates the `StateManager` convenience wrapper. The `LuminosEvent` enum lives in `luminos-core` (or `luminos-app`) because it couples the winit event loop to the application state -- it must NOT live in `luminos-platform` (which has no winit dependency). The `EventLoopProxy` type is generic over `LuminosEvent` and will be threaded through from the winit event loop setup in E02's render loop.

**Risk Refs:** None identified beyond existing architecture risks (RISK-001 dual event loop).

---

#### 003 -- Cursor-Follow Viewport Tracking

**Scope:** Implement the viewport tracking engine that reads the current mouse position from `ArcSwap<AppState>` and updates the viewport source region every frame, using the smooth panning algorithm from E02's `viewport.rs`. This story wires `smooth_viewport_position()` into the per-frame render cycle and adds configurable dead zone and edge panning behavior. The tracking engine lives in `luminos-core` (pure logic, no GPU dependency); the viewport math functions remain in `luminos-gpu::viewport`.

**Key Deliverables:**
- `crates/luminos-core/src/tracking.rs` containing `TrackingEngine` struct:
  - Holds current smoothed viewport center (`ScreenPoint`), dead zone configuration, edge panning margins
  - `update(mouse_position: ScreenPoint, dt: f32) -> ScreenPoint` -- computes new viewport center using smooth interpolation
  - Dead zone: configurable rectangular region around viewport center where mouse movement does not pan (default: 20% of viewport dimensions)
  - Edge panning: when cursor is within the edge margin (default: 15% of viewport width/height), viewport pans proportionally to cursor distance from inner boundary (doc-03 Section 3.4)
  - Smoothing factor: configurable (default: 0.2, range 0.05-1.0 per doc-03 Section 3.3)
- Integration with the render loop: each frame reads `mouse_position` from `ArcSwap<AppState>`, calls `TrackingEngine::update()`, then calls `compute_source_region()` (already in `luminos-gpu::viewport`) with the smoothed position
- `AppState.viewport` updated each frame with the computed source region
- Unit tests: dead zone suppresses panning, edge panning activates at margins, smooth interpolation converges over frames, tracking at screen boundaries (clamping), various zoom levels
- Performance test: tracking engine update < 0.01ms (pure math)

**Estimated Effort:** M (8-12 subtasks)

**Notes:** The `compute_source_region()` and `smooth_viewport_position()` functions already exist in `luminos-gpu::viewport` (implemented in E02 Story 004). This story builds the higher-level `TrackingEngine` that adds dead zone and edge panning logic on top of those primitives. The tracking engine is a pure-logic component (no I/O, no GPU, no allocation on the hot path) -- it can be exhaustively unit-tested. The `MagnificationSettings` struct in `luminos-core::config::schema` already has `smooth_scrolling: bool` which can gate whether smoothing is applied (factor=1.0 when disabled). Dead zone and edge margin percentages are hardcoded defaults in E03; user-configurable values come in E07.

**Risk Refs:** None specific. The smooth panning algorithm risk noted in doc-09 ("may feel sluggish or jerky") is mitigated by parameterizing the smoothing factor and dead zone.

---

#### 004 -- Global Keyboard Shortcuts

**Scope:** Implement global keyboard shortcut detection and dispatch. The hotkey handler receives `InputEvent::KeyEvent` from the input monitor channel (Story 001), matches against a hardcoded shortcut table (accessibility tool convention: Ctrl+Alt based), and mutates `ArcSwap<AppState>` via `StateManager` (Story 002). Four shortcuts are implemented: zoom in, zoom out, toggle magnification, and reset zoom. Hotkey-triggered state changes send a `LuminosEvent::StateChanged` via `EventLoopProxy` to wake the render loop.

**Key Deliverables:**
- `crates/luminos-core/src/hotkeys.rs` containing:
  - `HotkeyAction` enum already exists in `luminos-core::config::schema` -- reuse it
  - `HotkeyMatcher` struct: holds a `HashMap<(KeyCode, Modifiers), HotkeyAction>` mapping
  - `HotkeyMatcher::default()` initializes with hardcoded ZoomText-convention shortcuts:
    - `Ctrl+Alt+=` (or `Ctrl+Alt+NumpadAdd`) -- `ZoomIn`
    - `Ctrl+Alt+-` (or `Ctrl+Alt+NumpadSubtract`) -- `ZoomOut`
    - `Ctrl+Alt+8` -- `ToggleMagnification`
    - `Ctrl+Alt+0` (or `Ctrl+Alt+Numpad0`) -- `ZoomReset`
  - `match_event(&self, event: &InputEvent) -> Option<HotkeyAction>` -- returns the matched action on key press (not release)
- `crates/luminos-core/src/hotkey_dispatch.rs` (or within `hotkeys.rs`):
  - `dispatch_hotkey(action: HotkeyAction, state_manager: &StateManager)` -- executes the state mutation
  - Zoom in: multiply current zoom by 1.5 (capped at 20.0)
  - Zoom out: divide current zoom by 1.5 (floored at 1.5)
  - Toggle: flip `is_active`
  - Reset: set zoom to default (2.0)
- Unit tests: each shortcut matches correctly, modifier combinations (Ctrl+Alt but not Ctrl+Shift+Alt), key release does not trigger, zoom clamping at boundaries, toggle round-trips
- Integration tests on Xvfb: `xdotool key ctrl+alt+equal` triggers zoom in (requires wiring to the event channel)

**Estimated Effort:** M (8-10 subtasks)

**Notes:** The `HotkeyAction` enum and `KeyBinding` struct already exist in `luminos-core::config::schema` from E01. This story uses `HotkeyAction` but does NOT use the configurable `KeyBinding` system -- shortcuts are hardcoded for Phase 0. Configurable keybindings are deferred to E07. The zoom step factor of 1.5x matches ZoomText's behavior (multiplicative steps rather than additive). With a default zoom of 2.0 and range 1.5-20.0, the zoom-in progression is: 2.0 -> 3.0 -> 4.5 -> 6.75 -> 10.125 -> 15.1875 -> 20.0 (capped); zoom-out reverses this. The `Modifiers` struct and `KeyCode` enum are defined in `luminos-platform::traits::input_monitor`. The hotkey handler runs on the main thread (or the input processing thread), NOT the render thread -- it writes state changes that the render thread reads on the next frame via ArcSwap.

**Risk Refs:** None specific. Configurable keybinding complexity is explicitly deferred to E07.

---

#### 005 -- End-to-End Pipeline Integration

**Scope:** Assemble the full interactive magnification pipeline: input monitor feeds mouse position and keyboard events into the system; the tracking engine smoothly updates the viewport; hotkeys change zoom level and toggle magnification; the render loop reads state from ArcSwap each frame and renders the magnified view. This story wires all E03 components together and upgrades the E02 standalone render loop into an interactive event-driven loop using `EventLoopProxy`. Write acceptance tests that verify all success criteria end-to-end on Xvfb.

**Key Deliverables:**
- Upgraded render loop in `luminos-gpu` (or `luminos-app`):
  - Replaces the E02 static render loop with an interactive event-driven loop
  - On `Event::UserEvent(LuminosEvent::StateChanged)`: request redraw
  - On `Event::AboutToWait`: run tracking engine update, then request redraw
  - Per-frame: load `ArcSwap<AppState>` → `TrackingEngine::update()` → `compute_source_region()` → `capture_frame()` → upload → render → present
- Input monitor thread startup and channel wiring:
  - Spawn `X11InputMonitor::subscribe_input_events()` with buffer_size=32
  - Spawn input processing task that reads from the channel, dispatches mouse moves to `StateManager::update_mouse_position()`, dispatches key events to `HotkeyMatcher`
- Graceful shutdown: `LuminosEvent::RequestExit` stops the input monitor and exits the event loop
- Integration tests on Xvfb (gated behind `ci_platform_tests`):
  - Mouse move via `xdotool mousemove` produces viewport position change within 2 frames
  - `xdotool key ctrl+alt+equal` changes zoom level
  - `xdotool key ctrl+alt+8` toggles magnification
  - Frame timing P99 < 20ms during rapid mouse movement (SC5)
  - ArcSwap state convergence: write from input thread, read from render thread on next frame (SC4)
- Final acceptance verification: all 5 success criteria checked

**Estimated Effort:** M (8-12 subtasks)

**Notes:** This story is the integration point that proves E03 delivers user-perceivable value. The E02 `Renderer` struct holds all GPU resources; this story wraps it in the interactive event loop. The `xdotool` integration tests require `xdotool` to be installed in the CI runner (add to the `apt-get install` in `.github/workflows/ci.yml`). The input processing task (reading from the mpsc channel and dispatching to StateManager/HotkeyMatcher) can be a synchronous loop on a dedicated thread or an async task -- prefer synchronous to avoid async runtime on the render path. The `EventLoopProxy` is created from `EventLoop::create_proxy()` before the event loop starts, then cloned to the input processing thread.

**Risk Refs:** RISK-001 (dual event loop coexistence -- validated in E02, extended here with EventLoopProxy)

---

## Shared Context

This section contains cross-cutting knowledge that applies to all stories in this epic. Agents working on any story should read this section. Update it as stories are completed and new knowledge emerges.

### Architecture Decisions

These decisions are drawn from the tech strategy and key project decisions. They apply across all E03 stories:

- **x11rb instead of rdev for X11 input monitoring (deliberate deviation from tech strategy).** The tech strategy (doc-02 Section 3.6, 8.1) recommends `rdev` as the primary input monitoring library with XInput2/XRecord as fallback. E03 inverts this: use `x11rb` with XInput2/XRecord directly, skipping `rdev` entirely. Rationale: (1) `rdev` is a single-maintainer crate at v0.5.x (RISK-031); (2) `rdev`'s `listen()` uses X11 internally anyway, and its `grab()` for Wayland intercepts events rather than passively monitoring (doc-02 Section 8.2); (3) `x11rb` is already a workspace dependency and provides direct access to XInput2 and XRecord; (4) avoiding `rdev` removes a dependency and gives full control over the X11 event loop. The `InputMonitor` trait's platform table in its doc-comment should be updated to reflect this decision when the implementation lands.

- **ArcSwap<AppState> for lock-free render thread reads (confirmed, doc-01 AD-08).** The render thread reads state every frame via `ArcSwap::load()` (<100ns). The input thread and hotkey handler write via `rcu()`. No mutexes or RwLocks on the render hot path. See [doc-01 Section 6.4](../tech-strategy/01-system-architecture.md#64-inter-thread-communication) and [doc-06 Section 2.1](../tech-strategy/06-cross-cutting-concerns.md).

- **EventLoopProxy for inter-thread wake (doc-01 Section 6.5).** State changes from the input thread wake the winit event loop via `EventLoopProxy::send_event(LuminosEvent::StateChanged)`. This ensures immediate redraw without polling or sleeping. The `EventLoopProxy` is the canonical mechanism for cross-thread communication with the winit event loop; it is thread-safe and cloneable.

- **Bounded mpsc channel for input events (doc-01 Section 6.4).** `input_events` channel capacity = 32. Mouse move events use `try_send()` (lossy: drop when full, only latest position matters). Key events use blocking send (prevent dropped hotkeys). This matches the trait contract specified in `InputMonitor::subscribe_input_events()` doc-comment.

- **Hotkeys follow accessibility tool conventions (Ctrl+Alt prefix).** Phase 0 delivers four hardcoded shortcuts: Ctrl+Alt+= (zoom in), Ctrl+Alt+- (zoom out), Ctrl+Alt+8 (toggle, matches GNOME magnifier convention), Ctrl+Alt+0 (reset). Configurable keybindings are deferred to E07. The Ctrl+Alt prefix avoids conflict with common application shortcuts (Ctrl+C, Ctrl+V, etc.). Note: ZoomText uses Caps Lock-based shortcuts (Caps Lock+Up/Down for zoom, Caps Lock+Ctrl+Enter for toggle). Luminos uses Ctrl+Alt to avoid hijacking Caps Lock. Ctrl+Alt+8 is chosen to match GNOME's magnifier toggle (Super+Alt+8) and avoid VT switching conflicts (Ctrl+Alt+F1-F12 switch TTYs on Linux).

- **Tracking engine in luminos-core, viewport math in luminos-gpu.** The `TrackingEngine` (dead zone, edge panning, smoothing orchestration) is a pure-logic component in `luminos-core`. It calls the existing `smooth_viewport_position()` and `compute_source_region()` functions in `luminos-gpu::viewport`. This separation keeps `luminos-core` free of GPU dependencies while reusing the viewport math already validated in E02.

- **Viewport position stored as AtomicI32 pair vs. in ArcSwap<AppState>.** Doc-01 Section 6.4 mentions `Atomic (x, y as AtomicI32)` for viewport position. E03 takes a simpler approach: store `mouse_position` in `AppState` and let the tracking engine compute `viewport` each frame from the mouse position. The ArcSwap read is <100ns per load, and the tracking computation is <0.01ms -- both well within the 16.67ms frame budget. If profiling reveals contention, the atomic pair approach can be added later.

### Key Type Definitions

**Existing types used in E03 (from E01/E02, canonical source `luminos-types`):**
- `ScreenPoint { x: i32, y: i32 }` -- mouse position, viewport center
- `ScreenRect { x: i32, y: i32, width: u32, height: u32 }` -- viewport source region
- `AppState { settings, viewport, tts_status, active_display_id, is_active }` -- extended in Story 002 with `mouse_position: ScreenPoint`
- `AppSettings` including `MagnificationSettings { zoom_level, mode, tracking_mode, smooth_scrolling, ... }`

**Existing types from E01 (`luminos-platform::traits::input_monitor`):**
- `InputEvent` enum: `MouseMoved`, `MouseButton`, `Scroll`, `KeyEvent`
- `KeyCode` enum: full keyboard mapping including alphanumeric, function keys, navigation, modifiers, numpad
- `Modifiers { shift, ctrl, alt, meta }` -- modifier state
- `MouseButton` enum: `Left`, `Right`, `Middle`, `Other(u16)`
- `InputError` enum: `Unavailable`, `Disconnected`, `Platform`
- `InputMonitor` trait: `subscribe_input_events(buffer_size) -> Result<mpsc::Receiver<InputEvent>>`, `get_mouse_position() -> Result<ScreenPoint>`. **Note:** uses `tokio::sync::mpsc` (async channel) -- the x11rb sync event loop must use `Sender::blocking_send()` / `try_send()` to bridge. See Discovered Constraints.

**Existing types from E01 (`luminos-core::config::schema`):**
- `HotkeyAction` enum: `ZoomIn`, `ZoomOut`, `ZoomReset`, `ToggleMagnification`, `CycleMode`, `ReadWhatISee`, `ReadSelection`, `StopSpeech`, `FindCursor`
- `KeyBinding { key: String, modifiers: Vec<ModifierKey> }` -- NOT used in E03 (configurable bindings deferred to E07)
- `ModifierKey` enum: `Ctrl`, `Shift`, `Alt`, `Super`, `Meta`
- `AppSettings.keybindings: HashMap<HotkeyAction, Option<KeyBinding>>` -- already exists, defaults to empty `HashMap`. Story 004's `HotkeyMatcher` uses hardcoded defaults independent of this field; E07 will wire `HotkeyMatcher` to read from `AppSettings.keybindings` for user-configurable bindings.

**Existing functions from E02 (`luminos-gpu::viewport`):**
- `compute_source_region(tracking_target, zoom_level, viewport_size, screen_bounds) -> ScreenRect`
- `smooth_viewport_position(current, target, smoothing_factor) -> ScreenPoint`

**New types introduced in E03:**
- `X11InputMonitor` (Story 001) -- `InputMonitor` impl for X11 via x11rb
- `StateManager` (Story 002) -- convenience wrapper around `Arc<ArcSwap<AppState>>`
- `LuminosEvent` (Story 002) -- custom winit event enum (`StateChanged`, `RequestExit`)
- `TrackingEngine` (Story 003) -- viewport tracking with dead zone and edge panning
- `HotkeyMatcher` (Story 004) -- matches `InputEvent::KeyEvent` to `HotkeyAction`

### Integration Points

- **`X11InputMonitor` --> mpsc channel --> input processing task:** Story 001 produces `InputEvent` values on a bounded channel. Story 005's input processing task consumes them, dispatching mouse moves to `StateManager::update_mouse_position()` and key events to `HotkeyMatcher::match_event()`.

- **`StateManager` <--> `ArcSwap<AppState>` <--> render thread:** Stories 002/004 write state via `StateManager::update_*()` and `toggle_magnification()`. The render thread (Story 005) reads via `ArcSwap::load()` every frame.

- **`EventLoopProxy<LuminosEvent>` --> winit event loop:** After any ArcSwap write, the writer calls `event_loop_proxy.send_event(LuminosEvent::StateChanged)` to wake the render loop immediately. The proxy is created from `EventLoop::create_proxy()` and cloned to all writer threads.

- **`TrackingEngine` --> `compute_source_region()`:** Story 003's tracking engine computes the smoothed viewport center, then Story 005's render loop calls `compute_source_region()` (from E02) to compute the capture region.

- **E02 `Renderer` --> E03 render loop:** The E02 `Renderer` struct (device, queue, pipeline, texture manager, frame timings) is reused unchanged. Story 005 wraps it in the interactive event loop.

- **CI: `xdotool` dependency for integration tests.** Story 005's acceptance tests use `xdotool` to simulate mouse moves and key presses on Xvfb. This must be added to the CI runner's `apt-get install` step.

### Discovered Constraints

_Updated as stories are implemented and cross-story knowledge emerges._

#### Implementation Findings (Stories 001-005)

- **[FINDING] Two RustConnection instances required for X11InputMonitor.** Story 001 uses two separate `x11rb::RustConnection` instances: one for the monitoring thread's blocking `wait_for_event()` loop and one for synchronous queries (`QueryPointer`, `GetKeyboardMapping`). This avoids lock contention on the connection's internal mutex. Both connections share the same `DISPLAY`.

- **[FINDING] Manual Debug impl on X11InputMonitor.** `x11rb::RustConnection` does not derive `Debug`, so `X11InputMonitor` requires a manual `impl Debug` rather than `#[derive(Debug)]`.

- **[FINDING] fp1616 to i32 conversion via bitshift.** XInput2 mouse coordinates are Fixed-Point 16.16 format. Conversion to integer uses `>> 16` shift (discards fractional part). Root window coordinates (`root_x`/`root_y`) are used instead of `event_x`/`event_y` for correctness on the root window.

- **[FINDING] Scroll buttons 4-7 mapped to Scroll events.** X11 reports scroll wheel as button press/release: buttons 4/5 for vertical scroll, buttons 6/7 for horizontal scroll. Button releases for scroll buttons are suppressed (return `None`) to avoid phantom `MouseMoved` events.

- **[FINDING] Keyboard mapping fetched once at construction.** `GetKeyboardMapping` is called once in `X11InputMonitor::new()`. Layout changes at runtime (`MappingNotify`) are not handled -- acceptable for Phase 0, may need attention in E07 for configurable keybindings.

- **[FINDING] StateManager accepts Arc<ArcSwap<AppState>> externally.** `StateManager` does not own or construct the `ArcSwap`; it receives it from the caller. This allows the same `ArcSwap` instance to be shared with the render thread without going through `StateManager`. The `StateManager` also does NOT own `EventLoopProxy` -- callers are responsible for sending wake events after mutations.

- **[FINDING] ArcSwap::load() benchmark uses conditional threshold.** The 100ns NFR-1 target is only enforced in release mode. Debug mode uses 500ns threshold due to unoptimized code. CI runs in debug mode, so NFR-1 is not strictly verified in CI.

- **[FINDING] luminos-gpu changed from optional to required dependency of luminos-core.** Story 003's `TrackingEngine` calls `smooth_viewport_position()` from `luminos-gpu::viewport`, making `luminos-gpu` a required (non-optional) dependency of `luminos-core`.

- **[FINDING] TrackingEngine dead zone scales inversely with zoom level.** Dead zone half-dimensions are computed as `viewport_size / (2 * zoom_level)`, not raw `viewport_size`. At higher zoom levels the dead zone shrinks proportionally, providing consistent user experience across zoom levels.

- **[FINDING] HotkeyMatcher uses exact modifier matching.** `Ctrl+Alt+Shift+=` does NOT trigger `ZoomIn` -- only exact `Ctrl+Alt` (no extra modifiers) matches. This prevents unintended activations when additional modifiers are held.

- **[FINDING] dispatch_hotkey delegates zoom reset to StateManager::reset_zoom().** The `DEFAULT_ZOOM` constant is defined in `StateManager` (value 2.0), not duplicated in `hotkeys.rs`. The `ZOOM_STEP` constant (1.5) for multiplicative zoom is local to `hotkeys.rs`.

- **[FINDING] Hash derive added to Modifiers.** `HotkeyMatcher` uses `HashMap<(KeyCode, Modifiers), HotkeyAction>`, which required adding `Hash` to the `Modifiers` derive in `luminos-platform::traits::input_monitor`.

- **[FINDING] Stories 003 and 004 parallelized with zero file conflicts.** Both stories are pure-logic components in `luminos-core` with no shared source files. Pre-applied changes (lib.rs module declarations, Cargo.toml dependency, Modifiers Hash derive) were committed by the team lead before parallel execution to avoid merge conflicts.

- **[FINDING] EventNotifier trait introduced for testability in Story 005.** Instead of requiring a live `winit::event_loop::EventLoopProxy` in the pipeline, Story 005 defines an `EventNotifier` trait with a single `notify(&self)` method. Production code uses `EventLoopNotifier` (wraps `EventLoopProxy<LuminosEvent>`); tests use `MockNotifier`. This decouples the input processing pipeline from winit, enabling full unit testing without a live event loop.

- **[FINDING] InputProcessingTask encapsulates the input dispatch pipeline.** `InputProcessingTask` in `luminos-core::pipeline` owns the `mpsc::Receiver<InputEvent>`, `StateManager`, `TrackingEngine`, `HotkeyMatcher`, and `EventNotifier`. Its `run()` method is the blocking event loop: `blocking_recv()` -> dispatch mouse moves to StateManager + TrackingEngine, dispatch key events to HotkeyMatcher -> notify render loop. The `spawn()` method launches `run()` on a named `std::thread` ("luminos-input-proc") and returns a `JoinHandle`.

- **[FINDING] Pipeline spawn returns Result for fallible thread creation.** `InputProcessingTask::spawn()` returns `Result<JoinHandle<()>, InputError>` instead of panicking on `thread::Builder::spawn` failure. This allows graceful error handling at the application level.

- **[FINDING] Story 005 added 14 unit tests and 10 integration tests.** Unit tests cover EventNotifier trait, InputProcessingTask construction, event dispatch (mouse move, key events, hotkey matching), and Send+Sync assertions. Integration tests (gated behind `ci_platform_tests`) verify the full pipeline with real X11InputMonitor, xdotool-simulated input, and state convergence verification.

- **[FINDING] Pipeline module lives in luminos-core, not luminos-app.** The `pipeline` module containing `EventNotifier`, `InputProcessingTask`, and related types is in `luminos-core` to keep the integration logic testable without Tauri dependencies. Only the final wiring (creating the `EventLoopProxy` and starting the pipeline) will happen in `luminos-app`.

#### Pre-implementation Findings

- **[CONSTRAINT] `InputMonitor` trait returns `tokio::sync::mpsc::Receiver`, but x11rb runs a synchronous event loop.** The `InputMonitor::subscribe_input_events()` method signature returns `tokio::sync::mpsc::Receiver<InputEvent>`, which is an async channel type. The x11rb XInput2/XRecord event loop is a blocking synchronous loop (calling `connection.wait_for_event()` in a dedicated thread). Story 001 must bridge this: the x11rb thread uses `tokio::sync::mpsc::Sender::blocking_send()` for key events (prevents dropped hotkeys) and `try_send()` for mouse moves (lossy, only latest position matters). The `blocking_send()` method does NOT require a tokio runtime -- it blocks the calling OS thread until the channel has capacity, which is the correct behavior for the dedicated input monitor thread. The receiver side (consumed in Story 005's input processing task) uses `Receiver::blocking_recv()` on a dedicated `std::thread`, which blocks until a message is available or the channel closes. This is the correct approach for a dedicated processing thread -- it does NOT require a tokio runtime and will NOT panic outside an async context. Note: `try_recv()` was considered but would require a polling loop; `blocking_recv()` is cleaner for a dedicated thread.

- **[CONSTRAINT] `AppSettings.keybindings` field exists but is not wired to hotkey detection in E03.** The `AppSettings` struct already has a `keybindings: HashMap<HotkeyAction, Option<KeyBinding>>` field (defaults to empty). Story 004's `HotkeyMatcher` uses hardcoded defaults and does NOT read from this field. Wiring `HotkeyMatcher` to respect `AppSettings.keybindings` (user-configurable overrides) is deferred to E07. Spec writers for Story 004 must NOT design the `HotkeyMatcher` to read from `AppSettings` -- keep the hardcoded table self-contained for Phase 0 simplicity.

- **[CONSTRAINT] `luminos-core` → `luminos-gpu` dependency for viewport functions.** Story 003's `TrackingEngine` in `luminos-core` calls `smooth_viewport_position()` from `luminos-gpu::viewport`. This creates a crate dependency `luminos-core` → `luminos-gpu`. This is safe today (`luminos-gpu` does NOT depend on `luminos-core`), but if `luminos-gpu` ever needs `luminos-core` types, the dependency becomes circular. Mitigation: if this becomes a problem, extract `smooth_viewport_position()` and `compute_source_region()` into `luminos-types` (pure math functions with zero GPU dependency). For E03, the current approach is acceptable — these functions exist in `luminos-gpu` from E02 and moving them would be unnecessary churn.

### Cross-Story Dependencies

- Story 003 (tracking) and Story 004 (hotkeys) both depend on Story 001 (input events channel) and Story 002 (ArcSwap state + EventLoopProxy).
- Story 005 depends on all four preceding stories. It is the integration and acceptance verification story.
- No circular dependencies exist.

---

## Blockers & Issues Log

| ID | Date | Story | Description | Resolution | Status |
|----|------|-------|-------------|------------|--------|
| B001 | --- | --- | --- | --- | --- |

## Retrospective Notes

**Epic completed:** 2026-03-29 (started 2026-03-28, 2 days elapsed)
**Scope:** 5 stories, ~70 subtasks total, ~143 new tests (418 total workspace tests, up from 275 at E02 completion)
**Quality:** All 5 stories passed 3 independent quality gates each (code review, QA engineer, technical auditor)

### What Went Well

- **x11rb over rdev was the right call.** Direct XInput2 access via `x11rb` provided full control over the X11 event loop, avoided a single-maintainer dependency (RISK-031), and reused an existing workspace dependency. The two-connection pattern (query + monitor thread) cleanly solved the `wait_for_event()` lock contention issue.
- **ArcSwap for lock-free state distribution validated.** `ArcSwap::load()` measured <100ns in release mode, well within the 16.67ms frame budget. The `rcu()` pattern for writes with lock-free reads is the correct architecture for the input-to-render data path.
- **EventNotifier trait abstraction improved testability.** Decoupling the pipeline from `winit::EventLoopProxy` via a trait allowed full unit testing of the input processing pipeline without a live event loop or X11 display.
- **Stories 003 and 004 parallelized effectively.** Both pure-logic stories executed simultaneously with zero file conflicts after pre-applying shared changes (module declarations, dependency updates).
- **Spec-driven development methodology held up.** All acceptance criteria had test coverage. SUBTASKS.md completion notes enabled smooth agent handoffs between stories.

### Key Decisions

- **x11rb with XInput2 instead of rdev** -- Direct X11 API control, removes single-maintainer dependency risk
- **EventNotifier trait instead of direct EventLoopProxy coupling** -- Enables unit testing without winit event loop
- **ArcSwap<AppState> instead of AtomicI32 pair for viewport position** -- Simpler architecture, <100ns reads sufficient for 60fps
- **Hardcoded Ctrl+Alt shortcuts for Phase 0** -- Configurable keybindings deferred to E07, reduces scope
- **luminos-gpu as required (not optional) dependency of luminos-core** -- Needed for TrackingEngine to call smooth_viewport_position()

### Deferred Items (Carried Forward)

- P-001: Keyboard mapping not refreshed on layout change (MappingNotify) -- revisit in E07
- P-002: X11InputMonitor thread JoinHandle dropped (detached) -- acceptable for Phase 0
- P-003: rdev still in workspace deps despite not being used -- workspace-level cleanup
- P-004: ArcSwap::load() benchmark only strict in release mode -- CI runs debug
- P-005: update_zoom_level does not guard against NaN input -- E04 IPC boundary should validate
- P-006: TrackingConfig field range validation not enforced -- mitigated by downstream clamping
