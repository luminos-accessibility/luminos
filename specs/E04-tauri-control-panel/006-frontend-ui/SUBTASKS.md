# Subtasks: Story E04/006 -- Frontend Control Panel UI

**Status:** NOT STARTED
**Started:** ---
**Completed:** ---
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup (scaffold) | 2 | 0 | 0 | 2 |
| 2. Core (store, gate, controls) | 5 | 0 | 0 | 5 |
| 3. Integration (events, a11y) | 2 | 0 | 0 | 2 |
| 4. Polish & Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **10** | **0** | **0** | **10** |

> TS/React TDD: the Red phase may use behavioral/component/a11y tests (RTL + vitest-axe) rather than visual tests (SKILL exception). All deps exact-pinned (supply-chain rule). IPC mocked via `@tauri-apps/api/mocks`.

---

## Phase 1: Setup

### T001 -- Scaffold `ui/` project + Tauri wiring
**Traces to:** FR-1, AC-1.1
**Status:** TODO
**Files:** `ui/package.json`, `ui/tsconfig.json`, `ui/vite.config.ts`, `ui/vitest.config.ts`, `ui/index.html`, `ui/src/main.tsx`, `crates/luminos-app/tauri.conf.json`

**TDD Cycle:** (setup)
1. **Green:** `pnpm init`; add exact-pinned React 19 / Vite 6 / TS 5 / Zustand / Zod / Vitest / RTL / vitest-axe / eslint-plugin-jsx-a11y / `@tauri-apps/api` (exact-pin a single version that is **≥ 2.7.0** — that floor is required for `@tauri-apps/api/mocks` event mocking used in T008; the pin itself MUST be exact per the supply-chain rule, not a `>=` range). Vite config (port 1420). `tauri.conf.json` beforeDev/beforeBuild/devUrl/frontendDist. Replace story-001 placeholder index with the real entry.
2. **Refactor:** Confirm no dep younger than 2 weeks; lockfile committed.

**Completion Notes:**
>

---

### T002 -- IPC wrappers + Zod types + defaults
**Traces to:** FR-2, NFR-2
**Status:** TODO
**Files:** `ui/src/ipc/{commands,events}.ts`, `ui/src/types/{settings,enums}.ts`, `ui/src/constants/defaults.ts`, `ui/src/test/setup.ts`

**TDD Cycle:**
1. **Red:** `commands re-export typed bindings` — type-check test that `getCurrentSettings` returns `Promise<AppSettings>` (compile-time; tsc in CI).
2. **Green:** Thin wrappers over generated `bindings.ts`; Zod schemas for non-generated shapes; `DEFAULT_SETTINGS`; vitest setup (axe + tauri mocks).
3. **Refactor:** `import type` for type-only imports.

**Completion Notes:**
>

---

## Phase 2: Core (store, gate, controls)

### T003 -- `useSettingsStore`
**Traces to:** FR-3, AC-1.2
**Status:** TODO
**Files:** `ui/src/hooks/useSettingsStore.ts`

**TDD Cycle:**
1. **Red:** `hydrate sets isHydrating false`; `setZoomLevel clamps 1.5-20`; `applyEngineUpdate replaces settings`.
2. **Green:** Zustand + immer store per DESIGN.
3. **Refactor:** —

**Completion Notes:**
>

---

### T004 -- `App` hydration + `HydrationGate` + `ToastProvider`
**Traces to:** FR-4, AC-1.2
**Status:** TODO
**Files:** `ui/src/App.tsx`, `ui/src/components/{HydrationGate,ToastProvider}.tsx`, `ui/src/hooks/useToast.ts`

**TDD Cycle:**
1. **Red:**
   - [ ] `hydration success populates store` (mocked resolve).
   - [ ] `hydration failure uses defaults + toast` (mocked reject).
   - [ ] `HydrationGate shows spinner then children`.
2. **Green:** Implement; toast uses `role="alert"`/`aria-live`.
3. **Refactor:** —

**Completion Notes:**
>

---

