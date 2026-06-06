# Subtasks: Story E04/006 -- Frontend Control Panel UI

**Status:** DONE (Node-only; tauri-driver E2E + generated-bindings swap deferred — see T010)
**Started:** 2026-06-04
**Completed:** 2026-06-04
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup (scaffold) | 2 | 2 | 0 | 0 |
| 2. Core (store, gate, controls) | 5 | 5 | 0 | 0 |
| 3. Integration (events, a11y) | 2 | 2 | 0 | 0 |
| 4. Polish & Acceptance | 1 | 1 | 0 | 0 |
| **Total** | **10** | **10** | **0** | **0** |

**Node-only DoD (2026-06-04):** `pnpm install --frozen-lockfile` OK · `tsc --noEmit` clean · `pnpm lint`
(eslint + jsx-a11y) clean · `pnpm test` = **70 passed / 70** · coverage 98.13% stmts / 98.43% lines ·
`pnpm build` → `ui/dist` (JS 273.8 kB, **83.1 kB gzipped** — within NFR-5) · **0 axe-core violations**
across App/Shell/MagnificationPage/ZoomLevelSlider/MagnificationModeSelector/toast.
**Deferred (toolchain absent):** generated `bindings.ts` swap-in (story 005) and `tauri-driver` E2E (story 007).

> TS/React TDD: the Red phase may use behavioral/component/a11y tests (RTL + vitest-axe) rather than visual tests (SKILL exception). All deps exact-pinned (supply-chain rule). IPC mocked via `@tauri-apps/api/mocks`.

---

## Phase 1: Setup

### T001 -- Scaffold `ui/` project + Tauri wiring
**Traces to:** FR-1, AC-1.1
**Status:** DONE
**Files:** `ui/package.json`, `ui/tsconfig.json`, `ui/vite.config.ts`, `ui/vitest.config.ts`, `ui/eslint.config.js`, `ui/.prettierrc.json`, `ui/index.html`, `ui/src/main.tsx`, `ui/src/styles/global.css`, `ui/.gitignore`, `crates/luminos-app/tauri.conf.json`

**TDD Cycle:** (setup)
1. **Green:** `pnpm install` (pnpm 10.33.4 via corepack) with ALL deps exact-pinned per PINNED_VERSIONS §2 (React 19.2.6, Vite 6.4.2, TS 6.0.3, Zustand 5.0.13, Zod 4.4.3, Vitest 4.1.7, `@tauri-apps/api` 2.11.0 ≥2.7.0 for event mocking, jest-axe 10.0.0, eslint 9.39.4 + jsx-a11y 6.10.2). Vite fixed to port 1420 (strictPort). `tauri.conf.json` wires beforeDev/beforeBuild/devUrl/frontendDist `../../ui/dist`. Global CSS uses rem + prefers-reduced-motion/forced-colors.
2. **Refactor:** All pins published ≤2026-05-21 (≥2 weeks). `pnpm-lock.yaml` committed; `--frozen-lockfile` reproducible. Added `pnpm.onlyBuiltDependencies:["esbuild"]` so `vite build` gets the esbuild native binary.

**Completion Notes:**
> Scaffolded the whole Node toolchain. Deviations from spec text (all per PINNED_VERSIONS §4): TS 6 (not 5), Zod 4 (not 3), pnpm 10 (not 9). jest-axe chosen over vitest-axe per pins; needs a local `jest-axe.d.ts` + a vitest matcher augmentation (`src/test/vitest.d.ts`) since it ships no types. eslint-plugin-react-hooks v7's `recommended-latest` carries a legacy plugins ARRAY incompatible with flat config — registered the plugin object directly + spread only its rules. Added `globals` (16.5.0) to devDeps for the flat config. tauri.conf.json added but the Tauri CLI is NOT invoked (toolchain absent).

---

