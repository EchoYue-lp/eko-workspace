# B-PATH-01: EKO entry-point and composition inventory

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (baseline; not the focus of this task)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Which startup constructors and live entry points assemble TUI, GUI, CLI,
channel, cron, and background capabilities?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/src/main.rs` (full, 572 lines)
- `echo-agent-cli/src-tauri/src/main.rs` (full, 7 lines)
- `echo-agent-cli/src/lib.rs` (full, 19 lines)
- `echo-agent-cli/src/cli/{mod,args,modes,channels,rs,repl}.rs`
- `echo-agent-cli/src/tauri/{mod,desktop,state}.rs`
- `echo-agent-cli/src/tui/mod.rs` (entry-relevant slices)
- `echo-agent-cli/echo-agent-app-core/src/runtime.rs` (full, 728 lines)
- `echo-agent-cli/echo-agent-app-core/src/state.rs` (1-750)
- `echo-agent-cli/echo-agent-app-core/src/infra.rs` (1-1500 + entry-relevant tails)
- `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs` (entry-relevant slices)
- `echo-agent-cli/echo-agent-app-core/src/scheduler/{mod,runner}.rs`
- `echo-agent-cli/Cargo.toml`, `build.rs`, `tauri.conf.json`,
  `src-tauri/capabilities/default.json`

## Out Of Scope

Deferred to downstream tasks:

- **B-ARCH-01 / B-BASE-01**: framework crate layering, full manifest
  inventory, and CI-vs-AGENTS gate comparison.
- **A-TASK-*** (TaskRuntime): full audit of `tasks/task_runtime/` store
  internals, recovery semantics, and DAG validation.
- **A-POOL-*** (AgentPool): full audit of pool concurrency, eviction, and
  `PooledAgent` lifecycle.
- **A-CHN-*** (channels): full audit of `AppChannelMessageHandler` and
  IM channel semantics.
- **F-RCT-*** (ReAct engine): the framework's run/turn loop, not the
  application's call graph.
- **Q-CMP-*** (compile matrix): per-feature `cargo check` validation.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (workspace instructions).
- `docs/comprehensive-review/README.md` (review invariants and artifact model).
- `docs/comprehensive-review/REPORTING.md` (task and validation contracts).
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (B-PATH-01 card and `B-BASE-01`
  dependency).

Dependency reports read: none. `B-BASE-01` is the declared dependency but
its report is `needs_rerun`/in progress at the time of this review. The
required baseline facts (commits, features, binary targets, workspace
members) were re-verified directly from `echo-agent-cli/Cargo.toml`,
matching what B-BASE-01 would have established.

Historical documents treated as hypotheses:

- `docs/memory-evolution-full-audit.md` (claims about Dreaming parity;
  re-validated against current code — see Historical Claim Status).

## Layering Decision

This is an **application-layer** task. All inspected paths are in
`echo-agent-cli` (the EKO product). The only framework types touched are
the consumer-facing APIs:

- `echo_agent::agent::CancellationToken`
- `echo_agent::config::AppConfig`, `TuiConfig`
- `echo_agent::memory::{ConversationStore, Store, FileStore, InMemoryStore,
  ConversationFilter}`
- `echo_agent::state::{RuntimeStateStore, FileRuntimeStateStore}`
- `echo_agent::scheduler::{SchedulerRunner, CronTask, CronTaskStore, FireFn}`
- `echo_agent::paths`, `echo_agent::plugin`
- `echo_agent::channels::{ChannelManager, ...}` (channels feature)
- `echo_agent::telemetry::TelemetryConfig` (telemetry feature)
- `echo_agent::tasks::register_task_tools` (the framework's task-tool
  registration helper used post-bootstrap)

No cross-repository movement of code is recommended by this review. The
two parity findings (P2-01, P2-02) propose moving **calls** within the
application layer, not code between layers.

### Repository-wide duplicate search

Searched both `echo-agent/` and `echo-agent-cli/` for:

- `build_task_runtime_store_for_headless` (name) — single definition in
  `src/main.rs:35`; no duplicate.
- `TaskRuntimeStore::new` / `new_in_memory` (constructor) — defined in
  app-core `tasks/task_runtime/`; called from `src/main.rs:37, 41` (headless
  path) and `state.rs:548, 552` (GUI/AppState path). **Two parallel
  construction sites** — flagged as B-PATH-01-P2-03.
- `recover_incomplete` — defined in `tasks/task_runtime/`; called from
  `src/main.rs:52` and `state.rs:557`. Two parallel recovery sites.
- `start_headless_services` — single definition in `src/cli/modes.rs:32`;
  callers: `src/main.rs:258` (TUI), `src/cli/modes.rs:83` (CLI via
  `run_cli_mode`). Not called from `run_channels_mode` or `desktop.rs`.
- `spawn_dreaming_task` — single definition in
  `echo-agent-app-core/src/infra.rs:1143`; three callers: TUI
  (`tui/mod.rs:1999`), CLI (`repl.rs:106`), GUI (`desktop.rs:247`).
- `spawn_mcp_health_check` — single definition in
  `echo-agent-app-core/src/infra.rs:1111`; **single caller**: GUI
  (`desktop.rs:243`).
- `init_logging*` — single definition family in `infra.rs:1503-1622`;
  callers in `src/main.rs` and `src/tauri/desktop.rs`.

No duplicate definitions found. The duplications flagged as findings are
**call-site duplications**, not definition duplications.

## Current Path

### Two-binary, three-dispatch topology

```
                          ┌─── echo-agent-cli bin (src/main.rs) ───┐
                          │   main() at line 60                     │
                          │                                         │
