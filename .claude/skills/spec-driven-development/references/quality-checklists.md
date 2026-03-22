# SDD Quality Checklists & Governance

Detailed quality gates, governance rules, and common pitfalls for each SDD artifact.
Reference this file when you need to validate an artifact before advancing it to the next stage.

---

## 1. HIGH_LEVEL_PLAN.md Quality Checklist

### Completeness
- [ ] Every roadmap deliverable (from doc-09 epic section) is covered by at least one story
- [ ] Every story maps to at least one roadmap deliverable
- [ ] Success criteria copied verbatim from roadmap epic definition
- [ ] Hard/Soft Dependencies match the roadmap's dependency table (doc-09 Section 3.2)
- [ ] Primary Docs field lists all docs from the roadmap epic definition

### Structure
- [ ] Story count is 3-8 (split epic if > 8; merge if < 3)
- [ ] Dependencies between stories are acyclic (no circular refs)
- [ ] No story appears to need more than 20 subtasks
- [ ] Progress Summary table initialized with all stories as NOT STARTED
- [ ] Each story description has: Scope, Key Deliverables, Estimated Effort, Notes

### Shared Context
- [ ] Architecture Decisions section initialized with known decisions from tech strategy docs
- [ ] Key Type Definitions section prepared (may be empty initially)
- [ ] Integration Points section identifies connections to existing code or other epics

### Common Pitfalls
- Decomposing horizontally (all traits first, then all impls) instead of vertically (trait + impl + test per story)
- Creating too many tiny stories that have heavy cross-dependencies
- Forgetting to copy the Primary Docs field -- agents need this to know which tech strategy docs to read
- Not initializing Shared Context with pre-existing architectural decisions from the tech strategy

---

## 2. STORY.md Quality Checklist

### Acceptance Criteria Quality
- [ ] Every AC uses Given-When-Then format (no exceptions)
- [ ] Every AC is independently testable (no hidden dependencies on other ACs)
- [ ] ACs cover both happy path AND error/edge cases
- [ ] ACs are specific: include concrete values, types, error names
- [ ] ACs are observable: outcomes can be verified programmatically or by inspection

### Requirements Coverage
- [ ] Every FR traces to at least one AC
- [ ] Every roadmap deliverable assigned to this story has at least one AC
- [ ] NFRs have specific, measurable targets (not "must be fast" -- instead "P99 < 20ms")
- [ ] Out of Scope explicitly lists excluded items (prevents scope creep)

### Readiness
- [ ] All Open Questions are resolved (checked off) before APPROVED status
- [ ] Priority levels (P0/P1/P2/P3) assigned to each user scenario
- [ ] Depends On field correctly references prerequisite stories
- [ ] Problem Statement explains the "why" (not just the "what")

### Common Pitfalls
- Writing ACs that describe implementation ("Then the function calls X") instead of behavior ("Then a CaptureFrame is returned")
- Coupling ACs: "Given AC-1.1 passed..." -- each AC must be independently verifiable
- Vague outcomes: "Then it works correctly" -- be specific about what "correctly" means
- Missing error cases: every "Given valid X" AC should have a companion "Given invalid X" AC
- Forgetting NFRs: performance targets from doc-06 and epic success criteria are binding

---

## 3. DESIGN.md Quality Checklist

### Architecture Alignment
- [ ] Component diagram shows how new code fits into the existing 5-crate workspace
- [ ] New components use existing traits/modules from CLAUDE.md, not parallel abstractions
- [ ] Data flow description covers the primary scenario step-by-step
- [ ] Affected Traits/Modules table lists every trait and module touched

### API Completeness
- [ ] All new public APIs have full type signatures (Rust signatures or TypeScript types)
- [ ] IPC commands show both Rust command signature and TypeScript call pattern
- [ ] Error types are defined and follow CLAUDE.md conventions (`?` propagation, `From` conversions, no `unwrap()`)

### Testing Strategy
- [ ] Every AC from STORY.md appears in the Testing Strategy table
- [ ] Each AC has a test type assigned (unit, integration, property-based, accessibility, manual)
- [ ] Each AC has a concrete verification method described
- [ ] The Testing Strategy table is detailed enough to derive SUBTASKS.md tasks from it

### Risk and Alternatives
- [ ] Risk Refs field references relevant risks from doc-10 (or "None identified")
- [ ] Alternatives Considered section documents at least one rejected approach with rationale
- [ ] Platform Considerations table documents approach for the current target platform

