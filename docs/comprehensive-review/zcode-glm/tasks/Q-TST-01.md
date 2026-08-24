# Q-TST-01: Test suite credibility and coverage map

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929 (baseline 9b0e0fa + 1 post-baseline commit `3aa7929` "M1 test-credibility re-basing (mock 隐身衣 removal)"; see Coverage And Uncertainty)
> `echo-agent-cli` commit: b3b2e81 (== baseline)
> Worktree state: clean (both repos `git status --short` empty)

## Question

Which production invariants have meaningful tests, which tests only restate
implementations, and where do mocks hide integration failures?

**Answer:** The EKO store/executor layer (claims, terminal monotonicity,
atomic rollback, stale revisions), subagent recovery, compression invariants,
and the builtin tools carry genuinely meaningful invariant tests; the
framework streaming-channel suite became provider-contract-faithful in the
post-baseline M1 commit (two-chunk wire shape, usage-on-terminal-chunk,
call-order batch); but the **non-streaming ReAct loop, both the Anthropic and
OpenAI streaming response parse paths, and the EKO revisioned-task adapter
have zero tests**, and the mock still hides structured tool failure, subagent
intermediate-event ordering, and mid-stream cancellation. One compressor
"test" is print-only (false confidence).

## Scope

Primary source paths and behaviors inspected (read-only static analysis):

