---
name: e04-008-impl-audit
description: E04 Story 008 post-E04 runtime/CI bugfix audit — AC-8 close-quits integration test is FLAKY (~17% fail), STORY overclaims it cleanly passes
metadata:
  type: project
---

# E04/008 Post-E04 Runtime & CI Fixes — Audit (2026-06-12, uncommitted on main)

Verdict: APPROVE-WITH-CAVEATS. Code fixes are correct; one test-honesty overclaim.

**Why:** Bugfix story for 5 post-E04 defects (2 CI + 3 runtime) + round-2 (quiet console, cargo-run-default, close-quits).

**How to apply:** Most claims VERIFIED. The single real finding is the AC-8 test flakiness — flag it on any re-audit or if it lands red in CI.

## Verified
- AC-6: all 5 markers (`redraw=`,`inactive_clear`,`magnify_region/capture/present`) are `log::debug!`, zero remain `info` (app.rs). Crate `luminos-app`→module root `luminos_app`; `RUST_LOG=info,luminos_app=debug` in tests/common/mod.rs:189 makes them visible (env_logger filters by compile-time module_path!, thread-agnostic). Mechanism sound.
- AC-7: `default = ["tauri","custom-protocol"]` + `custom-protocol = ["tauri?/custom-protocol"]` (Cargo.toml). Tauri 2.11.2. Root cause CORRECT: Tauri v2 `is_dev()` = NOT(custom-protocol feature); embedded assets only compiled when custom-protocol on. CAVEAT: docs say `cargo tauri dev --no-default-features --features tauri` — `cargo tauri dev` ALREADY strips tauri/custom-protocol from defaults automatically, so the flag is redundant-but-harmless (not wrong).
- AC-8 code: schema.rs:267 + ui/defaults.ts:53 both flipped true→false; UI doc-comment says "MUST track Rust defaults". handle_close_to_tray gates on label=="control-panel" + only exits when !minimize. NO stray hardcoded `true` defaults repo-wide (schema.rs:439 + state_manager.rs:338 + settings.schema.test.ts:53 are serialization/validity tests, not defaults — correctly left as-is).
- AC-3/AC-5: platform_env (7 unit tests pass) + set_input_passthrough recursive query_tree+empty ShapeInput match STORY. AC-1 libwayland-dev in 4 apt lists (ci.yml 226/279/342/450); AC-2 corepack pnpm@10.33.4 in 3 blocks (370/482/496).

## FINDING (MEDIUM) — AC-8 test is FLAKY, STORY overclaims
- `tray::close_quits_app_when_minimize_to_tray_disabled` FAILS intermittently on this dev box.
- Stress: 2/12 fail via `cargo test` (~17%); also failed under nextest `ci` profile with ALL retries (TRY1+TRY2) failing → would go red in CI.
- Two failure modes: `left: None` (app didn't self-exit within 10s of WM_DELETE_WINDOW) and `left: Some(1)` (non-zero exit). Test asserts exit==Some(0).
- STORY §"Verification round 2" says it "passes under a self-launched Xvfb" — true SOMETIMES; the flake is not disclosed. The send_wm_delete x11rb approach itself is legit/defensible (better than xdotool windowclose which doesn't deliver ICCCM on KWin/XWayland — that honesty claim IS defensible).
- Root cause likely timing: WM_DELETE_WINDOW→CloseRequested→exit(0)→ExitRequested/Exit teardown race under headless software-GL; intermittently the close request doesn't realize a CloseRequested or teardown overruns 10s.
- Recommendation: STORY must disclose the flake; either harden the test (longer/dynamic timeout, retry the WM_DELETE send, or assert on log marker not just exit code) or mark known-flaky. Full 4-test tray suite via `cargo test --test tray` passed 3/3 (flake surfaces mainly in isolated/nextest-ci runs).