### T005 -- `Shell` + `Sidebar` (landmarks)
**Traces to:** FR-7, AC-3.1
**Status:** TODO
**Files:** `ui/src/components/{Shell,Sidebar}.tsx`, `ui/src/pages/MagnificationPage.tsx`

**TDD Cycle:**
1. **Red:** `Shell has nav + main landmarks`; `Sidebar Magnification active, others disabled`.
2. **Green:** Implement with `<nav>`/`<main>`, in-store active-page selector.
3. **Refactor:** —

**Completion Notes:**
>

---

### T006 -- `ZoomLevelSlider` (optimistic + revert + a11y)
**Traces to:** FR-5, FR-7, AC-2.1
**Status:** TODO
**Files:** `ui/src/components/magnification/ZoomLevelSlider.tsx`

**TDD Cycle:**
1. **Red:**
   - [ ] `change invokes setZoomLevel with value` (mocked).
   - [ ] `ipc error reverts and toasts` (mocked reject).
   - [ ] `has aria-labelledby/aria-valuetext/output aria-live, keyboard operable`.
2. **Green:** Implement per DESIGN.
3. **Refactor:** Extract a shared accessible-control pattern.

**Completion Notes:**
>

---

### T007 -- `MagnificationModeSelector` + `FrameTimingDisplay`
**Traces to:** FR-5, FR-6, AC-2.2
**Status:** TODO
**Files:** `ui/src/components/magnification/{MagnificationModeSelector,FrameTimingDisplay}.tsx`

**TDD Cycle:**
1. **Red:**
   - [ ] `select mode invokes setMagnificationMode + store updates`; Lens/Docked selectable placeholders.
   - [ ] `FrameTimingDisplay shows P99 from getFrameTimings` (mocked); `absent when not DEV`.
2. **Green:** Implement; gate FrameTimingDisplay on `import.meta.env.DEV`.
3. **Refactor:** —

**Checkpoint:** Controls drive IPC; store consistent; build passes.

**Completion Notes:**
>

---

## Phase 3: Integration (events, a11y)

### T008 -- Engine→panel event subscriptions
**Traces to:** FR-8, AC-3.1
**Status:** TODO
**Files:** `ui/src/App.tsx`, `ui/src/ipc/events.ts`

**TDD Cycle:**
1. **Red:** `zoom_changed emit updates store`; `mode_changed emit updates store`; `unsubscribes on unmount` (mocked listeners).
2. **Green:** Subscribe in `App` effect; return cleanup.
3. **Refactor:** —

**Completion Notes:**
>

---

### T009 -- Accessibility sweep (axe + jsx-a11y)
**Traces to:** FR-9, AC-3.1
**Status:** TODO
**Files:** `ui/src/**/*.test.tsx`, `ui/.eslintrc`

**TDD Cycle:**
1. **Red:** `vitest-axe` assertions on App/Shell/MagnificationPage/ZoomLevelSlider/ModeSelector/toast → **zero** violations.
2. **Green:** Fix any violations; ensure `eslint-plugin-jsx-a11y` clean; `prefers-reduced-motion` respected.
3. **Refactor:** —

**Checkpoint:** Zero axe violations; lint clean; events sync.

**Completion Notes:**
>

---

## Phase 4: Polish & Acceptance

### T010 -- Acceptance + AC matrix
**Traces to:** All ACs
**Status:** TODO
**Files:** story docs, `ui/**`

**Verification Checklist:**
- [ ] AC-1.1 scaffold + `pnpm build`→dist + bindings consumed + lint/test pass
- [ ] AC-1.2 hydration (success + failure)
- [ ] AC-2.1 zoom slider optimistic + revert + a11y
- [ ] AC-2.2 mode selector IPC + debug FrameTimingDisplay
- [ ] AC-3.1 event sync + zero axe violations (D8) + jsx-a11y clean
- [ ] No `any`; `import type` used; exact-pinned deps; no Python
- [ ] WCAG 2.1 AA spot-check (keyboard nav, reduced motion)

**Completion Notes:**
>

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| --- | --- | --- |
