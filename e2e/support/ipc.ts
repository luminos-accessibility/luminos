/**
 * IPC round-trip helpers for the E2E suite.
 *
 * The suite drives the React UI (slider/radios) and then asserts the REAL
 * engine state — not just the React store — by invoking the Phase-0 read
 * command `get_current_settings` directly in the webview. This proves the IPC
 * contract end-to-end (UI -> command -> StateManager -> ArcSwap), which is the
 * whole point of the `tauri-driver` job (DESIGN "Alternatives Considered" #4).
 *
 * `window.__TAURI_INTERNALS__.invoke` is the low-level entry that
 * `@tauri-apps/api/core`'s `invoke` calls; a Rust command returning
 * `Result::Ok(T)` resolves the promise with `T` directly (tauri-specta's
 * TS-layer `{ status }` envelope is applied above this layer, so the raw value
 * is `T`).
 */

/** The two enum types the engine round-trips that the E2E asserts against. */
export type MagnificationMode = 'FullScreen' | 'Docked' | 'Lens';

/** Subset of `AppSettings.magnification` the E2E reads back. */
export interface MagnificationSettings {
  readonly zoom_level: number;
  readonly mode: MagnificationMode;
}

/** Subset of `AppSettings` the E2E reads back via `get_current_settings`. */
export interface EngineSettings {
  readonly magnification: MagnificationSettings;
}

/** Shape of `FrameTimingSummary` (camelCase per DC-5) read via the command. */
export interface FrameTimingSummary {
  readonly averageMs: number;
  readonly p99Ms: number;
  readonly minMs: number;
  readonly maxMs: number;
  readonly targetFps: number;
}

/**
 * Invokes a Phase-0 read command in the webview and returns its resolved value.
 *
 * Runs inside `browser.execute`, so the body executes in the webview context
 * where `window.__TAURI_INTERNALS__` exists. `args` are forwarded to the
 * command (e.g. `{ level }` / `{ mode }`), matching the Rust param names.
 */
async function invokeInWebview<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return browser.execute(
    (cmd: string, cmdArgs: Record<string, unknown> | undefined) => {
      const internals = (
        window as unknown as {
          __TAURI_INTERNALS__?: {
            invoke: (c: string, a?: Record<string, unknown>) => Promise<unknown>;
          };
        }
      ).__TAURI_INTERNALS__;
      if (internals === undefined) {
        throw new Error('Tauri internals not present in the webview');
      }
      return internals.invoke(cmd, cmdArgs) as Promise<T>;
    },
    command,
    args,
  );
}

/** Reads the engine's current settings (D2/D3 assertion source of truth). */
export async function getCurrentSettings(): Promise<EngineSettings> {
  return invokeInWebview<EngineSettings>('get_current_settings');
}

/** Reads the engine's current frame-timing summary (D4 assertion source). */
export async function getFrameTimings(): Promise<FrameTimingSummary> {
  return invokeInWebview<FrameTimingSummary>('get_frame_timings');
}

/** Reads the engine's current zoom level. */
export async function getEngineZoom(): Promise<number> {
  return (await getCurrentSettings()).magnification.zoom_level;
}

/** Reads the engine's current magnification mode. */
export async function getEngineMode(): Promise<MagnificationMode> {
  return (await getCurrentSettings()).magnification.mode;
}

/**
 * Switches the WebDriver session to the control-panel webview.
 *
 * The app opens two webviews (control-panel + overlay); WebKitWebDriver may
 * attach to either. This polls the window handles and selects the one whose
 * document renders the zoom slider (the control panel). It is a no-op when the
 * session is already on the control panel. Throws (via `waitUntil`) if no
 * window renders the slider within the timeout (a real failure — the control
 * panel never hydrated).
 */
export async function switchToControlPanel(): Promise<void> {
  // The control panel gates its UI behind async settings hydration (a
  // `get_current_settings` IPC round-trip): the zoom slider only enters the DOM
  // after that resolves, which lags the native window-open by hundreds of ms
  // under headless software GL. Poll every window handle (NFR-3:
  // condition-based, never a fixed sleep) until one webview has rendered the
  // slider, leaving the driver focused there. A single-shot probe races the
  // hydration and intermittently attaches to the still-loading panel (showing
  // "Loading settings…") or the empty overlay — which is what failed the first
  // CI run of this job (DC-13).
  await browser.waitUntil(
    async () => {
      const handles = await browser.getWindowHandles();
      for (const handle of handles) {
        try {
          await browser.switchToWindow(handle);
          const hasSlider = await browser.execute(
            () => document.querySelector('[role="slider"], input[type="range"]') !== null,
          );
          if (hasSlider) {
            return true; // leaves the driver focused on the control-panel window
          }
        } catch {
          // A window handle can transiently vanish during the race; skip it
          // this tick and re-scan on the next poll.
        }
      }
      return false;
    },
    {
      timeout: 30_000,
      interval: 500,
      timeoutMsg:
        'no webview rendered the control-panel zoom slider within 30s (control panel never hydrated)',
    },
  );
}
