# A-TSK-03: Task execution controller boundary

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Does EKO inject only product policy into `RuntimeDagExecutor`, with no second
ready-frontier, retry, cancellation, or stall loop?

**Answer: Yes for the core authority question — there is exactly one live DAG
execution loop (`echo_orchestration::tasks::RuntimeDagExecutor`), and EKO's
controller injects only product policy (ownership-safe wave filter, review/
retry/acceptance policy, worktree integration, durable-result reuse, drain
completion gate); no second ready frontier, retry state machine, cancellation
loop, or stall detection exists anywhere in EKO. However, the controller
boundary itself mishandles three outcome classes, each aggravated by the EKO
integration: (P1) a pause request issued while a wave is in flight is silently
converted into a permanent run cancellation (the framework's in-wave cancel
branch hardcodes `Cancelled` and never consults `interruption_outcome`, and
EKO's `finalize_cancelled_run_state` force-transitions the durably-Paused run
to Cancelled); (P2) a mid-wave controller/store fault aborts the whole run and
orphans sibling Running claims with no in-process recovery — EKO marks the run
Failed without the task cleanup its own cancel/pause paths perform (EKO-side
manifestation of F-TSK-03-P2-01); (P2) EKO's advertised per-task cancellation
remains dead code whose comment contradicts the framework's run-level cancel
semantics (re-verification of F-TSK-03-P2-02).**

## Scope

Primary source paths inspected (deep read):

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs`
  (full production code, lines 1-4030): `execute_run` drain loop
  (:321-585), `run_completion_blockers` (:587-641),
  `finalize_cancelled_run_state` (:643-661), `assess_task_execution`
  (:695-765), `TaskDispatcher`/`RealTaskDispatcher` (:793-1030),
  `select_ownership_safe_wave` (:1127-1145), `EkoRuntimeDagController`
  (:1147-1620), `execute_runtime_plan` (:1622-1683),
  `integrate_reviewed_task` (:1686-1752), `run_review_gate` (:1773-1836),
  `execute_task` + per-task cancel wiring (:1843-2508), `run_readonly_subagent`/
  `run_writer_subagent`/`run_main_agent_task` (:2798-3494), `drive_agent_run`
  (:3649+), test module inventory (:4030-6272).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs`
  (scheduling-relevant sections): run cancel/pause registry and per-task
  cancel tokens (:490-622), `complete_run_if_quiescent` (:453-488),
  `claim_task` (:986-1029), `set_claimed_task_status` (:1032-1062),
  `requeue_claimed_task` (:1066-1105), `task_claim_is_current` (:1107-1121),
  `retry_blocked_task` (:1220-1340), `recover_incomplete` (:1631-1776),
  `transition_run`/`resume_task_run` (:385-448).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/task_execute_tool.rs`
  (full, 966 lines): tool boundary, per-run execution lock, cancel wiring,
  outcome mapping.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/register.rs`
  (full): tool registration for GUI/TUI.
- `echo-agent-cli/echo-agent-app-core/src/run_driver.rs` (full): background/
  agent-driven run driver (L1 only).
- `echo-agent-cli/echo-agent-app-core/src/tasks/service.rs` (:380-490),
  `src/tauri/commands/task_runtime.rs` (:230-290, :318-349, :432-465),
  `src/tui/events.rs` (:4330-4360, :4700-4790): entry points.
- `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs` (full,
  incl. tests): the single authority loop.
- `echo-agent/echo-orchestration/src/tasks/runtime.rs` (full): `TaskStatus`/
  `TaskClaim`/`DagExecutionState`/`ready_task_ids`/`in_flight` semantics.
