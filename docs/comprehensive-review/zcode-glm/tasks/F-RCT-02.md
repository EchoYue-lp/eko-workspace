# F-RCT-02: Non-streaming ReAct loop

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Does one non-streaming turn transition correctly through thinking,
tool batches, stopping, errors, limits, and final response?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/agent/react/run/react_loop.rs` (824 lines) — the
  non-streaming entry `run_react_loop`, `prepare_react_context`,
  `direct_answer`, and the legacy (dead) `process_steps`.
- `echo-agent/src/agent/react/run/stream_channel.rs:35-756` —
  `run_stream_channel` (streaming entry) and `AgentRunSnapshot::run_core_loop`
  (the **single unified ReAct loop body** shared by streaming and
  non-streaming paths).
- `echo-agent/src/agent/react/run/phases/mod.rs` (179 lines) —
  `LoopState`, `RunBudgetState`, and the outcome enums
  (`PrepareOutcome`, `CompactOutcome`, `ThinkOutcome`, `IterOutcome`).
- `echo-agent/src/agent/react/run/phases/prepare.rs` (325 lines) —
  `prepare_turn` (MemoryRecalled, audit, UserPromptSubmit hook, TaskNode).
- `echo-agent/src/agent/react/run/phases/compact.rs` (501 lines) —
  `run_compact` (PreCompact, pre_compaction_flush, checkpoint,
  ContextManager.prepare, PostCompact).
- `echo-agent/src/agent/react/run/phases/think.rs` (527 lines) —
  `run_think` + `create_llm_stream` (intervention, streaming, usage,
  calibration, `tools_for_request`, `cache_fingerprint`).
- `echo-agent/src/agent/react/run/phases/tools.rs` (443 lines) —
  `run_tools` (concurrent + serial batches, cancel/timeout, verifier
  handoff for `final_answer`).
- `echo-agent/src/agent/react/run/phases/verify.rs` (223 lines) —
  `verify_answer` (Critic) + `verify_final_text` (text branch).
- `echo-agent/src/agent/react/run/phases/finalize.rs` (362 lines) —
  `finalize_completed_run`, `emit_final_text`, `finalize_no_response`,
  `finalize_max_iterations`.
- `echo-agent/src/agent/react/run/processor.rs` (271 lines) —
  `process_stream_chunk`, `parse_tool_args`, `build_tool_calls_from_map`.
- `echo-agent/src/agent/react/run/retry.rs` (151 lines) — `retry_llm_call`,
  `compute_concurrent_tool_batch_timeout`.
- `echo-agent/src/agent/react/run/stream_macros.rs` (79 lines) —
  `yield_event_or`, `try_send_or`, `yield_final_event_or`, `yield_final_event`.
- `echo-agent/src/agent/react/run/direct.rs` (46 lines) — `run_direct`,
  `run_chat_direct` (non-streaming entry points).
- `echo-agent/src/agent/react/loop_detector.rs` (217 lines) —
  `LoopDetector`, `LoopDetectorConfig`, `LoopVerdict` (verified DEAD —
  see V03).
- `echo-agent/src/agent/react/run/execution.rs:1-120` —
  `ToolExecutionOutcome`/`ToolExecutionFailure` + the legacy (dead)
  `execute_tool_feedback_raw`/`execute_tool_feedback` helpers.
- `echo-agent/src/agent/react/run/pipeline.rs:1-70` — confirmed this is
  the **tool-execution pipeline** (13-stage per-call middleware), not
  the turn-level pipeline; reached via `snap.execute_tool_with_policy`.
- `echo-agent/src/agent/react/mod.rs:2767-2819` — `Agent::execute`/`chat`
  (non-streaming trait impls) routing to `run_direct`/`run_chat_direct`.
- `echo-agent/src/agent/config.rs:49-51, 176, 212-235, 531-543, 1025-1032,
  1072-1123` — `max_iterations` (default 100), `run_budget` (default
  `RunBudgetPolicy::default()` → both fields `None`),
  `loop_detector_config`, `stream_buffer_size`.
- `echo-agent/echo-core/src/agent/mod.rs:34-59` — `RunBudgetPolicy`,
  `BudgetDecision`.
- `echo-agent/src/agent/snapshot.rs:548-559` — `AgentRunSnapshot::finalize_run`.

## Out Of Scope

Deferred to named task IDs:

- The streaming-specific entry path (`run_stream_channel` queueing,
  invocation context, snapshot construction) — **F-RCT-03** if declared.
- Tool-execution-pipeline internals (the 13 `PipelineStage`s in
  `pipeline.rs`) — a tool-execution-focused task. This task confirms
  `pipeline.rs` is the per-tool-call middleware, not the turn machine.
- `ContextManager` compression algorithm correctness — **F-MEM-01**.
- LLM client / provider routing / cache hints — **F-LLM-01/02/03**.
- `TaskNode` / DAG status tracking semantics — **F-TSK-01/02**.
- Subagent executor dispatch — the subagent task.
- `IntentRouter` classification algorithm — its own task (behaviour
  validated here only as a pre-loop shortcut that bypasses `run_core_loop`).

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  system reminder — especially the framework-vs-application layering
  gate, the "first check if it already exists" rule, the dead-code
  cleanup rule, and the no-panic / UTF-8 safety rules).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-RCT-01.md` (in full).
  F-RCT-01 establishes the construction path: single tool registry,
  deterministic prompt assembly, `set_system_prompt` does not refresh
  `CanonicalContext` post-construction (handed off to this task —
  confirmed out of scope for the turn loop; the loop reads
  `ContextManager` state, not `CanonicalContext`, at runtime).