echo-agent-cli workspace ─┤   ┌─ [gui, not tui]: run_desktop_entry   │
                          │   │   (src/main.rs:75-76)                 │
                          │   │                                       │
                          │   ├─ [tui]: run_tui_or_cli_entry          │
                          │   │   (src/main.rs:78-81)                 │
                          │   │                                       │
                          │   ├─ [channels, not gui, not tui]:        │
                          │   │   run_tui_or_cli_entry (line 83-86)   │
                          │   │                                       │
                          │   └─ [none]: compile_error (line 88-91)   │
                          │                                         │
                          └─── echo-agent-tauri bin (src-tauri/...) ─┘
                              main() → run_desktop_entry (always)
                              (requires gui feature)

run_tui_or_cli_entry (src/main.rs:95):
  1. Args::parse + load_config + apply_env_overrides
  2. WebhookEmitter::from_config
  3. is_tui_entry = args.tui || (!web && !cli && !channels)
  4. init_logging (TUI file vs Stderr, gated by feature = "tui")
  5. create_conversation_store + resume/continue handling
  6. AgentCreateParams{all None except model/project/conversation_id}
  7. AgentRuntime::bootstrap(&app_config, params)  ←─ SHARED ROOT
  8. inject_conversation_store
  9. build_task_runtime_store_for_headless  → register_task_tools_on_agent
  10. AgentPool::from_runtime → bind_task_execute_to_pool → spawn_cleanup_monitor
  11. spawn_config_watcher(cancel_token)
  12. ── DISPATCH ──
       a. is_tui_entry (TUI feature): swap HITL repl→tui;
          cli::start_headless_services → bind_scheduler;
          tui::run_tui; session-end review; shutdown; return
       b. args.channels (channels feature):
          tokio::spawn(cli::run_channels_mode(...))
          if run_cli: also cli::run_cli_mode(...)
          else: await channels_handle + shutdown_signal; return
       c. args.cli (always): cli::run_cli_mode(...)
          (run_cli_mode internally calls start_headless_services)

run_desktop (src/tauri/desktop.rs:124):
  1. dotenvy + load_shell_env (macOS) + cli::Args::parse_from + load_config
  2. WebhookEmitter::from_config
  3. init_logging (Stderr)
  4. AgentCreateParams{all None}
  5. AgentRuntime::bootstrap(&app_config, params)  ←─ SHARED ROOT
  6. spawn_config_watcher
  7. scheduler_store = FileStore at persistence/scheduler_store
  8. create_conversation_store + inject_conversation_store
  9. AppState::from_shared → with_review_integration/prompt_assembly/plugin_runtime
  10. register_task_tools_on_agent(state.tasks.runtime)
  11. runtime.init_pool(...) → bind_task_execute_to_pool → set_pool
  12. state.start_task_service() (BackgroundTaskService)
  13. state.start_scheduler_with_store(Some(scheduler_store))
  14. plugin_runtime.bind_scheduler
  15. spawn_mcp_health_check   ←─ GUI ONLY
  16. spawn_dreaming_task      ←─ GUI (also TUI/CLI elsewhere)
  17. build_tauri_app(state, browser_runtime).run(tauri::generate_context!())
      → 5 plugins + ~200 IPC handlers + 2 setup hooks
  18. on Tauri exit: cancel + shutdown_hook_events + browser.shutdown
