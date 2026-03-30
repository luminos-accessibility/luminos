---
name: E03/003-004 Implementation Decisions
description: TrackingEngine and HotkeyMatcher implementation details, quality gate results, and deferred items for E03 Stories 003+004
type: project
---

E03/003 and E03/004 implemented in parallel 2026-03-29. All 3 quality gates approved for both stories.

**Deliverables:**
- `crates/luminos-core/src/tracking.rs` — TrackingEngine, TrackingConfig (dead zone, edge panning, smooth interpolation)
- `crates/luminos-core/src/hotkeys.rs` — HotkeyMatcher (7 bindings), dispatch_hotkey (zoom in/out/toggle/reset)
- `crates/luminos-core/src/lib.rs` — added tracking + hotkeys modules and re-exports
- `crates/luminos-core/Cargo.toml` — luminos-gpu changed from optional to required dependency
- `crates/luminos-platform/src/traits/input_monitor.rs` — Hash derive added to Modifiers
- 404 total workspace tests (58 new: 24 tracking + 34 hotkeys)

**Pre-applied changes (by team lead to avoid file conflicts):**
- lib.rs module declarations and re-exports
- Cargo.toml luminos-gpu non-optional
- Modifiers Hash derive

**Key implementation details:**
- TrackingConfig defaults: smoothing_factor=0.2, dead_zone_percent=0.2, edge_margin_percent=0.15
- TrackingEngine::update() 4-step algorithm: first-frame snap → dead zone check → edge panning → smooth_viewport_position()
- Dead zone scales with viewport_size/(2*zoom_level), not raw viewport_size
- Edge panning guarded against div-by-zero with edge_margin > 0.0 checks
- screen_bounds parameter accepted but unused (prefixed with _), reserved for Story 005 wiring
- HotkeyMatcher uses HashMap<(KeyCode, Modifiers), HotkeyAction> with 7 entries
- dispatch_hotkey: ZoomIn *= 1.5, ZoomOut /= 1.5, Toggle flips is_active, Reset to 2.0
- DEFAULT_ZOOM constant removed from hotkeys.rs (deviation) — delegates to StateManager::reset_zoom()
- ZOOM_STEP: f32 = 1.5 constant for multiplicative zoom steps

**Quality gate results:**
- Code reviewer: PASS WITH FINDINGS (2 MEDIUM mitigated, 3 LOW)
- QA engineer: APPROVED (404 tests, 30/30 ACs covered, fmt+clippy+security clean)
- Technical auditor: APPROVED (0 HIGH, 0 MEDIUM, 5 LOW)

**Deferred items (LOW, non-blocking):**
- F-001: _screen_bounds unused in TrackingEngine::update() (reserved for Story 005)
- F-002: clippy allow attribute scoped to entire impl block instead of just update() method
- F-003: No TrackingConfig field range validation (mitigated by smooth_viewport_position clamping)
- F-004: TOCTOU in dispatch_hotkey ZoomIn/ZoomOut (mitigated: single-threaded input processor in Story 005)
- F-005: HashMap::new() instead of with_capacity(7) in HotkeyMatcher::default() (init-time only)

**Why:** Both stories are pure-logic components (no I/O, no GPU, no platform deps) that were cleanly parallelizable with zero file conflicts thanks to pre-applied shared changes.

**How to apply:** Story 005 (E2E Pipeline Integration) wires both components together — TrackingEngine into the render loop, HotkeyMatcher into the input processing task. The _screen_bounds parameter becomes used in Story 005. If concurrent hotkey dispatch is ever needed, refactor update_zoom_level() to accept a closure.
