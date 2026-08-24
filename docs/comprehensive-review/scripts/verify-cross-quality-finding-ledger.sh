#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
review_dir=$(CDPATH='' cd -- "$script_dir/.." && pwd)
task_dir="$review_dir/codex/reports/tasks"
ledger="$review_dir/cross-quality-finding-ledger.md"
overlay="$review_dir/cross-quality-remediation.md"

canonical_ids() {
  rg --no-heading --no-filename -o \
    '^### (B|X|Q)-[A-Z0-9-]+-P[0-3]-[0-9]+' \
    "$task_dir" -g 'B-*.md' -g 'X-*.md' -g 'Q-*.md' \
    | sed 's/^### //' \
    | LC_ALL=C sort
}

ledger_rows() {
  rg --no-heading '^\| [0-9]+ \| `(B|X|Q)-[A-Z0-9-]+-P[0-3]-[0-9]+` \|' "$ledger"
}

ledger_ids() {
  # Backticks are part of the Markdown table syntax.
  # shellcheck disable=SC2016
  ledger_rows \
    | sed -E 's/^\| [0-9]+ \| `([^`]+)`.*/\1/' \
    | LC_ALL=C sort
}

canonical_count=$(canonical_ids | wc -l | tr -d ' ')
canonical_unique=$(canonical_ids | uniq | wc -l | tr -d ' ')
ledger_count=$(ledger_ids | wc -l | tr -d ' ')
ledger_unique=$(ledger_ids | uniq | wc -l | tr -d ' ')

if [[ "$canonical_count" != 75 || "$canonical_unique" != 75 || "$ledger_count" != 75 || "$ledger_unique" != 75 ]]; then
  printf 'finding count mismatch: headings=%s unique-headings=%s rows=%s unique-rows=%s\n' \
    "$canonical_count" "$canonical_unique" "$ledger_count" "$ledger_unique" >&2
  exit 1
fi

if ! diff -u <(canonical_ids) <(ledger_ids); then
  printf 'cross-layer/quality ledger ID set differs from canonical task headings\n' >&2
  exit 1
fi

invalid_dispositions=$(ledger_rows \
  | awk -F'|' '
      {
        disposition = $7
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", disposition)
        if (disposition !~ /^(fixed|stale|retained|residual|evidence-only)$/) {
          print $0
        }
      }
    ')
if [[ -n "$invalid_dispositions" ]]; then
  printf 'ledger contains missing or unsupported dispositions:\n%s\n' "$invalid_dispositions" >&2
  exit 1
fi

if rg -n '\bpending\b|TBD|TODO' "$ledger"; then
  printf 'ledger still contains placeholder evidence\n' >&2
  exit 1
fi

website_ids() {
  rg --no-heading -o '`W-[A-Z0-9-]+-P[0-3]-[0-9]+`' "$overlay" \
    | tr -d '`' \
    | LC_ALL=C sort -u
}

expected_website_ids() {
  printf '%s\n' \
    W-DEPLOY-01-P1-01 \
    W-DOC-01-P1-01 \
    W-GATE-01-P1-01 \
    W-ROUTE-01-P1-01 \
    W-SEO-01-P1-01 \
    W-SEO-01-P2-02
}

if ! diff -u <(expected_website_ids) <(website_ids); then
  printf 'website finding set differs from the six fresh review findings\n' >&2
  exit 1
fi

printf 'cross-layer/quality finding ledger verified: 75 canonical disposition rows + 6 fresh website findings\n'
