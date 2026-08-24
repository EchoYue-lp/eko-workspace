# A-PLG-01: Skills, plugins, hooks, and reload lifecycle

> Status: complete
> Reviewer: Codex primary reviewer (delegated evidence independently sampled)
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean at baseline; only Codex review reports added

## Question

Does EKO discovery, activation, reload, and unload correctly apply product
components while framework registrations roll back and clean up coherently?

## Scope

- EKO `PluginRuntimeService`, product component preparation and projection.
- Plugin enable/disable/install/configure/reload/uninstall and native lifecycle.
- Tauri, CLI, TUI, frontend, primary-Agent, AgentPool, scheduler and LSP
  reachability.
- SkillsHub enable/disable durable policy and runtime/pool projection.
- Config-watched user Hooks, plugin Hooks, TaskRuntime Hook dispatcher queue,
  flush and shutdown boundaries.
- Existing tests and future regressions, inspected statically only.

## Out Of Scope

- Framework manifest/registry/integrator defects already owned by `F-PLG-01`.
- Framework Skill discovery/activation/unload defects owned by `F-SKL-01`.
- General config precedence and webhook invalid-reload behavior owned by
  `A-CFG-01`.
- Tool implementation, MCP protocol, LSP protocol, hook payload semantics,
  frontend visual design, implementation fixes, and source mutation.
- Cargo, rustc, tests, builds, fixtures, network, and runtime launch.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, exact task card in
  `TASKS.md`; Codex isolation rules.
- Codex dependencies `F-SKL-01`, `F-PLG-01`, and `A-CFG-01` only.
- Current clean committed source at the revisions above.
- No other reviewer directory or report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Manifest/schema validation, source identity, dependency ordering, reversible component receipts, generic Skill/Hook/MCP registration, and lifecycle transition outcomes belong to `echo-agent`. |
| EKO product policy | Enabled local roots, SkillsHub/install state, Subagent/LSP/monitor/theme/style construction, pool/surface propagation, and live-versus-restart responses belong to `echo-agent-cli`. |
| Adapter boundary | EKO prepares product files and commits its projections around one framework component transaction. It must not own a second dependency graph or component ownership registry. |
| Duplicate search | Both repositories were searched by type, trait, store, component, command, registration, mutation, flush/shutdown behavior, and real callers. |
| Migration deletion | Keep the shared service. Replace branch-local best-effort rollback with one receipt/outcome, then delete restore helpers; do not introduce another plugin host or Skill registry. |

Local user-installed extensions remain user-trusted. These findings prevent
state divergence and unintended behavior; none justify cloud-style permission
gates.

## Current Path

```text
AgentRuntime::bootstrap
  -> one PluginRuntimeService -> initial reload
  -> framework Registry scan/dependency order
  -> EKO prepare Subagent/LSP/monitor/theme/style
  -> lifecycle deactivate + monitor candidate
  -> primary Agent: unload old -> framework wire candidate -> Subagent register
  -> LSP manager swap -> lifecycle activate
  -> output-style projection on primary -> PluginLoaded events

Tauri/CLI/TUI plugin commands -> same PluginRuntimeService
GUI/channel/task/cron execution -> AgentPool Agents
  -> shared ToolManager and HookRegistry
  -> independent system-context projections

TaskRuntime persisted event -> bounded HookEventDispatcher queue
  -> bridge reads shared HookRegistry at consumption time
Config/plugin reload -> mutates same HookRegistry without queue generation barrier
```

Positive conclusions:

- EKO has one live plugin mutation owner and does not duplicate the framework
  dependency graph.
- Product-specific Subagent/LSP/monitor/theme/style policy stays in EKO.
- Candidate product files and duplicate names are prepared/validated before
  primary-Agent mutation.
- Framework Tools and Hooks reach pool Agents through shared registries.
- Reload is serialized; monitor replacement, LSP replacement, and primary
  components have substantial compensation; the existing tests cover those
  primary happy/failure paths.
- Plugin commands across GUI/TUI/CLI converge on the same service, without an
  inappropriate permission gate for user-selected local extensions.

## Findings

