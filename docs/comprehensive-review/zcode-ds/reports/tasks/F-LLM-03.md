# F-LLM-03: Anthropic provider and prompt cache adapter

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Does the Anthropic adapter preserve the same contract, including thinking
blocks and cache-control behavior?

## Scope

- `echo-integration/src/providers/anthropic.rs` (full read, 1279 lines):
  `convert_request` / `convert_response` / `chat` / `chat_stream`, wire
  types (`AnthropicRequest`, `ContentBlock`, `AnthropicStreamEvent`,
  `AnthropicUsage`), `build_anthropic_thinking`, cache breakpoint placement.
- `echo-integration/src/providers/anthropic_cache.rs` (full read):
  `AnthropicCachePlan`.
- Thinking translation: `build_anthropic_thinking` (anthropic.rs:734-776),
  `echo-core/src/llm/thinking.rs` (ThinkingConfig/ThinkingProtocol
  semantics), `thinking_translate.rs` (contrast only).
- Neutral contract: `echo-core/src/llm/mod.rs` (ChatRequest/ChatChunk/
  ToolChoice), `echo-core/src/llm/types.rs` (Message/Usage/DeltaMessage),
  `echo-core/src/llm/cache/{mod,layout}.rs` (CacheHints/BreakpointTarget/
  PromptCacheLayout).
- Core-loop producers/consumers: `src/agent/react/run/phases/think.rs`
  (cache_hints, tool_choice, usage_reported), `react_loop.rs`
  (direct_answer/call_llm_with_retry), `processor.rs` (reasoning_content,
  tool_call assembly), `echo-state/src/compression/mod.rs` (canonical
  reinjection), `echo-core/src/compression.rs` (CanonicalContext).
- Adapter reachability: `echo-integration/src/providers/config.rs`
  (ProviderFactory → AnthropicClient).

## Out Of Scope

- OpenAI adapter internals — F-LLM-02 (completed); only the cache_hints
  consumer contrast was read.
- Shared SSE transport `client.rs` — F-LLM-01 (P1-01/P3-01); referenced only
  for cross-checks.
- Core streaming loop state machine / cancellation ordering — F-RCT-03.
- EKO-side usage display/aggregation — A-* tasks.
- `docs/comprehensive-review/codex/` and `zcode-glm/` directories (not read).

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, layering, no-parallel-semantics),
  shared `REPORTING.md`/`TASKS.md`, `zcode-ds/README.md`.
- Dependency reports read: zcode-ds `F-LLM-01` (mandatory — P1-01/P2-01/
  P2-02/P3-01, V03 usage assumption, handoff item), `F-CORE-01` (error
  taxonomy, LlmUsage semantics), `F-LLM-02` (OpenAI adapter contrast,
  cache_hints claim).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md`
  (M9 usage/cache observability), the adapter's own doc comments
  (e.g. "Capture output_tokens from message_delta").
- Anthropic Messages API wire shapes verified against public documentation
  via web search (message_start vs message_delta usage fields; SSE event
  sequence).

## Layering Decision

- Generic mechanism: the neutral LLM contract, thinking protocols, and cache
  layout primitives in `echo_core` (confirmed by F-LLM-01; no new claim).
- EKO product policy: none in this task; the adapter is provider plumbing.
- Adapter boundary: `anthropic.rs` is the request/response converter. It is
  mostly thin, but it owns cache-breakpoint strategy derivation (from_layout
  fallback) and inline SSE parsing — both legitimately adapter-local.
- Duplicate search terms: `AnthropicClient`, `AnthropicCachePlan`,
  `stream_post`/`parse_sse_chunk` consumers, `cache_hints` consumers,
  `reasoning_content` producers, `tool_choice` producers, `message_delta`,
  `ThinkingProtocol` implementations. Results: single Anthropic adapter;
  single cache_hints consumer (Anthropic); Anthropic does NOT use the shared
  SSE transport (separate inline parser — new cross-check); single
  thinking-protocol authority (`build_anthropic_thinking` keyed off
  `ModelProfile`). No parallel implementations.
- Cross-repository boundary gate: all findings stay inside `echo-agent`
  (adapter + core contract); nothing moves between repositories.

## Current Path

`ProviderFactory` (`config.rs:305-311`) constructs `AnthropicClient` →
`LlmClient::chat`/`chat_stream` (anthropic.rs:367-635) → `convert_request`
(request body, cache plan, thinking) → Anthropic `/v1/messages` SSE /
JSON → `convert_response` / inline stream handler → `ChatResponse`/
`ChatChunk` → core loop (`think.rs` trait path via `retry_llm_call`,
`react_loop.rs` direct_answer, `context.rs` pre_compaction_flush).
Usage: final streaming chunk → `last_usage` → `AgentEvent::LlmUsage`
(think.rs:112-211); non-streaming → `raw.usage` (react_loop.rs:100-102).
Thinking: `ChatRequest.thinking` → `build_anthropic_thinking`
(model-profile-driven) → `thinking` block / `effort` wire fields; response
thinking content is not modeled anywhere in the adapter.

## Findings

### F-LLM-03-P1-01: Streaming tool-call accumulator is keyed by map length, not stream block index — interleaved streams lose or corrupt tool calls

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/anthropic.rs:520-528`
  (`ContentBlockStart` ToolUse arm: `let idx = tool_call_args.len();
  tool_call_args.insert(idx, ...)` — the event's real `index` is bound to
  the ignored `_index` field at `:1054-1058`); `:547-552`
  (`ContentBlockDelta` looks up `tool_call_args.get_mut(&index)` by stream
  index); `:554-583` (`ContentBlockStop` removes by stream index)