```

### Composition-root ownership DAG

```
AgentRuntime::bootstrap (echo-agent-app-core/src/runtime.rs:73)
  │
  ├─ AgentHandle = Arc<RwLock<ReactAgent>>            (process)
  ├─ HitlDispatcher (Arc)                             (process)
  ├─ AppConfig (clone)                                (process)
  ├─ KeywordClassifier                                (process)
  ├─ state_store: Option<Arc<dyn RuntimeStateStore>>  (process, FileRuntimeStateStore)
  ├─ review_integration: Option<Arc<ReviewIntegration>>(process)
  ├─ browser_runtime: Arc<BrowserRuntime>             (process)
  ├─ prompt_assembly: PromptAssembly                  (process, captured at build)
  └─ plugin_runtime: Arc<PluginRuntimeService>        (process)
        │
        ▼
  AgentPool::from_runtime (agent_pool.rs:219)
   │  extracts SharedResources from primary agent
   │  pre-creates __background__ agent if configured
   │
   ├─ pooled conversation agents (lazy, per conversation_id)
   ├─ __task__: prefixed task subagents (lazy)
   └─ __cron__:{task_id}:{fire_id} (per cron run, released after)

AppState::from_shared (state.rs:454)        ← GUI path
  OR
build_task_runtime_store_for_headless       ← headless path (TUI/CLI/channels)
  (src/main.rs:35) — TaskRuntimeStore built outside AppState
        │
        ▼
  SchedulerRunner (scheduler/runner.rs:23)
    alias for framework echo_agent::scheduler::SchedulerRunner
    fire_fn routes every cron fire through launch_cron_run (Phase 3.1)