### A-PLG-01-P1-01: EKO suppresses compensation failures after durable plugin mutations

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:221`,
  `:233`, `:242`, `:254`, `:266`, `:278`, `:360`, `:377`, `:1016`, `:1030`,
  `:1039`.
- Reachability: every Tauri/CLI/TUI enable, disable, install and configure call
  reaches these service methods; registry mutation persists before candidate
  live application.
- Expected invariant: failure either leaves durable and live generations both
  old, or reports an explicit indeterminate/rollback-failed outcome.
- Observed behavior: compensation starts a new scan; scan failure is ignored,
  mutation/cleanup failure is logged, and the command returns only the original
  apply error. No rollback receipt reaches the caller.
- Impact: the current session can retain the old generation while restart uses
  a failed enabled/configured candidate or a partial installed artifact. The
  user cannot distinguish safe rejection from cleanup debt.
- Root cause: the application brackets several already-persisting registry APIs
  with `async fn -> ()` best-effort helpers instead of owning one transaction.
- Direction: consume a framework prepare/commit/reversible receipt; until then,
  return typed mutation plus compensation outcomes and retain cleanup ownership.
  Delete `restore_enabled_state`, `restore_plugin_config`, and
  `rollback_install` after the canonical transaction replaces them.
- Regression validation: inject scan/save/uninstall failure at each compensation
  boundary and assert exact disk/live/restart state plus command outcome.
- Validation reports: [V04](../validations/A-PLG-01/V04-01.md).

### A-PLG-01-P1-02: Active plugin output styles never reach pooled conversation and background Agents

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:457`,
  `:480`, `:723`, `:741`, `:755`;
  `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:219`, `:233`, `:824`,
  `:882`, `:934`;
  `echo-agent-cli/echo-agent-app-core/src/state.rs:301`.
- Reachability: GUI chats and channel/task/cron/background paths acquire pool
  Agents; Tauri/CLI/TUI can persist and report one active plugin output style.
- Expected invariant: an active response-style policy affects primary, existing
  pool, and future pool Agents, or the surface reports its narrower scope.
- Observed behavior: the projection is applied only through the primary
  `AgentHandle`. It is not a shared registry, no pool update exists, and future
  `create_agent` does not consume plugin preferences.
- Impact: settings and TUI may report a style active while GUI conversations,
  channels and background work answer without it; behavior varies by mode and
  Agent creation time.
- Root cause: plugin runtime was constructed before AgentPool and retained only
  one Agent identity; per-Agent prompt policy was treated like shared component
  state.
- Direction: make EKO own one active product prompt projection and propagate it
  to primary, existing pool and future pool construction using an explicit pool
  operation/generation. Keep generic context projection in the framework.
- Regression validation: style activate/reload/remove across primary, existing
  and future conversation, background, channel, cron and task Agents.
- Validation reports: [V03](../validations/A-PLG-01/V03-01.md),
  [V09](../validations/A-PLG-01/V09-01.md).

### A-PLG-01-P1-03: Hook reload reinterprets already persisted queued events under a different registry generation

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:249`,
  `:258`, `:261`;
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/hook_event_dispatcher.rs:58`,
  `:92`, `:94`, `:152`;
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:246`;
  `echo-agent/src/hooks_bridge.rs:99`, `:125`, `:170`, `:238`, `:285`;
  `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:817`, `:1165`.
- Reachability: TaskRuntime appends persisted events synchronously and dispatches
  them through the bounded queue; config watcher and plugin commands mutate the
  shared HookRegistry while tasks may still be active.
- Expected invariant: an event is processed by the Hook generation active at
  persistence, or reload establishes a documented flush/snapshot boundary.
- Observed behavior: queued commands contain translations but no registry
  generation. The bridge reads the shared registry only when the consumer later
  fires. Reload clears/unregisters/registers without using the implemented
  TaskRuntime `flush_hook_events` barrier.
- Impact: a pre-reload event can unexpectedly invoke a new Hook, skip an old
  Hook, or cross plugin/user configuration generations, breaking deterministic
  lifecycle automation and audit interpretation.
- Root cause: ordered event transport and mutable Hook configuration have
  independent lifecycle owners with no generation coordinator.
- Direction: application reload should acquire one hook-generation coordinator,
  flush through an explicit safe point or attach an immutable registry snapshot/
  generation to queued work, then commit the new registry. Do not create a
  second Hook registry authority.
- Regression validation: pause the consumer, enqueue persisted events, reload
  user and plugin Hooks, resume, and assert the chosen old-or-new contract.
- Validation reports: [V06](../validations/A-PLG-01/V06-01.md),
  [V09](../validations/A-PLG-01/V09-01.md).

### A-PLG-01-P1-04: Skill enable/disable reports durable success after persistence failure

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/src/tauri/commands/panels.rs:388`, `:394`, `:402`,
  `:423`, `:592`, `:610`, `:621`, `:635`, `:650`, `:655`;
  `echo-agent-cli/echo-agent-app-core/src/skills_hub/enabled_skills.rs:84`,
  `:100`.
- Reachability: registered GUI Skill enable/disable commands call these helpers;
  bootstrap reads `enabled-skills.json` for later sessions.
- Expected invariant: claimed enable/disable durability survives restart, or the
  response explicitly reports a live-only/failed persistence outcome.
