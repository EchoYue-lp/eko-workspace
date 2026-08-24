# A-BOOT-01: Application composition and startup lifecycle

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both repositories clean

## Question

Does each EKO entry point construct the same core services exactly once with
consistent config, working directory, shutdown, and reload behavior?

## Scope

- Entry points: `echo-agent-cli/src/main.rs` (TUI/CLI/channels dispatch),
  `src-tauri/src/main.rs` + `src/tauri/desktop.rs` (GUI), `src/tauri/mod.rs`
  (Tauri builder/state), `src/tauri/state.rs` (TauriState).
- Composition roots: app-core `runtime.rs` (AgentRuntime::bootstrap,
  into_app_state, init_pool), `state.rs` (AppState::from_shared, task/scheduler
  service starters), `infra.rs` (create_agent, stores, health/dreaming/watcher
  spawns, logging), `cli/modes.rs` (start_headless_services,
  run_channels_mode, run_cli_mode), `cli/repl.rs`, `tui/mod.rs` (run_tui),
  `config_watcher.rs`, `scheduler/runner.rs`, `tasks/background.rs`
  (entry-relevant portions), `src/tauri/terminal.rs` (PTY cleanup).

## Out Of Scope

- Per-command correctness of the Tauri IPC surface (`A-SRF-02`); TUI widget
  loop (`A-SRF-01`); channels/cron runtime behavior (`A-SRF-04`, `A-TSK-03`);
  config precedence and hot-reload domains (`A-CFG-01`); plugin reload
  lifecycle (`A-PLG-01`); framework scheduler internals (`F-OPS-01`).

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-BOOT-01 card), `zcode-ds/README.md`, templates.
- Dependency report: `B-PATH-01` (zcode-ds track) — used for its entry-point
  inventory and to avoid duplicating its findings (P2-01 MCP health GUI-only,
  P2-02 task-store duplication, P3-01 dead `--web` args, P3-02 GUI argv).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md`,
  `docs/2026-07-17-surface-parity-closeout.md`,
  `docs/2026-07-16-agent-lifecycle-audit.md`,
  `docs/2026-07-28-app-core-full-audit.md`.

## Layering Decision

- Generic mechanism: none new at this layer. The framework already provides
  `SchedulerRunner`, `FileStore`/`InMemoryStore`, `FileRuntimeStateStore`,
  `FileConversationStore`, `CancellationToken`; EKO only selects and wires
  them.
- EKO product policy: entry dispatch (`main.rs:75-91`), per-entry service
  assembly, the surface-parity invariant (AGENTS.md), session-end memory
  review policy, channels composition, workspace/CWD semantics.
- Adapter boundary: `AgentRuntime::bootstrap` is the single correct
  composition root (one definition, two call sites — verified).
  `start_headless_services` (cli/modes.rs:32-64) is an adapter that violates
  the thin-adapter rule: instead of calling `AppState::start_task_service` /
  `start_scheduler_with_store` on a real state, it builds a throwaway
  `AppState` whose own TaskRuntimeStore and WebhookEmitter are immediately
  replaced and discarded (see A-BOOT-01-P2-01).
- Duplicate search (terms + results in V01-01): `AgentRuntime::bootstrap`,
  `AppState::from_shared`, `start_headless_services`, `run_channels_mode`,
  `TaskRuntimeStore::new` / `new_in_memory`, `recover_incomplete`,
  `register_task_tools_on_agent`, `bind_task_execute_to_pool`,
  `spawn_config_watcher`, `spawn_dreaming_task`, `spawn_mcp_health_check`,
  `on_session_end`, `into_app_state`, `close_all`, `sqlite`, `worker`.
  One definition per concept; two production construction sites for the
  file-backed TaskRuntimeStore (main.rs:37, state.rs:548 — the B-PATH-01-P2-02
  duplication, cross-referenced) plus a discarded third instance created by
  the temporary AppState (this task's P2-01). No `worker` terminology.

## Current Path

Verified call graph (V02-01):

- `main.rs:60` sets `~/.eko` roots, then either routes the package bin to
  `run_desktop_entry` (gui-only build, main.rs:75-76) or runs
  `run_tui_or_cli_entry` (TUI default; `--cli`/`--channels`/`--web` internal).
- Headless (main.rs:94-451): args -> config -> logging -> conversation store
  -> `AgentRuntime::bootstrap` (main.rs:168) -> `build_task_runtime_store_for_headless`
  (main.rs:35-57: file -> in-memory fallback + `recover_incomplete`) ->
  `register_task_tools_on_agent` (main.rs:177) -> AgentPool +
  `bind_task_execute_to_pool` (main.rs:184-201) -> resume/messages ->
  `spawn_config_watcher` (main.rs:232) -> mode dispatch: TUI swaps the HITL
  provider and calls `start_headless_services` (main.rs:258) + `run_tui`
  (main.rs:276); CLI calls `run_cli_mode` (main.rs:374/414) which calls
  `start_headless_services` again (modes.rs:83); channels spawns
  `run_channels_mode` (main.rs:365).
- `start_headless_services` (modes.rs:32-64) builds a temporary `AppState`
  (modes.rs:57), starts BackgroundTaskService + SchedulerRunner, and returns
  their handles; the AppState and its own TaskRuntimeStore/WebhookEmitter are
  discarded (modes.rs:58-60).
- GUI (desktop.rs:124-271): shell env -> fixed argv (desktop.rs:132) -> config
  -> logging -> `bootstrap` (desktop.rs:160) -> watcher (desktop.rs:166) ->
  scheduler store (desktop.rs:174-181) -> `AppState::from_shared`
  (desktop.rs:187) -> task tools (desktop.rs:201) -> `init_pool`
  (desktop.rs:210) -> bind (desktop.rs:217) -> `start_task_service` +
  `start_scheduler_with_store` (desktop.rs:231-232) -> MCP health
  (desktop.rs:243) -> Dreaming (desktop.rs:247) -> `build_tauri_app().run()`
  (desktop.rs:257).
- Shutdown: hook dispatcher + browser runtime shut down on all four paths;
  config-watcher cancel order differs (GUI first, headless last);
  scheduler/task-service cancel tokens never cancelled (V03-03).

## Findings

### A-BOOT-01-P2-01: `start_headless_services` builds a throwaway AppState that constructs and discards a second TaskRuntimeStore plus other boot services

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/cli/modes.rs:56-63` — `let mut state = AppState::from_shared(...)`, then `state.webhook.emitter = ...` (58), `state.tasks.runtime = task_runtime_store` (60) and only `start_task_service()` / `start_scheduler_with_store()` are used; the state drops at return.
  - `echo-agent-cli/echo-agent-app-core/src/state.rs:547-566` — `from_shared` itself opens the file-backed `TaskRuntimeStore` (548), runs `recover_incomplete()` (557), and builds a `WebhookEmitter` (state.rs:469); both are replaced at modes.rs:58/60 and dropped.
  - `echo-agent-cli/src/main.rs:175` — the store actually kept was already constructed + recovered before the pool.
