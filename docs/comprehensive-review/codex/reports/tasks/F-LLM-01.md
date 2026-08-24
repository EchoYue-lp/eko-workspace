# F-LLM-01: Provider-neutral LLM contract

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d3`
> `echo-agent-cli` commit: `b3b2e81f2b2d`
> Worktree state: both source repositories clean; review reports are outside both source repositories

## Question

Can provider implementations preserve messages, tools, thinking, structured
output, usage, caching, streaming, cancellation, and errors without semantic
loss through the real framework and EKO adapter paths?

## Scope

- `echo-agent/echo-core/src/llm`: neutral traits, request/response/message/tool/
  usage/chunk types, capabilities, thinking, and cache diagnostics/layout.
- `echo-agent/echo-integration/src/providers`: shared transport, concrete
  OpenAI-compatible and Anthropic clients, provider traits/adapter, factory,
  configuration, and current tests.
- Root facade and live ReAct request/stream reconstruction paths.
- EKO model resolution, LLM config construction, initial agent creation,
  pooled-agent construction, and later model-apply adapters.
- Static field/variant, provider conversion, streaming, usage/cache, definition,
  duplicate, registration, and reachability matrices.
- Local protocol fixtures for malformed messages/SSE, split UTF-8, cancellation,
  Anthropic system/tool mapping, usage overflow, and cache layout panic.

## Out Of Scope

- Implementing fixes or changing public source/API shapes.
- Exhaustive OpenAI wire compatibility (`F-LLM-02`) and exhaustive Anthropic
  prompt-cache/thinking protocol coverage (`F-LLM-03`).
- ReAct state/event lifecycle after normalized chunks (`F-RCT-02/03`).
- Context-budget and compression semantics beyond their use of provider
  capabilities/cache layout (`F-CTX-01`, `F-CMP-01`).
- Live external-provider calls, prices, or remote model behavior.
- Security attack analysis; this is ordinary Rust API/protocol correctness.

## Inputs

- Root `AGENTS.md` and shared `README.md`, `REPORTING.md`, `TASKS.md`.
- Codex review rules in `codex/README.md`.
- Dependency report [F-CORE-01](F-CORE-01.md), limited to typed-error loss and
  public token arithmetic already established there.
- No other reviewer directory or report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Provider-neutral messages, tools, structured output, thinking, usage, cancellation, typed failures, stream chunks, capability facts, and cache-hint shape are reusable framework contracts. `echo_core` should own their types; integration clients should implement lossless or explicit capability adaptation. |
| EKO product policy | Model selection, persisted defaults, provider credentials, UI thinking choice, and entry-point reload behavior belong to EKO. |
| Adapter boundary | EKO may resolve a selected model and inject product defaults; it must preserve every selected field. Provider adapters may translate wire shape; they must either preserve facts or return typed unsupported/invalid-response errors. Neither adapter should own a second neutral protocol model. |
| Duplicate search | Searches covered type/trait names, fields, conversions, implementations, constructors, registrations, and live calls for `LlmClient`, all chat types, `ToolChoice`, both thinking protocol enums, `ProviderAdapter`/`AdapterClient`, capabilities, config models, cache hints, and EKO DTOs. EKO chat DTOs are valid application projections. `ThinkingProtocolPreference` and the unused generic provider adapter overlap live thinking/capability authority. |
| Migration deletion | Select one built-in provider adaptation authority. If concrete clients remain authoritative, delete the unimplemented `ProviderAdapter`/`AdapterClient` promise and duplicate thinking enum. If the generic adapter replaces them, factory construction and tests must move first, then duplicated concrete translation paths must be deleted. |

Public framework options were not declared dead merely because EKO does not
call them. The `ProviderAdapter` finding instead rests on internal semantic
disconnection: no built-in implementation, no factory registration, and its
own methods are ignored by `AdapterClient`.

## Current Path

```text
echo_core::llm::{ChatRequest, ChatResponse, ChatChunk, LlmClient}
  -> EKO resolve_runtime_model/build_llm_config
  -> ReactAgentBuilder::llm_config -> ReactAgent::set_llm_config
  -> LlmConfig::build_client
       OpenAiClient -> shared post/stream_post -> first choice -> ChatChunk
       AnthropicClient -> convert_request/custom SSE -> ChatChunk
  -> ReAct create_llm_stream
       PromptCacheLayout + fingerprint -> ChatRequest
       LlmClient::chat_stream -> flattened ChatChunk
       fabricate ChatCompletionChunk(id="", choice index=0)
  -> stream processor -> text/reasoning/tool buffers + last Usage
