# F-RCT-04: Tool batch execution

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: no tracked/staged source diff; external generated CLI `ApiError.ts` / `StreamingEvent.ts` changes were not read, modified, or reverted; this task changed only Codex reports

## Question

Are tool validation, concurrency, timeout, cancellation, partial output, retry,
and result insertion correct for one live ReAct tool batch?

## Scope

- Canonical `run_tools` batch construction, event emission, assistant/tool
  context insertion, concurrent and serial scheduling, cancellation, timeout,
  final-answer selection, and checkpoints.
- Tool pipeline call identity, streaming forwarding, output processing, trace,
  ToolManager permits/per-attempt timeout/retry, typed side-effect failures.
- Checkpoint pairing and resume consumers, PostToolBatch reachability, current
  tests, history, panic/UTF-8/overflow inspection.
- EKO source was searched only as needed for duplicate/reachability; external
  generated files were not opened.

## Out Of Scope

- Generic tool schema validation, duplicate registration, result kinds, binary
  output, and generic executor cancellation: F-EXT-01.
- Non-stream loop terminal ownership and dead `process_steps` as a broad
  duplicate finding: F-RCT-02.
- Primary channel event dropping/disconnect and stream terminal protocol:
  F-RCT-03.
- Full resume orchestration and completed-call skipping: F-RCT-05.
- Provider-specific ID generation promises and application reducers.
- Cargo, rustc, tests, builds, and dynamic fixtures, explicitly prohibited.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md`; Codex
  `README.md`.
- F-RCT-02 (`needs_evidence`), F-RCT-03 (`needs_evidence`), and F-EXT-01
  (`complete`) dependency reports, used only for de-duplication and boundary.
- Current source, tests, git log, and blame in the two source repositories; no
  other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Call identity, tool-result pairing, concurrency boundaries, cancellation/timeout, retry, side-effect recovery, checkpoint validity, and batch hooks are reusable framework contracts. |
| EKO product policy | EKO may choose limits, permission prompts, and rendering; it must not repair missing IDs/results or own a second scheduler. |
| Adapter boundary | Adapters project canonical ToolCall/Stream/Result/Error/BatchEnd and invoke cancellation; conversion must preserve call ID and typed outcome. |
| Duplicate search | Searched batch/parallel/sequential/timeout/retry/cancel/PostToolBatch/run_tools/process_steps/ToolResult/checkpoint definitions and callers across both repositories. One live batch loop and one dead competing processor were found. |
| Migration deletion | Retain the phase-based `run_tools` plus ToolManager per-call executor. Delete dead `process_steps` under F-RCT-02; do not implement fixes in both paths. |

## Current Path

```text
provider deltas -> tool_call_map keyed by provider index
  -> build_tool_calls_from_map (stable index order; no ID gate)
  -> run_tools
     -> emit BatchStart + every ToolCall
     -> append one assistant message containing every call
     -> partition into serial and concurrent vectors
     -> all concurrent calls via FuturesUnordered
        -> execute_tool_with_policy -> 15-stage pipeline -> ToolManager
        -> stream/result/error -> public event + context tool result
     -> all serial calls through the same pipeline
     -> first guaranteed all-paired checkpoint
     -> BatchEnd -> Finish or Continue
```

The pipeline independently canonicalizes an empty local call ID and writes
per-call trace results. Runtime recovery does not consume trace as a replay
ledger: `resume_from_state_store` restores checkpoint messages and rejects
duplicate, orphan, or incomplete pairings. ToolManager's typed retry is
conservative for possible side effects and suppresses streaming retry after
output, but this does not provide batch-atomic recovery.

## Findings

### F-RCT-04-P1-01: Partial batch checkpoints are invalid and completed side effects can replay

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/phases/tools.rs:95`, `:203`,
  `:257`, `:309`, `:336`, `:414`, `:426`; `src/state/mod.rs:155`, `:186`;
  `src/agent/react/mod.rs:1680`
- Reachability: live phase driver -> `run_tools` -> assistant containing all
  calls -> individual completions/errors -> checkpoint -> public resume.
- Expected invariant: every persisted checkpoint is provider-valid and each
  completed side effect has durable identity before interruption can replay it.
