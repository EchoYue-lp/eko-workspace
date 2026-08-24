# F-INT-02: LSP, channels, and A2A integrations

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean

## Question

Do LSP, IM channel, and A2A adapters isolate external protocols while
preserving typed internal lifecycle and cleanup?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-integration/src/lsp/mod.rs` — LSP integration
  facade (module re-exports).
- `echo-agent/echo-integration/src/lsp/client.rs` — `StdioLspClient`
  lifecycle (spawn → initialize → requests → shutdown).
- `echo-agent/echo-integration/src/lsp/manager.rs` — `LspManager`
  multi-language collection + start/stop/restart/shutdown_all.
- `echo-agent/echo-integration/src/lsp/config.rs` — `LspConfig`
  YAML + PATH-based auto-discovery.
- `echo-agent/echo-integration/src/lsp/jsonrpc.rs` — JSON-RPC
  framing + `parse_content_length`.
- `echo-agent/echo-core/src/lsp/client.rs` — typed `LspClient` trait
  + `LspError`.
- `echo-agent/echo-core/src/lsp/types.rs` — `CompletionItem`,
  `Diagnostic`, `HoverInfo`, `Location`, `LspServerConfig`,
  `LspServerStatus`, `Position`, `TextChange`.
- `echo-agent/src/lsp.rs` — root framework re-exports.
- `echo-agent/echo-integration/src/channels/types.rs` —
  `ChannelPlugin` / `MessageHandler` traits, `InboundMessage` /
  `OutboundMessage`.
- `echo-agent/echo-integration/src/channels/manager.rs` —
  `ChannelManager` (multi-plugin facade) + `Drop` impl.
- `echo-agent/echo-integration/src/channels/session.rs` —
  `SessionHandler` per-user session lifecycle.
- `echo-agent/echo-integration/src/channels/channels/mod.rs` —
  shared `dispatch_stream_to_send_tx` /
  `reply_with_empty_guard` helpers.
- `echo-agent/echo-integration/src/channels/channels/feishu/{channel,long_poll,webhook,api,proto}.rs`
  — Feishu WebSocket long-poll + axum webhook + HTTP API + PBBP2
  protobuf.
- `echo-agent/echo-integration/src/channels/channels/qq/{channel,gateway,api}.rs`
  — QQ Bot WebSocket gateway + HTTP API.
- `echo-agent/src/channels.rs` — framework `AgentChannelHandler`
  adapter + re-exports.
- `echo-agent/src/a2a/mod.rs` — A2A facade.
- `echo-agent/src/a2a/types.rs` — `AgentCard`, `TaskState`,
  JSON-RPC request/response/event types.
- `echo-agent/src/a2a/auth.rs` — `JwtConfig` + `jwt_middleware`.
- `echo-agent/src/a2a/server.rs` — `A2AServer` (handle_request,
  handle_request_stream, handle_task_send/get/cancel).
- `echo-agent/src/a2a/serve.rs` — axum router + graceful shutdown.
- `echo-agent/src/a2a/client.rs` — `A2AClient` (discover,
  send_task, send_task_streaming, get_task, cancel_task).
- Application callers verified for reachability:
  `echo-agent-cli/echo-agent-app-core/src/runtime.rs:499-590`
  (`register_lsp_tools`),
  `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs`
  (`shutdown_all` paths),
  `echo-agent-cli/src/cli/modes.rs:130-231` (channel launcher).

## Out Of Scope

- Tauri command DTO ↔ frontend type parity for the LSP panel —
  deferred to A-FE-01.
- Tauri command surface for channel management — EKO CLI modes
  (`src/cli/modes.rs`) is the only application caller; deferred to
  A-SRF-04.
- Application-layer permission gating of MCP/LSP/Browser (cf.
  A-INT-01) — the framework LSP/channels/A2A layers themselves have
  no permission gates by design (per AGENTS.md "MCP/LSP are
  user-configured").
- Cross-repository invariant audit for worker terminology — owned
  by X-INV-01; F-INT-02 only validates its own scope (V04).
- Subagent execution and Task DAG semantics — owned by F-SUB-0x and
  F-TSK-0x.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — Subagent-only terminology rule;
    framework-vs-application layering; no-panic / UTF-8 safety;
    "MCP/LSP user-configured, no over-gating"; code-cleanup rule;
    cross-repository path rules.
  - `docs/comprehensive-review/REPORTING.md` — finding/validation
    contracts.
  - `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
  - `docs/comprehensive-review/TASKS.md` — F-INT-02 task spec.
- Dependency task reports read:
  - `F-CORE-01` (zcode-glm) — relied on its conclusion that
    `CancellationToken` is the framework's canonical cancellation
    primitive. Used here to assess whether LSP / channels / A2A
    thread cancellation through correctly (they don't, in most
    paths).
  - `F-INT-01` (zcode-glm) — relied on its conclusion that the MCP
    `notification_tx` advertises an unimplemented channel and that
    MCP cancellation is a no-op server-side. Used here as a
    comparison reference for transport-level cancellation discipline.
