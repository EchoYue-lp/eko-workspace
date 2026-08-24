# A-SRF-04: CLI, channels, cron, and background triggers

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: `echo-agent` clean; `echo-agent-cli` Rust sources clean, 79
> pre-existing modified files under `web-frontend/src/generated/` (generated
> output outside the reviewed Rust slice, not touched by this review)
> Synthesis note: this task report was synthesized by a later session from the
> 10 completed immutable validation reports. The original task execution was
> interrupted by a network failure after all validations completed; no
> validation was rerun, and no new validation is missing (see Validation
> Matrix).

## Question

Do non-GUI/TUI triggers (CLI REPL, IM channels, cron, background) enter the
same core runtime and preserve identity, events, memory, tools, cancellation,
and terminal semantics?

**Answer: partially.** Entry and identity are preserved: all four trigger
families are registered at boot and reach the shared core runtime — the
interactive triggers (REPL, channel) enter the same `drive_chat` application
chat driver used by TUI/GUI, and the scheduled/background triggers enter the
TaskRuntime executor path (`launch_cron_run`/`drive_unattended_run` for cron,
`start_run_driver`→`execute_run` for background). Conversation identity
propagates per trigger family into `drive_chat`, the TaskRuntime store, and
the agent pool key. But cancellation and shutdown semantics are broken on the
CLI/channel surfaces (turns are not cancellable; no signal handler on the CLI
path; scheduler/background loops have no graceful stop), the channels-only
product entry omits the scheduler and the background task service entirely
(a surface-parity violation), and interrupted cron runs are unrecoverable from
the CLI/channel surfaces. Noninteractive event output exists through the
durable run store and webhooks (CronTaskCompleted), and the automatic cron
trigger itself never fires due to the framework tick predicate
(F-OPS-01-P1-01, re-anchored at current commits).

## Scope

Primary source paths and behaviors inspected (all `echo-agent-cli` unless
noted):

- Boot composition: `src/main.rs` (:52 boot recovery, :355-445 mode branches,
  channels-only :389-403, combined `--channels --cli` :373-404, shutdown
  sequence :431-445), `src/cli/modes.rs` (:62/83 `start_headless_services`,
  :109 `run_repl`, :118-235 `run_channels_mode`, :228 `shutdown_signal`),
  `echo-agent-app-core/src/state.rs` (:640-708 scheduler/background
  construction, :662/692/699 cancel-token clones).
- Trigger adapters: `src/cli/repl.rs` (:160-185 command registration, :234
  inline turn await, :477-560 `chat_with_agent`, :533 fresh cancel token,
  :563-838 render loop), `src/cli/channels.rs` (:195-270 turn construction,
  :244 fresh cancel token, :262 `drive_chat` spawn, :515-654
  `aggregate_by_sentence`), `src/cli/cmd_impls/cron.rs` (:45-111 CronCommand,
  :376-410 `cmd_cron_run` → `runner.run_once`), `src/cli/cmd_impls/advanced.rs`
  (:286-299 duplicate cron stub), `src/cli/command.rs` (:288-302 register
  overwrite).
- TaskRuntime trigger paths: `echo-agent-app-core/src/tasks/task_runtime/
  executor.rs` (:3571-3592 `launch_unattended_run` conversation_id + empty
  root_message_id, :3616 `drive_unattended_run`, :3649 `drive_agent_run`,
  :3895-3914 `launch_cron_run`), `echo-agent-app-core/src/tasks/service.rs`
  (:219/:253 submit paths, :270-272 `background:` conversation_id, :469-489
  resume ownership rejection, :556-558 `resume_pending` filter),
  `echo-agent-app-core/src/tasks/task_runtime/store.rs` (:1631-1776
  `recover_incomplete` Running→Paused).
- Shared driver integration: `echo-agent-app-core/src/chat_driver.rs` (:202
  `drive_chat`, :571-591 `ChannelChatSink`), `agent_pool.rs` (:819-850
  conversation_id as pool key), `echo-agent-app-core/src/scheduler/runner.rs`
  (:100-125 fire_fn → `launch_cron_run` + CronTaskCompleted webhook),
  `surface_contract.rs` (:50-56 capability-matrix cron row).
- Framework dependency re-anchor: `echo-agent/echo-orchestration/src/scheduler/
  runner.rs` (:80-93 tick predicate, :169 `run_once`).

## Out Of Scope

- Shared chat driver lifecycle and sink responsibilities → A-CHAT-01
  (dependency; its P1-01/P2-01/P2-02 are consumed as facts, not re-filed).
