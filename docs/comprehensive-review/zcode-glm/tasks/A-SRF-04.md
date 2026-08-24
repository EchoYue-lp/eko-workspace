# A-SRF-04: CLI, channels, cron, and background triggers

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (read-only; `MessageHandler` trait, `SchedulerRunner`, IM channels)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Do non-GUI/TUI triggers enter the same core runtime and preserve identity,
events, memory, tools, cancellation, and terminal semantics?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/src/main.rs` (full, 572 lines) — `main`, `run_tui_or_cli_entry`,
  `build_task_runtime_store_for_headless`; the `--cli`, `--channels`, and
  channels-only exit branches.
- `echo-agent-cli/src/cli/modes.rs` (full, 236 lines) — `start_headless_services`,
  `run_cli_mode`, `run_channels_mode`.
- `echo-agent-cli/src/cli/channels.rs` (full, 903 lines) — `AppChannelMessageHandler`,
  `handle` / `handle_stream`, per-sender `conversation_id` / `cache_user_id`,
  `aggregate_by_sentence` event projection, `ChannelRenderEvent`, channel
  attachment staging, the `#[cfg(test)]` 18-test module.
- `echo-agent-cli/src/cli/repl.rs:95-267, 485-545, 563-845` — REPL loop,
  `chat_with_agent`, `drive_chat` spawn, render switch, no SIGINT/SIGTERM
  handler for in-flight turn cancellation.
- `echo-agent-cli/src/cli/cmd_impls/cron.rs` (full, 566 lines) — slash-command
  surface for `SchedulerRunner` (create/list/delete/pause/resume/run/reload);
  none of these touch an in-flight run identity.
- `echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs` (full, 268
  lines) — `build_fire_fn`, `new_scheduler_runner`, the three unit tests that
  pin the cron → `launch_cron_run` route, the per-fire pool acquire/release.
- `echo-agent-cli/echo-agent-app-core/src/tasks/service.rs:1-610` —
  `BackgroundTaskService` (`submit`, `submit_prompt_run`, `submit_dag`,
  `submit_run`, `start_run_driver`, `cancel`, `pause`, `resume`,
  `resume_pending`, `spawn`), the `SingleAgentTaskProvider` /
  `PoolTaskAgentProvider` adapters, the `background:` conversation-id filter.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs`
  relevant slices (6272 lines total): `launch_unattended_run` (3571-3613),
  `drive_unattended_run` (3616-3639), `drive_agent_run` (3649-3891) — the
  cron/background primary driver; `launch_cron_run` (3895-3934) — the
  cron-specific wrapper.
- `echo-agent-cli/echo-agent-app-core/src/run_driver.rs` (full, ~190 lines) —
  `drive_run_async` (`create_complex_task` half; confirmed separate from
  `drive_chat` per A-CHAT-01).
- `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:1-300, 425-591` —
  `drive_chat` lifecycle (consumed from A-CHAT-01), `ChannelChatSink`,
  `formal_run_id_for_turn`, the Task-mode-only `register_run_cancellation`.
- `echo-agent-cli/echo-agent-app-core/src/chat_resources.rs` (full, 89 lines) —
  `ChatResources` shape consumed by every trigger.
- `echo-agent-cli/echo-agent-app-core/src/hitl/channel_provider.rs:1-90` —
  `ChannelHumanLoopProvider` (per-sender HITL), `subscribe_prompts`,
  `resolve_message`.
- `echo-agent-cli/echo-agent-app-core/src/state.rs:540-720` — `AppState::from_shared`,
  `start_task_service`, `start_scheduler_with_store` (where the scheduler is
  spawned and where its `cancel_token` lives).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:88-161,
  261-300, 525-605, 1631-1760` — `RunCancellationRegistration` (Drop semantics),
  `shutdown_hook_events`, `register_run_cancellation`, `request_cancel`,
  `recover_incomplete`.
- Framework IM channel surface (read-only):
  - `echo-agent/echo-integration/src/channels/types.rs:170-290` — `MessageHandler`
    trait (`handle`, `handle_stream`, `reply`).
  - `echo-agent/echo-integration/src/channels/session.rs` (full, ~470 lines) —
    `SessionHandler`, `SessionConfig` (timeout, reset keywords, command prefix).
  - `echo-agent/echo-integration/src/channels/channels/mod.rs` (full, 131 lines)
    — `dispatch_stream_to_send_tx`, `reply_with_empty_guard` (the empty-text
    placeholder protocol that prevents the gateway's second `reply` from
    double-sending the last chunk).
  - `echo-agent/echo-integration/src/channels/channels/feishu/channel.rs:340-430`
    — `send_feishu_message_internal` (card patching for streaming),
    `FeishuMessageHandler`.
  - `echo-agent/echo-integration/src/channels/channels/qq/channel.rs:90-260` —
    QQ `send` task (one message per chunk), `QqMessageHandler`,
    `gateway` reconnect loop.
- `echo-agent/echo-orchestration/src/scheduler/runner.rs` (full, 193 lines) —
  the framework `SchedulerRunner` (`run_loop`, `tick`, `fire_task`,
  `run_once`, `cancel` token semantics, 30s tick cadence).

Executed cargo tests (all exit 0):

```text
cd echo-agent-cli
cargo test -p echo-agent-app-core --lib scheduler::               → 3 passed
cargo test --no-default-features --features channels --lib        → 27 passed
cargo test -p echo-agent-app-core --lib launch_unattended         → 1 passed
```

## Out Of Scope

Deferred to downstream tasks:

- **A-CHAT-01** (complete) — `drive_chat` single lifecycle ownership, the
  `envelope_event_stream` one-terminal invariant, sink responsibility split,
  the post-`drive_chat` `TurnStatus` asymmetry across callers. This task
  consumes those conclusions and audits only how non-GUI triggers feed into
  `drive_chat` and what they do with the events that come out.
- **A-TSK-03 / A-TSK-04** — `RuntimeDagExecutor` ownership, the
  `EkoRuntimeDagController` callback inventory, claims/revisions/resume
  correctness. This task treats `execute_run` / `drive_unattended_run` as
  the authoritative task-run half and audits only how triggers launch into
  it.
- **A-BOOT-01** (complete) — the boot-time parity findings (P2-01/02/03)
  that this task sharpens into trigger-side consequences for channels-only
  mode. Consumed as input.
- **A-INP-01** — `PreparedUserTurn` normalization. This task treats it as
  the input contract.
- **A-SRF-02 / A-SRF-03** — Tauri command-side adapter and the React
  frontend reducer. Out of scope; this task is the non-GUI counterpart.
