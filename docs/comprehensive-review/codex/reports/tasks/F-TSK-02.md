# F-TSK-02: DAG validation and dependency analysis

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: `echo-agent` clean; `echo-agent-cli` later acquired concurrent tracked modifications confined to `web-frontend/src/generated/*.ts`, which this review did not read or modify; only Codex report artifacts were created by this task

## Question

Is there one structural validator and one dependency analysis for cycles, missing nodes, readiness, skip, blocked propagation, and recovery states?

## Scope

- Framework authoring compilation and structural validation: `planning/plan_spec.rs`, `planning/validator.rs`.
- Framework task models and dependency queries: `tasks/task.rs`, `tasks/dag.rs`, `tasks/manager.rs`, `tasks/runtime.rs`.
- Structural/recovery boundary of the full DAG kernel: `tasks/runtime_executor.rs`, `tasks/executor.rs`, `tasks/revisioned.rs`.
- Reachability only across `echo-agent` facade and EKO `RuntimeDagController` adapter.
- Static analysis, six exact crate tests, and standalone read-only probes linked to the already compiled crate.

## Out Of Scope

- Claim atomicity, concurrent revision safe points, dispatch cancellation draining, retry execution policy, hooks, and verifier behavior: `F-TSK-03`.
- Canonical task model and revision CRUD semantics beyond their validator boundary: `F-TSK-01`.
- EKO file/store authority and field-level adapter losslessness: `A-TSK-01`.
- Product scheduling/ownership policy and Subagent implementation: `F-SUB-02` and application task reviews.
- Workflow DAGs, plugin dependency graphs, security research, source fixes, workspace/all-feature builds, and network research.

## Inputs

- Root `AGENTS.md` supplied in task context.
- `docs/comprehensive-review/README.md`, `REPORTING.md`, `TASKS.md`, templates, and `codex/README.md`.
- Required dependency `codex/reports/tasks/F-TSK-01.md` was checked and did not exist. No conclusions were imported; the model/dependency boundary was independently reconstructed.
- No other reviewer directory or report was read.
- History was inspected directly at commits `a3dded2` and `100f44c`, not via historical review prose.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Immutable dependency structure, cycle/missing/self checks, ready frontier, skip/failure propagation, status classification, and bounded traversal belong in `echo-agent`; unrelated framework consumers need these invariants. |
| EKO product policy | File ownership, resource semaphores, attended Fail/Pause choice, recovery UX, and task projection remain in EKO. |
| Adapter boundary | EKO implements `RuntimeDagController`; it selects an ownership-safe subset and persists outcomes. It consumes framework frontier and propagation results and therefore cannot repair their missing semantics without becoming a second DAG authority. |
| Duplicate search | Searched names and behavior across both repositories: `TaskManager`, `PlanValidator`, `DagExecutionState`, cycle/topological/ready/blocked/unresolvable/skip/cancel/pause/recover, definitions, facade exports, constructors, and live callers. |
| Migration deletion | Keep one-wave APIs if useful, but make them call a single pure dependency analyzer. Delete `TaskManager`'s independent ready/wake/unresolvable rules and unsafe recursive dependency traversal after equivalent public behavior is exposed through the canonical analyzer. |

## Current Path

1. `PlanSpec::to_task_specs` compiles only required edges into canonical `TaskSpec.depends_on`, sorting and deduplicating each dependency list (`echo-orchestration/src/planning/plan_spec.rs:355`).
2. `PlanValidator::validate_task_snapshot` checks `spec.id == execution.task_id` and delegates structure to `validate_task_specs` (`planning/validator.rs:178`). The validator owns duplicate IDs, missing/self dependencies, cycle DFS, depth, size, retry, and required string checks (`planning/validator.rs:209`).
3. `TaskRevisionService::{create_prepared,apply_patch}` call `finalize_and_validate` before CAS commit (`tasks/revisioned.rs:894`, `tasks/revisioned.rs:934`, `tasks/revisioned.rs:990`). `RuntimeDagExecutor` revalidates every loaded snapshot before analysis (`tasks/runtime_executor.rs:211`).
4. Full-DAG framework execution and EKO execution both reach `RuntimeDagExecutor`; it rebuilds `DagExecutionState`, propagates direct failure blocks, decides terminal outcome, then computes the ready wave (`tasks/executor.rs:1409`, `tasks/runtime_executor.rs:232`, `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:1623`).
5. Public one-wave framework entry points still call `TaskManager::get_ready_tasks`; its readiness, wake, and unresolvable rules are separate from `DagExecutionState` (`tasks/executor.rs:561`, `tasks/executor.rs:1454`, `tasks/manager.rs:240`).
6. Recovery through `TaskExecutor::resume_from_store` reloads every nonterminal task, resets only `Running` to `Pending`, then invokes the canonical kernel (`tasks/executor.rs:1562`).

