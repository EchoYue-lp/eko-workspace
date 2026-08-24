# X-SRF-01: Surface feature parity

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only cross-cutting synthesis; both repos
> `git status --short` empty)

## Question

Are GUI, TUI, CLI, channels, cron, and background modes complete Agents
differing only in trigger and rendering policy?

Per AGENTS.md: all modes must have feature parity. Missing capabilities
are gaps.

## Scope

This is a **cross-cutting synthesis task**. It consumes the four
application-surface dependency reports (A-SRF-01, A-SRF-02, A-SRF-03,
A-SRF-04) and three capability dependency reports (A-TOOL-01, A-PLG-01,
A-INT-01), then re-verifies the parity facts they assert against the
live code at the pinned commits.

Primary source paths inspected directly (not via the dependencies) for
cross-mode comparison:

- `echo-agent-cli/src/main.rs:240-445` — the four entry branches
  (`run_tui_or_cli_entry`, channels-only exit, the `--cli` REPL branch,
  the GUI `run_desktop_entry` call).
- `echo-agent-cli/src/cli/modes.rs:32-64, 118-235` —
  `start_headless_services`, `run_cli_mode`, `run_channels_mode`.
- `echo-agent-cli/src/cli/repl.rs:160-267, 483-545` — REPL
  `CommandRegistry` registration (20 `register_all` calls), the
  `chat_with_agent` `drive_chat` spawn.
- `echo-agent-cli/src/cli/channels.rs:108-265, 306-412` — the channel
  command surface (`/mode`, `/trace`, `/analysis`, `/papers`, `/skills`
  only — verified at `:313, :339, :381, :394, :407`), `handle_stream`
  → `drive_chat`, the `register_run_cancellation` *absence* in Chat/Auto.
- `echo-agent-cli/src/cli/cmd_impls/skills.rs:240-291` — the REPL
  `/mcp [list|connect|disconnect]` stub (verified: `connect`/`disconnect`
  branches only `println!`, no `agent.connect_mcp_*` call).
- `echo-agent-cli/src/tui/commands.rs` (skim, 399 lines) — the 57-variant
  `SlashCommand` enum (counted via grep).
- `echo-agent-cli/src/tauri/mod.rs:69-310` — the 219 `#[tauri::command]`
  registrations (counted via grep).
- `echo-agent-cli/echo-agent-app-core/src/tasks/service.rs:474, 516,
  541, 552, 563` — every recovery / resume path filter on
  `conversation_id.starts_with("background:")` (cron runs excluded).
- `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:240-252` —
  `register_run_cancellation` invoked **only** for `InteractionMode::Task`.
- `echo-agent-cli/echo-agent-app-core/src/runtime.rs:130-131` —
  `ReplHumanLoopProvider` registered under dispatcher key `"repl"`
  (confirms REPL HITL).
- `echo-agent-cli/echo-agent-app-core/src/hitl/channel_provider.rs:24-
  90` — `ChannelHumanLoopProvider` per-sender HITL (confirms channels
  HITL).

Framework anchors cross-referenced (read-only):

- `echo-agent/src/agent/react/run/execution.rs` — the shared ReAct loop
  that every mode's `drive_chat` reaches.

## Out Of Scope

Deferred to named task IDs:

- **A-SRF-01**: TUI reducer / sink / TaskRuntime view internals. This
  task consumes its capability inventory.
- **A-SRF-02**: Tauri command-side adapter correctness, lock semantics,
  window-event handling. This task consumes its command-surface count
  and the parity findings (P2-01..P2-03, P3-01..P3-04).
- **A-SRF-03**: React frontend reducer / transport. This task consumes
  its reducer-monotonicity and recovery findings.
- **A-SRF-04**: Non-GUI trigger adapter matrix, cancel / recovery
  semantics. This task consumes its trigger-lane conclusions
  (P2-01..P2-03, P3-01).
- **A-TOOL-01**: per-mode tool visibility matrix and the
  interactive-terminal-vs-`run_code` separation. This task consumes its
  tool-exposure matrix.
- **A-PLG-01**: plugin / skill / hook reload lifecycle. This task
  consumes its single-orchestrator conclusion.
- **A-INT-01**: browser / MCP / LSP integration reachability. This task
  consumes its P1-01 / P2-01 / P2-02 parity findings.
- **A-BOOT-01**: boot-lifecycle parity findings (P2-01/02/03) that
  A-SRF-04-P2-03 sharpens. Not re-audited here.
