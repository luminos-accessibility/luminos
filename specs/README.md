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

Specs are organized in a **two-level hierarchy**: engineering epics (from the [Implementation Roadmap](./tech-strategy/09-implementation-roadmap.md)) contain implementation stories. Each epic gets a numbered folder with a `HIGH_LEVEL_PLAN.md` that tracks story breakdown and progress. Each story gets a subfolder with the three standard artifacts.

```
specs/
  README.md                                 # This file
  PRODUCT_STRATEGY.md                       # Product strategy & roadmap
  TECH_STACK_EVALUATION.md                  # Technology stack validation
  tech-strategy/                            # Technical strategy (10 documents)
  E01-project-scaffolding/                  # Epic 1 (from roadmap Section 4.1)
    HIGH_LEVEL_PLAN.md                      # Epic-level plan, story breakdown, shared context
    001-workspace-setup/                    # Story 1 within this epic
      STORY.md                              # Requirements specification
      DESIGN.md                             # Technical design document
      SUBTASKS.md                           # TDD task breakdown + progress tracking
    002-platform-traits/                    # Story 2 within this epic
      STORY.md
      DESIGN.md
      SUBTASKS.md
    ...
  E02-x11-capture-gpu/                     # Epic 2 (from roadmap Section 4.2)
    HIGH_LEVEL_PLAN.md
    001-screen-capture-x11/
      ...
    002-gpu-magnification-pipeline/
      ...
  ...
```

### Naming Conventions

- **Epic folders:** `ENN-kebab-case-descriptor` where `NN` is the epic number from the roadmap (E01-E20). The descriptor is a short summary of the epic scope (3-5 words).
- **Story folders:** `NNN-kebab-case-descriptor` where `NNN` is the story number _within the epic_ (001, 002, ...). Numbering restarts at 001 for each epic.
- **Global story references:** Use the format `ENN/NNN` (e.g., `E01/001`) when referencing stories across epics or in commit messages.
- **Prose vs. folder names:** In prose and commit messages, the shorter form (E1, E2) may be used for readability. Folder names always use zero-padded format (E01, E02) to ensure correct lexicographic sort order.

### The Four Artifacts

| File | Level | Purpose | Owner | When Written |
|------|-------|---------|-------|--------------|
| **HIGH_LEVEL_PLAN.md** | Epic | _What stories_ compose this epic, shared context, and progress tracking. | Architect / Lead | When epic work begins |
| **STORY.md** | Story | _What_ to build and _why_. Requirements, user scenarios, acceptance criteria. | Product / Architect | Before design begins |
| **DESIGN.md** | Story | _How_ to build it. Architecture, data flow, component design, testing strategy. | Engineer / Architect | After story approval |
| **SUBTASKS.md** | Story | _Execution plan_. TDD-driven task breakdown, progress tracking, completion log. | Implementing Agent / Engineer | After design approval |

### Information Scoping Rules

To keep AI agent context windows manageable and prevent cross-contamination between stories:

