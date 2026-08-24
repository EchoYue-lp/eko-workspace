# X-EVT-01: Event lifecycle conformance

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (read-only review)

## Question

Do framework events, EKO persistence, Rust surfaces, and TypeScript
reducers agree on identity, ordering, terminal status, cancel, and
timeout?

## Scope

Primary source paths and behaviors inspected:

- **Framework event contract** (read in full):
  - `echo-agent/echo-core/src/agent/mod.rs:140-446` — the 20-variant
    `AgentEvent` enum, `#[non_exhaustive]`, `is_terminal()`, `phase()`,
    `is_checkpoint()`.
  - `echo-agent/echo-core/src/agent/event_envelope.rs:1-295` —
    `EventEnvelope`, `EventIdentity`, `stable_event_id`,
    `envelope_event_stream` / `envelope_event_stream_after`,
    `validate_event_trajectory`, and the trajectory + terminal-
    normalization tests at `:297-514`.
  - `echo-agent/src/event_bus.rs` — `GLOBAL_EVENT_BUS` / `EventBus`
    (consumed as the F-CORE-01-P2-01 dead-infra conclusion; not
    re-audited).
- **EKO chat driver / persistence** (read in full):
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:1-591` —
    `ChatDriverEvent`, `ChatSink`, `drive_chat` lifecycle,
    `WebhookTurnObserver`, `ChannelChatSink`,
    `subagent_trace_sink_for`, `framework_trace_sink_for`,
    `ensure_task_mode_run` / `finalize_task_mode_run`
    `trace_sink(ExecEvent::run)` emission at `:323/361/377`.
- **Rust interactive surfaces** (read in full):
  - `echo-agent-cli/src/tauri/commands/chat.rs:1-220, 1140-1572` —
    `ChatEvent` (19-variant), `emit_chat_event`,
    `emit_execution_event`, `TauriChatSink` struct +
    `handle_tool_event` + `cancel_active_tools` + `ChatSink` impl +
    `agent_event_to_chat_event`, plus `send_chat:625-712` post-
    `drive_chat` `TurnStatus` emission.
  - `echo-agent-cli/src/tui/events.rs:2013-2229` — `TuiChatSink`
    variant mapping + `send_to_agent` (no post-`drive_chat`
    `TurnStatus`).
  - `echo-agent-cli/src/cli/channels.rs:515-650` —
    `aggregate_by_sentence` (channel renderer) terminal arms +
    silent-drop catch-all.
  - `echo-agent-cli/src/cli/repl.rs:495-545` (head) — REPL drive_chat
    caller (no post-`drive_chat` `TurnStatus`).
- **TypeScript reducers** (read in full):
  - `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts` — event
    listener registration, `isCurrentRunEvent` `message_key` filter,
    `execution://event` fan-out to subagent / tool / run stores.
  - `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts` —
    17-arm switch over `ChatEvent`, no `default`/`never` exhaustiveness
    guard.
  - `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.test.ts`
    — 4 tests (terminal-status projection, notice rendering,
    execution-path, LLM usage).
  - `echo-agent-cli/web-frontend/src/stores/chatStore.ts:350-416` —
    `finalizeAssistantMessage`, `setRunStatus` (no terminal lock),
    `markCancelled`.
  - `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:140-535`
    — `ingest` terminal lock, `taskRuntimeSubagentExecutionEvents`
    replay adapter.
  - `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.ts:50-254`
    — `mergeToolExecution` (hydrate monotone) vs live `ingest` (not
    monotone).
  - `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts:1-290`
    (head, generation-counter region).

## Out Of Scope

Deferred to named task IDs:

- The `ReactAgent`-never-emits-`Cancelled` root cause — **F-RCT-03**
  owns it (F-RCT-03-P2-02). This task consumes it as the upstream
  contract and analyses only the cross-surface downstream
  consequence.
- The streaming backpressure / dropped intermediate events defect
  (F-RCT-03-P1-01) and the droppable error-terminal defect
  (F-RCT-03-P2-01) — **F-RCT-03** owns both. This task references
  them only as the upstream "why the envelope sometimes synthesizes
  the terminal" context.
- The dead `GLOBAL_EVENT_BUS` / `EventBus` — **F-CORE-01** (P2-01)
  established it; this task re-uses that conclusion and does not
  re-grep.
- `TauriChatSink`'s misplaced tool-execution persistence authority —
  **A-CHAT-01** (P2-01) owns it. This task audits only the
  terminal/cancel behavior of that persistence, not its layering.
- The `ToolInfo` wire drift, vestigial Rust DTOs, orphan ts-rs files,
  and the IPC type-contract matrix — **A-FE-01** owns them.
- The subagent reducer identity model, the tool-execution live-
  ingest overwrite, and the acceptance/check projection gap —
  **A-FE-02** owns them. This task cross-references A-SRF-03-P2-01 /
  A-FE-02-P3-03 (live-ingest non-monotonicity) but does not re-audit.
- The backend task-runtime state machine and the `events.jsonl`
  persistence format — **A-TSK-04** owns them. This task consumes
  them as the durable replay source.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (multi-mode parity rule; framework-vs-
  application layering gate; "first prove no duplicate exists";
  UTF-8 / panic safety; the cleanup rule).
