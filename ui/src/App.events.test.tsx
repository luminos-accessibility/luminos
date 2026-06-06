import { render, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, test, vi } from 'vitest';

import { DEFAULT_SETTINGS } from './constants/defaults';
import { useSettingsStore } from './hooks/useSettingsStore';
import type { MagnificationMode } from './types/enums';

vi.mock('./ipc/commands');
vi.mock('./ipc/events');

import { App } from './App';
import { getCurrentSettings, getFrameTimings } from './ipc/commands';
import { onModeChanged, onZoomChanged } from './ipc/events';

const mockedOnZoomChanged = vi.mocked(onZoomChanged);
const mockedOnModeChanged = vi.mocked(onModeChanged);

/**
 * Engine→panel event subscription tests (FR-8, AC-3.1). We capture the
 * callbacks the App registers and invoke them to simulate engine-originated
 * `zoom_changed`/`mode_changed` events, then assert the store reflects them.
 */
describe('App event subscriptions', () => {
  let zoomCallback: ((level: number) => void) | undefined;
  let modeCallback: ((mode: MagnificationMode) => void) | undefined;
  const zoomUnlisten = vi.fn();
  const modeUnlisten = vi.fn();

  beforeEach(() => {
    useSettingsStore.setState({ settings: DEFAULT_SETTINGS, isHydrating: false });
    vi.mocked(getCurrentSettings).mockResolvedValue(DEFAULT_SETTINGS);
    vi.mocked(getFrameTimings).mockResolvedValue({
      averageMs: 8,
      p99Ms: 14,
      minMs: 6,
      maxMs: 19,
      targetFps: 60,
    });
    zoomCallback = undefined;
    modeCallback = undefined;
    mockedOnZoomChanged.mockImplementation((cb) => {
      zoomCallback = cb;
      return Promise.resolve(zoomUnlisten);
    });
    mockedOnModeChanged.mockImplementation((cb) => {
      modeCallback = cb;
      return Promise.resolve(modeUnlisten);
    });
  });

  test('app_subscribes_to_zoom_and_mode_events_on_mount', async () => {
    render(<App />);
    await waitFor(() => {
      expect(mockedOnZoomChanged).toHaveBeenCalledTimes(1);
      expect(mockedOnModeChanged).toHaveBeenCalledTimes(1);
    });
  });

  test('zoom_changed_event_updates_store', async () => {
    render(<App />);
    await waitFor(() => expect(zoomCallback).toBeDefined());

    zoomCallback?.(11.0);
    await waitFor(() => {
      expect(useSettingsStore.getState().settings.magnification.zoom_level).toBe(11.0);
    });
  });

  test('mode_changed_event_updates_store', async () => {
    render(<App />);
    await waitFor(() => expect(modeCallback).toBeDefined());

    modeCallback?.('Docked');
    await waitFor(() => {
      expect(useSettingsStore.getState().settings.magnification.mode).toBe('Docked');
    });
  });

  test('app_unsubscribes_from_events_on_unmount', async () => {
    const { unmount } = render(<App />);
    await waitFor(() => {
      expect(mockedOnZoomChanged).toHaveBeenCalled();
      expect(mockedOnModeChanged).toHaveBeenCalled();
    });

    unmount();
    await waitFor(() => {
      expect(zoomUnlisten).toHaveBeenCalledTimes(1);
      expect(modeUnlisten).toHaveBeenCalledTimes(1);
    });
  });
});
