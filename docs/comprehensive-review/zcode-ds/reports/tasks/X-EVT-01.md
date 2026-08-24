# X-EVT-01: Event lifecycle conformance

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline 9b0e0fa)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline b3b2e81)
> Worktree state: clean in both repositories (verified before and after every
> executable validation; no command regenerating `web-frontend/src/generated/*.ts`
> was executed)

## Question

Do framework events, EKO persistence, Rust surfaces, and TypeScript reducers
agree on identity, ordering, terminal status, cancel, and timeout?

**Answer: identity and ordering mostly yes at every layer; terminal status
partially no; cancel and timeout no.** Identity: one `EventEnvelope` contract
(deterministic `event_id`, monotonic `sequence`, per-invocation identity) is
produced by one envelope adapter and consumed by all four EKO surfaces;
subagent execution ids `{run_id}:{task_id}:{plan_revision}:{attempt}` are
consistent backend-to-frontend. Ordering: envelope sequences are contiguous
and truncated at the first terminal, GUI chat events arrive in producer
order on one channel. Terminal status: the one-terminal invariant holds only
at the envelope boundary and in the subagent store; the GUI wire emits two
contradictory terminals per error/cancel turn (canonical
A-CHAT-01-P1-01/A-SRF-03-P1-02) and the live tool ingest can regress a
terminal row (canonical A-FE-02-P2-01). Cancel: the framework's documented
`AgentEvent::Cancelled` terminal has zero production producers
(F-RCT-03-P1-02), so every surface guesses cancel from secondary signals and
subagents only classify it correctly when the token race is configured.
Timeout: the typed `AgentError::Timeout`/`ToolError::Timeout` classes are
collapsed by the envelope; the chat wire has no timeout terminal at all, and
the only `timed_out` status in the TS layer is reachable only through the
subagent race paths.

## Scope

Deep-read sources (production code at the reviewed commits):

- `echo-agent/echo-core/src/agent/mod.rs` (`AgentEvent` :143-310,
  `is_terminal` :331-336, `AgentPhase`),
  `echo-core/src/agent/event_envelope.rs` (full; `EventEnvelope`,
  `EventIdentity`, `stable_event_id` :64-84, `envelope_event_stream`
  :107-194, `validate_event_trajectory` :196-295), `echo-core/src/error.rs`
  (`AgentError` variant inventory).
- `echo-agent/src/agent/subagent/executor.rs` (`subagent_status_from_error`
  :138-147, envelope consumption :1182, loop terminal handling :1399-1402,
  pre-execution cancel :1684-1692, Sync race :1497-1535, Teammate race
  :915-970, dispatch Err mapping :630-650).
- `echo-agent/src/agent/react/run/stream_macros.rs` (:38-53 drop path),
  `run/phases/finalize.rs` (:179-201 Stop continuation — canonical anchors
  re-verified only), `src/agent/react/mod.rs` cancel-variant overrides
  (:2821-2933, re-verified only).
- `echo-agent/src/trace/mod.rs` (`RunStatus` :150-161, `JsonlRunStore`
  :672-793).
