# A-TSK-04: Claims, revisions, recovery, and terminal monotonicity

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Can stale revisions/attempts, cancellation, restart, and event replay update
state only through valid claims without terminal regression?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs`
  (3496 lines) — read in relevant slices:
  - module doc + store struct + `with_run_lock` (1-295) — per-run write
    serialization boundary;
  - `create_run` / `transition_run` / `resume_task_run` (302-448) — run-level
    state machine, including the recovery-blocker gate on resume;
  - `complete_run_if_quiescent` (453-488) — the run-level CAS that the
    executor's drain loop relies on;
  - task-level cancel tokens + `request_cancel` / `request_pause`
    (490-622) — the in-memory driver-token control surface;
  - `attach_plan_for_test` + `compare_and_commit_revisioned_task_graph`
    (624-885) — the optimistic-concurrency revision commit, including
    `expected_revision` CAS and terminal-run rejection;
  - `set_task_status` (953-983) — the **naked** status writer (no claim
    guard, no `TodoStatus` monotonicity check);
  - `claim_task` (986-1029) — the optimistic-concurrency claim with
    `expected_revision` + `attempt = retry_count + 1`;
  - `set_claimed_task_status` (1032-1062) — CAS on Running + claim identity;
  - `requeue_claimed_task` (1066-1105) — atomic Running→Pending flip +
    retry_count bump (the retry primitive);
  - `task_claim_is_current` (1107-1121), `append_task_status_event`
    (1123-1176) — the shared event-append path and the
    typed-status→event-kind mapping;
  - `retry_blocked_task` (1220-1367) — the user-initiated retry path
    (Paused/Failed → Running under the per-run lock, including the
    descendant-unblock sweep);
  - `transition_run_locked` (1372-1401) — re-entrant variant for use
    inside `with_run_lock`;
  - `active_subagent_boundaries` / `active_tool_boundaries`
    (1528-1594) — the event-fold that classifies post-crash tasks;
  - `record_recovery_blocker` / `recover_incomplete`
    (1596-1776) — the boot-time crash recovery sweep;
  - `list_recovery_blockers` / `resolve_recovery_task`
    (2081-2179) — the fail-closed barrier and its user-driven resolution;
  - `recoverable_subagent_result` (2039-2078) — the durable-result fold
    keyed on `(task_id, execution_id)`;
  - 34-test `tests` module (2204-3496), with particular attention to:
    `illegal_transition_is_rejected_and_leaves_no_event` (2300),
    `task_terminal_events_follow_typed_status_not_detail_text` (2394),
    `resume_task_run_transitions_paused_to_running` (2551),
    `boot_recovery_pauses_run_and_preserves_completed_tasks` (2677),
    `boot_recovery_requeues_orphaned_running_task` (2721),
    `boot_recovery_reuses_completed_subagent_without_redispatch` (2738),
    `mutating_in_doubt_subagent_blocks_resume_until_user_decides` (2810),
    `blocked_todo_restores_barrier_if_resolution_crashes_before_mutation`
    (2903), `claim_reloads_when_task_update_wins_revision_race` (3060),
    `stale_claim_cannot_overwrite_cancelled_task` (3100),
    `patched_spec_uses_new_execution_identity_without_retry_bump` (3148),
    `completion_gate_rechecks_latest_plan_revision` (3307),
    `task_update_rejects_stale_revision_without_appending_event` (3039),
    `file_path_rejects_illegal_transition_and_appends_no_event` (3374),
    `file_path_rejects_dependency_cycle_and_appends_no_event` (3398).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/event_rebuild.rs`
  (501 lines) — read in full: the deterministic event-fold that rebuilds
  `plan.json` / `run-state.json` from `events.jsonl`, including the
  `PlanRevisionCommitted` task-execution carry-over and the
  skipped/reset-task projection. Three-test suite confirms run/plan/task
  parity after a full lifecycle and after a task patch.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/hook_event_dispatcher.rs`
  (810 lines) — read in full: the bounded single-consumer queue that
  translates the persisted `RuntimeEventKind` stream into framework
  `HookEvent`s in persisted order. The 12-test suite confirms the
  terminal-status distinction (Cancelled/TimedOut not collapsed to
  Failed/Skipped) and the backpressure-without-dropping invariant.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/ledger.rs`
  (291 lines) — read in full: confirmed it is a **pure read-only**
  progress-markdown export; it has no write path, no claim semantics, no
  terminal authority. The module doc (1-9) explicitly states "the
  markdown export is a derived recovery view; run events and plan files
  remain authoritative."
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs`
  (1-200) — confirmed the framework `RevisionedTaskStore` adapter is a
  thin persistence shell over `load_revisioned_task_graph` /
  `compare_and_commit_revisioned_task_graph`; no patch or validation
  logic (consistent with A-TSK-03's adapter-thinness conclusion).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs`
  (1-436) — read in relevant slices:
  - `append_event_line` (118-178) — the single file-authority write
    primitive, holding the per-run write lock across seq-alloc → append →
    cache-bump → event-hook fire;
  - `rewrite_plan` (208-280) — the deterministic projection rebuild that
    re-acquires the same per-run write lock and skips work when the
    latest event affects neither `plan.json` nor `run-state.json`;
  - `atomic_write` (405-422) — the unique-tmp + fsync + rename primitive;
  - `append_line` (427-436) — `O_APPEND` + `sync_all`;
  - 9-test suite confirms per-run seq isolation, concurrent-append strict
    monotonicity, concurrent-atomic-write no-tmp-collision, and that
    rewrite_plan re-acquires the same-run write lock.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs`
  (365-528) — `TodoStatus` (no `can_transition_to`), `TaskRunStatus`
  (`can_transition_to` enforces terminal monotonicity for
  Cancelled/Completed), `RuntimeEventKind` enum.
- Framework contract:
  - `echo-agent/echo-orchestration/src/tasks/runtime.rs:177-224` —
    `TaskSpec::stable_hash` (SHA-256 over the canonical JSON),
    `TaskClaim { revision, attempt, spec_hash }`,
    `TaskClaim::execution_id` = `{run_id}:{task_id}:{revision}:{attempt}`,
    `TaskStatus::can_transition_to` (framework's richer state machine,
    including `Retrying`/`Paused` which EKO never persists per
    A-TSK-03-P3-02);
  - `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:38-78,
    320-447` — the kernel's claim/dispatch/resolve loop, including the
    fact that `RuntimeTaskClaimOutcome::ReloadSnapshot` is a retry signal
    (not an error) and `RuntimeTaskResolution::Pending` is the kernel's
    "the controller requeued this task; re-claim on the next safe point"
    channel;
  - `echo-agent/echo-orchestration/src/tasks/revisioned.rs:160-258` —
    `TaskGraphCommit`, `RevisionedTaskStore` trait (load +
    compare_and_commit), `RevisionedTaskStoreError::Conflict`.
- Cross-repo duplicate search (V01) for `TaskClaim`, `execution_id`,
  `claim_task`, `set_claimed_task_status`, `requeue_claimed_task`,
  `compare_and_commit_revisioned_task_graph`, `recover_incomplete`,
  `list_recovery_blockers`, `recoverable_subagent_result`,
  `RecoveryBlocked`, `RecoveryResolved`, `append_event_line`,
  `rewrite_plan`, `RevisionedTaskStore`, `PlanValidator` across the
  whole `echo-agent-cli` repository.

## Out Of Scope

Deferred to downstream tasks:

- **A-TSK-05**: worktree, file ownership, and merge policy. The
  file-write authority exercised by `integrate_reviewed_task` is the
  seam this task stops at.
- **A-TSK-06**: task review, artifacts, and parent context. The
  review-gate outcome that drives `RuntimeTaskResolution::Blocked` is
  consumed here only as a state-transition trigger.
- **F-TSK-03** (already complete) owns the framework-level claim /
  cancel / stall semantics. This task only verifies that EKO's
  application-layer persistence obeys them. In particular,
  F-TSK-03-P2-01 (in-flight stall timeout) and F-TSK-03-P2-02
  (abort-orphan reconciliation) remain framework-owned; A-TSK-03
  established that EKO mitigates P2-02 at the application layer via
  `finalize_cancelled_run_state`, and this task confirms that
  mitigation holds for the recovery path too.
- The internal mechanics of the LLM reviewer gate (`run_review_gate`)
  and the worktree-merge step (`integrate_reviewed_task`) — these are
  the product-policy decisions behind Blocked/Completed resolutions,
  audited only as the state-transition producers.
- A-TSK-03-P3-01's drain-loop race is referenced where it interacts
  with recovery, but the fix remains owned by that task.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — rule 6 (single task-relationship authority);
    the framework-vs-application layering gate; the "adapter must stay
    thin" rule; UTF-8 / panic safety; the cleanup rule ("delete over
    retain"); "本地个人助理" threat model (over-engineering defenses
    is the standing hazard).
  - `docs/comprehensive-review/REPORTING.md`,
    `docs/comprehensive-review/templates/{task-report,validation-report}.md`,
    `docs/comprehensive-review/TASKS.md` (A-TSK-04 spec).
- Dependency task reports read:
  - **A-TSK-03** (complete) — established the controller is thin, the
    eight callbacks inject only product policy, the post-outcome
    sweeps (`finalize_cancelled_run_state`, the Paused→Pending sweep)
    mitigate F-TSK-03-P2-02 at the application layer, and
    A-TSK-03-P3-02 resolved A-TSK-01-P2-02 as latent (EKO never
    produces framework `Retrying`/`Paused`). This task relies on the
    controller boundary A-TSK-03 drew and answers the handoff item:
    "A-TSK-04 → may rely on the controller's reconciliation sweeps
    being in place; must verify the resume path correctly re-dispatches
    the Running→Pending reset tasks and that
    `complete_run_if_quiescent`'s CAS is sound under concurrent plan
    patches."
  - **F-TSK-03** (complete) — the framework kernel authority for claim,
    cancel, retry-as-Pending, and failure propagation. This task
    confirms EKO's persistence obeys the kernel contract.
  - **F-TSK-01** (complete) — the canonical framework `Task`/
    `RevisionedTaskStore` model.
  - **F-TSK-02** (complete) — `PlanValidator` is the sole structural
    DAG validator. This task confirms EKO's commit adapter delegates
    to it and adds no second validator.
  - **A-TSK-01** (complete) — file authority and the
    `TodoStatus`/`TaskStatus` projection boundary. A-TSK-01-P2-02
    (lossy projection) was resolved latent by A-TSK-03-P3-02; this
    task re-verifies that the executor/recovery paths still never
    produce the lossy statuses.
- Historical documents treated as hypotheses: the store.rs module doc
  (1-9), the `with_run_lock` doc (274-280), the `claim_task` doc
  (985), the `requeue_claimed_task` doc (1064-1066), the
  `recover_incomplete` doc (1620-1627), the
  `recoverable_subagent_result` doc (2036-2038), the
  `list_recovery_blockers` doc (2080), the `resolve_recovery_task`
  doc (2132-2151), and the `file_shadow::append_event_line` doc
  (105-117). All verified below.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Confirmed framework-owned: the `TaskClaim` identity contract (`{revision, attempt, spec_hash}` with `execution_id = {run_id}:{task_id}:{revision}:{attempt}`), the kernel's optimistic-concurrency claim loop (`RuntimeTaskClaimOutcome::ReloadSnapshot` as the retry signal, `RuntimeTaskResolution::Pending` as the requeue channel), the `RuntimeDagOutcome` terminal set (Completed/Failed{task}/Paused{task}/Cancelled), the `TaskStatus::can_transition_to` table, and `PlanValidator` structural DAG validation. All live in `echo-orchestration::tasks`. EKO touches them only through the controller callbacks (A-TSK-03 V01). |
| EKO product policy | Confirmed app-owned: the file-backed event authority (`events.jsonl` + `plan.json`/`run-state.json` projections), the per-run write lock (`with_run_lock` 281-290), the 6-state `TaskRunStatus` machine and its `can_transition_to` table (types.rs:517-527), the `TodoStatus` UI projection (8 states, no monotonicity table), the recovery sweep policy (`recover_incomplete` 1631-1776: Running→Paused, Running todo → Pending/Blocked by subagent/tool boundary), the fail-closed `RecoveryBlocked`/`RecoveryResolved` barrier, the durable-result reuse policy (`recoverable_subagent_result` keyed on `execution_id`), the retry budget (`max_retries` checked in `resolve_dispatch` and `retry_blocked_task`), and the user-driven retry/skip decision (`resolve_recovery_task`). |
| Adapter boundary | `EkoRevisionedTaskStore` (revisioned_adapter.rs:26-56) is a thin persistence shell: it delegates `load`/`compare_and_commit` to the store with only error mapping (58-76). No patch logic, no validation, no scheduling. `EkoRuntimeDagController`'s claim/resolve callbacks (audited in A-TSK-03) call `claim_task`/`set_claimed_task_status`/`requeue_claimed_task` directly; this task confirms those calls obey the kernel's `ReloadSnapshot`/`Superseded` contract. |
| Duplicate search | Searched names (whole `echo-agent-cli` repo): `TaskClaim`, `execution_id`, `claim_task`, `set_claimed_task_status`, `requeue_claimed_task`, `task_claim_is_current`, `compare_and_commit_revisioned_task_graph`, `recover_incomplete`, `list_recovery_blockers`, `resolve_recovery_task`, `recoverable_subagent_result`, `RecoveryBlocked`, `RecoveryResolved`, `append_event_line`, `rewrite_plan`, `RevisionedTaskStore`, `PlanValidator`, `TodoStatus`, `TaskRunStatus`, `RuntimeEventKind`. Result: ONE `RevisionedTaskStore` impl (`EkoRevisionedTaskStore`); ONE production `TaskClaim` construction site (inside `claim_task` at store.rs:1010, plus test fixtures using the struct literal); ONE `recover_incomplete` definition; ONE `compare_and_commit_revisioned_task_graph` definition; ONE `append_event_line` write primitive (file_shadow.rs:118); ZERO second validators / second event streams / second claim authorities in `echo-agent-cli`. V01. |
| Migration deletion | No migration proposed. Two P3 cleanup recommendations (P3-01, P3-02); both are localized hardening, not authority moves. |

## Current Path

Verified claim/revision/recovery/terminal flow at commits
`echo-agent` 9b0e0fa / `echo-agent-cli` b3b2e81.

### Claim identity and persistence

```text
claim_task(run_id, expected_task, expected_revision)             [store.rs:986]
  under with_run_lock(run_id):
    plan = get_plan(run_id)                       [file authority read]
    if plan.revision != expected_revision:
        return RuntimeTaskClaimOutcome::ReloadSnapshot            [:996-998]
    task = plan.tasks.find(id == expected_task.spec.id)
    if task.status != Pending || task.spec != expected_task.spec:
        return ReloadSnapshot                                      [:1007-1009]
    claim = TaskClaim {
        revision: expected_revision,
        attempt: task.retry_count + 1,           // saturating_add
        spec_hash: task.spec.stable_hash(),      // SHA-256 of canonical JSON
    }
    append TaskStarted{ status=Running, claim=Some(claim) }
    return Claimed(claim)                                         [:1027]

  // Identity derivation (framework-owned, deterministic):
  claim.execution_id(run_id, task_id) =
      "{run_id}:{task_id}:{claim.revision}:{claim.attempt}"        [runtime.rs:221]
