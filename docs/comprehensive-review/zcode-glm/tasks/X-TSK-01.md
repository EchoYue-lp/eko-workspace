# X-TSK-01: Task graph and adapter conformance

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Is there one revisioned TaskRun graph with lossless EKO projection and no
second validator/executor/store authority?

## Scope

Primary source paths and behaviors inspected:

- Framework task model and revision authority (echo-agent):
  - `echo-agent/echo-orchestration/src/tasks/runtime.rs:1-260` — `TaskKind`
    (8 variants, 22-39), `TaskStatus` (10 variants, 90-107) with the
    `can_transition_to` table (128-164), `TaskSpec` (13 fields, 179-195) +
    `stable_hash` (199-207), `TaskClaim` (211-216) + `execution_id`
    (221-223), `TaskExecution` (5 fields, 228-235), `Task` (251-254).
  - `echo-agent/echo-orchestration/src/tasks/revisioned.rs:1-1014` — the
    whole revision authority: `RevisionedTaskGraph` (42-45),
    `TaskDraft` (49-63), `TaskCreateInput` (67-74), `TaskSpecPatch`
    (78-90), `TaskPlanPatchOp` (94-113), `TaskPlanPatchInputOp`
    (125-144), `TaskGraphCommit` (167-172), `RevisionedTaskStore` trait
    (247-258), `TaskToolPolicy` trait (270-314), `DefaultTaskToolPolicy`
    (318-410), `InMemoryRevisionedTaskStore` (414-464), `TaskPatchEngine`
    (474-628), `TaskRevisionService` (674-1014) — including
    `create_from_tool`, `update_from_tool`, `apply_patch`,
    `apply_patch_to_loaded`, `create_prepared`, `finalize_and_validate`.
  - `echo-agent/echo-orchestration/src/tasks/task_tools.rs:1-732` —
    `TaskCreateTool`, `TaskUpdateTool`, `TaskListTool` (the three
    framework tools), `build_task_tools` (205), and the schemas / parsers
    they own.
  - `echo-agent/src/tasks.rs:1-23` — `register_task_tools` (18-22) wraps
    `agent.add_tools(build_task_tools(service))`; module is a thin re-export.
  - `echo-agent/src/agent/react/mod.rs:386-400` — `ReactAgent::new`
    installs the framework tools with the in-memory store + default
    policy.
- EKO projection and adapter (echo-agent-cli):
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs`
    (350-460 TodoStatus, 800-1320 PlanTask / EkoTaskSpec /
    EkoTaskExecution / TaskPatch / TaskUpdateRequest / EkoTaskMetadata):
    the full projection layer and the `to_task_spec` / `to_task` /
    `TryFrom<Task> for PlanTask` / `to_task_plan_patch` conversion
    functions.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs`
    (full, 1-389) — `EkoRevisionedTaskStore` (the `RevisionedTaskStore`
    impl, 26-76), `EkoTaskToolPolicy` (the `TaskToolPolicy` impl,
    80-296), `build_eko_task_revision_service` (309-317),
    `apply_eko_task_update` (321-339, the IPC adapter),
    `commit_eko_task_plan` (344-388, the planner adapter).
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs`
    (1-100 module + struct, 624-885 commit adapter, 986-1105 claim
    primitives, 1123-1176 status-event write): `TaskRuntimeStore`,
    `load_revisioned_task_graph`, `compare_and_commit_revisioned_task_graph`,
    `append_task_status_event`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/task_tools.rs`
    (full, 1-1154) — `TaskCapabilityCatalog`, the task-local run context,
    and the three EKO-owned orchestration tools (`create_complex_task`,
    `check_run_status`, `cancel_run`). The framework tools are reached
    via `FrameworkTaskCreateTool = echo_agent::tasks::TaskCreateTool`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/register.rs`
    (full, 1-183) — `register_task_tools_on_agent`, the single
    post-hoc swap point that replaces the in-memory default backing with
    the EKO file-backed adapter.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/task_execute_tool.rs`
    (header + name/schema, 1-260) — confirmed `task_execute` is a dispatch
    shell, not a CRUD.
  - `echo-agent-cli/echo-agent-app-core/src/tool_exposure.rs:60-85, 219-267,
    295-360` — the `TASK_TOOLS` constant (the four canonical names).
- Cross-repo duplicate search (V03) for every task/plan/todo CRUD name
  across both `echo-agent` and `echo-agent-cli` repositories.
- Shared-fixture test verification (V04) for whether the same task
  graph is exercised through both framework and EKO paths.

