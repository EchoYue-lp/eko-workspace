# F-RCT-04: Tool batch execution

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Are tool validation, concurrency, timeout, cancellation, partial output,
retry, and result insertion correct for a tool batch?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/agent/react/run/phases/tools.rs` (443 lines) —
  `run_tools`: per-iteration tool-call branch. Emits `ToolBatchStart` /
  `ToolCall` events, pushes the assistant-with-tools message, splits the
  batch into serial vs concurrent sub-batches, drives the concurrent
  `FuturesUnordered` + `tokio::select!` loop (timeout / cancel-grace /
  stream-forwarding), executes the serial tail, runs `final_answer`
  verifier handoff, persists the post-batch checkpoint.
- `echo-agent/src/agent/react/run/pipeline.rs` (1–1023, stage bodies) —
  the per-tool-call middleware (`ToolExecutionPipeline::default_pipeline`)
  run by `snap.execute_tool_with_policy`. 16 stages: Intervention,
  ParseValidate, ToolVisibility, PlanMode, PreToolUseHook, Permission,
  ReadBeforeEdit, SkillPermission, CallbackStart, Audit, Execute,
  PostToolUseHook, OutputGuard, Truncation, TraceRecording, CallbackEnd.
- `echo-agent/src/agent/react/run/pipeline.rs:1035-1722` — pipeline
  test module (sampled for the partial-failure / ToolBatchEnd tests).
- `echo-agent/echo-execution/src/tools.rs` (1–913) — `ToolManager`:
  per-tool semaphore (read/write split), result cache for read-only
  tools, per-attempt timeout, retry loop with
  `ToolFailure::allows_automatic_retry` gate, streaming variant
  (`execute_tool_stream_with_context`) with `output_forwarded` guard.
- `echo-agent/echo-core/src/tools/mod.rs:17-254` — `ToolFailureCategory`
  (7 variants), `ToolRecoveryAction` (5 variants), `ToolSideEffect`,
  `ToolFailure` (`new`, `from_error`, `allows_automatic_retry`,
  `with_side_effect`/`with_idempotency_key`).
- `echo-agent/echo-core/src/tools/mod.rs:795-829` — `Tool` trait
  `exempt_from_batch_timeout`, `allows_parallel_batch_execution`,
  `manages_own_timeout` defaults.
- `echo-agent/src/agent/react/run/retry.rs` (151 lines) —
  `compute_concurrent_tool_batch_timeout` (wave-based budget formula)
  + `retry_llm_call` (LLM-level, out of scope for tool batch but
  inspected for shared retry vocabulary).
- `echo-agent/src/agent/snapshot.rs:1182-1279` —
  `execute_tool_with_policy`: constructs `ToolExecutionContext`, runs
  the pipeline, maps blocked / success / failure into
  `Result<String, ToolCallFailure>`.
- `echo-agent/src/agent/snapshot.rs:25-33` — `ToolCallFailure` struct
  (`error: ReactError`, `failure: ToolFailure`).
- `echo-agent/src/agent/react/run/processor.rs:138-183` —
  `build_tool_calls_from_map`: sorts tool calls by streaming index,
  repairs DeepSeek trailing-junk args, drops unparseable calls.
- `echo-agent/src/agent/react/run/stream_macros.rs` (79 lines) —
  `yield_event_or!`, `yield_final_event_or!`, `try_send_or!` macros.
- `echo-agent/echo-core/src/llm/types.rs:421-430` —
  `Message::tool_result(tool_call_id, name, content)`.

## Out Of Scope

Deferred to named task IDs:

- The 16 individual pipeline stage bodies (permission, hooks, audit,
  read-before-edit, plan-mode, skill-permission, output-guard,
  truncation, trace) — each is a cross-cutting concern; this task
  confirms only that the batch layer calls the pipeline and consumes
  its `Result<String, ToolCallFailure>` correctly.
- HITL approval state machine inside `PermissionStage` / approval.rs —
  the batch layer routes approval-needing tools to the serial path via
  `requires_sequential_execution`; the approval loop itself is a
  tool-execution concern.
- `ContextManager` compression / window management — **F-MEM-01**.
- LLM retry (`retry_llm_call`) — the tool batch consumes
  `compute_concurrent_tool_batch_timeout` from the same module but LLM
  retries are **F-RCT-02** / **F-LLM-01**.
- The dead `process_steps` parallel tool-batch implementation — already
  filed as F-RCT-02-P2-02; this task references it only for the
  duplicate-search conclusion.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  system reminder — especially the framework-vs-application layering
  gate, the "first check if it already exists" rule, dead-code cleanup,
  no-panic / UTF-8 safety, and the prompt-driven-over-state-machine
  guidance from the Claude Code / Codex research rule).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-RCT-02.md` (in full).
  Establishes: `run_core_loop` is the single loop body;
  `phases::tools::run_tools` is the authoritative tool-batch path; the
  dead `process_steps` duplicates batch semantics but is unreachable;
  `pipeline.rs` is the per-call middleware reached via
  `execute_tool_with_policy`. This task drills into `run_tools` and the
  pipeline, which F-RCT-02 explicitly deferred.
