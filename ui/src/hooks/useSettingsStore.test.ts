import { beforeEach, describe, expect, test } from 'vitest';

import { DEFAULT_SETTINGS } from '../constants/defaults';
import { useSettingsStore } from './useSettingsStore';

/**
 * Store unit tests. The store is tested in isolation via getState/setState
 * (no component render). The Zustand auto-reset mock restores initial state
 * between tests, so each test starts from the store's defaults.
 */
describe('useSettingsStore', () => {
  beforeEach(() => {
    // Start each test from a known hydrated baseline where relevant.
    useSettingsStore.setState({ settings: DEFAULT_SETTINGS, isHydrating: true });
  });

  test('settings_store_initial_state_is_hydrating', () => {
    // Fresh store (before any hydrate) must signal that it is still hydrating.
    useSettingsStore.setState(useSettingsStore.getInitialState(), true);
    expect(useSettingsStore.getState().isHydrating).toBe(true);
  });

  test('settings_store_hydrate_sets_is_hydrating_false', () => {
    useSettingsStore.getState().hydrate(DEFAULT_SETTINGS);
    expect(useSettingsStore.getState().isHydrating).toBe(false);
  });

  test('settings_store_hydrate_populates_settings', () => {
    const custom = {
      ...DEFAULT_SETTINGS,
      magnification: { ...DEFAULT_SETTINGS.magnification, zoom_level: 6.0 },
    };
    useSettingsStore.getState().hydrate(custom);
    expect(useSettingsStore.getState().settings.magnification.zoom_level).toBe(6.0);
  });

  test('settings_store_set_zoom_clamps_to_maximum', () => {
    useSettingsStore.getState().setZoomLevel(100);
    expect(useSettingsStore.getState().settings.magnification.zoom_level).toBe(20);
  });

  test('settings_store_set_zoom_clamps_to_minimum', () => {
    useSettingsStore.getState().setZoomLevel(0.1);
    expect(useSettingsStore.getState().settings.magnification.zoom_level).toBe(1.5);
  });

  test('settings_store_set_zoom_keeps_in_range_value', () => {
    useSettingsStore.getState().setZoomLevel(7.5);
    expect(useSettingsStore.getState().settings.magnification.zoom_level).toBe(7.5);
  });

  test('settings_store_set_mode_updates_mode', () => {
    useSettingsStore.getState().setMode('Lens');
    expect(useSettingsStore.getState().settings.magnification.mode).toBe('Lens');
  });

  test('settings_store_apply_engine_update_replaces_settings', () => {
    const replacement = {
      ...DEFAULT_SETTINGS,
      magnification: { ...DEFAULT_SETTINGS.magnification, zoom_level: 12.0, mode: 'Docked' as const },
    };
    useSettingsStore.getState().applyEngineUpdate(replacement);
    const state = useSettingsStore.getState();
    expect(state.settings.magnification.zoom_level).toBe(12.0);
    expect(state.settings.magnification.mode).toBe('Docked');
  });

  test('settings_store_set_zoom_does_not_mutate_previous_state_object', () => {
    const before = useSettingsStore.getState().settings;
    useSettingsStore.getState().setZoomLevel(9.0);
    const after = useSettingsStore.getState().settings;
    // plain spreads produce a new object; the previous reference is untouched.
    expect(after).not.toBe(before);
    expect(before.magnification.zoom_level).toBe(DEFAULT_SETTINGS.magnification.zoom_level);
  });
});
