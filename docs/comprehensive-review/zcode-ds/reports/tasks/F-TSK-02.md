# F-TSK-02: DAG validation and dependency analysis

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Is there one structural validator and one dependency analysis for cycles,
missing nodes, readiness, skip, and blocked propagation?

**Answer: Yes for structure (one `PlanValidator`, one DFS/topology, one ready
frontier) and for blocked propagation; NO for skip — skip has no propagation
and skipping a task with Pending dependents stalls and fails the run with a
misleading "cycle or blocked" error (P1). Two additional P2 issues: a
cross-repository string-literal contract for blocked propagation, and an
accepted-but-never-enforced `execution_mode: "sequential"`.**

## Scope

- `echo-orchestration/src/planning/`: `validator.rs` (PlanValidator,
  `task_dependency_cycles`, `task_topology`, `task_topological_order`),
  `plan_spec.rs` (PlanSpec -> TaskSpec compilation, topological_order,
  DependencyType semantics), `policy.rs` (classified, out of analysis).
- `echo-orchestration/src/tasks/`: `runtime.rs` (`DagExecutionState`,
  `ready_task_ids`, `blocked_by_failures`, `all_completed`,
  `all_unfinished_failed_or_blocked`, `refresh_in_flight`, `TaskStatus`),
  `runtime_executor.rs` (safe-point validation, wave selection, failure
  blocking, stall), `revisioned.rs` (patch engine, Skip op,
  `finalize_and_validate`), `dag.rs` (delegation), `manager.rs` (legacy
  queries, reachability only), `scheduler.rs`/`replanner.rs`/
  `background_task.rs`/`composite.rs`/`events.rs`/`hooks.rs` (duplicate
  search only).
- `echo-agent/src/topology.rs`, `src/state/mod.rs` (TaskNode),
  `echo-orchestration/src/workflow/dag.rs` (duplicate search; distinct
  concepts).
- EKO `echo-agent-app-core/src/tasks/task_runtime/`: `revisioned_adapter.rs`
  (policy hook), `task_tools.rs` (`TaskCapabilityCatalog::validate_task_spec`),
  `store.rs` (commit path, retry-unblock walk, recovery skip, test helpers),
  `executor.rs` (`EkoRuntimeDagController`, `select_ownership_safe_wave`,
  drain loop, `note_stalled`), `types.rs` (`to_task`, `TaskUpdateOperation`),
  `planner.rs` (file ownership policy), `compact_context.rs` (mode display).

## Out Of Scope

- `RuntimeDagExecutor` claims/retries/cancellation/waves semantics beyond the
  frontier and blocked/skip propagation -> F-TSK-03.
- EKO file authorities, adapter losslessness, authoring tools -> A-TSK-01,
  A-TSK-02; EKO controller boundary -> A-TSK-03; recovery/claims -> A-TSK-04.
- Static workflow engine validation (`workflow/dag.rs` own Kahn/DFS) ->
  F-WFL-01. Agent call-graph topology (`src/topology.rs`) -> F-MAG-01.
- TaskNode checkpoint state machine -> F-TSK-01-P3-04.
- Legacy `TaskManager`/`TaskExecutor` production-unreachable surface ->
  F-TSK-01-P3-01 (second ready loop `get_ready_tasks`/`wake_dependents`/
  `execute_ready_tasks` exists there; production callers are `#[cfg(test)]`
  only; no new finding created here).

## Inputs

- Root `AGENTS.md` (single task-relation API; one authoritative
  validator/scheduler; framework-vs-app layering; Subagent terminology;
  "already exists" gate).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (F-TSK-02 card),
  `zcode-ds/README.md`.