```

The canonical contract owns these facts:

| Contract area | Neutral representation | Current preservation |
|---|---|---|
| Message | role; text/image/file parts; name; tool ID/calls; reasoning | OpenAI mostly preserves after explicit file normalization; Anthropic overwrites earlier system messages and has no thinking response block |
| Tools | definitions/calls/deltas; `ToolChoice` enum also exported | definitions/calls work on happy paths; named choice cannot enter `ChatRequest` as typed JSON; Anthropic interleaved block index loses calls |
| Structured output | Text/JsonObject/JsonSchema | OpenAI maps it; Anthropic silently omits it; main ReAct stream always sends `None` |
| Thinking | typed `ThinkingConfig`, delta reasoning string | translation tests pass once config arrives; config/factory/EKO startup lose it on several paths; Anthropic output has no representation |
| Usage/cache | provider-family optional fields and derived helpers | happy-path tests pass; family inference, ratio, hash completeness, and arithmetic/range edges fail |
| Stream identity | provider OpenAI chunk has ID/choices/index; neutral `ChatChunk` has none | first choice only; ID/index/candidates discarded and later fabricated |
| Cancellation/errors | request token and `Result<ChatChunk>` | shared transport does not race cancellation against pending I/O; malformed SSE disappears instead of yielding error |

`AnthropicClient` does not override `LlmClient::capabilities`, so it inherits
OpenAI-compatible capabilities. This is live: both summary compressors branch
on `llm.capabilities().structured_output` before sending `ResponseFormat`.
Existing targeted tests all pass, but their fixtures stop before the failed
boundaries demonstrated by V08-V17.

## Findings

### F-LLM-01-P1-01: Streaming corrupts split UTF-8 and silently drops malformed events

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/client.rs:315`,
  `client.rs:317`, `client.rs:336`,
  `echo-agent/echo-integration/src/providers/anthropic.rs:496`,
  `anthropic.rs:510`
- Reachability: `OpenAiClient::chat_stream` and `AdapterClient::chat_stream`
  call shared `stream_post`; `AnthropicClient::chat_stream` repeats the same
  byte-to-string pattern. All feed the public `LlmClient` stream and live ReAct
  loop. Local fixtures split one valid Chinese scalar across HTTP chunks and
  send invalid SSE JSON.
- Expected invariant: HTTP byte chunk boundaries cannot change valid UTF-8, and
  malformed provider events yield a typed invalid-response error.
- Observed behavior: each received byte chunk is independently decoded with
  `String::from_utf8_lossy`, producing three replacement characters for one
  split scalar. `parse_sse_chunk`/Anthropic `if let Ok` discard malformed JSON,
  and EOF can be reported as successful stream completion.
- Impact: ordinary streamed model output can be corrupted; missing text,
  thinking, tool, usage, or terminal events are indistinguishable from a valid
  provider omission.
- Root cause: byte transport decoding and event framing are coupled to each
  received chunk, while parse failures use `Option`/ignored `Result` instead of
  an error-bearing incremental decoder.
- Direction: buffer bytes until complete SSE framing, decode complete UTF-8
  incrementally, and make malformed/truncated event parsing a typed stream
  error. Share one tested decoder between compatible and Anthropic transports.
- Regression validation: split every byte position of multilingual/emoji
  events; inject invalid UTF-8, malformed JSON, and truncated EOF; assert either
  exact deltas or one typed error, never silent loss.
- Validation reports: [V05](../validations/F-LLM-01/V05-01.md),
  [V11 attempt 01](../validations/F-LLM-01/V11-01.md),
  [V11 attempt 02](../validations/F-LLM-01/V11-02.md),
  [V12](../validations/F-LLM-01/V12-01.md),
  [V20](../validations/F-LLM-01/V20-01.md)

