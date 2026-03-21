# TypeScript Expert Memory — Luminos Project

## Project Overview
- Luminos: GPLv3 cross-platform screen magnification + TTS accessibility suite
- Pre-development phase: no app code yet, only specs in `specs/`
- Spec documents: `specs/tech-strategy/01–05-*.md` (architecture through control panel)

## TypeScript Stack
- **Framework:** Tauri 2.0 (control panel webview only — NOT the magnification overlay)
- **UI:** React 19 + React Router 7 (hash router)
- **State:** Zustand v5 with immer middleware
- **Validation:** Zod v3 (schemas first, `z.infer<>` for types)
- **Type safety across IPC:** `tauri-specta` v2 generates `ui/src/ipc/bindings.ts` from Rust types
- **Build:** Vite + `@vitejs/plugin-react`
- **Testing:** Vitest + React Testing Library + `@tauri-apps/api/mocks` + axe-core

## Key Conventions
- Enum variants: **PascalCase** on both Rust and TS sides (`'FullScreen'`, not `'full_screen'`)
- Serde: `rename_all = "PascalCase"` on Rust enums for wire format consistency
- IPC commands: `Result<T, String>` return type (structured errors are Phase 2)
- Optimistic updates: update Zustand store immediately, call IPC, revert + toast on error
- Sliders debounce IPC calls 150ms (use `use-debounce`) to avoid lock contention
- All IPC responses parsed through Zod schemas in command wrappers

## File Locations
- TS source: `ui/src/`
- IPC bindings (auto-generated): `ui/src/ipc/bindings.ts`
- Typed IPC wrappers: `ui/src/ipc/commands.ts`, `ui/src/ipc/events.ts`
- Zod schemas: `ui/src/types/{enums,settings,tts,diagnostics,profiles}.ts`
- Zustand stores: `ui/src/hooks/use{Settings,Tts,Profiles}Store.ts`
- Rust IPC handlers: `crates/luminos-app/src/tauri_commands.rs`
- Shared Rust handle: `LuminosHandle` in `tauri_commands.rs`

## IPC Architecture
- Commands: Panel→Engine via `invoke()`; Events: Engine→Panel via `emit()`/`listen()`
- Commands run on Tauri async tokio runtime — never block render thread
- State mutations: write to `ArcSwap<AppState>`, then send `EventLoopProxy` event to wake winit
- Hotkey changes flow back to panel via emitted events (`zoom_changed`, `settings_changed`)

## Accessibility (Non-Negotiable)
- Control panel must meet WCAG 2.1 AA (users are low-vision — inaccessible panel = critical bug)
- All colors + CSS sizes in `rem`; forced-colors + prefers-contrast + prefers-reduced-motion handled
- ARIA live regions on all dynamic status (TTS state, zoom level, model loading)

## TypeScript TDD Skill
- Skill created: `.claude/skills/typescript-test-driven-development/`
- SKILL.md (~520 lines) + references/test-patterns.md (~640 lines)
- Mirrors the Rust TDD skill structure, adapted for TS/React/Vitest
- Key patterns: Zod schema accept/reject tests, Zustand store isolation via getState/setState, mockIPC from @tauri-apps/api/mocks, vitest-axe for a11y, userEvent over fireEvent
- Tauri IPC event mocking requires `shouldMockEvents: true` (Tauri >= 2.7.0)
- Zustand auto-reset between tests via official `__mocks__/zustand.ts` pattern
- jsdom required (not happy-dom) for axe-core compatibility
- Test naming: `{module}_{behavior}_{condition}` with underscores (matches Rust convention)
- Eval results: 100% with-skill vs 95.8% baseline, 22% fewer tokens, 14% faster

## Spec Document Style
- Strategy docs live in `specs/tech-strategy/NNN-name.md`
- Section cross-references use `[Document](./path.md)` with precise section numbers
- Phase attribution tables at start of each document
- Rust code examples use `pub(crate)` visibility consistently
- TS code examples use Zod schemas before TypeScript types