- Observed behavior: error and cancellation paths save while peers may still
  lack results; restore rejects exactly that shape. The code itself identifies
  the post-batch save as the first fully paired point. A process crash between a
  side effect and that point leaves no completed-call replay guard.
- Impact: resume can fail outright after error/cancel, or repeat an already
  completed write/execute call after a crash.
- Root cause: whole-message checkpoint atomicity is used as per-call effect
  durability, although an assistant tool-call message cannot be valid while
  only a subset of results exists.
- Direction: introduce one durable per-call execution ledger/idempotency record
  keyed by canonical call ID, then synthesize only fully paired provider
  history. Stop saving invalid partial message checkpoints; delete misleading
  “checkpoint on tool error for recovery” calls after the ledger owns recovery.
- Regression validation: two-call batch with completed write plus blocked,
  failed, cancelled, and crashed peer; resume must neither reject nor re-run the
  write.
- Validation reports: [V02](../validations/F-RCT-04/V02-01.md),
  [V10](../validations/F-RCT-04/V10-01.md)

### F-RCT-04-P1-02: Empty and duplicate call IDs execute before identity validation

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/processor.rs:53`, `:107`;
  `run/phases/tools.rs:63`, `:71`, `:143`; `run/pipeline.rs:475`;
  `state/mod.rs:194`
- Reachability: provider streaming delta -> map -> batch dispatch -> pipeline,
  public events, context, trace, checkpoint.
- Expected invariant: each call has one non-empty, batch-unique ID canonicalized
  before events or execution, reused by every projection.
- Observed behavior: empty and duplicate IDs are preserved through dispatch.
  Empty IDs are repaired only inside a local pipeline context, splitting trace
  and ToolContext from public events/history. Duplicate IDs both execute and
  later make the checkpoint unrestorable.
- Impact: two side effects can share one public identity; observers cannot join
  trace to context; recovery rejects the batch after effects have happened.
- Root cause: identity normalization is per-tool and too late, while uniqueness
  is only checked by recovery.
- Direction: atomically validate/canonicalize the whole batch before BatchStart,
  context insertion, or tool execution. Return the canonical structure to every
  consumer; delete ExecuteStage's independent empty-ID repair.
- Regression validation: empty ID and duplicate IDs at distinct provider
  indices; assert canonical cross-projection identity and zero execution for a
  rejected collision.
- Validation reports: [V03](../validations/F-RCT-04/V03-01.md),
  [V10](../validations/F-RCT-04/V10-01.md)

### F-RCT-04-P1-03: Biased stream selection can starve cancellation and timeout

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/phases/tools.rs:194`, `:266`,
  `:278`, `:284`, `:326`, `:340`, `:347`; `run/pipeline.rs:527`, `:545`
- Reachability: any streaming tool -> bounded forwarding channels -> live
  concurrent/serial select loops; cancellation token and batch timer are lower
  priority branches.
- Expected invariant: ready control signals preempt or are fairly polled within
  a bounded interval regardless of data rate.
- Observed behavior: biased selection always checks completion/stream before the
  first cancel observation and timeout. A continuously ready stream can keep
  both ready control branches from running. Serial post-result drain waits for
  channel closure outside cancellation selection.
- Impact: stop/timeout latency is unbounded for a high-rate or ill-behaved
  streaming tool; execution lease and possible effects remain live.
- Root cause: control and data plane share biased selects with data first.
- Direction: poll cancellation/deadline first or use explicit fair control
  state; bound post-result drain and define late-event discard. Preserve the
  grace period only after prompt control observation.
- Regression validation: never-idle stream with simultaneous cancel and timeout;
  assert bounded cancellation, one terminal, and no post-terminal stream event.
- Validation reports: [V04](../validations/F-RCT-04/V04-01.md),
  [V10](../validations/F-RCT-04/V10-01.md)

