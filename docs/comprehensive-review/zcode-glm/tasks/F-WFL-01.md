# F-WFL-01: Workflow and pipeline engine

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb532d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean

## Question

Are graph, DAG, sequential/concurrent pipelines, checkpoints, and state
transitions a coherent generic workflow API distinct from dynamic tasks?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-orchestration/src/workflow/mod.rs` — module surface,
  `Workflow` trait, `WorkflowEvent`, `WorkflowOutput`/`StepOutput`,
  `SharedAgent`.
- `echo-agent/echo-orchestration/src/workflow/graph.rs` — `Graph`,
  `GraphBuilder`, `EdgeKind`/`Edge`/`ConditionFn`, `InterruptConfig`/
  `InterruptState`/`RunUntilInterruptResult`, `run`/`run_until_interrupt`/
  `resume`/`resume_with_state`/`restore_to_checkpoint`/`branch_from`/
  `tag_checkpoint`/`run_stream`, `resolve_next`, `NextStep`, build-time
  validation, and the 16 unit tests.
- `echo-agent/echo-orchestration/src/workflow/state.rs` — `SharedState`,
  `StateInner`, `StateError`, `set`/`get`/`fork`/`merge`/`merge_overwrite`/
  `deep_merge`/`snapshot`/`from_json`, `deep_merge_values` helper.
- `echo-agent/echo-orchestration/src/workflow/node.rs` — `Node`,
  `NodeAction` (Agent/Subgraph/Function/Passthrough), `NodeFn` trait.
- `echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs` —
  `Checkpoint`, `InterruptType`, `CheckpointInfo`, `CheckpointFilter`,
  `CheckpointStore` trait, `MemoryCheckpointStore`, `FileCheckpointStore`.
- `echo-agent/echo-orchestration/src/workflow/sequential.rs` —
  `SequentialWorkflow`(+Builder), `WorkflowStep`.
- `echo-agent/echo-orchestration/src/workflow/concurrent.rs` —
  `ConcurrentWorkflow`(+Builder), `default_merge`.
- `echo-agent/echo-orchestration/src/workflow/dag.rs` — `DagWorkflow`
  (+Builder), `DagNode`/`DagEdge`, `topological_sort`/`detect_cycle`/
  `compute_in_degree`/`build_predecessors`/`build_successors`.
- `echo-agent/echo-orchestration/src/workflow/pipelines/` — `mod.rs`,
  `data_pipeline.rs` (path validation, contract prompt), `writing_pipeline.rs`
  (conditional revise loop).
- `echo-agent/src/workflow/mod.rs` + `echo-agent/src/workflow/loader.rs` —
  root re-export and declarative `WorkflowDefinition` YAML/JSON loader.
- Cross-repo duplicate/registration/reachability search for the workflow
  surface across `echo-agent` and `echo-agent-cli`.

## Out Of Scope

- The dynamic task system (`echo-orchestration::tasks`) — audited under
  `F-TSK-01` (read as dependency); this task references its conclusions
  but does not re-audit it.
- `ReactAgent` internals used by agent nodes (`NodeAction::Agent` calls
  `Agent::execute`/`chat`) — deferred to the agent-runtime task.
- The Tauri `execute_workflow` command's GUI/IPC wiring beyond its
  framework-API consumption — deferred to application tasks
  (`A-*`).
- Performance benchmarking of DAG scheduling or `deep_merge` cost.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — especially the framework-vs-application
    layering gate, the "single authority API" rule 6 (workflow is
    separate from the task CRUD), the "code cleanup / no compatibility
    burden" rule, the UTF-8 / no-panic rules, and the local single-user
    threat model that shapes the path-traversal and secret findings.
  - `docs/comprehensive-review/REPORTING.md`,
    `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
- Dependency task reports read:
  - `F-CORE-01` (this reviewer) — for the `ReactError::Other` catch-all
    pattern reused by `StateError`/`Checkpoint::restore_state`, and the
    convention that root `echo-agent/src/*.rs` modules are thin
    re-exports.
  - `F-TSK-01` (this reviewer) — for the canonical task-system
    authority (`TaskRevisionService` + `RevisionedTaskStore` +
    `task_create`/`task_update`/`task_list`) that this task must prove
    distinct from.
