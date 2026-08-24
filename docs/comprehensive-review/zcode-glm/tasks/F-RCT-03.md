# F-RCT-03: Streaming ReAct event flow

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Are streaming deltas and terminal events ordered, lossless, bounded, and
semantically equivalent to non-streaming execution?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/agent/react/run/stream_channel.rs` (2161 lines) —
  `run_stream_channel` (streaming entry, `:35-316`),
  `direct_answer_stream` (DirectAnswer shortcut, `:362-480`),
  `AgentRunSnapshot::run_core_loop` (the single shared loop body,
  `:494-756`), and the streaming test module (`:759-2161`).
- `echo-agent/src/agent/react/run/stream_macros.rs` (79 lines) —
  `yield_event_or!` (try_send, drop on Full), `yield_final_event!` /
  `yield_final_event_or!` (send().await, block), `try_send_or!`
  (error-forwarding try_send), `DEFAULT_STREAM_BUFFER = 256`.
- `echo-agent/src/agent/react/run/processor.rs` (271 lines) —
  `process_stream_chunk` (the single `ChatCompletionChunk → AgentEvent`
  converter), `parse_tool_args` (DeepSeek repair),
  `build_tool_calls_from_map` (drop-unparseable semantics).
- `echo-agent/src/agent/react/run/phases/think.rs` (527 lines) —
  `run_think` (LLM stream → buffered output + events), `create_llm_stream`
  (LlmClient trait path + legacy reqwest path).
- `echo-agent/src/agent/react/run/phases/tools.rs` (443 lines) —
  `run_tools` (ToolBatchStart/ToolCall/ToolResult/ToolError/ToolStream/
  ToolBatchEnd emission, concurrent + serial batches).
- `echo-agent/src/agent/react/run/phases/prepare.rs` (93 lines) —
  `prepare_turn` (MemoryRecalled, UserPromptSubmit hook block).
- `echo-agent/src/agent/react/run/phases/compact.rs` (501 lines) —
  `run_compact` (ContextCompressed event).
- `echo-agent/src/agent/react/run/phases/finalize.rs` (362 lines) —
  the four terminal helpers and their send APIs.
- `echo-agent/src/agent/react/run/react_loop.rs:598-751` — the
  non-streaming `run_react_loop` collector (for V04 conformance).
- `echo-agent/src/agent/react/mod.rs:1833-1870, 2767-2930` — streaming
  trait impls and `run_stream_entry` / `run_stream_message_entry`.
- `echo-agent/echo-core/src/agent/mod.rs:140-340, 555-569, 896-917` —
  `AgentEvent` enum, `is_terminal()`, `cancel_aware_stream` default impl.
- `echo-agent/echo-core/src/agent/event_envelope.rs:64-194` —
  `stable_event_id`, `envelope_event_stream_after` (consumer-side
  wrapping, per F-CORE-01).
- `echo-agent/src/agent/config.rs:111-112, 235` — `stream_buffer_size`
  (default 256, documented lossy).

## Out Of Scope

Deferred to named task IDs:

- The 13-stage tool-execution pipeline in `pipeline.rs` (per-tool-call
  middleware) — **F-RCT-04**. This task confirms tool events reach the
  channel but does not audit the pipeline internals.
- `ContextManager` compression algorithm correctness — **F-MEM-01** /
  **F-CMP-01**.
- LLM client / provider routing / SSE transport — **F-LLM-01/02/03**.
- Snapshot/resume/checkpoint round-trip — **F-RCT-05**.
- Application-layer event projection (`chat_driver`, `task_runtime`,
  `a2a/server`) envelope consumption — the application tasks. This task
  confirms the framework returns raw `Result<AgentEvent>` streams and the
  consumer applies `envelope_event_stream`; the consumer-side projection
  is out of scope.
- Steer-during-LLM semantics (sampled in
  `steer_during_llm_call_continues_same_turn_with_new_input`) — adjacent
  to F-RCT-05.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  system reminder — especially the framework-vs-application layering
  gate, the no-panic / UTF-8 safety rules, and the dead-code cleanup
  rule).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-RCT-02.md` (in full) +
  its four validations (V01–V04). F-RCT-02 establishes the single
  `run_core_loop` body, the 10-arm terminal partition, `max_iterations`
  + soft budgets as the only loop bounds, dead `LoopDetector`, and the
  trace-finalization asymmetry (F-RCT-02-P2-03 / P3-01). This task
  inherits those conclusions and re-analyses the terminal arms through
  the streaming send-API lens.
- `docs/comprehensive-review/zcode-glm/tasks/F-CORE-01.md` (in full) +
  its four validations. F-CORE-01 establishes `AgentEvent`,
  `EventEnvelope`, `EventIdentity`, the `Agent` trait, and
  `cancel_aware_stream`. Its finding F-CORE-01-P2-01 (dead
  `GLOBAL_EVENT_BUS`) and F-CORE-01-P2-02 (`event_id` cross-stream
  collision) are the structural context for this task's envelope-layer
  analysis.

Historical documents treated as hypotheses:

- `echo-agent/src/agent/react/run/stream_channel.rs:1-16` module
  docstring — claims "all streaming execution goes through this module"
  and convergence with the non-streaming pre-flight. Treated as
  **current** — verified by V01 and V04.
