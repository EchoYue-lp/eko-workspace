# F-FEAT-01: Feature topology and isolation

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Does each feature enable exactly its required code and dependencies, including no-default and standalone feature use?

## Scope

- All 8 framework manifests (`echo-agent/Cargo.toml` + 7 sub-crate `Cargo.toml`): `[features]`, optional `dep:` entries, `default`, `full`, `[package.metadata.docs.rs]`, example `required-features`.
- Every `#[cfg(feature = "...")]` / `#[cfg_attr(docsrs, doc(cfg(...)))]` declaration in `echo-agent/src/**`, `echo-core/src/**`, `echo-tools/src/**`, `echo-integration/src/**`, `echo-state/src/**`, `echo-orchestration/src/**`, `echo-execution/src/**`, `echo-macros/src/**` (plus tests/examples/benches), and every optional-crate usage site.
- The 13 marker features, one by one: code gates, dep edges, module gating, example gates.
- Feature topology documentation: `echo-agent/README.md:113-147`, root `AGENTS.md` feature matrix, docs.rs metadata.

## Out Of Scope

- Real `cargo check`/compile execution — owned by `Q-FW-02` (V04 is static-only by task definition; rows handed over as needs_evidence).
- CLI/application feature sets (`F-APP-*` tasks), dependency advisories (`Q-DEP-01`), CI enforcement (`B-BASE-01`/`Q-FW-01`).
- `docs/comprehensive-review/codex/` and `zcode-ds` sibling conclusions — not read (B-ARCH-01 facts were re-verified independently here, not copied).

## Inputs

- Root `AGENTS.md` (Subagent terminology, framework/application layering, UTF-8/panic rules — no violations found in the reviewed surface), `docs/comprehensive-review/REPORTING.md`, `zcode-ds/README.md`, report templates.
- Dependency report: `B-BASE-01` (task + V01/V02/V03/V04) — manifest facts and feature graph confirmed and re-derived independently.
- B-ARCH-01 findings (tasks marker no-op, lsp double-forward, full-marker omissions, docs.rs omissions) were **re-verified independently**; see Historical Claim Status.

## Layering Decision

- Generic mechanism: the eight-package feature topology, cfg gating, and no-default semantics are framework build facts; all findings below belong to the framework layer (echo_tools/echo_execution/echo_agent manifests and facade code).
- EKO product policy: not implicated — the CLI's own feature selections (`echo-agent-cli` enables shell/files/etc.) mask the always-on issue for EKO, but framework consumers in general are affected.
- Adapter boundary: root facade → echo_execution dependency edge (missing `default-features = false`) is the adapter point where the default-feature leak occurs.
- Duplicate search: greps for every feature name, optional crate name, `cfg(feature`, `cfg_attr(feature`, `cfg!(feature`, `CARGO_FEATURE_`, and `echo_tools::` references across both repositories' framework side; no second feature system or parallel gating mechanism found.

## Current Path

Declared topology: root `echo_agent` exposes 33 features + `default`/`full` (`echo-agent/Cargo.toml:65-103`); 7 of 33 are forwarding edges to owner sub-crates, 13 are empty markers, 13 are code-gated with/without deps (V02 table). Sub-crates gate their own code correctly (echo_tools 11/11 features gated per module, echo_state sqlite gated, echo_orchestration websocket gated, echo_core reqwest/guard/permission/mcp/lsp/channels gated, echo_integration mcp/lsp/channels gated). Optional-dependency usage is fully confined to gating features (V03).

Two topological breaks were found:

1. **always-on `files`/`shell`**: `echo-execution/Cargo.toml:22` `default = ["files", "shell"]`; root edge `echo-agent/Cargo.toml:108` does not disable defaults; root facade re-exports `echo_tools::{files,shell}` unconditionally (`echo-agent/src/tools/mod.rs:58-59,78-79`). Result: every `echo_agent` consumer — including `--no-default-features` builds (Cargo feature-union semantics keep echo_execution defaults through the dependency edge, V04) — compiles echo_tools/files (7 tree-sitter grammar crates, `echo-tools/Cargo.toml:67-73`) and echo_tools/shell. Root `files`/`shell` features gate nothing consumer-visible.
2. **six zero-effect markers**: sandbox, semantic-memory, macros, provider-factory, workflow, multimodal have zero `cfg` references; their modules are always compiled (`src/lib.rs:54,65,111`) or do not exist; their only effect is example `required-features` gating.

