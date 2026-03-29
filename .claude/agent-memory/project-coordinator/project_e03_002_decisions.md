---
name: E03/002 Implementation Decisions
description: StateManager, LuminosEvent, AppState mouse_position — quality gate results and deferred items
type: project
---

E03/002 implementation completed 2026-03-29. All 3 quality gates approved (code-reviewer, QA, technical-auditor).

**Deliverables:**
- `crates/luminos-core/src/state_manager.rs` — StateManager wrapping Arc<ArcSwap<AppState>> with rcu() mutations
- `crates/luminos-core/src/event.rs` — LuminosEvent enum (StateChanged, RequestExit)
- `crates/luminos-core/src/state.rs` — mouse_position: ScreenPoint field added to AppState
- 24 new tests, 343 total workspace tests (57 luminos-core)

**Key implementation details:**
- StateManager accepts Arc<ArcSwap<AppState>> externally (FR-6), does NOT own EventLoopProxy
- All mutations use rcu() — retries on contention, never blocks readers
- Zoom constants: MIN_ZOOM=1.5, MAX_ZOOM=20.0, DEFAULT_ZOOM=2.0 (public)
- load() returns arc_swap::Guard (lock-free, <100ns in release, <500ns in debug)
- Benchmark test uses conditional threshold: 500ns debug / 100ns release (documented deviation)
- #[must_use] on load() and inner() — good addition beyond spec
- Concurrent writer test runs 100 iterations with barriers for race coverage

**Deferred items (LOW, non-blocking):**
- F-001: Broken rustdoc link on state_manager.rs (Guard not in scope for docs)
- P-001: Benchmark only strict in release mode; CI runs debug
- P-002: update_zoom_level does not guard against NaN input (IEEE 754 clamp pass-through)

**Why:** Validated through 3 independent quality gates. All findings LOW severity, acceptable for Phase 0.

**How to apply:** E04 IPC boundary should validate zoom_level is not NaN before calling update_zoom_level (P-002). A release-mode benchmark CI job would close the NFR-1 verification gap (P-001).
