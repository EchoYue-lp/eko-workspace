# Q-FLT-02: Task and Subagent fault-injection suite

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both source repositories; all probe artifacts live
> under `/tmp/qflt02-probe` — no files added to either repository)

## Question

Do DAG/claim/Subagent invariants survive stale revisions, old attempts,
cancel, timeout, crash, restart, worktree conflict, and failed review?

**Answer: The claim/revision kernel survives five of the eight scenarios and
is enforced end to end (stale revisions -> `ReloadSnapshot`, stale claims ->
`Superseded`, terminal monotonicity holds, restart recovery is claim-aware,
failed reviews are claim-guarded and budget-bounded). Four scenarios expose
invariant violations, all traceable to canonical findings: pause-during-wave
becomes a permanent cancellation (A-TSK-04-P1-01 / A-TSK-03-P1-01), a crash
mid-pause leaves an unrecoverable stranded task inside a Paused run (**NEW
P1-01**, no in-process or boot-recovery path exists), Team timeout detaches
already-spawned members (F-SUB-02-P1-02), and a torn `events.jsonl` tail
bricks reads and writes so replay/recovery never begin (A-TSK-01-P1-01 /
A-TSK-04-P1-03). Worktree isolation survives the happy path but crash lock
residue permanently blocks the logical task with no repair (A-TSK-05-P2-01)
and `eko-fork-*` leftovers are never swept (A-TSK-05-P2-02).**

### Scenario survival verdicts

| # | Scenario | Verdict | Evidence | Canonical / new finding |
|---|---|---|---|---|
| 1 | Stale revision writes rejected | **SURVIVED** | claim CAS `ReloadSnapshot`; stale claims `Superseded`; spec-reset clears claim; patch engine refuses Running-task edits; tool layer rejects stale `task_execute` revision | none new (X-TSK-01-P3-01 window referenced) |
| 2 | Old attempt completion cannot regress terminal | **SURVIVED** | attempt identity `retry_count+1`; old-attempt writes `Superseded`; post-terminal duplicates `Superseded`; run cancel durable | none new |
| 3 | Cancel in wave (pause) | **NOT SURVIVED** | pause during wave -> durable Paused run force-transitioned to Cancelled; resume impossible; replay preserves the regression | canonical A-TSK-04-P1-01 / A-TSK-03-P1-01 |
| 3b | Crash between pause and task reset | **NOT SURVIVED** | run Paused + task Running+claimed; `recover_incomplete` skips Paused runs; resume polls forever; no repair surface | **NEW Q-FLT-02-P1-01** |
| 4 | Task timeout (Team) | **NOT SURVIVED** | outer timeout drops the orchestrator future; `tokio::spawn`ed members keep running/writing; JoinSet+abort contrast works | canonical F-SUB-02-P1-02 (+P1-01 zero-token gap) |
| 5 | Crash — torn events.jsonl tail | **NOT SURVIVED** | `read_events` Decode error bricks reads and writes; `recover_incomplete` = 0; only manual truncation heals | canonical A-TSK-01-P1-01 / A-TSK-04-P1-03 |
| 6 | Restart recovery (Running->Paused) | **SURVIVED** | Running->Paused, in-flight replay-safe -> Pending claim cleared, Completed preserved, completed work never re-dispatched, resume works | none new (A-TSK-04-P2-01 unguarded `set_task_status` referenced) |
| 7 | Worktree conflict / dirty tree / lock residue | **PARTIAL** | happy path reuse + dirty retention + manual-unlock reuse work; crash lock blocks `acquire_fork` forever; no `eko-fork-*` sweep | canonical A-TSK-05-P2-01 / P2-02 |
| 8 | Failed review retry / acceptance | **SURVIVED** | claim-guarded requeue, no unclaimed window, breaker suspends on fingerprint/budget, Blocked short-circuits to Suspend at the gate, fix tasks preserve id/deps | none new (A-TSK-06-P2-01/P3-01 dead arms referenced) |

## Scope