- Historical documents treated as hypotheses: none.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. The graph engine (`Graph`/`GraphBuilder`/`SharedState`/`CheckpointStore`), the three pipeline flavors (`Sequential`/`Concurrent`/`Dag`), the two prebuilt pipelines (`data`/`writing`), and the `Workflow` trait describe generic static-orchestration concepts any `echo-agent` consumer may need. They live correctly in `echo-orchestration` (V01 confirms single definition site; root `echo-agent/src/workflow/mod.rs:63-66` only re-exports, plus a `loader` submodule that depends back on `ReactAgentBuilder` — correctly kept out of `echo-orchestration`). |
| EKO product policy | None at the framework layer. The Tauri `execute_workflow` command (`echo-agent-cli/src/tauri/commands/panels.rs:745-785`) is the EKO GUI entry point; it consumes the framework loader and adds IPC/policy, correct layer. No EKO-specific field has leaked into `Graph`/`Checkpoint`/`SharedState`. |
| Adapter boundary | The declarative loader (`echo-agent/src/workflow/loader.rs`) is a thin adapter: YAML/JSON → `WorkflowDefinition` → `GraphBuilder` calls → `Graph`. It performs no scheduling, state authority, or independent validation beyond what `GraphBuilder::build()` already does (its conditional-edge closure merely returns the YAML author's `then`/`else_node` strings). |
| Duplicate search | Searched names (whole `echo-agent` + `echo-agent-cli`): `Graph`, `GraphBuilder`, `SequentialWorkflow`, `ConcurrentWorkflow`, `DagWorkflow`, `Workflow`, `WorkflowEvent`, `SharedState`, `Checkpoint`, `CheckpointStore`, `run_data_pipeline`, `run_writing_pipeline`, `WorkflowDefinition`, `load_graph_from_yaml`, plus the task-side (`TaskRevisionService`, `RevisionedTaskGraph`, `task_create`, `task_update`, `task_list`). Result: single definition site in `echo-orchestration::workflow`; zero references from workflow to the task system (V01); workflow is not registered as an agent tool. Internal duplication: `Graph` and `DagWorkflow` are two overlapping graph implementations inside the same subsystem (V02/V03). |
| Migration deletion | No migration proposed. The two graph flavors (`Graph`, `DagWorkflow`) overlap but have distinct contracts (cycles+interrupts vs strict-DAG+true-concurrency); neither is dead. Converging them is a design decision, not a cleanup, and is out of scope for this read-only review. |

## Current Path

Verified workflow data flow at commit `9b0e0fa` / `b3b2e81`:

1. **Public surface.** `echo-orchestration::workflow` (mod.rs:32-57)
   exports the graph engine, the three pipeline flavors, the two prebuilt
   pipelines, and the checkpoint types. Root `echo-agent/src/workflow/mod.rs`
   re-exports everything (`:66`) and adds the declarative `loader`
   submodule (`:71`). `echo-agent/src/lib.rs:65` exposes `pub mod workflow`
   and `:241` brings the surface into the framework prelude.

2. **Two orchestration paradigms, cleanly split from tasks.**
   - **Workflow = static topology.** Built ahead-of-time via builders
     (`GraphBuilder`, `*WorkflowBuilder`), executed by the framework's
     own loop. Agents are embedded as `Arc<dyn Agent>` inside nodes; the
     embedder (not the agent) decides the topology.
   - **Tasks = dynamic plan.** Created/updated at runtime by the agent
     via `task_create`/`task_update`/`task_list` tools, versioned by
     `RevisionedTaskGraph`, mutated solely by `TaskRevisionService`
     (F-TSK-01 V01-01/V02-01).
   - Coupling search: `grep` for the task-system names inside
     `echo-orchestration/src/workflow/` returns zero matches (V01). The
     split is total.

3. **Graph execution.** `Graph::run` (`graph.rs:595-725`) walks from
   `entry`, executes each node, and routes via `resolve_next`
   (`:1372-1416`) which takes the single outgoing edge (build rejects
   multiple). Termination on `__end__` or a finish node. `max_steps`
   (default 100, `:513`) bounds loops; cycles are legal (test
   `test_loop_graph`). Cancellation is cooperative via an optional
   `CancellationToken` (`:579-590`), checked at node boundaries.

4. **Parallel fan-out (graph).** `add_parallel_edge(from, targets, then)`
   (`:367-381`) records `EdgeKind::Parallel`. `run()` forks a branch
   state per target (`state.fork()`, `:695`), executes, and
   `state.deep_merge(&branch_state)` (`:701`). Branches run sequentially
   in-loop (no `tokio::spawn`); the doc comment blames a `Send + 'static`
   bound that does not in fact hold (V03).

5. **Pipeline flavors.** `SequentialWorkflow` chains steps
   (output → next input, `sequential.rs:82-141`). `ConcurrentWorkflow`
   spawns all agents in parallel via `tokio::spawn` and merges via a
   user fn (`concurrent.rs:84-147`). `DagWorkflow` runs a Kahn topological
   schedule, spawning each ready batch (`dag.rs:96-224`), and enforces
   acyclicity at build (`dag.rs:255-297`).

6. **Interrupt + checkpoint lifecycle.** `run_until_interrupt`
   (`graph.rs:738-906`) honors `interrupt_before`/`interrupt_after`
   (with `*` wildcard). At each interrupt it builds a `Checkpoint`
   capturing `(graph_name, current_node, state_snapshot, path, step_count,
   interrupt_type)`, persists via `CheckpointStore::save`, and returns
   `Interrupted`. `resume(checkpoint, decision)` (`:913-1092`) honors
   `ApprovalDecision` (`Approved` continues; `Rejected`/`Deferred` abort
   with restored state), restores state from the JSON snapshot, and
   deletes the checkpoint on completion. `branch_from` (`:1147-1178`)
   forks a checkpoint with `parent_checkpoint_id` lineage. Default store
   is `MemoryCheckpointStore` (`:515`); `with_checkpoint_store` can attach
   `FileCheckpointStore` for persistence.

7. **Application reachability.** The Tauri command `execute_workflow`
   (`echo-agent-cli/src/tauri/commands/panels.rs:745-785`) loads a graph
   from a YAML/JSON string stored in app state via
   `load_graph_from_yaml_str`/`load_graph_from_json_str`, sets `input`
   on a fresh `SharedState`, and calls `graph.run(...)`. This is the only
   production (non-example) end-to-end driver; examples and benches
   exercise the programmatic API (`demo28/34/39/46/47/49/57`,
   `agent_bench.rs`).

## Findings

The headline result on the task question is **positive**: the workflow
engine is a coherent static-orchestration API and is cleanly distinct
from the dynamic task system — no shared authority, no store coupling,
zero code references across the boundary (V01). The prebuilt pipelines
and the declarative loader are correctly layered. The findings below are
about **internal coherence of the workflow subsystem itself**, not about
the workflow-vs-task boundary.

### F-WFL-01-P1-01: `Graph::resume()` parallel branch diverges from `run()`/`run_until_interrupt()`/`run_stream()` — no fork, no merge

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/workflow/graph.rs:1064-1081` —
    `resume()` parallel branch executes on the **shared** `state`
    (`state.set_current_node(target_name)`, `target_node.execute(&state)`)
    with no `fork()` and no `deep_merge()`. Inline comment `:1065`:
    `"Parallel branch execution, shared SharedState (sequential execution)"`.
  - `echo-agent/echo-orchestration/src/workflow/graph.rs:686-706` —
    `run()` parallel branch: `state.fork()?` (`:695`) +
    `state.deep_merge(&branch_state)?` (`:701`).
  - `echo-agent/echo-orchestration/src/workflow/graph.rs:867-887` —
    `run_until_interrupt()` parallel branch: same fork+merge (`:876`,
    `:882`).
  - `echo-agent/echo-orchestration/src/workflow/graph.rs:1318-1350` —
    `run_stream()` parallel branch: same fork+merge (`:1329`, `:1344`).
- Reachability: `resume()` is reached by any caller that drives a graph
  through `run_until_interrupt` and then approves continuation. The Tauri
  `execute_workflow` command does not use interrupts today, but the pub
  API is advertised (graph.rs:727-1178) and the only resume test
  (`test_resume_with_state_reuses_checkpoint_identity`, `:1883`) uses a
  **linear** graph — the divergence is unverified by tests.
- Expected invariant: all four execution paths (`run`,
  `run_until_interrupt`, `resume`, `run_stream`) must apply identical
  semantics to a parallel fan-out so that a workflow resumed from a
  checkpoint behaves the same as one run straight through.
- Observed behavior: three paths fork each branch into an isolated
  `SharedState` and `deep_merge` results back; `resume()` mutates the
  shared state in place, sequentially, with no isolation. Later branches
  observe earlier branches' writes; there is no merge step.
- Impact: any workflow that combines `add_parallel_edge` with an
  interrupt point and is resumed from a checkpoint produces different
  state than the same workflow run without interruption. A branch that
  reads a key before writing it may see pollution from a sibling branch
  only on the resume path. Silent correctness defect for
  parallel+interrupt workflows.
- Root cause: the `resume()` loop was written (or last touched) without
  porting the fork+merge logic the other three sites use; the inline
  comment documents the divergence as intentional ("shared SharedState")
  rather than flagging it as a bug.
- Direction: make `resume()`'s `NextStep::Parallel` arm fork+merge each
  branch exactly like the other three sites (`state.fork()?` per target,
  `state.deep_merge(&branch_state)?` after). Add a regression test that
  runs a parallel-edge graph to completion and a second run that
  interrupts then resumes, asserting identical final state. No deletion
  target — this is a behavior fix.
- Regression validation: `cargo test -p echo_orchestration workflow::graph`;
  new test `resume_parallel_branch_matches_run`.
- Validation reports: [V03](../validations/F-WFL-01/V03-01.md)

### F-WFL-01-P2-01: Two overlapping graph implementations (`Graph` + `DagWorkflow`) with asymmetric validation and asymmetric concurrency

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/workflow/graph.rs:533-552` —
    `Graph` (general directed graph: cycles, conditional edges,
    interrupts, checkpoints; "parallel" edges run sequentially).
  - `echo-agent/echo-orchestration/src/workflow/dag.rs:81-85` —
    `DagWorkflow` (strict DAG: acyclicity enforced at build, no
    conditional edges, true `tokio::spawn` batch concurrency, no
    interrupts/checkpoints).
  - `echo-agent/echo-orchestration/src/workflow/dag.rs:255-297` —
    `DagWorkflow::build()` runs both `detect_cycle` and
    `topological_sort` (Kahn's also detects cycles at `:360-366`),
    i.e. redundant cycle detection.
- Reachability: both are live pub APIs, both exercised by examples
  (`demo28_workflow.rs` uses all three pipeline flavors + the graph
  examples use `GraphBuilder`). The module doc (`mod.rs:5-23`) presents
  them as complementary ("Graph Workflow" vs "Pipeline Workflow") but
  does not explain when a user should pick `Graph`-with-only-fixed-edges
  over `DagWorkflow`, or why `Graph`'s parallel edge is sequential while
  `DagWorkflow`'s independent nodes are truly concurrent.
- Expected invariant: a generic framework should not host two
  overlapping graph engines with silently different concurrency and
  validation contracts unless the distinction is documented and the
  boundary is clean.
- Observed behavior: a user building a fan-out/fan-in pipeline gets
  sequential branch execution if they reach for `Graph::add_parallel_edge`
  and true concurrency if they reach for `DagWorkflow`. A user who needs
  a conditional branch must use `Graph` (DAG has no conditional edges).
  A user who needs interrupts/checkpoints must use `Graph` (DAG has
  none). The four capabilities (conditionals, cycles, true parallelism,
  interrupts) are split across two types with no unified story.
- Impact: API confusion and a maintainability burden (two node/edge
  representations, two executors, two validation sets). Not a correctness
  defect in either flavor in isolation.
- Root cause: accretion. `Graph` was built first (LangGraph-style, with
  cycles/interrupts); the simpler pipeline flavors were added later
  (`Sequential`/`Concurrent`/`Dag`) without converging on a shared
  executor or node type.
- Direction: either (a) document the decision matrix explicitly in
  `mod.rs` (when to use which flavor) and keep both, accepting the
  duplication; or (b) unify `DagWorkflow`'s true-concurrency scheduler
  into `Graph` so `add_parallel_edge` actually spawns (this also
  resolves F-WFL-01-P3-02). Converging is a design task, not a cleanup —
  deferred to a normal implementation milestone, not this review. No
  deletion recommended today: both flavors are live and have distinct
  contracts.
- Regression validation: whichever direction is chosen, the existing
  tests in `graph.rs` and `dag.rs` plus `demo28_workflow.rs` must still
  pass.
- Validation reports: [V01](../validations/F-WFL-01/V01-01.md),
  [V02](../validations/F-WFL-01/V02-01.md),
  [V03](../validations/F-WFL-01/V03-01.md)

### F-WFL-01-P2-02: Conditional-edge targets are not validated at build time — declarative typos pass build and fail at runtime

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/workflow/graph.rs:457-485` —
    `Graph::build()` validation switch; the `EdgeKind::Conditional(_)`
    arm falls through to `_ => {}` (`:484`) — no target check.
  - `echo-agent/echo-orchestration/src/workflow/graph.rs:1398-1404` —
    `resolve_next` returns `NextStep::Single(target)` for whatever string
    the condition closure produced.
  - `echo-agent/echo-orchestration/src/workflow/graph.rs:650-655` —
    `run()` looks up `self.nodes.get(&current)` and errors at runtime with
    `"Node 'X' not found in graph"` if the condition returned an
    unregistered name.
  - `echo-agent/src/workflow/loader.rs:236-259` — the declarative loader
    turns YAML `condition.then`/`else_node` into a conditional-edge
    closure returning the author-supplied strings verbatim.
- Reachability: every `add_conditional_edge` user and every declarative
  workflow with a `condition:` edge. The writing_pipeline
  (`writing_pipeline.rs:369-404`) uses hardcoded literals
  (`"finalize_prompt"`, `"revise_prompt"`) that match its node names, so
  it is safe today; the risk is user-authored YAML.
- Expected invariant: build() should reject any edge whose target cannot
  resolve to a registered node or `__end__`, so that a successfully built
  graph is guaranteed runnable.
- Observed behavior: Fixed and Parallel edge targets are validated at
  build; Conditional edge targets are opaque inside `Box<dyn ConditionFn>`
  and only resolved at runtime. A typo in a YAML `then:` field builds
  cleanly and fails mid-execution.
- Impact: poor UX for the declarative path (the primary EKO GUI entry
  point, `panels.rs:760-761`, loads user-authored YAML). The error
  surfaces far from its cause. Not a correctness defect once running, but
  a validation gap that contradicts the otherwise-strict build checks.
- Root cause: conditions are object-safe closures returning `String`, so
  the builder cannot enumerate possible targets. The closure contract
  does not require the author to declare the candidate set.
- Direction: extend `add_conditional_edge` (and the loader's
  `ConditionDefinition`) to accept an optional declared target set
  (`Vec<String>`) that `build()` validates against the node registry;
  fall back to runtime resolution only when the set is absent. This is a
  backward-compatible API addition. Alternatively, document the
  limitation loudly in the `add_conditional_edge` doc comment and have
  the loader validate `then`/`else_node` against the declared node list
  before constructing the closure.
- Regression validation: new build-time test that a conditional edge
  with a declared target set containing an unknown node fails `build()`;
  existing conditional tests unchanged.
- Validation reports: [V02](../validations/F-WFL-01/V02-01.md)

### F-WFL-01-P2-03: No schema-version field on `Checkpoint` — persisted checkpoints break silently on struct evolution

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:38-72`
    — `Checkpoint` struct has no version field.
  - `echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:395-406`
    — `FileCheckpointStore::load` deserializes straight to `Checkpoint`
    via `serde_json::from_str`; any field change that is not
    `#[serde(default)]` errors with `"Failed to parse checkpoint"`.
- Reachability: any consumer that persists checkpoints via
  `FileCheckpointStore` across a version bump. The EKO CLI does not
  currently persist workflow checkpoints across versions, but the pub
  API advertises persistence and `FileCheckpointStore` is part of the
  framework's stated menu.
- Expected invariant: a persistent artifact that may outlive a binary
  upgrade should carry a schema version so the loader can branch.
- Observed behavior: no version field. This is structurally the same gap
  F-RCT-05-P1-04 found for the ReAct snapshot checkpoint (this reviewer,
  same baseline) — that report flagged "no version field" for the ReAct
  snapshot; the workflow `Checkpoint` repeats the pattern.
- Impact: low for the current single-user, dev-stage CLI; medium for any
  third-party `echo-agent` consumer that relies on `FileCheckpointStore`
  for long-running workflow durability. Under AGENTS.md "no compatibility
  burden" this is acceptable for now, but the pub API should at minimum
  reserve a `schema_version` field.
- Root cause: same as F-RCT-05 — checkpoint types were not versioned at
  introduction.
- Direction: add `#[serde(default)] pub schema_version: u32` to
  `Checkpoint` (and `CheckpointInfo`), set it to `1` in `Checkpoint::new`,
  and have `FileCheckpointStore::load` return a typed
  `ReactError::Config(...)` on a future-version mismatch instead of a
  generic parse error.
- Regression validation: `cargo test -p echo_orchestration
  workflow::checkpoint_store`; add a fixture that loads a v0 (no-version)
  JSON and asserts it deserializes with `schema_version == 0`.
- Validation reports: [V04](../validations/F-WFL-01/V04-01.md)

### F-WFL-01-P3-01: `add_parallel_edge` doc comment mis-describes the sequential-execution constraint

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/workflow/graph.rs:361-366` —
    comment: "Currently, parallel branches execute sequentially within
    the same async task (not via `tokio::spawn`), because `dyn Agent`
    nodes are not `Send + 'static`."
  - `echo-agent/echo-orchestration/src/workflow/mod.rs:71` —
    `pub type SharedAgent = Arc<dyn Agent>;` and `Agent: Send + Sync`
    (per F-CORE-01) ⇒ `Arc<dyn Agent>` is `Send + Sync`.
  - `echo-agent/echo-orchestration/src/workflow/concurrent.rs:99-111`
    and `dag.rs:134-163` — both `tokio::spawn` the same `Arc<dyn Agent>`
    type, proving the bound is not the blocker.
- Reachability: the comment is part of the pub `add_parallel_edge` doc.
- Expected invariant: doc rationale should match the real constraint.
- Observed behavior: the stated `Send + 'static` bound does not hold;
  the real reason `Graph::run()` cannot easily spawn is that the async
  loop borrows `&self` and `&state`, and spawning would require cloning
  the relevant `Arc`s out and restructuring the loop body.
- Impact: low (documentation), but it actively misleads any contributor
  trying to fix F-WFL-01-P2-01 / enable true concurrency in `Graph`.
- Root cause: incorrect comment, likely written before the pipeline
  flavors demonstrated spawnability.
- Direction: rewrite the comment to state the real constraint
  (lifetime of `&self`/`&state` in the async loop), and note that the
  pipeline flavors prove `Arc<dyn Agent>` is spawnable.
- Regression validation: doc-only change.
- Validation reports: [V03](../validations/F-WFL-01/V03-01.md)

### F-WFL-01-P3-02: `SharedState::merge()` (non-overwrite) contains dead/confused lock acquisitions and a nonsensical SAFETY comment

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/workflow/state.rs:324-355` —
    `merge()` locks `self.inner.read()` and binds the guard to
    `other_inner` (`:325`, mis-named — it locks `self`, not `other`),
    drops it; locks `self.inner.read()` again as `self_inner` (`:330`),
    drops it; comment `:334` `// SAFETY: need read both locks to prevent
    deadlock.` Then performs the actual read of `other.inner` and write
    of `self.inner` at `:338-353`.
- Reachability: `merge()` has no production caller (only
    `state.rs:508` test). Retained as pub framework API per AGENTS.md
    "a public framework API is retained unless framework-wide evidence
    shows it is obsolete" — the dead lock code is the cleanup target,
    not the fn itself.
- Expected invariant: a lock acquisition should be used or removed; a
  `SAFETY:` comment should explain a real safety obligation.
- Observed behavior: the first two `read()` calls acquire and
  immediately drop the guard without use; the comment invokes "deadlock"
  reasoning that does not apply (the real read+write happens after, with
  a fresh pair of guards). The actual merge (`or_insert_with`,
  no-overwrite) is correct.
- Impact: low. No functional defect (the real merge is correct). Hazards
  are confusion for future readers and a small wasted lock cycle.
- Root cause: a botched refactor left two dead lock acquisitions and a
  stale comment.
- Direction: delete the two dead `read()` blocks (`:325-336`) and the
  `SAFETY:` comment; keep the real read+write pair at `:338-353`.
- Regression validation: `cargo test -p echo_orchestration
  workflow::state`; `test_merge` (`:499-512`) must still pass.
- Validation reports: [V03](../validations/F-WFL-01/V03-01.md)

### F-WFL-01-P3-03: `FileCheckpointStore` filename derived from raw user-supplied id (path-traversal surface) and `list()` silently skips corrupt entries

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:359-361`
    — `checkpoint_path(id)` does `self.base_path.join(format!("{}.json",
    id))`. `load(id)` (`:395`) and `delete(id)` (`:433`) accept any
    `&str`.
  - `echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:420-425`
    — `list()` uses `if path.extension ... && let Ok(json) = ... && let
    Ok(cp) = ...`, silently skipping entries that fail to deserialize.
- Reachability: `load`/`delete` are pub fns on a pub store. In the normal
  flow `id` is a UUID generated by `Checkpoint::new` (`:84`), so the
  traversal requires a caller to pass an attacker-controlled id. The EKO
  threat model is local single-user (AGENTS.md), so practical risk is
  low; the API surface is the concern.
- Expected invariant: a store that sandboxes entries under `base_path`
  should not let an id escape that sandbox; listing and explicit loading
  should agree on what exists.
- Observed behavior: `id = "../../etc/x"` would resolve to a path
  outside `base_path`; `list()` hides corrupt files while `load()` on the
  same id errors.
- Impact: low (local single-user, UUID ids in practice). Defense-in-depth
  gap, not an exploitable defect under the AGENTS.md threat model —
  documented here because the framework is a reusable library and a
  third-party consumer could feed non-UUID ids.
- Root cause: no id sanitization; `list()` chose robustness (skip bad
  files) over consistency (surface them).
- Direction: sanitize `id` (reject `/`, `\`, and `..` components) in
  `checkpoint_path`, or `canonicalize` and assert the result stays within
  `base_path`. Optionally have `list()` record/surface skipped entries
  via `tracing::warn!` so the two failure modes agree.
- Regression validation: `cargo test -p echo_orchestration
  workflow::checkpoint_store`; add a test that a traversal-style id is
  rejected.
- Validation reports: [V04](../validations/F-WFL-01/V04-01.md)

### F-WFL-01-P3-04: Doc-comment example in `sequential.rs` uses byte slicing that can panic on UTF-8

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/src/workflow/sequential.rs:67` —
    doc example prints
    `&s.output[..s.output.len().min(80)]`, a byte slice on a `String`.
- Reachability: doc comment only (not compiled into the binary), but
  rendered in `cargo doc` and copied by users.
- Expected invariant: AGENTS.md "UTF-8 safe, no byte-level truncation" —
  examples should use `s.chars().take(80).collect::<String>()`.
- Observed behavior: the example would panic on a multibyte char at the
  80-byte boundary and teaches readers the unsafe pattern.
- Impact: low (documentation), but violates the project-wide UTF-8 rule.
- Root cause: example predates / sidesteps the rule.
- Direction: replace with `s.output.chars().take(80).collect::<String>()`.
- Regression validation: doc-only change; `cargo doc -p echo_orchestration`
  should still build clean.
- Validation reports: [V01](../validations/F-WFL-01/V01-01.md) (scope
  inventory)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Task/workflow semantic overlap search and workflow surface inventory | yes | passed | [V01-01](../validations/F-WFL-01/V01-01.md) |
| V02 | Graph and DAG build-time validation (cycle detection, missing-node, conditional-target) | yes | passed | [V02-01](../validations/F-WFL-01/V02-01.md) |
| V03 | Concurrent/parallel state merge semantics across flavors and execution paths | yes | failed | [V03-01](../validations/F-WFL-01/V03-01.md) |
| V04 | Checkpoint/resume lifecycle, stores, and gaps | yes | passed (with documented gaps) | [V04-01](../validations/F-WFL-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | No historical document is reused for a claim in this report. |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md rule 6: "TaskPlan 只能是可编辑/可审阅的版本化 artifact … `task_create`/`task_update`/`task_list`" (workflow must be separate from task CRUD) | current (corroborated) | Zero references from `echo-orchestration::workflow` to `TaskRevisionService`/`RevisionedTaskStore`/`task_*`; workflow is not registered as an agent tool. V01-01. |
| F-TSK-01 (this reviewer): single canonical task authority | current (supported, distinct) | The workflow engine is the separate static-orchestration subsystem the task model is distinct from. V01-01. |
| F-CORE-01-P3-01 (this reviewer): `ReactError::Other(String)` is the untyped catch-all | current (reused) | `StateError` and `Checkpoint::restore_state` both map their failures to `ReactError::Other(String)` (`state.rs:80-102`, `checkpoint_store.rs:165-169`). Consistent with the prior finding; not separately actionable here. |
| F-RCT-05-P1-04 (this reviewer): checkpoint with no version field | current (repeated here) | The workflow `Checkpoint` (`checkpoint_store.rs:38-72`) repeats the same no-version-field pattern. F-WFL-01-P2-03. V04-01. |

## Coverage And Uncertainty

- **Not exercised at runtime.** All four validations are static
  inspection. A targeted executable test for F-WFL-01-P1-01
  (`resume()` + `add_parallel_edge`, asserting state equality with
  `run()`) would convert the high-confidence static claim into a
  failing test; deferred to a follow-up implementation task.
- **`ConcurrentWorkflow` / `DagWorkflow` detached-task behavior not
  traced.** When an agent errors inside a spawned batch, the early-return
  leaves sibling spawned tasks to complete detached (their results are
  dropped). This was noted but not promoted to a finding: there is no
  shared mutable state between siblings, so the only cost is wasted work
  and possible late log noise. Worth a follow-up if these flavors become
  a hot path.
- **Prebuilt pipelines inspected at the contract level only.**
  `data_pipeline.rs` path validation (`validate_workspace_relative_path`,
  `:198-217`) is sound (rejects absolute, `..`, prefix). The
  `writing_pipeline` revise loop is bounded by `max_revisions` and the
  graph's `max_steps`. Prompt-construction correctness (does the agent
  actually follow the contract?) is out of scope — it is an agent-quality
  question, not a workflow-engine question.
- **Loader's `function` node type unsupported** (`loader.rs:216-224`
  returns an error). This is a documented limitation, not a defect.
- **Environmental limits:** none. Both repos clean at audited commits.

## Handoff

- **Conclusions downstream tasks may rely on:**
  - The workflow engine (`echo-orchestration::workflow`) and the dynamic
    task system (`echo-orchestration::tasks`) are cleanly distinct: no
    shared authority, no store coupling, no code references either way.
    AGENTS.md rule 6 holds at this boundary. Downstream framework and
    application tasks can treat workflow as the static-orchestration
    subsystem and tasks as the dynamic-plan subsystem.
  - The workflow module is correctly layered in the framework
    (`echo-orchestration` engine + root `echo_agent::workflow::loader`
    adapter that depends back on `ReactAgentBuilder`). EKO product
    policy lives only at the Tauri command layer
    (`panels.rs:745-785`).
  - `Graph::resume()` + `add_parallel_edge` has a correctness defect
    (F-WFL-01-P1-01) that any downstream task touching interrupts,
    parallel execution, or checkpoint/resume must not regress further
    and should plan to fix.
- **Reports downstream tasks must read:**
  - [V01-01](../validations/F-WFL-01/V01-01.md) for the workflow/task
    separation proof and the workflow surface inventory.
  - [V03-01](../validations/F-WFL-01/V03-01.md) for the resume/parallel
    divergence (the only P1) and the spawn-vs-sequential analysis.
- **Task-to-reference mapping:**
  - `F-TSK-02` / `F-TSK-03` (DAG validation / runtime DAG execution)
    should treat `DagWorkflow` as a *separate* static-pipeline DAG, not
    the dynamic `RevisionedTaskGraph`; the two must not be conflated.
  - Application tasks (`A-TSK-*`, `A-BOOT-*`) that wire the Tauri
    `execute_workflow` command should note it is the only production
    workflow driver and that it does not use interrupts today.
  - A future framework task on engine convergence may pick up
    F-WFL-01-P2-01 (unify `Graph`/`DagWorkflow`) — it is a design
    decision, not a cleanup.
- **Conditions that make this report stale:**
  - Any commit that unifies `resume()`'s parallel branch with the other
    three sites invalidates F-WFL-01-P1-01 and the V03 failure.
  - Any commit that adds a declared-target-set to `add_conditional_edge`
    and validates it in `build()` invalidates F-WFL-01-P2-02.
  - Any commit that adds a `schema_version` to `Checkpoint` invalidates
    F-WFL-01-P2-03.
  - Any commit that introduces a workflow→task reference (or registers a
    workflow tool) invalidates the V01 primary conclusion.
- **Follow-up task IDs (no fixes implemented in this review):**
  - A framework implementation task to fix F-WFL-01-P1-01 (resume/parallel
    fork+merge) with a regression test. Highest priority among the
    findings.
  - A framework cleanup task for F-WFL-01-P3-01 / P3-02 (doc + dead lock
    code) — low priority, can ride a maintenance commit.
