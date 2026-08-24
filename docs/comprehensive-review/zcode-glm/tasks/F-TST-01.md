# F-TST-01: Framework test and mock utilities

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean

## Question

Do public/internal mocks and testing helpers faithfully model real streaming,
tool, usage, error, cancellation, and ordering contracts?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/testing/mod.rs` (105 lines) — module root, doc table, public
  re-exports. Gate is `#[cfg(any(test, feature = "testing"))] pub mod testing`
  at `src/lib.rs:59-60`; the re-export under `prelude` at `src/lib.rs:274-275`
  carries the same gate.
- `echo-agent/src/testing/mock_llm.rs` (466 lines) — `MockLlmClient`,
  `MockLlmResponse` (private enum), `PopResult`, the `LlmClient` impl
  (`chat`/`chat_stream`/`model_name`), the scripted response queue,
  `with_delay` + cancel-aware `select!`, `with_response_usage`,
  `then_tool_call` / `then_tool_calls` / `then_tool_call_with_usage` /
  `then_reasoning_tool_call` (`#[cfg(test)]`), `with_error` /
  `with_network_error` / `with_rate_limit_error`, call recording
  (`call_count` / `last_messages` / `all_calls` / `all_tool_choices` /
  `all_tool_counts`).
- `echo-agent/src/testing/mock_agent.rs` (424 lines) — `MockAgent` +
  `FailingMockAgent`, the `Agent` trait impl (`execute` / `execute_stream` /
  `chat` / `chat_stream` /
  `execute_stream_message_with_cancel` /
  `execute_stream_with_invocation_context` /
  `execute_stream_message_with_invocation_context` / `reset` /
  `set_working_dir` / `clear_working_dir`), `with_delay_ms`, call/message/
  invocation-context/working-dir recording.
- `echo-agent/src/testing/mock_tool.rs` (173 lines) — `MockTool`,
  `MockToolResponse` (private enum), `Tool` impl (`name` / `description` /
  `parameters` / `execute` only), `with_response` / `with_responses` /
  `with_failure`, `with_description` / `with_parameters`, call recording
  (`call_count` / `last_args` / `all_calls`).
- `echo-agent/src/testing/mock_embedder.rs` (62 lines) — `MockEmbedder`,
  deterministic hash-based `Embedder` impl.
- `echo-agent/Cargo.toml:65-97` — `[features]`: `default = []`,
  `full = [...]` (does NOT include `testing`), `testing = []` (empty,
  unlock-only gate). Lines 253/257/261/265/269/329/345 — seven examples
  declare `required-features = ["testing"]`.
- Real-provider streaming contract cross-references (from F-LLM-01 /
  F-LLM-02, re-verified at this commit):
  - `echo-integration/src/providers/client.rs:182-354` — `stream_post`
    SSE loop, `yield chunk` per SSE event, `is_cancelled()` polled between
    chunks (`client.rs:252-254` per F-LLM-01).
  - `echo-integration/src/providers/openai.rs:322-377` — `chat_stream`
    delegate to `stream_post`.
  - `echo-integration/src/providers/anthropic.rs:421-628` — standalone
    stream loop, mid-loop cancel poll (`:482-484` per F-LLM-01).
- Streaming consumer side (from F-RCT-03, re-verified):
  - `echo-agent/src/agent/react/run/phases/think.rs:104-125` — the
    `while let Some(cr) = llm_stream.next().await` accumulation loop over
    `content_buffer` / `reasoning_buffer` / `tool_call_map`.
  - `echo-agent/src/agent/react/run/processor.rs:16-88` —
    `process_stream_chunk` (per-chunk → events), `in_reasoning` state,
    tool-call delta accumulation via `dc.index`.
  - `echo-agent/src/agent/react/run/processor.rs:104-183` —
    `parse_tool_args` (DeepSeek trailing-junk repair) +
    `build_tool_calls_from_map` (drop-unparseable).
- Production test consumers (sampled): `src/agent/react/run/stream_channel.rs:759-2161`
  (streaming test module), `src/agent/react/tests.rs`, `src/agent/subagent/executor.rs:1891-`,
  `src/agent/subagent/registry.rs:478-`, `src/agent/react/builder.rs:1129-`,
  `src/agent/react/run/phases/compact.rs:381-`, `src/intent/classifier.rs:468-`,
  `src/agent/default_factory.rs:57-`.
- Downstream consumer (feature leak probe):
  `echo-agent-cli/echo-agent-app-core/Cargo.toml:9-15, 58-59` — production
  `[dependencies]` do NOT enable `testing`; `[dev-dependencies]` enables
  `testing` (the correct Cargo pattern).

## Out Of Scope

Deferred to named task IDs:

- Adapter fidelity (how OpenAI/Anthropic map their wire format into the
  neutral `ChatChunk`) — owned by **F-LLM-02** / **F-LLM-03**. This task
  only references the neutral contract to judge whether the mock models
  the *shape* of provider behavior.
- Concrete streaming-backpressure and terminal-event defects in the
  production runtime (drop-on-Full, droppable error terminals, missing
  `Cancelled`) — owned by **F-RCT-03**. This task only asks whether the
  mock can *exercise* those defects (it cannot).
- `Tool` / `ToolResult` / `ToolFailure` contract design — owned by
  **F-EXT-01**. This task references the contract only to judge mock
  coverage of it.
- Application-layer test quality (`echo-agent-cli` test suite) — the
  application testing task. This task covers the framework-provided mocks
  that the application tests are built on.
