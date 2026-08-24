# F-INT-01: MCP and representative integration adapter contracts

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: source repositories clean at final source inspection; reports
> live outside both source repositories

## Question

Do MCP configuration, client/server transports, Tool adaptation, cancellation,
reconnect, and schema handling preserve framework contracts, and do representative
LSP/A2A adapters exhibit the same typed lifecycle and protocol-boundary discipline?

## Scope

- Full MCP definition/export/config/manager/client/server/Tool/transport path,
  React registration, and live EKO Tauri/TUI/plugin reachability.
- Representative full LSP chain: core contract, integration process client and
  manager, root Tool adapters, EKO startup/registration.
- Representative A2A client/server task, stream, cancellation, identity, error,
  and cleanup chain.
- Static test/history/docs, UTF-8, panic, and overflow inspection.

## Out Of Scope

- Source fixes and all Cargo/rustc/test/build/network/dynamic fixtures.
- IM channels and exhaustive F-INT-02 coverage.
- Generic Tool registry/schema/result defects already owned by F-EXT-01, except
  protocol-specific mapping manifestations.
- Core Agent event authority already owned by F-CORE-01, except the exact A2A
  terminal conversion.
- EKO UI behavior beyond proving MCP/LSP live registration.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md`; Codex protocol.
- [F-EXT-01](F-EXT-01.md): generic Tool authority, collisions, schema and result
  boundary, used to avoid duplicating generic findings.
- [F-CORE-01](F-CORE-01.md): canonical typed Agent event/error/cancellation
  boundary used to evaluate adapter loss.
- [B-ARCH-01](B-ARCH-01.md) and [B-REF-01](B-REF-01.md): feature/facade and
  current-vs-historical classification boundary.
- Current source and scoped Git history. No other reviewer report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Protocol framing, typed requests/results/errors, capability negotiation, session lifecycle, cancellation, reconnect, schema conversion, and rich-result adaptation are reusable framework mechanisms. |
| EKO product policy | Which user-configured servers start, project discovery, UI status, retry prompts, and local interaction policy remain application concerns. |
| Adapter boundary | Protocol adapters should perform lossless conversion and lifecycle delegation; they must not silently invent success, duplicate registry authority, or own uncorrelated cancellation. |
| Duplicate search | Searched MCP/LSP/A2A definitions, features, exports, registration, transport/session/pending/reconnect/cancel/timeout, schema/capability/result/error/identity, and all live callers across both repositories. |
| Migration deletion | Replace incomplete transport/request loops and lossy converters rather than retaining parallel paths; remove duplicate broad command policy when one lightweight configuration validator is authoritative. |

## Current Paths

```text
EKO MCP config/UI/TUI/plugin
  -> ReactAgent::connect_mcp_from_config
  -> McpManager -> McpClient -> HTTP | SSE | stdio
  -> McpToolAdapter -> ToolManager -> model Tool call

EKO project/plugin startup
  -> LspManager -> StdioLspClient
  -> five root LSP Tool adapters -> model Tool call

standalone framework consumer
  -> A2AClient HTTP/SSE <-> A2AServer
  -> raw Agent execute/execute_stream -> A2A task/event projection
```

## Findings

### F-INT-01-P1-01: Streamable HTTP advertises an asynchronous response path with no response reader

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/mcp/transport/http.rs:25`,
  `:39`, `:69`, `:160`, `:230`; selected by `McpClient::new` and live EKO MCP.
- Expected: a 202 response is correlated to later server output and close settles
  each request/session.
- Observed/impact: no code writes the stored pending sender or notification
  broadcaster, so every 202 waits 60 seconds then fails; notification errors and
  session termination are also discarded.
- Root cause/direction: Streamable HTTP was modeled by fields/comments without a
  receiving lifecycle. Implement one spec-shaped receive/session/cancel path or
  remove the unsupported 202 claim/branch until it exists.
- Regression: synchronous response, 202 response, notification, session close,
  cancel, timeout, 5xx, and no orphaned pending entries.
