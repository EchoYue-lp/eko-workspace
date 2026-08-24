# B-PATH-01: EKO entry-point and composition inventory

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d3`
> `echo-agent-cli` commit: `b3b2e81f2b2d`
> Worktree state: both source repositories clean

## Question

Which startup constructors and live entry points assemble TUI, GUI, CLI,
channel, cron, and background capabilities, and are those services reachable
in every supported mode?

## Scope

- Binary declarations and features in `echo-agent-cli/Cargo.toml` and
  `tauri.conf.json`.
- `src/main.rs`, `src-tauri/src/main.rs`, `src/lib.rs`.
- Headless mode routing in `src/cli/modes.rs`, channel handler construction,
  CLI REPL service injection, and TUI entry wiring.
- GUI composition in `src/tauri/desktop.rs`, `src/tauri/mod.rs`, and Tauri
  managed state.
- Shared application constructors in app-core `runtime.rs` and `state.rs`.
- Scheduler/background/Dreaming/MCP-health registration and top-level
  cancellation paths.
- Build reachability of default TUI, channels-only, GUI targets, and combined
  TUI/channels configurations.

## Out Of Scope

- Constructor option consistency, startup rollback, and full shutdown design:
  `A-BOOT-01`.
- Config precedence and workspace switching: `A-CFG-01`.
- Chat behavior and event rendering beyond proving entry registration.
- Individual Tauri IPC command correctness.
- Framework scheduler/channel internals except the minimal adapter boundary.

## Inputs

- Root `AGENTS.md` read in full.
- Shared `README.md`, `REPORTING.md`, and the `B-PATH-01` task card.
- Codex track `README.md`.
- Codex dependency report `B-BASE-01`; relied only on its current target and
  feature inventory, and independently compiled every entry combination here.
- No other reviewer's report or historical audit report was read.

## Layering Decision

- Generic mechanism: `echo-agent` provides reusable channels and
  `SchedulerRunner` primitives. Their independent framework APIs are not judged
  by whether EKO uses them.
- EKO product policy: selecting TUI/GUI/CLI/channel modes; constructing
  `AgentRuntime`, AgentPool, TaskRuntime, BackgroundTaskService, cron,
  Dreaming, config watchers, and render bridges; and requiring mode parity all
  belong to `echo-agent-cli`.
- Adapter boundary: app-core `scheduler::build_fire_fn` converts a framework
  `CronTask` fire into EKO `launch_cron_run`. This is a thin callback adapter;
  the framework runner retains generic tick/store behavior.
- Duplicate search terms: `fn main`, `AgentRuntime::bootstrap`,
  `AppState::from_shared`, `run_desktop_entry`, `run_tui`, `run_cli_mode`,
  `run_channels_mode`, `start_headless_services`, `start_task_service`,
  `start_scheduler_with_store`, `bind_scheduler`, `spawn_dreaming_task`, and
  `spawn_mcp_health_check` across both repositories. No additional EKO binary
  or hidden composition root was found.

## Current Path

```text
echo-agent-cli binary (default feature: tui)
  main
    -> run_tui_or_cli_entry
       -> AgentRuntime::bootstrap (shared primary runtime)
       -> TaskRuntimeStore + task tool registration
       -> AgentPool + task_execute pool binding + cleanup monitor
       -> TUI
          -> start_headless_services -> temporary AppState
             -> BackgroundTaskService + SchedulerRunner
          -> plugin scheduler bind -> run_tui -> Dreaming -> drive_chat
       -> CLI
          -> run_cli_mode -> start_headless_services
          -> plugin scheduler bind -> run_repl -> Dreaming -> drive_chat
       -> channel
          -> run_channels_mode -> ChannelManager -> SessionHandler
          -> AppChannelMessageHandler -> pooled agent -> drive_chat

echo-agent-tauri binary (required feature: gui)
  main -> run_desktop_entry -> run_desktop
    -> AgentRuntime::bootstrap
    -> AppState + TaskRuntime tools + AgentPool
    -> BackgroundTaskService + SchedulerRunner + plugin scheduler bind
    -> MCP health + Dreaming
    -> build_tauri_app -> managed TauriState + IPC/event bridges

