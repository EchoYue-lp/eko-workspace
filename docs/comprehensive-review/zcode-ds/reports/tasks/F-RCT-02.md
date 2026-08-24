# F-RCT-02: Non-streaming ReAct loop

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Does one non-streaming turn transition correctly through thinking, tool
batches, stopping, errors, limits, and final response?

## Scope

- `echo-agent/src/agent/react/run/react_loop.rs` (full read: `run_react_loop`
  :598-751, `call_llm_with_retry` :23-172, `process_steps` :177-502,
  `prepare_react_context` :508-591, `direct_answer` :755-823).
- `echo-agent/src/agent/react/run/stream_channel.rs` (full read of
  `run_stream_channel` :35-316, `run_core_loop` :494-757, `direct_answer_stream`
  :362-480, tests :759-2000).
- `echo-agent/src/agent/react/run/phases/` (full read: `mod.rs` state/outcomes,
  `prepare.rs`, `think.rs`, `tools.rs`, `verify.rs`, `finalize.rs`, `compact.rs`).
- `echo-agent/src/agent/react/run/stream_macros.rs`, `run/types.rs`,
  `run/direct.rs`, `run/context.rs` (prepare_stream_context :490-623,
  restore_thread_context :230-261, fire_lifecycle_hook :305-484).
- `echo-agent/src/agent/react/loop_detector.rs` (full read).
- `echo-agent/src/agent/snapshot.rs` (trace helpers :539-559, execute_tool_with_policy
  :1189-1279, tool_needs_approval :1152-1180, transcript projection).
- Callers/registration: `src/agent/react/mod.rs` (Agent trait execute/chat
  :2767-2819, run_stream_entry :1833-1870, add_intervention_callback :1642-1647),
  `src/agent/react/structured.rs`, `src/runner.rs`, `src/agent/react/run/pipeline.rs`
  (stage list only; internals owned by F-RCT-04), `src/agent/handle.rs`.
- EKO side: `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs`,
  `tasks/task_runtime/executor.rs` (entry points only), `infra.rs` (builder
  options :276-301), `profiles/types.rs` (max_iterations default).
- Executed tests: `cargo test -p echo_agent --lib --locked
  'react::run::stream_channel'`, `'react::run::phases'`, `'react::loop_detector'`.

## Out Of Scope

- Streaming event flow / backpressure / channel close ordering → F-RCT-03
  (the shared loop body is reviewed here only as it serves the non-streaming
  question; the buffer-full drop at stream_macros.rs:42-47 is left to F-RCT-03).
- Tool batch internals (pipeline stages, concurrency, timeouts, partial
  output) → F-RCT-04, F-EXT-01; only batch entry/exit points and terminal
  propagation were checked.
- Steer/interrupt/snapshot/resume → F-RCT-05 (checkpoint/resume tests observed,
  not re-audited).
- Provider/LLM contract fidelity (usage, malformed chunks) → F-LLM-01..03
  (F-LLM-01-P1-01/P2-02 cross-referenced only).
