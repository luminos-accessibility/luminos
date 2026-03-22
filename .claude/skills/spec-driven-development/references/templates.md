# SDD Artifact Templates

Complete templates for all four SDD artifacts. Copy these when creating new artifacts,
then fill in the bracketed placeholders.

---

## Section 1: HIGH_LEVEL_PLAN.md Template

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

---

## Section 2: STORY.md Template

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

---

## Section 3: DESIGN.md Template

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

---

## Section 4: SUBTASKS.md Template

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

---

## Task Notation Reference

- **T001, T002, ...** -- Sequential task IDs within the story.
- **[P]** -- Task can run in parallel with other [P] tasks (no file conflicts or shared state).
- **Traces to** -- Links the task back to specific FRs or ACs from STORY.md.
- **Status:** `TODO` -> `IN PROGRESS` -> `DONE` or `BLOCKED` (with entry in Blockers log).