- `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs` (full:
  `drive_chat` :202-287, `ensure_task_mode_run` :289-336,
  `finalize_task_mode_run` :338-383, `drive_chat_inner` :425-569,
  `EventIdentity` :483-489, envelope wrap :538, loop :540-565,
  `ChannelChatSink` :575-591), `surface_contract.rs` (wire tests),
  `tool_execution.rs` (`ToolExecutionSummary`/`Status` :40-130).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs`
  (`subagent_execution_id` :174-180, `execute_task` :1843-2268 incl.
  token reclassification :2257-2258, `record_subagent_released` :2337-2343,
  main-agent stream terminal mapping :3440-3478, agent-driven run loop
  :3734-3789, cancel checks :3748/:3852), `task_runtime/types.rs`
  (RuntimeEventKind), `file_shadow.rs` (:92-145), `store.rs` replay reads
  (:1547-1583), `infra.rs` (:377 JsonlRunStore wiring).
- `echo-agent-cli/src/tauri/commands/chat.rs` (`ChatEvent` :30-112,
  `emit_chat_event` :114-143, `emit_execution_event` :153-183,
  `emit_tool_execution_summary` :185-208, `send_chat_message` :443-731,
  `cancel_chat` :807-830, `TauriChatSink` :1148-1417,
  `handle_tool_event` :1193-1331, `agent_event_to_chat_event` :1449-1572,
  `TauriExecutionProjector` :895-1114), `src/tauri/mod.rs` (bridge
  :353-768, subagent terminal mapping :573-645), `src/tauri/commands/
  conversations.rs` (:392-470), `src/tui/events.rs` (`TuiChatSink`
  :2021-2210, terminal loop :657-907), `src/cli/repl.rs` (sink :560-850),
  `src/cli/channels.rs` (aggregator :515-650).
- `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts` (full),
  `useTauriChat.ts` (full), `stores/chatStore.ts` (full),
  `stores/toolExecutionStore.ts` (full), `stores/subagentRunStore.ts`
  (full), `stores/taskRuntimeStore.ts` (replay/polling slices :104-301),
  `types/api.ts` (`ChatEvent` :125-177, `ChatRunStatus` :38-52).

## Out Of Scope

- ReAct loop internals, streaming producer losslessness, cancel-terminal
  emission — F-RCT-02/03 (consumed as dependency facts; anchors re-verified
  only).
- Tool batch execution, timeout ownership inside the pipeline — F-RCT-04.
- Chat driver lifecycle ownership, sink responsibility, `Interrupt` dead
  variant — A-CHAT-01.
- Frontend chat-surface defects (interrupt ghost, content wipe) — A-SRF-03
  (P1-01/P1-02 canonical).
- Frontend projection identity defects (tool ingest keying, revision-blind
  selector, dead components) — A-FE-02.
- DTO/type drift, generated-artifact state — A-FE-01 (canonical).
- Tauri command lifecycle, browser://event, duplicate tool producer —
  A-SRF-02 (canonical).
- TaskRuntime claims/recovery/terminal monotonicity under crash — A-TSK-04
  (read as dependency; steady-state replay facts reused).
- Usage/cache conformance — F-LLM-03 (P1-02 cross-referenced only).
- Dynamic GUI/TUI verification — Q-GUI-01/Q-E2E-01/Q-FLT-01.

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, surface parity, no-parallel-semantics,
  framework-vs-app layering, read-only review), shared `README.md`,
  `REPORTING.md`, `TASKS.md` (X-EVT-01 card), `zcode-ds/README.md`, report
  templates.
- Dependency task reports read (zcode-ds, all complete): `F-CORE-01`
  (envelope identities/errors), `F-RCT-03` (streaming event flow, terminal
  producer inventory, envelope adapter), `A-CHAT-01` (driver/sink contract,
  P1-01/P2-01/P2-02), `A-FE-01` (wire contracts, 19/19 ChatEvent coverage),
  `A-FE-02` (frontend projections, P2-01/P2-02). Canonical cross-reference
  reads: `A-SRF-03` (P1-01/P1-02), `A-TSK-04` (replay monotonicity,
  P1-01).
- Historical documents treated as hypotheses: none beyond the dependency
  reports' own classifications (re-verified at current code, see
  Historical Claim Status).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correct) | `EventEnvelope`/`EventIdentity`, `envelope_event_stream` (one adapter), `AgentEvent` terminal contract, `AgentError` typed error classes, `subagent_status_from_error`, trace `RunStore`. No movement recommended. |
| EKO product policy (application, correct) | `drive_chat`/`ChatDriverEvent`/`ChatSink`, GUI `TurnStatus` wrapper, `ChatEvent` wire enum, `ToolExecutionRepository`, TaskRuntime `events.jsonl`, the four surface sinks, all TS stores. |
| Adapter boundary (findings) | (a) The envelope is the single raw-stream adapter but it collapses typed cancel/timeout classes into a generic `Error` payload, and the GUI wire drops the envelope identity/sequence (X-EVT-01-P1-01/P1-02/P2-01); (b) the chat-turn persistence contract has no terminal field (X-EVT-01-P2-02); (c) `ChatEvent::Cancelled` is a dead wire variant (X-EVT-01-P3-01). |
| Duplicate search (V01-01 terms) | `EventEnvelope`/`envelope_event_stream`/`event_id`/`sequence`, `AgentEvent::Cancelled` producers, `ChatEvent` variants, `SubagentStatus`/`subagent_status_from_error`, `TimedOut`/`Timeout` terminals, `TurnStatus`, `run_status`, `finalizeAssistantMessage`/`markCancelled`, `STORED_SUBAGENT_EVENTS`, `ingest`, `loadByConversation`. Result: one envelope adapter, one driver, one GUI wire enum, one store per projection; no parallel implementation found. |

## Current Path

Verified data flow (V01-01/V02-01/V03-01/V04-01):

1. **Framework production**: ReactAgent streaming loop emits raw
   `Result<AgentEvent>` items; typed cancel (`Cancelled`) is never produced
   (F-RCT-03-P1-02); typed timeouts exist (`AgentError::Timeout`,
   `ToolFailure::Timeout`) but stream-item errors are untyped
   (`Error{source,message}`).
2. **Envelope adapter** (echo-core, one instance per invocation):
   `envelope_event_stream` (event_envelope.rs:107-194) assigns deterministic
   `event_id` + contiguous `sequence`, links tool events to their ToolCall
   via `parent_event_id`, truncates at the first terminal, converts every
   raw `Err` into `Error{"agent_stream": ...}` and fabricates an Error on
   terminal-less ends. Consumers: `drive_chat_inner` (chat_driver.rs:538),
   framework subagent executor (executor.rs:1182), task-runtime main-agent
   and agent-driven loops (executor.rs:3119-3130, 3734-3789).
3. **EKO Rust surfaces**: all four chat surfaces consume the envelope stream
   through `drive_chat` and the shared sink contract; terminal mapping
   differs per surface — GUI derives `TurnStatus` from the cancel token +
   `drive_chat` Result (chat.rs:690-696) and emits `RunStatus`+`Done`
   (chat.rs:1365-1387); TUI clears on FinalAnswer/Error (masking errors when
   its own token fired, events.rs:805-841); REPL prints; channel flushes on
   FinalAnswer/Cancelled and errors on Error.
4. **EKO persistence**: tool executions (durable journal, terminal statuses),
   TaskRuntime `events.jsonl` (per-run seq, append-locked) + plan snapshot,
   conversation messages (final content only, no terminal field), trace runs
   (JsonlRunStore). Replay paths rebuild subagent/tool/run facts
   monotonically (V03-01).
5. **TypeScript reducers**: chat events dispatch by variant
   (chatEventHandler.ts, 19/19); subagent events guarded monotonic
   (subagentRunStore.ts:450-460); tool live ingest unguarded (A-FE-02-P2-01);
   TaskRuntime polling append-only past `lastSeq` with generation guards.

## Findings

### X-EVT-01-P1-01: Cancel/timeout class is lost at the envelope boundary for subagent streams — the typed `subagent_status_from_error` mapping is bypassed by envelope normalization, so mid-stream cancelled/timed-out subagents surface as `failed` unless the token race fires first

- Priority: P1
- Confidence: high (static chain fully verified)
- Layer: framework (envelope + subagent executor) with adapter consequence
- Evidence:
  - `echo-agent/src/agent/subagent/executor.rs:138-147` —
    `subagent_status_from_error` maps `AgentError::Timeout -> TimedOut`,
    `AgentError::Interrupted | Cancelled -> Cancelled` (with dedicated
    tests :2450-2926).
  - `executor.rs:1182` — every subagent stream is wrapped by
    `envelope_event_stream`; `event_envelope.rs:134-140` converts every raw
    `Err` item (NoResponse, mid-stream LLM/tool errors) into an
    `Error{"agent_stream": ...}` payload; `executor.rs:1400-1402` maps any
    such payload to `Err(ReactError::Other(...))`; `subagent_status_from_error`
    on `ReactError::Other` returns `Failed` (executor.rs:143).
  - Correct classification exists only where the token race intercepts:
    Sync race (`executor.rs:1501-1535`, `biased` cancel/timeout arms),
    Teammate race (`:915-970`), pre-execution check (`:1684-1692`).
  - Consequence chain: `SubagentStatus::Failed` -> bridge emits event
    `status.as_str()` = `"failed"` (`src/tauri/mod.rs:604-614`) ->
    `subagentRunStore` terminal `failed` (subagentRunStore.ts:146-175);
    the durable `SubagentReleaseRecord.status` (executor.rs:2337-2343)
    persists `failed`, so replay reproduces the same misclassification
    (consistency preserved, truth lost).
  - `timeout_secs = 0` (config override) removes the Sync race entirely
    (`executor.rs:1535-1544`), leaving mid-execution cancel to the envelope
    path -> `Failed`.
- Reachability: every user-cancelled or provider-timed-out subagent whose
  cancellation arrives as a silent stream end rather than through the race
  arm — i.e., any mode/config where the token race does not fire first, and
  every mid-stream timeout (provider/LLM error) that is not caught by the
  per-subagent race timer.
- Expected invariant: cancel and timeout are distinguishable from failure
  end-to-end; the typed mapping the framework exposes must be reachable for
  the events it claims to classify (the terminal vocabulary of the wire —
  `cancelled`/`timed_out` — must correspond to real producer capability).
- Observed behavior: a cancelled or mid-stream-timed-out subagent is
  persisted, emitted and rendered as `failed`; the `cancelled`/`timed_out`
  wire statuses are reachable only via race-detected paths or stream-setup
  errors; `subagent_status_from_error` is partially dead (its typed inputs
  cannot arrive from the envelope path).
- Impact: subagent cards lie about the outcome of user cancels and timeouts
  (surface behavior violating the product invariant "cancelled is
  distinguishable from failure"); timeout retry policy is unreachable; the
  typed framework API is misleading.
- Root cause: the envelope adapter normalizes typed stream errors into an
  untyped payload before the subagent loop can classify them; the typed
  mapping and the normalization were designed against different error
  channels (Result items vs stream items).
- Direction: at the subagent loop terminal, check the cancel token
  (`execution_cancel.is_cancelled()` after the loop, mirroring
  executor.rs:1684) before mapping an Error payload to `Failed`; or have
  the envelope preserve the typed class (e.g. carry `AgentError` classification
  in the `Error` payload); align with the F-RCT-03-P1-02 fix (emit
  `AgentEvent::Cancelled` at cancel terminal points) so the subagent loop's
  `AgentEvent::Cancelled` arm (:1399) becomes reachable.
- Regression validation: framework fixture — subagent cancelled mid-LLM with
  `timeout_secs = 0` -> `SubagentStatus::Cancelled` and wire event
  `"cancelled"`; subagent mid-stream LLM timeout -> `TimedOut`/`"timed_out"`;
  EKO fixture — `record_subagent_released` status matches the emitted
  terminal.
- Validation reports: [V01-01](../validations/X-EVT-01/V01-01.md),
  [V02-01](../validations/X-EVT-01/V02-01.md),
  [V04-01](../validations/X-EVT-01/V04-01.md)

### X-EVT-01-P1-02: Chat-turn timeouts have no typed terminal at any layer below the producer — the envelope collapses `AgentError::Timeout`/`ToolError::Timeout`, `ChatEvent` has no timeout variant, and the TS reducer ends timed-out turns at `'completed'`

- Priority: P1
- Confidence: high (static; the 'completed' endpoint is the verified
  A-SRF-03-P1-02 chain)
- Layer: adapter (envelope) with application wire/reducer consequence
- Evidence:
  - Typed classes exist: `AgentError::Timeout` (`echo-core/src/error.rs`),
    `ToolFailure::Timeout`/`ToolError::Timeout` (echo-core tools); F-CORE-01
    V01 recorded them as the typed timeout vocabulary.
  - Envelope collapse: `event_envelope.rs:134-140` turns every raw `Err`
    (LLM request timeout, provider mid-stream error, batch timeout) into
    `Error{"agent_stream": ...}` — the class is gone before any consumer.
  - Batch timeout ends the turn without a typed terminal at all
    (F-RCT-04-P1-02 canonical).
  - Wire: `ChatEvent` (:30-112) has no timeout variant; `ChatRunStatus`
    (`api.ts:38-52`) has no `timed_out` for chat; the only `timed_out`
    terminal on the GUI wire is the subagent status emitted by the bridge
    (mod.rs:604-614, `status.as_str()`) and consumed by the subagent store
    (subagentRunStore.ts:146-175) — reachable only via race-detected
    subagent timeouts (X-EVT-01-P1-01).
  - Reducer endpoint: `error` -> `setRunStatus('failed')` +
    `finalizeAssistantMessage` -> `'completed'` (chatEventHandler.ts:140-150,
    chatStore.ts:354-362 — canonical A-SRF-03-P1-02); tool-level timeouts
    survive only in the tool detail manifest (`failure` field,
    tool_execution.rs:106-112), not in the turn terminal.
- Reachability: every chat turn whose LLM request times out, whose tool
  batch times out, or whose provider fails mid-stream.
- Expected invariant: timeout is a typed, distinguishable terminal
  (the framework models it as such; users must be able to tell a timeout
  from a failure, and the turn must not end `'completed'`).
- Observed behavior: a timed-out turn arrives as a generic
  `ChatEvent::error` with message text, is classified `failed` then
  `completed` in the store, and the frontend cannot represent the timeout
  class at all.
- Impact: users see success ("就绪"/completed) for timed-out turns; no
  timeout-aware retry UX is possible; the timeout terminal class — present
  in the framework's typed error model and in the subagent wire — is absent
  from the chat product surface.
- Root cause: the envelope normalizes typed stream errors to text before
  the product layer; the chat wire was defined against the normalized
  vocabulary and never gained a timeout variant.
- Direction: forward a typed timeout terminal (e.g. a `Timeout` `AgentEvent`
  terminal or a structured `Error` payload carrying the class) through the
  envelope; add `ChatEvent::timeout` (or reuse `error` with a typed
  `code`) and a reducer arm ending `'failed'` (never `'completed'`);
  align with the F-RCT-03-P1-01 fix so the specific terminal survives
  backpressure.
- Regression validation: driver fixture — LLM-timeout and batch-timeout
  turns through `drive_chat` yield a typed timeout terminal; frontend
  fixture — timeout event ends `runStatus 'failed'` with partial content
  kept; store fixture — no timeout turn ends `'completed'`.
- Validation reports: [V01-01](../validations/X-EVT-01/V01-01.md),
  [V02-01](../validations/X-EVT-01/V02-01.md),
  [V04-01](../validations/X-EVT-01/V04-01.md)

### X-EVT-01-P2-01: The GUI wire drops the envelope identity and sequence — `ChatEvent` carries only `message_key`/`conversation_id`, so the TS layer cannot detect dropped events, verify ordering, or correlate chat events with the execution channel

- Priority: P2
- Confidence: high
- Layer: adapter (Tauri event bridge)
- Evidence: `emit_chat_event` injects only `message_key` +
  `conversation_id` (`chat.rs:128-140`); `ChatEvent` itself has no identity
  fields (:30-112); the envelope carries `event_id`/`sequence`/`run_id`/
  `turn_id`/`execution_id` (event_envelope.rs:26-38) but none survive to
  the wire. The producer can drop events under backpressure with only a
  `warn!` (stream_macros.rs:38-53 — F-RCT-03-P1-01 canonical); the frontend
  `isCurrentRunEvent` filter (useTauriChat.ts:50-58) and the reducers have
  no sequence to detect the gap. `execution://event` is a second,
  independently ordered channel (chat.rs:153-183); cross-channel ordering
  (tool_batch_start/end on chat channel vs kind=tool summaries on execution
  channel) is unverifiable.
