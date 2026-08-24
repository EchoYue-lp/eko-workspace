# F-SUB-02: Subagent execution modes and teams

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Do Sync, Fork, Teammate, Team, manager, timeout, cancellation, and
isolation modes share one lifecycle without detached execution?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/agent/subagent/executor.rs` (3672 lines) — the unified
  dispatch engine. Read in full where relevant:
  `DispatchRequest` (36-101), `subagent_status_from_error` (138-147),
  `TeammateHandle` / `BackgroundSubagentHandle` (256-298),
  `SubagentExecutorConfig` (304-354), `SubagentExecutor` (359-401),
  `dispatch` routing + retry/delegation loop (407-781),
  `clone_for_spawn` (784-800), `dispatch_background` (836-868),
  `dispatch_teammate` (871-980), `dispatch_team` (992-1113),
  `compile_invocation` (1117-1145), `execute_agent_streaming`
  (1148-1453), `isolated_dispatch_agent` (1455-1466), `dispatch_sync`
  (1469-1551), `dispatch_fork` (1554-1888).
- `echo-agent/src/agent/subagent/team/mod.rs` (827 lines) — `Team`,
  `TeamMember`, `TeamAgent`, `TeamAgentBuilder`, `TeamExecutionResult`,
  and the four-strategy `execute_inner` dispatch (300-463). Read in full.
- `echo-agent/src/agent/subagent/team/manager_subagent.rs` (739 lines) —
  `ManagerSubagentOrchestrator` plan → fan-out → synthesize with
  checkpoint/resume (the production Team-mode path). Read in full.
- `echo-agent/src/agent/subagent/team/coordinator.rs` (347 lines) —
  `TeamCoordinator` task-distribution + reassignment logic. Read in full.
- `echo-agent/src/agent/subagent/team/runner.rs` (178 lines) —
  `TeamRunner` parallel fan-out with per-member timeout. Read in full.
- `echo-agent/src/agent/subagent/team/strategy.rs` (60 lines) —
  `TeamStrategy` enum (ManagerSubagent / Pipeline / Debate / Swarm). Read
  in full.
- `echo-agent/src/agent/subagent/team/agent_box.rs` (132 lines) —
  `ArcAgentBox` adapter. Read in full.
- `echo-agent/src/agent/subagent/team/mailbox.rs` (224 lines) — `Mailbox`,
  `MailboxMessage`, `MessageKind`. Read in full.
- `echo-agent/src/agent/subagent/team/message.rs` (68 lines) — skim
    (re-export only).
- `echo-agent/src/agent/subagent/isolated.rs` (81 lines) — legacy
  `run_isolated`. Read in full.
- `echo-agent/src/agent/subagent/worktree.rs` (169 lines) — Sprint 8
  worktree-isolation factory trait + handle. Read in full.
- `echo-agent/src/agent/subagent/workspace.rs` (173 lines) — Sprint 10
  data-workspace isolation factory trait + handle. Read in full.
- `echo-agent/src/tools/builtin/agent_dispatch.rs` (581 lines) —
  `AgentDispatchTool`, `ParentContextFactory`, `child_cancel_token`,
  `dispatch_with_context`. Read in full.
- `echo-agent/echo-core/src/agent/mod.rs:570-665` — `Agent` trait
  defaults for `execute_stream_with_invocation_context` and the
  `execute` signature (no cancel parameter).

Cross-checks: the `subagent::*`, `agent_dispatch::*`, and `team::*` test
suites (133 tests, all passing under `--features subagent`); the
F-SUB-01 report for the definition/registry/catalog invariants; the
F-RCT-04 report for the tool-batch cancellation-grace pattern that the
executor's per-mode select loops mirror.

## Out Of Scope

Deferred to named task IDs:

- The `SubagentEvent` lifecycle enum and event-bus delivery semantics →
  F-SUB-01 (already covered) and a future event-delivery task.
- Handoff / topology multi-agent coordination APIs → **F-MAG-01**.
- The full ReAct loop inside a subagent instance (how
  `execute_stream_with_invocation_context` consumes the cancel token
  inside `ReactAgent`) → **F-RCT-02**.
- Application-layer (EKO) team wiring and DomainProfile → the
  application task_runtime task.
- Hook registry behaviour on dispatch (`before_dispatch` /
  `after_dispatch` / `on_failure`) → a hooks-focused task.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  system reminder). Key constraints applied: Subagent-only terminology
  (no Worker), framework-vs-application layering, "first check if it
  already exists," dead-code cleanup (no backward-compat burden),
  prompt-driven-over-state-machine, UTF-8 safety, cross-repository
  boundary gate.
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-SUB-01.md` (in full).
  Established: single registry, single dispatch tool, single result
  contract (`SubagentOutcome`), single-message return, bounded history
  inheritance. F-SUB-02 owns the execution-mode lifecycle that F-SUB-01
  deferred. F-SUB-01's dead-surface findings (`tool_filter`,
  `compile_system`, `SubagentOutput`, `lightweight`) are assumed still
  present; this task does not re-verify them.
- `docs/comprehensive-review/zcode-glm/tasks/F-RCT-04.md` (in full).
  Established the two-timeout-layer pattern (per-tool ToolManager timeout
  + per-batch `compute_concurrent_tool_batch_timeout`) and the
  5-second-cancel-grace vs immediate-timeout asymmetry. The subagent
  executor's per-mode `select!` loops mirror the cancel-grace idea but
  apply it to whole-dispatch lifecycle, not tool batches. F-RCT-04 also
  confirmed `AgentDispatchTool` is `exempt_from_batch_timeout`
  (agent_dispatch.rs:384), so a subagent dispatch's own timeout governs,
  not the tool-batch timer.

Historical documents treated as hypotheses:

- `executor.rs:987-988` comment on `dispatch_team`: "Timeout: relies on
  `TeamAgent::execute`'s own `tokio::time::timeout` wrapper ... no second
  timeout here (would double-wrap)." Treated as design intent; **code
  evidence shows the team timeout source is disconnected from
  `SubagentDefinition.timeout_secs`** (F-SUB-02-P2-01).
- `team/mod.rs:60-72` comment: `default_timeout_secs: 600` "Aligned with
  `AgentConfig.subagent_timeout_secs` ... single source of truth for all
  subagent dispatch timeouts (Sync/Fork/Teammate + team)." Treated as
  design intent; **code evidence shows team does not actually read
  `AgentConfig.subagent_timeout_secs` at runtime — it hardcodes 600 in
  `TeamConfig::default()` and only overrides via the builder's explicit
  `timeout_secs()`** (F-SUB-02-P2-01).
