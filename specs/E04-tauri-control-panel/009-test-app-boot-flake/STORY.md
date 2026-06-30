# Story E04/009 — `test-app` Subprocess Boot Flake (Quarantine + Root-Cause)

**Status:** Open — boot-dependent tests quarantined; root cause TBD
**Type:** Bugfix / CI-reliability story (independent, post-Epic-04)
**Date:** 2026-06-29

## Context

The `test-app` CI job (App Shell Tests, Tauri + Xvfb) has **never been green
since E04**. Its 19 subprocess integration tests spawn the real `luminos-app`
binary under a per-test Xvfb + picom + software-GL (Mesa llvmpipe) and poll the
captured log for boot markers (e.g. `managed_state_ok`, `overlay window
'overlay' opened`) with a ≈20 s `wait_for_log` ceiling.

On the Ubuntu-24.04 GitHub runner the app **boots too slowly under load and
intermittently misses those timeouts**. This is a **flaky** failure, not a
deterministic hang — in run `27521471598` (commit `83b42fc`) the job was
`75 tests run: 68 passed (4 flaky), 7 failed, 0 skipped`:

| Test | Outcome | Notable timing |
|---|---|---|
| `managed_state_handle_is_retrievable` | FLAKY (TRY 3 PASS) | ~24.97 s |
| `overlay_no_compositor_logs_warn_and_continues` | FLAKY (TRY 3 PASS) | ~17.2 s |
| `app_logs_overlay_xid` | FLAKY (TRY 2 PASS) | ~21.27 s |
| `overlay_surface_is_created_from_owned_window` | FLAKY (TRY 3 PASS) | ~14.5 s |
| `app_boots_two_windows_and_exits_clean` | FAILED (3/3) | ~21 s each try |
| `notify_state_changed_triggers_render` | FAILED (3/3) | ~21 s |
| `overlay_window_is_undecorated_and_clickthrough` | FAILED (3/3) | ~21 s |
| `app_overlay_window_is_bound_and_mapped` | FAILED (3/3) | ~26 s |
| `app_self_capture_hook_runs_without_panic` | FAILED (3/3) | ~36 s |
| `redraw_cadence_advances_over_one_second` | FAILED (3/3) | ~21 s |
| `tray_init_reaches_definite_outcome_without_panic` | FAILED (3/3) | ~32 s |

The flaky-pass times (≈14–25 s) straddle the 20 s timeout. Passing attempts
boot **fully** — they reach the setup hook and the overlay-XID marker — so the
binary is correct and the app *does* get past `WebKitWebView` construction when
it wins the race.

### Evidence gathered (what is known)

- **Same binary boots fine elsewhere:** the `test-e2e` job (real webview via
  `WebKitWebDriver` under `xvfb-run`) is green in the same run; the binary also
  boots on local desktops (E04/008 manual verification). ⇒ Not a product defect.
- **Failing-attempt stall point:** captured log stops after the `platform_env`
  X11-backend-pin (DEBUG) and `luminos_core::config::manager` "using default
  settings" (INFO), then silence until timeout. The stall is *after* config-load
  and *before* the first setup-hook log. Localizing it to `WebKitWebView`
  construction is a **hypothesis**, not confirmed.
- **Not locally reproducible:** on a fast, unloaded, non-Ubuntu-24.04 dev box the
  app boots in ~1 s every time, including under simulated no-session-bus /
  no-`dbus-launch` / no-`XDG_RUNTIME_DIR` conditions.

### Hypotheses ruled out (with evidence)

- **GTK AT-SPI atk-bridge stall** — `NO_AT_BRIDGE=1` (commit `83b42fc`) was in
  the failing run and did **not** help.
- **D-Bus session-bus autolaunch** — local simulation with no bus, no
  `dbus-launch`, no `XDG_RUNTIME_DIR` still boots in ~1 s.
- **WebKitGTK bwrap sandbox + Ubuntu-24.04 userns restriction** — `wry` 0.55.1,
  `tauri` 2.11.2, and `tauri-runtime-wry` 2.11.2 never call
  `set_sandbox_enabled(true)`, so the bwrap sandbox is not enabled.

### Leading (untested) hypothesis

Slow boot under runner CPU/IO contention, amplified by the harness launching a
**fresh** Xvfb + picom + software-GL stack **per test**, racing a tight 20 s
boot-marker timeout. This points at timing/contention, not a fixed
code/environment incompatibility.

## Decision (this story)

Per maintainer decision, **quarantine** the boot-dependent tests so CI is
unblocked, and diagnose the flake separately. All 19 spawn-tests carry
`#[ignore = "quarantined (DC-10/DC-13): …"]`; the `test-app` CI step runs with no
`--run-ignored`, so it executes the (passing) lib unit tests only. Tests remain
runnable locally with `--run-ignored all`.

## Acceptance Criteria

- **AC-1 (CI green via quarantine):** *Given* the `test-app` job on the
  Ubuntu-24.04 runner, *when* it runs `cargo nextest run --profile ci -p
  luminos-app … --test-threads 1`, *then* it passes (lib unit tests pass; the 19
  boot-dependent spawn-tests are skipped, not run). **(Done.)**
- **AC-2 (honest documentation):** *Given* the quarantine, *when* a developer
  reads CLAUDE.md §8 and the `#[ignore]` messages, *then* the failure is
  described accurately as an **intermittent** boot-timeout flake (not a
  deterministic hang), the localization is stated as a hypothesis, and the
  local `--run-ignored all` recovery command is correct. **(Done.)**
- **AC-3 (root cause):** *Given* the quarantined tests, *when* the flake is
  investigated, *then* the root cause of the slow/intermittent boot on the
  Ubuntu-24.04 runner is identified with evidence (e.g. CI-side instrumentation:
  `WEBKIT_DEBUG`/`G_MESSAGES_DEBUG`, process-liveness + `/proc/<pid>/wchan` +
  `gdb` thread-dump on boot-marker timeout). **(Open.)**
- **AC-4 (fix + un-ignore):** *Given* an identified root cause, *when* the fix is
  applied (candidates: raise boot-marker timeouts, reduce per-test Xvfb/GL
  startup cost, share a display, or address the underlying boot slowness), *then*
  the 19 tests are un-`#[ignore]`d and the `test-app` job is green across
  multiple consecutive CI runs. **(Open.)**

## Notes

- Exit criterion for closing: a green `test-app` run with all 19 spawn-tests
  un-ignored, repeated to confirm the flake is gone (not merely passing once).
- Do **not** un-ignore without that proof.
