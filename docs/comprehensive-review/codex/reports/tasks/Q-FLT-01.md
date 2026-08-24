# Q-FLT-01: ReAct and Tool fault-injection suite

> Status: needs_evidence
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: external dirty source excluded; report-only static planning

## Question

Do Agent and Tool invariants survive malformed LLM output, Unicode, huge
output, timeout, cancellation, disconnect, crash, and partial effects?

## Answer

Not yet established dynamically. This report converts the accepted ReAct,
Tool, artifact, terminal and recovery findings into ten executable fault
families with exact observable outcomes. No fixture was run because the user
explicitly stopped compilation/testing during review. Each family has its own
immutable `not_run` report rather than being mislabeled passed.

## Invariant Contract

Every scenario must satisfy all applicable facts:

1. one invocation/turn identity and one monotonic terminal;
2. no `FinalAnswer` after Error/Cancelled and no successful empty EOF fallback;
3. cancellation/disconnect stops upstream provider/Tool work within a bound;
4. requested and effective Tool inputs are distinguishable and persisted;
5. each call ID is non-empty, unique, and receives one paired typed outcome;
6. complete output is recoverable through a digest-bound artifact/cursor;
7. successful side effects reach a durable safe point before later model work;
8. restart never replays a completed effect or silently discards corrupt state;
9. all strings, counters, delays, and sizes accept adversarial typed values
   without panic, wrap, or UTF-8 boundary failure;
10. framework result, EKO persistence, and surface projection agree on the same
    effective invocation, terminal reason, bytes, artifact and identity.

## Scenario Matrix

| ID | Fault family | Canonical owners | Required oracle | Status |
|---|---|---|---|---|
| V01 | malformed provider envelope/tool JSON/call identity | F-LLM, F-RCT-04, F-EXT-01 | typed error, zero side effects, one terminal | not_run |
| V02 | Unicode and malformed string boundaries | Q-STA, F-EVO, F-CMP | no panic; typed fallback/error | not_run |
| V03 | huge output/channel saturation/artifact spill | F-RCT-03, F-EXT-01, X-TOL-01 | bounded flow plus complete digest-bound output | not_run |
| V04 | provider/tool/batch timeout | F-REL, F-RCT-04, X-TOL-01 | one timed-out outcome per call; no detached work | not_run |
| V05 | cancellation at provider/tool/finalization safe points | F-RCT-02..04, F-EXT-01 | one Cancelled terminal; bounded upstream stop | not_run |
| V06 | consumer/sink disconnect | F-RCT-02/03 | upstream cancellation and no fallback success | not_run |
| V07 | crash/corrupt checkpoint/restart | F-RCT-05 | fail closed; original bytes preserved; no replay | not_run |
| V08 | partial batch effects/checkpoint failure | F-RCT-04/05 | completed effects paired/persisted; no duplicate execution | not_run |
| V09 | intervention/hook/approval effective Tool rewrite | F-EXT-01, X-TOL-01 | actual/persisted/audit effective values agree | not_run |
| V10 | binary/paged/spilled Tool result across EKO | F-EXT-01, X-TOL-01 | complete model-visible and UI-recoverable result | not_run |

## Findings

No new findings. The absent execution evidence is the task result, while every
underlying defect remains owned by its atomic/cross-contract finding. This task
must not multiply those counts.

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Exact dependencies, commits and execution prohibition | yes | passed | [V00](../validations/Q-FLT-01/V00-01.md) |
| V01 | Malformed output and call identity | yes | not_run | [V01](../validations/Q-FLT-01/V01-01.md) |
| V02 | Unicode/string boundary | yes | not_run | [V02](../validations/Q-FLT-01/V02-01.md) |
| V03 | Huge output/backpressure/artifact | yes | not_run | [V03](../validations/Q-FLT-01/V03-01.md) |
| V04 | Timeout | yes | not_run | [V04](../validations/Q-FLT-01/V04-01.md) |
| V05 | Cancellation | yes | not_run | [V05](../validations/Q-FLT-01/V05-01.md) |
| V06 | Disconnect/EOF | yes | not_run | [V06](../validations/Q-FLT-01/V06-01.md) |
| V07 | Crash/corrupt restart | yes | not_run | [V07](../validations/Q-FLT-01/V07-01.md) |
| V08 | Partial effects/checkpoint | yes | not_run | [V08](../validations/Q-FLT-01/V08-01.md) |
| V09 | Effective Tool rewrite | yes | not_run | [V09](../validations/Q-FLT-01/V09-01.md) |
| V10 | Rich output/paging/artifact | yes | not_run | [V10](../validations/Q-FLT-01/V10-01.md) |
| V11 | Scenario coverage and canonical-owner gate | yes | passed | [V11](../validations/Q-FLT-01/V11-01.md) |
| V99 | Links, headers, IDs, isolation and status | yes | passed | [V99](../validations/Q-FLT-01/V99-01.md) |

## Handoff

- Keep `needs_evidence` until every V01-V10 family has executable immutable
  attempts at current source. A failed scenario remains evidence and receives a
  new attempt after the fix.
- Use deterministic scripted LLM/Tool/clock/store/sink primitives; provider
  network tests are separate compatibility checks, not substitutes.
- Execute negative controls that deliberately drop/reorder/duplicate the exact
  fact each scenario asserts.