### T002 -- IPC wrappers + Zod types + defaults
**Traces to:** FR-2, NFR-2
**Status:** DONE
**Files:** `ui/src/ipc/{bindings,commands,events}.ts`, `ui/src/types/{settings,enums}.ts`, `ui/src/constants/defaults.ts`, `ui/src/test/setup.ts`, `__mocks__/zustand.ts`, plus tests `ui/src/types/{enums,settings}.schema.test.ts`, `ui/src/ipc/commands.test.ts`

**TDD Cycle:**
1. **Red:** Schema accept/reject tests (PascalCase enums, snake_case AppSettings keys, zoom-range bounds, camelCase FrameTimingSummary) + IPC seam tests (correct snake_case command names, arg shapes, Result-unwrap, Zod parse) — failed before impl.
2. **Green:** Hand-authored PLACEHOLDER `bindings.ts` mirroring tauri-specta v2 default `Result` mode shape (`commands.<camel>()→Promise<Result<T,string>>`, `events.<camel>.listen(cb)`); thin `commands.ts`/`events.ts` wrappers that unwrap the Result (throw `IpcCommandError`) + Zod-parse payloads; Zod schemas; `DEFAULT_SETTINGS` mirroring Rust `AppSettings::default()`; vitest setup (jest-axe matcher + clearMocks).
3. **Refactor:** `import type` throughout; documented the snake_case-vs-camelCase asymmetry inline.

**Completion Notes:**
> Verified the wire format against the REAL Rust code (not memory): `AppSettings` + sub-structs in `luminos-core::config::schema` have NO `rename_all` → snake_case keys; enum variants are bare PascalCase strings (confirmed in `luminos-types` serde roundtrip tests); `FrameTimingSummary` is the lone camelCase type (story 005 adds the rename, DC-5). Used `z.partialRecord` (Zod 4) for the sparse keybindings HashMap. The seam is structured so swapping story-005's generated `bindings.ts` is a one-file change; everything downstream imports `./commands`/`./events`. 23 tests green.

---

## Phase 2: Core (store, gate, controls)

### T003 -- `useSettingsStore`
**Traces to:** FR-3, AC-1.2
**Status:** DONE
**Files:** `ui/src/hooks/useSettingsStore.ts`, `ui/src/hooks/useSettingsStore.test.ts`

**TDD Cycle:**
1. **Red:** `hydrate sets isHydrating false`; `hydrate populates settings`; `setZoomLevel clamps to min/max`; in-range passthrough; `setMode updates`; `applyEngineUpdate replaces`; immutability (new object reference). 9 tests, failed pre-impl.
2. **Green:** Zustand v5 store with `settings`/`isHydrating` + actions; `clampZoom` to [1.5,20].
3. **Refactor:** Plain immutable spreads instead of immer middleware.

**Completion Notes:**
> **Deviation from DESIGN.md:** DESIGN specified Zustand+immer, but `immer` is NOT in PINNED_VERSIONS §2 and is a required peer of `zustand/middleware/immer`. Adding an unlisted dependency would violate the exact-pin/supply-chain mandate. Since the mutated shape (`settings.magnification`) is shallow, plain immutable spreads are equally clear with zero added deps (also helps NFR-5 bundle). Recorded in Deviations table. 9 tests green. Auto-reset between tests via the official `__mocks__/zustand.ts` recipe.

---

### T004 -- `App` hydration + `HydrationGate` + `ToastProvider`
**Traces to:** FR-4, AC-1.2
**Status:** DONE
**Files:** `ui/src/App.tsx`, `ui/src/components/HydrationGate.tsx`, `ui/src/hooks/useToast.tsx`, tests `ui/src/App.test.tsx`, `ui/src/components/HydrationGate.test.tsx`, `ui/src/hooks/useToast.test.tsx`

**TDD Cycle:**
1. **Red (all failed pre-impl):**
   - [x] `hydration success populates store` (mocked resolve).
   - [x] `hydration failure uses defaults + toast` (mocked reject).
   - [x] `HydrationGate shows loading status then children`.
   - [x] toast variants (error→alert, info→status) + dismiss + throws-outside-provider.
