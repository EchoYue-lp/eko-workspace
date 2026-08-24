# F-CTX-01: Context selection and budget accounting

> Status: complete
> Reviewer: Codex primary reviewer
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; review artifacts are outside the source repositories

## Question

Are canonical instructions, history, tools, attachments, memory, and reserved
output selected deterministically within model limits?

## Scope

- `echo-agent/src/context`, `echo-core/src/{budget,tokenizer}.rs`, model-profile
  context-window facts, and `echo-state/src/compression` accounting boundaries.
- Default ReAct construction, per-iteration compaction, request schema assembly,
  multimodal entry points, runtime model-window mutation, and transcript projection
  identity.
- Bounded EKO runtime model-switch callers solely to prove public framework
  mutation reachability.
- Existing source tests and current git history as coverage/drift evidence. No
  executable validation was run.

## Out Of Scope

- Invalid allocation percentages and extreme-count arithmetic already owned by
  `F-REL-01-P2-04`.
- Prompt composition/cardinality and allowlist defects owned by `F-RCT-01`.
- Compressor-specific semantic preservation, repeated-summary quality, and
  recovery facts (`F-CMP-01`).
- Tool-result artifact and schema-validation contracts (`F-EXT-01`).
- Provider-native tokenization formula accuracy and current vendor window facts;
  no external time-sensitive claim is required for these internal wiring defects.
- Source fixes, commits, index changes, other-reviewer conclusions, Cargo/rustc,
  tests, builds, Clippy, or dynamic fixtures.

## Inputs

- Root `AGENTS.md`; shared review `README.md`, `REPORTING.md`, `TASKS.md`; Codex
  review rules and report templates.
- Completed dependency reports [F-RCT-01](F-RCT-01.md) and
  [F-LLM-01](F-LLM-01.md). The former supplied the canonical prompt/one
  ToolManager call graph; the latter supplied provider-visible message/tool facts.
- [F-REL-01](F-REL-01.md) only for de-duplicating its accepted TokenBudget
  validation/overflow finding.
- Current source, tests, and bounded history for the reviewed paths. No other AI
  reviewer directory or conclusion was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Complete request accounting, tokenizer abstraction, context-window application, deterministic selection, compression admission/postconditions, and typed internal context identity are generic framework responsibilities. |
| EKO product policy | EKO selects its model, configured reserve/output policy, attachment admission UX, and when a user chooses to switch models. It should not recalculate a second model budget. |
| Adapter boundary | EKO resolves product config into one typed framework model profile and invokes one fallible runtime reconfiguration API. The adapter must not mutate a config field that is disconnected from the live ContextManager. |
| Duplicate search | Searched names, fields, setters, builders, token counters, model-window resolution, schema metrics, multimodal contents, protected/projection markers, selection helpers, and all production/test consumers across both repositories. |
| Migration deletion | Converge on one request-budget snapshot and one model-profile/window authority. Delete the separate ContextAssembler arithmetic or make it delegate to that engine; delete the config-file-only resolver and inert runtime setter after callers migrate. Retain ContextSelector as a valid framework option, but make ordering/config validation deterministic. |

Nothing here should move generic budget or compression behavior into EKO. The
application call sites demonstrate impact, not ownership.

## Current Path

```text
construction
  ReactAgentBuilder / AgentConfig / AppConfig
    -> token_limit (independent default/config field)
    -> TokenBudgetConfig.build(token_limit fallback)
    -> optional ModelProfile.context_window (not read by context construction)
    -> ContextManager {
         messages,
         calibrated text tokenizer,
         token_limit,
         TokenBudget,
         SlidingWindow(40),
       }

each ReAct iteration
  run_compact
    -> apply typed-looking but content-prefixed projections
    -> ContextManager.prepare
         estimate message text only
         TokenBudget.allocate(system=0, tools=0, conversation=all text)
         split protected content
         SlidingWindow keep 40 messages (ignores token_limit)
         merge protected + reinject canonical
         return without final token-bound assertion
    -> tools_for_request
         measure schema tokens for log/metrics only
    -> ChatRequest(messages, tools, configured max_tokens)

multimodal
  MessageContent::Parts -> as_text keeps text, ignores Image/File cost
  streaming path -> ContextManager.prepare with text-only estimate
  chat_multimodal -> raw ContextManager messages -> request, no prepare

runtime model switch
  EKO GUI/TUI/AgentPool -> ReactAgent::set_token_limit
    -> AgentConfig.token_limit only
    -X-> live ContextManager/token budget/compressor
```

