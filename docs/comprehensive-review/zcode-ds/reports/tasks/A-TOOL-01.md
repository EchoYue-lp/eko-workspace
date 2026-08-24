# A-TOOL-01: Tool exposure, execution, sandbox, and terminal

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both repositories clean

## Question

Does each Agent/mode expose the intended tools with common error and artifact
behavior, while keeping the interactive terminal separate from Agent
`run_code` policy?

## Scope

- `echo-agent-cli/echo-agent-app-core/src/tool_exposure.rs` (full read),
  `tool_execution.rs` (full read), `infra.rs` (create_agent,
  build_writer_subagent_agent, build_readonly_subagent_agent,
  register_default_subagents, configure_run_code_capability,
  resolve_subagent_model tests), `chat_driver.rs` (drive_chat invocation
  exposure), `tasks/task_runtime/executor.rs` (Task/Auto exposure sites,
  writer/readonly dispatch), `analysis.rs` (run_analysis tool execution),
  `agent_pool.rs` (shared ToolManager/tool_execution_pipeline),
  `state.rs` (ToolState/tool_states, ToolExecutionRepository construction).
- `echo-agent-cli/src/tauri/terminal.rs` (full read), `src/tauri/mod.rs`
  (handler registration + subagent event bridge), `src/tauri/error.rs`
  (IpcAuth), `src/tauri/commands/tools.rs`, `commands/chat.rs`
  (TauriExecutionProjector), `commands/tool_executions.rs`,
  `commands/panels.rs` (execute_sandbox/get_sandbox_status),
  `commands/mcp.rs` (connect_mcp_server gate check).
- `web-frontend/src/components/terminal/Terminal.tsx`,
  `components/tools/ToolsPanel.tsx`, `api/endpoints.ts`.
- Framework anchors: `echo-agent/src/agent/snapshot.rs` (ToolRuntime
  from_agent, tools_for_llm, plan_mode filter), `src/agent/react/run/pipeline.rs`
  (ToolVisibilityStage, PlanModeStage, ToolExecutionPipeline), `src/agent/react/mod.rs`
  (delegate_to_agent_with_prompt_payload), `echo-tools/src/code.rs` (isolation
  floor), `echo-execution/src/sandbox/manager.rs` + `local.rs` (probe).

## Out Of Scope

- Framework tool-contract/registry internals -> F-EXT-01 (re-read).
- Domain tool correctness (shell/file/git paths, UTF-8, process handling) ->
  F-EXT-02 (re-read; EKO-side exposure interplay verified here).
- Permission/HITL policy internals -> F-HITL-01, A-HITL-01.
- Subagent lifecycle -> F-SUB-01/02, A-SUB-01.
- TaskRuntime execution controller -> A-TSK-03 (writer dispatch chain traced
  only as far as tool exposure).
