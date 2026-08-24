# F-TSK-03: Runtime DAG execution and claims

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: no tracked/staged source diff at task start; external generated CLI `ApiError.ts` / `StreamingEvent.ts` changes were not read, modified, or reverted; only Codex reports changed

## Question

Does `RuntimeDagExecutor` correctly own revision safe points, bounded Subagent
waves, unique claims/attempts, retry, cancellation, crash recovery, and terminal
settlement?

## Scope

- RuntimeDagExecutor/controller contracts and normal/error/cancel wave paths.
- TaskClaim identity, constructors, equality, execution ID and state fencing.
- Framework ManagedTask controller single-task retry/timeout/hooks/store behavior.
- Narrow EKO controller/store inspection to prove the reusable contract against
  a durable implementation: claim CAS, recovery, resolution and replay guard.
- Current tests, authorized history, panic/UTF-8/overflow.

## Out Of Scope

- Task model/store authority and pause API: F-TSK-01.
- Skip semantics, transitive failure, Paused/Retrying restart classification,
  cycle traversal and duplicate readiness: F-TSK-02.
- General Subagent queue/background/team lifecycle findings: F-SUB-02.
- EKO-specific event replay/terminal monotonicity and adapter policy: A-TSK-04.
- Source fixes, Cargo, rustc, tests, builds, or dynamic fixtures.

## Inputs

- Root AGENTS; shared README/REPORTING/TASKS; Codex README/templates.
- Codex F-TSK-01, F-TSK-02, and F-SUB-02 complete dependency reports.
- Current source, scoped git/source searches, and `MASTER-PLAN.md` only.
- V00 discloses one broad history search whose output accidentally exposed
  snippets from other reviewer paths. Those snippets were not adopted; final
  evidence was independently reconstructed from authorized inputs.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Unique leases/fencing, claim settlement, bounded waves, revision safe points, cancellation grace, retry-attempt identity and terminal outcomes belong in the framework. |
| EKO product policy | File/worktree ownership, review, attended disposition, concrete limits, and UI events remain application policy. |
| Adapter boundary | Controller atomically claims/resolves and may select a conflict-free subset; it must not repair kernel-lost claims or run another traversal/retry authority. |
| Duplicate search | Searched RuntimeDagExecutor/controller/claim/lease/fence/attempt/retry/cancel/timeout/revision/safe-point/store/resume definitions and callers across both repos. One full-DAG kernel exists; two controllers differ in retry placement. |
| Migration deletion | Keep RuntimeDagExecutor as traversal/settlement owner. Move generic controller retries to kernel safe points and delete per-dispatch multi-attempt loops once claim renewal covers them. |

## Current Path

```text
execute(run, cancel)
  -> load/validate snapshot revision at loop safe point
  -> compute/select/validate ready wave
  -> per selected task: semaphore -> controller.claim_task(revision/spec/status)
     -> dispatch_task(context with child-capable cancel, claim, snapshot task)
  -> JoinSet collect entire normal wave
  -> controller.resolve_dispatch(claim, task, dispatch result)
  -> only then reload revision / decide stop
```

Normal waves are bounded and revision reload occurs after normal settlement.
`TaskClaim` is `(revision, attempt, spec_hash)` and its execution ID additionally
uses run/task. EKO persists it; generic TaskManager stores claims only in a
controller map while ManagedTask projection sets `claim: None`. EKO retries by
durably requeueing and reclaiming next loop; the generic controller dispatches a
single claim into a multi-attempt `run_task_with_retry` pipeline.

## Findings

### F-TSK-03-P1-01: TaskClaim is vulnerable to ABA after restart or same-attempt requeue

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/tasks/runtime.rs:210`;
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:985`,
  `:1620`, `:1692`; `task_runtime/executor.rs:1209`
- Reachability: EKO durable claim -> interrupted Running -> boot recovery Pending
  without retry bump -> same revision/spec claim -> stale external completion ->
  structural equality fence.
- Expected invariant: every physical lease/reclaim has an unrepeatable fencing
  token, independent of logical retry count.
- Observed behavior: unchanged revision, retry_count and spec reproduce the same
  claim and execution_id. Current-write checks compare equality only.
- Impact: a delayed completion from the old physical dispatch can update the
  newly claimed task or be reused as its result, violating terminal monotonicity
  and possibly accepting stale side effects.
- Root cause: logical coordinates double as lease epoch; no nonce/generation is
  persisted.
- Direction: add a durable monotonic claim generation or unique lease ID and
  include it in execution identity/all writes. Recovery must invalidate the old
  generation before requeue. Do not rely on process death as fencing.