- The internal mechanics of plugin-supplied channels beyond the
  `MessageHandler::handle_stream` contract — anything a future plugin
  channel (Slack, Discord) would do is bounded by the same trait shape
  audited here.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (multi-mode parity rule, "only Subagent no
  Worker" terminology, framework-vs-application layering gate, no-panic /
  UTF-8 safety, "check whether it already exists before adding").
- `docs/comprehensive-review/REPORTING.md`,
  `docs/comprehensive-review/templates/task-report.md`,
  `docs/comprehensive-review/templates/validation-report.md`.

Dependency reports read:

- `zcode-glm/tasks/A-CHAT-01.md` (complete) — establishes `drive_chat` as
  the single chat-turn lifecycle owner, the one-terminal invariant via
  `envelope_event_stream`, and the sink responsibility diff. Load-bearing
  for V01: every chat-style trigger must go through `drive_chat`; for V03:
  terminal semantics are uniform across sinks; for V04: only Task-mode
  chat turns register their `CancellationToken`.
- `zcode-glm/tasks/A-TSK-03.md` (complete) — establishes that
  `RuntimeDagExecutor` is the single task-run authority and that EKO
  injects only product policy. Load-bearing for V01: the task-run half
  (`drive_run_async` / `drive_agent_run` / `launch_cron_run`) is
  intentionally separate from the chat half and is the correct entry for
  cron / background; for V04: `finalize_cancelled_run_state` reconciles
  orphaned claims after a cancel.
- `zcode-glm/tasks/A-BOOT-01.md` (complete) — establishes that
  channels-only mode skips `start_headless_services` (P2-02), that
  `TaskRuntimeStore` is built by two parallel paths (P2-01), and that
  MCP-health / dreaming are GUI-only (P2-03). Load-bearing for V04: the
  parity gap is a trigger-side fact for `--channels` users.

Historical documents treated as hypotheses:

- `echo-agent-cli/src/cli/channels.rs:1-11` module docstring — claims the
  handler routes through `drive_chat` for TUI/GUI parity and holds the
  per-sender `AgentPool` + `TaskRuntimeStore`. Verified current (V01).
- `echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs:1-46` module
  doc — claims "ALL cron tasks route through `launch_cron_run`" and the
  `[plan]` prefix is a no-op stripped for backward compat. Verified
  current (V01).
- `echo-agent-cli/echo-agent-app-core/src/tasks/service.rs:1-7` module
  doc — claims CLI/Tauri/cron/background all create a TaskRun and share
  the same persistence/recovery/terminal contracts. Verified current
  with one gap on cron auto-resume (V04).

## Layering Decision

This is an **application-layer** task. The non-GUI triggers
(`AppChannelMessageHandler`, REPL `chat_with_agent`, the cron `FireFn`
closure, `BackgroundTaskService::start_run_driver`) all live in
`echo-agent-cli` / `echo-agent-app-core` and inject EKO product policy
(per-sender isolation, conversation-id conventions, slash-command
surfaces, `[plan]` backward-compat, the cron → TaskRuntime bridge, the
`background:` ownership filter). None belong in the framework.

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework supplies the right primitives: `MessageHandler` trait + `InboundMessage` / `OutboundMessage` (`echo-integration/src/channels/types.rs:170-290`), `SessionHandler` for per-sender lifecycle, `SchedulerRunner` generic over `FireFn`, `CancellationToken`, the `Message` / `EventEnvelope` / `AgentEvent` streaming contract. The triggers correctly compose these — no second scheduler, no second handler trait. |
| EKO product policy | The `AppChannelMessageHandler` (channels.rs), `build_fire_fn` (scheduler/runner.rs), `BackgroundTaskService` (tasks/service.rs), the conversation-id conventions (`channel:`, `background:`, `cron:`), the slash-command dispatchers (cmd_impls/), the `aggregate_by_sentence` projection, the `[plan]` marker strip — all EKO product policy in the application layer. |
| Adapter boundary | Each non-GUI trigger is a thin adapter: it builds a `PreparedUserTurn` + `ChatResources` (chat-style) or a `RunPayload` / cron payload (task-style), calls one entry (`drive_chat` / `drive_run_async` / `launch_cron_run` / `execute_run`), and renders the resulting event stream. None recomputes the chat lifecycle, the DAG, or the frontier. `AppChannelMessageHandler` adds the per-sender pool acquire + HITL provider wiring on top, which is product policy. |
| Duplicate search | Searched both repos for: `drive_chat`, `drive_run_async`, `drive_agent_run`, `drive_unattended_run`, `launch_unattended_run`, `launch_cron_run`, `execute_run`, `AppChannelMessageHandler`, `build_fire_fn`, `BackgroundTaskService`, `chat_with_agent`, `MessageHandler`, `FireFn`, `aggregate_by_sentence`, `dispatch_stream_to_send_tx`, `reply_with_empty_guard`. Result: ONE definition of each; chat-style triggers (REPL/channels/TUI/GUI) all route through `drive_chat` (A-CHAT-01 V01); task-style triggers (cron/background/`create_complex_task`/resume) all route through `drive_run_async` / `drive_agent_run` / `execute_run` (A-TSK-03 V01). No parallel chat driver, no parallel task-run pipeline. |
| Migration deletion | No deletion proposed. The findings are parity gaps (channels-only missing services, REPL/channels missing in-flight cancel, cron missing auto-resume), not parallel implementations to deduplicate. |

## Current Path

### Verified trigger adapter matrix (V01)

There are four classes of non-GUI trigger. They split into two lanes:

```text
─── Chat lane (one entry: drive_chat) ──────────────────────────────

REPL run_repl_turn (repl.rs:485-545)
   │  PreparedUserTurn::build (repl.rs:509-517)
   │  sink = ChannelChatSink(tx)                                       [:493-494]
   │  ChatResources{ sink, cancel=CancellationToken::new(), ... }      [:525-540]
   ↓
tokio::spawn(drive_chat(&agent, &turn, resources))                     [:542-544]

Channels handle_stream (channels.rs:195-265)
   │  PreparedUserTurn::build (channels.rs:208-216)
   │  sink = ChannelChatSink(tx)                                       [:245-246]
   │  ChatResources{ sink, cancel=CancellationToken::new(), ... }      [:247-261]
   ↓
tokio::spawn(drive_chat(&agent_owned, &turn, res))                     [:262-264]
   │  (events projected via aggregate_by_sentence → OutboundMessage)  [:267-291]

─── Task lane (one entry: execute_run / drive_agent_run) ───────────

Cron fire (scheduler/runner.rs:47-130)
   │  fire_id = uuid; cancel = CancellationToken::new()                [:75-76]
   │  pool.acquire("__cron__:{task.id}:{fire_id}")                     [:87-93]
   │  register_task_execute_on_agent                                   [:94]
   ↓
launch_cron_run(store, run_agent, &task.id, &fire_id, prompt, cancel)  [:101]
   → launch_unattended_run → drive_unattended_run → drive_agent_run    [executor.rs:3571-3891]
   → (agent may call task_create + task_execute → execute_run)         [executor.rs:418]

BackgroundTaskService::submit / submit_prompt_run (service.rs:219-305)
   │  run_id = uuid; conversation_id = "background:{source}:{uuid}"    [:274-275]
   │  cancel = self.cancel.child_token()                               [:366]
   │  register_run_cancellation(run_id, cancel)                        [:367-369]
   ↓
start_run_driver → tokio::spawn({                                     [:372]
   │  plan present? execute_run(...)                                    [:418-430]
   │  plan absent?  drive_unattended_run(...)                          [:431-444]
})
```

The TUI and GUI chat lanes are documented in A-CHAT-01 and are out of
scope here. The four non-GUI triggers above are the complete set: there
is no `--daemon` flag, no IPC listener that bypasses these paths, no
plugin-supplied trigger that skips `drive_chat` / `drive_agent_run`.

### Verified identity propagation (V02)

| Trigger | conversation_id | run_id | turn_id (message_id) |
|---|---|---|---|
| REPL chat (`repl.rs:495-540`) | `config.conversation_id` (CLI `--resume` / `--continue` / fresh uuid) | derived `taskrun:{turn_id}` only in Task mode; otherwise none | fresh uuid per turn (`repl.rs:495`) |
| Channel chat (`channels.rs:131, 202, 247-261`) | `channel:{channel_id}:{sender_id}` (stable per sender) | derived `taskrun:{turn_id}` only in Task mode; otherwise none | fresh uuid per turn (`channels.rs:202`) |
| Cron (`executor.rs:3581-3582, 3666-3728`) | `cron:{cron_task_id}:{fire_id}` | fresh uuid, persisted in `TaskRuntimeStore` | empty (no chat message; `message_id_for_scope` filtered out at `:3669-3672`) |
| Background (`service.rs:274-275, 367-369`) | `background:{source}:{uuid}` | fresh uuid, persisted + registered for cancellation | empty (same as cron) |

The identity scheme is consistent within each lane:

- **Chat lane**: every turn carries `turn_id` (the message id) and the
  optional `taskrun:{turn_id}` derivation (Task mode only). The
  `EventIdentity` envelope (`chat_driver.rs:483-489`) threads
  `conversation_id`, `run_id` (Task mode only), and `turn_id` into the
  framework's `AgentInvocationContext`, so tool calls during the turn
  see the same identity via `ExternalRunContext`
  (`chat_driver.rs:490-511`).
- **Task lane**: every run carries its own `run_id` (persisted in
  `TaskRuntimeStore`) and a synthetic `conversation_id`. The cron and
  background paths deliberately pass an empty `root_message_id` to
  `create_run` (`executor.rs:3589`, `service.rs:284`) — there is no chat
  message to bind to. `drive_agent_run` reads the run back from the
  store and reconstructs `conversation_id_for_scope` /
  `message_id_for_scope` from the persisted row
  (`executor.rs:3665-3672`), so the in-scope identity matches what is
  on disk.

Cross-trigger identity facts verified by static inspection:

- `formal_run_id_for_turn(turn_id)` (`task_tools.rs:178-180`) is the
  single derivation rule for "the formal run id for a chat turn", used
  by `drive_chat:222` and by `scoped_with_ctx_run_id:217`. Chat-mode
  task tools that need a run id (`create_complex_task`,
  `check_run_status`) reach it via `current_run_id()` /
  `require_run_id()` (`task_tools.rs:174-176`).
- The channel `cache_user_id` (`channels.rs:71-73`) flows into
  `agent.config_mut().set_cache_user_id(...)` (`channels.rs:142-144`),
  isolating DeepSeek KV cache per sender. Same hook the GUI/TUI use.
- The channel conversation_id is **stable per sender** across restarts
  (it is a pure function of `channel_id` + `sender_id`), so a user who
  restarts the bot resumes the same agent context via
  `AgentPool::acquire(&conv)` (`channels.rs:135-139`). The cron and
  background conversation_ids include a per-fire / per-submit uuid, so
  each fire/submit is a fresh conversation (intentional — no cross-fire
  memory leak).

### Noninteractive event output (V03)

Three production renderers consume the shared product stream. None
invents lifecycle state; each projects the same `ChatDriverEvent` /
`ExecEvent` stream into its surface.

- **REPL** (`repl.rs:563-845`): a `match` on `ChatDriverEvent`. Token
  streaming → `output.print_token`. Tool calls → `output.print_tool_call`
  with a "DANGER" prefix for `shell` / `delete_file` / `git_commit`.
  Terminal events (`FinalAnswer`, `Cancelled`, `Error`) handled
  inline. `TurnStatus { status != "running" }` prints the status line.
  `ExecutionPath` prints the requested → observed path. Plain text;
  no markdown escaping; UTF-8 safe (the `chars().take(500)` preview at
  `:568` mirrors the channel convention).
- **Channel (IM)** (`channels.rs:514-654`): `aggregate_by_sentence`
  accumulates `AgentEvent::Token` into a sentence buffer and flushes on
  newline / sentence-end punctuation / 80-char threshold, projecting
  to a stream of `OutboundMessage` chunks. Terminal events flush the
  buffer (`:564-575`). Non-token events surface as bracketed notices:
  `[budget]`, `[guard]`, `[chart]`, `[safety]`, `[parameter]`,
  `[task:{run_id}]` (attention events only), `[paused:{run_id}]`. The
  `MemoryRecalled` event is silenced (`:594-596`). UTF-8 safe: all
  truncation uses `chars().take()` / `chars().skip()` / `chars().count()`
  (verified by the 9-test `aggregate` module at `channels.rs:738-903`,
  including `multibyte_no_panic` and `fullwidth_punctuation_flushes`).
- **QQ gateway** (`qq/channel.rs:108-132`): one `OutboundMessage` per
  chunk, sent as separate text messages via `send_qq_message`. No card
  patching; no markdown.