## Out Of Scope

Deferred to downstream / already-audited tasks:

- The structural DAG validator — owned by F-TSK-02 (one
  `PlanValidator` authority). X-TSK-01 only verifies no second
  application-side validator is constructed.
- The runtime DAG kernel (`RuntimeDagExecutor`, ready-frontier,
  bounded waves, claims, cancellation drain) — owned by F-TSK-03.
  X-TSK-01 only verifies EKO injects one controller and does not
  duplicate the kernel.
- The claim/revision CAS, recovery, and terminal monotonicity — owned
  by A-TSK-04. X-TSK-01 only verifies the CAS routes through the
  framework service.
- The executor controller boundary (`EkoRuntimeDagController` thinness)
  — owned by A-TSK-03.
- The reviewer / artifact / parent-context paths — owned by A-TSK-06.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — rule 6 (single task-relationship authority;
    framework default is `task_create`/`task_update`/`task_list`, EKO
    adds `task_execute`; `TaskPlan` is a versioned artifact only;
    `TodoItem` is a UI projection with no store/state-machine/executor;
    the legacy global `todo_write` was deleted and must not be
    reintroduced; `plan_create`/`plan_patch`/`plan_execute` parallel
    CRUD is banned); the framework-vs-application layering gate; the
    "先查是不是已经有了" pre-implementation gate; the "adapter must stay
    thin" rule ("adapter 不得重新拥有 ready frontier、DAG 主循环、通用
    重试/取消、死锁判断"); the "echo-agent-cli does not need SQLite"
    invariant; UTF-8 / panic safety.
  - `docs/comprehensive-review/REPORTING.md`.
  - `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
- Dependency task reports read:
  - **F-TSK-01** (complete) — established the framework's canonical
    `TaskSpec`/`TaskExecution`/`TaskState` model, `RevisionedTaskGraph`
    as the sole graph authority, `TaskRevisionService` as the sole
    mutator, and the `RevisionedTaskStore` / `TaskToolPolicy` contracts
    this task's adapter implements. X-TSK-01 cross-cuts F-TSK-01 with
    A-TSK-01..06 to prove the layered seam holds end-to-end.
  - **F-TSK-02** (complete) — established `PlanValidator` as the sole
    structural DAG validator. X-TSK-01 verifies no second application
    validator exists.
  - **F-TSK-03** (complete) — established `RuntimeDagExecutor::execute`
    as the single production scheduling authority. X-TSK-01 verifies
    EKO does not duplicate it.
  - **A-TSK-01** (complete) — established the single file-authority
    model and the thin adapter pair at the persistence layer. X-TSK-01
    extends that conclusion to the field round-trip + tool-surface +
    shared-fixture axis.
  - **A-TSK-02** (complete) — established the tool inventory is clean
    (one framework trio + one EKO extension). X-TSK-01 re-verifies the
    forbidden-CRUD grep at the cross-repo scope.
  - **A-TSK-03** (complete) — established the executor controller is
    thin. X-TSK-01 consumes its conclusion that EKO injects only
    product policy.
  - **A-TSK-04** (complete) — established the claim/revision CAS is
    sound. X-TSK-01 uses its conclusion that every state mutation
    routes through the framework service.
  - **A-TSK-06** (complete) — explicitly hands off the field
    round-trip conclusion to X-TSK-01: "the field round-trip from EKO
    `PlanTask` to framework `TaskSpec` is lossless for
    `execution_checks` and `acceptance_criteria`." X-TSK-01 verifies
    that for every spec field.
- Historical documents treated as hypotheses: the module docstrings at
  `echo-agent/echo-orchestration/src/tasks/revisioned.rs:1-6`
  ("The framework owns patch semantics, structural validation,
  revisions, and optimistic concurrency. Applications provide
  persistence and product policy through narrow adapters."),
  `echo-agent-cli/.../revisioned_adapter.rs:1, 24-25` ("Thin EKO
  adapters ... deliberately has no patch or validation logic"), and
  `types.rs:917-920` ("shared `TaskStatus` remains authoritative and
  lossless. `TodoStatus` is derived only when building UI-facing
  plan/todo projections"). All verified below.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. The framework owns the canonical task model (`TaskSpec` 13 fields, `TaskExecution` 5 fields, `TaskStatus` 10 variants), the revisioned graph (`RevisionedTaskGraph`), the patch engine (`TaskPatchEngine::apply_operations` — pure, framework-owned, 474-628), the structural validator (`PlanValidator`, F-TSK-02), the kernel (`RuntimeDagExecutor`, F-TSK-03), the persistence contract (`RevisionedTaskStore` trait, 247-258), the policy contract (`TaskToolPolicy` trait, 270-314), the service (`TaskRevisionService`, 674), and the three tools (`task_create`/`task_update`/`task_list`). All live in `echo-orchestration::tasks`; the root `echo-agent/src/tasks.rs` only re-exports. |
| EKO product policy | Yes, correctly app-owned. The file-backed event authority (`events.jsonl` + `plan.json` spec + `run-state.json` execution), the 8-state `TodoStatus` UI projection, the `EkoTaskMetadata` payload (`domain_profile` + `parallel_group` + `sort_order`), the `EkoPlanMetadata` payload, the `TaskCapabilityCatalog` validation hook, the run-lifecycle `BackgroundTaskService`, the `task_execute` dispatcher, and the four EKO orchestration tools are all product policy layered on top of the framework model — they mutate run state, not the task graph directly. |
| Adapter boundary | `EkoRevisionedTaskStore` (`revisioned_adapter.rs:26-56`) is a pure pass-through (load → `store.load_revisioned_task_graph`; compare_and_commit → `store.compare_and_commit_revisioned_task_graph`; both with only error mapping). `EkoTaskToolPolicy` (80-296) does product policy only (scope resolution, run bootstrap, metadata round-trip, capability validation). `apply_eko_task_update` (321-339) and `commit_eko_task_plan` (344-388) are thin DTO converters that route through `service.apply_patch` / `service.create_prepared`. No EKO code re-implements `TaskPatchEngine`, `PlanValidator`, ready-frontier, retry, cancellation, or stall logic. |
| Duplicate search | Searched names (whole `echo-agent` + `echo-agent-cli`): `RevisionedTaskStore`, `TaskToolPolicy`, `TaskRevisionService`, `TaskPatchEngine`, `PlanValidator`, `RuntimeDagExecutor`, `build_task_tools`, `register_task_tools`, `InMemoryRevisionedTaskStore`, `EkoRevisionedTaskStore`, `EkoTaskToolPolicy`, `task_create`, `task_update`, `task_list`, `task_execute`, `todo_write`, `plan_create`, `plan_patch`, `plan_execute`. Result: ONE `RevisionedTaskStore` impl per side (framework `InMemoryRevisionedTaskStore` + EKO `EkoRevisionedTaskStore`); ONE `TaskToolPolicy` impl per side (`DefaultTaskToolPolicy` + `EkoTaskToolPolicy`); ONE `TaskRevisionService` definition; ONE `TaskPatchEngine::apply_operations`; ZERO application-defined `RuntimeDagExecutor` / `PlanValidator` / `task_*` tool; ZERO banned CRUD (`todo_write` survives only as a negative test in `echo-agent/src/agent/react/builder.rs:1181`). V01, V03. |
| Migration deletion | No migration proposed. No deletion candidate. The layering is live and singular. The one asymmetry (`commit_eko_task_plan` using `DefaultTaskToolPolicy` instead of `EkoTaskToolPolicy`) was already filed as A-TSK-01-P3-01 and is not re-raised here. |

## Current Path

Verified end-to-end task-graph and adapter flow at commits
`echo-agent` 9b0e0fa / `echo-agent-cli` b3b2e81.

### Authority chain

```text
Tool / IPC entrypoint (task_create / task_update / task_list / Tauri update_tasks)
   │
   ▼