Positive conclusions:

- The streaming core shares one calibrated tokenizer Arc between ContextManager
  and usage feedback; when a provider reports prompt usage, normal text estimates
  can converge over subsequent requests.
- Projection scope replacement, tombstones, and canonical exact-prompt
  reinjection have focused source tests. Tool schemas are deterministically
  sorted before their diagnostic size is measured.
- UTF-8-safe tool-output truncation is handled elsewhere. This task found no new
  byte-slicing panic and does not duplicate that positive boundary.

## Findings

### F-CTX-01-P1-01: ReAct has no complete pre-request model-window admission check

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/budget.rs:88`,
  `echo-agent/echo-state/src/compression/mod.rs:1295`,
  `echo-agent/echo-state/src/compression/mod.rs:1309`,
  `echo-agent/src/agent/react/run/phases/think.rs:270`,
  `echo-agent/src/agent/react/run/phases/think.rs:395`
- Reachability: every normal streaming ReAct iteration prepares ContextManager
  messages, then independently selects tool definitions and builds ChatRequest.
- Expected invariant: one admission snapshot charges all provider-visible input
  (system/canonical/history/projections/memory/attachments/tool schemas), reserves
  bounded output and safety, then compresses/reselects or returns a typed cannot-fit
  error before network I/O.
- Observed behavior: ContextManager counts only message text and calls
  `TokenBudget::allocate(0, 0, estimated_tokens)`, treating system text as
  conversation and omitting tool schemas. The later schema calculation records
  metrics only. Neither configured `max_tokens` nor profile max output is
  reconciled with the reserved output category or final request size.
- Impact: a request can pass context preparation and still exceed the provider's
  window when tools/output are added, causing avoidable provider rejection or
  uncontrolled loss of output headroom on a core Agent path.
- Root cause: accounting is split between ContextManager, ToolManager diagnostics,
  provider request fields, and model-profile metadata without one request owner.
- Direction: introduce a framework-owned `RequestBudgetSnapshot` (or equivalent)
  that serializes/counts the exact selected messages and schemas with explicit
  output/safety reserve. Feed its conversation allowance into compression, then
  remeasure and fail typed if protected/request content cannot fit. Delete the
  metrics-only parallel budget semantics once the request owner emits those
  metrics.
- Regression validation: exact-boundary and one-over requests combining system,
  history, memory projection, giant schemas, and max output; assert no HTTP call
  occurs when the total cannot fit.
- Validation reports: [V02-01](../validations/F-CTX-01/V02-01.md),
  [V03-06](../validations/F-CTX-01/V03-06.md),
  [V05-01](../validations/F-CTX-01/V05-01.md)

### F-CTX-01-P1-02: Resolved model-profile context windows are disconnected from context construction

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/llm/capabilities.rs:182`,
  `echo-agent/echo-core/src/llm/capabilities.rs:271`,
  `echo-agent/src/agent/config.rs:52`,
  `echo-agent/src/agent/react/mod.rs:336`,
  `echo-agent/src/config.rs:97`
- Reachability: consumers can resolve/install ModelProfile through public builder
  APIs; default ReAct construction always creates ContextManager from the separate
  `AgentConfig.token_limit` and `TokenBudgetConfig` fields.
- Expected invariant: one resolved model profile is the default source of context
  and output limits, with explicit consumer overrides taking documented precedence.