Reachability of the always-on claim: any build selecting `echo_agent` (default or no-default) → dependency resolution enables echo_execution default → echo_tools files+shell → tree-sitter built; root `src/tools/mod.rs:58-59` requires `echo_tools::files` to exist, which is guaranteed only by that chain, making the features non-optional in practice.

## Findings

### F-FEAT-01-P2-01: `files`/`shell` are effectively always-on — no-default builds cannot disable them

- Priority: P2
- Confidence: high (static evidence; real-compile confirmation delegated to Q-FW-02)
- Layer: framework
- Evidence: `echo-execution/Cargo.toml:22` (`default = ["files", "shell"]`); `echo-agent/Cargo.toml:108` (echo_execution edge without `default-features = false`); `echo-agent/src/tools/mod.rs:58-59` (ungated `pub mod files { pub use echo_tools::files::*; }`), `:78-79` (ungated shell re-export); `echo-tools/Cargo.toml:20-29,67-73` (files feature + 7 tree-sitter grammar deps); `README.md:104` ("zero default features ... minimal compile time and dependency footprint")
- Reachability: every build that selects `echo_agent` (default features, `--no-default-features`, any feature subset) resolves echo_execution's defaults through the root dependency edge (Cargo feature union), enabling echo_tools files+shell → tree-sitter grammars compiled; the ungated facade re-exports additionally make `echo_tools::{files,shell}` mandatory for the root crate to compile. Root `files`/`shell` features (`echo-agent/Cargo.toml:92-93`) add nothing a consumer can turn off.
- Expected invariant: with `default = []`, no optional-feature code or dependency compiles unless the consumer opts in via `--features files`/`shell`; the workspace `--no-default-features` gate (AGENTS.md) exercises a genuine no-optional-features build.
- Observed behavior: files/shell code and tree-sitter grammars compile unconditionally for all consumers; `--no-default-features` leaves them enabled (vacuously passing gate); the root `files`/`shell` features are decoration.
- Impact: every `echo_agent` consumer pays the tree-sitter build cost (8 C-composing crates) and gets shell/file tooling they never asked for; the README "minimal footprint" promise is false; the declared no-default gate gives false confidence; the feature topology misleads API consumers about what "exactly the required code and dependencies" means.
- Root cause: echo_execution's `default = ["files", "shell"]` combined with a facade dependency edge that does not disable defaults, plus ungated re-export modules in the facade that require those features to be on.
- Direction: (a) set `default = []` in `echo-execution/Cargo.toml` (its own modules gate correctly via `cfg(feature = "files"/"shell")` — `echo-execution/src/skills/builtin/mod.rs:1-3`, `prompt_exec.rs:586`); (b) add `#[cfg(feature = "files")]` and `#[cfg(feature = "shell")]` to the facade re-export modules at `echo-agent/src/tools/mod.rs:58-59,78-79` so the root crate compiles under true no-default; (c) update `README.md:104` claim; delete nothing else — echo_tools gating itself is correct.
- Regression validation: `cargo check --workspace --lib --no-default-features --locked` must pass with echo_execution defaults off; `cargo tree -e features -i tree-sitter` must show tree-sitter only under `files`; a consumer with only `mcp` enabled must not compile tree-sitter. (Real runs → Q-FW-02; static verdict in V04.)
- Validation reports: [V04](../validations/F-FEAT-01/V04-01.md), [V05](../validations/F-FEAT-01/V05-01.md), [V03](../validations/F-FEAT-01/V03-01.md)

