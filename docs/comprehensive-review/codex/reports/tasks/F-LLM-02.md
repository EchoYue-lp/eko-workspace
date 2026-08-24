# F-LLM-02: OpenAI provider adapter

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d3`
> `echo-agent-cli` commit: `b3b2e81f2b2d`
> Worktree state: both source repositories clean; review reports are outside both source repositories

## Question

Does the OpenAI-compatible adapter faithfully implement the neutral contract
for request construction, non-stream/stream responses, tool calls, usage,
failures, cancellation, and live framework/EKO callers?

## Scope

- `echo-agent/echo-integration/src/providers/openai.rs` and the shared provider
  transport/client.
- OpenAI wire types in `echo-core`, compatible thinking translation/config/
  factory, root facade, and live ReAct mapping/assembly.
- EKO provider connection/doctor paths and its document-to-message boundary.
- Repository-wide compatible-client definition, duplicate, registration,
  construction, call-path, and targeted-test searches.
- Private localhost protocol fixtures for request capture, parallel tool deltas,
  usage, raw/response facts, HTTP/JSON/SSE errors, and non-stream cancellation.

## Out Of Scope

- Implementing fixes or changing source/public APIs.
- Provider-neutral defects already owned by F-LLM-01: split UTF-8/malformed SSE,
  pending streaming cancellation, non-first choice/identity loss, typed named
  tool choice, malformed neutral messages, generic adapter duplication, and
  usage/cache authority.
- Retry/circuit-breaker defects already owned by F-REL-01.
- Anthropic-specific mapping/cache/thinking (`F-LLM-03`).
- Exhaustive ReAct terminal/event semantics after normalized chunks
  (`F-RCT-02/03`).
- Live external-provider calls and any security attack/vulnerability analysis.
- Time-sensitive assertions about the current OpenAI `max_tokens` versus
  `max_completion_tokens` field: official documentation was inaccessible (V14).

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md`; Codex track
  rules in `codex/README.md`.
- Dependency [F-LLM-01](F-LLM-01.md), including its required F-CORE-01 boundary;
  provider-neutral findings were used only to prevent duplication.
- Dependency [F-REL-01](F-REL-01.md), limited to retry/cancellation authority
  boundaries inherited by the shared transport.
- No other reviewer directory or report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Neutral request/response/chunk/tool/usage/error/cancellation types and explicit loss/unsupported outcomes belong to reusable framework crates. |
| EKO product policy | Selected model/provider, credentials, inline-vs-reference resource delivery, UI/TUI cancellation, and product connectivity diagnostics belong to EKO. |
| Adapter boundary | The OpenAI adapter may translate wire names/shapes and capability differences, but must preserve every accepted neutral fact or return a typed unsupported/invalid-response result. It must not silently synthesize success or erase an inline resource. |
| Duplicate search | Names, traits, types, fields, constructors, re-exports, request builders, transport functions, and live call paths were searched across both repositories. Three public compatible-provider adaptation routes remain: concrete `OpenAiClient`, free functions plus `DefaultLlmClient`, and the disconnected generic `AdapterClient`. |
| Migration deletion | Keep one concrete built-in adapter authority. Migrate live free-helper callers (legacy ReAct and EKO doctor), then delete `DefaultLlmClient` and duplicated free-function request assembly if no distinct public framework use remains. `AdapterClient` migration/deletion remains owned by F-LLM-01-P2-10. |

`OpenAiClient` and public framework choices were not called dead merely because
EKO does not use every constructor. The duplicate finding rests on framework-
wide semantic divergence and a complete internal construction search.

## Current Path

```text
EKO model/provider config or framework builder
  -> LlmConfig::build_client -> OpenAiClient
  -> ChatRequest
       chat        -> ChatCompletionRequest -> shared post
       chat_stream -> ChatCompletionRequest -> shared stream_post
  -> permissive OpenAI-shaped serde types
  -> first Choice / first ChunkChoice -> ChatResponse / ChatChunk
  -> ReAct buffers tool deltas by index; usage owner is last usage chunk

Legacy compatibility path
  EKO doctor / no-client ReAct
  -> echo_agent::llm::{chat,stream_chat}
  -> provider free functions
  -> same shared transport

EKO inline document path
  PreparedUserTurn(Delivery::Inline)
  -> ContentPart::File(base64)
  -> OpenAI normalize
  -> text-extension inline text OR name-only placeholder
```

