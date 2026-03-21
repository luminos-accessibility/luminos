---
name: typescript-test-driven-development
description: >
  Guide for performing Test-Driven Development in TypeScript/React within the
  Luminos project's Spec-Driven Development methodology. Use this skill whenever
  an agent is implementing TypeScript subtasks from a SUBTASKS.md file, writing
  Vitest tests, testing React components, testing Zustand stores, testing Zod
  schemas, testing Tauri IPC wrappers, or executing red-green-refactor cycles on
  any TypeScript/React implementation story. Trigger when you see phrases like
  "implement TS story", "work on UI subtasks", "write frontend tests", "do TDD
  in TypeScript", "test React component", "test Zustand store", "test Zod
  schema", "implement T001" (in a TypeScript context), or any variation of
  test-driven TypeScript development. If TypeScript or React code is being
  written as part of a story, this skill should be active. Also use when
  reviewing whether TypeScript tests follow project conventions, when an agent
  needs to choose between test types (unit vs component vs integration vs
  accessibility), or when testing Tauri 2 IPC interactions from the frontend.
---

# TypeScript Test-Driven Development

This skill guides AI agents through the TDD workflow for TypeScript/React implementation stories in the Luminos control panel. It translates the spec-driven development methodology (specs/README.md) into concrete, step-by-step actions for each subtask.

The core idea is the same as the Rust TDD skill: tests are a **specification contract**. When you write tests first from acceptance criteria, the tests constrain the implementation to match the design. This is especially important for AI-generated code, where the implementation can drift from the spec if there's no test anchoring it.

TypeScript TDD in Luminos has a key difference from Rust TDD: the TypeScript compiler is less strict than Rust's borrow checker, so tests carry more of the verification burden. Zod schemas compensate for TypeScript's structural typing at runtime boundaries, and accessibility tests enforce requirements the type system can't express.

---

## Before You Start

When you pick up a story for implementation, read these files in order:

1. The epic's `HIGH_LEVEL_PLAN.md` -- understand context, shared types, decisions
2. Your story's `STORY.md` -- understand requirements and acceptance criteria
3. Your story's `DESIGN.md` -- understand architecture, APIs, and testing strategy
4. Your story's `SUBTASKS.md` -- find your current task (first unchecked item)

Do not read other stories' files. The HIGH_LEVEL_PLAN.md Shared Context section contains everything you need from earlier stories.

---

## The TDD Cycle

Every implementation task in SUBTASKS.md follows three phases. Execute them **sequentially** -- complete each phase fully before moving to the next.

### Phase 1: Red -- Write Failing Tests

The goal is to express the expected behavior as tests **before any implementation exists**. This is the most important phase because it forces you to think about behavior, not code.

**Step-by-step:**

1. Read the task's "Traces to" field to find which acceptance criteria (AC-X.X) and functional requirements (FR-X) this task covers.

2. Read the DESIGN.md Testing Strategy table to find the test type and verification method for each AC.

3. Translate each acceptance criterion into one or more test functions using the Given-When-Then to Arrange-Act-Assert mapping:

   ```typescript
   // AC-1.1: Given valid settings from the engine, when hydration completes,
   //         then the store is populated and isHydrated is true.

   test('settings_store_hydrate_populates_state_from_engine', async () => {
     // Arrange -- set up the "Given" precondition
     mockIPC((cmd) => {
       if (cmd === 'get_current_settings') {
         return { magnification: { zoomLevel: 5.0, mode: 'FullScreen' } };
       }
     });

     // Act -- perform the "When" action
     await useSettingsStore.getState().hydrate();

     // Assert -- verify the "Then" outcome
     const state = useSettingsStore.getState();
     expect(state.isHydrated).toBe(true);
     expect(state.settings.magnification.zoomLevel).toBe(5.0);
   });
   ```

4. Write the test functions. Follow these naming and placement conventions:
   - **Names:** `{module}_{behavior}_{condition}` -- e.g., `settings_store_hydrate_populates_state_from_engine`
   - **Placement:** Test files are colocated with their source files: `useSettingsStore.test.ts` next to `useSettingsStore.ts`, `ZoomLevelSlider.test.tsx` next to `ZoomLevelSlider.tsx`
   - **Schema tests:** Use the `.schema.test.ts` suffix: `settings.schema.test.ts` next to `settings.ts`

