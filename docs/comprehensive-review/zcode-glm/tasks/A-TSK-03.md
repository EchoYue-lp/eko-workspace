# A-TSK-03: Task execution controller boundary

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Does EKO inject only product policy into `RuntimeDagExecutor`, with no
second ready-frontier, retry, cancellation, or stall loop?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs`
  (6272 lines) — read in relevant slices:
  - module doc + imports (1-46) — declares the layering contract;
  - `execute_run` (321-585) — the outer drain loop and outcome handling;
  - `finalize_cancelled_run_state` (643-661) — the orphan reconciliation
    sweep that mitigates F-TSK-03-P2-02;
  - `CompletionAssessment` / `assess_task_execution` (668-765) — the
    execution-evidence gate (NOT a scheduling authority);
  - `TaskDispatcher` trait + `RealTaskDispatcher` (793-1030) — the
    production dispatch/integration surface;
  - `select_ownership_safe_wave` (1127-1145) — product-policy wave
    narrowing;
  - `EkoRuntimeDagController` (1147-1620) — the central artifact: the
    framework's `RuntimeDagController` impl, including all eight
    callbacks (load_snapshot, claim_task, select_ready_wave,
    dispatch_task, resolve_dispatch, block_task, failed_task_disposition,
    interruption_outcome, note_stalled);
  - `execute_runtime_plan` (1623-1683) — the single site that constructs
    `RuntimeDagExecutor` and invokes `execute`;
  - `integrate_reviewed_task` (1686-1754) — the worktree-merge step;
  - `run_review_gate` / `ReviewGateOutcome` (1756-1842) — the
    reviewer-LLM gate;
  - `execute_task` (1843-2509) — the per-task Subagent pipeline;
  - 46-test `tests` module (4031-6272), with particular attention to:
    `runtime_plan_respects_dependency_order` (5524),
    `runtime_plan_applies_inserted_revision_after_active_wave` (5564),
    `runtime_plan_failure_propagates_and_blocks_downstream` (5638),
    `runtime_plan_cancellation_propagates_to_cancelled_outcome` (5727),
    `runtime_plan_cancellation_preserves_explicit_pause` (5758),
    `invalid_cycle_is_rejected_before_scheduler_dispatch` (5785),
    `runtime_plan_does_not_redispatch_in_flight_running_tasks` (5824),
    `real_execution_failure_retries_within_budget` (6125),
    `wave_processes_all_results_when_one_task_blocks` (6183).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/task_execute_tool.rs`
  (966 lines) — read in full: confirmed it is a per-run serialization
  shell + `execute_run` awaiter; no scheduling primitives.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs`
  — relevant slices:
  - `claim_task` (986-1029) — optimistic-concurrency claim with
    `expected_revision` guard and `attempt = retry_count + 1`;
  - `set_claimed_task_status` (1032-1062) — CAS on Running + claim
    identity;
  - `requeue_claimed_task` (1066-1105) — atomic Running→Pending flip +
    retry_count bump (the retry mechanism);
  - `complete_run_if_quiescent` (453-488) — the run-level CAS.
- Framework contract:
  `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:80-234`
  — `RuntimeDagController` trait + `RuntimeDagExecutor::execute`
  safe-point loop top.
- Cross-repo duplicate search (V01/V02) for `RuntimeDagExecutor`,
  `RuntimeDagController`, `execute_run`, `JoinSet`, `ready_task_ids`,
  `validate_selected_wave`, `note_stalled`, `PlanValidator` across the
  whole `echo-agent-cli` repository.

## Out Of Scope

Deferred to downstream tasks:

- **A-TSK-04**: claims, revisions, recovery, terminal monotonicity,
  resume correctness (the post-`Paused`/`Cancelled` reconciliation
  sweeps' long-term soundness; the CAS mechanics of
  `complete_run_if_quiescent` under concurrent plan patches; the
  pre-flight cycle rejection's parity with the framework
  `PlanValidator`).
- **A-TSK-02** already established the task authoring tool surface;
  this task consumes its conclusion that `task_execute` is the single
  EKO extension over the framework's `task_create/update/list`.
- The internal mechanics of `execute_task` (per-Subagent dispatch,
  isolation/worktree setup, trace streaming) — these are the dispatch
  pipeline behind `dispatch_task`, audited only as the seam the
  controller hands off to.
- Framework-side kernel gaps F-TSK-03-P2-01 (in-flight stall) and
  F-TSK-03-P2-02 (abort-orphan) — owned by F-TSK-03's follow-ups. This
  task only establishes that EKO does not re-implement them and that
  EKO's reconciliation sweeps mitigate P2-02 at the application layer.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — rule 6 (single task-relationship authority);
    the framework-vs-application layering gate; the "先查是不是已经有了"
    pre-implementation gate; the "adapter must stay thin" rule
    ("adapter 不得重新拥有 ready frontier、DAG 主循环、通用重试/取消、
    死锁判断"); the "delete over retain" cleanup rule; UTF-8 / panic
    safety.
  - `docs/comprehensive-review/REPORTING.md`,
    `docs/comprehensive-review/templates/{task-report,validation-report}.md`.
- Dependency task reports read:
  - **F-TSK-03** (complete) — established `RuntimeDagExecutor::execute`
    as the single production authority for safe points, bounded waves,
    claims, cancellation, failure propagation, and (partial) stall
    detection; the `ManagedTaskDagController` reference adapter is
    thin; retry is controller-owned. F-TSK-03-P2-02's handoff item 5
    ("downstream tasks must verify that the application adapter
    reconciles orphaned Running tasks on Cancelled") is answered here.
  - **F-TSK-01** (complete) — the canonical framework task model and
    `RevisionedTaskStore` / `TaskToolPolicy` contracts this controller
    threads through.
  - **F-TSK-02** (complete) — `PlanValidator` is the sole structural
    DAG validator. This task verifies EKO does not introduce a second
    one.
  - **A-TSK-01** (complete) — file authority and the lossy
    `TodoStatus` round-trip (A-TSK-01-P2-02). This task resolves
    A-TSK-01-P2-02's open question: does the executor persist
    framework `Retrying` / `Paused` through the lossy path?
- Historical documents treated as hypotheses: the executor.rs module
  doc (1-35), the task_execute_tool.rs spec doc (1-19), and the
  store.rs `claim_task` / `requeue_claimed_task` doc comments — all
  verified below.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Confirmed framework-owned: safe-point loop, ready-frontier computation, bounded-wave dispatch, optimistic-concurrency claims, cancellation-grace drain, failure propagation, structural DAG validation, and (partial) stall detection. All live in `echo-orchestration::tasks::RuntimeDagExecutor` (echo-agent `runtime_executor.rs:196-447`) and the kernel's `DagExecutionState` (echo-agent `runtime.rs:340-501`). EKO touches them only through the trait callbacks. |
| EKO product policy | Confirmed app-owned: write/shell/LLM semaphores (`EkoExecutionLimits` 50-66, only `max_concurrent_subagents` is forwarded to the kernel), per-file write locks (`file_write_locks` 1154), file-ownership wave narrowing (`select_ownership_safe_wave` 1127-1145), retry budget (`task.max_retries` checked in `resolve_dispatch` 1396-1432), review gate (`run_review_gate` 1773), worktree integration (`integrate_reviewed_task` 1686), attended-vs-unattended disposition (`review_stop_disposition` 1174-1186), durable-result restart short-circuit (`recoverable_subagent_result`), and the post-outcome orphan reconciliation sweeps (`finalize_cancelled_run_state` 643-661; Paused-state cleanup 546-559). |
| Adapter boundary | `EkoRuntimeDagController<W>` (1147-1620) is thin and faithful to the framework contract: every callback is persistence, dispatch, or product policy. It does NOT recompute the frontier (the kernel passes `ready_task_ids` in and EKO only filters them by ownership), does NOT spawn its own wave (the kernel spawns and joins), does NOT drive its own cancellation abort (the kernel drains; EKO only reconciles the leftovers afterward), does NOT validate DAG structure (the framework `PlanValidator` does at runtime_executor.rs:214), and does NOT loop on retry (one-shot `Pending` resolution that relies on the kernel's re-claim). |
| Duplicate search | Searched names (whole `echo-agent-cli` repo): `RuntimeDagExecutor`, `RuntimeDagController`, `RuntimePlanSnapshot`, `RuntimeTaskClaimOutcome`, `RuntimeTaskResolution`, `RuntimeStopDisposition`, `RuntimeDagOutcome`, `TaskClaim`, `TaskSubagentContext`, `DagExecutionState`, `ready_task_ids`, `validate_selected_wave`, `note_stalled`, `PlanValidator`, `execute_run`, `execute_runtime_plan`. Result: ONE `RuntimeDagExecutor` construction (executor.rs:1645); ONE `RuntimeDagController` impl (executor.rs:1222-1620); ONE production `execute_run` entry with five callers (service.rs:418, task_execute_tool.rs:401, tui/events.rs:4737, tauri/commands/task_runtime.rs:262 and 364) — all funnel through `execute_runtime_plan`. ZERO `JoinSet`/`validate_selected_wave`/`subagent_semaphore`/`PlanValidator` constructions in `echo-agent-cli`. ZERO structural DAG validators. V01, V02. |
| Migration deletion | No migration proposed. One P3 cleanup recommendation (A-TSK-03-P3-01) for a missing run-state guard in the drain loop; the fix is localized. |

## Current Path

Verified EKO→framework execution boundary at commits
`echo-agent` 9b0e0fa / `echo-agent-cli` b3b2e81:

```text
External entrypoint (UI/TUI/CLI/tool)
   │
   ├── service.rs:418                (interactive chat path)
   ├── task_execute_tool.rs:401      (model invokes task_execute)
   ├── tui/events.rs:4737            (TUI resume)
   └── tauri/commands/task_runtime.rs:262, 364  (GUI resume / drive)
       │
       ▼
