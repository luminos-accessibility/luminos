# Subtasks: Story E04/005 -- IPC Command Layer & tauri-specta Bindings

**Status:** COMPLETE
**Started:** 2026-06-05
**Completed:** 2026-06-05
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 2 | 2 | 0 | 0 |
| 2. Core (commands) | 3 | 3 | 0 | 0 |
| 3. Integration (events, builder, capability) | 3 | 3 | 0 | 0 |
| 4. Polish & Acceptance | 1 | 1 | 0 | 0 |
| **Total** | **9** | **9** | **0** | **0** |

**Status:** COMPLETE (2026-06-05). 7 commands + 2 events + bindings + capability wired. 20 new Rust tests (4 StateManager, 6 specta/serde, 13 command, 1 bindings-smoke, 1 capability, 1 subprocess emit) + the cross-language gate (70/70 story-006 Vitest tests against the regenerated bindings). fmt/clippy/build/deny/audit all green.

> Reconcile to REAL engine types (epic Integration Points): zoom at `AppState.settings.magnification.zoom_level` via `StateManager`; wake via `AppNotifier` dirty flag; `LuminosEvent` unchanged. Events are hotkey-origin only (AD-5).

---

## Phase 1: Setup

### T001 -- `StateManager` additive methods + `frame_timings` slot on handle
**Traces to:** FR-3, FR-4, FR-5, FR-2
**Status:** DONE
**Files:** `crates/luminos-core/src/state_manager.rs`, `crates/luminos-app/src/handle.rs`

**TDD Cycle:**
1. **Red:**
   - [x] `state_manager_set_magnification_mode_writes_mode` / `_preserves_other_fields` -- RCU write of `settings.magnification.mode`.
   - [x] `state_manager_replace_settings_replaces_whole_settings` / `_preserves_runtime_fields` -- RCU replace of `settings`.
2. **Green:** Added `set_magnification_mode(MagnificationMode)` and `replace_settings(&AppSettings)` to `StateManager`. The `frame_timings: Arc<Mutex<FrameTimingSummary>>` slot already existed on `LuminosHandle` (added by story 003 — `handle.rs:39`, `set_frame_timings`/`frame_timings()` accessors at `:88`/`:98`); NOT re-added (per IMPLEMENTATION_NOTES F.2).
3. **Refactor:** RCU pattern mirrors the existing `update_zoom_level`/`reset_zoom` methods; closure clones inside (rcu may re-run).

**Completion Notes:**
> Two pure-core RCU methods added, 4 new unit tests (all pass), clippy clean. `replace_settings` takes `&AppSettings` not `AppSettings` (clippy `needless_pass_by_value` under pedantic) — see Deviations. The handle's frame-timing slot was already present from story 003, so T001 reduced to just the StateManager methods as the notes predicted.

---

