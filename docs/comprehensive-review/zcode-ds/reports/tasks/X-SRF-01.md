# X-SRF-01: Surface feature parity

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both repositories clean (`git status --porcelain` empty at
> review start; this review executed no write to any source file)

## Question

Are GUI, TUI, CLI, channels, cron, and background modes complete Agents
differing only in trigger and rendering policy?

**Answer: yes at the agent-capability core, no at the management and control
surface layer.** All six entry classes reach the same shared core — one
`AgentRuntime::bootstrap` agent construction, the same `drive_chat`
application driver (4 live production call sites: TUI/GUI/REPL/channels),
one TaskRuntime store + `task_create/update/list` + `task_execute`, one
`AgentPool`, one `PreparedUserTurn` attachment path, one browser runtime, one
HITL dispatcher — so the "complete Agent differing only in trigger and
rendering" invariant holds for chat, task execution, subagents, tools,
attachments, memory, browser and MCP tools. It fails at the management and
control layer: workspace, research, evolution, task-run management, browser
management, steer, and cancel are unevenly distributed across surfaces. The
task files **three new gaps** (REPL has no browser management surface;
channels have no task-run management surface; REPL/channel turns cannot be
steered) and folds **21 already-filed canonical findings** into the
capability matrix (V01-01/V03-01), all re-verified current at the reviewed
commits (V04-01).

## Scope

Primary source paths inspected (anchor re-verification at the reviewed
commits; deep subsystem semantics consumed from dependency reports):

- Entry composition: `src/main.rs` (mode branches :95-445, TUI HITL swap
  :249-257, channels branch :357-411), `src/cli/modes.rs` (full;
  `start_headless_services` :32-64, `run_cli_mode` :68-110,
  `run_channels_mode` :118-235), `src/tauri/desktop.rs` (:124-271 GUI boot),
  `echo-agent-app-core/src/runtime.rs` (:110-150 bootstrap + HITL dispatcher).
- Chat/attachment adapters: `src/tui/events.rs` (dispatch_turn :1341-1436,
  send_to_agent :2212-2230, cancel/queue :1937-1958, terminal arms :602-908),
  `src/tauri/commands/chat.rs` (send/interrupt :443-534, cancel :807-826),
  `src/cli/repl.rs` (:190-237 input loop, :477-544 turn build),
  `src/cli/channels.rs` (:102-265 turn path, :334-413 NL special handlers,
  :515-654 aggregator).
- Management surfaces: `src/tui/commands.rs` (SlashCommand inventory),
  `src/cli/cmd_impls/{workspace,coding,research,evolution,cron,skills}.rs`,
  `src/tauri/commands/{workspace,research,browser,mcp,scheduler,
  task_runtime,memory}.rs`, `src/cli/channels.rs` (mode commands),
  `echo-agent-app-core/src/tasks/service.rs` (submit/resume).
- Unattended triggers: `echo-agent-app-core/src/tasks/task_runtime/
  executor.rs` (:3571 launch_unattended_run, :3616 drive_unattended_run,
  :3649 drive_agent_run, :3895 launch_cron_run, :3662-3664 run cancel
  registration), `echo-agent-app-core/src/scheduler/runner.rs` (fire_fn).
- Framework anchors: `drive_chat` definition
  (`echo-agent-app-core/src/chat_driver.rs:202`).

## Out Of Scope

- Shared driver/sink semantics, envelope terminal contract → A-CHAT-01.
- Per-surface rendering correctness (frontend reducers, TUI reducer,
  TauriChatSink) → A-SRF-01/02/03 (consumed; their findings folded).
- Trigger adapter internals beyond the matrix rows (cron/background
  lifecycle, recovery) → A-SRF-04 (consumed; P1-01/P1-02/P2-01/P2-02/P3-01/
  P3-02 folded).
- Tool exposure/sandbox/terminal → A-TOOL-01 (P1-01/P2-01/P3-01/P3-02
  folded).
- Skills/plugins/hooks lifecycle → A-PLG-01 (P1-01/P2-01/P2-02/P3-01/P3-02
  are workspace-scope or lifecycle, folded as workspace/plugin context).
