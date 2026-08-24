# F-TSK-01: Canonical task model and revision tools

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only scope)
> Worktree state: clean

## Question

Is `TaskSpec + TaskExecution + TaskStatus` the sole generic dynamic task
model with coherent revisioned `task_create/update/list` semantics?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-orchestration/src/tasks/task.rs` — `TaskType`,
  `TaskInput`/`InputType`, `TaskOutput`/`OutputType`, `ContextScope`,
  `RiskLevel`, `VerificationSpec`/`VerificationType`/`FallbackStrategy`/
  `CheckpointPolicy`, `TaskAttempt`/`AttemptStatus`, `Evidence`/`EvidenceType`,
  `FileChange`/`ChangeType`, `Artifact`/`ArtifactType`, `CommandRecord`,
  `VerificationResult`, `TaskState`, `ManagedTask` (builder).
- `echo-agent/echo-orchestration/src/tasks/revisioned.rs` —
  `TaskGraphExecutionMode`, `TaskGraphContext`, `RevisionedTaskGraph`,
  `TaskDraft`, `TaskCreateInput`, `TaskSpecPatch`, `TaskPlanPatchOp`,
  `TaskPlanPatch`, `TaskPlanPatchInputOp`, `TaskUpdateInput`,
  `TaskPatchEffects`, `TaskGraphCommit`, `RevisionedTaskStoreError`,
  `TaskPolicyError`, `TaskRevisionError`, `RevisionedTaskStore` trait,
  `TaskToolPolicy` trait, `DefaultTaskToolPolicy`, `TaskRevisionService`.
- `echo-agent/echo-orchestration/src/tasks/task_tools.rs` — `TaskCreateTool`,
  `TaskUpdateTool`, `TaskListTool`, `build_task_tools`.
- `echo-agent/src/tasks.rs` — root re-export and `register_task_tools()`.
- Cross-repo duplicate search for `todo_write`, `plan_create`, `plan_patch`,
  `plan_execute` across the whole `echo-agent` repository.

## Out Of Scope

- Application-layer task runtime (`echo-agent-cli/.../task_runtime/`) and any
  EKO-specific `task_execute` extension — deferred to application task
  review (A-TSK-*).
- DAG validation, dependency analysis, cycle/missing-node checks — deferred
  to `F-TSK-02`.
- Runtime DAG execution, safe points, bounded concurrency — deferred to
  `F-TSK-03`.
- The `RevisionedTaskStore` concrete implementations' internal behavior
  (only the trait contract is in scope).

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — especially the "single authority API" rule 6, the
    `todo_write` removal note, the framework-vs-application layering gate,
    and the "only Subagents, no Workers" terminology rule.
  - `docs/comprehensive-review/REPORTING.md`.
  - `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
- Dependency task reports read:
  - `F-CORE-01` (this reviewer) — for the identity/error typing baseline and
    the convention that root `echo-agent/src/*.rs` files are thin re-exports.
  - `B-REF-01` (this reviewer) — for convergence C1 (plan is artifact, not
    runtime approval state machine) and C5 (isolation-first delegation with
    bounded caps, matching `TaskRun→PlanTask→SubagentRun`).
- Historical documents treated as hypotheses: none.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. The task model (`TaskSpec`/`TaskExecution`/`TaskState` + verification/evidence/artifact types), the revisioned graph (`RevisionedTaskGraph`), the store trait (`RevisionedTaskStore`), the policy trait (`TaskToolPolicy`), the revision service (`TaskRevisionService`), and the three tools (`task_create`/`task_update`/`task_list`) describe generic dynamic-task concepts any `echo-agent` consumer may need. They live correctly in `echo-orchestration` (V01 confirms single definition site; root `echo-agent/src/tasks.rs:10-13` only re-exports). |
| EKO product policy | None at this layer. EKO's `task_execute` extension belongs to the application; the framework exposes only `task_create`/`task_update`/`task_list` (AGENTS.md rule 6). |
| Adapter boundary | The three tools in `task_tools.rs` are a thin adapter: they expose JSON Schema (via the `Tool` trait) and forward arguments to `TaskRevisionService`. They hold no independent store, state machine, or validator (V03). |
| Duplicate search | Searched names (whole `echo-agent` repo): `todo_write`, `plan_create`, `plan_patch`, `plan_execute`, `TaskRevisionService`, `RevisionedTaskStore`, `RevisionedTaskGraph`, `TaskCreateTool`, `TaskUpdateTool`, `TaskListTool`. Result: no parallel task/plan/todo CRUD. `todo_write` survives only in a test assertion (`builder.rs:1181`); `plan_create`/`plan_patch`/`plan_execute` have zero matches. Single authority: `TaskRevisionService` + `RevisionedTaskStore` (V01). |
| Migration deletion | No migration proposed. No deletion candidate identified at this layer — the model is live and singular. |

