# F-CORE-01: Core identities, errors, and event envelope

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Are run/turn/message/tool/event identities and error semantics stable,
typed, and sufficient for independent consumers?

## Scope

- `echo-core/src/agent/event_envelope.rs` (515 lines, full read),
  `echo-core/src/error.rs` (435 lines, full read),
  `echo-core/src/agent/mod.rs:40-474`, `echo-core/src/agent/types.rs`,
  `echo-core/src/agent/builder.rs`.
- `echo-core/src/tools/mod.rs` (ToolFailure/ToolFailureCategory/
  ExternalRunContext excerpts).
- Producer/consumer sites: `echo-agent/src/agent/subagent/executor.rs`,
  `echo-agent/src/a2a/server.rs`, `echo-agent/src/lib.rs`;
  `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:478-545`,
  `surface_contract.rs`, `src/cli/channels.rs`.

## Out Of Scope

- ReAct engine internals (F-RCT-02/03), tool execution (F-EXT-01),
  persistence adapters (F-RCT-05), cross-consumer conformance (X-EVT-01).
- Full `Agent` trait body (mod.rs:474-990) — sampled only.

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`
  (F-CORE-01 card), `zcode-ds/README.md`.
- Dependency report: zcode-ds `B-ARCH-01` (facade ownership).

## Layering Decision

- Generic mechanism: all types under review are framework-core primitives
  (identity, error taxonomy, event transport) — correctly placed in
  `echo_core`.
- EKO product policy: none in this task; EKO is a consumer
  (chat_driver/surface_contract).
- Adapter boundary: chat_driver's `EventIdentity` construction is the
  application adapter providing stable run/turn ids.
- Duplicate search terms: `EventEnvelope`, `EventIdentity`,
  `envelope_event_stream`, `AgentEvent`, `ReactError`, `ToolFailure`,
  `ExternalRunContext`, `is_terminal`, `is_checkpoint` — single
  authoritative definitions in echo_core; no parallel identity model found.

## Current Path

Identity flows: EKO `drive_chat` builds `ExternalRunContext`
(conversation/run/turn) → `AgentInvocationContext.runtime` →
`EventIdentity::from_invocation` or direct construction → raw `AgentEvent`
stream wrapped by `envelope_event_stream` (chat_driver:538) or by the
subagent executor (executor.rs:1168) → typed `EventEnvelope` with
deterministic `event_id` and monotonic `sequence` → EKO sinks. Errors flow:
subsystem errors → `ReactError` (boxed, `#[non_exhaustive]`) → event
payload (`AgentEvent::Error{source,message}` or tool `ToolError{failure}`).

## Findings

