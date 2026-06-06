import { create } from 'zustand';

import { DEFAULT_SETTINGS } from '../constants/defaults';
import { ZOOM_MAX, ZOOM_MIN } from '../types/enums';
import type { AppSettings } from '../types/settings';
import type { MagnificationMode } from '../types/enums';

/** Clamps a zoom level into the engine-enforced `[ZOOM_MIN, ZOOM_MAX]` range. */
const clampZoom = (level: number): number => Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, level));

/**
 * Settings store state and actions.
 *
 * The store is the single source of truth for the UI. Mutating actions
 * (`setZoomLevel`, `setMode`) perform LOCAL optimistic updates only — the IPC
 * call and revert-on-error live in the components, which revert by calling the
 * action again with the prior value (or `applyEngineUpdate`). `hydrate` seeds
 * it on startup; `applyEngineUpdate` replaces it from engine events.
 *
 * Updates use plain immutable spreads rather than immer middleware: the
 * mutated shape (`settings.magnification`) is shallow, and immer is not in the
 * E04 pinned-dependency set, so avoiding it keeps the dependency surface and
 * bundle smaller (NFR-5) with no loss of clarity. (Deviation from DESIGN.md,
 * which assumed immer; recorded in SUBTASKS Deviations.)
 */
export interface SettingsState {
  /** Current settings snapshot mirroring the engine's `AppSettings`. */
  readonly settings: AppSettings;
  /** True until the first `hydrate` resolves; gates the UI. */
  readonly isHydrating: boolean;
  /** Seeds the store from the engine and marks hydration complete. */
  hydrate(settings: AppSettings): void;
  /** Replaces the settings wholesale (used by engine→panel events). */
  applyEngineUpdate(settings: AppSettings): void;
  /** Optimistically sets the zoom level, clamped to `[1.5, 20]`. */
  setZoomLevel(level: number): void;
  /** Optimistically sets the magnification mode. */
  setMode(mode: MagnificationMode): void;
}

/**
 * Zustand store for application settings. Exported as a hook for components
 * and usable directly via `useSettingsStore.getState()` in non-React code
 * (e.g. tests, IPC event handlers).
 */
export const useSettingsStore = create<SettingsState>()((set) => ({
  settings: DEFAULT_SETTINGS,
  isHydrating: true,
  hydrate: (settings) => set({ settings, isHydrating: false }),
  applyEngineUpdate: (settings) => set({ settings }),
  setZoomLevel: (level) =>
    set((state) => ({
      settings: {
        ...state.settings,
        magnification: { ...state.settings.magnification, zoom_level: clampZoom(level) },
      },
    })),
  setMode: (mode) =>
    set((state) => ({
      settings: {
        ...state.settings,
        magnification: { ...state.settings.magnification, mode },
      },
    })),
}));
