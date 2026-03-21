---
name: rust-test-driven-development
description: >
  Guide for performing Test-Driven Development in Rust within the Luminos project's
  Spec-Driven Development methodology. Use this skill whenever an agent is implementing
  subtasks from a SUBTASKS.md file, writing Rust tests, executing red-green-refactor
  cycles, working on any Rust implementation story, or needs guidance on Rust testing
  patterns. Trigger when you see phrases like "implement story", "work on subtasks",
  "write tests first", "do TDD", "red-green-refactor", "implement T001", or any
  variation of test-driven Rust development. If Rust code is being written as part of
  a story, this skill should be active. Also use when reviewing whether Rust tests
  follow project conventions, or when an agent needs to choose between test types
  (unit vs integration vs property-based).
---

# Rust Test-Driven Development

This skill guides AI agents through the TDD workflow for Rust implementation stories in the Luminos project. It translates the spec-driven development methodology (specs/README.md) into concrete, step-by-step actions for each subtask.

The core idea: tests are a **specification contract**. When an AI agent writes tests first from acceptance criteria, the tests constrain the implementation to match the design — not the other way around. This is especially important for AI-generated code, where the implementation can drift from the spec if there's no test anchoring it.

---

## Before You Start

When you pick up a story for implementation, read these files in order:

1. The epic's `HIGH_LEVEL_PLAN.md` — understand context, shared types, decisions
2. Your story's `STORY.md` — understand requirements and acceptance criteria
3. Your story's `DESIGN.md` — understand architecture, APIs, and testing strategy
4. Your story's `SUBTASKS.md` — find your current task (first unchecked item)

Do not read other stories' files. The HIGH_LEVEL_PLAN.md Shared Context section contains everything you need from earlier stories.

---

## The TDD Cycle

Every implementation task in SUBTASKS.md follows three phases. Execute them **sequentially** — complete each phase fully before moving to the next.

### Phase 1: Red — Write Failing Tests

The goal is to express the expected behavior as tests **before any implementation exists**. This is the most important phase for AI agents because it forces you to think about behavior, not code.

**Step-by-step:**

1. Read the task's "Traces to" field to find which acceptance criteria (AC-X.X) and functional requirements (FR-X) this task covers.

2. Read the DESIGN.md Testing Strategy table to find the test type and verification method for each AC.

3. Translate each acceptance criterion into one or more test functions using the Given-When-Then → Arrange-Act-Assert mapping:

   ```rust
   // AC-1.1: Given a valid X11 display, when capture is requested,
   //         then a CaptureFrame with correct dimensions is returned.

   #[test]
   fn screen_capture_x11_valid_display_returns_frame() {
       // Arrange — set up the "Given" precondition
       let config = generate_test_capture_config();
       let capture = MockScreenCapture::new(config);

       // Act — perform the "When" action
       let frame = capture.capture_frame().unwrap();

       // Assert — verify the "Then" outcome
       assert_eq!(frame.width, config.width);
       assert_eq!(frame.height, config.height);
       assert!(!frame.data.is_empty());
   }
   ```

4. Write the test functions. Follow these naming and placement conventions:
   - **Names:** Hierarchical prefixes for `cargo nextest run` filtering.
     Pattern: `{module}_{behavior}_{scenario}` — e.g., `screen_capture_x11_valid_display_returns_frame`
   - **Placement:** Unit tests go in `#[cfg(test)] mod tests` at the bottom of the file being tested. Integration tests go in `tests/` directory.
   - **Fixtures:** Use `generate_test_` prefixed functions. Place them in the same module as the type they construct. Make them `pub` and parametrizable.

5. **Verify the Red phase.** Run:
   ```bash
   cargo nextest run -E 'test(~your_test_name_prefix)'
   ```
   The tests should **fail** (compile errors count as failures). If they pass, something is wrong — you may be testing existing behavior rather than new behavior, or the test is tautological.

   Record the failure output. This confirms your tests are actually testing something.

**Mental discipline:** During this phase, think only about **what** the code should do, not **how** it will do it. If you catch yourself designing the implementation while writing tests, stop. The test should describe the interface contract from DESIGN.md, not the internal structure.

### Phase 2: Green — Minimal Implementation

