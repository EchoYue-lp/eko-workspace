# F-API-01: Public facade and documentation contract

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (not inspected; framework-only task)
> Worktree state: clean

## Question

Do root re-exports, crate-level APIs, examples, and docs expose one coherent
framework rather than accidental internals?

## Scope

- All 8 `lib.rs` files (root 331 lines + 7 sub-crates).
- All root facade modules: `retry.rs`, `error.rs`, `llm.rs`, `tokenizer.rs`,
  `scheduler.rs`, `human_loop.rs`, `tasks.rs`, `mcp.rs`, `lsp.rs`,
  `compression.rs`, `audit.rs`, `sandbox.rs`, `skills/`, `workflow/`,
  `guard/`, `plugin.rs`, `memory.rs`, `tools/mod.rs`, `agent/mod.rs`;
  `prelude` and `advanced` (src/lib.rs:137-331).
- `echo-agent/Cargo.toml` (features, docs.rs metadata, `[[example]]` blocks),
  `echo-tools/Cargo.toml`, `echo-execution/Cargo.toml`,
  `echo-core/Cargo.toml` (feature wiring).
- README and API docs: `echo-agent/README.md` (1240 lines, sampled
  sections), `echo-core/README.md` (full), sub-crate `lib.rs` doc headers
  (all 7, full), `echo-macros` doc examples.
- Compile checks of the advertised minimal posture (`--no-default-features`).
- Historical docs sampling: `echo-agent/AUDIT_REPORT.md`, root
  `docs/MASTER-PLAN.md` (API-name claims only).

## Out Of Scope

- Module/type placement inside the root engine (`B-ARCH-01`, `F-*` subsystem
  tasks); behavioral state-machine authority (`F-TSK-01`/`F-TSK-03`).
- Real compilation of all doctests/examples (`Q-FW-02`).
- Content drift of the 45 CLI docs and framework `docs/` tree
  (`Q-DOC-01`, `B-DOC-01` index).
- Marker-feature reconciliation (`F-FEAT-01` owns the fix).
- `echo-agent-cli` side of the contract (`A-*`/`X-*` tasks).

## Inputs

- Root `AGENTS.md` (Subagent terminology, framework/application layering,
  UTF-8/panic safety), shared `REPORTING.md`, `TASKS.md` (F-API-01 card),
  `zcode-ds/README.md`.
- Dependency task reports: zcode-ds `B-ARCH-01` (facade mapping,
  `tasks` marker P3-02, `workspace` facade P2-01), `B-DOC-01` (AUDIT/MASTER
  -PLAN index). Both verified independently, not copied.
- Historical documents treated as hypotheses: `AUDIT_REPORT.md`,
  `docs/MASTER-PLAN.md`.

## Layering Decision

- Generic mechanism: the root facade re-export pattern, the prelude, the
  `workspace` migration module, and feature topology are framework build
  architecture.
