# F-WFL-01: Workflow and pipeline engine

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; Codex reports only

## Question

Are graph, DAG, sequential/concurrent pipelines, checkpoints, interrupts, and
state transitions a coherent generic workflow API distinct from dynamic tasks?

## Scope

- All code under `echo-agent/echo-orchestration/src/workflow`: Graph, nodes,
  state, checkpoint stores, sequential/concurrent/DAG workflows, and built-in
  data/writing pipelines.
- Root `echo-agent/src/workflow` DSL/loader adapters, root/prelude exports,
  Cargo feature declarations, examples, benchmarks, and existing tests.
- Definition/export/reasonable external reachability, graph validation, stable
  identity/lineage, before/after/tool interrupt, resume/replay, checkpoint
  persistence, parallel merge/failure, cancellation/timeout/loop bounds,
  panic/UTF-8/overflow, and dynamic-task semantic separation.

## Out Of Scope

- Source fixes, Cargo/rustc/build/test execution, dynamic fixtures, and network
  research under the explicit review-stage constraint.
- ReAct run checkpoint corruption/public snapshot defects owned by F-RCT-05.
- Tool permission policy/transport/argument preservation owned by F-HITL-01;
  this task only reviews workflow's separate interrupt surface.
- Dynamic TaskRun/PlanTask/SubagentRun scheduling and state propagation owned by
  F-TSK-01..03. This report does not introduce Plan approval runtime states.
- EKO workflow/product composition; absence of a CLI caller is not evidence that
  a reasonable public framework capability should be deleted.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, exact F-WFL-01 task
  card in `TASKS.md`; Codex reviewer rules.
- Codex [B-REF-01](B-REF-01.md), [F-CORE-01](F-CORE-01.md),
  [F-TSK-01](F-TSK-01.md), [F-RCT-05](F-RCT-05.md), and
  [F-HITL-01](F-HITL-01.md).
- Current source and current tests were authoritative; historical prose was
  classified rather than copied.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Caller-defined fixed graph topology, typed node/edge validation, bounded execution, deterministic branch merge, checkpoint identity/lineage, interruption, resume, cancellation, and Store interfaces are valid reusable framework mechanisms. |
| EKO product policy | Which workflow an assistant chooses, UI prompts, automated-action approval defaults, and TaskRun/Plan artifacts remain application policy. Workflow must not become an EKO task scheduler or Plan approval state machine. |
| Adapter boundary | Root StateGraph and YAML/JSON loader may remain thin construction adapters over GraphBuilder. They must preserve topology fields and own no second executor/checkpoint state machine. |
| Duplicate search | Searched both repositories for Graph/StateGraph/DagWorkflow/SequentialWorkflow/ConcurrentWorkflow, workflow/checkpoint/interrupt/resume/pipeline, TaskRun/PlanTask/TaskPlan/Todo/TaskManager/RuntimeDagExecutor, approval states, exports, features, examples, tests, and callers. |
| Migration deletion | Fix the existing Graph/Checkpoint authorities. Do not add a third workflow engine. If public interrupt variants remain, wire them through the canonical typed continuation; otherwise delete only the unsupported variants, not the generic workflow capability. |

## Current Path

```text
echo_orchestration::workflow
  -> SequentialWorkflow: string output feeds next Agent
  -> ConcurrentWorkflow: spawn all Agents, registration-order join/merge
  -> DagWorkflow: validate/toposort, batch spawn, predecessor output join
  -> GraphBuilder -> Graph
       Node::execute -> Agent/function/subgraph
       SharedState -> fork/deep_merge
       run | run_stream | run_until_interrupt
       checkpoint_store -> MemoryCheckpointStore (default)
                        -> FileCheckpointStore (optional public Store)
       resume | resume_with_state | branch_from

echo_agent root
  -> unconditional workflow module/re-export/prelude
  -> StateGraph DSL -> GraphBuilder
  -> WorkflowDefinition YAML/JSON -> GraphBuilder
  -> examples/benchmarks/built-in pipelines
```

This is a real public framework surface with reasonable external consumers.
Dynamic TaskRun/PlanTask/SubagentRun remains a separate revisioned, product-
projected execution authority; no workflow source references Plan approval
states.

## Findings

### F-WFL-01-P1-01: Resume accepts replayed or cross-graph checkpoint values without a run/attempt claim

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:38`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:913`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:948`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:990`
- Reachability: public Graph checkpoint/interrupt APIs -> returned clonable Checkpoint -> public `resume`/`resume_with_state`/`branch_from`.
- Expected invariant: a durable continuation is bound to one graph definition,
  run, and attempt, then atomically claimed so stale/replayed/cross-graph input
  cannot execute side-effecting nodes.
