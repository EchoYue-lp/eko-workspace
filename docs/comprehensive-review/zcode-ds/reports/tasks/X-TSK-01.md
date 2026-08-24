# X-TSK-01: Task graph and adapter conformance

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both source repositories; the cargo test runs left no
> tracked changes)

## Question

Is there one revisioned TaskRun graph with lossless EKO projection and no
second validator/executor/store authority?

**Answer: Yes. There is exactly one live revisioned task graph
(`TaskRevisionService` + `PlanValidator` + `RuntimeDagExecutor` +
`RevisionedTaskStore` in `echo-orchestration`), one file-backed store CAS in
EKO (`EkoRevisionedTaskStore`/`TaskRuntimeStore`), and the EKO projection is
lossless for every field and every status the live paths can produce. All nine
established framework/application findings in this area were re-anchored and
remain `current`; they are behavior defects inside the single authority or its
adapter, not second authorities. One new P3 finding: the read-back adapter
silently fabricates a Pending execution for plan tasks missing from
`run-state.json`, violating losslessness inside the A-TSK-01-P2-01 crash
window. No P0/P1/P2 new finding.**

## Scope

Primary source paths inspected (this task; dependency reports cover the
deep reads):

- Framework model/authority: `echo-agent/echo-orchestration/src/tasks/
  runtime.rs` (TaskSpec/TaskExecution/TaskStatus/TaskClaim/DagExecutionState,
  full), `revisioned.rs` (TaskRevisionService/TaskPatchEngine/
  TaskPlanPatchOp/TaskGraphCommit/RevisionedTaskStore, full),
  `task_tools.rs` (tool names), `runtime_executor.rs` (executor construction,
  wave/cancel/stall anchors), `executor.rs`/`manager.rs` (legacy reachability),
  `echo-agent/src/agent/react/builder.rs:1181` (single-API test).
- EKO adapter/model: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
  types.rs` (EkoTaskSpec/PlanTask/EkoTaskExecution/TodoStatus/TaskUpdateRequest,
  conversions + round-trip tests), `revisioned_adapter.rs` (full),
  `store.rs` (load_revisioned_task_graph :676-749, compare_and_commit
  :755-885, set_task_status :953-983, test helpers), `executor.rs`
  (execute_runtime_plan :1622-1683, EkoRuntimeDagController :1147-1620,
  load_snapshot :1227-1253, select_ready_wave :1265-1281, block_task
  :1564-1580), `register.rs` (tool registration :45-130),
  `task_tools.rs` (capability policy), `file_shadow.rs` (:356-380 read_events).
- Executable evidence: 7 test invocations (V04-01..07), all green.

## Out Of Scope

- Framework executor runtime semantics (claims/waves/stalls/cancellation) —
  F-TSK-03 (consumed); EKO controller boundary — A-TSK-03 (consumed);
  claims/recovery/replay — A-TSK-04 (consumed); file authorities and crash
  consistency — A-TSK-01 (consumed; P3-01 of this task is the round-trip
  angle of its P2-01).
- Authoring tool semantics — A-TSK-02; worktree/finalize — A-TSK-05; review/
  artifacts — A-TSK-06; frontend ts-rs projections — A-FE-01/02.
- Plan as artifact / TodoItem as UI projection semantics — F-TSK-01/A-TSK-02
  (consumed).

## Inputs

- Root `AGENTS.md` (single task-relation API; TaskPlan artifact / TodoItem UI
  projection; no todo_write/plan_create/plan_patch/plan_execute; adapter
  thin-and-lossless gate; framework-vs-app layering; "already exists" gate).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (X-TSK-01 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read (all complete): `F-TSK-01.md`, `F-TSK-02.md`,
  `F-TSK-03.md`, `A-TSK-01.md`, `A-TSK-02.md`, `A-TSK-03.md`, `A-TSK-04.md`,
  `A-TSK-05.md`, `A-TSK-06.md`.
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md` M13
  (Phases 1-5) and the runtime-dag-kernel-convergence record — re-validated
  only through the dependency reports' V05 executions and the anchors
  re-verified in this task's V05.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correct) | `TaskSpec`/`TaskExecution`/`TaskStatus`/`TaskClaim`, `TaskRevisionService` + `TaskPatchEngine` + `TaskPlanPatchOp`, `PlanValidator`, `RuntimeDagExecutor` + `DagExecutionState`, `RevisionedTaskStore` trait + `InMemoryRevisionedTaskStore`, the `task_create`/`task_update`/`task_list` tools — sole authorities (V02). Legacy `TaskManager`/`TaskExecutor` remains a framework public option, production-unreachable (F-TSK-01-P3-01). |
