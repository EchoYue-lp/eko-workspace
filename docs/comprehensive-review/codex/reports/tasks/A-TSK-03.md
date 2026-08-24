# A-TSK-03: Task execution controller boundary

> Status: complete
> Reviewer: Codex primary reviewer (delegated evidence independently sampled)
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean at review start; Codex report files only

## Question

Does EKO inject only product policy into `RuntimeDagExecutor`, with no second ready-frontier, retry, cancellation, or stall loop?

## Scope

- Framework `RuntimeDagExecutor`, `RuntimeDagController`, ready-wave validation, safe points, cancellation/stall ownership, and current tests.
- EKO `execute_run`, `EkoRuntimeDagController`, `execute_runtime_plan`, resource/file ownership selection, dispatch resolution, completion drain, and run settlement.
- `task_execute` registration/entry path, GUI/TUI resume callers, and background planned-run service.
- Repository-wide duplicate searches for controller implementations, executor construction, ready/stall/dependency/retry/cancel loops.

## Out Of Scope

- File/event/projection atomicity and `SubagentRun` DTO authority: A-TSK-01.
- Authoring/bootstrap/capability policy: A-TSK-02.
- Durable claim identity, replay, recovery, and full terminal monotonicity: A-TSK-04.
- Framework claim ABA, cancellation-abandon callback, wave infrastructure settlement, generic ManagedTask retry, and retry math: F-TSK-03.
- Task tool presentation/success semantics across surfaces, source fixes, builds, tests, fixtures, and network.

## Inputs

- Root AGENTS.md; shared README/REPORTING/TASKS; Codex README and report templates.
- Authorized Codex dependency reports A-TSK-01 and F-TSK-03.
- Current source at the commits above and root MASTER-PLAN as a historical hypothesis.
- V00 records an accidental broad-search exposure of other reviewer path snippets. No exposed conclusion was used.

## Layering Decision

| Classification | Current answer |
|---|---|
| Generic mechanism | Dependency readiness, revision safe points, bounded waves, claim lifecycle, cancellation join/settlement, failure propagation, retry safe points, and stall outcomes belong in `echo-agent`. |
| EKO product policy | Attended disposition, review, worktree integration, file ownership, write/shell/LLM limits, UI/trace projection, and concrete dispatcher belong in `echo-agent-cli`. |
| Adapter boundary | `EkoRuntimeDagController` is structurally thin for DAG traversal, but retry classification and run settlement remain application responsibilities and are internally inconsistent. |
| Duplicate search | Searched controller/executor implementations, construction, `ready_task_ids`, `select_ready_wave`, `note_stalled`, dependency waits, retry counters/requeue, cancel tokens, completion drains, and transition callers across both repositories. |
| Migration deletion | Preserve `RuntimeDagExecutor` and EKO ownership selector. Delete the cross-TaskRun dependency polling relation if sequencing is represented in the canonical graph; delete inner/duplicate run transitions once one fallible settlement owner exists. |

No SQLite or online-service permission gate is involved.

## Current Path

```text
task_execute / GUI resume / TUI resume / background planned run
  -> execute_run (EKO outer completion + terminal projection)
     -> execute_runtime_plan
        -> EkoRuntimeDagController
        -> echo_agent::RuntimeDagExecutor::execute
           -> load/validate snapshot
           -> framework DagExecutionState ready frontier
           -> EKO conflict-free ownership subset
           -> framework bounded wave + claim + dispatch + resolution
           -> framework failure/cancel/stall outcome
     -> EKO complete_run_if_quiescent / event, trace, memory projection
```

The PlanTask path has one traversal kernel. Framework current source at `runtime_executor.rs:196-449` owns loop/safe-point/readiness/wave/cancel/stall; EKO selection at `executor.rs:1125-1145` and `:1265-1282` filters the already-computed frontier only. The controller is constructed once at `:1623-1655` and is reached from registered/live callers ([V01](../validations/A-TSK-03/V01-01.md), [V02](../validations/A-TSK-03/V02-01.md)).

The boundary nevertheless has four deviations. Background submission stores a separate `TaskRun -> TaskRun` dependency list and polls it before the framework path. Dispatch errors and returned execution failures use different retry policies. The outer completion drain does not arbitrate pause/cancel intent. Run terminal transitions are split across inner adapter and outer executor, with store errors discarded.

## Findings