- Historical documents treated as hypotheses: none.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. LSP, channels, and A2A are pure protocol integrations: any `echo-agent` consumer that wants to talk to a language server, an IM platform, or another A2A-compatible agent needs them. They live correctly in `echo-integration` (LSP, channels) and the root `src/a2a` (HTTP server/client), not in `echo-core`. V01/V02/V03 confirm single definition sites. |
| EKO product policy | None at this layer. EKO's only role is to *launch* these integrations (`runtime.rs:register_lsp_tools`, `cli/modes.rs:channel run`). No permission gates exist in the framework layer; EKO does not add any either (consistent with AGENTS.md "user-configured"). |
| Adapter boundary | `AgentChannelHandler` (`src/channels.rs:79-130`) is a thin adapter: `MessageHandler::handle` calls `agent.chat(&msg.text).await`, then wraps the reply in `OutboundMessage`. It owns no scheduler/registry. `A2AServer` is more than an adapter — it owns a per-task state map (`tasks`), a cancel-token map (`cancel_tokens`), and the state-machine authority — but that authority is *the* A2A protocol authority (no second implementation elsewhere), so it is correctly placed. |
| Duplicate search | Searched names: `StdioLspClient`, `LspManager`, `LspClient`, `LspConfig`, `ChannelManager`, `ChannelPlugin`, `FeishuChannel`, `FeishuConfig`, `QqChannel`, `QqConfig`, `SessionHandler`, `MessageHandler`, `A2AServer`, `A2AClient`, `AgentCard`, `TaskState`, `JwtConfig`. Result: each is defined exactly once (V01/V02/V03). Application only consumes via `echo_agent::*` re-exports. |
| Migration deletion | No migration proposed. The dead `restart_count` / `last_error` fields on `StdioLspClient` (F-INT-02-P2-01) are deletion targets within the same patch that introduces real restart tracking. |

## Current Path

Verified data flow at commit `9b0e0fa`:

1. **LSP ingestion.**
   - EKO startup: `echo-agent-cli/echo-agent-app-core/src/runtime.rs:499-590`
     `register_lsp_tools` discovers servers via
     `LspConfig::discover(&project_root)` (PATH-based, probes
     `rust-analyzer`, `pyright-langserver`, `basedpyright-langserver`,
     `pylsp`, `typescript-language-server`, `gopls`, `jdtls`,
     `clangd` per `config.rs:121-196`), merges global
     (`~/.eko/.lsp.yaml`) and project (`.lsp.yaml`) configs, then
     calls `LspManager::start_server` per language.
   - `start_server` (`manager.rs:64-89`) wraps `client.initialize` in
     a 15 s `tokio::time::timeout`, stores `Arc<RwLock<StdioLspClient>>`.
   - On plugin reload / rollback, `plugin_runtime.rs` calls
     `shutdown_all()` (`:583, 596, 639, 691, 707, 721`) to tear down
     the previous LSP set before installing the replacement.

2. **LSP request lifecycle.**
   - Tool exposure: `runtime.rs:578-` registers `LspGotoDefinitionTool`,
     `LspHoverTool`, `LspDiagnosticsTool`, `LspFindReferencesTool`,
     etc., each of which calls `LspManager::get_client_for_file` to
     pick a server by file extension (`manager.rs:111-123`).
   - The tool then calls `LspClient::goto_definition` /
     `hover` / etc. (`echo-integration/.../client.rs:338-521`), each
     of which is `Box::pin(async move { send_request(...).await })`
     (`client.rs:202-236`).
   - `send_request` allocates a `u64` id, registers a `oneshot` in
     `pending`, writes the framed message via `writer_tx`, awaits the
     response.
   - Reader task (`client.rs:125-199`) parses Content-Length framed
     messages, dispatches responses by id (line 176-183), stores
     diagnostics notifications in `diagnostics_cache`.

3. **Channel ingestion.**
   - EKO CLI: `echo-agent-cli/src/cli/modes.rs:130-231` constructs a
     `ChannelManager`, registers `QqChannel` and/or `FeishuChannel`,
     calls `start_all(handler_factory)` (modes.rs:217), awaits
     shutdown signal, then `stop_all()` (modes.rs:231).
   - `start_all` (`manager.rs:53-76`) iterates per-plugin; each
     plugin's `start` spawns the platform-specific background task.

