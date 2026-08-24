# F-SUB-02: Subagent execution modes and teams

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: source repositories clean at final source inspection; previously disclosed externally owned changes at `echo-agent-cli/web-frontend/src/generated/ApiError.ts` and `StreamingEvent.ts` were not read, modified, or reverted; reports live outside both source repositories

## Question

Do Sync, Fork, Teammate, Team, manager, timeout, cancellation, background,
checkpoint/resume, and isolation form one bounded, observable Subagent lifecycle
without detached execution or false terminal claims?

## Scope

- `echo-agent/src/agent/subagent/executor.rs`: four-mode router, background and
  Teammate handles, timeout/cancellation, events, fresh instances, and Fork
  worktree/workspace lifecycle.
- `echo-agent/src/agent/subagent/team/*`: Team object/config, strategies,
  ManagerSubagent orchestration, checkpoint/resume, runner/coordinator/mailbox,
  shared-Agent adapter, partial results, and usage.
- `echo-agent/src/agent/subagent/{types,events,worktree,workspace}.rs` and
  `echo-agent/echo-core/src/agent/mod.rs`: public contracts and lifecycle
  invariants.
- `echo-agent/src/tools/builtin/agent_dispatch.rs` and narrowly scoped EKO
  loader/registration sites to prove real model-facing and application reachability.
- Static duplicate/field-use/test/panic/UTF-8/overflow searches across both
  repositories, excluding the disclosed generated CLI files.

## Out Of Scope

- Source fixes, builds, Cargo/rustc/tests, or dynamic fixtures.
- Definition/registry/catalog/prompt/attachment/result-adapter defects already
  owned by F-SUB-01.
- Generic ReAct tool-batch correctness (F-RCT-04), which has no Codex report yet.
- Task graph/DAG authority, TaskRuntime scheduling, and application result
  acceptance (F-TSK-03 and application Task tasks).
- EKO role selection/pooling/product prompt policy (A-SUB-01).
- General artifact provenance already owned by F-SUB-01.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md`; Codex reviewer
  protocol and report templates.
- [F-SUB-01](F-SUB-01.md), initially consumed as temporary needs-evidence input
  and observed primary-promoted to `complete` during this review. Its registry,
  catalog, mode-default, attachment, adapter-error, artifact, and definition-field
  findings are not repeated here.
- F-RCT-04 had no Codex report and remains a declared dependency evidence gap.
- Current source and scoped Git metadata. No other reviewer directory/report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Mode routing, queue admission, cancellation/timeout, non-blocking handles, Team member invocation, bounded concurrency, typed partial failure, checkpoint identity/error semantics, and isolation establishment are framework concerns for any consumer. |
| EKO product policy | Which roles form a Team, concrete timeout/concurrency values, worktree retention/merge, data workspace persistence, UI rendering, and TaskRun acceptance remain application policy. |
| Adapter boundary | EKO may parse role frontmatter, inject factories/store, and project events. It must not own a second member scheduler, checkpoint DAG, cancellation authority, or terminal classifier. |
| Duplicate search | Searched mode/handle names, TeamSpec/TeamConfig fields, TeamAgent/TeamRunner/TeamCoordinator/Mailbox, dispatch and background callers, cancellation/timeout/semaphore, checkpoint nodes/store calls, worktree/workspace factories, and tests across both repositories. |
| Migration deletion | Keep one canonical SubagentExecutor lifecycle and thin adapters. Fold Team members through it; remove or clearly demote unused TeamRunner/Coordinator/Mailbox execution authority and inert config fields after the canonical replacement covers their reasonable public use. |

## Current Path

```text
model agent_tool / programmatic caller
  -> DispatchRequest { mode, cancel, runtime identity, background }
  -> background? tokio::spawn(dispatch) + string-only handle
  -> SubagentExecutor::dispatch
       Sync     -> fresh Agent -> invocation-aware streaming -> terminal events
       Fork     -> wait semaphore -> create isolation -> invocation-aware streaming
                   -> finalize isolation -> terminal events
       Teammate -> spawn invocation-aware streaming -> common dispatch immediately join
       Team     -> resolve cached manager/member Arc Agents
                   -> ArcAgentBox -> TeamAgent outer timeout
                   -> raw Agent::execute plan / fan-out / synthesis
                   -> optional best-effort TaskNode checkpoint
                   -> unconditional Completed result on successful synthesis
  -> retry/delegate hook or exactly one outer terminal event
