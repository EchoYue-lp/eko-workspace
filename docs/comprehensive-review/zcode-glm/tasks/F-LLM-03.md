# F-LLM-03: Anthropic provider adapter and prompt cache

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable
> Worktree state: clean

## Question

Does the Anthropic adapter preserve the same contract, including thinking
blocks and cache-control behavior?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-integration/src/providers/anthropic.rs` — `AnthropicClient`
  (`LlmClient` impl), `convert_request`, `convert_response`, the streaming
  `chat_stream` event loop, the `AnthropicRequest` / `AnthropicMessage` /
  `ContentBlock` / `AnthropicStreamEvent` wire types, the
  `build_anthropic_thinking` resolver, `apply_conversation_cache_breakpoints`,
  `file_to_content_block`, `data_url_to_image_source`.
- `echo-agent/echo-integration/src/providers/anthropic_cache.rs` —
  `AnthropicCachePlan::from_layout`, `from_layout_or_default`,
  `history_breakpoint_count`, `has_*` accessors.
- `echo-agent/echo-integration/src/providers/thinking_translate.rs` — read in
  full; confirmed it explicitly does NOT cover Anthropic (the module doc and
  the `AnthropicEffort | AnthropicThinkingBudget` arm at `:141-145` both
  return the empty default and defer to `anthropic.rs::build_anthropic_thinking`).
- `echo-agent/echo-integration/src/providers/config.rs` — `LlmConfig::build_client`
  routing for `LlmProvider::Anthropic`, `ProviderFactory::from_provider_model`.
- `echo-agent/echo-core/src/llm/capabilities.rs` — `ProviderCapabilities::anthropic`
  preset, `ModelProfile::new`, `resolve_thinking_protocol`.
- `echo-agent/echo-core/src/llm/thinking.rs` — `ThinkingConfig` translators
  (`to_anthropic_effort`, `to_anthropic_budget`), `ThinkingProtocol` enum.
- `echo-agent/echo-core/src/llm/cache/layout.rs` — `BreakpointTarget`,
  `CacheHints`, `PromptCacheLayout::from_messages`.
- `echo-agent/echo-core/src/llm/types.rs` — `Message`, `MessageContent`,
  `ContentPart`, `Usage` normalization.
- `echo-agent/echo-core/src/error.rs` — `LlmError` variants used by the
  Anthropic path.
- `echo-agent/src/agent/react/run/phases/think.rs:329` — production
  `chat_stream` consumer (confirms the streaming path is live).

## Out Of Scope

- The neutral contract types themselves — audited in **F-LLM-01** and relied
  on here without re-auditing. Specifically: `Message::reasoning_content`
  flat-string limitation (F-LLM-01-P2-02) and `ChatResponse` missing
  top-level `usage` (F-LLM-02-P2-04) are reused as dependencies.
- OpenAI-compat adapter fidelity — audited in **F-LLM-02**. The OpenAI path
  is referenced only comparatively (e.g. `anthropic-beta` header asymmetry
  against the shared transport).
- HTTP mock-based end-to-end tests of `AnthropicClient::chat` /
  `chat_stream` — no such tests exist in the codebase (see V04 coverage gap).
- MCP / IM-channel Anthropic bridging — out of scope.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — framework-vs-application layering gate, dead-code
    cleanup rule (framework API retention test), UTF-8 safety, no-panic rule.
  - `docs/comprehensive-review/REPORTING.md`.
  - `docs/comprehensive-review/templates/task-report.md`,
    `validation-report.md`.
- Dependency task reports read:
  - [F-LLM-01](./F-LLM-01.md) — established that the neutral contract is
    singly defined in `echo-core/src/llm`; that `Usage` normalization
    (`effective_prompt_tokens`, `cached_prompt_tokens`, `cache_hit_rate`) is
    the single authority handling Anthropic-exclusive cache semantics (input
    tokens exclude cached/read/creation); that Anthropic `reasoning_content`
    is never populated (F-LLM-01-P2-01); that the flat
    `reasoning_content: Option<String>` cannot carry Anthropic signed
    thinking blocks (F-LLM-01-P2-02); that `ProviderAdapter` is declaration-
    only.
  - [F-LLM-02](./F-LLM-02.md) — established that `ChatResponse` has no
    top-level `usage` field, forcing non-streaming consumers to read
    `response.raw.usage` (F-LLM-02-P2-04, affects the Anthropic non-streaming
    path identically); that the shared transport `client.rs::post`/
    `stream_post` is the single HTTP/SSE authority for OpenAI-compat
    providers (Anthropic is the one provider that does NOT route through it);
    that `translate_thinking_openai_compat` is the OpenAI-compat thinking
    dispatch and explicitly excludes Anthropic.
  - [F-CORE-01](./F-CORE-01.md) — `LlmError` is the single typed LLM error
    type with no `Other(String)` escape hatch.
- Historical documents treated as hypotheses: none.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `AnthropicClient`, `AnthropicCachePlan`, `build_anthropic_thinking`, and the `ContentBlock` / `AnthropicStreamEvent` wire types are generic Claude-API plumbing any `echo-agent` consumer may use. They correctly live in `echo-integration` alongside the OpenAI-compat client. |
| EKO product policy | None at this layer. No EKO-specific field leaks into the Anthropic adapter. `cache_hints` (provider-neutral) and `user_id` (provider-neutral) are the only request-side fields consumed beyond the standard `messages`/`tools`/`thinking`. |
| Adapter boundary | `AnthropicClient` is a thin translator: neutral `ChatRequest` → Anthropic `/v1/messages` body, Anthropic response / SSE event → neutral `ChatResponse` / `ChatChunk`. It does NOT route through the shared `client.rs::post`/`stream_post` transport — it owns its own HTTP send + retry loop (`anthropic.rs:374-408` non-streaming, `:432-465` streaming). Retry delegates to `with_retry_if(... is_retryable)` (the same retry policy shape as the shared transport), so no retry-loop authority leaks into the adapter. |
| Duplicate search | Searched: `struct AnthropicClient`, `struct AnthropicCachePlan`, `enum ContentBlock`, `enum AnthropicStreamEvent`, `fn build_anthropic_thinking`, `fn convert_request`, `fn convert_response`, `impl LlmClient for AnthropicClient`, `enum AnthropicSystem`, `struct AnthropicUsage`. Result: exactly one public definition each. `AnthropicClient` is `pub use`-exported at three layers (`providers/mod.rs:11`, `echo-agent/src/llm.rs:80, 100`, `echo-agent/src/lib.rs:156`); constructed at exactly one site (`config.rs:309` via `with_base_url`). The Anthropic `ContentBlock` enum is private to `anthropic.rs` (not exported) and is distinct from the contract-level `echo_core::llm::types::ContentPart` (which has only `Text`/`ImageUrl`/`File` variants — no Anthropic-specific blocks). No duplicate definition. |
| Migration deletion | No migration proposed. |

## Current Path

Verified Anthropic data flow at commit `9b0e0fa`:

1. **Provider routing.** `LlmConfig::build_client`
   (`config.rs:302-317`) matches on `LlmProvider::Anthropic` and constructs
   `AnthropicClient::with_base_url(&self.base_url, &self.api_key, &self.model)`.
   `ProviderFactory::from_provider_model` (`config.rs:394-427`) parses
   `provider:model` strings, looks up the base URL (`provider_base_url`), reads
   `ANTHROPIC_API_KEY` from env, and delegates to `build_client`. The
   Anthropic path does NOT go through the shared `client.rs::post`/
   `stream_post` transport — it owns its own reqwest client and SSE parser.

2. **Request construction.** `AnthropicClient::convert_request`
   (`anthropic.rs:69-291`) walks `request.messages`:
   - `Role::System` → extracted to the top-level `system` field (Anthropic
     requires system outside the messages array). Only the LAST system
     message's text survives (see F-LLM-03-P3-01).
   - `Role::Tool` → emitted as a `user`-role message containing a single
     `tool_result` block (`:80-89`).
   - `Role::Assistant` with `tool_calls` → emitted as an `assistant`-role
     message with interleaved `text` + `tool_use` blocks (`:91-117`). Tool
     arguments are re-parsed via `serde_json::from_str(&tc.function.arguments)
     .unwrap_or_default()` so they ride as native JSON, not strings.
   - All other messages (User, plain Assistant, Custom) → content mapped
     through `MessageContent::Parts` → `ContentBlock` translation
     (`:119-151`), with `ContentPart::File` dispatched by inferred media type
     to `file_to_content_block` (`:981-1017`).
   - `temperature`, `max_tokens`, `tools`, `user_id` → forwarded to the
     wire body. `max_tokens` defaults to 4096 when `None` (`:273`).
   - `thinking` → translated by `build_anthropic_thinking` (`:274-275`,
     `:734-776`) into one of `{type:"enabled", budget_tokens:N}` (3.7–4.5),
     `{type:"adaptive"}` + `effort` (4.6), or dropped entirely (Opus 4.7+).
   - `cache_hints` → consumed if non-empty; otherwise `AnthropicCachePlan::
     from_layout` recomputes the plan from the request (`:171-202`).
   - `tool_choice` and `response_format` → **silently dropped** (no field on
     `AnthropicRequest`). See F-LLM-03-P2-04, F-LLM-03-P2-05.

3. **Cache-control placement** (`:159-268`). Builds an `AnthropicCachePlan`,
   then places `cache_control: {type:"ephemeral"}` on:
   - The last tool definition (`:214-218`) when `has_tool_breakpoint`.
   - The system block (`:228-232`) when `has_system_breakpoint`.
   - Up to `4 - used_breakpoints` conversation messages (`:237-268`), via
     either explicit `BreakpointTarget::HistoryIndex` mapping or the
     `apply_conversation_cache_breakpoints` heuristic (75% depth + last
     stable). Runtime-context messages (tagged `[runtime_context:`) are
     excluded.

4. **Non-streaming call.** `LlmClient::chat` (`anthropic.rs:368-419`) sends
   the JSON body with `x-api-key`, `anthropic-version: 2023-06-01`,
   `anthropic-beta: prompt-caching-2024-07-31` headers, retries on
   network/429/5xx via `with_retry_if(... is_retryable)` (`:24-30`), and
   deserializes into `AnthropicResponse`. `convert_response`
   (`:293-364`) iterates `resp.content`, maps `Text`/`ToolUse` blocks, drops
   everything else via `_ => {}` (`:310`), and hard-codes
   `reasoning_content: None` (`:335`). Usage is built from `AnthropicUsage`
   (`:339-350`) carrying `input_tokens`, `output_tokens`,
   `cache_creation_input_tokens`, `cache_read_input_tokens`.

5. **Streaming call.** `LlmClient::chat_stream` (`anthropic.rs:421-630`)
   sets `stream: Some(true)` and sends WITHOUT the `anthropic-beta` header
   (`:440-448`). The event loop polls `cancel_token.is_cancelled()` between
   SSE chunks (`:482-484`) and matches `AnthropicStreamEvent` variants:
   - `MessageStart` captures `input_tokens`, `cache_creation_input_tokens`,
     `cache_read_input_tokens` from `message.usage` (`:514-518`).
   - `ContentBlockStart` with `ToolUse` body inserts an entry into
     `tool_call_args` keyed by `tool_call_args.len()` (`:526-527`); other
     block types fall through to a no-op arm (`:529-531`).
   - `ContentBlockDelta` emits a text `ChatChunk` when `delta.text` is set
     (`:536-546`), or accumulates `partial_json` into `tool_call_args`
     keyed by the event `index` (`:547-552`).
   - `ContentBlockStop` removes `tool_call_args[index]` (event index) and
     emits a `DeltaToolCall` carrying the assembled args (`:554-583`).
   - `MessageDelta` captures `output_tokens` (`:584-588`) and emits the
     final chunk with finish reason + accumulated usage (`:589-617`).
   - Catch-all `_ => {}` (`:618`) drops `ping`, `error`, `message_stop`,
     and any future event type.
   - Hard-coded `reasoning_content: None` on every chunk delta
     (`:541, 566, 611`).

6. **Thinking translation.** `build_anthropic_thinking`
   (`anthropic.rs:734-776`) calls `ModelProfile::new(model, "anthropic",
   ProviderCapabilities::anthropic())` and matches on `thinking_protocol`:
   - `AnthropicThinkingBudget` (3.7–4.5): emits `{type:"enabled",
     budget_tokens:N}` via `cfg.to_anthropic_budget(max_tokens)`; `effort`
     is `None`.
   - `AnthropicEffort` (4.6): emits `{type:"adaptive"}` (no budget) +
     `effort` from `cfg.to_anthropic_effort()`. `Disabled` → no block.
   - `AnthropicAdaptive` (Opus 4.7+): `warn!`s and drops both fields
     (sending anything returns 400).
   - All other protocols: `(None, None)`.

7. **Errors.** Send failure → `LlmError::NetworkError` (`:392, 448`). Non-
   success HTTP → `LlmError::ApiError { status, message }` with the body
   text (`:395-401, 451-457`). Response JSON parse failure →
   `LlmError::NetworkError("Response parse error: ...")` (`:413`). The
   adapter does NOT emit `LlmError::InvalidResponse`, `EmptyResponse`, or
   `SerializationError` on any path — body parse failures are mislabeled as
   `NetworkError`. See F-LLM-03-P3-04.

## Findings

### F-LLM-03-P1-01: Streaming tool-call index/key desync drops tool calls whenever text (or any non-tool-use block) precedes tool_use

- Priority: P1
- Confidence: high
- Layer: framework (adapter streaming path)
- Evidence:
  - `echo-integration/src/providers/anthropic.rs:520-528` —
    `ContentBlockStart { content_block: ToolUse { id, name }, .. }` is
    matched with `..` ignoring the event's `_index` field. The entry is
    inserted at `idx = tool_call_args.len()` — a dense tool counter
    independent of the event's content-block index.
  - `echo-integration/src/providers/anthropic.rs:547-552` —
    `ContentBlockDelta` looks up `tool_call_args.get_mut(&index)` using the
    EVENT's `index` field (the content-block index, which counts text,
    image, thinking, and tool_use blocks alike).
  - `echo-integration/src/providers/anthropic.rs:554-583` —
    `ContentBlockStop` removes `tool_call_args.remove(&index)` using the
    event's `index`, and emits a `DeltaToolCall` carrying
    `index: index as u32`.
  - `echo-integration/src/providers/anthropic.rs:1054-1058` — the
    `ContentBlockStart` event variant names its field `_index: usize`
    (underscore-prefixed to suppress the unused-variable warning),
    confirming the desync is by design oversight, not by accident.
- Reachability: every Anthropic streaming call. The streaming path is live
  via `think.rs:329` (`llm_client.chat_stream(request)`) which is the
  primary think-phase execution mode in the React engine. Any Claude
  response whose content array contains a non-tool-use block before or
  between tool_use blocks triggers the bug. This is the COMMON case in
  agentic flows — Claude Sonnet/Opus typically emits "Let me check..." text
  before the tool call, or interleaves text + multiple tool calls.
- Expected invariant: the streaming adapter must preserve every tool_use
  block and route each block's `partial_json` deltas to the corresponding
  tool entry, regardless of how many non-tool-use blocks the response
  contains. Anthropic's `index` field on `content_block_*` events is the
  content-block index, not the tool-use index; the adapter must use it
  consistently for both insert and lookup.
- Observed behavior: when the response is `[text_block, tool_use_block]`:
  1. `ContentBlockStart(text)` → no-op (catch-all arm).
  2. `ContentBlockStart(tool_use, id=X, name=foo)` → `tool_call_args.len()
     == 0`, so insert at key 0.
  3. `ContentBlockDelta(index=1, partial_json="{")` → lookup
     `tool_call_args[1]` → NOT FOUND → `partial_json` silently dropped.
  4. `ContentBlockDelta(index=1, partial_json="}")` → same, dropped.
  5. `ContentBlockStop(index=1)` → `tool_call_args.remove(&1)` → NOT FOUND
     → NO `DeltaToolCall` emitted.
  6. `MessageDelta` emits `finish_reason="tool_calls"` and usage, but the
     stream carries zero tool calls.
  Result: the React engine sees `finish_reason="tool_calls"` with an empty
  tool-call list. The model's intended tool invocation is silently lost.
  For two-tool responses interleaved with text, the second tool's data is
  also lost (or, depending on ordering, the first tool's args are routed to
  the second tool's slot — a worse correctness outcome).
- Impact: severe. Every Claude streaming agentic flow that emits any text
  alongside tool calls — the dominant case in production — silently loses
  the tool call. The user-visible symptom is "Claude thought about it but
  didn't actually call the tool", which is hard to distinguish from model
  behavior. The bug is not covered by any test (V04).
- Root cause: `idx = tool_call_args.len()` was written assuming the
  response would contain only tool_use blocks (so block index == tool
  count). The code was never updated when text/thinking blocks became part
  of the streaming surface. The `_index` field on `ContentBlockStart` was
  renamed to suppress the unused warning rather than fix the desync.
- Direction: use the event's content-block `index` as the insertion key:
  rename `_index` → `index` on `AnthropicStreamEvent::ContentBlockStart`,
  insert at `tool_call_args.insert(index, (id, name, String::new()))`. The
  existing `get_mut(&index)` and `remove(&index)` calls then line up. The
  emitted `DeltaToolCall.index` should continue to use the event index
  (the downstream React-engine assembler in `processor.rs` aggregates by
  this index and will produce one tool call per index). Add a streaming
  fixture test (currently none exist — see V04) covering
  `[text_block, tool_use_block]` and `[tool_use_A, text_block,
  tool_use_B]`.
- Regression validation: a streaming test that feeds a fixture SSE stream
  containing a text block followed by a tool_use block and asserts the
  emitted `ChatChunk` stream contains exactly one `DeltaToolCall` with the
  correct id/name/args. Today no such test exists, so this regression
  would not be caught.
- Validation reports: [V02](../validations/F-LLM-03/V02-01.md),
  [V04](../validations/F-LLM-03/V04-01.md)

### F-LLM-03-P2-01: Anthropic thinking blocks are not surfaced through the neutral contract (response-side gap)

- Priority: P2
- Confidence: high
- Layer: framework (adapter implementation gap; contract-level note in
  F-LLM-01-P2-01)
- Evidence:
  - `echo-integration/src/providers/anthropic.rs:829-866` — the
    `ContentBlock` enum has variants `Text`, `Image`, `Document`,
    `ToolUse`, `ToolResult`. No `Thinking { thinking, signature }` or
    `RedactedThinking { data }` variant. No `#[serde(other)]` fallback.
  - `echo-integration/src/providers/anthropic.rs:310` — non-streaming
    `convert_response` `_ => {}` catch-all drops anything that is not
    `Text` or `ToolUse`.
  - `echo-integration/src/providers/anthropic.rs:335` — non-streaming
    `Message.reasoning_content: None` (hard-coded).
  - `echo-integration/src/providers/anthropic.rs:541, 566, 611` — every
    streaming `ChatChunk.delta.reasoning_content: None`.
  - `echo-integration/src/providers/anthropic.rs:1088-1093` —
    `ContentDelta` has only `text` and `partial_json` fields. Anthropic's
    `thinking_delta` and `signature_delta` events deserialize to
    `{ text: None, partial_json: None }` and are silently dropped (no
    error, but no content either).
  - `echo-core/src/llm/capabilities.rs:98` —
    `ProviderCapabilities::anthropic().reasoning_content: false` with the
    honest comment "not mapped in this implementation".
- Reachability: every Anthropic chat call (streaming and non-streaming).
  Live in any deployment using Claude. The streaming path degrades
  gracefully (thinking deltas silently dropped); the non-streaming path
  hard-fails if Anthropic returns any content-block type not in the enum
  (see F-LLM-03-P2-02).
- Expected invariant: per F-LLM-01-P2-01, the task asks whether the
  adapter preserves thinking "without semantic loss". The contract types
  already carry `reasoning_content`; the Anthropic adapter never populates
  it. OpenAI-compatible reasoning models (GPT-5, DeepSeek-r1, Qwen3)
  faithfully stream `reasoning_content` deltas; Anthropic models do not.
- Observed behavior: an observer consuming `ChatChunk` cannot tell whether
  a Claude model thought at all. UI thinking panels are blank for Claude,
  populated for OpenAI-family reasoning models.
- Impact: asymmetry in the otherwise-uniform streaming surface. The
  `reasoning_content: false` capability encodes this gap as if it were a
  permanent provider property rather than a temporary implementation gap.
- Root cause: the Anthropic adapter predates extended-thinking output on
  the Claude API; the `ContentBlock` enum and the response/stream mappers
  were never extended.
- Direction: add a `Thinking { thinking: String, signature: Option<String> }`
  variant to the adapter's private `ContentBlock` enum, map it to
  `Message.reasoning_content` (joining text) on the non-streaming path,
  emit a `ChatChunk` with `reasoning_content: Some(thinking_delta)` on the
  streaming path (and accumulate signature separately if a future
  round-trip is needed — see F-LLM-03-P3-02), and flip
  `ProviderCapabilities::anthropic().reasoning_content` to `true`. The
  contract change to carry signatures (F-LLM-01-P2-02) is a separate,
  larger design question; this finding only covers the observational gap.
- Regression validation: a non-streaming fixture test that deserializes an
  Anthropic response containing `{"type":"thinking","thinking":"..."}`
  blocks and asserts the resulting `ChatResponse.message.reasoning_content`
  is `Some`. A streaming fixture test asserting `thinking_delta` events
  produce `ChatChunk.delta.reasoning_content: Some`. Today neither exists.
- Validation reports: [V02](../validations/F-LLM-03/V02-01.md),
  [V04](../validations/F-LLM-03/V04-01.md)

### F-LLM-03-P2-02: Non-streaming response deserialization hard-fails on any unknown content-block type (including thinking, redacted_thinking, and future Anthropic blocks)

- Priority: P2
- Confidence: high
- Layer: framework (adapter wire types)
- Evidence:
  - `echo-integration/src/providers/anthropic.rs:827-866` — `ContentBlock`
    is `#[serde(tag = "type")]` with five renamed variants. There is no
    `#[serde(other)]` arm and no per-variant `#[serde(other)]` fallback.
    Unknown `type` values (e.g. `thinking`, `redacted_thinking`,
    `container_upload`, any future block) cause `ContentBlock::deserialize`
    to return `Err`, which propagates up through `Vec<ContentBlock>` and
    `AnthropicResponse` and surfaces as
    `LlmError::NetworkError("Response parse error: ...")` at
    `anthropic.rs:413`.
  - By contrast, `AnthropicStreamEvent` (`:1048-1071`) and
    `ContentBlockStartBody` (`:1079-1086`) both have `#[serde(other)]`
    fallback arms, so the streaming path is resilient to unknown event and
    block types.
- Reachability: every non-streaming Anthropic call where the response
  contains a content-block type not in the enum. Triggered today by:
  - Claude 4.6 (`AnthropicEffort` protocol) with thinking config set →
    `build_anthropic_thinking` emits `{type:"adaptive"}` → API may return
    `thinking` blocks → deserialization fails.
  - Claude 3.7–4.5 (`AnthropicThinkingBudget`) with a `Level`/`BudgetTokens`
    config → API returns `thinking` blocks → deserialization fails.
  - Any future Anthropic block type (the API has added several over time).
- Expected invariant: a provider adapter should degrade gracefully on
  unknown response fields (the streaming path already does). A single
  unknown block type should not cause the entire response to be
  undeliverable.
- Observed behavior: when the bug triggers, the caller sees a generic
  `NetworkError` with a serde message — there is no signal that the cause
  is an unknown content block, no partial response is delivered, and the
  retry policy treats it as retryable (`is_retryable` returns true for
  `NetworkError`), so the call retries uselessly against an unchanged
  response shape.
- Impact: thinking-enabled Claude calls on the non-streaming path silently
  retry-then-fail. Combined with F-LLM-03-P2-01, the adapter is currently
  unable to handle thinking-enabled Claude responses on EITHER path
  (streaming drops them, non-streaming crashes on them).
- Root cause: the streaming types were written defensively with
  `#[serde(other)]`; the non-streaming `ContentBlock` was not.
- Direction: add `#[serde(other)] Other` to `ContentBlock` (and update the
  non-streaming match arm to drop it). Combined with the F-LLM-03-P2-01
  fix (add an explicit `Thinking` variant), this gives forward
  compatibility with future Anthropic block types. Under AGENTS.md
  "no backward compatibility burden", this is a safe in-place change.
- Regression validation: a non-streaming fixture test that deserializes a
  response containing an unknown block type and asserts it does NOT error
  (the block is dropped, the rest of the content survives).
- Validation reports: [V02](../validations/F-LLM-03/V02-01.md),
  [V04](../validations/F-LLM-03/V04-01.md)

### F-LLM-03-P2-03: Streaming path omits the `anthropic-beta: prompt-caching-2024-07-31` header, making cache-control behavior inconsistent across paths

- Priority: P2
- Confidence: high
- Layer: framework (adapter HTTP send)
- Evidence:
  - `echo-integration/src/providers/anthropic.rs:383-392` — non-streaming
    `chat()` send: includes `.header("anthropic-beta",
    "prompt-caching-2024-07-31")`.
  - `echo-integration/src/providers/anthropic.rs:440-448` — streaming
    `chat_stream()` send: includes only `x-api-key`,
    `anthropic-version`, `content-type`. The `anthropic-beta` header is
    absent.
  - The request body built by `convert_request` is shared between both
    paths (`:372` non-streaming, `:428` streaming) and emits
    `cache_control: {type:"ephemeral"}` markers per `AnthropicCachePlan`
    on both paths identically.
- Reachability: every streaming Anthropic call. The streaming path is
  live via `think.rs:329` (the React engine's primary think-phase path).
- Expected invariant: the cache-control markers in the request body should
  be honored identically on both paths. Anthropic's prompt-caching API
  requires the beta header on API versions where caching is still gated
  (the `2024-07-31` date in the header string indicates this code was
  written when the beta header was required). Even now that caching is GA
  for stable customers, the beta header selects the caching beta behavior
  on some account tiers.
- Observed behavior: streaming calls send `cache_control` markers in the
  body but no beta header. Depending on Anthropic API version and account
  tier, this either (a) silently disables caching on the streaming path
  (markers ignored, no cache writes/reads reported in usage), or (b)
  returns a 400 error complaining about the cache_control fields without
  the beta opt-in. Either way the streaming path's cache behavior
  diverges from the non-streaming path's.
- Impact: cache hit-rate observability for streaming Claude flows is
  unreliable; on strict API accounts streaming may 400 outright. The
  non-streaming path also sends the prompt-caching beta header but NOT
  the thinking beta header (`thinking-2025-04-15` or similar), so
  thinking-enabled calls may also fail there. The header set is
  inconsistent across paths AND incomplete for the thinking feature on
  both paths.
- Root cause: the streaming send block was likely copy-pasted from the
  non-streaming send and the beta header line was dropped during the
  paste; the inconsistency was never noticed because no streaming test
  exercises the HTTP send path.
- Direction: extract the header set into a shared helper
  (`fn anthropic_headers() -> Vec<(&'static str, &'static str)>` or
  similar) and call it from both send sites. While there, evaluate
  whether the thinking beta header is also needed when
  `body.thinking.is_some()` — if so, add it conditionally. Under AGENTS.md
  UTF-8/no-panic rules, the helper must not panic on missing values.
- Regression validation: a test that builds both `chat()` and
  `chat_stream()` requests and asserts the HTTP header sets are identical
  (or differ only by intentional, documented per-path additions). The
  current lack of HTTP-mock tests (see V04) is the reason this slipped.
- Validation reports: [V03](../validations/F-LLM-03/V03-01.md),
  [V04](../validations/F-LLM-03/V04-01.md)

### F-LLM-03-P2-04: `ChatRequest.tool_choice` is silently dropped — no `tool_choice` field on the Anthropic wire body

- Priority: P2
- Confidence: high
- Layer: framework (adapter request mapping)
- Evidence:
  - `echo-integration/src/providers/anthropic.rs:673-705` —
    `AnthropicRequest` has fields `model`, `max_tokens`, `system`,
    `messages`, `temperature`, `tools`, `stream`, `thinking`, `effort`,
    `metadata`. No `tool_choice`.
  - `echo-integration/src/providers/anthropic.rs:69-291` —
    `convert_request` never reads `request.tool_choice`.
  - `echo-agent/src/agent/react/run/phases/think.rs:317-318` — the
    production caller explicitly sets
    `tool_choice: Some("none".to_string())` on the final-think-only
    request when `supports_tool_choice_none` is true. For Anthropic,
    `supports_tool_choice_none` is `false`
    (`capabilities.rs:106`), so this specific caller is gated off — but
    other callers may set `tool_choice` without checking the capability.
- Reachability: every Anthropic call where the caller sets
  `ChatRequest.tool_choice`. Today the only production caller
    (`think.rs`) is gated by `supports_tool_choice_none` so it does not
    set the field for Anthropic, but the contract accepts the field and
    any future caller can set it.
- Expected invariant: a neutral contract field that the caller sets should
  either be translated to the provider's wire format or fail loudly.
  Anthropic's `/v1/messages` API supports
  `tool_choice: {"type":"auto"|"any"|"tool","name":"..."}` natively, so
  translation is feasible.
- Observed behavior: the field is silently dropped. A caller setting
  `tool_choice = "required"` expecting Anthropic to force a tool call
  sees no error and no effect.
- Impact: today, low (the production caller is gated off). The risk is
  future callers (or callers that bypass the capability gate) setting
  `tool_choice` for Anthropic and getting silent no-op. Also affects
  F-LLM-01-P3-01's typed-enum migration: the migration should add
  Anthropic translation, not perpetuate the silent drop.
- Root cause: the Anthropic adapter was written before the contract grew
  the `tool_choice` field; the field was never wired through.
- Direction: add a `tool_choice: Option<AnthropicToolChoice>` field to
  `AnthropicRequest`, and translate the OpenAI-shaped string
  (`"auto"`/`"none"`/`"required"`/JSON object) to Anthropic's
  `{type:"auto"|"any"|"tool"}` shape. Note Anthropic has no `"none"`
  equivalent (it has no `supports_tool_choice_none`), so `"none"` must
  either be dropped with a `warn!` or — better — rejected up front by
  the caller checking `ProviderCapabilities.supports_tool_choice_none`
  (which `think.rs` already does). Coordinate with F-LLM-01-P3-01's
  typed `ToolChoice` migration.
- Regression validation: a test that builds a `ChatRequest` with
  `tool_choice = "required"` for an Anthropic model, serializes the
  resulting `AnthropicRequest`, and asserts the body contains
  `"tool_choice":{"type":"any"}`.
- Validation reports: [V01](../validations/F-LLM-03/V01-01.md)

### F-LLM-03-P2-05: `ChatRequest.response_format` is silently dropped — Anthropic's structured-output mechanism is not translated

- Priority: P2
- Confidence: medium
- Layer: framework (adapter request mapping)
- Evidence:
  - `echo-integration/src/providers/anthropic.rs:673-705` —
    `AnthropicRequest` has no `response_format` field.
  - `echo-integration/src/providers/anthropic.rs:69-291` —
    `convert_request` never reads `request.response_format`.
  - `echo-core/src/llm/capabilities.rs:103` —
    `ProviderCapabilities::anthropic().structured_output: false` with the
    implicit understanding that Anthropic has no JSON mode.
- Reachability: every Anthropic call where the caller sets
  `ChatRequest.response_format`. No current production caller does so for
  Anthropic (the React engine does not populate `response_format`).
- Expected invariant: same as F-LLM-03-P2-04 — a set contract field
  should either translate or fail loudly.
- Observed behavior: `response_format` is silently dropped. Anthropic's
  Messages API does not have a JSON-mode field, but it does support
  structured output via tool-based forced schema (define a tool with the
  desired schema, force `tool_choice` to that tool) — this is the
  documented Anthropic pattern for structured output. The adapter does
  not perform this translation.
- Impact: today, none (no caller sets the field for Anthropic). The risk
  is future callers expecting structured output on Claude and getting
  free-form text. Lower priority than F-LLM-03-P2-04 because structured
  output is a more involved translation (tool-synthesizing) and the
  capability flag already declares `structured_output: false`.
- Root cause: the Anthropic API has no native JSON mode; the contract's
  `response_format` was modeled on OpenAI's shape; no translation was
  written.
- Direction: either (a) document on `ProviderCapabilities` that
  `structured_output: false` means `response_format` is dropped, and emit
  a `debug!` log when the field is set on such a provider; or (b) implement
  the Anthropic tool-based structured-output translation (synthesize a
  tool whose schema is `response_format.json_schema.schema`, force
  `tool_choice` to that tool, parse the resulting `tool_use.input` as the
  structured output). Option (a) is cheaper; option (b) is more useful.
- Regression validation: doc-only change requires no test; if (b), add an
  end-to-end test that asserts the synthesized tool round-trips.
- Validation reports: [V01](../validations/F-LLM-03/V01-01.md)

### F-LLM-03-P3-01: Multiple `Role::System` messages collapse to the last; earlier system messages are silently dropped

- Priority: P3
- Confidence: high
- Layer: framework (adapter request mapping)
- Evidence:
  - `echo-integration/src/providers/anthropic.rs:73-77` —
    ```
    for msg in &request.messages {
        if msg.role == Role::System {
            system = msg.content.as_text();
            continue;
        }
    ```
    The loop overwrites `system` on each System message; only the last
    survives. No aggregation.
  - `echo-core/src/llm/cache/layout.rs:74-77` — the layout treats the
    System segment as a sequence (`position` to find segment end), so the
    contract-level cache layout DOES expect multiple system messages to
    exist. The adapter drops them after the layout was computed.
- Reachability: every Anthropic call with more than one System message in
  `request.messages`. The React engine typically sends one system
  message, but the contract allows many.
- Expected invariant: all System messages should be preserved (e.g., as
  multiple `text` blocks inside the top-level `system` array, which
  Anthropic supports natively).
- Observed behavior: if a caller builds `[system("persona"), system("tool
  guidance"), user("...")]`, only "tool guidance" reaches the API. The
  persona text is silently lost.
- Impact: low today (single-system-message callers are unaffected). Affects
  callers that compose the system prompt from multiple messages (a
  reasonable pattern).
- Root cause: the simplest possible mapping (overwrite) was chosen; the
  `AnthropicSystem::Blocks` variant already supports an array, so
  aggregation is a one-line change.
- Direction: aggregate System messages into a `Vec<SystemBlock>` and emit
  `AnthropicSystem::Blocks(blocks)`. Apply `cache_control` to the last
  block when `has_system_breakpoint`.
- Regression validation: a test that builds `[system("a"), system("b")]`,
  converts the request, and asserts both texts appear in the serialized
  `system` array.
- Validation reports: [V01](../validations/F-LLM-03/V01-01.md)

### F-LLM-03-P3-02: Signed thinking blocks cannot round-trip on the request side (no `Thinking` variant in request `ContentBlock`)

- Priority: P3
- Confidence: high
- Layer: framework (contract + adapter; design question)
- Evidence:
  - `echo-integration/src/providers/anthropic.rs:827-866` — the
    `ContentBlock` enum (used for both request and response sides) has no
    `Thinking` variant. A caller wishing to replay a prior assistant turn
    that included thinking blocks (the Anthropic multi-turn extended-
    thinking protocol requires this) has no way to attach the thinking
    block on the request side.
  - `echo-core/src/llm/types.rs:251` — `Message::reasoning_content:
    Option<String>` is a flat string and cannot carry the per-block
    `signature`.
  - See F-LLM-01-P2-02 for the contract-level framing. This finding
    confirms the adapter has the same gap on its private wire types.
- Reachability: not reachable today (F-LLM-03-P2-01 means thinking is
  never populated on the response side either, so there is nothing to
  round-trip). Becomes reachable once P2-01 is fixed and a caller wants
  multi-turn thinking-with-tool-use on Claude.
- Expected invariant: Anthropic extended-thinking protocol requires the
  prior turn's thinking blocks (text + signature) to be replayed
  verbatim in the next request when interleaving thinking with tool use.
  Without round-trip support, the multi-turn thinking flow cannot work.
- Observed behavior: no mechanism exists to attach a thinking block to an
  outgoing assistant `Message`.
- Impact: deferred — does not affect any current flow because P2-01 masks
  it. Once P2-01 is fixed, this becomes the next bottleneck.
- Root cause: same as F-LLM-01-P2-02 — the contract was designed around
  OpenAI/Qwen3/DeepSeek opaque reasoning text; Anthropic's signed-block
  model was not represented.
- Direction: coordinate with the F-LLM-01-P2-02 contract change (introduce
  a neutral `ThinkingBlock { text, signature }` or extend `ContentPart`
  with a `Thinking` variant). The adapter's `ContentBlock::Thinking`
  variant then serializes naturally to Anthropic's
  `{"type":"thinking","thinking":text,"signature":sig}`. Keep
  `reasoning_content` as a convenience text-only accessor.
- Regression validation: a round-trip test that receives an Anthropic
  thinking block, echoes it back in the next request, and asserts the
  signature survives unchanged.
- Validation reports: [V02](../validations/F-LLM-03/V02-01.md)

### F-LLM-03-P3-03: Body parse failures are mislabeled as `LlmError::NetworkError` instead of `LlmError::InvalidResponse`

- Priority: P3
- Confidence: high
- Layer: framework (adapter error mapping)
- Evidence:
  - `echo-integration/src/providers/anthropic.rs:410-413` — non-streaming
    response JSON deserialization failure:
    ```
    let anthropic_resp: AnthropicResponse = resp
        .json()
        .await
        .map_err(|e| LlmError::NetworkError(format!("Response parse error: {e}")))?;
    ```
  - `echo-core/src/error.rs:87-108` — `LlmError` has an
    `InvalidResponse(String)` variant specifically for this class of
    failure (per F-CORE-01).
  - `echo-integration/src/providers/anthropic.rs:24-30` — `is_retryable`
    returns true for `NetworkError` and false for `InvalidResponse`. So
    the mislabeling causes parse failures to be RETRIED against an
    unchanged response shape (the same JSON will fail the same way next
    time).
- Reachability: every non-streaming Anthropic call whose response body
  fails to deserialize as `AnthropicResponse`. Triggered by the
  F-LLM-03-P2-02 path (unknown content-block type) and by any transport-
  level body corruption.
- Expected invariant: body parse failures should be `InvalidResponse` so
  the retry policy does not waste calls on them.
- Observed behavior: parse failures retry up to `RetryPolicy::default`
  attempts, then surface as `NetworkError` to the caller. The caller has
  no typed signal that the cause was a parse failure (vs an actual
  network failure).
- Impact: low (retries are bounded). Degrades observability and wastes
  rate-limit budget on unparseable responses.
- Root cause: the error mapping was written before `InvalidResponse` was
  the canonical variant for this class; the mapping was not updated.
- Direction: change `:413` to
    `.map_err(|e| LlmError::InvalidResponse(format!("Response parse error: {e}")))?`.
  Add a regression test asserting a malformed body does not retry.
- Regression validation: unit test feeding a malformed JSON body and
  asserting the resulting error is `LlmError::InvalidResponse` and that
  `is_retryable` returns false.
- Validation reports: [V04](../validations/F-LLM-03/V04-01.md)

### F-LLM-03-P3-04: `ChatResponse` exposes usage only via `raw.usage` on the Anthropic non-streaming path (asymmetry with streaming `ChatChunk.usage`)

- Priority: P3
- Confidence: high
- Layer: framework (contract observation; affects Anthropic path
  identically to OpenAI)
- Evidence:
  - `echo-integration/src/providers/anthropic.rs:352-363` —
    `ChatResponse { message, finish_reason, raw: ChatCompletionResponse
    { ..., usage, ... } }`. No top-level `usage`.
  - `echo-integration/src/providers/anthropic.rs:595-606` — the streaming
    final chunk emits `usage: Some(Usage { ... })` at the `ChatChunk`
    top level.
- Reachability: every non-streaming Anthropic call.
- Expected invariant: the streaming and non-streaming surfaces of the
  same contract should expose usage at the same level.
- Observed behavior: same as F-LLM-02-P2-04 — non-streaming consumers
  must read `response.raw.usage`, streaming consumers read
  `chunk.usage`. Asymmetric.
- Impact: ergonomic defect. The OpenAI-compat path has the same
  asymmetry (F-LLM-02-P2-04); fixing the contract fixes both.
- Root cause: same as F-LLM-02-P2-04.
- Direction: add `pub usage: Option<Usage>` to `ChatResponse`, populate
  from `raw.usage.clone()` in both adapters. This is the contract change
  already proposed in F-LLM-02-P2-04; flagged here to confirm it covers
  Anthropic.
- Regression validation: `cargo test --workspace --all-features`; update
  the few callers that read `response.raw.usage` to read
  `response.usage`.
- Validation reports: [V03](../validations/F-LLM-03/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Request/response field mapping (ChatRequest → Anthropic wire body, both paths) | yes | passed (with `tool_choice`/`response_format`/multi-system drops noted) | [V01-01](../validations/F-LLM-03/V01-01.md) |
| V02 | Thinking-block and stream-event translation (thinking dropped on both paths; signed round-trip absent) | yes | passed (with P1 streaming tool-index desync and P2 thinking-drop noted) | [V02-01](../validations/F-LLM-03/V02-01.md) |
| V03 | Cache-control placement and cache usage accounting (Anthropic-exclusive normalization) | yes | passed (with streaming beta-header gap noted) | [V03-01](../validations/F-LLM-03/V03-01.md) |
| V04 | Protocol fixture tests + executable check (Anthropic test inventory, cargo clippy/test) | yes | passed (with fixture coverage gap noted) | [V04-01](../validations/F-LLM-03/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | No historical document is cited as evidence. No prior F-LLM-03 report exists in this reviewer directory. |

## Historical Claim Status

No historical documents are cited as evidence for any claim in this
report. All findings are based on code at commit `9b0e0fa` and the four
validation reports above. The F-LLM-01 conclusions reused as dependencies
(`reasoning_content` gap, signed-thinking limitation, `Usage` normalization
authority, `ProviderAdapter` declaration-only) are current at the cited
echo-core and echo-integration line anchors.

## Coverage And Uncertainty

- Code not inspected in depth: the `ProviderFactory` provider-routing
  logic (`config.rs`) was inspected only enough to confirm Anthropic
  routes through `AnthropicClient` (not the shared transport). The full
  `ProviderFactory` parsing and env-var lookup logic was not exhaustively
  audited (belongs to a configuration task).
- The OpenAI-compat adapter was inspected only comparatively (e.g. for
  the `anthropic-beta` header inconsistency). Full OpenAI adapter
  fidelity is the subject of F-LLM-02.
- No HTTP-mock-based end-to-end test of `AnthropicClient::chat` /
  `chat_stream` exists in the codebase. The existing tests exercise only
  `convert_request` (cache_control placement, metadata, file dispatch)
  and `apply_conversation_cache_breakpoints` / `AnthropicCachePlan`. Live
  request/response fidelity to the Anthropic API is verified by static
  inspection only, not by executable fixture. This is the reason
  F-LLM-03-P1-01 (streaming tool desync), F-LLM-03-P2-02 (unknown block
  parse failure), and F-LLM-03-P2-03 (missing beta header) slipped
  through.
- The streaming tool-index desync (F-LLM-03-P1-01) is inferred from
  static analysis of the event-loop match arms and confirmed by the
  absence of any covering test. It is not exercised against a live
  Anthropic API in this task; the impact trace is constructed from
  Anthropic's documented event ordering. Confidence is high because the
  desync is visible directly in the source (`tool_call_args.len()` vs
  event `index`).
- The `build_anthropic_thinking` path emits the thinking block in the
  body but neither path sends the `thinking-2025-04-15` beta header. It
  is unclear whether the Anthropic API returns thinking blocks in the
  response without that header for Claude 4.6 / 4.7+ models. If it does
  not, then F-LLM-03-P2-02 (non-streaming parse failure on thinking
  blocks) is currently latent rather than active; if it does (or if
  Anthropic changes the gating), the parse failure becomes active.
  Either way, the adapter is not robust to thinking-block responses.
- Environmental limits: `cargo test -p echo_integration --lib providers::`
  (62 tests, 0 failures), `cargo clippy -p echo_integration --all-targets
  --all-features --locked -- -D warnings` (0 warnings) both pass. The
  full workspace test suite and feature matrix were not run (out of scope
  for this adapter-fidelity task; they belong to the commit gate).
- Claims that remain uncertain:
  - Whether any third-party `echo-agent` consumer outside this monorepo
    constructs `AnthropicClient` directly (rather than via
    `ProviderFactory`). The pub-export at three layers suggests it is a
    reasonable external API surface.
  - Whether the `cache_creation_input_tokens` / `cache_read_input_tokens`
    fields appear in Anthropic's `message_delta.usage` (newer API
    versions) in addition to `message_start.usage`. The adapter reads
    them only from `message_start`; if they appear in `message_delta`
    for some account tier, the streaming cache-accounting would miss
    them. The non-streaming path reads from the merged `AnthropicUsage`
    and is unaffected.

## Handoff

- Conclusions downstream tasks may rely on:
  - The Anthropic adapter is a thin translator with its own HTTP/SSE
    path (does NOT use the shared `client.rs::post`/`stream_post`
    transport). Cache-control placement is delegated to
    `AnthropicCachePlan` (the unit of strategy) and the
    `convert_request` body builder (the unit of wire emission); both are
    singly defined and unit-tested for cache breakpoints.
  - `build_anthropic_thinking` (`anthropic.rs:734-776`) is the single
    Anthropic thinking translator; `thinking_translate.rs` explicitly
    excludes Anthropic (`:141-145`). The two are not in conflict.
  - The Anthropic `Usage` extraction (input/cache at `MessageStart`,
    output at `MessageDelta`) feeds the F-LLM-01 `Usage` normalization
    authority correctly on both paths; the Anthropic-exclusive cache
    semantics (input tokens exclude cached/read/creation) is handled by
    `Usage::effective_prompt_tokens` without adapter-side duplication.
  - `ProviderCapabilities::anthropic().reasoning_content: false` is an
    adapter-implementation statement, not a permanent provider property;
    F-LLM-03-P2-01 tracks the fix.
- Reports they must read:
  - [V01-01](../validations/F-LLM-03/V01-01.md) for the field-by-field
    request mapping table (including the `tool_choice` /
    `response_format` / multi-system drops).
  - [V02-01](../validations/F-LLM-03/V02-01.md) for the streaming tool
    desync trace and the thinking-block drop analysis.
  - [V03-01](../validations/F-LLM-03/V03-01.md) for the cache breakpoint
    strategy and the streaming beta-header inconsistency.
  - [V04-01](../validations/F-LLM-03/V04-01.md) for the test inventory
    and the executable clippy/test result.
- Conditions that make this report stale:
  - Any fix to the streaming tool-index desync (rename `_index` → `index`
    and use it as the insertion key) invalidates F-LLM-03-P1-01.
  - Any addition of a `Thinking` variant to the adapter `ContentBlock`
    enum invalidates F-LLM-03-P2-01 and F-LLM-03-P2-02.
  - Any addition of the `anthropic-beta` header to the streaming send
    invalidates F-LLM-03-P2-03.
  - Any addition of `tool_choice` / `response_format` to the
    `AnthropicRequest` body invalidates F-LLM-03-P2-04 / F-LLM-03-P2-05.
  - Any change to `LlmError` mapping at `anthropic.rs:413` to use
    `InvalidResponse` invalidates F-LLM-03-P3-03.
  - Any addition of a top-level `usage` field to `ChatResponse`
    invalidates F-LLM-03-P3-04.
- Follow-up task IDs (no fixes implemented in this review):
  - A future **contract evolution** task should pick up the F-LLM-01-P2-02
    / F-LLM-03-P3-02 design (signed thinking round-trip) together with
    the F-LLM-03-P2-01 adapter fix, since both touch thinking
    preservation.
  - A future **observability** task should add HTTP-mock fixture tests
    for the Anthropic streaming path; the absence of such tests is the
    root cause of F-LLM-03-P1-01, P2-02, and P2-03 slipping through.
  - The F-LLM-01-P3-01 typed `ToolChoice` migration should be done
    together with F-LLM-03-P2-04 so the typed enum has a translation
    target on the Anthropic side from day one.