- Reachability: `chat_stream` is the streaming main path for every
  Anthropic/DeepSeek-Anthropic call (think.rs:329 via
  `retry_llm_call`). Trigger: any assistant turn where a non-tool_use block
  (text, or a thinking block when thinking is on) precedes the tool_use
  block — the common "I'll check that" + tool call shape.
- Expected invariant: every tool_use block accumulates its `partial_json`
  deltas against its own stream `index` and emits exactly one assembled
  `DeltaToolCall` at `content_block_stop`.
- Observed behavior: [text@0, tool_use@1] → tool_use inserted under key 0,
  deltas/stop at index 1 miss → the tool call is silently dropped from the
  stream; [text@0, tool_use@1, tool_use@2] → tool_use@1's args are appended
  to tool_use@2's entry and stop@1 emits tool_use@2 with the wrong
  arguments. Works only by accident when tool_use is the first block.
- Impact: the ReAct loop never receives the tool call → agent produces a
  final answer without executing the tool (wrong answers), or executes a
  tool with another tool's arguments (corrupt execution). Silent — no log,
  no error.
- Root cause: accumulator key (insertion order) and lookup key (stream
  index) are two different numbering schemes that diverge exactly when
  blocks interleave.
- Direction: use the event's `index` as the map key (stop ignoring
  `_index`); add a streaming fixture test with [text, tool_use] and
  [tool_use, text, tool_use] event sequences asserting emitted tool calls.
- Regression validation: unit test feeding the `AnthropicStreamEvent`
  sequence for interleaved blocks through the stream handler and asserting
  the two emitted `ChatChunk`s carry id/name/args belonging to the correct
  tools; a two-tool stream asserting both calls arrive.
- Validation reports: [V02](../validations/F-LLM-03/V02-01.md),
  [V01](../validations/F-LLM-03/V01-01.md)

### F-LLM-03-P1-02: `message_delta.usage` ({output_tokens} only) fails the strict `AnthropicUsage` deserializer — the final usage/finish chunk is silently dropped on every real Anthropic stream

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `anthropic.rs:1037-1046` (`AnthropicUsage` requires
  `input_tokens: u32` and `output_tokens: u32` with no `#[serde(default)]`);
  `:1063-1068` (`MessageDelta { delta, usage }` parses the same struct);
  `:510` (`if let Ok(event) = serde_json::from_str::<AnthropicStreamEvent>` —
  parse failure drops the event silently); `:584-617` (the final chunk with
  finish_reason + usage is emitted only inside the `MessageDelta` arm);
  `:586` (code comment "Capture output_tokens from message_delta" — the
  author expected output_tokens-only usage, contradicting the struct)