- `echo-agent/echo-orchestration/src/tasks/revisioned.rs` (:100-145,
  :547-560 Skip guard, :275-277 policy default), `echo-agent/echo-orchestration/
  src/tasks/executor.rs` (legacy second-controller reachability only),
  `echo-agent/echo-orchestration/src/tasks/task_tools.rs` (:304-320 set_status
  schema gate).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs`
  (:517-527 run transition table, :1255-1272 `TaskUpdateOperation`),
  `revisioned_adapter.rs` (:80-130 `EkoTaskToolPolicy`).

## Out Of Scope

- File authorities / adapter losslessness — A-TSK-01 (its P1-01 torn-tail
  finding is consumed here as a wave-abort trigger, not re-reviewed).
- Authoring tools (`task_create/update/list`) — A-TSK-02.
- Claims/revisions/recovery end-to-end, terminal monotonicity, event replay —
  A-TSK-04 (boot recovery consumed here only as the sole orphan-claim
  recovery).
- Worktree/finalize policy — A-TSK-05.
- Framework loop internals beyond the boundary behavior (safe-point validator,
  skip semantics, blocked-reason contract) — F-TSK-02/F-TSK-03 (re-verified
  only where the boundary consumes them).
- Frontend projections of run/task events — A-FE-01/02.

## Inputs

- Root `AGENTS.md` (adapter thin-and-lossless gate; no second frontier/loop;
  one-authority rules; framework-vs-app layering; UTF-8/panic safety).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (A-TSK-03 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read: `A-TSK-01` (complete) and `F-TSK-03`
  (complete; its P2-01/P2-02 are the two defects this task was asked to
  re-verify from the EKO side).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/
  MASTER-PLAN.md:216-234`, `echo-agent-cli/docs/2026-07-27-runtime-dag-kernel-
  convergence.md` (both classified in V05-01).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correctly placed) | `RuntimeDagExecutor` + `RuntimeDagController` + `TaskClaim`/`RuntimeTaskClaimOutcome`/`RuntimeTaskResolution` + `DagExecutionState` + safe points, bounded waves, cancellation grace, external polling, stall detection — all in `echo-orchestration`; the sole live scheduling authority (V01-01). Legacy `TaskExecutor::execute_all`/`ManagedTaskDagController` (executor.rs:1415-1424, 1609) is a second controller but production-unreachable (zero callers in `echo-agent-cli`, V01-01). |
| EKO product policy (application) | `select_ownership_safe_wave` writer filter (executor.rs:1127-1145), auto-retry budget via one CAS requeue (store.rs:1066-1105 + resolve_dispatch policy executor.rs:1396-1431), review/acceptance gate and circuit breaker (executor.rs:1773-1836, review.rs), attended/unattended stop disposition (executor.rs:1174-1186), drain-loop completion gate (executor.rs:366-434, 453-488, 587-641), boot recovery (store.rs:1631-1776), durable Subagent-result reuse (executor.rs:1293-1322), per-task cancel token registration (executor.rs:1887-1907) — the last one is dead-end policy (P2-02). |
| Adapter boundary | `EkoRuntimeDagController` maps the framework's six callbacks 1:1 onto store CAS; `RealTaskDispatcher` threads only context/cancel/semaphores; conversion is thin and lossless (load_snapshot → plan file; claim → store CAS; resolve → claim-guarded writes). The boundary's fault/cancellation *outcome handling* is defective (P1-01, P2-01), but that is a behavior defect inside the boundary, not a second authority. |
| Duplicate search | Terms (both repos, V01-01): `RuntimeDagExecutor`, `RuntimeDagController`, `claim_task`, `requeue_claimed_task`, `ready_task_ids`, `select_ready_wave`, `select_ownership_safe_wave`, `note_stalled`, `stalled`, `DAG stalled`, `external_progress_poll`, `cancellation_grace`, `refresh_in_flight`, `DagRefresh`, `retry_count`, `max_retries`, `max_concurrent_subagents`, `run_dag`, `deadlock`, `worker`, `execute_ready_tasks`, `get_ready_tasks`, `execute_all`, `ManagedTaskDagController`, `cancel_task`, `TaskManager`, `TaskExecutor`. Result: one live loop; EKO hits are all policy or comments; zero `worker` terms. |
| Migration deletion | No new deletion targets in EKO beyond F-TSK-03's: the dead per-task cancel trigger (`store.cancel_task` + token map, P2-02) and — after A-TSK-04 picks the product decision — either the pause-agnostic `finalize_cancelled_run_state` force-transition or the framework's hardcoded in-wave Cancelled (P1-01). Legacy framework `TaskExecutor`/hooks/verifier remain F-TSK-01-P3-01's deletion target, not this task's. |

## Current Path

Verified call graph (V01-01/V02-01; detailed in those reports):

