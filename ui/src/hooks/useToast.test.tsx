import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, test } from 'vitest';
import type { JSX } from 'react';

import { ToastProvider, useToast } from './useToast';

/** Test harness exposing buttons that raise/clear toasts via the hook. */
const ToastHarness = (): JSX.Element => {
  const toast = useToast();
  return (
    <div>
      <button type="button" onClick={() => toast.error('boom')}>
        raise error
      </button>
      <button type="button" onClick={() => toast.info('heads up')}>
        raise info
      </button>
    </div>
  );
};

describe('useToast', () => {
  test('use_toast_error_renders_assertive_alert', async () => {
    const user = userEvent.setup();
    render(
      <ToastProvider>
        <ToastHarness />
      </ToastProvider>
    );
    await user.click(screen.getByRole('button', { name: /raise error/i }));
    expect(screen.getByRole('alert')).toHaveTextContent('boom');
  });

  test('use_toast_info_renders_polite_status', async () => {
    const user = userEvent.setup();
    render(
      <ToastProvider>
        <ToastHarness />
      </ToastProvider>
    );
    await user.click(screen.getByRole('button', { name: /raise info/i }));
    expect(screen.getByRole('status')).toHaveTextContent('heads up');
  });

  test('use_toast_dismiss_removes_the_toast', async () => {
    const user = userEvent.setup();
    render(
      <ToastProvider>
        <ToastHarness />
      </ToastProvider>
    );
    await user.click(screen.getByRole('button', { name: /raise error/i }));
    expect(screen.getByRole('alert')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: /dismiss notification/i }));
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  test('use_toast_throws_outside_provider', () => {
    // Rendering the consumer without a provider must throw a clear error.
    expect(() => render(<ToastHarness />)).toThrow(/ToastProvider/);
  });
});
