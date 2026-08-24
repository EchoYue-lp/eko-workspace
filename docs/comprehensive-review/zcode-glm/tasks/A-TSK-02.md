# A-TSK-02: EKO task authoring tools

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Are `task_create`/`task_update`/`task_list` thin product shells over the one
revisioned task graph, with no independent Todo/Plan CRUD and no hidden global
task state?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/task_tools.rs`
  (full, 1-1154) — `TaskCapabilityCatalog`, the `tokio::task_local!` run
  context (`CURRENT_RUN_ID` / `CURRENT_CANCEL` / `CURRENT_TRACE_SINK` /
  `CURRENT_DELEGATE_DEPTH` / `CURRENT_UNATTENDED_WRITE_MODE`),
  `with_run_context` / `with_run_id` / `scoped_with_ctx_run_id` /
  `require_run_id`, and the three EKO-owned orchestration tools
  (`CreateComplexTaskTool` / `CheckRunStatusTool` / `CancelRunTool`).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/register.rs`
  (full, 1-183) — `register_task_tools_on_agent` / `bind_task_execute_to_pool`
  / `task_revision_service_for_agent`; the single post-hoc registration entry
  point that swaps the framework's default task tools for EKO-backed ones and
  adds the EKO extensions.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs`
  (full, 1-389) — `build_eko_task_revision_service` (the single
  `TaskRevisionService::new` for the tool path), `apply_eko_task_update` (the
  IPC/Tauri path adapter), `commit_eko_task_plan` (the planner commit path),
  plus the `EkoRevisionedTaskStore` / `EkoTaskToolPolicy` adapter pair.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/task_execute_tool.rs`
  (1-200, 200-470, schema/dispatch path) — `ExecuteTaskTool` name/schema,
  preflight, dispatch to `execute_run`. Read to confirm it is a dispatcher
  that consumes the committed graph, not a CRUD over it.