### F-LLM-01-P1-02: Cancellation does not interrupt pending stream I/O

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/client.rs:276`,
  `client.rs:298`, `echo-agent/echo-integration/src/providers/anthropic.rs:480`
- Reachability: every compatible provider stream enters `stream_post`; the
  public request token is copied from ReAct `AgentRunSnapshot`. Anthropic checks
  only after the next byte future resolves.
- Expected invariant: cancellation wins while awaiting connection/first byte/
  idle byte and no later content escapes after it.
- Observed behavior: cancelling 50 ms into a 500 ms wait returned only after
  502.57 ms and still yielded the delayed `late` chunk.
- Impact: stop/interrupt appears unresponsive and can deliver content after the
  caller has cancelled, violating recovery/UI lifecycle assumptions.
- Root cause: the transport checks the token between I/O awaits instead of
  selecting the cancellation future against send/next-byte futures.
- Direction: race cancellation against request send and every pending stream
  read, define cancellation as a typed terminal result, and delete duplicated
  provider polling once one transport owns it.
- Regression validation: cancel before send, during first-byte wait, during
  idle wait, between complete events, and after terminal; assert prompt end and
  zero post-cancel items.
- Validation reports: [V05](../validations/F-LLM-01/V05-01.md),
  [V13 attempt 01](../validations/F-LLM-01/V13-01.md),
  [V13 attempt 02](../validations/F-LLM-01/V13-02.md)

### F-LLM-01-P1-03: Anthropic silently loses request and response semantics

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/anthropic.rs:197`,
  `anthropic.rs:277`, `anthropic.rs:293`, `anthropic.rs:352`
- Reachability: `LlmConfig::anthropic` and `ProviderFactory` construct this
  concrete client; EKO uses that path for configured Anthropic models.
- Expected invariant: accepted neutral fields survive conversion or the client
  rejects unsupported controls explicitly; `raw` exposes actual response
  metadata as documented.
- Observed behavior: each system message overwrites the prior top-level system,
  so only the last survives. `tool_choice` and `response_format` are omitted
  without error. Thinking blocks cannot populate `reasoning_content`, and the
  purported raw response has empty ID/choices/model/extra.
- Impact: instructions can disappear, structured/tool constraints can be
  ignored, reasoning is unavailable, and callers inspecting `raw` receive
  fabricated absence rather than provider metadata.
- Root cause: an OpenAI-shaped neutral/raw model was adapted ad hoc without a
  completeness table or explicit unsupported-feature outcome.
- Direction: define lossless neutral metadata and capability-gated request
  behavior; combine multiple system instructions deterministically if the wire
  permits one field; reject unsupported controls or implement a documented
  fallback; preserve provider raw metadata without pretending it is empty
  OpenAI data.
- Regression validation: full field matrix with multiple system messages,
  tools/tool choice, each response format, thinking blocks, raw metadata, and
  explicit unsupported results.
- Validation reports: [V03](../validations/F-LLM-01/V03-01.md),
  [V04](../validations/F-LLM-01/V04-01.md),
  [V15](../validations/F-LLM-01/V15-01.md),
  [V21](../validations/F-LLM-01/V21-01.md)

### F-LLM-01-P1-04: Anthropic tool streaming keys state by the wrong index

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/anthropic.rs:520`,
  `anthropic.rs:526`, `anthropic.rs:532`, `anthropic.rs:554`
- Reachability: concrete Anthropic streaming tracks tool arguments in a map;
  live ReAct consumes emitted `DeltaToolCall`s to execute tools.
- Expected invariant: provider content-block index identifies the same tool
  across start, delta, and stop, including interleaved text/thinking blocks.
- Observed behavior: tool start ignores its `_index` and inserts at
  `tool_call_args.len()`. With text block 0 then tool block 1, deltas/stops look
  up 1 while state is at 0; the stream finishes as `tool_calls` but emits none.
- Impact: a valid tool request can disappear and leave the agent with an
  inconsistent finish reason, preventing intended tool execution.
- Root cause: tool ordinal was substituted for provider content-block identity.
- Direction: key state by the event's exact block index and model all block
  variants; emit a typed error for unmatched/malformed stop or arguments instead
  of `null`/silent removal.
- Regression validation: text-before-tool, thinking-before-tool, two tools,
  interleaved deltas, out-of-order/unmatched stops, and invalid argument JSON.
- Validation reports: [V03](../validations/F-LLM-01/V03-01.md),
  [V05](../validations/F-LLM-01/V05-01.md),
  [V14](../validations/F-LLM-01/V14-01.md),
  [V21](../validations/F-LLM-01/V21-01.md)

### F-LLM-01-P1-05: Anthropic advertises OpenAI capabilities it does not implement

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-core/src/llm/mod.rs:100`,
  `echo-agent/echo-core/src/llm/capabilities.rs:93`,
  `echo-agent/echo-state/src/compression/compressor/summary.rs:223`,
  `echo-agent/echo-state/src/compression/levels.rs:510`