- TaskRuntime controller boundary, pause/cancel outcome handling, per-task
  cancel dead code → A-TSK-03 (dependency; P1-01/P2-01/P2-02 consumed).
- Framework scheduler tick correctness (automatic cron never fires) →
  F-OPS-01-P1-01 (canonical framework finding, re-anchored only).
- TUI/GUI internals, frontend reducers → A-SRF-01/02/03.
- Boot/service assembly ownership → A-BOOT-01 (P2-02/P3-03/P3-04
  cross-checked at current commits, not re-derived).
- Channel protocol adapters (QQ/Feishu transports) → F-INT-02.

## Inputs

- Root `AGENTS.md` (surface parity; "X mode doesn't use Y is a gap";
  Subagent-only terminology; UTF-8/panic safety; layering gate), shared
  `README.md`, `REPORTING.md`, `TASKS.md` (A-SRF-04 card), `zcode-ds/README.md`,
  report templates.
- Dependency task reports read: `A-CHAT-01` (complete; four `drive_chat` call
  sites, sink model, REPL/channel fresh tokens noted as A-SRF-04 cancel gap),
  `A-TSK-03` (complete; executor/store/run_driver boundary facts).
- The 10 A-SRF-04 validation reports (all read; each finding below links its
  source validation).
- Historical documents treated as hypotheses (classified in V05-01):
  `echo-agent-cli/docs/2026-07-17-surface-parity-closeout.md` (M10),
  `echo-agent-cli/docs/MASTER-PLAN.md`, root `docs/MASTER-PLAN.md`.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correctly placed) | The scheduler runner (`echo-orchestration/src/scheduler/runner.rs`), the envelope/terminal contract, `drive_agent_run` ReAct loop, `RuntimeDagExecutor` — all framework-owned, reused as-is. No movement recommended. |
| EKO product policy (application) | Boot composition (`main.rs`/`modes.rs`/`state.rs`), trigger adapters (REPL/channel/cron commands), `launch_cron_run`/`launch_unattended_run` run-identity policy, `resume_pending` ownership filter, background service, capability matrix — all application-owned. All findings in this report stay in the application layer (adapter boundary for the entry adapters). |
| Adapter boundary | The four trigger adapters are thin on the main path (build `PreparedUserTurn` + `ChatResources` and call `drive_chat`, or call the TaskRuntime launch helpers); two adapters hold a cancel token they never fire (P1-01) and one product entry omits the shared headless services (P1-02) — behavior defects inside the adapters, not second authorities. |
| Duplicate search | Terms (V01-01, both repos): `drive_chat`, `ChatSink`, `BackgroundTaskService::new/with_pool/with_agent_provider`, `start_scheduler_with_store`, `SchedulerRunner::new/new_scheduler_runner`, `launch_cron_run`, `bind_scheduler`, `worker`, `"cron"`/`CronCommand`. Results: one `drive_chat` definition with four live production call sites; three production sinks; one background-service construction path (`AppState::start_task_service`); one scheduler construction path (`start_scheduler_with_store`); single cron fire_fn caller (`build_fire_fn` → `launch_cron_run`); zero `worker` terms; **one duplicate** slash-command registration — `CronCommand` "cron" defined twice (P3-01). |
| Migration deletion | Deletion targets: the `advanced.rs` cron stub (P3-01); none elsewhere — fixes reuse existing service constructors and the already-registered cancel tokens. |

## Current Path

Verified call graph (V02-01, V01-01; anchors re-confirmed in this synthesis
session):

1. CLI REPL: `main.rs` → `run_cli_mode` (modes.rs:68) → `start_headless_services`
   (modes.rs:83; `start_task_service` state.rs:687 + `start_scheduler_with_store`
   state.rs:644) → `run_repl` (modes.rs:109) → `chat_with_agent` (repl.rs:234)
   → `PreparedUserTurn::build` + `ChatResources` with a fresh
   `CancellationToken` (repl.rs:509-540, token :533) → `drive_chat` spawned
   (repl.rs:541-545) → `ChannelChatSink` render loop (repl.rs:563-838).
2. Channels: `main.rs:365` `tokio::spawn(run_channels_mode)` (modes.rs:118)
   → `ChannelManager` + `AppChannelMessageHandler` (channels.rs:35) →
   per-sender pool agent (`pool.acquire`, channels.rs:135-139) →
   `PreparedUserTurn` + `ChatResources` with a fresh token (channels.rs:208-261,
   token :244) → `drive_chat` spawned (channels.rs:262) →
   `aggregate_by_sentence` outbound stream (channels.rs:515-654).