- Observed behavior: Checkpoint has a UUID and graph_name but no graph version,
  run, or attempt identity. `resume` never validates graph_name or persisted
  existence and accepts the value itself; deletion occurs only after success.
  The same clone/deserialized checkpoint can run repeatedly or on another Graph
  containing its current node. New interrupts use `Checkpoint::new` and drop
  the prior checkpoint's lineage.
- Impact: nodes/tools can execute more than once after retry, duplicate delivery,
  stale UI state, or caller error; changed graph code can reinterpret old state.
- Root cause: checkpoint is both public snapshot and unclaimed resume command;
  store membership/lineage is metadata rather than execution authority.
- Direction: persist stable workflow_run_id, graph revision/hash, checkpoint
  generation, and resume_attempt_id; resume by ID through an atomic claim/CAS,
  validate graph/current-node lineage, and reject consumed/stale attempts.
  Preserve Plan as an artifact; do not add Planning/AwaitingApproval states.
- Regression validation: cross-graph, graph-revision drift, duplicate concurrent
  resume, stale generation, crash before/after claim, and exactly-once effect.
- Validation reports: [V05](../validations/F-WFL-01/V05-01.md)

### F-WFL-01-P1-02: Interrupt-after a fan-out checkpoints only the convergence node and skips every branch

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:825`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:829`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:832`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:950`
- Reachability: any Graph with `interrupt_after(from)` whose outgoing edge is
  `Parallel { targets, then }` -> run_until_interrupt -> approve/resume.
- Expected invariant: the checkpoint retains the exact pending continuation,
  including all unexecuted fan-out targets and merge policy.
- Observed behavior: creation matches Parallel and stores only `then` as
  current_node. Resume starts at `then`, so none of the targets runs.
- Impact: an approved resumed workflow can report completion using missing
  branch outputs, silently skipping validation/research/side effects.
- Root cause: a structured NextStep is flattened to one node string.
- Direction: persist a typed continuation cursor (single/fan-out/end plus branch
  progress/merge identity) and use the same resume interpreter as fresh run.
- Regression validation: before/after interruption at single, finish, END, and
  multi-target fan-out, including restart and partial branch completion.
- Validation reports: [V06](../validations/F-WFL-01/V06-01.md)

### F-WFL-01-P1-03: Builders allow ambiguous node identities and invalid control references

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:243`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:430`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:507`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/dag.rs:232`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/dag.rs:256`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/dag.rs:289`
- Reachability: programmatic Graph/DAG builders and root DSL/loader all converge
  on these constructors.
- Expected invariant: node IDs are unique; finish/interrupt/fan-out references
  are valid and non-ambiguous; the execution bound covers every node.
- Observed behavior: Graph HashMap insertion silently replaces duplicate IDs.
  DAG validates through a HashSet, retains duplicates in node_order, then
  collapses agents into a HashMap. Finish/interrupt IDs and empty/duplicate
  parallel target lists are not validated; conditional targets fail only at
  execution. Fan-out increments steps without rechecking max_steps.
- Impact: configured topology can execute a different Agent than authored,
  execute a duplicate ID multiple times, fail only after prior side effects, or
  exceed its declared bound.
- Root cause: validation checks endpoint existence/cycles but not canonical
  identity and complete control-surface invariants.
- Direction: one reusable validator over a canonical graph definition; reject
  duplicate IDs/references, invalid finish/interrupt/fan-out, and account before
  every execution. DSL/loader remain thin adapters.
- Regression validation: table of duplicate/empty/unknown/self/conditional/
  interrupt/finish/fan-out cases and max_steps at branch boundaries.
- Validation reports: [V03](../validations/F-WFL-01/V03-01.md)