- EKO product policy: none (framework-only task).
- Adapter boundary: the root crate is the single consumer-facing adapter
  over the split crates; sub-crate lib.rs docs ("most users should depend on
  `echo_agent`") confirm this contract.
- Duplicate-search terms (both directions, all crates): `Agent`,
  `ReactAgent`, `StructuredAgent`, `AgentBuilder`, `Error`, `Result`,
  `ReactError`, `Store`, `InMemoryStore`, `FileStore`, `SqliteStore`,
  `Task`, `TaskSpec`, `TaskManager`, `TaskNode`, `Tool`, `ToolResult`,
  `ToolManager`, `ToolRunner`, `SandboxExecutor`, `LocalSandbox`,
  `Skill`, `SkillRegistry`, `HookRegistry`, `Guard`, `RuleGuard`,
  `LlmGuard`, `WorkflowDefinition`, `GraphBuilder`, `PlanSpec`,
  `LlmMessage`, `Message`, `register_all_tools`, `register_task_tools`,
  `prelude`, `TypedTool`, `SubAgent`, `TokenUsage`. Result: single
  authorities everywhere; no parallel implementations (V02).

## Current Path

Verified facade topology (V01): root `src/lib.rs` declares 31
always-compiled modules, 9 feature-gated modules, 8 proc-macro re-exports
(lib.rs:115-117), a 5-of-7-crate `workspace` alias module (lib.rs:124-130),
a ~170-symbol `prelude` and an `advanced` module. Each sub-crate-facing
facade follows one of three shapes: (a) flat `pub use <crate>::*` plus an
inner alias module (`core`/`state`/`orchestration`/`integration`/`execution`)
— retry/error/tokenizer/tasks/human_loop/memory/compression/audit/mcp/llm/
guard/workflow/tools; (b) flat re-export only — scheduler/sandbox/lsp;
(c) real implementation owned by the root — `plugin.rs` (`PluginIntegrator`,
~450 lines), `tasks.rs::register_task_tools`, `tools/mod.rs` consts/helpers,
`workflow/{dsl,loader}`, `agent/`, `state`, `trace`, `evolution`, etc.
Compile evidence: `cargo check -p echo_agent --no-default-features --locked`
and `cargo check --workspace --lib --no-default-features --locked` both exit
0; `cargo tree -e features` shows `tools::shell`/`tools::files` resolve only
through `echo_execution`'s `default = ["files", "shell"]`.

## Findings

### F-API-01-P2-01: README (root and echo-core) documents features and APIs that do not exist in the current manifest or code

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/README.md:125-147` (feature table:
  `plan-execute`, `self-reflection` listed, both "In `full`? yes";
  `sandbox` "yes" in `full`); `echo-agent/Cargo.toml:65-103` ([features] has
  no `plan-execute`/`self-reflection`; `full` at :67 has no `sandbox`;
  real features `lsp`/`statistics`/`eval`/`improve`/`testing`/
  `semantic-memory`/`macros`/`provider-factory`/`workflow`/`multimodal`/
  `default` not in the table); `README.md:711-732` (§12 Task Planning shows
  `agent.execute_with_planning(...)` and "Requires the `tasks` feature");
  `echo-core/README.md:18-31` (quickstart `use echo_core::prelude::*;`
  — echo_core has no prelude module; Contents list names `ReActAgent`,
  `SubAgent`, `TypedTool`, `TokenUsage`, none of which exist in echo-core);
  `README.md:446` ("the `TypedTool` implementation automatically").
- Reachability: `README.md` is the crate-level API doc — root lib.rs:23
  (`#![doc = include_str!("../README.md")]`) embeds it into the docs.rs
  surface; echo-core/README.md is its published readme. A consumer
  following either document gets a compile error (`cargo add echo-agent
  --features plan-execute` fails with "the package does not have feature
  `plan-execute`"; `use echo_core::prelude::*` fails to resolve; copying
  §12 fails on `execute_with_planning`).
- Expected invariant: the published documentation (which is the framework's
  primary API contract) must only reference features and symbols that exist
  in the crate it documents.
- Observed behavior: README documents the pre-M13/pre-workspace-split API
  surface; the manifest/code have moved on (`plan_create/plan_patch/
  plan_execute` were deleted in M13 — `docs/MASTER-PLAN.md:387` documents
  the deletion).
- Impact: users of the framework cannot follow the primary documentation;
  three phantom feature names and one phantom method break builds; the
  `sandbox ∈ full` claim misleads feature selection.
- Root cause: M13 (task-API unification) and the workspace split removed or
  renamed features/APIs but the README feature table, §12, and the
  echo-core README were never updated.
- Direction: rewrite `README.md:121-147` from `[features]`; delete §12's
  plan-execute snippet or replace with the current task tools
  (`TaskCreateTool`/`TaskListTool`/`TaskUpdateTool`); rewrite
  `echo-core/README.md` quickstart (drop `prelude`, `ReActAgent`,
  `SubAgent`, `TypedTool`, `TokenUsage`); fix `README.md:446`. No code to
  delete — only doc text.
- Regression validation: `cargo doc` build plus a follow-along compile of
  every README snippet after the rewrite (Q-DOC-01); grep for
  `plan-execute|self-reflection|execute_with_planning|TypedTool|
  echo_core::prelude` returning zero hits in READMEs.
- Validation reports: [V04](../validations/F-API-01/V04-01.md),
  [V05](../validations/F-API-01/V05-01.md)

### F-API-01-P2-02: Root `tools::shell`/`tools::files` are unconditional re-exports of feature-gated echo_tools modules; root `shell`/`files` features cannot disable them

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/tools/mod.rs:58-60` (`pub mod files` →
  `echo_tools::files::*`) and `:78-80` (`pub mod shell` →
  `echo_tools::shell::*`) — no `#[cfg(feature = ...)]` on either;
  `echo-tools/src/lib.rs:25-27` (`#[cfg(feature = "files")] pub mod files`)
  and `:30-32` (`#[cfg(feature = "shell")] pub mod shell`);
  `echo-agent/Cargo.toml:92-93` (`shell = ["echo_tools/shell"]`,
  `files = ["echo_tools/files"]`); `echo-execution/Cargo.toml:22`
  (`default = ["files", "shell"]`).
- Reachability: `pub mod tools` is unconditional (src/lib.rs:62). Both
  modules always compile into every consumer build. Verified: `cargo check
  -p echo_agent --no-default-features --locked` exits 0 and `cargo tree
  --workspace --no-default-features -e features -i echo_tools` shows
  `echo_execution feature "shell"/"files"` activating `echo_tools/shell`
  and `echo_tools/files` via feature unification (V01).
- Expected invariant: a feature named `shell`/`files` on the root crate
  gates the corresponding public modules, and `--no-default-features`
  removes them; the facade's module availability must not depend on an
  unrelated crate's default features.
- Observed behavior: the root modules are always present; the root
  `shell`/`files` features toggle nothing (they only forward to echo_tools,
  whose features echo_execution's defaults re-enable anyway); a consumer
  that builds `echo_execution` with `default-features = false` (or any
  future trim of echo_execution's defaults) breaks the whole `echo_agent`
  facade compile.
- Impact: (a) the advertised "zero default features ... minimal compile
  time" posture (`README.md:104`) is false for these subsystems — shell and
  file tools always compile; (b) latent breakage — the facade's correctness
  is hostage to echo_execution's default features, an implicit cross-crate
  coupling invisible in the root manifest; (c) consumers cannot slim the
  build through the documented features.
- Root cause: during the workspace split the root facade copied
  `pub mod shell`/`pub mod files` as unconditional modules while the
  features that should gate them were left as echo_tools-forwarding
  no-ops; the compile graph hides the defect behind echo_execution's
  defaults.
- Direction: gate `tools::shell`/`tools::files` with
  `#[cfg(feature = "shell")]`/`#[cfg(feature = "files")]` in
  `src/tools/mod.rs` (keeping the echo_tools forwarding), or — if
  always-on is intended — delete the `shell`/`files` features and document
  the modules as unconditional. Align with F-FEAT-01's marker-feature
  audit. No other code needs deletion.
- Regression validation: `cargo check -p echo_agent --no-default-features
  --locked` must still exit 0 (after the gate, echo_tools shell/files must
  be enabled by `-p echo_agent --features shell` instead); `cargo tree -e
  features` must show shell/files originating from `echo_agent`, not
  `echo_execution`; run `cargo check --workspace --lib
  --no-default-features --locked`.
- Validation reports: [V01](../validations/F-API-01/V01-01.md)

### F-API-01-P3-01: echo-macros docs claim macros are importable via `echo_agent::prelude::*` and demo `ToolRunner`, which the facade does not export

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-macros/src/lib.rs:29-31` ("Most users should import these
  macros via `echo_agent::prelude::*` or `use echo_agent::{tool, callback,
  guard, handler};`"); prelude `src/lib.rs:137-276` exports no proc macros
  (V04); `echo-macros/src/lib.rs:85-90` (derive example `impl
  ToolRunner<ReadFileToolParams>`); `echo-core/src/tools/mod.rs:733`
  (`pub trait ToolRunner`) is not re-exported by root
  `tools/mod.rs:109-113`.
- Reachability: doc text only; the derive example is `rust,ignore` so
  nothing fails to compile today. A user copying the example or following
  the prelude guidance gets "cannot find value `tool`" / "cannot find trait
  `ToolRunner`" errors.
- Expected invariant: crate-level macro docs must state the import path
  that actually works and only reference symbols exported by the facade.
- Observed behavior: the working import is `use echo_agent::{tool, ...}`
  (crate root, lib.rs:115-117); the documented prelude path fails;
  `ToolRunner` is unreachable through `echo_agent`.
- Impact: misleading guidance in the framework's macro documentation;
  low severity because examples are ignore-gated.
- Root cause: prelude was curated after the macros were re-exported at the
  crate root; the docs were not updated, and `ToolRunner` was never added
  to the facade's export list.
- Direction: fix `echo-macros/src/lib.rs:29-31` to point at the crate-root
  imports; either export `ToolRunner` from `echo_agent::tools` (if the
  derive workflow is intended to be the primary one) or rewrite the derive
  example without it.
- Regression validation: `cargo doc` + compiling the corrected example
  text (Q-DOC-01/Q-FW-02).
- Validation reports: [V03](../validations/F-API-01/V03-01.md),
  [V04](../validations/F-API-01/V04-01.md)

### F-API-01-P3-02: 13 example files are absent from the manifest's `[[example]]` list; README example count is stale

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence: `echo-agent/Cargo.toml:163-372` (`[[example]]` blocks) vs
  `examples/` (68 files): undeclared are demo00_quickstart, demo01_tools,
  demo02_tasks, demo07_skills, demo09_file_shell, demo10_streaming,
  demo11_callbacks, demo13_tool_execution, demo17_chat, demo19_guard,
  demo32_token_budget, demo33_retry_policy, demo40_snapshot (demo42 is
  declared under `demo42_browser_mcp` with `path`); `README.md:389`
  ("66 runnable examples").
- Reachability: undeclared examples are still auto-discovered by Cargo and
  built by `cargo test --workspace --all-targets`; all 13 currently use
  only unconditional facade paths (demo19 self-gates `content-guard`), so
  default-feature builds pass today (V03).
- Expected invariant: every shipped example carries a manifest declaration
  with the `required-features` its imports need; doc counts match the
  directory.
- Observed behavior: 13 examples have no declaration and therefore no
  `required-features` metadata; README count (66) differs from the actual
  file count (68).
- Impact: a future edit adding a gated API to an undeclared example
  silently breaks default-feature example builds; doc count drift.
- Root cause: examples predate the feature split; declarations were added
  per-feature without a sweep of the whole directory.
- Direction: add `[[example]]` blocks (with correct `required-features`)
  for the 13 files or delete the stale ones; fix the README count.
- Regression validation: `cargo build --examples` under default features
  and `--all-features` stays green (Q-FW-02).
- Validation reports: [V03](../validations/F-API-01/V03-01.md)

### F-API-01-P3-03: `workspace` facade exposes only 5 of 7 split crates and the inner-alias convention is inconsistent

- Priority: P3
- Confidence: low
- Layer: framework
- Evidence: `src/lib.rs:119-130` (`pub mod workspace` re-exports
  `echo_core`, `echo_execution`, `echo_integration`, `echo_orchestration`,
  `echo_state` — `echo_tools` and `echo_macros` absent; doc comment claims
  "Direct access to split workspace crates during migration");
  alias-module naming: `pub mod core` (retry/error/tokenizer/llm/guard),
  `pub mod state` (memory/compression/audit), `pub mod orchestration`
  (tasks/human_loop/workflow), `pub mod integration` (mcp), `pub mod
  execution` (tools) vs no alias in `scheduler.rs:5`, `sandbox.rs:27`,
  `lsp.rs:7-10`.
- Reachability: `workspace::*` is referenced only in doc comments today
  (`src/tasks.rs:6`, `src/human_loop.rs:6`), but it is public API forever
  once published (B-ARCH-01-P2-01 flags the same surface).
- Expected invariant: the migration facade covers all split crates or
  documents the omission; alias naming is uniform so consumers can predict
  the path shape.
- Observed behavior: a migrating consumer cannot reach `echo_tools` or
  `echo_macros` through `workspace` and must guess whether a facade has an
  inner alias module.
- Impact: small today (docs-only usage); increases facade-surface drift
  risk and makes the migration story incomplete.
- Root cause: the `workspace` module was written when 5 crates were split;
  `echo_tools`/`echo_macros` and the newer facades were added later
  without extending the convention.
- Direction: either add `echo_tools`/`echo_macros` to `workspace` (and
  alias modules to scheduler/sandbox/lsp), or delete `workspace` in favor
  of direct sub-crate deps once migration completes — decision belongs to
  X-BND-01/B-ARCH-01-P2-01.
- Regression validation: `cargo doc` for the facade paths after any change;
  compile a consumer using the new paths.
- Validation reports: [V01](../validations/F-API-01/V01-01.md)

### F-API-01-P3-04: `src/error.rs` module doc names the error enum `Error`; the public type is `ReactError` and no `Error` alias exists

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `src/error.rs:4-6` ("All errors are collected under the `Error`
  enum with variants for LLM failures, tool errors, MCP issues, and more");
  `echo-core/src/error.rs:30` (`pub enum ReactError`), `:415` (`pub type
  Result<T> = std::result::Result<T, ReactError>`); grep for
  `pub enum Error` / `as Error` across all crates: zero hits.
- Reachability: doc text only; `use echo_agent::error::Error` fails to
  resolve, `echo_agent::error::ReactError` works.
- Expected invariant: module docs reference the actual public type name.
- Observed behavior: docs advertise a non-existent name (`Error`), so
  users pattern-matching on `error::Error` get compile errors and may
  assume the framework is broken.
- Impact: localized documentation defect; naming confusion in the most
  widely used facade module.
- Root cause: the type was renamed `ReactError` during error unification
  and the facade doc string was not updated.
- Direction: rename doc text to `ReactError` (or add
  `pub use ReactError as Error` if the alias is desired — prefer doc fix
  to avoid a second public name for the same type).
- Regression validation: `cargo doc` grep for the corrected name;
  doc-testing `use echo_agent::error::ReactError`.
- Validation reports: [V02](../validations/F-API-01/V02-01.md),
  [V04](../validations/F-API-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Public re-export map + minimal-posture compile checks | yes | passed | [V01](../validations/F-API-01/V01-01.md) |
| V02 | Duplicate public concept search | yes | passed | [V02](../validations/F-API-01/V02-01.md) |
| V03 | Doctest and example sampling (static) | yes | passed | [V03](../validations/F-API-01/V03-01.md) |
| V04 | Feature/documentation consistency | yes | passed | [V04](../validations/F-API-01/V04-01.md) |
| V05 | Historical-document drift sampling | yes | passed | [V05](../validations/F-API-01/V05-01.md) |

All validations executed; every required validation has its own report. The
task card's V03 allows static-only checking (Q-FW-02 owns real
compilation), which is what was performed.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| README feature table `plan-execute`/`self-reflection` (in `full`) | stale | `README.md:125-147` vs `Cargo.toml:65-103`; features never in current manifest; [V04](../validations/F-API-01/V04-01.md) |
| README §12 `execute_with_planning` plan-execute flow | stale | `README.md:711-732`; 0 code matches; M13 deletion in `docs/MASTER-PLAN.md:387`; [V04](../validations/F-API-01/V04-01.md), [V05](../validations/F-API-01/V05-01.md) |
| README `sandbox` in `full` | stale | `Cargo.toml:67` lacks `sandbox`; [V04](../validations/F-API-01/V04-01.md) |
| echo-core README prelude/`ReActAgent`/`SubAgent`/`TypedTool`/`TokenUsage` | stale | no such module/types; [V04](../validations/F-API-01/V04-01.md) |
| MASTER-PLAN M13 `task_create/task_update/task_list/task_execute` unification; `plan_create/plan_patch/plan_execute` deleted | current | `TaskCreateTool`/`TaskUpdateTool`/`TaskListTool` in `echo-orchestration/src/tasks/task_tools.rs:15/82/143`; no plan_* symbols; [V05](../validations/F-API-01/V05-01.md) |
| MASTER-PLAN `PlanSpec`, `HookRegistry`, `SkillRegistry`, `ToolManager`, `TaskManager`, `FileStore` | current | all exist and are facade-reachable; [V05](../validations/F-API-01/V05-01.md) |
| AUDIT_REPORT.md findings/anchors | current/fixed/stale per B-DOC-01 | no API-name claims contradicted; [V05](../validations/F-API-01/V05-01.md) |
| README "zero default features" | current (manifest-level) | `Cargo.toml:66`; effective surface caveat in F-API-01-P2-02; [V01](../validations/F-API-01/V01-01.md) |

## Coverage And Uncertainty

- Root engine internals (`src/agent/` 65 files, `evolution`, `trace`,
  `context`, `notebook`, `intent`, `state` internals, `eval`, `improve`,
  `testing`) were classified as root-owned modules only; their internal
  public types were not individually audited (F-* subsystem tasks own
  that).
- `README.md` was sampled (feature table, §2, §12, §13, quickstart,
  highlights), not read end-to-end; further stale snippets are possible
  (Q-DOC-01).
- The "67 registered tools" claim (`README.md:181,389`) was not verified —
  tool counting requires tracing `register_all_tools` registrations
  (Q-FW-02).
- Doctests/examples: static import-path verification only; real
  compilation is Q-FW-02.
- `echo-agent/docs/` (en/zh/knowledge) not read — Q-DOC-01 scope.
- The `tasks` marker no-op (B-ARCH-01-P3-02) was re-confirmed but not
  re-filed; root `shell`/`files` no-op semantics are filed here as part of
  F-API-01-P2-02 with F-FEAT-01 owning the fix.
- `subagent` and `eval`/`improve`/`project-rules` gates are real
  (`cfg(feature=...)` counts: subagent gates `src/agent/subagent/`,
  eval 15, improve 2, project-rules 3); `sandbox`/`semantic-memory`/
  `macros`/`provider-factory`/`workflow`/`multimodal` have 0 gates.
- No runtime behavior was exercised; all conclusions are static plus two
  compile checks.

## Handoff

- Downstream tasks may rely on: the complete facade authority map (V01);
  single-authority confirmation for Agent/Error/Store/Task/Tool/Sandbox/
  Skill/Guard/Workflow (V02); static-clean doctest/example imports (V03);
  the README/manifest mismatch inventory (V04); the current/stale
  classification of MASTER-PLAN API claims (V05).
- Reports to read: this report + the 5 validation reports; B-ARCH-01
  (facade/marker-feature context), B-DOC-01 (full historical index).
- Stale triggers: any change to `src/lib.rs`, `Cargo.toml` (features or
  `[[example]]`), README files, echo-macros docs, or the echo_execution
  default features invalidates the corresponding claims.
- Follow-up tasks (fixes are not implemented in this review):
  - `F-FEAT-01`: reconcile docs.rs metadata with `[features]`; decide the
    fate of the 6 no-op markers and of root `shell`/`files` (P2-02);
  - `Q-DOC-01`: README/echo-core-README/echo-macros doc rewrites
    (P2-01, P3-01, P3-04);
  - `Q-FW-02`: real compilation of doctests and examples, verify the
    static V03 results;
  - `X-BND-01`: final decision on the `workspace` facade shape (P3-03,
    with B-ARCH-01-P2-01).
