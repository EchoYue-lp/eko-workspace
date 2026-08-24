# A-SRF-04: CLI, channels, cron, and background triggers

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: `echo-agent-cli` clean; unrelated `echo-agent` modifications excluded, with one corrected dirty-source exposure recorded in V00
> Accepted by: Codex primary reviewer after independent committed-source,
> reachability, finding-count, link, executor, commit, and isolation sampling.

## Question

Do non-GUI/TUI triggers enter the same core runtime and preserve identity,
events, memory, tools, attachments/artifacts, Task/Subagent/HITL capability,
cancellation, terminal semantics, automation, and shutdown ownership?

## Scope

- CLI argument and main dispatch, interactive REPL, prepared-turn construction,
  shared chat call, event rendering, task/background/cron commands, and service
  startup.
- QQ and Feishu production ingress/egress, session/handler construction,
  conversation/cache/delivery identity, attachments, HITL, event projection,
  cancellation, and channel manager lifecycle.
- BackgroundTaskService and cron adapter definition, registration, startup,
  canonical TaskRuntime entry, recovery, cancellation roots, and shutdown.
- Static definition/duplicate, registration/reachability, trigger/capability,
  identity, event, cancellation/shutdown, and existing-test matrices.

## Out Of Scope

- Shared driver terminal/outcome collapse, pre-stream terminal gaps,
  FinalAnswer loss, sink backpressure/disconnect typing, and per-sink terminal
  projection are canonical in [A-CHAT-01](A-CHAT-01.md).
- TaskRuntime retry, completion/cancel arbitration, terminal settlement, and
  cross-TaskRun dependency polling are canonical in [A-TSK-03](A-TSK-03.md).
- GUI/Tauri ownership is A-SRF-02; generic tool execution is A-TOOL-01; generic
  export/file delivery is A-OUT-01; conversation persistence is A-STATE-01.
- Source fixes, shared indexes, Cargo/rustc/tests/builds, dynamic fixtures, and
  network activity.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, exact A-SRF-04 card in
  `TASKS.md`; Codex isolation README and report templates.
- Authorized Codex dependency reports [A-CHAT-01](A-CHAT-01.md) and
  [A-TSK-03](A-TSK-03.md), both classified current at the reviewed commits.
- Current clean `echo-agent-cli`, clean framework channel source, and the
  committed scheduler HEAD blob. No other reviewer directory was read.

## Layering Decision

| Classification | Current answer |
|---|---|
| Generic mechanism | Channel message types/session primitives and SchedulerRunner/Agent cancellation primitives are reusable framework capabilities. Framework defects are not inferred merely because EKO adapters are incomplete. |
| EKO product policy | CLI/channel mode composition, full surface parity, conversation/delivery key selection, one-shot protocol, artifact projection, and process shutdown order belong in `echo-agent-cli`. |
| Adapter boundary | CLI/channel should convert transport input to `PreparedUserTurn`, call one `drive_chat`/TaskRuntime, and losslessly project identity/events/artifacts. They must not own another Agent loop or Task graph. |
| Duplicate search | Searched both repositories for trigger entry names, `drive_chat`, ChatResources/events, task/cron launch, cancel/shutdown tokens, attachment builders, session/delivery keys, and ready/retry/dependency loops. One shared chat runtime and one canonical PlanTask executor are live. |
| Migration deletion | Preserve shared driver, TaskRuntime, framework channel types, and scheduler primitive. Delete sender-only group key/target construction, temporary headless state ownership, and detached/unowned turn/service tasks when replaced by one lifecycle adapter. |

No SQLite dependency or online-service permission gate is implicated. Channel
parity is a required EKO capability, not an optional reduced product mode.

## Current Path