- `echo-agent/src/agent/react/run/phases/mod.rs:1-4` docstring — claims
  `run_core_loop` is the single unified loop. Treated as **current**
  (confirmed by F-RCT-02; re-verified for streaming in V04).
- `echo-agent/src/agent/config.rs:111-112` doc comment — claims "When
  full, events are dropped with a warning". Treated as **current and
  load-bearing** — this is the documented contract that V02 confirms and
  flags against the task's "lossless" criterion.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `run_stream_channel`, the three send macros, the mpsc channel construction, `process_stream_chunk`, and `AgentEvent` are generic agent-runtime streaming machinery any `echo-agent` consumer needs. They live correctly in `echo-agent` (root crate) and `echo-core` (`AgentEvent`, `cancel_aware_stream`). |
| EKO product policy | None at this layer. The buffer size (256), the drop-on-full policy, and the cancellation surfacing are framework defaults; EKO product policy enters only through the consumer-side envelope wrapper and the application cancellation polling (F-CORE-01). |
| Adapter boundary | The framework exposes `BoxStream<Result<AgentEvent>>` as the streaming contract. `EventEnvelope` wrapping is the consumer's responsibility — the framework does not call `envelope_event_stream` anywhere. The application adapter (`chat_driver.rs:538`, `executor.rs:3743`) wraps the raw stream losslessly (per F-CORE-01). |
| Duplicate search | Searched names: `run_stream_channel`, `run_core_loop`, `direct_answer_stream`, `direct_answer`, `run_react_loop`, `cancel_aware_stream`, `process_stream_chunk`, `yield_event_or`, `yield_final_event`, `try_send_or`, `stream_buffer_size`, `DEFAULT_STREAM_BUFFER`. Searched behaviours: delta→event conversion, channel send semantics, terminal emission, cancellation surfacing. Result: one canonical streaming entry (`run_stream_channel`); one canonical loop body (`run_core_loop`, shared with non-streaming per F-RCT-02); one canonical chunk converter (`process_stream_chunk`); one canonical envelope wrapper (`envelope_event_stream_after`, in the consumer layer). No parallel/sibling streaming pipeline exists. |
| Migration deletion | No deletion proposed in this task. The cancellation divergence (F-RCT-03-P2-02) could be fixed by wrapping `cancel_aware_stream` inside `chat_stream_with_cancel`, but that is a behaviour change, not a dead-code deletion. |

## Current Path

Verified streaming event flow at commit `9b0e0fa`. The four-stage
pipeline (full detail in V01-01):

```text
Provider SSE stream
   ↓  crate::llm::stream_chat / llm_client.chat_stream    [think.rs:329, 366]
   ↓  retry::retry_llm_call wrapper (exp backoff + CB)    [think.rs:289, 354]
ChatCompletionChunk stream  (Box<dyn Stream<Item = Result<ChatCompletionChunk>>>)
   ↓  create_llm_stream returns this                      [think.rs:261-393]
   ↓  run_think: while let Some(cr) = llm_stream.next()   [think.rs:110]
        try_send_or!(tx, cr, Abandoned)  ← on Err: forward Err, bail
        process_stream_chunk(&chunk, buffers...) → Vec<AgentEvent>  [processor.rs:16]
        for event in events { yield_event_or!(tx, event, Abandoned) }
   ↓  post-stream: LlmUsage + ThinkEnd + Token(content_buffer flush)  [think.rs:199-247]
AgentEvent stream  (inside mpsc<(Result<AgentEvent>)>)
   ↓  run_core_loop driver consumes ThinkOutput → run_tools / verify / finalize
   ↓  terminal: FinalAnswer via send().await  |  Err(...) via try_send
Result<AgentEvent> on mpsc::ReceiverStream
   ↓  returned as Box::pin(ReceiverStream::new(rx))       [stream_channel.rs:314]
BoxStream<Result<AgentEvent>>  (framework contract)
   ↓  consumer wraps: envelope_event_stream(raw, identity) [chat_driver.rs:538 / executor.rs:3743]
BoxStream<EventEnvelope>  (consumer-side wire contract)
```

**Channel construction** (`stream_channel.rs:40-41`):
`mpsc::channel::<Result<AgentEvent>>(self.config.stream_buffer_size)`,
default 256 (`config.rs:235`). The `tx` is moved into the spawned
`run_core_loop` task; `rx` is returned as the BoxStream.

**Send semantics** (the load-bearing detail — full table in V01-01 /
V02-01):

| Path | API | On `Full` | Used for |
|---|---|---|---|
| `yield_event_or!` | `try_send` | **DROP** + warn | 16 intermediate-event sites |
| `yield_final_event!`/`_or!` | `send().await` | BLOCK | 12 terminal + ToolStream + ToolBatchEnd sites |
| direct `tx.send().await` | `send().await` | BLOCK | 8 sites (FinalAnswer, BudgetDecision, DirectAnswer tokens) |
| direct `tx.try_send(Err)` | `try_send` | **DROP** (`let _`) | 5 terminal-error sites |

