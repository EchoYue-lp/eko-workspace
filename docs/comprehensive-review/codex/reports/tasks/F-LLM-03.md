# F-LLM-03: Anthropic provider and prompt-cache adapter

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d3`
> `echo-agent-cli` commit: `b3b2e81f2b2d`
> Worktree state: framework clean; CLI had unrelated generated-frontend changes; review reports/fixtures are outside source repositories

## Question

Does the Anthropic adapter preserve the provider-neutral request/response/
stream contract, including thinking blocks and prompt-cache behavior?

## Scope

- Concrete Anthropic request conversion, non-stream response conversion, SSE
  event conversion, tool blocks, terminal reasons, usage, errors, and
  cancellation in `echo-integration/src/providers/anthropic.rs`.
- Cache layout and breakpoint planning in `echo-core::llm::cache` and
  `echo-integration/src/providers/anthropic_cache.rs`.
- Anthropic thinking selection/translation in `ModelProfile`,
  `ThinkingConfig`, and the request builder.
- Public facade/config construction and EKO's real provider/thinking path.
- Static definition/duplicate/reachability/variant/test matrices, focused
  existing tests, and private localhost protocol fixtures.

## Out Of Scope

- Source fixes, public API changes, live provider calls, and security attack or
  vulnerability analysis.
- Provider-neutral defects already accepted in F-LLM-01: multiple-system
  overwrite; omitted tool choice/response format/raw facts; wrong capability
  default; wrong interleaved tool index; split UTF-8; malformed generic SSE;
  pending stream cancellation; usage overflow; generic adapter duplication.
- OpenAI-specific behavior owned by F-LLM-02.
- ReAct lifecycle after normalized chunks and EKO rendering.
- A claim about whether `prompt-caching-2024-07-31` is currently required or
  obsolete. Official Anthropic pages were inaccessible in V04-V10.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md);
  Codex track rules.
- Accepted dependency [F-LLM-01](F-LLM-01.md), used as the provider-neutral
  contract and deduplication boundary.
- [F-LLM-02](F-LLM-02.md), used only to avoid renumbering OpenAI-specific
  equivalents.
- No other reviewer directory or report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Neutral thinking/reasoning blocks, tool calls, usage/cache facts, terminal reasons, errors, and cancellation belong to reusable framework contracts. |
| EKO product policy | Selected Anthropic model, credential/base URL, UI thinking level, and user-visible cancellation belong to EKO. |
| Adapter boundary | Anthropic conversion owns wire blocks/headers/version differences. It must preserve accepted neutral facts or return typed unsupported/invalid-response results; it must not silently rewrite input or synthesize success. |
| Duplicate search | Searched names, wire structs, fields, traits, constructors, exports, config registrations, cache planners, tests, and live calls across both repositories. One concrete Anthropic wire authority and one cache-plan authority were found. F-LLM-01's disconnected generic adapter remains separate existing debt. |
| Migration deletion | Keep one concrete Anthropic adapter and one shared model resolver. Replace its narrow response/event structs with one field-complete Anthropic wire model, then delete obsolete duplicated stream/non-stream maps and tests that bless lossy behavior. |

## Current Path

```text
EKO provider/model config
  -> build_llm_config("anthropic") -> LlmConfig::anthropic
  -> LlmConfig::build_client -> AnthropicClient
  -> ChatRequest
       convert_request
         messages/files/tools
         CacheHints or PromptCacheLayout -> AnthropicCachePlan -> cache_control
         ModelProfile(model) -> build_anthropic_thinking
       chat
         headers + HTTP -> AnthropicResponse -> ChatResponse
       chat_stream
         different headers + HTTP bytes -> AnthropicStreamEvent -> ChatChunk
  -> ReAct/EKO consumers
