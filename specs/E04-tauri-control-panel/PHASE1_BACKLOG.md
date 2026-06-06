# Epic E04 -- Phase-1 Code-Polish Backlog

**Filed:** 2026-06-05 (Epic E04 close-out)
**Status of every item below:** **Phase-1, non-blocking.** None of these block the E04 DONE status or Phase 0 exit. They are cleanups/hardening surfaced during E04 implementation, to be scheduled into Phase 1 (or the relevant downstream epic) as capacity allows.

This file is the durable home for the carry-forward items noted across the E04 stories' SUBTASKS/IMPLEMENTATION_NOTES and the HIGH_LEVEL_PLAN Retrospective. Risk-register items (RISK-039, RISK-040) are cross-referenced where relevant.

---

## Backlog Items

| # | Source story | Item | Why (Phase-1 rationale) |
|---|--------------|------|-------------------------|
| 1 | 002 | **Route `window.rs` property-mutation errors via `WindowError::PropertyFailed`.** The X11 `WindowManager` backend currently surfaces some X11 property-set failures through generic/coarse error paths; route them through the dedicated `WindowError::PropertyFailed` variant for precise diagnostics. | Better error attribution for overlay geometry/visibility failures; no behavior change. |
| 2 | 003 | **Typed `SurfaceErrorKind` discriminant for `OverlayGpu` surface-error classification.** Replace the current string/coarse classification of `wgpu::SurfaceError` in `OverlayGpu` with a typed `SurfaceErrorKind` discriminant so lost/outdated/oom/timeout surfaces are handled (and tested) distinctly. | Makes surface-recovery logic testable and explicit; currently the classification is implicit. |
| 3 | 005 | **Extract a pure `compute_emit_delta()` for unit-testing the event delta logic.** The render loop's `(zoom, mode)` delta computation that decides whether to emit `zoom_changed`/`mode_changed` is currently inline in the `MainEventsCleared` arm; extract it as a pure function so the delta logic can be unit-tested without the loop. | The emit-on-delta logic is contract-critical (panel sync) but only covered indirectly today. |
| 4 | 002 | **`WindowManager::raw_*_handle()` trait cleanup (always `None`).** The X11 backend's `raw_window_handle()` / related accessors always return `None` (AD-3: the surface is sourced by `luminos-app`'s `OverlayGpu` from the owned overlay window, not the platform layer). The trait methods are now dead weight -- either drop `raw_*_handle()` from the `WindowManager` trait or formally relocate surface-sourcing into the app layer. | Removes a misleading trait surface; reduces the chance a future backend wires a redundant/conflicting surface source. |
| 5 | 007 | **Dedup the `CONTROL_PANEL_LABEL` string literal + document the dead `MENU_ID_QUIT` arm.** The `"control-panel"` window label is repeated as a literal across modules -- hoist to one shared `const`. Separately, the `MENU_ID_QUIT` arm in `tray::handle_menu_event` is currently dead (the predefined quit menu item handles quit directly) -- either wire a custom quit item to it or document the arm as intentionally inert. | Single source of truth for the window label; removes confusion about which quit path is live. |
| 6 | 002/003 | **RISK-039: per-frame `x11rb::connect` in `XcbCapture` self-capture exclusion.** `XcbCapture::{unmap,remap}_excluded_windows` open a fresh `x11rb::connect(None)` per captured frame on the 60 fps hot path (DC-12). Cache one persistent connection (or reuse the `X11WindowManager`'s), bound to the captured screen's display rather than ambient `$DISPLAY`. | Per-frame X11 handshake is a latency/correctness risk against the 8 ms capture budget. See [10 -- Risk Register](../tech-strategy/10-risk-register.md) RISK-039. |
| 7 | 005 | **AD-5 origin-tagging for event emission.** `zoom_changed`/`mode_changed` are emitted from the render loop on a `(zoom, mode)` delta regardless of whether the change originated from the UI (a command echo) or from engine input (a hotkey). Tag the change origin so the panel can suppress its own command echoes and only react to genuinely engine-originated changes (AD-5). | Avoids redundant UI churn / potential optimistic-update fights when a command's own write echoes back as an event. |

---

## Related (already captured elsewhere -- listed for completeness, not duplicated here)

- **RISK-040 (uninterruptible X11 input-monitor shutdown):** the XI2 monitor's `wait_for_event()` is uninterruptible, so input threads are detached + process-reaped on shutdown; the real fix is `poll_for_event()` + a stop-flag with a cooperative join. Tracked as [10 -- Risk Register](../tech-strategy/10-risk-register.md) RISK-040, not a backlog row above.
- **RISK-002 flicker-free self-capture:** the unmap/remap exclusion produces visible per-frame flicker under tao/GTK3; a flicker-free strategy is a Phase-1 follow-up tracked under RISK-002 (Mitigating).
- **`test-e2e` first green CI run:** the `tauri-driver` E2E suite is authored/wired/`tsc`-typechecked; confirming a non-flaky first green CI run is a CI-runtime verification item (see the epic's Success Criteria), not a code-polish item.
- **Tech-strategy doc reconciliation (winit→tao, nested `AppState`):** completed at this E04 close-out (doc-01 §3.3/§6.5, doc-05 §4.1, roadmap §4.4, risk register). Recorded here only so the carry-forward chain is auditable.
