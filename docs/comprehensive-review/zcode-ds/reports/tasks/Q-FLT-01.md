# Q-FLT-01: ReAct and tool fault-injection suite

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline 9b0e0fa)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline b3b2e81)
> Worktree state: clean in both repositories (`git status --porcelain` empty
> before and after every step); probe crate `/tmp/qflt01-probe` outside both
> repos; no source file in either repository was created or modified

## Question

Do Agent/tool invariants survive malformed LLM output, Unicode, huge output,
timeout, cancellation, disconnect, crash, and partial effects?

**Answer: partially.** Five of eight fault scenarios survive with intact
core-loop invariants (malformed tool args, empty content, Unicode, huge
output, per-tool timeout); three break invariants in ways that reach the
product: **truncated/disconnected streams are silently accepted as complete
final answers** (new P1-01), **LLM stream timeouts are never retried** (new
P1-02), and **cancel leaves no typed terminal plus a poisoned recovery
checkpoint that wipes the conversation on the next turn** (canonical
F-RCT-03-P1-02, F-RCT-05-P1-01, dynamically reconfirmed). The per-scenario
survival matrix is the task's core deliverable (section "Survival Matrix").

## Scope

One end-to-end fault-injection suite over the framework's live ReAct path
(`echo-agent`), with EKO reachability evidence only:

- `echo-agent/src/agent/react/run/processor.rs` (chunk processing,
  `parse_tool_args` repair, `build_tool_calls_from_map`).
- `echo-agent/src/agent/react/run/phases/{think,tools,finalize,verify}.rs`
  (streaming LLM call, batch arms, terminals).
- `echo-agent/src/agent/react/run/{stream_channel.rs,stream_macros.rs,retry.rs}`
  (core loop driver, send macros, retry/backoff, batch-timeout math).
- `echo-agent/src/agent/snapshot.rs` (`process_tool_output_for_call` spill/
  truncation :926-1065, `execute_tool_with_policy` :1189-1279).
- `echo-agent/echo-integration/src/providers/{client.rs,anthropic.rs}`
  (SSE transports, stream timeouts, inline Anthropic parser).
- `echo-agent/echo-core/src/tools/mod.rs` (ToolFailure taxonomy,
  `allows_automatic_retry`), `echo-core/src/agent/event_envelope.rs`
  (terminal normalization), `echo-core/src/agent/mod.rs` (cancel contract).
- `echo-agent/src/state/{mod.rs,file.rs}` (AgentCheckpoint, validator,
  atomic file store).
- `echo-agent-cli/echo-agent-app-core/src/{chat_driver.rs,infra.rs}`,
  `tool_exposure.rs` (reachability + product defaults: streaming entry,
  `max_tool_output_tokens` = `MAX_MODEL_VISIBLE_TOOL_RESULT_TOKENS` = 4_000,
  32 KiB artifact threshold).
- Executed probes: `/tmp/qflt01-probe` (8 scenario binaries against the real
  `run_core_loop` with scripted `LlmClient`s and real tools), plus reruns of
  `/tmp/frct05-probe` (F-RCT-05 cancel-checkpoint probe).

## Out Of Scope

- DAG/Task/Subagent fault injection → Q-FLT-02 (F-TSK-03, F-SUB-02 deps).
- Live external providers / real network fault timing → not executed (read-
  only; transport timeouts are env-var configurable and were injected at the
  exact yield points instead).
- GUI/TUI rendering of the fault outcomes → A-CHAT-01 / A-SRF-01..04
  (consumer contract cited only).
- Per-domain tool correctness (shell process cleanup on drop, git semantics)
  → F-EXT-02/F-EXT-03.
- `docs/comprehensive-review/codex/` and `zcode-glm/` (independence rule).

## Inputs

- Root `AGENTS.md` (UTF-8/panic hard rules, layering, one-authority,
  local-assistant threat model), shared `README.md`, `REPORTING.md`,
  `TASKS.md` (Q-FLT-01 card), `zcode-ds/README.md`, both report templates.
