---
name: e05-hlp-audit
description: Audit of E05 (Lens & Docked Modes) HIGH_LEVEL_PLAN.md — verdict CHANGES REQUIRED, IPC command-count drift
metadata:
  type: project
---

# E05 HIGH_LEVEL_PLAN.md Audit (2026-06-12, NOT STARTED epic)

Verdict: CHANGES REQUIRED (no blocking architecture errors; 2 MEDIUM accuracy defects + minors). Type claims, risk refs, deps, SC verbatim all PASS.

## Verified ground truth (reusable facts)
- crates/luminos-types/src/overlay.rs: OverlayMode (FullScreen | Lens{width:u32,height:u32,shape:LensShape} | Docked{edge:DockEdge,size_px:u32}) derives Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize but NOT specta::Type. DockEdge + LensShape DO derive specta::Type. Plan's quoted signatures EXACT.
- crates/luminos-types/src/state.rs: MagnificationMode { FullScreen, Docked, Lens } flat, derives specta::Type. state_enums_implement_specta_type test asserts it.
- doc-05 §2.3 line 184: `set_dock_edge(edge: DockEdge, size_percent: u32)` — DOCKED SIZE IS ALREADY FOLDED INTO set_dock_edge (size_percent 10-90%). doc-05 line 1116 maps the docked-size slider to set_dock_edge. So there is NO open question / no need for a new set_dock_size; roadmap "four commands" already covers size.
- `set_magnification_mode` ALREADY SHIPPED in E04 (one of the 7 Phase-0 commands; crates/luminos-app/src/tauri_commands.rs:170, ipc.rs:42). E05 adds only 3 genuinely-new commands (set_dock_edge, set_lens_size, set_lens_shape), not 4.
- RISK-002 (Self-Capture Infinite Feedback Loop, Mitigating, score 9): residual flicker + composite-pixmap/input-shape/root-region wording accurate. RISK-039 (Per-frame x11rb::connect, Open, P1, score 6) accurate. RISK-006 (Multi-Display/HiDPI, Open, score 6) accurate.
- E03 status DONE, E04 status DONE (2026-06-05). Both confirmed.
- SC block (plan lines 26-30) is byte-identical to roadmap §5.1 lines 483-487 (diff IDENTICAL).
- Primary docs all resolve: doc-03 §7 (Zoom Mode Rendering: 7.1 Full/7.2 Lens/7.3 Docked/7.4 Transitions), doc-02 §8.1 (EWMH _NET_WM_STRUT_PARTIAL), doc-05 §2.3 (IPC catalogue).

## Defects found
- MEDIUM: Plan Overview (line 16) + Integration Points (line 159) say "four new IPC commands" but set_magnification_mode already shipped in E04. Only 3 are new.
- MEDIUM: Open Architecture Question "Docked-size IPC command" (line 134) is a false premise — set_dock_edge already carries size_percent per doc-05 §2.3 L184 / L1116. Should reference existing signature, not invent set_dock_size.
- LOW: Story 002 line 84 repeats the same "not in the roadmap's listed four" framing (same root cause).