Normal mapping evidence is positive: V04 captures tools/tool choice/JSON schema/
thinking/user ID/token limit and temperature suppression; V06 preserves two
interleaved tool indices, argument fragments, finish reason, and the choices-
empty usage chunk; V12/V13 preserve HTTP 400 facts and reject invalid JSON.
The failed boundaries are not explained by the inherited first-choice or SSE
framing defects: well-formed error envelopes and refusal/extra fields are
accepted by permissive structs and discarded before neutral mapping.

## Findings

### F-LLM-02-P1-01: Well-formed provider error envelopes become unrelated or successful empty responses

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-core/src/llm/types.rs:617`,
  `types.rs:787`, `echo-agent/echo-integration/src/providers/client.rs:163`,
  `client.rs:317`, `echo-agent/echo-integration/src/providers/openai.rs:310`,
  `openai.rs:368`
- Reachability: live `OpenAiClient::{chat,chat_stream}` and compatibility
  helpers both use the shared decoders. Local non-stream and SSE loopback
  fixtures supplied well-formed JSON with top-level `error`.
- Expected invariant: an error envelope terminates with a provider/invalid-
  response error retaining available message/type/code; it cannot enter a
  successful completion/chunk value.
- Observed behavior: response/chunk structs default `id`/`choices` and ignore
  unknown fields. Non-stream decoding succeeds then reports generic
  `EmptyResponse`, losing `provider overloaded`; streaming emits one successful
  empty `ChatChunk` and no error.
- Impact: provider failures are misclassified or appear as successful empty
  stream progress. Callers lose actionable provider diagnostics and can make
  incorrect retry/terminal decisions.
- Root cause: success and error envelopes share a permissive success-only serde
  target with no validated wire boundary.
- Direction: decode an explicit success/error union before neutral conversion;
  require the structural fields of a completion/chunk; share error
  normalization between stream and non-stream paths.
- Regression validation: status-success error envelopes carrying every error
  field in non-stream/SSE plus empty/missing-choices successful bodies; assert
  one typed terminal error and zero successful chunks.
- Validation reports: [V03](../validations/F-LLM-02/V03-01.md),
  [V09](../validations/F-LLM-02/V09-01.md),
  [V10](../validations/F-LLM-02/V10-01.md),
  [V12](../validations/F-LLM-02/V12-01.md),
  [V13](../validations/F-LLM-02/V13-01.md)

### F-LLM-02-P1-02: OpenAI refusal responses are accepted after the refusal fact is erased

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-core/src/llm/types.rs:238`,
  `types.rs:282`, `types.rs:617`,
  `echo-agent/echo-integration/src/providers/openai.rs:310`
- Reachability: the live non-stream client clones the first provider message
  into `ChatResponse`. A loopback response with `content:null` and a refusal
  field exercised this exact path.
- Expected invariant: a provider-declared refusal is represented distinctly or
  rejected explicitly; it cannot become indistinguishable from missing data.
- Observed behavior: `RawMessage` has no refusal field and ignores it. Null
  content becomes `MessageContent::Empty`; the public call succeeds, and the
  normalized message serializes as only `{"role":"assistant"}`.
- Impact: framework consumers cannot explain a refusal, distinguish it from a
  malformed/empty response, or reliably choose user-visible and retry behavior.
- Root cause: an OpenAI response message is deserialized directly into the
  narrower conversation `Message` without a lossless provider response model.
- Direction: preserve refusal as a neutral response block/fact (or provider raw
  metadata) and define its `chat_simple`/ReAct behavior; do not accept it after
  silently discarding the only semantic payload.
- Regression validation: content refusal, structured-output refusal, streamed
  refusal deltas, refusal plus metadata, and ordinary empty tool-call messages.
- Validation reports: [V03](../validations/F-LLM-02/V03-01.md),
  [V08](../validations/F-LLM-02/V08-01.md)

