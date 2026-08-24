# A-BOOT-01: Application composition and startup lifecycle

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: not-applicable (application-layer task; framework not modified)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Does each EKO entry point construct the same core services exactly once with
consistent config, working directory, shutdown, and reload behavior?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/src/main.rs` (full, 572 lines) — `main`, `run_tui_or_cli_entry`,
  `build_task_runtime_store_for_headless`.
- `echo-agent-cli/src-tauri/src/main.rs` (full, 7 lines) — dedicated GUI binary.
- `echo-agent-cli/src/tauri/desktop.rs` (full, 272 lines) — `run_desktop_entry`,
  `run_desktop`.
- `echo-agent-cli/src/cli/modes.rs` (full, 236 lines) — `start_headless_services`,
  `run_cli_mode`, `run_channels_mode`.
- `echo-agent-cli/echo-agent-app-core/src/runtime.rs` (full, 728 lines) —
  `AgentRuntime`, `bootstrap`, `into_app_state`, `init_pool`.
- `echo-agent-cli/echo-agent-app-core/src/state.rs` (280-720) — `AppState`,
  `from_shared`, `start_task_service`, `start_scheduler_with_store`,
  `TaskState::runtime` construction.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs`
  (entry-relevant: `new`, `new_in_memory`, `recover_incomplete`,
  `shutdown_hook_events`).

## Out Of Scope

Deferred to downstream tasks:

- **B-PATH-01**: full entry-point inventory (this task consumes its findings as
  inputs; it does not re-enumerate every IPC handler).
- **A-TASK-*** (TaskRuntime): recovery semantics correctness, DAG validation,
  store internals.
- **A-POOL-*** (AgentPool): pool concurrency and `PooledAgent` lifecycle.
- **A-CHN-*** (channels): `AppChannelMessageHandler` per-sender semantics.
- **B-ARCH-01 / B-BASE-01**: framework crate layering and manifest inventory.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (workspace instructions; multi-mode parity rule,
  layering gate, no-duplicate rule).
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (A-BOOT-01 card and B-PATH-01 dependency).

Dependency reports read:

- `zcode-glm/tasks/B-PATH-01.md` (complete) — entry-point inventory and the
  three parity findings (P2-01/02/03) that this task sharpens into boot-lifecycle
  findings.

Historical documents treated as hypotheses: none.

## Layering Decision

This is an **application-layer** task. All inspected paths live in
`echo-agent-cli` / `echo-agent-app-core` (the EKO product). The composition root
`AgentRuntime::bootstrap` (`runtime.rs:73`) is correctly placed in the
application core: it wires EKO-specific product policy (HITL dispatcher, built-in
skills, methodology baseline, review integration, plugin runtime, browser
runtime, intent router) onto the framework `ReactAgent`. No EKO product concepts
were found leaking into the framework during this audit.

Duplicate-search terms used across the whole `echo-agent-cli` tree:

- `TaskRuntimeStore::new` / `new_in_memory` / `recover_incomplete` — three
  construction sites found (see Current Path).
- `start_headless_services` / `start_task_service` / `start_scheduler_with_store`
  — scheduler/task service startup.
- `AgentRuntime::bootstrap` — single composition root (confirmed unique).
- `build_task_runtime_store_for_headless` — headless-only helper.

## Current Path

### Entry-point dispatch

`echo-agent-cli/src/main.rs:60` `async fn main()` dispatches by feature gate:

- `#[cfg(all(feature = "gui", not(feature = "tui")))]` →
  `echo_agent_cli::tauri::desktop::run_desktop_entry()` (`main.rs:76`). This is
  the path Tauri CLI uses when building the package-name binary with
  `--no-default-features --features gui`.
- `#[cfg(feature = "tui")]` → `run_tui_or_cli_entry()` (`main.rs:80`).
- `#[cfg(all(feature = "channels", not(feature = "gui"), not(feature = "tui")))]`
  → `run_tui_or_cli_entry()` (`main.rs:85`).
- The dedicated GUI binary `src-tauri/src/main.rs:5` calls
  `run_desktop_entry()` directly (own tokio runtime).

Every dispatch path sets the global data dir name first
(`set_user_data_dir_name(".eko")` + `set_plugin_data_base_dir_name(".eko")`)
before any path resolution — `main.rs:66-70` (headless) and `desktop.rs:70-72`
(GUI). Consistent.