## Current Path

Verified task-model data flow at commit `9b0e0fa`:

1. **Model definition.** `echo-orchestration/src/tasks/task.rs` defines the
   canonical task model. `TaskType` (14 variants incl. `background`) tags
   the task kind. `ManagedTask` (line 292) is the builder: `new()` followed
   by `with_dependencies()`, `with_priority()`, `with_timeout()`,
   `with_max_retries()`, `with_assigned_agent()`, `with_tags()`,
   `with_execute_fn()`, `with_metadata()`, then `task_spec()`,
   `task_execution()`, `to_task()`. Supporting types: `TaskState` (275),
   `TaskAttempt` (171) / `AttemptStatus` (184), `Evidence` (194) /
   `EvidenceType` (204) / `FileChange` (214) / `ChangeType` (224),
   `Artifact` (233) / `ArtifactType` (244) / `CommandRecord` (255) /
   `VerificationResult` (265), `VerificationSpec` (98) / `VerificationType`
   (134) / `FallbackStrategy` (149) / `CheckpointPolicy` (161), `TaskInput`
   (30) / `InputType` (40) / `TaskOutput` (49) / `OutputType` (59),
   `ContextScope` (70) / `RiskLevel` (86).

2. **Revisioned layer.** `revisioned.rs` wraps the model in a versioned
   graph. `RevisionedTaskGraph` (42) holds versioned state;
   `TaskGraphContext` (31) and `TaskGraphExecutionMode` (23) parameterize
   access. Mutations are expressed as `TaskCreateInput` (67) → `TaskDraft`
   (49) for creation, and `TaskUpdateInput` (148) carrying `TaskSpecPatch`
   (78) + `TaskPlanPatch` (117) for edits. Plan/todo sub-edits use
   `TaskPlanPatchOp` (94) (canonical) and `TaskPlanPatchInputOp` (125)
   (tool-input deserialization). Each mutation returns `TaskPatchEffects`
   (156) and is recorded as a `TaskGraphCommit` (167), bumping the graph
   revision.

3. **Single authority.** `TaskRevisionService` (line 674) is the only
   mutator. It composes a `RevisionedTaskStore` (trait at 247, `Send + Sync`)
   for persistence and a `TaskToolPolicy` (trait at 270, default
   `DefaultTaskToolPolicy` at 318) for permission checks. No other code path
   writes to the store or bumps the revision (V02).

4. **Tool surface.** `build_task_tools(service: Arc<TaskRevisionService>)`
   (`task_tools.rs:205`) returns exactly three `Box<dyn Tool>`:
   `TaskCreateTool` (15), `TaskUpdateTool` (82), `TaskListTool` (143). Each
   holds an `Arc<TaskRevisionService>`, exposes a JSON Schema via the `Tool`
   trait, and forwards execution to the service. No tool performs direct
   store writes (V03).

5. **Registration.** `register_task_tools()` (`echo-agent/src/tasks.rs:18`)
   installs the three tools on an agent. The only production caller is
   `echo-agent/src/agent/react/builder.rs:963`.