- Reachability: every GUI chat turn; the loss-detection gap materializes
  whenever the producer drops events (slow sink/backpressure) or the two
  channels interleave.
- Expected invariant: a lossless/ordered contract is either guaranteed by
  the producer or detectable by the consumer; the framework designed
  `event_id`+`sequence` precisely so persistence consumers can reject
  duplicates and detect gaps (event_envelope.rs:119-122 doc).
- Observed behavior: the only consumer that can use the envelope identity —
  none — because no surface forwards it; the TS layer cannot distinguish a
  dropped token burst from a turn that produced none.
- Impact: F-RCT-03-P1-01's drops are invisible to the frontend (missing
  tokens/results without a signal); ordering conformance between the two
  channels is untestable at the TS layer; the framework's idempotency
  design is unused by the only live consumer chain.
- Root cause: the wire contract was defined from the sink's rendering needs
  (message_key/conversation_id) rather than from the envelope contract;
  identity/sequence forwarding was never added.
- Direction: include `sequence` (and `event_id`/`turn_id`) in the
  `ChatEvent` payload (or a per-turn `event_ack`), and let
  `chatEventHandler` detect non-contiguous sequences and log/flag gaps;
  document the two-channel ordering contract (or merge channels).
- Regression validation: fixture emitting a stream with a dropped sequence
  -> reducer flags the gap; fixture asserting `sequence` is contiguous
  across a normal turn.