### F-LLM-02-P1-03: Inline non-image documents reach OpenAI-compatible models as filename-only placeholders

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/prepared_turn.rs:162`,
  `prepared_turn.rs:257`, `prepared_turn.rs:335`,
  `echo-agent/echo-integration/src/providers/openai.rs:22`,
  `openai.rs:53`, `openai.rs:69`
- Reachability: GUI/TUI/CLI prepared-turn paths classify normal uploads as
  `Delivery::Inline`, read their bytes, and create `ContentPart::File`. The live
  concrete/legacy OpenAI paths both call `normalize_messages`.
- Expected invariant: a resource selected for inline delivery reaches the model
  as bytes/text, or conversion retains a model-readable reference/retrieval
  path when the wire cannot represent the resource.
- Observed behavior: only an extension allowlist is base64-decoded to text.
  PDF and every other binary/non-allowlisted document are replaced with
  `[Attachment: name]`; their bytes and path are removed. The existing unit
  test treats this filename-only result as expected.
- Impact: users can attach a PDF/document and the agent sees that a file exists
  but cannot inspect its contents, violating EKO attachment capability parity.
- Root cause: EKO decides inline delivery before provider capabilities are
  known, then the adapter resolves incompatibility by silent destructive
  normalization instead of a lossless reference/typed unsupported outcome.
- Direction: make application delivery capability-aware. Use a supported
  provider file/document representation, extract content, or convert to a
  durable tool reference before constructing the request; delete the
  filename-only success fallback.
- Regression validation: GUI/TUI/CLI and subagent paths for PDF, office/binary,
  allowlisted UTF-8/non-UTF-8 text, and unsupported documents; assert model-
  visible bytes/text or retrievable reference, never filename-only success.
- Validation reports: [V03](../validations/F-LLM-02/V03-01.md),
  [V16](../validations/F-LLM-02/V16-01.md)

### F-LLM-02-P1-04: Non-stream ChatRequest cancellation is silently ignored

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-core/src/llm/mod.rs:148`,
  `echo-agent/echo-integration/src/providers/openai.rs:263`,
  `openai.rs:280`, `echo-agent/echo-integration/src/providers/client.rs:109`
- Reachability: `OpenAiClient::chat` accepts the unified public `ChatRequest`.
  It constructs the wire request without consuming `cancel_token`, then calls
  `post`, which has no token parameter. Several live non-stream paths construct
  requests, although some currently set the token to `None`.
- Expected invariant: an accepted cancellation control interrupts the in-flight
  operation, or the API explicitly rejects/does not expose that control for the
  operation.
- Observed behavior: cancellation after 30 ms during a delayed 350 ms response
  did nothing; the call returned successful content `late` after 355.98 ms.
- Impact: framework consumers using the advertised unified request cannot stop
  non-stream work and can observe results after cancellation.
- Root cause: cancellation was implemented only in the streaming transport,
  while one request shape exposes it to both trait methods without capability
  validation.
- Direction: race the token against non-stream send/body I/O and return a typed
  cancellation terminal; alternatively split request controls so unsupported
  options cannot be silently accepted.
- Regression validation: cancel before send, while awaiting headers/body,
  during retry backoff (coordinated with F-REL-01), after completion, and on
  both concrete/compatibility routes.
- Validation reports: [V02](../validations/F-LLM-02/V02-01.md),
  [V03](../validations/F-LLM-02/V03-01.md),
  [V11](../validations/F-LLM-02/V11-01.md)

### F-LLM-02-P2-05: ChatResponse.raw does not retain the extra provider metadata it promises

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/llm/mod.rs:201`,
  `mod.rs:208`, `echo-agent/echo-core/src/llm/types.rs:617`,
  `types.rs:635`, `types.rs:640`
- Reachability: every compatible non-stream response stores
  `ChatCompletionResponse` in public `ChatResponse.raw`. The fixture used the
  same live decode/client path.
- Expected invariant: the field documented for callers needing extra metadata
  retains unmodeled provider response facts.
- Observed behavior: `extra` is a normal JSON property named `extra`, not a
  flattened map. `object`, `system_fingerprint`, `service_tier`, and choice
  `logprobs` were ignored; `raw.extra` remained `None`.
- Impact: observability/correlation consumers receive fabricated absence and
  cannot recover provider metadata from the public response.
- Root cause: a narrowed typed projection is labeled raw, while serde's unknown-
  field behavior discards everything outside it.
- Direction: retain the actual provider JSON or use flattened extra maps at
  each relevant nesting level; otherwise rename/document the value as a lossy
  projection and expose an honest raw channel.
- Regression validation: round-trip unknown top-level, choice, message, usage,
  and nested detail fields without weakening required-field validation.
- Validation reports: [V03](../validations/F-LLM-02/V03-01.md),
  [V07](../validations/F-LLM-02/V07-01.md)

### F-LLM-02-P2-06: Public compatible-client routes have divergent request and convenience semantics

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-integration/src/providers/openai.rs:139`,
  `openai.rs:175`, `openai.rs:263`, `openai.rs:386`, `openai.rs:401`,
  `openai.rs:472`, `echo-agent/src/llm.rs:99`,
  `echo-agent/echo-integration/src/providers/config.rs:301`
