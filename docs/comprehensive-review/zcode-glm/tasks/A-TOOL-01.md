# A-TOOL-01: Tool exposure, execution, sandbox, and terminal

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: not-applicable (cross-references `echo-agent` for V04 truncation path; framework not modified)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Does each Agent/mode expose the intended tools with common error and artifact
behavior, while keeping interactive terminal separate from Agent `run_code`
policy?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/tool_exposure.rs` (full, 366 lines) —
  `groups_for_mode`, `initial_visible_tools`, `disabled_tools_for_mode`,
  `rollout_for_mode`, `record_mode_schema_budget`, per-mode snapshot tests.
- `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs` (460-485, 600-650) —
  invocation-scoped `visible_tools` / `disabled_tools` wiring for the chat path.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs`
  (3070-3130, 3670-3720) — Task-run primary-agent path and Auto/scope subagent
  path tool exposure.
- `echo-agent-cli/echo-agent-app-core/src/infra.rs` (1-525, 533-543, 881-1042,
  2095-2180) — `create_agent_with_diagnostics` composition root,
  `configure_run_code_capability`, `build_readonly_subagent_agent`,
  `build_writer_subagent_agent`, sandbox/run_code tests.
- `echo-agent-cli/echo-agent-app-core/src/state.rs` (240-341, 480-534) —
  `SandboxTier`, `SandboxConfigData`, `tool_executions` repository field.
- `echo-agent-cli/src/tauri/terminal.rs` (full, 420 lines) — PTY terminal,
  `create_terminal`, `write_terminal`, `confirm_terminal_consent`.
- `echo-agent-cli/src/tauri/commands/panels.rs` (810-920) — `get_sandbox_status`,
  `get_sandbox_config`, `update_sandbox_config`, `execute_sandbox`.
- `echo-agent-cli/src/tauri/commands/chat.rs` (960-1180) — runtime-event →
  `ToolExecutionRepository` observer bridge, `cancel_active_tools`.
- `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs` (1-460, 579-810) —
  `ToolExecutionRepository`: `start` / `append_output` / `finish` / `cancel` /
  `read_output`, chunk/page limits.
- `echo-agent/src/agent/snapshot.rs` (185-266, 920-1070) — framework
  `ToolRuntime::from_agent` visible/disabled composition, `process_tool_output`
  truncation + artifact spill.
- `echo-agent/src/agent/react/mod.rs` (740-746) — `readonly_tools` gating of
  `register_readonly_tools` vs `register_all_tools`.
- `echo-agent/echo-execution/src/sandbox/manager.rs` (1-60, 220-260) —
  `SandboxManager::local_sandbox`, `has_local_os_sandbox`.

## Out Of Scope

Deferred to downstream/other tasks:

- **F-EXT-01 / F-EXT-02**: framework `Tool` / `ToolResult` / `ToolFailure`
  contract correctness and individual shell/file/code/git tool semantics. This
  task consumes their conclusions; it does not re-audit tool internals.
- **F-SEC-01**: the `SandboxExecutor` trait and concrete sandbox backends
  (local/docker/k8s). This task only verifies how EKO selects and gates the
  sandbox for `run_code`, not the sandbox internals.
- **A-BOOT-01**: entry-point composition. This task consumes its conclusion
  that `create_agent_with_diagnostics` is the single agent composition root.
- **A-INP-01 / A-INT-01**: MCP/LSP/browser integration and prepared-input
  artifacts.
- **A-FE-02**: frontend projection of tool executions (reducer identity,
  out-of-order events). This task audits the Rust-side repository and observer
  only.
- Per-tool permission prompts (HITL approval flow) — owned by the HITL review
  track.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` — local-assistant threat model (no online
  multi-user gating), "interactive terminal/file picker/MCP are USER actions,
  not agent automation; don't gate with full-auto/default permission";
  "run_code sandbox is agent automation"; multi-mode parity rule; no-duplicate
  / single-authority rule; UTF-8 / no-panic hard rules; the `create_terminal` /
  `connect_mcp_server` `require_full_auto` historical lesson.
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`, `docs/comprehensive-review/REPORTING.md`.
- `docs/comprehensive-review/TASKS.md` (A-TOOL-01 card and F-EXT-01/02 +
  A-BOOT-01 dependencies).

