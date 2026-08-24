# F-LLM-01: Provider-neutral LLM contract

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean except an untracked
> `echo-agent/tests/f_rct_01_probe.rs` (owned by another task; untouched)

## Question

Can provider implementations preserve messages, tools, thinking, usage,
caching, streaming, cancellation, and errors without semantic loss?

## Scope

- `echo-core/src/llm/mod.rs` (LlmClient trait, ChatRequest/ChatResponse/
  ChatChunk, ToolChoice, SimpleChatOptions), `types.rs` (wire types + 56
  unit tests), `thinking.rs`, `capabilities.rs`, `cache/{mod,layout}.rs`.
- `echo-integration/src/providers/traits.rs` (ProviderAdapter contract),
  `adapter_client.rs`, `thinking_translate.rs`, `client.rs` (post/stream_post
  transport), `config.rs` (LlmConfig/LlmProvider/ModelConfig/ProviderFactory).
- Root facade `echo-agent/src/llm.rs`.
- Usage/event authority anchors: `echo-core/src/agent/mod.rs:157-177`
  (AgentEvent::LlmUsage), `src/agent/react/run/phases/think.rs`,
  `run/stream_channel.rs`, `run/react_loop.rs`, `run/context.rs`.

## Out Of Scope

- Provider adapter internals (`openai.rs`, `anthropic.rs`, `anthropic_cache.rs`)
  — delegated to F-LLM-02 / F-LLM-03; only the chunk-mapping boundary
  (chat_stream → ChatChunk) was read for V02.
- Core streaming loop state machine and cancellation ordering — F-RCT-03.
- EKO-side usage aggregation/display — A-* tasks.

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, framework/application layering,
  no-parallel-semantics), shared `REPORTING.md`, `TASKS.md` (F-LLM-01 card),
  `zcode-ds/README.md`.
- Dependency report: zcode-ds `F-CORE-01` (LlmUsage/usage_reported semantics,
  error taxonomy), `B-ARCH-01` (facade ownership, `src/llm.rs` re-export map).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md`
  (M9 usage authority), `echo-agent/AUDIT_REPORT.md` (unsafe set_var claim).

## Layering Decision

- Generic mechanism: the neutral LLM contract (`LlmClient` + wire types +
  `Usage` normalization + `ThinkingConfig` translation) is framework-core
  and provider-neutral — correctly placed in `echo_core`.
- EKO product policy: none in this task.
- Adapter boundary: `echo_integration::providers` adapters map the neutral
  contract to vendor wires; `traits.rs`+`adapter_client.rs` is the declared
  adapter contract.
- Duplicate search terms: `ThinkingProtocol`, `ThinkingProtocolPreference`,
  `ProviderAdapter`, `AdapterClient`, `ChatCompletionResponse.extra`,
  `LlmUsage`, `usage_reported`, `cache_hints` — two parallel thinking-protocol
  authorities and one dead adapter contract found (P2-03); everything else
  single-authority.
- Cross-repository boundary gate: all findings below stay in `echo-agent`
  (framework/adapter); nothing moves between repositories.

## Current Path

`LlmClient` (trait) → adapters (`OpenAiClient`/`DefaultLlmClient`/
`AnthropicClient`) → shared transport `post`/`stream_post` (client.rs) →
`ChatCompletionRequest`/`ChatCompletionResponse`/`ChatCompletionChunk` wire
types → `ChatRequest`/`ChatResponse`/`ChatChunk` neutral contract → core
loop (`think.rs` streaming path, `react_loop.rs` non-streaming paths).
Usage flows: provider `usage` → `Usage` normalization → `last_usage` →
`AgentEvent::LlmUsage{...,usage_reported}` (think.rs:201, stream_channel.rs:466)
→ subagent executor / EKO sinks. Thinking flows: `ChatRequest.thinking` →
`translate_thinking_openai_compat` (model-name/provider-driven
`resolve_thinking_protocol`) → `reasoning_effort`/`enable_thinking`/
`thinking_budget`/`glm_thinking` wire fields. Cancellation: `cancel_token`
→ transport stops at SSE boundary (silent end). Errors: `LlmError` (5
variants) → `ReactError` boxed.

## Findings

### F-LLM-01-P1-01: Streaming transport silently drops malformed SSE chunks, diverging from non-streaming error handling

- Priority: P1
- Confidence: medium
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/client.rs:99-105`
  (serde failure → `warn!` + `None`, chunk dropped, stream continues);
  `client.rs:163-164` (non-streaming `post()` fails with
  `LlmError::InvalidResponse` on the same malformed body);
  `client.rs:76-107` (`parse_sse_chunk`); lenient parse entry point
  `echo-core/src/llm/types.rs:802-808` (`deserialize_null_as_default`,
  whose doc comment at `:814-821` explicitly describes the historical
  silent-usage-loss bug class)