- Dependency task reports read (all zcode-ds, complete): `F-RCT-02`,
  `F-RCT-03`, `F-RCT-04`, `F-RCT-05`, `F-EXT-01`, `X-TOL-01`; canonical
  references read: `F-LLM-01`, `F-LLM-03`, `X-INV-01`, `Q-STA-01`.
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md`
  (terminal convergence :44-58, cancel-terminal :149, oversized-result
  claims :115/:275/:602, resume claims :67/:96/:147/:148) — classified in the
  Historical Claim Status section.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework) | The fault surface is framework-owned: chunk processing, tool-args repair, spill/truncation, retry, batch timeout/cancel, checkpoint/resume, terminal normalization, SSE transports. All new findings stay in `echo-agent`; no repository movement proposed. |
| EKO product policy (application) | `max_tool_output_tokens = 4_000` (tool_exposure.rs:5), the 32 KiB artifact threshold + 30-day retention (infra.rs:30-33,61-74), and streaming-only turn driving (chat_driver.rs:513,538) turn the framework gaps into product-visible behavior (partial answers rendered as final; cancelled turns rendered as errors). |
| Adapter boundary | `envelope_event_stream` (echo-core) is the single raw→product adapter: it truncates at the first terminal and fabricates `Error{"agent stream ended without a terminal event"}` for terminal-less ends (event_envelope.rs:174-191) — thin, lossless in the happy path, but it masks the raw-stream terminal-less ends instead of the loop guaranteeing one terminal. |
| Duplicate search | Terms searched across both repositories: `process_stream_chunk`, `parse_tool_args`, `build_tool_calls_from_map`, `truncate_tool_output`, `process_tool_output_for_call`, `retry_llm_call`, `compute_concurrent_tool_batch_timeout`, `save_runtime_checkpoint`, `resume_from_state_store`, `envelope_event_stream`, `AnthropicStreamEvent`, `parse_sse_chunk`, `finish_reason`, `cancel_aware_stream`. Results (V00-01): one live authority per semantic; `truncate_tool_output` in execution.rs:215 is dead-path only (F-RCT-02-P3-01); `finish_reason` is read only by the non-streaming react_loop.rs — the streaming loop never consults it. |
| Migration deletion | None proposed (findings are behavior gaps, not authority splits). |

## Current Path

Verified data flow (anchors; full inventory in V00-01): EKO drives every turn
through the streaming entry (chat_driver.rs:513, :538 envelope) →
`run_stream_entry` → `run_stream_channel` → `run_core_loop`
(stream_channel.rs:494-757): `prepare_turn` → `run_compact` →
`run_think` (streaming LLM call via `create_llm_stream` →
`retry_llm_call` wrap of `chat_stream` creation, chunk accumulation through
`process_stream_chunk` with `emit_content_tokens=false`, usage, end-of-stream
content burst) → tools branch `run_tools` (serial/concurrent split,
`execute_tool_with_policy` → 15-stage pipeline → `ToolManager` per-tool
timeout/retry → `process_tool_output_for_call` spill/truncation → ToolResult/
ToolError events + context push) → text branch `verify_final_text` →
`emit_final_text`; NoResponse / max-iterations / abandon exits. Transport:
`stream_post` (client.rs:182-354) yields mid-stream timeout errors (first-
chunk 30 s / idle 60 s / overall disabled, client.rs:240-301) as stream items;
EOF → clean stream end. Anthropic inline parser (anthropic.rs:478-623):
`if let Ok(event)` gate (:510), `#[serde(other)]` → `Other` (:1069-1070),
`message_delta` strict usage. Terminals: FinalAnswer (finalize.rs:87/:179),
Err items (finalize.rs:226/:267, think.rs:47/65, tools.rs:285-292,
stream_channel.rs:310); no `AgentEvent::Cancelled` producer on the main loop
(F-RCT-03-P1-02). Checkpoint/resume: cancel/error arms save mid-batch
(tools.rs:203/:258/:296/:309/:336/:414/:419), restore rejects unpaired
calls (state/mod.rs:186-231) and falls back to `reset_messages()`
(context.rs:245-248).