## Findings

### F-TSK-02-P1-01: Skipping a prerequisite strands every dependent in a stalled graph

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/runtime.rs:437`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/runtime.rs:480`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/revisioned.rs:547`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:271`
- Reachability: framework `task_update skip` and EKO recovery `Skip` persist `TaskStatus::Skipped` -> next kernel load counts that node as graph-resolved -> dependent still requires membership in `completed` -> frontier becomes empty -> kernel returns generic stall failure.
- Expected invariant: Skip must have one explicit DAG meaning: satisfy dependency, or transitively skip/block dependent nodes so the graph reaches a truthful terminal outcome.
- Observed behavior: `all_completed` treats skipped as resolved, while `ready_task_ids` does not. No propagation step exists; the sole skip executor test uses an isolated node.
- Impact: the documented attended recovery action “skip, then resume” can fail the entire run whenever the skipped task has required dependents; generic framework revision clients have the same defect.
- Root cause: completion and dependency satisfaction encode different meanings for the same `Skipped` state.
- Direction: define skip dependency policy in the sole generic analyzer; implement transitive propagation or resolved-dependency semantics and remove any adapter workaround. Preserve EKO's product choice only as policy input if needed.
- Regression validation: patch/recovery-skip `a` in `a -> b -> c`, resume, and assert all nodes/run reach the chosen explicit outcome without stall.
- Validation reports: [V03-02](../validations/F-TSK-02/V03-02.md), [V04-05](../validations/F-TSK-02/V04-05.md), [V04-10](../validations/F-TSK-02/V04-10.md), [V02-03](../validations/F-TSK-02/V02-03.md)