- Reachability: `start_headless_services` runs on every TUI start (main.rs:258) and every CLI start (modes.rs:83); the temporary AppState construction is therefore on both main headless boot paths. `from_shared` additionally reindexes the session search engine (state.rs:496-506), opens the tool-execution repository (state.rs:507-534) and the workspace registry (state.rs:575-591) — all discarded.
- Expected invariant: each core service is constructed exactly once per process; entry points share one composition path (AGENTS.md duplicate-authority rule; "严禁平行实现同一语义").
- Observed behavior: in the same process, the file-backed TaskRuntimeStore is opened twice and `recover_incomplete()` runs twice (second instance discarded); a second WebhookEmitter, search-engine reindex, tool-execution repo, and workspace registry are created and thrown away on every TUI/CLI boot.
- Impact: duplicate I/O and a double recovery pass at every headless boot; more importantly, a second composition path for the canonical task store that can silently diverge (different fallback/recovery logic in `from_shared` vs `build_task_runtime_store_for_headless`) — exactly the lockstep-divergence risk B-PATH-01-P2-02 describes, now with a third site.
- Root cause: `start_headless_services` reuses `AppState` as a service container without wanting the state; the service starters are methods on AppState, so the adapter fabricates a whole state to call two methods.
- Direction: extract `start_task_service`/`start_scheduler_with_store` into a small shared builder (or make them static helpers taking (agent, pool, store, webhook) and returning the services), so headless assembly does not construct a discarded AppState; delete the temporary-state block (modes.rs:56-63). Coordinate with B-PATH-01-P2-02 (unify the two/three TaskRuntimeStore construction sites into one app-core helper).
- Regression validation: assert exactly one `TaskRuntimeStore::new()` (and one `recover_incomplete` pass) per headless boot via trace logs; run the shared boot fixture through TUI and CLI and compare service counts.
- Validation reports: [V01](../validations/A-BOOT-01/V01-01.md), [V02](../validations/A-BOOT-01/V02-01.md)