3. Cron: `start_scheduler_with_store` (state.rs:644) → `new_scheduler_runner`
   (state.rs:660; cancel token :662) → `SchedulerRunner::spawn` (state.rs:672)
   → framework tick (runner.rs:80-93) or `run_once` via `/cron run`
   (cron.rs:410) → `build_fire_fn` (runner.rs:100-125) → `launch_cron_run`
   (executor.rs:3895) → `launch_unattended_run` (:3571; conversation_id
   `"cron:{task}:{fire}"`, empty root_message_id :3587) → `drive_unattended_run`
   (:3616) → `drive_agent_run` (:3649) → ReAct loop; `CronTaskCompleted`
   webhook on success (runner.rs:111-122).
4. Background: `start_task_service` (state.rs:687) →
   `BackgroundTaskService::with_pool/new` (state.rs:692/:699; cancel token)
   → `spawn` (state.rs:708; `resume_pending` service.rs:556) and
   `submit/submit_run` (service.rs:219/:253; conversation_id
   `"background:{source}:{uuid}"` :270-272) → `start_run_driver`
   (service.rs:359-407) → `execute_run` (plan exists) or
   `drive_unattended_run` (no plan). CLI surface: `/tasks` commands
   (coding.rs:281).
5. Identity: conversation_id per trigger family (channel `channel:{cid}:{sid}`,
   cron `cron:{source}:{fire}`, background `background:{source}:{uuid}`, REPL
   boot uuid) reaches `drive_chat` (turn_id fresh uuid per turn), TaskRuntime
   run creation, and the pool key (`create_agent`, agent_pool.rs:819-850);
   task-mode formal run derived as `taskrun:<turn_id>`; cron/background runs
   carry empty root_message_id — turn_id None in `ExternalRunContext` for
   unattended runs (by design, V03-e).
6. Noninteractive event output: cron/background runs persist terminal facts in
   the durable run store + webhook (CronTaskCompleted); REPL renders to
   stdout (no single-shot/noninteractive mode — P3-02); channels project
   per-sentence OutboundMessages with terminal flush semantics (V04-05).

## Findings

### A-SRF-04-P1-01: REPL and IM-channel chat turns are not cancellable — fresh CancellationToken with zero producers, blocking input loop, and no signal handler on the CLI path

- Priority: P1
- Confidence: high
- Layer: application (adapter boundary, CLI/channels)
- Evidence:
  - `echo-agent-cli/src/cli/repl.rs:533` — `cancel: echo_agent::agent::CancellationToken::new()` in `ChatResources`; repository-wide grep finds zero `.cancel()` producers on this token (the only cancel calls in repl.rs, :262-263, belong to the unrelated `dreaming_cancel` memory-review token).
  - `echo-agent-cli/src/cli/repl.rs:234-237` — the input loop awaits `chat_with_agent(...)` inline; the whole loop is blocked for the turn's duration; no `select!`/abort path.
  - `echo-agent-cli/src/cli/channels.rs:244` — `let cancel = echo_agent::agent::CancellationToken::new();` inside the per-turn spawn; grep of `channels.rs` for `.cancel()` returns zero matches.
  - `echo-agent-cli/src/cli/modes.rs:228` — `crate::infra::shutdown_signal().await` is the **only** `tokio::signal::ctrl_c` call site in `src/` (verified by grep); it exists only in `run_channels_mode`. The CLI mode installs no signal handler, so mid-turn Ctrl+C takes the default SIGINT disposition and exits the process, skipping `store.shutdown_hook_events()` (main.rs:431-435) and graceful store shutdown.
  - Contrast (surfaces that can cancel): TUI `src/tui/events.rs:1937-1958` (`active_cancel`/`cancel()`), GUI `src/tauri/commands/chat.rs:807-826` (`cancel_chat` fires the turn token).
