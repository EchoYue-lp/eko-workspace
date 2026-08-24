# X-BND-01: Capability placement and duplicate authority map

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (9b0e0faf74d35c9a432370b923acabfbb5f32d63)
> `echo-agent-cli` commit: b3b2e81 (b3b2e81f2b2d9fdb319ec604a561beec5f66fea5)
> Worktree state: echo-agent has unrelated dirty paths (`src/agent/react/run/phases/tools.rs`, `src/agent/react/run/stream_channel.rs`, `src/testing/mock_llm.rs`, `src/testing/mock_tool.rs`, `src/testing/mod.rs`); echo-agent-cli clean. No dirty file is in the boundary scope (tasks/subagent/workflow/adapter/state) inspected here; all evidence is read from the committed code at the hashes above.

## Question

Across both repositories, which concepts are correctly framework, EKO
policy, or thin adapters, and where do semantic duplicates remain?

## Scope

Cross-repository boundary audit at the type, behavior, public-option, and
adapter-authority layers. Primary evidence is repo-wide `grep` across all
`.rs` source under `echo-agent/**/src/` (7 sub-crates + root, 419 files)
and `echo-agent-cli/**/src/` (app-core + src-tauri, 198 files), excluding
`target/`. Per-site bodies and call paths were read where classification
required it.

Concept axes inspected:

- **Type/trait/enum name duplicates** (V01): intersect the pub-name sets
  of both repos; classify each shared name.
- **Behavior/call-path duplicates** (V02): `atomic_write` primitive, the
  `TaskSubagent` contract liveness, workflow execution, worktree creation.
- **Public framework options** (V03): the Store/Compressor menu, dead
  framework pub surfaces vs retained options.
- **Adapter authority + deletion targets** (V04): frontier/retry/DAG
  ownership in app-core; closed deletion-target matrix.

## Out Of Scope

Deferred to named task IDs:

- `X-TSK-01` — task-graph round-trip field parity and shared-fixture
  execution. This task establishes the *type/behavior* duplicate map;
  X-TSK-01 owns the *field-level* round-trip (and inherits A-TSK-01-P2-02
  for the `Retrying`/`Paused` lossiness).
- `X-EVT-01` — event-lifecycle conformance across producers/consumers.
  This task re-uses F-CORE-01's dead-bus finding but does not re-audit
  event ordering/terminals.
- `F-MAG-01` — handoff/topology ownership. The `Topology` trait (1 impl)
  was not deeply classified here.
- `F-MEM-02` / `Q-FW-02` — per-feature compile matrix for retained
  framework options. This task is static; feature-isolation compile is
  F-FEAT-01/Q-FW-02.
- Implementation of any deletion/consolidation fix. Findings hand off.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  the workspace instructions — particularly the framework-vs-application
  layering gate, the "delete framework code" criterion, the "first check
  if it already exists" gate, the Subagent-only terminology, and the
  cross-repository boundary gate in REPORTING.md).
- `docs/comprehensive-review/REPORTING.md` (in full — the
  cross-repository boundary gate table).
- `docs/comprehensive-review/templates/{task-report,validation-report}.md`.
- `docs/comprehensive-review/TASKS.md` (the X-BND-01 card + dependency
  declarations).

Dependency task reports read (selected F-*/A-*/X-* showing boundary
findings, per the task brief):

- `zcode-glm/tasks/B-ARCH-01.md` — root crate is not a pure facade;
  `RuntimeStateStore`/`EventBus` live in the root; parallel access paths.
- `zcode-glm/tasks/F-CORE-01.md` — `GLOBAL_EVENT_BUS`/`EventBus` dead;
  `EventEnvelope` is the single transport contract.
- `zcode-glm/tasks/F-API-01.md` — facade parallel access paths; docs.rs
  gaps; the `workspace` module asymmetry.
- `zcode-glm/tasks/A-TSK-01.md` — TaskRuntime file authority is singular;
  `EkoRevisionedTaskStore`/`EkoTaskToolPolicy` are thin adapters; the
  `Retrying`/`Paused` round-trip lossiness (adapter boundary).
