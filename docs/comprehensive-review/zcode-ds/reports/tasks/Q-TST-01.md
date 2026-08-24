# Q-TST-01: Test suite credibility and coverage map

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline 9b0e0fa)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline b3b2e81)
> Worktree state: clean (both repositories)

## Question

Which production invariants have meaningful tests, which tests only restate
implementations, and where do mocks hide integration failures?

**Answer:** The EKO store/executor layer (claims, terminal monotonicity,
revisions) and the frontend subagent/task stores have genuinely meaningful
tests; the framework loop-level suite certifies mock-shape behavior on a wire
no provider produces (F-TST-01-P1-01/02); the non-streaming ReAct loop, the
tool batch phase, the Anthropic streaming parse path, the EKO revisioned
adapter, and 6 frontend stores have zero tests; one pipeline test actively
pins the wrong (completion-order) contract; two compression tests are
zero-assertion/toy-scaffolding. Mocks hide integration failures at three
specific seams: usage accounting (mock emits usage on the content chunk),
streaming (single chunk), and agent-event vocabulary/cancel (single
FinalAnswer, cancel no-op).

## Scope

- Test inventory and production-module-to-test map for both repositories:
  ReAct loop (`react_loop.rs`, `stream_channel.rs`, `phases/*`, `pipeline.rs`,
  `stream_macros.rs`, `processor.rs`, `retry.rs`, `react/tests.rs`,
  `snapshot.rs`), LLM adapters (`echo-integration/src/providers/*`),
  compression (`echo-state/src/compression/**`), framework task runtime
  (`echo-orchestration/src/tasks/*`), subagent executor, EKO task runtime
  (`echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/*`),
  `chat_driver.rs`, `agent_pool.rs`, and the frontend stores/hooks
  (`web-frontend/src/stores/*`, `src/hooks/*.test.ts`).
- Assertion/fixture quality sampling: 21 tests read in full (V02).
- `#[ignore]` / platform-gated / skipped-test inventory (V03).
- Negative-control analysis on usage_reported, terminal monotonicity, claim
  protocol, batch ordering (V04).
- Cross-reference with filed mock-fidelity findings (V05).

## Out Of Scope

- The mock implementation itself (`src/testing/*`) — consumed from F-TST-01,
  not reread; the two load-bearing anchors were re-verified only.
- Defects behind the coverage gaps (F-RCT-02-P1-01, F-RCT-04-P1-01/02,
  F-LLM-03-P1-01/02, A-SRF-03-P1-02, A-SRF-03-P2-01) — canonical IDs, not
  re-filed.
- Executing the full test suite (Q-FW-01/Q-CLI-01/Q-GUI-01/Q-WEB-01 own the
  gates; this task is static with dependency-carried greenness).
- Doc drift of test documentation (F-TST-01-P3-02) and the `demo16_testing`
  example reference.

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (Q-TST-01 card only), `zcode-ds/README.md`, report templates.
- Dependency task reports read (5, at the budget): F-TST-01 (full),
  F-RCT-04 (full), F-LLM-03 (findings sections), A-SRF-03 (findings +
  cross-verified sections), Q-GUI-01 (full); plus the A-FE-03-P3-04 finding
  section for the canonical chatStore zero-test ID.
- Historical documents treated as hypotheses: `tests/react_smoke.rs` header
  (deferred full-loop tests), `tests/cache_user_id_test.rs` manual-verification
  claims, F-TST-01/F-RCT-04/A-SRF-03/Q-GUI-01 validation claims (re-verified
  anchors only).

## Layering Decision

- Generic mechanism (framework): the loop-level suites, the mocks, and the
  pipeline/compression tests are framework test infrastructure; all
  Q-TST-01-P1 findings in this layer concern framework coverage/fixture
  fidelity.
- EKO product policy (application): frontend store tests and the EKO task
  runtime tests; the well-tested store/executor layer is the positive pole of
  the map.
- Adapter boundary: `revisioned_adapter.rs` (EKO->framework revisioned task
  store) has zero round-trip/field-level tests despite AGENTS.md's adapter
  losslessness requirement (Q-TST-01-P2-02); the Anthropic streaming parse
  path is the provider adapter seam with zero tests (Q-TST-01-P1-03).