framework TaskCreateTool / TaskUpdateTool / TaskListTool
   (echo-orchestration/src/tasks/task_tools.rs:15, 82, 143)
   │  parses JSON params locally (parse_task_create_input, parse_task_update_input)
   │  forwards to the service
   ▼
TaskRevisionService                                          [revisioned.rs:674]
   ├─ service.resolve_scope(context)                         [:702]
   │     → policy.resolve_scope
   ├─ service.policy.ensure_scope(scope_id, input, context)  [:724]
   │     (EKO bootstraps a Run if none exists)
   ├─ policy.prepare_task(scope_id, draft, position)         [:736]
   │     (EKO injects domain-default role + EkoTaskMetadata JSON)
   ├─ TaskPatchEngine::apply_operations(...)                 [:956]
   │     ★ framework-owned patch semantics (insert/update/skip/reorder/set_status)
   ├─ policy.finalize_task_metadata(...)                     [:999]
   ├─ policy.validate_candidate(scope_id, tasks)             [:1003]
   │     (EKO capability validation; framework DAG check below)
   ├─ PlanValidator.validate_task_snapshot(&tasks)           [:1007]
   │     ★ single structural DAG validator (F-TSK-02)
   └─ store.compare_and_commit(scope_id, commit)             [:976]
         (EkoRevisionedTaskStore.compare_and_commit)
           → TaskRuntimeStore.compare_and_commit_revisioned_task_graph
               under with_run_lock(run_id):
                 ① terminal-run guard
                 ② expected_revision CAS → PlanConflict on mismatch
                 ③ next_revision = expected+1 invariant
                 ④ EkoTaskMetadata decode round-trip
                 ⑤ initial-plan-must-be-pending invariant
                 ⑥ append PlanRevisionCommitted event
                 ⑦ rewrite_plan (rebuilds plan.json + run-state.json projections)
                 ⑧ return loaded RevisionedTaskGraph
