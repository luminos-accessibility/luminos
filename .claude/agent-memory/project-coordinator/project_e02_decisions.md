---
name: E02 Decomposition Decisions
description: Key decisions and findings from E02 epic decomposition - xcap RGBA, self-capture unmap/remap, story structure, audit findings
type: project
---

E02 (X11 Screen Capture & GPU Magnification) decomposition completed 2026-03-28 with the following decisions and findings:

- **Story structure:** 5 stories (001-X11 Capture, 002-Overlay Window, 003-GPU Textures, 004-Shaders, 005-Render Loop & CI)
- **Dependency graph:** 001 ∥ 002 → 003/004 → 005
- **Spec approval:** Same as E01 — specs auto-approved, one commit per story
- **Parallelism:** 2-3 parallel spec writers after HIGH_LEVEL_PLAN

**Critical research findings (changed tech strategy assumptions):**
- xcap 0.9.3 returns **RGBA** (not BGRA as doc-03 assumed for X11). Internal conversion from X11 native format.
- RISK-002 mitigation #2 in risk register is **incorrect** — `xcb_composite_redirect_window` does NOT exclude override-redirect windows. Correct approach: **unmap/remap cycle** around each capture call.
- xcap creates a new XCB connection per capture call (not pooled). Performance concern at 60fps.
- Display change events not in xcap — must use x11rb RandR subscription separately.
- `ScreenCapture` trait extended with `set_excluded_windows(&mut self, window_ids: &[u64])` — breaking E01 change.

**User-driven scope changes:**
- Bicubic shader moved from E06 to E02 (both bilinear + bicubic implemented)
- DockEdge/LensShape type unification addressed in E02 (not deferred to E04)
- Self-capture prevention (RISK-002) included in E02

**Audit findings resolved:**
- F-001: D5 satisfied by single-buffer sequential pipeline (double-buffer deferred to Phase 1)
- F-002: Story 002 scoped to FullScreen only (docked/lens are E05)
- F-009: set_excluded_windows() is a trait-level method (not struct-level)

**Why:** These decisions shape all E02 implementation work and affect future epics (E05 docked/lens, E06 bicubic removal, E08 XShm optimization).

**How to apply:** Implementation agents should read the E02 HIGH_LEVEL_PLAN.md Shared Context and Discovered Constraints sections. The RGBA finding and unmap/remap approach are critical for Stories 001 and 004.
