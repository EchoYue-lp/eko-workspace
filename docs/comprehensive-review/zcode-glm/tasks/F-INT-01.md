# F-INT-01: MCP integration

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean

## Question

Does MCP configuration, client/server transport, tool adaptation,
cancellation, reconnect, and schema handling preserve framework contracts?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-integration/src/mcp/mod.rs` — `McpManager` (multi-server
  facade).
- `echo-agent/echo-integration/src/mcp/client.rs` — `McpClient` lifecycle
  (initialize → discover → call → close).
- `echo-agent/echo-integration/src/mcp/tool_adapter.rs` — `McpToolAdapter`
  (MCP tool → framework `Tool` trait).
- `echo-agent/echo-integration/src/mcp/transport/mod.rs` — `McpTransport`
  trait.
- `echo-agent/echo-integration/src/mcp/transport/stdio.rs` —
  `StdioTransport` (subprocess).
- `echo-agent/echo-integration/src/mcp/transport/sse.rs` — `SseTransport`
  (legacy HTTP+SSE).
- `echo-agent/echo-integration/src/mcp/transport/http.rs` — `HttpTransport`
  (Streamable HTTP).
- `echo-agent/echo-integration/src/mcp/config_loader.rs` — `McpConfigFile`
  / `McpServerEntry` mcp.json parsing.
- `echo-agent/echo-integration/src/mcp/server_config.rs` —
  `McpServerConfig` / `TransportConfig`.
- `echo-agent/echo-integration/src/mcp/types.rs` — JSON-RPC + MCP schema
  types.
- `echo-agent/echo-integration/src/mcp/server.rs` — `McpServer` (framework
  `Tool` exposed as MCP server, header + notification handler only).
- `echo-agent/src/agent/react/capabilities.rs:1124-1332` — framework seam
  exposing MCP to agents (`connect_mcp_from_config`, `disconnect_mcp`,
  `list_mcp_servers`, `mcp_client`).
- `echo-agent/echo-core/src/error.rs:230-249` — `McpError` variants.
- `echo-agent/echo-core/src/tools/mod.rs:165-253` — `ToolFailure::from_error`
  McpError arms (classification).
- `echo-agent-cli/echo-agent-app-core/src/state.rs:281-793` — application
  MCP state (config, health) + `run_mcp_health_check`.
- `echo-agent-cli/src/tauri/commands/mcp.rs` — Tauri commands
  `connect_mcp_server`, `disconnect_mcp_server`, `list_mcp_servers`, and
  the IPC-triggered reconnect path.

## Out Of Scope

- Permission / risk-gating runtime behavior applied to MCP tools — the
  adapter exposes `risk_level()` but the gate itself is reviewed under the
  permission / risk-gating task.
- Tauri command DTO ↔ frontend type parity for the MCP panel — deferred to
  A-FE-01.
- The `McpServer` server-side full request loop (only the notification
  handler was inspected for cancellation semantics; full server side is
  exercised by external MCP clients and is a separate concern).
- Per-provider tool-call serialization of the adapted `parameters()` JSON
  Schema onto OpenAI/Anthropic wire formats — deferred to F-LLM-01 /
  F-LLM-02 / F-LLM-03.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — MCP is user-configured, no over-gating;
    framework-vs-application layering; no-panic rule; UTF-8 safety;
    code-cleanup rule.
  - `docs/comprehensive-review/REPORTING.md` — finding/validation contracts.
  - `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
  - `docs/comprehensive-review/TASKS.md` — F-INT-01 task spec.
- Dependency task reports read:
  - `F-CORE-01` (zcode-glm) — relied on its conclusion that
    `CancellationToken` is the framework's canonical cancellation primitive
    and is threaded through the agent runtime; used here to assess whether
    MCP transport cancellations are consistent.
  - `F-EXT-01` (zcode-glm) — relied on its conclusion that
    `Tool::execute(parameters) -> BoxFuture<Result<ToolResult>>` is the
    single typed tool contract, that `ToolFailure::from_error` is the
    canonical error-classifier, and that **the framework `Tool::execute`
    does not accept a `CancellationToken`** (limitation flagged in
    F-EXT-01). Used here to assess cancellation in the MCP adapter.