5. **Verify the Red phase.** Run:
   ```bash
   pnpm --dir ui test -- --run -t "your_test_name_prefix"
   ```
   The tests should **fail** (compile errors count as failures). If they pass, something is wrong -- you may be testing existing behavior rather than new behavior, or the test is tautological.

   Record the failure output. This confirms your tests are actually testing something.

**Mental discipline:** During this phase, think only about **what** the code should do, not **how** it will do it. Write tests from the STORY.md acceptance criteria and DESIGN.md API contracts, not from knowledge of the implementation you plan to write.

### Phase 2: Green -- Minimal Implementation

The goal is the simplest code that makes all Red-phase tests pass. Nothing more.

**Step-by-step:**

1. Implement just enough code to make the tests pass. Resist the urge to add error handling for cases you haven't tested, optimize performance, or build abstractions you'll "need later."

2. Follow the type signatures and APIs from DESIGN.md exactly. These are contracts that other stories depend on.

3. Run the tests:
   ```bash
   pnpm --dir ui test -- --run -t "your_test_name_prefix"
   ```
   All tests from the Red phase should now pass. If any fail, fix the implementation (not the tests) until they pass.

4. Run the linter and type checker:
   ```bash
   pnpm --dir ui lint && pnpm --dir ui typecheck
   ```
   Fix any errors before proceeding.

**The key rule:** if you find yourself writing code that isn't needed to pass a test, either write a test for it first (go back to Red) or leave it for the Refactor phase.

### Phase 3: Refactor -- Clean Up

The goal is to improve code quality while keeping all tests green. The tests are your safety net.

**Step-by-step:**

1. Look for these improvement opportunities:
   - Duplicated code that can be extracted into helpers
   - Naming that doesn't match the project's conventions (CLAUDE.md)
   - `any` types that should be properly typed
   - Missing `import type` where only type information is used
   - Complex conditionals that can be simplified with early returns
   - Missing JSDoc on exported functions
   - Zod schemas that should use `.describe()` for better error messages

2. Make one refactoring change at a time. After each change, run:
   ```bash
   pnpm --dir ui test -- --run -t "your_test_name_prefix"
   ```
   If tests break, revert the change and try a different approach.

3. Run the full check suite when done:
   ```bash
   pnpm --dir ui test -- --run && pnpm --dir ui lint && pnpm --dir ui typecheck
   ```

---

## After Each Task

1. **Check off** the completed items in SUBTASKS.md (Red/Green/Refactor checkboxes).
2. **Write Completion Notes** -- what you actually implemented, files created/modified, test names, any deviations from the plan.
3. **Update epic Shared Context** if this task produced knowledge other stories need (new public types, Zod schemas, IPC wrapper patterns, discovered constraints).
4. **Update the Progress Summary table** in SUBTASKS.md.
5. At **phase boundaries** (between Phase 1->2, 2->3, etc. in the SUBTASKS.md), run the full test suite to confirm all previous tasks still pass:
   ```bash
   pnpm --dir ui test -- --run
   ```

---

## Test Infrastructure

### Setup File

All tests run with a shared setup file (`ui/src/test/setup.ts`) that initializes:
- `@testing-library/jest-dom/vitest` matchers (`.toBeInTheDocument()`, etc.)
- `vitest-axe` matchers (`.toHaveNoViolations()`)
- Tauri mock cleanup (`clearMocks()` in `afterEach`)
- WebCrypto polyfill for jsdom (required by some Tauri internals)

Zustand store reset is handled by a separate mock file (`ui/src/test/__mocks__/zustand.ts`) that captures initial state and resets all stores between tests automatically.

You do not need to write `afterEach(clearMocks)` or reset stores manually in individual test files -- the infrastructure handles this.

### Tauri IPC Mocking

All Tauri IPC calls are mocked via `@tauri-apps/api/mocks`. The key function is `mockIPC()`:

```typescript
import { mockIPC } from '@tauri-apps/api/mocks';

beforeEach(() => {
  mockIPC((cmd, args) => {
    switch (cmd) {
      case 'get_current_settings':
        return { magnification: { zoomLevel: 5.0, mode: 'FullScreen' } };
      case 'set_zoom_level':
        return null;
      default:
        throw new Error(`Unmocked command: ${cmd}`);
    }
  });
});
```

