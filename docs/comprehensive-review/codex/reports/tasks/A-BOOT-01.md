# A-BOOT-01: Application composition and startup lifecycle

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean before report creation; only
> Codex review reports were added

## Question

Does each EKO entry point construct the same core services exactly once with
consistent config, working directory, startup rollback, reload, and orderly
shutdown behavior?

## Scope

- Both executable GUI entries and the shared desktop composition root.
- Default TUI, hidden CLI, pure-channel, and combined CLI-plus-channel branches.
- `AgentRuntime`, `AppState`, TaskRuntime store, pool, scheduler, background
  service, browser runtime, config watcher, and channel-manager construction.
- `--project` versus `working_dir` propagation into instructions, tools,
  memory, worktrees, CLI coding context, and TUI file projection.
- Startup failure propagation/rollback and normal cleanup ownership.

## Out Of Scope

- Pure-channel absence of cron/background/Dreaming, already owned by
  `B-PATH-01-P1-01` and followed by `A-SRF-04`.
- Scheduler/background cancellation tokens and join ownership, already owned
  by `B-PATH-01-P2-03`.
- Full provider/config/workspace transaction semantics (`A-CFG-01`).
- Per-surface command/event parity (`A-SRF-01` through `A-SRF-04`, then
  `X-SRF-01`).
- TaskRuntime execution correctness and file-store concurrency beyond the
  constructor evidence required here.
- Implementation of any fix or mutation of source code.

## Inputs

- Root `AGENTS.md` and its EKO local-product, mode-parity, layering, and
  terminology constraints.
- Shared `docs/comprehensive-review/README.md`, `REPORTING.md`, and the
  `A-BOOT-01` card in `TASKS.md`.
- Codex isolation rules in `docs/comprehensive-review/codex/README.md`.
- Dependency report [`B-PATH-01`](./B-PATH-01.md) and accepted validation
  [`V14`](../validations/B-PATH-01/V14-01.md).
- No report from another reviewer directory was read.

## Layering Decision

Application startup ordering, entry-option normalization, service identity,
and process shutdown are EKO product policy and belong in `echo-agent-cli`.
The framework already exposes the generic `Agent::close` /
`ReactAgent::shutdown` mechanism and MCP transport close operations; no new
framework lifecycle authority is justified. The EKO adapter should resolve one
options value, construct one retained lifecycle owner, pass shared framework
handles into it, and invoke one idempotent async shutdown.

Duplicate search covered `AgentRuntime::bootstrap`, `AppState::from_shared`,
`TaskRuntimeStore::new`, `AgentPool::from_runtime`, `init_pool`,
`start_headless_services`, watcher construction, all `shutdown`/`close` calls,
and all binary targets across both repositories. No second existing EKO
application-lifecycle owner was found. When centralizing, delete branch-local
browser/hook/config cleanup and the temporary headless `AppState` factory; do
not retain two authorities.

## Current Path

### GUI

`src-tauri/src/main.rs` and GUI-only `src/main.rs` both call
`run_desktop_entry` (`src-tauri/src/main.rs:3-5`, `src/main.rs:72-76`). It
creates diagnostics, calls `run_desktop`, but converts every returned error to
success (`src/tauri/desktop.rs:94-121`). `run_desktop` synthesizes default Args,
loads config, bootstraps one `AgentRuntime`, spawns the watcher, constructs and
retains one `AppState`, registers TaskRuntime tools, creates one pool, starts
task and scheduler services, then runs Tauri (`desktop.rs:124-258`). Exit
cancels only the common watcher/Dreaming/MCP-health token, drains task hook
events, and shuts down BrowserRuntime (`:260-268`).

### Headless

`run_tui_or_cli_entry` parses real Args, loads config, creates one
`AgentRuntime`, creates/recovers one TaskRuntime store, registers its tools,
creates one pool, and spawns its cleanup monitor (`src/main.rs:95-200`). TUI and
CLI then call `start_headless_services`; this constructs a second full
`AppState`, including another TaskRuntime store and recovery pass, overwrites
that store with the first handle, starts two services, returns only their Arcs,
and drops the temporary state (`src/cli/modes.rs:32-64`,
`app-core/src/state.rs:452-596`). Normal cleanup drains hook events and stops
BrowserRuntime but does not close primary/pool agents (`src/main.rs:329-338,
:394-401, :437-445`). Combined channel-plus-CLI additionally detaches the
channel handle without reaching its `ChannelManager::stop_all` block
(`src/main.rs:365-405`, `src/cli/modes.rs:216-234`).

