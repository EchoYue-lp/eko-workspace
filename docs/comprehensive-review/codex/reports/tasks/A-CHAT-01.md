# A-CHAT-01: Shared chat driver and sinks

> Status: complete
> Reviewer: Codex primary reviewer (delegated evidence independently sampled)
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean before report creation; only
> Codex A-CHAT-01 reports were added

## Question

Does `drive_chat` own exactly one EKO chat lifecycle while GUI, TUI, CLI, and
channel sinks only transport or render complete canonical events?

## Scope

- `echo-agent-app-core/src/chat_driver.rs`, ChatResources, response/event types,
  framework envelope adapter usage, trace bridges, and WebhookTurnObserver.
- Production GUI, TUI, CLI REPL, and IM channel callers and sinks, including
  frontend terminal consumption needed to prove impact.
- Caller/reachability map, sink responsibility diff, one-terminal and ordering
  state table, stream error/cancel/steer/disconnect behavior, and static tests.

## Out Of Scope

- Framework terminal persistence, Stop continuation, EOF fallback, DirectAnswer
  double-terminal/deadlock, channel loss, draft/reasoning semantics, and
  upstream disconnect cancellation are owned by [F-RCT-02](F-RCT-02.md) and
  [F-RCT-03](F-RCT-03.md). This task consumes their facts without duplicating
  their finding IDs.
- Prepared-turn/attachment identity and live TUI steer normalization are owned
  by [A-INP-01](A-INP-01.md). A-INP-01-P1-02 is used as a concrete pre-stream
  failure source; A-INP-01-P1-03 remains the steer finding.
- Full GUI/TUI/CLI/channel feature review, frontend reducer replay, webhook
  observability, TaskRuntime state machine correctness, source fixes, and
  roadmap design.
- Cargo, rustc, tests, builds, dynamic fixtures, and network activity, all
  prohibited for this review.

## Inputs

- Root `AGENTS.md`; shared review `README.md`, `REPORTING.md`, exact A-CHAT-01
  task card in `TASKS.md`; Codex isolation protocol and templates.
- Codex dependency reports [A-INP-01](A-INP-01.md),
  [F-RCT-02](F-RCT-02.md), and [F-RCT-03](F-RCT-03.md).
- Current clean source and scoped Git history. No other reviewer directory was
  read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Framework Agent streams must eventually expose one typed terminal and lossless/cancellable delivery. Those defects stay with F-RCT. |
| EKO product policy | Task/Auto mode setup, formal TaskRun projection, per-surface rendering, queue admission, tool-detail persistence, webhook projection, and GUI Done belong in `echo-agent-cli`. |
| Adapter boundary | `drive_chat` should convert the framework terminal into one typed EKO outcome after all EKO postlude work. Sinks may render/transport but cannot infer completion, suppress canonical events because persistence failed, or independently advance the next turn. |
| Duplicate search | Searched both repositories for `drive_chat`, `execute_stream_message`, `ChatSink`, `ChatDriverEvent`, Agent stream consumers, terminal status, Done, and sink implementations. One shared EKO driver is live; lifecycle authority is nevertheless fragmented across driver/callers/sinks. |
| Migration deletion | Retain one `drive_chat` and one canonical outcome. Delete caller-synthesized terminal status, TUI terminal-specific queue dispatch, sink-local terminal inference, and persistence-gated event suppression after cutover. |

No EKO surface policy should move into the framework. Conversely, framework
terminal bugs must be fixed there rather than patched independently by every
EKO sink.

## Current Path

```text
GUI/TUI/CLI/channel
  -> PreparedUserTurn + ChatResources
  -> drive_chat
     -> optional formal Task-mode run + cancellation/projection registrations
     -> drive_chat_inner
        -> PreparedUserTurn::to_message
        -> framework execute_stream_message_with_invocation_context
        -> envelope_event_stream (identity, sequence, one visible terminal)
        -> WebhookTurnObserver + ChatSink::on_event
     -> Task-mode finalize + ExecutionPath
     -> Result<(), String>
  -> caller/sink-specific lifecycle completion
```