- `team/mod.rs:107-110` (types.rs): "Only `TeamStrategy::ManagerSubagent`
  is frontmatter-declarable ... `Pipeline`/`Debate`/`Swarm` ... remain
  without production callers." Treated as design intent; **confirmed:
  zero production callers** (F-SUB-02-P3-03).

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `SubagentExecutor` (unified dispatch for four modes), `execute_agent_streaming` (the streaming + cancellation + event-bus path), `WorktreeFactory` / `DataWorkspaceFactory` (isolation trait boundaries), and the `CancellationToken::child_token()` propagation pattern are generic agent-delegation machinery any `echo-agent` consumer needs. They correctly live in the `echo-agent` root crate. The `Team` / `TeamAgent` / `ManagerSubagentOrchestrator` multi-agent coordination is also generic (not EKO-specific) and lives correctly at `echo-agent/src/agent/subagent/team/`. |
| EKO product policy | None at this layer. The executor takes framework inputs (`DispatchRequest`, `SubagentRegistry`, `SubagentExecutorConfig`); the application injects concrete factories (worktree, workspace, state store) and compilers. No EKO-specific decision is baked into the execution-mode machinery. The `ManagerSubagent` strategy's plan→fan-out→synthesize prompt is product-neutral. |
| Adapter boundary | `ArcAgentBox` (team/agent_box.rs) is the thin adapter that lets a shared `Arc<dyn Agent>` feed into `TeamAgentBuilder` (which consumes `Box<dyn Agent>`). It delegates the four required `Agent` methods. It does NOT implement `execute_stream_with_invocation_context` — falling back to the trait default that drops the invocation context and cancel token. This is the seam where Team mode loses its lifecycle coupling (F-SUB-02-P2-02). |
| Duplicate search | Searched names: `SubagentExecutor`, `dispatch_sync`, `dispatch_fork`, `dispatch_teammate`, `dispatch_team`, `dispatch_background`, `run_isolated`, `TeamRunner`, `TeamCoordinator`, `fan_out`, `execute_sub_tasks`, `ManagerSubagentOrchestrator`, `ArcAgentBox`, `TeamStrategy::{Pipeline,Debate,Swarm}`. Searched both `echo-agent` and `echo-agent-cli`. Result: one canonical executor and one canonical dispatch entry; **but three dead/duplicate surfaces** — `run_isolated` (legacy isolated dispatch, zero callers), `TeamRunner` (parallel fan-out with per-member timeout, zero production callers), and `TeamCoordinator`'s reassignment logic (stored as a field, never invoked by the orchestrator). See findings. |
| Migration deletion | No deletion proposed in this review. The dead facilities identified here are candidates for deletion or rewiring per the AGENTS.md "code cleanup" rule, but that is a follow-up action, not part of this review task. |

## Current Path

Verified execution-mode call graph at commit `9b0e0fa`:

```text
LLM invokes agent_tool(agent_name, task, mode?, constraints?, background?)
   [agent_dispatch.rs:196-264]
   ├─ child_cancel_token: invocation_cancel.child_token() OR shared_handle.child_token()
   │      [agent_dispatch.rs:164-178]
   ├─ DispatchRequest { cancel, ... }                                 [agent_dispatch.rs:286-299]
   │
   ├─ if background → dispatch_background(req)                        [agent_dispatch.rs:302-325]
   │      tokio::spawn(spawned.dispatch(req))                         [executor.rs:854]
   │      → returns BackgroundSubagentHandle immediately (detached from caller,
   │        but req.cancel still flows into dispatch → mode path → cancellation works)
   │
   └─ else → executor.dispatch(req)                                   [executor.rs:407-781]
          ├─ entry cancel check (parent_cancel.is_cancelled())       [executor.rs:417-423]
          ├─ delegation-depth + retry-count guards                   [executor.rs:427-442]
          ├─ mode = mode_override OR definition.execution_mode       [executor.rs:449-453]
          │
          ├─ Sync   → dispatch_sync(&req)                            [executor.rs:553, 1469]
          │      ├─ execution_cancel = req.cancel.child_token()      [:1486]
          │      ├─ timeout_secs = def.timeout_secs > 0 ? def : cfg  [:1475-1478]
          │      └─ select! { cancel | timeout(execute_agent_streaming) | execute_agent_streaming }
          │           [:1501-1549]  → on timeout: execution_cancel.cancel() + Err(Timeout)
          │
          ├─ Fork   → dispatch_fork(&req)                            [executor.rs:554, 1554]
          │      ├─ semaphore.acquire_owned() (max_concurrent_forks) [:1555-1560]
          │      ├─ execution_cancel = req.cancel.child_token()      [:1671]
          │      ├─ timeout_secs = def.timeout_secs > 0 ? def : cfg  [:1569-1573]
          │      ├─ resolve worktree_factory / data_workspace_factory [:1604-1636]
          │      ├─ tokio::spawn({                                  [:1678]
          │      │     create worktree/workspace (hard-fail on failure)
          │      │     select! { cancel | timeout(execute_agent_streaming) | execute_agent_streaming }
          │      │       [:1772-1828] → on timeout: execution_cancel.cancel() + Err(Timeout)
          │      │     finalize worktree/workspace → append diff/listing
          │      │   }).await                                       [:1884]
          │      └─ result.isolation_observed = Worktree|Workspace|Context [:1830]
          │
          ├─ Teammate → dispatch_teammate(req.clone()).await?.join()  [executor.rs:557-559, 871]
          │      ├─ child_token = req.cancel.child_token()           [:879]
          │      ├─ timeout_secs = def.timeout_secs > 0 ? def : cfg  [:891-895]
          │      └─ tokio::spawn({                                  [:914]
          │           select! { cancel | sleep(timeout) | execute_agent_streaming }
          │             [:920-971] → on timeout: child_token.cancel() + Err(Timeout)
          │         }) → TeammateHandle { join_handle }
          │      then dispatch awaits handle.join()                  [:559]
          │
          └─ Team   → dispatch_team(&req)                            [executor.rs:564, 992]
                 ├─ resolve TeamSpec.manager + subagents by name      [:1007-1069]
                 ├─ TeamAgent::builder()...build()                    [:1031-1070]
                 │      (NO cancel token, NO req.cancel reference)
                 ├─ team_agent.execute_with_usage(&compiled.task_input) [:1078]
                 │      ├─ timeout = team.config.default_timeout_secs.max(60)  [team/mod.rs:345]
                 │      └─ tokio::time::timeout(timeout, execute_inner(task))  [:346]
                 │           └─ ManagerSubagentOrchestrator::run      [manager_subagent.rs:67]
                 │                ├─ Phase 1: plan (manager.execute)  [:169-209]
                 │                ├─ Phase 2: execute_sub_tasks       [:217-336]
                 │                │      for each sub-task:
                 │                │        tokio::spawn(agent.execute(&task))  [:277]
                 │                │          (NO cancel, NO per-subagent timeout)
                 │                │      collect all handles → Vec<(task, Result)>
                 │                │        (NO sibling abort on failure)
                 │                └─ Phase 3: synthesize (manager.execute) [:338-377]
                 │           on timeout: execute_inner dropped → handles dropped
                 │             → spawned tasks DETACHED (no abort)    ★ F-SUB-02-P1-02
                 └─ map result → SubagentResult { status: Completed }  [:1092-1112]
                      (ALWAYS Completed even if all subagents failed)  ★ F-SUB-02-P2-03
```

Key invariants verified by this graph (full evidence in V01-V04):

- **Single routing entry, single retry/delegation loop.** All four modes
  + background enter through `SubagentExecutor::dispatch`
  (executor.rs:407) or its background wrapper. The retry/delegation loop
  (executor.rs:425-780) is the one place that emits
  `DispatchStarted`/`DispatchCompleted`/`DispatchFailed`/`DispatchCancelled`
  and fires the unified `SubagentStart`/`SubagentStop` hooks. No second
  dispatch authority exists.
