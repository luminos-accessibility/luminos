# Design: Story E04/006 -- Frontend Control Panel UI

**Story:** [STORY.md](./STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** APPROVED (approved-as-authoritative per E04 execution, 2026-06-04)
**Author:** principal-architect
**Risk Refs:** RISK-020 (webview surface — React auto-escaping + no external content), RISK-023 (self-accessibility / WCAG 2.1 AA)

---

## Overview

Scaffold `ui/` and build the Phase-0 React control panel against the generated `bindings.ts` (story 005). Zustand store, hydration gate, accessible zoom slider + mode selector, debug frame-timing readout, and engine→panel event subscriptions. TypeScript-only, exact-pinned deps, WCAG 2.1 AA, zero `axe-core` violations.

## Architecture

### Project layout
```
ui/
  package.json  tsconfig.json  vite.config.ts  vitest.config.ts  .eslintrc (jsx-a11y)
  index.html
  src/
    main.tsx  App.tsx
    components/{HydrationGate,Shell,Sidebar,ToastProvider}.tsx
    pages/MagnificationPage.tsx
    components/magnification/{ZoomLevelSlider,MagnificationModeSelector,FrameTimingDisplay}.tsx
    hooks/{useSettingsStore,useToast}.ts
    ipc/{bindings.ts (generated), commands.ts, events.ts}
    types/{settings.ts (zod), enums.ts}
    constants/defaults.ts
    test/setup.ts  (vitest-axe, @tauri-apps/api/mocks)
```

### Component hierarchy (Phase 0)
```
App (hydrate on mount; subscribe events)
 └─ ToastProvider (aria-live region)
     └─ HydrationGate (spinner until !isHydrating)
         └─ Shell (role=application; <nav> Sidebar + <main> outlet)
             ├─ Sidebar (nav landmark; Magnification active; others disabled)
             └─ MagnificationPage
                 ├─ ZoomLevelSlider
                 ├─ MagnificationModeSelector
                 └─ FrameTimingDisplay (import.meta.env.DEV only)
```

### Data flow
- **Hydrate:** `App` mount → `commands.getCurrentSettings()` → `useSettingsStore.hydrate(settings)`; on reject → `hydrate(DEFAULT_SETTINGS)` + `toast.error`.
- **UI → engine:** slider `onChange` → optimistic `store.setZoomLevel(v)` → `commands.setZoomLevel(v)`; on reject → revert + toast.
- **Engine → UI:** `events.zoomChanged.listen(v => store.setZoomLevel(v))`, `events.modeChanged.listen(m => store.setMode(m))`; unsubscribe on unmount.

## API Design (key types/contracts)

```typescript
// ipc/commands.ts — thin wrappers over generated bindings
import { commands } from './bindings';
export const getCurrentSettings = () => commands.getCurrentSettings();
export const setZoomLevel = (level: number) => commands.setZoomLevel(level);
export const setMagnificationMode = (mode: MagnificationMode) => commands.setMagnificationMode(mode);
export const getFrameTimings = () => commands.getFrameTimings();
// ipc/events.ts
import { events } from './bindings';
export const onZoomChanged = (cb: (v: number) => void) => events.zoomChangedEvent.listen(e => cb(e.payload));
export const onModeChanged = (cb: (m: MagnificationMode) => void) => events.modeChangedEvent.listen(e => cb(e.payload));

// hooks/useSettingsStore.ts
interface SettingsState {
  readonly settings: AppSettings;
  readonly isHydrating: boolean;
  hydrate(s: AppSettings): void;
  applyEngineUpdate(s: AppSettings): void;
  setZoomLevel(level: number): void;     // local optimistic (clamps 1.5–20)
  setMode(mode: MagnificationMode): void;
}
export const useSettingsStore = create<SettingsState>()(immer(/* ... */));
```

```tsx
// ZoomLevelSlider (accessibility-critical)
<label className="control">
  <span id="zoom-label">Zoom level</span>
  <input type="range" aria-labelledby="zoom-label" aria-valuetext={`${zoom.toFixed(1)}x`}
         min={1.5} max={20} step={0.5} value={zoom}
         onChange={e => handleChange(parseFloat(e.target.value))} />
  <output aria-live="polite">{zoom.toFixed(1)}x</output>
</label>
// handleChange: optimistic store.setZoomLevel(v); try setZoomLevel(v); catch → revert + toast.
```

## Error Handling
- IPC rejections caught at the call site → revert optimistic update + accessible toast (`role="alert"` / `aria-live`).
- Hydration failure → defaults + toast, never blank.
- Type-only imports use `import type`; no `any`; Zod parses any non-generated external shape.

## Platform Considerations
- Webview is cross-platform (Tauri). No platform branches in the UI. Reduced-motion via CSS media query (all platforms).

## Testing Strategy

### Unit / component (Vitest + RTL + `@tauri-apps/api/mocks` for IPC)
- `useSettingsStore`: hydrate sets `isHydrating=false`; `setZoomLevel` clamps; `applyEngineUpdate` replaces.
- `HydrationGate`: shows spinner while hydrating; renders children after (AC-1.2).
- `App` hydration: mocked `getCurrentSettings` resolve → store populated; reject → defaults + toast (AC-1.2).
- `ZoomLevelSlider`: change → `setZoomLevel` called with value; mocked reject → reverts + toast; a11y attrs present (AC-2.1).
- `MagnificationModeSelector`: select → `setMagnificationMode` called; store updates (AC-2.2).
- `FrameTimingDisplay`: renders P99 from mocked `getFrameTimings`; absent in prod build (AC-2.2).
- Event subscriptions: mocked `zoom_changed` emit → store updates (AC-3.1).

### Accessibility
- `vitest-axe`: assert **zero** violations for `App`, `Shell`, `MagnificationPage`, `ZoomLevelSlider`, `MagnificationModeSelector`, toasts (AC-3.1, D8).
- `eslint-plugin-jsx-a11y`: clean in `pnpm lint` (AC-3.1).

### Build
- `pnpm build` produces `ui/dist` (AC-1.1); `bindings.ts` import resolves (type-checks).

### Acceptance Tests

| AC | Test Type | Verification |
|----|-----------|--------------|
| AC-1.1 | Build + lint + test | `pnpm build`→dist; bindings consumed; lint/test pass. |
| AC-1.2 | Component (RTL) | HydrationGate gating; hydrate success + failure paths. |
| AC-2.1 | Component (RTL) | Slider optimistic + revert + a11y attrs. |
| AC-2.2 | Component (RTL) | Mode selector IPC + store; debug FrameTimingDisplay. |
| AC-3.1 | Component + axe | Event subscriptions update store; zero axe violations; jsx-a11y clean. |

## Performance Targets
- Bundle ~200–500KB gzipped (NFR-5). Hydration render fast (human-speed UI).

## Security Considerations
- React auto-escaping; no `dangerouslySetInnerHTML`; no external content/CDN; default Tauri CSP (RISK-020). All data from typed IPC.

## Alternatives Considered
1. **Redux/MobX instead of Zustand.** Rejected — doc-05 specifies Zustand; lighter for this scope.
2. **`react-router` for one page.** Deferred — in-store active-page selector avoids a dep until E5 adds pages.
3. **Hand-written IPC types.** Rejected — use generated `bindings.ts` (story 005) for lock-step typing; manual types only as fallback.
4. **CSS-in-JS lib.** Rejected — plain CSS/modules keep the bundle small (NFR-5) and avoid a dep.