```text
main bootstrap
  -> primary Agent + task tools + task_execute + canonical TaskRuntimeStore
  -> AgentPool
     -> CLI --cli
        -> start_headless_services -> BackgroundTaskService + SchedulerRunner
        -> Reedline -> PreparedUserTurn -> ChatResources -> drive_chat
        -> human terminal rendering
     -> --channels
        -> ChannelManager -> QQ/Feishu -> SessionHandler
        -> AppChannelMessageHandler -> PreparedUserTurn -> ChatResources
        -> drive_chat -> sentence/text OutboundMessage
     -> cron/background
        -> launch canonical TaskRuntime run -> isolated pool Agent/Subagents
```

The architecture converges before the adapter boundary: CLI and channel both
use the same prepared turn, pool, store, tool registration, memory/review layer,
Task/Subagent path, HITL provider, and serializable `ChatDriverEvent` stream
([V01](../validations/A-SRF-04/V01-01.md),
[V02](../validations/A-SRF-04/V02-01.md)). Cron/background also enter the
canonical TaskRuntime rather than implementing a second PlanTask executor.

### Trigger and capability matrix

| Trigger | Shared runtime | Task/Subagent/tools | HITL | Attachments/artifacts | Cancel | Automation | Terminal/output |
|---|---|---|---|---|---|---|---|
| Interactive CLI REPL | `drive_chat` | yes | terminal provider | staged input refs; human text output | token exists but surface cannot reach it during turn | full background/cron commands and services | human rendering; A-CHAT owns terminal losses |
| QQ/Feishu channel | `drive_chat` | yes | per-session text provider | app staging exists, production transports never populate it; output text only | token detached; `/cancel` only cancels pending HITL | no service startup/cron commands in channel-only mode | selected text projection; A-CHAT owns terminal losses |
| Background submission | TaskRuntime | yes | unattended policy | TaskRuntime refs | run-store cancel API | service started only by TUI/CLI/GUI | persisted TaskRuntime facts/webhooks |
| Cron | TaskRuntime | yes | unattended policy | TaskRuntime refs | run-store token after launch | scheduler absent from channel-only mode | persisted TaskRuntime facts/webhooks |
| Noninteractive CLI | absent | absent | absent | absent | absent | cannot invoke | no typed JSONL/exit contract |

Six adapter/lifecycle deviations remain.

## Findings

### A-SRF-04-P1-01: Group channels discard `chat_id` for both state identity and reply routing

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-integration/src/channels/types.rs:74`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-integration/src/channels/session.rs:295`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/channels.rs:65`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/channels.rs:85`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-integration/src/channels/channels/qq/api.rs:266`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-integration/src/channels/channels/feishu/channel.rs:350`
- Reachability: QQ group gateway and both Feishu receivers populate distinct
  sender and chat IDs -> SessionHandler/AppChannelMessageHandler discard chat ID
  -> sender-only AgentPool/conversation/cache/HITL -> sender ID used as group API
  destination.
- Expected invariant: full conversation identity includes channel and chat;
  actor remains sender; group delivery targets chat ID.
- Observed behavior: one sender's messages in every group on a channel share the
  same session, Agent, memory, TaskRuntime conversation, cache, interaction mode,
  and pending HITL. Replies label the sender ID as a QQ group ID or Feishu
  `chat_id`.
- Impact: group replies can fail, and unrelated group contexts can share history,
  task state, and approval/input messages, violating surface identity parity.
- Root cause: direct-message identity (`chat_id == sender_id`) was generalized as
  a per-sender channel model even though the transport already preserves chat ID.
- Direction: define one typed channel conversation/delivery identity using
  channel + chat for state/destination and sender for actor; apply it to session,
  pool/cache/conversation/HITL and reply. Delete sender-only key/target helpers.
- Regression validation: QQ/Feishu direct and group fixtures, including one
  sender in two simultaneous groups, independent HITL, memory/tasks, and exact
  outbound destination.
- Validation reports: [V03](../validations/A-SRF-04/V03-01.md)

