import { render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, test, vi } from 'vitest';

import { DEFAULT_SETTINGS } from './constants/defaults';
import { useSettingsStore } from './hooks/useSettingsStore';

// Mock the IPC seam directly (no Tauri runtime). The App depends on the
// wrapper modules, never on bindings, so this is the seam the brief mandates.
vi.mock('./ipc/commands');
vi.mock('./ipc/events');

import { App } from './App';
import { getCurrentSettings, getFrameTimings } from './ipc/commands';
import { onModeChanged, onZoomChanged } from './ipc/events';

const mockedGetCurrentSettings = vi.mocked(getCurrentSettings);
const mockedGetFrameTimings = vi.mocked(getFrameTimings);
const mockedOnZoomChanged = vi.mocked(onZoomChanged);
const mockedOnModeChanged = vi.mocked(onModeChanged);

describe('App hydration', () => {
  beforeEach(() => {
    useSettingsStore.setState({ settings: DEFAULT_SETTINGS, isHydrating: true });
    mockedOnZoomChanged.mockResolvedValue(() => {});
    mockedOnModeChanged.mockResolvedValue(() => {});
    // The debug FrameTimingDisplay polls on mount; give it a valid summary.
    mockedGetFrameTimings.mockResolvedValue({
      averageMs: 8.1,
      p99Ms: 14.2,
      minMs: 6.0,
      maxMs: 19.9,
      targetFps: 60,
    });
  });

  test('app_hydration_success_populates_store', async () => {
    const engineSettings = {
      ...DEFAULT_SETTINGS,
      magnification: { ...DEFAULT_SETTINGS.magnification, zoom_level: 8.0 },
    };
    mockedGetCurrentSettings.mockResolvedValue(engineSettings);

    render(<App />);

    await waitFor(() => {
      expect(useSettingsStore.getState().isHydrating).toBe(false);
    });
    expect(useSettingsStore.getState().settings.magnification.zoom_level).toBe(8.0);
  });

  test('app_hydration_failure_uses_defaults_and_shows_toast', async () => {
    mockedGetCurrentSettings.mockRejectedValue(new Error('engine offline'));

    render(<App />);

    // Falls back to defaults (no blank screen) ...
    await waitFor(() => {
      expect(useSettingsStore.getState().isHydrating).toBe(false);
    });
    expect(useSettingsStore.getState().settings.magnification.zoom_level).toBe(
      DEFAULT_SETTINGS.magnification.zoom_level
    );
    // ... and surfaces an accessible error toast.
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent(/engine offline|failed|error/i);
  });

  test('app_renders_magnification_heading_after_hydration', async () => {
    mockedGetCurrentSettings.mockResolvedValue(DEFAULT_SETTINGS);
    render(<App />);
    expect(await screen.findByRole('heading', { name: /magnification/i })).toBeInTheDocument();
  });
});
