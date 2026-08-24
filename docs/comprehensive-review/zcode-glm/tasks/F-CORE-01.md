# F-CORE-01: Core identities, errors, and event envelope

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean

## Question

Are run/turn/message/tool/event identities and error semantics stable,
typed, and sufficient for independent consumers?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-core/src/agent/mod.rs` — `AgentEvent`, `AgentPhase`,
  `BudgetDecision`, `StepType`, `Agent` trait, `AgentCallback`,
  `AgentInvocationContext`, `cancel_aware_stream`.
- `echo-agent/echo-core/src/agent/event_envelope.rs` — `EventEnvelope`,
  `EventIdentity`, `AGENT_EVENT_SCHEMA_VERSION`, `stable_event_id`,
  `envelope_event_stream`, `envelope_event_stream_after`,
  `validate_event_trajectory`.
- `echo-agent/echo-core/src/agent/intervention.rs` — `InterventionCallback`,
  `InterventionResult`, sentinel `CallbackBridge`.
- `echo-agent/echo-core/src/agent/types.rs` — `Critique`, `CritiqueOutput`.
- `echo-agent/echo-core/src/error.rs` — `ReactError` hierarchy.
- `echo-agent/src/error.rs` — facade re-export.
- `echo-agent/src/event_bus.rs` — `EventBus`, `GLOBAL_EVENT_BUS`.
- Cross-checks: `echo-agent/src/a2a/server.rs`,
  `echo-agent/src/agent/subagent/executor.rs`,
  `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs`,
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs`,
  `echo-agent-cli/echo-agent-app-core/src/surface_contract.rs`,
  `echo-agent-cli/src/cli/channels.rs`.

## Out Of Scope

- Concrete `ReactAgent` implementation of the `Agent` trait — deferred to
  the agent-runtime task.
- Task / TaskRun / PlanTask identities in `echo-orchestration` and the
  application `task_runtime` — deferred to framework and application Task
  review tasks.
- Webhook / trace / UI event projection mapping in the application layer —
  deferred to application event-projection tasks.
