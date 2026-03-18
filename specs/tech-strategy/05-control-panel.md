# 05 -- Control Panel

**Status:** DRAFT v1.1 (post audit review)
**Date:** 2026-03-16
**Audience:** Frontend engineers, AI agents implementing the control panel UI and IPC layer
**Source Documents:** [Product Strategy](../PRODUCT_STRATEGY.md) (v1.3, Sections 5, 7, 8), [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) (FINAL, Section 3), [System Architecture](./01-system-architecture.md) (Sections 3.3, 4.6, 4.7, 5.4, 6.2, 7.1, 9.4), [Platform Abstraction](./02-platform-abstraction.md), [Rendering Pipeline](./03-rendering-pipeline.md) (Sections 8.2, 8.3, 9.1), [TTS Pipeline](./04-tts-pipeline.md) (Sections 8.5, 13)

---

## 1. Overview

### 1.1 Purpose

This document defines the control panel subsystem: the Tauri 2.0 webview window that provides the user interface for configuring and controlling Luminos. It is the engineering specification for everything TypeScript and React in the project.

This document answers: **How does the settings UI communicate with the Rust engine, what does the UI look like architecturally, and how is application state managed across the IPC boundary?**

### 1.2 Scope

This document covers:
- The complete Tauri IPC command and event catalogue (typed)
- TypeScript type system: Zod schemas mirroring Rust serde types
- IPC layer implementation: Rust command handlers and TypeScript wrappers
- React component architecture, routing, and state management (Zustand)
- Settings schema: all configurable values, validation, and defaults
- Profile management: creation, import/export, built-in condition-based presets
- Voice selection UI: voice list, model loading state, TTS status indicator
- Performance and diagnostics panel: frame timing display, GPU config, degradation warnings
- Control panel startup and settings hydration sequence
- Accessibility requirements for the control panel itself
- Testing strategy for TypeScript/React code and IPC integration
- Module organization for the `ui/` directory and `luminos-app/src/tauri_commands.rs`

This document does NOT cover:
- Magnification rendering or GPU shader details (see [03 -- Rendering Pipeline](./03-rendering-pipeline.md))
- TTS pipeline internals (see [04 -- TTS Pipeline](./04-tts-pipeline.md))
- Platform abstraction trait definitions (see [02 -- Platform Abstraction](./02-platform-abstraction.md))
- Build, packaging, or distribution (see [08 -- Build and Distribution](./08-build-and-distribution.md) (planned))

### 1.3 Phase Attribution

The control panel is introduced in Phase 0 as a minimal shell and grows with each phase:

| Phase | Control Panel Milestone |
|-------|------------------------|
| **Phase 0** | Basic Tauri shell. Zoom level slider, magnification mode selector. `get_current_settings`, `set_zoom_level`, `set_magnification_mode` IPC commands. System tray icon. |
| **Phase 1** | Color filter controls, cursor enhancement config, lens/docked mode controls, frame rate target, settings persistence, basic profile save/load. |
| **Phase 2** | Full TTS controls: voice selector, speech rate/volume sliders, TTS status indicator, model loading progress, espeak-ng availability warning. Keybinding configuration page. |
| **Phase 3** | Condition-based profile presets (AMD, Glaucoma, etc.), GPU preference selector. Diagnostics page gains a historical frame timing chart and system info panel. Note: the basic `get_frame_timings` command and a simple P99 readout are available from Phase 0; the full diagnostics page with polling chart and `get_system_info` is Phase 3. |
| **Phase 4** | Application-specific profiles, import/export, enterprise configuration, i18n UI. |

### 1.4 Relationship to Other Documents

```
01-system-architecture.md   -- Defines IPC layer role, settings data flow, startup sequence
    |
    v
05-control-panel.md (this)  -- Full IPC catalogue, TypeScript types, React architecture
    |
    v
Implementation stories       -- Per-feature UI stories (STORY.md / DESIGN.md / SUBTASKS.md)
```

The control panel communicates exclusively through the IPC boundary defined in this document. It never imports Rust types directly. All shared state passes through typed Tauri commands and events.

---

## 2. IPC Architecture

### 2.1 Communication Model

The control panel and the Rust core engine communicate via Tauri 2.0's typed IPC system. Two patterns are used:

| Pattern | Direction | Mechanism | Use Case |
|---------|-----------|-----------|----------|
| **Command** | Panel → Engine | `invoke('command_name', args)` | User changes a setting; panel requests current state |
| **Event** | Engine → Panel | `emit('event_name', payload)` / `listen(...)` | Engine notifies panel of state changes (hotkeys, TTS status, performance) |

**Key constraints** (from [01 -- System Architecture](./01-system-architecture.md), Section 4.7):
- Commands run on Tauri's async tokio runtime, never on the render thread.
- Commands must not acquire long-lived locks that would stall the render thread.
- All command parameters and return values are `serde`-serializable.
- Events are fire-and-forget; the panel must tolerate missing events (e.g., if it is minimized).

**EventLoopProxy integration:** When a command modifies shared app state (e.g., `set_zoom_level`), the Tauri IPC handler writes to the shared `ArcSwap<AppState>` and then sends a custom event to the winit event loop via `EventLoopProxy`. This ensures the render thread sees the change on the very next frame without polling. See [01 -- System Architecture](./01-system-architecture.md), Section 6.5.

```
Control Panel (TypeScript)
    |
    | invoke('set_zoom_level', { level: 5.0 })
    v
Tauri IPC Thread (Rust async fn)
    |
    +---> writes to ArcSwap<AppState>          (render thread reads this lock-free, every frame)
    +---> EventLoopProxy::send_event(...)       (wakes winit loop for immediate redraw)
    +---> returns Ok(()) to TypeScript
```

### 2.2 Type Safety Strategy

Keeping TypeScript types in sync with Rust serde types manually is error-prone. Luminos uses **`tauri-specta`** (v2, MIT) to generate TypeScript type bindings automatically from Rust types.

`tauri-specta` inspects `#[tauri::command]` functions and their parameter/return types at build time and emits a TypeScript file (`ui/src/ipc/bindings.ts`) containing:
- Fully typed `invoke` wrappers for every command
- TypeScript union types and interfaces for every Rust enum and struct used in IPC
- Event payload type annotations

This file is re-generated whenever `luminos-app` is rebuilt and is committed to the repository so the UI can be developed without a Rust build.

```toml
# luminos-app/Cargo.toml
[dependencies]
tauri-specta = { version = "2", features = ["derive", "typescript"] }
specta-typescript = "0.0.9"
```

In `tauri-specta` v2 the `Builder` object is **both** the invoke handler and the codegen source. It must be a regular `[dependencies]` entry (not `[dev-dependencies]`) because `builder.invoke_handler()` runs in the main binary, not only in tests. The `#[cfg(debug_assertions)]` guard on the export call is the correct mechanism to prevent regeneration in release builds.

The `Builder` is constructed once at startup, used to register the invoke handler at runtime, and — in debug builds only — additionally exports the TypeScript bindings file:

```rust
// luminos-app/src/main.rs
use tauri_specta::{collect_commands, collect_events, Builder};

fn build_ipc_handler() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            tauri_commands::get_current_settings,
            tauri_commands::set_zoom_level,
            // ... full list mirrors Section 2.3
        ])
        .events(collect_events![
            // Rust event types defined in luminos-app/src/events.rs (see §4.1)
            events::SettingsChangedEvent,
            events::TtsStatusChangedEvent,
            events::ZoomChangedEvent,
            events::ModeChangedEvent,
            events::VoiceModelLoadingEvent,
            events::PerformanceWarningEvent,
            events::EspeakStatusChangedEvent,
        ])
}

fn main() {
    let builder = build_ipc_handler();

    // In debug builds, regenerate ui/src/ipc/bindings.ts before launching
    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../ui/src/ipc/bindings.ts",
        )
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .manage(luminos_handle)
        // tauri-specta's Builder IS the invoke handler — do not also call
        // tauri::generate_handler![] for the same commands.
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Fallback:** If `tauri-specta` introduces a blocker (e.g., unsupported type), that command's types are defined manually in `ui/src/types/ipc-manual.ts` with a comment explaining why. Manual types are validated against Rust types in the integration test suite (Section 13.3).

### 2.3 IPC Command Catalogue

All commands are `async` Rust functions decorated with `#[tauri::command]`. Return types use `Result<T, String>` -- errors are serialized as strings for simplicity; structured error types are a Phase 2 enhancement.

#### Magnification Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `get_current_settings` | — | `AppSettings` | Full settings snapshot for panel hydration on startup |
| `set_zoom_level` | `level: f32` | `()` | Set magnification zoom (1.5–20.0). Clamped server-side. |
| `set_magnification_mode` | `mode: MagnificationMode` | `()` | Switch between `FullScreen`, `Lens`, `Docked` |
| `set_tracking_mode` | `mode: TrackingMode` | `()` | Switch between `Cursor`, `Focus`, `TextCaret` |
| `set_dock_edge` | `edge: DockEdge, size_percent: u32` | `()` | Set docked overlay edge and reserved height/width (10–90%) |
| `set_lens_size` | `width: u32, height: u32` | `()` | Set lens overlay dimensions in pixels |
| `set_lens_shape` | `shape: LensShape` | `()` | Set lens shape: `Rectangle` or `Ellipse` |
| `toggle_magnification` | — | `bool` | Toggle magnification on/off; returns new `is_active` state |

#### Display Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `set_color_filter` | `config: ColorFilterConfig` | `()` | Set active color filter (type, brightness, contrast, matrix) |
| `set_cursor_config` | `config: CursorConfig` | `()` | Configure cursor enhancement (size, crosshairs, halo) |
| `set_interpolation_mode` | `mode: InterpolationMode` | `()` | `Bilinear` (Phase 0) or `Bicubic` (Phase 1+) |
| `set_frame_rate_target` | `fps: u32` | `()` | Set target FPS (15–144). Only meaningful in `Performance` present mode. |
| `set_present_mode` | `mode: PresentMode` | `()` | `Quality` (Fifo/vsync), `LowLatency` (Mailbox), `Performance` (Immediate) |
| `set_gpu_preference` | `preference: GpuPreference` | `()` | `LowPower` (default, integrated GPU) or `HighPerformance` (discrete GPU). Requires restart to take effect. |

#### Speech Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_voices` | — | `Vec<VoiceInfo>` | All available voices (Kokoro speakers, Piper models, platform-native) |
| `set_voice` | `voice_id: String` | `()` | Select active voice. May trigger async model loading. |
| `set_speech_rate` | `rate: f32` | `()` | Speech rate multiplier (0.5–3.0, 1.0 = normal) |
| `set_speech_volume` | `volume: f32` | `()` | Speech volume (0.0–1.0) |
| `set_model_variant` | `variant: ModelVariant` | `()` | Select Kokoro quantization: `Q4`, `Q8` (default), `Fp16`, `Fp32` |
| `speak_text` | `text: String, interrupt: bool` | `()` | Send text to TTS pipeline. `interrupt=true` stops current speech. |
| `stop_speech` | — | `()` | Stop current speech immediately |
| `get_tts_status` | — | `TtsStatus` | Current TTS state: `Idle`, `Loading`, `Speaking`, `Draining`, `Error` |
| `check_espeak_available` | — | `bool` | Whether espeak-ng binary is found at expected path |