```

Positive evidence is material: the automatic cache fixture emits exactly four
breakpoints at system, last tool, deep history, and last stable history while
excluding runtime context; normal stream usage retains input/output totals and
both cache token fields. All 12 existing Anthropic/cache unit tests pass.

## Findings

### F-LLM-03-P1-01: Repository-standard Claude 4.6 model IDs select the legacy budget protocol

- Priority: P1
- Confidence: high
- Layer: framework/adapter
- Evidence: `echo-agent/echo-core/src/llm/capabilities.rs:420`,
  `capabilities.rs:425`, `capabilities.rs:439`,
  `echo-agent/echo-integration/src/providers/anthropic.rs:270`,
  `anthropic.rs:744`, `anthropic.rs:756`,
  `echo-agent/echo-integration/src/providers/config.rs:155`,
  `echo-agent-cli/src/main.rs:12`
- Reachability: public config examples and EKO CLI use
  `claude-sonnet-4-6`; EKO's thinking-support query and the live request
  conversion both call `ModelProfile::new`.
- Expected invariant: a model the repository identifies as Claude 4.6 selects
  its declared `AnthropicEffort` translation: adaptive block plus effort.
- Observed behavior: the resolver scans dash segments, sees bare `4`, and
  exits as legacy budget before reading `6`. V12 captured high thinking as
  `enabled + budget_tokens:3277`, with no effort. Tests cover only dotted
  forms such as `claude-4.6-sonnet`.
- Impact: EKO reports the wrong support protocol and sends a different
  thinking request from the selected user level; the adapter's own comments
  state that budget tokens are rejected on 4.6.
- Root cause: a free-form version scanner does not recognize Anthropic's
  repository-standard split major/minor spelling, and tests do not use the
  public examples' model names.
- Direction: parse known model-ID grammar once in the model-profile authority;
  cover split and dotted aliases plus dated suffixes. Do not add a second
  Anthropic-only resolver in the adapter.
- Regression validation: exact public/EKO model IDs for 3.7, 4, 4.5, 4.6, and
  future/adaptive classes; assert profile, UI support response, and captured
  request wire.
- Validation reports: [V01](../validations/F-LLM-03/V01-01.md),
  [V02](../validations/F-LLM-03/V02-01.md),
  [V03](../validations/F-LLM-03/V03-01.md),
  [V12 attempt 02](../validations/F-LLM-03/V12-02.md),
  [V20](../validations/F-LLM-03/V20-01.md)

### F-LLM-03-P1-02: Thinking-enabled Anthropic responses cannot round-trip through either response path

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/anthropic.rs:293`,
  `anthropic.rs:410`, `anthropic.rs:825`, `anthropic.rs:1029`,
  `anthropic.rs:1048`, `anthropic.rs:1080`, `anthropic.rs:1088`
- Reachability: the same client that emits thinking request fields decodes all
  non-stream and stream responses using these enums.
- Expected invariant: enabling a supported thinking mode cannot make its valid
  response undecodable; reasoning, text, signatures/redacted blocks, tools,
  and usage must be retained or explicitly represented as unsupported.
- Observed behavior: `ContentBlock` lacks thinking/redacted-thinking, so V13's
  valid thinking+text non-stream body fails the entire response parse as a
  network error. Stream start/delta types lack thinking/signature fields; V14
  retains text and usage but drops all reasoning/signature facts.
- Impact: a request-side feature advertised by framework/EKO can fail the
  non-stream call outright and silently erase reasoning in streaming.
- Root cause: request-side thinking support was added without versioned,
  field-complete response block models or conformance fixtures.
- Direction: model all accepted Anthropic content block/event versions, retain
  reasoning and opaque signatures/redacted facts in an honest neutral/raw
  channel, and use typed invalid-response errors for unknown required blocks.
- Regression validation: enabled/adaptive thinking, signature and redacted
  variants, text-thinking-tool interleaving, unknown future block, and semantic
  equality between stream and non-stream.