cron: SchedulerRunner -> app build_fire_fn -> launch_cron_run
background: BackgroundTaskService -> TaskRuntime unattended run
```

The package-name binary also redirects to `run_desktop_entry` when compiled
`gui && !tui` (`src/main.rs:75`). With both `tui` and `gui`, that binary takes
the TUI branch; the dedicated `echo-agent-tauri` target still supplies GUI.

### Mode-To-Service Matrix

| Service/capability | GUI | TUI | CLI | pure channel | CLI + channel |
|---|---:|---:|---:|---:|---:|
| `AgentRuntime::bootstrap` | yes | yes | yes | yes | yes |
| file TaskRuntime + task tools | yes | yes | yes | yes | yes |
| AgentPool + cleanup monitor | yes | yes | yes | yes | yes |
| `drive_chat` | yes | yes | yes | yes | yes |
| `BackgroundTaskService` retained | yes | yes | yes | **no** | yes (CLI-owned) |
| persisted `SchedulerRunner` retained | yes | yes | yes | **no** | yes (CLI-owned) |
| plugin scheduler monitors bound | yes | yes | yes | **no** | yes (CLI-owned) |
| Dreaming task | yes | yes | yes | **no** | yes (CLI-owned) |
| MCP health projection poller | yes | no | no | no | no |
| explicit scheduler/task-service cancellation | **no** | **no** | **no** | n/a | **no** |

MCP health is currently a GUI `AppState` projection used by Tauri IPC, so its
absence elsewhere is not independently reported as a parity defect.

## Findings

### B-PATH-01-P1-01: Pure channel mode omits the unattended service composition

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:357`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:365`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/modes.rs:32`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/modes.rs:83`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/modes.rs:118`
- Reachability: `--no-default-features --features channels` compiles (V04);
  `main -> run_tui_or_cli_entry -> run_channels_mode` bypasses the only
  headless call to `start_headless_services`. `run_channels_mode` accepts no
  scheduler, BackgroundTaskService, plugin runtime, or Dreaming lifecycle.
- Expected invariant: channel is a full long-lived Agent surface; cron and
  recoverable background work must remain active regardless of whether a CLI
  REPL happens to share the process.
- Observed behavior: pure channel retains TaskRuntime/AgentPool for per-message
  complex tasks, but does not start persisted cron, resume/serve
  `BackgroundTaskService` work, bind plugin monitor cron definitions, or start
  Dreaming. The same channel process gains those services only when `--cli` is
  also set, because CLI owns `start_headless_services`.
- Impact: enabled persisted cron tasks and plugin monitors silently never fire
  in the channels-only deployment; pending background runs are not resumed by
  BackgroundTaskService; memory Dreaming does not run. Behavior changes merely
  by adding an unrelated interactive REPL flag.
- Root cause: common service composition is nested in the TUI and CLI branches
  instead of being owned once by the headless process before mode branching.
- Direction: create one retained application-layer headless lifecycle owner
  after runtime/pool construction, start common services once, pass the handles
  into every selected surface, and remove branch-local duplicate startup. Do
  not add a second scheduler or move EKO mode policy into the framework.
- Regression validation: constructor-level matrix for TUI, CLI, channel, and
  CLI+channel asserting exactly one BackgroundTaskService, SchedulerRunner,
  plugin scheduler binding, and appropriate Dreaming lifecycle; integration
  fixture proving a persisted cron fires in pure channels mode.
- Validation reports: [V02](../validations/B-PATH-01/V02-01.md),
  [V04](../validations/B-PATH-01/V04-01.md),
  [V09](../validations/B-PATH-01/V09-01.md)

### B-PATH-01-P2-02: The surface parity test cannot detect false composition claims

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/surface_contract.rs:28`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/surface_contract.rs:141`
- Reachability: app-core includes the module under `#[cfg(test)]` at
  `echo-agent-app-core/src/lib.rs:41`; the exact test executes and passes (V10).
- Expected invariant: a test named `capability_matrix_has_evidence_for_every_surface`
  should fail when a declared surface lacks the startup wiring behind a
  capability claim.
- Observed behavior: it asserts only row/column counts and non-empty string
  literals. For channel `foreground_background_cron`, the literal
  `"shared background tools"` remains accepted even though V09 proves no
  scheduler or BackgroundTaskService starts in pure channel mode.
- Impact: composition regressions can merge while the advertised parity
  contract stays green, giving reviewers misleading confidence.
- Root cause: intended evidence is encoded as prose rather than executable
  constructor state or behavior.
- Direction: retain prose only as documentation; replace/supplement this test
  with mode-constructor contracts over actual service handles and one small
  persisted-cron/background recovery fixture.
- Regression validation: negative control that removes channel common-service
  startup and proves the new contract fails.
- Validation reports: [V09](../validations/B-PATH-01/V09-01.md),
  [V10](../validations/B-PATH-01/V10-01.md)

### B-PATH-01-P2-03: Scheduler and background services have no orderly shutdown owner

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/state.rs:540`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/state.rs:662`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/state.rs:692`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/modes.rs:57`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:329`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/desktop.rs:260`
- Reachability: TUI/CLI/GUI call the constructors that clone independent
  scheduler/task tokens into spawned tasks. Shutdown traces call
  `shutdown_hook_events`, browser shutdown, and a different config/Dreaming/MCP
  token, but never the two service tokens.
- Expected invariant: composition roots that start long-lived services retain
  their lifecycle owner and explicitly stop/join those services during orderly
  shutdown.
