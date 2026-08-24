# A-INT-01: Browser, MCP, and LSP application integration

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both repositories clean

## Question

Are local browser sessions and user-configured MCP/LSP capabilities
reachable, recoverable, and not blocked by irrelevant permission gates?

Answer: reachable on every interactive surface, mostly well-recovered
(browser sidecar restart, agent-level MCP reconnect, session restore), and
**not blocked by any `permission_mode` gate** (the AGENTS.md hard rule is
not triggered). However: the GUI MCP config editor silently loses all user
edits on restart (P1-01); the CLI `/mcp connect|disconnect` commands are
no-op stubs (P2-01); boot-time MCP config handling is all-or-nothing and
unbounded (P2-02); the GUI MCP dialog applies SSRF-style private-range/HTTPS
validation that contradicts the local-assistant model and the config-file
path (P2-03); dropped user MCP servers never auto-recover (P2-04); browser
consequential-action confirmations inherit the REPL EOF auto-approve defect
(P2-05, cross-reference A-HITL-01-P1-02).

## Scope

- EKO browser integration: `echo-agent-cli/echo-agent-app-core/src/browser/
  {mod.rs (1996 lines), session.rs, sidecar.rs, config.rs, risk.rs, event.rs,
  error.rs}` (full reads), tool installation sites `infra.rs:454-455,
  951-952,1028-1029`, `runtime.rs:93-100,144-146`, Tauri commands
  `src/tauri/commands/browser.rs` (full read), shutdown wiring `main.rs:
  299,334,399,443`, `desktop.rs:263-267`.
- EKO MCP integration: `src/tauri/commands/mcp.rs` (full read), `infra.rs:
  1069-1108` (boot loader), `state.rs:281,357,490,750-797` (config + health),
  `runtime.rs:110`, `plugin_runtime.rs:1172-1173`, `config_discovery.rs:
  243-262`, `types/request.rs:35,response.rs:59`, TUI `/mcp` (`tui/events.rs:
  3434-3450`), CLI `/mcp` (`cli/cmd_impls/skills.rs:236-274`), frontend
  `web-frontend/src/components/mcp/McpPanel.tsx`, `api/endpoints.ts:206-228`.
- EKO LSP integration: `runtime.rs:499-592` (`register_lsp_tools`), plugin
  LSP reload paths (`plugin_runtime.rs`, cross-checked with F-INT-02).
- Framework anchors (reachability + validation semantics only):
  `echo-agent/src/agent/react/capabilities.rs:1149-1331` (agent MCP surface),
  `echo-integration/src/mcp/{client.rs,config_loader.rs,transport/stdio.rs}`,
  `echo-agent/src/agent/snapshot.rs:227-236` (plan-mode filter), agent pool /
  HITL wiring (`runtime.rs:128-146`), framework `McpManager`/`LspManager`.

## Out Of Scope

- Framework MCP/LSP protocol correctness (transports, cancellation, SSE,
  server side) -> F-INT-01, F-INT-02 (dependency reports read; findings
  cross-referenced, not re-audited).
- Permission/HITL decision semantics and per-surface providers -> F-HITL-01,
  A-HITL-01 (read; browser confirmation routing re-verified here).
- Terminal / tool exposure / sandbox -> A-TOOL-01 (dependency report read;
  `create_terminal` gate status cross-referenced only).
- Frontend stores/reducers for browser/MCP panels -> A-FE-01..03 (only the
  MCP panel save flow and endpoints were checked).
- TaskRuntime executor behavior -> A-TSK-01..06 (subagent tool surfaces
  referenced only for the LSP parity note).
