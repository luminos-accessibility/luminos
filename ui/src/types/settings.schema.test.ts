import { describe, expect, test } from 'vitest';

import { DEFAULT_SETTINGS } from '../constants/defaults';
import { appSettingsSchema, frameTimingSummarySchema } from './settings';

describe('settings.schema', () => {
  test('app_settings_schema_accepts_default_settings', () => {
    // The frontend DEFAULT_SETTINGS mirrors Rust AppSettings::default(); it
    // must always satisfy the schema (guards default/schema drift).
    const result = appSettingsSchema.safeParse(DEFAULT_SETTINGS);
    expect(result.success).toBe(true);
  });

  test('app_settings_schema_accepts_snake_case_wire_payload', () => {
    // Mirrors the exact JSON serde produces for AppSettings (snake_case keys,
    // PascalCase enum values, null for None options).
    const wirePayload = {
      magnification: {
        zoom_level: 5.0,
        mode: 'FullScreen',
        tracking_mode: 'Cursor',
        docked_edge: null,
        docked_size_percent: null,
        lens_width: null,
        lens_height: null,
        lens_shape: null,
        target_fps: 60,
        present_mode: 'Quality',
        gpu_preference: 'LowPower',
        interpolation: 'Bilinear',
        smooth_scrolling: true,
      },
      color_filter: { filter_type: 'None', brightness: 0, contrast: 1, color_matrix: null },
      cursor: {
        enlarged_cursor: false,
        cursor_scale: 1,
        crosshairs_enabled: false,
        crosshair_width: 2,
        crosshair_color: '#ff0000',
        halo_enabled: false,
        halo_radius: 50,
        halo_color: '#ffff0080',
      },
      speech: {
        enabled: false,
        voice_id: '',
        speech_rate: 1,
        speech_volume: 1,
        model_variant: 'Q8',
      },
      keybindings: { ZoomIn: { key: 'Equal', modifiers: ['Ctrl'] }, ZoomOut: null },
      start_on_login: false,
      minimize_to_tray: true,
      show_panel_on_start: true,
    };
    expect(appSettingsSchema.safeParse(wirePayload).success).toBe(true);
  });

  test('app_settings_schema_rejects_camel_case_zoom_field', () => {
    // Catches an accidental camelCase rename on the Rust side (contract guard).
    const camel = { ...DEFAULT_SETTINGS, magnification: { zoomLevel: 5.0 } };
    expect(appSettingsSchema.safeParse(camel).success).toBe(false);
  });

  test('app_settings_schema_rejects_zoom_below_minimum', () => {
    const invalid = {
      ...DEFAULT_SETTINGS,
      magnification: { ...DEFAULT_SETTINGS.magnification, zoom_level: 0.5 },
    };
    const result = appSettingsSchema.safeParse(invalid);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0]?.path).toContain('zoom_level');
    }
  });

  test('app_settings_schema_rejects_zoom_above_maximum', () => {
    const invalid = {
      ...DEFAULT_SETTINGS,
      magnification: { ...DEFAULT_SETTINGS.magnification, zoom_level: 99 },
    };
    expect(appSettingsSchema.safeParse(invalid).success).toBe(false);
  });

  test('frame_timing_summary_schema_accepts_camel_case_payload', () => {
    // FrameTimingSummary is the ONE IPC type renamed to camelCase (story 005).
    const payload = { averageMs: 8.1, p99Ms: 14.2, minMs: 6.0, maxMs: 19.9, targetFps: 60 };
    expect(frameTimingSummarySchema.safeParse(payload).success).toBe(true);
  });

  test('frame_timing_summary_schema_rejects_snake_case_payload', () => {
    const snake = { average_ms: 8.1, p99_ms: 14.2, min_ms: 6.0, max_ms: 19.9, target_fps: 60 };
    expect(frameTimingSummarySchema.safeParse(snake).success).toBe(false);
  });
});