1. Entry: `task_execute` tool (register.rs:45-130; GUI `src/tauri/desktop.rs`,
   TUI `src/main.rs`) or resume commands (Tauri `task_runtime.rs:262/364`,
   TUI `tui/events.rs:4737`, background `service.rs:418`) →
   `execute_run` (executor.rs:321-585). `drive_run_async`/`drive_agent_run`
   (run_driver.rs, executor.rs:3649) is the L1 ReAct path and reaches the same
   executor only through the `task_execute` tool.
2. Drain loop (executor.rs:366-434): reload plan; count unresolved tasks;
   if zero → `run_completion_blockers` (acceptance/review policy, :587-641) →
   `complete_run_if_quiescent` (store.rs:453-488); re-enter only when the
   outcome is `Completed` (to drain appended tasks); otherwise break.
3. `execute_runtime_plan` (executor.rs:1622-1683): builds
   `EkoRuntimeDagController` + `RuntimeDagExecutor` with
   `max_concurrent_subagents` only; maps `RuntimeDagOutcome` → `RunOutcome`.
4. Framework loop (runtime_executor.rs:196-449): safe point per iteration
   (load_snapshot → `PlanValidator` → `active_revision`); cancelled-snapshot
   check (:267-269); `ready_task_ids` (runtime.rs:438-458, deps must be
   `completed`); EKO `select_ready_wave` filters by writer ownership
   (executor.rs:1265-1282; always returns ≥1 for a non-empty frontier, so the
   framework's empty-wave guard at runtime_executor.rs:471-475 is never
   tripped); wave spawn with `claim_task` CAS (runtime_executor.rs:339-366,
   store.rs:986-1029); in-wave cancel with grace then abort (:390-414);
   `resolve_dispatch` (executor.rs:1348-1562) — dispatch error → terminal
   status, execution-failure → requeue-with-budget or Failed, acceptance
   pending → Blocked + disposition, executed → review gate → integrate →
   Completed, all claim-guarded (`Superseded` on mismatch); failure blocking
   (:235-246); external polling (:276-285); stall (:287-313, EKO
   `note_stalled` writes the note).
5. Terminal mapping: Failed/Paused transition the run (executor.rs:1656-1664);
   Cancelled → `finalize_cancelled_run_state` (executor.rs:508, :643-661);
   Paused → reset Running→Pending (:546-559); Err → run Failed (:570-582).
6. Cancellation control: `request_cancel`/`request_pause` (store.rs:577-622)
   operate on the driver token registered by every caller
   (`register_run_cancellation`); pause transitions Paused first, then cancels
   the token.
7. Boot recovery (store.rs:1631-1776): resets orphaned Running tasks to
   Pending (or Blocked with a recovery blocker when a non-replay-safe
   boundary is active).

## Findings

### A-TSK-03-P1-01: Pause during an active wave is silently converted into a permanent run cancellation

- Priority: P1
- Confidence: high (code path and transition legality are deterministic) /
  high (pause-during-wave is the normal usage case)