- Observed behavior: repository-wide production consumers of
  `ModelProfile.context_window` end at construction/override storage. The config
  module separately infers a window, but when no explicit token_limit/context_window
  is configured it writes `token_limit=usize::MAX` while its enabled default
  TokenBudget falls back to 396K. Generic builder defaults also remain 396K even
  for an installed profile with another window.
- Impact: advertised profile resolution cannot protect independent framework
  consumers, and two construction paths can apply contradictory compression and
  budget thresholds for the same model.
- Root cause: model capabilities, config-file inference, AgentConfig, and
  TokenBudget were added as parallel authorities rather than normalized once.
- Direction: resolve one immutable model/request limits object during build using
  explicit override > profile > conservative unknown-model policy. Delete the
  config module's duplicate name resolver and raw fallback semantics after all
  constructors consume the same object.
- Regression validation: table generic builder and config-file construction for
  known, overridden, and unknown profiles; ContextManager/request budget must
  report the exact same effective window.
- Validation reports: [V01-02](../validations/F-CTX-01/V01-02.md),
  [V02-02](../validations/F-CTX-01/V02-02.md),
  [V05-01](../validations/F-CTX-01/V05-01.md)

### F-CTX-01-P1-03: Runtime token-limit mutation reports success but leaves the live context unchanged

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/capabilities.rs:484`,
  `echo-agent/src/agent/react/mod.rs:336`,
  `echo-agent-cli/src/tauri/commands/providers.rs:110`,
  `echo-agent-cli/src/tui/events.rs:2720`,
  `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:433`
- Reachability: GUI provider updates, TUI model switching, and pooled-agent model
  updates call public `set_token_limit` on existing Agent instances.
- Expected invariant: a successful runtime context-window update applies to the
  next request's ContextManager limit, TokenBudget total, compressor policy, and
  observability atomically, or returns an error requiring agent reconstruction.
- Observed behavior: the setter changes only `self.config.token_limit`. The
  ContextManager captured its own limit, budget, and optional compressor at Agent
  construction and has no update call here.
- Impact: model switches appear configured while subsequent prompts keep the old
  compression threshold/window, potentially overfilling a smaller model or
  prematurely compressing a larger one across all EKO interaction modes.
- Root cause: mutable configuration is not the runtime authority, but the API
  exposes field mutation as though it were.
- Direction: make runtime model/window replacement a fallible async framework
  operation that acquires the execution/context boundary and rebuilds all derived
  limit state. Delete `set_token_limit` after GUI/TUI/pool callers migrate; if
  safe mutation is not supported, require explicit Agent reconstruction.
- Regression validation: warm an Agent, switch large->small and small->large
  through each public caller, inspect the immediately next prepared request and
  compression event.
- Validation reports: [V02-02](../validations/F-CTX-01/V02-02.md),
  [V03-06](../validations/F-CTX-01/V03-06.md)

### F-CTX-01-P1-04: Attachments are free in accounting and one public multimodal path bypasses preparation

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/llm/types.rs:13`,
  `echo-agent/echo-core/src/llm/types.rs:90`,
  `echo-agent/echo-state/src/compression/mod.rs:1541`,
  `echo-agent/src/agent/react/run/context.rs:561`,
  `echo-agent/src/agent/react/mod.rs:3170`
- Reachability: public streaming Message entry points preserve ImageUrl/File parts
  through ContextManager; public `chat_multimodal` sends the stored context
  directly through the selected LLM client.
- Expected invariant: every provider-visible attachment has a conservative/model-
  specific cost or typed unsupported/unbounded decision, and every multimodal
  request goes through the same budget/preparation path.
- Observed behavior: `MessageContent::as_text` discards all image URLs/base64 and
  file names/content, so these parts count as zero. Streaming preparation uses
  this text-only value. Non-streaming `chat_multimodal` never calls `prepare`,
  sends `max_tokens=None`, and bypasses configured context/output selection.
- Impact: large inline files/images can reach providers after a false within-
  budget decision; the direct method can grow context indefinitely and behaves
  differently from documented multimodal streaming.