### A-BOOT-01-P2-02: channels-only entry never starts BackgroundTaskService or Scheduler

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/main.rs:391-403` — channels-only branch (`--channels` without `--cli`) only awaits `run_channels_mode`; `start_headless_services` is never called on this path.
  - `echo-agent-cli/src/cli/modes.rs:118-235` — `run_channels_mode` receives the pool and store but constructs no task service and no scheduler.
  - `start_headless_services` callers are only main.rs:258 (TUI) and modes.rs:83 (CLI).
- Reachability: `--channels` is a live hidden flag (args.rs:54-55); channels-only is a reachable boot path that runs IM channels with per-sender pool agents.
- Expected invariant: every surface is a full Agent surface (AGENTS.md); services that exist in one mode (background task service, cron scheduler) exist in all modes — same class as B-PATH-01-P2-01 (MCP health GUI-only) and the channels Dreaming gap.
- Observed behavior: in channels-only mode, cron tasks never fire and background TaskRuntime runs are not resumed at boot (no scheduler runner, no `resume_pending` service); TUI/CLI/GUI all have both.
- Impact: a user running `--channels` silently loses the cron/background capability that other surfaces have; a scheduled task or background run created from a channel conversation never executes.
- Root cause: the channels boot path was wired to the pool/store directly and never given the shared headless service assembly.
- Direction: call `start_headless_services` (or the extracted builder from P2-01) in the channels-only branch before spawning `run_channels_mode`, threading the returned scheduler into the plugin runtime like the other modes (main.rs:267-274); add the channels row to the X-SRF-01 capability matrix.
- Regression validation: with a cron task configured, run channels-only mode and verify the task fires; verify background runs resume after restart in channels-only mode.
- Validation reports: [V02](../validations/A-BOOT-01/V02-01.md), [V05](../validations/A-BOOT-01/V05-01.md)

### A-BOOT-01-P2-03: GUI never runs session-end memory review

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/main.rs:306-327` (TUI) and `echo-agent-cli/src/cli/repl.rs:419/454` (CLI) are the only production callers of `review_integration.on_session_end()` (definition: `echo-agent-app-core/src/evolution/review_integration.rs:114`).
  - `echo-agent-cli/src/tauri/desktop.rs` and all of `src/tauri/**` contain no `on_session_end` / session-end review call (grep, V02-01).
- Reachability: GUI is the primary product surface; the TUI/CLI paths run the review on every interactive session end.
- Expected invariant: TUI/GUI/CLI are feature-equivalent surfaces (AGENTS.md); a maintenance capability wired in TUI/CLI must exist in GUI.
- Observed behavior: on the desktop app, memory staleness scoring, conflict detection, and review-inbox proposal generation never run at session end; they only run if the user manually triggers the review command/panels path.
- Impact: stale-memory accumulation and missed conflict/GC proposals on the flagship surface; the "review_on_session_end" product feature is effectively TUI/CLI-only.
- Root cause: the session-end hook was added to the REPL/TUI exit flows and never propagated to the Tauri exit path (desktop.rs:260-268).
- Direction: invoke `state.review_integration.on_session_end()` in the desktop exit path (before `store.shutdown_hook_events()`, mirroring main.rs:306-327), or hook a Tauri window-close/exit callback; add a GUI row to the surface capability matrix.
- Regression validation: run GUI with memory typed, close the app, verify the review report is logged/proposals queued; compare with TUI run.
- Validation reports: [V01](../validations/A-BOOT-01/V01-01.md), [V02](../validations/A-BOOT-01/V02-01.md), [V05](../validations/A-BOOT-01/V05-01.md)

