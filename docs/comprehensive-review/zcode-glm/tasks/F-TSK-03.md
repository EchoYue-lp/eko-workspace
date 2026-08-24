# F-TSK-03: Runtime DAG execution and claims

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only scope)
> Worktree state: clean

## Question

Does `RuntimeDagExecutor` correctly own safe points, bounded Subagent
waves, claims, retries, cancellation, external polling, and stalls?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs` (988
  lines) — the canonical runtime kernel. Read in full:
  `RuntimePlanSnapshot` (26), `RuntimeStopDisposition` (33),
  `RuntimeTaskResolution` (40, including `Superseded` at 53),
  `RuntimeTaskClaimOutcome` (58, including `ReloadSnapshot` at 62),
  `RuntimeDagOutcome` (67), the `RuntimeDagController` trait (86, with
  `select_ready_wave` default at 105, `note_stalled` default at 143),
  `RuntimeDagExecutorConfig` (150, defaults at 159),
  `RuntimeDagExecutor` (175), `execute` (196, the full safe-point loop),
  the wave JoinSet + cancellation-grace select (339-414),
  `validate_selected_wave` (467), `stop_outcome` (494), and the
  `ScriptedController` test harness + 7 tests (511-988).
- `echo-agent/echo-orchestration/src/tasks/runtime.rs` (665 lines) — the
  product-neutral runtime primitives. Read in full: `TaskStatus` +
  `can_transition_to`/`transition_to` (90-174), `TaskSpec`/`stable_hash`
  (179-208), `TaskClaim`/`execution_id` (211-224), `TaskExecution` (227),
  `Task` (251), `TaskSubagentContext` (258, with `child_delegation_context`
  at 283), `TaskSubagent` trait (331), `DagExecutionState` (342, with
  `from_tasks` at 353, `refresh_in_flight` at 395, `ready_task_ids` at
  438, `blocked_by_failures` at 461, `all_completed` at 481,
  `all_unfinished_failed_or_blocked` at 487).
- `echo-agent/echo-orchestration/src/tasks/executor.rs` (2556 lines) —
  the application-facing `TaskExecutor` that wraps the kernel. Read:
  the dual-path architecture (1-65), `execute_ready_tasks` (570, legacy
  path), `execute_with_scheduler` (603), `spawn_parallel_batch` (661),
  `execute_all` (1415, the sole production entry that delegates to
  `RuntimeDagExecutor`), `execute_all_async` (1463, one-wave primitive),
  `resume_from_store` (1571, calls `execute_all` at 1604), and the
  `ManagedTaskDagController` adapter (1607-1821, implements
  `RuntimeDagController`).
- `echo-agent/echo-orchestration/src/tasks/verifier.rs` (535 lines) —
  confirmed to contain NO DAG/frontier/cycle logic; it is purely
  per-task completion verification (`CommandVerifier` etc.).
- `echo-agent/echo-orchestration/src/tasks/revisioned.rs` (1155 lines) —
  confirmed to contain NO scheduler/executor/wave/frontier logic; it is
  the `RevisionedTaskGraph` data model audited under F-TSK-01.
- `echo-agent/echo-orchestration/src/tasks/hooks.rs` (371 lines) —
  confirmed to contain NO scheduling primitives (no
  `Semaphore`/`JoinSet`/`spawn`/`dispatch`/`wave`); it is the hook
  registry (`RetryDecision`, `TaskHookRegistry`).
- `echo-agent/echo-orchestration/src/tasks/manager.rs:136-160` —
  `claim_pending_task` atomicity and spec-equality guard.

Cross-repo duplicate search (see V01) for `RuntimeDagExecutor`,
`execute_ready_tasks`, `execute_all_async`, `refresh_in_flight`,
`note_stalled`, `TaskExecutor`, `RuntimeDagController` across
`echo-agent` and `echo-agent-cli`.

## Out Of Scope

Deferred to named task IDs:

- The structural DAG validator (`PlanValidator`,
  `task_dependency_cycles`, `task_topological_order`) — audited under
  **F-TSK-02**. This task treats the validator as a black box that the
  kernel invokes at line 214.
- The revisioned task-graph data model (`RevisionedTaskGraph`,
  `TaskPlan` artifact semantics) — audited under **F-TSK-01**. This
  task consumes its conclusion that the graph is the sole authority
  and that `TaskPlan` is an artifact, not a runtime state machine.
- The `SubagentExecutor` execution-mode lifecycle (Sync/Fork/Teammate/
  Team/Background) — audited under **F-SUB-02**. This task treats the
  dispatch layer behind `dispatch_task` as opaque; only the kernel's
  cancellation/claims contract toward it is in scope.
- Application-layer (EKO) task runtime, `task_execute`, DomainProfile,
  worktree/file-ownership policy, review gates — deferred to
  **A-TSK-03** through **A-TSK-06**. The `ManagedTaskDagController` is
  inspected only as the in-tree reference adapter, not as product
  policy.
- The full retry/verification/replanning pipeline inside
  `run_task_with_retry` (executor.rs) and the `Verifier` trait
  implementations — these live in the controller's per-task pipeline,
  not in the kernel.

## Inputs

Required repository documents read:

- `AGENTS.md` (root, via system reminder). Key constraints applied:
  rule 6 (one task-relationship authority API; `TaskPlan` is an
  artifact; `task_create/task_update/task_list` + EKO `task_execute`,
  no parallel plan CRUD); the framework-vs-application layering gate;
  the "first search whether it already exists" pre-implementation gate;
  the adapter-must-stay-thin rule ("adapter 不得重新拥有 ready frontier、
  DAG 主循环、通用重试/取消、死锁判断"); code-cleanup (delete over
  retain); UTF-8 safety; the cross-repository boundary gate.
- `docs/comprehensive-review/REPORTING.md` (in full).
- `docs/comprehensive-review/templates/task-report.md` and
  `templates/validation-report.md` (in full).

Dependency task reports read:

- `tasks/F-TSK-02.md` (in full). Established: exactly one structural
  DAG validator (`PlanValidator` + free functions in
  `planning/validator.rs`); `tasks/dag.rs` is thin delegation;
  structural validation is status-independent and runs on `TaskSpec[]`
  pre-execution. F-TSK-03 consumes this: the kernel invokes
  `PlanValidator` at the top of every safe-point loop and must not
  introduce a second validator.
- `tasks/F-SUB-02.md` (in full). Established: Sync/Fork/Teammate/
  Background share one lifecycle (cancel-token propagation +
  `select!`-based grace/timeout); **Team mode is the outlier** — no
  cancel propagation, detached on timeout, bypasses
  `execute_agent_streaming`, always reports `Completed`. F-TSK-03
  consumes the cancellation-grace pattern as the design precedent the
  kernel mirrors, and notes that the kernel's `dispatch_task` adapter
  contract is the seam where Team mode's gaps would resurface if a
  Team-mode subagent were dispatched through this kernel.

Historical documents treated as hypotheses:

- `runtime_executor.rs:1-6` module doc: "The framework owns dependency
  traversal, revision safe points, bounded Subagent waves, cancellation,
  failure propagation, and stall detection. Applications provide
  persistence, dispatch, review, and product policy." Treated as design
  intent; **mostly corroborated** with one exception — stall detection
  does not cover the externally in-flight branch (F-TSK-03-P2-01).
- `executor.rs:1409-1414` doc on `execute_all`: "Hooks, retries,
  verification, replanning, timeout, and persistence stay in this
  executor's per-task pipeline. Dependency traversal, bounded waves,
  cancellation, failure propagation, and stall detection have one
  authority: `RuntimeDagExecutor`." Treated as design intent;
  **corroborated** for production, but two legacy scheduling paths
  remain on `TaskExecutor` itself (F-TSK-03-P3-01).
- `runtime.rs:351-352` doc on `from_tasks`: "Already-completed tasks
  are treated as resolved; already-running tasks are treated as
  externally in-flight." Treated as design intent; **corroborated** and
  load-bearing for the external-polling branch.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `RuntimeDagExecutor` is generic revisioned-DAG execution machinery any `echo-agent` consumer needs: ready-frontier computation, bounded-wave dispatch, optimistic-concurrency claims, cancellation-grace coordination, failure/stall detection. It lives correctly in `echo-orchestration::tasks::runtime_executor`. `DagExecutionState`, `TaskClaim`, `TaskStatus`, `TaskSubagentContext` are product-neutral primitives in `runtime.rs`. The kernel has no EKO-specific field. |
| EKO product policy | None at the kernel layer. All product policy (persistence shape, scheduling strategy, review gates, file-ownership, retry/verify/replan) is injected via the `RuntimeDagController` trait. The kernel never reads `DomainProfile`, worktree config, or EKO event protocols. |
| Adapter boundary | `ManagedTaskDagController` (executor.rs:1607-1821) is the in-tree reference adapter that bridges the in-memory `TaskManager` to the kernel. It is mostly thin: `load_snapshot` projects `ManagedTask` → `Task`; `claim_task` delegates to `claim_pending_task`; `dispatch_task` maps status → `execute_selected_task`; `resolve_dispatch` checks claim identity (Superseded on mismatch) and persists. It owns the `claims` map (a claim-identity mirror) and the scheduler call in `select_ready_wave` — both acceptable adapter responsibilities. It does NOT recompute the ready frontier (the kernel does) or run its own DAG traversal. |
| Duplicate search | Searched names (whole `echo-agent` + `echo-agent-cli`): `RuntimeDagExecutor`, `RuntimeDagController`, `RuntimeDagOutcome`, `execute_ready_tasks`, `execute_all`, `execute_all_async`, `TaskExecutor`, `refresh_in_flight`, `note_stalled`, `select_ready_wave`, `validate_selected_wave`, `claim_pending_task`. Result: ONE kernel authority (`RuntimeDagExecutor::execute`). `TaskExecutor::execute_all` is the sole production caller (constructs the kernel at executor.rs:1422). Two legacy scheduling paths on `TaskExecutor` (`execute_ready_tasks`, `execute_all_async`) bypass the kernel but have zero production callers (F-TSK-03-P3-01). `DagExecutionState::refresh_in_flight` is defined but unused outside its own tests (F-TSK-03-P3-02). |
| Migration deletion | No migration proposed in this review. The dead/legacy surfaces identified here are deletion candidates per the AGENTS.md "code cleanup" rule, but that is a follow-up action, not part of this review task. |

## Current Path

Verified runtime-kernel call graph at commit `9b0e0fa`:

```text
TaskExecutor::execute_all()                                  [executor.rs:1415]
   ├─ ManagedTaskDagController::new(self.clone())             [:1421]
   ├─ RuntimeDagExecutor::new(controller, config)             [:1422, runtime_executor.rs:182]
   │      .with_validator(PlanValidator{...relaxed...})       [:1435]
   └─ runtime.execute("framework-task-executor", cancel)      [runtime_executor.rs:196]
          loop {
            ① SAFE-POINT ENTRY
               if cancel.is_cancelled() → interruption_outcome        [:207]
               snapshot = controller.load_snapshot(run_id)            [:213]
               validator.validate_task_snapshot(&snapshot.tasks)?     [:214]   ← single validator (F-TSK-02)
               if active_revision != Some(snapshot.revision) { log }  [:220]   ← safe-point revision tracking

            ② TERMINAL DETECTION
               if any task in state.failed →                        [:235]
                  block downstream via controller.block_task          [:239-245]
                  failed_task_disposition → stop_outcome(Fail|Pause) [:252-264]
                  return Ok(stop_outcome)
               if !state.cancelled.is_empty() → interruption_outcome [:267]
               if state.all_completed → return Ok(Completed)         [:271]

            ③ READY FRONTIER + STALL DETECTION
               ready = state.ready_task_ids(&tasks)                  [:275]
               if ready.is_empty() {
                  if !state.in_flight.is_empty() {                   [:277]   ← external poll (NO stall timeout)
                     select! { cancel | sleep(poll_interval) }       [:278-283]
                     continue
                  }
                  if any Blocked task → stop_outcome for it          [:287-306]
                  else → controller.note_stalled + return Failed     [:308-313]   ← stall backstop
               }

            ④ WAVE SELECTION + DISPATCH
               selected = controller.select_ready_wave(tasks, ready)  [:316-318]
               validate_selected_wave(ready, selected)?               [:319]   ← non-empty, subset, no dupes
               for task in selected:                                  [:340-366]
                  join_set.spawn({
                     semaphore.acquire_owned().await?                 [:349]
                     claim = controller.claim_task(run_id, task, expected_revision)  [:352-358]
                       Claimed(claim) | ReloadSnapshot → Ok(None)
                     dispatch = controller.dispatch_task(ctx, claim, task)          [:360-362]
                     Ok(Some((task, claim, dispatch)))
                  })

            ⑤ WAVE DRAIN WITH CANCELLATION GRACE
               cancellation_observed = false;  cancellation_grace = sleep(ZERO)  [:369-371]
               while !join_set.is_empty() {                            [:372]
                  select! biased {
                    join_next → wave_results.push | Ok(None) skip | Err → return Err   [:375-388]
                    cancel.cancelled(), if !observed → observed=true; reset grace      [:390-395]
                    cancellation_grace, if observed →                              [:396-413]
                       join_set.abort_all()
                       drain: Ok(Some)→push, Ok(None)→skip,
                              Err(cancelled)→ {}  ★ aborted claims silently dropped
                              Err(other)→ return Err
                       break
                  }
               }

            ⑥ RESOLUTION (safe-point write-back)
               pending_outcome = cancellation_observed ? Cancelled : None  [:416]
               for (task, claim, dispatch) in wave_results:                 [:417-441]
                  resolution = controller.resolve_dispatch(run_id, claim, task, dispatch)
                  Completed|Pending|Skipped|Superseded → {}
                  Failed{error} → failure_errors.insert(task.id, error)     [:427-429]
                  Blocked{error,disp} → pending_outcome ||= stop_outcome    [:430-434]
                  Cancelled → pending_outcome ||= Cancelled                 [:435-439]
               if pending_outcome → return Ok(outcome)   ← "resolve whole wave first" [:443-447]
          } loop