```

### Identities and recovery

- **Process identity**: `cache_user_id` (`infra.rs:160-176`) — loaded or
  created once at `~/.eko/cache_user_id`, shared by primary + subagents
  for provider KV cache partitioning.
- **Conversation identity (primary)**: `default_primary_conversation_id()`
  (`infra.rs:152-154`) returns `"primary-{uuid}"` — fresh per process,
  unless `--continue`/`--resume` overrides at `main.rs:128-150`.
- **Conversation identity (pooled)**: caller-supplied key — GUI uses
  frontend conversation id; channels use `channel:{channel_id}:{sender_id}`
  (`channels.rs:66`); cron uses `__cron__:{task_id}:{fire_id}`.
- **TaskRuntime recovery**: `TaskRuntimeStore::recover_incomplete()` runs
  exactly once per process — at `main.rs:52` (headless) or `state.rs:557`
  (GUI). Interrupted runs are promoted to resumable Paused state.

### Terminal and recovery points

- **TUI shutdown**: `main.rs:329-336` — `shutdown_hook_events`,
  `browser_runtime.shutdown`, `drop(runtime)`, `cancel_token.cancel()`.
- **CLI shutdown**: `main.rs:437-445` — same pattern.
- **Channels shutdown**: `modes.rs:228-232` — `shutdown_signal().await`
  → `manager.stop_all()`.
- **GUI shutdown**: `desktop.rs:261-268` — Tauri exits → `cancel_token`,
  `shutdown_hook_events`, `browser_runtime.shutdown`.
- **Recovery on next boot**: TaskRuntimeStore recovers incomplete runs;
  ConversationStore replays messages on `--resume`/`--continue`.

## Findings

### B-PATH-01-P2-01: MCP health check is GUI-only — TUI/CLI/channels parity gap

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - Definition: `echo-agent-cli/echo-agent-app-core/src/infra.rs:1111-1130`
    (`pub fn spawn_mcp_health_check(state: Arc<AppState>, cancel:
    CancellationToken)`)
  - Sole caller: `echo-agent-cli/src/tauri/desktop.rs:243` (GUI)
  - Consumer: `AppState::run_mcp_health_check` at
    `echo-agent-cli/echo-agent-app-core/src/state.rs:751`, invoked from the
    spawned task at `infra.rs:1125`.
  - State field fed by the loop: `PluginState.mcp_health` at
    `state.rs:358` (`RwLock<HashMap<String, McpHealthStatus>>`).
- Reachability: definition (`infra.rs:1111`) → registration
  (`desktop.rs:243`) → live caller (GUI process only). The function is
  `pub` and has no feature gate beyond the implicit `gui` gate on
  `pub mod tauri`.
- Expected invariant: AGENTS.md — "TUI、GUI(以及 CLI/channel)必须功能
  对等" / "禁止以'某模式不需要'为由拒绝给该模式接入能力". The MCP health
  loop is a product-level service (periodic 30s check, writes to
  AppState.mcp_health) that should be available in every user-facing mode.
- Observed behavior: only GUI spawns the loop. TUI, CLI, channels, and
  channels+CLI never start it; their `mcp_health` map stays empty. The GUI
  surfaces MCP health in the panels UI; the other modes have no on-demand
  equivalent — a server that goes unhealthy is invisible until a tool call
  fails.
- Impact: TUI/CLI/channels users lose live MCP server health visibility.
  Not a correctness bug; a feature-parity gap that violates the AGENTS.md
  invariant.
- Root cause: the loop was historically added as part of the GUI/Tauri
  surface and never wired into the headless bootstrap path. The headless
  `start_headless_services` helper at `modes.rs:32-64` starts
  `task_service` and `scheduler` but not MCP health.
- Direction: call `spawn_mcp_health_check` from `start_headless_services`
  (`src/cli/modes.rs`) so TUI/CLI/channels+CLI all run it. For pure
  channels mode (which skips `start_headless_services`), either route
  through `start_headless_services` after refactoring (see P2-02) or add a
  dedicated call site in `run_channels_mode`. The call requires an
  `Arc<AppState>` — currently `start_headless_services` builds AppState
  transiently and drops it (see P3-01); that pattern would need to keep
  the Arc alive or refactor the loop to not depend on full AppState.
- Regression validation: a test that asserts `spawn_mcp_health_check` is
  invoked once per process in TUI, CLI, channels, and GUI modes
  (mockable via a counter on the spawned task). Manual: start TUI, stop an
  MCP server, observe `mcp_health` updates via `/mcp` slash command or
  equivalent.
- Validation reports:
  [V02-01](../validations/B-PATH-01/V02-01.md) (composition root),
  [V04-01](../validations/B-PATH-01/V04-01.md) (parity matrix GAP-1).

### B-PATH-01-P2-02: Channels-only mode skips `start_headless_services` (no scheduler, no BackgroundTaskService)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/main.rs:357-405` — channels branch spawns
    `cli::run_channels_mode(...)` and (when `run_cli` is false) awaits
    `channels_handle` then `shutdown_signal`.
  - `echo-agent-cli/src/cli/modes.rs:117-235` — `run_channels_mode`
    builds a `ChannelManager`, registers channels, and calls
    `manager.start_all(handler_factory)`. It does NOT call
    `start_headless_services`.
  - `echo-agent-cli/src/cli/modes.rs:32-64` — `start_headless_services`
    calls `state.start_task_service()` (line 61) and
    `state.start_scheduler_with_store(...)` (line 62). Both are skipped.
- Reachability: `run_channels_mode` is reachable when `args.channels` is
  true and the `channels` feature is on (`main.rs:357-371`). The pure
  channels path is live in production for IM bot scenarios.
- Expected invariant: AGENTS.md — TUI/GUI/CLI/channels must be
  functionally equivalent. Cron scheduling and the unified
  BackgroundTaskService are core agent capabilities.
- Observed behavior: in pure channels mode, the in-process
  `SchedulerRunner` is never started, so `cron::*` slash commands and
  `add_scheduler_task` have no runner. The `BackgroundTaskService` is
  never started, so `submit_run` / `submit_dag` (the GUI's
  `commands::tasks::submit_task` path) is unavailable. The
  `TaskRuntimeStore` itself IS passed into `AppChannelMessageHandler.store`
  (`channels.rs:37`), so ad-hoc `task_create` / `task_execute` from the
  agent works; only the user-facing scheduler and background-submission
  surfaces are missing.
- Impact: IM-bot-only deployments cannot schedule cron tasks and cannot
  receive background task submissions through the unified service. Cron
  jobs configured in `echo-agent.yaml` will never fire.
- Root cause: `run_channels_mode` was written to be self-contained (build
  channels, run until shutdown) and predates the unified
  `start_headless_services` helper. The headless helper was added for
  TUI/CLI parity but not extended to channels.