- Reachability: `AnthropicClient` has no `capabilities` override, so the trait
  returns OpenAI-compatible `structured_output=true` and tool/stream facts.
  Both summary paths consult this live value before sending JSON mode.
- Expected invariant: capability queries describe the concrete client and are
  authoritative for generic consumers.
- Observed behavior: Anthropic reports OpenAI capabilities even though a
  dedicated `ProviderCapabilities::anthropic()` exists and conversion omits
  `response_format`.
- Impact: generic framework code selects unsupported paths and treats prompt-
  requested JSON as schema-constrained output; other capability decisions can
  be similarly wrong.
- Root cause: a permissive trait default hides missing provider overrides.
- Direction: require capabilities explicitly when constructing every concrete
  client or remove the permissive default; add factory conformance assertions.
- Regression validation: factory-create each provider and compare every
  capability field with request/response fixtures.
- Validation reports: [V04](../validations/F-LLM-01/V04-01.md),
  [V10](../validations/F-LLM-01/V10-01.md),
  [V15](../validations/F-LLM-01/V15-01.md)

### F-LLM-01-P1-06: Thinking configuration is documented but lost before requests

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/config.rs:178`,
  `config.rs:227`, `config.rs:320`, `config.rs:528`, `config.rs:844`,
  `echo-agent/src/agent/react/mod.rs:870`,
  `echo-agent-cli/echo-agent-app-core/src/model_config.rs:255`,
  `model_config.rs:315`,
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:306`, `infra.rs:440`
- Reachability: framework users can construct `LlmConfig { thinking }` and pass
  it through builder/setter; EKO startup resolves a configured model and passes
  `build_llm_config` to the same path. Later GUI/TUI model-change paths call
  `set_thinking`, proving intended runtime use.
- Expected invariant: the selected/configured thinking spec reaches every
  `ChatRequest` consistently at initial construction and later updates.
- Observed behavior: framework model YAML cannot express thinking; `from_model`
  and `to_model_config` set it to `None`; `set_llm_config` builds a client but
  never parses/applies it. EKO resolves the legacy mirror's thinking instead of
  the selected record, does not pass it to `build_llm_config`, and initial agent
  creation never calls `set_thinking`; later update paths compensate separately.
- Impact: users can configure reasoning depth and receive model defaults instead,
  with behavior changing after a hot model apply or between entry paths.
- Root cause: connection config, selected-model policy, and per-agent request
  state each hold part of the setting without one lossless adapter.
- Direction: keep typed thinking in the framework request/agent contract;
  preserve the spec through framework config conversion and have EKO select and
  parse it once at its application boundary for both startup and updates.
  Delete duplicated per-entry translation after one application service owns it.
- Regression validation: table-drive YAML/direct config/EKO selected model
  through initial and hot-applied agents, then assert captured request fields for
  each thinking protocol and invalid/auto/disabled cases.
- Validation reports: [V02](../validations/F-LLM-01/V02-01.md),
  [V07](../validations/F-LLM-01/V07-01.md),
  [V10](../validations/F-LLM-01/V10-01.md),
  [V22](../validations/F-LLM-01/V22-01.md),
  [V23](../validations/F-LLM-01/V23-01.md),
  [V26](../validations/F-LLM-01/V26-01.md)

### F-LLM-01-P1-07: Canonical streaming ignores the configured response format

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/builder.rs:916`,
  `echo-agent/src/agent/react/run/react_loop.rs:34`,
  `echo-agent/src/agent/react/run/phases/think.rs:312`,
  `think.rs:319`, `think.rs:375`
- Reachability: public builder methods store `ResponseFormat` in `AgentConfig`;
  non-stream `call_llm_with_retry` forwards it, while the canonical streaming
  core constructs both trait and legacy requests with `None`.
- Expected invariant: a public structured-output setting has the same meaning
  on the main Agent execution path independent of stream transport.
- Observed behavior: streaming never sends the configured format even when the
  selected provider reports support; only the non-stream loop forwards it.
- Impact: schema-guided extraction/structured Agent output can degrade to
  unconstrained text on the principal runtime path.
- Root cause: request assembly is duplicated between streaming/non-streaming
  paths and the streaming copy omitted the config field.
- Direction: construct one neutral request from Agent state, then select only
  the transport mode; delete duplicated field assembly.
- Regression validation: capture streaming and non-streaming requests from the
  same Agent configuration and assert field equality for all non-mode fields.
- Validation reports: [V02](../validations/F-LLM-01/V02-01.md),
  [V05](../validations/F-LLM-01/V05-01.md),
  [V24](../validations/F-LLM-01/V24-01.md)

### F-LLM-01-P1-08: Provider usage and cache layout can panic on representable inputs

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-integration/src/providers/anthropic.rs:339`,
  `anthropic.rs:595`,
  `echo-agent/echo-core/src/llm/cache/layout.rs:72`, `layout.rs:126`