- Layer: adapter (EKO) with framework behavior mismatch
- Evidence: framework in-wave cancel branch
  `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:390-416`
  (`cancel.cancelled()` → `cancellation_observed` → grace → `abort_all`, then
  `let mut pending_outcome = cancellation_observed.then_some(RuntimeDagOutcome::Cancelled)`)
  and `:443-447` (returns it); `interruption_outcome` is consulted only at the
  loop top (`:207-209`) and the external-poll branch (`:278-281`) — never
  inside the wave; EKO `request_pause` `store.rs:598-622` (transitions
  Running→Paused **first**, then cancels the shared run token; comment: "The
  executor observes the durable Paused status and leaves the run resumable");
  EKO `interruption_outcome` `executor.rs:1595-1613` (would return Paused if
  consulted); `execute_runtime_plan` maps Cancelled → `RunOutcome::Cancelled`
  (`executor.rs:1656-1682`); `execute_run`'s Cancelled branch unconditionally
  runs `finalize_cancelled_run_state` (`executor.rs:508`, `:643-661`), which
  force-transitions the durably-Paused run to Cancelled — legal per
  `types.rs:517-527` (`Paused => matches!(next, Running | Cancelled)`) — and
  flips Pending/Running/Blocked tasks to Cancelled.
- Reachability: GUI pause button → `src/tauri/commands/task_runtime.rs:463`
  `request_pause`; TUI → `src/tui/events.rs:4346`. A wave is in flight
  whenever Subagents are running (seconds to minutes), so a pause lands inside
  the wave-drain `select!` in the common case. The existing test
  `runtime_plan_cancellation_preserves_explicit_pause` (executor.rs:5757-5783)
  seeds the run Paused *before* a pre-cancelled token — the loop-top path —
  and never covers pause-during-wave.
- Expected invariant: pause is durable and resumable regardless of wave
  activity; every cancellation path consults the controller's
  `interruption_outcome`; a Paused run is never force-transitioned to
  Cancelled.
- Observed behavior: `request_pause` during a wave → framework returns
  `Cancelled` → EKO `finalize_cancelled_run_state` transitions the Paused run
  to Cancelled and marks tasks Cancelled → run is terminal and not resumable;
  in-flight Subagent work is aborted after the 5 s grace. The same token
  cannot distinguish pause from cancel, so the user's pause request silently
  becomes a cancel.
- Impact: the product's documented pause contract (store.rs:595-597) is
  violated on the path users actually use it (while work is running); a pause
  kills in-flight work, flips tasks to terminal Cancelled (no requeue op exists
  in EKO's task_update surface, types.rs:1255-1272), and loses the run.
- Root cause: the framework hardcodes `Cancelled` for in-wave cancellation
  instead of asking the controller (the `interruption_outcome` hook exists
  precisely for this), and EKO's `finalize_cancelled_run_state` ignores the
  durable Paused status. Additionally, even a fixed outcome would leave
  mid-dispatch tasks written `Cancelled` by `resolve_dispatch`'s dispatch-error
  branch (executor.rs:1356-1381), which the Paused-resume path (executor.rs:
  546-559, resets only `Running`) would not repair — the fix must distinguish
  pause from cancel at the dispatch-error write as well.
- Direction: (a) framework — after the wave drain, if `cancellation_observed`,
  consult `controller.interruption_outcome(run_id)` instead of hardcoding
  `Cancelled`; (b) EKO — `resolve_dispatch`'s dispatch-error branch should
  write `Pending` (not `Cancelled`) when the run is durably Paused; (c) EKO —
  `finalize_cancelled_run_state` must skip runs whose durable status is Paused
  (or be replaced by the Paused branch's Running→Pending reset). No deletion
  needed; the framework hook already exists.
- Regression validation: framework fixture "cancel fires mid-wave while
  controller's `interruption_outcome` returns Paused → outcome is Paused,
  sibling claims resolved, completed siblings preserved"; EKO fixture
  "`request_pause` while a dispatch is in flight leaves the run Paused and a
  resume re-dispatches without replaying completed tasks" (Q-FLT-02 candidate).
- Validation reports: [V03-01](../validations/A-TSK-03/V03-01.md),
  [V04-01](../validations/A-TSK-03/V04-01.md)

### A-TSK-03-P2-01: EKO's error path is the only terminal path that orphans sibling Running claims — a mid-wave store fault becomes an in-process permanent hang (aggravates F-TSK-03-P2-01)

- Priority: P2
- Confidence: high (code path) / medium (trigger probability — requires a
  real store fault mid-wave, e.g. A-TSK-01-P1-01's torn `events.jsonl` tail)
- Layer: adapter (EKO integration of the framework defect)
- Evidence: wave closure propagates semaphore/`claim_task` errors with `?`
  (`runtime_executor.rs:348-365`), `:379-381` (`Some(Ok(Err(error))) => return Err`
  aborts the wave), `:418-421` (`resolve_dispatch(...).await?` drops the
  remaining wave results), JoinSet drop aborts sibling dispatches; EKO
  `execute_run`'s Err branch marks the run Failed with **no** task cleanup
  (`executor.rs:570-582`) — asymmetric with the cancel path
  (`finalize_cancelled_run_state`, `:643-661`) and the pause path
  (`Running→Pending`, `:546-559`); EKO store fault sources that trigger the
  abort: torn tail hard error (`file_shadow.rs:362-379`, A-TSK-01-P1-01), plan
  read errors, run-lock poison; in-process recovery is impossible —
  `task_update` `Skip` rejects non-Pending/Blocked tasks
  (`revisioned.rs:554-557`), EKO's `TaskUpdateOperation` has no `SetStatus`
  (`types.rs:1255-1272`) and `EkoTaskToolPolicy` does not enable
  `allow_manual_progress_updates` (framework default `false`,
  `revisioned.rs:275-277` vs `revisioned_adapter.rs:106`), `retry_blocked_task`
  rejects Running (`store.rs:1244-1251`); a same-process resume re-enters the
  framework executor which treats the orphaned Running tasks as in-flight
  (`runtime.rs:353-363`) and polls `external_progress_poll_interval` forever
  (`runtime_executor.rs:276-285`); only boot-time `recover_incomplete`
  (`store.rs:1631-1776`, test `boot_recovery_requeues_orphaned_running_task`
  at :2721) resets them.
- Reachability: `task_execute`/resume → `execute_run` → `execute_runtime_plan`
  → framework wave; triggered when `claim_task`/`resolve_dispatch`/store reads
  return a real error mid-wave (torn tail, corrupt projection, poisoned run
  lock, plan missing).
- Expected invariant: a per-task persistence/controller fault must not corrupt
  sibling state; at minimum the run ends in a state a same-process retry can
  recover from (the pause and cancel paths already demonstrate the required
  task-reset behavior).
- Observed behavior: `execute()` returns `Err` → EKO marks the run Failed
  (`executor.rs:570-582`) while siblings stay `Running` with durable claims and
  no terminal event; resuming hangs in external polling forever; only a
  process restart (boot recovery) or manual file edits recover the run.
- Impact: one transient store fault during a wave turns a run that the
  `ReloadSnapshot` mechanism was designed to survive into a permanent hang
  until restart, with sibling claims in a state that is neither terminal nor
  in-process-recoverable.
- Root cause: the framework wave closure conflates "claim conflict"
  (graceful `ReloadSnapshot`) with "claim fault" (abort) and never drains or
  abandons sibling claims on error (F-TSK-03-P2-01), and EKO's terminal
  mapping reuses its cleanup machinery only on the cancel/pause outcomes, not
  on the error outcome.
- Direction: (a) EKO — on `execute_runtime_plan` Err, run the same
  Running-task reset used by the pause path (or `finalize_cancelled_run_state`
  minus the run transition) before marking the run Failed; (b) framework —
  drain the JoinSet and resolve/abandon sibling claims on a wave error, or
  treat claim faults like `ReloadSnapshot` after N bounded attempts (see
  F-TSK-03-P2-01 direction). Delete nothing.
- Regression validation: framework fixture "claim error for one of two wave
  tasks → sibling claim resolved or abandoned, run returns a typed error,
  healed re-run completes" (F-TSK-03-P2-01); EKO fixture "poisoned run lock
  mid-wave → run Failed with zero Running survivors"; EKO fixture "resume of a
  Failed run with an orphaned Running task completes after boot recovery".
- Validation reports: [V03-01](../validations/A-TSK-03/V03-01.md),
  [V04-03](../validations/A-TSK-03/V04-03.md)

### A-TSK-03-P2-02: EKO's per-task cancellation is dead code whose documented intent contradicts the framework's run-level cancel semantics (re-verified)

- Priority: P2
- Confidence: high
- Layer: adapter (EKO surface) with framework behavior mismatch
- Evidence: live token registration/unregistration
  `executor.rs:1887-1907` (comment: "remove_task / update_task can cancel it
  to stop this Subagent promptly **without cancelling sibling tasks**") and
  `store.rs:497-528` (`register_task_cancel_token`/`unregister_task_cancel_token`/
  `cancel_task`); `cancel_task` has **zero callers** in either repository
  (V01-01; the Tauri `commands/tasks.rs:157` `cancel_task` is the legacy
  TaskManager surface); the framework converts any cancelled task into a
  run-level cancellation — `runtime_executor.rs:267-269` (any `Cancelled` task
  in the snapshot → `interruption_outcome`) and `:435-439`
  (`RuntimeTaskResolution::Cancelled` → `pending_outcome = Cancelled`);
  EKO's own dispatch-error path demonstrates the same rule
  (`executor.rs:1356-1381`: cancelled dispatch → `TodoStatus::Cancelled` →
  `Resolution::Cancelled` → run Cancelled); a persisted Cancelled task cannot
  be requeued through EKO's task_update surface (types.rs:1255-1272).
- Reachability: registration runs on every dispatch; the trigger
  (`cancel_task`) is never invoked; if wired per its comment, the cancelled
  task would resolve `Cancelled` and abort the whole run.
- Expected invariant: an advertised per-task control surface either works
  (one task stops, siblings continue) or is not advertised; EKO comments and
  framework semantics agree.
- Observed behavior: the per-task token machinery is inert (dead trigger with
  a misleading comment); the framework would make the advertised behavior a
  run abort anyway.
- Impact: misleading public API plus a latent trap — wiring the surface later
  would silently convert a per-task stop into a whole-run cancellation, and a
  run containing any Cancelled task can never be re-executed (no requeue op).
- Root cause: the per-task token machinery predates the run-level cancel
  contract and was never deleted or reconciled (same root cause as
  F-TSK-03-P2-02, now with the EKO-side registration/comment evidence).
- Direction: (a) implement per-task cancel in the framework (`Cancelled`
  treated like `Skipped` for readiness) and wire
  `update/remove_task → store.cancel_task`; or (b) delete the token
  registration, `cancel_task`, and the token map, and correct the comment to
  state that task cancellation is run-level. A-TSK-04 picks the product
  decision; the dead trigger is the deletion target either way.
- Regression validation: grep `cancel_task` after the change; framework
  fixture "one task resolves Cancelled, run continues and completes siblings";
  EKO test wiring `update_task → cancel_task` asserting the run outcome
  matches the chosen semantics.
- Validation reports: [V03-01](../validations/A-TSK-03/V03-01.md),
  [V01-01](../validations/A-TSK-03/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search (ready frontier / retry / cancel / stall second-implementation sweep; legacy loop reachability; worker terms; cancel_task callers; status-reset surface) | yes | passed | [V01-01](../validations/A-TSK-03/V01-01.md) |
| V02 | Registration + runtime reachability (task_execute / TUI / GUI / background → execute_run → RuntimeDagExecutor; controller callback map; single driver construction) | yes | passed | [V02-01](../validations/A-TSK-03/V02-01.md) |
| V03 | Invariant/edge cases (pause-during-wave; mid-wave fault + orphan claims + in-process recovery; per-task cancel; basic DAG execution cross-check) | yes | failed (3 findings) | [V03-01](../validations/A-TSK-03/V03-01.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked tasks::runtime_executor` | yes | passed (exit 0, 7 ok) | [V04-01](../validations/A-TSK-03/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::executor` | yes | passed (exit 0, 46 ok) | [V04-02](../validations/A-TSK-03/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::store` | yes | passed (exit 0, 34 ok) | [V04-03](../validations/A-TSK-03/V04-03.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::task_execute_tool` | yes | passed (exit 0, 8 ok) | [V04-04](../validations/A-TSK-03/V04-04.md) |
| V05 | Historical-document drift (MASTER-PLAN M13, runtime-dag-kernel-convergence) | conditional | passed | [V05-01](../validations/A-TSK-03/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN M13: `RuntimeDagExecutor` owns revision safe points, ready frontiers, bounded waves, cancellation, failure propagation, external in-flight polling, stall detection; EKO injects snapshots/review/ownership/worktree/events through `EkoRuntimeDagController`; old 596-line `run_dag` loop deleted | current | runtime_executor.rs:196-449; executor.rs:1147-1683; drain loop is a completion gate only (executor.rs:366-434) (V01-01/V02-01/V05-01) |
| Convergence doc: "The adapter may select a product-safe subset of the ready frontier and resolve a dispatch... It may not implement another DAG loop, dependency validator, or generic retry state machine" | current | EKO `select_ready_wave` is a policy filter; retry = single CAS requeue; no second loop/validator (V01-01) |
| Convergence doc ownership table (framework: claim/safe point/frontier/wave/cancel/poll/stall; EKO: policy/persistence/recovery) | current | V05-01 anchor map |
| Convergence doc: "Fixed skipped-plan nodes so they count as deliberately resolved instead of producing a false DAG stall" | current but incomplete | all-skipped graphs complete (runtime_executor.rs:271-273); mid-DAG skip-with-dependents still stalls (F-TSK-02-P1-01) |
| MASTER-PLAN: atomic `TaskClaim`; claim conflict reloads instead of failing; stale writes rejected | current | store.rs:986-1122; runtime_executor.rs:352-358; V04-03 |
| F-TSK-03-P2-01 (wave abort orphans sibling claims; only boot recovery resets them) | current (independent re-verification, EKO side) | runtime_executor.rs:379-381/418-421 + executor.rs:570-582; in-process recovery surface checked and found absent (V03-01) |
| F-TSK-03-P2-02 (per-task cancel dead + semantically mismatched) | current (independent re-verification, EKO side) | executor.rs:1887-1907 + store.rs:497-528; zero callers (V01-01/V03-01) |
| F-TSK-03-P3-02 (stall branch untested, conflating message) | current (outside this task's EKO boundary; note_stalled is a note writer) | runtime_executor.rs:287-313; executor.rs:1615-1619 |

## Coverage And Uncertainty

- All conclusions are static except the V04 test runs; no live LLM DAG run was
  executed (read-only review). P1-01's trigger (pause during an active wave) is
  the common usage case and the code path is deterministic; P2-01's trigger
  (real store fault mid-wave) is hard to provoke in practice — hence medium
  likelihood confidence despite high confidence in the behavior.
- The pause-during-wave behavior is established by code trace (framework
  select! branch + hardcoded `pending_outcome` + legal Paused→Cancelled
  transition), not dynamically reproduced; the proposed Q-FLT-02 fixture will
  pin it.
- EKO `resolve_dispatch`'s dispatch-error mapping (subagent hard
  failure/timeout → `Failed` without consuming the retry budget) was read as
  deliberate M7 review-gate policy and not raised; A-TSK-04 should confirm the
  intended retry surface.
- `drive_agent_run`'s L1 loop and `service.rs` background scheduler were
  inspected only at the boundary; their own lifecycle semantics belong to
  A-BOOT-01/A-CHAT-01/A-SRF-04.
- `TaskClaim::execution_id` and the `{run}:{task}:{revision}:{attempt}`
  identity are consumed as given (runtime.rs:218-224); identity end-to-end is
  A-TSK-04.

## Handoff

- Downstream tasks may rely on: one live DAG execution authority
  (`RuntimeDagExecutor`); EKO has no second ready frontier / retry state
  machine / stall loop / cancellation loop (V01-01, V02-01); `EkoRuntimeDagController`
  is a thin lossless policy adapter whose basic DAG behavior is tested and
  green (V04-02); the controller boundary mishandles three outcome classes —
  pause-during-wave → permanent cancel (P1-01), mid-wave fault → orphaned
  Running claims with no in-process recovery (P2-01), per-task cancel dead and
  semantically mismatched (P2-02).
- Reports to read: the 8 validation reports above; dependency reports A-TSK-01
  (file authority; torn tail P1-01 is a P2-01 trigger) and F-TSK-03 (framework
  origin of P2-01/P2-02; P3-01/P3-02 cleanup).
- Stale conditions: this report becomes stale if `runtime_executor.rs` wave/
  cancel/stall logic, EKO `executor.rs` drain/controller/terminal mapping,
  EKO `store.rs` cancel/pause/claim/recovery, the run transition table
  (`types.rs:517-527`), or the `TaskUpdateOperation` op set change; also if a
  `cancel_task` caller appears (P2-02's "dead" claim weakens, its "mismatched"
  claim strengthens) or `interruption_outcome` starts being consulted
  in-wave (P1-01 fixed).
- Follow-up task IDs: A-TSK-04 (per-task vs run-level cancel decision,
  orphaned-claim recovery, P1-01 fix validation), X-TSK-01 (cross-repo
  conformance of the cancel/pause contract), Q-FLT-02 (pause-during-wave and
  mid-wave fault fixtures), S-RDM-01 (roadmap: P1-01 pause fix, P2-01
  error-path cleanup, P2-02 surface decision).