- **A-CHAT-01**: the `drive_chat` single-lifecycle invariant. Treated
  as the load-bearing contract that makes "chat lane" meaningful.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` via system reminder. Load-bearing
  sections: multi-mode functional parity ("TUI、GUI(以及 CLI/channel)
  必须功能对等", "禁止以'某模式不需要'为由拒绝给该模式接入能力",
  "代码里若出现 'X 模式 doesn't use Y' 之类的注释/None 传参,那是
  待补的缺口,不是产品定位"), Claude Code parity target
  (TUI = complete Agent), framework-vs-application layering gate,
  no-panic / UTF-8 safety rules, "first check whether it already
  exists" rule.
- `docs/comprehensive-review/REPORTING.md`.
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.

Dependency reports read:

- `zcode-glm/tasks/A-SRF-01.md` (complete) — TUI capability inventory
  (57 slash commands, 22-variant AgentEvent reducer, TaskRuntime /
  SubagentRuntime / HITL render); load-bearing for V01: the TUI column
  of the matrix. Two parity findings (P2-01 dead `parallel_tasks`,
  P2-02 subagent detail collapse) imported.
- `zcode-glm/tasks/A-SRF-02.md` (complete) — GUI command inventory
  (219 Tauri commands); load-bearing for V01: the GUI column. Four
  parity findings (P2-01 terminal cleanup, P2-02 permission alias
  drift, P2-03 subagent bridge duplicate authority, P3-01..P3-04)
  imported.
- `zcode-glm/tasks/A-SRF-03.md` (complete) — Frontend reducer and
  transport; load-bearing for V02: scenario-replay shape on the GUI
  receive side. Four findings (P2-01 live reducer not monotone,
  P3-01..P3-04) imported.
- `zcode-glm/tasks/A-SRF-04.md` (complete) — Non-GUI trigger matrix;
  load-bearing for V01: the CLI/channels/cron/background columns and
  the two-lane conclusion (chat lane → `drive_chat`, task lane →
  `execute_run`/`drive_agent_run`). Four findings (P2-01 chat-turn
  cancel, P2-02 cron no auto-resume, P2-03 channels-only no services,
  P3-01 channel slash set reduced) imported.
- `zcode-glm/tasks/A-TOOL-01.md` (complete) — Per-mode tool visibility
  matrix and interactive-terminal separation; load-bearing for V01:
  the `tool` row + the TUI-terminal parity gap (P3-02).
- `zcode-glm/tasks/A-PLG-01.md` (complete) — Plugin / skill / hook
  lifecycle; load-bearing for V01: the `plugins` / `skills` rows
  (single shared `PluginRuntimeService` across all surfaces).
- `zcode-glm/tasks/A-INT-01.md` (complete) — Browser / MCP / LSP
  reachability; load-bearing for V01: the `browser` / `MCP` / `LSP`
  rows and the four findings (P1-01 IPC over-validation, P2-01 no
  graceful MCP/LSP shutdown, P2-02 no LSP restart surface,
  P3-01..P3-02).

Historical documents treated as hypotheses: none. No prior X-SRF-01
report exists in this reviewer's directory; the surface reports above
are treated as primary evidence and re-verified at the pinned commits.

## Layering Decision

This is a **cross-cutting synthesis** task at the application layer.
No framework code is touched; no new code paths are proposed. The
parity gaps identified here are owned by the dependency reports'
follow-up task IDs; this report does not introduce new fixes.

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework supplies the right parity primitives: the shared `drive_chat` chat-lane entry (A-CHAT-01), the shared `execute_run` / `drive_agent_run` task-lane entry (A-TSK-03), the shared `MessageHandler` trait for channels, the shared `SchedulerRunner` generic for cron, the shared `TaskRuntimeStore` for recovery, the shared `PluginRuntimeService` for plugins/skills/hooks, the shared `HumanLoopProvider` dispatcher for HITL. Every mode composes these primitives without parallel implementations. |
| EKO product policy | The per-mode surface policy (TUI's 57 slash commands, GUI's 219 IPC commands, channels' 5 slash commands, the cron/background trigger adapters, the per-mode tool visibility filter) is correctly in the application layer. The parity gaps are *missing surface wiring*, not framework defects. |
| Adapter boundary | Every mode is correctly a thin adapter: it builds a `PreparedUserTurn` + `ChatResources` (chat lane) or a `RunPayload` (task lane), calls the shared entry, and renders the resulting event stream. None recomputes the chat lifecycle, the DAG, or the frontier. The parity question is solely: does each mode wire every capability that the others wire? |
| Duplicate search | Searched both repos for the shared entry points to confirm single-lane routing: `drive_chat` (4 chat-lane callers — TUI, GUI, REPL, channels), `execute_run` / `drive_run_async` / `drive_agent_run` (5 task-lane callers — TUI, GUI, cron, background, `create_complex_task`). No third lane. The dependency reports'V01 greps confirm one canonical definition of `MessageHandler`, `SchedulerRunner`, `BackgroundTaskService`, `PluginRuntimeService`, `TaskRuntimeStore`. |
| Migration deletion | No deletion proposed. The findings identify missing surface wiring; resolution is left to the dependency reports' follow-up task IDs. |

## Current Path

### Verified six-mode inventory (V01)

The application ships **six trigger modes** that share **two execution
lanes**. Every mode is a thin adapter; the lanes are the framework
/application-shared entries.

```text
─── Chat lane (one entry: drive_chat) ──────────────────────────────

  TUI            GUI              CLI/REPL         channels
  ─────────      ─────────────    ──────────────   ──────────────
  handle_enter   send_chat_msg    chat_with_agent  handle_stream
  dispatch_turn  (chat.rs:442)    (repl.rs:485)    (channels.rs:195)
  (events.rs:
   1341)
      │              │                │                │
      └──────────────┴────────────────┴────────────────┘
                            ↓
                  PreparedUserTurn + ChatResources
                            ↓
                  drive_chat (chat_driver.rs:202)
                            ↓
                  envelope_event_stream (one terminal)
                            ↓
                  TuiChatSink / TauriChatSink / ChannelChatSink
                  (pure renderers — A-CHAT-01 V02)

─── Task lane (one entry: execute_run / drive_agent_run) ───────────

  TUI            GUI              cron             background
  ─────────      ─────────────    ──────────────   ──────────────
  start_tui_     execute_run      launch_cron_run  start_run_driver
  task_run_      (from            (runner.rs:101)  (service.rs:372)
  driver         create_complex_
  (events.rs:    task tool)
  4716)
      │              │                │                │
      └──────────────┴────────────────┴────────────────┘
                            ↓
                  RunPayload / cron payload
                            ↓
                  execute_run / drive_unattended_run / drive_agent_run
                  (executor.rs:3571-3891)
                            ↓
                  TaskRuntimeStore (single persistence / recovery /
                  cancellation registry — A-TSK-03)
