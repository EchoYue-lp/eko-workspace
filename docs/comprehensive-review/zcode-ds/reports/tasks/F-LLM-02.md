# F-LLM-02: OpenAI provider adapter fidelity

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean except an untracked
> `echo-agent/tests/f_rct_01_probe.rs` (owned by another task; untouched)

## Question

Does the OpenAI adapter faithfully implement the neutral contract for request
construction, deltas, tool calls, usage, and failures?

## Scope

- `echo-integration/src/providers/openai.rs` (full read: standalone
  `chat`/`stream_chat`, `OpenAiClient`, `DefaultLlmClient`, `normalize_messages`).
- Shared transport `echo-integration/src/providers/client.rs` (`post`/
  `stream_post`/`parse_sse_chunk`/`split_sse_event`) — the SSE boundary every
  OpenAI streaming call traverses.
- Wire contract `echo-core/src/llm/types.rs` (`ChatCompletionRequest/Response/
  Chunk`, `DeltaMessage`, `DeltaToolCall`, `Usage`), neutral contract
  `echo-core/src/llm/mod.rs` (`ChatRequest`/`ChatResponse`/`ChatChunk`).
- Thinking translation `thinking_translate.rs` (request-field interaction).
- Delta merging downstream of the adapter: `echo-agent/src/agent/react/run/
  processor.rs` (tool-call assembly), `phases/think.rs`, `stream_channel.rs`,
  `react_loop.rs` (usage/finish_reason consumers).
- Config `config.rs` (`provider_base_url`, `get_model`), adapter reachability
  `src/agent/react/builder.rs`, facade `src/llm.rs`.

## Out Of Scope

- Anthropic adapter internals — F-LLM-03 owns them; only the cache_hints
  consumption contrast (anthropic.rs:165-188) was read.
- Core streaming loop state machine / cancellation ordering — F-RCT-03.
- EKO usage display/aggregation — A-* tasks.
- `tests/f_rct_01_probe.rs` and all files under `docs/comprehensive-review/
  codex/` and `zcode-glm/` (not read).

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, layering, no-parallel-semantics),
  shared `REPORTING.md`/`TASKS.md`, `zcode-ds/README.md`.
- Dependency reports read: zcode-ds `F-LLM-01` (mandatory — P1-01/P2-01/P2-02/
  P2-03/P3-01 and the handoff item "F-LLM-02 must check whether the OpenAI
  adapter's request assembly can produce the malformed-shape cases P1-01 warns
  about, and own the malformed-chunk fixture tests"), `F-CORE-01` (error
  taxonomy, LlmUsage semantics).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md` (M9
  usage/cache observability claims).

## Layering Decision

- Generic mechanism: the neutral LLM contract and wire types stay in
  `echo_core` (confirmed by F-LLM-01; no new claim).
- EKO product policy: none in this task.
- Adapter boundary: `openai.rs` is a thin request/response converter. Verified
  lossless pass-through for messages/temperature/tools/tool_choice/
  response_format/thinking/user_id; two fidelity gaps at the boundary
  (P1-01 max_tokens, P2-01 cache_hints) and two hardcodes (P3-01, P3-02).
  No scheduling/state authority lives in the adapter.
- Duplicate search terms: `max_completion_tokens`, `max_tokens` (wire
  emission), `cache_hints` consumers, `include_usage`, `ChatChunk`,
  `DeltaToolCall` assemblers, `DefaultLlmClient` constructors — single
  authoritative adapter pair (`OpenAiClient`/`DefaultLlmClient`), single
  delta-assembly authority (`processor.rs`), single cache_hints consumer
  (`anthropic.rs`). No parallel implementations found.
- Cross-repository boundary gate: all findings stay inside `echo-agent`
  (adapter + wire types); nothing moves between repositories.

## Current Path

`LlmClient` trait (`OpenAiClient`/`DefaultLlmClient`, openai.rs:263/:401)
← constructed by `ReactAgentBuilder` (builder.rs:256-276, incl.
`OpenAiClient::from_env`) → `chat`/`chat_stream` build `ChatCompletionRequest`
(openai.rs:280-300/:336-355) → shared transport `post`/`stream_post`
(client.rs:110/:182) → wire types → neutral `ChatResponse`/`ChatChunk`
(openai.rs:310-316/:366-375) → core loop (`think.rs` trait path :285-350,
`stream_channel.rs`, `react_loop.rs`). Thinking config resolved per model/
provider by `translate_thinking_openai_compat` (thinking_translate.rs:40-147)
before request assembly. Usage: streaming final chunk → `ChatChunk.usage` →
`last_usage` → `AgentEvent::LlmUsage{usage_reported}` (think.rs:110-211);
non-streaming → `raw.usage` (react_loop.rs:100-102).

## Findings

### F-LLM-02-P1-01: OpenAI adapter sends `max_tokens` to reasoning models (o1/o3/o4/gpt-5) that reject it with HTTP 400 — no `max_completion_tokens` mapping

- Priority: P1
- Confidence: medium
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/openai.rs:289` and
  `:343` (`max_tokens: request.max_tokens` emitted unconditionally in
  `OpenAiClient::chat` and `chat_stream`); wire type `echo-core/src/llm/types.rs:537-538`
  (serde field named `max_tokens`); the adapter's sibling special-casing of
  the same model families in `thinking_translate.rs:56-110`
  (`drop_temperature` at :106, `reasoning_effort`) proves o-series/gpt-5 are
  first-class targets; neutral field `ChatRequest.max_tokens`
  (`echo-core/src/llm/mod.rs:156`)
