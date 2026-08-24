# F-SUB-02: Subagent execution modes and teams

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Do Sync, Fork, Teammate, Team, manager, timeout, cancellation, and isolation
modes share one lifecycle without detached execution?

## Scope

Primary source paths inspected (deep read):

- `echo-agent/src/agent/subagent/executor.rs` (full, 3672 lines) —
  `dispatch` unified loop (`:407-781`), `dispatch_background` (`:836-868`),
  `dispatch_teammate` (`:871-980`), `dispatch_team` (`:992-1113`),
  `execute_agent_streaming` (`:1147-1453`), `dispatch_sync` (`:1469-1551`),
  `dispatch_fork` (`:1554-1888`, incl. worktree/workspace isolation and
  finalize), and the full test suite (`:1891-3672`).
- `echo-agent/src/agent/subagent/team/mod.rs` (full) — `Team`/`TeamMember`/
  `TeamConfig` (`:44-73`), `TeamAgent::execute_with_usage` timeout wrapper
  (`:343-353`), strategies (`:355-463`), builder (`:481-632`), tests.
- `echo-agent/src/agent/subagent/team/manager_subagent.rs` (full) — plan →
  fan-out → synthesize with checkpoint/resume; sub-task spawn/await/detach
  semantics (`:217-336`).
- `echo-agent/src/agent/subagent/team/coordinator.rs`, `runner.rs`, `mailbox.rs`,
  `agent_box.rs`, `message.rs`, `strategy.rs` (full or sampled) — supporting
  machinery and its (non-)reachability.
- `echo-agent/src/agent/subagent/worktree.rs`, `workspace.rs` (full) —
  isolation factory contracts.
- `echo-agent/src/agent/subagent/types.rs:1-260` — `ExecutionMode`,
  `ObservedIsolation`, `TeamSpec`, `SubagentDefinition` lifecycle fields.
- `echo-agent/src/tools/builtin/agent_dispatch.rs:150-460` — LLM-facing
  `agent_tool` mode routing, cancel-token derivation, background routing.
- `echo-agent/src/agent/react/mod.rs:405-440, 2260-2560` — executor
  construction (config threading), programmatic delegation helpers.
- `echo-agent/src/agent/config.rs:70-75, 220-225, 400-405, 520-530` —
  `subagent_timeout_secs` single source.
- `echo-agent-cli/echo-agent-app-core/src/infra.rs:400-440` (EKO factory
  injection), `subagent_loader.rs:440-500` (team frontmatter → `TeamSpec`),
  `plugin_components.rs:470-545` (plugin subagent registration, Team mode),
  `tasks/task_runtime/executor.rs:2820-2960` (programmatic delegation callers).

## Out Of Scope

- Definition/registry/prompt/result contract semantics → F-SUB-01 (its findings
  P1-01/P2-02 were cross-checked only at the dispatch surface).
- Batch tool execution, timeout/cancel of tool batches → F-RCT-04 (P1-02
  cross-referenced for the terminal-less-turn family).
- EKO TaskRuntime worktree reuse/merge/ownership, writer locks → A-TSK-05,
  F-EXT-02.
- Run-level cancel registry and its EKO surfaces → A-TSK-04, A-CHAT-01.
- Frontend subagent event projections → A-FE-02 / X-EVT-01.

## Inputs

