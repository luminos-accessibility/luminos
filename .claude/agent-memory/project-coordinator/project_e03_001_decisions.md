---
name: E03/001 Implementation Decisions
description: X11InputMonitor implementation details, quality gate results, and deferred improvements for E03 Story 001
type: project
---

E03/001 implementation completed 2026-03-29. All 3 quality gates approved (code-reviewer, QA, technical-auditor).

**Deliverables:**
- `crates/luminos-platform/src/linux_x11/input.rs` — X11InputMonitor struct, XI2 event loop, event translation
- `crates/luminos-platform/src/linux_x11/keymap.rs` — 89-keysym mapping + modifier extraction
- 36 new tests (17 keymap + 19 input), 311 total workspace tests
- 7 integration tests gated behind `ci_platform_tests` (Xvfb + xdotool)

**Key implementation details:**
- Two RustConnection instances (query + monitor thread) to avoid lock contention on wait_for_event()
- GetKeyboardMapping for keysym resolution (core protocol, no xkb)
- Manual Debug impl on X11InputMonitor (RustConnection doesn't derive Debug)
- unsafe blocks for set_var/remove_var in tests (Rust 2024 edition requirement)
- fp1616 to i32 via `>> 16` shift
- Thread named "luminos-input-x11"
- root_x/root_y used instead of event_x/event_y (equivalent on root window, more correct)

**Post-review fixes applied (same session):**
- F-001 FIXED: Scroll button 4/5 release now returns None (not phantom MouseMoved)
- F-002 FIXED: subscribe_input_events() doc-comment documents multiple-call behavior
- F-003 FIXED: Horizontal scroll buttons 6/7 mapped to Scroll events with delta_x
- F-004 FIXED: DESIGN.md updated event_x/event_y → root_x/root_y
- F-005 FIXED: Logging uses single quotes around dynamic values per CLAUDE.md
- F-006 FIXED: Send+Sync static assertion (was Send-only)
- +6 tests added (154 total luminos-platform, 343 workspace)

**Remaining deferred items (not fixed, acceptable for Phase 0):**
- P-001: Keyboard mapping fetched once (no MappingNotify refresh on layout change)
- P-002: Monitor thread JoinHandle dropped (detached), no explicit join on drop
- P-003: rdev still in workspace deps despite not being used in this story

**Why:** These decisions were validated through 3 independent quality gates. All findings were LOW severity and explicitly acceptable for Phase 0.

**How to apply:** Future stories touching input monitoring should check these deferred items. P-001 (keyboard layout refresh) may matter for E07 configurable keybindings. P-003 (rdev removal) is a workspace-level decision.