```

The Sync/Fork/Teammate admitted-execution path has substantial positive
invariants: value-scoped runtime identity, child cancellation tokens, fresh
factory instances, structured event parsing, typed cancellation/timeout, and
UTF-8-safe result handling. Fork also hard-fails missing worktree isolation and
preserves the original execution result when finalization fails. Those guarantees
do not extend to queue admission, background result recovery, Team members, or
missing data-workspace isolation.

## Findings

### F-SUB-02-P1-01: Fork queue wait ignores parent cancellation and configured timeout

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/executor.rs:1554`,
  `echo-agent/src/agent/subagent/executor.rs:1569`,
  `echo-agent/src/agent/subagent/executor.rs:1671`,
  `echo-agent/src/agent/subagent/executor.rs:1772`
- Reachability: every Fork call, including background Fork, awaits the shared
  semaphore before entering its execution cancellation/timeout race.
- Expected invariant: parent cancellation and the configured dispatch deadline
  bound queue wait plus execution; a cancelled queued request never starts later.
- Observed behavior: permit acquisition is an unconditional await. The child
  token and timeout are consulted only after admission, so a saturated queue can
  wait indefinitely, ignore cancellation, and later execute stale work.
- Impact: parent/task cancellation is not reliable under load; user-visible
  timeout can substantially exceed configuration and cancelled work may mutate
  files after its owner has terminated.
- Root cause: admission is outside the lifecycle select/deadline and the timeout
  is duration-local rather than an end-to-end deadline.
- Direction: create one child token/deadline before admission and select permit,
  cancellation, and deadline together; pass the remaining deadline into the
  admitted execution and delete the second duration authority.
- Regression validation: saturate every permit, enqueue a Fork, cancel and time
  it out while queued, release permits, and prove no Agent/factory/isolation call
  begins.
- Validation reports: [V03](../validations/F-SUB-02/V03-01.md),
  [V13](../validations/F-SUB-02/V13-01.md)

