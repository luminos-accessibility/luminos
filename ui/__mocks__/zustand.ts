import { act } from '@testing-library/react';
import type * as ZustandExportedTypes from 'zustand';
import { afterEach, vi } from 'vitest';

export * from 'zustand';

const { create: actualCreate, createStore: actualCreateStore } =
  await vi.importActual<typeof ZustandExportedTypes>('zustand');

/** Reset functions captured from every store created during a test run. */
const storeResetFns = new Set<() => void>();

/**
 * Wraps the real Zustand `create` so each store's initial state is captured
 * and restored between tests. This mirrors Zustand's official test recipe and
 * removes the need for manual store resets in individual test files.
 */
const createUncurried = <T>(stateCreator: ZustandExportedTypes.StateCreator<T>) => {
  const store = actualCreate(stateCreator);
  const initialState = store.getInitialState();
  storeResetFns.add(() => {
    store.setState(initialState, true);
  });
  return store;
};

export const create = (<T>(stateCreator: ZustandExportedTypes.StateCreator<T> | undefined) => {
  return typeof stateCreator === 'function' ? createUncurried(stateCreator) : createUncurried;
}) as typeof ZustandExportedTypes.create;

const createStoreUncurried = <T>(stateCreator: ZustandExportedTypes.StateCreator<T>) => {
  const store = actualCreateStore(stateCreator);
  const initialState = store.getInitialState();
  storeResetFns.add(() => {
    store.setState(initialState, true);
  });
  return store;
};

export const createStore = (<T>(
  stateCreator: ZustandExportedTypes.StateCreator<T> | undefined
) => {
  return typeof stateCreator === 'function' ? createStoreUncurried(stateCreator) : createStoreUncurried;
}) as typeof ZustandExportedTypes.createStore;

afterEach(() => {
  act(() => {
    storeResetFns.forEach((resetFn) => {
      resetFn();
    });
  });
});