- Validation: [V03](../validations/F-INT-01/V03-01.md)

### F-INT-01-P1-02: SSE and stdio lose transport ownership on common failure paths

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/mcp/transport/sse.rs:98`,
  `:202`, `:322`, `:415`; `echo-agent/echo-integration/src/mcp/transport/stdio.rs:144`,
  `:188`, `:265`; both are public manager-selected transports.
- Expected: EOF/error reconnects or completes all waiters, notifications remain
  observable, and close drains owned work.
- Observed/impact: normal SSE EOF never reconnects; several send/timeout paths
  retain pending entries and close neither drains nor awaits. Stdio read/write
  failures can retain waiters, and all server notifications are discarded.
- Root cause/direction: background task, pending map, and caller lifetimes have no
  single session owner. Introduce one transport state machine with terminal drain,
  bounded reconnect, and correlated cancellation; delete per-branch cleanup.
- Regression: EOF/read/write failure, endpoint race, timeout/cancel/close, list
  notification, reconnect exhaustion, and pending count zero.
- Validation: [V04](../validations/F-INT-01/V04-01.md)

### F-INT-01-P1-03: MCP advertises unimplemented client capabilities and treats incomplete discovery as success

- Priority: P1; confidence: high; layer: framework.
- Evidence/reachability: `echo-agent/echo-integration/src/mcp/client.rs:91`, `:148`,
  `:162`, `:195`; every connection performs this handshake/discovery.
- Expected: negotiate a supported version, advertise only handled methods, and
  publish caches only after complete discovery.
- Observed/impact: server version is accepted blindly; roots/list-changed,
  sampling, and elicitation are advertised without handlers/receivers; a later
  page error returns a partial list as `Ok`, creating false capability state.
- Root cause/direction: declarations and cache fetching are not coupled to a
  request dispatcher/transaction. Add handlers or remove declarations; validate
  version and fail discovery atomically with typed error.
- Regression: incompatible version, server request per advertised capability,
  changed notification, page-N error, cursor cycle/limit, and cache immutability.
- Validation: [V05](../validations/F-INT-01/V05-01.md)

### F-INT-01-P1-04: MCP server advertisement and execution can diverge and rich results are flattened

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/mcp/server.rs:409`, `:446`,
  `:456`, `:654`; `echo-agent/echo-integration/src/mcp/tool_adapter.rs:166`.
- Expected: one schema/identity resolves both listing and execution; structured,
  binary, failure, metadata, and artifact facts remain reconstructable.
- Observed/impact: duplicate names remain in the advertised vector but last-write
  wins in the execution map; input schema is not enforced; server conversion
  reduces ToolResult to text, while client rich media becomes placeholders.
- Root cause/direction: list/map and result DTOs are independently built. Use one
  validated registration record and explicit lossless result/artifact conversion;
  delete duplicate list authority and text-only conversion.
- Regression: duplicate name, malformed schema, structured result, image/audio,
  failed ToolResult, metadata/artifact round-trip.
- Validation: [V06](../validations/F-INT-01/V06-01.md)

### F-INT-01-P1-05: Live LSP requests have no timeout/cancel owner and process status becomes stale

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/lsp/client.rs:120`, `:202`,
  `:523`; `echo-agent/echo-integration/src/lsp/manager.rs:76`, `:157`; EKO registers
  all five LSP tools.
- Expected: all requests are bounded/cancellable, process exit completes callers
  and status/restart counters describe reality.
- Observed/impact: only initialize is timed out; capability requests can wait
  forever. EOF clears pending but leaves running/initialized true, while documented
  max restart/count/error fields are inert.
- Root cause/direction: process reader and manager status/restart policy are
  disconnected. Give the session one supervised owner with token/deadline, exit
  observation, typed failure, and real restart accounting; remove inert fields if
  automatic restart is not supported.
- Regression: stalled request, Tool cancel, writer loss, process exit, restart
  limit/backoff, status transition, and shutdown.
- Validation: [V07](../validations/F-INT-01/V07-01.md),
  [V08](../validations/F-INT-01/V08-01.md)

### F-INT-01-P1-06: LSP framing trusts an unbounded Content-Length

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/lsp/client.rs:125`,
  `:154`, `:164`; every started language server feeds this reader.