- Validation reports: [V01-01](../validations/X-EVT-01/V01-01.md),
  [V02-01](../validations/X-EVT-01/V02-01.md)

### X-EVT-01-P2-02: Chat-turn terminal status is absent from EKO persistence — conversations persist messages only, so a failed/cancelled turn is indistinguishable from a completed one after reload, while TaskRuntime/tool/subagent terminals persist and replay truthfully

- Priority: P2
- Confidence: high (static; replay of the other three families verified)
- Layer: application (persistence contract)
- Evidence: `save_conversation`/`get_conversation` persist
  `SavedMessage[]` only (`conversations.rs:392-470`); the frontend restore
  resets `runStatus: 'idle'` (`chatStore.ts:449-462`); `ChatMessage` has no
  status field. Contrast: TaskRuntime runs persist `TaskRunStatus`
  (transitioned via `finalize_task_mode_run` chat_driver.rs:338-383 and the
  DAG executor) and replay from `events.jsonl` (taskRuntimeStore.ts:219-283);
  tool terminals persist in the journal and replay with a status-rank guard
  (toolExecutionStore.ts:50-86); subagent terminals persist via
  `record_subagent_released` (executor.rs:2337-2343) and replay
  monotonically (subagentRunStore.ts:450-460). On error/cancel the persisted
  content is the `[Error] ...`-replaced body (canonical A-SRF-03-P1-02), so
  the reloaded conversation is also content-corrupted.