```

Invariants verified by this graph (full evidence in V01-V04):

- **Single kernel authority.** `RuntimeDagExecutor::execute` is the one
  owner of ready-frontier traversal, bounded-wave dispatch, claims,
  cancellation, failure propagation, and stall detection. The only
  production entry is `TaskExecutor::execute_all` (executor.rs:1415),
  which constructs the kernel and delegates. `resume_from_store` reuses
  `execute_all` (executor.rs:1604), so resume also goes through the
  kernel. The `ManagedTaskDagController` adapter holds no frontier, no
  DAG loop, no second validator.
- **Single validator.** The kernel holds one `PlanValidator`
  (runtime_executor.rs:178) and invokes
  `validate_task_snapshot` at the top of every safe-point iteration
  (line 214). No second validator exists in the kernel or the adapter
  (consistent with F-TSK-02).
- **Safe points are loop iterations.** Every iteration begins by
  reloading the snapshot (line 213). The previous wave's JoinSet is
  fully drained (line 372) and resolved (line 417) before the next
  iteration. The comment at line 211-212 ("Every loop boundary is a
  safe point") is accurate. Revision changes are detected at line 220
  and logged; the loop naturally re-derives `DagExecutionState` from
  the new snapshot.
- **Bounded Subagent waves.** A `Semaphore(max_concurrent_subagents.max(1))`
  bounds concurrent dispatches within a wave (line 201, acquired at
  349). `validate_selected_wave` (line 319/467) guarantees the wave is
  a non-empty, duplicate-free subset of the ready frontier.
  `select_ready_wave` (controller hook, default identity at line 105)
  is where product policy (e.g. file-ownership conflict avoidance) may
  narrow the wave — but it must return at least one id.
- **Claims are optimistic-concurrency controlled.** `claim_task` takes
  `expected_revision` (line 95-98); mismatch returns `ReloadSnapshot`
  (line 62), which the spawned closure converts to `Ok(None)` (line
  357) — the task silently drops out of the wave and the next
  iteration reloads. `TaskClaim` carries `revision + attempt +
  spec_hash` (runtime.rs:211-216); `resolve_dispatch` checks claim
  identity and returns `Superseded` on mismatch (adapter at
  executor.rs:1741-1744).
- **Cancellation has a grace period.** `cancel` is cloned into each
  `TaskSubagentContext` (line 343-346). When `cancel.cancelled()` fires
  mid-wave, the kernel sets `cancellation_observed` and resets a grace
  timer to `now + cancellation_grace_period` (line 390-395). Only
  after the grace does `join_set.abort_all()` fire (line 397). The wave
  is fully resolved before honoring the stop outcome (line 443-447).

Invariants **violated** or only partially held (full evidence in V01-V04):

- **Stall detection does not cover the externally in-flight branch.**
  When `ready` is empty but `state.in_flight` is non-empty, the kernel
  polls `external_progress_poll_interval` forever with no timeout and
  no `note_stalled` callback (line 277-285). If an in-flight task
  never resolves, the executor loops indefinitely; only the cancel
  token breaks it. (F-TSK-03-P2-01)
- **Cancellation abort silently drops in-flight claims.** Aborted
  tasks (`Err(cancelled)`) are skipped at line 403 — they are NOT
  passed to `resolve_dispatch`, so their `Running` status (persisted
  by `claim_task`) is never reconciled by the kernel. On resume they
  become orphaned in-flight tasks and combine with F-TSK-03-P2-01.
  (F-TSK-03-P2-02)
- **Retry is NOT owned by the kernel.** `Retrying` is treated as
  in-flight (`TaskStatus::is_running()` at runtime.rs:123-125). The
  kernel does not re-dispatch retrying tasks — that is the
  controller's job. This is a deliberate layering choice (retry +
  backoff + verification lives in `run_task_with_retry` behind
  `dispatch_task`), but it means a controller that fails to re-dispatch
  a `Retrying` task also stalls (same mechanism as F-TSK-03-P2-01).

## Findings

The headline result is mostly positive: `RuntimeDagExecutor` is the
single production authority for DAG traversal, bounded waves, claims,
cancellation, failure propagation, and (partial) stall detection; it
reuses the single `PlanValidator` from F-TSK-02; the
`ManagedTaskDagController` adapter is thin. The recorded findings are
two P2 recovery/stall gaps and three P3 cleanup/coverage items.

### F-TSK-03-P2-01: Stall detection does not cover the externally in-flight branch; the executor can poll indefinitely

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:275-314`
    — the ready/stall block. When `ready_task_ids.is_empty()`, the
    first sub-branch is `if !state.in_flight.is_empty()` (line 277):
    it `select!`s between `cancel.cancelled()` and
    `tokio::time::sleep(self.config.external_progress_poll_interval)`
    and `continue`s. There is **no maximum poll count, no max-poll
    duration, and no `note_stalled` callback** on this branch. The
    stall callback (`note_stalled` + `Failed("DAG stalled...")`) lives
    only in the final `else` at line 308-313, which requires
    `state.in_flight.is_empty()`.
  - `echo-agent/echo-orchestration/src/tasks/runtime.rs:359-363,
    487-500` — `DagExecutionState::from_tasks` classifies any task
    with `TaskStatus::Running` or `Retrying` as `in_flight`, and
    `ready_task_ids` (line 438-458) excludes `in_flight` tasks. So an
    orphaned `Running`/`Retrying` task lands in `in_flight`, not in
    the stall path.
  - The module doc (runtime_executor.rs:1-6) claims the framework
    "owns … stall detection," but the implementation only detects
    stalls when nothing is in-flight.