Production reachability is converged:

| Surface | Driver caller | Sink/consumer | Caller-owned terminal behavior |
|---|---|---|---|
| GUI | `chat.rs:681-725` | TauriChatSink -> ChatEvent -> frontend | Emits running before build; after `drive_chat`, maps cancel token/`Result` to status; non-running status causes sink to emit Done. |
| TUI | `events.rs:2212-2229` | TuiChatSink -> TUI event loop | Agent FinalAnswer/Error/Cancelled mutate UI directly; no caller TurnStatus; driver error only logged. |
| CLI REPL | `repl.rs:490-544` | ChannelChatSink -> local render loop | Renders Agent events, then separately joins/logs driver result. |
| IM channel | `channels.rs:190-291` | ChannelChatSink -> aggregate_by_sentence | Renders selected facts; driver error only logged in detached producer. |

The framework envelope wrapper stops after the first Agent terminal and creates
an Error on raw error/EOF (`event_envelope.rs:107-193`). That does not make
`drive_chat`'s `Ok(())` a successful semantic outcome: the driver discards the
terminal kind and returns transport-loop success (`chat_driver.rs:538-565`).

### Static lifecycle table

| Scenario | Agent event seen by sink | `drive_chat` result | GUI status/Done | TUI | CLI | Channel |
|---|---|---|---|---|---|---|
| FinalAnswer | FinalAnswer(data) | Ok | completed + Done | finalizes buffered Tokens; ignores data | ignores data | flushes Tokens; ignores data |
| Cancelled | Cancelled | Ok | cancelled + Done if token set, else completed | clears cancel but not active turn/queue | warning | flush + close |
| Agent Error/raw EOF | Error | Ok | **completed + Done** | error/release/queue | prints error | stream error |
| Stream setup failure | Error synthesized by driver | Err | failed + Done | error event then log | error event then join log | stream error then producer log |
| Pre-stream application failure | none | Err | caller sends failed + Done | **log only/stuck** | **join log only** | **silent close/log** |
| Sink closes | delivery stops | Ok | caller may complete | channel dropped | receiver dropped | outbound consumer dropped |

## Findings

### A-CHAT-01-P1-01: `drive_chat` collapses Agent failure/cancellation into transport success and GUI reports completion

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:515`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:538`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:563`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:687`
- Reachability: every GUI normal turn consumes this driver. Framework envelope
  Error, Cancelled, and FinalAnswer are delivered through the same loop, after
  which `drive_chat_inner` returns `Ok(())` regardless of terminal kind.
- Expected invariant: one typed EKO outcome preserves completed/failed/cancelled/
  disconnected and agrees with the terminal event, TaskRun, webhook, and UI.
- Observed behavior: GUI maps `Ok` to completed unless its local cancel token is
  set. Therefore an Agent Error or synthesized EOF Error is followed by
  RunStatus(completed) and Done. Task mode may separately transition its formal
  run to Failed, creating an explicit contradiction.
- Impact: the GUI can replace/show failure then declare the same turn completed;
  queueing, diagnostics, TaskRun projection, and user trust see inconsistent
  outcomes.
- Root cause: `Result<(), String>` describes stream-adapter execution, not the
  Agent terminal, but callers treat it as semantic success.
- Direction: make `drive_chat` return one typed application `ChatTurnOutcome`
  derived from the terminal after EKO postlude work; publish status/Done once
  from that outcome. Delete caller inference from `Result` and cancel-token
  sampling. Do not create a second framework terminal state machine.
- Regression validation: FinalAnswer, Cancelled, Agent Error, raw EOF, setup
  error, sink close, and Task-mode postlude with exact agreement across Agent
  event, EKO outcome, TaskRun, webhook, and GUI status.
