# Epic E05: Lens & Docked Magnification Modes

**Status:** NOT STARTED
**Roadmap Ref:** [tech-strategy/09-implementation-roadmap.md Section 5.1](../tech-strategy/09-implementation-roadmap.md)
**Phase:** Phase 1 — Core Magnification (Months 4-6)
**Estimated Duration:** 3 weeks (1.5 sprints) — per roadmap §5.1
**Started:** ---
**Completed:** ---
**Hard Dependencies:** E03 (input tracking, viewport engine) — DONE
**Soft Dependencies:** E04 (control panel, IPC, settings persistence) — DONE
**Primary Docs:** [03 — Rendering Pipeline](../tech-strategy/03-rendering-pipeline.md) Section 7, [02 — Platform Abstraction](../tech-strategy/02-platform-abstraction.md) Section 8.1 (EWMH struts), [05 — Control Panel](../tech-strategy/05-control-panel.md) Section 2.3

---

## Overview

E05 adds the two remaining magnification modes beyond the full-screen mode shipped in E04: a **lens** (a movable magnifying glass that follows the cursor, rectangular first then elliptical) and a **docked** strip (the magnified region pinned to a configurable screen edge at an adjustable size). It extends the existing wgpu rendering pipeline and `WindowManager` overlay to support multiple viewport geometries, wires three new IPC commands (`set_dock_edge`, `set_lens_size`, `set_lens_shape`) — reusing the `set_magnification_mode` command already shipped in E04 — adds mode-specific control-panel UI, and a mode-cycle keyboard shortcut (full-screen → lens → docked).

**User-perceivable value:** a low-vision user chooses how they magnify — full-screen immersion, a floating lens they move around, or a docked strip along one edge — and switches between them with a single keypress, with the control panel reflecting the active mode reactively.

E05 also closes the **RISK-002 residual** carried over from E04: the per-frame overlay flicker caused by the X11 unmap/remap self-capture mitigation. Lens and docked modes make that flicker far more visible (partial-screen overlays with the original desktop in view), so this epic replaces the unmap/remap approach with a flicker-free strategy.

## Success Criteria

Copied from the roadmap epic definition (doc-09 §5.1):

- [ ] All three magnification modes render correctly at zoom levels 1.5x, 5x, 10x, 20x
- [ ] Mode switching completes within 1 frame (no visible flicker)
- [ ] Lens mode tracks cursor smoothly (same P99 < 20ms target as full-screen)
- [ ] Docked mode resizes correctly when edge or size percentage changes
- [ ] Control panel UI updates reactively when mode changes via hotkey

**Epic-local additional criterion (RISK-002 residual, per planning decision 2026-06-12):**

- [ ] Self-capture exclusion no longer produces per-frame overlay flicker (unmap/remap replaced)

---

## Story Breakdown

This epic uses **vertical slicing**: each story delivers a fully functional, end-to-end feature — rendering + IPC + control-panel UI + tests — rather than horizontal layers. The sequence is docked-first (fixed → full), then rectangular lens, then elliptical lens, with the flicker-free self-capture work as an independent parallel story.

### Progress Summary

| # | Story | Status | Depends On | Notes |
|---|-------|--------|------------|-------|
| 001 | Docked mode core + mode-switch foundation | NOT STARTED | --- | Single edge (bottom), fixed size, `set_magnification_mode` IPC, mode toggle UI, full↔docked hotkey. Carries the OverlayMode-dispatch plumbing later stories extend. |
| 002 | Docked mode complete: 4 edges, size control, EWMH struts | NOT STARTED | 001 | All edges via `set_dock_edge`, configurable size 10-90%, EWMH strut reservation (D3), edge selector + size slider UI. |
| 003 | Rectangular lens mode + 3-mode cycle hotkey | NOT STARTED | 001 | Cursor-following rect lens, `set_lens_size` IPC, dimension sliders UI, completes full→lens→docked hotkey (D4). |
| 004 | Elliptical lens shape (shader mask) | NOT STARTED | 003 | Ellipse boundary via shader mask/stencil, `set_lens_shape` IPC, shape selector UI. Completes D1. Isolated-risk story (shader mask). |
| 005 | Flicker-free self-capture (RISK-002 residual) | NOT STARTED | --- | Replace unmap/remap with composite-pixmap / input-shape / root-region capture. No new UI. Parallelizable; recommended early so modes don't ship flickery. |

**Total Stories:** 5 | **Done:** 0 | **In Progress:** 0 | **Blocked:** 0

### Deliverable Traceability

Roadmap deliverables (doc-09 §5.1) → owning story:

| Deliverable | Owning Story | Notes |
|-------------|--------------|-------|
| D1 — Lens follows cursor, rect/ellipse shape | 003 (rect) + 004 (ellipse) | Split per planning decision: rect ships first, ellipse isolated. |
| D2 — Docked renders on all four edges | 002 | 001 ships bottom-edge only as the first functional slice. |
| D3 — EWMH struts reserve screen space | 002 | Verified via `xprop` strut hints. |
| D4 — Mode-cycling hotkey rotates all three modes | 001 + 003 | 001 ships full↔docked; 003 completes full↔lens↔docked once lens exists. |
| D5 — Control panel mode-specific controls show/hide | 001 / 002 / 003 / 004 | Built incrementally; each mode story ships its own reactive controls. |
| (RISK-002 residual) Flicker-free self-capture | 005 | Beyond the 5 roadmap deliverables; added per planning decision 2026-06-12. |