1. **Agents working on a story** read ONLY:
   - The epic's `HIGH_LEVEL_PLAN.md` (for context, shared context, and understanding where the story fits)
   - Their story's `STORY.md`, `DESIGN.md`, and `SUBTASKS.md`
   - Referenced tech strategy documents (listed in the story's DESIGN.md)
2. **Agents do NOT read** other stories' STORY.md/DESIGN.md/SUBTASKS.md files within the same epic, unless explicitly directed. The `HIGH_LEVEL_PLAN.md` shared context section provides any cross-story knowledge that agents need.
3. **When starting an epic**, the lead reads the roadmap epic definition (doc-09 Section N.N) and the `HIGH_LEVEL_PLAN.md` to decompose it into stories. The lead does NOT need to read all tech strategy docs upfront -- only those referenced in the epic's "Primary Docs" field.

---

## HIGH_LEVEL_PLAN.md -- Epic-Level Plan & Shared Context

The high-level plan is the **coordination file** for an engineering epic. It breaks the epic into stories, tracks their completion, and provides a shared context section for cross-story knowledge that agents need. It is the epic-level analog of SUBTASKS.md.

### Template

```markdown
# Epic ENN: [Title from Roadmap]

**Status:** NOT STARTED | IN PROGRESS | BLOCKED | DONE
**Roadmap Ref:** [Link to tech-strategy/09-implementation-roadmap.md Section N.N]
**Phase:** [Phase number and name]
**Started:** [Date or "---"]
**Completed:** [Date or "---"]
**Hard Dependencies:** [Epic IDs that must be DONE, or "None"]
**Soft Dependencies:** [Epic IDs that benefit this work, or "None"]
**Primary Docs:** [Tech strategy documents relevant to this epic, copied from doc-09 epic definition. These are needed during story design (DESIGN.md authoring), not during subtask execution.]

---

## Overview

[1-2 paragraphs. Summarize what this epic delivers, copied/adapted from the roadmap
epic summary. Include the user-perceivable value.]

## Success Criteria

[Copied from the roadmap epic definition. These are epic-level acceptance criteria.]

- [ ] [Criterion 1]
- [ ] [Criterion 2]
- [ ] ...

---

## Story Breakdown

### Progress Summary

| # | Story | Status | Depends On | Notes |
|---|-------|--------|------------|-------|
| 001 | [Story title] | NOT STARTED | --- | [Brief note] |
| 002 | [Story title] | NOT STARTED | 001 | [Brief note] |
| 003 | [Story title] | NOT STARTED | --- | Can run parallel with 001 |
| ... | ... | ... | ... | ... |

**Total Stories:** N | **Done:** 0 | **In Progress:** 0 | **Blocked:** 0

### Story Descriptions

#### 001 -- [Story Title]
**Scope:** [1-2 sentences describing what this story delivers]
**Key Deliverables:** [Bullet list of concrete outputs]
**Estimated Effort:** [T-shirt size: S/M/L or story points]
**Notes:** [Any context for the agent picking this up]

#### 002 -- [Story Title]
...

---

## Shared Context

[This section contains knowledge that is shared across all stories in this epic.
Agents working on any story within this epic read this section for cross-cutting context.
Update this section as stories are completed and new knowledge emerges.

DO record: new public type/trait signatures with full Rust/TS definition, architecture decisions
with rationale, platform-specific constraints discovered during testing, module paths and
file locations that later stories need to reference.

DO NOT record: internal implementation details private to a single story, full code listings,
debugging logs, or anything already documented in a story's SUBTASKS.md completion notes.]

### Architecture Decisions

[Key architectural decisions made during this epic that affect multiple stories.
Include rationale and links to relevant tech strategy docs.]

- **[Decision]:** [Rationale]. See [doc reference].

### Key Type Definitions

[If this epic introduces new types, traits, or APIs that multiple stories depend on,
document them here so later stories don't need to read earlier stories' DESIGN.md files.]

### Integration Points

[Document how this epic's components connect to existing code or other epics' outputs.
Include module paths, trait implementations, and IPC contracts.]

### Discovered Constraints

[Technical constraints or platform-specific issues discovered during implementation
that affect the remaining stories. Update as work progresses.]

### Cross-Story Dependencies

[Runtime or compile-time dependencies between stories that emerged during implementation.
Note: these should also be reflected in the Progress Summary table's "Depends On" column.]

---

## Blockers & Issues Log

| ID | Date | Story | Description | Resolution | Status |
|----|------|-------|-------------|------------|--------|
| B001 | --- | --- | --- | --- | --- |

## Retrospective Notes

[Filled in when the epic is DONE. What went well, what didn't, what to carry forward
to future epics. This informs process improvements.]
```

### HIGH_LEVEL_PLAN.md Rules

1. **Created when an epic begins.** The first step of starting any epic is to create the `HIGH_LEVEL_PLAN.md` by decomposing the roadmap's epic definition into stories.
2. **Story count target: 3-8 stories per epic.** If decomposition yields more than 8 stories, consider whether the epic is too large or whether stories are too granular. If fewer than 3, consider whether the epic should be merged.
3. **Progress Summary is updated whenever a story changes status.** This table is the first thing anyone reads to understand epic state.
4. **Shared Context is a living section.** As stories are completed, agents record discoveries, decisions, and constraints that affect subsequent stories. This is the primary mechanism for cross-story knowledge transfer without requiring agents to read other stories' files.
5. **Story dependencies within an epic** are tracked in the Progress Summary table. Cross-epic dependencies are tracked in the epic's `Hard Dependencies` and `Soft Dependencies` fields.
6. **The Shared Context section replaces the need to read other stories.** When an agent starts story 004, it reads the Shared Context (which was updated by stories 001-003) instead of reading those stories' SUBTASKS.md files.
7. **Shared Context granularity:** Record information that later stories need to _compile against or integrate with_. Concrete examples of what to record: new public type/trait signatures (full definition), architecture decisions with rationale, platform-specific constraints discovered during testing, module paths that later stories must reference. Do NOT record: internal implementation details private to a single story, full code listings, or debugging history.
8. **Cross-epic blockers:** If a blocker affects the cross-epic dependency graph (e.g., a completed epic's trait API needs to change), update the `Hard Dependencies` / `Soft Dependencies` fields in this document AND create a story in the blocking epic to resolve the issue. Log the blocker in the Blockers & Issues Log with a reference to the upstream epic and story.

---

## STORY.md -- Requirements Specification

The story file defines the problem, scope, and acceptance criteria. It must be implementation-agnostic -- describe _behavior_, not _code_.

### Template

```markdown
# Story ENN/NNN: [Title]

**Epic:** [Relative link to HIGH_LEVEL_PLAN.md]
**Status:** DRAFT | APPROVED | IN PROGRESS | DONE | CANCELLED
**Depends On:** [Story IDs within this epic (e.g., "001"), cross-epic refs (e.g., "E01/003"), or "None"]

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
5. The `Depends On` field creates an explicit dependency graph between stories. Use `NNN` for within-epic dependencies, `ENN/NNN` for cross-epic dependencies. See Dependency Rules below.

---

## DESIGN.md -- Technical Design Document

The design file translates story requirements into architecture. It constrains AI agents to produce code that fits the existing system.

### Template

```markdown
# Design: Story ENN/NNN -- [Title]

**Story:** [Relative link to STORY.md]
**Epic:** [Relative link to HIGH_LEVEL_PLAN.md]
**Status:** DRAFT | IN PROGRESS | APPROVED | REVISION NEEDED
**Author:** [Name / Agent]
**Risk Refs:** [Relevant risk IDs from tech-strategy/10-risk-register.md]

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
| Linux X11 | ... | ... |
| Linux Wayland | ... | ... |
| macOS | ... | ... |
| OpenBSD | ... | ... |
| Windows | ... | ... |

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
4. Platform-specific approaches must be called out explicitly per the Linux-first development order.
5. The `Risk Refs` field must reference relevant risks from `specs/tech-strategy/10-risk-register.md`. If no risks apply, state "None identified."

---

## SUBTASKS.md -- TDD Task Breakdown & Progress Tracking

This is the **execution file**. It breaks the design into ordered, atomic tasks that each follow the TDD red-green-refactor cycle. It also serves as the **progress memory** for the story -- agents and developers update it as work proceeds, creating a living record of what was done, what passed, and what remains.

### Template

```markdown
# Subtasks: Story ENN/NNN -- [Title]

**Status:** NOT STARTED | IN PROGRESS | BLOCKED | DONE
**Started:** [Date]
**Completed:** [Date or "---"]
**Story:** [Relative link to STORY.md]
**Design:** [Relative link to DESIGN.md]
**Epic:** [Relative link to HIGH_LEVEL_PLAN.md]

---

## Progress Summary

| Phase | Total | Done | Blocked | Remaining |
|-------|-------|------|---------|-----------|
| 1. Setup | 3 | 0 | 0 | 3 |
| 2. Core Implementation | 5 | 0 | 0 | 5 |
| 3. Integration | 4 | 0 | 0 | 4 |
| 4. Polish & Acceptance | 2 | 0 | 0 | 2 |
| **Total** | **14** | **0** | **0** | **14** |

---

## Phase 1: Setup

### T001 [P] -- [Task title]
**Traces to:** FR-1, AC-1.1
**Status:** TODO | IN PROGRESS | DONE | BLOCKED
**Files:** `src/capture/mod.rs`, `src/capture/macos.rs`

**TDD Cycle:**
1. **Red** -- Write test(s):
   - [ ] `screen_capture_macos_init_success` -- Verify capture session initializes on macOS
   - [ ] `screen_capture_macos_init_missing_permission` -- Verify graceful error without permission
2. **Green** -- Implement minimum code to pass:
   - [ ] Implement `MacOSScreenCapture::new()` returning `Result<Self, CaptureError>`
3. **Refactor** -- Clean up while tests stay green:
   - [ ] Extract permission check into `check_screen_capture_permission()` helper

**Completion Notes:**
> [Agent/developer fills this in after completing the task. What was implemented,
> any deviations from the plan, issues encountered, decisions made.
> Also update the epic's HIGH_LEVEL_PLAN.md Shared Context section if this task
> produced knowledge relevant to other stories.]

---

### T002 -- [Task title]
...

---

## Phase 2: Core Implementation

### T003 [P] -- [Task title]
...

**Checkpoint:** After completing Phase 2, run full test suite and verify:
- [ ] All Phase 1 + Phase 2 tests pass
- [ ] [Specific integration check]

---

## Phase 3: Integration

### T004 -- [Task title]
...

---

## Phase 4: Polish & Acceptance

### T005 -- Acceptance test verification
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
| B001 | --- | --- | --- | --- |

## Deviations from Design

| Task | Deviation | Rationale |
|------|-----------|-----------|
| --- | --- | --- |
```

### Task Notation

- **T001, T002, ...** -- Sequential task IDs within the story. Referenced in commit messages and discussions.
- **[P]** -- Task can run in **parallel** with other [P] tasks (no file conflicts or shared state).
- **Traces to** -- Links the task back to specific FRs or ACs from STORY.md, ensuring full traceability.
- **Status values:** `TODO` -> `IN PROGRESS` -> `DONE` or `BLOCKED` (with entry in Blockers log).

### TDD Cycle Rules

Every implementation task follows the strict **red-green-refactor** cycle:

1. **Red** -- Write failing tests first. Tests are derived from acceptance criteria and the design's testing strategy. Each test is listed as a checkbox. Tests must fail before any implementation.
2. **Green** -- Write the minimum code to make all tests pass. No more, no less. Each implementation step is a checkbox.
3. **Refactor** -- Improve code quality (naming, structure, duplication) while all tests stay green. Refactoring items are checkboxes.

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
3. **Update the epic's HIGH_LEVEL_PLAN.md** after completing a task that produces cross-story knowledge. Add discoveries, constraints, type definitions, or architecture decisions to the Shared Context section.
4. **Checkpoints between phases** require all preceding tests to pass before the next phase begins. Every phase boundary should include a checkpoint (shown as a placeholder in the template after Phase 2). If a checkpoint fails, address failing tests in the current phase before proceeding.
5. **Blockers & Issues Log** captures anything that stalled progress, with resolution details for future reference.
6. **Deviations from Design** documents any implementation decisions that diverged from DESIGN.md, preserving the rationale.

### Why SUBTASKS.md is the Memory File

When an AI agent picks up work on a story, it reads SUBTASKS.md to understand:
- **Where we are** -- Progress Summary table shows phase completion at a glance
- **What was already done** -- Completion Notes record actual implementation details and decisions
- **What's blocked** -- Blockers log explains stalled work without re-investigation
- **What changed** -- Deviations table shows where reality diverged from the design
- **What's next** -- The first unchecked task in the current phase is the next unit of work

This eliminates context loss between sessions or agent handoffs. The file is the single source of truth for execution state.

---

## Workflow Summary

```
+------------------------------------------------------------------+
|                    SPEC-DRIVEN DEVELOPMENT                        |
|                                                                   |
|  0. DECOMPOSE EPIC (HIGH_LEVEL_PLAN.md)                          |
|     +-- Read roadmap epic definition (doc-09 Section N.N)         |
|     +-- Break into 3-8 stories with scope and dependencies       |
|     +-- Initialize Shared Context with architecture notes        |
|     +-- Status -> IN PROGRESS                                    |
|                                                                   |
|  1. SPECIFY (STORY.md)                                            |
|     +-- Define problem, user scenarios, acceptance criteria       |
|     +-- Resolve all open questions                               |
|     +-- Status -> APPROVED                                       |
|                                                                   |
|  2. DESIGN (DESIGN.md)                                            |
|     +-- Translate requirements into architecture                 |
|     +-- Map every AC to a test strategy                          |
|     +-- Define public APIs with type signatures                  |
|     +-- Reference relevant risks from risk register              |
|     +-- Status -> APPROVED                                       |
|                                                                   |
|  3. IMPLEMENT (SUBTASKS.md + TDD)                                 |
|     +-- Break design into atomic tasks                           |
|     +-- For each task:                                           |
|     |   +-- RED: Write failing tests from ACs                    |
|     |   +-- GREEN: Implement minimum passing code                |
|     |   +-- REFACTOR: Clean up, tests stay green                 |
|     |   +-- UPDATE: Mark done, write completion notes            |
|     |   +-- SHARE: Update epic Shared Context if needed          |
|     +-- Phase checkpoints: all tests pass before next phase      |
|     +-- Final acceptance: verify all ACs from STORY.md           |
|                                                                   |
|  4. REVIEW & CLOSE                                                |
|     +-- All acceptance criteria verified                         |
|     +-- SUBTASKS.md fully completed (progress = 100%)            |
|     +-- STORY.md status -> DONE                                  |
|     +-- Update epic HIGH_LEVEL_PLAN.md progress table            |
|     +-- Commit with story reference: "ENN/NNN: ..."  |
|                                                                   |
|  5. EPIC COMPLETION                                               |
|     +-- All stories DONE                                         |
|     +-- All epic success criteria verified                       |
|     +-- HIGH_LEVEL_PLAN.md status -> DONE                        |
|     +-- Fill in Retrospective Notes                              |
+------------------------------------------------------------------+
```

## Story Lifecycle States

```
STORY.md:    DRAFT --> APPROVED --> IN PROGRESS --> DONE
                                        |               |
                                        +--> CANCELLED <-+

DESIGN.md:   DRAFT --> IN PROGRESS --> APPROVED
                            ^               |
                            |               v
                            +-- REVISION NEEDED

SUBTASKS.md: NOT STARTED --> IN PROGRESS --> DONE
                                 |    ^
                                 v    |
                              BLOCKED-+

HIGH_LEVEL_PLAN.md: NOT STARTED --> IN PROGRESS --> DONE
                                         |    ^
                                         v    |
                                      BLOCKED-+
```

- An epic moves to IN PROGRESS when the HIGH_LEVEL_PLAN.md is created and story decomposition is complete.
- A story cannot move to IN PROGRESS until both STORY.md and DESIGN.md are APPROVED.
- DESIGN.md can cycle back to REVISION NEEDED if implementation reveals design flaws (captured in Deviations table). REVISION NEEDED returns to IN PROGRESS for rework, then back to APPROVED.
- BLOCKED returns to IN PROGRESS once the blocker is resolved (documented in Blockers & Issues Log).
- A story is DONE only when all subtasks are complete and all acceptance criteria are verified.
- A story may be CANCELLED at any point after APPROVED if deprioritized or found infeasible. Record the reason in a Completion Notes section at the bottom of STORY.md.
- An epic is DONE only when all its stories are DONE and all epic-level success criteria are verified.

## Governance Rules

1. **Specification first.** No implementation PR is accepted without a corresponding approved STORY.md and DESIGN.md.
2. **Tests first.** No implementation code is merged without tests written before the code (TDD). The SUBTASKS.md checklist structure enforces this ordering.
3. **Living documents.** Specs are updated as the implementation reveals new information. Old versions are preserved in git history.
4. **Independence.** Stories should be independently implementable. Where dependencies exist, they must be explicit in the `Depends On` field.
5. **Traceability.** Every test traces to an acceptance criterion. Every task traces to a functional requirement. Every design decision traces to a user scenario. The chain is: User Scenario -> Acceptance Criterion -> Test -> Implementation.
6. **Completion notes are mandatory.** AI agents and future developers depend on the SUBTASKS.md completion log to understand what actually happened. Skipping notes degrades the memory function of the file.
7. **Shared context updates are mandatory.** When a story produces knowledge relevant to other stories in the same epic (new types, discovered constraints, architecture decisions), the agent must update the HIGH_LEVEL_PLAN.md Shared Context section.
8. **Architecture compliance.** All specs must align with the architecture and constraints defined in `CLAUDE.md` (which serves as the project's constitution). Violations require documented rationale in the Deviations table.
9. **Story sizing.** Stories should target 5-15 subtasks. If a story exceeds 20 subtasks or 5 acceptance criteria, consider splitting it into multiple stories.
10. **Epic sizing.** Epics should contain 3-8 stories. If decomposition yields more than 8, split the epic (update the roadmap accordingly). If fewer than 3, consider merging.
11. **Information scoping.** Agents working on a story read only the epic's HIGH_LEVEL_PLAN.md and their story's three artifacts. They do not read other stories' files. The Shared Context section in HIGH_LEVEL_PLAN.md is the mechanism for cross-story knowledge transfer.

## Dependency Rules

### Within an Epic

Stories within an epic track dependencies in the HIGH_LEVEL_PLAN.md Progress Summary table and in each story's `Depends On` field (using the story number, e.g., "001").

1. **Design work may begin** on a dependent story once the dependency's DESIGN.md is APPROVED. This allows parallel design progress.
2. **Implementation cannot begin** until all within-epic dependencies have SUBTASKS.md status = DONE.
3. **Circular dependencies are forbidden.** If detected, refactor stories to break the cycle.

### Across Epics

Cross-epic dependencies are managed at the epic level via the `Hard Dependencies` and `Soft Dependencies` fields in HIGH_LEVEL_PLAN.md, mirroring the dependency graph in doc-09 Section 3.

1. **An epic cannot begin implementation** until all its hard dependencies are DONE.
2. **Design work on an epic may begin** once its hard dependencies' HIGH_LEVEL_PLAN.md is IN PROGRESS (story breakdown is known).
3. **Soft dependencies** indicate beneficial sequencing but are not blocking. An epic can start without its soft dependencies being complete.
4. **Cross-epic story references** use the `ENN/NNN` format (e.g., "E01/003" = Epic 1, Story 3).
5. If an agent encounters an epic whose hard dependencies are not yet DONE, it should work on design for that epic's stories or switch to an independent epic.

## Approval Process

- **Solo development:** The story author may self-approve STORY.md and DESIGN.md after verifying all open questions are resolved and the design aligns with `CLAUDE.md`.
- **Team development:** Approval requires review by at least one other team member or architect before status moves from DRAFT/IN PROGRESS to APPROVED.
- **Epic decomposition:** The HIGH_LEVEL_PLAN.md story breakdown should be reviewed by an architect or tech lead before individual stories begin, to validate scope and sequencing.