4. **Channel message flow.**
   - **Feishu LongPoll**: WebSocket binary frames (PBBP2 protobuf)
     → `WsClient::handle_data_frame` (`long_poll.rs:375-448`)
     → ack immediately (3 s deadline), spawn `process_event_async`
     → `process_im_message` → `handler.handle(inbound)` →
     `handler.reply(outbound)`.
   - **Feishu Webhook**: HTTP POST → `handle_event`
     (`webhook.rs:80-214`) → challenge passthrough / HMAC verify /
     dedup → `tokio::spawn(process)`.
   - **QQ Gateway**: WebSocket text frames (Discord-style opcodes)
     → `handle_gateway_event` → `handle_c2c_message` /
     `handle_group_at_message` → `dispatch_to_handler`.
   - The agent is bridged via `AgentChannelHandler` (`src/channels.rs`)
     which calls `agent.chat(&msg.text).await` and wraps the reply.
   - The platform-specific `*MessageHandler` wrapper
     (`feishu/channel.rs:400-429`, `qq/channel.rs:243-259`) consumes
     the inner handler's `handle_stream` chunks via
     `dispatch_stream_to_send_tx` (`channels/mod.rs:16-32`) and
     pushes each chunk through the channel's `send_tx` for true
     streaming delivery; an empty placeholder prevents the gateway's
     follow-up `reply` from duplicating the last chunk
     (`reply_with_empty_guard`, `channels/mod.rs:36-48`).

5. **A2A ingestion.**
   - `serve` / `serve_with_auth` / `serve_from_config[_with_auth]`
     (`serve.rs:55-94`) build an axum router: protected group
     (`/.well-known/agent.json`, `/`) + unprotected group
     (`/health`, `/ready`) + `DefaultBodyLimit::max(max_body_bytes)`.
   - On `POST /`: `handle_json_rpc` parses method, dispatches
     `tasks/sendSubscribe` to the SSE handler and everything else to
     `handle_request` (sync).
   - Sync `tasks/send` (`server.rs:378-485`): Submitted → Working →
     agent.execute → Completed/Failed. Stores a CancellationToken
     but never polls it during execute.
   - Streaming `tasks/sendSubscribe` (`server.rs:140-347`): yields
     SSE events, polls `cancel_token.is_cancelled()` every iteration,
     terminates on cancel/error/completion.
   - `tasks/cancel` (`server.rs:527-584`): sets state to Canceled,
     cancels the token. Calls `tasks.write()` then nested
     `cancel_tokens.read()` (no deadlock — no reverse-order path).

## Findings

### F-INT-02-P1-01: A2A sync `tasks/send` does not honor cancellation; terminal monotonicity violated when cancel races a late completion

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/a2a/server.rs:404-408` creates and stores a
    `CancellationToken` for the sync path.
  - `echo-agent/src/a2a/server.rs:414` awaits `agent.execute(&input_text)`
    with **no `cancel_token.is_cancelled()` check before, during,
    or after**.
  - `echo-agent/src/a2a/server.rs:439-442` and `:467-470` write the
    terminal Completed / Failed state via raw
    `tasks.insert(task_id, completed_task)`, **bypassing
    `update_task_state`** and its `can_transition_to` matrix
    (`server.rs:588-602`).
  - `echo-agent/src/a2a/server.rs:527-584` `handle_task_cancel`
    acquires `tasks.write()`, sees Working (non-terminal), writes
    `Canceled` directly to `task.status` (also bypassing
    `update_task_state`), then cancels the token via
    `cancel_tokens.read().await.get(&task_id).cancel()`.
- Reachability: any A2A client that calls `tasks/send` for a
  long-running agent task while another client (or the same client
  on another connection) calls `tasks/cancel` with the same task id.
  Concrete sequence:
  1. Client A: `tasks/send` → server sets Working, enters
     `agent.execute().await`.
  2. Client B: `tasks/cancel` → server sees Working, writes Canceled,
     calls `cancel_token.cancel()` (no listener), returns Canceled
     response.
  3. `agent.execute()` returns to Client A's path. Server overwrites
     the task with `completed_task` (state = Completed).
- Expected invariant (per `src/a2a/types.rs:322-327` and
  `src/a2a/mod.rs:13-18`): once a task reaches a terminal state
  (`completed / failed / canceled`), its state must not regress.
  `TaskState::is_terminal` (`types.rs:357-359`) and
  `can_transition_to` (`types.rs:362-378`) encode this; terminal
  states return false for every target. The streaming path honors
  this via `update_task_state` and an explicit early return on
  cancel (`server.rs:224-235`). The sync path does not.
- Observed behavior: `tasks/get` after the race returns `Completed`
  even though `tasks/cancel` returned `Canceled` to its caller. The
  task state has regressed from Canceled to Completed.
- Impact: Violates A2A's terminal-state monotonicity contract. A
  client that issued `tasks/cancel` cannot trust the cancel
  response; downstream observers see Completed. For long-running
  tasks (translation, code-generation, research) this can also
  continue consuming LLM tokens and side-effect budget after the
  user explicitly canceled. The cancel_token is wired but useless
  for the sync path because nothing polls it.
- Root cause: `handle_task_send`'s terminal write was written
  before `update_task_state`'s transition-check helper existed, and
  was never migrated to use it. The cancel token was added to the
  sync path's state but no cancellation checkpoint was inserted
  around `agent.execute()`.
- Direction:
  1. Route the Completed / Failed writes in `handle_task_send`
     through `update_task_state` so the `Canceled → Completed`
     transition is rejected. Per AGENTS.md code-cleanup, remove the
     raw `tasks.insert` writes.
  2. Wrap `agent.execute(&input_text).await` in a
     `tokio::select!` against `cancel_token.cancelled()`. On
     cancel, transition to Canceled and return a Canceled response
     instead of waiting for `execute` to finish. (Mirror the
     streaming path's per-iteration check.)
  3. Optionally: also fix `handle_task_cancel`'s direct
     `task.status = Canceled` write to go through
     `update_task_state` (less critical because cancel can never
     regress a terminal, but consistency is cheap).
- Regression validation: a deterministic two-task test — start a
  sync `tasks/send` against a slow agent, fire `tasks/cancel`
  mid-flight, assert (a) cancel response is Canceled, (b)
  `tasks/get` after `execute` completes still reports Canceled,
  (c) `execute`'s late result is discarded.
- Validation reports: [V03-01](../validations/F-INT-02/V03-01.md)

### F-INT-02-P2-01: LSP `restart_count` and `last_error` are dead fields; status report is misleading

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/lsp/client.rs:39-42` declares
    `restart_count: u32` and `last_error: Option<String>` on
    `StdioLspClient`.
  - `client.rs:60-61` initializes them to `0` / `None` in `new`.
  - `client.rs:523-532` `status()` reports them verbatim into
    `LspServerStatus`.
  - **No assignment to either field exists** anywhere in
    `echo-integration/src/lsp/` (grep `restart_count|last_error`
    returns only the declaration, init, and read sites).
  - `LspManager::restart_server` (`manager.rs:105-108`) is stop then
    start, which constructs a fresh `StdioLspClient` — the old
    instance's `restart_count` is discarded; the new one starts at
    0. There is no "restart count" tracking anywhere.
  - `LspServerStatus::restart_count` and `:last_error` are surfaced
    through `status_all` (`manager.rs:141-165`) to the application
    and (transitively) to the GUI/TUI LSP panel.