### F-TSK-02-P1-02: Persisted Paused and Retrying states fail or poll forever after restart

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/executor.rs:1562`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/runtime.rs:350`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:275`
- Reachability: persistent framework consumers call `TaskExecutor::with_task_store(...).resume_from_store()` -> reload every nonterminal record -> reset only Running -> `execute_all` -> runtime kernel.
- Expected invariant: restart normalization leaves each nonterminal task either deliberately paused or runnable with a live owner.
- Observed behavior: a persisted `Paused` task is in no `DagExecutionState` set and becomes generic stall failure. A persisted `Retrying` task is classified as externally in-flight; because restart has no external owner, the executor polls forever.
- Impact: restart during pause/retry can turn intended resumability into failure or an indefinitely nonterminal call.
- Root cause: persisted execution states have no single restart-normalization table shared with DAG classification.
- Direction: centralize restart normalization and state classification; explicitly return Paused; reset orphan Running/Retrying claims to Pending (or recover a durable owner) before dispatch.
- Regression validation: persistent store fixtures for Paused restart and Retrying restart; bound every run with timeout and assert exact terminal/status transitions and persisted events.
- Validation reports: [V03-04](../validations/F-TSK-02/V03-04.md), [V04-12](../validations/F-TSK-02/V04-12.md), [V04-17](../validations/F-TSK-02/V04-17.md)

### F-TSK-02-P1-03: Failure blocking is only one edge deep and can choose the wrong run disposition

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/runtime.rs:460`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/runtime.rs:485`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:235`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:1582`
- Reachability: failed runtime task -> kernel `blocked_by_failures` and `all_unfinished_failed_or_blocked` -> controller `failed_task_disposition`; EKO returns Fail only when the boolean is true, otherwise an attended run returns Pause.
- Expected invariant: a failed prerequisite transitively blocks all descendants before the kernel decides whether unfinished work is exhausted.
- Observed behavior: only direct children whose dependency ID is in `failed` are returned. In `a(Failed) -> b(Pending) -> c(Pending)`, only `b` is blocked and `c` makes `all_unfinished_failed_or_blocked` false; the kernel returns before reloading the newly persisted block.
- Impact: attended runs with fully exhausted multi-level chains can be reported Paused rather than Failed, and deeper task projections remain misleadingly Pending.
- Root cause: failure closure is computed as a one-hop predicate rather than a transitive graph operation; terminal disposition is evaluated in the same pre-propagation snapshot.
- Direction: compute failed/blocked transitive closure in the generic analyzer and return the normalized status effects plus exhaustion result from one operation.
- Regression validation: branched and three-level graphs with Failed/TimedOut roots; assert every descendant and the final Fail/Pause policy input.
- Validation reports: [V03-03](../validations/F-TSK-02/V03-03.md), [V04-06](../validations/F-TSK-02/V04-06.md), [V04-11](../validations/F-TSK-02/V04-11.md), [V02-03](../validations/F-TSK-02/V02-03.md)

### F-TSK-02-P1-04: Public dependency-chain query recurses forever on an admitted cycle

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/manager.rs:69`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/dag.rs:44`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/manager.rs:384`
- Reachability: any framework consumer can `TaskManager::add_task` two cyclic nodes and call the facade-exported `get_dependency_chain`; no validation gate lies between them.
- Expected invariant: malformed/cyclic graphs must return an error or bounded diagnostic; framework public queries must not panic/abort.
- Observed behavior: the recursive query has no visited or active-path set and follows `a -> b -> a` indefinitely until stack overflow. Canonical cycle detection is available but not used by this query.
- Impact: malformed input can terminate the embedding process, violating the repository's no-panic rule.
- Root cause: a legacy unguarded DFS survived the cycle-analysis consolidation.
- Direction: implement dependency-chain enumeration through the canonical guarded analyzer and return a typed cycle/missing-node error; delete the recursive helper.
- Regression validation: self-cycle, two-node cycle, missing node, and diamond DAG must all terminate with deterministic results.
- Validation reports: [V03-05](../validations/F-TSK-02/V03-05.md), [V01-02](../validations/F-TSK-02/V01-02.md), [V04-03](../validations/F-TSK-02/V04-03.md)