### F-SUB-02-P1-02: Background dispatch has no recoverable result or control handle

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/executor.rs:273`,
  `echo-agent/src/agent/subagent/executor.rs:830`,
  `echo-agent/src/agent/subagent/events.rs:337`,
  `echo-agent/src/agent/subagent/events.rs:351`,
  `echo-agent/src/agent/subagent/events.rs:370`
- Reachability: `agent_tool` selects `dispatch_background` for explicit
  `background=true` or a background role and returns only execution ID/name.
- Expected invariant: a non-blocking run remains joinable/cancellable/queryable,
  with a durable terminal result even when no receiver is attached or a receiver
  lags/restarts.
- Observed behavior: the spawned join handle and cancellation handle are dropped;
  `BackgroundSubagentHandle` stores only strings. Completion is sent to a bounded
  broadcast bus with no replay/store/query and discarded send errors.
- Impact: callers can receive `started` but permanently lose success/failure and
  cannot stop the run through the returned framework authority.
- Root cause: background was modeled as an event projection, not an owned
  execution record/handle lifecycle.
- Direction: retain execution state in one registry/store keyed by execution ID;
  expose typed cancel/join/query/replay, then let event/UI projections subscribe
  to that authority. Delete the string-only handle and event-only completion
  assumption once migrated.
- Regression validation: no subscriber, delayed subscriber, broadcast lag,
  explicit cancel, parent completion, executor drop, and restart/query.
- Validation reports: [V04](../validations/F-SUB-02/V04-01.md),
  [V13](../validations/F-SUB-02/V13-01.md)

### F-SUB-02-P1-03: Team members bypass the canonical invocation and cancellation lifecycle

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/executor.rs:1020`,
  `echo-agent/src/agent/subagent/executor.rs:1058`,
  `echo-agent/src/agent/subagent/team/agent_box.rs:23`,
  `echo-agent/src/agent/subagent/team/mod.rs:343`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:277`,
  `echo-agent/echo-core/src/agent/mod.rs:464`
- Reachability: live Team dispatch resolves cached registry Agents, wraps them,
  and every strategy calls raw `execute`; ManagerSubagent and Swarm can call
  multiple shared instances concurrently.
- Expected invariant: Team is a mode of the same lifecycle: fresh/serialized
  Agent instance, parent identity, child cancellation, tool/history/budget/
  working-dir constraints, streaming events, and observed result evidence.
- Observed behavior: member calls bypass `execute_agent_streaming` and
  `AgentInvocationContext`. They receive none of those invocation-scoped facts or
  member events and use cached shared instances even though the Agent trait
  explicitly requires callers to serialize concurrent execute calls.
- Impact: Team children can lose run/trace identity and restrictions, continue
  underlying work after outer timeout, race mutable Agent context, and become
  invisible as individual Subagent runs.
- Root cause: TeamAgent directly owns Agent execution rather than planning member
  work through the canonical SubagentExecutor.
- Direction: make Team orchestration produce typed child dispatches through one
  executor/scheduler with derived identity/token/budget/isolation; delete raw
  Agent storage/execution and the four-method Arc adapter from the live Team path.
- Regression validation: invocation-recording members for every strategy,
  concurrent same-role calls, parent cancel/timeout, child tool process cancel,
  per-member events/usage/artifacts, and nested delegation depth.
- Validation reports: [V02](../validations/F-SUB-02/V02-01.md),
  [V06](../validations/F-SUB-02/V06-01.md),
  [V13](../validations/F-SUB-02/V13-01.md)

### F-SUB-02-P1-04: TeamSpec runtime configuration is discarded and fan-out is not generally bounded

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/types.rs:112`,
  `echo-agent/src/agent/subagent/executor.rs:1031`,
  `echo-agent/src/agent/subagent/team/mod.rs:44`,
  `echo-agent/src/agent/subagent/team/mod.rs:594`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:244`
- Reachability: EKO loader builds `TeamSpec.config`; live dispatch builds TeamAgent
  from the spec but copies only manager/member names and strategy.
- Expected invariant: configured concurrency/timeout/coordination policy governs
  the built Team and every parallel strategy.
- Observed behavior: `spec.config` is never applied. Builder defaults win;
  ManagerSubagent spawns every parsed task with no semaphore, reassignment and
  cross-talk flags are unread, and Swarm ignores batch_size.
- Impact: operators cannot enforce declared concurrency/deadlines; malformed or
  noncompliant manager output can create unbounded member starts, and public
  configuration silently lies about runtime behavior.
- Root cause: configuration exists on both TeamSpec and Team builder/runner but
  no complete conversion or single scheduler consumes it.
- Direction: pass one validated TeamConfig into the canonical scheduler, apply a
  nonzero bounded permit count/deadline to all strategies, and reject unsupported
  policy fields. Delete inert fields and parallel timeout/concurrency owners.
- Regression validation: field-by-field config matrix, zero/max values,
  oversized plan, every strategy, and concurrent start counters.
- Validation reports: [V01](../validations/F-SUB-02/V01-01.md),
  [V07](../validations/F-SUB-02/V07-01.md),
  [V13](../validations/F-SUB-02/V13-01.md)

### F-SUB-02-P1-05: Team member failures and panics can be projected as Completed

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/team/manager_subagent.rs:286`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:327`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:348`,
  `echo-agent/src/agent/subagent/team/mod.rs:435`,
  `echo-agent/src/agent/subagent/executor.rs:1092`
- Reachability: every live ManagerSubagent Team passes member errors to a final
  manager call; Swarm filters unsuccessful joins; successful synthesis/reduction
  returns through the unconditional Completed construction.
- Expected invariant: runtime-owned member terminal states survive synthesis;
  partial success remains typed with failed members and remaining work.
- Observed behavior: ordinary errors become prompt text, panicked tasks disappear,
  and Swarm ignores failures. Any successful final Agent call becomes Completed
  with default empty evidence/remaining work.
- Impact: downstream Task/UI logic can accept an incomplete Team result as fully
  successful, losing which work failed and whether retry is required.
- Root cause: Team returns only a String plus aggregate usage; synthesis prose is
  treated as terminal authority.
- Direction: make TeamExecutionResult carry ordered child outcomes and a
  runtime-derived overall status; synthesis may summarize but cannot overwrite
  child facts. Delete String-only partial-failure projection.
- Regression validation: mixed success/error/panic/cancel/timeout per strategy,
  typed member identity, overall status, remaining work, and terminal events.
- Validation reports: [V08](../validations/F-SUB-02/V08-01.md),
  [V13](../validations/F-SUB-02/V13-01.md)

