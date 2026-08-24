# F-INT-01: MCP integration

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Does MCP configuration, client/server transport, tool adaptation, cancellation,
reconnect, and schema handling preserve framework contracts?

## Scope

- `echo-integration/src/mcp/` — full reads: `mod.rs` (McpManager),
  `client.rs` (McpClient, handshake, pagination), `types.rs` (wire types,
  notification receiver), `config_loader.rs` (mcp.json, command validation),
  `server_config.rs` (TransportConfig), `tool_adapter.rs` (McpToolAdapter),
  `server.rs` (McpServer + builder + tests), `transport/{mod,stdio,http,sse}.rs`.
- Root MCP facade: `echo-agent/src/mcp.rs`, `src/lib.rs:88-91` (module gate)
  and `:290` (`advanced` re-exports).
- Framework Agent MCP surface: `src/agent/react/capabilities.rs:1133-1331`
  (register_mcp_tools / connect_mcp_from_config / connect_mcp_from_json /
  load_mcp_from_file / load_mcp_config / disconnect_mcp),
  `src/agent/react/subsystems/tool_exec.rs:38,74-77`,
  `src/agent/react/mod.rs:513,2625` (manager construction/reset),
  `src/plugin.rs:311-345,429-441` (plugin MCP wiring).
- Error/schema contract: `echo-core/src/error.rs:233-259` (McpError),
  `echo-core/src/tools/mod.rs:154-260` (ToolFailure::from_error /
  allows_automatic_retry), `echo-integration/Cargo.toml` (mcp feature).
- EKO consumers (reachability + duplicate check only):
  `echo-agent-cli/echo-agent-app-core/src/{infra.rs:1091-1103,state.rs:757-773,
  browser/mod.rs,sidecar.rs:35-66}`, `src/tauri/commands/mcp.rs`,
  `src/tui/events.rs:3434-3450`, `types/{response.rs:59,request.rs:35}`.

## Out Of Scope

- EKO MCP product policy, GUI/TUI/Browser flows and permission gates ->
  A-INT-01 (framework-side review only; the CLI `connect_mcp_server` gate
  question is checked here only to confirm the historical `require_full_auto`
  over-gating was not reintroduced: `src/tauri/commands/mcp.rs:211-247` runs
  un-gated with input validation, consistent with AGENTS.md).
