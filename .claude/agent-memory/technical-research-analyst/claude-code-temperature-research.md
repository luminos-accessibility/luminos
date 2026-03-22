# Claude Code Temperature Control Research (2026-03-21)

## Key Finding
Claude Code does NOT expose temperature as a user-configurable parameter, regardless of provider (Anthropic API, Bedrock, Vertex, Foundry).

## Verified Facts
- Claude Messages API default temperature: 1.0 (range 0.0-1.0)
- Extended thinking (enabled by default in Claude Code) is **incompatible** with temperature and top_k modifications
- With thinking enabled, only top_p can be adjusted (0.95-1.0)
- Even temperature=0.0 does not guarantee determinism (stated in Anthropic docs)
- Claude 4.1+ models reject requests with both temperature AND top_p specified simultaneously
- Anthropic changed default top_p from 0.999 to 0.99 (API release notes)

## GitHub Issues (all requesting temperature control)
- claude-code #3370: CLOSED (NOT_PLANNED)
- claude-code #6096: CLOSED (DUPLICATE of #3370)
- claude-code #9028: CLOSED
- claude-agent-sdk-python #273: OPEN (26 thumbs up, no Anthropic response)
- claude-agent-sdk-python #464: OPEN
- claude-agent-sdk-python #303: OPEN
- claude-agent-sdk-python #674: OPEN (proposes semantic "creativity" param, no response)

## What CAN Be Controlled for Stability
- `effortLevel` (low/medium/high/max) via /effort, --effort, CLAUDE_CODE_EFFORT_LEVEL, or settings
- `MAX_THINKING_TOKENS` env var (set to 0 to disable thinking entirely)
- `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING=1` (reverts to fixed thinking budget)
- CLAUDE.md instructions (advisory, ~80% compliance per community reports)
- Hooks (deterministic, 100% enforcement)
- Prompt specificity and structure

## Workaround Options (limited)
- Claude Code Router (CCR) with custom transformer plugin can inject temperature, but ONLY works when extended thinking is disabled
- Custom API proxy modifying request body — same thinking limitation
- Direct Bedrock API calls (bypasses Claude Code entirely)

## Authoritative Sources
- Claude Code settings: code.claude.com/docs/en/settings
- Claude Code env vars: code.claude.com/docs/en/env-vars
- Extended thinking docs: platform.claude.com/docs/en/docs/build-with-claude/extended-thinking
- Messages API: platform.claude.com/docs/en/api/messages
- Model config: code.claude.com/docs/en/model-config