### F-SUB-02-P1-06: Checkpoint store failures are treated as successful recovery/persistence

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/team/manager_subagent.rs:81`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:134`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:268`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:303`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:158`
- Reachability: when EKO injects RuntimeStateStore and a run ID, every live
  ManagerSubagent Team uses these read/write sites.
- Expected invariant: storage failure is explicit and cannot make a run claim
  durable success or silently replay completed side effects.
- Observed behavior: load error becomes an empty checkpoint set and every save
  error is discarded. The Team still returns success.
- Impact: transient store failure can replan/rerun completed member work and its
  filesystem/tool side effects; callers cannot distinguish durable completion
  from an uncheckpointed result.
- Root cause: checkpoint persistence is implemented as best-effort telemetry
  despite being used as recovery authority.
- Direction: define typed fail-closed or explicit non-resumable semantics; surface
  every store error before terminal success and centralize checkpoint commit.
- Regression validation: injected failure at load and each write boundary,
  crash/retry, exactly-once skip evidence, and typed degraded/failure outcome.
- Validation reports: [V09](../validations/F-SUB-02/V09-01.md),
  [V13](../validations/F-SUB-02/V13-01.md)

### F-SUB-02-P1-07: Team checkpoint identity cannot prove task, plan, topology, or member assignment

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/team/manager_subagent.rs:20`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:93`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:114`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:244`,
  `echo-agent/src/agent/subagent/team/mod.rs:102`,
  `echo-agent/src/agent/subagent/team/mod.rs:233`
- Reachability: every resumed ManagerSubagent Team keys synthesis/plan/member nodes
  from run ID and numeric index, then reconstructs member order from a HashMap.
- Expected invariant: cached outputs are reused only for the identical task,
  validated ordered plan, Team revision, and same assigned member.
- Observed behavior: keys/payloads omit task/plan/Team/member identity; synthesis
  fast-path returns any String for the run; malformed plan elements are silently
  dropped; stored idx is ignored; randomized member iteration drives modulo
  assignment.
- Impact: restart/reconfiguration or malformed state can return stale synthesis,
  skip work, bind output to the wrong plan item, or rerun a failed item under a
  different role.
- Root cause: positional checkpoint keys were substituted for content-addressed,
  revisioned child execution identities.
- Direction: persist a validated Team execution manifest containing task hash,
  Team/catalog revision, ordered plan item ID/hash, assigned member generation,
  attempt, and output status; reject partial/malformed mismatches. Use stable
  member order and delete index-only compatibility nodes after migration.
- Regression validation: process reseed, registration reorder, changed task/Team,
  partial malformed plan, wrong idx/member, stale synthesis, and exact replay.
- Validation reports: [V10](../validations/F-SUB-02/V10-01.md),
  [V13](../validations/F-SUB-02/V13-01.md)

### F-SUB-02-P1-08: Missing data-workspace factory silently violates requested isolation

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/types.rs:177`,
  `echo-agent/src/agent/subagent/executor.rs:1620`,
  `echo-agent/src/agent/subagent/executor.rs:1630`,
  `echo-agent/src/agent/subagent/executor.rs:1737`,
  `echo-agent/src/agent/subagent/workspace.rs:10`
- Reachability: any Fork definition with isolate_workspace=true and no injected
  factory takes this path; executor default has no factory.
- Expected invariant: requested disjoint filesystem isolation either exists or
  the dispatch fails before any tool runs, matching worktree behavior.
- Observed behavior: missing WorktreeFactory hard-fails, while missing
  DataWorkspaceFactory only warns and runs in the ordinary context directory.
- Impact: concurrent data Subagents can overwrite one another or the user's
  primary files despite an explicit isolation declaration.
- Root cause: missing workspace capability is treated as optional degradation
  although the definition models it as an execution guarantee.
- Direction: fail before Agent execution when requested factory is absent; keep
  product-specific directory/retention behavior in the application factory.
- Regression validation: absent/failing factory, two identical output names,
  observed isolation, no primary-directory writes, and cleanup/finalize failure.
- Validation reports: [V11](../validations/F-SUB-02/V11-01.md),
  [V13](../validations/F-SUB-02/V13-01.md)