```

### Round-trip seam (EKO ↔ framework)

```text
EKO file/UI projection               framework canonical model
───────────────────────              ─────────────────────────
PlanTask (types.rs:985)
  ├─ spec()  → EkoTaskSpec (892)
  │     └─ to_task_spec() ─────────► echo_agent::tasks::TaskSpec (13 fields, runtime.rs:179)
  │                                   └─ metadata packs {domain_profile, parallel_group, sort_order}
  ├─ execution() → EkoTaskExecution (922)
  │     └─ status: echo_agent::tasks::TaskStatus  (by identity, no conversion)
  └─ to_task() ─────────────────────► echo_agent::tasks::Task       (runtime.rs:251)

TryFrom<Task> for PlanTask (types.rs:1162)
  ├─ EkoTaskMetadata::from_str(spec.metadata)   (unpack)
  ├─ TodoStatus::try_from_task_status(execution.status)
  │     Err on Retrying/Paused  ← lossy edge (A-TSK-01-P2-02, latent per A-TSK-03-P3-02)
  └─ reconstructs PlanTask field-by-field

TaskUpdateRequest → TaskPlanPatch (types.rs:1283)
  └─ to_task_plan_patch() ─────────► echo_agent::tasks::TaskPlanPatch
                                       operations: Vec<TaskPlanPatchOp>
                                       (insert/update/skip/reorder)
```

Invariants verified by this graph (full evidence in V01-V04):

- **One revisioned graph.** `RevisionedTaskGraph`
  (`runtime.rs:42` + `revisioned.rs:42`) is the sole task-graph
  representation. `TaskRevisionService::create_from_tool` /
  `update_from_tool` / `apply_patch` / `create_prepared` all funnel
  through `apply_patch_to_loaded` → `TaskPatchEngine::apply_operations`
  → `store.compare_and_commit`. V02.
- **One mutator.** `TaskRevisionService` is the only writer; no EKO
  code calls `TaskPatchEngine::apply_operations` or constructs a
  `TaskGraphCommit` directly (V03). The two EKO-side adapters
  (`apply_eko_task_update`, `commit_eko_task_plan`) call
  `service.apply_patch` / `service.create_prepared` — they do not
  bypass the service.
- **One persistence contract.** `RevisionedTaskStore` trait
  (revisioned.rs:247) has exactly one framework impl
  (`InMemoryRevisionedTaskStore`, the default agent backing) and one
  EKO impl (`EkoRevisionedTaskStore`, the file-backed backing). The
  default `InMemory` store is replaced atomically by name at
  registration time (`register_task_tools_on_agent` →
  `agent.add_tools(build_task_tools(service))` → `DashMap::insert`),
  so per-agent only one store is live. V02.
- **One validator.** `PlanValidator` is invoked only at
  `TaskRevisionService::finalize_and_validate` (revisioned.rs:1007)
  and at the kernel's safe-point (echo-agent
  `runtime_executor.rs:214`, per F-TSK-03). The two `PlanValidator`
  references inside `TaskRuntimeStore`
  (`store.rs:96-100` `validate_runtime_plan` and
  `store.rs:921-923`) are both `#[cfg(test)]` — defense-in-depth in
  test helpers, never reached in production. V02.
- **One kernel.** `RuntimeDagExecutor::new` is constructed exactly
  once in `echo-agent-cli` (`executor.rs:1645`, per A-TSK-03 V01).
  EKO never duplicates the kernel. (Cross-references F-TSK-03 V01.)
