# A-SRF-02: Tauri command and desktop integration

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (read-only; `AgentEvent`, `CancellationToken`, sandbox types)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Are Tauri commands thin, lifecycle-safe adapters with consistent state and no
duplicate business authority?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/src-tauri/src/main.rs` (full, 7 lines) — dedicated GUI
  binary; `tokio::Runtime` + `run_desktop_entry()`.
- `echo-agent-cli/src/tauri/desktop.rs` (full, 272 lines) — `run_desktop_entry`,
  `run_desktop`, panic hook, crash log, window-launch + shutdown sequence.
- `echo-agent-cli/src/tauri/mod.rs` (full, 791 lines) — `build_tauri_app`,
  plugin list, `invoke_handler` registration (219 commands), `setup`
  (browser/subagent event bridges, global shortcut), `task_id_from_subagent_execution_id`.
- `echo-agent-cli/src/tauri/state.rs` (full, 23 lines) — `TauriState`.
- `echo-agent-cli/src/tauri/terminal.rs` (full, 421 lines) — `PtySession`,
  `TerminalManager`, 6 PTY commands, `close_all`.
- `echo-agent-cli/src/tauri/error.rs` — `IpcError` contract (read via grep).
- `echo-agent-cli/src/tauri/commands/chat.rs` (1-210, 442-830, 1148-1411) —
  `ChatEvent` enum, `emit_chat_event`, `emit_execution_event`,
  `emit_tool_execution_summary`, `send_chat_message`, `steer_chat_message`,
  `cancel_chat`, `TauriChatSink` + `ChatSink` impl.
- `echo-agent-cli/src/tauri/commands/conversations.rs` (1-450) — projection
  helpers + 8 conversation commands.
- `echo-agent-cli/src/tauri/commands/panels.rs` (17-210, 859-920) — permission
  mode, audit, sandbox command samples.
- `echo-agent-cli/src/tauri/commands/tools.rs` (full) — tool enable/disable
  lock scoping.
- `echo-agent-cli/src/tauri/commands/hooks.rs` (130-174) — nested-lock path.
- Whole-tree grep for `#[tauri::command]`, `.emit(`, `.write().await`,
  `.read().await`, `close_all`, `on_window_event`.

## Out Of Scope

Deferred to downstream tasks:

- **A-CHAT-01**: `drive_chat` lifecycle ownership and the `TauriChatSink` tool
  execution persistence authority (A-CHAT-01-P2-01). This task consumes that as
  the upstream contract and only audits how Tauri commands assemble the sink.
- **A-STATE-01**: `ConversationStore` backend correctness (file vs SQLite),
  atomic write semantics, and restore round-trip. This task classifies that the
  projection/merge logic is misplaced, not its correctness.
- **A-SRF-03**: the React frontend's consumption of the events emitted here
  (the receive half of the V03 contract).
- **A-TSK-***: `TaskRuntimeStore` internals; this task only audits the
  task_runtime command delegation shape.
- **A-SRF-01**: TUI parity (referenced for the permission-mode triplication
  finding, not re-audited).

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (multi-mode parity rule, "only Subagent no
  Worker", framework-vs-application layering gate, implementation gate rules
  1-6, no-panic / UTF-8 safety rules).
- `docs/comprehensive-review/REPORTING.md`, `templates/task-report.md`,
  `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (A-SRF-02 card and dependencies).

Dependency reports read:

- `zcode-glm/tasks/A-BOOT-01.md` (complete) — establishes that GUI builds
  services exactly once via `AgentRuntime::bootstrap` + `AppState::from_shared`,
  and documents the GUI shutdown sequence (`cancel_token.cancel()` →
  `shutdown_hook_events` → `browser_runtime.shutdown()`). Load-bearing for V04:
  the post-`.run()` cleanup block in `desktop.rs:260-267` is the only shutdown
  path for the GUI, and A-BOOT-01 confirmed no `SchedulerRunner` /
  `BackgroundTaskService` explicit stop there either.
- `zcode-glm/tasks/A-CHAT-01.md` (complete) — establishes that
  `TauriChatSink` owns `ToolExecutionRepository` persistence authority
  (A-CHAT-01-P2-01) and that `drive_chat` is the single chat-turn lifecycle
  owner. Load-bearing for V01: the `send_chat_message` command must delegate the
  turn lifecycle to `drive_chat` and not re-implement it; the
  tool-execution-persistence duplication surface is already known from there.

Historical documents treated as hypotheses:

- `src/tauri/commands/mod.rs:1-7` docstring — claims each command module
  "Deserialize parameters / Call into `echo-agent-app-core` via `AppState` /
  Convert errors to `IpcError` / Return DTOs". Treated as the **thin-adapter
  claim** this task falsifies for several commands.

## Layering Decision

This is an **application-layer** task. All inspected paths live in
`echo-agent-cli/src/tauri`, `echo-agent-cli/src-tauri`, and (for the parity
cross-check) `echo-agent-cli/src/{tui,cli}`. No EKO product concepts were found
leaking into the framework during this audit; the framework supplies only the
primitives (`AgentHandle`, `CancellationToken`, `ConversationStore` trait,
`SandboxManager`, `portable_pty`).

| Classification | Required answer |
|---|---|
| Generic mechanism | Tauri's `invoke_handler` + `app.emit` are the IPC transport; `RwLock`/`DashMap` are the concurrency primitives. All correctly framework-supplied. |
| EKO product policy | The command surface (chat turn assembly, permission-mode canonicalization, conversation UI projection merge, terminal consent, audit log) is EKO product policy, correctly in `src/tauri`. The question is whether it is **thin**, not whether it lives here. |
| Adapter boundary | Commands should be adapters: deserialize → call one app-core service → convert error → return DTO. `steer_chat_message`, `cancel_chat`, `tools::*`, `terminal::*`, `conversations::list_conversations` meet this bar. `send_chat_message`, `conversations::save_conversation`, `panels::set_permissions_mode`, and the subagent event bridge in `mod.rs` exceed it. |
| Duplicate search | Whole-tree searches run for: permission-mode aliases (3 sites — see P2-02); `close_all` (1 definition, 0 callers — see P2-01); `ToolExecutionRepository` persistence (2 sites: `TauriChatSink` + subagent bridge — see P2-03); `ChatEvent` / `emit_chat_event` (single definition — clean); `project_saved_messages` / `pack_ui_projection` (single definition, but in the wrong layer — see P3-01). |
| Migration deletion | No deletion proposed in this review. The findings identify misplaced authority and triplication; resolution is left to follow-up task IDs. |

## Current Path

### Command surface inventory (V01)

`build_tauri_app` (`mod.rs:29`) registers **219 `#[tauri::command]` functions**
in a single `tauri::generate_handler![...]` macro (`mod.rs:69-310`).
Distribution by file (grep `#[tauri::command]`):