### T002 -- specta::Type prerequisite + IPC module scaffold + deps
**Traces to:** FR-1, FR-2, FR-7 (CRITICAL prerequisite — commands won't compile without it)
**Status:** DONE
**Files:** `crates/luminos-types/**`, `crates/luminos-core/src/config/schema.rs`, `crates/luminos-gpu/src/frame_timings.rs`, `crates/{luminos-types,luminos-core,luminos-gpu}/Cargo.toml` + `lib.rs`, `crates/luminos-app/src/{tauri_commands,events,ipc}.rs`, `crates/luminos-app/Cargo.toml`, root `Cargo.toml`

**TDD Cycle:**
1. **Red:**
   - [x] `frame_timing_summary_serde_camelcase` / `_serde_roundtrip` / `_implements_specta_type` -- `FrameTimingSummary` serializes to `averageMs`/`p99Ms`/… (it had NO serde).
   - [x] `appsettings_implements_specta_type` (+ all sub-structs) / `state_enums_implement_specta_type` / `appsettings_wire_format_stays_snake_case` -- compile-time bound checks + the snake_case wire-shape guard.
2. **Green:**
   - [x] Pinned `specta = "=2.0.0-rc.25"` (features `["derive"]`) in `[workspace.dependencies]`; added it as a NORMAL (non-optional, non-feature-gated) dep to `luminos-types`/`luminos-core`/`luminos-gpu` (+ `serde` to gpu, + `serde_json` dev-dep to gpu for the test).
   - [x] `#[derive(specta::Type)]` on: types — `MagnificationMode`, `TrackingMode`, `ColorFilterType`, `DockEdge`, `LensShape`, `PresentMode`, `GpuPreference`, `InterpolationMode`; core — `AppSettings`, `MagnificationSettings`, `ColorFilterConfig`, `CursorConfig`, `SpeechSettings`, `KeyBinding`, `ModelVariant`, `HotkeyAction`, `ModifierKey`; gpu — `FrameTimingSummary` (+ `serde::{Serialize,Deserialize}` + `#[serde(rename_all="camelCase")]`).
   - [x] gpu re-exports (`Renderer`, `FrameTimingSummary`, `InterpolationMethod`) ALREADY existed (`luminos-gpu/src/lib.rs:22`). `luminos-platform::ScreenCapture` re-export NOT added — no story-005 code needs it (commands use only `AppSettings`/`MagnificationMode`/`FrameTimingSummary`); the app already uses the full `luminos_platform::traits::ScreenCapture` path. Recorded as a deviation (keeps luminos-platform untouched).
   - [x] Added `specta` (optional, under the app's `tauri` feature) to `luminos-app` — `#[specta::specta]`/`collect_commands!`/`derive(specta::Type)` need it directly in scope. Scaffolded the three IPC modules (`events.rs`, `tauri_commands.rs`, `ipc.rs`) feature-gated in `lib.rs`.
3. **Refactor:** Existing serde field names unchanged; `cargo build --workspace --exclude luminos-app` passes (engine crates compile with specta, no tauri feature).

> ⚠️ This task has the largest blast radius in E04 — it touches three engine crates. Alternative (rejected here): local specta DTOs in `luminos-app` with `From` conversions (better layering, more boilerplate). Chosen: derive on engine types for one canonical type set (doc-05 intent).

**Completion Notes:**
> Specta derives applied across 3 engine crates + the app. `cargo build --workspace --exclude luminos-app` green (no tauri feature needed for the engine derives). `FrameTimingSummary` is the ONLY camelCase type; `AppSettings`+sub-structs stay snake_case; enums stay bare PascalCase — proven by `appsettings_wire_format_stays_snake_case`, `frame_timing_summary_serde_camelcase`, and the existing per-enum serde round-trip tests. specta resolved to a single `2.0.0-rc.25` (lockstep with tauri-specta; no dual versions).

---

## Phase 2: Core (commands)

### T003 -- Read commands
**Traces to:** FR-1, FR-2, AC-1.1
**Status:** DONE
**Files:** `crates/luminos-app/src/tauri_commands.rs`

**TDD Cycle:**
1. **Red:**
   - [x] `get_current_settings_returns_state` -- returns `app_state.settings` clone.
   - [x] `get_frame_timings_zeroed_before_loop` -- zeroed summary when loop not running.
2. **Green:** Implemented `get_current_settings`/`get_frame_timings` as thin `#[tauri::command] #[specta::specta]` wrappers delegating to runtime-free inner fns (`*_inner`) that take the handle's component fields directly — so the logic is unit-testable without a Tauri runtime (constructing `State<LuminosHandle>` needs a live `AppHandle`).
3. **Refactor:** Inner fns extracted; `get_frame_timings_inner` is poison-tolerant (never errors — AC-1.1).

**Completion Notes:**
> Both read commands implemented + tested. `get_frame_timings` uses the per-field `Mutex<FrameTimingSummary>` slot (poison-tolerant clone). The inner-fn split is the key testability seam used by T004/T005 too.

---

### T004 -- Mutation commands (zoom/mode/toggle) + validation
**Traces to:** FR-3, FR-4, NFR-2, AC-1.2
**Status:** DONE
**Files:** `crates/luminos-app/src/tauri_commands.rs`

**TDD Cycle:**
1. **Red:**
   - [x] `set_zoom_level_clamps` -- 0.5→1.5, 50→20.
   - [x] `set_zoom_level_rejects_nan` -- NaN → `Err`.
   - [x] `set_zoom_level_wakes_loop` / `set_magnification_mode_wakes_loop` -- dirty flag set after the write.
   - [x] `set_magnification_mode_writes_state`.
   - [x] `toggle_magnification_returns_new_state`.
2. **Green:** Each mutation writes via `StateManager` (RCU on the shared Arc) then `notifier.notify_state_changed()`. NaN guard runs BEFORE the clamp (NFR-2 server-side validation). Rust param names kept exactly `level`/`mode` (the generated arg-object key is the camelCased param name → stays `level`/`mode`, matching `commands.test.ts`).
3. **Refactor:** Wake is explicit per command; no shared helper needed (each command's pre/post differs).

**Completion Notes:**
> Zoom clamps to [1.5,20] (via `StateManager::update_zoom_level`), NaN rejected with a clear `Err` message. Mode write uses the NEW `StateManager::set_magnification_mode` (T001). Toggle reads `is_active` back after flipping. All three set the dirty flag (asserted). 7 tests pass.

---

### T005 -- Persistence commands
**Traces to:** FR-5, AC-2.1
**Status:** DONE
**Files:** `crates/luminos-app/src/tauri_commands.rs`, `crates/luminos-core/src/config/manager.rs`

**TDD Cycle:**
1. **Red:**
   - [x] `save_settings_delegates_to_config` (temp-dir ConfigManager) -- persists current settings.
   - [x] `reset_settings_returns_defaults` / `reset_settings_wakes_loop` -- reset applied to state + returned + wake.
   - [x] `save_settings_config_none_errors` / `reset_settings_config_none_errors` -- `None` config → `Err("config unavailable")`, no panic.
2. **Green:** save/reset lock the `config` mutex, delegate to `ConfigManager::save`/`reset`; reset applies the defaults to live `AppState` via the NEW `StateManager::replace_settings` (T001) and wakes. `save` does NOT wake (it does not change `AppState`).
3. **Refactor:** Promoted `ConfigManager::load_from(&Path)` from private to `pub` (it was already the documented internal entry point) so the app's persistence tests can build a temp-dir-rooted manager without env coupling — a legitimate API addition (E12/E17 will want explicit-path construction too).

**Checkpoint:** All 7 command bodies unit-tested against per-field handle seams (13 tests, all pass).

**Completion Notes:**
> Persistence wired through `ConfigManager`. `CONFIG_UNAVAILABLE = "config unavailable"` constant matches the story-006 contract. `reset` applies defaults to the live state so the render loop + a subsequent `get_current_settings` agree. Made `ConfigManager::load_from` public (see Deviations).

---

## Phase 3: Integration (events, builder, capability)

### T006 -- Events + emission from the render loop
**Traces to:** FR-6, AC-2.2
**Status:** DONE
**Files:** `crates/luminos-app/src/events.rs`, `crates/luminos-app/src/app.rs`, `crates/luminos-app/tests/ipc_events.rs`

**TDD Cycle:**
1. **Red:** `ipc_hotkey_emits_zoom_changed_event` (subprocess) -- `xdotool ctrl+alt+equal` → `emit zoom_changed=3` logged beside the `.emit()` call.
2. **Green:** Defined `ZoomChangedEvent(f32)` / `ModeChangedEvent(MagnificationMode)` with the MANDATORY `#[tauri_specta(event_name = "zoom_changed"/"mode_changed")]`. Emission lives in the RENDER LOOP (`MainEventsCleared` arm), NOT the input thread (the `InputProcessingTask` thread has no `AppHandle`): `emit_state_events` tracks last `(zoom_bits, mode)` and emits on a delta via `app_handle`.
3. **Refactor:** First observation seeds `last` without emitting (no startup echo); emit is origin-agnostic (a UI-origin command also moves the same ArcSwap → a harmless idempotent echo).

**Completion Notes:**
> Both events exist with explicit wire names (without `event_name` the macro would kebab-case `ZoomChangedEvent` → `"zoom-changed-event"`, breaking the 006 contract — verified against the unpacked rc.25 macro source). **AD-5 deviation:** events emit from the loop on a delta (origin-agnostic), not exclusively on hotkey-origin — see Deviations. **Phase-0 caveat:** `dispatch_hotkey`'s `CycleMode` is a no-op (`luminos_core::hotkeys`), so `mode_changed` has NO engine-origin hotkey trigger in Phase 0 — the type/binding still must exist for the 006 cross-language contract. Subprocess emit test passes (1.7s).

---

### T007 -- `tauri-specta` Builder + bindings export
**Traces to:** FR-7, AC-3.1
**Status:** DONE
**Files:** `crates/luminos-app/src/ipc.rs`, `crates/luminos-app/src/app.rs`, `crates/luminos-app/src/main.rs`, `ui/src/ipc/bindings.ts`, `ui/src/ipc/events.ts`, `ui/src/ipc/commands.ts`

**TDD Cycle:**
1. **Red:** `bindings_export_smoke` -- exporting the handler produces bindings with all 7 commands + both events (wire names + camelCase keys), FrameTimingSummary camelCase, AppSettings snake_case.
2. **Green:** `build_ipc_handler()` collects the 7 commands + 2 events; wired `ipc.invoke_handler()` on the `tauri::Builder` and `ipc.mount_events(app)` in `.setup`. Debug export + a `--export-bindings` CLI seam both write to the manifest-anchored `../../ui/src/ipc/bindings.ts`. Default `ErrorHandlingMode::Result` (NOT Throw). Regenerated + committed `bindings.ts`; edited `events.ts` lines 22/34 to `events.zoomChanged`/`events.modeChanged`.
3. **Refactor:** Export path anchored to `CARGO_MANIFEST_DIR` (not process CWD) so it lands correctly regardless of launch dir; `semantic_types(enable_lossless_floats())` flattens `f32/f64` to plain `number` to match the 006 Zod schemas.

**Completion Notes:**
> See Deviations for the FOUR swap realities: (1) export path is `../../ui` not `../ui`, and must be CWD-independent (anchored to `CARGO_MANIFEST_DIR`); (2) generated events object keys are `zoomChanged`/`modeChanged` → the licensed `events.ts` 2-line edit; (3) the generated file does NOT export a named `Result` type → minimal `commands.ts` edit defining the envelope locally (HLP anticipates touching commands.ts); (4) `f32/f64` default to `number | null` → `enable_lossless_floats()` flattens to `number` to match Zod. `--export-bindings` is idempotent (proven) — the basis for the CI diff check (T009).

---

### T008 -- Capability file
**Traces to:** FR-8, AC-3.1
**Status:** DONE
**Files:** `crates/luminos-app/capabilities/default.json`

**TDD Cycle:**
1. **Red:** `capability_minimal` -- JSON grants only `core:default` + `core:event:default`; windows `["control-panel","overlay"]`; explicitly rejects `fs`/`http`/`shell:allow-open`.
2. **Green:** Extended the EXISTING `default.json` (story 001's `core:default` stub) — added `core:event:default` (so the webview can listen for the zoom/mode events) and the `overlay` window. Window labels are the REAL `control-panel`/`overlay` (NOT the stale `main` from DESIGN.md:118). `tauri.conf.json` already references the `default` capability — no change needed.
3. **Refactor:** —

**Checkpoint:** Commands + events + bindings + capability all wired; app builds with IPC (verified — no capability-validation failure at `generate_context!`).

**Completion Notes:**
> **shell:allow-open decision: DROPPED for Phase 0** (recorded as a deviation). No Phase-0 command/UI opens an external shell, and granting it requires registering `tauri-plugin-shell` (a new dependency) purely for an unused permission — against the supply-chain "pin only what's needed" rule. Deferred until a real shell-open need lands (a future story adds the plugin + permission together). Capability stays least-privilege: `core:default` + `core:event:default` only (RISK-020).

---

## Phase 4: Polish & Acceptance

### T009 -- CI bindings-up-to-date check + acceptance + AC matrix
**Traces to:** FR-7, All ACs
**Status:** DONE
**Files:** `.github/workflows/ci.yml`, `CLAUDE.md`, story docs

**Verification Checklist:**
- [x] AC-1.1 read commands → `get_current_settings_returns_state`, `get_frame_timings_zeroed_before_loop`
- [x] AC-1.2 mutation validation + write + wake → `set_zoom_level_clamps`, `set_zoom_level_rejects_nan`, `set_zoom_level_wakes_loop`, `set_magnification_mode_writes_state`, `set_magnification_mode_wakes_loop`, `toggle_magnification_returns_new_state`
- [x] AC-2.1 persistence commands (+ config-None handling) → `save_settings_delegates_to_config`, `reset_settings_returns_defaults`, `reset_settings_wakes_loop`, `save_settings_config_none_errors`, `reset_settings_config_none_errors`
- [x] AC-2.2 hotkey events emitted → `ipc_hotkey_emits_zoom_changed_event` (subprocess)
- [x] AC-3.1 bindings exported + CI `git diff --exit-code ui/src/ipc/bindings.ts` (D7) + capability minimal → `bindings_export_smoke`, `capability_minimal`, the 006 Vitest gate (70/70), the new CI step
- [x] `cargo fmt`/clippy clean (both gates); no `unwrap`/`expect` in production
- [x] CI step runs `--export-bindings` then `git diff --exit-code` and fails on stale bindings; mirrored into CLAUDE.md §8

**Completion Notes:**
> Added the bindings-diff CI step to the `test-app` job (`cargo run -p luminos-app --features tauri -- --export-bindings` then `git diff --exit-code ui/src/ipc/bindings.ts`) and mirrored it into CLAUDE.md's QA section (project rule). The `--export-bindings` seam exits without opening a window (no Xvfb needed) and is idempotent. **The cross-language gate is GREEN: all 70 story-006 Vitest tests pass against the regenerated bindings + edited events.ts/commands.ts** (the contract proof). `cargo deny check licenses advisories` + `cargo audit` both clean with the new specta dep.

#### AC → Test coverage matrix

| AC | Tests | Type |
|----|-------|------|
| AC-1.1 | `get_current_settings_returns_state`, `get_frame_timings_zeroed_before_loop` | unit (in-process) |
| AC-1.2 | `set_zoom_level_clamps`, `_rejects_nan`, `_wakes_loop`, `set_magnification_mode_writes_state`, `_wakes_loop`, `toggle_magnification_returns_new_state` | unit (in-process) |
| AC-2.1 | `save_settings_delegates_to_config`, `reset_settings_returns_defaults`, `_wakes_loop`, `save_settings_config_none_errors`, `reset_settings_config_none_errors` | unit (temp-dir ConfigManager) |
| AC-2.2 | `ipc_hotkey_emits_zoom_changed_event` | subprocess (Xvfb + xdotool) |
| AC-3.1 | `bindings_export_smoke`, `capability_minimal`, 006 Vitest (70/70), CI bindings-diff step | unit + cross-language gate + CI |

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | 2026-06-05 | Story-003 subprocess test `live_magnification_capture_path_wired` is environment-flaky on this dev box: under headless software GL the active per-frame X11-connect self-capture path (DC-12 cost) intermittently starves the marshaled `redraw=N` heartbeat during the 700ms sample window (`start=7,end=7`). NOT caused by story 005 — `emit_state_events` is a lock-free ArcSwap read with no I/O. Passes ~2/3 in isolation; full `ci`-profile run reports `60/60 passed (1 flaky)` via the profile's `retries=2`. | Tolerated via the `ci` profile retries; flagged for a story-003/DC-12 follow-up (cache the capture's X11 connection instead of per-frame `connect`). | OPEN (pre-existing, not story-005) |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| T001 | `StateManager::replace_settings` takes `&AppSettings`, not `AppSettings` (DESIGN/IMPLEMENTATION_NOTES showed by-value) | clippy `needless_pass_by_value` under `-W clippy::pedantic` (the test-app + workspace gates run pedantic). By-ref avoids an extra clone at the call site (`reset_settings` already owns and returns the defaults). Semantically identical — the rcu closure clones inside either way. |
| T002 | `luminos-platform::ScreenCapture` crate-root re-export NOT added (DC-5 / notes list it) | No story-005 code needs it (commands use only `AppSettings`/`MagnificationMode`/`FrameTimingSummary`); the app already uses the full `luminos_platform::traits::ScreenCapture` path. Keeps `luminos-platform` untouched (it must stay `tauri`/`winit`-free). |
| T002 | `specta` added as an optional dep to `luminos-app` (under the `tauri` feature), not just the engine crates | `#[specta::specta]` on commands, `#[derive(specta::Type)]` on events, and `collect_commands!`/`collect_events!` need `specta` directly in scope in the app crate. |
| T006 | Events emit from the RENDER LOOP on a `(zoom,mode)` delta (origin-agnostic), not exclusively on hotkey origin (AD-5 said hotkey-origin only) | The `InputProcessingTask` thread has no `AppHandle`; only the loop can reach `app_handle`. Reading `AppState` cannot distinguish hotkey-origin from UI-origin, so the panel may get a redundant echo of a value it just set — idempotent (the store already holds it). Origin-tagging is deferred. **Phase-0 caveat:** `CycleMode` is a no-op, so `mode_changed` has no engine-origin trigger yet; the event still exists for the 006 contract. |
| T007 | Export path is `../../ui/...` (DESIGN said `../ui/...`) AND anchored to `CARGO_MANIFEST_DIR` (not a relative literal) | The crate is two levels under the repo root. A relative literal resolves against the process CWD (which differs for `cargo run`, the binary, `tauri dev`, CI), so it was anchored to the compile-time manifest dir to land deterministically. |
| T007 | Added a `--export-bindings` CLI seam in `main.rs` (not in DESIGN) | The debug auto-export runs only at app *runtime* (needs Xvfb/webview). CI needs a windowless, deterministic regenerate-and-exit path to diff the committed bindings — `--export-bindings` provides it. |
| T007 | `events.ts` lines 22/34 edited (`events.zoomChangedEvent` → `events.zoomChanged`) | HLP line 260 licenses this: tauri-specta names the generated event object keys by the camelCased `event_name` (`zoomChanged`/`modeChanged`), not the struct-ident-derived `zoomChangedEvent` the 006 placeholder assumed. |
| T007 | `commands.ts` edited to define `Result<T,E>` locally instead of importing it from `./bindings` | The generated `bindings.ts` inlines the `{status:"ok"|"error"}` envelope as each command's return type and exports NO named `Result`. The wrapper still needs the type for its `unwrap` helper. HLP §Integration Points anticipates touching `commands.ts` on a shape mismatch. |
| T007 | `semantic_types(Configuration::default().enable_lossless_floats())` on the Builder (not in DESIGN) | specta-typescript maps `f32`/`f64` to `number | null` by default (JSON NaN/Infinity → null). The 006 Zod schemas use `z.number()` and `onZoomChanged(level: number)`, so the default broke `tsc`. The engine never emits NaN/Infinity over IPC (zoom NaN rejected server-side; timings finite), so flattening to `number` is accurate and keeps the cross-language contract green. |
| T007 | Added `ui/.prettierignore` excluding `src/ipc/bindings.ts` | Prettier wants to reformat the generated file; if `pnpm format` ran on it, the committed file would diverge from a fresh `cargo` export and break the CI diff check. Mirrors eslint's existing `globalIgnores` of the file. The DoD lint gate (`eslint .`) already ignores it. |
| T008 | `shell:allow-open` DROPPED for Phase 0 (DESIGN/STORY/HLP listed it) | No Phase-0 command/UI opens an external shell; granting it requires registering `tauri-plugin-shell` (a new dependency) for an unused permission, against the supply-chain "pin only what's needed" rule. Deferred — a future story adds the plugin + permission together. Capability stays least-privilege (`core:default` + `core:event:default`). |
| T005 | `ConfigManager::load_from(&Path)` promoted from private to `pub` | It was already the documented internal entry point (default-on-missing, recover-on-corrupt). Making it public lets the app's persistence tests build a temp-dir-rooted manager without env coupling, and parallels the explicit-path construction macOS/Windows branches (E12/E17) will need. |