- Historical documents treated as hypotheses: none.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. MCP client/transport/adapter/types/server are pure protocol implementations — any `echo-agent` consumer that wants to talk to an MCP server, or expose its tools as an MCP server, needs them. Lives correctly in `echo-integration` (a generic integration crate), not in `echo-core` (which only defines `McpError` under the `mcp` feature). V01 confirms single definition site. |
| EKO product policy | None at this layer. The adapter exposes `risk_level()` to feed the generic permission system; EKO-specific gating is the application's job (out of scope). Health-check surfaces (`McpHealthStatus` in `state.rs:281-295`) live in the application, not the framework. |
| Adapter boundary | `McpToolAdapter` is a thin adapter: it converts `McpToolCallResult` to `ToolResult` and forwards `parameters()` verbatim. It does **not** own a scheduler, registry, or DAG. The framework seam (`AgentCapabilities::connect_mcp_from_config`) only delegates to `McpManager::connect` + `add_tools`. |
| Duplicate search | Searched names: `McpManager`, `McpClient`, `McpToolAdapter`, `McpTransport`, `StdioTransport`, `HttpTransport`, `SseTransport`, `McpConfigFile`, `McpServerEntry`, `McpServerConfig`, `TransportConfig`, `connect_mcp_from_config`, `disconnect_mcp`, `list_mcp_servers`, `mcp_client`. Result: each is defined exactly once (V01). Application only consumes via `echo_agent::mcp::*` re-exports. |
| Migration deletion | No migration proposed. The `McpServer` server-side `notifications/cancelled` handler is a no-op (just logs) — recorded as F-INT-01-P2-03 but no deletion target. |

## Current Path

Verified MCP data flow at commit `9b0e0fa`:

1. **Configuration ingestion.** User-provided mcp.json reaches the framework
   via two paths:
   - **CLI startup** (`state.rs` + `config_discovery.rs`): `McpConfigFile`
     loaded from `~/.eko/mcp.json` or `<root>/.mcp.json`, stored in
     `PluginState::mcp_config` (`state.rs:357`).
   - **Tauri IPC** (`tauri/commands/mcp.rs:481`): the frontend `save` →
     `serde_json::from_value` into `McpConfigFile`, then a background task
     (`mcp.rs:493-545`) disconnects every existing server and reconnects
     each enabled one with a 15 s per-server timeout.

2. **Connection.** The application calls
   `AgentCapabilities::connect_mcp_from_config(McpServerConfig)`
   (`capabilities.rs:1149`). If a same-named client already exists it first
   calls `disconnect_mcp` (which also removes the agent's tool
   registrations — `capabilities.rs:1315-1332`). Then
   `McpManager::connect` (`mod.rs:73-100`) constructs an
   `Arc<McpClient>`, fetches tools, wraps each in `McpToolAdapter` with
   `with_server_name`, and returns `Vec<Box<dyn Tool>>` which the agent
   registers via `add_tools`.

3. **Transport selection** (`client.rs:48-57`):
   - `TransportConfig::Stdio` → `StdioTransport::new` (spawn subprocess,
     validate command, install stdout/stderr router tasks).
   - `TransportConfig::Http` → `HttpTransport::new` (Streamable HTTP,
     retry on transient errors, Mcp-Session-Id tracking).
   - `TransportConfig::Sse` → `SseTransport::new` (legacy 2024-11-05,
     background SSE reader task with reconnect budget).

4. **Handshake** (`client.rs:65-115`): JSON-RPC `initialize` with
   `protocolVersion = "2025-11-25"` (types.rs:5) + client capabilities
   (roots/sampling/elicitation); on success, the negotiated version is
   stored; then `notifications/initialized` is sent.

5. **Capability-gated discovery** (`client.rs:117-145`): `tools`,
   `resources`, `prompts` are fetched only if `server_capabilities`
   advertises them. Each fetcher paginates with `next_cursor` and a hard
   `MAX_PAGINATION = 100` pages + 30 s per-page timeout.