- Reachability: every Anthropic streaming turn. Wire shape verified against
  Anthropic docs (message_start.usage = {input_tokens,
  cache_creation_input_tokens, cache_read_input_tokens}; message_delta.usage
  = {output_tokens} only). Anthropic-compatible gateways that echo
  input_tokens into message_delta (some DeepSeek endpoints) would parse.
- Expected invariant: the final streaming chunk carries `finish_reason` and
  `Usage` whenever the provider reports them (F-LLM-01 V03: Anthropic usage
  feeds `ChatChunk.usage`; MASTER-PLAN M9: provider usage is the accounting
  authority).
- Observed behavior: `message_delta` never deserializes → silently dropped
  → no final chunk → `last_usage = None` → `usage_reported: false` on every
  Anthropic streaming turn; token tracker, tokenizer calibration
  (think.rs:171-180), and cache-hit observability see zero; `stop_reason`
  is also lost.
- Impact: total usage accounting loss on the main Anthropic path; cache-hit
  rate metrics always zero; tokenizer calibration never converges;
  observability cannot distinguish "provider didn't report" from "adapter
  dropped it".
- Root cause: strict struct deserialization combined with the silent drop
  path of the inline SSE parser; the struct was written for the
  message_start shape and reused for message_delta.
- Direction: split `AnthropicUsage` into message_start-shape and
  message_delta-shape structs (delta usage = output_tokens only, with
  `#[serde(default)]` on all fields), or make all `AnthropicUsage` fields
  optional; treat unparseable events as errors or at least log (see P2-01);
  add a fixture reproducing the real message_delta payload.
- Regression validation: unit test parsing
  `{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":15}}`
  and asserting a final chunk with usage + finish_reason is produced; a
  loop-level test asserting `usage_reported: true` when the provider
  reported usage.
- Validation reports: [V03](../validations/F-LLM-03/V03-01.md),
  [V04](../validations/F-LLM-03/V04-01.md),
  [V05](../validations/F-LLM-03/V05-01.md)

### F-LLM-03-P1-03: Multiple leading system messages collapse to the last one — the base system prompt is silently dropped after canonical-context reinjection

- Priority: P1
- Confidence: medium
- Layer: adapter
- Evidence: `anthropic.rs:73-77` (`if msg.role == Role::System {
  system = msg.content.as_text(); continue; }` — each system message
  overwrites the previous; only the last survives); producer side:
  `echo-state/src/compression/mod.rs:900-931` (`reinject_canonical_context`
  inserts the base system prompt at index 0 and supplemental
  `[Canonical context — ...]` messages at `sys_end` as `Message::system`);
  `echo-core/src/compression.rs:362-401` (`to_reinjection_messages` builds
  project-rules/active-skills system texts); layout model explicitly
  supports multi-system lists (`echo-core/src/llm/cache/layout.rs:56-60,74-96`);
  EKO enables compression via token_limit (`echo-agent-cli/
  echo-agent-app-core/src/agent_pool.rs:433-435`, `state.rs:47` default 8000)
- Reachability: any conversation that triggers compression with canonical
  context configured (auto-wired at `src/agent/react/mod.rs:358-382` with
  system_prompt + project-rules when the `project-rules` feature is on) →
  next Anthropic request's `system` field = only the last system message
  (e.g. only the canonical-rules text, no persona prompt). Single-system
  conversations (pre-compression) are unaffected.
- Expected invariant: all leading system messages are preserved in order in
  the top-level `system` field.
- Observed behavior: the base system prompt is silently discarded whenever
  any system message follows it; only the last system message is sent.
- Impact: after the first compression cycle, Anthropic/DeepSeek-Anthropic
  turns lose the agent's persona and behavior rules — silent behavioral
  corruption of the whole session.
- Root cause: the adapter assumes at most one system message; the framework
  explicitly supports and produces several (canonical reinjection).
- Direction: collect all leading `Role::System` texts in order and join them
  into the `system` field (or emit multiple `SystemBlock`s), keeping
  cache_control on the last block; add a convert_request test with
  [system prompt, canonical context, history...].