- Browser/MCP/LSP integration → A-INT-01 (P1-01/P2-01..05/P3-01/P3-02
  folded; the REPL browser-management absence is the new finding here).
- HITL provider arbitration → A-HITL-01 (P1-02/P1-03/P2-01..03 folded).
- Framework stream/cancel defects → F-RCT-03-P1-01/P1-02 (folded as
  shared-event rows).
- Dynamic multi-surface scenarios → Q-E2E-01 (this task is static; V02
  replays scenarios as traces for Q-E2E-01 to execute).

## Inputs

- Root `AGENTS.md` (full; multi-mode parity as product positioning,
  "X mode doesn't use Y" is a gap, TUI-positioning history lesson),
  shared `README.md`, `REPORTING.md`, `TASKS.md` (X-SRF-01 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read (zcode-ds, all complete): `A-SRF-01`,
  `A-SRF-02`, `A-SRF-03`, `A-SRF-04`, `A-TOOL-01`, `A-PLG-01`, `A-INT-01`;
  canonical IDs additionally read in full: `A-CFG-01` (P1-03), `A-BOOT-01`
  (P2-02), `A-OUT-01` (P2-04), `A-EVO-01` (P2-03), `A-HITL-01` (P1-02,
  P2-03), `A-CHAT-01` (P2-01).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md` (surface
  parity claims :18/:124/:333/:770), `echo-agent-cli/docs/
  2026-07-17-surface-parity-closeout.md` — classified in Historical Claim
  Status.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework) | The shared driver (`drive_chat`), envelope terminal contract, `PreparedUserTurn`, TaskRuntime tools, agent pool, HITL dispatcher, scheduler runner, browser/MCP/LSP adapters are framework/application-core as already classified by A-CHAT-01/A-TSK-03/A-HITL-01/F-INT-01 — reused identically by every surface. No movement recommended. |
| EKO product policy (application) | All per-surface adapters, sinks, slash-command inventories, and management surfaces are application-layer. The three new findings are application/adapter-surface gaps: REPL browser management (P2-01), channel task-run management (P2-02), REPL/channel steer (P3-01). |
| Adapter boundary | The four chat adapters (TUI/GUI/REPL/channels) are thin over the shared driver (each builds `PreparedUserTurn` + `ChatResources` and calls `drive_chat`); the two new gaps on REPL/channels are missing adapter surfaces, not second authorities. Unattended adapters (`launch_unattended_run`/`submit`) are thin over `drive_agent_run`/`execute_run`. |
| Duplicate search | Terms (V01-01, both repos): `drive_chat` (1 def, 4 live call sites + tests), `ChatSink` impls (4), `PreparedUserTurn::build` (4 live call sites), `register_task_tools_on_agent` (TUI main.rs:177, GUI desktop.rs:201), `bind_task_execute_to_pool` (2), `start_headless_services` (TUI main.rs:258, CLI modes.rs:83; channels none), `SlashCommand` variants vs REPL `cmd!` registrations (TUI 66 / REPL ~40+ / channels 0), `workspace` (GUI full / TUI 0 / REPL stub), `browser` (TUI + GUI cmds / REPL 0), `research` (GUI + REPL + channels NL / TUI 0), `steer` (TUI + GUI / REPL 0 / channels 0), `worker` (zero in all surfaces). |
| Migration deletion | No deletion targets from this task; fixes extend existing services (`AppState` workspace registry, `HitlDispatcher`, `SchedulerRunner`, task-store commands) with new surface adapters. |

## Current Path

Verified call graph (V01-01/V02-01):

1. Boot: `AgentRuntime::bootstrap` (runtime.rs) builds one agent per entry —
   TUI/CLI (`main.rs:167-168`), GUI (`desktop.rs:159-160`); REPL provider
   registered as the default HITL provider on every boot (runtime.rs:130-131);
   TUI swaps it for `TuiHumanLoopProvider` (main.rs:249-257).
2. TaskRuntime: one store per process — built in `main.rs:175` (headless),
   `AppState::from_shared` (GUI); task tools registered on the primary agent
   (main.rs:177, desktop.rs:201), `task_execute` bound to the pool
   (main.rs:192, desktop.rs:217).
3. Interactive chat: user input → surface adapter → `PreparedUserTurn::build`
   (TUI events.rs:1347, GUI chat.rs:536, REPL repl.rs:509, channels
   channels.rs:208) → `ChatResources` (incl. per-surface cancel token) →
   `drive_chat` (TUI events.rs:2226, GUI chat.rs:688, REPL repl.rs:543,
   channels channels.rs:262) → per-surface sink.
4. Task triggers: TUI `/task-*` + GUI commands + REPL `/tasks` + channel
   agent `create_complex_task` all drive the same TaskRuntime store; cron →
   `launch_cron_run` (executor.rs:3895) → `launch_unattended_run` (:3571) →
   `drive_unattended_run` (:3616) → `drive_agent_run` (:3649); background →
   `BackgroundTaskService::submit` → `start_run_driver` → `execute_run`/
   `drive_unattended_run`; unattended runs carry Auto-mode tool exposure
   (executor.rs:3686-3713) and register a run cancel token (:3662-3664).
5. HITL: per-surface providers (TUI swap main.rs:253-254, GUI per-conversation
   chat.rs:575-589, channels per-handler channels.rs:146-150, REPL default);
   unattended runs resolve approvals through pool agents' empty dispatcher →
   fail-closed auto-reject (A-HITL-01-P2-03).

## Findings

### X-SRF-01-P2-01: The REPL has no browser management surface — TUI has `/browser status|managed|chrome` and the GUI has a browser command set, but the CLI surface has zero browser commands, so a CLI user cannot inspect or restart the browser sidecar or navigate

- Priority: P2
- Confidence: high
- Layer: application (CLI surface; adapter)
- Evidence: `echo-agent-cli/src/tui/events.rs:4551-4615` (TUI
  `SlashCommand::Browser` → `status|managed|chrome` on the shared
  `BrowserRuntime`); `echo-agent-cli/src/tauri/commands/browser.rs:23-95+`
  (GUI `browser_navigate/back/reload/screenshot/click_at/scroll` command
  set, registered in `src/tauri/mod.rs`); `echo-agent-cli/src/cli/` —
  repository-wide `grep -rln "browser" src/cli/` returns **zero hits**
  (V01-01); browser tools themselves are registered on the primary agent
  (`echo-agent-app-core/src/infra.rs:454-455`) and on subagents
  (`infra.rs:951-952,1028-1029`), so chat-mediated use works on every
  surface.
- Reachability: definition (absent) → registration (absent) → caller: a REPL
  user cannot check sidecar status (`/browser status`), force a restart
  (`/browser managed`/`chrome`), or navigate; the agent can still drive the
  browser via `browser_navigate`-family tools, but there is no user-side
  management or direct navigation path.
- Expected invariant: every surface is a complete Agent surface with the
  same capability set (AGENTS.md parity); a management surface present on
  TUI and GUI is present on the CLI; the CLI is not reduced to "agent tools
  only".
- Observed behavior: TUI and GUI expose a browser management/navigation
  surface; REPL exposes none. This mirrors the CLI MCP stubs defect
  (A-INT-01-P2-01) but for browser instead of MCP: the management surface is
  simply absent rather than stubbed.
- Impact: a CLI user cannot verify or recover the browser sidecar (the
  only recovery path is the agent asking to retry); direct user-driven
  navigation exists on GUI/TUI but not on the CLI. Parity gap on a
  documented capability; lower impact than the CLI MCP stubs because the
  agent-level tools remain reachable.
- Root cause: the browser management commands were added to the TUI slash
  surface and the GUI IPC surface when the BrowserRuntime landed, and the
  CLI command registry was never extended (same growth pattern A-OUT-01-P2-04
  documents for research/export).
- Direction: add a `BrowserCommand` to the CLI registry (mirror
  `tui/events.rs:4551-4615`: `/browser status|managed|chrome` calling
  `BrowserRuntime`), or explicitly document browser management as TUI/GUI
  only — the gap must be resolved, not silently kept.
- Regression validation: REPL fixture — `browser_runtime` present, `/browser
  status` prints sidecar state identical to the TUI output; `/browser chrome`
  triggers a restart. Q-E2E-01 candidate.
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md),
  [V03-01](../validations/X-SRF-01/V03-01.md)

### X-SRF-01-P2-02: Channels have no task-run management surface — TUI/GUI/REPL can list/pause/cancel/resume runs, but a channel user (and the channel agent, which has only `task_execute`) cannot stop or inspect a run from the channel

- Priority: P2
- Confidence: high
- Layer: application (channel surface)
- Evidence: TUI `/task-cancel|pause|resume|retry|skip|recovery`
  (`src/tui/events.rs:4305-4507`); GUI `src/tauri/commands/task_runtime.rs`
  (list_task_runs etc.); REPL `/tasks` full surface
  (`src/cli/cmd_impls/coding.rs:50-289`); channels: `src/cli/channels.rs`
  has no slash-command handling at all (`parse_channel_mode_command` :118
  handles only mode commands; NL special handlers :153-188 cover trace/
  analysis/papers/skills — nothing for task runs); the channel agent's tool
  set is the Auto-mode exposure including `create_complex_task` and
  `task_create/update/list/execute` (A-TSK-02/A-TOOL-01) — no
  `task_cancel`/`task_pause` tool exists anywhere (run control is a
  store-command surface, `store.request_cancel`, not an agent tool);
  channels-only boot has no background service at all (canonical
  A-SRF-04-P1-02 / A-BOOT-01-P2-02).
- Reachability: every channel conversation; a channel agent that starts a
  background run (create_complex_task) leaves the run uncontrollable from
  the channel; in combined `--channels --cli` mode the run is manageable
  only from the REPL/TUI window, and in channels-only mode not at all after
  restart (canonical A-SRF-04-P2-01).
- Expected invariant: surface parity — the run-control capability available
  on TUI/GUI/REPL is available on channels, differing only in trigger and
  rendering (NL commands on channels, slash commands elsewhere).
- Observed behavior: channel users cannot list, pause, cancel, resume, retry
  or skip runs; the only channel-side control is asking the agent in natural
  language, which the agent cannot perform (no run-control tool).
- Impact: a runaway or unwanted run started from a channel can only be
  stopped by leaving the channel surface; combined-mode users must switch
  windows; the channel surface is the only interactive surface with no
  run-control path.
- Root cause: run control was implemented as surface-store commands
  (TUI slash handlers, GUI IPC, REPL /tasks) when the channel surface was
  designed NL-only; no agent tool for run control was added and no NL
  routing was built for run control (unlike trace/analysis/papers/skills).
- Direction: either (a) add NL routing in `handle_stream` for run control
  (`/tasks`-equivalent intents: status/pause/cancel/resume) mapping to the
  same `TaskRuntimeStore` calls the other surfaces use, or (b) add a
  `task_run_control`-family agent tool (product-scoped, not framework) so
  the channel agent can act on explicit user instructions; fix the
  channels-only service absence (A-SRF-04-P1-02) in the same milestone.
- Regression validation: channel fixture — channel agent creates a run; a
  follow-up channel message "cancel my task" cancels it (run terminal
  Cancelled in the store); combined-mode fixture unchanged.
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md),
  [V03-01](../validations/X-SRF-01/V03-01.md)

### X-SRF-01-P3-01: REPL and channel chat turns cannot be steered — the steer capability (new instruction to an in-flight turn) exists on TUI (`/steer`) and GUI (`steer_chat_message`) but has zero producers on the CLI and channel adapters

- Priority: P3
- Confidence: high
- Layer: application (adapter boundary; same family as A-SRF-04-P1-01)
- Evidence: TUI `/steer` both turn-id and active-turn variants
  (`src/tui/events.rs:1438-1474, 4231-4304`); GUI `steer_chat_message` with
  expected-turn-id precondition (`src/tauri/commands/chat.rs:745-751`);
  REPL — grep of `src/cli/repl.rs` and `src/cli/cmd_impls/*.rs` for `steer`
  returns zero (V01-01); channels — no steer path in `handle_stream`
  (channels.rs:102-265); the input loop blocks inline on the turn in the
  REPL (`chat_with_agent(...).await`, repl.rs:236), so no second input can
  even reach a steer adapter.
- Reachability: definition (absent on REPL/channels) → registration (absent)
  → caller: a REPL user whose agent is mid-turn (e.g. a long research turn)
  cannot redirect it; a channel user cannot redirect an in-flight response
  (the aggregation loop only projects).
- Expected invariant: turn-control semantics (cancel and steer) are shared
  across interactive surfaces; steering is part of the shared driver's
  contract (A-CHAT-01 scope: the driver registers steer paths for TUI/GUI).
- Observed behavior: steer is TUI/GUI-only; the REPL/channel adapters
  construct a cancel token they never fire (canonical A-SRF-04-P1-01) and no
  steer handle at all; the REPL input loop cannot even accept input during a
  turn.
- Impact: minor today (steer is a refinement over cancel), but a control
  capability that violates the parity invariant; same root cause as
  A-SRF-04-P1-01 (adapters never retain a handle on the in-flight turn) and
  should be fixed together with it.
- Root cause: the steer plumbing was added to TUI and GUI (which hold a
  turn handle / active-turn state) while the REPL/channel adapters were
  never given a steer path; the inline-await REPL loop structurally
  precludes it.
- Direction: with the A-SRF-04-P1-01 fix (retain the turn token/handle in
  the adapters), add a steer path: REPL `/steer <text>` raced against the
  turn (select! over the input loop), channel NL routing for redirect
  intents; or explicitly scope steer as an interactive-panel-only control
  with a documented decision — resolve the gap, don't keep it silently.
- Regression validation: after the fix, a REPL fixture with a slow stream
  followed by `/steer "focus on X"` produces a steer event on the driver
  stream; channel fixture similar via NL.
- Validation reports: [V01-01](../validations/X-SRF-01/V01-01.md),
  [V03-01](../validations/X-SRF-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Capability matrix per surface × capability with definition/registration/reachability | yes | passed (3 new gaps found) | [V01-01](../validations/X-SRF-01/V01-01.md) |
| V02 | Common scenario replay — same scenario (chat/cancel/queue/task/attachment) across surfaces, event-flow comparison | yes | passed (5 scenarios; deviations mapped to findings) | [V02-01](../validations/X-SRF-01/V02-01.md) |
| V03 | Missing event/tool/attachment/HITL path inventory per surface | yes | passed (inventory: 4 families, all rows classified) | [V03-01](../validations/X-SRF-01/V03-01.md) |
| V04 | Cross-check with existing findings — canonical ID re-anchor + new-finding disjointness | yes | passed (21 canonical IDs current; 3 new disjoint) | [V04-01](../validations/X-SRF-01/V04-01.md) |

All required validations executed with immutable reports; static inspection
only (no command exit codes apply; the review is read-only and no build was
run — Q-CLI-01/Q-GUI-01/Q-WEB-01 own the build gates).

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `docs/MASTER-PLAN.md:18` "TUI, GUI, CLI, channels share the same Agent capabilities and differ only in input, rendering, and event projection" | partial/regressed at the management layer | shared core holds (V01-01/V02-01); management/control surfaces diverge: workspace (A-SRF-01-P1-01/A-CFG-01-P1-03), research (A-OUT-01-P2-04), evolution (A-EVO-01-P2-03), browser on REPL (P2-01), task control on channels (P2-02), steer on REPL/channels (P3-01), channels-only services (A-SRF-04-P1-02) |
| `echo-agent-cli/docs/2026-07-17-surface-parity-closeout.md:14` "Foreground, background, cron runs required on all surfaces" | regressed (channels) | A-SRF-04-P1-02 re-verified at modes.rs:118-235 (V04-01) |
| `echo-agent-cli/docs/2026-07-17-surface-parity-closeout.md:70-71/:76/:117` (CLI memory/mode/cron closeouts) | fixed | `/remember`/`/forget` functional, `/mode` mutates shared lock, `/cron` drives SchedulerRunner (A-SRF-04 V05, cross-referenced) |
| `docs/MASTER-PLAN.md:124` "一条权威生命周期" | regressed | GUI interrupt ghost (A-SRF-03-P1-01) + error→completed (A-SRF-03-P1-02) re-verified (V04-01) |
| `docs/MASTER-PLAN.md:333` "All six entry points switched to PreparedUserTurn" | current | 4 live call sites verified (V01-01) |
| `docs/MASTER-PLAN.md:770` "interrupt 不再静默丢失" | regressed | `ChatDriverEvent::Interrupt` zero producers (A-CHAT-01-P2-01); GUI interrupt strands frontend (A-SRF-03-P1-01) |
| AGENTS.md historical lesson "TUI is a full surface; 'TUI doesn't use X' is a gap" | current (positive) | TUI drives the same core; the remaining TUI gaps (workspace/research/evolution) are filed as gaps, not positioning (V01-01/V03-01) |

## Coverage And Uncertainty

- All conclusions are static; no process was launched (read-only review).
  The V02 scenario replays are traces, not executions — Q-E2E-01 must run
  them dynamically.
- The concurrent-turn question on channels (two spawned `drive_chat` on the
  same per-sender pool agent, channels.rs:135-265) was not resolved; no
  busy-guard found in the adapter. Recorded as residual uncertainty
  (A-SUB-01/A-CHAT-01 territory), not a finding.
- The "Ctrl+C kills the CLI process" consequence (canonical A-SRF-04-P1-01)
  is inferred from the absence of a signal handler, not dynamically
  reproduced.
- GUI/TUI rendering internals were not re-audited (A-SRF-01/02/03 scope);
  their findings are folded with re-verification of the anchors that carry
  the parity verdict (interrupt, cancel, queue, workspace, MCP stubs,
  double setup).
- The research-row "partial" on channels is based on the NL papers routing
  (channels.rs:171-179); export/audit remain absent there — the placement
  decision (is a full research management surface required on an NL-only
  surface) is left to the product decision documented in A-OUT-01-P2-04's
  direction.
- The parity verdict treats "trigger and rendering policy" as including the
  absence of a slash-command surface on channels (NL-only by design); the
  three new findings are precisely the gaps where no NL route exists either
  (task control, steer, browser management).

## Handoff

- Downstream tasks may rely on: the shared-core conclusion (one driver, one
  TaskRuntime, one pool, one PreparedUserTurn across all six entry classes —
  V01-01/V02-01); the per-capability parity matrix (V01-01); the missing
  path inventory (V03-01); 21 canonical findings re-verified current
  (V04-01); three new gaps — REPL browser management (P2-01), channel
  task-run management (P2-02), REPL/channel steer (P3-01).
- Reports to read: this report + V01-01..V04-01; dependency reports
  A-SRF-01..04, A-TOOL-01, A-PLG-01, A-INT-01; canonical findings
  A-CFG-01-P1-03, A-OUT-01-P2-04, A-EVO-01-P2-03, A-BOOT-01-P2-02,
  A-HITL-01-P1-02/P2-03, A-CHAT-01-P2-01.
- Conditions that make this report stale: changes to `main.rs` mode
  branches, `modes.rs`, `desktop.rs` boot, the four chat adapters (cancel/
  steer/queue/attachment), the CLI command registry (`cmd_impls/*.rs`),
  `channels.rs` handle_stream, `src/tui/events.rs` terminal arms, or the
  Tauri `.setup()` closures invalidate the corresponding rows; a browser
  command added to `src/cli` weakens P2-01; an NL run-control route in
  `channels.rs` weakens P2-02; a REPL/channel steer path weakens P3-01.
- Follow-up task IDs (fixes are not implemented in this review):
  Q-E2E-01 (dynamic multi-surface parity suite — use V02's five scenarios),
  X-EVT-01 (terminal/event conformance rows: interrupt, cancel, queue,
  steer), S-RDM-01 (roadmap: P1-01 cancel wiring first — fixes P3-01's
  family; then workspace/research/evolution surface completion; channels
  services; REPL browser; channel run control).