- Reachability: every chat-mode conversation opened after a reload/restart.
- Expected invariant: restart continuity — a turn's outcome survives in the
  durable record where the product shows a terminal (surface parity and
  X-STA-01 scope; "界面状态不是唯一事实源").
- Observed behavior: the chat-turn terminal exists transiently in the store
  (`runStatus`), is never written to the conversation record, and is reset
  to `idle` on load; TaskRuntime-backed turns restore their full terminal
  while plain chat turns restore nothing.
- Impact: after reload, users cannot tell a cancelled/failed turn from a
  completed one on the flagship surface; the only hint is the corrupted
  content; persistence parity between chat turns and TaskRuntime/tool/
  subagent facts is broken.
- Root cause: the conversation persistence schema was defined around message
  content before per-turn outcome tracking existed; the terminal was never
  added when `runStatus`/`TurnStatus` were introduced.
- Direction: persist a per-turn terminal (e.g. a `status` field on the
  assistant message or a turn record keyed by message_key) and restore it in
  `replaceMessages`/`loadConversation`; combine with the A-SRF-03-P1-02
  content fix so the persisted body matches the outcome.
- Regression validation: fixture — save a failed turn (status + partial
  content), reload, assert restored status `failed` and content preserved;
  conversation round-trip test in A-STATE-01/Q-STA-01 style.