```

The cron and background triggers do NOT have a chat surface — they
enter the task lane directly. This is intentional and not a parity gap
(they are unattended triggers, not chat surfaces). The parity question
for them is whether they preserve identity, events, recovery, and
terminal semantics — which they do, with the two exceptions documented
in A-SRF-04-P2-02 (cron no auto-resume) and A-SRF-04-P2-03 (channels-
only skips the scheduler service).

### Verified capability matrix (V01)

See [V01-01](../validations/X-SRF-01/V01-01.md) for the full matrix.
Headline: of 17 capability rows × 6 mode columns = 102 cells, **61
are full-parity ✅**, **28 are partial**, **8 are explicit gaps ❌**,
and **5 are intentionally N/A** (unattended triggers don't have chat
surfaces).

### Verified scenario replay (V02)

Three representative scenarios traced through every applicable mode.
See [V02-01](../validations/X-SRF-01/V02-01.md) for the full trace.
Headline: all modes produce **equivalent facts** (same `turn_id` /
`run_id` derivation, same tool results, same terminal status, same
recovery semantics). The differences are purely in *rendering policy*
(TUI's ratatui widgets vs GUI's React stores vs REPL's plain text vs
channels' chunked IM messages vs cron/background's no UI). The
underlying `AgentEvent` / `ChatDriverEvent` / `ExecEvent` stream is
identical.

### Verified gap inventory (V03)

Twelve distinct parity gaps aggregate from the dependency reports. See
[V03-01](../validations/X-SRF-01/V03-01.md) for the per-gap matrix
with owning task and mode impact. Headline categories:

1. **Missing slash-command surface** (channels: `/cron`, `/tasks`,
   `/plugins`, `/worktrees`, `/diff`, `/pipeline`; REPL: `/mcp
   connect/disconnect` is a stub).
2. **Missing interactive panes** (TUI: no terminal pane; TUI+GUI+CLI:
   no LSP management panel).
3. **Missing service wiring** (channels-only: no scheduler, no BG
   service).
4. **Missing cancel handle** (REPL + channels Chat/Auto turns have no
   externally reachable cancel; GUI window close orphans PTY shells).
5. **Missing recovery path** (cron runs recovered to Paused but never
   auto-resumed).
6. **Asymmetric persistence** (GUI-only `ToolExecutionRepository`;
   TUI/CLI/channels drop tool-execution detail on session exit).
7. **Asymmetric subagent observability** (TUI collapses 11/16
   `SubagentEvent` variants to a counter; channels surface only
   attention events; GUI has full dashboard).
8. **Cross-surface canonicalization drift** (permission-mode aliases
   triplicated TUI/GUI/CLI with drift).

## Findings

The headline result is **strongly positive on the primary paths and
the two-lane architecture**: every mode that is supposed to be a
complete Agent (TUI, GUI, CLI) reaches the shared `drive_chat` and
`execute_run` entries, binds the shared services, and renders the
shared event stream. AGENTS.md's "TUI = complete Agent" mandate is met
on the chat / task / tool / HITL / resume / attachment / browser / MCP
/ skills / hooks / plugins / memory / cron / worktrees / analysis /
research paths. Channels, cron, and background are correctly thin
adapters into the same lanes.

The gaps are **surface-wiring gaps**, not architectural gaps. Eight
findings aggregate the dependency reports' parity conclusions into a
cross-cutting view; the fix direction for each is owned by the cited
dependency-report finding.

### X-SRF-01-P1-01: GUI MCP IPC over-validation blocks legitimate local servers — surfaces-asymmetric with the on-disk config path (cross-filed from A-INT-01-P1-01)

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/mcp.rs:117-160` —
    `validate_ipc_mcp_stdio` rejects any stdio command whose base-name
    is not in `{npx, node, uvx, uv, python, python3, pipx, docker,
    java}` (an executable allowlist).
  - `echo-agent-cli/src/tauri/commands/mcp.rs:169-208` —
    `validate_ipc_mcp_url` rejects any URL whose host matches
    `localhost`, `127.0.0.1`, `::1`, `169.254.*`, `10.*`,
    `192.168.*`, `172.16.*..172.31.*` (private-range block).
  - The on-disk path (`McpConfigFile::from_file` →
    `validate_stdio_command` at
    `echo-integration/src/mcp/config_loader.rs:229-261`) only blocks
    shell metacharacters + a small dangerous-command denylist + path
    traversal. It accepts `http://localhost:8080/mcp` (which the
    framework's own doc examples at `config_loader.rs:27, 32` use).
  - The TUI `/mcp load <path>` (`tui/events.rs:3425-3477`) calls
    `agent.load_mcp_from_file(path)` directly — no IPC allowlist, so
    any on-disk config is accepted.
- Reachability: every GUI user opening the MCP panel and trying to add
  a server whose URL is `https://localhost:8100/mcp` or whose stdio
  command is `/usr/local/bin/my-custom-mcp`. The same user can put
  identical content in `~/.eko/mcp.json` (or call TUI `/mcp load`) and
  it works.
- Expected invariant: AGENTS.md "产品定位与安全边界" — EKO is a local
  personal assistant; "不要套用线上 Web 服务的威胁模型" (do not
  apply the online web-service threat model); "保留对明显错误输入的
  轻量校验即可,不要做权限级拦截" (lightweight validation only, no
  permission-level interception).
- Observed behavior: the GUI IPC path applies an executable allowlist
  and a private-range URL block under an online XSS/SSRF threat model
  that AGENTS.md explicitly excludes for EKO. The on-disk path (used
  by TUI/CLI/channels/GUI-startup) applies only lightweight typo /
  shell-injection guards. The two paths are inconsistent.
- Impact: this is the highest-severity parity gap because it makes a
  user-configured capability **unreachable** in the default GUI panel
  while the same configuration works in every other surface. It is the
  same class of regression as the historical `require_full_auto` gate
  (AGENTS.md "历史教训") — over-gating that produces surface asymmetry.
- Root cause: the validator was added under an online / multi-user
  threat model before AGENTS.md codified the local-assistant rule.
- Direction: align `validate_ipc_mcp_stdio` and `validate_ipc_mcp_url`
  with the on-disk `validate_stdio_command` discipline (denylist +
  shell-metacharacter + path-traversal only; drop the executable
  allowlist; drop the loopback / private-range rejection). Owned by
  A-INT-01-P1-01's follow-up.
- Regression validation: extend the `validate_ipc_mcp_*` unit tests
  (`mcp.rs:565-613`) to assert that `http://localhost:8080/mcp` and
  `/usr/local/bin/my-custom-mcp` are accepted.
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md)
  (matrix row "MCP"), [V03-01](../validations/X-SRF-01/V03-01.md).

### X-SRF-01-P2-01: REPL and channels Chat/Auto turns have no externally reachable cancel — TUI parity gap (cross-filed from A-SRF-04-P2-01)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/cli/repl.rs:533` and
    `echo-agent-cli/src/cli/channels.rs:244` — fresh
    `CancellationToken::new()` per turn, never registered.
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:240-252` —
    `register_run_cancellation` invoked **only** for
    `InteractionMode::Task`. Chat and Auto turns (the defaults for REPL
    and channels) skip registration.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:
    532-548` — `register_run_cancellation` is the only inserter into
    `run_cancel_tokens`; an unregistered token is unreachable via
    `request_cancel`.
  - Contrast: `echo-agent-cli/src/tui/events.rs:1937-1958` (`handle_esc`)
    and `:1147-1153` (`q` key) cancel via `app.active_cancel.cancel()`;
    `:1410` sets `app.active_cancel = Some(cancel.clone())` before
    spawning. TUI therefore CAN cancel an in-flight Chat/Auto turn.