- Live dynamic scenarios (real Playwright sidecar, real MCP/LSP servers) ->
  Q-E2E-01 (environmental; read-only review).

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-INT-01 card), `zcode-ds/README.md`, report templates.
- Dependency reports read: zcode-ds `F-INT-01` (complete), `F-INT-02`
  (complete), `A-TOOL-01` (complete); cross-referenced: `A-HITL-01`
  (complete, read in full for the REPL/approval chain).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md`,
  `echo-agent-cli/docs/MASTER-PLAN.md`, `browser-runtime-design.md`,
  `2026-07-17-surface-parity-closeout.md`, `2026-07-28-app-core-full-audit.md`.

## Layering Decision

- Generic mechanism (framework, correct placement, reused as-is): `McpClient`/
  `McpManager`/transports/`McpServerConfig`/`McpConfigFile` (echo-integration),
  `LspManager`/`StdioLspClient` (echo-integration), `McpToolAdapter` tool
  exposure. EKO adds no parallel implementation (V01-01).
- EKO product policy (application, correct placement): `BrowserRuntime`/
  `BrowserSessionManager`/`BrowserSidecar`/`BrowserConfig` (app-core
  `browser/`), mcp.json discovery + boot loading, Tauri MCP commands + input
  validation, `McpHealthStatus`/health loop, CLI/TUI `/mcp` commands, LSP
  config discovery + tool registration at boot, the browser approval-provider
  wiring.
- Adapter boundary: thin — `BrowserSidecar::server_config` converts
  `BrowserConfig` -> `McpServerConfig` (stdio); `BrowserRuntime::call` ->
  framework `McpClient::call_tool` with EKO-side cancel/restart policy;
  `confirm_action` -> `HumanLoopRequest`. No scheduling/state authority
  duplicated (browser sessions are EKO-owned by design, matching the
  app-core audit verdict at 2026-07-28-app-core-full-audit.md:249).
- Duplicate search terms (both repositories, V01-01): `McpClient|McpManager|
  StdioTransport|SseTransport|HttpTransport|JsonRpcRequest`, `McpServerConfig|
  McpConfigFile|McpServerEntry`, `connect_mcp*|disconnect_mcp*`, `LspManager|
  StdioLspClient|LspConfig`, `BrowserRuntime|BrowserSessionManager|
  BrowserSidecar|BrowserSession|BrowserTab`, `mcp_config|McpConfigFile`,
  `mcp.json|.lsp.yaml`, `require_full_auto|IpcAuth|permission_mode`.
  Result: single authoritative implementation per concept; EKO consumers only;
  no second config surface with independent authority (the GUI `plugins.
  mcp_config` RwLock is the *same* config concept but never synced with disk —
  that is the P1-01 defect, not a duplicate implementation).

## Current Path

Verified call graph (V02-01):

1. **Boot MCP**: `AgentRuntime::bootstrap` (runtime.rs:110) ->
   `infra::load_mcp_config` (infra.rs:1069-1108; path precedence CLI `--
   mcp-config` > YAML `mcp.config_path` > `MCP_CONFIG_PATH` > `~/.echo-agent/
   mcp.json`) -> `agent.load_mcp_from_file` (capabilities.rs:1241) ->
   `to_server_configs` (all-or-nothing) -> per-server
   `connect_mcp_from_config` (reconnect removes stale tools first,
   capabilities.rs:1149-1162) -> `McpManager::connect` -> `McpClient::new`
   (initialize handshake, capability listing) -> tools registered as
   `mcp__server__tool`.
2. **GUI MCP**: six commands (tauri/mod.rs:170-180) over
   `plugins.mcp_config` + agent; `connect_mcp_server` un-gated with input
   validation; `update_mcp_config` updates the in-memory config and
   reconnects all servers in a background task (15 s per-server timeout).
3. **TUI/CLI MCP**: `/mcp list|load|disconnect` (functional) / `/mcp
   list|connect|disconnect` (connect/disconnect are stubs — P2-01).
4. **Browser**: one `BrowserRuntime` per process (runtime.rs:93-100);
   background prewarm of the Playwright sidecar (`npx -y @playwright/mcp@
   latest` via framework stdio `McpClient`); tools installed on primary
   (infra.rs:454-455) and writer/readonly subagents (infra.rs:951-952,
   1028-1029); per-call `call_mcp` with cancel-observe, retry-safe restart
   once, `PartialSideEffect` otherwise (mod.rs:867-938, 1057-1086);
   session manager per conversation with tab leases per run/execution
   (session.rs); shutdown/interrupt close clients + sessions on every entry
   point.
5. **LSP**: `register_lsp_tools` (runtime.rs:499-592) discovers `.lsp.yaml`
   (global + nearest project), starts `LspManager` per language, registers
   five tools on the primary agent only.
6. **HITL for browser**: `confirm_action` (mod.rs:978-1018) uses the
   per-conversation provider or the default provider = `HitlDispatcher`
   (runtime.rs:144-146), which includes the REPL provider (P2-05).

## Findings

### A-INT-01-P1-01: GUI MCP config editor never persists to disk and the panel state is never seeded from the on-disk file — every GUI-created server and disabled flag silently disappears on restart

- Priority: P1 (borders P0: silent loss of user configuration, with a
  success message claiming persistence)
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/commands/mcp.rs:476-559`
  (`update_mcp_config` — the "Persist the new config synchronously" step is
  only `*cfg = new_config.clone()` into the in-memory
  `AppState.plugins.mcp_config`; no file write exists anywhere: grep of
  `mcp.json` across echo-agent-cli shows only reads/display, and no
  `serde_json::to_vec`/`fs::write` of the config in mcp.rs);
  `echo-agent-app-core/src/state.rs:357` (`plugins.mcp_config: RwLock<
  McpConfigFile>`), `state.rs:490` (initialized to `McpConfigFile::default()`
  — the on-disk `~/.echo-agent/mcp.json` is loaded only into the agent at
  boot, infra.rs:1097, never into this RwLock); frontend `McpPanel.tsx:
  33-55` (`saveConfig` shows "配置已保存并应用" / "config saved and applied"
  and the panel reloads from `get_mcp_config`); `endpoints.ts:225-228`.