## Survival Matrix (the task's core conclusion)

| Scenario | Invariant checked | Verdict | Evidence |
|---|---|---|---|
| 1. Malformed LLM output — non-JSON tool args | loop survives; model can retry | **SURVIVES** | V01-01 (a): call dropped, fallback note pushed (tools.rs:101-109), model retried → FinalAnswer |
| 1. Empty content | typed error, trace truthful | **SURVIVES** | V01-01 (b): `Err(NoResponse)` terminal (finalize.rs:226), trace Failed |
| 1. Unknown/malformed events | loss observable | **FAILS** (canonical + new P2-01) | V01-01 (c): silent drops (F-LLM-01-P1-01, F-LLM-03-P1-02/P2-01); Anthropic `error` events silently ignored (new Q-FLT-01-P2-01) |
| 1. Truncated stream | partial output never "final" | **FAILS** (new P1-01) | V01-01 (d), V06-01 (b): partial content → FinalAnswer, trace Completed |
| 2. Unicode (CJK/emoji) | content/args/truncation byte-safe | **SURVIVES** (core loop) | V02-01: content + args + results preserved; 2M-CJK truncation UTF-8-safe; peripheral panics canonical (X-INV-01-P1-01/02, Q-STA-01-P1-01/P2-01) |
| 3. Huge output | model output bounded; artifact complete | **SURVIVES** | V03-01: spill 2 MiB → 2004-byte preview + artifact 6,000,011 bytes, sha256 matches note, payload identical; truncation fallback UTF-8-safe |
| 4. LLM timeout | retry covers configured timeouts | **FAILS** (new P1-02) | V04-01: mid-stream timeout not retried (calls=1); call-start retried (calls=4) but no terminal + trace Running |
| 4. Per-tool timeout | classified, batch continues | **SURVIVES** | V04-01 (b): ToolError Timeout + side_effect Possible; write tools not auto-retried |
| 4. Batch timeout | typed terminal | **FAILS** (canonical F-RCT-04-P1-02) | static + V04-01 (c): `try_send_or` droppable, no per-call ToolError |
| 5. Cancel mid-stream | `Cancelled` terminal per trait contract | **FAILS** (canonical F-RCT-03-P1-02) | V05-01 (a): no Cancelled event; error item / silent end |
| 5. Cancel mid-batch | recovery state restorable | **FAILS** (canonical F-RCT-05-P1-01) | V05-01 (b) + frct05-probe rerun: checkpoint with unpaired call → restore Err → context wipe |
| 6. Disconnect/EOF | partial content not final; terminal truthful | **FAILS** (new P1-01 + canonical) | V06-01: EOF partial → FinalAnswer + trace Completed; mid-stream error → terminal-less end, trace Running; envelope fabricates Error |
| 7. Crash | atomic save; corrupt file preserved; resume preserves context | **PARTIAL** (canonical) | V07-01: atomic write round-trip OK; corrupt file overwritten silently (F-RCT-05-P3-01); interrupted-batch checkpoint rejects + wipes (F-RCT-05-P1-01); pre-batch checkpoint can replay side effects (F-RCT-05-P2-01) |
| 8. Partial side effects | interrupted tool always reported | **PARTIAL** (canonical) | V08-01: per-tool timeout classified and reported to model; batch-kill produces no failure record (F-RCT-04-P2-02) and EKO records bare "cancelled" (X-TOL-01-P2-01) |

## Findings

### Q-FLT-01-P1-01: A truncated or cleanly-disconnected provider stream is silently accepted as a complete final answer — partial output is finalized `Completed` and there is no truncation signal anywhere in the loop

