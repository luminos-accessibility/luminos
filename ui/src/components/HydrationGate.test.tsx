import { render, screen } from '@testing-library/react';
import { describe, expect, test } from 'vitest';

import { DEFAULT_SETTINGS } from '../constants/defaults';
import { useSettingsStore } from '../hooks/useSettingsStore';
import { HydrationGate } from './HydrationGate';

describe('HydrationGate', () => {
  test('hydration_gate_shows_loading_status_while_hydrating', () => {
    useSettingsStore.setState({ settings: DEFAULT_SETTINGS, isHydrating: true });
    render(
      <HydrationGate>
        <p>panel content</p>
      </HydrationGate>
    );
    expect(screen.getByRole('status')).toHaveTextContent(/loading/i);
    expect(screen.queryByText('panel content')).not.toBeInTheDocument();
  });

  test('hydration_gate_renders_children_after_hydration', () => {
    useSettingsStore.setState({ settings: DEFAULT_SETTINGS, isHydrating: false });
    render(
      <HydrationGate>
        <p>panel content</p>
      </HydrationGate>
    );
    expect(screen.getByText('panel content')).toBeInTheDocument();
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
});