### Story Descriptions

#### 001 — Docked Mode Core + Mode-Switch Foundation
**Scope:** Deliver a minimal but fully functional docked mode — magnified content pinned to the bottom edge at a fixed size — together with the mode-switching plumbing the rest of the epic builds on.
**Key Deliverables:**
- `OverlayMode` dispatch in the render/viewport path and `WindowManager` overlay geometry (full-screen vs docked-bottom)
- Docked-bottom rendering at a fixed size, honoring the existing zoom level
- Wire the **existing** `set_magnification_mode` IPC command (shipped E04) through to the new docked-overlay dispatch — no new command here; regenerate `bindings.ts` only if a `specta::Type` changes
- Control panel: mode toggle (full-screen ↔ docked) with reactive show of the docked panel
- Mode-cycle hotkey (full-screen ↔ docked for now)
**Estimated Effort:** M
**Notes:** Resolve the `OverlayMode` vs `MagnificationMode` enum question here (see Shared Context → Open Architecture Questions); this decision constrains every later story. Touches `luminos-gpu` (viewport/render), `luminos-platform` (`WindowManager` overlay geometry), `luminos-core` (state/hotkeys), `luminos-app` (IPC), and `ui/`. Remember the `bindings.ts` CI diff gate.

#### 002 — Docked Mode Complete: Four Edges, Size Control, EWMH Struts
**Scope:** Promote docked mode to the full roadmap spec — all four edges, configurable size 10-90%, and EWMH strut reservation so other windows don't overlap the docked strip.
**Key Deliverables:**
- `set_dock_edge` IPC (signature already specced in doc-05 §2.3: `edge: DockEdge, size_percent: u32`) + edge selector UI (top/bottom/left/right)
- Configurable docked size (10-90%) with size slider UI — size is carried by the same `set_dock_edge` command's `size_percent` arg (no separate command)
- EWMH `_NET_WM_STRUT_PARTIAL` reservation via `x11rb` (D3), verified with `xprop`
- Reactive geometry recompute on edge/size change within 1 frame
**Estimated Effort:** M/L (densest story in the epic)
**Notes:** Test strut behavior on GNOME, KDE, and a tiling WM; fall back to plain window positioning where struts are unsupported (roadmap key risk). Multi-monitor is **out of scope** (Epic 16) — keep to the primary display; touches RISK-006. **Pre-authorized split:** if story design reveals >15 subtasks (edge × size × per-WM strut behavior + UI + IPC is a lot), split into 002a (four edges + size control + UI) and 002b (EWMH struts + per-WM fallback).

#### 003 — Rectangular Lens Mode + 3-Mode Cycle Hotkey
**Scope:** Deliver a fully functional rectangular lens that follows the cursor with configurable dimensions, and complete the three-mode cycle hotkey.
**Key Deliverables:**
- Lens rendering: rectangular region tracking the cursor, reusing the E03 tracking/viewport engine
- `set_lens_size` IPC (width/height) + lens dimension sliders UI
- Mode-cycle hotkey extended to full-screen → lens → docked (completes D4)
- Lens-mode control panel section, shown reactively
**Estimated Effort:** L
**Notes:** Lens tracking must hit the same P99 < 20ms target as full-screen. Shape is fixed to `Rectangle` here; `LensShape::Ellipse` is story 004.

#### 004 — Elliptical Lens Shape (Shader Mask)
**Scope:** Add the elliptical lens boundary, completing deliverable D1's "rectangle/ellipse" requirement.
**Key Deliverables:**
- Ellipse boundary via a WGSL shader mask (or stencil), per the roadmap mitigation
- `set_lens_shape` IPC (`Rectangle` | `Ellipse`) + shape selector UI
- Anti-aliased ellipse edge at all zoom levels
**Estimated Effort:** M
**Notes:** Isolated-risk story — if the shader-mask approach proves expensive, it can slip without blocking the rest of the epic (rect lens already shipped in 003). Builds on the lens render path from 003.

#### 005 — Flicker-Free Self-Capture (RISK-002 Residual)
**Scope:** Replace the E04 unmap/remap self-capture mitigation with a flicker-free strategy, eliminating the per-frame overlay flicker that lens/docked modes make highly visible.
**Key Deliverables:**
- Flicker-free self-capture exclusion: evaluate composite-pixmap capture, X11 input-shape exclusion, or root-region capture (see RISK-002 mitigation notes)
- Removal of the per-frame `x11rb::connect` cost (addresses RISK-039 on the same code path)
- Capture-correctness tests under the X11/Mesa CI jobs
**Estimated Effort:** M/L
**Notes:** Independent of the mode stories (no shared UI) — **parallelizable**, touches `luminos-platform::linux_x11` capture path only. Cross-references RISK-002, RISK-039. **Recorded sequencing decision (2026-06-12):** stories 001-004 MAY ship/demo with the known unmap/remap flicker before 005 lands; this is an accepted, tracked trade-off, not a regression — success criterion SC-2's "no visible flicker" clause is **not** considered closed for the epic until 005 is DONE. Log the carried flicker as a blocker entry (B00x) when the first mode story ships so it is not lost. Soft coupling: RISK-039's mitigation reuses the persistent x11 connection that the `WindowManager` (stories 001/002) also touches — coordinate the shared connection handling.

