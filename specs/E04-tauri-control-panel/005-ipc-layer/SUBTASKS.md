# Subtasks: Story E04/005 -- IPC Command Layer & tauri-specta Bindings

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
| 1. Setup | 2 | 0 | 0 | 2 |
| 2. Core (commands) | 3 | 0 | 0 | 3 |
| 3. Integration (events, builder, capability) | 3 | 0 | 0 | 3 |
| 4. Polish & Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **9** | **0** | **0** | **9** |

> Reconcile to REAL engine types (epic Integration Points): zoom at `AppState.settings.magnification.zoom_level` via `StateManager`; wake via `AppNotifier` dirty flag; `LuminosEvent` unchanged. Events are hotkey-origin only (AD-5).

---

## Phase 1: Setup

### T001 -- `StateManager` additive methods + `frame_timings` slot on handle
**Traces to:** FR-3, FR-4, FR-5, FR-2
**Status:** TODO
**Files:** `crates/luminos-core/src/state_manager.rs`, `crates/luminos-app/src/handle.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `state_manager_set_magnification_mode` -- RCU write of `settings.magnification.mode`.
   - [ ] `state_manager_replace_settings` -- RCU replace of `settings`.
2. **Green:** Add `set_magnification_mode(MagnificationMode)` and `replace_settings(AppSettings)` to `StateManager`. Add `frame_timings: Arc<Mutex<FrameTimingSummary>>` to `LuminosHandle` (default zeroed), updated by the story-003 loop.
3. **Refactor:** Keep RCU pattern consistent with existing methods.

**Completion Notes:**
>

---

### T002 -- specta::Type prerequisite + IPC module scaffold + deps
**Traces to:** FR-1, FR-2, FR-7 (CRITICAL prerequisite — commands won't compile without it)
**Status:** TODO
**Files:** `crates/luminos-types/**`, `crates/luminos-core/src/config/schema.rs`, `crates/luminos-gpu/src/frame_timings.rs`, `crates/{luminos-types,luminos-core,luminos-gpu}/Cargo.toml` + `lib.rs`, `crates/luminos-app/src/{tauri_commands,events,ipc}.rs`, `crates/luminos-app/Cargo.toml`, root `Cargo.toml`

**TDD Cycle:**
1. **Red:**
   - [ ] `frame_timing_summary_serde_camelcase` -- `FrameTimingSummary` serializes to `averageMs`/`p99Ms`/… (it currently has NO serde).
   - [ ] `appsettings_implements_specta_type` / `magmode_implements_specta_type` -- compile-time bound checks (`fn _assert<T: specta::Type>(){}`).
2. **Green:**
   - [ ] Pin `specta` in `[workspace.dependencies]`; add it to `luminos-types`/`luminos-core`/`luminos-gpu`.
   - [ ] `#[derive(specta::Type)]` on `MagnificationMode` (+ other IPC enums), `AppSettings` + all sub-structs; add `serde::{Serialize,Deserialize}` + `#[serde(rename_all="camelCase")]` + `specta::Type` to `FrameTimingSummary`.
   - [ ] Add `pub use` re-exports: `luminos-gpu` (`Renderer`, `FrameTimingSummary`, `InterpolationMethod`), `luminos-platform` (`ScreenCapture`) — so ergonomic paths resolve.
   - [ ] Ensure `tauri-specta`/`specta`/`specta-typescript` under the `tauri` feature (workspace pins). Scaffold the three IPC modules.
3. **Refactor:** Keep existing serde field names stable; verify all engine crates still build without the `tauri` feature (specta is a normal dep, not feature-gated).

**Completion Notes:**
>

> ⚠️ This task has the largest blast radius in E04 — it touches three engine crates. Alternative (rejected here): local specta DTOs in `luminos-app` with `From` conversions (better layering, more boilerplate). Chosen: derive on engine types for one canonical type set (doc-05 intent). If the team prefers DTOs, swap this task's approach and keep engine crates specta-free.

**Completion Notes:**
>

---

## Phase 2: Core (commands)

### T003 -- Read commands
**Traces to:** FR-1, FR-2, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-app/src/tauri_commands.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `get_current_settings_returns_state` -- returns `app_state.settings` clone.
   - [ ] `get_frame_timings_zeroed_before_loop` -- zeroed summary when loop not running.
2. **Green:** Implement both (logic testable without the async Tauri runtime via a constructed handle).
3. **Refactor:** Extract testable inner fns.

**Completion Notes:**
>

---

### T004 -- Mutation commands (zoom/mode/toggle) + validation
**Traces to:** FR-3, FR-4, NFR-2, AC-1.2
**Status:** TODO
**Files:** `crates/luminos-app/src/tauri_commands.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `set_zoom_level_clamps` -- 0.5→1.5, 50→20.
   - [ ] `set_zoom_level_rejects_nan` -- NaN → `Err`.
   - [ ] `set_magnification_mode_writes_state`.
   - [ ] `toggle_magnification_returns_new_state`.
2. **Green:** Implement; each writes via `StateManager` + `notify_state_changed()`.
3. **Refactor:** Shared validation helper.

**Completion Notes:**
>

---

### T005 -- Persistence commands
**Traces to:** FR-5, AC-2.1
**Status:** TODO
**Files:** `crates/luminos-app/src/tauri_commands.rs`

**TDD Cycle:**
1. **Red:**
   - [ ] `save_settings_delegates_to_config` (temp-dir ConfigManager in handle) -- persists current settings.
   - [ ] `reset_settings_returns_defaults` -- reset applied to state + returned.
   - [ ] `commands_handle_config_none` -- `None` config → `Err("config unavailable")`, no panic.
2. **Green:** Implement save/reset per DESIGN (lock `config`, delegate, apply to state, wake).
3. **Refactor:** —

**Checkpoint:** All 7 command bodies unit-tested against a constructed handle.

**Completion Notes:**
>

---

## Phase 3: Integration (events, builder, capability)

### T006 -- Events + emission from the hotkey path
**Traces to:** FR-6, AC-2.2
**Status:** TODO
**Files:** `crates/luminos-app/src/events.rs`, `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:** `hotkey_emits_zoom_changed` (subprocess, with story 003) -- `xdotool` zoom hotkey → `ZoomChangedEvent` emitted (logged emit or test listener).
2. **Green:** Define `ZoomChangedEvent`/`ModeChangedEvent`; in the input path, after a hotkey-origin state change, `Event::emit(&app_handle)`.
3. **Refactor:** Emit only on hotkey origin (not UI origin) — AD-5.

**Completion Notes:**
>

---

### T007 -- `tauri-specta` Builder + bindings export
**Traces to:** FR-7, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-app/src/ipc.rs`, `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:** `bindings_export_smoke` -- running the export produces `ui/src/ipc/bindings.ts` with all 7 commands + 2 events (string contains checks).
2. **Green:** `build_ipc_handler()`; wire `invoke_handler` + `mount_events`; `#[cfg(debug_assertions)]` export to `../ui/src/ipc/bindings.ts`.
3. **Refactor:** Ensure release build skips export.

**Completion Notes:**
>

---

### T008 -- Capability file
**Traces to:** FR-8, AC-3.1
**Status:** TODO
**Files:** `crates/luminos-app/capabilities/default.json`, `crates/luminos-app/tauri.conf.json`

**TDD Cycle:**
1. **Red:** `capability_minimal` -- JSON contains only `core:default`, `core:event:default`, `shell:allow-open`; windows `["main","overlay"]`.
2. **Green:** Author the capability; reference it from `tauri.conf.json`.
3. **Refactor:** —

**Checkpoint:** Commands + events + bindings + capability all wired; app builds with IPC.

**Completion Notes:**
>

---

## Phase 4: Polish & Acceptance

### T009 -- CI bindings-up-to-date check + acceptance + AC matrix
**Traces to:** FR-7, All ACs
**Status:** TODO
**Files:** `.github/workflows/ci.yml`, story docs

**Verification Checklist:**
- [ ] AC-1.1 read commands
- [ ] AC-1.2 mutation validation + write + wake
- [ ] AC-2.1 persistence commands (+ config-None handling)
- [ ] AC-2.2 hotkey events emitted
- [ ] AC-3.1 bindings exported + CI `git diff --exit-code ui/src/ipc/bindings.ts` passes (D7) + capability minimal
- [ ] `cargo fmt`/clippy clean; no `unwrap`/`expect`
- [ ] CI step runs the debug export and fails on stale bindings

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
