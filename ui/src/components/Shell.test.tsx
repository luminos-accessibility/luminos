import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, test, vi } from 'vitest';

import { DEFAULT_SETTINGS } from '../constants/defaults';
import { useNavigationStore } from '../hooks/useNavigationStore';
import { useSettingsStore } from '../hooks/useSettingsStore';
import { ToastProvider } from '../hooks/useToast';

vi.mock('../ipc/commands');

import { Shell } from './Shell';
import { getFrameTimings } from '../ipc/commands';

const renderShell = () =>
  render(
    <ToastProvider>
      <Shell />
    </ToastProvider>
  );

describe('Shell + Sidebar landmarks', () => {
  beforeEach(() => {
    useSettingsStore.setState({ settings: DEFAULT_SETTINGS, isHydrating: false });
    useNavigationStore.setState({ activePage: 'magnification' });
    vi.mocked(getFrameTimings).mockResolvedValue({
      averageMs: 8,
      p99Ms: 14,
      minMs: 6,
      maxMs: 19,
      targetFps: 60,
    });
  });

  test('shell_exposes_navigation_and_main_landmarks', () => {
    renderShell();
    expect(screen.getByRole('navigation', { name: /settings sections/i })).toBeInTheDocument();
    expect(screen.getByRole('main')).toBeInTheDocument();
  });

  test('sidebar_marks_magnification_as_current_page', () => {
    renderShell();
    const magnification = screen.getByRole('button', { name: 'Magnification' });
    expect(magnification).toHaveAttribute('aria-current', 'page');
  });

  test('sidebar_disables_not_yet_implemented_pages', () => {
    renderShell();
    expect(screen.getByRole('button', { name: 'Display' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Speech' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Keybindings' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Diagnostics' })).toBeDisabled();
  });

  test('shell_renders_magnification_page_outlet', () => {
    renderShell();
    expect(screen.getByRole('heading', { name: /magnification/i })).toBeInTheDocument();
  });
});