- Dependency report read: `zcode-ds/reports/tasks/F-TSK-01.md` (canonical
  model singular; legacy surface P3-01; PlanValidator as structural
  authority; handoff explicitly deferred validator/readiness semantics to
  F-TSK-02).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md:366,383-387`
  (M13 Phases 1-5), `echo-agent-cli/docs/2026-07-27-runtime-dag-kernel-convergence.md`
  (Phase 1-5 claims, verification list).

## Layering Decision

- Generic mechanism (framework, correctly placed): `PlanValidator` +
  `task_dependency_cycles`/`task_topology`/`task_topological_order`
  (validator.rs), `PlanSpec::to_task_specs` compilation incl. optional-edge
  validation (plan_spec.rs:359-393), `DagExecutionState` ready/blocked/
  completion analysis (runtime.rs:438-500), `RuntimeDagExecutor` safe-point
  validation + stall (runtime_executor.rs:214-219, 287-313),
  `TaskRevisionService::finalize_and_validate` (revisioned.rs:990-1013).
- EKO product policy (application): `TaskCapabilityCatalog::validate_task_spec`
  (catalog capability checks), `select_ownership_safe_wave` (writer-ownership
  filter of the framework frontier), retry-unblock walk (store.rs:1287-1335),
  recovery skip (store.rs:2133-2176), drain loop/completion gate
  (executor.rs:360-445), TodoStatus projections.
- Adapter boundary: `EkoRevisionedTaskStore` (thin), `EkoTaskToolPolicy::validate_candidate`
  (policy hook into the framework's finalize path) — with one defect: the
  blocked-propagation marker crosses the boundary as a string literal
  (P2-01).
- Duplicate search terms (both repos, V01): `PlanValidator`,
  `validate_task_snapshot`, `validate_task_specs`, `task_dependency_cycles`,
  `task_topological_order`, `task_topology`, `CycleVisitState`, `VisitState`,
  `detect_cycle`, `topological_sort`, `indegree`, `ready_task_ids`,
  `get_ready_tasks`, `ready_frontier`, `frontier`, `select_ready_wave`,
  `wake_dependents`, `get_dependency_chain`, `get_next_task`,
  `execution_mode`, `parallel_group`, `Skipped` (dependency semantics).
  Result: one structural validator, one DFS/topology, one production ready
  frontier; legacy second ready loop is test-only; workflow engine and agent
  topology are distinct concepts (V01-01).
- Migration deletion check: legacy `TaskManager` ready queries and
  `TaskExecutor::execute_ready_tasks` are production-unreachable (F-TSK-01
  P3-01); `get_dependency_chain_recursive` chain query likewise; no new
  deletion target beyond F-TSK-01's P3-01 decision.

## Current Path

Verified call graph (details in V02-01):

1. Authoring: `PlanSpec::validate` -> `to_task_specs` (Required edges only;
   dangling Preferred/Optional endpoints still error, plan_spec.rs:363-374)
   -> `PlanValidator::validate_task_specs` (validator.rs:210-294).
2. Commit: `task_create/task_update` -> `TaskRevisionService` ->
   `TaskPatchEngine::apply_operations` -> `finalize_and_validate` ->
   `PlanValidator::validate_task_snapshot` (revisioned.rs:1007-1011) ->
   store CAS. EKO reuses the same service; its store test helper and native
   commit path also call the framework validator (store.rs:921-923,
   `#[cfg(test)]` at 98-100).
3. Execution: `task_execute` -> EKO `execute_runtime_plan`
   (executor.rs:1623-1681) -> `RuntimeDagExecutor::execute`; every loop
   re-validates the snapshot (runtime_executor.rs:214-219), derives
   `DagExecutionState::from_tasks`, reads `ready_task_ids`
   (runtime_executor.rs:275), filters via controller `select_ready_wave`
   + `validate_selected_wave` (318-319, 467-492), dispatches the wave.
4. Failure: `blocked_by_failures` -> `block_task` with reason
   "blocked: upstream task failed" (runtime_executor.rs:235-246) ->
   EKO marks TodoStatus::Blocked; retry unblocking walks dependents by
   matching that summary string (store.rs:1287-1295).