- Reachability: definition (`McpConfigFile`, state.rs:357) -> registration
  (tauri/mod.rs:175, `update_mcp_config`) -> live caller (GUI MCP panel
  Save button, McpPanel.tsx:228): every session. Consequence chain:
  save -> in-memory config + background reconnect (works for this process) ->
  app exit -> config gone; restart -> `plugins.mcp_config` empty again ->
  `list_mcp_servers` shows only boot-connected servers; `toggle_mcp_server`
  for a GUI-added server returns "not found in config" (mcp.rs:304-308);
  disabled flags reset to enabled.
- Expected invariant: a config editor that says "saved" persists the user's
  MCP configuration across restarts; the GUI panel reflects the same config
  the agent actually loaded (single source of truth).
- Observed behavior: GUI edits live only in process memory; nothing writes
  `mcp.json` (or any other file); the panel state is never seeded from disk
  at boot. Two divergent sources of truth (agent's boot-loaded file vs
  in-memory `plugins.mcp_config`), never synchronized.
- Impact: every server a user adds or edits in the GUI (and every
  enable/disable toggle) is silently lost on restart — the flagship
  "user-configured MCP" surface of the task question is not durable; the
  "已保存" success message is false. The agent's live tool set then differs
  from what the user believes they configured.
- Root cause: `update_mcp_config` was written as an in-memory + reconnect
  path (the comment claims persistence it does not perform); the
  `plugins.mcp_config` store predates the boot file loader and was never
  wired to it (load direction absent).
- Direction: (a) persist `update_mcp_config`'s new config to
  `~/.echo-agent/mcp.json` (atomic write, UTF-8-safe, matching
  `infra.rs:1091` path resolution) and (b) seed `plugins.mcp_config` from
  the same resolved path at boot (or drop the RwLock and read the file on
  demand); then reconnect. Delete the misleading "persist" comment; update
  the frontend message only when the file write succeeds.
- Regression validation: unit test: `update_mcp_config` writes a file that
  `load_mcp_config` (infra.rs:1069) reads back identically (round-trip);
  boot test: an on-disk `mcp.json` with 2 servers yields a non-empty
  `plugins.mcp_config` with the same entries; GUI fixture: save a server,
  restart runtime, `list_mcp_servers` still shows it configured.
- Validation reports: [V01-01](../validations/A-INT-01/V01-01.md),
  [V02-01](../validations/A-INT-01/V02-01.md), [V05-01](../validations/A-INT-01/V05-01.md)

### A-INT-01-P2-01: CLI `/mcp connect <name>` and `/mcp disconnect <name>` are no-op stubs that print success-like messages without connecting or disconnecting anything

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/cli/cmd_impls/skills.rs:236-274` —
  `"connect"` branch prints `Connecting to MCP server: {name}` and returns
  `CommandOutcome::Continue` with no `connect_mcp*` call; `"disconnect"`
  likewise prints and returns; the functional TUI equivalent exists
  (`tui/events.rs:3434-3450` `/mcp load|disconnect` calls
  `load_mcp_from_file`/`disconnect_mcp`).
- Reachability: definition (`McpCommand` via `cmd!` at skills.rs:276-283) ->
  registration (`repl.rs:169` `cmd_impls::skills::register_all`) -> live
  caller: interactive CLI REPL `/mcp connect my-server` or `/mcp disconnect
  my-server`.
- Expected invariant: a management command either performs the stated
  operation or returns a clear "not supported" error; CLI is a primary
  surface (TUI/GUI/CLI parity, AGENTS.md).
- Observed behavior: both subcommands print an assertion ("Connecting…/
  Disconnecting…") and do nothing; `list` works; `connect` cannot even work
  by name from the config file (no config lookup).
- Impact: a CLI user believes a server was connected/disconnected; the
  agent's tool set is unchanged. Silent false-success on a primary surface;
  the parity invariant (multi-mode functional equality) is violated on the
  MCP-management surface.
- Root cause: stub written before the connection plumbing existed and never
  completed; the TUI path got the real implementation, the CLI path did not.
- Direction: implement `connect` as config-file lookup (`mcp.json` entry by
  name -> `connect_mcp_from_config`) and `disconnect` as `disconnect_mcp`
  (mirroring tui/events.rs), or remove the subcommands and print a
  "configure in mcp.json / use the GUI" hint; add a REPL-level test that a
  `connect` on a real entry registers `mcp__` tools.
- Regression validation: CLI fixture: `/mcp connect <server>` with a stub
  config entry -> `list_mcp_servers` contains the name (after fix); the
  stub's false-success string is gone.
- Validation reports: [V02-01](../validations/A-INT-01/V02-01.md),
  [V01-01](../validations/A-INT-01/V01-01.md)

### A-INT-01-P2-02: Boot-time MCP load is all-or-nothing and unbounded — one invalid entry rejects the whole file (zero servers connect) and a hung-but-alive stdio server stalls startup for up to 120 s per server

- Priority: P2
- Confidence: high (all-or-nothing is a code fact); medium-high (hang
  requires a server that spawns and never answers)
- Layer: application (boot wiring); the validation semantics are framework-
  owned (`McpConfigFile::to_server_configs`)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:1069-1108`
  (`load_mcp_config` runs during `AgentRuntime::bootstrap`, runtime.rs:110 —
  blocks TUI and GUI startup with no per-server deadline);
  `echo-agent/echo-integration/src/mcp/config_loader.rs:194-201`
  (`to_server_configs` propagates `?` on the first invalid entry — a server
  with neither `command` nor `url` fails the entire file before any
  connection is attempted); `config_loader.rs:109-152` (entry validation);
  stdio transport 120 s response timeout (`transport/stdio.rs:209-211`),
  `McpClient::new` initialize goes through `transport.send` with no extra
  deadline (client.rs:81-91). Contrast: the GUI reconnect path skips invalid
  entries individually and bounds each connect at 15 s
  (mcp.rs:503,526-531).