- **Sync/Fork/Teammate share the `execute_agent_streaming` path.** All
  three derive `execution_cancel = req.cancel.child_token()`, resolve
  `timeout_secs` from the same `def.timeout_secs > cfg.default` expression,
  and race cancel/timeout/execute in a `tokio::select!`. The agent future
  is dropped when any branch wins, so cancellation and timeout both
  terminate the agent cleanly (verified by the three
  `*_timeout_cancels_detached_stream_producer` tests).
- **Background is detached from the caller but not from cancellation.**
  `dispatch_background` spawns `spawned.dispatch(req)` and returns a
  handle; `req.cancel` still flows into the dispatch loop, so a parent
  cancel reaches the background subagent. Lifecycle events fire on the
  bus. This is intentional "fire and observe" semantics, not a leak.
- **Isolation is Fork-only and hard-fails closed.** Worktree
  (`isolate_worktree`) and data-workspace (`isolate_workspace`) isolation
  are resolved only in `dispatch_fork`. If a subagent declares isolation
  but no factory is configured, Fork hard-fails (executor.rs:1609-1619,
  1630-1636) — never silently shares the main tree. Sync/Teammate/Team do
  not support isolation flags today.

Key **violations** of the "one lifecycle without detached execution"
invariant (full evidence in V01-V04):

- **Team mode does not propagate parent cancellation.** `dispatch_team`
  never reads `req.cancel`; the team module has zero `CancellationToken`
  references in production; the orchestrator calls `agent.execute(&task)`
  (no cancel parameter). Cancelling the parent run does not stop a
  running team. (F-SUB-02-P1-01)
- **Team subagent tasks are detached on timeout.** On the aggregate team
  timeout, `execute_inner` is dropped, the `handles: Vec<JoinHandle>` is
  dropped without `abort()`, and the spawned `agent.execute` tasks
  continue running with no observer. (F-SUB-02-P1-02)
- **Team timeout source is disconnected from the per-subagent config.**
  `TeamAgent::execute_with_usage` uses
  `team.config.default_timeout_secs.max(60)`, ignoring
  `SubagentDefinition.timeout_secs` and silently flooring sub-minute
  values to 60. (F-SUB-02-P2-01)
- **Team subagents bypass `execute_agent_streaming`.** The orchestrator
  invokes the plain `Agent::execute` trait method, so team members get no
  streaming events, no invocation context (working_dir, runtime, history,
  disabled_tools), and no cancel token — structurally unlike the other
  three modes. (F-SUB-02-P2-02)

## Findings

### F-SUB-02-P1-01: Team mode does not propagate parent cancellation (`dispatch_team` ignores `req.cancel`)

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/executor.rs:992-1113` —
    `dispatch_team` signature takes `&DispatchRequest` but the body never
    references `req.cancel`. Confirmed via
    `sed -n '992,1113p' executor.rs | grep -n cancel` returning nothing.
  - `echo-agent/src/agent/subagent/team/` — grep for `CancellationToken`
    in production code (excluding `#[cfg(test)]`) returns **zero hits**
    across all eight team files.
  - `echo-agent/src/agent/subagent/team/manager_subagent.rs:277-283` —
    each subagent is spawned via
    `tokio::spawn(async move { agent.execute(&task).await ... })`.
    `Agent::execute(&self, task: &str)` (echo-core/src/agent/mod.rs) has
    **no cancellation parameter**. The spawned task has no way to observe
    a cancel.
  - `echo-agent/src/agent/subagent/team/agent_box.rs:23-45` —
    `ArcAgentBox` implements only `name`, `model_name`, `system_prompt`,
    `token_usage_summary`, `execute`, `execute_stream`. It does NOT
    override `execute_stream_with_invocation_context`, so even if the
    orchestrator called that method, the trait default
    (echo-core/src/agent/mod.rs:575-582) would route to
    `execute_stream_with_cancel` — still requiring a token the orchestrator
    does not hold.
  - Contrast with Sync/Fork/Teammate, all of which derive
    `execution_cancel = req.cancel.child_token()` and race it in a
    `select!` arm (executor.rs:1486, 1671, 879).
- Reachability: any `agent_tool` call with `mode=team` (or a definition
  with `execution_mode: Team`) → `dispatch` → `dispatch_team` →
  `team_agent.execute_with_usage` → `ManagerSubagentOrchestrator::run` →
  `execute_sub_tasks` → `tokio::spawn(agent.execute(...))`. The parent's
  `CancellationToken` is available in `req.cancel` at the `dispatch_team`
  boundary but is never threaded into the team execution.
- Expected invariant: per F-SUB-01's handoff ("the four modes share one
  lifecycle") and the task question ("without detached execution"),
  cancelling the parent run should cancel a Team-mode dispatch the same
  way it cancels Sync/Fork/Teammate.
- Observed behavior: cancelling the parent run has no effect on a running
  team. The manager's plan/synthesis LLM calls and every subagent's
  `execute` call run to completion (or until the aggregate team timeout).
  There is no `select!` arm listening for cancellation anywhere on the
  team path.
- Impact: a user who cancels a long-running team task (e.g. a 5-subagent
  research team with a 600 s timeout) cannot stop it — the LLM budget and
  tool executions continue for up to the full timeout window. For a local
  personal assistant this is a real cost leak (tokens, API calls, tool
  side effects). The other three modes stop within milliseconds of
  cancel; Team mode does not stop at all.
- Root cause: Team mode was added in Sprint 11 as a self-contained
  orchestrator that pre-dates (or was not wired into) the
  `CancellationToken` propagation pattern the other three modes use. The
  `TeamAgent` / `TeamAgentBuilder` API has no cancel-token field, and the
  orchestrator was written against the plain `Agent::execute` signature.
  The `dispatch_team` comment (executor.rs:987-988) discusses timeout
  delegation but not cancellation — cancellation was simply not part of
  the team design.
- Direction: thread `req.cancel` (or a child token) into the team
  execution. Concretely: (1) add a `cancel: Option<CancellationToken>`
  field to `TeamAgentBuilder` / `TeamAgent`; (2) in
  `ManagerSubagentOrchestrator::execute_sub_tasks`, race each
  `agent.execute` against the token (or wrap in a `select!` with
  `cancel.cancelled()`); (3) in `TeamAgent::execute_with_usage`, race the
  `execute_inner` future against the token in addition to the timeout; (4)
  in `dispatch_team`, pass `req.cancel.child_token()` into the builder.
  The `Swarm` strategy (team/mod.rs:414-461) should likewise race its
  spawned tasks. Given that team subagents go through `Agent::execute`
  (which cannot carry a token), the practical approach is to wrap each
  `agent.execute(&task)` future in a `tokio::select!` with the cancel
  token at the orchestrator level, mirroring what `dispatch_teammate`
  does (executor.rs:920-971). Per AGENTS.md, this is a generic
  framework-level fix (not EKO-specific), so it belongs in the executor
  / orchestrator.
- Regression validation: add a `team_dispatch_propagates_cancel` test
  that registers a team with a `CancellationAwareStreamAgent` subagent,
  dispatches Team-mode, cancels the parent token, and asserts the
  subagent's cancellation signal fires within 1 s. Mirror the
  `teammate_timeout_cancels_detached_stream_producer` test shape.