### Shared Bootstrap And Reload

`AgentRuntime::bootstrap` starts default-enabled BrowserRuntime and its sidecar
prewarm before the sole fallible agent constructor (`app-core/src/runtime.rs:
73-105`); there is no rollback owner. The config watcher is otherwise composed
consistently in GUI and headless. Its intentionally limited live domains are
hooks and webhooks, and pooled agents share the same hook-registry Arc
(`config_watcher.rs:249-277`, `agent_pool.rs:120-159,882-887`). The targeted
watcher tests passed.

## Findings

### A-BOOT-01-P1-01: `--project` selects instructions but not the agent execution root

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:153`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/infra.rs:233`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/infra.rs:334`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/infra.rs:341`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/mod.rs:1966`
- Reachability: real TUI/CLI/channel Args feed `project` into every primary agent
  build, while `working_dir` remains `None`; CLI and TUI consume their own
  independently resolved project roots.
- Expected invariant: one explicit project selection controls instructions,
  Subagent/worktree discovery, tools, memory/artifacts, coding commands, and
  file projection.
- Observed behavior: `project` controls Subagent definitions and CLI context,
  but only `working_dir` reaches tool contexts and project-scoped memory. TUI
  file collection hard-codes `.`.
- Impact: launching from directory A with `--project B` can show B's
  instructions while shell/file/git tools and memory operate in A; TUI file
  completion also exposes A. This can modify or remember against the wrong
  project despite an explicit user selection.
- Root cause: `AgentCreateParams` models `project` and `working_dir` as
  independent roots and the headless composition root normalizes neither.
- Direction: resolve one canonical headless project/workspace root before
  bootstrap and pass it to all consumers. Delete later CLI/TUI `.`-based
  re-resolution after migration.
- Regression validation: launch from temp A with `--project` temp B; assert
  prompt/instruction, agent working directory, memory path, worktree factory,
  CLI coding root, and TUI file list all select B.
- Validation reports: [V02](../validations/A-BOOT-01/V02-01.md)

### A-BOOT-01-P1-02: Desktop startup failures exit successfully

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src-tauri/src/main.rs:3`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:72`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/desktop.rs:94`
- Reachability: both GUI executable routes return `run_desktop_entry`; every
  `run_desktop` error enters its logging branch and reaches unconditional
  `Ok(())`.
- Expected invariant: diagnostics may be written, but startup failure must
  propagate so the binary returns a non-zero exit status.
- Observed behavior: all desktop startup and Tauri runtime errors are swallowed
  after logging/dialog display.
- Impact: launchers, package smoke tests, scripts, and supervisors see success
  when no usable GUI was started, preventing reliable restart/failure handling.
- Root cause: diagnostic reporting and error handling are conflated in the
  outer entry adapter.
- Direction: log/display the failure, then return the original `Err` with its
  context. Do not add a parallel exit-status channel.
- Regression validation: inject a deterministic pre-Tauri failure into each
  GUI binary and assert non-zero status plus a diagnostic record.
- Validation reports: [V03](../validations/A-BOOT-01/V03-01.md)

### A-BOOT-01-P1-03: Browser sidecar prewarm has no bootstrap rollback owner

- Priority: P1
- Confidence: medium
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/runtime.rs:93`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/browser/config.rs:106`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/browser/mod.rs:66`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-integration/src/mcp/transport/stdio.rs:270`
- Reachability: every normal EKO bootstrap passes no BrowserRuntime, so default
  enabled startup spawns prewarm before the fallible agent build.
- Expected invariant: any process resource created before bootstrap commit is
  owned by a rollback guard and synchronously closed if a later step fails.
- Observed behavior: `create_agent_with_diagnostics(...)?` can return after
  prewarm begins, with no call to `BrowserRuntime::shutdown`. Transport Drop
  spawns another async cleanup task rather than awaiting it.
- Impact: a failed boot can leave Playwright MCP child cleanup timing-dependent,
  particularly when the outer Tokio runtime exits immediately. Repeated failed
  starts can accumulate orphan processes or locked browser profiles.
- Root cause: browser startup is eager and detached, but bootstrap has no
  transaction/commit boundary.