- Reachability: `stream_post` is the transport for `OpenAiClient::chat_stream`
  (openai.rs:346), `DefaultLlmClient::chat_stream`, `AdapterClient::chat_stream`
  → every streaming LLM call in the framework main path
  (`think.rs:99-103` via `retry_llm_call`). A chunk failing
  `serde_json::from_str::<ChatCompletionChunk>` (e.g. `"content": 123`,
  `"usage": "x"`, `"choices": {}`) is dropped mid-stream; the stream
  terminates normally and the caller merges nothing from it.
- Expected invariant: the streaming contract preserves all provider data or
  fails loudly; streaming and non-streaming must not diverge on malformed
  input.
- Observed behavior: silent content/usage loss with only a `warn!` log; a
  reported usage chunk that fails to parse makes the whole turn report
  `usage_reported: false`; the same body in `chat()` returns
  `LlmError::InvalidResponse`.
- Impact: silent content loss can produce a wrong final answer; usage
  observability mis-signals; the caller cannot distinguish "provider ended"
  from "chunk dropped".
- Root cause: lenient-default parsing combined with drop-instead-of-error in
  the shared SSE transport; the `deserialize_null_as_default` fix covered one
  malformed shape (null delta) but the general drop path remains.
- Direction: make `parse_sse_chunk` return a distinguishable result (typed
  `LlmError::InvalidResponse` with the offending body, or a drop counter the
  loop can surface); align streaming with the non-streaming error contract;
  add malformed-chunk fixtures (wrong-typed content/usage, object choices).
- Regression validation: unit tests in `client.rs` feeding wrong-typed SSE
  chunks through `parse_sse_chunk` asserting an error (or a counted drop);
  a loop-level test asserting `usage_reported` is not falsely false when the
  provider did report usage.
- Validation reports: [V02](../validations/F-LLM-01/V02-01.md),
  [V04](../validations/F-LLM-01/V04-01.md)

