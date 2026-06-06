import { useId } from 'react';
import type { JSX } from 'react';

import { useSettingsStore } from '../../hooks/useSettingsStore';
import { useToast } from '../../hooks/useToast';
import { setMagnificationMode } from '../../ipc/commands';
import type { MagnificationMode } from '../../types/enums';

/** Selectable modes with human-readable labels, in display order. */
const MODE_OPTIONS: ReadonlyArray<{ readonly value: MagnificationMode; readonly label: string }> = [
  { value: 'FullScreen', label: 'Full screen' },
  { value: 'Lens', label: 'Lens' },
  { value: 'Docked', label: 'Docked' },
];

/**
 * Accessible magnification-mode selector, implemented as a native radio group.
 *
 * Selecting a mode optimistically updates the store, calls
 * `setMagnificationMode` over IPC, and reverts with an error toast on failure
 * (FR-5, AC-2.2). In Phase 0 only Full screen is engine-active; Lens and
 * Docked are selectable placeholders that Epic 5 will fully implement. Native
 * radios give arrow-key navigation and labeling for free (FR-7).
 */
export const MagnificationModeSelector = (): JSX.Element => {
  const groupLabelId = useId();
  const mode = useSettingsStore((state) => state.settings.magnification.mode);
  const setStoreMode = useSettingsStore((state) => state.setMode);
  const toast = useToast();

  const handleSelect = async (next: MagnificationMode): Promise<void> => {
    const previousMode = useSettingsStore.getState().settings.magnification.mode;
    setStoreMode(next);
    try {
      await setMagnificationMode(next);
    } catch (error) {
      setStoreMode(previousMode);
      const message = error instanceof Error ? error.message : 'Failed to change magnification mode';
      toast.error(message);
    }
  };

  return (
    <fieldset className="control" role="radiogroup" aria-labelledby={groupLabelId}>
      <legend id={groupLabelId}>Magnification mode</legend>
      {MODE_OPTIONS.map((option) => (
        <label key={option.value}>
          <input
            type="radio"
            name="magnification-mode"
            value={option.value}
            checked={mode === option.value}
            onChange={() => handleSelect(option.value)}
          />
          {option.label}
        </label>
      ))}
    </fieldset>
  );
};
