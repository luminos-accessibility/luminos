import { render, screen, waitFor } from '@testing-library/react';
import { axe } from 'jest-axe';
import { beforeEach, describe, expect, test, vi } from 'vitest';

import { DEFAULT_SETTINGS } from './constants/defaults';
import { useSettingsStore } from './hooks/useSettingsStore';
import { ToastProvider, useToast } from './hooks/useToast';
import type { JSX, ReactNode } from 'react';

vi.mock('./ipc/commands');
vi.mock('./ipc/events');

import { App } from './App';
import { Shell } from './components/Shell';
import { MagnificationModeSelector } from './components/magnification/MagnificationModeSelector';
import { ZoomLevelSlider } from './components/magnification/ZoomLevelSlider';
import { MagnificationPage } from './pages/MagnificationPage';
import { getCurrentSettings, getFrameTimings } from './ipc/commands';
import { onModeChanged, onZoomChanged } from './ipc/events';

const withToasts = (children: ReactNode): JSX.Element => <ToastProvider>{children}</ToastProvider>;

/** Renders a child that raises an error toast immediately, for axe to scan. */
const ToastRaiser = (): JSX.Element => {
  const toast = useToast();
  // Raise once on first render so the alert is present when axe runs.
  if (screen.queryByRole('alert') === null) {
    queueMicrotask(() => toast.error('Something went wrong'));
  }
  return <p>content</p>;
};

/**
 * Accessibility sweep (AC-3.1 / D8). Every Phase-0 component must pass
 * axe-core with ZERO violations. Luminos is an accessibility product; an
 * inaccessible control panel is a critical bug.
 */
describe('accessibility (axe-core)', () => {
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
    vi.mocked(onZoomChanged).mockResolvedValue(() => {});
    vi.mocked(onModeChanged).mockResolvedValue(() => {});
  });

  test('zoom_level_slider_has_no_axe_violations', async () => {
    const { container } = render(withToasts(<ZoomLevelSlider />));
    expect(await axe(container)).toHaveNoViolations();
  });

  test('magnification_mode_selector_has_no_axe_violations', async () => {
    const { container } = render(withToasts(<MagnificationModeSelector />));
    expect(await axe(container)).toHaveNoViolations();
  });

  test('magnification_page_has_no_axe_violations', async () => {
    const { container } = render(withToasts(<MagnificationPage />));
    expect(await axe(container)).toHaveNoViolations();
  });

  test('shell_has_no_axe_violations', async () => {
    const { container } = render(withToasts(<Shell />));
    expect(await axe(container)).toHaveNoViolations();
  });

  test('app_has_no_axe_violations_after_hydration', async () => {
    const { container } = render(<App />);
    await screen.findByRole('heading', { name: /magnification/i });
    expect(await axe(container)).toHaveNoViolations();
  });

  test('error_toast_has_no_axe_violations', async () => {
    const { container } = render(withToasts(<ToastRaiser />));
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(await axe(container)).toHaveNoViolations();
  });
});