### F-SUB-02-P2-01: Teammate mode is advertised as mailbox-parallel but common dispatch immediately joins it

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/types.rs:28`,
  `echo-agent/src/tools/builtin/agent_dispatch.rs:429`,
  `echo-agent/src/agent/subagent/executor.rs:555`,
  `echo-agent/src/agent/subagent/executor.rs:871`,
  `echo-agent/src/agent/subagent/team/mailbox.rs:84`
- Reachability: model-facing non-background Teammate calls enter common dispatch;
  programmatic callers can separately call the lower-level handle method.
- Expected invariant: a distinct mode should have distinct stable lifecycle
  semantics consistent across the public/router/schema surfaces.
- Observed behavior: common dispatch spawns then immediately joins, and
  TeammateHandle has no mailbox. The only Mailbox belongs to separate Team APIs.
  The independent background flag, not Teammate, controls model-facing
  non-blocking behavior.
- Impact: consumers cannot reason from the mode name/schema about concurrency or
  communication, and may build coordination around a capability that is absent.
- Root cause: spawn mechanics, non-blocking policy, and Team message coordination
  were exposed as overlapping concepts rather than one contract.
- Direction: choose one Teammate contract: either a typed asynchronous child with
  durable handle/message endpoint, or remove the mode and use explicit background
  on Sync/Fork. Update schema/docs and delete the unused branch/API.
- Regression validation: common/tool/programmatic timing, join/cancel, message
  exchange, parent completion, and background combinations.
- Validation reports: [V05](../validations/F-SUB-02/V05-01.md),
  [V13](../validations/F-SUB-02/V13-01.md)

### F-SUB-02-P2-02: Public TeamRunner, Coordinator, and Mailbox are a disconnected second lifecycle authority

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/team/mod.rs:14`,
  `echo-agent/src/agent/subagent/team/mod.rs:93`,
  `echo-agent/src/agent/subagent/team/runner.rs:17`,
  `echo-agent/src/agent/subagent/team/coordinator.rs:42`,
  `echo-agent/src/agent/subagent/team/mailbox.rs:80`,
  `echo-agent/src/agent/subagent/team/manager_subagent.rs:30`
- Reachability: these types are public/re-exported and documented standalone
  framework options, but live dispatch Team uses ManagerSubagentOrchestrator
  directly and does not call Runner, Coordinator, or Mailbox.
- Expected invariant: reasonable public framework options may exist, but one Team
  lifecycle owns assignment, retry, concurrency, timeout, failure, and messaging,
  or boundaries are explicit and non-overlapping.
- Observed behavior: Runner owns another semaphore/timeout/result shape;
  Coordinator owns retry/task states; Mailbox owns message/cancel variants;
  live ManagerSubagent owns different fan-out/checkpoint/failure logic. Config
  names imply integration that does not exist.
- Impact: external consumers choose incompatible semantics, fixes/tests land in
  the wrong subsystem, and advertised reassignment/cross-talk/mailbox behavior
  does not affect live Team dispatch.
- Root cause: earlier Team primitives remained public after a separate
  orchestration path became authoritative.
- Direction: converge on canonical scheduler primitives used by live TeamAgent;
  retain only genuinely composable low-level APIs with explicit contracts and
  delete the disconnected runner/coordinator authority and misleading config.
- Regression validation: one end-to-end Team API exercises assignment,
  concurrency, retry, message, cancellation, checkpoint, and typed terminal
  events; repository-wide search proves the replaced loop is removed.
