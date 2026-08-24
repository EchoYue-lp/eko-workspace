# A-TSK-02: EKO task authoring tools

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean (verified before and after
> all validations; `cargo test` runs left no tracked changes)

## Question

Are `task_create/update/list` thin product shells over the one revisioned
graph without independent Todo/Plan CRUD or hidden global state?

**Answer: Yes. `task_create`/`task_update`/`task_list` are thin EKO shells:
the framework `TaskRevisionService` (patch engine, `PlanValidator`, CAS)
owns all authoring semantics; EKO contributes only product policy (run
bootstrap, domain defaults, capability checks, metadata round trip) and file
persistence. There is no independent Todo/Plan CRUD, no global todo search,
and no hidden global task-authoring state — the graph lives in files
(`events.jsonl` + projections, A-TSK-01). Three P3 surface issues remain:
a Debug-format status vocabulary in `check_run_status`, silent
schema-vs-parse fallbacks in `create_complex_task`, and stale framework docs
referencing the removed `demo22_plan_execute` example. No P0/P1/P2.**

## Scope

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/`:
  `task_tools.rs` (EKO tools: `CreateComplexTaskTool`, `CheckRunStatusTool`,
  `CancelRunTool`, `TaskCapabilityCatalog`, task-local run context),
  `register.rs` (registration for TUI/GUI primary agents),
  `revisioned_adapter.rs` (`EkoRevisionedTaskStore`, `EkoTaskToolPolicy`,
  `apply_eko_task_update`, `commit_eko_task_plan`,
  `build_eko_task_revision_service`), `planner.rs` (file-ownership policy),
  `types.rs` (EkoTaskSpec/TaskPatch/TaskUpdateRequest conversions,
  TaskRunStatus), `store.rs` (authoring-relevant CAS surface + stale-update
  tests only).
- `echo-agent-cli/echo-agent-app-core/src/tasks/service.rs`
  (BackgroundTaskService: background trigger adapter, `submit_dag` plan
  commit), `tool_exposure.rs` (per-mode tool groups), `infra.rs:495-522`
  (pooled-agent tool registration), `src/tauri/commands/task_runtime.rs`
  (`update_tasks` and read commands), `src/main.rs:177-192`,
  `src/tauri/desktop.rs:201-217` (registration call sites).
- Framework boundary: `echo-agent/echo-orchestration/src/tasks/task_tools.rs`
  (task_create/update/list schemas and execution),
  `revisioned.rs` (service, policy trait, patch application, stale rejection),
  `runtime.rs` (TaskSpec/TaskSpecPatch model).

## Out Of Scope

- File authority / crash consistency / conversion losslessness — A-TSK-01
  (its findings P1-01/P2-01 are assumed; this task verified the authoring
  boundary on top of that evidence).
- Execution semantics (claims, waves, retries, stalls), `task_execute`
  behavior and `RunExecutionLocks` global — A-TSK-03 / F-TSK-03.
- Recovery / replay / stale-write rejection across restarts — A-TSK-04.
- Frontend projections and ts-rs bindings — A-FE-01/02.
- Framework-side schema drift (F-TSK-01-P3-02/P3-03) — recorded in F-TSK-01,
  not re-recorded here.

## Inputs

- Root `AGENTS.md` (single task-relation API; TaskPlan artifact / TodoItem UI
  projection; no todo_write/plan_create/plan_patch/plan_execute
  reintroduction; framework-vs-app layering gate; UTF-8/panic safety).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (A-TSK-02 card),
  `zcode-ds/README.md`.
- Dependency reports read: `zcode-ds/reports/tasks/A-TSK-01.md` (events.jsonl
  sole write authority; thin lossless adapter; CAS verified at
  store.rs:676-885; findings P1-01/P2-01/P3-01/P3-02) and
  `zcode-ds/reports/tasks/F-TSK-01.md` (canonical model singular; tool family
  task_create/update/list; EKO adapter boundary = conversion only;
  P3-01 legacy TaskManager surface; P3-02/P3-03 framework schema drift).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/
  MASTER-PLAN.md`, `docs/2026-07-27-runtime-dag-kernel-convergence.md`,
  `docs/2026-07-28-task-tools-framework-migration-design.md` (classified in
  V05-01).

## Layering Decision

- Generic mechanism (framework, correctly placed): `TaskSpec`/`TaskExecution`/
  `TaskStatus`/`TaskClaim` model, `TaskRevisionService` +
  `TaskPatchEngine` + `TaskPlanPatchOp` semantics, `PlanValidator` structural
  validation, `RevisionedTaskStore` CAS boundary, and the
  `task_create`/`task_update`/`task_list` tools themselves
  (`echo-orchestration/src/tasks/task_tools.rs:15-203`). EKO never
  re-implements any of these (V01-01/V02-01).
