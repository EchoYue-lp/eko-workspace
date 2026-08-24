# Q-DOC-01: Current public and operator documentation validation

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-13
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Do README, feature/config references, examples, EKO setup docs, and
architecture claims match the reviewed code and executable commands?

## Scope

Primary documentation surfaces inspected (public + operator):

- `echo-agent/README.md` (1241 lines) — Quick Start, two Feature Flags
  tables, Workspace Structure, Core Concepts, Macro Reference, Examples
  table, Documentation link table, Contributing commands.
- `echo-agent/README.zh.md` — Workspace Structure (spot-checked vs EN).
- `echo-agent/examples/README.md` — example classification (acceptance /
  conditional / teaching).
- `echo-agent-cli/README.md` (611 lines) — project structure, build/install
  commands (TUI + GUI), Feature Flags table, "echo-agent 依赖 Features"
  table, CLI argument table, architecture diagram, workspace layout.
- `echo-agent-cli/docs/getting-started.md` — EKO setup: install, onboard,
  CLI subcommands, headless mode, config.
- `echo-agent-cli/docs/configuration.md` and
  `echo-agent-cli/docs/architecture.md` — spot-checked for SQLite / feature
  / path claims.
- 5 sampled examples from `echo-agent/examples/` (demo05, demo25, demo42,
  demo57, demo70).

Canonical manifests used as ground truth:

- `echo-agent/Cargo.toml` — `[features]` (lines 65-103), workspace members
  (lines 2-10), `[[example]]` registrations.
- `echo-agent-cli/Cargo.toml` — `[features]` (lines 21-37), `echo-agent`
  dependency features (line 50), `[[bin]]` definitions.
- `echo-agent-cli/src/cli/args.rs` — the clap CLI argument surface.
- `echo-agent-cli/.cargo/config.toml` — cargo aliases.

## Out Of Scope

Deferred to named task IDs:

- Sub-crate README deep audit (echo-core/echo-state/echo-orchestration/
  echo-integration example breaks and missing-contents lists) → already
  covered by B-ARCH-01-P2-03/P2-04/P3-01/P3-02/P3-03 and inherited by
  F-API-01.
- The 40+ dated design docs under `echo-agent-cli/docs/2026-07-*` and
  `docs/superpowers/plans/*` → historical design notes, not "current
  public/operator documentation".
- Per-feature build-matrix compile checks → F-FEAT-01.
- Tool-count verification ("67 registered tools") → F-EXT-01.
- Macro-expansion correctness for example code → F-MAC-01.
- AUDIT_REPORT.md code-anchor drift → B-DOC-01.
- The full bilingual doc body under `echo-agent/docs/{en,zh}/` → only the
  README Documentation link table's targets were link-checked; per-doc
  content accuracy is not re-audited here.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  system reminder — particularly the "只有 Subagent,没有 Worker" rule,
  "echo-agent-cli 不需要 SQLite" rule, and the three-project positioning
  table).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full) and the task/validation templates.

Dependency task reports read:

- `zcode-glm/tasks/B-DOC-01.md` (in full) — historical-audit/drift index;
  its P2-02 (echo-agent-eval stale) and P3-01 (MASTER-PLAN anchors current)
  directly inform Q-DOC-01's stale-term and link checks.
- `zcode-glm/tasks/F-API-01.md` (in full) — public-facade contract; its
  P3-03 (root examples compile) and P3-05 ("67 tools" README claim) set the
  examples-sampling and feature-claim baselines that Q-DOC-01 extends.
- `zcode-glm/validations/B-DOC-01/V01-01.md` (in full) — for the
  `improve/store.rs` removed-path precedent and AUDIT_REPORT drift method.

Historical documents treated as hypotheses: the READMEs' implicit promises
that (a) every advertised example/doc file exists, (b) feature tables match
the manifest, and (c) the documented CLI commands actually run.

## Layering Decision