- Root `AGENTS.md` (Subagent-only terminology, UTF-8/panic safety, one-authority,
  layering gates), shared `README.md`, `REPORTING.md`, `TASKS.md` (F-SUB-02
  card), `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-SUB-01` (complete — executor
  dispatch/compile surface, events/result contract) and `F-RCT-04` (complete —
  batch timeout/cancel terminal gaps, fixture gaps).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md`
  (:54, :100, :101, :149, :214-216, :233, :371), `echo-agent-cli/docs/MASTER-PLAN.md`
  (:56-61, :140-170), `2026-07-16-agent-lifecycle-audit.md`,
  `subagent-unification-plan.md` — classified in the Historical Claim Status
  section and V05-01.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | The unified dispatch loop, per-mode routers, `TeamAgent`/`TeamConfig`/`ManagerSubagentOrchestrator`, the `WorktreeFactory`/`DataWorkspaceFactory` traits, `TeamRunner`/`TeamCoordinator`/mailbox (framework API surface) — all correctly placed in `echo-agent`. The gaps found (team cancel/timeout wiring, dead team machinery) are framework-internal defects, not layering errors; no repository movement is recommended. |
| EKO product policy | TeamSpec construction from frontmatter/plugins (`subagent_loader.rs`, `plugin_components.rs`), the concrete `EkoWorktreeFactory`/`EkoDataWorkspaceFactory`/`FileRuntimeStateStore` injection (`infra.rs:401-437`), `register_agent_dispatch_tool` on EKO agents, and worktree keep/remove policy (A-TSK-05). |
| Adapter boundary | `ArcAgentBox` (thin `Arc<dyn Agent>` → `Box<dyn Agent>` adapter, no authority) and `agent_dispatch.rs` `ParentContextFactory`/`child_cancel_token` (thin context/cancel plumbing) — lossless, no scheduling authority. |
| Duplicate search | Terms: `worker`, `ExecutionMode`, `dispatch_sync/fork/teammate/team`, `TeamAgent`, `TeamRunner`, `TeamCoordinator`, `ManagerSubagent`, `Mailbox`, `fan_out`, `CancellationToken`/`child_token` in `team/`, `ObservedIsolation::Primary`/`PrimaryFallback`, `default_timeout_secs`, `subagent_timeout_secs`, `timeout_secs`, `max_concurrent_forks`, `Semaphore`, `TeamConfig`. Results: single dispatch lifecycle authority (`dispatch` loop); zero worker terminology; zero cancellation tokens in `team/`; `TeamCoordinator`/`TeamRunner`/`Mailbox` zero production callers (P2-03); `Primary`/`PrimaryFallback` zero producers (P3-01); semaphore bounds Fork only. |
| Migration deletion | If P2-03/P3-01 directions are taken: delete `TeamRunner`, `TeamCoordinator` (or `Team.coordinator` field), `mailbox.rs`, `message.rs` (and their tests) or wire them into a real mailbox strategy; delete the unproduced `ObservedIsolation` variants and `as_str` arms. |

## Current Path

Verified data flow: `agent_tool` (LLM, EKO-enabled `infra.rs:288, 930, 1008`) or
programmatic `delegate_to_agent_*` (TaskRuntime `executor.rs:2833, 2941, 2955`)
builds a `DispatchRequest` (mode override, child cancel token
`agent_dispatch.rs:164-178, 283-284`, parent context, runtime context) →
`SubagentExecutor::dispatch` (`executor.rs:407`) → pre-cancelled shortcut
(`:417-423`, no events — F-SUB-01-P3-02) → `DispatchStarted` + hooks
(`:510-526`) → mode match (`:552-565`) → `dispatch_sync` (inline select over
child token/timeout/stream, `:1469-1551`) / `dispatch_fork` (semaphore,
spawn, worktree/workspace isolation with hard-fail gates, select, finalize,
`:1554-1888`) / `dispatch_teammate` (spawn + handle, select, join back in the
loop, `:871-980, 555-561`) / `dispatch_team` (resolve manager/members by name,
build `TeamAgent` with run_id + state store, `execute_with_usage`, no cancel
usage, `:992-1113`) → terminal events + hooks + retry/delegation in the shared
loop (`:569-778`). Team members execute as shared registry singletons via
`ArcAgentBox` calling `agent.execute(&task)` directly; `ManagerSubagentOrchestrator`
writes checkpoint nodes (`team_{run_id}_{plan|subagent_{idx}|synthesis}`) when a
store is present (EKO injects `FileRuntimeStateStore`). Fork isolation:
`WorktreeHandle`/`DataWorkspaceHandle` created by injected factories, bound as
`working_dir`, finalized into the result (diff/file listing); no removal
contract at framework level (application policy, A-TSK-05).

## Findings

### F-SUB-02-P1-01: Team mode has no cancellation path — parent cancellation never propagates into a Team run, no `DispatchCancelled` terminal, and members keep executing after the parent is stopped

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `dispatch_team` (`echo-agent/src/agent/subagent/executor.rs:992-1113`)
  never reads `req.cancel`; `TeamAgent::execute_with_usage` (`team/mod.rs:343-353`)
  has no cancellation parameter (only a `tokio::time::timeout` wrapper);
  `ManagerSubagentOrchestrator` (`manager_subagent.rs:67-377`) — planning
  (`:192-196`), fan-out (`:277-284`) and synthesis (`:372-376`) call
  `agent.execute` with no token; V01-01: zero `CancellationToken` occurrences in
  the whole `team/` module. Contrast: Sync/Fork/Teammate derive
  `req.cancel.child_token()` (`executor.rs:1486, 1671, 879`) and race it with a
  biased select (cancel first) (`:1502-1506, 1774-1777, 922-926`).
- Reachability: EKO production — team roles declared via frontmatter
  (`subagent_loader.rs:454-487`) or plugins (`plugin_components.rs:481-514`),
  dispatched by the LLM `agent_tool` `mode="team"` (`agent_dispatch.rs:215,
  414-418`) or programmatic `dispatch()`; a parent cancel fired during the team
  run (user stops the turn / TaskRuntime cancel) is the trigger.
- Expected invariant: MASTER-PLAN:101 — "主运行被停止时，后台 Subagent 必须同步取消，
  不得继续脱离运行"; MASTER-PLAN:149 — cancellation propagates to every Subagent and
  ends in exactly one cancelled terminal. The F-SUB-01 V02-01 handoff confirmed
  the `ToolContext.cancel`-authoritative child token for the other modes.
- Observed behavior: cancel during a Team run is invisible; the team continues
  until completion or its own timeout (EKO default 600 s); the parent receives
  a `DispatchCompleted` (or `DispatchFailed`), never `DispatchCancelled`; no
  `AgentEvent::Cancelled` anywhere in the team path.
- Impact: a user who stops the main run still has team members working (and
  writing files) for up to 10 minutes; surfaces and hooks never see a
  cancellation; behavior is inconsistent with Sync/Fork/Teammate — the core
  product invariant "cancellation reaches every Subagent" is violated on one
  of the four documented modes.
- Root cause: `TeamAgent::execute` predates the unified cancellation contract;
  `dispatch_team` was wired on top of it without threading the token.
- Direction: thread `CancellationToken` through `TeamAgent::execute_with_usage`
  → `execute_inner` → `ManagerSubagentOrchestrator::run` → planning/fan-out/
  synthesis; race member futures against the token (e.g. `select!` in
  `execute_sub_tasks` with a per-member child token, or drop the join-set on
  cancel) so in-flight `agent.execute` futures are cancelled; let the existing
  `dispatch` loop emit the standard `DispatchCancelled` terminal (the status
  mapping at `executor.rs:138-147` already handles `AgentError::Interrupted`).
  Add a test: team with a hung member, cancel the parent token mid-run →
  `SubagentStatus::Cancelled`, member future dropped, exactly one terminal event.
- Regression validation: `cargo test -p echo_agent --lib --features "subagent,tasks"`
  with the new cancel fixture; keep V04-01/V04-02 green.
- Validation reports: [V01-01](../validations/F-SUB-02/V01-01.md),
  [V02-01](../validations/F-SUB-02/V02-01.md), [V03-01](../validations/F-SUB-02/V03-01.md),
  [V05-01](../validations/F-SUB-02/V05-01.md)

### F-SUB-02-P1-02: Team timeout/abort detaches the already-spawned member subagent tasks — the whole-team timeout drops `execute_inner` while `tokio::spawn`ed members keep running, so a reported failure coexists with live member execution

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `execute_with_usage` wraps `execute_inner` in
  `tokio::time::timeout` (`team/mod.rs:346-348`) — on expiry the future is
  dropped; `execute_sub_tasks` spawns members with `tokio::spawn`
  (`manager_subagent.rs:277-284`) and holds `JoinHandle`s in a local `Vec`
  (`:242, 288`) — dropping the vec detaches the tasks; `agent.execute(&task)`
  carries no token, so there is nothing to observe cancellation. There is no
  grace period, no sibling cancellation on one member's failure
  (`:288-331` awaits all handles regardless), and no cleanup on error
  (`dispatch_team` maps the error and returns, `executor.rs:1081-1087`).
- Reachability: any Team dispatch where a member exceeds the team budget (EKO
  default 600 s) or where the parent context is dropped mid-run; member agents
  are EKO writer roles that mutate files — the detached members keep writing
  after the parent has been told the team failed/timed out.
- Expected invariant: a timed-out/cancelled team stops its members
  (MASTER-PLAN:101 "不得继续脱离运行"); one terminal per dispatch.
- Observed behavior: on team timeout, the manager's in-flight `execute` is
  cancelled at its await point, but each already-spawned member task continues
  to completion (detached); on one member's failure, the remaining members are
  not cancelled; `dispatch_team` returns the error and the shared loop emits a
  terminal while members are still executing.
- Impact: concurrency/recovery error — the exact "detached execution" the task
  question asks about; side effects (file writes) continue after a reported
  failure; Q-FLT-02 would reproduce this today with a hanging member.
- Root cause: the outer-timeout design assumes dropping the orchestrator future
  cancels the work, but the fan-out layer escaped that scope via `tokio::spawn`
  with no cancellation contract on `Agent::execute`.
- Direction: with the P1-01 token threading, make `execute_sub_tasks`
  cancel-aware: race each member future against the token (or keep handles in a
  `JoinSet` and `abort_all` on timeout/cancel/error), optionally wait a short
  grace period for in-flight tool side effects to settle before returning;
  cancel siblings when one member fails (or explicitly document divergence).
  Add a fixture: one hung member + team timeout → after `execute_with_usage`
  returns Err, the member's `execute` future has been dropped (observable via a
  mock agent's drop/notification).
- Regression validation: new fixture in `team/mod.rs` tests; keep V04-02 green.
- Validation reports: [V01-01](../validations/F-SUB-02/V01-01.md),
  [V03-01](../validations/F-SUB-02/V03-01.md), [V04-02](../validations/F-SUB-02/V04-02.md)

### F-SUB-02-P2-01: Timeout ownership is split for Team mode — `SubagentDefinition.timeout_secs` and `SubagentExecutorConfig.default_timeout_secs` are ignored, `TeamConfig.default_timeout_secs` (a fourth knob from `TeamSpec.config`) governs, and `execute_with_usage` enforces an undocumented 60 s floor that violates the documented "0 = no timeout"

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: Sync/Fork/Teammate resolve `definition.timeout_secs > 0 ? def :
  executor.default_timeout_secs` (`executor.rs:1475-1478, 1569-1573, 891-895`;
  executor default = `AgentConfig.subagent_timeout_secs`, `react/mod.rs:425-426`);
  `dispatch_team` builds the `TeamAgent` without `.timeout_secs(...)`
  (`executor.rs:1031-1043`), so the executor/definition knobs are unreachable
  for Team (builder doc at `team/mod.rs:562-569` says callers thread the
  unified config — `dispatch_team` does not); `execute_with_usage` uses
  `self.team.config.default_timeout_secs.max(60)` (`team/mod.rs:345`) while the
  field doc says "0 = no timeout" (`team/mod.rs:49`); EKO always sets
  `TeamConfig::default()` (600) (`subagent_loader.rs:471`); the comment
  `executor.rs:1474` ("one config ... governs all three modes") is stale for
  four modes. Timeout classification additionally relies on string matching
  (`error.to_ascii_lowercase().contains("timed out")`, `executor.rs:1082-1084`).
- Reachability: any app configuring `subagent_timeout_secs` (EKO or other) or a
  team definition with `timeout_secs` gets a silent different value for Team;
  a definition `timeout_secs: 0` ("no timeout") still caps the team at 600 s.
- Expected invariant: one timeout authority per dispatch (the definition
  override or the unified executor default); the documented "0 = no timeout"
  holds everywhere; docs describe the actual control flow.
- Observed behavior: three knobs exist for four modes; Team reads a fourth,
  with a 60 s minimum that is nowhere documented and contradicts the field doc;
  today's EKO default (600 everywhere) hides the divergence.
- Impact: misleading public API and product config; a user who raises/lowers
  `subagent_timeout_secs` sees every mode honor it except Team; silent
  divergence across mode family.
- Root cause: `dispatch_team` was implemented on the pre-existing `TeamAgent`
  API (its own `TeamConfig`) without adapting the unified timeout contract.
- Direction: in `dispatch_team`, apply `definition.timeout_secs` then the
  executor `default_timeout_secs` into the builder's `.timeout_secs(...)`
  (single source), and honor "0 = no timeout" by removing the `.max(60)` floor
  (or documenting the floor if deliberate); replace the string-match timeout
  classification with the typed error path. Add a builder/executor test
  asserting the team timeout equals the executor default when the definition
  has none.
- Regression validation: `cargo test -p echo_agent --lib --features "subagent,tasks"`
  green; new timeout-threading fixture; `test_team_agent_builder_timeout_override`
  (`team/mod.rs:728-742`) extended for the executor path.
- Validation reports: [V01-01](../validations/F-SUB-02/V01-01.md),
  [V03-01](../validations/F-SUB-02/V03-01.md)

### F-SUB-02-P2-02: The task card's required team fixtures do not exist — no test covers a failing team member (partial success), team timeout, team cancellation, or member cleanup; MASTER-PLAN:233's acceptance ("多 Subagent 部分成功 ... 均有测试") is unfulfilled; the Swarm strategy silently swallows member failures

- Priority: P2
- Confidence: high
- Layer: framework (test coverage)
- Evidence: test inventory — executor tests cover routing/terminal/isolation/
  timeout-cancel for Sync/Fork/Teammate only (`executor.rs:2604-2983, 3116-3671`);
  team tests cover happy path + checkpoints/resume + config alignment
  (`manager_subagent.rs:538-728`, `team/mod.rs:704-826`); zero tests for a
  failing member during fan-out, team timeout, team cancel, or cleanup of
  in-flight members (V03-01). Swarm strategy drops member errors:
  `if let Ok((name, Ok(output))) = h.await` (`team/mod.rs:437`) — failures
  silently vanish (programmatic-only strategy today). MASTER-PLAN:233 lists
  "多 Subagent 部分成功" and "synthesis 失败" acceptance tests.
- Reachability: not-applicable (test gap); the missing fixtures are exactly
  what Q-FLT-02 needs.
- Expected invariant: the required validations "team partial-failure and
  cleanup fixtures" are exercised by tests that fail before the P1 fixes and
  pass after.
- Observed behavior: ManagerSubagent's partial-failure design (errors collected
  into `results`, surfaced to synthesis with truthful-reporting instructions,
  `manager_subagent.rs:356-359, 367-369`) is sound but unproven by any test;
  cleanup behavior is entirely untested.
- Impact: the two P1 defects shipped without a regression net; Q-FLT-02 and
  X-TSK-01 have no fixtures to reuse; future team changes are unprotected.
- Root cause: team execution was added with happy-path and checkpoint tests
  only; failure-path fixtures were never written.
- Direction: add the fixture family: (a) one failing member → synthesis receives
  the error and the team still completes; (b) hung member + team timeout →
  members cancelled/aborted (must fail today per P1-02); (c) parent cancel
  mid-team → `DispatchCancelled` (must fail today per P1-01); (d) member
  cleanup/grace after abort. Fix or delete the Swarm silent-swallow (report
  failures to the reducer text or return an error listing them).
- Regression validation: the new fixtures themselves; `cargo test -p echo_agent
  --lib --features "subagent,tasks"` stays green after P1 fixes land.
- Validation reports: [V03-01](../validations/F-SUB-02/V03-01.md),
  [V04-01](../validations/F-SUB-02/V04-01.md), [V04-02](../validations/F-SUB-02/V04-02.md),
  [V05-01](../validations/F-SUB-02/V05-01.md)

### F-SUB-02-P2-03: `TeamCoordinator`, `TeamRunner`, and the mailbox machinery are dead public framework APIs — the documented "member/mailbox lifecycle" (CLI MASTER-PLAN:152) does not exist; `Team::coordinator` is created but never used, and `ManagerSubagentOrchestrator` drives members with plain `agent.execute`

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: zero production callers for `TeamRunner::fan_out/fan_out_to`,
  `TeamCoordinator::*`, `Mailbox`, `TeamMessage` in either repo (V01-01 — only
  the defining modules, the re-export `team/mod.rs:16`, and `#[cfg(test)]`
  uses); `Team::coordinator` field (`team/mod.rs:104`) constructed in
  `Team::new` (`:128`) and never invoked; `TeamConfig.allow_reassignment` /
  `cross_talk` / `mailbox_capacity` (`team/mod.rs:51-56`) have no readers
  either; the CLI MASTER-PLAN:148-152 text claims "TeamAgent's persistent
  member/mailbox lifecycle remains a separate path"; the executor module doc
  (`executor.rs:1`) still says "Sync / Fork / Teammate modes".