6. **Error semantics.** Stale-update / concurrent-modification surfaces as
   `TaskRevisionError` (200); policy violations surface as `TaskPolicyError`
   (190); storage errors surface as `RevisionedTaskStoreError` (175). The
   service converts store/policy errors into `TaskRevisionError` for tool
   callers, keeping the stale-update path typed end-to-end (V04).

7. **Stable public surface.** Root `echo-agent/src/tasks.rs:10-13` brings
   `echo_orchestration::tasks::*` into `echo_agent::tasks`; `lib.rs:321`
   re-exports the task tools into the framework prelude.

## Findings

The headline result is positive: the framework exposes exactly one
canonical, revisioned task model with one service, one store contract, and
one tool trio — no parallel plan/todo/task CRUD. This satisfies AGENTS.md
rule 6. The single recorded finding is a P3 readability concern.

### F-TSK-01-P3-01: `TaskPlanPatchOp` and `TaskPlanPatchInputOp` are two same-family enums whose proximity invites confusion

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/tasks/revisioned.rs:94` —
    `pub enum TaskPlanPatchOp` (canonical plan/todo edit op consumed by
    `TaskRevisionService`).
  - `echo-agent/echo-orchestration/src/tasks/revisioned.rs:125` —
    `pub enum TaskPlanPatchInputOp` (parallel enum used to deserialize tool
    input before normalization into `TaskPlanPatchOp`).
  - `echo-agent/echo-orchestration/src/tasks/revisioned.rs:117` —
    `TaskPlanPatch` ties the two together.
- Reachability: both enums are live. `TaskPlanPatchOp` is consumed inside
  `TaskRevisionService`'s update path; `TaskPlanPatchInputOp` is reached via
  `TaskUpdateTool`'s argument deserialization (V03).
- Expected invariant: one semantic edit-op family should have one canonical
  type, or the split between "wire input" and "canonical op" should be
  self-evident from naming/docs.
- Observed behavior: two enums with near-identical names
  (`...PatchOp` vs `...PatchInputOp`) model adjacent stages of the same
  plan/todo edit. The split is intentional (input shaping vs. canonical op)
  and the conversion is lossless, but nothing in the type names or comments
  signals the relationship. A reader can reasonably assume a duplicate
  authority where none exists.
- Impact: low. No correctness defect — both types feed the same
  `TaskRevisionService` path. Maintenance hazard: a future contributor
  adding a new op must update both enums and the normalization, and the
  naming makes the coupling easy to miss.
- Root cause: the wire-input vs. canonical-op distinction was encoded as
  two public enums without a doc/comment explaining the relationship.
- Direction: either (a) collapse `TaskPlanPatchInputOp` into
  `TaskPlanPatchOp` if serde attributes can carry the input-vs-canonical
  difference (preferred — removes the coupling entirely), or (b) keep both
  but add a doc comment on each pointing to the other and explaining the
  normalization step. Either way, the `Tool`-trait schema (V03) must still
  round-trip.
- Regression validation: `cargo test -p echo_orchestration` must pass;
  additionally a `TaskUpdateTool` round-trip test (input JSON →
  `TaskPatchEffects`) should be added if one does not exist, to lock the
  normalization.
- Validation reports: [V02](../validations/F-TSK-01/V02-01.md),
  [V03](../validations/F-TSK-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Duplicate task/plan/todo model search | yes | passed | [V01-01](../validations/F-TSK-01/V01-01.md) |
| V02 | Transition/revision table (single authority) | yes | passed | [V02-01](../validations/F-TSK-01/V02-01.md) |
| V03 | Tool schema round-trip via `Tool` trait | yes | passed | [V03-01](../validations/F-TSK-01/V03-01.md) |
| V04 | Stale-update / revision-conflict error semantics | conditional (yes — error semantics in scope) | passed | [V04-01](../validations/F-TSK-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | No historical document is reused for a claim in this report. |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md rule 6: "旧的进程全局 `todo_write` 已由框架直接删除" | current (corroborated) | `todo_write` survives only in a test assertion at `echo-agent/src/agent/react/builder.rs:1181`. V01-01. |
| AGENTS.md rule 6: "不得重新引入 `plan_create/plan_patch/plan_execute` 或其它平行任务 CRUD" | current (corroborated) | Zero matches for all three names across the `echo-agent` repository. V01-01. |
| AGENTS.md rule 6: framework default is `task_create/task_update/task_list`; EKO adds `task_execute` | current (corroborated) | `build_task_tools` (`task_tools.rs:205`) returns exactly the three framework tools. V03-01. |
| B-REF-01-P1-01 / C1: plan is a versioned artifact, not a runtime approval state machine | current (supported) | `RevisionedTaskGraph` (42) is versioned; approval is not a run-state column; conflict is resolved by revision (`TaskRevisionError`, V04-01). |
| B-REF-01-P1-03 / C5: isolation-first delegation with bounded caps matches `TaskRun→PlanTask→SubagentRun` | current (supported, adjacent) | The task model is compatible with the C5 subagent shape; subagent execution itself is audited under F-SUB-* / F-TSK-03, not here. |

## Coverage And Uncertainty

- **Store implementations not exercised at runtime.** Only the
  `RevisionedTaskStore` trait contract was inspected, not the behavior of
  any concrete implementation under concurrent updates. A runtime test
  asserting a `TaskRevisionError` on conflicting updates would strengthen
  V04; deferred to a follow-up test task.
- **`TaskPlanPatchOp` / `TaskPlanPatchInputOp` normalization not traced
  line-by-line.** V03 confirms the tool delegates to the service and that
  both enums feed the same path, but the per-variant normalization mapping
  was not enumerated. The losslessness claim rests on the absence of data
  between two same-shaped enums and on the single-authority invariant.
- **Application-layer `task_execute` out of scope.** This report does not
  assess the EKO `task_execute` extension or the application `task_runtime`;
  those belong to A-TSK-*.
- **Environmental limits:** none. The repository is clean at the audited
  commit.

## Handoff

- **Conclusions downstream tasks may rely on:**
  - There is exactly one canonical, revisioned task model in
    `echo-orchestration::tasks`. `TaskRevisionService` is the sole mutator;
    `RevisionedTaskStore` is the sole persistence contract; the three
    framework tools are `task_create` / `task_update` / `task_list`.
  - The legacy `todo_write` is gone from production (test-only); no parallel
    `plan_create/plan_patch/plan_execute` exists. AGENTS.md rule 6 holds.
  - Stale/concurrent updates are handled by a typed revision-conflict error
    (`TaskRevisionError`), not by an approval state machine — consistent
    with B-REF-01 C1.
- **Reports downstream tasks must read:**
  - [V01-01](../validations/F-TSK-01/V01-01.md) for the duplicate-model
    search result and the full type inventory.
  - [V02-01](../validations/F-TSK-01/V02-01.md) for the transition/revision
    table and the single-authority proof.
- **Task-to-reference mapping:**
  - F-TSK-02 (DAG validation) → builds on this canonical task model; should
    treat `RevisionedTaskGraph` as the sole graph authority.
  - F-TSK-03 (runtime DAG execution) → must not introduce a parallel task
    CRUD; subagent shape should follow B-REF-01 C5.
  - A-TSK-* (application task runtime / `task_execute`) → may layer EKO
    product policy on top of this framework model, not beside it.
- **Conditions that make this report stale:**
  - Any commit that reintroduces `todo_write` into production, or adds
    `plan_create`/`plan_patch`/`plan_execute` tools, invalidates V01 and the
    primary conclusion.
  - Any commit that lets a tool bypass `TaskRevisionService` to write to the
    store directly invalidates V02/V03 and the single-authority conclusion.
  - Any change to `TaskPlanPatchOp` / `TaskPlanPatchInputOp` invalidates
    finding F-TSK-01-P3-01.
- **Follow-up task IDs (no fixes implemented in this review):**
  - A test-only follow-up (not a new review task) to add a conflicting-update
    runtime test asserting `TaskRevisionError`.
  - The readability cleanup in F-TSK-01-P3-01 is a code change, deferred to
    a normal maintenance commit, not a review task.
