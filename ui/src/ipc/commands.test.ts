import { mockIPC } from '@tauri-apps/api/mocks';
import { beforeEach, describe, expect, test, vi } from 'vitest';

import { DEFAULT_SETTINGS } from '../constants/defaults';
import {
  getCurrentSettings,
  getFrameTimings,
  IpcCommandError,
  resetSettings,
  setMagnificationMode,
  setZoomLevel,
  toggleMagnification,
} from './commands';

/**
 * IPC seam tests. These mock at the lowest boundary (Tauri `invoke`) so the
 * real placeholder bindings + wrapper logic (command-name mapping, arg
 * shaping, Result-unwrapping, Zod parsing) all execute. When story 005 swaps
 * in generated bindings, these continue to assert the same contract.
 */
describe('ipc/commands', () => {
  beforeEach(() => {
    mockIPC((cmd) => {
      switch (cmd) {
        case 'get_current_settings':
          return DEFAULT_SETTINGS;
        case 'reset_settings':
          return DEFAULT_SETTINGS;
        case 'set_zoom_level':
          return null;
        case 'set_magnification_mode':
          return null;
        case 'toggle_magnification':
          return false;
        case 'get_frame_timings':
          return { averageMs: 8.1, p99Ms: 14.2, minMs: 6.0, maxMs: 19.9, targetFps: 60 };
        default:
          throw new Error(`Unmocked command: ${cmd}`);
      }
    });
  });

  test('get_current_settings_returns_parsed_app_settings', async () => {
    const settings = await getCurrentSettings();
    expect(settings.magnification.zoom_level).toBe(2.0);
    expect(settings.magnification.mode).toBe('FullScreen');
  });

  test('set_zoom_level_invokes_snake_case_command_with_level_arg', async () => {
    const spy = vi.spyOn(window.__TAURI_INTERNALS__, 'invoke');
    await setZoomLevel(7.5);
    expect(spy).toHaveBeenCalledWith('set_zoom_level', { level: 7.5 }, undefined);
  });

  test('set_magnification_mode_invokes_command_with_mode_arg', async () => {
    const spy = vi.spyOn(window.__TAURI_INTERNALS__, 'invoke');
    await setMagnificationMode('Lens');
    expect(spy).toHaveBeenCalledWith('set_magnification_mode', { mode: 'Lens' }, undefined);
  });

  test('toggle_magnification_returns_boolean', async () => {
    await expect(toggleMagnification()).resolves.toBe(false);
  });

  test('get_frame_timings_returns_parsed_camel_case_summary', async () => {
    const summary = await getFrameTimings();
    expect(summary.p99Ms).toBe(14.2);
    expect(summary.targetFps).toBe(60);
  });

  test('reset_settings_returns_parsed_defaults', async () => {
    const settings = await resetSettings();
    expect(settings.minimize_to_tray).toBe(true);
  });

  test('command_rejects_with_ipc_command_error_on_engine_error', async () => {
    // A rejected invoke (non-Error reason) surfaces as IpcCommandError.
    mockIPC((cmd) => {
      if (cmd === 'set_zoom_level') {
        return Promise.reject('engine busy');
      }
      return null;
    });
    await expect(setZoomLevel(5)).rejects.toBeInstanceOf(IpcCommandError);
  });

  test('command_rejects_when_payload_fails_schema_validation', async () => {
    mockIPC((cmd) => {
      if (cmd === 'get_current_settings') {
        // Missing required fields → Zod parse must throw at the boundary.
        return { magnification: { zoom_level: 5 } };
      }
      return null;
    });
    await expect(getCurrentSettings()).rejects.toBeTruthy();
  });
});