### F-RCT-04-P1-04: Serial/concurrent partition crosses declared ordering boundaries

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/phases/tools.rs:24`, `:115`,
  `:130`, `:303`; `echo-agent/echo-core/src/tools/mod.rs:811`;
  `echo-agent/echo-orchestration/src/tasks/task_tools.rs:38`, `:105`
- Reachability: live tools may override `allows_parallel_batch_execution`; Task
  revision tools do. Every mixed model batch is partitioned before execution.
- Expected invariant: a call that disallows peer parallelism acts as a barrier
  at its model-produced position.
- Observed behavior: all parallel-eligible calls execute first, then all serial
  calls. A serial first/middle call therefore runs after later calls.
- Impact: revision/state-dependent write-then-read or create-then-update batches
  can observe stale state or fail despite the tool explicitly opting out.
- Root cause: a boolean concurrency property is implemented as grouping rather
  than ordered barrier scheduling.
- Direction: build ordered waves separated by non-parallel barriers; retain
  concurrency only within contiguous eligible spans.
- Regression validation: serial call at first/middle/last positions with
  revision assertions and overlap probes.
- Validation reports: [V05](../validations/F-RCT-04/V05-01.md),
  [V10](../validations/F-RCT-04/V10-01.md)

### F-RCT-04-P1-05: Batch timeout abandons calls without paired outcomes or durable terminal state

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/phases/tools.rs:167`, `:284`;
  `run/stream_macros.rs:64`; `run/phases/tools.rs:426`
- Reachability: non-exempt concurrent batch -> computed outer deadline ->
  timeout branch -> immediate `Abandoned` return and future drop.
- Expected invariant: timeout assigns every announced call a result/error/unknown
  outcome, emits BatchEnd once, and persists effect identity before returning.
- Observed behavior: one raw error is attempted, then the function returns with
  no BatchEnd, per-call outcomes, or checkpoint. Already completed results and
  possible in-flight effects remain partial. A mixed batch with one exempt tool
  disables this outer deadline for all peers.
- Impact: consumers retain running/orphan calls and recovery cannot decide what
  executed; effects may be repeated. Under a full primary channel even the raw
  error is lost (owned by F-RCT-03-P1-02).
- Root cause: timeout is an early-return transport error, not a batch state
  transition.
- Direction: make timeout a typed batch transition sharing the cancellation
  safe point and per-call ledger; emit deterministic unknown/cancelled outcomes
  and one BatchEnd. Separate exempt tools into deadline groups rather than
  disabling the whole batch deadline.
- Regression validation: completed write plus hung peer, including full consumer
  buffer and mixed exempt/ordinary tools.
- Validation reports: [V06](../validations/F-RCT-04/V06-01.md),
  [V10](../validations/F-RCT-04/V10-01.md)

### F-RCT-04-P2-06: Public retry delay can overflow batch timeout calculation

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/retry.rs:70`, `:85`;
  `echo-agent/echo-core/src/tools/mod.rs:523`
- Reachability: public ToolExecutionConfig -> any ordinary concurrent batch ->
  `compute_concurrent_tool_batch_timeout`.
- Expected invariant: all representable configuration values are panic-free and
  bounded.
- Observed behavior: retry delay multiplication and summation use plain `u64`
  arithmetic before later saturating operations.
- Impact: extreme valid configuration can panic with overflow checks or wrap to
  an unexpectedly short timeout otherwise.
- Root cause: only the final budget combination is saturating.
- Direction: use a saturating fold for every retry-delay term and sum; optionally
  validate operational bounds at construction.
- Regression validation: maximum delay/retry fields yield a saturated Duration
  without panic or wrap.
- Validation reports: [V08](../validations/F-RCT-04/V08-01.md)

### F-RCT-04-P2-07: PostToolBatch exists only in the dead batch processor

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/react_loop.rs:177`, `:319`, `:455`;
  `run/phases/tools.rs:50`; `run/context.rs:365`
- Reachability: canonical phase-based batch never invokes PostToolBatch; search
  finds ReAct invocations only in no-caller `process_steps`.
- Expected invariant: a registered lifecycle advertised by the framework fires
  once from the live authority with accurate counts.
- Observed behavior: per-tool hooks run, but PostToolBatch plugins do not observe
  canonical success, failure, timeout, or cancellation.
- Impact: aggregation plugins silently miss their lifecycle despite API support;
  maintainers reading dead code can incorrectly conclude it is wired.
- Root cause: loop migration copied per-tool pipeline behavior but left batch
  lifecycle ownership in the obsolete processor.
- Direction: implement the lifecycle once in `run_tools` after outcomes are
  canonicalized, then delete its dead implementation with `process_steps`.
- Regression validation: exact cardinality/counts for success, partial failure,
  timeout, and cancellation.