- LLM provider tool-schema conversion of MCP schemas -> F-LLM-01/02.
- Browser-sidecar retry policy (`browser/mod.rs:875-911`) -> A-INT-01.
- Unified retry migration -> F-REL-01 (cross-referenced only, V05).
- Framework tool registry contract itself -> F-EXT-01 (consumed as dependency).

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md` (F-INT-01
  card), `zcode-ds/README.md`.
- Dependency reports read: zcode-ds `F-EXT-01` (complete), `F-CORE-01`
  (complete).
- Historical documents treated as hypotheses: `echo-agent/AUDIT_REPORT.md`
  section 2.4; root `docs/MASTER-PLAN.md` MCP claims (lines 99, 184, 188,
  532, 570, 997).

## Layering Decision

- Generic mechanism (framework, `echo_integration`): McpManager/McpClient/
  McpServer/McpToolAdapter/transports/config/schema types — correctly placed
  in the integration crate; the root facade is a pure re-export.
- EKO product policy (application): mcp.json discovery paths, Tauri/TUI
  connect commands, browser sidecar usage, `McpServerInfo`/`McpTransportConfig`
  DTO projections — correctly placed in `echo-agent-cli`.
- Adapter boundary: `McpToolAdapter` (external protocol -> framework `Tool`)
  and `McpServer` (`Tool` -> external protocol) are the framework-internal
  adapters; conversion is thin and lossless for inputs (schema passthrough),
  but lossy for outputs on the server side (P3-02).
- Duplicate search terms (both repositories): `McpManager|McpClient|
  McpServer|McpTool|McpToolAdapter|McpTransport|StdioTransport|SseTransport|
  HttpTransport`, `JsonRpcRequest|JsonRpcResponse|JsonRpcNotification|
  NotificationReceiver`, `mcp__`, `mcpServers`, `McpConfigFile`,
  `connect_mcp*|disconnect_mcp|refresh_tools|get_all_tools`, `McpServerInfo`.
  Result: single authoritative MCP implementation in `echo-integration`; the
  CLI contains no parallel client/transport/JSON-RPC implementation (V01-01);
  1 dead stub (`mcp_manager_arc`, P3-04); 1 dead notification channel
  (P2-02).

## Current Path

Verified data flow: `mcp.json` (EKO `infra.rs:1091`) -> `Agent::
load_mcp_from_file` (`capabilities.rs:1241`, mcp-gated by the attribute at
`:1206`) -> `load_mcp_config` (`:1251`) -> `connect_mcp_from_config`
(`:1149`; reconnect first removes stale tools via `disconnect_mcp`) ->
`McpManager::connect` (`mod.rs:73-97`; same-name replace) -> `McpClient::new`
(`client.rs:46-146`; stdio/http/sse transport, initialize handshake with
version negotiation, `notifications/initialized`, capability-gated
tools/resources/prompts listing with 30 s page timeout, 100-page cap) ->
per-tool `McpToolAdapter::with_server_name` (`tool_adapter.rs:35-57`,
exposed name `mcp__server__tool`) registered into the shared ToolManager via
`add_tools`. Tool calls: React loop -> `ToolManager::execute_*` -> adapter
`execute` (`tool_adapter.rs:151-204`) -> `McpClient::call_tool` -> transport
`send` (stdio: id -> stdin -> stdout reader task routes by id, 120 s timeout,
EOF drains pending with -32000 errors; http: POST with retry loop + session
id, 202 wait (dead), sync JSON parse; sse: POST to dynamic endpoint, SSE
reader routes responses/notifications, auto-reconnect x5). Errors map through
`ToolFailure::from_error` (`echo-core/src/tools/mod.rs:226-245`) using
JSON-RPC codes and annotation-derived risk. Disconnect: agent-level
`disconnect_mcp` removes registered tools by exposed name then closes the
client; `ReactAgent::drop` closes all clients via a runtime-guarded spawn
(`react/mod.rs:2624-2634`). Server side: `McpServer` (stdio `serve_stdio` +
`handle_json_rpc`) is an example-exercised public API (demo30) with
initialize/version negotiation/tools/resources/prompts/ping handlers.

## Findings

### F-INT-01-P1-01: HTTP transport's 202-async-response path is dead — compliant Streamable HTTP servers hang every call for 60 s

- Priority: P1
- Confidence: high (static chain fully verified; no dynamic run in read-only review)
- Layer: framework (`echo_integration`)
- Evidence: `echo-integration/src/mcp/transport/http.rs:69-74` inserts a
  oneshot sender into `pending`; `:161-179` waits up to 60 s on it when the
  server answers `202 Accepted`; **no code anywhere sends into these channels**
  (grep `tx.send`/`.send(` in http.rs shows only reqwest calls; the only
  senders elsewhere are `stdio.rs:109` and `sse.rs:289`). `close()`
  (`:230-238`) only clears the map. `notification_tx` is never `.send()`-ed,
  so `notification_rx()` (`:240-245`) yields nothing. The `Accept:
  application/json, text/event-stream` header (`:82`) is never honored: a
  200-with-SSE response fails `response.json()` (`:193-198`). Module doc
  claims "支持异步响应" (`http.rs:22-24`).
- Reachability: `TransportConfig::Http` is a first-class config option
  (`server_config.rs:32-37`, `config_loader.rs:140-143`) and the EKO Tauri
  MCP panel can create HTTP servers (`echo-agent-cli/src/tauri/commands/
  mcp.rs:229-232`) — any server that processes asynchronously (202) or
  streams the response (200+SSE) fails every request after exactly 60 s
  (ProtocolError "等待 HTTP 异步响应超时").
- Expected invariant: the transport delivers the response for every
  server-valid response style the header advertises.
- Observed behavior: 202 -> guaranteed 60 s timeout; 200+SSE -> JSON parse
  error. Only synchronous 200 JSON responses work.
- Impact: major capability failure of the HTTP transport against a class of
  spec-compliant servers; silent until the 60 s timeout; no test exists
  (V04-02), which is why the dead path survived.
- Root cause: the async-response routing was scaffolded (pending map +
  broadcast channel) but the receive side (SSE GET stream task that parses
  `message` events and fires the oneshots, per the Streamable HTTP spec) was
  never implemented.
- Direction: implement the SSE GET receive stream for HTTP (route `message`
  events to `pending`, notifications to `notification_tx`) or explicitly
  reject 202/SSE responses with a clear error instead of a 60 s hang; add a
  fixture with a fake server answering 202 and one answering 200+SSE.
- Regression validation: transport test with a `tokio::test`-hosted fake
  endpoint: 202 then SSE `message` event -> send() returns the response;
  assert non-202 paths unchanged.
- Validation reports: [V02-01](../validations/F-INT-01/V02-01.md),
  [V03-01](../validations/F-INT-01/V03-01.md), [V04-02](../validations/F-INT-01/V04-02.md)

### F-INT-01-P1-02: HTTP transport retries non-idempotent `tools/call` on ambiguous transport failures — duplicate side effects possible, bypassing the framework retry contract

- Priority: P1
- Confidence: medium (retry behavior is a code fact; trigger requires an
  ambiguous network failure; no dynamic run)
- Layer: framework (`echo_integration`) — adapter-boundary interaction
- Evidence: `echo-integration/src/mcp/transport/http.rs:96-147` retries
  `req.send()` up to 3 times when `is_retryable_error` (`:251-270`: timeout,
  connect, 502/503/504, and message checks including "connection reset",
  "broken pipe", "eof", "connection refused"); the framework's retry gate
  `ToolFailure::allows_automatic_retry` (`echo-core/src/tools/mod.rs:154-162`)
  only retries when side effects are proven safe, and `from_error`
  (`:165-260`) converts failures into `PartialSideEffect` when the tool is
  not ReadOnly (`tool_adapter.rs:157`) — but the transport retry happens
  before the adapter ever classifies the failure, so the gate is bypassed.
- Reachability: every HTTP `tools/call` (and initialize/listing) when the
  first POST is ambiguous; MCP tools without annotations default to
  `ToolRiskLevel::Standard` (`tool_adapter.rs:148`), i.e. side effects are
  possible from the framework's perspective, yet the transport re-sends.
- Expected invariant: automatic replay only when the request is known
  side-effect-free or idempotent (AGENTS.md "有限重试"; MASTER-PLAN line 184).
- Observed behavior: a POST that reached the server and was executed but whose
  response was lost (reset/broken pipe/EOF mid-flight, or a 5xx from a proxy
  that already forwarded) is re-sent, executing a destructive tool twice.
- Impact: duplicate mutations/charges on external systems reachable via HTTP
  MCP servers; invisible at the framework layer (each attempt looks like one
  call).
- Root cause: the transport owns a retry policy instead of delegating
  retryability to the tool contract; hand-rolled backoff duplicates
  `echo_core::retry::RetryPolicy` (same class as F-REL-01-P2-01 /
  F-EXT-01-P3-04).
- Direction: remove transport-level retries for `tools/call` (keep them only
  for connection-establishing/idempotent reads, or make retry conditional on
  `Tool::risk_level()`/`annotations.idempotent_hint`); surface the failure as
  `ToolFailure` so the manager-level contract gate decides; align backoff with
  `RetryPolicy` per F-REL-01.
- Regression validation: fixture where the first POST "succeeds server-side
  but resets the connection" and the tool is not read-only -> assert exactly
  one server-side execution; same fixture with `read_only_hint` -> assert
  retry happens.
- Validation reports: [V03-01](../validations/F-INT-01/V03-01.md),
  [V05-01](../validations/F-INT-01/V05-01.md)

### F-INT-01-P2-01: SSE transport treats UTF-8 chunk splits as protocol errors and recognizes only `\n\n` separators — spurious disconnect/reconnect and lost in-flight responses

- Priority: P2
- Confidence: high (code fact); trigger probability medium (needs multibyte
  content split across network chunks, or CRLF server)
- Layer: framework (`echo_integration`)
- Evidence: `sse.rs:210-215` decodes each `bytes_stream()` chunk with
  `std::str::from_utf8` and returns `ProtocolError` on failure, tearing down
  the whole loop (reconnect with backoff, max 5, `:65-127`); a multi-byte
  char split across chunk boundaries (normal for large responses containing
  Chinese/emoji) triggers this. Event framing splits only on `\n\n`
  (`:219-221`) — CRLF (`\r\n\r\n`) servers are never parsed, so every
  response times out (30 s, `:371-383`). `send` failure paths leak pending
  entries: POST error (`:353-358`) and timeout (`:371-383`) do not remove
  them; `close()` (`:415-421`) neither fails in-flight requests nor clears
  `pending`. The reconnect loop reconnects the SSE stream but never
  re-initializes the MCP session (message endpoint reset at `:158-165`,
  session state lost; in-flight requests are not retransmitted).
- Reachability: `TransportConfig::Sse` via `McpServerConfig::sse*` and the
  EKO Tauri MCP panel (`tauri/commands/mcp.rs:235-237`) for legacy
  2024-11-05 servers.
- Expected invariant: transport decodes byte streams incrementally (accumulate
  bytes, then decode) and splits events on any valid SSE line ending;
  every failure path cleans up `pending`; a lost in-flight request is either
  failed with a typed error or retransmitted.
- Observed behavior: spurious reconnects, 30 s hangs, permanently leaked
  pending entries (memory growth over long sessions with flaky servers), and
  lost responses after reconnect.
- Impact: legacy-SSE MCP servers with multibyte content fail intermittently;
  requests silently lost on reconnect.
- Root cause: byte-to-string conversion per chunk instead of incremental
  buffered decoding; SSE framing implemented for the `\n`-only case; no
  lifecycle hook for pending entries on failure/reconnect.
- Direction: accumulate raw bytes and decode at event boundaries; accept
  `\r\n`/`\r` line endings; fail or retransmit in-flight requests on
  reconnect; clear `pending` in `close()` with typed errors; add fixtures
  with a chunk-split multibyte payload and a CRLF SSE stream.
- Regression validation: unit fixture feeding `data: {"result":"中…"}` split
  mid-character across two chunks -> one decoded event, no reconnect; CRLF
  fixture -> parsed; POST-failure fixture -> pending map empty.
- Validation reports: [V03-01](../validations/F-INT-01/V03-01.md),
  [V04-02](../validations/F-INT-01/V04-02.md)

### F-INT-01-P2-02: Client handshake declares roots/sampling/elicitation capabilities with no handlers — server-initiated requests are silently dropped and the notification channel is dead

- Priority: P2
- Confidence: high
- Layer: framework (`echo_integration`)
- Evidence: `client.rs:149-158` (`build_client_capabilities` declares
  `roots.listChanged`, `sampling`, `elicitation`); no transport implements
  server-request handling: stdio misparses a server request (id+method) as a
  `JsonRpcResponse` (all fields optional) and drops it (`stdio.rs:104-122`);
  SSE drops it as "unknown format" (`sse.rs:305-307`); HTTP has no inbound
  path. `McpTransport::notification_rx` (`transport/mod.rs:27`) is never
  called by `McpClient` or anything else (V02-01).
- Reachability: any server that honors the declared capabilities sends
  `roots/list`, `sampling/createMessage`, or `elicitation` requests — all
  silently dropped, so the server hangs waiting; `tools/list_changed` /
  `resources/list_changed` notifications are also never consumed, and
  `refresh_tools` (`client.rs:219-227`) has no caller.
- Expected invariant: a client must not declare capabilities it cannot
  handle (MCP spec); declared notification support must have a consumer.
- Observed behavior: negotiation promises features that do not exist; server
  requests vanish; the notification receiver trait is dead scaffolding.
- Impact: protocol-level breakage for servers that use sampling/roots;
  misleading handshake; refresh path unreachable.
- Root cause: capability declaration and receive plumbing were written
  independently; the receive side was never wired into `McpClient`.
- Direction: either remove the unhandled capability declarations (keep the
  handshake honest) or implement the handlers and consume `notification_rx`
  (route `tools/list_changed` to `refresh_tools`); add a fixture where a fake
  server sends `sampling/createMessage` and assert a typed error or a real
  response.
- Regression validation: client-handshake fixture asserting the declared
  capabilities set matches the handled set; notification fixture asserting
  `list_changed` triggers `refresh_tools`.
- Validation reports: [V02-01](../validations/F-INT-01/V02-01.md),
  [V03-01](../validations/F-INT-01/V03-01.md)

### F-INT-01-P2-03: MCP cancellation contract is unimplemented end-to-end — cancelled runs cannot abort server-side tool execution and the only `notifications/cancelled` emitted is malformed

- Priority: P2
- Confidence: high (behavior); medium (impact frequency)
- Layer: framework (`echo_integration`)
- Evidence: `tool_adapter.rs:151-204` implements only `Tool::execute`
  (no `execute_with_context` override), so `ToolContext.cancel` is never
  observed; the client never sends `notifications/cancelled` for a real
  cancellation (grep: the only emission is `http.rs:232-234` inside
  `close()`, which sends it **without the spec-required `requestId`
  parameter**); the server's handler for it is a log-only no-op
  (`server.rs:306-308`), and `serve_stdio` (`server.rs:192-255`) is a
  sequential loop — a long-running `tools/call` blocks ping and every other
  request.
- Reachability: any cancelled/timeout tool invocation on an MCP tool; the
  stdio server keeps executing the tool to completion after the client gave
  up (client-side 120 s stdio timeout / dropped future only discards the
  response).
- Expected invariant: cancellation is either propagated (spec
  `notifications/cancelled` with `requestId`) or honestly documented as
  unsupported; cancelled calls must not silently continue mutating external
  systems without notice.
- Observed behavior: a cancelled MCP call continues executing server-side and
  its side effects persist; no abort channel exists in either direction.
- Impact: post-cancellation side effects on external systems (the MCP spec
  provides the mechanism; the framework just does not use it). Consistent
  with the framework's cooperative-cancel model (F-EXT-01 V01 note), so
  severity is P2 not P1.
- Root cause: cancellation was never plumbed into the MCP layer (adapter,
  client, or server), and the one `notifications/cancelled` site was written
  without the mandatory `requestId` field.
- Direction: send `notifications/cancelled {requestId}` from `McpClient` when
  the framework cancels the invocation (requires the adapter to observe
  `ToolContext.cancel`), have `McpServer` track in-flight calls and abort the
  tool future on cancel, and fix the malformed notification in `close()`;
  add a cancel fixture asserting the server aborts and no response is
  produced after cancel.
- Regression validation: fake-server fixture: cancel a long `tools/call` ->
  assert `notifications/cancelled` with `requestId` is received; server-side
  fixture: cancel notification -> tool future dropped, response suppressed.
- Validation reports: [V03-01](../validations/F-INT-01/V03-01.md),
  [V04-02](../validations/F-INT-01/V04-02.md)

### F-INT-01-P3-01: `McpManager::connect` reconnect leaves previously returned adapters bound to the dead client and is destructive on failure

- Priority: P3
- Confidence: high
- Layer: framework (`echo_integration`)
- Evidence: `mod.rs:73-97` disconnects the old same-name client before
  connecting; previously returned `McpToolAdapter`s (and tools registered on
  an agent by a direct `McpManager` user) keep the old `Arc<McpClient>`
  whose transport is closed — they fail forever after reconnect; if the new
  connection fails, the old connection is already gone (destructive-on-
  failure). The agent-level path is safe (`capabilities.rs:1153-1162`
  disconnects and re-registers), so only direct `McpManager` consumers are
  affected (framework API, e.g. demo06).
- Expected invariant: reconnect either preserves or explicitly invalidates
  previously handed-out tool bindings.
- Observed behavior: stale adapters fail with transport errors after
  reconnect; no invalidation signal, no documentation.
- Impact: confusing failures for framework consumers that cache
  `McpManager::connect` results across reconnects.
- Root cause: `connect` returns tool handles whose client ownership is not
  versioned; the manager has no invalidation/registry-version concept.
- Direction: document the contract (re-request tools after reconnect) or make
  `connect` return a connection-epoch so consumers can detect staleness;
  consider not destroying the old connection until the new one succeeds.
- Regression validation: unit test: connect -> disconnect-on-reconnect ->
  old adapter returns a typed "stale connection" error while the new adapter
  works.
- Validation reports: [V03-01](../validations/F-INT-01/V03-01.md)

### F-INT-01-P3-02: `McpServer` adapter is lossy and placeholder — structured tool data dropped, resources/read and prompts/get return fake content, duplicate tool names silently overwrite

- Priority: P3
- Confidence: high
- Layer: framework (`echo_integration`)
- Evidence: `server.rs:446-468` maps `ToolResult` to text-only content,
  dropping `data`/structured kind (the client direction preserves it via
  `structured_content`, `tool_adapter.rs:172-190` — asymmetry); `:512-517`
  `resources/read` returns the placeholder `format!("Resource: {}", name)`
  instead of resource content; `:561-564` `prompts/get` always returns
  `messages: vec![]` while the server advertises the prompts capability
  (`:367-373`); `McpServerBuilder::build` (`:653-669`) inserts duplicate tool
  names into `tool_map` last-wins while `tool_list` advertises both — the
  same silent-overwrite class as F-EXT-01-P1-02.
- Reachability: any MCP client connecting to an `McpServer` (example-exercised
  today; the served tools' structured output, registered resources, and
  prompts are contractually advertised).
- Expected invariant: adapters convert losslessly or do not advertise the
  capability.
- Observed behavior: structured tool output lost; resources/read and
  prompts/get return fabricated/empty content; duplicate tool names advertise
  two tools but execute one.
- Impact: consumers of the server side get degraded or false data; capability
  advertising is dishonest.
- Root cause: the server-side mapping was written before the structured
  result kinds existed (F-CORE-01/F-EXT-01 contracts) and was never upgraded;
  builder does not detect name collisions.
- Direction: map `ToolResultKind::Json` to `structured_content`, advertise
  resources/prompts only with real content (or implement storage), and make
  the builder reject or deduplicate tool names with a warning.
- Regression validation: server fixture calling a Json-kind tool ->
  `structured_content` populated; duplicate-name build -> warning or error;
  prompts/get on a registered prompt returns its messages.
- Validation reports: [V03-01](../validations/F-INT-01/V03-01.md)

### F-INT-01-P3-03: MCP doc drift — `load_mcp_from_file` claims YAML support that does not exist, and the module doc references a nonexistent `connect_mcp` method

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `src/agent/react/capabilities.rs:1210,1216` document "supports
  `.json` or `.yaml` format" and "supports JSON or YAML format", but
  `McpConfigFile::from_file` (`config_loader.rs:181-191`) parses with
  `serde_json` only; `capabilities.rs:5` module doc lists "MCP connections
  (`connect_mcp` / `load_mcp_from_file`)" — no `connect_mcp` method exists
  (only `connect_mcp_from_config`/`connect_mcp_from_json`).
- Reachability: documentation consumers; a YAML-config user gets a parse
  error.
- Expected invariant: docs describe the actual API.
- Observed behavior: two stale doc claims.
- Impact: misleading API surface; low.
- Root cause: docs written before the API was finalized (the JSON-only loader
  and the renamed connect methods).
- Direction: fix the doc text (JSON only; correct method names).
- Regression validation: none needed (doc-only).
- Validation reports: [V01-01](../validations/F-INT-01/V01-01.md)

### F-INT-01-P3-04: Dead `mcp_manager_arc` stub and unguarded `tokio::spawn` in `StdioTransport::drop`

- Priority: P3
- Confidence: high (dead stub); medium (drop panic requires runtime-less teardown)
- Layer: framework
- Evidence: `src/agent/react/subsystems/tool_exec.rs:74-77`
  (`mcp_manager_arc` returns `None` with `#[allow(dead_code)]` and a comment
  claiming the manager "is not Arc-wrapped" — dead scaffolding);
  `echo-integration/src/mcp/transport/stdio.rs:270-282` (`Drop` calls
  `tokio::spawn` unconditionally — panics when no Tokio runtime is current,
  e.g. an `Arc<McpClient>` dropped after runtime shutdown, and a panic in
  `Drop` aborts the process; contrast `ReactAgent::drop`,
  `react/mod.rs:2624-2634`, which guards with `Handle::try_current`).