- Reachability: Anthropic normalizes provider-supplied `u32` counters on every
  response; live ReAct constructs `PromptCacheLayout` before every request.
- Expected invariant: provider/input values representable by public types cannot
  panic; invalid combinations return errors or safe values.
- Observed behavior: `u32::MAX + 1` panics in Anthropic response normalization.
  A system message starting with the public runtime-context marker yields
  `sys_end=1`, `rt_start=0`, then panics slicing `[1..0]`.
- Impact: an LLM response or valid public message vector can unwind a request/
  agent task instead of returning a typed error.
- Root cause: provider arithmetic and independently derived slice boundaries
  assume unstated invariants without checked/saturating operations or validation.
- Direction: use checked/saturating normalization with explicit semantics;
  derive monotonic segment boundaries or return `Result`; never slice until
  bounds are validated.
- Regression validation: counter boundary matrix for streamed/non-streamed
  usage and role/marker permutations for cache layout under panic-deny Clippy.
- Validation reports: [V06](../validations/F-LLM-01/V06-01.md),
  [V16](../validations/F-LLM-01/V16-01.md),
  [V17 attempt 01](../validations/F-LLM-01/V17-01.md),
  [V17 attempt 02](../validations/F-LLM-01/V17-02.md),
  [F-CORE-01 V08](../validations/F-CORE-01/V08-01.md)

### F-LLM-01-P2-08: The neutral stream discards response and candidate identity

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/llm/mod.rs:232`,
  `echo-agent/echo-core/src/llm/types.rs:787`,
  `echo-agent/echo-integration/src/providers/openai.rs:366`,
  `echo-agent/src/agent/react/run/phases/think.rs:329`
- Reachability: OpenAI-compatible wire chunks can contain ID and multiple
  indexed choices; every concrete/generic compatible client chooses `.first()`
  into `ChatChunk`; ReAct then reconstructs ID empty/index zero while claiming
  no information is lost.
- Expected invariant: a neutral stream either preserves provider response/
  candidate identity or explicitly constrains the client contract to one
  candidate before the wire boundary.
- Observed behavior: response ID, all non-first candidates, and the first
  candidate's original index disappear. The downstream shape fabricates values.
- Impact: independent consumers cannot correlate chunks/candidates, and future
  multi-candidate/provider behavior is silently truncated rather than rejected.
- Root cause: `ChatChunk` was flattened for one ReAct consumer while the richer
  OpenAI chunk remained a second runtime shape.
- Direction: select one neutral stream shape. Preserve response ID and indexed
  deltas, or enforce/validate exactly one candidate at the adapter boundary and
  remove the misleading rich reconstruction/comment.
- Regression validation: stream two candidates with nonzero indices and stable
  response ID through provider client and ReAct adapter.
- Validation reports: [V03](../validations/F-LLM-01/V03-01.md),
  [V05](../validations/F-LLM-01/V05-01.md),
  [V25](../validations/F-LLM-01/V25-01.md)

### F-LLM-01-P2-09: Typed named ToolChoice cannot enter ChatRequest correctly

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/llm/mod.rs:109`, `mod.rs:135`,
  `mod.rs:148`, `echo-agent/echo-core/src/llm/types.rs:520`
- Reachability: `ToolChoice` is public and documented as preferred, but searches
  find no runtime call. `ChatRequest.tool_choice` and wire request both accept
  only `Option<String>`.
- Expected invariant: every public typed variant has an unambiguous request
  representation.
- Observed behavior: Auto/None/Required are strings, while Function produces a
  JSON object. The request channel can hold only a string, so callers must
  stringify the object into the wrong JSON wire type or bypass the typed API.
