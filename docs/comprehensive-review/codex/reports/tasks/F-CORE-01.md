# F-CORE-01: Core identities, errors, and event envelope

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean at inspection time; review reports are outside both source repositories

## Question

Are run, turn, message, tool, and event identities plus error semantics stable,
typed, ordered, serializable, and sufficient for independent framework
consumers?

## Scope

- `echo-agent/echo-core/src/agent/mod.rs` and `event_envelope.rs`.
- `echo-agent/echo-core/src/error.rs`, LLM `Message`/`ToolCall`, and tool runtime
  identity/failure types.
- Root facade re-exports, `src/event_bus.rs`, live ReAct error producers,
  Subagent and A2A envelope adapters.
- EKO call sites only where needed to prove producer/consumer reachability and
  loss at the adapter boundary.
- Current JSON round-trip, legacy schema-v1 decoding, identity collision,
  ordering exhaustion, terminal normalization, and token accounting.

## Out Of Scope

- Fixing source code or selecting the complete migration sequence.
- Full public facade/doc review (`F-API-01`) and feature matrix
  (`F-FEAT-01`).
- Semantics of task, Subagent, trace, checkpoint, and frontend event systems
  beyond their contact with the core envelope (their respective atomic tasks).
- Proving durable delivery through `tokio::broadcast`; this review treats all
  broadcast buses as lossy observation surfaces, never recovery authority.
- Reopening external sources already bounded by `B-REF-01`.

## Inputs

- Root `AGENTS.md`.
- Shared `README.md`, `REPORTING.md`, and the `F-CORE-01` task card in
  `TASKS.md`.
- Codex reviewer protocol `codex/README.md`.
- Dependency report [B-ARCH-01](B-ARCH-01.md), limited to `echo_core` ownership
  and facade layering.
- Reference report [B-REF-01](B-REF-01.md), limited to stable hierarchical
  identity, typed lifecycle events, persisted history, and projection/authority
  boundaries.
- No report from another reviewer was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Typed run/turn/message/execution/tool/event identities, stable wire errors, envelope versioning, sequence/lineage validation, and an instance-scoped observation contract are reusable framework mechanisms owned by `echo_core` or its facade. |
| EKO product policy | Which message card renders an event, which events are persisted, and GUI/TUI/CLI presentation remain application policy. EKO may choose `message_id == turn_id`; the framework must not assume they are identical. |
| Adapter boundary | EKO adapters may inject product IDs and project canonical events, but conversion must preserve all identity and error facts. They must not infer timeout/cancellation by parsing strings or silently discard the envelope before a consumer boundary. |

Duplicate searches covered identity field names/newtypes, `Message`/`ToolCall`,
`ExternalRunContext`/`ToolContext`, all `EventIdentity` constructors, all
`EventEnvelope` wrappers/consumers, `GLOBAL_EVENT_BUS`, and the separate
Task/Subagent buses across both repositories. No new source type is proposed by
this review. The live specialized buses are not declared obsolete merely
because the global bus exists; authority must be selected before deleting an
implementation.

## Current Path

```text
ReactAgent phase
  -> Result<AgentEvent, ReactError>
  -> envelope_event_stream
       identity: ExternalRunContext -> EventIdentity
       ordering: last_sequence + 1 (saturating)
       tool parent: call_id -> ToolCall event_id
       raw error: ReactError -> AgentEvent::Error(source, Display text)
  -> EventEnvelope
       EKO chat: complete envelope -> ChatDriverSink
       Subagent: envelope.payload -> SubagentEventBus projection
       A2A: envelope.payload -> A2A projection
       EKO task runtime: envelope.payload -> ExecEvent projection
```

`EventIdentity::from_invocation` copies conversation, run, turn, and execution
from `ExternalRunContext` (`event_envelope.rs:86`), but not its message identity
(`tools/mod.rs:981`). `EventEnvelope::new` derives `event_id` solely from schema,
those four identity slots, and sequence (`event_envelope.rs:40`). Tool results,
errors, and progress are parented to the preceding `ToolCall` envelope by
`call_id` (`event_envelope.rs:147`). The wrapper normalizes stream failure or a
missing terminal into exactly one terminal Error and stops after the first
terminal (`event_envelope.rs:133`).