- Duplicate-search terms (both repositories, V01): `#[cfg(test)]` files,
  `MockLlmClient`/`MockAgent`/`then_tool_call(s)` usages, `#[ignore]`,
  `cfg(target_os)`/`cfg(unix)` in tests, frontend `*.test.ts(x)` files,
  per-store test mapping. Results: no duplicate test infrastructure; the only
  duplicate doubles are the 4 local `#[cfg(test)]` doubles already
  inventoried in F-TST-01 (not re-audited).

## Current Path

Verified at the reviewed commits (full evidence in V01/V02/V04):

1. Streaming loop: `stream_channel.rs` `run_core_loop` has 23 tests driven by
   `MockLlmClient` (`agent_with_mock_llm`), whose `chat_stream` yields exactly
   one `stream::once` chunk carrying content + finish_reason + usage
   (mock_llm.rs:413-435). `think.rs:112-113,147` derives
   `usage_reported = last_usage.is_some()` from any chunk. The suite therefore
   certifies usage semantics on a wire shape no real provider produces.
2. Non-streaming loop: `react_loop.rs` `run_react_loop` (:598, :711-727
   error-swallow site) has zero inline tests; `react/tests.rs` (81 tests) uses
   `MockAgent` (23 hits) and zero `MockLlmClient`/`then_text`/`then_tool_call`
   — it never drives the real loop. `direct.rs` routes `run_direct`/
   `run_chat_direct` through `run_react_loop`.
3. Tool batch: `phases/tools.rs` has zero tests; `then_tool_calls` (the
   multi-tool scripting API, mock_llm.rs:233) has zero usages repository-wide
   (F-RCT-04-P2-01 re-confirmed); concurrent results are pushed in
   `FuturesUnordered` completion order (tools.rs:215-227). The only ordering
   test, pipeline.rs:1634-1720, asserts `terminal_ids == ["call-b","call-a"]`
   — completion order, the F-RCT-04-P1-01 violation.
4. Anthropic adapter: 7 tests cover request/cache conversion only; the inline
   SSE parse path (anthropic.rs:410-617) has zero tests; no `message_delta`
   or interleaved-block fixture exists anywhere in `echo-integration`.
5. Compression: `mod.rs` 28 tests (one print-only), `levels.rs` 10,
   `horizon.rs` 12, `invariants.rs` 13, `verifier.rs` 3, `hybrid.rs` 2,
   `sliding_window.rs` 0, `summary.rs` 1 (toy Mutex test).
6. EKO task runtime: `store.rs` 34 meaningful tests (stale claims :3100,
   illegal transitions :2300, stale revisions :3039), `executor.rs` 46 tests
   driving the real loop through `MockLlmClient` (incl. `then_tool_call` at
   :5915), `event_rebuild.rs` 3 crash-replay tests, `revisioned_adapter.rs`
   0 tests (388 lines).
7. Frontend: 27 test files; `chatStore.ts` (527 lines) has zero direct tests
   (A-FE-03-P3-04); `browserStore.ts` (351), `subagentDetailStore`,
   `workspaceStore`, `uiStore`, `toastStore`, `authStore` have no test files;
   `subagentRunStore`/`taskRuntimeStore` terminal-monotonicity tests are
   meaningful.
8. Ignored/platform-gated inventory is clean: 5 documented opt-in live smoke
   tests, standard unix/windows splits, zero frontend skips (V03).

## Findings

### Q-TST-01-P1-01: The non-streaming ReAct loop has zero tests — `react/tests.rs`'s 81 tests never drive a real loop, so the `Ok("")` error-swallow class (F-RCT-02-P1-01) is invisible to the suite and any regression of the non-streaming core path ships green

- Priority: P1
- Confidence: high
- Layer: framework (test coverage)
- Evidence: `echo-agent/src/agent/react/run/react_loop.rs:598` (`run_react_loop`,
  the non-streaming loop) — zero `#[test]`/`#[tokio::test]` in the file;
  `react_loop.rs:711-727` (core-loop error logged instead of forwarded —
  F-RCT-02-P1-01); `src/agent/react/tests.rs` — 81 tests, 23 use `MockAgent`,
  zero use `MockLlmClient`/`then_text`/`then_tool_call` (grep at V01);
  callers of `run_react_loop`: `run/direct.rs` (:23, :42 `run_direct`/
  `run_chat_direct`), `stream_channel.rs`, `capabilities.rs`, `run/mod.rs`;
  `tests/react_smoke.rs:9-12` header documents the full-loop mock tests as
  deferred ("tracked separately as a larger refactor") — still absent
  (F-TST-01 V05).