- `docs/comprehensive-review/zcode-glm/tasks/F-EXT-01.md` (in full).
  Establishes: the `Tool` / `ToolResult` / `ToolFailure` taxonomy is the
  single typed tool contract in `echo-core`; cancellation is
  `CancellationToken`-via-`BoxFuture`; the default
  category→recovery mapping routes `PartialSideEffect`/`Timeout` to
  `VerifyThenRetry`; the contract is authoritative for this task.

Historical documents treated as hypotheses:

- `echo-agent/src/agent/react/run/phases/tools.rs:1-3` module docstring
  claiming the branch "split sequential/concurrent batches, execute,
  dispatch verifier handoff". Treated as **current** — verified by V01.
- `echo-agent/src/agent/react/run/retry.rs:1` module docstring claiming
  "concurrent tool timeout calculation". Treated as **current** —
  verified by V03.
- `echo-agent/echo-execution/src/tools.rs:600-605` docstring claiming
  `ToolManager` is stateless w.r.t. `ToolContext` (caller passes ctx per
  call). Treated as **current** — verified by V01 (the `CtxCapturingTool`
  test `test_execute_tool_with_context_forwards_ctx` confirms).

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `run_tools` (batch orchestration), `ToolManager` (per-tool semaphore/timeout/retry), `ToolExecutionPipeline` (16-stage middleware), `compute_concurrent_tool_batch_timeout` (wave budget), and the `ToolFailure` recovery taxonomy are generic tool-runtime machinery any `echo-agent` consumer needs. They live correctly: batch orchestration + pipeline in `echo-agent` (root crate); the `Tool` contract + `ToolFailure` taxonomy in `echo-core`; the `ToolManager` executor in `echo-execution`. |
| EKO product policy | None at this layer. The batch takes framework inputs (`AgentConfig.tool_execution`, `CancellationToken`, callbacks, hooks, tool registry). EKO product policy (which tools exist, approval UI, permission mode) enters only through the injected registry / hook registry / approval provider consumed by pipeline stages. |
| Adapter boundary | `execute_tool_with_policy` is the thin seam: it constructs a `ToolExecutionContext` from the per-agent snapshot, runs the framework pipeline, and maps the pipeline's `ctx.blocked` / `result` into `Result<String, ToolCallFailure>`. No batch-level product logic leaks across this boundary. |
| Duplicate search | Searched names: `run_tools`, `execute_tool_with_policy`, `compute_concurrent_tool_batch_timeout`, `execute_tool_inner`, `execute_tool_with_context`, `execute_tool_stream_with_context`, `ToolCallFailure`, `ToolBatchEnd`, `FuturesUnordered`. Result: one canonical batch orchestrator (`run_tools` in `phases/tools.rs`); one canonical per-tool executor (`execute_tool_with_policy` → pipeline); the second `compute_concurrent_tool_batch_timeout` caller (`react_loop.rs:310`) is inside the dead `process_steps` (F-RCT-02-P2-02). No live duplicate. |
| Migration deletion | No deletion proposed in this task beyond what F-RCT-02-P2-02 already filed (dead `process_steps` + helpers). The batch layer itself is the single live implementation. |

## Current Path

Verified tool-batch call graph at commit `9b0e0fa`:

```text
run_core_loop iteration ─ tool_call_map non-empty ────────────── [stream_channel.rs:654-702]
   ↓
phases::tools::run_tools(snap, context, tx, state, iteration, think)   [tools.rs:50]
   │
   │  build_tool_calls_from_map(&think.tool_call_map)                   [:63, processor.rs:138]
   │    sort indices ascending ; repair DeepSeek trailing junk ;
   │    drop unparseable calls (keeps assistant/provider pair consistent)
   │  yield ToolBatchStart ; yield ToolCall per step ; on_think_end cb  [:64-94]
   │  push assistant_with_tools(msg_tc) into context                    [:101-113]
   │    (fallback to text assistant msg if ALL args failed parse)
   │
   │  split: requires_sequential_execution? → serial ; else concurrent  [:115-126]
   │    sequential = !allows_parallel_batch_execution() || needs_approval  [:24-38]
   │
   ├─ CONCURRENT sub-batch (if non-empty)                               [:130-301]
   │    max_concurrency = tool_manager.max_concurrency()
   │    has_timeout_exempt_tool? → batch timer = None (tool owns deadline)
   │    else → compute_concurrent_tool_batch_timeout(timeout_ms, count, mc)
   │    FuturesUnordered::push(execute_tool_with_policy(...) per call)
   │    tokio::select! { biased;
   │      cancellation_grace (if observed) → checkpoint + ToolBatchEnd + Abandoned
   │      futs.next() (result ready)      → yield ToolResult|ToolError ;
   │                                          push Message::tool_result(id,..) ;
   │                                          final_answer? verify → Finish|retry++
   │      stream_rx.recv()                → forward ToolStream events
   │      cancel (if !observed)           → set observed ; reset grace = 5s
   │      timeout (if !observed)          → forward Err(Timeout) ; return Abandoned  ★ asymmetry
   │    }
   │    post-loop: if cancellation_observed → checkpoint + ToolBatchEnd + Abandoned
   │
   ├─ SERIAL sub-batch (for each step)                                  [:303-424]
   │    if cancel_token.is_cancelled() → checkpoint + ToolBatchEnd + Abandoned
   │    execute_tool_with_policy(...) via select! {
   │      result ready / cancellation_grace / stream_rx.recv() / cancel-token
   │    }
   │    yield ToolResult|ToolError ; push Message::tool_result(id,..)
   │    final_answer? verify → Finish | retry++
   │    if cancellation_observed → checkpoint + ToolBatchEnd + Abandoned
   │    (no batch-level timeout — only per-tool ToolManager timeout applies)
   │
   │  save_runtime_checkpoint(context, None)   ← first point all calls have a result [:429]
   │  yield ToolBatchEnd                                                   [:430]
   │  if finish_output → return IterOutcome::Finish { output }             [:431-433]
   │  auto_snapshot ; periodic checkpoint                                   [:434-440]
   │  return IterOutcome::Continue                                          [:442]
   ↓
execute_tool_with_policy(call_id, name, params, input, stream_tx)     [snapshot.rs:1189]
   │  pipeline = default_pipeline() (16 stages)
   │  ctx = ToolExecutionContext { call_id, name, params, input, stream_tx, .. }
   │  pipeline.run(&mut ctx, snapshot)
   │    for stage in stages { if ctx.blocked break; stage.run(ctx, snapshot)? }
   │  map: blocked → Ok(reason) ; success → Ok(output) ;
   │       failure → Err(ToolCallFailure { error, failure }) ;
   │       pipeline-Err → Err(ToolCallFailure { from_error(..), error })
   ↓
ToolManager::execute_tool_inner / execute_tool_stream_with_context     [echo-execution/tools.rs:618, 759]
   │  acquire semaphore (read vs write split) ; read-cache hit? return
   │  for attempt in 0..=max_retries {
   │    per-attempt timeout (tokio::time::timeout) unless manages_own_timeout
   │    on Err: failure = from_error(..) ; if allows_automatic_retry retry else return
   │    on Ok(failed): if failure.allows_automatic_retry retry else return
   │  }
```

Key invariants verified (full evidence in V01–V04):

- **Single batch orchestrator.** `run_tools` is the only live
  per-iteration tool-batch driver. The `process_steps` variant
  (`react_loop.rs:177`) is dead (F-RCT-02-P2-02).
- **Pairing by `tool_call_id`.** Every result — success or error,
  concurrent or serial — is pushed as
  `Message::tool_result(id, name, output)` with the call's original id
  (`tools.rs:226, 252, 382, 408`). Provider-side matching is by id, not
  positional order.
- **Split is policy-driven.** A tool opts into the serial path by
  returning `allows_parallel_batch_execution() == false` or by requiring
  approval under the `human-loop` feature (`tools.rs:24-38`). Default is
  concurrent.
- **Two timeout layers.** Per-tool: `ToolManager` wraps each
  `tool.execute()` in `tokio::time::timeout(timeout_ms)` unless
  `manages_own_timeout()` (`echo-execution/tools.rs:686-695`). Per-batch:
  `compute_concurrent_tool_batch_timeout` scales the per-tool budget by
  retry-attempts and concurrency waves (`retry.rs:70-109`); only the
  concurrent path uses it — the serial path has no batch-level timer.
- **Cancellation has a 5-second grace; timeout does not.** On cancel,
  the loop sets `cancellation_observed` and resets a 5 s grace timer;
  in-flight tools may complete during grace; after grace (or once all
  complete) the batch emits `ToolBatchEnd`, saves a checkpoint, and
  returns Abandoned (`tools.rs:196-206, 278-300`). On batch timeout the
  loop forwards `Err(Timeout)` and returns Abandoned immediately
  (`tools.rs:284-292`) — see F-RCT-04-P2-01 for the asymmetry.
- **Retry is conservative and gated by `allows_automatic_retry`.** Only
  `Unavailable` / `Timeout` / `Transient` with `Retry` recovery AND
  (`side_effect == None` OR `idempotency_key.is_some()`) are retried
  (`echo-core/mod.rs:154-162`). `PartialSideEffect` / `Permanent` /
  `Cancelled` are never auto-retried — verified by
  `permanent_and_partial_failures_cannot_be_made_automatically_retryable`.
- **No panic on the batch path.** All error sites use `?`, `try_send`,
  or the `*_or!` macros; UTF-8-safe previews via `chars().take(200)` in
  the pipeline trace stage; semaphore acquisition uses `ok()` for reads
  and explicit `Err` mapping for writes.

## Findings