Dependency reports read:

- `zcode-glm/tasks/F-EXT-01.md` (complete) — `Tool` / `ToolResult` /
  `ToolFailure` contract is the single typed tool surface; `ToolFailure`
  carries `category` + `recovery`.
- `zcode-glm/tasks/F-EXT-02.md` (complete) — `ShellTool` and `RunCodeTool`
  delegate to the sandbox when configured; framework file/shell/code tool
  semantics.
- `zcode-glm/tasks/A-BOOT-01.md` (complete) — `create_agent_with_diagnostics`
  is the single composition root; multi-mode parity gaps (TUI missing
  interactive terminal pane, channels missing services) are boot-lifecycle.

Historical documents treated as hypotheses:

- AGENTS.md "历史教训 ... require_full_auto 门控已移除" — treated as a claim to
  re-verify at `create_terminal` (see Historical Claim Status).

## Layering Decision

This is an **application-layer** task. All tool-exposure policy (per-mode
visible/disabled sets), the interactive PTY terminal, the `SandboxConfigData`
panel state, and the `ToolExecutionRepository` projection live in
`echo-agent-cli` / `echo-agent-app-core` (EKO product). The framework
contributes only generic primitives that any consumer needs:

- **Generic mechanism (framework, retained):** `ToolRuntime::from_agent`
  invocation-scoped visible+disabled filter composition
  (`echo-agent/src/agent/snapshot.rs:185`), `process_tool_output_for_call`
  truncation + sha256 artifact spill (`snapshot.rs:926`), `readonly_tools`
  builder gate (`react/mod.rs:742`), `SandboxManager` + `has_local_os_sandbox`
  probe (`echo-execution/src/sandbox/manager.rs`). These are correct framework
  capabilities and are not EKO-specific.
- **EKO product policy (application):** the per-mode tool groups
  (`tool_exposure.rs:57-113`), the disabled legacy-tool set
  (`tool_exposure.rs:137-169`), the PTY consent+audit gate
  (`tauri/terminal.rs:300`), the `SandboxConfigData` panel DTO (`state.rs:261`),
  the `ToolExecutionRepository` durable projection (`tool_execution.rs`).
- **Adapter boundary:** the runtime-event observer in
  `tauri/commands/chat.rs:960-1180` is a thin projection adapter: it converts
  framework `RuntimeEventKind` into repository `start/append_output/finish/cancel`
  calls. It holds no scheduling authority, no ready-frontier, no second
  cancellation path — framework `CancellationToken` (threaded via
  `ExternalRunContext.cancel`) remains the single execution-cancellation
  authority, and the repository's `cancel()` only records projection state.

Duplicate-search terms run across both repositories:

- `initial_visible_tools` / `disabled_tools_for_mode` / `groups_for_mode` —
  single definition in `tool_exposure.rs`; consumed at three call sites only
  (chat_driver, executor Task path, executor Auto path).
- `configure_run_code_capability` — single definition; called for main agent
  and writer subagent only.
- `SandboxManager::local_sandbox` — single framework definition; the only
  construction site in the application is `infra.rs:265` (plus two test sites).
- `security_level` / `SandboxTier` — defined once; **zero read sites other than
  its own default** (see finding A-TOOL-01-P2-01).
- `create_terminal` / `write_terminal` / `confirm_terminal_consent` — single
  definition in `tauri/terminal.rs`; no second terminal implementation in TUI
  (see Coverage And Uncertainty).
- `ToolExecutionRepository` — single definition; one field on `AppState.storage`
  (`state.rs:366`), consumed only by Tauri commands and the observer bridge.

No parallel implementation of per-mode tool exposure, terminal, or tool-exec
projection was found. The legacy background-task tools and `agent_tool` are
registered on the framework but hard-hidden by `disabled_tools_for_mode`
(correct thin-adapter pattern, not a duplicate authority).

## Current Path

### Tool registration (single registry, invocation-scoped filtering)

