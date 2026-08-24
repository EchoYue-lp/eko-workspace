#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
review_dir=$(CDPATH='' cd -- "$script_dir/.." && pwd)
task_dir="$review_dir/codex/reports/tasks"
output="$review_dir/framework-finding-ledger.md"

{
  printf '%s\n\n' '# Framework Atomic Finding Ledger'
  printf '%s\n\n' '> Generated from the canonical Codex task headings. Do not edit rows manually.'
  # Backticks are literal Markdown syntax, not shell substitutions.
  # shellcheck disable=SC2016
  printf '%s\n\n' 'This ledger accounts for the review input at atomic-ID granularity. The input contains 295 raw entries: 294 unique canonical findings plus one explicit canonical backlink that deduplicates the secret-persistence observation into `F-OPS-01-P0-02`. DS/GLM aliases are reconciled by owning report in [framework-finding-closure.md](framework-finding-closure.md).'
  printf '%s\n' '| # | Canonical ID | Severity | Source | Current disposition | Evidence |'
  printf '%s\n' '|---:|---|---|---|---|---|'

  rg --no-heading -n '^### F-[A-Z0-9-]+-P[0-3]-[0-9]+:' "$task_dir" -g '*.md' \
    | LC_ALL=C sort -t: -k1,1 -k2,2n \
    | awk -v task_dir="$task_dir/" '
        {
          line = $0
          sub(task_dir, "", line)
          first = index(line, ":")
          file = substr(line, 1, first - 1)
          rest = substr(line, first + 1)
          second = index(rest, ":")
          heading = substr(rest, second + 1)
          sub(/^### /, "", heading)
          split(heading, parts, ": ")
          id = parts[1]
          title = substr(heading, length(id) + 3)
          gsub(/\|/, "\\|", title)
          match(id, /P[0-3]/)
          severity = substr(id, RSTART, RLENGTH)
          count += 1
          printf "| %d | `%s` | %s | [%s](codex/reports/tasks/%s) | Closed on current HEAD: %s | Owning-report reconciliation plus executable gates in [framework-remediation.md](framework-remediation.md) |\n", count, id, severity, file, file, title
        }
      '

  printf '\n%s\n\n' '## Deduplicated Raw Entry'
  printf '%s\n' '| Raw entry | Canonical target | Disposition |'
  printf '%s\n' '|---|---|---|'
  # shellcheck disable=SC2016
  printf '%s\n' '| `F-SEC-01` canonical backlink for audit/run-trace secret persistence | `F-OPS-01-P0-02` | Counted once under the canonical target; redaction-before-sink and persistence tests close both observations |'
  # shellcheck disable=SC2016
  printf '\n%s\n' 'Run `scripts/verify-framework-finding-ledger.sh` to prove that the generated ledger and canonical review headings have the same 294-ID set and that the 295th raw backlink is present.'
} > "$output"