| Module | Commands | Notes |
|---|---:|---|
| `commands/panels.rs` | 55 | permissions, audit, auto-memory, skills, workflows, sandbox, context, extraction, review, worktrees |
| `commands/research.rs` | 20 | research papers/evidence/reviews |
| `commands/task_runtime.rs` | 19 | task runs, plans, todos, events, recovery |
| `commands/plugins.rs` | 14 | plugin CRUD + themes/styles |
| `commands/browser.rs` | 11 | browser navigation/screenshot |
| `commands/workspace.rs` | 10 | workspace CRUD + migration |
| `commands/providers.rs` | 8 | model templates + configured models |
| `commands/conversations.rs` | 8 | conversation CRUD + search/export |
| `commands/files.rs` | 7 | file read/write/tree/diff |
| `terminal.rs` | 6 | PTY create/write/resize/close/list/consent |
| `commands/chat.rs` | 6 | send/steer/cancel/approval/input/selection |
| `commands/{mcp,session,analysis}.rs` | 6 each | |
| `ipc.rs` | 5 | native read/write/notify/sysinfo/open |
| `commands/{tasks,scheduler,memory,config}.rs` | 5 each | |
| `commands/tools.rs` | 4 | list/get/enable/disable |
| `commands/hooks.rs` | 4 | list/events/reload/test |
| `commands/tool_executions.rs` | 3 | detail/output/list |
| `commands/mod.rs` | 1 | (registration only) |
| **Total** | **219** | |

Plus 3 non-command native handlers in `ipc.rs` (registered, counted above).

### State and delegation shape (V01)

