# A-TSK-01: TaskRuntime file authorities and typed adapter

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Do plan/events/run-state files have unambiguous authority, and is conversion
to framework task types thin and lossless?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs` (1-205,
  302-425, 676-885, 953-1010, 1123-1195, 1482-1530, 1778-1817) —
  `TaskRuntimeStore` write authority, `StoreError`, per-run `plan_locks`,
  `load_revisioned_task_graph` / `compare_and_commit_revisioned_task_graph`
  EKO↔framework graph conversion, `set_task_status` /
  `append_task_status_event` event payload shape, read delegations.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs`
  (full, 1-436) — `FileTaskShadow`: `events.jsonl` append authority,
  `plan.json` / `run-state.json` projection rewrite, `read_events` /
  `read_plan` / `read_run_state`, `atomic_write` / `append_line` durability,
  per-run write locks, seq cache.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_store.rs`
  (full, 1-222) — `FileTaskStore` read-side helper: `load` (reads all three
  files), `get_run` / `get_plan` / `list_todos` merge path,
  `list_runs` / `latest_run_for_conversation` collection queries.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs`
  (full, 1-389) — `EkoRevisionedTaskStore` (`RevisionedTaskStore` impl),
  `EkoTaskToolPolicy` (`TaskToolPolicy` impl), `build_eko_task_revision_service`,
  `apply_eko_task_update`, `commit_eko_task_plan`.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/event_rebuild.rs`
  (full, 1-501) — `rebuild_plan_from_events` fold, `RebuiltPlan`,
  `RebuildError`, partial-payload defaults, execution preservation across
  plan revisions.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs`
  (250-450, 800-1320) — `PlanTaskKind` / `TodoStatus` ↔ framework
  `TaskKind` / `TaskStatus` maps, `TaskPlan`, `EkoTaskSpec`,
  `EkoTaskExecution`, `PlanRevision`, `RunStateSnapshot`, `PlanTask`
  (`to_task` / `try_from` / `from_parts` / `spec` / `execution`),
  `TaskPatch` / `TaskUpdateRequest` patch conversion, `TodoItem` projection,
  `RuntimeTaskEvent`.
- Cross-repo duplicate search for `TaskPlan`, `PlanTask`, `TodoItem`,
  `RevisionedTaskStore`, `TaskToolPolicy`, `PlanValidator`,
  `todo_write`, `plan_create`, `plan_patch`, `plan_execute` across the whole
  `echo-agent-cli` repository.

## Out Of Scope

Deferred to downstream tasks:

- **A-TSK-02**: EKO task authoring tools (`task_create/update/list` product
  shells, registered tool inventory, schema parity). This task audits the
  adapter types and file authority, not the tool surface.
- **A-TSK-03**: task execution controller boundary (`RuntimeDagExecutor`
  injection, ready-frontier/retry/cancel ownership). How the framework
  executor's `TaskStatus` updates (incl. `Retrying`/`Paused`) reach the store
  is touched here only as it bears on round-trip losslessness.
- **A-TSK-04**: claims, revisions, recovery, terminal monotonicity
  (recovery barriers, `RecoveryBlocker`/`RecoveryDecision` lifecycle). Only
  the file-authority and CAS mechanics are in scope here.
- Framework-side `RevisionedTaskStore` trait contract and
  `TaskRevisionService` internals — audited under F-TSK-01.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — rule 6 (single task-relationship authority;
    `TaskPlan` is a versioned artifact only; `TodoItem` is a UI projection;
    no independent store/state-machine/executor; framework defaults to
    `task_create/update/list`, EKO adds `task_execute`), the framework-vs-
    application layering gate, the "先查是不是已经有了" pre-implementation
    gate, the "echo-agent-cli does not need SQLite" invariant, and the
    UTF-8 / panic-safety rules.
  - `docs/comprehensive-review/REPORTING.md`,
    `docs/comprehensive-review/templates/{task-report,validation-report}.md`.