This is a documentation task spanning both repositories and all doc layers.
No code movement is recommended. The findings classify each defect by the
layer the *documented subject* lives in: the echo-agent README feature
table is framework-layer documentation; the CLI README/sqlite claims and
getting-started commands are application-layer (EKO) documentation. The
sqlite finding (Q-DOC-01-P2-02) explicitly invokes AGENTS.md's
"echo-agent-cli 不需要 SQLite" product-positioning rule as the invariant
the documentation violates.

Repository-wide duplicate-search was not required (documentation validation,
not authority migration). Stale-term grep covered both repos' `docs/`
trees, both READMEs (EN + ZH), AGENTS.md, and the root MASTER-PLAN.

## Current Path

The documentation landscape (confirmed via filesystem + manifest reads):

- Public framework docs: `echo-agent/README.md` (+ `.zh.md`), 45 files
  under `echo-agent/docs/{en,zh}/`, 7 sub-crate READMEs, `examples/README.md`.
- Public/operator app docs: `echo-agent-cli/README.md`,
  `docs/getting-started.md`, `docs/configuration.md`, `docs/architecture.md`,
  `docs/gui-status.md`, `docs/MASTER-PLAN.md`, 7 `system-deep-dive/` files.
- Canonical feature truth: `echo-agent/Cargo.toml [features]` (33
  individual features; `full` enables 27; `default = []`),
  `echo-agent-cli/Cargo.toml [features]` (5 features; `default = ["tui"]`).
- Canonical CLI-command truth: `echo-agent-cli/src/cli/args.rs` — a flat
  clap `Parser` with 14 flags and NO `#[command(subcommand)]` field, i.e.
  the CLI accepts only flags, no subcommands.

## Findings

### Q-DOC-01-P2-01: echo-agent README feature-flag tables are materially inaccurate

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/README.md:113-141` (Feature Flags table 1, "In `full`?")
    and `:246-277` (Feature Flags table 2).
  - `echo-agent/Cargo.toml:65-103` — canonical `[features]`.
- Reachability: the README is the primary public surface (crates.io /
  GitHub landing page). A user copying `features = ["plan-execute"]` from
  the table gets a Cargo error (no such feature). A user trusting the
  "in full?" column mis-predicts what `features = ["full"]` enables.
- Expected invariant: README feature tables enumerate the real features
  and their `full` membership accurately.
- Observed behavior:
  - **Phantom features**: `plan-execute` and `self-reflection` are listed
    in BOTH tables but are NOT defined in `Cargo.toml [features]` (no such
    features exist).
  - **Wrong `full`-membership flags (table 1)**: `research`, `database`,
    `content-guard`, `project-rules` are marked "in full = no" but ARE in
    `full` (`Cargo.toml:67`); `sandbox` is marked "yes" but is NOT in
    `full`.
  - **Omitted real features**: table 1 omits `lsp` (which IS in `full`)
    plus `statistics`, `eval`, `improve`, `testing`, `semantic-memory`,
    `macros`, `provider-factory`, `workflow`, `multimodal`. Table 2 omits
    `lsp`, `statistics`, `eval`, `improve`, `testing`.
- Impact: users mis-configure Cargo features; the "zero default features"
  promise (line 104) is undercut by the "Full (default)" comment (line 240)
  in the same README. Reviewers cannot trust the tables without opening the
  manifest.
- Root cause: the tables predate the per-feature modularisation and the
  `plan-execute`/`self-reflection` rename/removal, and were not refreshed.
- Direction: regenerate both feature tables from `Cargo.toml [features]`;
  remove `plan-execute`/`self-reflection`; correct the `full` flags; add
  the omitted features.
- Regression validation: after the edit, diff the table feature set against
  `Cargo.toml [features]` keys; the two sets must match exactly.
- Validation reports: [V03](../validations/Q-DOC-01/V03-01.md).

### Q-DOC-01-P2-02: echo-agent-cli docs falsely claim SQLite-backed session/memory persistence and an enabled sqlite feature

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/README.md:56` — `sessions/  # 会话管理（SQLite + FTS）`
  - `echo-agent-cli/README.md:239` — "echo-agent 依赖 Features" table row
    `| sqlite | SQLite 会话存储 |`
  - `echo-agent-cli/README.md:495` — architecture diagram lists `sqlite`
    among echo-agent features
  - `echo-agent-cli/README.md:557` — `sessions/  # 会话历史（SQLite + FTS）`
  - `echo-agent-cli/docs/architecture.md:160,190,222` — "会话历史持久化
    （SQLite）", "SQLite（默认）", "memory/  # 记忆存储 (SQLite)"
  - `echo-agent-cli/Cargo.toml:50` — `echo-agent` dep features =
    `["mcp","lsp","human-loop","subagent","tasks"]` (NO sqlite, NO eval,
    NO improve)
  - `echo-agent-cli/echo-agent-app-core/src/sessions/search.rs:3` —
    `//! U1c: EKO is local — no SQLite/FTS5.` (in-memory substring index)
  - `AGENTS.md` — "echo-agent-cli 不需要 SQLite … 禁止给 echo-agent-cli
    引入或保留 SQLite 依赖"
