# F-RCT-04: Tool batch execution

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Are tool validation, concurrency, timeout, cancellation, partial output,
retry, and result insertion correct for a tool batch?

## Scope

- `echo-agent/src/agent/react/run/phases/tools.rs` — `run_tools` batch phase
  (full read).
- `echo-agent/src/agent/react/run/stream_channel.rs` — `run_core_loop` tools
  branch :660-687, wrapper `run_stream_channel` :35-316, batch-related tests
  :873-2088 (sampled; cancellation tests read in full).
- `echo-agent/src/agent/react/run/stream_macros.rs` — `yield_event_or!` /
  `yield_final_event_or!` / `try_send_or!` (full read).
- `echo-agent/src/agent/react/run/retry.rs` — `compute_concurrent_tool_batch_timeout`
  :69-107 (full read).
- `echo-agent/src/agent/react/run/pipeline.rs` — `ExecuteStage` :460-560,
  `TruncationStage` :716-749, `PlanModeStage` :989, `Default` pipeline
  composition :1019, tests :1045-1720 (sampled).
- `echo-agent/src/agent/react/run/processor.rs` — `build_tool_calls_from_map`
  :138-163.
- `echo-agent/src/agent/snapshot.rs` — `execute_tool_with_policy` :1189-1279,
  `tool_needs_approval` :1152-1180, `process_tool_output_for_call` :926-1060,
  cancel-token construction :449-496.
- `echo-agent/echo-execution/src/tools.rs` — `ToolManager` construction,
  `execute_tool_inner` :618-728, `execute_tool_stream_with_context` :759-900,
  `retry_delay_ms` :26-46, tests :1193-1560 (sampled).
- `echo-agent/echo-core/src/tools/mod.rs` — `ToolContext` :1001-1030,
  `ToolFailure` classification :20-198, `ToolExecutionConfig` :525-551,
  `exempt_from_batch_timeout`/`manages_own_timeout` :807-829.
- `echo-agent/echo-state/src/compression/mod.rs` — `sanitize_tool_call_pairing`
  :1562-1718 and its call sites :848, :1451, :1494; `horizon.rs` :80-260
  (grouping only).
- `echo-agent/echo-integration/src/providers/anthropic.rs` — `convert_request`
  :60-150 (tool-result serialization order).
- EKO side: `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs` :100-250,
  425-555 (stream consumption, error normalization), `infra.rs` :290-301,
  915-925, 994-1004 (tool config), `echo-agent/src/config.rs` :439-476
  (tool_timeout_ms default), `analysis.rs:421` (single-tool manager consumer),
  `web-frontend` grep for `ToolBatch` (empty).
- Executed tests: `cargo test -p echo_agent --lib --locked 'react::run::stream_channel'`
  (23 passed), `cargo test -p echo_execution --lib --locked tools` (19 passed),
  `cargo test -p echo_core --lib --locked tools` (50 passed).

## Out Of Scope

- Streaming event flow / buffer-full drop decisions → F-RCT-03 (the
  `yield_event_or!` drop behavior is referenced here only as an event-level
  pairing consequence).
- Terminal ownership of the tools branch (`finalize_completed_run`), Stop-hook
  continuation, loop detection → F-RCT-02 (P2-01/P2-04/P2-02 already filed;
  batch trace-finalization gap is filed here as part of P1-02 with
  cross-reference).
- Per-tool domain correctness (shell process cleanup, artifact writer usage)
  → F-EXT-02, F-EXT-03.
- Approval/HITL ordering of the serial split → F-HITL-01 (the serial split by
  approval is noted only as reachability).
- Resume/replay of completed batch work → F-RCT-05 (batch checkpoint sites
  verified, replay semantics not audited).
