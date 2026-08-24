# A-TSK-04: Claims, revisions, recovery, and terminal monotonicity

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Can stale revisions/attempts, cancellation, restart, and event replay update
state only through valid claims without terminal regression?

**Answer: Yes in the steady state — the claim protocol is real and enforced:
`TaskClaim {revision, attempt, spec_hash}` is durably persisted in
`events.jsonl`, survives the event-sourced rebuild, gates every executor
terminal write (`Superseded` on mismatch), and the run state machine
(`can_transition_to`) makes Cancelled/Completed hard-terminal. Stale
revisions are rejected at both the tool boundary and the claim CAS, and
boot recovery is claim-aware. However, four scenario classes violate the
"only through valid claims" and "no terminal regression" guarantees: (P1)
a pause issued while a wave is in flight is converted into a permanent
cancellation — the durable Paused run is force-transitioned to terminal
Cancelled (independent re-verification of A-TSK-03-P1-01); (P1) a mid-wave
store fault aborts the run and leaves sibling Running claims durable with
no in-process recovery — same-process resume hangs in external polling,
only a restart heals (re-verification of A-TSK-03-P2-01); (P1) a torn final
line in `events.jsonl` makes that run permanently unreadable and unwritable
so replay/recovery never begin (re-verification of A-TSK-01-P1-01); (P2)
`block_task` and the terminal finalizers write through the unguarded public
`set_task_status`, contradicting the documented "every block write carries
the claim" and exposing a cross-process dual-driver claim overwrite; (P3)
hook events enqueued but not fired at crash are never replayed at boot.**

## Scope

Primary source paths inspected (deep read):

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs`
  (full production code): `transition_run` (:387-428), `request_cancel`/
  `request_pause` (:577-622), `compare_and_commit_revisioned_task_graph`
  (:755-885), `set_task_status` (:953-983), `claim_task` (:986-1029),
  `set_claimed_task_status` (:1032-1062), `requeue_claimed_task`
  (:1066-1105), `task_claim_is_current` (:1107-1121),
  `retry_blocked_task` (:1220-1340), `recover_incomplete` (:1631-1776),
  `active_subagent_boundaries`/`active_tool_boundaries` (:1528-1594),
  `list_recovery_blockers`/`resolve_recovery_task` (:2081-2179),
  `recoverable_subagent_result` (:2039-2078).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/event_rebuild.rs`
  (full): the seq-order fold, claim restore, revision-commit execution merge.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs`
  (full): the `events.jsonl` ledger (`append_event_line`/`next_seq`/
  `read_events`/`rewrite_plan`), per-run write lock, torn-tail behavior.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_store.rs`
  (:37-222): projection reads, `list_events` seq delta, Todo status authority.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/hook_event_dispatcher.rs`
  (full): append-side translation, bounded queue, no replay path.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs`
  (:321-585 drain loop + terminal mapping, :643-661 finalize, :1147-1620
  controller, :1348-1562 `resolve_dispatch`, :1622-1683
  `execute_runtime_plan`, :1564-1580 `block_task`).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/task_execute_tool.rs`
  (:55-127 per-run execution lock, :236-417 stale-revision rejection).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs`
  (:464-528 `TaskRunStatus`/`can_transition_to`, :922-929
  `EkoTaskExecution`, :1257-1272 `TaskUpdateOperation`).
- Framework: `echo-agent/echo-orchestration/src/tasks/runtime.rs`
  (:197-224 `TaskClaim`/`execution_id`), `runtime_executor.rs` (:196-449
  loop, :352-366 claim, :390-416 in-wave cancel, :416-447 outcome
  resolution).

## Out Of Scope

- File authorities / adapter losslessness — A-TSK-01 (its P1-01 torn-tail is
  re-verified here only as an event-replay blocker; P2-01 read-side rebuild
  gap is consumed, not re-reviewed).
- Controller boundary policy (pause/cancel ownership, review gates, retry
  budget) — A-TSK-03 (its P1-01/P2-01 are the objects of cross-verification).
