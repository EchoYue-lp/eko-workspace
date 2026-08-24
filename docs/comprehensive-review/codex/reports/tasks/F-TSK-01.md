# F-TSK-01: Canonical task model and revision tools

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: `echo-agent` clean; unrelated pre-existing/parallel `echo-agent-cli/web-frontend/src/generated/*.ts` modifications; reports are outside both source repositories

## Question

Is `TaskSpec + TaskExecution + TaskStatus` the sole generic dynamic task model with coherent revisioned `task_create/update/list` semantics?

## Scope

Framework task model, revision service/store, patch engine, public tools/schema/parser, default Agent registration, `ManagedTask` projection, `TaskManager`, `TaskStore`, root exports, executable stale/CAS/schema/pause fixtures, panic/UTF-8/integer safety, and historical convergence claims.

## Out Of Scope

- DAG dependency classification and propagation: `F-TSK-02`.
- Runtime claim/dispatch/retry/cancel execution policy: `F-TSK-03`.
- Generic tool-schema enforcement ownership: `F-EXT-01`.
- EKO adapter/store/execution-tool correctness and mode parity: `A-TSK-02`, `X-TSK-01`.
- Workspace Cargo gates, withheld under the assigned disk/build-lock constraint and left for primary evidence.

## Inputs

- `AGENTS.md`, `docs/comprehensive-review/{README.md,REPORTING.md,TASKS.md}`, `docs/comprehensive-review/codex/README.md`.
- Dependency reports read: Codex `F-CORE-01`, `F-REL-01`, `B-REF-01` only.
- `docs/MASTER-PLAN.md` and git history were treated as hypotheses, never as proof.

## Layering Decision

`TaskSpec`, `TaskExecution`, `TaskStatus`, structural validation, CAS/revision semantics, and product-neutral task CRUD are generic framework mechanisms. EKO's file authority, UI projections, approvals, worktrees, and `task_execute` policy remain application concerns. A framework/application adapter must preserve every scheduling field without owning another graph loop/state/store authority. Duplicate searches covered Task/TaskSpec/TaskExecution/TaskStatus/ManagedTask/TaskState/TaskPlan/TodoItem/PlanTask/TaskManager/TaskStore/RevisionedTaskStore/TaskPatch plus definitions, exports, constructors, examples, registration, persistence, and runtime callers across both repositories.

## Current Path

Default `ReactAgent::new` constructs one `TaskRevisionService` over `InMemoryRevisionedTaskStore`, then `build_task_tools` registers `task_create`, `task_update`, and `task_list` (`src/agent/react/mod.rs:386-400`). The service prepares tasks, delegates structure to `PlanValidator`, and atomically commits via `RevisionedTaskStore::compare_and_commit` (`revisioned.rs:673-710`, `:900-1012`). Updates precheck the requested revision and the store repeats CAS under its write lock (`:433-463`, `:934-987`). Tool dispatch resolves by name then directly calls `execute_with_context` (`echo-execution/src/tools.rs:618-698`).

A second public path remains: `ManagedTask` stores dependencies/status and projects to canonical `Task` (`task.rs:292-335`, `:606-695`); `TaskManager` owns CRUD, claims, readiness, run lifecycle, and store loading/saving (`manager.rs:67-207`, `:240-355`, `:405-487`); `TaskStore` persists `ManagedTask` (`store.rs:13-44`). `ManagedTaskDagController` reads this authority into a synthetic revision-0 snapshot before the shared `RuntimeDagExecutor` (`executor.rs:1608-1620`). Thus the traversal kernel is shared, but graph/state/store authority is not unique.

## Findings

