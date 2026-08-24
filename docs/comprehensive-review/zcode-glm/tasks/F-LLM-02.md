# F-LLM-02: OpenAI provider adapter

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable
> Worktree state: clean

## Question

Does the OpenAI adapter faithfully implement the neutral contract for request
construction, deltas, tool calls, usage, and failures?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-integration/src/providers/openai.rs` — `OpenAiClient`
  (`LlmClient` impl), standalone `chat` / `stream_chat` functions,
  `DefaultLlmClient`, `normalize_messages`, `assemble_req_header`.
- `echo-agent/echo-integration/src/providers/adapter_client.rs` —
  `AdapterClient<A: ProviderAdapter>` (the alternative OpenAI-compat client
  that delegates per-vendor behaviour to a `ProviderAdapter`).
- `echo-agent/echo-integration/src/providers/client.rs` — shared transport
  `post`, `stream_post`, SSE parsing (`split_sse_event`, `parse_sse_data`,
  `parse_sse_chunk`), retry policy (`is_retryable`), stream timeouts.
- `echo-agent/echo-integration/src/providers/thinking_translate.rs` —
  `translate_thinking_openai_compat` (consumed by both `OpenAiClient` and
  `AdapterClient`).
- `echo-agent/echo-integration/src/providers/traits.rs` — `ProviderAdapter`
  trait, `ThinkingProtocolPreference`, `resolve_base_url`.
- `echo-agent/echo-integration/src/providers/config.rs` — `LlmConfig`,
  `ProviderFactory`, provider routing (`build_client`, `from_provider_model`).
- `echo-agent/echo-core/src/llm/types.rs` — wire body (`ChatCompletionRequest`,
  `ChatCompletionResponse`, `ChatCompletionChunk`, `DeltaMessage`,
  `DeltaToolCall`, `Usage`).
- `echo-agent/echo-core/src/llm/mod.rs` — `ChatRequest`, `ChatResponse`,
  `ChatChunk`, `LlmClient` trait.
- `echo-agent/echo-core/src/error.rs` — `LlmError` hierarchy.
- `echo-agent/src/agent/react/run/processor.rs` — downstream tool-call
  assembler (`process_stream_chunk`, `build_tool_calls_from_map`,
  `parse_tool_args`).

## Out Of Scope

- Anthropic adapter fidelity (interleaved content blocks, cache-control,
  thinking-block population) — deferred to **F-LLM-03**. The Anthropic
  adapter is referenced only where it provides comparative context for the
  OpenAI path.
- The neutral contract types themselves — audited in **F-LLM-01** and relied
  on here without re-auditing.
- React-engine loop behavior, multi-turn tool dispatch, plan/subagent
  orchestration — out of scope; only the tool-call assembler is inspected
  because the OpenAI adapter delegates assembly to it.
- HTTP mock-based end-to-end tests of `OpenAiClient::chat` / `chat_stream` —
  no such tests exist in the codebase (see Coverage And Uncertainty).

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
    the single authority handling OpenAI-inclusive cache semantics; that
    `ChatRequest.tool_choice` is `Option<String>` (stringly-typed OpenAI wire
    format — finding F-LLM-01-P3-01); that `ProviderAdapter` is a declaration-
    only surface (F-LLM-01 V04-01); that `ThinkingProtocolPreference`
    (transport) and `ThinkingProtocol` (framework) overlap (F-LLM-01-P3-02).
    This report relies on those conclusions and does not re-audit the
    contract types.
  - [F-CORE-01](./F-CORE-01.md) — established that `LlmError` is the single
    typed LLm error type with no `Other(String)` escape hatch.
- Historical documents treated as hypotheses: none.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `OpenAiClient`, `AdapterClient`, the shared `post`/`stream_post` transport, `translate_thinking_openai_compat`, and `LlmError` mapping are generic OpenAI-compat plumbing any `echo-agent` consumer may use. They correctly live in `echo-integration`. |
| EKO product policy | None at this layer. No EKO-specific field leaks into the OpenAI adapter. |
| Adapter boundary | `OpenAiClient` is a thin translator: neutral `ChatRequest` → OpenAI-shaped `ChatCompletionRequest`, OpenAI `ChatCompletionResponse`/`ChatCompletionChunk` → neutral `ChatResponse`/`ChatChunk`. No retry-loop authority (delegates to `with_retry_if` in `client.rs`), no state, no scheduling. `AdapterClient` is equally thin (one extra `prepare_request` hook) but dormant — see F-LLM-02-P2-01. |
| Duplicate search | Searched: `struct OpenAiClient`, `struct AdapterClient`, `struct DefaultLlmClient`, `trait ProviderAdapter`, `impl LlmClient`, `fn chat_stream`, `fn assemble_req_header`, `normalize_messages`, `translate_thinking_openai_compat`. Result: one `OpenAiClient` (`openai.rs:215`), one `AdapterClient` (`adapter_client.rs:25`), one `DefaultLlmClient` (`openai.rs:387`). Three `impl LlmClient` for OpenAI-compat: `OpenAiClient`, `AdapterClient<A>`, `DefaultLlmClient`. `OpenAiClient` is the live path; the other two have no construction site in either repo (see findings F-LLM-02-P2-01, F-LLM-02-P2-02). No duplicate definition of the shared transport — `post`/`stream_post` are singly defined in `client.rs`. |
| Migration deletion | No migration proposed. |

## Current Path

Verified OpenAI-compat data flow at commit `9b0e0fa`:

1. **Provider routing.** `LlmConfig::build_client`
   (`config.rs:302-317`) matches on `LlmProvider` (`OpenAi` | `Anthropic`)
   and constructs `OpenAiClient::new(self.clone())` for every OpenAI-compat
   provider. `ProviderFactory::from_provider_model` (`config.rs:394-427`)
   parses `provider:model` strings, looks up the base URL, reads the API key
   from provider-specific env vars (`:430-441`), and delegates to
   `build_client`. All nine supported providers (`openai`, `anthropic`,
   `deepseek`, `dashscope`, `qwen`, `moonshot`, `kimi`, `zhipu`, `glm`) route
   through this single entry point. The `AdapterClient` path is never
   selected.

2. **Request construction.** `OpenAiClient::chat` / `chat_stream`
   (`openai.rs:263-379`) build a `ChatCompletionRequest` by direct field
   assignment, calling `translate_thinking_openai_compat` to convert
   `request.thinking` into the right vendor wire fields. The model name and
   `provider_name` (e.g. `"deepseek"`, `"dashscope"`, `"zhipu"`) drive the
   thinking-protocol resolution inside the translator — see
   `thinking_translate.rs:40-147`.

3. **Non-streaming call.** `client.rs::post` (`client.rs:109-172`) sends the
   JSON body with `Authorization`/`Content-Type` headers, retries on
   network/429/5xx via `with_retry_if`, and deserializes the response into
   `ChatCompletionResponse`. `OpenAiClient::chat` (`openai.rs:310-316`) reads
   `choices.first()`, returns `ChatResponse { message, finish_reason, raw }`.

4. **Streaming call.** `client.rs::stream_post` (`client.rs:182-354`) sends
   the same body with `stream: true`, returns a `Stream<Item =
   Result<ChatCompletionChunk>>` that parses SSE events, handles `[DONE]`,
   enforces three independent timeouts, and checks `cancel_token` between
   chunks. `OpenAiClient::chat_stream` (`openai.rs:366-375`) maps each chunk
   to `ChatChunk { delta, finish_reason, usage }`, with `delta` and
   `finish_reason` taken from `choices.first()` and `usage` taken from the
   chunk top level — so the final usage-only chunk (empty `choices`)
   preserves its `usage`.

5. **Tool calls.** The OpenAI adapter performs no assembly. Non-streaming
   responses carry fully-assembled `ToolCall` objects in
   `Choice.message.tool_calls` (`types.rs:451-460`); streaming chunks carry
   `DeltaToolCall` fragments indexed by `index` (`types.rs:851-863`). The
   downstream React-engine assembler (`processor.rs:16-183`) aggregates by
   `index`, concatenates argument fragments, repairs trailing-character
   corruption, and drops unrepairable calls.

6. **Usage.** Streaming: `ChatChunk.usage: Option<Usage>` is populated from
   `chunk.usage` (`openai.rs:372`) — the final chunk carries usage because
   `stream_options: {"include_usage": true}` is set (`openai.rs:346`).
   Non-streaming: `ChatResponse.raw.usage` (`types.rs:633-634`) is the
   authoritative source; `ChatResponse` has no top-level `usage` field.
   Normalization (`effective_prompt_tokens`, `cached_prompt_tokens`,
   `cache_hit_rate` at `types.rs:686-771`) is inherited unchanged from
   F-LLM-01.

7. **Errors.** Every failure maps onto `LlmError` (`error.rs:87-108`):
   network failure → `NetworkError`, HTTP non-success → `ApiError{status,
   message}`, body parse failure → `InvalidResponse`, empty `choices` →
   `EmptyResponse`. `is_retryable` (`client.rs:13-19`) returns true for
   `NetworkError` and `ApiError` with status 429 or >=500; everything else
   fails fast.

## Findings

### F-LLM-02-P2-01: `AdapterClient` + `ProviderAdapter` are dormant — no implementor, no consumer, doc-comment claims routing it does not perform

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-integration/src/providers/adapter_client.rs:1-4` — module
    doc-comment claims the abstraction "avoids duplicating the HTTP/SSE layer
    across DeepSeek, GLM, Kimi, Qwen".
  - `echo-integration/src/providers/traits.rs:22-69` — `ProviderAdapter`
    trait declared; 7 declaration hooks including `prepare_request`.
  - `echo-integration/src/providers/mod.rs:8-14` — both `AdapterClient` and
    `ProviderAdapter` are `pub use`-exported.
  - Repository-wide grep `AdapterClient` and `impl ProviderAdapter` across
    `echo-agent/` and `echo-agent-cli/` returns zero construction sites and
    zero implementors. The only references are the type's own definition, the
    `pub use`, and the `traits.rs` doc-comment.
  - The actual OpenAI-compat routing goes through `OpenAiClient` via
    `LlmConfig::build_client` (`config.rs:302-317`), which collapses every
    provider to `LlmProvider::OpenAi`.
  - Per-vendor differences (DeepSeek dual-field thinking, GLM `thinking.type`,
    Qwen `enable_thinking`) are encoded in `translate_thinking_openai_compat`
    (`thinking_translate.rs:40-147`) keyed on the `provider_name` string, NOT
    in `ProviderAdapter` implementations.