- `docs/comprehensive-review/zcode-glm/tasks/F-CORE-01.md` (in full).
  F-CORE-01 establishes `AgentEvent`, `EventEnvelope`, `ReactError`, the
  `Agent` trait, and `cancel_aware_stream`. Its finding F-CORE-01-P2-01
  (dead `GLOBAL_EVENT_BUS` infra) is the structural template for this
  task's F-RCT-02-P2-01 (dead `LoopDetector` infra).
- `docs/comprehensive-review/zcode-glm/validations/F-RCT-01/V01-01.md`
  (Category-A/B/C/D builder map) — used to confirm which config fields
  the loop reads at runtime.

Historical documents treated as hypotheses:

- `echo-agent/src/agent/react/run/mod.rs:1-12` module docstring claims
  the module owns `think / process_steps / run_react_loop`. Treated as
  **partially stale**: `process_steps` is dead code (F-RCT-02-P2-02),
  and the actual loop body is `run_core_loop` in `stream_channel.rs`.
- `echo-agent/src/agent/react/loop_detector.rs:1-7` docstring claims
  "Detects three types of loops". Treated as **stale/misleading**: the
  detector is never instantiated on the live path (F-RCT-02-P2-01).
- `echo-agent/src/agent/react/run/phases/mod.rs:1-12` docstring
  describing `run_core_loop` as the single unified loop. Treated as
  **current** — verified by V01.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `run_core_loop`, the phase functions, the outcome enums, `max_iterations` enforcement, `RunBudgetPolicy`, `retry_llm_call`, and `finalize_run` are generic agent-runtime machinery any `echo-agent` consumer needs. They live correctly in `echo-agent` (root crate) and `echo-core` (`RunBudgetPolicy`, `BudgetDecision`). |
| EKO product policy | None at this layer. The loop takes pure framework inputs (`AgentConfig` budgets, callbacks, intervention callbacks, hooks); EKO product policy enters only through injected adapters (the `Critic`, `InterventionCallback`, `HookRegistry`, `RuntimeStateStore`) all supplied by the caller. |
| Adapter boundary | The non-streaming entry `run_react_loop` is a thin wrapper: it locks `execution_mutex`, prepares context, optionally short-circuits via `IntentRouter`, spawns `run_core_loop`, and collects `FinalAnswer`/`Error` from the channel. No scheduling authority, no state machine of its own — the loop body owns all transitions. |
| Duplicate search | Searched names: `run_core_loop`, `run_react_loop`, `run_loop`, `run_direct`, `run_chat_direct`, `process_steps`, `think`, `run_think`, `run_tools`, `run_compact`, `prepare_turn`, `verify_answer`, `verify_final_text`, `finalize_completed_run`, `emit_final_text`, `finalize_no_response`, `finalize_max_iterations`, `LoopDetector`, `LoopDetectorConfig`, `loop_detector_config`, `execute_tool_feedback_raw`, `execute_tool_feedback`, `ToolExecutionOutcome`, `execute_tool_with_policy`. Searched behaviours: think→tools→finalize transition, terminal-event emission, max_iterations enforcement, loop-detection record/check. Result: one canonical loop body (`run_core_loop`); two dead parallel implementations (`process_steps`, `LoopDetector`). |
| Migration deletion | Deletion candidates identified (no migration implemented in this review): (a) `LoopDetector` + `LoopDetectorConfig` + `loop_detector_config` field + builder/accessor (F-RCT-02-P2-01); (b) `process_steps` + `execute_tool_feedback_raw` + `execute_tool_feedback` + `ToolExecutionOutcome` (F-RCT-02-P2-02). |

## Current Path

Verified non-streaming turn call graph at commit `9b0e0fa`:

