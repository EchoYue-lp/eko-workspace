#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
review_dir=$(CDPATH='' cd -- "$script_dir/.." && pwd)

validate_root() {
  local validation_root="$1"
  local record_count=0
  local seen_file
  seen_file=$(mktemp)

  while IFS= read -r report; do
    local task_id filename stem key attempt header_key header_attempt
    task_id=$(basename -- "$(dirname -- "$report")")
    filename=$(basename -- "$report")
    stem=${filename%.md}
    if [[ ! "$stem" =~ ^(.+)--attempt-([0-9]+)$ ]]; then
      printf 'invalid validation-v2 filename: %s\n' "$report" >&2
      rm -f -- "$seen_file"
      return 1
    fi
    key=${BASH_REMATCH[1]}
    attempt=${BASH_REMATCH[2]}
    if [[ "$attempt" == 0 ]]; then
      printf 'attempt must be positive: %s\n' "$report" >&2
      rm -f -- "$seen_file"
      return 1
    fi
    if ! rg -q '^> Schema: validation-v2$' "$report"; then
      printf 'missing validation-v2 schema header: %s\n' "$report" >&2
      rm -f -- "$seen_file"
      return 1
    fi
    header_key=$(sed -n 's/^> Validation key: //p' "$report")
    header_attempt=$(sed -n 's/^> Attempt: //p' "$report")
    if [[ "$header_key" != "$key" || "$header_attempt" != "$((10#$attempt))" ]]; then
      printf 'path/header lineage mismatch: %s\n' "$report" >&2
      rm -f -- "$seen_file"
      return 1
    fi
    printf '%s\t%s\t%s\n' "$task_id" "$key" "$((10#$attempt))" >>"$seen_file"
    record_count=$((record_count + 1))
  done < <(
    rg -l '^> Schema: validation-v2$' "$validation_root" \
      -g '**/reports/validations/**/*.md' | LC_ALL=C sort
  )

  local duplicate_count
  duplicate_count=$(LC_ALL=C sort "$seen_file" | uniq -d | wc -l | tr -d ' ')
  rm -f -- "$seen_file"
  if [[ "$duplicate_count" != 0 ]]; then
    printf 'duplicate validation key/attempt identity detected\n' >&2
    return 1
  fi
  printf 'validation-v2 lineage verified: %s immutable report(s)\n' "$record_count"
}

self_test() {
  local fixture_root
  fixture_root=$(mktemp -d)
  local report_root="$fixture_root/codex/reports/validations/Q-FIXTURE"
  mkdir -p -- "$report_root"
  local spec key attempt status
  for spec in \
    'V01 1 failed' \
    'V01 2 inconclusive' \
    'V01 3 passed' \
    'V02-01 1 passed' \
    'V02-02 1 passed' \
    'V02-03 1 passed'; do
    read -r key attempt status <<<"$spec"
    printf '# Q-FIXTURE / %s / Attempt %s\n\n> Schema: validation-v2\n> Validation key: %s\n> Attempt: %s\n> Status: %s\n' \
      "$key" "$attempt" "$key" "$attempt" "$status" \
      >"$report_root/${key}--attempt-0${attempt}.md"
  done
  validate_root "$fixture_root"
  printf '# invalid\n\n> Schema: validation-v2\n> Validation key: V03\n> Attempt: 2\n' \
    >"$report_root/V03--attempt-01.md"
  if validate_root "$fixture_root" >/dev/null 2>&1; then
    printf 'validation-v2 negative control unexpectedly passed\n' >&2
    rm -rf -- "$fixture_root"
    return 1
  fi
  rm -rf -- "$fixture_root"
  printf 'validation-v2 negative control rejected a path/header mismatch\n'
}

if [[ "${1:-}" == "--self-test" ]]; then
  self_test
else
  validate_root "${1:-$review_dir}"
fi
