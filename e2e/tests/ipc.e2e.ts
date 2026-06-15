/**
 * Story E04/007 end-to-end IPC tests (D2/D3/D4) via `tauri-driver`.
 *
 * These drive the REAL control-panel webview (the built React UI) and assert
 * the REAL engine state through the `get_current_settings` round-trip — not the
 * React store — so the full IPC contract (UI -> command -> StateManager ->
 * ArcSwap) is verified, which is the purpose of the `tauri-driver` job.
 *
 * Determinism (NFR-3): every assertion is gated by `browser.waitUntil`, never a
 * fixed sleep. Selectors are role/name based (`aria/...`) or value-based —
 * never the slider's `aria-labelledby`, which is a dynamic React `useId()`.
 *
 * Tier (DC-10/DC-13): under headless software GL the GPU present never runs, so
 * frame timings stay 0 — D4 asserts the P99 field is PRESENT and finite, not
 * non-zero (the non-zero P99 is a real-GPU / manual concern, per the matrix).
 *
 * CI-ONLY: requires `WebKitWebDriver` + the Rust `tauri-driver`. Not runnable on
 * a dev box lacking those (the suite is authored + typechecked locally).
 */

import {
  driveZoomSlider,
  getCurrentSettings,
  getEngineMode,
  getEngineZoom,
  getFrameTimings,
  switchToControlPanel,
} from '../support/ipc.js';

/** Default seeded zoom is 2.0; the slider target must differ from it. */
const TARGET_ZOOM = 8;
/** Tolerance for float equality on the zoom round-trip. */
const ZOOM_EPSILON = 0.01;

describe('Luminos control panel IPC (D2/D3/D4)', () => {
  before(async () => {
    // The app opens TWO webviews (control-panel + overlay). WebKitWebDriver may
    // attach to either; switch to the one rendering the control panel (the
    // slider only exists there) before any assertion. This is the T007
    // CI-stabilization seam — without it the suite could attach to the empty
    // overlay and time out.
    await switchToControlPanel();

    // Wait until the React control panel has hydrated: the zoom slider is the
    // canonical "UI is interactive" signal (it renders after hydration).
    const slider = await $('aria/Zoom level');
    await slider.waitForExist({ timeout: 30_000 });
  });

  it('D2 zoom slider round-trips through IPC to engine zoom', async () => {
    // Sanity: the engine starts at a zoom different from our target so the
    // assertion is non-vacuous.
    const initialZoom = await getEngineZoom();
    expect(Math.abs(initialZoom - TARGET_ZOOM)).toBeGreaterThan(ZOOM_EPSILON);

    // Drive the slider to the target value (fires the React onChange ->
    // setZoomLevel command). See `driveZoomSlider` for why WebdriverIO
    // `setValue` can't be used on a range input under WebKitWebDriver.
    await driveZoomSlider(TARGET_ZOOM);

    // Assert the ENGINE state, not the DOM: poll get_current_settings until the
    // round-trip lands (UI -> set_zoom_level -> StateManager -> ArcSwap).
    await browser.waitUntil(
      async () => Math.abs((await getEngineZoom()) - TARGET_ZOOM) < ZOOM_EPSILON,
      {
        timeout: 15_000,
        timeoutMsg: `engine zoom never reached ${TARGET_ZOOM} after driving the slider`,
      },
    );
  });

  it('D3 mode selector round-trips through IPC to engine mode', async () => {
    // The mode radio group exposes a radio per mode by its enum `value`.
    const lensRadio = await $('input[value="Lens"]');
    await lensRadio.waitForClickable({ timeout: 15_000 });
    await lensRadio.click();

    await browser.waitUntil(async () => (await getEngineMode()) === 'Lens', {
      timeout: 15_000,
      timeoutMsg: 'engine mode never switched to Lens after selecting the radio',
    });

    // Switch back to FullScreen to prove the round-trip is not a one-way latch.
    const fullScreenRadio = await $('input[value="FullScreen"]');
    await fullScreenRadio.waitForClickable({ timeout: 15_000 });
    await fullScreenRadio.click();

    await browser.waitUntil(async () => (await getEngineMode()) === 'FullScreen', {
      timeout: 15_000,
      timeoutMsg: 'engine mode never returned to FullScreen',
    });
  });

  it('D4 frame-timing readout exposes a finite P99 via IPC', async () => {
    // The dev-only `FrameTimingDisplay` is stripped from production builds, so
    // assert via the command path (the robust seam). Under headless software GL
    // P99 is 0 (no GPU present) — assert PRESENCE and finiteness, not non-zero.
    const summary = await getFrameTimings();

    expect(typeof summary.p99Ms).toBe('number');
    expect(Number.isFinite(summary.p99Ms)).toBe(true);
    expect(summary.p99Ms).toBeGreaterThanOrEqual(0);

    // The summary shape must round-trip the camelCase keys (DC-5).
    expect(typeof summary.averageMs).toBe('number');
    expect(typeof summary.targetFps).toBe('number');
    expect(summary.targetFps).toBeGreaterThan(0);
  });

  it('exposes the full settings shape over IPC (contract smoke)', async () => {
    // A belt-and-braces check that the read command returns the nested shape
    // the frontend depends on (story-006 contract), via the real webview.
    const settings = await getCurrentSettings();
    expect(settings.magnification).toBeDefined();
    expect(typeof settings.magnification.zoom_level).toBe('number');
    expect(['FullScreen', 'Docked', 'Lens']).toContain(settings.magnification.mode);
  });
});