**To also mock events** (needed when testing `listen()` or `emit()`), pass the `shouldMockEvents` option (requires Tauri >= 2.7.0):

```typescript
import { mockIPC } from '@tauri-apps/api/mocks';
import { emit, listen } from '@tauri-apps/api/event';

beforeEach(() => {
  mockIPC((cmd) => { /* handle commands */ }, { shouldMockEvents: true });
});

test('store_updates_on_engine_event', async () => {
  const { subscribeToEvents } = useTtsStore.getState();
  await subscribeToEvents();

  await emit('tts-status-changed', { status: 'Speaking', voiceId: 'en-us' });

  await waitFor(() => {
    expect(useTtsStore.getState().status).toBe('Speaking');
  });
});
```

**Spy on IPC calls** to verify a component called the right command:

```typescript
test('zoom_slider_calls_set_zoom_level', async () => {
  mockIPC(() => null);
  const spy = vi.spyOn(window.__TAURI_INTERNALS__, 'invoke');

  render(<ZoomLevelSlider />);
  await userEvent.setup().keyboard('{ArrowRight}');

  expect(spy).toHaveBeenCalledWith(
    'set_zoom_level',
    expect.objectContaining({ level: expect.any(Number) })
  );
});
```

---

## Test Writing Patterns

### Choosing the Right Test Type

| Situation | Test Type | File Suffix | Dependencies |
|-----------|-----------|-------------|-------------|
| Zod schema validation (accept/reject) | Unit test | `.schema.test.ts` | None |
| Zustand store logic (hydration, actions) | Unit test | `.test.ts` | Mocked IPC |
| IPC wrapper functions (delegation, Zod parsing) | Unit test | `.test.ts` | Mocked IPC |
| Compiled-in defaults validity | Unit test | `.test.ts` | None |
| React component rendering and interaction | Component test | `.test.tsx` | JSDOM, mocked IPC |
| Accessibility (WCAG 2.1 AA) | Component test | `.test.tsx` | JSDOM, axe-core |
| IPC roundtrip (TS types match Rust serde) | Integration test | `.integration.test.ts` | Full Tauri process |
| Keyboard navigation, focus management | Component test | `.test.tsx` | JSDOM, userEvent |

### Zod Schema Tests

Every Zod schema needs both accept and reject tests. Schemas are the contract between TypeScript and Rust -- test them like they're the most important boundary in your code, because they are.

```typescript
// ui/src/types/settings.schema.test.ts
import { AppSettingsSchema } from './settings';

test('settings_schema_accepts_valid_complete_settings', () => {
  const valid = {
    magnification: { zoomLevel: 5.0, mode: 'FullScreen', trackingMode: 'Cursor' },
    // ... all required fields
  };
  const result = AppSettingsSchema.safeParse(valid);
  expect(result.success).toBe(true);
});

test('settings_schema_rejects_zoom_below_minimum', () => {
  const invalid = { magnification: { zoomLevel: 0.5, mode: 'FullScreen' } };
  const result = AppSettingsSchema.safeParse(invalid);
  expect(result.success).toBe(false);
  if (!result.success) {
    expect(result.error.issues[0].path).toContain('zoomLevel');
  }
});
```

**Defaults validity test** -- catches schema/defaults drift:

```typescript
import { DEFAULT_SETTINGS } from '../constants/defaults';
import { AppSettingsSchema } from '../types/settings';

test('default_settings_are_valid_against_schema', () => {
  const result = AppSettingsSchema.safeParse(DEFAULT_SETTINGS);
  expect(result.success).toBe(true);
});
```

**Enum variant coverage** -- verify all PascalCase enum variants are handled:

```typescript
import { MagnificationModeSchema } from './enums';

const VALID_MODES = ['FullScreen', 'Lens', 'Docked'] as const;

test.each(VALID_MODES)('magnification_mode_schema_accepts_%s', (mode) => {
  expect(MagnificationModeSchema.safeParse(mode).success).toBe(true);
});

test('magnification_mode_schema_rejects_snake_case', () => {
  expect(MagnificationModeSchema.safeParse('full_screen').success).toBe(false);
});
```

### Zustand Store Tests

Test stores in isolation (without rendering components) by calling `getState()` and `setState()` directly.