- Authoring tools `task_create/update/list` — A-TSK-02; frontend projections
  and hooks at the surface — A-FE-01/02; worktree recovery — A-TSK-05;
  durable result/review content — A-TSK-06.
- Framework executor internals beyond claim/cancel/reload behavior —
  F-TSK-03.

## Inputs

- Root `AGENTS.md` (single task-relation authority; adapter thin-and-lossless;
  framework-vs-app layering; UTF-8/panic safety; claim/ledger pre-gate).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (A-TSK-04 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read: `A-TSK-03` (complete; P1-01/P2-01 re-verified
  here), `A-TSK-01` (complete; P1-01 re-verified here).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/
  MASTER-PLAN.md:120-235`, `echo-agent-cli/docs/2026-07-27-runtime-dag-kernel-
  convergence.md:140-230` (classified in V05-01).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correctly placed) | `TaskClaim`/`RuntimeTaskClaimOutcome`/`RuntimeTaskResolution` + `TaskExecution.claim` (runtime.rs:197-247); `RuntimeDagExecutor` safe points, bounded waves, claim/reload, in-wave cancel, stall/poll (runtime_executor.rs:196-449). Legacy `TaskExecutor` claim machinery (tasks/executor.rs:1628-1647) is a second definition but production-unreachable — framework pub API retained (F-TSK-01-P3-01 deletion target), not a live authority (V01-01). |
| EKO product policy (application) | `events.jsonl` ledger + `plan.json`/`run-state.json` projections, claim persistence in event payloads, boot recovery, pause/cancel control API, retry surfaces (`requeue_claimed_task`/`retry_blocked_task`), recovery blockers, durable-result reuse keyed by `execution_id`. |
| Adapter boundary | `EkoRuntimeDagController` maps load_snapshot/claim/resolve 1:1 to the store CAS; executor terminal writes are claim-guarded through `set_claimed_status`/`claim_is_current`. **Deviations**: the controller's `block_task` callback and EKO's terminal finalizers call the unguarded `set_task_status` (P2-01), and the in-wave cancellation outcome is hardcoded Cancelled on the framework side (P1-01) — behavior defects in the boundary, not a second authority. |
| Duplicate search (V01-01) | Terms: `TaskClaim`, `RuntimeTaskClaimOutcome`, `claim_task`, `set_claimed_task_status`, `requeue_claimed_task`, `task_claim_is_current`, `ClaimWriteOutcome`, `execution_id`, `stable_hash`, `ManagedTaskDagController`, `TaskExecutor`, `rebuild_plan_from_events`, `RebuiltPlan`, `replay`, `seq_cache`, `append_event_line`, `read_events`, `rewrite_plan`, `worker`. Result: one live claim authority, one ledger, one rebuild consumer; legacy executor claim code unreachable; zero `worker` terms. |
| Migration deletion | New targets from this task: none beyond A-TSK-03's — the unguarded `set_task_status` surfaces used by `block_task`/finalizers should either be moved to claim-aware helpers or explicitly documented as system-override (P2-01); the per-task cancel token machinery (A-TSK-03-P2-02) remains a deletion candidate. |

## Current Path

Verified data flow (V01-01/V02-01/V03-01):

1. Claim creation: `claim_task` CAS (plan revision + Pending + spec equality;
   mismatch -> `ReloadSnapshot`) builds `TaskClaim {revision, attempt =
   retry_count+1, spec_hash}` and persists it in the `TaskStarted` event
   payload (store.rs:1010-1026, :1169).
2. Claim persistence: `append_event_line` -> `events.jsonl` (seq + payload,
   per-run write lock, fsync; file_shadow.rs:118-178) -> `rewrite_plan`
   rebuilds `plan.json`/`run-state.json` from the full stream
   (file_shadow.rs:208-280; event_rebuild.rs fold restores `claim` at
   :229-231 and preserves execution+claim across revision commits at
   :147-162; skipped/reset ids clear the claim at :183-193).
3. Claim consumption: guard writes (`set_claimed_task_status`,
   `requeue_claimed_task`, `task_claim_is_current`, store.rs:1032-1121);
   framework reload on `ReloadSnapshot` (runtime_executor.rs:352-366);
   execution id `{run}:{task}:{revision}:{attempt}` for durable-result
   reuse (executor.rs:1292-1295) and recovery blockers (store.rs:1665-1674,
   :2039-2078).
4. Stale revision rejection: `task_execute` requires the exact latest
   revision (task_execute_tool.rs:277-282); `task_update` CAS on revision
   (store.rs:786-792); terminal runs reject plan modification
   (store.rs:764-772).
5. Cancellation: run-scoped driver token (store.rs:532-548, :577-622);
   framework in-wave cancel with grace then abort
   (runtime_executor.rs:390-416); EKO Cancelled branch ->
   `finalize_cancelled_run_state` (executor.rs:499-508, :643-661).
6. Pause: `request_pause` transitions Running->Paused **first**, then
   cancels the same token (store.rs:598-622); framework consults
   `interruption_outcome` only at loop top (:207-209), cancelled-task
   check (:267-269), and external-poll branch (:278-281) — never in-wave.
7. Restart recovery: `recover_incomplete` (store.rs:1631-1776) moves
   Running runs to Paused, resets Running tasks to Pending (replay-safe) or
   Blocked + `RecoveryBlocked` (non-replay-safe boundary), reuses completed
   Subagent results by persisted claim-derived execution id; resume
   (Paused->Running, store.rs:434-448) re-enters the executor which skips
   Completed tasks via the ready frontier.
8. Event replay ordering: single per-run append lock => file order == seq
   order; fold and projections deterministic; Todo status authority is
   run-state.json, older Task events only restore display metadata
   (file_store.rs:190-199).

## Findings

### A-TSK-04-P1-01: Pause during an active wave is converted into a permanent run cancellation — the durably-Paused run regresses to terminal Cancelled (independent re-verification of A-TSK-03-P1-01)

- Priority: P1
- Confidence: high (code path deterministic; pause-during-wave is the common
  usage case — waves run for seconds to minutes)
- Layer: adapter (EKO) with framework behavior mismatch
- Evidence: framework in-wave cancel branch
  `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:390-416`
  (`cancel.cancelled()` -> grace -> `abort_all`, then
  `:416 let mut pending_outcome = cancellation_observed.then_some(RuntimeDagOutcome::Cancelled)`);
  `interruption_outcome` is consulted only at `:207-209`, `:267-269`,
  `:278-281` — never inside the wave; EKO `interruption_outcome` would
  return Paused for a durably-Paused run (`executor.rs:1595-1613`);
  `request_pause` transitions Running->Paused **before** cancelling the
  shared token (`store.rs:598-622`, comment "The executor observes the
  durable Paused status and leaves the run resumable"); EKO Cancelled
  branch unconditionally runs `finalize_cancelled_run_state`
  (`executor.rs:508`, `:643-661`) which flips Pending/Running/Blocked tasks
  to Cancelled and force-transitions the Paused run to Cancelled — legal
  per `types.rs:522` (`Paused => matches!(next, Running | Cancelled)`);
  `resolve_dispatch`'s dispatch-error branch additionally writes
  `TodoStatus::Cancelled` for aborted in-flight dispatches
  (`executor.rs:1356-1381`).
- Reachability: GUI pause button -> `src/tauri/commands/task_runtime.rs:463`
  `request_pause`; TUI -> `src/tui/events.rs:4346`; service.rs:465. A pause
  issued while Subagents are running lands inside the wave-drain `select!`
  in the common case. The existing test
  `runtime_plan_cancellation_preserves_explicit_pause` (executor.rs:5757-5783)
  seeds the run Paused before a pre-cancelled token (loop-top path) and
  never covers pause-during-wave.
- Expected invariant: pause is durable and resumable regardless of wave
  activity; every cancellation path consults the controller's
  `interruption_outcome`; a Paused run is never force-transitioned to
  Cancelled; replay of the event stream must not make the regression
  invisible.
- Observed behavior: `request_pause` during a wave -> framework returns
  `Cancelled` -> `finalize_cancelled_run_state` transitions the durable
  Paused run to Cancelled and marks tasks Cancelled -> run terminal, not
  resumable; the event stream faithfully records
  `RunStatusChanged(Paused)` then `RunStatusChanged(Cancelled)` — the
  replay reproduces the regression, so the defect is the state machine
  allowance plus the hardcoded outcome, not the fold.
- Impact: the documented pause contract (store.rs:595-597) is violated on
  the path users actually use; pause kills in-flight work, flips tasks to
  terminal Cancelled (no requeue op exists in EKO's task_update surface,
  types.rs:1257-1272; `retry_blocked_task` rejects Cancelled,
  store.rs:1244-1251), and loses the run.
- Root cause: the framework hardcodes `Cancelled` for in-wave cancellation
  instead of asking the controller (`interruption_outcome` exists precisely
  for this), and EKO's `finalize_cancelled_run_state` ignores the durable
  Paused status; the pause/cancel share one token so they are
  indistinguishable in-wave.
- Direction: (a) framework — after the wave drain, if
  `cancellation_observed`, consult `controller.interruption_outcome(run_id)`
  instead of hardcoding `Cancelled`; (b) EKO — `resolve_dispatch`'s
  dispatch-error branch writes `Pending` (not `Cancelled`) when the run is
  durably Paused; (c) EKO — `finalize_cancelled_run_state` skips runs whose
  durable status is Paused (or is replaced by the Paused branch's
  Running->Pending reset). No deletion needed; the framework hook already
  exists. Canonical finding: A-TSK-03-P1-01 (fix validates both).
- Regression validation: framework fixture "cancel fires mid-wave while
  `interruption_outcome` returns Paused -> outcome Paused, sibling claims
  resolved, completed siblings preserved"; EKO fixture "`request_pause`
  mid-dispatch leaves the run Paused and resume re-dispatches without
  replaying completed tasks" (Q-FLT-02).
- Validation reports: [V03-01](../validations/A-TSK-04/V03-01.md),
  [V04-02](../validations/A-TSK-04/V04-02.md), [V04-08](../validations/A-TSK-04/V04-08.md)

### A-TSK-04-P1-02: A mid-wave store fault leaves sibling Running claims durable with no in-process recovery; same-process resume hangs in external polling (independent re-verification of A-TSK-03-P2-01)

- Priority: P1
- Confidence: high (code path) / medium (trigger probability — requires a
  real store fault mid-wave, e.g. A-TSK-01-P1-01's torn `events.jsonl` tail)
- Layer: adapter (EKO integration of the framework defect)
- Evidence: wave closure propagates semaphore/`claim_task` errors with `?`
  (`runtime_executor.rs:348-365`), `:379-381` (`Some(Ok(Err(error))) =>
  return Err` aborts the wave), `:418-421` (`resolve_dispatch(...).await?`
  drops remaining wave results), JoinSet drop aborts sibling dispatches;
  EKO `execute_run`'s Err branch marks the run Failed with **no** task
  cleanup (`executor.rs:570-582`) — asymmetric with the cancel path
  (`finalize_cancelled_run_state`) and the pause path (Running->Pending);
  orphaned claims persist durably (claim in TaskStarted event, rebuilt —
  V02-01) and nothing clears them in-process: `retry_blocked_task` rejects
  Running (store.rs:1244-1251), `TaskUpdateOperation` has no `SetStatus`
  (types.rs:1257-1272), `EkoTaskToolPolicy` does not enable
  `allow_manual_progress_updates` (framework default false), `resume_task_run`
  requires Paused (store.rs:434-448) and a re-launched executor sees the
  orphaned Running tasks as in-flight and polls
  `external_progress_poll_interval` forever (runtime_executor.rs:276-285);
  only boot-time `recover_incomplete` (store.rs:1631-1776; test
  `boot_recovery_requeues_orphaned_running_task` store.rs:2721) resets them.
- Reachability: `task_execute`/resume -> `execute_run` ->
  `execute_runtime_plan` -> framework wave; triggered when
  `claim_task`/`resolve_dispatch`/store reads return a real error mid-wave
  (torn tail, corrupt projection, poisoned run lock, missing plan).
- Expected invariant: a per-task persistence/controller fault must not
  corrupt sibling state; the run must end in a state a same-process retry
  can recover from; event replay must not strand durable Running claims.
- Observed behavior: `execute()` returns `Err` -> EKO marks the run Failed
  while siblings stay `Running` with durable claims and no terminal event;
  a same-process resume (Failed->Running is legal per types.rs:523) re-enters
  the executor which polls forever; only a process restart (boot recovery)
  or manual file edits recover the run.
- Impact: one transient store fault during a wave turns a run that the
  `ReloadSnapshot` mechanism was designed to survive into a permanent
  hang until restart, with sibling claims in a state that is neither
  terminal nor in-process-recoverable.
- Root cause: the framework wave closure conflates "claim conflict"
  (graceful ReloadSnapshot) with "claim fault" (abort) and never drains or
  abandons sibling claims on error (F-TSK-03-P2-01), and EKO's terminal
  mapping reuses its cleanup machinery only on the cancel/pause outcomes.
- Direction: (a) EKO — on `execute_runtime_plan` Err, run the same
  Running-task reset used by the pause path before marking the run Failed;
  (b) framework — drain the JoinSet and resolve/abandon sibling claims on a
  wave error, or treat claim faults like `ReloadSnapshot` after N bounded
  attempts. Delete nothing. Canonical finding: A-TSK-03-P2-01 /
  F-TSK-03-P2-01.
- Regression validation: EKO fixture "poisoned run lock mid-wave -> run
  Failed with zero Running survivors"; EKO fixture "resume of a Failed run
  with an orphaned Running task completes after boot recovery" (Q-FLT-02).
- Validation reports: [V03-01](../validations/A-TSK-04/V03-01.md),
  [V04-01](../validations/A-TSK-04/V04-01.md)

### A-TSK-04-P1-03: A torn final line in `events.jsonl` bricks both reads and writes, so event replay and recovery never begin (independent re-verification of A-TSK-01-P1-01)

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `file_shadow.rs:362-379` (`read_events` hard-errors
  `ShadowError::Decode` on any malformed line), `:289-299` (`next_seq`
  seeds from the same read, so appends fail too), `:114-117` (comment
  promises a future truncation pass — grep for truncate/partial-tail repair
  in `task_runtime/` returns zero hits), `store.rs:1635-1638`
  (`recover_incomplete` warns and skips the run).
- Reachability: crash mid-`append_line` (OOM kill, power loss) leaves a torn
  final line -> every read (`get_run`/`get_plan`/`list_todos`/CAS load via
  file_store.rs:37-55) and every write (`next_seq`) of that run fails ->
  boot `recover_incomplete` sees the zombie run every restart but can never
  transition it (transition_run -> get_run -> read_events -> Err).
- Expected invariant: the append-only ledger tolerates a torn final line
  (the project's own stated intent, file_shadow.rs:114-117): reads succeed
  with the partial event dropped and seq continues from the last valid
  event.
- Observed behavior: a torn tail permanently hides the run's data from the
  app; event replay (the fold) can never run because the event stream
  itself is unreadable; only manual file editing recovers the run.
- Impact: one crash during any event append can permanently brick the run —
  no claim, recovery, or replay path can execute; A-TSK-01-P1-01's executed
  reproduction (`ERROR line 3: EOF while parsing`) is unchanged here.
- Root cause: `append_event_line` is not crash-atomic (plain O_APPEND write
  with no framing) and the read side has no tail-repair logic.
- Direction: on a decode error of the FINAL line, truncate `events.jsonl`
  to the last valid line boundary and rebuild; or prefix lines with length
  and skip a truncated final record; add a torn-tail regression test.
  Canonical finding: A-TSK-01-P1-01.
- Regression validation: fixture "events.jsonl with torn final line still
  yields get_run/get_plan and the next append gets the correct seq"; boot
  recovery test with a torn-tail run (Q-FLT-02).
- Validation reports: [V03-01](../validations/A-TSK-04/V03-01.md),
  [V04-04](../validations/A-TSK-04/V04-04.md)

### A-TSK-04-P2-01: `block_task` and the terminal finalizers bypass the claim guard through the public unguarded `set_task_status`; the documented "every block write carries the claim" is false

- Priority: P2
- Confidence: medium (mechanism high; trigger requires cross-process
  dual-driver, the A-TSK-01-P3-02 edge)
- Layer: adapter (EKO store surface) with documentation drift
- Evidence: `executor.rs:1564-1580` (`block_task` -> `store.set_task_status(
  ... Blocked ...)`); `set_task_status` (store.rs:953-983) performs **no**
  status/claim precondition check — it writes any status to any task and
  clears the claim (`claim: None` at store.rs:980); other live callers:
  pause reset (executor.rs:551), `finalize_cancelled_run_state`
  (executor.rs:651), `recover_incomplete` (store.rs:1706),
  `resolve_recovery_task` (store.rs:2163/2170); convergence doc:153 claims
  "Every completion, failure, **block**, and retry write carries the claim"
  (V05-01: stale/regressed). Within one driver the per-run execution lock
  (task_execute_tool.rs:55-127) and the framework's join-before-block
  ordering make the unguarded writes safe; across processes the write locks
  and seq caches are process-local (file_shadow.rs:26-34; A-TSK-01-P3-02),
  so driver B's `block_task` can overwrite driver A's live Running claim
  with Blocked; A's terminal write then returns Superseded
  (store.rs:1048) and the completed dispatch result is silently discarded.
- Reachability: framework loop top calls `block_task` whenever a failed
  task exists and has non-completed dependents (runtime_executor.rs:235-246);
  reachable from every driver; harmful only when a second process drives
  the same run.
- Expected invariant: either every state write is claim-guarded, or the
  unguarded override surface is explicit, documented, and cannot hit a
  live claim.
- Observed behavior: the unguarded surface exists as a public store API,
  is used by live controller callbacks, and is reachable in a race that
  discards completed work without an error.
- Impact: silent completed-work loss under concurrent GUI+TUI use of one
  run (the P3-02 dual-process edge); misleading public API and false
  historical documentation for single-process users.
- Root cause: `block_task` predates the claim protocol and was never
  migrated to a claim-aware write; the docs were written for the claim
  protocol's intent rather than the implementation.
- Direction: (a) keep a single claim-guarded write path and route
  `block_task` through a preconditioned store method that returns
  Superseded when a live claim exists (or explicitly abandons the claim
  first); (b) correct the convergence-doc wording; (c) the durable fix for
  the dual-process race remains A-TSK-01-P3-02's cross-process writer
  exclusion. Deletion target if fixed: none new; `set_task_status` remains
  the boot-recovery escape hatch by design.
- Regression validation: fixture "block_task while a live claim exists ->
  Superseded, terminal write of the original claim still applied or run
  outcome consistent"; cross-process fixture with two stores over one
  shadow root.
- Validation reports: [V03-01](../validations/A-TSK-04/V03-01.md),
  [V05-01](../validations/A-TSK-04/V05-01.md)

### A-TSK-04-P3-01: Hook events enqueued but not yet fired at crash are never replayed at boot — hook consumers can permanently miss persisted lifecycle events

- Priority: P3
- Confidence: medium (mechanism certain; impact depends on hook consumers)
- Layer: adapter (EKO hook bridge)
- Evidence: the hook fires synchronously after append
  (`file_shadow.rs:174-176`) into the bounded queue
  (`hook_event_dispatcher.rs:130-150`, capacity 256); the queue is
  in-memory; there is no boot-time replay of `events.jsonl` into hooks —
  `recover_incomplete` (store.rs:1631-1776) only appends Note/TodoUpdated
  events, and `TodoUpdated`/`Note` are not translated by the dispatcher
  (hook_event_dispatcher.rs:347-352, `_ => return Vec::new()`);
  `attach_hook_event_dispatcher` (store.rs:221-244) attaches the hook but
  never replays.
- Reachability: any process crash between append and consumer drain loses
  the fired-but-queued events; a restart recovers the run state
  (claims/projections intact) but hook consumers (e.g. external
  integrations observing TaskStarted/TaskCompleted) never see those
  transitions.
- Expected invariant: hook delivery is either durable (replay at boot) or
  documented as best-effort; a crash must not permanently desynchronize
  hook consumers from the authoritative event stream.
- Observed behavior: hook delivery is best-effort with no documented
  crash-loss semantics and no replay; recovery resets are invisible to
  hooks.
- Impact: for a local personal assistant, hook consumers (chat/status
  surfaces, user-configured YAML hooks) may silently miss lifecycle events
  after a crash; no data corruption.
- Root cause: the hook pipeline is append-side by design; crash
  consistency of the queue was not considered.
- Direction: either replay un-fired events into hooks at bootstrap (bounded
  by the last acknowledged seq) or document the best-effort contract in
  the dispatcher module docs; no deletion needed.
- Regression validation: fixture "store with N persisted events + a fresh
  dispatcher -> hooks fire the previously unfired transitions once" or a
  doc-only fix with a contract test.
- Validation reports: [V03-01](../validations/A-TSK-04/V03-01.md),
  [V04-06](../validations/A-TSK-04/V04-06.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search (claim authority; ledger naming; rebuild consumers; legacy executor reachability) | yes | passed | [V01-01](../validations/A-TSK-04/V01-01.md) |
| V02 | Claim identity persistence + registration/runtime reachability (append -> rebuild -> projection -> framework -> recovery) | yes | passed | [V02-01](../validations/A-TSK-04/V02-01.md) |
| V03 | Invariant/edge cases (stale write rejection; replay ordering; crash/restart/cancel/retry; terminal monotonicity; cross-checks) | yes | failed (5 findings) | [V03-01](../validations/A-TSK-04/V03-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::store` | yes | passed (exit 0, 34 ok) | [V04-01](../validations/A-TSK-04/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::executor` | yes | passed (exit 0, 46 ok) | [V04-02](../validations/A-TSK-04/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::event_rebuild` | yes | passed (exit 0, 3 ok) | [V04-03](../validations/A-TSK-04/V04-03.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::file_shadow` | yes | passed (exit 0, 9 ok) | [V04-04](../validations/A-TSK-04/V04-04.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::file_store` | yes | passed (exit 0, 5 ok) | [V04-05](../validations/A-TSK-04/V04-05.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::hook_event_dispatcher` | yes | passed (exit 0, 12 ok) | [V04-06](../validations/A-TSK-04/V04-06.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::types` | yes | passed (exit 0, 51 ok) | [V04-07](../validations/A-TSK-04/V04-07.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked tasks::runtime_executor` | yes | passed (exit 0, 7 ok) | [V04-08](../validations/A-TSK-04/V04-08.md) |
| V05 | Historical-document drift (MASTER-PLAN claim/recovery/replay sections; convergence doc; file_shadow comment) | conditional | passed | [V05-01](../validations/A-TSK-04/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:176-182: events.jsonl recovery authority; safe-point reload; completed attempts never restarted implicitly | current | file_shadow.rs:3-7,208-280; runtime_executor.rs:211-230; ready frontier runtime.rs:438-458 (V01-01/V03-01) |
| MASTER-PLAN:196-215: execution identity `{run}:{task}:{revision}:{attempt}`; run-state.json authoritative for Todo status; restart recovery receives identical evidence | current | runtime.rs:221-223; store.rs:1665-1674,2039-2078; file_store.rs:190-199 (V02-01) |
| MASTER-PLAN:229-235 + convergence:150-154: atomic TaskClaim; conflict reloads; completion/failure/retry writes accepted only for the still-running claim | current | store.rs:986-1121; runtime_executor.rs:352-366,422-426; tests store.rs:3060/3100 (V03-01/V04-01) |
| Convergence:153: "Every completion, failure, **block**, and retry write carries the claim" | stale/regressed | block_task writes unguarded (executor.rs:1564-1580, store.rs:953-983) -> P2-01 (V05-01) |
| Convergence:156-160: durable result lookup cannot reuse older-TaskSpec output | current | patched_spec test store.rs:3148; execution id includes revision (V02-01) |
| file_shadow.rs:114-117: torn partial tail tolerated/truncated by future hardening | regressed (unimplemented) | read_events hard-errors (file_shadow.rs:362-379) -> P1-03 (V03-01/V04-04) |
| store.rs:595-597: pause keeps the run resumable | regressed on the in-wave path | runtime_executor.rs:390-416 + executor.rs:643-661 -> P1-01 (V03-01) |
| A-TSK-03-P1-01 (pause-in-wave -> permanent cancel) | current (independent re-verification) | same evidence + event-replay/monotonicity angle -> P1-01 (V03-01) |
| A-TSK-03-P2-01 (mid-wave fault orphans sibling claims; only boot recovery heals) | current (independent re-verification) | runtime_executor.rs:379-381/418-421 + executor.rs:570-582 + in-process surface absent -> P1-02 (V03-01) |
| A-TSK-01-P1-01 (torn events.jsonl tail bricks the run) | current (independent re-verification) | file_shadow.rs:362-379,289-299 + store.rs:1635-1638 -> P1-03 (V03-01) |

## Coverage And Uncertainty

- All conclusions are static except the V04 test runs; no live LLM DAG run
  and no fault-injection run was executed (read-only review). P1-01's
  trigger (pause during an active wave) is the common usage case and the
  code path is deterministic; P1-02's trigger (real store fault mid-wave)
  is hard to provoke — hence medium likelihood confidence despite high
  confidence in the behavior.
- The pause-during-wave chain (framework select! branch -> hardcoded
  Cancelled -> finalize force-transition) is established by code trace;
  the existing test covers only the loop-top path (V04-02). The proposed
  Q-FLT-02 fixture will pin it.
- The claim protocol's remaining surfaces (review gate, worktree
  integration, `drive_agent_run` L1 loop) were inspected only at their
  store-call sites; their own lifecycle semantics belong to A-TSK-05/A-TSK-06.
- Cross-process dual-driver (GUI+TUI) behavior was not exercised (two live
  processes); P2-01's trigger rests on A-TSK-01-P3-02's documented absence
  of cross-process writer exclusion.
- `TaskClaim::execution_id` uniqueness across plan resets was checked
  structurally (revision + attempt both monotonic on all live paths) but
  not fuzzed.

## Handoff

- Downstream tasks may rely on: one live claim authority and one event-log
  ledger with deterministic replay (V01-01/V02-01); claim identity is
  persisted losslessly and gates every executor terminal write (V02-01,
  V03-01); run-terminal monotonicity is enforced by `can_transition_to`
  and the sole run-status write path `transition_run`; boot recovery is
  claim-aware and heals orphaned Running tasks; the four scenario
  violations above (P1-01 pause-in-wave, P1-02 mid-wave fault, P1-03 torn
  tail, P2-01 unguarded block writes) and the hook-crash gap (P3-01).
- Reports to read: the 11 validation reports above; dependency reports
  A-TSK-03 (canonical P1-01/P2-01) and A-TSK-01 (canonical P1-01/P2-01);
  F-TSK-03 (framework origin of the wave-abort defect).
- Stale conditions: this report becomes stale if `runtime_executor.rs`
  wave/cancel/reload logic, EKO `executor.rs` drain/terminal mapping or
  `block_task`, EKO `store.rs` claim/recovery/pause-cancel paths,
  `file_shadow.rs` append/read/seq behavior, `event_rebuild.rs` fold, or
  the `TaskRunStatus` transition table change; also if a production caller
  of the unguarded `set_task_status` on Running tasks appears, if
  `interruption_outcome` starts being consulted in-wave (P1-01 fixed), or
  if a read-side/torn-tail repair lands (P1-03 fixed).
- Follow-up task IDs: A-TSK-06 (review/artifacts consume the recovered
  claims), X-TSK-01 (cross-repo conformance of claim/cancel/replay),
  X-STA-01 (identity continuity across restart), Q-FLT-02 (pause-during-
  wave, mid-wave fault, torn-tail, dual-driver fixtures), S-RDM-01
  (roadmap: P1-01 pause fix, P1-02 error-path cleanup, P1-03 torn-tail
  repair, P2-01 claim-aware block_task, P3-01 hook replay or documented
  contract).