- Reachability: `OpenAiClient::chat/chat_stream` with model `o1*`/`o3*`/
  `o4*`/`gpt-5*` and `request.max_tokens = Some(...)` → provider HTTP 400
  "Unsupported parameter: 'max_tokens' is not supported with this model" →
  `LlmError::ApiError`. `max_tokens` is set by: agent config
  (`src/agent/config.rs:157,257`, user-supplied, default `None`), and the
  `chat_simple*` conveniences (`openai.rs:476-478` hardcodes 2048; trait
  default `SimpleChatOptions` 2048 at `mod.rs:29-36`), whose EKO callers are
  `echo-agent-cli/echo-agent-app-core/src/runtime.rs:454` (checkpoint
  reflection, `with_max_tokens(300)`), `src/cli/repl.rs:371`,
  `src/tauri/commands/providers.rs:382` — any of these pointed at a reasoning
  model fails with 400 (runtime.rs degrades to a fallback with a warn;
  providers.rs surfaces the error)
- Expected invariant: request construction must produce provider-accepted
  bodies for the models the adapter explicitly supports
- Observed behavior: `max_tokens` is emitted for every model; the models the
  adapter itself special-cases in thinking translation reject it
- Impact: every gpt-5/o1/o3/o4 call with a token cap fails; reasoning-model
  usage on this adapter is broken whenever `max_tokens` is configured or a
  `chat_simple` path is used
- Root cause: 1:1 neutral-field pass-through without the model-family
  awareness that the sibling thinking translation already has
- Direction: teach `ChatCompletionRequest` a `max_completion_tokens` field
  (serde-renamed, mutually exclusive with `max_tokens`), and resolve the wire
  name by model prefix (`o1/o3/o4/gpt-5`) in `translate_thinking_openai_compat`
  or a companion helper; never send both fields
- Regression validation: fixture asserting the request JSON for model
  `gpt-5`/`o3-mini` with `max_tokens = Some` contains
  `max_completion_tokens` and no `max_tokens`, and for `gpt-4o` contains
  `max_tokens`; a second fixture asserting neither field when `None`
- Validation reports: [V01](../validations/F-LLM-02/V01-01.md),
  [V02](../validations/F-LLM-02/V02-01.md)

### F-LLM-02-P2-01: `ChatRequest.cache_hints` is silently dropped by the OpenAI adapter while the core loop computes a full cache layout on every call

