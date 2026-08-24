#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
review_dir=$(CDPATH='' cd -- "$script_dir/.." && pwd)
task_dir="$review_dir/codex/reports/tasks"
output="$review_dir/cross-quality-finding-ledger.md"

{
  printf '%s\n\n' '# Cross-Layer And Quality Atomic Finding Ledger'
  printf '%s\n\n' '> Generated skeleton from the canonical Codex task headings. Replace every pending cell with current-code evidence before validation.'
  # Backticks are Markdown literals, not shell substitutions.
  # shellcheck disable=SC2016
  printf '%s\n\n' 'This ledger accounts for the baseline (`B-*`), cross-repository (`X-*`), and quality (`Q-*`) scope at atomic-ID granularity. Cluster summaries in [cross-quality-remediation.md](cross-quality-remediation.md) are navigation only and do not close an atomic row.'
  printf '%s\n' '| # | Canonical ID | Severity | Source | Historical claim | Disposition | Current-code proof | Executed validation |'
  printf '%s\n' '|---:|---|---|---|---|---|---|---|'

  rg --no-heading -n '^### (B|X|Q)-[A-Z0-9-]+-P[0-3]-[0-9]+(:| - )' \
    "$task_dir" -g 'B-*.md' -g 'X-*.md' -g 'Q-*.md' \
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
          id = heading
          sub(/[: ].*$/, "", id)
          title = heading
          sub(/^[^: ]+(: | - )/, "", title)
          gsub(/\|/, "\\|", title)
          match(id, /P[0-3]/)
          severity = substr(id, RSTART, RLENGTH)
          count += 1
          printf "| %d | `%s` | %s | [%s](codex/reports/tasks/%s) | %s | pending | pending | pending |\n", count, id, severity, file, file, title
        }
      '

  # shellcheck disable=SC2016
  printf '\n%s\n' 'Run `scripts/verify-cross-quality-finding-ledger.sh` after replacing every pending cell. The verifier requires the exact 75-ID source set, one supported disposition per row, and the six fresh website findings recorded in the current-code overlay.'
} > "$output"