- **Feishu gateway** (`feishu/channel.rs:340-397`): chunks are rendered
  as **interactive cards** (`build_card_content` at `:158-171`,
  embedding markdown). The first chunk creates a card via
  `reply_message` / `send_card_message`; subsequent chunks **patch the
  same card** via `patch_card_message` for a streaming-update effect
  (`:362-381`). Card ids are cached in `running_cards` with a 1-hour
  TTL (`:358-359`). This is the most sophisticated projection — but it
  is still pure rendering: the underlying `OutboundMessage` stream is
  the same one QQ consumes.

The streaming-protocol invariant that makes "one chunk per send" and
"patch the same card" both correct is `dispatch_stream_to_send_tx`
(`channels/channels/mod.rs:16-32`): the gateway wrapper's `handle`
calls `inner.handle_stream`, drains it chunk by chunk into `send_tx`,
and returns an **empty-text placeholder** `OutboundMessage`. The
gateway then calls `reply`, which `reply_with_empty_guard`
(`channels/channels/mod.rs:36-48`) treats as a no-op — preventing the
last chunk from being double-sent. Verified by the three unit tests at
`channels/channels/mod.rs:87-129`.

Tool execution events (`ChatDriverEvent::Execution`) on the channel
surface only render for `event.event.is_attention_event()`
(`channels.rs:627-638`), which is the explicit-lifecycle subset
(`RunFailed`, `TaskCancelled`, `ArtifactProduced`, `MergeStarted`, etc.
— `types.rs:717-733`). High-frequency progress events are dropped on
the channel surface (intentional — IM users do not want tool-call
spam). The REPL prints every execution event with a 500-char preview
(`repl.rs:566-574`) — a presentation asymmetry, not a contract one.

### Cancel / shutdown / recovery (V04)

The cancellation matrix across non-GUI triggers:

| Trigger | Cancel handle | Reachable how | On shutdown |
|---|---|---|---|
| REPL chat turn | fresh `CancellationToken::new()` per turn (`repl.rs:533`) | **NOT externally reachable.** No `register_run_cancellation` for Chat/Auto turns (only Task mode registers at `chat_driver.rs:240-252`). No SIGINT handler installed by REPL. | Process exit forcible-aborts the spawned drive_chat task. No graceful cancel. |
| Channel chat turn | fresh `CancellationToken::new()` per turn (`channels.rs:244`) | **NOT externally reachable.** Same as REPL. The IM user's only option is to wait or restart the bot. | `manager.stop_all()` drops the manager; spawned drive_chat tasks are forcible-aborted on runtime drop. |
| Cron fire | `cancel = CancellationToken::new()` at `runner.rs:76`; becomes the parent of a child token registered via `drive_agent_run:3663`. | `store.request_cancel(run_id)` cancels the child (`store.rs:577-605`). No other handle. | Scheduler's own `cancel_token` (state.rs:384 / state.rs:662) is NEVER cancelled by any shutdown path — only main's local `cancel_token` is. Scheduler task is forcible-aborted on runtime drop; the run transitions to Paused on next boot via `recover_incomplete`. |
| Background run | `self.cancel.child_token()` at `service.rs:366`; registered at `:367-369`. | `BackgroundTaskService::cancel(id)` (`service.rs:457-461`) → `store.request_cancel(id)`. | Same as cron — the service's `cancel` field is never explicitly cancelled on shutdown; tasks are forcible-aborted. |

Recovery on crash / restart:

- `TaskRuntimeStore::recover_incomplete` (`store.rs:1631-1760`) runs at
  every boot (via `build_task_runtime_store_for_headless` at
  `main.rs:52` for headless, via `AppState::from_shared` for GUI). It
  lists every run whose status is `Running` and transitions it to
  `Paused`, recording a recovery note. It then walks each `Running`
  todo and decides `Pending` (interrupted; pending resume),
  `Blocked` (mutating side effect indeterminate), or `Pending` (subagent
  already completed; pending review). So no run is left as `Running`
  after a crash — the on-disk status is reconciled.
- The post-boot resume differs by trigger:
  - **Background runs** (`background:` conversation prefix):
    `BackgroundTaskService::resume_pending` (`service.rs:556-589`) is
    invoked from `BackgroundTaskService::spawn` (`service.rs:591-603`)
    which is called by `AppState::start_task_service`
    (`state.rs:684-708`). It filters runs by
    `conversation_id.starts_with("background:")` (`service.rs:563`),
    skips runs with unresolved recovery blockers, and re-dispatches
    via `start_run_driver`. So background runs auto-resume.
  - **Cron runs** (`cron:` conversation prefix): there is NO
    automatic resume path. `BackgroundTaskService::resume_pending`
    explicitly filters them out (the `background:` prefix test fails
    for `cron:...`). No other component scans for paused cron runs.
    A cron run interrupted mid-flight becomes Paused and sits there
    until a user manually resumes it via the GUI's task-runtime
    surface (if available) — see A-SRF-04-P2-02.
  - **Task-mode chat runs** (`taskrun:` conversation prefix): they
    are recovered to Paused by `recover_incomplete` but, like cron
    runs, are not in the `background:` filter. They surface in the
    GUI's task list (via `task_runtime_store.list_runs_in(...)`),
    where the user can resume them.

`shutdown_hook_events` (`store.rs:261`) flushes the task hook
dispatcher on every entry's exit path (`main.rs:330, 395, 438`;
`desktop.rs:263`). It does NOT cancel in-flight runs — the runs'
tokens are independent.

## Findings