- Reachability: defined → pub-exported → never constructed. The
  `prepare_request` hook (`traits.rs:62`) is invoked only inside
  `AdapterClient::chat`/`chat_stream` (`adapter_client.rs:104, 158`), which
  itself is never instantiated. No live runtime path reaches this code.
- Expected invariant: AGENTS.md "动手前先查是不是已经有了" requires that
  abstractions be either live or removed; "code cleanup: over-time code can
  be deleted" applies. AGENTS.md "framework-delete" test (✅ branch 1)
  requires that deletion candidates be "not a reasonable external API"
  (no pub, no doc, non-trait) — `ProviderAdapter` IS a pub trait with docs,
  so it is a "reasonable framework option" that is retained by default. The
  issue is therefore not "delete it" but "the abstraction is unproven and
  its doc-comment is false".
- Observed behavior: a `ProviderAdapter`-shaped abstraction exists,
  pub-exported, with a doc-comment claiming it routes DeepSeek/GLM/Kimi/Qwen,
  but the live code routes those providers through `OpenAiClient` +
  string-keyed thinking translation instead. A contributor reading
  `adapter_client.rs` would believe the abstraction is in use and try to
  extend it, while the actual extension point is `translate_thinking_openai_compat`.
- Impact: maintainability + contributor confusion. Two paradigms exist for
  the same concern (per-vendor OpenAI-compat customization): the dormant
  trait-based one and the live string-dispatch one. The doc-comment actively
  misleads about which one is authoritative.