- TaskRuntime/Subagent execution loops in EKO → F-TSK/A-TSK/F-SUB tasks.

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, layering, one-authority rules,
  Subagent terminology), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (F-RCT-02 card), `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-RCT-01` (complete) and `F-LLM-01`
  (complete) — used for the option map, `max_iterations` default divergence
  (F-RCT-01-P2-03), usage authority (F-LLM-01-P2-02) and loop-relevant
  transport findings.
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md` (M2
  terminal convergence), `docs/PROJECT-ANALYSIS.md` (loop anchors) —
  classified in the Historical Claim Status section.

## Layering Decision

- Generic mechanism (framework): the unified core loop (`run_core_loop`),
  phase functions, `max_iterations`/budget semantics, terminal event
  ownership, trace finalization, loop detection, hook firing, and the
  `Agent::chat`/`execute` non-streaming API are all correctly placed in
  `echo-agent`; `StructuredAgent` and `AgentRunner` delegate. No repository
  movement is recommended by any finding.
- EKO product policy (application): EKO profile default `max_iterations = 0`
  (unlimited) and the decision to drive turns exclusively through the
  streaming entry (`chat_driver.rs`, `executor.rs`) are application policy;
  they make the framework loop-detection gap (P2-02) and the streaming
  hook double-fire (P2-03) directly visible in the product.
- Adapter boundary: none new.
- Duplicate-search terms (both repositories, see V01): `run_core_loop`,
  `run_react_loop`, `run_stream_channel`, `run_chat_direct`, `run_direct`,
  `process_steps`, `LoopDetector`, `loop_detector`, `MaxIterationsExceeded`,
  `max_iterations`, `StreamMode`, `IterOutcome`, `execute_tool_with_policy`,
  `execute_tool_feedback`, `execute_tool_feedback_raw`, `execute_tool(`,
  `AgentEvent::FinalAnswer`, `AgentEvent::Cancelled`, `finalize_run`,
  `finalize_trace_run`. Results: one unified loop authority; loop detection is
  defined-but-unwired (P2-02); `process_steps` + `execution.rs` tool-execution
  functions and `run/approval.rs` are dead parallel authorities (P3-01/P3-02);
  no loop copy exists in `echo-agent-cli`.

## Current Path

Verified data flow (anchors): `Agent::execute`/`Agent::chat`
(mod.rs:2767,2797) → `run_direct`/`run_chat_direct` (direct.rs:9,29) →
`run_react_loop` (react_loop.rs:598): takes the execution mutex, runs
`prepare_react_context` (guard → trace start → memory recall → workspace/memory
projections → user message push, :508-591), intent-route (DirectAnswer
shortcut → `direct_answer` :755-823 or fall-through), then spawns
`run_core_loop(context, text, None, "", StreamMode::Chat, recalled, tx)` in a
`tokio::spawn` (:711-727) and collects events until `FinalAnswer` /
`Cancelled` / `Error` / `Err` (:729-750). The loop body (stream_channel.rs:494-757):
`prepare_turn` (audit, UserPromptSubmit hook, TaskNode) → per-iteration
`drain_steer` → budget wind-down/final-only → `run_compact`
(pre-compact flush, checkpoint, projections, `ContextManager::prepare`) →
`run_think` (interventions, streaming LLM call via `create_llm_stream`, chunk
accumulation into content/tool-call map, `LlmUsage` event) → branch: tool calls
→ `run_tools` (serial/concurrent split, `execute_tool_with_policy` →
`ToolExecutionPipeline`, verifier on `final_answer` → `IterOutcome::Finish`);
text → `verify_final_text` → `emit_final_text`; neither → `finalize_no_response`;
loop exhaustion → `finalize_max_iterations`. The LLM call inside the loop is
always streaming (`chat_stream`) even on the non-streaming turn; the
non-streaming `call_llm_with_retry` (`chat()`) is used only by `direct_answer`
and the dead `process_steps`.

## Findings

### F-RCT-02-P1-01: Non-streaming turns silently return an empty-string success when the spawned core loop errors without forwarding the error

- Priority: P1
- Confidence: high (static chain fully verified; runtime trigger requires an
  intervention callback that cancels/blocks at final answer)
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/react_loop.rs:711-727` (spawned
  wrapper logs `"Core loop error (already sent via channel)"` and does NOT
  forward `Err(e)` to `tx`), `react_loop.rs:729-750` (when `rx.recv()` returns
  `None` without a terminal, `run_react_loop` returns `Ok(answer)` with
  `answer == ""`); `phases/finalize.rs:40-53`
  (`finalize_completed_run` returns `Err` on intervention cancel/block with no
  `tx` write); contrast `stream_channel.rs:306-311` (streaming wrapper forwards
  `let _ = tx.try_send(Err(e))`); intervention API is public
  (mod.rs:1642-1647, builder.rs:602/1060-1061).
- Reachability: definition → registration (framework public
  `Agent::chat`/`execute`, `StructuredAgent`, `AgentHandle`) → live caller
  (`run_react_loop` → spawned `run_core_loop`). Trigger: an
  `InterventionCallback` returning `cancel`/`block` at final answer (tools
  branch), or any future phase error propagated via `?` out of `run_core_loop`;
  additionally, `finalize_no_response`/`finalize_max_iterations` use
  non-blocking `try_send` (finalize.rs:226,267) which drops the error when the
  buffer is full.
- Expected invariant: a turn returns either a final answer or a typed error;
  streaming and non-streaming entry points surface the same errors.
- Observed behavior: on the tools-branch intervention cancel/block, the caller
  receives a closed channel and returns `Ok("")` — an empty string treated as
  a successful answer; the trace run is not marked failed.
- Impact: silent loss of the final answer and the error reason on the
  framework's non-streaming public API; consumers cannot distinguish
  "answered" from "error swallowed"; trace stays `Running`.
- Root cause: wrapper asymmetry — the streaming wrapper forwards loop errors,
  the non-streaming wrapper only logs; `finalize_completed_run` propagates
  intervention errors out-of-band instead of sending them onto the channel
  (unlike `think.rs:47-68`).
- Direction: forward errors in `run_react_loop`'s spawned task exactly like
  `stream_channel.rs:306-311`, and make `finalize_completed_run` send its
  intervention cancel/block errors onto the channel before returning (mirror
  the think-phase pattern); consider blocking sends for the two finalize
  error paths. No deletion needed; the wrong comment at react_loop.rs:724 is
  removed with the fix.
- Regression validation: mocked test — `MockLlmClient` final-answer tool +
  an intervention callback that cancels at final answer, driven through
  `agent.chat(...)`: must return `Err`, not `Ok("")`; a variant asserting the
  stream error arrives on the streaming entry for the same scenario.
- Validation reports: [V02](../validations/F-RCT-02/V02-01.md),
  [V03](../validations/F-RCT-02/V03-01.md), [V04](../validations/F-RCT-02/V04-01.md)

### F-RCT-02-P2-01: Tools-branch terminal (`final_answer` accepted) never finalizes the trace run — status stays `Running` with no final output

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `phases/finalize.rs:23-112` (`finalize_completed_run` — audit,
  checkpoint, transcript projection, TaskNode Success, `FinalAnswer` event,
  Stop hook, SessionEnd hook; no `snap.finalize_run`); contrast
  `finalize.rs:175` (text terminal), `:216` (no-response), `:261`
  (max-iterations), `stream_channel.rs:226,235` and `react_loop.rs:637,647`
  (direct-answer paths); `snapshot.rs:548-559` (`finalize_run` sets
  status/final_output/finished_at).
- Reachability: every turn that completes via a `final_answer` tool call (the
  standard tools-branch terminal of the shared loop, both streaming and
  non-streaming) on an agent with a run store attached.
- Expected invariant: every terminal path finalizes the trace run exactly once
  with a truthful status.
- Observed behavior: the tools-branch terminal leaves the run `Running`; no
  `final_output`, no `finished_at`; terminal events already emitted (FinalAnswer)
  contradict the persisted status.
- Impact: observability/run-history consumers (run list, per-run token
  accounting, A-OBS-01/X-STA-01 surfaces) see completed tool turns as
  perpetually running; "running vs hung" becomes indistinguishable.
- Root cause: `finalize_run` was wired into the text/no-response/max-iterations
  paths (and the direct-answer paths) but never added to
  `finalize_completed_run` when the terminal was refactored into phases.
- Direction: add `snap.finalize_run(RunStatus::Completed, Some(output), None)`
  in `finalize_completed_run` (mirror `emit_final_text` order, finalize.rs:175);
  keep exactly-once semantics (no other caller finalizes this run).
- Regression validation: unit test driving a tools-branch terminal through
  `finalize_completed_run` with an `InMemoryRunStore` (pattern exists at
  finalize.rs:285-295) asserting `RunStatus::Completed` and `final_output`.
- Validation reports: [V02](../validations/F-RCT-02/V02-01.md),
  [V03](../validations/F-RCT-02/V03-01.md)

### F-RCT-02-P2-02: `LoopDetector` is dead code — defined, configurable, publicly exported, never consulted by the loop; unlimited-iteration agents have no duplicate/no-progress protection

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `loop_detector.rs` (module doc "Loop detection for agent runs";
  `LoopDetector::new/record_tool_call/record_iteration/check/reset`,
  `LoopVerdict::{Continue,Warn,Break}`); `config.rs:176,264` (field),
  `config.rs:1025-1032` (builder setter + getter); `react/mod.rs:65`
  (`pub mod loop_detector`); repo-wide grep: zero callers of the type's methods
  in either repository; the only step limit in `run_core_loop` is the
  `max_iterations` for-loop (stream_channel.rs:521-528).
- Reachability: none at runtime — the stored `loop_detector_config` is never
  read by any loop path; `echo-agent-cli` profile default
  `max_iterations: 0` = unlimited (`profiles/types.rs:50-52`), passed through
  at `infra.rs:290` with no `run_budget` on the main agent, so the default EKO
  main agent runs with no iteration cap and no loop detection; the only
  runtime guards are per-tool/batch timeouts and the (optional) verifier retry
  cap.
- Expected invariant: either loop detection (exact-duplicate Break,
  failure-streak Warn, no-progress Warn) runs per iteration, or the API is
  removed (AGENTS.md cleanup); a config option must have an effect.
- Observed behavior: `LoopDetector` is never instantiated; its 5 unit tests
  pass only in isolation; the documented "Stopping to prevent runaway
  execution" verdict can never fire.
- Impact: a model stuck calling the same tool with the same arguments runs
  unbounded on default EKO config (each iteration costs a full LLM call);
  consumers setting `loop_detector(...)` get a silent no-op; the public module
  is a misleading surface.
- Root cause: scaffolded during planning, never wired into `run_core_loop`
  (a plain for-loop with `max_iterations` was implemented instead), never
  deleted.
- Direction: either (a) wire it — construct per turn in `run_core_loop`,
  `record_tool_call` in `run_tools`, `record_iteration` per iteration,
  `LoopVerdict::Break` → a loop-stop terminal (new or reuse
  `finalize_max_iterations` shape), `Warn` → runtime-context note; or (b)
  delete `loop_detector.rs`, `config.rs:1025-1032`, the field at config.rs:176
  and the `pub mod` at mod.rs:65. Prefer (a) for unlimited configs.
- Regression validation: mocked loop test — identical tool call 3x →
  turn terminates with the loop-stop terminal; a no-progress Warn variant
  asserting the injected context note; existing loop-detector unit tests
  repurposed as integration tests.
- Validation reports: [V01](../validations/F-RCT-02/V01-01.md),
  [V03](../validations/F-RCT-02/V03-01.md)

### F-RCT-02-P2-03: `UserPromptSubmit` lifecycle hook fires twice per streaming turn and once per non-streaming turn; block is enforced only on the second firing

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `run/context.rs:544-546` (`prepare_stream_context` fires
  `HookEvent::UserPromptSubmit`); `phases/prepare.rs:57-88` (`prepare_turn`
  fires the same hook inside the shared loop, with block → `FinalAnswer` and
  injected-context push); `react_loop.rs:508-591` (`prepare_react_context`
  never fires it); `fire_lifecycle_hook` also injects context and activates
  skills (context.rs:450-481).
- Reachability: every streaming turn (EKO main path: `chat_driver.rs:513`,
  `executor.rs:3119-3130`) with any `UserPromptSubmit` hook rule registered;
  non-streaming turns fire once (prepare_turn only).
- Expected invariant: one hook firing per user-prompt submission; identical
  semantics across streaming and non-streaming (AGENTS.md surface parity);
  the `block` decision is enforced on every firing.
- Observed behavior: streaming fires the hook twice (double hook-script
  execution, duplicate `Hook:UserPromptSubmit` context notes, second skill
  activation attempt is consumed by the first firing at context.rs:466-481);
  the first firing's `block` result is never checked.
- Impact: side-effectful hook scripts run twice per streaming turn; duplicated
  context injection wastes tokens; streaming and non-streaming turns diverge
  for the same hook configuration.
- Root cause: `prepare_stream_context` was "converged" with the non-streaming
  pre-flight (guard, intent) and added the hook firing without noticing that
  `prepare_turn` inside the shared loop already fires it.
- Direction: single authority — keep the firing (and block/injection) only in
  `prepare_turn` and delete the `fire_lifecycle_hook(HookEvent::UserPromptSubmit,
  ...)` call at context.rs:544 (preserve the `activate_skill` cache write by
  relocating it), or keep it only in the pre-flight and make `prepare_turn`
  skip it. No other deletion needed.
- Regression validation: streaming-turn test with a scripted hook registry
  counting `UserPromptSubmit` executions → exactly one; existing
  `prepare_turn_user_prompt_submit_block_short_circuits` stays green.
- Validation reports: [V02](../validations/F-RCT-02/V02-01.md),
  [V03](../validations/F-RCT-02/V03-01.md)

### F-RCT-02-P2-04: Stop-hook continuation emits `FinalAnswer` before the turn actually ends — consumers see a terminal answer while the loop (and a hidden extra LLM call on non-streaming) continues

- Priority: P2
- Confidence: medium (mechanism statically verified; trigger requires a Stop
  hook script returning `continue_reason`, parsed at
  `echo-execution/src/skills/hooks.rs:1429`)
- Layer: framework
- Evidence: `phases/finalize.rs:179` (text terminal sends `FinalAnswer`) →
  `:190-201` (Stop hook `continue_reason` → `ControlFlow::Continue`, one-shot
  flag set) → driver `continue`s (stream_channel.rs:694-695) and may emit a
  second `FinalAnswer` later; the trace was already finalized `Completed`
  (finalize.rs:175) before continuation; non-streaming caller breaks on the
  first `FinalAnswer` (react_loop.rs:733-736) while the spawned task runs one
  more full LLM request before abandoning on the first failed send;
  `finalize_completed_run` pushes the reason as a note but never continues
  (finalize.rs:95-105) — divergent Stop semantics between branches.
- Reachability: any text-branch final answer on an agent whose Stop hook
  registry returns `continue_reason` (hooks/types.rs:1087; scripted via
  `parse_hook_output`, hooks.rs:2126-2129).
- Expected invariant: the `FinalAnswer` event is genuinely terminal (frontend
  and drivers treat it as end-of-stream, see direct_answer_stream comment
  stream_channel.rs:477); continuation decisions happen before terminal
  emission; no work happens after the caller observes the terminal.
- Observed behavior: `FinalAnswer` followed by more events on streaming;
  on non-streaming the caller returns the first answer while a hidden LLM
  request runs in the background; a second `FinalAnswer` is possible in one
  turn (multiple-terminal violation).
- Impact: consumers see a "final" answer that is not final (work is dropped
  or mis-rendered); token spend after the terminal; per-turn trace status
  already `Completed` while the turn continues.
- Root cause: the one-shot continuation was layered onto a terminal path that
  emits the terminal event first and finalizes the trace first; terminal
  ownership was not re-architected when continuation was added.
- Direction: consult the Stop hook BEFORE emitting `FinalAnswer` (move the
  hook call ahead of the send and skip the send when continuing), and finalize
  the trace only on true termination; align `finalize_completed_run` to the
  same contract (or drop the text-branch continuation to match the tools
  branch).
- Regression validation: mocked test — Stop hook with `continue_reason` →
  exactly one `FinalAnswer` per turn and no loop continuation (or the chosen
  contract); a non-streaming test asserting no LLM call occurs after
  `agent.chat()` returns.
- Validation reports: [V02](../validations/F-RCT-02/V02-01.md),
  [V03](../validations/F-RCT-02/V03-01.md)

### F-RCT-02-P3-01: `process_steps` and the `execute_tool_feedback*`/`execute_tool` functions are a dead parallel tool-execution authority

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `react_loop.rs:177-502` (`process_steps`, `#[allow(dead_code)]`
  at :177, zero callers); `run/execution.rs:168,196,404` (`execute_tool_feedback_raw`,
  `execute_tool`, `execute_tool_feedback` — only internal/self callers and the
  dead `process_steps`); the live loop executes tools exclusively via
  `execute_tool_with_policy` (snapshot.rs:1189) → `ToolExecutionPipeline`
  (pipeline.rs:935-942).
- Reachability: none (pub(crate), no live caller; tests call `execute_tool`
  directly, tests.rs:1384).
- Expected invariant: one tool-execution authority (AGENTS.md 严禁平行实现
  同一语义).
- Observed behavior: a second, unmaintained execution path exists in dead
  code; `process_steps`' approval split and batch-timeout logic duplicate
  `run_tools` semantics.
- Impact: maintenance burden and drift risk; its docs (`react/mod.rs:8`,
  `run/mod.rs:8`) misdescribe the module contents.
- Root cause: legacy pre-pipeline implementation superseded by
  `execute_tool_with_policy` and never deleted.
- Direction: delete `process_steps` (react_loop.rs:174-502) and the
  `execute_tool_feedback_raw`/`execute_tool_feedback`/`execute_tool` functions
  in `run/execution.rs` (and their test call sites), after confirming
  `run/execution.rs`'s remaining helpers (`truncate_tool_output`,
  `check_tool_output_guard`) are not used by the live path (snapshot.rs has its
  own copies).