2. **Green:** `ToastProvider` (live region, error=`role="alert"`, info=`role="status"`); `HydrationGate` gates on `isHydrating` (`role="status"` loader); `App` hydrates on mount, falls back to `DEFAULT_SETTINGS`+toast on reject.
3. **Refactor:** Split toast context so ACTIONS are stable (memo on stable callbacks, separate from the toasts array).

**Completion Notes:**
> Bug found & fixed via the failing axe/hydration tests: the hydrate `useEffect` originally depended on the whole `toast` object, which changed identity whenever a toast was pushed → hydrate→toast→re-render→hydrate infinite loop (surfaced as "multiple alert elements"). Fixed by making `useToast()` return only stable action references. 70-test suite green. IPC mocked at the wrapper module (`vi.mock('./ipc/commands')`), no Tauri runtime.

---

### T005 -- `Shell` + `Sidebar` (landmarks)
**Traces to:** FR-7, AC-3.1
**Status:** DONE
**Files:** `ui/src/components/{Shell,Sidebar}.tsx`, `ui/src/hooks/useNavigationStore.ts`, `ui/src/pages/MagnificationPage.tsx`, tests `ui/src/components/Shell.test.tsx`, `ui/src/hooks/useNavigationStore.test.ts`

**TDD Cycle:**
1. **Red:** `Shell exposes navigation + main landmarks`; `Sidebar marks Magnification aria-current=page`; `disables Display/Speech/Keybindings/Diagnostics`; outlet renders the page. Navigation store: default + ignores-disabled. Failed pre-impl.
2. **Green:** `<nav aria-label>` + `<main>`; `useNavigationStore` lightweight in-store active-page selector (no router dep, per resolved open question); `renderPage` switch.
3. **Refactor:** Extracted page IDs/labels/enabled-set to the nav store.

**Completion Notes:**
> Used a tiny Zustand `useNavigationStore` instead of a router (Phase 0 has one page). `setActivePage` no-ops for disabled pages so the UI can't navigate to unimplemented sections. Landmarks verified by role queries (which double as a11y checks). 7 tests green.

---

### T006 -- `ZoomLevelSlider` (optimistic + revert + a11y)
**Traces to:** FR-5, FR-7, AC-2.1
**Status:** DONE
**Files:** `ui/src/components/magnification/ZoomLevelSlider.tsx`, test `ui/src/components/magnification/ZoomLevelSlider.test.tsx`

**TDD Cycle:**
1. **Red (failed pre-impl):**
   - [x] `renders current zoom level`.
   - [x] `change invokes setZoomLevel with value` (mocked).
   - [x] `ipc error reverts and toasts` (mocked reject).
   - [x] `has aria-labelledby/aria-valuetext/output aria-live`, focusable native range.
2. **Green:** Native `<input type=range>` (min 1.5 / max 20 / step 0.5), optimistic `store.setZoomLevel`, `setZoomLevel(v)` IPC, revert to prior value + error toast on reject. `aria-valuetext="{n}x"`, polite `<output>`.
3. **Refactor:** `useId` for the label association; `formatZoom` helper.

**Completion Notes:**
> jsdom does not move a range input's value on arrow keys, so the value-change tests use `fireEvent.change` (the documented RTL exception); keyboard-operability is asserted separately via role+focus. Native range gives full keyboard support for free. 4 tests green.

---

### T007 -- `MagnificationModeSelector` + `FrameTimingDisplay`
**Traces to:** FR-5, FR-6, AC-2.2
**Status:** DONE
**Files:** `ui/src/components/magnification/{MagnificationModeSelector,FrameTimingDisplay}.tsx`, tests `*.test.tsx`

**TDD Cycle:**
1. **Red (failed pre-impl):**
   - [x] `renders all modes as a radiogroup`; FullScreen checked; Lens/Docked selectable placeholders.
   - [x] `select Lens invokes setMagnificationMode + store updates`.
   - [x] `ipc error reverts + toasts`.
   - [x] `FrameTimingDisplay shows P99 from getFrameTimings` (DEV); `absent when not DEV`.
