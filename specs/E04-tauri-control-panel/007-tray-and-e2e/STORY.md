# Story E04/007: System Tray & tauri-driver CI E2E

**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** APPROVED (approved-as-authoritative per E04 execution, 2026-06-05)
**Depends On:** 003 (live engine to assert against), 005 (IPC commands), 006 (UI)

---

## Problem Statement

E04 is functionally complete after stories 001-006, but two epic deliverables remain: the **system tray** (D6 — a tray icon with minimize-to-tray so Luminos runs unobtrusively) and the **`tauri-driver` IPC integration tests in CI** (the verification mechanism the roadmap specifies for D2/D3/D4). This story adds both, then closes the epic with an acceptance pass and an AC coverage matrix.

The tray must **degrade gracefully**: on Linux, the tray relies on a StatusNotifierItem (SNI) host that not every desktop environment runs; where absent, the app must log and keep the window visible rather than vanish into an invisible tray.

## User Scenarios

> **AC count = 4.**

### US-1: Run unobtrusively in the tray
As a user, I want Luminos to minimize to a system tray so it stays out of my way while running.

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1 (tray + minimize + graceful degrade):** Given a Linux session with a StatusNotifierItem host, when the app starts, then a tray icon appears with a menu (Show/Hide, Quit); and when the control-panel window is closed/minimized with "minimize to tray" enabled, then it hides to the tray and is restorable from the tray menu; and given **no** SNI host, when the app starts, then it logs a `warn!` and keeps the window visible (no silent disappearance), without panicking. *(FR-1, FR-2, FR-3)* — **D6**

### US-2: End-to-end IPC verified in CI
As a maintainer, I want the zoom/mode/timing round-trips verified through the real webview in CI, so that regressions in the IPC contract are caught automatically.

**Priority:** P0
**Acceptance Criteria:**

- **AC-2.1 (E2E CI job):** Given the CI pipeline, when it runs, then a `test-e2e` job (Xvfb + picom + WebKitWebDriver + `tauri-driver`) builds the app + frontend and runs the IPC integration suite to completion (pass/fail reported). *(FR-4, FR-5)*
- **AC-2.2 (D2/D3/D4 round-trips):** Given the running app under `tauri-driver`, when the suite drives the zoom slider, the mode selector, and reads the frame-timing readout, then the engine state changes accordingly (zoom level in `AppState`; mode switched; P99 displayed) — verifying D2, D3, D4 end-to-end. *(FR-6)*

### US-3: Epic closure
As the epic owner, I want all E04 success criteria verified with a traceability matrix, so that the epic can be marked DONE with evidence.

**Priority:** P0
**Acceptance Criteria:**

- **AC-3.1 (acceptance + matrix):** Given all stories 001-006 DONE, when the epic acceptance runs, then every E04 success criterion (roadmap §4.4) and deliverable D1-D8 is verified with a test/evidence reference in an AC coverage matrix, and the `HIGH_LEVEL_PLAN.md` is updated (Progress Summary + Shared Context + status). *(FR-7)*

## Functional Requirements

- **FR-1:** Add a Tauri tray icon (`tray-icon` feature, already enabled on the `tauri` workspace dep) with a menu: Show/Hide control panel, Quit. *(AC-1.1)*
- **FR-2:** Implement minimize-to-tray: closing/minimizing the control-panel window with the setting enabled hides it (window stays running); restore via tray menu / icon click. *(AC-1.1)*
- **FR-3:** Detect absence of a StatusNotifierItem host (or tray creation failure) and degrade gracefully — `warn!`, keep the window visible, no panic. *(AC-1.1)*
- **FR-4:** Add a `test-e2e` CI job installing WebKitWebDriver (ships with webkit2gtk) + `tauri-driver`, running under Xvfb + picom. *(AC-2.1)*
- **FR-5:** The job MUST build the frontend (`pnpm build`) and the app, then run the WebDriver IPC suite. *(AC-2.1)*
- **FR-6:** The IPC suite MUST verify D2 (zoom slider → `AppState` zoom), D3 (mode selector → mode switch), D4 (frame-timing readout shows P99). *(AC-2.2)*
- **FR-7:** Produce an AC/deliverable coverage matrix and update `HIGH_LEVEL_PLAN.md` to DONE when criteria pass. *(AC-3.1)*

## Non-Functional Requirements

- **NFR-1:** The tray's platform-specific glue SHOULD sit behind a clear boundary (tray module in `luminos-app`); Linux-only behavior this epic, with macOS/Windows noted for later.
- **NFR-2:** `tauri-driver` is Linux + Windows only (no macOS WKWebView driver) — the E2E job runs on Linux; document this.
- **NFR-3:** E2E tests MUST be deterministic/non-flaky (explicit waits for elements/state, not sleeps); retries via the CI profile where needed.
- **NFR-4:** No `unwrap()`/`expect()` in production tray code.

## Out of Scope

- Tray on macOS/Windows → later platform epics (E12/E17/E18); document the approach.
- Rich tray menus (per-feature toggles, profiles) → later epics; Phase 0 tray is Show/Hide + Quit.
- Auto-update / packaging → Epic 9.
- `start_on_login` behavior → later (the setting field exists but wiring is out of scope here).

## Open Questions

- [x] Does the `tray-icon` feature need extra Linux deps? — **Resolved:** Tauri tray on Linux uses libayatana-appindicator (StatusNotifierItem); CI must install it. Where no SNI host runs at runtime, degrade gracefully (FR-3).
- [x] Where do E2E tests live? — **Resolved:** a dedicated `e2e/` (WebdriverIO + `tauri-driver`) or `ui/e2e/`; TypeScript, run via `npx tsx`/the WDIO runner. They drive the built app binary, not Vitest.
- [x] How does `tauri-driver` find the binary? — **Resolved:** it launches the built `luminos-app` (release or debug) via the WebDriver `tauri:options` `application` path; the job builds it first.
- [x] Minimize-to-tray vs close-to-tray semantics? — **Resolved:** honor the existing `AppSettings.minimize_to_tray` flag; when enabled, the window-close intercept hides instead of quits; Quit is explicit via tray menu.