- `echo-agent-cli/echo-agent-app-core/src/tasks/service.rs` (full, 1-863) —
  `BackgroundTaskService`: submits/cancels/resumes/retries *runs*; uses
  `commit_eko_task_plan` for DAG submission. Read to confirm it owns run
  lifecycle (not task CRUD) and routes the DAG through the same framework
  service.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/planner.rs`
  (full, 1-336) — file-ownership analysis only (no plan/task CRUD despite the
  legacy module name).
- `echo-agent-cli/src/tauri/commands/task_runtime.rs` (395-427, IPC
  `update_tasks` command) — confirmed IPC routes through the same framework
  revision service via `apply_eko_task_update`.
- `echo-agent-cli/echo-agent-app-core/src/tool_exposure.rs` (60-85, 295-360)
  — `TASK_TOOLS` constant, mode filtering. Confirms the canonical four task
  tool names visible to the policy/mode layer.
- Framework side for schema parity:
  `echo-agent/echo-orchestration/src/tasks/task_tools.rs` (full, 1-732) —
  `TaskCreateTool` / `TaskUpdateTool` / `TaskListTool`,
  `task_create_schema` / `task_update_schema` / `task_input_schema`,
  `parse_task_create_input` / `parse_task_update_input` / `parse_task_draft`,
  `task_kind_schema` (8 variants), `set_status` gated by
  `allow_manual_progress_updates`.
- Framework side for trait / model contract:
  `echo-agent/echo-orchestration/src/tasks/revisioned.rs` (40-152, 247-340,
  674-895) — `TaskDraft` / `TaskCreateInput` / `TaskSpecPatch` /
  `TaskPlanPatchOp` / `TaskPlanPatchInputOp` / `TaskUpdateInput` /
  `RevisionedTaskStore` trait / `TaskToolPolicy` trait /
  `DefaultTaskToolPolicy.allow_manual_progress_updates() = true` (338-340) /
  `TaskRevisionService.create_from_tool` / `update_from_tool` / `apply_patch`.
  `echo-agent/echo-orchestration/src/tasks/runtime.rs` (1-260) —
  framework `TaskSpec` (13 fields, 179-195) / `TaskExecution` (5 fields,
  228-235) / `TaskStatus` (10 variants, 90-107) / `TaskKind` (8 variants,
  22-39).
- Framework default registration:
  `echo-agent/src/agent/react/mod.rs:386-400` — `ReactAgent::new` installs
  `task_create`/`task_update`/`task_list` backed by
  `InMemoryRevisionedTaskStore` + `DefaultTaskToolPolicy`.
  `echo-agent/src/tasks.rs:14-23` — `register_task_tools` (name-based
  replacement via `add_tools` → `register` → `DashMap::insert`).
  `echo-agent/src/agent/react/builder.rs:1170-1183` — test asserting
  `todo_write` is NOT registered by default.
- Cross-repo duplicate search for `todo_write`, `plan_create`, `plan_patch`,
  `plan_execute`, `static.*TODO`, `TODO_REGISTRY`, `global_todo`, plus
  every tool-name returning `fn name(&self) -> &str` across the
  `echo-agent-cli` tree.

## Out Of Scope

Deferred to downstream tasks:

- **A-TSK-01** (complete) — file authorities and the typed adapter conversion
  (events.jsonl authority, projection round-trip, `Retrying`/`Paused`
  lossiness in the event stream). This task relies on those conclusions and
  does not re-audit the file authority.
- **A-TSK-03** — `task_execute` controller boundary, `RuntimeDagExecutor`
  injection, ready-frontier/retry/cancel ownership. Only the tool's surface
  (name/schema/dispatch shape) is inspected here; the executor internals are
  out of scope.
- **A-TSK-04** — recovery barriers, terminal monotonicity, CAS mechanics.
- **F-TSK-01** (complete) / **F-TSK-02** (complete) — the framework-side
  canonical task model and DAG validator. This task treats them as the
  authoritative contract.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — rule 6 ("task relationship has one authority API";
    framework defaults to `task_create/update/list`; EKO adds `task_execute`;
    `TaskPlan` is a versioned artifact only; `TodoItem` is a UI projection
    with no store/state-machine/executor; the legacy global `todo_write` was
    deleted and must not be reintroduced; `plan_create`/`plan_patch`/
    `plan_execute` parallel CRUD banned); the framework-vs-application
    layering gate; the "first search whether it already exists"
    pre-implementation gate; the TUI/GUI parity rule; the UTF-8 / panic
    safety rules.
  - `docs/comprehensive-review/REPORTING.md`,
    `docs/comprehensive-review/templates/{task-report,validation-report}.md`.
- Dependency task reports read:
  - **F-TSK-01** (complete) — established the framework's canonical
    `TaskRevisionService` as the sole mutator, `RevisionedTaskStore` as the
    sole persistence contract, and the three framework tools
    (`task_create`/`task_update`/`task_list`) as the default. This task
    verifies the EKO application delegates to them.
  - **F-TSK-02** (complete) — established `PlanValidator` as the sole DAG
    validator and `PlanSpec` as a versioned artifact. This task relies on
    that for the "no parallel plan CRUD" claim.
  - **A-TSK-01** (complete) — established the single projection set
    (`TaskPlan`/`PlanTask`/`EkoTaskSpec`/`EkoTaskExecution`/`TodoItem`),
    the single adapter pair (`EkoRevisionedTaskStore`/`EkoTaskToolPolicy`),
    the lossless spec round-trip, and the absence of parallel model/store/
    validator/CRUD at the persistence layer. This task extends that to the
    tool surface.
  - **B-REF-01** (complete) — convergence C1 (plan is artifact, not runtime
    approval state machine); used to assess the run-lifecycle tools.
- Historical documents treated as hypotheses: the module docstrings at
  `task_tools.rs:1-16` ("framework owns `task_create`/`task_update`/
  `task_list`; this module keeps EKO's run-level tools"), `register.rs:1-15`
  ("Registers the revisioned TaskCreate/TaskUpdate/TaskList contract plus
  CreateComplexTask / CheckRunStatus / CancelRun and `task_execute`"), and
  `revisioned_adapter.rs:24-25` ("thin adapter; no patch or validation
  logic") — all verified below.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework owns the three task-relationship tools (`task_create`/`task_update`/`task_list`) and the revision service that backs them (`echo-orchestration::tasks::TaskRevisionService`, F-TSK-01). EKO does not duplicate any of these. V01-01 confirms the application defines zero task-authoring tools of its own; it only adds run-orchestration and dispatch tools around them. |
| EKO product policy | The four EKO-owned tools (`task_execute`, `create_complex_task`, `check_run_status`, `cancel_run`) are product policy: they orchestrate the EKO TaskRun lifecycle (scope bootstrap, attended/unattended gating, background spawn, cancel). They mutate run state, not the task graph; the graph still goes through the framework's `task_create`/`task_update`. The `parallel_group` schema extension and `EkoTaskMetadata` injection are also product policy, layered via `TaskToolPolicy.task_input_schema_extensions` and `prepare_task` (V02-01). |
| Adapter boundary | `build_eko_task_revision_service` and `apply_eko_task_update` are thin: pure type conversion + adapter wiring + product policy hooks. No EKO code re-implements `parse_task_create_input`, `task_create_schema`, `task_update_schema`, or `apply_operations`. The framework tools parse input and apply patches; EKO provides the store/policy that the framework service calls. V02-01. |
| Duplicate search | Searched names (whole `echo-agent-cli` repo): `todo_write`, `plan_create`, `plan_patch`, `plan_execute`, `TodoStore`, `fn create_todo`, `fn update_todo`, `global_todo`, `TODO_REGISTRY`, `static.*TODO`, every `fn name(&self) -> &str` in `tasks/`. Result: ZERO matches for any banned parallel CRUD name; ZERO global todo state; the only `todo_write` reference is the framework's negative test assertion at `echo-agent/src/agent/react/builder.rs:1181`. ONE definition of `to_task_plan_patch` (the legitimate adapter converter, A-TSK-01 V03-01). V04-01. |
| Migration deletion | No migration proposed. No deletion candidate — the tool set is live and singular. |

## Current Path

Verified tool inventory and call graph at commits
`echo-agent` 9b0e0fa / `echo-agent-cli` b3b2e81:

1. **Framework default registration.** `ReactAgent::new`
   (`echo-agent/src/agent/react/mod.rs:386-400`) installs the three
   framework tools — `task_create` / `task_update` / `task_list` — backed by
   an in-process `InMemoryRevisionedTaskStore` + `DefaultTaskToolPolicy`.
   The schema for `task_update` therefore initially includes the
   `set_status` op (gated by `DefaultTaskToolPolicy.allow_manual_progress_updates() == true`,
   `echo-orchestration/src/tasks/revisioned.rs:338-340`). V01-01.

2. **EKO replacement is name-based and atomic.** The TaskRuntimeStore is
   not available at agent construction time (it is built later, see
   `register.rs:1-9`), so EKO registers the task tools post-hoc via
   `register_task_tools_on_agent` (`register.rs:45-130`). That function
   builds the EKO revision service
   (`build_eko_task_revision_service`, `revisioned_adapter.rs:309-317`)
   and calls the framework's `echo_agent::tasks::register_task_tools(agent, service)`
   (`register.rs:65`). The framework function calls
   `agent.add_tools(build_task_tools(service))`
   (`echo-agent/src/tasks.rs:18-23`), which calls
   `ToolManager::register_tools` → `DashMap::insert(tool.name().to_string(), tool)`
   (`echo-execution/src/tools.rs:534-539`). Because insertion is keyed by
   `tool.name()`, the three EKO-backed tools atomically replace the three
   default in-memory ones — no name collision, no parallel API. V01-01, V02-01.

3. **EKO-owned orchestration tools (additive, not CRUD).** After the
   framework swap, `register_task_tools_on_agent` adds four EKO-owned
   tools (`register.rs:69-73, 88-94`):
   - `create_complex_task` (`task_tools.rs:797-1034`) — bootstraps a new
     background/foreground Run (UUID, `store.create_run`,
     `store.transition_run(Running)`, spawn `drive_run_async`). It writes
     *run* state only; the PlanTask graph inside the run is created by the
     spawned agent via the framework `task_create` tool.
   - `check_run_status` (`task_tools.rs:1039-1097`) — reads
     `store.get_run`; pure read.
   - `cancel_run` (`task_tools.rs:1100-1154`) — calls
     `store.request_cancel`; cancels the run, not the task graph.
   - `task_execute` (`task_execute_tool.rs:134-483`) — schema is
     `{revision: integer ≥ 1}` only; reads `store.get_plan`, dispatches
     `execute_run`, no plan mutation (only the run-status `Failed`
     transition on preflight rejection at line 306).
   None of the four is a task/plan/todo CRUD. V01-01.

4. **`task_create` call path (framework).** `TaskCreateTool::execute_with_context`
   (`echo-orchestration/src/tasks/task_tools.rs:42-79`) parses input
   locally (`parse_task_create_input`, lines 402-439) and forwards to
   `service.create_from_tool(input, context)` (line 56).
   `TaskRevisionService::create_from_tool`
   (`echo-orchestration/src/tasks/revisioned.rs:713-823`) resolves scope
   via `EkoTaskToolPolicy::resolve_scope`
   (`revisioned_adapter.rs:114-123`), bootstraps a Run via
   `ensure_scope` (`revisioned_adapter.rs:125-212`) when none exists,
   prepares each draft via `prepare_task`
   (`revisioned_adapter.rs:214-243`, which injects `EkoTaskMetadata` +
   domain-default agent role), validates the candidate via
   `validate_candidate` (`revisioned_adapter.rs:284-295`, which delegates
   to `TaskCapabilityCatalog::validate_task_spec`), and commits via
   `EkoRevisionedTaskStore::compare_and_commit` →
   `TaskRuntimeStore::compare_and_commit_revisioned_task_graph`. The
   framework owns the validation/CAS/revision bump; EKO owns the
   product policy. V02-01.

5. **`task_update` call path (framework).** `TaskUpdateTool::execute_with_context`
   (`echo-orchestration/src/tasks/task_tools.rs:109-141`) parses input
   (`parse_task_update_input`, lines 441-525) and forwards to
   `service.update_from_tool(input, context)` (line 123).
   `TaskRevisionService::update_from_tool`
   (`echo-orchestration/src/tasks/revisioned.rs:825-892`) normalizes the
   input ops to canonical `TaskPlanPatchOp` (insert draft goes through
   `prepare_task` again) and applies the patch via
   `apply_patch_to_loaded` → `EkoRevisionedTaskStore::compare_and_commit`.
   The `set_status` op is gated by `service.allow_manual_progress_updates()`
   (task_tools.rs:309, 497). `EkoTaskToolPolicy` does NOT override
   `allow_manual_progress_updates`, so it inherits the trait default of
   `false` (`echo-orchestration/src/tasks/revisioned.rs:275-277`), and the
   EKO `task_update` schema therefore OMITS `set_status` (V03-01). This
   matches EKO's own `TaskUpdateOperation` enum, which also has 4 variants
   (insert/update/skip/reorder) and no `SetStatus`
   (`echo-agent-app-core/src/tasks/task_runtime/types.rs:1257-1272`). V02-01, V03-01.

6. **`task_list` call path (framework).** `TaskListTool::execute_with_context`
   (`echo-orchestration/src/tasks/task_tools.rs:166-203`) is read-only:
   `service.resolve_scope(context)` → `service.load(&scope_id)` →
   format tasks as `[status] id — title` text. The load goes through
   `EkoRevisionedTaskStore::load` →
   `TaskRuntimeStore::load_revisioned_task_graph`. No direct store access
   from the tool; everything is via the framework service. V02-01.

7. **IPC `update_tasks` path (Tauri).** The Tauri command `update_tasks`
   (`echo-agent-cli/src/tauri/commands/task_runtime.rs:399-427`) is the
   GUI's manual plan-edit entry point. It builds the same EKO revision
   service (`task_revision_service_for_agent`, `register.rs:27-41`), then
   calls `apply_eko_task_update`
   (`revisioned_adapter.rs:321-339`), which converts `TaskUpdateRequest`
   to framework `TaskPlanPatch` via `to_task_plan_patch`
   (`types.rs:1284-1316`) and calls `service.apply_patch`
   (`echo-orchestration/src/tasks/revisioned.rs:920-…`). The same
   framework authority handles both the agent-tool path and the IPC path —
   one mutator, two surfaces. V02-01, V03-01.

8. **`commit_eko_task_plan` (initial planner commit).**
   `BackgroundTaskService::submit_dag` (`service.rs:307-357) calls
   `commit_eko_task_plan` (`revisioned_adapter.rs:344-388`) which converts
   each `PlanTask` to a framework `Task` (`PlanTask::to_task`) and calls
   `TaskRevisionService::create_prepared`. This path uses
   `DefaultTaskToolPolicy` instead of `EkoTaskToolPolicy` (recorded as
   A-TSK-01-P3-01; not re-raised here). V02-01.