- Direction: make a bootstrap guard own pre-started resources and await rollback
  on failure, or start browser prewarm only after all fallible construction has
  committed. Delete detached cleanup as the normal failure protocol; retain it
  only as last-resort Drop behavior.
- Regression validation: use a fake MCP child and forced post-prewarm agent
  construction failure; assert child exit and profile release before bootstrap
  returns `Err`.
- Validation reports: [V03](../validations/A-BOOT-01/V03-01.md)

### A-BOOT-01-P1-04: Implemented primary-agent and pool shutdown is unreachable from EKO exits

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/react/mod.rs:1998`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/react/mod.rs:2616`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:744`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:437`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/desktop.rs:260`
- Reachability: all GUI/TUI/CLI/channel modes construct a primary agent and
  pool/cleanup monitor, but production search finds no call to pool shutdown,
  `Agent::close`, or `ReactAgent::shutdown`.
- Expected invariant: normal process exit stops request producers, cancels and
  drains the pool, closes the primary agent/MCP clients, and only then returns
  from the Tokio runtime.
- Observed behavior: EKO closes only its separate BrowserRuntime. Framework
  ReactAgent Drop spawns detached MCP cleanup, which can be aborted by immediate
  runtime teardown; pool cleanup has an explicit unused cancellation API.
- Impact: core MCP child/process cleanup is nondeterministic, pool background
  cleanup cannot be observed to terminate, and the shutdown-specific
  `ReactAgent::shutdown` hook path is skipped.
- Root cause: EKO has no retained application lifecycle owner that composes
  already-existing resource shutdown APIs.
- Direction: add one EKO lifecycle owner with ordered, idempotent async shutdown
  and use it in every root. Reuse framework `Agent::close`; do not add another
  framework shutdown mechanism. Delete branch-local teardown after cutover.
- Regression validation: instrument pool monitor and MCP transports, exit each
  mode, and assert all close acknowledgements occur before the entry future
  resolves.
- Validation reports: [V04](../validations/A-BOOT-01/V04-01.md),
  [V07](../validations/A-BOOT-01/V07-01.md)

### A-BOOT-01-P2-05: Combined channel-plus-CLI exit bypasses `ChannelManager::stop_all`

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:365`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:373`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/modes.rs:216`
- Reachability: `--channels --cli` spawns a channel task, runs REPL, then drops
  the JoinHandle when REPL returns; this is a real parsed mode combination.
- Expected invariant: normal foreground exit signals, awaits, and confirms
  orderly shutdown of every co-hosted channel transport.
- Observed behavior: `ChannelManager::stop_all` is only after the channel task's
  independent Ctrl-C wait. The combined branch neither signals nor awaits it;
  runtime destruction aborts the task.
- Impact: channel transports and buffered protocol work are stopped abruptly on
  normal `/exit` or Ctrl-D, and cleanup failures are unobservable.
- Root cause: the task handle is treated as fire-and-forget rather than part of
  the mode's lifecycle ownership.
- Direction: use the common lifecycle owner's cancellation and join protocol;
  remove the comment claiming the background channel automatically ends once
  an explicit stop is wired.
- Regression validation: fake channel records `stop`; leave combined CLI
  normally and assert `stop_all` completes before process return.
- Validation reports: [V05](../validations/A-BOOT-01/V05-01.md)

### A-BOOT-01-P2-06: Headless service startup constructs and discards a second application state

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:35`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/modes.rs:32`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/state.rs:452`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/state.rs:544`
- Reachability: every TUI and CLI call to `start_headless_services` constructs
  this state after main already constructed the canonical TaskRuntime store.
- Expected invariant: process-level stores and startup recovery run exactly
  once; service factories do not construct unrelated disposable subsystems.
- Observed behavior: `AppState::from_shared` builds another TaskRuntime store,
  performs recovery, creates persistence/search/tool execution/workspace/skills
  state and another webhook emitter; the adapter overwrites selected fields and
  immediately drops the state after cloning service Arcs.
- Impact: every headless startup repeats file scans/recovery and unrelated I/O;
  constructors that later gain side effects will silently run twice, while
  ownership is obscured by a discarded state container. The two file stores
  also have independent in-process lock/cache identities, although current
  sequencing did not establish concurrent corruption.
- Root cause: a mutable GUI state aggregate is reused as a headless service
  factory instead of composing shared services from explicit dependencies.
- Direction: make one retained EKO lifecycle/state composition or extract
  side-effect-free service constructors. Delete
  `build_task_runtime_store_for_headless` or `AppState`'s implicit store
  construction so exactly one path owns creation and recovery.