- Reachability: every non-streaming turn (EKO REPL/direct-answer paths,
  `run_direct`, scheduler `run` calls), every framework consumer of
  `Agent::chat`.
- Expected invariant: the loop that returns the agent's answer text has
  loop-level tests driven by the real LLM contract (mocked), so an
  error-swallowing return (`Ok("")`) or a lost terminal fails the suite.
- Observed behavior: zero tests execute `run_react_loop` with any LLM double;
  `react/tests.rs` covers builder/accessor/registry and MockAgent-level
  orchestration only; the non-streaming path is compile-tested only.
- Impact: the shipped P1 (F-RCT-02-P1-01: silent empty answer on core-loop
  error) is exactly the class this gap allows; every future non-streaming
  regression (empty answers, lost errors, missing terminals) passes the gate
  green (V04 negative control: no test references the loop's error path).
- Root cause: the full-loop mock tests were deferred pending the snapshot
  `llm_client` field (which shipped, snapshot.rs:343) and never re-landed;
  the streaming wrapper (`run_stream_channel`) absorbed the mock-driven test
  attention while the non-streaming entry stayed untested.
- Direction: add a `MockLlmClient`-driven test family for `run_react_loop`:
  (a) text-only turn returns the answer; (b) core-loop error → typed error,
  not `Ok("")` (regression test for F-RCT-02-P1-01); (c) max-iteration and
  tool-cycle paths; delete the stale "deferred" claim in
  `tests/react_smoke.rs` when the tests land.
- Regression validation: `cargo test -p echo_agent --lib non_stream` fixtures
  above; the F-RCT-02-P1-01 regression test must fail before the fix.
- Validation reports: [V01-01](../validations/Q-TST-01/V01-01.md), [V02-01](../validations/Q-TST-01/V02-01.md), [V04-01](../validations/Q-TST-01/V04-01.md)

### Q-TST-01-P1-02: `multiplexed_streams_preserve_identity_and_terminal_order` enshrines completion-order terminals `["call-b","call-a"]` — the suite certifies the F-RCT-04-P1-01 provider-contract violation and the correct fix cannot land without changing this test

- Priority: P1
- Confidence: high
- Layer: framework (test that restates a wrong implementation)
- Evidence: `echo-agent/src/agent/react/run/pipeline.rs:1634-1720` — the test
  spawns two `ExecuteStage` contexts (call-a 60 ms delay, call-b 10 ms),
  asserts stream-chunk interleaving `[a-1, b-1, b-2, a-2]` and
  `terminal_ids == ["call-b","call-a"]` (completion order); production
  `phases/tools.rs:215-227` pushes `Message::tool_result` as each
  `FuturesUnordered` entry completes (completion order) while the assistant
  message carries calls in stream-index order (processor.rs:141-147) — the
  F-RCT-04-P1-01 violation (strict providers reject the next request 400).
- Reachability: every `cargo test -p echo_agent` run (the test is part of the
  mandatory gate); the assertion is the only ordering contract in the suite.
- Expected invariant: the suite's ordering test matches the provider-legal
  contract (tool results in the same order as the calls they answer), so the
  P1-01 fix is accepted by the gate.
- Observed behavior: the test pins completion order as the expected contract
  and would FAIL if production were fixed to call order (V04 negative
  control 4); the actual concurrent batch path itself has no test
  (`then_tool_calls` zero usages, F-RCT-04-P2-01).
- Impact: the regression net actively blocks the F-RCT-04-P1-01 fix (its own
  direction notes the test "must be extended or its assertion reconciled");
  the gate currently certifies a contract that breaks real provider calls —
  a test that restates (and defends) the bug.
- Root cause: the test was written against the execution layer's natural
  completion-order behavior when the batch phase was refactored, without the
  provider-constraint awareness that F-RCT-04 later established.
- Direction: change the fixture to assert call-order terminals `["call-a","call-b"]`
  once `run_tools` buffers and reorders results (F-RCT-04-P1-01 direction),
  or move the assertion to the batch level (tools.rs test family); keep the
  stream-chunk interleaving assertion only where the identity contract is
  genuinely completion-ordered (per-stream events are).
- Regression validation: after the P1-01 fix, the reworked test asserts call
  order; a `run_tools`-level fixture with staggered completion asserts the
  context tool messages appear in assistant call order.
- Validation reports: [V02-01](../validations/Q-TST-01/V02-01.md), [V04-01](../validations/Q-TST-01/V04-01.md)

### Q-TST-01-P1-03: The Anthropic streaming SSE parse path has zero tests — all 7 `anthropic.rs` tests cover request/cache conversion, so F-LLM-03-P1-01/P1-02/P2-01 (interleaved tool calls, dropped final usage chunk, silent event drop) all shipped in a compile-tested-only seam

- Priority: P1
- Confidence: high
- Layer: adapter (test coverage of the provider seam)
- Evidence: `echo-integration/src/providers/anthropic.rs` test module (:1101)
  — 7 tests: `conversation_cache_breakpoints_skip_trailing_runtime_context`,
  `metadata_user_id_present_when_set`, `metadata_absent_when_user_id_none`,
  `cache_hints_with_empty_breakpoints_still_places_cache_control` + 3 more of
  the same request/cache class (V02 sample 6); the streaming parse path
  (anthropic.rs:410-617: `convert_response`, `AnthropicStreamEvent` handling,
  `MessageDelta` arm at :584-617, silent `if let Ok` drop at :510) has zero
  test references — grep `AnthropicStreamEvent` in test contexts: zero hits;
  no `message_delta` payload or interleaved-block fixture exists anywhere in
  `echo-integration` (F-TST-01 V01); defects shipped there:
  F-LLM-03-P1-01 (accumulator keyed by length vs stream index), F-LLM-03-P1-02
  (`AnthropicUsage` strict struct drops `message_delta.usage`), F-LLM-03-P2-01
  (silent drop of unparseable events).
- Reachability: every Anthropic/DeepSeek-Anthropic streaming turn (EKO main
  path); every future provider adapter regression run — the gate cannot fail
  on this seam.
- Expected invariant: the adapter's streaming contract (interleaved blocks,
  final usage chunk, malformed events) has fixture-level tests per F-LLM-03
  regression validations.
