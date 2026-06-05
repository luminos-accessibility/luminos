# Design: Story E04/005 -- IPC Command Layer & tauri-specta Bindings

**Story:** [STORY.md](./STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** principal-architect
**Risk Refs:** RISK-020 (webview attack surface — accepted, mitigated by minimal capability + server-side validation), RISK-022 (license — no new copyleft deps)

---

## Overview

Implement the seven Phase-0 Tauri commands, two engine→panel events, the `tauri-specta` `Builder` (commands + events + TS export), and the capability file (extending story 001's `core:default` stub). Commands are thin: validate → mutate via `StateManager`/`ConfigManager` → wake via `AppNotifier` → return serde types. Reconciles to real engine types (Integration Points table in the epic plan).

## Architecture

### Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| **`luminos-types`, `luminos-core`, `luminos-gpu`** | **Modified (CRITICAL prerequisite)** | **No engine type implements `specta::Type` today** — `#[specta::specta]` requires every IPC arg/return type to. Add `specta` (pinned) and `#[derive(specta::Type)]` to all IPC-reachable types: `MagnificationMode` (types), `AppSettings` + sub-structs (`MagnificationSettings`, `ColorFilterConfig`, `CursorConfig`, `SpeechSettings`, `KeyBinding`, enums) (core), and `FrameTimingSummary` (gpu — also add `serde::{Serialize,Deserialize}` + `#[serde(rename_all="camelCase")]`, it currently has neither). Also add `pub use` re-exports in `luminos-gpu`/`luminos-platform` lib.rs (`Renderer`, `FrameTimingSummary`, `InterpolationMethod`, `ScreenCapture`) so ergonomic paths resolve. |
| `luminos-app/src/tauri_commands.rs` | New | 7 `#[tauri::command] #[specta::specta]` fns. |
| `luminos-app/src/events.rs` | New | `ZoomChangedEvent(f32)`, `ModeChangedEvent(MagnificationMode)` (`#[tauri_specta::Event]`). |
| `luminos-app/src/ipc.rs` | New | `build_ipc_handler() -> tauri_specta::Builder<tauri::Wry>` (commands + events + debug export). |
| `luminos-app/src/main.rs` | Modified | Use the Builder's invoke handler + `mount_events`; debug export to `ui/src/ipc/bindings.ts`. |
| `luminos-app/capabilities/default.json` | Modified/Extended | Created as a `core:default` stub in story 001 (HLP DC-8); this story extends it to `core:default` + `core:event:default` + `shell:allow-open`. |
| `luminos-app/src/main.rs` (input path, story 003) | Modified | Emit `ZoomChangedEvent`/`ModeChangedEvent` on hotkey-origin state change. |
| `luminos-app/src/handle.rs` | Modified | Add `frame_timings: Arc<Mutex<FrameTimingSummary>>` (updated by loop) for `get_frame_timings`. |

### Data Flow (set_zoom_level example)
`invoke('setZoomLevel', {level})` → Tauri async cmd → `level.clamp` is done inside `StateManager::update_zoom_level` (range [1.5,20]) on `handle.app_state` → `handle.notifier.notify_state_changed()` (dirty flag) → render loop reads new zoom next frame → `Ok(())`. (No event emitted for UI-originated changes — the UI already knows; events are only for hotkey-origin changes, AD-5.)

## API Design

```rust
// luminos-app/src/tauri_commands.rs  (real engine types; State is the last param)
use tauri::State; use crate::handle::LuminosHandle;
use luminos_core::AppSettings;          // re-exported from config::schema
use luminos_types::MagnificationMode;
use luminos_gpu::FrameTimingSummary;

#[tauri::command] #[specta::specta]
pub(crate) async fn get_current_settings(h: State<'_, LuminosHandle>) -> Result<AppSettings, String> {
    Ok(h.app_state.load().settings.clone())
}

#[tauri::command] #[specta::specta]
pub(crate) async fn get_frame_timings(h: State<'_, LuminosHandle>) -> Result<FrameTimingSummary, String> {
    Ok(h.frame_timings.lock().map_err(|e| e.to_string())?.clone())
}

#[tauri::command] #[specta::specta]
pub(crate) async fn set_zoom_level(level: f32, h: State<'_, LuminosHandle>) -> Result<(), String> {
    StateManager::new(h.app_state.clone()).update_zoom_level(level);   // clamps [1.5,20] internally
    h.notifier.notify_state_changed();
    Ok(())
}

#[tauri::command] #[specta::specta]
pub(crate) async fn set_magnification_mode(mode: MagnificationMode, h: State<'_, LuminosHandle>) -> Result<(), String> {
    // mode is a validated enum by deserialization; write via StateManager (RCU on settings.magnification.mode)
    StateManager::new(h.app_state.clone()).set_magnification_mode(mode);
    h.notifier.notify_state_changed();
    Ok(())
}

#[tauri::command] #[specta::specta]
pub(crate) async fn toggle_magnification(h: State<'_, LuminosHandle>) -> Result<bool, String> {
    let sm = StateManager::new(h.app_state.clone()); sm.toggle_magnification();
    h.notifier.notify_state_changed();
    Ok(h.app_state.load().is_active)
}

#[tauri::command] #[specta::specta]
pub(crate) async fn save_settings(h: State<'_, LuminosHandle>) -> Result<(), String> {
    let settings = h.app_state.load().settings.clone();
    let mut guard = h.config.lock().map_err(|e| e.to_string())?;
    match guard.as_mut() {
        Some(cm) => cm.save(&settings).map_err(|e| e.to_string()),
        None => Err("config unavailable".into()),
    }
}

#[tauri::command] #[specta::specta]
pub(crate) async fn reset_settings(h: State<'_, LuminosHandle>) -> Result<AppSettings, String> {
    let defaults = { let mut g = h.config.lock().map_err(|e| e.to_string())?;
        g.as_mut().ok_or("config unavailable")?.reset().map_err(|e| e.to_string())? };
    StateManager::new(h.app_state.clone()).replace_settings(defaults.clone());  // apply to ArcSwap
    h.notifier.notify_state_changed();
    Ok(defaults)
}
```

> `StateManager::new(h.app_state.clone())` / `set_magnification_mode` / `replace_settings`: `StateManager` (E3) currently has `update_zoom_level`/`toggle_magnification`/`reset_zoom`/`update_mouse_position`. This story adds `set_magnification_mode(MagnificationMode)` and `replace_settings(AppSettings)` to `StateManager` (RCU writes), or constructs a `StateManager` view over `h.app_state`. **Small additive change to `luminos-core::state_manager`** — listed as affected.

```rust
// luminos-app/src/events.rs  (Deserialize required for Event::listen)
#[derive(Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
pub struct ZoomChangedEvent(pub f32);
#[derive(Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
pub struct ModeChangedEvent(pub luminos_types::MagnificationMode);

// luminos-app/src/ipc.rs
pub(crate) fn build_ipc_handler() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            tauri_commands::get_current_settings, tauri_commands::set_zoom_level,
            tauri_commands::set_magnification_mode, tauri_commands::toggle_magnification,
            tauri_commands::get_frame_timings, tauri_commands::save_settings,
            tauri_commands::reset_settings,
        ])
        .events(tauri_specta::collect_events![events::ZoomChangedEvent, events::ModeChangedEvent])
}
```

```jsonc
// luminos-app/capabilities/default.json
{ "identifier": "default", "windows": ["main", "overlay"],
  "permissions": ["core:default", "core:event:default", "shell:allow-open"] }
```

**main.rs glue:** build the handler; `#[cfg(debug_assertions)] builder.export(Typescript::default(), "../ui/src/ipc/bindings.ts")?;` then `tauri::Builder::default().invoke_handler(builder.invoke_handler()).setup(move |app| { builder.mount_events(app); … })`.

## Error Handling
- Commands return `Result<T, String>` (Tauri convention). Internal errors (`ConfigError`, lock poisoning) → `.map_err(|e| e.to_string())` + `log::error!`. No `unwrap`/`expect`.
- Enum/range validation: `MagnificationMode` validated by deserialization; `f32` zoom clamped in `StateManager`. Reject NaN zoom (clamp treats NaN — guard explicitly: `if level.is_nan() { return Err(...) }`).

## Platform Considerations
- IPC is platform-agnostic (Tauri). Capability `windows` includes both `main` and `overlay` labels. No platform branches.

## Testing Strategy

### Unit / in-process (no full Tauri runtime — test command bodies via a constructed `LuminosHandle` + a seam, or extract pure logic)
- `set_zoom_level_clamps` — calling the command logic with 0.5/50 clamps to 1.5/20 in state.
- `set_zoom_level_rejects_nan` — NaN → `Err`.
- `toggle_magnification_returns_new_state` — flips `is_active`, returns it.
- `get_current_settings_returns_state` — mirrors `app_state.settings`.
- `save_settings_delegates_to_config` / `reset_settings_returns_defaults` — via a temp-dir `ConfigManager` in the handle.
- `get_frame_timings_zeroed_before_loop` — returns zeroed summary when loop not running.
- `set_magnification_mode_writes_state` — mode reflected in `AppState`.
- Pure logic extracted to `fn`s callable without the async Tauri attribute where possible (test the inner logic; the `#[tauri::command]` wrapper is thin).

### Generation / build
- `bindings_up_to_date` (CI) — run the debug export, `git diff --exit-code ui/src/ipc/bindings.ts` (fail if stale). (D7)
- `capability_minimal` — assert the capability JSON contains only the three permissions (a test or a CI grep).

### Events
- `hotkey_emits_zoom_changed` (subprocess, with story 003) — `xdotool` zoom hotkey → assert a `ZoomChangedEvent` reaches a test webview listener (or a logged emit). (May be validated more fully by story 007 `tauri-driver`.)

### Acceptance Tests

| AC | Test Type | Verification |
|----|-----------|--------------|
| AC-1.1 | Unit | `get_current_settings`/`get_frame_timings` return correct types/values (zeroed timings pre-loop). |
| AC-1.2 | Unit | zoom clamp + NaN reject; mode write; toggle returns new state; dirty flag set. |
| AC-2.1 | Unit (temp ConfigManager) | save persists; reset → defaults applied + returned. |
| AC-2.2 | Subprocess | hotkey → `ZoomChangedEvent`/`ModeChangedEvent` emitted. |
| AC-3.1 | CI build | `bindings.ts` exported + up-to-date check; capability has only the 3 permissions. |

## Performance Targets
- Command latency dominated by IPC round-trip (sub-ms engine work); no long locks (NFR-1).

## Security Considerations
- Minimal capability (no `fs`/`http`); all config I/O via Rust (RISK-020 mitigation). Server-side validation of every input (NFR-2). `ModeChangedEvent`/`ZoomChangedEvent` are typed, emitted only from trusted Rust paths.

## Alternatives Considered
1. **Manual TS types instead of `tauri-specta`.** Rejected as primary — generation keeps types in lock-step (D7); manual `ui/src/types/ipc-manual.ts` retained only as a fallback for any unsupported type (doc-05 §2.2).
2. **Emit events for UI-originated changes too.** Rejected — causes echo/loops; events are hotkey-origin only (AD-5); the UI updates its own store optimistically (story 006).
3. **Pass zoom clamp responsibility to the UI.** Rejected — never trust the webview; clamp server-side (NFR-2).
