# F-INT-02: LSP, channels, and A2A integrations

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Do LSP, IM channel, and A2A adapters isolate external protocols while
preserving typed internal lifecycle and cleanup?

## Scope

- LSP: `echo-core/src/lsp/` (client.rs, types.rs, mod.rs — full reads),
  `echo-integration/src/lsp/` (client.rs 534 lines, jsonrpc.rs, manager.rs,
  config.rs — full reads), `echo-agent/src/tools/lsp.rs` (tool
  implementations), EKO wiring `echo-agent-cli/echo-agent-app-core/src/
  runtime.rs:499-597` and `plugin_runtime.rs` (LSP manager reload paths).
- Channels: `echo-integration/src/channels/` (types.rs, manager.rs,
  session.rs, mod.rs, channels/mod.rs, channels/qq/{channel,gateway,api}.rs,
  channels/feishu/{channel,api,long_poll,webhook,proto}.rs — full reads),
  root facade `echo-agent/src/channels.rs`, EKO consumer
  `echo-agent-cli/src/cli/channels.rs` and `src/cli/modes.rs:110-255`.
- A2A: `echo-agent/src/a2a/` (types.rs 801 lines, server.rs 735 lines,
  client.rs, serve.rs, auth.rs — full reads), root exports `src/lib.rs`.
- Error types: `echo-core/src/error.rs` ChannelError.

## Out Of Scope

