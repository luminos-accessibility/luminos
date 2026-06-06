import { useId } from 'react';
import type { ChangeEvent, JSX } from 'react';

import { useSettingsStore } from '../../hooks/useSettingsStore';
import { useToast } from '../../hooks/useToast';
import { setZoomLevel } from '../../ipc/commands';
import { ZOOM_MAX, ZOOM_MIN, ZOOM_STEP } from '../../types/enums';

/** Formats a zoom level for display and `aria-valuetext` (e.g. `5.0x`). */
const formatZoom = (zoom: number): string => `${zoom.toFixed(1)}x`;

/**
 * Accessible zoom-level slider.
 *
 * Performs an optimistic store update on change, calls `setZoomLevel` over
 * IPC, and reverts to the prior value with an error toast if the engine
 * rejects (FR-5, AC-2.1). The native `<input type="range">` provides keyboard
 * operability for free; `aria-labelledby`, `aria-valuetext`, and a polite
 * `<output>` make the value legible to assistive tech (FR-7).
 */
export const ZoomLevelSlider = (): JSX.Element => {
  const labelId = useId();
  const zoom = useSettingsStore((state) => state.settings.magnification.zoom_level);
  const setStoreZoom = useSettingsStore((state) => state.setZoomLevel);
  const toast = useToast();

  const handleChange = async (event: ChangeEvent<HTMLInputElement>): Promise<void> => {
    const previousZoom = useSettingsStore.getState().settings.magnification.zoom_level;
    const nextZoom = Number.parseFloat(event.target.value);

    // Optimistic update first so the UI feels instant.
    setStoreZoom(nextZoom);
    try {
      await setZoomLevel(nextZoom);
    } catch (error) {
      // Revert to the value the engine still holds and notify the user.
      setStoreZoom(previousZoom);
      const message = error instanceof Error ? error.message : 'Failed to set zoom level';
      toast.error(message);
    }
  };

  return (
    <label className="control">
      <span id={labelId}>Zoom level</span>
      <input
        type="range"
        aria-labelledby={labelId}
        aria-valuetext={formatZoom(zoom)}
        min={ZOOM_MIN}
        max={ZOOM_MAX}
        step={ZOOM_STEP}
        value={zoom}
        onChange={handleChange}
      />
      <output aria-live="polite">{formatZoom(zoom)}</output>
    </label>
  );
};