### F-CORE-01-P3-01: `parent_event_id` child-invocation contract is never populated in production

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-core/src/agent/event_envelope.rs:21` (doc: "Parent event
  for a child invocation, such as a delegated subagent"); `:102`
  (`from_invocation` sets `parent_event_id: None`);
  `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:487` (None);
  `echo-agent/src/agent/subagent/executor.rs:1168` (uses from_invocation);
  only `surface_contract.rs:159` (test) sets it
- Reachability: every production construction site hardcodes `None`;
  `envelope_event_stream` assigns tool-level parents internally
  (`event_envelope.rs:147-152`) but never agent-level parents.
- Expected invariant: the documented "child invocation" correlation works,
  or the field is documented as tool-correlation-only.
- Observed behavior: delegated subagent streams carry `parent_event_id =
  None`; association must be derived from run_id/execution_id.
- Impact: consumers cannot rebuild the invocation tree from
  `parent_event_id`; the field's doc is misleading; EKO works around it via
  `subagent_run_id = {run_id}:{task_id}:{plan_revision}:{attempt}`
  (MASTER-PLAN), so impact is limited but the contract is inconsistent.
- Root cause: the envelope design anticipated parent linking; the invocation
  plumbing (value-carried ExternalRunContext) made it unnecessary and the
  field was left unpopulated.
- Direction: either populate it in `from_invocation` when the invocation
  carries a parent id, or re-document the field as tool-correlation-only;
  X-EVT-01 should decide based on consumer needs.
- Regression validation: a delegated-subagent fixture asserting
  parent_event_id on child envelopes (or the corrected doc contract).
- Validation reports: [V02](../validations/F-CORE-01/V02-01.md),
  [V03](../validations/F-CORE-01/V03-01.md)

### F-CORE-01-P3-02: `from_invocation` turn_id fallback silently breaks event_id determinism

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-core/src/agent/event_envelope.rs:92-96` (turn_id falls
  back to `Uuid::new_v4()` when runtime has no turn/execution/run id);
  `:64-84` (event_id = hash of identity + sequence); `:119-122` (doc:
  "deterministic event_id generation lets persistence adapters reject
  duplicate side-effect completion events idempotently")
- Reachability: any consumer calling `from_invocation` without a populated
  `ExternalRunContext` produces a fresh random turn_id per call; EKO's main
  path always supplies turn_id (chat_driver:483-488) so it is unaffected
  today.
- Expected invariant: `event_id` is reproducible for the same logical
  event, per the idempotency doc.
- Observed behavior: the fallback produces a non-reproducible identity and
  event_id; no warning or doc guidance for callers.
- Impact: framework consumers that skip identity plumbing silently lose
  idempotent-persistence guarantees; the failure is invisible until a
  duplicate side-effect event is replayed.
- Root cause: convenience fallback in `from_invocation` conflicts with the
  determinism contract.
- Direction: document the contract (turn_id must be stable per invocation)
  or return a distinguishable "anonymous" identity that persistence
  adapters can detect; add a test asserting determinism for
  fully-populated vs. empty runtimes.
- Regression validation: unit test that `from_invocation` with an empty
  runtime yields a different event_id for the same sequence (documenting
  the current behavior) or, after the fix, a stable one.
- Validation reports: [V03](../validations/F-CORE-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Type/variant inventory | yes | passed | [V01](../validations/F-CORE-01/V01-01.md) |
| V02 | Producer-consumer reachability | yes | passed | [V02](../validations/F-CORE-01/V02-01.md) |
| V03 | Identity collision/ordering inspection | yes | passed | [V03](../validations/F-CORE-01/V03-01.md) |
| V04 | Serialization round-trip tests | yes | passed | [V04](../validations/F-CORE-01/V04-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root MASTER-PLAN: "框架已有 versioned EventEnvelope、稳定 sequence/event identity、tool parent identity 和 terminal exactly-once 校验" | current (agent-level parent link unused) | [V03](../validations/F-CORE-01/V03-01.md), [V04](../validations/F-CORE-01/V04-01.md) |
| Root MASTER-PLAN: cancellation/timeout are typed variants, not text | current (AgentError::Cancelled/Timeout; ToolError::Timeout; ToolFailure::Cancelled/Timeout) | [V01](../validations/F-CORE-01/V01-01.md) |

## Coverage And Uncertainty

- `AgentEvent::Error` carries only `source`+`message` (no structured class);
  tool errors are structured via ToolFailure. Whether agent-level error
  classification is needed by consumers is an open question for X-EVT-01.
- `AgentPhase` has no serde derive (local mapping only) — intentional per
  its doc, not a gap.
- The `Agent` trait body (:474-990) was not fully reviewed (F-RCT-01/F-API-01
  scope).

## Handoff

- Downstream tasks may rely on: envelope invariants (V04), identity flow
  (V02), the two P3 findings.
- `F-RCT-05` (resume/steer) should treat event_id determinism as the
  persistence contract and evaluate P3-02's impact.
- `X-EVT-01` should decide the parent_event_id contract (P3-01) and
  whether agent-level Error classification is needed.
- This report becomes stale if the envelope, error enum, or identity
  construction changes.