- **Spec round-trip is field-by-field lossless.** All 13
  `echo_agent::tasks::TaskSpec` fields round-trip through
  `EkoTaskSpec` + `EkoTaskMetadata` (3 EKO-only fields packed into
  `metadata` JSON). Verified by inspecting every conversion site
  (`to_task_spec`, `to_task`, `TryFrom<Task>`, `load_revisioned_task_graph`,
  `compare_and_commit_revisioned_task_graph`, `to_task_spec_patch`,
  `to_task_plan_patch`). V01.
- **No parallel task/plan/todo CRUD.** Zero matches for `todo_write`,
  `plan_create`, `plan_patch`, `plan_execute` as tool/function names
  in production. The single `todo_write` reference is the framework's
  negative test (`echo-agent/src/agent/react/builder.rs:1181`).
  `to_task_plan_patch` is the legitimate adapter converter, not a
  tool. V03.
- **Shared fixture exercises both paths.** The EKO test
  `task_create_bootstraps_run_before_plan_events`
  (`task_tools.rs:416`) drives the framework `TaskCreateTool` (an
  alias for `echo_agent::tasks::TaskCreateTool`) wired through
  `build_eko_task_revision_service` and observes the framework
  service mutating the EKO file store. The EKO test
  `runtime_plan_respects_dependency_order` (`executor.rs:5524`) seeds
  a `PlanTask` graph via `attach_plan_for_test` (which itself runs
  `PlanValidator::validate_task_snapshot`, store.rs:96-100) and then
  drives `RuntimeDagExecutor::execute` through
  `execute_runtime_plan` — exercising the same `Task` graph through
  the framework validator, the framework patch engine, the framework
  kernel, AND the EKO persistence layer in one run. V04.

## Findings

The headline result is strongly positive: there is exactly one
revisioned TaskRun graph, EKO's projection is lossless for spec
fields, the adapter pair is thin, and there is no second
validator / executor / store / CRUD authority. AGENTS.md rule 6
holds end-to-end. The single recorded finding is a P3 documentation
note about an already-known execution-state lossiness; no new defect
is filed here.

### X-TSK-01-P3-01: The `types.rs:917-920` doc claim "shared `TaskStatus` remains authoritative and lossless" is inaccurate for the persisted event-stream path (carried over from A-TSK-01-P2-02 / A-TSK-03-P3-02)

- Priority: P3
- Confidence: high
- Layer: adapter (documentation)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:917-920`
    — the doc on `EkoTaskExecution` states "The shared `TaskStatus`
    remains authoritative and lossless. `TodoStatus` is derived only
    when building UI-facing plan/todo projections."
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1123-1176`
    — `append_task_status_event` writes `"status":
    status.as_str()` with `status: TodoStatus`. The authoritative
    event stream therefore encodes the 8-state `TodoStatus`, not the
    10-state framework `TaskStatus`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:424-439`
    — `TodoStatus::try_from_task_status` returns `Err` for
    `TaskStatus::Retrying { .. }` and `TaskStatus::Paused(_)`:
    "framework task status {status:?} has no lossless EKO todo
    projection."
- Reachability: the lossiness requires a writer that produces
  framework `Retrying` or `Paused` and routes it through
  `append_task_status_event`. A-TSK-03-P3-02 (high confidence)
  established that EKO's executor never writes those statuses:
  retry is expressed as `Pending` + `retry_count`, and pause as
  run-level `TaskRunStatus::Paused` (not task-level). The lossiness
  is therefore latent on the current executor path. The documentation
  drift remains: a reader of `types.rs:917-920` infers end-to-end
  losslessness that does not hold for the event-stream encoding.
- Expected invariant: documentation should describe the actual
  guarantee. Either the framework `TaskStatus` is persisted
  losslessly end-to-end, or the doc states that `Retrying`/`Paused`
  are deliberately not produced on the EKO executor path.
- Observed behavior: the doc claims losslessness; the event stream
  cannot represent two framework statuses; the executor never
  produces them today. Net effect: documentation overstates the
  contract.
- Impact: very low. No data loss on the current path. The risk is
  future drift: a commit that adds a `Retrying`/`Paused` writer on
  the executor→store path would make A-TSK-01-P2-02 live (data loss
  after `rewrite_plan`), and the misleading doc would slow diagnosis.
- Root cause: the type-level doc was written when `EkoTaskExecution`
  carried the framework `TaskStatus` by identity (which is true
  in-memory). It was not updated when `append_task_status_event`
  settled on `TodoStatus` as the event payload type.
- Direction: narrow the `types.rs:917-920` doc to state that
  `Retrying`/`Paused` are never produced by the EKO executor path
  (per A-TSK-03-P3-02), so the lossiness is a deliberate projection
  boundary, not a hazard. Same fix recommended by A-TSK-03-P3-02.
  Optionally add a regression test asserting the executor never
  produces those statuses.
- Regression validation: doc-only change. The optional regression
  test is the one proposed in A-TSK-03-P3-02 (drive a task through
  retry and assert the reloaded `EkoTaskExecution.status` round-trips
  the `Pending` + `retry_count` representation).
- Validation reports: [V01-01](../validations/X-TSK-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Field round-trip: framework `TaskSpec` ↔ EKO `EkoTaskSpec` is lossless for every spec field; `Retrying`/`Paused` are the only lossy statuses (latent) | yes | passed-with-caveat | [V01-01](../validations/X-TSK-01/V01-01.md) |
| V02 | Authority call graph: one mutator (`TaskRevisionService`), one persistence contract (`RevisionedTaskStore`), one validator (`PlanValidator`); adapter is thin | yes | passed | [V02-01](../validations/X-TSK-01/V02-01.md) |
| V03 | Forbidden CRUD search: zero `todo_write`/`plan_create`/`plan_patch`/`plan_execute` and zero parallel task_create/update/list/execute in production | yes | passed | [V03-01](../validations/X-TSK-01/V03-01.md) |
| V04 | Shared fixture through both framework and EKO paths: targeted tests pass | yes | passed | [V04-01](../validations/X-TSK-01/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — three module/type docstrings treated as hypotheses; classifications inline in Historical Claim Status) | passed | classified inline |

Executed cargo commands (all exit 0):

```text
cd echo-agent && cargo test -p echo_orchestration --lib revisioned::tests::
  → 3 passed; 0 failed; 0 ignored (revisioned::tests::{creates_and_patches_one_canonical_graph,
    stale_patch_reports_conflict, default_policy_supports_manual_progress_updates})

