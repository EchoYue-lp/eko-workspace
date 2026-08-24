# A-CHAT-01: Shared chat driver and sinks

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (read-only; `envelope_event_stream` + `AgentEvent` contract)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Does `drive_chat` own one application lifecycle while sinks only
render/transport events?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs` (full, 1175 lines)
  — `drive_chat`, `drive_chat_inner`, the `ChatSink` trait,
  `ChatDriverEvent`, `ChannelChatSink`, `WebhookTurnObserver`,
  `subagent_trace_sink_for`, `framework_trace_sink_for`,
  `ensure_task_mode_run`, `finalize_task_mode_run`,
  `observe_execution_path`, and the 9-test unit module.
- `echo-agent-cli/echo-agent-app-core/src/chat_resources.rs` (full, 89 lines)
  — `ChatResources`, the `CURRENT_CHAT_RESOURCES` task_local,
  `with_chat_resources`, `current_chat_resources`.
- `echo-agent-cli/echo-agent-app-core/src/run_driver.rs` (head, 50 lines) —
  confirms `drive_run_async` is the *separate* task-run half, not a parallel
  chat driver.
- Sink implementations:
  - `echo-agent-cli/src/tui/events.rs:2008-2050` (`TuiChatSink`) and
    `2052-2224` (`on_event` mapping).
  - `echo-agent-cli/src/tauri/commands/chat.rs:1148-1156` (`TauriChatSink`
    struct), `1164-1191` (`cancel_active_tools`), `1193-1340`
    (`handle_tool_event`), `1341-1411` (`ChatSink` impl).
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:575-591`
    (`ChannelChatSink`, shared by REPL + channels).
- Entry-point callers of `drive_chat`:
  - TUI: `echo-agent-cli/src/tui/events.rs:2226` (`send_to_agent`) and
    `1378-1435` (`handle_enter` resources assembly, cancel binding).
  - REPL: `echo-agent-cli/src/cli/repl.rs:495-545` (`run_repl_turn`).
  - Tauri: `echo-agent-cli/src/tauri/commands/chat.rs:625-712` (`send_chat`,
    including post-`drive_chat` `TurnStatus` emission) and `745-803`
    (`steer_turn`) and `805-830` (`cancel_chat`).
  - Channels: `echo-agent-cli/src/cli/channels.rs:240-270`.
- Framework terminal/normalization contract (read-only, for the one-terminal
  invariant):
  - `echo-agent/echo-core/src/agent/event_envelope.rs:107-194`
    (`envelope_event_stream` / `envelope_event_stream_after`).
  - `echo-agent/echo-core/src/agent/mod.rs:330-336` (`AgentEvent::is_terminal`).
  - `echo-agent/src/agent/handle.rs:187` + `react/mod.rs:271`
    (`steer_input`, mid-turn injection).

## Out Of Scope

Deferred to downstream tasks:

- **A-INP-01**: `PreparedUserTurn` normalization and the TUI `/steer`
  input-box bypass (already reported as A-INP-01-P2-01). This task treats
  `PreparedUserTurn` as the input contract.
- **F-RCT-02 / F-RCT-03**: the framework `run_core_loop` terminal partition,
  the dropped-terminal-under-backpressure defect (F-RCT-03-P1-01/P2-01), and
  the ReactAgent-never-emits-`Cancelled` defect (F-RCT-03-P2-02). This task
  inherits those as the upstream contract and analyses only the
  application-layer wrapper.
- **A-TSK-***: `TaskRuntimeStore` internals, `ensure_task_mode_run` /
  `finalize_task_mode_run` DAG/plan semantics, and the
  `drive_run_async` / `drive_agent_run` task-run lifecycle (confirmed
  separate from chat, not audited here).
- **A-SRF-01 / A-SRF-02**: full TUI / Tauri capability matrices. This task
  audits only the sink responsibility split.
- **F-CORE-01**: the `EventEnvelope` / `EventIdentity` / `stable_event_id`
  contracts (consumed as stable input).

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (multi-mode parity rule, "only Subagent no
  Worker" terminology, framework-vs-application layering gate, no-panic /
  UTF-8 safety rules, "check whether it already exists before adding").
- `docs/comprehensive-review/REPORTING.md` (finding + validation contract).
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (A-CHAT-01 card).

Dependency reports read:

- `zcode-glm/tasks/A-INP-01.md` (complete) — establishes
  `PreparedUserTurn` as the single input contract handed to `drive_chat`,
  and the five-entry field matrix. Its finding A-INP-01-P2-01 (TUI
  `/steer` input-box bypass) is load-bearing for this task's steer
  analysis: it confirms the *slash-command* `/steer` path does NOT go
  through `drive_chat` (it calls `agent.steer_input` directly), so steer
  is a mid-turn injection, not a second `drive_chat` lifecycle.
