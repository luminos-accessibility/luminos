/**
 * Minimal typing for Tauri's internal IPC bridge, used only in tests to spy on
 * `invoke` after `mockIPC` installs the mock transport. The real object is
 * created by `@tauri-apps/api/mocks`; we expose just the `invoke` member we
 * assert against so tests can `vi.spyOn(window.__TAURI_INTERNALS__, 'invoke')`.
 */
interface TauriInternals {
  invoke<T>(cmd: string, args?: unknown, options?: unknown): Promise<T>;
}

interface Window {
  __TAURI_INTERNALS__: TauriInternals;
}