- Reachability: the README and architecture doc are the operator-facing
  setup/design references. A contributor reading them believes EKO uses
  SQLite and may re-introduce a SQLite dependency, directly violating
  AGENTS.md.
- Expected invariant: operator documentation's persistence/feature claims
  match the actual implementation and the AGENTS.md product-positioning
  rule (echo-agent-cli = file/memory-backed, no SQLite).
- Observed behavior: 7 distinct doc locations claim SQLite/FTS for sessions
  or memory, and the README's "echo-agent 依赖 Features" table lists
  `sqlite` (plus `eval`, `improve`) as enabled echo-agent features. None
  of the three is enabled in `Cargo.toml:50`; the session search module
  explicitly states it does NOT use SQLite/FTS5.
- Impact: misleading architecture picture; invites AGENTS.md-violating
  changes; the "echo-agent 依赖 Features" table over-states enabled
  features by 3 (sqlite, eval, improve).
- Root cause: the docs predate the "remove SQLite from echo-agent-cli"
  decision and the migration of session search to an in-memory engine;
  they were not refreshed. (Same drift class as the echo-agent feature
  tables.)
- Direction: replace "SQLite + FTS" with "file-backed (in-memory index)"
  in README:56, :557 and architecture.md:160,222; drop the `sqlite`,
  `eval`, `improve` rows from the "echo-agent 依赖 Features" table (or
  correct them to the 5 actually-enabled features); reconcile
  architecture.md:190,495.
- Regression validation: after the edit, grep `SQLite|sqlite|FTS` across
  `echo-agent-cli/README.md` and `docs/architecture.md`; session/memory
  persistence claims must not mention SQLite.
- Validation reports: [V03](../validations/Q-DOC-01/V03-01.md).
- Note: the database-TOOLS references (`getting-started.md:151`,
  `architecture.md:348` — "数据库: sqlite, postgresql, mysql") describe the
  optional SQL query tools, not session persistence, and are out of scope
  for this finding.

### Q-DOC-01-P2-03: echo-agent-cli getting-started.md documents non-existent CLI subcommands and a broken GUI command

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/docs/getting-started.md:35` — `cargo run --bin
    echo-agent-cli -- onboard` (onboard wizard)
  - `:95` — `echo-agent-cli run "解释这段代码"`
  - `:97` — `echo-agent-cli --headless "写一个快速排序并测试"`
  - `:100` — `echo-agent-cli --model qwen3.7-max run "…"`
  - `:103` — `echo-agent-cli --config ./my-config.yaml run "…"`
  - `:128-131` — `echo-agent-cli sessions list/show/export/delete`
  - `:87-88` — GUI start: `cargo run --bin echo-agent-tauri` (no features)
  - `echo-agent-cli/src/cli/args.rs:8-68` — the clap `Args` struct defines
    ONLY flags (`--tui`, `--model`, `--config`, `--mcp-config`, `--project`,
    `--continue`, `--resume`, `--verbose`, …); there is NO
    `#[command(subcommand)]` field, NO `--headless` flag, NO `onboard`/
    `run`/`sessions` subcommands.
  - `echo-agent-cli/Cargo.toml:43-46` — `[[bin]] echo-agent-tauri …
    required-features = ["gui"]`; with `default = ["tui"]`, `cargo run
    --bin echo-agent-tauri` errors ("requires the features: gui").
  - Contrast: `echo-agent-cli/README.md:420-428` — the README's own CLI
    parameter table lists ONLY the flags that actually exist (it is
    correct), so getting-started.md contradicts the README.
