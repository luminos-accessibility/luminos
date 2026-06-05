# Subtasks: Story E04/007 -- System Tray & tauri-driver CI E2E

**Status:** NOT STARTED
**Started:** ---
**Completed:** ---
**Story:** [STORY.md](./STORY.md)
**Design:** [DESIGN.md](./DESIGN.md)
**Epic:** [../HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 1 | 0 | 0 | 1 |
| 2. Core (tray) | 3 | 0 | 0 | 3 |
| 3. Integration (E2E + CI) | 3 | 0 | 0 | 3 |
| 4. Polish & Epic Acceptance | 1 | 0 | 0 | 1 |
| **Total** | **8** | **0** | **0** | **8** |

---

## Phase 1: Setup

### T001 -- Tray module scaffold + debug state probe
**Traces to:** FR-1, FR-6
**Status:** TODO
**Files:** `crates/luminos-app/src/tray.rs`, `crates/luminos-app/src/main.rs`

**TDD Cycle:** (setup + probe)
1. **Green:** Scaffold `tray.rs` (`init_tray`, `toggle_main_window`). Add a test/debug-only state-readout element/command the E2E suite can assert against (e.g. a hidden `data-zoom` element fed by `get_current_settings`, gated to debug builds).
2. **Refactor:** —

**Completion Notes:**
>

---

## Phase 2: Core (tray)

### T002 -- Tray icon + menu (Show/Hide, Quit)
**Traces to:** FR-1, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-app/src/tray.rs`, `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:** `tray_appears_with_sni` (subprocess, appindicator host) -- tray present; menu Show/Hide + Quit; `quit_from_tray_exits` -- exit 0.
2. **Green:** `TrayIconBuilder` with menu + handlers; `init_tray` in `setup`.
3. **Refactor:** —

**Completion Notes:**
>

---

### T003 -- Minimize-to-tray window intercept
**Traces to:** FR-2, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-app/src/main.rs`

**TDD Cycle:**
1. **Red:** `minimize_to_tray_hides_window` -- with `minimize_to_tray=true`, close → window hidden (running); tray Show restores.
2. **Green:** `on_window_event` `CloseRequested` → `api.prevent_close()` + `hide()` when flag set; else quit.
3. **Refactor:** Read flag from `AppState.settings.minimize_to_tray`.

**Completion Notes:**
>

---

### T004 -- Graceful degrade (no SNI host)
**Traces to:** FR-3, NFR-4, AC-1.1
**Status:** TODO
**Files:** `crates/luminos-app/src/tray.rs`

**TDD Cycle:**
1. **Red:** `tray_absent_host_degrades` (subprocess, no SNI host) -- `warn!` logged, window visible, no panic.
2. **Green:** Wrap tray init in error handling; on failure `warn!` + continue.
3. **Refactor:** —

**Checkpoint:** Tray works with SNI host; degrades cleanly without; minimize/restore/quit functional (D6).

**Completion Notes:**
>

---

## Phase 3: Integration (E2E + CI)

### T005 -- E2E project (WebdriverIO + tauri-driver)
**Traces to:** FR-5, FR-6, AC-2.2
**Status:** TODO
**Files:** `e2e/{package.json,tsconfig.json,wdio.conf.ts}`, `e2e/tests/ipc.e2e.ts`

**TDD Cycle:**
1. **Red (E2E specs):**
   - [ ] `D2 zoom slider changes engine zoom` -- set slider → `waitUntil` engine zoom == value (debug probe).
   - [ ] `D3 mode selector switches mode` -- select → mode switched.
   - [ ] `D4 frame-timing readout shows P99` -- readout renders a P99.
2. **Green:** WDIO config targeting the built binary via `tauri:options.application`; pin the driver as **`@crabnebula/tauri-driver`** (tauri-driver moved to CrabNebula) at an exact version; TypeScript (run via WDIO/`tsx`); explicit `waitUntil` (no sleeps).
3. **Refactor:** Page-object helpers; `readZoomProbe()` util.

**Completion Notes:**
>

---

### T006 -- CI `test-e2e` job
**Traces to:** FR-4, AC-2.1
**Status:** TODO
**Files:** `.github/workflows/ci.yml`, `CLAUDE.md`

**TDD Cycle:** (CI)
1. **Green:** Add `test-e2e` job: install webkit2gtk + WebKitWebDriver + libayatana-appindicator + xvfb + picom + tauri-driver; `pnpm -C ui build`; `cargo build -p luminos-app --features tauri`; start picom under xvfb; run WDIO suite. Document in `CLAUDE.md` CI section.
2. **Refactor:** Cache deps; mark job appropriately in the pipeline DAG.

**Completion Notes:**
>

---

### T007 -- Stabilize E2E (determinism)
**Traces to:** NFR-3, AC-2.1
**Status:** TODO
**Files:** `e2e/**`

**TDD Cycle:**
1. **Red:** Run the suite repeatedly; assert no flakiness (explicit waits, retries per CI profile).
2. **Green:** Replace any implicit timing with `waitUntil`; add retry where justified.
3. **Refactor:** —

**Checkpoint:** `test-e2e` green and stable in CI; D2/D3/D4 verified end-to-end.

**Completion Notes:**
>

---

## Phase 4: Polish & Epic Acceptance

### T008 -- Epic acceptance: AC/deliverable matrix + close-out
**Traces to:** FR-7, AC-3.1, all E04 success criteria
**Status:** TODO
**Files:** `specs/E04-tauri-control-panel/HIGH_LEVEL_PLAN.md`, story docs

**Verification Checklist:**
- [ ] D1 windows open (001) ✔ evidence
- [ ] D2 zoom real-time (003+005+006, E2E 007) ✔
- [ ] D3 mode selector (005+006, E2E 007) ✔
- [ ] D4 frame timing (003+005+006, E2E 007) ✔
- [ ] D5 persistence (004) ✔
- [ ] D6 tray + minimize-to-tray (this story) ✔
- [ ] D7 tauri-specta bindings (005) ✔
- [ ] D8 axe-core zero violations (006) ✔
- [ ] All 7 roadmap success criteria verified with test references
- [ ] Phase-0 gate docs task filed (doc-01/05/roadmap winit→tao update; RISK-001 status)
- [ ] `HIGH_LEVEL_PLAN.md`: Progress Summary all DONE, Shared Context final, status DONE
- [ ] AC coverage matrix produced (all stories' ACs → passing tests)

**Completion Notes:**
>

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| --- | --- | --- |