- `zcode-glm/tasks/F-RCT-02.md` (complete) — establishes the single
  `run_core_loop`, the 10-arm terminal partition, and the
  trace-finalization asymmetry. Load-bearing for the one-terminal
  invariant: the framework can end a turn without emitting a terminal
  (F-RCT-02-P2-03 / P3-01 abandoned arms, F-RCT-03-P2-01 dropped error
  terminals).
- `zcode-glm/tasks/F-RCT-03.md` (complete) — establishes the streaming
  event flow, the lossy intermediate-event drop policy, and
  F-RCT-03-P2-02 (ReactAgent never emits `AgentEvent::Cancelled`).
  Load-bearing for V03/V04: `drive_chat` must compensate for the
  framework's missing terminal on cancel/abandon.
- `zcode-glm/tasks/B-PATH-01.md` (complete) — confirms all chat entry
  points (TUI/CLI/channels/GUI) route through one shared composition root
  and that there is no sibling chat path. Used as the cross-reference for
  the four-caller reachability map.

Historical documents treated as hypotheses:

- `chat_driver.rs:1-15` module docstring — claims `drive_chat` is "the
  single, thin entry for a chat turn across TUI / CLI channel / GUI" and
  "streams the agent's ReAct reply through a per-mode `ChatSink`, and
  stops". Treated as **current** for the single-entry claim (V01) and
  **partially overstated** for the "sinks render" claim (V02: TauriChatSink
  owns persistence authority).
- `chat_resources.rs:1-9` docstring — claims `drive_chat` scopes
  `ChatResources` per turn. Treated as **current** (V01).

## Layering Decision

This is an **application-layer** task. `drive_chat`, `ChatSink`,
`ChatDriverEvent`, `ChatResources`, the per-mode sinks, the
`WebhookTurnObserver`, the `ensure_task_mode_run` /
`finalize_task_mode_run` Task-mode gating, and the
`observe_execution_path` telemetry are all EKO product policy (the
local-assistant turn lifecycle, mode-hint folding, Task-vs-Auto-vs-Chat
tool hiding, webhook fan-out). None belong in the framework.

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework supplies the right primitives: `AgentEvent`, `EventEnvelope`, `envelope_event_stream` (terminal normalization), `execute_stream_message_with_invocation_context`, `steer_input`, `CancellationToken`. `drive_chat` is the application's composition of these — correct layering. |
| EKO product policy | The `ChatSink` trait, the `ChatDriverEvent` enum (adds `Execution` / `TurnStatus` / `ExecutionPath` / `Interrupt` on top of framework `AgentEvent`), the Task-mode run bootstrap, and the per-mode rendering are all product policy, correctly in `echo-agent-app-core` / `src/{tui,tauri,cli}`. |
| Adapter boundary | `drive_chat` is a thin adapter: it converts `PreparedUserTurn → Message`, sets up `AgentInvocationContext`, calls one framework streaming API, wraps with `envelope_event_stream`, and forwards each envelope to `sink.on_event`. It owns no scheduling beyond the Task-mode run bookkeeping (which delegates to `TaskRuntimeStore`). The `WebhookTurnObserver` is a pure read-only observer. |
| Duplicate search | Searched both repos for: `drive_chat`, `drive_chat_inner`, `ChatSink`, `ChatDriverEvent`, `ChatResources`, `with_chat_resources`, `current_chat_resources`, `TuiChatSink`, `TauriChatSink`, `ChannelChatSink`, `WebhookTurnObserver`, `send_to_agent`, `run_repl_turn`, `execute_stream_message_with_invocation_context`. Result: exactly one `drive_chat` definition (`chat_driver.rs:202`); exactly one chat-turn lifecycle owner. The only other `execute_stream_*` callers in the app layer are `tasks/task_runtime/executor.rs:3119, 3130, 3734` — the **task-run** half (`drive_agent_run` / `drive_run_async`), which is intentionally a separate lifecycle on isolated pool agents (confirmed by `run_driver.rs:1-12` docstring). No parallel chat driver. |
| Migration deletion | No deletion proposed. The dead `Err(e)` branch in `drive_chat_inner` (P3-01) is a candidate for removal or an explicit unreachable guard, not a parallel-authority deletion. |

## Current Path

### Verified caller / reachability map (V01)