- Dependency task reports read:
  - **F-TSK-01** (complete) — established the canonical framework task model
    (`TaskSpec`/`TaskExecution`/`TaskState`), `RevisionedTaskGraph` as the
    sole graph authority, `TaskRevisionService` as the sole mutator, and the
    `RevisionedTaskStore` / `TaskToolPolicy` contracts this adapter
    implements.
  - **F-TSK-02** (complete) — established `PlanValidator` as the sole
    structural DAG validator; confirmed `PlanSpec` is a versioned artifact.
    This task verifies the application delegates to it (V03-01).
  - **B-REF-01** (complete) — convergence C1 (plan is a versioned artifact,
    not a runtime approval state machine); used to assess `TaskRunStatus`.
  - **A-STATE-01** (complete) — recurring atomic-write prior
    (`Persistence::write_json` missing fsync + parent-dir sync); used as the
    baseline for the `atomic_write` / `append_line` durability read.
- Historical documents treated as hypotheses: the module docstrings at
  `store.rs:1-9`, `file_shadow.rs:1-7` (events authority + projection
  model), `revisioned_adapter.rs:1, 24-25` (thin adapter, no patch logic),
  and `types.rs:917-920` (`TaskStatus` authoritative and lossless,
  `TodoStatus` derived) — all verified below.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework owns the canonical task model and revisioned graph (`RevisionedTaskStore`, `TaskToolPolicy`, `TaskRevisionService`, `PlanValidator` — F-TSK-01/02). The application must not duplicate these. V03-01 confirms it does not. |
| EKO product policy | The file-authority model (`events.jsonl` + projections, no SQL), the EKO projection types (`TaskPlan`/`PlanTask`/`EkoTaskSpec`/`EkoTaskExecution`/`TodoItem`), the `DomainProfile`/`AttendedMode`/`parallel_group`/`sort_order` metadata, run bootstrap in `ensure_scope`, and the `task_execute` extension are all EKO product policy correctly layered on top of the framework model. |
| Adapter boundary | `EkoRevisionedTaskStore` and `EkoTaskToolPolicy` are thin: pure type conversion + metadata injection + product policy (scope resolution, run bootstrap, capability validation via `TaskCapabilityCatalog`). No patch/DAG validation/ready-frontier/retry authority lives in the adapter (V02-01). One asymmetry: `commit_eko_task_plan` uses `DefaultTaskToolPolicy` instead of `EkoTaskToolPolicy` (P3-01). |
| Duplicate search | Searched names (whole `echo-agent-cli` repo): `struct TaskPlan`, `struct PlanTask`, `struct EkoTaskSpec`, `struct EkoTaskExecution`, `struct TodoItem`, `impl RevisionedTaskStore`, `impl TaskToolPolicy`, `struct PlanValidator`, `fn task_dependency_cycles`, `fn task_topological_order`, `todo_write`, `plan_create`, `plan_patch`, `plan_execute`. Result: ONE definition each of the projection types (all in `types.rs`); ONE `impl RevisionedTaskStore` + ONE `impl TaskToolPolicy` (both in `revisioned_adapter.rs`); ZERO application DAG validators (framework delegated); ZERO banned CRUD tools (`to_task_plan_patch` is the legitimate adapter converter, not a tool). V03-01. |
| Migration deletion | No migration proposed. No deletion candidate at this layer — the projection set and adapter pair are live and singular. |

## Current Path

Verified file-authority and conversion data flow at commits
`echo-agent` 9b0e0fa / `echo-agent-cli` b3b2e81:

1. **File authority (single writer).** Every `TaskRuntimeStore` mutator
   follows one shape: acquire per-run `plan_locks` mutex (`store.rs:77-82`)
   → `shadow.append_event_line(...)` (`file_shadow.rs:118-178`, the sole
   write primitive, `O_APPEND` + `sync_all`) → `shadow.rewrite_plan(run_id)`
   (`file_shadow.rs:208-280`). No mutator writes `plan.json` or
   `run-state.json` except through `rewrite_plan`. Verified on `create_run`
   (`store.rs:336-356`), `transition_run` (`store.rs:387-425`),
   `compare_and_commit_revisioned_task_graph` (`store.rs:867-881`),
   `set_task_status` → `append_task_status_event`
   (`store.rs:1123-1186`). Same-run writes serialize on `plan_locks` AND
   `FileTaskShadow::run_write_locks` (`file_shadow.rs:30-34`); different
   runs run in parallel. V01-01.