- Reachability: any `~/.echo-agent/mcp.json` (or `--mcp-config`) containing
  one malformed entry -> boot logs "MCP 配置加载失败" and connects nothing;
  any configured stdio server whose process spawns but never completes the
  initialize handshake -> startup blocks ~120 s (per such server) before the
  warning at infra.rs:1101-1103.
- Expected invariant: invalid config handling isolates the bad entry
  (consistent with the GUI path); startup completes in bounded time even
  with a dead server configured (the GUI path already documents this intent,
  mcp.rs:497-499).
- Observed behavior: all-or-nothing at the file level and unbounded boot
  blocking; the two EKO paths (boot vs GUI save) disagree on the same
  invalid entry.
- Impact: a typo in one server entry silently disables every user-configured
  MCP server for the whole session; a hung server makes the app appear
  frozen at startup (TUI and GUI alike).
- Root cause: `to_server_configs` was written strict-total (fail fast) while
  the per-connect loop is skip-on-error; the boot path was never given the
  timeout the reconnect path received.
- Direction: (a) EKO-side: wrap `load_mcp_from_file` with a per-server
  timeout (reuse `CONNECT_TIMEOUT` semantics) and, (b) framework-side
  (config_loader.rs): make `to_server_configs` skip invalid entries with a
  warning (or add `to_server_configs_lenient`) so one bad entry does not
  disable the file — align with `load_mcp_config`'s skip-on-error contract
  (capabilities.rs:1251-1262); add a boot fixture with one invalid + one
  valid entry asserting the valid one connects.
- Regression validation: EKO boot test: mcp.json with one entry missing
  `command`/`url` and one valid stdio entry -> valid server connects, warn
  logged for the bad one; timeout fixture: a stub stdio server that never
  answers initialize -> boot proceeds within the configured deadline.
- Validation reports: [V03-01](../validations/A-INT-01/V03-01.md),
  [V02-01](../validations/A-INT-01/V02-01.md)

### A-INT-01-P2-03: GUI `connect_mcp_server` dialog rejects loopback/private-range MCP endpoints and requires HTTPS — SSRF-style over-gating that blocks legitimate local MCP servers and contradicts the config-file path, EKO's own network policy, and AGENTS.md's light-validation stance

- Priority: P2
- Confidence: high (behavior); medium (whether the rejection is a defect
  rather than a deliberate choice — the balance of evidence says over-gating)
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/commands/mcp.rs:169-208`
  (`validate_ipc_mcp_url`: https-only + rejects localhost/127.0.0.1/::1/
  169.254.x/10.x/192.168.x/172.16-31.x "to prevent SSRF", comment at
  mcp.rs:166-168); the same server is accepted on the config-file path:
  framework `McpServerEntry::to_server_config` (config_loader.rs:109-152)
  with the module doc example `"url": "http://localhost:8080/mcp"`
  (config_loader.rs:26) and `update_mcp_config` deserializing
  `McpConfigFile` with no URL checks (mcp.rs:481-482); EKO's own network
  policy for web tools allows loopback/private plain HTTP
  (echo-agent-cli/docs/MASTER-PLAN.md:74 "Plain HTTP supports
  loopback/private/link-local IPs, localhost …; remote hosts require HTTPS");
  root MASTER-PLAN:993 flags HTTP/MCP restrictions as "按本地威胁模型审查"
  (pending local-threat-model review); AGENTS.md: user self-extension is
  user-owned, only light validation for obvious errors, no permission-level
  interception.
- Reachability: GUI "add server" dialog -> `connect_mcp_server` (invoked via
  endpoints.ts:206) with a local stdio-independent MCP endpoint such as
  `http://localhost:8100/mcp` or `http://192.168.1.50:8100/mcp` -> hard
  rejection; the identical entry pasted into `~/.echo-agent/mcp.json`
  connects fine. Local MCP servers (docker containers, local tool daemons)
  are a primary use case for a local personal assistant.
- Expected invariant: user-configured MCP endpoints are subject to the
  same rules as the config file and to only light validation (typo-level);
  the local desktop threat model does not include a compromised-page SSRF
  pivot (page JS is trusted, A-TOOL-01 residual note).