The wrapper is live in Subagent execution (`src/agent/subagent/executor.rs:1168`),
A2A (`src/a2a/server.rs:211`), EKO chat
(`echo-agent-app-core/src/chat_driver.rs:483`), and EKO task runtime
(`echo-agent-app-core/src/tasks/task_runtime/executor.rs:3115`). The public
`EventBus` is different: its global instance has no producer call anywhere in
either source repository. Task and Subagent operations continue to publish to
their own live buses.

The Rust error hierarchy is substantially typed (`echo-core/src/error.rs:18`),
and tool events preserve `ToolFailure` recovery and side-effect facts
(`echo-core/src/tools/mod.rs:17`). Non-tool stream errors lose that structure at
the envelope boundary and become two strings. This is observable when Subagent
execution reconstructs `ReactError::Other` (`src/agent/subagent/executor.rs:1403`)
before its typed status mapper (`src/agent/subagent/executor.rs:138`).

## Findings

### F-CORE-01-P2-01: Schema v1 no longer describes one stable ToolError wire form

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/agent/event_envelope.rs:9`,
  `echo-agent/echo-core/src/agent/mod.rs:211`, commit `dba349e`, commit `55d7a25`
- Reachability: framework streams emit serialized `EventEnvelope<AgentEvent>`;
  `ToolError` is a live event from `src/agent/react/run/phases/tools.rs:241`.
  A public-consumer fixture using the original schema-v1 ToolError shape fails
  current deserialization because `failure` is missing.
- Expected invariant: one schema version has one decodable wire contract, or
  changes supply defaults/migration logic.
- Observed behavior: schema remains `1`, while the later `failure: ToolFailure`
  field is required and has no serde default. Current-format JSON round-trips,
  but an older schema-v1 ToolError fails with `missing field failure`.
- Impact: the exported schema version cannot be used to identify or validate
  the payload contract, and a consumer cannot tell which v1 shape it received.
  The project explicitly has no backward-compatibility obligation, so this is
  not classified as a current data-migration failure without evidence of a live
  retained v1 store.
- Root cause: `AgentEvent` evolved independently of the envelope schema version;
  no compatibility fixture gates changes to nested payload variants.
- Direction: define what the exported version means before the next payload
  change. If only current-process transport is supported, remove the misleading
  persistence/version promise and old compatibility expectations. If durable
  event files are a supported framework contract, advance the version and add
  frozen fixtures/migration for every retained version.
- Regression validation: assert one selected policy: either current-only
  decoding with no durable/version compatibility claim, or frozen fixture
  decoding and explicit migration for every supported version.
- Validation reports: [V03-03](../validations/F-CORE-01/V03-03.md),
  [V04-05](../validations/F-CORE-01/V04-05.md)

### F-CORE-01-P1-02: The advertised unified EventBus has no framework producer

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/event_bus.rs:1`, `echo-agent/src/event_bus.rs:26`,
  `echo-agent/src/event_bus.rs:42`, `echo-agent/src/agent/subagent/events.rs:334`,
  `echo-agent/echo-orchestration/src/tasks/events.rs:150`
- Reachability: the facade publicly exposes `event_bus`, but repository-wide
  search finds zero calls to `GLOBAL_EVENT_BUS.send` or `EventBus::send`.
  Meanwhile live Subagent and task producers emit to separate broadcast buses.
- Expected invariant: an API documented as the common Webhook/Trace/UI/Audit
  stream is attached to the framework execution path.
- Observed behavior: subscribers can wait forever while agents run. Complete
  envelopes only reach consumers that directly wrap/intercept specific streams.
- Impact: an independent framework integration cannot observe execution through
  the advertised shared contract; each surface must rebuild mapping, defeating
  the stated purpose and allowing identities/error semantics to drift.