```

The claim is the **only** ticket that authorizes a subsequent status
write on that task. Every state-mutating callback re-validates it:

```text
set_claimed_task_status(run_id, task_id, claim, status, ...)      [store.rs:1032]
  under with_run_lock:
    task = plan.tasks.find(id == task_id)
    if task is None                         → Superseded          [:1045-1047]
    if task.status != Running               → Superseded          [:1048]
    if task.claim != Some(claim)            → Superseded          [:1048]
    append_task_status_event(...)           → Applied             [:1051-1060]

requeue_claimed_task(run_id, task_id, claim, ...)                 [store.rs:1066]
  same guard (Running + claim identity)                            [:1081]
  then: append TodoUpdated{ status=Pending, retry_count+1, claim=null }
        → Applied (the kernel re-claims with attempt = retry_count+1
                   on the next safe point)                         [:1084-1104]
```

The naked `set_task_status` (store.rs:953) does **not** check the
claim or the current status — it writes whatever status the caller
passes. It is the **only** status writer without a guard. All current
callers are list-filtered before calling (see Findings A-TSK-04-P3-01).

### Revision optimistic concurrency

```text
compare_and_commit_revisioned_task_graph(run_id, commit)          [store.rs:755]
  under with_run_lock:
    run = get_run(run_id)
    if run.status in {Completed, Cancelled}:
        return Err(InvalidPlan)  // terminal runs are immutable     [:764-772]
    current = load_revisioned_task_graph(run_id)
    current_revision = current?.snapshot.revision
    if current_revision != commit.expected_revision:
        return Err(PlanConflict{ expected, current })              [:786-792]
    // next revision must be exactly expected+1
    expected_next = commit.expected_revision + 1
    if commit.next.snapshot.revision != expected_next:
        return Err(InvalidPlan)                                    [:798-803]
    // EKO metadata round-trip + spec/execution id match           [:807-832]
    // initial-plan tasks must be in pending execution state       [:833-844]
    append PlanRevisionCommitted{ base_revision, reason, effects,
                                  created_task_ids, plan }
    rewrite_plan(run_id)
    return load_revisioned_task_graph(run_id)                      [:867-883]