- Reachability: none in production; only unit tests (`coordinator.rs:244-346`,
  `team/mod.rs:806-826`) construct these types.
- Expected invariant: no dead public API advertising parallel coordination
  semantics (AGENTS.md cleanup + one-authority); documented architecture
  matches code.
- Observed behavior: three coordination layers coexist — the dead mailbox
  machinery, the live `ManagerSubagentOrchestrator` fan-out, and the
  programmatic-only `Swarm`/`Debate`/`Pipeline` strategies — while only the
  orchestrator is reachable; `allow_reassignment`/`cross_talk` config exists
  but no code reads it.
- Impact: misleading public API (a consumer wiring `TeamCoordinator` gets a
  system nobody else uses); the "mailbox lifecycle" doc claim is fiction;
  dead code per cleanup rules; maintenance burden when the team module evolves.
- Root cause: the coordinator/mailbox subsystem predates the Sprint 11
  orchestrator rewrite and was never deleted or wired.
- Direction: either (a) wire the mailbox/coordinator into a real
  message-passing strategy and make it reachable from a `TeamStrategy`, or
  (b) delete `TeamRunner`, `TeamCoordinator`, `mailbox.rs`, `message.rs`, the
  `Team.coordinator` field, and the unread `TeamConfig` fields
  (`allow_reassignment`, `cross_talk`, `mailbox_capacity`) with their tests;
  fix the executor module doc to include Team. Prefer (b) unless a strategy
  needs (a).