- Reachability: every REPL chat turn (repl.rs:236) and every channel turn (channels.rs:244-262); a turn lasts as long as the agent stream runs (seconds to minutes), and there is no user-facing way to stop it.
- Expected invariant: turns on every surface are cancellable and shutdown is graceful — the shared driver exposes cancellation through `ChatResources.cancel`, and AGENTS.md surface parity requires every surface to expose the same control semantics; the task card requires cancel/shutdown fixtures for all non-GUI/TUI triggers.
- Observed behavior: the REPL/channel tokens are constructed and never fired; a long turn cannot be interrupted; in CLI mode a mid-turn Ctrl+C kills the process without running the hook shutdown or flushing the store; in channels mode Ctrl+C stops the channel manager but the in-flight `drive_chat` spawn is left to run detached until process exit.
- Impact: the two surfaces with the least supervision are the only ones that cannot cancel an in-flight turn — user must kill the process, losing in-flight state and skipping shutdown hooks (P0-adjacent but not data loss by itself); contradicts the TUI/GUI parity baseline and A-CHAT-01's driver contract (the driver already registers the token; the adapters just never fire it).
- Root cause: the adapter layer constructs the token to satisfy `ChatResources` but never retains a firing handle; the CLI mode predates any signal-handler plumbing (only the channels mode got one, for `manager.stop_all`); the REPL loop awaits the turn inline instead of racing it against input.
- Direction: (a) retain the token in `chat_with_agent` and `run_channels_mode`'s handler; (b) install a `tokio::signal::ctrl_c`/SIGTERM handler in the CLI mode (mirroring `infra::shutdown_signal`, or reuse it) that fires the turn token; (c) race the turn await against the signal/input (select) so Ctrl+C during a turn yields a Cancelled terminal and the normal shutdown sequence (main.rs:431-445) runs; (d) in the channel handler, fire the per-turn token when the channel manager stops. No deletion targets; the driver contract already supports it.
- Regression validation: (a) driver-level fixture — a slow stream through `chat_with_agent` with the token fired terminates the turn with a Cancelled/error terminal; (b) CLI-mode fixture — Ctrl+C mid-turn exits gracefully with `shutdown_hook_events` observed; (c) channel fixture — `stop_all` cancels the in-flight turn. Candidates for Q-FLT-01/Q-E2E-01.
- Validation reports: [V03-01](../validations/A-SRF-04/V03-01.md) (failed; item a), [V01-01](../validations/A-SRF-04/V01-01.md)

### A-SRF-04-P1-02: Channels-only mode omits the scheduler and the background task service — cron never exists and background runs never resume on that surface

- Priority: P1
- Confidence: high
- Layer: application (boot composition)
- Evidence:
  - `echo-agent-cli/src/main.rs:365` — channels-only boot spawns `cli::run_channels_mode(...)`; the `else` branch (:389-403) awaits it and returns. `run_channels_mode` (`src/cli/modes.rs:118-235`) builds the `ChannelManager`, starts channels, awaits `shutdown_signal`, stops channels — it never calls `start_task_service` or `start_scheduler_with_store` (contrast: `start_headless_services`, modes.rs:62, calls both at :59-60 and is invoked by the CLI :83 and GUI `desktop.rs:232` paths).
  - `echo-agent-cli/src/main.rs:373-404` — combined `--channels --cli`: the `channels_handle` JoinHandle is dropped without await when `run_cli` is set (the await at :393 is inside the `else`), so the channels task runs detached/unmanaged and ends with process exit (A-BOOT-01-P3-04 cross-check at current commits).
  - `echo-agent-cli/echo-agent-app-core/src/state.rs:640-708` — the two services and their spawns (`start_scheduler_with_store` :644/:672, `start_task_service` :687/:708) exist and are reusable; the channels path just never calls them.
  - V04-06 proves the channels-only binary entry compiles and is reachable in a real build (not a compile-time artifact).
- Reachability: `--channels` without `--cli`/TUI/GUI (main.rs:389-403); background runs submitted by a channel agent (`create_complex_task` → background run) stay Paused after a restart because `resume_pending` is only ever invoked from `BackgroundTaskService::spawn` (service.rs:556), and no service exists here; cron entries never fire because no scheduler runner exists; plugin `bind_scheduler` (present on CLI modes.rs:93, TUI main.rs:270, GUI desktop.rs:236) is absent here too (V01-01).
- Expected invariant: MASTER-PLAN:18 — TUI/GUI/CLI/channels share the same Agent capabilities and differ only in input, rendering, and event projection; M10 closeout:14 — foreground/background/cron available on all surfaces (AGENTS.md: "X mode doesn't use Y" is a gap, not a positioning).
- Observed behavior: the channels-only product entry is missing both services; V05-01 classifies the parity claim as **regressed** on this surface; a channels-only deployment can neither schedule cron nor recover background runs.
- Impact: a whole capability class (cron + background + recovery) is silently absent from the channels product entry — the most visible surface-parity violation in this task, and it compounds P2-01 (no resume surface exists at all in channels-only mode).
- Root cause: `run_channels_mode` predates the shared `start_headless_services` composition and was never extended to call it; the `main.rs` boot branch kept a special-case path instead of unifying with the CLI/TUI assembly.
- Direction: call the shared headless-service assembly (`start_headless_services` or the state.rs:640-708 constructors) inside `run_channels_mode` before starting channels; unify the channels-only branch with the CLI branch so the handle is awaited/selected properly (also fixes the combined-mode detach); delete the special-case branch structure once unified. No new service authority — reuse the existing constructors.
- Regression validation: (a) channels-only boot fixture asserting scheduler runner + background service are constructed and spawned; (b) combined `--channels --cli` fixture asserting both modes run and shutdown gracefully; (c) channel-agent background run survives restart (resume_pending resumes it). Q-E2E-01 candidate.
- Validation reports: [V02-01](../validations/A-SRF-04/V02-01.md), [V03-01](../validations/A-SRF-04/V03-01.md) (failed; item d), [V04-06](../validations/A-SRF-04/V04-06.md), [V05-01](../validations/A-SRF-04/V05-01.md)