```text
TUI handle_enter (events.rs:1294)
   │  PreparedUserTurn::build (events.rs:1371-1380)
   │  cancel = CancellationToken::new(); app.active_cancel = Some(cancel.clone())  [:1409-1410]
   │  sink = TuiChatSink(tx)                                          [:2010, assembled in handle_enter]
   │  ChatResources{ sink, cancel, root_message_id=turn_id, ... }     [:1411-1435]
   ↓
send_to_agent → tokio::spawn(drive_chat(agent, turn, res))            [:2222-2227]

REPL run_repl_turn (repl.rs:495)
   │  PreparedUserTurn::build (repl.rs:509-517)
   │  sink = ChannelChatSink(tx)                                      [:507]
   │  ChatResources{ sink, cancel=CancellationToken::new(), ... }     [:524-540]
   ↓
tokio::spawn(drive_chat(agent, turn, resources))                      [:541-543]

Tauri send_chat (chat.rs:625)
   │  PreparedUserTurn::build (chat.rs:636-644)
   │  cancel_token registered in session.cancel_token[message_key]    [:625 ctx]
   │  sink = TauriChatSink{ tool_executions, execution_projector, ... } [:constructed earlier in send_chat]
   │  ChatResources{ sink, cancel=cancel_token, ... }                 [:664-680]
   ↓
tokio::spawn(drive_chat(agent, prepared_turn, res))                   [:686-689]
   │  AFTER drive_chat returns:
   │     terminal_status = cancel.is_cancelled() ? "cancelled" : outcome.is_ok() ? "completed" : "failed"
   │     sink.on_event(TurnStatus{ status: terminal_status })         [:704-712]

Channels handle (channels.rs:195)
   │  PreparedUserTurn::build (channels.rs:208-216)
   │  sink = ChannelChatSink(tx)                                      [:247]
   │  ChatResources{ sink, cancel=CancellationToken::new(), ... }     [:249-260]
   ↓
drive_chat(agent, turn, res)                                          [:262]
```

**Four production callers, one entry.** Every chat turn goes through
`drive_chat` (`chat_driver.rs:202`). There is no sibling chat-streaming
path: `grep "execute_stream_message_with_invocation_context"` in the app
layer returns only `chat_driver.rs:513` (inside `drive_chat_inner`) and
the task-run executor sites. `drive_run_async` / `drive_agent_run`
(`run_driver.rs`, `executor.rs`) drive already-created TaskRuntime runs
on isolated pool agents — a deliberately separate lifecycle, not a
parallel chat driver.

### Verified `drive_chat` lifecycle (V01, V03)

```text
drive_chat(agent, turn, res)                                 [chat_driver.rs:202]
   │  turn_id = res.root_message_id (fallback uuid if empty)   [:211-216]
   │  trace_sink = subagent_trace_sink_for(&sink)              [:221]
   │  formal_run_id = formal_run_id_for_turn(&turn_id)         [:222]
   │  if Task mode:
   │      ensure_task_mode_run(store, formal_run_id, …)         [:230-238]
   │      store.register_run_cancellation(formal_run_id, cancel)[:243-249]
   │  projection_registry.register(formal_run_id, store)        [:254-257]
   ↓
with_run_context(formal_run_id, cancel, trace_sink, drive_chat_inner(...))  [:258-264]
   │
   │  drive_chat_inner(agent, turn, res, turn_id)               [:425]
   │     msg = turn.to_message()                                 [:435-438]
   │     invocation = AgentInvocationContext{ runtime: ExternalRunContext{...}, disabled_tools, visible_tools }  [:483-511]
   │     stream_result = guard.execute_stream_message_with_invocation_context(msg, cancel, invocation)            [:512-514]
   │     ┌─ Err(e): sink.on_event(Agent(Error{source:"chat_driver", e})) + webhook AgentError + return Err  [:517-536]
   │     └─ Ok(raw_stream):
   │          stream = envelope_event_stream(raw_stream, event_identity)  [:538]
   │          while let Some(event_result) = stream.next().await:         [:540]
   │              Ok(event) → webhook_observer.observe; if !sink.on_event(Agent(event)) { break }  [:542-547]
   │              Err(e)   → [DEAD today: envelope never yields Err] webhook + return Err           [:548-560]
   │          webhook_observer.finish(); Ok(())                          [:563-564]
   ↓
if Task mode: finalize_task_mode_run(store, formal_run_id, cancel.is_cancelled(), trace_sink)  [:265-272]
observed = observe_execution_path(store, formal_run_id, turn_id, requested_mode)               [:273-275]
let _ = sink.on_event(ExecutionPath{ requested_mode, observed_path })                          [:276-279]
return result                                                                                          [:286]
```

**Single lifecycle owner.** `drive_chat` is entered exactly once per chat
turn, owns the `turn_id` ↔ `formal_run_id` identity, scopes
`ChatResources` into the `CURRENT_CHAT_RESOURCES` task_local (via
`with_chat_resources` inside `drive_chat_inner`), and returns once. The
spawned task (`send_to_agent` / `send_chat` / REPL / channels) awaits
exactly one `drive_chat` completion.

### One-terminal invariant (V03)

`drive_chat` does **not** enforce the one-terminal invariant itself; it
delegates it to `envelope_event_stream`
(`echo-core/src/agent/event_envelope.rs:112-194`), which:

1. normalizes every raw `Err(error)` into an `AgentEvent::Error` envelope
   (`:134-140`);
