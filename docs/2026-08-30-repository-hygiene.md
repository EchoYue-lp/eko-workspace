# Repository Hygiene Record

Review date: 2026-08-30

## `.txt` ownership

The workspace contains 122 non-dependency `.txt` files in the current scan:

| Class | Count | Decision |
| --- | ---: | --- |
| Tracked audit evidence | 1 | Keep: `docs/comprehensive-review/codex/reports/validations/F-MAC-01/artifacts/generated-dirty-metadata.txt` is referenced by validation evidence. |
| Website publication/generation assets | 6 | Keep: `echo-website/public/{robots,llms,llms-full}.txt` and generated `dist/` counterparts are checked by the website generators. |
| EKO runtime artifacts | 115 | Keep: `.eko` traces belong to TaskRun/runtime scopes and are not deleted by extension. |

The eight `.eko/soak` roots and the detached `/private/tmp/eko-r4-final`
acceptance worktrees remain available to their validation ledgers. Their
ownership and evidence retention were not ambiguous enough to authorize
deletion in this pass.

## Empty directories removed

The following directories were empty at review time, had no active owner, and
were either rebuildable caches or source placeholders without a generator or
tracked file requirement:

- `echo-agent/.playwright-mcp/`
- `echo-agent/src/handoff/`
- `echo-agent/src/notebook/`
- `echo-agent-cli/.claude/worktrees/`
- `echo-agent-cli/chrome-extension/`
- `echo-agent-cli/echo-agent-app-core/src/tasks/pipelines/`
- `echo-agent-cli/echo-agent-app-core/web-frontend/src/`
- `echo-agent-cli/echo-agent-app-core/target/test-tmp/`
- `echo-agent-cli/echo-agent-app-core/web-frontend/src/generated/`
- `echo-agent-cli/evals/`
- `echo-agent-cli/src/bin/`
- `echo-agent-cli/web-frontend/src/components/changes/`
- `echo-agent-cli/web-frontend/src/components/chat/tools/`
- `echo-agent-cli/web-frontend/src/components/notebook/`
- `echo-agent-cli/web-frontend/src/components/permissions/`
- `echo-agent-cli/web-frontend/src/components/runtime/`
- `echo-agent-cli/web-frontend/node_modules/.vite-temp/`
- `echo-website/.worktrees/`
- `docs/architecture/`
- `echo-agent/docs/zh/adr/`

Deletion uses `rmdir` semantics: a target that is no longer empty is left in
place rather than recursively removed. Empty-directory deletion is not a code
behavior change; future work recreates a source directory with its first real
file.

## Cargo caches

Before cleanup, the two owned Cargo target directories occupied approximately
22 GiB (`echo-agent/target`) and 20 GiB (`echo-agent-cli/target`), while free
space was about 15 GiB. No repository Cargo build process was active. After all
validation completed, `cargo clean` removed 23.8 GiB from `echo-agent` and
24.7 GiB from `echo-agent-cli`; only build caches were removed.

## Protected scope

No `.eko` runtime scope, user artifact, journal, trace, release evidence,
`.git` internals, non-empty dependency package, or website publication asset was
deleted. Any future runtime cleanup must identify a complete terminal scope,
check live handles and recovery debt, and confirm that no ledger or user history
still references it.