cd echo-agent-cli && cargo test -p echo-agent-app-core --lib task_create_bootstraps_run_before_plan_events
  → 1 passed; 0 failed; 0 ignored

cd echo-agent-cli && cargo test -p echo-agent-app-core --lib task_create_appends_to_the_same_revisioned_graph
  → 1 passed; 0 failed; 0 ignored

cd echo-agent-cli && cargo test -p echo-agent-app-core --lib task_update_insert_accepts_task_create_task_shape
  → 1 passed; 0 failed; 0 ignored

cd echo-agent-cli && cargo test -p echo-agent-app-core --lib runtime_plan_respects_dependency_order
  → 1 passed; 0 failed; 0 ignored
```

The full pre-commit matrix (fmt / clippy / all-features test) was not
re-run because this review is read-only; the targeted subsets above
are the directly relevant evidence — they are the suites that prove
the framework service and the EKO adapter exercise the same graph
through the same code path.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `echo-agent/echo-orchestration/src/tasks/revisioned.rs:1-6` module doc: "The framework owns patch semantics, structural validation, revisions, and optimistic concurrency. Applications provide persistence and product policy through narrow adapters." | current | Verified end-to-end by V02: framework owns `TaskPatchEngine::apply_operations`, `PlanValidator`, revision CAS in `compare_and_commit`; EKO supplies `EkoRevisionedTaskStore` (persistence) + `EkoTaskToolPolicy` (product policy) only. |
| `revisioned_adapter.rs:1, 24-25` "Thin EKO adapters ... deliberately has no patch or validation logic; those remain authoritative in the framework service." | current | Verified by V02: `EkoRevisionedTaskStore` (26-56) is pure pass-through with error mapping; `EkoTaskToolPolicy` (80-296) does product policy only; no patch/DAG/ready-frontier/retry logic. The single asymmetry (`commit_eko_task_plan` using `DefaultTaskToolPolicy`) was filed as A-TSK-01-P3-01 and is intentional. |
| `types.rs:917-920` "shared `TaskStatus` remains authoritative and lossless. `TodoStatus` is derived only when building UI-facing plan/todo projections." | current for in-memory adapter conversion; stale for the persisted event-stream path | `load_revisioned_task_graph` (store.rs:718-724) carries `TaskStatus` by identity in-memory. But `append_task_status_event` writes `TodoStatus` to the event stream (store.rs:1163-1172), so `Retrying`/`Paused` cannot survive a `rewrite_plan`. Latent because the EKO executor never produces those statuses (A-TSK-03-P3-02). Filed as X-TSK-01-P3-01 / A-TSK-01-P2-02. |
| `echo-agent/src/tasks.rs:15-17` doc on `register_task_tools`: "Tool registration is name-based, so this atomically selects the supplied store/policy adapter without exposing a second task API." | current | Verified by V02: framework tools are registered through `agent.add_tools(build_task_tools(service))` → `ToolManager::register_tools` → `DashMap::insert(tool.name(), tool)` (echo-execution/tools.rs:534-539). EKO's post-hoc registration replaces the three default tools atomically. |
| AGENTS.md rule 6: "任务关系只有一个权威 API" | current (corroborated) | V02 + V03 confirm: one `TaskRevisionService`, one `RevisionedTaskStore` per side, one `TaskPatchEngine`, one `PlanValidator`, one `RuntimeDagExecutor`, no parallel CRUD. |
| AGENTS.md rule 6: "framework default is `task_create/task_update/task_list`; EKO adds `task_execute`" | current (corroborated) | V03: `tool_exposure.rs:69 TASK_TOOLS = ["task_create", "task_update", "task_list", "task_execute"]`; the EKO module defines only `create_complex_task`/`check_run_status`/`cancel_run` (run-lifecycle) plus `task_execute` (dispatch shell). |
| AGENTS.md rule 6: "不得重新引入 `plan_create/plan_patch/plan_execute` 或其它平行任务 CRUD" | current (corroborated) | V03: zero matches for all three names anywhere in `echo-agent-cli`; the only `todo_write` reference is the framework's negative test. |
| AGENTS.md rule 6: "旧的进程全局 `todo_write` 已由框架直接删除" | current (corroborated) | V03: zero `todo_write` references in `echo-agent-cli`; one in `echo-agent/src/agent/react/builder.rs:1181` as a negative assertion. |
| AGENTS.md: "adapter 不得重新拥有 ready frontier、DAG 主循环、通用重试/取消、死锁判断" | current (corroborated) | V02: the EKO adapter holds no frontier (kernel does), no DAG loop, no retry (one-shot `Pending` resolution), no deadlock detection (kernel stall branch). |
| F-TSK-01 handoff: "A-TSK-* may layer EKO product policy on top of this framework model, not beside it" | current (extended) | V01/V02/V03 confirm the EKO layer adds product policy on top, never beside. |
| F-TSK-02 handoff: "F-TSK-03 must not introduce a second validator; should call `PlanValidator` before scheduling" | current (corroborated for EKO) | V02: EKO constructs zero `PlanValidator` in production; the two `store.rs` references are `#[cfg(test)]`. |
| A-TSK-06 handoff: "the field round-trip from EKO `PlanTask` to framework `TaskSpec` is lossless for `execution_checks` and `acceptance_criteria`" | current (extended to every spec field) | V01: all 13 `TaskSpec` fields round-trip losslessly, including `execution_checks` and `acceptance_criteria`. |