- ReAct loop internals (F-RCT-02/03), generic tool execution/timeouts
  (F-EXT-01, F-RCT-04), MCP (F-INT-01), EKO app-side channel/HITL policy
  (A-SRF-04, A-HITL-01, A-INT-01), feature-topology matrix (F-FEAT-01),
  frontend projections (A-FE-*). The React tool timeout
  (`react_loop.rs:316`) was consulted only to calibrate LSP hang impact.

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`
  (F-INT-02 card), `zcode-ds/README.md`.
- Dependency report: zcode-ds `F-CORE-01` (event identity/envelope,
  error taxonomy, ChannelError placement).
- Historical docs treated as hypotheses: `echo-agent/AUDIT_REPORT.md`,
  `docs/MASTER-PLAN.md`, `docs/PROJECT-ANALYSIS.md` (V05).

## Layering Decision

- Generic mechanism: LSP client trait/types, `LspManager`,
  `StdioLspClient`, `ChannelManager`/`MessageHandler`/
  `InboundMessage`/`OutboundMessage`/`SessionHandler`, `A2AServer`/
  `A2AClient`/`serve*`/`JwtConfig` — all framework capabilities correctly
  placed in echo-agent (core trait + integration implementation + root
  facade), independently of EKO usage.
- EKO product policy: none in these families; EKO consumes framework
  types (`AppChannelMessageHandler`, LSP tools, per-sender pool keys).
- Adapter boundary: `AgentChannelHandler` (root), `QqMessageHandler`,
  `FeishuMessageHandler`, `open_file_for_lsp`, `dispatch_stream_to_send_tx`
  — thin conversions; A2A `A2ATask`/`TaskState` are wire-protocol artifacts
  (external fixed protocol exception), not a second task authority (A2A
  dispatches directly to `dyn Agent`; see V03-12).
- Duplicate search terms: `MessageHandler`, `InboundMessage`,
  `OutboundMessage`, `ChannelManager`, `SessionHandler`, `LspClient`,
  `StdioLspClient`, `LspManager`, `A2AServer`, `A2AClient`, `worker`,
  `LspError::Timeout`, `max_restarts`, `restart_count`, `restart_server`,
  `cleanup_completed_tasks`, `uri_to_path`, `register_lsp_tools` — each has
  exactly one authoritative definition; no parallel implementations across
  the two repositories (V01-01, V02-01..03).

## Current Path

- LSP: EKO boot (`runtime.rs:274`) → `LspConfig::discover` + global/project
  `.lsp.yaml` merge → `LspManager.start_server` (15s init timeout) → five
  tools (`lsp_diagnostics` etc.) registered on the primary agent →
  `StdioLspClient` JSON-RPC over stdio (framed Content-Length) with a
  diagnostics cache fed by `textDocument/publishDiagnostics`. Plugin
  reload replaces the manager atomically and awaits `shutdown_all`
  (plugin_runtime.rs:583-975).
- Channels: EKO `run_channels_mode` (modes.rs:122-247) registers
  QQ/Feishu from `echo-agent.yaml`, `start_all` with one `SessionHandler`
  per channel wrapping `AppChannelMessageHandler` (per-sender pool key),
  waits for shutdown signal, then `stop_all`. QQ = WS gateway + detached
  send task; Feishu = long-poll PBBP2 protobuf client (fragment
  reassembly, dedup, backoff reconnect) or webhook server (challenge,
  token + HMAC-SHA256 verification, dedup).
- A2A: framework-only; `A2AServer` (tasks map + cancel tokens) executing a
  wrapped `dyn Agent` for `tasks/send`, SSE streaming for
  `tasks/sendSubscribe`, `tasks/get`, `tasks/cancel`; `serve_inner` binds
  axum with optional JWT (`JwtConfig`) and a non-loopback no-auth warning;
  `A2AClient` (reqwest) with discover/send/get/cancel + SSE parsing. No
  EKO consumer (V02-03).

## Findings

### F-INT-02-P1-01: LSP JSON-RPC requests have no timeout and no cancel cleanup; a hung server blocks shutdown and leaks the pending map

- Priority: P1
- Confidence: high
- Layer: framework (integration)
- Evidence: `echo-agent/echo-integration/src/lsp/client.rs:225-227`
  (`rx.await` with no timeout in `send_request`), `:214-218` (pending
  entry inserted before writer send), `:311-329` (`shutdown` awaits
  `send_request("shutdown")` unbounded), `:125-199` (read_loop);
  `echo-agent/echo-core/src/lsp/client.rs:15` (`LspError::Timeout`
  declared, never constructed).
- Reachability: EKO registers LSP tools at boot
  (`echo-agent-cli/echo-agent-app-core/src/runtime.rs:274,499-597`); tool
  calls go through `send_request` (echo-agent/src/tools/lsp.rs:54-105);
  plugin reload and removals await `shutdown_all()`
  (plugin_runtime.rs:583-721,975). React bounds the agent turn via
  `tokio::time::timeout` (echo-agent/src/agent/react/run/react_loop.rs:316),
  but a timed-out tool call leaves its pending entry behind.
- Expected invariant: every request completes or fails in bounded time;
  cancellation releases request bookkeeping; `stop`/`restart` terminate.
- Observed behavior: a server that is alive but not answering blocks
  `shutdown` (and therefore `LspManager::stop_server`/`restart_server`/
  `shutdown_all`) forever; cancelled requests leak `pending` entries that
  are reclaimed only on server response or reader exit — unbounded growth
  under a hung server plus repeated calls.
- Impact: EKO plugin reload/app shutdown hangs with a hung language
  server; long sessions with a stalled server grow the pending map without
  bound; the `Timeout` error variant is dead contract.
- Root cause: convenience design without a deadline on `send_request`; the
  oneshot pair is inserted into shared state before the send that can fail,
  and cancellation has no cleanup path.
- Direction: wrap `rx.await` in `tokio::time::timeout` (construct
  `LspError::Timeout`); on timeout/cancel remove the pending entry; bound
  `shutdown` with a deadline before killing the child; add tests for
  request timeout, cancel cleanup, and hung-server shutdown.
- Regression validation: unit tests with a stub server that never answers
  — assert `LspError::Timeout` at the configured deadline, assert `pending`
  len returns to 0 after a cancelled call, assert `shutdown()` returns
  within a deadline and the child is killed.
- Validation reports: [V02-01](../validations/F-INT-02/V02-01.md),
  [V03-01](../validations/F-INT-02/V03-01.md),
  [V03-02](../validations/F-INT-02/V03-02.md),
  [V03-03](../validations/F-INT-02/V03-03.md)

### F-INT-02-P1-02: QQ channel send task busy-loops on a CPU core after `stop()`

- Priority: P1
- Confidence: high
- Layer: framework (integration)
- Evidence: `echo-agent/echo-integration/src/channels/channels/qq/channel.rs:108-132`
  (`loop { if let Some(msg) = send_rx.recv().await { ... } }` with the
  JoinHandle discarded via `let _send_task = tokio::spawn`), `:198-208`
  (`stop()` aborts only `gateway_handle`, drops `self.send_tx`), `:100-103`
  (wrapper holds the last sender clone).
- Reachability: `ChannelManager::start_all`/`stop_all`
  (manager.rs:53-107); EKO channel mode calls `stop_all` on shutdown
  (`echo-agent-cli/src/cli/modes.rs:244-247`) and exits soon after, but
  any long-running framework consumer that stops/restarts channels hits
  the spin for the process lifetime.
- Expected invariant: after `stop()` no channel-owned task remains
  running.
- Observed behavior: after `stop()` aborts the gateway task, the wrapper
  (last `send_tx` holder) is dropped, the channel closes, `recv()` returns
  `None` immediately, and the `loop`/`if let` spins forever (100% of one
  core) until process exit. Feishu's equivalent task uses
  `while let` (feishu/channel.rs:218) and terminates cleanly.
- Impact: one permanently spinning tokio task per stopped QQ channel in
  long-lived consumers; EKO's immediate process exit masks it today.
- Root cause: `loop` + `if let` instead of `while let` on the receiver,
  plus a discarded JoinHandle that cannot be aborted.
- Direction: change to `while let Some(msg) = send_rx.recv().await`,
  store the JoinHandle and abort it in `stop()`, or drive termination via
  the `running` flag; add a stop-lifecycle test for QQ.
- Regression validation: async test that starts a QQ channel with a stub
  handler, calls `stop()`, and asserts the send task terminates (e.g.,
  via a completion signal) and the gateway handle is finished.
- Validation reports: [V02-02](../validations/F-INT-02/V02-02.md),
  [V03-05](../validations/F-INT-02/V03-05.md)

### F-INT-02-P1-03: A2A `tasks/cancel` does not cancel execution for sync tasks and regresses terminal state

- Priority: P1
- Confidence: high
- Layer: framework (adapter boundary)
- Evidence: `echo-agent/src/a2a/server.rs:404-408` (cancel token stored,
  never used by `handle_task_send`), `:414` (`agent.execute` without
  token), `:439-442` (completed record overwrites Canceled), `:527-584`
  (`handle_task_cancel`), `:225-235` (streaming cancel checks the token
  between events); `echo-core/src/agent/mod.rs:560-568`
  (`execute_stream_with_cancel` exists, unused here).
- Reachability: any client sending `tasks/send` with an explicit `id`,
  then `tasks/cancel` while the run is in flight; `tasks/get` afterwards
  shows the regression. Streaming cancel works (one-event latency) by
  dropping the stream future, but the underlying run is aborted implicitly
  rather than through the token.
- Expected invariant: A2A state machine — terminal states are final;
  cancel stops execution (A2A spec `tasks/cancel` semantics; doc at
  mod.rs:7-18).
- Observed behavior: `tasks/cancel` marks the record Canceled, execution
  continues to completion, and the completed record overwrites it —
  Canceled → Completed is observable via `tasks/get`, a terminal-state
  regression; the cancel token is never consulted on the sync path.
- Impact: remote callers cannot stop a running task; protocol-state
  invariant broken; cancel is cosmetic on the primary sync path.
- Root cause: the token is created and stored but never connected to the
  agent run (framework offers `execute_stream_with_cancel`, the server
  uses plain `execute`/`execute_stream`).
- Direction: use a cancel-aware execution variant on the sync path
  (cancel the in-flight run and keep the terminal Canceled state), or
  document token as stream-only; add a fixture asserting a sync task
  cancelled mid-run terminates Canceled and never transitions again.
- Regression validation: async test driving `handle_task_send` with a
  slow stub agent + `handle_task_cancel` mid-run; assert final stored
  state is Canceled and no further transition occurs.
- Validation reports: [V03-09](../validations/F-INT-02/V03-09.md),
  [V03-11](../validations/F-INT-02/V03-11.md)

### F-INT-02-P2-01: LSP restart contract (`max_restarts`/`restart_count`/`restart_server`) is inert

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/lsp/types.rs:142-149` (`max_restarts`
  default 3), `echo-integration/src/lsp/manager.rs:105-108`
  (`restart_server` has no production caller), `client.rs:40,60,528`
  (`restart_count` never incremented), `config.rs:59` (value set, never
  read), `examples/demo55_lsp_tools.rs:238` (documents the field).