### F-TSK-01-P1-01: `TaskManager::pause_run` validates but discards every transition

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/manager.rs:444`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/runtime.rs:166`
- Reachability: public root/prelude `TaskManager` export -> public `pause_run` -> live fixture; repository has no internal caller, but this is a reusable framework API rather than CLI-private dead code.
- Expected invariant: pausing a run changes eligible tasks to `Paused`, reports invalid transitions, and keeps state/events/persistence coherent.
- Observed behavior: `transition_to` is pure and returns the target status, but `pause_run` assigns the result to `_`; a Pending Unicode task remains Pending. The method also silently ignores invalid transitions.
- Impact: framework consumers cannot pause a task run through the advertised public lifecycle API; a caller may proceed believing execution is paused.
- Root cause: the caller treats validation-returning state transition as an in-place mutation.
- Direction: route lifecycle changes through one canonical mutation owner, assign/persist the returned status, emit events, and return structured operation results. Delete the direct silent mutation loop after migration.
- Regression validation: Pending/Running/Retrying/Blocked and terminal tasks across pause/resume, asserting state, errors, events, and persisted reload.
- Validation reports: [V07](../validations/F-TSK-01/V07-01.md), [V17](../validations/F-TSK-01/V17-01.md), [V20-03](../validations/F-TSK-01/V20-03.md)

Ownership note: this finding is the public TaskManager model/lifecycle contract. `F-TSK-02` owns DAG dependency-state classification/propagation and should not duplicate this defect.

### F-TSK-01-P1-02: Task tool parser silently commits schema-invalid graph intent

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/task_tools.rs:225`; `:419`; `:544`; `:590`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-execution/src/tools.rs:618`
- Reachability: default Agent registration -> ToolManager normal execution -> TaskCreateTool parser -> revision service commit; normal execution never calls the separate optional parameter validator.
- Expected invariant: values outside advertised `additionalProperties:false`, array/string, enum, and integer bounds are rejected without committing a graph.
- Observed behavior: `depends_on: [17,true,null]` becomes empty, invalid `max_retries` becomes 3, unknown execution mode becomes Parallel, and unknown fields disappear; the tool returns success and commits revision 1.
- Impact: malformed dependency intent can become a different valid DAG and execute out of intended order; retry/mode intent is also changed without visibility.
- Root cause: handwritten permissive extraction is not derived from/enforced by the advertised JSON Schema, and the live tool pipeline bypasses optional validation.
- Direction: establish one strict typed parse/validation boundary derived from the same contract; reject explicitly malformed or unknown values. Decide centralized generic enforcement under `F-EXT-01`, while task tools retain domain validation.
- Regression validation: table-driven create/update inputs for wrong types, mixed arrays, unknown fields, enum/bounds, no-commit assertion, and schema-valid round-trip.
- Validation reports: [V05](../validations/F-TSK-01/V05-01.md), [V13](../validations/F-TSK-01/V13-01.md), [V16](../validations/F-TSK-01/V16-01.md), [V20-02](../validations/F-TSK-01/V20-02.md), [V26-02](../validations/F-TSK-01/V26-02.md)