```text
Agent::execute(task) / Agent::chat(message)        [mod.rs:2767, 2797]
   ↓
ReactAgent::run_direct / run_chat_direct            [direct.rs:9, 29]
   ↓
ReactAgent::run_react_loop(message)                 [react_loop.rs:598]
   │  ★ execution_mutex.lock().await                [:600]  (one run per agent)
   │  prepare_react_context(message)                [:603, 508-591]
   │      AgentTurn::new / clear_read_files /
   │      input guard check (Ok|Block→return Ok(msg)) /
   │      start_trace_run / Recall / detect_and_write_memory_triggers /
   │      recall_long_term_memories / push user message
   │  intent_router.classify →
   │      DirectAnswer + allows_direct_answer_shortcut → direct_answer() RETURN  [:623-656]
   │      SkillRequired → activate_skill (fall through)
   │      Fallback / DirectAnswer(no shortcut) → fall through
   │  (tx, rx) = mpsc::channel(stream_buffer_size=256)            [:686]
   │  snap = AgentRunSnapshot::from_agent(self)                   [:687-706]
   │  tokio::spawn(run_core_loop(snap, ctx, text, …, StreamMode::Chat, recalled, tx))  [:711-727]
   │  loop rx.recv():
   │      FinalAnswer(a) → Ok(a)        [:733]
   │      Cancelled       → Ok("Cancelled.")  [:737]
   │      Error{message}  → Err(Other(message))  [:741]
   │      Err(e)          → Err(e)        [:744]
   ↓
AgentRunSnapshot::run_core_loop(self, context, text, …, tx)   [stream_channel.rs:494]
   │  state = prepare_turn(…)? → LoopState | BlockedAndDone|Abandoned → return Ok(())  [:509-516]
   │  max_iterations = (config.max_iterations==0) ? usize::MAX : config.max_iterations  [:521-527]
   │  for iteration in 0..max_iterations {                                            [:528]
   │      cb.on_iteration / drain_steer / wind_down (once) / final_only (once)        [:529-583]
   │      messages = run_compact(…)? → Continue(m) | Abandoned → return Ok(())        [:589-595]
   │      think = run_think(…, messages, final_only)? →                                                [:599-610]
   │                Continue(ThinkOutput) | Abandoned|Cancelled|Blocked → return Ok(())
   │      state.budget.record_usage(pt+ct) ; max_model_tokens → set final_only        [:612-641]
   │      if final_only && tool_calls → continue (tools blocked)                       [:643-651]
   │      outcome =                                                                     [:654-702]
   │          !tool_call_map.is_empty()? run_tools(…)?            → IterOutcome
   │        : !content_buffer.is_empty()? verify_final_text(…)?  → (Continue|FinalText|other)
   │        :                                  NoResponse
   │      match outcome {                                                               [:704-751]
   │          Continue       → continue
   │          Finish{output} → drain_steer; return finalize_completed_run(…)   [finalize.rs:23]
   │          FinalText{..}  → emit_final_text(…)? → Continue→continue | Break→return  [:681-696, 728-744]
   │          NoResponse     → return finalize_no_response(…)                  [finalize.rs:211]
   │          Abandoned      → return Ok(())
   │      }
   │  }
   │  finalize_max_iterations(…)                                               [:755, finalize.rs:234]
```

Per-iteration branch decision (the core transition logic):

```text
ThinkOutput { tool_call_map, content_buffer, … }
   │
   ├─ !tool_call_map.is_empty()  → run_tools
   │     ├─ verifier-pass on final_answer tool → IterOutcome::Finish{output}
   │     ├─ verifier-fail on final_answer     → verifier_retry_count++ ; Continue
   │     ├─ any tool error                     → tool_result "[Error] …" ; Continue
   │     └─ otherwise (no final_answer accepted) → Continue
   ├─ tool_call_map empty && !content_buffer.is_empty() → verify_final_text
   │     ├─ verifier-pass → IterOutcome::FinalText{answer, reasoning}
   │     └─ verifier-fail → push assistant attempt ; verifier_retry_count++ ; Continue
   └─ both empty → IterOutcome::NoResponse
```

Key invariants verified by this graph (full evidence in V01–V04):

- **Single loop body.** Both `Agent::execute`/`chat` (non-streaming)
  and `Agent::execute_stream`/`chat_stream` (streaming) reach
  `run_core_loop`. The non-streaming wrapper spawns it in a `tokio::task`
  and collects via a local `mpsc::channel`; the streaming wrapper
  returns the receiver as a `BoxStream`. Confirmed by `phases/mod.rs:1-4`
  docstring and by the spawn at `react_loop.rs:711-727`.
- **Typed transitions.** Every phase boundary returns a dedicated
  outcome enum (`PrepareOutcome`, `CompactOutcome`, `ThinkOutcome`,
  `IterOutcome`) consumed by an exhaustive `match` in the driver. No
  phase mutates loop control flow except by returning its outcome.
- **Exhaustive terminal partition.** The four `finalize_*` helpers plus
  the `Abandoned`/`Cancelled`/`Blocked` short-circuits cover every way
  the loop can exit (V02 lists all 10 terminal arms).
