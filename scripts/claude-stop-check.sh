#!/usr/bin/env bash
# Claude Code Stop hook (.claude/settings.json): Claude cannot finish a task
# while `make check` fails. Skipped when the working tree is clean.
#
# Exit 2 = block the stop and hand the failure back to Claude. If Claude was
# already continuing because of this hook (stop_hook_active), it reports the
# failure to the user instead of blocking again, to avoid an endless loop.
set -uo pipefail
cd "$(dirname "$0")/.."

input="$(cat)"

if [ -z "$(git status --porcelain 2>/dev/null)" ]; then
  exit 0
fi

if output="$(make check 2>&1)"; then
  exit 0
fi

summary="$(printf '%s\n' "$output" | tail -n 40)"

if printf '%s' "$input" | grep -q '"stop_hook_active"[[:space:]]*:[[:space:]]*true'; then
  python3 -c 'import json,sys; print(json.dumps({"systemMessage": "make check is still failing:\n" + sys.argv[1]}))' "$summary"
  exit 0
fi

printf 'make check failed. Fix it before finishing:\n%s\n' "$summary" >&2
exit 2