- Reachability: any framework consumer dropping a stdio MCP client outside a
  runtime context.
- Expected invariant: no panics in `Drop`; no dead public helpers with
  misleading returns.
- Observed behavior: possible abort on runtime-less teardown; the stub invites
  callers to rely on a nonexistent accessor.
- Impact: low-frequency crash risk; maintenance hazard.
- Root cause: cleanup written before a runtime guard existed; the stub is a
  leftover from an earlier Arc-based manager design.
- Direction: mirror the `Handle::try_current` guard in `StdioTransport::drop`
  (or skip spawn and let the child be reaped by the process), and delete the
  dead `mcp_manager_arc` stub.
- Regression validation: unit test dropping a `StdioTransport` without a
  runtime context asserts no panic; `cargo check` after deleting the stub.
- Validation reports: [V01-01](../validations/F-INT-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (both repos) | yes | passed | [V01-01](../validations/F-INT-01/V01-01.md) |
| V02 | Registration and runtime reachability trace | yes | passed | [V02-01](../validations/F-INT-01/V02-01.md) |
| V03 | Transport/schema/cancellation/reconnect invariant and edge-case inspection | yes | passed | [V03-01](../validations/F-INT-01/V03-01.md) |
| V04 | `cargo check -p echo_agent --no-default-features --locked` (fresh target) | yes | passed (exit 0) | [V04-01](../validations/F-INT-01/V04-01.md) |
| V04 | `cargo test -p echo_integration --features mcp --lib --locked [mcp]` | yes | passed (exit 0; 18 mcp / 80 total) | [V04-02](../validations/F-INT-01/V04-02.md) |
| V04 | `cargo check -p echo_agent --no-default-features --features mcp --locked` | yes | passed (exit 0) | [V04-03](../validations/F-INT-01/V04-03.md) |
| V05 | Historical-document drift check (AUDIT 2.4, MASTER-PLAN) | yes | passed | [V05-01](../validations/F-INT-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AUDIT_REPORT 2.4: stdio drops pending requests on stdout close (no per-request failure info) | fixed | `stdio.rs:124-143` drains and sends typed -32000 errors with the request id; audit still cites old line numbers |
| MASTER-PLAN Phase B: "网络瞬断、MCP 重连…均有专项测试" (line 188) | stale | no disconnect/reconnect/cancellation fixtures exist anywhere (V04-02) |
| MASTER-PLAN line 532: preserve JSON-RPC code; annotations decide side-effect risk | current | `client.rs:240-243`, `echo-core/src/tools/mod.rs:226-245`, `tool_adapter.rs:139-149,172-181` |
| MASTER-PLAN lines 184/532: "MCP 根据 tool contract 判断能否重试" | partially regressed | contract gate works at manager level; HTTP transport retries before classification (P1-02); SSE reconnect loop is a 4th hand-rolled retry (F-REL-01 cross-ref) |
| MASTER-PLAN line 997: "MCP 暂不 disconnect(无 per-server API)" | fixed | per-server `disconnect_mcp` exists (`capabilities.rs:1315-1331`); plugin unload uses it (`plugin_runtime.rs:1173`) |

## Coverage And Uncertainty

- Static-only review (read-only): no dynamic client<->server round-trip was
  executed; P1-01/P1-02 rely on fully verified code chains (no sender into
  the 202 channel; retry loop semantics), not runtime reproduction.
- The repo contains no client-handshake, transport round-trip, disconnect,
  reconnect, or cancellation tests (V04-02 inventory) — the task-card
  required fixtures are absent and become the primary regression-validation
  targets for P1/P2 findings.
- EKO browser sidecar uses stdio transport (`browser/sidecar.rs:35-66`), so
  HTTP/SSE defects are latent for that path today; the Tauri MCP panel can
  configure HTTP/SSE servers, making P1-01/P2-01 reachable in EKO through
  user-configured remote servers (A-INT-01 will recheck the application
  surface).
- `McpServer` serving path is example-exercised only (demo30); its findings
  (P3-02) have no production consumer today but are framework-contract
  defects on a public API.
- `McpManager::get_all_tools` has no callers; retained as public framework
  API per AGENTS.md (recorded, not a finding).
- Schema handling is lossless passthrough (V03-01); `McpTool.annotations`
  mapping aligns with the F-EXT-01 single-authority direction.

## Handoff

- Downstream tasks may rely on: single-authority map (V01), reachability
  chain (V02), transport invariant inventory (V03), green feature gates
  (V04-01/02/03), historical classification (V05).
- `A-INT-01`: P1-01/P2-01 become user-visible when EKO users configure
  HTTP/SSE MCP servers (60 s hangs, spurious reconnects); P2-03 matters for
  cancelled EKO runs; the CLI gate check (un-gated connect) passed.
- `F-REL-01`: P1-02 and the SSE reconnect loop extend the retry-unification
  target to `http.rs:96-147` and `sse.rs:65-127`.
- `Q-FLT-01`/`Q-TST-01`: the missing malformed/disconnected-server fixtures
  (V04-02) are prime fault-injection candidates once transports are fixed.
- `F-API-01`/`Q-DOC-01`: P3-03 doc drift; P3-04 dead stub deletion.
- This report becomes stale if the transport layer, `McpToolAdapter`,
  capability declarations, or `McpServer` mapping change.