- Impact: the advertised specific-tool control is unusable/misleading and
  provider translation cannot be type-safe.
- Root cause: typed construction was added without replacing the older
  OpenAI-string request field.
- Direction: make the neutral request field typed and translate per provider;
  remove the raw string authority after caller migration.
- Regression validation: serialize every variant for compatible and Anthropic
  adapters and reject unsupported combinations explicitly.
- Validation reports: [V01](../validations/F-LLM-01/V01-01.md),
  [V03](../validations/F-LLM-01/V03-01.md),
  [V10](../validations/F-LLM-01/V10-01.md)

### F-LLM-01-P2-10: ProviderAdapter is a disconnected second provider authority

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-integration/src/providers/traits.rs:15`,
  `traits.rs:44`, `echo-agent/echo-integration/src/providers/adapter_client.rs:22`,
  `adapter_client.rs:76`, `adapter_client.rs:131`,
  `echo-agent/echo-integration/src/providers/config.rs:301`
- Reachability: repository-wide search finds no implementation or construction;
  factories instantiate concrete clients. Even an external implementation's
  thinking protocol/drop-temperature/base URL/env/cache hints are not consumed
  as documented: AdapterClient hardcodes OpenAI capabilities/provider-string
  translation and receives base URL separately.
- Expected invariant: a public generic adapter either owns built-in provider
  differences end to end or remains a narrow extension hook whose methods are
  all honored.
- Observed behavior: it duplicates core thinking protocol/capability/config
  concepts, is bypassed by built-ins, and ignores several declared methods.
- Impact: framework consumers face two incompatible provider-extension stories;
  implementing the documented trait does not produce the documented behavior.
- Root cause: a new abstraction was added without migrating a live factory path
  and deleting the old provider string/concrete-client authority.
- Direction: choose one authority using the migration deletion rule above; do
  not retain two thinking enums or adapter routes.
- Regression validation: construct every built-in through the selected factory,
  assert each declared hook changes a captured request, and repository-search
  for the deleted parallel route.
- Validation reports: [V01](../validations/F-LLM-01/V01-01.md),
  [V02](../validations/F-LLM-01/V02-01.md)

### F-LLM-01-P2-11: Usage/cache diagnostics can report impossible or colliding facts

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/llm/types.rs:686`, `types.rs:717`,
  `types.rs:752`,
  `echo-agent/echo-core/src/llm/cache/diagnostic.rs:104`,
  `diagnostic.rs:114`
- Reachability: provider response usage feeds live token tracking/events; ReAct
  calculates fingerprints before requests and EKO consumes cache diagnostics.
- Expected invariant: derived usage is provider-unambiguous and obeys documented
  ranges; fingerprints change whenever provider-visible stable prefix changes.
- Observed behavior: mixed provider fields use first-non-None cache value but
  Anthropic-field presence selects a different total formula; cached 20/prompt
  10 yields rate 2.0 despite `[0,1]` docs. Message hashes ignore image URL/detail,
  file name/content, tool calls/IDs, name and reasoning; tool hashes ignore type
  and description, so distinct prompts collide.
- Impact: token/cache telemetry and cache-stability diagnoses can be materially
  wrong, obscuring why provider caching changed and confusing budget decisions.
- Root cause: one untagged struct mixes incompatible provider usage families,
  and diagnostic hashing uses text-only approximations without documenting that
  limitation.
- Direction: normalize with explicit provider semantics/validation at the
  adapter boundary; clamp only if policy intentionally tolerates invalid data,
  otherwise return an anomaly. Hash canonical serialization of all
  provider-visible facts or rename/document the approximation.
- Regression validation: provider-family table including contradictory and
  boundary values; mutation test every message/tool field and require a hash
  change when wire content changes.
- Validation reports: [V06](../validations/F-LLM-01/V06-01.md),
  [V09 attempt 01](../validations/F-LLM-01/V09-01.md),
  [V09 attempt 02](../validations/F-LLM-01/V09-02.md),
  [V18](../validations/F-LLM-01/V18-01.md),
  [V19](../validations/F-LLM-01/V19-01.md),
  [V24](../validations/F-LLM-01/V24-01.md)