- Root cause: the `ProviderAdapter` abstraction was introduced (likely
  anticipating per-vendor request customization beyond thinking) but never
  wired up. Thinking translation grew independently in
  `thinking_translate.rs`, becoming the de-facto per-vendor dispatch.
- Direction: pick one of:
  (a) **Wire it up** — implement `ProviderAdapter` for at least one provider
  that has non-thinking customization (e.g., DeepSeek's `user_id`-for-cache
  behavior or a provider-specific header), update `build_client` to construct
  `AdapterClient<DeepSeekAdapter>` instead of bare `OpenAiClient`, and
  correct the doc-comment to match. This proves the abstraction.
  (b) **Delete it** — under AGENTS.md "code cleanup", remove
  `adapter_client.rs`, `traits.rs` (`ProviderAdapter`,
  `ThinkingProtocolPreference`, `resolve_base_url`), and the `pub use` in
  `mod.rs`, since `translate_thinking_openai_compat` + `OpenAiClient` already
  cover the live per-vendor concerns cleanly.
  Do not leave the dormant abstraction with a misleading doc-comment.
- Regression validation: `cargo check -p echo_integration --all-features`;
  `cargo test -p echo_integration --lib providers::`. If option (a), add a
  test that exercises the wired-up adapter's `prepare_request` hook end-to-end.
- Validation reports: [V01](../validations/F-LLM-02/V01-01.md),
  [V04](../validations/F-LLM-02/V04-01.md)

