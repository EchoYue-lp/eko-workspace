# A-TSK-04: Claims, revisions, recovery, and terminal monotonicity

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Primary acceptance: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: CLI clean; framework had external changes in `echo-core/src/utils`, `echo-state/src/memory`, `echo-tools` worktree files, `src/eval`, `src/evolution`, and `src/trace`; none intersects inspected `echo-orchestration` task files and no external diff was read

## Question

Can stale revisions/attempts, cancellation, restart, and event replay update EKO task state only through valid claims without terminal regression?

## Scope

- Framework `TaskClaim`, `RuntimeDagController`, and dependency-block callback contract.
- EKO file-backed claim acquisition/settlement/requeue, broad status mutation, retry and recovery decisions.
- Boot recovery selection/write order and background resume consumption.
- Event append/sequence/fold and task/Subagent hook dispatch ordering.
- Existing static tests for claim races, recovery, replay, hooks, and terminal transitions.

## Out Of Scope

- Framework claim ABA, forced-abort settlement, wave error settlement, and generic per-claim retry: F-TSK-03.
- EKO completion/pause/cancel race, split run settlement, dispatch retry classification, and cross-TaskRun relation loop: A-TSK-03.
- General malformed/partial event authority and projection repair: A-TSK-01.
- Worktree policy, surface presentation, source fixes, builds, tests, fixtures, and network.

## Inputs

- Root AGENTS.md; shared README/REPORTING/TASKS; Codex README and templates.
- Exact Codex dependency reports A-TSK-03 and F-TSK-03.
- Current clean relevant source at the commits above.
- V00 discloses a narrow source-planning-document path overreach; no exposed claim is used.

## Layering Decision

| Classification | Current answer |
|---|---|
| Generic mechanism | Unique durable claim generation, revision-safe controller callbacks, claim abandon/settlement, and retry safe points belong in `echo-agent`. |
| EKO product policy | File persistence, boot recovery choice, user-confirmed recovery Retry/Skip, background auto-resume, and configured lifecycle hooks belong in `echo-agent-cli`. |
| Adapter boundary | EKO should perform atomic field conversion/persistence and return Applied/Superseded; it must not create a second relation, retry, or terminal-settlement authority. |
| Duplicate search | Searched claim/revision/spec/attempt/retry/recovery/block/status/event/seq/replay/hook definitions and live call paths across both repositories. One DAG relation kernel remains. |
| Migration deletion | Extend the existing controller/store primitives. Delete broad unclaimed `set_task_status` use from revision-derived blocking and recovery Retry once fenced primitives replace those calls; do not introduce another store or executor. |

No SQLite or public-service permission boundary is involved.

## Current Path

```text
RuntimeDagExecutor safe-point snapshot(revision/spec/status)
  -> claim_task(expected revision/spec) -> persisted Running claim
  -> dispatch -> set_claimed_task_status/requeue_claimed_task exact-claim CAS
  -> failed dependency -> block_task (currently loses snapshot revision)

process bootstrap -> recover_incomplete
  -> Running run -> Paused run -> reset each Running task / record blocker
  -> background resume -> standard RuntimeDagExecutor

append_event_line(seq, durable JSONL)
  -> rebuild plan/run projection in append order
  -> live in-memory HookEventDispatcher, only after attachment
```

The normal dispatch result path is claim-fenced. The deviations are all claim-external control paths: stale dependency blocking, multi-step boot repair, recovery Retry, and hook delivery after durable append.

## Findings