- Reachability: factory/builder/EKO connectivity constructs `OpenAiClient`;
  legacy ReAct and EKO doctor call free helpers. `DefaultLlmClient` is publicly
  re-exported from integration/root but has no internal construction point.
- Expected invariant: compatibility routes delegate to one field-complete
  authority and preserve trait defaults/semantics.
- Observed behavior: `DefaultLlmClient` explicitly drops thinking; free helpers
  have separate request builders with all thinking fields hardcoded absent;
  `chat` even accepts a stream flag while using the non-stream decoder.
  `DefaultLlmClient::chat_simple` changes the trait default temperature 0.7 to
  0.3 and turns empty content into a different error.
- Impact: behavior depends on which equally public entry point a framework
  consumer chooses; fixes to concrete translation can leave compatibility
  paths stale.
- Root cause: migration added a configured concrete client without retiring or
  reducing the prior free-function/client authority.
- Direction: move all live callers to the concrete `LlmClient`; retain only thin
  compatibility wrappers that construct/delegate to it if a framework-wide
  need is demonstrated, then delete duplicate request assembly and
  `DefaultLlmClient`.
- Regression validation: repository construction search plus captured request/
  response parity for every retained entry point and trait convenience method.
- Validation reports: [V01](../validations/F-LLM-02/V01-01.md),
  [V02](../validations/F-LLM-02/V02-01.md),
  [V03](../validations/F-LLM-02/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate-authority search | yes | failed invariant | [V01](../validations/F-LLM-02/V01-01.md) |
| V02 | Registration and real runtime reachability | yes | passed | [V02](../validations/F-LLM-02/V02-01.md) |
| V03 | Field/variant/edge-case/test matrix | yes | failed invariant | [V03](../validations/F-LLM-02/V03-01.md) |
| V04 | Non-stream request capture | yes | passed after environment retry | [attempt 01](../validations/F-LLM-02/V04-01.md), [attempt 02](../validations/F-LLM-02/V04-02.md) |
| V05 | Private fixture compile | yes | passed | [V05](../validations/F-LLM-02/V05-01.md) |
| V06 | Parallel tool-delta and usage stream | yes | passed | [V06](../validations/F-LLM-02/V06-01.md) |
| V07 | Raw metadata retention | yes | failed invariant | [V07](../validations/F-LLM-02/V07-01.md) |
| V08 | Refusal response preservation | yes | failed invariant | [V08](../validations/F-LLM-02/V08-01.md) |
| V09 | Non-stream error-envelope normalization | yes | failed invariant | [V09](../validations/F-LLM-02/V09-01.md) |
| V10 | Stream error-envelope normalization | yes | failed invariant | [V10](../validations/F-LLM-02/V10-01.md) |
| V11 | Non-stream cancellation | yes | failed invariant | [V11](../validations/F-LLM-02/V11-01.md) |
| V12 | HTTP 400 status/body mapping | yes | passed | [V12](../validations/F-LLM-02/V12-01.md) |
| V13 | Malformed JSON rejection | yes | passed | [V13](../validations/F-LLM-02/V13-01.md) |
| V14 | Current official OpenAI documentation and browser cleanup | conditional | inconclusive after isolated attempts | [grouping correction](../validations/F-LLM-02/V14-01.md), [search](../validations/F-LLM-02/V14-02.md), [direct open](../validations/F-LLM-02/V14-03.md), [sandbox HTTPS](../validations/F-LLM-02/V14-04.md), [approved platform HTTPS](../validations/F-LLM-02/V14-05.md), [developers HTTPS](../validations/F-LLM-02/V14-06.md), [browser setup](../validations/F-LLM-02/V14-07.md), [browser page](../validations/F-LLM-02/V14-08.md), [browser cleanup](../validations/F-LLM-02/V14-09.md) |
| V15 | Private target cleanup | yes | passed | [V15](../validations/F-LLM-02/V15-01.md) |
| V16 | EKO inline-document delivery trace | yes | failed invariant | [V16](../validations/F-LLM-02/V16-01.md) |
| V17 | Final links/isolation/source/session gate | yes | passed after unavailable OS observation | [attempt 01](../validations/F-LLM-02/V17-01.md), [attempt 02](../validations/F-LLM-02/V17-02.md) |
| V18 | Post-gate complete-chain check | yes | passed | [V18](../validations/F-LLM-02/V18-01.md) |
| V20 | Primary error/refusal envelope reconstruction | yes | failed invariant | [V20-01](../validations/F-LLM-02/V20-01.md) |
| V20 | Primary inline-document trace | yes | failed invariant | [V20-02](../validations/F-LLM-02/V20-02.md) |
| V20 | Primary non-stream cancellation trace | yes | failed invariant | [V20-03](../validations/F-LLM-02/V20-03.md) |
| V20 | Primary raw/duplicate-authority reconstruction | yes | failed invariant | [V20-04](../validations/F-LLM-02/V20-04.md) |

There are 19 validation IDs and 32 immutable attempts.
No workspace build was run. Every Cargo command used
`/private/tmp/f-llm-02-target`, which V15 removed after execution.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-LLM-01-P1-01: shared compatible streaming corrupts split UTF-8/drops malformed events | current, inherited; not duplicated | dependency report and F-LLM-01 V11/V12 |
| F-LLM-01-P1-02: pending stream I/O ignores cancellation | current, inherited; non-stream analogue newly established | [V11](../validations/F-LLM-02/V11-01.md), P1-04 |
| F-LLM-01-P2-08: compatible neutral stream discards response/candidate identity | current, inherited; normal tool/usage first-choice path passes | [V06](../validations/F-LLM-02/V06-01.md) |
| F-LLM-01-P2-09/P2-10: typed tool choice and generic provider duplicate authority | current, inherited; not duplicated | [V01](../validations/F-LLM-02/V01-01.md) |
| F-REL-01-P2-03: nested retry budgets | current in shared post/stream_post; not re-executed | shared client call trace in [V02](../validations/F-LLM-02/V02-01.md) |

## Coverage And Uncertainty

All neutral request fields were mapped through the concrete adapter and the
parallel compatibility paths were searched. Non-stream/stream happy paths,
parallel tool calls, terminal usage, HTTP error, syntax error, semantic error
envelopes, response metadata/refusal, and non-stream cancellation were exercised
with deterministic local fixtures. Existing OpenAI/shared client tests and the
live ReAct/EKO caller boundaries were inspected.

No external provider was called. Official OpenAI documentation was inaccessible
through both official retrieval paths and browser, so current model-specific
field support remains deliberately unclassified. The loopback error envelopes
are malformed/external-provider fixtures proving the adapter's own validation
contract; this report does not assert that every upstream OpenAI endpoint emits
HTTP 200 errors. Stream UTF-8/framing, first-choice identity, retry, and pending
stream cancellation rely on the named dependency reports and were not rebuilt.

Primary independently reconstructed all six findings through the current wire
types, adapter code and live application/framework callers in V20-01 through
V20-04. Delegated loopback fixtures retain the time-dependent and concrete wire
effects. V17-02 establishes that this task owns no open Cargo/exec session. Disk
availability was below the repository threshold; no shared target was used or
cleaned.

## Handoff

- F-LLM-03 should reuse the explicit success/error-envelope and raw metadata
  acceptance criteria for Anthropic without assuming OpenAI wire shapes.
- F-RCT-02/03 should consume typed refusal/error/cancellation terminals and must
  not compensate for a provider adapter that emits successful empty values.
- EKO attachment tasks should decide provider-capable inline/reference delivery
  before constructing the framework message; subagent paths need the same test.
- A fix should migrate real factory/legacy/doctor callers to one `OpenAiClient`
  request builder and delete divergent compatibility code in the same staged
  migration.
- This report becomes stale when core LLM response/message/chunk types, OpenAI
  provider/shared client/config/factory, ReAct request/stream mapping, EKO
  prepared turns/attachments/provider probes, or either reviewed commit changes.