- Validation reports: [V01-01](../validations/X-EVT-01/V01-01.md),
  [V03-01](../validations/X-EVT-01/V03-01.md)

### X-EVT-01-P3-01: `ChatEvent::Cancelled` is a dead wire variant — implemented by the TS reducer and all four surfaces, produced by none, because the framework never emits the `AgentEvent::Cancelled` terminal

- Priority: P3
- Confidence: high
- Layer: adapter (wire contract) / framework (producer, canonical F-RCT-03-P1-02)
- Evidence: `ChatEvent::Cancelled` (:66-67) maps from `AgentEvent::Cancelled`
  (`chat.rs:1562`); the reducer arm exists (chatEventHandler.ts:151-158);
  TUI (events.rs:661-675), REPL (repl.rs:627, 813), channel (channels.rs:
  567-571) all implement the terminal; production producers of
  `AgentEvent::Cancelled` on the ReactAgent path: none (F-RCT-03-P1-02
  canonical; the only producers are the default `cancel_aware_stream`
  wrapper, overridden by ReactAgent, and a test mock at
  `executor.rs:2013`).
- Reachability: none on the main path; the reducer state `'cancelled'` is
  reached only via `markCancelled` (useTauriChat.ts:316, chatStore.ts:
  399-416), never via the wire.
