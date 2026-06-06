# Story 005 — Implementation Notes (lead briefing, 2026-06-05)

Source-verified against worktree HEAD (001/002/003/004/006 done) + the unpacked **tauri-specta-2.0.0-rc.25**
/ **tauri-specta-macros-2.0.0-rc.25** crate source. SUPERSEDES stale DESIGN parts — log conflicts in
`SUBTASKS.md → Deviations from Design`. The cross-language round-trip (generated `bindings.ts` ↔ story-006
Zod schemas) is the HARD GATE: the closing step runs the 006 Vitest suite against the regenerated file.

## ⚠️ CWD (every Bash block)
Bare shell = `/home/renatorro/Development/luminos` (branch main — 001/002/003 absent). Work in the worktree:
`cd /home/renatorro/Development/luminos/.claude/worktrees/epic+e04-control-panel`. Verify branch before commit.

## A. The 7 commands (new `crates/luminos-app/src/tauri_commands.rs`; `#[tauri::command] #[specta::specta]`, `State<'_, LuminosHandle>` LAST param)
LuminosHandle is complete today (`handle.rs:23-51`): app_state, config `Arc<Mutex<Option<ConfigManager>>>`,
notifier, frame_timings `Arc<Mutex<FrameTimingSummary>>`, app `AppHandle`, accessor `frame_timings()` (clone, poison-tolerant).
1. `get_current_settings(h) -> Result<AppSettings, String>` = `h.app_state.load().settings.clone()`. no wake.
2. `get_frame_timings(h) -> Result<FrameTimingSummary, String>` = `h.frame_timings()` (USE the accessor, NOT a raw lock; never error on poison — AC-1.1). no wake.
3. `set_zoom_level(level: f32, h) -> Result<(), String>` — **NaN guard first** (`if level.is_nan() {Err}`) then `StateManager::new(Arc::clone(&h.app_state)).update_zoom_level(level)` (clamps [1.5,20] internally). **wake** `h.notifier.notify_state_changed()`.
4. `set_magnification_mode(mode: MagnificationMode, h) -> Result<(), String>` → **NEW seam** `StateManager::set_magnification_mode(mode)`. **wake**.
5. `toggle_magnification(h) -> Result<bool, String>` = `StateManager::toggle_magnification()` then read back `h.app_state.load().is_active`. **wake**.
6. `save_settings(h) -> Result<(), String>` = lock `h.config`; `None`→`Err("config unavailable")`; `cm.save(&settings)`. no wake.
7. `reset_settings(h) -> Result<AppSettings, String>` = lock `h.config`; `cm.reset()` → defaults; then **NEW seam** `StateManager::replace_settings(defaults.clone())`. **wake**.
Keep Rust param names exactly `level` and `mode` (the generated arg-object key is the camelCased param name → stays `level`/`mode`, matching `commands.test.ts`). No `unwrap`/`expect` (clippy `-D` in test-app job).

### NEW StateManager methods (add to `luminos-core/src/state_manager.rs`, T001, mirror the existing RCU pattern)
```rust
pub fn set_magnification_mode(&self, mode: MagnificationMode) {
    self.state.rcu(|c| { let mut s = (**c).clone(); s.settings.magnification.mode = mode; s });
}
pub fn replace_settings(&self, settings: AppSettings) {
    self.state.rcu(|c| { let mut s = (**c).clone(); s.settings = settings.clone(); s });  // clone inside: rcu may re-run
}
```