### A-SRF-04-P2-01: Chat/Auto turns on REPL and channels have no externally reachable cancel handle

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/cli/repl.rs:533` — `cancel:
    echo_agent::agent::CancellationToken::new()` is a fresh token local
    to the `ChatResources` built for this turn. The token is captured
    into the spawned `drive_chat` future (`:542-544`) and never
    registered anywhere.
  - `echo-agent-cli/src/cli/channels.rs:244` — same pattern:
    `let cancel = echo_agent::agent::CancellationToken::new();` local
    to the per-sender spawned task (`:237-265`).
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:240-252` —
    only `InteractionMode::Task` calls `register_run_cancellation`. For
    Chat/Auto (the default for REPL and the default for channels —
    `channels.rs:60` initializes `InteractionMode::Auto`), the
    cancellation registration is skipped.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:532-548`
    — `register_run_cancellation` is the only path that inserts into
    `run_cancel_tokens`; nothing else does. So an unregistered token is
    unreachable via `request_cancel`.
  - Contrast: `echo-agent-cli/src/tui/events.rs:1937-1958` (`handle_esc`)
    and `:1147-1153` (`q` key) cancel via `app.active_cancel.cancel()`;
    `:1410` sets `app.active_cancel = Some(cancel.clone())` before
    spawning. TUI therefore CAN cancel an in-flight Chat/Auto turn.
  - Contrast: `BackgroundTaskService::cancel` (`service.rs:457-461`)
    cancels a background run by `run_id` via the registered token.
- Reachability: every Chat/Auto turn on REPL and every chat turn on
  channels. The REPL `Ctrl+C` handler at `repl.rs:239-241` only prints
  a hint ("输入 /exit 退出"); it does NOT cancel the in-flight turn
  (and it is only reached between turns, since `chat_with_agent` is
  awaited inline at `:234` and blocks the read loop). During an
  in-flight REPL turn, Ctrl+C falls through to the process's default
  SIGINT handler and terminates the process — the in-flight turn is
  forcible-aborted, the run (if any) becomes Paused on next boot, and
  the user loses the partial response.
- Expected invariant: AGENTS.md multi-mode parity rule — "TUI、GUI(以及
  CLI/channel)必须功能对等". TUI users can cancel an in-flight turn;
  CLI and channel users cannot. The chat turn's cancel token should be
  reachable from the surface that started it.
- Observed behavior: REPL and channel chat turns in Chat/Auto mode
  (the common case) have no `/cancel` command, no SIGINT handler, and
  no registered token. The turn's CancellationToken is held only by the
  spawned task and dies with it. A long-running turn on a channel
  blocks that sender's pool agent until completion (the agent is held
  via `pool.acquire(&conv)` at `channels.rs:135-139`, and the pool
  serializes per-key), so the user's next message is queued behind the
  in-flight one — there is no escape hatch.
- Impact: (a) functional parity gap vs TUI; (b) operational — a
  runaway agent turn on a channel (e.g. an infinite tool loop) ties
  up the per-sender agent until the bot is killed; (c) UX — the IM
  user has no way to stop a turn that is producing unwanted side
  effects (file writes, MCP calls).
- Root cause: the per-turn CancellationToken was modeled as a
  private resource for the chat lane, with the assumption that only
  Task mode needed external cancellation (because Task mode persists
  a run that survives the turn). For Chat/Auto, the cancel handle was
  never wired out to the surface. The TUI solved this independently
  via `app.active_cancel`; REPL and channels never gained the
  equivalent field.
- Direction: (1) extend `ChatResources` or the surface state with an
  accessible cancel handle — either register the chat turn's token
  under a synthetic run id (`chatturn:{turn_id}`) in
  `TaskRuntimeStore::run_cancel_tokens` and expose a `cancel_chat_turn`
  helper, or store the token on a per-surface registry (REPL: a
  `Mutex<Option<CancellationToken>>` on the ReplConfig; channels: a
  per-sender map on `AppChannelMessageHandler`). (2) For REPL, install
  a `tokio::signal::ctrl_c()` handler that cancels the current turn's
  token (matching the TUI's `handle_esc` semantics). (3) For channels,
  add a `/cancel` slash command (the framework's SessionHandler already
  routes `/`-prefixed messages as commands) that cancels the sender's
  in-flight turn. All three are small, localized changes that preserve
  the existing lifecycle.
- Regression validation: a test that drives a slow `MockLlmClient`
  through `drive_chat`, cancels the registered token mid-turn, and
  asserts the sink receives exactly one terminal `AgentEvent::Error`
  (the synthesized "ended without terminal" — see A-CHAT-01 V03).
  Plus an integration test that boots REPL, sends a turn, issues
  `/cancel`, and asserts the turn terminates.
- Validation reports: [V04-01](../validations/A-SRF-04/V04-01.md).

### A-SRF-04-P2-02: Cron runs are recovered to Paused on restart but never auto-resumed

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:3581-3582`
    — `launch_unattended_run` constructs
    `conversation_id = format!("{source_kind}:{source_id}:{fire_id}")`.
    For cron, `source_kind = "cron"` (`scheduler/runner.rs:101` passing
    `"cron"` via `launch_cron_run` at `executor.rs:3905-3907`). So a
    cron run's `conversation_id` starts with `"cron:"`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/service.rs:541, 552, 563, 616, 910`
    — every recovery / resume / list path in `BackgroundTaskService`
    filters on `conversation_id.starts_with("background:")`. A cron
    run's `"cron:..."` prefix fails this filter and is invisible to
    the background service.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1631-1760`
    — `recover_incomplete` transitions any `Running` run to `Paused`
    regardless of conversation prefix. So cron runs ARE reconciled to
    Paused on next boot — but nothing then wakes them up.
  - `echo-agent-cli/src/cli/cmd_impls/cron.rs:296-302` — `/cron resume`
    re-enables a CronTask schedule (the `CronTaskStatus`), not an
    interrupted run. The slash command surface has no notion of
    resuming a paused cron RUN.
  - `echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs:47-130`
    — the `FireFn` creates a fresh `fire_id` per tick and never looks
    up previously-interrupted runs. The framework `SchedulerRunner`
    (`echo-orchestration/src/scheduler/runner.rs:52-65`) only ticks
    forward; it has no "resume interrupted" pass.
- Reachability: any cron run interrupted by process restart
  (Ctrl+C, SIGTERM, crash, machine reboot). The run is transitioned
  to Paused by `recover_incomplete` and stays there indefinitely.
- Expected invariant: AGENTS.md multi-mode parity — background runs
  resume on next boot (`service.rs:556-589`), and cron runs are
  conceptually equivalent (an unattended TaskRuntime run on a pool
  agent). They should resume the same way. The trigger-origin
  distinction (`background:` vs `cron:`) is a product-policy
  convention, not a semantic difference at the executor level.
- Observed behavior: a cron run that is e.g. 90% through a long
  task plan, interrupted by a nightly bot restart, becomes Paused
  forever. Its partial work (worktree, subagent results, completed
  todos) is preserved on disk, but no component re-dispatches it.
  The next cron tick fires a NEW run for the same CronTask,
  duplicating the work.
- Impact: (a) silent capability gap — interrupted cron runs leak
  Paused runs that pile up in the store; (b) potential duplicate
  work when the schedule fires again before the user notices; (c)
  the cron promise of "runs on schedule" is broken across any
  restart, which is the opposite of what users expect from cron.
- Root cause: the `background:` filter at `service.rs:541, 552, 563`
  was written when background was the only unattended trigger; cron
  was added later (Phase 3.1) with its own conversation prefix but
  was not added to the resume filter. The `FireFn` creates fresh
  fire_ids without consulting interrupted runs.
- Direction: two complementary options. (1) Extend the resume filter
  in `BackgroundTaskService::resume_pending` to also cover
  `conversation_id.starts_with("cron:")` (or, more cleanly, drop the
  prefix filter and resume any Paused run that the recovery blockers
  allow). (2) Have the scheduler's `FireFn` consult
  `TaskRuntimeStore::list_runs_in(&[Paused])` for runs matching the
  CronTask's id before creating a new fire — re-dispatch the
  interrupted run instead of duplicating. Option (1) is the smaller
  fix and aligns cron with the existing background resume path.
- Regression validation: a test that seeds a Paused cron run in the
  store, calls `BackgroundTaskService::resume_pending` (after the
  filter fix), and asserts the run is re-dispatched
  (`start_run_driver` invoked, status transitions Pending → Running).
- Validation reports: [V04-01](../validations/A-SRF-04/V04-01.md).

### A-SRF-04-P2-03: Channels-only entry starts neither SchedulerRunner nor BackgroundTaskService (imported from A-BOOT-01-P2-02)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/main.rs:357-403` — the channels-only branch
    spawns `cli::run_channels_mode(...)` and exits without calling
    `start_headless_services`.
  - `echo-agent-cli/src/cli/modes.rs:32-64` — `start_headless_services`
    is the only place `state.start_task_service().await` and
    `state.start_scheduler_with_store(...)` are called for headless.
    Channels-only bypasses both.
  - `echo-agent-cli/src/cli/modes.rs:118-235` — `run_channels_mode`
    constructs the `ChannelManager` and registers the
    `AppChannelMessageHandler`, but never starts the scheduler or the
    background task service. Even though
    `build_task_runtime_store_for_headless` (`main.rs:175`) is called
    and the store is passed to the channel handler
    (`modes.rs:122-123`), there is no `SchedulerRunner` to fire cron
    tasks and no `BackgroundTaskService` to drain the queue.
  - Import: A-BOOT-01-P2-02 confirmed the same fact from the
    boot-lifecycle side; this task sharpens it from the trigger side.
