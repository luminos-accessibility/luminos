# Story 007 — Implementation Notes (lead briefing, 2026-06-05)

Source-verified against worktree HEAD (001-006 done) + the unpacked tauri-2.11.2 crate + the official Tauri
WebDriver docs. SUPERSEDES stale DESIGN parts — log conflicts in `SUBTASKS.md → Deviations from Design`.

## ⚠️ CWD + ENVIRONMENT
Bare shell = `/home/renatorro/Development/luminos` (branch main — 001-006 absent). Work in the worktree:
`cd /home/renatorro/Development/luminos/.claude/worktrees/epic+e04-control-panel`. **`WebKitWebDriver` and
`xvfb-run` are NOT on this dev box** → the tauri-driver E2E (B/C) is **CI-AUTHORED, NOT locally runnable**.
Only the tray (A) + its degrade subprocess test are locally runnable. Do NOT claim local E2E green — rely on CI.

## A. System tray (D6) — new `crates/luminos-app/src/tray.rs`
Tauri `tray-icon` feature is already ON (root Cargo.toml:72). Real API (tauri-2.11.2/src/tray/mod.rs):
`TrayIconBuilder::new().menu(&menu).icon(img).tooltip(s).on_menu_event(|app,ev|...).on_tray_icon_event(|tray,ev|...)
.build(app) -> Result<TrayIcon>`. Menu: `Menu::with_items(app, &[&show_hide, &sep, &quit])`,
`MenuItem::with_id(app, "toggle", "Show/Hide Panel", true, None)`, `PredefinedMenuItem::quit(app, None)` + `::separator`.
Icon: `app.default_window_icon().cloned()` (reuse the loaded window icon) or `Image::from_path` anchored to
`env!("CARGO_MANIFEST_DIR")` (mirror ipc.rs:63 — never CWD).
- `init_tray(app: &tauri::App) -> Result<Option<TrayIcon<tauri::Wry>>, AppError>`; call from `app::run`'s `.setup`
  right after `setup_overlay_window(app)?`. **STASH the returned TrayIcon** (it's refcounted; dropping it removes the
  icon) — add a Linux-gated field to `LuminosHandle` (like `window_manager`) or `app.manage()` it.
- `toggle_control_panel(app: &AppHandle)`: `app.get_webview_window("control-panel")` → `is_visible()?` → `hide()`/`show()+set_focus()`.
- Menu event: match `event.id().as_ref()` → "toggle" → toggle_control_panel; "quit" → `app.exit(0)` (routes through the
  existing ExitRequested|Exit teardown). Tray-icon left-click is best-effort (SNI backend often only delivers menu
  events) — the **menu Show/Hide item is the reliable restore path**, not icon-click.
- Reuse `AppError::Tauri(tauri::Error)` (already exists).

### Graceful degrade (FR-3, the KEY requirement)
On Linux `build()` returns Ok even when NO StatusNotifierWatcher host exists (e.g. headless Xvfb) — the icon just
never shows. Do NOT gate on the Result alone. Strategy: `init_tray` returns `Result<Option<TrayIcon>, _>`; on
`build()` Err → `log::warn!("tray unavailable: '{e}'; keeping control panel visible")` + return `Ok(None)` (NEVER
`?`-propagate out of setup → would abort startup). Pre-check heuristic: if `$DBUS_SESSION_BUS_ADDRESS` is absent →
`warn!` + skip the build (return Ok(None)). Emit a structured marker `tray=ready` / `tray=degraded` for the test.
INVARIANT under every degrade path: the control-panel window stays visible; no panic; no `unwrap`/`expect` (NFR-4).
(A robust zbus SNI-watcher probe is a Phase-1 nicety; Phase-0 ships env-heuristic + Ok-on-error.)

