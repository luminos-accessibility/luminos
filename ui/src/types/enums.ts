import { z } from 'zod';

/**
 * Enum schemas mirroring the Rust wire format.
 *
 * The Rust enums in `luminos-types` / `luminos-core` derive `serde` with the
 * DEFAULT (externally-tagged, unrenamed) representation, so each variant
 * serializes to its **PascalCase** identifier as a bare JSON string
 * (e.g. `"FullScreen"`, not `"full_screen"`). These schemas must stay in
 * lock-step with those Rust definitions; story 005's `tauri-specta` bindings
 * generate the same string-literal unions.
 */

/** Screen magnification display mode (`luminos_types::MagnificationMode`). */
export const magnificationModeSchema = z.enum(['FullScreen', 'Docked', 'Lens']);
export type MagnificationMode = z.infer<typeof magnificationModeSchema>;

/** Which element the viewport tracks (`luminos_types::TrackingMode`). */
export const trackingModeSchema = z.enum(['Cursor', 'Focus', 'TextCaret']);
export type TrackingMode = z.infer<typeof trackingModeSchema>;

/** Color filter applied to the magnified view (`luminos_types::ColorFilterType`). */
export const colorFilterTypeSchema = z.enum([
  'None',
  'Invert',
  'SmartInvert',
  'Grayscale',
  'HighContrast',
  'Custom',
]);
export type ColorFilterType = z.infer<typeof colorFilterTypeSchema>;

/** VSync / presentation strategy (`luminos_types::PresentMode`). */
export const presentModeSchema = z.enum(['Quality', 'LowLatency', 'Performance']);
export type PresentMode = z.infer<typeof presentModeSchema>;

/** GPU device preference (`luminos_types::GpuPreference`). */
export const gpuPreferenceSchema = z.enum(['LowPower', 'HighPerformance']);
export type GpuPreference = z.infer<typeof gpuPreferenceSchema>;

/** Scaling interpolation algorithm (`luminos_types::InterpolationMode`). */
export const interpolationModeSchema = z.enum(['Bilinear', 'Bicubic']);
export type InterpolationMode = z.infer<typeof interpolationModeSchema>;

/** Docked-mode screen edge (`luminos_types::DockEdge`). */
export const dockEdgeSchema = z.enum(['Top', 'Bottom', 'Left', 'Right']);
export type DockEdge = z.infer<typeof dockEdgeSchema>;

/** Lens boundary shape (`luminos_types::LensShape`). */
export const lensShapeSchema = z.enum(['Rectangle', 'Ellipse']);
export type LensShape = z.infer<typeof lensShapeSchema>;

/** Kokoro ONNX quantization variant (`luminos_core::config::ModelVariant`). */
export const modelVariantSchema = z.enum(['Q4', 'Q8', 'Fp16', 'Fp32']);
export type ModelVariant = z.infer<typeof modelVariantSchema>;

/** Hotkey action identifiers (`luminos_core::config::HotkeyAction`). */
export const hotkeyActionSchema = z.enum([
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
export type HotkeyAction = z.infer<typeof hotkeyActionSchema>;

/** Modifier key names for keybindings (`luminos_core::config::ModifierKey`). */
export const modifierKeySchema = z.enum(['Ctrl', 'Shift', 'Alt', 'Super', 'Meta']);
export type ModifierKey = z.infer<typeof modifierKeySchema>;

/**
 * Inclusive zoom bounds shared by the engine (`StateManager` clamps to this
 * range) and the UI slider. Keep in sync with `luminos-core`'s clamp.
 */
export const ZOOM_MIN = 1.5;
export const ZOOM_MAX = 20;
export const ZOOM_STEP = 0.5;