---

## Shared Context

### Architecture Decisions

- **Vertical slicing over horizontal layering** (planning, 2026-06-12): each story ships a working feature end-to-end (render + IPC + UI + tests), not a foundation/shared-parts layer. The first story (001) carries the mode-dispatch plumbing; later stories extend it. Rationale: every story closes with a demonstrable, usable capability.
- **Rect-first, ellipse isolated** (planning, 2026-06-12): rectangular lens ships in 003, elliptical lens in its own story (004), de-risking the shader-mask work flagged in the roadmap.
- **Flicker-free self-capture brought into E05** (planning, 2026-06-12): the RISK-002 residual is fixed here rather than deferred, because lens/docked overlays expose the flicker far more than full-screen did. See [risk register RISK-002](../tech-strategy/10-risk-register.md).

### Open Architecture Questions (resolve during story 001 design)

- **`OverlayMode` vs `MagnificationMode` reconciliation.** Two enums currently coexist:
  - `luminos_types::OverlayMode` (`crates/luminos-types/src/overlay.rs`) — rich, data-carrying (`FullScreen`, `Lens { width, height, shape }`, `Docked { edge, size_px }`), but **does not derive `specta::Type`** (not IPC-serializable today).
  - `luminos_types::MagnificationMode` (`crates/luminos-types/src/state.rs`) — flat (`FullScreen`, `Docked`, `Lens`), **derives `specta::Type`**, used in app state and IPC.
  doc-05 §2.3 already fixes the **wire format**: IPC carries the flat `MagnificationMode` (`set_magnification_mode(mode: MagnificationMode)`) and passes per-mode parameters through separate commands (`set_dock_edge(edge, size_percent)`, `set_lens_size(...)`, `set_lens_shape(...)`). So option (b) is effectively decided for the IPC boundary. The remaining question for story 001 is **internal**: does the render/state path adopt the rich `OverlayMode` (and reconstruct it from flat mode + params), keep them as parallel fields on `AppState`, or collapse the two enums? This internal representation constrains every later story — record the decision here once made.

### Key Type Definitions (existing — to be extended, not redefined)

```rust
// crates/luminos-types/src/overlay.rs
pub enum DockEdge { Top, Bottom, Left, Right }          // derives specta::Type
pub enum LensShape { Rectangle, Ellipse }               // derives specta::Type
pub enum OverlayMode {                                   // does NOT derive specta::Type (yet)
    FullScreen,
    Lens { width: u32, height: u32, shape: LensShape },
    Docked { edge: DockEdge, size_px: u32 },
}

// crates/luminos-types/src/state.rs
pub enum MagnificationMode { FullScreen, Docked, Lens }  // derives specta::Type
```

### Integration Points

- **Render/viewport:** `luminos-gpu` `Renderer`, `viewport.rs` (`compute_source_region`, `smooth_viewport_position`) — extend to compute lens/docked source regions and destination geometry. (E02)
- **Overlay window:** `luminos-platform` x11rb `WindowManager` — overlay geometry, click-through, and EWMH struts (`_NET_WM_STRUT_PARTIAL`). (E04)
- **Tracking:** `luminos-core` `TrackingEngine` — drives lens position from cursor. (E03)
- **Hotkeys:** `luminos-core` `HotkeyMatcher` / `dispatch_hotkey` — add the mode-cycle action. (E03)
- **State:** `luminos-core` `StateManager` (`ArcSwap<AppState>`) — active mode + parameters; lock-free render-thread reads. (E03)
- **IPC:** `luminos-app` `tauri_commands` / `ipc` — three new commands (`set_dock_edge`, `set_lens_size`, `set_lens_shape`); `set_magnification_mode` already exists from E04 and is reused. **Regenerate `ui/src/ipc/bindings.ts` and honor the CI diff gate** when commands/events/`specta::Type`s change. (E04)
- **Config:** `luminos-core` `ConfigManager` — persist active mode + per-mode parameters to `config.toml`. (E04)
- **UI:** `ui/` React control panel — mode-specific controls, reactive to mode-change events. (E04)

### Discovered Constraints

[To be filled in as stories complete.]

### Cross-Story Dependencies

- 002, 003 depend on the mode-dispatch plumbing and the enum decision from **001**.
- 004 depends on the lens render path from **003**.
- 005 is independent of all mode stories (capture-path only) and may run in parallel.

---

## Blockers & Issues Log

| ID | Date | Story | Description | Resolution | Status |
|----|------|-------|-------------|------------|--------|
| B001 | --- | --- | --- | --- | --- |

## Retrospective Notes

[Filled in when the epic is DONE.]
