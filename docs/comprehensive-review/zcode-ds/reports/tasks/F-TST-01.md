# F-TST-01: Framework test and mock utilities

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Do public/internal mocks and testing helpers faithfully model real streaming,
tool, usage, error, cancellation, and ordering contracts?

Answer: **no**. Request-level scripting (text/error/429 responses, usage on a
content chunk, tool calls in one chunk) is faithful; every wire-shape
dimension the framework must survive is either unscriptable or modeled
inverted: streaming emits exactly one chunk (no ordering, no mid-stream
errors, no incremental tool-call assembly, no usage-on-final-chunk shape),
cancellation is modeled as a loud `ReactError::Other` instead of the real
silent end-of-stream, and `MockTool` can only script Permanent (non-retryable)
failures. The green loop suite (V04-03) therefore certifies shapes real
providers never produce — it structurally cannot reproduce F-LLM-03-P1-01,
F-LLM-03-P1-02, F-LLM-01-P1-01, F-RCT-04-P1-01/P1-02, or F-RCT-03-P1-02.

## Scope

- `echo-agent/src/testing/` full reads: `mod.rs` (docs + exports),
  `mock_llm.rs` (466 lines), `mock_tool.rs` (174 lines), `mock_agent.rs`
  (425 lines), `mock_embedder.rs` (63 lines); the `testing` feature
  (`Cargo.toml:97`, `lib.rs:59-60,274-275`).
- Mock consumers: `src/agent/react/run/stream_channel.rs` (23 loop tests,
  helpers `agent_with_mock_llm`/`collect_events`), `phases/think.rs`,
  `phases/compact.rs`, `builder.rs`, `intent/classifier.rs`,
  `react/tests.rs`, `agent/subagent/executor.rs`, `src/agent/snapshot.rs:343`
  (`llm_client` injection point), `src/agent/react/run/phases/think.rs:95-250`
  (usage/chunk consumption).
- Contract anchors: `echo-core/src/tools/mod.rs:319-398,154-162` (ToolResult/
  ToolFailure), `echo-execution/src/tools.rs:618-730` (retry gate),
  `echo-integration/src/providers/client.rs:251-256` (silent cancel).