`infra::create_agent_with_diagnostics` (`infra.rs:194`) builds one `ReactAgent`
via `ReactAgentBuilder`. Tool registration is framework-driven and mode-agnostic:

- `.enable_tools()` (`infra.rs:280`) → `register_all_tools` (or
  `register_readonly_tools` when `.readonly_tools()` is set, see subagents
  below).
- `.register_agent_dispatch_tool()` (`infra.rs:288`) → ad-hoc `agent_tool`
  (Phase-0 transitional; hidden in all modes by `disabled_tools_for_mode`,
  visible-set intersection, or both).
- `browser_runtime.install_tools(&mut agent)` (`infra.rs:455`) when a browser
  runtime is present.
- `echo_agent::tasks::register_task_tools` (`infra.rs:515`) when a
  `TaskRuntimeStore` is supplied → authoritative `task_create` /
  `task_update` / `task_list`.

EKO does **not** re-register tools per mode. Instead, each turn resolves a mode
at the chat/run boundary and passes two invocation-scoped filters through
`AgentInvocationContext`:

- `visible_tools: Option<HashSet<String>>` — the first-turn surface.
- `disabled_tools: Option<HashSet<String>>` — a hard blocklist.

The framework composes them in `ToolRuntime::from_agent`
(`echo-agent/src/agent/snapshot.rs:185`):

1. `disabled_tools` accumulates agent-level + profile-excluded + invocation
   disabled (`snapshot.rs:192-204`).
2. `visibility` is built from `available` (all registered minus disabled, minus
   write tools in plan mode) ∩ `eligible` (skill-allowed) ∩ `initial`
   (invocation visible) (`snapshot.rs:224-254`). `tool_search` is force-inserted
   into the initial visible set (`snapshot.rs:250`).
3. `tools_for_llm` filters by both disabled and visibility
   (`snapshot.rs:269-279`).

So a tool reaches the model iff it is registered AND not disabled AND visible.
The three filters compose correctly.

### Per-mode exposure policy (`tool_exposure.rs`)

`groups_for_mode` (`tool_exposure.rs:76`) selects a first-turn visible group per
`InteractionMode`. `disabled_tools_for_mode` (`tool_exposure.rs:137`) hard-hides
legacy/parallel tools. `initial_visible_tools` (`tool_exposure.rs:118`)
intersects the policy group with the agent's actually-registered names
(defensive — a missing registration is silently dropped, not an error).

Authoritative per-mode matrix (verified by the snapshot tests at
`tool_exposure.rs:206-274`):

| Mode | Visible (groups) | Notable excludes from visible | Extra disabled (besides legacy bg-task set) |
|---|---|---|---|
| Chat | control, file, directory, exec (`shell`), code (`run_code`), task, web_search | `diff`, `web_fetch`, memory tools, `read_skill_resource` | `create_complex_task`, `check_run_status`, `cancel_run` |
| Task | control, file, exec, code, task, skill_resource, web_search, `diff` | `list_dir`(!), memory tools, `web_fetch` | `agent_tool`, `create_complex_task`, `check_run_status`, `cancel_run` |
| Auto | control, file, directory, exec, task, web_search, `web_fetch`, `diff`, memory | `run_code`(!), `read_skill_resource` | `agent_tool` |

All three modes keep the authoritative task-graph tools
(`task_create/update/list/execute`) visible and not disabled
(`tool_exposure.rs:350-365`), and all three hard-disable the framework's
parallel background-task tools (`spawn_background_task`,
`check_task_status`, `list_background_tasks`) so the LLM cannot use the
non-authoritative path. This is the correct single-authority enforcement.

The three live call sites all thread the user- or run-resolved mode through the
same helpers:

- Chat: `chat_driver.rs:465-472` (user-selected `interaction_mode`).
- TaskRun primary agent: `executor.rs:3084-3109` (forced
  `InteractionMode::Task`).
- Auto / scope subagent: `executor.rs:3686-3710` (forced
  `InteractionMode::Auto`, plus `unattended_direct_disabled_tools`).

### Per-role (subagent) tool diff

Subagent role diff is enforced at **registration** time, not via the visibility
filter:

- `build_readonly_subagent_agent` (`infra.rs:968`) sets `.readonly_tools()`
  (`infra.rs:990`) → framework calls `register_readonly_tools`
  (`react/mod.rs:743`): no shell, no write tools, no `run_code`. Readonly
  subagents get **no** `sandbox_manager` and do not call
  `configure_run_code_capability` (physically cannot run code).
- `build_writer_subagent_agent` (`infra.rs:881`) omits `.readonly_tools()` →
  full tool set; takes `sandbox_manager`; calls
  `configure_run_code_capability(&mut subagent, run_code_available)`
  (`infra.rs:950`).
- `configure_run_code_capability` (`infra.rs:533`) is the single no-bare
  gate: if `run_code_available == false`, it removes `run_code` from the agent
  (verified by `unavailable_os_sandbox_removes_run_code` at `infra.rs:2106`).
- `agent_tool` is registered on subagents only when `can_delegate`
  (`infra.rs:925-931`, `1003-1009`); the writer-subagent test at `infra.rs:2145`
  confirms `agent_tool` is absent when `can_delegate=false`.

### Sandbox selection and no-bare fallback (V02)

- The agent's `SandboxManager` is built exactly once, at
  `infra.rs:265`: `SandboxManager::local_sandbox()`. This uses
  `LocalConfig::default()` + `SandboxPolicy::local_os()` (or
  `SandboxPolicy::local_process()` on Windows) with `allow_fallback: false`
  (`echo-execution/src/sandbox/manager.rs:229-240`).
- Probe: `run_code_available = sandbox_manager.has_local_os_sandbox().await`
  (`infra.rs:269`), which requires
  `local.isolation_level() >= OsSandbox && local.is_available()`
  (`echo-execution/src/sandbox/manager.rs:259`).
- If unavailable: `configure_run_code_capability(&mut agent, false)` removes
  `run_code` (`infra.rs:533`); the agent keeps `shell` (which also routes
  through the sandbox) but `run_code` is not left registered in a degraded
  form. There is **no bare-unsandboxed fallback** for `run_code`. This matches
  AGENTS.md (no over-gating of local tools, but also no silent safety
  downgrade of the agent-automation path).

### Interactive terminal (V03) — separate from `run_code`

The only interactive terminal in EKO is the GUI PTY in
`echo-agent-cli/src/tauri/terminal.rs`. It uses `portable_pty` and a per-session
`PtySession`; it shares no state with the agent's `SandboxManager` or the
`run_code` tool. Confirmed separation:

- `create_terminal` (`terminal.rs:278`) — **no permission gate**. The comment
  explicitly states "interactive developer tool, not an agent-auto path... no
  multi-user/online threat model warrants a permission gate here"
  (`terminal.rs:286-289`). This matches AGENTS.md's local-assistant rule.
- `write_terminal` (`terminal.rs:300`) — gated by a **per-session user consent
  flag** (`confirm_terminal_consent`, `terminal.rs:366`), a 64 KiB payload cap,
  and an audit log (`terminal.rs:316-355`). The gate is an XSS-shell-injection
  guard, not an agent `full-auto`/`permission_mode` gate; the comments
  (`terminal.rs:310-329`) are explicit about this distinction.
- No path connects the PTY to the agent's `run_code` execution; the PTY spawns
  `$SHELL` directly (`terminal.rs:108-114`), the agent sandbox spawns through
  `SandboxManager`. They are independent execution channels.

TUI has no embedded interactive terminal pane (its "terminal" references in
`tui/mod.rs` are ratatui/crossterm UI plumbing, not a shell pane). This is a
multi-mode parity gap already filed by A-BOOT-01 / B-PATH-01; it is not re-filed
here.

### Tool output bounding, cancel, and error (V04)

Large output is bounded at two independent layers:

1. **Model-facing** (`echo-agent/src/agent/snapshot.rs:926`
   `process_tool_output_for_call`): if estimated tokens exceed
   `max_tool_output_tokens` OR raw bytes exceed the artifact threshold, the full
   output is spilled to a sha256-tagged artifact file and the model receives a
   short preview + a `read_artifact` pointer (`snapshot.rs:967-984`). If the
   spill write fails, a head+tail token truncation is used with an 8000-token
   fallback (`snapshot.rs:991-1064`, `TOOL_OUTPUT_SPILL_FAILURE_FALLBACK_TOKENS`
   at `snapshot.rs:21`). Truncation uses `chars().take()` (UTF-8 safe). EKO's
   `DEFAULT_MAX_TOOL_OUTPUT_TOKENS` is 4000 (`infra.rs:30-31`, aliased to
   `MAX_MODEL_VISIBLE_TOOL_RESULT_TOKENS`); artifact threshold 32 KiB, max age
   30 days (`infra.rs:32-33`).
2. **GUI-facing** (`tool_execution.rs`): the durable projection stores output
   as 8 KiB JSONL chunks (`STORED_OUTPUT_CHUNK_BYTES`, `tool_execution.rs:29`);
   `read_output(cursor, limit)` paginates at `DEFAULT_DETAIL_PAGE_BYTES = 64
   KiB` (`tool_execution.rs:413-450`), and the artifact path is also paged
   (`read_artifact_page`, `tool_execution.rs:723`). No unbounded output is
   loaded into the frontend or model context.

Cancellation: the observer in `tauri/commands/chat.rs:1116`
(`cancel_active_tools`) cascades a `Cancelled` status to active tool executions
when the run emits `Completed` / `Failed` / `Cancelled` / `TimedOut`
(`chat.rs:1106-1111`). The actual in-flight cancellation is performed by the
framework's `CancellationToken`, threaded through `ExternalRunContext.cancel`
(seen in the invocation contexts at `chat_driver.rs:480-490` and
`executor.rs:3094-3104`). The repository's `cancel()`
(`tool_execution.rs:360`) only records projection state; it does not claim a
second execution-cancellation authority.

Error: `ToolFailure` (category + recovery, per F-EXT-01) is persisted by
`finish(success, result, failure, ...)` (`tool_execution.rs:293`) into the
manifest's `failure` field and surfaced via `detail_manifest`
(`tool_execution.rs:406`). Runtime-event observer failure decoding at
`chat.rs:1075-1078`.

## Findings

### A-TOOL-01-P2-01: `SandboxConfigData.security_level` (Low/Medium/High) is never consulted by the agent's sandbox; the GUI tier selector is cosmetic for `run_code`

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/state.rs:262` —
    `pub security_level: SandboxTier` (Low/Medium/High, default Medium).
  - `echo-agent-cli/echo-agent-app-core/src/state.rs:271` — only write site is
    the default initializer.
  - `echo-agent-cli/src/tauri/commands/panels.rs:840-857` — `get_sandbox_config`
    and `update_sandbox_config` read/write the whole `SandboxConfigData`
    (including `security_level`) but never propagate it to the agent.
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:265` — the agent's
    `SandboxManager` is built via `SandboxManager::local_sandbox()` with
    `LocalConfig::default()` and does NOT consult `SandboxConfigData`.
  - `echo-agent-cli/src/tauri/commands/panels.rs:905-918` — `execute_sandbox`
    consumes only `max_memory_mb` / `max_cpu_seconds` / `network_enabled`;
    `security_level` is not read.
- Reachability: GUI panel exposes the Low/Medium/High selector; user change
  flows through `update_sandbox_config` → writes the in-memory `RwLock`. The
  agent's `run_code` sandbox is built once at `create_agent_with_diagnostics`
  and never re-reads this field. Verified by repository-wide grep: zero
  `.security_level` read sites outside the struct definition and default.
- Expected invariant: a user-visible sandbox "security level" selector should
  affect the sandbox applied to agent `run_code` execution, or be removed.
- Observed behavior: changing Low↔Medium↔High has no effect on the agent's
  `run_code`. The agent always uses `SandboxPolicy::local_os()`
  (`SandboxManager::local_sandbox()`). `security_level` is serialized to the
  frontend and back but is otherwise dead. The numeric limits
  (`max_memory_mb` / `max_cpu_seconds` / `network_enabled`) ARE consumed, but
  only by the manual `execute_sandbox` panel command — not by `run_code`,
  which uses the sandbox manager's own `ResourceLimits`.
