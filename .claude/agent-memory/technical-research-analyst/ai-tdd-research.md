# AI-Assisted TDD Research Notes (2026-03-21)

## Key Sources (verified, authoritative)
- Simon Willison agentic patterns: simonwillison.net/guides/agentic-engineering-patterns/red-green-tdd/
- TDAD paper (Alonso 2026): arxiv.org/abs/2603.17973 - 70% regression reduction with test impact analysis
- LLM4TDD (Piya & Sullivan 2024): Best practices for TDD with LLMs
- Superpowers framework: 99k+ GitHub stars (Jan 2026), enforces 7-phase workflow with TDD
- alexop.dev: Skills + subagents for Claude Code TDD, solves context pollution
- Joe Gaebel: Principled Agentic Software Development, tdd-test-writer/tdd-implementer pattern
- Anthropic blog (Mar 2026): claude.com/blog/how-anthropic-teams-use-claude-code - confirms TDD usage internally

## Critical Findings
1. AI agents default to implementation-first; explicit enforcement required
2. Context pollution: same agent writing tests+impl = tests coupled to implementation
3. TDAD paradox: for small models, TDD *prompting* increases regressions; contextual info (which tests to check) works better than procedural instructions (how to do TDD)
4. For frontier models (Claude Opus/Sonnet), procedural TDD instructions remain effective
5. Excessive mocking is the #1 test quality issue with AI-generated tests
6. Red phase verification (confirming tests fail) is most commonly skipped step

## Anti-Patterns Catalog
1. Tests and implementation simultaneously (defeats TDD purpose)
2. Excessive mocking (tests verify mocks, not code)
3. Tests that always pass (tautological assertions)
4. Testing implementation details, not behavior
5. Context pollution (implementation knowledge bleeds into tests)
6. AI "cheating" (modifying tests to pass instead of fixing code)
7. Kitchen Sink tests (one test verifying 15 things)
8. Green Bar Addiction (hardcoded return values)

## Rust TDD Tooling
- `cargo nextest` for parallel test execution with per-test process isolation
- `mockall` crate with `#[cfg_attr(test, mockall::automock)]` for trait mocking
- `cargo-tarpaulin` for coverage
- `cargo-mutants` for mutation testing (test quality verification)
- `cargo insta` for snapshot testing (API contracts)
- `cargo clippy --all-targets -- -D warnings` as quality gate at every Green/Refactor step

## Luminos SDD Alignment
- SUBTASKS.md TDD template already well-structured for AI TDD
- Key addition needed: explicit Red-phase failure verification checkbox
- Given-When-Then maps directly to Arrange-Act-Assert in Rust tests
- Six platform traits are natural mock boundaries
- Phase checkpoints in SUBTASKS.md prevent error accumulation