- Subagent lifecycle event taxonomy (`SubagentEventBus`,
  `SubagentRunState`) — deferred to the Subagent task.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — especially the framework-vs-application layering
    gate and the dead-code/dead-infra cleanup rule.
  - `docs/comprehensive-review/REPORTING.md`.
  - `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
- Dependency task reports read: none (this is the first Phase-F task
  executed in this reviewer directory; `B-ARCH-01`, `B-BASE-01`,
  `B-PATH-01` reports from this reviewer were available for cross-checking
  crate structure but no direct dependency claims are reused).
- Historical documents treated as hypotheses: none.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `EventEnvelope`, `EventIdentity`, `AgentEvent`, the `ReactError` hierarchy, `Agent`/`AgentCallback`/`InterventionCallback` traits describe generic agent-runtime concepts that any `echo-agent` consumer (CLI, third-party headless, future reuse) needs. They live correctly in `echo-core` (V01 confirms single definition site; `echo-agent/src/error.rs:10-14` only re-exports). |
| EKO product policy | None at this layer. Application-specific identities (`message_id`, `isolation_id`, `delegation_policy`) are kept out of `EventIdentity` and live on `ExternalRunContext` in `echo-core` and on `ChatDriverEvent` in the application. |
| Adapter boundary | The framework exposes `EventEnvelope` as the wire contract; the application adapter in `chat_driver.rs:538` wraps the framework stream unchanged. `surface_contract.rs:162` and `channels.rs:756` construct envelopes directly only for synthesized events — the adapter is thin and lossless. |
| Duplicate search | Searched names: `EventEnvelope`, `EventIdentity`, `AgentEvent`, `BudgetDecision`, `AgentPhase`, `StepType`, `ReactError`, `EventBus`, `GLOBAL_EVENT_BUS`, `envelope_event_stream`. Searched traits: `Agent`, `AgentCallback`, `InterventionCallback`. Searched fields: `run_id`, `turn_id`, `execution_id`, `conversation_id`, `parent_event_id`, `event_id`, `sequence`. Result: no duplicate definition of the same semantics inside `echo-core` for these concerns. |
| Migration deletion | No migration proposed in this task. The only deletion candidate is dead infra (`GLOBAL_EVENT_BUS` / `EventBus`) — see finding F-CORE-01-P2-01. |

## Current Path

Verified identity/event/error data flow at commit `9b0e0fa`:

1. **Identity construction.**
   - Chat path: `chat_driver.rs:211-216` derives `turn_id` from
     `root_message_id` (or fresh uuid on empty). `chat_driver.rs:483-489`
     builds an `EventIdentity { conversation_id, run_id, turn_id,
     execution_id: None, parent_event_id: None }` per turn.
   - Subagent path: `subagent/executor.rs:1168` calls
     `EventIdentity::from_invocation(&invocation)`. When `turn_id` is None
     (the typical subagent case), `from_invocation`
     (`event_envelope.rs:91-96`) falls back to `execution_id`, then
     `run_id`, then a fresh uuid. The executor forces a unique
     `execution_id` at `subagent/executor.rs:802-825`
     (`agent_tool-{uuid}`), so concurrent forks get distinct identity
     spaces.
   - A2A path: `a2a/server.rs:211-217` sets all three of `run_id`,
     `turn_id`, `execution_id` to the same `task_id` (unique per A2A task).
   - Task-runtime path: `task_runtime/executor.rs:3136-3139` and `:3743`
     wrap the agent stream with `envelope_event_stream(raw_stream,
     event_identity)`.

2. **Event production.** `envelope_event_stream_after`
   (`event_envelope.rs:123-194`) is the single production point. Per
   incoming `AgentEvent` it: increments `sequence` via
   `saturating_add(1)`, computes `parent_event_id` (ToolResult/Error/Stream
   → prior ToolCall event_id; everything else → `identity.parent_event_id`),
   constructs the envelope via `EventEnvelope::new` which calls
   `stable_event_id`, tracks in-flight tool calls in a `HashMap`, and emits.
   On stream end without terminal payload it synthesizes exactly one
   `AgentEvent::Error { source: "agent_stream", .. }` envelope.

3. **`event_id` determinism.** `stable_event_id`
   (`event_envelope.rs:64-84`) hashes schema_version,
   conversation_id, run_id, turn_id, execution_id, sequence with SHA-256.
   Identical `(identity, sequence)` pairs always produce identical event_id
   — verified by `resumes_after_persisted_sequence_with_stable_ids`
   (`event_envelope.rs:398-427`). `parent_event_id` is intentionally
   excluded so that replay under a different parent stays idempotent.

4. **Event consumption.** The same caller that owns the wrapper also drains
   it (`chat_driver.rs:538-565`, `executor.rs:3747-3763`,
   `a2a/server.rs:224-…`). No multi-subscriber fan-out: the
   `GLOBAL_EVENT_BUS` (`event_bus.rs:44-45`) advertised for that purpose is
   never read or written by any code in either repository (V02).

5. **Cancellation.** Two cooperating layers:
   - `cancel_aware_stream` (`mod.rs:896-917`) wraps any `AgentEvent` stream
     so a triggered `CancellationToken` yields `AgentEvent::Cancelled` and
     terminates.
   - Application-level loops additionally poll `cancel.is_cancelled()` to
     break early (`executor.rs:3748-3750`, `a2a/server.rs:225-227`).

6. **Errors.** `ReactError` (`error.rs:16-57`) is `#[non_exhaustive]`,
   Box-wrapped sub-enums, with explicit `From` impls for every typed
   sub-error and `#[from]` only for `std::io::Error` and
   `serde_json::Error`. The framework Result alias is
   `std::result::Result<T, ReactError>` (`error.rs:415`). Terminal agent
   errors are surfaced as `AgentEvent::Error { source, message }`
   (`mod.rs:273-279`) which carries typed source/tags but flattens the
   `ReactError` tree into two strings for the wire.

