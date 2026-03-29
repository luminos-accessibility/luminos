---
name: E03 Audit Findings
description: Technical audit results for E03 Input Tracking & Interactive Magnification spec phase (2026-03-29)
type: project
---

E03 spec phase APPROVED WITH FINDINGS (2026-03-29). 16 files, 7 findings (0 BLOCKING, 3 ADVISORY, 4 INFO), 6 pointers.

**Key findings:**
- F-001: "ZoomText convention" claim is MISLEADING. ZoomText uses Caps Lock-based shortcuts, not Ctrl+Alt. Luminos chose Ctrl+Alt (reasonable) but attribution is wrong.
- F-002: Story 004 SUBTASKS T002 says Ctrl+Alt+M for toggle, should be Ctrl+Alt+F1 (matches STORY/DESIGN/HLP).
- F-003: Story 005 SUBTASKS T003 fabricates `modifiers` field on MouseButton and Scroll InputEvent variants (field doesn't exist).
- F-004: TrackingEngine (luminos-core) imports `smooth_viewport_position` from luminos-gpu which is only an optional dep.
- F-005: x11rb 0.13 uses newtype XIEventMask with UPPER_SNAKE_CASE constants, not enum. Device::ALL_MASTER is correct type-safe API.
- F-007: HLP Discovered Constraints says "prefer try_recv()" but DESIGN.md correctly uses blocking_recv() for dedicated thread.

**Key pointers:**
- P-001: Ctrl+Alt+F1 conflicts with Linux VT switching -- may be intercepted before X11 delivers it.
- P-005: GetKeyboardMapping insufficient for non-Latin keyboard layouts in E07.
- P-006: .expect() in InputProcessingTask::spawn() violates CLAUDE.md no-expect rule.

**Why:** Three ADVISORY findings need text corrections before implementation. All other items are notes for implementing agents.
**How to apply:** Check these findings when re-auditing any E03 revisions. If implementing agent diverges from spec, verify against these audit findings.