### F-TSK-01-P1-03: Default framework task creation directs callers to an unavailable tool

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/task_tools.rs:57`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/react/mod.rs:392`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-execution/src/tools.rs:624`
- Reachability: default Agent registers create/update/list -> successful initial create returns `Call task_execute with revision=1` -> default ToolManager has no such name -> `ToolError::NotFound`.
- Expected invariant: standalone framework success output recommends only reachable framework capabilities.
- Observed behavior: framework-wide search finds no `task_execute` definition/registration; it exists only in this response string, while EKO separately owns the real tool.
- Impact: default framework consumers materialize an inert graph and are explicitly sent into an unavailable next step, breaking the advertised workflow.
- Root cause: application-specific execution guidance leaked into a generic framework tool response during CRUD convergence.
- Direction: make the framework response product-neutral (committed revision/next inspectable state). Applications that register execution may inject their own guidance; do not move EKO policy into the framework merely to satisfy the string.
- Regression validation: default and custom Agent inventories must agree with every tool name in returned guidance; EKO separately verifies its injected execution guidance.
- Validation reports: [V19](../validations/F-TSK-01/V19-01.md), [V25](../validations/F-TSK-01/V25-01.md), [V20-04](../validations/F-TSK-01/V20-04.md)

### F-TSK-01-P2-01: Rich task records retain a second graph/state/store authority

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/task.rs:292`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/manager.rs:11`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/store.rs:13`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/executor.rs:1608`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/lib.rs:321`
- Reachability: public exports/prelude and demo construct TaskManager/ManagedTask; TaskExecutor loads manager state at revision 0 and projects to the shared runtime kernel; resume loads the independent TaskStore.
- Expected invariant: rich hook/verifier records may remain framework APIs, but graph relations, mutable lifecycle and persistence commit through the canonical revision service/store.
- Observed behavior: ManagedTask owns dependencies/status, TaskManager owns CRUD/claims/readiness/lifecycle, and TaskStore persists it without revisions. Projection shares the runtime executor and validator but does not remove the second authority.
- Impact: framework consumers can create and mutate equivalent graphs through incompatible revisioned and revision-0 APIs, with different concurrency, lifecycle, and persistence guarantees.
- Root cause: executor/validator convergence stopped at an adapter while the prior public manager/store authority remained operational.
- Direction: preserve generic hooks/verifiers/rich records and reusable Store options, but make them adapters over the revisioned authority. Migrate real callers, then delete ManagedTask/TaskManager graph CRUD/status/dependency ownership and the independent TaskStore transaction semantics. Do not delete framework Store/SQLite options merely because EKO does not use them.
- Regression validation: one graph identity/revision across tool CRUD, rich executor resume, hooks, and store reload; repository search must show no revision-0 mutation authority.
- Validation reports: [V01](../validations/F-TSK-01/V01-01.md), [V18](../validations/F-TSK-01/V18-01.md), [V23](../validations/F-TSK-01/V23-01.md), [V20-05](../validations/F-TSK-01/V20-05.md), [V25-02](../validations/F-TSK-01/V25-02.md)