### F-FEAT-01-P2-02: Six marker features have zero code effect and mislead consumers

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: zero `cfg(feature = ...)` references repo-wide for sandbox, semantic-memory, macros, provider-factory, workflow, multimodal (V01 census); modules always compiled — `echo-agent/src/lib.rs:54` (`pub mod sandbox;`), `:65` (`pub mod workflow;`), `:111` (`mod macros;`), `:96` (`pub mod tasks;`); no module exists for semantic-memory/multimodal/provider-factory; features declared at `echo-agent/Cargo.toml:98-103`; example gates only (`:181,:192,:204,:206,:213,:216,:227,:232,:233,:237,:301`)
- Reachability: enabling any of these six features changes no compiled code and enables no dependency; the only observable effect is that demo examples with matching `required-features` become buildable.
- Expected invariant: a declared public feature must enable exactly the code/dependencies it names; `docs.rs` and consumers can rely on feature meaning.
- Observed behavior: the six features are empty no-ops; the capabilities they name (sandbox, workflow, macros, semantic memory, provider factory, multimodal) are either always compiled or absent regardless of the flag.
- Impact: consumers (including EKO and third parties) enabling `sandbox`/`workflow`/`multimodal` believe they are opting into gated capability when nothing changes; the 13-marker set conflates 7 functional gates with 6 pure no-ops; feature-based dependency selection (e.g., future `#[cfg]`-dependent builds, docs.rs metadata) cannot distinguish intent. Note `sandbox` in particular: `README.md:137,264` and the demo examples imply opt-in sandboxing, but `echo_agent::sandbox` is always available.
- Root cause: roadmap features declared as placeholders before implementation; the corresponding modules were compiled unconditionally instead of being gated behind their features.
- Direction: either (a) gate the existing modules — `#[cfg(feature = "sandbox")] pub mod sandbox;` (`src/lib.rs:54`), `#[cfg(feature = "workflow")] pub mod workflow;` (`:65`), `#[cfg(feature = "macros")] mod macros;` (`:111`) — and add real modules for semantic-memory/multimodal/provider-factory, or (b) delete the six features plus their example `required-features` entries and README rows. If kept as roadmap markers, document them as such in the manifest and README; do not leave them silently no-op.
- Regression validation: after gating, `cargo check -p echo_agent --no-default-features` must pass and `--features sandbox` must add `echo_agent::sandbox`; after deletion, grep for the feature names must return zero hits and examples must be removed/re-keyed.
- Validation reports: [V01](../validations/F-FEAT-01/V01-01.md), [V02](../validations/F-FEAT-01/V02-01.md)

### F-FEAT-01-P3-01: README feature table documents two nonexistent features and mislabels `full` membership

- Priority: P3
- Confidence: high
- Layer: framework (documentation)
- Evidence: `echo-agent/README.md:121-122` (`plan-execute`, `self-reflection`, both "yes" in `full`), `:137` (sandbox "yes"), `:132` (research "no"), `:139-140` (content-guard/project-rules "no"), `:257-258`, `:1170,1173` (docs index); manifest `echo-agent/Cargo.toml:65-103` has no `plan-execute`/`self-reflection` and `full` (:67) excludes sandbox but includes research/content-guard/project-rules/shell/files
- Reachability: any user copying `features = ["plan-execute"]` or `["self-reflection"]` from the README gets a Cargo "unknown feature" build error; users choosing `full` for sandbox get nothing (the sandbox module is always compiled anyway).
- Expected invariant: the published feature table matches the manifest exactly, since it is the contract users copy.
- Observed behavior: two phantom features; four wrong membership rows; 10 declared features (lsp, statistics, eval, improve, testing, semantic-memory, macros, provider-factory, workflow, multimodal) missing from the table.
- Impact: user-facing build errors and wrong capability expectations; the table is the primary public surface for feature selection.
- Root cause: table written for an earlier feature set; not synced after renames/removals (plan-execute/self-reflection predate the tasks/eval split) and after the marker set changed.
- Direction: regenerate the table from the manifest; drop the two phantom rows (or map to real equivalents); correct sandbox/research/content-guard/project-rules membership; add the missing rows.
- Regression validation: for every row, `grep` the feature name in `echo-agent/Cargo.toml` `[features]` and verify the "In full?" column equals membership in `full` (:67); diff table rows against declared features.
- Validation reports: [V05](../validations/F-FEAT-01/V05-01.md)

### F-FEAT-01-P3-02: `subagent = ["tasks"]` coupling is unnecessary and its manifest rationale is stale

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/Cargo.toml:77-80` (comment: "Without `tasks` the `subagent` feature fails to compile on its own"); `src/agent/subagent/context_builder.rs:7` (`use crate::tasks::Evidence`), `executor.rs:30` (`use crate::tasks::NestedDelegationPolicy`); `src/lib.rs:96` (`pub mod tasks;` — ungated facade re-export of `echo_orchestration::tasks`); tasks-gated symbols (`TaskSpawner`, `SpawnBackgroundTaskTool`, `CheckTaskStatusTool`, `ListBackgroundTasksTool`) appear only under `#[cfg(feature = "tasks")]` in `src/agent/react/mod.rs:31-45` and are never referenced by subagent-gated code (V01 cross-check)
- Reachability: `--features subagent` alone resolves because `crate::tasks` is always compiled; the declared `subagent = ["tasks"]` edge additionally enables the tasks feature's background-task tools (`src/tools/builtin/spawn_task.rs`, `check_task.rs`, `src/agent/callbacks/progress_bridge.rs`) in every subagent build.
- Expected invariant: a feature edge exists only where the target feature's gated code is actually required; manifest comments describe the real reason.
- Observed behavior: the edge is not required for compilation (stale rationale), and `tasks` — contrary to the B-ARCH-01 "no-op marker" label — gates real code (V01/V02).
- Impact: minor — subagent consumers silently pull background-task tools; the manifest comment misleads future maintainers into believing the coupling is load-bearing.
- Root cause: `pub mod tasks` was ungated during the workspace-split migration, making the edge's original justification obsolete without updating the comment.
- Direction: update the comment (or drop the edge if the runtime coupling is unwanted; `full` and the CLI already enable `tasks` separately). Do not treat `tasks` itself as deletable — it gates working background-task tooling.
- Regression validation: `cargo check -p echo_agent --no-default-features --features subagent` (needs Q-FW-02 run) before and after the change; `cargo tree -e features -i echo_orchestration` shows tasks only under the desired edges.
- Validation reports: [V01](../validations/F-FEAT-01/V01-01.md), [V05](../validations/F-FEAT-01/V05-01.md)