- Root cause: the envelope and global bus were introduced as APIs without one
  owning publication point; existing specialized buses remained authoritative.
- Direction: make enveloped streaming/observation an explicit instance-scoped
  Agent capability and publish once at the wrapper boundary. Keep broadcast
  documented as lossy observation only. Delete `GLOBAL_EVENT_BUS` and its stale
  examples unless a real framework lifecycle owner is established; do not make
  it recovery authority or add a second projection loop.
- Regression validation: attach two independent observers to a real Agent run
  and assert identical ordered envelopes for success, typed failure,
  cancellation, and tool lifecycle; also assert lag is reported rather than
  presented as durable completeness.
- Validation reports: [V02-01](../validations/F-CORE-01/V02-01.md),
  [V02-02](../validations/F-CORE-01/V02-02.md)

### F-CORE-01-P1-03: Non-tool typed errors collapse to text and change terminal semantics

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/error.rs:18`,
  `echo-agent/echo-core/src/agent/mod.rs:272`,
  `echo-agent/echo-core/src/agent/event_envelope.rs:133`,
  `echo-agent/src/agent/react/run/phases/finalize.rs:226`,
  `echo-agent/src/agent/subagent/executor.rs:138`,
  `echo-agent/src/agent/subagent/executor.rs:1403`
- Reachability: no-response/max-iteration and other ReAct paths send typed
  `ReactError`; the live envelope adapter converts it to Error strings; the
  live Subagent consumer reconstructs `ReactError::Other`, then its typed mapper
  can no longer identify timeout/cancellation.
- Expected invariant: a consumer can distinguish retryable provider failures,
  cancellation, timeout, permission, invalid response, and permanent failure
  without parsing display text.
- Observed behavior: `LlmError::ApiError { status: 429 }` becomes source
  `agent_stream` plus formatted text. A Subagent timeout carried through this
  path becomes generic `Failed`; provider status/retryability and nested error
  kind are unrecoverable. Tool errors are the positive typed exception.
- Impact: terminal status, retry policy, telemetry, and UI recovery actions can
  be wrong after the serialization boundary even though the producer had the
  exact type.
- Root cause: `AgentEvent::Error` was designed as display data rather than a
  stable transport failure contract.
- Direction: add a serializable non-tool failure classification patterned after
  `ToolFailure` (stable code/category, terminal kind, retry facts, source, and
  human message). Convert once from `ReactError`; preserve the structured value
  through Subagent/EKO adapters. Delete string-to-`ReactError::Other`
  reconstruction and any status inference from messages.
- Regression validation: table-drive every public `ReactError` variant through
  envelope JSON and Subagent/application adapters; assert terminal class and
  retry facts survive, including LLM 429, timeout, cancellation, permission,
  parse, and I/O cases.
- Validation reports: [V02-01](../validations/F-CORE-01/V02-01.md),
  [V03-02](../validations/F-CORE-01/V03-02.md),
  [V04-04](../validations/F-CORE-01/V04-04.md),
  [V04-05](../validations/F-CORE-01/V04-05.md)

### F-CORE-01-P2-04: Identity is lossful, interchangeable, and permits an empty mandatory turn

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/llm/types.rs:238`,
  `echo-agent/echo-core/src/llm/types.rs:450`,
  `echo-agent/echo-core/src/tools/mod.rs:963`,
  `echo-agent/echo-core/src/tools/mod.rs:1000`,
  `echo-agent/echo-core/src/agent/event_envelope.rs:12`,
  `echo-agent/echo-core/src/agent/event_envelope.rs:86`
- Reachability: EKO chat supplies a real `ExternalRunContext.message_id` at
  `echo-agent-app-core/src/chat_driver.rs:490`; live wrappers derive
  `EventIdentity::from_invocation`. Public callers can also construct or default
  the identity directly.
- Expected invariant: distinct identity domains cannot be swapped at compile
  time, all invocation correlation facts cross the envelope boundary, and a
  required turn ID is non-empty.