- Validation reports: [V01](../validations/F-SUB-02/V01-01.md),
  [V02](../validations/F-SUB-02/V02-01.md).

### F-SUB-02-P1-02: Team subagent tasks are detached (leaked) on team-level timeout

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/team/mod.rs:343-353` —
    `TeamAgent::execute_with_usage` wraps `execute_inner` in
    `tokio::time::timeout`. On timeout (`.unwrap_or_else(|_| Err(...))`),
    the `execute_inner` future is dropped.
  - `echo-agent/src/agent/subagent/team/manager_subagent.rs:242-284` —
    `execute_sub_tasks` pushes `tokio::spawn(...)` JoinHandles into a
    `Vec`. When `execute_sub_tasks` is dropped (because the outer timeout
    dropped `execute_inner`), the `Vec<JoinHandle>` is dropped. Per tokio
    semantics, dropping a `JoinHandle` **detaches** the task — it
    continues running; only `JoinHandle::abort()` stops it.
  - `echo-agent/src/agent/subagent/team/manager_subagent.rs:286-331` —
    the handle-await loop is the only place JoinHandles are polled. If
    the loop is never reached (timeout dropped the future mid-spawn) or
    interrupted, the unpolled handles detach.
  - Contrast with `dispatch_teammate` (executor.rs:920-948), which on
    timeout calls `child_token.cancel()` — the agent future observes the
    cancel and is dropped by the `select!`. No such cleanup exists for
    team subagents.
- Reachability: any Team-mode dispatch that hits the aggregate timeout
  while one or more subagents are still running. With the default 600 s
  timeout and sub-second LLM calls this is rare in tests, but any real
  team with a slow or hung subagent will trigger it.
- Expected invariant: on any terminal exit (timeout, cancel, error), all
  spawned sub-tasks should either complete, be aborted, or be observed.
  Detached tasks that continue consuming an LLM/tool budget after the
  caller has given up are a resource leak.
- Observed behavior: on team timeout, the spawned `agent.execute` tasks
  continue running in the tokio runtime. They complete their LLM calls,
  run their tools, and then their results are silently discarded (the
  JoinHandle resolution goes nowhere). The parent receives
  `Err("Team execution timed out...")` and has no way to observe or stop
  the lingering tasks.
- Impact: leaked LLM budget and tool side effects. A team of 5 subagents
  where one hangs: the aggregate timeout fires, the parent gets a
  timeout error, but the other 4 (and the hung one) keep running to
  completion — burning tokens and potentially mutating files after the
  user has moved on. This is the "detached execution" the task question
  asks about, and Team mode is the only mode that exhibits it.
- Root cause: `tokio::spawn` was used for concurrency without pairing it
  with a cleanup path on timeout/cancel. The orchestrator was written
  assuming the happy path (all handles awaited in the loop); the
  timeout-drops-the-future case was not handled.
- Direction: hold the `Vec<JoinHandle>` in a scope that can abort them
  on early exit. Concretely, in `execute_sub_tasks`, after the collect
  loop, if the future is cancelled (via `tokio::select!` on cancel, or a
  `CancellationToken`), call `handle.abort()` for each remaining handle.
  Alternatively, restructure `execute_sub_tasks` to race the collect loop
  against the cancel token, and on cancel, abort all handles. If
  F-SUB-02-P1-01 is fixed (cancel token threaded in), this cleanup
  becomes a natural part of the cancel arm. Pair with F-SUB-02-P1-01's
  fix so both timeout and cancel share the same cleanup path.
- Regression validation: add a `team_timeout_aborts_running_subagents`
  test that registers a team with a slow subagent (longer than the team
  timeout), sets a short team timeout, dispatches, and asserts the
  subagent's cancellation signal fires (or its task is no longer alive)
  after the timeout.
- Validation reports: [V01](../validations/F-SUB-02/V01-01.md),
  [V03](../validations/F-SUB-02/V03-01.md).

### F-SUB-02-P2-01: Team timeout source is disconnected from `SubagentDefinition.timeout_secs` and silently floored at `.max(60)`

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/team/mod.rs:345` —
    `let timeout = std::time::Duration::from_secs(self.team.config.default_timeout_secs.max(60));`
    The `.max(60)` floor means any `TeamConfig.default_timeout_secs < 60`
    is silently bumped to 60.
  - `echo-agent/src/agent/subagent/team/mod.rs:59-73` —
    `TeamConfig::default()` hardcodes `default_timeout_secs: 600`. The
    comment claims alignment with `AgentConfig.subagent_timeout_secs`,
    but there is no runtime link — `TeamConfig::default()` is a constant,
    not a read from `AgentConfig`.
  - `echo-agent/src/agent/subagent/team/mod.rs:608-612` — the builder
    only overrides the timeout if the caller explicitly calls
    `.timeout_secs(secs)`. The `dispatch_team` path
    (executor.rs:1031-1070) does NOT call `.timeout_secs()`, so the team
    always gets `TeamConfig::default()` = 600 s.
  - `echo-agent/src/agent/subagent/executor.rs:992-1070` — `dispatch_team`
    never reads `registered.definition.timeout_secs`. Contrast with
    `dispatch_sync` (executor.rs:1475-1478), `dispatch_fork`
    (executor.rs:1569-1573), and `dispatch_teammate`
    (executor.rs:891-895), all of which resolve
    `def.timeout_secs > 0 ? def : cfg.default_timeout_secs`.
- Reachability: every Team-mode dispatch. A subagent definition with
  `timeout_secs: 30` and `execution_mode: Team` will run for 600 s (the
  team default), not 30 s.
- Expected invariant: per the `dispatch_team` comment
  (executor.rs:307-310) and the `SubagentDefinition.timeout_secs` doc
  (types.rs:154-155, "Timeout in seconds (0 = no timeout)"), the
  per-subagent timeout should govern, falling back to the executor
  default. The team timeout should not be a separate hardcoded island.
- Observed behavior: Team timeout is always 600 s (or whatever
  `TeamConfig::default()` says), regardless of the dispatched subagent's
  `timeout_secs` or the executor's `default_timeout_secs`. The `.max(60)`
  floor additionally prevents any sub-minute team timeout even if the
  caller threads one through the builder.
- Impact: a framework consumer who sets
  `SubagentDefinition.timeout_secs = 30` on a team-mode subagent
  reasonably expects a 30 s timeout. They get 600 s. A consumer who
  explicitly builds a `TeamAgent` with `.timeout_secs(10)` gets 60 s.
  Both are silent mismatches. For the local-assistant threat model this
  is a cost/latency issue, not a safety issue, but it breaks the
  "single source of truth" the code comments claim.
- Root cause: the team timeout predates the Sprint 5 unification that
  aligned `TeamConfig.default_timeout_secs` to 600 in principle. The
  alignment was done at the default-value level (hardcode 600) but not
  wired at runtime (no read from `AgentConfig.subagent_timeout_secs`).
  The `.max(60)` floor appears to be a safety guard against
  sub-minute timeouts but is undocumented.
