# B-PATH-01: EKO entry-point and composition inventory

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Which startup constructors and live entry points assemble TUI, GUI, CLI,
channel, cron, and background capabilities, and does the assembled service
set respect the surface-parity invariant?

## Scope

- `echo-agent-cli/src/main.rs`, `src-tauri/src/main.rs`,
  `src/tauri/desktop.rs`, `src/lib.rs`, `src/cli/` (modes, args, repl,
  channels), `src/tui/mod.rs` (entry portion), `src/tauri/mod.rs` (builder).
- app-core `runtime.rs`, `infra.rs` (entry-relevant constructors),
  `state.rs` (AppState), `scheduler/runner.rs`, `tasks/background.rs`.

## Out Of Scope

- Per-command correctness of the Tauri IPC surface (`A-SRF-02`).
- Full TUI widget/event-loop review (`A-SRF-01`).
- Runtime behavior of cron/tasks (`A-SRF-04`, `A-TSK-03`).
- MCP reconnect semantics inside the framework (`F-INT-01`).

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`
  (B-PATH-01 card), `zcode-ds/README.md`.
- Dependency report `B-BASE-01` (zcode-ds track).
- No historical audit conclusion accepted as evidence.

## Layering Decision

- Generic mechanism: none at this layer (all EKO product composition).
- EKO product policy: every entry point, service selection, feature split,
  and the parity invariant.
- Adapter boundary: `AgentRuntime::bootstrap` and `AppState::from_shared`
  are the application composition roots; `register_task_tools_on_agent` /
  `bind_task_execute_to_pool` are the thin adapters joining EKO's
  TaskRuntime to the framework agent.
- Duplicate search terms: `fn main`, `AgentRuntime::bootstrap`,
  `AppState::from_shared`, `start_headless_services`, `run_channels_mode`,
  `spawn_mcp_health_check`, `spawn_dreaming_task`,
  `start_task_service`, `start_scheduler_with_store`, `spawn_config_watcher`,
  `register_task_tools_on_agent`, `bind_task_execute_to_pool`,
  `recover_incomplete`, `cfg(feature = "tui"/"gui"/"channels")`.
  One bootstrap, one desktop runtime, one headless-services helper, one
  channels root; two TaskRuntimeStore construction sites and two
  task-tool registration pairs (finding P2-02).

## Current Path

`main.rs` dispatches by feature (TUI default, GUI-only → desktop, channels
fallback, `compile_error!` otherwise) and by flag inside
`run_tui_or_cli_entry` (TUI default, `--cli`, `--channels`, `--web` removed).
All modes bootstrap through `AgentRuntime::bootstrap` and then assemble
TaskRuntimeStore + task tools + AgentPool + config watcher + task service +
scheduler + Dreaming per entry (V01, V02). The GUI additionally starts the
only MCP health-monitoring loop (V04).

## Findings

### B-PATH-01-P2-01: MCP health monitoring exists only in the GUI entry

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/desktop.rs:243` (sole call site of
  `infra::spawn_mcp_health_check`);
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:1111-1141`
  (loop: 5 s initial delay, then every 30 s → `state.run_mcp_health_check()`,
  `infra.rs:1125`)
- Reachability: `spawn_mcp_health_check` has exactly one caller in the whole
  repository (desktop.rs:243); `run_mcp_health_check` is reachable only from
  that loop. TUI (`main.rs`/`tui/mod.rs`), CLI (`modes.rs`/`repl.rs`) and
  channels (`modes.rs`) never spawn it.
- Expected invariant: every surface is a full Agent surface; services that
  keep a capability healthy in one mode exist in all modes (AGENTS.md
  surface parity).
- Observed behavior: only the desktop app periodically probes MCP server
  health; TUI/CLI/channels MCP connections are never proactively checked.
- Impact: a dead/hung MCP server in TUI/CLI/channels is only discovered at
  the next tool call (per-call failure), while GUI surfaces recover
  proactively within ~30 s; users of the default TUI get no early
  indication that an MCP server went away.
- Root cause: the health loop was added to the desktop path and never
  propagated to the shared headless entry.
- Direction: move the health-check spawn into the shared bootstrap or into
  `start_headless_services`, with the same cancellation token; delete the
  desktop-only call.
- Regression validation: run TUI and GUI with a configured MCP server that
  is stopped mid-session; verify both surfaces log a health failure within
  the probe window.
- Validation reports: [V04](../validations/B-PATH-01/V04-01.md)

### B-PATH-01-P2-02: TaskRuntimeStore construction and task-tool wiring duplicated across entry points

- Priority: P2
- Confidence: medium
- Layer: application
- Evidence: `src/main.rs:35-57` (`build_task_runtime_store_for_headless`),
  `echo-agent-app-core/src/state.rs:547-566` (TaskRuntimeStore in
  `AppState::from_shared`), `src/main.rs:177-181` + `:192-197` and
  `src/tauri/desktop.rs:201-206` + `:217-222` (the same two registration
  calls in both entries); the deferral is documented at
  `echo-agent-app-core/src/runtime.rs:120-125`
- Reachability: both construction sites run the same
  file-backed→in-memory fallback and `recover_incomplete()`; both entries
  call `register_task_tools_on_agent` then `bind_task_execute_to_pool`.
- Expected invariant: one composition path per service; entry points differ
  only in trigger/rendering policy.
- Observed behavior: the fallback+recover logic and the registration pair
  are written twice (three files), so a semantic change (e.g. recovery
  policy, registration order) must be applied in lockstep in both places.
- Impact: divergence risk — today the two paths are behaviorally equal
  (verified in V04), but a future edit to one site silently breaks the
  parity the code comments claim; the duplicated fallback logic also
  multiplies failure modes.
- Root cause: bootstrap deliberately defers TaskRuntimeStore creation
  (store not ready at primary-agent build time), and each entry re-implemented
  the same post-hoc wiring.
- Direction: extract the task-store construction + tool-registration +
  pool-binding sequence into one app-core helper (called by both entries),
  or pass an injectable store factory into bootstrap; delete the two inline
  copies.
- Regression validation: run the shared fixture through both entries and
  assert identical recovery counts and tool registration sets
  (A-TSK-01/A-TSK-03 own the behavioral matrix).
- Validation reports: [V02](../validations/B-PATH-01/V02-01.md),
  [V04](../validations/B-PATH-01/V04-01.md)

### B-PATH-01-P3-01: Legacy `--web`/`--port`/`--host` arguments remain dead surface

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `src/cli/args.rs:23` (`web`), `:31-35` (`port`, `host`);
  `src/main.rs:351-353` (`--web` → eprintln removal message + `exit(1)`);
  `src/main.rs:521,543` (tests exercising `port`)
- Reachability: `args.web` is read only to hard-exit; `args.port` and
  `args.host` are read by no production code (grep: only test assertions).
- Expected invariant: declared CLI surface matches live behavior.
- Observed behavior: the web mode was removed but its flags remain
  parseable and advertised by clap help; `--web` fails with a runtime
  message instead of a compile-time/arg-validation error.
- Impact: user confusion (a documented-looking flag that cannot work), dead
  test surface, and `port`/`host` fields with no consumer.
- Root cause: removal of the web mode left the argument schema and tests
  behind.
- Direction: delete `web`/`port`/`host` from `Args` (and their tests) or
  make `--web` a clap-level error; `A-SRF-04` should confirm no other
  surface reads them.
- Regression validation: `echo-agent-cli --help` no longer lists the flags;
  `test_args_internal_modes_remain_parseable` and
  `test_args_custom_port_for_internal_web` are removed/replaced.
- Validation reports: [V01](../validations/B-PATH-01/V01-01.md)

### B-PATH-01-P3-02: GUI entry silently ignores all CLI arguments

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `src/tauri/desktop.rs:132`
  (`cli::Args::parse_from(["echo-agent-tauri"])` — fixed argv)
- Reachability: every desktop launch (dedicated bin or gui-only default bin
  via main.rs:75-76) parses a hard-coded single argument; user-supplied
  `--config/--model/--project` are dropped without warning.
- Expected invariant: either GUI is config-file-driven by design (then the
  ignored args are acceptable but should be documented), or CLI args are
  honored.
- Observed behavior: the GUI silently ignores argv; `--config` cannot be
  used to point the desktop app at a custom config.
- Impact: a user running `echo-agent-cli --config prod.yaml` from the
  terminal in a gui-only build gets the default config with no error; CLI
  and GUI argument handling differ invisibly.
- Root cause: desktop entry always uses defaults, relying on the config file
  discovery chain.
- Direction: document the GUI config-source contract explicitly, or accept
  `--config`/`--model` in the desktop entry; `A-CFG-01` owns the final
  precedence decision.
- Regression validation: launching the desktop bin with `--config` either
  applies it or prints a clear "GUI ignores CLI args" notice.
- Validation reports: [V01](../validations/B-PATH-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Entry-point call graph | yes | passed | [V01](../validations/B-PATH-01/V01-01.md) |
| V02 | Composition-root inventory | yes | passed | [V02](../validations/B-PATH-01/V02-01.md) |
| V03 | Feature-gated reachability | yes | passed | [V03](../validations/B-PATH-01/V03-01.md) |
| V04 | Mode-to-service matrix | yes | failed | [V04](../validations/B-PATH-01/V04-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root `AGENTS.md`: TUI/GUI/CLI/channel are full Agents with parity | regressed (one gap) | MCP health monitoring is GUI-only; [V04](../validations/B-PATH-01/V04-01.md) |
| `main.rs` doc: "Web 模式已移除" | current | [V01](../validations/B-PATH-01/V01-01.md) |
| `runtime.rs` comment: task tools registered post-bootstrap by both entries | current | [V02](../validations/B-PATH-01/V02-01.md) |
| `desktop.rs` comment: Dreaming runs "in every mode" | current | Dreaming spawned in TUI (tui/mod.rs:1999), GUI (desktop.rs:247), CLI (repl.rs:106); [V04](../validations/B-PATH-01/V04-01.md) |

## Coverage And Uncertainty

- Channels-mode Dreaming: no explicit spawn was found; pooled per-sender
  agents may run memory maintenance themselves — impact requires A-MEM-01.
- Whether framework-level MCP reconnect-on-use masks the missing TUI/CLI
  health loop is F-INT-01's question.
- No process was launched; all reachability claims are static call-graph
  evidence.

## Handoff

- `A-BOOT-01` may rely on: one shared bootstrap; per-entry service assembly
  list; the duplicated task-store composition (P2-02).
- `A-SRF-04` should pick up P3-01 (dead args) and the channels surface;
  `A-INT-01` and `F-INT-01` should evaluate P2-01's impact.
- `X-SRF-01` should include the MCP-health row in the final capability
  matrix.
- This report becomes stale if entry dispatch, bootstrap steps, or the
  service assembly in any entry changes.