- Direction: call `start_headless_services` from `run_channels_mode`
  (passing the already-built `pool`, `task_runtime_store`,
  `webhook_emitter`, `app_config`). The returned
  `(task_service, scheduler_runner)` should be held for the lifetime of
  the channels process and shut down alongside `manager.stop_all()`. This
  is a small change but needs care: `start_headless_services` builds an
  AppState transiently (see P3-01) — that pattern should be cleaned up
  first or in the same change.
- Regression validation: a test that starts channels mode with a fake
  channel and asserts `SchedulerRunner` and `BackgroundTaskService` are
  both `Some` after startup. Manual: configure a cron task in
  `echo-agent.yaml`, start with `--channels`, observe the cron firing.
- Validation reports:
  [V01-01](../validations/B-PATH-01/V01-01.md) (cron reachability),
  [V02-01](../validations/B-PATH-01/V02-01.md) (per-mode assembly table),
  [V04-01](../validations/B-PATH-01/V04-01.md) (parity matrix GAP-2).

### B-PATH-01-P2-03: TaskRuntimeStore + recovery constructed in two parallel paths

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - Headless path: `echo-agent-cli/src/main.rs:35-57`
    `build_task_runtime_store_for_headless()` — calls
    `TaskRuntimeStore::new()` → fallback `new_in_memory()` →
    `store.recover_incomplete()` → wrap in `Option<Arc<...>>`.
  - GUI path: `echo-agent-cli/echo-agent-app-core/src/state.rs:547-566`
    inside `AppState::from_shared` — same sequence (`new` →
    `new_in_memory` fallback → `recover_incomplete` → `Arc::new`), inline.
- Reachability: both paths are reachable in production — headless for
  TUI/CLI/channels, GUI path for Tauri.
- Expected invariant: AGENTS.md "实现前门禁:严禁平行实现同一语义"
  (no parallel implementation of the same semantics) and "动手前先查'是不是
  已经有了'" (check first). Recovery semantics for interrupted runs are a
  single concern and should have one construction site.
- Observed behavior: the two paths produce semantically identical stores
  (both call the same constructors), but the duplication means:
  (a) future changes to recovery (e.g. adding a boot-time validation pass)
  must be applied in two places; (b) the two sites log differently
  (`main.rs:42-46` uses two-level error logging, `state.rs:549-563` uses a
  single warn + info); (c) the headless site returns `Option` (can be
  `None` if both factories fail), while the GUI site also returns `Option`
  but is later unwrapped with `ok_or` in some downstream paths
  (`service.rs:208-209`).
- Impact: maintainability hazard, not a runtime bug today. Risk of silent
  drift if one site is updated and the other isn't.