### F-TSK-02-P2-01: Public one-wave execution retains a second readiness authority

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/manager.rs:240`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/manager.rs:281`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/executor.rs:561`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/runtime.rs:437`
- Reachability: `TaskExecutor::execute_ready_tasks` and `execute_all_async` call `TaskManager` analysis directly; full `execute_all` and EKO call `DagExecutionState`.
- Expected invariant: one pure status-aware dependency analyzer defines ready, unresolved, blocked, skipped, cancelled, and deterministic frontier semantics for all execution shapes.
- Observed behavior: `TaskManager` independently implements `get_ready_tasks`, `wake_dependents`, and `has_unresolvable_pending`; `DagExecutionState` separately implements ready and failed blocking. Canonical frontier preserves snapshot order, while manager input comes from `DashMap` and only later sorts by priority, leaving equal-priority tie order unspecified.
- Impact: fixes and new statuses can land in one execution shape without the other; existing skip/recovery gaps already demonstrate that state policy is hard to reason about across paths.
- Root cause: full-DAG loop consolidation kept older public one-wave dependency logic rather than adapting it to a shared pure analyzer.
- Direction: keep public one-wave capabilities if framework consumers need them, but route them through the same analyzer over a deterministic snapshot; delete duplicated manager predicates and tests after porting coverage.
- Regression validation: feed identical snapshots to full-DAG and one-wave APIs and compare ready/blocked outcomes for every `TaskStatus`, equal priorities, missing nodes, and skip/failure chains.
- Validation reports: [V01-02](../validations/F-TSK-02/V01-02.md), [V02-02](../validations/F-TSK-02/V02-02.md), [V04-14](../validations/F-TSK-02/V04-14.md), [V05-01](../validations/F-TSK-02/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01-01 | Definition/concept search | yes | passed | [report](../validations/F-TSK-02/V01-01.md) |
| V01-02 | Duplicate-authority inspection | yes | failed | [report](../validations/F-TSK-02/V01-02.md) |
| V02-01 | Revision write reachability | yes | passed | [report](../validations/F-TSK-02/V02-01.md) |
| V02-02 | Framework execution reachability | yes | failed | [report](../validations/F-TSK-02/V02-02.md) |
| V02-03 | EKO consumer reachability | yes | passed | [report](../validations/F-TSK-02/V02-03.md) |
| V03-01 | Structural invariant inspection | yes | passed | [report](../validations/F-TSK-02/V03-01.md) |
| V03-02 | Skip edge inspection | yes | failed | [report](../validations/F-TSK-02/V03-02.md) |
| V03-03 | Failure closure inspection | yes | failed | [report](../validations/F-TSK-02/V03-03.md) |
| V03-04 | Pause/recovery inspection | yes | failed | [report](../validations/F-TSK-02/V03-04.md) |
| V03-05 | Malformed-graph traversal | yes | failed | [report](../validations/F-TSK-02/V03-05.md) |
| V04-01 | Missing/cycle unit test | yes | passed | [report](../validations/F-TSK-02/V04-01.md) |
| V04-02 | Guessed combined test filter | no | inconclusive | [report](../validations/F-TSK-02/V04-02.md) |
| V04-03 | Self-cycle unit test | yes | passed | [report](../validations/F-TSK-02/V04-03.md) |
| V04-04 | Completed dependency frontier test | yes | passed | [report](../validations/F-TSK-02/V04-04.md) |
| V04-05 | Isolated skip test | yes | passed | [report](../validations/F-TSK-02/V04-05.md) |
| V04-06 | Direct failure block test | yes | passed | [report](../validations/F-TSK-02/V04-06.md) |
| V04-07 | Persisted terminal detail test | yes | passed | [report](../validations/F-TSK-02/V04-07.md) |
| V04-08 | Probe compile v1 | yes | passed | [report](../validations/F-TSK-02/V04-08.md) |
| V04-09 | Pause-run probe | yes | failed | [report](../validations/F-TSK-02/V04-09.md) |
| V04-10 | Skip-dependent probe | yes | failed | [report](../validations/F-TSK-02/V04-10.md) |
| V04-11 | Transitive-failure probe | yes | failed | [report](../validations/F-TSK-02/V04-11.md) |
| V04-12 | Paused-snapshot probe | yes | failed | [report](../validations/F-TSK-02/V04-12.md) |
| V04-13 | Probe compile v2 | yes | passed | [report](../validations/F-TSK-02/V04-13.md) |
| V04-14 | Frontier order probe | yes | passed | [report](../validations/F-TSK-02/V04-14.md) |
| V04-15 | Status-independent validator probe | yes | passed | [report](../validations/F-TSK-02/V04-15.md) |
| V04-16 | Probe compile v3 | yes | passed | [report](../validations/F-TSK-02/V04-16.md) |
| V04-17 | Retrying-restart probe | yes | failed | [report](../validations/F-TSK-02/V04-17.md) |
| V05-01 | Direct git-history drift check | yes | passed | [report](../validations/F-TSK-02/V05-01.md) |
| V06-01 | Disk/build constraint check | yes | passed | [report](../validations/F-TSK-02/V06-01.md) |
| V06-02 | Concurrent process check | no | inconclusive | [report](../validations/F-TSK-02/V06-02.md) |
| V06-03 | Source dirty-state check | yes | passed | [report](../validations/F-TSK-02/V06-03.md) |
| V06-04 | Final source dirty-state observation | yes | passed | [report](../validations/F-TSK-02/V06-04.md) |
| V07-01 | Mistyped report-path read | no | inconclusive | [report](../validations/F-TSK-02/V07-01.md) |
| V07-02 | Validation artifact inventory | yes | passed | [report](../validations/F-TSK-02/V07-02.md) |
| V07-03 | Targeted report-content read | yes | passed | [report](../validations/F-TSK-02/V07-03.md) |
| V07-04 | Stale wording check | yes | passed | [report](../validations/F-TSK-02/V07-04.md) |
| V07-05 | Unscoped integrity audit | no | failed | [report](../validations/F-TSK-02/V07-05.md) |
| V07-06 | Link-scoped final integrity gate | yes | passed | [report](../validations/F-TSK-02/V07-06.md) |
| V20-01 | Primary status/restart/skip reconstruction | yes | failed | [report](../validations/F-TSK-02/V20-01.md) |
| V20-02 | Primary transitive-failure reconstruction | yes | failed | [report](../validations/F-TSK-02/V20-02.md) |
| V20-03 | Primary cycle/readiness-authority reconstruction | yes | failed | [report](../validations/F-TSK-02/V20-03.md) |
| V20-04 | Primary integrity script with special-variable bug | no | inconclusive | [report](../validations/F-TSK-02/V20-04.md) |
| V20-05 | Corrected primary integrity gate | yes | passed | [report](../validations/F-TSK-02/V20-05.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `a3dded2 refactor(tasks): unify dependency cycle analysis` | current | `tasks/dag.rs:5` delegates cycle/topological operations to `planning::validator`; [V05-01](../validations/F-TSK-02/V05-01.md). |
| `100f44c refactor(tasks): unify framework DAG execution` full-DAG loop | current | `TaskExecutor::execute_all` and EKO both use `RuntimeDagExecutor`; [V02-02](../validations/F-TSK-02/V02-02.md), [V02-03](../validations/F-TSK-02/V02-03.md). |
| `100f44c` implication that dependency traversal/failure propagation has one authority | regressed/incomplete | Manager one-wave analysis remains public, and canonical propagation has uncovered state gaps; [V01-02](../validations/F-TSK-02/V01-02.md). |

## Coverage And Uncertainty

- `F-TSK-01` is now primary-complete. Its model-authority result confirms this task's boundary: rich framework records are not dead, but their dependency/state/store authority must adapt to the revisioned model. The five findings here were independently reconstructed in V20-01..03.
- Six exact `echo_orchestration` tests passed. One guessed filter ran zero tests and is retained as inconclusive. No workspace build, Clippy, all-feature matrix, or application test ran because free disk fell below the repository's 50 GiB threshold.
- The Retrying finding proves orphan in-flight classification plus the recovery call graph; a bounded end-to-end timeout fixture is deferred to F-TSK-03.
- The dependency-chain stack overflow was not deliberately executed because doing so would crash the probe process; static control flow is conclusive, but platform-specific failure depth is unknown.
- The `TaskManager::pause_run` no-op was independently reproduced in V04-09, but its mutation/state-machine authority belongs to pending dependency F-TSK-01 and is intentionally not duplicated as an F-TSK-02 finding.
- Workflow DAG and other dependency domains were excluded after confirming they use different node models.
- No source fixes were attempted. The primary reconstruction and corrected link gate are complete, so this task is accepted as `complete`.

## Handoff

- F-TSK-03 must read this report and independently test: skip with dependents, a three-level failure chain and disposition, Paused snapshot outcome, Retrying restart bounded by timeout, and claim/revision interactions around those states.
- F-TSK-01 must decide whether `ManagedTask` remains a justified rich framework record or an adapter around canonical runtime tasks, and must own the `TaskManager::pause_run` no-op evidence in V04-09; this report only establishes duplicate dependency behavior, not deletion eligibility.
- A-TSK-01 must verify EKO's store conversion and recovery `Skip` transaction, using the framework semantics here as a dependency rather than adding application DAG logic.
- This report becomes stale if `TaskStatus`, `DagExecutionState`, `TaskManager` dependency APIs, `RuntimeDagExecutor` terminal branches, `TaskRevisionService` validation, or EKO's `RuntimeDagController` changes.