execute_run(store, agent, reviewer_llm, ..., run_id, parent_cancel, ...)
   [executor.rs:321-585]
   │
   │  outer drain loop (367-434):
   │    ┌─ unresolved_count = 0?  → run_completion_blockers + CAS via
   │    │                            complete_run_if_quiescent  [store.rs:453]
   │    │
   │    └─ unresolved_count > 0?  → execute_runtime_plan(...)  [407-417]
   │
   ▼
execute_runtime_plan(store, dispatcher, reviewer_llm, run_id, limits, cancel, sink)
   [executor.rs:1623-1683]
   │
   ├─ controller = Arc::new(EkoRuntimeDagController { ... })      [:1633]
   │      write_sem/shell_sem/llm_sem/file_write_locks/trace_sink/cancel
   │
   ├─ executor = RuntimeDagExecutor::new(controller, config)      [:1645]
   │      config.max_concurrent_subagents = limits.max_concurrent_subagents
   │      (other limits stay in EKO and are applied inside dispatch_task)
   │
   └─ executor.execute(run_id, parent_cancel).await               [:1652]
         │
         ▼
      framework safe-point loop  [echo-agent runtime_executor.rs:206-447]
         │  (kernel-owned: load_snapshot → validate → ready → wave →
         │   claim → spawn → drain → resolve → repeat)
         │
         ├─ controller.load_snapshot(run_id)        [executor.rs:1227]
         │     store.get_plan → RuntimePlanSnapshot + plan_tasks cache
         │
         ├─ PlanValidator.validate_task_snapshot    [echo-agent :214]
         │
         ├─ state.ready_task_ids                   [echo-agent :275]
         │
         ├─ controller.select_ready_wave(tasks, ready_ids)  [:1265]
         │     select_ownership_safe_wave → ≥1 id subset
         │
         ├─ validate_selected_wave                 [echo-agent :319]
         │
         ├─ join_set.spawn {
         │     semaphore.acquire_owned().await?    [echo-agent :349]
         │     controller.claim_task(...)          [executor.rs:1254]
         │       → store.claim_task (CAS + attempt++)  [store.rs:986]
         │     controller.dispatch_task(ctx, claim, task)  [executor.rs:1284]
         │       → dispatcher.dispatch(...) (RealTaskDispatcher)
         │           → execute_task(...)           [executor.rs:841-887]
         │               (per-task subagent pipeline, write/shell/llm
         │                semaphores, worktree setup, trace streaming)
         │   }
         │
         ├─ join_set drain + cancellation grace    [echo-agent :372-413]
         │
         └─ for (task, claim, dispatch) in wave_results:
              controller.resolve_dispatch(...)    [executor.rs:1348]
                │
                ├─ ExecutionFailed + retry budget:
                │     store.requeue_claimed_task (Pending + retry++)
                │     return RuntimeTaskResolution::Pending
                │     → kernel re-iterates, re-claims on next safe point
                │
                ├─ AcceptancePending or ReviewGate fail:
                │     set_claimed_task_status(Blocked)
                │     return RuntimeTaskResolution::Blocked{disposition}
                │     → kernel stops with Paused/Fail per disposition
                │
                └─ Executed + ReviewGate pass:
                      integrate_reviewed_task (worktree merge for writers)
                      set_claimed_task_status(Completed)
                      return RuntimeTaskResolution::Completed

   execute_runtime_plan returns RunOutcome
   │
   ▼