### F-LLM-02-P2-02: `DefaultLlmClient` is dormant — pub-exported, never constructed, and silently bypasses thinking translation

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-integration/src/providers/openai.rs:387-514` — `DefaultLlmClient`
    struct, `new()` constructor, and `LlmClient` impl.
  - `echo-integration/src/providers/mod.rs:14` — `pub use openai::{
    DefaultLlmClient, OpenAiClient};`.
  - `echo-agent/src/llm.rs:81, 101` — re-exported at the framework root.
  - Repository-wide grep `DefaultLlmClient::new` and `DefaultLlmClient::
    with_client` returns zero call sites in `echo-agent/` or `echo-agent-cli/`
    (only doc-comment references in `llm.rs:10`).
  - `openai.rs:404-409, 438-443` — when `request.thinking.is_some()`, the
    impl emits a `warn!` ("does not translate thinking config") and proceeds
    to call the standalone `chat` / `stream_chat` functions
    (`openai.rs:410-421, 445-456`), which hard-code all thinking wire fields
    to `None` (`openai.rs:162-165, 198-201`).
- Reachability: defined → pub-exported → never constructed. No runtime path
  reaches this code from anywhere in the monorepo.
- Expected invariant: a pub `LlmClient` implementation that advertises itself
  as the "default" should either be the actual default or removed. The real
  default for the framework is `OpenAiClient` (constructed by
  `LlmConfig::build_client` and `AgentBuilder` at `builder.rs:275`).
- Observed behavior: `DefaultLlmClient` exists, is pub-exported at two
  layers, is named as if it were the default, but is never used. If it were
  picked up by a downstream consumer, it would silently drop thinking config
  (with only a `warn!` log) — a behavior informed users would not expect
  from a struct called "Default".
- Impact: misleading API surface + silent thinking-drop trap for downstream
  consumers who reach for the "Default" client by name.
- Root cause: predates `OpenAiClient`'s thinking translation; superseded when
  `OpenAiClient::new(LlmConfig)` became the standard constructor. Never
  removed.
- Direction: delete `DefaultLlmClient`, its `LlmClient` impl, the standalone
  `chat` / `stream_chat` functions (used only by `DefaultLlmClient`), and
  the re-exports in `mod.rs:14` and `llm.rs:81, 101`. Under AGENTS.md
  framework-delete test, this is the ✅ branch 1 case (internal-only dead
  code with a pub surface that is superseded by `OpenAiClient`). If a
  downstream consumer outside the monorepo depends on it, they can use
  `OpenAiClient::from_env` or `OpenAiClient::new(LlmConfig)` instead.
- Regression validation: `cargo check --workspace --all-features`; update
  the doc table in `echo-agent/src/llm.rs:9-12` to remove the
  `DefaultLlmClient` row.
- Validation reports: [V01](../validations/F-LLM-02/V01-01.md)

### F-LLM-02-P2-03: Malformed SSE chunks are silently dropped at the transport layer, never surfaced as errors

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-integration/src/providers/client.rs:99-105` — `parse_sse_chunk`
    on serde failure: logs `tracing::warn!` and returns `None`.
  - `echo-integration/src/providers/client.rs:317-333` — the stream loop
    matches `parse_sse_chunk` returning `Some(parsed)`; when it returns
    `None` (parse failure), the loop simply continues to the next event with
    no error propagation.
- Reachability: every streaming OpenAI-compat call. Live whenever a provider
  returns an SSE chunk that fails to deserialize as `ChatCompletionChunk`
  (e.g., a usage-only chunk with an unexpected field type, or a content
  chunk with a non-string `content` field).
- Expected invariant: a transport-layer parse failure on a chunk should
  either be surfaced as `LlmError::InvalidResponse` (fail-fast) or
  propagated to the consumer via the stream's `Result` item type. Silently
  dropping means neither the adapter, the React engine, nor the caller can
  distinguish "provider sent fewer chunks than expected" from "provider sent
  a chunk we couldn't parse".
- Observed behavior: a malformed chunk is warn-logged and discarded; the
  stream continues. If the malformed chunk carried usage, the call silently
  reports `usage_reported: false` downstream. If it carried content, the
  content is silently absent from the assembled message. The only signal is
  a warn log line.
- Impact: debugging "why is my usage missing" or "why is my response
  truncated" becomes very hard — the failure is invisible to programmatic
  consumers. The comment at `client.rs:100-103` acknowledges the tradeoff
  ("provider may use non-standard format"), but the chosen mitigation
  (silent drop) is the most aggressive option.
- Root cause: the parser was written defensively to keep streaming robust
  against quirky providers, with nofail-fast escape hatch. The tradeoff was
  not revisited once the framework matured.