2. `break`s after the first terminal payload (`is_terminal()` true —
   `FinalAnswer | Cancelled | Error`) (`:157, 174-177`);
3. if the raw stream ends (`None`) without a terminal, synthesizes exactly
   one `AgentEvent::Error { source: "agent_stream", message: "agent stream
   ended without a terminal event" }` (`:180-191`).

Because `envelope_event_stream` only ever `yield Ok(envelope)` (`:173,
:182`), the `Err(e)` arm of `drive_chat_inner`'s loop
(`chat_driver.rs:548-560`) is **unreachable today** — see P3-01.

The stream-**setup** error path (`execute_stream_message_with_invocation_context`
returning `Err`) is handled separately and correctly: `drive_chat_inner`
synthesizes one `AgentEvent::Error { source: "chat_driver", … }` envelope
and forwards it to the sink before returning `Err` (`:517-536`). So the
sink sees exactly one terminal even when the stream never starts.

Net result for the four terminal classes:

| Class | Path to sink | Terminal delivered? |
|---|---|---|
| Happy (FinalAnswer) | envelope forwards real FinalAnswer, breaks | yes, 1 |
| Stream-setup error | drive_chat_inner synthesizes Error | yes, 1 |
| Mid-stream framework error / dropped terminal (F-RCT-03-P2-01) | raw stream ends → envelope synthesizes "ended without terminal" Error | yes, 1 (degraded semantics) |
| Cancel (F-RCT-03-P2-02: ReactAgent never emits Cancelled) | raw stream ends → envelope synthesizes Error | yes, 1 (labelled Error, not Cancelled) |
| Sink closes early (`on_event` returns false) | loop breaks; terminal may not be forwarded | consumer's choice |

So the invariant "`drive_chat` delivers exactly one terminal AgentEvent
to the sink per call" **holds** (assuming the sink does not close early),
but the *semantic* label on cancel/abandon is uniformly `Error`, never
`Cancelled`. The Tauri caller compensates with a post-`drive_chat`
`TurnStatus { status: "cancelled" | "completed" | "failed" }` event
(`chat.rs:704-712`); the TUI / REPL / channels callers do **not** emit
such a status — see Coverage.

### Sink responsibility diff (V02)

| Sink | Location | What `on_event` does | Persistence / state authority? |
|---|---|---|---|
| `ChannelChatSink` | `chat_driver.rs:575-591` | `tx.send(event).is_ok()` — pure forward to mpsc | **None.** Used by REPL + channels. |
| `TuiChatSink` | `tui/events.rs:2031-2224` | Maps each `ChatDriverEvent` to the TUI's local `AgentEvent` enum, forwards to mpsc `tx` | **None.** Pure render projection. The TUI's `ToolExecutionMessage` is an in-memory UI struct, not a repository. |
| `TauriChatSink` | `tauri/commands/chat.rs:1341-1411` | Calls `handle_tool_event` (persists tool start / output / completion / failure / cancel via `ToolExecutionRepository`), maps the rest to `ChatEvent`, emits Tauri events; on `TurnStatus != running` calls `cancel_active_tools` | **Yes — owns `ToolExecutionRepository` (Arc), `active_tool_ids`, `tool_completions`.** Mutates a durable store from inside the sink. |
| `MockChatSink` | `chat_driver.rs:692-714` | Records events for assertions | test only |

The `WebhookTurnObserver` (`chat_driver.rs:83-176`) is correctly layered
**inside** `drive_chat_inner`, not inside any sink — so webhook fan-out
is identical across all four modes. This is the pattern the
tool-execution persistence *should* follow but does not (P2-01).

## Findings

### A-CHAT-01-P2-01: `TauriChatSink` owns tool-execution persistence authority — sinks do not "only render/transport"

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/chat.rs:1148-1156` —
    `TauriChatSink` holds
    `tool_executions: Arc<ToolExecutionRepository>`,
    `tool_completions: StdMutex<HashMap<String, PendingToolCompletion>>`,
    `active_tool_ids: StdMutex<HashSet<String>>`,
    `execution_projector: Arc<TauriExecutionProjector>`.
  - `echo-agent-cli/src/tauri/commands/chat.rs:1193-1340` —
    `handle_tool_event` calls
    `self.tool_executions.start(owner, …, call_id, name, args)`
    (`:1219`), `self.tool_executions.append_output(...)` (`:1248`),
    `self.tool_executions.finish(&owner, call_id, true, output, …)`
    (`:1268`), `self.tool_executions.finish(…, false, error, …)`
    (`:1300`), and on terminal `cancel_active_tools` calls
    `self.tool_executions.cancel(owner, &call_id)` (`:1177`).
  - `echo-agent-cli/src/tauri/commands/chat.rs:1342-1350` — the
    `ChatSink::on_event` impl dispatches every `Agent` event through
    `handle_tool_event` *before* rendering, so persistence is mandatory
    and unconditional for GUI.
  - Contrast: `echo-agent-cli/src/tui/events.rs:2031-2200`
    (`TuiChatSink::on_event`) performs only enum mapping +
    `self.tx.send(mapped).is_ok()` — no repository. `grep -rn
    "ToolExecutionRepository" echo-agent-cli/src/tui` returns zero hits.
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:83-176` —
    `WebhookTurnObserver` is the precedent for the correct layering: a
    cross-cutting observer lives inside `drive_chat_inner` and runs for
    every sink, so webhook emission is mode-uniform.