- Impact: misleading UX / config surface — a user who raises the tier to "High"
  believing they are hardening agent code execution gets no change. Also a
  layering wart: product-owned panel state duplicates a tier concept that the
  framework's `SandboxPolicy` already owns. Not a security regression (the
  default `local_os` policy is already OS-sandboxed; AGENTS.md does not require
  a tier knob at all), but a real "config that doesn't do what it says"
  defect.
- Root cause: the panel DTO predates, or was never wired into, the agent
  composition root; `security_level` was added as a UI field without a
  corresponding read in `create_agent_with_diagnostics` or the sandbox command
  builder.
- Direction: pick one — (a) remove `security_level` from `SandboxConfigData`
  and the GUI selector (preferred under YAGNI: AGENTS.md does not require a
  tier knob, and `local_os` is already the safe default), keeping only the
  numeric limits that `execute_sandbox` actually uses; or (b) if a tier is
  desired, map `SandboxTier` → `SandboxPolicy` in
  `create_agent_with_diagnostics` and document the mapping. Either way,
  eliminate the dead field. Note this is NOT a reason to add a permission gate
  — per AGENTS.md the local assistant does not over-gate `run_code`; the issue
  is config-vs-effect honesty, not gating.
- Regression validation: a test that builds the agent, reads
  `agent.sandbox_manager().policy`, and asserts it equals the policy derived
  from `SandboxConfigData` (or, under option (a), a test that
  `SandboxConfigData` no longer has `security_level` and the GUI no longer
  renders the selector).
- Validation reports: [V02](../validations/A-TOOL-01/V02-01.md)

### A-TOOL-01-P3-01: `Auto` interaction mode exposes `shell` but not `run_code`; Chat/Task expose both — asymmetric and apparently unintentional

- Priority: P3
- Confidence: low
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tool_exposure.rs:96-106` — the
    `AUTO` group array includes `EXECUTION_TOOLS` (`shell`) but omits
    `CODE_EXECUTION_TOOLS` (`run_code`).
  - `echo-agent-cli/echo-agent-app-core/src/tool_exposure.rs:77-95` — `CHAT`
    and `TASK` both include `CODE_EXECUTION_TOOLS`.
  - `echo-agent-cli/echo-agent-app-core/src/tool_exposure.rs:250-274` — the
    snapshot test locks `Auto` visible set without `run_code`.
- Reachability: every Auto-mode (unattended long-running) turn. The Auto path
  is entered via `executor.rs:3699` (`initial_visible_tools(InteractionMode::Auto,
  ...)`).
- Expected invariant: code-execution surface should be consistent across modes
  unless there is a documented product reason for the difference. `run_code`
  and `shell` both route through the same `SandboxManager`; excluding
  `run_code` from Auto does not change the security posture (shell can still
  run `python script.py`), only the structured code-execution surface.
- Observed behavior: an Auto-mode agent loses the structured `run_code` tool
  and must shell out for code execution. Chat and Task keep both. No comment
  documents why Auto drops `run_code`.
- Impact: low. No correctness or security defect (same sandbox either way);
  the only consequence is a slightly worse tool surface for unattended
  code-heavy work (the agent cannot benefit from `run_code`'s structured
  language/script handling).
- Root cause: appears to be an oversight in the AUTO group definition rather
  than a deliberate product decision (no comment, no test asserting the
  asymmetry is intended). Cannot fully confirm intent from code alone — hence
  low confidence.
- Direction: confirm whether Auto should exclude `run_code`. If not, add
  `CODE_EXECUTION_TOOLS` to the AUTO group and update the snapshot test. If
  yes, add a one-line comment explaining why (e.g. "Auto prefers shell for
  unattended scripted work").
- Regression validation: re-run `mode_exposure_snapshots_are_stable`
  (`tool_exposure.rs:206`) after the change; schema-budget test
  (`tool_exposure.rs:292`) must still pass.
- Validation reports: [V01](../validations/A-TOOL-01/V01-01.md)

### A-TOOL-01-P3-02: TUI has no interactive terminal pane (parity gap imported from A-BOOT-01)

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tui/mod.rs` — all "terminal" references are
    ratatui/crossterm UI plumbing (`enable_raw_mode`, `AlternateScreen`,
    `TerminalGuard`); grep for `pty`/`portable_pty`/`create_terminal` in
    `src/tui/` returns zero hits.
  - `echo-agent-cli/src/tauri/terminal.rs:278` — interactive PTY terminal is
    GUI-only.