- Reachability: `LspServerStatus.restart_count` always 0; no code path
  enforces a restart bound; dead-lettered by design.
- Expected invariant: documented "Maximum restart attempts before giving
  up" is enforced.
- Observed behavior: nothing ever restarts a server automatically, counts
  restarts, or consults the bound; the public field and status report are
  misleading.
- Impact: consumers cannot rely on the documented recovery policy; status
  reporting is fake-faithful.
- Root cause: restart accounting was designed but never wired.
- Direction: either implement auto-restart with `max_restarts` in
  `LspManager` (with `restart_count` increment in the client) or remove
  the fields and document restart as caller-managed; delete the dead
  example documentation accordingly.
- Regression validation: manager test with a failing server executable
  asserting restart stops after `max_restarts` attempts and
  `restart_count` is reported.
- Validation reports: [V02-01](../validations/F-INT-02/V02-01.md),
  [V03-01](../validations/F-INT-02/V03-01.md),
  [V03-03](../validations/F-INT-02/V03-03.md)

### F-INT-02-P2-02: LSP `Content-Length`-driven allocation is unbounded

- Priority: P2
- Confidence: high
- Layer: framework (integration)
- Evidence: `echo-agent/echo-integration/src/lsp/client.rs:159-164`
  (`let mut body = vec![0u8; len]` with `len` from the server header,
  no cap).