- Observed behavior: request-side conversion is tested; the entire response/
  stream side is compile-tested only; the F-LLM-03 fixes' prescribed
  regression fixtures do not exist.
- Impact: three P1/P2-class adapter defects shipped through a green gate;
  future streaming regressions on the Anthropic path (usage loss, tool-call
  corruption, silent drops) are structurally invisible to the suite —
  the single most defect-dense untested seam in the framework.
- Root cause: `convert_response`/stream parsing was written inline with the
  request-conversion code, and tests were added for the cache-plan features
  only; no wire fixtures were ever created for the response side
  (F-TST-01 V01 recorded the same for the whole integration crate).
- Direction: add the F-LLM-03-prescribed fixtures: (a) `message_delta`
  `{"usage":{"output_tokens":15}}` payload → final chunk with usage +
  finish_reason; (b) [text, tool_use] and [tool_use, text, tool_use] event
  sequences → correct tool-call assembly; (c) wrong-typed event line →
  logged/counted drop; place them in a `tests/` integration module or the
  provider test module with literal wire strings.
- Regression validation: the three fixtures above; they must fail before the
  F-LLM-03 fixes and pass after.
- Validation reports: [V01-01](../validations/Q-TST-01/V01-01.md), [V02-01](../validations/Q-TST-01/V02-01.md)

### Q-TST-01-P2-01: The compressors that host the F-CMP-01 P1 defects have zero-assertion or toy tests — `test_sliding_window_compressor` is print-only and the only `summary.rs` test exercises a bare `Mutex`, never `compress`

- Priority: P2
- Confidence: high
- Layer: framework (test quality)
- Evidence: `echo-state/src/compression/mod.rs:2252-2273`
  (`test_sliding_window_compressor`: pushes 6 turns, `prepare`, prints,
  `Ok(())` — zero asserts); `compressor/summary.rs:715-737`
  (`test_incremental_summary_state_management`: locks/reads/writes a bare
  `Mutex<Option<String>>`, never `SummaryCompressor::compress` at :593);
  `compressor/sliding_window.rs` 0 inline tests; the P1 defects live exactly
  there: F-CMP-01-P1-01 (sliding_window.rs:48-66 message-count windows never
  bound tokens), F-CMP-01-P1-02 (summary.rs:346 immortal system summary grows
  unbounded), F-CMP-01-P1-03 (levels.rs:392-396 fold inserts `Role::User`
  between paired tool calls — levels.rs has 10 tests but none covers the
  fold-interleaving case).