### A-SRF-04-P2-01: Interrupted cron runs are unrecoverable from the CLI/channel surfaces — boot recovery turns them Paused but every CLI resume surface filters them out

- Priority: P2
- Confidence: high
- Layer: application (recovery policy)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1631-1776` — `recover_incomplete` transitions every Running run (including cron runs) to Paused at boot (:1653, note :1756 "recovered interrupted run -> Paused at boot"); its own comment (:1625-1626) says Paused so the "normal resume path" can re-read the plan.
  - Cron runs carry `conversation_id = format!("{source_kind}:{source_id}:{fire_id}")` → `"cron:{task}:{fire}"` (`executor.rs:3580-3582` via `launch_cron_run` executor.rs:3895 → `launch_unattended_run` :3571).
  - `echo-agent-cli/echo-agent-app-core/src/tasks/service.rs:556-558` — `resume_pending` filters `run.conversation_id.starts_with("background:")`; `service.rs:469-489` — `resume` rejects any run whose conversation_id does not start with `"background:"` ("task run is not owned by the background service: {id}").
  - TUI `src/tui/events.rs:4710` and GUI `src/tauri/commands/task_runtime.rs:220-248` resume through the store directly (no prefix filter) — so cron runs are resumable only from TUI/GUI.
- Reachability: any cron run interrupted by process death (or by the process-lifetime loops of P2-02) → Paused at next boot → `BackgroundTaskService::spawn`'s `resume_pending` never picks it up; CLI `/tasks resume` (service.rs:469) rejects it; in channels-only mode no resume surface exists at all (P1-02).
- Expected invariant: every interrupted run has a recovery surface on the surface that owns it (task card: cancel/shutdown and recovery fixtures); `recover_incomplete`'s documented contract — Paused is the resumable state the normal resume path consumes.
- Observed behavior: a Paused cron run after restart is a dead run on the CLI and channel surfaces; only TUI/GUI can resume it.
- Impact: a user running cron from the CLI who restarts the process loses access to every interrupted cron run's continuation — the data is intact but unreachable through the owning surface.
- Root cause: the recovery filter was keyed on the `"background:"` conversation-id prefix when only background runs existed; cron joined the same store later via `launch_unattended_run`, and the filter was never generalized to the run's trigger kind.
- Direction: generalize the ownership filter to the trigger families that share the store (e.g. accept `background:` and `cron:` prefixes, or key on an explicit run-kind field instead of the conversation-id prefix); add a CLI `/tasks resume` acceptance fixture for cron runs; keep TUI/GUI behavior unchanged. Delete the hardcoded prefix checks at service.rs:474 and :556 when replaced.
- Regression validation: (a) fixture — Paused cron run → `resume_pending` resumes it and the run completes; (b) CLI `resume` on a Paused cron run succeeds; (c) boot-recovery of an interrupted cron run produces a resumable run. Q-FLT-02 candidate.
- Validation reports: [V03-01](../validations/A-SRF-04/V03-01.md) (failed; item c), [V02-01](../validations/A-SRF-04/V02-01.md)

### A-SRF-04-P2-02: Scheduler and background-service cancel tokens are never cancelled — cron/background loops are process-lifetime with no graceful stop (cross-check of A-BOOT-01-P3-03 at current commits)

- Priority: P2
- Confidence: high
- Layer: application (shutdown plumbing)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/state.rs:378` (`scheduler.cancel_token`), `:384` (`tasks.cancel_token`); the only production uses are the clones into the service constructors — `:662` (`new_scheduler_runner(store, self.scheduler.cancel_token.clone(), ...)`), `:692`/`:699` (`BackgroundTaskService::with_pool/new(..., self.tasks.cancel_token.clone(), ...)`).
  - Repository-wide grep of `.cancel()` in `src/` + `echo-agent-app-core/src/` (this session, excluding tests) shows no call targets either token; the `cancel_token.cancel()` calls at `src/main.rs:336/400/445` and `src/tauri/desktop.rs:261` belong to the top-level runtime token, not the scheduler/tasks tokens.