**Pre-loop short-circuits** (`run_stream_channel:141-280`): guard block
→ `FinalAnswer("blocked")` via `send().await`; IntentRouter DirectAnswer
→ `direct_answer_stream` (per-token streaming + terminal FinalAnswer
via `send().await`); SkillRequired → fall through to loop spawn. These
mirror the non-streaming `prepare_react_context` + IntentRouter
(`react_loop.rs:603-682`) — verified equivalent in V04.

**Spawned task** (`stream_channel.rs:302-312`):
`tokio::spawn(run_core_loop(snap, ctx, text, msg, label, mode, recalled,
tx.clone()))`. If the loop returns `Err(e)`, it is forwarded via
`tx.try_send(Err(e))` (result discarded). The `execution_guard` and
`active_turn_lease` are moved into the task and held for the full
stream lifetime.

Key invariants verified (full evidence in V01–V04):

- **Single streaming entry.** `Agent::execute_stream` / `chat_stream` /
  `chat_stream_with_cancel` / `execute_stream_with_invocation_context` /
  the `_message_with_*` variants all delegate to `run_stream_entry` /
  `run_stream_message_entry` → `run_stream_channel`. No sibling
  streaming path. (V01)
- **Single chunk converter.** `process_stream_chunk` is the only
  `ChatCompletionChunk → AgentEvent` converter; pure function, no I/O.
  (V01)
- **Framework returns raw events; consumer wraps envelopes.** Zero
  `envelope_event_stream` calls inside `echo-agent/src`; the wrapper
  lives in the consumer layer (F-CORE-01). (V01)
- **Bounded capacity, lossy intermediates.** 256-slot mpsc; intermediate
  events dropped on full with a warn log (documented at `config.rs:111`).
  Terminal success events use blocking `send().await` and are lossless.
  (V02)
- **Loop-body equivalence with non-streaming.** Both entries spawn the
  same `run_core_loop` with the same snapshot shape; same tools, same
  budgets, same terminal partition. (V04, inherits F-RCT-02)

## Findings