### F-RCT-04-P2-01: Concurrent batch timeout returns Abandoned without emitting `ToolBatchEnd` or saving a checkpoint (asymmetric with every other terminal path)

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/phases/tools.rs:284-292` — the
    `timeout` arm of the concurrent `tokio::select!`:
    ```rust
    _ = &mut timeout, if !cancellation_observed => {
        try_send_or!(
            tx,
            Err(ReactError::from(crate::error::ToolError::Timeout(
                "batch timeout".into()
            ))),
            IterOutcome::Abandoned
        )
    }
    ```
    `try_send_or!` (`stream_macros.rs:65-75`) evaluates `fallible`
    (here an always-`Err`), forwards `Err(e.into())` to `tx` via
    `try_send`, and `return Ok($bail)`. The function exits without
    reaching the `ToolBatchEnd` emit at `:430` or the
    `save_runtime_checkpoint` at `:429`.
  - Contrast with the 6 other `ToolBatchEnd` emit sites in the same
    function, all of which pair the event with a checkpoint save:
    `:204` (concurrent cancel-grace elapsed), `:298` (concurrent
    post-loop cancel-observed), `:311` (serial pre-iteration cancel),
    `:337` (serial cancel-grace elapsed), `:421` (serial post-exec
    cancel-observed), `:430` (normal completion). Grep confirms:
    `grep -n "AgentEvent::ToolBatchEnd" phases/tools.rs` returns exactly
    these lines; the timeout arm at `:284-292` is the only early-return
    from the concurrent batch that does not appear.
  - `echo-agent/src/agent/react/run/stream_macros.rs:65-75` — `try_send_or!`
    uses `try_send` (non-async); if the channel buffer is `Full`, the
    error is silently dropped (`let _ = $tx.try_send(Err(e.into()))`).
    In that case the caller receives `IterOutcome::Abandoned` with no
    `Err(Timeout)` event at all, losing the timeout signal.
- Reachability: any concurrent batch that exceeds
  `compute_concurrent_tool_batch_timeout`. This is the configured
  behavior whenever `tool_execution.timeout_ms > 0` and no tool in the
  batch is `exempt_from_batch_timeout`. The default
  `ToolExecutionConfig` carries a non-zero `timeout_ms`, so this path
  is live for any agent that ships the default tool-execution policy
  and issues >= 2 concurrent non-exempt tools whose combined latency
  exceeds the wave budget.
- Expected invariant: every terminal exit from the tool batch should
  (a) emit `ToolBatchEnd` to close the `ToolBatchStart` emitted at
  `:64`, and (b) persist a checkpoint so the completed tool results
  survive a restart. The cancellation paths and the normal-completion
  path both honour this; the timeout path does not.
- Observed behavior: on batch timeout the caller's event stream ends
  without a `ToolBatchEnd`. A consumer that gates "batch finished" on
  `ToolBatchEnd` (e.g. a UI spinner, the application adapter's batch
  accumulator, or a test like
  `completed_tool_batch_is_checkpointed_before_next_model_call` which
  `break`s on `ToolBatchEnd`) will hang or need its own timeout.
  Separately, because no checkpoint is saved, the tool results that
  *did* complete before the timeout are not persisted to the
  `RuntimeStateStore` — a resume after the timeout-driven abandonment
  loses them. The in-memory `ContextManager` retains the
  `assistant_with_tools([..])` message (pushed at `:110-113`) plus
  whatever subset of `tool_result` messages completed before the
  timeout; the remaining tool calls have no matching `tool_result`,
  which would make the next provider request HTTP-400 if the same
  context were reused (the turn does not continue because the driver
  receives `Abandoned`, but a caller that catches the error and retries
  on the same agent/context would hit this).
- Impact: event-stream pairing break + checkpoint gap on the timeout
  path. Functional correctness of the *answer* is unaffected (the turn
  ends with an `Error` event), but downstream consumers relying on the
  `ToolBatchStart`/`ToolBatchEnd` bracket or on checkpoint durability
  are broken specifically on this path. The asymmetry is the defect —
  the cancellation paths prove the intended contract.
- Root cause: the timeout arm was written with `try_send_or!` (the
  error-forwarding macro) rather than the `yield_final_event_or!` +
  `save_runtime_checkpoint` + `return` pattern that the cancellation
  arms use. Likely an oversight when the batch-timeout path was added
  alongside the cancellation-grace refactor: the cancel path was given
  the full cleanup sequence, the timeout path was given only the error
  forward.
- Direction: restructure the timeout arm to mirror the cancellation
  arms. Before forwarding the error, save a checkpoint (so completed
  results survive) and emit `ToolBatchEnd`:
    ```rust
    _ = &mut timeout, if !cancellation_observed => {
        snap.save_runtime_checkpoint(context, Some("Tool batch timed out".to_string())).await;
        yield_final_event_or!(tx, AgentEvent::ToolBatchEnd, IterOutcome::Abandoned);
        try_send_or!(
            tx,
            Err(ReactError::from(crate::error::ToolError::Timeout("batch timeout".into()))),
            IterOutcome::Abandoned
        );
        return Ok(IterOutcome::Abandoned);
    }
    ```
    (Ordering: emit `ToolBatchEnd` before the `Err(Timeout)` so
    consumers see the bracket close before the error; or, if the
    consumer contract is "Error terminates the stream", document that
    explicitly.) Use `send().await` (`yield_final_event_or!`) rather
    than `try_send` for the error forward if the error must not be
    dropped under buffer pressure — or accept the drop and document it.
    Add a regression test that drives a concurrent batch past
    `compute_concurrent_tool_batch_timeout` (two slow tools, tiny
    `timeout_ms`) and asserts both (a) `ToolBatchEnd` is observed and
    (b) a checkpoint is recorded.
- Regression validation: new test as above; rerun
  `cargo test --lib -p echo_agent -- tools batch concurrent timeout cancel`
  (currently 29 passed) and
  `cargo test --lib -p echo_agent completed_tool_batch_is_checkpointed_before_next_model_call`.
- Validation reports: [V03](../validations/F-RCT-04/V03-01.md),
  [V01](../validations/F-RCT-04/V01-01.md).

### F-RCT-04-P3-01: Concurrent tool results are inserted into conversation history in completion order, not call-order (nondeterministic across runs)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/phases/tools.rs:195-265` — the
    concurrent loop drives a `FuturesUnordered` and, on each
    `futs.next()` ready, immediately pushes the result:
    ```rust
    Some((id, fname, result)) = futs.next(), if !futs.is_empty() => {
        match result {
            Ok(output) => {
                yield_event_or!(tx, AgentEvent::ToolResult { call_id: id.clone(), .. }, ..);
                context.lock().await.push(Message::tool_result(id, fname.clone(), output.clone()));
                ...
            }
            ...
        }
    }
    ```
    `FuturesUnordered::next()` yields in **completion** order, which
    depends on tokio scheduling, tool latency, and system load.
  - `echo-agent/src/agent/react/run/processor.rs:141-142` — the call
    side is deterministic: `sorted_indices.sort()` orders tool calls by
    streaming index, so the `assistant_with_tools` message carries
    tool_calls in ascending index order.
  - Net: the assistant message lists calls `[call-0, call-1, call-2]`
    deterministically, but the following `tool_result` messages appear
    in whatever order the futures completed — different across runs of
    the same batch.