### F-WFL-01-P1-04: Graph fan-out is order-dependent serial execution with non-atomic partial merge

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:356`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:678`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:686`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/state.rs:381`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:1064`
- Reachability: public `add_parallel_edge` -> run, run_stream,
  run_until_interrupt, or resume.
- Expected invariant: advertised fan-out branches receive the same input
  snapshot, execute under one stated concurrency model, and merge conflicts/
  partial failures deterministically.
- Observed behavior: run paths loop serially. Each later fork is taken after
  earlier branch state was merged, so sibling observations depend on target
  order; scalar conflicts are overwritten by later branches. If a later branch
  fails, earlier state/side effects remain. Resume skips fork/deep_merge and
  mutates the shared state directly, so its semantics differ.
- Impact: reordering equivalent branches changes outputs; a failed workflow can
  leave a committed state/effect prefix; restart and fresh execution disagree.
- Root cause: “parallel” topology, isolation, scheduling, merge, and failure
  policy are encoded in four copied loops rather than one execution primitive.
- Direction: define one fan-out executor with one pre-fan-out snapshot, explicit
  conflict reducer, cancellation/error policy, and shared use by all run modes.
  If serial semantics are intended, rename/document them and still make merge
  atomic; do not retain four divergent loops.
- Regression validation: conflicting scalars/nested objects/messages, target
  order permutation, slow/failing branches, cancellation, and all run modes.
- Validation reports: [V04](../validations/F-WFL-01/V04-01.md)

### F-WFL-01-P1-05: FileCheckpointStore can escape its root and hides durable corruption/cleanup failure

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:359`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:378`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:409`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:433`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:443`
- Reachability: public FileCheckpointStore/CheckpointStore plus public mutable
  Checkpoint IDs and load/delete ID arguments; this is a reasonable framework
  persistence option even if EKO does not use it.
- Expected invariant: IDs stay under base_path; corrupt/truncated files are
  explicit errors; same-ID saves are serialized/atomic; cleanup failure is
  reported.
- Observed behavior: ID is joined as a path component without validation, so
  separators/parent components can target outside the root. Same-ID saves share
  one predictable temp path. `list` silently skips read/parse errors and `clear`
  discards remove errors but returns Ok; failed save leaves temp files.
- Impact: malformed IDs can overwrite/delete unintended local files; corrupt
  checkpoints disappear from recovery UI; callers believe cleanup succeeded.
- Root cause: a storage key is treated as a filesystem path fragment and
  best-effort enumeration/cleanup is presented as authoritative success.
- Direction: validate/encode opaque IDs, create paths only under a canonical
  root, synchronize unique temp/replace operations, surface corrupt entries and
  partial cleanup, and clean temp artifacts.
- Regression validation: parent/separator/Unicode IDs, symlink boundary,
  concurrent same-ID save, truncated JSON, unreadable entry, rename/remove
  failures, and restart.
- Validation reports: [V07](../validations/F-WFL-01/V07-01.md)

### F-WFL-01-P1-06: Concurrent and DAG errors detach sibling tasks, while no workflow offers an in-node deadline

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/concurrent.rs:97`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/concurrent.rs:116`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/dag.rs:134`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/dag.rs:165`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:574`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:659`
- Reachability: public Workflow implementations call `tokio::spawn` for every
  concurrent/DAG batch; Graph cancellation is public but checked only between
  awaited node executions.
- Expected invariant: cancel/error/timeout has one terminal outcome and owns all
  spawned children; no sibling continues detached after the caller gets Err.
- Observed behavior: registration-order join returns immediately on a JoinHandle
  or Agent error and drops unawaited handles, which detaches rather than aborts
  Tokio tasks. Sequential/Concurrent/DAG expose no cancellation/deadline;
  Graph's token cannot stop a hung node and there is no timeout API.
- Impact: after workflow failure/cancel expectations, siblings can continue LLM/
  tool side effects unobserved; one hung node can block indefinitely.
- Root cause: spawned task ownership and deadline/cancellation are absent from
  the common Workflow contract.
- Direction: add a generic execution context with cancellation/deadline, use
  structured task ownership (abort-and-drain or explicitly drain), and propagate
  one typed terminal outcome. EKO policy remains outside.
- Regression validation: first/middle/last task error, panic/join failure,
  cancellation, deadline, hung node, and proof all siblings terminated.
- Validation reports: [V08](../validations/F-WFL-01/V08-01.md)

### F-WFL-01-P1-07: Rejected and deferred resumes are reported as successful completion and retain the live checkpoint

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:918`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:927`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:933`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:939`
- Reachability: public `resume(checkpoint, ApprovalDecision)` with Rejected or
  Deferred.
- Expected invariant: rejection is aborted/rejected, deferral remains suspended,
  and checkpoint retention/consumption matches that terminal fact.
- Observed behavior: both decisions return `RunUntilInterruptResult::Completed`
  using the partial state/path. Neither deletes nor updates the stored
  checkpoint, so it remains resumable/listed after reported completion.
- Impact: callers/UI can record false success while missing downstream nodes,
  then later replay the still-live checkpoint and execute side effects.
- Root cause: approval decision is projected onto a two-variant
  Completed/Interrupted result that cannot represent aborted/deferred state.
