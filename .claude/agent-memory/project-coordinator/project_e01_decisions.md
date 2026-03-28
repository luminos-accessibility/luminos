---
name: E01 Implementation Decisions
description: Key decisions made at E01 kickoff - repo URL, commit strategy, parallelism, spec approval
type: project
---

E01 implementation started 2026-03-27 with the following decisions:

- **Repository URL:** https://github.com/luminos-accessibility/luminos (NOT luminos-app/luminos as in some design docs - docs need updating)
- **Commit strategy:** One commit per story (not per phase or subtask)
- **Spec approval:** All STORY.md and DESIGN.md treated as approved without formal review cycle
- **Dependency versions:** Verify against crates.io for latest compatible versions, document deviations in SUBTASKS.md
- **Parallelism:** Stories 003 & 004 run in parallel (separate teams) after Story 002 completes
- **Story execution order:** 001 -> 002 -> (003 || 004) -> 005
- **Stale files:** package.json and package-lock.json deleted from repo root (not needed until E4 TypeScript work)

**Why:** User confirmed these decisions during pre-implementation Q&A. Parallelizing 003/004 saves time since they're independent after 002.

**How to apply:** Follow these decisions for all E01 story implementations. Update design docs with correct repo URL. Verify crate versions at implementation time.