2. **File layout.** `{root}/{run_id}/` (`root` = `~/.eko/tasks/`,
   `file_shadow.rs:82-101`) holds exactly:
   - `events.jsonl` — append-only event stream (authority).
   - `plan.json` — `PlanRevision` = **spec only** (no execution status);
     atomically rewritten via `atomic_write` (unique tmp + fsync + rename,
     `file_shadow.rs:267-272, 405-422`).
   - `run-state.json` — `RunStateSnapshot` = run header +
     `Vec<EkoTaskExecution>`; `EkoTaskExecution.status` is typed as the
     **framework `echo_agent::tasks::TaskStatus`** (`types.rs:922-929`);
     atomically rewritten (`file_shadow.rs:273-278`).

3. **Projection rebuild.** `rewrite_plan` reads the full event stream and
   folds it via `rebuild_plan_from_events` (`event_rebuild.rs:59-260`),
   then rewrites only the projection(s) the latest event affects
   (`affects_plan` for `PlanRevisionCommitted`; `affects_run_state` for
   Run/Task status events). The fold carries forward prior execution state
   across plan revisions (`event_rebuild.rs:147-162`) and applies
   `skipped_task_ids` / `reset_task_ids` effects (lines 163-193). V04-01.

4. **Read path.** `get_run`/`get_plan`/`list_todos` delegate to
   `FileTaskStore` (`store.rs:1482-1487, 1778-1788`).
   `FileTaskStore::load` (`file_store.rs:37-55`) reads all three files and
   propagates any error; `get_plan` (`file_store.rs:125-160`) joins the
   spec snapshot with execution state via `PlanTask::from_parts` and never
   rebuilds from events. V01-01, V04-01.

5. **EKO↔framework conversion (spec).** `EkoTaskSpec::to_task_spec`
   (`types.rs:892-914`) maps all 13 spec fields 1:1 onto
   `echo_agent::tasks::TaskSpec` and packs EKO-only fields
   (`domain_profile`, `parallel_group`, `sort_order`) into
   `TaskSpec.metadata` as `EkoTaskMetadata` JSON. The inverse runs in
   `compare_and_commit_revisioned_task_graph` (`store.rs:804-832`) and
   `TryFrom<Task> for PlanTask` (`types.rs:1173-1199`). `TaskPatch` /
   `TaskUpdateRequest` convert 1:1 onto framework `TaskSpecPatch` /
   `TaskPlanPatch` / `TaskPlanPatchOp` (`types.rs:1236-1316`). No spec field
   is dropped. V02-01.

6. **EKO↔framework conversion (execution).** `EkoTaskExecution.status` IS
   the framework `TaskStatus` (`types.rs:924`), so `load_revisioned_task_graph`
   (`store.rs:718-724`) and `compare_and_commit_revisioned_task_graph`
   (`store.rs:807-832`) carry it through by identity. The lossy surface is
   `TodoStatus` (`types.rs:367-377`): `to_task_status` (forward) is total,
   but `try_from_task_status` (`types.rs:424-439`) errors on `Retrying`/
   `Paused`, and `project_task_status` (`types.rs:443-455`) collapses them.
   Crucially, the authoritative event stream stores the lossy form:
   `append_task_status_event` writes `"status": status.as_str()` with
   `status: TodoStatus` (`store.rs:1170-1180`), and `rewrite_plan`
   regenerates `run-state.json` from that stream. V02-01.

7. **Adapter policy hooks.** `EkoTaskToolPolicy` (`revisioned_adapter.rs:80-296`)
   owns: `task_input_schema_extensions` (the `parallel_group` extension),
   `resolve_scope` / `ensure_scope` (run bootstrap from `task_create` when
   no run exists), `prepare_task` (domain-default agent role + EKO
   metadata), `prepare_initial_context` (plan metadata), `finalize_task_metadata`
   (sort_order normalization), `validate_candidate` (capability check via
   `TaskCapabilityCatalog`). None of these re-implement framework
   patch/validation/scheduling. V02-01, V03-01.

