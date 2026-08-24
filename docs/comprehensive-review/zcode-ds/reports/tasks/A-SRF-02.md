# A-SRF-02: Tauri command and desktop integration

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both repositories clean

## Question

Are Tauri commands thin, lifecycle-safe adapters with consistent state and no
duplicate business authority?

Answer: **mostly yes, with one P1 composition defect and two P2 duplicate
authorities.** All ~150 registered commands are thin adapters over app-core
services, and per-turn state (turn-busy registry, cancel-token registry, tool
states) is consistently owned in app-core `SessionState`. But (a) `build_tauri_app`
registers **two `.setup()` closures** and Tauri executes only the last one —
the first (DevTools auto-open + the only `browser://event` forwarder) is
silently discarded, leaving the documented browser-event stream dead and the
GUI browser workspace panel non-functional (P1-01); (b) subagent tool
execution projection has **two live producers** — the mod.rs SubagentEventBus
bridge and `TauriExecutionProjector` — that persist the same tool events into
the non-idempotent `ToolExecutionRepository` and emit duplicate
`execution://event` summaries (P2-01); (c) panels.rs re-implements git
worktree helpers already public in app-core `worktree.rs`, with a
field-divergent parser (P2-02).

## Scope

- `echo-agent-cli/src/tauri/` full reads: `mod.rs` (790), `desktop.rs` (271),
  `state.rs` (23), `ipc.rs` (152), `error.rs` (123), `terminal.rs` (420),
  `path_validator.rs` (186).
- `echo-agent-cli/src/tauri/commands/` full reads: `chat.rs` (1723),
  `conversations.rs` (747), `config.rs` (335), `mcp.rs` (614),
  `task_runtime.rs` (488), `tasks.rs` (265), `files.rs` (722), `scheduler.rs`
  (101), `session.rs` (160), `tools.rs` (72), `tool_executions.rs` (57),
  `hooks.rs` (174), `memory.rs` (280); structural reads: `panels.rs` (2238,
  worktree/skills/sandbox/evolution sections), `workspace.rs`, `providers.rs`,
  `plugins.rs`, `research.rs`, `analysis.rs`, `browser.rs`.
- `echo-agent-cli/src-tauri/`: `src/main.rs`, `capabilities/default.json`.
- App-core anchors: `state.rs` (SessionState :347-352, mcp_config :357/:490),
  `tool_execution.rs` (ToolExecutionRepository :163-239), `chat_driver.rs`
  (trace sinks :59-81, drive_chat :202-287), `tasks/task_runtime/worktree.rs`
  (public git helpers), `tasks/task_runtime/task_tools.rs:985`,
  `revisioned_adapter.rs:173`, `browser/session.rs` (event producers), `infra.rs`
  (load_mcp_config :1069-1108).
- Framework anchors: `tauri 2.11.2 Builder::setup` (vendored source),
  `echo-agent/src/agent/subagent/executor.rs:1284-1296` (SubagentEventBus
  producer), `echo-agent/src/agent/react/mod.rs:1617-1636`
  (provider replacement), `echo-core/src/agent/event_envelope.rs` (terminal
  normalization).
- Frontend (event-contract anchors only): `web-frontend/src/hooks/
  useBrowserEvents.ts`, `stores/browserStore.ts`, `components/browser/
  BrowserPanel.tsx`.

## Out Of Scope

- Frontend reducer/store correctness -> A-SRF-03, A-FE-01/02.
- Chat driver/sink semantics -> A-CHAT-01 (dependency, cross-verified).
- Boot composition/shutdown -> A-BOOT-01 (dependency, cross-verified).
- Config/precedence/workspace lifecycle -> A-CFG-01, A-STATE-01 (dependency
  reports read; their findings cross-referenced, not re-filed).
- Browser/MCP/LSP integration semantics -> A-INT-01 (cross-verified P1-01).
- TaskRuntime executor behavior -> A-TSK-01..06.
- GUI dynamic smoke (app launch, webview console) -> Q-GUI-01, Q-E2E-01.

