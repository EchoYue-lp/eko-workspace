# F-TSK-02: Plan/task DAG and dependency validation authority

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only scope)
> Worktree state: clean

## Question

Is there exactly one authority for plan/task DAG structural validation
(cycle detection, topological order, missing-node and self-dependency checks),
with no parallel DFS/validator implementation?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-orchestration/src/planning/validator.rs` —
  `PlanValidator` struct (14), `validate(plan: &PlanSpec) -> ValidationReport`
  (50), `validate_task_snapshot(tasks: &[Task]) -> Result` (184),
  `validate_task_specs(tasks: &[TaskSpec]) -> Result` (210); free functions
  `task_dependency_cycles()`, `task_topological_order()`.
- `echo-agent/echo-orchestration/src/planning/plan_spec.rs` —
  `PlanSpec` (18), `PlanTaskSpec` (60), `Dependency` (229),
  `DependencyType` (243), `Milestone` (254),
  `PlanVerificationStrategy` (278), `Complexity` (294),
  `PlanSpec::to_task_specs()` (359).
- `echo-agent/echo-orchestration/src/tasks/dag.rs` — `TaskManager` methods
  delegating to `planning::validator`: `detect_circular_dependencies()`,
  `get_topological_order()`, `visualize_dependencies()` (Mermaid graph),
  `get_dependency_chain()` (recursive chain).
- Cross-repo duplicate search for `PlanValidator`, `task_dependency_cycles`,
  `task_topological_order`, `dfs`, `cycle`, `topological` across the whole
  `echo-agent` repository.

## Out Of Scope

- The canonical task model (`TaskSpec`/`TaskExecution`/`TaskState`,
  `RevisionedTaskGraph`) — audited under `F-TSK-01`.
- Runtime DAG execution, safe points, bounded concurrency, subagent
  scheduling — deferred to `F-TSK-03`.
- Application-layer `task_execute` and EKO product policy on top of the plan
  artifact — deferred to `A-TSK-*`.
- Persistence/store behavior of the revisioned graph (only the validator's
  contract is in scope).

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — especially rule 6 ("task relationship has one
    authority API"; `TaskPlan` is editable/reviewable versioned artifact
    only, no independent store/state-machine/executor), the framework-vs-
    application layering gate, and the "first search whether it already
    exists" pre-implementation gate.
  - `docs/comprehensive-review/REPORTING.md`.
  - `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
- Dependency task reports read:
  - `F-TSK-01` (this reviewer) — for the canonical revisioned task model
    (`RevisionedTaskGraph` as the sole graph authority) that this DAG
    validator operates over, and for the convention that root re-export
    files are thin.
  - `B-REF-01` (this reviewer) — for convergence C1 (plan is a versioned
    artifact, not a runtime approval state machine); the plan-spec layer
    audited here is precisely that artifact.
- Historical documents treated as hypotheses: none.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. Structural DAG validation (cycles, topological order, missing/self dependencies) over a set of task specs is generic: any `echo-agent` consumer that builds a dependency graph needs it. It lives correctly in `echo-orchestration::planning::validator` (V01 confirms single definition site). |
| EKO product policy | None at this layer. `PlanValidator` operates on `PlanSpec`/`TaskSpec`/`Task` shapes that carry no EKO product decisions; the application may layer review/approval policy on top, but the structural rules are framework-generic. |
| Adapter boundary | `tasks/dag.rs` `TaskManager` methods are a thin delegation adapter: each method forwards the spec slice to the corresponding free function in `planning::validator` and shapes the result (e.g. Mermaid rendering). They hold no independent DFS/cycle state or validator (V01). |
| Duplicate search | Searched names (whole `echo-agent` repo): `PlanValidator`, `task_dependency_cycles`, `task_topological_order`, `validate_task_specs`, `validate_task_snapshot`, `dfs`, `cycle`, `topological`, `tarjan`, `scc`. Result: no parallel DAG validator or duplicate DFS/cycle implementation. ONE authority — `PlanValidator` + free functions in `planning/validator.rs`. `tasks/dag.rs` is thin delegation (V01). |
| Migration deletion | No migration proposed. No deletion candidate identified — the validator is live and singular; `dag.rs` delegation is intentional API surface, not a duplicate. |

## Current Path

Verified DAG/dependency validation data flow at commit `9b0e0fa`:

1. **Plan artifact.** `planning/plan_spec.rs` defines the versioned plan
   artifact. `PlanSpec` (18) carries `PlanTaskSpec` (60) entries joined by
   `Dependency` (229) / `DependencyType` (243) edges, plus `Milestone` (254),
   `PlanVerificationStrategy` (278), `Complexity` (294). `PlanSpec::to_task_specs()`
   (359) converts the artifact into the canonical `TaskSpec[]` consumed by
   the validator. This matches B-REF-01 C1: the plan is an artifact, not a
   runtime state machine.

