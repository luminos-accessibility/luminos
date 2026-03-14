# Luminos Spec-Driven Development Guide

This directory contains all implementation specs for the Luminos project. Spec-driven development (SDD) is the primary engineering methodology: **every feature begins with a written specification before any code is written**. The specification is the source of truth for both human developers and AI coding agents.

---

## Why Spec-Driven Development

Luminos uses AI-agent driven development. Without structured specifications, AI agents produce inconsistent, architecturally misaligned code ("vibe coding"). SDD solves this by:

- **Forcing clarity** on requirements, constraints, and acceptance criteria before implementation
- **Encoding architectural decisions** so AI-generated code is native to the project, not bolted on
- **Creating a shared contract** between product, engineering, and AI agents
- **Enabling parallel development** through independent, self-contained stories
- **Providing traceable progress** through living documents that evolve with the implementation

SDD complements Test-Driven Development (TDD) -- specs define _what_ the system does at the story level, TDD enforces _how_ each subtask is verified at the code level. Together they produce a layered guarantee: specifications catch design errors, tests catch implementation errors.

---

## Directory Structure

Each story gets its own folder under `docs/`. Stories are numbered sequentially with a short kebab-case descriptor:

```
docs/
  README.md                          # This file
  001-screen-capture-foundation/
    STORY.md                         # Requirements specification
    DESIGN.md                        # Technical design document
    SUBTASKS.md                      # TDD task breakdown + progress tracking
  002-gpu-magnification-pipeline/
    STORY.md
    DESIGN.md
    SUBTASKS.md
  ...
```

### The Three Artifacts

| File | Purpose | Owner | When Written |
|------|---------|-------|--------------|
| **STORY.md** | _What_ to build and _why_. Requirements, user scenarios, acceptance criteria. | Product / Architect | Before design begins |
| **DESIGN.md** | _How_ to build it. Architecture, data flow, component design, testing strategy. | Engineer / Architect | After story approval |
| **SUBTASKS.md** | _Execution plan_. TDD-driven task breakdown, progress tracking, completion log. | Implementing Agent / Engineer | After design approval |

---

## STORY.md -- Requirements Specification

The story file defines the problem, scope, and acceptance criteria. It must be implementation-agnostic -- describe _behavior_, not _code_.

### Template

```markdown
# Story [NNN]: [Title]

**Priority:** P0 | P1 | P2 | P3
**Phase:** [Product strategy phase reference]
**Status:** DRAFT | APPROVED | IN PROGRESS | DONE | CANCELLED
**Depends On:** [Story IDs, or "None"]

---

## Problem Statement

[1-3 paragraphs. What user problem does this solve? Why does it matter?]

## User Scenarios

### US-1: [Scenario Name]
As a [persona], I want to [capability] so that [value].

**Priority:** P0
**Acceptance Criteria:**

- **AC-1.1:** Given [precondition], when [action], then [expected outcome].
- **AC-1.2:** Given [precondition], when [action], then [expected outcome].

### US-2: [Scenario Name]
...

## Functional Requirements

- **FR-1:** [Requirement statement]
- **FR-2:** [Requirement statement]

## Non-Functional Requirements

- **NFR-1:** [Performance / security / compatibility constraint]
- **NFR-2:** [Constraint]

## Out of Scope

- [Explicitly excluded items to prevent scope creep]

## Open Questions

- [ ] [Question requiring clarification before design]
```

### Rules

1. Every acceptance criterion uses **Given-When-Then** format. These become the basis for automated tests.
2. Each user scenario must be **independently testable** -- no hidden dependencies on other scenarios.
3. Priority levels map to the product strategy phases: P0 = MVP, P1 = high-value, P2 = medium, P3 = future.
4. Open questions must be resolved (checked off) before the story moves to APPROVED status.
5. The `Depends On` field creates an explicit dependency graph between stories. See Dependency Rules below.

---

## DESIGN.md -- Technical Design Document

The design file translates story requirements into architecture. It constrains AI agents to produce code that fits the existing system.

### Template

