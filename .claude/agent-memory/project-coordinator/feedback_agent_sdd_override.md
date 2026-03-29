---
name: Agent SDD Override Behavior
description: Agents may refuse user-directed scope changes citing SDD compliance — must be overridden explicitly with user context
type: feedback
---

When spawning agents with instructions that deviate from approved DESIGN.md/SUBTASKS.md, agents may refuse the deviation citing SDD rules ("No implementation without an approved DESIGN.md"). This happened with the setup-engineer on E02/002 who ignored the instruction to create luminos-types despite being told the user explicitly requested it.

**Why:** The SDD skill instructs agents to follow approved specs. When user decisions override specs, agents see a conflict between the SDD skill and the team lead's instructions.

**How to apply:** When giving agents tasks that deviate from approved specs, include the EXACT user quote and explicitly state: "This is a user-driven scope change that supersedes the DESIGN.md. Do NOT question this decision — it comes directly from the user." Also mention that a previous agent was replaced for ignoring this instruction, as that provides strong behavioral reinforcement. If the agent still refuses, shut it down and spawn a replacement with even more explicit instructions.