- Reachability: any code that reads `LspServerStatus` after a
  restart or a server crash — i.e. every status refresh in EKO's
  LSP panel. The reported `restart_count: 0` and `last_error: None`
  are unconditionally false after the first incident.
- Expected invariant: a status field that purports to track
  observed events (`restart_count`, `last_error`) must reflect
  observed events.
- Observed behavior: `restart_count` and `last_error` always read
  as their initial values for the entire lifetime of a client
  instance. Consumers that depend on them to surface "the LSP
  server has crashed N times, last error was X" see no signal.
- Impact: misleading observability. Users debugging an unstable
  language server see no crash signal in the status panel. The
  fields also contradict the documented `max_restarts: 3` config
  field on `LspServerConfig` (set at `config.rs:60`, but never read
  by the client either — there is no auto-restart logic).
- Root cause: the fields were scaffolded for a restart-tracking
  feature that was never implemented. The client has no auto-
  restart on crash (cf. F-INT-02-P3-02: the reader task exits on
  EOF without restarting), and no error-path updates `last_error`.
- Direction: Two consistent options:
  - (a) **Implement**: add auto-restart on reader-exit (bounded by
    `config.max_restarts`), increment `restart_count` on each
    restart, set `last_error` on every `Err` path in
    `spawn_process` / `send_request` / reader loop. Surface
    `last_error` in the diagnostics UI.
  - (b) **Delete** (preferred per AGENTS.md code-cleanup unless a
    consumer is wired): remove `restart_count` / `last_error` from
    `StdioLspClient`, `status()`, and `LspServerStatus`. Remove
    `max_restarts` from `LspServerConfig` if no other consumer
    reads it (grep confirms only `config.rs:60` sets it).
- Regression validation: a unit test that drives the client to a
  failure (mock spawn error) and asserts `status().last_error` is
  populated (option a) OR a doctest confirming the fields no
  longer exist (option b).
- Validation reports: [V01-01](../validations/F-INT-02/V01-01.md)