- Reachability: getting-started.md is the first-run operator doc. Every
  command in the list above errors when copied verbatim.
- Expected invariant: documented setup/run commands resolve against the
  actual CLI parser and compile.
- Observed behavior: 6 documented commands reference a subcommand/flag
  surface (`run`, `sessions`, `onboard`, `--headless`) that does not exist
  in `args.rs`; the GUI command omits the required `gui` feature and
  errors under default features.
- Impact: new users following the setup guide hit immediate failures on
  every documented CLI subcommand and on the GUI start command; the guide
  is unreliable as a first-run path.
- Root cause: getting-started.md predates a CLI redesign that removed the
  subcommand surface (the CLI is now flag-only, TUI-by-default) and the
  `gui`-feature requirement on the Tauri binary; it was not refreshed. The
  README was updated but the setup doc was not.
- Direction: rewrite the "命令行模式" and "启动 GUI" sections of
  getting-started.md against the real `args.rs` surface (flag-only; GUI via
  `cargo tauri dev` / `cargo run --bin echo-agent-tauri --no-default-features
  --features gui`, matching README:153-159). Remove the `onboard`/`run`/
  `sessions`/`--headless` examples unless those commands are reintroduced.
- Regression validation: after the edit, copy-execute each documented
  command with `--help`/dry-run to confirm it parses; cross-check the
  flag set against `args.rs`.
- Validation reports: [V01](../validations/Q-DOC-01/V01-01.md) (path
  integrity), with command-surface evidence inline above.

### Q-DOC-01-P2-04: echo-agent README contains broken doc/example links, a phantom workspace crate, and the CLI README links a missing LICENSE

- Priority: P2
- Confidence: high
- Layer: framework (echo-agent README/README.zh.md) + application (CLI LICENSE)
- Evidence:
  - `echo-agent/README.md:1170,1173` (+ README.zh.md) — Documentation
    table links `[EN](docs/en/16-plan-execute.md)`,
    `[EN](docs/en/19-self-reflection.md)`; neither file exists in
    `docs/en/` or `docs/zh/` (verified: directory sequence jumps 14 → 15
    → 17 → 18 → 20).
  - `echo-agent/README.md:1078,1080,1086` — examples table rows for
    `demo14_memory_isolation.rs`, `demo16_testing.rs`,
    `demo22_plan_execute.rs`; none of the three files exist (actual set is
    demo00–demo70 with 14/16/22/63 absent; total 67 demoXX + 1 smoke =
    68 .rs files).
  - `echo-agent/examples/README.md` — lists the same three missing files
    in its Acceptance/Conditional/Teaching buckets.
  - `echo-agent/README.md:291` + `README.zh.md:252` — Workspace Structure
    diagram line `├── echo-agents/  Agent implementations: ReactAgent,
    PlanExecute, Subagent`; `echo-agent/Cargo.toml:2-10` declares 7
    workspace members, no `echo-agents` (the directory does not exist).
  - `echo-agent-cli/README.md:6,601` — badge + footer link `LICENSE`;
    echo-agent-cli has no `LICENSE` file (echo-agent root does).
  - `echo-agent/examples/demo42_playwright_mcp.rs:22,28` — instructs
    `cp examples/mcp.json.example mcp.json`; the file is at repo root
    (`./mcp.json.example`), not under `examples/`.
- Reachability: README/examples-README are the primary public surfaces;
  broken links and a phantom crate mislead every first-time reader.
- Expected invariant: public-doc relative links resolve; the workspace
  diagram matches the manifest's member list.
- Observed behavior: 4 broken doc links, 3 broken example links, 1 phantom
  crate, 1 missing LICENSE, 1 wrong asset path — 10 distinct broken
  relative-path targets.