- **`max_iterations` is a hard ceiling.** `for iteration in
  0..max_iterations` (`:528`) with the `0 → usize::MAX` rewrite
  (`:521-527`). Default 100 (`config.rs:212`).
- **No panic on the live path.** All error sites use `?`, `try_send_or!`,
  or `try_send`; no `unwrap`/`expect`/indexing on external input in the
  loop body (UTF-8-safe previews via `chars().take(200)` at
  `react_loop.rs:69, 112` and `tools.rs` string formatting).

## Findings

### F-RCT-02-P2-01: `LoopDetector` and its config plumbing are dead infrastructure that advertises loop detection the runtime never performs

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/loop_detector.rs:1-7` — module docstring
    advertises "Detects three types of loops: 1. Exact duplicate …
    2. Same-tool failure … 3. No-progress …".
  - `echo-agent/src/agent/react/loop_detector.rs:41-151` — `LoopDetector`
    struct, `record_tool_call`/`record_iteration`/`check`/`reset` methods.
  - `echo-agent/src/agent/config.rs:176` —
    `pub(crate) loop_detector_config: LoopDetectorConfig`.
  - `echo-agent/src/agent/config.rs:264` — default initialiser.
  - `echo-agent/src/agent/config.rs:1025-1032` — `.loop_detector(...)`
    builder method + `.get_loop_detector_config()` accessor.
  - `echo-agent/src/agent/react/mod.rs:65` — `pub mod loop_detector`.
- Reachability: definition + registration only.
  `LoopDetector::new(...)` is called **only** inside the module's own
  `#[cfg(test)]` block (`loop_detector.rs:159, 168, 180, 193, 206`).
  `record_tool_call`/`record_iteration`/`check` have zero production
  callers. Neither `echo-agent` nor `echo-agent-cli` invokes the
  builder method `.loop_detector(...)` (grep across both repos: zero
  matches outside `config.rs`). `run_core_loop` never references
  `loop_detector` at all.
- Expected invariant: a public framework module + config field +
  builder method that advertise loop-detection should either be wired
  into at least one producer/consumer path, or removed.
- Observed behavior: `LoopDetector` is compiled in, the config field is
  populated on every agent, the accessor is callable — but no runtime
  path constructs a `LoopDetector` or consults it. The actual
  loop-protection is the hard `max_iterations` ceiling (default 100)
  plus the two optional `RunBudgetPolicy` soft budgets. An agent that
  calls the same read-only tool with identical args 99 times burns 99
  LLM rounds undetected.
- Impact: misleads API consumers (third-party `echo-agent` users,
  reviewers, new contributors) into believing loop detection is active.
  Under AGENTS.md "code cleanup: no compatibility burden", this is
  exactly the kind of dead path that should not be kept. Structurally
  identical to F-CORE-01-P2-01 (dead `GLOBAL_EVENT_BUS`).
- Root cause: the detector was scaffolded (its tests even pass) but
  never wired into the loop. The later `RunBudgetPolicy`
  (wind_down / final_only) was added as the actual soft-budget
  mechanism, leaving `LoopDetector` orphaned.
- Direction: choose one.
  (a) **Delete** `loop_detector.rs`, the `pub mod loop_detector` line
  at `mod.rs:65`, the `loop_detector_config` field + default at
  `config.rs:176, 264`, and the builder/accessor at `config.rs:1025-1032`.
  Preferred under the cleanup rule — no concrete consumer exists.
  (b) **Wire it**: have `run_tools` call `record_tool_call(name, args,
  success)` per tool result, have the driver call `record_iteration()`
  + `check()` per iteration, and honour `LoopVerdict::Break` by routing
  to a new `finalize_loop_detected` terminal. This is a real capability
  addition; only choose it if product wants the detection.
  Do not keep the unwired scaffolding.
- Regression validation: after (a), `cargo check --workspace` and
  `cargo check -p echo_agent --no-default-features --locked` must pass;
  no caller should be affected (none today). After (b), add a test that
  three identical `read_file` calls produce a `Break` terminal.
- Validation reports: [V03](../validations/F-RCT-02/V03-01.md).

### F-RCT-02-P2-02: `process_steps` and its helper chain are dead code (the non-streaming loop uses the unified `run_core_loop`)

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/react_loop.rs:177` —
    `#[allow(dead_code)] pub(crate) async fn process_steps(...)`.
  - `echo-agent/src/agent/react/run/react_loop.rs:179-502` — the full
    ~325-line non-streaming tool-batch implementation (approval split,
    concurrent/exempt batches, PostToolBatch hook, final_answer
    extraction). Never called.
  - `echo-agent/src/agent/react/run/execution.rs:17-22` —
    `#[allow(dead_code)] pub(crate) struct ToolExecutionOutcome`.
  - `echo-agent/src/agent/react/run/execution.rs:168` —
    `execute_tool_feedback_raw` (only caller: `process_steps` at
    `react_loop.rs:306, 359`).
  - `echo-agent/src/agent/react/run/execution.rs:404-410` —
    `execute_tool_feedback` (only caller: `process_steps` at
    `react_loop.rs:484`).
  - `echo-agent/src/agent/react/run/pipeline.rs:3` — comment: "Replaces
    the monolithic `execute_tool_feedback_raw` with a configurable
    pipeline".