- Reachability: the mandatory `cargo test --workspace` runs these tests —
  they always pass regardless of compressor behavior.
- Expected invariant: compressors that can delete or corrupt context have
  assertions on message count, token bound, pairing preservation, and
  repeated-compression stability (task card F-CMP-01 requirements).
- Observed behavior: the sliding-window test would pass even if the
  compressor removed every message or exceeded the token budget; the summary
  "test" would pass if `compress` were deleted entirely; the P1 defects
  shipped with zero assertion that could catch them (V04 note: the F-CMP-01
  regression tests prescribed by the dependency report do not exist).
- Impact: the context-compression subsystem — the one that can silently drop
  or corrupt conversation content (data-loss adjacent) — has no regression
  net for its core algorithm; fix work for F-CMP-01-P1-01/02/03 lands without
  any failing-then-passing test.
- Root cause: the compressor tests were written as demonstration/print
  scripts during the original implementation and never converted into
  assertions; the summary state test was added later for the Mutex refactor
  only.
- Direction: convert `test_sliding_window_compressor` into assertions
  (after-count ≤ window, messages preserved in order, protected markers
  survive); rewrite the summary test to drive `SummaryCompressor::compress`
  twice and assert exactly one system summary exists (F-CMP-01-P1-02
  regression); add the P1-03 fold-pairing fixture; delete the toy Mutex test
  once real coverage exists.
- Regression validation: the converted tests fail before the F-CMP-01 fixes
  and pass after.
- Validation reports: [V01-01](../validations/Q-TST-01/V01-01.md), [V02-01](../validations/Q-TST-01/V02-01.md)

### Q-TST-01-P2-02: `revisioned_adapter.rs` — the EKO-to-framework revisioned task graph adapter (388 lines) — has zero tests and no round-trip/field-level conversion test, violating the AGENTS.md adapter-losslessness rule

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs`
  — 388 lines, zero `#[cfg(test)]` hits; header doc "Thin EKO adapters for
  the framework-owned task revision service" (:1-3); implements
  `RevisionedTaskStore`/`TaskRevisionService` conversion
  (`EkoRevisionedTaskStore::load` etc.) over `TaskRuntimeStore`; AGENTS.md
  "适配器必须保持薄且转换无损…转换必须有 round-trip/字段级测试"; adjacent
  `store.rs` has 34 meaningful tests but none drives this adapter (V01
  inventory).
- Reachability: every `task_create`/`task_update`/`task_list` call in EKO
  (the revisioned graph path A-TSK-02/A-TSK-04); a conversion bug (dropped
  field, wrong revision, lost metadata) is invisible to the entire suite.
- Expected invariant: the adapter has round-trip and field-level tests
  (EKO PlanTask/TaskRunStatus → framework TaskSpec/TaskExecution/TaskStatus →
  back) as mandated by AGENTS.md.
- Observed behavior: zero tests; the framework side has its own round-trip
  tests (revisioned.rs 3) but the EKO conversion boundary is compile-tested
  only.
- Impact: the claim/revision semantics that A-TSK-04 verified at the store
  level are unverified at the boundary where EKO facts become framework
  facts; any field drift (e.g., status mapping, revision propagation,
  metadata) ships silently through a green gate.
- Root cause: the adapter was added during the revisioned-graph migration
  (F-TSK-01/A-TSK-02) with tests concentrated in the store and framework
  layers, not at the conversion boundary.
- Direction: add a `revisioned_adapter.rs` test module with a
  field-by-field round-trip fixture: create/update/list a plan through the
  adapter, reload, assert TaskSpec/Execution/Status and EkoPlanMetadata
  survive; include a revision-bump and a stale-write rejection case.
- Regression validation: the round-trip fixture fails if any mapped field is
  dropped or reordered; `cargo test -p echo-agent-app-core` stays green.
- Validation reports: [V01-01](../validations/Q-TST-01/V01-01.md)

### Q-TST-01-P2-03: `chatStore.toolExecution.test.ts` "ignores duplicate start events for one execution ID" certifies id-keyed dedupe only — the live two-producer duplicate class (two ids, one logical call) of A-SRF-03-P2-01 is invisible to the suite

