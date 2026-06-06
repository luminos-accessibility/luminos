import type { UnlistenFn } from '@tauri-apps/api/event';

import type { MagnificationMode } from '../types/enums';
import { events } from './bindings';

/**
 * Typed IPC event wrappers (engine → panel).
 *
 * These subscribe to the `tauri-specta`-generated events emitted when the
 * engine changes state out-of-band (e.g. a global hotkey adjusts zoom while
 * the user is not touching the panel). Each returns the `UnlistenFn` so the
 * subscriber can clean up on unmount (FR-8). Like `./commands`, this module
 * is the seam: callers never touch `./bindings` directly.
 */

/**
 * Subscribes to engine-originated zoom changes.
 * @param callback - Invoked with the new zoom level.
 * @returns A promise resolving to the unsubscribe function.
 */
export const onZoomChanged = (callback: (level: number) => void): Promise<UnlistenFn> =>
  events.zoomChanged.listen((event) => {
    callback(event.payload);
  });

/**
 * Subscribes to engine-originated magnification-mode changes.
 * @param callback - Invoked with the new {@link MagnificationMode}.
 * @returns A promise resolving to the unsubscribe function.
 */
export const onModeChanged = (
  callback: (mode: MagnificationMode) => void
): Promise<UnlistenFn> =>
  events.modeChanged.listen((event) => {
    callback(event.payload);
  });
