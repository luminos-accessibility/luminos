import type { JSX } from 'react';

import {
  ENABLED_PAGES,
  PAGE_IDS,
  PAGE_LABELS,
  useNavigationStore,
} from '../hooks/useNavigationStore';

/**
 * Sidebar navigation. Rendered inside the `Shell`'s `<nav>` landmark. The
 * active page is marked with `aria-current="page"`; Phase 0 disables all
 * pages except Magnification (they arrive in later epics). Every entry is a
 * real `<button>`, so the list is keyboard-navigable and screen-reader labeled
 * (FR-7, AC-3.1).
 */
export const Sidebar = (): JSX.Element => {
  const activePage = useNavigationStore((state) => state.activePage);
  const setActivePage = useNavigationStore((state) => state.setActivePage);

  return (
    <nav className="app-sidebar" aria-label="Settings sections">
      <ul>
        {PAGE_IDS.map((pageId) => {
          const isEnabled = ENABLED_PAGES.has(pageId);
          const isActive = pageId === activePage;
          return (
            <li key={pageId}>
              <button
                type="button"
                disabled={!isEnabled}
                aria-current={isActive ? 'page' : undefined}
                onClick={() => setActivePage(pageId)}
              >
                {PAGE_LABELS[pageId]}
              </button>
            </li>
          );
        })}
      </ul>
    </nav>
  );
};
