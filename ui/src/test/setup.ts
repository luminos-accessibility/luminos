import '@testing-library/jest-dom/vitest';

import { clearMocks } from '@tauri-apps/api/mocks';
import { toHaveNoViolations } from 'jest-axe';
import { afterEach, expect, vi } from 'vitest';

// Replace Zustand's `create`/`createStore` with the auto-reset variants so
// every store returns to its initial state between tests (official recipe).
vi.mock('zustand');

/**
 * Shared Vitest setup for the control panel test suite.
 *
 * - Registers `@testing-library/jest-dom` matchers (`.toBeInTheDocument()`).
 * - Registers the `jest-axe` `.toHaveNoViolations()` matcher.
 * - Clears Tauri IPC mocks after every test so state never leaks across tests.
 */
expect.extend(toHaveNoViolations);

afterEach(() => {
  clearMocks();
});