### A-BOOT-01-P3-01: config-watcher cancellation ordering diverges between GUI and headless shutdown

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `src/tauri/desktop.rs:261` cancels the watcher token FIRST; `src/main.rs:336/400/445` cancels it LAST (after `store.shutdown_hook_events()` and `runtime.browser_runtime.shutdown()`).
- Reachability: every normal exit of every entry.
- Expected invariant: background reload loops are stopped before dependent services are torn down, consistently across entries.
- Observed behavior: headless teardown keeps the config watcher armed while the task hook dispatcher and browser runtime are already shut down; a config save in that window can trigger a hooks/webhook reload against a partially torn-down runtime.
- Impact: low-probability reload-on-teardown race; inconsistent ordering makes the teardown contract harder to reason about.
- Root cause: the two entries were written independently and never aligned on the cancel order.
- Direction: pick one order (cancel watcher first, matching GUI) in all four headless shutdown blocks.
- Regression validation: a test that saves config during teardown and asserts no hook reload fires after hook-dispatcher shutdown.
- Validation reports: [V03-03](../validations/A-BOOT-01/V03-03.md)

### A-BOOT-01-P3-02: scheduler and background-task service are never cancelled on any exit path

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence: cancel tokens `state.scheduler.cancel_token` / `state.tasks.cancel_token` (state.rs:542, 546) are never cancelled in desktop.rs:260-268, main.rs:329-338/391-403/437-445, or anywhere else (grep: only workspace-switch code touches related tokens); `BackgroundTaskService::spawn` (tasks/service.rs:591) and the scheduler runner are process-lifetime loops.
- Reachability: cron ticks (framework runner) and the background service run until process exit on every surface.
- Expected invariant: shutdown cancels long-lived loops; nothing runs against a torn-down runtime.
- Observed behavior: both loops are only killed by process exit; in the TUI/CLI window between session end and process exit (memory review, cleanup block), and in the GUI window between window close and process exit, a cron tick can start a TaskRuntime run on a pool agent during teardown.
- Impact: low (windows are milliseconds), but shutdown is incomplete by construction and any future in-process reset (e.g., workspace switch or config-driven restart) has no scheduler stop primitive.
- Root cause: process-exit was treated as the only shutdown mechanism; the tokens exist but are never triggered.
- Direction: cancel `scheduler.cancel_token` + `tasks.cancel_token` in every exit block (and, with P2-01, return them from the headless service builder so the temporary AppState cannot own them silently).
- Regression validation: a shutdown fixture asserting scheduler loop exits and no cron fire after cancel.
- Validation reports: [V03-03](../validations/A-BOOT-01/V03-03.md)

### A-BOOT-01-P3-03: GUI exit never closes live PTY terminal sessions (`TerminalManager::close_all` is dead)

- Priority: P3
- Confidence: high (dead API), medium (impact)
- Layer: application
- Evidence: `src/tauri/terminal.rs:256-266` defines `close_all()`; zero callers outside terminal.rs (grep, V01-01). Desktop shutdown (desktop.rs:260-268) only cancels tokens, shuts the store hook dispatcher and browser runtime.
- Reachability: `create_terminal` IPC (terminal.rs:293) is reachable from the GUI; sessions live for the app lifetime.
- Expected invariant: application shutdown kills child processes it spawned.
- Observed behavior: PTY children are left to the kernel: when the PTY master fd closes at process exit, the foreground process group typically receives SIGHUP — usually self-cleaning, not guaranteed for daemonized/reparented children.
- Impact: possible orphaned shell processes after quitting EKO on macOS/Windows; the explicit kill API exists and is simply not wired.
- Root cause: `close_all` was added to TerminalManager but never called from the exit path.
- Direction: call `state.terminal_manager.close_all()` in the desktop shutdown block; remove it if the SIGHUP behavior is deemed sufficient (then delete the dead method per AGENTS.md cleanup rules).
- Regression validation: GUI shutdown test/script verifying child PIDs of live terminal sessions are gone after app exit.
- Validation reports: [V01](../validations/A-BOOT-01/V01-01.md), [V03-03](../validations/A-BOOT-01/V03-03.md)

