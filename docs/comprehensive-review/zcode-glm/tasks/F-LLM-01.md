# F-LLM-01: Provider-neutral LLM contract

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable
> Worktree state: clean

## Question

Can provider implementations preserve messages, tools, thinking, usage,
caching, streaming, cancellation, and errors without semantic loss?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-core/src/llm/mod.rs` — `LlmClient` trait, `ChatRequest`,
  `ChatResponse`, `ChatChunk`, `ToolChoice`, `SimpleChatOptions`.
- `echo-agent/echo-core/src/llm/types.rs` — `Message`, `MessageContent`,
  `ContentPart`, `Role`, `ToolCall`, `ToolDefinition`, `ResponseFormat`,
  `Usage`, `TokenUsageDetails`, `ChatCompletionRequest`,
  `ChatCompletionResponse`, streaming types (`ChatCompletionChunk`,
  `DeltaMessage`, `DeltaToolCall`).
- `echo-agent/echo-core/src/llm/capabilities.rs` — `ProviderCapabilities`,
  `ModelProfile`, `ModelProfileResolver`, `CachePolicy`,
  `resolve_thinking_protocol`.
- `echo-agent/echo-core/src/llm/thinking.rs` — `ThinkingConfig`,
  `ThinkingLevel`, `ThinkingProtocol`, per-vendor translators.
- `echo-agent/echo-core/src/llm/cache/` — `CacheHints`, `BreakpointTarget`,
  `PromptCacheLayout`, `stable_prefix_hash`, `PromptCacheFingerprint`.
- `echo-agent/echo-core/src/error.rs` — `LlmError` hierarchy.
- `echo-agent/echo-integration/src/providers/traits.rs` — `ProviderAdapter`,
  `ThinkingProtocolPreference`, `resolve_base_url`.
- `echo-agent/echo-integration/src/providers/config.rs` — `ProviderFactory`.
- Cross-checks: `echo-integration/src/providers/openai.rs`,
  `anthropic.rs`, `client.rs`, `adapter_client.rs`, `thinking_translate.rs`.

## Out Of Scope

- Concrete OpenAI adapter fidelity (request field mapping, delta assembly,
  tool-call edge cases) — deferred to **F-LLM-02**.
- Concrete Anthropic adapter fidelity (interleaved content blocks,
  cache-control placement, thinking-block population) — deferred to
  **F-LLM-03**. The present report references the Anthropic adapter only to
  test whether the neutral *contract* permits lossless thinking preservation;
  it does not audit the adapter's own correctness.
- MCP / IM-channel LLM bridging — deferred to integration tasks.
- Tokenizer accuracy (`tokenizer_name` is only named, not implemented here).

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — framework-vs-application layering gate, dead-code
    cleanup rule, UTF-8 safety, no-panic rule.
  - `docs/comprehensive-review/REPORTING.md`.
  - `docs/comprehensive-review/templates/task-report.md`,
    `validation-report.md`.
- Dependency task reports read:
  - [F-CORE-01](./F-CORE-01.md) — established that `ReactError::Llm(Box<
    LlmError>)` is the single framework error type with one definition site,
    and that `LlmError` is a typed sub-enum (not `Other(String)`). This report
    relies on that conclusion and does not re-audit the error hierarchy.
- Historical documents treated as hypotheses: none.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `LlmClient`, the request/response/chunk types, `Message`, `Usage`, `ThinkingConfig`, `ProviderCapabilities`, `CachePolicy`, `ProviderAdapter` are generic agent-runtime concepts any `echo-agent` consumer needs. They correctly live in `echo-core` and `echo-integration`. V01 confirms single definition sites; no duplicate public contract type exists in either repo. |
| EKO product policy | None at this layer. No EKO-specific field leaks into the neutral contract (`user_id`, `cache_hints`, `cancel_token` are all provider-generic). |
| Adapter boundary | The `ProviderAdapter` trait is a declaration-only surface (identity, endpoint, auth, thinking protocol, cache policy, one `prepare_request` hook). `ProviderFactory` returns `Box<dyn LlmClient>`. No scheduling, retry-loop, or state authority lives at the adapter (V04). Conversion is thin. |
| Duplicate search | Searched: `trait LlmClient`, `struct ChatRequest`, `struct ChatResponse`, `struct ChatChunk`, `pub struct Message`, `enum Role`, `enum ContentPart`, `enum ThinkingConfig`, `enum ThinkingProtocol`, `struct Usage`, `struct ProviderCapabilities`, `trait ProviderAdapter`, `struct ProviderFactory`. Result: exactly one public definition each, all in `echo-core` or `echo-integration`. `ThinkingProtocolPreference` (transport, `traits.rs:96`) and `ThinkingProtocol` (framework, `thinking.rs:270`) model overlapping concepts at different layers — see F-LLM-01-P3-02. |
| Migration deletion | No migration proposed. |

## Current Path

Verified neutral-contract data flow at commit `9b0e0fa`:

1. **Request construction.** A caller builds a `ChatRequest`
   (`mod.rs:149-199`) carrying messages, tools, `tool_choice`,
   `response_format`, `thinking: Option<ThinkingConfig>`, `cancel_token`,
   `user_id`, and `cache_hints`. `ProviderFactory::create`
   (`config.rs:371-378`) returns a `Box<dyn LlmClient>`; the caller is
   oblivious to which provider is wired.

2. **Non-streaming call.** `LlmClient::chat(&self, ChatRequest) ->
   BoxFuture<Result<ChatResponse>>` (`mod.rs:55`). Provider implementation
   translates the neutral request into its native body, executes, and returns
   `ChatResponse { message, finish_reason, raw }` (`mod.rs:202-210`).

3. **Streaming call.** `LlmClient::chat_stream` (`mod.rs:65-68`) returns
   `BoxFuture<Result<BoxStream<'static, Result<ChatChunk>>>>`. The `'static`
   lifetime is deliberate (`mod.rs:58-64`) so the stream outlives the
   trait-method borrow and routes through `Arc<dyn LlmClient>`. Each
   provider's stream emits `ChatChunk { delta, finish_reason, usage }`
   (`mod.rs:233-241`).

4. **Thinking translation.** `ChatRequest.thinking: Option<ThinkingConfig>`
   is resolved per-model by `resolve_thinking_protocol`
   (`capabilities.rs:401-488`) into a `ThinkingProtocol`. Each protocol has a
   pure translator (`to_reasoning_effort`, `to_anthropic_effort`,
   `to_anthropic_budget`, `to_enable_thinking`, `to_glm_thinking_type`,
   `to_glm_reasoning_effort`). `ThinkingProtocol::emits_field()`
   (`thinking.rs:301-312`) returns false for `None` and `AnthropicAdaptive`,
   so the field is dropped rather than sent to a model that would 400.

5. **Usage authority.** Every provider normalizes token/cache metrics into
   the same `Usage` struct (`types.rs:654-684`). `effective_prompt_tokens()`,
   `cached_prompt_tokens()`, `cache_creation_prompt_tokens()`,
   `cache_hit_rate()` (`types.rs:686-771`) are the single normalization site,
   handling OpenAI-inclusive, DeepSeek-inclusive, and Anthropic-exclusive
   cache semantics.

6. **Cache hints.** `ChatRequest.cache_hints: Option<CacheHints>`
   (`mod.rs:182`) carries provider-neutral breakpoint targets
   (`BreakpointTarget` enum) and segment ranges. `PromptCacheLayout`
   (`cache/layout.rs:72-136`) is a read-only, zero-copy view that segments
   the messages array by role + content markers; `stable_prefix_hash`
   (`cache/diagnostic.rs:27-55`) is a cross-process-reproducible SHA-256 over
   the stable prefix.

7. **Cancellation.** `ChatRequest.cancel_token: Option<CancellationToken>`
   (`mod.rs:175`). Shared transport `stream_post` polls
   `is_cancelled()` between SSE chunks (`client.rs:252-254`); Anthropic's
   standalone stream loop polls the same token inline
   (`anthropic.rs:482-484`). Both terminate cleanly.

8. **Errors.** Provider failures surface as `ReactError::Llm(Box<LlmError>)`
   (`error.rs:21`). `LlmError` (`error.rs:87-108`) has 5 typed variants;
   retryability is decided by `is_retryable` checking status codes
   (`anthropic.rs:24-29`: 429 or >=500).

## Findings

### F-LLM-01-P2-01: Anthropic reasoning/thinking output is not surfaced through the neutral contract in any path

- Priority: P2
- Confidence: high
- Layer: framework (contract observation) / adapter (implementation gap)
- Evidence:
  - `echo-core/src/llm/capabilities.rs:98` —
    `ProviderCapabilities::anthropic()` sets `reasoning_content: false`
    with comment "not mapped in this implementation".
  - `echo-integration/src/providers/anthropic.rs:335` — non-streaming
    response `Message.reasoning_content: None` (hard-coded).
  - `echo-integration/src/providers/anthropic.rs:541,566,611` — every
    streaming `ChatChunk.delta.reasoning_content: None`.
  - `echo-integration/src/providers/anthropic.rs:829-862` — the
    `ContentBlock` enum has no `Thinking` / `RedactedThinking` variant.
  - `echo-integration/src/providers/anthropic.rs:308` — the `_ => {}`
    catch-all silently drops thinking content blocks returned by the API.
- Reachability: every Anthropic chat call (streaming and non-streaming).
  `AnthropicClient` implements `LlmClient` (`anthropic.rs` impl block) and is
  constructed by `ProviderFactory::from_provider_model` for `provider ==
  "anthropic"` (`config.rs:394-427`). Live in any deployment using Claude.
- Expected invariant: the task asks whether providers can "preserve thinking
  without semantic loss". The neutral contract types *do* carry
  `reasoning_content` (`Message` + `DeltaMessage`), so the contract supports
  it; but one of the three first-party providers never populates it, so in
  practice Anthropic thinking is lost at the contract boundary.
- Observed behavior: OpenAI-compatible reasoning models (GPT-5, DeepSeek-r1,
  Qwen3) stream `reasoning_content` deltas through the neutral contract;
  Anthropic models produce extended-thinking blocks that the adapter drops
  before they reach the contract. An observer consuming `ChatChunk` cannot
  tell whether a Claude model thought at all.
- Impact: downstream consumers (observability, UI "thinking" panels, memory
  of prior reasoning) see reasoning for OpenAI-family models and silence for
  Anthropic. This is an asymmetry in the otherwise-uniform streaming surface.
- Root cause: the Anthropic adapter was written before extended-thinking
  output was added to the Claude API; the `ContentBlock` enum and the
  response mapper were never extended. The `reasoning_content: false`
  capability encodes this as if it were a permanent provider property
  rather than a temporary implementation gap.
- Direction: this is primarily an adapter fix (add a `Thinking` variant to
  the Anthropic `ContentBlock`, map it to `reasoning_content` on both paths,
  flip `ProviderCapabilities.anthropic().reasoning_content` to `true`). That
  work belongs to **F-LLM-03**. The contract itself needs no change — it
  already has the field. Record here so F-LLM-03 has a concrete checklist,
  and so synthesis does not treat the silence as "by design".
- Regression validation: after the F-LLM-03 fix, a streaming fixture carrying
  a `thinking` content block must produce a `ChatChunk` with non-`None`
  `reasoning_content`; a non-streaming response must populate
  `Message.reasoning_content`.
- Validation reports: [V02](../validations/F-LLM-01/V02-01.md),
  [V01](../validations/F-LLM-01/V01-01.md)

### F-LLM-01-P2-02: `Message::reasoning_content` cannot carry Anthropic's signed thinking blocks for multi-turn round-trip

- Priority: P2
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-core/src/llm/types.rs:251` — `pub reasoning_content: Option<String>`.
  - `echo-core/src/llm/types.rs:843` — `DeltaMessage.reasoning_content:
    Option<String>` (streaming delta).