execute_run outcome branch (437-583):
   Completed  → finalize + memory write
   Failed     → transition_run(Failed) + note
   Cancelled  → finalize_cancelled_run_state  [643-661]
                  (Pending|Running|Blocked → Cancelled)
                  transition_run(Cancelled)
   Paused     → Running → Pending sweep  [546-559]
                  (so resume re-dispatches)
                  note + save_trace
   Err        → transition_run(Failed)
```

Invariants verified by this graph (full evidence in V01-V04):

- **Single kernel construction.** `RuntimeDagExecutor::new` is called
  exactly once in echo-agent-cli, at executor.rs:1645. The kernel's
  `execute` is invoked exactly once per `execute_runtime_plan` call
  (executor.rs:1652). No code in `echo-agent-cli` constructs a second
  kernel, drives a `JoinSet`, or computes a frontier outside the
  callback. V01.
- **Single controller impl.** `impl RuntimeDagController for
  EkoRuntimeDagController<W>` at executor.rs:1222-1620 is the only
  application-side implementation. The framework's in-tree reference
  adapter (`ManagedTaskDagController`, echo-agent
  `executor.rs:1609`) is not used by EKO. V01.
- **Single entry surface.** All five callers of `execute_run` route
  through the same `execute_run → execute_runtime_plan → kernel`
  chain. None bypasses it. V01.
- **No scheduling-loop duplicate.** The outer drain loop
  (executor.rs:367-434) only checks quiescence and re-invokes the
  kernel; it never computes the frontier, spawns a wave, or claims a
  task. V01, V02.
- **No second DAG validator.** Zero `PlanValidator` / cycle / topology
  constructions in `echo-agent-cli`. The framework validator at
  echo-agent `runtime_executor.rs:178, 214` is the sole runtime
  authority. (A pre-flight cycle check at `attach_plan_for_test` is
  defense-in-depth at commit time, not a runtime duplicate.) V02.
- **No retry loop in EKO.** Retry is a one-shot `Pending` resolution
  in `resolve_dispatch` that flips Running→Pending via
  `requeue_claimed_task` (store.rs:1066) and returns
  `RuntimeTaskResolution::Pending`. The kernel's next safe point
  re-claims the task with `attempt = retry_count + 1`. V02.
- **No second stall detector.** EKO's `note_stalled` callback
  (executor.rs:1615) only records the reason; no timer, no abort. V02.
- **No second cancellation drain.** EKO's
  `finalize_cancelled_run_state` (executor.rs:643-661) is a one-shot
  sweep that runs AFTER the kernel returns `Cancelled`. The kernel's
  wave-drain (`select!` + grace + abort) remains the only in-flight
  cancellation mechanism. V02, V03.
- **Orphan reconciliation mitigates F-TSK-03-P2-02.** The
  framework-level abort-orphan gap is closed at the application layer:
  the post-cancel sweep flips every `Pending|Running|Blocked` task to
  `Cancelled`, and the post-pause sweep flips `Running` back to
  `Pending` for resume. V03.
- **A-TSK-01-P2-02 resolved.** EKO never produces framework
  `TaskStatus::Retrying` or `TaskStatus::Paused`. Retry is expressed
  as `TodoStatus::Pending` + `retry_count`, and pause as
  `TodoStatus::Blocked` + a run-level `TaskRunStatus::Paused`. The
  lossy `TodoStatus` round-trip A-TSK-01-P2-02 flagged therefore does
  not lose data on EKO's executor path — the lossy statuses are
  never generated. The doc string at types.rs:917-920 should be
  narrowed to match. V02, V03.

## Findings

The headline result is strongly positive: EKO injects only product
policy into `RuntimeDagExecutor`, with no second ready-frontier, retry
loop, cancellation drain, or stall detector. The eight controller
callbacks and the two post-outcome sweeps are exactly the persistence,
dispatch, and product-policy layer the framework expects. One P3
cleanup recommendation is recorded; one P3 informational note resolves
A-TSK-01-P2-02's open question.

### A-TSK-03-P3-01: The `execute_run` drain loop does not guard against the run transitioning to a non-Running terminal state during the quiescent-completion window

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:367-398`
    — the drain loop. The quiescent branch reads `unresolved_count == 0`,
    calls `run_completion_blockers`, and then
    `store.complete_run_if_quiescent(run_id)?`. If that returns `Ok(false)`
    (run is no longer `Running`), the loop does `drain_cycle += 1;
    continue;` — there is no check of `parent_cancel.is_cancelled()` and
    no check of the run's current status.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:453-488`
    — `complete_run_if_quiescent` returns `Ok(false)` for any run whose
    status is not `Running` and not `Completed` (i.e., `Cancelled`,
    `Paused`, `Failed`). It returns `Ok(true)` without doing anything if
    the run is already `Completed`.
  - The kernel returns `RuntimeDagOutcome::Completed` only when
    `state.all_completed` (echo-agent `runtime_executor.rs:271`). On the
    EKO side, that becomes `RunOutcome::Completed`, which the drain loop
    re-iterates to catch racing plan patches (executor.rs:420-431). If
    the user (or a cron timeout, or an external `cancel_run`/`pause_run`)
    transitions the run out of `Running` in the window between the
    kernel's Completed and `complete_run_if_quiescent`, the CAS returns
    false and the loop spins.