- Direction: at minimum, expose a structured signal — either:
  (a) Add an `on_malformed_chunk` callback or counter on the stream context,
  so observability layers can count parse failures.
  (b) Surface the first malformed chunk as `Err(LlmError::InvalidResponse)`
  through the stream and let the caller decide whether to terminate.
  (c) Add a "strict mode" env flag (`ECHO_AGENT_STREAM_STRICT=1`) that
  promotes parse failures to errors; keep the silent-drop default for
  backward compatibility.
  Option (c) is the least invasive.
- Regression validation: a unit test that injects a malformed SSE chunk and
  asserts either the counter increments, the error propagates, or the strict
  flag fires — depending on the chosen option.
- Validation reports: [V02](../validations/F-LLM-02/V02-01.md),
  [V04](../validations/F-LLM-02/V04-01.md)

### F-LLM-02-P2-04: `ChatResponse` has no top-level `usage` field; non-streaming consumers must reach into `raw.usage`

- Priority: P2
- Confidence: high
- Layer: framework (contract observation; closest relevant task is F-LLM-02
  because the OpenAI adapter is the canonical non-streaming consumer)
- Evidence:
  - `echo-core/src/llm/mod.rs:202-210` — `ChatResponse { message,
    finish_reason, raw }` — no `usage` field.
  - `echo-core/src/llm/mod.rs:233-241` — `ChatChunk { delta, finish_reason,
    usage }` — streaming chunk HAS a top-level `usage` field.
  - `echo-agent/src/agent/react/run/react_loop.rs:100` — non-streaming path
    reads `response.raw.usage.clone()` to work around the missing field.
  - `echo-agent/src/agent/react/run/react_loop.rs:154` — the legacy
    standalone-function path reads `response.usage.clone()` directly because
    that path returns the raw `ChatCompletionResponse` (which has `usage`).
- Reachability: every non-streaming LLM call. The OpenAI adapter's
  `OpenAiClient::chat` (`openai.rs:312-316`) constructs `ChatResponse`
  without a top-level `usage`, so every caller that wants usage must use
  `response.raw.usage` (the path the React engine takes).
- Expected invariant: the streaming and non-streaming surfaces of the same
  contract should expose the same data at the same level. `ChatChunk.usage`
  is top-level; `ChatResponse.usage` should be too.
- Observed behavior: streaming consumers read `chunk.usage`; non-streaming
  consumers must know to read `response.raw.usage`. The asymmetry is
  undocumented at the type level. New callers that copy the streaming
  pattern (`response.usage`) will not compile; callers that forget to read
  usage at all silently report no usage.
- Impact: ergonomic defect + silent no-usage trap. The React engine already
  works around it (`react_loop.rs:100`), but every new non-streaming consumer
  must learn the workaround.
- Root cause: `ChatResponse` was designed to carry the raw body for
  "callers needing extra metadata" (per its doc-comment), and `usage` was
  considered "metadata" that could live on `raw`. The streaming chunk,
  added later, did not have a `raw` to lean on and so exposed `usage`
  directly — creating the asymmetry.
- Direction: add `pub usage: Option<Usage>` to `ChatResponse` and populate
  it from `raw.usage.clone()` in each adapter (OpenAI: `openai.rs:312`;
  Anthropic: `anthropic.rs` non-streaming path; AdapterClient:
  `adapter_client.rs:114`). Note this is a contract change — coordinate with
  F-LLM-01's contract ownership. Flagged here because the OpenAI adapter is
  the canonical consumer that would populate it, and the workaround
  (`react_loop.rs:100`) is in plain sight.
- Regression validation: `cargo test --workspace --all-features`; update
  `react_loop.rs:100` to use the new top-level field; add a test asserting
  `ChatResponse.usage` is populated when the provider returns usage.
- Validation reports: [V02](../validations/F-LLM-02/V02-01.md),
  [V04](../validations/F-LLM-02/V04-01.md)

### F-LLM-02-P3-01: `[DONE]` stream terminator encoded as a sentinel string inside `LlmError::NetworkError`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-integration/src/providers/client.rs:74` —
    `const STREAM_DONE_SENTINEL: &str = "__ECHO_AGENT_STREAM_DONE__";`
  - `echo-integration/src/providers/client.rs:81-85` — `parse_sse_chunk`
    returns `Err(LlmError::NetworkError(STREAM_DONE_SENTINEL.into()))` for
    the `[DONE]` marker.
  - `echo-integration/src/providers/client.rs:329, 347` — the stream loop
    detects termination by `err.to_string().contains(STREAM_DONE_SENTINEL)`.
- Reachability: every streaming call. The `[DONE]` marker is the standard
  SSE terminator for OpenAI-compat providers.