```markdown
# Design: Story [NNN] -- [Title]

**Status:** DRAFT | IN PROGRESS | APPROVED | REVISION NEEDED
**Story:** [Link to STORY.md]
**Author:** [Name / Agent]

---

## Overview

[1-2 paragraphs. High-level approach and rationale.]

## Architecture

### Component Diagram

[ASCII diagram or description of how new components fit into the existing architecture]

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `ScreenCapture` | New impl | macOS backend via xcap |
| ... | ... | ... |

### Data Flow

[Step-by-step description of data flow through the system for the primary scenario]

## API Design

[Public function signatures, trait definitions, IPC commands. Include types.]

## Error Handling

[Error types, recovery strategies, user-facing error messages.
Must follow CLAUDE.md error handling conventions: prefer `?` propagation,
`From` trait conversions for error type mismatches, no `unwrap()`/`expect()` in production code.]

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| macOS | ... | ... |
| Windows | ... | ... |
| Linux X11 | ... | ... |

## Testing Strategy

### Unit Tests
- [What to test at the unit level, which modules]

### Integration Tests
- [Cross-module or cross-platform verification]

### Acceptance Tests
- [Mapping from AC-X.X identifiers in STORY.md to test approach]

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Integration | [Description] |
| AC-1.2 | Unit | [Description] |

## Performance Targets

[Specific measurable targets derived from NFRs in the story]

## Security Considerations

[If applicable: permissions, sandboxing, data handling]

## Alternatives Considered

[Briefly note rejected approaches and why]
```

### Rules

1. Every acceptance criterion from STORY.md must appear in the Testing Strategy with a concrete verification method.
2. All new public APIs must include type signatures -- AI agents use these as implementation contracts.
3. The design must reference existing traits and modules from `CLAUDE.md` architecture section, not invent parallel abstractions.
4. Platform-specific approaches must be called out explicitly per the macOS-first development order.

---

## SUBTASKS.md -- TDD Task Breakdown & Progress Tracking

This is the **execution file**. It breaks the design into ordered, atomic tasks that each follow the TDD red-green-refactor cycle. It also serves as the **progress memory** for the story -- agents and developers update it as work proceeds, creating a living record of what was done, what passed, and what remains.

### Template

```markdown
# Subtasks: Story [NNN] -- [Title]

**Status:** NOT STARTED | IN PROGRESS | BLOCKED | DONE
**Started:** [Date]
**Completed:** [Date or "—"]
**Story:** [Link to STORY.md]
**Design:** [Link to DESIGN.md]

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 3 | 0 | 0 | 3 |
| 2. Core | 5 | 0 | 0 | 5 |
| 3. Integration | 4 | 0 | 0 | 4 |
| 4. Polish | 2 | 0 | 0 | 2 |
| **Total** | **14** | **0** | **0** | **14** |

---

## Phase 1: Setup

### T001 [P] — [Task title]
**Traces to:** FR-1, AC-1.1
**Status:** TODO | IN PROGRESS | DONE | BLOCKED
**Files:** `src/capture/mod.rs`, `src/capture/macos.rs`

**TDD Cycle:**
1. **Red** — Write test(s):
   - [ ] `screen_capture_macos_init_success` — Verify capture session initializes on macOS
   - [ ] `screen_capture_macos_init_missing_permission` — Verify graceful error without permission
2. **Green** — Implement minimum code to pass:
   - [ ] Implement `MacOSScreenCapture::new()` returning `Result<Self, CaptureError>`
3. **Refactor** — Clean up while tests stay green:
   - [ ] Extract permission check into `check_screen_capture_permission()` helper

**Completion Notes:**
> [Agent/developer fills this in after completing the task. What was implemented,
> any deviations from the plan, issues encountered, decisions made.]

---

### T002 — [Task title]
...

---

## Phase 2: Core Implementation

### T003 [P] — [Task title]
...

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [ ] All Phase 1 + Phase 2 tests pass
- [ ] [Specific integration check]

---

## Phase 3: Integration

### T004 — [Task title]
...

---

## Phase 4: Polish & Acceptance

### T005 — Acceptance test verification
**Traces to:** All ACs
**Status:** TODO

**Verification Checklist:**
- [ ] AC-1.1: [Describe manual or automated verification]
- [ ] AC-1.2: [Describe manual or automated verification]
- [ ] All clippy warnings resolved
- [ ] No `unwrap()` in production code paths

**Completion Notes:**
>

---

## Blockers & Issues Log

| ID | Date | Description | Resolution | Status |
|----|------|-------------|------------|--------|
| B001 | — | — | — | — |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| — | — | — |
```