- Expected: malformed or excessive frames produce bounded typed failure.
- Observed/impact: arbitrary Content-Length is used directly for allocation,
  allowing a faulty local language server to exhaust process memory.
- Root cause/direction: framing has no configured maximum. Reject excessive
  length before allocation, terminate the session, and drain callers.
- Regression: zero/invalid/missing/excessive/truncated frames and exact maximum.
- Validation: [V08](../validations/F-INT-01/V08-01.md)

### F-INT-01-P1-07: LSP numeric wire enums decode as empty successful results

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/echo-core/src/lsp/types.rs:36`, `:69`;
  `echo-agent/echo-integration/src/lsp/client.rs:190`, `:353`, `:444`.
- Expected: protocol numeric enum values deserialize, while malformed responses
  return typed failure.
- Observed/impact: simplified enums derive default string serde although LSP uses
  numeric severities/kinds; `unwrap_or_default` turns failures into empty
  diagnostics/completions/locations, making live capabilities silently lie.
- Root cause/direction: internal domain types are used as wire DTOs. Add explicit
  protocol conversion (including unknown values) and propagate parse failure;
  delete empty-success fallbacks.
- Regression: every numeric value, missing/unknown kind/severity, malformed
  location and completion list, and real representative server envelopes.
- Validation: [V09](../validations/F-INT-01/V09-01.md)

### F-INT-01-P1-08: A2A cancel updates projection state but does not cancel Agent execution

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/src/a2a/server.rs:179`, `:207`, `:404`, `:414`,
  `:527`; public server used by framework examples/consumers.
- Expected: cancel reaches the Agent/tool execution and terminal state cannot be
  overwritten.
- Observed/impact: created tokens are not passed into `execute`/`execute_stream`;
  streaming checks only between events and sync completion overwrites Canceled
  with Completed. Stalled work continues after accepted cancellation.
- Root cause/direction: A2A task state is a projection detached from canonical
  invocation context. Pass the token/deadline into one owned Agent execution and
  apply monotonic terminal writes; delete polling-only cancellation.
- Regression: sync/stream model stall, Tool stall, cancel before/after event,
  late completion, exactly one terminal status, no late side effect.
- Validation: [V10](../validations/F-INT-01/V10-01.md)

### F-INT-01-P1-09: A2A streaming can convert Agent failure to Completed and caller IDs can replace active ownership

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/src/a2a/server.rs:154`, `:175`, `:224`, `:278`,
  `:296`, `:379`, `:400`.
- Expected: typed error remains Failed, active identity cannot be replaced, and
  every token is cleaned on terminal paths.
- Observed/impact: envelope `AgentEvent::Error` enters `Ok(_)`, is ignored, and EOF
  then writes Completed. Duplicate caller task IDs replace task/token entries while
  old work continues. Some early stream failures return without token cleanup.
- Root cause/direction: projection pattern matching does not own terminal event
  semantics or enforce active-ID uniqueness. Use canonical terminal classifier and
  reservation/generation identity; centralize cleanup in an owned guard.
- Regression: Error payload vs stream error, duplicate concurrent ID, caller drop,
  execute_stream setup error, token map zero, terminal monotonicity.
- Validation: [V10](../validations/F-INT-01/V10-01.md)

### F-INT-01-P1-10: A2A client loses split Unicode and hides streaming transport failure

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/src/a2a/client.rs:37`, `:195`, `:242`, `:250`,
  `:298`, `:334`; all public client calls use these paths.
- Expected: frame bytes before UTF-8 decode, expose terminal stream error, bound
  calls, preserve remote error code/provenance.
- Observed/impact: chunks are decoded independently, so split Unicode is dropped;
  read/decode/parse errors only log/end because stream items cannot carry errors.
  Requests have no configured timeout and errors flatten to Other text.