- Observed behavior: GUI retains `AppState` but never cancels its two service
  tokens. Headless `start_headless_services` returns only service Arcs and drops
  the temporary `AppState` that owns the tokens; there is no `Drop` cancellation.
- Impact: process exit relies on Tokio runtime destruction rather than orderly
  quiescence. Embedded/test shutdown cannot assert completion; in-flight cron
  or background persistence may be interrupted without a deterministic service
  stop point.
- Root cause: startup exposes service handles while lifecycle control remains
  split across temporary state and unrelated cancellation tokens.
- Direction: make the same application lifecycle owner retain service tokens
  and task handles; provide idempotent async shutdown that cancels and joins
  before runtime/browser teardown. Delete branch-local teardown once centralized.
- Regression validation: start each mode with short-interval test services,
  invoke orderly shutdown, and assert both loops observe cancellation and no
  task remains active.
- Validation reports: [V02](../validations/B-PATH-01/V02-01.md),
  [V11](../validations/B-PATH-01/V11-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Entry-point/duplicate call graph | yes | passed | [V01](../validations/B-PATH-01/V01-02.md) |
| V02 | Composition-root inventory | yes | passed | [V02](../validations/B-PATH-01/V02-01.md) |
| V03 | Default TUI compile | yes | passed | [V03](../validations/B-PATH-01/V03-01.md) |
| V04 | Channels-only compile | yes | passed | [V04](../validations/B-PATH-01/V04-01.md) |
| V05 | Dedicated GUI compile | yes | passed | [V05](../validations/B-PATH-01/V05-01.md) |
| V06 | Package GUI redirect compile | yes | passed | [V06](../validations/B-PATH-01/V06-01.md) |
| V07 | Combined TUI/channels compile | yes | passed | [V07](../validations/B-PATH-01/V07-01.md) |
| V08 | No-surface compile rejection | yes | passed | [V08](../validations/B-PATH-01/V08-01.md) |
| V09 | Mode-to-service matrix | yes | failed | [V09](../validations/B-PATH-01/V09-01.md) |
| V10 | Existing surface-contract sensitivity | yes | failed | [V10](../validations/B-PATH-01/V10-01.md) |
| V11 | Service shutdown ownership | yes | failed | [V11](../validations/B-PATH-01/V11-01.md) |
| V12 | Limited historical drift check | yes | passed | [V12](../validations/B-PATH-01/V12-01.md) |
| V13 | Report task IDs, links, executor, and path isolation | yes | passed | [V13](../validations/B-PATH-01/V13-01.md) |
| V14 | Primary source/finding-boundary acceptance | yes | passed | [V14](../validations/B-PATH-01/V14-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AGENTS.md`: TUI/GUI/CLI/channel target capability parity | current invariant, currently violated for pure channel services | [V09](../validations/B-PATH-01/V09-01.md) |
| Historical TUI `task_runtime_store = None` gap | fixed | `echo-agent-cli/src/main.rs:172`; [V02](../validations/B-PATH-01/V02-01.md) |
| `state.rs`: state supports Web/CLI and dual Web+CLI modes | stale | `echo-agent-cli/src/main.rs:351`; [V12](../validations/B-PATH-01/V12-01.md) |
| Surface contract: channel has foreground/background/cron evidence | regressed/misleading | [V09](../validations/B-PATH-01/V09-01.md), [V10](../validations/B-PATH-01/V10-01.md) |

## Coverage And Uncertainty

No real QQ/Feishu credentials were used and no GUI window was launched; build
reachability plus source registration were the appropriate non-mutating checks
for this inventory task. Tauri IPC handler bodies were not individually
reviewed. Whether runtime destruction already gives sufficient practical
process-exit behavior is environment-dependent, but the absence of an orderly,
awaitable service stop path is certain. Broader constructor failure rollback
and config/reload parity remain for `A-BOOT-01`.

## Handoff

- `A-BOOT-01` may rely on this entry graph and mode matrix. It should review one
  common application lifecycle owner, constructor option differences, startup
  rollback, and async shutdown/join semantics.
- `B-DOC-01` should treat Web/dual-mode comments and hard-coded surface evidence
  as drift candidates.
- `A-SRF-04` is the direct consumer for pure-channel cron/background/Dreaming
  behavior after composition is centralized, without introducing a
  channel-specific executor.
- `X-SRF-01` should consume the corrected mode matrix when synthesizing surface
  parity; `Q-E2E-01` should own the real pure-channel persisted-cron/background
  recovery regression after the relevant implementation work lands.
- This report becomes stale if `Cargo.toml` features/targets, `src/main.rs`,
  `src/cli/modes.rs`, `src/tauri/desktop.rs`, app-core runtime/state service
  constructors, or the reviewed commits change.