## Inputs

- Root `AGENTS.md` (full; surface parity, layering gate, no parallel
  semantics, no over-gating), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-SRF-02 card), `zcode-ds/README.md`, report templates.
- Dependency reports read: zcode-ds `A-BOOT-01` (complete), `A-CHAT-01`
  (complete); cross-referenced: `A-INT-01` (P1-01 MCP config persistence),
  `A-STATE-01` (conversation projection), `A-CFG-01` (web_config).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md`,
  `echo-agent-cli/docs/MASTER-PLAN.md`, `echo-agent-cli/docs/
  browser-runtime-design.md`, `docs/PROJECT-ANALYSIS.md`.

## Layering Decision

- Generic mechanism (framework, reused as-is): `Agent`/`AgentHandle` event
  stream, `EventEnvelope`, `SchedulerRunner`, `FileStore`, `PathValidator`
  (via `echo_agent::tools::security`), `McpServerConfig`/`McpConfigFile`,
  `SubagentEventBus`. No movement recommended.
- EKO product policy (application, correct placement): `AppState` +
  `SessionState` (turn-busy/cancel-token/tool-state registries), the Tauri
  command surface, MCP input validation (stdio allowlist / https URL),
  path-validator secret denylist, terminal consent gate, TauriChatSink
  projection policy.
- Adapter boundary violations (findings): the SubagentEventBus bridge and
  `TauriExecutionProjector` both persist subagent tool executions (P2-01);
  panels.rs duplicates app-core worktree git helpers (P2-02); TauriChatSink
  owns durable projection state (A-CHAT-01-P2-02, cross-verified);
  `conversations.rs:65-150` owns a canonical-vs-UI merge algorithm (A-STATE-01
  scope, cross-referenced).
- Duplicate search (terms + results in V01-01): `IpcAuth|require_full_auto|
  require_not_strict|IpcPermission` (dead, zero callers); `browser://event`
  (one producer, dead); `execution://event` producers (three, two overlap for
  subagent tools); `run_git|git_repo_root|parse_worktree_list|
  validate_branch_name|default_worktree_path|validate_worktree_target|
  WorktreeInfo` (duplicated panels.rs vs worktree.rs); `create_terminal|
  TerminalManager|close_all` (close_all dead); `PENDING_RESPONSES|
  active_chat_turns|cancel_token|tool_states` (app-core owned);
  `ChatDriverEvent|TauriChatSink|TauriExecutionProjector` (single definitions);
  `worker` (zero in Tauri layer).

## Current Path

Verified call graph (V02-01, V03-01..03):

1. Boot: `src-tauri/src/main.rs:5` -> `desktop::run_desktop_entry` (crash log
   hook) -> `run_desktop` (desktop.rs:124-271): shell env -> config ->
   `AgentRuntime::bootstrap` -> config watcher -> scheduler store ->
   `AppState::from_shared` -> task tools -> `init_pool` -> task service +
   scheduler -> MCP health -> Dreaming -> `build_tauri_app(...).run()`.