- Validation reports: [V03](../validations/F-LLM-03/V03-01.md),
  [V13](../validations/F-LLM-03/V13-01.md),
  [V14](../validations/F-LLM-03/V14-01.md)
- Deduplication: this refines accepted F-LLM-01-P1-03 from “reasoning cannot be
  populated” to the executable non-stream whole-response failure. Synthesis
  should merge the IDs if one canonical Anthropic semantic-loss item is kept.

### F-LLM-03-P1-03: A well-formed Anthropic error stream ends as silent success

- Priority: P1
- Confidence: medium
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/anthropic.rs:478`,
  `anthropic.rs:502`, `anthropic.rs:510`,
  `anthropic.rs:1048`, `anthropic.rs:1069`
- Reachability: every concrete Anthropic stream passes through the local event
  enum and catch-all.
- Expected invariant: a named provider error event emits one typed terminal
  error with available type/message and cannot be confused with clean EOF.
- Observed behavior: the event enum has no error payload; V15 supplied a
  well-formed named `error/overloaded_error` envelope with Anthropic's error
  shape and received zero items. Current official documentation could not be
  retrieved, so external-version applicability remains the reason for medium
  rather than high confidence.
- Impact: callers can treat provider failure as successful empty completion and
  lose all provider diagnostics.
- Root cause: `#[serde(other)] Other` erases event identity/payload and the
  stream has no protocol-terminal validation.
- Direction: add explicit provider error/ping events, normalize error facts,
  require a valid terminal event, and share the error-bearing SSE decoder
  direction already required by F-LLM-01.
- Regression validation: every provider error shape before/during content,
  ping interleaving, error after usage, and EOF without terminal.
- Validation reports: [V03](../validations/F-LLM-03/V03-01.md),
  [V15](../validations/F-LLM-03/V15-01.md)

### F-LLM-03-P1-04: Anthropic non-stream cancellation is accepted but ignored

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-core/src/llm/mod.rs:148`,
  `echo-agent/echo-integration/src/providers/anthropic.rs:368`,
  `anthropic.rs:375`, `anthropic.rs:410`
- Reachability: public `LlmClient::chat` accepts the same `ChatRequest`
  control as streaming; EKO/framework consumers can call it through the live
  configured client.
- Expected invariant: cancellation interrupts pending send/header/body/retry
  work, or the non-stream method does not accept that control.
- Observed behavior: `chat` never reads the token. V17 cancelled after 30 ms
  during a 350 ms response; it returned successful late content after 371 ms.
- Impact: stop/recovery actions cannot stop non-stream work and can observe a
  result after cancellation.
- Root cause: cancellation was added only as between-chunk polling in the
  separate stream implementation.
- Direction: race a shared typed cancellation future against send/body/retry
  waits for both methods; do not add another polling loop.
- Regression validation: cancel before send, awaiting headers/body/backoff,
  after completion, and semantic parity with stream cancellation.
- Validation reports: [V03](../validations/F-LLM-03/V03-01.md),
  [V17](../validations/F-LLM-03/V17-01.md)
- Deduplication: analogous OpenAI behavior is F-LLM-02-P1-04 and pending
  Anthropic stream cancellation is F-LLM-01-P1-02; synthesis should converge
  them on one shared transport cancellation design.

### F-LLM-03-P1-05: Malformed assistant tool arguments are rewritten as JSON null

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/anthropic.rs:91`,
  `anthropic.rs:99`, `anthropic.rs:103`
- Reachability: the public neutral `Message.tool_calls` field accepts an
  argument string; every assistant tool-call history message takes this branch.
- Expected invariant: malformed JSON arguments produce a typed conversion error
  before sending; the adapter cannot silently change tool semantics.
- Observed behavior: `serde_json::from_str(...).unwrap_or_default()` converts
  invalid input to `Value::Null`. V18 captured `"input":null`, and the
  public call returned success after the fixture accepted it.
- Impact: replayed/external tool history can be silently mutated and then
  rejected remotely or interpreted as a different call; the original bad input
  and local cause are lost.
