# F-WFL-01: Workflow and pipeline engine

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean (baseline matches README.md shared baseline)

## Question

Are graph, DAG, sequential/concurrent pipelines, checkpoints, and state
transitions a coherent generic workflow API distinct from dynamic tasks?

**Answer: Yes — the workflow engine is one coherent generic API with no
parallel implementation of the same semantics, and it is fully independent
of the dynamic task model (zero cross-references). However, its
checkpoint/resume path has a recovery-correctness bug (P1), two
concurrent-state-merge defects (P2), and one approval-terminus ambiguity
(P2); the rest of the findings are P3 dead surface / docs / cleanup.**

## Scope

- `echo-orchestration/src/workflow/`: `mod.rs` (160 L, Workflow trait /
  WorkflowEvent / SharedAgent), `graph.rs` (1934 L, Graph/GraphBuilder/
  Interrupt/Resume), `node.rs` (242 L), `state.rs` (543 L, SharedState +
  merge semantics), `checkpoint_store.rs` (588 L, Checkpoint/CheckpointStore/
  Memory/File stores), `sequential.rs` (183 L), `concurrent.rs` (180 L),
  `dag.rs` (524 L, topological sort + DFS cycle detection),
  `pipelines/data_pipeline.rs` (492 L), `pipelines/writing_pipeline.rs`
  (614 L) — all read in full (5,474 lines).
- Root-crate facade `echo-agent/src/workflow/`: `mod.rs` (73 L),
  `dsl.rs` (372 L, StateGraph), `loader.rs` (442 L, WorkflowDefinition +
  YAML/JSON loaders) — read in full.
- EKO consumers: `echo-agent-cli/src/tauri/commands/panels.rs:746-782`
  (execute_workflow), `src/tauri/mod.rs:248-252` (registration),
  `echo-agent-cli/echo-agent-app-core/src/state.rs:200-240`
  (StoredWorkflow/WorkflowStep/WorkflowDef), frontend
  `web-frontend/src/api/endpoints.ts:348-356`, `WorkflowPanel.tsx`.

## Out Of Scope

- ReAct agent internals executed by nodes (F-RCT-*); agent-snapshot
  checkpoints (F-RCT-05); `TaskNode` checkpoint state machine
  (F-TSK-01-P3-04, classified); dynamic task model / revisioned graph /
  `RuntimeDagExecutor` semantics (F-TSK-01/02/03); EKO TaskRuntime file
  authorities (A-TSK-*); scheduler (F-OPS-01); React prompt-assembly
  "pipeline" (`src/agent/react/run/pipeline.rs`) — unrelated concept,
  term only.

## Inputs

- Root `AGENTS.md` (single-authority / no-parallel-semantics rules;
  Subagent-only terminology; framework-vs-app layering gate; UTF-8 and
  panic safety), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (F-WFL-01 card), `zcode-ds/README.md`.
- Dependency reports read: `zcode-ds/reports/tasks/F-CORE-01.md`
  (event/error contracts), `zcode-ds/reports/tasks/F-TSK-01.md`
  (canonical task model; P3-01 legacy TaskManager, P3-04 TaskNode naming
  overlap — both cross-referenced, no interaction with workflow code).
- Historical documents treated as hypotheses: `echo-agent/README.md`,
  `echo-agent/echo-orchestration/README.md`, root
  `docs/MASTER-PLAN.md` (no workflow-engine claims found), module
  docstrings in `src/workflow/mod.rs` and
  `echo-orchestration/src/workflow/mod.rs`.

## Layering Decision

- Generic mechanism (framework, correctly placed): the entire
  `echo-orchestration/src/workflow/` engine — Graph/GraphBuilder,
  SharedState, CheckpointStore, Sequential/Concurrent/DagWorkflow,
  Workflow trait, pipelines. Any `echo-agent` consumer could use it;
  nothing depends on EKO policy.