### F-LLM-01-P2-01: `ChatCompletionResponse.extra` is never populated; `ChatResponse.raw` silently drops unmodeled provider fields

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-core/src/llm/types.rs:618-638` (`ChatCompletionResponse`
  with `extra: Option<serde_json::Value>` bound only to a literal `"extra"`
  key via `#[serde(default)]`; no `#[serde(flatten)]` catch-all);
  `echo-core/src/llm/mod.rs:207-210` (`ChatResponse.raw` doc: "Raw provider
  response for callers needing extra metadata"); repo-wide grep: zero
  readers/writers of `.extra` on this type
- Reachability: `post()` (`client.rs:163-164`) deserializes every
  non-streaming response; `ChatResponse.raw` is exposed to all callers
  (e.g. `react_loop.rs:100` reads `response.raw.usage`). Unknown provider
  fields (e.g. `system_fingerprint`, `logprobs`, reasoning metadata) are
  silently dropped on every call.
- Expected invariant: "raw" preserves the provider payload, or the doc says
  it models a fixed subset.
- Observed behavior: unknown fields are discarded; `extra` stays `None`;
  the doc promises more than the type delivers.
- Impact: consumers needing unmodeled metadata have no channel; the dead
  `extra` field invites misuse; provider-neutrality claim is weaker than
  documented.
- Root cause: serde behavior — `extra` only binds a literal `"extra"` key;
  no catch-all collection was implemented.
- Direction: either delete `extra` and re-document `raw` as "modeled fields
  only", or implement a custom `Deserialize`/`#[serde(flatten)]` catch-all
  map so `extra` genuinely captures unknown fields.
- Regression validation: unit test deserializing a response containing an
  unknown field and asserting it lands in `extra` (or the doc change).
- Validation reports: [V01](../validations/F-LLM-01/V01-01.md)

### F-LLM-01-P2-02: Usage observability is path-dependent — `AgentEvent::LlmUsage` is not emitted on direct_answer/structured-output paths

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `src/agent/react/run/react_loop.rs:763-813` (`direct_answer`
  records `RunEvent::LlmCall` + token_tracker but never emits
  `AgentEvent::LlmUsage`); `react_loop.rs:100-102` (`call_llm_with_retry`
  returns the usage tuple, consumed only at `:763-781`); contrast
  `src/agent/react/run/phases/think.rs:199-211` and
  `src/agent/react/run/stream_channel.rs:465-473` (emit `LlmUsage` with
  `usage_reported`); consumer `src/agent/subagent/executor.rs:1253`
  (aggregates usage from `AgentEvent::LlmUsage` → `DispatchLlmUsage`)
- Reachability: `direct_answer` is used by IntentRouter for simple intents;
  `pre_compaction_flush` (`run/context.rs:734`) also calls `llm_client.chat`
  and discards usage; every turn on these paths produces no LlmUsage event.
- Expected invariant: every provider usage report reaches consumers through
  the typed event with `usage_reported` semantics (F-CORE-01 V01:
  "LlmUsage cache observability").
- Observed behavior: usage on these paths exists only in the `RunEvent`
  trace; event-based consumers (subagent executor, EKO chat surface) see
  zero usage for these turns and cannot distinguish "provider didn't report"
  from "path doesn't emit".
- Impact: token accounting, budget tracking, and cache-hit-rate
  observability undercount on direct-answer/structured turns; the
  `usage_reported` contract is honored only on the streaming main path.
- Root cause: LlmUsage emission was added to the streaming loop paths; the
  non-streaming paths were not wired to emit the event.
- Direction: emit `AgentEvent::LlmUsage` from `direct_answer` (and decide
  explicitly for `pre_compaction_flush`) with `usage_reported =
  usage.is_some()`, reusing the same normalization as `think.rs:127-147`.
- Regression validation: mock-client test where `direct_answer` receives
  usage → assert an `LlmUsage` event with `usage_reported: true`; a
  no-usage variant asserting `usage_reported: false`.
- Validation reports: [V03](../validations/F-LLM-01/V03-01.md),
  [V04](../validations/F-LLM-01/V04-01.md)

### F-LLM-01-P2-03: ProviderAdapter/AdapterClient is a dead provider contract carrying a second thinking-protocol authority

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-integration/src/providers/traits.rs:22-69` (`ProviderAdapter`
  incl. `thinking_protocol()` :47-49, `drops_temperature_on_thinking` :52-57);
  `traits.rs:91-107` (`ThinkingProtocolPreference`); `adapter_client.rs:71-186`
  (`AdapterClient<A>` LlmClient impl never calls `thinking_protocol()` —
  always `translate_thinking_openai_compat` with the echo_core resolver,
  `:77-82`, `:132-137`); repo-wide grep (echo-agent + echo-agent-cli): zero
  `impl ProviderAdapter`, zero `AdapterClient::new`; still re-exported as
  public API (`providers/mod.rs:10`, root `src/llm.rs:64-66`)
- Reachability: defined and re-exported, never constructed or instantiated
  at runtime; `ThinkingProtocolPreference` has zero uses outside traits.rs.
- Expected invariant: a single authoritative provider contract; no parallel
  semantics for the same decision (AGENTS.md: 严禁平行实现同一语义; the
  B-ARCH-01 "zero parallel implementations" claim needs this carve-out).
- Observed behavior: the task card's "provider 适配契约" is unregistered dead
  code; if a provider adapter were written against it, its declared thinking
  protocol would be ignored by the transport (which uses the echo_core
  model-name resolver via `translate_thinking_openai_compat`).
- Impact: misleading public API surface; duplicate thinking-protocol
  authority invites future divergence between `ThinkingProtocol` and
  `ThinkingProtocolPreference`; dead code to maintain.
- Root cause: the adapter mechanism (mirroring Hermes's ProviderProfile)
  predates the echo_core model-profile resolver and was never adopted by the
  concrete clients.
- Direction: delete `traits.rs` + `adapter_client.rs` and their re-exports
  (with X-BND-01 confirming no external consumer), or wire
  `thinking_protocol()` into the transport and implement real adapters;
  the authoritative protocol stays `echo_core::llm::thinking::ThinkingProtocol`.
- Regression validation: grep for `ProviderAdapter`/`AdapterClient`/
  `ThinkingProtocolPreference` returns zero code hits after removal;
  `cargo check -p echo_integration` and the full framework gate stay green.
- Validation reports: [V01](../validations/F-LLM-01/V01-01.md)

### F-LLM-01-P3-01: `LlmError` has no typed timeout/cancellation — timeout is text in NetworkError, cancellation ends the stream silently

- Priority: P3
- Confidence: high
- Layer: framework (error taxonomy) + adapter (transport)
- Evidence: `echo-core/src/error.rs:87-107` (`LlmError` = NetworkError/
  ApiError/InvalidResponse/EmptyResponse/SerializationError — no
  Cancelled/Timeout variants); `client.rs:29-34` (`timeout_error` → text
  inside `LlmError::NetworkError`); `client.rs:251-256` (cancelled token →
  silent `return`, no terminal chunk/error); `client.rs:81-86` + `:329,347`
  (`[DONE]` turned into an error whose payload is matched by string
  containment); contract doc `echo-core/src/llm/mod.rs:173-175`
  ("streaming responses will stop at the next SSE boundary")
- Reachability: every streaming call; exercised whenever a turn is cancelled
  mid-stream or a provider stalls.
- Expected invariant: typed error semantics for cancellation/timeout at the
  LLM contract, matching the typed `AgentError::Cancelled/Timeout` and
  `ToolError::Timeout` taxonomy (F-CORE-01 V01).
- Observed behavior: timeout is an untyped text string; cancellation yields
  a normal end-of-stream with no signal; stream-end detection relies on a
  sentinel string inside an error.
- Impact: stream consumers cannot distinguish cancelled/timeout/normal-end
  from the contract itself; text-based matching is fragile against providers
  that emit `[DONE]`-like payloads or unexpected final data.
- Root cause: transport-level shortcuts (sentinel-as-error, silent return on
  cancel) predate the typed-error taxonomy.
- Direction: add `LlmError::Cancelled`/`LlmError::Timeout` in echo_core and
  propagate them from the transport; replace sentinel string matching.
- Regression validation: unit tests for `stream_post` with a pre-cancelled
  token and with an idle timeout asserting typed errors.
- Validation reports: [V02](../validations/F-LLM-01/V02-01.md),
  [V04](../validations/F-LLM-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---:|---|
| V01 | Field/variant matrix + serde loss + duplicate search | yes | passed | [V01](../validations/F-LLM-01/V01-01.md) |
| V02 | Streaming-neutrality trace | yes | passed | [V02](../validations/F-LLM-01/V02-01.md) |
| V03 | Usage/cache authority vs `usage_reported` | yes | passed | [V03](../validations/F-LLM-01/V03-01.md) |
| V04 | `cargo test -p echo_core --lib --locked llm` + `cargo test -p echo_integration --lib --locked providers` | yes | passed (exit 0 / 0) | [V04](../validations/F-LLM-01/V04-01.md) |
| V05 | Historical-document drift | conditional | performed inline | see Historical Claim Status |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN M9: "provider/API usage 是记账权威;缺失时标记 unknown" | current (streaming main paths) / regressed on non-streaming paths | [V03](../validations/F-LLM-01/V03-01.md) — think.rs/stream_channel.rs honor `usage_reported`; direct_answer/structured emit no LlmUsage event (P2-02) |
| MASTER-PLAN M9: 统一 prompt/cached/cache creation/output/total 的 provider 归一语义 | current | `Usage` normalization, types.rs:686-772, tests at types.rs:1086-1231 |
| MASTER-PLAN M9: "各 provider fixture 覆盖 cache token 语义" | current (unit-level) / fixture-level adapter tests owned by F-LLM-02/03 | [V04](../validations/F-LLM-01/V04-01.md) |
| `AUDIT_REPORT.md` §4.1: `providers/config.rs` unsafe `set_var`/`remove_var` "in tests + production" | stale for the "production" part | verified `config.rs:1246-1309` — `unsafe` env mutation exists only in `#[cfg(test)]` `EnvGuard` (mutex-guarded); production code only reads env |

## Coverage And Uncertainty

- `openai.rs`/`anthropic.rs` internals not reviewed (F-LLM-02/03); only the
  `ChatChunk` mapping boundary was read — adapter-side delta fidelity claims
  here are limited to the shared transport (`client.rs`).
- F-LLM-01-P1-01 confidence is medium: the drop path is confirmed by code
  inspection, but real-provider malformed-chunk frequency is unknown; the
  warn log makes it partially observable.
- Whether `cache_hints` being OpenAI-adapter-ignored (Anthropic-only) is a
  contract defect is left to F-LLM-02/03 (adapter fidelity), recorded in V01.
- Loop-level cancellation recovery and stream termination ordering are
  F-RCT-03 scope; this task only attests the contract-level signal absence
  (P3-01).
- No `echo-agent-cli` code was modified or deeply read; the CLI-side usage
  display contract is an A-* task.

## Handoff

- Downstream tasks may rely on: neutral contract inventory (V01); streaming
  losslessness on well-formed data + the single silent-drop path (V02);
  usage authority + `usage_reported` semantics on streaming paths (V03);
  both test commands green at the reviewed commits (V04).
- F-LLM-02 must check whether the OpenAI adapter's request assembly can
  produce the malformed-shape cases P1-01 warns about, and own the
  malformed-chunk fixture tests at the `parse_sse_chunk` boundary.
- F-LLM-03 should confirm Anthropic usage mapping feeds `ChatChunk.usage`
  (V03 depends on it) and cache_hints consumption.
- F-RCT-03 should treat cancellation as end-of-stream with no typed signal
  (P3-01) when verifying loop recovery.
- X-BND-01 should confirm deletion of `traits.rs`/`adapter_client.rs` has no
  external consumer (P2-03) and settle the `ChatResponse.raw` contract
  (P2-01).
- A task owning token accounting (A-* or Q-*) should verify the P2-02
  undercount impact on EKO usage display.
- This report becomes stale if the LlmClient trait, the wire types, the
  transport error handling, or the LlmUsage emission paths change.