### Task Notation

- **T001, T002, ...** — Sequential task IDs within the story. Referenced in commit messages and discussions.
- **[P]** — Task can run in **parallel** with other [P] tasks (no file conflicts or shared state).
- **Traces to** — Links the task back to specific FRs or ACs from STORY.md, ensuring full traceability.
- **Status values:** `TODO` → `IN PROGRESS` → `DONE` or `BLOCKED` (with entry in Blockers log).

### TDD Cycle Rules

Every implementation task follows the strict **red-green-refactor** cycle:

1. **Red** — Write failing tests first. Tests are derived from acceptance criteria and the design's testing strategy. Each test is listed as a checkbox. Tests must fail before any implementation.
2. **Green** — Write the minimum code to make all tests pass. No more, no less. Each implementation step is a checkbox.
3. **Refactor** — Improve code quality (naming, structure, duplication) while all tests stay green. Refactoring items are checkboxes.

This is **non-negotiable** for Rust backend code. Code without a preceding test is a process violation. The only exceptions are:
- Pure scaffolding tasks (directory creation, dependency additions) which are marked as setup tasks without a TDD cycle.
- TypeScript/React UI code where the "Red" phase may use behavioral or accessibility tests rather than visual tests. Snapshot testing or component-level integration tests may substitute for strict test-first on layout code.

### Rust Testing Conventions

All Rust tests must follow the conventions defined in `CLAUDE.md`:

- **Test naming:** Use hierarchical prefixes for granular selection via `cargo nextest run` (e.g., `screen_capture_macos_init_success`, not `test_init`).
- **Test generators:** Prefix all test object/fixture generators with `generate_test_` (e.g., `generate_test_capture_config()`). Make them public and parametrizable for reuse across modules.
- **Test gating:** Gate test-only code with `#[cfg(test)]` or `#[cfg(feature = "test_utils")]`.
- **Mock/fixture placement:** Place test mock/fixture generation in the same module where the type is defined.
- **`unwrap()` in tests:** Acceptable in unit tests for conciseness. Forbidden in production code.

### Progress Tracking Rules

1. **Update the Progress Summary table** whenever a task changes status. This table is the first thing an agent or developer reads to understand story state.
2. **Fill in Completion Notes** after each task. This is the story's memory -- record what was actually done, not just what was planned. Include:
   - Actual files created or modified
   - Test names and pass/fail results
   - Deviations from the planned approach and why
   - Blockers encountered and how they were resolved
3. **Checkpoints between phases** require all preceding tests to pass before the next phase begins. Every phase boundary should include a checkpoint (shown as a placeholder in the template after Phase 2). If a checkpoint fails, address failing tests in the current phase before proceeding.
4. **Blockers & Issues Log** captures anything that stalled progress, with resolution details for future reference.
5. **Deviations from Design** documents any implementation decisions that diverged from DESIGN.md, preserving the rationale.

### Why SUBTASKS.md is the Memory File

When an AI agent picks up work on a story, it reads SUBTASKS.md to understand:
- **Where we are** — Progress Summary table shows phase completion at a glance
- **What was already done** — Completion Notes record actual implementation details and decisions
- **What's blocked** — Blockers log explains stalled work without re-investigation
- **What changed** — Deviations table shows where reality diverged from the design
- **What's next** — The first unchecked task in the current phase is the next unit of work

This eliminates context loss between sessions or agent handoffs. The file is the single source of truth for execution state.

---

## Workflow Summary