- Expected invariant: control-flow signals should not be encoded as
  substring matches over an error variant's Display output. The Display impl
  is `#[error("Network error: {0}")]` (`error.rs:89`), so the detection
  relies on `to_string()` containing the sentinel somewhere in the formatted
  message — fragile to Display formatting changes.
- Observed behavior: the stream-termination protocol works (verified by
  `parse_done_marker` test at `client.rs:388-393`), but it works by accident
  of the Display impl, not by type. A future change to the `NetworkError`
  Display format, or a real network error whose message happens to contain
  the sentinel string, would break or mis-classify.
- Impact: low. The sentinel is unusual enough that accidental collision is
  improbable, and the Display impl is stable. But the design is brittle and
  the kind of thing a contributor might "improve" without realizing the
  substring contract.
- Root cause: `parse_sse_chunk` returns `Option<Result<ChatCompletionChunk>>`
  — three states (no chunk / chunk / done) encoded as two. The done state
  was shoe-horned into the error path because there was no clean third arm.
- Direction: change `parse_sse_chunk` to return an enum
  `enum SseParseOutcome { Chunk(ChatCompletionChunk), Done, Malformed, Empty }`
  (or have `stream_post` check `trimmed == "[DONE]"` directly before invoking
  `parse_sse_chunk`). Removes the sentinel entirely.
- Regression validation: `cargo test -p echo_integration --lib providers::client`;
  add a test that streams `[DONE]` and asserts clean termination without
  relying on the Display impl.
- Validation reports: [V02](../validations/F-LLM-02/V02-01.md),
  [V04](../validations/F-LLM-02/V04-01.md)