- Reachability: every GUI chat turn. `send_chat` constructs the
  `TauriChatSink` with the shared `state.app_state` repository
  (`tool_executions`) and passes it as `ChatResources.sink`; `drive_chat`
  then forwards every `AgentEvent::ToolCall/ToolResult/ToolError/ToolStream`
  to `sink.on_event`, which mutates the repository. The TUI/REPL/channels
  turns forward the same events to their sinks, which render only.
- Expected invariant: the task question — "sinks only render/transport
  events". AGENTS.md multi-mode parity rule — "TUI、GUI(以及
  CLI/channel)必须功能对等". A sink that owns a durable repository is a
  data-authority boundary, not a renderer.
- Observed behavior: tool-execution history (the `ToolExecutionRepository`
  that backs the GUI's tool-history panels and the
  `echo-assistant://tool-execution` surface) is written **only** on the
  GUI path, **inside the sink**. TUI / CLI / channels turns render the
  same `ToolCall`/`ToolResult` events to the screen/channel and then
  discard them; no tool-execution record is persisted for those turns.
  The authority is also fragile: if a future TUI sink wanted parity, it
  would have to duplicate the `start/append_output/finish/cancel` state
  machine currently buried in `TauriChatSink::handle_tool_event`.
- Impact: (a) multi-mode parity gap — only GUI has durable
  tool-execution history; (b) misplaced authority — a sink is a
  render/transport boundary, but `TauriChatSink` doubles as the
  tool-execution write side; (c) the `ToolExecutionRepository` writes
  happen "best effort" (errors are `tracing::warn!` and the sink returns
  `Some(true)` to keep streaming, `:1226-1230, :1255-1258`), so a
  repository failure is silently swallowed inside a renderer.
- Root cause: the GUI tool-history feature was added as a Tauri command
  concern, and the persistence was wired into the GUI sink because that
  was the only consumer at the time. The cross-cutting-observer pattern
  (`WebhookTurnObserver`) was not generalized to tool-execution
  recording.
- Direction: extract tool-execution recording into a driver-level
  observer mirroring `WebhookTurnObserver` — e.g. a
  `ToolExecutionObserver` constructed inside `drive_chat_inner` from
  `res.tool_executions` (a new `Option<Arc<ToolExecutionRepository>>`
  field on `ChatResources`, supplied by all modes that want durable
  history). `drive_chat_inner` calls `observer.observe(&event.payload)`
  alongside `webhook_observer.observe(...)`. `TauriChatSink` then only
  renders (emits Tauri events / `ChatEvent`s) and the
  `handle_tool_event`/`cancel_active_tools` persistence moves to the
  observer. TUI / CLI / channels gain durable history for free by
  supplying the repository in their `ChatResources`. The
  `PendingToolCompletion` / `active_tool_ids` bookkeeping moves with the
  observer. No `ChatSink` implementation should hold a repository after
  this change.
- Regression validation: a test that drives a tool-calling turn through
  `drive_chat` with a `MockChatSink` (pure render) AND a
  `tool_executions` repository on `ChatResources`, then asserts the
  repository recorded `start`+`finish` for the `call_id`. Today no such
  test exists because the persistence is GUI-sink-coupled.
- Validation reports: [V02](../validations/A-CHAT-01/V02-01.md).

### A-CHAT-01-P3-01: `drive_chat_inner`'s `Err(e)` stream branch is dead code and would drop the terminal if it ever fired

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:548-560` — the
    `Err(e)` arm of `while let Some(event_result) = stream.next().await`.
    On `Err` it logs, emits a webhook `AgentError`, and `return
    Err(error)` — it does **not** forward any event to `sink`.
  - `echo-agent/echo-core/src/agent/event_envelope.rs:128-192` —
    `envelope_event_stream_after` only ever `yield Ok(envelope)` (`:173`)
    or `yield Ok(EventEnvelope::new(..., AgentEvent::Error{...}))`
    (`:182`). It never yields `Err(...)`: raw stream errors are
    normalized into `AgentEvent::Error` payloads at `:134-140`.
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:538` — the
    `stream` is the output of `envelope_event_stream`, so `event_result`
    is always `Ok(_)`.
- Reachability: unreachable on the live path. The branch is a defensive
  guard for "future transport adapters that can fail independently" per
  the inline comment (`:549-551`).