| EKO product policy (application, correct) | `TaskRun`/`TaskPlan`/`PlanTask`/`TodoItem`/`TodoStatus` projections, `events.jsonl`+`plan.json`+`run-state.json` file layout, boot recovery, capability catalog, `select_ownership_safe_wave`, retry requeue, review gates, drain completion gate, Auto-mode launchers (`create_complex_task`/`check_run_status`/`cancel_run`). |
| Adapter boundary (thin, lossless on reachable paths) | `EkoRevisionedTaskStore` implements only `load`/`compare_and_commit` (revisioned_adapter.rs:36-56); conversions in `types.rs` and `store.rs` map 1:1 with EKO metadata inside `TaskSpec.metadata`; `EkoTaskToolPolicy` supplies schema extensions/scope/defaults/capability checks only. One defect: `load_revisioned_task_graph` silently defaults a missing execution to Pending (X-TSK-01-P3-01). |
| Duplicate search | Terms (both repos, V01-V03): `TaskSpec`, `TaskExecution`, `TaskStatus`, `TaskClaim`, `TaskPlan`, `PlanTask`, `TodoItem`, `TodoStatus`, `EkoTaskSpec`, `todo_write`, `plan_create`, `plan_patch`, `plan_execute`, `TaskPlanPatch`, `to_task_plan_patch`, `RuntimeDagExecutor::new`, `PlanValidator`, `validate_task_snapshot`, `ready_task_ids`, `get_ready_tasks`, `select_ready_wave`, `select_ownership_safe_wave`, `execute_ready_tasks`, `TaskManager::new`, `TaskExecutor::new`, `execute_all`, `set_task_status`, `worker`. Result: one live authority per concept; one tool family; zero forbidden CRUD; zero `worker` terms. |
| Migration deletion | No new deletion target from this task. P3-01's fix is an addition (error or rebuild instead of fabricated default) sharing A-TSK-01-P2-01's read-side rebuild; legacy `TaskManager`/`TaskExecutor`/`hooks`/`verifier` remain F-TSK-01-P3-01's deletion decision for S-RDM-01. |

## Current Path

Verified call graph (details in V01/V02):

1. Authoring: `task_create`/`task_update` (framework tools) or EKO
   `apply_eko_task_update`/`commit_eko_task_plan` -> `TaskRevisionService`
   (patch engine -> policy hooks -> `PlanValidator::validate_task_snapshot`
   at revisioned.rs:1007-1011) -> `compare_and_commit` CAS
   (EkoRevisionedTaskStore -> store.rs:755-885: revision check, terminal-run
   rejection, EKO metadata decode, `PlanRevisionCommitted` event, projection
   rebuild). Test-only EKO helpers use the same engine/validator
   (store.rs:96-101, :915-923, both `#[cfg(test)]`).
2. Execution: `task_execute` -> `execute_run` drain loop (completion gate
   only) -> `execute_runtime_plan` (executor.rs:1622-1683) -> the single
   production `RuntimeDagExecutor::new` (:1645). Each safe point reloads the
   snapshot via controller `load_snapshot` (:1227-1253, file-backed,
   refreshes the display cache) and re-validates with `PlanValidator`
   (runtime_executor.rs:214); frontier = `DagExecutionState::ready_task_ids`
   (runtime.rs:438); EKO filters the wave by writer ownership
   (select_ready_wave :1265-1281) — never recomputes readiness.
3. Status writes: claim-guarded `set_claimed_task_status`/
   `requeue_claimed_task`/`task_claim_is_current` (store.rs:986-1121) plus the
   unguarded `set_task_status` escape hatches (A-TSK-04-P2-01: block_task and
   finalizers).
4. Read-back: `load_revisioned_task_graph` (store.rs:676-749) maps
   plan.json + run-state.json to the framework graph; `EkoTaskExecution
   .status` carries the framework `TaskStatus` verbatim, so the projection is
   lossless in the steady state (V01; `rebuild_reflects_task_patch` and
   `plan_task_round_trips_through_framework_task` passed).

## Findings

### X-TSK-01-P3-01: Read-back silently fabricates a Pending execution for plan tasks missing from run-state.json — a lossless-round-trip violation in the projection-divergence window