- Direction: (1) In `dispatch_team`, resolve the timeout the same way
  the other modes do: `if registered.definition.timeout_secs > 0 { that }
  else { self.config.default_timeout_secs }`, then thread it into the
  builder via `.timeout_secs(resolved)`. (2) Remove the `.max(60)` floor
  in `execute_with_usage`, or document why a sub-minute team timeout is
  unsafe (and make the floor configurable). (3) Optionally, have the
  executor pass `self.config.default_timeout_secs` into the builder so
  the team default tracks the executor default without a separate
  constant.
- Regression validation: add a test that registers a team-mode subagent
  with `timeout_secs: 1`, dispatches, and asserts the dispatch times out
  in ~1 s (not 600 s). Add a test that `TeamAgentBuilder::timeout_secs(10)`
  yields a 10 s timeout (not 60 s).
- Validation reports: [V03](../validations/F-SUB-02/V03-01.md).

### F-SUB-02-P2-02: Team subagents bypass `execute_agent_streaming` (no events, no isolation, no invocation context, no cancel)

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/team/manager_subagent.rs:277-283` —
    `tokio::spawn(async move { let result = agent.execute(&task).await ... })`.
    The orchestrator calls the plain `Agent::execute` trait method, which
    returns `Result<String>` — no streaming, no cancel, no invocation
    metadata.
  - `echo-agent/src/agent/subagent/team/agent_box.rs:23-45` — `ArcAgentBox`
    implements `execute` and `execute_stream` but NOT
    `execute_stream_with_invocation_context` /
    `execute_stream_message_with_invocation_context`. Even if the
    orchestrator wanted to pass a cancel token, the adapter does not
    expose the method.
  - Contrast with Sync/Fork/Teammate, all of which call
    `Self::execute_agent_streaming` (executor.rs:1148-1453) — the path
    that emits `DispatchTokenDelta` / `DispatchToolStarted` /
    `DispatchToolCompleted` / `DispatchThinkingDelta` / `DispatchLlmUsage`
    events, observes verification/file-access/artifacts, and respects the
    invocation context (working_dir, runtime, disabled_tools).
- Reachability: every Team-mode subagent execution. Team members produce
  no `SubagentEvent` stream — the parent observes only the final
  synthesized string (via `dispatch_team`'s single
  `SubagentResult { output }`).
- Expected invariant: per the task question and F-SUB-01's handoff, all
  four modes should share one lifecycle — including the event stream and
  invocation context. A team subagent is still a subagent; its tool
  calls, token deltas, and thinking should be observable the same way a
  Fork subagent's are.
- Observed behavior: team members run as opaque `execute(task)` calls.
  No `DispatchToolStarted`/`DispatchToolCompleted` events fire for their
  tool calls. No `DispatchTokenDelta` / `DispatchThinkingDelta`. No
  working_dir binding (worktree/workspace isolation is Fork-only). No
  `disabled_tools` enforcement. No `runtime_context` propagation
  (run_id, trace_sink, cancel). The parent's UI sees the team as a single
  black box that eventually returns a string.
- Impact: observability gap (team member tool calls are invisible to the
  frontend/trace), capability gap (team members cannot be isolated,
  cannot have scoped tools, cannot participate in the run's trace sink),
  and consistency gap (team members are second-class subagents relative
  to Sync/Fork/Teammate). For a product targeting TUI/GUI parity (per
  AGENTS.md), a Team card that shows no tool activity while 5 subagents
  are actively running tools is a UX regression.
- Root cause: `ManagerSubagentOrchestrator` was written against the
  simplest `Agent` trait method to get the plan→fan-out→synthesize flow
  working quickly. The `execute_agent_streaming` path (which requires
  `&dyn Agent + Send + Sync`, a cancel token, and an
  `AgentInvocationContext`) was not lifted into the orchestrator. The
  `ArcAgentBox` adapter was added to bridge `Arc<dyn Agent>` →
  `Box<dyn Agent>` but only for the two simplest methods.
- Direction: route team member execution through
  `execute_agent_streaming` (or an equivalent cancel-aware streaming
  path). This requires: (1) giving the orchestrator access to the
  `SubagentRegistry` (to emit events on its event bus) and a cancel
  token; (2) having the orchestrator call
  `execute_stream_with_invocation_context` on each member (which means
  `ArcAgentBox` must implement it, or the team must hold `Arc<dyn Agent>`
  directly and skip the box); (3) building an `AgentInvocationContext`
  per member (with the team's run_id, a per-member child cancel token,
  and the member's working_dir if isolated). This is a larger change
  than the other fixes; it may be staged (first add cancel, then add
  events, then add isolation). Per AGENTS.md layering, this is generic
  framework work — the orchestrator is in `echo-agent`.
- Regression validation: add a test that dispatches a Team-mode
  subagent whose member calls a tool, and asserts a
  `DispatchToolStarted` / `DispatchToolCompleted` event fires on the
  registry event bus. (Today this would fail — confirming the gap.)
- Validation reports: [V01](../validations/F-SUB-02/V01-01.md),
  [V02](../validations/F-SUB-02/V02-01.md).

### F-SUB-02-P2-03: Team partial-failure has no sibling cancellation, no per-subagent timeout, and always reports `Completed`

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/team/manager_subagent.rs:244-284` —
    `execute_sub_tasks` spawns every sub-task unconditionally. There is
    no branch that cancels remaining handles when one result is `Err`.
  - `echo-agent/src/agent/subagent/team/manager_subagent.rs:286-331` —
    the collect loop awaits each handle in order, records Ok or Err, and
    continues. No early exit on failure.
  - `echo-agent/src/agent/subagent/team/manager_subagent.rs:217-284` —
    each spawned task is `agent.execute(&task)` with **no
    `tokio::time::timeout` wrapper**. A single hung subagent blocks the
    collect loop until the aggregate team timeout
    (`TeamAgent::execute_with_usage`, team/mod.rs:346) fires.
  - `echo-agent/src/agent/subagent/team/manager_subagent.rs:338-377` —
    `synthesize` builds a prompt including `Error: {e}` for failed
    sub-tasks and asks the manager to "report failed or blocked
    sub-tasks truthfully." But it returns `Ok(String)` regardless of how
    many sub-tasks failed.
  - `echo-agent/src/agent/subagent/executor.rs:1092-1106` — `dispatch_team`
    maps the `Ok(output)` from `execute_with_usage` to
    `SubagentResult { outcome: SubagentOutcome { status: Completed, .. } }`
    unconditionally. A team where every subagent failed still reports
    `Completed` to the parent.
- Reachability: any Team-mode dispatch where at least one subagent fails
  or hangs. With real LLM-backed subagents, transient failures are
  expected.
- Expected invariant: a multi-agent execution should (a) not let one
  hung member block the whole team indefinitely (per-subagent timeout),
  (b) surface partial failure in the terminal status (not mask it as
  `Completed`), and (c) optionally fail-fast or cancel siblings on
  hard failure. At minimum, the status contract should distinguish
  "all succeeded" from "some failed."
- Observed behavior: all subagents run to completion (or aggregate
  timeout); failures are collected as text; synthesis runs even on total
  failure; `dispatch_team` always returns `status: Completed`. The
  parent LLM sees a `Completed` outcome with a summary that may mention
  errors — but the structured `status` field (which the parent uses to
  decide next steps) says success.