- EKO GUI/TUI rendering of dangling tool calls → A-SRF-03, A-FE-02.

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, layering, one-authority, Subagent
  terminology), shared `README.md`, `REPORTING.md`, `TASKS.md` (F-RCT-04
  card), `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-RCT-02` (complete) and `F-EXT-01`
  (complete) — used for the loop/tool authorities, `execute_tool_with_policy`
  pipeline facts, retry authority map (F-EXT-01-P3-04), and the plan-mode
  filter (F-EXT-01-P1-01/P2-01) which were cross-referenced, not re-audited.
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md` (batch
  checkpoint :96/:147, tool terminal :98, cancel-terminal :149, M4 failure
  contract :472, oversized-result claims :115/:275/:602) — classified in the
  Historical Claim Status section.

## Layering Decision

- Generic mechanism (framework): the batch phase `run_tools`, the
  `ToolExecutionPipeline`, manager-level concurrency/timeout/retry, the
  spill/truncation policy, and the pairing repair `sanitize_tool_call_pairing`
  are all correctly placed in `echo-agent`; `AgentInvocationContext`-carried
  cancel wiring is framework-internal. No repository movement is recommended
  by any finding.
- EKO product policy (application): `tool_timeout_ms = 120_000` default
  (`echo-agent/src/config.rs:476`), the decision not to configure
  `max_concurrency` (unbounded write-tool concurrency, `infra.rs:296-301`),
  the `Err`-to-`Error`-event normalization in `chat_driver.rs` + the
  framework `envelope_event_stream` adapter, and `finalize_task_mode_run`'s
  Cancelled/Failed compensation for terminal-less turns are application
  policy. The product default directly exposes the framework gaps (P1-01's
  ordering matters only for the providers EKO selects; P1-02's missing
  terminal is partly papered over in Task mode only).
- Adapter boundary: `envelope_event_stream` (echo-core/src/agent/event_envelope.rs:112-170)
  is a thin lossless error-normalization adapter with no scheduling/state
  authority. No other new boundary.
- Duplicate-search terms (both repositories; see V01-01): `run_tools`,
  `execute_tool_with_policy`, `execute_tool_feedback_raw`,
  `execute_tool_feedback`, `execute_tool(`, `ToolExecutionPipeline`,
  `execute_tool_with_context`, `execute_tool_stream_with_context`,
  `compute_concurrent_tool_batch_timeout`, `retry_delay_ms`,
  `allows_automatic_retry`, `sanitize_tool_call_pairing`, `ToolBatchStart`,
  `ToolBatchEnd`, `max_concurrency`, `Semaphore`,
  `TOOL_CANCELLATION_GRACE_PERIOD`, `exempt_from_batch_timeout`,
  `process_tool_output`. Results: one live batch authority (`run_tools`);
  the dead `process_steps` batch path (react_loop.rs:177-502) diverges
  behaviorally from the live path on timeout-exempt tools (P3-02); no batch
  machinery in `echo-agent-cli`; single repair and single spill authorities.

## Current Path

Verified data flow: LLM tool calls → `run_core_loop` tools branch
(stream_channel.rs:660-661) → `run_tools` (phases/tools.rs:50):
`build_tool_calls_from_map` sorts calls by stream index and drops
unparseable ones (processor.rs:138-163) → emits `ToolBatchStart`/`ToolCall`
events → pushes one `assistant_with_tools` message (fallback content message
when all args failed, tools.rs:95-113) → splits into serial (approval or
`allows_parallel_batch_execution()==false`) and concurrent subsets
(tools.rs:115-126) → concurrent: spawns the whole subset into
`FuturesUnordered` and selects over completion / stream events / cancel
token / batch timer (tools.rs:142-294); serial: per-tool pinned execution
with the same cancel-grace structure (tools.rs:303-424). Per call:
`execute_tool_with_policy` (snapshot.rs:1189) → 15-stage pipeline
(`ExecuteStage` builds `ToolContext` with `call_id` + `external_cancel`,
pipeline.rs:495-512) → `ToolManager::execute_tool_stream_with_context`
(semaphore → per-attempt `timeout_ms` → retry gated on
`ToolFailure::allows_automatic_retry`, echo-execution/src/tools.rs:759-900)
→ `TruncationStage` applies the single spill/truncation policy
(snapshot.rs:926-1060) → `run_tools` emits `ToolResult`/`ToolError` and
pushes `Message::tool_result` into context in completion order (concurrent)
or call order (serial). After the batch: checkpoint (tools.rs:429),
`ToolBatchEnd` (tools.rs:430), `Finish` on verifier-accepted `final_answer`
or `Continue`. Terminal/error exits from the batch: cancel → 5 s grace →
checkpoint + `ToolBatchEnd` + `Abandoned`; batch timeout → one
`try_send(Err(Timeout))` + `Abandoned` with futures dropped; both return
through the driver as `Ok(())` with no trace finalization and no typed
terminal event. Before each next LLM request, `ContextManager::prepare`
always runs `sanitize_tool_call_pairing` (mod.rs:1451-1453), which either
clears orphaned calls or inserts a placeholder result — pairing is repaired,
never reordered.

## Findings

### F-RCT-04-P1-01: Concurrent batch results are inserted into context in completion order while the assistant message carries the calls in stream-index order — strict providers (Anthropic, Gemini family) reject the next request with HTTP 400

- Priority: P1
- Confidence: medium-high (framework chain fully verified statically; the
  provider constraint is externally documented by multiple independent
  sources; no live provider run in this read-only review)
- Layer: framework (`run_tools` result insertion; provider adapters serialize
  in message order)
- Evidence: `echo-agent/src/agent/react/run/phases/tools.rs:207-240`
  (concurrent arm pushes `Message::tool_result` as each `FuturesUnordered`
  entry completes — completion order, nondeterministic);
  `processor.rs:141-147` (`build_tool_calls_from_map` sorts the assistant
  message's `tool_calls` by stream index — call order);
  `echo-integration/src/providers/anthropic.rs:60-90` (`convert_request`
  emits one `user` message per `Role::Tool` message in context order, no
  re-sort; same for the OpenAI adapter — no ordering logic anywhere, verified
  by grep); `pipeline.rs:1634-1720` (`multiplexed_streams_preserve_identity_and_terminal_order`
  enshrines completion-order terminals `["call-b","call-a"]` at the execution
  level). External constraint: Anthropic requires `tool_result` blocks in the
  same order as the corresponding `tool_use` blocks (Roo-Code issue #11804;
  Anthropic SDK example fix), Gemini requires exact order (MCP SEP-1577),
  Kimi/GLM reject out-of-order shapes (qwen-code PR #8165) — see V03-02.
- Reachability: definition → registration (framework `Agent::chat`/streaming
  entry, EKO `chat_driver.rs` main path) → live caller: any turn where the
  model emits ≥2 tool calls that are not approval-sequential, on a provider
  that enforces result order. Serial batches and single-call batches are
  unaffected. OpenAI (EKO default family) is order-insensitive.
- Expected invariant: tool results are presented to the provider in the same
  order as the tool calls they answer, or the batch executes calls in a
  provider-legal order.
- Observed behavior: on the next LLM request after an out-of-order
  completion, the context contains `assistant_with_tools([a,b])` followed by
  `tool_result(b)`, `tool_result(a)`; strict providers answer HTTP 400
  ("unexpected tool_use_id found in tool_result blocks" family), the request
  is retried 2× (react_loop retry path) and the turn ends with an opaque
  error — after the tools have already executed and their side effects
  happened.
- Impact: nondeterministic, provider-dependent mid-turn failure of the
  concurrent tool path on Anthropic/Gemini-family endpoints — the batch's
  completed work is lost to the user (results never reach the model), and the
  failure is intermittent (depends on completion timing), making it hard to
  diagnose. Violates the framework's provider-neutrality contract
  (F-LLM-01 scope).
- Root cause: `run_tools` treats the batch as completion-ordered (matching
  the execution-layer test) and never reconciles insertion order with the
  provider's ordering constraint; the constraint lives at the provider
  boundary and is unenforced.
- Direction: reorder result insertion and event emission to call order for
  the concurrent subset (buffer results and insert in `steps` order after the
  batch completes, or key by `call_id` and sort), or execute the batch in
  call order; add a provider-agnostic conformance fixture (two tools with
  staggered delays, assert context tool messages appear in assistant call
  order) plus an Anthropic conversion test asserting tool_result block order.
- Regression validation: mocked two-tool batch with staggered completion →
  context order equals `tool_calls` order; Anthropic `convert_request` unit
  test with out-of-order input messages asserting the emitted `tool_use_id`
  sequence matches the assistant order; existing
  `multiplexed_streams_preserve_identity_and_terminal_order` must be extended
  or its assertion reconciled with the new contract.
- Validation reports: [V01-01](../validations/F-RCT-04/V01-01.md),
  [V03-01](../validations/F-RCT-04/V03-01.md),
  [V03-02](../validations/F-RCT-04/V03-02.md)

### F-RCT-04-P1-02: Batch timeout and cancellation end the turn without a typed terminal — the timeout's only signal is a droppable `try_send` error, cancel emits only `ToolBatchEnd`, neither finalizes the trace run, and an already-verified `final_answer` is discarded on peer timeout

- Priority: P1
- Confidence: high (static chain fully verified; trigger requires a batch
  timeout or user cancellation, both routine)
- Layer: framework
- Evidence: `phases/tools.rs:284-292` (batch-timeout arm sends
  `Err(ReactError::from(ToolError::Timeout("batch timeout")))` via
  `try_send_or!`, which uses non-blocking `try_send` and ignores failure —
  `stream_macros.rs:65-75`; on a full buffer the error is silently dropped)
  then returns `IterOutcome::Abandoned`, dropping the in-flight futures;
  `tools.rs:295-300,418-423` (cancel path emits `ToolBatchEnd` only, then
  `Abandoned`); driver `stream_channel.rs:686-687`
  (`IterOutcome::Abandoned => return Ok(())`); wrapper forwards only
  `Err`-returned loop errors (`stream_channel.rs:305-311`); `finalize_run`
  call sites are only direct-answer (stream_channel.rs:226,235) and
  finalize.rs:175/:216/:261 — no batch path finalizes, so the trace run stays
  `Running`; `finish_output` set by an accepted `final_answer` (tools.rs:231-239)
  is discarded when the batch timer later fires (tools.rs:284-292) — the
  verified answer never becomes a `FinalAnswer` terminal; MASTER-PLAN:149
  ("取消…最终只产生一个 cancelled terminal") is regressed on this path;
  `AgentEvent::Cancelled` producers are subagent dispatch only
  (F-RCT-02-P3-03). EKO compensation: Task mode marks the run
  Cancelled/Failed itself (`chat_driver.rs:357-373`); Chat mode gets no
  signal; the envelope adapter converts the (usually delivered) timeout
  `Err` into an `AgentEvent::Error` payload (`event_envelope.rs:136-139`).
- Reachability: any tool batch that exceeds the batch budget (EKO default
  ≈ 360.9 s for one wave, `run/retry.rs:69-107` with
  `timeout_ms=120_000, max_retries=2`, infra.rs:296-301) or any user
  cancellation during a batch — both routine on the EKO main path.
- Expected invariant: a turn ends with exactly one typed terminal
  (`FinalAnswer`, `Cancelled`, or `Error`); timeout and cancellation are
  distinguishable to consumers; the trace run is finalized truthfully on
  every exit.
- Observed behavior: timeout → (error dropped or delivered) turn ends with no
  `ToolBatchEnd`, no terminal event, run left `Running`; cancel → `ToolBatchEnd`
  then channel close, no terminal, run left `Running`; an already
  verifier-accepted `final_answer` is discarded if a peer tool later trips
  the batch timer.
- Impact: consumers cannot distinguish "timed out/cancelled" from an abrupt
  success; run-history and observability (A-OBS-01, X-STA-01) show completed
  turns as perpetually running; an accepted answer is lost to the user after
  tools have already run; on buffer-full the timeout is invisible (silent
  turn end).
- Root cause: the batch exits are layered on the non-terminal
  `IterOutcome::Abandoned` + out-of-band error channel; the error is sent
  non-blockingly, and terminal ownership was never extended to the
  timeout/cancel exits (mirrors the F-RCT-02-P2-01 terminal-finalization
  gap for the batch's abnormal exits).
- Direction: on batch timeout, emit a typed error terminal (blocking send or
  ensure delivery — `yield_final_event_or!`-style) plus `ToolBatchEnd`, and
  return a distinguishable outcome; on cancel, emit `AgentEvent::Cancelled`
  (or a `ToolBatchEnd`+`Cancelled` sequence) before abandoning; finalize the
  trace run (`finalize_run(Failed/Cancelled)`) on both exits; when
  `finish_output` is already set, prefer completing the turn with the
  accepted answer (or cancel remaining peers) instead of discarding it.
- Regression validation: mocked batch where a peer tool overruns the batch
  timer after `final_answer` verified → turn ends with `FinalAnswer` (or a
  typed timeout error) and `RunStatus` finalized; a cancel-mid-batch test
  asserting a typed cancelled terminal arrives and the run status is
  `Cancelled`; a buffer-full variant asserting the error is not lost.
- Validation reports: [V02-01](../validations/F-RCT-04/V02-01.md),
  [V03-01](../validations/F-RCT-04/V03-01.md), [V05-01](../validations/F-RCT-04/V05-01.md)

### F-RCT-04-P2-01: No test anywhere exercises a concurrent tool batch — `MockLlmClient::then_tool_calls` has zero usages; the task card's required fixtures (pairing, ordering, batch timeout vs cancel, partial side effects) do not exist

- Priority: P2
- Confidence: high
- Layer: framework (test infrastructure and coverage)
- Evidence: `echo-agent/src/testing/mock_llm.rs:233` defines
  `then_tool_calls` (multi-tool parallel response); repository-wide grep of
  both source trees: zero usages; every turn-level test drives single-tool
  calls (`then_tool_call*`, stream_channel.rs tests); the only
  batch-adjacent tests are `cancellation_drains_running_tool_before_abandoning_turn`
  (stream_channel.rs:2041-2082, single tool) and the ExecuteStage-level
  `multiplexed_streams_preserve_identity_and_terminal_order`
  (pipeline.rs:1634-1720, no `run_tools` involvement); no test covers the
  serial/concurrent split, the batch timeout arm (tools.rs:284-292), the
  mixed exempt-tool path (tools.rs:133-139), or multi-result pairing.
- Reachability: not-applicable (test gap).
- Expected invariant: the framework's concurrency-heavy batch path is
  exercised by fixtures for pairing, ordering, timeout vs cancel, partial
  side effects, and oversized results (task card requirement).
- Observed behavior: the entire concurrent batch machinery is
  compile-tested only; `then_tool_calls` is dead test infrastructure.
- Impact: the two P1 defects above shipped and survive because no test can
  observe them; Q-FLT-01 and X-TOL-01 will find no fixtures to reuse; future
  batch changes have no regression net.
- Root cause: batch tests were never written when the phase was refactored
  into `run_tools`; single-tool tests were sufficient to keep the loop green.
- Direction: add the fixture family under `react::run::phases::tools`:
  (a) two-tool concurrent batch with staggered completion → pairing, event
  order, context order (must fail today per P1-01); (b) batch timeout with a
  slow tool → typed error + run finalization (must fail today per P1-02);
  (c) cancel mid-batch → typed cancelled terminal; (d) write-tool timeout →
  `PartialSideEffect`, no auto-retry; (e) oversized result in a batch →
  artifact spill path. Reuse `then_tool_calls`.
- Regression validation: the new fixtures themselves; `cargo test -p echo_agent --lib`
  stays green after the P1 fixes land.
- Validation reports: [V01-01](../validations/F-RCT-04/V01-01.md),
  [V03-01](../validations/F-RCT-04/V03-01.md), [V04-01](../validations/F-RCT-04/V04-01.md),
  [V04-02](../validations/F-RCT-04/V04-02.md)

### F-RCT-04-P2-02: Tools killed by batch timeout/cancel leave no failure record — the model is never told the calls existed or ran partially; the only context repair is a compression-attributed placeholder that never warns of possible partial side effects

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: batch kill paths drop in-flight futures without any
  `ToolError`/`ToolFailure` (tools.rs:284-292,295-300) — the manager-level
  `PartialSideEffect` classification (`echo-core/src/tools/mod.rs:165-198`,
  used at `echo-execution/src/tools.rs:715-722` for per-tool timeouts) is
  bypassed for the whole batch; the pairing repair
  `sanitize_tool_call_pairing` (`echo-state/src/compression/mod.rs:1562-1718`)
  either clears all-orphaned calls with no trace (mod.rs:1637-1645,
  `DanglingCallCleared`) or inserts
  `"[Result unavailable — tool result was removed during context compression]"`
  with name "unknown" (mod.rs:1686-1698,1702-1714); the message attributes
  the loss to compression, which is false for batch kills, and never
  mentions possible partial side effects (a timed-out `edit_file`/`shell`
  may have applied part of its work); the repair runs unconditionally in
  `ContextManager::prepare` before every LLM request (mod.rs:1451-1453,
  compact.rs:60), so the next turn's model sees either silence or the
  misleading placeholder.
- Reachability: every batch timeout or cancellation that abandons in-flight
  write tools, on every subsequent LLM request of the next turn.
- Expected invariant: a tool that may have partially executed is either
  reported as `PartialSideEffect` to the model or explicitly flagged as
  possibly-incomplete; the repair never fabricates a cause.
- Observed behavior: silent disappearance of the calls (all-orphaned) or a
  placeholder with a wrong cause and no side-effect warning.
- Impact: for a local coding agent, a killed write tool can leave a
  half-applied edit or a running process the model will never know about;
  the model may confidently continue from an incorrect disk state — a
  correctness hazard specific to the local-assistant product (AGENTS.md
  data-loss protections).
- Root cause: cancellation by future-drop has no completion path, so no
  failure record is ever produced; `sanitize_tool_call_pairing` was written
  for the compression case and its placeholder text was never generalized.
- Direction: on batch timeout/cancel, emit a synthetic per-call
  `ToolError`/`ToolResult` for in-flight calls — `PartialSideEffect` for
  write tools, `Cancelled` for read-only — and push a truthful message
  ("tool call interrupted; may have partially executed"); correct the
  sanitize placeholder text to not blame compression (e.g., "result
  unavailable — call was interrupted"), and keep the all-orphaned clear
  path only for genuine compression eviction.
- Regression validation: mocked batch with a slow write tool, timeout fired
  → the next prepared messages contain an explicit partial-side-effect note;
  sanitize unit test with an interrupted-call fixture asserting the accurate
  placeholder text (existing tests at mod.rs:2321-2376 extended).
- Validation reports: [V02-01](../validations/F-RCT-04/V02-01.md),
  [V03-01](../validations/F-RCT-04/V03-01.md), [V05-01](../validations/F-RCT-04/V05-01.md)

### F-RCT-04-P3-01: Batch concurrency is unbounded by default and the configured cap is manager-level only — `run_tools` spawns the whole concurrent subset at once, and a semaphore-permit wait sits outside the timeout and cancel bounds

- Priority: P3
- Confidence: high
- Layer: framework (`run_tools` + `ToolManager`) with EKO product default
- Evidence: `phases/tools.rs:142-160` (every concurrent step pushed into
  `FuturesUnordered`; `mc` = `max_concurrency()` is used only for the batch
  timer's wave estimate, tools.rs:131,170-175); semaphores are created only
  when configured (`echo-execution/src/tools.rs:505-511`); permit acquisition
  (`sem.acquire().await`) is outside `tokio::time::timeout` and outside any
  cancel check (tools.rs:643-665,772-803) — a permit-waiting tool is only
  freed by batch-timer/grace future-drop; `ToolExecutionConfig::default()`
  has `max_concurrency: None` (echo-core/src/tools/mod.rs:547); EKO passes
  only `timeout_ms` (infra.rs:296-301).
- Reachability: default EKO config → N-call batches run N tools
  simultaneously with no cap; framework consumers that set `max_concurrency`
  get the permit-wait hole.
- Expected invariant: either the batch bounds its own concurrency or the
  documented cap is enforced within the batch's timeout/cancel semantics.
- Observed behavior: no cap by default; with a cap, waiting is unbounded in
  time until the batch timer fires.
- Impact: resource contention for large batches (many parallel shell/file
  tools on a laptop); the configured knob's guarantee is weaker than its
  name implies.
- Root cause: concurrency was implemented as an optional manager semaphore,
  and the batch layer never wired wave gating; EKO never configured the
  knob.
- Direction: gate the `futs.push` loop with a wave/semaphore owned by the
  batch (or document `max_concurrency` as manager-global and set an EKO
  product value), and move permit acquisition inside the per-attempt timeout
  so a waiting tool is bounded; alternatively remove the misleading
  `max_concurrency` doc from the batch context.
- Regression validation: batch with 10 tools and `max_concurrency=2` →
  observed concurrent executions ≤ 2 and batch completes within the wave
  budget; permit-wait fixture with a stuck holder → wait fails within
  `timeout_ms`.
- Validation reports: [V01-01](../validations/F-RCT-04/V01-01.md),
  [V03-01](../validations/F-RCT-04/V03-01.md)

### F-RCT-04-P3-02: A batch containing one timeout-exempt tool disables the outer timer for the whole batch — ordinary peers lose the batch bound; the dead `process_steps` path separates exempt tools instead; and the `run_tools` doc claims a short-circuit that does not exist

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `phases/tools.rs:133-139,167-175` (`has_timeout_exempt_tool` →
  `bt = None` for the entire batch; the comment at :162-166 relies on
  per-tool timeouts for peers); per-tool timeout is disabled when
  `timeout_ms == 0` or the tool `manages_own_timeout()`
  (`echo-execution/src/tools.rs:686,863`; `echo-core/src/tools/mod.rs:827-829`);
  `agent_dispatch` is the only production exempt tool
  (`src/tools/builtin/agent_dispatch.rs:384-388`); the dead batch path
  `react_loop.rs:240-270` separates `exempt_indices`/`timed_indices` so
  peers keep the timer — live path diverges; doc comment
  `phases/tools.rs:40-44` ("short-circuits … the moment a `final_answer`
  tool call is verifier-accepted") — the code waits for the whole batch
  (tools.rs:195-294) and only then returns `Finish` (tools.rs:431-433).
- Reachability: any batch mixing `agent_tool` with ordinary tools; with
  EKO's default `timeout_ms=120_000` peers still get the per-tool bound, so
  today the hole requires `timeout_ms=0` config.
- Expected invariant: peers of a long-running exempt tool keep a bounded
  execution window; comments describe actual control flow.
- Observed behavior: whole-batch timer off with any exempt tool; an
  already-accepted `final_answer` does not short-circuit (related to
  P1-02's discard case); doc/code drift.
- Impact: minor today (default config still bounds peers); dead-code
  divergence is a maintenance trap when `process_steps` is deleted
  (F-RCT-02-P3-01 must preserve or discard the exempt-separation behavior
  deliberately).
- Root cause: the live implementation was simplified to a boolean while the
  dead predecessor had the finer-grained split; the short-circuit comment
  predates the wait-for-all refactor.
- Direction: either port the exempt/timed split to `run_tools` (keep a batch
  bound for peers) or document the boolean trade-off; fix the
  short-circuit comment; when deleting `process_steps` (F-RCT-02-P3-01),
  confirm no behavior is silently lost.
- Regression validation: batch with `agent_tool` + one slow ordinary tool and
  `timeout_ms=0` → peer still bounded by a batch-level timer; a
  `final_answer`-in-batch fixture asserting the batch's terminal timing
  matches the (fixed) comment.
- Validation reports: [V01-01](../validations/F-RCT-04/V01-01.md),
  [V02-01](../validations/F-RCT-04/V02-01.md), [V03-01](../validations/F-RCT-04/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (batch/execution authorities, timeout/concurrency/retry/repair helpers) | yes | passed | [V01-01](../validations/F-RCT-04/V01-01.md) |
| V02 | Registration and runtime reachability trace (driver → run_tools → pipeline → manager → context; cancel token identity; terminal/finalize inventory) | yes | passed | [V02-01](../validations/F-RCT-04/V02-01.md) |
| V03 | Invariant/edge-case inspection vs tests (pairing, concurrent ordering, timeout vs cancel, partial side effects, oversized results, fixture availability) | yes | passed | [V03-01](../validations/F-RCT-04/V03-01.md) |
| V03 | External provider ordering constraint (Anthropic/Gemini tool_result order) | yes | passed | [V03-02](../validations/F-RCT-04/V03-02.md) |
| V04 | `cargo test -p echo_agent --lib --locked 'react::run::stream_channel'` | yes | passed (exit 0; 23 passed) | [V04-01](../validations/F-RCT-04/V04-01.md) |
| V04 | `cargo test -p echo_execution --lib --locked tools` + `cargo test -p echo_core --lib --locked tools` | yes | passed (exit 0; 19 + 50 passed) | [V04-02](../validations/F-RCT-04/V04-02.md) |
| V05 | Historical-document drift (MASTER-PLAN batch checkpoint/terminal/cancel/failure-contract claims) | conditional | passed | [V05-01](../validations/F-RCT-04/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:96/:147 — batch-completion checkpoint; resume skips completed work by call_id/task node | current (checkpoint site) | `run_tools` `save_runtime_checkpoint` after all results (phases/tools.rs:429) and per-error checkpoints (:258-262,:414); replay semantics owned by F-RCT-05; [V05-01](../validations/F-RCT-04/V05-01.md) |
| MASTER-PLAN:98 — tools have explicit success/failure/cancelled terminal states | regressed (batch path) | batch cancel ends with `ToolBatchEnd` + channel close, no cancelled terminal (phases/tools.rs:295-300,418-423; P1-02) |
| MASTER-PLAN:149 — cancellation propagates everywhere and ends in exactly one cancelled terminal | regressed (main-loop batch path) | no `AgentEvent::Cancelled` on batch cancel/timeout (P1-02; consistent with F-RCT-02-P3-03) |
| MASTER-PLAN:472 (M4) — unified failure classification, limited retry, partial-side-effect category | current (manager level) / bypassed (batch kill) | `ToolFailure` categories + `allows_automatic_retry` (echo-core/src/tools/mod.rs:20-198); batch kill produces no failure record (P2-02) |
| MASTER-PLAN:115/:275/:602 — oversized results as artifact + bounded preview, shared path/hash/retention | current | `process_tool_output_for_call` spill + preview (snapshot.rs:926-1060), TruncationStage (pipeline.rs:716-749); [V03-01](../validations/F-RCT-04/V03-01.md) |
| PROJECT-ANALYSIS:245 — EKO writer concurrency note | not revalidated | owned by A-TSK-05 / F-EXT-02 |

## Coverage And Uncertainty

- All conclusions are static except three test runs (V04) and the external
  provider check (V03-02); no dynamic run exercised a concurrent batch,
  batch timeout, or batch cancel with multiple tools — no such fixtures
  exist (P2-01).
- F-RCT-04-P1-01's provider claim rests on ecosystem documentation (the
  primary Anthropic docs page was unreachable); confidence medium-high, not
  high. The OpenAI path is unaffected; the finding's reachability depends on
  EKO's provider selection.
- The batch-timeout arithmetic was checked for the EKO default config only;
  other `max_concurrency`/retry combinations were sanity-checked, not
  exhaustively validated.
- The dead `process_steps` batch implementation was inspected only for the
  exempt-tool divergence (P3-02); its other behavior was already covered by
  F-RCT-02-P3-01.
- Per-tool domain cancellation (shell process cleanup on future-drop,
  artifact-writer behavior) is F-EXT-02/F-EXT-03 scope; the framework-level
  drop semantics are recorded here only.
- ToolBatchStart/ToolBatchEnd have no frontend consumers (V01-01), so the
  missing batch-end on timeout has no UI nesting impact — noted, not a
  finding.

## Handoff

- Downstream tasks may rely on: one live batch authority and its exact
  exit paths (V02-01); completion-order result insertion and its provider
  conflict (P1-01); batch timeout/cancel terminal gaps and trace
  finalization inventory (P1-02); sanitize repair semantics (P2-02); test
  green state at the reviewed commits (V04-01/02); missing batch fixtures
  (P2-01).
- `F-RCT-03` must treat the `yield_event_or!` event-drop (stream_macros.rs:38-53)
  as an event-level pairing breaker for ToolResult under backpressure
  (referenced in V03-01) and confirm the buffer-size interplay with the
  batch timeout signal.
- `Q-FLT-01` / `X-TOL-01` should build their batch fault fixtures from
  P2-01's list (pairing, ordering, timeout vs cancel, partial side effects,
  oversized results); P1-01's fixture must fail before the fix.
- `A-CHAT-01`/`A-SRF-03` should account for the terminal-less turn end
  (P1-02) in the one-terminal invariant and the GUI's handling of
  timeout/cancel; Task mode's compensation is verified, Chat mode's is not.
- `X-BND-01` should record the batch concurrency ownership question
  (P3-01) and the exempt-tool split decision (P3-02).
- Reports to read: this report + [V01-01](../validations/F-RCT-04/V01-01.md)
  through [V05-01](../validations/F-RCT-04/V05-01.md); dependency reports
  F-RCT-02 and F-EXT-01.
- Stale triggers: any change to `phases/tools.rs` `run_tools` (order,
  timeout/cancel arms), `stream_macros.rs`, `run/retry.rs`
  `compute_concurrent_tool_batch_timeout`, `echo-execution/src/tools.rs`
  timeout/retry/semaphore logic, `sanitize_tool_call_pairing` or its
  placeholder text, `AgentEvent` variants, or provider adapters' tool-result
  serialization invalidates the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): F-RCT-03,
  Q-FLT-01, X-TOL-01, A-CHAT-01, X-BND-01, Q-TST-01 (fixture gap P2-01).