### F-TSK-01-P3-01: `record_attempt` can wrap the retry counter on narrowing conversion

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/task.rs:877`
- Reachability: public `ManagedTask::record_attempt`; no production repository caller was found, but it remains a reusable public API.
- Expected invariant: integer narrowing is checked/saturating or types remain consistent.
- Observed behavior: `attempts.len() as u32` wraps above `u32::MAX`.
- Impact: only an extreme record count can make retry_count contradict retained attempts; practical risk is low.
- Root cause: unchecked narrowing cast.
- Direction: use checked/saturating conversion or keep the counter as `usize`; if the rich authority is removed, delete this duplicate counter with it.
- Regression validation: explicit conversion-boundary unit test.
- Validation reports: [V21](../validations/F-TSK-01/V21-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Task identity, commits, dirty state, protocol | yes | passed | [V00](../validations/F-TSK-01/V00-01.md) |
| V01 | Definition/duplicate authority search | yes | failed | [V01](../validations/F-TSK-01/V01-01.md) |
| V02 | Registration/runtime reachability | yes | passed | [V02](../validations/F-TSK-01/V02-01.md) |
| V03 | Transition/patch invariant table | yes | passed | [V03](../validations/F-TSK-01/V03-01.md) |
| V04 | Concurrent CAS fixture | yes | passed | [V04](../validations/F-TSK-01/V04-01.md) |
| V05 | Invalid schema round-trip fixture | yes | failed | [V05](../validations/F-TSK-01/V05-01.md) |
| V06 | Pause fixture compile attempts | supporting | mixed | [A1](../validations/F-TSK-01/V06-01.md), [A2](../validations/F-TSK-01/V06-02.md), [A3](../validations/F-TSK-01/V06-03.md) |
| V07 | Pause behavior fixture | yes | failed | [V07](../validations/F-TSK-01/V07-01.md) |
| V08 | Earlier direct test rerun records | supporting | passed | [V08](../validations/F-TSK-01/V08-01.md) |
| V12-V14 | Direct-rustc artifact selection | supporting | mixed | [V12](../validations/F-TSK-01/V12-01.md), [V13](../validations/F-TSK-01/V13-01.md), [V14](../validations/F-TSK-01/V14-01.md) |
| V15-V18 | Existing focused framework tests | yes | passed | [V15](../validations/F-TSK-01/V15-01.md), [V16](../validations/F-TSK-01/V16-01.md), [V17](../validations/F-TSK-01/V17-01.md), [V18](../validations/F-TSK-01/V18-01.md) |
| V19/V25 | Default tool inventory and response contract | yes | mixed | [V19](../validations/F-TSK-01/V19-01.md), [V25](../validations/F-TSK-01/V25-01.md) |
| V21-V22 | Panic/UTF-8/integer and terminology scan | yes | mixed | [V21](../validations/F-TSK-01/V21-01.md), [V22](../validations/F-TSK-01/V22-01.md) |
| V23 | Historical-document drift | yes | failed | [V23](../validations/F-TSK-01/V23-01.md) |
| V24 | Report/task-ID/path/executor integrity | yes | mixed, final passed | [A1](../validations/F-TSK-01/V24-01.md), [A2](../validations/F-TSK-01/V24-02.md), [A3](../validations/F-TSK-01/V24-03.md), [A4](../validations/F-TSK-01/V24-04.md) |
| V25 | Tool-response evidence plus primary collision/inventory attempts | yes | mixed, final passed | [delegated](../validations/F-TSK-01/V25-01.md), [primary inventory](../validations/F-TSK-01/V25-02.md) |
| V26 | Fresh current-source revision and CRUD regressions | yes | passed | [A1](../validations/F-TSK-01/V26-01.md), [A2](../validations/F-TSK-01/V26-02.md) |
| V27 | Primary filename-collision disclosure and final acceptance gate | yes | mixed, final passed | [A1](../validations/F-TSK-01/V27-01.md), [A2](../validations/F-TSK-01/V27-02.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `MASTER-PLAN.md:102` sole dynamic model and no second scheduling/structural validation | regressed in authority, current for executor/validator | [V01](../validations/F-TSK-01/V01-01.md), [V23](../validations/F-TSK-01/V23-01.md) |
| `MASTER-PLAN.md:383-386` one runtime executor, canonical validator/status/model | current in its narrow executor/validator/status claims; regressed in broad final authority claim | [V03](../validations/F-TSK-01/V03-01.md), [V18](../validations/F-TSK-01/V18-01.md), [V23](../validations/F-TSK-01/V23-01.md) |
| `MASTER-PLAN.md:387` framework CRUD plus EKO execute | current ownership, but framework response leaks unavailable EKO action | [V25](../validations/F-TSK-01/V25-01.md) |
| Public `pause_run` behavior | longstanding uncovered defect | [V07](../validations/F-TSK-01/V07-01.md), git blame `8647bb27` |

## Coverage And Uncertainty

No Cargo workspace build, feature matrix, or Clippy gate was run because this is a read-only atomic review rather than a source submission. Delegated behavior fixtures linked current-source rlibs; after disk cleanup and lock coordination, the primary freshly ran the exact stale-CAS and canonical CRUD regressions through Cargo in V26-01/02. Third-party `RevisionedTaskStore` implementations, full runtime scheduling, EKO adapter semantics, and dependency propagation remain assigned downstream. The five findings were independently reconstructed from live source in V20-02..05 and V25-02, so this task is accepted as `complete`.

## Handoff

- The five findings are primary-accepted. Source fixes remain deferred to synthesis and the iteration roadmap.
- `F-TSK-02` owns dependency classification/propagation and must not duplicate P1-01's public pause API defect.
- `F-TSK-03` owns runtime claims/dispatch/retry/cancel; `F-EXT-01` owns generic schema enforcement; `A-TSK-02`/`X-TSK-01` own EKO adapters and mode paths.
- This report becomes stale if task model exports, TaskManager/TaskStore, task schemas/parsers, ToolManager validation, default Agent registration, or reviewed commits change.
- Any convergence fix must delete the displaced mutation/store authority rather than retain two systems.