- Adapter boundary (thin, lossless): root `src/workflow/mod.rs` re-exports
  `echo_orchestration::workflow::*`; `dsl.rs` StateGraph and `loader.rs`
  WorkflowDefinition only lower declarative specs into GraphBuilder calls
  (no scheduling/state authority); EKO `panels.rs` + `StoredWorkflow`
  stores the raw definition string and lowers it through the framework
  loader — no second engine.
- EKO product policy / dead model: `WorkflowDef`/`WorkflowStep`
  (`app-core/src/state.rs:220-240`) is an unused application-side linear
  workflow model (P3-08) — cleanup candidate, not authority.
- Duplicate search terms (V01-01): `Graph`, `GraphBuilder`, `SharedState`,
  `Checkpoint`, `CheckpointStore`, `Workflow`, `SequentialWorkflow`,
  `ConcurrentWorkflow`, `DagWorkflow`, `DagNode`, `DagEdge`,
  `WorkflowStep`, `StepOutput`, `WorkflowOutput`, `WorkflowDefinition`,
  `StateGraph`; task-side `TaskSpec/TaskExecution/TaskStatus/
  TaskRevisionService/RuntimeDagExecutor/DagExecutionState/PlanSpec/
  TaskPlan/TodoItem/PlanTask/EkoTaskSpec`; concepts `topological`,
  `detect_cycle`, `checkpoint`, `pipeline`, `worker`. Result: one workflow
  authority; **zero** workflow↔tasks cross-references; no `worker`
  terminology; EKO `WorkflowDef` dead (P3-08).

## Current Path

1. Build: `GraphBuilder` (programmatic), `WorkflowDefinition`/`StateGraph`
   (declarative) → `build()` validates entry/edge/multi-edge → immutable
   `Graph` with default `MemoryCheckpointStore` and `max_steps=100`.
2. Execute: `Graph::run` / `run_until_interrupt` / `run_stream` loop
   entry→…→finish/`__end__`; Fixed/Conditional edges; Parallel fan-out
   executes **sequentially** with `SharedState::fork()` per branch and
   `deep_merge` back (documented; `dyn Agent` not `Send+'static`);
   cooperative cancellation at node boundaries via `CancellationToken`.
3. Interrupt/resume: `interrupt_before`/`interrupt_after` save a
   `Checkpoint` (id = random UUID) into the `CheckpointStore`
   (Memory default, File with atomic tmp+rename); `resume(checkpoint,
   ApprovalDecision)` restores state and continues from
   `checkpoint.current_node`; `resume_with_state`/`branch_from`/
   `restore_to_checkpoint`/`tag_checkpoint` support external
   modification, forking, and lineage.
4. Pipelines: `SequentialWorkflow`/`ConcurrentWorkflow`/`DagWorkflow`
   implement the `Workflow` trait (string-in/string-out, `tokio::spawn`
   for real concurrency; Kahn topological order; DFS cycle detection);
   `run_data_pipeline`/`run_writing_pipeline` are single/multi-agent
   Graph workflows with contract prompts and quality-loop.
5. EKO: GUI WorkflowPanel (TS) → Tauri `execute_workflow` (panels.rs:746)
   → `load_graph_from_yaml_str/json_str` → `Graph::run` → serialized
   `GraphResult.state` returned to the frontend. No interrupts used.

## Findings

