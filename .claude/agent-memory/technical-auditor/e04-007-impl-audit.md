---
name: e04-007-impl-audit
description: E04 Story 007 (system tray + tauri-driver E2E + EPIC ACCEPTANCE) audit (2026-06-05, commit 8a4a4f3) — AUDIT PASS, epic E04 honestly DONE, 0 blocking
metadata:
  type: project
---

# E04 Story 007 (tray + E2E + epic close-out) Audit — 2026-06-05 (commit 8a4a4f3)

**Verdict: AUDIT PASS, 0 blocking.** This story declares Epic E04 DONE; the declaration is HONEST. No item is marked done-but-unverified. The 3 genuinely-not-CI-verifiable things (live present-on-screen, tray-icon-pixel, e2e first-run) are explicitly tiered HW/manual or CI-only, NOT papered over.

## Epic-acceptance honesty (the headline) — every SC/deliverable categorized
- All 8 deliverables (D1-D8) + 7 roadmap SCs verified-present in SUBTASKS T008 matrix + HLP Success Criteria. Each row's cited test EXISTS and covers its claim.
- Category (b) honestly-disclosed-as-CI-only/HW-manual: tauri-driver E2E (CI-only, never green-run yet — recorded as carry-forward #5), live magnified-pixels-present (HW/manual DC-10), tray-icon-VISIBLE + left-click restore (HW/manual, needs real SNI host).
- Category (c) OVERCLAIMED: NONE found.
- "Zoom UI→Rust→ArcSwap→render→frame" precision: NOT one single test. It's a CHAIN — E2E D2 covers UI→cmd→ArcSwap (CI-only); `live_zoom_change_reflected_next_frame` (tests/live_magnification.rs:133) covers ArcSwap→render BUT trigger is HOTKEY (ctrl+alt+equal) not slider, and asserts via STATE/REGION LOG not on-screen pixels. The →render leg IS real: `present_if_ready` (app.rs:407/409 in loop) reads zoom from ArcSwap and logs `magnify_region zoom='3'` (app.rs:756). Final pixel-present honestly tiered HW/manual. Honest decomposition, not overclaim.

## Graceful-degrade robustness (headline #2) — PROVABLY cannot panic/abort
- tray.rs::init_tray: every path returns `Ok` (none propagate). 3 `?` are confined to build_menu (110-112); its Result consumed by match in init_tray (70-79) → warn!+Ok(None). NO unwrap/expect/panic/unreachable outside #[cfg(test)]. `#[allow(unnecessary_wraps)]` is correct.
- Degrade is DETERMINISTIC: `session_bus_available()` = `is_some_and(|v| !v.is_empty())`; test `tray_absent_host_degrades` spawns with `("DBUS_SESSION_BUS_ADDRESS","")` → empty → false → pre-check degrade fires UNCONDITIONALLY even though this box HAS a bus. True forcing.
- BOTH paths verified LIVE on this box: positive `tray=ready`+`tray_stashed=true` (ran binary 8s w/ real bus, libayatana-appindicator loaded); degrade test PASSED (panel stays X11-mapped, exit 0, no panic). Implementer's "verified both" claim TRUE.
- init_tray_into_handle (app.rs) further log-and-swallows even the (impossible) Err. Control panel provably stays visible.

## Minimize-to-tray + FR-1 — exactly 2 hide sites, overlay never hidden
- Exactly 2 `.hide()`: app.rs:270 (close-handler, gated `window.label()=="control-panel"` at :255) + tray.rs:152 (menu toggle, gated CONTROL_PANEL_LABEL). Overlay structurally unreachable by both.
- `handle_close_to_tray` reads minimize_to_tray lock-free from SAME ArcSwap the force-hook mutated (LUMINOS_FORCE_MINIMIZE_TO_TRAY=1 sets state.settings BEFORE ArcSwap::from_pointee; builder_state=Arc::clone). is_some_and / if-let-Err — no panic.
- NO winit EventLoop in app src (only doc-comments forbidding it). Quit via app.exit(0) (tray.rs:133 + predefined quit item). FR-1 intact.

## Counts/deps/pins — ALL EXACT
- bindings.ts UNCHANGED in 8a4a4f3 (git diff empty). Cargo.lock ZERO diff (tray-icon 0.23.1 already transitive via tauri 2.11.2). No new cargo dep.
- 446 workspace tests pass (3 skipped). 67 app tests pass (49 src unit + 18 tests/) = +7 tray (4 unit tray::tests + 3 subprocess tests/tray.rs). All exact. (App suite has 11 flaky-retries on this CONTENDED box — pre-existing story-003/DC-12 per-frame-x11-connect; NOT tray tests; all eventually pass.)
- e2e npm pins EXACT in PINNED_VERSIONS §2a, all match package.json. Registry-verified publish dates: webdriverio 9.27.1=2026-04-30, @wdio/mocha-framework 9.27.1=2026-04-30, mocha 11.7.6=2026-05-21 (==cutoff, eligible), tsx 4.22.3=2026-05-19. webdriverio 9.27.2=2026-05-26 (>cutoff, correctly excluded as "too young"). Bulk npm advisory endpoint = {} (zero advisories). All ≤2026-05-21.
- 8 active CI jobs (lint,test-rust-unit,security,coverage,test-platform,test-gpu,test-app,test-e2e) + 2 if:false placeholders. YAML parses. test-e2e job fully authored (webkit2gtk-driver, libayatana-appindicator3-dev, tauri-driver 2.0.6 --locked, xvfb/picom, headless env).
- clippy clean (exit 0) on luminos-app --features "tauri ci_platform_tests" w/ unwrap_used+expect_used+pedantic. fmt --check clean. e2e tsc --noEmit exit 0.

## Self-flagged items — all honestly disclosed + non-blocking
- Dead MENU_ID_QUIT arm (tray.rs:131): predefined quit handles quit; arm retained for future custom item. In Deviations + carry-forward #5.
- minimize-to-tray test SKIP guards (window never maps / WM_DELETE send fails): graceful skips, honest.
- LUMINOS_FORCE_MINIMIZE_TO_TRAY: env-gated, default no-op, mirrors LUMINOS_FORCE_ACTIVE, test-only. In Deviations.
- Local-E2E deferral: T005/T007 status strings say "authored + typechecked; CI-only run" / "live flake-soak deferred to CI". Carry-forward #5.

## Carry-forward backlog (HLP 367-374) — RECORDED, not dropped
6 items: RISK-001→Retired in risk register, winit→tao doc updates (doc-01/05/roadmap), raw_*_handle cleanup, code-polish backlog (incl RISK-004/DC-12 per-frame x11 connect), 007-specifics (e2e flake-soak + tray dogfood + real-GPU present + dead MENU_ID_QUIT), DC-4 @crabnebula→Rust tauri-driver doc replace. Owned by cross-cutting task #9.

## Non-blocking pointers
- P-001: handle_close_to_tray double-retrieves the window (param `tauri::Window` → get_webview_window for `.hide()`). Correct (WindowEvent gives bare Window; hide on WebviewWindow) but slightly redundant. Style nit.
- P-002: "zoom round-trips...→render→frame" SC wording could mislead a skim-reader into thinking one E2E drives slider→pixels. It's a multi-test chain w/ pixel-present tiered HW/manual. Matrix discloses this correctly; wording is defensible.