### AgentRuntime::bootstrap — the single agent composition root

`runtime.rs:73-369`. Called exactly once per process from:
- headless: `main.rs:168`.
- GUI: `desktop.rs:160`.

Constructs, in order: runtime state store (`infra::create_runtime_state_store`),
default primary conversation id, shared `BrowserRuntime`, `ReactAgent` via
`infra::create_agent_with_diagnostics`, MCP config load, auto-compression,
`AgentHandle`, `HitlDispatcher` (with `repl` provider wired), built-in skills
load, methodology baseline injection, user hooks (single merged load),
`ReviewIntegration` (when a Store is available), LSP tools, `PluginRuntimeService`,
research-library tools, startup hook, `TriggerSupervisor` (Keyword → LLM → Hook
fusion) + `IntentRouter`. All of these are EKO product policy and correctly live
in the app core. Bootstrap does **not** construct the `TaskRuntimeStore` — that
is deferred to the caller (see Findings).

### TaskRuntimeStore construction — three sites, two parallel paths

This is the core asymmetry. `TaskRuntimeStore` is constructed and recovered in
three places:

1. `echo-agent-cli/src/main.rs:35-57` `build_task_runtime_store_for_headless()`
   — headless helper. Opens on-disk store, falls back to in-memory, calls
   `recover_incomplete()`, returns `Option<Arc<TaskRuntimeStore>>`. Used by
   `run_tui_or_cli_entry` (`main.rs:175`).
2. `echo-agent-app-core/src/state.rs:547-566` inside `AppState::from_shared` →
   `TaskState::runtime` field initializer. Same open-or-fallback + same
   `recover_incomplete()`. This runs for **every** `AppState::from_shared`
   caller, i.e. both GUI and headless-via-`start_headless_services`.
3. (No third distinct constructor; the two above are the parallel paths. The
   `new()` / `new_in_memory()` primitives at `store.rs:191` are the shared
   low-level layer both call.)

Call-site matrix:

| Entry | TaskRuntimeStore built where | Recovered where | Final store used |
|---|---|---|---|
| GUI (`run_desktop`) | `from_shared` (`desktop.rs:187`) | `from_shared` | `state.tasks.runtime` |
| TUI (`run_tui_or_cli_entry`) | `build_task_runtime_store_for_headless` (`main.rs:175`) **+** `from_shared` (via `start_headless_services` `modes.rs:57`) | **both** | `build_task_runtime_store_for_headless` one (`modes.rs:60` overwrites `state.tasks.runtime`) |
| CLI (`run_cli_mode`) | same as TUI (`modes.rs:83`) | **both** | same as TUI |
| Channels-only (`run_channels_mode`) | `build_task_runtime_store_for_headless` only (`main.rs:175`); `start_headless_services` is **never called** for the channels-only branch | headless helper only | headless helper one |

Net effect for TUI/CLI: `TaskRuntimeStore::new()` (or `new_in_memory()`) **and**
`recover_incomplete()` run **twice**; the store built inside `from_shared` is
discarded immediately when `modes.rs:60` overwrites `state.tasks.runtime`. The
first recovery mutates on-disk state, so the second recovery finds nothing —
functionally idempotent but wasteful and confusing. See A-BOOT-01-P2-01.

### Scheduler / BackgroundTaskService startup

Two paths, both idempotent (guarded by `if self.X.is_some() return;`):

- GUI: `state.start_task_service()` + `state.start_scheduler_with_store(...)`
  called directly on the long-lived `AppState` (`desktop.rs:231-232`).
- TUI/CLI: `start_headless_services` (`modes.rs:32`) builds a **throwaway**
  `AppState` via `from_shared` (`modes.rs:57`), starts services on it, then
  returns `(task_service, scheduler_runner)` as loose `Option<Arc<...>>` handles.
  The throwaway `AppState` is dropped; only the two `Arc`s escape.
- Channels-only: `start_headless_services` is **not called** — no
  `SchedulerRunner`, no `BackgroundTaskService` (confirms B-PATH-01-P2-02).

### Pool construction

Both entries build the pool through `AgentPool::from_runtime`:

- Headless: inline in `run_tui_or_cli_entry` (`main.rs:183-201`), then
  `spawn_cleanup_monitor`.