```
┌─────────────────────────────────────────────────────────────┐
│                    SPEC-DRIVEN DEVELOPMENT                  │
│                                                             │
│  1. SPECIFY (STORY.md)                                      │
│     ├─ Define problem, user scenarios, acceptance criteria   │
│     ├─ Resolve all open questions                           │
│     └─ Status → APPROVED                                    │
│                                                             │
│  2. DESIGN (DESIGN.md)                                      │
│     ├─ Translate requirements into architecture             │
│     ├─ Map every AC to a test strategy                      │
│     ├─ Define public APIs with type signatures              │
│     └─ Status → APPROVED                                    │
│                                                             │
│  3. IMPLEMENT (SUBTASKS.md + TDD)                           │
│     ├─ Break design into atomic tasks                       │
│     ├─ For each task:                                       │
│     │   ├─ RED: Write failing tests from ACs                │
│     │   ├─ GREEN: Implement minimum passing code            │
│     │   ├─ REFACTOR: Clean up, tests stay green             │
│     │   └─ UPDATE: Mark done, write completion notes        │
│     ├─ Phase checkpoints: all tests pass before next phase  │
│     └─ Final acceptance: verify all ACs from STORY.md       │
│                                                             │
│  4. REVIEW & CLOSE                                          │
│     ├─ All acceptance criteria verified                     │
│     ├─ SUBTASKS.md fully completed (progress = 100%)        │
│     ├─ STORY.md status → DONE                               │
│     └─ Commit with story reference: "Story NNN: ..."        │
└─────────────────────────────────────────────────────────────┘
```

## Story Lifecycle States

```
STORY.md:    DRAFT ──→ APPROVED ──→ IN PROGRESS ──→ DONE
                                        |               |
                                        └──→ CANCELLED ←┘

DESIGN.md:   DRAFT ──→ IN PROGRESS ──→ APPROVED
                            ^               |
                            |               v
                            └── REVISION NEEDED

SUBTASKS.md: NOT STARTED ──→ IN PROGRESS ──→ DONE
                                 |    ^
                                 v    |
                              BLOCKED─┘
```

- A story cannot move to IN PROGRESS until both STORY.md and DESIGN.md are APPROVED.
- DESIGN.md can cycle back to REVISION NEEDED if implementation reveals design flaws (captured in Deviations table). REVISION NEEDED returns to IN PROGRESS for rework, then back to APPROVED.
- BLOCKED returns to IN PROGRESS once the blocker is resolved (documented in Blockers & Issues Log).
- A story is DONE only when all subtasks are complete and all acceptance criteria are verified.
- A story may be CANCELLED at any point after APPROVED if deprioritized or found infeasible. Record the reason in a Completion Notes section at the bottom of STORY.md.

## Governance Rules

1. **Specification first.** No implementation PR is accepted without a corresponding approved STORY.md and DESIGN.md.
2. **Tests first.** No implementation code is merged without tests written before the code (TDD). The SUBTASKS.md checklist structure enforces this ordering.
3. **Living documents.** Specs are updated as the implementation reveals new information. Old versions are preserved in git history.
4. **Independence.** Stories should be independently implementable. Where dependencies exist, they must be explicit in the `Depends On` field.
5. **Traceability.** Every test traces to an acceptance criterion. Every task traces to a functional requirement. Every design decision traces to a user scenario. The chain is: User Scenario → Acceptance Criterion → Test → Implementation.
6. **Completion notes are mandatory.** AI agents and future developers depend on the SUBTASKS.md completion log to understand what actually happened. Skipping notes degrades the memory function of the file.
7. **Architecture compliance.** All specs must align with the architecture and constraints defined in `CLAUDE.md` (which serves as the project's constitution). Violations require documented rationale in the Deviations table.
8. **Story sizing.** Stories should target 5-15 subtasks. If a story exceeds 20 subtasks or 5 acceptance criteria, consider splitting it into multiple stories.

## Dependency Rules

When a story has a `Depends On` field referencing other stories:

1. **Design work may begin** on a dependent story once the dependency's DESIGN.md is APPROVED. This allows parallel design progress.
2. **Implementation cannot begin** until all dependencies have SUBTASKS.md status = DONE.
3. **Circular dependencies are forbidden.** If detected, refactor stories to break the cycle.
4. If an agent encounters a story whose dependencies are not yet DONE, it should skip to the next independent story or work on design for the dependent story.

## Approval Process

- **Solo development:** The story author may self-approve STORY.md and DESIGN.md after verifying all open questions are resolved and the design aligns with `CLAUDE.md`.
- **Team development:** Approval requires review by at least one other team member or architect before status moves from DRAFT/IN PROGRESS to APPROVED.