### F-WFL-01-P1-01: AfterNode checkpoint resume drops the pending parallel fan-out and bypasses the next node's before-interrupt

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-orchestration/src/workflow/graph.rs:826-849` (interrupt_after
  resolves `next` and stores only the target name as `checkpoint.current_node`;
  for `NextStep::Parallel` it stores `then`, discarding `targets`);
  `:913-1092` (`resume` continues from `checkpoint.current_node` without
  replaying the routing decision); `:981-1010` (executes the checkpointed
  node with no before-interrupt check); `:1044-1061` (before-interrupt is
  only re-checked for the node *after* the resumed one).
- Reachability: public framework API; exercised by `run_until_interrupt` +
  `resume` for any consumer using `interrupt_after` on a node whose
  outgoing edge is a Parallel fan-out or whose successor has
  `interrupt_before`. No EKO consumer uses interrupts today; no test covers
  this combination (V03-03).
- Expected invariant: resume yields the same execution (node sequence and
  final state) as an uninterrupted run of the same graph.
- Observed behavior: (1) with `interrupt_after(A)` where A fans out to
  `[b1,b2] → then`, the checkpoint stores `current_node = then`; `resume`
  jumps straight to `then` and **b1/b2 never execute** — the workflow
  completes with a different result than the uninterrupted run. (2) With
  `interrupt_after(A)` where A's next node B has `interrupt_before`,
  `resume` executes B without pausing — the configured approval point is
  silently bypassed.
- Impact: interrupted workflows resume to a *different execution*; a whole
  fan-out stage silently never runs (data/result corruption for consumers
  using interrupts + parallel edges), and an approval gate is silently
  skipped on the resume path.
- Root cause: the AfterNode checkpoint collapses the pending routing
  decision into a bare node name; `resume` treats the checkpoint as
  "already routed" instead of replaying the pending `NextStep`
  (targets+then, or the before-interrupt decision) at resume time.
- Direction: store the pending `NextStep` (targets + then, or the
  resolved single target *plus* the fact that the before-interrupt was
  already granted) in the checkpoint (e.g., extend `Checkpoint` with a
  pending-routing field — note `pending_action` exists but is never
  populated, P3-02), and replay it in `resume`; add regression fixtures:
  interrupt_after on a fan-out node → assert branches execute after
  resume; interrupt_after followed by a before-interrupt node → assert the
  pause still fires.
- Regression validation: `cargo test -p echo_orchestration --lib --locked
  workflow` with the two new fixtures (V03-03-style trace, then executable
  test).
- Validation reports: [V03-03](../validations/F-WFL-01/V03-03.md),
  [V01-01](../validations/F-WFL-01/V01-01.md)

### F-WFL-01-P2-01: `resume` parallel fan-out skips fork/deep_merge, diverging from the other three execution paths

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `graph.rs:1064-1081` (resume parallel branch executes
  `target_node.execute(&state)` directly on the shared state) vs
  `graph.rs:686-706` (run), `:867-887` (run_until_interrupt),
  `:1320-1349` (run_stream) which all `state.fork()` per branch and
  `state.deep_merge(&branch_state)` back; `state.rs:284-293` (fork),
  `:381-408` (deep_merge).
- Reachability: any `resume` that crosses a parallel edge (same trigger as
  P1-01 but for BeforeNode checkpoints whose node has a parallel outgoing
  edge); public API, no current EKO consumer, no test.
- Expected invariant: all four execution paths share one fan-out/merge
  semantic (isolated branches, recursive object merge).
- Observed behavior: on the resume path branches write into the same
  `SharedState` — no isolation (branches see each other's intermediate
  writes), same-key conflicts are wholesale last-writer-wins instead of
  recursive deep merge, and per-branch `set_current_node` races.
- Impact: state after a resume that crosses a parallel edge can differ
  from the non-resumed execution of the same graph — a second,
  inconsistent fan-in semantic on the recovery path.
- Root cause: resume path was implemented before the fork/deep_merge
  design and never migrated.
- Direction: route resume's parallel branch through the same
  fork + deep_merge helper used by `run`/`run_stream`; extract a private
  `run_parallel(targets, then, state)` helper so all four paths call one
  implementation.
- Regression validation: fixture resuming from a BeforeNode checkpoint at
  a fan-out node, asserting branch isolation (branch A writes key X, branch
  B reads X unmodified) and deep-merge of nested objects.
- Validation reports: [V03-02](../validations/F-WFL-01/V03-02.md)

### F-WFL-01-P2-02: parallel-branch message history is silently dropped by `deep_merge`/`merge` (values-only merge contract gap)

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `state.rs:381-408` (deep_merge iterates only `values`),
  `:324-355` (merge/merge_overwrite same), `:128-137` (StateInner has
  `values` **and** `messages`); `node.rs:196` (agent nodes
  `state.push_message(...assistant(output))`); `graph.rs:701, 882, 1344`
  (branch merge after fan-out); `state.rs:9-12` (doc: "Structured message
  history" is part of the state contract).
- Reachability: any graph with a parallel edge whose branches contain
  agent nodes; the final `GraphResult.state` (serialized by EKO
  `panels.rs:779-781` into the workflow response) loses branch messages.
- Expected invariant: the fan-in merge preserves all documented state
  (values + message history).
- Observed behavior: messages appended inside parallel branches never
  appear in the merged state; only scalar/JSON `values` are merged.
- Impact: consumers observing/checkpointing message history get an
  incomplete stream after any parallel fan-in; checkpoint snapshots
  (`Checkpoint::new` snapshots full state) are also missing branch
  messages. In-engine impact is currently low (nothing reads `messages()`
  inside the engine), so this is a contract-completeness defect, not an
  in-engine failure.
- Root cause: merge helpers were written for the KV contract only; the
  message-history part of `StateInner` was added later without extending
  the merge.
- Direction: extend `deep_merge` (and decide for `merge`/`merge_overwrite`)
  to concatenate `messages`, or document explicitly that message history
  is per-branch and discarded at fan-in; add a fixture with two parallel
  agent nodes asserting merged `messages()`.
- Regression validation: `cargo test -p echo_orchestration --lib --locked
  workflow` with the message-merge fixture.
- Validation reports: [V03-02](../validations/F-WFL-01/V03-02.md)

### F-WFL-01-P2-03: Rejected/Deferred approval in `resume` reports `Completed` — aborted workflow indistinguishable from a finished one

- Priority: P2
- Confidence: medium
- Layer: framework
- Evidence: `graph.rs:919-946` (both branches return
  `Ok(RunUntilInterruptResult::Completed(GraphResult { state:
  checkpoint.restore_state()?, .. }))` for `Rejected` and `Deferred`);
  `graph.rs:192-197` (RunUntilInterruptResult has only Completed/
  Interrupted); `human_loop::ApprovalDecision` (source of the variants).
- Reachability: any interrupt/resume consumer that feeds an actual
  approval decision; no current EKO consumer; no test covers the rejection
  path (V03-03).
- Expected invariant: an aborted workflow is a distinct terminal from a
  completed workflow (caller can react to rejection/deferral).
- Observed behavior: rejection and deferral both return the Completed
  variant with the checkpoint's partial state; the caller cannot tell
  whether the workflow finished or was refused without inspecting state or
  re-deriving the approval decision.
- Impact: approval-driven orchestration cannot branch on rejection;
  "workflow succeeded" metrics/UI would misreport aborted runs as
  completed.
- Root cause: abort was modeled as "return the checkpoint state" without a
  terminal variant to carry the abort reason.
- Direction: add an `Aborted { state, reason }` (or `Rejected`) variant to
  `RunUntilInterruptResult` for Rejected/Deferred, or return a typed error;
  update the doc contract and add a rejection fixture.
- Regression validation: fixture resuming with `ApprovalDecision::Rejected`
  asserting the aborted terminal is distinguishable from Completed.
- Validation reports: [V03-03](../validations/F-WFL-01/V03-03.md)

### F-WFL-01-P3-01: `WorkflowEvent::Token` and `NodeError` are dead variants — documented streaming contract is not implemented

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-orchestration/src/workflow/mod.rs:100-103` (variants with
  doc "Token produced by node (forwarded during streaming agent output)",
  "Node execution error (non-fatal; ... stream continues)");
  `graph.rs:1234-1369` (run_stream emits only NodeStart/NodeEnd/Completed;
  a node error aborts the stream via `?`).