- Validation reports: [V03](../validations/A-CHAT-01/V03-01.md)

### A-CHAT-01-P1-02: Failures before stream creation have no shared sink terminal and leave non-GUI surfaces inconsistent

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:225`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:240`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:435`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:2225`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/channels.rs:262`
- Reachability: Task mode store creation/registration and all inline attachment
  `to_message` reads run before framework stream setup. A-INP-01-P1-02 makes a
  missing/replaced attachment read a concrete production failure.
- Expected invariant: every accepted turn ends with one product terminal event
  and releases active/queued state equivalently across surfaces.
- Observed behavior: these failures return `Err` without calling the sink.
  GUI happens to synthesize failed TurnStatus; TUI only logs and leaves
  `is_processing`/active IDs set; CLI logs after its channel closes; IM channel
  closes without a user-visible error. The driver can still emit ExecutionPath
  after its error result.
- Impact: TUI can become stuck and fail to advance queued turns, while CLI and
  channel users get silence or incomplete output for the same failure.
- Root cause: accepted-turn terminal ownership is split: some errors are driver
  events, some are raw returns, and only one caller repairs them.
- Direction: route all post-admission exits through the one typed driver outcome
  and terminal publisher, including pre-stream failures; sinks only transport
  it. Delete caller-specific repair and post-error ExecutionPath emission unless
  it is part of the ordered non-terminal facts before terminal.
- Regression validation: fault formal run create, cancellation registration,
  attachment read, stream setup, and postlude across all surfaces; assert one
  visible failure and released state.
- Validation reports: [V04](../validations/A-CHAT-01/V04-01.md)

### A-CHAT-01-P1-03: TUI, CLI, and channels discard the committed FinalAnswer payload

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:657`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/repl.rs:807`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/channels.rs:563`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/react/run/stream_channel.rs:136`
- Reachability: all three live consumers match `AgentEvent::FinalAnswer` and
  ignore its data. Framework's live input-guard branch emits a non-empty
  FinalAnswer without any Token event.
- Expected invariant: FinalAnswer(data) is the authoritative committed response;
  Token chunks may optimize provisional rendering but cannot be the sole source
  of user-visible content.
- Observed behavior: TUI finalizes only `streaming_text`, CLI merely clears the
  spinner, and channel only flushes Token buffer. With guard-block or any
  terminal-only response, they display/send no answer. GUI correctly uses data.
- Impact: identical Agent behavior produces a complete answer in GUI and an
  empty response in TUI/CLI/channel, violating mandatory mode parity.
- Root cause: three sinks assume Token concatenation is canonical and treat the
  terminal payload as a marker instead of committed content.
- Direction: make each adapter commit FinalAnswer(data), reconciling any
  provisional Token buffer by identity rather than duplicating it. Delete
  terminal handlers that discard data.
- Regression validation: non-empty FinalAnswer with zero, partial, full, and
  duplicated Tokens, including Unicode and guard-block paths, across all sinks.
- Validation reports: [V05](../validations/A-CHAT-01/V05-01.md)

### A-CHAT-01-P1-04: TUI advances queued turns before driver completion and handles cancellation differently from error/success

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:657`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:661`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:805`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:265`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:273`
- Reachability: every TUI normal turn uses TuiChatSink and the central event
  loop. `drive_chat` performs Task-mode finalization and ExecutionPath after the
  Agent stream terminal.
- Expected invariant: queued dispatch and active identity release happen once,
  after the shared driver has completed all lifecycle work, for every terminal.
- Observed behavior: FinalAnswer immediately clears state and starts the next
  queued turn before driver postlude; Error also advances the queue; Cancelled
  clears only the token, leaves `active_turn_id`, and does not advance the queue.
  A later TurnStatus handler exists but production TUI callers never send one.
- Impact: queued work can overlap postlude on the same agent/store, or remain
  stuck after cancellation; stale turn identity can misroute steer input.
