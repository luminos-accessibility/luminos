import type { AppSettings } from '../types/settings';

/**
 * Frontend mirror of Rust `AppSettings::default()`
 * (`luminos-core::config::schema`). Used as the hydration fallback when the
 * `get_current_settings` IPC call fails, so the panel never renders blank.
 *
 * These values MUST track the Rust defaults; the schema's accept test and the
 * `default_settings_are_valid_against_schema` test guard against drift, and
 * story 005's bindings/round-trip is the cross-language check.
 */
export const DEFAULT_SETTINGS: AppSettings = {
  magnification: {
    zoom_level: 2.0,
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
  color_filter: {
    filter_type: 'None',
    brightness: 0.0,
    contrast: 1.0,
    color_matrix: null,
  },
  cursor: {
    enlarged_cursor: false,
    cursor_scale: 1.0,
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
    speech_rate: 1.0,
    speech_volume: 1.0,
    model_variant: 'Q8',
  },
  keybindings: {},
  start_on_login: false,
  minimize_to_tray: false,
  show_panel_on_start: true,
};
