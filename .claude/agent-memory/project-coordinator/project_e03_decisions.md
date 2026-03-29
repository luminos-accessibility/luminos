---
name: E03 Spec Decisions
description: Key decisions from E03 (Input Tracking & Interactive Magnification) spec decomposition phase
type: project
---

E03 spec phase completed 2026-03-29. All 16 artifacts approved by code-reviewer and technical-auditor.

**Key decisions:**
- x11rb (XInput2) instead of rdev for input monitoring. Rationale: single-maintainer risk (RISK-031), more control over X11 APIs.
- Toggle shortcut is **Ctrl+Alt+8** (GNOME magnifier convention). Ctrl+Alt+F1 was rejected because it conflicts with Linux VT switching (kernel-level intercept). ZoomText actually uses Caps Lock-based shortcuts, not Ctrl+Alt.
- Full shortcut table: Ctrl+Alt+= (zoom in), Ctrl+Alt+- (zoom out), Ctrl+Alt+8 (toggle), Ctrl+Alt+0 (reset)
- Viewport math stays in luminos-gpu, tracking engine in luminos-core. Creates core→gpu dependency (safe today, documented as future risk).
- ArcSwap rcu() for state writes, load() for reads. blocking_recv() on dedicated input processing thread.
- Wording changed from "ZoomText convention" to "accessibility tool convention" per technical audit finding.

**Why:** These decisions were iterated through user questions, code-reviewer findings (existing HotkeyAction/KeyBinding types, tokio::sync::mpsc async bridge), and technical-auditor findings (VT switching conflict, ZoomText actually uses Caps Lock shortcuts, blocking_recv vs try_recv).

**How to apply:** Any implementation work on E03 should read the HLP Shared Context section and these decisions. The toggle shortcut went through 3 changes (M → F1 → 8) so verify all specs reference Ctrl+Alt+8.