Primary source paths inspected (deep read this task):

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs` —
  `create_run` (:302-357), `transition_run` (:387-428), `request_cancel`/
  `request_pause` (:577-622), `compare_and_commit_revisioned_task_graph`
  (:755-884), `set_task_status` (:953-983), `claim_task` (:986-1029),
  `set_claimed_task_status` (:1032-1062), `requeue_claimed_task`
  (:1066-1105), `task_claim_is_current` (:1107-1121), `retry_blocked_task`
  (:1220-1340), `recover_incomplete` (:1631-1776), `recoverable_subagent_result`
  (:2039-2078, pub(crate) — read-only reference).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs`
  (full) — `append_event_line` (:118-178), `rewrite_plan` (:208-280),
  `next_seq` (:289-299), `read_events` (:362-379), `atomic_write`/`append_line`.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/event_rebuild.rs`
  (full) — the seq-order fold, claim restore (:229-231), execution-preserving
  revision commits (:147-162), reset/skip claim clearing (:183-193).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs` —
  drain-loop terminal mapping (:480-585), `finalize_cancelled_run_state`
  (:643-661), `resolve_dispatch` retry/review branches (:1396-1560),
  `load_snapshot` (:1227-1253), `interruption_outcome` (:1595-1613),
  `run_review_gate` (:1773-1836), `assess_task_execution` (:663-790).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/review.rs`
  (full) — `requires_review`, `review_task`, `circuit_breaker_action`,
  `build_fix_task`.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs` —
  `RunWorktree::acquire_fork`/`create_fork` (:364-460), `run_git`,
  `has_changes`, `integrate_fork_worktree` boundary (:569-807).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs` —
  `TaskRunStatus::can_transition_to` (:480-527), `PlanTask::to_task`
  (:1117-1160), `TaskUpdateOperation` (:1257-1272).
- `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs` (:196-449) —
  safe points, wave dispatch, in-wave cancel (:390-416), external polling
  (:276-285), stall (:287-313).
- `echo-agent/echo-orchestration/src/tasks/runtime.rs` — `TaskClaim`/
  `execution_id` (:197-224), `DagExecutionState::from_tasks`/`ready_task_ids`.
- `echo-agent/echo-orchestration/src/tasks/revisioned.rs` (:477-600) —
  patch-engine effects (reset/skip), Running-status edit rejection.
- `echo-agent/src/agent/subagent/team/mod.rs` (:343-353), `team/
  manager_subagent.rs` (:270-300), `subagent/executor.rs` (:1075-1090) —
  Team timeout/fan-out anchors.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/task_execute_tool.rs`
  (:255-285) — tool-level stale-revision rejection.

## Out Of Scope

- File-authority/adapter losslessness, plan.json/run-state.json authority —
  A-TSK-01 / X-TSK-01 (their P1-01/P3-01 findings consumed as references).
- Controller boundary policy, review-then-integrate ordering — A-TSK-03
  (its P1-01 consumed as the canonical pause regression).
- Full Team/Subagent lifecycle semantics beyond the timeout/cancel anchors —
  F-SUB-02 (consumed).
- Frontend projections of cancel/timeout events — A-FE-02 / X-EVT-01
  (X-EVT-01-P1-01 cancel-class loss referenced, not re-reviewed).
- No live LLM DAG run and no GUI/TUI process was launched (read-only review);
  probes drive the real store/ledger/review/worktree code over /tmp fixtures.

## Inputs

- Root `AGENTS.md` (claim/ledger pre-gate, one-authority, framework-vs-app
  layering, UTF-8/panic safety, read-only review).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (Q-FLT-02 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read (all complete): `F-TSK-03`, `F-SUB-02`,
  `A-TSK-04`, `A-TSK-05`, `A-TSK-06`, `X-TSK-01`.
- Historical documents treated as hypotheses: `echo-agent-cli/docs/
  MASTER-PLAN.md` and `docs/2026-07-27-runtime-dag-kernel-convergence.md`
  claims about claims/pause/recovery (classified below); the file_shadow
  torn-tail comment (`file_shadow.rs:114-117`).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correct) | `TaskClaim`/`RuntimeTaskClaimOutcome`/`RuntimeTaskResolution`, safe-point reload, wave/cancel/stall (runtime_executor.rs:196-449); `DagExecutionState` frontier; patch-engine effects. All behaviors probed are inside the single framework authority. |
