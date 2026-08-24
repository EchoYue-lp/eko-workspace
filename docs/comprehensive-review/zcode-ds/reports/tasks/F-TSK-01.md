# F-TSK-01: Canonical task model and revision tools

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Is `TaskSpec + TaskExecution + TaskStatus` the sole generic dynamic task
model with coherent revisioned `task_create/update/list` semantics?

**Answer: Yes, with five P3-level surface issues; no P0/P1/P2 finding.**

## Scope

- `echo-agent/echo-orchestration/src/tasks/`: `runtime.rs` (TaskSpec/
  TaskExecution/TaskStatus/TaskKind/Task/DagExecutionState), `revisioned.rs`
  (TaskRevisionService/TaskPatchEngine/TaskPlanPatch/RevisionedTaskStore),
  `task_tools.rs` (task_create/task_update/task_list schemas + parse),
  `store.rs`, `task.rs` (ManagedTask), `mod.rs`, `dag.rs`, `manager.rs`
  (cycle-query delegation only).
- `echo-agent/src/tasks.rs` (facade), `echo-agent/src/lib.rs:322-323`
  (exports), `echo-agent/src/agent/react/mod.rs:394-400` and
  `builder.rs:962-964,1170-1183` (registration + single-API test),
  `echo-agent/src/state/mod.rs` (TaskNode — classified).
- `echo-orchestration/src/planning/validator.rs` + `plan_spec.rs`
  (PlanValidator; authoring artifact projection).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/`: `types.rs`
  (TaskPlan/EkoTaskSpec/EkoTaskExecution/PlanTask/TodoItem/TodoStatus),
  `revisioned_adapter.rs`, `register.rs`, `store.rs:96-101`,
  `tool_exposure.rs:69`.

## Out Of Scope

- `RuntimeDagExecutor` runtime semantics (claims, safe points, waves,
  cancellation, stalls) — F-TSK-03.
- EKO file authorities / adapter losslessness / EKO authoring tools —
  A-TSK-01, A-TSK-02.
- DAG readiness-frontier/validator semantics — F-TSK-02.
- `TaskManager` deep behavioral review — classified for reachability and
  delegation only.

## Inputs

- Root `AGENTS.md` (Subagent terminology; single task-relation API rule;
  TaskPlan artifact / TodoItem UI projection; no todo_write/plan_create
  reintroduction; framework-vs-app deletion rules).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (F-TSK-01 card),
  `zcode-ds/README.md`.
- Dependency reports read: `zcode-ds/reports/tasks/F-CORE-01.md` (identity/
  error/event envelope; P3-01 parent_event_id, P3-02 determinism fallback),
  `zcode-ds/reports/tasks/B-REF-01.md` (convergence matrix: plans as
  editable artifacts, event-sourced recovery, typed terminals).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md:383-387`
  (M13 Phase 1/2/4/5), `:103` (EKO tool shell + policy in app layer).

## Layering Decision

- Generic mechanism (framework, correctly placed): `TaskSpec`/
  `TaskExecution`/`TaskStatus` model, `TaskRevisionService` +
  `TaskPatchEngine` patch semantics, `PlanValidator` structural validation,
  `RevisionedTaskStore` CAS boundary, task tools, `PlanSpec` authoring
  artifact — all in `echo-orchestration`.
- EKO product policy (application): `EkoTaskToolPolicy`, `EkoRevisionedTaskStore`,
  `TaskRuntimeStore`, `TaskPlan`/`PlanRevision`/`PlanTask`/`TodoItem`/
  `TodoStatus` projections, `task_execute` tool — all in
  `echo-agent-app-core/src/tasks/task_runtime/`.
- Adapter boundary: `EkoRevisionedTaskStore` maps `load`/`compare_and_commit`
  only (`revisioned_adapter.rs:26-56`) — no patch/validation/state-machine
  logic; `EkoTaskToolPolicy` supplies schema extensions, scope resolution,
  defaults, capability checks. Conversion into framework types is
  `EkoTaskSpec::to_task_spec` / `TaskUpdateRequest::to_task_plan_patch` /
  `PlanTask::to_task` (A-TSK-01 verifies losslessness).