All commands receive `state: tauri::State<'_, TauriState>` (or
`tauri::AppHandle`) and delegate through `state.app_state: Arc<AppState>`.
`TauriState` (`state.rs:9-23`) holds exactly three fields:
`app_state`, `browser_runtime`, `terminal_manager`. No command constructs an
`AppState` or `AgentRuntime`; all reuse the single long-lived `AppState` built
once in `desktop.rs:187` (confirms A-BOOT-01's "GUI builds services once").

Delegation classification (sampled across all 20 command modules):

- **Thin (deserialize → one service call → DTO)**: `steer_chat_message`
  (`chat.rs:735`, delegates to `agent.steer_input`), `cancel_chat`
  (`chat.rs:807`), all `tools.rs`, all `terminal.rs` commands, `list_conversations`,
  `get_conversation`, `list_memory`, `list_mcp_servers`, `list_plugins`,
  `list_tasks`, `session::get_session`, `files::read_file`, `providers::list_*`.
- **Fat (embeds product policy / orchestration)**:
  - `send_chat_message` (`chat.rs:443`, ~290 lines) — attachment persistence,
    agent routing, cache-user-id setup, in-progress-run interrupt detection,
    DashMap tracking, HITL + browser-approval wiring, sink + projector
    construction, `PreparedUserTurn` build, spawn-with-cleanup. See P3-03.
  - `save_conversation` (`conversations.rs:392`) + the
    `project_saved_messages` family (`conversations.rs:9-150`) — UI-projection
    merge algorithm. See P3-01.
  - `set_permissions_mode` (`panels.rs:39`) — mode-alias canonicalization +
    three-way mutation (config lock + agent + pool). See P2-02.
  - `execute_sandbox` (`panels.rs:860`) — language-alias detection +
    `ResourceLimits` construction.
- **Registration-layer business logic**: the subagent event bridge in
  `mod.rs:335-769` (~420 lines) — `SubagentEvent` → JSON mapping,
  `ToolExecutionRepository` start/finish/cancel persistence, active-tool-id
  tracking, conversation-id resolution. See P2-03.

The chat-turn lifecycle is correctly delegated: `send_chat_message` assembles
`ChatResources` and spawns `drive_chat` (`chat.rs:688`), confirming A-CHAT-01's
single-lifecycle-owner conclusion holds at the command boundary. The fat part
is the **setup/teardown** around `drive_chat`, not a parallel lifecycle.

### Lock and await inspection (V02)

- 76 `.write().await` / `.read().await` / `.lock().await` sites across command
  modules (grep). The dominant pattern is correct: acquire a guard, mutate,
  drop it inside a block `{ ... }` before any further `.await`. Examples:
  `tools.rs:37-42` / `58-63`, `panels.rs:59-61` (`drop(mode_lock)` explicit),
  `state.rs:908-911` (workspace-switch write short-scoped).
- One **lock-held-across-await** case: `save_conversation`
  (`conversations.rs:398-444`) holds the `conversation_store.read()` guard
  across `get_conversation`, `update_conversation`, `get_messages`, and
  `save_messages` awaits. The lock guards only the `Option<Arc<dyn
  ConversationStore>>` wrapper (`state.rs:363`), not the store interior, so
  concurrent readers do not block each other; the contention surface is only
  against the workspace-switch write path (`state.rs:909`). See P3-04.
- One **nested lock** case reviewed and clean: `hooks.rs:152-162` acquires
  `agent.write_async`, then inside the closure acquires
  `a.hook_registry().write().await`. The registry guard is acquired and
  released inside the same closure with only sync calls between
  (`clear_user_hooks`, `register_user_hooks`), so no await-on-held-lock and no
  deadlock cycle.
- No `std::sync::Mutex` held across `.await` was found in the command path.
  `TauriChatSink` deliberately uses `std::sync::Mutex` for
  `tool_completions` / `active_tool_ids` (`chat.rs:1148-1156`) but the guards
  are scoped to sync sections only — the `on_event` impl (`chat.rs:1341-1411`)
  is a sync `fn` (no `.await`), so the std Mutex cannot deadlock the async
  runtime.
- `DashMap` (`session.active_chat_turns`, `session.cancel_token`) is used for
  chat-turn tracking without explicit locking — lock-free, no contention.

No deadlock cycle identified. One minor contention smell (P3-04).

### Event emission contract (V03)

GUI emits to the frontend over four `app.emit` channels:

| Channel | Source | Typing | Discriminator |
|---|---|---|---|
| `chat://event` | `emit_chat_event` (`chat.rs:114-143`) from `TauriChatSink::on_event` | **Typed** — `ChatEvent` enum (`chat.rs:30-112`, 20 variants, `#[serde(tag = "type")]` + per-variant `rename`) | `type` field (e.g. `token`, `final_answer`, `done`) |
| `execution://event` | `emit_execution_event` (`chat.rs:153-183`) + subagent bridge (`mod.rs:703-752`) + `emit_tool_execution_summary` (`chat.rs:185-208`) | **Untyped** — hand-built `serde_json::Map::new()` + `insert(...)` | `kind` field (`run` / `task` / `subagent` / `tool`) + `event` field |
| `browser://event` | `mod.rs:54-66` (broadcast forwarder) | Typed by framework `BrowserEvent` | (opaque framework payload) |
| `terminal-output` / `terminal-exit` | `terminal.rs:154-167` | Typed — `OutputEvent` / `ExitEvent` structs | channel name |

The terminal contract guarantees one `terminal-exit` per session (emitted on
EOF, read error, or process exit — `terminal.rs:144-169`). The chat contract
guarantees a terminal pair: `TauriChatSink::on_event` emits `RunStatus` then
`Done` on every non-running `TurnStatus` (`chat.rs:1365-1387`), so the frontend
always receives a `Done` per turn. Both confirmed.

The split: the two **smaller/simpler** channels (`chat://event`, terminal) are
strongly typed via Rust structs/enums; the **largest/most-complex** channel
(`execution://event`, carrying subagent/tool/run/task lifecycles) is the least
typed — hand-built JSON maps with no shared schema struct, populated by string
keys (`"kind"`, `"subagent_run_id"`, `"run_id"`, `"event"`, `"agent"`, plus
event-specific fields inlined). See P3-02.

Routing metadata (`message_key`, `conversation_id`) is injected at the top level
of the `chat://event` payload outside the `ChatEvent` serde body
(`emit_chat_event:128-140`), so the emitted JSON is a flat merge of routing +
event fields. Functional, but couples the two concerns.

### Window/terminal cleanup (V04)

The GUI has **no window-event handler**. `grep -rn "on_window_event\|CloseRequested\|RunEvent\|prevent_close"` across `src/` and `src-tauri/` returns zero hits. `build_tauri_app` does not register any `.on_window_event(...)`; `tauri.conf.json` has no close/prevention config.

The only cleanup path is the post-`.run()` block in `desktop.rs:260-267`:

```text
tauri_result = build_tauri_app(...).run(tauri::generate_context!());  // blocks until window closed
cancel_token.cancel();                                                 // :261
if let Some(store) = state.tasks.runtime.as_ref() { store.shutdown_hook_events().await }  // :262-265
runtime.browser_runtime.shutdown().await;                              // :267
```

This matches A-BOOT-01's documented GUI shutdown sequence. What is **missing**:

- `state.terminal_manager.close_all()` is never called. `TerminalManager::close_all`
  (`terminal.rs:256-267`) exists, drains every session, and best-effort
  `kill()`s each shell — but it has **zero callers** (grep
  `close_all\|terminal_manager` returns only the definition and the per-command
  `state.terminal_manager.{create,get,remove,list}` uses). Structurally it
  cannot be reached from `desktop.rs` either, because `terminal_manager` is
  owned by `TauriState` (constructed inside `TauriState::new` at `state.rs:20`,
  handed to `.manage(...)` at `mod.rs:39`), not by the `Arc<AppState>` that
  `desktop.rs` holds. See P2-01.

Net effect of P2-01: on every GUI window close, every open PTY shell process is
orphaned (reparented to launchd on macOS), the `pty-reader-{id}` std threads
are terminated with the process (so no thread leak), but the child shells
remain until they exit on their own or the user kills them by PID.

## Findings

### A-SRF-02-P2-01: `TerminalManager.close_all()` is never called on window close — PTY shells orphaned

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/terminal.rs:256-267` — `close_all` definition
    (drains `sessions`, spawns `kill()` per session).
  - `echo-agent-cli/src/tauri/desktop.rs:256-268` — the only GUI cleanup path
    after `.run()` returns; calls `cancel_token.cancel()`,
    `shutdown_hook_events`, `browser_runtime.shutdown()`. No terminal cleanup.
  - `echo-agent-cli/src/tauri/mod.rs:69-310`, `:311-772` — `build_tauri_app`
    registers no `.on_window_event(...)`. `grep -rn "on_window_event"` across
    `src/`, `src-tauri/` returns zero hits.
  - `echo-agent-cli/src/tauri/state.rs:9-23` — `terminal_manager` lives on
    `TauriState`, which is constructed inside `TauriState::new` and handed to
    `.manage(...)`; it is **not** reachable from the `Arc<AppState>` that
    `desktop.rs` holds.
- Reachability: every GUI window close. `run_desktop` → `.run(...)` blocks
  until the main window is closed by the user → the post-`.run()` block runs →
  process exits. At no point is `close_all` (or any per-session `kill`)
  invoked. `close_terminal` (the per-session command, `terminal.rs:402`) is
  only reachable when the frontend explicitly sends it for a single session.
- Expected invariant: desktop app lifecycle must release OS resources (PTY
  pairs, child shell processes) it allocated. AGENTS.md "framework自身 bug 造成
  破坏" / data-loss avoidance: orphaned shells that the user opened in the EKO
  terminal can hold locks, run long jobs, or consume resources after the user
  believes they closed the app.
- Observed behavior: on window close, every live `PtySession`'s child shell is
  left running. The `pty-reader` std threads die with the process, but the
  shell PIDs persist. `close_all` — the exact method designed to drain them —
  has no caller. The structural cause compounds it: `terminal_manager` is
  owned by `TauriState` (Tauri-managed), so `desktop.rs` cannot reach it
  without an `on_window_event` hook inside `build_tauri_app`.
- Impact: resource leak. On macOS, orphaned shells are reparented to launchd
  and keep running (e.g. a `npm run dev` the user started in an EKO terminal
  keeps serving after the app is "closed"). The user has no UI to discover or
  kill them short of `ps`/Activity Monitor. Severity scales with the number of
  terminals the user had open.
- Root cause: the GUI entry (`desktop.rs`) was written before the terminal
  feature, and the terminal was added purely as a command surface
  (`create_terminal` etc.) without wiring its lifecycle into the app's
  shutdown. `close_all` was implemented (defensive) but never hooked up.
- Direction: register an `on_window_event` handler in `build_tauri_app` for
  `WindowEvent::CloseRequested` (or `Destroy`) that pulls
  `app.state::<TauriState>().terminal_manager.close_all()`. Because `close_all`
  spawns `kill()` on a tokio task, either make it awaitable or call the per-
  session `kill().await` synchronously in the handler before allowing close.
  Alternatively, expose `terminal_manager` via `AppState` so the existing
  `desktop.rs` post-`.run()` block can call it (but `on_window_event` is
  cleaner because `.run()` may not return on all platforms' close behaviors).
  Add an integration assertion: open a terminal, close the window, assert the
  child shell PID is gone.
- Regression validation: a test that creates a `PtySession`, calls
  `close_all`, and asserts the child process is reaped. Plus a manual
  scenario: open EKO, open a terminal, run `sleep 1000`, close the window,
  confirm `pgrep -f <shell>` returns nothing.
- Validation reports: [V04](../validations/A-SRF-02/V04-01.md).

### A-SRF-02-P2-02: Permission-mode alias normalization is triplicated (Tauri / TUI / CLI) with drift

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/panels.rs:43-58` — Tauri
    `set_permissions_mode` canonicalization: `"default"`, `"auto-edit" |
    "autoedit" | "accept-edits" | "auto-approve"` → `"auto-edit"`, `"full-auto"
    | "fullauto" | "bypass"` → `"full-auto"`, `"auto" | "plan"` → `"default"`,
    `"strict" | "strict-confirm" | "strict-confirmation"` → `"strict"`.
  - `echo-agent-cli/src/cli/cmd_impls/coding.rs:663-667` — CLI
    `:set-permissions` canonicalization: same full alias set as Tauri.
  - `echo-agent-cli/src/tui/events.rs:3583-3591` — TUI canonicalization: a
    **reduced** alias set — only `"auto" | "auto-edit"` → `"auto-edit"`,
    `"full-auto"` → `"full-auto"`, `"deny" | "strict"` → `"strict"`. **Missing
    from TUI**: `autoedit`, `accept-edits`, `auto-approve`, `fullauto`,
    `bypass`, `strict-confirm`, `strict-confirmation`, and the `plan`→`default`
    legacy alias.
- Reachability: every permission-mode change in any of the three surfaces.
  Tauri: frontend sends `set_permissions_mode` (`panels.rs:39`). TUI: `/permissions`
  slash command (`events.rs:3580` region). CLI: `:set-permissions` REPL command
  (`coding.rs:660` region).
- Expected invariant: AGENTS.md implementation gate rule 3 — "同一种…语义只能
  有一个权威实现" (one authoritative implementation per semantic). The
  multi-mode parity rule requires all three surfaces to accept the same mode
  vocabulary.
- Observed behavior: three independent match blocks canonicalize the same
  permission-mode vocabulary. The TUI block has drifted: it accepts a strictly
  smaller alias set than Tauri/CLI. A user who types `/permissions autoedit`
  in the TUI gets "Unknown permission mode", while the same alias works in the
  GUI panel and the CLI. The drift is silent (no compile error, no shared
  test).
- Impact: (a) mode-vocabulary divergence between surfaces — a user moving
  between GUI and TUI hits inconsistent accepted aliases; (b) any future alias
  addition (or a renamed mode) must be applied in three places, inviting
  repeat drift; (c) the canonical four-mode set (`default`/`auto-edit`/`full-auto`/`strict`)
  is not expressed as an enum, so the strings flow untyped through config,
  agent, and pool.
- Root cause: permission mode was originally a string field; each surface
  added its own alias table when adding its own setter. The canonicalization
  was never lifted into app-core.
- Direction: introduce `PermissionMode::from_alias(&str) -> Result<PermissionMode, …>`
  (or `parse`) in `echo-agent-app-core`, with the full alias table (the union
  of all three current tables). All three surfaces call it; the result is the
  single source of truth for both canonicalization and the valid-mode error
  message. Add a shared test matrix covering every alias. After migration, the
  three match blocks collapse to one `PermissionMode::from_alias(...)?` call.
- Regression validation: a table-driven unit test in app-core covering every
  alias across all three current tables mapping to the four canonical modes;
  the TUI/CLI/Tauri callers each get a thin-wrapper test.
- Validation reports: [V01](../validations/A-SRF-02/V01-01.md).

### A-SRF-02-P2-03: Subagent event bridge in `mod.rs` is a second tool-execution-persistence authority, parallel to `TauriChatSink`

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/mod.rs:335-769` — the `setup` block spawns a
    tokio task that subscribes to `agent.subagent_registry().event_bus()`,
    then for `DispatchToolStarted` calls `tool_executions.start(owner, ...)`
    (`:416-435`), for `DispatchToolCompleted` calls
    `tool_executions.finish(&owner, call_id, ...)` (`:459-484`), and on
    `DispatchCompleted/Failed/Cancelled` drains `active_tool_ids_by_execution`
    and calls `tool_executions.cancel(&owner, &call_id)` (`:506-529`). It
    maintains its own `active_tool_ids_by_execution` bookkeeping and its own
    `subagent_context_by_execution` conversation-id resolver.
  - `echo-agent-cli/src/tauri/commands/chat.rs:1193-1340` — `TauriChatSink::handle_tool_event`
    performs the **same** `tool_executions.start / append_output / finish /
    cancel` state machine for the foreground-agent tool calls, with its own
    `active_tool_ids` + `tool_completions` bookkeeping. This is the
    A-CHAT-01-P2-01 authority.
  - The two sites share the same `Arc<ToolExecutionRepository>`
    (`state.app_state.storage.tool_executions`) but each independently
    maintains an in-memory call-id set and each independently maps events to
    `start/finish/cancel`. There is no shared "tool-execution recorder" type.
- Reachability: every GUI process. The bridge task is spawned unconditionally
    in `setup` for every GUI launch. Every subagent-dispatched tool call is
    persisted through it; every foreground-agent tool call is persisted
    through `TauriChatSink`. Both are live.
- Expected invariant: AGENTS.md rule 3 — "同一种动态…调度…只能有一个权威
  实现". The tool-execution recording state machine (start → append → finish |
  cancel) should have one owner. A-CHAT-01-P2-01 already flagged that the
  sink owns this authority (and recommended extracting it to a driver-level
  observer); this finding adds that there is a **second** implementation for
  the subagent path, in the registration layer, that must also be unified.
- Observed behavior: two parallel implementations of the
  start/append/finish/cancel recorder exist. They diverge in detail: the sink
  records `append_output` chunks (`chat.rs:1248`) and tracks
  `tool_completions` (`PendingToolCompletion`), while the bridge does not
  record streaming chunks at all (only start/finish/cancel). So a subagent
  tool's streaming output is never persisted as `append_output`, while a
  foreground tool's is — an asymmetry the frontend tool-detail panel will
  observe as "subagent tool output appears only on completion".
- Impact: (a) parallel authority — the recording state machine is implemented
  twice with drift (streaming-chunk recording is present in one, absent in the
  other); (b) the bridge is ~420 lines of business logic (event mapping,
  persistence, conversation-id resolution, sequence numbering for usage
  events) in what should be a thin registration/setup function, making
  `build_tauri_app` hard to audit; (c) any change to the recording contract
  (e.g. adding a new terminal state, or changing the owner shape for the
  `Subagent` owner variant) must be applied in both places; (d) the
  A-CHAT-01-P2-01 fix (extract a `ToolExecutionObserver`) cannot fully resolve
  the duplication unless it also subsumes this bridge.
- Root cause: subagent tool dispatch was added as a separate event stream
  (`SubagentEventBus`) from the foreground chat sink, and the recording was
  wired directly into the bridge instead of routing through the same recorder
  as the foreground path.
- Direction: unify on a single `ToolExecutionRecorder` (the same target as
  A-CHAT-01-P2-01's recommended `ToolExecutionObserver`). The recorder takes
  `Arc<ToolExecutionRepository>` + owns the call-id bookkeeping, and exposes
  one method per event class (`on_tool_start`, `on_tool_output`,
  `on_tool_finish`, `on_tool_cancel`). `TauriChatSink::handle_tool_event`
  becomes a thin delegate; the subagent bridge calls the same recorder per
  `DispatchTool*` event. The bridge itself should shrink to event-routing
  only (mapping `SubagentEvent` → `execution://event` payload), shedding the
  persistence calls and the `active_tool_ids_by_execution` map. After this,
  `grep "tool_executions\.start\|tool_executions\.finish\|tool_executions\.cancel"
  src/tauri` should return exactly one call site (inside the recorder).
- Regression validation: a test driving a subagent tool call through the
  bridge AND a foreground tool call through the sink, asserting both produce
  the same `ToolExecutionSummary` shape (start + append_output + finish) in
  the repository; today the subagent path lacks `append_output`.
- Validation reports: [V01](../validations/A-SRF-02/V01-01.md),
  [V03](../validations/A-SRF-02/V03-01.md).

### A-SRF-02-P3-01: Conversation UI-projection merge algorithm lives in the Tauri command module, not app-core

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/conversations.rs:9-150` — four free
    functions: `pack_ui_projection` (builds `AttachmentsPayload` from
    `SavedMessage`), `is_framework_projection` (detects canonical-transcript
    marker `"_echo_message_version"`), `merge_projection_json` (merges
    canonical + UI JSON objects), `project_saved_messages` (the alignment
    algorithm: when a canonical transcript exists, aligns user/assistant UI
    messages to existing positions by role, skipping trimmed prefixes; otherwise
    projects UI messages directly). ~140 lines plus 3 tests (`:152-330`).
  - `echo-agent-cli/src/tauri/commands/conversations.rs:437` —
    `save_conversation` calls `project_saved_messages(&conversation_id,
    messages, &existing_messages)` inline.
- Reachability: every `save_conversation` IPC call from the frontend.
- Expected invariant: the command layer should deserialize and delegate
  (`commands/mod.rs:1-7` docstring claim). A 140-line position-alignment merge
  algorithm that decides how UI messages coexist with the framework's
  canonical transcript is product business logic, not IPC adaptation.
- Observed behavior: the merge algorithm is defined and tested entirely inside
  the Tauri command module. It is not reachable from the TUI or CLI (grep
  `project_saved_messages` returns only this file), so the TUI/CLI cannot save
  UI projections — a parity gap, but more importantly the logic has no shared
  owner. The algorithm also encodes a non-trivial contract (canonical wins for
  content; UI merges in only message-id + thinking + steps + rounds +
  attachments metadata) that should be explicit and reusable.
- Impact: (a) misplaced authority — a command module owns a transcript-merge
  algorithm; (b) the algorithm cannot be reused by a future TUI save path or
  by an import/export tool without duplicating it; (c) the
  canonical-vs-UI-projection contract is implicit in helper code rather than
  documented in app-core where the `SavedMessage` / `StoredMessage` types live.
- Root cause: the GUI was the only surface persisting conversations when the
  projection logic was written, so it was inlined. The
  framework-vs-application layering gate (decide before implementing) was not
  applied.
- Direction: lift `project_saved_messages` (and its three helpers) into
  `echo-agent-app-core` (e.g. a `conversation_projection` module next to the
  `SavedMessage` / `AttachmentsPayload` types), expose a single
  `project_ui_messages(conv_id, ui_messages, existing) -> Vec<StoredMessage>`,
  and have `save_conversation` call it. Move the three existing tests with it.
  This also unblocks a future TUI save-with-projection path.
- Regression validation: the three existing tests (`:171-330`) move and pass
  unchanged; add a round-trip test that `save_conversation` → `get_conversation`
  preserves both canonical content and UI metadata.
- Validation reports: [V01](../validations/A-SRF-02/V01-01.md).

### A-SRF-02-P3-02: `execution://event` channel is untyped while `chat://event` is typed — inconsistent emission contract

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/chat.rs:30-112` — `chat://event` uses
    the `ChatEvent` enum: 20 variants, `#[serde(tag = "type")]`, per-variant
    `#[serde(rename = "...")]`, strongly typed fields. `emit_chat_event`
    (`:114-143`) serializes the enum and merges in routing metadata.
  - `echo-agent-cli/src/tauri/commands/chat.rs:153-183` — `emit_execution_event`
    builds the payload by hand: `let mut map = serde_json::Map::new();
    map.insert("kind".into(), …); map.insert("run_id".into(), …); …;
    app.emit("execution://event", Value::Object(map))`. No shared struct.
  - `echo-agent-cli/src/tauri/mod.rs:703-752` — the subagent bridge hand-builds
    the same channel's payload: `let mut payload = serde_json::Map::new();
    payload.insert("kind".into(), "subagent".into()); payload.insert("task_id"…);
    …` with conditional field insertion.
  - `echo-agent-cli/src/tauri/commands/chat.rs:185-208` —
    `emit_tool_execution_summary` reuses `emit_execution_event` for `kind="tool"`,
    forwarding the `ToolExecutionSummary` serde payload as the field bag.
- Reachability: every subagent/tool/run/task lifecycle event the frontend
  renders. `execution://event` is the unified channel for the
  execution/dashboard UI (per the `mod.rs:335-345` comment).
- Expected invariant: a single emission contract. Either all channels are
  typed via serde structs/enums (preferred — compile-time field check,
  refactor-safe), or all are dynamic. The current split means the
  highest-cardinality channel (subagent/tool/run/task with dozens of
  event-specific fields) has the weakest schema.
- Observed behavior: `chat://event` field renames are compiler-checked (a typo
  in a `#[serde(rename = …)]` is a compile error); `execution://event` field
  names are strings typed by hand in three locations, and a typo
  (`"subagent_run_id"` vs `"subagent_run_Id"`) compiles fine and silently
  breaks the frontend's grouping. The `kind` discriminator accepts four
  strings (`"run"`, `"task"`, `"subagent"`, `"tool"`) with no enum.
- Impact: (a) maintainability — frontend and backend must agree on string
  field names with no shared schema; (b) refactor hazard — renaming a field on
  the backend requires grepping string literals; (c) the
  `emit_execution_event` helper accepts an opaque `serde_json::Value` payload,
  so callers can inline arbitrary fields with no validation.
- Root cause: `execution://event` evolved by accumulating
  `SubagentEvent`-variant-specific JSON in the bridge; no one paused to lift
  the payload shape into a struct.
- Direction: introduce an `ExecutionEvent` enum (mirroring `ChatEvent`) with
  variants per `kind` (`Run`, `Task`, `Subagent`, `Tool`), each carrying a
  typed payload struct. Refactor `emit_execution_event` to take
  `&ExecutionEvent`. The subagent bridge's match becomes
  `SubagentEvent → ExecutionEvent::Subagent(SubagentPayload{…})`. Keep the
  `kind`/`event`/`subagent_run_id` JSON shape for frontend compatibility; the
  enum is purely a backend-side compile-time check via serde derive.
- Regression validation: a snapshot test asserting the `execution://event`
  JSON shape per variant is unchanged after the refactor; a grep confirming no
  `serde_json::Map::new()` remains in the emission path.
- Validation reports: [V03](../validations/A-SRF-02/V03-01.md).

### A-SRF-02-P3-03: `send_chat_message` is a ~290-line fat orchestration command, not a thin adapter

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/chat.rs:442-731` — the command body:
    persist attachments + build refs (`:458-477`), agent routing
    (`agent_for` vs `primary_agent`, `:488-492`), cache-user-id injection
    (`:500-509`), in-progress-run interrupt detection + `InterruptPrompt` emit
    (`:516-534`), `active_chat_turns` DashMap entry insert (`:536-554`),
    cancel-token registration (`:556-566`), HITL handler attachment +
    browser-approval provider wiring (`:568-589`), interaction-mode read
    (`:591-598`), sink + `TauriExecutionProjector` construction (`:604-617`),
    `PreparedUserTurn::build` (`:630-663`), `ChatResources` assembly
    (`:664-680`), `tokio::spawn(drive_chat + cleanup + status emit)`
    (`:681-725`).
  - Contrast: `steer_chat_message` (`:735-803`) is ~70 lines and delegates
    cleanly to `agent.steer_input`; `cancel_chat` (`:807`) is ~25 lines.
- Reachability: every GUI chat send.
- Expected invariant: the `commands/mod.rs:1-7` docstring — commands
  deserialize, call one app-core service, convert errors, return a DTO.
- Observed behavior: `send_chat_message` embeds substantial product policy
  (interrupt detection, attachment persistence, HITL/browser-approval
  attachment, cache-user-id). Some is genuinely GUI-specific (the
  `InterruptPrompt` event, the `TauriHumanLoopHandler` attachment). Some is
  cross-cutting and duplicable: attachment persistence (`:458-477`) is repeated
  nearly verbatim in `steer_chat_message` (`:751-761`); the
  `active_chat_turns` / `cancel_token` bookkeeping pattern is GUI-local but
  parallels what the TUI does in `handle_enter`. The interrupt detection
  (`find_in_progress_run_by_conversation` + `InterruptPrompt`) is product
  policy that the TUI/CLI do not have at all.
- Impact: (a) the command is hard to audit for correctness because ten
  concerns are interleaved; (b) the attachment-persistence block is
  copy-pasted into `steer_chat_message`; (c) the interrupt-prompt feature is
  GUI-only, a soft parity gap (TUI/CLI silently start a second turn instead
  of prompting). Low severity because the lifecycle itself is correctly
  delegated to `drive_chat` (no parallel authority) — the fat part is setup.
- Root cause: the command accreted concerns as features (attachments,
  interrupt, HITL, browser approval) were added, each inlined rather than
  lifted to a helper.
- Direction: extract reusable chunks into app-core helpers: (a)
  `persist_attachments_for_turn(attachments, ws_root) -> Vec<AttachmentRef>`
  (shared by `send_chat_message` + `steer_chat_message`); (b) a
  `begin_chat_turn(state, conv_id, message_key) -> TurnHandle` that owns the
  DashMap entry + cancel-token registration + cleanup-on-drop, returning a
  guard; (c) optionally, lift the interrupt-detection into a shared helper so
  TUI/CLI can adopt it for parity. The command body shrinks to: validate →
  persist attachments → begin turn → assemble resources → spawn drive_chat.
- Regression validation: existing chat-flow tests pass; new unit tests for the
  extracted helpers (attachment round-trip; begin_turn cleans up on drop).
- Validation reports: [V01](../validations/A-SRF-02/V01-01.md).

### A-SRF-02-P3-04: `save_conversation` holds the `conversation_store` read lock across multiple awaits

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/conversations.rs:398-444` —
    `let store_guard = state.app_state.storage.conversation_store.read().await`
    at `:398`, then `store.get_conversation(&id).await` (`:411`),
    `store.update_conversation(...).await` (`:414`),
    `store.get_messages(...).await` (`:434`), `store.save_messages(...).await`
    (`:441`) all execute while `store_guard` is live. The guard is not dropped
    until function return.
  - `echo-agent-cli/echo-agent-app-core/src/state.rs:363` — the field is
    `RwLock<Option<Arc<dyn ConversationStore>>>`; the guard only pins the
    `Option`.
  - `echo-agent-cli/echo-agent-app-core/src/state.rs:909` and `:1081` — the
    only writers, both short-scoped (`{ let mut guard = …; *guard = …; }`).
- Reachability: every concurrent `save_conversation` call (the frontend
  auto-saves on a timer and on turn completion).
- Expected invariant: avoid holding async locks across long `.await` chains
  where a writer may be waiting.
- Observed behavior: the read guard on the `Option` wrapper is held for the
  full duration of four store I/O awaits. Because `RwLock` permits concurrent
  readers, parallel saves do not block each other on this lock (the inner
  store handles its own concurrency). The real contention surface is a
  workspace switch (`state.rs:909`) trying to swap the `Option` — it will wait
  for all in-flight saves to drain. No deadlock (the writer is short-scoped
  and the readers don't re-enter the lock), but a slow store (large
  conversation, slow disk) delays workspace switching.
- Impact: low. Worst case a workspace switch stalls until a save completes.
  No correctness issue.
- Root cause: the command clones the `Arc` once and drops the guard, vs.
  holding the guard for convenience.
- Direction: clone the `Arc<dyn ConversationStore>` out of the guard and drop
  the guard before the first `.await`:
  `let store = { let g = …read().await; g.as_ref().ok_or(…)?.clone() };` then
  use `store` directly. One-line-per-call fix.
- Regression validation: existing `save_conversation` tests pass; add a test
  asserting `conversation_store.write().await` is not blocked by an in-flight
  `save_conversation` (or assert the guard is dropped before the first store
  call via a debug assert / type-level check).
- Validation reports: [V02](../validations/A-SRF-02/V02-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Command-to-service map: 219 commands, delegation vs duplicated business logic | yes | passed (with findings) | [V01-01](../validations/A-SRF-02/V01-01.md) |
| V02 | State/lock/await inspection: lock contention, long-held locks, deadlocks | yes | passed | [V02-01](../validations/A-SRF-02/V02-01.md) |
| V03 | Event emission contract: channels, typing, terminal guarantees | yes | passed (with finding) | [V03-01](../validations/A-SRF-02/V03-01.md) |
| V04 | Window/terminal cleanup: `close_all` reachability, window-event handlers | yes | passed (with finding) | [V04-01](../validations/A-SRF-02/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a | No prior A-SRF-02 report under `zcode-glm/`; the one historical claim (`commands/mod.rs` thin-adapter docstring) is classified inline in the Inputs and Findings (P3-01/P3-03 partially falsify it). |

No cargo command required: this is a static-inspection review of adapter
correctness. The chat-driver / app-core tests already ran in A-CHAT-01 and
A-BOOT-01; no new executable claim is made here.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `commands/mod.rs:1-7` — "each module Deserialize parameters / Call into app-core via AppState / Convert errors to IpcError / Return DTOs" (the thin-adapter claim) | partially overstated | Most commands (~210 of 219) are thin; `send_chat_message` (P3-03), `save_conversation` + projection helpers (P3-01), `set_permissions_mode` (P2-02), `execute_sandbox`, and the subagent bridge in `mod.rs` (P2-03) exceed the bar. |
| A-BOOT-01 GUI shutdown sequence (`desktop.rs:261-267`: cancel → shutdown_hook_events → browser_shutdown) | current | Re-confirmed verbatim at `desktop.rs:260-268`; this task adds that terminal cleanup is absent from that sequence (P2-01). |
| A-CHAT-01-P2-01 (`TauriChatSink` owns tool-execution persistence) | current (load-bearing) | Re-confirmed; this task adds that the subagent bridge in `mod.rs:335-769` is a second, parallel persistence authority for the subagent path (P2-03). |
| `mod.rs:1-4` — "All business logic goes through Tauri IPC — no embedded Axum server needed" | current (transport) but understated on logic placement | The transport is IPC-only (confirmed); however "business logic goes through IPC" does not mean the IPC layer is thin — ~420 lines of business logic live in `mod.rs` itself (P2-03). |

## Coverage And Uncertainty

Inspected in full: `desktop.rs`, `state.rs`, `terminal.rs`, `mod.rs`
(including the full 435-line subagent bridge), `chat.rs` event/sink/send/steer/cancel
regions, `conversations.rs` projection helpers + save/get, `panels.rs`
permission/audit/sandbox samples, `tools.rs`, the `hooks.rs` nested-lock path,
`commands/mod.rs`. Whole-tree greeps for `#[tauri::command]`, `.emit(`,
`.write().await`/`.read().await`/`.lock().await`, `close_all`,
`on_window_event`, permission-mode aliases.

Not inspected (out of scope or deferred):

- The 55 `panels.rs` commands beyond the permission/audit/sandbox samples
  (skills, workflows, context compression, extraction, review, worktrees).
  Spot-checked `execute_sandbox` (`:859`) and `set_permissions_mode` (`:39`);
  both embed product policy. The remaining panels commands are likely a mix of
  thin delegates and fat adapters following the same pattern — a full
  per-command audit would char a new task (A-SRF-02 only samples).
- The 20 `research.rs` commands and 14 `plugins.rs` commands beyond confirming
  they exist and are registered. They appear to delegate to app-core services
  (research library, plugin runtime) but were not line-by-line audited for
  fat-adapter drift.
- `task_runtime.rs` (19 commands) internals — deferred to A-TSK-*. This task
  confirms only that they are registered and that their names suggest
  delegation to `TaskRuntimeStore`.
- Frontend consumption of `execution://event` (the receive half of V03) —
  A-SRF-03 owns it. This task classifies only the emit-side typing.
- The `IpcError` enum's full variant set and whether every command maps errors
  consistently — spot-checked `Validation`/`NotFound`/`Internal` usage; a
  dedicated error-contract audit would be a separate task.

Environmental constraints:

- Read-only static review against `echo-agent-cli` commit `b3b2e81`. No build
  or test execution in this task (the validations are static-inspection
  greps + read proofs). The workspace was not mutated.

Uncertain claims:

- Whether macOS actually orphans the PTY shells on app exit, or whether Tauri
  / the OS sends SIGHUP to the process group on window close. The P2-01
  finding is robust either way (the explicit `close_all` + `kill()` is the
  deterministic cleanup path and it is never invoked), but the user-visible
  severity ("shells keep running" vs "shells die on SIGHUP") depends on
  platform behavior I did not execute. Recommend the regression-validation
  manual scenario (open terminal, run `sleep 1000`, close window, `pgrep`) to
  confirm.
- Whether the TUI's reduced permission-mode alias set
  (`events.rs:3583-3591`) is intentional ("TUI users type canonical names")
  or drift. The CLI has the full set, which suggests drift rather than intent.
  Flagged as a parity finding regardless; the direction (one shared
  canonicalizer) holds either way.

## Handoff

Conclusions downstream tasks may rely on:

1. **The command surface is large (219 commands) and mostly thin.** Downstream
   tasks auditing a specific feature (research, plugins, task_runtime) can
   assume the registration in `mod.rs:69-310` is exhaustive and that
   `TauriState` is the single shared state. No command constructs its own
   `AppState` or `AgentRuntime`.
2. **The chat-turn lifecycle is correctly delegated to `drive_chat`.** No
   parallel chat lifecycle exists in the Tauri layer (confirms A-CHAT-01 from
   the command side). The fat part of `send_chat_message` is setup/teardown
   (P3-03), not lifecycle duplication.
3. **Two tool-execution-persistence authorities exist in the GUI** —
   `TauriChatSink` (foreground) and the subagent bridge in `mod.rs`
   (subagents). The A-CHAT-01-P2-01 `ToolExecutionObserver` extraction must
   subsume both, or the duplication (and the streaming-chunk drift, P2-03)
   persists. Downstream A-CHAT / A-TSK fixes should treat this as one unified
   recorder target.
4. **GUI window close does not clean up terminals.** Any task touching
   desktop shutdown or terminal lifecycle must address P2-01 (the
   `on_window_event` + `close_all` hook). A-BOOT-01's shutdown-sequence
   finding is complementary (it covers the post-`.run()` block; this covers
   the missing terminal piece).
5. **Event emission has two contracts**: typed (`chat://event`, terminal) and
   untyped (`execution://event`). A-SRF-03 (frontend consumption) should be
   told that the `execution://event` schema is enforced only by string
   convention; frontend zod/types should be the safety net until P3-02 lands.
6. **No deadlock was found** in the command path. The one lock-across-await
   (`save_conversation`, P3-04) is bounded and non-deadlocking. Downstream
   tasks can trust the lock-scoping pattern (block-scoped guards + explicit
   `drop`) as the convention.

Reports downstream tasks must read:

- This report (A-SRF-02) for the command-surface map, the triplicated
  permission-mode canonicalization (P2-02), and the terminal-cleanup gap
  (P2-01).
- `tasks/A-BOOT-01.md` for the GUI service-construction and shutdown
  sequence that this task's V04 extends.
- `tasks/A-CHAT-01.md` for the `TauriChatSink` persistence authority that
  P2-03 parallels.

Conditions that make this report stale:

- Registering an `on_window_event` handler that calls `close_all` (resolving
  P2-01) invalidates V04's central claim.
- Lifting permission-mode canonicalization into a shared app-core helper
  (resolving P2-02) invalidates V01's triplication evidence.
- Extracting a unified `ToolExecutionRecorder` that both `TauriChatSink` and
  the subagent bridge call (resolving P2-03 and A-CHAT-01-P2-01 together)
  invalidates V01's parallel-authority evidence.
- Introducing an `ExecutionEvent` enum (resolving P3-02) invalidates V03's
  "untyped execution channel" claim.
- Adding any new `#[tauri::command]` requires updating the V01 count (219).

Follow-up task IDs (no fixes implemented in this review):

- A **terminal-lifecycle** task — resolve A-SRF-02-P2-01 by wiring
  `on_window_event(CloseRequested)` → `terminal_manager.close_all()`.
- A **permission-mode unification** task — resolve A-SRF-02-P2-02 by lifting
  canonicalization into app-core; one caller per surface.
- A **tool-execution-recorder unification** task — resolve A-SRF-02-P2-03
  together with A-CHAT-01-P2-01 by introducing one recorder used by both the
  sink and the subagent bridge. The bridge in `mod.rs` shrinks to event
  routing only.
- A **conversation-projection lift** task — resolve A-SRF-02-P3-01 by moving
  `project_saved_messages` into app-core.
- An **execution-event typing** task — resolve A-SRF-02-P3-02 by introducing
  the `ExecutionEvent` enum.
- A **send_chat_message refactor** task — resolve A-SRF-02-P3-03 by extracting
  attachment-persist + begin-turn helpers.
- A **save_conversation lock-scope** task — resolve A-SRF-02-P3-04 (one-line
  clone-out-of-guard fix).