### F-LLM-02-P3-02: `cache_hints` silently dropped on every OpenAI-compat path

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-core/src/llm/mod.rs:182` — `pub cache_hints: Option<CacheHints>`
    on `ChatRequest`.
  - `echo-core/src/llm/types.rs:520-573` — `ChatCompletionRequest` has no
    `cache_hints` field.
  - `echo-integration/src/providers/openai.rs:280-300, 336-355` —
    `OpenAiClient::chat`/`chat_stream` build `ChatCompletionRequest` without
    referencing `request.cache_hints`.
  - `echo-integration/src/providers/adapter_client.rs:83-102, 138-157` —
    `AdapterClient` likewise drops `cache_hints`.
- Reachability: every OpenAI-compat call where the caller sets
  `cache_hints`. No live caller in the monorepo sets it for OpenAI-compat
  providers (the React engine does not currently populate `cache_hints`).
- Expected invariant: when a contract field is silently ignored, the
  framework should either document it on the field or emit a one-time
  diagnostic so callers know their hint had no effect.
- Observed behavior: callers who set `cache_hints` on a `ChatRequest`
  routed to an OpenAI-compat provider see no error, no log, and no effect.
- Impact: none today (the field is unused on this path and OpenAI's cache is
  automatic). The risk is future contributors setting `cache_hints` for
  OpenAI expecting breakpoint placement and getting silent no-op.
- Root cause: `cache_hints` was added to the contract for Anthropic-style
  explicit cache control (F-LLM-01 V01-01, `cache/layout.rs`); the
  OpenAI-compat path was never taught about it because OpenAI has no
  equivalent wire field.
- Direction: either:
  (a) Document on `ChatRequest.cache_hints` (`mod.rs:182`) that it takes
  effect only on providers with explicit cache control (Anthropic), and is
  a no-op elsewhere.
  (b) Emit a one-time `debug!` in `OpenAiClient::chat`/`chat_stream` when
  `request.cache_hints.is_some()` so the silent drop is observable.
  Option (a) is cheaper; option (b) is more discoverable.
- Regression validation: doc-only change requires no test; if (b), add a
  test asserting the log fires once.
- Validation reports: [V01](../validations/F-LLM-02/V01-01.md)

### F-LLM-02-P3-03: `chunk.choices.first()` / `raw.choices.first()` silently ignore non-first choices

- Priority: P3
- Confidence: high
- Layer: framework (adapter)
- Evidence:
  - `echo-integration/src/providers/openai.rs:310, 368` — non-streaming and
    streaming paths both take `choices.first()`.
  - `echo-integration/src/providers/adapter_client.rs:113, 170` — same.
  - `echo-core/src/llm/types.rs:520-573` — `ChatCompletionRequest` has no
    `n` field, so the framework cannot request `n > 1`.
- Reachability: every call. But because `n` is not exposed, the framework
  always receives exactly one choice from a conformant provider.
- Expected invariant: when only one of N choices is mapped, the drop should
  be documented.
- Observed behavior: non-first choices (if a provider ever sent them) are
  silently discarded. The full `choices` array is still reachable on
  `ChatResponse.raw.choices` (`types.rs:624-625`), but the streaming chunk
  has no equivalent — non-first `ChunkChoice`s are lost.
- Impact: none today. Becomes relevant only if `n` is ever added (no current
  plan).
- Root cause: the adapter was written for the single-choice case; no
  multi-choice support exists at any layer.
- Direction: add a doc-comment on `OpenAiClient::chat`/`chat_stream`
  noting that non-first choices are ignored, OR add an `n>1` guard that
  fails fast if a future caller sets `n`. Low priority.
- Regression validation: doc-only.
- Validation reports: [V02](../validations/F-LLM-02/V02-01.md)

### F-LLM-02-P3-04: Empty-string tool-call arguments (`""`, distinct from `"{}"`) cause the call to be silently dropped

- Priority: P3
- Confidence: medium
- Layer: framework (React-engine assembler, not OpenAI adapter)
- Evidence:
  - `echo-agent/src/agent/react/run/processor.rs:104-135` — `parse_tool_args`
    on `""`: `serde_json::from_str::<Value>("")` fails; the repair loop trims
    to `""` and retries — still fails; returns `Err`.
  - `echo-agent/src/agent/react/run/processor.rs:165-178` — on `Err`, the
    tool call is dropped with a `warn!`.
  - `echo-agent/src/agent/react/run/processor.rs:236-242` — the
    `parse_tool_args_empty_object` test verifies `"{}"` succeeds, but no
    test covers `""`.
- Reachability: any streaming call where a provider's incremental argument
  fragments concatenate to an empty string. OpenAI servers send
  `arguments: "{}"` for no-arg tools, so this is rarely hit in practice.
- Expected invariant: an empty-arguments tool call (model emits
  `{"index":0,"id":"...","function":{"name":"foo"}}` with no `arguments`
  fragments) should resolve to a no-arg call with arguments `{}`, not be
  dropped.
- Observed behavior: empty-string args fail JSON parsing and the call is
  dropped. The model must retry the call on the next turn.
- Impact: low probability (conformant OpenAI servers send `{}`), but a
  non-conformant or future provider that emits `arguments: ""` (or omits
  `arguments` entirely, which `DeltaFunctionCall` deserializes as `None`
  → no `push_str` → empty `args_str`) would silently lose the call.
- Root cause: `parse_tool_args` treats `""` as a parse failure rather than
  as the implicit empty-object.
- Direction: in `parse_tool_args` (`processor.rs:104`), special-case empty
  input: `if args_str.trim().is_empty() { return Ok((Value::Object(
  Default::default()), "{}".to_string())); }` before the parse attempt.
  Add a regression test.
- Regression validation: new unit test `parse_tool_args_empty_string_treated_as_empty_object`.
- Validation reports: [V03](../validations/F-LLM-02/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Request field mapping (ChatRequest → OpenAI wire body) | yes | passed | [V01-01](../validations/F-LLM-02/V01-01.md) |
| V02 | Response mapping (streamed vs non-streamed → ChatChunk/ChatResponse) | yes | passed | [V02-01](../validations/F-LLM-02/V02-01.md) |
| V03 | Tool-call assembly edge cases (parallel, empty args, malformed) | yes | passed | [V03-01](../validations/F-LLM-02/V03-01.md) |
| V04 | Usage extraction and LlmError mapping | yes | passed | [V04-01](../validations/F-LLM-02/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | No historical document is cited as evidence. No prior F-LLM-02 report exists in this reviewer directory. |

## Historical Claim Status

No historical documents are cited as evidence for any claim in this report.
All findings are based on code at commit `9b0e0fa` and the four validation
reports above. The F-LLM-01 conclusions reused as dependencies (single
contract definition, `Usage` normalization authority, `LlmError` typed
hierarchy, `ProviderAdapter` declaration-only) are current at the cited
echo-core line anchors.

## Coverage And Uncertainty

- Code not inspected in depth: the `config.rs` provider-routing logic was
  inspected only enough to confirm that all OpenAI-compat providers route
  through `OpenAiClient` (not `AdapterClient`). The full `ProviderFactory`
  parsing and env-var lookup logic was not exhaustively audited (belongs to
  a configuration task, not the adapter-fidelity task).
- The Anthropic adapter (`anthropic.rs`) was inspected only comparatively
  for the `reasoning_content` and `usage` paths. Full Anthropic adapter
  fidelity is the subject of F-LLM-03.
- No HTTP-mock-based end-to-end test of `OpenAiClient::chat` /
  `chat_stream` exists in the codebase. The existing tests exercise only
  the helper functions (`normalize_messages`, `parse_sse_chunk`,
  `parse_tool_args`, `process_stream_chunk`). Live request/response fidelity
  to the OpenAI API is therefore verified by static inspection only, not by
  executable fixture. This is a coverage gap — a wiremock-based test would
  materially increase confidence in V01/V02.
- The empty-args edge case (F-LLM-02-P3-04) is inferred from the code path
  and the absence of a covering test; not exercised against a live provider.
- Environmental limits: `cargo test -p echo_integration --lib providers::`
  (62 tests), `cargo test -p echo_agent --lib react::run::processor` (7
  tests), and `cargo clippy -p echo_integration --all-targets --all-features
  --locked -- -D warnings` all pass. The full workspace test suite and
  feature matrix were not run (out of scope for this adapter-fidelity task;
  they belong to the commit gate).
- Claims that remain uncertain:
  - Whether any third-party `echo-agent` consumer outside this monorepo
    constructs `AdapterClient` or `DefaultLlmClient`. Per AGENTS.md
    framework-delete test, the pub surface is retained by default — the
    findings recommend deletion under the ✅ branch 1 (superseded) criterion,
    but a downstream impact check is advisable before implementing.
  - Whether OpenAI's API ever returns `arguments: ""` (not `"{}"`) for a
    no-arg tool. The OpenAI docs and observed behavior suggest `{}`, but the
    OpenAI spec does not strictly forbid `""`.

## Handoff

- Conclusions downstream tasks may rely on:
  - The OpenAI adapter (`OpenAiClient`) faithfully implements the neutral
    contract on both paths: request field mapping is complete (modulo the
    documented `cache_hints` no-op), streaming/non-streaming response
    mapping preserves content/reasoning/tool-call deltas/finish reason/
    usage, tool-call fragments are correctly relayed for downstream
    index-keyed assembly, and every failure mode maps onto the typed
    `LlmError` hierarchy with correct retry gating.
  - The shared transport (`client.rs::post`/`stream_post`) is the single
    authority for HTTP, retry, SSE parsing, timeouts, and cancellation for
    every OpenAI-compat provider. Anthropic is the only provider that does
    not route through it.
  - `translate_thinking_openai_compat` (`thinking_translate.rs:40-147`) is
    the de-facto per-vendor dispatch for OpenAI-compat thinking wire fields,
    keyed on `provider_name`. This is the live extension point for new
    vendors — NOT `ProviderAdapter::prepare_request`.
  - The React-engine assembler (`processor.rs`) is the single authority for
    streaming tool-call assembly (index-keyed aggregation, argument
    concatenation, trailing-character repair, drop-on-fail policy). The
    OpenAI adapter correctly does not duplicate this.
- Reports they must read:
  - [V01-01](../validations/F-LLM-02/V01-01.md) for the field-by-field
    request mapping table.
  - [V02-01](../validations/F-LLM-02/V02-01.md) for the streaming
    null-delta deserializer and the silent-drop behavior on malformed SSE.
  - [V03-01](../validations/F-LLM-02/V03-01.md) for the tool-call assembly
    edge cases.
  - [V04-01](../validations/F-LLM-02/V04-01.md) for the usage/error mapping
    and retry gating.
- Conditions that make this report stale:
  - Any wiring-up of `AdapterClient` for a real provider invalidates
    F-LLM-02-P2-01.
  - Any deletion of `DefaultLlmClient` invalidates F-LLM-02-P2-02.
  - Any change to `parse_sse_chunk` that surfaces malformed chunks as errors
    invalidates F-LLM-02-P2-03.
  - Any addition of a top-level `usage` field to `ChatResponse` invalidates
    F-LLM-02-P2-04.
  - Any change to the `[DONE]` handling that introduces a typed terminator
    invalidates F-LLM-02-P3-01.
  - Any change to `parse_tool_args` that special-cases empty input
    invalidates F-LLM-02-P3-04.
- Follow-up task IDs (no fixes implemented in this review):
  - **F-LLM-03** (Anthropic adapter) should mirror this audit for the
    Anthropic path: non-streaming `Message.reasoning_content` (currently
    `None` — see F-LLM-01-P2-01), streaming thinking blocks, cache-control
    placement, and the `usage` asymmetry (F-LLM-02-P2-04 affects Anthropic
    too — its non-streaming `ChatResponse` also lacks a top-level `usage`).
  - A future **configuration** task should audit `ProviderFactory` and the
    env-var/provider-name parsing in `config.rs` end-to-end (only lightly
    inspected here).
  - A future **observability** task should pick one of the options in
    F-LLM-02-P2-03 and F-LLM-02-P3-01 to make silent drops and the
    `[DONE]` terminator observable/typed.
