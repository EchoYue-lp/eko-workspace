# F-RCT-03: Streaming ReAct event flow

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories had no tracked/staged diff; concurrent untracked CLI generated files were not read or modified; this task changed only Codex reports

## Question

Are streaming deltas and terminal events ordered, lossless, bounded, and
semantically equivalent to non-streaming execution?

## Scope

- Public Agent streaming entries and live `run_stream_channel` reachability.
- Guard/DirectAnswer pre-core branches, spawned canonical core, raw LLM chunk
  assembly, primary mpsc channel, event macros, terminal/error behavior.
- Content/reasoning tokens, ThinkStart/ThinkEnd/LlmUsage, tool-call delta
  assembly, steer/verifier visibility, cancel/disconnect, trace alignment.
- Envelope and EKO/A2A/Subagent consumers only to prove live reachability and
  concrete protocol impact.
- Static duplicate, history, test-coverage, panic, UTF-8, and overflow checks.

## Out Of Scope

- Shared non-stream/core terminal ownership: F-RCT-02-P1-01 through P1-03.
- EventEnvelope identity, sequence saturation, typed-error loss, and global
  EventBus: F-CORE-01.
- Tool execution concurrency, timeout, partial side effects, and pairing beyond
  delta-to-ToolCall production: `F-RCT-04`.
- Resume correctness after interruption: `F-RCT-05`.
- Provider-specific SSE decoding/cancellation: provider tasks.
- Application reducer correctness beyond reachability: `A-CHAT-01` and surface
  tasks.
- Cargo, rustc, tests, builds, and dynamic fixtures, prohibited for this task.

## Inputs

- Root `AGENTS.md` and shared review `README.md`, `REPORTING.md`, `TASKS.md`.
- `docs/comprehensive-review/codex/README.md`.
- F-RCT-02 (`needs_evidence`): canonical loop and terminal ownership findings.
- F-CORE-01 (`complete`): envelope/event identity, typed error, EventBus, token
  overflow, and sequence findings.
- Current source/history only; no other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Stream production, ordered/lossless lifecycle delivery, backpressure, disconnect cancellation, delta semantics, and terminal/trace agreement are framework contracts. |
| EKO product policy | Throttling/rendering and whether provisional reasoning is shown are product policy, but framework event types must distinguish provisional reasoning/draft from accepted response. |
| Adapter boundary | Consumers may project canonical events but must not repair missing lifecycle events, retract untyped drafts, or infer success from EOF. Disconnect adapters should signal the invocation cancellation authority. |
| Duplicate search | Public stream traits, entry helpers, channel producer, DirectAnswer bypass, raw chunk processor, macros, envelope wrapper, and consumer loops were searched. One main entry exists; DirectAnswer is a separate live production lifecycle. |
| Migration deletion | Retain `run_stream_channel` plus one spawned producer. Delete the synchronous DirectAnswer producer lifecycle and drop-on-full lifecycle macros after all paths use the canonical transport/outcome contract. |

## Current Path

```text
public execute/chat stream APIs
  -> run_stream_channel
     -> bounded mpsc(256), acquire execution lease, start trace/context
     -> guard block: enqueue FinalAnswer, return receiver
     -> DirectAnswer: await entire producer before returning receiver
     -> normal: spawn run_core_loop, return ReceiverStream
        -> run_think
           -> provider stream -> process_stream_chunk
           -> try_send reasoning/phase events
           -> buffer ordinary content/tool deltas
           -> LlmUsage + buffered Token
        -> steer -> verifier -> tool/finalizer
        -> terminal event
  -> optional envelope_event_stream
  -> chat/Subagent/task/A2A consumers
```

The normal stream is live through all public Agent stream methods. EKO chat,
TaskRuntime, framework Subagent, and A2A wrap/consume it. The envelope wrapper
normalizes raw error or missing terminal and stops after the first terminal;
that improves consumer cardinality but cannot repair producer trace/persistence
or events already dropped before the wrapper.

Tool-call delta assembly is deterministic by provider index and UTF-8 safe.
Finite usage accounting uses bounded conversions in the inspected producer.
No separate legacy stream module was found, but DirectAnswer remains a second
producer with different concurrency, error, lifecycle, and token semantics.

## Findings