9. **Run-lifecycle service is not a task CRUD.** `BackgroundTaskService`
   (`service.rs:162-643`) owns run submission/cancel/pause/resume/retry
   for `background:*` conversation runs. It writes *run* state
   (`create_run`, `transition_run`, `request_cancel`, `request_pause`,
   `resume_task_run`, `retry_blocked_task`) and submits DAGs via
   `commit_eko_task_plan` + `execute_run`. It never writes the task graph
   directly. The unified-list helpers (`list_unified`, `get_unified`,
   `get_progress`) are read-only derivations over `TaskRuntimeStore`
   projections. V01-01, V02-01.

## Findings

The headline result is positive: the framework's three task-relationship
tools (`task_create`/`task_update`/`task_list`) are the sole task-authoring
surface, EKO replaces them in-place by name (no parallel API), and EKO's
own tools/dispatcher/service operate strictly on run lifecycle, never on
the task graph directly. AGENTS.md rule 6 holds at the tool layer (V01/V02
positive, V04 clean). The single recorded finding is a P3 robustness note
on the run-state fallback inside the orchestration tool; the rest of the
section documents the verified invariants.

### A-TSK-02-P3-01: `create_complex_task` falls back to a synthetic conversation id and empty root message id when no chat resources are active, weakening traceability for foreground/background runs created outside a chat turn

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/task_tools.rs:951-968`
    — `let run_id = uuid::Uuid::new_v4().to_string();` followed by
    `let conv = res.conv_id.clone().unwrap_or_else(|| format!("message:{run_id}"));`
    and `store.create_run(&run_id, "default", &conv, &res.root_message_id, …)`.
    When `res.conv_id` is `None` (chat resources exist but carry no
    conversation id — e.g. boot-triggered or programmatic runs),
    `conversation_id` becomes `message:{run_id}`, and `root_message_id`
    is whatever `res.root_message_id` holds (which may be empty).
  - `echo-agent-cli/echo-agent-app-core/src/tasks/service.rs:273-305`
    — for comparison, `BackgroundTaskService::submit_prompt_run` deliberately
    uses `background:{source}:{uuid}` and forwards `""` as the message id
    for truly headless runs; that path is internally consistent because
    `list_unified` filters on the `background:` prefix.
- Reachability: any `create_complex_task` invocation where
  `current_chat_resources()` returns `Some` but `conv_id` is `None`. The
  tool's own guard (`task_tools.rs:843-850`) rejects calls with no chat
  resources at all, so the synthetic id surfaces only for the partial case.
- Expected invariant: a Run created by `create_complex_task` should carry
  the conversation/message identity of the chat turn that spawned it, or
  the synthetic fallback should be uniquely identifiable and consistent
  with downstream filters (the way `BackgroundTaskService` uses the
  `background:` prefix).
- Observed behavior: the `message:{run_id}` fallback is unique per run but
  does NOT match the `background:` prefix that `BackgroundTaskService::
  list_unified` / `get_unified` filter on (`service.rs:541, 552, 563, 574`).
  A run created via `create_complex_task` from a partial chat context is
  therefore unreachable from the background-listing APIs (intentional for
  foreground runs, but surprising for the background default).
- Impact: low. In practice `create_complex_task` is called from inside a
  chat turn that has a real `conv_id`, so the fallback is rarely hit. When
  it is hit, the run is still readable via `get_run(run_id)` and surfaced
  via `check_run_status` / `cancel_run`; only the background-listing API
  omits it.
- Root cause: the conversation-id fallback was written for uniqueness, not
  for filter-prefixed consistency with the headless background path.
- Direction: either (a) document that `create_complex_task` requires a
  chat turn with a real `conv_id` and reject the partial case at the same
  gate that already rejects no-resources, or (b) make the fallback prefix
  explicit (e.g. `complex:{run_id}`) and decide whether
  `BackgroundTaskService::list_unified` should include it. Either way, add
  a test asserting the run remains reachable from at least one listing API.
- Regression validation: a test that calls `create_complex_task` with
  `chat_resources` whose `conv_id` is `None`, then asserts
  `BackgroundTaskService::list_unified` / `get_unified` visibility (or
  documents the omission) and that `check_run_status` still works.
- Validation reports: [V02-01](../validations/A-TSK-02/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Registered task tool inventory (framework defaults + EKO replacement + EKO extensions; no parallel CRUD) | yes | passed | [V01-01](../validations/A-TSK-02/V01-01.md) |
| V02 | Create/update/list call paths are thin wrappers over the framework `TaskRevisionService` (no direct store writes from the tool itself) | yes | passed | [V02-01](../validations/A-TSK-02/V02-01.md) |
| V03 | EKO task schemas match framework `TaskSpec`/`TaskExecution`/`TaskPlanPatchOp` field-by-field | yes | passed | [V03-01](../validations/A-TSK-02/V03-01.md) |
| V04 | Forbidden parallel CRUD (`todo_write`/`plan_create`/`plan_patch`/`plan_execute`) and global todo state are absent | yes | passed | [V04-01](../validations/A-TSK-02/V04-01.md) |
| V05 | Historical-document drift | conditional | not-applicable | — |

V05 is not applicable: there is no prior A-TSK-02 report. The four module
docstrings (`task_tools.rs:1-16`, `register.rs:1-15`,
`revisioned_adapter.rs:1, 24-25`, `task_execute_tool.rs:1-19`) make
falsifiable claims that are classified inline in the Historical Claim
Status table.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `task_tools.rs:1-7` "The framework owns `task_create`, `task_update`, and `task_list`. This module keeps EKO's run-level tools…" | current | Confirmed by V01-01: the three framework tools are registered by the framework's `register_task_tools`; the EKO module defines only `CreateComplexTaskTool`/`CheckRunStatusTool`/`CancelRunTool` (+ the run-context task_local helpers). |
| `register.rs:10-12` "Registers the revisioned TaskCreate/TaskUpdate/TaskList contract plus CreateComplexTask / CheckRunStatus / CancelRun and `task_execute`" | current | Confirmed by V01-01: the registration order and tool set match exactly. |
| `register.rs:13-15` "TUI/GUI functional parity … both entry points call this" | current | Confirmed: `src/main.rs:177` (TUI) and `src/tauri/desktop.rs:201` (GUI) both call `register_task_tools_on_agent`. AGENTS.md TUI/GUI parity holds. |
| `revisioned_adapter.rs:1, 24-25` "Thin EKO adapters … deliberately has no patch or validation logic" | current | Confirmed by V02-01/V03-01: the adapter only converts types, injects metadata, and routes to `TaskRevisionService`; no patch/DAG/ready-frontier authority. (Carry-over from A-TSK-01; the tool-surface read agrees.) |
| `task_execute_tool.rs:1-19` "`task_execute` submits one committed task-graph revision to the framework runtime DAG executor" | current | Confirmed by V02-01: the tool's schema is `{revision: integer}`; it dispatches `execute_run` and does not mutate the plan. |
| AGENTS.md rule 6: framework default is `task_create/task_update/task_list`; EKO adds `task_execute` | current (corroborated) | V01-01: those exact four tool names are the only task-relationship tools registered; the `TASK_TOOLS` constant at `tool_exposure.rs:69` is precisely those four. |
| AGENTS.md rule 6: "不得重新引入 `plan_create/plan_patch/plan_execute` 或其它平行任务 CRUD" | current (corroborated) | V04-01: zero matches for all three names anywhere in `echo-agent-cli`. |
| AGENTS.md rule 6: "旧的进程全局 `todo_write` 已由框架直接删除" | current (corroborated) | V04-01: zero `todo_write` matches in `echo-agent-cli`; the only repository-wide hit is the framework's negative test assertion at `echo-agent/src/agent/react/builder.rs:1181`. |
| F-TSK-01 handoff: "A-TSK-* may layer EKO product policy on top of this framework model, not beside it" | current (supported) | The EKO tool layer adds product policy (run orchestration, capability validation, domain defaults) on top of the framework revision service; it does not introduce a parallel task-relationship API. V01-01, V02-01. |
| A-TSK-01 handoff: "one projection set + one adapter pair; no parallel model/store/validator/CRUD" | current (extended) | A-TSK-01 established it at the persistence layer; this task extends the conclusion to the tool layer (V01-01, V04-01). |

## Coverage And Uncertainty

- **Tool-internal parsing logic not traced line-by-line.** V02 confirms
  `TaskCreateTool` / `TaskUpdateTool` / `TaskListTool` forward to the
  service and that `parse_task_create_input` / `parse_task_update_input`
  live in the framework, but the per-field normalization inside the
  parsers (default `max_retries = 3`, trim/filter rules, `execution_mode`
  default) was not re-enumerated. F-TSK-01 V03-01 covers the framework
  side.
- **`task_execute` executor internals out of scope.** The dispatch shape
  (schema, revision-match guard, preflight, `execute_run` await) is
  verified; what `execute_run` does inside (subagent scheduling,
  semaphores, retry loop) is owned by A-TSK-03. The conclusion "task_execute
  is a dispatcher, not a CRUD" rests on the tool's read/write footprint
  (only `get_plan`, `get_run`, `transition_run(Failed)` on preflight
  rejection, `register_run_cancellation`, `note`), not on the executor
  internals.
- **No executable tests run.** All four validations are static inspection
  + grep. The single finding (P3-01) is a robustness concern whose
  regression validation proposes a targeted test.
- **Subagent-pool path (`bind_task_execute_to_pool`) inspected but not
  exercised at runtime.** Confirmed it adds the same `ExecuteTaskTool`
  with a `Weak<AgentPool>` for conversation-scoped resolution; the tool's
  name/schema/dispatch are unchanged. Not re-listed as a finding.
- **`planner.rs` module name is misleading.** The file no longer contains
  plan generation (despite `mod.rs:17-18` docstring claiming
  "structured plan generation via a JSON-mode LLM call"); it now only
  hosts `FileOwnership` / `analyze_file_ownership`. This is a stale
  docstring, not a layering defect — noted here so a future reader does
  not waste time looking for plan-generation code in this module.
- **Environmental limits:** none. The repository is clean at the audited
  commits.

## Handoff

- **Conclusions downstream tasks may rely on:**
  - The framework's `task_create` / `task_update` / `task_list` are the
    sole task-authoring tools registered on an EKO agent. EKO replaces
    the framework's default in-memory backing in-place (name-based
    `DashMap::insert`) and adds four orchestration tools; no parallel
    task/plan/todo CRUD tool exists (V01-01, V04-01).
  - All three framework tools are thin wrappers: they parse JSON params
    in the framework and forward to `TaskRevisionService.create_from_tool`
    / `update_from_tool` / `load`. No EKO code re-implements parsing,
    validation, or patch application (V02-01).
  - The IPC Tauri command `update_tasks` and the agent-tool `task_update`
    share the same `TaskRevisionService` authority (via
    `apply_eko_task_update`); one mutator, two surfaces (V02-01).
  - EKO schemas match the framework's `TaskSpec`/`TaskExecution`/
    `TaskPlanPatchOp` field-by-field; the EKO-only `domain_profile` /
    `parallel_group` / `sort_order` ride through `TaskSpec.metadata` as
    `EkoTaskMetadata` JSON. The `set_status` op is deliberately disabled
    on the EKO path (`EkoTaskToolPolicy` does not override
    `allow_manual_progress_updates`, inheriting the trait default
    `false`), matching EKO's own `TaskUpdateOperation` enum (V03-01).
- **Reports downstream tasks must read:**
  - [V01-01](../validations/A-TSK-02/V01-01.md) for the full tool
    inventory and registration order.
  - [V02-01](../validations/A-TSK-02/V02-01.md) for the call-path table
    and the thin-wrapper proof.
  - [V03-01](../validations/A-TSK-02/V03-01.md) for the field-by-field
    schema parity table.
  - [V04-01](../validations/A-TSK-02/V04-01.md) for the forbidden-CRUD
    grep result.
- **Task-to-reference mapping:**
  - A-TSK-03 (executor boundary) → may rely on `task_execute` being a
    dispatcher whose only store writes are run-level (Failed on preflight,
    cancellation registration). Must not introduce a second task-graph
    mutator inside the executor.
  - A-TSK-04 (recovery/revisions) → may rely on all task-graph mutations
    routing through `TaskRevisionService.apply_patch` /
    `create_from_tool` / `create_prepared`; the CAS revision contract is
    the framework's.
- **Conditions that make this report stale:**
  - Any commit that adds a new task-relationship tool name (beyond
    `task_create`/`task_update`/`task_list`/`task_execute`) invalidates
    V01-01.
  - Any commit that lets an EKO tool write to `TaskRuntimeStore`'s plan
    mutators directly (bypassing `TaskRevisionService`) invalidates
    V02-01.
  - Any commit that reintroduces `todo_write`, `plan_create`,
    `plan_patch`, or `plan_execute` as a registered tool invalidates
    V04-01 and the primary conclusion.
  - Any change to `EkoTaskToolPolicy.allow_manual_progress_updates`
    (or the trait default) invalidates the `set_status` parity claim in
    V03-01.
- **Follow-up task IDs (no fixes implemented in this review):**
  - P3-01 (`create_complex_task` conversation-id fallback) is a localized
    product-policy cleanup; the fix direction is proposed above. Not
    blocking.
  - The stale `planner.rs` / `mod.rs:17-18` docstring is a doc-only
    cleanup; safe to do in any maintenance commit.