- Regression validation: old remote completion arriving after restart/reclaim
  with unchanged logical attempt must return Superseded.
- Validation reports: [V02](../validations/F-TSK-03/V02-01.md),
  [V07](../validations/F-TSK-03/V07-01.md)

### F-TSK-03-P1-02: Forced cancellation abort has no per-claim settlement callback

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:368`,
  `:396`, `:416`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:1595`
- Reachability: active non-cooperative dispatch -> cancel -> grace expiry ->
  JoinSet abort -> cancelled JoinError discarded -> interruption outcome.
- Expected invariant: every acquired claim is resolved or durably abandoned
  before Cancelled/Paused returns.
- Observed behavior: aborted tasks produce no wave result, hence no
  `resolve_dispatch`; controller trait has no abandon/release method. EKO
  interruption merely classifies outcome. Running claims are repaired only by a
  later boot-recovery pass.
- Impact: current process can report cancellation while tasks remain Running;
  immediate resume/external observers see stale in-flight ownership and may poll
  indefinitely or require crash-style repair.
- Root cause: cancellation owns JoinSet handles but not a registry/settlement
  protocol for claims already acquired inside them.
- Direction: kernel must track every claimed token outside child futures and call
  a mandatory `abandon_claim`/cancel resolution after abort, then return only
  when persisted. Delete controller-specific broad cleanup as the primary fix.
- Regression validation: non-cooperative dispatch aborted after grace; snapshot
  must contain no Running claim before outcome is returned.
- Validation reports: [V03](../validations/F-TSK-03/V03-01.md),
  [V07](../validations/F-TSK-03/V07-01.md)

### F-TSK-03-P1-03: Wave infrastructure errors discard completed sibling safe points

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:368`,
  `:379`, `:382`, `:417`
- Reachability: siblings dispatch concurrently -> A completes -> B claim returns
  Err or panics -> JoinSet branch returns before resolution; similarly one
  resolve error precedes unresolved siblings.
- Expected invariant: every acquired claim reaches resolve/abandon once, and
  completed siblings become durable before any wave error escapes.
- Observed behavior: early `return Err` drops `wave_results` and remaining
  JoinSet; resolution loop uses `?`, dropping later siblings on first persistence
  error.
- Impact: successful side effects/results can replay; claimed tasks stay Running;
  a transient controller error corrupts the whole wave's recovery boundary.
- Root cause: collection and durable settlement are not a finally-style phase.
- Direction: collect child errors as values, drain/abort all handles, then settle
  every acquired claim; return aggregated infrastructure error only afterward.
- Regression validation: three siblings with success plus claim error, panic, or
  resolve error; assert all claims have durable terminal/unknown status.
- Validation reports: [V04](../validations/F-TSK-03/V04-01.md),
  [V07](../validations/F-TSK-03/V07-01.md)

### F-TSK-03-P1-04: Generic controller runs multiple retry attempts under one claim

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/tasks/executor.rs:1623`, `:1679`,
  `:927`, `:948`, `:1157`; `tasks/runtime.rs:218`
- Reachability: framework `TaskExecutor::execute_all` -> one manager claim ->
  `execute_selected_task` -> `run_task_with_retry` loops actual executions.
- Expected invariant: each actual execution attempt has a distinct claim and
  execution ID so stale outputs/effects are fenced independently.
- Observed behavior: dispatch ignores `_claim`; local `current_attempt` changes
  across retries but controller claim does not. Result/hooks can name attempt 2
  while resolution checks attempt-1 claim.
- Impact: retry side effects and terminal writes cannot be attributed/fenced per
  attempt; recovery cannot distinguish which physical execution produced output.
- Root cause: generic retry remained inside the single-task adapter after claims
  became a kernel/controller safe-point contract.
- Direction: return retryable resolution to Pending, durably advance count, and
  reclaim in the next kernel loop; delete the multi-attempt inner loop once hook
  decisions are adapted.
- Regression validation: first attempt fails after possible effect, second
  succeeds; claims/execution IDs/events must differ and stale first completion
  must be rejected.
- Validation reports: [V10](../validations/F-TSK-03/V10-01.md),
  [V07](../validations/F-TSK-03/V07-01.md)