```typescript
// ui/src/hooks/useSettingsStore.test.ts
import { mockIPC } from '@tauri-apps/api/mocks';
import { useSettingsStore } from './useSettingsStore';

beforeEach(() => {
  mockIPC((cmd) => {
    if (cmd === 'get_current_settings') {
      return { magnification: { zoomLevel: 5.0, mode: 'FullScreen' } };
    }
    if (cmd === 'set_zoom_level') return null;
  });
});

test('settings_store_hydrate_sets_is_hydrating_false', async () => {
  await useSettingsStore.getState().hydrate();
  expect(useSettingsStore.getState().isHydrated).toBe(true);
});

test('settings_store_set_zoom_clamps_to_bounds', () => {
  useSettingsStore.setState({
    settings: { magnification: { zoomLevel: 5.0 } },
    isHydrated: true,
  });

  useSettingsStore.getState().setZoomLevel(100); // above max
  expect(useSettingsStore.getState().settings.magnification.zoomLevel).toBe(20);

  useSettingsStore.getState().setZoomLevel(0.1); // below min
  expect(useSettingsStore.getState().settings.magnification.zoomLevel).toBe(1.5);
});
```

For optimistic update testing, immer immutability testing, and event-driven store updates, see `references/test-patterns.md` Sections 4-5.

### React Component Tests

Components are tested with React Testing Library. Query by accessible roles and labels -- this simultaneously tests accessibility and behavior.

```typescript
// ui/src/components/magnification/ZoomLevelSlider.test.tsx
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { useSettingsStore } from '../../hooks/useSettingsStore';
import { ZoomLevelSlider } from './ZoomLevelSlider';

beforeEach(() => {
  mockIPC(() => null);
  useSettingsStore.setState({
    settings: { magnification: { zoomLevel: 5.0, mode: 'FullScreen' } },
    isHydrated: true,
  });
});

test('zoom_slider_renders_current_zoom_level', () => {
  render(<ZoomLevelSlider />);
  expect(screen.getByRole('slider', { name: /zoom level/i })).toHaveValue('5');
  expect(screen.getByText('5.0x')).toBeInTheDocument();
});

test('zoom_slider_reverts_on_ipc_error', async () => {
  mockIPC((cmd) => {
    if (cmd === 'set_zoom_level') return Promise.reject(new Error('Engine busy'));
    return null;
  });

  const user = userEvent.setup();
  render(<ZoomLevelSlider />);
  const slider = screen.getByRole('slider', { name: /zoom level/i });

  await user.click(slider);
  await user.keyboard('{ArrowRight}');

  await waitFor(() => {
    expect(slider).toHaveValue('5'); // reverted
  });
  expect(screen.getByRole('alert')).toHaveTextContent(/engine busy/i);
});
```

**Query priority for accessibility** (always prefer the highest applicable):

1. `getByRole('slider', { name: /zoom level/i })` -- best; tests ARIA role + accessible name
2. `getByLabelText('Zoom Level')` -- good; tests label association
3. `getByText('5.0x')` -- acceptable for display text
4. `getByTestId('zoom-slider')` -- last resort; no accessibility value

**Always use `userEvent` over `fireEvent`.** `userEvent` produces realistic event sequences (focus, keydown, keyup, input) that catch real-world focus and interaction issues:

```typescript
const user = userEvent.setup();
await user.click(element);    // not fireEvent.click(element)
await user.keyboard('{Tab}'); // not fireEvent.keyDown(element, { key: 'Tab' })
```

### Accessibility Tests

Every page-level component gets an axe-core check. This is non-negotiable -- Luminos is an accessibility application. An inaccessible control panel is a critical bug.

```typescript
import { axe } from 'vitest-axe';

test('magnification_page_has_no_accessibility_violations', async () => {
  const { container } = render(<MagnificationPage />);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});
```

For multi-state accessibility testing, ARIA attribute testing, keyboard navigation, and focus management patterns, see `references/test-patterns.md` Section 11.

### What to Test vs. What the Compiler Guarantees

TypeScript's type system is structural, not nominal, and has escape hatches (`as`, `any`). It catches fewer things at compile time than Rust. Focus your tests on:

