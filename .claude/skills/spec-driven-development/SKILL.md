---
name: spec-driven-development
description: >
  Guide for planning, decomposing, and specifying work in the Luminos project using
  the Spec-Driven Development (SDD) methodology. Use this skill whenever an agent
  needs to decompose an epic into stories, write a HIGH_LEVEL_PLAN.md, create a
  STORY.md (requirements specification), write a DESIGN.md (technical design), or
  create a SUBTASKS.md (TDD task breakdown). Trigger when you see phrases like
  "plan epic", "decompose epic", "break down epic", "start epic E01", "create stories",
  "write story spec", "design story", "create subtasks", "story breakdown",
  "HIGH_LEVEL_PLAN", "begin Phase 0", or any mention of SDD artifacts. Also activate
  when an agent asks "what do I build next?" or needs to understand the planning
  workflow before implementation begins. This skill covers the PLANNING side of SDD
  (what to build and how to design it). For the IMPLEMENTATION side (TDD red-green-refactor),
  use the rust-test-driven-development or typescript-test-driven-development skills instead.
---

# Spec-Driven Development

This skill guides AI agents through the planning and specification workflow for the Luminos project. Every feature begins with a written specification before any code is written. The specification is the source of truth for both human developers and AI coding agents.

SDD exists because AI agents produce inconsistent, architecturally misaligned code without structured specifications. SDD forces clarity on requirements, encodes architectural decisions, and creates a shared contract between product, engineering, and AI agents.

**This skill covers phases 0-2 of the SDD workflow** (decompose, specify, design) plus creating the SUBTASKS.md execution plan. Once SUBTASKS.md exists and is approved, hand off to the `rust-test-driven-development` or `typescript-test-driven-development` skill for the actual implementation.

---

## Decision Tree: What Are You Doing?

Start here. Identify which artifact you need to produce, then follow that section.

```
Are you starting a new epic from the roadmap?
  YES --> Section 1: Create HIGH_LEVEL_PLAN.md
  NO  --> Do you have a story that needs requirements?
            YES --> Section 2: Create STORY.md
            NO  --> Do you have an approved STORY.md that needs a design?
                      YES --> Section 3: Create DESIGN.md
                      NO  --> Do you have an approved DESIGN.md that needs subtasks?
                                YES --> Section 4: Create SUBTASKS.md
                                NO  --> Read the epic's HIGH_LEVEL_PLAN.md to find what's next
```

---

## Section 1: Create HIGH_LEVEL_PLAN.md

The HIGH_LEVEL_PLAN.md is the coordination file for an epic. It breaks the epic into stories, tracks their completion, and provides shared context for cross-story knowledge transfer.

### Before You Start

Read these inputs in order:

1. **The roadmap epic definition** -- `specs/tech-strategy/09-implementation-roadmap.md`, find the section for your epic (e.g., Section 4.1 for E1). Extract: summary, scope, dependencies, deliverables, success criteria, primary docs.
2. **The primary tech strategy docs** listed in the epic definition -- skim them for architectural context relevant to decomposition.
3. **The risk register** -- `specs/tech-strategy/10-risk-register.md`, identify risks tagged to this epic's phase.
4. **CLAUDE.md** -- Refresh your understanding of the architecture, constraints, and coding conventions.

### How to Decompose an Epic into Stories

Think of stories as independently deliverable slices of the epic. Each story should produce a testable, reviewable unit of work.

**Decomposition principles:**

1. **Slice vertically, not horizontally.** A story should deliver a working capability, not a layer. "Implement ScreenCapture trait + X11 backend + integration test" is better than "Define all trait signatures" as a standalone story.
2. **Respect the dependency graph.** Stories that produce types, traits, or APIs that others depend on come first. Stories that can run in parallel should be marked as such.
3. **Target 3-8 stories per epic.** Fewer than 3 suggests the epic is too small. More than 8 suggests it needs splitting.
4. **Each story targets 5-15 subtasks.** If you can already see a story will need 20+ subtasks, split it.
5. **Every story traces to epic deliverables.** If a story doesn't map to at least one deliverable from the roadmap, question whether it belongs.

**Decomposition process:**