### F-TSK-03-P2-05: Public retry configuration can panic in delay construction

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/tasks/executor.rs:67`, `:123`
- Reachability: public TaskExecutorConfig -> generic controller failure retry ->
  `retry_delay_for_attempt`.
- Expected invariant: public config and high attempt values cannot panic or
  produce invalid durations.
- Observed behavior: negative/NaN `retry_backoff_factor` can produce negative/NaN
  seconds passed to `Duration::from_secs_f64`, which panics; `attempt as i32`
  also wraps for high attempts.
- Impact: malformed configuration can terminate an embedding process during
  error recovery.
- Root cause: unrestricted float input and unchecked narrowing precede a
  panic-capable duration constructor.
- Direction: validate finite nonnegative factor, use checked/saturating integer
  exponent/budget arithmetic and return typed configuration error. If inner
  retry is removed per P1-04, keep one safe helper at the kernel retry owner.
- Regression validation: negative, NaN, infinity, zero, huge factor and high
  attempt values.
- Validation reports: [V06](../validations/F-TSK-03/V06-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Isolation incident disclosure | yes | inconclusive | [report](../validations/F-TSK-03/V00-01.md) |
| V01 | Authority and controller call graph | yes | passed | [report](../validations/F-TSK-03/V01-01.md) |
| V02 | Claim uniqueness and restart ABA | yes | failed | [report](../validations/F-TSK-03/V02-01.md) |
| V03 | Cancellation grace claim settlement | yes | failed | [report](../validations/F-TSK-03/V03-01.md) |
| V04 | Sibling settlement on wave errors | yes | failed | [report](../validations/F-TSK-03/V04-01.md) |
| V05 | Normal concurrency and safe point | yes | passed | [report](../validations/F-TSK-03/V05-01.md) |
| V06 | Panic/UTF-8/overflow inspection | yes | failed | [report](../validations/F-TSK-03/V06-01.md) |
| V07 | Existing test coverage inventory | yes | failed | [report](../validations/F-TSK-03/V07-01.md) |
| V08 | Authorized historical drift | yes | passed | [report](../validations/F-TSK-03/V08-01.md) |
| V09 | Targeted dynamic cases | policy-deferred | not_run | [report](../validations/F-TSK-03/V09-01.md) |
| V10 | Generic per-attempt claim trace | yes | failed | [report](../validations/F-TSK-03/V10-01.md) |
| V11 | Report/link/source integrity gate | yes | passed | [report](../validations/F-TSK-03/V11-01.md) |
| V12 | Post-gate link-check harness | yes | inconclusive | [report](../validations/F-TSK-03/V12-01.md) |
| V13 | Final report/link/source integrity gate | yes | passed | [report](../validations/F-TSK-03/V13-01.md) |
| V30 | Primary source-anchor sampling and acceptance | yes | passed | [report](../validations/F-TSK-03/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN M13: one full-DAG RuntimeDagExecutor | current | [V01](../validations/F-TSK-03/V01-01.md) |
| Revision changes apply at safe points | current on normal waves; incomplete on error/abort | [V05](../validations/F-TSK-03/V05-01.md), [V03](../validations/F-TSK-03/V03-01.md), [V04](../validations/F-TSK-03/V04-01.md) |
| Completed siblings are never replayed after stop/resume | incomplete/regressed for infrastructure failure | [V04](../validations/F-TSK-03/V04-01.md) |
| Claims/revision identity fence stale task completion | current for changed tuple, incomplete for ABA | [V02](../validations/F-TSK-03/V02-01.md) |

## Coverage And Uncertainty

- No dynamic cases ran by explicit instruction; V09 is `not_run`.
- F-SUB-02 owns internal Subagent dispatch cancellation/detachment. This task
  assumes controller dispatch may be non-cooperative and reviews kernel claim
  ownership around it.
- EKO A-TSK-04 must independently review file event replay and terminal
  monotonicity; application details here only prove generic contract impact.
- F-TSK-02's skip/failure/Paused/Retrying findings were not duplicated.
- V00 is an immutable isolation incident; no exposed third-party snippet was
  used by V01-V10 or this report.
- Primary independently sampled the claim identity, wave collection/abort,
  settlement, inner retry, and retry-delay anchors in V30; the report is
  accepted as `complete` without dynamic execution.

## Handoff

- A-TSK-04 and crash-resume synthesis must consume P1-01 through P1-03; adapter
  status CAS cannot compensate for a repeatable token or missing kernel callback.
- Remediation order: unique lease generation -> kernel-owned claimed-token
  registry/final settlement -> retry-at-safe-point -> panic-safe retry math.
- Preserve normal bounded wave and selected-wave validation from V05.
- This report becomes stale if TaskClaim fields/equality, RuntimeDagExecutor
  JoinSet/settlement, either controller claim/resolve/recovery, or TaskExecutor
  retry changes.