| EKO product policy (application, correct) | `events.jsonl` ledger + projections, boot recovery, pause/cancel control API, retry requeue policy, review/breaker policy, worktree lifecycle. |
| Adapter boundary | `EkoRuntimeDagController`/store CAS are thin and lossless on the probed paths; the two pause-path violations live at this boundary: the framework hardcodes in-wave `Cancelled` without consulting `interruption_outcome` (canonical A-TSK-04-P1-01), and `request_pause` is not crash-atomic with the executor's Paused-branch task reset (NEW P1-01). |
| Duplicate search | Terms (this task's verification): `claim_task`, `set_claimed_task_status`, `requeue_claimed_task`, `task_claim_is_current`, `recover_incomplete`, `request_pause`, `request_cancel`, `interruption_outcome`, `read_events`, `append_event_line`, `PlanRevisionCommitted`, `reset_task_ids`, `circuit_breaker_action`, `run_review_gate`, `acquire_fork`, `worktree lock`, `RecoveryBlocked`. Result: one claim authority, one ledger, one recovery path, one breaker, one worktree lifecycle per concept (consistent with dependency reports' V01 searches); zero new duplicate authorities found. |
| Migration deletion | No new deletion target from this task. Canonical deletion candidates remain as filed (dead `Retrying` variant F-TSK-03-P3-01, dead per-task cancel F-TSK-03-P2-02, panels.rs duplicates A-TSK-05-P2-04, dead artifact/memory arms A-TSK-06-P2-01/P3-01). |

## Current Path

Verified data flow (details in V01-01..V08-01):

1. **Claim creation**: `claim_task` CAS — plan revision check first, then
   Pending + spec equality (`store.rs:992-1028`); claim `{revision, attempt =
   retry_count+1, spec_hash}` persisted in the `TaskStarted` event payload and
   rebuilt by the fold (`event_rebuild.rs:229-231`).
2. **Revision commits**: `compare_and_commit_revisioned_task_graph`
   (`store.rs:755-884`) persists `PlanRevisionCommitted` with
   `skipped_task_ids`/`reset_task_ids` from the patch-engine effects; the fold
   preserves execution+claim across non-reset commits (`event_rebuild.rs:
   147-162`) and clears claim+status on reset/skip (:183-193); the patch engine
   rejects edits to Running tasks (`revisioned.rs:525-541`).
3. **Claim consumption**: `set_claimed_task_status`/`requeue_claimed_task`
   accept only while the exact claim is Running (`store.rs:1032-1105`);
   `Superseded` otherwise; the framework ignores `Superseded`
   (`runtime_executor.rs:423-426`).
4. **Pause/cancel**: `request_pause` transitions Running->Paused first, then
   cancels the shared driver token (`store.rs:598-622`); the framework consults
   `interruption_outcome` only at loop top/cancelled-task/external-poll
   (`runtime_executor.rs:207-209, 267-269, 278-281`) and hardcodes `Cancelled`
   in-wave (:390-416); EKO `finalize_cancelled_run_state` force-transitions the
   Paused run to Cancelled (`executor.rs:643-661`, legal per `types.rs:522`).
5. **Recovery**: `recover_incomplete` processes only `Running` runs
   (`store.rs:1631-1639`); Running tasks -> Pending (replay-safe) or Blocked +
   `RecoveryBlocked` (non-replay-safe boundary) via `set_task_status` (claim
   cleared, `store.rs:1706-1750`); durable result reuse keyed by
   `execution_id` (:1668-1677).
6. **Review/retry**: `resolve_dispatch` budget `claim.attempt-1 <
   max_retries` -> `requeue_claimed_task`; at budget -> Failed;
   `AcceptancePending` -> Blocked; `Executed` -> `run_review_gate` (Pass ->
   integrate -> Completed; NeedsFix -> breaker -> fix task or suspend; Blocked
   -> Suspend; all claim-guarded).
7. **Worktree**: `acquire_fork` reuses unlocked checkouts, hard-errors on any
   locked checkout (`worktree.rs:381-391`), `create_fork` locks (`:431-460`);
   finalize/preserve_error/cleanup are the only unlock paths; fork namespace
   has no sweep (A-TSK-05-P2-01/P2-02).

## Findings

### Q-FLT-02-P1-01: A crash between `request_pause` and the executor's Paused-branch reset strands a Running+claimed task inside a Paused run — no in-process or boot-recovery path exists, and resume hangs the executor in external polling forever

- Priority: P1
- Confidence: high (mechanism deterministic — dynamically reproduced at the
  store level) / medium (trigger probability: pause is a common user action,
  but the crash must land in the small window between `request_pause` and the
  reset)