The goal is the simplest code that makes all Red-phase tests pass. Nothing more.

**Step-by-step:**

1. Implement just enough code to make the tests pass. Resist the urge to add error handling for cases you haven't tested, optimize performance, or build abstractions you'll "need later."

2. Follow the type signatures and APIs from DESIGN.md exactly. These are contracts that other stories depend on.

3. Run the tests:
   ```bash
   cargo nextest run -E 'test(~your_test_name_prefix)'
   ```
   All tests from the Red phase should now pass. If any fail, fix the implementation (not the tests) until they pass.

4. Run the linter — it catches issues the tests won't:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```
   Fix any warnings before proceeding.

5. Check formatting:
   ```bash
   cargo fmt -- --check
   ```

**The key rule:** if you find yourself writing code that isn't needed to pass a test, either write a test for it first (go back to Red) or leave it for the Refactor phase.

### Phase 3: Refactor — Clean Up

The goal is to improve code quality while keeping all tests green. The tests are your safety net.

**Step-by-step:**

1. Look for these improvement opportunities:
   - Duplicated code that can be extracted into helpers
   - Naming that doesn't match the project's conventions (CLAUDE.md)
   - Nesting deeper than 2 levels (use early-return guards)
   - `unwrap()` or `expect()` in production code (replace with `?` propagation)
   - Eager evaluation where lazy would be better (`.unwrap_or()` → `.unwrap_or_else()`)
   - Manual loops that could be iterator chains

2. Make one refactoring change at a time. After each change, run:
   ```bash
   cargo nextest run -E 'test(~your_test_name_prefix)'
   ```
   If tests break, revert the change and try a different approach.

3. Run the full check suite when done:
   ```bash
   cargo nextest run && cargo clippy --all-targets -- -D warnings && cargo fmt -- --check
   ```

---

## After Each Task

1. **Check off** the completed items in SUBTASKS.md (Red/Green/Refactor checkboxes).
2. **Write Completion Notes** — what you actually implemented, files created/modified, test names, any deviations from the plan.
3. **Update epic Shared Context** if this task produced knowledge other stories need (new public types, discovered constraints, architecture decisions).
4. **Update the Progress Summary table** in SUBTASKS.md.
5. At **phase boundaries** (between Phase 1→2, 2→3, etc. in the SUBTASKS.md), run the full test suite to confirm all previous tasks still pass:
   ```bash
   cargo nextest run
   ```

---

## Test Writing Patterns

### Choosing the Right Test Type

| Situation | Test Type | Location | Feature Gate |
|-----------|-----------|----------|-------------|
| Pure logic, state machines, data structures | Unit test | `#[cfg(test)] mod tests` in source file | None |
| Cross-module behavior | Integration test | `tests/` directory | None |
| Needs real platform API (X11, GPU) | Integration test | `tests/` directory | `#[cfg(feature = "integration_tests")]` |
| Shader correctness | Shader test | `tests/` directory | `#[cfg(feature = "integration_tests")]` |
| Algorithmic invariants | Property-based | Unit test module | Uses `proptest` |
| Multiple input variations | Parameterized | Unit test module | Uses `rstest` |
| Serialization stability | Snapshot | Unit test module | Uses `insta` |
| Doc examples that must compile | Doc test | `///` comments | Run via `cargo test --doc` |

### Test Generators

Every test type or struct that appears in multiple tests should have a generator:

```rust
#[cfg(test)]
pub(crate) fn generate_test_capture_config() -> CaptureConfig {
    CaptureConfig {
        display_id: 0,
        width: 1920,
        height: 1080,
        pixel_format: PixelFormat::Bgra8,
    }
}

// Parametrizable variant for edge cases
#[cfg(test)]
pub(crate) fn generate_test_capture_config_with_size(w: u32, h: u32) -> CaptureConfig {
    CaptureConfig {
        width: w,
        height: h,
        ..generate_test_capture_config()
    }
}
```

### Testing Trait Implementations

Luminos has six platform traits (`ScreenCapture`, `FocusTracker`, `TtsEngine`, `WindowManager`, `InputMonitor`, `AudioOutput`). These are the natural mock boundaries.