6. **Tool call.** `McpToolAdapter::execute` (`tool_adapter.rs:148-209`)
   forwards `ToolParameters` as `serde_json::Value::Object` to
   `McpClient::call_tool`, which issues `tools/call` JSON-RPC. The result
   is mapped onto `ToolResult` with one of:
   - `Ok(Err(e))` → `ToolResult::error(e) + ToolFailure::from_error`,
     tagged `result_type = "protocol_error"`.
   - `Ok(Ok(result)) where is_error` →
     `ToolResult::failure(Permanent, text)`, optionally with
     `StructuredError { error_code: "mcp_is_error" }` and `data` if
     structured content is present.
   - `Ok(Ok(result)) success` → `ToolResult::success_json(structured)`
     or `ToolResult::success(text)`. Non-standard `extra` fields are
     appended to `output` under `"\n\n附加字段:\n"`.
   The framework error classifier `ToolFailure::from_error`
   (`echo-core/src/tools/mod.rs:227-250`) handles every `McpError` variant
   explicitly: `ConnectionFailed`/`InitializationFailed`/`TransportClosed`
   → `Unavailable` (retryable when no side effects); `ToolCallFailed {
   code: -32602 }` → `InvalidArguments`; `-32603`/`-32099..=-32000` →
   `Transient` retryable; everything else → `Permanent`. This is the
   safety-critical retry classification.

7. **Disconnection / cleanup.**
   - User-initiated: `disconnect_mcp(name)` (`capabilities.rs:1315`)
     computes the adapter's exposed tool names, removes each from the
     `ToolManager`, then calls `McpManager::disconnect`
     (`mod.rs:140-153`) which removes the map entry and calls
     `client.close()` → `transport.close()`.
   - Stdio close (`stdio.rs:339-353`): `child.kill().await` +
     `child.wait().await`. The stdout router task (`stdio.rs:96-138`)
     drains pending requests with a synthetic -32000 error before the
     stream ends.
   - SSE close (`sse.rs:411-420`): `cancel_token.cancel()` signals the
     background task; `Drop` (`sse.rs:428-432`) cancels again.
   - HTTP close (`http.rs:227-237`): sends `notifications/cancelled`
     fire-and-forget, clears the `pending` map.

## Findings