- Expected invariant: every terminal path forwards a terminal event to
  the sink (the one-terminal invariant). A branch that returns `Err`
  without forwarding a terminal would violate it.
- Observed behavior: today the branch cannot fire, so the invariant is
  not actually violated. But the branch is misleading: it suggests a
  mid-stream error can bypass the sink, and if a future change makes
  `envelope_event_stream` (or a sibling wrapper) yield `Err`, this
  branch would silently swallow the terminal — the sink would see no
  `Error`/`FinalAnswer` and the turn would appear to end cleanly.
- Impact: low (dead code today). Maintainability trap if the envelope
  contract ever changes.
- Root cause: the branch predates the envelope normalization guarantee
  (or was written defensively against an unenforced contract).
- Direction: either (a) delete the `Err(e)` arm and document that
  `envelope_event_stream` is terminal-normalizing by contract, or (b)
  keep it but make it forward a synthesized `AgentEvent::Error` to the
  sink (mirroring the stream-setup path at `:526-534`) before returning
  `Err`, so the invariant holds even if the envelope contract regresses.
  Option (b) is the safer defensive choice.
- Regression validation: a test that asserts `envelope_event_stream`
  never yields `Err` over a mock stream emitting `Err(...)` items
  (already implicit in F-RCT-03 V01; could be made explicit here).
- Validation reports: [V03](../validations/A-CHAT-01/V03-01.md).

### A-CHAT-01-P3-02: No `drive_chat`-level cancel / steer / error fixtures — the one-terminal invariant is only regression-guarded on the happy path

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:593-1175` —
    the 9 unit tests cover: tool-mode visibility (3), happy-path
    streaming (`drive_chat_streams_agent_events_via_sink`, asserts
    `has_valid_contract` → exactly one terminal + identity contract),
    Task-mode run creation, projection spawn-boundary survival, mode-hint
    prepend, invocation-scoped tool exclusion, and `ChannelChatSink`
    forwarding.
  - The one-terminal assertion lives in `MockChatSink::has_valid_contract`
    (`:734-748`), exercised only by
    `drive_chat_streams_agent_events_via_sink` against a `MockLlmClient`
    that returns a single direct answer — i.e. the easiest terminal
    (`FinalAnswer`).
  - `grep -rn "fn .*cancel\|fn .*steer\|fn .*stream_error\|fn .*terminal"
    chat_driver.rs` returns no test named for cancel, steer, mid-stream
    error, or synthesized-terminal scenarios.
- Reachability: the cancel path (`ChatResources.cancel` →
  `execute_stream_message_with_invocation_context(msg, cancel, …)`) is
  live for every turn; the steer path is exercised via
  `agent.steer_input` (not through `drive_chat`, but feeding the same
  running turn). Neither has an app-level regression test against
  `drive_chat`.
- Expected invariant: per the task's V04, cancel / steer / stream-error
  fixtures should exist to guard the one-terminal invariant on
  non-happy paths.
- Observed behavior: the one-terminal guarantee on cancel / error relies
  entirely on (i) F-RCT-03's framework-level tests and (ii) the
  `envelope_event_stream` contract read in this review. There is no
  app-level test that cancels a `drive_chat` turn and asserts the sink
  received exactly one (synthesized `Error`) terminal, and no test that
  asserts a `MockLlmClient` failing mid-stream yields a sink terminal.
- Impact: a regression in `envelope_event_stream` (or in
  `drive_chat_inner`'s wiring of it) that broke the cancel/error
  terminal would not be caught by the app test suite; it would surface
  only as a UI "stream ended without response" symptom. Combined with
  F-RCT-03-P2-01 (droppable framework terminals) and F-RCT-03-P2-02
  (ReactAgent never emits `Cancelled`), the application layer is the
  last line of defense and currently has no guard.
- Root cause: the tests were written to lock in the construction-time
  invariants (run creation, mode hint, tool visibility) and the
  happy-path stream; the terminal-normalization behavior was treated as
  a framework concern.
- Direction: add three tests using `MockLlmClient`: (a) cancel
  mid-turn — `res.cancel.cancel()` after the first `ToolCall`, assert
  `MockChatSink` ends with exactly one `is_terminal()` event; (b)
  mid-stream error — a mock whose stream yields `Err`, assert the sink
  sees one `AgentEvent::Error`; (c) stream-setup failure — agent whose
  `execute_stream_message_with_invocation_context` returns `Err`, assert
  the sink sees the `source: "chat_driver"` Error. All three should
  reuse `has_valid_contract`'s "exactly one terminal" check.
- Regression validation: the three tests above are themselves the
  regression guard.
- Validation reports: [V04](../validations/A-CHAT-01/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Caller/reachability map: `drive_chat` is the single chat-turn lifecycle owner (definition + duplicate search + 4 callers) | yes | passed | [V01-01](../validations/A-CHAT-01/V01-01.md) |
| V02 | Sink responsibility diff: enumerate every `ChatSink` impl and classify render/transport vs business logic | yes | passed (with finding) | [V02-01](../validations/A-CHAT-01/V02-01.md) |
| V03 | One-terminal invariant: `drive_chat` delivers exactly one terminal per call across happy/error/cancel paths | yes | passed | [V03-01](../validations/A-CHAT-01/V03-01.md) |
| V04 | Stream error/cancel/steer fixtures: what happens on LLM error / cancel / steer mid-turn | yes | passed (static; no app-level fixture — see P3-02) | [V04-01](../validations/A-CHAT-01/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a | No prior A-CHAT-01 report under `zcode-glm/`; historical-claim classification is inline in the Inputs section. |

Executed cargo command (exit 0):

```text
cd echo-agent-cli
cargo test -p echo-agent-app-core --lib chat_driver::     (9 passed, 0 failed)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `chat_driver.rs:1-15` — "`drive_chat` is the single, thin entry for a chat turn across TUI / CLI channel / GUI" | current | V01 confirms 4 production callers, one definition, no sibling chat path; the task-run half (`drive_run_async`) is intentionally separate. |
| `chat_driver.rs:1-15` — "streams the agent's ReAct reply through a per-mode `ChatSink`, and stops" (implying sinks only render) | partially overstated | V02 confirms `TuiChatSink` / `ChannelChatSink` only render/transport, but `TauriChatSink` owns `ToolExecutionRepository` persistence authority — see A-CHAT-01-P2-01. |
| `chat_resources.rs:1-9` — "`drive_chat` scopes an `Arc<ChatResources>` per turn" | current | V01 confirms `with_chat_resources` is called inside `drive_chat_inner` (`chat_driver.rs:456`). |
| `chat_driver.rs:549-551` — "`Err` branch remains for future transport adapters that can fail independently" | current (dead today) | V03 confirms `envelope_event_stream` never yields `Err`, so the branch is unreachable — see A-CHAT-01-P3-01. |
| A-INP-01 handoff — "`drive_chat` consumes `PreparedUserTurn` as the input contract" | current | V01 confirms `drive_chat` takes `&PreparedUserTurn` and calls `turn.to_message()` once at `chat_driver.rs:435`. |
| F-RCT-03 handoff — "ReactAgent never emits `AgentEvent::Cancelled`; cancel ends the stream without a terminal" | current (load-bearing) | V03/V04 confirm `drive_chat` compensates via `envelope_event_stream`'s "ended without terminal" synthesis; the sink sees an `Error`, not `Cancelled`. |
| F-RCT-03 handoff — "framework returns raw `Result<AgentEvent>`; consumer wraps with `envelope_event_stream`" | current | V01/V03 confirm `drive_chat_inner:538` is the single `envelope_event_stream` wrap site in the chat path. |