- EKO product policy (application, correctly placed): `TaskRun`/`TaskPlan`/
  `PlanTask`/`TodoItem`/`TodoStatus` projections, run bootstrap via
  `ensure_scope`, domain defaults via `prepare_task`, capability validation via
  `TaskCapabilityCatalog`, `parallel_group` schema extension, file layout,
  `EkoTaskToolPolicy` metadata round trips, and the run-level tools
  (`create_complex_task`/`check_run_status`/`cancel_run` are Auto-mode run
  launchers, not task CRUD). `planner.rs` is file-ownership classification
  only — no validator, no frontier (AGENTS.md gate 4 passes).
- Adapter boundary: `EkoRevisionedTaskStore` implements only
  `load`/`compare_and_commit` (revisioned_adapter.rs:36-56); conversions
  `EkoTaskSpec::to_task_spec`, `TaskPatch::to_task_spec_patch`,
  `TaskUpdateRequest::to_task_plan_patch` are field-for-field
  (V03-01, A-TSK-01 V03-01). `apply_eko_task_update`/
  `commit_eko_task_plan` delegate to `TaskRevisionService::apply_patch` /
  `create_prepared`.
- Duplicate search terms (both repos, V01-01): `todo_write`, `TodoWriteTool`,
  `plan_create`, `plan_patch`, `plan_execute`, `to_task_plan_patch`,
  `TaskPlan`, `PlanTask`, `TodoItem`, `TodoStatus`, `EkoTaskSpec`,
  `TaskUpdateRequest`, `TaskUpdateOperation`, `TaskSpecPatch`,
  `task_create`, `task_update`, `task_list`, `task_execute`,
  `create_complex_task`, `check_run_status`, `cancel_run`, `search_todo`,
  `search_task`, `find_todo`, global `static`/`LazyLock`/`OnceLock` in
  `tasks/`. Result: one tool family; zero forbidden-tool definitions;
  zero global todo search; the only `plan_*`-named symbols are conversion
  helpers and doc/example references.
- Migration deletion check: no deletion target found on the EKO side; the
  only removal candidates are documentation references to the deleted
  `demo22_plan_execute` example (A-TSK-02-P3-03).

## Current Path

Verified call graph (V01-01/V02-01/V03-01):

1. Registration: TUI (`main.rs:177-192`) and GUI (`desktop.rs:201-217`) call
   `register_task_tools_on_agent` (register.rs:45-130) which builds
   `EkoRevisionedTaskStore`+`EkoTaskToolPolicy` via
   `build_eko_task_revision_service` (revisioned_adapter.rs:309-317) and
   registers the framework `task_create/task_update/task_list` through
   `echo_agent::tasks::register_task_tools` (`src/tasks.rs:18-23`), plus
   `CreateComplexTaskTool`, `CheckRunStatusTool`, `CancelRunTool`,
   `ExecuteTaskTool`. Pooled agents get the same three framework tools in
   `infra.rs:505-522` (task_execute intentionally excluded, §10.2).
2. Execution: `TaskCreateTool::execute_with_context` /
   `TaskUpdateTool::execute_with_context` parse with
   `service.task_input_schema_extensions()` (framework task_tools.rs:48-53,
   115-122) -> `TaskRevisionService::create_from_tool`/`update_from_tool`
   (revisioned.rs:723-940) -> `EkoTaskToolPolicy` hooks (ensure_scope run
   bootstrap, prepare_task defaults+metadata, prepare_initial_context,
   finalize_task_metadata, validate_candidate) -> `TaskPatchEngine` ->
   `finalize_and_validate` (EKO capability + framework PlanValidator)
   -> `compare_and_commit` CAS against `expected_revision`.
3. GUI authoring: `update_tasks` command (commands/task_runtime.rs:398-427)
   -> `apply_eko_task_update` (revisioned_adapter.rs:321-339) ->
   `TaskUpdateRequest::to_task_plan_patch` (types.rs:1284) ->
   `service.apply_patch` — same validator/CAS path.
4. Background/pipeline authoring: `service.rs:307-357` `submit_dag` ->
   `commit_eko_task_plan` (revisioned_adapter.rs:344-388) ->
   `create_prepared` (framework CAS revision 1).
5. Scope resolution: `EkoTaskToolPolicy::resolve_scope` (run_id ->
   turn_id-derived `taskrun:{turn_id}` -> task_local), with run_id injected
   by the executor via `with_run_context` (task_tools.rs:125-147).
6. Reads: `task_list` -> `service.load` -> `EkoRevisionedTaskStore::load` ->
   files (A-TSK-01); GUI reads (`get_task_plan`, `list_task_todos`,
   `list_task_runs`) go through `FileTaskStore` per run.

## Findings