- Reachability: any language server (including a buggy or hostile one)
  publishing a large `Content-Length` aborts the host process on
  allocation failure.
- Expected invariant: framed message size is bounded before allocation.
- Observed behavior: unbounded `vec![0u8; len]`.
- Impact: process abort (OOM) from local child-process output; also
  `read_exact` blocks indefinitely when the advertised length exceeds the
  body actually sent (no timeout, see P1-01).
- Root cause: missing size cap in the framing parser.
- Direction: cap the frame size (e.g., 16 MiB) and return
  `LspError::CommunicationError` on oversized headers; add a fixture with
  a bogus header.
- Regression validation: unit test feeding a fabricated
  `Content-Length: 999999999` header to `read_loop` and asserting an
  error (not an allocation).
- Validation reports: [V03-02](../validations/F-INT-02/V03-02.md)

### F-INT-02-P2-03: `lsp_diagnostics` tool can report "clean" on stale/absent diagnostics

- Priority: P2
- Confidence: medium
- Layer: framework (tools)
- Evidence: `echo-agent/src/tools/lsp.rs:74` (fixed
  `tokio::time::sleep(150ms)` after `did_open`), `:76-81` (empty cache →
  "No diagnostics found. File is clean."), `:458-476`
  (`open_file_for_lsp` re-`did_open`s the file with version 1 on every
  call), `echo-integration/src/lsp/client.rs:194-195` (cache insert on
  `publishDiagnostics`), `:513-521` (`did_close` does not purge the
  cache).
- Reachability: agent-invoked `lsp_diagnostics` on a slow server or large
  file returns a false "clean" verdict; diagnostics for closed files stay
  cached (stale results); repeated `did_change` reuse version 2
  (`client.rs:495`), violating LSP versioning.
- Expected invariant: tool result reflects the server's actual
  diagnostics (or an explicit timeout/error); caches are invalidated on
  close.
- Observed behavior: fixed-sleep race produces empty results that are
  presented as authoritative; cache never invalidated on `did_close`.
- Impact: misleading code-quality signal to the agent; stale results
  after file close; versioning protocol drift.
- Root cause: polling-by-sleep instead of waiting for the
  `publishDiagnostics` notification; no cache lifecycle on close.
- Direction: wait for a diagnostics notification (bounded) instead of a
  fixed sleep, or return an explicit "diagnostics not ready" result;
  clear the per-URI cache on `did_close`; use a monotonic version counter.
- Regression validation: stub-server fixture that delays
  `publishDiagnostics` past 150ms and asserts the tool either waits or
  reports not-ready; a second fixture asserting cache eviction on close.
- Validation reports: [V03-01](../validations/F-INT-02/V03-01.md),
  [V03-03](../validations/F-INT-02/V03-03.md)

### F-INT-02-P2-04: `SessionHandler` never evicts idle sessions

- Priority: P2
- Confidence: high
- Layer: framework (integration)
- Evidence: `echo-agent/echo-integration/src/channels/session.rs:260-271`
  (`get_or_create` inserts per `(channel_id, sender_id)`), `:319-332`
  (timeout replaces `guard.handler` with a fresh factory instance but
  keeps the DashMap entry), `:298-312` (entry removed only on command
  reset), `:255-256` (`active_sessions` is only a counter).
- Reachability: any long-running channel consumer (public QQ/Feishu bot
  with many unique senders) accumulates one entry — holding a full
  `MessageHandler` (an Agent with memory, per EKO's pool-backed factory)
  — per unique sender for the process lifetime.
- Expected invariant: idle sessions are eventually reclaimed (the
  documented timeout is the eviction policy).
- Observed behavior: timeout replaces the handler object but never
  removes the key; entries grow monotonically with distinct senders.
- Impact: unbounded memory growth in long-lived bot deployments.
- Root cause: timeout semantics implemented as replace, not evict;
  missing TTL sweep.
- Direction: remove the entry on timeout (recreating lazily on next
  message) or add a periodic sweep; keep `on_session_end` firing;
  regression-test entry count after timeout.
- Regression validation: session test asserting
  `active_sessions()`/map size drops after timeout and a subsequent
  message creates a fresh handler.
- Validation reports: [V03-05](../validations/F-INT-02/V03-05.md)

### F-INT-02-P2-05: `ChannelManager::drop` does not stop channels despite the documented contract

- Priority: P2
- Confidence: high
- Layer: framework (integration)
- Evidence: `echo-agent/echo-integration/src/channels/manager.rs:14-15`
  (doc: "Auto-stop all channels on Drop"), `:131-141` (Drop only logs).
- Reachability: any consumer dropping a `ChannelManager` without
  `stop_all` leaves QQ gateway reconnect loops and Feishu `WsClient`
  reconnect loops running detached (they hold their own clones of
  client/token state).
- Expected invariant: dropping the manager terminates its channels.
- Observed behavior: drop is a no-op (log-only).
- Impact: orphaned network tasks and credentials-bearing token managers
  outlive their owner; combined with the QQ spin (P1-02) this is a real
  resource leak in long-running consumers.
- Root cause: doc/behavior mismatch; `stop_all` was never wired into
  `Drop` (async Drop is not possible, so a documented contract needs
  either a synchronous abort or explicit ownership documentation).
- Direction: either document drop as non-stopping (fix the doc) or
  provide a `shutdown()` that aborts all channel handles and call it from
  EKO/consumers; keep `stop_all` as the async path.
- Regression validation: test dropping a manager with a registered stub
  channel and asserting the channel's spawned task terminates (via a
  signal the stub sets on abort/drop).