- Regression validation: fixture with two system messages asserting the
  request body `system` contains both texts in order (and cache_control on
  the last); a compression-reinjection round-trip test if feasible.
- Validation reports: [V01](../validations/F-LLM-03/V01-01.md)

### F-LLM-03-P1-04: Thinking blocks in responses are unmodeled — non-streaming `chat()` fails to parse them; streaming silently discards them and never populates `reasoning_content`

- Priority: P1
- Confidence: medium
- Layer: adapter
- Evidence: `anthropic.rs:827-866` (`ContentBlock` has no `thinking`/
  `redacted_thinking` variant and no `#[serde(other)]` → a response
  containing a thinking block fails deserialization); `:410-413`
  (non-streaming `resp.json()` → `LlmError::NetworkError("Response parse
  error")`); `:1080-1086` (`ContentBlockStartBody` models only `tool_use`,
  rest → `Other`); `:1088-1093` (`ContentDelta` models only `text`/
  `partial_json` — `thinking_delta` deltas ignored); `:538-546` (text
  chunks yield `reasoning_content: None`); request side enables thinking
  (`build_anthropic_thinking` :734-776); contract consumers:
  `src/agent/react/run/processor.rs:18-36` (ThinkStart/ThinkEnd from
  `reasoning_content`), `src/agent/react/run/stream_channel.rs:658-737`
- Reachability: non-streaming `chat()` is called by `direct_answer`
  (`react_loop.rs:763-813` via `call_llm_with_retry`, which sets
  `thinking: self.thinking.clone()` at `:60`) and by external
  `chat_simple` users; whenever thinking is enabled (or an adaptive-only
  model thinks by default) the response contains thinking blocks → hard
  parse failure. Streaming (main path) parses fine but drops thinking
  content silently.
- Expected invariant: the adapter that translates thinking on the request
  side must represent thinking content on the response side
  (`Message.reasoning_content` / `ChatChunk.delta.reasoning_content` exist
  in the neutral contract and are consumed by the core loop).
- Observed behavior: non-streaming thinking responses error with a
  misleading NetworkError; streaming thinking deltas are silently dropped;
  `reasoning_content` is always `None` on Anthropic — ThinkStart/ThinkEnd
  events never fire, no reasoning display on any Anthropic surface.
- Impact: non-streaming paths (direct answer, structured output with
  thinking) fail outright on thinking-capable models; the thinking
  observability contract (F-LLM-01 current path: thinking translation +
  reasoning display) is not preserved at the Anthropic boundary.
- Root cause: the response-side wire model was built before thinking
  responses existed and was never extended; the neutral contract's
  `reasoning_content` channel is unused by this adapter.
- Direction: add `Thinking { thinking, signature }` (and
  `RedactedThinking`) variants to `ContentBlock`; map thinking deltas to
  `reasoning_content` in `ChatChunk` (and non-streaming thinking blocks to
  `Message.reasoning_content`); add fixtures for both stream and
  non-streaming thinking payloads.
- Regression validation: fixture deserializing a non-streaming response
  with a thinking block and asserting `ChatResponse.message.
  reasoning_content`; a streaming fixture with thinking deltas asserting
  `reasoning_content` chunks.
- Validation reports: [V01](../validations/F-LLM-03/V01-01.md),
  [V02](../validations/F-LLM-03/V02-01.md)

### F-LLM-03-P2-01: The Anthropic inline SSE parser silently drops unparseable events with no logging — same defect class as F-LLM-01-P1-01 on a separate code path

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `anthropic.rs:510` (`if let Ok(event) = serde_json::from_str::<AnthropicStreamEvent>(data)` — no else, no `warn!`, no counter);
  contrast the shared transport `echo-integration/src/providers/client.rs:99-105`
  (serde failure → `warn!` + drop) reviewed in F-LLM-01-P1-01