- Root cause: text extraction was reused as token accounting and an older direct
  multimodal implementation remains outside the canonical ReAct request path.
- Direction: add typed content-part accounting at the selected provider/model
  boundary and route all multimodal entry points through one prepared request
  builder. Delete the direct raw-context implementation once callers migrate.
- Regression validation: inline file/image URL/base64 at under/over boundaries
  through streaming and non-streaming public APIs; captured requests and errors
  must be equivalent.
- Validation reports: [V03-01](../validations/F-CTX-01/V03-01.md),
  [V03-06](../validations/F-CTX-01/V03-06.md)

### F-CTX-01-P1-05: Default compression does not establish a within-budget postcondition

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/mod.rs:346`,
  `echo-agent/echo-state/src/compression/mod.rs:1297`,
  `echo-agent/echo-state/src/compression/mod.rs:1335`,
  `echo-agent/echo-state/src/compression/mod.rs:1525`,
  `echo-agent/echo-state/src/compression/compressor/sliding_window.rs:29`
- Reachability: enabled default TokenBudget causes `new_inner` to install
  `SlidingWindowCompressor::new(40)`; normal iterations invoke it whenever the
  text estimate exceeds the conversation allowance.
- Expected invariant: auto-compression returns total protected + compressed +
  canonical context within the effective allowance or a typed cannot-fit error.
- Observed behavior: SlidingWindow ignores `CompressionInput.token_limit` and
  retains the last 40 non-system messages solely by count. Protected messages are
  excluded before compression, then merged; canonical content is injected after
  compression. No final estimate/assertion/retry checks the complete buffer.
- Impact: 40 large recent messages, one huge recent message, or protected content
  larger than the window still produces an over-limit request after a successful
  “compression”, making the primary OOM/provider-limit defense non-binding.
- Root cause: a message-count retention policy is used as the token-limit default,
  while the manager treats compressor completion as proof of budget compliance.
- Direction: make the default compressor token-driven and define the manager's
  universal postcondition after protected/canonical reinsertion. If protected
  content alone cannot fit, return a typed error rather than silently sending it.
  Retain optional count caps only as supplementary guards.
- Regression validation: 40 large messages, one giant message, protected-only
  overflow, canonical reinjection overflow, and multilingual estimates; every
  returned context must be within the declared allowance.
- Validation reports: [V03-02](../validations/F-CTX-01/V03-02.md),
  [V03-06](../validations/F-CTX-01/V03-06.md),
  [V05-01](../validations/F-CTX-01/V05-01.md)

### F-CTX-01-P2-06: Public ContextAssembler does not enforce its total budget contract

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/context/mod.rs:31`,
  `echo-agent/src/context/mod.rs:111`,
  `echo-agent/src/context/mod.rs:183`,
  `echo-agent/src/context/mod.rs:200`,
  `echo-agent/src/context/mod.rs:223`
- Reachability: this is a public documented framework building block with
  examples for independent consumers; it is explicitly not the default ReAct
  path, which prevents falsely crediting ReAct with its behavior.
- Expected invariant: `total_tokens`, `user_reserve`, and per-source limits use
  one token unit and deterministically constrain the complete assembled list.
- Observed behavior: `total_tokens` and `user_reserve` are never read. System,
  developer/project/task/hook, subagent reports, and user input are unbounded.
  Memory uses character count as though it were tokens; history/tools use UTF-8
  byte length/4 and unchecked accumulation.
- Impact: custom-loop consumers can follow the documented API and receive a
  context far above their declared total while believing the user reserve is
  protected; multilingual sources receive inconsistent treatment.
- Root cause: per-source trimming was implemented without a final allocator and
  independently of the live TokenBudget/tokenizer system.
- Direction: make ContextAssembler delegate to the canonical request accounting
  engine, or delete its budget surface and document it solely as an ordering
  helper. Do not create a third budget implementation.
- Regression validation: source-by-source and aggregate tables with Chinese,
  emoji, zero reserve, protected content, over-total user input, and maximum
  counters; final accounting must conserve the configured window.
