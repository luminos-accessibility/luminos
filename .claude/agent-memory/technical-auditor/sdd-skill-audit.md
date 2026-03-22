# SDD Skill Audit Details (2026-03-21)

## Files Audited
- `.claude/skills/spec-driven-development/SKILL.md`
- `.claude/skills/spec-driven-development/references/templates.md`
- `.claude/skills/spec-driven-development/references/quality-checklists.md`

## Canonical Source
- `specs/README.md` (649 lines)

## Key Findings

### DESIGN.md Lifecycle Oversimplification
SKILL.md line 333 shows `APPROVED <--> REVISION NEEDED` implying direct back-and-forth.
Canonical shows APPROVED -> REVISION NEEDED -> IN PROGRESS -> APPROVED (three-step cycle).
The intermediate IN PROGRESS state is lost.

### Missing TypeScript TDD Exception
Canonical (specs/README.md line 491) allows TypeScript/React UI code to use behavioral/accessibility
tests instead of strict test-first. Snapshot or component-level integration tests may substitute
for layout code. Skill only mentions setup tasks as TDD exceptions.

### Missing Cross-Epic Blocker Rule
Canonical HLP rule #8 (specs/README.md line 204): When blocker affects cross-epic graph, update
Hard/Soft Dependencies AND create a story in the blocking epic. Log in Blockers & Issues Log.
Not mentioned anywhere in skill.

### AC-Count Splitting Rule Omitted from SKILL.md
Canonical governance rule #9: split if >20 subtasks OR >5 ACs.
SKILL.md governance only mentions subtask count.
Quality-checklists.md does cover this (line 171) but adds "2-5 ACs" target range not in canonical.

### "2-5 ACs" Target Range Added Without Canonical Basis
Quality-checklists.md line 171 states target of "2-5 ACs" per story.
Canonical only says "> 5 ACs" triggers split. No minimum or target range specified.

### Approval Process Simplified
Quality-checklists.md says "before DRAFT -> APPROVED" for team review.
Canonical says "before DRAFT/IN PROGRESS -> APPROVED" -- covers both transitions.
This matters for DESIGN.md which goes through IN PROGRESS before APPROVED.

### Governance Rule #4 (Independence) Missing
Canonical: "Stories should be independently implementable. Where dependencies exist,
they must be explicit in the Depends On field."
Not in SKILL.md governance quick reference. Partially covered by Dependency Rules section.

### Missing Lifecycle Transition Details
Canonical has 7 bullet points explaining lifecycle transitions. Skill only covers 3.
Missing: epic IN PROGRESS trigger, BLOCKED->IN PROGRESS, story DONE conditions,
CANCELLED availability from any post-APPROVED state.

### Information Scoping "unless explicitly directed" Omitted
Canonical rule #2: agents do NOT read other stories' files "unless explicitly directed."
Skill omits this escape hatch.

## Templates Verification
All four templates (HLP, STORY, DESIGN, SUBTASKS) match canonical source exactly.
Task Notation Reference in templates.md also matches canonical Task Notation section.