### F-RCT-03-P1-01: DirectAnswer can deadlock before returning its stream

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/stream_channel.rs:35-41`,
  `:185-245`, `:362-415`, `:465-479`
- Reachability: public streaming entry -> IntentRouter DirectAnswer with shortcut
  allowed -> `direct_answer_stream` awaited inline -> bounded sends; only after
  completion does `run_stream_channel` return the receiver.
- Expected invariant: the receiver is returned before a bounded producer can
  block; streaming events are observable while the model is producing.
- Observed behavior: up to 256 events are buffered invisibly. A response with
  more distinct deltas blocks on the next `send().await`, while the caller cannot
  poll the receiver it has not received. Execution mutex and turn lease remain
  held. Short answers are delivered only after full model completion.
- Impact: long/fine-grained DirectAnswer responses can hang the agent and block
  subsequent turns; ordinary DirectAnswer has no streaming latency benefit.
- Root cause: this bypass runs producer and setup in the same awaited future,
  unlike the spawned canonical producer.
- Direction: return the receiver immediately and spawn DirectAnswer under the
  same execution/terminal owner as the canonical loop, or remove the shortcut's
  separate stream lifecycle. Delete the inline producer branch.
- Regression validation: emit `capacity + 1` deltas, assert the first event is
  observable before provider completion, the terminal arrives once, and a
  second turn is not lease-blocked.
- Validation reports: [V02-01](../validations/F-RCT-03/V02-01.md),
  [V06](../validations/F-RCT-03/V06-01.md)

### F-RCT-03-P1-02: The primary channel silently drops lifecycle and tool events under load

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/stream_macros.rs:10-74`,
  `run/phases/think.rs:99-245`, `run/phases/tools.rs:63-430`
- Reachability: all normal streaming runs emit phase, usage, tool, memory,
  compression, budget, and most error events through `yield_event_or!`; slow
  consumers and high-rate ToolStream producers can fill the fixed buffer.
- Expected invariant: primary lifecycle/tool events are ordered and lossless or
  stream failure is explicit. Backpressure cannot silently erase call/result or
  phase boundaries.
- Observed behavior: `try_send` drops any non-terminal event when Full and only
  logs a warning. A ToolCall may be dropped while its result survives, or vice
  versa; batch/Think boundaries, usage, errors, and visible content may vanish.
  The warning hardcodes 256 rather than reporting actual capacity.
- Impact: consumers can display orphan/running tools, miss errors or content,
  calculate wrong usage, and persist an unreplayable event trajectory even when
  the underlying execution completes.
- Root cause: one lossy queue is used as the authoritative invocation stream
  without delivery-class semantics or gap signaling.
- Direction: use ordered awaited backpressure for canonical lifecycle/tool
  events. If high-volume progress needs coalescing, give it a distinct lossy
  type with explicit gap/coalescing facts. Delete silent Full-drop behavior.
- Regression validation: a deliberately slow consumer plus more than capacity
  mixed Token/tool/phase events; assert complete ordered pairing and no silent
  loss, including disconnect.
- Validation reports: [V02-02](../validations/F-RCT-03/V02-02.md),
  [V06](../validations/F-RCT-03/V06-01.md)

### F-RCT-03-P1-03: Canonical streaming exposes unaccepted drafts and is not incremental

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/processor.rs:16-87`,
  `run/phases/think.rs:110-125`, `:224-247`,
  `run/stream_channel.rs:657-676`, `run/phases/verify.rs:107-131`,
  `echo-agent-cli/src/tui/events.rs:602-606`
- Reachability: every normal non-tool response passes this path; steer and
  configured verifier decisions occur after `run_think` emits the buffered Token.
- Expected invariant: content streams incrementally with explicit provisional
  semantics, and rejected/superseded drafts are retractable or not published as
  accepted assistant output.
- Observed behavior: `emit_content_tokens=false` buffers all normal content and
  emits one full Token only after provider EOF. The driver then checks steer and
  verifier. A rejected/superseded draft has already been appended by consumers,
  with no retract/replace identity. Reasoning is emitted live using the same
  Token variant, mixing reasoning and answer semantics.
- Impact: users see stale draft plus corrected answer; consumers cannot know
  what text belongs to reasoning, provisional response, or accepted response.
  First-token latency equals full model latency for ordinary answers.
- Root cause: raw transport deltas, provisional UI drafts, reasoning, and
  accepted semantic output share one Token type while acceptance is downstream.
- Direction: define distinct reasoning/provisional/accepted events with draft
  identity and replacement semantics, or delay answer publication until
  acceptance. If provisional streaming is supported, emit content deltas as
  received and provide explicit commit/retract.
- Regression validation: steer and verifier reject after multiple deltas; assert
  consumers end with only the accepted answer and first-token latency remains
  incremental under the chosen protocol.
- Validation reports: [V03-01](../validations/F-RCT-03/V03-01.md),
  [V06](../validations/F-RCT-03/V06-01.md)

### F-RCT-03-P1-04: Consumer disconnect does not promptly cancel upstream work

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:540-546`,
  `echo-agent/src/agent/react/run/phases/think.rs:110-125`, `:224-247`,
  `run/stream_macros.rs:38-74`