- Impact: the parent agent (and the UI) cannot distinguish a successful
  team execution from a failed one without parsing the free-text
  summary. A team that produced zero usable results still consumes a
  "success" slot in the parent's reasoning. Combined with
  F-SUB-02-P1-02 (detached on timeout), the worst case is: one member
  hangs → aggregate timeout fires → parent gets `Err(Timeout)` → but the
  other members keep running detached and their results are lost.
- Root cause: "collect-all-then-synthesize" is a defensible strategy
  (the synthesis prompt explicitly handles failures), but it was
  implemented without (a) per-subagent timeout, (b) status
  differentiation, or (c) a sibling-cancellation option. The
  `TeamCoordinator` (coordinator.rs) implements reassignment/retry with
  `max_retries` — exactly the kind of failure handling the production
  path lacks — but is not wired in (F-SUB-02-P3-02).
- Direction: (1) Add a per-subagent `tokio::time::timeout` in
  `execute_sub_tasks` (mirror `TeamRunner::fan_out`, runner.rs:73-77,
  which already does this but is dead). (2) Differentiate the terminal
  status: if all sub-tasks succeeded → `Completed`; if any failed but
  synthesis succeeded → consider a `PartialSuccess` status or at least
  set `remaining_work` / `blockers` in the outcome; if synthesis itself
  failed → `Failed`. (3) Optionally, add a fail-fast mode
  (`TeamConfig.allow_reassignment` already exists but is unused) that
  cancels siblings on hard failure. (4) Wire `TeamCoordinator`'s
  retry/reassignment or delete it (F-SUB-02-P3-02). Per AGENTS.md, do
  not add a state-machine gate — keep it prompt/behaviour-driven where
  possible (the synthesis prompt already handles failure reporting).
- Regression validation: add a test that dispatches a team where one
  subagent fails, and asserts (a) the other subagents still complete,
  (b) the terminal status reflects the partial failure (after the fix),
  and (c) the synthesis text mentions the failure.
- Validation reports: [V04](../validations/F-SUB-02/V04-01.md).

### F-SUB-02-P3-01: `isolated.rs::run_isolated` is dead code (zero production callers)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/isolated.rs:47-81` — `run_isolated`
    builds a fresh `ReactAgent` via `ReactAgentBuilder` and runs it with
    a `tokio::time::timeout`. Returns `IsolatedSubAgentResult { output,
    success }`.
  - Whole-repo grep for `run_isolated` / `IsolatedSubAgent` /
    `IsolatedSubAgentConfig` across `echo-agent/src/` and
    `echo-agent-cli/` (excluding the declaration file): **zero
    production callers**. Only the declaration and its own `#[cfg(test)]`
    would match — but isolated.rs has no tests.
  - The live "isolated dispatch" path is `isolated_dispatch_agent`
    (executor.rs:1455-1466) + `execute_agent_streaming`
    (executor.rs:1148-1453), which is what Sync/Fork/Teammate use.
- Reachability: none. `run_isolated` is never invoked.
- Expected invariant: per AGENTS.md "code cleanup: delete over retain,"
  a function with zero callers should be deleted (YAGNI).
- Observed behavior: `run_isolated` and its config/result types occupy
  81 lines of framework API surface with no consumer. A framework reader
  sees two "isolated" concepts (`run_isolated` and
  `isolated_dispatch_agent`) and may assume both are live.
- Impact: API clutter and a false signal that a standalone isolated-mode
  exists. Low severity.
- Root cause: `isolated.rs` is an earlier iteration of isolated
  execution. The dispatch path later settled on
  `isolated_dispatch_agent` (registry-driven, factory-aware) +
  `execute_agent_streaming` (event-emitting, cancel-aware), and
  `run_isolated` was not removed.
- Direction: delete `isolated.rs` entirely (`run_isolated`,
  `IsolatedSubAgentConfig`, `IsolatedSubAgentResult`). Remove the
  `pub mod isolated;` and any re-exports in `mod.rs`. Per AGENTS.md, no
  backward-compat burden. If a standalone isolated run is ever needed,
  it should reuse `isolated_dispatch_agent` + `execute_agent_streaming`.
- Regression validation: `cargo test --workspace --all-features`; grep
  for any remaining `run_isolated` references (should be zero).
- Validation reports: [V01](../validations/F-SUB-02/V01-01.md).