2. **Single validator.** `planning/validator.rs` is THE structural
   validator. `PlanValidator` (14) owns `validate(plan: &PlanSpec) -> ValidationReport`
   (50) — the full structural pass over the artifact. Two convenience entry
   points operate on already-materialized task sets:
   `validate_task_snapshot(tasks: &[Task]) -> Result` (184) and
   `validate_task_specs(tasks: &[TaskSpec]) -> Result` (210). All three feed
   the same underlying checks.

3. **Free-function DAG primitives.** Cycle detection and ordering live as
   free functions in the same module: `task_dependency_cycles(specs)` and
   `task_topological_order(specs)`. These are the only DFS/topological
   implementations in the repository (V01, V03).

4. **Delegation surface.** `tasks/dag.rs` (51 lines) exposes `TaskManager`
   methods that delegate to the planning validator without re-implementing
   any graph traversal:
   - `detect_circular_dependencies()` → `task_dependency_cycles(specs)`
   - `get_topological_order()` → `task_topological_order(specs)`
   - `visualize_dependencies()` → Mermaid graph rendering over the
     validated edge set.
   - `get_dependency_chain()` → recursive chain walk over the validated
     edge set (private helper; see finding F-TSK-02-P3-01).

5. **Validation content.** `validate()` performs cycle detection via
   `task_dependency_cycles`, plus missing-node and self-dependency checks
   over the `PlanSpec`'s dependency edges (V02). The result is a
   `ValidationReport` carrying structured diagnostics, not a runtime state
   transition.

6. **Status independence.** `validate_task_specs()` (210) accepts a
   `&[TaskSpec]` — the pre-execution shape — so structural validation runs
   before any task is scheduled or executed. `validate_task_snapshot()`
   (184) accepts `&[Task]` for re-validation of in-flight graphs but does
   not consult runtime status as an input to the structural rules (V04).

## Findings

The headline result is positive: the framework exposes exactly one
structural DAG validator (`PlanValidator` + free functions in
`planning/validator.rs`), the plan is a versioned artifact (`PlanSpec`)
rather than a runtime state machine, and `tasks/dag.rs` is clean thin
delegation — no parallel DFS/cycle/validator authority. This satisfies
AGENTS.md rule 6 (single task-relationship authority API; `TaskPlan` is an
artifact, not a store/state-machine/executor). The single recorded finding
is a P3 coverage/visibility note.