- Observed behavior: the dialog blocks the same endpoints the file path
  allows; the rejection is a permission-level interception in validation
  clothing, and the only check of its kind in the codebase (the framework
  fetch/web tools apply their own policy, MASTER-PLAN:74).
- Impact: users cannot connect local/private-network MCP servers from the
  GUI (the natural surface), forcing file editing; inconsistent rules
  between two paths of the same capability. Note: this is NOT a
  `permission_mode` gate — the AGENTS.md hard rule (P1) is not triggered;
  the defect class is over-validation contradicting the documented threat
  model.
- Root cause: the validator was written with a Web-service SSRF mindset
  (comment cites a compromised page forcing authenticated POSTs) and never
  reconciled with the local-assistant threat model or the file path.
- Direction: align `validate_ipc_mcp_url` with MASTER-PLAN:74 (allow
  loopback/private/link-local and `.local`/`.lan` plain HTTP; require HTTPS
  only for remote public hosts) — or, if the https-only rule is kept for
  the dialog, at minimum allow loopback/private hosts and document the
  asymmetry; keep the stdio executable allowlist (typo protection, in line
  with AGENTS.md); update the tests in mcp.rs:591-613 accordingly.
- Regression validation: dialog fixture: `http://localhost:8100/mcp` and
  `http://192.168.1.5:8100/mcp` accepted; `http://evil.example/mcp`
  rejected (obvious-typo rule: plain http to a remote host); config-file
  round-trip unchanged.
- Validation reports: [V03-03](../validations/A-INT-01/V03-03.md),
  [V04-03](../validations/A-INT-01/V04-03.md), [V05-01](../validations/A-INT-01/V05-01.md)

### A-INT-01-P2-04: Dropped user-configured MCP servers never auto-recover — tools stay registered and fail until a manual toggle/config save; the health loop only paints status

- Priority: P2
- Confidence: medium-high (static chain; the "server died" trigger is
  transport-level, F-INT-01)
- Layer: application (recovery policy); transports are framework-owned
- Evidence: `state.rs:750-797` (`run_mcp_health_check` only writes
  `mcp_health` status; healthy = client present && non-empty tools —
  a dead client keeps its tools, so the health flag flips to error but
  nothing reconnects); the only reconnect triggers in echo-agent-cli are
  `update_mcp_config` (mcp.rs:502-553) and browser `call_mcp` restart
  (mod.rs:867-938 — browser-only); framework `McpManager` has no
  auto-reconnect for stdio clients (F-INT-01; SSE has its own reconnect
  loop); agent-level reconnect exists only when a connect is re-requested
  (capabilities.rs:1149-1162).
- Reachability: any user-configured stdio MCP server that crashes or is
  killed (EOF drains pending with -32000 per F-INT-01); every subsequent
  tool call on its `mcp__` tools fails until the user manually reconnects
  via the GUI toggle/config save; the agent keeps advertising the tools to
  the model.
- Expected invariant: MASTER-PLAN:188 ("MCP 重连 … 均有专项测试" — already
  classified stale in F-INT-01 V05) and the task question's "recoverable":
  a dropped server is re-established (bounded retry) or its tools are
  removed/hidden so the model stops calling dead tools.
- Observed behavior: no auto-recovery, no tool removal, no signal other
  than a health flag the agent never reads; the run continues to waste
  calls on dead tools.
- Impact: after a transient local-server crash the MCP capability is
  silently dead for the session; the model keeps attempting and failing —
  the "recoverable" half of the task question is only manually satisfied.
- Root cause: recovery was implemented for the browser sidecar (EKO-owned
  client lifecycle) but never for user-configured servers; the health loop
  was written as telemetry, not control.
- Direction: on unhealthy status (or on repeated tool failures), either
  re-run `connect_mcp_from_config` for that server with the 15 s bound
  (reusing the reconnect helper from mcp.rs) or remove/disable its tools
  and notify; mirror the browser's retry-safe classification so
  `mcp__`-tool failures do not replay ambiguous calls (see F-INT-01-P1-02).
- Regression validation: fixture: connect a stub stdio server, kill its
  process, run one health cycle -> assert the server is reconnected (or its
  tools removed) and the model surface no longer lists dead tools.
- Validation reports: [V03-01](../validations/A-INT-01/V03-01.md),
  [V05-01](../validations/A-INT-01/V05-01.md)

### A-INT-01-P2-05: Browser consequential-action confirmations inherit the REPL provider's EOF auto-approve and the session-wide "*" approve-all — the EKO browser gate is undermined by A-HITL-01-P1-02/P1-03

- Priority: P2 (cross-task; the leaf defect is A-HITL-01-P1-02, the browser
  exposure is new here)
