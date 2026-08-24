# A-CHAT-01: Shared chat driver and sinks

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both repositories clean

## Question

Does `drive_chat` own one application lifecycle while sinks only render/transport
events?

Answer: **mostly yes with one product-visible violation and one dead wire
variant.** `drive_chat` is the single application chat driver — it owns turn
identity, task-mode run creation/finalization, cancel registration, the
task_local `ChatResources` scope, and execution-path observation; the three
production sinks (`TuiChatSink`, `TauriChatSink`, `ChannelChatSink`) render or
transport events and hold no lifecycle/retry/state-transition authority. But
(a) `drive_chat`'s `Result` is decoupled from the agent-stream terminal — the
envelope yields only `Ok` items, so error-terminated turns return `Ok(())` and
the GUI wrapper then labels them `completed` (contradicting the error it also
renders) — and (b) `ChatDriverEvent::Interrupt` is a defined-and-consumed wire
variant with zero producers; the GUI emits its interrupt prompt outside the
shared sink. The framework defects F-RCT-03-P1-01/P1-02 surface at this
boundary as fabricated error messages on cancelled turns.

## Scope

- `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs` (full, 1175 lines:
  `ChatDriverEvent`, `ChatSink`, `drive_chat` :202-287, `drive_chat_inner`
  :425-569, `ChannelChatSink` :571-591, `WebhookTurnObserver` :83-176,
  tests :593-1175).
- Response types: `ChatDriverEvent` (chat_driver.rs:30-45), `EventEnvelope` +
  `envelope_event_stream` (`echo-agent/echo-core/src/agent/event_envelope.rs`,
  full), `AgentEvent::is_terminal` (`echo-core/src/agent/mod.rs:331-336`),
  `ExecEvent` entry points.
- Runtime integration: `chat_resources.rs` (full), the four entry adapters —
  TUI `src/tui/events.rs` (`dispatch_turn` :1341-1436, `send_to_agent`
  :2212-2230, `TuiChatSink` :2020-2210, event loop :640-908), GUI
  `src/tauri/commands/chat.rs` (`send_chat_message` :443-733,
  `steer_chat_message` :735-803, `cancel_chat` :807+, `TauriChatSink`
  :1141-1417, `agent_event_to_chat_event` :1448-1572), CLI REPL
  `src/cli/repl.rs` (`chat_with_agent` :477-849), channel
  `src/cli/channels.rs` (`handle_stream` :150-292,
  `aggregate_by_sentence` :515-654); task-runtime envelope consumers
  `executor.rs:3119-3130, 3734-3789` (entry points only).
- Steer: `AgentHandle::steer_input` (`echo-agent/src/agent/handle.rs:187`),
  EKO call sites TUI events.rs:4273, GUI chat.rs:780 (out-of-band control
  path, not through `drive_chat`).

## Out Of Scope

- Framework ReAct loop, streaming producer, terminal producer inventory,
  cancel terminal semantics, envelope internals → F-RCT-02/F-RCT-03
  (consumed as dependency facts).
- Steer mailbox correctness → F-RCT-05 (P1-02); steer identity divergence
  between surfaces → A-INP-01-P3-04 (cross-referenced, not re-filed).
- Task-runtime executor stream loops, run finalization, claims → A-TSK-01/03/04.
- TUI rendering completeness → A-SRF-01; Tauri command lifecycle → A-SRF-02;
  frontend reducers → A-SRF-03; CLI/channel cancel & shutdown → A-SRF-04.
- PreparedUserTurn/attachment normalization → A-INP-01 (dependency report).
- Frontend tool-card rendering of `ToolExecutionRepository` projections →
  A-FE-01/02.

## Inputs

- Root `AGENTS.md` (surface parity, "X doesn't use Y is a gap", UTF-8/panic
  safety, layering gate, no parallel semantics), shared `README.md`,
  `REPORTING.md`, `TASKS.md` (A-CHAT-01 card), `zcode-ds/README.md`, report
  templates.
