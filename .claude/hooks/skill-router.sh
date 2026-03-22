#!/usr/bin/env bash
set -euo pipefail

##
# Skill Router Hook for Claude Code (UserPromptSubmit)
#
# Reads the user prompt from stdin JSON, calls Claude (Haiku 4.5) to classify
# which skills should be activated, and echoes an activation instruction.
##

# Recursion guard — this hook spawns a claude subprocess which triggers hooks again.
# The env var is inherited by the child process, so the nested invocation exits immediately.
if [ "${SKILL_ROUTER_ACTIVE:-}" = "1" ]; then
  exit 0
fi
export SKILL_ROUTER_ACTIVE=1

# Read the full stdin JSON
INPUT=$(cat)

# Extract the prompt field; exit non-zero if stdin isn't valid JSON
PROMPT=$(printf '%s' "$INPUT" | jq -r '.prompt // empty') || {
  echo "skill-router: failed to parse stdin JSON" >&2
  exit 1
}

# Empty prompt is not an error — just nothing to route
if [ -z "$PROMPT" ]; then
  exit 0
fi

# Call Claude (Haiku 4.5) in non-interactive mode.
# --tools "Skill" exposes the Skill tool so the model sees all registered project skills.
# No --bare flag: skill discovery requires full initialization.
RESULT=$(claude -p \
  --output-format json \
  --no-session-persistence \
  --model haiku \
  --bare \
  --tools "" \
  --effort low \
  --system-prompt "You are a skill router for Claude Code. You will receive a user prompt. Your job is to determine which of the available skills should be activated for that prompt. Return ONLY a raw JSON array of skill names (e.g. ["skill-a","skill-b"]). Return an empty array [] if no skills are relevant. No markdown, no code fences, no explanation. Evaluate ALL the skills below: $(for i in $(ls .claude/skills/); do cat .claude/skills/$i/SKILL.md | head -20 | sed -n '/^---$/,/^---$/p'; done;)" \
  "Which skills should be activated for this prompt? ${PROMPT}") || {
  echo "skill-router: claude command failed" >&2
  exit 1
}

# Check for API errors
IS_ERROR=$(printf '%s' "$RESULT" | jq -r '.is_error // false') || {
  echo "skill-router: failed to parse claude output envelope" >&2
  exit 1
}
if [ "$IS_ERROR" = "true" ]; then
  ERROR_MSG=$(printf '%s' "$RESULT" | jq -r '.result // "unknown error"')
  echo "skill-router: API error — ${ERROR_MSG}" >&2
  exit 1
fi

# Extract the result text from the JSON envelope
RAW_RESULT=$(printf '%s' "$RESULT" | jq -r '.result // empty')
if [ -z "$RAW_RESULT" ]; then
  echo "skill-router: empty result from claude" >&2
  exit 1
fi

# Extract the JSON array from the result — handles raw JSON, markdown code blocks, or extra text.
# Collapse to single line, find the first [...] occurrence.
CLEAN_JSON=$(printf '%s' "$RAW_RESULT" | tr '\n' ' ' | grep -o '\[.*\]' | head -1)

if [ -z "$CLEAN_JSON" ]; then
  echo "skill-router: no JSON array found in model response" >&2
  exit 1
fi

# Validate the extracted JSON is a parseable array
if ! printf '%s' "$CLEAN_JSON" | jq -e 'type == "array"' >/dev/null 2>&1; then
  echo "skill-router: model response is not a valid JSON array" >&2
  exit 1
fi

# Parse array elements and join with ", "
SKILL_LIST=$(printf '%s' "$CLEAN_JSON" | jq -r '.[]' | paste -sd ', ' -)

# No skills matched — not an error, just nothing to instruct
if [ -z "$SKILL_LIST" ]; then
  exit 0
fi

echo "INSTRUCTION: ACTIVATE these skills before doing anything else: ${SKILL_LIST}"