## Coverage And Uncertainty

- **Inspected in full:** the framework `TaskSpec` / `TaskExecution` /
  `TaskStatus` model (echo-agent `runtime.rs:1-260`), the framework
  `RevisionedTaskStore` / `TaskToolPolicy` / `TaskPatchEngine` /
  `TaskRevisionService` definitions (echo-agent
  `revisioned.rs:1-1014`), the framework `task_tools.rs` tool
  definitions, the EKO `revisioned_adapter.rs` (full 1-389), the EKO
  `types.rs` projection and conversion layer (350-460, 800-1320), the
  EKO `store.rs` authority/commit/claim primitives (1-100, 624-885,
  953-1105, 1123-1176), the EKO `task_tools.rs` orchestration surface,
  the EKO `register.rs` post-hoc swap, and the `task_execute` tool
  header/schema/dispatch shape.
- **Inspected partially:** the framework `task_tools.rs:1-732` was
  read in the slices that bear on the round-trip (the three `Tool`
  impls + the schema builders + the parsers); the per-field
  normalization inside the parsers (default `max_retries`, trim
  rules, execution-mode default) is F-TSK-01's territory and was not
  re-traced here. The EKO `executor.rs` was only sampled for the
  shared-fixture test names; its controller boundary is A-TSK-03's
  scope.
- **Not inspected (out of scope):**
  - The framework `RuntimeDagExecutor` kernel internals — F-TSK-03.
  - The framework `PlanValidator` rules — F-TSK-02.
  - The EKO worktree/ownership policy — A-TSK-05.
  - The full `echo-agent-cli` pre-commit matrix. The review is
    read-only; the targeted test subsets above are the directly
    relevant evidence.