#### Settings and Persistence Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `save_settings` | — | `()` | Persist current settings to `~/.config/luminos/config.toml` |
| `reset_settings` | — | `AppSettings` | Reset to compiled-in defaults; returns default settings for UI update |
| `set_start_on_login` | `enabled: bool` | `()` | Register/unregister login item |
| `set_minimize_to_tray` | `enabled: bool` | `()` | Control window-close behavior |

#### Profile Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_profiles` | — | `Vec<ProfileInfo>` | All profiles (built-in + user-created), ordered by name |
| `load_profile` | `id: String` | `AppSettings` | Load profile; returns updated settings for UI sync |
| `save_profile` | `name: String` | `ProfileInfo` | Snapshot current settings as a named profile; returns new profile metadata |
| `delete_profile` | `id: String` | `()` | Delete a user-created profile. Built-in profiles cannot be deleted. |
| `export_profile` | `id: String` | `String` | Serialize profile to JSON string |
| `import_profile` | `json: String` | `ProfileInfo` | Parse and save a JSON profile; returns saved profile metadata |

#### Keybinding Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `get_keybindings` | — | `KeybindingsConfig` | Current keybinding configuration |
| `set_keybinding` | `action: HotkeyAction, binding: Option<KeyBinding>` | `()` | Assign or clear a keybinding for an action |
| `reset_keybindings` | — | `KeybindingsConfig` | Reset all keybindings to defaults |

#### Diagnostics Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `get_frame_timings` | — | `FrameTimingSummary` | Average and P99 frame times from the last 120 frames (Section 10.1) |
| `get_system_info` | — | `SystemInfo` | GPU name, OS, Luminos version, memory usage |

### 2.4 IPC Event Catalogue

Events are emitted by the Rust engine and received by the control panel via `listen()`. The panel must re-subscribe after each window reload.

| Event | Payload Type | Trigger |
|-------|-------------|---------|
| `settings_changed` | `SettingsChangedPayload` | Any setting changes (hotkey, IPC command, or profile load). Panel updates its display to match. |
| `zoom_changed` | `f32` | Zoom level changed via hotkey. Panel updates zoom slider. |
| `mode_changed` | `MagnificationMode` | Mode changed via hotkey. Panel updates mode selector. |
| `tts_status_changed` | `TtsStatusPayload` | TTS state transitions: Idle → Speaking → Draining → Idle, or any → Error. |
| `voice_model_loading` | `ModelLoadingPayload` | Voice model download or load progress (0.0–1.0). |
| `performance_warning` | `PerformanceWarningPayload` | P99 frame time exceeded 20ms for 5 consecutive seconds. Panel displays a suggestion. |
| `espeak_status_changed` | `EspeakStatusPayload` | espeak-ng subprocess crashed, recovered, or became unavailable. |

### 2.5 Thread Safety

All IPC command handlers in `tauri_commands.rs` run on Tauri's async tokio runtime (the `Tauri IPC Thread` in [01 -- System Architecture](./01-system-architecture.md), Section 6.2). They must not:
- Acquire a write lock for longer than microseconds (the render thread holds a read lock every frame)
- Perform blocking I/O on the tokio thread (use `tokio::task::spawn_blocking` for file operations)
- Call wgpu or winit APIs directly (send via `EventLoopProxy` instead)

```rust
// tauri_commands.rs — illustrative command structure
#[tauri::command]
#[specta::specta]
pub(crate) async fn set_zoom_level(
    level: f32,
    handle: tauri::State<'_, LuminosHandle>,
) -> Result<(), String> {
    let clamped = level.clamp(1.5, 20.0);

    // Lock-free write via ArcSwap (see 01-system-architecture.md Section 6.4)
    handle.app_state.rcu(|s| AppState { zoom_level: clamped, ..(**s).clone() });

    // Wake the winit render loop to apply the change on the next frame
    handle.event_proxy
        .send_event(LuminosEvent::ZoomChanged(clamped))
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

---

## 3. TypeScript Type System

All types used at the IPC boundary are defined as **Zod schemas** first; TypeScript types are derived via `z.infer<>`. Zod is used to validate IPC responses at runtime (since `tauri-specta` generation can drift from reality during early development) and for form validation in settings panels.

Schemas live in `ui/src/types/`. Each file corresponds to a domain.

### 3.1 Core Enumerations (`ui/src/types/enums.ts`)

```typescript
import { z } from 'zod';

export const MagnificationModeSchema = z.enum(['FullScreen', 'Lens', 'Docked']);
export type MagnificationMode = z.infer<typeof MagnificationModeSchema>;

export const TrackingModeSchema = z.enum(['Cursor', 'Focus', 'TextCaret']);
export type TrackingMode = z.infer<typeof TrackingModeSchema>;

export const DockEdgeSchema = z.enum(['Top', 'Bottom', 'Left', 'Right']);
export type DockEdge = z.infer<typeof DockEdgeSchema>;

export const LensShapeSchema = z.enum(['Rectangle', 'Ellipse']);
export type LensShape = z.infer<typeof LensShapeSchema>;

export const ColorFilterTypeSchema = z.enum([
  'None',
  'Invert',
  'SmartInvert',
  'Grayscale',
  'HighContrast',
  'Custom',
]);
export type ColorFilterType = z.infer<typeof ColorFilterTypeSchema>;

export const InterpolationModeSchema = z.enum(['Bilinear', 'Bicubic']);
export type InterpolationMode = z.infer<typeof InterpolationModeSchema>;

export const PresentModeSchema = z.enum(['Quality', 'LowLatency', 'Performance']);
export type PresentMode = z.infer<typeof PresentModeSchema>;

export const GpuPreferenceSchema = z.enum(['LowPower', 'HighPerformance']);
export type GpuPreference = z.infer<typeof GpuPreferenceSchema>;

export const ModelVariantSchema = z.enum(['Q4', 'Q8', 'Fp16', 'Fp32']);
export type ModelVariant = z.infer<typeof ModelVariantSchema>;

export const TtsBackendSchema = z.enum(['Kokoro', 'Piper', 'Native']);
export type TtsBackend = z.infer<typeof TtsBackendSchema>;

export const TtsStatusSchema = z.enum(['Idle', 'Loading', 'Speaking', 'Draining', 'Error']);
export type TtsStatus = z.infer<typeof TtsStatusSchema>;

export const HotkeyActionSchema = z.enum([
  'ZoomIn',
  'ZoomOut',
  'ZoomReset',
  'ToggleMagnification',
  'CycleMode',
  'ReadWhatISee',
  'ReadSelection',
  'StopSpeech',
  'FindCursor',
]);
export type HotkeyAction = z.infer<typeof HotkeyActionSchema>;
```

**Naming convention:** Enum variants use PascalCase in both Rust (`MagnificationMode::FullScreen`) and TypeScript (`'FullScreen'`). Serde's `rename_all = "PascalCase"` ensures wire format consistency.

### 3.2 Settings Types (`ui/src/types/settings.ts`)

```typescript
import { z } from 'zod';
import {
  MagnificationModeSchema, TrackingModeSchema, DockEdgeSchema, LensShapeSchema,
  ColorFilterTypeSchema, InterpolationModeSchema, PresentModeSchema,
  GpuPreferenceSchema, ModelVariantSchema, HotkeyActionSchema,
} from './enums';

// --- Magnification ---

export const MagnificationSettingsSchema = z.object({
  zoomLevel:        z.number().min(1.5).max(20.0),
  mode:             MagnificationModeSchema,
  trackingMode:     TrackingModeSchema,
  dockedEdge:       DockEdgeSchema.optional(),
  dockedSizePercent: z.number().int().min(10).max(90).optional(),
  lensWidth:        z.number().int().min(200).max(3840).optional(),
  lensHeight:       z.number().int().min(150).max(2160).optional(),
  lensShape:        LensShapeSchema.optional(),
  targetFps:        z.number().int().min(15).max(144),
  presentMode:      PresentModeSchema,
  gpuPreference:    GpuPreferenceSchema,
  interpolation:    InterpolationModeSchema,
  smoothScrolling:  z.boolean(),
});
export type MagnificationSettings = z.infer<typeof MagnificationSettingsSchema>;

// --- Color Filter ---

export const ColorFilterConfigSchema = z.object({
  filterType:   ColorFilterTypeSchema,
  brightness:   z.number().min(-1.0).max(1.0),
  contrast:     z.number().min(0.0).max(3.0),
  // 16-element row-major 4x4 matrix. Only present when filterType === 'Custom'.
  colorMatrix:  z.array(z.number()).length(16).optional(),
});
export type ColorFilterConfig = z.infer<typeof ColorFilterConfigSchema>;

// --- Cursor Enhancement ---