- Validation reports: [V02-02](../validations/F-INT-02/V02-02.md),
  [V03-05](../validations/F-INT-02/V03-05.md)

### F-INT-02-P2-06: Feishu long-poll fragment `sum` header drives unbounded allocation

- Priority: P2
- Confidence: high
- Layer: framework (integration)
- Evidence: `echo-agent/echo-integration/src/channels/channels/feishu/long_poll.rs:381-384`
  (`sum`/`seq` from headers), `:482-484` (`vec![None; sum as usize]`),
  `:490-492` (slot write guarded).
- Reachability: a malformed or hostile frame from the Feishu long-poll
  endpoint with a huge `sum` header aborts the host process on
  allocation failure.
- Expected invariant: fragment buffers are bounded by a protocol cap.
- Observed behavior: no cap on `sum`.
- Impact: process abort from external (Feishu) input; also each fragment
  slot may hold a payload, multiplying the allocation.
- Root cause: trust of the wire header without a bound.
- Direction: cap `sum` (e.g., 1024) and reject/reconnect on overflow;
  add a malformed-frame fixture with `sum = i32::MAX`.
- Regression validation: unit test calling `combine_fragments` with a
  large `sum` and asserting bounded behavior (error, not allocation).
- Validation reports: [V03-06](../validations/F-INT-02/V03-06.md)

### F-INT-02-P2-07: A2A client has no request timeouts

- Priority: P2
- Confidence: high
- Layer: framework (adapter boundary)
- Evidence: `echo-agent/src/a2a/client.rs:25` (`Client::new()` with no
  timeout), `:49-84` (discover), `:108-157` (send), `:298-331` (get),
  `:334-367` (cancel), `:242-280` (SSE read loop without idle timeout).
- Reachability: any framework consumer invoking A2A against a
  dead-but-alive remote blocks until OS TCP timeouts (potentially many
  minutes), with no recovery knob; the SSE stream hangs forever if the
  server stops sending.
- Expected invariant: client operations complete or fail in bounded
  time.
- Observed behavior: unbounded awaits.
- Impact: hung agent turns/futures in consumers; no cancellation story at
  the transport level.
- Root cause: default reqwest client without connect/read/overall
  timeouts.
- Direction: set connect + overall timeouts (and an idle read timeout for
  SSE) or expose them on `A2AClient::new`/builder; regression fixture
  with a non-answering stub server asserting the deadline.
- Regression validation: tokio test against a `TcpListener` that accepts
  and never responds, asserting `send_task`/`discover` return an error
  within the configured timeout.
- Validation reports: [V03-11](../validations/F-INT-02/V03-11.md)

### F-INT-02-P2-08: `JwtConfig::rs256` cannot validate any token

- Priority: P2
- Confidence: high
- Layer: framework (adapter boundary)
- Evidence: `echo-agent/src/a2a/auth.rs:75-83` (`rs256` stores the PEM
  public key in `secret`), `:247` (`validate_token` always builds
  `DecodingKey::from_secret`, never `from_rsa_pem`/`from_ec_pem`).
- Reachability: `serve_with_auth(.., JwtConfig::rs256(pem))` accepts no
  token (HMAC verification against RSA key material fails); AUDIT_REPORT
  §7 claim "algorithm restriction ... proper" is regressed (V05-01).
- Expected invariant: configuring RS256 verifies RS256-signed bearer
  tokens.
- Observed behavior: all validation attempts fail; the API misleads
  callers into believing RSA auth is supported.
- Impact: RS256-deployed A2A endpoints reject every client (or force
  callers onto HS256 with a shared secret).
