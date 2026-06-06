import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, test, vi } from 'vitest';

import { DEFAULT_SETTINGS } from '../../constants/defaults';
import { useSettingsStore } from '../../hooks/useSettingsStore';
import { ToastProvider } from '../../hooks/useToast';

vi.mock('../../ipc/commands');

import { MagnificationModeSelector } from './MagnificationModeSelector';
import { setMagnificationMode } from '../../ipc/commands';

const mockedSetMode = vi.mocked(setMagnificationMode);

const renderSelector = () =>
  render(
    <ToastProvider>
      <MagnificationModeSelector />
    </ToastProvider>
  );

describe('MagnificationModeSelector', () => {
  beforeEach(() => {
    useSettingsStore.setState({ settings: DEFAULT_SETTINGS, isHydrating: false });
    mockedSetMode.mockResolvedValue(undefined);
  });

  test('mode_selector_renders_all_modes_as_radio_group', () => {
    renderSelector();
    expect(screen.getByRole('radiogroup', { name: /magnification mode/i })).toBeInTheDocument();
    expect(screen.getByRole('radio', { name: /full screen/i })).toBeChecked();
    expect(screen.getByRole('radio', { name: /lens/i })).toBeInTheDocument();
    expect(screen.getByRole('radio', { name: /docked/i })).toBeInTheDocument();
  });

  test('mode_selector_select_lens_invokes_set_magnification_mode', async () => {
    const user = userEvent.setup();
    renderSelector();
    await user.click(screen.getByRole('radio', { name: /lens/i }));

    await waitFor(() => {
      expect(mockedSetMode).toHaveBeenCalledWith('Lens');
    });
    expect(useSettingsStore.getState().settings.magnification.mode).toBe('Lens');
  });

  test('mode_selector_reverts_and_toasts_on_ipc_error', async () => {
    mockedSetMode.mockRejectedValue(new Error('mode unavailable'));
    const user = userEvent.setup();
    renderSelector();
    await user.click(screen.getByRole('radio', { name: /docked/i }));

    await waitFor(() => {
      expect(useSettingsStore.getState().settings.magnification.mode).toBe('FullScreen');
    });
    expect(await screen.findByRole('alert')).toHaveTextContent(/mode unavailable/i);
  });
});
