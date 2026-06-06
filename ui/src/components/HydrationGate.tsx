import type { JSX, ReactNode } from 'react';

import { useSettingsStore } from '../hooks/useSettingsStore';

/**
 * Gates the panel UI on store hydration. While `isHydrating` is true it shows
 * an accessible loading status (`role="status"`, polite live region); once
 * hydration completes (success OR defaults-on-error), it renders `children`.
 * This guarantees the user never sees a blank screen (AC-1.2).
 *
 * @param props.children - The hydrated panel subtree.
 */
export const HydrationGate = ({ children }: { children: ReactNode }): JSX.Element => {
  const isHydrating = useSettingsStore((state) => state.isHydrating);

  if (isHydrating) {
    return (
      <div className="hydration-gate">
        <p role="status">Loading settings…</p>
      </div>
    );
  }

  return <>{children}</>;
};
