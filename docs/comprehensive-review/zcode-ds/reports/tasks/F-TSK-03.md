# F-TSK-03: Runtime DAG execution and claims

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Does `RuntimeDagExecutor` correctly own safe points, bounded Subagent waves,
claims, retries, cancellation, external polling, and stalls?

**Answer: Yes for authority and for the core mechanisms — the framework owns
safe points, the bounded wave, claims, cancellation, failure propagation,
external polling, and stall detection, with no second ready-frontier/retry/
stall loop in EKO. Two P2 defects: (1) a per-task controller error mid-wave
aborts the whole run and orphans the durable Running claims of sibling
dispatches (only boot-time recovery resets them), and (2) EKO's advertised
per-task cancellation is dead code whose documented intent ("stop one
Subagent without cancelling siblings") contradicts the framework's
run-level cancel semantics. Two P3 cleanup items (dead `Retrying` status
variant; untested stall branch with a conflating message). The three
F-TSK-02 findings in executor scope (skip-stall P1-01, string-literal
blocker P2-01, inert `execution_mode: "sequential"` P2-02) were
independently re-verified and remain valid.**

## Scope

Primary source paths inspected (deep read):

- `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs` (full, 988
  lines) — `RuntimeDagExecutor::execute`, safe points, wave dispatch +
  cancellation grace, claim CAS, stall detection, all 7 unit tests.
- `echo-agent/echo-orchestration/src/tasks/runtime.rs` (full) —
  `TaskStatus`/`TaskClaim`/`DagExecutionState`/`ready_task_ids`/
  `blocked_by_failures`/`all_completed`/`refresh_in_flight`.
- `echo-agent/echo-orchestration/src/tasks/verifier.rs` (full) — legacy
  `ManagedTask` verifier family, reachability only.
- `echo-agent/echo-orchestration/src/tasks/hooks.rs` (full) — legacy
  `TaskHooks`/`RetryDecision`/`TaskHookRegistry`, reachability only.
- `echo-agent/echo-orchestration/src/tasks/executor.rs` (sections) — legacy
  `TaskExecutor` + `ManagedTaskDagController` (the second consumer of
  `RuntimeDagExecutor`), reachability only.
- `echo-agent/echo-orchestration/src/tasks/revisioned.rs` (patch engine,
  Skip op), `task_tools.rs` (execution_mode schema), `mod.rs` re-exports.
- EKO `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/`:
  `executor.rs` (EkoRuntimeDagController 1147-1620, execute_runtime_plan
  1622-1683, drain loop 366-434, RealTaskDispatcher 837-960, execute_task
  cancel wiring 1885-1990, resolve_dispatch 1348-1562, finalize_cancelled_run_state
  643-660), `store.rs` (claim_task 986-1029, set_claimed_task_status
  1032-1062, requeue_claimed_task 1066-1105, task_claim_is_current
  1107-1122, set_task_status 953-983, recover_incomplete 1631-1774,
  recoverable_subagent_result 2039-2079, completion gate 453-505,
  run_completion_blockers 587-641, per-task cancel 515-531),
  `task_execute_tool.rs` (entry, preflight, run lock), `register.rs`,
  `types.rs` (to_task 1117-1160, status mapping 370-460),
  `planner.rs` (ownership/conflict analysis 145-200), `service.rs`
  (execute_run callers), `src/tauri/commands/task_runtime.rs`,
  `src/tui/events.rs` (resume entry points).

## Out Of Scope

- EKO file authorities, ledger/event-rebuild losslessness -> A-TSK-01;
  authoring tools -> A-TSK-02; controller boundary (verified only for the
  no-second-loop claim) -> A-TSK-03; claims/recovery end-to-end and
  terminal monotonicity -> A-TSK-04; worktree/finalize -> A-TSK-05.
- Run-level cancel registry and its TUI/GUI surfaces -> A-CHAT-01 / A-TSK-04.
- Subagent dispatch internals (fork/team) -> F-SUB-02 (its P1-01/P1-02
  findings reused only as context; the EKO programmatic delegation surface
  was verified unchanged).
- Full verification of the legacy `ManagedTask` pipeline semantics
  (hooks/verifier/replanner) beyond reachability -> F-TSK-01-P3-01.
- `execution_mode` enforcement decision -> F-TSK-02-P2-02 (re-verified
  here, not re-reported).

## Inputs

- Root `AGENTS.md` (TaskRun → PlanTask → SubagentRun; no worker terms;
  one-authority gates; framework-vs-app layering; UTF-8/panic safety).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (F-TSK-03 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read: `F-TSK-02` (complete — frontier/blocked/
  skip/sequential analysis; P1-01/P2-01/P2-02 cross-checked here) and
  `F-SUB-02` (complete — subagent lifecycle; programmatic delegation
  surface used by EKO task runtime).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md:383-387`
  (M13 Phases 1-5), `echo-agent-cli/docs/2026-07-27-runtime-dag-kernel-convergence.md`
  (ownership boundary, correctness re-open, verification list).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | `RuntimeDagExecutor` + `RuntimeDagController` + `TaskClaim`/`RuntimeTaskClaimOutcome`/`RuntimeTaskResolution` + `DagExecutionState` + safe points, wave semaphore, cancellation grace, external polling, stall — all correctly placed in `echo-orchestration` (single authority). Legacy `hooks.rs`/`verifier.rs`/`TaskExecutor` remain a documented, production-unreachable framework capability (convergence doc Phase 3); not a second live authority. |
| EKO product policy | `select_ownership_safe_wave` (writer-ownership filter of the framework frontier, `executor.rs:1127-1145`), auto-retry budget via `requeue_claimed_task` (`resolve_dispatch` 1396-1431), review/acceptance block policy, attended/unattended stop disposition, drain-loop completion gate (`executor.rs:366-434`), boot recovery (`store.rs:1631-1774`), durable Subagent-result reuse (`store.rs:2039-2079`), per-task cancel tokens (`store.rs:515-531`, executor.rs:1885-1901). |
| Adapter boundary | `EkoRuntimeDagController` is thin and lossless (load_snapshot/claim/resolve map 1:1 onto store CAS); `RealTaskDispatcher` threads only context/cancel/semaphores. Defect found: the per-task cancel contract crossing this boundary is dead and its documented semantics contradict the framework (F-TSK-03-P2-02). |
| Duplicate search | Terms: `RuntimeDagExecutor`, `RuntimeDagController`, `claim_task`, `TaskClaim`, `RuntimeTaskClaimOutcome`, `RuntimeTaskResolution`, `note_stalled`, `stalled`, `DAG stalled`, `ready_task_ids`, `ready frontier`, `select_ready_wave`, `external_progress_poll_interval`, `cancellation_grace`, `Retrying`, `retry_count`, `requeue`, `refresh_in_flight`, `DagRefresh`, `get_ready_tasks`, `execute_ready_tasks`, `execute_all`, `worker`, `TaskHooks`, `VerifierFactory`, `ManagedTask`, `RetryDecision`, `execution_mode`, `max_concurrent_subagents`. Results: one live DAG execution authority (`RuntimeDagExecutor`); the legacy `TaskExecutor::execute_ready_tasks`/`get_ready_tasks` loop and `ManagedTaskDagController` are production-unreachable (V01-01, V02-01); zero `worker` terms in either repo; EKO drain loop is a completion gate only; `refresh_in_flight`/`DagRefresh` still have zero production callers (confirms F-TSK-02-P3-01). |
| Migration deletion | No new deletion targets beyond F-TSK-02's: dead `TaskStatus::Retrying` producer surface (F-TSK-03-P3-01), dead per-task cancel trigger (P2-02), legacy `TaskExecutor`/`hooks`/`verifier` surface (already F-TSK-01-P3-01). |

## Current Path

Verified call graph (details in V02-01):

1. Entry: `task_execute` tool (registered `register.rs`; GUI/TUI via
   `resume_task_run`/`drive_run_async`) -> `execute_run` drain loop
   (`executor.rs:330-434`) -> `execute_runtime_plan` (`:1623-1683`) ->
   `RuntimeDagExecutor::execute` with the run's `CancellationToken`.
2. Safe point: every loop boundary reloads the snapshot
   (`runtime_executor.rs:211-233`), re-validates with `PlanValidator`
   (`:214-219`), tracks `active_revision` (`:220-230`).
3. Frontier: `DagExecutionState::from_tasks` -> `ready_task_ids`
   (requires every dep `completed`, `runtime.rs:449-455`) -> controller
   `select_ready_wave` (EKO: `select_ownership_safe_wave`, writer-conflict
   filter) -> `validate_selected_wave` (`runtime_executor.rs:467-492`).
4. Wave: one spawned task per selected task; `subagent_semaphore`
   (`max_concurrent_subagents`) bounds concurrent dispatches; each closure
   acquires the permit, `claim_task(expected_revision)` CAS
   (`Claimed | ReloadSnapshot`), then `dispatch_task` (`:348-366`).
5. Claim: EKO `store.claim_task` (`store.rs:986-1029`) atomically checks
   revision + Pending + spec and persists `Running` + `TaskClaim
   { revision, attempt = retry_count+1, spec_hash }`; conflicts return
   `ReloadSnapshot` (never a failure).
6. Resolution: `resolve_dispatch` (`executor.rs:1348-1562`) commits only
   while the same claim is Running (`set_claimed_task_status` /
   `requeue_claimed_task` / `task_claim_is_current` -> `Superseded` on any
   mismatch); auto-retry budget = `claim.attempt-1 < max_retries` ->
   requeue to Pending with retry_count+1 (no unclaimed window, same lock).
7. Cancellation: loop top (`:207-209`) and in-wave biased select with
   `cancellation_grace_period` then `abort_all` (`:390-414`); resolved
   siblings are committed before the run returns Cancelled
   (`:416-447`); EKO `interruption_outcome` distinguishes Paused runs
   (`executor.rs:1595-1613`).
8. Failure: `RuntimeTaskResolution::Failed` -> `failure_errors` map;
   next safe point blocks all dependents
   (`block_task`, "blocked: upstream task failed", `:235-246, 239-244`)
   and returns `Failed`/`Paused` via `failed_task_disposition`
   (`:252-264`).
9. External polling: ready empty + in_flight non-empty -> sleep
   `external_progress_poll_interval` (250 ms) racing cancel (`:276-285`);
   state is rebuilt from the reloaded snapshot (never
   `refresh_in_flight`).
10. Stall: ready empty + in_flight empty + no Blocked task ->
    `note_stalled` + `Failed { "<none>", "DAG stalled with unfinished
    tasks (cycle or blocked)" }` (`:287-313`).
11. Terminal: `execute_runtime_plan` maps outcomes; drain loop is a
    completion gate (`complete_run_if_quiescent` + `run_completion_blockers`),
    re-entering only on Completed to catch appended tasks (`:420-433`).

## Findings

### F-TSK-03-P2-01: A per-task controller error mid-wave aborts the whole run and orphans sibling claims — only boot-time recovery can unstick the run

- Priority: P2
- Confidence: high (code path) / medium (trigger probability)
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:348-365`
  (the wave closure propagates semaphore and `claim_task` errors with `?`,
  returning `Err`), `:379-381` (`Some(Ok(Err(error))) => return Err` —
  the whole wave/run aborts), `:418-421` (`resolve_dispatch(...).await?` —
  a resolve error drops the remaining wave results), and the fact that
  `JoinSet` drop aborts the still-running sibling dispatch tasks.
- Reachability: `task_execute` (GUI/TUI/CLI/LLM) -> `execute_run` ->
  `execute_runtime_plan` -> `execute`; triggered when EKO
  `store.claim_task`/`resolve_dispatch` returns a real error (run-lock
  poison, plan missing mid-run, dispatcher task-id mismatch at
  `executor.rs:1337-1345, 1389-1394`) or a join panic occurs. Claimed
  siblings whose dispatch was aborted never pass through `resolve_dispatch`.
- Expected invariant: a per-task persistence/controller fault must not
  corrupt the state of the other tasks in the same wave; at minimum the
  run must end in a state a same-process retry can recover from.
- Observed behavior: `execute()` returns `Err` immediately; EKO marks the
  run `Failed` (`executor.rs:537-541`). The aborted siblings stay
  `Running` with durable claims and no terminal event. On a same-process
  retry the framework executor treats them as externally in-flight
  (`runtime.rs:353-363`) and polls `external_progress_poll_interval`
  forever — they can never complete because their dispatch futures were
  aborted. Only `recover_incomplete` at process boot
  (`store.rs:1631-1774`, resets Running -> Pending/Blocked) or manual
  `task_update` clears the orphans.
- Impact: a single transient store fault during a wave turns a run that
  could have reloaded (`ReloadSnapshot` exists precisely for optimistic
  conflicts) into a permanent hang until process restart; sibling claims
  are left in a state that is neither terminal nor recoverable in-process.
- Root cause: the wave closure conflates "claim conflict" (graceful
  `ReloadSnapshot`) with "claim fault" (abort), and the executor returns
  on the first `Err` without resolving or resetting the sibling claims it
  already made durable.
- Direction: on a claim/dispatch fault, drain the JoinSet the same way the
  cancellation path does (`:396-412`), resolve or explicitly abandon the
  sibling claims through the controller (e.g., a new
  `RuntimeTaskResolution::Aborted` or a controller `abandon_claim`),
  and then return the error; alternatively treat claim faults like
  `ReloadSnapshot` after N bounded attempts. Delete nothing: the change is
  inside the wave loop.
- Regression validation: a framework fixture where `claim_task` errors for
  one of two wave tasks asserts (a) the other task's claim is resolved or
  explicitly abandoned (no durable `Running` survivor), (b) the run
  returns a typed error, (c) re-running with a healed controller
  completes. An EKO test with a poisoned run lock mid-wave.
- Validation reports: [V03-01](../validations/F-TSK-03/V03-01.md),
  [V01-01](../validations/F-TSK-03/V01-01.md)

### F-TSK-03-P2-02: EKO's per-task cancellation is dead code whose documented intent contradicts the framework's run-level cancel semantics

- Priority: P2
- Confidence: high
- Layer: adapter (EKO surface) with framework behavior mismatch
- Evidence: EKO per-task token registration + comment
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:1885-1901`
  ("remove_task / update_task can cancel it to stop this Subagent promptly
  without cancelling sibling tasks"), token map `store.rs:515-531`
  (`task_cancel_tokens`), zero callers of `TaskRuntimeStore::cancel_task`
  (V01-01); framework converts any cancelled task into a run-level
  cancellation — `runtime_executor.rs:435-439`
  (`RuntimeTaskResolution::Cancelled` -> `pending_outcome = Cancelled`)
  and `:267-269` (any `Cancelled` task in the snapshot ->
  `interruption_outcome`).
- Reachability: `cancel_task` is a `pub fn` with no call site in either
  repository today; if it were wired (per its doc comment), the cancelled
  task would resolve `Cancelled` and the framework would abort the whole
  run; a persisted `Cancelled` task also makes every subsequent
  `execute()` return `Cancelled` immediately (no requeue op for
  `Cancelled` tasks exists in `TaskUpdateOperation`, types.rs:1280-1325).
- Expected invariant: an advertised per-task control surface either works
  (one task stops, siblings continue, run continues) or is not advertised;
  framework `Cancelled` semantics and EKO comments agree.
- Observed behavior: the per-task cancel trigger is never invoked
  (dead code with a misleading comment), and the framework's semantics
  would make it a run-abort anyway — the EKO comment's "without cancelling
  sibling tasks" cannot be honored by the current executor.
- Impact: misleading public API (`store.cancel_task`, the
  `task_cancel_tokens` registration) plus a latent trap: wiring the
  advertised surface later would silently convert a per-task stop into a
  whole-run cancellation, and a run containing any `Cancelled` task can
  never be re-executed without manual edits.
- Root cause: the per-task token machinery predates the run-level cancel
  contract and was never either deleted or reconciled with the framework's
  "cancelled task == cancelled run" rule.
- Direction: either (a) implement per-task cancel in the framework
  (`RuntimeTaskResolution::Cancelled` keeps the run alive — cancelled
  treated like `Skipped` for readiness, at safe-point policy) and wire
  `update/remove_task` -> `store.cancel_task`, or (b) delete the per-task
  token registration/`cancel_task` surface and correct the comment to
  state that task cancellation is run-level. A-TSK-04 should pick the
  product decision; delete target is the dead trigger whichever way.
- Regression validation: framework fixture "one task resolves Cancelled,
  run continues and completes siblings"; EKO test wiring
  `update_task -> cancel_task` asserting the run outcome matches the
  chosen semantics; grep for `cancel_task` callers after the change.
- Validation reports: [V03-01](../validations/F-TSK-03/V03-01.md),
  [V01-01](../validations/F-TSK-03/V01-01.md)

### F-TSK-03-P3-01: `TaskStatus::Retrying` is a dead variant of the canonical status model — no live producer exists; the real retry mechanism uses `Pending` + `retry_count`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `TaskStatus::Retrying` defined at
  `echo-orchestration/src/tasks/runtime.rs:102-105` with transition rules
  (`:148-155`, `is_running` `:124`); the only producer in either
  repository is the legacy `TaskExecutor` (`executor.rs:1193`), which is
  production-unreachable (V01-01); EKO's live auto-retry requeues to
  `Pending` with a retry_count bump (`store.rs:1066-1105`,
  `executor.rs:1396-1418`); EKO `try_from_task_status` rejects
  `Retrying` as unrepresentable (`types.rs:432-440`) and
  `project_task_status` maps it to `Running` (`:443-447`).
- Reachability: definition -> re-export (`mod.rs`) -> EKO projection
  arms + framework display arms (`task_tools.rs:632`, `replanner.rs:165`)
  -> no producer on any live path.
- Expected invariant: every variant of the canonical `TaskStatus` is
  either produced by the runtime or deleted (AGENTS.md cleanup rule).
- Observed behavior: the model advertises a `Retrying` lifecycle state
  that no live code path enters; consumers must still handle it
  (transition table, projections, persisted-status error extraction at
  `runtime_executor.rs:458`).
- Impact: misleading public API and dead surface; a consumer relying on
  `Retrying` to observe retry activity never sees it (EKO retries look
  like Pending -> Running cycles).
- Root cause: the variant was carried over from the pre-M13 managed model
  when status enums were merged (convergence doc Phase 4) and the new
  kernel retry was implemented with `Pending` + `retry_count` instead.
- Direction: delete `TaskStatus::Retrying` (variant, transition arms,
  `is_running` arm, `persisted_status_error` arm, EKO projection arms) or,
  if retry visibility is desired, produce it from `requeue_claimed_task`'s
  transition — pick one and remove the other; the Pending+retry_count
  mechanism is the live one.
- Regression validation: `cargo test -p echo_orchestration --lib --locked`
  green after removal; grep for `Retrying` returns only expected matches;
  EKO retry test `task_update_*`/requeue suite stays green.
- Validation reports: [V01-01](../validations/F-TSK-03/V01-01.md),
  [V03-01](../validations/F-TSK-03/V03-01.md)

### F-TSK-03-P3-02: The stall branch has no test coverage and conflates skip-with-dependents and other unreachable states with genuine deadlock

- Priority: P3
- Confidence: high (facts) / low (impact beyond F-TSK-02-P1-01)
- Layer: framework
- Evidence: stall path `runtime_executor.rs:287-313` (ready empty, in_flight
  empty, no `Blocked(_)` task -> `note_stalled` + `Failed { "<none>",
  "DAG stalled with unfinished tasks (cycle or blocked)" }`); no unit test
  exercises it (V03-01 inventory: the 7 `runtime_executor` tests cover
  ordering/revision, all-skipped, cancelled snapshot, cancel-drain, claim
  reload, terminal details, failure-block — none hits `:287-313`);
  `DagExecutionState::from_tasks` leaves `TaskStatus::Paused` unclassified
  (`runtime.rs:353-391`) so a hypothetical Paused task also lands in the
  stall branch; `ready_task_ids` demands `completed` dependencies
  (`runtime.rs:449-455`) so mid-DAG `Skipped` stalls (F-TSK-02-P1-01).
- Reachability: the stall branch is reachable today via skip-with-
  dependents (F-TSK-02-P1-01, medium confidence end-to-end); the Paused
  variant has no live producer in either repo (V01-01) and is theoretical.
- Expected invariant: the single stall detection reports the actual cause
  (skip propagation missing vs genuine deadlock vs unsupported status) and
  is covered by a fixture; "cycle" must not be named for non-cycle causes.
- Observed behavior: all unreachable-non-terminal configurations collapse
  into one message naming "cycle or blocked", with `failed_task_id =
  "<none>"` and no way for the UI to distinguish user-caused skip stalls
  from real deadlocks.
- Impact: misleading diagnostics on the one supported path (skip), no
  regression net for the stall decision, and a silent mis-handling of
  `Paused` snapshots for future framework consumers.
- Root cause: the stall branch was implemented as a catch-all after the
  convergence and never given per-cause differentiation or tests; the
  skip semantics gap (F-TSK-02-P1-01) is the only live trigger.
- Direction: with F-TSK-02-P1-01's fix (skip propagation or restricted
  skip), differentiate the stall reason (skip-dependency vs blocked vs
  unsupported status) and add a framework fixture for the stall branch
  (a `Paused` task and a skip-with-dependents graph); keep the framework
  as the sole stall owner.
- Regression validation: new fixtures asserting the precise stall/fail
  reason per configuration; EKO test asserting the run terminal message
  for a mid-DAG skip is never "cycle".
- Validation reports: [V03-01](../validations/F-TSK-03/V03-01.md),
  [V04-01](../validations/F-TSK-03/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (authority call graph: safe point/claim/retry/stall/wave/worker terms; legacy second-loop search; per-task cancel callers) | yes | passed | [V01-01](../validations/F-TSK-03/V01-01.md) |
| V02 | Registration and runtime reachability trace (task_execute -> execute_run -> RuntimeDagExecutor -> controller -> subagent dispatch) | yes | passed | [V02-01](../validations/F-TSK-03/V02-01.md) |
| V03 | Invariant/edge cases: stale claim/attempt, cancel & failure propagation, revision reload, stall fixtures, F-TSK-02 cross-checks (P1-01/P2-01/P2-02) | yes | passed (4 findings produced) | [V03-01](../validations/F-TSK-03/V03-01.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked` (runtime_executor, runtime, hooks, verifier, legacy executor) | yes | passed (exit 0; 7 + 10 + 3 + 5 + 23 ok) | [V04-01](../validations/F-TSK-03/V04-01.md) |
| V04 | EKO store claim/requeue/recovery tests (claim race, stale claim, boot recovery, execution identity) | yes | passed (exit 0; 7 ok) | [V04-02](../validations/F-TSK-03/V04-02.md) |
| V04 | EKO executor cancellation tests | yes | passed (exit 0; 3 ok) | [V04-03](../validations/F-TSK-03/V04-03.md) |
| V05 | Historical-document drift (MASTER-PLAN M13, runtime-dag-kernel-convergence) | conditional | passed | [V05-01](../validations/F-TSK-03/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:383 M13 P1 — framework `RuntimeDagExecutor` sole execution loop; revision safe point, ready frontier, Subagent wave, cancellation, failure propagation, external in-flight wait, stall removed from the app layer | current | `runtime_executor.rs:196-449`; EKO drain loop is completion policy only (`executor.rs:366-434`); no second loop/validator (V01-01/V02-01) |
| Convergence doc ownership table — revision safe-point reload and atomic task claim framework-owned; EKO owns durable result recovery, review/acceptance, product limits | current | `store.rs:986-1122` CAS claim/requeue; `executor.rs:1348-1562` review/retry policy; V02-01 |
| Convergence doc — "adapter may not implement another DAG loop, dependency validator, or generic retry state machine" | current | EKO retry = requeue-to-Pending policy (single write), not a state machine; legacy retry loop production-unreachable (V01-01) |
| Convergence doc — "Fixed skipped-plan nodes so they count as deliberately resolved instead of producing a false DAG stall" | current but incomplete | all-skipped graphs complete (`runtime_executor.rs:271-273`, V04-01); mid-DAG skip stalls (F-TSK-02-P1-01, re-confirmed V03-01) |
| Convergence doc — `TaskClaim { revision, attempt, spec_hash }`; every completion/failure/block/retry write carries the claim; late results return `Superseded` | current | `store.rs:986-1122`; `Superseded` handled at `executor.rs:1374, 1408, 1449, 1459, 1510, 1519, 1542, 1555`; framework ignores `Superseded` (`runtime_executor.rs:423-426`); tests V04-02 |
| Convergence doc — execution identity `{run}:{task}:{revision}:{attempt}`; changed spec gets a new id without retry bump | current | `executor.rs:174-179`; `patched_spec_uses_new_execution_identity_without_retry_bump` passed (V04-02) |
| Convergence doc verification — framework executor tests cover safe-point revision, claim-conflict reload, skipped tasks, downstream blocking, persisted details | current | 7 tests re-run and passed (V04-01) |
| Convergence doc verification — EKO store tests cover patch-before-claim, stale completion after cancellation | current | `claim_reloads_when_task_update_wins_revision_race`, `stale_claim_cannot_overwrite_cancelled_task` passed (V04-02) |
| Convergence doc — snapshot cancellation is a first-class runtime outcome; downstream tasks transition directly to Blocked after upstream failure | current | `runtime_executor.rs:267-269`; `:239-244`; tests V04-01/V04-03 |
| F-TSK-02-P1-01 skip-with-dependents stalls with misleading "cycle or blocked" | current (independent re-verification) | `runtime.rs:449-455` + `runtime_executor.rs:287-313` (V03-01) |
| F-TSK-02-P2-01 blocked-reason string-literal cross-repo contract | current (independent re-verification) | `runtime_executor.rs:243` vs `store.rs:1292` (V03-01) |
| F-TSK-02-P2-02 `execution_mode: "sequential"` never enforced | current (independent re-verification) | zero reads in both executors (V03-01) |
| F-TSK-02-P3-01 `refresh_in_flight`/`DagRefresh` no production caller | current (independent re-verification) | executor polls via snapshot reload, never `refresh_in_flight` (V03-01) |
| F-TSK-01-P3-01 legacy `TaskManager`/`TaskExecutor` (incl. hooks/verifier/RetryDecision) production-unreachable | current (independent re-verification) | V01-01/V02-01; only `#[cfg(test)]` constructions |

## Coverage And Uncertainty

- All conclusions are static except the V04 test runs; no live LLM DAG run
  was executed (read-only review). P2-01's end-to-end impact (hang until
  boot) is a deterministic code trace plus the store fault trigger that is
  hard to provoke in practice — hence medium confidence on likelihood,
  high on the behavior itself.
- EKO `resolve_dispatch`'s dispatch-error branch maps subagent hard
  failures/timeouts straight to `Failed` without consuming the retry
  budget (retry applies only to post-execution assessment failures). This
  was read as deliberate product policy (M7 review-gate semantics) and not
  raised as a finding; A-TSK-04 should confirm the intended retry surface.
- The EKO retry/requeue CAS, the drain-loop completion gate, and
  `recover_incomplete` were verified for this task's authority questions
  but their full recovery semantics belong to A-TSK-04; anything there
  invalidating "only boot recovery resets orphaned claims" would weaken
  P2-01's impact.
- `TaskHookBridge`/`SubagentHookBridge` (`src/hooks_bridge.rs`) are public
  framework APIs with zero internal callers; they were classified as
  retained public API (per framework deletion rules), not as a new
  finding beyond F-TSK-01-P3-01.
- The legacy `ManagedTaskDagController` inside `executor.rs` was verified
  as reachable only from `execute_all` (zero production callers); its
  semantics were not re-reviewed (F-TSK-01 scope).

## Handoff

- Downstream tasks may rely on: one live DAG execution authority
  (`RuntimeDagExecutor`) with safe points, bounded waves, claim CAS,
  cancellation grace, failure blocking, external polling, and stall
  detection; zero `worker` terms; EKO has no second frontier/retry/stall
  loop; claims are CAS-protected end to end and stale writes return
  `Superseded` (V01-V04); retry is EKO policy via requeue-to-Pending
  (framework re-dispatches); per-task controller faults orphan sibling
  claims (P2-01); per-task cancel is dead and semantically mismatched
  (P2-02); `Retrying` variant dead (P3-01); stall branch untested and
  conflating (P3-02); F-TSK-02 P1-01/P2-01/P2-02/P3-01 independently
  confirmed.
- Reports to read: all seven validation reports; dependency reports
  F-TSK-02 and F-SUB-02 (the EKO programmatic delegation signatures
  `delegate_to_agent_with_prompt_payload` at executor.rs:2833/2941/2955
  used by the task runtime were confirmed unchanged — F-SUB-02 P1 fixes
  must not alter them).
- Stale conditions: this report becomes stale if `runtime_executor.rs`
  wave/claim/stall logic, `runtime.rs` frontier semantics, EKO
  `store.rs` claim/requeue/recovery, EKO `executor.rs` controller or
  `finalize_cancelled_run_state`, or the `TaskStatus` enum change; also
  if a per-task cancel caller appears (would make P2-02's "dead" claim
  wrong but its "mismatched" claim stronger).
- Follow-up task IDs: A-TSK-03 (controller boundary; P2-02 surface
  decision), A-TSK-04 (retry/requeue surface, orphaned-claim recovery,
  per-task vs run-level cancel), X-TSK-01 (canonical graph conformance;
  P2-01/P2-02 cross-repo semantics), Q-FLT-02 (stall/skip/cancel/claim
  fault fixtures from P3-02 and P2-01), S-RDM-01 (P2-01 wave-abort fix,
  P2-02 cancel decision, P3-01/P3-02 cleanup).
