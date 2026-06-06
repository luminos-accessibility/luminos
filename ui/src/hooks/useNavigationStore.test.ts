import { describe, expect, test } from 'vitest';

import { useNavigationStore } from './useNavigationStore';

describe('useNavigationStore', () => {
  test('navigation_store_defaults_to_magnification', () => {
    expect(useNavigationStore.getState().activePage).toBe('magnification');
  });

  test('navigation_store_ignores_disabled_page', () => {
    // Display is disabled in Phase 0; navigating to it must be a no-op.
    useNavigationStore.getState().setActivePage('display');
    expect(useNavigationStore.getState().activePage).toBe('magnification');
  });

  test('navigation_store_allows_enabled_page', () => {
    // Magnification is the only enabled page; selecting it stays valid.
    useNavigationStore.getState().setActivePage('magnification');
    expect(useNavigationStore.getState().activePage).toBe('magnification');
  });
});