- Layer: adapter (EKO store/controller with framework executor) — recovery
  gap at the pause boundary
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:598-622`
    — `request_pause` transitions Running->Paused and cancels the driver token
    but does NOT touch the task's Running status/claim;
  - `executor.rs:545-560` — the Running->Pending reset happens only in the
    `RunOutcome::Paused` branch AFTER the framework returns (crash between the
    two leaves the strand durable);
  - `store.rs:1631-1639` — `recover_incomplete` lists only
    `[TaskRunStatus::Running]`, so the Paused run is never recovered at boot;
  - `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:275-285` —
    after resume the stranded Running task is classified in-flight; ready
    empty + in_flight non-empty -> `external_progress_poll_interval` forever;
  - no repair surface: `retry_blocked_task` rejects the state (probe: "run r1
    is Running; retry requires Paused or Failed"), `task_update` rejects
    Running tasks (`revisioned.rs:525-541`), `TaskUpdateOperation` has no
    `SetStatus` (`types.rs:1257-1272`), `EkoTaskToolPolicy` does not enable
    `allow_manual_progress_updates`, and `load_snapshot` (`executor.rs:1227-
    1253`) is a pure projection with no reset.
- Reachability: GUI/TUI pause (`src/tauri/commands/task_runtime.rs:463`,
  `src/tui/events.rs:4346`) -> `request_pause` -> process crash (kill -9, OOM,
  power loss) -> restart -> resume. Dynamically reproduced in
  `/tmp/qflt02-probe` (s9): `request_pause` leaves run Paused + task Running
  with a live claim; `recover_incomplete()` returns 0; `resume_task_run`
  succeeds; re-claim -> `ReloadSnapshot` (never dispatched); no surface
  clears the strand.
- Expected invariant: every durable state a crash can leave must be
  recoverable at boot or through a product surface; in particular the pause
  contract ("pause is durable and resumable", store.rs:595-597) must not be
  able to strand in-flight work. MASTER-PLAN's recovery claims cover Running
  runs only; the Paused-run variant is unaddressed.
- Observed behavior: run Paused + task Running with live claim survives
  restart untouched; resume re-enters the executor, which polls
  `external_progress_poll_interval` forever; the run never completes, never
  fails, and no in-process or restart path (not even boot recovery) can clear
  it — only manual file edits recover the run.
- Impact: a user-visible action (pause) followed by a crash leaves the run
  permanently stuck with zero recovery paths — strictly worse than canonical
  A-TSK-04-P1-02 (mid-wave fault), which boot recovery heals because the run
  is Running; the pause-during-wave regression (A-TSK-04-P1-01) is a sibling
  of this window and its fix direction (resolve_dispatch writes Pending when
  durably Paused) does not cover the crash case.
- Root cause: `request_pause` is not crash-atomic with the executor's task
  reset, and `recover_incomplete` scopes recovery to `Running` runs, so a
  Paused run can durably own a Running task that no code path ever resets.
- Direction: (a) in `recover_incomplete`, also scan Paused runs for
  Running/claimed tasks and reset them to Pending (claim-aware, reusing the
  existing Running-task reset logic at store.rs:1706-1750); and/or (b) make
  `request_pause` reset Running tasks to Pending in the same per-run locked
  section as the Paused transition (single event batch), eliminating the
  window; (c) add a typed "stranded" detection at resume that refuses to start
  the executor with in-flight tasks. No deletion needed.
- Regression validation: fixture "crash between request_pause and the
  Paused-branch reset -> boot recovery clears the strand (task Pending, claim
  cleared) and resume completes the run"; fixture "resume of a run with a
  stranded Running task fails with a typed error instead of polling forever".
- Validation reports: [V03-02](validations/Q-FLT-02/V03-02.md),
  [V03-01](validations/Q-FLT-02/V03-01.md)

### Canonical findings referenced as scenario evidence (NOT re-filed)

| Canonical ID | Scenario role | Current-code anchor re-verified |
|---|---|---|
| A-TSK-04-P1-01 / A-TSK-03-P1-01 | 3 (pause->cancel) | runtime_executor.rs:390-416, :416; executor.rs:643-661; types.rs:522; probe V03-01 |
| F-SUB-02-P1-01 / P1-02 | 4 (team cancel/timeout detach) | team/mod.rs:343-353; manager_subagent.rs:277-284; executor.rs:1082-1084; probe V04-01 |
| A-TSK-01-P1-01 / A-TSK-04-P1-03 | 5 (torn tail) | file_shadow.rs:362-379, :289-299; store.rs:1631-1639; probe V05-01 |
| A-TSK-05-P2-01 / P2-02 | 7 (worktree lock/sweep) | worktree.rs:381-391, :1001-1017, :1046-1070; probe V07-01 |
| A-TSK-04-P2-01 | 2/6 (unguarded `set_task_status` in finalize/recovery) | store.rs:953-983; executor.rs:651, :551; store.rs:1706 |
| A-TSK-04-P1-02 / F-TSK-03-P2-01 | 1/6 adjacent (mid-wave fault orphans) | runtime_executor.rs:379-381, :418-421; executor.rs:570-582 |
| X-TSK-01-P3-01 | 1/6 adjacent (fabricated-Pending read-back) | store.rs:694-696; file_shadow.rs:267-278 |
| F-SUB-02-P2-01 | 4 adjacent (timeout authority split, string-match classification) | team/mod.rs:345; executor.rs:1082-1084 |
| A-TSK-06-P2-01 / P3-01 / P3-02 / P3-03 | 8 adjacent (dead artifact/memory arms; unbounded full_output) | store.rs:1432-1447; memory_bridge.rs:231-250; store.rs:1964-1966 |

## Validation Matrix

| ID | Claim / scenario | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Scenario 1: stale revision writes rejected (claim CAS, tool layer, spec reset, terminal monotonicity) | yes | passed | [V01-01](validations/Q-FLT-02/V01-01.md) |
| V02 | Scenario 2: old attempt completion cannot regress terminal state (requeue/re-claim/stale writes/cancel durability) | yes | passed | [V02-01](validations/Q-FLT-02/V02-01.md) |
| V03 | Scenario 3: cancel in wave vs pause (pause lands durably; Paused->Cancelled regression legal; replay preserves it) | yes | failed (canonical regression reproduced) | [V03-01](validations/Q-FLT-02/V03-01.md) |
| V03 | Scenario 3b: crash between pause and reset strands Running+claimed task in Paused run | yes | failed (NEW P1-01) | [V03-02](validations/Q-FLT-02/V03-02.md) |
| V04 | Scenario 4: Team timeout detaches spawned members (mechanism probe; JoinSet contrast) | yes | failed (canonical detach reproduced) | [V04-01](validations/Q-FLT-02/V04-01.md) |
| V05 | Scenario 5: torn events.jsonl tail bricks reads and writes; recovery never begins | yes | failed (canonical brick reproduced) | [V05-01](validations/Q-FLT-02/V05-01.md) |
| V06 | Scenario 6: restart recovery Running->Paused, task requeue, completed preserved, no re-dispatch | yes | passed | [V06-01](validations/Q-FLT-02/V06-01.md) |
| V07 | Scenario 7: worktree lock residue blocks acquire forever; dirty retention/reuse work; no fork sweep | yes | failed (canonical P2-01/P2-02 reproduced) | [V07-01](validations/Q-FLT-02/V07-01.md) |
| V08 | Scenario 8: failed review retry/acceptance semantics (breaker, fix task, claim-guarded requeue) | yes | passed | [V08-01](validations/Q-FLT-02/V08-01.md) |

All probes executed with exit code 0; probe sources under `/tmp/qflt02-probe/
src/bin/`, full outputs (persisted before/after snapshots) under
`/tmp/qflt02-probe/out/*.log`. One-time probe build ~10 min (full
`echo-agent-app-core` dependency tree), subsequent builds < 40 s.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Convergence doc: "completion/failure/block/retry writes carry the claim; stale writes Superseded" | current (completion/retry) / regressed in part (block via unguarded `set_task_status`, canonical A-TSK-04-P2-01) | V01-01/V02-01 probes; store.rs:1032-1105 vs :953-983 |
| Convergence doc: "every revision commit preserves in-flight execution; reset clears claim" | current | V01-01 (live rev-1 claim completion accepted after rev-2 bump; spec reset clears claim); event_rebuild.rs:147-193 |
| store.rs:595-597: "pause keeps the run resumable" | regressed (in-wave path) + gap (crash window) | V03-01 (pause -> permanent cancel); V03-02 (pause-crash strand, NEW P1-01) |
| MASTER-PLAN (app): boot recovery "Running -> Paused, replay-safe tasks requeued, completed tasks preserved" | current for Running runs | V06-01; store.rs:1631-1776 |
| file_shadow.rs:114-117: "future hardening pass will truncate a partial tail" | regressed (unimplemented) | V05-01 (bricking reproduced) |
| MASTER-PLAN (root): "主运行被停止时，后台 Subagent 必须同步取消，不得继续脱离运行" | regressed (Team path) | V04-01 (detach reproduced); F-SUB-02-P1-01/P1-02 |
| MASTER-PLAN (app): "worktree repair … complete" | current for legacy unattended surface / regressed for fork namespace | V07-01; A-TSK-05-P2-01/P2-02 |
| A-TSK-06: review consumes complete output, persisted on the terminal boundary, reused after restart | current (not re-executed; referenced) | A-TSK-06 V02-01/V04-04 |
| F-SUB-02: required team partial-failure/cleanup fixtures do not exist | current (test gap) | V04-01 mechanism probe covers the defect class, not a framework fixture |

## Coverage And Uncertainty

- All dynamic evidence comes from /tmp probes driving the real
  `TaskRuntimeStore`, `FileTaskShadow`, `RunWorktree`, and review helpers over
  fixtures — not from the product binary. The framework executor wave/cancel
  behavior (V03-01's in-wave hardcoded Cancelled, V03-02's external polling)
  is a deterministic code trace anchored at runtime_executor.rs:390-416 and
  :275-285; the EKO controller is private so the live executor was not driven.
- V03-02's trigger (crash in the pause window) is a real but narrow window;
  the strand state itself is dynamically proven (store level), and the
  "poll forever" consequence is the same framework trace already accepted for
  canonical A-TSK-04-P1-02.
- V07 uses a real git repo; GPG signing was disabled in the fixture (the
  machine's global `commit.gpgsign=true` blocks on a passphrase prompt, the
  known environment issue recorded in AGENTS.md). Framework-side finalize was
  mirrored manually (unlock+retain), not invoked.
- V08 did not run a real reviewer LLM; `review_task`'s prompt/JSON path and
  the restart-equivalence of review input are covered by A-TSK-06 V04-04.
- Non-replay-safe (mutating) recovery blocking (`store.rs:1697-1750`) was not
  exercised dynamically; covered by A-TSK-04 tests.
- The X-TSK-01-P3-01 plan/run-state divergence window (crash between the two
  projection renames, file_shadow.rs:267-278) is referenced from V06, not
  re-probed (the fabrication lives in the private
  `load_revisioned_task_graph`).
- Cross-process dual-driver behavior (GUI+TUI on one run) was not exercised;
  canonical A-TSK-01-P3-02 / A-TSK-04-P2-01 cover that edge.

## Handoff

- Downstream tasks may rely on: the claim/revision kernel is enforced and
  probed end to end (V01/V02/V06/V08); the pause family has two distinct
  violations (in-wave regression — canonical A-TSK-04-P1-01; crash-window
  strand — NEW Q-FLT-02-P1-01); the Team timeout/cancel detach class
  (F-SUB-02-P1-01/P1-02) and the torn-tail bricking (A-TSK-01-P1-01) are
  dynamically confirmed; worktree happy path is sound, crash lock residue and
  fork leak are canonical P2-01/P2-02; review/acceptance semantics are sound.
- Reports to read: the 9 validation reports above (immutable, with persisted
  before/after snapshots in /tmp/qflt02-probe/out/); dependency reports
  F-TSK-03, F-SUB-02, A-TSK-04, A-TSK-05, A-TSK-06, X-TSK-01.
- Stale conditions: this report becomes stale if `runtime_executor.rs` wave/
  cancel/interruption handling, EKO `request_pause`/`recover_incomplete`/
  `finalize_cancelled_run_state`, `file_shadow.rs` append/read/seq behavior,
  `review.rs` breaker or `run_review_gate`, `worktree.rs` acquire/lock
  semantics, or the `TaskRunStatus` transition table change; also if a
  recovery sweep covering Paused runs appears (P1-01 fixed), a torn-tail
  repair lands (A-TSK-01-P1-01 fixed), or Team cancellation tokens land
  (F-SUB-02-P1-01/P1-02 fixed).
- Follow-up task IDs: S-RDM-01 (roadmap: Q-FLT-02-P1-01 recovery sweep or
  atomic pause; plus the canonical fixes A-TSK-04-P1-01, A-TSK-01-P1-01,
  F-SUB-02-P1-01/P1-02, A-TSK-05-P2-01/P2-02); X-STA-01 (identity continuity
  across restart — V06 evidence); X-EVT-01 (cancel/timeout event conformance —
  V03/V04 evidence).
