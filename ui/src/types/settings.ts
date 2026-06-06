import { z } from 'zod';

import {
  colorFilterTypeSchema,
  dockEdgeSchema,
  gpuPreferenceSchema,
  hotkeyActionSchema,
  interpolationModeSchema,
  lensShapeSchema,
  magnificationModeSchema,
  modelVariantSchema,
  modifierKeySchema,
  presentModeSchema,
  trackingModeSchema,
  ZOOM_MAX,
  ZOOM_MIN,
} from './enums';

/**
 * `AppSettings` Zod schema — the runtime contract for the IPC settings payload.
 *
 * IMPORTANT — wire format: the Rust `AppSettings` (and its sub-structs in
 * `luminos-core::config::schema`) derive `serde` with the DEFAULT field
 * naming. They carry **no** `#[serde(rename_all = ...)]`, so JSON keys are
 * **snake_case** (`zoom_level`, `color_filter`, `tracking_mode`, ...) while
 * enum *values* are PascalCase strings (see `./enums`). These schemas mirror
 * that exactly; story 005's `tauri-specta` bindings MUST preserve the same
 * shape (do not add a camelCase rename to `AppSettings` without updating both
 * sides). This snake_case contract is recorded in the epic Shared Context.
 */

/** Mirrors `luminos_core::config::schema::KeyBinding`. */
export const keyBindingSchema = z.object({
  key: z.string(),
  modifiers: z.array(modifierKeySchema),
});
export type KeyBinding = z.infer<typeof keyBindingSchema>;

/** Mirrors `luminos_core::config::schema::MagnificationSettings`. */
export const magnificationSettingsSchema = z.object({
  zoom_level: z.number().min(ZOOM_MIN).max(ZOOM_MAX),
  mode: magnificationModeSchema,
  tracking_mode: trackingModeSchema,
  docked_edge: dockEdgeSchema.nullable(),
  docked_size_percent: z.number().int().nullable(),
  lens_width: z.number().int().nullable(),
  lens_height: z.number().int().nullable(),
  lens_shape: lensShapeSchema.nullable(),
  target_fps: z.number().int(),
  present_mode: presentModeSchema,
  gpu_preference: gpuPreferenceSchema,
  interpolation: interpolationModeSchema,
  smooth_scrolling: z.boolean(),
});
export type MagnificationSettings = z.infer<typeof magnificationSettingsSchema>;

/** Mirrors `luminos_core::config::schema::ColorFilterConfig`. */
export const colorFilterConfigSchema = z.object({
  filter_type: colorFilterTypeSchema,
  brightness: z.number(),
  contrast: z.number(),
  color_matrix: z.array(z.number()).length(16).nullable(),
});
export type ColorFilterConfig = z.infer<typeof colorFilterConfigSchema>;

/** Mirrors `luminos_core::config::schema::CursorConfig`. */
export const cursorConfigSchema = z.object({
  enlarged_cursor: z.boolean(),
  cursor_scale: z.number(),
  crosshairs_enabled: z.boolean(),
  crosshair_width: z.number().int(),
  crosshair_color: z.string(),
  halo_enabled: z.boolean(),
  halo_radius: z.number().int(),
  halo_color: z.string(),
});
export type CursorConfig = z.infer<typeof cursorConfigSchema>;

/** Mirrors `luminos_core::config::schema::SpeechSettings`. */
export const speechSettingsSchema = z.object({
  enabled: z.boolean(),
  voice_id: z.string(),
  speech_rate: z.number(),
  speech_volume: z.number(),
  model_variant: modelVariantSchema,
});
export type SpeechSettings = z.infer<typeof speechSettingsSchema>;

/**
 * Keybindings map: a serde `HashMap<HotkeyAction, Option<KeyBinding>>`
 * serializes to a JSON object keyed by the PascalCase action name with a
 * `KeyBinding` or `null` value. The map is sparse (not every action is
 * bound), so `z.partialRecord` is required — `z.record` over an enum key
 * would make every action mandatory.
 */
export const keybindingsSchema = z.partialRecord(hotkeyActionSchema, keyBindingSchema.nullable());
export type Keybindings = z.infer<typeof keybindingsSchema>;

/** Mirrors `luminos_core::config::schema::AppSettings`. */
export const appSettingsSchema = z.object({
  magnification: magnificationSettingsSchema,
  color_filter: colorFilterConfigSchema,
  cursor: cursorConfigSchema,
  speech: speechSettingsSchema,
  keybindings: keybindingsSchema,
  start_on_login: z.boolean(),
  minimize_to_tray: z.boolean(),
  show_panel_on_start: z.boolean(),
});
export type AppSettings = z.infer<typeof appSettingsSchema>;

/**
 * `FrameTimingSummary` Zod schema.
 *
 * Unlike `AppSettings`, story 005 adds `#[serde(rename_all = "camelCase")]`
 * to the Rust `FrameTimingSummary` (it has no serde derive today; DC-5), so
 * its JSON keys are camelCase: `averageMs`, `p99Ms`, `minMs`, `maxMs`,
 * `targetFps`. This asymmetry is intentional and recorded in Shared Context.
 */
export const frameTimingSummarySchema = z.object({
  averageMs: z.number(),
  p99Ms: z.number(),
  minMs: z.number(),
  maxMs: z.number(),
  targetFps: z.number().int(),
});
export type FrameTimingSummary = z.infer<typeof frameTimingSummarySchema>;