- Priority: P2
- Confidence: high
- Layer: application (test that restates a weaker implementation)
- Evidence: `echo-agent-cli/web-frontend/src/stores/chatStore.toolExecution.test.ts:26-45`
  (same-execution-id dedupe); production live path keys by producer id
  (`toolExecutionStore.ts:206-217` `ingest` by `tool.id`); the two live
  producers allocate distinct `detail_ref` UUIDs for the same logical
  (owner, call_id) (`tool_execution.rs:191-200`; A-SRF-02-P2-01) — the
  duplicate scenario the A-SRF-03-P2-01 finding describes (duplicated cards,
  inflated counts); the identity-based dedupe used by hydration
  (`executionIdentity`/`mergeToolExecution`, toolExecutionStore.ts:46-86) is
  not exercised by `ingest`.
- Reachability: `npx vitest run` runs this test — it passes for both the
  single-producer case and (by construction) the broken two-producer case.
- Expected invariant: a fixture covering the two-producer duplicate
  (identical owner+call_id, different ids → one row) exists, per the
  A-SRF-03-P2-01 regression validation.
- Observed behavior: the test certifies the weaker id-based invariant;
  A-SRF-03-P2-01 shipped green.
- Impact: the duplicate-projection defect class on the flagship surface has
  no regression net; the fix (upsert by executionIdentity) has no
  failing-then-passing test.
- Root cause: the test was written for the single producer that existed at
  the time; the second producer (A-SRF-02-P2-01) landed later without test
  extension.
- Direction: add the two-producer fixture from A-SRF-03-P2-01's regression
  validation (two started/finished pairs, same (owner, call_id), different
  ids → exactly one row); keep the same-id case as a second fixture.
- Regression validation: the new fixture fails before the ingest dedupe fix
  and passes after.
- Validation reports: [V01-01](../validations/Q-TST-01/V01-01.md), [V02-01](../validations/Q-TST-01/V02-01.md)

### Q-TST-01-P3-01: Six frontend stores have no test files — `browserStore.ts` (351 lines), `subagentDetailStore`, `workspaceStore`, `uiStore`, `toastStore`, `authStore` (122) — while `chatStore.ts` (527) is covered only via the tool slice (A-FE-03-P3-04)

- Priority: P3
- Confidence: high
- Layer: application (test coverage inventory)
- Evidence: V01 store-to-test mapping (grep of `src/stores/*.test.ts`):
  `authStore`, `browserStore`, `chatStore`, `subagentDetailStore`, `toastStore`,
  `uiStore`, `workspaceStore` have no test files; `browserStore.ts:351` lines
  (the `browser://event` ingest surface behind the dead-bridge defect
  A-SRF-02-P1-01); `chatStore` core reducer untested (A-FE-03-P3-04).
- Reachability: `npx vitest run` never loads these stores' logic.
- Expected invariant: every store that owns domain facts has fixture tests
  for its invariants (per A-FE-03 and the task card's frontend coverage
  requirement).
- Observed behavior: the stores are compile-tested only (TypeScript) or
  exercised indirectly through components without assertions.
- Impact: regressions in browser-event ingest, workspace switching, and UI
  state are invisible; low today because some surfaces are dormant (auth) or
  partially dead (browser bridge), but the pattern invites silent drift.
- Root cause: test effort concentrated on the chat/task/tool stores; the
  remaining stores grew organically without fixtures.
- Direction: add minimal fixture suites for `browserStore` (ingest
  idempotency), `subagentDetailStore`, and `workspaceStore`; delete
  `authStore`'s module-level intervals (A-FE-03-P3-05) before adding tests.