- The `MockEmbedder` — in scope as inventory, but it is a trivial
  deterministic vector generator with no streaming/tool/error surface;
  it is assessed as faithful for its narrow purpose and not analysed
  further.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — framework-vs-application layering gate (the mock
    surface is a generic framework capability any consumer needs, so it
    lives correctly in `echo-agent`), dead-code cleanup rule, no-panic /
    UTF-8 safety rules, the "echo-agent is a reusable framework, not the
    CLI's private library" rule (relevant to V03: the `testing` feature is
    a framework API surface, not CLI-private).
  - `docs/comprehensive-review/REPORTING.md` (in full).
  - `docs/comprehensive-review/templates/task-report.md`,
    `templates/validation-report.md` (in full).
- Dependency task reports read:
  - [F-LLM-01](./F-LLM-01.md) — established the neutral `LlmClient`
    contract (`ChatRequest` / `ChatResponse` / `ChatChunk` / `Usage` /
    `LlmError`), the `'static`-lifetime stream, and that real providers
    poll `cancel_token` between SSE chunks. This report uses those
    conclusions as the contract the mock must model.
  - [F-RCT-03](./F-RCT-03.md) — established that the production stream
    consumer (`run_think` + `process_stream_chunk`) accumulates across
    many chunks, that the `parse_tool_args` DeepSeek repair path exists,
    and that the chunk loop has no inter-chunk backpressure. This report
    checks whether the mock can reproduce those conditions.
  - [F-EXT-01](./F-EXT-01.md) — established the `Tool` / `ToolResult` /
    `ToolFailure` taxonomy, the `execute` vs `execute_with_context`
    default-delegation pattern, cursor pagination, and the artifact
    spill. This report checks whether `MockTool` models that surface.
- Historical documents treated as hypotheses: none. The `src/testing/mod.rs`
  module docstring (design-principle table) is treated as **current** and
  re-verified (V01).

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. A test-double surface for `LlmClient`, `Agent`, `Tool`, `Embedder` is a generic capability any `echo-agent` consumer (the CLI, a third-party headless user, a downstream reuse) needs in order to write tests against the framework without real network/LLM access. The mocks live correctly in the framework crate, gated behind `cfg(any(test, feature = "testing"))`. |
| EKO product policy | None at this layer. No EKO-specific concept leaks into the mocks. `MockAgent` records `set_working_dir` / `clear_working_dir` (the worktree-chroot seam), but that is the *generic* `Agent` trait surface (F-EXT-01 / F-CORE-01), not an EKO policy. |
| Adapter boundary | The mocks implement the same framework traits (`LlmClient`, `Agent`, `Tool`, `Embedder`) as production. No adapter layer is introduced. The mock is a sibling implementation, not a wrapper. |
| Duplicate search | Searched names: `MockLlmClient`, `MockAgent`, `FailingMockAgent`, `MockTool`, `MockEmbedder`, `testing` module, `with_response`, `with_error`, `then_tool_call`, `with_delay`, `call_count`, `last_messages`, `all_calls`. Searched behaviours: scripted response queues, single-chunk vs multi-chunk streaming, cancel-aware delay, tool-result success/error only. Result: exactly one definition of each mock type, all in `echo-agent/src/testing/`. The CLI does not define its own framework-level mocks; it reuses these via `echo_agent::testing::` (see V03 for the dev-dependency wiring). No parallel/sibling mock surface. |
| Migration deletion | No deletion proposed. The mocks are live and exercised by ~40 in-crate test sites and ~20 CLI test sites. |

## Current Path

Verified mock surface and consumption at commit `9b0e0fa`. The mocks are
trait-faithful at the *type* level — they implement exactly the production
traits — but model only a *subset* of the production *behaviours*.

```text
Production contract          Mock coverage           Source anchor
───────────────────────────── ─────────────────────── ─────────────────────────────
LlmClient::chat              yes (single-shot)       mock_llm.rs:331-378
LlmClient::chat_stream       PARTIAL (single chunk)  mock_llm.rs:380-461
  multi-chunk SSE stream       NO                     stream::once (one ChatChunk)
  fragmented tool-call deltas  NO (one complete delta) mock_llm.rs:426-442
  reasoning_content streaming  NO (hardcoded None)    mock_llm.rs:417
  mid-stream error             NO (pre-stream only)   mock_llm.rs:410-422
  cancel mid-stream            NO (pre-stream only)   mock_llm.rs:400-408
  cancel pre-stream            yes (with_delay)       mock_llm.rs:349-357, 400-408
  usage                        yes                     mock_llm.rs:111-121, 207-230
  typed LlmError               yes (3 of 5 variants)  mock_llm.rs:245-257
Agent::execute / chat         yes                     mock_agent.rs:224-315
Agent::execute_stream         PARTIAL (single event)  mock_agent.rs:237-246
  multi-event stream            NO (FinalAnswer only)  stream::once
  AgentEvent variety            NO (FinalAnswer/Err)   mock_agent.rs:243,269,323
FailingMockAgent::execute     yes (always errors)     mock_agent.rs:388-398
  error variety                 NO (one variant)       AgentError::InitializationFailed
Tool::execute                  yes                     mock_tool.rs:152-172
Tool::execute_with_context    inherited default       (delegates to execute, ignores ctx)
  ToolFailure (structured)      NO (text error only)   mock_tool.rs:167
  bytes / data / truncated       NO                     not constructed
  pagination                     NO                     not modelled
Embedder::embed               yes (deterministic)     mock_embedder.rs:44-61
```

**Feature gate.** `src/lib.rs:59-60` declares
`#[cfg(any(test, feature = "testing"))] pub mod testing;`. The feature
itself (`Cargo.toml:97`) is an empty `testing = []` — it unlocks the cfg
gate without pulling any implementation dependency. It is NOT in `default`
or `full` (`Cargo.toml:66-67`). Seven examples declare
`required-features = ["testing"]` (`Cargo.toml:253-345`). The downstream
CLI scopes it to `[dev-dependencies]`
(`echo-agent-cli/echo-agent-app-core/Cargo.toml:58-59`).