- Reachability: every Chat/Auto turn on REPL and every chat turn on
  channels. The REPL Ctrl+C handler at `repl.rs:239-241` only prints a
  hint; it does not cancel the in-flight turn.
- Expected invariant: AGENTS.md multi-mode parity — TUI users can
  cancel an in-flight turn; CLI and channel users cannot. The chat
  turn's cancel token should be reachable from the surface that
  started it.
- Observed behavior: a long-running turn on a channel blocks that
  sender's pool agent until completion (the agent is held via
  `pool.acquire(&conv)` at `channels.rs:135-139`, and the pool
  serializes per-key). The user's next message is queued behind the
  in-flight one with no escape hatch.
- Impact: (a) functional parity gap vs TUI; (b) operational — a
  runaway agent turn on a channel ties up the per-sender agent until
  the bot is killed; (c) UX — the IM user has no way to stop a turn
  that is producing unwanted side effects.
- Root cause: the per-turn CancellationToken was modeled as a private
  resource for the chat lane; only Task mode was wired for external
  cancellation. The TUI solved this independently via
  `app.active_cancel`; REPL and channels never gained the equivalent.
- Direction: (1) install a `tokio::signal::ctrl_c()` handler on REPL
  and a `/cancel` slash command on channels (the framework's
  `SessionHandler` already routes `/`-prefixed messages as commands);
  (2) extend `ChatResources` or the surface state with an accessible
  cancel handle (REPL: `Mutex<Option<CancellationToken>>` on
  `ReplConfig`; channels: a per-sender map on
  `AppChannelMessageHandler`). Owned by A-SRF-04-P2-01's follow-up.
- Regression validation: an integration test that boots REPL, sends a
  turn, issues `/cancel`, and asserts the turn terminates; a
  MockLlmClient-driven test that cancels mid-turn and asserts the sink
  receives a terminal.
- Validation reports: [V02-01](../validations/X-SRF-01/V02-01.md),
  [V03-01](../validations/X-SRF-01/V03-01.md).

### X-SRF-01-P2-02: Channels-only entry skips SchedulerRunner + BackgroundTaskService — cron and background capabilities unavailable (cross-filed from A-SRF-04-P2-03 / A-BOOT-01-P2-02)

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
    constructs the `ChannelManager` and registers
    `AppChannelMessageHandler`, but never starts the scheduler or the
    background task service.
- Reachability: any `echo-agent-cli --channels` launch (without
  `--cli`). Live gap.
- Expected invariant: AGENTS.md multi-mode parity — channels must be
  a full Agent surface. Cron and background tasks are core services
  that should be available in every long-running entry.
- Observed behavior: a user running `echo-agent-cli --channels` as a
  long-running IM bot gets no scheduled cron fires and no
  `BackgroundTaskService::submit` pathway. Tasks created via the
  channel chat (the agent calling `create_complex_task`) DO still run
  (they go through `drive_run_async` from the tool body, not from the
  background service). But the cron schedule and the explicit
  `BackgroundTaskService::submit` API are unavailable.
- Impact: silent capability gap for channels-only deployments. A user
  who configures cron tasks in `~/.eko/...` expects them to fire; in
  channels-only mode they never do.
- Root cause: the channels branch was wired directly to
  `run_channels_mode` without routing through the shared headless
  service starter.
- Direction: call `start_headless_services` in the channels-only
  branch before spawning `run_channels_mode`, mirroring the TUI/CLI
  branches at `main.rs:258-274`. Owned by A-SRF-04-P2-03's follow-up.
- Regression validation: boot `--channels` with a fake CronTask and
  assert the scheduler ticks; assert `BackgroundTaskService` is
  constructible.
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md),
  [V03-01](../validations/X-SRF-01/V03-01.md).

### X-SRF-01-P2-03: Cron runs recovered to Paused on restart but never auto-resumed — recovery parity gap vs background (cross-filed from A-SRF-04-P2-02)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/service.rs:474, 516,
    541, 552, 563` — every recovery / resume / list path in
    `BackgroundTaskService` filters on
    `conversation_id.starts_with("background:")`. A cron run's
    `"cron:..."` prefix fails this filter and is invisible to the
    background service.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:
    1631-1760` — `recover_incomplete` transitions any `Running` run to
    `Paused` regardless of conversation prefix. So cron runs ARE
    reconciled to Paused on next boot — but nothing then wakes them up.
  - `echo-agent-cli/src/cli/cmd_impls/cron.rs:296-302` — `/cron resume`
    re-enables a `CronTask` schedule (the `CronTaskStatus`), not an
    interrupted run. The slash command surface has no notion of
    resuming a paused cron RUN.
- Reachability: any cron run interrupted by process restart (Ctrl+C,
  SIGTERM, crash, machine reboot).
- Expected invariant: AGENTS.md multi-mode parity — background runs
  resume on next boot (`service.rs:556-589`), and cron runs are
  conceptually equivalent (an unattended TaskRuntime run on a pool
  agent). They should resume the same way.
- Observed behavior: a cron run interrupted mid-flight becomes Paused
  forever. Its partial work is preserved on disk, but no component
  re-dispatches it. The next cron tick fires a NEW run for the same
  CronTask, duplicating the work.
- Impact: (a) silent capability gap — interrupted cron runs leak
  Paused runs that pile up in the store; (b) potential duplicate work
  when the schedule fires again before the user notices; (c) the cron
  promise of "runs on schedule" is broken across any restart.
- Root cause: the `background:` filter was written when background was
  the only unattended trigger; cron was added later with its own
  conversation prefix but was not added to the resume filter.
- Direction: extend the resume filter in
  `BackgroundTaskService::resume_pending` to also cover
  `conversation_id.starts_with("cron:")` (or drop the prefix filter
  and resume any Paused run that the recovery blockers allow).
  Owned by A-SRF-04-P2-02's follow-up.
- Regression validation: a test that seeds a Paused cron run in the
  store, calls `resume_pending` (after the filter fix), and asserts
  the run is re-dispatched.
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md).