- Reachability: variant definitions only; zero construction sites repo-wide
  (V02-01).
- Expected invariant: every public `WorkflowEvent` variant is emitted by
  the streaming executor, or the variant is removed.
- Observed behavior: `Token` and `NodeError` never appear in any stream.
- Impact: consumers matching on the documented event set write unreachable
  branches; the streaming contract promises non-fatal node-error
  continuation that does not exist (node errors terminate the stream).
- Root cause: event vocabulary was designed ahead of the streaming
  implementation.
- Direction: either implement token forwarding and non-fatal node error
  continuation in `run_stream`, or delete the two variants and update the
  docs; add a stream-variant coverage test.
- Regression validation: test asserting the stream emits exactly
  NodeStart/NodeEnd/Completed for a failing node (documenting current
  behavior) or the corrected contract.
- Validation reports: [V02-01](../validations/F-WFL-01/V02-01.md)

### F-WFL-01-P3-02: `InterruptType::ToolApproval`/`UserRequest` and `InterruptState::tool_approval` are unreachable; `Checkpoint.pending_action` is never populated

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `checkpoint_store.rs:181,183` (InterruptType variants),
  `:56,95` (pending_action field always None); `graph.rs:173-185`
  (`InterruptState::tool_approval` constructor); only `checkpoint_store.rs:
  502` (unit test) constructs `UserRequest`.