- Observed behavior: all IDs are `String`; `Message` has no identity field;
  `message_id` is dropped by `from_invocation`; `EventEnvelope` has no message
  slot; `EventIdentity::default()` accepts `turn_id == ""`. Two otherwise equal
  invocations differing only by message ID produce identical event identity and
  IDs.
- Impact: an independent consumer cannot reliably attach a delegated/tool event
  to the triggering message when message and turn are not the same product key.
  Accidental run/turn/execution swaps compile, and empty/default identity can
  merge unrelated producer streams.
- Root cause: identity accumulated as optional metadata on several structs
  instead of one validated typed hierarchy; the envelope copied only the fields
  needed by its first adapters.
- Direction: introduce validated core ID newtypes and a lossless invocation
  identity containing optional `message_id`; provide explicit constructors for
  chat/run/subexecution cases and remove public `Default` for a mandatory turn.
  Keep provider wire tool-call strings converted at the LLM adapter boundary.
  Delete parallel raw-field construction once callers migrate.
- Regression validation: compile-fail swapped-ID probes, reject blank IDs,
  round-trip every identity field, and run two equal turns with distinct message
  IDs through chat and Subagent projection without collision.
- Validation reports: [V01](../validations/F-CORE-01/V01-01.md),
  [V03-01](../validations/F-CORE-01/V03-01.md),
  [V04-04](../validations/F-CORE-01/V04-04.md),
  [V04-05](../validations/F-CORE-01/V04-05.md)

### F-CORE-01-P2-05: Event slot collisions and saturated ordering are not rejected at construction

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/agent/event_envelope.rs:40`,
  `echo-agent/echo-core/src/agent/event_envelope.rs:64`,
  `echo-agent/echo-core/src/agent/event_envelope.rs:123`,
  `echo-agent/echo-core/src/agent/event_envelope.rs:197`
- Reachability: `EventEnvelope::new` and `envelope_event_stream_after` are public;
  chat setup errors also call `new` directly. Resume is advertised as a public
  persistence adapter even though the current repositories do not call the
  `_after` form outside tests.
- Expected invariant: one event ID cannot silently name two divergent event
  facts, sequence exhaustion fails closed, and deserialized IDs can be verified
  from their identity/sequence.
- Observed behavior: payload, parent, and timestamp are not covered by the ID.
  Same identity/sequence with different payload and parent produces one ID.
  Resuming after `u64::MAX` emits repeated maximum sequences and IDs; the
  validator's own saturated expected sequence does not flag ordering, though
  duplicate-ID detection catches later events. The validator never recomputes
  event IDs and is unused outside its unit tests.
- Impact: a persistence adapter deduplicating on the advertised deterministic
  ID can silently retain the wrong divergent event. Corrupt/exhausted resume
  input yields ambiguous ordering instead of a typed failure.
- Root cause: one hash is serving as a logical slot/idempotency key without a
  separate content-integrity contract, while public sequence arithmetic uses
  saturation as error handling.
- Direction: retain a deterministic logical slot only if divergent content is
  separately hashed/compared. Validate non-empty identity and recomputed IDs at
  ingestion; replace saturation with checked sequence allocation that returns a
  typed terminal error. Make recovery adapters invoke validation before accepting
  a trajectory; delete the unused public validator if no recovery owner adopts it.
- Regression validation: divergent same-slot replay must fail, tampered event ID
  must fail, `u64::MAX - 1` transitions once then rejects exhaustion, and valid
  resumed tool-parent trajectories remain accepted.
- Validation reports: [V03-01](../validations/F-CORE-01/V03-01.md),
  [V04-03](../validations/F-CORE-01/V04-03.md),
  [V04-04](../validations/F-CORE-01/V04-04.md),
  [V04-05](../validations/F-CORE-01/V04-05.md)

### F-CORE-01-P2-06: Public token accounting panics or wraps on provider-sized input

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/agent/mod.rs:338`,
  `echo-agent/echo-core/src/agent/mod.rs:360`