### X-SRF-01-P2-04: TUI subagent internal lifecycle collapses to a counter — 11 of 16 framework SubagentEvent variants silently dropped (cross-filed from A-SRF-01-P2-02)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tui/events.rs:5343-5434` —
    `update_subagent_runs` explicitly matches `DispatchStarted`,
    `DispatchToolStarted`, `DispatchCompleted`, `DispatchFailed`,
    `DispatchCancelled`, and a catch-all `_ => {}` at `:5428`. Five
    variants handled.
  - `echo-agent/src/agent/subagent/events.rs:14-248` (read-only) — the
    framework enum declares **16 variants** (incl. `DispatchToolCompleted`,
    `DispatchTokenDelta`, `DispatchThinkingDelta` / `Started` / `Ended`,
    `DispatchLlmUsage`, `DispatchIsolationObserved`, `Registered` /
    `Unregistered`, `TeamCreated` / `TeamDissolved`).
  - Contrast: `echo-agent-cli/src/tauri/mod.rs:335-769` (the GUI
    subagent bridge) persists every `DispatchToolStarted` /
    `DispatchToolCompleted` into the shared `ToolExecutionRepository`,
    and emits the per-event payload to `execution://event` for the
    frontend dashboard.
- Reachability: every subagent dispatch (every `agent_tool` /
  `task_execute` invocation that creates a subagent). Live on every
  TUI session.
- Expected invariant: AGENTS.md multi-mode parity — "任何一方有的
  能力...其它方也应有". The GUI exposes per-subagent-tool detail and
  per-subagent LLM usage; the TUI does not.
- Observed behavior: during a subagent run, the TUI shows a counter
  increment ("3 tools" in the sidebar) and the user must wait for
  `DispatchCompleted` to see the summary. There is no live indicator
  of what the subagent is doing, no token usage for the subagent's LLM
  calls, no thinking trace. The status bar's context-window ring
  reflects only the foreground agent's `LlmUsage`, so a long subagent
  run can consume significant context without the ring moving.
- Impact: (a) parity gap — subagent observability is materially lower
  in the TUI than the GUI; (b) the context-window indicator
  undercounts during subagent-heavy turns; (c) for debugging subagent
  hangs, the TUI user has no streaming signal.
- Root cause: `update_subagent_runs` was written when the
  `SubagentEvent` enum was smaller; the thinking / token / LLM-usage
  variants were added later (for the GUI dashboard) without
  backfilling the TUI reducer.
- Direction: short-term, extend `update_subagent_runs` to handle
  `DispatchLlmUsage` (accumulate into a new status bar snapshot),
  `DispatchThinkingStarted/Delta/Ended` (one-line status), and
  optionally `DispatchTokenDelta`. Long-term, inherit per-tool detail
  from a unified `ToolExecutionObserver` (A-CHAT-01-P2-01 /
  A-SRF-02-P2-03). Owned by A-SRF-01-P2-02's follow-up.
- Regression validation: a test that drives
  `SubagentEvent::DispatchLlmUsage { tokens_used: 100 }` through
  `update_subagent_runs` and asserts the accumulator increased.
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md).

### X-SRF-01-P2-05: No LSP interactive management surface on any mode — framework `LspManager::restart_server` has zero application callers (cross-filed from A-INT-01-P2-02)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent/echo-integration/src/lsp/manager.rs:105-108` —
    `pub async fn restart_server(&mut self, language: &str) ->
    Result<(), String>` (stop-then-start).
  - `grep -rn "restart_server|restart_lsp|lsp_restart" echo-agent-cli/`
    returns **zero** application callers.
  - `echo-agent-cli/src/tui/events.rs:3425-3477` exposes a `/mcp`
    slash command but no `/lsp` equivalent.
  - `echo-agent-cli/src/tauri/commands/` contains `mcp.rs` (full CRUD)
    but no `lsp.rs`.
- Reachability: any LSP server that crashes or hangs. After the first
  incident the corresponding LSP tools silently no-op or hang for
  every subsequent call until the user restarts the entire EKO process.
- Expected invariant: AGENTS.md multi-mode parity — if GUI has a
  panel, TUI should have an equivalent slash command, and vice versa.
  Here, no surface has it.
- Observed behavior: the framework's restart primitive exists but is
  unreachable from any application surface. Recovery requires app
  restart. Compounds F-INT-02-P2-01 (no auto-restart on crash) and
  F-INT-02-P2-02 (no per-request timeout).
- Impact: usability regression. A user debugging a flaky
  `rust-analyzer` or `pyright` has no in-app recovery.
- Root cause: the application never wired the framework's restart
  primitive to any surface. The MCP side got a full panel; the LSP
  side got only startup discovery.
- Direction: add a Tauri command `restart_lsp_server(language)` and
  a TUI `/lsp restart <lang>` slash command, both delegating to
  `plugin_runtime.lsp.manager.write().await.restart_server(...)`.
  Owned by A-INT-01-P2-02's follow-up.
- Regression validation: a unit test that constructs an `LspManager`
  with a mock client, calls restart, and asserts stop+start counters
  increment.
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md).

### X-SRF-01-P2-06: GUI window close orphans PTY shells — `TerminalManager.close_all()` never invoked (cross-filed from A-SRF-02-P2-01)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/terminal.rs:256-267` — `close_all`
    definition (drains `sessions`, spawns `kill()` per session).
  - `echo-agent-cli/src/tauri/desktop.rs:256-268` — the only GUI
    cleanup path after `.run()` returns; calls `cancel_token.cancel()`,
    `shutdown_hook_events`, `browser_runtime.shutdown()`. **No terminal
    cleanup.**
  - `echo-agent-cli/src/tauri/mod.rs:69-310` — `build_tauri_app`
    registers no `.on_window_event(...)`. `grep -rn "on_window_event"`
    across `src/`, `src-tauri/` returns zero hits.
  - `echo-agent-cli/src/tauri/state.rs:9-23` — `terminal_manager`
    lives on `TauriState`, which is constructed inside
    `TauriState::new` and handed to `.manage(...)`; it is **not**
    reachable from the `Arc<AppState>` that `desktop.rs` holds.
- Reachability: every GUI window close.
- Expected invariant: desktop app lifecycle must release OS resources
  it allocated. AGENTS.md "framework自身 bug 造成破坏" / data-loss
  avoidance.
- Observed behavior: on window close, every live `PtySession`'s child
  shell is left running. The `pty-reader` std threads die with the
  process, but the shell PIDs persist (reparented to launchd on macOS).
- Impact: resource leak. A `npm run dev` the user started in an EKO
  terminal keeps serving after the app is "closed". The user has no UI
  to discover or kill them short of `ps` / Activity Monitor.