- Reachability: every response/delta that carries reasoning. Live for
  OpenAI-family reasoning models today; would be live for Anthropic once
  F-LLM-01-P2-01 is fixed.
- Expected invariant: "preserve thinking without semantic loss" (task
  question). Anthropic's extended-thinking protocol emits an encrypted
  `signature` alongside each thinking block; when the caller wants to
  continue a multi-turn conversation that interleaves thinking with tool
  use, the *prior* thinking blocks (text + signature) must be replayed in
  the request verbatim. A flat `String` cannot represent this.
- Observed behavior: the contract exposes thinking purely as observational
  text. There is no structured `ThinkingBlock { text, signature }` type, no
  request-side way to attach a previously-seen thinking block, and
  `MessageContent::Parts` has no thinking variant either. Anthropic thinking
  therefore cannot be round-tripped through the neutral contract even if the
  adapter populated the response side.
- Impact: any multi-turn flow that relies on preserving prior reasoning
  (chain-of-thought continuity, "extended thinking with tool use") works for
  the request-side `thinking` *config* but has no contract home for the
  response-side thinking *content*. Today this is masked because P2-01 means
  Anthropic thinking never reaches the contract at all; once P2-01 is fixed,
  this becomes the next bottleneck.
- Root cause: the contract was designed around the OpenAI/Qwen3/DeepSeek
  model where reasoning is an opaque text stream. Anthropic's signed-block
  model was not represented in the neutral types.