2. **Green:** Mode selector = native radio group (arrow-key nav for free), optimistic + revert + toast. `FrameTimingDisplay` gated on `import.meta.env.DEV`, polls `getFrameTimings` every 1s, shows P99/avg/target.
3. **Refactor:** Read `import.meta.env.DEV` once into `isDev` so render+effect agree; early-return null in prod.

**Checkpoint:** Controls drive IPC; store consistent; `pnpm build` passes (273.8 kB / 83.1 kB gz).

**Completion Notes:**
> Set `clearMocks:true`+`unstubEnvs:true` in vitest config so the timing-display poll's mock-call history doesn't leak into the "absent when not DEV" test. Both controls revert the store to the prior value and raise an accessible error toast on IPC reject. 6 tests green.

---

## Phase 3: Integration (events, a11y)

### T008 -- Engine→panel event subscriptions
**Traces to:** FR-8, AC-3.1
**Status:** DONE
**Files:** `ui/src/App.tsx`, `ui/src/ipc/events.ts`, tests `ui/src/App.events.test.tsx`, `ui/src/ipc/events.test.ts`

**TDD Cycle:**
1. **Red (failed pre-impl):** `app subscribes on mount`; `zoom_changed updates store`; `mode_changed updates store`; `unsubscribes on unmount`; wrapper-level event tests via `mockIPC(..., {shouldMockEvents:true})` + raw `emit`.
2. **Green:** `App` effect calls `onZoomChanged`/`onModeChanged` → store `setZoomLevel`/`setMode`; collects the async unlisten fns and runs them on cleanup.
3. **Refactor:** —

**Completion Notes:**
> Event wrappers use `@tauri-apps/api/event listen`; verified end-to-end with `shouldMockEvents:true` (requires `@tauri-apps/api` ≥2.7.0 — pinned 2.11.0). Cleanup handles the case where the component unmounts before the unlisten promise resolves. 7 tests green (4 App.events + 3 events wrapper).

---

### T009 -- Accessibility sweep (axe + jsx-a11y)
**Traces to:** FR-9, AC-3.1
**Status:** DONE
**Files:** `ui/src/accessibility.test.tsx`, `ui/eslint.config.js`, `ui/src/styles/global.css`

**TDD Cycle:**
1. **Red:** `jest-axe` `toHaveNoViolations()` on App/Shell/MagnificationPage/ZoomLevelSlider/MagnificationModeSelector/error-toast. Initially RED — real violation found.
2. **Green:** Fixed the toast region (`<div aria-label>` with no role → `aria-prohibited-attr`) by adding `role="region"`. `eslint-plugin-jsx-a11y` (recommended) clean. `prefers-reduced-motion` handled in `global.css`.
3. **Refactor:** —

**Checkpoint:** **0 axe violations** across all 6 scopes; lint clean; events sync.

**Completion Notes:**
> The a11y gate did its job: the first sweep caught the unlabeled-region violation, fixed at source. jest-axe (not vitest-axe) per pins, wired via a local matcher augmentation. jsdom (not happy-dom) required for axe. 6 axe tests green; total suite 70/70.

---

## Phase 4: Polish & Acceptance

### T010 -- Acceptance + AC matrix
**Traces to:** All ACs
**Status:** DONE
**Files:** story docs, `ui/**`

**Verification Checklist:**
- [x] AC-1.1 scaffold + `pnpm build`→`ui/dist` + bindings seam consumed + lint/test pass
- [x] AC-1.2 hydration (success populates store; failure → defaults + accessible toast)
- [x] AC-2.1 zoom slider optimistic + revert + a11y attrs (aria-labelledby/valuetext/output live)
- [x] AC-2.2 mode selector IPC + store; debug-only FrameTimingDisplay shows P99
- [x] AC-3.1 event sync (zoom_changed/mode_changed) + **0 axe violations** (D8) + jsx-a11y clean
- [x] No `any` (eslint `no-explicit-any: error`); `import type` enforced; **zero type assertions** (the lone `error as string` in `bindings.ts` was replaced with `String(error)` post-review); exact-pinned deps; no Python
- [x] WCAG 2.1 AA spot-check: keyboard-operable native controls, reduced-motion CSS, `prefers-contrast: more` border-strengthening, forced-colors via system color tokens, rem sizing