- Analysis workbench correctness -> A-DOM-01 (execution path referenced only).
- Frontend stores/reducers -> A-FE-01/02 (only the two tool/terminal
  components inspected).

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-TOOL-01 card), `zcode-ds/README.md`, templates.
- Dependency reports read: zcode-ds `F-EXT-01` (complete), `F-EXT-02`
  (complete), `A-BOOT-01` (complete).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md:114`,
  `echo-agent-cli/docs/2026-07-16-run-code-sandbox.md`,
  `2026-07-11-tool-execution-rendering-implementation-plan.md`,
  `2026-07-17-surface-parity-closeout.md`.

## Layering Decision

- Generic mechanism (framework): ToolManager registry + 16-stage
  ToolExecutionPipeline (incl. ToolVisibilityStage/PlanModeStage), run_code
  isolation floor, SandboxManager probing — all framework-owned and reused
  correctly; EKO shares one manager/pipeline across pool agents
  (`agent_pool.rs:882-899`) and applies invocation-scoped visibility.
- EKO product policy (application): per-mode tool surfaces
  (tool_exposure.rs), per-role subagent toolsets (readonly vs writer builders
  in infra.rs), run_code capability probe/removal, GUI terminal consent +
  audit, tool-execution detail projection (ToolExecutionRepository).
- Adapter boundary: chat_driver/executor invocation contexts are thin
  adapters (visible_tools + disabled_tools + ExternalRunContext); the
  TauriExecutionProjector + mod.rs bridge are GUI-only projection adapters
  over framework/ExecEvents (no execution authority).
- Duplicate search terms (both repos, V01-01): `initial_visible_tools`,
  `disabled_tools_for_mode`, `register_all_tools`, `register_readonly_tools`,
  `ToolManager::new`, `execute_tool_with_context`, `TerminalManager` /
  `PtySession` / `create_terminal`, `has_local_os_sandbox` / `SandboxManager`,
  `permission_mode`, `require_full_auto` / `full_auto` / `IpcAuth`,
  `ToolExecutionPipeline` / `tool_execution_pipeline`, `tool_states` /
  `ToolState`. Results: one definition per concept; no second EKO tool
  executor (analysis.rs reuses the agent's framework manager); `IpcAuth` has
  zero callers.

## Current Path

Verified data flow:

1. **Exposure**: every turn/run carries an invocation-scoped surface:
   `chat_driver.rs:465-475` (Chat/Auto interactive), `executor.rs:3084-3113`
   (Task-mode main agent), `executor.rs:3686-3713` (Auto-mode unattended) —
   each builds `initial_visible_tools(mode, registered)` + `disabled_tools_for_mode`
   + `record_mode_schema_budget` and passes them as
   `AgentInvocationContext.visible_tools/disabled_tools`. Framework merges
   them into `ToolRuntime` (`snapshot.rs:189-236`) and enforces them in
   `ToolVisibilityStage` (`pipeline.rs:172-210`); `tools_for_llm`
   (`snapshot.rs:273+`) filters the LLM-visible definitions.
2. **Roles**: primary agent = full registry + task tools + browser tools +
   memory layered tools (`infra.rs:276-519`); readonly subagents =
   `readonly_tools()` + plan_mode (`infra.rs:968-1042`); writer subagents =
   full registry + run_code-if-sandbox + plan_mode (`infra.rs:881-965`) —
   the plan_mode on the writer is the defect (P1-01). TaskRuntime routes
   `Implementation|Debugging` to `run_writer_subagent`
   (`executor.rs:2114-2213, 2889`) -> `delegate_to_agent_with_prompt_payload`
   (`echo-agent/src/agent/react/mod.rs:2341`) -> Fork dispatch -> registered
   fork factory (`infra.rs:791-809`) -> `build_writer_subagent_agent`
   (plan_mode=true).
3. **Sandbox**: `create_agent` probes `SandboxManager::local_sandbox()`
   `has_local_os_sandbox()` (`infra.rs:265-272`); on failure `run_code` is
   removed from the primary and writer subagents
   (`configure_run_code_capability`, `infra.rs:533-543, 950`); framework
   `run_code` additionally enforces `IsolationLevel::OsSandbox` as a hard
   floor (`code.rs:313,324`) with `allow_fallback: false` on the EKO manager
   (`manager.rs:229-250`); the GUI sandbox panel fails closed
   (`panels.rs:791-799, 894`). No bare fallback exists.
4. **Terminal**: Tauri PTY (`terminal.rs`) — `create_terminal` has no
   permission gate (explicit local-model comment, `terminal.rs:286-290`);
   `write_terminal` requires per-session user consent
   (`confirm_terminal_consent` on the first genuine keystroke,
   `Terminal.tsx:128-153`), caps writes at 64 KiB, and audit-logs every write.
   Zero `require_full_auto` call sites anywhere (`error.rs` module is dead —
   P3-01). Terminal is separate from agent `run_code` policy by construction
   (run_code is an Agent tool with sandbox floor; the terminal is a
   user-operated PTY).
5. **Artifacts/errors**: execution is identical across modes (single framework
   executor); GUI additionally projects complete args/output into
   `ToolExecutionRepository` via the chat sink (`commands/chat.rs:957-1310`)
   and the subagent event bridge (`src/tauri/mod.rs:353-770`), with paged
   UTF-8-safe reads (`tool_execution.rs`), journal crash repair, and
   Running->Cancelled recovery at boot.

## Findings

### A-TOOL-01-P1-01: Writer subagents (Implementation/Debugging tasks) are silently read-only — EKO-side confirmation and completion of F-EXT-01-P1-01: `set_plan_mode(true)` in the writer builder collides with the framework plan-mode tool filter

- Priority: P1
- Confidence: high (full static chain in both repos; no end-to-end dynamic run — read-only review)
- Layer: application (wiring); framework behavior is by design
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:963`
  (`subagent.set_plan_mode(true)` in `build_writer_subagent_agent`, whose own
  comment at infra.rs:900-911 declares "full tool set (write capability)");
  `infra.rs:1040` (readonly builder — plan_mode appropriate there);
  framework filter `echo-agent/src/agent/snapshot.rs:227-236` (visibility
  whitelist excludes write tools/shell/delete_file when plan_mode),
  `snapshot.rs:282-285` (`tools_for_llm` same filter);
  `echo-agent/src/agent/react/run/pipeline.rs:1004-1018` (PlanModeStage
  blocks the same tools); `set_plan_mode(false)` has zero occurrences in
  either repository (V02-01).