- Duplicate search terms (both repos, V01): `Task`, `TaskSpec`,
  `TaskExecution`, `TaskStatus`, `TaskKind`, `PlanSpec`, `PlanTaskSpec`,
  `ManagedTask`, `TaskPlan`, `TodoItem`, `TodoStatus`, `EkoTaskSpec`,
  `EkoTaskExecution`, `TaskNode`, `TaskNodeStatus`, `BackgroundTaskStatus`,
  `TaskState`, `task_create`, `task_update`, `task_list`, `task_execute`,
  `todo_write`, `plan_create`, `plan_patch`, `plan_execute`,
  `VisitState`, cycle/topological queries. Result: one authoritative model;
  one tool family; forbidden tool names absent (V01-01).
- Migration deletion check: legacy `TaskManager`+`TaskExecutor`
  (non-revisioned, in-memory) has zero production callers — see P3-01; per
  AGENTS.md framework-public-API rules it is retained as a framework option
  unless the final iteration roadmap deletes it.

## Current Path

Registration and data flow (verified):

1. `ReactAgent::new` (`src/agent/react/mod.rs:394-400`) builds
   `TaskRevisionService` with `InMemoryRevisionedTaskStore` +
   `DefaultTaskToolPolicy::default()` and registers
   `build_task_tools` → `task_create`/`task_update`/`task_list`
   (`task_tools.rs:205-211`).
2. `ReactAgentBuilder::build` (`builder.rs:962-964`) re-registers tools from
   a caller-supplied service via `register_task_tools` (`src/tasks.rs:18-23`);
   name-based registration replaces the default. Test
   `default_agent_uses_one_task_relation_api` (`builder.rs:1170-1183`)
   asserts exactly this family and no `todo_write`.
3. EKO (both GUI and TUI entry points, `register.rs:45-130`): builds the
   service via `build_eko_task_revision_service` (`revisioned_adapter.rs:309-317`)
   with the file-backed `EkoRevisionedTaskStore` and `EkoTaskToolPolicy`,
   registers the same three framework tools plus `task_execute`
   (`task_execute_tool.rs`), `create_complex_task`, `check_run_status`,
   `cancel_run`.
4. Tool execution path: `TaskCreateTool`/`TaskUpdateTool` parse params
   (manual parse + serde `TaskSpecPatch`) → `TaskRevisionService::create_from_tool`
   / `update_from_tool` → `TaskPatchEngine::apply_operations` on the loaded
   snapshot → `PlanValidator::validate_task_snapshot`
   (`revisioned.rs:1007-1011`) → store CAS
   (`compare_and_commit`, expected_revision check) → next revision returned.
5. State owner: revisioned `RuntimePlanSnapshot { revision, tasks }` inside
   the store; `TaskExecution.claim` carries `revision/attempt/spec_hash`
   (`runtime.rs:210-224`) and is invalidated on any Update/Skip/SetStatus.
6. Authoring artifacts project into the model, never the reverse:
   `PlanSpec::to_task_specs` (`plan_spec.rs:359`), `ManagedTask::task_spec`
   (`task.rs:610`); EKO DTOs convert into framework types at the adapter
   (`revisioned_adapter.rs:328,373`).

## Findings