// CSS hex color string: '#rrggbb' or '#rrggbbaa'
const HexColorSchema = z.string().regex(/^#[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$/);

export const CursorConfigSchema = z.object({
  enlargedCursor:    z.boolean(),
  cursorScale:       z.number().min(1.0).max(4.0),
  crosshairsEnabled: z.boolean(),
  crosshairWidth:    z.number().int().min(1).max(10),
  crosshairColor:    HexColorSchema,
  haloEnabled:       z.boolean(),
  haloRadius:        z.number().int().min(10).max(200),
  haloColor:         HexColorSchema,
});
export type CursorConfig = z.infer<typeof CursorConfigSchema>;

// --- Speech ---

export const SpeechSettingsSchema = z.object({
  enabled:      z.boolean(),
  voiceId:      z.string(),
  speechRate:   z.number().min(0.5).max(3.0),
  speechVolume: z.number().min(0.0).max(1.0),
  modelVariant: ModelVariantSchema,
});
export type SpeechSettings = z.infer<typeof SpeechSettingsSchema>;

// --- Keybindings ---

const ModifierKeySchema = z.enum(['Ctrl', 'Shift', 'Alt', 'Super', 'Meta']);

export const KeyBindingSchema = z.object({
  key:       z.string().min(1),      // e.g. "Equal", "Minus", "F1"
  modifiers: z.array(ModifierKeySchema),
});
export type KeyBinding = z.infer<typeof KeyBindingSchema>;

export const KeybindingsConfigSchema = z.record(
  HotkeyActionSchema,
  KeyBindingSchema.nullable(),        // null = unbound
);
export type KeybindingsConfig = z.infer<typeof KeybindingsConfigSchema>;

// --- Application Settings (root) ---

export const AppSettingsSchema = z.object({
  magnification:     MagnificationSettingsSchema,
  colorFilter:       ColorFilterConfigSchema,
  cursor:            CursorConfigSchema,
  speech:            SpeechSettingsSchema,
  keybindings:       KeybindingsConfigSchema,
  startOnLogin:      z.boolean(),
  minimizeToTray:    z.boolean(),
  showPanelOnStart:  z.boolean(),
});
export type AppSettings = z.infer<typeof AppSettingsSchema>;
```

### 3.3 TTS Types (`ui/src/types/tts.ts`)

```typescript
import { z } from 'zod';
import { TtsBackendSchema, TtsStatusSchema, ModelVariantSchema } from './enums';

export const VoiceInfoSchema = z.object({
  id:          z.string(),          // Unique voice ID (e.g. "kokoro-heart-en-us")
  name:        z.string(),          // Display name (e.g. "Heart (Kokoro, en-US)")
  engine:      TtsBackendSchema,
  language:    z.string(),          // BCP 47 language code (e.g. "en-US")
  modelId:     z.string().optional(), // Model file ID (Kokoro/Piper only)
  isInstalled: z.boolean(),         // False if model file is not yet downloaded
});
export type VoiceInfo = z.infer<typeof VoiceInfoSchema>;

export const TtsStatusPayloadSchema = z.object({
  status:  TtsStatusSchema,
  voiceId: z.string().optional(),   // Active voice at time of status change
  error:   z.string().optional(),   // Set when status === 'Error'
});
export type TtsStatusPayload = z.infer<typeof TtsStatusPayloadSchema>;

export const ModelLoadingPayloadSchema = z.object({
  voiceId:  z.string(),
  progress: z.number().min(0).max(1), // 0.0 = started, 1.0 = complete
  stage:    z.enum(['Downloading', 'Loading', 'Complete', 'Failed']),
  errorMessage: z.string().optional(),
});
export type ModelLoadingPayload = z.infer<typeof ModelLoadingPayloadSchema>;
```

### 3.4 Diagnostics Types (`ui/src/types/diagnostics.ts`)

```typescript
import { z } from 'zod';

export const FrameTimingSummarySchema = z.object({
  averageMs: z.number(),   // Average frame time over the last 120 frames
  p99Ms:     z.number(),   // 99th-percentile frame time over the last 120 frames
  minMs:     z.number(),
  maxMs:     z.number(),
  targetFps: z.number().int(),
});
export type FrameTimingSummary = z.infer<typeof FrameTimingSummarySchema>;

export const PerformanceWarningPayloadSchema = z.object({
  p99Ms:          z.number(),
  recommendation: z.enum(['SwitchToPerformanceMode', 'ReduceZoomLevel', 'None']),
});
export type PerformanceWarningPayload = z.infer<typeof PerformanceWarningPayloadSchema>;

export const SystemInfoSchema = z.object({
  luminosVersion: z.string(),
  os:             z.string(),   // e.g. "Linux 6.8 (X11)"
  gpuName:        z.string(),   // e.g. "Intel UHD Graphics 770"
  gpuBackend:     z.string(),   // e.g. "Vulkan"
  totalRamMb:     z.number().int(),
  processRamMb:   z.number().int(),
});
export type SystemInfo = z.infer<typeof SystemInfoSchema>;
```

### 3.5 Profile Types (`ui/src/types/profiles.ts`)

```typescript
import { z } from 'zod';
import { AppSettingsSchema } from './settings';

export const ProfileKindSchema = z.enum([
  'BuiltIn',     // Shipped with Luminos, read-only
  'UserCreated', // Saved by the user
]);
export type ProfileKind = z.infer<typeof ProfileKindSchema>;

export const ProfileInfoSchema = z.object({
  id:          z.string(),          // Stable UUID
  name:        z.string().min(1).max(64),
  kind:        ProfileKindSchema,
  description: z.string().optional(),
  createdAt:   z.string().datetime().optional(), // ISO 8601, absent for built-ins
});
export type ProfileInfo = z.infer<typeof ProfileInfoSchema>;

// Full profile payload used for export/import
export const ProfileDocumentSchema = z.object({
  version:  z.literal(1),
  profile:  ProfileInfoSchema,
  settings: AppSettingsSchema,
});
export type ProfileDocument = z.infer<typeof ProfileDocumentSchema>;
```

---

## 4. IPC Layer Implementation

### 4.1 Rust Side: `luminos-app/src/tauri_commands.rs`

All `#[tauri::command]` functions live in a single file. The file has one responsibility: translate Tauri IPC calls into operations on the shared `LuminosHandle` (which owns `Arc<ArcSwap<AppState>>`, `Arc<parking_lot::Mutex<ConfigManager>>`, the `EventLoopProxy`, and the `TtsSender`).

```
luminos-app/src/
  tauri_commands.rs   -- All #[tauri::command] #[specta::specta] functions
  events.rs           -- Rust event structs (SettingsChangedEvent, TtsStatusChangedEvent,
  |                      ZoomChangedEvent, etc.) deriving tauri_specta::Event
  |
  tauri_commands.rs --> LuminosHandle (Arc<AppState>, ConfigManager, EventLoopProxy, TtsSender)
                            |
                            +--- ArcSwap<AppState>         (render thread reads this lock-free)
                            +--- Arc<Mutex<ConfigManager>>  (settings persistence, brief lock)
                            +--- EventLoopProxy<LuminosEvent>  (wake winit loop)
                            +--- TtsSender                 (send SpeechRequest to TTS Coordinator)
                            +--- tauri::AppHandle           (emit events back to webview)
```

The Rust event structs (e.g., `SettingsChangedEvent`) live in `luminos-app/src/events.rs`. Each derives `tauri_specta::Event` so the Builder can include their payload types in the generated TypeScript bindings.

**State access pattern:** Commands receive a `tauri::State<'_, LuminosHandle>` parameter containing all shared resources. This is registered at application startup via `tauri::Builder::manage()`.

```rust
/// Shared handle passed to every Tauri command.
/// Registered via `tauri::Builder::manage(luminos_handle)` at startup.
pub(crate) struct LuminosHandle {
    /// Live application state. Render thread reads this lock-free every frame.
    pub app_state: Arc<ArcSwap<AppState>>,
    /// Configuration manager for persistence (settings, profiles).
    pub config: Arc<parking_lot::Mutex<ConfigManager>>,
    /// Channel for sending speech requests to the TTS Coordinator.
    pub tts_tx: TtsSender,
    /// Sends custom events to the winit event loop on the main thread.
    pub event_proxy: winit::event_loop::EventLoopProxy<LuminosEvent>,
    /// Tauri app handle for emitting events back to the webview.
    pub app: tauri::AppHandle,
}
```

**Command registration** is handled by the `tauri-specta` `Builder` constructed in `main()` (see Section 2.2). Every function in `tauri_commands.rs` is annotated with both `#[tauri::command]` and `#[specta::specta]` so the Builder can introspect its types for TypeScript generation:

```rust
// tauri_commands.rs — annotation pattern for every command
#[tauri::command]
#[specta::specta]
pub(crate) async fn set_zoom_level(
    level: f32,
    handle: tauri::State<'_, LuminosHandle>,
) -> Result<(), String> {
    // ...
}
```

The complete list of commands passed to `collect_commands![...]` in `main.rs` mirrors the IPC catalogue in Section 2.3. The `tauri::generate_handler![]` macro is **not** used — `builder.invoke_handler()` replaces it entirely.

### 4.2 TypeScript Command Wrappers (`ui/src/ipc/commands.ts`)

When using `tauri-specta`, typed wrappers are generated into `ui/src/ipc/bindings.ts` automatically. For commands that need additional client-side validation, thin wrappers in `commands.ts` apply Zod parsing to the response:

```typescript
import { commands } from './bindings';    // tauri-specta generated
import { AppSettingsSchema } from '../types/settings';
import { VoiceInfoSchema } from '../types/tts';
import type { AppSettings } from '../types/settings';
import type { VoiceInfo } from '../types/tts';

/**
 * Fetches the current application settings from the engine.
 * Validates the response against the Zod schema to catch IPC drift early.
 *
 * @returns Validated AppSettings
 * @throws ZodError if the response does not match the expected schema
 */
export async function getCurrentSettings(): Promise<AppSettings> {
  const raw = await commands.getCurrentSettings();
  return AppSettingsSchema.parse(raw);
}

/**
 * Sets the zoom level. Client-side clamping prevents sending invalid values.
 *
 * @param level - Zoom multiplier (clamped to [1.5, 20.0])
 */
export async function setZoomLevel(level: number): Promise<void> {
  const clamped = Math.min(20.0, Math.max(1.5, level));
  await commands.setZoomLevel({ level: clamped });
}

/**
 * Retrieves all available TTS voices.
 *
 * @returns Validated array of VoiceInfo
 */
export async function listVoices(): Promise<VoiceInfo[]> {
  const raw = await commands.listVoices();
  return raw.map(v => VoiceInfoSchema.parse(v));
}

// All other commands follow the same pattern:
// re-export directly from bindings when no additional validation is needed,
// or wrap with Zod validation when the response type is complex.
//
// tauri-specta v2 generates top-level named functions in bindings.ts,
// so they can be re-exported directly with a standard named export clause.
export {
  setMagnificationMode,
  setTrackingMode,
  setDockEdge,
  setLensSize,
  setLensShape,
  toggleMagnification,
  setColorFilter,
  setCursorConfig,
  setInterpolationMode,
  setFrameRateTarget,
  setPresentMode,
  setGpuPreference,
  setVoice,
  setSpeechRate,
  setSpeechVolume,
  setModelVariant,
  speakText,
  stopSpeech,
  getTtsStatus,
  saveSettings,
  resetSettings,
  setStartOnLogin,
  setMinimizeToTray,
  listProfiles,
  loadProfile,
  saveProfile,
  deleteProfile,
  exportProfile,
  importProfile,
  getKeybindings,
  setKeybinding,
  resetKeybindings,
  getFrameTimings,
  getSystemInfo,
  checkEspeakAvailable,
} from './bindings';
```

### 4.3 TypeScript Event Subscriptions (`ui/src/ipc/events.ts`)

Events use Tauri's `listen()` API. Each subscription returns an `UnlistenFn` that must be called on component unmount to prevent memory leaks.

```typescript
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  TtsStatusPayloadSchema,
  ModelLoadingPayloadSchema,
} from '../types/tts';
import {
  PerformanceWarningPayloadSchema,
} from '../types/diagnostics';
import type { TtsStatusPayload, ModelLoadingPayload } from '../types/tts';
import type { PerformanceWarningPayload } from '../types/diagnostics';
import type { AppSettings } from '../types/settings';
import type { MagnificationMode } from '../types/enums';

/** Subscribe to full settings snapshot updates (hotkey changes, profile loads). */
export async function onSettingsChanged(
  handler: (settings: AppSettings) => void,
): Promise<UnlistenFn> {
  return listen<AppSettings>('settings_changed', e => handler(e.payload));
}

/** Subscribe to zoom level changes originating from hotkeys. */
export async function onZoomChanged(
  handler: (level: number) => void,
): Promise<UnlistenFn> {
  return listen<number>('zoom_changed', e => handler(e.payload));
}

/** Subscribe to magnification mode changes from hotkeys. */
export async function onModeChanged(
  handler: (mode: MagnificationMode) => void,
): Promise<UnlistenFn> {
  return listen<MagnificationMode>('mode_changed', e => handler(e.payload));
}

/** Subscribe to TTS state transitions. */
export async function onTtsStatusChanged(
  handler: (payload: TtsStatusPayload) => void,
): Promise<UnlistenFn> {
  return listen<unknown>('tts_status_changed', e => {
    const payload = TtsStatusPayloadSchema.parse(e.payload);
    handler(payload);
  });
}

/** Subscribe to voice model download/load progress. */
export async function onVoiceModelLoading(
  handler: (payload: ModelLoadingPayload) => void,
): Promise<UnlistenFn> {
  return listen<unknown>('voice_model_loading', e => {
    const payload = ModelLoadingPayloadSchema.parse(e.payload);
    handler(payload);
  });
}

/** Subscribe to performance degradation warnings from the render pipeline. */
export async function onPerformanceWarning(
  handler: (payload: PerformanceWarningPayload) => void,
): Promise<UnlistenFn> {
  return listen<unknown>('performance_warning', e => {
    const payload = PerformanceWarningPayloadSchema.parse(e.payload);
    handler(payload);
  });
}
```

---

## 5. State Management

### 5.1 Strategy: Zustand

The control panel uses **Zustand** (v5, MIT) for client-side state management. Zustand is chosen because:

- **Minimal boilerplate.** A store is a single `create()` call. No reducers, action creators, or context providers.
- **TypeScript-first.** Fully typed without ceremony.
- **Small bundle footprint.** ~1KB minified+gzipped. Appropriate for a settings panel that runs inside WebkitGTK.
- **No stale closure issues.** Zustand's `getState()` always returns current state, which is important when Tauri event handlers are registered once at mount.

Three stores cover all panel state:

| Store | File | Responsibility |
|-------|------|----------------|
| `useSettingsStore` | `hooks/useSettingsStore.ts` | Full `AppSettings` snapshot, in-flight update flags |
| `useTtsStore` | `hooks/useTtsStore.ts` | `TtsStatus`, active voice, model loading progress |
| `useProfilesStore` | `hooks/useProfilesStore.ts` | Profile list, currently loaded profile ID |

### 5.2 Settings Store (`ui/src/hooks/useSettingsStore.ts`)

```typescript
import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import type { AppSettings, MagnificationSettings, ColorFilterConfig } from '../types/settings';
import { DEFAULT_SETTINGS } from '../constants/defaults';

interface SettingsState {
  /** Current settings as synced from the engine. */
  readonly settings: AppSettings;
  /** True while the initial settings load (getCurrentSettings) is in flight. */
  readonly isHydrating: boolean;

  // Actions
  hydrate: (settings: AppSettings) => void;
  applyEngineUpdate: (settings: AppSettings) => void;
  setZoomLevel: (level: number) => void;
  setColorFilter: (config: ColorFilterConfig) => void;
  // ... one action per IPC command that mutates settings
}

export const useSettingsStore = create<SettingsState>()(
  immer((set) => ({
    settings: DEFAULT_SETTINGS,
    isHydrating: true,

    hydrate: (settings) => set(state => {
      state.settings = settings;
      state.isHydrating = false;
    }),

    // Called when the engine emits `settings_changed` (e.g., via hotkey).
    // Merges the engine's snapshot into local state so the UI always reflects truth.
    applyEngineUpdate: (settings) => set(state => {
      state.settings = settings;
    }),

    setZoomLevel: (level) => set(state => {
      state.settings.magnification.zoomLevel = Math.min(20, Math.max(1.5, level));
    }),

    setColorFilter: (config) => set(state => {
      state.settings.colorFilter = config;
    }),
  })),
);
```

**Optimistic updates:** When a user moves a slider, the store is updated immediately for a responsive feel. The `invoke()` call runs concurrently. If the command fails (returns an error), the store reverts to the previous value and displays an error toast. This prevents the UI from feeling laggy on the ~1ms round-trip through Tauri IPC.

### 5.3 TTS Store (`ui/src/hooks/useTtsStore.ts`)

```typescript
import { create } from 'zustand';
import type { TtsStatus, VoiceInfo, ModelLoadingPayload } from '../types/tts';

interface TtsState {
  readonly status: TtsStatus;
  readonly activeVoiceId: string | null;
  readonly availableVoices: readonly VoiceInfo[];
  readonly modelLoading: ModelLoadingPayload | null;
  readonly espeakAvailable: boolean;
  readonly lastError: string | null;

  setStatus: (payload: { status: TtsStatus; voiceId?: string; error?: string }) => void;
  setVoices: (voices: VoiceInfo[]) => void;
  setModelLoading: (payload: ModelLoadingPayload | null) => void;
  setEspeakAvailable: (available: boolean) => void;
}

export const useTtsStore = create<TtsState>()((set) => ({
  status: 'Idle',
  activeVoiceId: null,
  availableVoices: [],
  modelLoading: null,
  espeakAvailable: true,
  lastError: null,

  setStatus: ({ status, voiceId, error }) =>
    set({ status, activeVoiceId: voiceId ?? null, lastError: error ?? null }),

  setVoices: (voices) => set({ availableVoices: voices }),
  setModelLoading: (payload) => set({ modelLoading: payload }),
  setEspeakAvailable: (available) => set({ espeakAvailable: available }),
}));
```

### 5.4 Hydration on Startup

On application start, the control panel must load the current engine state before rendering interactive controls. The hydration sequence runs in the root `App` component:

```typescript
// ui/src/App.tsx
import { useEffect } from 'react';
import { getCurrentSettings } from './ipc/commands';
import { onSettingsChanged, onZoomChanged, onTtsStatusChanged,
         onVoiceModelLoading, onPerformanceWarning } from './ipc/events';
import { useSettingsStore } from './hooks/useSettingsStore';
import { useTtsStore } from './hooks/useTtsStore';

export function App() {
  const hydrate = useSettingsStore(s => s.hydrate);
  const applyEngineUpdate = useSettingsStore(s => s.applyEngineUpdate);
  const { setStatus, setModelLoading } = useTtsStore();
  const { showError } = useToast();

  useEffect(() => {
    // 1. Fetch initial settings snapshot. On failure, exit the loading state
    // so the user sees an actionable error instead of a permanent spinner.
    getCurrentSettings()
      .then(hydrate)
      .catch(err => {
        hydrate(DEFAULT_SETTINGS);      // unblock the UI with safe defaults
        showError(`Failed to load settings: ${String(err)}`);
      });

    // 2. Subscribe to engine-side state changes
    const unlisteners = Promise.all([
      onSettingsChanged(applyEngineUpdate),
      onZoomChanged(level =>
        useSettingsStore.getState().setZoomLevel(level)),
      onTtsStatusChanged(setStatus),
      onVoiceModelLoading(payload =>
        setModelLoading(payload.stage === 'Complete' ? null : payload)),
      onPerformanceWarning(payload =>
        console.warn('Performance warning:', payload)),
    ]);

    // 3. Clean up event listeners when App unmounts (window reload)
    return () => {
      unlisteners.then(fns => fns.forEach(fn => fn()));
    };
  }, []);  // Empty deps: register once at mount

  // ... render
}
```

**Hydration timing:** The control panel webview loads at T≈1000ms in the startup sequence (see [01 -- System Architecture](./01-system-architecture.md), Section 9.4). By that time, the Rust engine has already initialized and `get_current_settings` will return a valid response. There is no race condition to handle: the webview cannot fire IPC calls until it is fully loaded, and the Rust IPC handlers are registered before Tauri starts the webview.

---

## 6. React Component Architecture

### 6.1 Component Hierarchy

The control panel is a single-page application using React Router (v7, MIT) for in-panel navigation between settings pages. Navigation is sidebar-based (always visible). The overall tree:

```
App
  ├── HydrationGate          -- Shows loading spinner until isHydrating = false
  │     └── Shell
  │           ├── Sidebar    -- Navigation links (keyboard-accessible, ARIA nav landmark)
  │           └── Outlet     -- Active page content
  │                 ├── MagnificationPage
  │                 │     ├── ZoomLevelSlider
  │                 │     ├── MagnificationModeSelector
  │                 │     ├── TrackingModeSelector
  │                 │     ├── DockedModeControls     (shown only when mode = Docked)
  │                 │     └── LensModeControls       (shown only when mode = Lens)
  │                 ├── DisplayPage
  │                 │     ├── ColorFilterPanel
  │                 │     │     ├── FilterTypeSelector
  │                 │     │     ├── BrightnessSlider
  │                 │     │     └── ContrastSlider
  │                 │     ├── CursorEnhancementPanel
  │                 │     └── RenderingControls      (FPS, present mode, interpolation)
  │                 ├── SpeechPage
  │                 │     ├── EspeakWarningBanner    (shown only when espeakAvailable = false)
  │                 │     ├── VoiceSelector
  │                 │     │     ├── VoiceList
  │                 │     │     │     └── VoiceListItem  (x N voices)
  │                 │     │     └── ModelDownloadProgress
  │                 │     ├── SpeechRateSlider
  │                 │     ├── SpeechVolumeSlider
  │                 │     ├── ModelVariantSelector
  │                 │     └── TtsStatusIndicator
  │                 ├── KeybindingsPage
  │                 │     └── KeybindingTable
  │                 │           └── KeybindingRow    (x N actions)
  │                 │                 └── KeyCaptureInput
  │                 ├── ProfilesPage
  │                 │     ├── BuiltInProfileList
  │                 │     ├── UserProfileList
  │                 │     │     └── ProfileCard      (x N profiles)
  │                 │     ├── SaveProfileDialog
  │                 │     └── ImportExportControls
  │                 └── DiagnosticsPage              (shown in debug builds only)
  │                       ├── FrameTimingDisplay
  │                       ├── SystemInfoPanel
  │                       └── EspeakStatusPanel
  └── ToastProvider          -- Error/warning notifications (accessible live region)
```

### 6.2 Routing

Routes are defined in `ui/src/App.tsx` using React Router's `createHashRouter` (hash routing avoids server-side routing issues inside the Tauri webview):

```typescript
import { createHashRouter, RouterProvider, Navigate } from 'react-router';
import { Shell } from './components/Shell';
import { MagnificationPage } from './pages/MagnificationPage';
import { DisplayPage } from './pages/DisplayPage';
import { SpeechPage } from './pages/SpeechPage';
import { KeybindingsPage } from './pages/KeybindingsPage';
import { ProfilesPage } from './pages/ProfilesPage';
import { DiagnosticsPage } from './pages/DiagnosticsPage';

const router = createHashRouter([
  {
    path: '/',
    element: <Shell />,
    children: [
      { index: true, element: <Navigate to="/magnification" replace /> },
      { path: 'magnification', element: <MagnificationPage /> },
      { path: 'display',       element: <DisplayPage /> },
      { path: 'speech',        element: <SpeechPage /> },
      { path: 'keybindings',   element: <KeybindingsPage /> },
      { path: 'profiles',      element: <ProfilesPage /> },
      { path: 'diagnostics',   element: <DiagnosticsPage /> },
    ],
  },
]);
```

### 6.3 Component Patterns

**Settings control pattern:** Every control that maps to an IPC command follows the same three-step pattern:

1. Read current value from the Zustand store (optimistic local state)
2. On change, update the store immediately (optimistic) and `invoke` the IPC command
3. On IPC error, revert store to previous value and show an error toast

```typescript
// Example: ZoomLevelSlider
import { useCallback } from 'react';
import { useSettingsStore } from '../hooks/useSettingsStore';
import { setZoomLevel } from '../ipc/commands';
import { useToast } from '../hooks/useToast';

export function ZoomLevelSlider() {
  const zoomLevel = useSettingsStore(s => s.settings.magnification.zoomLevel);
  const setZoomLevelStore = useSettingsStore(s => s.setZoomLevel);
  const { showError } = useToast();

  const handleChange = useCallback(async (value: number) => {
    const previous = zoomLevel;
    setZoomLevelStore(value);             // Optimistic update

    try {
      await setZoomLevel(value);          // IPC command
    } catch (err) {
      setZoomLevelStore(previous);        // Revert on failure
      showError(`Failed to set zoom level: ${String(err)}`);
    }
  }, [zoomLevel, setZoomLevelStore, showError]);

  return (
    <label className="settings-control">
      <span id="zoom-label">Zoom level</span>
      <input
        type="range"
        aria-labelledby="zoom-label"
        aria-valuetext={`${zoomLevel.toFixed(1)}x`}
        min={1.5}
        max={20}
        step={0.5}
        value={zoomLevel}
        onChange={e => handleChange(parseFloat(e.target.value))}
      />
      <output aria-live="polite">{zoomLevel.toFixed(1)}x</output>
    </label>
  );
}
```

**Debounced sliders:** Sliders (zoom, brightness, contrast, speech rate) debounce IPC calls to avoid flooding the engine with updates during drag. The store is updated on every slider `onChange` for visual responsiveness; IPC is called only after a 150ms debounce. This prevents configuration write-lock contention during rapid slider movement.

```typescript
import { useDebouncedCallback } from 'use-debounce';

const invokeSetZoomLevel = useDebouncedCallback(setZoomLevel, 150);
```

**Shared primitive components** (`ui/src/components/shared/`):

| Component | Props | Description |
|-----------|-------|-------------|
| `Slider` | `value`, `min`, `max`, `step`, `label`, `onChange`, `valueFormatter` | Accessible range slider with visible label and output |
| `ToggleSwitch` | `checked`, `label`, `onChange` | Accessible checkbox styled as a toggle |
| `Select` | `value`, `options`, `label`, `onChange` | Accessible `<select>` with label |
| `SectionHeader` | `title`, `description?` | Consistent section heading within a page |
| `StatusBadge` | `status: 'ok' \| 'warning' \| 'error'`, `label` | Color-coded status pill with text label |
| `SettingsRow` | `label`, `description?`, `children` | Consistent row layout: label left, control right |

---

## 7. Settings Pages

### 7.1 Magnification Page (`MagnificationPage.tsx`)

The primary page. Controls the core magnification parameters.

| Control | Type | Setting | IPC Command |
|---------|------|---------|-------------|
| Zoom level | Slider (1.5–20.0, step 0.5) | `magnification.zoomLevel` | `set_zoom_level` |
| Magnification mode | Radio group | `magnification.mode` | `set_magnification_mode` |
| Tracking mode | Select | `magnification.trackingMode` | `set_tracking_mode` |
| Docked edge | Radio group (Top/Bottom/Left/Right) | `magnification.dockedEdge` | `set_dock_edge` |
| Docked size | Slider (10–90%, step 5) | `magnification.dockedSizePercent` | `set_dock_edge` |
| Lens width | Number input (200–3840px) | `magnification.lensWidth` | `set_lens_size` |
| Lens height | Number input (150–2160px) | `magnification.lensHeight` | `set_lens_size` |
| Lens shape | Radio group | `magnification.lensShape` | `set_lens_shape` |

Docked and Lens controls are shown conditionally based on the selected mode to avoid cluttering the page. A compact current-zoom readout (e.g. "**5.0×**") appears prominently at the top of the page for users who need magnification just to use the panel.

### 7.2 Display Page (`DisplayPage.tsx`)

Color filter, cursor enhancement, and rendering quality controls.

**Color Filter Panel:**

| Control | Type | Setting | IPC Command |
|---------|------|---------|-------------|
| Filter type | Select (None / Invert / Smart Invert / Grayscale / High Contrast / Custom) | `colorFilter.filterType` | `set_color_filter` |
| Brightness | Slider (-1.0 to +1.0) | `colorFilter.brightness` | `set_color_filter` |
| Contrast | Slider (0.0 to 3.0) | `colorFilter.contrast` | `set_color_filter` |
| Preset scheme picker | Button group | Sets `filterType = Custom` + preset `colorMatrix` | `set_color_filter` |

**Preset schemes** (from [03 -- Rendering Pipeline](./03-rendering-pipeline.md), Section 6.3) are presented as labeled swatches so users can preview the color shift before applying:

| Label | Visual | Use Case |
|-------|--------|----------|
| White on Black | Dark background + white text | General low vision |
| Yellow on Blue | Warm foreground, cool background | Cataracts, glare sensitivity |
| Green on Black | High-luminance text | Night use, photophobia |
| Yellow on Black | High-contrast warm | AMD, diabetic retinopathy |

**Cursor Enhancement Panel:**

| Control | Type | Setting | IPC Command |
|---------|------|---------|-------------|
| Enlarge cursor | Toggle | `cursor.enlargedCursor` | `set_cursor_config` |
| Cursor size | Slider (1×–4×) | `cursor.cursorScale` | `set_cursor_config` |
| Crosshairs | Toggle | `cursor.crosshairsEnabled` | `set_cursor_config` |
| Crosshair width | Slider (1–10px) | `cursor.crosshairWidth` | `set_cursor_config` |
| Crosshair color | Color picker | `cursor.crosshairColor` | `set_cursor_config` |
| Halo | Toggle | `cursor.haloEnabled` | `set_cursor_config` |
| Halo radius | Slider (10–200px) | `cursor.haloRadius` | `set_cursor_config` |
| Halo color | Color picker | `cursor.haloColor` | `set_cursor_config` |

**Rendering Controls:**

| Control | Type | Setting | IPC Command | Phase |
|---------|------|---------|-------------|-------|
| Interpolation | Select (Bilinear / Bicubic) | `magnification.interpolation` | `set_interpolation_mode` | Phase 1+ |
| Present mode | Select (Quality / Low-latency / Performance) | `magnification.presentMode` | `set_present_mode` | Phase 0 |
| Target FPS | Slider (15–144) | `magnification.targetFps` | `set_frame_rate_target` | Phase 0 |
| GPU preference | Select (Integrated / Discrete) | `magnification.gpuPreference` | `set_gpu_preference` | Phase 3 |

The GPU preference control (Phase 3) displays a restart-required badge when changed, since the GPU device is selected at startup and cannot be changed at runtime.

### 7.3 Speech Page (`SpeechPage.tsx`)

Described in detail in Section 9.

### 7.4 Keybindings Page (`KeybindingsPage.tsx`)

Displays a table of all configurable hotkey actions. Users can click any row to re-bind the key, or click a "clear" button to unbind it.

| Column | Content |
|--------|---------|
| Action | Human-readable action name (e.g. "Zoom In", "Read What I See") |
| Current binding | Formatted key combination (e.g. "Ctrl + =") or "Not bound" |
| Edit button | Opens an inline `KeyCaptureInput` that listens for the next keypress |
| Clear button | Sets binding to `null` (unbound) |

**`KeyCaptureInput` component:** When active, it intercepts the next keypress (preventing default browser behavior) and displays the captured combination. Escape cancels the capture. On confirm, it calls `set_keybinding` with the captured combination.

```
+----------------------------------+
| Zoom In         [Ctrl + =]  [×]  |
| Zoom Out        [Ctrl + -]  [×]  |
| Read What I See [Ctrl + R]  [×]  |  <- Clicking "Ctrl + R" opens capture mode
| Read Selection  [Not bound] [+]  |  <- Clicking "+" opens capture mode
+----------------------------------+
   [ Reset all to defaults ]
```

**Default keybindings:**

| Action | Default | Notes |
|--------|---------|-------|
| Zoom In | `Ctrl + =` | `=` key (no shift required for `+`) |
| Zoom Out | `Ctrl + -` | |
| Zoom Reset | `Ctrl + 0` | |
| Toggle Magnification | `Ctrl + Alt + M` | |
| Cycle Mode | `Ctrl + Alt + Z` | Cycles Full → Lens → Docked → Full |
| Read What I See | `Ctrl + Alt + R` | |
| Read Selection | `Ctrl + Alt + S` | |
| Stop Speech | `Ctrl + Alt + X` | |
| Find Cursor | `Ctrl + Alt + F` | Triggers locator animation |

### 7.5 Profiles Page (`ProfilesPage.tsx`)

Described in detail in Section 8.

---

## 8. Profile Management

### 8.1 Profile Data Model

A profile is a named snapshot of the full `AppSettings`. Profiles allow users to switch between configurations for different tasks (e.g., reading vs. code editing) or visual conditions.

**Rust-side storage:** Profiles are stored as individual TOML files in the user's data directory:

```
~/.config/luminos/profiles/
  built-in/                    # Shipped with Luminos (read-only, embedded in binary)
    default.toml
    high-contrast.toml
    low-light.toml
    reading-mode.toml
  user/                        # User-created profiles
    {uuid}.toml
  manifest.json                # Profile registry (id, name, kind, createdAt)
```

**Profile uniqueness:** Each profile has a stable UUID (`id`) assigned at creation. Display names are not guaranteed to be unique. The `ProfileInfo` type carries the UUID; the UI displays the human name.

### 8.2 Built-in Profiles

Four profiles ship with Luminos (Phase 1). These cannot be edited or deleted by the user but can be loaded and then "Save as new profile" to create a user copy.

| Profile | Description | Key Settings |
|---------|-------------|--------------|
| **Default** | Standard settings for new installs | 2× zoom, full-screen, cursor tracking, no filters |
| **High Contrast** | Maximum legibility for severe low vision | 5× zoom, Yellow-on-Black color matrix, bicubic interpolation, enlarged cursor + halo |
| **Low Light** | Comfortable for night use / photophobia | 3× zoom, Green-on-Black filter, reduced brightness (-0.2), halo disabled |
| **Reading Mode** | Optimized for long reading sessions | 4× zoom, docked (top, 40%), focus tracking, White-on-Black filter, 0.85× speech rate |

**Phase 3 condition-based profiles:**

| Profile | Target Condition | Key Settings |
|---------|-----------------|--------------|
| **Age-Related Macular Degeneration** | AMD | 8× zoom, eccentric viewing mode (offset viewport), Yellow-on-Black |
| **Glaucoma** | Glaucoma (tunnel vision) | 3× zoom, lens mode (wider lens), high brightness |
| **Cataracts** | Cataracts (glare, haze) | 5× zoom, Yellow-on-Blue filter, increased contrast (1.8×) |
| **Diabetic Retinopathy** | DR (patchy vision) | 6× zoom, full-screen, focus tracking, Yellow-on-Black |

These are presented in a "Condition-based quick start" section of the Profiles page to help new users find relevant settings quickly. A setup wizard (Phase 3) also uses these profiles.

### 8.3 Import/Export (JSON, Git-friendly)

Profiles can be exported as JSON for sharing, institutional deployment, or backup. The export format is `ProfileDocument` (Section 3.5):

```json
{
  "version": 1,
  "profile": {
    "id": "a1b2c3d4-...",
    "name": "Enterprise Standard",
    "kind": "UserCreated",
    "description": "Configured by IT for all staff",
    "createdAt": "2026-04-01T09:00:00Z"
  },
  "settings": {
    "magnification": { "zoomLevel": 3.0, "mode": "Docked", ... },
    "colorFilter": { "filterType": "None", "brightness": 0.0, "contrast": 1.0 },
    ...
  }
}
```

**Git-friendliness:** The JSON format is deterministic (keys sorted alphabetically by the Rust serializer). This means profiles can be committed to a configuration repository and diffed meaningfully by IT departments managing accessibility configurations.

**Import validation:** The `import_profile` command validates the JSON against the `AppSettings` schema server-side using `serde`. Unknown fields are ignored (forward compatibility). Missing required fields return an error. The TypeScript side additionally runs `ProfileDocumentSchema.parse()` before calling `import_profile` to provide clear validation messages in the UI.

### 8.4 Profiles Page UI

```
Profiles
  ┌──────────────────────────────────────────┐
  │ Built-in                                  │
  │  [Default]          Load                  │
  │  [High Contrast]    Load                  │
  │  [Low Light]        Load                  │
  │  [Reading Mode]     Load                  │
  ├──────────────────────────────────────────┤
  │ Your Profiles                             │
  │  [Work Setup]      Load  Export  Delete   │
  │  [Night Use]       Load  Export  Delete   │
  │                                           │
  │  [ Save current settings as profile... ] │
  │  [ Import profile from file...         ] │
  └──────────────────────────────────────────┘
```

"Save current settings as profile" opens a dialog with a name field (required) and optional description. On confirm, calls `save_profile` and adds the result to the user profile list.

---

## 9. Voice Selection and TTS Controls

### 9.1 espeak-ng Warning Banner

If `check_espeak_available` returns `false`, the Speech page renders a prominent warning banner above all other controls:

```
┌──────────────────────────────────────────────────────────────┐
│ ⚠  Text-to-speech requires espeak-ng, which was not found.  │
│    Neural TTS (Kokoro, Piper) needs it for phonemization.   │
│                                                              │
│    Install with:  sudo apt install espeak-ng               │
│    (See documentation for macOS and other platforms)        │
└──────────────────────────────────────────────────────────────┘
```

The install command shown is platform-specific (populated by the `SystemInfo.os` field). The banner is dismissible but re-appears on next launch until espeak-ng is detected. If the platform-native TTS fallback is available (`TtsStatus !== 'Error'`), the banner includes a note: "Platform TTS is available as a fallback."

### 9.2 Voice List (`VoiceSelector` + `VoiceList`)

`list_voices` returns all available voices grouped by engine. The voice selector displays them in a grouped `<listbox>` (ARIA `role="listbox"`):

```
Speech Voice
┌──────────────────────────────────────────────┐
│ ◉ Heart (Kokoro · en-US)          [Default]  │  <- Currently active
│   Bella (Kokoro · en-US)                      │
│   Nichole (Kokoro · en-US)                    │
│   Michael (Kokoro · en-US)                    │
│   ─────────── French ─────────────           │
│   Céline (Kokoro · fr-FR)                     │
│   ─────────── German ──────────────           │
│   Klaus (Piper · de-DE)           [Install]  │  <- Not installed
│   ─────────── System ──────────────           │
│   System Default (Platform TTS)               │
└──────────────────────────────────────────────┘
```

**Grouping:** Voices are grouped first by language (BCP 47 display name), then by engine within a language. Languages with no installed voices are hidden unless the user enables "Show all available voices."

**"Install" affordance:** Uninstalled Piper or Kokoro voice models show an "Install" button instead of a radio button. Clicking it sends a download request (handled by the Rust engine's `ModelManager`). The `voice_model_loading` event streams progress updates to the `ModelDownloadProgress` component.

**Voice selection flow:**

```
User clicks voice "Klaus (Piper · de-DE)"
    |
    v
VoiceListItem calls invoke('set_voice', { voiceId: 'piper-de-thorsten-medium' })
    |
    v
Rust: TtsCoordinator queues model load (async, background)
    |
    v
Engine emits voice_model_loading { stage: 'Downloading', progress: 0.0 }
Engine emits voice_model_loading { stage: 'Loading',     progress: 0.8 }
Engine emits voice_model_loading { stage: 'Complete',    progress: 1.0 }
    |
    v
Panel: ModelDownloadProgress shows progress bar, then hides on Complete
Panel: TtsStatusIndicator shows 'Idle' (ready)
```

### 9.3 Model Download Progress (`ModelDownloadProgress`)

Displays a progress bar while a voice model is being downloaded or loaded. Only visible when `modelLoading !== null` in the TTS store.

```typescript
export function ModelDownloadProgress() {
  const modelLoading = useTtsStore(s => s.modelLoading);
  if (!modelLoading) return null;

  const { voiceId, stage, progress, errorMessage } = modelLoading;

  if (stage === 'Failed') {
    return (
      <div role="alert" aria-live="assertive" className="error-banner">
        Failed to load voice model: {errorMessage}
      </div>
    );
  }

  return (
    <div role="status" aria-live="polite" aria-label={`Loading voice model: ${Math.round(progress * 100)}%`}>
      <progress value={progress} max={1} />
      <span>{stage === 'Downloading' ? 'Downloading model...' : 'Loading model...'}</span>
    </div>
  );
}
```

### 9.4 TTS Status Indicator (`TtsStatusIndicator`)

A compact status badge showing the current TTS state. Always visible on the Speech page; also available as a tray tooltip (Phase 2).

| Status | Visual | ARIA |
|--------|--------|------|
| `Idle` | Grey dot "Ready" | `aria-live="polite"` |
| `Loading` | Spinner "Loading model…" | `aria-live="polite"` |
| `Speaking` | Animated waveform "Speaking" | `aria-live="off"` (do not interrupt) |
| `Draining` | Fading waveform "Finishing" | `aria-live="off"` |
| `Error` | Red dot "Error: {message}" | `aria-live="assertive"` |

### 9.5 Speech Controls

| Control | Type | Range | Default | IPC Command |
|---------|------|-------|---------|-------------|
| Speech rate | Slider | 0.5×–3.0× (step 0.1) | 1.0× | `set_speech_rate` |
| Volume | Slider | 0%–100% (step 5) | 100% | `set_speech_volume` |
| Model quality | Select (Q4 / Q8 / Fp16 / Fp32) | — | Q8 | `set_model_variant` |
| Enable TTS | Toggle | — | false | `save_settings` |

*The Enable TTS toggle exists in the `AppSettings` schema from Phase 0 (defaults to `false`) but is only functional from Phase 2, when the TTS pipeline is implemented. In Phase 0-1 it is hidden in the UI since the Speech page itself is a Phase 2 addition (Section 1.3).*

**Model quality tradeoffs** (shown as a tooltip/info panel next to the selector):

| Variant | Size | Quality | Speed | Use Case |
|---------|------|---------|-------|----------|
| Q4 | ~80MB | Good | Fastest | Low-memory or slow hardware |
| Q8 | ~92MB | Very good | Fast (default) | Most systems |
| Fp16 | ~163MB | Excellent | Moderate | Users prioritizing quality |
| Fp32 | ~327MB | Reference | Moderate | Development / quality comparison only |

*Model sizes match the figures in [04 -- TTS Pipeline](./04-tts-pipeline.md), Section 6.2 (onnx-community/Kokoro-82M-v1.0-ONNX distribution). Fp32 is selectable in the UI but shown with a "high memory usage" warning badge.*

Changing model variant triggers an asynchronous model reload. The `TtsStatusIndicator` shows `Loading` during the reload.

---

## 10. Performance and Diagnostics Panel

### 10.1 Frame Timing Display (`FrameTimingDisplay`)

The `get_frame_timings` command returns a `FrameTimingSummary` from the render pipeline's `FrameTimings` circular buffer (defined in [03 -- Rendering Pipeline](./03-rendering-pipeline.md), Section 8.3). The Diagnostics page polls this every 2 seconds when open:

```typescript
export function FrameTimingDisplay() {
  const [timings, setTimings] = useState<FrameTimingSummary | null>(null);

  useEffect(() => {
    const interval = setInterval(async () => {
      const raw = await getFrameTimings();
      setTimings(FrameTimingSummarySchema.parse(raw));
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  if (!timings) return <p>Loading frame timings...</p>;

  const isHealthy = timings.p99Ms < 20;

  return (
    <section aria-label="Frame timing">
      <dl>
        <dt>Average frame time</dt>
        <dd>{timings.averageMs.toFixed(1)} ms ({Math.round(1000 / timings.averageMs)} fps)</dd>
        <dt>P99 frame time</dt>
        <dd className={isHealthy ? 'ok' : 'warning'}>{timings.p99Ms.toFixed(1)} ms</dd>
        <dt>Min / Max</dt>
        <dd>{timings.minMs.toFixed(1)} / {timings.maxMs.toFixed(1)} ms</dd>
        <dt>Target FPS</dt>
        <dd>{timings.targetFps}</dd>
      </dl>
    </section>
  );
}
```

**P99 threshold:** A P99 frame time above 20ms is highlighted in amber (warning). Above 33ms (under 30fps) is highlighted in red. See [03 -- Rendering Pipeline](./03-rendering-pipeline.md), Section 8.3, for the degradation threshold logic on the Rust side.

### 10.2 Performance Degradation Warning

When the Rust engine detects P99 > 20ms for 5 consecutive seconds, it emits a `performance_warning` event. The control panel handles this in `App.tsx` (subscribed globally) and shows a non-modal toast notification:

```
┌──────────────────────────────────────────────────────────────────┐
│ ⚡ Performance notice                                             │
│    Frame rendering is slower than expected (P99: 24ms).          │
│    Suggestion: Switch to Performance mode or reduce zoom level.  │
│                                                                  │
│    [ Switch to Performance mode ]   [ Dismiss ]                  │
└──────────────────────────────────────────────────────────────────┘
```

The "Switch to Performance mode" button calls `set_present_mode({ mode: 'Performance' })` and updates the settings store. The `PerformanceWarningPayload.recommendation` field drives the button label:

| Recommendation | Button Action |
|----------------|---------------|
| `SwitchToPerformanceMode` | Call `set_present_mode('Performance')` |
| `ReduceZoomLevel` | Navigate to Magnification page and focus the zoom slider |
| `None` | No action button, just a dismiss |

### 10.3 System Info Panel (`SystemInfoPanel`)

Displays hardware and version information useful for bug reports:

```
┌─────────────────────────────────────┐
│ System Information                   │
│  Luminos version:  0.1.0            │
│  Operating system: Linux 6.8 (X11)  │
│  GPU:              Intel UHD 770    │
│  GPU backend:      Vulkan           │
│  System RAM:       16,384 MB        │
│  Process RAM:      412 MB           │
└─────────────────────────────────────┘
  [ Copy to clipboard ]
```

"Copy to clipboard" serializes the `SystemInfo` as formatted plain text and calls `navigator.clipboard.writeText()`.

### 10.4 Diagnostics Page Visibility

The Diagnostics page is visible in **all builds** but placed last in the sidebar (after Profiles). It is not hidden in release builds because frame timing information is genuinely useful to users troubleshooting performance issues. The Diagnostics sidebar entry is labelled "Performance & Info."

Debug-only features (e.g., per-frame capture buffer visualization) are gated behind `#[cfg(debug_assertions)]` on the Rust side and will simply not appear as available commands in release builds.

---

## 11. Startup and Initialization

### 11.1 Load Sequence

The control panel webview is loaded **in the background** after magnification is already usable. From the startup sequence in [01 -- System Architecture](./01-system-architecture.md), Section 9.4:

```
T=0ms    Process start
T=50ms   Parse config, initialize logging
T=100ms  Create winit event loop, overlay window
T=200ms  Initialize wgpu, rendering pipeline
T=300ms  Initialize ScreenCapture backend
T=400ms  First magnified frame rendered  <-- USER CAN SEE MAGNIFIED SCREEN
T=500ms  Input monitoring, focus tracking start
T=1000ms Tauri initializes; control panel webview loads (background)
T=1200ms React app mounts; calls get_current_settings; panel is ready
T=2000ms TTS model loading begins (background)
```

The user is never blocked from magnification by the control panel load. If the user opens the control panel before T=1200ms (unlikely but possible), the `HydrationGate` component shows a loading spinner while `isHydrating = true`. The spinner itself is accessible (ARIA `role="status"`, `aria-label="Loading settings…"`).

### 11.2 Settings Hydration

On mount, the `App` component calls `get_current_settings()` once to load the full settings snapshot. This is the canonical source of truth; the Zustand store is initialized from this response.

**Hydration guard pattern:**

```typescript
export function HydrationGate({ children }: { children: React.ReactNode }) {
  const isHydrating = useSettingsStore(s => s.isHydrating);

  if (isHydrating) {
    return (
      <div role="status" aria-label="Loading settings" aria-live="polite">
        <span aria-hidden="true" className="spinner" />
        <span className="sr-only">Loading settings…</span>
      </div>
    );
  }

  return <>{children}</>;
}
```

### 11.3 Hotkey State Synchronization

When the user changes settings via keyboard shortcuts (while the control panel may or may not be open), the Rust engine:

1. Applies the state change to `ArcSwap<AppState>`
2. Emits the appropriate event (`zoom_changed`, `mode_changed`, or `settings_changed`) via `app.emit()`

The TypeScript event listeners (registered during hydration in Section 5.4) receive these events and update the Zustand store. This ensures the control panel always displays current state even when it was minimized or closed while the user operated Luminos via hotkeys.

**Example:** User presses `Ctrl + =` to zoom in while control panel is open on the Magnification page. The zoom slider animates to the new value within one Tauri event round-trip (~1–5ms). The slider does not "fight" the hotkey because updates come through the store — both the slider's `onChange` and the `onZoomChanged` event write to the same `setZoomLevel` store action.

### 11.4 System Tray

The Tauri application registers a system tray icon (Phase 0). The tray icon provides:
- Left-click: Show/hide the control panel window
- Right-click context menu:
  - "Show Luminos" (if window hidden)
  - "Zoom In / Zoom Out" (quick access)
  - "Toggle Magnification" (on/off)
  - separator
  - "Quit"

The tray menu items that trigger magnification changes call Rust-side handlers directly (not through the webview). They follow the same state update pattern: mutate `ArcSwap<AppState>`, send `EventLoopProxy` event, emit Tauri event for panel sync.

**`minimizeToTray` setting:** When `minimizeToTray = true`, closing the control panel window hides it (minimizes to tray) rather than quitting. The application continues running with magnification active. Quit is only available via the tray context menu or `Ctrl+Q`.

---

## 12. Control Panel Accessibility

The control panel is used by people with low vision and other disabilities. It must meet WCAG 2.1 AA as a minimum requirement, with several AA+ provisions where feasible. An inaccessible control panel is a critical bug, not a cosmetic issue.

### 12.1 Keyboard Navigation

All interactive elements must be reachable and operable via keyboard alone:

| Requirement | Implementation |
|-------------|----------------|
| Full `Tab` order through all controls | Natural DOM order; no `tabindex` values except -1/0 |
| No keyboard traps | Modal dialogs trap focus internally; `Escape` always closes them and returns focus to trigger element |
| Sidebar navigation via arrow keys | Sidebar `<nav>` implements ARIA `role="tablist"` pattern with `ArrowUp`/`ArrowDown` navigation |
| Sliders via arrow keys | Native `<input type="range">` handles arrow key increment/decrement natively |
| Key capture input | `KeyCaptureInput` intercepts keydown and prevents default; `Escape` cancels |
| Skip navigation link | First focusable element is a visually-hidden "Skip to main content" link |

**Focus visibility:** Focus indicators must be visible at all zoom levels. The CSS focus outline uses `outline: 3px solid var(--focus-color)` with `outline-offset: 2px`. The focus color (`--focus-color`) is `#0078D4` on light backgrounds, `#60CDFF` on dark backgrounds. At 1px CSS = 5+ screen pixels when Luminos is magnifying at 5×, even a 3px outline is clearly visible.

### 12.2 Screen Reader Compatibility

The control panel must work alongside Orca (Linux), NVDA/JAWS (Windows), and VoiceOver (macOS). Since these screen readers share the screen with Luminos, they may be announcing control panel content while Luminos magnifies their output -- an intentional, expected use case.

| Requirement | Implementation |
|-------------|----------------|
| All form controls have accessible names | `<label>` + `for`, or `aria-labelledby`, or `aria-label` |
| Status changes announced | `aria-live="polite"` on TTS status, model loading, and zoom readout |
| Error messages announced | `aria-live="assertive"` on error banners and toasts |
| Grouped controls have group labels | `<fieldset>` + `<legend>` for radio groups (mode selector, dock edge) |
| Dynamic content updates | `aria-live` regions on all panels that update without navigation |
| No purely visual information | Icons are always accompanied by text labels or `aria-label` |

**Zoom level live region:** The zoom percentage readout at the top of the Magnification page is wrapped in an `aria-live="polite"` region so screen reader users hear the current zoom as it changes during slider drag or hotkey use.

```html
<output aria-live="polite" aria-atomic="true" aria-label="Current zoom level">
  5.0×
</output>
```

### 12.3 High-Contrast Mode

The control panel must be visually usable under OS-level high-contrast themes (GNOME High Contrast, Windows High Contrast, macOS Increased Contrast). This is implemented via CSS media queries:

```css
@media (forced-colors: active) {
  /* Forced colors mode (Windows High Contrast):
     the browser overrides colors; we ensure no information is conveyed by color alone */
  .status-badge { border: 2px solid ButtonText; }
  .progress-bar { forced-color-adjust: none; }  /* Preserve progress semantics */
}

@media (prefers-contrast: more) {
  /* Increase contrast beyond normal (macOS Increased Contrast, GNOME) */
  :root {
    --focus-color: #000000;
    --border-color: #000000;
    --text-muted: #333333;
  }
}
```

All UI state (error, warning, ok) is communicated through both **color and text/icon**, never color alone.

### 12.4 Text Size and Zoom

The control panel's own UI must remain usable when:
- The OS font size is increased (e.g., GNOME "Large Text" at 125–150%)
- The system-level zoom is active (e.g., macOS Display Zoom)
- The user opens Luminos's own magnification on the control panel

**Implementation:** All font sizes use `rem` units (relative to the root font size, which respects the OS font scale). No pixel-fixed heights for interactive elements — use `min-height` with padding instead. All layouts use flexbox/grid with wrapping enabled.

**Minimum font size:** 14px equivalent (0.875rem at default 16px root). Labels and descriptions are 14px; headings are 16–20px.

### 12.5 Motion and Animation

The control panel uses minimal animation. Where animation is used (e.g., toast slide-in, spinner):

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

The TTS waveform animation in `TtsStatusIndicator` is paused when `prefers-reduced-motion: reduce` is active, showing a static icon instead.

---

## 13. Testing Strategy

### 13.1 Test Toolchain

| Tool | Purpose | Version |
|------|---------|---------|
| Vitest | Unit and component test runner | Latest |
| React Testing Library | Component rendering and interaction | Latest |
| `@tauri-apps/api/mocks` | Mock Tauri IPC calls in tests | Latest |
| zod | Runtime validation (already in production code) | v3 |

### 13.2 Unit Tests (TypeScript)

**What to test:** Pure functions, Zod schema validation, store logic, and IPC wrapper functions.

| Module | Test Suite | Example Tests |
|--------|-----------|---------------|
| `types/settings.ts` | `settings.schema.test.ts` | `settings_schema_rejects_zoom_below_minimum`, `settings_schema_rejects_invalid_hex_color`, `settings_schema_accepts_valid_complete_settings` |
| `types/tts.ts` | `tts.schema.test.ts` | `tts_voice_info_schema_rejects_missing_id`, `tts_status_payload_parses_error_with_message` |
| `types/profiles.ts` | `profiles.schema.test.ts` | `profile_document_rejects_wrong_version`, `profile_info_rejects_empty_name` |
| `hooks/useSettingsStore.ts` | `settings.store.test.ts` | `settings_store_hydrate_sets_is_hydrating_false`, `settings_store_set_zoom_clamps_to_bounds`, `settings_store_optimistic_zoom_reverts_on_error` |
| `hooks/useTtsStore.ts` | `tts.store.test.ts` | `tts_store_set_status_speaking_updates_voice_id`, `tts_store_model_loading_sets_null_on_complete` |
| `constants/defaults.ts` | `defaults.test.ts` | `default_settings_are_valid_against_schema` |

**Test naming convention:** `{module}_{behavior}_{condition}` — consistent with the Rust convention in CLAUDE.md.

### 13.3 Component Tests

Components are tested with React Testing Library in JSDOM. Tauri IPC calls are mocked via `@tauri-apps/api/mocks`:

```typescript
// ui/src/components/magnification/ZoomLevelSlider.test.tsx
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';
import { ZoomLevelSlider } from './ZoomLevelSlider';

beforeEach(() => {
  mockIPC(cmd => {
    if (cmd === 'set_zoom_level') return Promise.resolve(null);
  });
});
afterEach(clearMocks);

test('zoom_slider_renders_current_zoom_level', () => {
  render(<ZoomLevelSlider />, {
    wrapper: ({ children }) => (
      <SettingsStoreProvider initial={{ magnification: { zoomLevel: 5.0, ... } }}>
        {children}
      </SettingsStoreProvider>
    ),
  });
  expect(screen.getByRole('slider', { name: /zoom level/i })).toHaveValue('5');
  expect(screen.getByText('5.0×')).toBeInTheDocument();
});

test('zoom_slider_calls_ipc_on_change', async () => {
  const user = userEvent.setup();
  const invokeSpy = vi.fn().mockResolvedValue(null);
  mockIPC(cmd => cmd === 'set_zoom_level' ? invokeSpy() : Promise.resolve(null));

  render(<ZoomLevelSlider />, { /* wrapper */ });
  await user.type(screen.getByRole('slider'), '{ArrowRight}');

  expect(invokeSpy).toHaveBeenCalled();
});

test('zoom_slider_reverts_on_ipc_error', async () => {
  mockIPC(cmd => {
    if (cmd === 'set_zoom_level') return Promise.reject(new Error('Engine error'));
  });
  // ... verify store reverts and error toast is shown
});
```

| Component | Example Tests |
|-----------|---------------|
| `ZoomLevelSlider` | `zoom_slider_renders_current_zoom_level`, `zoom_slider_clamps_input_to_bounds`, `zoom_slider_reverts_on_ipc_error` |
| `VoiceSelector` | `voice_selector_groups_voices_by_language`, `voice_selector_shows_install_button_for_uninstalled`, `voice_selector_triggers_download_on_install_click` |
| `ColorFilterPanel` | `color_filter_panel_shows_preset_schemes_for_high_contrast`, `color_filter_panel_disables_sliders_when_filter_is_none` |
| `TtsStatusIndicator` | `tts_status_indicator_shows_speaking_state`, `tts_status_indicator_announces_error_assertively` |
| `KeybindingTable` | `keybinding_table_renders_all_actions`, `keybinding_table_captures_key_combination`, `keybinding_table_escape_cancels_capture` |
| `ProfileCard` | `profile_card_shows_load_button_for_user_profiles`, `profile_card_hides_delete_for_built_in_profiles` |
| `FrameTimingDisplay` | `frame_timing_display_highlights_p99_when_above_threshold` |

### 13.4 IPC Integration Tests

Integration tests verify that TypeScript types match the actual Rust serde serialization. They run against a real Tauri process in CI (using `tauri-driver` + WebDriver):

```
test/integration/
  ipc_settings_roundtrip.test.ts   -- get_current_settings → modify → save → reload → compare
  ipc_tts_commands.test.ts         -- list_voices, set_voice, speak_text, stop_speech
  ipc_profiles.test.ts             -- save_profile, list_profiles, load_profile, export, import
  ipc_type_compatibility.test.ts   -- Verifies every Zod schema accepts the actual engine response
```

**`ipc_type_compatibility.test.ts`** guards against IPC drift: every command response is parsed through its Zod schema. If Rust adds a field or renames an enum variant without updating the TypeScript schema, this test fails.

### 13.5 Accessibility Tests

Automated accessibility checks run via `axe-core` integrated with React Testing Library:

```typescript
import { checkA11y } from 'axe-playwright';  // or @axe-core/react for unit tests

test('magnification_page_has_no_accessibility_violations', async () => {
  const { container } = render(<MagnificationPage />);
  const results = await axe(container);
  expect(results.violations).toHaveLength(0);
});
```

Automated axe checks catch: missing labels, insufficient color contrast, keyboard traps, missing ARIA roles. Manual screen reader testing with Orca is required before each major release and is tracked in the release checklist (see [07 -- Testing Strategy](./07-testing-strategy.md)).

---

## 14. Module Organization

### 14.1 Rust: `luminos-app/src/tauri_commands.rs`

This file is the sole location for `#[tauri::command]` functions. It is the translation layer between Tauri IPC and the `LuminosHandle`. No business logic lives here — it delegates to `luminos-core` and `luminos-tts`.

```
crates/luminos-app/
  Cargo.toml
  src/
    main.rs               # Startup: winit loop, Tauri builder, LuminosHandle registration
    overlay.rs            # Magnification overlay window setup (winit + wgpu)
    tauri_commands.rs     # All #[tauri::command] functions (IPC handlers)
    events.rs             # LuminosEvent enum (sent via EventLoopProxy to winit loop)
    tray.rs               # System tray icon and menu setup
```

`tauri_commands.rs` dependencies: `luminos-core` (for `AppState`, `ConfigManager`), `luminos-tts` (for `TtsCoordinator` handle), `luminos-platform` (for `Voice` types). It does **not** depend on `luminos-gpu` — GPU resources are owned by the render thread and are not accessible from IPC handlers.

### 14.2 TypeScript: `ui/src/` Directory

```
ui/
  package.json                   # { "name": "luminos-ui", "type": "module" }
  tsconfig.json                  # Strict mode, path aliases (@/ -> src/)
  vite.config.ts                 # Tauri + React plugin configuration
  src/
    main.tsx                     # ReactDOM.createRoot entry point
    App.tsx                      # Root: router, hydration, global event subscriptions

    ipc/
      bindings.ts                # AUTO-GENERATED by tauri-specta (do not edit manually)
      commands.ts                # Typed wrappers with Zod validation for complex responses
      events.ts                  # listen() helpers for all engine events

    types/
      enums.ts                   # Zod enums: MagnificationMode, TtsStatus, etc.
      settings.ts                # Zod schemas: AppSettings, MagnificationSettings, etc.
      tts.ts                     # Zod schemas: VoiceInfo, TtsStatusPayload, etc.
      diagnostics.ts             # Zod schemas: FrameTimingSummary, SystemInfo, etc.
      profiles.ts                # Zod schemas: ProfileInfo, ProfileDocument, etc.

    hooks/
      useSettingsStore.ts        # Zustand store: AppSettings, hydration, optimistic updates
      useTtsStore.ts             # Zustand store: TtsStatus, voice list, model loading
      useProfilesStore.ts        # Zustand store: profile list, active profile
      useToast.ts                # Toast notification hook (error, warning, info)
      useDebounce.ts             # Generic debounce hook for slider IPC calls

    constants/
      defaults.ts                # DEFAULT_SETTINGS: AppSettings compiled-in defaults
      zoom.ts                    # MIN_ZOOM, MAX_ZOOM, ZOOM_STEP, ZOOM_PRESETS
      colors.ts                  # HIGH_CONTRAST_SCHEMES: preset 4x4 color matrices
      keybindings.ts             # DEFAULT_KEYBINDINGS: HotkeyAction -> KeyBinding map

    components/
      Shell.tsx                  # Sidebar + Outlet layout wrapper
      HydrationGate.tsx          # Loading spinner until settings are loaded
      Sidebar.tsx                # Navigation (ARIA tablist pattern)
      ToastProvider.tsx          # Global toast notification system

      shared/
        Slider.tsx               # Accessible range slider (label + output)
        ToggleSwitch.tsx         # Accessible toggle (styled checkbox)
        Select.tsx               # Accessible dropdown
        ColorPicker.tsx          # Hex color input with preview swatch
        SettingsRow.tsx          # Label + control row layout
        SectionHeader.tsx        # Page section heading
        StatusBadge.tsx          # Color + text status indicator

      magnification/
        ZoomLevelSlider.tsx
        MagnificationModeSelector.tsx
        TrackingModeSelector.tsx
        DockedModeControls.tsx   # Edge + size controls (shown when mode = Docked)
        LensModeControls.tsx     # Width + height + shape (shown when mode = Lens)

      display/
        ColorFilterPanel.tsx
        ColorSchemePresets.tsx   # Built-in high-contrast scheme swatches
        CursorEnhancementPanel.tsx
        RenderingControls.tsx    # FPS target, present mode, interpolation, GPU pref

      speech/
        EspeakWarningBanner.tsx
        VoiceSelector.tsx
        VoiceList.tsx
        VoiceListItem.tsx
        ModelDownloadProgress.tsx
        SpeechRateSlider.tsx
        SpeechVolumeSlider.tsx
        ModelVariantSelector.tsx
        TtsStatusIndicator.tsx

      keybindings/
        KeybindingTable.tsx
        KeybindingRow.tsx
        KeyCaptureInput.tsx      # Intercepts keypress; displays captured combination

      profiles/
        BuiltInProfileList.tsx
        UserProfileList.tsx
        ProfileCard.tsx
        SaveProfileDialog.tsx    # Name + description form; calls save_profile
        ImportExportControls.tsx # File picker + JSON textarea

      diagnostics/
        FrameTimingDisplay.tsx
        SystemInfoPanel.tsx
        EspeakStatusPanel.tsx    # espeak-ng subprocess status + install instructions

    pages/
      MagnificationPage.tsx
      DisplayPage.tsx
      SpeechPage.tsx
      KeybindingsPage.tsx
      ProfilesPage.tsx
      DiagnosticsPage.tsx

    styles/
      globals.css                # CSS reset, font stack, base element styles
      variables.css              # CSS custom properties: colors, spacing, radii
      themes/
        light.css                # Light theme variable values
        dark.css                 # Dark theme variable values (prefers-color-scheme)
        high-contrast.css        # High contrast overrides (forced-colors: active)
```

### 14.3 Dependencies (`ui/package.json`)

| Package | Version | Purpose |
|---------|---------|---------|
| `react`, `react-dom` | 19+ | UI framework |
| `react-router` | 7 | In-panel routing (hash router) |
| `zustand` | 5 | State management |
| `immer` | Latest | Immutable state updates in Zustand |
| `zod` | 3 | Schema validation for IPC responses and form input |
| `use-debounce` | Latest | Debounced IPC calls for sliders |
| `@tauri-apps/api` | 2 | Tauri IPC (`invoke`, `listen`) |
| `@tauri-apps/plugin-shell` | 2 | Open links in system browser (documentation) |

**Dev dependencies:**

| Package | Purpose |
|---------|---------|
| `vitest` | Test runner |
| `@testing-library/react` | Component testing utilities |
| `@testing-library/user-event` | Simulates user interactions in tests |
| `axe-core` | Automated accessibility checks |
| `@tauri-apps/api/mocks` | Mock IPC calls in tests |
| `typescript` | TypeScript compiler |
| `vite` + `@vitejs/plugin-react` | Build tooling |

---

## 15. Cross-References

| Topic | Document | Section |
|-------|----------|---------|
| Dual-window design and IPC role | [01 -- System Architecture](./01-system-architecture.md) | 3.3, 4.7 |
| Settings data flow (IPC → ArcSwap → render thread) | [01 -- System Architecture](./01-system-architecture.md) | 5.4 |
| EventLoopProxy / winit integration | [01 -- System Architecture](./01-system-architecture.md) | 6.5 |
| Thread model and IPC thread constraints | [01 -- System Architecture](./01-system-architecture.md) | 6.2, 6.3 |
| Configuration Manager (ArcSwap, persistence) | [01 -- System Architecture](./01-system-architecture.md) | 4.6 |
| Startup sequence (T=1000ms webview load) | [01 -- System Architecture](./01-system-architecture.md) | 9.4 |
| Memory budget (Tauri webview ~30–50MB) | [01 -- System Architecture](./01-system-architecture.md) | 9.3 |
| Cargo workspace layout (`luminos-app`, `ui/`) | [01 -- System Architecture](./01-system-architecture.md) | 7.1 |
| Platform permission model | [01 -- System Architecture](./01-system-architecture.md) | 10.3 |
| `FrameTimings` struct and P99 threshold | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 8.3 |
| Performance degradation event emission | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 8.3 |
| Interpolation modes (Bilinear vs Bicubic) | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 6.2 |
| Present modes (Fifo / Mailbox / Immediate) | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 8.1 |
| GPU device selection (LowPower preference) | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 9.1 |
| Color filter shader and preset matrices | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 6.3 |
| Cursor enhancement uniforms | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 6.4 |
| Zoom mode rendering (full-screen, lens, docked) | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 7 |
| `Voice`, `TtsBackend`, `VoiceInfo` Rust types | [02 -- Platform Abstraction](./02-platform-abstraction.md) | 3.4 |
| `TtsEngine::get_voices()` implementation | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 8.5 |
| Voice model manifest and storage paths | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 8.1, 8.2 |
| Model loading lifecycle and TTS startup | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 8.3 |
| espeak-ng availability detection | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 5.5 |
| `TtsStatus` state machine | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 13.2 |
| `TextSource::ControlPanel` enum variant | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 3.1 |
| Speech queue management | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 13 |
| Kokoro model variant sizes | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 6.2 |
| Phase 2 TTS features (voice control, word highlighting) | [Product Strategy](../PRODUCT_STRATEGY.md) | 7.3 |
| Phase 1 settings persistence and profiles | [Product Strategy](../PRODUCT_STRATEGY.md) | 7.2 |
| Tauri 2.0 selection rationale | [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) | 3.1 |
| Testing strategy (Tauri integration tests) | [07 -- Testing Strategy](./07-testing-strategy.md) | Section 4.5 |
| Consolidated performance targets (IPC latency) | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 2.1 |
| Tauri security configuration (capability-based permissions) | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 3.4 |
| WCAG compliance strategy | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 5 |
| Logging and observability | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 6 |
| Error handling policy | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 7 |
| Internationalization strategy | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 8 |

---

## 16. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-16 | Initial control panel strategy |
| 1.1 | 2026-03-16 | Post-audit revision: resolved tauri-specta Builder vs generate_handler![] architectural conflict (use Builder as invoke handler throughout); fixed tauri-specta/specta-typescript from dev-dependencies to dependencies; updated specta-typescript to 0.0.9; corrected export syntax in commands.ts (property-access expressions are invalid in ES named export clauses); fixed set_zoom_level illustrative example to use LuminosHandle (not AppHandle/EventLoopProxyHandle); corrected Q4 Kokoro model size from ~50MB to ~80MB (per doc 04 Section 6.2); fixed fp16 → Fp16 PascalCase and added Fp32 row to model variant table; added error recovery path to getCurrentSettings() hydration; removed unused pendingKeys field from settings store; added note locating Rust event type definitions (events.rs); clarified diagnostics phase attribution (basic display Phase 0, full chart Phase 3); fixed FrameTimingDisplay to use named import instead of commands.* namespace |
