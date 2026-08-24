# A-TSK-02: EKO task authoring tools

> Status: complete
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean; review-doc changes only

## Question

Are `task_create/update/list` thin product shells over the one revisioned graph without independent Todo/Plan CRUD or hidden global state?

## Scope

- Framework task tool schemas/parsers, registration, revision service, patch engine, validator, and store contract.
- EKO revisioned adapter, task capability policy, product metadata, registration, background service, GUI update command, planner, tool exposure, and task-local context.
- Definition/registration/reachability, schema/field parity, bootstrap ordering, capability-policy parity, forbidden CRUD, tests, and history.

## Out Of Scope

- File/event/projection durability: A-TSK-01.
- DAG execution controller and scheduling loops: A-TSK-03.
- Claim/retry/recovery terminality: A-TSK-04.
- Cross-surface renderer behavior: A-SRF and X-TSK tasks.
- Framework parser permissiveness and framework second TaskManager authority: F-TSK-01.
- Source fixes or dynamic execution.

## Inputs

- Read root AGENTS.md, review README/REPORTING/TASKS, report templates, and current source.
- Read completed Codex dependency reports A-TSK-01 and F-TSK-01 for ownership and deduplication.
- Historical `MASTER-PLAN.md` claims were treated as hypotheses and classified against current source.
- No other reviewer's directory was read.

## Layering Decision

| Classification | Current answer |
|---|---|
| Generic mechanism | Task CRUD schemas, immutable/mutable task split, patch/CAS, structural DAG validation, and revision semantics correctly remain in `echo-agent`. |
| EKO product policy | Run bootstrap, domain/parallel/sort metadata, live Subagent/tool catalog, UI/file projection, background source metadata, and `task_execute` remain in `echo-agent-cli`. |
| Adapter boundary | Field conversion is thin and lossless, but lifecycle bootstrap precedes canonical commit and capability policy is not shared consistently across entry points. |
| Duplicate search | Searched tool/type names, CRUD verbs, constructors, registrations, patch/validator calls, Todo/Plan mutations, task-local/global state, and background/GUI/TUI paths across both repositories. |
| Migration deletion | Keep the framework service. Replace EKO's pre-commit side-effect bootstrap with one application transaction and delete snapshot-specific/no-op capability service construction once one live policy provider exists. |

## Current Path

The normal Agent-authored path is:

```text
TUI / GUI / pooled Agent
  -> register_task_tools_on_agent or infra construction
  -> framework TaskCreateTool / TaskUpdateTool / TaskListTool
  -> TaskRevisionService
       -> EkoTaskToolPolicy (scope/defaults/metadata/capabilities)
       -> TaskPatchEngine + PlanValidator
       -> EkoRevisionedTaskStore compare-and-commit
  -> TaskRuntimeStore event + plan/todo projections
```

GUI `update_tasks` converts `TaskUpdateRequest` to the same framework patch and service. Background `submit_dag` also reaches `create_prepared`, but constructs that service with `DefaultTaskToolPolicy`. `create_complex_task`, status, cancel, and `task_execute` are run-level application commands, not parallel PlanTask CRUD. Production searches find no `todo_write`, `plan_create`, `plan_patch`, or `plan_execute` definition/registration ([V01](../validations/A-TSK-02/V01-01.md)).

The field adapter preserves generic scheduling fields and carries EKO-only values in typed metadata ([V03](../validations/A-TSK-02/V03-01.md)). The remaining problems are transaction and policy-boundary defects, not a second graph implementation.

## Findings

### A-TSK-02-P1-01: Rejected initial task creation leaves an orphan Running TaskRun

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/revisioned.rs:713`, `:724`, `:798`, `:811`, `:894`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs:125`, `:176`, `:190`, `:195`, `:200`, `:284`
- Reachability: registered `task_create` -> `TaskRevisionService::create_from_tool` -> EKO `ensure_scope` -> durable create/start/event -> later policy/structural validation and graph CAS.
- Expected invariant: initial run identity and revision-1 graph become visible as one application transaction, or every failed creation leaves neither a run nor a start event.
- Observed behavior: `ensure_scope` creates the run, ignores attachment-binding failure, transitions it to Running, and emits RunStarted before task preparation, live capability validation, DAG validation, or graph commit. The store exposes no rollback/delete-run path. Any later rejection/backend failure leaves a started run with no plan.
- Impact: invalid model output or a storage failure creates ghost running tasks in history/UI/recovery, loses initial attachments silently, and lets subsequent operations observe lifecycle state that never had an accepted graph.
- Root cause: a policy hook intended to ensure scope performs irreversible product lifecycle side effects before the generic transaction reaches its validation/CAS boundary.
- Direction: keep this application-local. Split non-mutating scope/context preparation from commit, preflight the complete candidate first, then atomically publish RunCreated + revision 1 + attachments + initial status/event in the EKO store. Delete the current side-effecting `ensure_scope` bootstrap and warning-only attachment branch after migration; do not add SQLite or a second graph service.
- Regression validation: invalid Subagent/tool/cycle/metadata and injected failures after each persistence step must leave no run/events/projections; successful creation must expose exactly one run and revision 1.
- Validation reports: [V04](../validations/A-TSK-02/V04-01.md), [V07](../validations/A-TSK-02/V07-01.md), [V08](../validations/A-TSK-02/V08-01.md)