- Impact: users clicking the doc table hit 404s; users running the
  examples-table commands for demo14/16/22 get "no example target";
  contributors are confused by a non-existent 8th crate.
- Root cause: docs predate file removals (16/19 doc files and demo14/16/22
  examples were deleted or never created; `echo-agents` was folded into
  root `src/agent/` + echo-orchestration) and were not refreshed.
- Direction: remove the dead rows (or restore the files if they are
  intended); correct the workspace diagram to the 7 real crates + `src/`;
  add a `LICENSE` file to echo-agent-cli (or drop the badge/footer link);
  fix the demo42 `cp` path to `cp mcp.json.example`.
- Regression validation: after the edit, link-check all README relative
  paths (`grep -oE '\]\([^)]+\)'` + `test -e`); the workspace diagram must
  match `Cargo.toml` members.
- Validation reports: [V01](../validations/Q-DOC-01/V01-01.md),
  [V04](../validations/Q-DOC-01/V04-01.md).

### Q-DOC-01-P3-01: echo-agent README example-count claims are inconsistent and wrong

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/README.md:294` — `├── examples/  64 runnable demos`
  - `echo-agent/README.md:389` — `**66 runnable examples**`
  - Filesystem truth: 67 `demoXX.rs` files + 1 `smoke_usage_passthrough.rs`
    = 68 example `.rs` files.
- Reachability: README headline claims.
- Expected invariant: the two count claims agree with each other and with
  the actual file count.
- Observed behavior: the two claims contradict each other (64 vs 66) and
  both are wrong (actual 67 demoXX / 68 total).
- Impact: minor credibility nit; reviewers notice the internal
  contradiction.
- Root cause: counts were updated piecemeal as examples were added; the
  two locations drifted apart and from reality.
- Direction: replace both with the current count (e.g. "67 demo examples")
  or derive dynamically; keep the two locations in sync.
- Regression validation: after the edit, `ls examples/demo*.rs | wc -l`
  must equal the stated count.
- Validation reports: [V01](../validations/Q-DOC-01/V01-01.md) (context).

### Q-DOC-01-P3-02: echo-agent README "Full (default)" inline comment contradicts `default = []`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/README.md:104` — "The crate ships with **zero default
    features** (`default = []`)" (correct).
  - `echo-agent/README.md:239-241` — comment `# Full (default) — all
    features enabled` over `echo-agent = "0.2.0"` (the bare dependency
    enables `default = []`, i.e. ZERO features, not `full`).
  - `echo-agent/Cargo.toml:66` — `default = []`.
- Reachability: README Feature Flags section.
- Expected invariant: the inline comment matches the manifest's `default`.
- Observed behavior: the comment implies bare `echo-agent = "0.2.0"` gives
  all features; it gives none.
- Impact: users expecting a working agent from the bare dependency get a
  no-op build (no tools, no providers); contradicts line 104 in the same
  file.
- Root cause: leftover from an earlier `default = ["full"]` configuration
  (or copy-paste from a different convention).
- Direction: change the comment to "Minimal — default features only
  (ReAct engine core)" or remove it; align with line 104.
- Regression validation: after the edit, the comment must not imply
  `default = full`.
- Validation reports: [V03](../validations/Q-DOC-01/V03-01.md).

### Q-DOC-01-P3-03: AGENTS.md still references echo-agent-eval as a current submodule (re-flag of B-DOC-01-P2-02)

- Priority: P3
- Confidence: high
- Layer: application (positioning doc)
- Evidence:
  - `AGENTS.md:139` — "三个项目的定位" table lists `echo-agent-eval(评测)`
    as a submodule of echo-agent-cli.
  - `AGENTS.md:370` — worktree Cargo.toml path rule references
    `echo-agent-cli/echo-agent-eval/Cargo.toml`.
  - Filesystem: `echo-agent-cli/echo-agent-eval/` does not exist; CLI
    workspace members = `["echo-agent-app-core"]` only.
- Reachability: AGENTS.md is the highest-priority instruction file read by
  every reviewer/agent.