### F-LLM-01-P2-12: Malformed messages deserialize into valid-looking empty values

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/llm/types.rs:69`, `types.rs:85`,
  `types.rs:282`, `types.rs:286`
- Reachability: these are public provider/wire types used by response decoding
  and independent consumers. Existing tests deserialize normal strings/arrays.
- Expected invariant: invalid content shapes and missing mandatory roles are
  rejected with a parse error.
- Observed behavior: object/number/null content becomes `MessageContent::Empty`;
  a missing role defaults to empty string and becomes `Role::Custom("")`.
- Impact: malformed provider data is accepted and semantic absence cannot be
  distinguished from an intentionally empty message, delaying failure into
  downstream Agent logic.
- Root cause: deserialization defaults are used for compatibility without a
  validated wire boundary or explicit unknown variant.
- Direction: reject unsupported shapes/missing role at provider decoding; if a
  lenient internal type is required, make lossy conversion explicit and retain
  original/diagnostic facts.
- Regression validation: table-drive missing/wrong-type role/content/tool fields
  and require typed errors while retaining valid empty tool-call messages.
- Validation reports: [V03](../validations/F-LLM-01/V03-01.md),
  [V08](../validations/F-LLM-01/V08-01.md),
  [V18](../validations/F-LLM-01/V18-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate authority search | yes | failed invariant | [V01](../validations/F-LLM-01/V01-01.md) |
| V02 | Registration and live reachability trace | yes | failed invariant | [V02](../validations/F-LLM-01/V02-01.md) |
| V03 | Neutral field/variant matrix | yes | failed invariant | [V03](../validations/F-LLM-01/V03-01.md) |
| V04 | Concrete provider conversion matrix | yes | failed invariant | [V04](../validations/F-LLM-01/V04-01.md) |
| V05 | Streaming-neutrality trace | yes | failed invariant | [V05](../validations/F-LLM-01/V05-01.md) |
| V06 | Usage/cache authority and panic inspection | yes | failed invariant | [V06](../validations/F-LLM-01/V06-01.md) |
| V07 | EKO model/provider adapter trace | yes | failed invariant | [V07](../validations/F-LLM-01/V07-01.md) |
| V08 | Malformed neutral-message fixture | yes | passed probe; invariant false | [V08](../validations/F-LLM-01/V08-01.md) |
| V09 | Usage/cache executable fixture | yes | passed after bad expectation | [attempt 01](../validations/F-LLM-01/V09-01.md), [attempt 02](../validations/F-LLM-01/V09-02.md) |
| V10 | Capability/config/tool-choice fixture | yes | passed probe; invariants false | [V10](../validations/F-LLM-01/V10-01.md) |
| V11 | Split UTF-8 stream fixture | yes | invariant failed after environment retry | [attempt 01](../validations/F-LLM-01/V11-01.md), [attempt 02](../validations/F-LLM-01/V11-02.md) |
| V12 | Malformed SSE fixture | yes | failed invariant | [V12](../validations/F-LLM-01/V12-01.md) |
| V13 | Cancellation-during-I/O fixture | yes | failed after diagnostic correction | [attempt 01](../validations/F-LLM-01/V13-01.md), [attempt 02](../validations/F-LLM-01/V13-02.md) |
| V14 | Anthropic interleaved tool block fixture | yes | failed invariant | [V14](../validations/F-LLM-01/V14-01.md) |
| V15 | Anthropic request capture fixture | yes | failed invariant | [V15](../validations/F-LLM-01/V15-01.md) |
| V16 | Anthropic usage overflow fixture | yes | failed invariant | [V16](../validations/F-LLM-01/V16-01.md) |
| V17 | Cache layout panic fixture | yes | failed after fixture correction | [attempt 01](../validations/F-LLM-01/V17-01.md), [attempt 02](../validations/F-LLM-01/V17-02.md) |
| V18 | `echo_core` LLM type tests | yes | passed | [V18](../validations/F-LLM-01/V18-01.md) |
| V19 | `echo_core` cache tests | yes | passed | [V19](../validations/F-LLM-01/V19-01.md) |
| V20 | shared provider client tests | yes | passed | [V20](../validations/F-LLM-01/V20-01.md) |
| V21 | Anthropic adapter tests | yes | passed | [V21](../validations/F-LLM-01/V21-01.md) |
| V22 | thinking translation tests | yes | passed | [V22](../validations/F-LLM-01/V22-01.md) |
| V23 | provider config tests | yes | passed | [V23](../validations/F-LLM-01/V23-01.md) |
| V24 | ReAct think/cache tests | yes | passed | [V24](../validations/F-LLM-01/V24-01.md) |
| V25 | ReAct stream processor tests | yes | passed | [V25](../validations/F-LLM-01/V25-01.md) |
| V26 | EKO model-config tests | yes | passed | [V26](../validations/F-LLM-01/V26-01.md) |
| V27 | source cleanliness/session/build-lock gate | yes | passed | [V27](../validations/F-LLM-01/V27-01.md) |
| V28 | private probe target cleanup | yes | passed | [V28](../validations/F-LLM-01/V28-01.md) |
| V29 | final report-link/executor/isolation/owned-session gate | yes | passed after two external-lock observations | [attempt 01](../validations/F-LLM-01/V29-01.md), [attempt 02](../validations/F-LLM-01/V29-02.md), [attempt 03](../validations/F-LLM-01/V29-03.md) |
| V30 | Primary stream/UTF-8/cancel/identity reconstruction | yes | failed invariant | [V30-01](../validations/F-LLM-01/V30-01.md) |
| V30 | Primary Anthropic semantic/capability reconstruction | yes | failed invariant | [V30-02](../validations/F-LLM-01/V30-02.md) |
| V30 | Primary thinking/structured-output trace | yes | failed invariant | [V30-03](../validations/F-LLM-01/V30-03.md) |
| V30 | Primary usage/cache/message boundary reconstruction | yes | failed invariant | [V30-04](../validations/F-LLM-01/V30-04.md) |
| V30 | Primary ToolChoice/provider-authority reconstruction | yes | failed invariant | [V30-05](../validations/F-LLM-01/V30-05.md) |

There are 30 validation IDs and 40 immutable attempt reports. No historical-
document drift validation was applicable: no historical design document was
used as evidence beyond the current root rules and dependency report.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AGENTS.md`: strings must be UTF-8 safe | regressed in provider byte transport | [V11-02](../validations/F-LLM-01/V11-02.md), P1-01 |
| `AGENTS.md`: public/input paths must not panic | regressed in provider usage and cache layout | [V16](../validations/F-LLM-01/V16-01.md), [V17-02](../validations/F-LLM-01/V17-02.md), P1-08 |
| F-CORE-01-P2-06: public provider-sized token arithmetic can panic | current and narrowed to a live adapter | [V16](../validations/F-LLM-01/V16-01.md), P1-08 |

