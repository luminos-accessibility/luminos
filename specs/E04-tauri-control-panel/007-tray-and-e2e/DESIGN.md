# Design: Story E04/007 -- System Tray & tauri-driver CI E2E

**Story:** [STORY.md](./STORY.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DRAFT
**Author:** principal-architect
**Risk Refs:** RISK-028 (Tauri build/runtime constraints), RISK-020 (webview surface — E2E exercises it)

---

## Overview

Two additions plus epic closure: (1) a Tauri system tray with Show/Hide + Quit and minimize-to-tray, degrading gracefully where no StatusNotifierItem host exists; (2) a `test-e2e` CI job using `tauri-driver` + WebKitWebDriver to verify D2/D3/D4 IPC round-trips through the real webview. Then an acceptance pass + AC matrix and `HIGH_LEVEL_PLAN.md` → DONE.

## Architecture

### Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| `luminos-app/src/tray.rs` | New | Build the tray icon + menu; wire Show/Hide/Quit; minimize-to-tray window intercept; SNI-absence detection + graceful degrade. |
| `luminos-app/src/main.rs` | Modified | Init tray in `setup`; intercept window-close per `minimize_to_tray`. |
| `e2e/` (WebdriverIO + tauri-driver) | New | TS E2E suite driving the built binary. |
| `e2e/{wdio.conf.ts, tests/ipc.e2e.ts, package.json, tsconfig.json}` | New | WDIO config + D2/D3/D4 specs. |
| `.github/workflows/ci.yml` | Modified | `test-e2e` job (Xvfb+picom+WebKitWebDriver+libayatana-appindicator+tauri-driver). |
| `CLAUDE.md` (CI section) | Modified | Document the new job (source of truth). |
| `HIGH_LEVEL_PLAN.md` | Modified | Epic acceptance: Progress Summary, Shared Context, status DONE. |

### Tray flow
```
setup(): TrayIconBuilder.menu(Show/Hide, Quit).on_menu_event(...).on_tray_icon_event(click→toggle window).build(app)
   └─ on error / no SNI host → warn!("tray unavailable: {e}; window stays visible"), continue (FR-3)
window CloseRequested: if settings.minimize_to_tray { api.prevent_close(); window.hide() } else { quit }
tray "Quit": app.exit(0)
tray "Show/Hide" or icon click: toggle window.show()/hide()
```

### E2E flow
```
CI test-e2e job:
  install webkit2gtk + WebKitWebDriver + libayatana-appindicator + xvfb + picom
  pnpm -C ui build ; cargo build -p luminos-app --features tauri
  start picom under xvfb
  tauri-driver (spawns WebKitWebDriver) ; wdio runs e2e/tests/ipc.e2e.ts against the built binary
ipc.e2e.ts:
  D2: set slider → assert zoom in a debug state-readout element / a get_current_settings probe
  D3: choose mode → assert mode switched
  D4: read FrameTimingDisplay → assert a P99 value renders
```

## API Design

```rust
// luminos-app/src/tray.rs
pub(crate) fn init_tray(app: &tauri::App) -> Result<(), AppError>; // Ok even if SNI absent (logs + degrades)
fn toggle_main_window(app: &tauri::AppHandle);
// main.rs window-close intercept
on_window_event(|w, ev| if let WindowEvent::CloseRequested { api, .. } = ev {
    if minimize_to_tray(w) { api.prevent_close(); let _ = w.hide(); }
});
```

```typescript
// e2e/tests/ipc.e2e.ts (WebdriverIO; explicit waits)
it('D2 zoom slider changes engine zoom', async () => {
  const slider = await $('input[aria-labelledby="zoom-label"]');
  await slider.setValue(8);                 // or keyboard
  await browser.waitUntil(async () => (await readZoomProbe()) === 8.0);
});
// D3 mode selector, D4 frame-timing readout similar, with explicit waitUntil (no sleeps).
```

> The E2E suite needs an observable hook for engine state. Reuse the debug `get_current_settings`/a hidden debug readout element (gated to test/debug builds) so assertions read real engine state, not just UI state.

## Error Handling
- Tray build failure / no SNI host → `warn!` + continue (FR-3); never panic.
- Window hide/show errors → logged, non-fatal.
- E2E: explicit `waitUntil` with timeouts; failures surface as test failures with diagnostics; CI retries per profile (NFR-3).

## Platform Considerations

| Platform | Tray | E2E |
|----------|------|-----|
| Linux X11 | libayatana-appindicator (SNI); graceful degrade if absent. | `tauri-driver` + WebKitWebDriver (this story). |
| macOS | `NSStatusItem` (E12); `tauri-driver` NOT available (no WKWebView driver) — manual/Playwright-webview alternative later. | Deferred. |
| Windows | Win32 notification-area icon (E17/18); `tauri-driver` via Edge WebView2 driver. | Deferred. |

## Testing Strategy

### Tray (subprocess, Linux)
- `tray_appears_with_sni` (subprocess, with appindicator host) — tray present; menu has Show/Hide, Quit.
- `tray_absent_host_degrades` (subprocess, no SNI host) — `warn!` logged, window visible, no panic (AC-1.1).
- `minimize_to_tray_hides_window` — with flag on, close → window hidden (still running); tray Show restores (AC-1.1).
- `quit_from_tray_exits` — tray Quit → exit 0.

### E2E (tauri-driver, CI)
- `D2/D3/D4` specs (AC-2.2) drive slider/mode/timing and assert engine state via the debug probe.
- Job-level: `test-e2e` runs to completion in CI (AC-2.1).

### Epic acceptance
- AC coverage matrix mapping D1-D8 + each success criterion → the verifying test across stories 001-007 (AC-3.1).

### Acceptance Tests

| AC | Test Type | Verification |
|----|-----------|--------------|
| AC-1.1 | Subprocess | Tray present + minimize/restore + quit; no-SNI graceful degrade. |
| AC-2.1 | CI | `test-e2e` job runs to completion. |
| AC-2.2 | E2E (tauri-driver) | D2/D3/D4 round-trips assert engine state. |
| AC-3.1 | Doc + review | AC/deliverable matrix complete; HIGH_LEVEL_PLAN updated to DONE. |

## Performance Targets
- N/A (tray + CI); E2E must complete within a reasonable CI budget (parallel job).

## Security Considerations
- Tray menu actions are local. E2E runs trusted built binary. No new capabilities (Show/Hide/Quit use core window APIs already permitted; confirm no extra capability needed for tray — `tray-icon` is a build feature, not a capability permission).

## Alternatives Considered
1. **Playwright/Selenium against the webview instead of `tauri-driver`.** Rejected for Linux — `tauri-driver` is the official path (Linux/Windows); revisit for macOS later (NFR-2).
2. **Close-to-quit (no tray hide).** Rejected — D6 requires minimize-to-tray; honor `minimize_to_tray` setting.
3. **Skip E2E, rely on Vitest + manual.** Rejected — roadmap specifies `tauri-driver` IPC tests for D2/D3/D4 (the user confirmed standing up the CI job).
4. **Assert UI state only in E2E.** Rejected — assert real engine state via a debug probe so the IPC contract (not just the React store) is verified.