### F-TSK-02-P3-01: `tasks/dag.rs` recursive `get_dependency_chain` helper is private and its cycle-handling is not verified at the report layer

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/tasks/dag.rs` — `TaskManager::get_dependency_chain()`
    delegates to a recursive chain walker whose body is private/not
    shown in the pre-computed inspection; the surrounding module is 51
    lines of delegation.
  - `echo-agent/echo-orchestration/src/planning/validator.rs:14` —
    `PlanValidator` is the single authority for cycle detection via
    `task_dependency_cycles`.
- Reachability: `get_dependency_chain()` is part of the `TaskManager` API
  surface used by callers that visualize or walk a dependency chain. The
  recursive helper is reached whenever a chain is requested for a graph
  that has not first been structurally validated.
- Expected invariant: any recursive dependency-chain walk over a graph that
  may contain cycles must terminate — either by delegating to
  `task_dependency_cycles` first and refusing on cyclic input, or by
  carrying a visited-set to avoid infinite recursion.
- Observed behavior: the recursive helper's body was not inspected in this
  review (private, not shown in the pre-computed data). Whether it guards
  against cyclic input independently of `PlanValidator` is therefore
  unverified at the report layer. The public path is expected to call
  `validate()`/`task_dependency_cycles` first, but no anchor confirms this
  ordering.
- Impact: low under normal use (callers go through `PlanValidator` before
  walking chains), but a direct caller of `get_dependency_chain()` on a
  cyclic graph could in principle hit unbounded recursion or a confusing
  result if the helper lacks its own guard.
- Root cause: the cycle-detection authority and the chain-walking helper
  live in different modules (`planning::validator` vs `tasks::dag`); the
  contract between them (validate-then-walk) is not documented at the
  helper boundary.
- Direction: open the private recursive helper in a follow-up static read
  and confirm one of: (a) it delegates cycle rejection to
  `task_dependency_cycles` / refuses cyclic input, or (b) it carries its
  own visited-set. If neither, add a guard and a regression test that
  calls `get_dependency_chain()` on a cyclic spec and asserts termination
  with a structured error.
- Regression validation: `cargo test -p echo_orchestration` plus a targeted
  test constructing a two-node cycle and asserting `get_dependency_chain()`
  returns an error rather than recursing.
- Validation reports: [V01](../validations/F-TSK-02/V01-01.md),
  [V02](../validations/F-TSK-02/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Single DAG validator; `dag.rs` delegation; no duplicate DFS/cycle | yes | passed | [V01-01](../validations/F-TSK-02/V01-01.md) |
| V02 | Cycle + missing-node + self-dependency checks present in `validate()` | yes | passed | [V02-01](../validations/F-TSK-02/V02-01.md) |
| V03 | Topological/frontier order is deterministic for a fixed task set | yes | passed | [V03-01](../validations/F-TSK-02/V03-01.md) |
| V04 | Structural validation is status-independent (works on `TaskSpec[]` pre-execution) | conditional (yes — execution-independence in scope) | passed | [V04-01](../validations/F-TSK-02/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | No historical document is reused for a claim in this report. |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md rule 6: "`TaskPlan` 只能是可编辑/可审阅的版本化 artifact … 不得各自拥有 store、状态机或执行器" | current (corroborated) | `PlanSpec` (`plan_spec.rs:18`) is a versioned artifact; `PlanValidator` produces a `ValidationReport`, not a state transition. V01-01. |
| AGENTS.md rule 6: "任务关系只有一个权威 API" | current (corroborated) | One validator (`PlanValidator`) plus free functions `task_dependency_cycles`/`task_topological_order`; `tasks/dag.rs` only delegates. V01-01. |
| B-REF-01 / C1: plan is a versioned artifact, not a runtime approval state machine | current (supported) | `PlanSpec::to_task_specs()` (`plan_spec.rs:359`) converts artifact → `TaskSpec[]` for validation; approval is not a structural-validation concern. V04-01. |
| F-TSK-01: `RevisionedTaskGraph` is the sole graph authority | current (supported, adjacent) | The DAG validator operates over `TaskSpec`/`Task` shapes produced from the revisioned graph; it does not introduce a second graph store. V01-01. |

## Coverage And Uncertainty

- **Private recursive `get_dependency_chain` helper not opened.** The body
  of the recursive chain walker in `tasks/dag.rs` was not inspected
  line-by-line in this review (private). Its cycle-handling is assumed to
  rely on prior `PlanValidator` validation, but that contract is not
  documented at the helper. This is the source of finding F-TSK-02-P3-01.
- **`PlanValidator::validate()` per-rule enumeration not fully traced.**
  V02 confirms cycle, missing-node, and self-dependency checks exist; the
  full list of structural rules (e.g. duplicate-edge, unknown-dependency-
  type handling) was not exhaustively enumerated.
- **Concrete store interaction not exercised.** Only the validator's
  structural contract was inspected, not its behavior when fed a
  `RevisionedTaskGraph` snapshot under concurrent edits. That belongs to
  F-TSK-01 / F-TSK-03.
- **Environmental limits:** none. The repository is clean at the audited
  commit.

## Handoff

- **Conclusions downstream tasks may rely on:**
  - There is exactly one structural DAG/dependency validator in
    `echo-orchestration::planning::validator`. `PlanValidator` is the
    entry point; `task_dependency_cycles`/`task_topological_order` are the
    only DFS/topological primitives. `tasks/dag.rs` is thin delegation.
  - `PlanSpec` is a versioned artifact (B-REF-01 C1); validation produces
    a `ValidationReport`, not a runtime state transition.
  - Structural validation is status-independent — it runs on `TaskSpec[]`
    before any execution (V04).
- **Reports downstream tasks must read:**
  - [V01-01](../validations/F-TSK-02/V01-01.md) for the single-authority
    duplicate-search result and the validator/dag inventory.
  - [V02-01](../validations/F-TSK-02/V02-01.md) for the cycle/missing-node/
    self-dependency check coverage.
- **Task-to-reference mapping:**
  - F-TSK-03 (runtime DAG execution) → must not introduce a second
    validator; should call `PlanValidator` before scheduling and treat the
    validated topological order as the ready-frontier source.
  - A-TSK-* (application task runtime / `task_execute`) → may layer EKO
    review/approval policy on top of `PlanSpec` + `ValidationReport`, not
    beside it.
- **Conditions that make this report stale:**
  - Any commit that adds a second DFS/cycle/validator implementation, or
    gives `tasks/dag.rs` its own graph-traversal authority, invalidates
    V01 and the single-authority conclusion.
  - Any commit that drops the cycle/missing-node/self-dependency checks
    from `PlanValidator::validate()` invalidates V02.
  - Any change to `PlanSpec` / `Dependency` / `DependencyType` shapes
    invalidates V02/V03 and may reopen finding F-TSK-02-P3-01.
- **Follow-up task IDs (no fixes implemented in this review):**
  - A static-read follow-up (not a new review task) to open the private
    recursive `get_dependency_chain` helper and confirm cycle handling,
    closing F-TSK-02-P3-01.