### F-INT-02-P2-02: LSP `send_request` has no per-request timeout

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/lsp/client.rs:202-236`
    `send_request` awaits `rx.await` (a `oneshot::Receiver`) with
    no `tokio::time::timeout` wrapper.
  - The only LSP timeout in the integration is at
    `manager.rs:76-82`, a 15 s timeout around the *initialize*
    handshake.
  - Goto-definition, hover, completion, references, and diagnostics
    calls (`client.rs:338-521`) all call `send_request` directly
    with no per-call timeout.
- Reachability: every LSP tool call after initialization. If a
  language server stops responding to a specific request without
  closing stdout (deadlocked on its own internal computation,
  waiting on a network resource, kernel-paused), the calling
  future parks on `rx.await` forever.
- Expected invariant: a network/IPC protocol call must have a
  bounded timeout so the framework's `CancellationToken` and outer
  agent-loop budgets can reclaim the slot.
- Observed behavior: an unresponsive LSP server can hang the agent
  tool call indefinitely. The framework's React-loop / `CancellationToken`
  will eventually drop the future (freeing the slot at the framework
  layer), but the underlying `pending` entry remains in the
  `StdioLspClient::pending` map until the reader exits — leaking
  memory across many such cancellations within a single LSP
  session.
- Impact: the EKO LSP tools (hover, definition, references,
  completion) become unusable for the affected language server
  after one hung request, because all subsequent requests on the
  same client share the same `pending` map and `next_id` counter —
  they queue up behind the hung id. In practice this manifests as
  "LSP tools work for a while, then permanently hang."
- Root cause: missing per-request `tokio::time::timeout` in
  `send_request`. Initialize got one (`manager.rs:76`); ordinary
  requests did not.
- Direction: wrap `rx.await` (or the whole `send_request` body
  after registering the pending entry) in
  `tokio::time::timeout(Duration::from_secs(30), ...)`. On timeout,
  remove the entry from `pending` (to avoid the oneshot firing
  into a dropped receiver later) and return `LspError::Timeout`.
  Make the timeout configurable via `LspServerConfig` if a
  per-server value is desirable.
- Regression validation: a unit test that registers a pending
  request whose `onesot::Sender` is never invoked, asserts
  `send_request` returns `LspError::Timeout` after the configured
  duration.
- Validation reports: [V01-01](../validations/F-INT-02/V01-01.md)

### F-INT-02-P2-03: `ChannelManager::Drop` only logs; cannot run async cleanup, leaking per-channel background tasks

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/channels/manager.rs:131-141`
    `impl Drop for ChannelManager` only logs the count of
    remaining channels; it cannot call `stop_all` because `Drop`
    cannot `.await`.
  - Each `ChannelPlugin::start` spawns long-lived background tasks
    (`feishu/channel.rs:217, 260, 281`, `qq/channel.rs:108, 136`).
  - The application caller `echo-agent-cli/src/cli/modes.rs:231`
    calls `stop_all()` explicitly, so the normal shutdown path is
    fine. The defect is the *fallback* path: any panic, early
    return, or test that drops `ChannelManager` without `stop_all`
    leaves these tasks running.
- Reachability: any code path that constructs a `ChannelManager`
  and drops it without calling `stop_all` — most visibly in tests,
  but also in any future caller that bails out early on a
  configuration error after `register` but before `start_all` /
  `stop_all`.
- Expected invariant: a framework type that owns spawning long-
  lived tasks should either (a) cancel them on `Drop` via
  `JoinHandle::abort()` (no `.await` needed) or (b) make `stop_all`
  trivially required via a builder that consumes `Self`.
- Observed behavior: dropped `ChannelManager` leaves per-channel
  tasks running until process exit or until their next I/O fails.
  For webhook mode this also leaks a bound TCP listener.
- Impact: leaked tasks and TCP listeners. In the local single-user
  EKO scenario (AGENTS.md) the blast radius is bounded — the
  process exits eventually — but it is observably incorrect
  behavior and a foot-gun for tests / library consumers.
- Root cause: `Drop` cannot `.await`, so the implementation punted
  to log-only. The per-channel `JoinHandle`s are stored inside
  `Box<dyn ChannelPlugin>` and are not reachable from the manager
  without an additional abort API.
- Direction: extend `ChannelPlugin` with a `fn abort_handle(&self)
  -> Option<JoinHandle<()>>` accessor (or equivalent) so
  `ChannelManager::Drop` can iterate and call `.abort()` on each.
  Alternatively, document the requirement that callers must
  `stop_all().await` before drop, and add a `debug_assert!` or
  panic-in-debug that fires when Drop sees a non-empty map.
- Regression validation: a test that constructs a manager with one
  dummy channel, starts it, drops the manager without stop_all,
  and asserts the background task terminated (poll
  `JoinHandle::is_finished()` within a short timeout).
- Validation reports: [V02-01](../validations/F-INT-02/V02-01.md)