- Direction: use typed workflow terminal/suspended outcomes; Deferred preserves
  one claimable checkpoint, Rejected atomically terminates/consumes it according
  to policy. This is workflow continuation state, not Plan approval state.
- Regression validation: approve/reject/defer/modified decisions, checkpoint
  list/restart, terminal monotonicity, and replay rejection.
- Validation reports: [V06](../validations/F-WFL-01/V06-01.md)

### F-WFL-01-P2-01: ToolApproval and UserRequest are misleading public interrupt capabilities with no live producer/consumer

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:55`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/checkpoint_store.rs:172`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:172`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/graph.rs:727`
- Reachability: the enum variants and `InterruptState::tool_approval` are public,
  but repository-wide search finds no graph producer/consumer; actual ReAct tool
  policy uses F-HITL-01's separate canonical path.
- Expected invariant: a public interrupt kind either has an end-to-end typed
  suspension/resume path or is clearly a data-only extension point.
- Observed behavior: Graph execution only constructs BeforeNode/AfterNode.
  pending_action is always None; ToolApproval/UserRequest are never interpreted.
- Impact: framework consumers can construct/persist states that Graph cannot
  naturally produce or correctly resume, and may mistake them for integrated
  tool approval.
- Root cause: aspirational checkpoint fields/constructors were exported before
  connection to the canonical HumanLoop continuation contract.
- Direction: either wire a typed generic suspension adapter to F-HITL's decision
  result without duplicating policy, or delete unsupported variants/fields.
- Regression validation: definition -> producer -> store -> transport -> one
  resume/deny/cancel terminal path with stable request/call identity.
- Validation reports: [V06](../validations/F-WFL-01/V06-01.md),
  [V11](../validations/F-WFL-01/V11-01.md)