- Reachability: `AgentEvent::ThinkEnd` is public/serializable and its public
  `total_tokens`/`tokens_used` helpers accept deserialized/provider-derived
  `usize` counts. Current repositories do not call these helpers outside their
  implementation, so the immediate risk is to independent consumers.
- Expected invariant: public accounting helpers never panic and never return an
  silently wrapped total for valid field values.
- Observed behavior: `prompt_tokens + completion_tokens` is unchecked. A public
  probe with `usize::MAX` and `1` panics in debug mode at line 366; optimized
  arithmetic may wrap.
- Impact: malformed or extreme provider data can terminate a debug consumer or
  corrupt usage/budget accounting in release mode.
- Root cause: count fields are individually valid but their derived aggregate
  does not use checked or saturating arithmetic.
- Direction: use `checked_add` and expose overflow as a typed result, or use
  saturating addition if the documented semantic is a conservative upper bound.
  Remove the unchecked compatibility helper path.
- Regression validation: cover zero, normal, maximum-boundary, and overflow
  cases in debug and release-compatible semantics without panic.
- Validation reports: [V04-05](../validations/F-CORE-01/V04-05.md)

### F-CORE-01-P3-07: Public EventBus constructor panics on an accepted capacity

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/event_bus.rs:16`,
  `echo-agent/src/event_bus.rs:17`
- Reachability: `EventBus::new` is public through `pub mod event_bus`; an
  independent framework consumer can pass any `usize`. The current global bus
  uses 1024 and does not trigger the defect.
- Expected invariant: a public configuration constructor either validates zero
  capacity as a typed error or normalizes it; it must not delegate invalid input
  to a panic API.
- Observed behavior: `EventBus::new(0)` calls `broadcast::channel(0)` and panics
  with `broadcast channel capacity cannot be zero`.
- Impact: external configuration or a computed capacity can terminate the
  process, violating the repository's explicit no-panic contract. Immediate
  internal blast radius is limited because current code only constructs 1024.
- Root cause: the facade exposes the dependency constructor's unchecked
  precondition without a validated capacity type or `Result`.
- Direction: make the constructor return a typed error for zero or accept a
  `NonZeroUsize`; keep `Default` as the infallible common path. Delete the
  unchecked public signature after callers migrate.
- Regression validation: external probe covers zero, one, and default capacity;
  zero returns a stable typed error and never unwinds.
- Validation reports: [V04-06](../validations/F-CORE-01/V04-06.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Identity/type inventory and duplicate search | yes | failed | [V01](../validations/F-CORE-01/V01-01.md) |
| V02 | Envelope producer-consumer reachability | yes | passed | [V02-01](../validations/F-CORE-01/V02-01.md) |
| V02 | Unified EventBus registration/reachability | yes | failed | [V02-02](../validations/F-CORE-01/V02-02.md) |
| V03 | Collision, ordering, validation, and edge-case inspection | yes | failed | [V03-01](../validations/F-CORE-01/V03-01.md) |
| V03 | Typed error inventory and boundary preservation | yes | failed | [V03-02](../validations/F-CORE-01/V03-02.md) |
| V03 | Schema evolution history | yes | failed | [V03-03](../validations/F-CORE-01/V03-03.md) |
| V04 | Repository event-envelope tests | yes | passed | [V04-01](../validations/F-CORE-01/V04-01.md) |
| V04 | External probe, online attempt | yes | inconclusive | [V04-02](../validations/F-CORE-01/V04-02.md) |
| V04 | External probe, first offline expectation | yes | failed | [V04-03](../validations/F-CORE-01/V04-03.md) |
| V04 | Corrected external identity/error/collision probe | yes | passed | [V04-04](../validations/F-CORE-01/V04-04.md) |
| V04 | Expanded schema/token/identity/error probe | yes | passed | [V04-05](../validations/F-CORE-01/V04-05.md) |
| V04 | Zero-capacity EventBus panic probe | yes | passed | [V04-06](../validations/F-CORE-01/V04-06.md) |
| V05 | Dependency/historical conclusion drift | yes | passed | [V05](../validations/F-CORE-01/V05-01.md) |
| V06 | Combined root/subrepository report gate | yes | inconclusive | [V06-01](../validations/F-CORE-01/V06-01.md) |
| V06 | Corrected source-isolation/link/executor gate | yes | passed | [V06-02](../validations/F-CORE-01/V06-02.md) |
| V06 | Final post-probe report gate | yes | passed | [V06-03](../validations/F-CORE-01/V06-03.md) |
| V07 | Primary schema-policy isolation/calibration | yes | passed after isolation failure | [V07-01](../validations/F-CORE-01/V07-01.md), [V07-02](../validations/F-CORE-01/V07-02.md) |
| V08 | Primary source/probe/finding acceptance | yes | passed | [V08](../validations/F-CORE-01/V08-01.md) |
| V09 | Primary rerun of event-envelope unit tests | yes | passed | [V09](../validations/F-CORE-01/V09-01.md) |
| V10 | Final report links/executor/source-isolation gate | yes | passed | [V10](../validations/F-CORE-01/V10-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| B-ARCH-01: core contracts belong to reusable `echo_core`, surfaced through the root facade | current | Core owns `AgentEvent`, `EventEnvelope`, identity, errors, and tools context; root re-exports them. |
| B-REF-01: stable hierarchical identity and typed start/update/terminal events are cross-system constraints | current constraint, partially unmet | Findings P1-03, P2-04, and P2-05 plus V05. |
| Commit `dba349e`: schema v1 is a versioned framework transport contract | regressed/ambiguous | Later required ToolError field did not advance the schema; P2-01. The repository does not otherwise require backward compatibility. |
| Commit `93fac5b`: trajectory validation supports replay/resume contracts | current implementation, unreachable outside tests | V03-01 finds no production validator/resume caller. |
| Commit `55d7a25`: tool failures retain structured recovery facts | current | `ToolFailure` survives `AgentEvent::ToolError`; V03-02. |
| `src/event_bus.rs`: the unified bus replaces scattered frontend mappings | stale/unrealized | V02-02 finds no producer and multiple live specialized buses. |

## Coverage And Uncertainty

- The targeted `echo_core` event tests passed; a full workspace gate was not run
  because this is a read-only atomic review with no source changes.
- Current-format round-trip was executed for `FinalAnswer`; existing repository
  tests cover additional event serialization, but this task did not exhaustively
  deserialize every `AgentEvent` variant. The recommended fixture matrix remains
  necessary.
- Sequence exhaustion is practically rare, but the public resume API accepts the
  value and its advertised job is recovery correctness; the finding is therefore
  retained at P2 rather than dismissed.
- `GLOBAL_EVENT_BUS` could be used only by an external caller that manually sends
  events. No current internal producer makes it a framework observation surface.
- The first probe attempt could not resolve `rsproxy.cn`; offline attempts used
  the cached dependency set successfully.
- The temporary probe is outside the repositories and may be removed by normal
  system cleanup. All reproducible source and assertions are summarized in its
  immutable validation reports.

## Handoff

- `F-API-01` should consume P1-02 and validate/remove the stale `BusEvent` and
  `send_for_run` examples without duplicating the core reachability review.
- `F-RCT-*`, `F-SUB-*`, `F-TSK-*`, and `X-EVT-01` may rely on P1-03 and P2-04:
  payload-only adapters are lossy for identity/error semantics.
- State/recovery tasks must read P2-01 and P2-05 before treating schema v1,
  `event_id`, or `validate_event_trajectory` as durable authority.
- Preserve the positive `ToolFailure` contract; extend its pattern to non-tool
  terminal failures rather than replacing it with strings.
- This report becomes stale when `AgentEvent`, `ReactError`, `EventIdentity`,
  `EventEnvelope`, sequence/hash logic, the global bus, or the inspected live
  wrapper call sites change.