- Confidence: high (chain verified; trigger contexts from A-HITL-01)
- Layer: adapter (browser -> HITL dispatcher) / application
- Evidence: browser default approval provider = `HitlDispatcher`
  (`runtime.rs:144-146`); `confirm_action` picks the per-conversation
  provider or the default (browser/mod.rs:978-992); the dispatcher includes
  the REPL provider registered at bootstrap (`runtime.rs:130-131`;
  GUI per-conversation `TauriHumanLoopHandler` replaces it only per
  conversation, chat.rs:583-589); REPL provider auto-approves on empty/EOF
  stdin and blocks the shared deadline (A-HITL-01-P1-02:
  `hitl/repl_provider.rs:69-77`); "approve all" -> `SessionAllTools` -> "*"
  wildcard (A-HITL-01-P1-03, F-HITL-01-P1-03). Browser risk-gated actions:
  browser/mod.rs:465-522 (`requires_confirmation`, 5-minute request
  timeout) with `effect`/`destination` metadata from the tool schema
  (mod.rs:1536-1576).
- Reachability: (a) piped/scripted CLI run (stdin EOF) whose agent calls a
  consequential browser action (e.g. `browser_click` with
  `effect: "purchase"`/`"send_message"`): confirmation auto-approves;
  (b) GUI window whose conversation has no per-conversation provider yet
  (Finder launch, stdin /dev/null): same EOF auto-approve via the default
  provider; (c) any surface "approve all" (`a`/本会话同意) unlocks ALL
  tools including browser navigation for the session.
- Expected invariant: a consequential browser action (purchase/publish/send)
  without an explicit user response fails closed; the confirmation gate is
  the browser's safety boundary for real-world side effects.
- Observed behavior: the browser confirmation inherits the REPL EOF
  auto-approval and the wildcard approve-all; the browser layer itself
  implements the gate faithfully (risk classification, metadata stripping,
  failure status) but the provider chain defeats it in those contexts.
- Impact: silent approval of consequential web actions (purchases,
  messages) in scripted/EOF contexts — the same class as A-HITL-01-P1-02,
  now with real-world side effects; EKO does not mitigate (it routes
  through the same dispatcher by design).
- Root cause: EKO deliberately routes browser confirmations through the
  shared dispatcher (good single-authority design) whose REPL leaf provider
  has the EOF defect; no fail-closed default for the no-response case.
- Direction: fix A-HITL-01-P1-02 (EOF -> Rejected; async stdin read so the
  shared deadline fires) — the browser path then inherits the fix; consider
  an explicit `Rejected` on provider absence/EOF for browser confirmations
  in the meantime; align "approve all" scoping per A-HITL-01-P1-03.
- Regression validation: dispatcher/browser fixture: closed stdin + pending
  browser confirmation -> `Rejected` and the action fails with "not
  approved"; piped-CLI scenario with a consequential `browser_click` ->
  denied, no auto-approval.
- Validation reports: [V03-03](../validations/A-INT-01/V03-03.md),
  cross-references: [A-HITL-01 V01-01](../A-HITL-01/V01-01.md),
  [A-HITL-01 V02-01](../A-HITL-01/V02-01.md)

### A-INT-01-P3-01: Browser session metadata files accumulate forever — one JSON file per conversation ever created, never deleted, reloaded at every boot

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `browser/session.rs:523-539` (`persist` writes
  `{session_id}.json`, no deletion anywhere: `close_all` (504-521),
  `restore_metadata` (102-158) and `lease_tab` (168-262) only mark/overwrite;
  no TTL/GC loop; `BrowserRuntime::shutdown` (mod.rs:194-204) closes but
  does not delete).
- Reachability: every conversation that uses browser tools; long-lived
  installations with many conversations grow the `browser/sessions/`
  directory without bound (small files, but unbounded count).
- Expected invariant: session metadata is eventually reclaimed (the
  session/close lifecycle is the cleanup policy).
- Observed behavior: files persist indefinitely; restore reloads them all
  and re-persists on the next close.
- Impact: slow disk growth; cosmetic (no functional effect).
- Root cause: persist was written as append-only bookkeeping; no lifecycle
  hook deletes files for closed/abandoned sessions.
- Direction: delete the metadata file when a session is finally closed
  (or GC files older than a retention window at restore); keep the
  closed-status restore for the same-process window.
- Regression validation: session test: lease -> close_all -> file removed
  (or GC run -> old files removed, current closed file kept until window).
- Validation reports: [V03-02](../validations/A-INT-01/V03-02.md)

### A-INT-01-P3-02: LSP tools are registered on the primary agent only — TaskRuntime writer/readonly subagents (which do receive browser tools) cannot use `lsp_diagnostics` and friends

- Priority: P3
- Confidence: high (code fact); medium (intent unknown)
- Layer: application
- Evidence: `runtime.rs:574-589` (`register_lsp_tools` adds the five LSP
  tools to the primary agent only); `infra.rs:951-952,1028-1029` (subagent
  builders install browser tools but no LSP tools — grep of `infra.rs`/
  `agent_pool.rs` for Lsp: zero hits, V01-01); the framework LSP tool
  surface is `echo-agent/src/tools/lsp.rs` (F-INT-02).