7. **Stable public surface.** Root re-exports at `echo-agent/src/lib.rs:139-144`
   bring `EventEnvelope`, `EventIdentity`, `CancellationToken`,
   `envelope_event_stream`, `envelope_event_stream_after`,
   `validate_event_trajectory`, `AGENT_EVENT_SCHEMA_VERSION` into
   `echo_agent::prelude`. `pub mod event_bus` at `echo-agent/src/lib.rs:39`
   exposes the (dead) bus as well.

## Findings

### F-CORE-01-P2-01: `GLOBAL_EVENT_BUS` and `EventBus` are dead infrastructure that advertises a multi-sink transport that does not exist

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/event_bus.rs:1-4` (doc comment promising
    "Webhook/Trace/UI/Audit" fan-out), `:11-14` (struct), `:16-34`
    (methods), `:36-40` (Default impl), `:42-45`
    (`GLOBAL_EVENT_BUS` static).
  - `echo-agent/src/lib.rs:39` (`pub mod event_bus`).
- Reachability: definition only. Zero callers of `GLOBAL_EVENT_BUS.send`,
  `GLOBAL_EVENT_BUS.subscribe`, `EventBus::new`, `EventBus::default`, or
  `.subscribe()` / `.subscriber_count()` on any `EventBus` instance in
  `echo-agent` or `echo-agent-cli` (outside `event_bus.rs` itself).
  Producers and consumers of `EventEnvelope` bypass the bus entirely (V02).
- Expected invariant: a public framework API advertised as the unified
  event distribution hub must either be wired into at least one
  producer/consumer path, or removed.
- Observed behavior: the bus is exported as a public symbol and given a
  global static with a doc that promises fan-out, but no code path feeds
  events into it or reads from it. Real event distribution uses direct
  stream composition.
- Impact: misleads API consumers (third-party `echo-agent` users,
  reviewers, new contributors) into believing a multi-sink observability
  transport exists. Under AGENTS.md "code cleanup: no compatibility
  burden", this is exactly the kind of dead path that should not be kept.
- Root cause: the bus was scaffolded as a future fan-out point but never
  connected. Production code chose direct stream composition instead, and
  no commit removed the scaffolding.
- Direction: either (a) delete `echo-agent/src/event_bus.rs`, the
  `pub mod event_bus` line in `echo-agent/src/lib.rs:39`, and any re-export
  of `EventBus` / `GLOBAL_EVENT_BUS` (preferred under the cleanup rule,
  because no concrete multi-sink consumer exists in either repo); or
  (b) wire the bus by having `envelope_event_stream_after` publish each
  envelope to `GLOBAL_EVENT_BUS` and at least one application subscriber
  (trace/UI) read from it. Choose one — keeping the unwired scaffolding is
  the worst option.
- Regression validation: after deletion, `cargo check --workspace` and
  `cargo check -p echo_agent --no-default-features` must both pass; no
  caller should be affected because there are none today. After wiring,
  add a test that one envelope appears on a `subscribe()` receiver.
- Validation reports: [V02](../validations/F-CORE-01/V02-01.md)

### F-CORE-01-P2-02: Cross-stream `event_id` collision is silently possible when concurrent envelope streams share identity fields

- Priority: P2
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/agent/event_envelope.rs:64-84`
    (`stable_event_id`).
  - `echo-agent/echo-core/src/agent/event_envelope.rs:91-96`
    (`from_invocation` fallback chain).
  - `echo-agent/echo-core/src/agent/event_envelope.rs:197-295`
    (`validate_event_trajectory` — intra-trajectory only).
- Reachability: every producer that calls `envelope_event_stream` /
  `envelope_event_stream_after` is reachable (V02 lists five live sites).
