import { render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

vi.mock('../../ipc/commands');

import { FrameTimingDisplay } from './FrameTimingDisplay';
import { getFrameTimings } from '../../ipc/commands';

const mockedGetFrameTimings = vi.mocked(getFrameTimings);

describe('FrameTimingDisplay', () => {
  beforeEach(() => {
    mockedGetFrameTimings.mockResolvedValue({
      averageMs: 8.1,
      p99Ms: 14.2,
      minMs: 6.0,
      maxMs: 19.9,
      targetFps: 60,
    });
  });

  afterEach(() => {
    vi.unstubAllEnvs();
  });

  test('frame_timing_display_shows_p99_when_dev', async () => {
    vi.stubEnv('DEV', true);
    render(<FrameTimingDisplay />);
    await waitFor(() => {
      expect(screen.getByText(/P99/i)).toBeInTheDocument();
    });
    expect(screen.getByText(/14\.2/)).toBeInTheDocument();
  });

  test('frame_timing_display_absent_when_not_dev', () => {
    vi.stubEnv('DEV', false);
    const { container } = render(<FrameTimingDisplay />);
    expect(container).toBeEmptyDOMElement();
    expect(mockedGetFrameTimings).not.toHaveBeenCalled();
  });
});