- Reachability: every malformed/unknown event on every Anthropic streaming
  call; P1-02 is a live instance (well-formed per Anthropic spec,
  malformed per the adapter's struct).
- Expected invariant: streaming must preserve provider data or fail loudly;
  dropped events must at least be observable (F-LLM-01-P1-01 invariant).
- Observed behavior: events failing `AnthropicStreamEvent`/`ContentBlock`/
  `AnthropicUsage` deserialization vanish with zero trace; a whole turn can
  silently lose its final chunk.
- Impact: silent content/usage loss indistinguishable from normal
  end-of-stream; debugging requires byte-level inspection; the F-LLM-01
  fix for `client.rs` will NOT cover this path (separate parser).
- Root cause: inline parser predates the shared transport's lenient-parse
  hardening and was written with a bare `if let Ok` guard.
- Direction: make the parse arm emit a `warn!` with the offending line (or
  a typed error / drop counter surfaced at stream end), aligning with
  F-LLM-01-P1-01's direction; reuse the shared `split_sse_event` helpers
  if possible.
- Regression validation: unit test feeding a wrong-typed event line
  (`{"type":"message_delta","usage":{"output_tokens":"x"}}`, unknown block
  type) and asserting a logged drop or counted error.
- Validation reports: [V05](../validations/F-LLM-03/V05-01.md),
  [V04](../validations/F-LLM-03/V04-01.md)

### F-LLM-03-P2-02: `ChatRequest.tool_choice` and `response_format` are silently dropped by the Anthropic adapter

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `anthropic.rs:277-290` (`AnthropicRequest` has no
  `tool_choice`/`response_format` fields); contract doc
  `echo-core/src/llm/mod.rs:112-113` (ToolChoice documents its Anthropic
  mapping `{"type":"auto"/"any"/"tool","name":...}`); producers:
  `think.rs:317-318` (`tool_choice: Some("none")` on final_only turns),
  `react_loop.rs:60-62` (`response_format: self.config.response_format.clone()`)
- Reachability: today the core loop sets `tool_choice` only to `"none"` on
  final_only turns, which is masked because `tools_for_request` returns
  `None` on the same turns (think.rs:395-396) — so the drop is currently
  latent, not live; `response_format` is dropped whenever configured
  (structured-output config) with no warning (Anthropic has no direct
  equivalent).
- Expected invariant: a neutral-contract field either reaches the provider
  wire or is observably ignored.
- Observed behavior: both fields vanish silently; a future
  `ToolChoice::Required`/`Function` producer would silently get `auto`.
- Impact: silent contract drop; structured-output config on Anthropic is
  a no-op; the documented ToolChoice mapping is unfulfilled.
- Root cause: adapter predates these neutral fields and never mapped them.
- Direction: map `tool_choice` to Anthropic `tool_choice` wire
  (`{"type":"any"|"tool","name":...}`, `"none"` → `{"type":"none"}`-equivalent
  or documented ignore) or add a documented-ignore warn; add a `warn!` for
  dropped `response_format`.
- Regression validation: request-JSON fixture with `ToolChoice::Function`
  asserting the wire `tool_choice` field; a fixture with `response_format`
  asserting the documented warn/ignore.
- Validation reports: [V01](../validations/F-LLM-03/V01-01.md)

### F-LLM-03-P2-03: `Minimal`/`None` thinking level on Claude 4.6 (AnthropicEffort) still emits `thinking:{type:"adaptive"}` — thinking stays ON despite the documented "Minimal = 关闭思考" semantics

- Priority: P2
- Confidence: medium
- Layer: adapter
- Evidence: `anthropic.rs:755-766` (AnthropicEffort arm: block is `None`
  only when `ThinkingConfig::Disabled`; `Level(None)`/`Minimal` still emit
  the adaptive block; `effort` is `None` for them via
  `to_anthropic_effort`); documented semantics:
  `echo-core/src/llm/thinking.rs:24-28` ("对 Claude 映射为不发 thinking
  字段(=关闭)" for Minimal) and `thinking.rs:127-128`
- Reachability: `ChatRequest.thinking = Some(Level(Minimal))` (or
  `Level(None)`) with a Claude 4.6 model → adaptive thinking block sent →
  thinking engaged.
- Expected invariant: Minimal on Claude means thinking off (no thinking
  field), per the shared ThinkingConfig docs.
- Observed behavior: the adaptive block is emitted (thinking ON) with no
  effort — the "fastest/cheapest" user intent is not honored; only
  `Disabled` suppresses the block.
- Impact: user-selected minimal thinking on Claude 4.6 pays full thinking
  cost/latency; semantic inconsistency between Disabled and Minimal.
- Root cause: the block-suppression condition checks only `Disabled`
  instead of the documented off-set (`Disabled | None | Minimal`).
- Direction: align the block condition with `to_anthropic_effort`'s
  off-set (`matches!(cfg, Disabled | Level(None) | Level(Minimal))` →
  no block).
- Regression validation: convert_request fixture for model `claude-sonnet-4-6`
  with `Level(Minimal)` asserting no `thinking` field; `Level(High)`
  asserting adaptive + `effort:"high"`.
- Validation reports: [V01](../validations/F-LLM-03/V01-01.md)

### F-LLM-03-P3-01: Dead code in the cache/stream path — unreachable `apply_conversation_cache_breakpoints`, test-only `AnthropicCachePlan` helpers, never-constructed `AnthropicSystem::Text`, dead `[DONE]` check

- Priority: P3
- Confidence: high
- Layer: adapter
- Evidence: `anthropic.rs:243-245` (the `cache_plan.breakpoints.is_empty()`
  branch is unreachable: `history_breakpoint_count() > 0` at `:237` implies
  `breakpoints` non-empty) → `apply_conversation_cache_breakpoints`
  (`:909-939`) has no live caller (its heuristic lives on via
  `AnthropicCachePlan::from_layout`); `anthropic_cache.rs:83-121`
  (`from_layout_or_default`, `default_plan`, `has`, `history_indices`,
  `has_history_last_stable` — zero callers outside tests; `AnthropicCachePlan`
  itself is re-exported at `providers/mod.rs:12`); `anthropic.rs:668-671`
  (`AnthropicSystem::Text` never constructed); `:507-509` (`[DONE]` never
  sent by the Messages SSE API)
- Reachability: never executed at runtime (dead branches/types).
- Expected invariant: no dead code in the adapter (AGENTS.md: 过时代码直接删).
- Observed behavior: the dead fallback and helpers remain, including a
  public re-export of a partially-dead type; `apply_conversation_cache_breakpoints`
  keeps its own tests that pin unreachable behavior.
- Impact: maintenance confusion (two breakpoint heuristics — one dead),
  misleading public API surface; no runtime effect.
- Root cause: iterative extraction of the cache plan from convert_request
  left the old fallback and convenience helpers behind.
- Direction: delete `apply_conversation_cache_breakpoints` + its tests,
  delete the test-only `AnthropicCachePlan` helpers, delete
  `AnthropicSystem::Text` and the `[DONE]` arm; decide the
  `AnthropicCachePlan` re-export (keep if X-BND-01 finds an external
  consumer, else drop).
- Regression validation: grep for the deleted symbols returns zero code
  hits; `cargo test -p echo_integration --lib --locked anthropic` stays
  green after removing the dead tests.
- Validation reports: [V01](../validations/F-LLM-03/V01-01.md),
  [V04](../validations/F-LLM-03/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---:|---|
| V01 | Request/response field mapping (loss/rename/hardcode/dead code) | yes | passed | [V01](../validations/F-LLM-03/V01-01.md) |
| V02 | Interleaved content-block streaming (text/tool_use/thinking order, delta assembly, block-end signals) | yes | passed | [V02](../validations/F-LLM-03/V02-01.md) |
| V03 | Cache breakpoint injection + cache_creation/read accounting to ChatChunk/LlmUsage vs `usage_reported` | yes | passed | [V03](../validations/F-LLM-03/V03-01.md) |
| V04 | `cargo test -p echo_integration --lib --locked anthropic` + malformed-fixture coverage inventory | yes | passed (exit 0, 14/14) | [V04](../validations/F-LLM-03/V04-01.md) |
| V05 | Cross-reference with F-LLM-01 (P1-01 SSE scope, cache_hints consumption, V03 usage assumption) | conditional | passed | [V05](../validations/F-LLM-03/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-LLM-01 handoff: "F-LLM-03 should confirm Anthropic usage mapping feeds ChatChunk.usage (V03 depends on it)" | regressed (mapping exists, unreachable on real wire) | [V03](../validations/F-LLM-03/V03-01.md) — F-LLM-03-P1-02 |
| F-LLM-01-P1-01: "shared SSE transport silently drops malformed chunks" | current, but does NOT cover the Anthropic inline parser | [V05](../validations/F-LLM-03/V05-01.md) — separate code path, F-LLM-03-P2-01 |
| F-LLM-02-P2-01: "cache_hints is only consumed by the Anthropic adapter" | current | [V05](../validations/F-LLM-03/V05-01.md) — anthropic.rs:171-202 |
| MASTER-PLAN M9: "provider/API usage 是记账权威;缺失时标记 unknown" | regressed on the Anthropic streaming path | [V03](../validations/F-LLM-03/V03-01.md) — F-LLM-03-P1-02 forces `usage_reported: false` |
| MASTER-PLAN M9: "各 provider fixture 覆盖 cache token 语义" | current at unit level; fixture gap at adapter level | [V04](../validations/F-LLM-03/V04-01.md) — no streaming/response fixtures exist |
| Adapter doc comment: "Capture output_tokens from message_delta" (anthropic.rs:586) | regressed — the struct requires `input_tokens` | [V03](../validations/F-LLM-03/V03-01.md) |

## Coverage And Uncertainty

- Adapter behavior is verified by code inspection plus one external
  documentation cross-check (Anthropic message_start/message_delta usage
  shape); no live-provider test exists.
- F-LLM-03-P1-02 confidence is high on the serde mechanics and the wire
  shape, but the DeepSeek-Anthropic gateway (the adapter's main consumer
  per the metadata.user_id comments) may include `input_tokens` in
  `message_delta` and thus parse fine — provider-dependent.
- F-LLM-03-P1-03 confidence medium: the multi-system-message reachability
  depends on compression + canonical reinjection actually running in EKO
  (token_limit/budget configuration enables it; the canonical context is
  auto-wired). Pre-compression single-system conversations are unaffected.
- F-LLM-03-P1-04 confidence medium: the non-streaming parse failure is
  code-certain whenever thinking blocks appear; whether they appear
  depends on thinking being enabled (request-side) or adaptive-only models
  thinking by default.
- `temperature` interplay with thinking on Anthropic (some model families
  historically rejected temperature with thinking enabled) was not
  verified against docs for the mid-2026 model lineup and is left as an
  open question, not a finding.
- The `finish_reason`/stop_reason loss caused by P1-02 compounds
  F-LLM-01-P3-01/F-LLM-02's "finish_reason dropped" observations at the
  core-loop level; loop-level impact is F-RCT-03 scope.
- No `echo-agent-cli` source was modified; EKO call sites cited for
  reachability only.

## Handoff

- Downstream tasks may rely on: request/response mapping inventory (V01);
  streaming tool-call assembly defect + fix direction (V02, P1-01);
  usage-accounting break on the real wire (V03, P1-02); green unit suite
  with no streaming/response fixtures (V04); F-LLM-01 cross-reference
  conclusions (V05).
- F-LLM-01/transport: F-LLM-01-P1-01's fix must also cover the Anthropic
  inline parser (P2-01), and the malformed-chunk fixture work should be
  extended to Anthropic event fixtures (P1-02's regression tests).
- F-RCT-03: cancellation ends Anthropic streams silently (same as the
  shared transport, P3-01 extension); stop_reason is lost on real
  Anthropic streams (P1-02), so truncation signals are absent.
- X-BND-01: decide the `AnthropicCachePlan` public re-export after the
  P3-01 dead-code removal; confirm no external consumer of the
  tool_choice/response_format drop expectations.
- A-* (EKO usage display): Anthropic streaming turns report
  `usage_reported: false` and zero tokens — EKO token accounting on the
  Anthropic/DeepSeek-Anthropic path will be empty until P1-02 is fixed.
- This report becomes stale if the Anthropic wire types, the stream
  handler, the thinking response modeling, or the core-loop request
  builders change.