### A-TSK-04-P1-01: Dependency blocking can write through a stale plan revision

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:124`, `:213`, `:239`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:1564`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:953`
- Reachability: runtime executor loads revision N and detects failed dependency -> concurrent revision N+1 retains the downstream ID but removes/changes the dependency -> old `block_task` callback -> EKO broad status write.
- Expected invariant: every mutation derived from a loaded graph is conditional on that graph revision/spec/status; an obsolete callback returns Superseded and reloads.
- Observed behavior: `block_task` receives no revision/claim. EKO checks only that the task ID exists, writes Blocked into the latest graph, and clears the claim.
- Impact: a valid newer plan can be made unschedulable by an obsolete dependency conclusion, violating revision authority and requiring another edit/retry to recover.
- Root cause: the framework callback contract discards the safe-point revision and the adapter uses the unrestricted status writer.
- Direction: pass expected revision plus snapshot task identity to a store CAS and return Applied/Superseded so the framework reloads. Delete this `set_task_status` call after migration; retain the single framework dependency owner.
- Regression validation: pause the callback after loading N, patch the task/dependency at N+1, release it, and assert N+1 remains Pending with no Blocked event.
- Validation reports: [V04](../validations/A-TSK-04/V04-01.md), [V10](../validations/A-TSK-04/V10-01.md)

### A-TSK-04-P1-02: Interrupted boot recovery can strand an orphaned Running claim permanently

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1631`, `:1653`, `:1660`, `:1706`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/service.rs:556`
- Reachability: real headless/AppState bootstrap -> recovery of Running run -> durable Paused transition -> process stop or one task-reset write error -> next bootstrap/background auto-resume.
- Expected invariant: recovery is idempotent at every durable boundary and resume cannot start while a persisted Running task has no live driver.
- Observed behavior: only Running runs are scanned, but Paused is persisted before task cleanup. Cleanup failures are logged and continued. The next boot ignores Paused, and absence of a blocker lets background resume a snapshot that still treats the orphan as in flight.
- Impact: a restarted task run can poll/stall indefinitely with no Subagent executing the Running node, while recovery reports the run handled.
- Root cause: boot recovery is a non-transactional sequence whose selection predicate is invalidated by its first durable write.
- Direction: make run/task repair one idempotent store operation, or continue scanning Paused runs containing orphaned Running claims and reject resume until every claim is settled. Return successful recovery count, not initial candidate count. Do not add a second cleanup owner beside the A-TSK-03 settlement repair.
- Regression validation: stop/fail after each durable recovery append and call recovery twice; the second call must finish cleanup and resume must see no orphaned Running claim.
- Validation reports: [V06](../validations/A-TSK-04/V06-01.md), [V10](../validations/A-TSK-04/V10-01.md)

### A-TSK-04-P1-03: Recovery Retry bypasses max_retries and reuses the attempt number

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1010`, `:1211`, `:1253`, `:1264`, `:2132`, `:2162`
- Reachability: interrupted mutating task -> boot recovery Blocked -> user chooses Retry through service/surface -> `resolve_recovery_task` -> standard resume/reclaim.
- Expected invariant: each physical retry consumes the one declared retry budget and receives a fresh attempt/lease identity.
- Observed behavior: normal retry checks/increments the budget; recovery Retry performs only an unclaimed Pending write. The next claim derives the unchanged `retry_count + 1`, so it can repeat the prior attempt and recovery can exceed `max_retries`.
- Impact: recovery can execute side effects more times than configured and cannot distinguish stale results from the new physical dispatch. The claim-token equality consequence remains canonical F-TSK-03-P1-01.
- Root cause: recovery decision persistence bypasses the existing guarded retry primitive.
- Direction: route Recovery Retry through one atomic retry-budget/attempt-generation primitive under the run lock, then resume through the standard driver. Delete the plain Pending status mutation from this branch.
- Regression validation: recover attempts at zero/partial/exhausted budget and assert exact count, distinct durable execution identity, stale old completion rejection, and no retry beyond max.
- Validation reports: [V05](../validations/A-TSK-04/V05-01.md), [V10](../validations/A-TSK-04/V10-01.md)