- `docs/comprehensive-review/REPORTING.md`,
  `docs/comprehensive-review/templates/task-report.md`,
  `docs/comprehensive-review/templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (X-EVT-01 card).

Dependency reports read (all complete):

- **F-CORE-01** — establishes `AgentEvent`, `EventEnvelope`,
  `EventIdentity`, `stable_event_id`, `is_terminal()`,
  `cancel_aware_stream`, and the dead `GLOBAL_EVENT_BUS` (P2-01).
  Load-bearing for V01 (single producer → single sink, no fan-out)
  and V04 (the type contract that distinguishes Cancelled/Error).
- **F-RCT-03** — establishes the streaming event flow, the lossy
  intermediate drop policy (P1-01), the droppable error terminals
  (P2-01), and the ReactAgent-never-emits-Cancelled defect (P2-02).
  Load-bearing for V04: cancel surfaces as synthesized Error, not
  Cancelled, on every ReactAgent-driven turn.
- **A-CHAT-01** — establishes `drive_chat` as the single chat-turn
  lifecycle owner, the one-terminal invariant delegated to
  `envelope_event_stream`, the TauriChatSink tool-execution
  persistence authority (P2-01), and the TUI/REPL/channels missing
  post-`drive_chat` `TurnStatus{cancelled}` semantic.
  Load-bearing for V04 (the per-surface cancel-recovery matrix).
- **A-FE-01** — establishes the IPC type-contract matrix and the
  manual-only DTO drift. Load-bearing for V02 (the TS reducer's
  ChatEvent union is manual, no compiler-checked link to the Rust
  ChatEvent enum).
- **A-FE-02** — establishes the per-attempt subagent identity
  isolation, the `subagentRunStore` terminal lock, and the
  `toolExecutionStore` live-ingest non-monotonicity (cross-reference
  to A-SRF-03-P2-01). Load-bearing for V03 (replay matrix).

Historical documents treated as hypotheses: none. The framework
docstrings on `AgentEvent`, `envelope_event_stream_after`, and
`cancel_aware_stream` are read at the cited line numbers and treated
as current code-level contracts (verified, not asserted).

## Layering Decision

This is a **cross-layer conformance** task. It does not propose new
framework code or new application code; it audits whether the
existing layers agree. Findings may point at either layer for the
fix; the recommendation prefers the framework fix when the broken
contract is the framework's (Cancelled emission), and the
application fix when the broken contract is the application's
(channel silent drop, chatEventHandler exhaustiveness).

| Classification | Required answer |
|---|---|
| Generic mechanism | `AgentEvent`, `EventEnvelope`, `EventIdentity`, `envelope_event_stream_after`, `validate_event_trajectory`, `is_terminal()` are correctly in `echo-core`. The framework owns the terminal-status contract; consumers must conform. |
| EKO product policy | `ChatSink`, `ChatDriverEvent`, `TauriChatSink` (with its tool-execution authority), `TuiChatSink`, `aggregate_by_sentence`, the post-`drive_chat` `TurnStatus` compensation, and the TS reducers are all EKO product policy. The cross-surface parity is an EKO concern (per AGENTS.md multi-mode parity rule), not a framework concern. |
| Adapter boundary | `drive_chat` is a thin adapter; the four `ChatSink` implementations are the surface-specific renderers; `chatEventHandler.ts` is the GUI's typed-event reducer. None of these should own terminal-monotonicity — that belongs to the framework envelope. The exception is the GUI's post-`drive_chat` `TurnStatus` emission, which is an application-level compensation for a framework-level defect (F-RCT-03-P2-02). |
| Duplicate search | Searched both repos for: `AgentEvent`, `ChatEvent`, `ChatDriverEvent`, `envelope_event_stream`, `Cancelled`, `is_terminal`, `validate_event_trajectory`, `setRunStatus`, `finalizeAssistantMessage`, `markCancelled`, `subagentRunStoreKey`, `mergeToolExecution`, `TurnStatus`. Result: single definition of each; no parallel event lifecycle. The four `ChatSink` implementations are the only consumers of `ChatDriverEvent::Agent`. The TS `ChatEvent` type is a manual shadow of the Rust `ChatEvent` enum (A-FE-01-P3-04: no contract test links them). |
| Migration deletion | No deletion proposed. The `validate_event_trajectory` "deletion" question is whether to wire it into production (recommended) or stop exporting it; both are behavior changes, not deletions per se. |

## Current Path

Verified cross-surface event lifecycle at commits `9b0e0fa` /
`b3b2e81`. The complete producer-to-consumer matrix and per-consumer
exhaustiveness classification are in V01-01 and V02-01; the replay
matrix is in V03-01; the four terminal scenarios are in V04-01. The
essential flow:

```text
Provider SSE stream
   ↓  ReactAgent run_stream_channel                   [F-RCT-03 V01]
   ↓  raw Result<AgentEvent> stream (256-slot mpsc)
   ↓  envelope_event_stream(raw, identity)             [chat_driver.rs:538]
   ↓  EventEnvelope stream — sequences from 1, breaks at first terminal,
      synthesizes Error{source:"agent_stream"} on stream-end-without-terminal
   ↓  drive_chat_inner loop forwards each envelope to sink.on_event
        ├─ WebhookTurnObserver.observe (read-only)    [chat_driver.rs:106-161]
        ├─ sink.on_event(ChatDriverEvent::Agent(env)) [chat_driver.rs:542-547]
        ↓
        Per-surface renderer:
        ├─ GUI: TauriChatSink
        │    ├─ handle_tool_event → ToolExecutionRepository (persistence)
        │    └─ agent_event_to_chat_event → emit "chat://event" → chatEventHandler
        │
        ├─ TUI: TuiChatSink → mpsc<local AgentEvent> → render loop
        │
        ├─ REPL/Channels: ChannelChatSink → mpsc → aggregate_by_sentence
        │
        └─ (none of the above for kind="subagent"/"tool"/"run" — those go via
            emit_execution_event → "execution://event" → useTauriChat listener
            → subagentRunStore / toolExecutionStore / taskRuntimeStore)

Post-drive_chat (GUI only):
   send_chat polls cancel.is_cancelled()                [chat.rs:704-712]
   ↓ emits ChatEvent::RunStatus{status:"cancelled"|"completed"|"failed"}
   ↓ chatEventHandler.run_status arm updates chatStore.runStatus