- Expected invariant: AGENTS.md accurately describes current project
  structure.
- Observed behavior: presents a removed module as current.
- Impact: reviewers/agents waste time looking for a non-existent module;
  the worktree-path rule points at a non-existent Cargo.toml.
- Root cause: echo-agent-eval was removed (its role folded into the
  framework's eval feature); AGENTS.md was not updated.
- Direction: remove `echo-agent-eval` from the positioning table and the
  worktree-path rule (the eval capability is now `echo-agent`'s `eval`
  feature).
- Regression validation: after the edit, `grep echo-agent-eval AGENTS.md`
  returns zero "presents-as-current" hits.
- Validation reports: [V04](../validations/Q-DOC-01/V04-01.md).
- Note: B-DOC-01-P2-02 flagged this at the 2026-08-12 baseline; it remains
  unfixed at the 2026-08-13 Q-DOC-01 baseline. Re-flagged because Q-DOC-01
  re-ran the stale-term grep and the defect persists.

### Q-DOC-01-P3-04: Documentation is free of `worker` terminology (positive finding)

- Priority: P3
- Confidence: high
- Layer: both
- Evidence: `grep -rni 'worker[s]?'` across `echo-agent/docs/`, both
  READMEs, `echo-agent-cli/docs/`, CLI README → 0 real concept hits (the
  only raw matches were `NetworkError` strings in mock-testing docs).
- Reachability: n/a (positive).
- Expected invariant (AGENTS.md "只有 Subagent,没有 Worker"): documentation
  uses only `Subagent`/`subagent` terminology.
- Observed behavior: the invariant holds across all inspected docs. The
  subagent unification is fully reflected in the documentation layer.
- Impact: none (positive confirmation; contrasts with the echo-agent-eval
  and echo-agents stale terms, which do not hold).
- Validation reports: [V04](../validations/Q-DOC-01/V04-01.md).

### Q-DOC-01-P3-05: Sampled root examples reference current APIs (positive finding)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: 5 examples (demo05_compressor, demo25_macros,
  demo42_playwright_mcp, demo57_data_pipeline, demo70_scheduler) — every
  imported type, macro, builder method, and AgentEvent variant resolves to
  a real definition in the current tree (full per-symbol table in V02-01).
- Reachability: examples are registered as `[[example]]` in
  `echo-agent/Cargo.toml` and runnable via `cargo run --example <name>
  --features <…>`.
- Expected invariant: canonical examples track the current public API.
- Observed behavior: they do — including the newest demo70 (scheduler)
  and the workflow/pipelines API (demo57). This extends F-API-01-P3-03
  (which covered demo00-03) to 5 additional examples.
- Impact: the root examples are reliable API documentation; the only nits
  are non-API (demo42's `cp examples/mcp.json.example` path slip and its
  registered name `demo42_browser_mcp` ≠ filename, tracked in V01-01).