- Reachability: every concurrent batch with >= 2 tools whose latencies
  are close enough that completion order is scheduling-dependent. The
  default path (most tools) is concurrent.
- Expected invariant: for reproducible agent behavior, the tool-result
  ordering fed back to the model should be deterministic, ideally
  matching the call order. At minimum, the nondeterminism should be
  documented.
- Observed behavior: the `ContextManager` message tail after a
  concurrent batch is `[assistant_with_tools([call-0,call-1,call-2]),
  tool_result(call-?), tool_result(call-?), tool_result(call-?)]` where
  the last three are in completion order. The next LLM request sees
  this nondeterministic ordering. OpenAI and Anthropic both match
  `tool_result` to `tool_call` by `tool_call_id`/`tool_use_id` (the
  `Message::tool_result` constructor at `llm/types.rs:421` sets
  `tool_call_id: Some(tool_call_id)`), so the request is API-legal.
  But the model observes results in different orders across runs of
  identical inputs, which is a source of nondeterministic model
  output.
- Impact: low for API correctness (no rejection), but it undermines
  reproducibility of agent runs — the same prompt + tools + model can
  produce different reasoning simply because tool results were
  reshuffled. For eval/benchmark scenarios (echo-agent-eval) this is
  a noise source. The `ToolResult` *events* on the tx channel are
  similarly completion-ordered, so streaming consumers see
  nondeterministic event order too.
- Root cause: `FuturesUnordered` is the natural primitive for "run
  these concurrently and handle each as it completes," and pushing
  results as they arrive minimizes latency to the first result. The
  alternative — buffering all results and pushing them in call order
  after the whole batch completes — was not taken; it would add
  latency but remove the nondeterminism.
- Direction: two options.
  (a) **Buffer and order** (preferred for reproducibility): collect
  results into a `HashMap<call_id, result>` as they complete, and
  after the concurrent loop finishes (or on the terminal paths), push
  them into `context` in the original `steps` order. This makes the
  event stream and the conversation history deterministic. The latency
  cost is bounded by the slowest tool in the batch (which the batch
  timeout already bounds). Emit `ToolResult` events in the same
  deterministic order.
  (b) **Document and accept**: if the latency-to-first-event
  behavior is intentional (e.g. for streaming UX), add a doc comment
  on `run_tools` stating that concurrent results are emitted/pushed in
  completion order and that this is intentional. This is the cheaper
  fix but leaves the nondeterminism in eval/scoring paths.
  Prefer (a) unless a concrete streaming-UX requirement depends on
  completion-order emission.
- Regression validation: under (a), add a test that runs a concurrent
  batch of 3 tools with staggered latencies twice and asserts the
  `tool_result` messages in `ContextManager` are in call-id order both
  times. Under (b), doc-only.
- Validation reports: [V02](../validations/F-RCT-04/V02-01.md).

