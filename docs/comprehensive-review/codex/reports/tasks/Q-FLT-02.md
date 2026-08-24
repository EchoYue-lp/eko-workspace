# Q-FLT-02: Task and Subagent fault-injection suite

> Status: needs_evidence
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: committed reports/source boundary; no fixture executed

## Question

Do DAG, claim and Subagent invariants survive stale revisions, old attempts,
cancel, timeout, crash, restart, worktree conflict, and failed review?

## Answer

Not dynamically established. Ten deterministic scenario families below specify
the exact before/fault/after/restart facts needed to close the accepted framework
and EKO Task/Subagent findings. They are intentionally `not_run` under the user's
no-compilation/no-testing instruction.

## Invariant Contract

- One revisioned `TaskRun -> PlanTask -> SubagentRun` authority.
- Revision/claim/attempt tokens are monotonic, non-reusable and checked at every
  side-effect/settlement boundary.
- Every selected claim reaches one Completed/Failed/Cancelled/TimedOut/Abandoned
  terminal or one durable recoverable in-flight state.
- Cancellation/timeout joins all owned Subagent/task/worktree operations.
- Completed sibling work and safe points survive another sibling's failure.
- Restart produces an all-old or all-new graph/projection and never strands a
  Paused run with Running claims.
- Merge/review/artifact effects are attempt-bound and verify current ownership.
- Retry feedback enters the next attempt and increments a bounded budget.
- Worktree/data-workspace identity, locator and cleanup remain recoverable.

## Scenario Matrix

| ID | Fault family | Required persisted oracle | Status |
|---|---|---|---|
| V01 | stale revision after authoring/bootstrap | rejected mutation leaves no started/orphan product run | not_run |
| V02 | claim ABA/old attempt late result | late settlement/merge rejected by new token | not_run |
| V03 | cancel queued/running wave/Team/Fork | every claim/Subagent settles and joins | not_run |
| V04 | timeout during queue/dispatch/member/worktree | TimedOut identity and bounded cleanup | not_run |
| V05 | crash during commit/recovery/checkpoint | all-old/all-new or quarantined; restart continues safely | not_run |
| V06 | sibling panic/dispatch/store failure | completed siblings safe-pointed; failed sibling typed | not_run |
| V07 | DAG skipped/paused/retrying/transitive failure | coherent frontier/run disposition, no spin/stall | not_run |
| V08 | worktree branch/merge/conflict/ownership | attempt-bound target/base and no unrelated data loss | not_run |
| V09 | failed review/NeedsFix/retry feedback | feedback reaches next attempt; budget increments | not_run |
| V10 | artifact/workspace handoff and cleanup | durable locator, correct required artifact, bounded retention | not_run |

## Findings

No new findings. This task is a fault-execution backlog over F-TSK-03,
F-SUB-02, A-TSK-04..06 and X-TSK-01; canonical IDs remain there.

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Dependency/commit/source boundary | yes | passed | [V00](../validations/Q-FLT-02/V00-01.md) |
| V01 | Stale revision/bootstrap | yes | not_run | [V01](../validations/Q-FLT-02/V01-01.md) |
| V02 | ABA/old attempt | yes | not_run | [V02](../validations/Q-FLT-02/V02-01.md) |
| V03 | Cancellation | yes | not_run | [V03](../validations/Q-FLT-02/V03-01.md) |
| V04 | Timeout | yes | not_run | [V04](../validations/Q-FLT-02/V04-01.md) |
| V05 | Crash/restart | yes | not_run | [V05](../validations/Q-FLT-02/V05-01.md) |
| V06 | Sibling/infrastructure failure | yes | not_run | [V06](../validations/Q-FLT-02/V06-01.md) |
| V07 | DAG status/frontier | yes | not_run | [V07](../validations/Q-FLT-02/V07-01.md) |
| V08 | Worktree conflict/merge | yes | not_run | [V08](../validations/Q-FLT-02/V08-01.md) |
| V09 | Review failure/retry feedback | yes | not_run | [V09](../validations/Q-FLT-02/V09-01.md) |
| V10 | Artifact/workspace handoff | yes | not_run | [V10](../validations/Q-FLT-02/V10-01.md) |
| V11 | Coverage/canonical-owner check | yes | passed | [V11](../validations/Q-FLT-02/V11-01.md) |
| V99 | Integrity/isolation/status | yes | passed | [V99](../validations/Q-FLT-02/V99-01.md) |

## Handoff

Keep `needs_evidence` until V01-V10 run at current source. Implement generic
revision/DAG/claim/settlement fixes in the framework; keep worktree/review/
artifact/resource policy in EKO. Every future validation must record pre-fault,
post-fault and post-restart snapshots plus physical side-effect count.