## Coverage And Uncertainty

All declared neutral fields and variants were statically mapped, both concrete
providers and the generic adapter were traced, and the real root/EKO paths were
checked. Local fixtures used deterministic protocol stubs, not remote services.
Existing targeted suites were executed separately and passed.

OpenAI provider request details should be independently exhausted in F-LLM-02;
Anthropic cache-control, thinking block wire versions, malformed partial tool
JSON, and all interleavings remain for F-LLM-03. This task did not prove whether
all provider usage chunks are cumulative or delta because the trait has no
contract for that fact. It also did not execute an entire Agent turn with a
remote LLM. Disk availability fell below the repository threshold, so after all
required commands completed the task removed only its 998.5 MiB temporary
target and ran no further builds.

Primary independently reconstructed every finding through current source and
live caller paths in V30-01 through V30-05. Delegated fixtures retain the
timing, byte-split, protocol, and panic effects that source inspection alone
cannot execute under current disk pressure. V29-03 establishes that this task
owns no open Cargo/exec session; unrelated shared-workspace locks remain
documented in V29-01/V29-02 and are not a task blocker.

## Handoff

- `F-LLM-02` should read V03/V05/V11-V13 and establish one incremental SSE
  decoder plus complete compatible-provider field fixtures.
- `F-LLM-03` should read V04/V14-V16 and exhaust Anthropic system, thinking,
  cache, tool-block, usage, and malformed-event mappings.
- `F-RCT-03` should treat `ChatChunk` identity/candidate loss and cancellation
  terminal behavior as upstream constraints; it should not compensate by
  fabricating provider facts.
- `A-CFG-01` should verify startup/hot-apply parity for selected-model thinking
  after one application adapter owns it.
- `F-CTX-01`/`F-CMP-01` should not trust provider capability or cache-layout
  claims until P1-05/P1-08/P2-11 are corrected.
- This report becomes stale when core LLM types/capabilities/cache code,
  provider traits/clients/config/factory, ReAct request assembly, EKO model
  resolution/agent creation, or either reviewed commit changes.