**Production consumption of the mock.** Every in-crate `use crate::testing::`
is inside a `#[cfg(test)] mod tests` block (V03 enumerates the 11 sites).
The production tool-execution path calls `execute_with_context`
(`src/agent/react/run/pipeline.rs:1395`), to which `MockTool` responds via
the trait default that delegates to `execute` and ignores `ToolContext`
(`echo-core/src/tools/mod.rs:777-783`).

Key facts verified (full evidence in V01–V04):

- **Type-level contract fidelity holds.** Each mock implements the full
  production trait (`LlmClient`, `Agent`, `Tool`, `Embedder`); the
  request/response types are the real `ChatRequest` / `ChatResponse` /
  `ChatChunk` / `Message` / `Usage` / `ToolResult`. No parallel type.
  (V01)
- **Streaming shape is single-chunk.** `MockLlmClient::chat_stream`
  returns `Box::pin(stream::once(async move { Ok(ChatChunk { ... }) }))`
  for both the content and tool-call branches
  (`mock_llm.rs:412-423, 445-456`). There is no public or private API to
  script a sequence of chunks within one stream. (V01, V02)
- **Call-level scripting is rich; stream-level scripting is absent.** The
  response queue (`VecDeque<MockLlmResponse>`) scripts the *sequence of
  calls* (multi-turn ReAct ordering) precisely, including interleaved
  success / tool-call / error. But within one `chat_stream` call, exactly
  one chunk is emitted. (V02)
- **Feature isolation is clean.** No production code path depends on the
  `testing` feature; `cargo check -p echo_agent --no-default-features`
  succeeds (the module is cfg-gated out). The CLI's production
  `[dependencies]` do not enable it. (V03)
- **Mock-driven tests pass but do not exercise the hard paths.** The
  fragmentation, DeepSeek-repair, mid-stream-error, and backpressure paths
  identified in F-RCT-03 / F-LLM-02 are not reachable through the mock.
  (V04)

## Findings

### F-TST-01-P2-01: `MockLlmClient` emits exactly one chunk per stream and cannot reproduce multi-chunk streaming, fragmented tool-call assembly, or DeepSeek argument-repair conditions

- Priority: P2
- Confidence: high
- Layer: framework (testing)
- Evidence:
  - `echo-agent/src/testing/mock_llm.rs:411-423` — content branch:
    `let stream = futures::stream::once(async move { Ok(ChatChunk { delta: DeltaMessage { ... content: Some(text) ... }, ... }) });`
    exactly one chunk carrying the full text.
  - `echo-agent/src/testing/mock_llm.rs:426-457` — tool-call branch:
    builds the complete `DeltaToolCall { index, id, call_type, function: { name, arguments } }`
    in one shot (lines 428-442), then emits it via `stream::once`
    (lines 445-456). No fragmentation across multiple deltas keyed by
    `index`.
  - `echo-agent/src/agent/react/run/phases/think.rs:110-125` — the
    production consumer is a `while let Some(cr) = llm_stream.next().await`
    loop that accumulates `content_buffer` / `reasoning_buffer` /
    `tool_call_map` across chunks. With the mock, this loop executes
    exactly once per turn.
  - `echo-agent/src/agent/react/run/processor.rs:64-83` —
    `process_stream_chunk` accumulates tool-call fragments by `dc.index`
    into `tool_call_map`; the mock never exercises the multi-fragment
    path (it always provides `id` + `name` + full `arguments` on the
    first and only delta for each index).
  - `echo-agent/src/agent/react/run/processor.rs:104-135` —
    `parse_tool_args` DeepSeek trailing-junk repair (strip extra `}` / `]`
    / `,`). No mock-driven test reaches this code path through the
    streaming assembler; only the unit tests at `processor.rs:185-271`
    with hand-crafted JSON strings exercise it.
- Reachability: every mock-driven streaming test. Definition:
  `MockLlmClient::chat_stream`. Registration: `LlmClient` impl at
  `mock_llm.rs:330`. Live callers: `src/agent/react/run/stream_channel.rs`
  test module (~30 test sites), `src/agent/react/run/phases/compact.rs`
  test module, `src/agent/react/builder.rs` test module,
  `src/intent/classifier.rs` test module, `src/agent/default_factory.rs`
  test module, and ~20 sites in `echo-agent-cli`.
- Expected invariant: the task question asks whether the mock "faithfully
  models real streaming". Real providers (`stream_post`,
  `client.rs:317-333`) yield one chunk per SSE event; a single turn
  commonly emits tens to hundreds of content/reasoning deltas and
  multi-fragment tool-call deltas. A mock that emits exactly one chunk
  per turn does not model that shape.
- Observed behavior: every mock-driven streaming test sees a single
  `ChatChunk` per `chat_stream` call. The accumulation loop in
  `run_think` runs once; the `tool_call_map` always has at most one
  delta per index (with full arguments already present); the
  `parse_tool_args` "repair trailing junk" branch is unreachable through
  the mock; the `in_reasoning` state machine never sees a multi-chunk
  reasoning→content transition (the only reasoning path is the
  `#[cfg(test)] then_reasoning_tool_call` helper, which emits reasoning
  in the same single chunk as the tool call).