- Regression validation: new fixtures pass; `npx vitest run` stays green.
- Validation reports: [V01-01](../validations/Q-TST-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Production-module-to-test map (react loop, task runtime, LLM adapters, compression, frontend stores; driver classification; duplicate search) | yes | passed | [V01-01](../validations/Q-TST-01/V01-01.md) |
| V02 | Assertion/fixture quality sampling (21 tests, 3-class classification) | yes | passed | [V02-01](../validations/Q-TST-01/V02-01.md) |
| V03 | Ignored/flaky/platform-gated inventory (both repos + frontend skips) | yes | passed | [V03-01](../validations/Q-TST-01/V03-01.md) |
| V04 | Mutation/negative-control sampling on usage_reported, terminal monotonicity, claim protocol, batch ordering | yes | passed | [V04-01](../validations/Q-TST-01/V04-01.md) |
| V05 | Cross-reference with filed findings (mock invisibility cloak, canonical IDs) | yes | passed | [V05-01](../validations/Q-TST-01/V05-01.md) |

All required validations executed with known results; no validation is
pending; no command exit codes apply (read-only static review).

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `tests/react_smoke.rs:9-12` — full-loop mock tests deferred pending snapshot `llm_client` field | stale | field shipped (snapshot.rs:343); deferred tests still absent; non-streaming loop untested (Q-TST-01-P1-01) |
| F-TST-01-P1-01/P1-02 — mock usage/streaming shapes hide provider-contract classes | current | anchors re-verified (mock_llm.rs:413-435; think.rs:112-113,147); V04 negative control 1 |
| F-RCT-04-P2-01 — `then_tool_calls` zero usages, no concurrent batch tests | current | grep re-run: zero usages; tools.rs:215-227 completion-order push verified |
| A-FE-03-P3-04 — chatStore reducer zero direct tests | current | store-to-test mapping re-confirmed (V01) |
| Q-GUI-01-P3-01 — GUI matrix green with zero boot/setup tests | current | cross-ref; bin harness 0 tests |
| F-LLM-03 findings' prescribed regression fixtures exist | regressed/stale | none of the streaming fixtures (message_delta, interleaved blocks, malformed event) exist (Q-TST-01-P1-03) |

## Coverage And Uncertainty

- All conclusions are static; no test was executed in this task. Suite
  greenness at the reviewed commits is carried from dependency reports
  (F-TST-01 V04-03, F-RCT-04 V04-01/02, A-SRF-03 V04, Q-GUI-01 V02-01); a
  re-execution at these exact commits was not performed to conserve review
  budget — the gate tasks (Q-FW-01/Q-CLI-01/Q-WEB-01) own execution.
- V04 negative controls are thought experiments anchored to full reads of the
  tested code; they were not executed as mutations (read-only review).
- The 4 local `#[cfg(test)]` doubles (F-TST-01 inventory) and the
  `echo-state` MockEmbedder duplicate (F-TST-01-P3-01) were not re-audited.
- Framework `echo-orchestration` zero-test files (manager.rs, dag.rs,
  store.rs, events.rs, time.rs) are recorded as absence; their behavior is
  covered (if at all) by sibling-module tests — not exhaustively traced.
- Frontend totals (101 + 40 tests) are cited from A-SRF-03 V04 without
  re-execution.
- The pipeline.rs:1634 test's stream-chunk interleaving assertion has a minor
  timing nondeterminism risk (10 ms vs 60 ms controlled delays, biased
  select); not filed as flaky — noted in V02.

## Handoff

- Conclusions downstream tasks may rely on: the coverage map (V01) and
  credibility grading (V02/V04): EKO store/executor claims and terminal
  monotonicity are strongly tested; the framework loop suite is
  mock-shape-bound; the non-streaming loop, batch phase, Anthropic streaming
  parse path, EKO revisioned adapter, and 6 frontend stores are untested;
  pipeline.rs:1634 pins the wrong ordering contract; the ignored inventory is
  clean (V03); mock-invisibility theme confirmed via F-TST-01/F-RCT-04/
  F-LLM-03 canonical IDs (V05).
- Reports to read: this report + V01-01..V05-01; F-TST-01 (mock fidelity
  matrix), F-RCT-04 (P1-01/P2-01), F-LLM-03 (P1-01/P1-02), A-SRF-03 (P1-02/
  P2-01), A-FE-03 (P3-04), Q-GUI-01 (P3-01).
- Stale triggers: any change to `src/testing/*`, `think.rs` usage capture,
  `react_loop.rs` error path, `phases/tools.rs` ordering, `pipeline.rs:1634`,
  `anthropic.rs` parse path, `compression/**` compressor behavior,
  `revisioned_adapter.rs`, `toolExecutionStore.ts` ingest/merge, or the
  frontend store test inventory invalidates the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): F-RCT-02
  (non-streaming regression tests), F-RCT-04 (batch fixtures + ordering test
  reconciliation), F-LLM-03 (streaming fixtures), F-CMP-01 (compressor
  assertion tests), A-TSK-04/A-TSK-01 (revisioned adapter round-trip tests),
  A-SRF-03 (two-producer dedupe fixture), Q-FLT-01 (fault fixtures built from
  the P1-01/P1-03 directions), S-QA-01 (synthesis consumes this map).