- Regression validation: `cargo check -p echo_agent` after removal; grep for
  `process_steps`/`execute_tool_feedback` returns only docs.
- Validation reports: [V01](../validations/F-RCT-02/V01-01.md)

### F-RCT-02-P3-02: `run/approval.rs` is a dead parallel approval authority

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `run/approval.rs` (`tool_needs_approval` :17,:53,
  `check_tool_approval` :65,:173) — zero callers; the live approval authority
  is `snapshot.rs:1152-1180` (`tool_needs_approval`) and the pipeline
  `PermissionStage` (`pipeline.rs:330-336` via `subsystems::approval::
  ApprovalSubsystem`, mod.rs:120,531).
- Reachability: none.
- Expected invariant: one approval authority.
- Observed behavior: dead module duplicating approval semantics; its
  non-streaming/streaming divergence comment (snapshot.rs:1150) documents a
  difference between dead and live code.
- Impact: maintenance burden; misleading "divergence" note.
- Root cause: legacy pre-pipeline approval superseded by the pipeline stage
  and never deleted.
- Direction: delete `run/approval.rs` and re-check the snapshot.rs:1150
  comment after removal.
- Regression validation: `cargo check -p echo_agent`; grep `run::approval`
  returns nothing.
- Validation reports: [V01](../validations/F-RCT-02/V01-01.md)