- Regression validation: constructor counters and a recovery fixture assert one
  TaskRuntime identity/recovery per GUI, TUI, CLI, and channel service boot.
- Validation reports: [V01](../validations/A-BOOT-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Constructor and service ownership map | yes | failed | [V01](../validations/A-BOOT-01/V01-01.md) |
| V02 | Entry option and working-directory diff | yes | failed | [V02](../validations/A-BOOT-01/V02-01.md) |
| V03 | Startup error propagation and rollback | yes | failed | [V03](../validations/A-BOOT-01/V03-01.md) |
| V04 | Primary-agent/pool shutdown trace | yes | failed | [V04](../validations/A-BOOT-01/V04-01.md) |
| V05 | Combined channel shutdown trace | yes | failed | [V05](../validations/A-BOOT-01/V05-01.md) |
| V06 | Config watcher targeted tests and shared reload | yes | passed | [V06](../validations/A-BOOT-01/V06-01.md) |
| V07 | B-PATH deduplication and historical classification | yes | passed | [V07](../validations/A-BOOT-01/V07-01.md) |
| V08 | Report IDs, links, executor, and path isolation | yes | passed after failed attempt | [V08 attempt 01](../validations/A-BOOT-01/V08-01.md), [V08 attempt 02](../validations/A-BOOT-01/V08-02.md) |
| V09 | Primary source/reachability/finding acceptance | yes | passed | [V09](../validations/A-BOOT-01/V09-01.md) |
| V10 | Primary rerun of config-watcher targeted tests | yes | passed | [V10](../validations/A-BOOT-01/V10-01.md) |
| V11 | Final report links/executor/source-isolation gate | yes | passed after failed attempt | [V11 attempt 01](../validations/A-BOOT-01/V11-01.md), [V11 attempt 02](../validations/A-BOOT-01/V11-02.md) |

Failed validations mean the asserted invariant is false and are fully captured
as findings; they do not mean the review execution itself failed.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| B-PATH entry graph and service mode matrix | current | [V07](../validations/A-BOOT-01/V07-01.md) |
| B-PATH scheduler/background missing shutdown owner | current, not duplicated | [V07](../validations/A-BOOT-01/V07-01.md) |
| `runtime.rs`: bootstrap is the single source of truth for agent initialization | current but narrower than process lifecycle | `runtime.rs:54-73`; [V01](../validations/A-BOOT-01/V01-01.md) |
| `main.rs`: headless builds one TaskRuntime store | regressed/misleading after service composition | [V01](../validations/A-BOOT-01/V01-01.md) |
| `config_watcher.rs`: only hooks/webhooks reload live | current | [V06](../validations/A-BOOT-01/V06-01.md) |
| Framework ReactAgent shutdown/Drop cleanup documentation | current API, but explicit path unused by EKO | [V04](../validations/A-BOOT-01/V04-01.md) |

## Coverage And Uncertainty

No real GUI window, QQ/Feishu account, or Playwright sidecar was launched. Those
would require user environment/credentials and are not needed to prove the
control-flow and ownership gaps. The browser rollback impact is medium
confidence because orphan duration depends on runtime/task timing; the absence
of an awaited rollback is certain. No failure-injection harness currently
exercises GUI exit status, post-prewarm rollback, pool/agent close ordering, or
combined-channel normal exit. Config watcher unit tests were the only targeted
executable validation because they are hermetic and directly applicable.

## Handoff

- `A-CFG-01` should consume P1-01 when defining one project/workspace root and
  restart-required configuration contract.
- `A-SRF-04` should retain ownership of pure-channel common-service parity and
  consume P2-05 only for combined channel lifecycle behavior.
- `A-INT-01` should use P1-03/P1-04 as application integration evidence;
  `F-INT-01` may independently verify the generic transport Drop/close
  contract. Neither should move EKO process lifecycle policy into the
  framework; generic MCP close already exists.
- `Q-E2E-01` should own mode-level startup-failure and orderly-exit scenarios
  after lifecycle centralization.
- This report becomes stale if binary entry targets, `src/main.rs`,
  `src/tauri/desktop.rs`, app-core `runtime.rs`/`state.rs`, TaskRuntime
  construction, AgentPool shutdown, BrowserRuntime startup, config watcher, or
  ChannelManager ownership changes.