### A-TSK-03-P1-01: Dispatch errors bypass the declared retry budget

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:1356`, `:1396`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:1012`
- Reachability: any planned-run entry -> framework executor -> EKO dispatch -> `resolve_dispatch`; provider/resource/worktree/timeout failures enter `Err`, while a completed Subagent with failed structured result enters `ExecutionFailed`.
- Expected invariant: typed retryable execution failures consume `max_retries` uniformly and requeue at a framework safe point with a fresh claim; cancellation and permanent policy errors do not retry.
- Observed behavior: every dispatch `Err` becomes terminal Failed/TimedOut/Cancelled without inspecting `max_retries`. Only an `Ok` dispatch later assessed as `ExecutionFailed` uses the retry budget and returns Pending.
- Impact: transient provider, permit, timeout, or execution-path failures can fail a multi-task run on the first attempt even when the task declares remaining retries; equivalent self-reported execution failures receive all attempts.
- Root cause: dispatch errors are flattened to `ReactError`/Subagent status before a typed recoverability decision, and retry policy exists only in the successful-output branch.
- Direction: introduce one typed application failure classification and one resolution helper that either requeues or settles. Keep retry at the framework safe point and one durable claim per physical attempt; do not add an inner retry loop.
- Regression validation: for the same max_retries=2 task, inject retryable provider error, timeout, permanent policy error, cancel, and returned remaining-work; assert only retryable cases run three distinct claimed attempts.
- Validation reports: [V05](../validations/A-TSK-03/V05-01.md), [V08](../validations/A-TSK-03/V08-01.md)

### A-TSK-03-P1-02: Completion drain can lose cancellation or spin forever after pause

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:366`, `:393`, `:420`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:450`, `:573`, `:595`
- Reachability: final framework wave persists all tasks -> kernel returns Completed -> outer drain reloads plan -> concurrent `request_pause` or `request_cancel` before `complete_run_if_quiescent`.
- Expected invariant: completion, cancellation, and pause have one atomic winner, and every non-winning branch returns/reloads with cancellation awareness rather than spinning.
- Observed behavior: the outer loop checks neither its cancel token nor current run status. A pause makes the completion CAS return false forever and the no-await loop immediately repeats. An active cancel only cancels the token while leaving Running, so the completion CAS can still commit Completed.
- Impact: a user pause at finalization can pin an executor in a hot loop; a cancellation API can report success while the run becomes Completed and emits completion side effects.
- Root cause: the revision-aware completion drain was added outside the framework outcome lifecycle without including interruption intent/state in its arbitration.
- Direction: atomically gate completion on latest revision, Running state, and no durable interruption intent; check token/status at every drain boundary and map false completion to a typed interruption or awaited reload. Delete the unconditional synchronous continue for non-Running terminal/interrupted states.
- Regression validation: barriers immediately after kernel Completed and before completion CAS; race pause/cancel/plan patch and assert one monotonic terminal result, no spin, and no contradictory event/memory write.
- Validation reports: [V06](../validations/A-TSK-03/V06-01.md), [V08](../validations/A-TSK-03/V08-01.md)

### A-TSK-03-P1-03: Run terminal settlement is split and persistence failures are ignored

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:467`, `:499`, `:527`, `:570`, `:643`, `:1656`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:517`
- Reachability: every Failed/Paused/Cancelled outcome and executor error from every planned-run entry traverses these branches.
- Expected invariant: controller resolves task claims; one application boundary durably settles the run exactly once, propagates failure, then emits a matching event/trace/outcome.
- Observed behavior: `execute_runtime_plan` transitions Failed/Paused best-effort; `execute_run` transitions Failed again, performs separate Paused cleanup, and best-effort cancels tasks/run. All transition/status/note errors are ignored while events/traces/outcomes continue. The normal second Failed transition is illegal because Failed cannot transition to Failed.
- Impact: callers and UI/trace/memory can observe a terminal outcome whose durable TaskRun/tasks did not settle; resume/recovery then sees stale Running state or contradictory terminal records.
- Root cause: terminal ownership is divided between adapter conversion, outer orchestration, and helpers rather than one fallible settlement transaction.
- Direction: create one application run-settlement function after `RuntimeDagOutcome`, propagate its typed result, and emit projections only after it succeeds. Delete `execute_runtime_plan`'s run transition and duplicate best-effort branches after migration. A-TSK-04 should add claim fencing/monotonic recovery details without creating another owner.
- Regression validation: inject store failure at each task cleanup/run transition/note boundary for every outcome; assert no terminal event/result is returned until durable state matches and retry is idempotent.
- Validation reports: [V07](../validations/A-TSK-03/V07-01.md), [V08](../validations/A-TSK-03/V08-01.md)

