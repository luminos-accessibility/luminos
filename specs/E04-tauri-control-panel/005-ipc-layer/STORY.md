# Story E04/005: IPC Command Layer & tauri-specta Bindings

**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** APPROVED (approved-as-authoritative per E04 execution, 2026-06-05)
**Depends On:** 001 (`LuminosHandle`, `AppNotifier`), 004 (`ConfigManager` for save/reset). Soft: 003 (live timings for `get_frame_timings`).

---

## Problem Statement

The running app (stories 001-003) magnifies and persists settings (story 004), but the React control panel (story 006) has no way to talk to the engine. This story builds the **typed IPC layer**: the seven Phase-0 Tauri commands wired to the **real** `StateManager` and `ConfigManager` via `LuminosHandle`, the two engine→panel events (`zoom_changed`, `mode_changed`) emitted when a hotkey (not the UI) changes state, the `tauri-specta` `Builder` that generates `ui/src/ipc/bindings.ts`, and the minimal Tauri capability file.

All wiring reconciles to the **real** engine types (not doc-05's illustrative snippets): zoom lives at `AppState.settings.magnification.zoom_level`, mutated through `StateManager` methods; the engine wake is the story-001 dirty flag via `AppNotifier`; `LuminosEvent` is unchanged.

## User Scenarios

> **AC count = 5** (grouped per epic plan to honor rule 9).

### US-1: The panel can read and control magnification
As a control-panel developer, I want typed commands to read settings/timings and change zoom/mode, so that the UI can drive the engine.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1 (read commands):** Given the running app, when `get_current_settings()` is invoked, then it returns the current `AppState.settings` as `AppSettings`; and when `get_frame_timings()` is invoked, then it returns a `FrameTimingSummary` (live values once story-003's loop runs; a zeroed/last-known summary otherwise — never an error). *(FR-1, FR-2)*
- **AC-1.2 (mutation commands validate + write + wake):** Given the running app, when `set_zoom_level(level)` / `set_magnification_mode(mode)` / `toggle_magnification()` are invoked, then each validates input (zoom clamped to [1.5, 20] via `StateManager::update_zoom_level`; mode is a valid enum), writes through `StateManager` to `ArcSwap<AppState>`, sets the dirty flag via `AppNotifier`, and returns the appropriate result (`toggle` returns the new `bool`). *(FR-3, FR-4)*

### US-2: Persistence and engine→panel sync
As a user, I want a save/reset button to persist my settings, and the panel to stay in sync when I use a hotkey, so that UI and engine never diverge.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1 (persistence commands):** Given the running app, when `save_settings()` is invoked, then the current `AppState.settings` is persisted via `ConfigManager::save` (returns `Ok`); and when `reset_settings()` is invoked, then `ConfigManager::reset` runs, the defaults are applied to `AppState` and the loop is woken, and the defaults are returned as `AppSettings`. *(FR-5)*
- **AC-2.2 (engine→panel events):** Given a Phase-0 hotkey changes zoom or mode (story 003 input path), when the change is applied, then a `ZoomChangedEvent(f32)` / `ModeChangedEvent(MagnificationMode)` is emitted to the webview so the panel's store can update. *(FR-6)*

### US-3: Typed bindings and least-privilege capability
As a developer, I want generated TypeScript bindings matching the Rust signatures and a minimal permission set, so that the UI is type-safe and the app is least-privilege.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1 (bindings + capability):** Given a debug build, when the app starts, then the `tauri-specta` `Builder` exports `ui/src/ipc/bindings.ts` (under `#[cfg(debug_assertions)]`) whose command/event signatures match the Rust definitions, and CI's bindings-up-to-date check passes (D7); and the Tauri capability file grants only `core:default`, `core:event:default`, `shell:allow-open` (no `fs`/`http`). *(FR-7, FR-8)*

## Functional Requirements

- **FR-1:** `get_current_settings(State<LuminosHandle>) -> Result<AppSettings, String>` MUST return `app_state.load().settings.clone()`. *(AC-1.1)*
- **FR-2:** `get_frame_timings(State<LuminosHandle>) -> Result<FrameTimingSummary, String>` MUST return the renderer's summary (zeroed/last-known when the loop is not running). *(AC-1.1)*
- **FR-3:** `set_zoom_level(level: f32, State) -> Result<(), String>` MUST clamp via `StateManager::update_zoom_level` (range [1.5, 20]), then wake. *(AC-1.2)*
- **FR-4:** `set_magnification_mode(mode: MagnificationMode, State) -> Result<(), String>` and `toggle_magnification(State) -> Result<bool, String>` MUST write via `StateManager` and wake; `toggle` returns the new active state. *(AC-1.2)*
- **FR-5:** `save_settings(State) -> Result<(), String>` MUST persist current settings via `ConfigManager`; `reset_settings(State) -> Result<AppSettings, String>` MUST reset, apply to state, wake, and return defaults. *(AC-2.1)*
- **FR-6:** The engine input path MUST emit `ZoomChangedEvent`/`ModeChangedEvent` (`#[tauri_specta::Event]`) when a hotkey changes zoom/mode. *(AC-2.2)*
- **FR-7:** A `tauri-specta` `Builder` MUST collect the 7 commands + 2 events and export `ui/src/ipc/bindings.ts` in debug builds; a CI check MUST fail if committed bindings are stale. *(AC-3.1)*
- **FR-8:** The capability file MUST grant only `core:default`, `core:event:default`, `shell:allow-open`. *(AC-3.1)*

## Non-Functional Requirements

- **NFR-1:** Commands run on Tauri's async runtime, never on the render thread; they MUST NOT hold long locks (StateManager RCU writes are microsecond-scale; `config` mutex critical sections are brief). *(doc-01 §4.7)*
- **NFR-2:** All command inputs MUST be validated server-side (clamp/enum) before mutating state — never trust the webview. *(doc-06 §3.4)*
- **NFR-3:** No `unwrap()`/`expect()`; command errors return `Err(String)` (Tauri convention) with a clear message; internal errors are logged.

## Out of Scope

- All later-phase commands (color filter, cursor, display, tracking-mode, lens/dock, TTS, profiles, keybindings, system info) → their epics.
- The React UI consuming these → story 006.
- `tauri-driver` end-to-end IPC tests in CI → story 007 (this story unit-tests command logic + a bindings generation check).

## Open Questions

- [x] Do commands take `tauri::State<LuminosHandle>`? — **Resolved: yes**, appended as the last param (doc-05's snippets omitted it for brevity). `#[tauri::command] #[specta::specta]` on each.
- [x] How is `FrameTimingSummary` reached from a command? — **Resolved:** story 003 surfaces `frame_timing_summary()` through `LuminosHandle` (e.g. an `Arc<Mutex<FrameTimingSummary>>` updated by the loop, or an accessor); the command reads the latest. Zeroed default before the loop runs.
- [x] Where are events emitted from? — **Resolved:** from the input pipeline in `luminos-app` (story 003 path), using `tauri_specta::Event::emit(&app_handle)`. The UI subscribes (story 006).
- [x] Camel-case for TS? — **Resolved:** `FrameTimingSummary` Rust fields are snake_case; `specta`/serde rename to camelCase for the TS binding (`averageMs`, `p99Ms`, …) consistent with doc-05 §3.4.