- GUI: `AgentRuntime::init_pool` (`desktop.rs:210`, `runtime.rs:397`), which
  also calls `spawn_cleanup_monitor`.

`init_pool` exists and is equivalent to the inline headless block, but headless
does not use it — minor duplication (see A-BOOT-01-P3-02).

### Conversation store + working directory

- Conversation store: both entries call `infra::create_conversation_store()`
  (`main.rs:127`, `desktop.rs:184`) and `infra::inject_conversation_store`.
  Consistent.
- Working directory: both pass `working_dir: None` in `AgentCreateParams`
  (`main.rs:162`, `desktop.rs:152`). Bootstrap resolves to current dir via
  `register_lsp_tools` (`runtime.rs:504-508`) and memory-store path resolution
  (`runtime.rs:229`). Consistent.

### Reload behavior

Both entries spawn `config_watcher::spawn_config_watcher` with the same shape
(`main.rs:232`, `desktop.rs:166`): resolves config path, takes `agent_handle` +
optional `webhook_emitter` + a `CancellationToken`. Reload is therefore
consistent across entries for hooks + webhook endpoints. MCP health check and
dreaming are **not** part of the watcher in either path.

### Shutdown / cleanup

| Entry | Shutdown sequence |
|---|---|
| TUI | `task_runtime_store.shutdown_hook_events()` (`main.rs:330`), `browser_runtime.shutdown()` (`main.rs:334`), `drop(runtime)` (`main.rs:335`), `cancel_token.cancel()` (`main.rs:336`) |
| CLI | `shutdown_hook_events` (`main.rs:439`), `browser_runtime.shutdown()` (`main.rs:443`), `drop(runtime)` (`main.rs:444`), `cancel_token.cancel()` (`main.rs:445`) |
| Channels-only | `shutdown_hook_events` (`main.rs:395`), `browser_runtime.shutdown()` (`main.rs:399`), `cancel_token.cancel()` (`main.rs:400`) — `runtime` not explicitly dropped (implicit at scope end) |
| GUI | `cancel_token.cancel()` (`desktop.rs:261`), `shutdown_hook_events` (`desktop.rs:263`), `browser_runtime.shutdown()` (`desktop.rs:267`) — `runtime` not explicitly dropped (implicit at scope end) |

`shutdown_hook_events` (`store.rs:261`) flushes the task hook dispatcher. There
is no explicit stop for `SchedulerRunner` / `BackgroundTaskService` in any entry
— they rely on their own `CancellationToken` and process exit. The config-watcher
is cancelled via the shared `cancel_token` in every entry.

## Findings

### A-BOOT-01-P2-01: TaskRuntimeStore is constructed by two parallel code paths; TUI/CLI build and recover it twice (once discarded)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/main.rs:35-57` (`build_task_runtime_store_for_headless`).
  - `echo-agent-cli/echo-agent-app-core/src/state.rs:547-566` (`TaskState::runtime`
    initializer inside `AppState::from_shared`).
  - `echo-agent-cli/src/cli/modes.rs:57-60` (`from_shared` builds a store, then
    `state.tasks.runtime = task_runtime_store` overwrites it).
- Reachability: `main()` → `run_tui_or_cli_entry` (`main.rs:95`) →
  `build_task_runtime_store_for_headless` (`main.rs:175`) **and**
  `start_headless_services` (`modes.rs:83` via `run_cli_mode`, or `modes.rs:57`
  via the TUI branch at `main.rs:258`) → `AppState::from_shared`. Both paths are
  live for every TUI and CLI launch.
- Expected invariant: each core service is constructed **exactly once** per
  process (the question this task asks), and a single authoritative construction
  site exists (AGENTS.md "严禁平行实现同一语义" / implementation gate rule 3).
- Observed behavior: TUI/CLI build the `TaskRuntimeStore` twice. `from_shared`
  builds store #1, calls `recover_incomplete()`, then `modes.rs:60` overwrites
  the field with store #2 (from the headless helper), which had already called
  `recover_incomplete()` on the same file. Store #1 is dropped. GUI builds the
  store once (inside `from_shared`) and does not use the headless helper — so
  the two entries use **different** construction sites for the same service.
- Impact: (1) violates the "exactly once" property the task asks about; (2) two
  parallel implementations of "open + recover" that can drift; (3) wasted I/O
  and a redundant recovery mutation on every TUI/CLI boot; (4) the headless
  helper and the `from_shared` initializer are easy to change independently,
  inviting future divergence.
