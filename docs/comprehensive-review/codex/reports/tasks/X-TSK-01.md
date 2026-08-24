# X-TSK-01: Task graph and adapter conformance

> Status: complete
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework committed HEAD only; external CLI `Cargo.lock` excluded

## Question

Is there one revisioned TaskRun graph with lossless EKO projection and no
second validator, executor or store authority?

## Inputs And Boundary

- Complete F-TSK-01..03 and A-TSK-01..06 reports.
- Current committed TaskRevisionService/PlanValidator/RuntimeDagExecutor and
  current EKO revision adapter, TaskRuntime store/types/controller.
- Framework owns TaskSpec/TaskExecution/TaskStatus, validation, revision/CAS,
  DAG analysis, claims and generic retry/cancel/settlement safe points.
- EKO owns TaskRun file layout, DomainProfile, attended disposition,
  capabilities, Subagent dispatch, worktree/review/files and UI projection.

No dynamic field fixture was run under the explicit static-review restriction.

## Authority Map

```text
task_create/update/list
  -> TaskRevisionService + PlanValidator + RevisionedTaskStore CAS
  -> EkoRevisionedTaskStore -> TaskRuntimeStore file/event projections
  -> PlanTask/TaskPlan/TodoItem views

task_execute
  -> RuntimeDagExecutor
  -> EkoRuntimeDagController (ownership-safe subset + product dispatch)
  -> TaskRuntimeStore + Subagent/worktree/review projections
```

Positive conclusions:

- `TaskPlan` is a versioned product projection, `PlanTask` maps to one framework
  `Task`, and `TodoItem` is a derived UI projection, not a separate graph.
- `PlanTask -> Task -> PlanTask` preserves every specification/execution field,
  claim, status detail and EKO metadata; TaskPatch mapping is field-complete.
- EKO uses framework PlanValidator, TaskRevisionService and RuntimeDagExecutor.
  Replacing this stack would be a regression.

## Findings

### X-TSK-01-P1-01: EKO bootstrap side effects are outside the canonical revision transaction

