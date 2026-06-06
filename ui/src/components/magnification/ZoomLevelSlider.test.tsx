import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, test, vi } from 'vitest';

import { DEFAULT_SETTINGS } from '../../constants/defaults';
import { useSettingsStore } from '../../hooks/useSettingsStore';
import { ToastProvider } from '../../hooks/useToast';

vi.mock('../../ipc/commands');

import { ZoomLevelSlider } from './ZoomLevelSlider';
import { setZoomLevel } from '../../ipc/commands';

const mockedSetZoomLevel = vi.mocked(setZoomLevel);

const renderSlider = () =>
  render(
    <ToastProvider>
      <ZoomLevelSlider />
    </ToastProvider>
  );

describe('ZoomLevelSlider', () => {
  beforeEach(() => {
    useSettingsStore.setState({
      settings: {
        ...DEFAULT_SETTINGS,
        magnification: { ...DEFAULT_SETTINGS.magnification, zoom_level: 5.0 },
      },
      isHydrating: false,
    });
    mockedSetZoomLevel.mockResolvedValue(undefined);
  });

  test('zoom_slider_renders_current_zoom_level', () => {
    renderSlider();
    const slider = screen.getByRole('slider', { name: /zoom level/i });
    expect(slider).toHaveValue('5');
  });

  test('zoom_slider_exposes_accessible_attributes', () => {
    renderSlider();
    const slider = screen.getByRole('slider', { name: /zoom level/i });
    expect(slider).toHaveAttribute('aria-valuetext', '5.0x');
    expect(slider).toHaveAttribute('min', '1.5');
    expect(slider).toHaveAttribute('max', '20');
    expect(slider).toHaveAttribute('step', '0.5');
    // The native range input is focusable / keyboard-operable by construction.
    slider.focus();
    expect(slider).toHaveFocus();
    // The live output reflects the current value politely.
    const output = screen.getByText('5.0x', { selector: 'output' });
    expect(output).toHaveAttribute('aria-live', 'polite');
  });

  // jsdom does not implement value changes from arrow keys on `<input
  // type="range">`, so we drive the value change directly with `fireEvent`
  // (the documented RTL exception for range inputs) to exercise `onChange`.
  test('zoom_slider_change_invokes_set_zoom_level_with_value', async () => {
    renderSlider();
    const slider = screen.getByRole('slider', { name: /zoom level/i });
    fireEvent.change(slider, { target: { value: '5.5' } });

    await waitFor(() => {
      expect(mockedSetZoomLevel).toHaveBeenCalledWith(5.5);
    });
    expect(useSettingsStore.getState().settings.magnification.zoom_level).toBe(5.5);
  });

  test('zoom_slider_reverts_and_toasts_on_ipc_error', async () => {
    mockedSetZoomLevel.mockRejectedValue(new Error('engine busy'));
    renderSlider();
    const slider = screen.getByRole('slider', { name: /zoom level/i });
    fireEvent.change(slider, { target: { value: '5.5' } });

    // The optimistic update is rolled back to the prior value ...
    await waitFor(() => {
      expect(useSettingsStore.getState().settings.magnification.zoom_level).toBe(5.0);
    });
    expect(slider).toHaveValue('5');
    // ... and an accessible alert is shown.
    expect(await screen.findByRole('alert')).toHaveTextContent(/engine busy/i);
  });
});