- Direction: introduce a neutral `ThinkingBlock { text: String, signature:
  Option<String> }` (or extend `ContentPart` with a `Thinking` variant
  carrying text + signature), and a request-side way to attach prior
  thinking blocks on an assistant `Message`. Keep `reasoning_content` as a
  convenience accessor that joins thinking-block text for observers who do
  not need the signature. Evaluate as part of F-LLM-03 design; flagged here
  because it is a contract-level design question, not an adapter bug.
- Regression validation: a round-trip test that receives an Anthropic
  thinking block, echoes it back in the next request, and asserts the
  signature survives unchanged.
- Validation reports: [V03](../validations/F-LLM-01/V03-01.md),
  [V01](../validations/F-LLM-01/V01-01.md)

### F-LLM-01-P3-01: `ChatRequest.tool_choice` is stringly-typed OpenAI wire format, not the typed `ToolChoice` enum

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-core/src/llm/mod.rs:162` — `pub tool_choice: Option<String>`.
  - `echo-core/src/llm/mod.rs:114-146` — `pub enum ToolChoice { Auto, None,
    Required, Function{name} }` with `to_openai_value()` — typed construction
    exists, but the request stores its string output, not the enum.
- Reachability: live. Every provider reads `request.tool_choice` as a string
  and translates.
- Expected invariant: a typed neutral contract should carry the typed enum
  end-to-end, translating to wire format only at the provider boundary.
- Observed behavior: the enum is a construction helper only; the wire string
  leaks into `ChatRequest`. Anthropic/Ollama must re-parse the string to
  translate it. The field comment (`mod.rs:159-161`) acknowledges this and
  points users at `ToolChoice::to_openai_value`.
- Impact: low. Callers who construct `tool_choice` by hand (not via
  `ToolChoice::function(...)`) can put any string, including typos, and the
  framework will not catch it until the provider rejects it. Not a
  correctness defect for typed callers.
- Root cause: the request struct predates the typed enum; the field was not
  migrated when `ToolChoice` was added.
- Direction: change `ChatRequest.tool_choice` to `Option<ToolChoice>` and have
  each provider call the appropriate translator. Under AGENTS.md
  "no backward compatibility burden", this is a safe in-place migration.
  Alternatively, keep the string but add a `ToolChoice::into_request_value()`
  that is the only sanctioned way to populate the field, and document it.
- Regression validation: `cargo test --workspace --all-features`; update the
  provider sites that read `request.tool_choice` as a string.
- Validation reports: [V01](../validations/F-LLM-01/V01-01.md)

### F-LLM-01-P3-02: Two overlapping thinking-protocol enums at different layers

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-core/src/llm/thinking.rs:270-296` — `ThinkingProtocol`, 8 variants
    (framework-level, resolved per-model in `capabilities.rs`).
  - `echo-integration/src/providers/traits.rs:96-107` —
    `ThinkingProtocolPreference`, 5 variants (transport-level, declared per
    `ProviderAdapter`).