- Reachability: TaskRuntime Implementation/Debugging tasks run on writer
  subagents (A-TOOL-01-P1-01 chain); those agents can browse but cannot ask
  the language server for diagnostics/definitions — the diagnostics-driven
  coding loop is unavailable inside task runs.
- Expected invariant: multi-mode/surface functional equality (AGENTS.md);
  a capability present on the primary agent and subagents for one
  integration family (browser) should not silently differ for another
  (LSP).
- Observed behavior: browser tools reach subagents, LSP tools do not;
  nothing documents the asymmetry.
- Impact: task runs lose LSP-assisted analysis; users must copy into a chat
  turn to get diagnostics. Low today (LSP tool usage is niche), but a
  parity gap on a documented capability.
- Root cause: `register_lsp_tools` predates the subagent builder split and
  was never extended to subagent surfaces; no parity check exists.
- Direction: decide deliberately: either share the `LspManager`/tools with
  writer (and readonly) subagents (infra.rs builders, mirroring browser
  install_subagent_tools) or document LSP as primary-agent-only; add a
  registry-diff test asserting the intended per-role LSP surface.
- Regression validation: writer-subagent fixture asserting
  `lsp_diagnostics` is present (after fix) alongside browser tools; existing
  exposure tests stay green.
- Validation reports: [V01-01](../validations/A-INT-01/V01-01.md),
  [V02-01](../validations/A-INT-01/V02-01.md)

## F-INT-01 / F-INT-02 Cross-Reference (EKO exposure)