### A-TSK-02-P3-01: `check_run_status` emits Rust Debug-format status while the rest of the product uses canonical snake_case

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/task_tools.rs:1088-1092`
  (`format!("{:?}", run.status)` -> "Running"/"Paused"/"Cancelled"),
  `types.rs:490-508` (`TaskRunStatus::as_str` -> "running"/"paused"/
  "cancelled", used by `run_status_string` service.rs:741-750 and IPC
  serialization).
- Reachability: `check_run_status` registered on the primary agent
  (register.rs:69-73), exposed in Auto mode (tool_exposure.rs:140-160),
  executed via `do_check` when the LLM polls a background run.
- Expected invariant: one canonical status vocabulary across tool outputs,
  IPC, and persisted files.
- Observed behavior: the tool's JSON `status` field uses Rust `Debug` shapes
  ("Running") while every other surface ("running"), including the same
  enum's `as_str`, uses snake_case.
- Impact: minor LLM-facing vocabulary drift; a model or user matching status
  strings must know both spellings; no functional breakage.
- Root cause: `Debug` formatting used as a shortcut instead of `as_str()`.
- Direction: change `do_check` to use `run.status.as_str()`; add an assertion
  test on the tool output shape.
- Regression validation: unit test asserting
  `check_run_status`-style output contains "running" for a Running run.
- Validation reports: [V01-01](../validations/A-TSK-02/V01-01.md),
  [V03-01](../validations/A-TSK-02/V03-01.md)

### A-TSK-02-P3-02: `create_complex_task` silently coerces invalid `domain_profile`/`plan_mode`/`priority` to defaults despite schema enums

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence: `task_tools.rs:886-900` (`DomainProfile::from_str(...).unwrap_or(General)`,
  `if plan_mode == "direct_execute" { AllowDirect } else { RequirePlan }`,
  `priority` default "background") vs the tool schema enum
  (`task_tools.rs:817-820`) and `DomainProfile::from_str` returning None on
  unknown values (`types.rs:55-64`).
- Reachability: LLM-generated `create_complex_task` calls (Auto mode,
  primary agent).
- Expected invariant: schema (enum values) and parse path agree; invalid
  values rejected with an error, not silently remapped.
- Observed behavior: an unknown domain_profile silently creates a General
  run; an unknown plan_mode silently becomes plan_then_execute; unknown
  priority silently becomes background — a run with the wrong execution
  posture can be launched without any signal.
- Impact: same defect class as F-TSK-01-P3-03 but on the EKO tool; a mistyped
  domain or mode yields a silently different run contract instead of an
  actionable parse error.
- Root cause: hand-written schema vs lenient fallback parsing in the tool
  implementation.
- Direction: reject unknown enum values at parse with a clear message (or
  clamp only documented aliases); add a schema-vs-parse conformance test for
  `create_complex_task`.
- Regression validation: fixture feeding "domain_profile": "medical_researchx"
  asserting a tool error, not a General run.
- Validation reports: [V03-01](../validations/A-TSK-02/V03-01.md),
  [V01-01](../validations/A-TSK-02/V01-01.md)

### A-TSK-02-P3-03: Framework docs reference removed `demo22_plan_execute` example — stale `plan_*`-named pointer

- Priority: P3
- Confidence: high
- Layer: framework (documentation)
- Evidence: `echo-agent/README.md:1086`, `README.zh.md:791`,
  `docs/zh/README.md:207`, `docs/en/README.md:212`,
  `examples/README.md:72` reference `examples/demo22_plan_execute.rs`, which
  does not exist (examples dir contains demo00-demo41 with no demo14/demo22;
  V01-01).
- Reachability: documentation consumers reading the example catalog; the
  name carries the forbidden `plan_execute` term, implying a plan CRUD API
  that was deleted in M13.
- Expected invariant: the example catalog lists only existing examples;
  no stale `plan_*`-named API surface remains visible.
- Observed behavior: five stale references to a nonexistent example remain
  after the plan_* API removal.
- Impact: documentation drift; consumers could believe a plan_execute API or
  demo exists (AGENTS.md forbids re-introducing plan CRUD); link target 404.
- Root cause: example removed with the plan_* migration but the README rows
  were not updated.
- Direction: remove the five references (or restore a demo02-style example
  demonstrating the current `task_create/task_update/task_list/task_execute`
  API under a non-`plan_*` name).
- Regression validation: scripted check that every README example link has a
  file in `examples/`.
- Validation reports: [V05-01](../validations/A-TSK-02/V05-01.md),
  [V01-01](../validations/A-TSK-02/V01-01.md)

No other findings. In particular: no forbidden parallel CRUD, no global todo
search, no hidden global task state, no second authoring authority —
recorded as positive evidence in V01-01/V02-01/V03-01.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search (registered tool inventory; forbidden todo_write/plan_* CRUD; global todo search; hidden global state) | yes | passed | [V01-01](../validations/A-TSK-02/V01-01.md) |
| V02 | Registration + runtime reachability (create/update/list call paths on TUI/GUI/pool/Tauri/background) | yes | passed | [V02-01](../validations/A-TSK-02/V02-01.md) |
| V03 | Schema parity + invariants (framework vs EKO schemas/fields/ops; stale update; set_status policy) | yes | passed | [V03-01](../validations/A-TSK-02/V03-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::task_tools` | yes | passed (exit 0, 14 ok) | [V04-01](../validations/A-TSK-02/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::register` | yes | passed (exit 0, 1 ok) | [V04-02](../validations/A-TSK-02/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::service` | yes | passed (exit 0, 4 ok) | [V04-03](../validations/A-TSK-02/V04-03.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::store` (incl. stale-update rejection) | yes | passed (exit 0, 34 ok) | [V04-04](../validations/A-TSK-02/V04-04.md) |
| V05 | Historical-document drift (MASTER-PLAN, convergence, migration design) | conditional | passed (1 stale doc ref -> P3-03) | [V05-01](../validations/A-TSK-02/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN.md:442-445: todo_write replaced; TaskPlan/TodoItem only artifact/UI projections; do not reintroduce plan_* CRUD or an independent todo store | current | zero forbidden definitions (V01-01); types.rs:835,1323 projection docs |
| MASTER-PLAN.md:66: task-tools framework migration complete behind RevisionedTaskStore trait; EKO owns persistence/bootstrap | current | revisioned_adapter.rs:36-56, 309-317 (V02-01) |
| runtime-dag-kernel-convergence.md:174-191: plan_create/plan_patch/plan_execute replaced; no aliases; update_tasks shares TaskUpdateRequest; one task id/state/event/UI authority | current | grep V01-01; commands/task_runtime.rs:398-427; tool_exposure.rs:69 (V05-01) |
| task-tools-migration-design.md:47,141: separate task tools instead of action-switched todo_write | current | four distinct framework tools (V02-01) |
| README example catalog: demo22_plan_execute | stale | file absent in examples/ (V05-01 -> P3-03) |

## Coverage And Uncertainty

- `executor.rs` (EKO controller) and `task_execute_tool.rs` internals were
  inspected only at their tool-boundary/global-state surface; execution
  semantics are A-TSK-03/F-TSK-03.
- `TaskCapabilityCatalog` snapshots `agent.tool_names()` at registration
  time; a tool connected later (e.g., a mid-session MCP server) would be
  rejected as "unknown tool" in `validate_candidate` until re-registration.
  This is a product-policy edge with a clear error message; the MCP connect
  ordering is A-INT-01 scope and was not verified here.
- Frontend (ts-rs bindings, `TaskUpdateRequest` consumers in stores) not
  reviewed — A-FE-01/02.
- No live end-to-end run (GUI/TUI click-through) was executed; reachability
  is by static trace + unit tests.
- `RUN_EXECUTION_LOCKS` (DashMap of per-run Weak locks) is process-global and
  never removes entries for runs that are never re-executed — entries hold
  only a Weak so retention is a few bytes per distinct run; judged
  execution-side and negligible, delegated to A-TSK-03/Q-PERF-01 rather than
  a finding here.

## Handoff

- Downstream tasks may rely on: authoring tools are thin shells over one
  framework revisioned service (V01/V02/V03); schemas are framework + one
  declared extension with field-for-field DTO conversion (V03-01); EKO
  disables `set_status` (execution state cannot be forged via authoring
  tools); stale updates rejected on service + store layers (V04-04 +
  F-TSK-01 V04-02); no global todo search and no hidden global authoring
  state (V01-01).
- Reports to read: 8 validation reports above; A-TSK-01 (file authority,
  P1-01/P2-01 crash gaps), F-TSK-01 (P3-02/P3-03 framework schema drift,
  P3-01 legacy surface), F-TSK-02/F-TSK-03 for execution semantics.
- Stale conditions: this report becomes stale if `register.rs` registration,
  `revisioned_adapter.rs` policy/store adapters, framework
  `task_tools.rs`/`revisioned.rs` service paths, `types.rs` conversions, or
  `tool_exposure.rs` groups change; also if a `todo_write`/`plan_*` symbol or
  global todo search is (re)introduced.
- Follow-up task IDs: A-TSK-03 (controller boundary; set_status/executor
  ownership), A-TSK-04 (recovery/claims), X-TSK-01 (cross-repo adapter
  conformance), X-INV-01 (invariant audit can reuse V01-01 forbidden-term
  evidence), S-RDM-01 (roadmap: P3-01..03 fixes; README example link check).
