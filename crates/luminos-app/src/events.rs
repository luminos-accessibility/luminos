//! Engine -> control-panel `tauri-specta` events (story E04/005).
//!
//! These are the **panel-sync** channel (AD-5): emitted to the webview when the
//! engine changes zoom/mode out-of-band (e.g. a global hotkey), so the Zustand
//! store stays in sync. They are distinct from the engine *wake* channel
//! (`LuminosEvent::StateChanged` over the tao loop, AD-2).
//!
//! ## `event_name` is MANDATORY
//!
//! The `#[derive(tauri_specta::Event)]` macro derives the wire name from the
//! struct identifier kebab-cased when no `event_name` is given — so
//! `ZoomChangedEvent` would become `"zoom-changed-event"`, breaking story 006's
//! contract (which listens for `"zoom_changed"`). The explicit
//! `#[tauri_specta(event_name = "...")]` pins the wire name to the `snake_case`
//! identifier the frontend expects.

/// Emitted when the engine changes the zoom level (hotkey-origin). Payload is
/// the new zoom multiplier. Wire name: `zoom_changed`; generated TS object key:
/// `events.zoomChanged`.
#[derive(Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
#[tauri_specta(event_name = "zoom_changed")]
pub struct ZoomChangedEvent(pub f32);

/// Emitted when the engine changes the magnification mode (hotkey-origin).
/// Payload is the new mode. Wire name: `mode_changed`; generated TS object key:
/// `events.modeChanged`.
///
/// Phase-0 caveat: no Phase-0 hotkey changes mode yet (`dispatch_hotkey`'s
/// `CycleMode` is a no-op, `luminos_core::hotkeys`), so this event has no
/// engine-origin trigger in Phase 0 — the type/binding still must exist for the
/// story-006 cross-language contract (AD-5 deviation, recorded in SUBTASKS).
#[derive(Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
#[tauri_specta(event_name = "mode_changed")]
pub struct ModeChangedEvent(pub luminos_types::MagnificationMode);