### Minimize-to-tray (use `.on_window_event`, NOT RunEvent)
`RunEvent::WindowEvent` is observation-only (no prevent_close). Register `.on_window_event(|window, event|)` on the
`tauri::Builder` in `app::run` (app.rs:97):
```rust
.on_window_event(|window, event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == "control-panel" {
            let minimize = window.app_handle().try_state::<LuminosHandle>()
                .map(|h| h.app_state.load().settings.minimize_to_tray).unwrap_or(false);
            if minimize { api.prevent_close(); let _ = window.hide(); }  // logged, non-fatal
        }
    }
})
```
**Hide ONLY `control-panel` — NEVER the overlay** (hiding the overlay kills magnification). `minimize_to_tray` is at
`AppState.settings.minimize_to_tray` (read lock-free via ArcSwap).

## B. tauri-driver E2E (D2/D3/D4) — new `e2e/` TS project (CI-ONLY)
**Driver correction:** DESIGN/SUBTASKS say `@crabnebula/tauri-driver` (npm) — STALE. Use the **Rust crate
`tauri-driver` v2.0.6** (`cargo install tauri-driver --version 2.0.6 --locked`; already at ~/.cargo/bin). Harness:
**WebdriverIO 9 + Mocha** (TypeScript — no Python). `e2e/{package.json, tsconfig.json, wdio.conf.ts, tests/ipc.e2e.ts}`.
wdio capability `'tauri:options': { application: '../target/debug/luminos-app' }`, `hostname 127.0.0.1 port 4444`;
`beforeSession` spawns `~/.cargo/bin/tauri-driver`, cleanup on SIGINT/TERM/HUP. Pass the headless-webkit env to the
app (same as `tests/common/mod.rs:188`): `GDK_BACKEND=x11 WEBKIT_DISABLE_COMPOSITING_MODE=1
WEBKIT_DISABLE_DMABUF_RENDERER=1 LIBGL_ALWAYS_SOFTWARE=1`.
**Assert ENGINE state via `get_current_settings` round-trip (no new debug probe — the AD-4 write-through is proven):**
- D2: move zoom slider (`$('input[type=range]')` / `aria/Zoom level`) → poll `invoke('get_current_settings')` →
  `settings.magnification.zoom_level === expected`.
- D3: click mode radio (`$('input[value="Lens"]')`) → `get_current_settings` → `magnification.mode === 'Lens'`.
- D4: `commands.getFrameTimings()` → assert `typeof p99Ms === 'number'` and finite (it's **0 headless** per DC-13 —
  assert PRESENCE, not non-zero). The dev-only `FrameTimingDisplay` is stripped from production builds, so DO NOT rely
  on the rendered `<dd>`; use the IPC command. (If you build the frontend `--mode development` you may also assert the
  rendered field, but the command path is the robust one.)
Selectors: the slider's `aria-labelledby` is a dynamic `useId()` (NOT the literal `"zoom-label"` DESIGN assumes) — use
role/name. Determinism: `browser.waitUntil` with timeouts, NEVER `pause()`. Poll `get_current_settings` (authoritative
sync read) for the primary assertion, not the delta-gated `zoom_changed` event. `mode_changed` only fires from the
command echo (no Phase-0 hotkey — DC-14).