- Priority: P2
- Confidence: medium
- Layer: adapter
- Evidence: `openai.rs` never reads `request.cache_hints` (both `chat` and
  `chat_stream` map it nowhere; the wire type has no such field); the only
  consumer is the Anthropic adapter (`anthropic.rs:165-188`); contract doc
  `echo-core/src/llm/mod.rs:180-183` ("Providers consume this to place cache
  breakpoints and log diagnostics"); producer `think.rs:302-327` computes
  `PromptCacheLayout::from_messages` + `stable_prefix_hash` + `segment_ranges`
  into `cache_hints: Some(...)` on every streaming call (F-LLM-01 V01 recorded
  this as an open question for this task)
- Reachability: every framework streaming call to an OpenAI-compatible model
  (think.rs trait path) carries `cache_hints = Some(...)`; the OpenAI adapter
  discards it silently — no warn, no mapping, no documented ignore
- Expected invariant: a neutral-contract field either reaches the provider or
  is observably ignored; the framework's M9 cache diagnostics should not be
  silently absent per provider
- Observed behavior: for all OpenAI-compatible providers the computed
  breakpoint/prefix-hash payload is dead work on every turn and the cache
  diagnostics promised by M9 (stable prefix hash, segment ranges) never reach
  any consumer; the field's doc over-promises at the OpenAI boundary
- Impact: silent contract drop; wasted per-call layout computation; a future
  OpenAI-compatible provider with a hint mechanism would silently not receive
  hints; cache observability is provider-asymmetric without any surface
- Root cause: cache hints were designed for Anthropic's cache-control blocks;
  the OpenAI adapter has no wire mapping and no explicit ignore path
- Direction: document the OpenAI-boundary ignore on the adapter (or map to a
  provider-specific hint field if/when one exists); optionally emit a
  `debug!`/`warn!` when `cache_hints` is `Some`; consider skipping the layout
  computation in `think.rs` for OpenAI-compatible providers (core-side,
  coordinate with F-RCT)
- Regression validation: unit test asserting that a request built with
  `cache_hints = Some(...)` either maps to a wire field or logs the documented
  ignore; no behavioral change for Anthropic
- Validation reports: [V01](../validations/F-LLM-02/V01-01.md)

### F-LLM-02-P3-01: `DefaultLlmClient::chat_simple` hardcodes temperature 0.3, contradicting the documented `SimpleChatOptions` default of 0.7

- Priority: P3
- Confidence: high
- Layer: adapter
- Evidence: `openai.rs:472-480` (`chat_simple` → `SimpleChatOptions {
  temperature: Some(0.3), max_tokens: Some(2048) }`); `echo-core/src/llm/mod.rs:29-36`
  (`SimpleChatOptions::default` = 0.7/2048) and trait doc `mod.rs:71-76`
  ("defaults are temperature: 0.7 and max_tokens: 2048"); `OpenAiClient` does
  not override `chat_simple`, so the same call yields 0.3 on
  `DefaultLlmClient` and 0.7 on `OpenAiClient`
- Reachability: `DefaultLlmClient::chat_simple` callers (EKO
  `src/tauri/commands/providers.rs:382`); behavior divergence between the two
  OpenAI-compatible clients
- Expected invariant: a documented default must not be silently overridden in
  one implementation of the same trait
- Observed behavior: `chat_simple()` produces temperature 0.3 on
  `DefaultLlmClient` and 0.7 everywhere else, with no comment explaining the
  divergence
- Impact: same call, different sampling across client types; doc/behavior
  mismatch (minor)
- Root cause: leftover hardcoded override predating `SimpleChatOptions`
- Direction: delete the override and use `SimpleChatOptions::default()` (or
  keep 0.3 and document it as an intentional `DefaultLlmClient` policy — but
  the shared trait doc says 0.7)
- Regression validation: unit test asserting `chat_simple` resolves to
  `SimpleChatOptions::default()` (or the corrected doc)
- Validation reports: [V01](../validations/F-LLM-02/V01-01.md)

### F-LLM-02-P3-02: `stream_options: {"include_usage": true}` is hardcoded with no neutral-contract knob

- Priority: P3
- Confidence: medium
- Layer: adapter
- Evidence: `openai.rs:346` and `:194` (hardcoded in `chat_stream`/
  `stream_chat`); `ChatCompletionRequest.stream_options` is
  `Option<serde_json::Value>` (types.rs:543-544) — wire-typed but never
  driven by `ChatRequest`
- Reachability: every streaming request; strict OpenAI-compatible gateways
  that reject unknown `stream_options` fail the whole stream with a 400/422
  (loud); gateways that ignore it silently degrade to
  `usage_reported: false` (graceful, observability cost)
- Expected invariant: request construction should not hardcode provider
  options the neutral contract cannot control, or the hardcode should be
  documented as required for usage reporting
- Observed behavior: include_usage is always-on; the neutral contract has no
  way to express it or disable it
- Impact: provider-compat friction for strict gateways; no user-visible
  failure on conforming providers (usage reporting depends on it)
- Root cause: usage reporting was wired by hardcoding the OpenAI option
  instead of adding a contract field
- Direction: either document the hardcode as the usage-reporting contract (it
  matches OpenAI's spec for include_usage), or add an opt-out to
  `ChatRequest`; add a fixture asserting `stream_options.include_usage ==
  true` on the stream body so the behavior is pinned
- Regression validation: request-JSON fixture for `chat_stream` asserting
  `stream_options` presence (pins current behavior; a later opt-out must
  update the fixture)
- Validation reports: [V01](../validations/F-LLM-02/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---:|---|
| V01 | Request field mapping ChatRequest → OpenAI body (loss/rename/hardcode) | yes | passed | [V01-01](../validations/F-LLM-02/V01-01.md) |
| V02 | Streamed/non-streamed response + usage both modes + P1-01 trigger surface | yes | passed | [V02-01](../validations/F-LLM-02/V02-01.md) |
| V03 | Tool-call assembly edges, finish_reason, [DONE], panic safety | yes | passed | [V03-01](../validations/F-LLM-02/V03-01.md) |
| V04 | `cargo test -p echo_integration --lib --locked openai` | yes | passed (exit 0, 6/62) | [V04-01](../validations/F-LLM-02/V04-01.md) |
| V04 | `cargo test -p echo_integration --lib --locked providers::client` (SSE fixtures) | conditional | passed (exit 0, 3/62) | [V04-02](../validations/F-LLM-02/V04-02.md) |
| V04 | `cargo test -p echo_integration --lib --locked` (full suite) | conditional | passed (exit 0, 62/62) | [V04-03](../validations/F-LLM-02/V04-03.md) |
| V05 | Cross-reference with F-LLM-01 findings | conditional | performed | [V05-01](../validations/F-LLM-02/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN M9: "各 provider fixture 覆盖 cache token 语义" | current at unit level; fixture gap at adapter level | [V04-02](../validations/F-LLM-02/V04-02.md), [V04-03](../validations/F-LLM-02/V04-03.md) — cache-token semantics tested in `types.rs`/`anthropic_cache.rs`; no OpenAI adapter request/response fixture exists |
| MASTER-PLAN M9: "provider/API usage 是记账权威;缺失时标记 unknown" | current via streaming path | [V02-01](../validations/F-LLM-02/V02-01.md) — adapter delivers usage in both modes; `usage_reported` honored on streaming paths (think.rs:147) |
| MASTER-PLAN M9: "统一 prompt/cached/cache creation/output/total 的 provider 归一语义" | current | `Usage::effective_*`/`cached_prompt_tokens` (types.rs:686-771) consumed unchanged by the adapter (pass-through) |
| F-LLM-01 handoff: "F-LLM-02 must check whether the OpenAI adapter's request assembly can produce the malformed-shape cases P1-01 warns about, and own the malformed-chunk fixture tests" | settled (negative + gap) | [V05-01](../validations/F-LLM-02/V05-01.md), [V04-02](../validations/F-LLM-02/V04-02.md) — assembly cannot produce malformed chunks (typed serde); malformed-chunk fixtures still missing at the `parse_sse_chunk` boundary |

## Coverage And Uncertainty

- Adapter request/response mapping is verified by inspection only — no
  adapter-level fixture or live-provider test exists (V04-02 gap). A live
  OpenAI-compatible provider test would be needed to prove wire acceptance
  (in particular for P1-01's 400 claim, which rests on OpenAI's documented
  behavior cross-checked via web search, not a local fixture).
- P1-01 confidence is medium: the 400 behavior is documented by OpenAI and
  corroborated by multiple independent adapters, but the specific provider
  endpoints EKO uses (openai/deepseek/dashscope/zhipu/moonshot) may vary in
  strictness.
- The streaming `finish_reason` ("length" truncation) being dropped by the
  core loop (`processor.rs` ignores it, `ThinkOutput` has no field) is a
  framework-core gap observed during V02/V03; it is out of adapter scope and
  recorded for F-RCT.
- The multi-event-tail merge loss at `client.rs:336-350` (V02-01) extends
  F-LLM-01-P1-01's severity; fix ownership remains with the transport task.
- `anthropic.rs` internals not reviewed (F-LLM-03), so the cache_hints
  contrast (P2-01) only confirms the consumer side.
- No `echo-agent-cli` source was modified; EKO call sites cited for
  reachability only.

## Handoff

- Downstream tasks may rely on: adapter is a thin, panic-free pass-through
  (V01/V03); streaming usage delivery verified (V02); all echo_integration
  tests green at the reviewed commit (V04-01..03); F-LLM-01's adapter-boundary
  findings confirmed with one severity amplification (V05).
- F-LLM-01/transport: malformed-chunk fixtures remain missing at
  `parse_sse_chunk`; V02-01's multi-event tail merge is an extra severity
  signal for P1-01.
- F-RCT-03: streaming `finish_reason` is dropped by the core loop (no
  truncation signal); consider it when reviewing run termination.
- X-BND-01: no new cross-repository questions; P2-01's "documented ignore"
  decision can be folded into the raw-contract settlement.
- A-* (EKO usage display): non-streaming `direct_answer` usage undercount
  (F-LLM-01-P2-02) is unaffected by the adapter, which delivers usage in both
  modes.
- This report becomes stale if the wire types gain
  `max_completion_tokens`, if `ChatCompletionRequest`/`ChatChunk` mapping
  changes, or if the transport error handling changes.