8. **Run state machine.** `TaskRunStatus` (`types.rs:480-488`) is a clean
   6-state machine (Pending/Running/Paused/Cancelled/Failed/Completed) with
   a `can_transition_to` guard (`types.rs:517-527`). No plan-approval states
   (Planning/AwaitingApproval/Ready) — consistent with B-REF-01 C1.

## Findings

The layering is clean: one file authority, one thin adapter pair, one
projection set, framework-validator delegation, no banned CRUD (V01/V03
positive). The two recorded findings concern robustness of the authority
model under corruption (P2-01) and a latent lossiness in the execution-state
round-trip (P2-02), plus two P3 notes.

### A-TSK-01-P2-01: A single malformed line in `events.jsonl` bricks every read, including projection-only reads that do not depend on the event file

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:362-379`
    — `read_events` trims each line and `continue`s only on empty lines
    (370-373); a non-empty malformed line returns
    `ShadowError::Decode("line {i}: {e}")` (374-375) instead of being
    skipped.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_store.rs:37-55`
    — `FileTaskStore::load` eagerly reads `read_plan?`, `read_run_state?`,
    `read_events?` and propagates any error. Therefore `get_run`
    (`file_store.rs:57-59`, needs only `state.run`) and `get_plan`
    (`file_store.rs:125-160`, needs only `plan` + `state.tasks`) fail when
    `events.jsonl` is unparseable, even though `plan.json` and
    `run-state.json` are intact and sufficient.
- Reachability: every `get_run` / `get_plan` / `list_todos` /
  `list_events` / `load_revisioned_task_graph` call. Triggered by any
  single corrupted/partial line in a run's `events.jsonl` (disk error,
  editor save, crash mid-append — see P3-02).
- Expected invariant: per the V01-01 authority model, `plan.json` and
  `run-state.json` are "deterministic read projections"; a projection-derived
  answer must not depend on the event file being parseable end-to-end.
- Observed behavior: any `Decode` error from `read_events` aborts the whole
  `load`, so projection-only reads return
  `StoreError::InvalidPlan("file read: ...")`. The run vanishes from the
  UI/recovery surface until the offending line is manually repaired.
- Impact:
  - **Availability.** One bad line takes the whole run offline — the run
    list, the plan view, and recovery all error. Because EKO is a local
    assistant and `events.jsonl` is append-only and human-editable, a
    truncated tail (P3-02) or an external edit is a realistic trigger.
  - **Recovery friction.** The intact projections cannot be used to recover
    the run; the user must hand-edit `events.jsonl`.
  - **No correctness risk** to other runs (per-run isolation holds).
- Root cause: `load` was written to read all three files unconditionally
  (the projection-merge and the event-fold paths share one loader), and
  `read_events` treats any malformed line as fatal rather than best-effort.
  The `append_event_line` doc (`file_shadow.rs:113-117`) even asserts
  "`read_events` skips empty lines and a future hardening pass (gate 2)
  will truncate a partial tail" — but partial lines are not empty, and the
  truncation pass was never implemented (P3-02).
- Direction:
  1. Make `read_events` resilient: skip-and-log malformed lines (or at
     minimum truncate a partial trailing line) instead of erroring. Keep a
     strict mode for the parity test.
  2. Decouple projection reads from the event file: `get_run` should need
     only `run-state.json`; `get_plan` should need only `plan.json` +
     `run-state.json`. Reserve `read_events` for `list_events`,
     `rewrite_plan`, and `list_todos`'s runtime-field fold. Either split
     `load` into per-projection readers or make the event read best-effort
     inside `load`.
- Regression validation: a test that (a) writes a run, (b) appends a
  malformed line to its `events.jsonl`, then asserts `get_run` and
  `get_plan` still return the projection values and `list_events` skips the
  bad line (or returns the good prefix) with a logged warning.