### F-INT-02-P3-01: LSP silently drops JSON-RPC responses whose id is a string, leaving pending requests hung forever

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/lsp/jsonrpc.rs:50` declares
    `JsonRpcResponse.id: Option<u64>`.
  - `echo-agent/echo-integration/src/lsp/client.rs:176` extracts
    the id via `value.get("id").and_then(|v| v.as_u64())`.
  - If a server returns `{"id": "1", ...}` (stringified id), the
    `as_u64()` returns `None`, the response-matching branch is not
    entered, the `pending.get(&id).remove(&id)` never fires, and
    the corresponding `oneshot::Sender` is never consumed.
- Reachability: any LSP server that returns string-typed JSON-RPC
  ids. The framework's `StdioLspClient` only ever sends `u64` ids
  (`AtomicU64::fetch_add` at `client.rs:209`), so well-behaved
  servers echo integer ids. Some servers (notably older JVM-based
  LSPs and a few experimental Rust servers) stringify the id.
- Expected invariant: a response to a known request id should
  match regardless of the JSON type used to encode the id, as long
  as it round-trips through the id space the client uses.
- Observed behavior: a stringified-id response is silently
  ignored; the request hangs until the reader task exits.
- Impact: rare; only servers that stringify ids are affected.
  Marked P3 because the framework only sends integer ids, so
  compliant servers are unaffected. But the silent-drop behavior
  is hard to debug for the affected case.
- Root cause: tight coupling between the wire JSON type of the
  response id and the client's id space, without a normalization
  layer.
- Direction: extract the response id more permissively — accept
  `u64`, `i64`, or string-encoded numeric ids via
  `value.get("id").and_then(|v| v.as_u64().or_else(|| v
  .as_str().and_then(|s| s.parse::<u64>().ok())))`. Reject non-
  numeric strings with a debug-log rather than silently dropping.
- Regression validation: a unit test that feeds a string-id
  response through the reader's response branch and asserts the
  pending request resolves.
- Validation reports: [V01-01](../validations/F-INT-02/V01-01.md)

### F-INT-02-P3-02: LSP reader and writer tasks are not cancellable; cancellation requires explicit shutdown or child death

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/lsp/client.rs:98-108` spawns
    the writer task; `client.rs:113-116` spawns the reader task.
    Neither `JoinHandle` is stored on `StdioLspClient`.
  - grep confirms zero `CancellationToken` references under
    `echo-integration/src/lsp/`.
  - The tasks exit naturally only when:
    - Reader: stdout returns `Ok(0)` or err (`client.rs:138-146,
      165-168`).
    - Writer: `writer_rx.recv()` returns None (channel closed) or
      write/flush errors (`client.rs:100-107`).
- Reachability: when a parent agent run is cancelled, the calling
  future is dropped, but the LSP client struct is held alive by
  `Arc<RwLock<StdioLspClient>>` in `LspManager`. The reader /
  writer tasks continue running until the child process dies
  (which only happens via explicit `shutdown()` or drop of the
  client → `kill_on_drop`).
- Expected invariant: framework primitives should integrate with
  `CancellationToken` (F-CORE-01) so that an agent cancellation
  propagates to in-flight IPC reads.
- Observed behavior: agent cancellation drops the request future
  but does not stop the reader / writer. The client lingers in
  the `LspManager` map with stale `running=true`, `initialized=true`
  state (cf. F-INT-02-P2-01).
- Impact: in the local single-user EKO scenario the lingering
  child process is bounded (one per language, killed on
  `shutdown_all`). The blast radius is wasted resources until the
  next explicit shutdown, not data loss.
- Root cause: the LSP integration predates the framework's
  `CancellationToken` discipline and was never retrofitted.
- Direction: thread a `CancellationToken` (e.g. from the agent
  run) into `spawn_process`; on cancel, signal the reader/writer
  tasks and abort them. Add a `JoinHandle` for each so they can
  be explicitly awaited / aborted in `shutdown`.
- Regression validation: a test that constructs a client, spawns
  it, cancels the token, and asserts both tasks' `is_finished()`
  within a short timeout.
- Validation reports: [V01-01](../validations/F-INT-02/V01-01.md)

### F-INT-02-P3-03: Channel `stop()` leaks heartbeat / ping sub-tasks until next I/O fails

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/channels/channels/feishu/channel.rs:302-312`
    `FeishuChannel::stop` calls `task_handle.take().abort()` but
    never invokes `WsClient::stop` (the only path that sets
    `running=false`).
  - `echo-agent/echo-integration/src/channels/channels/feishu/long_poll.rs:262-276`
    spawns a ping task holding `running: self.running.clone()`.
    `long_poll.rs:280` aborts the ping task only when `message_loop`
    returns normally; if the outer task is abort()'d, the ping
    task continues until its next `sink.send(ping)` fails.
  - `echo-agent/echo-integration/src/channels/channels/qq/channel.rs:198-208`
    `QqChannel::stop` aborts `gateway_handle`; the heartbeat task
    spawned at `gateway.rs:73-99` is only aborted on explicit
    close/error/reconnect branches inside `connect_to_gateway`
    (`gateway.rs:175, 202, 208, 216`).
- Reachability: every channel `stop()` call. Worst-case lingering
  window: one ping interval (Feishu default 120 s, `long_poll.rs:32`)
  or one heartbeat interval (QQ, server-negotiated, default 30 s).
- Expected invariant: `stop()` should synchronously abort all
  sub-tasks it spawned.
- Observed behavior: sub-tasks linger for up to one interval after
  stop, consuming one extra network call attempt that fails.
- Impact: small. The lingering tasks exit on their own and do not
  hold user-visible resources beyond a TCP connection that is
  already closed at the parent.
- Root cause: sub-task lifecycle is owned by the parent future's
  body, not by the channel struct. When the parent is abort()'d
  mid-await, the sub-task abort statements never run.
- Direction: have `WsClient::run` return a `CancellationToken` (or
  store the ping task's `JoinHandle` on `WsClient`) and have
  `FeishuChannel::stop` signal it explicitly. Same for QQ
  gateway's heartbeat task.
- Regression validation: a test that starts a channel with a mock
  WebSocket, stops it, and asserts the heartbeat / ping task's
  `is_finished()` within 1 s (well below the ping interval).
- Validation reports: [V02-01](../validations/F-INT-02/V02-01.md)

### F-INT-02-P3-04: `A2AServer` has no `Drop`; in-flight tasks are not cancelled on shutdown

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/src/a2a/server.rs:35-40` declares `A2AServer` with
    no `impl Drop`.
  - The `tasks` and `cancel_tokens` maps (`server.rs:38-39`) hold
    `CancellationToken`s that are only fired by `handle_task_cancel`.
  - On server shutdown (axum graceful shutdown via `serve.rs:158-
    164`), in-flight `agent.execute()` / `agent.execute_stream()`
    futures are dropped (their streams return `None` to clients),
    but the tasks' state remains Working / InputRequired in the
    `tasks` map.