- Impact: a class of real-provider defects is structurally invisible to
  the mock-driven test suite. Specifically:
  - The DeepSeek "fake-stream" argument-corruption defect that
    `parse_tool_args` was written to defend against
    (`processor.rs:91-99`, citing vLLM #42878) has no end-to-end mock
    coverage; a regression in the repair logic would not be caught by
    any streaming test.
  - The backpressure-drop defects in F-RCT-03-P1-01 / P2-01 cannot be
    reproduced by mock-driven tests (one chunk cannot fill a 256-slot
    buffer).
  - Any future bug in multi-fragment tool-call assembly (e.g. two deltas
    carrying `id` then `name` then `arguments` separately) has no test
    scaffold at the mock layer.
- Root cause: the mock was written to satisfy the trait signature with
  the minimum machinery needed to drive the ReAct loop end-to-end
  (one chunk = one assistant turn). The streaming *shape* (many small
  chunks) was not modelled because the test goal was loop/cancellation
  coverage, not transport-fidelity coverage. There is no
  `with_stream_chunks` / `with_chunk_sequence` builder.
- Direction: add a stream-scripting builder to `MockLlmClient` that lets
  a test queue a `Vec<ChatChunk>` (or `Vec<Result<ChatChunk>>`) to be
  drained chunk-by-chunk by `chat_stream`, mirroring the real SSE
  pattern. Concretely:
  - Add `pub fn with_stream_chunks(self, chunks: Vec<ChatChunk>) -> Self`
    (and an `into_stream`/`yield_chunks` variant for per-call scripting).
  - Extend the private `MockLlmResponse` enum with a `Chunks(Vec<ChatChunk>, Option<Usage>)`
    variant so a single queued response can carry many chunks.
  - In `chat_stream`, drain the chunks via
    `futures::stream::iter(chunks.into_iter().map(Ok))` instead of
    `stream::once`.
  Keep the existing single-shot builders for back-compat (under the
  no-compat rule they could be removed, but they are shorter to write
  and cover the common case; a doc comment should recommend the
  chunk-sequence builder for streaming-shape coverage).
- Regression validation: a new test that queues a multi-chunk content
  stream and asserts the consumer's `content_buffer` equals the
  concatenation of all chunk deltas; a test that queues a tool call
  fragmented across three deltas (`{index:0, id, name}`,
  `{index:0, arguments:"{\"x\":"}`, `{index:0, arguments:"6}"}`) and
  asserts the assembler reconstructs `{"x":6}`; a test that queues a
  DeepSeek-style trailing-junk fragment and asserts `parse_tool_args`
  repair fires. None of these can be written against the current mock.
- Validation reports: [V01](../validations/F-TST-01/V01-01.md),
  [V02](../validations/F-TST-01/V02-01.md),
  [V04](../validations/F-TST-01/V04-01.md).

### F-TST-01-P2-02: `MockLlmClient` cannot simulate mid-stream errors or mid-stream cancellation; errors and cancellation are only modelled before the stream starts

- Priority: P2
- Confidence: high
- Layer: framework (testing)
- Evidence:
  - `echo-agent/src/testing/mock_llm.rs:380-422` — `chat_stream`:
    records the call, then `if let Some(d) = self.delay { select! { token.cancelled() => return Err(...), sleep(d) => {} } }`,
    then `match self.pop_response()? { ... }`. Both the cancel check and
    the error pop happen *before* the stream is constructed.
  - `echo-agent/src/testing/mock_llm.rs:410-422` — once `pop_response`
    returns `Ok(Content(...))`, the returned stream is
    `stream::once(async move { Ok(ChatChunk { ... }) })` — a single `Ok`
    chunk with no error variant and no cancel check inside.
  - `echo-integration/src/providers/client.rs:252-254` (per F-LLM-01) —
    the real transport polls `is_cancelled()` *between* SSE chunks, so a
    real stream can abort after N good chunks.
  - `echo-integration/src/providers/anthropic.rs:482-484` (per F-LLM-01)
    — same mid-stream cancel poll on the Anthropic standalone path.
  - `echo-agent/src/agent/react/run/phases/think.rs:110-111` — the
    consumer's chunk loop:
    `while let Some(cr) = llm_stream.next().await { let chunk = try_send_or!(tx, cr, ThinkOutcome::Abandoned); ... }`
    handles `Result<ChatChunk>` per chunk; an `Err` mid-stream is a
    reachable production case that the mock cannot produce.
- Reachability: every mock-driven streaming test. The single existing
  cancel test (`stream_channel.rs:1972-2038
  test_run_stream_cancelled_mid_llm_call`) uses `with_delay(30s)` and
  cancels during the pre-stream sleep — it cannot test cancellation
  after some chunks have already been emitted.
- Expected invariant: the task question asks whether the mock models
  "error" and "cancellation" contracts faithfully. Real providers can
  fail mid-stream (network drop, server error after SSE start, the
  `[DONE]`-sentinel edge cases noted in F-LLM-02) and can be cancelled
  mid-stream (the production code polls for it between chunks). A mock
  that only errors/cancels before the first chunk does not model this.
- Observed behavior: the mock's `chat_stream` either (a) returns
  `Err(...)` from the future (pre-stream error/cancel), or (b) returns
  a stream that emits exactly one `Ok(ChatChunk)` and ends. There is no
  way to produce a stream that emits N good chunks then `Err`, or that
  emits chunks until a cancel token fires mid-iteration.
- Impact: the chunk-loop error-handling path
  (`try_send_or!(tx, cr, ThinkOutcome::Abandoned)` at `think.rs:111`,
  which forwards an `Err` chunk and bails) is not exercised by any
  mock-driven test. Mid-stream cancellation (the case where the cancel
  arrives after the LLM has started producing tokens but before
  completion) is not distinguished from pre-stream cancellation. A
  regression in mid-stream error forwarding would not be caught.
- Root cause: symmetric to F-TST-01-P2-01 — the single-chunk model
  leaves no "mid" in which an error or cancel can occur. The cancel
  hook was added (`with_delay`, Phase 3 per the comment at
  `mock_llm.rs:64-66, 88-90`) specifically to test cancellation, but
  only at the pre-stream seam.
- Direction: once multi-chunk scripting exists (F-TST-01-P2-01), extend
  it to allow `Result<ChatChunk>` entries in the chunk queue so a test
  can script `Ok(chunk) × N` then `Err(LlmError::NetworkError(...))`.
  For mid-stream cancellation, add a `with_cancel_after_chunks(n)`
  builder that emits `n` chunks then awaits the cancel token (or
  completes if the token never fires), mirroring the real transport's
  per-chunk cancel poll.
- Regression validation: a test that scripts `Ok(chunk) × 2` then
  `Err(NetworkError)` and asserts `run_think` bails with
  `ThinkOutcome::Abandoned` and forwards the error; a test that emits
  chunks slowly and cancels after the first, asserting the stream
  terminates without waiting for the remaining chunks.
- Validation reports: [V02](../validations/F-TST-01/V02-01.md),
  [V04](../validations/F-TST-01/V04-01.md).

### F-TST-01-P2-03: `MockTool` models only text success/error and never constructs `ToolFailure`, `bytes`, `data`, `truncated`, or pagination — the F-EXT-01 structured-failure and bounded-output contracts are untested via the mock

- Priority: P2
- Confidence: high
- Layer: framework (testing)
- Evidence:
  - `echo-agent/src/testing/mock_tool.rs:160-170` — the only
    construction sites: `Some(MockToolResponse::Success(text)) =>
    Ok(ToolResult::success(text))`, `Some(MockToolResponse::Failure(msg))
    => Ok(ToolResult::error(msg))`, `None =>
    Ok(ToolResult::success("mock response"))`. All three produce a
    text-only `ToolResult` with `kind = Text`, no `failure`, no `bytes`,
    no `data`, `truncated = false`.
  - `echo-agent/src/testing/mock_tool.rs:139-172` — the `Tool` impl
    overrides `execute` only; it inherits the default
    `execute_with_context` (`echo-core/src/tools/mod.rs:777-783`) that
    delegates to `execute` and ignores `ToolContext`.
  - `echo-agent/echo-core/src/tools/mod.rs:288-315` — `ToolResult`
    carries `kind` (5 variants), `failure: Option<ToolFailure>`,
    `bytes: Option<Vec<u8>>`, `data: Option<Value>`, `truncated: bool`,
    `mime_type`, `metadata`. The mock populates none of the structured
    fields.
  - `echo-agent/echo-core/src/tools/mod.rs:78-100` — `ToolFailure`
    carries `category` (7 variants), `recovery` (5 variants),
    `side_effect`, `retry_after_ms`, `idempotency_key`, `postcondition`.
    The mock never constructs it.
  - `echo-agent/echo-core/src/tools/pagination.rs:14-32` —
    `PageRequest` / `PageInfo` / `PageError`. The mock has no pagination
    surface.
  - No `with_failure_category`, `with_bytes`, `with_data`,
    `with_truncated`, `with_paginated` builder exists in
    `src/testing/mock_tool.rs` (grep-confirmed: zero hits for
    `ToolFailure`, `bytes`, `data`, `truncated`, `PageRequest` in
    `src/testing/`).
- Reachability: every mock-driven tool test. Definition:
  `MockTool::execute`. Registration: `Tool` impl at `mock_tool.rs:139`.
  Live callers: `src/agent/react/run/stream_channel.rs` test module
  (~15 sites), `src/agent/react/tests.rs` (~20 sites), plus CLI sites.
  Production tool dispatch (`pipeline.rs:1395`) routes through
  `execute_with_context`, which for `MockTool` delegates to `execute`.
- Expected invariant: the task question asks whether the mock "faithfully
  models tool" contracts. The F-EXT-01 contract is explicitly a
  structured `ToolFailure` taxonomy with a `category → recovery` mapping
  and a bounded-output artifact path. A mock that produces only
  text-success / text-error models the pre-F-EXT-01 contract, not the
  current one.
- Observed behavior: tests that use `MockTool::with_failure("msg")`
  produce `ToolResult { success: false, error: Some("msg"),
  failure: None, kind: Error, ... }`. The agent's tool-failure handling
  therefore never observes `ToolFailure.category`, `.recovery`,
  `.side_effect`, or `.idempotency_key` from a mock-driven test. The
  F-EXT-01-P3-02 conservative-recovery mapping is not exercised. The
  artifact-spill path (`ToolOutputArtifactWriter`,
  `ToolResult::truncated`) is not exercised.
- Impact: a regression in structured-failure routing (e.g. a tool
  reporting `ToolFailureCategory::PartialSideEffect` not being routed to
  `VerifyThenRetry`) would not be caught by any mock-driven test. The
  bounded-output / artifact-spill behaviour (a tool returning a large
  payload that spills to the store) is similarly untested via the mock.
  The blast radius is contained because builtin-tool tests (F-EXT-02)
  cover some of this with real tool instances, but the *generic* tool
  contract is tested only through the text-only mock.
- Root cause: `MockTool` predates the F-EXT-01 `ToolFailure` /
  artifact-spill extensions. The structured-failure and bounded-output
  surfaces were added to the contract without a corresponding mock
  builder.
- Direction: add builders that let a test construct the full
  `ToolResult` surface:
  `with_failure_structured(ToolFailure)`,
  `with_bytes(Vec<u8>, mime_type)`,
  `with_data(Value)`,
  `with_truncated(ArtifactRef)`,
  and a paginated variant. Each appends a pre-built `ToolResult` to the
  response queue (the existing `Success`/`Failure` text variants stay
  for convenience). Under the no-compat rule this is a pure addition.
- Regression validation: a test that scripts a tool returning
  `ToolFailure { category: PartialSideEffect, ... }` and asserts the
  agent's retry path routes through verification rather than blind retry
  (the F-EXT-01 safety pattern); a test that scripts a tool returning
  `truncated = true` and asserts the consumer sees the artifact ref.
- Validation reports: [V01](../validations/F-TST-01/V01-01.md),
  [V04](../validations/F-TST-01/V04-01.md).

### F-TST-01-P3-01: `FailingMockAgent` models exactly one error variant (`AgentError::InitializationFailed`); the orchestration fault-tolerance tests it backs do not cover the diverse real-agent failure modes

- Priority: P3
- Confidence: high
- Layer: framework (testing)
- Evidence:
  - `echo-agent/src/testing/mock_agent.rs:388-398` —
    `FailingMockAgent::execute` always returns
    `Err(ReactError::Agent(Box::new(AgentError::InitializationFailed(self.error_message.clone()))))`.
  - `echo-agent/src/testing/mock_agent.rs:400-413` — `execute_stream`
    derives its error from `execute`, so the stream also only ever
    yields `Err(ReactError::Agent(...InitializationFailed...))`.
  - `echo-agent/echo-core/src/error.rs:186-248` — `AgentError` has
    `NoToolsAvailable`, `Interrupted`, `TokenLimitExceeded`,
    `TransportClosed`, plus the inherited `NoResponse` /
    `MaxIterationsExceeded` from `ReactError`. None of these are
    producible by `FailingMockAgent`.
  - Live callers: `src/agent/subagent/executor.rs:2385, 2438` (primary/
    recovery failover tests), `src/agent/react/tests.rs:496` (reset
    test).
- Reachability: every orchestration fault-tolerance test that uses
  `FailingMockAgent`.
- Expected invariant: a mock named "FailingMockAgent" used to test
  "orchestration fault-tolerance behavior" (per its docstring at
  `mock_agent.rs:352-353`) should let the test choose the failure mode,
  because orchestration recovery routing depends on the error category.
- Observed behavior: all failover/fault-tolerance tests using
  `FailingMockAgent` exercise only the `InitializationFailed` arm. The
  behaviour under `NoResponse`, `MaxIterationsExceeded`,
  `TokenLimitExceeded`, `TransportClosed`, or `Interrupted` is not
  tested via this mock.
- Impact: low. The fault-tolerance routing for `InitializationFailed` is
  the same as for the other variants in the current orchestration code
  (it triggers failover/recovery uniformly), so the narrow error
  variety does not mask a live bug today. The finding is preventive: if
  recovery routing ever differentiates by error category, the test
  scaffold will not cover the other categories.
- Root cause: `FailingMockAgent` was written for the simplest
  "sub-agent fails, orchestrator recovers" scenario and was not
  extended when `AgentError` grew.
- Direction: add `FailingMockAgent::with_error(ReactError)` (or a
  `with_error_kind(AgentError)` builder) so a test can choose the
  variant. Cheap, pure addition. Alternatively, deprecate
  `FailingMockAgent` in favour of `MockAgent` plus a `with_error`
  response variant (consolidates two types into one).
- Regression validation: a test that constructs a `FailingMockAgent`
  returning `ReactError::Agent(MaxIterationsExceeded)` and asserts the
  orchestrator's recovery path fires identically.
- Validation reports: [V01](../validations/F-TST-01/V01-01.md).

### F-TST-01-P3-02: `MockAgent::execute_stream` emits only `FinalAnswer`; the streaming variety of `AgentEvent` (ThinkStart/Token/LlmUsage/ToolCall/...) is never produced, so orchestration tests cannot assert on subagent streaming-event shape

- Priority: P3
- Confidence: high
- Layer: framework (testing)
- Evidence:
  - `echo-agent/src/testing/mock_agent.rs:237-246` —
    `MockAgent::execute_stream`:
    `let event_stream = stream::once(async move { Ok(AgentEvent::FinalAnswer(answer)) });`
  - `echo-agent/src/testing/mock_agent.rs:252-272, 274-287, 289-303,
    317-326` — every streaming override (`execute_stream_message_*`,
    `execute_stream_with_invocation_context`,
    `execute_stream_message_with_invocation_context`, `chat_stream`)
    delegates to the same single-`FinalAnswer` pattern.
  - `echo-agent/echo-core/src/agent/mod.rs:140-239` — `AgentEvent` has
    ~15 variants (`ThinkStart`, `ThinkEnd`, `Token`, `LlmUsage`,
    `ToolBatchStart`, `ToolCall`, `ToolResult`, `ToolError`,
    `MemoryRecalled`, `ContextCompressed`, `Cancelled`, `Error`, etc.).
    `MockAgent` produces exactly one (`FinalAnswer`), plus `Err` via
    `FailingMockAgent`.
- Reachability: every orchestration test that streams from a
  `MockAgent` (e.g. subagent dispatch tests in
  `src/agent/subagent/executor.rs:2117-2520`).
- Expected invariant: the task question asks whether the mock models
  "ordering contracts". A real subagent (e.g. a `ReactAgent` spawned as
  a subagent) emits the full `AgentEvent` sequence
  (`MemoryRecalled`? → `ThinkStart` → `Token` × N → `ThinkEnd` →
  `ToolBatchStart`/`ToolCall`/`ToolResult` → `FinalAnswer`). A mock
  that emits only `FinalAnswer` models the terminal, not the ordering.
- Observed behavior: orchestration tests that consume a `MockAgent`
  stream see `[FinalAnswer]` and cannot assert on intermediate events
  (e.g. "the parent saw the subagent's `ToolCall` before its
  `FinalAnswer`"). The mock faithfully models the *terminal* ordering
  contract (one terminal per stream) but not the *intermediate* event
  ordering.
- Impact: low for current orchestration tests (they assert on the final
  answer and call counts, not on intermediate subagent events). The
  finding is preventive: any future orchestration feature that routes
  or projects subagent intermediate events (e.g. a parent UI showing a
  child's thinking) has no mock scaffold to test against.
- Root cause: `MockAgent` was designed around `execute` (the
  non-streaming entry point); the streaming overrides were added later
  to satisfy the trait surface (especially the multimodal
  `execute_stream_message_*` overrides added in Sprint 8 per the
  comments at `mock_agent.rs:248-251, 335-336`) with the minimum
  possible event shape.
- Direction: add a `with_stream_events(Vec<AgentEvent>)` builder (and
  an `Err`-terminated variant) so a test can script a subagent's event
  sequence. Compose with the existing `with_response` for the common
  single-FinalAnswer case. This is symmetric to the
  `MockLlmClient` chunk-scripting recommendation in F-TST-01-P2-01.
- Regression validation: a test that scripts
  `[ThinkStart, Token("..."), FinalAnswer("done")]` and asserts the
  consumer observes all three in order.
- Validation reports: [V01](../validations/F-TST-01/V01-01.md),
  [V02](../validations/F-TST-01/V02-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Mock-vs-provider contract matrix: each mock implements the production trait; document which behaviours are modelled and which are absent | yes | passed (matrix recorded, gaps noted) | [V01-01](../validations/F-TST-01/V01-01.md) |
| V02 | Scripted ordering / error fixtures: mocks script call-level sequences (yes) and stream-level chunk sequences (no); errors and cancellation modelled pre-stream only | yes | passed (capability documented, stream-level gap noted) | [V02-01](../validations/F-TST-01/V02-01.md) |
| V03 | Testing feature isolation: `testing` is gated by `cfg(any(test, feature = "testing")))`, absent from `default`/`full`, scoped to `[dev-dependencies]` downstream; no production leak | yes | passed | [V03-01](../validations/F-TST-01/V03-01.md) |
| V04 | Production tests relying on unrealistic mock behaviour: mock-driven tests pass but do not exercise multi-chunk streaming, fragmented tool assembly, DeepSeek repair, mid-stream error/cancel, structured tool failure, or backpressure | yes | passed (tests green; coverage gaps noted as findings) | [V04-01](../validations/F-TST-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | No prior F-TST-01 report exists; the only historical artefact is the `src/testing/mod.rs` design-principle docstring, treated as current and re-verified in V01. |

Executed cargo commands (all exit 0):

```text
cd echo-agent
cargo check -p echo_agent --no-default-features --locked
  (V03 — crate compiles without the testing module; gate confirmed)
cargo test --lib -p echo_agent --locked -- \
  run_core_loop_text_only_yields_final_answer \
  run_core_loop_tool_call_cycle_completes \
  run_core_loop_empty_llm_response_terminates_gracefully \
  deepseek_tool_turn_replays_complete_assistant_message \
  test_run_stream_cancelled_mid_llm_call \
  react_stream_records_real_usage_in_run_trace \
  value_scoped_direct_answer_records_usage_in_child_trace
  (V04 — 7 passed, 0 failed; the mock-driven suite is green)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `src/testing/mod.rs:13-18` — "Design Principles: Zero network requests; Scriptable; Observable; Thread-safe" | current | V01 confirms all four hold. The mocks run in-memory, are scriptable via response queues, expose call-count/last-args/etc., and use `Arc<Mutex<_>>`. The "Scriptable" claim is accurate at the call level but not at the stream-chunk level (V02). |
| `src/testing/mod.rs:7-11` — type/purpose table (`MockLlmClient` for `SummaryCompressor` and `LlmClient` deps; `MockTool` for tool-call/error handling; `MockAgent`/`FailingMockAgent` for orchestration) | current | V01 confirms each mock is used for its stated purpose across ~60 in-crate + ~20 CLI sites. |
| `src/testing/mock_llm.rs:49-54` — "Returns preset responses in order; once the queue is exhausted, returns an `EmptyResponse` error" | current | V02 confirms `pop_response()` returns `Err(EmptyResponse)` when the queue is empty (`mock_llm.rs:320`). |
| `src/testing/mock_llm.rs:64-66, 88-90` — delay + cancel-awareness "lets tests verify mid-flight cancellation (Phase 3)" | partially stale | V02 confirms the cancel hook exists and works, but it fires only *before* the stream starts (pre-stream), not mid-stream. The "mid-flight" phrasing is accurate for the `chat` (non-streaming) path and for the single-chunk stream's pre-emit delay, but does not cover the real-provider mid-stream cancellation case. Feeds F-TST-01-P2-02. |
| `src/testing/mock_agent.rs:42-50` — "Returns preset responses in order; once the queue is exhausted, each call returns `"mock agent response"`" | current | V01 confirms `next_response()` falls back to the default string (`mock_agent.rs:202-208`). |

## Coverage And Uncertainty

- Code inspected in full: all five files under `src/testing/` (1230 lines
  total), the `src/lib.rs` gates (lines 59-60, 274-275), the `Cargo.toml`
  feature table (lines 65-97) and `[[example]]` required-features
  (lines 253-345), the real-provider streaming references
  (`client.rs:182-354`, `openai.rs:322-377`, `anthropic.rs:421-628` per
  F-LLM-01), and the consumer side (`think.rs:100-125`,
  `processor.rs:16-183`).
- Mock consumption sites sampled (not exhaustively traced): the
  `stream_channel.rs` test module, `react/tests.rs`, the `subagent`
  test modules, the CLI test modules. The grep enumeration (166 hits
  across the two repos, excluding the `src/testing/` definitions) gives
  high confidence that the usage pattern is uniform; the conclusions do
  not depend on a per-site trace.
- Validations executed: `cargo check --no-default-features` (V03) and a
  7-test subset of the mock-driven streaming suite (V04). The full
  feature matrix and the CLI test suite were not re-run; the task
  question is about mock fidelity, and the two executed commands are
  sufficient to confirm (a) the gate excludes the module in production
  and (b) the mock-driven tests pass.
- Environmental limits: builds used the existing incremental cache;
  `cargo check -p echo_agent --no-default-features --locked` completed
  in ~3 minutes (initial lock-contention wait). No `cargo clean` needed
  (disk pressure well below the AGENTS.md threshold). Final worktree
  state is clean (`git status` clean, commit `9b0e0fa`).
- Claims that remain uncertain:
  - Whether any downstream consumer outside this monorepo depends on
    the single-chunk stream shape of `MockLlmClient` (a third-party
    test that asserts "exactly one chunk" would need updating if the
    chunk-scripting builder is added). Under AGENTS.md no-compat rule
    this is acceptable, but the direction in F-TST-01-P2-01 keeps the
    existing builders for back-compat.
  - Whether the `Tool::execute` (vs `execute_with_context`) delegation
    for `MockTool` masks any `ToolContext`-dependent behaviour in
    audited tests. The default delegation ignores `ctx`
    (`echo-core/src/tools/mod.rs:777-783`), so a test that asserts on
    working-dir-aware tool output would silently get the default
    context. No audited test does this, but the scaffold does not
    prevent it.

## Handoff

- Conclusions downstream tasks may rely on:
  1. **The mocks are trait-faithful at the type level.** Every mock
     implements the full production trait with the real
     request/response types. Downstream tasks can rely on the mock
     surface for wiring/registration/cancellation tests.
  2. **The mocks do NOT model streaming-shape, structured-tool-failure,
     or bounded-output behaviour.** Any task that needs to test
     multi-chunk streaming, fragmented tool-call assembly, DeepSeek
     argument repair, mid-stream error/cancel, `ToolFailure`-category
     routing, or artifact spill cannot use the current mocks and must
     either (a) wait for the F-TST-01-P2-01/P2-03 directions to land or
     (b) build a one-off fixture. The relevant production-path
     findings (F-RCT-03-P1-01/P2-01 backpressure, F-LLM-02 SSE edge
     cases) are therefore not mock-covered.
  3. **Feature isolation is clean.** The `testing` feature is gated by
     `cfg(any(test, feature = "testing")))`, is absent from `default`
     and `full`, and the downstream CLI scopes it to
     `[dev-dependencies]`. No production leak exists at the audited
     commits. Downstream tasks can rely on this.
  4. **`MockTool` ignores `ToolContext`.** Tests that need to verify
     context-aware (working-dir, cancellation-via-context) tool
     behaviour cannot use the stock `MockTool`; they must override
     `execute_with_context` in a custom stub.
- Reports they must read:
  - [V01-01](../validations/F-TST-01/V01-01.md) for the full
    mock-vs-provider behaviour matrix.
  - [V02-01](../validations/F-TST-01/V02-01.md) for the
    call-level vs stream-level scripting analysis.
  - [V03-01](../validations/F-TST-01/V03-01.md) for the feature-gate
    and leak audit.
  - [V04-01](../validations/F-TST-01/V04-01.md) for the mock-driven
    test-pass + coverage-gap analysis.
  - `tasks/F-LLM-01.md`, `tasks/F-LLM-02.md` for the real-provider
    streaming/cancellation/error contract the mock should model.
  - `tasks/F-RCT-03.md` for the production-side streaming defects that
    are not mock-reproducible.
  - `tasks/F-EXT-01.md` for the `Tool`/`ToolResult`/`ToolFailure`
    contract the mock should cover.
- Conditions that make this report stale:
  - Adding a `with_stream_chunks` / `with_chunk_sequence` builder to
    `MockLlmClient` invalidates F-TST-01-P2-01 and F-TST-01-P2-02.
  - Adding `Result<ChatChunk>` entries to the chunk queue, or a
    `with_cancel_after_chunks` builder, invalidates F-TST-01-P2-02.
  - Adding `with_failure_structured` / `with_bytes` / `with_data` /
    `with_truncated` builders to `MockTool` invalidates F-TST-01-P2-03.
  - Adding `with_error` / `with_error_kind` to `FailingMockAgent`
    invalidates F-TST-01-P3-01.
  - Adding `with_stream_events` to `MockAgent` invalidates
    F-TST-01-P3-02.
  - Moving the `testing` feature into `default` or `full`, or adding it
    to the CLI's production `[dependencies]`, invalidates V03 and the
    "feature isolation is clean" handoff conclusion.
- Follow-up task IDs (no fixes implemented in this review):
  - A **mock-fidelity enhancement task** should land the
    `MockLlmClient` chunk-scripting builder (F-TST-01-P2-01), the
    mid-stream error/cancel builder (F-TST-01-P2-02), the `MockTool`
    structured-failure/bytes/data builders (F-TST-01-P2-03), the
    `FailingMockAgent::with_error` builder (F-TST-01-P3-01), and the
    `MockAgent::with_stream_events` builder (F-TST-01-P3-02). These
    are pure additions under the no-compat rule and unblock
    regression coverage for F-RCT-03 / F-LLM-02 / F-EXT-01.
  - The **application testing task** (the `echo-agent-cli` test-quality
    review) should re-audit the ~20 CLI sites that consume these mocks,
    because they inherit the same coverage gaps until the mock builders
    land.