- Root cause: request conversion is infallible and substitutes a serde default
  where a typed validation result is required.
- Direction: make conversion fallible and preserve exact JSON values; delete
  null/default fallbacks for tool arguments and pair tool-use/results
  explicitly.
- Regression validation: invalid/truncated/scalar/array/object arguments,
  Unicode JSON, multiple calls, and tool-result pairing.
- Validation reports: [V03](../validations/F-LLM-03/V03-01.md),
  [V18](../validations/F-LLM-03/V18-01.md)

### F-LLM-03-P2-06: Stream and non-stream terminal reasons use different neutral vocabularies

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-integration/src/providers/anthropic.rs:313`,
  `anthropic.rs:589`
- Reachability: both public trait methods execute separate match tables.
- Expected invariant: equivalent provider terminal causes normalize to the same
  provider-neutral finish reason.
- Observed behavior: non-stream maps `max_tokens -> length`; stream returns
  literal `max_tokens`. V14 observed the literal alongside otherwise correct
  usage.
- Impact: generic callers must branch on invocation mode to identify token
  exhaustion, undermining stream/non-stream conformance.
- Root cause: duplicated mapping tables drifted.
- Direction: one total terminal-reason normalization function shared by both
  methods; define behavior for unknown/new reasons.
- Regression validation: every known reason through both methods, unknown
  reason, null reason, and terminal/usage ordering.
- Validation reports: [V03](../validations/F-LLM-03/V03-01.md),
  [V14](../validations/F-LLM-03/V14-01.md)

## Cache And Usage Conclusions

| Area | Result |
|---|---|
| Automatic breakpoint planning | Passed: maximum four; system, last tool, ~75% history, last stable; runtime context excluded (V16). |
| Empty explicit hints fallback | Existing targeted test passes; provider recomputes layout (V19). |
| Normal non-stream usage | Prompt/completion and cache creation/read fields retained; accepted overflow defect remains F-LLM-01. |
| Normal stream usage | V14 retained 7 input, 3 output, total 10, cache creation 2, cache read 5. |
| Header parity | Non-stream sends `anthropic-beta: prompt-caching-2024-07-31`; stream does not. Current official requirement could not be retrieved, so impact is residual uncertainty, not a finding. |
| Hint diagnostics | `stable_prefix_hash` and `segments` guide observability/derivation upstream but are not serialized to Anthropic, as expected; no adapter finding. |

## Validation Matrix

| ID | Validation | Status | Evidence |
|---|---|---|---|
| V01 | Definition/duplicate inventory | passed | [attempt 01](../validations/F-LLM-03/V01-01.md) |
| V02 | Registration/runtime reachability | passed | [attempt 01](../validations/F-LLM-03/V02-01.md) |
| V03 | Field/variant/test invariant matrix | failed | [attempt 01](../validations/F-LLM-03/V03-01.md) |
| V04 | Official-doc search | inconclusive | [attempt 01](../validations/F-LLM-03/V04-01.md) |
| V05 | Official platform thinking page, sandbox | inconclusive | [attempt 01](../validations/F-LLM-03/V05-01.md) |
| V06 | Official platform thinking page, approved network | inconclusive | [attempt 01](../validations/F-LLM-03/V06-01.md) |
| V07 | Official legacy thinking page | inconclusive | [attempt 01](../validations/F-LLM-03/V07-01.md) |
| V08 | Official streaming page | inconclusive | [attempt 01](../validations/F-LLM-03/V08-01.md) |
| V09 | Official prompt-cache page | inconclusive | [attempt 01](../validations/F-LLM-03/V09-01.md) |
| V10 | Official model overview | inconclusive | [attempt 01](../validations/F-LLM-03/V10-01.md) |
| V11 | Private fixture compile | passed after fixture corrections | [01](../validations/F-LLM-03/V11-01.md), [02](../validations/F-LLM-03/V11-02.md), [03](../validations/F-LLM-03/V11-03.md) |
| V12 | 4.6 model-ID request capture | failed after sandbox retry | [01](../validations/F-LLM-03/V12-01.md), [02](../validations/F-LLM-03/V12-02.md) |
| V13 | Non-stream thinking response | failed | [attempt 01](../validations/F-LLM-03/V13-01.md) |
| V14 | Stream thinking/terminal/cache usage | failed, with positive usage result | [attempt 01](../validations/F-LLM-03/V14-01.md) |
| V15 | Stream provider error event | failed | [attempt 01](../validations/F-LLM-03/V15-01.md) |
| V16 | Cache request/breakpoint capture | passed | [attempt 01](../validations/F-LLM-03/V16-01.md) |
| V17 | Non-stream cancellation | failed | [attempt 01](../validations/F-LLM-03/V17-01.md) |
| V18 | Malformed tool input | failed | [attempt 01](../validations/F-LLM-03/V18-01.md) |
| V19 | Existing focused tests | passed, 12/12 | [attempt 01](../validations/F-LLM-03/V19-01.md) |
| V20 | Historical drift | passed | [attempt 01](../validations/F-LLM-03/V20-01.md) |
| V21 | Mechanical links/executor/status/dirty/cleanup gate | passed after inconclusive first run | [01](../validations/F-LLM-03/V21-01.md), [02](../validations/F-LLM-03/V21-02.md) |
| V30 | Primary static source reconstruction and acceptance gate | mixed, final passed | [01](../validations/F-LLM-03/V30-01.md), [02](../validations/F-LLM-03/V30-02.md), [03](../validations/F-LLM-03/V30-03.md), [04](../validations/F-LLM-03/V30-04.md), [05](../validations/F-LLM-03/V30-05.md), [06](../validations/F-LLM-03/V30-06.md) |

No workspace build or shared target was used. The one repository Cargo command
was package/test-filtered and used a private target because available disk was
below the repository's 50 GiB threshold.

## Historical Conclusions

| Historical claim | Classification | Current evidence |
|---|---|---|
| Anthropic main-path cache hints with empty breakpoints must derive a plan | current | V16 and the existing focused test in V19 |
| Cache token extraction is available | current with inherited arithmetic caveat | V14/V16; overflow remains F-LLM-01 |
| Provider-aware thinking selects correct Claude protocol | regressed for repository-standard split IDs | V12; dotted-only tests miss public spelling |
| Native Anthropic support generally exists | current but narrower than advertised controls | V02 plus all findings |

## Coverage Gaps And Residual Uncertainty

- Official Anthropic pages were inaccessible; current model/header/version
  policy was not asserted. Internal contradictions and local wire behavior are
  independently reproducible.
- No live provider/API-key call was made.
- Prompt-cache hit behavior and provider billing cannot be proven locally;
  request markers and returned usage fields were validated.
- Existing F-LLM-01 interleaved-tool-index and SSE framing fixtures were not
  duplicated. V14 adds thinking/text interleaving but intentionally does not
  renumber that accepted tool-index issue.
- Redacted-thinking and future unknown-block fixtures are required for a fix,
  but the current missing-enum root cause is already established by V13/V14.

## Handoff

- Primary review independently reconstructed all seven findings from source in
  V30-01..05. Per the read-only review policy, no further builds/tests are
  required here; dynamic cases become fix-stage regressions. P1-02 should merge
  into F-LLM-01-P1-03 during synthesis.
- Iteration order: repair shared model-ID parsing first; define field-complete
  Anthropic response/event/error models; unify stream/non-stream terminal and
  cancellation behavior; make request conversion fallible; retain V16's cache
  strategy.
- Stale triggers: any change to Anthropic request/response/event structs,
  `ModelProfile` version resolution, `CacheHints`/breakpoint mapping,
  cancellation transport, or provider API version/header policy.