- Regression validation: `cargo check -p echo_agent --features subagent` after
  removal; grep for the removed names returns nothing; V04-02 suite green
  (tests referencing the deleted types are deleted with them).
- Validation reports: [V01-01](../validations/F-SUB-02/V01-01.md),
  [V03-01](../validations/F-SUB-02/V03-01.md), [V05-01](../validations/F-SUB-02/V05-01.md)

### F-SUB-02-P3-01: `ObservedIsolation::Primary` and `PrimaryFallback` are never produced, and Sync dispatches always report `Unknown` isolation

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: variants declared at `types.rs:57-58` with `as_str` arms at `:68-71`;
  zero producers in either repo (V01-01). Producers of the enum: Fork sets
  `Worktree`/`Workspace`/`Context` (`executor.rs:1737-1743, 1829-1831`), Team
  hardcodes `Subagent` (`:1104`), Sync/Teammate leave the `execute_agent_streaming`
  default `Unknown` (`:1439`).
- Reachability: `ObservedIsolation` is serialized into `SubagentResult` and
  events; consumers see `Unknown` for every Sync/Teammate dispatch.
- Expected invariant: the observed-isolation report is accurate per mode
  ("requested/observed path", MASTER-PLAN:371); no unproduced variants.
- Observed behavior: Sync/Teammate dispatches report `Unknown` even though they
  run in the parent context (the enum's `Context` semantics would apply); two
  variants are dead.
- Impact: cosmetic/observability; `Unknown` is a truthful-but-uninformative
  value; dead variants add serialization surface.
- Root cause: the enum grew before the per-mode observation logic landed;
  Sync/Teammate never got their observation set.
- Direction: set `ObservedIsolation::Context` (or a new `None`/`Parent` variant)
  in `dispatch_sync`/`dispatch_teammate` results; delete the unproduced
  `Primary`/`PrimaryFallback` variants and their `as_str` arms.
- Regression validation: unit test asserting the observed isolation value per
  mode; grep for `PrimaryFallback` returns nothing.
- Validation reports: [V01-01](../validations/F-SUB-02/V01-01.md)

### F-SUB-02-P3-02: `dispatch_teammate` holds a misleading `_permit` placeholder, and Teammate/Sync/Team dispatches never acquire the executor's concurrency semaphore (`max_concurrent_forks` bounds Fork only)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `let _permit = child_token.clone();` (`executor.rs:915`) — a cloned
  cancellation token named like a concurrency permit, kept alive for the task
  duration; the semaphore is created from `max_concurrent_forks`
  (`executor.rs:363, 369`) and acquired only in `dispatch_fork`
  (`:1555-1560`); `dispatch_sync`/`dispatch_teammate`/`dispatch_team` never
  acquire it; the config doc (`:305-306`) names the knob "Maximum concurrent
  Fork dispatches", so the naming is honest, but the `_permit` binding is not.
- Reachability: every Teammate dispatch holds the fake permit; N parallel
  Teammate/Sync dispatches are unbounded.
- Expected invariant: no misleading dead bindings; the concurrency knob's scope
  is documented or enforced consistently across modes.
- Observed behavior: `_permit` suggests a permit where none is held; parallel
  Teammate dispatches are unbounded while the knob implies a cap for the
  subagent family.
- Impact: cosmetic/maintainability; a reader may "fix" the missing permit
  incorrectly; resource spikes for many parallel teammates are unconstrained.
- Root cause: the binding predates the semaphore wiring (Fork-only) and was
  never removed.
- Direction: delete the `_permit` binding; either document that
  `max_concurrent_forks` caps only Fork (rename to `max_concurrent_forks` is
  already accurate) or acquire the shared semaphore in Teammate/Sync paths.
- Regression validation: clippy/grep cleanup; V04-01 green.
- Validation reports: [V01-01](../validations/F-SUB-02/V01-01.md),
  [V03-01](../validations/F-SUB-02/V03-01.md)

### F-SUB-02-P3-03: `TeammateHandle` has no `Drop` cancellation and the Fork semaphore permit wait is not cancellation-aware — dropping a handle detaches its task until its own timeout, and a cancelled Fork dispatch blocks on the permit queue

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `TeammateHandle` (`executor.rs:262-271`) holds `join_handle` +
  `cancel` with no `Drop` impl (V01-01 grep: zero matches); dropping it
  detaches the spawned task which runs until completion or its timeout (default
  600 s) — `dispatch()`'s Teammate arm always joins (`:557-559`), but
  programmatic callers (public `dispatch_teammate`) may drop. Fork:
  `acquire_owned()` (`executor.rs:1555-1560`) sits outside the cancel select —
  a parent-cancelled Fork dispatch blocks until a permit frees (bounded only
  by other forks' completion), then the spawn sees the cancelled token and
  returns Cancelled without running (`:1684-1692`).
- Reachability: programmatic `dispatch_teammate` users; any Fork dispatch
  queued behind 5 running forks when the parent cancels.
- Expected invariant: dropping the handle should cancel the detached work (or
  be documented as fire-and-forget); cancellation is acknowledged promptly.
- Observed behavior: leaked/detached teammate execution for up to the timeout;
  delayed cancellation acknowledgement on a saturated fork pool.
- Impact: resource/process leaks in long-lived applications; cancellation feels
  unresponsive under load.
- Root cause: handle lifecycle was left to the caller; the permit wait predates
  the cancel contract.
- Direction: implement `Drop for TeammateHandle` that cancels the token (and
  document join-vs-drop semantics), or expose an explicit `cancel_and_join`;
  wrap the permit acquisition in a `select!` against the parent token (release
  the permit on cancel).
- Regression validation: unit test dropping a TeammateHandle mid-run asserting
  the token is cancelled; a saturated-permit fixture asserting a cancelled
  dispatch returns within a bounded time.
- Validation reports: [V03-01](../validations/F-SUB-02/V03-01.md),
  [V01-01](../validations/F-SUB-02/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition/duplicate search (mode/manager/team authorities, worker terms, timeout/cancel/isolation inventories, dead-API search) | yes | passed | [V01-01](../validations/F-SUB-02/V01-01.md) |
| V02 | Registration and runtime reachability trace (dispatch loop → four mode routers; EKO factories; team frontmatter/plugin reachability) | yes | passed | [V02-01](../validations/F-SUB-02/V02-01.md) |
| V03 | Invariant/edge-case inspection (mode lifecycle matrix; parent cancellation propagation; timeout ownership; team partial failure and cleanup; fixture inventory; F-SUB-01 cross-checks) | yes | passed | [V03-01](../validations/F-SUB-02/V03-01.md) |
| V04 | `cargo test -p echo_agent --lib --features "subagent,tasks" --locked 'agent::subagent'` | yes | passed (exit 0; 127 passed) | [V04-01](../validations/F-SUB-02/V04-01.md) |
| V04 | `cargo test -p echo_agent --lib --features "subagent,tasks" --locked 'agent::subagent::team'` | yes | passed (exit 0; 24 passed) | [V04-02](../validations/F-SUB-02/V04-02.md) |
| V05 | Historical-document drift (root + CLI MASTER-PLAN, lifecycle audit, unification plan) | yes | passed | [V05-01](../validations/F-SUB-02/V05-01.md) |

All required validations executed with known exit codes; no validation is
pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:54 — Subagent independent context, independent tool/permission config | current (context/summary) / regressed in part (definition tool/permission) | F-SUB-01-P1-01; re-confirmed fork-only `allowed_tools` (`executor.rs:1672-1676, 1744-1745`); [V03-01](../validations/F-SUB-02/V03-01.md) |
| MASTER-PLAN:100 — Sync/Fork/Teammate/Team + timeout/checkpoint/isolation basics | current / regressed in part (Team timeout/cancel) | `team/mod.rs:343-353`; `executor.rs:992-1113`; [V05-01](../validations/F-SUB-02/V05-01.md) |
| MASTER-PLAN:101 — child cancel token authoritative; stopped main run must cancel background Subagents, never keep running detached | regressed (Team path) | no cancel wiring in `team/` (P1-01, P1-02); [V05-01](../validations/F-SUB-02/V05-01.md) |
| MASTER-PLAN:149 — cancellation propagates to Subagents, one cancelled terminal | regressed (Team path; batch path per F-RCT-04-P1-02) | P1-01; [V05-01](../validations/F-SUB-02/V05-01.md) |
| MASTER-PLAN:214-216 — unified result with completed/failed/cancelled/timed_out | current | `types.rs:258-268`; `executor.rs:138-147`; [V05-01](../validations/F-SUB-02/V05-01.md) |
| MASTER-PLAN:233 — acceptance tests for timeout, single failure, partial success, synthesis failure, resume | stale/unfulfilled (team partial success/synthesis failure/cleanup) | P2-02; [V05-01](../validations/F-SUB-02/V05-01.md) |
| CLI MASTER-PLAN:148-152 — fresh instance per invocation + child cancel for Sync/Fork/Teammate; TeamAgent persistent member/mailbox lifecycle | current (first part) / stale (mailbox lifecycle) | P2-03; [V05-01](../validations/F-SUB-02/V05-01.md) |

## Coverage And Uncertainty

- All conclusions are static except the V04 test runs; no live LLM team run was
  executed (read-only review). P1-01/P1-02 rest on the absence of any
  `CancellationToken` in the team path (V01-01) plus the `tokio::spawn` drop
  semantics (static, standard Tokio behavior); dynamic proof belongs to
  Q-FLT-02 once P2-02 fixtures exist.
- The team partial-failure design (errors surfaced into synthesis) was judged
  sound but unproven; whether the P2-03 machinery is deleted vs wired is a
  product decision — the finding documents the divergence, not the choice.
- The EKO `EkoWorktreeFactory` cleanup/merge behavior was not inspected
  (A-TSK-05 scope); the framework `WorktreeHandle` contract (finalize-only, no
  removal) is recorded here only as a boundary note.
- The `Swarm`/`Debate`/`Pipeline` strategies were read only for their
  failure-handling semantics (P2-02 note); they remain programmatic-only with
  zero production callers (V01-01).
- Run-level cancel-registry claims in `2026-07-16-agent-lifecycle-audit.md` are
  application-side and were not revalidated (A-TSK-04/A-CHAT-01 scope).

## Handoff

- Conclusions downstream tasks may rely on: one unified dispatch lifecycle
  shell for all four modes with per-mode routers (V02-01); Sync/Fork/Teammate
  propagate parent cancellation and own their timeout (tests green, V04-01);
  Team is a non-cancellable, timeout-detached island (P1-01/P1-02); Team
  timeout comes from a fourth knob with a 60 s floor (P2-01); required team
  partial-failure/cleanup fixtures do not exist (P2-02); coordinator/runner/
  mailbox machinery is dead (P2-03); isolation is Fork-only with hard-fail
  gates and framework finalize-only handles (V03-01).
- F-SUB-01 cross-checks (independent re-confirmation at the dispatch surface):
  `tool_filter`/per-role tool restriction has zero readers; only Fork honors
  `allowed_tools` (`executor.rs:1672-1676, 1744-1745`); `inherit_history` is
  Fresh-transferred for Sync/Teammate/Team and pre-sliced to 2 on the LLM Fork
  path — the P1-01 fix (invocation `disabled_tools` in all modes) must extend
  to Teammate/Team invocation construction, not just Fork.
- `F-TSK-03`: RuntimeDagExecutor's per-task subagent dispatch uses the
  programmatic fork path (TaskRuntime `executor.rs:2833, 2941, 2955`) — the
  P1 fixes must not change the `delegate_to_agent_*` signatures it relies on.
- `Q-FLT-02`: build the subagent fault fixtures from P2-02's list (team
  partial failure, team timeout/cancel, member cleanup); P1-01/P1-02 fixtures
  must fail before the fixes.
- `X-BND-01`: record the team timeout-authority decision (P2-01), the
  coordinator/runner deletion-vs-wiring decision (P2-03), and the
  `max_concurrent_forks` scope (P3-02).
- Reports to read: this report + V01-01..V05-01; dependency reports F-SUB-01
  and F-RCT-04.
- Stale triggers: changes to `executor.rs` (dispatch/team/teammate/fork paths),
  `team/` (mod/manager_subagent/coordinator/runner/mailbox), `worktree.rs`,
  `workspace.rs`, `types.rs` (ExecutionMode/ObservedIsolation/TeamSpec),
  `agent_dispatch.rs`, `react/mod.rs` executor construction, or EKO
  `infra.rs`/`subagent_loader.rs` subagent wiring invalidate the corresponding
  claims.
- Follow-up task IDs (fixes not implemented in this review): F-TSK-03,
  Q-FLT-02, A-TSK-05, X-BND-01, S-RDM-01 (P1-01/P1-02/P2-01/P2-02/P2-03
  deletion targets).