- Root cause/direction: network chunk, SSE event, and domain event boundaries are
  conflated. Buffer bytes through event framing, return Result/event error, and use
  typed bounded request policy.
- Regression: every UTF-8 split point, CRLF/multi-data events, malformed JSON,
  disconnect/timeout/HTTP error, remote code and terminal observability.
- Validation: [V11](../validations/F-INT-01/V11-01.md)

### F-INT-01-P2-01: MCP teardown/backoff contains public no-panic and overflow violations

- Priority: P2; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/mcp/transport/stdio.rs:270`,
  `echo-agent/echo-integration/src/mcp/transport/sse.rs:124`.
- Expected: public transport drop is runtime-independent; server-controlled
  backoff arithmetic cannot overflow.
- Observed/impact: Stdio Drop unconditionally calls `tokio::spawn`, which may panic
  outside a runtime; SSE doubles parsed u64 before applying a cap.
- Root cause/direction: cleanup assumes ambient runtime and caps after arithmetic.
  Use owned kill-on-drop/supervisor cleanup and checked/saturating arithmetic.
- Regression: drop in synchronous context and extreme retry values.
- Validation: [V04](../validations/F-INT-01/V04-01.md)

### F-INT-01-P2-02: MCP exposed-name sanitization is non-injective

- Priority: P2; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/mcp/tool_adapter.rs:35`,
  `:77`; all discovered MCP tools use this constructor.
- Expected: distinct server/tool identities remain distinct registry keys.
- Observed/impact: punctuation and all non-ASCII characters collapse to `_`, so a
  later tool can silently replace another. Generic replacement behavior is owned
  by F-EXT-01; this finding owns the protocol namespace conversion.
- Root cause/direction: sanitization is used as identity. Use collision-resistant
  encoding or reject duplicates before registration while retaining original IDs.
- Regression: punctuation/Unicode/empty segments, duplicate reconnect/disconnect.
- Validation: [V02](../validations/F-INT-01/V02-01.md)

### F-INT-01-P2-03: MCP command policy is duplicated and overbroad for trusted local extensions

- Priority: P2; confidence: high; layer: adapter/application boundary.
- Evidence/reachability: `echo-agent/echo-integration/src/mcp/config_loader.rs:107`,
  `:117`; `echo-agent/echo-integration/src/mcp/transport/stdio.rs:284`, `:323`.
- Expected: one lightweight malformed-input validator, with user-selected local
  extension policy left to EKO/user configuration.
- Observed/impact: two validators differ, and transport rejects any `-o*` argument
  regardless of executable/meaning, blocking legitimate MCP configurations while
  providing no single coherent URL/command contract.
- Root cause/direction: product trust policy was embedded in a generic transport.
  Keep empty/syntax validation at config conversion; remove broad argv policy or
  make any optional warning/application policy explicit.
- Regression: known valid npx/python/ssh/unrelated `-o*`, empty command, malformed
  URL, and direct-vs-file config parity.
- Validation: [V12](../validations/F-INT-01/V12-01.md)

### F-INT-01-P2-04: LSP document/URI/position conversion is not a stable protocol adapter