## Coverage And Uncertainty

Inspected in full: `chat_driver.rs` (all 1175 lines), `chat_resources.rs`,
the three production `ChatSink` implementations (`ChannelChatSink`,
`TuiChatSink` `on_event` mapping, `TauriChatSink` struct +
`handle_tool_event` + `cancel_active_tools` + `ChatSink` impl),
`run_driver.rs` head (to confirm the task-run half is separate),
`envelope_event_stream` (the one-terminal guarantee), the four
`drive_chat` entry-point call sites, and the TUI cancel plumbing
(`handle_enter` → `app.active_cancel` → `handle_esc`/`Ctrl-C`).

Not inspected (out of scope or deferred):

- The `ensure_task_mode_run` / `finalize_task_mode_run` /
  `observe_execution_path` TaskRuntime interactions beyond their
  signatures and call order — A-TSK-* owns the store semantics. This
  task confirms they are invoked in the right order around
  `drive_chat_inner` and that they do not stream events themselves
  (they emit at most one `ExecEvent::run(...)` via `trace_sink`).
- The `TauriExecutionProjector` (`execution_projector`) internals — it
  consumes `ChatDriverEvent::Execution` events; its projection
  correctness is A-TSK / A-SRF-02 territory. This task confirms only
  that the sink forwards `Execution` events to it (`chat.rs:1396-1399`).
- The `subagent_trace_sink_for` / `framework_trace_sink_for` round-trip
  into the task-run executor — confirmed wired (P2-01's observer
  extraction would not touch these), but the subagent event projection
  is A-TSK / F-SUB-01.
- The `ToolExecutionRepository` persistence backend (SQLite vs file) —
  A-TSK / A-STATE-01 own it. This task only classifies that the
  repository is mutated from inside a sink.

Environmental constraints:

- `cargo test -p echo-agent-app-core --lib chat_driver::` ran against
  the existing incremental cache; 9 tests passed, exit 0. No feature
  matrix re-run (the chat driver is feature-independent — no
  `#[cfg(...)]` gates in `chat_driver.rs` outside the `#[cfg(test)]`
  module). Worktree clean at `b3b2e81`.

