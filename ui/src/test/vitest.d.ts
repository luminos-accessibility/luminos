import 'vitest';

/**
 * Augments Vitest's assertion interface with the custom matchers registered
 * in `setup.ts` that ship without their own Vitest typings.
 */
declare module 'vitest' {
  interface Assertion {
    toHaveNoViolations(): void;
  }
  interface AsymmetricMatchersContaining {
    toHaveNoViolations(): void;
  }
}