```

**Identity** (V01): the framework produces one stream per
`{conversation_id, run_id, turn_id, execution_id}`; the consumer that
wraps it is the consumer that drains it (no fan-out). Each subagent
fork forces a unique `execution_id` (`subagent/executor.rs:802-825`),
so concurrent forks have disjoint identity spaces. The frontend's
subagent reducer keys on `${runId}\u0000${subagentRunId}` where
`subagentRunId = {run_id}:{task_id}:{plan_revision}:{attempt}` —
attempt-scoped, so retries are structurally isolated (A-FE-02 V01).

**Ordering** (V01 + V03): within one stream, monotone sequences via
`envelope_event_stream_after`'s `saturating_add(1)`. Across streams,
no ordering invariant (each stream independent). The frontend's
`taskRuntimeStore` adds two generation counters
(`loadGeneration`, `refreshRequestGeneration`) and a `lastSeq` cursor
to suppress stale loads / refreshes during incremental polling.

**Terminal status** (V04): `is_terminal()` is `FinalAnswer(_) |
Cancelled | Error{..}`. The envelope wrapper breaks at the first
terminal (`event_envelope.rs:174-177`), so consumers receive at most
one terminal. On stream-end without terminal, the wrapper synthesizes
`Error{source:"agent_stream"}`. ReactAgent never emits `Cancelled`
(F-RCT-03-P2-02), so every cancel surfaces as the synthesized Error.
GUI recovers via post-`drive_chat` `TurnStatus` polling; TUI / REPL
/ channels do not.

**Cancel** (V04): see above. The `Cancelled` arm of every consumer's
match (V01 matrix) is dead code for ReactAgent-driven turns.

**Timeout** (V04): timeouts surface as `Error{source:"llm"|"llm_client",
message:"..."}` via the streaming error path, then normalized to the
envelope's terminal Error. No surface distinguishes timeout from
other LLM errors at the `AgentEvent` level (the typed detail lives in
`ReactError::Llm(...)` upstream, flattened to two strings by
`AgentEvent::Error`). Out of scope for this cross-surface review; the
framework error hierarchy is F-CORE-01-P3-01.

## Findings

### X-EVT-01-P2-01: Cancelled-vs-Error collapse — only GUI recovers; TUI / REPL / channels render cancel as error

- Priority: P2
- Confidence: high
- Layer: framework (root cause) + application (per-surface compensation asymmetry)
- Evidence:
  - `echo-agent/echo-core/src/agent/mod.rs:330-336` —
    `is_terminal()` includes both `FinalAnswer(_)` and `Cancelled`
    and `Error { .. }`. The framework type contract treats them as
    distinct terminals.
  - `echo-agent/echo-core/src/agent/event_envelope.rs:180-191` —
    `envelope_event_stream_after` synthesizes
    `AgentEvent::Error { source: "agent_stream", message: "agent
    stream ended without a terminal event" }` on stream-end without
    terminal. The wrapper cannot tell cancel from bug; always Error.
  - `echo-agent/src/agent/react/mod.rs:2821-2865` (per F-RCT-03-P2-02)
    — `ReactAgent` overrides `chat_stream_with_cancel` /
    `execute_stream_with_cancel` without wrapping
    `cancel_aware_stream`. Cancel never produces
    `AgentEvent::Cancelled`; the stream simply ends.
  - `echo-agent-cli/src/tauri/commands/chat.rs:704-712` — GUI
    compensates: `terminal_status = cancel.is_cancelled() ?
    "cancelled" : outcome.is_ok() ? "completed" : "failed"`, then
    `sink.on_event(TurnStatus{ status: terminal_status })`.
  - `echo-agent-cli/src/tui/events.rs:2222-2228` — TUI
    `send_to_agent` spawns `drive_chat` and returns; **no post-
    `drive_chat` `TurnStatus` emission**.
  - `echo-agent-cli/src/cli/repl.rs:541-545` — REPL
    `run_repl_turn` spawns `drive_chat` and returns; **no post-
    `drive_chat` `TurnStatus` emission**.
  - `echo-agent-cli/src/cli/channels.rs:262-270` — channels
    `handle` calls `drive_chat` directly; **no post-`drive_chat`
    `TurnStatus` emission**.
  - `echo-agent-cli/src/tui/events.rs:2071` — TUI local
    `AgentEvent::Cancelled` mapping (would render cancel correctly if
    framework ever emitted Cancelled). **Dead for ReactAgent.**
  - `echo-agent-cli/src/cli/channels.rs:566-569` — channel
    `aggregate_by_sentence` `AgentEvent::Cancelled => { flush + break }`
    (clean stop, no error). **Dead for ReactAgent.**
- Reachability: every cancelled chat turn on TUI / REPL / channels.
  The cancel button (`hooks/useTauriChat.ts:314-327`) on GUI calls
  `cancel_chat` which triggers `cancel.cancel_chat()` on the backend;
  the framework stream ends; `drive_chat` returns; GUI emits
  `TurnStatus{cancelled}`; TUI / REPL / channels emit nothing.
- Expected invariant: per AGENTS.md multi-mode parity rule ("TUI、
  GUI(以及 CLI/channel)必须功能对等"), cancel should be
  distinguishable from error on every surface that renders a status
  label. Per the framework type contract, `AgentEvent::Cancelled`
  and `AgentEvent::Error{..}` are distinct terminals and consumers
  should receive the right one.
- Observed behavior: GUI recovers cancel via post-`drive_chat`
  `CancellationToken::is_cancelled()` polling and a separate
  `TurnStatus{status:"cancelled"}` event through the
  `TauriChatSink::on_event` `TurnStatus` arm. TUI / REPL / channels
  receive the synthesized `Error{source:"agent_stream"}`, route it
  through their normal error path (`TuiChatSink` → local
  `AgentEvent::Error(message)`; `aggregate_by_sentence` →
  `Err(ReactError::Channel(...))`), and surface "agent stream ended
  without a terminal event" as an error to the user. The user-facing
  label on those three surfaces is "error", not "cancelled".
- Impact: 3 of 4 interactive surfaces mislabel cancel as error. The
  mislabel is semantic — tool execution is correctly cancelled
  (`TauriChatSink::handle_tool_event` calls `cancel_active_tools` on
  both `Cancelled | Error`, so tool history is correct on GUI), but
  the chat-level status the user sees is wrong on TUI / REPL /
  channels. For the IM channels case, the bot sends an actual error
  message to the chat ("agent stream error: agent stream ended
  without a terminal event"), which is misleading. Severity is P2
  (not P1) because no data is lost and no run is left in a bad
  state — the user can re-issue the turn — but the UX defect is
  systematic and contradicts the parity rule.
- Root cause: the framework's `ReactAgent` does not emit
  `AgentEvent::Cancelled` (F-RCT-03-P2-02). The GUI added a
  compensating post-`drive_chat` poll as a workaround; the other
  three surfaces did not. The asymmetry was never noticed because
  the GUI is the most-exercised surface.
- Direction: two paths, ideally both.
  1. **Framework fix (preferred, root cause)**: wrap
     `cancel_aware_stream` inside `ReactAgent`'s
     `chat_stream_with_cancel` / `execute_stream_with_cancel`
     overrides (`react/mod.rs:2821-2865`), as F-RCT-03-P2-02 already
     recommends. This makes `AgentEvent::Cancelled` flow through the
     envelope to every surface, making every surface's `Cancelled`
     arm live and the GUI's post-hoc compensation unnecessary.
  2. **Application fix (parity backstop, until the framework fix
     lands)**: add a post-`drive_chat` `TurnStatus{status:
     cancel.is_cancelled() ? "cancelled" : ...}` emission to
     `send_to_agent` (TUI), `run_repl_turn` (REPL), and `channels
     handle` — mirroring `chat.rs:704-712`. This requires those
     callers to forward the `CancellationToken` and poll it after
     `drive_chat` returns. The TUI would map `TurnStatus{cancelled}`
     to a local cancel signal (its `TuiChatSink::on_event` already
     has the arm at `events.rs:2037-2044`, but it is not currently
     used for `TurnStatus` because the TUI `ChatDriverEvent::TurnStatus`
     is forwarded by the sink itself — the application just needs to
     emit one).
- Regression validation:
  - Framework-side: extend `test_run_stream_cancelled_mid_llm_call`
    to assert the LAST event is `AgentEvent::Cancelled` after
    `cancel.cancel()` (per F-RCT-03-P2-02's recommendation).
  - Application-side: a per-surface test that drives a cancelled
    `drive_chat` and asserts the user-visible status is
    "cancelled", not "error". The GUI test would check
    `chatStore.runStatus === 'cancelled'`; the TUI test would check
    the rendered status line; the channel test would check the
    `OutboundMessage` text.
- Validation reports: [V04](../validations/X-EVT-01/V04-01.md),
  [V01](../validations/X-EVT-01/V01-01.md).

### X-EVT-01-P2-02: Channel renderer (aggregate_by_sentence) is the only consumer that silently drops unmatched AgentEvent variants

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/cli/channels.rs:625` — the channel
    renderer's match ends with `_ => {}`. This swallows:
    `ThinkStart`, `ThinkEnd`, `LlmUsage`, `ToolCall`, `ToolResult`,
    `ToolError`, `ToolStream`, `ToolBatchStart`, `ToolBatchEnd`,
    `ContextCompressed`.
  - Compare `echo-agent-cli/src/tui/events.rs:2204` —
    `other => AgentEvent::Notice(format!("Agent event: {other:?}"))`
    surfaces unknown variants as a visible Notice in the TUI.
  - Compare `echo-agent-cli/src/tauri/commands/chat.rs:1566-1570` —
    `other => ChatEvent::Notice { level:"info",
    code:"unknown_agent_event", message: format!("{other:?}") }`
    surfaces unknown variants as a typed Notice on `chat://event`.
- Reachability: every channel turn (REPL + IM channels) where any of
  the dropped variants fires. The most-consequential drops:
  - `ContextCompressed` — a channel user whose context was just
    compacted (potentially degrading answer quality by dropping
    earlier messages from the model's window) gets no indication.
    The final answer may be subtly worse and the user cannot tell
    why.
  - `LlmUsage` — silent. Token spend is invisible on the channel
    surface (no audit trail, no cost indicator).
  - `ToolCall` / `ToolResult` / `ToolError` / `ToolStream` — silent.
    A long-running turn with many tool calls produces no progress
    signal until the final answer; the channel user has no
    indication the agent is working. This is **partly intentional**
    (IM surfaces should not flood with tool chatter), but the
    silence also means a stuck tool looks identical to an idle agent.
- Expected invariant: unmatched variants should be at least logged
  (tracing) so a future variant addition or an investigation has an
  audit trail. The TUI and Tauri surfaces go further and surface
  unknown variants to the user; the channel surface does neither.
- Observed behavior: silent drop. No `tracing::debug!`, no
  `OutboundMessage`, no telemetry.
- Impact: medium. The dropped variants today are mostly dropped by
  design (tool chatter on IM is undesirable). The structural defect
  is the silence: a future `AgentEvent` variant (or a future caller
  that emits `ContextCompressed` to signal quality degradation)
  will be invisible on the channel surface, with no log to diagnose.
  The `ContextCompressed` case is the most material today — it
  hides a signal the user might want.
- Root cause: the channel renderer was written with the minimal
  "what does an IM user need to see?" filter (Token stream +
  terminal + a few notable events) and did not include even a debug
  log for the rest. The TUI and Tauri surfaces, written later and
  for a richer UI, added the catch-all-visible pattern; the channel
  surface was not retrofitted.
- Direction: at minimum, change `channels.rs:625` from `_ => {}` to
  log at debug level (`tracing::debug!(?payload, "channel renderer
  dropped agent event")`). Optionally, surface `ContextCompressed`
  as an `OutboundMessage("[context compressed]")` so the IM user
  has a quality-degradation signal — mirror what the GUI does via
  `ChatEvent::ContextCompressed` → `chatEventHandler.context_compressed`
  → `store.clearContextWindow()`. The other drops (tool lifecycle,
  LlmUsage, batch markers) can stay silent on IM by design, but
  should be commented as intentionally-ignored so the next
  contributor does not think they were missed.
- Regression validation: a unit test in `channels.rs`'s test module
  that feeds each currently-dropped variant through
  `aggregate_by_sentence` and asserts the drop is logged (for the
  debug-log fix) or asserted to produce the expected
  `OutboundMessage` (for the `ContextCompressed` surfacing fix).
- Validation reports: [V02](../validations/X-EVT-01/V02-01.md),
  [V01](../validations/X-EVT-01/V01-01.md).

### X-EVT-01-P2-03: `validate_event_trajectory` is exported but unused — terminal monotonicity is enforced only by the envelope wrapper's `break`, not independently verified

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/agent/event_envelope.rs:197-295` —
    `validate_event_trajectory` checks: schema version, contiguous
    sequences, identity stability, no duplicate `event_id`, parent-
    before-child tool lifecycle, exactly one terminal, terminal is
    last, no unfinished tool calls.
  - `echo-agent/echo-core/src/agent/mod.rs:18` — re-exports
    `validate_event_trajectory` from the agent module.
  - `echo-agent/src/lib.rs:143` — re-exports it into
    `echo_agent::prelude`.
  - `grep -rn "validate_event_trajectory" --include="*.rs"` returns:
    - the definition at `event_envelope.rs:197`,
    - two test calls at `event_envelope.rs:485, 502`,
    - the re-exports at `mod.rs:18` and `lib.rs:143`,
    - a reference in `echo-agent/src/agent/mod.rs:40` (another
      re-export).
  - **Zero production callers**. No consumer — not `chat_driver.rs`,
    not `task_runtime/executor.rs`, not `a2a/server.rs`, not
    `subagent/executor.rs` — invokes it on the envelopes they
    produce.
- Reachability: live as exported API; dead as production
  instrumentation. A third-party `echo-agent` consumer could call it,
  but the framework's own consumers do not.
- Expected invariant: the "exactly one terminal, terminal is last,
  contiguous sequence" invariants that `validate_event_trajectory`
  encodes are the same invariants the task question asks about
  ("ordering, terminal status"). They are enforced structurally by
  `envelope_event_stream_after`'s `break` (`event_envelope.rs:174-
  177`), but if that `break` were ever removed or weakened (e.g. by
  a future contributor who wants the wrapper to forward trailing
  diagnostic events after the terminal), no validator would catch
  the regression in production.
- Observed behavior: the validator runs only in its own unit tests
  (`event_envelope.rs:485, 502`). Production envelopes are not
  validated. The invariants hold today because the wrapper's `break`
  is correct, not because anything verifies it.
- Impact: low today, medium-long-term. The validator is a
  high-quality invariant checker that is not being used. The risk
  is regression: a future change to `envelope_event_stream_after`
  (or a sibling wrapper that does not break on terminal) would
  silently violate the "exactly one terminal" invariant downstream
  consumers rely on (the `subagentRunStore` terminal lock is a
  second line of defense, but the chat path's `chatStore` has no
  such lock — see X-EVT-01-P3-03).
- Root cause: the validator was authored alongside
  `envelope_event_stream_after` but never wired into a production
  assertion path. Its existence in `prelude` advertises it as a
  public contract without enforcing it.
- Direction: pick one.
  1. **Wire it in** (preferred): add a `debug_assert!(or `tracing::warn!`)
     that calls `validate_event_trajectory` on the envelope
     sequence at the end of `envelope_event_stream_after` (collect
     the yielded envelopes; validate; log violations). This makes
     the invariant self-checking in dev builds. The cost is minor
     (one pass over the sequence at stream end).
  2. **Stop exporting it**: remove it from `prelude` and the agent
     module re-export, and document it as a test-only helper. This
     is the smaller change but loses the invariant-checking
     opportunity.
  Option 1 is preferred under the AGENTS.md "no panic" rule (use
  `tracing::warn!`, not `debug_assert!`, so a violation is logged
  rather than crashing the agent).
- Regression validation: a test that drives `envelope_event_stream`
  with a stream that emits two terminals (e.g. via a custom mock
  that ignores the wrapper's break) and asserts
  `validate_event_trajectory` flags the violation. Today this test
  exists only for the in-wrapper case; a regression test for the
  out-of-wrapper case is the new addition.
- Validation reports: [V04](../validations/X-EVT-01/V04-01.md).

### X-EVT-01-P3-01: Chat AgentEvents are not persisted for replay — asymmetric durability vs subagent / tool / task-runtime events

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:323, 361, 377`
    — the only `trace_sink(ExecEvent::run(...))` calls in
    `chat_driver.rs`. They emit three run-level boundary events
    (`RunStarted`, `RunCancelled`, `RunFailed`) for Task-mode runs.
    No granular `AgentEvent` is persisted.
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:540-564`
    — the `drive_chat_inner` envelope-drain loop forwards each
    envelope to `sink.on_event` and discards; no `trace_sink` call
    for `AgentEvent` payloads.
  - `echo-agent-cli/web-frontend/src/stores/chatStore.ts:350-416`
    — `chatStore` has `replaceAll` (used by conversation load) but
    no hydrate entry that re-drives `AgentEvent`s. `scheduleAutoSave`
    persists only the final message array.
  - Contrast:
    `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:330-405`
    — `taskRuntimeSubagentExecutionEvents` adapter projects durable
    `RuntimeTaskEvent[]` into the live `ExecutionEvent` stream and
    re-drives `ingest`. Subagent state IS replayable.
  - Contrast:
    `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.ts:223-235`
    — `hydrateConversation` re-merges durable `ToolExecution[]` from
    the `ToolExecutionRepository`. Tool execution state IS replayable
    (on GUI; per A-CHAT-01-P2-01, only GUI persists).
- Reachability: every page reload during or after a chat turn.
- Expected invariant: the streaming trace (thinking segments,
  budget notices, guard notices, parameter errors, context-
  compressed notices) should be recoverable after reload, or the
  asymmetry should be documented. The other event families recover.
- Observed behavior: reload recovers (a) the final assistant
  message content, (b) GUI tool execution history (if a tool-using
  turn), (c) any active TaskRuntime run / plan / todos / subagents.
  Reload does NOT recover: thinking segments (`Token` events routed
  to `thinkingSegments`), budget notices, guard notices, parameter
  errors, context-compressed notices, per-iteration LLM usage
  history. The reloaded assistant message is a finalized blob with
  no record of the model's reasoning or any safety/budget events.
- Impact: medium for forensic / audit use cases. A user who
  supervises a long task, reloads the page (or restarts the TUI),
  and wants to review what happened during a specific turn sees
  only the final answer. The "why did the model decide X?" question
  cannot be answered from the persisted state. For Task-mode runs,
  the run-level boundary events are persisted, but not the
  per-iteration reasoning. Severity is P3 (not P2) because the
  final answer and the durable task/subagent/tool state ARE
  preserved; the gap is in streaming trace, not data integrity.
- Root cause: `AgentEvent` was designed as a live-streaming
  contract, not a durable record. `TaskRuntimeStore.events.jsonl`
  persists `RuntimeTaskEvent`s (which include tool/subagent
  boundaries) but not the raw chat `AgentEvent` stream. The
  `WebhookTurnObserver` partially compensates (it emits webhook
  events for some variants), but webhooks are an external HTTP
  side-effect, not a replayable store.
- Direction: pick one of:
  1. **Persist the envelope stream** for chat turns: have
     `drive_chat_inner` write each `EventEnvelope` to a per-turn
     file (e.g. `~/.eko/conversations/{conv_id}/{turn_id}/events.jsonl`)
     and add a frontend hydrate path that re-drives
     `chatEventHandler` from the file. This is the symmetric fix
     but adds I/O per event.
  2. **Document the asymmetry** in `chat_driver.rs` module docs:
     "chat AgentEvents are fire-and-forget; only the final assistant
     content and the Task-mode run boundaries are persisted. For
     full streaming trace recovery, persist envelopes upstream of
     the sink." This is the no-code-change option.
  3. **Persist only the audit-relevant subset** (budget notices,
     guard notices, parameter errors, context-compressed): route
     them through `trace_sink` as `ExecEvent::run`-style boundary
     events. Smaller surface than option 1; recovers the safety
     signals without the full token stream.
  Option 3 is the best ROI; option 1 is the structural fix; option
  2 is the documentation floor.
- Regression validation: for option 1/3, a test that drives a
  `drive_chat` turn with mock events, reloads the conversation, and
  asserts the streaming trace (or the audited subset) is recovered.
- Validation reports: [V03](../validations/X-EVT-01/V03-01.md).

### X-EVT-01-P3-02: `chatEventHandler.ts` is a non-exhaustive switch — TypeScript cannot catch a future `ChatEvent` variant addition

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts:25-220`
    — `switch (event.type)` over the `ChatEvent` discriminated
    union. 17 explicit case arms covering all 19 current variants
    (some variants share handling).
  - `grep -n "default\|never" chatEventHandler.ts` returns zero
    hits. No `default` case, no `const _exhaustive: never = event`
    assertion at the end.
  - Contrast: `echo-agent-cli/src/tui/events.rs:2204` uses
    `other => AgentEvent::Notice(...)` (visible catch-all).
    `echo-agent-cli/src/tauri/commands/chat.rs:1566-1570` uses
    `other => ChatEvent::Notice { code: "unknown_agent_event" }`
    (visible catch-all).
- Reachability: every GUI chat turn. Today the union is small and
  stable, so every variant is handled.
- Expected invariant: a discriminated-union switch should be
  compiler-exhaustive. TypeScript's structural exhaustiveness
  checking requires either a `: never` assertion or a `default`
  branch; without one, the compiler accepts the switch silently
  even when a variant is unhandled.
- Observed behavior: the switch handles every current variant but
  has no compiler-enforced guard. If a future commit adds a
  `ChatEvent` variant (e.g. someone wires `Chart` as a typed
  `chart` case instead of routing through `Notice`, or adds a new
  `subagent_card` variant), TypeScript would NOT flag the missing
  case; the switch would fall through and the event would be
  silently dropped at the frontend. The user would see no chart, no
  error, no log.
- Impact: low today (the union is stable), preventive for the
  future. The risk is the same silent-drop class as
  X-EVT-01-P2-02 but on the TS side and for future variants only.
- Root cause: the switch was written to handle the existing
  variants; the exhaustiveness assertion was not added. The TS
  `ChatEvent` type is a manual shadow of the Rust `ChatEvent` enum
  (A-FE-01-P3-04: no contract test links them), so a Rust-side
  variant addition does not propagate automatically.
- Direction: add an exhaustiveness assertion at the end of the
  switch:
  ```ts
  default: {
    const _exhaustive: never = event;
    console.warn('[chatEventHandler] unhandled ChatEvent', _exhaustive);
    return;
  }
  ```
  This makes TypeScript flag any future variant addition at compile
  time. The `console.warn` is the equivalent of the Rust surfaces'
  visible catch-all.
- Regression validation: a TS-side test that asserts the switch is
  exhaustive (compile-time check). Optionally, a contract test (per
  A-FE-01-P3-04) that imports the Rust-generated `ChatEvent` shape
  and asserts the manual TS union matches — this would catch
  Rust-side variant additions too.
- Validation reports: [V02](../validations/X-EVT-01/V02-01.md).

### X-EVT-01-P3-03: `chatStore.setRunStatus` has no terminal lock — terminal monotonicity relies on indirect `message_key` scoping

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/stores/chatStore.ts:391-397`
    — `setRunStatus: (status) => set({ runStatus: status, ... })`.
    Unconditional overwrite. No check like `if (prev === 'completed'
    && newStatus === 'running') return`.
  - `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:50-58`
    — `isCurrentRunEvent` filter: drops events whose `message_key`
    does not match `currentMessageKeyRef.current`. After the
    terminal `Done` event clears `currentMessageKeyRef`, late
    events from the old turn are dropped.
  - `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts:206-219`
    — `Done` arm clears `currentMessageKeyRef.current = null`,
    closing the filter window.
  - `echo-agent/echo-core/src/agent/event_envelope.rs:174-177` —
    framework-level `break` after the first terminal ensures no
    post-terminal events reach the envelope stream.
  - Contrast:
    `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:458-460`
    — explicit terminal lock `if (prev && prev.status !== 'running')
    return s;`. The subagent store IS structurally monotone.
- Reachability: every GUI chat turn. Today, the structural chain
  holds: framework break + message_key filter + Done clear → no
  late events reach `setRunStatus` after a terminal.
- Expected invariant: a finalized chat (`runStatus === 'completed'
  | 'failed' | 'cancelled'`) should not be reopenable by a late
  event. The store should enforce this directly.
- Observed behavior: the store does not enforce it. The monotonicity
  is an emergent property of three upstream layers (framework
  break, message_key filter, Done clear). A future change to any of
  those layers (e.g. allowing post-terminal diagnostic events
  through the envelope, or changing the message_key filter
  semantics) could let a late `run_status: running` event reopen a
  finalized chat — `setRunStatus` would happily flip `runStatus`
  back to `'running'`.
- Impact: low today (the upstream chain holds). Preventive: the
  subagent store's terminal lock is the right pattern, and
  `chatStore` should mirror it for consistency and defense-in-
  depth.
- Root cause: the chat reducer was written before the subagent
  reducer and did not adopt the terminal-lock pattern; the
  upstream message_key filter was considered sufficient.
- Direction: add a terminal lock to `setRunStatus`:
  ```ts
  setRunStatus: (status) => set((s) => {
    const TERMINAL = ['idle', 'completed', 'failed', 'cancelled'];
    // Allow transitions out of 'idle' (initial) and into any status,
    // but once terminal, do not reopen without an explicit reset.
    if (TERMINAL.includes(s.runStatus) && s.runStatus !== 'idle' &&
        !TERMINAL.includes(status)) {
      return s;
    }
    return { runStatus: status, ... };
  }),
  ```
  This mirrors `subagentRunStore`'s pattern and makes the chat
  store monotone independent of the upstream filter.
- Regression validation: a vitest test that drives `setRunStatus
  ('completed')` then `setRunStatus('running')` and asserts
  `runStatus === 'completed'`. Pair with the existing
  `chatEventHandler.test.ts` pattern.
- Validation reports: [V03](../validations/X-EVT-01/V03-01.md),
  [V04](../validations/X-EVT-01/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Producer-to-all-consumer matrix: each `AgentEvent` variant × each declared consumer | yes | passed (with finding) | [V01-01](../validations/X-EVT-01/V01-01.md) |
| V02 | Variant exhaustiveness per consumer (explicit / catch-all-visible / catch-all-silent) | yes | passed (with finding) | [V02-01](../validations/X-EVT-01/V02-01.md) |
| V03 | Recorded event replay: persist + hydrate + out-of-order/duplicate tolerance, per event family | yes | passed (with finding) | [V03-01](../validations/X-EVT-01/V03-01.md) |
| V04 | Duplicate / out-of-order / Cancelled-vs-Error terminal conformance across all surfaces | yes | passed (with finding) | [V04-01](../validations/X-EVT-01/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a | No prior X-EVT-01 report under `zcode-glm/`; no historical document is cited as evidence for a claim. The framework docstrings on `AgentEvent`, `envelope_event_stream_after`, and `cancel_aware_stream` are read at the cited line numbers and treated as current code-level contracts (verified, not asserted). |

No `cargo` or `vitest` command was required: this is a cross-surface
conformance review. The relevant executable claims are owned by the
dependency reports:

- F-RCT-03 executed `cargo test --lib -p echo_agent -- test_run_stream_cancelled_mid_llm_call ...` (exit 0).
- A-CHAT-01 executed `cargo test -p echo-agent-app-core --lib chat_driver::` (9 passed, exit 0).
- A-FE-02 executed `npx vitest run --reporter=dot` (26 files, 101 tests, exit 0).
- A-FE-01 was static-only.

This task's V01-V04 reuse those executable results as upstream
evidence; no new build was needed.

## Historical Claim Status

No historical documents are cited as evidence for any claim in this
report. All findings are based on code at commit `9b0e0fa` /
`b3b2e81` and the four validation reports above. The dependency
reports' conclusions are cited as current where load-bearing; their
 staleness conditions are inherited (see Handoff).

## Coverage And Uncertainty

Inspected in full: the framework `AgentEvent` enum + `EventEnvelope`
+ `envelope_event_stream_after` + `validate_event_trajectory` +
trajectory tests; the `ChatDriverEvent` enum + `WebhookTurnObserver`
+ `ChannelChatSink` + `drive_chat` lifecycle (including the
`trace_sink` emission points); the four `ChatSink` implementations
(`TauriChatSink` struct + `handle_tool_event` + `cancel_active_tools`
+ `ChatSink` impl + `agent_event_to_chat_event`,
`TuiChatSink::on_event`, `aggregate_by_sentence`, plus the
`ChannelChatSink` forwarder); the four entry-point callers
(`send_chat`, `send_to_agent`, `run_repl_turn`, channels `handle`)
for post-`drive_chat` status emission; the TS `useTauriChat`
listener + `chatEventHandler` + `chatStore` reducer +
`subagentRunStore` reducer + `toolExecutionStore` reducer +
`taskRuntimeStore` reducer (head).

Not inspected (out of scope or deferred):

- The concrete `ReactAgent` streaming impls (`run_stream_channel`,
  `cancel_aware_stream` trait default) — F-RCT-03 owns them. This
  task consumes F-RCT-03-P2-02 (ReactAgent never emits Cancelled) as
  the upstream contract.
- The framework `GLOBAL_EVENT_BUS` / `EventBus` dead-infra question
  — F-CORE-01-P2-01 owns it. This task reuses the conclusion.
- The `ToolExecutionRepository` persistence backend (SQLite vs file)
  — A-TSK-04 / A-STATE-01 own it. This task confirms tool
  cancellation is idempotent on terminal, not the backend.
- The Tauri-side `execution://event` bridge (`emit_execution_event`
  hand-built JSON) — A-SRF-02 owns the typing-level drift; A-FE-01
  owns the field-level drift. This task references the bridge only
  for terminal-event emission (kind="subagent", terminal
  event_types).
- The webhook emitter's HTTP transport — out of scope; the
  `WebhookTurnObserver` is classified as a read-only observer.

Environmental constraints:

- Read-only static review against `echo-agent` `9b0e0fa` and
  `echo-agent-cli` `b3b2e81`. No code was modified; no `cargo` /
  `vitest` was run by this task (the dependency reports' runs are
  reused).
- Both repos clean at the audited commits.

Uncertain claims:

- Whether the GUI's post-`drive_chat` `TurnStatus` compensation is
  truly race-free. Static analysis says yes
  (`cancel.is_cancelled()` is polled synchronously after `drive_chat`
  returns; the token is the same one passed into
  `ChatResources.cancel`). A runtime test would close the gap; this
  task inherits A-CHAT-01's static guarantee.
- Whether any third-party `echo-agent` consumer outside this monorepo
  relies on `AgentEvent::Cancelled` being emitted by the framework
  stream (F-RCT-03-P2-02 uncertainty, inherited). Such a consumer
  would break on ReactAgent today.
- Whether the `ContextCompressed` silent drop in the channel renderer
  (X-EVT-01-P2-02) has any user-visible consequence today. The
  signal IS dropped, but whether any channel user has noticed answer
  quality degrade after a compression is a usage question, not a
  code question.

## Handoff

Conclusions downstream tasks may rely on:

1. **The producer-to-consumer graph has no fan-out.** Framework
   `GLOBAL_EVENT_BUS` is dead (F-CORE-01-P2-01); each
   `envelope_event_stream` wrap site feeds exactly one sink. Any
   task reasoning about event distribution can treat the path as
   single-producer-single-consumer per stream.
2. **The one-terminal invariant holds at the framework envelope,
   but is not independently verified in production.**
   `validate_event_trajectory` exists but is unused
   (X-EVT-01-P2-03). The invariant depends on
   `envelope_event_stream_after`'s `break`; any change to that
   `break` would silently break downstream consumers (the
   subagentRunStore has a second-line terminal lock; chatStore does
   not — X-EVT-01-P3-03).
3. **Cancelled-vs-Error is structurally collapsed for ReactAgent
   turns.** Only GUI recovers via post-`drive_chat` polling
   (X-EVT-01-P2-01). TUI / REPL / channels render cancel as error.
   The framework fix (F-RCT-03-P2-02) would resolve this for every
   surface; until then, the application fix is to add post-
   `drive_chat` `TurnStatus{cancelled}` emission to TUI / REPL /
   channels.
4. **Chat AgentEvents are fire-and-forget at every sink.** No
   replay path exists for the streaming trace
   (X-EVT-01-P3-01). Subagent / tool / task-runtime events DO
   replay durably. Any task that needs full turn forensics after
   reload must persist envelopes upstream.
5. **The channel renderer is the only consumer that silently drops
   unmatched `AgentEvent` variants** (X-EVT-01-P2-02). Every other
   surface surfaces unknown variants (TUI / Tauri via Notice;
   chatStore via the typed `ChatEvent` Notice bucket). The TS
   chatEventHandler is exhaustive today but has no compiler guard
   (X-EVT-01-P3-02).

Reports downstream tasks must read:

- This report (X-EVT-01) for the cross-surface conformance matrix
  and the four headline findings.
- `tasks/F-CORE-01.md` for the `AgentEvent` / `EventEnvelope` /
  `is_terminal()` / `cancel_aware_stream` contracts and the dead
  `GLOBAL_EVENT_BUS`.
- `tasks/F-RCT-03.md` for the streaming event flow, the ReactAgent
  Cancelled-emission defect (P2-02), and the droppable error
  terminals (P2-01).
- `tasks/A-CHAT-01.md` for the `drive_chat` lifecycle, the
  one-terminal invariant delegation to `envelope_event_stream`, and
  the TUI/REPL/channels missing post-`drive_chat` `TurnStatus`.
- `tasks/A-FE-01.md` for the IPC type-contract matrix (the TS
  `ChatEvent` is a manual shadow of the Rust enum with no contract
  test).
- `tasks/A-FE-02.md` for the subagent reducer identity model and
  terminal lock, the tool-execution live-ingest overwrite, and the
  acceptance-gate projection gap.

Conditions that make this report stale:

- Any change to `envelope_event_stream_after`'s `break`-on-terminal
  invalidates V04 scenario 1 (duplicate terminals) and the
  "exactly one terminal" claim.
- Any change to `ReactAgent`'s `chat_stream_with_cancel` /
  `execute_stream_with_cancel` overrides that wraps
  `cancel_aware_stream` (resolving F-RCT-03-P2-02) invalidates
  X-EVT-01-P2-01 and V04 scenario 3.
- Wiring `validate_event_trajectory` into production (resolving
  X-EVT-01-P2-03) invalidates the "validator unused" claim.
- Adding a post-`drive_chat` `TurnStatus{cancelled}` emission to
  TUI / REPL / channels (the application-side fix for X-EVT-01-P2-01)
  invalidates the per-surface cancel-recovery matrix in V04.
- Adding a default/never exhaustiveness guard to
  `chatEventHandler.ts` (resolving X-EVT-01-P3-02) invalidates the
  "non-exhaustive switch" claim.
- Adding a terminal lock to `chatStore.setRunStatus` (resolving
  X-EVT-01-P3-03) invalidates the "no terminal lock" claim.
- Persisting chat `AgentEvent`s upstream of the sink (resolving
  X-EVT-01-P3-01) invalidates the "fire-and-forget" claim.
- Any new `AgentEvent` variant requires re-running V01/V02 (the
  variant × consumer matrix and the exhaustiveness classification).

Follow-up task IDs (no fixes implemented in this review):

- **F-RCT-03-P2-02** (framework, existing) — wrap
  `cancel_aware_stream` in ReactAgent's streaming-with-cancel
  overrides. This is the root-cause fix for X-EVT-01-P2-01.
- A **cross-surface cancel-parity** task (application) — until the
  framework fix lands, add post-`drive_chat` `TurnStatus{cancelled}`
  emission to `send_to_agent` (TUI), `run_repl_turn` (REPL), and
  channels `handle`. This is the parity backstop for X-EVT-01-P2-01.
- A **framework invariant self-check** task — wire
  `validate_event_trajectory` into `envelope_event_stream_after`'s
  tail as a `tracing::warn!` (resolves X-EVT-01-P2-03).
- A **channel renderer visibility** task — change
  `channels.rs:625` from `_ => {}` to at minimum a `tracing::debug!`
  log, and surface `ContextCompressed` as an `OutboundMessage`
  (resolves X-EVT-01-P2-02).
- A **chat-event persistence** task — persist the envelope stream
  (or at least the audit-relevant subset: budget / guard /
  parameter / context-compressed notices) for replay
  (resolves X-EVT-01-P3-01).
- A **TS exhaustiveness** task — add the `default: const _exhaustive:
  never = event` guard to `chatEventHandler.ts` (resolves
  X-EVT-01-P3-02).
- A **chatStore terminal lock** task — add a terminal-status guard
  to `chatStore.setRunStatus` mirroring `subagentRunStore`'s pattern
  (resolves X-EVT-01-P3-03).