- Validation reports: [V04-01](../validations/A-TSK-01/V04-01.md)

### A-TSK-01-P2-02: Execution-state round-trip is lossy — the authoritative event stream encodes `TodoStatus` and cannot represent the framework's `Retrying`/`Paused` task states

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:424-439`
    — `TodoStatus::try_from_task_status` returns `Err` for
    `TaskStatus::Retrying { .. }` and `TaskStatus::Paused(_)`: "framework
    task status {status:?} has no lossless EKO todo projection".
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:443-455`
    — `project_task_status` is the lossy fallback: `Retrying → Running`,
    `Paused → Blocked`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1170-1180`
    — `append_task_status_event` writes `"status": status.as_str()` where
    `status: TodoStatus`. The authoritative event stream (V01-01) therefore
    carries the lossy projection, not the framework `TaskStatus`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/event_rebuild.rs:216-220`
    — rebuild reads the payload `"status"` string via
    `TodoStatus::from_str`, and `PlanTask::execution`
    (`types.rs:1081-1089`) forward-maps it back to framework `TaskStatus`,
    so `Retrying`/`Paused` can never reappear after a `rewrite_plan`.
- Reachability: any task that enters `Retrying` or `Paused` under the
  framework executor and whose state is then persisted via the
  `set_task_status`/`append_task_status_event` path. The frequency depends
  on the executor↔store wiring audited under A-TSK-03; the type-level
  lossiness established here is reachable whenever a `TodoStatus`-typed
  status is the one written.
- Expected invariant: the declared authority (`events.jsonl`) must be able
  to reproduce every framework task state, or the lossiness must be
  documented as a deliberate projection boundary that the framework path
  routes around.
- Observed behavior: the event stream and the rebuilt `run-state.json` can
  only carry the eight `TodoStatus` values. `Retrying` and `Paused` are
  collapsed to `Running` and `Blocked` respectively on every
  status-event-driven rewrite. The `EkoTaskExecution.status` field is typed
  as framework `TaskStatus` (`types.rs:924`), suggesting lossless storage,
  but `rewrite_plan` overwrites it from the lossy stream. The docstring at
  `types.rs:917-920` ("`TaskStatus` remains authoritative and lossless") is
  accurate for the in-memory adapter conversion (`load_revisioned_task_graph`
  carries `TaskStatus` by identity) but inaccurate for the persisted
  event→projection path.
- Impact:
  - **State fidelity.** After a recoverable transient (`Retrying`) or an
    interactive hold (`Paused`), the persisted state and the UI badge show
    `running`/`blocked` instead of the richer framework state. A restart
    after such a transition resumes from the collapsed state, losing the
    retry-attempt count context or the pause reason that the framework
    `TaskStatus` carried.
  - **No spec loss.** Task *specifications* round-trip losslessly; only
    execution state is affected.
- Root cause: the event payload was keyed on `TodoStatus` (the UI-facing
  enum) rather than on the framework `TaskStatus`, and `rewrite_plan`
  regenerates the framework-typed `run-state.json` from that lossy source.
  The split between "lossless authority" and "lossy UI projection" was
  drawn at the wrong boundary.
- Direction: persist the framework `TaskStatus` (or its string form) in the
  event payload — e.g. add a `"task_status"` field carrying the framework
  status next to the existing `"status"` (`TodoStatus`) field — and have
  `rebuild_plan_from_events` + `rewrite_plan` restore `run-state.json`'s
  `EkoTaskExecution.status` from that field. Keep `TodoStatus` as the
  display-only badge derived via `project_task_status`. Alternatively, if
  the executor never persists `Retrying`/`Paused` through this path, narrow
  the docstring and add a test asserting the invariant. The decision turns
  on the A-TSK-03 executor↔store wiring and should be made there.
- Regression validation: a test that drives a task into `Retrying` (and
  `Paused`) via the executor path, persists, drops in-memory state, and
  asserts the reloaded `EkoTaskExecution.status` equals the original
  framework status.
- Validation reports: [V02-01](../validations/A-TSK-01/V02-01.md)

