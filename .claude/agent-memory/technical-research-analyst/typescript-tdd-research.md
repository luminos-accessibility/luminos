# TypeScript TDD Research Notes (2026-03-21)

## Stack: Tauri 2.0 + React 19 + Vitest + RTL + Zustand v5 + Zod v3

## Key Sources (verified, authoritative)
- Vitest docs: vitest.dev (Context7 ID: /vitest-dev/vitest, benchmark 88.26)
- Zustand testing guide: github.com/pmndrs/zustand/blob/main/docs/learn/guides/testing.md
- Tauri v2 mocking docs: v2.tauri.app/develop/tests/mocking/
- DeepWiki Tauri mocking: deepwiki.com/tauri-apps/tauri/4.4-testing-and-mocking
- vitest-axe: github.com/chaance/vitest-axe (fork of jest-axe, v0.1.0)
- zod-fast-check: github.com/DavidTimms/zod-fast-check (v0.9.0, bridges Zod v3 to fast-check)
- fast-check ecosystem: fast-check.dev/docs/ecosystem/
- Simon Willison agentic TDD: simonwillison.net/guides/agentic-engineering-patterns/red-green-tdd/
- alexop.dev: Custom TDD workflow with Claude Code using skills + subagents

## Tauri v2 IPC Mocking
- `mockIPC(cb, options?)` patches `window.__TAURI_INTERNALS__`
- `shouldMockEvents: true` option added in Tauri 2.7.0 - required for `listen()`/`emit()` mocking
- Without `shouldMockEvents`, `listen()` throws `TypeError: Cannot read properties of undefined (reading 'transformCallback')`
- `emitTo` and `emit_filter` NOT supported in mocking layer
- WebCrypto polyfill needed in jsdom for some Tauri operations
- Spy pattern: `vi.spyOn(window.__TAURI_INTERNALS__, 'invoke')` for call verification
- `clearMocks()` must be called in afterEach

## Zustand v5 Testing
- Official pattern: `__mocks__/zustand.ts` that wraps `create`/`createStore`
- Uses `store.getInitialState()` + `storeResetFns` Set for automatic reset
- `afterEach(() => { act(() => storeResetFns.forEach(fn => fn())) })`
- Supports both curried and uncurried `create()` patterns
- For Vitest: use `vi.importActual` instead of `jest.requireActual`
- Test stores directly: `const state = useStore.getState()` outside React

## vitest-axe Setup
- REQUIRES jsdom (NOT happy-dom, due to axe-core DOM compatibility)
- Install: `pnpm add -D vitest-axe`
- Setup file: `import { toHaveNoViolations } from 'vitest-axe/matchers'; expect.extend({ toHaveNoViolations })`
- Type augmentation needed if not importing `vitest/extend-expect`
- Alternative: `@chialab/vitest-axe` uses axe-core directly (different API)

## Zod Property-Based Testing
- `zod-fast-check` v0.9.0 bridges Zod v3 schemas to fast-check arbitraries
- `ZodFastCheck().inputOf(schema)` - generates valid parse inputs
- `ZodFastCheck().outputOf(schema)` - generates valid outputs (post-transform)
- `.override(schema, arbitrary)` for schemas with low-probability refinements
- WARNING: Refinements with low match probability will throw `ZodFastCheckGenerationError`
- Works with Vitest: `fc.assert(fc.property(arbitrary, (val) => { ... }))`

## Vitest TDD Workflow
- `vitest` (no args) starts watch mode, reruns only affected tests
- `vitest --reporter=verbose` for detailed TDD output
- `vitest -t "test name pattern"` for test name filtering
- `test.only()` / `describe.only()` for focused development
- Config: `clearMocks: true, mockReset: true, restoreMocks: true` for automatic cleanup
- File-level parallelism by default; sequential within file
- `--no-isolate` can speed up large suites but risks state leakage

## Luminos-Specific Notes
- Doc-05 Section 13 defines the TypeScript testing toolchain and example patterns
- Doc-07 Section 3.2 defines the full TypeScript test tool inventory
- Test naming convention: `{module}_{behavior}_{condition}` (matches Rust convention)
- Vitest config in `ui/vitest.config.ts` with jsdom environment
- Setup file at `ui/src/test/setup.ts` for mock initialization