### F-RCT-02-P3-03: Non-streaming driver's `AgentEvent::Cancelled` branch is unreachable in the main loop

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `react_loop.rs:737-740` (handler); `AgentEvent::Cancelled` is
  produced only by `src/agent/subagent/executor.rs:2013` (subagent dispatch),
  never by the main loop or its phases.
- Reachability: none on the turn path.
- Expected invariant: driver branches correspond to producer contract.
- Observed behavior: a dead branch suggesting a cancellation signal that never
  arrives; non-streaming cancellation (cancel token) surfaces only via a
  dropped/timed-out stream, not this event.
- Impact: minor; misleading code.
- Root cause: legacy event contract retained in the driver after cancellation
  moved to token-based semantics.
- Direction: remove the branch (and document how non-streaming cancellation
  terminates: token → stream end → `Ok("")`/error, which P1-01's fix should
  make explicit).
- Regression validation: `cargo check -p echo_agent`.
- Validation reports: [V02](../validations/F-RCT-02/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search across both repositories (loop, terminal, loop-detection, tool/approval authorities) | yes | passed | [V01-01](../validations/F-RCT-02/V01-01.md) |
| V02 | Registration and runtime reachability trace (non-stream entry → spawned core loop → terminal/error producers; trace finalization sites) | yes | passed | [V02-01](../validations/F-RCT-02/V02-01.md) |
| V03 | Invariant/edge-case inspection vs tests (max_iterations=0, wind-down, exhaustion, terminal-once, verifier cap, loop-detection wiring, coverage gaps) | yes | passed | [V03-01](../validations/F-RCT-02/V03-01.md) |
| V04 | `cargo test -p echo_agent --lib --locked 'react::run::stream_channel'` + `'react::run::phases'` + `'react::loop_detector'` | yes | passed (exit 0 / 0 / 0; 23+22+5 passed) | [V04-01](../validations/F-RCT-02/V04-01.md) |
| V05 | Historical-document drift (MASTER-PLAN M2 terminal convergence; PROJECT-ANALYSIS loop anchors) | conditional | passed | [V05-01](../validations/F-RCT-02/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| PROJECT-ANALYSIS:145 "`call_llm_with_retry` 建 `CacheHints`; `phases/think.rs:308`" | current | react_loop.rs:40-51; think.rs:304-328; [V05](../validations/F-RCT-02/V05-01.md) |
| PROJECT-ANALYSIS:221 LLM request/response warn logs (react_loop.rs:71/92) | current | react_loop.rs:72-80, :93-99 |
| PROJECT-ANALYSIS:234 `RetryPolicy`/`retry_llm_call` | current | retry.rs; react_loop.rs:81-92,124-153 |
| MASTER-PLAN M2: "turn/TaskRun terminal 已分离…terminal 收敛" + turn terminal convergence model (:51) | regressed (partial) | tools-branch terminal never finalizes trace (`finalize.rs:23-112`, P2-01); `FinalAnswer` emitted before Stop-hook continuation (`finalize.rs:179-201`, P2-04) |
| MASTER-PLAN M3 "已完成结果不重放" / M5/M8/M10 | not revalidated | owned by F-RCT-05 / F-TSK / A-* tasks |
| Loop detection as a documented feature | absent in shared docs | `LoopDetector` undocumented AND unwired (P2-02) |

## Coverage And Uncertainty

- All conclusions are static except three test runs (V04); no dynamic run
  exercised: max-iteration exhaustion end-to-end, a Stop hook returning
  `continue_reason`, an intervention cancelling at final answer through the
  non-streaming entry, or the UserPromptSubmit double-fire (no such tests
  exist — see V03).
- `run/execution.rs`'s `truncate_tool_output`/`check_tool_output_guard` overlap
  with snapshot.rs was noted but not fully audited (F-EXT-01/F-RCT-04 scope);
  P3-01's deletion list requires a confirmatory grep at fix time.
- The `stream buffer full → drop event` behavior (stream_macros.rs:42-47) is
  recorded here only as an observation; backpressure/ordering consequences are
  F-RCT-03 scope.
- `pre_compaction_flush`'s non-streaming LLM call (usage discarded) was
  cross-referenced to F-LLM-01-P2-02 and not re-audited.
- The `human-loop` feature path (`tool_needs_approval`) was inspected under
  the default-features build; feature-gated behavior beyond approval routing
  is F-HITL-01/F-RCT-04 scope.
- F-RCT-01-P2-03 (divergent `max_iterations` defaults 10/100/0) is a
  dependency fact, not re-filed; P2-02's "unbounded" impact assumes the EKO
  default (0) and builder users who follow the 0-unlimited doc.

## Handoff

- Downstream tasks may rely on: one unified loop with `max_iterations=0`
  unlimited semantics (V03); the non-streaming entry chain and its
  error-forwarding gap (P1-01); terminal ownership inventory (V02);
  trace-finalization asymmetry (P2-01); loop-detection dead code (P2-02);
  UserPromptSubmit double-fire on streaming (P2-03); Stop-hook continuation
  after terminal emission (P2-04); green test state for stream_channel/phases/
  loop_detector at the reviewed commits (V04).
- `F-RCT-03` must treat stream buffer-full drops (stream_macros.rs:42-47) and
  the FinalAnswer-then-continue sequence (P2-04) as ordering/terminal facts to
  verify in the streaming conformance fixtures.
- `A-CHAT-01`'s one-terminal invariant must account for P2-04 (continuation
  after `FinalAnswer`) and P2-03 (double UserPromptSubmit on the EKO main
  path).
- `F-RCT-05` should confirm the tools-branch terminal's checkpoint/transcript
  state is resume-safe despite the missing trace finalization (P2-01).
- `X-BND-01` should record the LoopDetector wiring-vs-deletion decision
  (P2-02) and confirm `run/approval.rs` + `execution.rs` deletion has no
  external consumer (P3-01/P3-02).
- Reports to read: this report + [V01-01](../validations/F-RCT-02/V01-01.md)
  through [V05-01](../validations/F-RCT-02/V05-01.md); F-RCT-01 (builder
  defaults), F-LLM-01 (usage/transport).
- Stale triggers: any change to `run/react_loop.rs` (wrapper/driver),
  `run/stream_channel.rs` `run_core_loop`, `run/phases/*` terminal handling,
  `loop_detector.rs` wiring, `context.rs` `prepare_stream_context`/hook
  firing, or `AgentEvent` variants invalidates the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): F-RCT-03,
  A-CHAT-01, F-RCT-05, X-BND-01, Q-FLT-01 (fault scenarios for P1-01/P2-04),
  Q-TST-01 (coverage gaps from V03).