- Root cause: `TaskState::runtime` is eagerly initialized inside `from_shared`
  (which is the GUI path's only source), but headless pre-builds its own store
  because it needs the `Option<Arc<...>>` handle before constructing the pool
  and registering tools. The two designs were never unified — `start_headless_services`
  papers over the duplication by overwriting the field.
- Direction: make `from_shared`'s `TaskState::runtime` the single source of
  truth. Either (a) have `from_shared` accept an optional pre-built store, or
  (b) have headless read the store back out of `state.tasks.runtime` after
  `from_shared` instead of pre-building. Delete `build_task_runtime_store_for_headless`
  or reduce it to a thin wrapper. The desired end state: exactly one
  `TaskRuntimeStore::new()` and one `recover_incomplete()` per process, on the
  same code path for all entries.
- Regression validation: integration test that boots the TUI/CLI entry and
  asserts `TaskRuntimeStore::new` is called exactly once (count constructor
  calls, or assert a single recovery log line). Manual: boot TUI, confirm a
  single "recovered incomplete task_runtime runs" / "Recovered interrupted
  task-runtime runs at boot" log entry.
- Validation reports: [V01](../validations/A-BOOT-01/V01-01.md),
  [V03](../validations/A-BOOT-01/V03-01.md)

### A-BOOT-01-P2-02: Channels-only entry skips start_headless_services — no SchedulerRunner, no BackgroundTaskService

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/main.rs:357-403` (channels-only branch: spawns
    `run_channels_mode`, never calls `start_headless_services`).
  - `echo-agent-cli/src/cli/modes.rs:32-64` (`start_headless_services` is the
    only starter of both services for headless).
- Reachability: `main()` → `run_tui_or_cli_entry` → `--channels` without `--cli`
  → `tokio::spawn(run_channels_mode(...))` (`main.rs:365`); the channels-only
  exit branch (`main.rs:392-403`) cleans up without ever having started the
  scheduler or background task service.
- Expected invariant: multi-mode functional parity — TUI and GUI are full Agent
  surfaces, and channels (per AGENTS.md "TUI 与 GUI 是功能完全一样的 Agent 完全体")
  must not silently drop core services. Cron-scheduled tasks and background
  tasks should be available in every long-running entry.
- Observed behavior: the channels-only path starts `ChannelManager` on the pool
  but never starts `SchedulerRunner` or `BackgroundTaskService`. Cron tasks and
  background tasks are unavailable in that mode. (Imported as
  B-PATH-01-P2-02; sharpened here as a boot-lifecycle parity gap.)
- Impact: a user running `echo-agent-cli --channels` as a long-running IM bot
  gets no scheduled cron runs and no background task execution — a silent
  capability gap versus GUI/CLI.
- Root cause: the channels branch was wired to `run_channels_mode` directly
  without routing through the shared headless service starter.
- Direction: call `start_headless_services` (or a shared starter) in the
  channels-only branch before spawning `run_channels_mode`, mirroring the
  TUI/CLI branches. Alternatively, fold service startup into a single
  `AppState`-based starter used by all headless branches.
- Regression validation: boot `--channels` with a fake cron definition and
  assert the scheduler fires; assert `BackgroundTaskService` is constructible.
- Validation reports: [V02](../validations/A-BOOT-01/V02-01.md)

### A-BOOT-01-P2-03: MCP health check and dreaming task are spawned only in GUI, not in TUI/CLI/channels

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - GUI spawn: `echo-agent-cli/src/tauri/desktop.rs:243`
    (`infra::spawn_mcp_health_check`) and `desktop.rs:247`
    (`infra::spawn_dreaming_task`).
  - Headless: `echo-agent-cli/src/main.rs:95-451` — grep for
    `spawn_mcp_health_check` / `spawn_dreaming_task` yields zero hits in the
    headless entry or `start_headless_services`.
- Reachability: `run_desktop` → both spawners are live for GUI. For TUI/CLI,
  `run_tui_or_cli_entry` never calls either; `start_headless_services`
  (`modes.rs:32-64`) does not call either.
- Expected invariant: per AGENTS.md, MCP health telemetry and stage-4 dreaming
  ("runs once after boot and then daily in every mode", per `desktop.rs:245-246`
  comment) should run in every entry, not only GUI.
- Observed behavior: only GUI gets MCP health polling and the daily dreaming
  pass. TUI/CLI/channels users get no MCP health status updates and no daily
  memory dreaming. (Imported as B-PATH-01-P2-01; sharpened here as a
  boot-lifecycle parity gap.)
- Impact: MCP server outages go undetected in TUI/channels until a tool call
  fails; memory dreaming / staleness review never runs for non-GUI users,
  degrading long-run memory quality.
- Root cause: the two spawners were added to the GUI entry directly instead of
  the shared `start_headless_services` (or `bootstrap`) path.
- Direction: move both spawners into the shared headless service starter (or
  `AgentRuntime::bootstrap`) so every entry gets them. Keep the existing
  `cancel_token` plumbing.
- Regression validation: boot TUI, assert `spawn_mcp_health_check` and
  `spawn_dreaming_task` tasks are live (log lines or task handle assertion).
- Validation reports: [V02](../validations/A-BOOT-01/V02-01.md)

### A-BOOT-01-P3-01: Headless builds the AgentPool inline instead of using AgentRuntime::init_pool

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - Inline headless pool build: `echo-agent-cli/src/main.rs:183-201`
    (`AgentPool::from_runtime` + `bind_task_execute_to_pool` +
    `spawn_cleanup_monitor`).
  - Shared helper that does the same: `echo-agent-app-core/src/runtime.rs:397-407`
    (`AgentRuntime::init_pool`).
  - GUI uses the helper: `echo-agent-cli/src/tauri/desktop.rs:210-215`.
- Reachability: `run_tui_or_cli_entry` always takes the inline path; GUI always
  takes `init_pool`. Both are live.
- Expected invariant: one authoritative pool-construction sequence (AGENTS.md
  rule 3).
- Observed behavior: headless duplicates the three-step pool setup
  (`from_runtime` → `Arc::new` → `spawn_cleanup_monitor`) that `init_pool`
  already encapsulates.
- Impact: low. The two sequences are functionally identical today, but any
  future change to pool init (e.g. adding a step) must be applied in two
  places, risking drift.
- Root cause: `init_pool` was added after the headless inline block; headless
  was not migrated.
- Direction: replace the inline headless block with `runtime.init_pool(...)`,
  then call `bind_task_execute_to_pool` (which `init_pool` does not cover).
- Regression validation: boot TUI, assert `pool.pool_size()` matches GUI's.
- Validation reports: [V02](../validations/A-BOOT-01/V02-01.md)

### A-BOOT-01-P3-02: Shutdown ordering differs between headless (drop runtime then cancel) and GUI (cancel then implicit drop)

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence:
  - Headless TUI/CLI: `main.rs:330-336` and `main.rs:438-445` —
    `shutdown_hook_events` → `browser_runtime.shutdown()` → `drop(runtime)` →
    `cancel_token.cancel()`.
  - GUI: `desktop.rs:261-267` — `cancel_token.cancel()` →
    `shutdown_hook_events` → `browser_runtime.shutdown()`; `runtime` is not
    explicitly dropped (implicit at scope end).
  - Channels-only: `main.rs:394-400` — `shutdown_hook_events` →
    `browser_runtime.shutdown()` → `cancel_token.cancel()`; no explicit
    `drop(runtime)`.
- Reachability: every entry's exit path.
- Expected invariant: consistent shutdown ordering across entries so that
  in-flight work is flushed before background tasks are cancelled.
- Observed behavior: GUI cancels the `CancellationToken` **before** flushing
  the task hook dispatcher, while headless flushes **before** cancelling. If
  `shutdown_hook_events` depends on any task kept alive by the token, GUI's
  ordering could truncate the flush. No bug observed today
  (`shutdown_hook_events` operates on its own dispatcher state), but the
  divergence is a latent footgun.
- Impact: low today; medium if a future hook depends on a token-gated task.
- Root cause: shutdown logic was written per-entry rather than shared.
- Direction: extract a single `shutdown(runtime, task_store, cancel_token)`
  helper and call it from all four exit paths, with a documented order (flush
  → shut down browser → cancel → drop).
- Regression validation: boot + clean exit in TUI and GUI, assert
  `shutdown_hook_events` completes before the token-cancelled tasks stop.
- Validation reports: [V04](../validations/A-BOOT-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | `AgentRuntime::bootstrap` is the single composition root; enumerate the services it constructs. | yes | passed | [V01-01](../validations/A-BOOT-01/V01-01.md) |
| V02 | TUI vs GUI entry construction and parity of services (TaskRuntimeStore, pool, scheduler, MCP health, dreaming). | yes | passed | [V02-01](../validations/A-BOOT-01/V02-01.md) |
| V03 | Bootstrap error handling and cleanup on failure. | yes | passed | [V03-01](../validations/A-BOOT-01/V03-01.md) |
| V04 | Shutdown / cleanup path inspection across entries. | yes | passed | [V04-01](../validations/A-BOOT-01/V04-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| B-PATH-01-P2-01 (MCP health check only in GUI) | current (sharpened into A-BOOT-01-P2-03) | `desktop.rs:243`; absence in `main.rs`/`modes.rs` |
| B-PATH-01-P2-02 (channels-only skips start_headless_services) | current (sharpened into A-BOOT-01-P2-02) | `main.rs:357-403` vs `modes.rs:32` |
| B-PATH-01-P2-03 (TaskRuntimeStore + recover_incomplete in two parallel paths) | current (sharpened into A-BOOT-01-P2-01) | `main.rs:35-57` + `state.rs:547-566` |
| `desktop.rs:245-246` comment "Dreaming runs ... daily in every mode" | stale (regressed) | dreaming is spawned only at `desktop.rs:247`, not in headless; "every mode" is not true today |

## Coverage And Uncertainty

- `bootstrap` was read in full; every service it constructs is enumerated in
  V01. The internals of `infra::create_agent_with_diagnostics`,
  `infra::load_mcp_config`, `infra::load_user_hooks`,
  `infra::fire_startup_hook`, `infra::create_runtime_state_store`, and
  `infra::create_conversation_store` were **not** audited for correctness —
  only their call sites and return types. A framework/infra follow-up task
  would cover those.
- `TaskRuntimeStore::recover_incomplete` (`store.rs:1631`) semantics were not
  audited for correctness; this task only confirms it is called twice in
  headless. Deeper recovery-correctness review belongs to A-TASK-*.
- The throwaway `AppState` built inside `start_headless_services` constructs
  more than just the TaskRuntimeStore (e.g. `SessionSearchEngine::reindex_all`
  at `state.rs:500-505`, `WorkspaceRegistry` at `state.rs:577-590`,
  `ToolExecutionRepository::open` at `state.rs:507-534`). These side effects
  also run twice-and-discard in TUI/CLI. Not elevated to a separate finding
  because the primary one (P2-01) covers the root cause; a fix that unifies
  construction will eliminate these too.
- No executable boot test was run in this review (read-only). V-series reports
  are static-inspection validations against `echo-agent-cli` commit `b3b2e81`.

## Handoff

Conclusions downstream tasks may rely on:

- `AgentRuntime::bootstrap` (`runtime.rs:73`) **is** the single composition root
  for the agent stack (agent, HITL, skills, hooks, review, LSP, plugins,
  browser, intent router). Downstream tasks auditing any of those services can
  treat `bootstrap` as the authoritative wiring point.
- The `TaskRuntimeStore`, `SchedulerRunner`, and `BackgroundTaskService` are
  **not** wired by `bootstrap`; they are wired per-entry. Downstream A-TASK /
  A-SCHED tasks must account for the three construction sites identified here.
- Multi-mode parity gaps (P2-02, P2-03) are boot-lifecycle issues, not per-tool
  issues. A fix belongs in the shared headless starter, not in individual tools.

Reports downstream tasks must read:

- `zcode-glm/tasks/B-PATH-01.md` (entry inventory; the three parity findings
  this task sharpens).

Conditions that make this report stale:

- Any change to `AgentRuntime::bootstrap`, `build_task_runtime_store_for_headless`,
  `AppState::from_shared`'s `TaskState::runtime` initializer, or
  `start_headless_services`.
- Any change that adds/removes `spawn_mcp_health_check` /
  `spawn_dreaming_task` from an entry.

Follow-up task IDs (not implemented in this review):

- A-TASK-*: own `TaskRuntimeStore` recovery and the single-construction fix.
- A-SCHED-* (if chartered): own `SchedulerRunner` / `BackgroundTaskService`
  parity in channels mode.
- A-PARITY-* (if chartered): own MCP-health / dreaming parity across entries.