- Dependency task reports read (zcode-ds): `A-INP-01` (complete; six
  `PreparedUserTurn::build` call sites, steer identity P3-04, TUI steer
  attachment loss P1-01), `F-RCT-02` (complete; P1-01 non-streaming `Ok("")`,
  P2-04 FinalAnswer-before-continuation), `F-RCT-03` (complete; P1-01 dropped
  terminal errors under backpressure, P1-02 cancel never emits `Cancelled`,
  P2-02 second FinalAnswer masked by the envelope, P2-04 abandoned streams
  leave trace Running).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md`
  (lines 93, 128, 302, 341, 371, 407, 431, 440, 454, 770, 775),
  `docs/PROJECT-ANALYSIS.md` (lines 13-29, 244),
  `echo-agent-cli/docs/2026-07-28-app-core-full-audit.md` (no drive_chat
  claims found).

## Layering Decision

- Generic mechanism (framework): `envelope_event_stream`/`EventEnvelope`,
  the `AgentEvent` terminal contract, `AgentHandle::steer_input` — already
  framework-owned, reused as-is; no movement recommended.
- EKO product policy (application): `drive_chat`, `ChatDriverEvent`,
  `ChatSink`, `ChatResources`, interaction-mode/tool-visibility policy,
  task-mode run lifecycle, the GUI `TurnStatus` lifecycle wrapper — all
  correctly placed in the application. All findings in this report stay in
  the application layer (or the adapter boundary); none recommend framework
  movement.
- Adapter boundary: the four entry adapters are thin and uniform on the main
  path (each builds `PreparedUserTurn` + `ChatResources` and calls
  `drive_chat`); two divergences: the GUI wrapper invents a product turn
  lifecycle (`TurnStatus`) computed from the cancel token and `drive_chat`'s
  Result (P1-01), and the GUI interrupt detection emits
  `ChatEvent::InterruptPrompt` directly, bypassing the shared sink (P2-01).
- Duplicate-search terms and results (V01-01): `drive_chat`, `ChatSink`,
  `ChatDriverEvent`, `ChatDriverEvent::Interrupt` producers,
  `envelope_event_stream`, `execute_stream*`, `steer_input`,
  `AgentEvent::Cancelled` producers, `find_in_progress_run_by_conversation`,
  `worker` (in the touched files). Results: one chat driver definition, four
  live call sites, three production sinks, no parallel chat driver in either
  repository; `ChatDriverEvent::Interrupt` has zero producers; framework
  `Cancelled` producers are the default `cancel_aware_stream` wrapper (which
  ReactAgent overrides away) and the subagent executor only; no `worker`
  terminology in the touched application files.

## Current Path

Verified call graph (V02-01, V03-01):

1. TUI: `dispatch_turn` (events.rs:1341) → `PreparedUserTurn::build` →
   `start_turn` → `send_to_agent` spawns `drive_chat` (events.rs:2225-2229);
   events loop clears `is_processing` on `FinalAnswer`/`Error`/`Cancelled`/
   `TurnStatus` (events.rs:657-675, 805-843, 880-886); Ctrl+C →
   `handle_esc` fires `active_cancel` (events.rs:1937-1958).
2. GUI: `send_chat_message` (chat.rs:443) — interrupt detection (:516-534,
   direct `ChatEvent::InterruptPrompt`), turn-busy + cancel-token registries
   (:536-566), HITL attach (:570-589), `TauriChatSink` (:609-617),
   `TurnStatus running` (:620), prepared turn (:636-663), spawned
   `drive_chat` (:681-725); terminal status derived from cancel token +
   `drive_chat` Result (:690-696) and emitted as `TurnStatus` (:709-711).
3. CLI REPL: `chat_with_agent` (repl.rs:477) → spawned `drive_chat`
   (:542-544), consumes the channel, awaits the drive task (:840-849);
   fresh token never fired — no cancel path.
4. Channel: `handle_stream` (channels.rs:150) → spawned `drive_chat`
   (:237-265) into `ChannelChatSink`; `aggregate_by_sentence` flushes on
   FinalAnswer/Cancelled, errors on Error (:563-575).
5. Driver: `drive_chat` (chat_driver.rs:202-287) scopes run_id
   (`formal_run_id_for_turn`), creates/finalizes the Task-mode formal run
   (:289-383), registers cancel + projection (:240-257), observes the
   execution path (:273-279, :385-422); `drive_chat_inner` (:425-569) builds
   the invocation context (tool visibility per mode, `EventIdentity`,
   `ExternalRunContext`), calls
   `execute_stream_message_with_invocation_context` (:512-514), wraps with
   `envelope_event_stream` (:538) and forwards envelopes to `sink.on_event`,
   breaking when the sink returns false. Setup failure emits an Error
   envelope and returns `Err` (:515-536); the stream-loop `Err` branch
   (:548-561) is unreachable — the envelope converts every raw `Err` and
   terminal-less end into an `AgentEvent::Error` payload and never yields
   `Err` (event_envelope.rs:134-191).
6. Terminal contract at the EKO boundary: the envelope yields exactly one
   terminal (`FinalAnswer | Cancelled | Error`, mod.rs:331-336) per turn
   (truncate at first, fabricate on terminal-less end); the driver forwards
   it; the GUI additionally emits a second, wrapper-invented terminal
   (`TurnStatus` + `ChatEvent::Done`).
7. Task runtime (separate consumer, not a chat driver): executor.rs:3119-3130
   (formal-run main agent) and :3734-3789 (background run agent) consume the
   framework streaming API through the envelope with their own loops and
   ExecEvent trace sinks; `ChatDriverEvent::Execution` is produced only by
   `subagent_trace_sink_for`/`framework_trace_sink_for`
   (chat_driver.rs:59-81).

## Findings

### A-CHAT-01-P1-01: `drive_chat`'s Result is decoupled from the agent-stream terminal — the GUI labels error-terminated turns "completed" and user cancellations render a fabricated error alongside the cancel status

- Priority: P1
- Confidence: high (static chain fully verified; frontend rendering of the
  contradictory signals is A-SRF-02/03 scope and inferred)
- Layer: application
- Evidence:
  - `echo-agent/echo-core/src/agent/event_envelope.rs:134-140` — every raw
    `Err` item becomes an `Error` payload; `:173-191` — stream breaks at the
    first terminal and fabricates
    `Error{"agent stream ended without a terminal event"}` on terminal-less
    ends; the envelope never yields `Err`.
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:540-561` —
    `drive_chat_inner` forwards envelope items and returns `Ok(())` after the
    loop; the `Err(e)` branch (:548-561, "remains for future transport
    adapters") is unreachable today; `:515-536` is the only `Err` path
    (stream setup failure, which also emits an Error envelope first).
  - `echo-agent-cli/src/tauri/commands/chat.rs:690-696` — GUI terminal status:
    `cancelled` if the cancel token fired, else `completed` if
    `outcome.is_ok()`, else `failed`; `:709-711` emits `TurnStatus`.
  - `echo-agent-cli/src/tauri/commands/chat.rs:1563-1565` — the GUI maps
    `AgentEvent::Error` to `ChatEvent::Error` (rendered to the user); the
    TUI, by contrast, masks it via `active_cancel.is_cancelled()`
    (`src/tui/events.rs:805-841` renders "Cancelled by user.").
- Reachability: every streaming chat turn on every surface; the mislabeling
  triggers on any envelope-normalized error terminal — provider/mid-stream
  error, NoResponse, max-iterations, tool-batch timeout (F-RCT-03-P1-01
  drops the specific Err under backpressure, so the generic fabricated
  message arrives instead), and every cancelled turn (F-RCT-03-P1-02: cancel
  ends the stream as NoResponse Err or a terminal-less abandon — the main
  loop never emits `Cancelled`).
- Expected invariant: one truthful terminal per turn; the product lifecycle
  terminal agrees with the agent stream terminal; cancellation is
  distinguishable from failure and never surfaces as an error.
- Observed behavior: `drive_chat` returns `Ok(())` for every
  envelope-normalized outcome including `Error` terminals; the GUI then
  reports `TurnStatus("completed")` + `Done` for error-terminated turns while
  the `ChatEvent::Error` is simultaneously rendered — two contradictory
  terminal signals. On user cancel the frontend receives a fabricated
  `ChatEvent::Error` ("agent_stream: agent stream ended without a terminal
  event" or "no response…") followed by `TurnStatus("cancelled")`.
- Impact: the GUI lifecycle status lies about the turn outcome (error turns
  marked completed — misleading success, same class as F-RCT-02-P1-01 at the
  EKO layer); cancelled turns show an error message a user cannot act on;
  any future consumer of `drive_chat`'s Result inherits the same decoupling.
- Root cause: the envelope normalizes every terminal into a payload, so the
  driver's `Result` carries no outcome information; the GUI wrapper
  substituted its own status derivation (cancel token + Result) for the
  stream terminal instead of reading the last envelope's payload.
- Direction: make `drive_chat` return a typed terminal outcome (e.g.
  inspect the last forwarded envelope — `FinalAnswer` vs `Error` vs
  cancel-token-fired — and return a `TurnOutcome`), and derive the GUI
  `TurnStatus` from it; alternatively have the GUI compute the status from
  the last agent event it forwarded rather than from the Result. Align with
  the F-RCT-02-P1-01 and F-RCT-03-P1-01/P1-02 fixes (specific error
  forwarding, `Cancelled` terminal) so the fabricated message disappears.
- Regression validation: driver-level tests — (a) cancelled-token turn
  through `drive_chat` yields `TurnOutcome::Cancelled` (or, at minimum, the
  driver Result distinguishes cancel); (b) an `Error`-terminal turn yields a
  failed outcome; (c) GUI-level fixture asserting `TurnStatus` agrees with
  the last agent event (completed only after FinalAnswer).
- Validation reports: [V02-01](../validations/A-CHAT-01/V02-01.md),
  [V03-01](../validations/A-CHAT-01/V03-01.md)

### A-CHAT-01-P2-01: `ChatDriverEvent::Interrupt` is a dead wire variant — consumed by all four sinks, produced by none; the GUI emits its interrupt prompt outside the shared sink

- Priority: P2
- Confidence: high
- Layer: application (shared wire contract) / adapter (GUI)
- Evidence:
  - Definition: `chat_driver.rs:40-45` (`ChatDriverEvent::Interrupt`);
    consumers: TUI events.rs:2045-2053, GUI chat.rs:1401-1414, REPL
    repl.rs:588-596, channel channels.rs:641-649; also serialized in the
    surface-contract test (surface_contract.rs:198-203).
  - Producers: repository-wide grep — zero production constructions of
    `ChatDriverEvent::Interrupt`; the GUI interrupt detection
    (chat.rs:516-534) emits the Tauri-layer `ChatEvent::InterruptPrompt`
    directly via `emit_chat_event`, bypassing `drive_chat` and the sink;
    `find_in_progress_run_by_conversation` has a single production caller,
    GUI-only (chat.rs:518; store.rs:1513).
- Reachability: none for the variant (defined + registered in every sink
  match but never produced); the GUI interrupt UX itself is reachable and
  works (direct emission); TUI/REPL/channel have no interrupt prompt — a
  parallel chat turn is silently started while a run is in progress.
- Expected invariant: the shared wire contract is exhaustive in both
  directions (every variant produced and consumed); all surfaces expose the
  same interrupt/resume policy (AGENTS.md surface parity); MASTER-PLAN:770
  ("interrupt 不再通过默认 no-op 静默丢失").
- Observed behavior: the Interrupt variant is dead; the GUI's
  InterruptPrompt does not travel through `drive_chat`; the other three
  surfaces never surface an in-progress-run interruption.
- Impact: misleading shared contract (dead variant kept alive by consumers);
  surface parity gap — a paused/attended run in progress prompts only the
  GUI user, while TUI/channel/REPL users get a concurrent second turn with no
  prompt.
- Root cause: the GUI interrupt path predates the shared sink contract and
  was never migrated; the variant was added to the contract as a target
  shape, and the migration was never completed.
- Direction: route the GUI interrupt detection through the shared path
  (produce `ChatDriverEvent::Interrupt` via the sink — e.g. in
  `send_chat_message` before the turn starts, or let `drive_chat` emit it),
  and add the same detection to TUI/REPL/channel send paths (one shared
  helper over `find_in_progress_run_by_conversation`); if the GUI-only direct
  emission is kept deliberately, delete the dead variant from
  `ChatDriverEvent` and the four sink arms.
- Regression validation: a fixture driving an in-progress run + new message
  through each surface adapter, asserting a single Interrupt event on the
  shared sink (or, if deleted, grep `ChatDriverEvent::Interrupt` returns only
  the definition removal).
- Validation reports: [V01-01](../validations/A-CHAT-01/V01-01.md),
  [V02-01](../validations/A-CHAT-01/V02-01.md),
  [V05-01](../validations/A-CHAT-01/V05-01.md)

### A-CHAT-01-P2-02: Sink responsibility asymmetry — `TauriChatSink` owns durable tool-execution projection state and guesses tool cancellation from terminal events; the other sinks are stateless

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `src/tauri/commands/chat.rs:1148-1156` — `TauriChatSink` fields:
    `tool_executions` (`ToolExecutionRepository`, durable),
    `tool_completions`, `active_tool_ids`; `:1193-1331` — `handle_tool_event`
    persists start/append_output/finish per call, tracks in-flight call ids,
    and on `Cancelled`/`Error` (:1325-1328) or non-running `TurnStatus`
    (:1365-1368) calls `cancel_active_tools` (:1171-1191), which cancels the
    persisted records for all in-flight tools.
  - Contrast: `TuiChatSink` (events.rs:2031-2209) and `ChannelChatSink`
    (chat_driver.rs:585-591) are stateless; the TUI keeps the equivalent tool
    card state in `TuiApp.messages` (events.rs:676-804) in the main loop, the
    channel in the aggregator buffer (channels.rs:528-575).
- Reachability: every GUI chat turn with tool calls; the cancel-on-terminal
    heuristic fires whenever an `Error` terminal (including the fabricated
    terminal-less error) or a non-running `TurnStatus` arrives — including
    error terminals that are NOT user cancels (provider failure, batch
    timeout).
- Expected invariant: sinks only render/transport events (task question);
    projection state derived from events, never a second authority guessing
    outcomes.
- Observed behavior: the GUI sink both renders and persists tool-execution
    projections, and marks in-flight tool records "cancelled" whenever any
    terminal/status transition occurs — even when the tool actually ran to
    completion (e.g. F-RCT-04-P1-02's batch-timeout turn end without a typed
    terminal is indistinguishable from a real cancel).
- Impact: tool cards can show "cancelled" for tools that completed
    (projection fidelity); the driver has no visibility into this state, so
    the GUI's tool projection is a second, sink-owned authority that can
    diverge from the event stream; the sink trait's "render/transport"
    contract is stretched by one implementation.
- Root cause: the GUI sink accreted durable projection state and
    terminal-driven cleanup before the driver/sink split was finalized; the
    other surfaces keep the same facts in UI-local state.
- Direction: keep durable tool-execution persistence in the GUI projection
    layer (it serves tool cards) but base "cancelled vs finished" on the
    event stream (pair `ToolStream::Complete`/`ToolResult`/`ToolError`), not
    on any terminal arriving later; document the sink contract as "render +
    surface-projection state allowed, lifecycle authority not allowed".
- Regression validation: GUI fixture — provider-error terminal after a
    completed tool: tool card must stay "finished", not "cancelled";
    cancel-after-tool-completion: card "cancelled" only for genuinely
    in-flight calls.
- Validation reports: [V03-01](../validations/A-CHAT-01/V03-01.md)

### A-CHAT-01-P3-01: Stale documentation — `main.rs:31` describes a nonexistent `drive_chat` signature; PROJECT-ANALYSIS:13-23 documents the old ChatSink trait shape with a forbidden "worker" term

- Priority: P3
- Confidence: high
- Layer: application (docs)
- Evidence:
  - `echo-agent-cli/src/main.rs:31` — "because `drive_chat` takes
    `Option<&TaskRuntimeStore>`"; current signature is
    `drive_chat(agent: &AgentHandle, turn: &PreparedUserTurn,
    res: Arc<ChatResources>)` (chat_driver.rs:202-206).
  - `docs/PROJECT-ANALYSIS.md:13-23` — old sink trait shape
    ("on_agent_event(event) -> bool, 其余 on_run_status/on_worker_trace/
    trace_sink() 等可选"; current trait is a single
    `on_event(ChatDriverEvent) -> bool`, chat_driver.rs:53-56) and old
    signature (:22); anchor line numbers stale (:17-19 vs current 1341/688/
    2226/262); the doc uses the forbidden `worker` term (:23).
- Reachability: n/a (docs).
- Expected invariant: repository docs describe current APIs and use Subagent
  terminology (AGENTS.md).
- Observed behavior: three stale claims in two files.
- Impact: minor — misleading reader of the driver contract; terminology
  drift.
- Root cause: documentation not updated when the driver signature and sink
  trait were converged.
- Direction: update main.rs:31 and PROJECT-ANALYSIS:13-29 to the current
  signature/trait/call-site anchors; remove the `worker` term.
- Regression validation: grep the two files for the corrected anchors after
  the edit.
- Validation reports: [V05-01](../validations/A-CHAT-01/V05-01.md)

### A-CHAT-01-P3-02: No driver-level stream error/cancel/steer fixtures — the Ok(()) mislabeling and the Interrupt gap are untested

- Priority: P3
- Confidence: high
- Layer: application (tests)
- Evidence: `chat_driver.rs` test module (:593-1175) contains 9 tests, all
  happy path: one-terminal contract via `has_valid_contract`
  (:734-748,836-895), task-mode run creation (:758-833), projection across
  spawn (:897-996), mode hint (:998-1064), tool exclusion (:1066-1122),
  channel-sink forwarding (:1124-1174). No test fires a
  `CancellationToken`, no test feeds an `Error` terminal or a terminal-less
  stream, no sink-returns-false test, no steer test at the driver level;
  `MockLlmClient` (F-TST-01-P1-01) emits content+usage in a single chunk, so
  streaming-order defects cannot be reproduced here either.
- Reachability: n/a (test inventory).
- Expected invariant: the required validation "stream error/cancel/steer
  fixtures" has executable fixtures (task card).
- Observed behavior: the fixtures do not exist; the P1-01 mislabeling and
  the P2-01 dead variant have no regression tests.
- Impact: P1-01/P2-01 can silently regress; the shared driver's error/cancel
  behavior is certified only by the happy path.
- Root cause: the driver tests were written when only the happy path was
  exercised; cancel/error fixtures were never added.
- Direction: add driver-level tests per P1-01's regression validation, a
  sink-returns-false test (driver stops forwarding, turn still terminates
  truthfully), and a steer-identity test at the GUI/TUI adapter boundary
  (with the F-RCT-05 fix).
- Regression validation: the new fixtures themselves (see P1-01).
- Validation reports: [V03-01](../validations/A-CHAT-01/V03-01.md),
  [V04-01](../validations/A-CHAT-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search across both repositories (driver, sinks, wire variants, envelope, cancel producers, steer, worker terms) | yes | passed | [V01-01](../validations/A-CHAT-01/V01-01.md) |
| V02 | Registration and runtime reachability (four entry adapters -> drive_chat -> envelope -> sinks; task runtime as separate consumer) | yes | passed | [V02-01](../validations/A-CHAT-01/V02-01.md) |
| V03 | Invariant/edge cases (sink responsibility diff; one-terminal invariant at envelope and product boundary; stream error/cancel/steer fixture inventory) | yes | passed | [V03-01](../validations/A-CHAT-01/V03-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked chat_driver`; `... surface_contract`; `cargo check --workspace --locked` | yes | passed (exit 0 / 0 / 0; 9 + 3 tests passed) | [V04-01](../validations/A-CHAT-01/V04-01.md), [V04-02](../validations/A-CHAT-01/V04-02.md), [V04-03](../validations/A-CHAT-01/V04-03.md) |
| V05 | Historical-document drift (MASTER-PLAN drive_chat/ChatSink/interrupt claims; PROJECT-ANALYSIS anchors) | conditional | passed | [V05-01](../validations/A-CHAT-01/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:93 "GUI/TUI/channel 已通过 drive_chat 统一驱动,差异主要在 ChatSink" | current | four live call sites, three sinks (V02-01) |
| MASTER-PLAN:431 "CLI REPL 绕过 drive_chat" | fixed | repl.rs:543 calls drive_chat |
| MASTER-PLAN:770 "ChatSink 收敛为单一、穷尽的 ChatDriverEvent 入口 … interrupt 不再静默丢失" | regressed (interrupt) | `ChatDriverEvent::Interrupt` has zero producers; GUI emits ChatEvent::InterruptPrompt directly (P2-01) |
| MASTER-PLAN:341/454/371 "交互入口消费同一 ChatDriverEvent; GUI/TUI/CLI/channel 共用 drive_chat; M2 已完成" | current | V02-01 |
| PROJECT-ANALYSIS:13-19 "drive_chat 单一驱动点,差异只在 ChatSink" | current (semantics); stale (line anchors) | anchors moved (V05-01) |
| PROJECT-ANALYSIS:22 drive_chat signature `(agent, message, multimodal, res)` | stale | current signature `(agent, &PreparedUserTurn, Arc<ChatResources>)` |
| PROJECT-ANALYSIS:23 old ChatSink trait shape (`on_agent_event` + optional callbacks, `on_worker_trace`) | stale | single `on_event(ChatDriverEvent) -> bool` (chat_driver.rs:53-56) |
| main.rs:31 "drive_chat takes Option<&TaskRuntimeStore>" | stale | ChatResources-based signature (P3-01) |

## Coverage And Uncertainty

- All conclusions are static except three test/compile runs (V04); no
  dynamic end-to-end turn exercised cancellation, an error terminal, or a
  slow sink. The P1-01 chain (envelope yields only Ok -> driver returns
  Ok(()) -> GUI labels completed) is a fully verified static proof; the
  exact frontend rendering of the contradictory signals (Error event +
  TurnStatus "completed") is A-SRF-02/A-SRF-03 scope and inferred.
- The Tauri `gui` feature bin and the `channels` feature were not compiled
  in this task (default workspace check passed; Q-GUI-01/Q-CLI-01 own the
  conditional matrix). The channel aggregator's behavior after a dropped
  terminal error (F-RCT-03-P1-01) is derived from the envelope contract, not
  dynamically reproduced.
- The task runtime's second envelope consumer (executor.rs:3119-3130,
  3734-3789) was read only at entry points; its terminal/claim handling is
  A-TSK-01/03/04 scope.
- F-RCT-05-P1-02 (steer mailbox lease keyed by turn_id vs drained by
  current_run_id) is cross-referenced for the steer identity consequences
  (A-INP-01-P3-04), not re-verified.
- The REPL and channel surfaces create a `CancellationToken` that is never
  fired — chat turns on those surfaces are not cancellable; classified as a
  surface-parity gap owned by A-SRF-04 (cancel/shutdown fixtures), noted
  here for its handoff.
- `WebhookTurnObserver::finish` emits ChatCompleted only after FinalAnswer;
  cancel/error turns emit no completion webhook — consistent behavior, not
  filed.

## Handoff

- Downstream tasks may rely on: one application chat driver with four live
  call sites and three sinks (V01/V02); the envelope boundary enforces one
  agent terminal per turn; `drive_chat`'s Result carries no outcome
  information (P1-01); `ChatDriverEvent::Interrupt` is dead (P2-01);
  TauriChatSink's tool-projection state (P2-02); green driver tests at the
  reviewed commits (V04).
- A-SRF-02/A-SRF-03 must reconcile the GUI's `TurnStatus`/Done lifecycle with
  the agent terminal payload (P1-01); A-SRF-01 should verify the TUI state
  clearing stays correct when cancel/error terminal semantics change
  (F-RCT-03 fixes); A-SRF-04 must add cancel wiring for REPL/channel turns
  and interrupt prompts; X-EVT-01 should include the GUI TurnStatus-vs-agent-
  terminal contradiction and the dead Interrupt variant in its terminal
  conformance matrix; X-SRF-01 should add a parity row for in-progress-run
  interruption; Q-E2E-01 scenarios: cancel a GUI turn and assert no
  fabricated error + status cancelled; provider-error turn asserting status
  failed.
- Reports to read: this report + V01-01..V05-01; A-INP-01 (steer/attachment
  chain), F-RCT-02 (P1-01/P2-04), F-RCT-03 (P1-01/P1-02/P2-02/P2-04).
- Stale triggers: any change to `chat_driver.rs` (driver, sink trait,
  ChatDriverEvent variants, `drive_chat_inner`), `event_envelope.rs`
  normalization, `chat_resources.rs`, the four adapter call sites
  (events.rs:2226, chat.rs:688, repl.rs:543, channels.rs:262), GUI
  `send_chat_message`/`steer_chat_message`/`cancel_chat`, or the TauriChatSink
  tool projection invalidates the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): A-SRF-02,
  A-SRF-03, A-SRF-04, X-EVT-01, X-SRF-01, Q-E2E-01, Q-FLT-01 (driver-level
  cancel/error fault scenarios).