- Production-module-to-test map: `src/agent/react/{tests.rs, run/*}`,
  `src/agent/subagent/*`, `echo-tools/src/*`, `echo-state/src/{compression,memory}/*`,
  `echo-orchestration/src/tasks/*`, `echo-integration/src/{providers,mcp}/*`,
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/*`,
  `chat_driver.rs`, `agent_pool.rs`, `unified_memory.rs`, `plugin_runtime.rs`.
- Mock source: `echo-agent/src/testing/{mock_llm,mock_agent,mock_tool,mod}.rs`.
- Assertion/fixture quality sampling: 10 critical tests graded A/B/C (V02).
- Ignored/flaky/platform-gated inventory across both repos + frontend (V03).
- Mock-invisibility seam analysis cross-referenced with F-TST-01 (V04).

Total: ~1942 framework test attributes across 8 crates + CLI task_runtime
(230 across its files). Search coverage excludes `target/`, `.worktrees/`,
`examples/`, `benches/`.

## Out Of Scope

Deferred to named task IDs:

- The mock implementation's own trait-fidelity matrix — consumed from
  **F-TST-01** (re-verified only the seams M1 touched).
- The defects *behind* the coverage gaps (F-RCT-02-P1-01 empty-answer,
  F-LLM-03-P1-01/P1-02/P2-01 Anthropic parse defects, F-CMP-01 compressor
  defects) — canonical IDs, not re-filed here.
- Executing the full gate (fmt/clippy/test/feature matrix/frontend) — owned by
  **Q-FW-01** / **Q-CLI-01** / **Q-GUI-01** / **Q-WEB-01**. This task is
  static; suite greenness is carried from F-TST-01 V04 + the M1 commit's own
  pre-commit gate.
- Frontend store-level coverage depth — **A-FE-03** owns it (this task
  records only the store-test inventory).

## Inputs

- Repository documents read: root `AGENTS.md` (Rust coding constraints,
  cleanup policy, framework-vs-application layering, adapter-losslessness
  rule), `REPORTING.md`, both report templates, shared `README.md`,
  `TASKS.md` (Q-TST-01 card).
- Dependency task report read: **F-TST-01** (full) — the mock-vs-provider
  fidelity matrix and the five mock-seam findings are the baseline for V04.
- Dependency findings referenced (anchors re-verified, not full reports
  re-read): F-RCT-02-P1-01 (empty-answer swallow), F-RCT-04-P1-01
  (completion-order batch), F-LLM-03-P1-01/P1-02/P2-01 (Anthropic parse),
  F-CMP-01-P1-0* (compressor), A-FE-03-P3-04 (chatStore zero tests).
- Historical documents treated as hypotheses: `tests/react_smoke.rs` header
  (deferred full-loop tests), F-TST-01 streaming-shape findings, and the
  parallel **zcode-ds** Q-TST-01 report (used only as a cross-reference
  checklist; every claim re-verified independently against current code —
  divergences recorded explicitly).

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework test infrastructure (`src/testing/*` mocks, `react/tests.rs`, `stream_channel.rs`, `echo-orchestration` task tests, `echo-integration` provider tests) is generic framework capability. The P1 findings here concern framework coverage gaps. |
| EKO product policy | The CLI `task_runtime` tests (store/executor/worktree) and `revisioned_adapter.rs` are application-layer; the well-tested store/executor layer is the positive pole, the untested revisioned adapter is the application-side gap (P2-01). |
| Adapter boundary | `revisioned_adapter.rs` (EKO→framework revisioned task graph) has zero round-trip/field-level tests despite AGENTS.md "适配器必须保持薄且转换无损…转换必须有 round-trip/字段级测试" (P2-01). Provider streaming parse (Anthropic/OpenAI `convert_response`/`chat_stream`) is the provider adapter seam with zero response-side tests (P1-02). |
| Duplicate search | Terms (both repos): `MockLlmClient`, `MockAgent`, `FailingMockAgent`, `then_tool_calls`, `with_stream_script`, `StreamChunk`, `#[cfg(test)]` modules, frontend `*.test.ts(x)`. Result: no duplicate test infrastructure; the four framework mock types are defined exactly once in `src/testing/`; the CLI reuses them via `echo_agent::testing::` (chat_driver.rs:764, agent_pool.rs:1250, unified_memory.rs:228, plugin_runtime.rs:1364, tasks/service.rs:872). |
| Migration deletion | No deletion proposed. |

## Current Path

Verified at the reviewed commits (full evidence in V01–V04):

1. **Streaming loop (post-M1, faithful):** `stream_channel.rs` `run_core_loop`
   has 27 tests via `MockLlmClient::agent_with_mock_llm`. Since M1,
   `chat_stream` emits the real two-chunk wire shape — `Delta(content)` then
   `Terminal(finish_reason+usage)` (mock_llm.rs:506-534) — and
   `with_stream_script(Vec<StreamChunk>)` scripts arbitrary sequences
   including mid-stream `StreamChunk::Err` (mock_llm.rs:578-601).
   `think.rs:107-147` derives `usage_reported = last_usage.is_some()`; the new
   fixtures `stream_script_terminal_chunk_reports_usage` (true) and
   `stream_script_without_usage_reports_false` (false) certify the real
   semantics. (V01, V02 #3, V04)
2. **Non-streaming loop (untested):** `react_loop.rs` `run_react_loop`
   (:598) has **zero** tests; its caller `run/direct.rs` (:23, :42) routes
   `run_direct`/`run_chat_direct`. The spawned `run_core_loop` error is
   logged, not forwarded (react_loop.rs:711-727); if the channel closes
   without a terminal, `Ok(answer)` returns `Ok("")` (react_loop.rs:730-750).
   `react/tests.rs` (81 tests) uses `MockAgent` (23 hits) and **zero**
   `MockLlmClient`/`then_text`/`then_tool_call` — it never drives the real
   loop. `tests/react_smoke.rs:9-12` still carries the stale "deferred"
   header. (V01, V02 #1/#2, P1-01)
3. **Batch ordering (post-M1, fixed):** `phases/tools.rs` emits tool results
   in call order (M1 reordered from `FuturesUnordered` completion order).
   `concurrent_batch_results_follow_call_order` (stream_channel.rs:2250)
   asserts `["call_1","call_2"]` call order via `then_tool_calls`.
   `pipeline.rs:1635-1640` doc restricts the older
   `multiplexed_streams_preserve_identity_and_terminal_order` test to a
   per-stream execution fact (its `["call-b","call-a"]` completion-order
   assertion is now legitimate for single-tool stages, not a restatement of
   the batch bug). (V02 #4/#5, V04)
4. **Provider streaming parse (untested):** Anthropic 7 tests (anthropic.rs
   test module :1100) and OpenAI 4 tests (openai.rs :516) are all
   request/cache/attachment conversion; `convert_response` (anthropic :293),
   `chat_stream` (anthropic :421, openai :175), and `AnthropicStreamEvent`
   handling have zero test references. `client.rs` 3 tests are SSE line-parser
   helpers, not the `stream_post` (:182) loop with inter-chunk cancel/error.
   No `message_delta`/interleaved-block/SSE wire fixture exists in
   `echo-integration`. (V01, P1-02)
5. **EKO task runtime (well-tested):** `store.rs` 34 meaningful tests
   (illegal transition + atomic rollback :2300, terminal-typed-status :2394),
   `executor.rs` 46 driving the real loop via MockLlmClient, `worktree.rs` 25.
   `revisioned_adapter.rs` (388 lines) has **zero** tests. (V01, V02 #8, P2-01)
6. **Compression (mixed):** `invariants.rs` 13 meaningful invariant tests
   (orphaned-tool, preserve-system-prompt, idempotent). `mod.rs:2252
   test_sliding_window_compressor` is print-only (zero assertions). (V02 #6/#7, P3-03)
7. **Ignored/platform inventory:** 6 `#[ignore]` tests total, all documented
   (1 pinned-red Q-FLT-01 placeholder, 5 opt-in live/credential-gated); 0
   frontend skips; platform gates are legitimate production branches. (V03)

## Findings

### Q-TST-01-P1-01: The non-streaming ReAct loop has zero tests — `react/tests.rs`'s 81 tests never drive a real loop, so the `Ok("")` empty-answer/error-swallow class (F-RCT-02-P1-01) is invisible to the suite

- Priority: P1
- Confidence: high
- Layer: framework (test coverage)
- Evidence:
  - `echo-agent/src/agent/react/run/react_loop.rs:598` (`run_react_loop`) —
    zero `#[test]`/`#[tokio::test]` in the file (grep-confirmed).
  - `react_loop.rs:711-727` — `run_core_loop` is spawned; on `Err(e)` it logs
    `"Core loop error (already sent via channel)"` and continues.
  - `react_loop.rs:730-750` — the `while let Some(event) = rx.recv().await`
    collector returns `Ok(answer)` with `answer = String::new()` if the
    channel closes without a `FinalAnswer`/`Cancelled`/`Error` terminal.
  - `src/agent/react/tests.rs` — 81 tests; `MockAgent` 23 hits, `MockLlmClient`
    0 hits, `then_text`/`then_tool_call` 0 hits.
  - `run/direct.rs:23,42` — `run_direct`/`run_chat_direct` call
    `run_react_loop`; `tests/react_smoke.rs:9-12` still defers full-loop mock
    tests pending a snapshot `llm_client` field that has since shipped.
- Reachability: every non-streaming turn (EKO REPL/direct-answer paths,
  `run_direct`, scheduler `run` calls); every framework consumer of
  `Agent::chat` that does not go through the streaming wrapper.
- Expected invariant: the loop that returns the agent's answer text has
  loop-level tests driven by the real LLM contract, so an error-swallowing
  `Ok("")` or a lost terminal fails the suite.
- Observed behavior: zero tests execute `run_react_loop` with any LLM double;
  `react/tests.rs` covers reset/builder/accessor/registry and MockAgent-level
  orchestration only; the non-streaming path is compile-tested only.
- Impact: the known P1 defect class (F-RCT-02-P1-01: silent empty answer on
  core-loop error) ships through a green gate; any future non-streaming
  regression (empty answers, lost errors, missing terminals) is invisible.
- Root cause: full-loop mock tests were deferred pending the snapshot
  `llm_client` field (which shipped); the streaming wrapper absorbed mock-
  driven attention while the non-streaming entry stayed untested; the stale
  "deferred" header in `react_smoke.rs` was never revisited.
- Direction: add a `MockLlmClient`-driven test family for `run_react_loop`:
  (a) text-only turn returns the answer; (b) core-loop error → typed error,
  not `Ok("")` (regression test for F-RCT-02-P1-01); (c) max-iteration and
  tool-cycle paths. Delete the stale "deferred" claim in `react_smoke.rs:9-12`.
- Regression validation: `cargo test -p echo_agent --lib` with the new
  fixtures; the F-RCT-02-P1-01 regression test must fail before the fix.
- Validation reports: [V01-01](../validations/Q-TST-01/V01-01.md),
  [V02-01](../validations/Q-TST-01/V02-01.md),
  [V04-01](../validations/Q-TST-01/V04-01.md).

### Q-TST-01-P1-02: Both the Anthropic and OpenAI streaming response-parse paths have zero tests — request-side conversion is tested, but `convert_response`/`chat_stream`/SSE event handling are compile-tested only, so the F-LLM-03 adapter defects and any future streaming regression ship invisible

- Priority: P1
- Confidence: high
- Layer: adapter (provider seam test coverage)
- Evidence:
  - `echo-integration/src/providers/anthropic.rs:1100` test module — 7 tests,
    all request/cache/attachment conversion (`conversation_cache_breakpoints_…`,
    `metadata_user_id_…`, `cache_hints_…`, `pdf_attachment_…`,
    `text_attachment_…`, `binary_non_pdf_…`).
  - `anthropic.rs:293` `convert_response`, `:421` `chat_stream`,
    `AnthropicStreamEvent` / `MessageDelta` handling — zero test references
    (grep `AnthropicStreamEvent`/`message_delta` in test contexts: 0 hits).
  - `openai.rs:516` test module — 4 tests, all request part conversion
    (`text_and_image_parts_pass_through_unchanged`, `text_class_file_…`,
    `binary_file_becomes_placeholder`, `plain_text_message_…`); `stream_chat`
    (`:175`) untested.
  - `client.rs:356` test module — 3 tests are SSE line-parser helpers
    (`parse_data_without_space`, `parse_data_with_crlf_and_keepalive`,
    `parse_done_marker`); the `stream_post` (`:182`) loop with inter-chunk
    `is_cancelled()` poll and mid-stream error is untested.
  - No `data: …`/`message_delta`/interleaved-block SSE wire fixture exists
    anywhere in `echo-integration` (grep `data: \`[\|data: done\|message_delta`: 0).
- Reachability: every streaming LLM turn (EKO main path for both providers);
  every future provider adapter regression run — the gate cannot fail on
  these seams.
- Expected invariant: the adapter's streaming contract (interleaved tool/text
  blocks, final usage chunk, malformed-event drop, mid-stream cancel) has
  fixture-level tests with literal wire strings per the F-LLM-03 regression
  validations.
- Observed behavior: request-side conversion is tested for both providers;
  the entire response/stream side is compile-tested only; the F-LLM-03-
  prescribed fixtures do not exist.
- Impact: the F-LLM-03 P1/P2-class Anthropic parse defects (accumulator keyed
  by length vs stream index, `AnthropicUsage` dropping `message_delta.usage`,
  silent unparseable-event drop) shipped through a green gate; future
  streaming regressions on either provider path are structurally invisible —
  the most defect-dense untested seam in the framework. (Broader than the
  Anthropic-only zcode-ds P1-03: OpenAI's `stream_chat` is equally untested.)
- Root cause: `convert_response`/stream parsing was written inline with
  request conversion and tests were added for the cache-plan features only;
  no wire fixtures were ever created for the response side.
- Direction: add wire-fixture tests with literal SSE/JSON strings: (a)
  Anthropic `message_delta {"usage":{"output_tokens":15}}` → final chunk with
  usage + finish_reason; (b) `[text, tool_use]` and `[tool_use, text, tool_use]`
  → correct tool-call assembly; (c) wrong-typed event line → logged/counted
  drop; (d) an OpenAI `[DONE]`-terminated multi-chunk stream. Place in the
  provider test modules with literal wire strings.
- Regression validation: the fixtures above must fail before the F-LLM-03
  fixes and pass after.
- Validation reports: [V01-01](../validations/Q-TST-01/V01-01.md),
  [V02-01](../validations/Q-TST-01/V02-01.md).

### Q-TST-01-P2-01: `revisioned_adapter.rs` — the EKO-to-framework revisioned task-graph adapter (388 lines) — has zero tests and no round-trip/field-level conversion test, violating the AGENTS.md adapter-losslessness rule

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs`
    — 388 lines, zero `#[cfg(test)]`/`#[test]`/`#[tokio::test]` hits.
  - Header doc "Thin EKO adapters for the framework-owned task revision
    service"; implements `RevisionedTaskStore`/`TaskRevisionService`
    conversion (`EkoRevisionedTaskStore::load`, etc.) over `TaskRuntimeStore`.
  - AGENTS.md "适配器必须保持薄且转换无损…转换必须有 round-trip/字段级测试".
  - Adjacent `store.rs` has 34 meaningful tests but none drives this adapter.
- Reachability: every `task_create`/`task_update`/`task_list` call in EKO
  (the revisioned-graph path); a conversion bug (dropped field, wrong
  revision, lost metadata) is invisible to the entire suite.
- Expected invariant: the adapter has round-trip and field-level tests
  (EKO PlanTask/TaskRunStatus → framework TaskSpec/TaskExecution/TaskStatus →
  back) as mandated by AGENTS.md.
- Observed behavior: zero tests; the framework side has its own round-trip
  tests (`echo-orchestration/src/tasks/revisioned.rs`, 3 tests) but the EKO
  conversion boundary is compile-tested only.
- Impact: the claim/revision semantics verified at the store level
  (A-TSK-04) are unverified at the boundary where EKO facts become framework
  facts; any field drift (status mapping, revision propagation, metadata)
  ships silently through a green gate.
- Root cause: the adapter was added during the revisioned-graph migration
  with tests concentrated in the store and framework layers, not at the
  conversion boundary.
- Direction: add a `revisioned_adapter.rs` test module with a field-by-field
  round-trip fixture (create/update/list a plan through the adapter, reload,
  assert TaskSpec/Execution/Status and EkoPlanMetadata survive); include a
  revision-bump and a stale-write rejection case.
- Regression validation: the round-trip fixture fails if any mapped field is
  dropped or reordered; `cargo test -p echo-agent-app-core` stays green.
- Validation reports: [V01-01](../validations/Q-TST-01/V01-01.md).

### Q-TST-01-P2-02: `MockTool` is still text-only — M1 added `with_delay` but no `ToolFailure`/`bytes`/`data`/`truncated`/pagination builder, so structured-failure routing and bounded-output/artifact-spill are untested via the mock (F-TST-01-P2-03 remains open)

- Priority: P2
- Confidence: high
- Layer: framework (testing)
- Evidence:
  - `echo-agent/src/testing/mock_tool.rs` — builders are `with_response`,
    `with_responses`, `with_failure(msg)`, `with_description`,
    `with_parameters`, and (M1) `with_delay`. The three construction sites
    (:139-172) produce only `ToolResult{kind:Text/Error, failure:None,
    bytes:None, data:None, truncated:false}`.
  - grep `ToolFailure`/`with_bytes`/`with_data`/`with_truncated`/`PageRequest`
    in `src/testing/`: 0 hits.
  - `echo-core/src/tools/mod.rs:288-315` `ToolResult` carries `kind` (5),
    `failure: Option<ToolFailure>`, `bytes`, `data`, `truncated`, `metadata`;
    `:78-100` `ToolFailure` carries `category` (7) / `recovery` (5).
- Reachability: every mock-driven tool test (`stream_channel.rs` ~15 sites,
  `react/tests.rs` ~20, plus CLI sites). Production tool dispatch
  (`pipeline.rs:1395`) routes through `execute_with_context`, which for
  `MockTool` delegates to `execute`.
- Expected invariant: the F-EXT-01 structured-failure taxonomy
  (`category → recovery`) and bounded-output artifact path are exercisable
  through the mock.
- Observed behavior: `MockTool::with_failure("msg")` yields a text-only
  error; the agent's tool-failure handling never observes
  `ToolFailure.category`/`.recovery`/`.side_effect` from a mock-driven test.
- Impact: a regression in structured-failure routing (e.g.
  `PartialSideEffect` not routed to verify-then-retry) or in artifact spill
  (`truncated=true`) is invisible to every mock-driven tool test. Blast
  radius is contained because builtin-tool tests (F-EXT-02) cover some of
  this with real tool instances, but the generic contract is text-only-tested.
- Root cause: `MockTool` predates the F-EXT-01 extensions; M1 added
  `with_delay` for batch-order control but did not add the structured-failure
  builders.
- Direction: add `with_failure_structured(ToolFailure)`, `with_bytes(...)`,
  `with_data(...)`, `with_truncated(ArtifactRef)`, and a paginated variant
  (each appends a pre-built `ToolResult`). Pure addition under no-compat.
- Regression validation: a test scripting
  `ToolFailure{category:PartialSideEffect,…}` asserting verify-then-retry
  routing; a test scripting `truncated=true` asserting the consumer sees the
  artifact ref.
- Validation reports: [V04-01](../validations/Q-TST-01/V04-01.md).

### Q-TST-01-P3-01: `MockAgent` still emits only `FinalAnswer` and `FailingMockAgent` still returns one error variant — `mock_agent.rs` is unchanged by M1, so orchestration tests cannot assert on subagent intermediate-event ordering or diverse failure modes (F-TST-01-P3-01/P3-02 remain open)

- Priority: P3
- Confidence: high
- Layer: framework (testing)
- Evidence:
  - `git log 9b0e0fa..HEAD -- src/testing/mock_agent.rs` is empty (M1 did not
    touch it).
  - `mock_agent.rs:243,269,323` — every streaming override emits
    `stream::once(async move { Ok(AgentEvent::FinalAnswer(answer)) })`.
  - `mock_agent.rs:394-395,406` — `FailingMockAgent::execute` always returns
    `AgentError::InitializationFailed`.
  - `echo-core/src/agent/mod.rs:140-239` — `AgentEvent` has ~15 variants
    (`ThinkStart`, `Token`, `LlmUsage`, `ToolBatchStart`, `ToolCall`,
    `ToolResult`, `MemoryRecalled`, `Cancelled`, `Error`, …).
- Reachability: every orchestration test that streams from a `MockAgent`
  (e.g. subagent dispatch tests `src/agent/subagent/executor.rs:2117-2520`).
- Expected invariant: a mock used for orchestration fault-tolerance
  (`FailingMockAgent` docstring :352-353) and event ordering should let the
  test choose the event sequence / failure mode.
- Observed behavior: orchestration tests see `[FinalAnswer]` and cannot
  assert on intermediate subagent events; all failover tests exercise only
  the `InitializationFailed` arm.
- Impact: low today (recovery routing is uniform across error variants;
  orchestration tests assert on finals/counts not intermediate events).
  Preventive: any future feature routing subagent intermediate events or
  differentiating recovery by error category has no scaffold.
- Root cause: `MockAgent` was designed around `execute`; streaming overrides
  were added to satisfy the trait surface with minimal event shape and never
  extended.
- Direction: add `MockAgent::with_stream_events(Vec<AgentEvent>)` (and an
  `Err`-terminated variant) and `FailingMockAgent::with_error(ReactError)`.
  Pure additions.
- Regression validation: a test scripting
  `[ThinkStart, Token("…"), FinalAnswer("done")]` asserting the consumer
  observes all three in order.
- Validation reports: [V04-01](../validations/Q-TST-01/V04-01.md).

### Q-TST-01-P3-02: Mid-stream cancellation is unmodelled and `MockTool` ignores `ToolContext` — the cancel-half of F-TST-01-P2-02 and the F-TST-01 handoff #4 remain open

- Priority: P3
- Confidence: high
- Layer: framework (testing)
- Evidence:
  - `echo-agent/src/testing/mock_llm.rs:496-504` — `with_delay` cancels only
    before the stream starts (`select!` on `token.cancelled()` vs `sleep`);
    no `with_cancel_after_chunks(n)` builder. The real transport polls
    `is_cancelled()` between SSE chunks (`client.rs:252-254` per F-LLM-01).
  - `echo-agent/src/testing/mock_tool.rs:139-172` — `Tool` impl overrides
    `execute` only; inherits the default `execute_with_context`
    (`echo-core/src/tools/mod.rs:777-783`) that delegates to `execute` and
    drops `ToolContext`.
- Reachability: every mock-driven streaming/tool test.
- Expected invariant: mid-stream cancellation (cancel arriving after the LLM
  has emitted tokens) and context-aware (working-dir, cancel-via-context)
  tool output are exercisable through the mock.
- Observed behavior: cancellation after the first chunk is indistinguishable
  from pre-stream cancel; a test asserting working-dir-aware tool output
  silently gets the default context.
- Impact: low. The chunk-loop cancel path and the `ToolContext`-aware path
  are not exercised, but no audited test depends on them. Preventive.
- Root cause: M1 added mid-stream *error* (`StreamChunk::Err`) but not
  mid-stream *cancel*; `MockTool` predates context-aware tooling.
- Direction: once multi-chunk scripting exists, add
  `with_cancel_after_chunks(n)`; add a `MockTool::with_context_handler`
  override or a context-aware stub for `execute_with_context`.
- Regression validation: a test emitting chunks slowly and cancelling after
  the first, asserting the stream terminates without waiting for the rest.
- Validation reports: [V04-01](../validations/Q-TST-01/V04-01.md).

### Q-TST-01-P3-03: `test_sliding_window_compressor` is a print-only fixture (zero assertions) — it passes regardless of compressor behavior, giving false confidence (the real sliding-window invariants ARE covered by `invariants.rs`, so net risk is low)

- Priority: P3
- Confidence: high
- Layer: framework (test quality)
- Evidence:
  - `echo-state/src/compression/mod.rs:2252-2277` — `test_sliding_window_compressor`:
    pushes 6 turns, `prepare`, `println!`s before/after counts and each
    message, ends `Ok(())`. Zero `assert!`/`assert_eq!`.
  - `echo-state/src/compression/invariants.rs:114-161` —
    `invariant_tool_pair_integrity_sliding_window` asserts no orphaned tool
    results (grade-A, V02 #7); plus 12 more invariant tests
    (`invariant_last_user_request_preserved`,
    `invariant_system_prompt_preserved`, `invariant_token_target_met`,
    `invariant_compression_idempotent`, …).
- Reachability: the mandatory `cargo test --workspace` runs the print-only
  test — it always passes regardless of compressor behavior.
- Expected invariant: a test named for a compressor asserts the compressor's
  contract, not merely that it runs.
- Observed behavior: the test would pass even if the compressor deleted every
  message or exceeded the token budget.
- Impact: low. Unlike the zcode-ds P2-01 framing (which grouped this with
  untested-summary/toy fixtures), the compressor subsystem IS covered by 13
  meaningful `invariants.rs` tests and `levels.rs` (10); the print-only test
  is a leftover demonstration script, not the sole coverage. The defect is
  "misleading dead test" (false confidence for a reader), not "no coverage".
- Root cause: the test was written as a demonstration/print script during
  original implementation and never converted to assertions.
- Direction: either delete `test_sliding_window_compressor` (the invariants
  module covers the contract) or convert it to assertions (after-count ≤
  window, messages preserved in order, protected markers survive). Under
  AGENTS.md no-compat, deletion is acceptable given the existing coverage.
- Regression validation: `cargo test -p echo_state --lib compression` stays
  green after deletion.
- Validation reports: [V02-01](../validations/Q-TST-01/V02-01.md).

### Positive confirmation: the M1 commit lifted the three highest-impact mock-invisibility seams

- The post-baseline commit `3aa7929` "fix(tests): M1 test-credibility re-basing
  (mock 隐身衣 removal)" resolved F-TST-01-P2-01 (single-chunk streaming →
  real two-chunk `Delta`+`Terminal` wire shape + `with_stream_script`),
  the usage-on-content-chunk invisibility cloak (two new `usage_reported`
  fixtures), F-RCT-04-P1-01 / zcode-ds-P1-02 (completion-order batch →
  call-order fix + `concurrent_batch_results_follow_call_order`), and the
  error-half of F-TST-01-P2-02 (`StreamChunk::Err`). See [V04-01](../validations/Q-TST-01/V04-01.md).

### Positive confirmation: the ignored/flaky/platform-gated inventory is clean

- 6 `#[ignore]` tests, all with explicit reason strings (1 pinned-red
  Q-FLT-01 placeholder, 5 opt-in live/credential-gated); 0 frontend skips;
  platform gates are legitimate production branches. No silently-skipped or
  hidden-flaky test inflates the pass rate. See [V03-01](../validations/Q-TST-01/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Production-module-to-test map (react/tools/memory/tasks/mcp/providers; driver classification; duplicate search) | yes | passed | [V01-01](../validations/Q-TST-01/V01-01.md) |
| V02 | Assertion/fixture quality sampling (10 critical tests, A/B/C grading) | yes | passed | [V02-01](../validations/Q-TST-01/V02-01.md) |
| V03 | Ignored/flaky/platform-gated inventory (both repos + frontend) | yes | passed | [V03-01](../validations/Q-TST-01/V03-01.md) |
| V04 | Mock-invisibility seam analysis (F-TST-01 cross-ref + negative controls) | yes | passed | [V04-01](../validations/Q-TST-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | Q-TST-01 is a current-state audit; the historical artefacts revalidated are the `react_smoke.rs` "deferred" header (stale) and F-TST-01 streaming-shape findings (superseded by M1 at the reviewed commit) — both classified under Historical Claim Status. |

All required validations executed; no command exit codes apply (read-only
static review; suite greenness carried from F-TST-01 V04 + the M1 commit's
own pre-commit gate, not re-executed to conserve build budget — the gate
tasks Q-FW-01/Q-CLI-01/Q-WEB-01 own execution).

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `tests/react_smoke.rs:9-12` — full-loop mock tests deferred pending snapshot `llm_client` field | stale | field shipped; deferred tests still absent; non-streaming loop untested (P1-01) |
| F-TST-01-P2-01 — single-chunk streaming hides multi-chunk/fragmentation classes | fixed (at reviewed commit) | M1 added two-chunk wire shape + `with_stream_script`; V04 RESOLVED |
| F-TST-01-P2-02 (error half) — mid-stream error unmodelled | fixed (at reviewed commit) | M1 added `StreamChunk::Err`; the cancel-half remains open (P3-02) |
| F-TST-01-P2-03 — `MockTool` text-only | current | M1 added `with_delay` only; structured-failure/bytes/data/truncated still absent (P2-02) |
| F-TST-01-P3-01/P3-02 — `FailingMockAgent` one variant / `MockAgent` single-FinalAnswer | current | `mock_agent.rs` unchanged by M1 (P3-01) |
| F-RCT-04-P1-01 / zcode-ds-P1-02 — completion-order batch / pipeline restates wrong contract | fixed (at reviewed commit) | production reordered to call order + call-order fixture; pipeline test scope clarified (V04) |
| zcode-ds-P1-01 — non-streaming loop zero tests | current | re-verified (P1-01) |
| zcode-ds-P1-03 — Anthropic streaming parse zero tests | current + broadened | re-verified; OpenAI `stream_chat` equally untested (P1-02) |
| zcode-ds-P2-02 — revisioned_adapter zero tests | current | re-verified (P2-01) |
| zcode-ds-P2-01 — compressor tests zero-assertion/toy | partially divergent | the specific `test_sliding_window_compressor` is print-only (P3-03), but `invariants.rs` has 13 meaningful sliding-window/levels tests — the subsystem is NOT as weak as the zcode-ds framing implied |

## Coverage And Uncertainty

- **Reviewed commit divergence (important).** The task header pins
  `echo-agent: 9b0e0fa`; the working tree is at `3aa7929` (baseline + exactly
  one post-baseline commit, `3aa7929`). That commit is directly on-topic
  ("mock 隐身衣 removal"). Per REPORTING.md ("Each report records the actual
  reviewed commits"; "If relevant code changes after a report, the phase
  synthesizer marks it stale"), this report records `3aa7929` and classifies
  the F-TST-01 streaming-shape findings as *fixed at the reviewed commit*.
  Reviewing at the pinned baseline `9b0e0fa` would reproduce findings already
  resolved and mislead downstream synthesis. `echo-agent-cli` is at `b3b2e81`
  (no divergence).
- All conclusions are static; no test was executed in this task. V02 grade-A
  gradings and V04 negative controls are read-only judgments anchored to full
  reads of the tested code, not executed mutations.
- The four framework mock types and the CLI application-layer test doubles
  were inventoried by grep, not per-site traced; the usage pattern is uniform
  (CLI reuses `echo_agent::testing::MockLlmClient`), so conclusions do not
  depend on a per-site trace.
- `echo-orchestration/src/tasks/{dag,events,manager,store,time}.rs` (0 inline
  tests each) are recorded as absence; their behavior, if covered, is covered
  by sibling-module tests — not exhaustively traced.
- Frontend store-test inventory is recorded at the directory level (A-FE-03
  owns depth); `chatStore.ts` core reducer remains untested (A-FE-03-P3-04),
  consistent with zcode-ds.

## Handoff

Conclusions downstream tasks may rely on:

1. **The mock invisibility cloak is substantially lifted.** The streaming-
   shape, usage-accounting, and batch-ordering contracts (the three
   highest-impact F-TST-01 seams) are now mock-faithful at `3aa7929`.
   `Q-FLT-01`/`Q-E2E-01` can rely on the streaming-channel suite to certify
   the real provider wire shape. Five lower-impact seams remain (P2-02,
   P3-01, P3-02) — fault-injection tasks should build their own fixtures for
   structured tool failure, subagent event ordering, and mid-stream cancel.
2. **Three production seams are compile-tested only and must not be trusted
   as regression nets:** the non-streaming `run_react_loop` (P1-01), the
   Anthropic AND OpenAI streaming response-parse paths (P1-02), and
   `revisioned_adapter.rs` (P2-01). Any fix in these areas must land its own
   failing-then-passing fixture.
3. **The well-tested poles are credible:** EKO store/executor (claims,
   terminal monotonicity, atomic rollback), subagent recovery, compression
   `invariants.rs`, builtin tools, and the post-M1 streaming channel assert
   genuine invariants that catch regressions (V02 grade-A).
4. **The ignored/flaky/platform inventory is clean** (V03) — no hidden skips
   inflate the pass rate.

Reports downstream tasks must read: this report + V01-01..V04-01; F-TST-01
(mock fidelity baseline, partially superseded at the reviewed commit);
F-LLM-03 (Anthropic parse defects behind P1-02); F-RCT-02 (empty-answer
defect behind P1-01); A-FE-03 (frontend store coverage behind the store
inventory).

Conditions that make this report stale:

- Any change to `src/testing/*` (the P2-02/P3-01/P3-02 seams), `react_loop.rs`
  (P1-01), `react/tests.rs` driver mix, `echo-integration/src/providers/*`
  streaming-parse tests (P1-02), `revisioned_adapter.rs` (P2-01), or
  `compression/mod.rs:2252` (P3-03).
- A second post-baseline commit on `echo-agent` (moves the reviewed commit
  past `3aa7929`).

Follow-up task IDs (no fixes implemented in this review):

- F-RCT-02 — non-streaming loop regression tests (P1-01).
- F-LLM-03 — Anthropic/OpenAI streaming wire fixtures (P1-02).
- A-TSK-04 / A-TSK-01 — revisioned adapter round-trip tests (P2-01).
- A mock-fidelity enhancement task — land the remaining builders
  (`MockTool` structured-failure P2-02; `MockAgent::with_stream_events` /
  `FailingMockAgent::with_error` P3-01; mid-stream cancel P3-02).
- S-QA-01 — synthesis consumes this coverage map.