- Reachability: `executor.rs:2114-2118` (`Implementation|Debugging` ->
  `is_writer_task`) -> `executor.rs:2213` -> `run_writer_subagent`
  (`executor.rs:2889`) -> `delegate_to_agent_with_prompt_payload`
  (`echo-agent/src/agent/react/mod.rs:2341`) -> Fork dispatch -> fork factory
  (`infra.rs:791-809`) -> `build_writer_subagent_agent` (plan_mode=true,
  infra.rs:963); the pre-registered handle (infra.rs:679-685, 846-850) is
  built by the same builder. EKO applies no per-role visible/disabled set on
  the delegated subagent, so the subagent's own config governs — and its
  plan_mode strips every write tool from the LLM-visible surface and blocks
  execution.
- Expected invariant: a writer role (frontmatter `readonly: false`,
  TaskRuntime `Implementation/Debugging`) can actually write in its isolated
  worktree; the per-role registry diff must deliver what it declares.
- Observed behavior: write tools (write_file/edit_file/shell/delete_file/
  run_code/git…) are invisible to the model and blocked at three layers
  (tools_for_llm, ToolVisibilityStage, PlanModeStage). The registry-level
  test `writer_subagent_inherits_sandbox_manager` (infra.rs:2120-2153)
  asserts only `list_tools()` and misses the plan-mode surface entirely —
  the feature is broken while tests stay green.
- Impact: TaskRuntime Implementation/Debugging tasks — the flagship
  complex-run capability — cannot modify the worktree; silent capability
  failure since the framework plan-mode filter landed (F-EXT-01-P1-01,
  framework commit 2266d0f).
- Root cause: `set_plan_mode(true)` was copied from the readonly builder
  tail into the writer builder when plan mode did not yet filter tools; the
  framework filter later changed the flag's meaning without a compatibility
  check (per F-EXT-01 root-cause analysis, confirmed on the EKO side).
- Direction: remove `subagent.set_plan_mode(true)` from
  `build_writer_subagent_agent` (keep infra.rs:1040); add an EKO test
  asserting the writer subagent's LLM-visible toolset contains
  write_file/shell/run_code; if read-only planning is desired for writer
  tasks, gate it on the task's own contract, not the tool surface.
- Regression validation: framework plan-mode visibility tests stay green
  (`cargo test -p echo_agent --lib` snapshot plan_mode tests); an EKO unit
  fixture builds a writer subagent and asserts write-tool visibility in
  `tools_for_llm`-equivalent output; an end-to-end mock-LLM writer task calls
  `write_file` and asserts the worktree file exists.