Uncertain claims:

- Whether any third path besides the GUI will want durable
  tool-execution history (e.g. a future TUI tool panel). The P2-01
  direction (extract to a driver-level observer) is justified by the
  AGENTS.md parity rule regardless of immediate demand, but the
  priority could be re-evaluated if product confirms TUI/CLI will never
  surface tool history.
- Whether the TUI/REPL/channels callers' lack of a post-`drive_chat`
  `TurnStatus { status: "cancelled" }` event (only Tauri emits one,
  `chat.rs:704-712`) is a real parity gap or an intentional "TUI/CLI
  render the synthetic Error themselves" decision. The synthetic
  `AgentEvent::Error` reaches every sink on cancel, so the terminal
  invariant holds; only the semantic label (cancelled vs error)
  differs. This is noted, not promoted to a finding — it belongs to
  A-SRF-01 / A-SRF-02 (surface parity), not to `drive_chat`.

## Handoff

Conclusions downstream tasks may rely on:

1. **`drive_chat` is the single chat-turn lifecycle owner.** Any task
   that needs to reason about "one chat turn" can treat
   `drive_chat` (`chat_driver.rs:202`) as the authoritative entry. There
   is no parallel chat driver; the task-run half (`drive_run_async` /
   `drive_agent_run`) is a separate, intentionally-decoupled lifecycle
   on isolated pool agents.
2. **The one-terminal invariant holds, via `envelope_event_stream`.**
   `drive_chat` delivers exactly one terminal `AgentEvent` to the sink
   per call (happy FinalAnswer, stream-setup Error, or synthesized
   "ended without terminal" Error). The invariant depends on the
   envelope wrapper at `chat_driver.rs:538`; any task that rewraps the
   stream must preserve `envelope_event_stream`'s normalization.
3. **Cancel and steer are NOT second lifecycles.** Cancel cancels the
   shared `CancellationToken` (one terminal synthesized); steer injects
   via `agent.steer_input` into the *running* turn (no new terminal, no
   new `drive_chat` call). Steer does bypass `drive_chat` by design
   (mid-turn injection), but it does not create a competing lifecycle.
4. **Sinks are NOT uniformly pure renderers.** `TauriChatSink` owns
   `ToolExecutionRepository` persistence (A-CHAT-01-P2-01); TUI /
   ChannelChatSink are pure render/transport. Any task that assumes
   "sinks only render" must qualify the GUI case.
5. **No app-level cancel/error/steer fixture exists** for `drive_chat`
   (A-CHAT-01-P3-02). The terminal invariant on those paths is guarded
   only by framework tests (F-RCT-03) + this static review.

Reports they must read:

- This report (A-CHAT-01) for the lifecycle-ownership and sink-split
  conclusions.
- `tasks/A-INP-01.md` for the `PreparedUserTurn` input contract and
  the TUI `/steer` input-box bypass (A-INP-01-P2-01) that this task
  treats as the steer mechanism.
- `tasks/F-RCT-02.md` and `tasks/F-RCT-03.md` for the upstream
  terminal/drop/cancel contract that `drive_chat` compensates for.
- `tasks/B-PATH-01.md` for the composition-root and entry-point
  inventory cross-reference.

Conditions that make this report stale:

- Any new caller of `drive_chat`, or any new code path that streams a
  chat turn via `execute_stream_*` without going through `drive_chat`,
  invalidates V01.
- Any change to `TauriChatSink` that moves tool-execution persistence
  out (resolving P2-01) invalidates V02's central finding.
- Any change to `envelope_event_stream`'s normalization (or removal of
  the wrap at `chat_driver.rs:538`) invalidates V03.
- Adding the cancel/error/steer fixtures recommended in P3-02
  invalidates V04's "no fixture" claim (and would upgrade V04 to
  executable evidence).
- A new `ChatSink` implementation must be added to the V02 sink table.

Follow-up task IDs (no fixes implemented in this review):

- A **tool-execution observer extraction** task — resolve
  A-CHAT-01-P2-01 by moving persistence from `TauriChatSink` into a
  driver-level observer supplied via `ChatResources`. Touches
  `chat_driver.rs`, `chat_resources.rs`, `tauri/commands/chat.rs`, and
  enables TUI/CLI/channels tool-history parity.
- A **chat-driver terminal regression** task — add the cancel /
  mid-stream-error / stream-setup-error fixtures recommended in
  A-CHAT-01-P3-02 (and decide P3-01: delete or fix the dead `Err`
  branch).
- **A-SRF-01 / A-SRF-02** — should pick up the sink-responsibility
  asymmetry and the TUI/CLI/channels missing `TurnStatus { "cancelled"
  }` semantic in their surface parity matrices.