**AC → test coverage matrix:**

| AC | Covering tests (file) |
|----|-----------------------|
| AC-1.1 | DoD: `pnpm build`→dist, `tsc`/`lint`/`test` green; seam: `ipc/commands.test.ts`, `ipc/events.test.ts` |
| AC-1.2 | `App.test.tsx` (success populates / failure→defaults+toast), `HydrationGate.test.tsx` (loading→children) |
| AC-2.1 | `ZoomLevelSlider.test.tsx` (renders, change→setZoomLevel, revert+toast, a11y attrs) |
| AC-2.2 | `MagnificationModeSelector.test.tsx` (radiogroup, select→IPC+store, revert), `FrameTimingDisplay.test.tsx` (P99 in DEV, absent in prod) |
| AC-3.1 | `App.events.test.tsx` (subscribe, zoom/mode events update store, unsubscribe), `accessibility.test.tsx` (0 axe across 6 scopes), `pnpm lint` jsx-a11y clean |

**Completion Notes:**
> All 5 ACs covered by ≥1 passing test (70 total). Build/lint/typecheck all clean. Deferred to story 005: replace placeholder `bindings.ts` with the tauri-specta-generated file (one-file swap). Deferred to story 007: `tauri-driver` E2E. Toolchain (Tauri/webkit) absent on this machine, so NO `tauri`/`tauri-driver` commands were run; everything verified is Node-only.
>
> **Post-review follow-up (2026-06-05, all 4 gates had APPROVED/PASS):** applied 4 Minor fixes + 1 doc.
> M1 — `bindings.ts` `error as string` → `String(error)` (removes the codebase's only type assertion; correct for non-string rejections). M2 — corrected a stale "immer produces a new object" comment in `useSettingsStore.test.ts` to "plain spreads…". M3 — made the `global.css` header docstring honest and **added a real `@media (prefers-contrast: more)` block** (strengthens focus ring + sidebar/active/toast borders); documented that forced-colors is handled implicitly via system color tokens (Canvas/CanvasText/Highlight/GrayText). M4 — disabled sidebar buttons now use `color: GrayText` instead of `opacity: 0.6` so the disabled state survives forced-colors. DOC — added `globals 16.5.0` to PINNED_VERSIONS §2. 70/70 still green; tsc/lint clean.

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| T003 | No immer middleware (DESIGN specified Zustand+immer) — plain immutable spreads instead | `immer` is not in PINNED_VERSIONS §2 and is a required peer of `zustand/middleware/immer`; adding an unlisted dep violates the exact-pin/supply-chain rule. Mutated shape is shallow, so spreads are equally clear with zero added deps (helps NFR-5). |
| T001 | Versions per PINNED_VERSIONS §4 (TS 6 not 5, Zod 4 not 3, pnpm 10 not 9) | PINNED_VERSIONS.md §4 is authoritative ("pin latest safe", user decision) and supersedes spec text. |
| T009 | jest-axe (not vitest-axe, which DESIGN/skill mention) | PINNED_VERSIONS §2 selects jest-axe (vitest-axe abandoned). Wired via a local matcher augmentation. |
| T002 | `bindings.ts` is a hand-authored PLACEHOLDER (story 005 not yet run) | Story 005's tauri-specta generation does not exist; placeholder mirrors the exact generated shape so swap-in is a one-file change. |
| T007 | `FrameTimingDisplay` uses interval polling (no engine event for timings) | Matches the resolved open question; timings have no push channel in Phase 0. |