- Priority: P1
- Confidence: high (dynamically reproduced through the real loop, trace store
  attached)
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/phases/think.rs:110-125` (the
  streaming loop consumes chunks and never checks `finish_reason`; grep:
  the only production readers of `finish_reason` are the non-streaming
  `react_loop.rs:101,161`); `processor.rs:16-88` (`process_stream_chunk`
  never reads `finish_reason`); `phases/finalize.rs:128-208`
  (`emit_final_text` finalizes the trace `Completed` and emits `FinalAnswer`
  on whatever content was accumulated); transport EOF paths that end the
  stream cleanly without any terminal signal: `client.rs:305-306`
  (`bytes=None → break`) and anthropic.rs:480-486 (loop end, and `:507-509`
  `[DONE]` is never sent by the Messages API); canonical adjacent fact:
  F-LLM-01-P3-01 (finish_reason dropped at the core-loop level).
- Reachability: every streaming turn whose provider connection closes cleanly
  mid-answer (network flap, proxy timeout, gateway cutoff) or whose stream is
  truncated without an error; the loop has no way to distinguish "stream
  ended" from "stream completed".
- Expected invariant: a final answer is complete and truthful; a truncated or
  partial response is detected (finish_reason/length expectation) or surfaced
  as an error; the trace status agrees with the emitted terminal.
- Observed behavior (probe, real loop + InMemoryRunStore): scripted stream
  `["The commit hash is ", "a1b2c3d4"]` then EOF without `finish_reason` →
  events `[Token("The commit hash is a1b2c3d4"), FinalAnswer("The commit hash
  is a1b2c3d4")]`, trace `status=Completed final_output_len=27`. The model's
  answer was cut off mid-generation and the partial text is presented and
  persisted as the complete answer.
- Impact: wrong/partial answers delivered as final with full product
  acceptance (history, trace, audit log all record the partial text as the
  answer); the user has no signal that the answer is truncated; silent
  correctness failure exactly in the scenario class this task injects.
- Root cause: the loop terminates on stream end without consulting the
  provider's `finish_reason`; the June 2026 streaming refactor
  (F-RCT-03-P2-01 context) buffered content and dropped the finish
  semantics, and no truncation-length or completion expectation exists.
- Direction: consult `finish_reason`/`stop_reason` at stream end — a
  missing/non-terminal finish with content should either (a) flag the answer
  as truncated (metadata/event) or (b) surface an error and not finalize
  `Completed`; align the Anthropic adapter (which loses `stop_reason` on the
  real wire, F-LLM-03-P1-02) with the same contract; add a loop-level fixture
  for EOF-with-partial-content (must fail before the fix).
- Regression validation: mocked stream ending without `finish_reason` →
  the turn must not finalize `Completed` with a partial `final_output`, and
  the consumer must observe a truncation signal; a `finish_reason=stop`
  fixture asserting today's happy path stays green.
- Validation reports: [V01-01](../validations/Q-FLT-01/V01-01.md),
  [V06-01](../validations/Q-FLT-01/V06-01.md)

### Q-FLT-01-P1-02: Configured LLM stream timeouts (first-chunk / idle / overall) fire inside the stream and are never retried — the documented retry policy covers only call-start transport errors, so a transient mid-stream stall fails the whole turn

- Priority: P1
- Confidence: high (dynamically reproduced; call-start vs mid-stream contrast)
- Layer: framework
- Evidence: `echo-integration/src/providers/client.rs:240-301` — all three
  timeouts are applied inside the `try_stream!` body around
  `byte_stream.next()`, yielding `Err(NetworkError "...timeout...")` as a
  stream item; `src/agent/react/run/retry.rs:13-68` (`retry_llm_call` wraps
  only `call_fn().await` — i.e. `chat_stream()` creation, think.rs:285-349);
  `phases/think.rs:110-111` (`try_send_or!(tx, cr, ThinkOutcome::Abandoned)`
  forwards the first Err item and abandons the turn); the loop-level retry
  predicate `is_retryable_llm_error` (react/mod.rs:76-85) is never reached
  for item errors.
- Reachability: every LLM streaming call that stalls (first chunk > 30 s,
  idle > 60 s; env-configurable) — routine under network flakiness; EKO uses
  the shared transport for OpenAI-family providers and the inline parser for
  Anthropic (Anthropic has no stream timeouts at all).
- Expected invariant: the retry policy documented by `llm_max_retries` /
  `retry_llm_call` applies to the configured timeout classes, or the timeout
  classes are documented as non-retried; a transient stall does not
  deterministically fail the turn.
- Observed behavior (probe): call-start failure (chat_stream returns
  `Err(NetworkError "first-chunk timeout")`) → 4 attempts, ~4.9 s backoff,
  then error forwarded; mid-stream timeout (same error text as a stream item
  after a chunk) → exactly 1 call, error forwarded immediately. Both end
  with no terminal event and the trace left `Running` (canonical
  F-RCT-03-P2-04 / F-RCT-04-P1-02 class).
- Impact: the most common timeout shapes (idle/first-chunk) get zero retry —
  a transient stall discards the partial turn and shows the user an error;
  the retry knob gives a false sense of resilience; behavior is timing-
  dependent (same failure retried or not depending on whether the transport
  error happens before or after stream creation).
- Root cause: timeout handling was implemented inside the stream producer
  while retry was implemented around the stream constructor; the two layers
  never met.
- Direction: hoist the timeout bounds to the constructor layer (wrap
  `chat_stream` in `tokio::time::timeout` at `retry_llm_call` call sites, or
  make `stream_post` return timeouts as constructor errors), or explicitly
  document and accept non-retried stream timeouts; add a loop-level fixture
  for a mid-stream timeout asserting the retry behavior chosen.
- Regression validation: scripted client yielding an idle-timeout item →
  either retried (calls > 1) or the documentation states otherwise; a
  call-start timeout fixture asserting current retry behavior stays green.
- Validation reports: [V04-01](../validations/Q-FLT-01/V04-01.md)

### Q-FLT-01-P2-01: Anthropic mid-stream provider `error` events are silently ignored — overload/rate-limit errors parse into the `Other` variant and are indistinguishable from a normal stream end

- Priority: P2
- Confidence: high (mirror fixtures; static chain; the enum shape is
  code-certain)
- Layer: adapter
- Evidence: `echo-integration/src/providers/anthropic.rs:1069-1070`
  (`#[serde(other)] Other` on `AnthropicStreamEvent` — an
  `{"type":"error",...}` payload deserializes successfully as `Other`);
  `anthropic.rs:510` (`if let Ok(event) = ...` — no else, no log);
  `anthropic.rs:618` (`_ => {}` — `Other` silently ignored); the
  `event: error` SSE line is skipped at `:502`; contrast the HTTP-level error
  handling (`:450-457` ApiError) which does surface.
- Reachability: any Anthropic/Anthropic-gateway stream that sends an
  `error`-type event mid-stream (overloaded_error, rate limit, context
  length) — the documented Anthropic error channel for post-start failures.
- Expected invariant: a provider-reported error terminates the stream with an
  observable error (the F-LLM-01-P1-01 invariant family); a provider failure
  is never indistinguishable from a normal end.
- Observed behavior (mirror of the exact parser logic): the error event
  parses as `Other` and is ignored; the stream then ends normally → the loop
  sees no content/tool calls → `NoResponse` error or a partial answer
  (compounding Q-FLT-01-P1-01), with no error text anywhere.
- Impact: silent failure of the turn on a real provider error class; the
  user sees "no response" instead of the provider's reason; extends canonical
  F-LLM-03-P2-01 (which covers *unparseable* events) to the *parseable-but-
  ignored* error events.
- Root cause: the wire enum was modeled with `#[serde(other)]` before
  mid-stream error events were considered, and the catch-all arm is silent.
- Direction: add an `Error { error: ... }` variant (or at least log/count
  `Other` events, surfacing a drop counter at stream end); map it to a
  yielded `Err(LlmError::ApiError{..})` so the loop terminates with a typed
  error; align with the F-LLM-03-P2-01 fix.
- Regression validation: parser fixture feeding
  `{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}`
  → the stream yields an error item (or a counted drop), never a silent end.
- Validation reports: [V01-01](../validations/Q-FLT-01/V01-01.md)

No further new findings. The remaining scenario failures are canonical
defects reconfirmed dynamically at these commits (see Survival Matrix):
F-RCT-03-P1-02 (cancel terminal), F-RCT-05-P1-01 (checkpoint poison → wipe),
F-RCT-05-P2-01 (side-effect replay), F-RCT-05-P3-01 (corrupt-file overwrite),
F-RCT-04-P1-02/P2-02 (batch timeout/cancel terminal + partial-side-effect
record), X-TOL-01-P2-01 (EKO kill-path classification collapse),
F-LLM-01-P1-01 / F-LLM-03-P1-02/P2-01 (silent chunk/event drops).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---:|---|
| V00 | Definition/duplicate search + reachability + fixture inventory for the fault surface | yes | passed | [V00-01](../validations/Q-FLT-01/V00-01.md) |
| V01 | Scenario 1 malformed LLM output — `./target/debug/scn1_malformed` (real loop + parser mirrors) | yes | failed (2 new findings P1-01, P2-01) | [V01-01](../validations/Q-FLT-01/V01-01.md) |
| V02 | Scenario 2 Unicode — `./target/debug/scn2_unicode` (real loop, CJK/emoji args + content, 2M-CJK truncation) | yes | passed | [V02-01](../validations/Q-FLT-01/V02-01.md) |
| V03 | Scenario 3 huge output — `./target/debug/scn3_huge` (spill + checksum + truncation fallback) | yes | passed | [V03-01](../validations/Q-FLT-01/V03-01.md) |
| V04 | Scenario 4 timeouts — `./target/debug/scn4_timeout` (call-start vs mid-stream LLM timeout, per-tool timeout, trace store) | yes | failed (new P1-02) | [V04-01](../validations/Q-FLT-01/V04-01.md) |
| V05 | Scenario 5 cancellation — `./target/debug/scn5_cancel` + rerun `/tmp/frct05-probe` | yes | failed (canonical F-RCT-03-P1-02, F-RCT-05-P1-01 reconfirmed) | [V05-01](../validations/Q-FLT-01/V05-01.md) |
| V06 | Scenario 6 disconnect/EOF — `./target/debug/scn6_disconnect` (mid-stream error, clean EOF, envelope) | yes | failed (new P1-01 + canonical) | [V06-01](../validations/Q-FLT-01/V06-01.md) |
| V07 | Scenario 7 crash/recovery — `./target/debug/scn7_crash` (atomic write, corrupt file) + frct05 rerun | yes | failed (canonical F-RCT-05-P1-01/P3-01 reconfirmed) | [V07-01](../validations/Q-FLT-01/V07-01.md) |
| V08 | Scenario 8 partial side effects — `./target/debug/scn8_partial` (per-tool timeout vs batch kill) | yes | failed (canonical F-RCT-04-P2-02, X-TOL-01-P2-01 reconfirmed) | [V08-01](../validations/Q-FLT-01/V08-01.md) |

All required validations executed; every reported command has a known exit
code (all probes exit 0; failed rows mean the system under test violated the
claimed invariant, recorded as findings or canonical references); no
validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Agent trait cancel contract: "cancel → stream yields `AgentEvent::Cancelled` and terminates" (echo-core/src/agent/mod.rs:552-553, 617-618) | regressed | main loop never emits Cancelled; cancel → Err item / silent end (V05-01; canonical F-RCT-03-P1-02) |
| MASTER-PLAN:149 — "取消…最终只产生一个 cancelled terminal" | regressed | cancel mid-batch → ToolBatchEnd + channel close, no terminal (V05-01; F-RCT-04-P1-02) |
| MASTER-PLAN:67/96/147 — resume 跳过已完成副作用 (call_id skip) | regressed (not implemented) | completed_tool_call_ids only logged (react/mod.rs:1731-1741); replay gap (V07-01; F-RCT-05-P2-01) |
| MASTER-PLAN:148 — 恢复时先校验 tool_call/tool_result 配对 | current (validation) / regressed (failure handling) | validator state/mod.rs:186-231; rejection → full context wipe (V05-01/V07-01; F-RCT-05-P1-01) |
| MASTER-PLAN:115/:275/:602 — 超长结果 = 完整 artifact + 有界预览,路径/大小/SHA-256 共享 | current | spill verified end-to-end: preview bounded, artifact complete, sha256 matches (V03-01) |
| MASTER-PLAN M9 — provider usage 是记账权威 | regressed on Anthropic streaming | message_delta usage drop reconfirmed in parser mirror (V01-01 (c); F-LLM-03-P1-02) |

## Coverage And Uncertainty

- All conclusions are either static traces or dynamic probes at the reviewed
  commits; no live provider, no real network fault, no GUI process was run
  (read-only review). Transport mirrors replicate the exact parse logic of
  `client.rs` / `anthropic.rs` (the types are private to `echo_integration`);
  the loop-level probes run the REAL `run_core_loop` with scripted
  `LlmClient`s.
- The mid-stream timeout fixture injected the error at the exact yield point
  the transport uses (client.rs:284-301); real-world timing (idle 60 s) was
  not waited out.
- F-RCT-05-P1-01's in-process next-turn wipe (context.rs:245-248 →
  reset_messages) is static-plus-canonical; the dynamic probe proved the
  checkpoint rejection, not the wipe itself (same boundary as F-RCT-05).
- The Chinese-language fallback message at tools.rs:102
  ("(流式工具调用参数解析失败,已跳过;请重新发起工具调用)") is a hardcoded
  non-English string in a generic framework path — noted, not filed (product
  language is Chinese today; low impact).
- Scenario 5(a) used `MockLlmClient::with_delay` cancel semantics (Err item);
  the real transport's cancel is a silent stream end (F-LLM-01-P3-01) — both
  converge on "no Cancelled event".
- The EKO sink behavior for the fabricated envelope error (cancel rendered as
  error) is A-CHAT-01 scope, not re-audited here.

## Handoff

- Downstream tasks may rely on: the survival matrix (above) as the per-
  scenario verdict inventory; three new framework findings — truncated/EOF
  streams accepted as complete answers (P1-01), LLM stream timeouts never
  retried (P1-02), Anthropic error events silently ignored (P2-01) — each
  with dynamic loop-level evidence; canonical defects reconfirmed at these
  commits (F-RCT-03-P1-02, F-RCT-04-P1-02/P2-02, F-RCT-05-P1-01/P2-01/P3-01,
  X-TOL-01-P2-01, F-LLM-01-P1-01, F-LLM-03-P1-02/P2-01); the probe crate
  `/tmp/qflt01-probe` as a ready-made fixture family (scripted LlmClient,
  tools, transcript collector) for converting each scenario into permanent
  regression fixtures.
- Reports to read: this report + V00-01..V08-01; dependency reports F-RCT-02
  ..05, F-EXT-01, X-TOL-01; canonical references F-LLM-01, F-LLM-03,
  X-INV-01, Q-STA-01.
- Fix ownership: P1-01/P1-02/P2-01 are framework (`echo-agent`); the EKO-side
  kill-path classification (X-TOL-01-P2-01) stays application; the envelope
  masking decision (flag-vs-guarantee) belongs with F-RCT-03's terminal work.
- Stale triggers: any change to `processor.rs`, `phases/think.rs`
  (finish_reason handling), `phases/finalize.rs` terminals,
  `run/retry.rs`, `client.rs` stream timeouts, `anthropic.rs` event enum/
  parser, `tools.rs` kill arms, `snapshot.rs` spill, or `state/{mod,file}.rs`
  invalidates the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): S-RDM-01
  (roadmap items for P1-01, P1-02, P2-01 with the canonical merge notes
  F-RCT-03-P2-04 / F-RCT-04-P1-02 / F-LLM-03-P2-01), Q-TST-01 (convert the
  probe family into permanent fixtures), Q-E2E-01 (partial-answer and cancel
  rendering across surfaces).