### A-SRF-04-P1-02: Channel-only mode never starts background or cron automation

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:357`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/modes.rs:32`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/modes.rs:83`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/modes.rs:118`
- Reachability: `--channels` without `--cli` directly spawns
  `run_channels_mode`; only TUI and CLI call `start_headless_services`.
- Expected invariant: every long-lived EKO host starts/recoveries the same
  background and scheduler services exactly once, independent of renderer.
- Observed behavior: channel-only receives the TaskRuntimeStore but no
  BackgroundTaskService/SchedulerRunner, never starts either, and exposes no
  cron management command. Stored schedules therefore do not fire.
- Impact: an EKO process intentionally run as an IM assistant silently stops
  cron/background recovery and cannot manage automation from that surface.
- Root cause: service startup is nested in individual TUI/CLI adapters instead
  of process composition shared by all active surfaces.
- Direction: move one start-once service owner above surface branching and pass
  handles into CLI/channel adapters. Do not add a second channel scheduler.
- Regression validation: channel-only restart with pending background work and
  due cron, plus combined CLI/channel start-once assertions.
- Validation reports: [V05](../validations/A-SRF-04/V05-01.md)

### A-SRF-04-P1-03: CLI and channel discard the only handle that can cancel an active foreground turn

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/repl.rs:215`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/repl.rs:525`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/channels.rs:237`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/hitl/channel_provider.rs:42`
- Reachability: every accepted CLI/channel chat constructs a fresh cancellation
  token and passes it through `drive_chat` to Agent/tools/Subagents; neither
  caller retains a surface-visible clone.
- Expected invariant: Ctrl+C/channel cancel/disconnect reaches the active turn
  until its one terminal, regardless of Chat/Task/Auto mode.
- Observed behavior: the interactive REPL is blocked in `chat_with_agent` so its
  Reedline Ctrl+C branch cannot run; channel token lives only inside the detached
  producer. Channel `/cancel` has semantics only while HITL is pending and is a
  new chat input otherwise.
- Impact: users cannot stop a long or destructive foreground turn through CLI
  or IM, even though cancellation propagation inside the shared driver exists.
- Root cause: adapters create cancellation as an anonymous resource rather than
  an active-turn lifecycle object keyed by conversation/turn identity.
- Direction: retain the token in one active-turn registry and map CLI Ctrl+C,
  channel cancel, and disconnect to it. Delete detached token construction once
  the registry owns admission/settlement. A-CHAT-01 remains canonical for
  terminal publication after cancellation.
- Regression validation: cancel before first event, during tool/Subagent/HITL,
  after terminal, and simultaneous replacement/disconnect in all modes.
- Validation reports: [V07](../validations/A-SRF-04/V07-01.md)

### A-SRF-04-P1-04: Channel attachment and full-artifact support is declared but unreachable through production transports

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-integration/src/channels/types.rs:89`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-integration/src/channels/channels/qq/gateway.rs:271`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-integration/src/channels/channels/feishu/webhook.rs:166`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-integration/src/channels/channels/feishu/long_poll.rs:576`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/channels.rs:199`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/channels.rs:528`
- Reachability: all live QQ/Feishu receivers construct text-only inbound
  messages -> app staging always sees an empty vector; all app channel output
  uses text-only constructors -> platform attachment field stays empty.
- Expected invariant: channel files/media become durable typed prepared-turn
  refs and task outputs expose complete artifacts or stable references.
- Observed behavior: QQ ignores attachment payloads, Feishu webhook rejects all
  non-text types, long-poll produces text only, and no ingress calls
  `with_attachments`. Output never calls `with_attachments`; complex output is
  reduced to text/previews.
- Impact: channel users cannot provide screenshots/documents or receive complete
  generated files/reports despite the advertised shared capability.
- Root cause: message data types and app conversion were added without wiring
  concrete platform media download/upload adapters and artifact projection.
- Direction: implement platform-specific media ingestion/egress as thin adapters
  around canonical AttachmentRef/artifact identity. Preserve framework public
  attachment types; coordinate full export policy with A-OUT-01.