- Root cause: `DecodingKey` selection ignores the configured algorithm
  family.
- Direction: build `DecodingKey` from PEM when RS/ES algorithms are
  configured (`DecodingKey::from_rsa_pem`/`from_ec_pem`); add a
  sign-verify round-trip test for both HS256 and RS256.
- Regression validation: test that creates an HS256 token and an RS256
  token, validates each under the matching `JwtConfig`, and asserts the
  mismatched pair is rejected.
- Validation reports: [V03-10](../validations/F-INT-02/V03-10.md),
  [V05-01](../validations/F-INT-02/V05-01.md)

### F-INT-02-P2-09: A2A server task registry and cancel tokens never reclaimed

- Priority: P2
- Confidence: high
- Layer: framework (adapter boundary)
- Evidence: `echo-agent/src/a2a/server.rs:166-183` (insert on
  subscribe), `:389-408` (insert on send), `:605-625`
  (`cleanup_completed_tasks` — zero production callers, verified by grep),
  `:189-344` (dropped SSE stream leaves the task in Submitted/Working and
  its token in `cancel_tokens`).
- Reachability: every A2A request (including clients that disconnect
  mid-stream) leaves a permanent entry in the in-memory registry for the
  server's lifetime; `serve_inner` installs no periodic cleanup.
- Expected invariant: task records are eventually reclaimed (the doc at
  :604 says "called periodically by the caller" — no caller exists).
- Observed behavior: unbounded growth of `tasks` and `cancel_tokens`
  maps.
- Impact: long-running A2A deployments grow memory without bound; dropped
  streams leave zombie "working" tasks queryable forever.
- Root cause: cleanup API defined but never wired into `serve_inner` or
  any owner.
- Direction: run a periodic cleanup task inside `serve_inner` (or expose
  an owned cleanup loop), and mark tasks orphaned by dropped streams
  (e.g., cancel on drop or TTL on non-terminal tasks); regression test
  with a disconnected stream asserting the entry is removed.
- Regression validation: serve-level test that subscribes, drops the SSE
  stream, runs cleanup, and asserts the task/cancel-token maps are
  empty.
- Validation reports: [V02-03](../validations/F-INT-02/V02-03.md),
  [V03-09](../validations/F-INT-02/V03-09.md)

### F-INT-02-P3-01: LSP tool URIs are not percent-encoded

- Priority: P3
- Confidence: high
- Layer: framework (tools)
- Evidence: `echo-agent/src/tools/lsp.rs:479-485` (`path_to_uri` is
  `format!("file://{path}")`; no percent-encoding, no canonicalization).
- Reachability: files whose paths contain spaces or URI-reserved
  characters produce non-conforming LSP URIs; strict servers fail the
  request (surfaced as a tool error), lenient servers may misresolve.
- Expected invariant: LSP URIs are encoded per RFC 3986.
- Observed behavior: raw path interpolation.
- Impact: flaky LSP behavior on unusual project paths.
- Root cause: convenience formatting instead of URI encoding.
- Direction: percent-encode (and canonicalize/absolutize) the path; keep
  `uri_to_path` decoding symmetric; add fixtures with spaces and CJK
  characters.
- Regression validation: unit test `path_to_uri`/`uri_to_path`
  round-trip for paths with spaces and non-ASCII characters.
- Validation reports: [V03-04](../validations/F-INT-02/V03-04.md)

### F-INT-02-P3-02: LSP `did_open`/`did_change` hardcode document versions

- Priority: P3
- Confidence: high
- Layer: framework (integration)
- Evidence: `echo-agent/echo-integration/src/lsp/client.rs:466`
  (`"version": 1`), `:495` (`"version": 2`).
- Reachability: repeated `did_change` calls always send version 2; the
  LSP versioning contract is not maintained.
- Impact: older/strict servers may reject out-of-order changes; part of
  the root cause behind P2-03's staleness.
- Direction: maintain a per-URI monotonic version counter.
- Regression validation: unit test asserting versions increment across
  did_open/did_change sequences.
- Validation reports: [V03-03](../validations/F-INT-02/V03-03.md)

### F-INT-02-P3-03: `echo-core/src/lsp/mod.rs` architecture doc points at a non-existent tool module