- `zcode-glm/tasks/X-INV-01.md` — CLI no-SQLite (clean); no parallel task
  CRUD (clean); panic/UTF-8 invariants.

Historical documents treated as hypotheses: the module docstrings at
`echo-orchestration/src/tasks/runtime.rs:1-5` ("Generic task-runtime
primitives … product-neutral"), `subagent/worktree.rs:30-50` ("the
framework doesn't model git internals"), and the A-TSK-01 docstrings on
adapter thinness — all verified below.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework owns the canonical task/event/memory/workflow contracts: `RevisionedTaskGraph` + `RuntimeDagExecutor` + `RuntimeDagController` (live task path), `EventEnvelope`/`envelope_event_stream` (live event path), the `Store`/`ConversationStore`/Compressor menu, `echo_tools::git_worktree`, and the `Workflow` engine. Any `echo-agent` consumer may need these. V01/V03 confirm. |
| EKO product policy | File-backed persistence (no SQL), EKO projection types (`TaskPlan`/`PlanTask`/`EkoTaskExecution`/`TodoItem`/`Artifact`/`TaskExecutionSummary`), `DomainProfile`/file-ownership waves/reviewer strategy, the CLI's own `JsonlRunStore`, `WorkflowDef`/`WorkflowStep` storage DTOs. All correctly in the application layer. |
| Adapter boundary | `EkoRevisionedTaskStore`/`EkoTaskToolPolicy` (A-TSK-01: thin), `EkoRuntimeDagController` (V04: thin — uses framework hooks for product policy), `TaskExecutionSummary::to_runtime_summary` / `SuggestedTask::to_runtime_suggested_task` (wire adapters), `SubagentVerification*` `From<&echo_agent::…>` projections. V04 confirms none owns scheduling/state authority. |
| Duplicate search | Names: all 20 pub names shared between repos (V01 table). Traits: every framework pub trait's implementor count (V03). Behaviors: `atomic_write`, `append_line`, worktree git invocation, `envelope_event_stream` producers, `to_runtime_summary` callers (V02). |
| Migration deletion | Five closed deletion/consolidation targets, each with a named successor (V04 matrix): `TaskSubagent` surface → `RuntimeDagController`; `GLOBAL_EVENT_BUS`/`EventBus` → direct stream composition; 5 of 6 `atomic_write` → one canonical helper; CLI `WorktreeError` → reuse framework's; (optional) CLI `run_git` worktree → reuse `echo_tools::git_worktree`. |

## Current Path

Verified cross-repository authority map at commits `9b0e0fa` / `b3b2e81`:

1. **Type-name overlap is small and mostly benign.** 20 pub type names are
   defined in both repos (V01). 16 are coincidental reuse (different
   domain/kind — e.g. three unrelated `ExecutionMode` enums; CLI
   `TaskState` is a service container, FW `TaskState` is `ManagedTask`
   execution state) or legitimate wire projections with explicit
   `From`/`to_*` conversion (`SubagentVerification*`,
   `SubagentTouchedFiles`, `TaskExecutionSummary`, `SuggestedTask`,
   `SkillInfo`/`SkillSource`). None is a parallel task/plan/todo/store/
   validator/CRUD authority (extends A-TSK-01 V03-01 to the whole CLI tree).

2. **The live task-execution contract is `RuntimeDagController`, not
   `TaskSubagent`.** The CLI implements
   `echo_agent::tasks::RuntimeDagController` for `EkoRuntimeDagController`
   (`executor.rs:1222`) and constructs the framework
   `RuntimeDagExecutor::new(...)` (`executor.rs:1645`). The framework's
   `TaskSubagent` trait (`runtime.rs:331`) has zero implementors; the
   framework's `TaskExecutionSummary`/`SuggestedTask` (runtime.rs) are
   never produced in CLI production; the CLI's adapter converter
   `to_runtime_summary()` has exactly one caller, a `#[test]`
   (`types.rs:2093`).

3. **`atomic_write` is duplicated 6 times with drifting durability.**
   V02-01 lists all six sites. Only one (`echo-agent/src/state/file.rs:210`)
   fsyncs the parent directory after rename; the other five (including the
   framework's own `echo-state/src/memory/file_conversation.rs:494`) omit
   it. This is the mechanism behind A-STATE-01's recurring parent-dir
   fsync concern.

4. **Framework backend "menu" is retained correctly.** V03 confirms
   `SqliteStore`/`SqliteConversationStore`/`HybridCompressor`/
   `InMemoryStore`/`EmbeddingStore` are legitimate retained framework
   options; the CLI picks File backends and does not enable `sqlite`
   (X-INV-01 V02-01 reaffirmed). Two dead framework surfaces
   (`TaskSubagent`, `GLOBAL_EVENT_BUS`/`EventBus`) are NOT retained
   options — they are fully-replaced and are deletion candidates.

5. **Adapters are thin.** V04 confirms the CLI uses the framework's
   designed product-policy hooks: `RuntimeDagController::select_ready_wave`
   filters the framework-provided frontier for file-ownership safety
   (`executor.rs:1265-1281`; framework doc at `runtime_executor.rs:100-108`
   explicitly authorizes this); `review.rs` retry-count is reviewer
   strategy dispatched through `resolve_dispatch`, not a generic retry
   loop; durable-result reuse is recovery policy inside `dispatch_task`.
   No second frontier/scheduler/DAG-validator/generic-retry authority.

## Findings

### X-BND-01-P2-01: `atomic_write` is duplicated six times across both repos with inconsistent parent-directory fsync

- Priority: P2
- Confidence: high
- Layer: framework + application (the canonical helper belongs in the
  framework; four of the six sites are in the application)
- Evidence (V02-01):
  - `echo-agent/echo-state/src/memory/file_conversation.rs:494` — `fn atomic_write(path, bytes) -> io::Result<()>`; **no** parent-dir fsync.
  - `echo-agent/src/state/file.rs:210` — same shape; **yes** calls `sync_parent_directory(parent)` (the only site that does).
  - `echo-agent-cli/echo-agent-app-core/src/analysis.rs:934` — same shape; no fsync.
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:1988` — same shape (returns `ResearchResult`); no fsync.
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:648` — `write_json_atomic<T: Serialize>`; same body; no fsync.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:405` — same shape (pid/nanos/counter tmp name); no fsync.
- Reachability: all six are live (called by their owning modules' write
  paths). Definition → live caller confirmed for each in V02-01.
- Expected invariant (AGENTS.md "framework vs application" + the
  REPORTING.md adapter rule): a generic, correctness-sensitive filesystem
  primitive (atomic replace) belongs in the framework, used unchanged by
  every consumer. Drift between copies is a duplicate-authority defect.
- Observed behavior: six implementations of the same
  tmp→write→sync_all→rename primitive that differ in tmp naming, error
  type, and — critically — whether the parent directory is fsynced after
  rename. Only 1/6 does the parent-dir fsync.
- Impact:
  - **Durability.** A crash after rename but before the directory entry is
    durable can lose the rename. This is exactly the A-STATE-01 recurring
    "missing parent-dir fsync" concern, now shown to be a duplicate-
    authority problem (5/6 sites independently omitted the fsync because
    each copy was written in isolation).
  - **Maintainability.** A future fix to the atomic-replace semantics
    (e.g. adding fsync, switching to `tmpfile(2)`, handling cross-device
    rename) must be applied in six places. The framework itself is
    inconsistent across its own two copies.
- Root cause: no canonical helper was extracted; each module wrote its
  own. The framework's `src/state/file.rs` is the only one that learned
  the parent-dir fsync lesson (A-STATE-01), and that lesson never
  propagated.
- Direction: add one canonical `pub fn atomic_write(path: &Path, bytes:
  &[u8]) -> io::Result<()>` (with parent-dir fsync) to `echo-core::utils`
  (or `echo-state::utils`), export it through the facade, and replace all
  six call sites. `write_json_atomic` becomes `serialize` + the canonical
  `atomic_write`. Per AGENTS.md cleanup rule, the five redundant copies
  are deleted in the same change.
- Regression validation: `cargo test --workspace --all-features --locked`
  for both repos; add one test asserting `atomic_write` fsyncs the parent
  (mockable via a `vfs` trait or asserted by code review + a
  `#[cfg(debug_assertions)]` counter).
- Validation reports: [V02-01](../validations/X-BND-01/V02-01.md)

### X-BND-01-P2-02: The framework `TaskSubagent` contract surface is dead — superseded by `RuntimeDagController`

- Priority: P2
- Confidence: high
- Layer: framework (deletion candidate)
- Evidence (V02-01, V03-01):
  - `echo-agent/echo-orchestration/src/tasks/runtime.rs:331` — `pub trait TaskSubagent: Send + Sync { async fn execute(...) -> Result<TaskExecutionSummary>; }`. **Zero implementors** in either repo (only the inherent `impl TaskSubagentContext` at `runtime.rs:264` exists).
  - `echo-agent/echo-orchestration/src/tasks/runtime.rs:309` — `pub struct TaskExecutionSummary` (framework). `echo-agent/echo-orchestration/src/tasks/runtime.rs:296` — `pub struct SuggestedTask` (framework). Both are return types of the dead trait.
  - `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:86` — `pub trait RuntimeDagController` is the LIVE task-execution contract (`dispatch_task` → `RuntimeTaskResolution`), used by `RuntimeDagExecutor`.
  - CLI: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:1222` — `impl RuntimeDagController for EkoRuntimeDagController` (the live implementation); `executor.rs:1645` constructs the framework `RuntimeDagExecutor::new(...)`.
  - CLI's adapter converter `TaskExecutionSummary::to_runtime_summary()` (`types.rs:1779`) and `SuggestedTask::to_runtime_suggested_task()` (`types.rs:1834`) have exactly one caller — a `#[test]` at `types.rs:2093` (inside the `#[test]` block opening at `:2061`).
- Reachability: `TaskSubagent` is `pub` and re-exported through
  `echo_agent::tasks::*`, but no production path constructs, dispatches,
  or consumes it. The framework's own `RuntimeDagExecutor` does not use
  it. The CLI production path persists the CLI-local
  `TaskExecutionSummary` (`executor.rs:2319/2413/5151` via
  `store.put_summary`) and never converts to the framework type outside
  tests.
- Expected invariant (AGENTS.md "framework API is retained unless
  framework-wide evidence shows it is obsolete or fully replaced" —
  deletion criterion (2)): a framework trait with zero implementors that
  is fully covered by a newer trait should be deleted, not retained as a
  menu option (it is not "one of multiple Store/Compressor impls"; it is
  a contract nothing honors).
- Observed behavior: `TaskSubagent` and its return-type contract sit in
  `tasks/runtime.rs` beside the live `TaskSubagentContext`/`TaskClaim`
  (which ARE used by `RuntimeDagController`). The CLI's adapter converters
  to the dead return types exist for symmetry but are test-only.
- Impact:
  - **Misleading API surface.** A framework consumer reading
    `echo_agent::tasks::TaskSubagent` believes it is the extension point
    for dynamic task execution. It is not — `RuntimeDagController` is.
    Third-party consumers would implement the wrong trait.
  - **Maintenance cost.** The CLI keeps two parallel `TaskExecutionSummary`
    /`SuggestedTask` types (one EKO, one framework) plus a converter,
    solely to satisfy a dead contract. The converter's only test caller
    exists to justify the converter.
- Root cause: `TaskSubagent` was the original task-execution abstraction;
  it was superseded by `RuntimeDagController` (which separates dispatch
  from resolution and adds product-policy hooks). The old trait and its
  return types were never removed.
- Direction: delete `pub trait TaskSubagent`, its return types
  `TaskExecutionSummary` and `SuggestedTask` (FW `runtime.rs`), and the
  re-exports. Keep `TaskSubagentContext` and `TaskClaim` (used by
  `RuntimeDagController`). In the CLI, drop the
  `to_runtime_summary`/`to_runtime_suggested_task` converters and their
  test — the EKO `TaskExecutionSummary`/`SuggestedTask` become the sole
  (app-local) types. Per AGENTS.md cleanup rule, deletion is preferred;
  no external consumer can break because nothing implements the trait.
  (If the framework wants to retain `TaskSubagent` as an alternative
  simpler extension point, it must ship at least one example and document
  the two paths — currently it does neither.)
- Regression validation: `cargo test --workspace --all-features --locked`
  for both repos; confirm no implementor remains after deletion.
- Validation reports: [V02-01](../validations/X-BND-01/V02-01.md),
  [V03-01](../validations/X-BND-01/V03-01.md)

### X-BND-01-P2-03: `GLOBAL_EVENT_BUS` / `EventBus` are dead framework infrastructure (reaffirms F-CORE-01-P2-01)

- Priority: P2
- Confidence: high
- Layer: framework (deletion candidate)
- Evidence:
  - `echo-agent/src/event_bus.rs:11-14` (`struct EventBus`), `:16-34`
    (methods), `:36-40` (`Default`), `:42-45` (`GLOBAL_EVENT_BUS`
    `LazyLock<Arc<EventBus>>`).
  - `echo-agent/src/lib.rs:39` — `pub mod event_bus;` (the only re-export).
- Reachability: at commit `9b0e0fa`, `GLOBAL_EVENT_BUS` appears ONLY at
  its definition. Zero `.send()` / `.subscribe()` /
  `EventBus::new` / `EventBus::default` callers in either repo (outside
  the definition file). Re-confirmed by V03-01 grep; identical to the
  F-CORE-01 V02-01 result at the same commit.
- Expected invariant (AGENTS.md "code cleanup" + deletion criterion (2)):
  a public multi-subscriber transport advertised as the unified event
  hub (doc at `event_bus.rs:1-4`) that has no producer or consumer is
  fully replaced (by direct stream composition via
  `envelope_event_stream`) and should be deleted.
- Observed behavior: the bus is scaffolded, exported, and documented as
  the fan-out point for "Webhook/Trace/UI/Audit", but no code path feeds
  it or reads from it.
- Impact: misleading framework API; cross-repo boundary cost is that an
  application subscriber (the natural consumer) is absent, so the bus is
  a dead cross-cutting surface rather than an active integration point.
  Reaffirms F-CORE-01-P2-01; restated here because the
  cross-repository-boundary task is the proper owner of the
  framework-vs-application event-distribution question.
- Root cause: scaffolded as a future fan-out point; never connected.
- Direction: delete `echo-agent/src/event_bus.rs`, the `pub mod event_bus;`
  at `lib.rs:39`, and any re-export. (Alternative: wire it — but F-CORE-01
  records that no concrete multi-sink consumer exists in either repo, so
  deletion is preferred under the cleanup rule.)
- Regression validation: `cargo check --workspace --all-features --locked`
  and `cargo check -p echo_agent --no-default-features --locked` after
  deletion; no caller exists to break.
- Validation reports: [V03-01](../validations/X-BND-01/V03-01.md)
  (re-affirms [F-CORE-01 V02-01](../validations/F-CORE-01/V02-01.md))

### X-BND-01-P3-01: `WorktreeError` is defined byte-identically in the framework and the CLI

- Priority: P3
- Confidence: high
- Layer: adapter
- Evidence (V01-01):
  - `echo-agent/src/agent/subagent/worktree.rs:35-52` — `pub struct WorktreeError { pub message: String }` with `Display`, `Error`, and `pub fn new(message: impl Into<String>) -> Self`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs:117-145` — the SAME `pub struct WorktreeError { pub message: String }`, identical `Display`/`Error`/`new`, plus an extra `impl From<std::io::Error>`.
- Reachability: both are live error types for their respective worktree
  subsystems (framework: subagent fork isolation; CLI: task-runtime
  logical-task worktree).
- Expected invariant: a trivial string-wrapper error type should be
  defined once and reused, unless the two subsystems genuinely need
  different error semantics.
- Observed behavior: the two definitions are byte-identical except for
  the CLI's extra `From<io::Error>`. The framework type is `pub` and
  re-exported (`subagent/mod.rs:51`).
- Impact: low. Maintenance cost (a Display/Error fix must be applied
  twice) and conceptual duplication. Not a correctness defect.
- Root cause: the CLI task-runtime worktree was written as a standalone
  module and re-created the error type rather than importing the
  framework's.
- Direction: add `From<std::io::Error>` to the framework
  `subagent::worktree::WorktreeError` and have the CLI task-runtime
  reuse it. If the task-runtime subsystem genuinely needs a separate
  type (e.g. to carry richer git context), document why beside the CLI
  definition.
- Regression validation: `cargo check --workspace --all-features --locked`
  for both repos.
- Validation reports: [V01-01](../validations/X-BND-01/V01-01.md)

### X-BND-01-P3-02: CLI task-runtime worktree reimplements the `git worktree add` invocation instead of reusing the framework helper

- Priority: P3
- Confidence: medium
- Layer: adapter
- Evidence (V02-01):
  - Framework pub helper: `echo-agent/echo-tools/src/git_worktree.rs:37` — `pub fn create_worktree(...)` running `git worktree add`.
  - Framework trait: `echo-agent/src/agent/subagent/worktree.rs:85` — `pub trait WorktreeFactory` (used for subagent isolation by the subagent executor at `executor.rs:323`).
  - CLI task-runtime worktree: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs:149` (`pub fn run_git(...)`, its own git-invocation helper), `:412` (`run_git(repo_root, &["worktree", "add", path_text, &branch])`), plus its own prune (`:403`) and merge-base (`:394,414`) logic.
- Reachability: the CLI task-runtime worktree is live (drives logical-task
  isolation for writer subagents). The framework `echo_tools::git_worktree`
  helper is live (used by the framework subagent path).
- Expected invariant: a single git-invocation primitive (or a single
  `WorktreeFactory` implementor) wraps `git worktree add` for the whole
  process, unless the two subsystems have materially different semantics.
- Observed behavior: two independent `git worktree add` invocation paths.
  The CLI's adds prune/merge-base/branch-validation that the framework
  helper may lack — a possible justification, but it is not documented.
- Impact: low. Behavioral drift risk (e.g. error handling, lock-recovery,
  worktree-list parsing) between the two paths. Not a correctness defect
  today.
- Root cause: the task-runtime worktree subsystem predates or was written
  without reference to `echo_tools::git_worktree`.
- Direction: either (a) reuse `echo_tools::git_worktree::create_worktree`
  for the primitive and layer the prune/merge-base logic on top in the
  CLI, or (b) document beside `task_runtime/worktree.rs:149` why a
  separate git invocation is required. Decide before the next worktree-
  semantics change.
- Regression validation: `cargo test --workspace --all-features --locked`
  for both repos; a worktree create/destroy round-trip test through the
  chosen path.
- Validation reports: [V02-01](../validations/X-BND-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Type/trait/name duplicate-authority search (20 shared pub names classified) | yes | passed_with_notes | [V01-01](../validations/X-BND-01/V01-01.md) |
| V02 | Behavior/call-path search (atomic_write ×6, `TaskSubagent` liveness, workflow engine, worktree git) | yes | failed (P2-01, P2-02 surfaced) | [V02-01](../validations/X-BND-01/V02-01.md) |
| V03 | Public framework option check (Store/Compressor menu retained; `TaskSubagent`/`GLOBAL_EVENT_BUS` dead) | yes | passed_with_notes | [V03-01](../validations/X-BND-01/V03-01.md) |
| V04 | Adapter-logic/deletion-target matrix (adapters thin; 5 closed deletion targets) | yes | passed_with_notes | [V04-01](../validations/X-BND-01/V04-01.md) |
| V05 | Historical-document drift | conditional | not-applicable | No historical boundary document is reused as evidence; the referenced F-CORE-01 / A-TSK-01 / B-ARCH-01 / F-API-01 findings are re-verified at the current commits in V01-V04 (see Historical Claim Status). |

The task is `complete`: every required validation has its own report with
a definitive result. Two validations surface findings (V02 failed; V03
and V04 passed-with-notes that become findings). No fix is implemented in
this review.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-CORE-01-P2-01: `GLOBAL_EVENT_BUS`/`EventBus` dead | current (reaffirmed) | V03-01: zero callers at `9b0e0fa`; restated as X-BND-01-P2-03 (proper owner is the cross-repo boundary task). |
| F-API-01-P2-02 / B-ARCH-01-P2-02: facade parallel access paths | current (out of scope here) | Owned by F-API-01 / B-ARCH-01; this task did not re-audit facade re-export paths. The `workspace` escape-hatch debt is framework-internal, not a framework-vs-application boundary issue. |
| B-ARCH-01-P2-01: root owns real impl (`RuntimeStateStore`, `EventBus`, LLM wrappers) | current (partly in scope) | The `EventBus` portion is reaffirmed as dead (P2-03). `RuntimeStateStore`-in-root and LLM-compat-wrappers-in-root are framework-internal migration debt (B-ARCH-01 owns); they do not create a framework-vs-application duplicate authority because the CLI consumes them through the facade. |
| A-TSK-01: TaskRuntime file authority singular; `EkoRevisionedTaskStore`/`EkoTaskToolPolicy` thin | current (extended) | V01-01 extends A-TSK-01 V03-01 to the whole CLI tree: zero parallel task/plan/todo/store/validator/CRUD. V04-01 confirms `EkoRuntimeDagController` is also thin. The A-TSK-01-P2-02 `Retrying`/`Paused` round-trip lossiness is a field-level concern owned by X-TSK-01, not re-audited here. |
| A-TSK-01 handoff: "EKO may layer product policy on top of the framework model, not beside it" | current (supported) | V04-01: CLI uses `RuntimeDagController` product-policy hooks; no second frontier/scheduler/validator/retry. |
| X-INV-01 V02-01: CLI never enables SQLite | current (reaffirmed) | V03-01: CLI picks File backends; `SqliteStore`/`SqliteConversationStore` are retained framework options, not deletion targets. |
| X-INV-01 V03-01: no parallel task CRUD | current (reaffirmed) | V01-01: zero `todo_write`/`plan_create`/`plan_patch`/`plan_execute`; only `task_execute` (permitted) added by CLI. |
| `echo-orchestration/src/tasks/runtime.rs:1-5` doc: "Generic task-runtime primitives … product-neutral" | current for the live subset; stale for `TaskSubagent` | `TaskSubagentContext`/`TaskClaim` (product-neutral, used by `RuntimeDagController`) are current. `TaskSubagent` trait + its return types are dead (P2-02). |
| `subagent/worktree.rs:30-50` doc: "the framework doesn't model git internals; the application's concrete factory surfaces the diagnostic" | current | V01-01/V02-01: framework `WorktreeFactory` is an abstract trait; CLI provides concrete worktree ops. The trivial `WorktreeError` duplication (P3-01) is the only blemish. |

## Coverage And Uncertainty

- **Type layer fully covered.** All 20 shared pub names classified (V01).
  The CLI's 3 pub traits enumerated; none duplicates a framework trait.
- **Behavior layer covered for the high-value behaviors.** `atomic_write`
  (6 sites), the `TaskSubagent` contract, workflow execution, and
  worktree creation are traced end-to-end. Behaviors NOT exhaustively
  audited: every `append_*`/`read_*` persistence helper (only
  `atomic_write` and `file_shadow::append_line` were inspected), every
  error-type definition (only `WorktreeError` was spot-checked), every
  status-mapping function (A-TSK-01 / X-TSK-01 own the field-level
  mapping).
- **`Topology` (1 implementor) not classified.** F-MAG-01 owns the
  handoff/topology ownership review. Residual uncertainty: a single-
  implementor trait may be a legitimate abstraction or a half-finished
  parallel authority; not resolved here.
- **`Handoff` not found as a trait.** The `pub mod handoff` re-exports
  `HandoffTool` (a tool, not a trait). F-MAG-01 owns the deeper review.
- **No executable validations run.** All four validations are static
  inspection + grep at the named commits. P2-01's parent-dir fsync
  observation is structural (read the bodies); P2-02's dead-trait claim
  is structural (zero implementors); P2-03's dead-bus claim is structural
  (zero callers). These are robust because they rely on exhaustive grep,
  not behavior.
- **echo-agent dirty paths.** The five modified files
  (`src/agent/react/run/*`, `src/testing/*`) are outside the boundary
  scope and were read at the committed version; the dirty modifications
  do not affect type/trait/behavior authorities in tasks/subagent/
  workflow/adapter/state.
- **Cross-references to A-TSK-01 / F-CORE-01 / X-INV-01 are taken at the
  matching baseline commits.** If those reports' conclusions move, the
  corresponding rows of this report's Historical Claim Status move with
  them.

## Handoff

Conclusions downstream tasks may rely on:

1. **No parallel task/plan/todo/store/validator/CRUD authority exists in
   the CLI.** V01-01 extends A-TSK-01 V03-01 to the whole CLI tree. The
   CLI's only framework-trait implementation for task execution is
   `RuntimeDagController` (`EkoRuntimeDagController`). AGENTS.md rule 6
   holds at the type, behavior, and adapter layers.
2. **The live framework task-execution contract is `RuntimeDagController`,
   not `TaskSubagent`.** `X-TSK-01` and `A-TSK-03` should treat
   `RuntimeDagController` as the sole task-execution extension point;
   `TaskSubagent` is dead (P2-02).
3. **`atomic_write` is a duplicate authority (6 copies).** Any
   durability/atomicity fix (A-STATE-01's parent-dir fsync, or future
   tmpfile/cross-device work) MUST consolidate to one helper or it will
   re-diverge. P2-01.
4. **The framework backend menu is correctly retained.** `F-MEM-02`
   (SQLite capabilities) and `Q-FW-02` (feature matrix) may rely on
   `SqliteStore`/`HybridCompressor`/etc. being legitimate retained
   options; the CLI's non-use is product policy.
5. **The CLI workflow feature is single-authority.** `F-WFL-01` may rely
   on the CLI routing workflow execution through the framework engine
   (no second engine in app-core).

Reports downstream tasks must read:

- `X-TSK-01` (task graph conformance) must read V02-01 (the
  `to_runtime_summary` test-only finding) and V04-01 (adapter authority)
  — it inherits the dead-`TaskSubagent` surface and the
  `EkoRuntimeDagController` thinness conclusion.
- `S-X-01` (cross-repository synthesis) should read V04-01's
  deletion-target matrix in full — it is the canonical list of five
  duplicate/dead authorities for the iteration roadmap.
- Any future task touching `atomic_write`, `WorktreeError`, or the git
  worktree primitive should read V02-01 and V01-01.

Conditions that make this report stale:

- Any commit that implements `TaskSubagent` (in either repo) or deletes
  it invalidates P2-02.
- Any commit that wires a producer/consumer to `GLOBAL_EVENT_BUS`, or
  deletes the bus, invalidates P2-03.
- Any commit that consolidates `atomic_write` into a single helper
  invalidates P2-01.
- Any commit that unifies `WorktreeError` or reuses
  `echo_tools::git_worktree::create_worktree` from the CLI task-runtime
  invalidates P3-01 / P3-02.
- Any commit that introduces a parallel task/plan/todo store, validator,
  or CRUD tool in the CLI invalidates V01-01's positive conclusion.
- A change to the baseline commits (`9b0e0fa` / `b3b2e81`) requires
  re-running V01-V04.

Follow-up task IDs (recommended, not implemented in this review):

- A framework cleanup task to delete the `TaskSubagent` trait + return
  types and the `GLOBAL_EVENT_BUS`/`EventBus` surface (P2-02, P2-03).
- A framework consolidation task to extract one canonical `atomic_write`
  with parent-dir fsync and migrate the six call sites (P2-01). This
  also closes the A-STATE-01 parent-dir fsync finding at its root.
- A small adapter cleanup task to deduplicate `WorktreeError` and decide
  on `echo_tools::git_worktree` reuse (P3-01, P3-02).
