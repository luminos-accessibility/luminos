---
name: ci-clippy-e2e-facts
description: Verified facts about Luminos CI clippy version skew and E2E range-input WebDriver behavior
metadata:
  type: reference
---

# CI clippy version skew + E2E range-input facts (verified 2026-06-14)

## clippy version skew (load-bearing)
- Dev box clippy = 0.1.93; CI = 1.96.0. VERIFIED: `map_unwrap_or` (pedantic, on Result) does NOT fire on 0.1.93 for `.map(|s| s.success()).unwrap_or(false)` but DOES on CI 1.96. So local `cargo clippy` is NOT a sufficient gate for pedantic-lint drift — only the next CI run is definitive proof. Commit messages stating this are accurate.
- `clippy::map_unwrap_or` targets `.map(f).unwrap_or(g)` and `.map(f).unwrap_or_else(g)`. It does NOT fire on `.map(f).unwrap_or_default()` (different lint family). Two such `.unwrap_or_default()` chains in tests/common/mod.rs (lines 353-354, 375-376) are safe — confirmed not a "third round" risk.
- `Result::is_ok_and(f)` is behavior-identical to `.map(f).unwrap_or(false)` for bool f: Err→false, Ok(s)→f(s). Verified by case analysis.
- `--all-targets` lints test helpers (tests/common/mod.rs) separately from src; a fix that only touches src/app.rs (commit 182d1f2) leaves test-helper lint hits for a later round (commit b6ccc4f sibling fix). Pattern: fix the SAME lint in BOTH src and tests/ at once.

## WebDriver Element Clear / range input (IMPRECISION found)
- Per W3C WebDriver spec, `input type="range"` IS "editable" (Range is explicitly in the editable type-state list alongside Text/Number/Color/File Upload) AND IS "resettable" (ALL `input` elements are resettable per HTML spec). So a SPEC-CONFORMANT Element Clear should NOT reject a range input with "invalid element state".
- Therefore the e2e/support/ipc.ts `driveZoomSlider` docstring claim that "the spec only allows clearing editable/resettable text-like controls, and a range input is neither" is FACTUALLY WRONG about the spec. The observed WebKitWebDriver rejection (if real) is an implementation deviation, NOT spec-mandated. Fix still correct (sidesteps clear entirely), only the narrative overstates spec certainty.

## React controlled-input driving (CORRECT technique)
- React 19.2.6. Driving a controlled `<input type=range>` from outside React: use the NATIVE prototype value setter (`Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(el, v)`) to bypass React's `_valueTracker`, then `dispatchEvent(new Event('input',{bubbles:true}))`. This is the canonical pattern; React onChange listens for `input` on range inputs. driveZoomSlider implements it correctly.
- ZoomLevelSlider: ZOOM_MIN=1.5, ZOOM_MAX=20, ZOOM_STEP=0.5; default seeded zoom = 2.0 (config/schema.rs:119). D2 TARGET_ZOOM=8 is in-range, on-step, ≠ default → assertion non-vacuous.