- Root cause: the GUI entry (`desktop.rs`) was written before the
  terminal feature; the terminal was added as a command surface only,
  without wiring its lifecycle into shutdown.
- Direction: register an `on_window_event` handler in `build_tauri_app`
  for `WindowEvent::CloseRequested` that pulls
  `app.state::<TauriState>().terminal_manager.close_all()`. Owned by
  A-SRF-02-P2-01's follow-up.
- Regression validation: open a terminal, run `sleep 1000`, close the
  window, confirm `pgrep -f <shell>` returns nothing.
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md).

### X-SRF-01-P3-01: Channels slash-command surface is reduced — only 5 of ~20 REPL commands wired (cross-filed from A-SRF-04-P3-01)

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/cli/channels.rs:313, 339, 381, 394, 407` — the
    only five slash commands parsed by `parse_channel_*`:
    `/mode`, `/trace`, `/analysis`, `/papers`, `/skills`.
  - `echo-agent-cli/src/cli/repl.rs:161-180` — REPL `CommandRegistry`
    has 20 `register_all` calls: analysis, coding, diff_cmd, git,
    session, info, context, advanced, skills, hooks, observability,
    evolution, tasks_ext, research, pipelines, pipeline, workspace,
    plugins, cron, all.
  - `echo-agent-cli/src/cli/cmd_impls/cron.rs:563-565` — `register_all`
    is only called from `repl.rs:179`. The channel handler never
    constructs a `CommandRegistry` and never calls `cron::register_all`.
- Reachability: every channel conversation. The IM user who tries
  `/cron list` over IM gets the message routed to `drive_chat` as a
  normal chat instruction (the agent answers conversationally) rather
  than as a structured slash command.
- Expected invariant: AGENTS.md multi-mode parity — slash commands
  available in REPL should be reachable from channels where they make
  sense.
- Observed behavior: only `/mode`, `/trace`, `/analysis`, `/papers`,
  `/skills` are wired on channels. The rest is REPL-only.
- Impact: low. The agent can answer conversational equivalents, and
  most slash commands are developer-tooling that does not map cleanly
  to IM. But `/cron` is operationally important for a channels-only
  bot.
- Root cause: the channel command surface was added incrementally (one
  helper per command as the need arose); there is no shared command
  dispatcher between REPL and channels.
- Direction: factor a `ChannelCommandDispatcher` that mirrors the REPL
  `CommandRegistry` for the subset of commands that make sense over
  IM (at minimum `/cron`, `/mode`, `/skills`, `/trace`), or route the
  entire `CommandRegistry` through channels with a denylist for
  commands that require a TTY. Owned by A-SRF-04-P3-01's follow-up.
- Regression validation: a `parse_channel_command` test that asserts
  `/cron list` dispatches and returns an `OutboundMessage`.
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md),
  [V03-01](../validations/X-SRF-01/V03-01.md).

### X-SRF-01-P3-02: TUI has no interactive terminal pane — only the GUI PTY exists (cross-filed from A-TOOL-01-P3-02)

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tui/mod.rs` — all "terminal" references are
    ratatui/crossterm UI plumbing (`enable_raw_mode`, `AlternateScreen`,
    `TerminalGuard`); grep for `pty` / `portable_pty` / `create_terminal`
    in `src/tui/` returns zero hits.
  - `echo-agent-cli/src/tauri/terminal.rs:278` — interactive PTY
    terminal is GUI-only.
- Reachability: every TUI session.
- Expected invariant: AGENTS.md "TUI 与 GUI 是功能完全一样的 Agent 完全体,
  只是交互方式不同". The interactive terminal (a user-action developer
  tool) should be reachable in both surfaces, or the gap should be
  tracked.
- Observed behavior: GUI users get an embedded PTY terminal; TUI users
  do not. (Note: the TUI does have a `!<shell>` escape at
  `events.rs:1664` that runs `sh -lc '<command>'` as a one-shot — but
  that is not an interactive pane.)
- Impact: low for the Agent capability question (the terminal is
  correctly separate from `run_code` wherever it exists). The parity
  gap itself is a surface concern.
- Root cause: the interactive terminal was implemented as a Tauri
  command only; no TUI widget was added.
- Direction: a future TUI terminal widget should reuse the same
  consent + audit semantics from `tauri/terminal.rs`, not a parallel
  implementation. Owned by A-TOOL-01-P3-02's follow-up (and
  A-BOOT-01 / B-PATH-01's broader boot-lifecycle parity track).
- Regression validation: N/A (no code change in this task).
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md).