- Reachability: every boot that calls `start_headless_services` (CLI modes.rs:62, TUI, GUI desktop.rs:232); the scheduler run loop and background service run until process exit.
- Expected invariant: shutdown stops scheduler/background loops (task card: cancel/shutdown fixtures); graceful stop lets in-flight runs settle and persist a recoverable state instead of being orphaned mid-write.
- Observed behavior: neither token is ever fired; there is no stop() path for either loop; process exit is the only stop — which is exactly what produces the Paused-at-boot cron/background orphans of P2-01.
- Impact: no graceful drain of in-flight cron/background work at shutdown; every kill during a run leaves a Paused-at-boot run that the CLI cannot resume (P2-01); no hook flush or cleanup on stop.
- Root cause: the tokens were wired into the service constructors but no owner retained a firing handle and no shutdown sequence calls them; the plumbing was never completed (A-BOOT-01-P3-03, re-confirmed at the current commits by this task's grep).
- Direction: fire `scheduler.cancel_token` and `tasks.cancel_token` from a shutdown path (e.g. the runtime shutdown sequence at main.rs:431-445 and the channels-mode shutdown at modes.rs:228-233), with a bounded drain so in-flight runs transition to Paused and hooks flush before process exit.
- Regression validation: fixture — start both services, fire the tokens, assert both loops exit and any in-flight run becomes Paused with hook events flushed; CLI-mode signal test asserting the same on Ctrl+C.
- Validation reports: [V03-01](../validations/A-SRF-04/V03-01.md) (failed; item b), [V01-01](../validations/A-SRF-04/V01-01.md)

### A-SRF-04-P3-01: Duplicate `cron` slash-command registration — the `advanced.rs` stub is shadowed in dispatch but still listed by `/help`

- Priority: P3
- Confidence: high
- Layer: application (CLI command registry)
- Evidence:
  - `echo-agent-cli/src/cli/cmd_impls/advanced.rs:286-299` — stub `cmd_cron` (prints "Use /cron add|remove|enable|disable|run") + `cmd!` registering `CronCommand` name "cron" (category Advanced).
  - `echo-agent-cli/src/cli/cmd_impls/cron.rs:45-111` — full `CronCommand` implementation (name "cron", aliases schedule/sched, subcommand dispatch; `cmd_cron_run` → `runner.run_once` at :410).
  - `echo-agent-cli/src/cli/repl.rs:168` (`advanced::register_all`) then `:179` (`cron::register_all`); `src/cli/command.rs:294-299` — `register` overwrites `by_name` (last wins = the full impl) but pushes into `commands`, and `/help` lists from `commands` via `by_category` (`src/cli/cmd_impls/all.rs:25-49`) — both entries are listed.
- Reachability: every REPL session; dispatch uses `by_name` so the full implementation wins; `/help` displays both the stub and the real command.
- Expected invariant: one definition per command name (V01 duplicate search; AGENTS.md no-parallel-authority for the same surface).
- Observed behavior: two `CronCommand` definitions under the same name coexist; `/help` shows two "cron" rows with different descriptions.
- Impact: confusing `/help` output; a future reorder of `register_all` calls silently swaps the working implementation back to the stub.
- Root cause: the stub predates the full cron implementation (Phase M10-era placeholder) and was never removed when the real command landed.
- Direction: delete the `advanced.rs` stub (`cmd_cron` + its `cmd!` block, :286-299) and its `register_all` entry; keep `cron.rs` as the single definition.
- Regression validation: grep `CronCommand` yields one production definition; `/help` shows one cron row; `cargo test -p echo-agent-cli` (repl/config tests) stays green.
- Validation reports: [V01-01](../validations/A-SRF-04/V01-01.md)

### A-SRF-04-P3-02: REPL is interactive-only — no single-shot/noninteractive mode; noninteractive event output exists only via the durable store and webhooks

- Priority: P3
- Confidence: high
- Layer: application (adapter, CLI surface)
- Evidence:
  - V03-01 item (f): no `--exec`/`--print` single-shot flag exists in the CLI args surface; `chat_with_agent` (repl.rs:477-560) is an interactive streaming render loop (:563-838).
  - Noninteractive consumers that do exist: durable run store + `CronTaskCompleted` webhook (`echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs:111-122`), `drive_chat` webhook observers (A-CHAT-01 facts).
- Reachability: a user scripting EKO from the shell must drive the REPL; there is no way to run one turn and exit with a typed terminal/exit code.
- Expected invariant: noninteractive triggers have a noninteractive consumer for terminal/error events (task card: noninteractive event output).
- Observed behavior: CLI turns are only rendered interactively; cron/background do persist terminal facts and emit webhooks.
- Impact: minor — scripting and CI integration need the webhook/store path instead of the CLI; the task card's noninteractive-event-output validation is only partially satisfied.
- Root cause: the single-shot mode was never added; the surface stayed REPL-only while the task card's parity requirement includes noninteractive output.
- Direction: add a single-shot mode (e.g. `--exec "<prompt>"`/`--print`) that reuses `drive_chat` with a stdout sink (same driver, new thin adapter), exiting with the turn outcome; or explicitly document webhooks as the noninteractive output contract.
- Regression validation: single-shot fixture asserting the terminal event is printed and the process exit code reflects the outcome (failed vs completed).
- Validation reports: [V03-01](../validations/A-SRF-04/V03-01.md) (failed; item f)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (drive_chat, ChatSink impls, BackgroundTaskService construction, scheduler construction, launch_cron_run callers, worker terms, duplicate cron command registration, framework tick re-anchor) | yes | passed | [V01-01](../validations/A-SRF-04/V01-01.md) |
| V02 | Registration and trigger-adapter matrix (REPL/channel → drive_chat; cron → launch_cron_run → drive_agent_run; background → start_run_driver → execute_run; channels-only boot gap; combined-mode handle drop) | yes | passed | [V02-01](../validations/A-SRF-04/V02-01.md) |
| V03 | Invariant/edge cases (identity propagation; noninteractive event output; cancel/shutdown; recovery surfaces per trigger) | yes | failed (5 observed violations → P1-01, P1-02, P2-01, P2-02, P3-02) | [V03-01](../validations/A-SRF-04/V03-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked scheduler::runner` | yes | passed (exit 0; 3 ok) | [V04-01](../validations/A-SRF-04/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked tasks::service` | yes | passed (exit 0; 4 ok) | [V04-02](../validations/A-SRF-04/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked tasks::background` | yes | passed (exit 0; 1 ok) | [V04-03](../validations/A-SRF-04/V04-03.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked surface_contract` | yes | passed (exit 0; 3 ok) | [V04-04](../validations/A-SRF-04/V04-04.md) |
| V04 | `cargo test -p echo-agent-cli --no-default-features --features channels --lib --locked` (channels feature; 27 ok incl. aggregator terminal/UTF-8 tests) | yes | passed (exit 0; 27 ok) | [V04-05](../validations/A-SRF-04/V04-05.md) |
| V04 | `cargo check --no-default-features --features channels --bin echo-agent-cli --locked` (channels-only entry compiles) | yes | passed (exit 0) | [V04-06](../validations/A-SRF-04/V04-06.md) |
| V05 | Historical-document drift (M10 surface-parity closeout; MASTER-PLAN:18/333/69; cron webhook claim) | conditional | passed | [V05-01](../validations/A-SRF-04/V05-01.md) |

All required validations have immutable reports; every reported command has a
known exit code; V03's failure is recorded as findings above. No validation
was rerun in this synthesis session; every P0/P1 finding anchor was re-read in
source at the reviewed commits and confirmed.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| M10 closeout:70-71 "CLI `/remember`/`/forget` print success without touching the memory store; CLI interaction mode fixed to Auto" | fixed | `/remember` writes via layer_manager (all.rs:118-147), `/forget` deletes via layer_manager (all.rs:158-227), `/mode chat\|task\|auto` mutates the shared interaction-mode lock (all.rs:333-361) (V05-01) |
| M10 closeout:76 "TUI `/cron` sends prose to the agent instead of controlling the scheduler" | fixed | real TUI cron command (`src/tui/commands.rs:118,193,259,290`; `handle_tui_cron` events.rs:2232+) (V05-01) |
| M10 closeout:117 "TUI controls the real scheduler directly; cron failures persist a Failed run" | current | TUI uses the shared runner; failure persistence proven by V04-01 (fire_fn error path persists Failed) |
| M10 closeout:14 "Foreground, background, and cron runs" required on all surfaces | regressed (channels) | channels-only mode has no BackgroundTaskService and no SchedulerRunner (V02-01/V03-01 → P1-02) |
| `docs/MASTER-PLAN.md:18` "TUI, GUI, CLI, and channels share the same Agent capabilities and differ only in input, rendering, and event projection" | regressed (channels services) | scheduler + background service absent on channels-only surface (P1-02); otherwise current (shared drive_chat, V02-01) |
| `docs/MASTER-PLAN.md:333` "All six entry points switched to PreparedUserTurn (CLI REPL, channel)" | current | repl.rs:509, channels.rs:208 (V05-01) |
| `docs/MASTER-PLAN.md:69` iteration-2 webhook claim "cron emits CronTaskCompleted" | current but blocked | emitter wired (runner.rs:111-122) but automatic tick never fires (F-OPS-01-P1-01); manual `/cron run` emits (V05-01, V04-01) |
| A-BOOT-01-P2-02 (channels-only boot lacks shared services) / A-BOOT-01-P3-04 (combined-mode channels handle dropped) | current (cross-checked) | main.rs:365-404, modes.rs:118-235 (V02-01/V03-01) |
| A-BOOT-01-P3-03 (scheduler/background cancel tokens never fired) | current (cross-checked) | state.rs:662/692/699 + zero `.cancel()` producers (V03-01 → P2-02) |
| A-CHAT-01 P1-01 handoff: "REPL and channel surfaces create a CancellationToken that is never fired — not cancellable; owned by A-SRF-04" | current (this task confirms) | repl.rs:533, channels.rs:244 → P1-01 |

## Coverage And Uncertainty

- All conclusions are static except the V04 test runs (executed at the
  reviewed commits); no live LLM turn or IM traffic was executed (read-only
  review, no credentials).
- The "mid-turn Ctrl+C terminates the process" consequence (P1-01) follows
  from the verified absence of any signal handler on the CLI path plus default
  SIGINT disposition; it was not dynamically reproduced.
- F-OPS-01-P1-01 (automatic cron tick never fires: runner.rs:80-93 `next <=
  now` unsatisfiable) is consumed as a canonical framework finding and
  re-anchored at the current commits (V01-01/V04-01/V05-01); it is not re-filed
  as an A-SRF-04 finding. Its consequence — cron triggers exist only via
  manual `/cron run` on every surface — is a surface-parity fact this report
  relies on.
- GUI/TUI internals and frontend rendering are A-SRF-02/03 scope and were not
  re-inspected here; the TUI/GUI cancel contrast (events.rs:1937-1958,
  chat.rs:807-826) was read only at the cited anchors.
- The REPL/channel identity propagation (V03-e) was verified statically; the
  exact pool-key behavior under concurrent per-sender turns is A-SUB-01 scope.
- `surface_contract.rs` capability matrix is a static evidence table (V04-04);
  it certifies wiring, not runtime parity (V03 failed on that).

## Handoff

- Downstream tasks may rely on: one shared chat driver with four live call
  sites (REPL/channel/TUI/GUI) and one TaskRuntime path for cron/background
  (V01/V02); identity propagation is consistent per trigger family (V03-e);
  REPL/channel turns are not cancellable and the CLI path has no signal
  handler (P1-01); channels-only boot lacks scheduler + background service and
  combined mode detaches the channels task (P1-02, A-BOOT-01-P2-02/P3-04);
  interrupted cron runs are unresumable from CLI/channel surfaces (P2-01);
  scheduler/background cancel tokens are never fired (P2-02, A-BOOT-01-P3-03);
  duplicate `cron` command registration (P3-01); REPL is interactive-only
  (P3-02); cron webhook wired but automatic trigger blocked by F-OPS-01-P1-01.
- Reports to read: this report + the 10 validation reports; A-CHAT-01
  (driver/sink model, P1-01 handoff), A-TSK-03 (executor boundary, cancel
  semantics), A-BOOT-01 (boot assembly ownership), F-OPS-01-P1-01 (tick).
- Stale triggers: changes to `main.rs` mode branches, `modes.rs`
  (`run_channels_mode`/`start_headless_services`/`shutdown_signal`),
  `repl.rs` `chat_with_agent`/`run_repl`, `channels.rs` turn path, `state.rs`
  `start_task_service`/`start_scheduler_with_store`, `tasks/service.rs`
  `resume`/`resume_pending`, `store.rs` `recover_incomplete`, `executor.rs`
  `launch_cron_run`/`launch_unattended_run`, the CLI command registry, or the
  framework scheduler tick invalidate the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): X-SRF-01
  (parity rows: CLI/channel cancellation, cron/background on channels,
  noninteractive output), Q-E2E-01 (cancel a CLI/channel turn; channels-only
  boot with a cron entry; interrupted cron resume), Q-FLT-01/02 (signal
  mid-turn; Paused cron-run recovery; scheduler/background shutdown), X-EVT-01
  (cancel terminal reachability on the four trigger surfaces), S-RDM-01
  (P1-01 cancel wiring, P1-02 channels services, P2-01 recovery filter,
  P2-02 shutdown, P3-01 stub deletion).