- Reachability: both live. `ThinkingProtocol` drives request-field emission;
  `ThinkingProtocolPreference` is read by the OpenAI-compat shared transport
  via `ProviderAdapter::thinking_protocol()`.
- Expected invariant: one authoritative enum for "which thinking wire field
  does this provider/model use", or a clearly documented mapping between the
  two.
- Observed behavior: the two enums overlap (`OpenAiReasoningEffort`,
  `EnableThinkingFlag`, `GlmReasoningEffort` appear in both with slightly
  different names) but disagree at the edges — the transport enum has
  `DeepSeekDual` and `None`; the framework enum has `AnthropicEffort`,
  `AnthropicThinkingBudget`, `AnthropicAdaptive`, `GlmThinkingType` that the
  transport enum lacks. Anthropic bypasses the shared transport, so its
  protocols never appear in `ThinkingProtocolPreference`.
- Impact: low. A contributor reading both files must reason about which enum
  is authoritative for a given code path. No runtime defect because the two
  are consumed by disjoint code.
- Root cause: the transport-level enum was added when `ProviderAdapter` was
  introduced, without reconciling against the framework-level enum.
- Direction: either (a) collapse into one enum and have the adapter return the
  framework type; or (b) keep both but add a doc cross-reference on each
  pointing to the other and stating which code path consumes which. Option
  (a) is cleaner under the no-compat rule.
- Regression validation: `cargo check --workspace --all-features`; confirm
  the OpenAI-compat transport still picks the right field for each provider.