### X-SRF-01-P3-03: Tool-execution persistence is GUI-only — TUI/CLI/channels/cron/background all drop tool-execution detail on session exit

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/chat.rs:1193-1340` —
    `TauriChatSink::handle_tool_event` performs `tool_executions.start
    / append_output / finish / cancel` for the foreground-agent tool
    calls (A-CHAT-01-P2-01).
  - `echo-agent-cli/src/tauri/mod.rs:335-769` — the subagent bridge
    persists subagent tool calls into the same repository
    (A-SRF-02-P2-03).
  - `echo-agent-cli/src/tui/events.rs:2031-2210` — `TuiChatSink::on_event`
    renders tool executions as in-memory `ToolExecutionMessage` entries
    that are discarded on session exit (A-SRF-01 handoff point 3).
  - `echo-agent-cli/src/cli/repl.rs:563-845` and
    `echo-agent-cli/src/cli/channels.rs:514-654` — both render tool
    events as plain text with no persistence.
- Reachability: every chat turn that invokes tools, in every non-GUI
  surface.
- Expected invariant: AGENTS.md multi-mode parity — tool-execution
  history should be durable across surfaces (or every surface should
  explicitly opt out).
- Observed behavior: GUI users can scroll back through a session's
  tool calls after reload; TUI / CLI / channels users cannot.
- Impact: medium. A TUI/CLI/channels user who reloads a conversation
  loses the per-tool-call detail (the messages reload, but the
  structured tool execution view does not).
- Root cause: the `ToolExecutionRepository` was wired into
  `TauriChatSink` only; the TUI/CLI/channel renderers predate it.
- Direction: long-term, extract a unified `ToolExecutionObserver`
  (A-CHAT-01-P2-01 / A-SRF-02-P2-03) into `ChatResources`, and have
  every surface's sink delegate to it. Short-term, document the
  asymmetry in user-facing docs.
- Regression validation: a test driving a foreground tool call
  through both `TauriChatSink` and `TuiChatSink` (post-fix) and
  asserting both produce the same `ToolExecutionSummary` shape.
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md),
  [V03-01](../validations/X-SRF-01/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Capability matrix: 17 capabilities × 6 modes; classify each cell ✅ / partial / ❌ / N/A with code anchor | yes | passed (with findings) | [V01-01](../validations/X-SRF-01/V01-01.md) |
| V02 | Common scenario replay: trace chat turn / task run / tool execution through every applicable mode; verify equivalent facts | yes | passed | [V02-01](../validations/X-SRF-01/V02-01.md) |
| V03 | Missing event / tool / attachment / HITL paths: per-gap matrix with owning task and mode impact | yes | passed (with findings) | [V03-01](../validations/X-SRF-01/V03-01.md) |
| V04 | Parity action list: prioritize which gaps are most impactful; fix direction | yes | passed | [V04-01](../validations/X-SRF-01/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a | No prior X-SRF-01 report under `zcode-glm/`; this is the first cross-cutting surface synthesis. |

No `cargo` / `vitest` command was executed in this task. The
validations are static cross-report syntheses re-verified against the
pinned commits. The underlying test evidence (TUI 39 tests, frontend
101 tests, scheduler/channels/launch_unattended cargo subsets) is in
the dependency reports' V* reports.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md "TUI 目标是与 GUI 功能对等的完全体(对标 Claude Code)" | mostly current, with two surface gaps | V01 confirms the TUI's broad capability matrix (57 slash commands across 9 categories); V03 records two TUI-specific gaps: no interactive terminal pane (P3-02) and subagent detail collapse (P2-04). The primary paths (chat, tasks, tools, HITL, resume, attachments, browser, MCP, skills, hooks, plugins, memory, cron, worktrees, analysis) are all reachable. |
| AGENTS.md "禁止以'某模式不需要'为由拒绝给该模式接入能力" | current (load-bearing) | V01 / V03 confirm there are no `// X mode doesn't use Y` comments justifying missing wiring. The gaps are undocumented absences, not product-policy decisions. |
| AGENTS.md "代码里若出现 'X 模式 doesn't use Y' 之类的注释/None 传参,那是待补的缺口,不是产品定位" | current (load-bearing) | V01 confirms: channels-only skipping `start_headless_services` is a missing-wiring gap (P2-02), not a deliberate "channels doesn't need cron" decision. Cron's exclusion from `resume_pending` is a missing-filter gap (P2-03), not "cron doesn't resume". |
| A-SRF-01 handoff point 1 "TUI is feature-complete on primary paths" | current | V01 re-confirms: every primary capability has a reachable TUI surface. The two TUI gaps (P2-04, P3-02) are observability / interactive-pane gaps, not core-capability gaps. |
| A-SRF-04 handoff point 1 "Two lanes, two entries" | current (load-bearing) | V01 confirms: chat lane (TUI+GUI+CLI+channels) → `drive_chat`; task lane (TUI+GUI+cron+background) → `execute_run` / `drive_agent_run`. No third lane. |
| A-SRF-04 handoff point 4 "Chat/Auto cancel is reachable only on TUI" | current (sharpened into X-SRF-01-P2-01) | V02 / V03 re-confirm at the pinned commits. |
| A-INT-01 handoff "all three integrations reachable from every surface at integration level; gap is interactive-surface parity" | current | V01 row "browser" / "MCP" / "LSP": integration-level cells are ✅ across modes; interactive-management cells are partial (MCP) or ❌ (LSP). |
| A-TOOL-01 handoff "tool registry is mode-agnostic; per-mode visibility filter applies" | current | V01 row "tool": ✅ across all six modes. The framework registry + `InteractionMode` filter is the single authority; no per-mode re-registration. |
| A-PLG-01 handoff "exactly one application orchestrator; GUI/CLI/TUI delegate to the same shared Arc" | current | V01 rows "plugins" / "skills": ✅ for TUI/GUI/CLI; channels is ❌ for plugins and partial for skills. The single-orchestrator design is intact; the gap is the channels command surface (P3-01). |

## Coverage And Uncertainty

Inspected in full (cross-cutting lens):

- `echo-agent-cli/src/main.rs:240-445` (entry branches),
  `src/cli/modes.rs:32-64, 118-235` (headless + channels),
  `src/cli/repl.rs:160-267, 483-545` (REPL registry + chat spawn),
  `src/cli/channels.rs:108-265, 306-412` (handle_stream + command
  parse), `src/cli/cmd_impls/skills.rs:240-291` (the `/mcp` stub),
  `echo-agent-app-core/src/chat_driver.rs:240-252` (cancel
  registration), `echo-agent-app-core/src/tasks/service.rs:474-563`
  (resume filter), `echo-agent-app-core/src/runtime.rs:130-131`
  (`ReplHumanLoopProvider`),
  `echo-agent-app-core/src/hitl/channel_provider.rs:24-90`
  (`ChannelHumanLoopProvider`). Cross-repo grep for every shared entry
  (`drive_chat`, `execute_run`, `MessageHandler`, `SchedulerRunner`,
  `BackgroundTaskService`).

Inspected partially (via dependencies):

- The TUI reducer / Tauri command bodies / React reducer internals —
  read through the dependency reports' line ranges and re-verified at
  the anchors cited above.

Not inspected (out of scope):

- The framework `ReactAgent`'s internal `Cancelled` synthesis behavior
  (F-RCT-03 scope).
- The frontend TypeScript panel rendering beyond what A-SRF-03
  inspects.
- Per-tool permission prompts (X-AUT-01 / A-HITL-01 scope).

Environmental constraints:

- Read-only static cross-cutting synthesis at `echo-agent` `9b0e0fa`
  and `echo-agent-cli` `b3b2e81`. No build or test execution in this
  task. The worktree is clean on both repos.

Uncertain claims:

- Whether any user is actually running `--channels` in production
  today (P2-02's blast radius depends on this). The AGENTS.md parity
  rule applies regardless of current deployment.
- Whether the cron auto-resume gap (P2-03) is masked in practice by
  users rarely restarting the bot during a cron fire.
- Whether the GUI's dashboard actually surfaces subagent thinking /
  token deltas live (P2-04's "TUI collapses 11/16 variants" gap is
  smaller if the GUI also collapses them). A-SRF-02 documents the GUI
  emits the events to `execution://event`; whether the frontend
  renders them live is owned by A-SRF-03.

## Handoff

Conclusions downstream tasks may rely on:

1. **The two-lane architecture is sound.** Every mode enters through
   `drive_chat` (chat lane) or `execute_run` / `drive_agent_run`
   (task lane). No third lane, no parallel implementation. Downstream
   cross-cutting tasks (X-* synthesis) can treat these as the single
   authoritative entries.
2. **Surface parity is high on primary paths.** Of 17 capability rows
   × 6 mode columns, 61 are full-parity ✅. Chat, task, tool, HITL,
   attachment, resume, browser (integration-level), MCP (config-file
   level), skills, memory, worktree, analysis, research all reach
   every applicable mode.
3. **Eight parity gaps aggregate from the dependency reports.**
   - **P1**: GUI MCP IPC over-validation (A-INT-01-P1-01) — the only
     P1, because it makes a user-configured capability unreachable in
     the default GUI panel.
   - **P2**: REPL/channels chat-turn cancel missing (A-SRF-04-P2-01);
     channels-only skips scheduler+BG (A-SRF-04-P2-03); cron no
     auto-resume (A-SRF-04-P2-02); TUI subagent detail collapse
     (A-SRF-01-P2-02); no LSP interactive management on any mode
     (A-INT-01-P2-02); GUI terminal cleanup missing (A-SRF-02-P2-01).
   - **P3**: channels slash set reduced (A-SRF-04-P3-01); TUI no
     interactive terminal pane (A-TOOL-01-P3-02); tool-execution
     persistence GUI-only.
4. **All gaps are surface-wiring gaps, not architectural gaps.** The
   framework supplies every primitive the surfaces need (cancellation
   tokens, slash-command routing, service starters, restart_server,
   close_all, ToolExecutionRepository). The fixes are localized to
   the application layer.
5. **The "X mode doesn't use Y" anti-pattern is absent.** No code
   comment or `None` parameter justifies a missing capability as
   product policy. Every gap is an undocumented absence that AGENTS.md
   classifies as "待补的缺口,不是产品定位" (a gap to fill, not a
   positioning decision).

Reports downstream tasks must read:

- This report (X-SRF-01) for the cross-cutting capability matrix,
  scenario replay, and the eight prioritized parity gaps.
- `tasks/A-SRF-01.md` for the TUI column detail and the TUI-specific
  findings (P2-01, P2-02, P3-01, P3-02).
- `tasks/A-SRF-02.md` for the GUI command surface (219 commands) and
  the GUI-specific findings (P2-01..P2-03, P3-01..P3-04).
- `tasks/A-SRF-03.md` for the frontend reducer policies and the
  recovery / reload semantics.
- `tasks/A-SRF-04.md` for the non-GUI trigger matrix (CLI / channels
  / cron / background) and the cancel / recovery gaps.
- `tasks/A-TOOL-01.md` for the per-mode tool visibility matrix.
- `tasks/A-PLG-01.md` for the plugin / skill / hook single-orchestrator
  contract.
- `tasks/A-INT-01.md` for the browser / MCP / LSP reachability gaps.

Conditions that make this report stale:

- Any new mode added (e.g. a `--daemon` flag, a plugin-supplied
  Slack/Discord channel, an HTTP webhook entry) requires a new column
  in V01.
- Any new capability added (e.g. a new agent tool family) requires a
  new row in V01.
- Resolving any of the eight cited findings invalidates the
  corresponding matrix cell. Specifically:
  - Wiring `start_headless_services` into the channels-only branch
    (resolving P2-02) flips the channels-column cells for cron /
    background to ✅.
  - Wiring an `on_window_event(CloseRequested)` → `close_all` (resolving
    P2-06) removes the GUI terminal cleanup gap.
  - Adding an `/lsp` slash command + `lsp.rs` Tauri module (resolving
    P2-05) flips the LSP row to ✅ across modes.
  - Aligning the IPC MCP validators with the on-disk path (resolving
    P1-01) removes the surface asymmetry.
- Adding the unified `ToolExecutionObserver` (resolving A-CHAT-01-P2-01
  / A-SRF-02-P2-03 / P3-03 here) flips the tool-execution persistence
  row to ✅ across modes.

Follow-up task IDs (no fixes implemented in this review):

- A **surface-parity P1 fix** task: resolve X-SRF-01-P1-01 by aligning
  `validate_ipc_mcp_stdio` / `validate_ipc_mcp_url` with the on-disk
  `validate_stdio_command` discipline. Touches only
  `tauri/commands/mcp.rs`.
- A **chat-turn cancel surface** task: resolve X-SRF-01-P2-01 by
  wiring an accessible cancel handle on REPL and channels. Touches
  `repl.rs`, `channels.rs`, possibly `chat_resources.rs`.
- A **channels-only services wiring** task: resolve X-SRF-01-P2-02 by
  routing the channels-only branch through `start_headless_services`.
  Touches `main.rs:357-403`.
- A **cron auto-resume** task: resolve X-SRF-01-P2-03 by extending
  `BackgroundTaskService::resume_pending`'s filter. Touches
  `tasks/service.rs`.
- A **TUI subagent detail extension** task: resolve X-SRF-01-P2-04 by
  extending `update_subagent_runs`. Touches `tui/events.rs`.
- An **LSP interactive surface** task: resolve X-SRF-01-P2-05 by
  adding `/lsp` (TUI) + `lsp.rs` (Tauri). Touches `tui/events.rs`,
  `tauri/commands/`.
- A **terminal-lifecycle** task: resolve X-SRF-01-P2-06 by wiring
  `on_window_event(CloseRequested)` → `close_all`. Touches
  `tauri/mod.rs`, `tauri/desktop.rs`.
- A **channel slash-command parity** task: resolve X-SRF-01-P3-01 by
  factoring a `ChannelCommandDispatcher`. Touches `channels.rs` and
  possibly `cmd_impls/`.
- A **TUI interactive terminal widget** task: resolve X-SRF-01-P3-02
  by adding a TUI PTY pane reusing `tauri/terminal.rs` semantics.
- A **unified tool-execution recorder** task: resolve X-SRF-01-P3-03
  together with A-CHAT-01-P2-01 / A-SRF-02-P2-03 by introducing one
  recorder used by every surface's sink.