- Reachability: requires the run to reach all-tasks-completed inside the
  kernel, and an external transition of the run to `Cancelled` or
  `Paused` to land in the microseconds-wide window between the kernel
  returning and the CAS running. No test exercises this. In practice the
  window is tiny and there is no normal reason for a user to cancel a run
  that is finishing successfully, but a cron-timeout or a queued pause
  can race it.
- Expected invariant: the drain loop should terminate for any input and
  any concurrent run-state transition.
- Observed behavior: the drain loop spins forever on this narrow race,
  consuming CPU. The cancel token is not checked in the quiescent branch
  (it would be checked only inside the next `execute_runtime_plan`
  call, which never happens because `unresolved_count == 0`).
- Impact: low. The probability is small and the failure mode is a hung
  drain task that the user must kill. For EKO's local-assistant threat
  model this is a robustness defect, not a correctness or safety issue.
  No data loss; no orphaned tasks (the run is already past completion).
- Root cause: the quiescent branch was written for the appended-tasks
  race (executor.rs:420-431 documents that case) and did not consider
  concurrent run-state transitions.
- Direction: at the top of the drain loop (or inside the quiescent
  branch), add a guard: if `parent_cancel.is_cancelled()` or the run's
  status is no longer `Running` (and not `Completed`), break with the
  appropriate outcome. A two-line check suffices. Alternatively, have
  `complete_run_if_quiescent` return a tri-state (`Completed |
  StillRunning | Aborted`) and let the loop break on `Aborted`.