- Validation reports: [V01-02](../validations/F-CTX-01/V01-02.md),
  [V03-03](../validations/F-CTX-01/V03-03.md)

### F-CTX-01-P2-07: ContextSelector tie order is nondeterministic

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/context/selector.rs:50`,
  `echo-agent/src/context/selector.rs:85`,
  `echo-agent/src/context/selector.rs:99`
- Reachability: a public documented selector is exercised by framework examples
  and available to independent consumers; no default ReAct caller was found.
- Expected invariant: identical inputs yield identical ranked paths, including
  ties; weights are finite validated values.
- Observed behavior: scores originate from randomized HashMap iteration, and the
  sort comparator returns Equal for equal or incomparable/NaN values without a
  secondary path key. `take(max_files)` therefore selects arbitrary tied files.
- Impact: custom Agent context/cache behavior and answers can vary between
  processes for the same task, especially when recency/git weights tie.
- Root cause: ranking defines only a partial score order and exposes raw f64
  weights without validation.
- Direction: reject/normalize non-finite weights and sort by validated score,
  then stable normalized path (and another documented key if needed).
- Regression validation: insertion-order permutations, all-tie and partial-tie
  sets, NaN/infinity/negative weights, and repeated-process determinism.
- Validation reports: [V01-02](../validations/F-CTX-01/V01-02.md),
  [V03-04](../validations/F-CTX-01/V03-04.md)

### F-CTX-01-P2-08: Ordinary message text can forge framework-owned projection identity

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/compression/mod.rs:30`,
  `echo-agent/echo-state/src/compression/mod.rs:36`,
  `echo-agent/echo-state/src/compression/mod.rs:678`,
  `echo-agent/src/agent/snapshot.rs:35`,
  `echo-agent/src/agent/react/run/context.rs:607`
- Reachability: plain and multimodal user messages enter ContextManager unchanged;
  protected splitting and ConversationStore transcript filtering both call the
  public predicate.
- Expected invariant: framework-owned dynamic context has typed/out-of-band
  identity that ordinary content cannot acquire; all user content remains
  persistable and subject to normal compression.
- Observed behavior: any text or first multimodal text part starting with the
  private literal prefix is classified as a projection without validating a
  framework-owned marker/scope. It is protected from compression and excluded
  from transcript persistence.
- Impact: copied/generated/user content matching the prefix can silently disappear
  from saved conversation while remaining indefinitely pinned in the model
  context. This is correctness/data-retention behavior, not a remote-security
  claim under EKO's local threat model.
- Root cause: presentation content doubles as ownership/type metadata.
- Direction: store projection identity in a typed message metadata/envelope owned
  by ContextManager and strip it only at provider/persistence adapters. Delete
  content-prefix identity checks after migration.
- Regression validation: plain/multimodal user prefix fixtures must persist and
  compress normally; only messages created through projection APIs are protected
  and filtered.
