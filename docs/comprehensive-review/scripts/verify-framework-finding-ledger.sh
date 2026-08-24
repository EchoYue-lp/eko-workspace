#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
review_dir=$(CDPATH='' cd -- "$script_dir/.." && pwd)
task_dir="$review_dir/codex/reports/tasks"
ledger="$review_dir/framework-finding-ledger.md"

canonical_count=$(rg --no-heading --no-filename '^### F-[A-Z0-9-]+-P[0-3]-[0-9]+:' "$task_dir" -g '*.md' | wc -l | tr -d ' ')
canonical_unique=$(rg --no-heading --no-filename -o '^### F-[A-Z0-9-]+-P[0-3]-[0-9]+' "$task_dir" -g '*.md' | sed 's/^### //' | LC_ALL=C sort -u | wc -l | tr -d ' ')
ledger_count=$(rg --no-heading -o '`F-[A-Z0-9-]+-P[0-3]-[0-9]+`' "$ledger" | tr -d '`' | LC_ALL=C sort -u | wc -l | tr -d ' ')

if [[ "$canonical_count" != 294 || "$canonical_unique" != 294 || "$ledger_count" != 294 ]]; then
  printf 'finding count mismatch: headings=%s unique=%s ledger=%s\n' "$canonical_count" "$canonical_unique" "$ledger_count" >&2
  exit 1
fi

if ! diff -u \
  <(rg --no-heading --no-filename -o '^### F-[A-Z0-9-]+-P[0-3]-[0-9]+' "$task_dir" -g '*.md' | sed 's/^### //' | LC_ALL=C sort -u) \
  <(rg --no-heading -o '`F-[A-Z0-9-]+-P[0-3]-[0-9]+`' "$ledger" | tr -d '`' | LC_ALL=C sort -u); then
  printf 'ledger ID set differs from canonical finding headings\n' >&2
  exit 1
fi

backlink_count=$(rg --no-heading '^### Canonical backlink: F-OPS-01-P0-02 covers raw audit/run-trace secret persistence$' "$task_dir"/F-SEC-01.md | wc -l | tr -d ' ')
if [[ "$backlink_count" != 1 ]] || ! rg -q '295 raw entries: 294 unique canonical findings plus one explicit canonical backlink' "$ledger"; then
  printf 'the 295th raw backlink is not accounted for\n' >&2
  exit 1
fi

printf 'framework finding ledger verified: 295 raw entries = 294 canonical IDs + 1 backlink\n'