5. Skip: `TaskUpdateOperation::Skip` (EKO types.rs:1301) / recovery skip
   (store.rs:2170-2173) -> `TaskStatus::Skipped` -> next executor loop:
   dependents stay Pending (dep not `completed`), are not blocked (dep not
   `failed`) -> `note_stalled` -> `RuntimeDagOutcome::Failed("<none>",
   "DAG stalled with unfinished tasks (cycle or blocked)")` (308-313).

## Findings

### F-TSK-02-P1-01: Skip is terminal without dependency propagation — skipping a task with Pending dependents stalls and fails the whole run with a misleading error

- Priority: P1
- Confidence: medium
- Layer: framework (reachable through EKO skip entry points)
- Evidence: `echo-orchestration/src/tasks/runtime.rs:449-455` (`ready_task_ids`
  requires every dependency in `completed`; `Skipped` never satisfies),
  `runtime.rs:481-483` (`all_completed` counts Skipped as resolved — the only
  place skip is treated as satisfying), `runtime_executor.rs:308-313` (stall ->
  `Failed { "<none>", "DAG stalled with unfinished tasks (cycle or blocked)" }`),
  `revisioned.rs:547-568` (Skip op on any Pending/Blocked task),
  EKO `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:1301`
  (Skip op exposed in task_update), `store.rs:2170-2173` (recovery skip sets
  only the one task).
- Reachability: `task_update` Skip (GUI/TUI/CLI tool surface, register.rs)
  or EKO recovery skip -> snapshot contains `TaskStatus::Skipped` with
  Pending dependents -> framework executor next safe point returns Failed.
  Verified code trace (V03-01); skip entry points verified live by tests
  V04-06 (`task_update_skip_preserves_spec_and_updates_execution`).
- Expected invariant: the single dependency analysis covers skip: a skipped
  dependency either resolves its dependents (skip propagation) or skip is
  restricted to leaf tasks; the run must not fail with a message naming
  "cycle or blocked" for a deliberate user action.
- Observed behavior: dependents of a skipped task remain Pending forever —
  not ready (dependency not completed), not blocked (dependency not failed) —
  so the executor reports a stall and fails the run. In the recovery flow a
  user must skip each downstream node one by one.
- Impact: a supported user operation (skip) on a non-leaf task hard-fails the
  run with a misleading reason; no automated skip propagation exists; the
  convergence document's claim that skipped nodes "count as deliberately
  resolved instead of producing a false DAG stall" holds only for
  all-skipped graphs, not mid-DAG skips.
- Root cause: the M13 convergence integrated `Skipped` into completion
  accounting (`all_completed`) but never into dependency resolution
  (`ready_task_ids` still demands `completed`), leaving the one frontier
  without skip semantics.
- Direction: extend the single analysis — in `DagExecutionState`, treat a
  `Skipped` (and cancelled, at safe-point policy) dependency as satisfying
  readiness, or propagate `Skipped` to direct dependents at the safe point;
  differentiate the stall reason (skip dependency vs genuine deadlock) and
  keep the framework the sole owner of the propagation. Delete no code: the
  change is inside `ready_task_ids`/executor, with the EKO projections
  following automatically.
- Regression validation: a framework fixture "graph A(Skipped) -> B(Pending)
  completes with B skipped/completed (or returns a precise skip reason,
  never 'cycle or blocked')" in `echo_orchestration`; an EKO end-to-end
  test skipping a mid-DAG task via `task_update` and asserting run
  completion.
- Validation reports: [V03-01](../validations/F-TSK-02/V03-01.md),
  [V02-01](../validations/F-TSK-02/V02-01.md),
  [V04-03](../validations/F-TSK-02/V04-03.md),
  [V04-06](../validations/F-TSK-02/V04-06.md)