### F-WFL-01-P2-02: The workflow feature is empty while workflow is always compiled and inconsistently gates examples

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/Cargo.toml:102`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/Cargo.toml:211`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/Cargo.toml:260`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/lib.rs:65`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/workflow/mod.rs:61`
- Reachability: root package always exports workflow and links
  echo_orchestration; selected examples require `workflow` while demo39 does not.
- Expected invariant: a named feature controls a coherent optional surface, or
  the capability is honestly unconditional with no inert gate.
- Observed behavior: `workflow = []` gates no module/dependency and is omitted
  from full/docs.rs. Example availability depends inconsistently on a no-op flag.
- Impact: consumers and CI cannot reason about minimal/full workflow support;
  feature-matrix claims can pass without testing isolation.
- Root cause: feature metadata remained after workflow became unconditional.
- Direction: choose one contract: gate root exports/dependencies/examples
  consistently, or delete the empty feature and all required-features entries.
- Regression validation: no-default, workflow-only, full, docs.rs, and every
  workflow example compile under the chosen contract.
- Validation reports: [V02](../validations/F-WFL-01/V02-01.md)

### F-WFL-01-P3-01: Public workflow code/documentation violates panic, UTF-8, and checked-arithmetic rules

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/sequential.rs:67`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/dag.rs:111`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/dag.rs:137`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/dag.rs:327`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/pipelines/writing_pipeline.rs:331`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/workflow/pipelines/writing_pipeline.rs:451`
- Reachability: DAG indexes occur in public run/build paths; writing pipeline
  accepts public usize config; the public SequentialWorkflow example can be
  copied/run with arbitrary Unicode output.
- Expected invariant: no direct indexing/panic-prone byte slicing; conversions
  and increments are checked/saturating.
- Observed behavior: DAG uses direct HashMap indexing, counters use unchecked
  addition, usize values are cast to i64, and the documentation slices
  `&s.output[..s.output.len().min(80)]` at a possible UTF-8 interior boundary.
- Impact: malformed/extreme configuration can panic or wrap; the public example
  teaches a crash-prone Unicode pattern. Practical overflow requires extreme
  size, hence P3.
- Root cause: builder invariants and ordinary-size assumptions substitute for
  typed safe access/arithmetic.
- Direction: use `get` plus typed errors, checked/saturating counters/conversion,
  and `chars().take(80).collect::<String>()` in documentation.
- Regression validation: malformed map/topology, numeric boundaries, and
  Chinese/emoji output example.
- Validation reports: [V09](../validations/F-WFL-01/V09-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Task identity, commits, source clean state | yes | passed | [V00](../validations/F-WFL-01/V00-01.md) |
| V01 | Definition/duplicate/layer boundary search | yes | passed | [V01](../validations/F-WFL-01/V01-01.md) |
| V02 | Export/feature/reasonable reachability trace | yes | failed | [V02](../validations/F-WFL-01/V02-01.md) |
| V03 | Node/edge/control/bound validation | yes | failed | [V03](../validations/F-WFL-01/V03-01.md) |
| V04 | Parallel isolation/merge/partial failure | yes | failed | [V04](../validations/F-WFL-01/V04-01.md) |
| V05 | Checkpoint identity/lineage/replay | yes | failed | [V05](../validations/F-WFL-01/V05-01.md) |
| V06 | Before/after/fan-out/tool interrupt/resume | yes | failed | [V06](../validations/F-WFL-01/V06-01.md) |
| V07 | File store atomicity/corruption/path/cleanup | yes | failed | [V07](../validations/F-WFL-01/V07-01.md) |
| V08 | Cancel/timeout/loop/parallel task ownership | yes | failed | [V08](../validations/F-WFL-01/V08-01.md) |
| V09 | Panic/UTF-8/overflow scan | yes | failed | [V09](../validations/F-WFL-01/V09-01.md) |
| V10 | Existing test coverage inventory | yes | failed | [V10](../validations/F-WFL-01/V10-01.md) |
| V11 | Dependency/historical boundary classification | yes | passed | [V11](../validations/F-WFL-01/V11-01.md) |
| V12 | Targeted executable regression matrix | future | not_run | [V12](../validations/F-WFL-01/V12-01.md) |
| V13 | Final links/headers/task-ID/source-clean integrity | yes | passed | [V13](../validations/F-WFL-01/V13-01.md) |
| V30 | Primary source-anchor sampling and acceptance | yes | passed | [V30](../validations/F-WFL-01/V30-01.md) |

Primary static acceptance is recorded in V30. Dynamic regressions remain
deliberately deferred to implementation and do not block the source-conclusive
findings.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| B-REF-01 stable run/turn/item/attempt identity and persisted recovery authority | current constraint; regressed by workflow checkpoint | [V05](../validations/F-WFL-01/V05-01.md) |
| B-REF-01 Plan is an artifact, not approval runtime state | current | [V01](../validations/F-WFL-01/V01-01.md), [V11](../validations/F-WFL-01/V11-01.md) |
| F-RCT-05 ReAct checkpoint/replay findings | current but distinct | [V11](../validations/F-WFL-01/V11-01.md); workflow findings use separate Graph/Checkpoint paths |
| F-HITL-01 canonical permission/HumanLoop boundary | current; workflow ToolApproval integration absent | [V06](../validations/F-WFL-01/V06-01.md), [V11](../validations/F-WFL-01/V11-01.md) |
| F-TSK-01 fixed workflow versus dynamic Task authority | current | [V01](../validations/F-WFL-01/V01-01.md) |
| workflow docs: state automatically checkpointed after each node and supports resume/replay | regressed/overstated | `echo-orchestration/src/workflow/state.rs:3`; checkpoints occur only at configured interrupts |
| Graph add_parallel_edge/module docs: simultaneous parallel fan-out | regressed/overstated | [V04](../validations/F-WFL-01/V04-01.md) |

## Coverage And Uncertainty

All requested production modules, root adapters, exports, examples, and current
tests were statically inspected. No Cargo, rustc, test, build, dynamic fixture,
or network command was run by explicit instruction. Therefore the report is
`needs_evidence`: source control-flow proves missing validation/identity/error
branches and copied execution semantics, but timing/platform filesystem behavior
and exact regression outputs still require primary executable evidence.

The review does not claim that EKO must use every workflow API, nor recommend
deleting framework capabilities because EKO does not call them. It also does
not establish whether third-party CheckpointStore implementations add their own
claim semantics; the Graph API cannot require or consume such semantics today.

## Handoff

- Primary should first validate P1-02 (fan-out resume skip), P1-01 (replay/
  cross-graph resume), and P1-06 (detached siblings), then the file-store matrix.
- F-HITL-01 owns tool permission decisions; any workflow suspension integration
  must be a thin typed continuation adapter, not a second policy engine.
- F-TSK tasks must keep TaskRun/PlanTask/SubagentRun separate from fixed workflow
  and must not solve this report with Plan approval states.
- F-FEAT-01 should consume P2-02 and select either a real workflow gate or no
  feature, with deletion of the displaced metadata.
- This report becomes stale if workflow Graph/Checkpoint/Store/Workflow APIs,
  root exports/features, built-in pipelines, or reviewed commits change.