- Validation reports: [V03-05](../validations/F-CTX-01/V03-05.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00-01 | Protocol, scope, dependencies, layering, de-duplication | yes | passed | [report](../validations/F-CTX-01/V00-01.md) |
| V01-01 | Broad definition/context read | no, superseded | inconclusive due truncation | [report](../validations/F-CTX-01/V01-01.md) |
| V01-02 | Bounded definition/duplicate/consumer inventory | yes | passed | [report](../validations/F-CTX-01/V01-02.md) |
| V02-01 | Complete request-budget reachability | yes | failed invariant | [report](../validations/F-CTX-01/V02-01.md) |
| V02-02 | Model window and runtime update reachability | yes | failed invariant | [report](../validations/F-CTX-01/V02-02.md) |
| V03-01 | Attachment accounting and multimodal path table | yes | failed invariant | [report](../validations/F-CTX-01/V03-01.md) |
| V03-02 | Compression within-budget postcondition | yes | failed invariant | [report](../validations/F-CTX-01/V03-02.md) |
| V03-03 | ContextAssembler field/unit/total mapping | yes | failed invariant | [report](../validations/F-CTX-01/V03-03.md) |
| V03-04 | ContextSelector deterministic tie ordering | yes | failed invariant | [report](../validations/F-CTX-01/V03-04.md) |
| V03-05 | Projection identity and persistence boundary | yes | failed invariant | [report](../validations/F-CTX-01/V03-05.md) |
| V03-06 | Existing source-test coverage inventory | yes | passed as inventory | [report](../validations/F-CTX-01/V03-06.md) |
| V04-01 | Dynamic boundary fixture matrix | no per explicit review rule | not run; future validation | [report](../validations/F-CTX-01/V04-01.md) |
| V05-01 | Current comment/history drift | yes | failed claim | [report](../validations/F-CTX-01/V05-01.md) |
| V99-01 | Final integrity gate with zsh special-variable bug | evidence integrity | inconclusive | [report](../validations/F-CTX-01/V99-01.md) |
| V99-02 | Corrected gate with wrong ripgrep short option | evidence integrity | inconclusive | [report](../validations/F-CTX-01/V99-02.md) |
| V99-03 | Final link/header/finding/source-clean gate | yes | passed | [report](../validations/F-CTX-01/V99-03.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `TokenBudget` divides system/tools/history/output/safety | current as a standalone calculator; regressed in live caller mapping | [V02-01](../validations/F-CTX-01/V02-01.md) |
| `ModelProfile.context_window` drives harness behavior | stale for context construction | [V02-02](../validations/F-CTX-01/V02-02.md) |
| `TokenBudgetConfig.total_window=None` auto-detects the model | stale; `build` only accepts a caller fallback and normal Agent passes token_limit | [V02-02](../validations/F-CTX-01/V02-02.md) |
| Default SlidingWindow keeps recent messages that fit token_limit | regressed; it keeps 40 messages and ignores the field | [V03-02](../validations/F-CTX-01/V03-02.md) |
| Shared CalibratedTokenizer improves normal text estimates from reported usage | current on the streaming think path | [V01-02](../validations/F-CTX-01/V01-02.md) |
| Framework projections replace/tombstone by scoped marker | current for API-created projections; content identity remains forgeable | [V03-05](../validations/F-CTX-01/V03-05.md) |

## Coverage And Uncertainty

- No Cargo, rustc, tests, builds, Clippy, feature matrix, remote provider call, or
  dynamic fixture was executed. V04-01 preserves the implementation acceptance
  matrix; this does not block conclusions based on absent readers/branches.
- Provider-native exact image/token accounting is model-specific and was not
  asserted. The finding requires only that the current cost is always zero and
  one public path skips preparation entirely.
- F-CMP-01 must assess semantic loss, tool-pair preservation across every
  compressor, repeated compression, recovery facts, and LLM-summary behavior.
  F-CTX-01 establishes only admission/accounting/postcondition boundaries.
- F-REL-01 owns percentage validation and arithmetic panic. No duplicate finding
  is created here even though live integration calls that primitive.
- Model window values themselves are time-sensitive. This report does not judge
  whether hardcoded numeric values match current vendor documentation; it proves
  the resolved field is not consumed by ContextManager.

## Handoff

- F-CMP-01 may rely on the verified ContextManager call graph and must test every
  compressor against a total postcondition after protected/canonical merge.
- Provider/tool synthesis should unify schema and attachment accounting with the
  selected request snapshot, not bolt more estimates onto ToolManager metrics.
- Cross-repository model switching must consume one fallible framework
  reconfiguration API; application modes should not rebuild budget semantics.
- Iteration design should coordinate P1-01 through P1-05 as one request-budget
  authority migration while keeping separate regression IDs. P2-06 should be
  deleted or delegated to that authority, not become a third implementation.
- This report becomes stale if ContextManager `prepare`, SlidingWindow,
  MessageContent accounting, React construction/setters, model-profile limits,
  request schema selection, projection identity, or public context helpers change.