### F-SUB-02-P3-02: `TeamRunner` and `TeamCoordinator` reassignment logic are dead in the production dispatch path

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/team/runner.rs:40-114` —
    `TeamRunner::fan_out` spawns each subagent with a semaphore and a
    per-member `tokio::time::timeout`. This is exactly the per-member
    timeout the production `ManagerSubagentOrchestrator` lacks
    (F-SUB-02-P2-03).
  - Whole-repo grep for `.fan_out(` / `.fan_out_to(`: **zero production
    callers**. Only `test_team_runner_default_timeout_aligned` references
    `TeamRunner`, and it only checks the default timeout field.
  - `echo-agent/src/agent/subagent/team/coordinator.rs:54-238` —
    `TeamCoordinator` with `assign` / `assign_next` / `record_failure`
    (reassignment with `max_retries`) / `record_result` / `is_complete` /
    `has_failures`. This implements failure-driven reassignment — the
    kind of failure handling the production path lacks.
  - Whole-repo grep for `TeamCoordinator` method calls outside
    `coordinator.rs` tests: only the `Team.coordinator` field
    declaration (team/mod.rs:104) and its construction in `Team::new`
    (team/mod.rs:128). The `ManagerSubagentOrchestrator` never calls any
    coordinator method.
- Reachability: none. `TeamRunner::fan_out` and `TeamCoordinator`
  methods are never invoked by `dispatch_team`, `TeamAgent`, or
  `ManagerSubagentOrchestrator`.
- Expected invariant: per AGENTS.md "if you find two systems doing the
  same thing, delete the old one" — or wire the live one to use the
  better facility. Here the dead facility (`TeamRunner`'s per-member
  timeout, `TeamCoordinator`'s reassignment) is arguably better than
  what the live path does, so the choice is "wire or delete."
- Observed behavior: ~525 lines of public team API (`TeamRunner` 178
  lines + `TeamCoordinator` 347 lines) that no live code exercises.
  They implement the timeout/retry behaviour whose absence is filed as
  F-SUB-02-P2-03 — a paradox where the fix already exists as dead code.
- Impact: maintainability. The dead code can drift from the production
  path's assumptions. More importantly, it obscures whether the
  production team path is *supposed* to have per-member timeouts and
  reassignment (a reader sees `TeamRunner` and assumes it's used).
- Root cause: `TeamRunner` and `TeamCoordinator` are an earlier team
  design layer. Sprint 11's `ManagerSubagentOrchestrator` was written as
  a fresh, simpler orchestrator that bypasses both; the older layer was
  not deleted.
- Direction: either (a) wire `ManagerSubagentOrchestrator::execute_sub_tasks`
  to use `TeamRunner::fan_out` (gaining per-member timeouts) and
  `TeamCoordinator` (gaining reassignment), resolving F-SUB-02-P2-03 as
  a side effect; or (b) delete `TeamRunner` and `TeamCoordinator` (and
  the `Team.coordinator` field), accepting that the ManagerSubagent
  strategy is intentionally simple (collect-all, no reassignment).
  Option (a) is higher-value if reassignment is wanted; option (b) is
  cleaner if the simplicity is intentional. Given the AGENTS.md bias
  toward deletion and prompt-driven-over-state-machine, (b) is preferred
  unless a concrete consumer needs reassignment. If deleting, also
  remove `pub use runner::TeamRunner;` (team/mod.rs:16) and the
  `test_team_runner_default_timeout_aligned` / coordinator tests.
- Regression validation: `cargo test --workspace --all-features`. Under
  (a), add tests that exercise reassignment and per-member timeout via
  the production path.
- Validation reports: [V04](../validations/F-SUB-02/V04-01.md).

### F-SUB-02-P3-03: Non-`ManagerSubagent` strategies (`Pipeline`/`Debate`/`Swarm`) have no production callers; `Swarm` silently drops errors

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/types.rs:107-110` — doc: "Only
    `TeamStrategy::ManagerSubagent` is frontmatter-declarable ...
    `Pipeline`/`Debate`/`Swarm` ... remain without production callers."
  - Whole-repo grep for `TeamStrategy::Pipeline` / `::Debate` / `::Swarm`
    construction outside `strategy.rs`: only tests
    (`team_execution_aggregates_member_usage` uses Pipeline).
    `dispatch_team` uses `spec.strategy.clone()` (executor.rs:1037),
    which is whatever the `TeamSpec` carries — and the only
    frontmatter-declarable value is `ManagerSubagent`.
  - `echo-agent/src/agent/subagent/team/mod.rs:414-461` — the `Swarm`
    strategy's collect loop:
    `for h in handles { if let Ok((name, Ok(output))) = h.await { ... } }`.
    Both `JoinError` (panic) and `Err(String)` (agent failure) are
    silently discarded — no `warn!`, no accounting.
- Reachability: none in production. Programmatic-only per the doc, and
  no programmatic caller exists in-repo.
- Expected invariant: either these strategies are live (and correct) or
  they are deleted. Dead strategies that silently drop errors are a
  latent trap if a future caller wires them.
- Observed behavior: three strategy implementations (~100 lines,
  team/mod.rs:367-461) with zero production callers. `Swarm` in
  particular would lose sub-agent failures without trace if activated.
- Impact: low (dead code). The risk is that a future consumer selects
  `Swarm` and gets silent failure-dropping. `Pipeline` and `Debate`
  propagate errors (`map_err`) so they are less dangerous.
- Root cause: the strategies were designed as a menu but only
  `ManagerSubagent` was productised. Per AGENTS.md YAGNI, the others
  should be deleted until a concrete consumer arrives.
- Direction: either delete `Pipeline`/`Debate`/`Swarm` (and their
  `TeamStrategy` variants, keeping only `ManagerSubagent`), or — if kept
  as "framework capability menu" per the AGENTS.md framework-API
  retention rule — at minimum fix `Swarm`'s silent error-drop (log a
  `warn!` for failed/panicked members). Recommend deletion: the
  strategies are not trait-implementations or multi-implementation
  options, they are unused branches of a sum type, and YAGNI applies.
- Regression validation: `cargo test --workspace --all-features`. Update
  the `team_execution_aggregates_member_usage` test if `Pipeline` is
  deleted (it would need to use `ManagerSubagent` instead).
- Validation reports: [V04](../validations/F-SUB-02/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Mode lifecycle matrix + single-authority duplicate search | yes | passed | [V01-01](../validations/F-SUB-02/V01-01.md) |
| V02 | Parent cancellation propagation (Sync/Fork/Teammate work; Team does not) | yes | passed | [V02-01](../validations/F-SUB-02/V02-01.md) |
| V03 | Timeout ownership (3 modes uniform; Team disconnected + detached on timeout) | yes | passed | [V03-01](../validations/F-SUB-02/V03-01.md) |
| V04 | Team partial-failure + cleanup + dead-facility caller search | yes | passed | [V04-01](../validations/F-SUB-02/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — three code comments treated as hypotheses; classifications in the Inputs section) | passed | classified inline (two stale, one current) |

Executed cargo commands (all exit 0):

```text
cargo test --lib -p echo_agent --features subagent -- subagent:: executor dispatch_team dispatch_teammate dispatch_sync dispatch_fork cancel timeout team   (133 passed)
cargo test --lib -p echo_agent --features subagent -- sync_timeout_cancels fork_timeout_cancels teammate_timeout_cancels teammate_handle_cancel invocation_cancel timed_out_dispatch test_dispatch_team   (8 passed)
cargo test --lib -p echo_agent --features subagent -- team dispatch_team manager_subagent run_with_store run_fast_path run_resumes run_resets run_synthesis   (10 passed)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `executor.rs:987-988` — `dispatch_team` "Timeout: relies on `TeamAgent::execute`'s own `tokio::time::timeout` wrapper ... no second timeout here (would double-wrap)" | current (intent) / stale (source) | The "no double-wrap" intent is honoured; but the team timeout source is disconnected from `SubagentDefinition.timeout_secs` (F-SUB-02-P2-01). The comment is accurate about *where* the timeout lives but misleading about *what feeds it*. |
| `team/mod.rs:60-72` — `default_timeout_secs: 600` "single source of truth for all subagent dispatch timeouts (Sync/Fork/Teammate + team)" | stale | The constant is aligned at 600, but there is no runtime link to `AgentConfig.subagent_timeout_secs`; `dispatch_team` does not read it (F-SUB-02-P2-01). |
| `types.rs:107-110` — "Pipeline/Debate/Swarm remain without production callers" | current | V04 confirms zero production callers (F-SUB-02-P3-03). |
| `team/mod.rs:307-315` — `TeamAgent` "wraps Team and ManagerSubagentOrchestrator" | current | V01/V04 confirm `execute_inner` routes ManagerSubagent to the orchestrator. |
| `agent_box.rs:1-8` — "Only the 4 required Agent methods are implemented; the rest fall back to their trait defaults (the team only calls execute)" | current (and load-bearing for F-SUB-02-P2-02) | Confirmed: `ArcAgentBox` implements only `execute`/`execute_stream`; the orchestrator calls `execute`. This is why team members get no cancel/invocation context. |
| `AGENTS.md` — "Only Subagent, no Worker" | current | Zero `Worker`/`worker_` hits in `echo-agent/src/agent/subagent/` (including team/). |
| `F-SUB-01` handoff — "the four modes share one lifecycle" | partially stale | Sync/Fork/Teammate/Background share one lifecycle; **Team mode does not** (F-SUB-02-P1-01, P1-02, P2-02). The F-SUB-01 handoff assumed uniformity; this task refutes it for Team. |
| `F-RCT-04` handoff — "AgentDispatchTool is `exempt_from_batch_timeout`; it has its own per-dispatch timeout" | current | Confirmed: `agent_dispatch.rs:384-386` returns `true`; the per-dispatch timeout is the executor's (executor.rs:1475, 1569, 891). |

## Coverage And Uncertainty

Inspected in full: `executor.rs` (3672 lines — all dispatch paths, the
streaming event loop, retry/delegation loop, background, all four mode
methods, isolation resolution), `team/mod.rs` (827 lines — Team,
TeamAgent, TeamAgentBuilder, all four strategy branches),
`team/manager_subagent.rs` (739 lines — plan/fan-out/synthesize +
checkpoint/resume), `team/coordinator.rs` (347 lines),
`team/runner.rs` (178 lines), `team/strategy.rs` (60 lines),
`team/agent_box.rs` (132 lines), `team/mailbox.rs` (224 lines),
`isolated.rs` (81 lines), `worktree.rs` (169 lines), `workspace.rs`
(173 lines), `agent_dispatch.rs` (581 lines),
`echo-core/src/agent/mod.rs:570-665` (Agent trait defaults).

Inspected partially (relevant slices only):
- `team/message.rs` (68 lines) — skimmed; it is a re-export facade for
  `TeamMessage` / `MailboxMessage`. No lifecycle logic.
- `echo-core/src/agent/mod.rs` beyond the 570-665 slice — read only the
  `execute` / `execute_stream` / `execute_stream_with_invocation_context`
  signatures and defaults. The full `ReactAgent` override of these
  methods (how it consumes the cancel token inside the run loop) is
  F-RCT-02's scope.

Not inspected (out of scope):
- The application-layer team wiring in `echo-agent-cli` (whether EKO
  constructs teams, supplies a state store, or threads
  `AgentConfig.subagent_timeout_secs` into `TeamAgentBuilder`). Only the
  framework-side `dispatch_team` was inspected.
- The Tauri bridge's consumption of team-mode `SubagentEvent`s — since
  team members emit no events (F-SUB-02-P2-02), there is effectively
  nothing to consume today.
- The `RuntimeStateStore` checkpoint/resume internals beyond confirming
  the orchestrator reads/writes nodes keyed by `run_id`. The
  checkpoint/resume logic is orthogonal to the lifecycle gaps filed here.

Environmental constraints:
- All 133 `agent::subagent::*` + `agent_dispatch::*` tests pass under
  `--features subagent`. Worktree state clean (commit `9b0e0fa`).
- The feature matrix beyond `subagent` was not re-run (F-FEAT-01 owns
  it). The team module is feature-gated behind `subagent` and does not
  have its own feature flag.
- No probe was added/removed — all validations are read-only or use
  pre-existing tests.

Uncertain claims:
- Whether "collect-all-then-synthesize" with no fail-fast is an
  *intentional* team semantics or an oversight. The synthesis prompt's
  explicit failure-reporting instruction suggests it may be intentional,
  but the lack of per-subagent timeout and the always-`Completed` status
  are not defensible under any interpretation. F-SUB-02-P2-03 is framed
  as "at minimum, differentiate status and add per-subagent timeout";
  whether to add fail-fast/sibling-cancel is left as a product choice.
- Whether any external (out-of-repo) `echo-agent` consumer constructs
  `TeamAgent` directly with `Pipeline`/`Debate`/`Swarm`. Per AGENTS.md
  framework-API retention, these pub types might have unknown consumers.
  The evidence (zero in-repo callers, `Swarm` silently drops errors)
  supports either deletion or error-handling fixes, not silent retention.

## Handoff

Conclusions downstream tasks may rely on:

1. **Sync/Fork/Teammate/Background share one lifecycle; Team mode does
   not.** The "one lifecycle without detached execution" invariant holds
   for three modes + background and is **violated** by Team mode. Any
   downstream task that assumes Team-mode subagents are cancelled when
   the parent is cancelled, or that their tasks are cleaned up on
   timeout, should be disabused: they are not (F-SUB-02-P1-01,
   F-SUB-02-P1-02).
2. **The `execute_agent_streaming` path is the canonical subagent
   execution path.** Sync/Fork/Teammate use it; Team bypasses it. Any
   task adding observability, isolation, or invocation-context features
   to subagents must ensure Team mode is routed through it too
   (F-SUB-02-P2-02).
3. **Timeout resolution is uniform for 3 modes; Team is the outlier.**
   `def.timeout_secs > 0 ? def : cfg.default_timeout_secs` is the
   pattern. Team ignores it. Any timeout-related work should align Team
   to the same expression (F-SUB-02-P2-01).
4. **Isolation (worktree/workspace) is Fork-only and hard-fails closed.**
   This is correct and intentional. No other mode supports isolation
   today. A task that wants isolated Team members must first route Team
   through `execute_agent_streaming` (F-SUB-02-P2-02).
5. **Three team facilities are dead in production.** `run_isolated`
   (F-SUB-02-P3-01), `TeamRunner` + `TeamCoordinator` (F-SUB-02-P3-02),
   and `Pipeline`/`Debate`/`Swarm` strategies (F-SUB-02-P3-03). A
   cleanup task can delete or rewire them.

Reports they must read:

- This report (F-SUB-02) for the execution-mode lifecycle invariants and
  the Team-mode gaps.
- `tasks/F-SUB-01.md` for the definition/registry/catalog/result
  invariants (this task assumes they hold).
- `tasks/F-RCT-04.md` for the tool-batch timeout/cancellation pattern
  that the per-mode `select!` loops mirror, and for the
  `exempt_from_batch_timeout` contract on `AgentDispatchTool`.
- `validations/F-SUB-02/V01-01.md` through `V04-01.md` for per-claim
  evidence.

Conditions that make this report stale:

- Threading `req.cancel` into `dispatch_team` / `TeamAgent` /
  `ManagerSubagentOrchestrator` — resolves F-SUB-02-P1-01, requires
  re-running V02.
- Adding `handle.abort()` on team timeout/cancel — resolves F-SUB-02-P1-02,
  requires re-running V01/V03.
- Aligning Team timeout to `def.timeout_secs` and removing the `.max(60)`
  floor — resolves F-SUB-02-P2-01, requires re-running V03.
- Routing team member execution through `execute_agent_streaming` —
  resolves F-SUB-02-P2-02 (and partially P2-03), requires re-running
  V01/V02/V04.
- Adding per-subagent timeout + status differentiation to Team — resolves
  F-SUB-02-P2-03, requires re-running V04.
- Deleting `run_isolated` / `TeamRunner` / `TeamCoordinator` / unused
  strategies — resolves F-SUB-02-P3-01/P3-02/P3-03, requires re-running
  V01/V04.

Follow-up task IDs (no implementation in this review):

- **A framework robustness task** should fix F-SUB-02-P1-01 (thread
  cancel into Team) and F-SUB-02-P1-02 (abort detached tasks on
  timeout). These are the highest-value fixes in this report — they
  close the "detached execution" gap that the task question asks about.
  The fixes are coupled (both need a cancel token in the team path and a
  cleanup-on-terminal-exit branch).
- **A team-alignment task** should fix F-SUB-02-P2-01 (timeout source),
  F-SUB-02-P2-02 (route through `execute_agent_streaming`), and
  F-SUB-02-P2-03 (per-subagent timeout + status). These bring Team mode
  to feature parity with the other modes (per AGENTS.md multi-mode
  parity requirement).
- **A cleanup task** should delete or rewire the dead facilities
  (F-SUB-02-P3-01 through P3-03). Per AGENTS.md, deletion is preferred
  unless a concrete consumer exists.
- **F-MAG-01** (handoff/topology) should confirm whether the Team-mode
  lifecycle gaps here interact with any handoff/topology routing (e.g.
  a handoff to a Team-mode subagent that then cannot be cancelled).
