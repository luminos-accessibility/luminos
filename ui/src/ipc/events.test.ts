import { mockIPC } from '@tauri-apps/api/mocks';
import { emit } from '@tauri-apps/api/event';
import { beforeEach, describe, expect, test, vi } from 'vitest';

import { onModeChanged, onZoomChanged } from './events';

/**
 * IPC event-wrapper tests. We mock at the Tauri boundary with event mocking
 * enabled (`shouldMockEvents`, Tauri >= 2.7.0), emit the raw engine events,
 * and assert the wrappers forward the payload to the callback (FR-8).
 */
describe('ipc/events', () => {
  beforeEach(() => {
    mockIPC(() => undefined, { shouldMockEvents: true });
  });

  test('on_zoom_changed_forwards_payload_to_callback', async () => {
    const handler = vi.fn();
    await onZoomChanged(handler);
    await emit('zoom_changed', 9.5);
    expect(handler).toHaveBeenCalledWith(9.5);
  });

  test('on_mode_changed_forwards_payload_to_callback', async () => {
    const handler = vi.fn();
    await onModeChanged(handler);
    await emit('mode_changed', 'Lens');
    expect(handler).toHaveBeenCalledWith('Lens');
  });

  test('on_zoom_changed_returns_an_unlisten_function', async () => {
    const unlisten = await onZoomChanged(vi.fn());
    expect(typeof unlisten).toBe('function');
  });
});
