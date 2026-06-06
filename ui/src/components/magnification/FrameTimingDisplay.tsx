import { useEffect, useState } from 'react';
import type { JSX } from 'react';

import { getFrameTimings } from '../../ipc/commands';
import type { FrameTimingSummary } from '../../types/settings';

/** Poll interval (ms) for refreshing the frame-timing readout. */
const POLL_INTERVAL_MS = 1000;

/**
 * Debug-only frame-timing readout.
 *
 * Renders nothing outside dev builds (`import.meta.env.DEV`), mirroring the
 * Rust `#[cfg(debug_assertions)]` boundary (AC-2.2). In dev, it polls
 * `getFrameTimings` and shows the P99 (and average) frame time so a developer
 * can watch the 60fps budget. There is no engine event for timings, so a
 * lightweight interval poll is used (resolved open question).
 */
export const FrameTimingDisplay = (): JSX.Element | null => {
  // Read once so render and effect agree on the same value within a test run.
  const isDev = import.meta.env.DEV;
  const [summary, setSummary] = useState<FrameTimingSummary | null>(null);

  useEffect(() => {
    if (!isDev) {
      return;
    }

    let isActive = true;
    const refresh = async (): Promise<void> => {
      try {
        const next = await getFrameTimings();
        if (isActive) {
          setSummary(next);
        }
      } catch {
        // Timings are diagnostic only; swallow errors to avoid noisy toasts.
      }
    };

    void refresh();
    const handle = window.setInterval(() => void refresh(), POLL_INTERVAL_MS);
    return () => {
      isActive = false;
      window.clearInterval(handle);
    };
  }, [isDev]);

  // Production builds: render nothing (matches Rust `#[cfg(debug_assertions)]`).
  if (!isDev) {
    return null;
  }

  return (
    <section aria-label="Frame timing diagnostics" className="control">
      <h2>Frame timing</h2>
      {summary === null ? (
        <p role="status">Collecting frame timings…</p>
      ) : (
        <dl>
          <dt>P99</dt>
          <dd>{summary.p99Ms.toFixed(1)} ms</dd>
          <dt>Average</dt>
          <dd>{summary.averageMs.toFixed(1)} ms</dd>
          <dt>Target</dt>
          <dd>{summary.targetFps} fps</dd>
        </dl>
      )}
    </section>
  );
};