### F-TSK-02-P2-01: EKO retry-unblock recovery is coupled to a cross-repository string literal `"blocked: upstream task failed"`

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: framework writer `echo-orchestration/src/tasks/runtime_executor.rs:243`
  (`block_task(..., "blocked: upstream task failed")`), EKO matcher
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1287-1295`
  (retry-unblock walk selects `TodoStatus::Blocked` todos whose `summary ==
  "blocked: upstream task failed"` to distinguish propagation blockers from
  review/acceptance blockers).
- Reachability: every upstream-failure propagation writes the string
  (runtime_executor.rs:239-244); every user-initiated retry reads it
  (store.rs:1261-1350). Both strings verified identical on the reviewed
  commits.
- Expected invariant: blocked-propagation reasons cross the repository
  boundary as a typed, stable contract, not as human-visible message text.
- Observed behavior: EKO recovery correctness depends on the exact English
  sentence emitted by the framework executor; review/acceptance blockers keep
  their own text and are intentionally excluded by the same match.
- Impact: a wording change to the framework message silently disables EKO's
  descendant-unblock after retry — descendants stay Blocked and the
  completion gate (store.rs:460-480) blocks the run indefinitely; no compile
  or test failure would surface the break.
- Root cause: the framework propagates blocked state as a free-form string
  (`TaskStatus::Blocked(String)`), and EKO re-encodes it as TodoStatus
  summary, then string-matches to recover the propagation structure.
- Direction: make propagation a typed contract — e.g., a structured
  `Blocked` reason enum or a dedicated field (event/projection carries
  `blocked_by_propagation: bool` or the failing upstream task id); EKO
  matches the type, not the text. The string matcher and its summary-write
  counterpart are the deletion targets once the typed field lands.
- Regression validation: a framework test pinning the propagation marker;
  an EKO test feeding one propagated Blocked and one review Blocked todo and
  asserting only the propagated one is reset after retry; a change-tolerance
  test that renames the message and still unblocks.
- Validation reports: [V03-01](../validations/F-TSK-02/V03-01.md),
  [V05-01](../validations/F-TSK-02/V05-01.md)

### F-TSK-02-P2-02: `execution_mode: "sequential"` is accepted, stored, and displayed but never enforced — the single frontier always dispatches the full ready wave

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: schema `["parallel","sequential"]`
  (`echo-orchestration/src/tasks/task_tools.rs:245`), parse
  (task_tools.rs:419-424), stored in `TaskGraphContext.execution_mode`
  (`revisioned.rs:31-38`), displayed to the model by EKO
  (`echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/compact_context.rs:199-201`);
  zero reads of `execution_mode`/`TaskGraphExecutionMode::Sequential` in
  `runtime_executor.rs` and in EKO `executor.rs` production code (V01/V03);
  EKO `select_ownership_safe_wave` (executor.rs:1127-1145) filters only
  writer ownership.
- Reachability: any plan created with `task_create execution_mode:
  "sequential"` (GUI/TUI/CLI/LLM) — the mode survives commit and
  presentation but never reaches the executor.
- Expected invariant: an advertised schema enum value must have observable
  scheduling effect (one task dispatched per wave for Sequential) or must
  not be advertised.
- Observed behavior: `Sequential` plans execute exactly like `Parallel`
  plans — the full ready frontier is dispatched concurrently, bounded only by
  `max_concurrent_subagents`.
- Impact: users/LLMs requesting sequential execution (e.g., to serialize
  dependent writes beyond DAG edges) silently get parallel waves; the model
  is shown `execution_mode=sequential` while actual behavior differs;
  potential write races for plans that relied on serialization.
- Root cause: the execution-mode field was added to the authoring artifact
  and context but the M13 runtime kernel did not wire it into the frontier
  or wave loop.
- Direction: enforce in `RuntimeDagExecutor` (when `context.execution_mode ==
  Sequential`, dispatch at most one selected task per wave), or remove the
  mode from the schema/context/UI if sequentiality is product policy to
  implement in the controller; add the enforcement at the framework layer
  with a scripted-controller fixture asserting wave size 1.
- Regression validation: framework fixture "Sequential mode dispatches one
  task per wave even when the frontier has multiple ready tasks"; EKO test
  asserting `compact_context` output matches actual wave behavior.
- Validation reports: [V03-01](../validations/F-TSK-02/V03-01.md),
  [V01-01](../validations/F-TSK-02/V01-01.md)

### F-TSK-02-P3-01: `DagRefresh` ordering is nondeterministic (HashSet iteration) and `refresh_in_flight` has no production caller

- Priority: P3
- Confidence: high (fact) / low (impact)
- Layer: framework
- Evidence: `echo-orchestration/src/tasks/runtime.rs:395-435`
  (`refresh_in_flight` iterates the `in_flight` HashSet at :401; pushes to
  `DagRefresh` lists in that nondeterministic order); only caller is a test
  (runtime.rs:647).
- Reachability: definition -> public API (`DagExecutionState` is exported) ->
  no production call site in either repository today.
- Expected invariant: public DAG bookkeeping APIs with ordered vector output
  are deterministic for equal inputs.
- Observed behavior: when several externally-completing in-flight tasks are
  refreshed in one call, the `completed/failed/terminal_non_success` lists
  are emitted in HashSet iteration order, which is not stable across runs.
- Impact: none today (no production caller); a future caller consuming
  `DagRefresh` order for event replay or projections would observe
  nondeterministic event order.
- Root cause: iteration over a `HashSet` clone instead of the deterministic
  tasks slice.
- Direction: iterate the `tasks` slice (or sort ids) when populating
  `DagRefresh`; if the framework-deletion gate (F-TSK-03 authority review)
  confirms zero framework consumers, fold `refresh_in_flight` into the
  legacy-surface deletion decision with `DagRefresh`.
- Regression validation: fixture with two in-flight tasks completing
  simultaneously asserting stable refresh-list order across repeated runs.
- Validation reports: [V03-01](../validations/F-TSK-02/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (validator/DFS/frontier across both repos) | yes | passed | [V01-01](../validations/F-TSK-02/V01-01.md) |
| V02 | Registration and runtime reachability trace (tools -> service -> validator -> executor) | yes | passed | [V02-01](../validations/F-TSK-02/V02-01.md) |
| V03 | Invariants: cycle/missing/self-dep fixtures; frontier determinism; status independence; skip/blocked semantics | yes | passed (3 findings produced) | [V03-01](../validations/F-TSK-02/V03-01.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked planning::validator` | yes | passed (exit 0, 11 ok) | [V04-01](../validations/F-TSK-02/V04-01.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked tasks::runtime` | yes | passed (exit 0, 17 ok) | [V04-02](../validations/F-TSK-02/V04-02.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked tasks::runtime_executor` | yes | passed (exit 0, 7 ok) | [V04-03](../validations/F-TSK-02/V04-03.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked tasks::tests` + `tasks::revisioned` | yes | passed (exit 0, 13 + 3 ok) | [V04-04](../validations/F-TSK-02/V04-04.md) |
| V04 | EKO `file_path_rejects_dependency_cycle_and_appends_no_event` | yes | passed (exit 0, 1 ok) | [V04-05](../validations/F-TSK-02/V04-05.md) |
| V04 | EKO `task_update_skip_*` + `task_update_update_requeues_blocked_task` | yes | passed (exit 0, 2 ok) | [V04-06](../validations/F-TSK-02/V04-06.md) |
| V05 | Historical-doc drift (MASTER-PLAN M13, convergence record) | conditional | passed | [V05-01](../validations/F-TSK-02/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN M13 P1: framework `RuntimeDagExecutor` sole execution loop; ready frontier / wave / cancellation / failure propagation / stall removed from the app layer | current | `runtime_executor.rs:196-449`; EKO `execute_runtime_plan` delegates (executor.rs:1623-1681); EKO drain loop computes completion policy only (executor.rs:360-445) |
| MASTER-PLAN M13 P2: framework `PlanValidator` sole structural authority; EKO duplicate dependency/DFS validator deleted | current | `revisioned.rs:1007-1011`; EKO `validate_task_spec` is capability policy (task_tools.rs:49-81); `CycleVisitState` sole DFS (validator.rs:348-407) |
| MASTER-PLAN M13 P3: `PlanSpec`/`TaskManager` topology queries unified into canonical topology; no own Kahn/depth validator | current | `plan_spec.rs:429-432`; `dag.rs:7-24` |
| MASTER-PLAN M13 P4: `TaskManager` cycle query on canonical dependency analysis; manager-local DFS/`VisitState` deleted | current | `dag.rs:7-24`; `get_dependency_chain_recursive` remains as chain query (manager.rs:384-403, matches F-TSK-01 V05) |
| Convergence record P1: "Fixed skipped-plan nodes so they count as deliberately resolved instead of producing a false DAG stall" | current but incomplete | `all_completed` counts skipped (runtime.rs:481-483) and all-skipped graphs complete (V04-03); mid-DAG skip still stalls (runtime.rs:449-455 + runtime_executor.rs:308-313) -> F-TSK-02-P1-01 |
| Convergence record: adapter "may not implement another DAG loop, dependency validator, or generic retry state machine" | current with caveat | EKO has no second frontier/validator; retry-unblock walk (store.rs:1287-1335) is recovery policy coupled to the framework by a string literal -> F-TSK-02-P2-01 |
| F-TSK-01 P3-01: legacy `TaskManager`/`TaskExecutor` task-graph surface production-unreachable (incl. its own `get_ready_tasks` ready logic) | current | manager.rs:241-256, executor.rs:570; all constructions/callers `#[cfg(test)]` (V01-01); no new finding raised here |

## Coverage And Uncertainty

- No end-to-end EKO run was executed with a skipped mid-DAG task; the P1-01
  stall path is a deterministic code trace (runtime.rs + runtime_executor.rs)
  plus live skip entry points (V04-06), not an executed scenario — hence
  medium confidence on the end-to-end impact.
- `RuntimeDagExecutor` claim/retry/recovery semantics beyond the frontier are
  deliberately left to F-TSK-03; `refresh_in_flight` may gain a caller there.
- EKO `compare_and_commit_revisioned_task_graph` CAS correctness is A-TSK-01
  scope; only its delegation of validation was verified here.
- Workflow engine validation (`workflow/dag.rs`) was classified as a distinct
  concept by inspection, not by a full F-WFL-01 review.
- The `sequential`-mode finding (P2-02) relies on zero-grep of
  `execution_mode` reads in both executors; a future controller or dispatcher
  implementing sequentiality outside the reviewed paths would invalidate it.

## Handoff

- Downstream tasks may rely on: one structural validator (PlanValidator) and
  one DFS/topology on every live path of both repositories (V01/V02/V04);
  deterministic frontier ordering and status-independent structural checks
  (V03); forward blocked propagation exists (V04-03); EKO has no second
  validator/frontier (V01/V04-05); skip-with-dependents stalls the run
  (P1-01); blocked reasons cross the repo boundary as a string literal
  (P2-01); `execution_mode` sequential is inert (P2-02).
- Reports to read: all ten validation reports above; F-TSK-01 for the
  canonical model and legacy-surface reachability; F-TSK-01-P3-03 for the
  `max_retries`/schema interplay adjacent to `validate_task_specs`.
- Stale conditions: this report becomes stale if `runtime.rs` frontier
  semantics, `validator.rs` structural checks, `runtime_executor.rs` stall
  path, EKO store retry-unblock, or the `execution_mode` wiring change.
- Follow-up task IDs: F-TSK-03 (skip handling in claims/waves; P3-01
  `refresh_in_flight` caller), A-TSK-03 (controller boundary; P2-01/P2-02
  EKO surface), A-TSK-04 (recovery skip/retry flows; P1-01 EKO reachability),
  X-TSK-01 (canonical graph conformance; P2-01 typed contract), S-RDM-01
  (P1-01 skip propagation, P2-01 typed blocker, P2-02 sequential
  enforcement, P3-01 cleanup).