- Expected invariant: "stable, typed, sufficient for independent
  consumers" (task question) — implies `event_id` should be globally
  unique per concrete emitted event, not just unique within one stream.
- Observed behavior: `event_id` is SHA-256 over `(schema_version,
  conversation_id, run_id, turn_id, execution_id, sequence)`. Two
  concurrent streams that share all five identity fields will produce
  identical `event_id` for the same `sequence`. The fallback chain lets
  `turn_id` collapse onto `execution_id`, and `execution_id` is sometimes
  propagated without forking (e.g. subagent contexts that do not pass
  through `ensure_background_execution_id`). Today's audited callers avoid
  the collision by always setting a unique `turn_id` or `execution_id`,
  but the framework does not enforce or document the invariant.
- Impact: an independent consumer that joins envelopes from multiple
  streams by `event_id` (e.g. a future `GLOBAL_EVENT_BUS` subscriber, a
  persistence adapter, an audit log) could see two distinct events under
  the same id, breaking idempotent replay and per-event deduplication.
- Root cause: identity is caller-controlled and the framework performs no
  liveness check for concurrent streams sharing the same identity prefix.
- Direction: at minimum, document the invariant
  ("`turn_id` MUST be unique per concurrent envelope stream") in
  `event_envelope.rs` next to `stable_event_id`. Stronger: add a
  process-local `(identity, sequence)` registry that warns or errors when
  a second concurrent stream starts with an already-in-use identity
  prefix, or include a process-wide counter / random nonce in the hash
  when concurrency is detected.
- Regression validation: a test that wraps two streams with the same
  `EventIdentity` and asserts `event_id` divergence; or, if the chosen
  direction is documentation-only, a doc-test citing the invariant.
- Validation reports: [V03](../validations/F-CORE-01/V03-01.md)

### F-CORE-01-P3-01: Two parallel IO error channels and an untyped catch-all inside an otherwise typed error hierarchy

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/error.rs:52-53` — `ReactError::Io(#[from]
    std::io::Error)`.
  - `echo-agent/echo-core/src/error.rs:55-56` — `ReactError::Other(String)`.
  - `echo-agent/echo-core/src/error.rs:64-65` —
    `MemoryError::IoError(String)`.
  - `echo-agent/echo-core/src/error.rs:79-83` —
    `impl From<std::io::Error> for MemoryError` converts IO into
    `MemoryError::IoError(String)`, losing the original `std::io::Error`
    kind.
- Reachability: live. `ReactError::Io` is reached via `?` from any IO
  call. `ReactError::Other` is used as the default delegation target for
  trait methods that "do not support" an operation — e.g.
  `execute_stream_message_with_cancel` (`mod.rs:648-651`) and
  `delegate_to` (`mod.rs:756-760`).
- Expected invariant: the framework error hierarchy should encode each
  error category once, with structured variants, so consumers can match
  exhaustively.
- Observed behavior: IO errors surface through both `ReactError::Io
  (std::io::Error)` (typed kind preserved) and
  `MemoryError::IoError(String)` (kind lost). `Other(String)` is the
  catch-all used by the `Agent` trait's "not supported" defaults and many
  `From<…> for ReactError::Other` sites in the wider codebase.
- Impact: low. Consumers cannot pattern-match a single IO variant to
  cover all IO failures, and `Other(String)` defeats typed handling for
  the affected trait defaults. Not a correctness defect.
- Root cause: incremental accretion. `MemoryError::IoError` predates the
  top-level `ReactError::Io`, and the trait defaults predate the typed
  error variants.
- Direction: unify IO handling (either drop `MemoryError::IoError` in
  favor of the top-level IO variant, or convert at the boundary without
  dropping `io::ErrorKind`). Narrow the trait default errors from
  `ReactError::Other` to a typed `ReactError::Agent(AgentError::…)` (e.g.
  a new `UnsupportedOperation` variant) so consumers can match.
- Regression validation: `cargo test --workspace --all-features`;
  confirm no caller depends on `ReactError::Other("…not supported")`
  string matching.
- Validation reports: [V01](../validations/F-CORE-01/V01-01.md)

### F-CORE-01-P3-02: `StepType` is the only public agent type without documented non-serialization

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/agent/mod.rs:449-462` —
    `pub enum StepType { Thought(String), Call { tool_call_id, function_name,
    arguments } }` with only `#[derive(Debug)]`.