```

`PlanConflict` is returned **before** any event is appended. Verified
by `task_update_rejects_stale_revision_without_appending_event`
(store.rs:3039). The framework `PlanValidator` runs at two sites:
inside `attach_plan_for_test` (store.rs:654, via the test helper) and
inside `apply_task_patch_for_test` (store.rs:921). The commit adapter
itself trusts the framework `TaskRevisionService` (which the tools
call) for patch semantics and re-validates structure on commit via
the framework validator at runtime_executor.rs:214 (the kernel's
safe-point entry).

### Event authority and replay

```text
append_event_line(run_id, task_id, step_id, kind, payload)        [file_shadow.rs:118]
  under run_write_lock(run_id):                  // per-run Mutex
    next_seq = self.next_seq(run_id)?           // 1-based, cached
    event = RuntimeTaskEvent{ seq, run_id, task_id, step_id, kind, payload, now }
    append_line(events_path, json + "\n")        // O_APPEND + sync_all
    seq_cache.insert(run_id, next_seq)
    if let Some(hook) = event_hook: hook(&event) // HookEventDispatcher
    return event

rewrite_plan(run_id)                                              [file_shadow.rs:208]
  under run_write_lock(run_id):                  // SAME lock as append
    events = read_events(run_id)                 // in seq order
    if last event affects neither projection: return
    rebuilt = rebuild_plan_from_events(&events)  // deterministic fold
    if affects_plan:    atomic_write(plan_path, ...)
    if affects_run_state: atomic_write(run_state_path, ...)