### F-RCT-04-P3-02: `VerifyThenRetry` recovery for `PartialSideEffect` / `Timeout` is advisory metadata — no framework verification gate runs before the model retries

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/tools/mod.rs:92-101` — the default
    `category → recovery` mapping routes `Timeout` and
    `PartialSideEffect` to `ToolRecoveryAction::VerifyThenRetry`.
  - `echo-agent/echo-core/src/tools/mod.rs:154-162` —
    `allows_automatic_retry()` returns `false` for `PartialSideEffect`
    (it is not in the `Unavailable`/`Timeout`/`Transient` set). So
    `ToolManager`'s retry loop (`echo-execution/tools.rs:703-723,
    879-901`) never auto-retries a `PartialSideEffect` failure —
    correct and conservative.
  - `echo-agent/src/agent/react/run/phases/tools.rs:241-263` — when a
    tool returns `Err(ToolCallFailure)`, the batch layer emits
    `ToolError`, pushes `Message::tool_result(id, name, "[Error] {error}")`
    into context, saves a checkpoint, and the loop continues. The
    `ToolFailure` (`failure.recovery == VerifyThenRetry`) is attached
    to the `AgentEvent::ToolError` event but is **not** consulted by
    the batch layer to gate anything — no verification step runs, no
    postcondition is checked, and the model is free to call the tool
    again on the next iteration.
  - `echo-agent/echo-core/src/tools/mod.rs:148-151` —
    `with_postcondition(postcondition)` stores a string but nothing in
    `phases/tools.rs` or the pipeline reads it to enforce a check.
- Reachability: every tool failure that classifies as
  `PartialSideEffect` (write/destructive tools whose
  `from_error`/explicit failure sets `side_effect != None`) or
  `Timeout` with `side_effect != None`. These flow through the batch
  as plain errors; the `recovery`/`postcondition` fields are carried
  as metadata only.
- Expected invariant: a recovery action named `VerifyThenRetry`
  implies that *something* verifies the postcondition before the
  operation is retried. The framework does not do so; the "verify" is
  expected to be performed by the model (prompt-driven) or by a
  higher-layer consumer reading the `ToolFailure` metadata.
- Observed behavior: on a `PartialSideEffect` failure, the model
  receives `[Error] {message}` plus the `failure` struct on the event.
  The conversation-history `tool_result` contains only the `[Error]`
  text — the `ToolFailure` taxonomy (category, recovery,
  side_effect, postcondition) does **not** reach the model through the
  `Message::tool_result` content. The model therefore has no signal
  that a side effect may have partially occurred and that it should
  verify before retrying. This is safe (no auto-retry of ambiguous
  state) but the "verify" half of `VerifyThenRetry` is unfulfilled at
  the framework layer.
- Impact: low. The conservative non-retry already prevents the
  dangerous case (blindly re-running a partially-applied write). But
  the taxonomy is richer than what the model sees: a tool that
  reports `PartialSideEffect` with a `postcondition` string is doing
  work the model never observes. For agents that would benefit from
  "the previous call may have partially written X; check before
  retrying," the framework does not surface that guidance.
- Root cause: design choice consistent with AGENTS.md's
  prompt-driven-over-state-machine rule (informed by the Claude Code /
  Codex research) — the framework does not impose a verification gate
  because verification is tool-specific and belongs in the prompt or
  the tool itself. The `ToolFailure` metadata is designed for
  observability (traces, UI, eval) more than for runtime gating. This
  is defensible but the `VerifyThenRetry` name oversells what the
  framework does.
- Direction: two options.
  (a) **Document** (cheapest): add a doc comment on
    `ToolRecoveryAction::VerifyThenRetry` stating that the framework
    does not enforce verification — the action is advisory, consumed
    by the model (if surfaced) or by an outer consumer; the
    framework's only enforcement is `allows_automatic_retry() ==
    false` for `PartialSideEffect`. Optionally, surface the
    `ToolFailure` summary in the `tool_result` content pushed to
    context (e.g. `[Error] {message} (category={category},
    recovery={recovery}{, side_effect={side_effect}})`) so the model
    can act on it.
  (b) **Surface to model** (small code change): when pushing the
    error `Message::tool_result`, include the `failure.category` and
    `failure.side_effect` in the content string so the model knows a
    side effect may have occurred. This makes `VerifyThenRetry`
    meaningful to the agent without adding a framework gate.
  Prefer (a) + the content-surfacing tweak. Do not add a framework-
  level verification gate — that would violate the prompt-driven
  design rule and require tool-specific verification logic in the
  framework (AGENTS.md layering violation).
- Regression validation: doc-only under (a) pure; under the
  content-surfacing tweak, `cargo test --workspace --all-features` and
  a test that asserts a `PartialSideEffect` failure produces a
  `tool_result` message mentioning the category/side_effect.
- Validation reports: [V04](../validations/F-RCT-04/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Tool-call/result pairing: every call has a matching result; split policy; pipeline reachability | yes | passed | [V01-01](../validations/F-RCT-04/V01-01.md) |
| V02 | Concurrent ordering: completion-order push confirmed; nondeterminism characterized | yes | passed | [V02-01](../validations/F-RCT-04/V02-01.md) |
| V03 | Timeout vs cancellation: two-layer timeout; 5 s cancel grace; timeout-arm asymmetry confirmed | yes | passed | [V03-01](../validations/F-RCT-04/V03-01.md) |
| V04 | Partial side-effect handling: conservative non-retry confirmed; VerifyThenRetry is metadata-only | yes | passed | [V04-01](../validations/F-RCT-04/V04-01.md) |
| V05 | Historical-document drift check | conditional | n/a | No prior F-RCT-04 report exists in this reviewer directory; the three docstrings treated as hypotheses are classified inline in the Inputs section (all current). |

Executed cargo commands (all exit 0):

```text
cargo test --lib -p echo_execution -- tools::                                    (12 passed)
cargo test --lib -p echo_execution -- timeout retry concurrency partial          (10 passed)
cargo test --lib -p echo_agent -- tools batch concurrent timeout cancel partial  (29 passed)
cargo test --lib -p echo_core -- permanent_and_partial_failures_cannot_be_made_automatically_retryable  (1 passed)
cargo test --lib -p echo_agent -- cancellation_drains_running_tool_before_abandoning_turn                (1 passed)
cargo test --lib -p echo_agent -- completed_tool_batch_is_checkpointed_before_next_model_call            (1 passed)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `phases/tools.rs:1-3` — "split sequential/concurrent batches, execute, dispatch verifier handoff to `finalize_completed_run`" | current | V01 confirms the split (`:115-126`), both execution paths, and the `final_answer` verifier handoff (`:231-239, 387-395`). |
| `retry.rs:1` — "concurrent tool timeout calculation" | current | V03 confirms `compute_concurrent_tool_batch_timeout` is the live batch-timer calculator, called from `tools.rs:170` (the `react_loop.rs:310` caller is inside dead `process_steps`). |
| `echo-execution/tools.rs:600-605` — "ToolManager itself holds no ToolContext state; ctx is passed by the caller (ExecuteStage) each time" | current | V01 confirms `ExecuteStage` constructs a per-call `ToolContext` from the snapshot (`pipeline.rs:495-508`); the `test_execute_tool_with_context_forwards_ctx` and `test_shared_tool_manager_does_not_cross_contaminate_cwd` tests pass. |
| `echo-core/tools/mod.rs:795-808` — "exempt_from_batch_timeout default false; long-running tools override to true" | current | V03 confirms the override is honoured: `tools.rs:133-139, 167-175` disable the batch timer when any concurrent tool is exempt; `InternallyTimedTool` test passes. |
| `echo-core/tools/mod.rs:811-818` — "allows_parallel_batch_execution default true; stateful tools return false" | current | V01 confirms the default-true policy and the serial-routing override in `requires_sequential_execution` (`tools.rs:24-38`). |
| F-RCT-02 handoff — "pipeline.rs is the per-tool-call middleware, not the turn machine" | current | V01 confirms `execute_tool_with_policy` (`snapshot.rs:1189`) constructs `ToolExecutionContext` and runs `default_pipeline()`; the 16 stages run per call, not per turn. |
| F-EXT-01 handoff — "PartialSideEffect/Timeout → VerifyThenRetry; allows_automatic_retry is the gate" | current | V04 confirms the mapping and the gate; `permanent_and_partial_failures_cannot_be_made_automatically_retryable` passes. |

