# E04 NO_AT_BRIDGE CI test-harness fix audit (2026-06-14)

Working-tree change (uncommitted): crates/luminos-app/tests/common/mod.rs +10 lines
(`.env("NO_AT_BRIDGE","1")` in RunningApp::spawn @ line 206) + CLAUDE.md doc line 177.
CI job "App Shell Tests (Tauri+Xvfb)", run 27519949143.

## Verdict: APPROVE WITH ONE LOW finding (imprecise comment wording)

- Root cause SOUND: NO_AT_BRIDGE prevents atk_bridge_init during GTK init; without
  session/a11y D-Bus the bridge stalls trying to reach org.a11y.Bus. Confirmed via
  web (freedesktop atk-bridge docs).
- F-LOW (IMPRECISE): the in-code comment + CLAUDE.md say the stall happens "while
  building the control-panel WebKitWebView". But the measured gap is config-load
  ("using default settings", luminos_core, BEFORE the Builder) -> "managed_state_ok"
  (probe_managed_state, INSIDE .setup() @ app.rs:132). Tauri runs setup BEFORE the
  config-declared control-panel webview is created (webview creation is during
  .build() AFTER setup returns). So the stall is GTK/atk-bridge INIT during Tauri's
  gtk setup, not WebKitWebView construction. Same root cause + same fix; only the
  attribution sentence is imprecise. Fix is still correct & well-placed.
- Fix placement: ALL 19 integration #[test] fns (10 files) spawn via RunningApp::spawn.
  Only direct Command::new in tests are xdotool/Xvfb/picom/sh. app_binary() referenced
  ONLY inside spawn. So single chokepoint => "covers all 7 fail + 4 flaky" is structural.
- Non-regression CONFIRMED: NO_AT_BRIDGE absent from all crates/*/src. main.rs only
  sets force_x11_backend (GDK_BACKEND=x11), never NO_AT_BRIDGE. Production a11y intact.
- Bisection (NO_AT_BRIDGE=1 OR dbus-run-session each removes stall) is valid: both
  address the SAME missing-bus root cause from different angles (kill the bridge vs
  provide a bus), so convergent evidence, not circular. "Sole cause" is reasonable
  given gap->0 both ways, though strictly it proves "the a11y-bus reach is the stall",
  which IS the bridge.
- Limitation honesty (claim 5) ACCURATE: can't repro the 20s-timeout FAILURE on a fast
  box (only the 9s stall) — legit, timeout is wall-clock+runner-speed dependent.
  ipc_hotkey + capture_path_wired fail locally on Wayland-hosted Xvfb: confirmed both
  depend on xdotool key/mouse INJECTION (ctrl+alt+equal etc.), which is the documented
  dev-box limit; they pass the xdotool-available gate but injection doesn't land.

## Tauri lifecycle fact (verified 2026-06-14)
Tauri 2.x (=2.11.2): config-declared windows' webviews are created during
Builder::build()/run, AFTER the .setup() closure returns. GTK init (where atk-bridge
loads) happens earlier, during Builder setup, before the setup closure. So logs emitted
inside .setup() (managed_state_ok) precede control-panel webview construction.