```

`events.jsonl` is the authority; `plan.json` and `run-state.json` are
deterministic projections. Every replay path iterates events in seq
order under the per-run lock:

- `rebuild_plan_from_events` (event_rebuild.rs:59-260) — folds events
  into the plan envelope + task execution states. `PlanRevisionCommitted`
  carries forward the prior execution projection for matching task ids
  (147-162), and applies `skipped_task_ids` / `reset_task_ids` effects
  (163-193).
- `recoverable_subagent_result` (store.rs:2039-2078) — folds events
  for one `(task_id, execution_id)`. `SubagentAssigned` clears any
  prior terminal fact; `SubagentReleased{status=completed}` sets the
  result. This is the deterministic rule that lets a resumed task
  reuse durable output for the exact same attempt without redispatch.
- `list_recovery_blockers` (store.rs:2081-2130) — folds
  `RecoveryBlocked`/`RecoveryResolved`. The "fail-closed" tail at
  2113-2128 synthesizes a blocker for any Blocked todo with the
  restart marker summary, so an interrupted `record_recovery_blocker`
  still gates resume.
- `active_subagent_boundaries` / `active_tool_boundaries`
  (store.rs:1528-1594) — fold `SubagentAssigned`/`SubagentReleased`
  and `ToolStarted`/`ToolCompleted`/`ToolFailed` to classify
  in-flight work at crash time.

### Crash / restart / cancel / retry

```text
Boot recovery (recover_incomplete)                                [store.rs:1631]
  zombies = list_runs_in([Running])
  for run in zombies:
    note(run, "recovered from running (interrupted by process restart)")
    transition_run(run, Paused)                   // ← run lock #1
    plan = get_plan(run)
    active_subagents = active_subagent_boundaries(run)
    active_tools     = active_tool_boundaries(run)
    for todo in list_todos(run).filter(status == Running):
      execution_id = task.claim.execution_id(run, task.id)
      completed = recoverable_subagent_result(run, todo.task_id, execution_id)
      active_mutating = active_tools/subagents
                         .find(task_id == todo.task_id && !replay_safe)
      (next_status, summary) = match:
          completed.is_some()     → (Pending, "Subagent completed before
                                       interruption; pending review")
          active_mutating.is_some() → (Blocked, "mutating side effect is
                                       indeterminate after restart")
          else                     → (Pending, "interrupted; pending resume")
      set_task_status(run, todo, next_status, ...) // ← run lock #2..N
      if next_status == Blocked:
          record_recovery_blocker(run, todo, execution_id, call_id,
                                  tool_name, summary)
    log "recovered interrupted run -> Paused at boot"

Resume (resume_task_run)                                          [store.rs:434]
  blockers = list_recovery_blockers(run)
  if !blockers.is_empty():
      return Err(RecoveryBlocked{ details })        // fail-closed
  transition_run(run, Running)                      // caller re-launches exec

Cancel (request_cancel)                                           [store.rs:577]
  if cancel_active_run(run):  // in-memory driver token
      return Ok(true)             // driver observes token, kernel drains,
                                  // finalize_cancelled_run_state sweeps
  match run.status:
      Pending | Paused | Failed → transition_run(Cancelled) → Ok(true)
      Running | Cancelled | Completed → Ok(false)   // must go via driver
  // post-kernel: finalize_cancelled_run_state (executor.rs:643)
  //   for todo in todos.filter(Pending|Running|Blocked):
  //       set_task_status(Cancelled)
  //   transition_run(Cancelled)

Pause (request_pause)                                             [store.rs:598]
  if run.status != Running: return Ok(false)
  token = run_cancel_tokens.remove(run)
  transition_run(Paused)          // status first
  token.cancel()                  // then stop in-flight subagents

Retry (retry_blocked_task)                                        [store.rs:1220]
  under with_run_lock:
    run.status must be Paused or Failed
    task.status must be Blocked | Failed | TimedOut
    task.retry_count < task.max_retries
    append TodoUpdated{ status=Pending, retry_count+1 }
    // sweep: descendants blocked by *this* upstream failure → Pending
    for descendant in upstream_blocked_descendants(task):
        append TodoUpdated{ status=Pending, "unblocked after retrying..." }
    append Note{ "user retried blocked task ..." }
    rewrite_plan
    transition_run_locked(Running)                 // re-entrant under lock
    return next_attempt

Auto-retry (resolve_dispatch path, executor.rs:1396)             [A-TSK-03 V03]
  on ExecutionFailed && claim.attempt-1 < task.max_retries:
      requeue_claimed_task(run, task, claim, ...) → Pending + retry_count+1
      return RuntimeTaskResolution::Pending  // kernel re-claims next safe point
```

Invariants verified by this graph (full evidence in V01-V04):

- **Stale revision writes are rejected, not silently applied.**
  `claim_task` returns `ReloadSnapshot`; `compare_and_commit` returns
  `Err(PlanConflict)`; both before any event append. V02.
- **Stale claim writes are rejected, not silently applied.**
  `set_claimed_task_status` / `requeue_claimed_task` return
  `Superseded` whenever the task is no longer `Running` or the claim
  identity no longer matches; no event is appended in the rejected
  branch. Verified by `stale_claim_cannot_overwrite_cancelled_task`
  (store.rs:3100) and `patched_spec_uses_new_execution_identity_without_retry_bump`
  (store.rs:3148). V02.
- **Claim identity is deterministic and attempt-scoped.**
  `execution_id = {run_id}:{task_id}:{revision}:{attempt}`; a retry
  bumps `attempt` (via `retry_count+1` in `claim_task`), producing a
  new execution_id; a spec change bumps `revision` (via
  `compare_and_commit`), producing a new spec_hash and a new
  execution_id. The durable subagent result is keyed on the exact
  execution_id, so it is reused only for the same attempt and never
  for a revised or retried one. Verified by
  `patched_spec_uses_new_execution_identity_without_retry_bump` and
  `boot_recovery_reuses_completed_subagent_without_redispatch`.
  V01.
- **Events are strictly ordered per run.**
  `append_event_line` holds the per-run write lock across seq-alloc →
  append → cache-bump. Concurrent appends produce strictly increasing
  per-run seq (verified by
  `concurrent_append_produces_unique_strictly_increasing_seq`).
  `rewrite_plan` re-acquires the same lock (verified by
  `rewrite_plan_waits_for_same_run_write_lock`). V03.
- **Run-level terminal monotonicity is enforced.**
  `TaskRunStatus::can_transition_to` makes `Cancelled` and `Completed`
  refuse any further transition; `transition_run` checks this before
  appending any event and rolls back (no event) on rejection. Verified
  by `illegal_transition_is_rejected_and_leaves_no_event` and
  `file_path_rejects_illegal_transition_and_appends_no_event`. V03.
- **Event-kind follows the typed status, not summary text.**
  `append_task_status_event` (store.rs:1143-1152) maps the typed
  `TodoStatus` to the typed `RuntimeEventKind`. A `Failed` task whose
  summary mentions "cancelled" still produces `TaskFailed`, not
  `TaskCancelled`. Verified by
  `task_terminal_events_follow_typed_status_not_detail_text`. V03.
- **Hook dispatch preserves terminal distinction.**
  `TaskCancelled → SubagentStopStatus::Cancelled`,
  `TaskTimedOut → TimedOut`, not collapsed to Failed/Skipped. The
  bounded single-consumer queue delivers hooks in persisted order
  (backpressure, not drop). V03.
- **Crash recovery is fail-closed for mutating in-doubt work.**
  `recover_incomplete` marks a task Blocked (and appends
  `RecoveryBlocked`) when a non-replay-safe tool or subagent was
  in-flight; resume then refuses until the user resolves. Verified by
  `mutating_in_doubt_subagent_blocks_resume_until_user_decides` and
  `blocked_todo_restores_barrier_if_resolution_crashes_before_mutation`.
  V04.
- **Auto-retry is bounded and attempt-monotonic.** The retry budget
  (`max_retries`) is checked in both `resolve_dispatch`
  (executor.rs:1396-1432) and `retry_blocked_task` (store.rs:1253);
  each retry bumps `retry_count` and produces a new `attempt`, so the
  same execution_id is never reused across retries. V04.

## Findings

The headline result is positive: state updates from stale
revisions/attempts, cancellation, restart, and event replay flow
exclusively through valid claims, and run-level terminal states are
monotonic. Two P3 defense-in-depth gaps are recorded; both are
robustness hardening, not correctness defects on current code paths.

### A-TSK-04-P3-01: `set_task_status` is a non-claim-guarded status writer with no `TodoStatus` monotonicity check

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:953-983`
    — `set_task_status` reads the plan to confirm the task exists, then
    calls `append_task_status_event` with whatever `TodoStatus` the
    caller supplied. There is no `task.status.can_transition_to(next)`
    check; there is no `task.claim` identity check; there is no
    terminal-state rejection.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:367-457`
    — `TodoStatus` has no `can_transition_to` table at all (contrast
    `TaskRunStatus::can_transition_to` at types.rs:517-527, which
    enforces run-level terminal monotonicity).
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1032-1062`
    — `set_claimed_task_status` IS guarded: it returns `Superseded`
    when `task.status != Running` or `task.claim != Some(claim)`. The
    executor's status writes therefore cannot regress a terminal task.
- Reachability: every production caller of the naked `set_task_status`
  was enumerated (`grep -rn '\.set_task_status(' echo-agent-app-core/src |
  grep -v test`). The non-test callers are exactly:
  1. `executor.rs:551` — the Paused sweep, filtered on
     `todo.status == Running` (executor.rs:549), so only Running→Pending
     is written;
  2. `executor.rs:651` — `finalize_cancelled_run_state`, filtered on
     `Pending | Running | Blocked` (executor.rs:645-649), so only
     non-terminal tasks are cancelled;
  3. `executor.rs:1571` — `block_task` callback, invoked by the kernel
     on a task that is currently Pending (the kernel's frontier never
     includes terminal tasks);
  4. `store.rs:1706` — inside `recover_incomplete`, gated on
     `todo.status == Running` (store.rs:1660-1663);
  5. `store.rs:2163, 2170` — `resolve_recovery_task`, after the
     `RecoveryResolved` event and on a task known to be Blocked.
  All five are list-filtered or runtime-guaranteed to operate on
  non-terminal tasks. No current caller can resurrect a terminal task.
- Expected invariant: a Cancelled/Completed/Skipped/Failed/TimedOut
  task should not be writable back to Pending/Running through any
  store API, by either an in-process caller or a future contributor
  who does not know the implicit contract.
- Observed behavior: the store permits any `TodoStatus` write on any
  task via `set_task_status`. The invariant holds today only because
  every caller self-restricts; the store does not enforce it.
- Impact: low today (no misuse), medium for evolution. A future
  recovery sweep or a tool that calls `set_task_status` directly (the
  method is `pub`) could regress a terminal task with no error. For
  EKO's local-assistant threat model this is a defense-in-depth gap,
  not a live data-loss path.
- Root cause: `TodoStatus` was modeled as a UI projection enum without
  a transition table, because the executor's claim guard was deemed
  sufficient. The naked writer was kept `pub` for recovery and the
  paused/cancelled sweeps, which then became implicit trusted callers.
- Direction: either (a) add a `TodoStatus::can_transition_to` table
  and a guard inside `set_task_status` (with an internal
  `_set_task_status_unchecked` for the recovery sweeps that
  intentionally move out of Running), or (b) keep the unguarded writer
  but rename it to make the trust contract explicit (e.g.
  `set_task_status_unchecked`) and gate the public surface behind a
  claim-guarded variant. Option (a) is preferable — it matches
  `transition_run`'s enforcement model.
- Regression validation: a test that drives `set_task_status(Cancelled
  → Pending)` and asserts `Err(IllegalTransition)` (or equivalent) and
  zero events appended; plus a test that the recovery sweeps still
  succeed for their intended transitions.
- Validation reports: [V03-01](../validations/A-TSK-04/V03-01.md)

### A-TSK-04-P3-02: `recover_incomplete` is not atomic across the run-transition and the per-task reset, and is not idempotent for Paused runs that still carry stuck Running tasks

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1631-1776`
    — `recover_incomplete` first calls `transition_run(run, Paused)`
    (acquires and releases the per-run lock at store.rs:1653 via
    `transition_run` → `with_run_lock`), then iterates `Running` todos
    and calls `set_task_status` for each (each call acquires and
    releases the per-run lock independently at store.rs:1706). There
    is no enclosing transaction: the per-run lock is released between
    the transition and every per-task reset.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1632`
    — `INTERRUPTED: &[TaskRunStatus] = &[TaskRunStatus::Running]`. The
    sweep only picks up runs whose persisted status is `Running`. A run
    that crashed mid-recovery (now `Paused`, but with one or more
    `Running` todos whose reset did not land) is **not** re-swept on
    the next boot.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:434-448`
    — `resume_task_run` checks `list_recovery_blockers`. If the crash
    happened before `record_recovery_blocker` landed for the
    affected todo (the mutating-in-doubt case), `list_recovery_blockers`
    returns empty and the resume succeeds. The kernel then sees a
    `Running` task in `load_snapshot`; `claim_task` returns
    `ReloadSnapshot` for it (status != Pending); the frontier never
    includes it; `complete_run_if_quiescent` returns false (the task
    is not Completed/Skipped); the drain loop re-iterates indefinitely.
    This is exactly the drain-loop soft-lock recorded as A-TSK-03-P3-01.
- Reachability: requires a process crash during `recover_incomplete`,
  between the `transition_run(Paused)` and any of the per-task
  `set_task_status` resets (or mid-iteration). The window is narrow
  for small plans but grows with the number of `Running` todos. No
  test exercises crash-during-recovery; the existing recovery tests
  (`boot_recovery_*`) assume `recover_incomplete` runs to completion.
- Expected invariant: boot-time recovery should either fully complete
  or be safely re-runnable on the next boot.
- Observed behavior: a mid-recovery crash leaves the run Paused with
  stuck Running tasks; subsequent boots skip it (status not Running);
  the user-driven resume then soft-locks because the Running task
  cannot be re-dispatched and cannot reach the completion CAS.
- Impact: low probability, medium severity when hit. The failure mode
  is a hung `execute_run` drain task that the user must manually
  cancel. No data corruption — the plan and events are intact — but
  the run is effectively bricked without manual intervention (cancel
  via `request_cancel` from another process, or hand-editing the
  events stream). For EKO's local-assistant threat model this is a
  robustness defect, not a safety issue.
- Root cause: the recovery flow was written as a sequential best-effort
  sweep, not as a transactional or idempotent operation. The
  `INTERRUPTED` filter was chosen to avoid re-pausing runs that
  successfully recovered, but it also prevents finishing partial
  recoveries.
- Direction: pick one of two hardening strategies (no behavior change
  for the happy path):
  1. **Reset-before-transition.** Iterate `Running` todos and reset
     each BEFORE `transition_run(Paused)`. A crash leaves the run
     still-Running; the next boot's sweep retries the whole sequence
     idempotently (the now-Pending todos are skipped by the
     `status == Running` filter).
  2. **Idempotent finish.** Extend the `INTERRUPTED` filter (or add a
     second sweep) to also catch Paused runs that still have Running
     todos, and finish their reset. The `status == Running` filter on
     the inner loop makes this safe to re-run.
  Option 1 is structurally simpler and removes the per-run-lock
  release between transition and reset; option 2 keeps the existing
  flow shape.
- Regression validation: a test that simulates a crash between
  `transition_run(Paused)` and the inner `set_task_status` (e.g. by
  short-circuiting after the first todo), then re-invokes
  `recover_incomplete` and asserts the run reaches a fully-Pending
  state and that `resume_task_run` succeeds without soft-locking.
- Validation reports: [V04-01](../validations/A-TSK-04/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Claim identity persistence: deterministic `(revision, attempt, spec_hash)`; stale claims cannot be replayed after revision/cancel/retry/crash | yes | passed | [V01-01](../validations/A-TSK-04/V01-01.md) |
| V02 | Stale write rejection: `claim_task` / `compare_and_commit` / `set_claimed_task_status` / `requeue_claimed_task` / `transition_run` all reject before appending any event | yes | passed | [V02-01](../validations/A-TSK-04/V02-01.md) |
| V03 | Event replay ordering & terminal monotonicity: per-run strict seq, deterministic fold, run-level terminal lock, typed-status event mapping, hook terminal distinction | yes | passed | [V03-01](../validations/A-TSK-04/V03-01.md) |
| V04 | Crash/restart/cancel/retry: 58 store + event_rebuild + hook_event_dispatcher + file_shadow tests pass; recovery/durable-reuse/blocking/barrier/stale-rejection scenarios covered | yes | passed | [V04-01](../validations/A-TSK-04/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — eight code/module comments treated as hypotheses; classifications inline in Historical Claim Status) | passed | classified inline |

Executed cargo commands (all exit 0):

```text
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib task_runtime::store
  → 34 passed; 0 failed; 0 ignored (1.55s)
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib task_runtime::event_rebuild
  → 3 passed; 0 failed; 0 ignored (0.13s)
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib task_runtime::hook_event_dispatcher
  → 12 passed; 0 failed; 0 ignored (0.06s)
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib task_runtime::file_shadow
  → 9 passed; 0 failed; 0 ignored (2.69s)
```

The full `echo-agent-cli` pre-commit matrix was not re-run because
this review is read-only; the four targeted subsets above are the
directly relevant evidence — they are the suites that exercise the
claim/revision/recovery/event-replay/terminal-monotonicity paths
audited in this task.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `store.rs:1-9` module doc: "Every state mutation appends a `RuntimeTaskEvent` to `events.jsonl` and refreshes only the affected projection from the full event stream." | current | Verified by V03: `append_event_line` is the only write primitive; `rewrite_plan` rebuilds from the full event stream; every state-mutating store method follows the append+rewrite pattern. |
| `store.rs:274-280` `with_run_lock` doc: "revision compare-and-commit / transition_run 都是一'读事件 → 校验 → 追加 → 重建投影'事务, 必须按 run 串行化。" | current-with-caveat | Verified that the per-run lock serializes the read-validate-append-rewrite critical section for the methods that use `with_run_lock` (claim_task, compare_and_commit, transition_run, set_task_status, set_claimed_task_status, requeue_claimed_task, complete_run_if_quiescent). The caveat is recorded as A-TSK-04-P3-02: `recover_incomplete` does NOT wrap its multi-step flow in a single `with_run_lock`, so the serialization is per-call, not per-recovery. |
| `store.rs:985` `claim_task` doc: "Atomically claim a Pending task from one exact plan revision." | current | Verified by V01/V02: the claim is gated on `plan.revision == expected_revision && task.status == Pending && task.spec == expected_task.spec`, all under `with_run_lock`. |
| `store.rs:1064-1066` `requeue_claimed_task` doc: "Atomically requeue one failed claimed attempt and advance its retry counter without exposing an unclaimed Pending window." | current | Verified by V01: the function CAS-checks Running + claim identity, then appends a single TodoUpdated with status=Pending + retry_count+1 + claim=null, all under `with_run_lock`. There is no observable Pending-without-claim window between Running and Pending. |
| `store.rs:1620-1627` `recover_incomplete` doc: "A run left in `Running` when the process died ... Move it to `Paused` so the normal resume path can re-read the plan and skip completed work." | current-with-caveat | Verified for the happy path by V04 (`boot_recovery_*` tests). The caveat is A-TSK-04-P3-02: a crash during the sweep leaves a Paused run with stuck Running tasks that the next boot will not finish recovering. |
| `store.rs:2036-2038` `recoverable_subagent_result` doc: "A later `SubagentAssigned` with the same id clears an older terminal fact, which is how an explicitly confirmed retry avoids reusing stale output." | current | Verified by V01/V04: the fold iterates events in seq order; `SubagentAssigned` resets `result = None`; the test `boot_recovery_reuses_completed_subagent_without_redispatch` exercises the reuse path, and `patched_spec_uses_new_execution_identity_without_retry_bump` exercises the new-execution-id (no-reuse) path. |
| `store.rs:2080` `list_recovery_blockers` doc: "Current unresolved recovery barriers, folded from append-only events." | current | Verified by V03/V04: the fold iterates `RecoveryBlocked`/`RecoveryResolved` in seq order; the fail-closed tail at 2113-2128 synthesizes a blocker for any Blocked todo with the restart marker summary. The test `blocked_todo_restores_barrier_if_resolution_crashes_before_mutation` exercises the fail-closed synthesis. |
| `store.rs:2132-2151` `resolve_recovery_task` doc: "Persist the user's decision first. If the process stops before the Todo mutation, the still-Blocked Todo synthesizes the barrier again on the next read, so recovery continues to fail closed." | current | Verified by V04 (`blocked_todo_restores_barrier_if_resolution_crashes_before_mutation`): the test manually appends `RecoveryResolved` without the Todo mutation and asserts `list_recovery_blockers` still returns the blocker via the fail-closed synthesis. |
| `file_shadow.rs:105-117` `append_event_line` doc: "the append is serialized by the store mutex (single writer), and the line is written with a trailing newline. A crash mid-append can at worst lose the last partial line." | current | Verified by V03: the per-run write lock (file_shadow.rs:185-195) serializes same-run appends; `append_line` (427-436) uses `O_APPEND` + `sync_all`. The partial-tail-truncation hardening ("gate 2") is acknowledged as future work in the doc; it is not a regression. |
| A-TSK-03 handoff: "A-TSK-04 → must verify the resume path correctly re-dispatches the Running→Pending reset tasks and that `complete_run_if_quiescent`'s CAS is sound under concurrent plan patches." | resolved (verified) | V01/V02 confirm the reset tasks are Pending (claim_task re-claims them on the next safe point); `completion_gate_rechecks_latest_plan_revision` (store.rs:3307) verifies the CAS rejects completion when a racing patch inserts an unresolved task. |
| A-TSK-03 handoff: "A-TSK-04 should verify the resume / recovery path stays sound under the EKO reconciliation sweeps." | resolved (with caveat) | V04 confirms the happy-path soundness. The caveat is A-TSK-04-P3-02: the recovery sweep itself is not crash-atomic. |
| A-TSK-03-P3-02 (latent lossiness of `Retrying`/`Paused` `TaskStatus`): "EKO never produces framework `Retrying` or `Paused` on the executor path." | current (re-verified) | V03 confirms: every status the store writes is one of `Pending | Running | Blocked | Completed | Failed | Cancelled | TimedOut | Skipped` (store.rs:1143-1152 maps these to typed events). Framework `Retrying`/`Paused` are never produced by any store writer. |

## Coverage And Uncertainty

- **Inspected in full:** the entire `TaskRuntimeStore` write surface
  (every state-mutating public method: `create_run`, `set_run_attachments`,
  `transition_run`, `resume_task_run`, `complete_run_if_quiescent`,
  `register_*_cancel_*`, `request_cancel`, `request_pause`,
  `compare_and_commit_revisioned_task_graph`, `set_task_status`,
  `claim_task`, `set_claimed_task_status`, `requeue_claimed_task`,
  `task_claim_is_current`, `increment_retry_count`, `retry_blocked_task`,
  `add_review`, `add_artifact`, `put_summary`, `note`,
  `record_subagent_*`, `record_tool_*`, `recover_incomplete`,
  `recoverable_subagent_result`, `list_recovery_blockers`,
  `resolve_recovery_task`); the entire `append_event_line` /
  `rewrite_plan` / `atomic_write` / `append_line` write path; the
  entire `rebuild_plan_from_events` fold; the entire
  `HookEventDispatcher` translation table and queue semantics; the
  `EkoRevisionedTaskStore` adapter; the framework `TaskClaim` /
  `TaskSpec::stable_hash` / `TaskClaim::execution_id` definitions;
  the framework `RuntimeDagController` claim/resolve contract.
- **Inspected partially:** the 247-K `executor.rs` was read in the
  slices cited by A-TSK-03 (the controller callbacks, the drain loop,
  `finalize_cancelled_run_state`, the Paused sweep, `resolve_dispatch`'s
  retry/blocked branches); A-TSK-03 V03 already catalogued every
  callback as persistence/dispatch/product-policy. This task did not
  re-audit `execute_task` (the per-Subagent pipeline) — it is the
  dispatch seam behind `dispatch_task`, out of scope for
  claim/recovery/terminal concerns.
- **Not inspected (out of scope):**
  - `compact_context.rs`, `file_shadow.rs` (test module only),
    `file_store.rs` (read paths only, no write authority),
    `memory_bridge.rs`, `planner.rs`, `profiles.rs`, `register.rs`,
    `review.rs`, `task_tools.rs`, `worktree.rs` — these are
    context/planner/review/worktree helpers; their internals are
    A-TSK-05 (worktree), A-TSK-06 (review/memory), and A-TSK-02
    (tool surface) territory.
  - The full `echo-agent-cli` pre-commit matrix (fmt / clippy /
    all-features test). The review is read-only; the four targeted
    subsets above are the directly relevant evidence.
- **Uncertain claims:**
  - The exact probability of A-TSK-04-P3-02 is hard to bound without
    measurement. It is filed as P3 (medium confidence) because the
    trigger requires a process crash during a narrow window and the
    failure mode (drain soft-lock) is recoverable by manual cancel.
  - Whether the `TaskRuntimeStore`'s `pub` surface is consumed by any
    out-of-repo plugin (e.g. a user-supplied MCP server that writes
    task status) is unknowable from this repo. If it is,
    A-TSK-04-P3-01's defense-in-depth gap becomes more consequential
    because such a caller would not be on the trusted-caller list.

## Handoff

- **Conclusions downstream tasks may rely on:**
  - Claim identity is deterministic and attempt-scoped
    (`{run_id}:{task_id}:{revision}:{attempt}`); stale claims cannot
    be replayed after a revision, a cancel, a retry, or a crash.
    (V01)
  - Every optimistic-concurrency surface (`claim_task`,
    `compare_and_commit`, `set_claimed_task_status`,
    `requeue_claimed_task`, `transition_run`) rejects stale inputs
    **before** appending any event. The "no event appended on
    rejection" invariant is the proof that the file authority stays
    consistent. (V02)
  - Events are strictly ordered per run under the per-run write lock;
    every replay path (`rebuild_plan_from_events`,
    `recoverable_subagent_result`, `list_recovery_blockers`,
    `active_*_boundaries`) iterates them in seq order.
    (V03)
  - Run-level terminal monotonicity is enforced by
    `TaskRunStatus::can_transition_to`; the typed `TodoStatus` drives
    the typed `RuntimeEventKind`; hook dispatch preserves the
    terminal-status distinction. (V03)
  - Crash recovery is fail-closed for mutating in-doubt work, and the
    durable-result reuse path reuses output only for the exact same
    execution_id. (V04)
  - Auto-retry is bounded by `max_retries` and produces a new
    `attempt` (and therefore a new execution_id) on each retry; the
    same execution_id is never reused across retries. (V01, V04)
- **Reports downstream tasks must read:**
  - [V01-01](../validations/A-TSK-04/V01-01.md) for the claim identity
    matrix and the stale-claim rejection evidence.
  - [V02-01](../validations/A-TSK-04/V02-01.md) for the per-API stale
    write rejection evidence and the "no event on rejection" invariant.
  - [V03-01](../validations/A-TSK-04/V03-01.md) for the event-ordering,
    replay-fold, and terminal-monotonicity analysis (and the
    `TodoStatus` monotonicity gap behind A-TSK-04-P3-01).
  - [V04-01](../validations/A-TSK-04/V04-01.md) for the 58-test
    inventory covering recovery, durable reuse, blocking, barrier
    crash-safety, stale-rejection, and the per-API test mapping.
- **Task-to-reference mapping:**
  - A-TSK-05 (worktree/file ownership) → may rely on the event
    authority and per-run lock being sound; the worktree-merge step
    (`integrate_reviewed_task`) is gated by the claim-guarded
    `set_claimed_task_status(Completed)`, so the claim identity
    invariant established here protects the merge.
  - A-TSK-06 (review/artifacts/parent context) → may rely on the
    typed-status event mapping and the durable-result fold; the
    review gate consumes the `RuntimeTaskResolution::Blocked`
    outcome that the claim-guarded status write produces.
  - F-TSK-03 (framework kernel) → the framework-level gaps
    F-TSK-03-P2-01 (in-flight stall) and F-TSK-03-P2-02
    (abort-orphan) remain owned by F-TSK-03; EKO mitigates P2-02 via
    `finalize_cancelled_run_state` (A-TSK-03 V03) and the
    `recover_incomplete` sweep (this task V04).
- **Conditions that make this report stale:**
  - Any commit that adds a second `events.jsonl` writer outside
    `append_event_line`, or a second `RevisionedTaskStore` impl in
    `echo-agent-cli`, invalidates V01/V03.
  - Any commit that removes the `with_run_lock` wrapper from
    `claim_task` / `compare_and_commit_revisioned_task_graph` /
    `set_claimed_task_status` / `requeue_claimed_task` invalidates
    V02.
  - Any commit that introduces a writer of framework `Retrying` or
    `Paused` `TaskStatus` on the executor→store path invalidates the
    A-TSK-03-P3-02 latent classification (it would become live data
    loss per A-TSK-01-P2-02).
  - Any commit that wraps `recover_incomplete` in a single
    `with_run_lock` (the P3-02 fix) invalidates that finding.
- **Follow-up task IDs (no fixes implemented in this review):**
  - A robustness-focused cleanup task should pick up A-TSK-04-P3-01
    (add a `TodoStatus::can_transition_to` table and guard the naked
    `set_task_status`, or rename it to make the trust contract
    explicit).
  - A recovery-hardening task should pick up A-TSK-04-P3-02 (reset
    before transition, or extend the sweep to finish partial
    recoveries). The fix should ship with a crash-during-recovery
    test.
  - A-TSK-03-P3-01's drain-loop guard remains the higher-leverage
    fix for the soft-lock symptom that P3-02 can produce; fixing
    P3-01 alone would bound the soft-lock to a clean break instead
    of an infinite spin, even without fixing P3-02.