- Reachability: any A2A server shutdown with non-terminal tasks in
  the map. The `cleanup_completed_tasks` helper (`server.rs:605-
  625`) is not invoked at shutdown.
- Expected invariant: on shutdown, in-flight tasks should either
  be cancelled (transition to Canceled) or the application should
  document that "task state is lost on shutdown."
- Observed behavior: in-flight tasks vanish without a state
  transition; clients see a dropped SSE stream (sync request
  clients see a TCP reset).
- Impact: small. A2A is a server-side framework; the application
  using it (`echo-agent-cli/src/cli/modes.rs`) is a CLI process
  whose exit naturally reclaims everything. The defect is a
  correctness gap, not a resource leak.
- Root cause: no explicit shutdown ordering between axum's
  graceful shutdown and the per-task cancel tokens.
- Direction: add an explicit `shutdown(&self)` method on
  `A2AServer` that cancels every non-terminal task's token, and
  wire it before `axum::serve(...).with_graceful_shutdown(...)` in
  `serve.rs`. Alternatively, document that A2A task state is
  best-effort across server restarts.
- Regression validation: a test that starts a streaming task,
  calls `server.shutdown()`, and asserts the task's terminal
  state is Canceled.
- Validation reports: [V03-01](../validations/F-INT-02/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | LSP lifecycle (client.rs, manager.rs, config.rs, jsonrpc.rs): start / stop / drop / status | yes | failed | [V01-01](../validations/F-INT-02/V01-01.md) |
| V02 | Channels lifecycle (Feishu LongPoll / Webhook, QQ Gateway): reconnect, dedup, HMAC verify, cleanup | yes | failed | [V02-01](../validations/F-INT-02/V02-01.md) |
| V03 | A2A lifecycle (server.rs, serve.rs, auth.rs): Agent Card, TaskState matrix, JWT, cancel honoring | yes | failed | [V03-01](../validations/F-INT-02/V03-01.md) |
| V04 | Worker-terminology search per AGENTS.md Subagent-only rule | yes | passed | [V04-01](../validations/F-INT-02/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | — |

V05 is not applicable: no prior F-INT-02 report exists in this
reviewer's directory at the time of writing; this is the first
F-INT-02 report. The historical claims inspected here come from
`AGENTS.md` itself, classified under "Historical Claim Status"
below.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AGENTS.md`: "MCP / LSP are user-configured, don't over-gate with permissions" | current | The framework LSP and channel layers (`echo-integration/src/lsp/`, `echo-integration/src/channels/`, `src/a2a/`) contain no permission gates. The application LSP wiring (`echo-agent-cli/echo-agent-app-core/src/runtime.rs:499-590`) constructs servers directly with no permission check. Consistent. |
| `AGENTS.md`: "统一术语:只有 Subagent,没有 Worker" | current | V04 confirms zero non-false-positive worker hits in the F-INT-02 scope. The 10 grep matches are all substrings inside `NetworkError`. |
| `AGENTS.md`: "no-panic rule" (no `.unwrap()` / `.expect()` / byte slicing on Unicode) | current (with one caveat) | LSP / channels / A2A code paths use `unwrap_or`, `unwrap_or_default`, `ok_or`, `chars().take()` for Unicode truncation. The only byte slicing is `parse_content_length` (`jsonrpc.rs:90-97`), which is safe because the prefix `"content-length:"` is ASCII and is the matched prefix. No panic-prone APIs found. |
| `echo-integration/src/lsp/mod.rs:1-15` doc: "LspManager manages multiple language server processes" | current | `LspManager` (`manager.rs:20-181`) does manage a per-language map. The "automatic restart" implicit in typical LSP managers is **not** implemented — see F-INT-02-P2-01. |
| `src/a2a/mod.rs:13-18` doc: "Any non-terminal state → canceled" | current as spec; **partially regressed** in implementation | The state machine enum (`types.rs:362-378`) and streaming path honor this. The sync `tasks/send` path does not — see F-INT-02-P1-01. |

## Coverage And Uncertainty

**Code not inspected:**
- The LSP `tools/lsp.rs` framework tool registration (only its
  caller in `runtime.rs:578-` was traced). The tool definitions
  are reviewed under F-EXT-01 / F-EXT-02.
- The full HTTP-layer details of `feishu/api.rs` send-message /
  reply-message / patch-card / add-reaction functions beyond the
  error-classification read done here. They are simple `reqwest`
  POSTs with error mapping; no finding.
- The exact contents of `feishu/proto.rs` beyond field/tag
  definitions. The protobuf is `prost`-derived; framing was
  spot-checked at the `handle_binary_frame` call site
  (`long_poll.rs:326-349`).
- The `A2AClient` SSE parser's behavior under malformed chunk
  boundaries beyond the read done here. It joins chunks on `\n\n`
  boundaries and trims lines (`client.rs:244-292`); UTF-8 decode
  failures are logged-and-skipped.

**Validations not available:**
- No executable end-to-end test against a real LSP / Feishu / QQ /
  A2A server was run (would require network credentials or
  spawning fixture servers). V01/V02/V03 are therefore static
  analyses; the findings rest on code reading, not on reproduced
  failures. The Feishu WebSocket smoke path in particular would
  benefit from a mocked-server integration test, but that is out
  of scope for this review.

**Claims that remain uncertain:**
- F-INT-02-P3-01 (string-id JSON-RPC responses) — the defect is
  real by code inspection, but I could not enumerate which
  LSP servers in the wild actually stringify ids. Marked P3 / medium
  confidence for this reason.
- F-INT-02-P3-04 (A2A shutdown ordering) — the blast radius
  depends on whether any A2A consumer relies on task state
  surviving server restart; if none do, this is purely a
  correctness nit. Marked P3 / medium.

## Handoff

**Conclusions downstream tasks may rely on:**
- The framework LSP / channels / A2A integrations correctly isolate
  their external protocols (JSON-RPC stdio, PBBP2 / Discord-style
  WebSocket, axum HTTP) behind typed framework traits (`LspClient`,
  `ChannelPlugin` / `MessageHandler`, `AgentCard` / `TaskState`).
  Downstream tasks (A-INT-01, A-SRF-04, X-BND-01) can rely on the
  trait boundaries; they only need to review application policy on
  top.
- The Feishu / QQ reconnect logic (bounded exponential backoff with
  reset-on-stable, dedup TTLs, fragment reassembly TTLs) is sound.
  This is a **better discipline** than MCP's SSE transport (cf.
  F-INT-01-P2-01 / -P2-02) and downstream tasks can cite it as a
  positive reference.
- The A2A `TaskState` enum and its `can_transition_to` matrix are
  the single authority for the A2A protocol's state machine
  (`types.rs:355-378`). Downstream tasks can rely on this matrix.
- The JWT middleware correctly restricts algorithms, redacts the
  secret in `Debug`, and applies the middleware to the right
  routes. Downstream security review (F-SEC-01, X-AUT-01) can rely
  on this for the A2A layer.
- The F-INT-02 scope fully complies with the Subagent-only
  terminology rule (V04). Downstream X-INV-01 may skip this scope
  or sample-confirm.

**Reports they must read:**
- This report + [V01-01](../validations/F-INT-02/V01-01.md),
  [V02-01](../validations/F-INT-02/V02-01.md),
  [V03-01](../validations/F-INT-02/V03-01.md),
  [V04-01](../validations/F-INT-02/V04-01.md).
- F-CORE-01 (this reviewer) for the framework `CancellationToken`
  background that F-INT-02-P3-02 builds on.
- F-INT-01 (this reviewer) for the MCP transport-level comparison
  (F-INT-02's channel reconnect discipline is consciously better).

**Conditions that make this report stale:**
- Any change to `echo-integration/src/lsp/client.rs` that adds a
  per-request timeout, restart tracking, or `CancellationToken`
  threading — would supersede F-INT-02-P2-01 / -P2-02 / -P3-02.
- Any change to `src/a2a/server.rs:handle_task_send` that routes
  terminal writes through `update_task_state` and/or wraps
  `agent.execute()` in a `select!` against the cancel token —
  would resolve F-INT-02-P1-01.
- Any change to `echo-integration/src/channels/manager.rs` that
  adds an `abort()`-based cleanup in `Drop` — would resolve
  F-INT-02-P2-03.

**Follow-up task IDs (no fixes implemented in this review):**
- Open one task for F-INT-02-P1-01 (A2A sync-send cancel race) —
  P1, isolated to `src/a2a/server.rs`, low regression risk if
  tests are added.
- Bundle F-INT-02-P2-01 / -P2-02 / -P3-01 / -P3-02 into one LSP
  resilience patch (status-field dead-code + per-request timeout +
  string-id handling + cancellation). Decide between
  delete-the-fields vs. implement-restart-tracking for P2-01
  before starting.
- Bundle F-INT-02-P2-03 / -P3-03 into one channels-cleanup patch
  (Drop abort + sub-task handle exposure).
- F-INT-02-P3-04 (A2A shutdown) is a localized correctness fix;
  bundle into the A2A resilience task or fold into the F-INT-02-P1-01
  patch since both touch `src/a2a/server.rs`.