- Reachability: definitions only; zero production construction sites
  (V02-01).
- Expected invariant: interrupt types are constructible and reachable, or
  the surface is removed.
- Observed behavior: only BeforeNode/AfterNode are produced; tool-approval
  and user-request pause modes exist only as dead API, and the checkpoint
  field designed to carry a pending tool call is always None.
- Impact: misleading API surface; a future tool-approval interrupt would
  need to build the plumbing from scratch anyway.
- Root cause: interrupt feature grew to Before/After first; the other
  variants and `pending_action` were scaffolding.
- Direction: remove the unreachable variants/constructor (and
  `pending_action`) or implement them; the P1-01 fix direction could reuse
  `pending_action` for the pending-routing field instead.
- Regression validation: n/a (compile-only cleanup) — re-grep for
  constructions after deletion.
- Validation reports: [V02-01](../validations/F-WFL-01/V02-01.md)

### F-WFL-01-P3-03: `workflow` feature is a no-op virtual gate — the engine is always compiled and the feature table is misleading

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `Cargo.toml:102` (`workflow = []`); zero
  `#[cfg(feature = "workflow")]` anywhere in the repo (V02-01);
  `echo-orchestration/src/lib.rs:26` and `src/lib.rs:65` declare the module
  unconditionally; `README.md:256` lists `workflow` as a feature
  ("Graph workflow engine") and `README.md:394` advertises it.
- Reachability: the feature only gates example targets
  (`required-features`, `Cargo.toml:211-236`).
- Expected invariant: a declared feature either gates code or is removed.
- Observed behavior: the module always compiles; enabling/disabling the
  feature changes nothing; the README feature table implies optionality.
- Impact: consumers reading the feature table believe workflow can be
  compiled out (size/attack-surface) and that `default = []` excludes it —
  false; F-FEAT-01 adjacency.
- Root cause: the feature was declared when the module was carved out of a
  plugin-style crate, and the cfg was dropped during the workspace
  migration.
- Direction: remove the `workflow = []` feature and update the README
  feature table (and the `required-features` of the demo examples), or
  actually gate `src/workflow`/`echo-orchestration` workflow module behind
  it (larger change; not recommended — the module is small and depended on
  by EKO unconditionally).
- Regression validation: `cargo check -p echo_agent --features workflow`
  and `--no-default-features` produce identical results; README table
  matches reality.
- Validation reports: [V02-01](../validations/F-WFL-01/V02-01.md),
  [V05-01](../validations/F-WFL-01/V05-01.md)