- Validation reports: [V02-01](../validations/A-TOOL-01/V02-01.md),
  [V03-01](../validations/A-TOOL-01/V03-01.md), [V04-03](../validations/A-TOOL-01/V04-03.md)
- Cross-reference: canonical framework-side finding is
  `F-EXT-01-P1-01`; this finding adds the complete EKO-side reachability
  chain, the per-role diff context, and the test blind spot. Synthesizer
  should merge under one canonical ID and keep this backlink.

### A-TOOL-01-P2-01: GUI tool enable/disable toggle is cosmetic — `ToolState` only feeds a display list; "disabled" tools remain fully callable by the agent

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/state.rs:27` (ToolState),
  `state.rs:348` (`tool_states` map), `state.rs:722-747`
  (`get_tool_infos` reads `tool_states` purely to fill `ToolInfo` display
  fields); `echo-agent-cli/src/tauri/commands/tools.rs:33-65`
  (`enable_tool`/`disable_tool` only flip `ToolState.enabled`);
  `web-frontend/src/components/tools/ToolsPanel.tsx:71-84` (toggle UI calls
  these commands and updates local state); grep of `ToolState`/`tool_states`
  shows no other consumer.
- Reachability: ToolsPanel toggle -> `enable_tool`/`disable_tool` IPC ->
  display map; the agent's `ToolManager` registry and the invocation
  visibility/disabled sets (chat_driver.rs:465-475) are never touched.
- Expected invariant: a user-visible "disable" switch on a tool must prevent
  the agent from calling it (or be clearly labeled display-only).
- Observed behavior: the switch flips a flag no execution path reads; the
  agent keeps calling the "disabled" tool; the commands return success.
- Impact: user believes shell/write/other tools are disabled (e.g., as a
  safety preference) while the agent still invokes them — a silent
  no-op control on the flagship surface.
- Root cause: `tool_states` predates the unified registry +
  invocation-visibility model and was never wired to an execution gate;
  the Tauri commands were written as if a gate existed.
- Direction: wire enable/disable into the agent's `disabled_tools` config
  (framework `snapshot.rs:192-200`) or the invocation disabled set
  (chat_driver.rs:465), or delete the toggle and mark the panel
  display-only; remove `tool_states` if unused after the fix.
- Regression validation: GUI fixture toggling `shell` off then a chat turn
  whose model calls `shell` must be blocked (or the toggle removed);
  existing exposure tests stay green.
- Validation reports: [V03-01](../validations/A-TOOL-01/V03-01.md)

### A-TOOL-01-P3-01: Dead `IpcAuth`/`IpcPermission` gate module with a stale doc claiming permission gating on dangerous IPC commands

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/error.rs:6-9` (module doc: "Commands
  that spawn processes, write files outside the workspace, or execute
  arbitrary code are gated behind `IpcAuth::require_full_auto()`"),
  `error.rs:18-60` (`IpcPermission`, `IpcAuth::require_full_auto`,
  `require_not_strict`); grep `IpcAuth|require_not_strict|require_full_auto`
  across `src/` and `echo-agent-app-core/src/` -> zero call sites
  (V01-01); the actual IPC surface is ungated by design
  (terminal.rs:286-290, mcp.rs:216-221).
- Reachability: none — the module compiles but nothing invokes it.
- Expected invariant: docs describe current behavior; dead code is removed
  (AGENTS.md cleanup; the AGENTS.md historical lesson says the gates must
  stay removed).
- Observed behavior: the doc claims gates that do not exist (the desired
  state) and the helper module survives as a trap for future re-gating.
- Impact: a reader or future LLM could conclude the GUI IPC surface is
  protected by full-auto mode (it is not) and reintroduce the exact gates
  the historical lesson removed.
- Root cause: gates were removed in a batch; the module and its doc were
  left behind.
- Direction: delete `IpcPermission`/`IpcAuth` (error.rs) and rewrite the
  module doc to state that interactive IPC is user-trusted under the local
  desktop model with input validation only.
- Regression validation: grep `IpcAuth` after removal -> nothing;
  `cargo check --no-default-features --features gui --bin echo-agent-tauri`
  stays green.
- Validation reports: [V01-01](../validations/A-TOOL-01/V01-01.md),
  [V05-01](../validations/A-TOOL-01/V05-01.md)

### A-TOOL-01-P3-02: GUI subagent event bridge silently drops lagged broadcast events — projected tool executions can stay "Running" until restart

- Priority: P3
- Confidence: medium (mechanism verified; likelihood low)
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/mod.rs:762-764`
  (`RecvError::Lagged(n)` -> warn only, events dropped); the projection
  records start/finish/cancel from these events (mod.rs:416-528) and tracks
  `active_tool_ids_by_execution` (mod.rs:363-366, 425-428, 510-528);
  channel capacity 128 (`echo-agent/src/agent/subagent/events.rs:10,345`);
  recovery only at boot (`tool_execution.rs:527-545`, Running -> Cancelled).
- Reachability: GUI setup bridge on the subagent registry event bus; requires
  >128 unconsumed events (heavy team/parallel fan-out with a stalled GUI
  loop).
- Expected invariant: every recorded tool execution reaches a terminal state
  in the projection (terminal monotonicity of the GUI tool cards).
- Observed behavior: a dropped `DispatchToolCompleted`/terminal event leaves
  the repository record Running and the frontend card spinning until the
  next process start.
- Impact: UI-only staleness of tool cards; no execution or data effect;
  self-heals on restart.
- Root cause: broadcast lag treated as warning-only in a projection that
  must be terminal-complete, with no reconciliation of active executions on
  lag.
- Direction: on `Lagged(n)`, reconcile `active_tool_ids_by_execution`
  (mark Cancelled, mirroring `rebuild_index_and_recover`) or track a gap
  marker; add a lag fixture to `execution_projector_tests`.
- Regression validation: unit test injecting a lagged read into the
  projection loop and asserting no active execution remains Running.
- Validation reports: [V03-01](../validations/A-TOOL-01/V03-01.md),
  [V04-04](../validations/A-TOOL-01/V04-04.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (registry/exposure/executor/terminal/sandbox/permission across both repos) | yes | passed | [V01-01](../validations/A-TOOL-01/V01-01.md) |
| V02 | Registration and runtime reachability (3 driver exposure sites, terminal handler list, sandbox probe, writer dispatch chain) | yes | passed | [V02-01](../validations/A-TOOL-01/V02-01.md) |
| V03 | Invariant/edge cases (per-mode/role registry diff, sandbox no-bare fallback, terminal permission path, large-output/cancel/error fixtures) | yes | passed | [V03-01](../validations/A-TOOL-01/V03-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked tool_exposure` | yes | passed (exit 0, 4 passed) | [V04-01](../validations/A-TOOL-01/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked tool_execution::tests` | yes | passed (exit 0, 5 passed) | [V04-02](../validations/A-TOOL-01/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked unavailable_os_sandbox` + `writer_subagent` | yes | passed (exit 0, 1+1 passed) | [V04-03](../validations/A-TOOL-01/V04-03.md) |
| V04 | `cargo test -p echo-agent-cli --features gui --lib --locked -- task_runtime_tools` | yes | passed (exit 0, 1 passed) | [V04-04](../validations/A-TOOL-01/V04-04.md) |
| V05 | Historical-document drift (M5, MASTER-PLAN:114, rendering plan, parity closeout, error.rs gate doc) | conditional | passed | [V05-01](../validations/A-TOOL-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `MASTER-PLAN.md:114` — bare fallback deleted; EKO probes OS sandbox and removes run_code from main/writer agents; interactive terminal unaffected by run_code contract or agent permission_mode | current | code.rs:313,324; infra.rs:269-272,533-543,950; terminal.rs:286-290; zero IpcAuth callers (V01-01, V05-01) |
| `2026-07-16-run-code-sandbox.md` (M5) — fail-closed run_code contract, per-agent removal table, interactive terminal outside the table | current | V05-01 (writer row additionally affected by P1-01, which is outside the M5 contract) |
| `2026-07-11-tool-execution-rendering-implementation-plan.md` — persisted ToolExecutionView, terminal reconstruction on history restore | current | ToolExecutionRepository + rebuild_index_and_recover (V05-01) |
| `2026-07-17-surface-parity-closeout.md` — tool/terminal capability on all five entry classes | current at capability level | execution identical across modes; PTY widget GUI-only by nature; GUI-only detail projection not a capability gap (V05-01) |
| `src/tauri/error.rs:6-9` — "dangerous IPC commands are gated behind require_full_auto" | stale | zero call sites (P3-01; V01-01, V05-01) |
| `AGENTS.md` historical lesson — require_full_auto gates removed from create_terminal/connect_mcp_server | current (no regression) | terminal.rs:286-290, mcp.rs:216-221 ungated; V01-01 |

## Coverage And Uncertainty

- All behavior claims are static (no process was launched; read-only
  review). The P1-01 chain is verified end-to-end at the source level and
  supported by F-EXT-01's framework-side verification, but no dynamic
  mock-LLM writer-task run was executed.
- The analysis path (`analysis.rs:382-420`) executes `run_code` through the
  agent's ToolManager directly, bypassing the 16-stage pipeline (no
  PreToolUse/PostToolUse hooks, no AuditStage, no TruncationStage — output
  truncation handled in analysis.rs itself). Recorded here, not raised as a
  finding: it is a user-initiated, fingerprint-gated product path with the
  tool-side isolation floor still enforced; hooks semantics belong to
  A-PLG-01 and the analysis feature to A-DOM-01.
- The `disabled_tools_for_mode` blacklist is partly redundant with the
  `initial_visible_tools` whitelist; both are enforced consistently by
  ToolVisibilityStage — noted, not a finding.
- `create_terminal` spawns the user's `$SHELL` from any page JS with no
  cwd containment check beyond the caller-provided `cwd`; the per-session
  consent + audit + write cap mitigate injection but a malicious page could
  still open shells in arbitrary dirs. Recorded as residual uncertainty
  (X-AUT-01 territory), not raised: page JS is trusted under the local
  model and the shell is user-visible.
- GUI-only tool-execution detail projection (TUI/CLI do not persist
  detail_ref pages) — noted; execution/error behavior is identical, so no
  capability gap per surface-parity invariant.
- Frontend stores/reducers beyond ToolsPanel and Terminal were not
  inspected (A-FE-*).

## Handoff

- Conclusions downstream tasks may rely on: per-mode exposure (Chat 16 /
  Task 17 / Auto 19, pinned by tests); one tool executor (framework
  ToolManager); fail-closed sandbox probing with no bare fallback on any
  EKO code path; interactive terminal fully separated from agent
  permission_mode (consent is user-action-gated); ToolExecutionRepository
  is a GUI projection with crash repair; writer-subagent plan_mode defect
  (P1-01), cosmetic tool toggle (P2-01), dead IpcAuth module (P3-01),
  bridge lag-drop (P3-02).
- Reports to read: this report, its 8 validation reports, and dependency
  reports F-EXT-01/F-EXT-02/A-BOOT-01.
- Conditions that make this report stale: changes to tool_exposure.rs
  tables, chat_driver/executor exposure wiring, infra.rs subagent builders
  (plan_mode/readonly_tools), terminal.rs gating/consent, sandbox probe or
  run_code removal, or the GUI event bridge.
- Follow-up task IDs: X-TOL-01 (tool error/artifact conformance — use the
  V04 fixtures), X-AUT-01 (IPC gating classification — P3-01 deletion),
  X-SRF-01 (per-surface tool surfaces — P2-01 toggle), A-TSK-03 (writer
  task capability — P1-01 interaction), A-DOM-01 (analysis execution path
  note), A-PLG-01 (hooks bypass on analysis path). Fixes are deferred to
  the iteration roadmap; this review is read-only.
