# F-RCT-03: Streaming ReAct event flow

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Are streaming deltas and terminal events ordered, lossless, bounded, and
semantically equivalent to non-streaming execution?

Answer: ordered and bounded yes; **lossless no** (bounded-channel drops under
backpressure, including terminal errors — P1-01); **terminal semantics no**
(cancel never yields the documented `Cancelled` terminal — P1-02; a second
`FinalAnswer` is possible on the raw stream via Stop-hook continuation, masked
by the envelope adapter — P2-02); **streaming deltas no** (the main loop
buffers content and emits one burst — P2-01); conformance with non-streaming
is partial (shared loop body, but divergent wrapper/hook/pre-flight behavior).

## Scope

- `echo-agent/src/agent/react/run/stream_channel.rs` (full read:
  `run_stream_channel` :35-316, `direct_answer_stream` :362-480,
  `run_core_loop` :494-757, tests :759-2161).
- `echo-agent/src/agent/react/run/stream_macros.rs` (full read; the
  `yield_event_or` drop-on-full path :38-53), `run/types.rs`,
  `run/direct.rs`, `run/retry.rs` (:13-68 streaming retry wrapper),
  `run/processor.rs` (full read; `process_stream_chunk`, tool-args repair),
  `run/context.rs` streaming parts (`prepare_stream_context` :490-555,
  `prepare_stream_context_with_message` :561-623, `fire_lifecycle_hook`
  :305-484).
- `echo-agent/src/agent/react/run/phases/{prepare,think,tools,verify,compact,finalize,mod}.rs`
  (full reads; streaming send sequences and terminal paths).
- `echo-agent/src/agent/react/mod.rs` streaming entry points
  (`run_stream_entry` :1833-1870, `impl Agent for ReactAgent` streaming
  methods :2648-2933, `record_trace_event` :1872-1881).
- `echo-agent/echo-core/src/agent/mod.rs` (`AgentEvent` enum and
  `is_terminal` :331-336, cancel contract docs + `cancel_aware_stream`
  :540-660, :891-917).
- `echo-agent/echo-core/src/agent/event_envelope.rs` (full read:
  `envelope_event_stream` :107-194, `validate_event_trajectory` :196-295).
- `echo-agent/src/event_bus.rs` (EventBus/GLOBAL_EVENT_BUS integration check).
- `echo-agent/src/agent/handle.rs` (:95-149 unbounded re-stream wrapper).
- `echo-agent/src/agent/subagent/executor.rs` (:1182 envelope consumption,
  :2013 `Cancelled` producer — cross-reference only).
- EKO side (adapter evidence only): `echo-agent-cli/echo-agent-app-core/src/
  chat_driver.rs` (:480-569 stream consumption), `tasks/task_runtime/
  executor.rs` (:3119-3130, 3734 entry points), `agent_pool.rs` / `plugin_runtime.rs`
  (`AgentHandle` users).
- Git history: commit `297fc54` (2026-06-17, "fix(agent): unify HITL approval
  and thinking streams") — introduced `emit_content_tokens=false` and the
  end-of-stream burst re-emission.

## Out Of Scope

- Non-streaming driver internals (react_loop.rs wrapper, `process_steps` dead
  code, direct_answer non-streaming) → F-RCT-02 (P1-01/P3-01 re-checked only
  as conformance evidence).
- Tool batch execution internals (pipeline stages, per-tool timeouts,
  artifact spill) → F-RCT-04; only batch event emission and cancel-abandon
  terminal behavior were checked.
- Steer/interrupt/snapshot/resume mechanics → F-RCT-05 (steer mid-LLM test
  observed, not re-audited).
- Provider chunk fidelity (malformed-chunk drop, usage normalization) →
  F-LLM-01..03 (P1-01/P3-01 cross-referenced for the cancel and drop facts).
- EKO sink/renderer behavior on terminal events (one-terminal projection,
  cancel UX) → A-CHAT-01/A-SRF-01/A-FE-01; the EKO side is used only as
  consumer-contract evidence.
- Loop-detection dead code, `max_iterations=0` unbounded semantics →
  F-RCT-02-P2-02 (cross-referenced, not re-filed).

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, layering, no-parallel-semantics,
  surface parity), shared `README.md`, `REPORTING.md`, `TASKS.md` (F-RCT-03
  card), `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-RCT-02` (complete; handoff items
  P1-01/P2-03/P2-04 re-verified independently for the streaming side) and
  `F-LLM-01` (complete; P1-01 malformed-chunk drop and P3-01
  cancellation-is-silent used as transport facts).