### F-INT-01-P1-01: HttpTransport advertises a notification channel it never feeds; 202-Accepted branch hangs to 60 s timeout

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/mcp/transport/http.rs:51` declares
    `notification_tx: broadcast::Sender<JsonRpcNotification>`.
  - `http.rs:240-244` returns `Some(Arc::new(NotificationReceiver::new(
    self.notification_tx.subscribe())))` from `notification_rx()`.
  - `http.rs:158` advertises `Accept: application/json, text/event-stream`.
  - **`http.rs` never calls `notification_tx.send(...)` anywhere**
    (confirmed by grep across the file). The channel is dead.
  - `http.rs:195-211` waits on a `oneshot::Receiver` for the request id
    on `status == 202`, but nothing in the file can fulfill that
    receiver — the synchronous-response path is *not* taken on 202.
- Reachability: `McpClient::call_tool` (`client.rs:218-237`) →
  `transport.send(req)` → on a Streamable-HTTP server that returns
  `202 Accepted` (the spec-compliant way to handle async tool calls),
  the future parks on `rx` and only resolves after the 60 s timeout at
  `http.rs:201`, producing
  `McpError::ProtocolError("等待 HTTP 异步响应超时 (id=...)")`.
  Any caller that subscribes to `notification_rx()` for server-push
  notifications will also block forever (channel empty).
- Expected invariant: if a transport returns `Some` from
  `notification_rx()`, server-pushed notifications and 202-routed async
  responses must reach that channel.
- Observed behavior: 202 hangs to timeout; notifications silently
  dropped; advertised capability is unimplemented.
- Impact: Any MCP server using the Streamable-HTTP async pattern (servers
  that delegate tool execution to a backend and POST back later) is
  unusable through this client. Tool calls appear to "succeed after
  60 s" with a misleading timeout error, hiding the real server
  response. Health-check loops that depend on server-push
  notifications (`notifications/*`) will see none.
- Root cause: incomplete implementation — the HTTP transport was
  written for the synchronous-response path only. The 202 + GET-SSE
  listener half of the Streamable-HTTP spec was scaffolded (channel
  field, `Accept` header, `notification_rx` return) but never wired.
- Direction: Either (a) implement the GET-SSE listener task that feeds
  `notification_tx` and routes 202 responses to `pending[id]` (mirrors
  `SseTransport::run_sse_loop`), or (b) drop the dead scaffold: remove
  `notification_tx`, return `None` from `notification_rx()`, and return
  `McpError::ProtocolError("server returned 202 but HTTP transport
  does not implement async responses")` immediately on 202 instead of
  waiting 60 s. Per AGENTS.md code-cleanup, pick one and remove the
  other half.
- Regression validation: a unit test that spins up a mock HTTP server
  returning `202 Accepted` and asserts the client fails fast (or
  delivers the async response) rather than hanging 60 s.
- Validation reports: [V03-01](../validations/F-INT-01/V03-01.md)

### F-INT-01-P2-01: SseTransport terminates permanently on clean stream close

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/mcp/transport/sse.rs:78-86`:
    ```rust
    match Self::run_sse_loop(...).await {
        Ok(_) => { break; }  // !!! clean stream end -> permanent exit
        Err(e) => { retry_count += 1; ... }
    }
    ```
  - `run_sse_loop` returns `Ok(())` when `response.bytes_stream()` ends
    (`sse.rs:227-228`, the `while let Some(chunk)` falls through) — this
    is the normal "server closed the SSE connection" path.
- Reachability: `McpClient::new` with a `TransportConfig::Sse`
  (`client.rs:54-56`) → `SseTransport::new` spawns the background task
  → any later clean close from the server side (load-balancer idle
  timeout, server restart, deliberate keep-alive reset) parks the
  transport in a permanently-broken state with no error surfaced. The
  client has no way to know it is disconnected.
- Expected invariant: a transient loss of the long-lived SSE
  connection should trigger reconnect, not termination. The function
  comment "SSE 连接正常关闭" (sse.rs:81) mischaracterizes a server-side
  close as user-intended.
- Observed behavior: server-side close → background task exits →
  subsequent `send()` calls fail with
  `McpError::ProtocolError("SSE: 尚未获取到 POST 端点 URI...")` once
  `message_endpoint` is reset on next (never-attempted) reconnect.
- Impact: legacy SSE servers (the target audience of this transport,
  per `sse.rs:4`) become unreachable mid-session whenever the
  connection closes cleanly. Users must manually reconnect. The
  `run_mcp_health_check` loop (`state.rs:751`) will report healthy on
  the client side until it next tries to send.
- Root cause: the outer loop's match conflates "stream ended cleanly"
  with "user asked to close". The distinction between an internal
  EOF and an external cancellation is already available via the
  `cancel_token` — a clean EOF that was not cancellation should be a
  retryable condition.
- Direction: After `run_sse_loop` returns `Ok(())`, check
  `cancel.is_cancelled()` (the only legitimate path to a clean exit).
  If not cancelled, treat it as an error and enter the retry branch.
  Reset `retry_count` after each successful run (see F-INT-01-P2-02).
- Regression validation: a unit test using a mock SSE server that
  closes the stream mid-session and asserts the transport reconnects.
- Validation reports: [V03-01](../validations/F-INT-01/V03-01.md)

### F-INT-01-P2-02: SseTransport retry budget is never reset; 5 lifetime failures is permanent

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/mcp/transport/sse.rs:67`:
    `let mut retry_count: u32 = 0;`
  - `sse.rs:83-93`: `retry_count += 1` on each error; condition
    `retry_count >= MAX_RETRIES` (5) breaks the loop.
  - **No `retry_count = 0` assignment anywhere in the file.**
- Reachability: same path as F-INT-01-P2-01. Once any five transient
  errors accumulate over the lifetime of the client — even spaced
  hours apart with successful reconnects in between — the transport
  permanently gives up.
- Expected invariant: a retry budget should reset after a successful
  operation, so that the budget bounds a *failure burst*, not the
  client's lifetime.
- Observed behavior: budget is monotonic; the comment "达到最大重试
  次数 (5)" (`sse.rs:73`) is reached even when the connection has been
  healthy for hours between failures.
- Impact: long-running sessions against flaky SSE servers quietly lose
  MCP tool availability after a small number of unrelated incidents.
  Compounds with F-INT-01-P2-01 (clean closes aren't retried at all).
- Root cause: missing reset on success.
- Direction: After any `Ok(_)` from `run_sse_loop` that is not a
  cancellation, reset `retry_count = 0` before reconnecting.
- Regression validation: a unit test that triggers 5 errors
  interleaved with successful runs and asserts the transport is still
  alive.
- Validation reports: [V03-01](../validations/F-INT-01/V03-01.md)

### F-INT-01-P2-03: No MCP tool-call cancellation; server-side `notifications/cancelled` is a no-op

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/mcp/tool_adapter.rs:148-209`:
    `execute` has no `CancellationToken` parameter (consistent with
    the framework `Tool::execute` signature per F-EXT-01).
  - `echo-agent/echo-integration/src/mcp/client.rs:218-237`:
    `call_tool` has no cancel hook.
  - Transports: no `CancellationToken` field on `StdioTransport` or
    `HttpTransport`; only `SseTransport` has one and it only cancels
    the background reader, not an in-flight `send`.
  - `echo-agent/echo-integration/src/mcp/server.rs:306-308`:
    `"notifications/cancelled" => { tracing::debug!(...); }` — the
    server side receives the cancellation notification and only logs;
    there is no map of in-flight requests to abort.
- Reachability: any tool call into an MCP server. Once `tools/call` is
  in flight, the only ways it resolves are (a) the server replies, or
  (b) the transport's per-request timeout (120 s for stdio at
  `stdio.rs:298`, 60 s for HTTP at `http.rs:207`, 30 s for SSE at
  `sse.rs:389`).
- Expected invariant: a long-running agent that is cancelled (e.g.,
  the chat turn is cancelled by the user, or the framework's
  `CancellationToken` fires) should propagate cancellation to MCP
  tool calls so the server can stop work and the client can free the
  request slot.
- Observed behavior: cancellation of the parent task drops the future
  but the in-flight JSON-RPC request remains registered in the
  transport's `pending` map; the response (when it eventually arrives)
  is routed to a dropped `oneshot::Sender` and silently discarded.
  The server keeps working.
- Impact: cancelling an agent run does not cancel expensive MCP tool
  calls (e.g. long-running code-execution servers). For stdio servers
  this also leaves the subprocess consuming resources until its own
  internal timeout.
- Root cause: the framework `Tool::execute` signature lacks a
  `CancellationToken` (F-EXT-01 limitation), and even where tokens
  exist (`SseTransport`) they are not threaded into `send`. The server
  side does not maintain an in-flight request table to honor
  `notifications/cancelled`.
- Direction: This is primarily a framework-level decision (extend
  `Tool::execute` to accept a `CancellationToken`, or use the
  `execute_with_context` variant that already receives a
  `ToolContext`). MCP-specific follow-up: when cancellation reaches
  the adapter, send `notifications/cancelled` with the request id to
  the server, and add a server-side in-flight request map to honor
  it. Out of scope for this review task; flag as a downstream
  cross-cutting concern.
- Regression validation: a test that cancels an in-flight `tools/call`
  and asserts (a) the client future resolves as Cancelled promptly,
  (b) the server receives `notifications/cancelled`.
- Validation reports: [V03-01](../validations/F-INT-01/V03-01.md)

### F-INT-01-P3-01: McpToolAdapter drops optional metadata (output_schema, title, icons, meta, execution)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/mcp/types.rs:261-287` defines
    `McpTool` with `output_schema: Option<Value>`, `title`, `icons`,
    `meta`, `execution`.
  - `echo-agent/echo-integration/src/mcp/tool_adapter.rs:113-120` only
    stores `client`, `tool` (whole struct), `server_name`, and
    `exposed_name`. `name()`, `description()`, `parameters()`,
    `risk_level()` only ever read `tool.name`, `tool.description`,
    `tool.input_schema`, `tool.annotations`.
- Reachability: every adapted MCP tool.
- Expected invariant: not strictly required — the framework `Tool`
  trait only consumes name/description/parameters/risk_level. But
  `output_schema` could be used to validate `structured_content`
  against the declared schema before returning it to the caller.
- Observed behavior: `output_schema` is silently dropped. Server
  metadata (icons, title) is not surfaced, even via `ToolResult::
  metadata`, which would be a natural home.
- Impact: cosmetic / observability. No data-integrity risk because
  `structured_content` is preserved verbatim in `ToolResult::data`.
- Root cause: adapter was written before the MCP 2025-11-25 superset
  fields were added to `McpTool`; the new fields were never wired.
- Direction: optionally surface `title` via `ToolResult::metadata` and
  optionally validate `structured_content` against `output_schema`
  using `jsonschema` when present. Low priority.
- Regression validation: unit test asserting `parameters()` equals
  the server's `input_schema` and `output_schema` validation accepts
  a conforming `structured_content`.
- Validation reports: [V02-01](../validations/F-INT-01/V02-01.md)

### F-INT-01-P3-02: StdioTransport Drop best-effort kill may leak the subprocess if the runtime is gone

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/src/mcp/transport/stdio.rs:362-372`:
    `Drop` calls `tokio::spawn(async move { child.kill().await; ... })`.
  - If the tokio runtime has already shut down (process exit, or the
    transport is dropped after the runtime in shutdown ordering),
    `tokio::spawn` silently fails to enqueue and the child is leaked.
- Reachability: only during runtime shutdown, in a specific drop
  ordering.
- Expected invariant: documented behavior should match the
  implementation. The comment "Drop 时尝试关闭，清理子进程"
  overstates the guarantee.
- Observed behavior: best-effort kill; no fallback if no runtime.
- Impact: rare orphan subprocess during abnormal shutdown. Local
  single-user scenario per AGENTS.md — minimal blast radius.
- Root cause: `Drop` cannot `.await`, so the implementation defers to
  a runtime task that may not run.
- Direction: either (a) tighten the comment to "best-effort kill if
  a runtime is available" or (b) on Drop, additionally attempt a
  blocking `kill()` via the raw OS pid cached at spawn time. Low
  priority.
- Regression validation: not material to a regression test; review
  the doc comment change.
- Validation reports: [V03-01](../validations/F-INT-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search | yes | passed | [V01-01](../validations/F-INT-01/V01-01.md) |
| V02 | Tool-adapter schema mapping + risk classification | yes | passed | [V02-01](../validations/F-INT-01/V02-01.md) |
| V03 | Transport error handling, reconnect, cancellation | yes | failed | [V03-01](../validations/F-INT-01/V03-01.md) |
| V04 | mcp.json parsing and malformed-config handling | yes | passed | [V04-01](../validations/F-INT-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | — |

V05 is not applicable: no prior F-INT-01 report exists in this
reviewer's directory or in `codex/` to compare against at the time of
writing; this is the first F-INT-01 report.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AGENTS.md`: "MCP is user-configured, don't over-gate with permissions" | current | `StdioTransport::validate_mcp_command` (`stdio.rs:311-344`) and `validate_stdio_command` (`config_loader.rs:236-261`) do lightweight validation only; no permission-level gating in the framework. Application's `tauri/commands/mcp.rs` has the connect command on the default path. |
| `AGENTS.md` historical lesson: "require_full_auto on connect_mcp_server caused MCP to be unreachable in default mode, gate removed" | current | `tauri/commands/mcp.rs:211 connect_mcp_server` does not gate on `full-auto`; the comment at `mcp.rs:113-120` documents the historical over-gating and its removal. |
| `echo-integration/src/mcp/mod.rs` doc: "完整实现 MCP 协议" (full MCP protocol implementation) | regressed (partial) | The client side is complete for stdio + sync-HTTP + SSE-legacy. The Streamable-HTTP async path (202 + GET-SSE) is scaffolded but not implemented (F-INT-01-P1-01). The doc overstates coverage. |

## Coverage And Uncertainty

**Code not inspected:**
- `McpServer::serve_stdio` full request loop (`server.rs` beyond the
  notification handler at lines 290-313). The server side is exercised
  by external MCP clients (Claude Desktop, Cursor) and was only spot-
  checked for cancellation handling.
- `McpResource`/`McpPrompt`/`McpResourceReadResult` types and the
  resource/prompt code paths in `client.rs:282-380`. They mirror the
  tool path structurally; no separate finding.
- LSP-style `tools/list_changed` handling: the client does not appear
  to subscribe to this notification; `refresh_tools` is a manual pull.
  Not investigated in depth; out of scope.

**Validations not available:**
- No executable end-to-end test against a real MCP server was run
  (would require spawning a fixture server). V03 is therefore a static
  analysis of the reconnect/cancel paths; the findings rest on code
  reading, not on a reproducible failure.

**Claims that remain uncertain:**
- Whether any production deployment actually returns `202 Accepted`
  from an MCP server in a way that exercises F-INT-01-P1-01. The defect
  is real by code inspection, but its blast radius depends on which
  servers EKO users actually configure. Marked P1 because the
  *contract* is violated regardless of current usage.

## Handoff

**Conclusions downstream tasks may rely on:**
- The MCP tool adapter (`McpToolAdapter`) is a correct, thin
  implementation of the framework `Tool` contract. Downstream tasks
  can rely on `parameters()` being the server's JSON Schema verbatim
  and on `ToolResult` faithfully representing `is_error` /
  `structured_content`. (Used by F-LLM-01/02/03 for provider tool-call
  serialization, and by A-FE-02 for tool-result rendering.)
- The framework error classifier `ToolFailure::from_error` covers every
  `McpError` variant explicitly; downstream retry/recovery logic can
  rely on the category mapping at `echo-core/src/tools/mod.rs:227-250`.
- `mcp.json` parsing is panic-free and surfaces structured errors;
  downstream tasks can rely on `McpConfigFile::parse` / `from_file` /
  `to_server_configs` to never panic on user input. (Used by A-FE-01
  for IPC DTO design.)

**Reports they must read:**
- This report + [V01-01](../validations/F-INT-01/V01-01.md),
  [V02-01](../validations/F-INT-01/V02-01.md),
  [V03-01](../validations/F-INT-01/V03-01.md),
  [V04-01](../validations/F-INT-01/V04-01.md).
- F-EXT-01 (this reviewer) for the framework `Tool` /
  `CancellationToken` background that F-INT-01-P2-03 builds on.

**Conditions that make this report stale:**
- Any change to `echo-integration/src/mcp/transport/{http,sse}.rs` —
  the P1/P2 findings are tightly anchored to the current
  implementations.
- Any change to the framework `Tool::execute` signature to add a
  `CancellationToken` — would supersede F-INT-01-P2-03's "framework
  limitation" framing.
- Any addition of a GET-SSE listener task in `HttpTransport` — would
  resolve F-INT-01-P1-01.

**Follow-up task IDs (no fixes implemented in this review):**
- Open a dedicated cleanup task for F-INT-01-P1-01 (HTTP transport
  async-path scaffolding): either complete the GET-SSE listener or
  remove the dead `notification_tx` + 202 branch.
- Fold F-INT-01-P2-01 and F-INT-01-P2-02 into one SSE-resilience
  patch (clean-close retry + budget reset).
- F-INT-01-P2-03 (cancellation) is cross-cutting with F-EXT-01's
  `Tool::execute` cancellation finding; track together.
- F-INT-01-P3-01 / P3-02 are localized cleanups; bundle into a
  maintenance task.