- Priority: P2; confidence: high for URI/version/cast, medium for position units;
  layer: adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/lsp/client.rs:455`, `:475`;
  `echo-agent/echo-integration/src/lsp/manager.rs:58`;
  `echo-agent/src/tools/lsp.rs:458`, `:478`.
- Expected: encoded file URIs, monotonic document lifecycle, checked numeric
  conversion, and explicit UTF-16 position semantics.
- Observed/impact: file URIs are string concatenation, every query repeats didOpen
  version 1 without didClose, didChange always uses 2, and u64 Tool arguments cast
  to u32. Paths with reserved characters, repeated queries, large inputs, and
  non-ASCII columns can produce wrong server state/locations.
- Root cause/direction: root Tool helpers own ad hoc protocol state/conversion.
  Centralize URI/document-version/position conversion in the LSP adapter and track
  open documents; remove per-call didOpen and unchecked casts.
- Regression: spaces/#/Unicode paths, repeated open/change/close, u32 boundary,
  emoji/CJK UTF-16 positions.
- Validation: [V09](../validations/F-INT-01/V09-01.md)

## Validation Matrix

| ID | Claim | Status | Report |
|---|---|---|---|
| V01 | feature/export/layering | passed | [V01](../validations/F-INT-01/V01-01.md) |
| V02 | MCP registration/reachability/identity | failed | [V02](../validations/F-INT-01/V02-01.md) |
| V03 | MCP Streamable HTTP lifecycle | failed | [V03](../validations/F-INT-01/V03-01.md) |
| V04 | MCP SSE/stdio failure and cleanup | failed | [V04](../validations/F-INT-01/V04-01.md) |
| V05 | MCP handshake/capability/discovery | failed | [V05](../validations/F-INT-01/V05-01.md) |
| V06 | MCP schema/result/server mapping | failed | [V06](../validations/F-INT-01/V06-01.md) |
| V07 | LSP definition/registration/layering | passed | [V07](../validations/F-INT-01/V07-01.md) |
| V08 | LSP request/process lifecycle | failed | [V08](../validations/F-INT-01/V08-01.md) |
| V09 | LSP wire/document mapping | failed | [V09](../validations/F-INT-01/V09-01.md) |
| V10 | A2A identity/cancel/terminal lifecycle | failed | [V10](../validations/F-INT-01/V10-01.md) |
| V11 | A2A client streaming/error behavior | failed | [V11](../validations/F-INT-01/V11-01.md) |
| V12 | local extension config boundary | failed | [V12](../validations/F-INT-01/V12-01.md) |
| V13 | tests/history/docs and future fixtures | not_run | [V13](../validations/F-INT-01/V13-01.md) |
| V14 | report integrity and source isolation | passed | [V14](../validations/F-INT-01/V14-01.md) |
| V30 | primary source-anchor sampling and acceptance | passed | [V30](../validations/F-INT-01/V30-01.md) |

Primary static acceptance is recorded in V30. Executable fixtures are deliberately
future validation under the explicit review-stage ban, not silently claimed as
completed.

## Historical Classification

| Claim | Classification | Current evidence |
|---|---|---|
| Server-qualified MCP names preserve identity (`c9f7e25`) | current but incomplete | Prefixing exists; sanitizer remains non-injective (V02). |
| Event envelopes preserve typed failures (`dba349e`) | regressed at A2A adapter | Error payload is ignored and terminal state becomes Completed (V10). |
| Auto-discovered LSP is live (`1c6442e`) | current | EKO startup and five Tool registrations are live (V07). |
| Docs: complete/latest MCP client | stale | 202/capability paths incomplete; docs/code protocol dates differ (V03-V05, V13). |
| Docs: LSP max_restarts is operational | stale | field is configured/exposed but not enforced or incremented (V08, V13). |
| README: one unified retry covers MCP/A2A | stale | MCP HTTP owns a local retry loop; A2A calls do not apply it (V03, V11, V13). |

## Coverage Gaps And Residual Uncertainty

- No executable fixture ran by explicit instruction. Future work should turn each
  regression list into a deterministic mock transport/process validation.
- LSP position-unit impact is medium-confidence until a representative real
  language server fixture confirms caller conventions; numeric enum mismatch is
  source-conclusive.
- A2A auth and public deployment hardening were not assessed; this review does not
  reinterpret local user extensions as an internet multi-tenant threat.
- IM channel coverage remains F-INT-02.

## Handoff

Prioritize MCP HTTP session/response ownership, A2A canonical cancellation and
terminal classification, then LSP wire DTOs and bounded request/framing. Preserve
framework/application separation: EKO chooses/configures local capabilities;
framework integrations own lossless protocol mapping and lifecycle. Downstream
work must reread this report if any scoped transport, protocol DTO, Tool result,
Agent event/cancel contract, feature gate, or EKO registration path changes.
