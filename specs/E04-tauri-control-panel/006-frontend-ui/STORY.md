# Story E04/006: Frontend Control Panel UI

**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** APPROVED (approved-as-authoritative per E04 execution, 2026-06-04)
**Depends On:** 005 (`ui/src/ipc/bindings.ts` + commands/events)

---

## Problem Statement

The engine is fully drivable over typed IPC (story 005), but there is no UI: the control-panel window (story 001) loads a placeholder. This story builds the **React control panel** — the `ui/` project (pnpm, Vite, React, TypeScript, Zustand, Zod, Vitest, RTL, axe-core) and the Phase-0 component tree: `App` → `HydrationGate` → `Shell` (sidebar + outlet) → `MagnificationPage` with a `ZoomLevelSlider`, a `MagnificationModeSelector`, and a debug-only `FrameTimingDisplay`. The store hydrates from `get_current_settings` on startup and subscribes to `zoom_changed`/`mode_changed` so hotkey-originated changes keep the UI in sync. Controls use optimistic updates with revert-on-error, accessible markup, and zero `axe-core` violations.

This is the user-facing payoff of E04: a low-vision user drags a slider and watches magnification change in real time, in an accessible panel.

## User Scenarios

> **AC count = 5** (grouped per epic plan).

### US-1: The panel scaffolds, builds, and hydrates
As a user, I want the control panel to open already showing my current settings, so that it reflects the live engine state.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1 (scaffold + build + bindings):** Given the `ui/` project (pnpm, Vite 6, React 19, TypeScript 5, Zustand, Zod, Vitest, RTL, axe-core), when `pnpm build` runs, then it produces `ui/dist` (wired to `tauri.conf.json` `frontendDist: "../ui/dist"`) and the app consumes the generated `ui/src/ipc/bindings.ts` (typed `commands`/`events`); `pnpm lint` (incl. `eslint-plugin-jsx-a11y`) and `pnpm test` (Vitest) pass. *(FR-1, FR-2)*
- **AC-1.2 (hydration):** Given the panel mounts, when it renders, then `HydrationGate` shows a loading state until `getCurrentSettings()` resolves and populates `useSettingsStore`; on IPC failure, it hydrates with `DEFAULT_SETTINGS` and shows an accessible error toast (no crash, no blank screen). *(FR-3, FR-4)*

### US-2: Controls drive the engine
As a low-vision user, I want a zoom slider and a mode selector that change magnification immediately, so that I can configure my magnifier.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1 (zoom slider round-trip):** Given the panel hydrated, when the user changes `ZoomLevelSlider`, then the store updates optimistically, `setZoomLevel(level)` is invoked, and on IPC error the value reverts and an error toast appears; the slider is accessible (`aria-labelledby`, `aria-valuetext="{n}x"`, an `output` with `aria-live="polite"`, keyboard operable, range 1.5–20 step 0.5). *(FR-5, FR-7)*
- **AC-2.2 (mode selector + frame timing):** Given the panel, when the user selects a mode in `MagnificationModeSelector`, then `setMagnificationMode(mode)` is invoked and the store updates (FullScreen active; Lens/Docked shown as selectable placeholders for E5); and in debug builds, `FrameTimingDisplay` shows the current P99 from `getFrameTimings()` (polled or event-less refresh). *(FR-5, FR-6)*

### US-3: Sync with hotkeys + accessibility
As a user who also uses keyboard shortcuts, I want the panel to reflect hotkey changes and be fully accessible, so that the UI and engine never diverge and I can use it with a screen reader.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1 (events + a11y):** Given the panel subscribes to `zoom_changed`/`mode_changed`, when a hotkey changes zoom/mode in the engine (story 003), then the store updates and the controls reflect it; and every Phase-0 component passes `axe-core` with **zero** violations and `eslint-plugin-jsx-a11y` is clean (keyboard-navigable, labeled, ARIA landmarks on `Shell`/`Sidebar`). *(FR-8, FR-9)* — **D8**

## Functional Requirements

