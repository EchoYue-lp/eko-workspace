# A-SRF-01: TUI integration

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (read-only; consumes `AgentEvent`, `SubagentEvent`,
> `CancellationToken`, `HumanLoopProvider`, `PermissionMode` contracts)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Does the TUI expose and correctly render the complete Agent feature set
rather than a reduced execution path?

## Scope

Primary source paths and behaviors inspected (all under
`echo-agent-cli/` at commit `b3b2e81`, application layer):

- `echo-agent-cli/src/tui/mod.rs` (full, 2019 lines) — `TuiApp` struct
  (every field audited for producer/consumer), `ChatMessage` /
  `MessageRole` / `MessageGroup` / `ToolExecutionMessage` /
  `TaskRuntimeView` / `SubagentRuntimeView` / `TaskProgressEntry` types,
  the `tool_command` / `tool_detail` / `tool_metadata_label` /
  `tool_output_tail` family, `run_tui` entry point, the
  `state_tests` module.
- `echo-agent-cli/src/tui/events.rs` (full, 5746 lines) — `run_event_loop`,
  the 22-variant local `AgentEvent` enum and its 22-arm reducer
  (`:602-907`), `TuiChatSink::on_event` (`:2031-2210`), `handle_enter` /
  `dispatch_turn` (`:1294-1436`), `steer_active_turn` (`:1438`),
  `handle_approval_key` + `send_pending_response` (`:36-252`),
  `handle_slash_command` and all 57 slash branches (`:2612-4655`),
  `refresh_task_runtime_view` (`:4920`), `update_subagent_runs`
  (`:5343-5434`), `resume_conversation` (`:4818`),
  `start_tui_task_run_driver` (`:4716`), `handle_tui_cron` /
  `handle_tui_worktrees`, `reset_conversation_state`, `tool_execution_tests`
  (10 tests at `:381-553`) and `tests` module (12 tests at `:5491-5746`).
- `echo-agent-cli/src/tui/commands.rs` (full, 399 lines) — `SlashCommand`
  enum (57 variants across 9 categories), complete description/usage/
  category tables, 3 unit tests.
- `echo-agent-cli/src/tui/ui.rs` (full, 266 lines) — top-level layout
  (StatusBar / Sidebar+Chat / Input / optional TaskStrip), the inline
  approval-card overlay (`render_approval_card`).
- `echo-agent-cli/src/tui/widgets/{chat,input,sidebar,status_bar,task_strip}.rs`
  (full) — every widget's render path; especially `widgets/chat.rs`
  `build_chat_lines` (5 `MessageRole` arms), `widgets/sidebar.rs` TaskRuntime
  + Subagent render, `widgets/status_bar.rs` context-window ring + cache-hit
  span, `widgets/task_strip.rs` progress strip.
- `echo-agent-cli/src/tui/markdown.rs` (skim, 578 lines) — heading/list/
  code-block/bold/italic rendering and the 6 markdown tests.
- `echo-agent-cli/src/tui/clipboard.rs` (skim, 181 lines) — Linux/X11
  clipboard lease, Ctrl+Y copy path.
- TUI providers in app-core:
  - `echo-agent-cli/echo-agent-app-core/src/hitl/tui_provider.rs` (full
    references via A-HITL-01; `PendingApproval`, `TuiHumanLoopProvider`).
- TUI entry wiring:
  - `echo-agent-cli/src/main.rs:240-345` — `run_tui_or_cli_entry` TUI
    branch, `TuiHumanLoopProvider` install + dispatcher swap,
    `run_tui(...)` invocation with all 14 resources wired.

Framework contracts consumed (read-only):

- `echo-agent/echo-core/src/agent/event_envelope.rs:107-194`
  (`envelope_event_stream`).
- `echo-agent/src/agent/subagent/events.rs:14-248` — the 16-variant
  `SubagentEvent` enum used to audit TUI coverage.
- `echo-agent/src/agent/handle.rs:187` (`steer_input`).

## Out Of Scope

Deferred to downstream / sibling task IDs:

- **A-CHAT-01**: the single chat-turn lifecycle owner and the
  `TuiChatSink` pure-renderer classification (A-CHAT-01's V02 table row).
  This task consumes that as the upstream contract and audits only the
  TUI render side.
- **A-HITL-01**: the `TuiHumanLoopProvider` registration + dispatcher
  swap and the 300s-timeout / fail-closed contract. This task only
  audits the in-TUI render and key-handler for the approval card.
- **A-TSK-03**: `execute_run` / `RuntimeDagExecutor` ownership; the
  TUI's `start_tui_task_run_driver` is confirmed to delegate (see V02).
- **A-SRF-02**: Tauri/GUI command surface. This task references
  A-SRF-02-P2-02 (TUI permission-mode alias drift) and A-SRF-02-P2-03
  (GUI subagent tool-execution bridge) without re-auditing the GUI side.
- **A-STATE-01**: `ConversationStore` backend correctness
  (file/SQLite/atomic-write). This task only verifies the TUI calls the
  store's resume / fork / save APIs.
- **A-INP-01**: `PreparedUserTurn` field matrix. This task treats it as
  the input contract.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` via system reminder. Load-bearing
  sections: multi-mode functional parity ("TUI、GUI(以及 CLI/channel)
  必须功能对等"; "禁止以'某模式不需要'为由拒绝给该模式接入能力";
  "代码里若出现 'X 模式 doesn't use Y' 之类的注释/None 传参,那是
  待补的缺口,不是产品定位"), the framework-vs-application layering
  gate, the "first check if it already exists" rule, the no-panic /
  UTF-8 safety rules, and the Claude Code research rule (TUI must be
  feature-complete per Claude Code parity target).
- `docs/comprehensive-review/REPORTING.md`.
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (A-SRF-01 card and dependencies).

Dependency reports read:

- `zcode-glm/tasks/A-CHAT-01.md` (complete) — establishes
  `drive_chat` as the single chat-turn lifecycle owner and classifies
  `TuiChatSink` as a pure renderer (`:2031-2224`). Load-bearing for V01
  and V02: the TUI sink is the rendering half of the shared driver; it
  has no persistence authority (unlike `TauriChatSink`).
- `zcode-glm/tasks/A-HITL-01.md` (complete) — establishes the
  `TuiHumanLoopProvider` install path (`main.rs:253-254` swaps
  "repl"→"tui") and the 300s shared-deadline + fail-closed invariant.
  Load-bearing for V02: the TUI approval card is the user-facing
  surface for the framework's `PermissionStage`.
- `zcode-glm/tasks/A-TSK-03.md` (complete) — establishes
  `execute_run` as the application's single entry to the framework
  `RuntimeDagExecutor`. Load-bearing for V02: TUI's
  `start_tui_task_run_driver` (`:4716`) is one of the five
  `execute_run` callers enumerated in A-TSK-03 V01.
- `zcode-glm/tasks/A-SRF-02.md` (complete) — establishes the GUI
  command surface (219 commands) and the GUI subagent event bridge
  (`mod.rs:335-769`) that persists subagent tool executions. Used for
  the parity cross-check on subagent detail rendering and on the
  permission-mode alias drift.

Historical documents treated as hypotheses:

- `echo-agent-cli/src/tui/mod.rs:1-13` module docstring — claims the
  TUI provides "Status bar / Sidebar: file tree, tools list, active
  tasks / Chat area / Input box" with "slash command completion". Treated
  as **partially understated**: the surface is in fact much broader
  (57 slash commands across 9 categories); the docstring names only
  the layout, not the capability matrix.
- `echo-agent-cli/src/tui/events.rs:2015-2020` `TuiChatSink` docstring —
  claims "this is the TUI's renderer for the shared `drive_chat` stream
  — the equivalent of GUI's `agent_event_to_chat_event`". Treated as
  **current** (V01 confirms; A-CHAT-01 V02 also classifies TuiChatSink
  as pure-render).

## Layering Decision

This is an **application-layer** task. All inspected code lives in
`echo-agent-cli/src/tui` and the TUI provider in
`echo-agent-cli/echo-agent-app-core/src/hitl/`. The framework is
consumed read-only via `AgentEvent`, `SubagentEvent`, `HumanLoopProvider`,
`CancellationToken`, `PermissionMode`, `AgentHandle::steer_input`, and
the `TaskRuntimeStore` / `BrowserRuntime` / `PluginRuntimeService` /
`ConversationStore` traits.

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework supplies the right primitives: the 22-variant `AgentEvent` stream, the 16-variant `SubagentEvent` bus, the `HumanLoopProvider` trait + `PermissionRequestHandler`, the `AgentHandle` async read/write API, `CancellationToken`, and the per-mode `ChatSink` contract. None of these depend on EKO product decisions. |
| EKO product policy | The 57-entry slash-command surface, the `TuiApp` reducer (the local 22-variant `AgentEvent` enum mapping), the `TaskRuntimeView` / `SubagentRuntimeView` / `TaskProgressEntry` UI projections, the `ToolExecutionMessage` rendering family (`tool_command` / `tool_detail` / `tool_metadata_label`), the `render_approval_card` overlay, the `TuiHumanLoopProvider` pending-slot mechanism, the status-bar context-window ring, the auto-resume of the last conversation at startup — all EKO product policy, correctly in `src/tui` / `echo-agent-app-core`. The framework never references any of these. |
| Adapter boundary | `TuiChatSink::on_event` (`events.rs:2031-2210`) is the thin seam: it pattern-matches each `ChatDriverEvent` and the inner `AgentEvent::payload`, maps to the local `AgentEvent`, and forwards to the mpsc `tx`. It owns no repository, no scheduling, no scheduling authority — pure render projection (confirms A-CHAT-01 V02). The slash commands each call one framework or app-core method (`agent.write`, `agent.steer_input`, `store.request_cancel`, etc.); the body is rendering + glue, not business logic. |
| Duplicate search | Searched both repos for: `TuiApp`, `TuiChatSink`, `run_event_loop`, `handle_slash_command`, `SlashCommand`, `TaskRuntimeView`, `SubagentRuntimeView`, `TaskProgressEntry`, `parallel_tasks`, `update_subagent_runs`, `refresh_task_runtime_view`, `render_approval_card`, `TuiHumanLoopProvider`, `resume_conversation`, `start_tui_task_run_driver`, `stage_attachment`, `handle_tui_cron`, `handle_tui_worktrees`. Result: one canonical TUI per symbol. No second TUI rendering path, no parallel `SlashCommand` enum, no parallel `TuiApp`. The `TaskProgressEntry` struct is unique to the TUI (no GUI analogue) — see P2-01. |
| Migration deletion | A-SRF-01-P2-01 proposes either populating `parallel_tasks` (preferred under parity) or deleting the field + `widgets/task_strip.rs` if product confirms the bottom strip is not wanted. A-SRF-01-P3-01 is a cross-reference (the alias drift is owned by A-SRF-02-P2-02). |

## Current Path

### Verified TUI entry wiring (V01)

```text
main.rs::run_tui_or_cli_entry (main.rs:95)
   │  AgentRuntime::bootstrap (consumed from A-BOOT-01)
   │  pool = AgentPool::new(...)
   │  task_runtime_store = Some(TaskRuntimeStore::new(...))
   │  webhook_emitter, scheduler = start_headless_services(...)
   │  conversation_id = uuid::Uuid::new_v4()
   │  conversation_store = Some(FileBackedConversationStore { ... })
   │
   ├─ if is_tui_entry (feature="tui"):                                 [main.rs:240]
   │     dispatcher.unregister("repl").await                           [:253]
   │     tui_provider = Arc::new(TuiHumanLoopProvider::new())           [:251]
   │     pending = tui_provider.pending_handle()                        [:252]
   │     dispatcher.register("tui", tui_provider).await                 [:254]
   │     (binds plugin_runtime to scheduler)                            [:267-274]
   │     run_tui(agent, &tui_config, "💬 通用", pending, pool,
   │             task_runtime_store, webhook_emitter, scheduler,
   │             review_integration, conversation_store, conversation_id,
   │             configured_models, browser_runtime, prompt_assembly,
   │             plugin_runtime, args.no_alt_screen)                    [:276-304]
   │     on return: shutdown_hook_events + browser_runtime.shutdown()   [:329-334]
   │
   └─ echo-agent-cli::tui::run_tui (mod.rs:1882)
         app = TuiApp::new(model, mode, theme)                          [:1933]
         app.context_window_size = agent.read(...).get_token_limit()    [:1944]
         app.permission_mode = agent.read(...).get_permission_mode()    [:1947-1949]
         app.pool = Some(pool); app.task_runtime_store = ...            [:1952-1953]
         app.scheduler = scheduler; app.review_integration = ...        [:1955-1956]
         app.conversation_id = Some(conversation_id)                    [:1959]
         app.conversation_store = conversation_store                    [:1960]
         app.plugin_runtime = Some(plugin_runtime)                      [:1963]
         app.browser_runtime = Some(browser_runtime)                    [:1964]
         app.project_files = collect_project_files(".", 10_000)         [:1966]
         if store.get_messages(conv_id).await? is non-empty:
             app.messages = restored (role-mapped)                      [:1967-1991]
         agent.write(|a| a.set_conversation_id(conv_id))                [:1992-1994]
         spawn_dreaming_task(review_integration, agent, pool, cancel)   [:1997-2007]
         events::run_event_loop(&mut terminal, &mut app, agent)         [:2010]
         on return: cancel.cancel() (stops dreaming)                    [:2013-2015]
```

**All 14 `TuiApp` resource fields are populated at `run_tui` startup.**
There is no second `TuiApp` constructor and no surface that constructs
its own `run_tui` outside `main.rs::run_tui_or_cli_entry`.

### Verified chat-turn flow (V01, V02)

```text
User types text + Enter
   │
   ↓
handle_enter (events.rs:1294)
   │  text = app.take_input()                                           [:1299]
   │  if text.starts_with('/')  → handle_slash_command                  [:1308-1319]
   │  if text.starts_with('!')  → run_local_shell                       [:1321-1323]
   │  if app.is_processing      → queue                                 [:1331-1336]
   │  else                      → dispatch_turn                         [:1338]
   ↓
dispatch_turn (events.rs:1341)
   │  store.ensure_conversation(NewConversation { ... })                [:1347-1362]
   │  turn_id = uuid::Uuid::new_v4()                                    [:1364]
   │  sink = Arc::new(TuiChatSink::new(agent_tx))                       [:1365-1366]
   │  spill_dir = resolve_user_input_spill_dir(None)                    [:1370]
   │  prepared = PreparedUserTurn::build(...)                           [:1371-1401]
   │  app.start_turn(&display_text)                                     [:1408]
   │  cancel = CancellationToken::new(); app.active_cancel = Some(...)  [:1409-1410]
   │  res = Arc::new(ChatResources { pool, store, sink, webhook_emitter,
   │                                conv_id, root_message_id=turn_id,
   │                                attachments, cancel, mode_hint,
   │                                interaction_mode, layer_manager })  [:1412-1434]
   ↓
send_to_agent (events.rs:2212)
   │  tokio::spawn(drive_chat(agent, turn, res))                        [:2225-2227]
   ↓
drive_chat (chat_driver.rs:202 — see A-CHAT-01)
   │  envelope_event_stream(agent.execute_stream_message_with_invocation_context(msg, cancel, invocation))
   │  for each envelope:
   │     sink.on_event(ChatDriverEvent::Agent(event.payload))           [chat_driver.rs:542-547]
   ↓
TuiChatSink::on_event (events.rs:2031)
   │  maps ChatDriverEvent → local AgentEvent                           [:2035-2206]
   │  self.tx.send(mapped).is_ok()                                      [:2208]
   ↓
run_event_loop poll (events.rs:602)
   │  while let Ok(event) = agent_rx.try_recv() {
   │     match event {
   │        Token(chunk)              → app.append_stream(chunk)        [:604-606]
   │        ThinkStart                → iteration_count += 1            [:607-610]
   │        LlmUsage { .. }           → context_snapshot = ...          [:620-649]
   │        FinalAnswer(_)            → finalize_stream + dispatch_next [:657-660]
   │        Cancelled                 → mark Running tools Cancelled    [:661-675]
   │        ToolCall { .. }           → push ToolExecution msg          [:676-699]
   │        ToolOutput { .. }         → append_bounded into stdout/...  [:706-720]
   │        ToolComplete { .. }       → set Succeeded/Failed + metadata [:721-755]
   │        ToolResult { .. }         → set status; push diff if file   [:756-804]
   │        Error(e)                  → fail running tools + push msg   [:805-843]
   │        ContextCompressed { .. }  → clear_usage + push system msg   [:844-861]
   │        Execution(event)          → status + attention-event msg    [:869-879]
   │        TurnStatus(status)        → status_msg + state clear        [:880-887]
   │        Interrupt { .. }          → push system msg                 [:894-906]
   │     }
   │  }
```

**Single chat lifecycle, single render path.** No parallel TUI chat
stream; confirms A-CHAT-01's V01 conclusion (TUI is one of the four
`drive_chat` callers).

### Verified TaskRuntime + Subagent rendering (V02)

```text
Sidebar Tasks tab + /tasks command:
   run_event_loop every 250ms:                                          [events.rs:574-577]
      refresh_task_runtime_view(app)                                    [:4920]
        store.latest_run_for_conversation(conv_id)                      [:4929]
        store.get_plan(&run.run_id) → tasks                             [:4940-4955]
        app.task_runtime_view = Some(TaskRuntimeView { run_id, goal,
                                        status, tasks })                [:4956-4961]
   render: widgets/sidebar.rs::render_tasks_list (:123-239)
        shows run_id, status badge, goal, per-task [status] title (agent_role)
        plus "Subagents" section if app.subagent_runs non-empty          [:179-235]

Subagent dispatch events:
   run_event_loop pre-poll:                                              [events.rs:570-572]
      while let Ok(event) = subagent_event_rx.try_recv() {
         update_subagent_runs(app, &event)                              [:5343]
      }
   update_subagent_runs handles:
      DispatchStarted        → push/init SubagentRuntimeView { status:"running" }
      DispatchToolStarted    → run.tool_calls = saturating_add(1)
      DispatchCompleted      → status + duration + tokens + apply_subagent_result
      DispatchFailed         → status + apply_subagent_result
      DispatchCancelled      → status="cancelled" + apply_subagent_result
      _ => {} (catch-all — see P2-02)
   cap: drain to last 50                                                [:5430-5433]
```

**Coverage:** 5 of the 16 framework `SubagentEvent` variants drive
visible state changes. The remaining 11 (token deltas, thinking,
LLM usage, tool completion, isolation observed, registry/team
lifecycle) fall into the catch-all `_ => {}` and are silently
dropped — see A-SRF-01-P2-02.

### Verified HITL approval flow (V02)

```text
Agent requests approval (framework PermissionStage → HumanLoopProvider)
   ↓
TuiHumanLoopProvider::request (tui_provider.rs:135)
   │  pending = PendingApproval { kind, prompt, options, response_tx, .. }
   │  *self.pending.lock() = Some(pending)
   │  await oneshot (300s timeout → HumanLoopResponse::Timeout → Deny)
   ↓
run_event_loop draws render_approval_card (ui.rs:107)
   │  pending_handle.try_lock(); approval = guard.as_ref()
   │  render card: title (tool_name), risk_label + prompt, args display,
   │               options (Approval=4 options, Selection=N, Input=text box)
   ↓
Next key event:
   if app.pending_approval.is_some() && has_pending:
      handle_approval_key(app, pending_handle, key) → bool             [events.rs:1045-1054]
   handle_approval_key (events.rs:36-208):
      Approval: y=approve / n=reject-with-reason / m=modify / a=session-all /
                 Left/Right=select / Enter=confirm / Esc=reject
      Input: type feedback, Enter=submit, Esc=dismiss
      Selection: Left/Right/Tab cycle, Enter=select
   send_pending_response (events.rs:211):
      tx.send(HumanLoopResponse::Approved | Rejected | Text | Selection | ApprovedWithScope)
```

**Coverage:** all three `PendingHumanLoopKind` variants (`Approval`,
`Input`, `Selection`) are handled with both option-selection and
free-text-input modes. The render path mirrors the GUI's per-turn
`TauriHumanLoopHandler` (A-HITL-01 V01) but uses the shared dispatcher
(`main.rs:254` registers "tui").

## Findings

The headline result is **strongly positive**: the TUI exposes the full
slash-command surface (57 commands across 9 categories), drives the
shared `drive_chat` lifecycle (single chat path, confirmed by A-CHAT-01),
binds all 14 resource fields at startup, and renders the full 22-variant
`AgentEvent` taxonomy for the foreground path. AGENTS.md's "TUI must be
feature-complete" mandate is met on the primary paths. Two real parity
gaps and two cleanup items are recorded.

### A-SRF-01-P2-01: `parallel_tasks` Vec and the `TaskStrip` widget are scaffolded but never populated — dead UI

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tui/mod.rs:334` — `pub parallel_tasks:
    Vec<TaskProgressEntry>`.
  - `echo-agent-cli/src/tui/mod.rs:830` — `parallel_tasks: vec![]`
    initialization in `TuiApp::new`; the field is never reassigned.
  - `echo-agent-cli/src/tui/ui.rs:37-48` — `task_strip_rows =
    app.parallel_tasks.len().min(5) as u16; let has_tasks =
    !app.parallel_tasks.is_empty();` and the conditional layout
    `if has_tasks { vec![..., Constraint::Length(task_strip_rows)] }`.
  - `echo-agent-cli/src/tui/widgets/task_strip.rs:25-31` — the widget
    early-returns when `app.parallel_tasks.is_empty()` and otherwise
    iterates `app.parallel_tasks.iter().take(max_rows)`.
  - **Repository-wide grep returns zero producers**: `grep -rn
    "parallel_tasks\.push\|parallel_tasks\.insert\|parallel_tasks =\|
    \.parallel_tasks =" echo-agent-cli/src
    echo-agent-cli/echo-agent-app-core/src` returns no hits. The field
    is read in three places (ui.rs:37, events.rs:586, task_strip.rs) and
    written nowhere outside the `vec![]` initializer.
  - The `TaskProgressEntry` struct (`mod.rs:200-218`) carries
    `task_id`, `name`, `status: TaskStripStatus`, `progress_pct`,
    `phase`, `message`, `started_at`, `elapsed_label` — i.e. it was
    designed for live progress reporting, not just status badges.
- Reachability: never reached with content. `has_tasks` is always false
  → the conditional `Constraint::Length(task_strip_rows)` branch is
  dead → the `TaskStrip.render` early-returns. The widget code is
  compiled but visually inert for every TUI session.
- Expected invariant: AGENTS.md multi-mode parity rule — "TUI、GUI
  (以及 CLI/channel) 必须功能对等". The `TaskProgressEntry` /
  `TaskStrip` scaffold implies a planned bottom-strip progress UI that
  the GUI counterpart has (per A-SRF-02 the GUI emits subagent /
  task-execution events to `execution://event` for the frontend's
  dashboard). The TUI ships the scaffold but no producer.
- Observed behavior: the TUI's task progress is shown only in the
  sidebar Tasks tab (via `task_runtime_view` + `subagent_runs`).
  Long-running parallel TaskRuntime waves have no always-visible
  progress strip; the user must press `Ctrl+B` then switch to the
  Tasks tab to see status. For a Claude Code-style TUI (per AGENTS.md
  parity target), a persistent bottom strip is the expected affordance.
- Impact: (a) dead UI scaffold — `TaskProgressEntry`, `TaskStrip`,
  `TaskStripStatus`, and the conditional layout branch in `ui.rs:42-54`
  exist only in the source, never on screen; (b) parity gap — the
  bottom progress strip feature is absent in practice; (c) maintainer
  trap — a future contributor editing `task_runtime_view` may assume
  the strip is wired and waste time debugging why it stays empty.
- Root cause: the scaffold was written before the TaskRuntime
  integration landed; the `task_runtime_view` projection (which shows
  plan tasks, not arbitrary parallel tasks) superseded it as the live
  data source, but the `parallel_tasks` field was never populated from
  the new projection nor deleted.
- Direction: pick one.
  (a) **Populate from TaskRuntimeView** (preferred under parity):
    in `refresh_task_runtime_view` (events.rs:4920) or a sibling
    helper, set `app.parallel_tasks = view.tasks.iter().filter(|t|
    t.status == "running" || t.status == "pending").map(|t|
    TaskProgressEntry { task_id, name: t.title, status: match
    t.status, progress_pct: 0.0, phase: t.agent_role, ..}).collect()`.
    Add `DispatchStarted`/`DispatchCompleted` from `subagent_runs` for
    subagent progress. The strip then mirrors what the GUI dashboard
    shows.
  (b) **Delete the scaffold** (preferred under YAGNI if the sidebar
    Tasks tab is product-sufficient): remove `parallel_tasks`,
    `TaskProgressEntry`, `TaskStripStatus`, `widgets/task_strip.rs`,
    the `TaskStrip.render` call in `ui.rs:98-100`, and the conditional
    branch in `ui.rs:42-54`. The TUI then has one task surface (sidebar)
    and no dead widget.
  Prefer (a) under the AGENTS.md parity rule; (b) is the fallback if
  product confirms the sidebar is enough.
- Regression validation: under (a), a test that builds a
  `TaskRuntimeView { tasks: [running_task, pending_task] }`, calls
  `refresh_task_runtime_view`, and asserts `app.parallel_tasks` has two
  entries with the expected `TaskStripStatus`. Plus a visual scenario
  in the running TUI: create a multi-task run, confirm the bottom strip
  renders rows.
- Validation reports: [V01-01](../validations/A-SRF-01/V01-01.md),
  [V02-01](../validations/A-SRF-01/V02-01.md).

### A-SRF-01-P2-02: Subagent internal lifecycle (token deltas, thinking, per-tool calls, LLM usage) collapses to a counter in the TUI; 11 of 16 framework `SubagentEvent` variants are silently dropped

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tui/events.rs:5343-5434` — `update_subagent_runs`
    explicitly matches `DispatchStarted`, `DispatchToolStarted`,
    `DispatchCompleted`, `DispatchFailed`, `DispatchCancelled`, and a
    catch-all `_ => {}` at `:5428`. Five variants handled.
  - `echo-agent/src/agent/subagent/events.rs:14-248` (read-only) — the
    framework enum declares **16 variants**:
    `DispatchCancelled`, `DispatchCompleted`, `DispatchFailed`,
    `DispatchIsolationObserved`, `DispatchLlmUsage`, `DispatchStarted`,
    `DispatchThinkingDelta`, `DispatchThinkingEnded`,
    `DispatchThinkingStarted`, `DispatchTokenDelta`,
    `DispatchToolCompleted`, `DispatchToolStarted`, `Registered`,
    `TeamCreated`, `TeamDissolved`, `Unregistered`.
  - **Eleven variants fall into `_ => {}`** in the TUI reducer:
    `DispatchToolCompleted` (only Started increments the counter — the
    per-tool result is dropped), `DispatchTokenDelta` (subagent
    streaming tokens never reach the chat), `DispatchThinkingDelta` /
    `DispatchThinkingEnded` / `DispatchThinkingStarted` (subagent
    thinking is invisible), `DispatchLlmUsage` (subagent token usage is
    not accumulated into the status bar's context window snapshot),
    `DispatchIsolationObserved`, `Registered` / `Unregistered`,
    `TeamCreated` / `TeamDissolved`.
  - `echo-agent-cli/src/tui/events.rs:5386-5389` — on
    `DispatchToolStarted`, the only state change is
    `run.tool_calls = run.tool_calls.saturating_add(1)`. The tool name,
    args, and result are not retained. Contrast with the foreground
    path which renders each tool as a full `ToolExecutionMessage`
    (`events.rs:676-804`) with status icon, output tail, and metadata.
  - `echo-agent-cli/src/tui/widgets/sidebar.rs:179-235` — the Subagents
    section in the sidebar renders `agent`, `tool_calls` (counter),
    optional `summary.chars().take(24)`, `artifacts.len()`,
    `verification.len()`, `remaining_work.len()`, `files_read.len() +
    files_written.len()`. The per-tool-call detail is absent; only the
    summary counts surface.
  - Contrast with GUI: A-SRF-02-P2-03 documents that
    `echo-agent-cli/src/tauri/mod.rs:335-769` persists every subagent
    `DispatchToolStarted` / `DispatchToolCompleted` into the shared
    `ToolExecutionRepository`, and emits the per-event payload to
    `execution://event` for the frontend dashboard. The frontend can
    show each subagent tool call with full output.
- Reachability: every subagent dispatch (every `agent_tool` /
  `task_execute` invocation that creates a subagent). Live on every
  TUI session.
- Expected invariant: AGENTS.md multi-mode parity rule — "任何一方有的
  能力...其它方也应有". The GUI exposes per-subagent-tool detail and
  per-subagent LLM usage; the TUI does not. The `SubagentRuntimeView`
  projection (`mod.rs:246-261`) was designed with the right fields
  (`tokens_used`, `tool_calls`, `summary`, `artifacts`,
  `verification`, `remaining_work`, `files_read`, `files_written`) but
  only the terminal summary is populated; the streaming/thinking/usage
  path is dropped.
- Observed behavior: during a subagent run, the TUI shows a counter
  increment (`3 tools` in the sidebar) and the user must wait for
  `DispatchCompleted` to see the summary. There is no live indicator
  of what the subagent is doing, no token usage for the subagent's
  LLM calls, no thinking trace. The status bar's context-window ring
  reflects only the foreground agent's `LlmUsage`, so a long subagent
  run can consume significant context without the ring moving.
- Impact: (a) parity gap — subagent observability is materially lower
  in the TUI than the GUI; (b) the context-window indicator
  undercounts during subagent-heavy turns, which can mislead the user
  about remaining budget; (c) for debugging subagent hangs, the TUI
  user has no streaming signal — only the terminal summary on
  completion/failure.
- Root cause: `update_subagent_runs` was written when the
  `SubagentEvent` enum was smaller (early dispatch lifecycle only);
  the thinking / token / LLM-usage variants were added to the framework
  enum later (for the GUI dashboard) without backfilling the TUI
  reducer.
- Direction:
  1. **Short-term, TUI-only** (independent of the A-CHAT-01-P2-01
     recorder extraction): extend `update_subagent_runs` to handle
     `DispatchLlmUsage` (accumulate into a new
     `subagent_tokens_used` field on `TuiApp` and add to the status
     bar's snapshot), `DispatchThinkingStarted`/`Delta`/`Ended`
     (surface as a one-line "subagent X thinking..." status), and
     `DispatchTokenDelta` (optionally accumulate into a live
     subagent-output panel if product wants it). These do not require
     the GUI's `ToolExecutionRepository` — they are pure TUI state.
  2. **Long-term, shared**: A-CHAT-01-P2-01 + A-SRF-02-P2-03 propose
     extracting tool-execution recording into a driver-level
     `ToolExecutionObserver` / unified recorder. Once that lands, the
     TUI gains per-subagent-tool detail for free by supplying the
     recorder in `ChatResources` (the same path as the GUI). This
     removes the per-tool detail asymmetry without TUI-specific work.
  Until (2) lands, the subagent tool call detail remains GUI-only.
- Regression validation: a test that drives
  `SubagentEvent::DispatchLlmUsage { tokens_used: 100 }` through
  `update_subagent_runs` and asserts the TUI's `subagent_tokens_used`
  accumulator (or the status bar snapshot) increased by 100. A test
  that drives `DispatchThinkingStarted` and asserts a status string
  appears. Plus a scenario: spawn a subagent that calls three tools,
  confirm the sidebar shows live activity (not just a final counter).
- Validation reports: [V01-01](../validations/A-SRF-01/V01-01.md),
  [V02-01](../validations/A-SRF-01/V02-01.md).

### A-SRF-01-P3-01: TUI `/permission` slash command has a reduced alias set relative to GUI/CLI — cross-reference to A-SRF-02-P2-02

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tui/events.rs:3583-3591` — the TUI
    `/permission` canonicalization accepts only `ask|default`,
    `auto|auto-edit`, `full-auto`, `deny|strict`. Missing: `autoedit`,
    `accept-edits`, `auto-approve`, `fullauto`, `bypass`,
    `strict-confirm`, `strict-confirmation`, and the `plan`→`default`
    legacy alias.
  - This is the same evidence already documented in **A-SRF-02-P2-02**,
    which owns the cross-surface unification direction. This finding
    records the TUI-side impact for the A-SRF-01 capability matrix.
- Reachability: every `/permission <alias>` invocation in the TUI.
- Expected invariant: AGENTS.md rule 3 ("one authoritative
  implementation per semantic") + multi-mode parity.
- Observed behavior: a user who types `/permission autoedit` in the
  TUI sees "Unknown permission mode", while the same alias works in
  the GUI panel and the CLI `:set-permissions`.
- Impact: cross-surface drift; documented in A-SRF-02-P2-02.
- Root cause: same as A-SRF-02-P2-02 — canonicalization was never
  lifted into app-core.
- Direction: defer to A-SRF-02-P2-02's recommended
  `PermissionMode::from_alias(&str)` in app-core. The TUI match at
  `events.rs:3583-3591` collapses to one `from_alias(...)?` call.
- Regression validation: covered by A-SRF-02-P2-02's table-driven
  alias test.
- Validation reports: [V01-01](../validations/A-SRF-01/V01-01.md)
  (cross-references the A-SRF-02 V01 grep).

### A-SRF-01-P3-02: No fixture test drives the TUI reducer's terminal event arms (Cancelled / Error / Interrupt / ContextCompressed) — the cancel-render path is regression-unguarded

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tui/events.rs:5491-5746` — the 12 tests in
    `tests` cover `parse_interaction_mode`, `handle_esc` interrupt
    semantics, UTF-8 cursor / delete safety, slash-busy allowlist,
    worktree format, double-Esc rewind, reverse-history search, file
    reference completion, workspace file resolution, `TaskRuntimeView`
    formatting, and one `update_subagent_runs` scenario.
  - `echo-agent-cli/src/tui/events.rs:381-553` — the 10
    `tool_execution_tests` cover only the rendering helpers
    (`tool_command`, `tool_detail`, `tool_metadata_label`,
    `tool_output_tail`, `append_bounded`) for known tool names; they
    do not drive the reducer.
  - The only test that touches cancel is
    `interrupt_cancels_backend_but_keeps_turn_busy_until_settle`
    (`:5529-5541`) — it asserts `handle_esc` behavior (cancel token
    signalled, `is_processing` stays true, status_msg = "Cancelling..."),
    NOT the `AgentEvent::Cancelled` render arm at `:661-675`.
  - `grep -n "AgentEvent::Cancelled\|AgentEvent::Error\|AgentEvent::Interrupt"
    src/tui/events.rs` inside `#[cfg(test)]` returns zero hits — none
    of these arms has a fixture.
- Reachability: every cancel (Esc / Ctrl+C during a turn), every
  mid-stream error (F-RCT-03-P2-01 dropped-terminal → synthesized
  Error), every `Interrupt` event (a queued turn pre-empts a running
  one). Live paths.
- Expected invariant: AGENTS.md validation gate (test what you
  render). Combined with A-CHAT-01-P3-02 (no app-level cancel/error
  fixture for `drive_chat`), the cancel render path is guarded only
  by manual use.
- Observed behavior: tests pass (39 green) but the cancel render arm
  (`mark every Running tool as Cancelled + finished_at + invalidate
  cache + clear active_cancel + status_msg="Cancelled"`) is not
  asserted. A regression that, say, forgot to invalidate the cache
  or to flip tool status would compile and pass.
- Impact: low (the paths run in production every cancel). The cost is
  regression hygield — a future edit to the cancel/error arms could
  silently break the visible state.
- Root cause: the test suite focused on input-parsing and rendering
  helpers (the parts with deterministic input/output); the reducer
  arms that depend on `mpsc::UnboundedReceiver<AgentEvent>` were not
  fixture-tested because the loop is hard to drive in isolation.
- Direction: add a `#[test]` that constructs a `TuiApp`, inserts a
  running `ToolExecutionMessage`, calls the equivalent of
  `agent_rx.try_recv()` logic by inlining the
  `AgentEvent::Cancelled` arm body, and asserts: every Running tool
  flipped to `Cancelled`, `finished_at` set, `is_processing == false`,
  `active_cancel == None`, `status_msg == "Cancelled"`. Repeat for
  `AgentEvent::Error` (assert Running tools flip to Failed, system
  message pushed, status_msg="Error") and `AgentEvent::Interrupt`
  (assert system message with run_id/goal/new_message). The arms are
  currently private fns embedded in `run_event_loop`'s `match`; the
  test may require extracting each arm into a small helper (`fn
  handle_agent_event(app, event)`), which is itself a readability
  win for the 305-line match block.
- Regression validation: the three fixture tests above.
- Validation reports: [V04-01](../validations/A-SRF-01/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | TUI capability/reducer matrix: enumerate every `SlashCommand`, every `AgentEvent` reducer arm, every `TuiApp` field producer | yes | passed (with findings) | [V01-01](../validations/A-SRF-01/V01-01.md) |
| V02 | Task/Subagent/tool/HITL rendering flows traceable end-to-end | yes | passed (with finding) | [V02-01](../validations/A-SRF-01/V02-01.md) |
| V03 | Resume / attachment / browser / MCP reachability via `run_tui`-wired resources | yes | passed | [V03-01](../validations/A-SRF-01/V03-01.md) |
| V04 | Terminal event fixtures: 39-test TUI suite passes; cancel/error/interrupt fixture gap documented | yes | passed (static; fixture gap → P3-02) | [V04-01](../validations/A-SRF-01/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a | No prior A-SRF-01 report under `zcode-glm/`; the two doc-claims (`mod.rs:1-13` layout docstring, `events.rs:2015-2020` sink docstring) are classified inline in the Inputs section. |

Executed cargo command (exit 0):

```text
cd echo-agent-cli && cargo test --features tui -p echo-agent-cli tui::
  → 39 passed; 0 failed; 0 ignored; 9 filtered out
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `src/tui/mod.rs:1-13` — TUI provides "Status bar / Sidebar: file tree, tools list, active tasks / Chat area: streaming messages with markdown rendering / Input box: slash command completion, multi-line input" | partially understated | The layout description is accurate, but the docstring significantly understates the surface breadth — 57 slash commands across 9 categories, full HITL card, TaskRuntime projection, subagent projection, attachment staging, browser/MCP/skills/hooks/plugins/cron/worktrees/analysis commands. |
| `src/tui/events.rs:2015-2020` — `TuiChatSink` "is the TUI's renderer for the shared `drive_chat` stream — the equivalent of GUI's `agent_event_to_chat_event`" | current | V01/V02 confirm: pure-render `on_event`, no repository, single mpsc forward. Aligns with A-CHAT-01 V02's TuiChatSink classification. |
| `src/tui/commands.rs:1-4` — "Slash commands for the TUI command palette. Enum-driven with strum for iteration, string conversion, and parsing." | current | V01 confirms the 57-variant strum enum and the complete category/description/usage/`complete` API. |
| AGENTS.md parity mandate — "TUI 目标是与 GUI 功能对等的完全体(对标 Claude Code)" | mostly current, with two gaps | V01 confirms the broad capability matrix is exposed; V02 finds two parity gaps: `parallel_tasks` dead scaffold (P2-01) and subagent internal detail loss (P2-02). The primary paths (chat, tasks, tools, HITL, resume, attachments, browser, MCP, skills, hooks, plugins, memory, cron, worktrees) are all reachable. |
| A-CHAT-01 handoff — "TuiChatSink / ChannelChatSink only render/transport" | current (load-bearing) | V01/V02 confirm: zero repository mutations from `TuiChatSink`; foreground tool executions are rendered as in-memory `ToolExecutionMessage` entries, never persisted (the GUI-only persistence asymmetry is owned by A-CHAT-01-P2-01 / A-SRF-02-P2-03). |
| A-HITL-01 handoff — "TUI registers `TuiHumanLoopProvider` in the dispatcher; 300s shared deadline; fail-closed" | current | V02 confirms `main.rs:253-254` swap and the `handle_approval_key` + `send_pending_response` paths covering Approval/Input/Selection. |
| A-TSK-03 handoff — "`execute_run` is one of five production entry points; TUI resume via `tui/events.rs:4737`" | current (line drift) | The TUI's `start_tui_task_run_driver` is at `events.rs:4716` (not :4737); it calls `execute_run` at `:4737`. Same caller; minor line drift in the handoff. |
| A-SRF-02-P2-02 — "TUI permission-mode alias set is reduced" | current (cross-reference) | V01 re-confirms at `events.rs:3583-3591`; cross-filed here as A-SRF-01-P3-01, but the fix is owned by A-SRF-02-P2-02. |

## Coverage And Uncertainty

Inspected in full: `tui/mod.rs` (all 2019 lines), `tui/events.rs` (the
5746-line reducer, `TuiChatSink`, slash handlers, `refresh_task_runtime_view`,
`update_subagent_runs`, `resume_conversation`, `start_tui_task_run_driver`,
`reset_conversation_state`, both test modules), `tui/commands.rs` (all
57 variants + 3 tests), `tui/ui.rs` (the layout + approval card),
`tui/widgets/{chat,input,sidebar,status_bar,task_strip,mod}.rs`,
`tui/markdown.rs` (skim of the renderer + 6 tests), `tui/clipboard.rs`
(skim). Cross-repo grep for every TUI symbol; framework `SubagentEvent`
enum cross-check.

Not inspected (out of scope or deferred):

- **`tui/markdown.rs` line-by-line** — skim confirmed the 6 tests cover
  heading/list/code-block/bold/italic/partial-code-block; the renderer
  is consumed as a black box by `widgets/chat.rs::build_chat_lines`. A
  full markdown-rendering audit belongs to a dedicated frontend-render
  task, not the capability-matrix audit.
- **`echo-agent-cli/src/cli/cmd_impls/coding.rs:660-667`** — the CLI's
  `:set-permissions` canonicalization. Already audited in A-SRF-02-P2-02;
  this task cross-references it.
- **The `Dreaming` / `ReviewIntegration` side paths** — `run_tui`
  spawns a dreaming task and the slash commands `/run-review`,
  `/evidence-inbox`, `/memory-review`, `/skill-candidates`,
  `/auto-memory`, `/evolution-dashboard`, `/prompt-diagnostics`,
  `/cost`, `/trace` exercise it. The dreaming memory subsystem itself
  is owned by A-MEM-01; this task confirms only that the slash surface
  is reachable and the runtime is wired.
- **The `AgentPool` / `TaskRuntimeStore` internals** — owned by
  A-BOOT-01 / A-TSK-03. This task confirms the TUI binds them at
  `run_tui` (`mod.rs:1952-1953`) and the slash commands reach them.
- **The conversation store backend** — owned by A-STATE-01. This task
  confirms only that the TUI calls `get_conversation`, `get_messages`,
  `save_messages`, `ensure_conversation`, `update_conversation`,
  `delete_conversation`.

Environmental constraints:

- `cargo test --features tui -p echo-agent-cli tui::` ran against the
  existing incremental cache; 39 tests passed, exit 0. The `tui` feature
  is the only feature gate exercised; no all-features matrix re-run (the
  TUI module is feature-isolated and the test scope is `tui::`).
- No `cargo clean` was needed (disk pressure well below threshold).

Uncertain claims:

- Whether the `parallel_tasks` strip was ever visually rendered in an
  older TUI revision and then orphaned, or whether it was scaffolded
  and never wired. The git history would tell, but the conclusion
  (dead UI today) holds either way.
- Whether the GUI's dashboard actually surfaces subagent thinking /
  token deltas live, or whether it also only shows the terminal
  summary. A-SRF-02 documents the GUI emits the events to
  `execution://event`; whether the frontend renders them live is owned
  by A-SRF-03. If the frontend also collapses them, the parity gap in
  A-SRF-01-P2-02 is smaller than claimed (a documentation gap, not a
  capability gap). The TUI's dropping of `DispatchLlmUsage` is
  independent of that and remains a real undercount.

## Handoff

Conclusions downstream tasks may rely on:

1. **The TUI is feature-complete on primary paths.** Every AGENTS.md
   parity target capability (chat, tasks, subagents, tools, HITL,
   resume, attachments, browser, MCP, skills, hooks, plugins, memory,
   cron, worktrees, analysis, plan/mode/steer, code review, diff,
   external editor, local shell) has a reachable TUI surface. The
   shared `drive_chat` lifecycle (A-CHAT-01) and shared dispatcher
   (A-HITL-01) underpin every TUI turn. Downstream tasks should treat
   the TUI as a first-class surface, not a reduced one.
2. **Two real parity gaps exist** and should drive follow-up:
   - **A-SRF-01-P2-01**: `parallel_tasks` / `TaskStrip` is dead UI
     (zero producers). Either populate from `task_runtime_view` /
     `subagent_runs` or delete the scaffold.
   - **A-SRF-01-P2-02**: subagent internal lifecycle collapses to a
     counter; 11 of 16 `SubagentEvent` variants are dropped. The
     context-window indicator undercounts during subagent-heavy turns.
3. **The TUI is a pure renderer for chat events.** `TuiChatSink` owns
   no persistence authority (A-CHAT-01 V02 row); tool executions are
   in-memory only and discarded on session exit. This means the
   GUI-only `ToolExecutionRepository` persistence (A-CHAT-01-P2-01 /
   A-SRF-02-P2-03) does NOT have a TUI counterpart. The unified
   `ToolExecutionObserver` extraction (when it lands) will give the
   TUI durable tool history for free.
4. **The TUI binds every shared service at startup.** `run_tui`
   (`mod.rs:1882-2019`) takes 17 parameters covering pool,
   task_runtime_store, webhook_emitter, scheduler, review_integration,
   conversation_store, configured_models, browser_runtime,
   prompt_assembly, plugin_runtime. Any task that needs to add a new
   shared capability to the TUI should extend `TuiApp` with a new
   `Option<Arc<...>>` field populated in `run_tui`, not invent a
   parallel wiring path.
5. **The 57-variant `SlashCommand` enum is the TUI capability
   surface of record.** Any task auditing TUI features should start
   from `commands.rs` (the enum) + `events.rs::handle_slash_command`
   (the dispatch). A missing variant in `commands.rs` means a missing
   capability; a missing `Some(SlashCommand::X) =>` arm in
   `handle_slash_command` is a dead variant.

Reports they must read:

- This report (A-SRF-01) for the capability matrix, the parallel_tasks
  dead-code finding (P2-01), and the subagent detail collapse finding
  (P2-02).
- `tasks/A-CHAT-01.md` for the single chat lifecycle that the TUI
  sink renders, and for the GUI-only tool-execution persistence
  asymmetry (P2-01 there).
- `tasks/A-HITL-01.md` for the `TuiHumanLoopProvider` install + 300s
  shared-deadline invariant.
- `tasks/A-TSK-03.md` for the `execute_run` ownership that the TUI's
  `start_tui_task_run_driver` delegates to.
- `tasks/A-SRF-02.md` for the GUI-side parity cross-check (especially
  P2-02 permission-mode drift and P2-03 subagent bridge).

Conditions that make this report stale:

- Any new `SlashCommand` variant in `commands.rs` without a matching
  arm in `handle_slash_command` invalidates V01's "all 57 handled"
  count.
- Any new framework `AgentEvent` or `SubagentEvent` variant not mapped
  in `TuiChatSink::on_event` or `update_subagent_runs` invalidates
  V01's reducer matrix and likely widens P2-02.
- Populating `parallel_tasks` (resolving P2-01 direction (a)) or
  deleting the scaffold (direction (b)) invalidates the corresponding
  V01 / V02 evidence.
- Lifting permission-mode canonicalization into app-core (resolving
  A-SRF-02-P2-02) invalidates A-SRF-01-P3-01's drift evidence.
- Adding the recommended cancel/error/interrupt fixture tests
  (P3-02 direction) invalidates V04's "fixture gap" claim.

Follow-up task IDs (no fixes implemented in this review):

- A **TUI task-strip wiring** task should action A-SRF-01-P2-01
  (populate `parallel_tasks` from TaskRuntime/Subagent views, or
  delete the scaffold).
- A **TUI subagent detail extension** task should action A-SRF-01-P2-02
  (short-term: handle `DispatchLlmUsage` + thinking in
  `update_subagent_runs`; long-term: inherit per-tool detail from the
  unified `ToolExecutionObserver` once A-CHAT-01-P2-01 / A-SRF-02-P2-03
  land).
- A **TUI reducer fixture** task should action A-SRF-01-P3-02
  (extract `handle_agent_event(app, event)` helper, add cancel/error/
  interrupt fixture tests).
- The cross-surface permission-mode unification is owned by
  A-SRF-02-P2-02's follow-up.