- Validation reports: [V02](../validations/Q-DOC-01/V02-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | README/example/doc relative-path integrity | yes | failed | [V01-01](../validations/Q-DOC-01/V01-01.md) |
| V02 | 5 sampled echo-agent examples reference current API | yes | passed | [V02-01](../validations/Q-DOC-01/V02-01.md) |
| V03 | Feature/config tables match Cargo.toml | yes | failed | [V03-01](../validations/Q-DOC-01/V03-01.md) |
| V04 | Docs free of stale worker / echo-agent-eval / phantom-path terms | yes | failed | [V04-01](../validations/Q-DOC-01/V04-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| echo-agent README: "zero default features (`default = []`)" (line 104) | current | `Cargo.toml:66 default = []` (V03) |
| echo-agent README: "# Full (default) — all features enabled" (line 240) | stale | `default = []`, not full; contradicts line 104 (Q-DOC-01-P3-02) |
| echo-agent README feature tables (lines 113-141, 246-277) | stale | 2 phantom features, 5 wrong `full` flags, ~10-14 omitted (Q-DOC-01-P2-01) |
| echo-agent README: "64 runnable demos" / "66 runnable examples" (294, 389) | stale | actual 67 demoXX / 68 total; claims also contradict each other (Q-DOC-01-P3-01) |
| echo-agent README Workspace Structure: `echo-agents/` crate (291, zh:252) | stale | no such crate; 7 real members (Q-DOC-01-P2-04, V04) |
| echo-agent README Documentation table: 16-plan-execute.md, 19-self-reflection.md | stale | files do not exist (Q-DOC-01-P2-04, V01) |
| echo-agent README examples table: demo14/16/22 | stale | files do not exist (Q-DOC-01-P2-04, V01) |
| echo-agent-cli README: "echo-agent 依赖 Features" includes sqlite/eval/improve (239-247) | stale | `Cargo.toml:50` enables only mcp/lsp/human-loop/subagent/tasks (Q-DOC-01-P2-02, V03) |
| echo-agent-cli README + architecture.md: sessions/memory are "SQLite + FTS" (56,557,arch:160,190,222) | stale | `sessions/search.rs:3` "no SQLite/FTS5"; AGENTS.md forbids SQLite (Q-DOC-01-P2-02) |
| echo-agent-cli getting-started.md: `onboard`/`run`/`sessions`/`--headless` CLI commands | stale | `args.rs` is flag-only, no subcommands/`--headless` (Q-DOC-01-P2-03) |
| echo-agent-cli getting-started.md: GUI via `cargo run --bin echo-agent-tauri` (87) | stale | binary requires `gui` feature; command errors under default features (Q-DOC-01-P2-03) |
| echo-agent-cli README: CLI parameter table (420-428) | current | matches `args.rs` flags exactly (V03) |
| echo-agent-cli README: Feature Flags table tui/gui/channels/telemetry/devtools (225-231) | current | matches `Cargo.toml:21-37` (V03) |
| echo-agent-cli README: GUI auto-includes channels; cargo aliases gui-dev/gui-bundle/gui-build/gui-run | current | `Cargo.toml:33` gui→channels; `.cargo/config.toml` aliases all present (verified) |
| AGENTS.md: echo-agent-eval is a current submodule (139, 370) | stale | directory absent; re-flag of B-DOC-01-P2-02 (Q-DOC-01-P3-03, V04) |
| F-API-01-P3-03: root examples compile conceptually (demo00-03) | current (extended) | Q-DOC-01-V02 extends to demo05/25/42/57/70 — all resolve (Q-DOC-01-P3-05) |
| AGENTS.md: "只有 Subagent,没有 Worker" applied to docs | current | 0 worker concept hits in docs (Q-DOC-01-P3-04, V04) |

## Coverage And Uncertainty

Documentation not inspected deeply:

- The 45 files under `echo-agent/docs/{en,zh}/` — only the README
  Documentation table's link targets were existence-checked; per-doc
  content accuracy (API examples inside each guide) is not re-audited.
  F-API-01 noted sub-crate README example drift; the `docs/{en,zh}/`
  guides may have similar drift but are out of Q-DOC-01's scoped surface
  (README + setup + feature/config + examples + architecture claims).
- The 40+ dated design docs under `echo-agent-cli/docs/2026-07-*` and
  `docs/superpowers/plans/*` — historical design notes; intentionally
  excluded (not "current public/operator documentation").
- Sub-crate READMEs (echo-core/echo-state/echo-orchestration/
  echo-integration/echo-tools/echo-execution/echo-macros) — owned by
  B-ARCH-01-V04; not re-audited.
- `echo-agent-cli/docs/configuration.md` body — spot-checked (no
  cargo-command or SQLite-persistence claims found in the grepped
  patterns); a full config-key audit was not performed.
- `web-frontend/README.md` and `docs/gui-status.md` — not audited
  (front-end surface; owned by A-FE-* and A-OUT-* tasks).

Validations not run:

- No `cargo run --example`, `cargo build`, or CLI command execution
  (read-only review). The CLI-command defects (Q-DOC-01-P2-03) are
  established by diffing the documented commands against the clap parser
  in `args.rs` and the `required-features` on the Tauri `[[bin]]`, which
  is a robust static determination (a `#[derive(Parser)]` with no
  subcommand field cannot accept positional subcommands; a binary with
  `required-features = ["gui"]` cannot build under `default = ["tui"]`).

Claims that remain uncertain:

- The "67 registered tools" README claim (lines 181, 388) was not
  independently recounted — owned by F-EXT-01. F-API-01-P3-05 already
  flagged its "accessible through a single prelude::*" qualifier as
  overstated; Q-DOC-01 does not re-litigate it.
- Whether the `Key Dependencies` column of echo-agent README feature
  table 2 is exhaustive — only feature names and `full`-membership were
  diffed (V03).
- The exact intended fate of `plan-execute`/`self-reflection` (renamed?
  removed? merged into `tasks`/`subagent`?) — the direction assumes
  removal, but a maintainer may prefer to re-add them. Either way the
  current tables are wrong.

## Handoff

Conclusions downstream tasks may rely on:

- The root `examples/*.rs` are reliable API documentation: 5/5 sampled
  examples (including the newest demo70) reference current APIs (V02).
  F-EXT-01 / F-MAC-01 / F-FEAT-01 may use the examples as a trusted API
  reference for the human-loop, macro, mcp, workflow/pipeline, and
  scheduler surfaces.
- The public feature/config documentation is NOT reliable: echo-agent
  feature tables, the CLI "依赖 Features" table, and the SQLite
  persistence claims all drift from the manifests and AGENTS.md (V03).
  Any task citing feature membership or CLI persistence MUST read
  `Cargo.toml` directly, not the README.
- The CLI is flag-only (no subcommands, no `--headless`) — downstream
  tasks documenting or testing CLI invocation must use the `args.rs`
  surface, not getting-started.md.
- `worker` terminology is clean in docs; the subagent unification holds
  at the documentation layer (V04). The remaining stale terms are
  `echo-agent-eval` (AGENTS.md) and the `echo-agents` phantom crate
  (echo-agent README).

Reports downstream tasks must read:

- F-EXT-01 (tool inventory) should read Q-DOC-01-P3-05 / V02 — the
  examples are a trusted source for which tools/APIs are current.
- F-FEAT-01 (feature topology) should read V03 — the README-vs-manifest
  feature diff is a precondition for any feature-gate compile matrix
  (the README cannot be used as the feature list).
- Any EKO operator/onboarding task must read Q-DOC-01-P2-03 — the
  getting-started CLI commands are wrong and must not be copied.

Conditions that make this report stale:

- Any commit that regenerates the echo-agent README feature tables from
  `Cargo.toml` invalidates Q-DOC-01-P2-01 and P3-02.
- Any commit that removes the SQLite claims from the CLI README/
  architecture.md and corrects the "依赖 Features" table invalidates
  Q-DOC-01-P2-02.
- Any commit that rewrites getting-started.md against the real `args.rs`
  surface invalidates Q-DOC-01-P2-03.
- Any commit that removes the dead doc/example links and the phantom
  `echo-agents` crate line, and adds a CLI LICENSE, invalidates
  Q-DOC-01-P2-04.
- Any commit that removes `echo-agent-eval` from AGENTS.md invalidates
  Q-DOC-01-P3-03.
- Any CLI redesign that reintroduces subcommands / `--headless` would
  re-validate (not invalidate) Q-DOC-01-P2-03 — re-check `args.rs`.

Follow-up task IDs (recommended, not implemented in this review):

- A single documentation-refresh pass over `echo-agent/README.md` +
  `README.zh.md` (feature tables, example count, workspace diagram, dead
  doc/example links, "Full (default)" comment) — P2/P3 bundle.
- A single documentation-refresh pass over `echo-agent-cli/README.md` +
  `docs/architecture.md` (remove SQLite/FTS persistence claims, correct
  the "依赖 Features" table, add LICENSE or drop the link) — P2.
- A rewrite of `echo-agent-cli/docs/getting-started.md` against the real
  CLI surface — P2.
- An AGENTS.md one-line edit removing `echo-agent-eval` (P3; pending
  since B-DOC-01).