- Regression validation: real-shape QQ/Feishu image/file payloads, Unicode names,
  large artifacts, download/upload failure, cancel, cleanup, and output delivery.
- Validation reports: [V04](../validations/A-SRF-04/V04-01.md)

### A-SRF-04-P1-05: Headless service and combined channel tasks have no cancel-and-join lifecycle owner

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/modes.rs:32`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/state.rs:375`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/service.rs:162`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/scheduler/runner.rs:23` (committed HEAD blob);
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:365`
- Reachability: TUI/CLI headless startup creates temporary AppState root tokens,
  returns service Arcs, then drops the only public token owners. Combined
  `--channels --cli` also drops the channel JoinHandle after REPL exit.
- Expected invariant: one process owner cancels active turns/services, awaits
  channel/background/scheduler children, drains durable hooks, then tears down
  pool/browser/runtime.
- Observed behavior: cleanup never cancels TaskState/SchedulerState tokens or
  joins their spawned loops. Combined mode bypasses `manager.stop_all`; Tokio
  runtime destruction is the effective abort mechanism.
- Impact: shutdown can interrupt sends, cron fires, background runs, or channel
  connections without ordered cancellation/terminal settlement/resource cleanup.
- Root cause: `start_headless_services` returns capabilities but not lifecycle
  control; spawned tasks expose neither unified shutdown nor join completion.
- Direction: return/retain one `HeadlessServices` owner with idempotent
  cancel-and-join, and include channel manager/turns in ordered shutdown. Delete
  temporary AppState ownership and the "ends automatically" branch.
- Regression validation: shutdown during channel send, foreground tool/HITL,
  cron fire, background run, and combined-mode REPL exit; assert settled terminal
  state and no live children.
- Validation reports: [V08](../validations/A-SRF-04/V08-01.md)

### A-SRF-04-P1-06: There is no noninteractive CLI Agent/event contract

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/args.rs:12`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:111`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/repl.rs:215`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:27`
- Reachability: all command-line dispatch is TUI, hidden interactive Reedline,
  channels, or removed web mode; no prompt/stdin/exec branch calls `drive_chat`.
- Expected invariant: CLI automation can submit one prepared turn, receive
  ordered canonical typed events with identity and one terminal, and use exit
  status to distinguish completed/failed/cancelled/disconnected.
- Observed behavior: no one-shot mode exists. The serializable event type is
  consumed only into human-oriented REPL rendering.
- Impact: shell/editor/CI automation cannot invoke EKO's complete Agent surface
  or reliably consume Task/Subagent/tool/artifact lifecycle without scraping
  terminal text.
- Root cause: "CLI" currently names an interactive compatibility REPL; no thin
  noninteractive adapter was added over the canonical driver.
- Direction: add one-shot stdin/argument input and JSONL output directly over
  `PreparedUserTurn`/`drive_chat`, with typed terminal-to-exit mapping and stderr
  diagnostics. Do not create a second event model or execution driver.
- Regression validation: success/failure/cancel/setup error/EOF, Unicode,
  attachment refs, tool and Task/Subagent events, broken pipe, SIGINT, and exact
  one-terminal/exit agreement.
- Validation reports: [V06](../validations/A-SRF-04/V06-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Dirty-source/reviewer isolation boundary | yes | inconclusive, corrected scope | [V00](../validations/A-SRF-04/V00-01.md) |
| V01 | Definition, layering, and duplicate-runtime search | yes | passed | [V01](../validations/A-SRF-04/V01-01.md) |
| V02 | Registration and real shared-runtime reachability | yes | passed | [V02](../validations/A-SRF-04/V02-01.md) |
| V03 | Channel conversation/delivery identity | yes | failed | [V03](../validations/A-SRF-04/V03-01.md) |
| V04 | Attachment/artifact transport reachability | yes | failed | [V04](../validations/A-SRF-04/V04-01.md) |
| V05 | Channel-only background/cron startup | yes | failed | [V05](../validations/A-SRF-04/V05-01.md) |
| V06 | Noninteractive typed event output | yes | failed | [V06](../validations/A-SRF-04/V06-01.md) |
| V07 | Foreground cancellation ownership | yes | failed | [V07](../validations/A-SRF-04/V07-01.md) |
| V08 | Cancel/shutdown and service ownership | yes | failed | [V08](../validations/A-SRF-04/V08-01.md) |
| V09 | Existing test/edge-case coverage inventory | yes | failed | [V09](../validations/A-SRF-04/V09-01.md) |
| V10 | Dependency deduplication/historical status | yes | passed | [V10](../validations/A-SRF-04/V10-01.md) |
| V11 | Dynamic trigger/cancel/recovery fixtures | policy-deferred | not_run | [V11](../validations/A-SRF-04/V11-01.md) |
| V12 | Report/link/executor/source integrity gate | yes | attempt 1 failed (cwd); attempt 2 passed | [A1](../validations/A-SRF-04/V12-01.md), [A2](../validations/A-SRF-04/V12-02.md) |
| V30 | Primary acceptance sampling | yes | passed | [V30](../validations/A-SRF-04/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| A-CHAT-01: all surfaces enter shared `drive_chat`, but sink/terminal semantics diverge | current | [V02](../validations/A-SRF-04/V02-01.md), [V10](../validations/A-SRF-04/V10-01.md) |
| A-CHAT-01: channel/CLI FinalAnswer and terminal issues | current, canonical there | not duplicated; [V10](../validations/A-SRF-04/V10-01.md) |
| A-TSK-03: cron/background planned work enters canonical RuntimeDagExecutor | current | [V01](../validations/A-SRF-04/V01-01.md), [V10](../validations/A-SRF-04/V10-01.md) |
| A-TSK-03: background cross-TaskRun dependency poll is a second authority | current, canonical there | not duplicated; [V10](../validations/A-SRF-04/V10-01.md) |
| Source comments: per-sender channel isolation provides parity | incomplete/regressed for groups | [V03](../validations/A-SRF-04/V03-01.md) |
| Source comments: IM attachments use the GUI/TUI path | definition exists but transport claim is stale | [V04](../validations/A-SRF-04/V04-01.md) |

## Coverage And Uncertainty

- This was pure static review. No Cargo/rustc/test/build/fixture/network process
  ran; V11 is explicitly `not_run` and future implementation validation remains.
- The `echo-agent` worktree was dirty. Only clean channel files and committed
  scheduler HEAD were used. V00 discloses one early dirty scheduler display and
  the clean-blob reconstruction; primary acceptance must sample the commit.
- Platform API behavior is inferred from current adapter construction and its
  own typed URL/receive-ID branches; no live QQ/Feishu call was made.
- A-CHAT-01 remains authoritative for terminal/event loss after the adapter
  reaches `drive_chat`; A-TSK-03 remains authoritative inside TaskRuntime.
- A-OUT-01 should define complete export/file delivery, while this task owns the
  fact that the channel transport cannot carry the existing attachment contract.

## Handoff

- Preserve the positive convergence on `PreparedUserTurn`, `drive_chat`,
  AgentPool, TaskRuntime, and one PlanTask executor.
- Fix trigger adapters in this order: full group identity/destination;
  process-level service start/stop owner; active-turn cancellation; channel
  media/artifact ingress/egress; noninteractive JSONL CLI.
- A-STATE-01 should consume P1-01 when reviewing channel conversation restore;
  A-OUT-01 should consume P1-04; X-SRF-01 and Q-E2E-01 should own cross-surface
  and dynamic regression matrices without creating another runtime.
- This report becomes stale if main mode dispatch, CLI args/REPL, channel
  session/transport/handler adapters, `start_headless_services`,
  BackgroundTaskService/SchedulerRunner lifecycle, or shared chat event types
  change.