### F-RCT-03-P1-01: Non-terminal streaming events are silently dropped under backpressure, breaking the lossless-streaming contract for live UX rendering

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/stream_macros.rs:38-53` —
    `yield_event_or!` uses `tx.try_send(Ok($event))`; on
    `TrySendError::Full` it emits `tracing::warn!("Stream buffer full
    ({}), dropping event", DEFAULT_STREAM_BUFFER)` and falls through
    (the event is gone).
  - `echo-agent/src/agent/config.rs:111-112` — doc comment:
    "Streaming channel buffer size (default 256). When full, events are
    dropped with a warning." (documented lossy contract).
  - 16 production sites use `yield_event_or!` for events the consumer
    renders live: `Token(reasoning)` / `Token(content)`
    (`think.rs:123,229,232,241,243`), `ThinkStart`/`ThinkEnd`
    (`think.rs:214,217,226,232,234`), `LlmUsage` (`think.rs:199`),
    `ToolBatchStart` (`tools.rs:64`), `ToolCall` (`tools.rs:72`),
    `ToolResult` (`tools.rs:217,373`), `ToolError` (`tools.rs:242,398`),
    `MemoryRecalled` (`prepare.rs:36`), `ContextCompressed`
    (`compact.rs:83`), `ToolBatchEnd` (`tools.rs:430`).
  - `echo-agent/src/agent/react/run/phases/think.rs:110-125` — the chunk
    loop has no `await` keyed to mpsc capacity between chunks; the only
    backpressure is the LLM client's own network stream.
- Reachability: every streaming turn under a slow consumer. The buffer
  (256) fills when the producer emits faster than the consumer drains
  (UI render stall, network write to remote client, GC pause). Long
  reasoning traces can emit hundreds of `Token(reasoning)` deltas in a
  tight CPU loop.
- Expected invariant: the F-RCT-03 task question asks whether streaming
  deltas are "lossless". A streaming API that drops `Token` events
  defeats the purpose of streaming — the consumer's rendered text has
  holes that cannot be reconciled until the terminal `FinalAnswer`
  arrives.
- Observed behavior: under sustained backpressure, `Token` events are
  dropped after the 256-slot buffer fills. The agent's internal buffers
  (`content_buffer`, `reasoning_buffer`, `tool_call_map` in
  `processor.rs`) are updated unconditionally, so the agent's own state
  and the final `FinalAnswer` are correct — but the consumer's live
  view of the stream is garbled (missing tokens, orphan `ThinkEnd`
  without `ThinkStart`, unbalanced `ToolCall`/`ToolResult` pairs).
- Impact: any consumer that renders `Token` events as they arrive (the
  entire point of streaming — e.g. a chat UI typing indicator, a TUI
  streaming pane, an audit log) sees corrupted output under load. The
  corruption is silent (only a `tracing::warn!` that does not reach the
  consumer's event stream). For reasoning models (Qwen3/DeepSeek) the
  reasoning trace can be long enough to fill 256 slots in a single
  think phase.
- Root cause: the macros were written with a "drop is safe because the
  buffers accumulate" mental model — correct for the agent's internal
  state, incorrect for the consumer's streaming UX. The
  `send().await` vs `try_send` split was chosen to prevent producer
  stalls, but applied too broadly (to events the consumer renders).
- Direction: change `yield_event_or!` to use `send().await` (matching
  `yield_final_event_or!`), OR introduce a third macro
  `yield_event_backpressure!` that blocks on `Full` but still
  short-circuits on `Closed`. The performance concern (producer stall
  on slow consumer) is the *desired* streaming backpressure — a slow
  consumer SHOULD slow the producer, not receive garbled output. If a
  truly non-blocking path is needed for some event class, make it
  explicit at the call site rather than the default. Alternatively,
  raise the default buffer (256 → 4096) as a stopgap, but this only
  delays the drop, does not fix it.
- Regression validation: a new test that drives `run_think` with a mock
  LLM emitting >256 chunks and a consumer that drains slowly (e.g.
  `tokio::time::sleep` per event) must assert (a) zero events dropped
  (every emitted chunk produces a delivered `Token`) and (b) the
  producer's spawned task is suspended (backpressure observed) while
  the consumer is slow. Today no such test exists.
- Validation reports: [V01](../validations/F-RCT-03/V01-01.md),
  [V02](../validations/F-RCT-03/V02-01.md).

### F-RCT-03-P2-01: Terminal error events (`Err(NoResponse)`, `Err(MaxIterations)`, intervention cancel/block) use `try_send` and can be silently dropped, leaving the consumer with no terminal signal

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/phases/finalize.rs:226` —
    `let _ = tx.try_send(Err(ReactError::Agent(Box::new(AgentError::NoResponse
    {…}))))`.
  - `echo-agent/src/agent/react/run/phases/finalize.rs:267` —
    `let _ = tx.try_send(Err(ReactError::Agent(Box::new(
    AgentError::MaxIterationsExceeded(…)))))`.
  - `echo-agent/src/agent/react/run/phases/think.rs:47` —
    `let _ = tx.try_send(Err(ReactError::Other("Agent execution cancelled
    by intervention at think")))`.
  - `echo-agent/src/agent/react/run/phases/think.rs:65` —
    `let _ = tx.try_send(Err(ReactError::Other(format!("Think blocked by
    intervention: {reason}"))))`.
  - `echo-agent/src/agent/react/run/stream_channel.rs:310` —
    `let _ = tx.try_send(Err(e))` (top-level `run_core_loop` error
    fallback).
- Reachability: every error-terminal path. T3 (NoResponse) and T4
  (MaxIterations) are the most likely to hit a full buffer because they
  fire after many iterations of accumulated undrained events (T4
  especially — `max_iterations` defaults to 100, each iteration emitting
  `LlmUsage` + think/tools events). The intervention arms (T6/T7) fire
  at think-start; lower probability but non-zero if prior iterations'
  events are undrained.
- Expected invariant: terminal events must be delivered. A stream that
  ends without a terminal leaves the consumer unable to distinguish
  failure from cancellation from success. The task question asks
  whether terminal events are "lossless" — they are not, on the error
  path.
- Observed behavior: when the 256-slot buffer is full at the moment of
  terminal `try_send`, the `Err(...)` item is dropped, the `let _`
  discards the `TrySendError::Full`, the function returns `Ok(())`, the
  spawned task exits, the channel's `tx` is dropped, and the consumer's
  `ReceiverStream` returns `None` — **the stream ends with no terminal
  item**. The consumer sees a clean stream end and cannot know the turn
  failed.
- Impact:
  - **Streaming consumer**: the UI shows a normal end-of-stream with no
    error indication. The user believes the turn succeeded (or was
    cancelled) when it actually hit `MaxIterationsExceeded` or
    `NoResponse`.
  - **Non-streaming collector** (`react_loop.rs:731-747`): `rx.recv()`
    returns `None`, the `while let Some(...)` exits, `answer` stays
    `String::new()`, and `run_react_loop` returns `Ok("")` — an empty
    success string for a turn that actually failed. This is a
    correctness defect: the caller cannot distinguish a genuine empty
    answer from a dropped `MaxIterationsExceeded`.
- Root cause: the `try_send` + `let _` pattern was copied from the
  intervention-error forwarding (which predates the terminal-event
  contract) into the `finalize_*` helpers. Terminal events should use
  the same `send().await` guarantee as `FinalAnswer`.
- Direction: change all five terminal-error sites to use
  `tx.send(Err(...)).await` (blocking, like `yield_final_event!`), with
  `.is_err()` → graceful return on `Closed`. Concretely, either add a
  `yield_final_err_event!` macro or inline the `send().await` at each
  site. This makes the terminal guarantee symmetric: both success and
  error terminals are delivered (or the loop exits gracefully if the
  receiver is gone).
- Regression validation: a test that fills the buffer to capacity (255
  items) and then triggers `finalize_no_response` must assert the
  `Err(NoResponse)` item is delivered to `rx.recv()` (not dropped).
  Today no such backpressure-on-terminal test exists.
- Validation reports: [V02](../validations/F-RCT-03/V02-01.md),
  [V03](../validations/F-RCT-03/V03-01.md).

### F-RCT-03-P2-02: ReactAgent never emits `AgentEvent::Cancelled` — the trait's cancellation terminal is bypassed by the override

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/agent/mod.rs:331-336` — `is_terminal()`
    returns true for `FinalAnswer(_) | Cancelled | Error { .. }`.
    `AgentEvent::Cancelled` is a documented terminal.
  - `echo-agent/echo-core/src/agent/mod.rs:560-569, 606-626` — the
    trait default `execute_stream_with_cancel` /
    `chat_stream_with_cancel` wrap with `cancel_aware_stream`.
  - `echo-agent/echo-core/src/agent/mod.rs:896-917` —
    `cancel_aware_stream` yields `AgentEvent::Cancelled` and terminates
    when the `CancellationToken` fires.
  - `echo-agent/src/agent/react/mod.rs:2821-2842` — ReactAgent
    **overrides** `chat_stream_with_cancel`, stores the token on the
    agent (`*self.cancel_token.lock().await = Some(cancel.clone())`),
    and delegates to `run_stream_entry` — **no `cancel_aware_stream`
    wrap**.
  - `echo-agent/src/agent/react/mod.rs:2844-2865` — same for
    `execute_stream_with_cancel`.
  - `echo-agent/src/agent/react/run/stream_channel.rs:1972-2038` —
    `test_run_stream_cancelled_mid_llm_call` asserts the stream
    terminates within 5s and that NO `FinalAnswer` is produced, but
    does NOT assert `AgentEvent::Cancelled` is produced (because it
    isn't).
  - `echo-agent/src/agent/react/run/react_loop.rs:737-739` — the
    non-streaming collector has an arm `Ok(AgentEvent::Cancelled) =>
    Ok("Cancelled.")` that is **dead for ReactAgent** (it can never
    fire).
- Reachability: every `chat_stream_with_cancel` /
  `execute_stream_with_cancel` caller on a ReactAgent (the production
  streaming + cancellation entry points).
- Expected invariant: the `Agent` trait contract documents
  `AgentEvent::Cancelled` as the cancellation terminal. Consumers
  (including the framework's own non-streaming collector) switch on it
  to detect cancellation. The concrete agent implementation should
  honour it.
- Observed behavior: when a ReactAgent stream is cancelled (via the
  stored `CancellationToken`), cancellation is handled inside
  `run_core_loop` — the LLM stream aborts (returning None) or the
  tools-phase `select!` observes the cancel and enters the 5s grace
  period. The loop then exits via `IterOutcome::NoResponse` →
  `finalize_no_response` (for mid-LLM cancel) or `IterOutcome::Abandoned`
  (for mid-tool cancel). Neither path emits `AgentEvent::Cancelled`:
  - mid-LLM cancel → `Err(NoResponse)` via `try_send` (which may itself
    be dropped per F-RCT-03-P2-01).
  - mid-tool cancel → `ToolBatchEnd` + loop returns `Ok(())`, stream
    ends with `None`.
- Impact:
  - **Streaming consumer**: cannot detect cancellation by inspecting
    the event stream. Must poll `cancel.is_cancelled()` separately (the
    workaround noted in F-CORE-01). A consumer that switches on
    `is_terminal()` misses the cancellation case entirely.
  - **Non-streaming consumer**: `run_react_loop` returns `Ok("")`
    (empty) or `Err(NoResponse)` for a cancelled turn, never the
    documented `Ok("Cancelled.")`. The dead arm at `react_loop.rs:737`
    is misleading code.
- Root cause: ReactAgent's override predates the `cancel_aware_stream`
  default (or was written to handle cancellation inside the loop for
  finer-grained control — the tools-phase grace period). The override
  forgot to wrap the output with `cancel_aware_stream` to preserve the
  trait's terminal contract.
- Direction: in `chat_stream_with_cancel` / `execute_stream_with_cancel`
  (and the `_with_invocation_context` / `_message_with_cancel`
  variants), wrap the `run_stream_entry` result with
  `cancel_aware_stream(stream, cancel.clone())` before returning —
  mirroring the trait default. The in-loop cancellation handling
  (grace period, LLM abort) stays as-is for responsiveness; the wrapper
  is the safety net that guarantees the `Cancelled` terminal is emitted
  if the in-loop handling races. Additionally, either remove the dead
  `Ok(AgentEvent::Cancelled)` arm in `react_loop.rs:737` or make it
  live by having the non-streaming path also wrap (less critical since
  the loop already exits).
- Regression validation: extend `test_run_stream_cancelled_mid_llm_call`
  to assert that the LAST event delivered to the consumer is
  `Ok(AgentEvent::Cancelled)` after `cancel.cancel()`. Today the test
  only asserts no `FinalAnswer`.
- Validation reports: [V03](../validations/F-RCT-03/V03-01.md),
  [V04](../validations/F-RCT-03/V04-01.md).

### F-RCT-03-P3-01: Stop-hook continuation can emit two `FinalAnswer` events on the streaming path; the non-streaming collector returns after the first

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/phases/finalize.rs:179` — `emit_final_text`
    emits `FinalAnswer(answer)` via `tx.send().await`.
  - `echo-agent/src/agent/react/run/phases/finalize.rs:190-200` — if the
    Stop hook returns `continue_reason` and `!state.stop_hook_continued`,
    sets the flag and returns `ControlFlow::Continue(())`.
  - `echo-agent/src/agent/react/run/stream_channel.rs:694` — the driver
    matches `ControlFlow::Continue(()) => continue`, looping again. The
    next iteration may produce another `emit_final_text` call with a
    second `FinalAnswer`.
  - `echo-agent/src/agent/react/run/phases/finalize.rs:199` — the
    one-shot `state.stop_hook_continued = true` bounds this to at most
    one continuation (at most two `FinalAnswer` events per stream).
  - `echo-agent/src/agent/react/run/react_loop.rs:731-735` — the
    non-streaming collector `break`s on the first `FinalAnswer` and
    returns it; the second is never collected.
- Reachability: requires a configured Stop hook
  (`HookEvent::SessionEnd`-adjacent `for_stop` context at
  `finalize.rs:88-93`) that returns a `continue_reason`. Uncommon in
  practice, but the code path is live and reachable.
- Expected invariant: "exactly one terminal per stream" (the task
  question). A stream with two `FinalAnswer` events violates the
  single-terminal expectation; the two paths (streaming vs
  non-streaming) return different answers.
- Observed behavior: streaming consumer sees
  `[…, FinalAnswer("v1"), …, FinalAnswer("v2")]`; non-streaming
  collector returns `Ok("v1")`. The two API contracts disagree on what
  the "final" answer is.
- Impact: low (narrow configuration required), but a genuine semantic
  divergence. A consumer that treats the first `FinalAnswer` as
  terminal and stops draining will miss the second; a consumer that
  waits for stream end will see two and must pick the last.
- Root cause: the Stop-hook continuation was designed for the
  non-streaming collector's "break on first FinalAnswer" semantics,
  where the continuation lets the loop run again and the *next*
  FinalAnswer is the one returned (because the first break already
  captured v1). Actually the collector breaks on the FIRST, so the
  continuation's second FinalAnswer is lost in non-streaming — the
  design intent is unclear. In streaming, both are visible.
- Direction: decide which answer is authoritative. If the
  continuation's second answer should win (the hook asked to continue,
  so the post-continue answer is the "real" one), the non-streaming
  collector should NOT break on the first `FinalAnswer` when
  `stop_hook_continued` is possible — or the continuation should
  suppress the first emission. If the first should win, the streaming
  path should not emit a second. Document the chosen semantics.
  Simplest fix: emit the continuation's `FinalAnswer` and suppress the
  pre-continue one (move the `send` to after the hook decision).
- Regression validation: a test with a Stop hook returning
  `continue_reason` that drives both streaming and non-streaming paths
  and asserts they return the SAME final answer.
- Validation reports: [V03](../validations/F-RCT-03/V03-01.md),
  [V04](../validations/F-RCT-03/V04-01.md).

### F-RCT-03-P3-02: Content `Token` events are batched (not per-chunk) on the full-ReAct path but per-chunk on the DirectAnswer path — inconsistent streaming granularity

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/phases/think.rs:121` — `run_think`
    calls `process_stream_chunk(…, /*emit_content_tokens=*/ false, …)`.
  - `echo-agent/src/agent/react/run/processor.rs:51-53` — when
    `emit_content_tokens` is false, per-chunk `Token(content)` events
    are NOT emitted; content is only accumulated into `content_buffer`.
  - `echo-agent/src/agent/react/run/phases/think.rs:241-245` — after
    the LLM stream ends, the full `content_buffer` is flushed as a
    single `Token` event.
  - `echo-agent/src/agent/react/run/stream_channel.rs:405-414` —
    `direct_answer_stream` emits per-chunk `Token(content)` via
    `tx.send().await` as each chunk arrives.
- Reachability: every streaming turn. Full-ReAct turns (the common
  case for tool-using agents) batch content into one post-stream
  `Token`; DirectAnswer turns stream per-chunk.
- Expected invariant: streaming granularity should be consistent
  across sub-paths, or the inconsistency should be documented.
- Observed behavior: a UI consumer sees smooth per-token typing for
  DirectAnswer turns and a single burst for full-ReAct turns. The UX
  is inconsistent.
- Impact: low (cosmetic/UX). No correctness defect — the final
  `FinalAnswer` carries the full content either way. But the streaming
  API's value proposition (live typing) is partially defeated for the
  main ReAct path.
- Root cause: the `emit_content_tokens=false` default was likely set
  to avoid event spam when the full content is also needed for context
  push and verification. The DirectAnswer path was written separately
  with per-chunk emission.
- Direction: either (a) set `emit_content_tokens=true` in `run_think`
  so full-ReAct also streams per-chunk content (preferred for UX
  consistency), or (b) document that full-ReAct batches content tokens
  and DirectAnswer streams them, so consumers know to expect the
  difference. Option (a) interacts with F-RCT-03-P1-01 (more events =
  more backpressure) — should be fixed together.
- Regression validation: a test that drives `run_think` with a
  multi-chunk content response and asserts multiple `Token` events are
  emitted (one per chunk), matching `direct_answer_stream`'s behaviour.
- Validation reports: [V01](../validations/F-RCT-03/V01-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Streaming event flow: deltas → ChatChunks → AgentEvents → EventEnvelopes (single pipeline, single production sites) | yes | passed | [V01-01](../validations/F-RCT-03/V01-01.md) |
| V02 | Channel capacity, backpressure, drop policy (lossless criterion) | yes | failed | [V02-01](../validations/F-RCT-03/V02-01.md) |
| V03 | Terminal event guarantee (exactly one, droppable error terminals, no Cancelled) | yes | failed | [V03-01](../validations/F-RCT-03/V03-01.md) |
| V04 | Streaming vs non-streaming conformance (same tools/budgets/terminals) | yes | passed (with documented divergences) | [V04-01](../validations/F-RCT-03/V04-01.md) |
| V05 | Historical-document drift check | conditional | n/a | No prior F-RCT-03 report exists; historical-claim classification is inline in the Inputs section (three current docstrings, one load-bearing lossy-contract doc). |

Executed cargo commands (all exit 0):

```text
cd echo-agent
cargo test --lib -p echo_agent -- run_core_loop_text_only_yields_final_answer \
  run_core_loop_tool_call_cycle_completes run_core_loop_empty_llm_response_terminates_gracefully \
  stream_guard_block_yields_single_final_answer test_run_stream_cancelled_mid_llm_call \
  iteration_wind_down_is_injected_once cancellation_drains_running_tool_before_abandoning_turn
  (7 passed)
cargo test --lib -p echo_agent -- finalize_no_response_sends_error finalize_max_iterations_sends_error \
  streaming_direct_answer_routes_through_projection value_scoped_direct_answer_records_usage \
  react_stream_records_real_usage
  (6 passed)
cargo clippy -p echo_agent --lib --all-features --locked -- \
  -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::unreachable
  (clean, no warnings)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `stream_channel.rs:1-16` — "All streaming execution goes through this module" + convergence with non-streaming pre-flight | current | V01 confirms single streaming entry; V04 confirms pre-flight convergence. |
| `stream_channel.rs:12-16` — "Converged with the non-streaming path: `run_stream_channel` runs the same pre-flight checks as `prepare_react_context`" | current | V04 confirms guard + IntentRouter + memory recall parity. |
| `phases/mod.rs:1-4` — "`run_core_loop` is the single, unified core loop" | current | Inherited from F-RCT-02; V04 re-verifies for streaming. |
| `config.rs:111-112` — "When full, events are dropped with a warning" | current + load-bearing | V02 confirms the drop behaviour. This documented contract is the basis for F-RCT-03-P1-01's argument that the lossy policy violates the task's "lossless" criterion. |
| `think.rs:330-334` — "Adapt the trait's flattened ChatChunk back into ChatCompletionChunk… no information is lost" | current | V01 confirms the mapping is lossless (both originate from the same OpenAI stream). |
| `processor.rs:90-99` — "Strategy: 1. Parse as-is. 2. Repair trailing. 3. Give up (return Err)" | current | Inherited from F-RCT-02 V04; the streaming parse-repair path is unchanged. |
| `mod.rs:893-895` (echo-core) — "When `cancel` is triggered, the stream yields `AgentEvent::Cancelled` and terminates" | current for the trait default, stale for ReactAgent | V03 confirms ReactAgent overrides the default and does NOT emit `Cancelled`. Feeds F-RCT-03-P2-02. |
| `mod.rs:330-336` (echo-core) — `is_terminal()` includes `Cancelled` | current as type contract, misleading for ReactAgent | V03 confirms ReactAgent never produces a `Cancelled` event, so the arm is dead for the concrete agent. |

## Coverage And Uncertainty

Inspected in full: `stream_channel.rs` (all 2161 lines — entry,
`direct_answer_stream`, `run_core_loop`, and the test module sampled for
the cited tests), `stream_macros.rs`, `processor.rs`, all six
`phases/*` files (for the event-production sites), `react_loop.rs`
(non-streaming collector for V04), the streaming trait impls in
`mod.rs:1833-1870, 2767-2930`, `echo-core/src/agent/mod.rs:140-340,
555-569, 896-917`, `echo-core/src/agent/event_envelope.rs:64-194`
(consumer-side wrapping, per F-CORE-01), and `config.rs` for
`stream_buffer_size`.

Not inspected (out of scope or deferred):

- The 13-stage `pipeline.rs` internals (per-tool middleware) — F-RCT-04.
  This task confirms `execute_tool_with_policy` feeds tool events into
  the same `tx` via `ToolStream`, but the pipeline stages themselves
  are not audited.
- `ContextManager` compression internals — F-MEM-01 / F-CMP-01. This
  task confirms `ContextCompressed` is emitted from `run_compact` but
  does not audit the compression algorithm.
- The application-layer envelope consumers (`chat_driver`,
  `task_runtime/executor`, `a2a/server`) — application tasks. This task
  confirms the framework returns raw `Result<AgentEvent>` and the
  consumer wraps with `envelope_event_stream`; the consumer-side
  projection correctness is out of scope.
- The `a2a/server.rs` streaming path — separate transport task; this
  review focuses on the `ReactAgent` streaming contract.
- Steer-during-LLM (sampled in
  `steer_during_llm_call_continues_same_turn_with_new_input:2089`) —
  adjacent to F-RCT-05 (snapshot/resume).

Environmental constraints:

- All cargo commands ran against the existing incremental build cache
  (`target/`); no `cargo clean` was needed (disk pressure well below
  threshold). Final worktree state is clean (`git status` clean, commit
  `9b0e0fa`).
- The feature matrix was not re-run; only the default feature set was
  exercised (the streaming path is feature-independent — no
  `#[cfg(...)]` gates in `stream_channel.rs` outside the test module).

Uncertain claims:

- The exact production triggerability of F-RCT-03-P1-01 / P2-01 under
  real load. The static analysis proves the drop paths exist and are
  reachable; whether a real consumer (the EKO GUI, a TUI client) hits
  them depends on consumer-side rendering speed and LLM token rate. The
  finding is preventive for current usage but becomes likely under
  sustained high-token-rate reasoning models (Qwen3/DeepSeek thinking).
- Whether any third-party `echo-agent` consumer relies on
  `AgentEvent::Cancelled` being emitted by the framework stream. The
  trait default emits it; ReactAgent does not. A consumer written
  against the default would break on ReactAgent. The framework layering
  rule retains the trait contract; the fix in F-RCT-03-P2-02 restores
  conformance.

## Handoff

Conclusions downstream tasks may rely on:

1. **Single streaming pipeline confirmed.** `run_stream_channel` is the
   only streaming entry; `process_stream_chunk` is the only chunk→event
   converter; the framework returns raw `BoxStream<Result<AgentEvent>>`
   and the consumer applies `envelope_event_stream`. Any task reasoning
   about streaming event production can treat this path as
   authoritative.
2. **Loop-body equivalence with non-streaming holds** (inherits
   F-RCT-02). Both entries spawn the same `run_core_loop`; same tools,
   same budgets, same terminal partition. Divergences are at the
   collection/wrapping layer only (V04).
3. **Streaming is bounded (256) but NOT lossless.** Intermediate events
   (`Token`, `ToolCall`, `ToolResult`, `ThinkStart/End`, `LlmUsage`) are
   silently dropped under backpressure. Success terminals
   (`FinalAnswer`) are lossless (blocking send). Error terminals
   (`Err(NoResponse)` etc.) are droppable. Any task that relies on
   complete event sequences must account for drops under load until
   F-RCT-03-P1-01 / P2-01 are fixed.
4. **`AgentEvent::Cancelled` is not emitted by ReactAgent.** Any
   consumer that switches on `is_terminal()` to detect cancellation
   misses the ReactAgent case. Use `CancellationToken::is_cancelled()`
   polling as a workaround (per F-CORE-01) until F-RCT-03-P2-02 is fixed.
5. **Stop-hook continuation can emit two `FinalAnswer` events** on
   streaming; non-streaming returns the first. Narrow but real.

Reports they must read:

- This report (F-RCT-03) for the streaming-specific event-flow and
  terminal-guarantee findings.
- `tasks/F-RCT-02.md` for the shared loop-body invariants (single
  `run_core_loop`, 10-arm terminal partition, `max_iterations` + soft
  budgets as the only bounds).
- `tasks/F-CORE-01.md` for the `AgentEvent` / `EventEnvelope` /
  `is_terminal()` / `cancel_aware_stream` contracts that this task's
  terminal and cancellation findings build on.
- `validations/F-RCT-03/V01-01.md` through `V04-01.md` for per-claim
  evidence.

Conditions that make this report stale:

- Any change to `yield_event_or!`'s send API (e.g. switching to
  `send().await`) invalidates F-RCT-03-P1-01 and the V02 lossless
  analysis.
- Any change to the five terminal-error `try_send` sites invalidates
  F-RCT-03-P2-01 and V03.
- Wrapping `cancel_aware_stream` inside ReactAgent's
  `chat_stream_with_cancel` invalidates F-RCT-03-P2-02 and V03.
- Any change to `emit_text_final`'s Stop-hook continuation logic
  invalidates F-RCT-03-P3-01.
- Any change to `emit_content_tokens` in `run_think` invalidates
  F-RCT-03-P3-02 and V01's deviation note.
- A new sibling streaming entry (parallel to `run_stream_channel`)
  invalidates V01's single-pipeline claim.

Follow-up task IDs (no fixes implemented in this review):

- **F-RCT-04** (tool batch execution) — owns the 13-stage pipeline and
  the `execute_tool_with_policy` middleware. This task confirms tool
  events reach the channel; the pipeline internals are F-RCT-04.
- **F-RCT-05** (steer / interrupt / snapshot / resume) — owns the
  steer-during-LLM and cancellation-grace semantics sampled here.
- A **framework streaming-backpressure task** — should fix F-RCT-03-P1-01
  (make `yield_event_or!` blocking or add a backpressure macro) and
  F-RCT-03-P2-01 (make terminal-error sites use `send().await`). These
  two are independent of each other and of the cancellation fix.
- A **framework cancellation-conformance task** — should fix
  F-RCT-03-P2-02 (wrap `cancel_aware_stream` in ReactAgent's
  `chat_stream_with_cancel` / `execute_stream_with_cancel` overrides)
  and remove or make-live the dead `Ok(AgentEvent::Cancelled)` arm in
  `react_loop.rs:737`.
- A **Stop-hook semantics task** — should decide and document the
  authoritative `FinalAnswer` under continuation (F-RCT-03-P3-01) and
  fix the streaming/non-streaming divergence.