## B. The 2 events (new `crates/luminos-app/src/events.rs`)
```rust
#[derive(Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
#[tauri_specta(event_name = "zoom_changed")]      // ★ MANDATORY — without it the macro kebab-cases to "zoom-changed-event"
pub struct ZoomChangedEvent(pub f32);
#[derive(Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
#[tauri_specta(event_name = "mode_changed")]
pub struct ModeChangedEvent(pub luminos_types::MagnificationMode);
```
**Emission — from the RENDER LOOP, not the input thread** (the InputProcessingTask thread has no AppHandle).
In the `MainEventsCleared` arm (`app.rs:274`, which already reads AppState each tick and keeps `state_log_last`):
track last `(zoom_bits, mode)`; on a delta, `ZoomChangedEvent(new_zoom).emit(app_handle)` /
`ModeChangedEvent(new_mode).emit(app_handle)`. This is origin-agnostic — accept the redundant echo after a UI
command (idempotent: the store already set that value). **RECORD the AD-5 deviation** + the fact that **no
Phase-0 hotkey changes mode** (`dispatch_hotkey`'s `CycleMode` is a no-op, `hotkeys.rs:140`), so `mode_changed`
has no engine-origin trigger yet — the event type/binding still must exist for the 006 contract. T006 subprocess
test: logged-emit assertion (`log::info!("emit zoom_changed={n}")` beside `.emit`) under `xdotool ctrl+alt+equal`;
the live webview-listener assertion is story 007's tauri-driver concern.

## C. specta::Type additions (DC-5) — the contract gate
Add to root `[workspace.dependencies]`: `specta = { version = "=2.0.0-rc.25", features = ["derive"] }`
(lockstep with tauri-specta rc.25; currently only transitive). Add `specta = { workspace = true }` as a
**normal non-optional, non-feature-gated** dep to the engine crates (they have no `tauri` feature; the derive
must compile so `cargo build --workspace --exclude luminos-app` passes). Add `serde` too where missing (gpu).
Derive `#[derive(specta::Type)]` on:
- `luminos-types/src/state.rs`: `MagnificationMode`, `TrackingMode`, `ColorFilterType` (+ the enums in other
  luminos-types modules referenced by AppSettings: `DockEdge`, `GpuPreference`, `InterpolationMode`, `LensShape`,
  `PresentMode` — derive wherever defined). (NOT TtsStatus — not in AppSettings.)
- `luminos-core/src/config/schema.rs`: `AppSettings`, `MagnificationSettings`, `ColorFilterConfig`,
  `CursorConfig`, `SpeechSettings`, `KeyBinding`, `ModelVariant`, `HotkeyAction`, `ModifierKey`.
- `luminos-gpu/src/frame_timings.rs`: `FrameTimingSummary` — currently only `Debug,Clone,PartialEq`. Add
  `serde::Serialize, serde::Deserialize, specta::Type` **+ `#[serde(rename_all = "camelCase")]`**.

**rename_all asymmetry (THE gate):** `FrameTimingSummary` ONLY → camelCase (`averageMs/p99Ms/minMs/maxMs/targetFps`).
`AppSettings` + all sub-structs STAY snake_case (NO rename — `zoom_level`, `color_filter`, …). Enums STAY bare
PascalCase (serde default; `"FullScreen"` etc. — do NOT add `#[specta(rename_all)]`). specta mirrors serde's repr.

## D. Bindings generation + the events.ts edit (new `crates/luminos-app/src/ipc.rs`)
```rust
pub(crate) fn build_ipc_handler() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![/* the 7, by path */])
        .events(tauri_specta::collect_events![crate::events::ZoomChangedEvent, crate::events::ModeChangedEvent])
    // ErrorHandlingMode::Result is the DEFAULT (matches the {status:"ok"|"error"} envelope) — do NOT set Throw.
}
```
Wire in `app.rs::run`: `.invoke_handler(ipc.invoke_handler())` on the `tauri::Builder`, and inside `.setup`
`ipc.mount_events(app)`. Export in debug:
`#[cfg(debug_assertions)] ipc.export(specta_typescript::Typescript::default(), "../../ui/src/ipc/bindings.ts")?;`
**Path is `../../ui/...`** (crate is two levels under repo root; DESIGN's `../ui` is wrong). `tauri-specta` +
`specta-typescript` are already optional deps under the app's `tauri` feature — no new app deps.

**★ Generated shape is NOT a byte-match for the placeholder — plan the swap:**
- Commands: clean (method key = camelCased fn name; wire name = snake fn name verbatim; arg key = `level`/`mode`). Matches `commands.test.ts`.
- Events: generated object key = `events.zoomChanged` / `events.modeChanged` (camelCase of the event_name), wire
  name = `zoom_changed`/`mode_changed`. The placeholder used `events.zoomChangedEvent`. **EDIT `ui/src/ipc/events.ts`
  lines ~22/34** to `events.zoomChanged` / `events.modeChanged` (HLP line 260 explicitly licenses this 2-line edit).
- The generated file is self-contained (inlines AppSettings/FrameTimingSummary/enum types, imports `invoke` +
  runtime helpers). `commands.ts`/`events.ts` consume only `commands`/`events`/`Result` from `./bindings` — fine.
- Regenerate + COMMIT `ui/src/ipc/bindings.ts`.

## E. Capability file — extend existing `crates/luminos-app/capabilities/default.json` (DC-8; do NOT recreate)
Target: `"windows": ["control-panel", "overlay"]`, `"permissions": ["core:default", "core:event:default",
"shell:allow-open"]`. ⚠️ **`shell:allow-open` requires the `tauri-plugin-shell` plugin** registered (`.plugin(...)`
+ dep) or capability validation FAILS at build. RESOLVE during impl: either add `tauri-plugin-shell` (pin exact,
add to PINNED_VERSIONS) OR drop `shell:allow-open` and record a deviation (defer until a shell need lands). The
real control-panel window label is `control-panel` (tauri.conf.json), NOT `main` — DESIGN.md:118 (`["main",...]`)
is stale; the `capability_minimal` test must use `control-panel`.

## F. DESIGN staleness (apply + log)
1. Re-exports for gpu/core ALREADY exist (`luminos-gpu/src/lib.rs:22`, `luminos-core/src/lib.rs:16`) — don't re-add.
2. `frame_timings` slot + accessors ALREADY on the handle (`handle.rs:39,56,98`) + the loop publishes it
   (`app.rs:293`). **T001 reduces to just the 2 new StateManager methods.**
3. AppState is nested: zoom `settings.magnification.zoom_level`, mode `settings.magnification.mode`, `is_active` top-level.
4. Capability window label `control-panel` not `main` (E).
5. Export path `../../ui/...` not `../ui/...` (D).
6. Event naming mechanics (event_name attr + kebab/camel) — DESIGN's biggest gap (B + D).
7. Commands swap clean; events need the 2-line `events.ts` edit (not a pure one-file drop-in).

## G. Subtask sequence (~9) + verification
T001 StateManager `set_magnification_mode`+`replace_settings` (pure core, TDD). T002 specta prerequisite (the
big blast radius: workspace pin + per-crate deps + all derives + FrameTimingSummary camelCase; assert
`cargo build --workspace --exclude luminos-app` passes; scaffold the 3 new app modules feature-gated in lib.rs).
T003 read commands (inner-fn extraction, no async runtime). T004 mutation commands (NaN/clamp/mode/toggle +
dirty-flag assert). T005 persistence commands (temp-dir ConfigManager; config-None→Err). **Checkpoint: 7 bodies
unit-tested.** T006 events + loop-delta emission (logged-emit subprocess test). T007 Builder + export +
`bindings_export_smoke` test; regenerate+commit bindings.ts; EDIT events.ts lines 22/34. T008 capability file +
resolve shell-plugin question. **Checkpoint: app builds with IPC.** T009 CI bindings check
(`git diff --exit-code ui/src/ipc/bindings.ts` in the test-app job) + **run the 006 Vitest gate** (`corepack pnpm
--dir ui test` → all 70 pass against regenerated bindings + edited events.ts) + AC matrix.
Three verification layers: in-process command unit tests (bulk of AC coverage); bindings-gen diff check (D7/drift);
the 006 Vitest suite against the generated file (the cross-language round-trip proof).