- Priority: P3
- Confidence: medium (mechanism certain; trigger requires the crash window)
- Layer: adapter (EKO read-back conversion)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:694-696`
  (`let execution = executions.remove(&spec.id).unwrap_or_else(|| EkoTaskExecution::pending(spec.id.clone()));`
  inside `load_revisioned_task_graph`); write side rewrites `plan.json` then
  `run-state.json` as two atomic renames (`file_shadow.rs:208-280`), so a
  crash between the renames leaves plan.json new / run-state.json old;
  `load_revisioned_task_graph` requires run-state.json to exist
  (store.rs:685-686) but does not check per-task coverage.
- Reachability: crash between the two projection renames (or any manual
  divergence) -> a task present in the new plan.json but absent from the old
  run-state.json reads back with a fabricated `Pending` execution and no
  claim; the framework executor then treats it as a fresh Pending task. In
  the consistent steady state the maps always agree (both files are rebuilt
  from the same event stream at every write), so this path is live only
  inside the A-TSK-01-P2-01 window. Not covered by any existing test
  (the round-trip tests exercise consistent snapshots only).
- Expected invariant: the documented "lossless conversion" is total on the
  read path — a missing execution record must either be rebuilt from
  `events.jsonl` (the declared recovery authority) or surface an error,
  never be silently invented.
- Observed behavior: `unwrap_or_else(pending)` invents a Pending execution;
  a task that the event log says was Completed (or Running with a claim) is
  handed to the framework executor as Pending and can be re-dispatched.
- Impact: inside the (small) crash-divergence window, a task can be
  re-executed even though its side effects already happened — a silent
  side-effect replay vector; more broadly, the read side is not lossless for
  partial states while the write side is, so "lossless round trip" holds
  only in the consistent steady state.
- Root cause: `load_revisioned_task_graph` trusts the projections' mutual
  consistency instead of the event log (the A-TSK-01-P2-01 root cause); the
  `unwrap_or_else` default was written as a convenience that turns the
  inconsistency into fabricated data.
- Direction: in `load_revisioned_task_graph`, when a plan task has no
  run-state.json execution, rebuild the projection from events (shared fix
  with A-TSK-01-P2-01) or return an explicit `StoreError` for that run;
  replace the silent default with an error path at minimum. No deletion.
- Regression validation: fixture "plan.json contains task T, run-state.json
  predates T, events.jsonl says T Completed -> read-back either restores
  Completed via rebuild or errors; never Pending"; Q-FLT-02 crash fixture
  between the two renames.
- Validation reports: [V01-01](../validations/X-TSK-01/V01-01.md),
  [V05-01](../validations/X-TSK-01/V05-01.md)

No P0/P1/P2 new findings. All established findings in the area were
re-anchored as current (V05): F-TSK-02-P1-01 (skip stalls), F-TSK-02-P2-01
(string-literal blocker), F-TSK-02-P2-02 (sequential mode inert),
F-TSK-03-P2-01 / A-TSK-03-P2-01 (wave-abort orphans), A-TSK-03-P1-01 /
A-TSK-04-P1-01 (pause-in-wave -> cancel), A-TSK-01-P1-01 (torn tail),
A-TSK-01-P2-01 (read path never rebuilds), A-TSK-01-P3-01 (Retrying/Paused
latent loss), A-TSK-04-P2-01 (unguarded set_task_status), F-TSK-01-P3-01
(legacy surface). These are referenced by canonical ID and NOT re-filed.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Field-by-field round trip (framework Task types <-> EKO task runtime types) | yes | passed (1 P3 finding) | [V01-01](../validations/X-TSK-01/V01-01.md) |
| V02 | Authority call graph (validator/executor/store each one; legacy reachability) | yes | passed | [V02-01](../validations/X-TSK-01/V02-01.md) |
| V03 | Forbidden CRUD search (todo_write/plan_create/plan_patch/plan_execute) | yes | passed | [V03-01](../validations/X-TSK-01/V03-01.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked tasks::revisioned` (framework shared fixture) | yes | passed (exit 0, 3 ok) | [V04-01](../validations/X-TSK-01/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked --lib plan_task_round_trips_through_framework_task` | yes | passed (exit 0, 1 ok) | [V04-02](../validations/X-TSK-01/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --locked --lib task_update_inserts_task_and_commits_one_revision` (EKO shared fixture) | yes | passed (exit 0, 1 ok) | [V04-03](../validations/X-TSK-01/V04-03.md) |
| V04 | `cargo test -p echo-agent-app-core --locked --lib rebuild_reflects_task_patch` | yes | passed (exit 0, 1 ok) | [V04-04](../validations/X-TSK-01/V04-04.md) |
| V04 | `cargo test -p echo-agent-app-core --locked --lib file_path_rejects_dependency_cycle_and_appends_no_event` | yes | passed (exit 0, 1 ok) | [V04-05](../validations/X-TSK-01/V04-05.md) |
| V04 | `cargo test -p echo-agent-app-core --locked --lib invalid_cycle_is_rejected_before_scheduler_dispatch` | yes | passed (exit 0, 1 ok) | [V04-06](../validations/X-TSK-01/V04-06.md) |
| V04 | `cargo test -p echo-agent-app-core --locked --lib run_completion_gate_requires_durable_structured_result` | yes | passed (exit 0, 1 ok) | [V04-07](../validations/X-TSK-01/V04-07.md) |
| V05 | Cross-reference with established findings (canonical IDs; anchor re-verification) | yes | passed | [V05-01](../validations/X-TSK-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN M13 P1: `RuntimeDagExecutor` sole execution loop; frontier/wave/cancel/stall removed from the app layer | current | single production construction executor.rs:1645; EKO drain loop is a completion gate (V02-01) |
| MASTER-PLAN M13 P2: `PlanValidator` sole structural authority; EKO duplicate validator deleted | current | revisioned.rs:1007-1011 + runtime_executor.rs:214 only production sites; store.rs:96-101/921-923 `#[cfg(test)]` (V02-01) |
| MASTER-PLAN M13 P5: single task-relation API; `todo_write`/`plan_*` CRUD deleted; TaskPlan artifact / TodoItem UI projection | current | zero forbidden definitions (V03-01); one tool family (register.rs:170-178) |
| AGENTS.md: adapter must be thin and conversion lossless; no second patch engine/validator/frontier | current (with the P3-01 read-side caveat) | revisioned_adapter.rs:36-56; V01-01; X-TSK-01-P3-01 |
| A-TSK-01 (events.jsonl authority; thin lossless adapter; crash gaps P1-01/P2-01) | current | re-anchored (V05-01); P2-01 window is the P3-01 trigger |
| F-TSK-02 P1-01/P2-01/P2-02, F-TSK-03 P2-01/P2-02, A-TSK-03 P1-01/P2-01, A-TSK-04 P1-01..03/P2-01, F-TSK-01-P3-01 | current (independent re-anchoring) | V05-01 anchors unchanged |

## Coverage And Uncertainty

- All conclusions are static except the 7 V04 test runs; no live LLM DAG run
  was executed (read-only review). The shared-fixture claim rests on running
  the same Insert operation shape through the framework in-memory service
  (V04-01) and through the EKO file-backed engine + commit adapter (V04-03),
  plus the field round trip (V04-02) and event rebuild (V04-04).
- X-TSK-01-P3-01 is a deterministic code trace, not dynamically reproduced
  (the plan/run-state divergence requires a crash between two renames);
  Q-FLT-02 can pin it with a torn-projection fixture.
- Framework executor/controller semantics beyond the boundary were consumed
  from F-TSK-03/A-TSK-03/A-TSK-04 reports rather than re-executed; their
  V04 evidence stands.
- Frontend (ts-rs) projections and TodoItem rendering are A-FE-01/02 scope,
  not verified here.
- `EkoTaskExecution.status` stores the framework `TaskStatus` verbatim, so
  the file projection itself is lossless even where `TodoStatus` is a
  smaller enum; the A-TSK-01-P3-01 latent gap (Retrying/Paused) has zero
  live producers today (V01-01 producer grep).

## Handoff

- Conclusions downstream tasks may rely on: one revisioned TaskRun graph
  with one validator, one executor, and one store CAS boundary on every live
  path (V02); the EKO projection is field-lossless and status-lossless for
  the reachable state space (V01, V04-02..04); zero forbidden CRUD (V03);
  the shared DAG fixture behaves identically through the framework and EKO
  adapters (V04-01/03); the one new gap is the fabricated-Pending read-back
  default (P3-01, fix shared with A-TSK-01-P2-01); all nine established
  findings remain current (V05).
- Reports to read: the 10 validation reports above; dependency reports
  F-TSK-01..03 and A-TSK-01..06 (deep reads and their validation matrices).
- Stale conditions: this report becomes stale if `revisioned.rs` patch/CAS
  semantics, `runtime.rs` model/frontier, `runtime_executor.rs` loop,
  EKO `store.rs` load/commit/status paths, `types.rs` conversions, or
  `revisioned_adapter.rs` change; also if a production caller of
  `TaskManager`/`TaskExecutor` appears (F-TSK-01-P3-01 changes from "legacy"
  to "live second authority"), if a read-side rebuild lands (P3-01 and
  A-TSK-01-P2-01 fixed), or if `Retrying`/`Paused` gain a live producer
  (A-TSK-01-P3-01 becomes active loss).
- Follow-up task IDs: Q-FLT-02 (plan/run-state divergence + torn-tail crash
  fixtures), X-STA-01 (identity continuity across restart), X-EVT-01 (event
  conformance across surfaces), S-RDM-01 (P3-01 fix bundled with
  A-TSK-01-P2-01; F-TSK-01-P3-01 legacy-surface deletion decision).