**Test these (TypeScript won't catch them):**
- Business logic correctness (wrong arithmetic, off-by-one in zoom clamping)
- Zod schema validation (runtime boundary between TS and Rust)
- IPC response handling (correct commands, correct arguments, error paths)
- Optimistic update + revert sequences
- State machine transitions in stores
- Accessibility (ARIA attributes, focus management, keyboard navigation)
- Enum variant wire format (PascalCase not snake_case)

**Skip these (the compiler catches them):**
- Type mismatches on function arguments (if you're using strong typing)
- Missing required properties on objects (if the type is correct)
- Import resolution (if `tsc --noEmit` passes)

---

## Anti-Patterns to Avoid

These are the most common ways AI agents get TypeScript TDD wrong. Being aware of them is the first defense.

1. **Writing tests and implementation simultaneously.** This defeats TDD's purpose -- tests end up testing the implementation you already wrote rather than the spec. Complete all test writing before thinking about implementation.

2. **Tests that always pass.** If a test can't fail, it tests nothing. Always verify the Red phase -- your tests should fail before the implementation exists.

3. **Excessive mocking.** When everything is mocked, tests verify mock behavior, not real behavior. Mock only at the IPC boundary (Tauri's `invoke`/`listen`). For internal modules, use real Zustand stores with the auto-reset pattern.

4. **Testing implementation details.** Test behavior ("given this store state, when I render this component, the slider shows the correct value") not structure ("the component calls `useSettingsStore` with these specific selectors"). Implementation-coupled tests break during refactoring.

5. **Modifying tests to make them pass.** If a test fails during the Green phase, fix the implementation. Only modify a test if the test itself has a bug (wrong assertion) or the AC changed.

6. **Skipping the Red verification.** Always run tests after writing them and confirm they fail. This is the most commonly skipped step and the one most likely to produce tautological tests.

7. **Kitchen-sink tests.** One test verifying 15 things. Each test should verify one behavior. Multiple assertions are fine if they all relate to the same behavior.

8. **Using `fireEvent` instead of `userEvent`.** `fireEvent` dispatches single synthetic events. `userEvent` simulates full user interaction sequences (focus, keydown, input, keyup, blur). The difference matters for accessibility testing -- `fireEvent.click` doesn't move focus, `userEvent.click` does.

9. **Querying by `data-testid` first.** Always prefer accessible queries (`getByRole`, `getByLabelText`). If you can't find an element by its accessible role, that's an accessibility bug to fix, not a reason to add a `data-testid`.

10. **Ignoring `waitFor` for async state.** Store actions that call IPC are async. After triggering an async action, use `waitFor()` or `findBy*` queries to wait for the state to settle, rather than asserting synchronously.

---

## Verification Commands Quick Reference

```bash
# Run specific tests by name pattern (TDD inner loop)
pnpm --dir ui test -- --run -t "settings_store_hydrate"

# Run all tests in a specific file
pnpm --dir ui test -- --run src/hooks/useSettingsStore.test.ts

# Run all schema tests
pnpm --dir ui test -- --run -t "schema"

# Run all tests matching a file pattern
pnpm --dir ui test -- --run "**/*.schema.test.ts"

# Run full test suite (phase boundary checkpoint)
pnpm --dir ui test -- --run

# Run with verbose output for debugging failures
pnpm --dir ui test -- --run -t "failing_test" --reporter=verbose

# Lint check (run after Green and Refactor phases)
pnpm --dir ui lint

# Type check
pnpm --dir ui typecheck

# Full pre-push validation
pnpm --dir ui lint && pnpm --dir ui typecheck && pnpm --dir ui test -- --run
```

---

## Quick Reference Checklist

For each subtask in SUBTASKS.md:

- [ ] Read the task's "Traces to" field -- know which ACs you're covering
- [ ] **RED:** Write test(s) from the acceptance criteria
- [ ] **RED:** Run tests -- confirm they FAIL
- [ ] **GREEN:** Write minimal implementation to pass
- [ ] **GREEN:** Run tests -- confirm they PASS
- [ ] **GREEN:** Run lint + typecheck -- fix any errors
- [ ] **REFACTOR:** Improve code quality (naming, types, duplication, JSDoc)
- [ ] **REFACTOR:** Run tests -- confirm they still PASS
- [ ] Check off items in SUBTASKS.md
- [ ] Write Completion Notes
- [ ] Update epic Shared Context if needed

---

## Additional Patterns

For detailed patterns on property-based testing with Zod schemas, testing event-driven store updates, IPC integration testing, and the Zustand mock setup, read `references/test-patterns.md`.