- Reachability: any `echo-agent-cli --channels` launch (without
  `--cli`). Live gap.
- Expected invariant: AGENTS.md multi-mode parity — channels must be
  a full Agent surface. Cron and background tasks are core services
  that should be available in every long-running entry.
- Observed behavior: a user running `echo-agent-cli --channels` as a
  long-running IM bot gets no scheduled cron fires and no background
  task execution. Tasks created via the channel chat (e.g. the agent
  calling `create_complex_task`) DO still run — they go through
  `drive_run_async` (`run_driver.rs:62`) which is invoked from the
  `create_complex_task` tool body, not from `BackgroundTaskService`.
  But the cron schedule and the explicit `BackgroundTaskService::submit`
  API are unavailable.
- Impact: silent capability gap for channels-only deployments. A
  user who configures cron tasks in `~/.eko/...` expects them to fire;
  in channels-only mode they never do.
- Root cause: the channels branch was wired directly to
  `run_channels_mode` without routing through the shared headless
  service starter (mirrors A-BOOT-01-P2-02's root cause).
- Direction: call `start_headless_services` in the channels-only
  branch before spawning `run_channels_mode`, mirroring the TUI/CLI
  branches at `main.rs:258-274`. The shared starter already returns
  `(task_service, scheduler_runner)` as loose `Arc`s; channels mode
  would ignore them (or hold them for the lifetime of the process)
  since the channel handler does not consume them directly.
- Regression validation: boot `--channels` with a fake CronTask and
  assert the scheduler ticks; assert `BackgroundTaskService` is
  constructible. (Already covered directionally by A-BOOT-01-P2-02's
  recommended test.)
- Validation reports: [V04-01](../validations/A-SRF-04/V04-01.md).

### A-SRF-04-P3-01: REPL slash commands (cron/tasks) are unavailable on channels — `/trace`, `/analysis`, `/papers`, `/skills` are wired but `/cron` and `/tasks` are not

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/cli/channels.rs:333-412` — the channel handler
    special-cases `/trace`, `/analysis`, `/papers`, `/skills`
    (returning an immediate `OutboundMessage` via the early-return
    path at `:121-128`). These delegate to `cmd_impls::analysis`,
    `cmd_impls::research`, `cmd_impls::skills`.
  - `echo-agent-cli/src/cli/cmd_impls/cron.rs:563-565` — `register_all`
    is only called from `repl.rs:179` (the REPL command registry). The
    channel handler never constructs a `CommandRegistry` and never
    calls `cron::register_all`.
  - `echo-agent-cli/src/cli/channels.rs:306-330` — `parse_channel_mode_command`
    handles `/mode` directly. There is no equivalent for `/cron`,
    `/tasks`, `/diff`, `/pipeline`, etc.
- Reachability: every channel conversation. The IM user who tries
  `/cron list` or `/tasks` over IM gets the message routed to
  `drive_chat` as a normal chat instruction (the agent answers
  conversationally) rather than as a structured slash command.
- Expected invariant: AGENTS.md multi-mode parity — slash commands
  available in REPL should be reachable from channels where they make
  sense (commands that mutate local state — e.g. `/cron`, `/skills`,
  `/mode` — are obviously channel-relevant).
- Observed behavior: only `/trace`, `/analysis`, `/papers`, `/skills`,
  and `/mode` are wired on channels. The rest of the
  `cmd_impls` registry (`cron`, `tasks_ext`, `diff_cmd`, `git`,
  `pipeline`, `pipelines`, `workspace`, `plugins`, `evolution`,
  `coding`, `context`, `hooks`, `observability`, `info`, `advanced`)
  is REPL-only.
- Impact: low. The agent can answer conversational equivalents ("list
  my cron tasks"), and most slash commands are developer-tooling that
  does not map cleanly to IM. But `/cron` is operationally important
  for a channels-only bot (the user's only way to manage schedules
  without dropping into REPL).
- Root cause: the channel command surface was added incrementally
  (one helper per command as the need arose); there is no shared
  command dispatcher between REPL and channels.
- Direction: factor a `ChannelCommandDispatcher` that mirrors the
  REPL `CommandRegistry` for the subset of commands that make sense
  over IM (at minimum `/cron`, `/mode`, `/skills`, `/trace`), or
  route the entire `CommandRegistry` through channels with a denylist
  for commands that require a TTY (`/diff`, `/coding`, etc.).
- Regression validation: a `parse_channel_command` test that asserts
  `/cron list` dispatches to the cron command surface and returns an
  `OutboundMessage` (mirroring `parse_channel_mode_command`'s tests).
- Validation reports: [V03-01](../validations/A-SRF-04/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Trigger adapter matrix: every non-GUI trigger routes through `drive_chat` (chat lane) or `drive_agent_run` / `execute_run` (task lane); no parallel path | yes | passed | [V01-01](../validations/A-SRF-04/V01-01.md) |
| V02 | Identity propagation: each trigger carries `conversation_id` + `run_id` (or `turn_id` for chat) correctly into `AgentInvocationContext` | yes | passed | [V02-01](../validations/A-SRF-04/V02-01.md) |
| V03 | Noninteractive event output: REPL / channel / Feishu / QQ all consume the shared `ChatDriverEvent` stream and project it without inventing lifecycle | yes | passed (with finding — P3-01) | [V03-01](../validations/A-SRF-04/V03-01.md) |
| V04 | Cancel/shutdown/recovery: every trigger has a reachable cancel handle; every run recovers on restart; cron auto-resumes | yes | failed (3 findings: P2-01/P2-02/P2-03) | [V04-01](../validations/A-SRF-04/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — three code comments / module docs treated as hypotheses; classifications inline in Historical Claim Status) | passed | classified inline |

Executed cargo commands (all exit 0):

```text
cd echo-agent-cli
cargo test -p echo-agent-app-core --lib scheduler::                → 3 passed
cargo test --no-default-features --features channels --lib         → 27 passed
cargo test -p echo-agent-app-core --lib launch_unattended          → 1 passed
```

The full pre-commit matrix (fmt / clippy / all-features test) was not
re-run because this review is read-only; the targeted subsets above are
the suites that exercise the trigger-adapter boundary.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `channels.rs:1-11` module doc: "channels drive chat through the shared `drive_chat` entry. Holds the per-sender `AgentPool` + the `TaskRuntimeStore` (so `create_complex_task` can build `ChatResources`)." | current | V01 confirms `AppChannelMessageHandler::handle_stream` calls `drive_chat` at `channels.rs:262` and constructs `ChatResources` with `pool` + `store` at `:247-261`. |
| `channels.rs:29-33` comment: "Whether a complex run is warranted is decided by the agent itself, not pre-judged here." | current | V01 confirms `handle_stream` does not branch on prompt complexity; the agent invokes `create_complex_task` from inside the ReAct loop if needed. |
| `scheduler/runner.rs:1-12` module doc: "ALL cron tasks route through the unified TaskRuntime executor (`launch_cron_run`)." | current | V01 confirms `build_fire_fn` always calls `launch_cron_run` at `runner.rs:101`; the `[plan]` strip at `:64-69` is a backward-compat no-op. The 3-test suite pins this. |
| `scheduler/runner.rs:36-39` comment: "the dead-in-practice `runtime_store=None` fallback ... has been removed — `AppState` always constructs a `TaskRuntimeStore` at boot." | partially stale (for channels-only) | V04 / A-BOOT-01-P2-02: channels-only mode does NOT call `start_headless_services`, so `AppState` is never built; the `task_runtime_store` is built by `build_task_runtime_store_for_headless` only. The "always constructs" claim holds for GUI and TUI/CLI, not for channels-only. |
| `tasks/service.rs:1-7` module doc: "CLI, Tauri, cron-style background work and structured pipelines all create an EKO TaskRun and use the same product persistence, recovery, and terminal contracts." | current-with-caveat | V01 confirms all task-lane triggers route through `execute_run` / `drive_unattended_run`. V04 confirms `recover_incomplete` reconciles every `Running` run on restart. BUT cron runs do not auto-resume (A-SRF-04-P2-02) — the recovery contract is partly breached for the cron trigger. |
| `executor.rs:3568-3569` doc on `launch_unattended_run`: "The run is created with `attended_mode = Unattended` so the configured write preflight applies inside `task_execute` / `execute_task`." | current | V01 confirms `AttendedMode::Unattended` is passed at `executor.rs:3593`; the preflight applies via `unattended_direct_disabled_tools` / `unattended_run_prompt` at `executor.rs:3683-3685`. |
| `chat_driver.rs:190-201` doc on turn/run identity: "普通 chat 轮次使用 `res.root_message_id` 作 turn_id. Task mode 和 task tools 从该 turn_id 派生独立的 `taskrun:<turn_id>`" | current | V02 confirms `drive_chat:211-216` reads `res.root_message_id` as `turn_id`, and `formal_run_id_for_turn` at `chat_driver.rs:222` is the single derivation. |
| A-CHAT-01 handoff: "Cancel and steer are NOT second lifecycles. Cancel cancels the shared `CancellationToken`." | current (with caveat) | V04 confirms the chat-lane cancel is via the shared token. BUT for Chat/Auto mode on REPL/channels, that token is not externally reachable (A-SRF-04-P2-01) — the cancellation surface is missing, not the lifecycle. |
| A-TSK-03 handoff: "`finalize_cancelled_run_state` reconciles every `Pending|Running|Blocked` task to `Cancelled` after the kernel returns `Cancelled`." | current | V04 confirms the same reconciliation runs on the cron and background paths (via `drive_agent_run`'s outcome branch). |
| A-BOOT-01-P2-02 handoff: "Channels-only entry skips `start_headless_services`." | current (sharpened into A-SRF-04-P2-03) | V04 confirms the trigger-side consequence: no SchedulerRunner, no BackgroundTaskService in channels-only mode. |

## Coverage And Uncertainty

- **Inspected in full:** `channels.rs` (903 lines), `scheduler/runner.rs`
  (268 lines), `tasks/service.rs` head + relevant slices (610 lines),
  `cmd_impls/cron.rs` (566 lines), `run_driver.rs` (~190 lines),
  `chat_resources.rs` (89 lines), `channels/channels/mod.rs` (131 lines),
  the framework `SchedulerRunner` (193 lines), `modes.rs` (236 lines).
- **Inspected partially:** `executor.rs` (6272 lines) was read in the
  3571-3935 slice covering `launch_unattended_run` /
  `drive_unattended_run` / `drive_agent_run` / `launch_cron_run`. The
  per-task pipeline (`execute_task` 1843-2509) and the broader
  controller (`EkoRuntimeDagController` 1147-1620) were not re-audited
  here — they are owned by A-TSK-03. `chat_driver.rs` was read in
  1-300, 425-591 (the lifecycle and the sink contract) — A-CHAT-01
  owns the deep audit. `store.rs` was read in the slices covering
  `RunCancellationRegistration`, `register_run_cancellation`,
  `request_cancel`, `recover_incomplete` (88-161, 261-300, 525-605,
  1631-1760).
- **Not inspected (out of scope):**
  - The Feishu / QQ gateway internals beyond the wrapper-handler
    contract (`feishu/long_poll.rs`, `feishu/webhook.rs`,
    `feishu/proto.rs`, `feishu/api.rs`, `qq/gateway.rs`, `qq/api.rs`).
    The wrapper-handler / send_tx / patch_card surface audited here is
    the contract layer; the wire-protocol internals are framework
    territory.
  - The full `cmd_impls/` registry beyond `cron.rs`. The REPL command
    surface's per-command correctness is owned by the individual
    command audits.
  - The `tui/events.rs` cancel plumbing beyond the `handle_esc` /
    `active_cancel` cross-reference (A-SRF-02 / A-CHAT-01 own it).

Environmental constraints:

- The cargo test runs were against the existing incremental cache; the
  three target subsets (scheduler / channels-feature / launch_unattended)
  all passed with exit 0. No feature matrix re-run (this review did not
  touch feature definitions or `#[cfg]` branches beyond reading them).
  Worktree clean at `b3b2e81`.

Uncertain claims:

- Whether any user is actually running `--channels` in production today.
  The channels feature is gated behind `--features channels` and the
  `--channels` CLI flag; the parity gaps in P2-03 are only live for
  users who do so. If the primary deployment is TUI/GUI, P2-03's
  priority could be revisited — but the AGENTS.md parity rule applies
  regardless of current deployment.
- Whether the cron auto-resume gap (P2-02) is masked in practice by
  users rarely restarting the bot during a cron fire. The defect is
  real (the resume filter excludes `cron:`), but its frequency depends
  on uptime patterns.

## Handoff

Conclusions downstream tasks may rely on:

1. **Two lanes, two entries.** Non-GUI triggers split cleanly: chat
   lane (REPL, channels, TUI, GUI) all route through `drive_chat`
   (`chat_driver.rs:202`); task lane (cron, background,
   `create_complex_task`, GUI resume) all route through
   `drive_run_async` / `drive_agent_run` / `execute_run`. There is
   no third lane and no parallel implementation of either. (V01)
2. **Identity is consistent within each lane.** Chat turns carry
   `turn_id` (and derive `taskrun:{turn_id}` for Task mode). Task runs
   carry their own persisted `run_id` + a synthetic conversation_id
   (`background:`, `cron:`, etc.). `AgentInvocationContext.runtime`
   threads these into the framework so tools see the same identity.
   (V02)
3. **Every renderer is a pure projection of the shared stream.**
   REPL, channel (QQ), and channel (Feishu) all consume the same
   `ChatDriverEvent` / `OutboundMessage` stream; none invent lifecycle.
   Feishu's card patching is the most sophisticated projection, but
   it operates on the same chunked `OutboundMessage` stream QQ uses.
   (V03)
4. **Chat/Auto cancel is reachable only on TUI.** REPL and channels
   build a fresh `CancellationToken` per turn that is never
   registered; only Task mode registers via
   `register_run_cancellation`. This is a parity gap (P2-01) that
   also affects the channel's ability to bail out of a runaway turn.
5. **Task-lane runs reconcile on crash but resume selectively.**
   `recover_incomplete` transitions every `Running` run to `Paused`
   at boot. `BackgroundTaskService::resume_pending` then re-dispatches
   `background:` runs. Cron runs (`cron:` prefix) are NOT in the
   resume filter and sit Paused forever — A-SRF-04-P2-02.
6. **Channels-only mode skips the scheduler and background service
   entirely** (A-SRF-04-P2-03 / A-BOOT-01-P2-02). The trigger-side
   consequence: no cron fires, no `BackgroundTaskService::submit`
   pathway. The `create_complex_task` tool still works (it goes
   through `drive_run_async`, not the service).

Reports they must read:

- This report (A-SRF-04) for the trigger-adapter matrix, the cancel
  / recovery gaps, and the cron auto-resume defect.
- `tasks/A-CHAT-01.md` for the chat-lane lifecycle ownership and the
  one-terminal invariant that the chat-lane triggers rely on.
- `tasks/A-TSK-03.md` for the task-lane ownership and the
  reconciliation sweeps that govern cron/background terminal states.
- `tasks/A-BOOT-01.md` for the boot-side parity findings
  (P2-01/02/03) that P2-03 imports.

Conditions that make this report stale:

- Any new non-GUI trigger (e.g. a plugin-supplied Slack channel, an
  HTTP webhook entry, a `--daemon` flag) invalidates V01 and requires
  adding the trigger to the matrix.
- Any change that adds a cancel handle to REPL or channels
  (resolving P2-01) invalidates V04's "no externally reachable
  cancel" claim for those surfaces.
- Any change to the `background:` / `cron:` conversation-id prefix
  convention invalidates V02 and A-SRF-04-P2-02's filter analysis.
- Any change that extends `BackgroundTaskService::resume_pending` to
  cron runs (resolving P2-02) invalidates V04's "cron never
  auto-resumes" claim.
- Any change that calls `start_headless_services` in the channels-only
  branch (resolving P2-03) invalidates V04's "no scheduler in
  channels-only" claim.
- Adding new `ChatSink` implementations or new renderers (e.g. a
  Discord channel with embed rendering) requires adding them to V03.

Follow-up task IDs (no fixes implemented in this review):

- A **chat-turn cancel surface** task: resolve A-SRF-04-P2-01 by
  wiring an accessible cancel handle on REPL (`/cancel` +
  `tokio::signal::ctrl_c()` handler) and channels (`/cancel` slash
  command routed through `SessionHandler`). Touches `repl.rs`,
  `channels.rs`, possibly `chat_resources.rs` to expose a registry.
- A **cron auto-resume** task: resolve A-SRF-04-P2-02 by extending
  `BackgroundTaskService::resume_pending`'s filter (or dropping the
  prefix filter) to cover `cron:` runs. Small localized change in
  `tasks/service.rs`; add a regression test that seeds a Paused cron
  run and asserts it is re-dispatched.
- A **channels-only services wiring** task: resolve A-SRF-04-P2-03
  (and A-BOOT-01-P2-02) by routing the channels-only branch through
  `start_headless_services`. Touches `main.rs:357-403`; the same fix
  also closes A-BOOT-01-P2-02.
- A **channel slash-command parity** task: resolve A-SRF-04-P3-01 by
  factoring a `ChannelCommandDispatcher` (or routing the existing
  `CommandRegistry` through channels with a denylist). Touches
  `channels.rs` and possibly `cmd_impls/`.