### A-TSK-04-P2-01: Lifecycle hook delivery has no durable replay cursor

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:153`, `:165`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:215`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/register.rs:106`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/hook_event_dispatcher.rs:69`
- Reachability: every task/Subagent event append -> optional attached callback -> bounded in-memory queue -> configured hook bridge.
- Expected invariant: if hooks are lifecycle consumers of durable events, append-before-notify uses a durable cursor/replay contract so restart preserves order without silent omission.
- Observed behavior: attachment explicitly omits earlier events; a crash after append or before queued fire leaves no cursor and startup does not scan `events.jsonl`. Backpressure/order only protects the current in-memory session.
- Impact: configured lifecycle automation/observability can permanently miss TaskStarted/Completed or SubagentStart/Stop despite the authoritative event existing.
- Root cause: durable event storage feeds a live-only side-effect queue without a declared delivery checkpoint.
- Direction: first make durability semantics explicit. If durable delivery is required, persist a per-consumer seq cursor and stable delivery ID, replay in order, and require idempotent hook handling. Retain the existing event log as the only source; do not add another lifecycle ledger.
- Regression validation: terminate after append before enqueue/fire, restart/reattach, and assert ordered at-least-once delivery with dedup by stable ID.
- Validation reports: [V09](../validations/A-TSK-04/V09-01.md), [V10](../validations/A-TSK-04/V10-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Exact-input isolation disclosure | yes | inconclusive | [V00](../validations/A-TSK-04/V00-01.md) |
| V01 | Definition/duplicate authority search | yes | passed | [V01](../validations/A-TSK-04/V01-01.md) |
| V02 | Registration and runtime reachability | yes | passed | [V02](../validations/A-TSK-04/V02-01.md) |
| V03 | Claim identity persistence and normal stale-write CAS | yes | passed | [V03](../validations/A-TSK-04/V03-01.md) |
| V04 | Revision fencing of dependency-block writes | yes | failed | [V04](../validations/A-TSK-04/V04-01.md) |
| V05 | Retry/attempt/budget path comparison | yes | failed | [V05](../validations/A-TSK-04/V05-01.md) |
| V06 | Crash/restart recovery idempotence trace | yes | failed | [V06](../validations/A-TSK-04/V06-01.md) |
| V07 | Cancellation/terminal monotonicity and ownership dedup | yes | passed | [V07](../validations/A-TSK-04/V07-01.md) |
| V08 | Event append/fold ordering | yes | passed | [V08](../validations/A-TSK-04/V08-01.md) |
| V09 | Hook delivery replay ordering | yes | failed | [V09](../validations/A-TSK-04/V09-01.md) |
| V10 | Existing test and edge-case inventory | yes | failed | [V10](../validations/A-TSK-04/V10-01.md) |
| V11 | Targeted executable interleavings | policy-deferred | not_run | [V11](../validations/A-TSK-04/V11-01.md) |
| V12 | Report/link/executor/source integrity gate | yes | attempt 1 inconclusive; attempt 2 passed | [A1](../validations/A-TSK-04/V12-01.md), [A2](../validations/A-TSK-04/V12-02.md) |
| V30 | Primary source-anchor and state-transition sample | yes | passed | [V30](../validations/A-TSK-04/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-TSK-03: EKO normal changed-tuple claim CAS exists, but physical claim identity can repeat | current; canonical in dependency | [V03](../validations/A-TSK-04/V03-01.md), [V05](../validations/A-TSK-04/V05-01.md) |
| F-TSK-03: forced abort/wave errors can leave claims unsettled | current; canonical in dependency | [V07](../validations/A-TSK-04/V07-01.md) |
| A-TSK-03: completion interruption arbitration and fallible run settlement are split | current; canonical in dependency | [V07](../validations/A-TSK-04/V07-01.md) |

## Coverage And Uncertainty

- This is a pure static review. No Cargo/rustc/test/build/fixture/network process ran; V11 is explicitly `not_run`.
- V00 records a narrow exact-input deviation. Findings use only current source and authorized dependency reports.
- The framework had unrelated external dirty paths at finalization. No inspected `echo-orchestration` task file was dirty; CLI stayed clean.
- Hook delivery priority assumes configured lifecycle hooks are expected to observe durable task events. If the product explicitly defines hooks as live-only, P2-01 becomes a documentation/contract decision rather than a replay implementation requirement.
- A-TSK-01 remains the owner for malformed/partial authoritative event-log recovery; this task verifies normal ordered folding only.

## Handoff

- Fix ordering: unique/fenced claim contract from F-TSK-03 plus stale block callback; idempotent boot repair and one fallible settlement owner; unified recovery retry budget; then decide hook durability.
- Preserve one `RuntimeDagExecutor`, one revisioned TaskRun graph, and one file event authority.
- Downstream A-STATE-01/A-SRF-02/A-CHAT-02/A-OPS-01/X-TSK-01 should read this report plus A-TSK-03/F-TSK-03 rather than recreating claim or cleanup owners.
- This report becomes stale if framework `TaskClaim`/`RuntimeDagController`, EKO store recovery/retry/status methods, event fold/append, hook dispatcher/registration, or A-TSK-03 settlement paths change.
