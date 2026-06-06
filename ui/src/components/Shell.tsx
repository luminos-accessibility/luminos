import type { JSX } from 'react';

import { useNavigationStore } from '../hooks/useNavigationStore';
import { MagnificationPage } from '../pages/MagnificationPage';
import type { PageId } from '../hooks/useNavigationStore';
import { Sidebar } from './Sidebar';

/** Maps each enabled page to its outlet component. */
const renderPage = (activePage: PageId): JSX.Element => {
  switch (activePage) {
    case 'magnification':
      return <MagnificationPage />;
    // Other pages are disabled in the sidebar in Phase 0 and cannot become
    // active; they arrive in later epics. Fall back to the magnification page.
    default:
      return <MagnificationPage />;
  }
};

/**
 * Application shell: a two-pane layout with the `Sidebar` `<nav>` landmark and
 * a `<main>` content landmark holding the active page's outlet. Phase 0 routes
 * only the Magnification page; `renderPage` extends to new pages without
 * changing the landmark structure (FR-7).
 */
export const Shell = (): JSX.Element => {
  const activePage = useNavigationStore((state) => state.activePage);

  return (
    <div className="app-shell">
      <Sidebar />
      <main className="app-main">{renderPage(activePage)}</main>
    </div>
  );
};