- Reachability: a live chat sink can return false; the envelope/raw receiver is
  dropped; producer is an independent spawned core holding the execution lease.
- Expected invariant: consumer termination propagates cancellation immediately,
  stops provider/tools, records a typed terminal, and releases the lease after
  safe-point persistence.
- Observed behavior: consumers break/drop without cancelling the invocation.
  Producer detects closure only at a later send. A non-reasoning response sends
  no content event until provider EOF, so it may finish the full model request
  first. Closed-channel macros abandon with `Ok`, inheriting F-RCT-02's missing
  terminal projections.
- Impact: invisible LLM/tool work continues after navigation/disconnect, spends
  resources, holds the agent lease, and can mutate context after its consumer is
  gone; recovery state is ambiguous.
- Root cause: stream ownership/drop is not coupled to the invocation cancellation
  authority, and the producer only polls channel liveness opportunistically.
- Direction: make the returned stream own a drop guard that requests cancellation,
  or require adapters to cancel before drop; make the producer select on cancel
  during LLM and tools and route cancellation through the common terminal commit.
- Regression validation: drop before first token, mid-content, and mid-tool;
  assert provider/tool cancellation, one durable cancelled terminal, and prompt
  release of the next turn.
- Validation reports: [V02-03](../validations/F-RCT-03/V02-03.md)

### F-RCT-03-P1-05: Partial DirectAnswer failure is recorded as both error and success

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/stream_channel.rs:385-395`,
  `:417-479`, `:223-242`,
  `echo-agent/echo-core/src/agent/event_envelope.rs:133-177`
- Reachability: provider yields one or more successful DirectAnswer chunks then
  an error; the live shortcut catches it inside `direct_answer_stream`.
- Expected invariant: one provider failure produces one failed terminal and a
  failed trace; partial content may be diagnostic but cannot be final success.
- Observed behavior: the producer sends terminal Error but returns `Ok(content)`,
  continues with LlmUsage and FinalAnswer, and its caller finalizes trace
  Completed. Raw consumers can see two terminals; envelope consumers stop at
  Error and hide the later success while trace storage says Completed.
- Impact: stream, trace, context, webhook, and callers disagree whether the turn
  failed. Partial content may be persisted as successful assistant output.
- Root cause: partial content and provider error are collapsed into a successful
  string return, and DirectAnswer has no typed terminal outcome.
- Direction: return the typed error plus optional partial draft and use the
  common terminal commit. Delete post-error usage/final success emission and
  branch-local trace completion.
- Regression validation: partial chunks then provider error through raw and
  envelope consumers; assert one Error, failed trace, no FinalAnswer, and an
  explicit policy for partial draft retention.
- Validation reports: [V03-02](../validations/F-RCT-03/V03-02.md),
  [V06](../validations/F-RCT-03/V06-01.md)

### F-RCT-03-P2-06: ThinkEnd cardinality and usage depend on provider chunk shape

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/processor.rs:28-63`,
  `run/phases/think.rs:127-245`, `echo-agent/echo-core/src/agent/mod.rs:329-375`
- Reachability: reasoning-capable provider streams drive `in_reasoning`; EKO/TUI
  and telemetry consume ThinkEnd/LlmUsage.
- Expected invariant: one logical reasoning phase has one start/end pair with a
  stable usage contract independent of whether usage arrives in a later chunk.
- Observed behavior: content/tool transition emits ThinkEnd with zero usage; EOF
  while reasoning emits real usage. Content combined with tool calls can emit a
  synthetic second ThinkStart/ThinkEnd around buffered content. LlmUsage already
  carries stable full-call accounting.
- Impact: phase UI/counts and token telemetry vary across equivalent provider
  responses solely due to chunk boundaries; double counting is easy because
  ThinkEnd and LlmUsage both expose tokens.
- Root cause: one event mixes structural phase closure with usage facts that are
  unavailable at transition time, followed by a second synthetic phase.
- Direction: make ThinkEnd purely structural and use LlmUsage as the sole token
  authority, or emit exactly one delayed ThinkEnd after usage is final. Delete
  zero-placeholder/synthetic duplicate phase emission.
- Regression validation: equivalent reasoning/content/tool responses split into
  multiple chunk layouts; assert identical phase cardinality and usage totals.
