import { create } from 'zustand';

/**
 * Identifiers for the control-panel pages. Phase 0 ships only `magnification`;
 * the rest are placeholders shown disabled in the sidebar and land in later
 * epics (E5+). A lightweight in-store selector avoids a router dependency for
 * a single active page (resolved open question in STORY.md).
 */
export const PAGE_IDS = ['magnification', 'display', 'speech', 'keybindings', 'diagnostics'] as const;
export type PageId = (typeof PAGE_IDS)[number];

/** Pages enabled in Phase 0. Others render as disabled sidebar entries. */
export const ENABLED_PAGES: ReadonlySet<PageId> = new Set<PageId>(['magnification']);

/** Human-readable labels for each page, in sidebar order. */
export const PAGE_LABELS: Readonly<Record<PageId, string>> = {
  magnification: 'Magnification',
  display: 'Display',
  speech: 'Speech',
  keybindings: 'Keybindings',
  diagnostics: 'Diagnostics',
};

interface NavigationState {
  /** The currently active page. */
  readonly activePage: PageId;
  /** Navigates to an enabled page (no-op for disabled pages). */
  setActivePage(page: PageId): void;
}

/** Navigation store holding the active control-panel page. */
export const useNavigationStore = create<NavigationState>()((set) => ({
  activePage: 'magnification',
  setActivePage: (page) => {
    if (ENABLED_PAGES.has(page)) {
      set({ activePage: page });
    }
  },
}));
