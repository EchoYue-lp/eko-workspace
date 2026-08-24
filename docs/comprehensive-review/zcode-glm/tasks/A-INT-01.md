# A-INT-01: Browser, MCP, and LSP application integration

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: not-applicable (cross-references framework `echo-agent` commit `9b0e0fa` only for inherited MCP/LSP behavior already filed under F-INT-01 / F-INT-02; no framework modification)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Are local browser sessions and user-configured MCP/LSP capabilities reachable,
recoverable, and not blocked by irrelevant permission gates?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/browser/mod.rs` — `BrowserRuntime`
  lifecycle (start / ensure_client / invalidate_client / call_mcp retry /
  interrupt / shutdown / extension_status).
- `echo-agent-cli/echo-agent-app-core/src/browser/session.rs` —
  `BrowserSessionManager` (per-conversation tabs, persist/restore, close_all).
- `echo-agent-cli/echo-agent-app-core/src/browser/sidecar.rs` —
  `BrowserSidecar::prepare` + `server_config` (Playwright MCP launcher).
- `echo-agent-cli/echo-agent-app-core/src/browser/config.rs` — `BrowserConfig`
  (env-driven, allowed/blocked domain policy, default `enabled=true`).
- `echo-agent-cli/echo-agent-app-core/src/config_discovery.rs` — `mcp.json`
  + `.lsp.yaml` discovery (global `~/.eko/` + project root).
- `echo-agent-cli/echo-agent-app-core/src/infra.rs:1069-1108` — `load_mcp_config`
  startup loader (`~/.eko/mcp.json` only, no CWD); `:1111-1130`
  `spawn_mcp_health_check` (30 s loop).
- `echo-agent-cli/echo-agent-app-core/src/runtime.rs:499-592` —
  `register_lsp_tools` (discover + global + nearest project `.lsp.yaml`,
  per-language `start_server`, 15 s init timeout).
- `echo-agent-cli/echo-agent-app-core/src/state.rs:340-450, 751-797` —
  `PluginState.mcp_config` / `mcp_health` fields and `run_mcp_health_check`.
- `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:560-720,
  946-1000, 1156-1176` — LSP restart-on-reload, `shutdown_all` rollback
  paths, MCP-component unload on plugin reload.
- `echo-agent-cli/src/tauri/commands/mcp.rs` — `connect_mcp_server`,
  `disconnect_mcp_server`, `toggle_mcp_server`, `update_mcp_config`,
  `get_mcp_config` (with `redact_mcp_config_secrets`), and the
  `validate_ipc_mcp_stdio` / `validate_ipc_mcp_url` input gates.
- `echo-agent-cli/src/tauri/commands/browser.rs` — interactive browser
  commands (`browser_navigate`, `browser_stop`, `browser_set_backend`, etc.).
- `echo-agent-cli/src/tauri/desktop.rs:124-271` — GUI bootstrap and the
  explicit `runtime.browser_runtime.shutdown().await` on Tauri window close.
- `echo-agent-cli/src/main.rs:155-445` — TUI/CLI/channels bootstrap and the
  matching browser shutdown call.
- `echo-agent-cli/src/tui/events.rs:3425-3477, 4552+` — TUI `/mcp` slash
  command (list / load / disconnect) and browser runtime reachability.
- Framework anchors cross-referenced (not re-audited):
  `echo-agent/src/agent/react/capabilities.rs:1149-1332`
  (`connect_mcp_from_config` reconnect-on-same-name, `disconnect_mcp`),
  `echo-agent/echo-integration/src/mcp/config_loader.rs:107-261`
  (`McpServerEntry::to_server_config`, on-disk `validate_stdio_command`),
  `echo-agent/echo-integration/src/lsp/manager.rs:60-108`
  (`start_server` / `stop_server` / `restart_server`).

## Out Of Scope

Deferred to downstream/other tasks:

- **F-INT-01** (zcode-glm): framework MCP transport correctness — HTTP 202
  async-path hang (F-INT-01-P1-01), SSE clean-close termination
  (F-INT-01-P2-01), SSE retry-budget monotonicity (F-INT-01-P2-02),
  tool-call cancellation no-op (F-INT-01-P2-03). This task consumes those
  conclusions; it does not re-audit the transport internals.
- **F-INT-02** (zcode-glm): framework LSP / channel / A2A lifecycle —
  `restart_count` / `last_error` dead fields (F-INT-02-P2-01), missing
  per-request timeout (F-INT-02-P2-02), reader/writer task not cancellable
  (F-INT-02-P3-02). This task treats them as inherited limitations.
- **A-TOOL-01** (zcode-glm): per-mode tool exposure matrix and the
  interactive-terminal-vs-`run_code` separation. This task relies on its
  conclusion that browser/MCP/LSP tools are registered once on the primary
  agent and surface through `tool_search`.
- **A-BOOT-01 / B-PATH-01**: TUI parity gaps (no TUI LSP panel, no TUI
  browser pane) are owned there. This task only flags integration
  reachability gaps that compound those parity gaps.
- **A-FE-01**: Tauri command DTO ↔ frontend type parity for the MCP panel
  and browser panel. This task audits the Rust-side command surface only.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` — local-assistant threat model (no online /
  multi-user gating); "MCP is user-configured, don't over-gate with
  permissions"; "保留对明显错误输入的轻量校验即可,不要做权限级拦截";
  the historical `require_full_auto` lesson on `create_terminal` /
  `connect_mcp_server`; multi-mode parity rule; no-duplicate / single-
  authority rule; UTF-8 / no-panic hard rules; code-cleanup rule.
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`, `docs/comprehensive-review/REPORTING.md`.
- `docs/comprehensive-review/TASKS.md` (A-INT-01 card and F-INT-01 / F-INT-02
  / A-TOOL-01 dependencies).

Dependency task reports read:

- `zcode-glm/tasks/F-INT-01.md` (complete) — framework MCP integration.
  Relied on for: `McpToolAdapter` correctness; `ToolFailure::from_error`
  classification; transport-level cancellation / reconnect defects
  (F-INT-01-P1-01 / -P2-01 / -P2-02 / -P2-03 / -P3-02) inherited by the
  application layer.
- `zcode-glm/tasks/F-INT-02.md` (complete) — framework LSP / channels /
  A2A. Relied on for: `LspManager::restart_server` exists but is framework-
  only; `StdioLspClient::restart_count` / `last_error` are dead fields
  (F-INT-02-P2-01); per-request timeout missing (F-INT-02-P2-02); reader /
  writer tasks have no `CancellationToken` (F-INT-02-P3-02).
- `zcode-glm/tasks/A-TOOL-01.md` (complete) — tool exposure and the
  interactive-terminal permission path. Relied on for: browser/MCP/LSP
  tools are registered once on the primary agent via
  `browser_runtime.install_tools` and surface through `tool_search`;
  `create_terminal` is ungated, matching the local-assistant rule.

Historical documents treated as hypotheses:

- AGENTS.md "历史教训 ... `require_full_auto` 门控已移除" — re-verified for
  `connect_mcp_server` / `disconnect_mcp_server` and the browser commands.
  See Historical Claim Status.
- AGENTS.md "保留对明显错误输入 ... 的轻量校验即可,不要做权限级拦截" —
  re-verified for `validate_ipc_mcp_stdio` (executable allowlist) and
  `validate_ipc_mcp_url` (private-range / loopback rejection). See finding
  A-INT-01-P1-01.

## Layering Decision

This is an **application-layer** task. All browser session state, MCP IPC
commands, MCP panel config DTO, LSP startup wiring, and the GUI bootstrap
shutdown sequence live in `echo-agent-cli` / `echo-agent-app-core` (EKO
product). The framework contributes only generic primitives that any
consumer needs:

- **Generic mechanism (framework, retained):** `McpClient` lifecycle,
  `McpServerConfig` / `McpConfigFile` parsers, `connect_mcp_from_config` /
  `disconnect_mcp` / `load_mcp_from_file` agent seam
  (`capabilities.rs:1149-1332`), `StdioLspClient` / `LspManager` /
  `LspConfig::discover` + `from_file`. All pure protocol / discovery code;
  any `echo-agent` consumer needs them. Correctly placed in
  `echo-integration`.
- **EKO product policy (application):** the GUI MCP panel (`tauri/commands/
  mcp.rs`), the Playwright MCP sidecar selection (`browser/sidecar.rs`),
  the per-conversation browser session/tab model (`browser/session.rs`),
  the env-driven `BrowserConfig` (`browser/config.rs`), the startup
  config discovery (`config_discovery.rs` + `infra.rs:load_mcp_config`),
  the LSP startup wiring (`runtime.rs:register_lsp_tools`), the health
  check loop (`infra.rs:spawn_mcp_health_check`).
- **Adapter boundary:** `tauri/commands/mcp.rs` is a thin IPC adapter —
  `connect_mcp_server` validates input, converts `McpTransportConfig` to
  `McpServerConfig`, delegates to `agent.connect_mcp_from_config`. It owns
  no scheduler, no second registry. `BrowserRuntime::call_mcp`
  (`browser/mod.rs:867`) is a thin adapter around `McpClient::call_tool`
  that adds one retry and risk-aware non-replay — the framework's
  `McpClient` remains the single transport authority.

Duplicate-search terms run across both repositories:

- `connect_mcp_from_config` / `disconnect_mcp` / `load_mcp_from_file` /
  `load_mcp_config` — single definitions in framework
  (`capabilities.rs:1149 / 1241 / 1251 / 1315`); only consumed via
  `echo_agent::prelude::*` in the application.
- `BrowserRuntime` / `BrowserSessionManager` / `BrowserSidecar` — single
  definitions in `echo-agent-app-core/src/browser/`; the framework has no
  browser concept.
- `register_lsp_tools` — single definition (`runtime.rs:499`); called once
  during `AgentRuntime::bootstrap`.
- `validate_ipc_mcp_stdio` / `validate_ipc_mcp_url` (IPC gate) vs
  `validate_stdio_command` (framework on-disk gate,
  `config_loader.rs:229`) — distinct validators at different trust
  boundaries; both kept (the framework's is a denylist, the application's
  is an allowlist+SSRF). See A-INT-01-P1-01 for whether the IPC variant's
  strictness is appropriate.
- `restart_server` (LSP) — single definition in framework (`manager.rs:105`)
  with **zero application callers** (see A-INT-01-P2-02).

No parallel implementation of MCP connect/disconnect, browser session
management, or LSP startup wiring was found. TUI and GUI share
`AgentRuntime::bootstrap` for all three integrations.

## Current Path

Verified data flow at `echo-agent-cli` commit `b3b2e81`, cross-referenced
against framework `echo-agent` commit `9b0e0fa`:

### 1. MCP ingestion

- **Startup (file path, TUI + GUI + CLI):** `infra::load_mcp_config`
  (`infra.rs:1069-1108`) resolves the config path in priority CLI override
  → YAML `mcp.config_path` → `MCP_CONFIG_PATH` env → `~/.eko/mcp.json`
  (only the user-data path; CWD is intentionally excluded to prevent
  repository injection per the comment at `infra.rs:1090`). The primary
  agent's `agent.load_mcp_from_file(&path).await` calls
  `McpConfigFile::from_file` + `to_server_configs` +
  per-server `connect_mcp_from_config`; per-server failures are logged
  and skipped (`capabilities.rs:1259-1269`).
- **GUI IPC (live reconfigure):** `update_mcp_config` (`mcp.rs:477-559`)
  deserializes the new config via `serde_json::from_value` (returns
  `IpcError::Validation` on JSON error), persists it synchronously into
  `PluginState.mcp_config`, returns immediately, then spawns a background
  task that holds the agent write lock, disconnects every existing server,
  and reconnects each enabled entry with a 15 s per-server timeout
  (`mcp.rs:502-553`). `connect_mcp_from_config` itself disconnects any
  same-named client first (`capabilities.rs:1157-1159`), so an explicit
  disconnect-then-reconnect sequence is idempotent.
- **GUI IPC (single server):** `connect_mcp_server` (`mcp.rs:211-258`)
  runs `validate_ipc_mcp_stdio` or `validate_ipc_mcp_url` (see finding
  A-INT-01-P1-01), then constructs `McpServerConfig::stdio_with_env` /
  `http_with_headers` / `sse_with_headers` and delegates to
  `connect_mcp_from_config`. Always returns `Ok(json!({"success": bool,
  ...}))` — never propagates connect errors as `Err`.
- **TUI slash command:** `/mcp` (`tui/events.rs:3425-3477`) supports
  `list` / `load <path>` / `disconnect <name>`. `load` calls
  `agent.load_mcp_from_file(path)` directly — no IPC allowlist, so any
  on-disk config is accepted.

### 2. Browser ingestion

- **Bootstrap:** `AgentRuntime::bootstrap` constructs `BrowserRuntime::start`
  once (via `infra::create_agent_with_diagnostics` at `infra.rs:455`,
  installing browser tools on the primary agent only when
  `config.enabled || config.extension_enabled`). On `start`, if
  `config.enabled`, a background task prewarms the Managed Playwright MCP
  sidecar (`mod.rs:89-101`) — failures only warn; tools retry lazily.
- **Per-call MCP path:** `BrowserRuntime::call_mcp` (`mod.rs:867-938`)
  resolves the cached `Arc<McpClient>` via `ensure_client` (double-checked
  with a per-backend `Mutex` connect lock); calls `McpClient::call_tool`;
  on `Err` and the call is `browser_mcp_retry_safe` (read-only / reversible
  action), it invalidates the client, closes it, re-`ensure_client`s, and
  retries exactly once. Non-retry-safe failures invalidate and propagate
  immediately.
- **Backend selection:** each conversation has a `BrowserBackend`
  (Managed Playwright vs Chrome extension), stored in
  `BrowserSessionManager`. `browser_set_backend` IPC + the `Backend`
  BrowserAction are the only switch paths; both route through
  `switch_backend` (`session.rs:386-419`), which resets tab indices.
- **Reconnect (Chrome extension):** the first Chrome-side `browser_tabs
  list` call (inside `BrowserAction::Backend`) probes the extension. If it
  fails, the error is stored in `extension_startup_error` and surfaced via
  `chrome_setup_status` (`browser.rs:138-142`). The extension connection is
  otherwise lazy — it is only established when a Chrome backend action
  runs.

### 3. LSP ingestion

- **Startup only:** `register_lsp_tools` (`runtime.rs:499-592`) is the
  sole LSP wiring point. It (a) `LspConfig::discover(&project_root)` from
  PATH + project markers; (b) merges `~/.eko/.lsp.yaml` (global) and the
  nearest `.lsp.yaml` walking up from the project root; (c) constructs one
  `LspManager`, calls `start_server` per discovered language (each bounded
  by a 15 s init timeout at `manager.rs:76-82`); (d) registers the five
  framework LSP tools (`LspDiagnosticsTool` / `LspGotoDefinitionTool` /
  `LspFindReferencesTool` / `LspHoverTool` / `LspStatusTool`) on the
  primary agent; (e) returns a `PluginLspRuntime` shared by
  `PluginRuntimeService`.
- **Plugin reload (atomic LSP swap):** `PluginRuntimeService::reload`
  (`plugin_runtime.rs:560-720`) prepares a replacement LspManager, rolls
  back / forward on activation errors, and calls `shutdown_all().await` on
  the previous manager. The agent's LSP tools hold `Arc<RwLock<LspManager>>`
  so the swap is transparent to in-flight tool calls.
- **No live reconfigure:** unlike MCP, LSP has no IPC command and no TUI
  slash command to add / remove / restart a single language server after
  startup. The framework exposes `LspManager::restart_server`
  (`manager.rs:105-108`) but no application caller invokes it (grep
  confirms zero hits in `echo-agent-cli`).

### 4. Shutdown sequence

- **Browser (explicit, all surfaces):** `main.rs:334 / 399 / 443` (TUI /
  channels / CLI) and `desktop.rs:267` (GUI) all call
  `runtime.browser_runtime.shutdown().await`. `shutdown` (`mod.rs:194-204`)
  cancels the runtime token, `sessions.close_all().await`, then takes and
  closes the McpClient for each backend (managed + extension). This is the
  only integration with explicit graceful shutdown.
- **MCP (best-effort Drop):** no application-level shutdown call. On
  process exit, `McpManager` and its `McpClient`s are dropped. Framework
  `StdioTransport::Drop` (`echo-integration/src/mcp/transport/stdio.rs:
  270-282`) `tokio::spawn`s a kill — best-effort, fails silently if the
  runtime is already shutting down (F-INT-01-P3-02). SSE / HTTP transports
  have similar drop-only cleanup.
- **LSP (best-effort Drop):** no application-level shutdown call. The
  `Arc<RwLock<LspManager>>` held by `PluginRuntimeService` drops on
  `drop(runtime)`. `StdioLspClient` was constructed with
  `kill_on_drop(true)` (`echo-integration/src/lsp/client.rs:79`), so the
  OS reaps the child — but no graceful LSP `shutdown` request is sent and
  reader / writer tasks are not aborted.

## Findings

### A-INT-01-P1-01: IPC MCP URL validation rejects legitimate local servers; conflicts with AGENTS.md no-over-gating rule and with the on-disk config path

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/mcp.rs:169-208` `validate_ipc_mcp_url`
    rejects any URL whose host matches `localhost`, `127.0.0.1`, `::1`,
    `169.254.*`, `10.*`, `192.168.*`, `172.16.*..172.31.*` (lines 188-200),
    in addition to requiring `https://`.
  - `echo-agent-cli/src/tauri/commands/mcp.rs:117-160`
    `validate_ipc_mcp_stdio` rejects any command whose base-name is not in
    `ALLOWED_MCP_STDIO_BASES = [npx, node, uvx, uv, python, python3, pipx,
    docker, java]` (line 140).
  - The doc comment at `mcp.rs:162-168` explicitly invokes the
    "SSRF pivot where a compromised page forces the app to issue
    authenticated POSTs to internal services" threat model, and
    `mcp.rs:110-119` invokes the "any XSS would then be a one-hop RCE"
    threat model.
  - In contrast, the on-disk config path (`McpConfigFile::from_file` →
    `McpServerEntry::to_server_config` → `validate_stdio_command` at
    `echo-integration/src/mcp/config_loader.rs:229-261`) only blocks shell
    metacharacters, a small denylist of dangerous commands, and path
    traversal. It does **not** restrict the executable to an allowlist and
    does **not** reject loopback / private-range URLs.
  - The framework's own `config_loader.rs:27, 32` documentation examples
    use `"url": "http://localhost:8080/mcp"` and `"url":
    "http://localhost:3000"` — URLs that the IPC path refuses.