- Regression validation: a test that seeds an all-completed run, flips
  the run to `Cancelled` between kernel return and CAS, and asserts the
  drain loop terminates within a bounded time.
- Validation reports: [V01-01](../validations/A-TSK-03/V01-01.md)

### A-TSK-03-P3-02: `Retrying` / `Paused` `TaskStatus` are never produced on the EKO executor path — A-TSK-01-P2-02's lossiness is latent, not live

- Priority: P3
- Confidence: high
- Layer: adapter
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:1396-1432`
    — on `ExecutionFailed`, the controller calls
    `store.requeue_claimed_task` and returns
    `RuntimeTaskResolution::Pending`. The store writes
    `"status": TodoStatus::Pending.as_str()` (store.rs:1090-1100) and
    bumps `retry_count`. EKO never sets the framework
    `TaskStatus::Retrying` on a task.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:1502-1516`
    — on `AcceptancePending` or a `ReviewGate` failure, the controller
    calls `set_claimed_task_status(..., TodoStatus::Blocked, ...)` and
    returns `RuntimeTaskResolution::Blocked`. The framework
    `TaskStatus::Paused` is never written by EKO; the run-level
    `TaskRunStatus::Paused` is set only by `transition_run` after the
    kernel returns `Paused` (executor.rs:1662-1664).
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1018-1026, 1051-1059, 1085-1101`
    — every status event written by `claim_task` /
    `set_claimed_task_status` / `requeue_claimed_task` carries a
    `TodoStatus`. The framework `TaskStatus` only appears as
    `EkoTaskExecution.status` in the in-memory `run-state.json`
    projection (`types.rs:924`), which `rewrite_plan` regenerates from
    the lossy event stream.
- Reachability: the lossiness in A-TSK-01-P2-02 requires the executor to
  persist a `Retrying` or `Paused` framework status. EKO's executor
  never does. Therefore the lossiness cannot surface through the
  executor→store path audited here. It could only surface if a future
  commit introduces a `set_claimed_task_status`-style call that writes
  the framework status directly, or if the framework kernel itself
  starts persisting `Retrying` outside the controller callbacks (it does
  not today).
- Expected invariant: A-TSK-01-P2-02's invariant was "the authority must
  reproduce every framework task state, OR the lossiness must be
  documented as a deliberate projection boundary." This finding
  establishes the second clause: the lossiness is a deliberate boundary,
  because EKO's product policy never uses the lossy statuses.
- Observed behavior: every status the executor writes is one of
  `Pending | Running | Blocked | Completed | Failed | Cancelled |
  TimedOut | Skipped` — all of which round-trip losslessly through
  `TodoStatus`. The framework's `Retrying` is represented as
  `Pending` + `retry_count`, and the framework's `Paused` is represented
  as `Blocked` + (run-level) `Paused`. These representations are
  sufficient for EKO's retry-budget and review-gate flows.
- Impact: low. A-TSK-01-P2-02's "state fidelity" concern does not apply
  on the current executor path. The residual risk is documentation drift:
  the doc string at types.rs:917-920 ("shared `TaskStatus` remains
  authoritative and lossless") overstates the guarantee.
- Root cause: the executor was written against `TodoStatus` from the
  start, with retry-as-Pending and pause-as-Blocked as the product
  vocabulary. The framework `TaskStatus`'s richer variants never had a
  writer on this path.
- Direction:
  1. Narrow the doc string at `types.rs:917-920` to reflect that
     `Retrying`/`Paused` are never produced on the executor path; the
     lossiness is a deliberate projection boundary, not a hazard.
  2. Optionally, add a regression test that drives a task through the
     retry path and asserts the persisted event stream + the reloaded
     `EkoTaskExecution.status` round-trip the `Pending` + retry_count
     representation losslessly.
  3. If a future commit introduces a writer of framework `Retrying`/
     `Paused` on this path, A-TSK-01-P2-02's recommended fix (persist
     the framework status in the event payload) becomes mandatory.
- Regression validation: a test that drives `real_execution_failure_retries_within_budget`'s
  scenario (executor.rs:6125), drops in-memory state after the retry,
  and asserts the reloaded task carries the correct `retry_count` and
  `Pending` status.
- Validation reports: [V02-01](../validations/A-TSK-03/V02-01.md),
  [V03-01](../validations/A-TSK-03/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Framework/application ownership call graph: single kernel construction, single controller impl, single execute_run entry surface, drain loop is not a scheduler | yes | passed | [V01-01](../validations/A-TSK-03/V01-01.md) |
| V02 | Scheduling-loop duplicate search: no frontier / wave / stall / validator / cancel-drain in EKO; retry is one-shot Pending resolution | yes | passed | [V02-01](../validations/A-TSK-03/V02-01.md) |
| V03 | Controller callback inventory: every callback is persistence / dispatch / product policy; orphan reconciliation mitigates F-TSK-03-P2-02 | yes | passed | [V03-01](../validations/A-TSK-03/V03-01.md) |
| V04 | Basic DAG execution: 46-test executor suite passes; dependency / cancel / retry / revision / in-flight scenarios verified | yes | passed | [V04-01](../validations/A-TSK-03/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — three code/module comments treated as hypotheses; classifications inline in Historical Claim Status) | passed | classified inline |

Executed cargo commands (all exit 0):

```text
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib task_runtime::executor
  → 46 passed; 0 failed; 0 ignored (1.22s)