- Priority: P3
- Confidence: high
- Layer: framework (docs)
- Evidence: `echo-agent/echo-core/src/lsp/mod.rs:11-12` ("echo-tools/src/lsp/
  ← Tool implementations"); no `echo-tools/src/lsp/` exists — tools live
  at `echo-agent/src/tools/lsp.rs` (V01-01).
- Impact: misleading architecture map for framework readers.
- Direction: fix the path in the module doc (and remove the dead
  reference if the layering note changes).
- Regression validation: none needed (doc-only).
- Validation reports: [V01-01](../validations/F-INT-02/V01-01.md),
  [V05-01](../validations/F-INT-02/V05-01.md)

### F-INT-02-P3-04: channel message sends have no retry or queue

- Priority: P3
- Confidence: high
- Layer: framework (integration)
- Evidence: `echo-agent/echo-integration/src/channels/channels/qq/channel.rs:118-129`
  (warn + `continue`, message dropped), `feishu/channel.rs:227-238`
  (same), `qq/api.rs:243-295` / `feishu/api.rs:296-360` (single-shot
  sends).
- Reachability: a transient network error during send silently loses the
  user's reply (no retry, no dead-letter, no at-least-once).
- Expected invariant: reply delivery is at-least-once or explicitly
  retryable.
- Observed behavior: drop-on-error.
- Impact: lost replies in flaky networks; acceptable for the local
  assistant context but undocumented.
- Direction: bounded retry with backoff in the send tasks (or document
  best-effort); regression test simulating a failing API then a
  succeeding one.
- Regression validation: unit test with a mock send that fails once and
  asserts the message is retried and delivered.
- Validation reports: [V03-07](../validations/F-INT-02/V03-07.md)

### F-INT-02-P3-05: A2A streaming re-subscribe overwrites task records; SSE request id is hardcoded

- Priority: P3
- Confidence: high
- Layer: framework (adapter boundary)
- Evidence: `echo-agent/src/a2a/server.rs:174-176` (`tasks.insert(task_id,
  initial_task)` clobbers an existing record), `serve.rs:275`
  (`format_sse_event(&event, "req-stream")` ignores the actual request
  id).
- Reachability: re-subscribing `tasks/sendSubscribe` with the same id
  loses prior history; SSE responses carry a fixed id that cannot be
  correlated by the client.
- Impact: history loss and weak request correlation.
- Direction: reject or resume existing ids (idempotency), and thread the
  parsed request id through `handle_sse`.
- Regression validation: server test asserting a second subscribe with
  the same id does not lose the stored task.
- Validation reports: [V03-09](../validations/F-INT-02/V03-09.md)

### F-INT-02-P3-06: A2A client SSE parser misses CRLF-framed events

- Priority: P3
- Confidence: medium
- Layer: framework (adapter boundary)
- Evidence: `echo-agent/src/a2a/client.rs:267-279` (`buffer.find("\n\n")`
  only; `\r\n\r\n` sequences are never split until the stream ends and
  the trailing buffer is flushed).
- Reachability: interop with SSE servers using CRLF line endings
  delivers no events until stream end (or never, for long-lived
  streams).
- Impact: incomplete streaming interop.
- Direction: normalize `\r\n` to `\n` before splitting (and handle `\r`
  line endings); add a CRLF fixture test.
- Regression validation: unit test feeding a `\r\n\r\n`-framed payload
  and asserting both events are parsed.
- Validation reports: [V03-10](../validations/F-INT-02/V03-10.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search | yes | passed | [V01-01](../validations/F-INT-02/V01-01.md) |
| V02-01 | LSP registration/reachability | yes | passed | [V02-01](../validations/F-INT-02/V02-01.md) |
| V02-02 | Channels registration/reachability | yes | passed | [V02-02](../validations/F-INT-02/V02-02.md) |
| V02-03 | A2A registration/reachability | yes | passed | [V02-03](../validations/F-INT-02/V02-03.md) |
| V03-01 | LSP lifecycle | yes | passed | [V03-01](../validations/F-INT-02/V03-01.md) |
| V03-02 | LSP malformed input | yes | passed | [V03-02](../validations/F-INT-02/V03-02.md) |
| V03-03 | LSP retry/cancel | yes | passed | [V03-03](../validations/F-INT-02/V03-03.md) |
| V03-04 | LSP naming conversion | yes | passed | [V03-04](../validations/F-INT-02/V03-04.md) |
| V03-05 | Channels lifecycle | yes | passed | [V03-05](../validations/F-INT-02/V03-05.md) |
| V03-06 | Channels malformed input | yes | passed | [V03-06](../validations/F-INT-02/V03-06.md) |
| V03-07 | Channels retry/cancel | yes | passed | [V03-07](../validations/F-INT-02/V03-07.md) |
| V03-08 | Channels naming conversion | yes | passed | [V03-08](../validations/F-INT-02/V03-08.md) |
| V03-09 | A2A lifecycle | yes | passed | [V03-09](../validations/F-INT-02/V03-09.md) |
| V03-10 | A2A malformed input | yes | passed | [V03-10](../validations/F-INT-02/V03-10.md) |
| V03-11 | A2A retry/cancel | yes | passed | [V03-11](../validations/F-INT-02/V03-11.md) |
| V03-12 | A2A naming conversion | yes | passed | [V03-12](../validations/F-INT-02/V03-12.md) |
| V04-01 | `cargo test -p echo_integration --features lsp,channels --lib lsp::` | yes | passed (exit 0; 9 passed, 1 ignored) | [V04-01](../validations/F-INT-02/V04-01.md) |
| V04-02 | `cargo test -p echo_integration --features lsp,channels --lib channels::` | yes | passed (exit 0; 6 passed) | [V04-02](../validations/F-INT-02/V04-02.md) |
| V04-03 | `cargo test -p echo_agent --features a2a --lib a2a::` | yes | passed (exit 0; 38 passed) | [V04-03](../validations/F-INT-02/V04-03.md) |
| V04-04 | `cargo check -p echo_agent --no-default-features --features lsp,channels,a2a --locked` | yes | passed (exit 0) | [V04-04](../validations/F-INT-02/V04-04.md) |
| V05-01 | Historical-document drift | yes | passed | [V05-01](../validations/F-INT-02/V05-01.md) |

Note: "passed" here means the inspection/command completed with the
expected outcome; several validations document failing invariants, which
are recorded as findings (per REPORTING.md, a failed validation does not
block completion — it becomes a finding).

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AUDIT_REPORT §1.10 "A2A Server Defaults to No Authentication" (warning recommendation) | fixed (warning added; `serve()` retained by design) | `echo-agent/src/a2a/serve.rs:103-117` |
| AUDIT_REPORT §7 #7 "JWT auth middleware — proper ... algorithm restriction" | regressed (RS256 cannot validate) | `echo-agent/src/a2a/auth.rs:247` → F-INT-02-P2-08 |
| MASTER-PLAN:987 "LSP/monitors/themes/output styles discovery (框架无消费者)" | current | plugin component discovery path (`echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs`) |
| MASTER-PLAN:1030 "cli/channels.rs:CLI/channel 消费者" | current | `echo-agent-cli/src/cli/channels.rs`, `src/cli/modes.rs:110-255` |
| PROJECT-ANALYSIS.md:19 IM channel projection | current | `echo-agent-cli/src/cli/modes.rs:58` |
| echo-core/src/lsp/mod.rs:11-12 "echo-tools/src/lsp/ ← Tool implementations" | stale | no such module; tools at `echo-agent/src/tools/lsp.rs` → F-INT-02-P3-03 |

## Coverage And Uncertainty

- `echo-agent/src/tools/lsp.rs` goto_definition/find_references/hover
  bodies were read; hover contents parsing is lossy for array-form
  `contents` (JSON-stringified) — observed, not promoted to a finding
  (tool-output cosmetic).
- QQ/Feishu live behavior (real token exchange, gateway frames) was not
  executed — no credentials; the opt-in LSP smoke test was skipped
  (`EKO_LSP_SMOKE` unset). Platform protocol correctness rests on static
  review only (V04-02).
- The `demo55_lsp_tools` example was not executed.
- A2A `handle_sse` returning HTTP 500 for parse errors in
  `handle_request_stream` (serve.rs:283-291) was noted in V03-10 as a
  minor protocol deviation, not a finding (error path still returns a
  structured failure).
- EKO app-layer channel/HITL policy (A-SRF-04, A-HITL-01, A-INT-01) was
  not reviewed here; only reachability was established.
- The `A2AClient` URL-fetching surface was recorded as an observation
  (caller-owned per local threat model), not a finding.

## Handoff

- Downstream tasks may rely on: single-authority conclusions (V01),
  reachability traces (V02-01..03), the three P1 findings (LSP request
  hang/pending leak, QQ send-task spin, A2A cancel semantics), and the
  dead-contract items (LSP restart, A2A cleanup).
- `A-INT-01` should treat F-INT-02-P1-01 and F-INT-02-P2-03 as live EKO
  surfaces (LSP tools registered at boot; plugin reload hangs on hung
  servers).
- `X-SRF-01`/`A-SRF-04` should re-check channel teardown (P1-02, P2-05)
  in the EKO context.
- `X-EVT-01`/`X-TSK-01`: A2A `TaskState` is a wire-protocol artifact and
  must not be conflated with the framework task model (V03-12).
- Reports to read: all validations listed in the matrix.
- This report becomes stale if the LSP client, channel lifecycle, or A2A
  server/client code changes.
- Follow-up task IDs: A-INT-01 (EKO LSP/channel surfaces), X-BND-01
  (capability placement), S-RDM-01 (roadmap, fix directions above).