- Reachability: live. Returned by ReAct parsers and consumed internally;
  referenced in `AgentCallback::on_think_end` (`mod.rs:941-949`).
- Expected invariant: any public type's serialization contract should be
  either explicit (derive) or explicitly documented as "internal-only".
- Observed behavior: every other public type in the agent module that
  crosses a process/transport boundary carries `Serialize + Deserialize`.
  `StepType` is the exception, with no comment explaining why.
- Impact: low. A third-party implementor of `AgentCallback::on_think_end`
  cannot serialize the step list for logging without manual conversion.
- Root cause: documentation gap.
- Direction: add a one-line doc comment stating `StepType` is an
  internal ReAct parse result not meant for transport; or, if transport is
  useful, add the derive and a wire-format test.
- Regression validation: doc-only change; no test impact.
- Validation reports: [V01](../validations/F-CORE-01/V01-01.md)

### F-CORE-01-P3-03: `EventIdentity` derives `Default`, exposing empty `turn_id` to callers that should be forced to construct one

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/agent/event_envelope.rs:13` —
    `#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize,
    Deserialize)]` on `EventIdentity`.
  - `echo-agent-cli/src/cli/channels.rs:747-749` (test) and
    `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:1131-1132`
    (test) — both construct `EventIdentity { turn_id: "...", ..
    EventIdentity::default() }`.
- Reachability: live for production callers that might use `Default` to
  short-circuit identity construction.
- Expected invariant: every envelope stream must carry a non-empty
  `turn_id` to avoid `event_id` collisions (see F-CORE-01-P2-02).
- Observed behavior: `Default::default()` produces `turn_id = ""`, which
  `stable_event_id` will accept and hash. Two callers that both reach for
  `Default` would collide on sequence 1.
- Impact: low today (production callers never use `Default`); medium if
  future code copies the test pattern into production.
- Root cause: derive is broad; no validation on construction.
- Direction: drop `Default` from the derive (forcing explicit
  construction), or add a `EventIdentity::new(turn_id)` constructor that
  rejects empty `turn_id` and keep `Default` private. Compiles clean
  because the only `Default` uses are in test fixtures that can be
  updated in the same commit.
- Regression validation: `cargo test --workspace --all-features`; update
  the two test sites to use explicit construction.
- Validation reports: [V03](../validations/F-CORE-01/V03-01.md)

### F-CORE-01-P3-04: No explicit Serialize→Deserialize round-trip test for `EventEnvelope`

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/agent/event_envelope.rs:429-451` —
    `serializes_versioned_contract` only checks outgoing `to_value`
    fields.
- Reachability: live transport contract.
- Expected invariant: the wire contract should be tested in both
  directions for an envelope carrying each lifecycle group.
- Observed behavior: only the serialization direction is asserted.
  Deserialization correctness is implied by the derive but not exercised.
- Impact: low — a future field-type change could silently break
  deserialization without test failure.
- Root cause: test-coverage gap.
- Direction: extend the existing test to round-trip a JSON string back
  through `serde_json::from_str::<EventEnvelope>` and assert field
  equality, including a non-ASCII `FinalAnswer` payload and a
  tool-lifecycle sequence.