- Reachability: every TUI session.
- Expected invariant: per AGENTS.md "TUI 与 GUI 是功能完全一样的 Agent 完全体,
  只是交互方式不同", the interactive terminal (a user-action developer tool)
  should be reachable in both surfaces, or the gap should be tracked.
- Observed behavior: GUI users get an embedded PTY terminal; TUI users do not.
- Impact: low for this task's question (the terminal is correctly separate
  from `run_code` wherever it exists). The parity gap itself is a
  boot-lifecycle/surface-parity concern owned by A-BOOT-01 / B-PATH-01.
- Root cause: the interactive terminal was implemented as a Tauri command
  only; no TUI widget was added.
- Direction: NOT a fix target for this review task — re-filed here only as a
  cross-reference. The single-authority terminal implementation is
  `tauri/terminal.rs`; a future TUI terminal should reuse the same consent +
  audit semantics, not a parallel implementation.
- Regression validation: N/A (no code change in this task).
- Validation reports: [V03](../validations/A-TOOL-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Per-role/mode tool registry diff: one registry + invocation-scoped visible/disabled filters compose correctly; subagent role diff enforced at registration. | yes | passed | [V01-01](../validations/A-TOOL-01/V01-01.md) |
| V02 | Sandbox tier selection and no-bare fallback: `local_sandbox()` + `has_local_os_sandbox` probe + `configure_run_code_capability` removes `run_code` when unavailable; `security_level` disconnected. | yes | passed | [V02-01](../validations/A-TOOL-01/V02-01.md) |
| V03 | Interactive terminal permission path: GUI PTY is separate from agent `run_code`; no `full_auto`/permission_mode gate on `create_terminal`; per-session consent+audit on writes. | yes | passed | [V03-01](../validations/A-TOOL-01/V03-01.md) |
| V04 | Large output / cancel / error: two-layer output bounding (model spill+truncate, GUI paginated JSONL); cancel recorded as projection while framework `CancellationToken` is the execution authority; `ToolFailure` persisted. | yes | passed | [V04-01](../validations/A-TOOL-01/V04-01.md) |

No `V05` (historical-document drift) report: the only historical claim audited
(the AGENTS.md "`require_full_auto` gates were removed from `create_terminal`"
lesson) is verified inside V03 and classified in the table below, not as a
separate execution.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md "历史教训:曾有一批安全 commit 给 `create_terminal` / `connect_mcp_server` 加了 `require_full_auto` 门控...这类门控已移除" | current (fixed) | `create_terminal` (`tauri/terminal.rs:278`) has no `require_full_auto` / permission-mode gate; comment at `terminal.rs:286-289` documents the local-assistant rationale. |
| AGENTS.md "Interactive terminal/file picker/MCP are USER actions, not agent automation. Don't gate with full-auto/default permission." | current | `create_terminal` ungated; `write_terminal` gated only by per-session consent + size cap + audit (injection guard, not permission mode). See V03. |
| AGENTS.md "run_code sandbox is agent automation." | current | `run_code` routed through `SandboxManager`; removed when OS sandbox unavailable (`infra.rs:533`); separate from the interactive PTY. See V02/V03. |
| F-EXT-01 "ToolFailure carries category + recovery" | current | `ToolFailure` decoded and persisted at `chat.rs:1075-1078` and `tool_execution.rs:299/406`. |
| F-EXT-02 "ShellTool and RunCodeTool delegate to the sandbox when configured" | current | Both tools are registered via `enable_tools()`; `run_code` gated by `configure_run_code_capability` against `has_local_os_sandbox`. |
| A-BOOT-01 "create_agent_with_diagnostics is the single composition root" | current | This task treats it as the authoritative wiring point for tool registration and sandbox. |

## Coverage And Uncertainty

- The framework internals of `SandboxManager::execute` (how `ResourceLimits`
  are enforced inside the local sandbox, `sandbox-exec` / `bwrap` argument
  construction) were NOT audited — that is F-SEC-01's scope. This task only
  verified that EKO selects `local_sandbox()` and removes `run_code` when the
  probe fails.
- The Tauri command `execute_sandbox` (panels.rs:859) is a manual panel
  execution path that consumes `max_memory_mb`/`max_cpu_seconds`/`network_enabled`
  but NOT `security_level`. It is a user-driven panel, not an agent path, so
  its disconnection from `security_level` is folded into A-TOOL-01-P2-01 rather
  than filed separately.
- MCP/LSP/browser tool exposure is out of scope (A-INT-01). The browser tools
  are installed via `browser_runtime.install_tools` and surface through
  `tool_search`; their per-mode visibility was not exhaustively checked.
- The runtime-event observer (`chat.rs:960-1180`) was read for the cancel and
  finish paths; its event-ordering edge cases (out-of-order, duplicate, late
  completion) belong to A-FE-02.
- No executable test was run in this review (read-only). All V-series reports
  are static-inspection validations against `echo-agent-cli` commit `b3b2e81`
  and `echo-agent` at the same checkout.
- A-TOOL-01-P3-01 (Auto excludes `run_code`) is low-confidence; intent could
  not be confirmed from code alone. If product confirms intent, the finding is
  void and only a comment is needed.

## Handoff

Conclusions downstream tasks may rely on:

- EKO uses **one** framework tool registry per agent and applies per-mode
  policy purely through invocation-scoped `visible_tools` + `disabled_tools`
  filters (`tool_exposure.rs`). Downstream tasks auditing any specific tool's
  availability can treat `groups_for_mode` + `disabled_tools_for_mode` as the
  authoritative matrix, and `ToolRuntime::from_agent`
  (`echo-agent/src/agent/snapshot.rs:185`) as the authoritative composition.
- Subagent role diff is enforced at registration: readonly subagents
  physically lack shell/write/`run_code`; writer subagents get the full set
  with the same `run_code` no-bare gate. Downstream subagent tasks can rely on
  this.
- The interactive terminal (`tauri/terminal.rs`) and agent `run_code` are
  independent execution channels with no shared state. A-INP-01 / A-INT-01
  auditing interactive surfaces can treat the PTY as user-action scope and
  `run_code` as agent-automation scope.
- The `ToolExecutionRepository` is a pure projection: it records start/output/
  finish/cancel but holds no scheduling or cancellation authority. A-FE-02 can
  treat it as the durable backing store for frontend tool-execution views.

Reports downstream tasks must read:

- `zcode-glm/tasks/F-EXT-01.md`, `F-EXT-02.md` (tool contract and shell/file/
  code/git tool semantics).
- `zcode-glm/tasks/A-BOOT-01.md` (composition root, multi-mode parity gaps
  including the TUI terminal absence re-referenced as A-TOOL-01-P3-02).

Conditions that make this report stale:

- Any change to `tool_exposure.rs` group arrays, `disabled_tools_for_mode`,
  or the snapshot tests.
- Any change to `configure_run_code_capability`, the `SandboxManager`
  construction at `infra.rs:265`, or the `has_local_os_sandbox` probe.
- Any change to `tauri/terminal.rs` command signatures or the consent gate.
- Wiring `SandboxConfigData.security_level` into the agent sandbox (would
  resolve A-TOOL-01-P2-01).
- Adding a TUI interactive terminal (would resolve A-TOOL-01-P3-02).

Follow-up task IDs (not implemented in this review):

- A-INT-01: MCP/LSP/browser integration reachability (depends on this task's
  tool-exposure matrix).
- A-FE-02: frontend projection of tool executions (depends on this task's
  repository/observer conclusions).
- A-SURFACE-PARITY (if chartered): own the TUI interactive-terminal gap
  (A-TOOL-01-P3-02) and the boot-lifecycle parity gaps from A-BOOT-01.