## C. CI `test-e2e` job (`.github/workflows/ci.yml`; mirror to CLAUDE.md)
`runs-on: ubuntu-latest`, `needs: [lint]`. apt: the test-app set PLUS **`webkit2gtk-driver`** (ships WebKitWebDriver on
Ubuntu) + **`libayatana-appindicator3-dev`** (tray SNI). `cargo install tauri-driver --version 2.0.6 --locked`. Build:
Node 24 + `pnpm --dir ui install --frozen-lockfile && pnpm --dir ui build`; `cargo build -p luminos-app --features
tauri` (debug — match the wdio `application` path `target/debug/luminos-app`). Run:
`xvfb-run -s "-screen 0 1920x1080x24" bash -c "picom --backend xrender --daemon && pnpm --dir e2e install
--frozen-lockfile && pnpm --dir e2e test"` with `MESA_GL_VERSION_OVERRIDE=4.5 LIBGL_ALWAYS_SOFTWARE=1` + the 4
`WEBKIT_*`/`GDK_BACKEND` vars. **Mirror to CLAUDE.md** (rule line 93): add "### E2E Tests" subsection, bump "7 active
jobs"→8, note the local run is deferred. Pin all e2e npm deps EXACT (supply-chain rule); research safe versions for
webdriverio/@wdio/* /mocha and add to PINNED_VERSIONS.

## D. Epic acceptance (T008)
Produce the AC coverage matrix across ALL 7 stories + verify the 8 Success Criteria + D1-D8, marking each tier
(Node / Rust-unit / CI-Xvfb / **hardware-only** per DC-10/DC-13). Hardware-only (record explicitly — do NOT mark
"fully CI-verified"): live GPU present, non-zero P99, tray-icon-visible-on-screen. Flip `HIGH_LEVEL_PLAN.md`: story-007
row → DONE, Success Criteria checkboxes ticked w/ refs, epic status → DONE, Retrospective filled. RECORD (do not
necessarily fix here) the close-out carry-forward backlog — the lead handles the cross-cutting close-out (task #9):
RISK-001→Retired, tech-strategy doc updates (winit→tao in doc-01 §3.3/§6.5, doc-05 §4.1, roadmap §4.4), the
WindowManager::raw_*_handle cleanup, and the code-polish backlog.

## E. DESIGN staleness (apply + log)
1. Tray init goes in `app.rs` `.setup` + `.on_window_event` — NOT `main.rs` (thin shim).
2. Window labels `control-panel` / `overlay` (not "main").
3. Driver = Rust `tauri-driver` v2.0.6 (not @crabnebula npm).
4. Slider selector = role/name (aria-labelledby is dynamic useId, not "zoom-label").
5. No new debug probe — `get_current_settings` is the authoritative engine-state hook.
6. D4 via `getFrameTimings()` IPC (FrameTimingDisplay is dev-only/stripped).
7. bindings path `../../ui`; e2e project at repo-root `e2e/`.
8. Tray needs NO capability change — keep `default.json` (`core:default`+`core:event:default`); don't break `capability_minimal`.

## F. Subtasks (~8) + tiers
T001 tray.rs scaffold + LuminosHandle field (local unit). T002 icon+menu wired in setup; menu/icon events; quit→exit(0)
(positive path needs CI SNI host). T003 minimize-to-tray `.on_window_event` (CI subprocess, x11rb window-tree). T004
graceful degrade — env/DBus heuristic, Ok-on-error, panel stays visible (**locally runnable** under no-SNI Xvfb,
mirror `launch_without_compositor`; this is the AC-load-bearing tray test). T005 e2e/ project (WDIO9+Mocha; D2/D3/D4;
CI-only — author + `tsc` typecheck locally). T006 CI test-e2e job + CLAUDE.md mirror. T007 stabilize E2E (waitUntil,
the D4 field decision). T008 epic acceptance + HLP→DONE + record carry-forward. Tray tests: new `tests/tray.rs`,
`#[cfg(all(target_os="linux", feature="ci_platform_tests"))]`, assert via structured log markers + `find_windows`;
use `LUMINOS_*`-style env hooks to force the degrade path deterministically.

## G. Risks
- Tray positive path may be untestable even in CI without a DBus session + appindicator host → make the DEGRADE path
  the AC-load-bearing test; treat tray-visible as manual/dogfood (record in matrix). - E2E CI-only (no local green). -
  D2/D4 can't assert pixels/non-zero P99 in CI (DC-10) — assert engine STATE + field presence. - FR-1 intact: tray +
  prevent-close run inside the ONE App::run loop; no winit EventLoop; quit via `app.exit(0)` → existing teardown. -
  Hide ONLY control-panel. - Stash the TrayIcon (lifetime). - tray/menu closures are Send+Sync+'static → reach state via
  `app.try_state::<LuminosHandle>()`, never hold a borrow. - Tray adds NO IPC → bindings.ts stays frozen (don't regress the diff gate).

## Close-out carry-forward (the lead/task-#9 owns these; T008 records them)
RISK-001→Retired; tech-strategy doc updates (winit→tao, ×3 docs + nested AppState); WindowManager::raw_*_handle
cleanup; code-polish [002 PropertyFailed, 003 SurfaceErrorKind, 003 shutdown risk-reg entry, 003 BGRA prose, 005
compute_emit_delta unit test, 005 deny.toml prune]; DC-4 driver-package correction (record as Deviation).