### F-TSK-01-P3-01: Legacy non-revisioned `TaskManager`/`TaskExecutor` task-graph surface is production-unreachable and coexists with the canonical service

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence: `echo-orchestration/src/tasks/manager.rs:13` (TaskManager,
  DashMap store), `:69` (`add_task` no CAS); `tasks/executor.rs:312`
  (TaskExecutor holds `Arc<TaskManager>`), `:1616` ("TaskManager is an
  in-memory authority without revision commits"); all `TaskManager::new`
  constructions are `#[cfg(test)]` (`executor.rs:1882,1918,1955,2037,2085,2112`)
  or README examples (`echo-orchestration/README.md:20-33`);
  `echo-agent/src/lib.rs:322` re-exports `TaskManager`.
- Reachability: definition → pub re-export (`mod.rs:42`, `lib.rs:322`) → no
  production construction anywhere in `echo-agent/src` or `echo-agent-cli`
  (grep V01-01); reachable only by framework consumers that opt in.
- Expected invariant: after M13 convergence, one dynamic task-graph
  authority (revisioned) remains.
- Observed behavior: the legacy in-memory path (its own
  `get_ready_tasks`/`get_next_task`/`wake_dependents` scheduling,
  `TaskStore`/`SqliteTaskStore` persistence) is still compiled, exported,
  and tested, but unreachable from both products.
- Impact: dual task-graph traversal implementations
  (`TaskManager.get_ready_tasks` `manager.rs:241-256` vs
  `DagExecutionState.ready_task_ids` `runtime.rs:438-458`) inflate
  framework surface, test matrix, and reviewer cognitive load; consumers
  could reintroduce the "second authority" anti-pattern AGENTS.md forbids.
- Root cause: M13 replaced the runtime execution authority but did not
  remove the superseded public surface (framework-public-API retention rule
  kept it).
- Direction: candidate for deletion in the final iteration roadmap — remove
  `manager.rs` + `executor.rs` + `store.rs` (`TaskStore`/`SqliteTaskStore`)
  + `events.rs`/`hooks.rs` only if framework-wide usage search stays empty;
  otherwise mark `#[deprecated]` with doc pointer to
  `TaskRevisionService`/`RuntimeDagExecutor`. Deletion requires the AGENTS.md
  framework-deletion gate (grep whole `echo-agent` repo; it is a framework
  option, not a trait-implementation member).
- Regression validation: `cargo test -p echo_orchestration --lib --locked`
  after removal; re-grep `TaskManager`/`TaskExecutor` across the repo.
- Validation reports: [V01](../validations/F-TSK-01/V01-01.md),
  [V05](../validations/F-TSK-01/V05-01.md)

### F-TSK-01-P3-02: `set_status` input spelling "in_progress" diverges from the output/serde spelling "running"

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-orchestration/src/tasks/task_tools.rs:316` (schema enum
  `["pending","in_progress","completed","cancelled"]`), `:504`
  (`"in_progress" => TaskStatus::Running`); `:622-635` (`status_name`
  renders Running as `"running"`); `tasks/runtime.rs:88-90` (serde
  `rename_all = "snake_case"` → `"running"`).
- Reachability: every `task_update` set_status round trip: model sends
  `in_progress`, list output says `running`; serde-persisted statuses say
  `running` (EKO `run-state.json` via `EkoTaskExecution.status`).
- Expected invariant: one canonical spelling per state across schema, wire,
  and serde.
- Observed behavior: two spellings for the same state.
- Impact: model confusion; any consumer matching status strings must know
  both; low functional impact today.
- Root cause: hand-written schema vocabulary chosen before the serde
  vocabulary was canonicalized.
- Direction: rename the schema/parser token to `"running"` (keep
  `"in_progress"` accepted for one release if desired) or extend
  `status_name` to `in_progress`; add a schema-vs-serde unit test.
- Regression validation: test asserting
  `serde_json::to_string(&TaskStatus::Running) == "\"running\""` matches the
  `set_status` enum token.
- Validation reports: [V02](../validations/F-TSK-01/V02-01.md),
  [V03](../validations/F-TSK-01/V03-01.md)

### F-TSK-01-P3-03: Tool schemas are hand-written and parser is more lenient than schema in three places

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence: `task_tools.rs:225-334` (hand-built JSON schemas; no schemars
  anywhere in `echo-orchestration/src/tasks`), `:270-286` (patch schema
  `additionalProperties: false`) vs `revisioned.rs:77-90`
  (`TaskSpecPatch` without `deny_unknown_fields`, parsed at
  `task_tools.rs:486-489`); `max_retries` schema `maximum: 10`
  (`task_tools.rs:284`) not clamped at parse (`:544-548`) — rejected later
  by `PlanValidator` (`validator.rs:243-248`) inside `finalize_and_validate`
  (`revisioned.rs:1007-1011`); `execution_mode` unknown values silently
  default to Parallel (`task_tools.rs:419-425`).
- Reachability: any LLM-generated payload hitting `task_create`/
  `task_update`.
- Expected invariant: schema (what the model sees) and parse path (what is
  enforced) agree; violations reported at parse time.
- Observed behavior: unknown `patch` fields silently dropped; `max_retries`
  >10 surfaces as a commit-time `InvalidPatch` with a generic message;
  invalid `execution_mode` silently becomes Parallel.
- Impact: model-visible schema drift risk; error messages surface one
  revision later than ideal; unknown patch fields could mask typos.
- Root cause: hand-written schemas evolved separately from the serde types
  and the parser.
- Direction: derive schemas from the serde types (schemars) or add a
  conformance test comparing `parameters()` against a
  `serde_json::from_value::<TaskSpecPatch>` round trip; clamp/validate
  `max_retries` and `execution_mode` at parse.
- Regression validation: golden-fixture tests feeding schema-conformant and
  schema-violating payloads through both tools.
- Validation reports: [V03](../validations/F-TSK-01/V03-01.md)

### F-TSK-01-P3-04: `TaskNode`/`TaskNodeStatus` checkpoint state machine overlaps the task-model naming

- Priority: P3
- Confidence: low
- Layer: framework
- Evidence: `echo-agent/src/state/mod.rs:26,57` (TaskNodeStatus/TaskNode,
  "DAG node state machine for long-running tasks"); live callers
  `src/agent/snapshot.rs:723-791` (`create_execution_node`,
  `update_node_status`, `hydrate_running_nodes`), injection points
  `src/agent/config.rs:441`, `builder.rs:636`.
- Reachability: definition → snapshot subsystem (live) → only when an
  application supplies a `RuntimeStateStore` (none of the reviewed products
  construct one in the dynamic-plan path).
- Expected invariant: one dynamic task model (V01); naming should not imply
  a second.
- Observed behavior: a per-turn checkpoint DAG-node state machine with its
  own status enum (Pending/Running/Success/Failed/Blocked/Hydrated) and its
  own store trait exists under the "TaskNode/DAG" name; it has no tools,
  revisions, or scheduling and never touches the revisioned model.
- Impact: naming/cognitive overlap only; reviewers could mistake it for a
  parallel task model (AGENTS.md anti-pattern).
- Root cause: historical checkpoint design kept its original vocabulary.
- Direction: no code change required; document the distinction in
  `state/mod.rs` module doc ("checkpoint-hydration nodes, not the dynamic
  task model") or rename to `CheckpointNode` if touched.
- Regression validation: n/a (documentation).
- Validation reports: [V01](../validations/F-TSK-01/V01-01.md)

### F-TSK-01-P3-05: Legacy rich-record helpers bypass the `TaskStatus` transition machine

- Priority: P3
- Confidence: low
- Layer: framework
- Evidence: `tasks/task.rs:862-874` (`mark_started` sets Running,
  `mark_completed` sets Completed directly); `tasks/manager.rs:136-155`
  (`claim_pending_task` assigns Running directly, guarded by a Pending
  pre-check), `:457-465` (`resume_run` assigns Pending directly from
  Paused); contrast: `manager.rs:89` and `revisioned.rs:614-620` use
  `transition_to`.
- Reachability: only the production-unreachable legacy path (P3-01) plus
  framework consumers of `ManagedTask`/`TaskManager`; the revisioned tool
  path always validates.
- Expected invariant: all status writes go through
  `TaskStatus::can_transition_to`.
- Observed behavior: guarded direct assignment on the legacy path; also
  `SetStatus` on an equal status still clears `claim` and records a
  progress effect (`revisioned.rs:614-622`).
- Impact: none on the reviewed products; future consumers of the legacy
  helpers could bypass terminal-state locking.
- Root cause: pre-revision-era code predating the transition machine.
- Direction: fold into the P3-01 deletion decision; if kept, route the four
  assignments through `transition_to` and make the no-op `SetStatus` path
  skip claim clearing/effect recording.
- Regression validation: tests asserting transition errors on
  Completed→Running and that a no-op set_status preserves `claim`.
- Validation reports: [V02](../validations/F-TSK-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (model, tools, projections) | yes | passed | [V01-01](../validations/F-TSK-01/V01-01.md) |
| V02 | Transition/revision table + stale rejection paths | yes | passed | [V02-01](../validations/F-TSK-01/V02-01.md) |
| V03 | Tool schema ↔ parse/serde round-trip | yes | passed | [V03-01](../validations/F-TSK-01/V03-01.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked tasks` | yes | passed (exit 0, 110 ok) | [V04-01](../validations/F-TSK-01/V04-01.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked revisioned` | yes | passed (exit 0, 3 ok incl. stale test) | [V04-02](../validations/F-TSK-01/V04-02.md) |
| V04 | `cargo test -p echo_agent --lib --locked tasks` (card command) | yes | passed (exit 0, 0 matched) | [V04-03](../validations/F-TSK-01/V04-03.md) |
| V04 | `cargo test -p echo_agent --lib --locked task` (real coverage) | yes | passed (exit 0, 4 ok incl. single-API test) | [V04-04](../validations/F-TSK-01/V04-04.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked runtime` (adjacent) | optional | passed (exit 0, 23 ok) | [V04-05](../validations/F-TSK-01/V04-05.md) |
| V05 | MASTER-PLAN drift (cycle-query delegation, DFS deletion) | conditional | passed | [V05-01](../validations/F-TSK-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN M13 Phase 2: "现有 framework `PlanValidator` 成为 revisioned runtime 结构校验唯一权威，EKO 重复的 dependency/DFS validator 已删除" | current | `revisioned.rs:1007-1011`; EKO `store.rs:96-101` (`#[cfg(test)]`, delegates to framework); no EKO cycle DFS (V01/V05) |
| MASTER-PLAN M13 Phase 4: "`TaskManager` cycle query 收归 `PlanValidator` 的 canonical dependency analysis，并删除 manager-local DFS/`VisitState`" | current (with minor note: `get_dependency_chain_recursive` remains as chain query) | `dag.rs:7-24`; `manager.rs:380-403`; `validator.rs:348-407` (sole VisitState); [V05-01] |
| MASTER-PLAN M13 Phase 5: single task-relation API `task_create/update/list/execute`; old `plan_create/plan_patch/plan_execute` and EKO-visible `todo_write` deleted; `TaskPlan` only artifact; `TodoItem` only UI projection | current | `register.rs:45-130`; `tool_exposure.rs:69`; builder.rs:1181 assertion; zero forbidden-tool definitions (V01); `types.rs:835,1323` docs (V01) |
| MASTER-PLAN: EKO tool shell/policy in application layer, no second ID/status/execution loop | current | `revisioned_adapter.rs:26-56` thin store adapter; framework owns patch/validation (V01/V02) |
| F-CORE-01 P3-01 (parent_event_id never populated) | current (independent, no interaction with task model) | F-CORE-01 V02/V03; task `execution_id` uses `run_id:task_id:revision:attempt` (`runtime.rs:221-223`) |
| B-REF-01 convergence matrix: plan as editable artifact, permission-gated approval | current (framework side: plan = revisioned artifact; approval is product policy) | `TaskGraphContext` + revisioned snapshots (V02); EKO approval is A-TSK/A-HITL scope |

## Coverage And Uncertainty

- `RuntimeDagExecutor` internals not reviewed (F-TSK-03); `scheduler.rs`
  (TaskScheduler/ConflictDetector) and `background_task.rs` not part of this
  model question (separate mechanisms).
- EKO `TaskRuntimeStore::compare_and_commit_revisioned_task_graph` CAS
  correctness not verified (A-TSK-01); only the adapter mapping is evidence
  here.
- No golden JSON fixtures exercise every `task_update` operation shape
  through the real `Tool` interface (only the unit tests in V04-01/02);
  LLM-shaped payload conformance remains an open question for Q-phase tests.
- `TaskManager`/`TaskExecutor` reachability was judged by repository grep;
  external consumers of the framework could still use it (hence P3, not
  deletion recommendation without the framework gate).
- `echo-agent-cli` code was inspected only at the adapter/registration
  boundary per task scope; EKO plan-authoring flows (planner/review) are
  A-TSK-02.

## Handoff

- Downstream tasks may rely on: the model is singular and coherent
  (V01/V02); stale updates are rejected at service + store layers
  (V04-02); tool schemas agree with parse path with three leniency gaps
  (V03); TaskManager cycle queries delegate to PlanValidator (V05);
  registration is name-based and replaceable (V04-04).
- Reports to read: all five validation reports above; F-CORE-01 for
  identity/error contracts; B-REF-01 matrix for artifact-plan constraint.
- Stale conditions: this report becomes stale if `runtime.rs` model types,
  `revisioned.rs` patch/CAS semantics, `task_tools.rs` schemas, or the
  tool registration points change.
- Follow-up task IDs: F-TSK-02 (validator/readiness — P3-03 max_retries
  interplay), F-TSK-03 (claim invalidation semantics of P3-05's no-op
  SetStatus), A-TSK-01 (store CAS), A-TSK-02 (authoring tools), S-RDM-01
  (P3-01 legacy surface deletion decision).