## Coverage And Uncertainty

Inspected in full: `phases/tools.rs` (443 lines, every arm of both
select loops), `pipeline.rs` stage bodies (`:1-1023`) and the pipeline
test module (sampled for the cited tests), `echo-execution/tools.rs`
per-tool executor (`:1-913`), `retry.rs` (151 lines, both functions +
tests), `snapshot.rs:1182-1279` (`execute_tool_with_policy`), the full
`ToolFailure`/`ToolFailureCategory`/`ToolRecoveryAction` definitions in
`echo-core/mod.rs:17-254`, the `Tool` trait concurrency/timeout methods
(`:795-829`), `processor.rs:138-183` (`build_tool_calls_from_map`),
`stream_macros.rs` (79 lines), `llm/types.rs:421-430`
(`Message::tool_result`).

Not inspected (out of scope or deferred):

- The 16 individual pipeline stage bodies beyond confirming the
  ExecuteStage → ToolManager routing and the TraceRecordingStage UTF-8
  preview. Permission/approval/hook/audit stage internals are
  cross-cutting concerns; each could be its own task.
- The streaming-entry-specific batch behavior (`run_stream_channel`
  queueing) — F-RCT-02 established the loop body is shared, so the
  batch path is the same for streaming and non-streaming; the
  streaming-specific event delivery was sampled via
  `cancellation_drains_running_tool_before_abandoning_turn`.
- Application-layer consumers of `ToolBatchEnd` (chat_driver,
  task_runtime executor) — confirms the framework event contract
  exists, but whether any consumer *currently* blocks on
  `ToolBatchEnd` is an application concern. The framework test
  `completed_tool_batch_is_checkpointed_before_next_model_call` does
  block on it, proving the contract is load-bearing for at least one
  consumer.
- The `exempt_from_batch_timeout` tool implementations beyond
  `agent_dispatch` (`builtin/agent_dispatch.rs:384`) — the subagent
  dispatch tool is the known exempt tool; whether others exist is a
  tool-inventory concern (F-EXT-02).

Environmental constraints:

- All cargo commands ran against the existing incremental build cache
  (`target/`); no `cargo clean` was needed (disk pressure well below
  threshold). Final worktree state is clean (`git -C echo-agent status`
  clean, commit `9b0e0fa`).
- The feature matrix was not re-run; only the default feature set was
  exercised. The batch path is feature-independent except for the
  `#[cfg(feature = "human-loop")]` branch in
  `requires_sequential_execution` (`tools.rs:30-37`), which was
  statically inspected (it only adds the approval check to the serial
  routing predicate).