| Framework finding | EKO-side exposure | Verdict |
|---|---|---|
| F-INT-01-P1-01 (HTTP 202 async path dead, 60 s hangs) | EKO Tauri MCP panel can configure HTTP/SSE servers (`mcp.rs:229-237`) -> user-configured HTTP servers that answer 202 hang every call 60 s; no EKO mitigation (only the stdio browser sidecar is stdio, unaffected) | EKO does not mitigate; exposed via the same dialog that P2-03 constrains |
| F-INT-01-P1-02 (non-idempotent `tools/call` transport retry) | EKO user-configured HTTP MCP tools inherit the pre-classification retry; EKO's browser layer applies its own correct retry-safe split (mod.rs:1031-1086) but only for the browser sidecar | EKO does not mitigate for user servers; browser path is a positive counterexample |
| F-INT-01-P2-03 (MCP cancellation unimplemented) | EKO browser layer observes `ToolContext.cancel` and closes/invalidates the client on cancel (mod.rs:876-883) — better than the framework adapter; user-configured MCP tools still have no cancel propagation | Browser mitigates locally; user servers remain uncovered |
| F-INT-02-P1-01 (LSP requests no timeout; shutdown hang) | EKO registers LSP tools at boot (runtime.rs:574-589) and plugin reload awaits `shutdown_all` (plugin_runtime.rs, F-INT-02) -> a hung language server blocks EKO plugin reload/app shutdown; no EKO-side deadline | EKO does not mitigate; reachable on the live path |
| F-INT-02-P2-03 (lsp_diagnostics false "clean" on stale cache) | Same tool surface exposed on the primary agent of every EKO mode | EKO does not mitigate |
| A-HITL-01-P1-02/P1-03 | Browser confirmations route through the dispatcher/REPL provider -> P2-05 | EKO aggravates by uniform surface choice (same as A-HITL-01 verdict) |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---|---:|
| V01 | Definition and duplicate search (MCP/LSP/browser across both repos, EKO side) | yes | passed | [V01-01](../validations/A-INT-01/V01-01.md) |
| V02 | Registration and runtime reachability (boot MCP, GUI/TUI/CLI commands, browser construction/tool install/shutdown, LSP registration) | yes | passed | [V02-01](../validations/A-INT-01/V02-01.md) |
| V03 | Connect/disconnect/reconnect + invalid config handling | yes | passed | [V03-01](../validations/A-INT-01/V03-01.md) |
| V03 | Session cleanup (browser shutdown/interrupt/close_all, metadata lifecycle, MCP health consistency) | yes | passed | [V03-02](../validations/A-INT-01/V03-02.md) |
| V03 | Default-permission interactive use (no permission-mode gates; validation inventory; browser HITL chain) | yes | passed | [V03-03](../validations/A-INT-01/V03-03.md) |
| V04 | `cargo check -p echo-agent-app-core --locked` | yes | passed (exit 0) | [V04-01](../validations/A-INT-01/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked browser` | yes | passed (exit 0; 34 passed) | [V04-02](../validations/A-INT-01/V04-02.md) |
| V04 | `cargo test -p echo-agent-cli --features gui --lib --locked mcp` | yes | passed (exit 0; 7 passed) | [V04-03](../validations/A-INT-01/V04-03.md) |
| V05 | Historical-document drift (MASTER-PLAN root/EKO, browser-runtime-design, closeout, app-core audit) | yes | passed | [V05-01](../validations/A-INT-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| root MASTER-PLAN:188 "MCP 重连…均有专项测试" | stale | no reconnect/disconnect fixtures anywhere (F-INT-01 V04-02; this review's V04-01..03 add none); EKO recovery manual (P2-04) |
| root MASTER-PLAN:533 browser retry split (read-only retry, click/fill -> partial-side-effect + verify) | current | `browser_mcp_retry_safe`/`browser_failure` (mod.rs:1031-1086, 1057-1086) |
| root MASTER-PLAN:993 "按本地威胁模型审查 HTTP/MCP 限制" | pending/partial | redaction (mcp.rs:402-474) + timeouts (mcp.rs:503) done; private-range rejection (mcp.rs:169-208) is the over-gating this review flags (P2-03) |
| EKO MASTER-PLAN:69 "Model/MCP/runtime topology still requires restart" | current | config watcher does not hot-reload MCP (V02-01) |
| EKO MASTER-PLAN:74 "User-configured MCP tools have no deny-list"; plain HTTP allowed for loopback/private | regressed on the GUI dialog path | framework config path has no deny-list (config_loader.rs); dialog rejects loopback/private + https-only (P2-03) |
| browser-runtime-design.md:95-108 sidecar restart, one restart + one retry, shutdown closes sidecar | current | mod.rs:867-938,185-204; entry-point shutdown (V03-02) |
| app-core-full-audit.md:30 MCP = restart required | current | V02-01 |
| app-core-full-audit.md:249 browser MCP-retry classifier stays in app | current | mod.rs:1031-1086 |

## Coverage And Uncertainty

- Read-only static review: no live MCP/LSP server and no real Playwright
  sidecar were exercised (Q-E2E-01 owns dynamic scenarios). The P1-01
  persistence defect and the CLI stub (P2-01) are pure code facts with full
  chains; P2-02's hang and P2-04's trigger depend on transport behavior
  established in F-INT-01.
- The GUI reconnect background task (mcp.rs:502-553) holds the agent write
  lock for the whole disconnect-all + reconnect loop (up to 15 s x N);
  acknowledged in a comment as acceptable; recorded as residual risk, not a
  finding (bounded, and no IPC caller waits).
- A timed-out `McpClient::new` (GUI reconnect 15 s / browser startup 60 s)
  can abandon an un-initialized npx/stdio child process (no kill-on-drop
  verified in the stdio transport; framework-owned, F-INT-01 P3-04
  territory) — residual uncertainty, not a new finding.
- `update_mcp_config` accepts an arbitrary `McpConfigFile` from the
  frontend (no allowlist/URL checks) while `connect_mcp_server` is strict —
  the "strict path" is bypassable through the config editor; under the
  local threat model this is acceptable (page JS trusted), noted for
  completeness of P2-03's asymmetry analysis.
- Health check `run_mcp_health_check` treats a connected-but-zero-tools
  server as unhealthy ("returned empty tools"); cosmetic status semantics.
- Browser tools are installed on subagents by default (`enabled: true` is
  the `BrowserConfig` default); the plan-mode filter (snapshot.rs:227-236)
  does not strip browser tools (they are not in WRITE_TOOLS), so the
  A-TOOL-01-P1-01 writer defect does not further affect browser reachability.
- Disk note: 32 GiB available / 53 GiB of Cargo targets at review time —
  below the AGENTS.md ~50 GiB cleanup threshold; not cleaned because this
  review is read-only and subsequent tasks need the incremental cache.

## Handoff

- Downstream tasks may rely on: single-authority conclusions (V01-01);
  full reachability table for boot MCP / GUI-TUI-CLI MCP / browser / LSP
  (V02-01); the positive verification that no `permission_mode` gate blocks
  interactive MCP/terminal/browser use (V03-03) — the AGENTS.md P1 hard
  rule is NOT triggered; the four P2 over-gating/recovery findings and the
  P1 persistence defect; browser-side positive patterns (cancel-observed
  MCP calls, retry-safe split) usable as the template for user-configured
  MCP recovery.
- Reports to read: this report, its 9 validation reports, dependency
  reports F-INT-01, F-INT-02, A-TOOL-01, A-HITL-01.
- Stale triggers: changes to `mcp.rs` (update_mcp_config persistence,
  connect validation), `infra.rs:1069-1108`, `state.rs:357,490,750-797`,
  `browser/*` (session lifecycle, retry split, confirm_action provider
  chain), `runtime.rs:110,144-146,499-592`, `cli/cmd_impls/skills.rs:236-274`,
  `config_loader.rs` to_server_configs semantics, `hitl/repl_provider.rs`
  EOF handling.
- Follow-up task IDs: X-AUT-01 (permission-boundary classification — P2-03
  is the over-gating item), X-SRF-01 (CLI /mcp stub row, LSP-subagent row,
  GUI config persistence row), Q-E2E-01 (dynamic browser/MCP/LSP scenarios),
  S-RDM-01 (roadmap ordering: P1-01 first, then P2-01..05). Fixes are
  deferred to the iteration roadmap; this review is read-only.