- Validation reports: [V04](../validations/F-LLM-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Contract type inventory and duplicate search | yes | passed | [V01-01](../validations/F-LLM-01/V01-01.md) |
| V02 | ProviderCapabilities completeness and streaming neutrality | yes | passed (with Anthropic reasoning gap noted) | [V02-01](../validations/F-LLM-01/V02-01.md) |
| V03 | Thinking translation faithfulness and usage/cache authority | yes | passed (with signature round-trip limitation noted) | [V03-01](../validations/F-LLM-01/V03-01.md) |
| V04 | Adapter contract thinness + compile/test check | yes | passed | [V04-01](../validations/F-LLM-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | No historical document is cited as evidence. No prior F-LLM-01 report exists in this reviewer directory. |

## Historical Claim Status

No historical documents are cited as evidence for any claim in this report.
All findings are based on code at commit `9b0e0fa` and the four validation
reports above. The F-CORE-01 conclusion that `LlmError` is the single typed
LLM error type is reused as a dependency (current at `error.rs:87-108`).

## Coverage And Uncertainty

- Code not inspected in depth: the concrete OpenAI (`openai.rs`) and
  Anthropic (`anthropic.rs`) request-building and response-mapping bodies were
  read only for the streaming/usage/thinking paths relevant to this contract
  review. Full field-by-field adapter fidelity is the subject of F-LLM-02 and
  F-LLM-03.
- The `thinking_translate.rs` shared translator was inspected for its public
  `OpenAiCompatThinking` output struct and its documented vendor matrix; its
  internal mapping functions were not exhaustively audited (belongs to
  F-LLM-02).
- Environmental limits: `cargo check` and `cargo test` were run only for
  `echo_core` (the contract crate). `echo_integration` provider tests were
  not run in this task; they are in scope for F-LLM-02/F-LLM-03.
- Claims that remain uncertain:
  - Whether any third-party `echo-agent` consumer outside this monorepo
    depends on `reasoning_content: Option<String>` being a flat string. The
    F-LLM-01-P2-02 direction proposes adding structure; under AGENTS.md
    "no backward compatibility burden" this is acceptable, but a downstream
    impact check is advisable before implementing.
  - Whether Anthropic's API currently returns `redacted_thinking` blocks in
    any model this framework targets. If it does, F-LLM-03 must also map
    those (they carry no text, only a signature/data blob).

## Handoff

- Conclusions downstream tasks may rely on:
  - The neutral LLM contract is singly defined in `echo-core/src/llm` and
    covers all eight contract dimensions (messages, tools, thinking, usage,
    caching, streaming, cancellation, errors). No duplicate public type.
  - `Usage` normalization (`effective_prompt_tokens`,
    `cached_prompt_tokens`, `cache_hit_rate`) is the single authority and is
    trusted by all providers. F-LLM-02/F-LLM-03 should verify their adapters
    feed the raw vendor fields and let these methods do the math.
  - `ThinkingConfig` + `ThinkingProtocol` is the single request-side thinking
    authority; per-vendor translators are pure functions and the source of
    truth for wire-field emission.
  - `ProviderAdapter` is a declaration-only surface; adapters must not gain
    transport, retry, or state authority.
  - Cancellation is uniformly wired through `ChatRequest.cancel_token` and is
    checked on both the shared transport and the Anthropic standalone path.
- Reports they must read:
  - [V01-01](../validations/F-LLM-01/V01-01.md) for the full contract field
    inventory.
  - [V02-01](../validations/F-LLM-01/V02-01.md) for the streaming-neutrality
    trace and the Anthropic reasoning gap.
  - [V03-01](../validations/F-LLM-01/V03-01.md) for the usage/cache authority
    and thinking-signature limitation.
- Conditions that make this report stale:
  - Any change to `Message::reasoning_content` type or addition of a
    `ThinkingBlock` type invalidates F-LLM-01-P2-02.
  - Any commit that populates Anthropic `reasoning_content` invalidates
    F-LLM-01-P2-01.
  - Any change to `Usage` field set or normalization methods invalidates the
    V03 authority claim.
  - Any migration of `ChatRequest.tool_choice` to the typed enum invalidates
    F-LLM-01-P3-01.
- Follow-up task IDs (no fixes implemented in this review):
  - **F-LLM-02** (OpenAI adapter) should verify it faithfully populates
    `reasoning_content`, tool-call deltas, and usage on both paths, and that
    it does not reintroduce provider-specific fields on the neutral
    `ChatRequest`.
  - **F-LLM-03** (Anthropic adapter) owns the fix for F-LLM-01-P2-01
    (populate `reasoning_content` from thinking blocks) and should evaluate
    the contract change in F-LLM-01-P2-02 (signed thinking round-trip)
    together with the adapter work, since both touch thinking preservation.