- Reachability: any GUI user opening the MCP panel and trying to add a
  server whose URL is `https://localhost:8100/mcp` (a very common local
  dev port for an MCP server) or whose stdio command is
  `/usr/local/bin/my-custom-mcp` is rejected with `IpcError::Validation`.
  The same user can put identical content in `~/.eko/mcp.json` and the
  startup loader accepts it. The defect is reachable on every MCP panel
  save.
- Expected invariant (AGENTS.md "产品定位与安全边界"): "EKO 是本地个人超级
  智能助理,运行在用户自己的机器上,不部署到线上,不存在多用户 / 公网攻击
  场景." and "不要套用线上 Web 服务的威胁模型:诸如'防 XSS→RCE''防 SSRF
  内网穿透'...这类线上服务的安全闸,**默认不适用于 EKO**." and "保留对
  **明显错误输入**(命令名拼错、URL 用了明文 http)的轻量校验即可,不要做
  权限级拦截."
- Observed behavior: the IPC gate applies an executable allowlist and a
  private-range URL blocklist — both are permission-level interceptions
  justified by an online-service XSS/SSRF threat model that AGENTS.md
  explicitly excludes for EKO. The on-disk path applies only lightweight
  typo / shell-injection guards. The two paths are inconsistent.
- Impact: GUI users cannot configure the same MCP servers that the file
  path accepts. Two concrete failure modes: (a) any locally-served MCP
  server (custom dev server, language-tool wrapper, anything on
  `http://localhost:PORT`) cannot be added through the GUI panel; (b) any
  non-allowlisted binary (a user's own `my-mcp-server` binary, a Go/Rust
  MCP server installed outside the npx/python/docker ecosystem) cannot be
  added through the GUI panel. This is the same class of regression as
  the historical `require_full_auto` gate — over-gating that makes a
  user-configured capability unreachable in default configuration.
