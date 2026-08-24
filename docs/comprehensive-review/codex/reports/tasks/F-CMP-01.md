# F-CMP-01: Compression correctness

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: `echo-agent` clean; `echo-agent-cli` had unrelated modified generated `ApiError.ts` and `StreamingEvent.ts`; review read neither file and edited no source

## Question

Do all framework compressors and their root ReAct adapters preserve provider
protocol, instructions, active tasks, recent evidence, attachment identity, and
recovery facts across repeated compression and failure paths?

## Scope

- Portable compression contracts and canonical/structured summary models in
  `echo-core/src/compression.rs`.
- All strategies, manager invariants, verifier, visibility horizon, and existing
  tests under `echo-state/src/compression`.
- Root compression facade, ReAct construction/configuration, automatic/manual
  compaction paths, memory promoter, live skill projections, and EKO's thin
  TaskRuntime recovery projection.
- Definition/export/duplicate search; real ReAct reachability; strategy and
  preservation matrices; repeated compression; fallback/error/cancellation;
  UTF-8/panic/overflow; static test inventory.

## Out Of Scope

- Fixing source or changing review indexes.
- Re-proving token estimation and complete-context budget enforcement. The
  canonical finding is [F-CTX-01-P1-05](F-CTX-01.md#f-ctx-01-p1-05-default-compression-does-not-establish-a-within-budget-postcondition).
- Generic File/InMemory Store correctness, owned by [F-MEM-01](F-MEM-01.md),
  except for the compression-to-promoter acknowledgement contract.
- Runtime-checkpoint store correctness (`F-RCT-05`) and application TaskRuntime
  graph semantics (`A-TSK-01`).
- Cargo, rustc, test, build, or dynamic fixture execution. Those were expressly
  prohibited for this review stage and remain future regression work.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, and `F-CMP-01` task card;
  Codex track `README.md`.
- Dependency reports [F-CTX-01](F-CTX-01.md) and [F-MEM-01](F-MEM-01.md), read
  only for their accepted boundary conclusions.
- Current source and current inline API/test documentation as hypotheses.
- No report from another reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Compressor contracts, protocol-group preservation, canonical/protected reinsertion, summary replacement/merge, verifier, cancellation, promotion acknowledgement, and numeric safety belong to the reusable framework. |
| EKO product policy | The TaskRuntime recovery capsule and current Subagent task brief are EKO projections of file-backed product state. Their contents stay in the application; the framework only provides a lossless projection/protection primitive. |
| Adapter boundary | EKO's `PreModelContextProjector` derives one marked message and registers it at the model boundary. It owns no compression algorithm, verifier, or recovery state authority. |
| Duplicate search | Searched both repositories for the compression trait/types, all implementations/constructors, manager, projections, task/skill markers, selectors, promoter, verifier, and live callers. One framework authority exists; EKO has only configuration and projections. |
| Migration deletion | Do not delete public optional compressors merely because EKO selects SlidingWindow. Repairs should replace faulty policy inside existing strategies. If `current_query` or unused canonical fields are not given coherent semantics, delete those misleading fields and all forwarding code rather than retain inert parallel promises. |

## Current Path

```text
ReactAgent::new_inner
  -> ContextManager(system, calibrated tokenizer, budget)
  -> default SlidingWindow(40) when finite
  -> canonical system/project-rule adapter

EKO runtime + pooled agents
  -> AppConfig::apply_compressor
  -> SlidingWindow | Summary | Hybrid | Adaptive

each streaming ReAct iteration
  -> run_compact
  -> pre-compaction best-effort memory flush + runtime checkpoint
  -> EKO PreModelContextProjector (active TaskRuntime capsule)
  -> ContextManager::apply_projection_scope
  -> ContextManager::prepare
       optional VisibilityHorizon
       split protected -> compressor -> merge protected
       promoter -> tool-pair sanitizer -> summary verifier
       canonical reinjection -> model messages
```

The real EKO YAML currently selects `sliding`; the reusable framework default
config is `summary`. Hybrid and Adaptive are live selector values,
VisibilityHorizon is builder-installable, and IncrementalSummary is a public
framework option. Optional external reuse is sufficient reason to retain it.

## Strategy And Invariant Matrix

| Strategy/path | Selection | Old-history policy | Tool protocol | Repeated state | Main static conclusion |
|---|---|---|---|---|---|
| SlidingWindow | framework default; EKO current YAML; fallback | keep N non-system messages | boundary may split; manager repairs IDs | count-idempotent | active/recovery facts survive only when protected or recent; ignores token/query semantics |
| Summary | framework config default | LLM summary + recent N | manager repairs after summary | old summary system messages accumulate | empty success can bypass verifier; attachments are text-only in prompt |
| IncrementalSummary | public external option | private structured merge + recent N | manager repairs when used through manager | private merge plus buffer summary accumulation | repeated summaries are not one replaceable artifact |
| Hybrid | selectable | ordered compressor pipeline | final manager repair | inherits stage behavior | custom short checkpoint ID can panic in stage label |
| Adaptive | selectable | L1 snip/fold, L2 micro, L3 collapse, L4 summary, L5 emergency | L1 fold can leave non-contiguous results | no canonical summary state | emergency/count layers discard recovery facts; unchecked public-config arithmetic |
| VisibilityHorizon | builder option | summarize old tool groups | deliberately clears compacted calls | stable symbolic summaries | only adjacent tool-result runs are grouped; best-effort promoter has no acknowledgement |
| ContextManager | all canonical paths | protected split/merge, fallback, verify, reinject | sanitizer | checkpoints/metrics | protected reinsertion can move projections before system; P1 verifier failures do not govern output |

## Findings

### F-CMP-01-P1-01: Adaptive tool folding can emit a provider-invalid split result group

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/compression/levels.rs:364`,
  `echo-agent/echo-state/src/compression/levels.rs:391`,
  `echo-agent/echo-state/src/compression/mod.rs:1448`,
  `echo-agent/echo-state/src/compression/mod.rs:1608`
- Reachability: `compress_strategy=adaptive` installs Adaptive; normal ReAct
  `prepare` runs L1 Fold then the common sanitizer before the provider call.
- Expected invariant: an assistant message's tool results form one contiguous
  group immediately after it, with one result or placeholder per call ID.
- Observed behavior: L1 Fold replaces removed leading tool results with a
  `user` summary but retains later results. Sanitization inserts missing
  placeholders before that user message yet leaves retained results after it.
- Impact: OpenAI-compatible providers can reject the next request even though
  the framework reports tool pairing sanitized.
- Root cause: folding treats consecutive result messages as an independent
  count window; sanitizer validates global IDs rather than assistant-group
  adjacency.
- Direction: model an assistant tool exchange as one atomic group and replace
  or retain it as a unit. Delete the message-run folding logic once the group
  transform owns L1 behavior; make sanitizer validate adjacency.
- Regression validation: capture the provider request for one three-call group
  folded with `keep_latest=2`; require contiguous ordered placeholders/results.
- Validation reports: [V04-01](../validations/F-CMP-01/V04-01.md)

### F-CMP-01-P1-02: Protected reinsertion can move a dynamic user projection ahead of the system prompt

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/compression/mod.rs:747`,
  `echo-agent/echo-state/src/compression/mod.rs:781`,
  `echo-agent/src/agent/react/run/phases/compact.rs:44`,
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/compact_context.rs:133`
- Reachability: every EKO task turn derives a protected user-role runtime capsule
  before normal `prepare`; the same mechanism protects live skill and product
  projections.
- Expected invariant: protection is lossless and preserves role-region order,
  especially system instructions before user history.
- Observed behavior: reinsertion uses the protected message's old count of
  trailing compressible messages. When compression shrinks below that distance,
  `saturating_sub` chooses index zero and can place a user projection before the
  base system prompt.
- Impact: provider message ordering and instruction priority can change exactly
  during long active tasks that depend on the recovery capsule.
- Root cause: an old positional distance is applied to a structurally different
  compressed list without system/history boundary constraints.
- Direction: represent protected placement by semantic region and stable local
  anchors; enforce all system/projection ordering invariants after merge. Delete
  the trailing-count approximation.
- Regression validation: compress a long history with protected system, middle
  user, task capsule, and recent user messages to 1/2/N items; assert exact roles,
  identity, relative order, and repeated-pass stability.
- Validation reports: [V03-01](../validations/F-CMP-01/V03-01.md)

### F-CMP-01-P1-03: Repeated summary compression accumulates obsolete system summaries

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/compression/compressor/summary.rs:292`,
  `echo-agent/echo-state/src/compression/compressor/summary.rs:319`,
  `echo-agent/echo-state/src/compression/compressor/summary.rs:346`,
  `echo-agent/echo-state/src/compression/compressor/summary.rs:604`,
  `echo-agent/echo-state/src/compression/compressor/summary.rs:684`
- Reachability: Summary is the framework configuration default and is selectable
  in EKO; IncrementalSummary is a public reusable option.
- Expected invariant: repeated compression leaves exactly one current summary
  artifact which subsumes earlier compressed history.
- Observed behavior: all system messages, including prior generated summaries,
  are partitioned out of the next summary input and retained. A new system
  summary is appended on every pass. IncrementalSummary also maintains private
  merged state, so obsolete buffer summaries coexist with its newest state.
- Impact: context grows with stale/contradictory recovery facts, prompt prefix
  churns, and old tasks/errors can continue to influence later decisions.
- Root cause: generated summaries have no typed marker/replacement lifecycle and
  are indistinguishable from permanent system instructions.
- Direction: maintain one replaceable, versioned summary projection and include
  its authoritative content in the next merge exactly once. Delete accumulated
  legacy summary messages during the same transition.
- Regression validation: three successive compressions with changed task/error/
  preference facts; require one summary, correct supersession, bounded tokens,
  and no stale fact resurrection.
- Validation reports: [V05-01](../validations/F-CMP-01/V05-01.md)

### F-CMP-01-P1-04: Empty successful summary output bypasses the non-empty verifier

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/compression/compressor/summary.rs:269`,
  `echo-agent/echo-state/src/compression/compressor/summary.rs:328`,
  `echo-agent/echo-state/src/compression/verifier.rs:50`,
  `echo-agent/echo-state/src/compression/mod.rs:1461`
- Reachability: any summary provider may return an HTTP-success response with
  empty content; normal `prepare` then invokes the verifier.
- Expected invariant: empty/malformed summary content fails closed and invokes a
  loss-minimizing fallback before old messages are committed as evicted.
- Observed behavior: empty text is accepted as `Some("")`, old messages are
  evicted, and `verify_compression` conditionally omits `summary_not_empty` when
  the checkpoint summary is empty.
- Impact: an entire old conversation can be replaced by an empty summary without
  the intended P0 fallback.
- Root cause: empty output is represented as success, and the verifier guards
  the check with the same condition the check must reject.
- Direction: normalize blank response to typed failure and always run non-empty
  validation for summary strategies before committing eviction.
- Regression validation: empty, whitespace, malformed structured JSON, truncated
  response, and content-less provider success; verify original/fallback facts.
- Validation reports: [V05-01](../validations/F-CMP-01/V05-01.md)

### F-CMP-01-P1-05: Eviction commits without a durable promotion outcome

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/compression/mod.rs:69`,
  `echo-agent/echo-state/src/compression/mod.rs:1353`,
  `echo-agent/echo-state/src/compression/mod.rs:1358`,
  `echo-agent/src/memory_promoter.rs:60`,
  `echo-agent/src/memory_promoter.rs:91`
- Reachability: attaching a Store installs StoreMemoryPromoter; every successful
  automatic/forced compression supplies evicted messages before finalizing its
  checkpoint.
- Expected invariant: irreversible eviction is acknowledged only after facts are
  durably written or after a surfaced policy decision; metrics distinguish
  submitted, extracted, written, deduplicated, and failed items.
- Observed behavior: `MemoryPromoter::promote` returns `()`. Store errors are
  logged at debug and swallowed. ContextManager still keeps the compressed
  buffer and records every evicted message as `memory_promotion_count`, although
  some yield no fact and writes may all fail.
- Impact: recovery facts can be lost while traces/checkpoints falsely imply that
  promotion succeeded.
- Root cause: a fire-and-forget callback is used as a durability boundary and a
  submitted-message count is named as promotion success.
- Direction: return a typed promotion receipt and define retry/retain/abort
  policy. Keep F-MEM-01's Store as the persistence authority; do not add a second
  compression store.
- Regression validation: extractor-empty, partial write failure, total Store
  failure, retry/dedup, and crash-boundary scenarios with truthful receipts.
- Validation reports: [V06-01](../validations/F-CMP-01/V06-01.md)

### F-CMP-01-P1-06: Compression LLM calls ignore active run cancellation

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/phases/compact.rs:55`,
  `echo-agent/echo-state/src/compression/compressor/summary.rs:239`,
  `echo-agent/echo-state/src/compression/compressor/summary.rs:468`,
  `echo-agent/echo-state/src/compression/compressor/summary.rs:537`,
  `echo-agent/echo-state/src/compression/levels.rs:523`
- Reachability: Summary/Incremental/Adaptive L4 may run before every ReAct think
  call while `run_compact` holds the context mutex and awaits `prepare`.
- Expected invariant: cancelling a run cancels every model call in that run,
  including pre-think compression, and releases context ownership promptly.
- Observed behavior: compression's ChatRequests hard-code `cancel_token: None`;
  `CompressionInput` has no cancellation field and `run_compact` performs no
  cancellation select around `prepare`.
- Impact: stop/cancel can remain stuck behind a slow summary provider and prevent
  other context operations until that call returns or its unrelated timeout
  fires.
- Root cause: cancellation was modeled only on primary think/tool paths, not in
  the compressor contract.
- Direction: pass the invocation cancellation context through CompressionInput
  and all nested calls; define atomic rollback if cancellation arrives after
  compression starts but before commit.
- Regression validation: cancel during structured call, natural-language
  fallback, Adaptive L4, and verifier fallback; require prompt termination and
  original-buffer consistency.
- Validation reports: [V06-01](../validations/F-CMP-01/V06-01.md)

### F-CMP-01-P1-07: Public compressor inputs retain panic and overflow paths

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/compression/compressor/hybrid.rs:95`,
  `echo-agent/echo-state/src/compression/levels.rs:107`,
  `echo-agent/echo-state/src/compression/levels.rs:226`,
  `echo-agent/echo-state/src/compression/levels.rs:348`,
  `echo-agent/echo-state/src/compression/horizon.rs:295`
- Reachability: Hybrid accepts arbitrary public `ContextCompressor` stages;
  Adaptive and Horizon configs are public and deserializable/framework-settable.
- Expected invariant: malformed custom checkpoint data and extreme configuration
  return errors or saturate safely, never panic/wrap.
- Observed behavior: Hybrid byte-slices `checkpoint_id[..8]`; a custom short or
  multibyte ID can panic. Threshold/character calculations use unchecked
  multiplication (`*60`, `*2`, `*4`, `keep*2`) and count additions.
- Impact: one reusable framework request can crash or misapply emergency policy
  under otherwise type-valid input.
- Root cause: UUID/config assumptions are not encoded in the contracts and are
  enforced with unchecked byte/numeric operations.
- Direction: use character-safe preview, checked/saturating arithmetic, and
  constructor/deserialization validation with typed errors.
- Regression validation: empty/short/emoji IDs and `usize::MAX` for every public
  numeric field; no panic and deterministic typed outcome.
- Validation reports: [V07-01](../validations/F-CMP-01/V07-01.md)

### F-CMP-01-P1-08: Evicted attachments have no semantic recovery representation

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/llm/types.rs:13`,
  `echo-agent/echo-state/src/compression/compressor/summary.rs:60`,
  `echo-agent/echo-state/src/compression/verifier.rs:192`,
  `echo-agent/src/memory_promoter.rs:117`
- Reachability: multimodal messages share the ContextManager buffer; Summary,
  Adaptive, and count fallbacks may evict any old non-protected message.
- Expected invariant: after evicting an image/file attachment, the compressed
  context retains typed identity, name/media kind, content hash or durable
  retrieval reference required to continue the task.
- Observed behavior: summary prompts include only text parts; verifier inspects
  textual paths only; StoreMemoryPromoter reads `as_text()` and has no attachment
  schema. File/image-only messages can therefore be evicted without any recovery
  fact.
- Impact: later turns cannot inspect or refer reliably to evidence the user
  uploaded earlier in the same task.
- Root cause: generic conversation summarization is text-shaped although Message
  is typed multimodal.
- Direction: generate a typed attachment manifest before lossy compression and
  preserve durable retrieval handles. Do not embed unlimited payload bytes in
  summaries.
- Regression validation: image URL/base64 and named file across repeated Summary,
  Adaptive L3/L5, fallback, restore, and Subagent task continuation.
- Validation reports: [V05-01](../validations/F-CMP-01/V05-01.md)

### F-CMP-01-P1-09: Detected loss of recovery facts does not influence model input

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/compression/verifier.rs:69`,
  `echo-agent/echo-state/src/compression/verifier.rs:90`,
  `echo-agent/echo-state/src/compression/mod.rs:1464`,
  `echo-agent/src/agent/react/run/phases/compact.rs:65`
- Reachability: Summary/Adaptive L4 checkpoints invoke verifier on the live
  `prepare` path; `run_compact` consumes only messages/stats.
- Expected invariant: detected loss of pending tasks, errors/resolutions, and
  user constraints blocks or repairs the lossy summary before the next model
  request.
- Observed behavior: these checks are P1, while `SummaryVerification.passed`
  considers only P0. `prepare` falls back only when `passed` is false and
  `run_compact` neither logs nor acts on returned P1 failures.
- Impact: the verifier can correctly detect lost recovery facts and still send
  the known-defective summary with no visible warning to the caller.
- Root cause: verification severity is an observation label without an explicit
  acceptance policy for core recovery facts.
- Direction: define required fact classes per compression policy and make the
  manager's commit decision consume them; remove unused advisory checks if no
  caller will act on them.
- Regression validation: summaries omitting only TODO, error resolution, or user
  constraint; assert repair/fallback or a typed degraded result, never silent use.
- Validation reports: [V05-01](../validations/F-CMP-01/V05-01.md),
  [V08-01](../validations/F-CMP-01/V08-01.md)

### F-CMP-01-P2-01: CanonicalContext promises fields the root adapter does not represent coherently

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-core/src/compression.rs:345`,
  `echo-agent/echo-core/src/compression.rs:362`,
  `echo-agent/echo-core/src/compression.rs:383`,
  `echo-agent/src/agent/react/mod.rs:676`,
  `echo-agent/src/agent/react/mod.rs:703`,
  `echo-agent/src/agent/react/mod.rs:358`
- Reachability: every ReactAgent builds CanonicalContext and reinjects it after
  compression; external consumers can also supply all public fields.
- Expected invariant: each canonical source is emitted once and every advertised
  field participates in reinjection.
- Observed behavior: `skill_injections` makes `has_any` true but is never emitted.
  The root adapter builds `system_prompt` after project-rule injection and also
  separately populates `project_rules`, so compression can reinsert the same
  rules again as a supplemental system message. Live activated skills use the
  newer projection path, leaving the public canonical field stale.
- Impact: independent consumers can silently lose canonical skill text, while
  the built-in path duplicates project instructions and tokens after compression.
- Root cause: CanonicalContext mixes obsolete snapshot fields with newer
  projection ownership and the root adapter does not normalize sources.
- Direction: use projections as one replaceable canonical mechanism or make each
  field losslessly emitted exactly once. Remove obsolete fields/adapters after
  migration; do not create a second application canonical store.
- Regression validation: base prompt + project rules + two active skills across
  three passes; exact content once, stable order, no accumulation.
- Validation reports: [V03-01](../validations/F-CMP-01/V03-01.md)

### F-CMP-01-P2-02: CompressionInput.current_query advertises protection but has no policy effect

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/compression.rs:45`,
  `echo-agent/echo-state/src/compression/mod.rs:1239`,
  `echo-agent/echo-state/src/compression/mod.rs:1342`,
  `echo-agent/echo-state/src/compression/compressor/hybrid.rs:52`
- Reachability: this is a public compressor input and manager parameter; normal
  ReAct explicitly calls `prepare(None)`.
- Expected invariant: the field either protects/focuses the active query as
  documented or is absent so consumers do not rely on it.
- Observed behavior: no built-in compressor reads it for selection, protection,
  or summary focus. Hybrid only copies it into checkpoint focus if explicit
  focus is absent.
- Impact: framework consumers may believe active task context is protected when
  count-based compression can evict it.
- Root cause: a reserved compatibility field outlived its implementation and is
  forwarded without semantics.
- Direction: define one active-query projection/focus contract or delete the
  field and forwarding. EKO's authoritative task capsule remains application
  policy and should not be moved into this generic field.
- Regression validation: active query placed outside the recent window for all
  strategies; prove documented behavior or API removal.
- Validation reports: [V03-01](../validations/F-CMP-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Contract/implementation/export matrix and duplicate search | yes | passed | [V01-01](../validations/F-CMP-01/V01-01.md) |
| V02 | Definition -> configuration -> real ReAct reachability and selector matrix | yes | passed | [V02-01](../validations/F-CMP-01/V02-01.md) |
| V03 | System/canonical/protected/projection/active task and query preservation | yes | failed | [V03-01](../validations/F-CMP-01/V03-01.md) |
| V04 | Tool-call/result identity, adjacency, ordering, and policy matrix | yes | failed | [V04-01](../validations/F-CMP-01/V04-01.md) |
| V05 | Repeated summary, merge, empty output, attachments, and recovery facts | yes | failed | [V05-01](../validations/F-CMP-01/V05-01.md) |
| V06 | Promotion acknowledgement, fallback, error, and cancellation paths | yes | failed | [V06-01](../validations/F-CMP-01/V06-01.md) |
| V07 | UTF-8, panic, overflow, and token-postcondition ownership | yes | failed | [V07-01](../validations/F-CMP-01/V07-01.md) |
| V08 | Existing test assertion inventory and uncovered cases | yes | passed | [V08-01](../validations/F-CMP-01/V08-01.md) |
| V09 | Dependency/historical classification and finding deduplication | yes | passed | [V09-01](../validations/F-CMP-01/V09-01.md) |
| V10 | Exact header/link/ID/path/source-state integrity | yes | passed on attempt 03 | [V10-01](../validations/F-CMP-01/V10-01.md), [V10-02](../validations/F-CMP-01/V10-02.md), [V10-03](../validations/F-CMP-01/V10-03.md) |
| V30 | Primary source-anchor sampling and acceptance | yes | passed | [V30-01](../validations/F-CMP-01/V30-01.md) |
| D01 | Repeated Summary/Incremental manager fixture with mock LLM | future | not_run | prohibited during static review; no fake report created |
| D02 | Provider-capture tool-group matrix | future | not_run | prohibited during static review; no fake report created |
| D03 | Cancel during compression and fallback | future | not_run | prohibited during static review; no fake report created |
| D04 | Multilingual/attachment/large-context recovery fixture | future | not_run | prohibited during static review; no fake report created |
| D05 | Maximum numeric configuration and custom checkpoint fuzz | future | not_run | prohibited during static review; no fake report created |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-CTX-01-P1-05: default compression lacks a within-budget postcondition | current | `ContextManager::finalize_checkpoint` updates counts but does not enforce the effective limit; canonical ownership remains F-CTX-01 |
| F-CTX-01 defers compressor semantic preservation/recovery facts to F-CMP-01 | current | V03-V06 cover the deferred boundary |
| `StructuredSummary`: field-level merge means “no summary drift” | stale | field merge exists, but obsolete system summaries accumulate; V05 |
| `CompressionInput.current_query` protects active task context | stale | no compressor consumes it; V03 |
| `CanonicalContext` restores rules and skill injections | regressed | project rules can duplicate; skill injection text is not emitted; V03 |
| Invariant 7 documents `post-compression tokens <= target` | stale | its assertion checks only reduction; V08 and F-CTX-01-P1-05 |
| EKO TaskRuntime capsule is a protected, refreshed projection | current | `compact_context.rs` plus `run_compact` registration path; V02/V03 |
| Primary+fallback failure restores original buffer | current | `ContextManager::prepare:1431-1439`; V06 |

## Coverage And Uncertainty

- The static review covered all production/test files in the assigned
  compression directories and the root construction/adaptation/promotion paths.
- No dynamic command was run. Provider-specific rejection details, scheduler
  timing, actual LLM summary quality, and failure injection remain future
  evidence, not grounds to claim these paths pass.
- Cancellation and persistence findings are source-conclusive about missing
  propagation/acknowledgement. Exact latency/data-loss frequency depends on
  provider and Store failures.
- F-CTX-01 owns typed multimodal token accounting. F-CMP-01-P1-08 is narrowly
  about loss of attachment identity/retrieval facts after eviction.
- Current source already has valuable mechanisms: projections, exact canonical
  system restoration, a runtime checkpoint before compression, original-buffer
  restore on dual compressor failure, char-safe truncation, and test coverage
  for several simple invariants. Findings should repair these authorities, not
  introduce parallel managers.

## Handoff

- Primary should independently sample P1-01 (Adaptive group order), P1-02
  (protected insertion index), P1-03/P1-04 (second pass/empty summary), P1-05
  (promotion receipt), and P1-06 (cancel token) before changing status.
- Downstream synthesis may rely on the ownership/export and real ReAct
  reachability conclusions in V01/V02.
- Budget remediation must consume F-CTX-01-P1-05; do not duplicate it as a new
  compression roadmap item.
- Application task-context semantics remain with `A-TSK-01`; framework
  projection/order repair remains here. Store backend correctness remains with
  F-MEM-01.
- This report becomes stale if compressor contracts, `ContextManager::prepare`,
  summary marking/merge, sanitizer, MemoryPromoter, root construction/config, or
  EKO task projection wiring changes.

### Worktree Metadata Correction (2026-08-12)

The initial metadata preserved the parent's earlier observation that two
generated EKO TypeScript files were dirty. Final direct inspection in V10-03
found both source repositories clean at the reviewed commits. No source file was
read, edited, or reverted by this task; the external changes had disappeared
before final handoff.