### A-TSK-03-P2-01: Background TaskRun dependencies form a second scheduling authority

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/tasks.rs:60`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/service.rs:229`, `:273`, `:359`, `:556`, `:763`
- Reachability: Tauri `submit_task(kind=research, depends_on=...)` -> `BackgroundTaskService::submit_with_options` -> trigger metadata -> `start_run_driver` -> `wait_for_dependencies`; startup/resume repeats it.
- Expected invariant: task relations use the one revisioned TaskRun graph and its framework structural validation/executor; application adapters do not own another readiness poll.
- Observed behavior: a separate list relates whole TaskRuns, persists outside the plan, and polls their status every 250 ms before entering the framework executor. Pending/Running/Paused dependencies wait without deadline/stall classification. The relation is invisible to PlanTask validation, graph updates, and framework failure propagation.
- Impact: identical dependency semantics differ by entry path; dependent background runs can remain Pending indefinitely behind Paused/stuck prerequisites, and graph/status tooling cannot inspect or revise the relation as part of the canonical plan.
- Root cause: compatibility background-task metadata retained orchestration semantics after PlanTask scheduling converged on `RuntimeDagExecutor`.
- Direction: model required sequencing as tasks/nodes in the canonical revisioned graph, or remove external `depends_on` from this surface if cross-run composition is not a product requirement. Delete `wait_for_dependencies` and trigger-metadata dependency replay after migration; keep TaskRun-level concurrency admission as EKO policy.
- Regression validation: submit dependency success/failure/pause/cancel/restart chains through each surface; assert one graph authority, validated relationships, bounded terminal behavior, and no polling loop outside the kernel.
- Validation reports: [V03](../validations/A-TSK-03/V03-01.md), [V08](../validations/A-TSK-03/V08-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Reviewer-isolation incident disclosure | yes | inconclusive | [V00](../validations/A-TSK-03/V00-01.md) |
| V01 | Framework/application ownership call graph | yes | passed | [V01](../validations/A-TSK-03/V01-01.md) |
| V02 | Registration and runtime reachability | yes | passed | [V02](../validations/A-TSK-03/V02-01.md) |
| V03 | Scheduling/dependency loop duplicate search | yes | failed | [V03](../validations/A-TSK-03/V03-01.md) |
| V04 | Controller callback responsibility matrix | yes | passed | [V04](../validations/A-TSK-03/V04-01.md) |
| V05 | Retry classification and safe-point trace | yes | failed | [V05](../validations/A-TSK-03/V05-01.md) |
| V06 | Completion/pause/cancel interleaving trace | yes | failed | [V06](../validations/A-TSK-03/V06-01.md) |
| V07 | Terminal settlement responsibility trace | yes | failed | [V07](../validations/A-TSK-03/V07-01.md) |
| V08 | Existing test inventory and edge-case coverage | yes | failed | [V08](../validations/A-TSK-03/V08-01.md) |
| V09 | Old framework hash to current-anchor reconstruction | yes | passed | [V09](../validations/A-TSK-03/V09-01.md) |
| V10 | Basic DAG executable validation | policy-deferred | not_run | [V10](../validations/A-TSK-03/V10-01.md) |
| V11 | Report/link/executor/source integrity gate | yes | attempt 1 inconclusive; attempt 2 passed | [A1](../validations/A-TSK-03/V11-01.md), [A2](../validations/A-TSK-03/V11-02.md) |
| V30 | Primary source sampling and acceptance | yes | passed | [V30](../validations/A-TSK-03/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN M13: `RuntimeDagExecutor` is EKO's only dynamic PlanTask loop | current for PlanTask DAG | [V01](../validations/A-TSK-03/V01-01.md), [V09](../validations/A-TSK-03/V09-01.md) |
| MASTER-PLAN M13: application retains only product policy | incomplete | retry and terminal lifecycle deviations in [V05](../validations/A-TSK-03/V05-01.md)-[V07](../validations/A-TSK-03/V07-01.md) |
| MASTER-PLAN M13: single Task/batch/dependency DAG uses one TaskRun graph | regressed/incomplete | cross-TaskRun relation loop in [V03](../validations/A-TSK-03/V03-01.md) |
| F-TSK-03 framework findings at old hash apply to current framework source | current anchors, canonical there | no relevant file changed and current code re-read in [V09](../validations/A-TSK-03/V09-01.md) |

## Coverage And Uncertainty

- This was pure static review. No Cargo/rustc/test/build/fixture/network process ran; V10 is explicitly `not_run`.
- A narrow completion-interruption window and store failures were source-traced but not dynamically injected. They remain required implementation regressions, not uncertainty about branch ordering.
- `task_execute` returns ordinary successful ToolResult text for Failed/Paused/Cancelled outcomes. Cross-surface tool semantics are deferred rather than promoted here because this task owns execution-controller authority.
- A-TSK-01 file-authority defects can amplify terminal-write failures; they remain canonical in A-TSK-01.
- F-TSK-03 framework defects remain canonical and were not duplicated.

## Handoff

- Preserve the single framework ready-frontier and EKO file-ownership subset; neither requires architectural replacement.
- Fix application issues in order: completion/interruption arbitration and one fallible run settlement; typed retry classification; remove/migrate cross-TaskRun dependency polling.
- A-TSK-04 must consume P1-02/P1-03 when auditing claim/revision/recovery terminal monotonicity and must not add another cleanup owner.
- This report becomes stale if `execute_run`, `EkoRuntimeDagController`, `execute_runtime_plan`, TaskRuntime cancellation/completion methods, background dependencies, or framework task runtime files change.