1. List the epic's deliverables from the roadmap definition.
2. Group related deliverables into logical units of work -- these become stories.
3. Identify dependencies between groups. Order them so dependent stories come after their prerequisites.
4. For each story, write a 1-2 sentence scope description and list key deliverables.
5. Estimate effort (S/M/L t-shirt sizing). S = 1-5 subtasks, M = 6-10, L = 11-15.
6. Verify the dependency chain is acyclic and the total story count is 3-8.

### Writing the HIGH_LEVEL_PLAN.md

Use the template from `references/templates.md` (Section 1). Key sections to get right:

- **Header fields:** Copy `Hard Dependencies`, `Soft Dependencies`, and `Primary Docs` directly from the roadmap epic definition.
- **Overview:** Adapt (don't just copy) the roadmap summary. Focus on what this epic delivers to users or the team.
- **Success Criteria:** Copy verbatim from the roadmap. These are epic-level acceptance criteria.
- **Progress Summary table:** Initialize all stories as `NOT STARTED`. Mark dependencies.
- **Story Descriptions:** For each story, write scope, key deliverables, estimated effort, and any notes for the implementing agent.
- **Shared Context:** Initialize with any architectural decisions that are already known (from the tech strategy docs). This section grows as stories complete.

### Quality Gate: HIGH_LEVEL_PLAN.md

Before moving to story creation, verify:

- [ ] Every roadmap deliverable is covered by at least one story
- [ ] Every story maps to at least one roadmap deliverable
- [ ] Story count is 3-8
- [ ] Dependencies between stories are acyclic
- [ ] No story appears to need more than 20 subtasks
- [ ] Hard/soft dependencies match the roadmap
- [ ] Primary docs are listed correctly
- [ ] Success criteria are copied from roadmap

---

## Section 2: Create STORY.md

The STORY.md defines what to build and why. It must be implementation-agnostic -- describe behavior, not code.

### Before You Start

Read these inputs:

1. **The epic's HIGH_LEVEL_PLAN.md** -- Read the full file for context, then focus on your story's description in the Story Descriptions section.
2. **The relevant tech strategy docs** (listed in the epic's Primary Docs field) -- but only the sections relevant to your story's scope.
3. **The product strategy** (`specs/PRODUCT_STRATEGY.md`) -- if your story involves user-facing behavior, find the relevant product requirements.

### Writing the STORY.md

Use the template from `references/templates.md` (Section 2). The critical section is **User Scenarios and Acceptance Criteria**.

**Writing acceptance criteria:**

Every AC must use the **Given-When-Then** format. These become the basis for automated tests.

```
Given [a specific precondition or system state],
When [a specific action is taken],
Then [a specific, observable, testable outcome occurs].
```

The quality of your ACs determines the quality of everything downstream. Good ACs are:

- **Specific:** "Given a 1920x1080 X11 display" not "Given a display"
- **Testable:** "Then the CaptureFrame has width=1920 and height=1080" not "Then it works"
- **Independent:** Each AC can be verified without relying on another AC's outcome
- **Boundary-aware:** Include edge cases, error conditions, and limits

**Deriving ACs from roadmap deliverables:**

Each deliverable from the roadmap's epic definition maps to one or more ACs. The deliverable says *what*; the AC says *how we verify it*.

Example:
- Roadmap deliverable: "ScreenCapture impl captures full-screen content on X11"
- AC-1.1: Given a valid X11 display connection, when `capture_frame()` is called, then a `CaptureFrame` is returned with dimensions matching the display resolution.
- AC-1.2: Given no X11 display connection, when `capture_frame()` is called, then a `CaptureError::DisplayNotFound` is returned.

**Other sections:**

- **Functional Requirements (FRs):** Distill from the epic deliverables and tech strategy. Number them (FR-1, FR-2, ...). Every FR should be traceable to at least one AC.
- **Non-Functional Requirements (NFRs):** Pull performance targets from `specs/tech-strategy/06-cross-cutting-concerns.md` and the epic's success criteria. Be specific: "Frame time P99 < 20ms" not "Must be fast."
- **Out of Scope:** Be explicit. Reference what the roadmap excludes from this epic, and also what other stories within the same epic handle.
- **Open Questions:** Any unresolved ambiguities. All must be answered before STORY.md moves to APPROVED.

### Quality Gate: STORY.md

Before moving to design, verify:

- [ ] Every roadmap deliverable assigned to this story has at least one AC
- [ ] Every AC uses Given-When-Then format
- [ ] Every AC is independently testable
- [ ] Every FR traces to at least one AC
- [ ] NFRs have specific, measurable targets
- [ ] Out of Scope explicitly lists excluded items
- [ ] All Open Questions are resolved (checked off)
- [ ] Priority levels (P0/P1/P2/P3) are assigned to each user scenario

---

## Section 3: Create DESIGN.md

The DESIGN.md translates story requirements into architecture. It constrains AI agents to produce code that fits the existing system.

### Before You Start

Read these inputs:

1. **Your STORY.md** -- The source of all requirements. Every AC must be addressed.
2. **The epic's HIGH_LEVEL_PLAN.md** -- Especially the Shared Context section for cross-story decisions and type definitions.
3. **The relevant tech strategy docs** -- Now read them in detail for the sections relevant to your implementation.
4. **The risk register** (`specs/tech-strategy/10-risk-register.md`) -- Identify risks relevant to this story. Reference them by ID.
5. **CLAUDE.md** -- The architecture section and coding rules are binding constraints on your design.

### Writing the DESIGN.md

Use the template from `references/templates.md` (Section 3).

**Architecture section:**

- Draw component relationships as ASCII diagrams or structured descriptions. Show how new components fit into the existing five-crate workspace.
- List every trait or module affected with the change type (New, Modified, Extended).
- Describe the data flow for the primary scenario step-by-step.

**API Design section:**

All new public APIs must include **full type signatures**. AI agents use these as implementation contracts. Don't describe the API in prose; show the actual Rust signatures or TypeScript types.

```rust
// Good: full signature
pub trait ScreenCapture: Send + Sync {
    fn capture_frame(&self, region: CaptureRegion) -> Result<CaptureFrame, CaptureError>;
}

// Bad: prose description
// "A method that captures a frame from a screen region"
```

For IPC commands (Tauri), show both the Rust command signature and the corresponding TypeScript call pattern.

**Testing Strategy section:**

This is critical. Every AC from STORY.md must appear in the testing strategy table with:
- The AC identifier (AC-X.X)
- The test type (unit, integration, property-based, accessibility, manual)
- A concrete verification method

This table becomes the blueprint for SUBTASKS.md. If you can't describe how to test an AC, the AC itself may be poorly specified.

**Error Handling section:**

Follow CLAUDE.md conventions: `?` propagation, `From` trait conversions, no `unwrap()`/`expect()` in production. Design the error types that this story introduces or extends.

**Platform Considerations:**

Even if this story only targets one platform (e.g., Linux X11), document the approach for that platform and note what future platforms will need. This helps agents working on porting stories later.

### Quality Gate: DESIGN.md

Before moving to subtask creation, verify:

- [ ] Every AC from STORY.md appears in the Testing Strategy table
- [ ] Every new public API has a full type signature
- [ ] Component diagram shows how new code fits into the existing workspace
- [ ] Error types are defined and follow CLAUDE.md conventions
- [ ] Risk Refs field references relevant risks (or states "None identified")
- [ ] Alternatives Considered section documents at least one rejected approach
- [ ] Design references existing traits/modules from CLAUDE.md, not parallel abstractions
- [ ] Platform-specific approaches are called out per the Linux-first development order

---

## Section 4: Create SUBTASKS.md

The SUBTASKS.md is the execution plan. It breaks the design into ordered, atomic tasks that each follow the TDD red-green-refactor cycle. It also serves as the **progress memory** for the story -- when an AI agent picks up work, it reads SUBTASKS.md to understand:

- **Where we are** -- Progress Summary table shows phase completion at a glance
- **What was already done** -- Completion Notes record actual implementation details and decisions
- **What's blocked** -- Blockers log explains stalled work without re-investigation
- **What changed** -- Deviations table shows where reality diverged from the design
- **What's next** -- The first unchecked task in the current phase is the next unit of work

This eliminates context loss between sessions or agent handoffs.

### Before You Start

Read these inputs:

1. **Your DESIGN.md** -- The Testing Strategy table is the primary input. Each AC-to-test mapping becomes one or more subtasks.
2. **Your STORY.md** -- For the FR/AC identifiers referenced by each task.
3. **The epic's HIGH_LEVEL_PLAN.md** -- Shared Context for any type definitions or decisions from earlier stories.

### How to Break Design into Subtasks

**Phase structure:**

Organize tasks into 3-4 phases that mirror the natural build order:

1. **Setup** -- Scaffolding, module creation, dependency additions. These tasks may skip TDD.
2. **Core Implementation** -- The primary functionality. Each task follows strict red-green-refactor.
3. **Integration** -- Cross-module wiring, IPC round-trips, end-to-end flows.
4. **Polish & Acceptance** -- Final acceptance test verification, clippy, documentation.

**TDD exceptions:** Two categories of tasks may relax the strict test-first cycle:
- **Setup tasks** (Phase 1): directory creation, dependency additions, module scaffolding.
- **TypeScript/React layout code**: the Red phase may use behavioral, accessibility, or component-level integration tests rather than visual tests. Snapshot testing may substitute for strict test-first on CSS/layout work.

**Creating tasks:**

For each row in the DESIGN.md Testing Strategy table:

1. The AC and test type tell you what test to write (Red phase).
2. The verification method tells you what to assert.
3. The affected traits/modules from the Architecture section tell you what to implement (Green phase).
4. The Refactor phase extracts helpers, improves naming, reduces duplication.

**Task sizing:** Each task should take 15-60 minutes for an experienced agent. If a task feels like it will take longer, split it into smaller tasks.

**Traceability:** Every task's "Traces to" field must reference specific FR or AC identifiers from STORY.md. If a task doesn't trace to any requirement, question whether it's needed.

**Parallelism:** Mark tasks that can run in parallel with `[P]`. Tasks are parallelizable when they touch different files and have no data dependencies.

**Phase checkpoints:** After each phase, add a checkpoint that verifies all preceding tests still pass before the next phase begins.

### Writing the SUBTASKS.md

Use the template from `references/templates.md` (Section 4).

For each task, specify:

- **Task ID and title:** `T001 -- [descriptive title]`
- **Traces to:** `FR-1, AC-1.1` (links back to STORY.md)
- **Files:** List the specific files that will be created or modified
- **TDD Cycle:**
  - **Red:** List each test with its hierarchical name (e.g., `screen_capture_x11_init_success`)
  - **Green:** List the minimum implementation steps
  - **Refactor:** List cleanup and improvement items
- **Completion Notes:** Empty block for the implementing agent to fill in

**Test naming for Rust:** Use hierarchical prefixes that allow granular selection via `cargo nextest run`. Pattern: `{module}_{function}_{scenario}`. Examples:
- `screen_capture_init_valid_display`
- `screen_capture_init_missing_permission`
- `viewport_calc_zoom_2x_centered`

**Test naming for TypeScript:** Use descriptive `describe`/`it` blocks. Pattern: `describe('{Component}') > it('should {behavior} when {condition}')`.

### Quality Gate: SUBTASKS.md

Before beginning implementation, verify:

- [ ] Every AC from STORY.md is traced to by at least one task
- [ ] Every FR from STORY.md is traced to by at least one task
- [ ] Total task count is 5-15 (split story if > 20)
- [ ] Every implementation task has a Red-Green-Refactor cycle (except setup tasks)
- [ ] Test names follow CLAUDE.md naming conventions
- [ ] Phase checkpoints exist between phases
- [ ] Files listed in each task are specific (not "various files")
- [ ] Progress Summary table is initialized with correct counts

---

## Naming Conventions

- **Epic folders:** `ENN-kebab-case-descriptor` (e.g., `E01-project-scaffolding`)
- **Story folders:** `NNN-kebab-case-descriptor` (e.g., `001-workspace-setup`)
- **Cross-epic references:** `ENN/NNN` (e.g., `E01/003`)
- **Prose shorthand:** E1, E2 (no zero-padding) for readability in text
- **Folder names:** Always zero-padded (E01, E02; 001, 002) for sort order

---

## Lifecycle States

```
STORY.md:          DRAFT --> APPROVED --> IN PROGRESS --> DONE
                                              |               |
                                              +--> CANCELLED <-+

DESIGN.md:         DRAFT --> IN PROGRESS --> APPROVED
                                  ^               |
                                  |               v
                                  +-- REVISION NEEDED

SUBTASKS.md:       NOT STARTED --> IN PROGRESS --> DONE
                                        |    ^
                                        v    |
                                     BLOCKED-+

HIGH_LEVEL_PLAN.md: NOT STARTED --> IN PROGRESS --> DONE
                                         |    ^
                                         v    |
                                      BLOCKED-+
```

Key transitions:
- An epic moves to IN PROGRESS when the HIGH_LEVEL_PLAN.md is created and story decomposition is complete.
- A story cannot move to IN PROGRESS until both STORY.md and DESIGN.md are APPROVED.
- DESIGN.md can cycle back to REVISION NEEDED if implementation reveals design flaws. REVISION NEEDED returns to IN PROGRESS for rework, then back to APPROVED.
- BLOCKED returns to IN PROGRESS once the blocker is resolved (documented in Blockers & Issues Log).
- A story is DONE only when all subtasks are complete AND all acceptance criteria are verified.
- A story may be CANCELLED at any point after APPROVED if deprioritized or found infeasible. Record the reason in a Completion Notes section at the bottom of STORY.md.
- An epic is DONE only when ALL its stories are DONE and ALL epic success criteria are verified.

---

## Information Scoping Rules

These rules keep agent context windows manageable:

1. **Agents working on a story** read ONLY: the epic's HIGH_LEVEL_PLAN.md, their story's three artifacts, and referenced tech strategy docs.
2. **Agents do NOT read** other stories' files, unless explicitly directed. The Shared Context section in HIGH_LEVEL_PLAN.md provides cross-story knowledge.
3. **When a story completes**, the agent updates the Shared Context section with any types, decisions, or constraints that later stories need.

---

## Dependency Rules

### Within an Epic
- Design work on a dependent story may begin once the dependency's DESIGN.md is APPROVED.
- Implementation cannot begin until within-epic dependencies have SUBTASKS.md status = DONE.
- Circular dependencies are forbidden.

### Across Epics
- An epic cannot begin implementation until all hard dependencies are DONE.
- Design work may begin once hard dependencies' HIGH_LEVEL_PLAN.md is IN PROGRESS.
- Soft dependencies are beneficial but not blocking.
- If an agent encounters an epic whose hard dependencies are not yet DONE, it should work on design for that epic's stories or switch to an independent epic.
- If a blocker affects the cross-epic dependency graph (e.g., a completed epic's trait API needs to change), update the Hard/Soft Dependencies fields AND create a story in the blocking epic to resolve the issue. Log the blocker in both epics' Blockers & Issues Log.

---

## Governance Quick Reference

1. No implementation without an approved STORY.md and DESIGN.md.
2. Tests are written before code (TDD). SUBTASKS.md enforces this ordering.
3. Specs are living documents -- update them as implementation reveals new information.
4. Stories should be independently implementable. Where dependencies exist, they must be explicit in the `Depends On` field.
5. Every test traces to an acceptance criterion. Every task traces to a requirement.
6. Completion notes in SUBTASKS.md are mandatory (they are agent handoff memory).
7. Shared context updates in HIGH_LEVEL_PLAN.md are mandatory when cross-story knowledge emerges.
8. All specs must align with CLAUDE.md (the project's constitution). Violations require documented rationale in the Deviations table.
9. Stories target 5-15 subtasks and up to 5 ACs; split if > 20 subtasks or > 5 ACs. Epics target 3-8 stories.

---

## After Implementation: Review, Close & Epic Completion

This skill focuses on planning, but agents should understand the full lifecycle. After SUBTASKS.md is complete:

### Story Closure (Phase 4: Review & Close)
1. All acceptance criteria verified (final task in SUBTASKS.md)
2. SUBTASKS.md: all tasks marked DONE, all completion notes filled in
3. STORY.md: status -> DONE
4. Update epic HIGH_LEVEL_PLAN.md: Progress Summary table and Shared Context section
5. Commit with story reference: `ENN/NNN: [description]`

### Epic Completion (Phase 5)
1. All stories in the epic are DONE
2. All epic success criteria (from HIGH_LEVEL_PLAN.md) are verified
3. HIGH_LEVEL_PLAN.md: status -> DONE
4. Fill in Retrospective Notes (what went well, what didn't, what to carry forward)

---

## Reference Files

For full artifact templates, read `references/templates.md`.
For detailed quality checklists and governance rules, read `references/quality-checklists.md`.

These files contain the complete templates from the SDD methodology guide (`specs/README.md`).
Only read them when you need the exact template structure -- the guidance above is sufficient
for understanding the workflow.