- Root cause: the validator was added under an online / multi-user threat
  model (XSS-RCE, SSRF) that AGENTS.md has since codified as not
  applicable to EKO. The historical lesson ("require_full_auto gate on
  connect_mcp_server removed") was applied to the `permission_mode` gate
  but a parallel allowlist/SSRF gate was added in the same file without
  re-evaluating against the local-assistant rule.
- Direction: align `validate_ipc_mcp_stdio` and `validate_ipc_mcp_url`
  with the on-disk `validate_stdio_command` discipline. Specifically:
  (a) drop the executable allowlist — keep only the denylist of dangerous
  commands + shell-metacharacter + path-traversal guards (the user takes
  responsibility for the binary they configure, per AGENTS.md "用户自扩展
  的能力由用户自己负责"); (b) drop the loopback / private-range rejection
  — keep only the `https://` requirement for non-`localhost` URLs (a
  user connecting to `http://example.com` over plain HTTP is the typo-
  class case the rule explicitly allows catching; a user connecting to
  `http://localhost:8080` knows what they are doing). If a residual
  XSS-RCE concern remains for the IPC path specifically, document why
  EKO's local-only threat model still warrants it (per AGENTS.md "何时该
  加防护 ... 默认不加权限门控;要加必须在注释里写明'本地场景下为何仍需
  要'"). Update the existing comment at `mcp.rs:110-119, 162-168` to drop
  the XSS/SSRF framing.
- Regression validation: extend the `validate_ipc_mcp_*` unit tests
  (`mcp.rs:565-613`) to assert that `http://localhost:8080/mcp` and
  `/usr/local/bin/my-custom-mcp` are accepted (or, if the team decides to
  keep a narrower gate, document the residual rule with a comment that
  cites AGENTS.md).
- Validation reports: [V04-01](../validations/A-INT-01/V04-01.md)

### A-INT-01-P2-01: No graceful MCP / LSP shutdown on app exit; relies on best-effort Drop while Browser has explicit shutdown

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/main.rs:334, 399, 443` and
    `src/tauri/desktop.rs:267` — the only explicit integration-shutdown
    call is `runtime.browser_runtime.shutdown().await`. There is no
    equivalent `runtime.mcp_shutdown().await` or `runtime.lsp_shutdown
    ().await`.
  - `echo-agent-cli/echo-agent-app-core/src/runtime.rs:31-52` `AgentRuntime`
    has no `impl Drop` and no shutdown method that calls
    `LspManager::shutdown_all` or iterates `agent.list_mcp_servers()` to
    disconnect each.
  - Framework `StdioTransport::Drop`
    (`echo-integration/src/mcp/transport/stdio.rs:270-282`) attempts
    `tokio::spawn(child.kill())` — best-effort, silently fails if the
    runtime has already shut down (filed as F-INT-01-P3-02).
  - Framework `StdioLspClient` uses `kill_on_drop(true)`
    (`echo-integration/src/lsp/client.rs:79`); no graceful LSP `shutdown`
    request is sent on drop. `LspManager::shutdown_all`
    (`echo-integration/src/lsp/manager.rs` — exists, but no application
    caller on the exit path).
  - `PluginRuntimeService` is held inside `Arc<...>` and dropped when
    `runtime` is dropped (`main.rs:444`). `PluginLspRuntime`'s
    `Arc<RwLock<LspManager>>` decrements its refcount but has no `Drop`
    that calls `shutdown_all`.
- Reachability: every EKO process exit (TUI quit, GUI window close, CLI
  completion, panic-induced exit). The browser path shuts down cleanly;
  the MCP and LSP paths do not.
- Expected invariant: a graceful shutdown should at minimum call the
  framework's documented close paths (`McpClient::close` for each
  connected server; `LspManager::shutdown_all`) so the child processes
  receive their intended termination sequence and the framework can free
  resources deterministically. Browser already does this.
- Observed behavior: MCP stdio subprocesses are killed via `Drop`-spawned
  tasks that may not run if the tokio runtime is shutting down; LSP
  subprocesses get `kill_on_drop` SIGKILL with no `shutdown` request.
  Orphan subprocesses and stale `diagnostics_cache` / pending request
  maps are reclaimed only by OS-level process exit.
- Impact: in the local single-user scenario (AGENTS.md) the blast radius
  is bounded — process exit eventually reaps everything. But: (a)
  inconsistent with the browser path, which suggests the cleanup
  discipline is incomplete rather than intentional; (b) `kill_on_drop`
  on Unix is SIGKILL, which skips the LSP server's own shutdown hooks
  (some servers flush caches / unlock files on graceful shutdown); (c)
  on a panic-induced exit where the runtime is partially alive, the
  MCP `tokio::spawn` Drop path can leak the subprocess until OS reap.
- Root cause: the shutdown sequence was wired for the browser (which
  owns a multi-subprocess Playwright MCP runtime that needs explicit
  teardown) but never extended to cover MCP and LSP, which were assumed
  to be "framework-managed" — but the framework's Drop impls are
  best-effort and the framework explicitly exposes async `close` /
  `shutdown_all` for the application to call.
- Direction: add an `async fn shutdown(&self)` on `AgentRuntime` (or
  extend the existing exit block) that (a) iterates
  `agent.list_mcp_servers()` and calls `agent.disconnect_mcp(name).await`
  for each (or更低层 `McpClient::close`); (b) calls
  `plugin_runtime.lsp.manager.write().await.shutdown_all().await`.
  Call it before `browser_runtime.shutdown()` in `main.rs:334, 399, 443`
  and `desktop.rs:267`. Per AGENTS.md code-cleanup, do not add a parallel
  `Drop` impl on `AgentRuntime` — `Drop` cannot `.await` and would
  reintroduce the same best-effort problem.
- Regression validation: a test that constructs an `AgentRuntime` with a
  mock MCP client and a mock LSP child, calls the new `shutdown`, and
  asserts both were closed / shut down (mock's `close` / `shutdown`
  counters incremented). Also a smoke test that exits the CLI/TUI/GUI
  and asserts no orphan subprocesses (best-effort, platform-dependent).
- Validation reports: [V02-01](../validations/A-INT-01/V02-01.md)

### A-INT-01-P2-02: Framework `LspManager::restart_server` has no application caller; users cannot recover a crashed / hung LSP server without restarting EKO

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent/echo-integration/src/lsp/manager.rs:105-108` defines
    `pub async fn restart_server(&mut self, language: &str) -> Result<(),
    String>` as stop-then-start.
  - `grep -rn "restart_server\|restart_lsp\|lsp_restart" echo-agent-cli/`
    returns **zero** application callers.
  - `echo-agent-cli/echo-agent-app-core/src/runtime.rs:499-592`
    `register_lsp_tools` runs only at bootstrap; no per-language restart
    hook is exposed.
  - `echo-agent-cli/src/tui/events.rs:3425-3477` exposes a `/mcp` slash
    command with `list / load / disconnect`, but there is no `/lsp`
    equivalent.
  - `echo-agent-cli/src/tauri/commands/` contains `mcp.rs` (full CRUD)
    but no `lsp.rs` — there is no IPC command to start / stop / restart
    an LSP server.
  - This compounds F-INT-02-P2-01 (no auto-restart on crash —
    `restart_count` / `last_error` are dead fields) and F-INT-02-P2-02
    (no per-request timeout — one hung request permanently blocks the
    client's `pending` map).
- Reachability: any LSP server that crashes (segfault, OOM, native
    assertion) or hangs (deadlock, infinite loop). After the first
    incident the corresponding LSP tools (`LspGotoDefinitionTool`, etc.)
    silently no-op or hang for every subsequent call until the user
    restarts the entire EKO process.
- Expected invariant: a long-running local assistant should let the user
  recover a single integration (here: one LSP language) without restarting
  the whole agent — parity with the MCP panel, which exposes per-server
  connect / disconnect / toggle. Multi-mode parity (AGENTS.md): if GUI
  has a panel, TUI should have an equivalent slash command, and vice
  versa.
- Observed behavior: the framework's restart primitive exists but is
  unreachable from any application surface. Recovery requires app
  restart.
- Impact: usability regression. A user debugging a flaky `rust-analyzer`
  or `pyright` has no in-app "restart this language server" affordance.
  Combined with F-INT-02-P2-02 (no per-request timeout), the only
  recovery today is full EKO restart. Marked P2 (not P1) because LSP is
  a developer convenience, not on the chat critical path.
- Root cause: the application never wired the framework's restart
  primitive to any surface. The MCP side got a full panel; the LSP side
  got only startup discovery.
- Direction: add a thin restart surface that reuses the existing
  framework primitive. Minimum: a Tauri command `restart_lsp_server(
  language: String)` and a TUI `/lsp restart <lang>` slash command,
  both delegating to `plugin_runtime.lsp.manager.write().await.
  restart_server(&language).await`. Optionally also `list_lsp_servers`
  / `stop_lsp_server` for parity with MCP. No framework change needed.
- Regression validation: a unit test that constructs an `LspManager`
  with a mock client, calls restart, and asserts stop+start counters
  increment. A smoke test that starts EKO, kills the spawned
  `rust-analyzer` from outside, invokes the new restart command, and
  asserts the next `LspGotoDefinitionTool` call succeeds.
- Validation reports: [V01-01](../validations/A-INT-01/V01-01.md)

### A-INT-01-P3-01: `disconnect_mcp_server` IPC always returns success even when the named server was not connected

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/mcp.rs:261-286` `disconnect_mcp_server`
    calls `agent.disconnect_mcp(&name).await` and discards the returned
    `bool` (the framework's `disconnect_mcp` returns `false` when no
    client matches the name — `capabilities.rs:1315-1332`).
  - The command unconditionally returns
    `Ok(json!({"success": true, "message": "Disconnected from MCP server
    '{name}'"}))` regardless of whether anything was disconnected.
  - `toggle_mcp_server` (`mcp.rs:293-374`) has a parallel inconsistency:
    `enabled=true` on a missing name returns `IpcError::NotFound`, but
    `enabled=false` on a missing name silently returns success (the
    `if let Some(entry)` branch falls through at `mcp.rs:301`).
- Reachability: every GUI call to disconnect a server that already
  dropped, crashed, or was never configured under that name.
- Expected invariant: an explicit user action on a non-existent resource
  should at minimum surface "not found" so the UI does not show a stale
  "disconnected" success.
- Observed behavior: `disconnect_mcp_server('never-existed')` returns
  the same success payload as disconnecting a live server. The MCP panel
  sees `success: true` and re-renders as if a state change occurred.
- Impact: minor — no data loss, no security consequence. Pure UX
  misleading-success. The local single-user scenario (AGENTS.md) does
  not require hard error semantics here.
- Root cause: the bool return contract from the framework seam was
  ignored when wrapping it in an IPC command.
- Direction: thread the framework's `disconnect_mcp` bool into the IPC
  response — on `false`, return either `IpcError::NotFound` or a
  `{"success": false, "message": "MCP server '{name}' was not
  connected"}` payload (mirroring `connect_mcp_server`'s soft-error
  shape at `mcp.rs:253-257`). Apply the same fix to `toggle_mcp_server`'s
  disable branch.
- Regression validation: extend `mcp.rs:561-613` test module with a test
  that calls `disconnect_mcp_server` against a name not in the agent and
  asserts the response carries `success: false` (or `IpcError::NotFound`).
- Validation reports: [V01-01](../validations/A-INT-01/V01-01.md)

### A-INT-01-P3-02: Browser `interrupt()` does not cancel in-flight agent tool calls; it tears down and silently rebuilds the sidecar on the next call

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/browser.rs:132-135` `browser_stop`
    calls `state.browser_runtime.interrupt().await`.
  - `echo-agent-cli/echo-agent-app-core/src/browser/mod.rs:185-192`
    `interrupt()` takes and closes the `McpClient` for each backend —
    but does not cancel any in-flight `call_mcp` future. The agent's
    `BrowserAction` tool execution continues to hold its `Arc<McpClient>`
    local.
  - `call_mcp` (`mod.rs:867-938`): on the next `McpClient::call_tool`
    error after `interrupt()`, the retry path calls `invalidate_client`
    (which is now a no-op because the slot is already empty) and
    `ensure_client` (which spawns a fresh sidecar). The original
    "stopped" action's result is then produced from the new sidecar.
- Reachability: a user clicking the GUI "Stop" button while an agent
  browser action is in flight (e.g., a long `browser_form_fill` on a
  slow page).
- Expected invariant: "Stop" should stop the in-flight action — either
  cancel the calling future (via the framework `CancellationToken`
  threaded through `ToolContext.cancel`) or at minimum not silently
  re-spawn the sidecar to complete the very action the user wanted to
  abort.
- Observed behavior: `browser_stop` closes the MCP client but the agent's
  in-flight `call_mcp` either (a) gets an error and falls into the retry
  path, which spawns a new sidecar and completes the action; or (b) the
  cancellation token in `ToolContext.cancel` (if any) is the only thing
  that actually stops the future (see `mod.rs:876-884`).
- Impact: low — the user's intent (stop the browser action) is not
  honored unless the chat turn itself was cancelled. The rebuild
  silently consumes sidecar startup time (up to `startup_timeout_secs`
  = 60 s default per `browser/config.rs:119`). In the local single-user
  scenario (AGENTS.md) the blast radius is bounded.
- Root cause: `interrupt()` predates the `ToolContext.cancel` threading
  and was intended for session-end teardown, not for mid-action cancel.
  The GUI wired it to a "Stop" button without adding cancel propagation.
- Direction: either (a) rename `interrupt()` to `teardown()` and add a
  separate `cancel_active(conversation_id)` that signals the per-
  conversation cancellation token, wiring `browser_stop` to the latter;
  or (b) document in `browser_stop`'s command doc-comment that "Stop"
  only stops future actions, not the currently in-flight one, and
  surface the chat-turn cancel as the way to abort an active action.
- Regression validation: a test that starts a `browser_navigate` action
  against a mock MCP server, calls `interrupt()` mid-flight, and asserts
  either (a) the action future resolves as `BrowserError::Cancelled`
  (preferred) or (b) the sidecar is not silently rebuilt (current
  behavior — document as known limitation).
- Validation reports: [V01-01](../validations/A-INT-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Connect / disconnect / reconnect: MCP IPC + framework reconnect, browser MCP retry-once, LSP start/stop (no restart caller). | yes | passed (with P2/P3 gaps) | [V01-01](../validations/A-INT-01/V01-01.md) |
| V02 | Session cleanup on shutdown: browser explicit, MCP/LSP best-effort Drop. | yes | failed | [V02-01](../validations/A-INT-01/V02-01.md) |
| V03 | Invalid config handling: mcp.json serde / .lsp.yaml YAML / missing binary — graceful startup skip. | yes | passed | [V03-01](../validations/A-INT-01/V03-01.md) |
| V04 | Default-permission interactive use: no permission_mode gate; IPC over-validates local URLs. | yes | failed | [V04-01](../validations/A-INT-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | — |

V05 is not applicable: this is the first A-INT-01 report in this
reviewer's directory; no prior report exists to compare against. The
historical claims audited here come from `AGENTS.md` itself, classified
under "Historical Claim Status" below.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md "MCP is user-configured, don't over-gate with permissions" | current (core statement) | `connect_mcp_server` (`mcp.rs:211`) does not gate on `permission_mode`; `disconnect_mcp_server`, `toggle_mcp_server`, `update_mcp_config` likewise. Browser commands (`browser.rs`) and the LSP startup path (`runtime.rs:499`) carry no permission gate. TUI `/mcp` slash command (`tui/events.rs:3425`) is ungated. The core statement holds. |
| AGENTS.md historical lesson: "require_full_auto gate on connect_mcp_server removed" | current (literal) but parallel over-gating re-introduced | The literal `require_full_auto` / permission-mode gate is gone. However `validate_ipc_mcp_stdio` (allowlist) and `validate_ipc_mcp_url` (private-range block) in the same file re-introduce permission-level interception under an online XSS/SSRF threat model — see A-INT-01-P1-01. |
| AGENTS.md "保留对明显错误输入的轻量校验即可,不要做权限级拦截" | regressed (in IPC path) | On-disk `validate_stdio_command` (`config_loader.rs:229`) is correctly lightweight (denylist + metachar + traversal). IPC `validate_ipc_mcp_stdio` / `validate_ipc_mcp_url` (`mcp.rs:127, 169`) go beyond lightweight typo-catching into allowlist + private-range rejection. See A-INT-01-P1-01. |
| AGENTS.md "TUI 与 GUI 是功能完全一样的 Agent 完全体" | regressed (for LSP interactive surface) | GUI has an MCP panel but no LSP panel; TUI has `/mcp` but no `/lsp`; `restart_server` exists in the framework with zero application callers (A-INT-01-P2-02). MCP and LSP interactive surfaces are asymmetric. |
| F-INT-01 conclusion: "McpToolAdapter is a correct, thin implementation of the framework Tool contract" | current | Application's `BrowserRuntime::call_mcp` builds on `McpClient::call_tool` faithfully; `McpTransportConfig` → `McpServerConfig` conversion is lossless. |
| F-INT-02 conclusion: "framework LSP / channels / A2A integrations correctly isolate their external protocols" | current | Application consumes the LSP integration through the documented `LspManager` / `LspConfig` API; no parallel implementation. The application gaps (no restart caller, no graceful shutdown) are layered on top, not protocol-level defects. |
| A-TOOL-01 conclusion: "browser/MCP/LSP tools are registered once on the primary agent and surface through tool_search" | current | Confirmed: `infra.rs:455 browser_runtime.install_tools`, `infra.rs:515 register_task_tools` (unrelated), `runtime.rs:582-586 Lsp*Tool::new`. All framework-tool registrations are mode-agnostic and surface via the visibility filter (A-TOOL-01). |

## Coverage And Uncertainty

**Code not inspected:**

- The frontend TypeScript MCP panel and browser panel rendering
  (A-FE-01 / A-FE-02 scope). This task audits the Rust command surface
  and the IPC contract only.
- The `McpHealthStatus` aggregation beyond `run_mcp_health_check`
  (`state.rs:751-797`) — read for the 30 s loop, not for the diagnostic
  semantics (out of scope; A-OBS-01).
- `BrowserConfig::allows_url` (`browser/config.rs:80-103`) — read for
  the allow/block domain policy but not audited for SSRF / typo edge
  cases; the browser URL policy is a navigation guard, not an MCP
  connection gate, and is correctly product-layer.
- The Chrome-extension install/setup wizard beyond
  `chrome_setup_status` / `chrome_open_extensions_page` — out of scope
  (product UX, not integration correctness).
- The plugin-component MCP unload path (`plugin_runtime.rs:1156-1176`)
  was inspected only for the disconnect call shape; full plugin reload
  rollback semantics belong to A-PLG-01.

**Validations not available:**

- No executable end-to-end test against a real MCP server, LSP server,
  or Playwright sidecar was run (would require spawning fixture
  processes). V01-V04 are static analyses; the findings rest on code
  reading plus the unit tests already present in the codebase
  (`mcp.rs:561-613`, `browser/sidecar.rs:108-167`,
  `browser/session.rs:542-755`, `lsp/config.rs:307-405`).
- The IPC over-validation blast radius (A-INT-01-P1-01) was confirmed
  by reading the test suite (`mcp.rs:591-613` asserts `https://localhost`
  is rejected and `https://127.0.0.1` is rejected) — i.e., the existing
  tests *lock in* the over-gating. A regression-test rewrite is part of
  the fix direction.

**Claims that remain uncertain:**

- A-INT-01-P3-02 (browser `interrupt()` rebuild behavior) is medium
  confidence because the in-flight `call_mcp` behavior depends on
  whether the calling agent turn supplied a `CancellationToken` in
  `ToolContext.cancel` — the chat path does (`chat_driver.rs:480-490`)
  but the IPC `browser_*` commands pass `None` (`browser.rs:17
  execute(... None)`). Without a token, the retry path indeed silently
  rebuilds. With a token, the cancel branch (`mod.rs:876-884`) would
  fire first. Marked medium because the dominant path (direct GUI
  button) currently passes `None`.

## Handoff

**Conclusions downstream tasks may rely on:**

- All three integrations (browser / MCP / LSP) are reachable from every
  surface (TUI / GUI / CLI) at the integration level: MCP via
  `~/.eko/mcp.json` + GUI panel + TUI `/mcp`; LSP via PATH-based
  discovery + `.lsp.yaml`; browser via the Playwright MCP sidecar
  installed on the primary agent. Downstream X-SRF-01 / X-BND-01 can
  treat reachability as confirmed for the registration layer; the gap
  is interactive-surface parity (A-INT-01-P2-02).
- Browser is the only integration with a correct, explicit graceful
  shutdown sequence (`BrowserRuntime::shutdown`). MCP and LSP rely on
  best-effort framework Drop impls. Downstream X-STA-01 / A-OBS-01
  citing "shutdown cleanly reclaims resources" should qualify the claim
  to browser-only until A-INT-01-P2-01 is resolved.
- The MCP IPC contract (`connect_mcp_server` / `disconnect_mcp_server` /
  `toggle_mcp_server` / `update_mcp_config` / `get_mcp_config`) is the
  authoritative surface for the GUI MCP panel. A-FE-01 can rely on the
  DTO shapes and the `redact_mcp_config_secrets` redaction
  (`mcp.rs:402-474`) for frontend type parity.
- The on-disk MCP config (`McpConfigFile` /
  `McpServerEntry::to_server_config` /
  `validate_stdio_command`) and the IPC MCP validators
  (`validate_ipc_mcp_stdio` / `validate_ipc_mcp_url`) are intentionally
  asymmetric today. X-BND-01 / X-INV-01 auditing capability placement
  can cite this as a known semantic duplicate (two validators at
  different trust boundaries) tracked under A-INT-01-P1-01.

**Reports they must read:**

- This report + [V01-01](../validations/A-INT-01/V01-01.md),
  [V02-01](../validations/A-INT-01/V02-01.md),
  [V03-01](../validations/A-INT-01/V03-01.md),
  [V04-01](../validations/A-INT-01/V04-01.md).
- `zcode-glm/tasks/F-INT-01.md` for the framework MCP transport defects
  inherited by the application (HTTP 202 hang, SSE clean-close, retry
  budget, cancellation).
- `zcode-glm/tasks/F-INT-02.md` for the framework LSP gaps
  (`restart_count` / `last_error` dead fields, per-request timeout,
  task cancellation) that compound A-INT-01-P2-02.
- `zcode-glm/tasks/A-TOOL-01.md` for the per-mode tool-exposure matrix
  that gates which integration tools reach the model per
  `InteractionMode`.

**Conditions that make this report stale:**

- Any change to `tauri/commands/mcp.rs:110-208` (the IPC validators) —
  A-INT-01-P1-01 is tightly anchored to the current allowlist /
  private-range logic.
- Any addition of an `AgentRuntime::shutdown` (or equivalent) that
  calls `LspManager::shutdown_all` and iterates `disconnect_mcp` —
  would resolve A-INT-01-P2-01.
- Any addition of an `/lsp` slash command, `lsp.rs` Tauri command
  module, or other caller of `LspManager::restart_server` — would
  resolve A-INT-01-P2-02.
- Any rewrite of `BrowserRuntime::interrupt` to propagate cancellation
  to in-flight tool calls — would resolve A-INT-01-P3-02.

**Follow-up task IDs (no fixes implemented in this review):**

- A-INT-01-P1-01 (IPC over-validation) — isolated to
  `tauri/commands/mcp.rs`, low regression risk if the test suite is
  updated in the same patch; P1 because it makes a user-configured
  capability unreachable in the default GUI.
- A-INT-01-P2-01 (no graceful MCP/LSP shutdown) — bundles cleanly with
  any future shutdown-ordering work in `main.rs` / `desktop.rs`.
- A-INT-01-P2-02 (no LSP restart surface) — should be chartered
  together with F-INT-02-P2-01 / -P2-02 (restart-tracking dead fields
  + per-request timeout) since they all touch the same LSP resilience
  story; bundling avoids two passes over `lsp/manager.rs` and the
  application LSP wiring.
- A-INT-01-P3-01 / -P3-02 — localized UX/correctness cleanups; bundle
  into a maintenance task.