- Root cause: TUI's Agent-event reducer independently owns lifecycle/queue state
  instead of consuming one driver-complete outcome.
- Direction: advance and clear from one typed application completion emitted
  after driver postlude. Keep Agent terminal rendering separate; delete the
  three terminal-specific queue/lifecycle branches.
- Regression validation: success/error/cancel with queued work, active steer,
  Task mode run finalization, and exactly-once next dispatch.
- Validation reports: [V06](../validations/A-CHAT-01/V06-01.md)

### A-CHAT-01-P1-05: GUI sink persistence failures suppress canonical tool events

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:1193`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:1209`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:1264`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:1293`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:1341`
- Reachability: every GUI tool Call/Stream/Result/Error first enters
  `handle_tool_event`; `Some(bool)` returns before generic Agent-to-Chat mapping.
- Expected invariant: a sink renders/transports the canonical tool event even if
  an auxiliary durable projection fails, and emits the persistence failure as a
  separate typed diagnostic.
- Observed behavior: repository start/finish errors are logged, then return
  `Some(true)`, so no tool start/result/error reaches the frontend. Append
  failures are also log-only. The event stream continues as if projection were
  successful.
- Impact: GUI tool cards can be absent or remain running while the underlying
  tool completed/failed; users and persistence see different trajectories.
- Root cause: rendering and persistence authority are fused inside a synchronous
  sink, with persistence success used as a condition for event delivery.
- Direction: transport the canonical event first/independently, project it
  idempotently through a dedicated repository consumer, and emit typed
  projection failure. Delete the early-return suppression path.
- Regression validation: inject start/append/finish/cancel storage failures and
  assert visible exact tool pairing plus explicit persistence status.
- Validation reports: [V02](../validations/A-CHAT-01/V02-01.md)

### A-CHAT-01-P2-06: The synchronous sink contract forces unbounded transport queues and untyped disconnect

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:47`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:571`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:561`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/channels.rs:195`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/repl.rs:492`
- Reachability: TUI, CLI, and channel use Channel/TUI sinks backed by
  `mpsc::unbounded_channel`; trace bridges also invoke the same synchronous
  callback and ignore false.
- Expected invariant: lifecycle/tool events are bounded and lossless with
  backpressure; progress may be explicitly coalesced; receiver close becomes a
  typed driver outcome and cancellation request.
- Observed behavior: a slow renderer accumulates arbitrary ChatDriverEvents in
  memory. `false` merely breaks the Agent loop, after which driver result can be
  Ok; execution trace callbacks ignore rejection and may continue delivering.
- Impact: large tool output/event bursts can grow memory without bound, while
  UI/channel closure is indistinguishable from successful completion to callers.
- Root cause: the `fn on_event(...) -> bool` API cannot await capacity or carry a
  typed delivery failure.
- Direction: use an async bounded sink contract with delivery classes and typed
  disconnect, coupled to the turn cancellation/outcome owner. Do not add local
  permission gates. Delete unbounded queues after migration.
- Regression validation: slow/closed consumers with lifecycle, tool output,
  execution, and terminal events; assert bounded memory, pairing, cancellation,
  and one outcome.
