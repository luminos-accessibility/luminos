import { useEffect } from 'react';
import type { JSX } from 'react';

import { HydrationGate } from './components/HydrationGate';
import { Shell } from './components/Shell';
import { DEFAULT_SETTINGS } from './constants/defaults';
import { useSettingsStore } from './hooks/useSettingsStore';
import { ToastProvider, useToast } from './hooks/useToast';
import { getCurrentSettings } from './ipc/commands';
import { onModeChanged, onZoomChanged } from './ipc/events';

/**
 * Hydrates the settings store and wires engine→panel event subscriptions.
 * Rendered inside `ToastProvider` so hydration failures can raise a toast.
 *
 * - On mount, calls `getCurrentSettings`; on failure, hydrates with
 *   `DEFAULT_SETTINGS` and raises an error toast (never blank — AC-1.2).
 * - Subscribes to `zoom_changed`/`mode_changed` so hotkey-originated engine
 *   changes keep the store in sync, and unsubscribes on unmount (FR-8).
 */
const PanelRoot = (): JSX.Element => {
  const hydrate = useSettingsStore((state) => state.hydrate);
  const setZoomLevel = useSettingsStore((state) => state.setZoomLevel);
  const setMode = useSettingsStore((state) => state.setMode);
  const toast = useToast();

  useEffect(() => {
    let isActive = true;
    const runHydration = async (): Promise<void> => {
      try {
        const settings = await getCurrentSettings();
        if (isActive) {
          hydrate(settings);
        }
      } catch (error) {
        if (isActive) {
          hydrate(DEFAULT_SETTINGS);
          const message =
            error instanceof Error ? error.message : 'Failed to load settings; using defaults';
          toast.error(message);
        }
      }
    };
    void runHydration();
    return () => {
      isActive = false;
    };
  }, [hydrate, toast]);

  useEffect(() => {
    // Subscriptions return their unlisten fns asynchronously; collect and run
    // them all on cleanup, even if the effect unmounts before they resolve.
    const unlistenPromises = [
      onZoomChanged((level) => setZoomLevel(level)),
      onModeChanged((mode) => setMode(mode)),
    ];
    return () => {
      for (const unlistenPromise of unlistenPromises) {
        void unlistenPromise.then((unlisten) => unlisten());
      }
    };
  }, [setZoomLevel, setMode]);

  return (
    <HydrationGate>
      <Shell />
    </HydrationGate>
  );
};

/**
 * Application root. Establishes the toast live region, then mounts the
 * hydrating panel.
 */
export const App = (): JSX.Element => {
  return (
    <ToastProvider>
      <PanelRoot />
    </ToastProvider>
  );
};
