#!/bin/zsh
set -euo pipefail

task=docs/comprehensive-review/codex/reports/tasks/F-MAC-01.md
vdir=docs/comprehensive-review/codex/reports/validations/F-MAC-01
catalog=docs/comprehensive-review/TASKS.md

report_count=$(find "$vdir" -maxdepth 1 -type f -name 'V*.md' | wc -l | tr -d ' ')
test "$report_count" = 34
executor_count=$(rg -l '^> Executor: Codex review subagent$' "$vdir"/V*.md | wc -l | tr -d ' ')
test "$executor_count" = 34
all_executor_count=$(rg '^> Executor:' "$vdir"/V*.md | wc -l | tr -d ' ')
test "$all_executor_count" = 34

rg -q '^> Status: needs_evidence$' "$task"
rg -q '^> Reviewer: Codex review subagent$' "$task"
rg -q '^\| V07 \|.*\| passed \| \[V07-05\]' "$task"
! rg -q '\| pending \||<TASK-ID>|<VALIDATION-ID>|YYYY-MM-DD|`[A-Z]-[A-Z0-9]+-\*`' "$task"

for link in $(rg -o '\.\./validations/F-MAC-01/V[0-9-]+\.md' "$task" | sort -u); do
  test -f "$(dirname "$task")/$link"
done
for id in $(rg -o '[ABFQXS]-[A-Z0-9]+-[0-9]{2}' "$task" | sort -u); do
  rg -q "^### $id " "$catalog"
done

finding_count=$(rg '^### F-MAC-01-P[0-3]-[0-9]{2}:' "$task" | wc -l | tr -d ' ')
unique_findings=$(rg -o '^### F-MAC-01-P[0-3]-[0-9]{2}' "$task" | sort -u | wc -l | tr -d ' ')
test "$finding_count" = 6
test "$unique_findings" = 6
awk '
  /^### F-MAC-01-P[0-3]-[0-9][0-9]:/ {
    expected = substr($2, 10, 2)
    waiting = 1
    next
  }
  waiting && /^- Priority: P[0-3]$/ {
    if ($3 != expected) exit 1
    waiting = 0
  }
  END { if (waiting) exit 1 }
' "$task"

rg -q 'does not duplicate' "$task"
rg -q 'Primary must independently reproduce' "$task"
test "$(git -C echo-agent rev-parse HEAD)" = 9b0e0faf74d35c9a432370b923acabfbb5f32d63
test "$(git -C echo-agent-cli rev-parse HEAD)" = b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
test -z "$(git -C echo-agent status --short)"
cli_dirty=$(git -C echo-agent-cli status --short)
test -n "$cli_dirty"
test "$(printf '%s\n' "$cli_dirty" | wc -l | tr -d ' ')" = 38
test -z "$(printf '%s\n' "$cli_dirty" | rg -v '^ M web-frontend/src/generated/[A-Za-z0-9_]+\.ts$' || true)"
test ! -d /private/tmp/f-mac-01-target