### Common Pitfalls
- Writing prose descriptions of APIs instead of actual type signatures
- Forgetting to map every AC to a test -- if an AC isn't in the Testing Strategy, it won't be tested
- Not referencing the risk register -- risks affect design decisions
- Designing in isolation without reading Shared Context from HIGH_LEVEL_PLAN.md
- Overdesigning: the design should constrain the implementation, not write it. Leave room for the implementing agent.

---

## 4. SUBTASKS.md Quality Checklist

### Traceability
- [ ] Every AC from STORY.md is traced to by at least one task
- [ ] Every FR from STORY.md is traced to by at least one task
- [ ] "Traces to" field on each task references specific FR/AC identifiers
- [ ] If a task doesn't trace to any requirement, it's either setup or needs justification

### Task Quality
- [ ] Total task count is 5-15 (split story if > 20)
- [ ] Every implementation task has a Red-Green-Refactor cycle
- [ ] Setup tasks are clearly marked (no TDD cycle required)
- [ ] Test names follow hierarchical naming conventions from CLAUDE.md
- [ ] Files listed in each task are specific paths (not "various files")

### Structure
- [ ] Tasks organized into 3-4 phases (Setup, Core, Integration, Polish)
- [ ] Phase checkpoints exist between phases
- [ ] Parallel tasks marked with [P]
- [ ] Progress Summary table counts match actual task count per phase

### TDD Discipline
- [ ] Red phase lists specific test names with assertion descriptions
- [ ] Green phase lists minimum implementation steps (not the full solution)
- [ ] Refactor phase lists specific cleanup actions
- [ ] No implementation step appears without a preceding test (except setup tasks)

### Common Pitfalls
- Making tasks too large (> 60 minutes estimated). Split if the Green phase has more than 3-4 steps.
- Writing tests that test implementation details instead of behavior
- Forgetting phase checkpoints -- all tests must pass before advancing
- Not pre-planning test names with hierarchical prefixes (makes `cargo nextest` filtering impossible)
- Writing the Refactor phase as "clean up code" -- be specific about what to refactor

---

## 5. Cross-Artifact Governance Rules

### The Traceability Chain
Every piece of the system traces back to a user need:

```
Product Strategy -> Epic (doc-09) -> Story (STORY.md)
  User Scenario -> Acceptance Criterion -> Test (SUBTASKS.md) -> Implementation
  Functional Req -> Acceptance Criterion -> Test -> Implementation
```

If you can't trace a piece of code back through this chain, question whether it's needed.

### Information Scoping
- Agents read ONLY: epic HIGH_LEVEL_PLAN.md + their story's 3 artifacts + referenced tech docs
- Agents do NOT read other stories' STORY.md / DESIGN.md / SUBTASKS.md (unless explicitly directed)
- Cross-story knowledge transfers through HIGH_LEVEL_PLAN.md Shared Context

### Cross-Epic Blocker Handling
If a blocker affects the cross-epic dependency graph (e.g., a completed epic's trait API needs to change):
1. Update the Hard/Soft Dependencies fields in the affected epic's HIGH_LEVEL_PLAN.md
2. Create a remediation story in the blocking epic to resolve the issue
3. Log the blocker in both epics' Blockers & Issues Log with cross-references

### Shared Context Update Rules
When to update HIGH_LEVEL_PLAN.md Shared Context:
- New public type/trait signatures (include full Rust/TS definition)
- Architecture decisions with rationale
- Platform-specific constraints discovered during testing
- Module paths and file locations that later stories need

When NOT to update:
- Internal implementation details private to a single story
- Full code listings
- Debugging logs or session-specific notes

### Epic and Story Sizing
| Level | Target | Split If | Merge If |
|-------|--------|----------|----------|
| Epic | 3-8 stories | > 8 stories | < 3 stories |
| Story | 5-15 subtasks | > 20 subtasks | < 5 subtasks |
| Story ACs | Up to 5 ACs (guideline) | > 5 ACs | n/a |

### Approval Flow
- **Solo dev:** Self-approve after verifying all open questions are resolved, the design aligns with CLAUDE.md, and quality checklists pass
- **Team dev:** At least one reviewer before DRAFT / IN PROGRESS -> APPROVED
- **Epic decomposition:** Architect/lead review before stories begin

### Mandatory Updates After Story Completion
1. SUBTASKS.md: All tasks marked DONE, completion notes filled in
2. STORY.md: Status -> DONE
3. HIGH_LEVEL_PLAN.md: Progress Summary updated, Shared Context updated if applicable
4. Commit message format: `ENN/NNN: [description]`