- Validation reports: [V01](../validations/F-RCT-04/V01-01.md),
  [V09](../validations/F-RCT-04/V09-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, duplicate authority, live caller | yes | passed | [report](../validations/F-RCT-04/V01-01.md) |
| V02 | Partial checkpoint and side-effect recovery | yes | failed | [report](../validations/F-RCT-04/V02-01.md) |
| V03 | Empty/duplicate call-id projection | yes | failed | [report](../validations/F-RCT-04/V03-01.md) |
| V04 | Cancel/timeout fairness under stream load | yes | failed | [report](../validations/F-RCT-04/V04-01.md) |
| V05 | Mixed serial/concurrent ordering | yes | failed | [report](../validations/F-RCT-04/V05-01.md) |
| V06 | Timeout outcome and persistence | yes | failed | [report](../validations/F-RCT-04/V06-01.md) |
| V07 | Per-tool retry/side-effect classification | yes | passed | [report](../validations/F-RCT-04/V07-01.md) |
| V08 | Panic, UTF-8, and overflow inspection | yes | failed | [report](../validations/F-RCT-04/V08-01.md) |
| V09 | PostToolBatch reachability | yes | failed | [report](../validations/F-RCT-04/V09-01.md) |
| V10 | Existing test coverage inventory | yes | failed | [report](../validations/F-RCT-04/V10-01.md) |
| V11 | Targeted executable fixtures | policy-deferred | not_run | [report](../validations/F-RCT-04/V11-01.md) |
| V12 | Historical drift classification | yes | passed | [report](../validations/F-RCT-04/V12-01.md) |
| V13-01 | Report integrity, incorrect shell quoting | yes | failed | [report](../validations/F-RCT-04/V13-01.md) |
| V13-02 | Corrected report integrity and link check | yes | passed | [report](../validations/F-RCT-04/V13-02.md) |
| V30 | Primary source-anchor acceptance | yes | passed | [report](../validations/F-RCT-04/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| “Persist complete tool batch so restart never loses completed write” | current but incomplete | Full-batch save exists, but no mid-batch ledger, [V02](../validations/F-RCT-04/V02-01.md) |
| Typed failures prevent unsafe automatic retry | current | ToolManager retry gates, [V07](../validations/F-RCT-04/V07-01.md) |
| Checkpoint pairing contract prevents provider-invalid resume | current | Restore rejects invalid shapes; live partial saves violate it, [V02](../validations/F-RCT-04/V02-01.md) |
| `process_steps` is the batch authority and fires PostToolBatch | stale | no caller; live phase lacks hook, [V01](../validations/F-RCT-04/V01-01.md), [V09](../validations/F-RCT-04/V09-01.md) |

## Coverage And Uncertainty

- No dynamic confirmation was run by explicit instruction; V11 is `not_run`.
- Provider adapters may usually supply valid unique IDs, but the public live
  boundary does not enforce that invariant; provider-specific probability is
  not claimed.
- Custom tool risk/failure declarations can be dishonest; this report validates
  framework behavior for declared metadata, not arbitrary implementations.
- Oversized output artifact mechanics were reviewed by dependency F-EXT-01 and
  only batch projection/UTF-8 safety was sampled here.
- `final_answer` is parallel-eligible and multiple accepted calls select by
  completion/order, but this was left as a coverage gap to avoid expanding the
  instructed five core finding classes.
- Primary reviewer must independently sample anchors and decide acceptance;
  status remains `needs_evidence`.

## Handoff

- F-RCT-05 must read P1-01/P1-02/P1-05 before claiming completed-call skip or
  resumability; current checkpoint messages are not a sufficient effect ledger.
- Tool consumer/application reviews may rely on canonical IDs and paired
  outcomes being absent today, but should not invent adapter-side repairs.
- Remediation order: batch identity gate -> per-call durable effect ledger ->
  ordered barrier scheduler -> control-first cancellation/timeout -> batch
  terminal/hook projection -> overflow hardening. Delete dead `process_steps`
  rather than duplicating fixes.
- Primary review independently sampled all seven finding paths and accepted
  their priorities. F-RCT-05 owns save failure/exact restore; this task retains
  batch-local shape, identity, ordering, control, and hook defects. See V30.
- This report becomes stale if `run_tools`, pipeline call-id propagation,
  ToolManager retry/timeout, checkpoint pairing/resume, or PostToolBatch
  reachability changes.
