/**
 * Minimal ambient typings for `jest-axe`.
 *
 * `jest-axe` ships no type declarations and the community `@types/jest-axe`
 * package depends on `@types/jest`, which conflicts with Vitest's globals.
 * We only consume three exports, so we declare exactly that slice here and
 * augment Vitest's matcher interface separately in `setup.ts`.
 */
declare module 'jest-axe' {
  import type { AxeResults, RunOptions, Spec } from 'axe-core';

  /** Runs axe-core against an HTML element or document fragment. */
  export function axe(
    html: Element | Document | DocumentFragment | string,
    options?: RunOptions
  ): Promise<AxeResults>;

  /** Creates a pre-configured `axe` runner with shared options. */
  export function configureAxe(options?: {
    globalOptions?: Spec;
    [key: string]: unknown;
  }): typeof axe;

  /** Vitest/Jest matcher asserting zero accessibility violations. */
  export const toHaveNoViolations: {
    toHaveNoViolations(results: AxeResults): {
      pass: boolean;
      message(): string;
    };
  };
}