- Validation reports: [V01](../validations/F-SUB-02/V01-01.md),
  [V02](../validations/F-SUB-02/V02-01.md),
  [V07](../validations/F-SUB-02/V07-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, field-use, and duplicate-authority search | yes | passed | [V01](../validations/F-SUB-02/V01-01.md) |
| V02 | Registration and model/EKO runtime reachability | yes | passed | [V02](../validations/F-SUB-02/V02-01.md) |
| V03 | Fork admission cancellation and timeout | yes | failed invariant | [V03](../validations/F-SUB-02/V03-01.md) |
| V04 | Background ownership, result recovery, and control | yes | failed invariant | [V04](../validations/F-SUB-02/V04-01.md) |
| V05 | Teammate mode semantics and mailbox reachability | yes | failed invariant | [V05](../validations/F-SUB-02/V05-01.md) |
| V06 | Team member invocation/cancel/event lifecycle | yes | failed invariant | [V06](../validations/F-SUB-02/V06-01.md) |
| V07 | Team config, bounded concurrency, and policy fields | yes | failed invariant | [V07](../validations/F-SUB-02/V07-01.md) |
| V08 | Partial failure, panic, and terminal truthfulness | yes | failed invariant | [V08](../validations/F-SUB-02/V08-01.md) |
| V09 | Checkpoint store fault propagation | yes | failed invariant | [V09](../validations/F-SUB-02/V09-01.md) |
| V10 | Checkpoint identity, malformed data, deterministic resume | yes | failed invariant | [V10](../validations/F-SUB-02/V10-01.md) |
| V11 | Fork worktree/workspace isolation contract | yes | failed invariant | [V11](../validations/F-SUB-02/V11-01.md) |
| V12 | Production panic/UTF-8/overflow inspection | yes | passed | [V12](../validations/F-SUB-02/V12-01.md) |
| V13 | Existing-test inventory and future executable matrix | yes | inconclusive | [V13](../validations/F-SUB-02/V13-01.md) |
| V14 | Report/link/anchor/executor/dirty-state integrity | yes | passed | [V14](../validations/F-SUB-02/V14-01.md) |
| V30 | Primary source-anchor sampling and acceptance | yes | passed | [V30](../validations/F-SUB-02/V30-01.md) |

No targeted executable check was applicable because the user explicitly forbade
Cargo, rustc, tests, builds, and dynamic fixtures. V13 records the future matrix
without claiming execution.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-SUB-01: registry/catalog/prompt/result findings are outside mode lifecycle | current | [F-SUB-01](F-SUB-01.md); findings were not duplicated |
| `TeamConfig.default_timeout_secs` is the single timeout source for all modes | regressed | [V07](../validations/F-SUB-02/V07-01.md): live dispatch drops TeamSpec.config and Team member calls have only an outer default timeout |
| ManagerSubagent checkpoint uses deterministic index binding | regressed | [V10](../validations/F-SUB-02/V10-01.md): index is not bound to task/member/revision and member order comes from HashMap |
| Team members communicate via mailbox and config controls cross-talk/reassignment | stale | [V01](../validations/F-SUB-02/V01-01.md), [V07](../validations/F-SUB-02/V07-01.md): these public primitives are not wired to live TeamAgent execution |
| Fork requested worktree isolation never silently falls back | current | [V11](../validations/F-SUB-02/V11-01.md); the analogous data workspace promise is not current |

## Coverage And Uncertainty

- No code was compiled or executed. Existing tests were inspected but not run;
  V13 is intentionally inconclusive and the task remains `needs_evidence` for
  primary static acceptance.
- F-RCT-04 was not available to the delegated reviewer while this report was
  authored. It is now primary-complete; primary V30 confirmed this report does
  not duplicate its generic ReAct batch-execution findings. F-TSK-03 must consume these child lifecycle
  facts rather than treating Team checkpoint nodes as the Task graph authority.
- Public Team strategies Pipeline/Debate/Swarm were inspected as framework APIs;
  only ManagerSubagent is frontmatter-declarable/currently application-reachable.
  Findings distinguish live impact from public API impact.
- Concrete EKO worktree/workspace/store implementations were not audited beyond
  narrow injection/reachability. Their product retention/cleanup correctness
  remains for application tasks.
- Externally owned generated CLI paths were neither read nor modified. Their
  changing dirty state does not affect reviewed Rust anchors.

## Handoff

- Downstream framework work may rely on the positive Sync/Fork/Teammate admitted
  lifecycle and the ten findings above after primary acceptance.
- The iteration order should first converge Team member execution on the canonical
  executor and durable background handle, then make deadlines/permits end-to-end,
  then replace best-effort positional checkpointing with a revisioned manifest,
  and finally remove disconnected Team authorities/config fields.
- F-RCT-04 should own generic tool-batch behavior. F-TSK-03 should own the one
  TaskRun/PlanTask graph and consume typed child outcomes; it must not grow a
  second Team scheduler to compensate for these defects.
- This report becomes stale if mode routing, Agent invocation methods, Team
  builder/config, RuntimeStateStore checkpoint schema, event bus/handle APIs, or
  isolation factory behavior changes.
- Primary reviewer must sample source anchors and recompute the finding/
  validation and configuration/checkpoint claims before changing status to
  complete. V14 confirms this subagent handoff is mechanically complete.