- Duplicate search: `echo-state/src/memory/mod.rs:54-92` (second MockEmbedder);
  `echo-orchestration/src/human_loop/classifier.rs:821-857` (local MockClient);
  `echo-tools/src/web/search.rs:230` (local MockProvider); echo-agent-cli
  test modules using `echo_agent::testing` (chat_driver.rs, agent_pool.rs,
  scheduler/runner.rs, unified_memory.rs, tasks/*).
- Docs: `echo-agent/docs/en/12-mock.md` (full), `docs/MASTER-PLAN.md`
  acceptance items (:151), `tests/react_smoke.rs` header, `README.md:1168`.

## Out Of Scope

- Provider adapter internals and their unit tests → F-LLM-02/F-LLM-03
  (completed); only the wire shapes the mocks must model are referenced.
- Loop/phase behavior itself (ordering, terminal loss, cancel) → F-RCT-02/03/04
  (completed); this task uses their findings as the real-contract baseline.
- Tool contract and failure taxonomy → F-CORE-01/F-EXT-01 (completed).
- Q-TST-01 (suite credibility map) consumes this report; it does not reread
  the mock code.
- EKO-side test strategy (CLI test modules) beyond the dev-dependency
  feature wiring.

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, no-parallel-semantics, layering),
  shared `README.md`, `REPORTING.md`, `TASKS.md` (F-TST-01 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-LLM-01` (complete), `F-RCT-03`
  (complete), `F-EXT-01` (complete) — the three declared dependencies; plus
  `F-LLM-03` and `F-RCT-04` (complete) for the explicitly cross-referenced
  findings F-LLM-03-P1-02 and F-RCT-04-P2-01.
- Historical documents treated as hypotheses: `echo-agent/docs/en/12-mock.md`,
  `docs/MASTER-PLAN.md` acceptance claims, `tests/react_smoke.rs` header,
  `src/testing/mod.rs` design-principle docs.

## Layering Decision

- Generic mechanism (framework, correctly placed in `echo-agent` root crate):
  the shared mocks are generic testing utilities any echo-agent consumer
  needs; they belong in the framework, not the application. The `testing`
  feature (empty, no deps) is the correct isolation mechanism.
- EKO product policy (application): none — echo-agent-cli consumes the
  framework mocks only via its own dev-dependencies
  (`echo-agent-app-core/Cargo.toml:59`), which is the correct adapter-style
  boundary; no application-side mock authority exists.
- Adapter boundary: the mock's `LlmClient`/`Tool`/`Agent` impls are the
  adapter from scripted behavior to the framework contract; the fidelity
  defect is precisely that this adapter models a non-existent wire.
- Duplicate-search terms (both repositories): `MockLlmClient`, `MockTool`,
  `MockAgent`, `FailingMockAgent`, `MockEmbedder`, `mod testing`,
  `feature = "testing"`, `struct Mock|Fake|Stub`, `impl LlmClient for`,
  `impl Agent for`, `impl Embedder for`, `then_tool_calls`,
  `stream::once`, wire-fixture files. Results (V01): one shared mock module;
  one byte-identical duplicate `MockEmbedder` in `echo-state` test_utils
  (P3-01); 4 local `#[cfg(test)]` doubles (classifier MockClient, executor
  CancellationAwareStreamAgent, team UsageAgent, workflow RecordingAgent —
  intentional, scoped); `then_tool_calls` zero callers (re-confirms
  F-RCT-04-P2-01); no wire fixtures anywhere in echo-integration.
- Cross-repository boundary gate: nothing moves between repositories; all
  findings stay in `echo-agent` (framework test infrastructure). The
  `MockEmbedder` dedup direction moves code from `echo-agent` root and
  `echo-state` into `echo_core` (where the `Embedder` trait lives).

## Current Path

`#[cfg(any(test, feature = "testing"))] pub mod testing` (lib.rs:59-60) →
`MockLlmClient`/`MockTool`/`MockAgent`/`FailingMockAgent`/`MockEmbedder`
re-exported (lib.rs:274-275) → consumed by (a) in-crate `#[cfg(test)]`
tests (stream_channel.rs 23 tests via `agent_with_mock_llm`, think.rs,
compact.rs, builder.rs, intent/classifier.rs, react/tests.rs,
subagent/executor.rs, team/*, eval/runner.rs), (b) 7 examples with
`required-features = ["testing"]`, (c) echo-agent-cli dev-dependency
(`echo-agent-app-core/Cargo.toml:59`). Production builds never see the mocks
(V02, V04-02). The full ReAct loop is driven through the mocks via the
snapshot-level `llm_client` field (snapshot.rs:343) → `create_llm_stream`
(think.rs:99-103). Usage flows: chunks → `last_usage` (think.rs:112-113) →
`usage_reported = last_usage.is_some()` (think.rs:147) → `AgentEvent::LlmUsage`
(think.rs:199-211). Stream flows: `chat_stream` returns a one-chunk
`stream::once` (mock_llm.rs:410-458); `think.rs` iterates chunks, applies
`process_stream_chunk` per chunk (incremental tool-call assembly and content
buffering are only ever exercised with a single chunk). Cancel: `with_delay`
+ cancelled token → `Err(ReactError::Other(...))` before the stream exists
(mock_llm.rs:349-357,400-408). Tool failures: `MockTool::with_failure` →
`ToolResult::error` → hardcoded `ToolFailureCategory::Permanent`
(echo-core/tools/mod.rs:373-385) → never retryable at
`execute_tool_inner` (echo-execution/tools.rs:687-698). Subagent orchestration:
`MockAgent::execute_stream` emits exactly one `FinalAnswer` and ignores the
cancel token (mock_agent.rs:237-246,255,277,317-326).

## Findings

### F-TST-01-P1-01: `MockLlmClient` emits content and usage in one chunk — the loop-level suite certifies `usage_reported: true` in a wire shape no real provider produces, hiding the F-LLM-03-P1-02 usage loss

- Priority: P1
- Confidence: high
- Layer: framework (test infrastructure)
- Evidence: `echo-agent/src/testing/mock_llm.rs:410-425` (`chat_stream` yields
  one `stream::once` chunk carrying `content + finish_reason("stop") +
  usage`); `mock_llm.rs:359-377` (same single-shot shape in `chat`); consumer
  `echo-agent/src/agent/react/run/phases/think.rs:112-113,147` (`last_usage`
  taken from any chunk carrying usage; `usage_reported = last_usage.is_some()`);
  tests asserting the mock shape as truth: `stream_channel.rs:1755-1809`
  (`react_stream_records_real_usage_in_run_trace` asserts `usage_reported:
  true`), `:1230-1301`, `:1811-1859`, `:1860-1898`; real wire contrast:
  Anthropic `message_delta.usage` = `{output_tokens}` only, separate final
  chunk, dropped by the strict `AnthropicUsage` deserializer →
  `usage_reported: false` on every real Anthropic stream (F-LLM-03-P1-02).
- Reachability: `MockLlmClient` drives the full loop in every streaming test
  of `stream_channel.rs` (23 tests), think/compact/builder/intent tests, and
  the echo-agent-cli test modules (chat_driver.rs:764-1073,
  task_runtime/executor.rs:4503-5255) via the dev-dependency feature; there is
  no other loop-level LLM double.
- Expected invariant: the mock can express the wire shapes the framework must
  survive, including usage arriving on a final chunk separate from content
  (OpenAI) and a usage-only final chunk (Anthropic), so loop tests exercise
  real provider accounting semantics.
- Observed behavior: the only scriptable usage shape is "usage on the same
  chunk as the content/tool calls"; no usage-only-final-chunk fixture exists;
  the suite therefore asserts `usage_reported: true` semantics that the real
  Anthropic path never reaches, and the F-LLM-03-P1-02 regression tests
  prescribed for the loop level cannot be written with this mock.
- Impact: token accounting, cache-hit observability, and tokenizer
  calibration silently break on the main Anthropic/DeepSeek-Anthropic path
  while 23+ loop tests stay green certifying the opposite; the framework
  suite cannot distinguish "provider didn't report usage" from "adapter
  dropped it" (F-LLM-03-P1-02) by construction.
- Root cause: the mock was designed as a request-level response queue (for
  SummaryCompressor-era tests) and later reused to drive the streaming loop
  without ever modeling chunk sequences; `think.rs`'s "last chunk with usage"
  rule was never validated against real provider chunk layouts.
- Direction: add scriptable chunk-sequence scripting to `MockLlmClient`
  (e.g. `then_chunks(vec![...])` with per-chunk delta/usage/finish_reason),
  including (a) content chunk then usage-only final chunk, (b) the exact
  Anthropic `{"type":"message_delta",...,"usage":{"output_tokens":15}}`
  payload shape; add a loop-level regression test asserting `usage_reported:
  true` for (a) and (b); align with the F-LLM-03-P1-02 fix and its adapter
  fixtures.
- Regression validation: loop test with chunks [content delta, final
  usage-only chunk {output_tokens}] asserting `AgentEvent::LlmUsage` with
  `usage_reported: true` and correct cached/creation token mapping; the
  same fixture through a fixed Anthropic adapter.
- Validation reports: [V03](../validations/F-TST-01/V03-01.md),
  [V04-03](../validations/F-TST-01/V04-03.md)

### F-TST-01-P1-02: Streaming is not scriptable at all — one-chunk streams only; no multi-chunk ordering, no mid-stream errors, no incremental tool-call deltas — the streaming contract (F-RCT-03's central invariants) has zero loop-level negative fixtures

- Priority: P1
- Confidence: high
- Layer: framework (test infrastructure)
- Evidence: `mock_llm.rs:410-458` (single `stream::once` chunk for content and
  tool calls; `chat_stream -> Result<BoxStream>` means errors can only occur
  before the stream starts — `pop_response()?` at `:410`); no API yields an
  `Err` stream item or a second chunk; all tool calls are delivered complete
  in one chunk (`:426-458`, index by `enumerate`); `reasoning_content` is
  always `None` (`:416-418`; only `#[cfg(test)] then_reasoning_tool_call`
  at `:179-204`, still one chunk); consumers: `think.rs:105-133` (per-chunk
  loop, `process_stream_chunk` incremental assembly), `stream_channel.rs:
  405-414` (`direct_answer_stream` per-delta Token emission) — both are only
  ever exercised with exactly one chunk.
- Reachability: every loop-level streaming test; every future consumer of the
  `testing` feature.
- Expected invariant: mocks can script multi-chunk sequences with per-delta
  content/reasoning, interleaved tool-call deltas, mid-stream errors, and
  ordering — the dimensions of the F-RCT-03 task question (ordered, lossless,
  bounded, conformance).
- Observed behavior: no such capability exists; the interleaved-block
  tool-call corruption (F-LLM-03-P1-01: accumulator keyed by length vs stream
  index), the malformed-chunk drop (F-LLM-01-P1-01), the batch result-order
  conflict (F-RCT-04-P1-01), and the content-burst ordering quirks
  (F-RCT-03-P2-01) are all structurally unreproducible at the loop level.
- Impact: the framework's core streaming contract is validated only against a
  degenerate one-chunk stream; every shipped streaming defect above passed a
  green suite for exactly this reason; Q-FLT-01 and Q-TST-01 have no reusable
  streaming fault fixtures and must hand-roll a second mock instead of
  reusing the public testing feature.
- Root cause: `chat_stream` was implemented as "`chat` plus one chunk" when
  the streaming loop refactor landed; the mock never grew a chunk-sequence
  model.
- Direction: extend `MockLlmClient` with a chunk-sequence API (ordered list of
  `DeltaMessage`/tool-call deltas/usage/finish_reason items plus mid-stream
  `Err` injection), or introduce a second streaming-focused double in
  `src/testing`; script the F-LLM-03-P1-01 [text, tool_use] and [tool_use,
  text, tool_use] sequences; script a malformed chunk per F-LLM-01-P1-01.
- Regression validation: loop test with a two-chunk [text delta, tool_use
  delta] stream asserting one correctly assembled tool call; a mid-stream
  `Err` fixture asserting the loop's error handling; an interleaved
  multi-tool fixture asserting call identity/args per F-LLM-03-P1-01.
- Validation reports: [V03](../validations/F-TST-01/V03-01.md),
  [V01](../validations/F-TST-01/V01-01.md)

### F-TST-01-P2-01: `MockTool` can only script Permanent failures — retryable/timeout/cancel/partial-side-effect fixtures are impossible through the public mock; loop tests hand-write tools instead

- Priority: P2
- Confidence: high
- Layer: framework (test infrastructure)
- Evidence: `mock_tool.rs:167` (`Failure(msg) => Ok(ToolResult::error(msg))`);
  `echo-core/src/tools/mod.rs:373-385` (`ToolResult::error` hardcodes
  `ToolFailure::new(ToolFailureCategory::Permanent)` — never retryable);
  retry gate `echo-execution/src/tools.rs:687-698`
  (`result.failure.as_ref().is_some_and(ToolFailure::allows_automatic_retry)`);
  contrast real tools: `echo-tools/src/shell.rs:1061,1121`,
  `echo-tools/src/code.rs:382-405`, `echo-tools/src/web/search.rs:181`
  (`ToolResult::error(msg).with_failure(ToolFailure::new(Transient).retryable())`).
  `MockTool` has no delay, no cancel hook, no `execute_stream`, no oversized
  output; the loop's only slow-tool test hand-writes `DelayedTerminalTool`
  (`stream_channel.rs:2041-2082`); echo-execution's own retry tests use
  hand-written closures (`tools.rs:997,1072`).
- Reachability: every mock-driven loop/orchestration test that needs a
  failing tool; the retry path of `execute_tool_inner` is otherwise covered
  only by echo-execution's private tests.
- Expected invariant: the mock tool can script the failure categories and
  timing behaviors the framework must handle (retry, timeout, cancel,
  partial side effect) so loop tests exercise them.
- Observed behavior: `with_failure` scripts only non-retryable Permanent; no
  timing/cancellation/streaming capability; MASTER-PLAN:151's cancel-during-
  tool scenarios are limited to the one hand-written-tool test.
- Impact: tool retry/timeout/cancel/partial-side-effect paths are untestable
  at the loop level through the public testing feature; F-RCT-04-P1-02
  (batch timeout/cancel without typed terminal) and F-RCT-04-P2-02 (killed
  tools leave no failure record) shipped with zero fixture coverage, and the
  F-RCT-04-P2-01 fixture family cannot reuse `MockTool`.
- Root cause: `MockTool` mirrors the pre-taxonomy `ToolResult::error`
  contract; the `ToolFailure` classification (F-CORE-01/F-EXT-01) evolved
  after the mock and was never wired into it.
- Direction: extend `MockTool` with `with_failure(ToolFailure)` (so
  Transient/Timeout/Unavailable/retry_after can be scripted), an optional
  delay, and cancellation awareness; or explicitly document MockTool as
  success/plain-error-only and provide a second tool double for
  classification fixtures. Keep `ToolResult::error`'s Permanent default
  unchanged (it is a framework contract, F-EXT scope).
- Regression validation: loop test with a Transient+retryable MockTool
  failure asserting two attempts and eventual success; a timeout-scripted
  tool asserting the partial-side-effect/typed-failure path.
- Validation reports: [V03](../validations/F-TST-01/V03-01.md)

### F-TST-01-P2-02: `MockAgent` streams only a single `FinalAnswer` and ignores cancellation — orchestration tests cannot model agent event vocabulary or cancel propagation

- Priority: P2
- Confidence: high
- Layer: framework (test infrastructure)
- Evidence: `mock_agent.rs:237-246` (`execute_stream` → one
  `stream::once(Ok(AgentEvent::FinalAnswer))`), `:317-326` (`chat_stream`
  same), `:255,277` (`_cancel` ignored), `:252-272` (message variant also
  single FinalAnswer); no Token/ThinkStart/ToolBatch/ToolCall/Error/Cancelled
  events are ever scriptable; consumers: `subagent/executor.rs:2117-2385`,
  team/manager modules, `eval/runner.rs`, workflow pipelines.
- Reachability: every orchestration-level test that consumes a subagent event
  stream or cancels a subagent mid-flight.
- Expected invariant: a mocked Agent stream models the event vocabulary and
  cancel semantics of a real Agent (F-RCT-03 terminal vocabulary:
  Token/ToolBatch/FinalAnswer/Cancelled/Error).
- Observed behavior: consumers are tested against a one-event stream;
  cancellation inside the mock is a no-op — executor-level cancel tests race
  the dispatch timeout/cancel against `with_delay_ms` sleep
  (executor.rs:2483,2535) and never see an agent-level `Cancelled` event;
  MASTER-PLAN:151's cancel-during-subagent acceptance is only partially
  implemented (dispatch-level, not event-level).
- Impact: orchestration event handling and cancel propagation (F-SUB-02
  scope) have no mock path; the framework's two cancel vocabularies
  (F-RCT-03-P1-02: main loop never emits Cancelled; subagent executor does)
  are invisible to orchestration tests.
- Root cause: `MockAgent` was written for text-in/text-out orchestration; the
  streaming Agent contract (events + cancel token) was never modeled.
- Direction: allow scripting a per-call sequence of `AgentEvent`s (including
  `Error`/`Cancelled`), and honor the cancel token (stop mid-sequence or emit
  `Cancelled`) mirroring real Agent semantics; update the doc examples that
  only show text returns.
- Regression validation: orchestration test scripting [Token, FinalAnswer]
  and asserting both events pass through the executor; a cancel-token test
  asserting the stream ends with `Cancelled`.
- Validation reports: [V03](../validations/F-TST-01/V03-01.md),
  [V05](../validations/F-TST-01/V05-01.md)

### F-TST-01-P2-03: Cancellation is modeled as a loud `ReactError::Other` error instead of the real silent end-of-stream — the only cancel-path test exercises a behavior real providers never produce

- Priority: P2
- Confidence: high (behavioral divergence is code-certain; loop impact
  already established by F-RCT-03-P1-02)
- Layer: framework (test infrastructure)
- Evidence: `mock_llm.rs:349-357,400-408` (with `with_delay`, a cancelled
  token makes `chat`/`chat_stream` return
  `Err(ReactError::Other("mock LLM call cancelled"))`); real transport:
  cancel → silent return, stream ends normally (`client.rs:251-256`;
  F-LLM-01-P3-01; same on the Anthropic inline parser, F-LLM-03); the only
  loop cancel test `test_run_stream_cancelled_mid_llm_call`
  (`stream_channel.rs:1972-2038`, using `with_delay(30s)`) asserts only "no
  FinalAnswer".
- Reachability: any test that cancels a delayed mock call — currently exactly
  the one test above.
- Expected invariant: mock cancel behaves like the real transport: the stream
  ends silently (no error), so tests exercise the production path
  (silent end → empty think → NoResponse, or the documented Cancelled
  terminal per F-RCT-03-P1-02's direction).
- Observed behavior: the mock makes cancellation loud — the only cancel-path
  test drives an error the real provider never sends; the production cancel
  path (silent end) is untested at loop level, which is precisely why
  F-RCT-03-P1-02 shipped.
- Impact: cancellation handling is validated against a fabricated behavior;
  the fix for F-RCT-03-P1-02 needs a silent-end fixture the current mock
  cannot produce without also exercising the fabricated error.
- Root cause: cancel modeling was added as a late "Phase 3" afterthought as an
  `Err`, never aligned with the transport's silent-end contract.
- Direction: on cancel, end the stream normally with zero chunks (silent) or
  emit the scripted terminal event; keep an error variant only if the typed
  `LlmError::Cancelled` from F-LLM-01-P3-01 lands and the transport actually
  returns it.
- Regression validation: loop test with `with_delay` + cancel asserting the
  turn ends per the production contract (empty-think NoResponse path) and a
  distinct silent-end fixture; the F-RCT-03-P1-02 regression test then
  becomes expressible.
- Validation reports: [V03](../validations/F-TST-01/V03-01.md),
  [V04-03](../validations/F-TST-01/V04-03.md)

### F-TST-01-P3-01: Duplicate `MockEmbedder` — byte-identical parallel implementation in `echo-state` test utilities; both constructors use `assert!` (panic API)

- Priority: P3
- Confidence: high
- Layer: framework (test infrastructure, layering)
- Evidence: `echo-agent/src/testing/mock_embedder.rs:1-62` and
  `echo-agent/echo-state/src/memory/mod.rs:54-92` — identical algorithms
  (byte accumulation `vec[i % self.dimension] += b as f32`, L2 normalization,
  zero-vector passthrough) and identical `assert!(dimension > 0)` guard;
  echo-state's copy is `#[cfg(test)] pub use test_utils::MockEmbedder`
  (`mod.rs:54-55`) and cannot use the root crate's mock (dependency
  direction: echo-state is a dependency of the root).
- Reachability: both copies are live in their respective test suites
  (embedding_store.rs, sqlite_store.rs use the echo-state copy; root tests
  use the root copy).
- Expected invariant: one mock implementation per concept (AGENTS.md
  no-parallel-semantics); no panic APIs (AGENTS.md panic rule) — both
  constructors panic on `dimension == 0`.
- Observed behavior: two identical implementations that can diverge silently;
  both can panic on invalid input instead of returning a typed error.
- Impact: maintenance duplication and divergence risk; panic-vs-Result
  inconsistency with the rest of the framework's mock surface.
- Root cause: the `Embedder` trait lives in `echo_core`, but no shared test
  helper was placed there; each consumer crate rolled its own copy.
- Direction: move the mock embedder next to the trait (e.g. an
  `echo_core` test-support module compiled under `#[cfg(any(test, ...))]` or
  a tiny shared dev-dependency crate), delete both copies, and replace
  `assert!` with a graceful fallback or `Result` per AGENTS.md.
- Regression validation: grep `MockEmbedder` returns one definition;
  `cargo test -p echo_state --lib` memory tests stay green.
- Validation reports: [V01](../validations/F-TST-01/V01-01.md)

### F-TST-01-P3-02: `docs/en/12-mock.md` references a nonexistent example (`demo16_testing`) and overstates error-injection coverage for the streaming contract

- Priority: P3
- Confidence: high
- Layer: framework (documentation)
- Evidence: `echo-agent/docs/en/12-mock.md:329-333` ("See:
  `examples/demo16_testing.rs`", `cargo run --example demo16_testing`) —
  `examples/` contains no demo16* file and `Cargo.toml` has no demo16 entry
  (V05); `docs/en/12-mock.md:35` ("Error injection: easily simulate network
  failures, rate limiting, service outages") — true only at the request
  level, no mid-stream error injection exists (V03); `README.md:1168` links
  the doc; `src/testing/mod.rs:13-18` "Scriptable" principle likewise holds
  only for request-level queues.
- Reachability: documentation consumers following the example command get a
  compile error; the coverage-map guidance overstates streaming
  scriptability.
- Expected invariant: documented commands run; documented claims match the
  mock's actual capability surface.
- Observed behavior: broken example reference; overstated error-injection
  claim for streaming.
- Impact: misleading developer onboarding; the doc suggests fault scenarios
  that cannot be scripted, hiding the P1-02 gap from readers.
- Root cause: the example was removed/renumbered without updating the doc;
  the doc predates the streaming loop refactor.
- Direction: point the example reference at a live example
  (demo04_subagent/demo12_resilience/demo61_agent_factory) or delete the
  section; qualify the error-injection claim as request-level; align the
  `src/testing/mod.rs` "Scriptable" principle wording.
- Regression validation: `cargo run --example <replacement>` succeeds; grep
  the doc for the corrected claim.
- Validation reports: [V05](../validations/F-TST-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---|---|
| V01 | Definition and duplicate search (mock inventory, `testing` feature, parallel impls, `then_tool_calls` reachability, wire fixtures) | yes | passed | [V01-01](../validations/F-TST-01/V01-01.md) |
| V02 | Registration and runtime reachability (cfg gating, CLI dev-dependency wiring, mock consumers) | yes | passed | [V02-01](../validations/F-TST-01/V02-01.md) |
| V03 | Mock-versus-provider contract matrix + scripted ordering/error fixture inventory | yes | passed | [V03-01](../validations/F-TST-01/V03-01.md) |
| V04 | `cargo check -p echo_agent --no-default-features --features testing --locked` (testing-only isolation) | yes | passed (exit 0) | [V04-01](../validations/F-TST-01/V04-01.md) |
| V04 | `cargo check -p echo_agent --no-default-features --locked` (module excluded) | yes | passed (exit 0) | [V04-02](../validations/F-TST-01/V04-02.md) |
| V04 | `cargo test -p echo_agent --lib --locked 'react::run::stream_channel'` (mock-driven loop suite green) | yes | passed (exit 0; 23 passed) | [V04-03](../validations/F-TST-01/V04-03.md) |
| V05 | Historical-document drift (12-mock.md, MASTER-PLAN acceptance, react_smoke header, module docs) | conditional | passed | [V05-01](../validations/F-TST-01/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `docs/en/12-mock.md:329-333` "See `examples/demo16_testing.rs`" | stale | no demo16* in `examples/` or `Cargo.toml` (V05) |
| `docs/en/12-mock.md:35` "Error injection: easily simulate network failures, rate limiting, service outages" | stale (streaming part) | request-level only; no mid-stream/malformed-chunk injection (V03) |
| `src/testing/mod.rs:16` "Scriptable: Precisely control return values" | stale (streaming part) | request-level queue only; chunk streams unscriptable (V03) |
| `tests/react_smoke.rs:9-12` full-loop mock tests deferred pending snapshot-level `LlmClient` field | stale | field shipped (`snapshot.rs:343`, wired at think.rs:99-103); deferred tests still absent (V02/V05) |
| `docs/MASTER-PLAN.md:151` cancel-during-tool / cancel-during-subagent scenario tests | regressed/partial | one hand-written-tool cancel test (stream_channel.rs:2041-2082); dispatch-level race only for subagent (executor.rs:2483); MockAgent cancel is a no-op (P2-02) |
| `echo-agent/README.md:1168` "Mock Testing" doc link | current (link) / stale (content) | link resolves; content carries the demo16 reference (V05) |

## Coverage And Uncertainty

- All conclusions are static plus two compile checks and one test run (V04);
  no dynamic run used a real provider to compare mock vs wire behavior — the
  provider wire facts are taken from the completed F-LLM-01/02/03 and F-RCT-03
  reports (their own external-doc verification), not re-verified here.
- P1-01's production impact is conditional on EKO actually running Anthropic-
  family streams (F-LLM-03-P1-02's confidence note: some DeepSeek-Anthropic
  gateways echo `input_tokens` into `message_delta` and would parse); the
  mock-fidelity fact (single-chunk usage shape, no final-chunk fixture) is
  unconditional.
- The `src/testing` module compiles under `#[cfg(test)]` even without the
  feature; the V04-02 no-default check verifies the feature-free library
  build, and the workspace test builds (all-features) additionally include
  the module — consistent with F-FEAT-01's feature-isolation conclusions,
  not re-audited here.
- `src/testing` files have no `#[cfg(test)]` unit tests of their own; mock
  behavior is pinned only by consumers (recorded, not a finding).
- The 4 local `#[cfg(test)]` doubles (classifier MockClient, executor
  CancellationAwareStreamAgent, team UsageAgent, workflow RecordingAgent)
  were inventoried, not deep-read; none is a parallel infrastructure.
- Whether the mock's `EmptyResponse` on queue exhaustion is "unrealistic"
  was not pursued (the loop's graceful-termination test at
  stream_channel.rs:1934 exercises a plausible edge).

## Handoff

- Downstream tasks may rely on: one shared mock module with clean feature
  isolation (V01/V02/V04); the mock-vs-provider contract matrix (V03) as the
  authoritative fidelity inventory; green mock-driven loop suite at the
  reviewed commits (V04-03); the finding set P1-01/P1-02/P2-01/P2-02/P2-03
  as the fidelity defects.
- F-TST-01-P1-01 extends F-LLM-03-P1-02's fix with a loop-level regression
  fixture; the chunk-sequence scripting direction in P1-01/P1-02 should land
  with the F-LLM-01-P1-01 malformed-chunk fixture work.
- Q-TST-01: use this report's matrix to classify loop-level tests as
  "restate mock behavior" vs "test real contracts"; the streaming/usage
  assertions in stream_channel.rs are mock-shape statements.
- Q-FLT-01: build streaming fault fixtures from the P1-02 direction (chunk
  sequences, mid-stream errors) once the mock supports them; until then the
  fault suite needs its own double.
- X-BND-01: record the `MockEmbedder` dedup decision (P3-01) and the
  MockTool/MockAgent extension scope as framework-level test-infrastructure
  work; no repository movement.
- Stale triggers: any change to `src/testing/*`, the `testing` feature
  declaration, `think.rs` usage capture, `ToolResult::error` classification,
  the transport cancel behavior, or the loop's chunk consumption invalidates
  the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): F-LLM-03
  (usage fixture alignment), F-RCT-03/F-RCT-04 (their regression tests become
  expressible), Q-FLT-01, Q-TST-01, X-BND-01.