```

The full `echo-agent-cli` pre-commit gate was not re-run because this
review is read-only; the targeted executor subset is the directly
relevant evidence and is the suite that exercises the controller
integration boundary.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `executor.rs:1-7` module doc: "Converts EKO `TaskPlan` snapshots into the framework's product-neutral task view, then injects EKO dispatch, review, persistence, worktree, and event policy. Dependency traversal, revision safe points, Subagent waves, cancellation, failure propagation, and stall detection live in `echo_orchestration::tasks::RuntimeDagExecutor`." | current | Fully corroborated by V01/V02/V03: the eight callbacks inject only product policy; no frontier/wave/cancel/stall/validator authority in EKO. |
| `executor.rs:18` module doc: "the overall Subagent count is capped by the framework executor; EKO owns write, shell, and LLM resource policy separately." | current | Confirmed by V01: only `max_concurrent_subagents` is forwarded to the kernel config (executor.rs:1647-1651); write/shell/LLM semaphores stay in `EkoRuntimeDagController` (executor.rs:1151-1153, 1637-1639) and are applied inside `dispatch_task`. |
| `executor.rs:20-24` module doc: "Cancellation: each dispatched task gets a child of the parent run's CancellationToken... Cancelling the run therefore cancels every in-flight task." | current-with-caveat | True for tasks the kernel actually dispatches. For tasks claimed but aborted by the kernel's grace-drain (F-TSK-03-P2-02), EKO reconciles them post-outcome via `finalize_cancelled_run_state` (V03). The framework-level orphan gap is mitigated, not closed (F-TSK-03-P2-02 remains for other consumers). |
| `task_execute_tool.rs:13-15` spec doc: "**§10.1**: `execute` 必须 `.await` `execute_run` 返回的 `RunOutcome`, 不得 fire-and-forget." | current | Confirmed by V01: the tool `await`s `execute_run` at task_execute_tool.rs:401 and translates the outcome. |
| `task_execute_tool.rs:16-17` spec doc: "**§10.2**: 本工具只注册在主 agent, subagent 绝不注册." | current (not re-verified) | Out of scope for this task (registration surface is A-TSK-02's territory). The doc claim is consistent with the single-controller/single-entry finding. |
| `store.rs:1064-1066` doc on `requeue_claimed_task`: "Atomically requeue one failed claimed attempt and advance its retry counter without exposing an unclaimed Pending window." | current | Confirmed by V02: the function acquires `plan_locks`, appends one event with `status=Pending, retry_count+1, claim=null`, and rewrites the projection under the same lock. No unclaimed window. |
| A-TSK-01 handoff: "A-TSK-03 owns the open question of how the framework executor's `TaskStatus` reaches the store; its conclusion determines whether P2-02 is live data loss or latent." | resolved (latent) | A-TSK-03-P3-02: EKO never produces `Retrying`/`Paused` on the executor path; A-TSK-01-P2-02 is latent, not live. |
| F-TSK-03 handoff item 1: "`RuntimeDagExecutor` is the single production authority for DAG execution." | current (corroborated for EKO) | V01 confirms EKO has one `RuntimeDagExecutor::new` (executor.rs:1645) and one `RuntimeDagController` impl; the kernel drives the loop. |
| F-TSK-03 handoff item 5: "Cancellation can orphan claims. Downstream tasks (especially A-TSK-04) must verify that the application adapter reconciles orphaned `Running` tasks on `Cancelled`." | current (verified for EKO) | V03 confirms `finalize_cancelled_run_state` (executor.rs:643-661) reconciles every `Pending|Running|Blocked` task to `Cancelled` after the kernel returns `Cancelled`. The Paused path similarly resets `Running`→`Pending` (executor.rs:546-559). |
| F-TSK-03 handoff item 6: "Retry is controller-owned. Any retry-related work must ensure the controller actually re-dispatches retrying tasks." | current (corroborated with nuance) | V02 confirms retry is one-shot `Pending` resolution; the kernel re-claims on the next safe point. EKO does not use `TaskStatus::Retrying` — it uses `Pending` + `retry_count`. The re-dispatch is guaranteed because `Pending` tasks are in the kernel's ready frontier. |

## Coverage And Uncertainty

- **Inspected in full:** the entire `EkoRuntimeDagController`
  implementation (executor.rs:1147-1620); the entire
  `execute_runtime_plan` constructor (1623-1683); the entire
  `execute_run` drain loop and outcome branches (321-585); the
  `TaskDispatcher` trait and `RealTaskDispatcher` (793-1030);
  `select_ownership_safe_wave` (1127-1145); `finalize_cancelled_run_state`
  (643-661); the four `store.rs` functions that own status writes
  (`claim_task`, `set_claimed_task_status`, `requeue_claimed_task`,
  `complete_run_if_quiescent`); the `RuntimeDagController` framework
  trait and the top of `RuntimeDagExecutor::execute`
  (echo-agent `runtime_executor.rs:80-234`).
- **Inspected partially:** the 6272-line `executor.rs` was read in
  relevant slices, not linearly. The `execute_task` per-task pipeline
  (1843-2509) was skimmed for scheduling primitives (none found) but
  not audited for correctness — it is the dispatch seam, owned by
  A-TSK-02 / future per-task audits. The `launch_unattended_run` /
  `drive_unattended_run` / `drive_agent_run` / `launch_cron_run`
  wrappers (3571-3939) were skimmed; they all eventually call
  `execute_run` and add no scheduling authority.
- **Not inspected (out of scope):**
  - `compact_context.rs`, `event_rebuild.rs`, `file_shadow.rs`,
    `file_store.rs`, `hook_event_dispatcher.rs`, `ledger.rs`,
    `memory_bridge.rs`, `planner.rs`, `profiles.rs`, `register.rs`,
    `review.rs`, `task_tools.rs`, `worktree.rs` — these are
    persistence / context / planner / review / worktree helpers
    consumed by the executor; their internals are A-TSK-01 (file
    authority), A-TSK-02 (tool surface), and A-TSK-04 (recovery)
    territory.
  - The full `echo-agent-cli` pre-commit matrix (fmt / clippy /
    all-features test). The review is read-only; the targeted
    `task_runtime::executor` subset is the directly relevant evidence
    and is the suite that exercises the controller integration
    boundary (46 tests pass).
- **Uncertain claims:**
  - The exact probability of A-TSK-03-P3-01's drain-loop race is hard
    to bound without measurement. It is filed as P3 (medium confidence)
    because the window is narrow and the trigger requires an
    unusual user/timer action.
  - Whether any *external* (out-of-repo) consumer of `echo-agent`'s
    `RuntimeDagController` trait reconciles orphaned claims is
    unknowable from this repo. F-TSK-03-P2-02's framework-level gap
    affects those consumers; EKO mitigates it for itself (V03).

## Handoff

- **Conclusions downstream tasks may rely on:**
  - EKO constructs `RuntimeDagExecutor` exactly once
    (executor.rs:1645) and implements `RuntimeDagController` exactly
    once (`EkoRuntimeDagController`, executor.rs:1147-1620). There is
    no second ready-frontier, wave dispatcher, claim authority,
    retry loop, stall detector, or DAG validator in EKO. AGENTS.md
    rule 6 and the "adapter must stay thin" rule hold. (V01, V02)
  - Retry is correctly expressed as a one-shot `Pending` resolution
    that relies on the kernel's safe-point re-claim; the retry budget
    (`task.max_retries`) is the only product-policy input. The kernel's
    `claim.attempt` carries the retry counter. (V02)
  - The eight controller callbacks are persistence, dispatch, or
    product policy — none is a scheduling authority. (V03)
  - F-TSK-03-P2-02's orphan-claim hazard is mitigated at the EKO
    application layer: `finalize_cancelled_run_state` reconciles all
    non-terminal tasks to `Cancelled` after the kernel returns
    `Cancelled`; the Paused path resets `Running`→`Pending` for
    resume. (V03)
  - A-TSK-01-P2-02's execution-state lossiness is **latent, not
    live**: EKO never produces framework `Retrying` or `Paused` on the
    executor path. The doc string at types.rs:917-920 should be
    narrowed. (V02, V03 → A-TSK-03-P3-02)
  - The 46-test executor suite passes; dependency ordering, failure
    propagation, cancellation (pre/dispatch and mid-wave), revision
    insertion at safe points, retry budget, in-flight non-redispatch,
    and blocked-sibling preservation are all covered. (V04)
- **Reports downstream tasks must read:**
  - [V01-01](../validations/A-TSK-03/V01-01.md) for the ownership
    matrix and the call graph.
  - [V02-01](../validations/A-TSK-03/V02-01.md) for the
    no-duplicate confirmation and the retry-path analysis.
  - [V03-01](../validations/A-TSK-03/V03-01.md) for the per-callback
    classification and the orphan-reconciliation evidence.
  - [V04-01](../validations/A-TSK-03/V04-01.md) for the test
    inventory and the EKO↔kernel test-pair mapping.
- **Task-to-reference mapping:**
  - A-TSK-04 (claims/revisions/recovery/terminal monotonicity) → may
    rely on the controller's reconciliation sweeps being in place; must
    verify the resume path correctly re-dispatches the `Running→Pending`
    reset tasks and that `complete_run_if_quiescent`'s CAS is sound
    under concurrent plan patches. Should also confirm the pre-flight
    cycle rejection at `attach_plan_for_test`/`commit_eko_task_plan`
    has parity with the framework `PlanValidator`.
  - A-TSK-02 (task authoring tools) → may rely on `task_execute` being
    a thin shell over `execute_run`; the controller integration is
    sound.
  - A-TSK-01-P2-02 → resolved as latent by A-TSK-03-P3-02. The
    doc-string narrow + optional regression test is the only
    follow-up.
- **Conditions that make this report stale:**
  - Any commit that adds a second `RuntimeDagExecutor::new` or a
    second `RuntimeDagController` impl in `echo-agent-cli` invalidates
    V01.
  - Any commit that introduces a `JoinSet` / wave / frontier /
    stall-timer / DAG-validator in `echo-agent-cli` invalidates V02.
  - Any commit that adds a writer of framework `Retrying` / `Paused`
    on the executor→store path invalidates A-TSK-03-P3-02's "latent"
    classification (it would become live data loss per
    A-TSK-01-P2-02).
  - Any change to the kernel's stall/abort behavior
    (F-TSK-03-P2-01/P2-02) that adds framework-level reconciliation
    changes what `finalize_cancelled_run_state` needs to clean up.
- **Follow-up task IDs (no fixes implemented in this review):**
  - A robustness-focused cleanup task should pick up A-TSK-03-P3-01
    (drain-loop guard). The fix is a two-line check.
  - A documentation-focused cleanup task should narrow the
    `types.rs:917-920` doc string per A-TSK-03-P3-02. The optional
    regression test for the retry round-trip can land in the same
    commit.
  - The framework-level follow-ups in F-TSK-03 (stall detection for
    the in-flight branch; abort-claim reconciliation) remain the
    higher-priority items for the framework robustness track; they are
    not blocking for EKO because EKO mitigates them at the application
    layer.