### F-FEAT-01-P3-03: Root feature forwardings carry optional deps unused by root code

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence: `echo-agent/Cargo.toml:81-88` (`web = ["dep:scraper", "dep:html2text", "dep:url", ...]`, `media = ["dep:pdf-extract", ..., "dep:encoding_rs", ...]`, `data = ["dep:polars", ...]`, `statistics = ["dep:polars", ...]`, `database = ["dep:sqlx", ...]`); zero usage of these crates in root code (V03 grep: no `scraper`, `html2text`, `url`, `pdf_extract`, `lopdf`, `calamine`, `docx_rs`, `encoding_rs`, `polars`, `sqlx` in `echo-agent/src`); root only re-exports `echo_tools::{web,media,research,...}` (`src/tools/mod.rs:83-98`)
- Reachability: enabling root `web`/`media`/`data`/`statistics`/`database` compiles the deps for the root crate even though only echo_tools' copies are used; feature lists for polars/sqlx are duplicated in two manifests and must stay in sync.
- Expected invariant: a feature's `dep:` entries name dependencies its own code requires.
- Observed behavior: the `dep:` entries are dead weight at root level — compile-neutral today but duplicating dependency metadata (polars/sqlx feature lists exist in both `echo-agent/Cargo.toml:159,161` and `echo-tools/Cargo.toml:74-75`).
- Impact: no compile or runtime break; maintenance/drift risk and misleading topology (a reader concludes root code consumes these crates). Exceptions that MUST stay: `a2a` (axum/jsonwebtoken used in `src/a2a`), `telemetry` (opentelemetry in `src/telemetry.rs`), `sqlite` (rusqlite in `src/state/sqlite.rs`).
- Root cause: forwarding features copied the full dependency lists from echo_tools during the workspace split.
- Direction: trim the unused `dep:` entries from root `web`/`media`/`data`/`statistics`/`database` (keep `echo_tools/<feature>` forwards); verify docs.rs `--no-default-features` metadata still resolves.
- Regression validation: `cargo check -p echo_agent --no-default-features --features web,media,data,statistics,database` (Q-FW-02) after trimming.
- Validation reports: [V02](../validations/F-FEAT-01/V02-01.md), [V03](../validations/F-FEAT-01/V03-01.md)

### F-FEAT-01-P3-04: docs.rs metadata omits ten features without comment