- Reachability: any scenario that leaves a task in `Running`/`Retrying`
  with no resolver:
  (a) **Cancellation abort orphans** — see F-TSK-03-P2-02. After
    `join_set.abort_all()` (line 397), tasks whose `claim_task`
    already persisted `Running` but whose dispatch was aborted are
    never resolved. On the next `execute()` call (resume), they appear
    in `in_flight`, ready is empty, and the kernel polls forever.
  (b) **External worker death** — a task dispatched by another
    process (or a previous `execute()` call that crashed) leaves a
    `Running` row with no live dispatcher.
  (c) **Retrying task the controller never re-dispatches** — the
    kernel treats `Retrying` as externally in-flight and waits for the
    controller to flip it back to `Pending`/`Running`; if the
    controller does not, poll forever.
- Expected invariant: per the task question ("can the executor stall
  indefinitely?"), the executor should terminate for any input. The
  module doc claims stall detection is a framework responsibility.
- Observed behavior: the kernel polls `external_progress_poll_interval`
  (default 250 ms) indefinitely while any task is in-flight. The
  `note_stalled` callback is never invoked for this case, so the
  controller/UI receives no stall signal. The cancel token is the only
  escape.
- Impact: a run that has any orphaned or externally-stuck in-flight
  task hangs. Because the kernel advertises stall detection
  (F-TSK-03-P2-01 contradicts the module doc), a controller author
  who relies on `note_stalled` to surface deadlocks will not be
  notified for the most common stall shape (orphaned in-flight). For
  a local personal assistant this is a hang that the user must
  manually cancel.
- Root cause: the in-flight branch was designed to wait for genuinely
  external workers (the doc on `from_tasks` at runtime.rs:351-352
  explicitly says already-running tasks are "treated as externally
  in-flight"). The design assumes external workers always terminate.
  Orphaned claims from cancellation abort (F-TSK-03-P2-02) and dead
  external workers were not considered as stall inputs.
- Direction: add stall detection to the in-flight branch. Options: (a)
  track per-task in-flight duration and fire `note_stalled` + `Failed`
  when any in-flight task exceeds a configurable `max_in_flight_wait`;
  (b) detect "snapshot unchanged across N consecutive polls while
  in-flight is non-empty" and fire `note_stalled`; (c) at minimum,
  surface a `note_stalled` callback the first time the in-flight poll
  loop is entered so the controller can decide. Pair with F-TSK-03-P2-02
  so cancellation-aborted claims are reconciled before the outcome
  returns, removing the most common orphan source.
- Regression validation: add a test that loads a snapshot with one
  `Running` task whose status never changes, dispatches with a fresh
  cancel token, and asserts the executor either terminates within a
  bounded time or invokes `note_stalled`. Today this test would hang
  without cancellation.
- Validation reports: [V01](../validations/F-TSK-03/V01-01.md),
  [V04](../validations/F-TSK-03/V04-01.md).

### F-TSK-03-P2-02: Cancellation abort silently drops in-flight claims without resolving them (orphaned Running tasks)

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:396-413`
    — the post-grace drain after `join_set.abort_all()` (line 397).
    The match arm
    `Err(error) if error.is_cancelled() => {}` (line 403) silently
    skips aborted JoinHandles. They are NOT pushed into
    `wave_results`, so the resolution loop at line 417-441 never sees
    them.
  - `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:352-358`
    — `claim_task` runs before `dispatch_task`. For the in-tree
    adapter, `ManagedTaskDagController::claim_task`
    (executor.rs:1623-1648) calls `claim_pending_task`, which
    atomically flips the task to `Running` (manager.rs:136-160). So an
    aborted task between claim and dispatch completion has `Running`
    persisted but no terminal resolution.
  - `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:116-122,
    84-88` — `resolve_dispatch` is the documented place where "task
    state [is committed] before completing so the next safe-point
    snapshot remains authoritative." Skipping it for aborted tasks
    leaves the snapshot non-authoritative for those tasks.
  - Contrast: completed siblings ARE pushed to `wave_results` (line
    400) and resolved, honoring the comment at line 443-444 ("Resolve
    the whole wave before honoring a stop request so completed
    siblings are never replayed after resume"). Aborted tasks are
    explicitly excluded from this protection.
- Reachability: any cancellation that fires after at least one task in
  the wave has claimed but not finished dispatching. With the default
  5 s grace period this is common — a wave dispatched, claims
  persisted, then the user cancels before the LLM/tool calls finish.
- Expected invariant: on any terminal exit (cancel, timeout, error),
  every claimed task should either be resolved to a terminal status by
  `resolve_dispatch` or be reconciled by an explicit contract. The
  kernel's current contract leaves aborted claims in `Running` with no
  documented reconciliation obligation on the controller.
- Observed behavior: after cancellation, the kernel returns
  `RuntimeDagOutcome::Cancelled` (via `pending_outcome` at line 416 /
  `interruption_outcome` at line 208/268/280). Aborted tasks remain
  `Running` in the store. Their claims are dropped from the in-tree
  adapter's `claims` map only if `resolve_dispatch` ran — which it did
  not. On resume, those tasks are `in_flight` and combine with
  F-TSK-03-P2-01 to stall the executor.
- Impact: orphaned `Running` rows after every non-trivial
  cancellation. The product-level symptoms are (1) a resume that
  hangs (with F-TSK-03-P2-01), or (2) a controller that must
  independently reconcile `Running` tasks whenever it receives a
  `Cancelled` outcome — an obligation the kernel does not document or
  enforce. For EKO's local-assistant threat model this is a
  reliability defect (hung runs), not a safety issue.
- Root cause: the cancellation-grace drain was written to honor
  completed siblings but treated aborted tasks as "cancel-propagated,
  so they'll clean themselves up." That is true for the dispatch
  future (it is dropped), but not for the already-persisted `Running`
  status — the controller's `claim_task` ran first and committed.
- Direction: the kernel should pair `abort_all` with a resolution
  pass for the aborted claims. Concretely, before returning the
  cancelled outcome, for each task that claimed but was not resolved,
  call `resolve_dispatch` with a synthetic cancelled/aborted dispatch
  result (or add a `controller.reconcile_aborted(run_id, &[claims])`
  hook). This closes the orphan source and, combined with F-TSK-03-P2-01's
  stall detection, removes the resume-hang. Alternatively, document
  the controller obligation explicitly in the `RuntimeDagController`
  trait doc and add a regression test that exercises resume after
  cancellation.
- Regression validation: extend `cancellation_drains_wave_and_preserves_completed_siblings`
  (runtime_executor.rs:837-894) with a second `execute()` call on the
  same controller, and assert the orphaned `Running` task either gets
  reconciled to `Cancelled` (after the fix) or that the executor does
  not hang.
- Validation reports: [V03](../validations/F-TSK-03/V03-01.md),
  [V04](../validations/F-TSK-03/V04-01.md).

### F-TSK-03-P3-01: `TaskExecutor::execute_ready_tasks` and `execute_all_async` are parallel scheduling paths that bypass `RuntimeDagExecutor` (test-only / dead in production)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/tasks/executor.rs:570-597` —
    `execute_ready_tasks` computes its own frontier
    (`task_manager.get_ready_tasks()`), sorts by priority, optionally
    applies a `TaskScheduler`, and calls `spawn_parallel_batch` (line
    661). It does NOT go through `RuntimeDagExecutor`: no claims, no
    revision safe points, no stall detection, no bounded Subagent
    waves, no `note_stalled`. The comment at line 587 labels it
    "legacy behaviour."
  - `echo-agent/echo-orchestration/src/tasks/executor.rs:1463-1560` —
    `execute_all_async` spawns the ready frontier as
    `BackgroundTask`s with its own semaphore and calls
    `manager.wake_dependents` (line 1550) for DAG advancement. The
    comment at line 1459 explicitly positions it as "a one-wave
    primitive, not a second full DAG executor."
  - Whole-repo caller search (V01): `echo-agent-cli` has **zero**
    references to `execute_ready_tasks`, `execute_all`, or
    `execute_all_async`. Within `echo-agent`, `execute_all` is used
    only by 4 test sites (executor.rs:2168, 2198, 2289, 2410, 2457);
    `execute_ready_tasks` is used by ~14 test sites
    (executor.rs:1888, 1938, 1962, 2104, 2133, 2138, 2143, 2232,
    2338, 2343, 2378, 2381, 2524, 2543) plus doc comments (36, 1596);
    `execute_all_async` has **zero** callers anywhere (only its own
    definition and doc references at 326, 330, 504).
- Reachability: none in production. The sole production caller of the
  task-execution pipeline is `execute_all` (which delegates to the
  kernel); `resume_from_store` reuses `execute_all` (executor.rs:1604).
- Expected invariant: per AGENTS.md rule 6 ("task relationship has one
  authority API") and the "code cleanup: delete over retain" rule, a
  second scheduling path that bypasses the kernel's claims/safe-points/
  stall detection should not linger as live API surface.
- Observed behavior: `TaskExecutor` exposes three scheduling entry
  points with materially different semantics. `execute_ready_tasks`
  and `execute_all_async` silently lose the kernel guarantees
  (claims, revision safety, stall detection). A framework consumer
  reading the API surface sees three "execute" methods and may pick
  the wrong one.
- Impact: API clutter + false signal. The "legacy" path's semantics
  differ from the canonical path; any production caller of
  `execute_ready_tasks` or `execute_all_async` would silently lose
  `RuntimeDagExecutor`'s guarantees. Low severity today (no
  production callers), but a latent trap.
- Root cause: `execute_ready_tasks` predates the Sprint that
  introduced `RuntimeDagExecutor`. `execute_all` was added as the new
  authority, and `execute_all_async` was added as a "non-blocking
  one-wave primitive," but neither legacy method was removed. The
  test suite was not migrated.
- Direction: per AGENTS.md, delete `execute_all_async` (zero callers).
  For `execute_ready_tasks`, either delete and migrate the ~14 test
  sites to `execute_all`, or — if kept as a test-only primitive —
  mark it `#[cfg(test)]` or document prominently that it bypasses the
  runtime kernel. Deleting is preferred: the AGENTS.md bias is toward
  deletion, and the test migration is mechanical.
- Regression validation: `cargo test --workspace --all-features`. After
  migrating the legacy test sites, confirm `execute_all_*` coverage
  still exercises scheduler priority, conflicts, and blocked-task
  skipping.
- Validation reports: [V01](../validations/F-TSK-03/V01-01.md).

### F-TSK-03-P3-02: `DagExecutionState::refresh_in_flight` is a dead public API (used only in its own unit test)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/tasks/runtime.rs:394-435` —
    `refresh_in_flight` updates `in_flight` against a newer snapshot
    and returns a `DagRefresh` summarizing the transitions.
  - Whole-repo caller search: `refresh_in_flight` is referenced at
    exactly two sites — its definition (runtime.rs:395) and its own
    unit test `dag_refresh_observes_external_in_flight_completion`
    (runtime.rs:647). `RuntimeDagExecutor::execute` does NOT call it;
    it rebuilds `DagExecutionState::from_tasks(&tasks)` on every
    iteration (runtime_executor.rs:233), making incremental refresh
    unnecessary.
- Reachability: none in production.
- Expected invariant: per AGENTS.md "first search whether it already
  exists" + "code cleanup," a public API with no live consumer should
  either have a documented external use case or be deleted.
- Observed behavior: the method exists as framework API surface. A
  reader may conclude the kernel does incremental in-flight refresh;
  it does not (full reload each safe point, by design).
- Impact: API clutter and a misleading signal about how the kernel
  tracks in-flight state. Low severity.
- Root cause: `refresh_in_flight` was likely written for an earlier
  incremental-tracking design. The kernel settled on full-reload per
  safe point (simpler, correct, and cheap for in-memory graphs), and
  `refresh_in_flight` was not removed.
- Direction: delete `refresh_in_flight` and `DagRefresh` (and the
  `dag_refresh_observes_external_in_flight_completion` test), OR
  document at the method that it is an opt-in utility for external
  consumers. Per the AGENTS.md framework-API retention rule, this is
  a `pub` method on a framework type, so the bias is toward retention
  unless clearly unused — it is clearly unused in-tree, so deletion
  is defensible. Lower priority than F-TSK-03-P3-01.
- Regression validation: `cargo test -p echo_orchestration --lib`.
- Validation reports: [V01](../validations/F-TSK-03/V01-01.md),
  [V02](../validations/F-TSK-03/V02-01.md).

### F-TSK-03-P3-03: The stall-detection, external-polling, and cancellation-abort branches have no test coverage

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:277-285`
    (external in-flight poll), `308-313` (`note_stalled` + `Failed`),
    `396-413` (post-grace abort drain, esp. the silent
    `Err(cancelled) => {}` arm at line 403) — grep over the test
    module (line 511-988) shows no test exercises these branches.
  - The seven existing tests cover: dependency ordering + revision
    insertion (line 769), skipped-as-resolved (808), cancelled-
    snapshot-as-interrupted (822), cancellation drains wave +
    preserves completed siblings (837), claim-conflict reloads
    (896), persisted terminal error details (924), downstream
    blocking + exhausted-graph failure (957). None covers stall,
    external-poll completion, or resume-after-orphan.
- Reachability: the untested branches are the highest-risk paths
  (stall = hang; external poll = cross-process coordination; abort
  drain = claim orphaning).
- Expected invariant: per AGENTS.md verification discipline, the
  branches most likely to misbehave under concurrency/recovery should
  have regression tests.
- Observed behavior: the most fragile branches are the least tested.
  F-TSK-03-P2-01 and F-TSK-03-P2-02 would likely have been caught by
  a stall/orphan-resume test.
- Impact: latent defects in P2-01/P2-02 went undetected. Future
  refactors of the safe-point loop have no regression net for these
  paths.
- Root cause: the test suite was built around the happy path and the
  explicit cancellation test (837) was added before the orphan-resume
  concern was identified.
- Direction: add tests for (a) a graph that reaches `note_stalled`
  (e.g. an N-task pending set with a cyclic dependency injected after
  bypassing the validator, or a validator stub that returns Ok);
  (b) an externally in-flight task that completes after one poll and
  unblocks a dependent; (c) cancellation abort followed by a second
  `execute()` on the same controller asserting no orphan `Running`
  remains (pairs with F-TSK-03-P2-02's fix).
- Regression validation: the new tests themselves.
- Validation reports: [V03](../validations/F-TSK-03/V03-01.md),
  [V04](../validations/F-TSK-03/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Authority call graph: single kernel; single production entry; legacy/dead scheduling paths inventoried | yes | passed | [V01-01](../validations/F-TSK-03/V01-01.md) |
| V02 | Stale claim/attempt scenarios: optimistic-concurrency reload, stale-claim Superseded, concurrent-claim guard | yes | passed | [V02-01](../validations/F-TSK-03/V02-01.md) |
| V03 | Cancellation + failure propagation: cancel token into ctx, grace-then-abort, partial failure deferred to next safe point | yes | passed | [V03-01](../validations/F-TSK-03/V03-01.md) |
| V04 | Revision reload + stall: safe-point reload, revision logging, stall backstop fires only when in_flight empty (can stall indefinitely otherwise) | yes | passed | [V04-01](../validations/F-TSK-03/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — three code comments treated as hypotheses; classifications inline in Inputs) | passed | classified inline (two current-with-caveat, one current) |

Executed cargo commands (all exit 0):

```text
cargo test -p echo_orchestration --lib runtime_executor   (7 passed)
cargo test -p echo_orchestration --lib dag_               (6 passed: 4 runtime + 2 executor DAG propagation)
cargo test -p echo_orchestration --lib execute_all        (2 passed)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `runtime_executor.rs:1-6` module doc: "The framework owns dependency traversal, revision safe points, bounded Subagent waves, cancellation, failure propagation, and stall detection." | current-with-caveat | Dependency traversal, safe points, bounded waves, cancellation, failure propagation all confirmed (V01, V03). **Stall detection is partial** — it fires only when `in_flight` is empty; the in-flight poll branch has no stall detection (F-TSK-03-P2-01). V04. |
| `executor.rs:1409-1414` `execute_all` doc: "Dependency traversal, bounded waves, cancellation, failure propagation, and stall detection have one authority: `RuntimeDagExecutor`." | current-for-production / stale-for-API-surface | True for the production path (`execute_all` is the sole production caller). But `TaskExecutor` still exposes `execute_ready_tasks` (test-only) and `execute_all_async` (dead) which bypass the kernel (F-TSK-03-P3-01). V01. |
| `runtime.rs:351-352` `from_tasks` doc: "already-running tasks are treated as externally in-flight." | current | `DagExecutionState::from_tasks` (runtime.rs:359-363) classifies `Running`/`Retrying` as `in_flight`; the kernel's external-poll branch (runtime_executor.rs:277-285) consumes this. V01, V04. |
| AGENTS.md rule 6: "任务关系只有一个权威 API" (single task-relationship authority) | current (corroborated for the kernel) | `RuntimeDagExecutor::execute` is the single production authority; `ManagedTaskDagController` is a thin adapter that holds no frontier/DAG-loop/validator. The legacy `execute_ready_tasks`/`execute_all_async` are violations of this rule but are out of production reach (F-TSK-03-P3-01). V01. |
| AGENTS.md: "adapter 不得重新拥有 ready frontier、DAG 主循环、通用重试/取消、死锁判断" | current (corroborated) | `ManagedTaskDagController` does not own the ready frontier (kernel's `DagExecutionState::ready_task_ids` does), the DAG loop (kernel's `execute` loop), retry (`run_task_with_retry` is behind `dispatch_task`, controller-owned), or deadlock detection (kernel's stall branch). V01, V02. |
| F-TSK-02 handoff: "F-TSK-03 must not introduce a second validator; should call `PlanValidator` before scheduling" | current (corroborated) | `RuntimeDagExecutor` holds one `PlanValidator` (runtime_executor.rs:178) and calls `validate_task_snapshot` at line 214 before computing the ready frontier. No second validator. V01. |
| F-SUB-02 handoff: "cancellation-grace pattern (per-mode `select!` with grace before hard abort)" | current (mirrored at the kernel level) | `RuntimeDagExecutor`'s wave drain mirrors F-SUB-02's grace pattern: cancel observed → reset grace timer → `abort_all` after grace (runtime_executor.rs:390-413). V03. The Team-mode gaps F-SUB-02 filed are behind `dispatch_task`, not the kernel's responsibility. |

## Coverage And Uncertainty

Inspected in full: `runtime_executor.rs` (988 lines — the entire
kernel: safe-point loop, terminal detection, ready/stall branches,
wave selection + dispatch, cancellation-grace drain, resolution
loop, all 7 tests), `runtime.rs` (665 lines — all primitives,
`DagExecutionState`, `TaskClaim`, `TaskSubagentContext`, all DAG-state
tests), `executor.rs` (relevant slices: 1-130 config, 560-660 legacy
path, 1390-1605 the `execute_all`/`execute_all_async`/`resume_from_store`
trio, 1607-1821 the full `ManagedTaskDagController` adapter),
`verifier.rs` (535 lines — confirmed no DAG logic), `hooks.rs` (371
lines — confirmed no scheduling primitives), `manager.rs:136-160`
(`claim_pending_task` atomicity).

Inspected partially:
- `revisioned.rs` (1155 lines) — only grep-confirmed to contain no
  scheduler/executor/wave/frontier logic. The full
  `RevisionedTaskGraph` semantics are F-TSK-01's scope. The kernel's
  `RuntimePlanSnapshot` (runtime_executor.rs:26-29) is a flat
  `(revision, Vec<Task>)` shape; how a controller builds it from a
  `RevisionedTaskGraph` is the controller's concern.
- The retry/verification/replanning pipeline inside
  `run_task_with_retry` and `execute_selected_task` (referenced from
  `ManagedTaskDagController::dispatch_task` at executor.rs:1699). The
  kernel treats this as opaque behind `dispatch_task`. Whether retry
  correctly flips `Retrying` ↔ `Running` and eventually terminates is
  behind that seam; it affects F-TSK-03-P2-01 (a stuck `Retrying`
  task stalls) but is not the kernel's bug.

Not inspected (out of scope):
- The application-layer (EKO) task runtime in `echo-agent-cli` —
  whether EKO constructs a `RuntimeDagController` that reconciles
  orphaned `Running` tasks on `Cancelled` (mitigating F-TSK-03-P2-02),
  supplies a real revisioned store, or threads `note_stalled` to the
  UI. Only the framework-side kernel + in-tree adapter were inspected.
- Whether any external (out-of-repo) `echo-agent` consumer subclasses
  `TaskExecutor` and calls `execute_ready_tasks`/`execute_all_async`
  in production. Per AGENTS.md framework-API retention, this
  possibility is the only argument for keeping the legacy paths; the
  in-repo evidence (zero callers) supports deletion.

Environmental constraints:
- All 15 relevant tests pass (7 `runtime_executor` + 6 DAG-state/DAG-
  propagation + 2 `execute_all`). Worktree state clean at `9b0e0fa`.
- The feature matrix beyond `echo-orchestration`'s default was not
  re-run (F-FEAT-01 owns it). The kernel has no feature gate.
- No probe was added/removed — all validations are read-only or use
  pre-existing tests.

Uncertain claims:
- Whether the "resolve the whole wave before honoring a stop request"
  invariant (comment at runtime_executor.rs:443-444) is intended to
  cover aborted claims or only completed siblings. The code clearly
  excludes aborted claims (line 403), but the comment's wording
  ("whole wave") suggests broader intent. F-TSK-03-P2-02 is framed
  as "the narrower reading is what the code does; the broader reading
  would close the orphan gap."
- Whether EKO's application adapter reconciles orphaned `Running`
  tasks on `Cancelled`. If it does, F-TSK-03-P2-02's impact is
  mitigated at the application layer (but the framework gap remains
  for other consumers). This is A-TSK-04's scope.

## Handoff

Conclusions downstream tasks may rely on:

1. **`RuntimeDagExecutor` is the single production authority for DAG
   execution.** Safe points, bounded waves, claims, cancellation,
   failure propagation, and (partial) stall detection all live in
   `runtime_executor.rs::execute`. `TaskExecutor::execute_all`
   (executor.rs:1415) is the sole production entry;
   `ManagedTaskDagController` is a thin adapter. Any downstream task
   that assumes a second scheduling authority exists should be
   disabused: there is none in production (F-TSK-03-P3-01 lists the
   test-only/dead exceptions).
2. **The kernel reuses the single `PlanValidator` from F-TSK-02.** No
   second validator was introduced. Downstream tasks can rely on the
   validator-then-schedule ordering (runtime_executor.rs:214 before
   275).
3. **Cancellation is grace-based and propagates into `TaskSubagentContext`.**
   The kernel mirrors F-SUB-02's grace-then-abort pattern at the wave
   level. Downstream tasks adding cancellation features must preserve
   the grace timer (runtime_executor.rs:370, 390-413).
4. **Stall detection is partial.** The kernel detects stalls only when
   `in_flight` is empty. Any in-flight task (orphaned claim, dead
   external worker, stuck `Retrying`) makes the kernel poll forever
   (F-TSK-03-P2-01). Downstream tasks must NOT assume `note_stalled`
   fires for in-flight stalls — it does not.
5. **Cancellation can orphan claims.** Aborted tasks are not resolved
   by the kernel (F-TSK-03-P2-02). Downstream tasks (especially
   A-TSK-04) must verify that the application adapter reconciles
   orphaned `Running` tasks on `Cancelled`, or adopt the fix
   recommended here.
6. **Retry is controller-owned, not kernel-owned.** `Retrying` is
   externally in-flight from the kernel's perspective. Any retry-
   related work must ensure the controller actually re-dispatches
   retrying tasks, or the kernel stalls (F-TSK-03-P2-01 mechanism).

Reports they must read:

- This report (F-TSK-03) for the kernel's authority, claims,
  cancellation, and stall coverage.
- `tasks/F-TSK-02.md` for the single-validator invariant the kernel
  relies on (line 214).
- `tasks/F-SUB-01.md` for the `TaskClaim`/`execution_id` identity
  contract the kernel's claim resolution uses.
- `tasks/F-SUB-02.md` for the cancellation-grace pattern the kernel
  mirrors and the Team-mode lifecycle gaps that live behind
  `dispatch_task`.
- `validations/F-TSK-03/V01-01.md` through `V04-01.md` for per-claim
  evidence.

Conditions that make this report stale:

- Adding stall detection to the in-flight poll branch
  (runtime_executor.rs:277-285) — resolves F-TSK-03-P2-01, requires
  re-running V04.
- Resolving aborted claims in the cancellation drain (line 396-413) —
  resolves F-TSK-03-P2-02, requires re-running V03/V04.
- Deleting `execute_ready_tasks`/`execute_all_async` (executor.rs:570,
  1463) — resolves F-TSK-03-P3-01, requires re-running V01.
- Deleting `DagExecutionState::refresh_in_flight` (runtime.rs:395) —
  resolves F-TSK-03-P3-02, requires re-running V01/V02.
- Threading retry through the kernel (rather than behind
  `dispatch_task`) — would change the `Retrying`-is-in-flight
  assumption that underpins F-TSK-03-P2-01.
- Any commit that gives the adapter its own frontier/DAG loop/validator
  — invalidates V01 and the single-authority conclusion.

Follow-up task IDs (no implementation in this review):

- **A framework robustness task** should fix F-TSK-03-P2-01 (stall
  detection for the in-flight branch) and F-TSK-03-P2-02 (resolve
  aborted claims). These are coupled: the orphaned-claim source
  (P2-02) feeds the stall (P2-01); fixing P2-02 removes the most
  common orphan source, and fixing P2-01 catches the rest.
- **A cleanup task** should delete `execute_all_async` (zero callers)
  and decide on `execute_ready_tasks` (delete + migrate tests, or
  `#[cfg(test)]`), resolving F-TSK-03-P3-01. It should also evaluate
  `DagExecutionState::refresh_in_flight` for deletion
  (F-TSK-03-P3-02).
- **A test-coverage task** should add the three regression tests
  listed in F-TSK-03-P3-03 (stall, external-poll completion,
  resume-after-orphan).
- **A-TSK-04** (claims/revisions/recovery/terminal monotonicity)
  should verify whether EKO's application adapter reconciles orphaned
  `Running` tasks on `Cancelled` outcomes — if not, F-TSK-03-P2-02's
  product impact is higher than this framework-only review can
  establish.