- **Uncertain claims:**
  - Whether any out-of-repo consumer of `echo-agent`'s framework
    API implements its own `RevisionedTaskStore` / `TaskToolPolicy`
    is unknowable from this repo. The in-repo evidence shows exactly
    two impls per trait (one framework default + one EKO). Per
    AGENTS.md's framework-API retention rule, that is the expected
    state.
  - Whether the `TaskRuntimeStore`'s `pub` surface is consumed by an
    out-of-repo plugin that bypasses the framework service is
    unknowable. If it is, V03's "no parallel CRUD" conclusion would
    not cover that plugin; the in-repo evidence shows zero bypass
    paths.

## Handoff

- **Conclusions downstream tasks may rely on:**
  - There is exactly one revisioned TaskRun graph model. The framework
    `TaskSpec` (13 fields) is the canonical task specification; the
    framework `RevisionedTaskGraph` is the canonical graph
    representation; the framework `TaskRevisionService` is the sole
    mutator; the framework `TaskPatchEngine::apply_operations` is the
    sole patch semantics authority. (V02)
  - The EKO projection is field-by-field lossless for every spec
    field. EKO-only metadata (`domain_profile`, `parallel_group`,
    `sort_order`) round-trips through `TaskSpec.metadata` as
    `EkoTaskMetadata` JSON. The only lossiness is in execution-state
    representation: framework `Retrying`/`Paused` cannot be encoded
    in the authoritative event stream, but the EKO executor never
    produces them (latent, A-TSK-03-P3-02). (V01)
  - The adapter pair (`EkoRevisionedTaskStore`, `EkoTaskToolPolicy`)
    is thin and contains no patch/DAG/frontier/retry/cancel
    authority. (V02)
  - No parallel task/plan/todo CRUD exists anywhere in either
    repository. AGENTS.md rule 6 holds end-to-end. (V03)
  - The framework service and the EKO adapter are exercised by the
    same fixtures; the layering seam is not theoretical. (V04)
- **Reports downstream tasks must read:**
  - [V01-01](../validations/X-TSK-01/V01-01.md) for the per-field
    round-trip matrix and the latent-lossiness caveat.
  - [V02-01](../validations/X-TSK-01/V02-01.md) for the authority
    call graph and the single-impl-per-trait proof.
  - [V03-01](../validations/X-TSK-01/V03-01.md) for the cross-repo
    forbidden-CRUD grep result.
  - [V04-01](../validations/X-TSK-01/V04-01.md) for the shared-fixture
    test inventory and the per-test mapping.
- **Task-to-reference mapping:**
  - **X-STA-01** (persistence/recovery/identity continuity) → may
    rely on the layering seam being sound; the recovery and CAS paths
    are downstream of the framework service.
  - **X-AUT-01** (TUI/GUI/CLI parity) → may rely on the four
    canonical tool names being the same across all entry surfaces;
    `register_task_tools_on_agent` is called from both
    `src/main.rs:177` (TUI) and `src/tauri/desktop.rs:201` (GUI).
  - **B-ARCH-01** / **B-REF-01** → may rely on the layering holding
    end-to-end as evidence for the architectural claims.
- **Conditions that make this report stale:**
  - Any commit that adds a second `RevisionedTaskStore` or
    `TaskToolPolicy` impl in `echo-agent-cli` (beyond the two
    framework + two EKO already present) invalidates V02.
  - Any commit that introduces a writer of framework `Retrying` or
    `Paused` on the executor→store path invalidates V01's
    "losslessness is latent" classification (it would become live
    data loss per A-TSK-01-P2-02) and invalidates X-TSK-01-P3-01.
  - Any commit that adds a new task/plan/todo tool name (beyond
    `task_create`/`task_update`/`task_list`/`task_execute`/
    `create_complex_task`/`check_run_status`/`cancel_run`) invalidates
    V03.
  - Any commit that lets an EKO path call
    `TaskPatchEngine::apply_operations` directly (bypassing
    `TaskRevisionService`) invalidates V02.
  - Any commit that reintroduces `todo_write` / `plan_create` /
    `plan_patch` / `plan_execute` as a registered tool invalidates V03
    and the primary conclusion.
- **Follow-up task IDs (no fixes implemented in this review):**
  - X-TSK-01-P3-01's doc-narrowing fix is the same one A-TSK-03-P3-02
    already recommended; it should land in a documentation-focused
    cleanup commit alongside A-TSK-01-P2-02's optional regression
    test. No new code change is required from this review.