- Root cause: `build_task_runtime_store_for_headless` was added when
  headless modes gained TaskRuntime parity (the historical "TUI doesn't
  use task runtime" gap, now fixed). The GUI path inside AppState was not
  refactored to call the new helper.
- Direction: extract the construction+recovery sequence into a single
  `pub fn build_task_runtime_store() -> Option<Arc<TaskRuntimeStore>>`
  in `echo-agent-app-core/src/infra.rs` (next to
  `create_runtime_state_store`). Both `main.rs:35-57` and
  `state.rs:547-566` call it. `AppState::from_shared` keeps its current
  `tasks.runtime` field but populated by the shared helper. Delete the
  inline duplication.
- Regression validation: existing tests that exercise
  `TaskRuntimeStore::new` / `new_in_memory` / `recover_incomplete` should
  still pass. Add a unit test for the new helper that asserts recovery is
  called exactly once.
- Validation reports:
  [V02-01](../validations/B-PATH-01/V02-01.md) (composition roots,
  "Composition root 2" + TaskRuntimeStore construction section).

### B-PATH-01-P3-01: `start_headless_services` builds full AppState only to discard it

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/cli/modes.rs:32-64`
  `start_headless_services`. The function constructs
  `AppState::from_shared(...)` (line 57) — which initializes all 14
  sub-states including `StorageState.tool_executions`
  (`ToolExecutionRepository::open`, `state.rs:507-534`),
  `WorkspaceRegistry::new()` (`state.rs:577-590`), `SessionSearchEngine`
  with full reindex (`state.rs:500-505`), etc. — purely to call
  `state.start_task_service()` (line 61) and
  `state.start_scheduler_with_store` (line 62). The AppState is then
  dropped (only `state.tasks.service` and `state.scheduler.runner` are
  cloned out at line 63).
- Reachability: called from `src/main.rs:258` (TUI) and from
  `run_cli_mode` at `modes.rs:83` (CLI).
- Expected invariant: AGENTS.md "动手前先查'是不是已经有了'" and the
  YAGNI principle — don't construct what you don't use.
- Observed behavior: every TUI/CLI startup pays the cost of opening the
  tool-execution repository, reindexing sessions, and initializing a
  workspace registry, none of which are used by TUI/CLI (which use the
  agent + pool directly, not AppState). This is wasted I/O at startup and
  a confusing code pattern (the call site looks like it starts two
  services; it actually builds the full 14-field AppState).
- Impact: slower startup for TUI/CLI (mostly the session reindex); minor
  confusion for maintainers. No correctness impact.
- Root cause: `start_scheduler_with_store` and `start_task_service` are
  methods on `AppState`, so the helper has to own an AppState to call
  them. When the helper was written, the methods were not free functions.
- Direction: either (a) extract `start_task_service` and
  `start_scheduler_with_store` into free functions that take only the
  inputs they need (agent, pool, task_runtime_store, scheduler_store,
  webhook_emitter), so the helper doesn't need AppState; or (b) keep
  AppState but make the expensive sub-states lazy (e.g.
  `SessionSearchEngine::reindex_all` deferred until first search query).
  Option (a) is cleaner but a larger refactor.
- Regression validation: existing tests cover scheduler and task service
  startup; they should still pass. Add a startup-cost smoke test if
  performance is a concern.
- Validation reports:
  [V02-01](../validations/B-PATH-01/V02-01.md) (composition root 2,
  Deviations section).

### B-PATH-01-P3-02: Dreaming task not spawned in channels-only mode

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - Definition: `echo-agent-cli/echo-agent-app-core/src/infra.rs:1143-1201`
    `spawn_dreaming_task`.
  - TUI caller: `echo-agent-cli/src/tui/mod.rs:1999`.
  - CLI caller: `echo-agent-cli/src/cli/repl.rs:106`.
  - GUI caller: `echo-agent-cli/src/tauri/desktop.rs:247`.
  - Channels caller: none. `run_channels_mode`
    (`modes.rs:117-235`) does not call it.
- Reachability: pure channels mode (main.rs:357-405 with `run_channels`
  set, `run_cli` false).
- Expected invariant: AGENTS.md TUI/GUI/CLI/channels parity for memory
  evolution. The historical audit `docs/memory-evolution-full-audit.md`
  explicitly flags Dreaming parity as a fixed concern (line 632: "三端均已
  接入") but the fix only covered TUI and CLI; channels was missed.
- Observed behavior: IM-bot-only processes never run the daily
  memory-evolution pass. Since channels processes are typically
  long-lived (running until shutdown_signal), this is the mode that would
  benefit most from automatic memory promotion/demotion.
- Impact: memory layer staleness for long-running channels deployments.
  Not a correctness bug; missed opportunity.
- Root cause: the Dreaming parity fix was applied to TUI and CLI
  (`repl.rs:100-114` comment) but not extended to channels mode, which has
  a separate entry path.
- Direction: call `spawn_dreaming_task` from `run_channels_mode` with a
  cancellation token that is cancelled alongside `manager.stop_all()`.
  This is a 5-line change once P2-02 is decided (the call needs the
  pool and primary agent, both already available in
  `run_channels_mode`'s scope).
- Regression validation: manual — start channels mode, wait 60s for the
  initial Dreaming pass, observe log line "Dreaming pass completed" or
  "Dreaming task stopped".
- Validation reports:
  [V04-01](../validations/B-PATH-01/V04-01.md) (parity matrix GAP-3).

### B-PATH-01-P3-03: Session-end memory review runs only in TUI

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - Sole caller: `echo-agent-cli/src/main.rs:307-327` (TUI mode block),
    gated behind `tui_result.is_ok() &&
    runtime.review_integration.is_some()`.
  - The block calls `review_integration.on_session_end().await` and prints
    a one-line summary (scanned/stale/conflicts/proposals).
- Reachability: TUI mode only.
- Expected invariant: AGENTS.md TUI/GUI/CLI parity for memory and review
  capabilities.
- Observed behavior: when a GUI window closes (`desktop.rs:261`) or a CLI
  REPL exits (`repl.rs`), no equivalent `on_session_end` call is made.
  The daily Dreaming pass partially compensates (it runs
  `run_dreaming_pass`, which is a different trigger — promotion/demotion,
  not the session-scope review report).
- Impact: GUI and CLI users never see the session-end memory review
  summary. Low impact — the underlying ReviewIntegration is shared and
  the data is not lost, just not surfaced at session end.
- Root cause: the session-end hook was added to TUI as a UX nicety and
  not propagated.
- Direction: add an equivalent `on_session_end` call to (a) `desktop.rs`
  just before `tauri_result` is returned (line 256-268), surfacing the
  summary via a final Tauri event or log; (b) `repl.rs` session exit
  path. The exact UX (print to stderr, emit event, log only) is a product
  decision.
- Regression validation: manual — run a TUI session, exit, confirm the
  review summary still prints; run a CLI session, exit, confirm the new
  summary prints; same for GUI.
- Validation reports:
  [V04-01](../validations/B-PATH-01/V04-01.md) (parity matrix GAP-4).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Entry-point call graph and `#[cfg(feature = ...)]` gate inventory | yes | passed | [V01-01](../validations/B-PATH-01/V01-01.md) |
| V02 | Composition-root inventory (AgentRuntime, AppState, AgentPool) and TaskRuntimeStore recovery | yes | passed | [V02-01](../validations/B-PATH-01/V02-01.md) |
| V03 | Feature-gated reachability per feature (tui/gui/channels/telemetry/devtools) | yes | passed | [V03-01](../validations/B-PATH-01/V03-01.md) |
| V04 | Mode-to-service matrix and parity gap identification | yes | passed | [V04-01](../validations/B-PATH-01/V04-01.md) |
| V05 (historical drift) | Re-validate claims from `memory-evolution-full-audit.md` | conditional | covered inline in "Historical Claim Status" below; no separate report | - |
| V04 (executable) | Compile/run each mode to confirm services actually start | conditional | not_run — deferred to Q-* quality gate tasks; out of scope for a read-only inventory task | - |

No executable validation was attempted because B-PATH-01 is a static
inventory task. Compile/run coverage belongs to Q-CMP-* and the
per-feature A-*/F-* audits.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `docs/memory-evolution-full-audit.md:576` "Dreaming only in Tauri desktop" | **fixed** for TUI/CLI; **stale** for channels | `tui/mod.rs:1999`, `repl.rs:106`, `desktop.rs:247` all call `spawn_dreaming_task`; channels mode does not — see B-PATH-01-P3-02. |
| `docs/memory-evolution-full-audit.md:632` "三端均已接入" (Dreaming wired in all three modes) | **regressed** for channels | the fix covered TUI/GUI/CLI but channels was never added — see B-PATH-01-P3-02. |
| `docs/memory-evolution-full-audit.md:805` "spawn_dreaming_task only in desktop.rs" | **fixed** | now in TUI, CLI, GUI (see above). |
| AGENTS.md historical lesson "TUI doesn't use the task runtime" comment in main.rs | **fixed** | `src/main.rs:172-182` now explicitly states "Every headless surface is a full Agent surface" and builds TaskRuntimeStore unconditionally; `runtime.rs:120-126` documents post-bootstrap tool registration for both TUI and GUI. |
| AGENTS.md historical lesson "security gates broke terminal/MCP default config" | **current** (no longer applies to entry paths) | no `require_full_auto` gates on `create_terminal` or MCP connect paths in the inspected entry code; terminal is wired via `tauri::terminal::*` IPC handlers without permission gates. |

## Coverage And Uncertainty

### Code inspected deeply

- All entry-point files (`main.rs`, `src-tauri/src/main.rs`, `lib.rs`).
- The two composition roots (`runtime.rs`, `state.rs` 1-750).
- The headless service helper (`modes.rs` full).
- The Tauri desktop entry and builder (`desktop.rs`, `tauri/mod.rs` 1-100
  for plugin/handler registration; the 200+ IPC handlers' bodies were not
  individually inspected).
- Feature declarations and build script (`Cargo.toml`, `build.rs`,
  `tauri.conf.json`).

### Code inspected shallowly or not at all

- The bodies of the ~200 Tauri IPC commands in `src/tauri/commands/`
  (only their registration in `tauri/mod.rs:69-310` was inventoried).
  Per-command reachability and correctness belong to A-* tasks.
- The TUI event loop (`tui/events.rs`) and rendering (`tui/ui.rs`,
  `tui/widgets/`) — only `run_tui` signature and pre-loop setup were
  inspected.
- The full CLI command registry (`cli/cmd_impls/*`) — only the
  registration calls in `repl.rs:160-180` were inventoried.
- The framework internals of `echo_agent::scheduler::SchedulerRunner`,
  `echo_agent::state::FileRuntimeStateStore`,
  `echo_agent::memory::FileConversationStore`, etc. — these are framework
  types consumed by the application; their internals belong to F-* tasks.
- The `TaskRuntimeStore` internal implementation in
  `echo-agent-app-core/src/tasks/task_runtime/` — only the constructor
  and recovery calls were inspected.

### Environmental limits

- No compilation or test execution (read-only review).
- The git worktree state was assumed clean based on the task brief; not
  independently verified.

### Claims that remain uncertain

- Whether the GUI-only `run_desktop_entry` shortcut at `main.rs:75-76`
  has any production user today beyond the Tauri bundler. The
  `tauri.conf.json:8-10` config builds with `--features gui`, so the
  bundler path is real, but I did not confirm whether anyone runs the
  `echo-agent-cli` binary directly with `--no-default-features --features
  gui`. Not material to the findings.
- Whether the `web` arg (now an error at `main.rs:351-354`) was fully
  removed from documentation and user-facing help. The arg is still
  declared `hide = true` in `cli/args.rs:22-23` and parseable; tests at
  `main.rs:534-544` confirm it parses. Not material to the findings.

## Handoff

### Conclusions downstream tasks may rely on

1. **Two binary targets, three dispatch trees, one shared composition
   root.** Any downstream task auditing an agent capability should start
   at `AgentRuntime::bootstrap` (`runtime.rs:73`) and trace through one
   of the three dispatch paths (TUI/CLI/channels via
   `run_tui_or_cli_entry`; GUI via `run_desktop`; cron via
   `SchedulerRunner` started from either).
2. **TaskRuntimeStore is constructed and recovered exactly once per
   process.** Downstream tasks auditing TaskRuntime internals do not need
   to worry about double-construction; they DO need to be aware of the
   two parallel call sites (P2-03) if they touch recovery.
3. **AgentPool is the single authority for per-conversation,
   per-task-subagent, and per-cron-run agents.** All concurrency goes
   through `AgentPool::acquire`; the pool key conventions are
   `conversation_id` (GUI), `channel:{ch}:{sender}` (channels),
   `__task__:{...}` (task subagents), `__cron__:{task_id}:{fire_id}`
   (cron).
4. **Cron always routes through `launch_cron_run` (Phase 3.1
   unification).** No legacy `[plan]` routing remains; the prefix is
   stripped for backward compatibility only.
5. **Feature parity is mostly achieved for TUI/GUI/CLI.** Channels mode
   has gaps (P2-02, P3-02). MCP health check is GUI-only (P2-01).
6. **The `web` entry is removed** (`main.rs:351-354`). Any downstream
   reference to web mode is stale.

### Reports downstream tasks must read

- B-ARCH-01 (when complete): for framework crate layering context.
- B-BASE-01 (when complete): for the canonical manifest/feature/target
  inventory. This task re-verified the baseline facts directly.

### Conditions that make this report stale

- Any change to `src/main.rs`'s `main()` or `run_tui_or_cli_entry`.
- Any change to `src/tauri/desktop.rs::run_desktop`.
- Any change to `src/cli/modes.rs::start_headless_services` or
  `run_channels_mode`.
- Any change to `runtime.rs::AgentRuntime::bootstrap` or
  `state.rs::AppState::from_shared`.
- Addition or removal of a Cargo feature.
- Moving `spawn_mcp_health_check` or `spawn_dreaming_task` to new call
   sites.

### Follow-up task IDs (suggested, not implemented)

- A task to audit the ~200 Tauri IPC handlers in `src/tauri/commands/`
  for reachability and correctness (likely A-* phase).
- A task to audit `TaskRuntimeStore` internals and recovery semantics
  (A-TASK-*).
- A task to audit `AgentPool` concurrency, eviction, and lifecycle
  (A-POOL-*).
- A task to audit `AppChannelMessageHandler` and IM channel semantics
  (A-CHN-*).
- A quality-gate task to actually compile and run each mode
  (Q-CMP-*), which would convert V01/V04 from static traces to dynamic
  evidence.

None of these are implemented by B-PATH-01 — this is a read-only review.