### A-BOOT-01-P3-04: `--channels --cli` combined mode abandons the channels task without graceful stop

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `src/main.rs:365` spawns `run_channels_mode`; in the `run_cli` branch (main.rs:373-404) the `channels_handle` is dropped without await; `manager.stop_all()` (modes.rs:228-232) is only reachable on the channels-only await path (main.rs:393). The comment at main.rs:404 documents the abandonment.
- Reachability: combined mode is a reachable hidden flag combination.
- Expected invariant: all spawned subsystems get their normal stop path.
- Observed behavior: channels die with the process; `shutdown_signal()`/`stop_all()` never run for them.
- Impact: QQ/Feishu transports are hard-killed instead of gracefully closed; HTTP long-polling connections may be left open server-side until timeout. Minor for internal mode.
- Root cause: combined mode treats channels as fire-and-forget.
- Direction: after `run_cli_mode` returns, cancel the channels task or await it with a short timeout, and remove the misleading "自动结束" comment once behavior is explicit.
- Regression validation: combined-mode run asserting channels stop path executes on CLI exit.
- Validation reports: [V03-03](../validations/A-BOOT-01/V03-03.md)

### A-BOOT-01-P3-05: `AgentRuntime::into_app_state` is a dead composition API

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-app-core/src/runtime.rs:374-390` — `pub fn into_app_state`; zero callers in either repository (grep, V01-01). GUI builds state manually (desktop.rs:187-195); headless builds a throwaway state (modes.rs:57).
- Reachability: none — definition exists, never registered/called.
- Expected invariant: no dead composition entry points in the boot layer (AGENTS.md: delete dead code; framework-layer rules do not apply — this is app-core).
- Observed behavior: the method compiles and documents a pattern nobody uses; it also duplicates the desktop.rs wiring it was supposed to replace.
- Impact: maintenance burden and a second "documented" composition shape that can drift from the real one.
- Root cause: leftover from a refactor that moved the GUI to manual wiring.
- Direction: delete `into_app_state` (with P2-01, prefer one real composition path).
- Regression validation: `cargo check -p echo-agent-cli` and `cargo check --no-default-features --features gui --bin echo-agent-tauri` after removal (V04-01/02 baseline).
- Validation reports: [V01](../validations/A-BOOT-01/V01-01.md), [V04-01](../validations/A-BOOT-01/V04-01.md)

### A-BOOT-01-P3-06: stale "sqlite-backed" / `runtime_state.db` comments contradict the file-backed stores

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-app-core/src/infra.rs:125` ("sqlite-backed" on `state_store` param) while the injected store is `FileRuntimeStateStore` (infra.rs:1254); `echo-agent-app-core/src/state.rs:931` logs `runtime_dir.join("runtime_state.db")` for a file-backed store. The 2026-07-16 audit already classified these as P2 doc cleanup (docs/2026-07-16-agent-lifecycle-audit.md:133) — still unfixed.
- Reachability: log line executes on GUI workspace switch; the comment is static.
- Expected invariant: comments/log messages describe the real storage engine; no SQLite wording in the no-SQLite application layer.
- Observed behavior: misleading text persists; a reader could conclude the app uses SQLite (it does not — V04-03 proves zero sqlite in the dependency tree).
- Impact: documentation drift only; no behavioral defect.
- Root cause: carried-over text from the pre-file-store era; the audit flagged it but the cleanup never landed.
- Direction: change infra.rs:125 to "file-backed" and state.rs:931 to the real file path (or a generic dir), as part of the audit's P2 doc-cleanup item.
- Regression validation: grep for `sqlite|runtime_state.db` in CLI sources after the fix (expect only intentional framework references).
- Validation reports: [V04-03](../validations/A-BOOT-01/V04-03.md), [V05](../validations/A-BOOT-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (ownership map, both repos) | yes | passed | [V01-01](../validations/A-BOOT-01/V01-01.md) |
| V02 | Registration and runtime reachability (entry call graph, dead APIs) | yes | passed | [V02-01](../validations/A-BOOT-01/V02-01.md) |
| V03 | Invariant/edge cases | yes | passed | [V03-01](../validations/A-BOOT-01/V03-01.md) (option diff), [V03-02](../validations/A-BOOT-01/V03-02.md) (startup failure rollback), [V03-03](../validations/A-BOOT-01/V03-03.md) (shutdown/resource cleanup) |
| V04 | Targeted executable check | yes | passed | [V04-01](../validations/A-BOOT-01/V04-01.md) (`cargo check -p echo-agent-cli --locked`, exit 0), [V04-02](../validations/A-BOOT-01/V04-02.md) (`cargo check --no-default-features --features gui --bin echo-agent-tauri --locked`, exit 0), [V04-03](../validations/A-BOOT-01/V04-03.md) (`cargo tree` sqlite absence, grep exit 1) |
| V05 | Historical-document drift | yes | passed | [V05-01](../validations/A-BOOT-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `MASTER-PLAN.md`:17 "echo-agent-cli does not enable SQLite" | current | V04-03 (`cargo tree` zero sqlite) |
| `MASTER-PLAN.md`:69 one shared WebhookEmitter; watcher reloads hooks + webhook endpoints, model/MCP requires restart | current | desktop.rs:196 / modes.rs:58 overrides; config_watcher.rs:1-14; both entries spawn watcher (main.rs:232, desktop.rs:166) |
| `MASTER-PLAN.md`:73-74 plugin/scheduler binding + hook reload | current | main.rs:267-274, desktop.rs:233-240; watcher reload path |
| `docs/2026-07-17-surface-parity-closeout.md` foreground/background/cron on all surfaces | regressed (three gaps) | channels-only has no task service/scheduler (main.rs:391-403); GUI has no session-end review (V02-01); channels has no Dreaming (call sites tui/mod.rs:1999, desktop.rs:247, repl.rs:106) |
| `docs/2026-07-16-agent-lifecycle-audit.md`:163 recover_incomplete sets Running->Failed | fixed | store.rs:1771 transitions to Paused; boot sites main.rs:52, state.rs:557 (P1-8 comment) |
| `docs/2026-07-16-agent-lifecycle-audit.md`:133 `task_runtime.db`/SQLite stale mentions as P2 doc cleanup | current (unfixed) | infra.rs:125, state.rs:931 (A-BOOT-01-P3-06) |
| `MASTER-PLAN.md` "TUI/GUI parity" boot wiring (bootstrap shared) | current | single `AgentRuntime::bootstrap`, two call sites (V01-01) |

## Coverage And Uncertainty

- No process was launched; all behavior claims are static call-graph evidence.
  Dynamic checks (cron tick during teardown, PTY orphan behavior, GUI
  session-end review absence) need the Q-*/X-* dynamic suites.
- GUI session-end review: absence proven by grep of production callers of
  `on_session_end`; a session-end review could theoretically be triggered via
  the review panels command path, but no automatic GUI trigger exists.
- The browser-runtime-on-startup-failure rollback (dropped without shutdown)
  is noted in V03-02 as bounded-by-process-exit; whether the Playwright
  sidecar survives an abrupt exit is a framework MCP question (F-INT-01).
- B-PATH-01's findings (P2-01 MCP health GUI-only, P2-02 task-store
  duplication, P3-01 dead args, P3-02 GUI argv) were read and are NOT
  duplicated here; this task's P2-01 extends P2-02 with a third construction
  site.

## Handoff

- Downstream tasks may rely on: one shared bootstrap (runtime.rs:73) reached
  once per entry; per-entry service assembly map (V02-01); the three parity
  gaps (channels services, GUI session-end review, channels Dreaming) and the
  temporary-AppState duplicate construction (P2-01/02/03).
- Reports to read: this report + B-PATH-01 (entry inventory) + V01-V05.
- `A-SRF-04` should own the channels-only service gap (P2-02) and combined-
  mode shutdown (P3-04); `A-MEM-01`/`A-HITL-01` should evaluate the GUI
  session-end review gap (P2-03); `A-TSK-03` should confirm scheduler/task
  service stop semantics (P3-02); `X-SRF-01` must add rows for per-surface
  task service, scheduler, session-end review, MCP health, and Dreaming.
- This report becomes stale if entry dispatch (main.rs:75-91), bootstrap
  steps (runtime.rs:73-369), `start_headless_services` (modes.rs:32-64), or
  the desktop assembly (desktop.rs:124-271) change.