### A-TSK-01-P3-01: `commit_eko_task_plan` uses `DefaultTaskToolPolicy` instead of `EkoTaskToolPolicy`, skipping capability validation and sort_order normalization for the planner path

- Priority: P3
- Confidence: high
- Layer: adapter
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs:375-378`
    — `commit_eko_task_plan` constructs
    `TaskRevisionService::new(Arc::new(EkoRevisionedTaskStore::new(store.clone())), Arc::new(DefaultTaskToolPolicy::new(run_id.clone())))`.
  - Contrast with `build_eko_task_revision_service` (`revisioned_adapter.rs:309-317`),
    which wires `EkoTaskToolPolicy` for the task-tool path.
- Reachability: every initial planner commit (the "agent_task_plan" /
  pre-built complete-plan path). The task-tool path (`task_create`) is
  unaffected.
- Expected invariant: both entry points into the framework revision service
  should apply the same EKO product policy, or the divergence should be
  justified and the skipped hooks harmless.
- Observed behavior: the planner path bypasses `EkoTaskToolPolicy`'s
  `validate_candidate` (capability validation via `TaskCapabilityCatalog`)
  and `finalize_task_metadata` (sort_order normalization by position). The
  caller pre-packs `EkoTaskMetadata` (`revisioned_adapter.rs:344-368`), so
  metadata round-trips, but a plan whose tasks reference unknown
  capabilities would be accepted here while being rejected by `task_create`.
- Impact: low. Planner output is structurally well-formed in practice, and
  the framework still owns canonical validation, revision 1, and CAS. The
  asymmetry is a latent inconsistency: a capability rule tightened later
  would not gate the planner.
- Root cause: the comment (`revisioned_adapter.rs:342-344`) states the
  planner "has already selected the EKO fields"; the policy hooks were
  dropped to avoid double-encoding metadata, without considering the
  validation hook.
- Direction: route `commit_eko_task_plan` through `EkoTaskToolPolicy` too
  (construct one for the run, or reuse `build_eko_task_revision_service`),
  and make `prepare_task`/`finalize_task_metadata` idempotent when metadata
  is already packed. If the bypass is intentional, document which hooks are
  intentionally skipped and add a test asserting capability rejection still
  fires on the task-tool path.
- Regression validation: a test that submits a plan with an unknown
  capability through both `commit_eko_task_plan` and `task_create` and
  asserts consistent acceptance/rejection.
- Validation reports: [V02-01](../validations/A-TSK-01/V02-01.md)

### A-TSK-01-P3-02: Partial-tail truncation for `events.jsonl` is documented as a "future hardening pass (gate 2)" but not implemented

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:113-117`
    — `append_event_line` doc: "A crash mid-append can at worst lose the
    last partial line — `read_events` skips empty lines and a future
    hardening pass (gate 2) will truncate a partial tail."
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:362-379`
    — `read_events` does not truncate a partial trailing line; it returns
    `ShadowError::Decode` for any malformed line (the mechanism that turns
    this into P2-01).
- Reachability: any process crash between `f.write_all` and the trailing
    newline `sync_all` in `append_line` (`file_shadow.rs:427-436`) leaves a
    partial JSON line that the next `read_events` rejects.
- Expected invariant: the documented "at worst lose the last partial line"
  behavior should hold — a partial tail should be truncated or skipped, not
  treated as a fatal decode error.
- Observed behavior: the partial line is neither empty nor valid JSON, so
  `read_events` errors; per P2-01 this bricks projection-only reads too.
- Impact: low under normal operation (crashes mid-append are rare and
  `O_APPEND` + `sync_all` minimize the window), but it is the most likely
  trigger for P2-01 in the field.
- Root cause: the "gate 2" hardening was scoped but never landed; the doc
  describes intended behavior the code does not provide.
- Direction: in `read_events`, detect a malformed trailing line (no
  terminating newline at EOF, or failed `serde_json::from_str` on the last
  line) and truncate it (with a `tracing::warn!`) rather than erroring. This
  both delivers the documented behavior and mitigates P2-01.
- Regression validation: a test that writes a valid event, appends a
  partial byte sequence with no newline, and asserts `read_events` returns
  the valid prefix and `rewrite_plan`/`get_plan` succeed.
- Validation reports: [V04-01](../validations/A-TSK-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | File-authority table (events authority, projections derived, no SQL/overlap) | yes | passed | [V01-01](../validations/A-TSK-01/V01-01.md) |
| V02 | Field-by-field EKO↔framework round-trip thin and lossless | yes | failed | [V02-01](../validations/A-TSK-01/V02-01.md) |
| V03 | Duplicate model/validator/CRUD search in app-core | yes | passed | [V03-01](../validations/A-TSK-01/V03-01.md) |
| V04 | Corrupt/partial state reconstruction | yes | failed | [V04-01](../validations/A-TSK-01/V04-01.md) |
| V05 | Historical-document drift | conditional | not-applicable | — |

V05 is not applicable: there is no prior A-TSK-01 report. The four module
docstrings (`store.rs:1-9`, `file_shadow.rs:1-7`, `revisioned_adapter.rs:1,24-25`,
`types.rs:917-920`) make falsifiable claims that are classified inline in
the Historical Claim Status table.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `store.rs:1-9` "file system (`events.jsonl` plus deterministic `plan.json` and `run-state.json` projections) is the source of truth" | current | Confirmed by V01-01: single `append_event_line` writer; projections regenerated by `rewrite_plan`; no SQL. |
| `store.rs:58-60` "event stream is authoritative; plan and execution files are deterministic read projections" | current as a *write* model; stale as a *read* guarantee | Write side confirmed (V01-01). Read side fails the "deterministic projection" guarantee when `events.jsonl` is corrupt — `load` makes projection reads depend on the event file (V04-01, P2-01). |
| `file_shadow.rs:1-7` "`events.jsonl` + `plan.json` is the read/write authority … SQL was retired" | current | Confirmed by V01-01/V03-01: no SQL anywhere; no parallel writer. |
| `file_shadow.rs:113-117` "crash mid-append can at worst lose the last partial line … gate 2 will truncate a partial tail" | stale (aspirational) | The truncation pass is not implemented; a partial tail surfaces as a fatal `Decode` error (V04-01, P3-02). |
| `revisioned_adapter.rs:1,24-25` "Thin EKO adapters … deliberately has no patch or validation logic" | current | Confirmed by V02-01: `EkoRevisionedTaskStore` is pass-through; `EkoTaskToolPolicy` does product policy only; no DAG/patch authority. Caveat: `commit_eko_task_plan` policy asymmetry (P3-01). |
| `types.rs:917-920` "shared `TaskStatus` remains authoritative and lossless. `TodoStatus` is derived only when building UI-facing projections" | current for in-memory adapter conversion; stale for the persisted event→projection path | `load_revisioned_task_graph` carries `TaskStatus` by identity (lossless). But the authoritative event stream stores `TodoStatus` and `rewrite_plan` regenerates `run-state.json` from it, so `Retrying`/`Paused` are not persisted losslessly (V02-01, P2-02). |
| F-TSK-01 handoff: "A-TSK-* may layer EKO product policy on top of this framework model, not beside it" | current (supported) | One projection set + one adapter pair; no parallel model/store/validator/CRUD (V03-01). |
| F-TSK-02: `PlanValidator` is the sole structural DAG validator | current (supported) | Zero application validator definitions; framework `PlanValidator` delegated at two test-only sites (V03-01). |
| B-REF-01 / C1: plan is a versioned artifact, not a runtime approval state machine | current (supported) | `TaskRunStatus` is a clean 6-state machine with no plan-approval states (`types.rs:480-527`); `TaskPlan` is a versioned artifact (`types.rs:835-848`). |

## Coverage And Uncertainty

- **Executor↔store status path not fully traced.** How the framework
  executor's `TaskStatus` (incl. `Retrying`/`Paused`) reaches
  `append_task_status_event` (or whether it bypasses it via
  `compare_and_commit`) determines whether P2-02 is a live data-loss path
  or a latent edge case. That wiring is owned by A-TSK-03; this report
  establishes the type-level lossiness regardless of reachability.
- **No executable tests run.** All four validations are static inspection +
  grep. P2-01 and P2-02 should be confirmed by targeted tests (append a
  malformed line; drive a task into `Retrying` and reload) — proposed as
  regression validation in each finding.
- **`atomic_write` parent-dir `fsync` not re-audited.** Consistent with the
  F-MEM-01 / A-STATE-01 recurring prior (no parent-dir fsync after rename);
  not re-raised here as it is not specific to the task-runtime authority
  question.
- **`task_execute` and the executor controller** are explicitly out of
  scope (A-TSK-03). Only the adapter/file-authority boundary was inspected.
- **Environmental limits:** none. The repository is clean at the audited
  commits.

## Handoff

- **Conclusions downstream tasks may rely on:**
  - The TaskRuntime has a single, unambiguous file authority
    (`events.jsonl`) with deterministic `plan.json` (spec) and
    `run-state.json` (execution) projections; no SQL; no competing writer
    (V01-01). Same-run writes serialize; different runs run in parallel.
  - EKO defines exactly one task/plan/todo projection set and one
    `RevisionedTaskStore` / `TaskToolPolicy` adapter pair; no parallel
    model, store, validator, or CRUD (V03-01). `TodoItem` is a derived UI
    projection with no store. AGENTS.md rule 6 holds.
  - The **spec** round-trip (EKO ↔ framework `TaskSpec` / `TaskPlanPatch`)
    is field-by-field lossless; the adapter is thin (no scheduling/
    validation authority) (V02-01).
  - The framework `PlanValidator` is the sole DAG validator; the application
    delegates to it (V03-01).
- **Reports downstream tasks must read:**
  - [V01-01](../validations/A-TSK-01/V01-01.md) for the file-authority table
    and write/read path inventory.
  - [V02-01](../validations/A-TSK-01/V02-01.md) for the field-by-field
    round-trip and the `Retrying`/`Paused` lossiness.
  - [V04-01](../validations/A-TSK-01/V04-01.md) for the corrupt-file
    reconstruction behavior.
- **Task-to-reference mapping:**
  - A-TSK-02 (task authoring tools) → may rely on the single projection set
    and the lossless spec round-trip; should verify `task_create/update/list`
    are thin product shells over the one revision service this adapter
    builds.
  - A-TSK-03 (executor boundary) → owns the open question of how the
    framework executor's `TaskStatus` reaches the store; its conclusion
    determines whether P2-02 is live data loss or latent. Must not
    introduce a second ready-frontier/retry/cancel authority.
  - A-TSK-04 (claims/revisions/recovery/terminal monotonicity) → owns the
    CAS/recovery mechanics; this report established the CAS path
    (`compare_and_commit_revisioned_task_graph`, `plan_locks`,
    `PlanConflict`) is sound at the adapter boundary.
- **Conditions that make this report stale:**
  - Any commit that adds a second task/plan/todo store, validator, or CRUD
    tool in `echo-agent-app-core` invalidates V03-01.
  - Any commit that lets a mutator write `plan.json`/`run-state.json`
    outside `rewrite_plan` invalidates V01-01.
  - Any change to `read_events` (skip-malformed), `FileTaskStore::load`
    (decouple from events), or `append_task_status_event` (persist framework
    `TaskStatus`) invalidates P2-01 / P2-02 respectively.
- **Follow-up task IDs (no fixes implemented in this review):**
  - P2-01 / P3-02 (corrupt-events resilience + partial-tail truncation)
    should be picked up by a robustness-focused cleanup task; the fix is
    localized to `read_events` and `FileTaskStore::load`.
  - P2-02 (execution-state lossiness) is gated on the A-TSK-03
    executor↔store wiring conclusion; the fix direction (persist framework
    `TaskStatus` in the event payload) is proposed above but should be
    confirmed against A-TSK-03.
  - P3-01 (`commit_eko_task_plan` policy asymmetry) is a localized adapter
    cleanup.
