import { appSettingsSchema, frameTimingSummarySchema } from '../types/settings';
import type { AppSettings, FrameTimingSummary } from '../types/settings';
import type { MagnificationMode } from '../types/enums';
import { commands } from './bindings';

/**
 * tauri-specta's discriminated `Result` envelope (default error-handling mode).
 *
 * The generated `bindings.ts` inlines this envelope as each command's return
 * type rather than exporting a named `Result`, so it is declared here to keep
 * the unwrap helper typed. It mirrors the runtime shape tauri-specta produces:
 * `{ status: "ok"; data: T } | { status: "error"; error: E }`.
 */
export type Result<T, E> =
  | { readonly status: 'ok'; readonly data: T }
  | { readonly status: 'error'; readonly error: E };

/**
 * Typed IPC command wrappers.
 *
 * This module is the seam between the React app and the engine. Components and
 * the Zustand store depend ONLY on these wrappers — never on `./bindings`
 * directly — so swapping the placeholder bindings for story 005's generated
 * file is a one-file change. Each wrapper:
 *
 *   1. unwraps tauri-specta's discriminated `Result` envelope, rejecting the
 *      Promise on `status: "error"` so callers use plain `try/catch`; and
 *   2. validates inbound payloads with the Zod schema at the boundary
 *      (defense against engine/binding drift; NFR-2).
 */

/** Error thrown when a command returns a `status: "error"` envelope. */
export class IpcCommandError extends Error {
  constructor(
    public readonly command: string,
    message: string
  ) {
    super(message);
    this.name = 'IpcCommandError';
  }
}

/** Unwraps a tauri-specta `Result`, throwing `IpcCommandError` on failure. */
const unwrap = <T>(command: string, result: Result<T, string>): T => {
  if (result.status === 'error') {
    throw new IpcCommandError(command, result.error);
  }
  return result.data;
};

/**
 * Fetches the engine's current settings.
 * @returns The validated current {@link AppSettings}.
 * @throws {IpcCommandError} If the engine reports an error.
 */
export const getCurrentSettings = async (): Promise<AppSettings> => {
  const raw = unwrap('get_current_settings', await commands.getCurrentSettings());
  return appSettingsSchema.parse(raw);
};

/**
 * Sets the engine zoom level. The engine clamps to `[1.5, 20]`.
 * @param level - Desired zoom multiplier.
 * @throws {IpcCommandError} If the engine reports an error.
 */
export const setZoomLevel = async (level: number): Promise<void> => {
  unwrap('set_zoom_level', await commands.setZoomLevel(level));
};

/**
 * Sets the active magnification mode.
 * @param mode - The {@link MagnificationMode} to activate.
 * @throws {IpcCommandError} If the engine reports an error.
 */
export const setMagnificationMode = async (mode: MagnificationMode): Promise<void> => {
  unwrap('set_magnification_mode', await commands.setMagnificationMode(mode));
};

/**
 * Toggles magnification on/off.
 * @returns The resulting enabled state.
 * @throws {IpcCommandError} If the engine reports an error.
 */
export const toggleMagnification = async (): Promise<boolean> => {
  return unwrap('toggle_magnification', await commands.toggleMagnification());
};

/**
 * Fetches the latest frame-timing summary (P99, average, ...).
 * @returns The validated {@link FrameTimingSummary}.
 * @throws {IpcCommandError} If the engine reports an error.
 */
export const getFrameTimings = async (): Promise<FrameTimingSummary> => {
  const raw = unwrap('get_frame_timings', await commands.getFrameTimings());
  return frameTimingSummarySchema.parse(raw);
};

/**
 * Persists the current settings to `config.toml`.
 * @throws {IpcCommandError} If the engine reports an error.
 */
export const saveSettings = async (): Promise<void> => {
  unwrap('save_settings', await commands.saveSettings());
};

/**
 * Resets settings to defaults and persists them.
 * @returns The validated default {@link AppSettings}.
 * @throws {IpcCommandError} If the engine reports an error.
 */
export const resetSettings = async (): Promise<AppSettings> => {
  const raw = unwrap('reset_settings', await commands.resetSettings());
  return appSettingsSchema.parse(raw);
};