Uncertain claims:

- Whether any third-party `echo-agent` consumer or the application
  adapter currently *depends* on receiving `ToolBatchEnd` on the
  timeout path. The framework test proves the contract exists; the
  severity of F-RCT-04-P2-01 scales with whether a live consumer
  blocks on it. The defect (missing event + missing checkpoint) is
  real regardless.
- Whether the completion-order push (F-RCT-04-P3-01) is intentional
  for streaming-UX latency. The code has no comment stating intent;
  the nondeterminism is a side effect of `FuturesUnordered`, not a
  documented choice.
- Whether any concrete tool reports a `postcondition` string that the
  model would benefit from seeing. The framework supports the field
  but F-EXT-02 (builtin tools) is needed to confirm whether any tool
  populates it.

## Handoff

Conclusions downstream tasks may rely on:

1. **Single batch orchestrator confirmed.** `phases::tools::run_tools`
   is the only live per-iteration tool-batch driver. The
   `process_steps` variant is dead (F-RCT-02-P2-02). Any task that
   needs to reason about tool-call batching, concurrency, timeout, or
   cancellation can treat `run_tools` as authoritative.
2. **Pairing is by `tool_call_id`, not position.** Every result is
   pushed as `Message::tool_result(id, name, output)`. Provider-side
   matching is by id. This holds for both concurrent and serial paths
   and for both success and error results. Downstream tasks should not
   assume positional ordering of results.
3. **Two timeout layers, distinct semantics.** Per-tool:
   `ToolManager`'s `tokio::time::timeout(timeout_ms)` unless
   `manages_own_timeout()` — returns a retryable `ToolError::Timeout`.
   Per-batch: `compute_concurrent_tool_batch_timeout` — on expiry,
   forwards `Err(Timeout)` and abandons the turn (serial path has no
   batch timer). Tasks reasoning about timeout behavior must
   distinguish the two.
4. **Cancellation has a 5-second grace; timeout does not.** The cancel
   path lets in-flight tools drain for 5 s before abandoning; the
   timeout path abandons immediately. Both drop remaining futures on
   return. Tasks reasoning about partial completion must account for
   this difference.
5. **Retry is gated by `allows_automatic_retry`, conservative.** Only
   `Unavailable`/`Timeout`/`Transient` with `Retry` recovery and no
   side-effect (or an idempotency key) auto-retry.
   `PartialSideEffect`/`Permanent`/`Cancelled` never auto-retry. The
   `VerifyThenRetry` recovery action is advisory metadata, not an
   enforced gate (prompt-driven design).
6. **Batch timeout path has an event/checkpoint asymmetry (P2).** Until
   F-RCT-04-P2-01 is fixed, consumers should not rely on
   `ToolBatchEnd` or on checkpoint durability specifically on the
   batch-timeout path. The cancellation and normal-completion paths
   are correct.

Reports they must read:

- This report (F-RCT-04) for the batch-layer invariants.
- `tasks/F-RCT-02.md` for the loop-body invariants (where `run_tools`
  sits in the turn state machine) and for the dead `process_steps`
  finding.
- `tasks/F-EXT-01.md` for the `Tool`/`ToolResult`/`ToolFailure`
  contract that the batch layer consumes.
- `validations/F-RCT-04/V01-01.md` through `V04-01.md` for per-claim
  evidence.

Conditions that make this report stale:

- Any change to `run_tools`'s select-loop structure, the
  timeout/cancellation arms, or the `ToolBatchEnd`/checkpoint emit
  sites invalidates V01/V03 and the F-RCT-04-P2-01 finding.
- Changing `FuturesUnordered` to an order-preserving collector (or
  buffering results before push) invalidates V02 and resolves
  F-RCT-04-P3-01.
- Any change to `allows_automatic_retry`, the `category → recovery`
  mapping, or the `from_error` classifier invalidates V04 and the
  F-RCT-04-P3-02 finding.
- Deleting the dead `process_steps` (per F-RCT-02-P2-02) removes the
  second `compute_concurrent_tool_batch_timeout` caller but does not
  affect this report's conclusions (that caller is already identified
  as dead).

Follow-up task IDs (no fixes implemented in this review):

- A **framework robustness task** should fix F-RCT-04-P2-01 (add
  `ToolBatchEnd` + checkpoint to the timeout arm) and add the
  corresponding regression test. This is the highest-value fix in this
  report.
- A **reproducibility task** should decide F-RCT-04-P3-01 (buffer-and-
  order vs document-and-accept) in coordination with eval/echo-agent-eval
  consumers, who are most affected by tool-result nondeterminism.
- A **tool-contract documentation task** should action F-RCT-04-P3-02
  (document `VerifyThenRetry` as advisory; optionally surface
  `failure.category`/`side_effect` in the error `tool_result` content).
- **F-EXT-02** (builtin tools) should confirm which builtin tools
  return `PartialSideEffect` with a `postcondition` in practice, and
  whether any tool sets `allows_parallel_batch_execution() == false`
  beyond the approval-driven case.