- **FR-1:** Scaffold `ui/` with pnpm 9 / Vite 6 / React 19 / TS 5 / Zustand / Zod / Vitest / RTL / axe-core / eslint-plugin-jsx-a11y; `tauri.conf.json` `beforeDevCommand: "pnpm dev"`, `devUrl: http://localhost:1420`, `beforeBuildCommand: "pnpm build"`, `frontendDist: "../ui/dist"`. *(AC-1.1)*
- **FR-2:** Consume the generated `ui/src/ipc/bindings.ts` for all command/event calls (no hand-written invoke strings); wrap in `ui/src/ipc/commands.ts`/`events.ts` if ergonomic. *(AC-1.1)*
- **FR-3:** `useSettingsStore` (Zustand) holds `settings: AppSettings` + `isHydrating: boolean` with actions `hydrate`/`applyEngineUpdate`/`setZoomLevel`/`setMode`. *(AC-1.2)*
- **FR-4:** `App` hydrates on mount via `getCurrentSettings()`; on error → `DEFAULT_SETTINGS` + accessible toast. *(AC-1.2)*
- **FR-5:** `ZoomLevelSlider`/`MagnificationModeSelector` perform optimistic updates, invoke the IPC command, and revert + toast on error. *(AC-2.1, AC-2.2)*
- **FR-6:** `FrameTimingDisplay` (debug builds only) shows P99 from `getFrameTimings()`. *(AC-2.2)*
- **FR-7:** Controls MUST be accessible (labels, `aria-*`, keyboard). *(AC-2.1, AC-3.1)*
- **FR-8:** `App` subscribes to `zoom_changed`/`mode_changed` and updates the store; unsubscribes on unmount. *(AC-3.1)*
- **FR-9:** All Phase-0 components MUST pass `axe-core` (zero violations) and `eslint-plugin-jsx-a11y`. *(AC-3.1)* — **D8**

## Non-Functional Requirements

- **NFR-1:** TypeScript-only tooling (no Python, per CLAUDE.md); any scripts run via `npx tsx`.
- **NFR-2:** Strong typing; no `any`; prefer Zod schemas + inferred types for non-generated shapes. `import type` for type-only imports.
- **NFR-3:** Dependencies pinned to exact versions (supply-chain rule); no auto-upgrades; no version younger than 2 weeks.
- **NFR-4:** WCAG 2.1 AA for the panel (doc-06 / RISK-023): keyboard operable, screen-reader labeled, respects reduced-motion.
- **NFR-5:** Bundle stays modest (doc-08: ~200–500KB gzipped) — no heavy UI frameworks beyond React.

## Out of Scope

- Display/Speech/Keybindings/Profiles/Diagnostics-chart pages → later epics (sidebar may show them disabled/hidden).
- `TrackingModeSelector`, lens/docked controls → Epic 5 (mode selector lists them but they're placeholders).
- `tauri-driver` end-to-end tests → story 007 (this story uses Vitest + RTL + mocked IPC via `@tauri-apps/api/mocks`).
- Theming/visual polish beyond accessible defaults.

## Open Questions

- [x] Routing library? — **Resolved:** minimal — a simple state-driven outlet (no router dep needed for one page) or `react-router` if the sidebar warrants it; default to a lightweight in-store active-page selector to avoid a dependency for Phase 0. Revisit when more pages land (E5+).
- [x] How are generated bindings consumed? — **Resolved:** `tauri-specta` emits typed `commands.*`/`events.*` in `bindings.ts`; thin wrappers in `ipc/commands.ts`/`events.ts` re-export for ergonomics + testability.
- [x] How is `import.meta.env.DEV`/debug gating done for `FrameTimingDisplay`? — **Resolved:** Vite `import.meta.env.DEV` gates the debug-only component (matches the Rust `#[cfg(debug_assertions)]` boundary conceptually).
- [x] Reduced motion? — **Resolved:** respect `prefers-reduced-motion` for any transition (NFR-4); avoid non-essential animation.