- Observed behavior: load failure silently switches to defaults, save failure is
  logged, and `persist_skill_enabled` cannot fail its caller. Enable already
  loads the Skill live; disable claims restart will apply the persisted removal.
- Impact: enable may vanish at restart; disable may return success and then
  re-enable the Skill at restart. UI state and actual future capability diverge.
- Root cause: EKO treats durable extension policy as best-effort telemetry after
  runtime/catalog mutation.
- Direction: use an atomic, fallible enabled-Skill policy transaction and return
  structured live/durable/restart state. Preserve last-known-good data on parse
  error; do not add an extension permission gate.
- Regression validation: corrupt/read-only/unwritable config for enable and
  disable, restart reconstruction, and primary/pool projections.
- Validation reports: [V07](../validations/A-PLG-01/V07-01.md),
  [V09](../validations/A-PLG-01/V09-01.md).

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Clean source baseline | yes | passed | [report](../validations/A-PLG-01/V00-01.md) |
| V01 | Definition, duplicate and layering map | yes | passed with inherited deviations | [report](../validations/A-PLG-01/V01-01.md) |
| V02 | Prepare/activate/live component reachability | yes | passed with framework dependencies | [report](../validations/A-PLG-01/V02-01.md) |
| V03 | Pool and mode projection | yes | failed -> finding | [report](../validations/A-PLG-01/V03-01.md) |
| V04 | Failed mutation compensation | yes | failed -> finding | [report](../validations/A-PLG-01/V04-01.md) |
| V05 | Reload/unload/lifecycle ownership | yes | passed with inherited deviations | [report](../validations/A-PLG-01/V05-01.md) |
| V06 | Hook generation and queue flush | yes | failed -> finding | [report](../validations/A-PLG-01/V06-01.md) |
| V07 | Skill enable/disable persistence | yes | failed -> finding | [report](../validations/A-PLG-01/V07-01.md) |
| V08 | GUI/TUI/CLI command convergence | yes | passed with product deviations | [report](../validations/A-PLG-01/V08-01.md) |
| V09 | Existing test inventory | yes | passed with gaps | [report](../validations/A-PLG-01/V09-01.md) |
| V10 | Dynamic regression matrix | future | not_run by direction | [report](../validations/A-PLG-01/V10-01.md) |
| V99 | Static report integrity gate | yes | passed | [report](../validations/A-PLG-01/V99-01.md) |
| V30 | Primary source sampling and acceptance | yes | passed | [report](../validations/A-PLG-01/V30-01.md) |

## Historical Claim Status

| Dependency claim | Classification | Current evidence |
|---|---|---|
| `F-PLG-01-P1-01` through `P1-06`: identity/cardinality/install/wiring/MCP/state contracts | current; not duplicated | V01, V02, V04, V05 |
| `F-PLG-01-P1-07`/`P2-08`: uninstall and callback cleanup debt | current; not duplicated | V05 |
| `F-SKL-01-P1-01`: two Skill activation authorities diverge | current; EKO durability impact deepened only | V01, V07 |
| `F-SKL-01-P2-05`: code Skill unload/shutdown unreachable | current framework issue; not duplicated | V07 |
| `A-CFG-01-P1-04`: invalid config reload clears webhooks | current but out of scope | V06 only uses Hook last-known-good path |
| `A-CFG-01-P1-05`: GUI config writes claim success on save failure | current pattern; A-PLG adds separate enabled-Skill artifact evidence | V07 |

## Coverage And Uncertainty

- No Cargo, rustc, test, build, fixture, application launch, or network command
  was run. V10 lists future dynamic evidence.
- Static ownership and call-chain evidence is conclusive for the four findings;
  exact race schedules and injected persistence failures remain unexecuted.
- Framework component payload correctness is delegated to its owning tasks. This
  report follows only EKO ownership, propagation, rollback and lifecycle.
- Full graceful process teardown was not elevated to a finding: lifecycle Drop
  and LSP `kill_on_drop` provide last-owner cleanup, while exact flush/exit timing
  still needs V10.
- Theme projection is surface-specific and was not incorrectly required on
  pooled Agents. Output style is prompt policy and therefore was.

## Handoff

- First converge framework and EKO on one source-owned prepare/commit/reversible
  plugin generation, then delete EKO's silent restore helpers.
- Add an EKO extension-generation coordinator spanning primary/pool prompt
  projection and Hook queue safe points; do not move pool/UI policy into the
  framework.
- Make `enabled-skills.json` mutation fallible and atomic before promising
  restart behavior. Reuse the framework's canonical Skill registry/unload path
  once `F-SKL-01` is fixed; do not add another runtime Skill authority.
- Primary must independently sample source anchors and run the static integrity
  gate before changing A-PLG-01 from `needs_evidence` to `complete`.