- Validation reports: [V03-03](../validations/F-RCT-03/V03-03.md),
  [V06](../validations/F-RCT-03/V06-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, duplicate, and public reachability | yes | passed with deviation | [V01](../validations/F-RCT-03/V01-01.md) |
| V02 | DirectAnswer producer/receiver ordering | yes | failed invariant | [V02-01](../validations/F-RCT-03/V02-01.md) |
| V02 | Primary channel backpressure/loss | yes | failed invariant | [V02-02](../validations/F-RCT-03/V02-02.md) |
| V02 | Consumer disconnect propagation | yes | failed invariant | [V02-03](../validations/F-RCT-03/V02-03.md) |
| V03 | Delta/draft/steer/verifier semantics | yes | failed invariant | [V03-01](../validations/F-RCT-03/V03-01.md) |
| V03 | Partial stream error and terminal trace | yes | failed invariant | [V03-02](../validations/F-RCT-03/V03-02.md) |
| V03 | ThinkEnd/usage cardinality | yes | failed invariant | [V03-03](../validations/F-RCT-03/V03-03.md) |
| V03 | Tool-call delta assembly and UTF-8 | yes | passed | [V03-04](../validations/F-RCT-03/V03-04.md) |
| V03 | Panic/UTF-8/overflow bounded scan | yes | passed | [V03-05](../validations/F-RCT-03/V03-05.md) |
| V04 | Dynamic stream conformance | future | not run by instruction | [V04](../validations/F-RCT-03/V04-01.md) |
| V05 | History/dependency de-dup | yes | passed | [V05](../validations/F-RCT-03/V05-01.md) |
| V06 | Existing test coverage inventory | yes | passed static inventory | [V06](../validations/F-RCT-03/V06-01.md) |
| V07 | Report/source integrity gate | yes | passed | [V07](../validations/F-RCT-03/V07-01.md) |
| V30 | Primary source-anchor acceptance | yes | passed | [V30](../validations/F-RCT-03/V30-01.md) |
| V31 | Primary acceptance integrity and source isolation | yes | passed | [V31](../validations/F-RCT-03/V31-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Stream module comment: all streaming uses one module | current at entry level | [V01](../validations/F-RCT-03/V01-01.md) |
| Stream module comment: DirectAnswer streams tokens as they arrive | regressed/false at public boundary | [V02-01](../validations/F-RCT-03/V02-01.md) |
| Stream macro comment: channel macros abandon gracefully | stale as a correctness claim | Full silently drops; Closed lacks durable terminal, [V02-02](../validations/F-RCT-03/V02-02.md), [V02-03](../validations/F-RCT-03/V02-03.md) |
| F-RCT-02 premature/missing terminal and fragmented persistence | current dependency | F-RCT-02-P1-01 through P1-03 |
| F-CORE-01 envelope normalization and typed-error loss | current dependency, not duplicated | F-CORE-01-P1-03 and P2-05 |

## Coverage And Uncertainty

- No dynamic command was run. Timing/deadlock assertions follow directly from
  bounded-channel ownership but require regression execution during fixes.
- Future cases: 257+ DirectAnswer deltas; slow consumer with mixed lifecycle and
  ToolStream events; drop before first token/mid-content/mid-tool; partial chunk
  then error; reasoning/content/tool/usage chunk permutations; steer/verifier
  rejection after visible draft; Stop continuation; malformed/interleaved tool
  deltas; huge multilingual content.
- Stream buffer size is private/default 256; dynamic capacity configuration was
  not claimed. Full remains reachable through consumer/progress rate mismatch.
- Provider adapter cancellation and exact SSE error mapping were not reopened.
- F-RCT-02 owns terminal persistence and Stop ordering; F-CORE-01 owns envelope
  identity/sequence/error structure. Synthesis must merge backlinks, not IDs.
- Primary must independently sample producer ordering and macro call sites;
  status remains `needs_evidence`.

## Handoff

- `F-RCT-04` must assume ToolCall/Result events can currently be lost before its
  pairing checks; keep channel loss owned by P1-02.
- `F-RCT-05` must treat consumer drop as non-durable abandonment until P1-04 and
  F-RCT-02 terminal ownership are fixed.
- `A-CHAT-01` and surfaces should not invent draft retraction or terminal repair;
  framework must first define the canonical protocol.
- Remediation order: eliminate DirectAnswer pre-return deadlock/contradictory
  terminal, make lifecycle delivery lossless, couple disconnect to cancellation,
  then define provisional/accepted token and stable phase/usage semantics.
- Primary review independently sampled the bounded-channel ownership,
  DirectAnswer producer/error path, lossy channel macros, buffered normal
  content, consumer drop, and thinking event construction. The six findings
  and priorities were accepted; see V30.
- This report becomes stale when stream entry spawning, channel macros/buffer,
  DirectAnswer, `process_stream_chunk`, `run_think`, AgentEvent phase/token types,
  or envelope terminal normalization changes.