- Validation reports: [V07](../validations/A-CHAT-01/V07-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, duplicate search, and production callers | yes | passed | [V01](../validations/A-CHAT-01/V01-01.md) |
| V02 | Sink responsibility diff | yes | failed | [V02](../validations/A-CHAT-01/V02-01.md) |
| V03 | One-terminal application outcome table | yes | failed | [V03](../validations/A-CHAT-01/V03-01.md) |
| V04 | Pre-stream/setup/stream error propagation | yes | failed | [V04](../validations/A-CHAT-01/V04-01.md) |
| V05 | Final-answer payload projection | yes | failed | [V05](../validations/A-CHAT-01/V05-01.md) |
| V06 | TUI cancel/error/success/queue lifecycle | yes | failed | [V06](../validations/A-CHAT-01/V06-01.md) |
| V07 | Sink bounds, closure, and cancellation | yes | failed | [V07](../validations/A-CHAT-01/V07-01.md) |
| V08 | Existing test coverage inventory | yes | passed | [V08](../validations/A-CHAT-01/V08-01.md) |
| V09 | Historical convergence/drift | yes | passed | [V09](../validations/A-CHAT-01/V09-01.md) |
| V10 | Dynamic error/cancel/steer/sink fixtures | conditional | not_run | [V10](../validations/A-CHAT-01/V10-01.md) |
| V11 | Exact-ID/link/header/source-clean integrity | yes | passed | [V11](../validations/A-CHAT-01/V11-01.md) |
| V30 | Primary source sampling and acceptance | yes | passed | [V30](../validations/A-CHAT-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `chat_driver.rs:3-11`: one shared chat entry across interactive surfaces | current | Four production callers converge; [V01](../validations/A-CHAT-01/V01-01.md) |
| `chat_driver.rs:27`: complete product event stream consumed by every interactive surface | regressed/overstated | FinalAnswer payload and terminal outcomes differ; [V03](../validations/A-CHAT-01/V03-01.md), [V05](../validations/A-CHAT-01/V05-01.md) |
| `chat_driver.rs:47-55`: sinks are per-mode event consumers and false stops a closed stream | current mechanism, incomplete contract | False is not typed cancellation/outcome and queues are unbounded; [V07](../validations/A-CHAT-01/V07-01.md) |
| `chat_resources.rs:31`: GUI/TUI/CLI/channel use identical lifecycle semantics | regressed | Pre-stream errors and TUI terminal/queue handling differ; [V04](../validations/A-CHAT-01/V04-01.md), [V06](../validations/A-CHAT-01/V06-01.md) |
| Commit `30c28d7`: unify product event transport | structurally current | One transport type remains, but sinks are behaviorally non-equivalent; [V09](../validations/A-CHAT-01/V09-01.md) |

## Coverage And Uncertainty

- Static source conclusively establishes calls, return mappings, ignored
  payloads, queue types, and event ordering. No runtime fault injection or
  memory measurement was performed.
- Tests were inventoried only. V10 is `not_run` by explicit instruction, so the
  report remains `needs_evidence` pending primary source sampling.
- F-RCT-02/F-RCT-03 must fix producer terminal/backpressure/disconnect contracts;
  these A-CHAT findings remain necessary because EKO additionally collapses and
  misprojects even a correctly delivered terminal.
- The Tauri/frontend reducer's response to Error then completed is source-clear,
  but full frontend state replay belongs to A-SRF-03/X-EVT-01.
- Webhook observer counts and redaction were inspected only enough to build the
  terminal table; observability correctness belongs to A-OBS-01.
- Steer does not call `drive_chat`; it injects a Message into an active framework
  turn. A-INP-01 owns preparation/parity. A live steer-after-FinalAnswer race
  depends on framework safe-point timing and is not newly claimed here.

## Handoff

- Primary should first sample V03, V04, V05, V06, and V02. Each demonstrates an
  application-owned defect without relying on disputed framework behavior.
- Remediation order: define one typed driver outcome and terminal publisher;
  move queue/status ownership there; make FinalAnswer payload authoritative;
  separate GUI persistence from transport; then introduce bounded async sinks.
- A-SRF-01 should consume P1-02/P1-03/P1-04 for TUI; A-SRF-02/A-SRF-03 should
  consume P1-01/P1-05; A-SRF-04 should consume P1-02/P1-03/P2-06. X-EVT-01
  should merge the outcome contract with F-RCT backlinks rather than inventing
  per-surface terminal repair.
- This report becomes stale if ChatDriverEvent/ChatSink, `drive_chat` return
  type or postlude, any production sink/caller, envelope normalization, TUI
  terminal reducer, GUI tool projection, or channel aggregation changes.