2. `build_tauri_app` (mod.rs:29-773): plugins -> `.manage(TauriState::new)` ->
   `.setup(|app| {...})` at :40 (DevTools + browser://event forwarder) ->
   `.invoke_handler` with 218-219 commands (:69-310) -> `.setup(|app| {...})`
   at :311 (global shortcut + SubagentEventBus bridge). **Only the second
   setup runs** (tauri `Builder::setup` overwrites the single setup slot,
   app.rs:1506/1765-1769/2522; V02-01).
3. Command surface: every command resolves `TauriState { app_state,
   browser_runtime, terminal_manager }` (state.rs:9-23) and delegates to
   app-core; chat commands additionally use `SessionState.active_chat_turns`
   / `cancel_token` (state.rs:347-352) and the global `PENDING_RESPONSES`
   HITL transport (chat.rs:212-213); `send_chat_message` (chat.rs:443-731)
   -> `PreparedUserTurn` -> spawned `drive_chat` -> `TauriChatSink` ->
   `chat://event` + `execution://event` + `ToolExecutionRepository`.
4. Event channels: `chat://event` (single producer, chat.rs:114-143);
   `execution://event` (emit_execution_event chat.rs:153-183 +
   emit_tauri_execution_event chat.rs:1419-1446 + bridge mod.rs:751-752);
   `browser://event` (producer mod.rs:58 — dead, P1-01); `terminal-output` /
   `terminal-exit` (PTY reader thread, terminal.rs:148-166).
5. Subagent tool projection (P2-01): framework `SubagentEventBus` publishes
   `DispatchToolStarted/Completed` for every dispatched subagent
   (echo-agent/src/agent/subagent/executor.rs:1284-1296); the live bridge
   (mod.rs:388-530) persists + emits; the task-runtime trace path
   (`subagent_trace_sink_for`, chat_driver.rs:221 / task_tools.rs:985 /
   revisioned_adapter.rs:173) -> `ChatDriverEvent::Execution` -> sink
   (chat.rs:1361-1364) -> `TauriExecutionProjector` (chat.rs:957-1114)
   persists + emits the same owner/call_id pair. `ToolExecutionRepository
   ::start` is non-idempotent (fresh detail_ref per call,
   tool_execution.rs:191-239).
6. Shutdown (desktop.rs:260-268): cancel boot token -> hook dispatcher ->
   browser runtime. Missing: `TerminalManager::close_all` (dead, terminal.rs:
   256-266), scheduler/task-service cancel tokens (state.rs:542/546), GUI
   session-end review (A-BOOT-01 cross-refs; V03-03).

## Findings

### A-SRF-02-P1-01: `build_tauri_app` registers two `.setup()` closures; Tauri runs only the last, so the `browser://event` forwarder and DevTools auto-open never execute — the GUI browser workspace panel is dead

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence:
  - `echo-agent-cli/src/tauri/mod.rs:40-68` — first `.setup()`: DevTools
    auto-open (:45-50) + the ONLY `browser://event` forwarder
    (`app.state::<TauriState>().browser_runtime.subscribe()` -> loop emitting
    `browser://event`, :53-58).
  - `echo-agent-cli/src/tauri/mod.rs:311-772` — second `.setup()`: global
    shortcut (:316-333) + SubagentEventBus bridge (:347-768).
  - tauri 2.11.2 vendored source: `Builder { setup: SetupHook<R> }` single
    non-Option field (app.rs:1506); `pub fn setup(mut self, setup: F) -> Self
    { self.setup = Box::new(setup) }` overwrites (app.rs:1765-1769);
    `App::run` executes exactly one closure via `if let Some(setup) =
    app.setup.take()` (app.rs:2363, 2522-2523).
  - Consumers: `web-frontend/src/hooks/useBrowserEvents.ts:12`
    (`listen('browser://event')` -> `browserStore.ingest`), mounted by
    `BrowserPanel.tsx:14`; views are created only by `ingest` on
    `session_started`/`session_updated` (browserStore.ts:114-131); frame
    polling requires an existing view (BrowserPanel.tsx:27-31). Producers:
    app-core browser session manager sends `BrowserEvent` continuously
    (browser/session.rs:215, 248, 286, 492, 511).
- Reachability: every GUI launch (desktop.rs:256-258). Definition -> builder
  chain -> runtime: the first closure is compiled in but its body never runs.
- Expected invariant: boot-time bridges spawn exactly once each; the
  documented contract `browser-runtime-design.md:215` ("forwards the existing
  `BrowserEvent` stream over `browser://event`") holds; DevTools opens in
  debug builds.
- Observed behavior: only the second setup runs. `browser://event` has no
  live producer; the frontend listener never fires; `browserStore.views` can
  never be created — the browser workspace panel shows no session, no tabs,
  no frame regardless of backend activity. DevTools auto-open is also dead.
- Impact: the GUI browser workspace surface is entirely non-functional
  (major capability failure on the flagship surface); the documented event
  contract is broken while the code still reads as if it works.
- Root cause: a later feature (browser panel bridge, commit 24ab55b) inserted
  a second `.setup()` block ahead of the pre-existing one (371bb90) without
  realizing Tauri's builder overwrites; `Builder::setup` is a plain
  assignment, not a chaining hook.
- Direction: merge both closures into a single `.setup()` (move the browser
  forwarder and DevTools block into the second closure, or refactor each
  block into a helper called from one setup); add a static guard (a comment
  plus a builder-level test) that exactly one `.setup()` exists. Do not keep
  two `.setup()` calls.
- Regression validation: an app-level fixture that starts a browser session
  and asserts the webview receives `browser://event`; or a unit test asserting
  `build_tauri_app`'s builder contains one setup closure that registers both
  forwarders. Manual check: debug GUI shows DevTools + browser panel populates
  after `browser_navigate`.
- Validation reports: [V02-01](../validations/A-SRF-02/V02-01.md), [V03-02](../validations/A-SRF-02/V03-02.md), [V05-01](../validations/A-SRF-02/V05-01.md)

### A-SRF-02-P2-01: Two live producers persist the same subagent tool events into `ToolExecutionRepository` and emit duplicate `execution://event` summaries (mod.rs SubagentEventBus bridge + chat.rs `TauriExecutionProjector`)

- Priority: P2
- Confidence: high
- Layer: application (projection policy) / adapter (both producers)
- Evidence:
  - Bridge: `src/tauri/mod.rs:353-768` subscribes to
    `agent.subagent_registry().event_bus()` (:354-358); on
    `DispatchToolStarted`/`DispatchToolCompleted`/terminal it calls
    `tool_executions.start/finish/cancel` (:416-529) and
    `emit_tool_execution_summary` (:429-434, :478-483, :516-521).
  - Projector: `src/tauri/commands/chat.rs:957-1114`
    (`project_tool_event` persists `ExecEvent::subagent` ToolStarted/
    ToolOutput/ToolCompleted), fed from the task-runtime trace sink:
    `chat_driver.rs:221` (drive_chat), `task_tools.rs:985` (foreground
    create_complex_task), `revisioned_adapter.rs:173` -> `ChatDriverEvent::
    Execution` -> sink (chat.rs:1361-1364).
  - Same-event provenance: the framework publishes `DispatchToolStarted` for
    every dispatched subagent (echo-agent/src/agent/subagent/executor.rs:
    1284-1296); the task runtime observes the same subagent's envelope stream
    and emits matching `ExecEvent::subagent` (executor.rs:3159-3380).
  - Non-idempotent store: `ToolExecutionRepository::start` always allocates a
    fresh `detail_ref` and overwrites `state.summaries[owner, call_id]`
    (tool_execution.rs:191-239) — the first detail's manifest/output/journal
    files become orphaned on disk.
- Reachability: any GUI chat turn that spawns a Task-mode/foreground complex
  run whose subagents call tools; also Task interaction-mode turns. Background
  runs (trace_sink=None) hit only the bridge.
- Expected invariant: one projection authority per logical tool execution; a
  tool call is persisted and announced exactly once (AGENTS.md "严禁平行实现
  同一语义"; MASTER-PLAN:771 "task-local trace 与 framework external trace 都
  桥接到同一交互事件").
- Observed behavior: the same (owner=Subagent{subagent_run_id}, call_id) is
  `start()`-ed and `finish()`-ed twice; two "started" summary events are
  emitted; the bridge and the projector keep independent `active_tool_ids`
  bookkeeping so terminal cleanup can double-fire or diverge; the first
  detail files are orphaned.
- Impact: duplicate `execution://event` tool events on the wire (frontend
  tool cards updated twice), redundant disk writes and accumulated orphaned
  detail files, and two mutable projection states that can diverge (e.g. one
  marks "cancelled" while the other has already removed the call).
- Root cause: the GUI has two event channels for the same framework bus (the
  generic bridge was added to make subagent tools visible app-wide; the
  projector was added to route TaskRuntime trace events through the shared
  sink) and both were wired to the same durable repository without
  deduplication.
- Direction: pick one authority — either delete the bridge's persistence and
  keep only its wire emission (or vice versa), or make one path idempotent
  (start() should upsert by (owner, call_id) and return the existing
  summary). Prefer the trace-sink projector as the canonical TaskRuntime
  path and reduce the bridge to a no-op for events the projector already
  covers, or gate the bridge to non-GUI surfaces. Delete the superseded
  persistence block.
- Regression validation: a GUI fixture running a plan-task subagent with one
  tool call, asserting exactly one `started` and one `finished` summary row in
  the repository and one event pair on `execution://event`; a repository test
  that double-`start` is idempotent.
- Validation reports: [V01-01](../validations/A-SRF-02/V01-01.md), [V03-02](../validations/A-SRF-02/V03-02.md)

### A-SRF-02-P2-02: panels.rs re-implements git worktree helpers already public in app-core `worktree.rs`, with a parser that omits lock state

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence:
  - Duplicated helpers: `src/tauri/commands/panels.rs:1804` (`run_git`),
    `:1823` (`git_repo_root`), `:1828` (`parse_worktree_list`), `:1896`
    (`validate_branch_name`), `:1908` (`default_worktree_path`), `:1929`
    (`validate_worktree_target`) vs the public app-core versions
    `echo-agent-app-core/src/tasks/task_runtime/worktree.rs:149, 194, 202,
    218, 245, 272`.
  - Divergence: panels.rs `WorktreeInfo` (panels.rs:1788-1793) has only
    path/branch/managed/head; the app-core `WorktreeInfo` adds `locked` and
    `lock_reason` (worktree.rs:97-107) and the app-core parser handles
    `locked`/`lock_reason` lines. `remove_worktree` (panels.rs:2005-2043)
    does not consult lock state.
- Reachability: GUI Worktrees panel commands `list_worktrees`,
  `create_worktree`, `remove_worktree` (registered mod.rs:295-297) on every
  repository workspace.
- Expected invariant: one implementation per git-worktree parsing/validation
  semantic; GUI and task-runtime views agree on worktree state (AGENTS.md
  "严禁平行实现同一语义").
- Observed behavior: two implementations of the same porcelain parsing and
  branch validation exist; the GUI's raw list drops lock info that the
  app-core listing (used by `list_unattended_worktrees`, which does delegate)
  exposes; the two parsers can drift independently.
- Impact: divergence risk (GUI may show locked worktrees as removable /
  missing lock reason); duplicated maintenance surface in the adapter layer.
- Root cause: the panels worktree UI was written before (or without
  consulting) the app-core worktree module that already exposed the same
  helpers.
- Direction: delete the six panels.rs helpers and call the app-core
  functions (map `WorktreeError` -> `IpcError`); extend the GUI `WorktreeInfo`
  DTO with `locked`/`lock_reason` (or serialize the app-core struct
  directly).
- Regression validation: GUI `list_worktrees` on a repo with a locked
  worktree returns `locked: true` + reason, matching `git worktree list`;
  `remove_worktree` on a locked worktree is refused with the same error the
  app-core path produces.
- Validation reports: [V01-01](../validations/A-SRF-02/V01-01.md)

### A-SRF-02-P3-01: `IpcAuth`/`IpcPermission` are dead gate machinery whose "Phase 6.2" header documents them as active

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `src/tauri/error.rs:15-70` defines `IpcPermission::FullAuto/
  NotStrict` + `IpcAuth::require_full_auto/require_not_strict`; repository-
  wide grep (V01-01): zero callers anywhere in either repository. Header
  error.rs:3-10 ("Authorization (Phase 6.2)… gated behind
  `IpcAuth::require_full_auto()`") describes the gates as active. AGENTS.md
  documents the removal of exactly these gates from `create_terminal`/
  `connect_mcp_server` as a history lesson.
- Reachability: none (dead code); the doc comment is read by every developer.
- Expected invariant: no dead permission machinery; docs describe the real
  behavior (no permission-mode gating on user-driven GUI features in the
  local-assistant threat model).
- Observed behavior: the types compile, the doc claims gating exists, but no
  command consults them.
- Impact: misleading documentation of the permission model; dead code
  retained (AGENTS.md: delete dead code).
- Root cause: the gate removal (post history-lesson) deleted call sites but
  left the type + doc header.
- Direction: delete `IpcPermission`/`IpcAuth` from error.rs and replace the
  header with the actual policy (input validation only; no permission-mode
  gating), or keep a single `IpcError`-only module.
- Regression validation: grep `IpcAuth|require_full_auto` returns nothing
  after removal; GUI terminal + MCP still work under default permission mode
  (Q-E2E-01 scenario).
- Validation reports: [V01-01](../validations/A-SRF-02/V01-01.md), [V05-01](../validations/A-SRF-02/V05-01.md)

### A-SRF-02-P3-02: the old chat turn clears the agent's HumanLoopProvider after emitting Done — a next turn attaching its own provider in that window gets it wiped, silently swallowing its HITL requests

- Priority: P3
- Confidence: medium (ordering race, statically verified; window not
  reproduced dynamically)
- Layer: adapter
- Evidence: `src/tauri/commands/chat.rs:700-719` — spawned turn task releases
  the busy registry (:701-706), emits `TurnStatus`+Done (:709-711), and only
  then replaces the provider with an empty `HitlDispatcher`
  (:712-719, `set_human_loop_provider_preserving_approvals`); the new turn
  attaches its `TauriHumanLoopHandler` at :575-582. The setter fully replaces
  the provider (`echo-agent/src/agent/react/mod.rs:1617-1636`). Both writes
  serialize on the agent write lock in unspecified order. The comment at
  chat.rs:707-708 explicitly anticipates the frontend dispatching the next
  queued turn on Done.
- Reachability: consecutive turns on the same agent with the second turn
  starting between the old turn's Done emission and its write_async
  completion (millisecond window; widened by auto-dispatched queued turns).
- Expected invariant: each turn's HITL transport remains attached for the
  lifetime of that turn only; a later turn's transport is never clobbered by
  an earlier turn's cleanup.
- Observed behavior: if the old clear lands after the new attach, the agent
  runs the new turn with an empty dispatcher — approval/input/selection
  requests never reach the GUI and resolve only via the 300 s timeout
  (chat.rs:350-353 etc.), appearing to the user as a silent hang with no
  prompt.
- Impact: rare but real loss of the HITL prompt on the flagship surface;
  inconsistent with the per-turn isolation design.
- Root cause: cleanup order — the provider clear is not ordered relative to
  the next turn's attach and is not guarded by ownership (any provider is
  replaced, not only the one this turn installed).
- Direction: clear the provider BEFORE emitting Done (move :712-719 ahead of
  :709-711), or make the clear conditional (only replace if the current
  provider is the `TauriHumanLoopHandler` this turn attached — compare
  Arc::ptr_eq), or route HITL through a per-conversation provider registry
  instead of mutating the agent.
- Regression validation: fixture with two back-to-back turns where the second
  starts immediately on Done; assert the second turn's approval request emits
  `ApprovalRequest` on `chat://event` and resolves via `send_approval_response`.
- Validation reports: [V03-01](../validations/A-SRF-02/V03-01.md)

### A-SRF-02-P3-03: `TerminalManager::create` has a check-then-insert race on duplicate session ids that can orphan a PTY

- Priority: P3
- Confidence: low (impact), high (pattern)
- Layer: application
- Evidence: `src/tauri/terminal.rs:216-231` — `contains_key` then
  `PtySession::spawn` then `sessions.insert` run synchronously but can
  interleave between two threads for the same id (commands run on the
  multi-threaded async runtime); the loser's `Arc<PtySession>` is dropped —
  its PTY master closes (child typically SIGHUP'd) while its reader thread
  keeps emitting until EOF.
- Reachability: two concurrent `create_terminal` calls with the same id (the
  frontend generates ids; collision requires a bug or a scripted client).
- Expected invariant: session ids are unique; creating a duplicate id never
  spawns a second process.
- Observed behavior: one session wins the map; the other's process may linger
  briefly and its reader thread emits duplicate `terminal-output` events for
  the same id.
- Impact: minor (rare, self-cleaning via SIGHUP); duplicate events and a
  short-lived orphan shell possible.
- Root cause: `DashMap::entry`-style atomic insert was not used.
- Direction: use `sessions.entry(id).or_insert_with(...)` (DashMap entry API)
  and return an error on an occupied entry without spawning; add a unit test
  with two threads racing the same id.
- Regression validation: concurrent `create_terminal(same_id)` test asserting
  exactly one session exists and the other returns "already exists".
- Validation reports: [V03-01](../validations/A-SRF-02/V03-01.md)

## Cross-Verified Dependency Findings (canonical IDs elsewhere; independently confirmed here)

| Canonical ID | Claim | Independent confirmation |
|---|---|---|
| A-INT-01-P1-01 | GUI MCP config never persists; panel never seeded from disk | Confirmed: `update_mcp_config` (mcp.rs:476-559) only writes the in-memory `plugins.mcp_config` RwLock (state.rs:357, initialized default state.rs:490); no `fs::write` of mcp.json anywhere; boot loader (infra.rs:1069-1108) loads only into the agent. |
| A-CHAT-01-P1-01 | GUI labels error-terminated turns "completed"; user cancel surfaces fabricated error | Confirmed: chat.rs:690-696 derives terminal status from cancel token + `drive_chat` Result (always `Ok` for envelope-normalized errors); envelope never yields `Err` (event_envelope.rs:134-191). |
| A-CHAT-01-P2-02 | TauriChatSink owns durable tool-execution projection and guesses cancellation from any terminal | Confirmed: chat.rs:1148-1331 (sink fields; `cancel_active_tools` on `Cancelled`/`Error` :1325-1328 and on any non-running TurnStatus :1365-1368). |
| A-CHAT-01-P2-01 | `ChatDriverEvent::Interrupt` dead; GUI emits `InterruptPrompt` directly | Confirmed: chat.rs:516-534 emits `ChatEvent::InterruptPrompt` bypassing the shared sink; zero producers of `ChatDriverEvent::Interrupt` (V03-02). |
| A-BOOT-01-P3-03 | `TerminalManager::close_all` dead on GUI exit | Confirmed: zero callers (V03-03). |
| A-BOOT-01-P3-02 / P2-03 | scheduler/task-service tokens never cancelled; GUI lacks session-end review | Confirmed (V03-03). |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (command-to-service map, both repos) | yes | passed | [V01-01](../validations/A-SRF-02/V01-01.md) |
| V02 | Registration and runtime reachability (invoke_handler, TauriState, setup closures, bridges) | yes | passed | [V02-01](../validations/A-SRF-02/V02-01.md) |
| V03 | Invariant/edge cases (state/lock/await; event emission contract; window/terminal cleanup) | yes | passed | [V03-01](../validations/A-SRF-02/V03-01.md), [V03-02](../validations/A-SRF-02/V03-02.md), [V03-03](../validations/A-SRF-02/V03-03.md) |
| V04 | Targeted executable check (`cargo check --workspace --locked`; gui-bin check; gui-feature Tauri tests) | yes | passed | [V04-01](../validations/A-SRF-02/V04-01.md) (exit 0), [V04-02](../validations/A-SRF-02/V04-02.md) (exit 0), [V04-03](../validations/A-SRF-02/V04-03.md) (exit 0; 21 tests) |
| V05 | Historical-document drift (browser-runtime-design, MASTER-PLAN, error.rs header) | yes | passed | [V05-01](../validations/A-SRF-02/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `browser-runtime-design.md:215` "forwards the existing BrowserEvent stream over browser://event" | regressed | producer mod.rs:58 is in the overwritten first `.setup()` (A-SRF-02-P1-01) |
| `MASTER-PLAN.md:380/733/771` shared Tauri/TUI/CLI/channel event contract; both trace bridges to one interactive event | current (shape) / regressed (uniqueness) | bridges exist (chat.rs:1361-1364, mod.rs:347-768) but persist the same tool events twice (A-SRF-02-P2-01) |
| `MASTER-PLAN.md:483` subagent tool events keep stable call_id and pass through Tauri | current | both projection paths preserve call_id (V03-02) |
| `error.rs:3-10` "Phase 6.2" IpcAuth gating active | stale | zero callers (A-SRF-02-P3-01) |
| `MASTER-PLAN.md:772` Tauri notice projections for budget/guard/memory/safety/parameter/chart | current | chat.rs:1511-1570 |
| `MASTER-PLAN.md:789` GUI first-screen browser check clean | stale (for panel behavior) | browser panel cannot populate (P1-01) |
| AGENTS.md history: `require_full_auto` gates on create_terminal/connect_mcp_server removed | current | no gate on terminal.rs:277-297 or mcp.rs:210-258; only the dead IpcAuth remains |

## Coverage And Uncertainty

- All conclusions are static except three compile/test runs (V04). No GUI
  process was launched: the browser-panel emptiness, duplicate wire events,
  and the HITL-clear race are proven by code traces, not observed at runtime
  (Q-GUI-01 / Q-E2E-01 own dynamic confirmation).
- The double-setup overwrite is verified against the vendored tauri 2.11.2
  source (single `setup` slot, `take()` at run) — a behavioral, not stylistic,
  claim.
- `panels.rs` (2238 lines) was structurally reviewed (worktree/permission/
  skills/sandbox/evolution surfaces); its deepest business logic
  (evolution scans, skill drafts) delegates to app-core services — not
  exhaustively re-audited (A-MEM-01/A-PLG-01/A-EVO-01 scope).
- `plugins.rs`/`providers.rs`/`research.rs`/`analysis.rs` were skimmed for
  authority; their domain correctness is owned by other tasks.
- The duplicate-projection finding assumes TaskRuntime foreground runs attach
  the trace sink (task_tools.rs:985) AND the framework publishes bus events
  for the same subagent — both verified statically; the exact interleaving
  under a real run is dynamic-suite scope.

## Handoff

- Downstream tasks may rely on: complete command registration (V02); thin
  command layer with app-core-owned per-turn state; the dead `browser://event`
  channel (P1-01) — `A-SRF-03`/`A-FE-01` must not rely on browser events
  until fixed; the duplicate subagent tool projection (P2-01) — frontend
  tool-card reducers should tolerate (and later dedupe) duplicate summaries;
  the panels.rs worktree duplication (P2-02) — `A-TSK-05` should unify the
  worktree surface when touching ownership policy.
- Reports to read: this report + V01-01..V05-01; A-BOOT-01, A-CHAT-01
  (canonical IDs cross-verified above), A-INT-01 (P1-01), A-STATE-01,
  A-CFG-01.
- Stale triggers: any change to `build_tauri_app` (mod.rs:29-773 — especially
  the setup closures and the bridge), `chat.rs` (send/steer/cancel, sink,
  projector, terminal-status derivation), `panels.rs` worktree helpers,
  `terminal.rs` (TerminalManager), `desktop.rs` shutdown block, or
  `error.rs` invalidates the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): A-SRF-03
  (browser panel + TurnStatus reconciliation), A-FE-01/02 (tool-card
  dedupe), A-TSK-05 (worktree unification), X-SRF-01 (browser-panel parity
  row; browser://event in the event-conformance matrix X-EVT-01), Q-GUI-01,
  Q-E2E-01 (browser panel session, duplicate event counts, HITL back-to-back
  turn, GUI MCP save-restart).