- Priority: P1; confidence: high; layer: application policy/adapter.
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs:125-211,308-387`;
  A-TSK-02-P1-01/02.
- Reachability: `ensure_scope` creates and starts a TaskRun and binds attachments
  before TaskRevisionService validates and commits the initial graph.
- Expected invariant: rejected creation leaves no live run/product projection,
  and all authoring entry points use the same current capability policy.
- Observed behavior: canonical validation/CAS failure can leave an orphan
  Running run. Tool creation, planner commit and snapshot update also construct
  fresh/stale/default policies with different capability checks.
- Impact: one rejected or differently entered graph has product-visible side
  effects despite no committed canonical graph.
- Direction: application transaction/saga prepares the run, invokes one shared
  EkoTaskToolPolicy + revision service, then publishes/starts only after commit;
  compensate every prior side effect on failure. Delete alternate policy construction.
- Validation reports: [V03](../validations/X-TSK-01/V03-01.md).

### X-TSK-01-P1-02: Generic DAG semantics fail identically in framework and EKO execution

- Priority: P1; confidence: high; layer: framework.
- Evidence: committed PlanValidator/RuntimeDagExecutor and F-TSK-02 findings;
  EKO controller invocation in `echo-agent-app-core/src/tasks/task_runtime/executor.rs:1645`.
- Observed behavior: skipped prerequisites strand dependents; persisted
  Paused/Retrying fail or poll forever; failure blocking propagates one edge;
  public dependency traversal can recurse on an admitted cyclic graph.
- Expected invariant: the generic analyzer defines one coherent meaning for
  every persisted TaskStatus and every consumer observes it.
- Impact: EKO correctly reusing the framework reproduces these failures; an
  EKO workaround would create a second DAG authority.
- Direction: repair the sole framework analyzer/status table and route every
  one-wave/query path through it. Keep EKO as a policy/controller adapter.
- Validation reports: [V04](../validations/X-TSK-01/V04-01.md).

### X-TSK-01-P1-03: RuntimeDagExecutor does not yet own all generic retry, cancellation and settlement semantics

- Priority: P1; confidence: high; layer: framework/application boundary.
- Evidence: F-TSK-03 and A-TSK-03/04; current framework executor and EKO
  `tasks/task_runtime/executor.rs`.
- Observed behavior: framework claims can repeat ABA tokens, forced cancellation
  and wave early-return can leave claims unsettled, and controller retry can run
  multiple physical attempts under one claim. EKO additionally treats dispatch
  Err differently from returned failure, can overwrite cancellation with
  completion, and owns several fallible terminal setters that swallow errors.
- Impact: attempt identity, retry budget, cancellation and run terminal depend
  on which side of the adapter reports the failure.
- Direction: one framework attempt/claim outcome and one cancellation+settlement
  safe point; controller returns typed dispatch facts only. EKO retains resource
  limits, worktree/review and attended disposition hooks.
- Validation reports: [V05](../validations/X-TSK-01/V05-01.md).

### X-TSK-01-P1-04: The EKO revision store is not one crash-atomic projection of the canonical commit

- Priority: P1; confidence: high; layer: application persistence.
- Evidence: A-TSK-01 and A-TSK-04; `TaskRuntimeStore`/FileShadow/event rebuild.
- Observed behavior: event append and plan/run-state files can split across a
  crash; a partial JSONL tail disables the authority; malformed committed plan
  data is silently ignored into an empty revision; recovery persists Paused
  before clearing task claims and can become permanently unscannable.
- Expected invariant: successful TaskRevisionService commit has one recoverable
  durable result or a detectable/quarantined incomplete transaction.
- Impact: CAS can be correct in memory while restart observes a different graph
  or a Paused run with Running claims.
- Direction: one append/manifest generation with checksum/revision and atomic
  projection swap, strict tail recovery/quarantine, and rebuild/self-heal from
  the authoritative event set. X-STA-01 owns the broader persistence plan.
- Validation reports: [V06](../validations/X-TSK-01/V06-01.md).

### X-TSK-01-P2-05: Parallel graph/readiness/scheduling authorities remain around the canonical path

- Priority: P2; confidence: high; layer: cleanup.
- Evidence: committed ManagedTask/TaskManager/TaskStore and one-wave execution;
  EKO background TaskRun `depends_on`; F-TSK-01/02 and A-TSK-03.
- Observed behavior: legacy framework records own revision-0 CRUD/status/
  readiness/store rules, and EKO background runs add a 250ms polling dependency
  scheduler outside the PlanTask DAG.
- Impact: users and maintainers can select public paths with incompatible
  revision and readiness semantics even though the canonical path is live.
- Direction: adapt reasonable framework rich APIs over the revisioned graph and
  sole analyzer, then delete displaced authority; represent product sequencing
  in the canonical graph and delete cross-run polling.
- Validation reports: [V07](../validations/X-TSK-01/V07-01.md).

## Validation Matrix

| ID | Claim | Status | Report |
|---|---|---|---|
| V00 | Scope, dependency and commit isolation | passed | [V00-01](../validations/X-TSK-01/V00-01.md) |
| V01 | Authority call graph | passed | [V01-01](../validations/X-TSK-01/V01-01.md) |
| V02 | Field/metadata/patch round-trip | passed | [V02-01](../validations/X-TSK-01/V02-01.md) |
| V03 | Authoring bootstrap/commit transaction | failed | [V03-01](../validations/X-TSK-01/V03-01.md) |
| V04 | Shared validator/DAG status semantics | failed | [V04-01](../validations/X-TSK-01/V04-01.md) |
| V05 | Claim/retry/cancel/settlement boundary | failed | [V05-01](../validations/X-TSK-01/V05-01.md) |
| V06 | Revision persistence/crash continuity | failed | [V06-01](../validations/X-TSK-01/V06-01.md) |
| V07 | Forbidden parallel CRUD/readiness/scheduler | failed | [V07-01](../validations/X-TSK-01/V07-01.md) |
| V08 | Executable shared adapter fixture | not_run | [V08-01](../validations/X-TSK-01/V08-01.md) |
| V99 | Finding/link/isolation integrity | passed | [V99-01](../validations/X-TSK-01/V99-01.md) |

## Handoff

Keep the current revision adapter and field mappings. Fix generic DAG/claim/
settlement behavior in the framework, wrap EKO bootstrap/persistence in one
recoverable application transaction, then delete legacy graph and cross-run
scheduler authorities. Do not create new Plan/Todo CRUD or another validator.