**Prefer hand-written fakes over mockall for most cases.** Fakes are simpler implementations that exercise real logic. Use mockall only when you need to verify that a specific method was called with specific arguments (interaction verification).

```rust
// A fake — preferred for most testing
#[cfg(test)]
struct FakeScreenCapture {
    frames: Vec<CaptureFrame>,
    error_factory: Option<Box<dyn Fn() -> CaptureError>>,
}

#[cfg(test)]
impl FakeScreenCapture {
    fn new(frames: Vec<CaptureFrame>) -> Self {
        Self { frames, error_factory: None }
    }

    fn with_error(factory: impl Fn() -> CaptureError + 'static) -> Self {
        Self { frames: vec![], error_factory: Some(Box::new(factory)) }
    }
}
```

For detailed patterns on test doubles, async testing, and crate-specific recipes, read `references/test-patterns.md`.

### What to Test vs. What the Compiler Guarantees

Focus your tests on things Rust's type system doesn't catch:

**Test these (the compiler won't catch them):**
- Business logic correctness (wrong arithmetic, off-by-one errors)
- Boundary conditions (cursor at display edge, zoom at min/max)
- Error variant correctness (right error returned for each failure mode)
- State machine transitions (valid and invalid sequences)
- Integration contracts (subsystem boundaries behave correctly together)

**Skip these (the compiler already catches them):**
- Type mismatches (passing wrong types to functions)
- Null handling (Option forces exhaustive matching)
- Data races (Send/Sync enforcement)
- Missing match arms (exhaustive pattern matching)
- Lifetime violations

---

## Anti-Patterns to Avoid

These are the most common ways AI agents get TDD wrong. Being aware of them is the first defense.

1. **Writing tests and implementation simultaneously.** This defeats TDD's purpose — tests end up testing the implementation you already wrote rather than the spec. Complete all test writing before thinking about implementation.

2. **Tests that always pass.** If a test can't fail, it tests nothing. Always verify the Red phase — your tests should fail before the implementation exists.

3. **Excessive mocking.** When everything is mocked, tests verify mock behavior, not real behavior. Mock only at the six trait boundaries. For internal modules, use real implementations.

4. **Testing implementation details.** Test behavior ("given this input, I get this output") not structure ("this function calls that function"). Implementation-coupled tests break during refactoring.

5. **Modifying tests to make them pass.** If a test fails during the Green phase, fix the implementation. Only modify a test if the test itself has a bug (wrong assertion) or the AC changed.

6. **Skipping the Red verification.** Always run tests after writing them and confirm they fail. This is the most commonly skipped step and the one most likely to produce tautological tests.

7. **Kitchen-sink tests.** One test verifying 15 things. Each test should verify one behavior. Multiple assertions are fine if they all relate to the same behavior.

---

## Verification Commands Quick Reference

```bash
# Run specific tests by name prefix (TDD inner loop)
cargo nextest run -E 'test(~screen_capture_x11)'

# Run all tests in a package
cargo nextest run -p luminos-core

# Run full test suite (phase boundary checkpoint)
cargo nextest run

# Run doctests separately (nextest doesn't support them)
cargo test --doc

# Lint check (run after Green and Refactor phases)
cargo clippy --all-targets -- -D warnings

# Format check
cargo fmt -- --check

# Run integration tests (requires platform APIs)
cargo nextest run --features integration_tests

# Run with verbose output for debugging failures
cargo nextest run -E 'test(~failing_test)' --no-capture
```

---

## Quick Reference Checklist

For each subtask in SUBTASKS.md:

- [ ] Read the task's "Traces to" field — know which ACs you're covering
- [ ] **RED:** Write test(s) from the acceptance criteria
- [ ] **RED:** Run tests — confirm they FAIL
- [ ] **GREEN:** Write minimal implementation to pass
- [ ] **GREEN:** Run tests — confirm they PASS
- [ ] **GREEN:** Run clippy — fix any warnings
- [ ] **REFACTOR:** Improve code quality (naming, structure, duplication)
- [ ] **REFACTOR:** Run tests — confirm they still PASS
- [ ] Check off items in SUBTASKS.md
- [ ] Write Completion Notes
- [ ] Update epic Shared Context if needed