- Reachability: zero live callers of `process_steps`,
  `execute_tool_feedback_raw`, `execute_tool_feedback`,
  `ToolExecutionOutcome`, or `ToolExecutionFailure` outside their own
  definitions. The live non-streaming path is `run_react_loop` → spawns
  `run_core_loop` → `phases::tools::run_tools` →
  `snap.execute_tool_with_policy` → `pipeline.rs` (13-stage middleware).
  Confirmed by `grep -rn "process_steps\|execute_tool_feedback\|
  ToolExecutionOutcome\|ToolExecutionFailure" echo-agent/src
  echo-agent-cli --include=*.rs` — the only hits outside
  `execution.rs` are inside the dead `process_steps` body
  (`react_loop.rs:292, 297, 378`).
- Expected invariant: under AGENTS.md "code cleanup", legacy code that
  is fully superseded by a new implementation should be deleted, not
  retained with `#[allow(dead_code)]`.
- Observed behavior: `process_steps` and its helpers compile in, are
  documented in the `run/mod.rs:8` module doc ("ReAct loop core
  (think / process_steps / run_react_loop)"), and appear in the
  `react/mod.rs:8` table — but no production path reaches them. They
  duplicate the tool-batch semantics that `phases::tools::run_tools`
  now owns authoritatively.
- Impact: maintenance burden and reviewer confusion. The two
  implementations have subtly diverged (e.g. `process_steps` does not
  run the 13-stage pipeline, does not support cancellation grace
  periods, and uses `execute_tool_feedback_raw` which bypasses
  `execute_tool_with_policy`). A future contributor editing
  `process_steps` believing it is live would introduce a phantom
  code path.
- Root cause: `process_steps` was the pre-unification non-streaming
  tool batch. When `run_core_loop` was introduced as the single loop
  body (shared by streaming and non-streaming), the non-streaming
  `run_react_loop` was rewritten to spawn `run_core_loop`, but
  `process_steps` was left in place with `#[allow(dead_code)]` rather
  than deleted.
- Direction: delete `process_steps` (`react_loop.rs:177-502`),
  `execute_tool_feedback_raw` and `execute_tool_feedback`
  (`execution.rs:168-425`), and both
  `ToolExecutionOutcome` (`execution.rs:17-22`) and
  `ToolExecutionFailure` (`execution.rs:24-27`). A repo-wide grep
  confirms both structs are referenced **only** from `react_loop.rs`
  (the dead `process_steps` at `:292, 297, 378`); the live
  `phases/tools.rs` path returns `Result<IterOutcome>` and consumes the
  error type of `snap.execute_tool_with_policy`, neither of which is
  `ToolExecutionFailure`. Also refresh the two stale module docstrings
  (`run/mod.rs:8`, `react/mod.rs:8`) to drop the `process_steps`
  reference.
- Regression validation: `cargo test --workspace --all-features --locked`;
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
  No behavioural test should change because the deleted code was
  unreachable.
- Validation reports: [V01](../validations/F-RCT-02/V01-01.md).

### F-RCT-02-P2-03: `finalize_completed_run` (tool-branch success) does not finalize the trace run, unlike the text branch

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/phases/finalize.rs:23-112` —
    `finalize_completed_run`. No call to `snap.finalize_run(...)` anywhere
    in the function body.
  - `echo-agent/src/agent/react/run/phases/finalize.rs:175` —
    `emit_final_text` calls
    `snap.finalize_run(RunStatus::Completed, Some(&answer), None)`.
  - `echo-agent/src/agent/react/run/phases/finalize.rs:216` —
    `finalize_no_response` calls `snap.finalize_run(RunStatus::Failed, …)`.
  - `echo-agent/src/agent/react/run/phases/finalize.rs:261` —
    `finalize_max_iterations` calls `snap.finalize_run(RunStatus::Failed, …)`.
  - `echo-agent/src/agent/snapshot.rs:548-559` — `finalize_run` mutates
    `run.status`, `run.final_output`, `run.error`, `run.finished_at` and
    saves to the `RunStore`. Skipping it leaves the run in `Running`.
  - `grep -n "finalize_run" phases/finalize.rs` returns exactly 3 hits
    (lines 175, 216, 261) — none inside `finalize_completed_run`.
- Reachability: every successful tool-based turn. When the LLM calls
  the `final_answer` tool and it passes the verifier, the driver takes
  the `IterOutcome::Finish { output }` arm (`stream_channel.rs:708-716`)
  and calls `finalize_completed_run`. This is the **normal happy path**
  for a tool-using agent (think → tool → think → final_answer tool).
  The text-only success path (`emit_final_text`) is the exception, not
  the rule, for an agent equipped with tools.
- Expected invariant: every terminal path that completes the turn
  should finalize the trace run with the same status it asserts
  elsewhere (Completed for success, Failed for failure). The four
  `finalize_*` helpers should be symmetric on trace finalization.
- Observed behavior: a successful tool-based turn yields the correct
  `FinalAnswer` event to the caller, fires `SessionEnd("complete")`,
  updates the TaskNode to `Success`, saves the transcript projection —
  but the trace run's `status` stays `Running` and `finished_at` stays
  `None` forever. Any `RunStore` query that lists runs by status will
  show these completed runs as still in progress.
- Impact: observability gap on the primary success path. The trace
  store (`InMemoryRunStore`, file-backed stores, or the application
  `RunStore` adapter) accumulates runs stuck in `Running`. Dashboards
  that count active runs, or queries that filter by `Completed`, will
  be wrong. Not a user-facing correctness defect — the answer is
  delivered correctly — but the framework's own observability contract
  (`finalize_run` exists precisely for this) is violated asymmetrically.
- Root cause: `emit_final_text` was written with `finalize_run` in
  place; `finalize_completed_run` was added/refactored separately (the
  tools branch) and the `finalize_run` call was omitted, likely an
  oversight during the streaming/tools-branch refactor. No test
  asserts trace status after `finalize_completed_run` — the existing
  `react_stream_records_real_usage_in_run_trace` test loads the run and
  checks events but not `run.status == Completed`.
- Direction: add
  `snap.finalize_run(crate::trace::RunStatus::Completed, Some(output), None).await;`
  to `finalize_completed_run` immediately before the `yield_final_event!`
  at `finalize.rs:87` (mirroring `emit_final_text:175-179`). Then add a
  regression test that drives a `final_answer` tool turn and asserts
  `run.status == Completed` and `run.finished_at.is_some()` in the
  `RunStore`.
- Regression validation: new test (described above); rerun
  `cargo test --lib -p echo_agent run_core_loop_tool_call_cycle_completes`
  to confirm the turn still completes; add a trace-status assertion.
- Validation reports: [V02](../validations/F-RCT-02/V02-01.md),
  [V04](../validations/F-RCT-02/V04-01.md).

### F-RCT-02-P3-01: LLM-error and intervention short-circuit paths also skip trace finalization (compounds F-RCT-02-P2-03)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/phases/think.rs:99-103` —
    `create_llm_stream` failure is wrapped in `try_send_or!(tx, …,
    ThinkOutcome::Abandoned)`.
  - `echo-agent/src/agent/react/run/stream_macros.rs:65-75` — `try_send_or!`
    forwards `Err` to `tx` and returns `Ok($bail)`.
  - `echo-agent/src/agent/react/run/stream_channel.rs:605-609` — the
    driver matches `Abandoned | Cancelled | Blocked` and returns `Ok(())`
    with no `finalize_run` call.
  - `echo-agent/src/agent/react/run/phases/think.rs:42-79` — Cancelled/
    Blocked forward the error via `tx.try_send(Err(...))` and update the
    TaskNode, but do not finalize the trace.
- Reachability: every LLM call that fails after `retry_llm_call`
  exhausts retries (network error, provider 5xx, non-retryable error),
  and every intervention cancel/block at think-start.
- Expected invariant: same as F-RCT-02-P2-03 — terminal paths should
  finalize the trace.
- Observed behavior: the error is correctly forwarded to the caller of
  `run_react_loop` (the non-streaming collector returns `Err(e)` at
  `react_loop.rs:744`), but the trace run remains in `Running` status.
  Compare with `finalize_no_response`/`finalize_max_iterations` which
  mark the trace `Failed`.
- Impact: same observability gap as F-RCT-02-P2-03, on the error paths.
  Lower priority than P2-03 because these are failure paths (already
  noisy) rather than the happy path, but they compound the same
  dangling-run problem.
- Root cause: the `Abandoned`/`Cancelled`/`Blocked` arms were designed
  as "graceful stream exit" and pre-date the trace-finalization
  contract that the `finalize_*` helpers enforce. The `try_send_or!`
  macro's `return Ok($bail)` short-circuit bypasses any cleanup the
  driver might otherwise do.
- Direction: either (a) add a `finalize_run(Failed, None,
  Some(error_text))` call before each `return Ok($bail)` in the
  Abandoned/Cancelled/Blocked arms (preferred for symmetry), or (b)
  introduce a single `finalize_abandoned(error)` helper that the arms
  call. Ensure the error string is captured for the trace.
- Regression validation: a test that forces `create_llm_stream` to fail
  (empty mock queue, as in
  `run_core_loop_empty_llm_response_terminates_gracefully`) and asserts
  `run.status == Failed` in the store.
- Validation reports: [V04](../validations/F-RCT-02/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Turn state machine transition trace (prepare → think → tools/verify → finalize) | yes | passed | [V01-01](../validations/F-RCT-02/V01-01.md) |
| V02 | Terminal ownership enumeration (10 arms, single FinalAnswer owner each) | yes | passed | [V02-01](../validations/F-RCT-02/V02-01.md) |
| V03 | `max_iterations` enforcement + `LoopDetector` dead-infra proof (static + `cargo test`) | yes | passed | [V03-01](../validations/F-RCT-02/V03-01.md) |
| V04 | LLM/tool/parse error handling (static + `cargo test` terminal-path tests) | yes | passed | [V04-01](../validations/F-RCT-02/V04-01.md) |
| V05 | Historical-document drift check | conditional | n/a | No prior F-RCT-02 report exists in this reviewer directory; historical-claim classification is inline in the Inputs section (two stale docstrings, one current). |

Executed cargo commands (all exit 0):

```text
cargo test --lib -p echo_agent run_core_loop_            (3 passed)
cargo test --lib -p echo_agent -- budget finalize iteration_wind_down  (12 passed)
cargo test --lib -p echo_agent loop_detector::          (5 passed — tests the isolated struct, not runtime wiring)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `run/mod.rs:8` — "react_loop — ReAct loop core (think / process_steps / run_react_loop)" | partially stale | `run_react_loop` is live; the actual loop body is `run_core_loop` in `stream_channel.rs`; `process_steps` is dead (F-RCT-02-P2-02). The `think` fn does not exist by that name — its successor is `phases::think::run_think`. |
| `react/mod.rs:8` — "`run.rs` | Execution engine (`think` / `process_steps` / `run_react_loop`)" | partially stale | Same as above. |
| `loop_detector.rs:1-7` — "Detects three types of loops" | stale / misleading | `LoopDetector` is never instantiated on the live path (F-RCT-02-P2-01, V03). |
| `phases/mod.rs:1-4` — "`run_core_loop` is the single, unified core loop" | current | V01 confirms both streaming and non-streaming entries spawn the same `run_core_loop`. |
| `pipeline.rs:3` — "Replaces the monolithic `execute_tool_feedback_raw`" | current | Confirmed: the live path uses `execute_tool_with_policy` → 13-stage pipeline; `execute_tool_feedback_raw` is dead (F-RCT-02-P2-02). |
| `finalize.rs` docstring (lines 1-4) — "Terminal-state phases" covering four helpers | current but asymmetric | All four helpers exist; `finalize_completed_run` omits `finalize_run` (F-RCT-02-P2-03). |
| `retry.rs:9` — "Shared by `think` and `create_llm_stream`" | current | Both call sites verified (`think.rs:289, 354`). |
| `F-RCT-01` handoff — "`set_system_prompt` doesn't refresh `CanonicalContext`" | out of scope for the loop | The loop reads `ContextManager` state, not `CanonicalContext`, at runtime; `CanonicalContext` is consumed only at construction (F-RCT-01). No loop-level defect. |

## Coverage And Uncertainty

Inspected in full: `react_loop.rs`, `stream_channel.rs:1-756` (loop body
+ streaming entry up to `run_core_loop`; the test module
`757-2161` was sampled for the terminal/error tests cited in V03/V04),
`direct.rs`, all six `phases/*` files, `processor.rs`, `retry.rs`,
`stream_macros.rs`, `loop_detector.rs`, `execution.rs:1-120`,
`pipeline.rs:1-70`, `mod.rs:2767-2819` (Agent trait impls),
`config.rs` budget/iteration fields, `echo-core/src/agent/mod.rs:30-133`
(`RunBudgetPolicy`/`BudgetDecision`/`AgentInvocationContext`),
`snapshot.rs:545-605` (`finalize_run` + checkpoint).

Not inspected (out of scope or deferred):

- `stream_channel.rs:757-2161` test module beyond the cited tests — the
  remaining tests cover cancellation, steer-during-llm, run-context
  isolation, and projection routing. They are adjacent to F-RCT-03
  (streaming entry) rather than this task's transition focus.
- `approval.rs` (456 lines) — HITL approval gating inside
  `execute_tool_with_policy`; the tools branch reaches it but the
  approval state machine itself is a tool-execution concern.
- `context.rs` (799 lines) — read selectively for
  `push_runtime_context_note`/`format_memory_context`; the full
  `ContextManager` is F-MEM-01 territory.
- `pipeline.rs:70-1722` — the 13 `PipelineStage` implementations.
  Confirmed scope (per-tool middleware) but internals deferred to a
  tool-execution task.
- The application-layer event projection (`chat_driver`,
  `task_runtime/executor`) — confirms the framework `Err`/`FinalAnswer`
  contract is consumed correctly, but belongs to application tasks.

Environmental constraints:

- All cargo commands ran against the existing incremental build cache
  (`target/`); no `cargo clean` was needed (disk pressure well below
  threshold). Final worktree state is clean (`git status` clean).
- The feature matrix was not re-run; only the default feature set was
  exercised (the loop is feature-independent except for the
  `#[cfg(human-loop)]` approval split in `phases/tools.rs:24-38` and the
  dead `process_steps` at `react_loop.rs:218-233`, both statically
  inspected).

Uncertain claims:

- Whether any third-party `echo-agent` consumer outside this monorepo
  calls `.loop_detector(...)` on the builder. The framework layering
  rule retains pub API unless framework-wide evidence shows obsolescence;
  the deletion recommendation in F-RCT-02-P2-01 is therefore conditional
  on a maintainer decision, but the pub-API surface is small (one
  builder method + one accessor) and the struct itself has zero
  production callers inside the framework.
- The exact severity of the trace-finalization gap (F-RCT-02-P2-03) for
  downstream consumers depends on whether they query `RunStore` by
  status. The defect is real (the `finalize_run` call is observably
  absent) but its user-visible impact scales with how heavily the
  trace store is queried.

## Handoff

Conclusions downstream tasks may rely on:

1. **Single ReAct loop confirmed.** `run_core_loop`
   (`stream_channel.rs:494`) is the only loop body. Any task that needs
   to reason about turn transitions, terminal events, or budget
   enforcement can treat it as authoritative. The streaming entry
   (`run_stream_channel`) and non-streaming entry (`run_react_loop`)
   differ only in how they collect events from the same loop.
2. **Terminal partition is exhaustive and non-overlapping.** The 10
   terminal arms enumerated in V02 are the complete set. Any new
   terminal path must be added to one of the four `finalize_*` helpers
   or documented as a new short-circuit arm.
3. **`max_iterations` (default 100) is the only hard loop bound.** Soft
   budgets (`RunBudgetPolicy`) are optional and default off. There is
   NO loop detection — `LoopDetector` is dead (F-RCT-02-P2-01).
4. **Trace finalization is asymmetric.** Tool-success
   (`finalize_completed_run`) and abandoned/error paths do not call
   `finalize_run`; text-success and the two failure helpers do. Any task
   that relies on `RunStore` status accuracy must account for this until
   F-RCT-02-P2-03/P3-01 are fixed.
5. **Error handling is graceful across all three classes** (LLM, tool,
   parse). No panic/hang path exists on the live loop.

Reports they must read:

- This report (F-RCT-02) for the loop-body invariants.
- `tasks/F-RCT-01.md` for the construction-path invariants (single tool
  registry, deterministic prompt) that this loop consumes.
- `tasks/F-CORE-01.md` for the `AgentEvent`/`EventEnvelope`/`ReactError`
  contract that the terminal arms emit.
- `validations/F-RCT-02/V01-01.md` through `V04-01.md` for per-claim
  evidence.

Conditions that make this report stale:

- Any change to `run_core_loop`'s iteration structure or the outcome
  enums (`PrepareOutcome`/`CompactOutcome`/`ThinkOutcome`/`IterOutcome`)
  invalidates V01.
- Any new terminal arm, or a change to which arms call `finalize_run`,
  invalidates V02 and the F-RCT-02-P2-03/P3-01 findings.
- Wiring `LoopDetector` into `run_tools`/the driver invalidates
  F-RCT-02-P2-01 and requires re-running V03.
- Deleting `process_steps`/`execute_tool_feedback_*` resolves
  F-RCT-02-P2-02 and requires re-running V01's duplicate search.

Follow-up task IDs (no fixes implemented in this review):

- **F-RCT-03** (streaming variant) — owns the `run_stream_channel`
  queueing, invocation-context snapshot construction, and the streaming-
  specific cancellation/steer tests sampled but not audited here.
- A **tool-execution-pipeline task** — owns `pipeline.rs`'s 13 stages
  and the `execute_tool_with_policy` middleware composition.
- A **framework cleanup task** — should decide F-RCT-02-P2-01 (delete
  vs wire `LoopDetector`) and execute F-RCT-02-P2-02 (delete
  `process_steps` + helpers). These two are independent of each other
  and of the trace-finalization fixes.
- A **trace-observability task** — should fix F-RCT-02-P2-03 and
  F-RCT-02-P3-01 (add `finalize_run` to `finalize_completed_run` and
  the Abandoned/Cancelled/Blocked arms) and add the corresponding
  trace-status regression tests.