- Regression validation: the new test.
- Validation reports: [V04](../validations/F-CORE-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Type/variant inventory across agent module and error module | yes | passed | [V01-01](../validations/F-CORE-01/V01-01.md) |
| V02 | Producer-consumer reachability for `EventEnvelope` and `GLOBAL_EVENT_BUS` | yes | failed | [V02-01](../validations/F-CORE-01/V02-01.md) |
| V03 | `event_id` collision, sequence monotonicity, turn_id fallback | yes | passed (with documented edge cases) | [V03-01](../validations/F-CORE-01/V03-01.md) |
| V04 | Serialization shape and unit-test coverage for `event_envelope.rs` | yes | passed | [V04-01](../validations/F-CORE-01/V04-01.md) |
| V05 | Historical-document drift check | not-applicable | n/a | No historical document is reused for a claim in this report. The first Phase-F task in this reviewer directory; no prior F-CORE-01 report to drift against. |

## Historical Claim Status

No historical documents are cited as evidence for any claim in this
report. All findings are based on code at commit `9b0e0fa` /
`b3b2e81` and the four validation reports above.

## Coverage And Uncertainty

- Code not inspected: the concrete `ReactAgent` implementation of
  `chat_stream_with_cancel` and how it materializes the
  `AgentInvocationContext.runtime` into per-event identity. The framework's
  own `envelope_event_stream_after` is verified, but the upstream
  propagation from `ExternalRunContext` into the stream that
  `envelope_event_stream` wraps is taken on faith from the type signatures
  at `mod.rs:575-582` and `mod.rs:658-665`.
- Validations not executed at runtime: the six tests in
  `event_envelope.rs:297-514` were inspected statically, not run. A
  V04-02 (cargo test) would close the gap; not blocking because the
  coverage analysis is structural and the derives are simple.
- Environmental limits: none. Both repos are clean at the audited commits.
- Claims that remain uncertain:
  - Whether any third-party `echo-agent` consumer outside this monorepo
    subscribes to `GLOBAL_EVENT_BUS`. The framework layering rule in
    AGENTS.md says a pub API is retained unless framework-wide evidence
    shows it is obsolete; the deletion recommendation in F-CORE-01-P2-01
    is therefore conditional on a maintainer decision and the report
    offers a wire-up alternative.
  - Whether the cross-stream collision in F-CORE-01-P2-02 is triggered
    by any code path not yet audited (e.g. a future
    `task_runtime`-managed concurrent subagent batch). The audited
    callers all avoid it today; the finding is preventive.

## Handoff

- Conclusions downstream tasks may rely on:
  - `EventEnvelope` is the single, versioned, typed transport contract
    for `AgentEvent` payloads; downstream framework and application
    event-projection tasks should treat it as authoritative.
  - `EventIdentity` is the canonical identity record
    `(conversation_id, run_id, turn_id, execution_id, parent_event_id)`.
    Any new consumer should construct it explicitly, never via `Default`.
  - `ReactError` is the framework Result error type with a single
    definition site; `echo-agent/src/error.rs` is a pure re-export.
- Reports they must read:
  - [V01-01](../validations/F-CORE-01/V01-01.md) for the full type
    inventory.
  - [V02-01](../validations/F-CORE-01/V02-01.md) for the producer/
    consumer map (and the dead-bus finding).
- Conditions that make this report stale:
  - Any commit that wires a producer or consumer to `GLOBAL_EVENT_BUS`
    invalidates F-CORE-01-P2-01.
  - Any change to `stable_event_id`'s input set invalidates
    F-CORE-01-P2-02 and the V03 conclusions.
  - Any new `ReactError` variant or removal of `Other(String)` invalidates
    the error-semantics portion of V01 and F-CORE-01-P3-01.
- Follow-up task IDs (no fixes implemented in this review):
  - An application-layer task should audit whether the EKO chat/UI layer
    relies on `event_id` uniqueness across streams and exercise the
    collision scenario end-to-end (relates to F-CORE-01-P2-02).
  - A framework cleanup task should decide F-CORE-01-P2-01 (delete vs
    wire `GLOBAL_EVENT_BUS`).