### A-TSK-02-P1-02: EKO authoring entry points apply stale, fresh, or no capability policy

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/register.rs:27`, `:45`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs:284`, `:344`, `:375`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/service.rs:307`, `:332`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/task_runtime.rs:399`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/mcp.rs:211`
- Reachability: primary/pooled registration captures a `TaskCapabilityCatalog`; GUI update rebuilds one per command; background DAG commit chooses `DefaultTaskToolPolicy`; skills, MCP, plugins, LSP, and other tool paths mutate the live Agent registry after startup.
- Expected invariant: every EKO task commit validates Subagent role and allowed tools against one current product capability authority.
- Observed behavior: the primary tool service uses a registration-time snapshot, GUI updates use a fresh snapshot, and background `submit_dag` bypasses EKO capability checks entirely because the default policy's validator returns `Ok(())`.
- Impact: after capability load/unload, identical task specs can be rejected, accepted, or committed for a missing Subagent/tool depending on entry point. Background runs can persist an unexecutable graph and fail only after scheduling; primary Agent plans can reject newly available capabilities or retain removed ones.
- Root cause: graph authority converged, but capability admission remained constructor-local snapshots and `commit_eko_task_plan` substituted a generic no-op policy.
- Direction: own one live EKO capability provider or service factory shared by Agent tools, GUI patches, and background DAG commits; it should read a versioned current registry at validation time. Delete `DefaultTaskToolPolicy` use in EKO commits and registration-frozen policy instances after migration. Keep the generic default policy for unrelated framework consumers.
- Regression validation: add/remove skill, MCP, plugin, LSP, and Subagent between revisions; submit equivalent create/update/background inputs and assert the same accept/reject result and no invalid commit.
- Validation reports: [V02](../validations/A-TSK-02/V02-01.md), [V05](../validations/A-TSK-02/V05-01.md), [V07](../validations/A-TSK-02/V07-01.md), [V08](../validations/A-TSK-02/V08-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, duplicate, forbidden CRUD search | yes | passed | [V01](../validations/A-TSK-02/V01-01.md) |
| V02 | Registration and runtime reachability | yes | passed with policy deviation | [V02](../validations/A-TSK-02/V02-01.md) |
| V03 | Schema/field/metadata adapter parity | yes | passed | [V03](../validations/A-TSK-02/V03-01.md) |
| V04 | Initial bootstrap transaction ordering | yes | failed | [V04](../validations/A-TSK-02/V04-01.md) |
| V05 | Cross-entry live capability policy | yes | failed | [V05](../validations/A-TSK-02/V05-01.md) |
| V06 | Hidden global/Todo/Plan authority inspection | yes | passed | [V06](../validations/A-TSK-02/V06-01.md) |
| V07 | Existing test inventory and history drift | yes | passed with gaps | [V07](../validations/A-TSK-02/V07-01.md) |
| V08 | Dynamic fault/capability fixtures | future | not run per review rule | [V08](../validations/A-TSK-02/V08-01.md) |
| V09 | Report/link/executor/source integrity | yes | attempt 1 inconclusive; attempt 2 passed | [A1](../validations/A-TSK-02/V09-01.md), [A2](../validations/A-TSK-02/V09-02.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `MASTER-PLAN.md:102-103` one framework task model with EKO product adapter | current with adapter defects | [V01](../validations/A-TSK-02/V01-01.md), [V03](../validations/A-TSK-02/V03-01.md) |
| `MASTER-PLAN.md:383-386` one executor/validator/status model | current in this authoring scope | [V03](../validations/A-TSK-02/V03-01.md), [V06](../validations/A-TSK-02/V06-01.md) |
| `MASTER-PLAN.md:387` one task_create/update/list/execute relation API, no Todo/Plan lifecycle | current | [V01](../validations/A-TSK-02/V01-01.md), [V02](../validations/A-TSK-02/V02-01.md) |
| Normal bootstrap and task-control rejection tests imply safe initial authoring | incomplete | [V04](../validations/A-TSK-02/V04-01.md), [V07](../validations/A-TSK-02/V07-01.md) |

## Coverage And Uncertainty

This was a pure static review. No Cargo/rustc/test/build/fixture/network command ran. The source order and selected policy types make both findings source-conclusive, but exact filesystem remnants and UI presentation after faults remain for future regression execution. A-TSK-03 owns execution-loop convergence; A-TSK-04 owns claim/recovery semantics. Dynamic capability availability can also depend on invocation-level allowed-tool filtering, which should be covered by A-TOOL/X-SRF rather than expanding this task.

## Handoff

- The application has one production relation CRUD/revision/validator path; downstream work must preserve it.
- Fix P1-01 in EKO transaction policy, not by creating another framework/application graph.
- Fix P1-02 with one live EKO capability provider across create/update/background; retain the generic framework default policy for other consumers.
- A-TSK-03 may rely on V01/V03/V06 for authoring ownership and should not duplicate these findings.
- This report becomes stale if task registration, `TaskRevisionService::create_from_tool`, EKO policy/bootstrap, background `submit_dag`, or dynamic capability registration changes.