- Expected invariant: wire variants correspond to producer capability
  (exhaustive in both directions); "cancelled" reaches the UI through the
  typed terminal, not through local button-side state.
- Observed behavior: the wire variant is dead; the frontend's only typed
  cancel path is local, and the wire still carries the fabricated error on
  cancel (A-CHAT-01-P1-01 chain).
- Impact: misleading contract (a developer trusting the TS type will expect
  `cancelled` events); the cancel UX depends on the button-side `markCancelled`
  and cannot be driven by the backend.
- Root cause: the wire enum was completed before the producer shipped; the
  F-RCT-03-P1-02 fix (emit `Cancelled`) would make it live.
- Direction: as part of the F-RCT-03-P1-02 fix, verify `ChatEvent::Cancelled`
  becomes reachable; otherwise delete the variant and the four consumer arms
  and re-document cancel as button-side only.
- Regression validation: after the fix — a cancelled GUI turn delivers
  exactly one `ChatEvent::Cancelled`; store status `cancelled` with no
  `[Error]` body (same fixture as A-SRF-03-P1-02's regression).
- Validation reports: [V01-01](../validations/X-EVT-01/V01-01.md),
  [V02-01](../validations/X-EVT-01/V02-01.md),
  [V04-01](../validations/X-EVT-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Producer-to-all-consumer matrix (every AgentEvent variant: framework -> EKO persistence -> Rust surface -> TS reducer) | yes | passed (20/20 traced; gaps -> findings) | [V01-01](../validations/X-EVT-01/V01-01.md) |
| V02 | Variant exhaustiveness (TS types/reducers cover all wire variants; dead variants flagged) | yes | passed (19/19 ChatEvent; 8/8 subagent kinds; dead `Cancelled`/`timed_out` reachability recorded) | [V02-01](../validations/X-EVT-01/V02-01.md) |
| V03 | Recorded event replay (replay through EKO persistence -> terminal state consistent) | yes | passed (TaskRuntime/tool/subagent consistent; chat-turn terminal absent -> P2-02) | [V03-01](../validations/X-EVT-01/V03-01.md) |
| V04 | Duplicate/out-of-order terminal conformance (9 scenarios across envelope/sinks/reducers) | yes | passed (invariant holds at envelope + subagent store; fails at GUI wire, live tool ingest, class preservation) | [V04-01](../validations/X-EVT-01/V04-01.md) |
| V05 | Cross-check with existing findings (canonical IDs) | yes | passed (17 canonical IDs re-verified current) | [V05-01](../validations/X-EVT-01/V05-01.md) |
| V04-support | `npx vitest run src/stores/{subagentRunStore,toolExecutionStore,taskRuntimeStore}.test.ts` | conditional | passed (exit 0, 23 tests) | [V03-01](../validations/X-EVT-01/V03-01.md), [V04-01](../validations/X-EVT-01/V04-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending. No command regenerating
`web-frontend/src/generated/*.ts` was executed; worktree verified clean
before and after.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-CORE-01 handoff: "envelope invariants (V04), identity flow (V02), two P3 findings; X-EVT-01 should decide the parent_event_id contract and agent-level Error classification" | current | parent_event_id still None everywhere (V05-01); envelope terminal truncation/fabrication verified (V01-01/V04-01) |
| F-RCT-03 handoff: "one streaming producer + one loop + one envelope adapter; one-terminal invariant holds only at the envelope boundary; X-EVT-01 should reconcile the two cancel vocabularies (main loop never emits Cancelled; subagent executor does) and envelope terminal normalization with the frontend contract" | current | V01-01 matrix rows 16/19/20; X-EVT-01-P1-01/P1-02/P3-01 implement this reconciliation |
| A-CHAT-01 handoff: "X-EVT-01 should include the GUI TurnStatus-vs-agent-terminal contradiction and the dead Interrupt variant in its terminal conformance matrix" | current | TurnStatus contradiction re-verified (chat.rs:690-696, V04-01 scenarios 2/7); Interrupt variant cross-referenced (A-CHAT-01-P2-01, out of scope) |
| A-FE-01 handoff: "event-conformance matrix incl. browser://event" | current | browser://event cross-referenced (A-SRF-02-P1-01, V05-01); not re-filed |
| A-FE-02 handoff: "X-EVT-01 (event lifecycle conformance: duplicate/out-of-order terminal across all consumers)" | current | V04-01 scenarios 3/4/5 implement this |

## Coverage And Uncertainty

- All conclusions are static traces plus one frontend store-test run; no GUI
  process was launched and no real cancel/timeout/backpressure scenario was
  executed dynamically (Q-E2E-01, Q-FLT-01, Q-GUI-01 own those).
- The framework stream internals (think.rs/tools.rs/finalize.rs) were
  re-verified only at the anchors F-RCT-03/F-RCT-04 filed; their full bodies
  are those tasks' scope.
- The exact EKO subagent dispatch mode for inline (non-TaskRuntime)
  subagents was traced to `delegate_to_agent_with_prompt_payload`
  (executor.rs:2810-2855) with the framework Sync/Fork races; the framework
  dispatch internals beyond the race anchors are F-SUB-02 scope.
- Whether `timeout_secs = 0` is reachable from EKO config (subagent
  definitions with explicit 0 or a config override) was not confirmed
  end-to-end; the code path exists and is the residual cancel-class-loss
  trigger.
- The frontend "queued chat" interaction with `done` (queue advances only on
  `done`, useTauriChat.ts:69-71) and the interrupt strand (A-SRF-03-P1-01)
  were noted but not re-audited.
- F-LLM-03-P1-02 (usage loss) is referenced for the LlmUsage matrix row only;
  its full audit is F-LLM-03's.
- Runtime ordering between the two GUI channels (chat://event vs
  execution://event) was assessed statically; Tauri event delivery ordering
  across channels was not empirically tested.

## Handoff

- Downstream tasks may rely on: the 20-variant producer-to-consumer matrix
  (V01-01); 19/19 ChatEvent exhaustiveness with dead-variant list (V02-01);
  replay conformance for TaskRuntime/tool/subagent and the chat-turn
  persistence gap (V03-01); the 9-scenario duplicate/out-of-order matrix
  (V04-01); 17 canonical IDs re-verified current (V05-01).
- X-STA-01 should fold in X-EVT-01-P2-02 (persist/restore chat-turn
  terminal) and X-EVT-01-P2-01 (identity/sequence on the wire) for its
  identity-continuity matrix.
- S-RDM-01 roadmap: X-EVT-01-P1-01 + P1-02 + P3-01 are the EKO-side halves
  of the F-RCT-03-P1-02 fix (emit `AgentEvent::Cancelled` + preserve typed
  classes in the envelope); P2-01 (wire sequence) and P2-02 (persist turn
  terminal) are application-layer items.
- Q-E2E-01 scenarios: cancel a GUI turn and assert one truthful terminal
  (no fabricated error, status cancelled); a timed-out turn asserting a
  timeout-class terminal; reload a failed conversation asserting restored
  status.
- Q-FLT-01 scenarios: backpressure drop detection via wire sequence;
  cancel-mid-tool-batch and cancel-mid-LLM at the GUI boundary.
- Reports to read: this report + V01-01..V05-01; dependency reports
  F-CORE-01, F-RCT-03, A-CHAT-01, A-FE-01, A-FE-02; canonical A-SRF-03
  (P1-01/P1-02), A-TSK-04 (replay), A-SRF-02 (P2-01), F-RCT-04 (P1-02).
- Stale triggers: any change to `event_envelope.rs` normalization/identity,
  `AgentEvent` variants or `is_terminal`, the ReactAgent cancel variants,
  `subagent_status_from_error` or the subagent loop terminal handling,
  `ChatEvent` (chat.rs:30-112) or `emit_chat_event` payload construction,
  `chatEventHandler.ts`, `chatStore.ts` terminal paths,
  `conversations.rs` message schema, or the TaskRuntime event/ledger
  protocol invalidates the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): X-STA-01,
  S-RDM-01, Q-E2E-01, Q-FLT-01.
