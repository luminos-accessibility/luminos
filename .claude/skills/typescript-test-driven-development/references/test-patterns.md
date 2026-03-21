# TypeScript Test Patterns Reference

Detailed testing patterns for the Luminos control panel. Read this file when you need guidance on
specific testing techniques beyond the core TDD workflow in SKILL.md.

## Table of Contents

1. [Test Setup Infrastructure](#1-test-setup-infrastructure)
2. [Zustand Mock and Reset Pattern](#2-zustand-mock-and-reset-pattern)
3. [Property-Based Testing with Zod Schemas](#3-property-based-testing-with-zod-schemas)
4. [Testing Event-Driven State Updates](#4-testing-event-driven-state-updates)
5. [Optimistic Update Testing Patterns](#5-optimistic-update-testing-patterns)
6. [IPC Wrapper Testing](#6-ipc-wrapper-testing)
7. [Debounced IPC Calls](#7-debounced-ipc-calls)
8. [Multi-Window Testing](#8-multi-window-testing)
9. [Vitest Configuration](#9-vitest-configuration)
10. [Test Dependencies](#10-test-dependencies)
11. [Accessibility Testing Patterns](#11-accessibility-testing-patterns)

---

## 1. Test Setup Infrastructure

The centralized test setup file initializes all shared infrastructure. Individual test files
should not need to set up mocks, matchers, or polyfills manually.

```typescript
// ui/src/test/setup.ts
import '@testing-library/jest-dom/vitest';
import { toHaveNoViolations } from 'vitest-axe/matchers';
import { clearMocks } from '@tauri-apps/api/mocks';
import { randomFillSync } from 'node:crypto';

// --- Vitest Matchers ---
expect.extend({ toHaveNoViolations });

// --- WebCrypto Polyfill (required by some Tauri internals in jsdom) ---
if (!globalThis.crypto?.getRandomValues) {
  Object.defineProperty(globalThis, 'crypto', {
    value: {
      getRandomValues: (buffer: Uint8Array) => randomFillSync(buffer),
    },
  });
}

// --- Tauri Mock Cleanup ---
afterEach(() => {
  clearMocks();
});

// --- Zustand Store Reset ---
// Handled by __mocks__/zustand.ts (see Section 2)
```

**TypeScript type augmentation** for vitest-axe matchers:

```typescript
// ui/src/test/vitest.d.ts
import 'vitest';
import type { AxeMatchers } from 'vitest-axe/matchers';

declare module 'vitest' {
  interface Assertion extends AxeMatchers {}
  interface AsymmetricMatchersContaining extends AxeMatchers {}
}
```

---

## 2. Zustand Mock and Reset Pattern

This is the official Zustand testing pattern. It wraps Zustand's `create` and `createStore`
functions to capture the initial state of every store, then resets all stores between tests
via `afterEach`. This prevents state leakage between tests without requiring manual cleanup
in each test file.

```typescript
// ui/src/test/__mocks__/zustand.ts
import { act } from '@testing-library/react';
import type * as ZustandExportedTypes from 'zustand';
export * from 'zustand';

const { create: actualCreate, createStore: actualCreateStore } =
  await vi.importActual<typeof ZustandExportedTypes>('zustand');

export const storeResetFns = new Set<() => void>();

const createUncurried = <T>(
  stateCreator: ZustandExportedTypes.StateCreator<T>,
) => {
  const store = actualCreate(stateCreator);
  const initialState = store.getInitialState();
  storeResetFns.add(() => {
    store.setState(initialState, true);
  });
  return store;
};

export const create = (<T>(
  stateCreator: ZustandExportedTypes.StateCreator<T>,
) => {
  return typeof stateCreator === 'function'
    ? createUncurried(stateCreator)
    : createUncurried;
}) as typeof ZustandExportedTypes.create;

const createStoreUncurried = <T>(
  stateCreator: ZustandExportedTypes.StateCreator<T>,
) => {
  const store = actualCreateStore(stateCreator);
  const initialState = store.getInitialState();
  storeResetFns.add(() => {
    store.setState(initialState, true);
  });
  return store;
};

export const createStore = (<T>(
  stateCreator: ZustandExportedTypes.StateCreator<T>,
) => {
  return typeof stateCreator === 'function'
    ? createStoreUncurried(stateCreator)
    : createStoreUncurried;
}) as typeof ZustandExportedTypes.createStore;

afterEach(() => {
  act(() => {
    storeResetFns.forEach((resetFn) => {
      resetFn();
    });
  });
});
```

To activate this mock, configure a Vitest alias:

```typescript
// In vitest.config.ts
test: {
  alias: {
    zustand: new URL('./src/test/__mocks__/zustand.ts', import.meta.url).pathname,
  },
}
```

**How to use in tests:**

```typescript
// Store is automatically reset between tests. Just set the state you need:
beforeEach(() => {
  useSettingsStore.setState({
    settings: { magnification: { zoomLevel: 5.0, mode: 'FullScreen' } },
    isHydrated: true,
  });
});

// After each test, the store automatically reverts to its initial state.
```

---

## 3. Property-Based Testing with Zod Schemas

For schemas with complex validation rules, property-based testing generates hundreds of random
valid inputs to verify invariants. The `zod-fast-check` library generates fast-check arbitraries
from Zod schemas.

```typescript
import * as fc from 'fast-check';
import { ZodFastCheck } from 'zod-fast-check';
import { AppSettingsSchema } from '../types/settings';

const settingsArbitrary = ZodFastCheck().inputOf(AppSettingsSchema);

test('settings_schema_roundtrip_parse_is_idempotent', () => {
  fc.assert(
    fc.property(settingsArbitrary, (settings) => {
      const parsed = AppSettingsSchema.parse(settings);
      const reparsed = AppSettingsSchema.parse(parsed);
      expect(reparsed).toEqual(parsed);
    }),
    { numRuns: 100 }
  );
});

test('settings_schema_parsed_zoom_is_within_bounds', () => {
  fc.assert(
    fc.property(settingsArbitrary, (settings) => {
      const parsed = AppSettingsSchema.parse(settings);
      expect(parsed.magnification.zoomLevel).toBeGreaterThanOrEqual(1.0);
      expect(parsed.magnification.zoomLevel).toBeLessThanOrEqual(40.0);
    }),
  );
});
```

**Override for constrained refinements.** Zod schemas with narrow refinements (e.g., hex color
strings) have low probability of being randomly generated. Use `.override()`:

```typescript
import { HexColorSchema } from '../types/settings';

const zfc = ZodFastCheck()
  .override(
    HexColorSchema,
    fc.hexaString({ minLength: 6, maxLength: 6 }).map(s => `#${s}`)
  );

const settingsArbitrary = zfc.inputOf(AppSettingsSchema);
```

**When to use property-based testing:** Settings validation invariants, schema roundtrip stability,
zoom level bounds enforcement, any schema with complex refinements or transforms.

**When NOT to use:** Simple accept/reject cases (use explicit tests), component behavior
(not randomly testable), accessibility (requires DOM rendering).

---

## 4. Testing Event-Driven State Updates

The Luminos engine sends events to the control panel via Tauri's `emit()` / `listen()` system.
Testing these requires the `shouldMockEvents` option.

```typescript
import { mockIPC } from '@tauri-apps/api/mocks';
import { emit } from '@tauri-apps/api/event';
import { waitFor } from '@testing-library/react';
import { useTtsStore } from '../hooks/useTtsStore';

beforeEach(() => {
  mockIPC(() => null, { shouldMockEvents: true });
});

test('tts_store_updates_status_on_engine_event', async () => {
  // Subscribe to events (this calls listen() internally)
  await useTtsStore.getState().subscribeToEvents();

  // Simulate engine emitting a status change
  await emit('tts-status-changed', {
    status: 'Speaking',
    voiceId: 'en-us',
  });

  // Wait for the store to process the event
  await waitFor(() => {
    expect(useTtsStore.getState().status).toBe('Speaking');
  });
});

test('settings_store_syncs_zoom_on_hotkey_event', async () => {
  useSettingsStore.setState({
    settings: { magnification: { zoomLevel: 5.0 } },
    isHydrated: true,
  });

  await useSettingsStore.getState().subscribeToEvents();
  await emit('zoom-changed', 10.0);

  await waitFor(() => {
    expect(useSettingsStore.getState().settings.magnification.zoomLevel).toBe(10.0);
  });
});
```

**Known limitation:** `emitTo` and `emit_filter` are not supported in the current Tauri mock
implementation. Testing targeted window events requires the full `tauri-driver` integration
test setup.

---

## 5. Optimistic Update Testing Patterns

The control panel uses optimistic updates: the UI updates immediately, then calls IPC, and reverts
if the IPC call fails. This pattern needs careful testing because three things happen in sequence.

**Pattern: Success path**

```typescript
test('settings_store_optimistic_zoom_updates_immediately', async () => {
  useSettingsStore.setState({
    settings: { magnification: { zoomLevel: 5.0 } },
    isHydrated: true,
  });

  // Don't await -- check the optimistic state immediately
  const promise = useSettingsStore.getState().setZoomLevel(10.0);

  // Optimistic update happened synchronously
  expect(useSettingsStore.getState().settings.magnification.zoomLevel).toBe(10.0);

  // IPC completes successfully
  await promise;
  expect(useSettingsStore.getState().settings.magnification.zoomLevel).toBe(10.0);
});
```

**Pattern: Failure with revert**

```typescript
test('settings_store_optimistic_zoom_reverts_on_error', async () => {
  mockIPC((cmd) => {
    if (cmd === 'set_zoom_level') return Promise.reject(new Error('Engine error'));
    return null;
  });

  useSettingsStore.setState({
    settings: { magnification: { zoomLevel: 5.0 } },
    isHydrated: true,
  });

  await useSettingsStore.getState().setZoomLevel(10.0);

  // Reverted to original value
  expect(useSettingsStore.getState().settings.magnification.zoomLevel).toBe(5.0);
  // Error captured for UI notification
  expect(useSettingsStore.getState().lastError).toContain('Engine error');
});
```

**Pattern: Component-level revert (tests the full cycle in the UI)**

```typescript
test('zoom_slider_shows_original_value_after_ipc_failure', async () => {
  mockIPC((cmd) => {
    if (cmd === 'set_zoom_level') return Promise.reject(new Error('busy'));
    return null;
  });

  useSettingsStore.setState({
    settings: { magnification: { zoomLevel: 5.0 } },
    isHydrated: true,
  });

  const user = userEvent.setup();
  render(<ZoomLevelSlider />);

  const slider = screen.getByRole('slider', { name: /zoom level/i });
  await user.click(slider);
  await user.keyboard('{ArrowRight}');

  // Wait for the revert to propagate to the DOM
  await waitFor(() => {
    expect(slider).toHaveValue('5');
  });

  // Error toast displayed
  expect(screen.getByRole('alert')).toBeInTheDocument();
});
```

---

## 6. IPC Wrapper Testing

IPC wrapper functions in `ui/src/ipc/commands.ts` delegate to `tauri-specta` bindings and
validate responses with Zod. Test that they:
1. Call the correct command with correct arguments
2. Parse responses through Zod schemas
3. Throw meaningful errors on invalid responses

```typescript
// ui/src/ipc/commands.test.ts
import { mockIPC } from '@tauri-apps/api/mocks';
import { getCurrentSettings, setZoomLevel } from './commands';

test('ipc_get_current_settings_parses_response_with_zod', async () => {
  mockIPC((cmd) => {
    if (cmd === 'get_current_settings') {
      return {
        magnification: { zoomLevel: 5.0, mode: 'FullScreen', trackingMode: 'Cursor' },
      };
    }
  });

  const settings = await getCurrentSettings();
  expect(settings.magnification.zoomLevel).toBe(5.0);
  expect(settings.magnification.mode).toBe('FullScreen');
});

test('ipc_get_current_settings_throws_on_invalid_response', async () => {
  mockIPC((cmd) => {
    if (cmd === 'get_current_settings') {
      return { invalid: 'data' }; // doesn't match schema
    }
  });

  await expect(getCurrentSettings()).rejects.toThrow();
});

test('ipc_set_zoom_level_passes_correct_args', async () => {
  const spy = vi.fn().mockResolvedValue(null);
  mockIPC((cmd, args) => {
    if (cmd === 'set_zoom_level') return spy(args);
  });

  await setZoomLevel(7.5);
  expect(spy).toHaveBeenCalledWith({ level: 7.5 });
});
```

---

## 7. Debounced IPC Calls

Slider components use `use-debounce` to debounce IPC calls (150ms, per the project spec).
Testing debounced behavior requires Vitest's fake timers.

```typescript
import { vi } from 'vitest';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

test('zoom_slider_debounces_ipc_calls', async () => {
  const ipcSpy = vi.fn().mockResolvedValue(null);
  mockIPC((cmd) => {
    if (cmd === 'set_zoom_level') return ipcSpy();
    return null;
  });

  const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
  render(<ZoomLevelSlider />);

  const slider = screen.getByRole('slider', { name: /zoom level/i });
  await user.click(slider);

  // Rapid keypresses
  await user.keyboard('{ArrowRight}');
  await user.keyboard('{ArrowRight}');
  await user.keyboard('{ArrowRight}');

  // IPC not called yet (within debounce window)
  expect(ipcSpy).not.toHaveBeenCalled();

  // Advance past debounce threshold
  await vi.advanceTimersByTimeAsync(150);

  // Only one IPC call with the final value
  expect(ipcSpy).toHaveBeenCalledTimes(1);
});
```

**Important:** When using fake timers with `userEvent`, pass `advanceTimers` to `userEvent.setup()`
so that `userEvent`'s internal delays resolve correctly.

---

## 8. Multi-Window Testing

Luminos has a dual-window architecture (control panel + magnification overlay). Use
`mockWindows()` to simulate window metadata:

```typescript
import { mockWindows } from '@tauri-apps/api/mocks';

test('control_panel_detects_overlay_window', async () => {
  mockWindows('control-panel', 'magnification-overlay');

  const { getCurrent, getAll } = await import('@tauri-apps/api/webviewWindow');
  expect(getCurrent()).toHaveProperty('label', 'control-panel');
  expect(getAll().map(w => w.label)).toContain('magnification-overlay');
});
```

---

## 9. Vitest Configuration

Complete recommended Vitest configuration for the Luminos control panel:

```typescript
// ui/vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import tsconfigPaths from 'vite-tsconfig-paths';

export default defineConfig({
  plugins: [tsconfigPaths(), react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    clearMocks: true,      // clear mock.calls, mock.instances between tests
    mockReset: true,       // reset mock state between tests
    restoreMocks: true,    // restore original implementations between tests
    alias: {
      // Auto-reset Zustand stores between tests
      zustand: new URL('./src/test/__mocks__/zustand.ts', import.meta.url).pathname,
    },
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'src/ipc/bindings.ts',   // Auto-generated by tauri-specta
        'src/test/**',           // Test utilities and mocks
      ],
    },
  },
});
```

**Why jsdom, not happy-dom:** `vitest-axe` (and axe-core) requires jsdom for correct DOM spec
compliance. Happy-dom has known compatibility issues with axe-core's DOM traversal. The
performance difference is negligible for the expected test suite size.

---

## 10. Test Dependencies

| Package | Purpose | Notes |
|---------|---------|-------|
| `vitest` | Test runner | Fast, native ESM, Vite-integrated |
| `@testing-library/react` | Component test utilities | DOM-based component testing |
| `@testing-library/jest-dom` | DOM matchers for Vitest | `.toBeInTheDocument()`, `.toHaveValue()`, etc. |
| `@testing-library/user-event` | User interaction simulation | Realistic event sequences |
| `vitest-axe` | Accessibility testing | `.toHaveNoViolations()` matcher |
| `@tauri-apps/api/mocks` | Tauri IPC mocking | `mockIPC()`, `mockWindows()`, `clearMocks()` |
| `jsdom` | DOM environment | Required by axe-core |
| `fast-check` | Property-based testing | Random input generation |
| `zod-fast-check` | Zod -> fast-check bridge | Generates arbitraries from Zod schemas |
| `vite-tsconfig-paths` | Path alias resolution | Maps `@/` to `src/` in Vitest |
| `eslint-plugin-jsx-a11y` | JSX accessibility linting | Static analysis for ARIA/WCAG |

All are `devDependencies` -- they do not affect the production bundle.

---

## 11. Accessibility Testing Patterns

Luminos is an accessibility application. These patterns go beyond the basic axe-core check
shown in SKILL.md.

### Multi-State Accessibility Testing

Accessibility violations often appear only in specific UI states (expanded, error, loading).
Test multiple states:

```typescript
test('voice_selector_accessible_in_expanded_state', async () => {
  const user = userEvent.setup();
  const { container } = render(<VoiceSelector />);

  // Collapsed state
  expect(await axe(container)).toHaveNoViolations();

  // Expanded state
  await user.click(screen.getByRole('combobox', { name: /voice/i }));
  expect(await axe(container)).toHaveNoViolations();
});
```

### ARIA Attribute Testing

axe-core catches missing roles and labels but may not validate that specific ARIA attributes
match the application's accessibility contract. Test these directly:

```typescript
test('tts_status_indicator_announces_error_assertively', () => {
  useTtsStore.setState({ status: 'Error', errorMessage: 'Model not found' });
  render(<TtsStatusIndicator />);

  const alert = screen.getByRole('alert');
  expect(alert).toHaveAttribute('aria-live', 'assertive');
  expect(alert).toHaveTextContent('Model not found');
});

test('zoom_slider_has_accessible_value_text', () => {
  useSettingsStore.setState({
    settings: { magnification: { zoomLevel: 5.0 } },
  });
  render(<ZoomLevelSlider />);

  const slider = screen.getByRole('slider', { name: /zoom level/i });
  expect(slider).toHaveAttribute('aria-valuemin', '1');
  expect(slider).toHaveAttribute('aria-valuemax', '40');
  expect(slider).toHaveAttribute('aria-valuenow', '5');
  expect(slider).toHaveAttribute('aria-valuetext', '5.0x magnification');
});
```

### Keyboard Navigation Testing

Verify tab order follows visual order and that all controls are keyboard-accessible:

```typescript
test('settings_page_tab_order_follows_visual_order', async () => {
  const user = userEvent.setup();
  render(<MagnificationPage />);

  await user.tab();
  expect(screen.getByRole('slider', { name: /zoom level/i })).toHaveFocus();

  await user.tab();
  expect(screen.getByRole('combobox', { name: /magnification mode/i })).toHaveFocus();

  await user.tab();
  expect(screen.getByRole('combobox', { name: /tracking mode/i })).toHaveFocus();
});
```

### Focus Management and Focus Trapping

Dialogs must trap focus and return it to the trigger element on close:

```typescript
test('dialog_traps_focus_and_returns_on_close', async () => {
  const user = userEvent.setup();
  render(<KeybindingTable />);

  // Open capture dialog
  await user.click(screen.getByRole('button', { name: /change zoom in hotkey/i }));
  const dialog = screen.getByRole('dialog');
  expect(dialog).toBeInTheDocument();

  // Focus should be inside dialog
  expect(document.activeElement).toBeInstanceOf(HTMLElement);
  expect(dialog.contains(document.activeElement)).toBe(true);

  // Escape closes and returns focus to trigger
  await user.keyboard('{Escape}');
  expect(dialog).not.toBeInTheDocument();
  expect(screen.getByRole('button', { name: /change zoom in hotkey/i })).toHaveFocus();
});
```

### What Automated Testing Cannot Catch

Automated axe checks and static analysis catch approximately 30-40% of accessibility issues.
The following require manual testing (tracked in the release checklist per doc-07 Section 8):

- Screen reader narration accuracy (Orca on Linux, VoiceOver on macOS)
- Logical focus order (as opposed to just reachability)
- Meaningful content structure and heading hierarchy
- Color contrast in custom-rendered elements
- Magnification overlay coexistence with screen readers