- F-RCT-02 validation report `V02-01` (wrapper asymmetry, terminal producer
  inventory, hook double-fire anchors).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md`
  (terminal convergence model :44-58; drive_chat unification :775),
  `docs/PROJECT-ANALYSIS.md` (:228 LlmUsage anchors) — classified in the
  Historical Claim Status section.

## Layering Decision

- Generic mechanism (framework): the channel streaming entry, the unified
  core loop, phase send sequences, terminal-event ownership, the envelope
  normalization adapter, and the cancel contract all belong to `echo-agent`
  / `echo_core` and are correctly placed there. All findings stay in the
  framework; no repository movement is recommended.
- EKO product policy (application): EKO drives every turn through the
  streaming entry (chat_driver.rs:513, executor.rs:3119-3130) and consumes
  only the envelope-normalized stream (chat_driver.rs:538) — this is what
  turns the raw-stream contract defects (P1-02, P2-02) into product-visible
  behavior (cancel rendered as error; continuation hidden).
- Adapter boundary: `envelope_event_stream` (echo-core) is the single
  raw-stream → product adapter: it truncates at the first terminal
  (event_envelope.rs:174-177) and fabricates an Error on terminal-less ends
  (:180-191). It is thin and lossless in the happy path but currently masks
  the raw-stream multi-terminal defect instead of the loop guaranteeing one
  terminal (P2-02).
- Duplicate-search terms (both repositories, see V01): `run_stream_channel`,
  `run_core_loop`, `execute_stream`/`chat_stream` (+ `_with_cancel`,
  `_with_invocation_context`, `_message_*` variants), `cancel_aware_stream`,
  `envelope_event_stream`, `process_stream_chunk`, `emit_content_tokens`,
  `yield_event_or`/`yield_final_event_or`/`try_send_or`, `stream_buffer_size`,
  `AgentEvent::FinalAnswer`/`Cancelled`/`Error` producers, `EventBus`/
  `GLOBAL_EVENT_BUS`, `RwLockAgentWrapper`. Results: one streaming producer
  (channel-based, no `try_stream!`), one loop body, one envelope adapter; no
  parallel streaming implementation in `echo-agent-cli`; the global event bus
  has zero publishers (P3-01); `RwLockAgentWrapper` adds an unbounded-channel
  re-stream transport for `AgentHandle` users (noted, not a parallel loop).

## Current Path

Verified data flow: `Agent::execute_stream/chat_stream/…` (ReactAgent impl,
mod.rs:2780-2933) → `run_stream_entry`/`run_stream_message_entry`
(mod.rs:1833-1870) → `run_stream_channel` (stream_channel.rs:35): execution
mutex via `lock_owned`, trace run start, `prepare_stream_context[_with_message]`
(context.rs:490-623 — fires `UserPromptSubmit` once here), guard check
(blocked → single `FinalAnswer` + trace `Failed`, :141-179), IntentRouter
(DirectAnswer → `direct_answer_stream` :362-480; SkillRequired → activate
skill), then `tokio::spawn(run_core_loop)` (:302-312) with the wrapper
forwarding loop errors via `try_send` (:306-311). The loop body
(:494-757): `prepare_turn` (prepare.rs — fires `UserPromptSubmit` a second
time, :57-88) → per-iteration compact (compact.rs) → `run_think`
(think.rs:26-258: streaming LLM via `create_llm_stream`, chunk processing
with `emit_content_tokens=false`, per-chunk events via `yield_event_or`,
usage + end-of-stream content burst :199-247) → tools branch (`run_tools`
tools.rs:50-443, `final_answer` accepted → `finalize_completed_run`
finalize.rs:23-112) or text branch (`verify_final_text` → `emit_final_text`
finalize.rs:128-208, Stop-hook continuation possible after the `FinalAnswer`
send :179-201) or NoResponse (`finalize_no_response` :211-231) or
max-iterations (`finalize_max_iterations` :234-271). Consumers: EKO
chat_driver.rs:513,538 / executor.rs:3119-3130,3734 via
`envelope_event_stream` (event_envelope.rs:112 — truncates at first terminal,
synthesizes Error on terminal-less end); subagent executor wraps subagent
streams the same way (executor.rs:1182).

Terminal producers on the raw stream: FinalAnswer — guard block
(stream_channel.rs:163), direct_answer_stream (:478), prepare_turn hook block
(prepare.rs:70), finalize_completed_run (finalize.rs:87), emit_final_text
(finalize.rs:179); Error event — direct_answer_stream (stream_channel.rs:390);
Err items — think interventions (think.rs:47,65), tool batch timeout
(tools.rs:285-292), finalize_no_response (finalize.rs:226),
finalize_max_iterations (finalize.rs:267), wrapper forward (stream_channel.rs:310).
`AgentEvent::Cancelled`: never produced by the main loop (only
subagent/executor.rs:2013 and the default `cancel_aware_stream` wrapper, which
ReactAgent overrides away).

## Findings

### F-RCT-03-P1-01: Streaming events are silently dropped when the bounded channel is full — including terminal errors — so the stream is not lossless and can end with no terminal at all

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `run/stream_macros.rs:38-53` (`yield_event_or!` — `try_send`,
  `TrySendError::Full` → `warn!` + drop); used by every non-terminal event:
  think.rs:123,199-246 (reasoning Token deltas, ThinkStart/ThinkEnd, LlmUsage,
  content burst), prepare.rs:36-41 (MemoryRecalled), tools.rs:64-80,217-225,
  373-407 (ToolBatchStart, ToolCall, ToolResult, ToolError), compact.rs:85
  (ContextCompressed). Terminal `Err` items also use `try_send` and can be
  dropped on a full buffer: finalize.rs:226 (NoResponse), finalize.rs:267
  (MaxIterationsExceeded), think.rs:47,65 (intervention cancel/block),
  tools.rs:285-292 (batch timeout), stream_channel.rs:310 (wrapper error
  forward). Blocking `send().await` is used only for FinalAnswer,
  ToolStream, ToolBatchEnd, BudgetDecision. Channel capacity is
  `config.stream_buffer_size` = 256 (config.rs:112,235; stream_channel.rs:40).
- Reachability: every streaming turn; triggers when the consumer does not
  drain ~256 events (slow UI, paused sink, backpressure from the envelope
  wrapper). A dropped terminal error leaves a raw stream that ends with no
  terminal; the envelope then fabricates a generic
  `Error{"agent stream ended without a terminal event"}`
  (event_envelope.rs:180-191), replacing the specific NoResponse /
  MaxIterationsExceeded / intervention reason.
- Expected invariant: streaming deltas and terminal events are lossless
  (the task question); under backpressure the producer blocks or spills,
  never silently drops; a turn always ends with exactly one truthful
  terminal.
- Observed behavior: events (and on a full buffer, terminal errors) are
  dropped with only a warning; the warning hardcodes `DEFAULT_STREAM_BUFFER`
  = 256 (stream_macros.rs:43-46) even though the buffer size is a config
  field; raw-stream consumers can observe a terminal-less end.
- Impact: UI gaps (missing tokens, missing tool results, missing usage,
  unbalanced ThinkStart/End states), loss of error specificity on terminal
  failures, and a framework stream contract that cannot guarantee lossless
  delivery — the central invariant of this task.
- Root cause: a best-effort `try_send` design for intermediate events plus
  `try_send` for the four error-terminal paths; no drop accounting, no
  consumer-visible signal, no blocking fallback on the terminal paths.
- Direction: use blocking `send().await` (or a spill buffer) for
  intermediate events; convert the four terminal-Err paths to blocking sends
  (mirror `yield_final_event!`); expose a typed drop counter or
  "events dropped" event so consumers can detect loss; fix the log to print
  the configured buffer size. No deletion needed beyond the drop arm.
- Regression validation: unit test with a tiny buffer (e.g. `buffer = 1`) and
  a slow consumer asserting zero dropped events (or a counted drop); a
  terminal-path test with a full buffer asserting the `Err` still arrives
  (blocking) and the envelope carries the specific error.
- Validation reports: [V01](../validations/F-RCT-03/V01-01.md),
  [V02](../validations/F-RCT-03/V02-01.md), [V03](../validations/F-RCT-03/V03-01.md)

### F-RCT-03-P1-02: Cancellation never yields the documented `AgentEvent::Cancelled` terminal on the ReactAgent streaming path — cancelled turns surface as NoResponse errors or synthesized stream errors

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: the `Agent` trait documents "When `cancel` is triggered, the
  stream yields [`AgentEvent::Cancelled`] and terminates" (echo-core/src/
  agent/mod.rs:552-553, 617-618) and provides `cancel_aware_stream`
  (:896-917) used only by the two default impls (:567,626). ReactAgent
  overrides all four cancel-carrying streaming methods
  (react/mod.rs:2821-2933) to set the token and call `run_stream_entry` —
  `cancel_aware_stream` is never reached. On the main path a cancelled
  mid-LLM call ends the stream silently (transport behavior, F-LLM-01-P3-01)
  → empty think output → `IterOutcome::NoResponse` → `Err(NoResponse)`
  (finalize.rs:226); cancel during a tool batch → `ToolBatchEnd` +
  `IterOutcome::Abandoned` → stream ends with no terminal (tools.rs:295-300,
  418-423). `AgentEvent::Cancelled` producers: subagent/executor.rs:2013 only.
  Test `test_run_stream_cancelled_mid_llm_call` (stream_channel.rs:1971-2038)
  asserts only "no FinalAnswer", not a `Cancelled` event.
- Reachability: every cancelled EKO turn (chat_driver.rs:513,
  executor.rs:3119) and any framework consumer of the ReactAgent streaming
  API.
- Expected invariant: per the public trait contract, a cancelled turn
  terminates with exactly one `AgentEvent::Cancelled` terminal, and
  "cancelled" is distinguishable from "no answer" and "error".
- Observed behavior: user cancellation is reported as a NoResponse error, or
  — when the tool-batch abandon path wins — as a terminal-less end that the
  envelope converts into a fabricated `Error` ("agent stream ended without a
  terminal event", event_envelope.rs:180-191).
- Impact: product surfaces render user-initiated cancellation as an error or
  no-answer; framework consumers cannot implement a "cancelled" UX per the
  documented contract; subagent and main paths emit different terminal
  vocabularies for the same outcome.
- Root cause: ReactAgent's cancel variants were reimplemented to thread the
  token into the LLM/tool layers without preserving the wrapper's `Cancelled`
  emission — contract drift between trait docs and the live implementation.
- Direction: emit `AgentEvent::Cancelled` at the cancel terminal points of
  `run_core_loop` (e.g., in the think and tools abandon paths when the cancel
  token fired) with trace finalization, or restore the `cancel_aware_stream`
  wrapper on top of the token threading; align subagent/main terminal
  semantics; update the trait docs only if silent-end is the intended
  contract.
- Regression validation: mocked test — cancel mid-LLM and cancel mid-tool-
  batch → each stream contains exactly one terminal `AgentEvent::Cancelled`;
  an envelope-level test asserting `Cancelled` passes through unchanged.
- Validation reports: [V01](../validations/F-RCT-03/V01-01.md),
  [V02](../validations/F-RCT-03/V02-01.md), [V03](../validations/F-RCT-03/V03-01.md)

### F-RCT-03-P2-01: The main ReAct streaming path does not stream content deltas — the whole model response is buffered and emitted as one `Token` burst after the LLM stream ends

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: think.rs:110-125 calls `process_stream_chunk(..., false)`; the
  `emit_content_tokens` flag gates per-delta content `Token` emission
  (processor.rs:50-54) and is `false` in the only production caller; after
  the stream ends the accumulated content is re-emitted as a single
  `Token(content_buffer)` (think.rs:224-247, with a synthetic
  `ThinkStart…ThinkEnd{pt,ct}` envelope when tool calls are also present,
  :225-239). Git history: commit `297fc54` (2026-06-17, "fix(agent): unify
  HITL approval and thinking streams") introduced the flag AND the
  end-of-stream re-emission; before it, per-delta content Tokens were
  emitted. `direct_answer_stream` still streams per-delta
  (stream_channel.rs:405-414). `Token` is documented as "LLM is generating a
  token (streaming)" (echo-core/src/agent/mod.rs:146). `emit_content_tokens`
  has no caller passing `true` (V01).
- Reachability: every ordinary streaming turn (EKO main path) — reasoning
  deltas stream per chunk, content arrives as one burst at LLM-call end.
- Expected invariant: streaming deltas are delivered incrementally and in
  order (the task question; the pre-`297fc54` behavior and the
  direct-answer path both do this).
- Observed behavior: a text-only turn emits
  `[LlmUsage, Token(full content), FinalAnswer]`; the content `Token` is a
  single burst; mixed reasoning+content+tool-call turns emit a second
  synthetic `ThinkStart`/`ThinkEnd` pair around the burst, and
  `LlmUsage` precedes the content burst.
- Impact: streaming UIs cannot render answer text incrementally on the main
  path (bursty rendering, no per-token UX); ordering quirks (orphan
  ThinkStart/End pairs, usage before content) can confuse consumer state
  machines; behavior diverges between entry points.
- Root cause: the June unification commit turned off per-delta content
  emission without a documented rationale and substituted a post-hoc burst
  re-emission.
- Direction: restore per-delta emission (`emit_content_tokens = true`) and
  drop the end-of-stream re-emission, or formally adopt the buffered
  contract, document it, and clean the synthetic ThinkStart/ThinkEnd
  envelope; align `direct_answer_stream` and the main loop on one contract.
- Regression validation: mocked multi-chunk stream (content split over
  several chunks) → assert per-delta `Token` events in order (or the
  documented burst contract); a reasoning+content+tools fixture asserting no
  duplicate ThinkStart/ThinkEnd and correct ordering of LlmUsage/Token.
- Validation reports: [V01](../validations/F-RCT-03/V01-01.md),
  [V02](../validations/F-RCT-03/V02-01.md), [V03](../validations/F-RCT-03/V03-01.md)

### F-RCT-03-P2-02: A Stop-hook continuation can emit a second `FinalAnswer` on the raw stream; the envelope adapter truncates it, so the continuation runs unseen (hidden LLM call and side effects) after the product already observed "final"

- Priority: P2
- Confidence: high
- Layer: framework (raw stream) + adapter (envelope)
- Evidence: `emit_final_text` sends `FinalAnswer` first (finalize.rs:179),
  then consults the Stop hook; a `continue_reason` returns
  `ControlFlow::Continue` (finalize.rs:190-201) → the driver `continue`s
  (stream_channel.rs:694-695) and a later iteration can emit a second
  `FinalAnswer` (emit_final_text or finalize_completed_run); the trace is
  finalized `Completed` before the continuation (finalize.rs:175).
  `envelope_event_stream` breaks at the first terminal
  (event_envelope.rs:174-177); its own test
  `normalizes_missing_and_duplicate_terminals` (event_envelope.rs:366-379)
  confirms truncation. EKO consumes only the enveloped stream
  (chat_driver.rs:538).
- Reachability: any text-branch final answer on an agent whose Stop hook
  registry returns `continue_reason` (scripted via hooks.rs:1429); this is
  the streaming-side re-verification of F-RCT-02-P2-04, whose finding this
  report independently confirmed rather than copied.
- Expected invariant: one terminal per turn on the raw stream; no work after
  the consumer observes the terminal; persisted trace status agrees with the
  emitted terminal.
- Observed behavior: raw stream can carry two `FinalAnswer`s; product
  consumers see one (truncation), but the spawned loop continues — a hidden
  extra LLM request and any tool side effects execute with zero
  observability, and the trace is already `Completed`.
- Impact: hidden token spend and invisible side effects after the "final"
  answer; framework consumers of the raw stream see a multiple-terminal
  violation; per-turn accounting under-reports.
- Root cause: the one-shot continuation was layered onto a terminal path that
  emits the terminal first and finalizes first (same root cause as
  F-RCT-02-P2-04); the envelope masks the defect instead of the loop
  guaranteeing one terminal.
- Direction: consult the Stop hook BEFORE emitting `FinalAnswer` and skip the
  send when continuing; finalize the trace only on true termination; have the
  envelope log/flag truncation so masking is observable; align with the
  F-RCT-02-P2-04 fix.
- Regression validation: streaming test with a Stop-hook `continue_reason` →
  exactly one `FinalAnswer` per turn and no loop continuation; envelope
  truncation test stays green and additionally asserts a truncation signal.
- Validation reports: [V02](../validations/F-RCT-03/V02-01.md),
  [V03](../validations/F-RCT-03/V03-01.md)

### F-RCT-03-P2-03: `direct_answer_stream` mid-stream errors finalize the trace `Completed` and push a (possibly empty) assistant message, while the consumer receives an `Error` terminal

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: on a chunk `Err`, `direct_answer_stream` sends
  `AgentEvent::Error` to `tx` and returns `Ok(content)` with the partial
  content (stream_channel.rs:385-396); the caller treats any `Ok` as a
  completed direct answer: `finalize_run(Completed, Some(partial), None)`
  (stream_channel.rs:235) and an unconditional
  `push(Message::assistant(content))` (stream_channel.rs:238-242) — with
  `content == ""` when the error occurs before any delta.
- Reachability: any DirectAnswer-routed turn whose provider transport fails
  mid-stream (network error, provider disconnect); reachable whenever an
  `IntentRouter` with `allows_direct_answer_shortcut` is configured.
- Expected invariant: the emitted terminal and the persisted run status
  agree; no assistant message (especially an empty one) is appended on
  failure.
- Observed behavior: consumer sees an `Error` terminal while the trace says
  `Completed` with partial content as `final_output`; an empty assistant
  message is appended to conversation history on early errors.
- Impact: run-history contradicts the event stream; empty assistant turns
  pollute history and downstream compression; `final_output` is a truncated
  partial answer labeled as complete.
- Root cause: `direct_answer_stream` collapses the error case into
  `Ok(partial)` and the caller has no way to distinguish "success" from
  "error after partial output".
- Direction: return a distinguishable failure (e.g. `Err` after the Error
  event, or a typed result) so the caller finalizes the trace `Failed` and
  skips the assistant push (mirror the guard path's
  `finalize_scoped_trace_run(Failed)`, stream_channel.rs:167-173); drop the
  unconditional push.
- Regression validation: unit test — mock stream yielding a chunk `Err`
  mid-way through `direct_answer_stream` → trace status `Failed`, no
  assistant message pushed, `Error` event delivered exactly once.
- Validation reports: [V02](../validations/F-RCT-03/V02-01.md),
  [V03](../validations/F-RCT-03/V03-01.md)

### F-RCT-03-P2-04: Abandoned streams (consumer close, tool-batch cancellation) leave the trace run `Running` with no terminal and no finalization; the envelope then invents an error

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: every abandon path returns `Ok(())` without `finalize_run`:
  prepare.rs:70-78 (`BlockedAndDone`/`Abandoned`), compact.rs:90-94,
  think.rs:123-124, tools.rs:295-300 and :418-423 (cancellation abandon →
  `ToolBatchEnd` + `Abandoned`); the driver returns `Ok(())`
  (stream_channel.rs:604-610, 745-750); channel-close aborts are prompt (the
  macros return `Ok(())` on `Closed`, stream_macros.rs:48-50,58-61). The
  envelope converts the terminal-less end into a fabricated `Error`
  (event_envelope.rs:180-191).
- Reachability: any turn whose consumer drops the stream mid-flight, or
  whose tool batch is cancelled and exceeds the 5s drain grace.
- Expected invariant: every stream termination (normal, error, cancel,
  abandon) finalizes the trace with a truthful status (same invariant class
  as F-RCT-02-P2-01).
- Observed behavior: the trace stays `Running` with no `final_output`; on
  cancel-abandon the product-facing envelope emits "agent stream ended
  without a terminal event".
- Impact: run-list and observability consumers see perpetually-running turns
  ("running vs hung" indistinguishable); token accounting for partial runs is
  incomplete; cancellations are recorded as errors.
- Root cause: trace finalization is wired only into the four `finalize.rs`
  terminal helpers and the direct-answer paths; the abandon early-returns
  bypass it.
- Direction: finalize the trace in the driver when the loop returns without a
  terminal (e.g. `RunStatus::Failed`/`Cancelled` with a reason) or at each
  abandon point, and distinguish cancel from consumer-close where possible.
- Regression validation: test — drop the stream mid-LLM and mid-tool-batch →
  trace status `Failed`/`Cancelled` with a recorded reason; a cancel-vs-close
  classification fixture.
- Validation reports: [V02](../validations/F-RCT-03/V02-01.md),
  [V03](../validations/F-RCT-03/V03-01.md)

### F-RCT-03-P3-01: The unified event bus is unconnected — `GLOBAL_EVENT_BUS` has zero publishers and the streaming path has no event-bus integration despite the module's and docs' claims

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `src/event_bus.rs` (EventBus :14-61, `GLOBAL_EVENT_BUS` :57-61,
  module doc "Unified event bus … replacing the current scattered per-
  frontend event mapping"); repo-wide grep: zero `EventBus`/`GLOBAL_EVENT_BUS`
  uses outside event_bus.rs; `record_trace_event` doc claims it "publishes
  trace lifecycle to global event bus for audit subscribers" but the body
  only appends to the run store (mod.rs:1872-1881); the task card's "event
  bus integration" primary path has no implementation in
  `src/agent/react/`.
- Reachability: none at runtime (dead public surface); the streaming path
  delivers events only through the per-invocation channel + envelope.
- Expected invariant: either the bus is wired (React loop publishes
  envelopes) or it is not claimed to be the unified transport.
- Observed behavior: a public, documented unified transport with no
  publishers; misleading doc comment.
- Impact: dead code and false documentation; future Webhook/Trace/UI/Audit
  integration would silently subscribe to an empty bus.
- Root cause: the bus was scaffolded as the target transport but never wired
  into the React loop when the envelope adapter shipped.
- Direction: publish envelopes from `run_stream_channel` (or the envelope
  adapter) onto `GLOBAL_EVENT_BUS`, or delete the module and fix the
  `record_trace_event` comment; coordinate with F-CORE-01/X-BND-01 on whether
  the bus is the chosen transport.
- Regression validation: a subscription test — one streaming turn publishes
  the same envelopes a channel consumer receives (or, if deleted, grep
  `GLOBAL_EVENT_BUS` returns nothing).
- Validation reports: [V01](../validations/F-RCT-03/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search across both repositories (streaming producers/consumers, envelope, cancel wrapper, event bus, chunk processor) | yes | passed | [V01-01](../validations/F-RCT-03/V01-01.md) |
| V02 | Registration and runtime reachability trace (streaming entry chain, terminal producer inventory, envelope adapter, F-RCT-02 handoff re-verification) | yes | passed | [V02-01](../validations/F-RCT-03/V02-01.md) |
| V03 | Invariant/edge-case inspection vs tests (lossless/backpressure, channel close, duplicate terminal, cancel terminal, content-delta streaming, conformance fixture, test coverage) | yes | passed | [V03-01](../validations/F-RCT-03/V03-01.md) |
| V04 | `cargo test -p echo_agent --lib --locked 'react::run::stream_channel'` + `'react::run::phases'` + `'react::run::processor'` + `cargo test -p echo_core --lib --locked 'event_envelope'` | yes | passed (exit 0 / 0 / 0 / 0; 23+22+7+6 passed) | [V04-01](../validations/F-RCT-03/V04-01.md) |
| V05 | Historical-document drift (MASTER-PLAN terminal convergence; PROJECT-ANALYSIS LlmUsage anchors) | conditional | passed | [V05-01](../validations/F-RCT-03/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:44-58 (Codex reference): delta stream + `turn/completed` terminal convergence as the design target | regressed (raw stream); contained at adapter | main loop emits content as one burst (P2-01); raw stream can emit a second FinalAnswer (P2-02); envelope truncates/fabricates (event_envelope.rs:174-191) — [V05](../validations/F-RCT-03/V05-01.md) |
| PROJECT-ANALYSIS:228 "`AgentEvent::LlmUsage` 在 stream_channel.rs:331 / phases/think.rs:185 发出" | stale (anchors), current (semantics) | LlmUsage emission now at think.rs:199-211 and stream_channel.rs:465-475 — [V05](../validations/F-RCT-03/V05-01.md) |
| MASTER-PLAN:775 "streaming/non-streaming 路径统一进入 drive_chat" | current | EKO consumes the envelope-normalized streaming stream exclusively (chat_driver.rs:513,538) |
| Agent trait cancel contract: "cancel → stream yields `AgentEvent::Cancelled` and terminates" (echo-core mod.rs:552-553, 617-618) | regressed | ReactAgent overrides bypass `cancel_aware_stream`; main loop never emits Cancelled (P1-02) |
| `Token` event semantics "LLM is generating a token (streaming)" (echo-core mod.rs:146) | regressed on main loop | content Tokens delivered as one end-of-stream burst (P2-01); per-delta only in direct_answer_stream |

## Coverage And Uncertainty

- All conclusions are static except four test runs (V04); no dynamic run
  exercised: a full buffer with a slow consumer, a Stop-hook `continue_reason`
  on the streaming path, a cancel-token mid-tool-batch, a mid-stream chunk
  error through `direct_answer_stream`, or a consumer dropping the stream
  (no such tests exist — V03).
- P1-01's practical frequency depends on consumer draining speed; the code
  path is fully verified, the trigger (256 unread events) is plausible for a
  busy GUI/paused sink, and the terminal-Err drop needs only the buffer to be
  full at finalize time.
- P1-02's product-visible outcome (EKO rendering "No response from LLM" on
  cancel) is inferred from the framework facts plus the EKO consumption
  points; the exact EKO projection is A-CHAT-01/A-SRF-01 scope and was not
  read.
- The tool pipeline's internal 64-capacity channels (tools.rs:141,319,
  pipeline.rs:515,1642) were not audited for their own drop behavior —
  F-RCT-04 scope; ToolStream relay to the main channel is blocking.
- `RwLockAgentWrapper`'s unbounded re-stream (handle.rs:95-149) was noted but
  not deep-read; whether any live `AgentHandle` streaming consumer exists in
  EKO was not established (agent_pool.rs uses `AgentHandle` for task
  execution).
- F-RCT-02-P2-02 (dead LoopDetector, `max_iterations=0` unlimited) is
  cross-referenced for the "bounded" dimension and not re-filed.
- The F-RCT-02 handoff items P1-01/P2-03/P2-04 were re-verified
  independently on the streaming side (V02) and confirmed; P2-04's
  streaming-specific envelope-truncation evidence is new to this task.

## Handoff

- Downstream tasks may rely on: single streaming producer + single loop +
  single envelope adapter (V01); terminal producer inventory and the
  terminal-Err drop paths (V02/V03); green test state at the reviewed
  commits (V04); the raw-stream double-terminal fact and envelope
  truncation (P2-02) as the authoritative producer/adapter behavior.
- A-CHAT-01 must treat cancelled turns as arriving with a NoResponse or
  fabricated stream Error (P1-02) and post-terminal continuation work as
  invisible to sinks (P2-02); the "one-terminal invariant" holds only at the
  envelope boundary, not on the raw stream.
- F-RCT-04 should check the 64-capacity tool-pipeline channels for their own
  drop semantics and confirm the tools.rs cancel-abandon path (P2-04
  evidence).
- X-EVT-01 should reconcile the framework's two cancel vocabularies (main
  loop never emits Cancelled; subagent executor does) and the envelope's
  terminal normalization with the frontend contract.
- X-BND-01 should settle the event-bus wiring-vs-deletion decision
  (P3-01) and confirm no external consumer of `GLOBAL_EVENT_BUS` exists.
- F-RCT-02-P2-04 and this task's P2-02 should be fixed together (one
  contract for Stop-hook continuation).
- Reports to read: this report + [V01-01](../validations/F-RCT-03/V01-01.md)
  through [V05-01](../validations/F-RCT-03/V05-01.md); F-RCT-02 (loop and
  non-streaming findings), F-LLM-01 (transport cancel/malformed-chunk facts).
- Stale triggers: any change to `run/stream_channel.rs` send paths,
  `run/stream_macros.rs` macros, `phases/*` terminal or abandon handling,
  `processor.rs` `emit_content_tokens`, the cancel variants in
  `react/mod.rs`, `event_envelope.rs` normalization, or `AgentEvent` variant
  semantics invalidates the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): A-CHAT-01,
  F-RCT-04, X-EVT-01, X-BND-01, Q-FLT-01 (backpressure and cancel fault
  scenarios), Q-TST-01 (fixture gaps from V03).