- Priority: P3
- Confidence: medium
- Layer: framework (docs metadata)
- Evidence: `echo-agent/Cargo.toml:55-63` — `[package.metadata.docs.rs]` lists 22 of 33 features; `data` omission documented (:59-63, polars nightly), but shell, files, research, testing, sandbox, semantic-memory, macros, provider-factory, workflow, multimodal omitted with no comment
- Reachability: docs.rs builds `no-default-features = true` + the listed features; gated modules for `research` (in `full`) never get a `doc(cfg)` badge on docs.rs; always-on shell/files omission is moot.
- Expected invariant: docs.rs metadata either matches the feature set or documents each omission.
- Observed behavior: 10 undocumented omissions; the one documented omission (`data`) is accurate with a correct upstream reference.
- Impact: minor — missing feature badges and an incomplete docs.rs feature list for a published crate.
- Root cause: metadata list updated incrementally as features were added; cleanup lagged.
- Direction: either add the missing features to the metadata list or add per-line comments; keep the polars exclusion note.
- Regression validation: `cargo doc --no-default-features --features <each added feature>` succeeds locally (Q-FW-02/GUI lanes).
- Validation reports: [V05](../validations/F-FEAT-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---|---|
| V01 | cfg-to-feature search: all refs, undeclared refs, zero-ref features incl. 13 markers | yes | passed | [V01](../validations/F-FEAT-01/V01-01.md) |
| V02 | Feature classification: code gate + dep / dep-only / empty marker / dead | yes | passed | [V02](../validations/F-FEAT-01/V02-01.md) |
| V03 | Optional dependency leakage check | yes | passed | [V03](../validations/F-FEAT-01/V03-01.md) |
| V04 | Static standalone-compile risk; real compiles → Q-FW-02 | conditional | inconclusive (static; needs_evidence) | [V04](../validations/F-FEAT-01/V04-01.md) |
| V05 | No-default and feature-implied checks (defaults, statistics→data, subagent→tasks, full vs markers, docs.rs metadata, README table) | yes | passed | [V05](../validations/F-FEAT-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| B-BASE-01 V02: "13 marker features" all empty | stale (refined) | 7 of 13 gate real code; 6 are pure no-ops; [V01](../validations/F-FEAT-01/V01-01.md), [V02](../validations/F-FEAT-01/V02-01.md) |
| B-ARCH-01 P3-02: `tasks` marker is a no-op; `subagent = ["tasks"]` is a problem | partially stale | `tasks` gates background-task tools (7 real cfg sites); the `subagent→tasks` edge is unnecessary with a stale comment, but harmless; F-FEAT-01-P3-02 |
| B-ARCH-01: `lsp` double-forward | current (confirmed) | root forwards `echo_core/lsp` + `echo_integration/lsp`, and integration/lsp also forwards `echo_core/lsp` — redundant, consistent, no impact; [V02](../validations/F-FEAT-01/V02-01.md) |
| B-ARCH-01: `full` omits 7 markers (macros/multimodal/provider-factory/sandbox/semantic-memory/testing/workflow) | current (confirmed) | exact list verified at `echo-agent/Cargo.toml:67`; [V05](../validations/F-FEAT-01/V05-01.md) |
| B-ARCH-01: docs.rs omissions incl. documented `data` exclusion | current (confirmed + extended) | `data` documented; 10 more omissions undocumented; F-FEAT-01-P3-04 |
| Root `README.md:104`: "zero default features for minimal compile time and dependency footprint" | regressed in effect | echo_execution defaults + ungated facade re-exports make files/shell always-on; F-FEAT-01-P2-01 |
| AGENTS.md conditional feature matrix (sqlite subagent human-loop mcp lsp a2a git database rag chart web media) | current | all 13 named features exist in root manifest; [V05](../validations/F-FEAT-01/V05-01.md) |

## Coverage And Uncertainty

- No `cargo` command was executed (task definition); all compile verdicts are static and listed in V04 for Q-FW-02 to confirm with exit codes. In particular F-FEAT-01-P2-01's "always-on" claim rests on Cargo feature-union semantics and ungated re-export reads — solid static evidence, but a consumer-style compile (`cargo tree -e features -i tree-sitter`) is the definitive check.
- `echo-agent-cli` feature usage was not audited (application tasks own it); it masks the always-on issue for EKO only because EKO enables shell/files explicitly.
- Rust edition-2024 / resolver v3 behavior of `--workspace --no-default-features` was reasoned, not executed; Q-FW-02 should run the exact AGENTS.md gate command.
- `echo-macros` has no features — nothing to check.
- 68 examples' `required-features` were cross-checked against declared features (all valid) but not compiled.

## Handoff

- Q-FW-02: execute the V04 matrix rows; specifically (1) `cargo check --workspace --lib --no-default-features --locked`, (2) `cargo check -p echo_agent --no-default-features --features subagent`, (3) `cargo tree -e features -i tree-sitter` under a minimal consumer build, (4) `cargo doc` metadata sanity. Report exit codes back to this task's V04 (new attempt report).
- F-APP-*: EKO's explicit shell/files enablement means no EKO-visible impact today; if EKO ever drops them, the always-on finding matters.
- B-DOC-01: README feature-table drift (F-FEAT-01-P3-01) and docs.rs metadata omissions (P3-04) are documentation findings for the docs-fix iteration.
- Deletion targets when fixing: none beyond the six no-op marker features if the "delete" direction is chosen (P2-02); the ungated re-exports are to be gated, not deleted (P2-01).
- This report becomes stale if any of the eight manifests' `[features]`, `echo-execution` defaults, the facade `src/tools/mod.rs` gates, the README table, or the reviewed commits change.