### F-WFL-01-P3-04: `finish_nodes` are not validated at build time — typo'd finish names surface only as runtime errors

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `graph.rs:431-518` (build validates entry existence :438, edge
  endpoints :448-486, multiple outgoing edges :493-505; `finish_nodes`
  only stored :512); `:1376-1385` (runtime error "has no outgoing edges and
  is not a finish node"); build tests `:1658-1708` (no finish-node case).
- Reachability: any `GraphBuilder` use with `set_finish` — including the
  declarative loaders (`loader.rs:270-272`) which pass user YAML finish
  lists; EKO `execute_workflow` runs user-authored YAML.
- Expected invariant: every finish node must exist in the node map at
  build time (same as entry and edge endpoints).
- Observed behavior: a finish name that matches no node is silently
  ignored; the workflow only fails at runtime when a path reaches a node
  with no outgoing edges and errors with a misleading message.
- Impact: user-authored workflow YAML with a typo'd `finish` fails late and
  confusingly; benign typos can produce wrong (missing) termination
  semantics.
- Root cause: build-validation coverage omission.
- Direction: validate `finish_nodes` against the node map in `build()`
  (also validate conditional-edge targets while at it, or document the
  runtime-check design); add a build-time test for unknown finish nodes and
  unknown conditional targets.
- Regression validation: unit tests asserting `build()` rejects unknown
  finish nodes / conditional targets.
- Validation reports: [V03-01](../validations/F-WFL-01/V03-01.md)

### F-WFL-01-P3-05: subgraph node steps are not counted toward the parent's max_steps despite the documented claim

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `node.rs:158-160` (doc: "Its execution is counted toward the
  parent's step limit"); `node.rs:199-205` (executes `subgraph.run(
  state_clone)` — the subgraph uses its own internal `step_count` and its
  own `max_steps`); `graph.rs:632-661` (parent counts exactly 1 step per
  subgraph node).
- Reachability: `GraphBuilder::add_subgraph_node` users.
- Expected invariant: parent `max_steps` bounds total execution, per doc.
- Observed behavior: a looping subgraph can execute up to its own 100-step
  default inside one parent step; the parent's limit does not bound it.
- Impact: runaway subgraph loops are not bounded by the parent's
  configured limit; doc is misleading.
- Root cause: doc written before implementation; subgraph runs with its own
  budget.
- Direction: either share the parent's step counter/limit into the
  subgraph run (pass a step budget), or correct the doc; add a fixture with
  a looping subgraph asserting the parent limit holds.
- Regression validation: test with parent `max_steps` small and a looping
  subgraph — expect termination at parent limit (after fix) and document
  the current behavior first.
- Validation reports: [V01-01](../validations/F-WFL-01/V01-01.md)

### F-WFL-01-P3-06: Documentation drift — `Graph::from_yaml` does not exist; orchestration README quickstart shows a nonexistent builder API

- Priority: P3
- Confidence: high
- Layer: framework (docs)
- Evidence: `echo-agent/README.md:212` (`Graph::from_yaml("wf.yaml")?`);
  `src/workflow/mod.rs:55-59` (same nonexistent API in the module
  docstring, `ignore`-marked); `echo-orchestration/README.md:18-33`
  (`GraphBuilder::new()` with no name, `.add_node(..)`, `.edge(..)` — none
  exist; the actual API is `GraphBuilder::new(name)` +
  `add_function_node`/`add_agent_node`/`add_edge`, and `TaskManager`
  shown is the legacy surface per F-TSK-01-P3-01). The real declarative API
  is `WorkflowDefinition::from_yaml` / `load_graph_from_yaml`
  (`src/workflow/loader.rs:141-312`).
- Reachability: documentation consumers only.
- Expected invariant: documented API samples compile against current code.
- Observed behavior: three stale samples referencing nonexistent APIs.
- Impact: users following the docs write code that fails to compile;
  reviewers misread capabilities.
- Root cause: docs predate the loader/DSL consolidation and were not
  revalidated.
- Direction: update both READMEs and the `src/workflow/mod.rs` docstring to
  the real API (`WorkflowDefinition::from_yaml_str` /
  `load_graph_from_yaml_str` / `GraphBuilder::new(name)`), or add
  `Graph::from_yaml` as a thin alias; mark the samples as compile-tested.
- Regression validation: compile the corrected quickstart samples
  (doctest/build of the README example).
- Validation reports: [V05-01](../validations/F-WFL-01/V05-01.md)

### F-WFL-01-P3-07: deprecated zero-user `SharedAgentMutex` alias and dead lock dance in `SharedState::merge`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-orchestration/src/workflow/mod.rs:80-81` (deprecated
  alias, zero users repo-wide, V02-01); `state.rs:324-336` (`merge` acquires
  `self.inner.read()` twice and drops both immediately before the real
  other-read + self-write locking, with a "SAFETY: need read both locks"
  comment that no longer describes the code).
- Reachability: alias — none; merge — public method, tests only
  (`test_merge`), not used by the engine.
- Expected invariant: no deprecated aliases with zero users; no dead code
  inside live methods.
- Observed behavior: dead alias and redundant lock acquire/drop remain
  compiled.
- Impact: minimal (clutter; the comment misleads future readers about
  lock ordering).
- Root cause: API migration leftovers.
- Direction: delete `SharedAgentMutex` and simplify `merge` to the actual
  other-read → self-write sequence with a correct ordering comment.
- Regression validation: `cargo test -p echo_orchestration --lib --locked
  workflow` (test_merge) after cleanup.
- Validation reports: [V02-01](../validations/F-WFL-01/V02-01.md),
  [V03-02](../validations/F-WFL-01/V03-02.md)

### F-WFL-01-P3-08: EKO `WorkflowDef`/`WorkflowStep` linear-step model is dead code

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/state.rs:220-240`
  (`WorkflowStep` with prompt/tool/condition step types, `WorkflowDef` with
  a doc pointing at the framework `WorkflowDefinition`); zero references
  anywhere in `echo-agent-cli` (Rust or frontend, V02-01/V01-01). The live
  EKO path uses `StoredWorkflow` (raw YAML/JSON string) + framework loader
  (`panels.rs:746-782`).
- Reachability: definition only; no constructor, no executor, no command.
- Expected invariant: per AGENTS.md "no parallel implementation / delete
  dead code", one workflow model per layer.
- Observed behavior: a second, unused linear-step workflow model coexists
  with the framework engine in the application layer.
- Impact: cognitive overlap (reviewers may mistake it for a parallel
  engine); dead surface to maintain.
- Root cause: early EKO CLI workflow design superseded by the framework
  loader adapter.
- Direction: delete `WorkflowDef`/`WorkflowStep` from app-core state.rs
  (grep-first: no users), keeping `StoredWorkflow`; if a linear-step REST
  model is ever needed, project it onto the framework definition schema.
- Regression validation: re-grep `WorkflowDef|WorkflowStep` across
  `echo-agent-cli` after deletion; `cargo check -p echo-agent-app-core
  --no-default-features --locked`.
- Validation reports: [V01-01](../validations/F-WFL-01/V01-01.md),
  [V02-01](../validations/F-WFL-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate/semantic-overlap search (workflow vs task/DAG/plan; concepts; worker term) | yes | passed | [V01-01](../validations/F-WFL-01/V01-01.md) |
| V02 | Registration and runtime reachability (exports, EKO Tauri path, examples, dead-surface inventory) | yes | passed | [V02-01](../validations/F-WFL-01/V02-01.md) |
| V03 | Graph validation invariants | yes | passed | [V03-01](../validations/F-WFL-01/V03-01.md) |
| V03 | Concurrent state merge invariants (4 fan-out paths, merge helpers) | yes | passed | [V03-02](../validations/F-WFL-01/V03-02.md) |
| V03 | Checkpoint/resume fixtures and traces (after-interrupt routing, rejection terminals, existing test coverage) | yes | passed | [V03-03](../validations/F-WFL-01/V03-03.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked workflow` | yes | passed (exit 0, 44 ok) | [V04-01](../validations/F-WFL-01/V04-01.md) |
| V04 | `cargo test -p echo_agent --lib --locked workflow` | yes | passed (exit 0, 11 ok) | [V04-02](../validations/F-WFL-01/V04-02.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked` (full crate) | conditional | passed (exit 0, 294 ok) | [V04-03](../validations/F-WFL-01/V04-03.md) |
| V05 | Historical-document drift (READMEs, module docstrings, MASTER-PLAN) | yes | passed | [V05-01](../validations/F-WFL-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `README.md:212` "Declarative Workflow ... `Graph::from_yaml("wf.yaml")?`" | stale | no `Graph::from_yaml` exists; loader API is `WorkflowDefinition::from_yaml`/`load_graph_from_yaml` (V05-01) |
| `README.md:256` feature table `workflow` row | stale | no `#[cfg(feature = "workflow")]` anywhere; always compiled (V02-01) |
| `echo-orchestration/README.md:18-33` quickstart (`GraphBuilder::new()`, `.add_node`, `.edge`) | stale | API requires name + `add_function_node`/`add_agent_node`/`add_edge` (V05-01) |
| `src/workflow/mod.rs:55-59` docstring `Graph::from_yaml` | stale | same as README claim (V05-01) |
| `docs/MASTER-PLAN.md` — workflow-engine milestone claims | none found | no drift to classify; only unrelated hits (:53, :81, :750, :860) |
| F-TSK-01-P3-04 (TaskNode checkpoint state machine naming overlap) | current (independent) | workflow `CheckpointStore` is a separate mechanism with no code overlap (V01-01) |
| F-TSK-01-P3-01 (legacy TaskManager surface) | current (independent) | echo-orchestration README still advertises it (P3-06 note) |

## Coverage And Uncertainty

- P1-01/P2-01/P2-03 behaviors were established by code trace, not by an
  executable fixture: no test combines `interrupt_after` with parallel
  edges or with a next-node before-interrupt, and no test covers the
  Rejected/Deferred resume path. A read-only review cannot add fixtures;
  Q-FLT-02 / Q-E2E-01 should carry the confirmation.
- The EKO GUI workflow path (WorkflowPanel → `execute_workflow`) was
  traced statically only; no runtime smoke test (A-SRF-02 / Q-E2E scope).
- Doctests in `graph.rs`/`state.rs`/`dsl.rs` were not executed (lib-only
  test runs); `state.rs`'s runnable doctest is a candidate for Q-FW-02.
- `Workflow` trait's default `run_stream` and `SharedAgentMutex` deprecation
  were inspected but no external consumer of either exists.
- `dag.rs` duplicate-node-id handling (builder allows registering the same
  id twice; HashMap overwrite with duplicated topo entries) was noted but
  judged too low-impact for a finding.

## Handoff

- Downstream tasks may rely on: workflow and dynamic tasks are distinct
  single authorities (V01); workflow engine is production-reachable through
  the EKO GUI `execute_workflow` (V02); build-time validation is solid
  except finish_nodes (V03-01); resume-path routing must be fixed before
  any interrupt-based product feature is built on top (P1-01, P2-01);
  merge contract gap on messages (P2-02); rejection terminal ambiguity
  (P2-03).
- Reports to read: all nine validation reports; F-TSK-01 (task model
  independence + TaskNode naming), F-CORE-01 (event/error contracts),
  F-CORE-01-P3-01 (parent_event_id — unrelated to workflow events, which
  are a separate `WorkflowEvent` vocabulary).
- Stale conditions: this report becomes stale if `graph.rs` run/resume/
  interrupt logic, `state.rs` merge helpers, `checkpoint_store.rs`
  checkpoint fields, or the EKO `execute_workflow` adapter change.
- Follow-up task IDs: Q-FLT-02 (resume fixtures for P1-01/P2-01/P2-03),
  Q-E2E-01 (EKO workflow panel smoke), F-FEAT-01 adjacency (P3-03 virtual
  feature), A-SRF-02 (workflow panel command contract), X-INV-01
  (doc-sample validity), S-RDM-01 (deletion targets: P3-01 dead variants,
  P3-07 alias, P3-08 WorkflowDef).
