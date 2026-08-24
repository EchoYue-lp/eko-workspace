# A-SRF-01: TUI integration

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: `echo-agent` clean; `echo-agent-cli` clean at review start;
> at report time 79 files under `web-frontend/src/generated/` show a
> ts-rs-regenerated diff (single/double-quote and union-layout changes,
> ~279+/679-). This review executed no write to any source file (no build
> script or test in either repository writes to `web-frontend`; both
> `build.rs` files only gate `tauri_build` on the `gui` feature, which was
> not enabled), so the regeneration is attributed to an external/concurrent
> process, not to this read-only task. All findings and commands were
> verified against the source as of the reviewed commits.

## Question

Does the TUI expose and correctly render the complete Agent feature set
rather than a reduced execution path?

**Answer: the TUI is a full-surface Agent entry, not a reduced path — it
drives the same `drive_chat`, TaskRuntime store/tools, HITL dispatcher,
browser runtime, and conversation persistence as the GUI, with one P1
capability gap (no workspace surface at all), three P2 gaps (task-detail
browsing, research workbench, queued-turn drain asymmetry on Cancelled),
and three P3 defects (dead parallel-task strip, dead TurnStatus reducer
arm, missing terminal event fixtures).** All four special-attention items
were independently re-verified: A-CHAT-01-P1-01 does **not** apply to the
TUI (error/cancel terminals render truthfully); A-CFG-01-P1-03 is confirmed
from the TUI side (no workspace surface; code comments even assert "TUI has
no workspace concept" as if it were positioning); A-TSK-03-P1-01 is
reachable from the TUI (`/task-pause` → `request_pause`); A-HITL-01-P1-02
does **not** apply to the TUI path (REPL provider unregistered in TUI
mode), while A-HITL-01-P1-03's SessionAllTools wildcard is produced by the
TUI `a` key.

## Scope

Primary source paths inspected (deep read):

- `echo-agent-cli/src/tui/mod.rs` (full, 2020 lines): `TuiApp` state,
  `TaskProgressEntry`/`parallel_tasks`, `TaskRuntimeView`/
  `SubagentRuntimeView` projections, `run_tui` composition
  (:1882-2019), message/tool rendering helpers, state tests.
- `echo-agent-cli/src/tui/events.rs` (5746 lines, all production slices):
  `run_event_loop` reducer (:556-976), approval keys + `send_pending_response`
  (:36-260), `dispatch_turn` (:1341-1436), steer (:1438-1474), queued-turn
  dispatch (:1476-1484), Esc/cancel + double-Esc rewind (:1937-1958),
  `TuiChatSink` (:2021-2210), `send_to_agent` (:2212-2230), cron
  (:2232-2298), worktrees (:2300-2410), full `handle_slash_command`
  (:2612-4696), task-run resume/driver (:4696-4755), conversation
  reset/resume/fork (:4778-4918), TaskRuntime view refresh/format
  (:4920-5016), `update_subagent_runs` (:5343-5436), tests (:5478-5746).
- `echo-agent-cli/src/tui/commands.rs` (full): SlashCommand inventory (66
  variants) and categories.
- `echo-agent-cli/src/tui/widgets/` (`chat.rs`, `sidebar.rs`,
  `task_strip.rs`, `status_bar.rs`, `input.rs`), `ui.rs`, `markdown.rs`
  (test module only).
- `echo-agent-cli/src/main.rs` (full): TUI/headless entry, TaskRuntime
  store build + task tool registration, pool, HITL provider swap
  (:245-315), resume path (:96-147).
- `echo-agent-cli/echo-agent-app-core/src/hitl/tui_provider.rs` (request
  flow + timeout, :180-222).
- GUI side for the capability matrix (definitions only): `src/tauri/
  commands/workspace.rs`, `task_runtime.rs`, `conversations.rs`,
  `research.rs`, `chat.rs` (TurnStatus producers :620/656/709),
  `src/tauri/terminal.rs` (PTY), plus REPL `src/cli/cmd_impls/research.rs`.
- Cross-repo duplicate search terms (V01-01): `drive_chat`, `ChatSink`,
  `ChatDriverEvent::TurnStatus`, `ChatDriverEvent::Interrupt`,
  `parallel_tasks`, `TaskProgressEntry`, `workspace`, `worker`,
  `SlashCommand::*`, `list_task_todos/events/artifacts/reviews`,
  `export_conversation`, `research` (TUI surface).

## Out Of Scope

- `drive_chat`/sink semantics, envelope terminal contract, steer mailbox →
  A-CHAT-01 (dependency report; its P1-01/P2-01 consumed as facts).
- TaskRuntime executor/controller behavior (pause-during-wave, orphan
  claims) → A-TSK-03 (dependency report; P1-01 cross-checked for TUI
  reachability only).
- HITL provider arbitration, REPL EOF behavior, scope mapping → A-HITL-01
  (dependency report; P1-02/P1-03 cross-checked for the TUI path only).
- Workspace/config surface policy → A-CFG-01 (its P1-03 is the canonical
  workspace finding; re-verified from the TUI side here).
- Research/analysis workbench placement, connector correctness → A-DOM-01
  (cross-referenced; not re-audited).
- GUI rendering (frontend reducers) → A-SRF-03, A-FE-01/02; Tauri command
  lifecycle → A-SRF-02; CLI/channel surfaces → A-SRF-04.
- Tool exposure/permission-mode gates on the agent → A-TOOL-01, X-AUT-01.
- Framework streaming/cancel defects (F-RCT-03-P1-01/P1-02) are consumed as
  dependency facts, not re-verified.

## Inputs

- Root `AGENTS.md` (multi-mode parity as product positioning; "X doesn't
  use Y" is a gap; UTF-8/panic safety; layering gate; no parallel
  semantics), shared `README.md`, `REPORTING.md`, `TASKS.md` (A-SRF-01
  card), `zcode-ds/README.md`, report templates.
- Dependency task reports read (zcode-ds): `A-CHAT-01` (complete; P1-01
  GUI mislabeling, P2-01 dead Interrupt variant, P3-01 stale main.rs doc),
  `A-TSK-03` (complete; P1-01 pause-during-wave → cancel), `A-HITL-01`
  (complete; P1-02 REPL EOF auto-approve, P1-03 SessionAllTools wildcard,
  P2-01 GUI dispatcher bypass). `A-CFG-01` P1-03 (workspace gap) read as a
  cross-check target per the task instructions.
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md`
  (lines 93, 107, 339, 454, 488-489, 733, 770-771), `echo-agent-cli/docs/
  2026-07-17-surface-parity-closeout.md`, `2026-07-11-tui-parity-design.md`,
  `gui-status.md`, `2026-07-28-app-core-full-audit.md` (no TUI claims found).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework) | None of this task's findings recommend framework movement. The chat driver, ChatSink trait, TaskRuntime store, HITL dispatcher, and envelope contract are framework/application as already classified by A-CHAT-01/A-TSK-03/A-HITL-01. |
| EKO product policy (application) | All TUI findings are application-layer: the workspace surface gap (P1-01) is an EKO surface-policy gap (GUI owns `workspace.rs`; the TUI needs an adapter onto `AppState::switch_workspace`); task-detail browsing (P2-01), research workbench (P2-02), queue drain on Cancelled (P2-03), dead task strip (P3-01), dead TurnStatus arm (P3-02), missing reducer fixtures (P3-03) are all TUI surface behavior. |
| Adapter boundary | `TuiChatSink` is a thin, stateless renderer (events.rs:2031-2210) — no lifecycle/state authority; `dispatch_turn` (events.rs:1341-1436) is a thin adapter over `PreparedUserTurn::build` + `ChatResources` + `drive_chat`, matching the GUI adapter (chat.rs:688). Two dead arms exist because the wire contract has GUI-only producers (TurnStatus, Interrupt) — shared-contract defect owned by A-CHAT-01-P2-01/P3-02. |
| Duplicate search | Terms searched across both repositories (V01-01): `drive_chat` (1 def, 4 call sites), `ChatSink` impls (3 production + 1 test mock), `ChatDriverEvent::TurnStatus` producers (GUI + test only), `ChatDriverEvent::Interrupt` producers (zero), `parallel_tasks`/`TaskProgressEntry` (TUI-only, zero population), `workspace` (TUI zero, GUI full command set), `worker` (zero in `src/tui`), slash-command handlers (66/66 handled), `list_task_todos/events/artifacts/reviews` (GUI-only), `export_conversation` (GUI-only), `research` (GUI + REPL; TUI none). |
| Migration deletion | P3-01 (dead `parallel_tasks`/`TaskProgressEntry`/task_strip widget) and P3-02 (dead TurnStatus arm) are deletion targets once the reducer is factored; P2-03 is a code change in the reducer, not a deletion. |

## Current Path

Verified call graph (V02-01, V03-01):

1. Entry: `main.rs run_tui_or_cli_entry` (default TUI, main.rs:99-101) →
   TaskRuntime store with boot recovery (main.rs:47-76) → task tools
   registered on the primary agent (main.rs:63-68) → `AgentPool::from_runtime`
   + `bind_task_execute_to_pool` (main.rs:69-84) → conversation resume
   from `--resume`/`--continue` (main.rs:96-147) → HITL REPL provider
   unregistered, TUI provider registered (main.rs:250-257) → `run_tui`
   (main.rs:275-315).
2. `run_tui` (mod.rs:1882-2019): builds `TuiApp` with pool/store/scheduler/
   review_integration/conversation_store/conversation_id/configured_models/
   browser_runtime/plugin_runtime; loads stored conversation messages
   (:1967-1991); spawns Dreaming (:1997-2007); enters `run_event_loop`.
3. `run_event_loop` (events.rs:556-976): subscribes to the subagent event
   bus (:562-564); refreshes `task_runtime_view` every 250 ms (:574-577);
   reducer `match` on `AgentEvent` (:602-908) renders tokens/thinking/
   usage/tool stream/tool results/terminals/notices; input events
   (key/paste/mouse) dispatch to handlers; `should_quit` exits.
4. Turn path: Enter → `handle_enter` (events.rs:1294) → slash/steer/`!`
   handling, else `QueuedTurn` → `dispatch_turn` (events.rs:1341-1436):
   `PreparedUserTurn::build` (shared path, attachments + mode hint +
   conversation id + turn id), `ChatResources` (pool, store, sink, conv_id,
   attachments, cancel, mode_hint, interaction_mode, layer_manager),
   `send_to_agent` spawns `drive_chat` (events.rs:2212-2230) into
   `TuiChatSink` (events.rs:2031-2210) — the same driver the GUI uses
   (chat.rs:688). Queued turns drain on FinalAnswer/Error (:657-660,
   :805-843) — but **not** on Cancelled (:661-675, P2-03).
5. Cancel: Esc → `handle_esc` fires `active_cancel.cancel()`
   (:1937-1958); the reducer masks the envelope-fabricated error as
   "Cancelled by user." via `active_cancel.is_cancelled()` (:805-843).
   Double-Esc rewinds the last persisted turn (:1960-2011).
6. Task flows: `/tasks` (projection + sidebar), `/task-cancel|pause|resume`
   → `store.request_cancel/request_pause`/`resume_tui_task_run`
   (:4305-4367, :4696-4755 — resume re-drives `execute_run` with a fresh
   registered token), `/task-recovery`, `/task-retry|skip`
   (:4368-4507).
7. Other capabilities: `/attach` + Ctrl+V paste staging (:3261-3300,
   :1510-1597), `/mcp list|load|disconnect` (:3425-3477), `/browser
   status|managed|chrome` on the shared `BrowserRuntime` (:4551-4615),
   `/resume|/sessions|/fork|/rename|/delete-session` (:2974-3070,
   :4778-4918), `/cron` on the real `SchedulerRunner` (:2232-2298),
   `/worktrees` (:2300-2410), `/skills|/hooks|/plugins|/permission|/model`,
   `/steer` (both turn-id and active-turn variants, :1438-1474, :4231-4304),
   `/plan`, `/mode`, `/analysis`, `/trace`, `/open-artifact`,
   `/memory-*`, `/evolution-*`, `/evidence-inbox`, `/auto-memory`,
   `/pipeline|/test|/code-review|/diff|/git` (agent-prompt dispatch,
   :4632-4688).
8. Terminal contract at the TUI boundary: the envelope yields exactly one
   terminal (FinalAnswer | Cancelled | Error) per turn; the TUI clears
   `is_processing`/`active_cancel`/`active_turn_id` on each and renders a
   truthful message; the `TurnStatus` arm (:880-886) is unreachable on the
   TUI path (no producer — V01-01).

## Findings

### A-SRF-01-P1-01: The TUI has no workspace surface at all — GUI owns a complete workspace command set while TUI comments assert "TUI has no workspace concept" as positioning (independent verification of A-CFG-01-P1-03)

- Priority: P1
- Confidence: high
- Layer: application (EKO surface policy)
- Evidence: `src/tui/commands.rs:58-136` — `SlashCommand` has no workspace
  variant; repository grep of `src/tui/*.rs` finds no workspace switch/exit
  handler (V01-01); TUI comments explicitly state the absence as a fact of
  the product: "TUI has no workspace concept (same as its uploads dir at
  line ~2494)" (`src/tui/events.rs:1368`), "TUI has no workspace concept,
  so use the global ~/.eko/uploads/ dir" (`events.rs:3266`, also `:2526`,
  `:2544`); `dispatch_turn` spills long pastes via
  `resolve_user_input_spill_dir(None)` (`events.rs:1370`). GUI side:
  `src/tauri/commands/workspace.rs:9-235` (list/create/switch/delete/
  link/migrations) wired via `src/tauri/mod.rs`; REPL `/workspace switch`
  is a print-only stub (A-CFG-01-P1-03).
- Reachability: TUI is the default entry (main.rs:99-101); the workspace
  surface is definitionally absent — no command, no key, no sidebar tab;
  `project_files`, prompt assembly, and the uploads/spill dir are fixed to
  the launch CWD for the whole session.
- Expected invariant: workspace switching is available on every Agent
  surface (GUI, TUI, CLI/REPL) per AGENTS.md multi-mode parity; the
  "X mode doesn't use Y" comment pattern is a gap to fill, not a
  positioning statement (AGENTS.md historical lesson).
- Observed behavior: GUI users can switch/create/delete workspaces and
  projects; TUI users cannot switch workspace at all — the working
  directory, project files for `@` completion, hook/plugin roots, and
  attachment upload dir stay pinned to the process CWD; long-paste
  attachments land in the global user-input dir instead of the workspace.
- Impact: a core Agent operating context (workspace/project) is
  GUI-only; TUI users must restart the app from a different directory to
  change project context; attachment/artifact spill paths diverge between
  surfaces (the same divergence A-CFG-01-P1-03 flags). This is the exact
  "TUI lacks a GUI capability with no justification" class the AGENTS.md
  parity rule rates P1.
- Root cause: workspace switching was built for the GUI IPC surface only
  (A-CFG-01-P1-03 root cause); the TUI's "no workspace concept" comments
  were written as a rationale instead of a TODO, which is precisely the
  pattern AGENTS.md warns about.
- Direction: add a TUI workspace surface (`/workspace list|switch|create|
  delete` + `exit`, or a sidebar workspace tab) backed by the same
  `AppState`/workspace-registry service the GUI uses
  (`switch_workspace`, `exit_workspace`), and route the uploads/spill dir
  through the current workspace (`resolve_user_input_spill_dir(Some(ws))`);
  remove the "TUI has no workspace concept" comments. Deletion target:
  none (the comments are the deletion target).
- Regression validation: TUI fixture — create workspace W, `/workspace
  switch W`, assert `state.current_workspace()` and CWD change; `/attach`
  then asserts the staged file lands under W's uploads dir; `@` completion
  lists W's files.
- Validation reports: [V01-01](../validations/A-SRF-01/V01-01.md),
  [V02-01](../validations/A-SRF-01/V02-01.md),
  [V05-01](../validations/A-SRF-01/V05-01.md)

### A-SRF-01-P2-01: Task detail browsing is GUI-only — the TUI cannot inspect a run's todos, events, artifacts, or reviews, only a coarse status projection

- Priority: P2
- Confidence: high
- Layer: application (surface parity)
- Evidence: GUI commands `list_task_todos`, `list_task_events`,
  `list_task_artifacts`, `list_task_reviews`, `get_task_summary`,
  `list_task_runs` (`src/tauri/commands/task_runtime.rs`) have **no TUI
  consumer** (V01-01); the TUI `/tasks` view shows only
  `run_id/goal/status` + per-task `title/status/agent_role`
  (`refresh_task_runtime_view` events.rs:4920-4963,
  `format_task_runtime_view` :4964-4979) plus the last 10 subagent
  summaries; the nearest TUI equivalents are `/task-recovery` (blockers
  only), `/open-artifact` (latest tool artifact), and `/trace` (run
  diagnostics, events.rs:4109-4158), which do not cover task events,
  todos, artifacts-by-task, or reviews.
- Reachability: every task-mode run; the TUI user gets the sidebar tasks
  tab (sidebar.rs render_tasks_list) and `/tasks` text, but cannot drill
  into a task's events/artifacts/reviews.
- Expected invariant: task/Subagent capabilities available on the GUI are
  available on the TUI (AGENTS.md parity); the TUI is a complete Agent
  surface, not a status-summary surface.
- Observed behavior: run-level control (cancel/pause/resume/retry/skip/
  recovery) is fully exposed, but run-level *introspection* (todo list,
  event timeline, artifact list, review results per task) is GUI-only.
- Impact: TUI users cannot audit what a run did (events), what it produced
  (artifacts per task), or how it was reviewed (acceptance) without
  reading files under ~/.eko; parity gap for the task capability.
- Root cause: the GUI task panels were built as Tauri-command-backed
  projections; the TUI task surface was implemented at control-op level
  and never given the corresponding read commands.
- Direction: expose the same read APIs on the TUI (extend
  `TaskRuntimeView`/`/tasks` with per-task artifacts, review/acceptance
  status, event counts, or add `/task-events <run-id>`,
  `/task-artifacts <run-id>` reusing the app-core store queries the GUI
  commands call — the store APIs already exist in app-core; only the
  surface adapters are missing).
- Regression validation: TUI fixture — completed run with artifacts and
  reviews: `/tasks` (or the new subcommand) renders task events/artifacts/
  reviews identical to the GUI's `get_task_run` payload.
- Validation reports: [V01-01](../validations/A-SRF-01/V01-01.md)

### A-SRF-01-P2-02: The research/paper workbench is GUI- and REPL-only — the TUI has no research command surface

- Priority: P2
- Confidence: medium (placement of the workbench as "Agent capability" is
  A-DOM-01/X-SRF-01 territory; the surface absence itself is factual)
- Layer: application (surface parity)
- Evidence: GUI `src/tauri/commands/research.rs` (papers, evidence,
  systematic reviews, Zotero, export) + frontend papers workbench;
  REPL research command surface `src/cli/cmd_impls/research.rs`; the TUI
  `SlashCommand` enum (commands.rs:58-136) has no research variant and
  events.rs has no research handler (V01-01).
- Reachability: definitionally absent on the TUI; reachable on GUI and
  REPL.
- Expected invariant: a capability available on GUI and REPL is available
  on the TUI (AGENTS.md parity; A-DOM-01 keeps the research placement
  decision).
- Observed behavior: TUI users cannot list/create/export papers, reviews,
  or evidence; the research *agent tools* (registered on every agent) are
  still callable via chat, but the management surface is missing.
- Impact: TUI users must switch to GUI/REPL for research document
  management; inconsistent capability set across surfaces.
- Root cause: the research workbench was built for the GUI panels first,
  then a REPL command surface; the TUI surface was never added (same
  pattern as P1-01).
- Direction: add a TUI `/research` command surface reusing the same
  app-core research store APIs the GUI commands call, or explicitly
  scope the workbench as GUI-only in X-SRF-01 with a product decision;
  either way the gap must be resolved, not silently kept.
- Regression validation: TUI fixture — `/research list` after creating a
  paper via GUI returns the same records.
- Validation reports: [V01-01](../validations/A-SRF-01/V01-01.md)

### A-SRF-01-P2-03: The TUI reducer does not drain queued turns on a `Cancelled` terminal — queued work stalls silently after cancel (latent until F-RCT-03-P1-02 is fixed)

- Priority: P2
- Confidence: high (code path deterministic) / medium (currently masked by
  the framework cancel defect)
- Layer: application (TUI reducer)
- Evidence: `FinalAnswer` arm calls `dispatch_next_queued`
  (events.rs:657-660); `Error` arm calls it (:842); the `Cancelled` arm
  (:661-675) only marks tools cancelled, clears state and sets
  `status_msg = "Cancelled"` — **no** `dispatch_next_queued`. Queued turns
  then only dispatch on the next user Enter (`handle_enter` returns early
  on empty input, events.rs:1299-1303, and only `dispatch_turn`/`/steer`
  drain). Masking: F-RCT-03-P1-02 — the ReactAgent stream never emits
  `Cancelled` (cancel ends as NoResponse Err / terminal-less abandon →
  envelope fabricates Error → the `Error` arm drains), so today the queue
  drains via the Error arm in practice.
- Reachability: any user with ≥1 queued turn who presses Esc during the
  active turn — once the framework emits a real `Cancelled` terminal
  (after F-RCT-03-P1-02's fix), the TUI would stall the queue.
- Expected invariant: every terminal drains the FIFO queue identically;
  cancellation of the current turn must not silently strand queued turns.
- Observed behavior: asymmetric — FinalAnswer/Error drain, Cancelled does
  not; after the framework fix the queued turns sit idle with no notice
  (status shows "Cancelled") until the user types anything.
- Impact: queued work silently lost from the user's point of view (still
  in memory, but invisible and stranded); a reducer inconsistency that
  will surface exactly when the framework cancel terminal is fixed.
- Root cause: the Cancelled arm was written before queued turns existed
  (or simply omitted the drain call); the Error arm's presence hid the
  asymmetry.
- Direction: call `dispatch_next_queued` in the `Cancelled` arm
  (events.rs:661-675) after clearing state, mirroring the FinalAnswer/Error
  arms; add a fixture (see P3-03) replaying `Cancelled` with queued turns.
- Regression validation: reducer fixture — queue 2 turns, replay
  `AgentEvent::Cancelled`, assert the first queued turn dispatches
  (is_processing becomes true again with the queued text).
- Validation reports: [V03-01](../validations/A-SRF-01/V03-01.md)

### A-SRF-01-P3-01: The parallel task progress strip is dead UI — `parallel_tasks`/`TaskProgressEntry` are never populated anywhere

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `TaskProgressEntry` (mod.rs:201-218), `TuiApp.parallel_tasks`
  (:334, init :830), layout `ui.rs:38`, widget
  `widgets/task_strip.rs:25,31`; repository-wide grep for
  `parallel_tasks.push` / `TaskProgressEntry {` → **zero hits** (V01-01).
- Reachability: the struct and widget compile; the strip is hidden when
  empty (task_strip.rs:25), so nothing renders; no event handler feeds it.
- Expected invariant: the module doc's "active tasks" sidebar/strip
  reflects live task state (the subagent/TaskRuntime projections already
  exist and work via `subagent_runs` / `task_runtime_view`).
- Observed behavior: the parallel task strip never shows anything; it is
  an unpopulated scaffold.
- Impact: dead code + misleading widget; no user-visible harm today, but a
  maintenance trap (a future contributor may populate it while a second
  projection already exists — duplicate-authority risk).
- Root cause: the strip was scaffolded during the parallel-task work and
  never wired to the event projections.
- Direction: either wire it to `update_subagent_runs`/TaskRuntime events
  or delete `parallel_tasks`, `TaskProgressEntry`, the task_strip widget,
  and the layout reference (AGENTS.md: delete dead code, no dual
  projection).
- Regression validation: after deletion, grep `parallel_tasks` returns
  zero; `cargo test -p echo-agent-cli --lib --locked` stays green.
- Validation reports: [V01-01](../validations/A-SRF-01/V01-01.md),
  [V03-01](../validations/A-SRF-01/V03-01.md)

### A-SRF-01-P3-02: The TUI reducer's `TurnStatus` arm and the sink's `Interrupt` arm are dead — no producer fires them on the TUI path

- Priority: P3
- Confidence: high
- Layer: application (shared wire contract; A-CHAT-01-P2-01's TUI side)
- Evidence: `ChatDriverEvent::TurnStatus` is constructed only by the GUI
  wrapper (`src/tauri/commands/chat.rs:620,656,709`) and a test
  (`surface_contract.rs:191`); `drive_chat` never emits it (V01-01). TUI
  consumers: `TuiChatSink` maps TurnStatus (events.rs:2037) and Interrupt
  (events.rs:2045-2053); the reducer clears turn state on TurnStatus
  (events.rs:880-886). `ChatDriverEvent::Interrupt` has zero producers in
  either repository (A-CHAT-01-P2-01).
- Reachability: arms exist, never fired in the TUI path.
- Expected invariant: reducer/sink arms are reachable or removed;
  MASTER-PLAN:770 "interrupt 不再静默丢失" holds.
- Observed behavior: both arms are dead; the TUI's turn-state clearing
  depends solely on the envelope's agent terminals (which is sufficient).
- Impact: misleading reducer surface; if the GUI-only TurnStatus emission
  pattern is ever copied into `drive_chat`, the TUI arm would start
  clearing state from a second signal (duplicate terminal authority);
  low today.
- Root cause: the shared wire contract carries variants with a single
  producer surface (A-CHAT-01-P2-01 root cause).
- Direction: when A-CHAT-01-P2-01 resolves (route GUI interrupt through
  the shared sink or delete the variant), either produce TurnStatus in
  `drive_chat` for all sinks or delete the TUI arms.
- Regression validation: after the A-CHAT-01-P2-01 resolution, grep
  `ChatDriverEvent::TurnStatus` producers; TUI fixture asserting turn
  state clears exactly once per terminal.
- Validation reports: [V01-01](../validations/A-SRF-01/V01-01.md),
  [V03-01](../validations/A-SRF-01/V03-01.md),
  [V04-03](../validations/A-SRF-01/V04-03.md)

### A-SRF-01-P3-03: No terminal event fixtures exist for the TUI reducer — the agent-event match is inline in the event loop and untested

- Priority: P3
- Confidence: high
- Layer: application (tests)
- Evidence: the reducer is an inline `match` inside `run_event_loop`
  (events.rs:602-908) — not factored into a testable function; the TUI
  test inventory (events.rs:5491-5746, mod.rs:1716-1799,
  commands.rs:352-399, markdown.rs:526+) covers helpers only; the single
  event fixture is `subagent_events_update_live_projection`
  (events.rs:5666-5746, SubagentEvent replay); no fixture replays
  `AgentEvent` terminals (FinalAnswer/Error/Cancelled/ToolCall/
  ToolComplete/ToolResult/TurnStatus) through TUI state transitions (V03).
- Reachability: test gap only — but the task card's required validation
  "terminal event fixtures" has no executable fixture.
- Expected invariant: terminal behavior (one-terminal clear, cancel
  masking, queued-turn drain, tool-status transitions) is pinned by
  fixtures (Q-TST-01 can rely on it); P2-03 would have been caught.
- Observed behavior: no coverage; regressions in the reducer pass the
  suite silently (48 lib tests, all helper-level, V04-01).
- Impact: the TUI's terminal contract is certified only by reading; the
  P2-03 asymmetry and any future reducer change are unprotected.
- Root cause: the reducer grew inline in the event loop (needs `&mut
  TuiApp` + `&AgentHandle`), so tests were written for the extractable
  helpers instead.
- Direction: extract the `AgentEvent` match into a testable function
  (e.g. `apply_agent_event(app, event, agent_tx)` returning an action for
  the loop — dispatch-next/draw decisions stay in the loop), then add
  fixtures: FinalAnswer clears + drains; Error masks cancel vs failure;
  Cancelled clears + drains (P2-03 regression); ToolCall→ToolProgress→
  ToolComplete→ToolResult sequence transitions; ToolOutput UTF-8 bounds
  (already covered by append_bounded tests).
- Regression validation: the new fixtures are green and `cargo test -p
  echo-agent-cli --lib --locked` stays green.
- Validation reports: [V03-01](../validations/A-SRF-01/V03-01.md),
  [V04-01](../validations/A-SRF-01/V04-01.md)

## Cross-Checked Dependency Findings (independent verification, not re-filed)

| Dependency finding | TUI-side verification | Verdict |
|---|---|---|
| A-CHAT-01-P1-01 (error turns labeled completed) | TUI never labels turns from `drive_chat`'s Result; the Error arm renders "Error: …" (events.rs:826-833) and the cancel case masks as "Cancelled by user." (:828-829) — truthful terminal rendering; the `TurnStatus`/Done double-terminal exists only in the GUI wrapper | **Not applicable to TUI**; TUI side correct |
| A-CHAT-01-P2-01 (dead Interrupt variant) | `TuiChatSink` maps Interrupt (events.rs:2045-2053), zero producers — same dead arm on the TUI side | Confirmed (P3-02, shared contract) |
| A-CFG-01-P1-03 (workspace print-only stubs / TUI no surface) | TUI has **no** workspace surface (commands.rs, events.rs; "TUI has no workspace concept" comments at events.rs:1368/2526/2544/3266) | Confirmed → A-SRF-01-P1-01 |
| A-TSK-03-P1-01 (pause-during-wave → permanent cancel) | `/task-pause` → `store.request_pause` (events.rs:4345-4355) — identical broken flow reachable from the TUI | Confirmed reachable; fix owned by A-TSK-03 |
| A-HITL-01-P1-02 (REPL EOF auto-approve + blocking read) | REPL provider unregistered in TUI mode (main.rs:250-257); `TuiHumanLoopProvider` waits for real keys, 300 s timeout → `Timeout` (tui_provider.rs:206-222), fail-closed | **Not applicable to TUI** |
| A-HITL-01-P1-03 (SessionAllTools → `*` wildcard) | TUI `a` key → `ApprovedWithScope { SessionAllTools }` (events.rs:242-244) — one of the four producers | Confirmed (TUI is a producer) |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search across both repositories (driver/sink/reducer/task-strip/workspace/TurnStatus producers/slash handlers/worker) | yes | passed | [V01-01](../validations/A-SRF-01/V01-01.md) |
| V02 | Registration + runtime reachability (main.rs TUI boot → run_tui → drive_chat; task tools; HITL swap; resume/browser/MCP/attachment paths) | yes | passed | [V02-01](../validations/A-SRF-01/V02-01.md) |
| V03 | Invariant/edge cases (terminal symmetry, task strip population, TurnStatus reachability, terminal event fixture inventory, dependency cross-checks) | yes | passed (findings recorded) | [V03-01](../validations/A-SRF-01/V03-01.md) |
| V04 | `cargo test -p echo-agent-cli --lib --locked` (48 ok, exit 0) | yes | passed | [V04-01](../validations/A-SRF-01/V04-01.md) |
| V04 | `cargo check --workspace --locked` (exit 0) | yes | passed | [V04-02](../validations/A-SRF-01/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked surface_contract` (3 ok, exit 0) | yes | passed | [V04-03](../validations/A-SRF-01/V04-03.md) |
| V05 | Historical-document drift (MASTER-PLAN TUI claims, surface-parity-closeout, tui-parity-design, gui-status) | conditional | passed | [V05-01](../validations/A-SRF-01/V05-01.md) |

All required validations executed; every reported command has a known exit
code (V04-01/V04-02/V04-03 exit 0); no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:93 "GUI/TUI/channel 已通过 drive_chat 统一驱动,差异主要在 ChatSink" | current | TUI events.rs:2226 → one TuiChatSink (V02-01) |
| MASTER-PLAN:107 "TUI 已接任务、plan、Subagent、HITL、记忆、附件、Browser/MCP 和会话恢复基础" | current | slash handlers + wiring (V02-01) |
| MASTER-PLAN:339 "新增 terminal 状态时同一提交更新 GUI/TUI/CLI/channel reducer" | current (one dead arm) | TUI reducer has TurnStatus/Interrupt dead arms (P3-02) |
| MASTER-PLAN:488 "TUI 提供 /task-recovery、/task-retry、/task-skip" | current | events.rs:4368-4507 (plus cancel/pause/resume) |
| MASTER-PLAN:489 "TUI approval future 被取消时只清理同一 request" | current | tui_provider.rs PendingCleanup |
| MASTER-PLAN:733/770-771 "删除剩余无理由功能缺口; interrupt 不再静默丢失" | regressed (interrupt, A-CHAT-01-P2-01); open gap (workspace) | dead Interrupt arm (P3-02); no TUI workspace surface (P1-01) |
| surface-parity-closeout:63 "TUI free-form Input auto-approved" | fixed | tui_provider input_mode (A-HITL-01 V05) |
| surface-parity-closeout:72 "reducers 未覆盖 budget/guard/memory" | fixed | sink maps them to notices (events.rs:2170-2183) |
| surface-parity-closeout:76/:117 "TUI /cron 把命令当自然语言" | fixed | handle_tui_cron drives SchedulerRunner (events.rs:2232-2298) |
| 2026-07-11-tui-parity-design:75 "TUI 不并行维护第二份 session JSON" | current | single conversation projection (V02-01) |
| gui-status.md:56 "Unified right workspace" | GUI-only panel; TUI absence is the drift signal for P1-01 | P1-01 |
| main.rs:31 "drive_chat takes Option<&TaskRuntimeStore>" (A-CHAT-01-P3-01) | stale (cross-referenced) | current signature `(agent, &PreparedUserTurn, Arc<ChatResources>)` |

## Coverage And Uncertainty

- All conclusions are static except the three V04 command runs; no live TUI
  session was exercised (read-only review; Q-E2E-01 owns dynamic surface
  scenarios). TUI rendering correctness is inferred from the reducer and
  widget code, not from a PTY run.
- Worktree note: `echo-agent-cli/web-frontend/src/generated/*.ts` (79 files)
  were regenerated by an external process during this review (mtime
  16:48:48, matching a `cargo test` window). Verified non-causality: no
  `build.rs` (only `tauri_build` gated on `gui`, not enabled), no Rust
  source references `web-frontend`, no test writes those files; this task
  wrote only under `docs/comprehensive-review/zcode-ds/`. The regenerated
  files do not affect any reviewed TUI claim (all anchors are in `src/tui`,
  `src/main.rs`, app-core).
- The P2-03 asymmetry (queue not drained on Cancelled) is proven by code
  trace; whether it fires today depends on the framework cancel terminal
  behavior (F-RCT-03-P1-02), which currently masks it — the finding is
  framed as latent.
- The `gui`/`channels` conditional matrices were not compiled in this task
  (Q-CLI-01/Q-GUI-01 own them); GUI-side claims used in the capability
  matrix (workspace.rs, task_runtime.rs, research.rs, conversations.rs
  command presence) are definition-level, not behavior-verified.
- The research workbench's placement as "Agent capability" vs "GUI product
  surface" is A-DOM-01/X-SRF-01 territory; P2-02 records the factual
  surface absence and defers the placement decision.
- The TUI's `!command` local shell vs the GUI's interactive PTY terminal
  (`src/tauri/terminal.rs`) and the GUI's native file dialogs vs the TUI's
  `/attach` + `@` completion are direct-user interactive surfaces, not
  Agent capabilities; listed as matrix rows for X-SRF-01 rather than
  filed.
- `chat_driver.rs`, `prepared_turn.rs`, `chat_resources.rs`, and the
  TaskRuntime store were consumed as dependency facts (A-CHAT-01, A-TSK-03),
  not re-audited.

## Handoff

- Downstream tasks may rely on: the TUI is a full-surface Agent entry
  (same drive_chat, TaskRuntime, HITL dispatcher, browser runtime,
  conversation persistence as GUI — V01/V02); terminal rendering is
  truthful on the TUI (A-CHAT-01-P1-01 does not apply); the TUI is a
  producer of the SessionAllTools wildcard (A-HITL-01-P1-03) and exposes
  the pause-during-wave cancel path (A-TSK-03-P1-01); the workspace gap
  (P1-01) is the only P1 parity violation found; P2-01 (task detail
  browsing), P2-02 (research surface), P2-03 (queue drain on Cancelled,
  latent), P3-01 (dead task strip), P3-02 (dead TurnStatus/Interrupt
  arms), P3-03 (no reducer terminal fixtures).
- Reports to read: this report + V01-01..V05-01; dependency reports
  A-CHAT-01, A-TSK-03, A-HITL-01; A-CFG-01 (P1-03 canonical workspace
  finding); A-DOM-01 (research placement).
- `X-SRF-01` should add rows: workspace switching per surface (TUI
  missing, P1-01), task detail browsing (P2-01), research workbench
  (P2-02), interactive PTY terminal vs `!command` (matrix only), native
  file dialogs vs `/attach` (matrix only), user-driven browser panel vs
  `/browser` (matrix only).
- `Q-TST-01`/`Q-FLT-01` should exercise: reducer terminal fixtures once
  extracted (P3-03), cancel-with-queued-turns after the F-RCT-03-P1-02 fix
  (P2-03), workspace switch from the TUI after P1-01's fix.
- `X-EVT-01` should include the TUI's dead TurnStatus/Interrupt arms in
  its producer-to-all-consumer matrix.
- Stale triggers: this report becomes stale if the TUI event-loop reducer
  (events.rs:602-908) is factored or its terminal arms change; if a
  workspace surface is added to `src/tui` (P1-01 fixed); if
  `parallel_tasks` gains a population site (P3-01 weakens); if
  `drive_chat` starts emitting TurnStatus/Interrupt (P3-02 weakens); if
  F-RCT-03-P1-02 lands (P2-03 becomes live).
- Follow-up task IDs (fixes are not implemented in this review): A-CFG-01
  (workspace wiring), A-CHAT-01 (Interrupt/TurnStatus contract), A-TSK-03
  (pause fix), A-HITL-01 (scope mapping), X-SRF-01, X-EVT-01, Q-TST-01,
  Q-FLT-01, S-RDM-01.
