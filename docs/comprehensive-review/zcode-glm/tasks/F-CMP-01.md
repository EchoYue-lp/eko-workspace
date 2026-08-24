# F-CMP-01: Compression correctness

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Do compressors preserve protocol pairs, instructions, active tasks, recent
evidence, and recovery facts under repeated compression?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-state/src/compression/mod.rs` (2584 lines, key ranges
  `:311-355`, `:380-446`, `:468-510`, `:640-793`, `:834-940`,
  `:1027-1240`, `:1243-1548`, `:1562-1710`, `:1712-1830`) — `ContextManager`,
  `push` / `apply_hard_cap`, `should_compress`, `split_protected` /
  `merge_protected`, `protected_token_estimate`, `reinject_canonical_context`,
  `force_compress*` family, `prepare` (the compression orchestration loop),
  `sanitize_tool_call_pairing`, `ContextManagerBuilder`.
- `echo-agent/echo-state/src/compression/compressor/sliding_window.rs`
  (full, 96 lines) — `SlidingWindowCompressor`.
- `echo-agent/echo-state/src/compression/compressor/summary.rs`
  (full, 742 lines) — `SummaryCompressor`,
  `IncrementalSummaryCompressor`, `StructuredSummary` prompt builders.
- `echo-agent/echo-state/src/compression/compressor/hybrid.rs`
  (full, 273 lines) — `HybridCompressor`, pipeline + short-circuit,
  `summary_buffer`.
- `echo-agent/echo-state/src/compression/horizon.rs` (full, 929 lines) —
  `VisibilityHorizonCompressor`, tool-group compaction.
- `echo-agent/echo-state/src/compression/levels.rs` (full, 929 lines) —
  `AdaptiveCompressor`, L1–L5 escalation, `safe_truncate`, `tune_for_model`.
- `echo-agent/echo-state/src/compression/invariants.rs` (full, 566 lines) —
  13 invariant tests covering tool-pair integrity, last-user-request,
  system-prompt, protected-markers, pending-tasks, file-paths, token-target,
  idempotency, adaptive escalation, horizon-no-orphans, focus flow.
- `echo-agent/echo-state/src/compression/verifier.rs` (full, 513 lines) —
  `verify_compression`, P0/P1 checks (token target, summary non-empty, last
  query, file paths, pending tasks, errors, preferences).
- `echo-agent/echo-core/src/compression.rs` (full, 467 lines) —
  `ContextCompressor` trait, `CompressionInput`/`Output`,
  `CompressionCheckpoint`, `StructuredSummary` (+ `merge_with`),
  `CanonicalContext`, `ToolPairFix`.
- `echo-agent/echo-core/src/tokenizer.rs:34-127` — `HeuristicTokenizer`,
  `SimpleTokenizer`, `CalibratedTokenizer`.

## Out Of Scope

Deferred to named task IDs:

- Token **budget** accounting and the phantom `(0,0,est)` reservation in
  `budget.allocate` → **F-CTX-01** (already complete; this task consumes
  its conclusions).
- Protected-content **token deduction** from `effective_limit` → **F-CTX-01**
  (finding F-CTX-01-P2-02).
- Tool-definition token cost visibility → **F-CTX-01** (finding
  F-CTX-01-P2-01).
- `Store` / `ConversationStore` durability for the `MemoryPromoter` sink →
  **F-MEM-01** (already complete).
- Subagent context builder (`src/agent/subagent/context*.rs`) → **F-SUB-01**.
- LLM client construction and `chat()` reliability for summary calls →
  **F-LLM-01/02/03**.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  system reminder — especially the UTF-8 / no-panic Rust constraints, the
  framework-vs-application layering rule, the "first check if it already
  exists" rule, and the local-assistant threat model).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-CTX-01.md` (in full).
  F-CTX-01 establishes that protected content survives compression via
  markers + projection envelope + canonical re-injection + tool-pair
  sanitisation (V02-01). It flags that `budget.allocate(0,0,est)` makes
  system/tool reservations phantom (F-CTX-01-P2-01) and that
  `protected_token_estimate` is observability-only (F-CTX-01-P2-02). This
  task relies on both conclusions and does not re-litigate them.
- `docs/comprehensive-review/zcode-glm/tasks/F-MEM-01.md` (in full).
  F-MEM-01 establishes that `FileStore` / `FileConversationStore` are the
  durable sinks for the `MemoryPromoter` callback and are path-safe. This
  task relies on the projection round-trip losslessness (F-MEM-01 V04-01)
  for session-resume scenarios where `sanitize_tool_call_pairing` must
  repair pre-existing invalid sequences.

Historical documents treated as hypotheses:

- `echo-state/src/compression/invariants.rs:1-19` module docstring claims
  the invariant suite covers "Tool call/result pairing integrity …
  Compression idempotency … VisibilityHorizon leaves no orphaned calls".
  Verified: the SlidingWindow idempotency test passes, but Summary
  accumulation is not covered (V03).
- `echo-state/src/compression/horizon.rs:1-26` claims a three-layer model
  where transient tool traces are compacted to symbolic summaries. Verified
  current.
- `echo-core/src/tokenizer.rs:34-47` claims `HeuristicTokenizer` is
  "recommended for mixed Chinese/English". Verified current; CJK tests pass.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. The `ContextCompressor` trait, all six concrete compressors (SlidingWindow, Summary, IncrementalSummary, Hybrid, VisibilityHorizon, Adaptive), the `ContextManager` orchestration, `sanitize_tool_call_pairing`, `verifier`, and `CompressionCheckpoint` are generic agent-framework capabilities any `echo-agent` consumer needs. They correctly live in `echo_core` (trait + types + tokenizer) and `echo_state` (implementations + orchestration). |
| EKO product policy | None at this layer. The compressors take pure framework inputs (`messages`, `token_limit`, `keep_recent`, `focus_instructions`). The EKO YAML (`AppConfig`) is the consumer: it selects `compress_strategy = "summary"` (default at `src/config.rs:482`) and wires an LLM client via `apply_compressor` (`src/config.rs:186-243`). |
| Adapter boundary | `AppConfig::apply_compressor` (`src/config.rs:186-243`) is the application-side adapter that turns the YAML strategy string into a concrete compressor. It is thin: it constructs `SummaryCompressor` / `HybridCompressor` / `SlidingWindowCompressor` and calls `ContextManager::set_compressor`. No product policy leaks into the compressors. |
| Duplicate search | Searched names: `ContextCompressor`, `impl ContextCompressor`, `CompressionInput`, `CompressionOutput`, `CompressionCheckpoint`, `SlidingWindowCompressor`, `SummaryCompressor`, `IncrementalSummaryCompressor`, `HybridCompressor`, `VisibilityHorizonCompressor`, `AdaptiveCompressor`, `sanitize_tool_call_pairing`, `split_protected`, `merge_protected`, `reinject_canonical_context`, `verify_compression`, `MemoryPromoter`, `StructuredSummary`, `safe_truncate`, `tune_for_model`. Result: one canonical definition per concept; six `impl ContextCompressor` in `echo_state`, zero in `echo-agent-cli`. `MemoryPromoter` is a callback trait, not a parallel compressor. |
| Migration deletion | No deletion recommended by this task. All six compressors are legitimate framework menu options per AGENTS.md ("通用框架提供多个 … 是正常的框架设计"). The findings below are wiring/state-management defects, not redundant implementations. |

## Current Path

Verified compression call graph at commit `9b0e0fa`:

```text
ReactAgent turn
   │
   ↓
ContextManager::prepare(current_query)                     [mod.rs:1243-1539]
   ├─ Snapshot original_messages                            [:1245]
   │
   ├─ [optional] VisibilityHorizon pre-pass                  [:1251-1293]
   │     compact_horizon → strip tool_calls + summary msg   [horizon.rs:129-193]
   │     evicted → MemoryPromoter (if configured)
   │
   ├─ estimated_tokens = Σ tokenizer.count_tokens(text)      [:1295]  ← uses self.tokenizer (Calibrated)
   ├─ effective_limit / needs_compression                    [:1299-1315]
   │     budget.allocate(0, 0, estimated_tokens)           ← phantom reservations (F-CTX-01-P2-01)
   │
   ├─ if needs_compression && compressor:
   │     split_protected → (compressible, protected)         [:1336]
   │     compressor.compress(CompressionInput {
   │         messages: compressible,
   │         token_limit: effective_limit,
   │         current_query,           ← plumbed but UNUSED by all compressors (P2-03)
   │         focus_instructions: None,
   │     })                                                   [:1338-1345]
   │       │
   │       ↓ (compressor uses its OWN HeuristicTokenizer, NOT self.tokenizer — P2-02)
   │       ↓ (Summary: partitions by role, summarizes old, appends system msg — P2-01)
   │     merge_protected(compressed, protected)               [:1353]
   │     evicted → MemoryPromoter                             [:1358-1367]
   │     ── fallback on primary error: SlidingWindow(40)      [:1407-1441]
   │
   ├─ sanitize_tool_call_pairing (ALWAYS)                    [:1448-1459]
   │     orphaned result → removed
   │     all calls orphaned → tool_calls cleared
   │     some calls orphaned → placeholder result inserted
   │
   ├─ [if summary produced] verify_compression               [:1464-1523]
   │     P0 fail → SlidingWindow(40) fallback from original + re-sanitize
   │
   └─ reinject_canonical_context (if configured)              [:1528-1530]
         system prompt / rules / skills restored with dedup
```

Key invariants verified by this graph:

- **Single sanitize choke point.** `sanitize_tool_call_pairing` runs
  unconditionally at `:1451`, after the verifier fallback at `:1494`, and
  inside `promote_and_sanitize` (`:848`) used by all `force_compress*`
  paths. No compression path bypasses it.
- **Protected survival.** `split_protected` / `merge_protected` /
  `is_protected` / projection envelope + `reinject_canonical_context`
  together guarantee protected content survives every pass (confirmed by
  F-CTX-01 V02-01; re-verified here).
- **Horizon is pair-aware.** `VisibilityHorizonCompressor` is the only
  compressor that maintains tool-pair integrity within its own logic
  (strips `tool_calls` from assistant AND drains results).
- **Fallback chain.** Primary compressor error → SlidingWindow(40)
  (`:1407`). Verifier P0 fail → SlidingWindow(40) from original
  (`:1482`). Both re-sanitize. The original buffer is restored only if
  BOTH fail (`:1433`).

The graph also exposes four defects (see Findings): summary-message
accumulation across cycles, tokenizer divergence between ContextManager
and compressors, unused `current_query`, and a minor horizon CJK
inconsistency.

## Findings

### F-CMP-01-P2-01: Summary system messages accumulate across repeated compression cycles

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-state/src/compression/compressor/summary.rs:292-297,
    346-348` — `SummaryCompressor::compress` partitions by
    `role == Role::System`, so the previous summary (a `Message::system(...)`
    from `:347`) lands in `system_msgs` and is preserved verbatim. The new
    summary is appended: `messages.push(Message::system(final_summary))`.
  - `echo-agent/echo-state/src/compression/compressor/summary.rs:604-686`
    — `IncrementalSummaryCompressor` has the same output assembly
    (`system_msgs` includes the previous summary; new summary appended).
    The `Mutex<Option<StructuredSummary>>` field-level merge prevents
    information *drift* but does not prevent message *accumulation*.
  - `echo-agent/echo-state/src/compression/mod.rs:1353, 1418, 1492` —
    `merge_protected` reinserts protected messages by position; it does not
    touch the system region.
  - `echo-agent/echo-state/src/compression/mod.rs:877-929` —
    `reinject_canonical_context` deduplicates only canonical context
    (project rules, active skills) by exact text match (`:914-921`). It
    does not match `[对话历史摘要]` envelopes.
  - `echo-agent/echo-state/src/compression/mod.rs:402-446` — `apply_hard_cap`
    (the only message-count cap) explicitly skips system messages
    (`:437`). The system region is never capped.
- Reachability: every `prepare()` call on a `ContextManager` configured
  with `SummaryCompressor` or `IncrementalSummaryCompressor` (the YAML
  default is `compress_strategy = "summary"` at `src/config.rs:482`). Once
  the token threshold is crossed, compression triggers every turn, adding
  one summary system message per cycle.
- Expected invariant: repeated compression should bound the context size;
  metadata produced by compression itself should not grow without limit.
- Observed behavior: after N compression cycles, the system region contains
  N `[对话历史摘要]` system messages. At ~500–2000 tokens per structured
  summary, 20 cycles add 10K–40K tokens — partially defeating compression's
  purpose and growing without bound.
- Impact: capability degradation on the core chat path under the YAML
  default strategy. A long session (the exact scenario compression exists
  for) gradually re-fills the window with accumulated summaries. Compounds
  with F-CTX-01-P2-01 (phantom budget) and F-CTX-01-P2-02 (protected not
  deducted): the real request body is `summaries + protected + recent`,
  none of which are bounded.
- Root cause: the Summary compressor appends a new system message but
  never removes or replaces the previous one. The partition-by-role design
  (preserve all system messages) was intended to protect the system prompt,
  but it inadvertently protects summary messages too. No dedup mechanism
  exists for `[对话历史摘要]` envelopes.
- Direction: before appending the new summary, remove any existing
  summary system messages from `system_msgs` (match by the
  `[对话历史摘要]` prefix or by a dedicated marker). Concretely, in
  `SummaryCompressor::compress` and `IncrementalSummaryCompressor::compress`:
  ```rust
  system_msgs.retain(|m| {
      !m.content.as_text_ref().is_some_and(|t| t.starts_with("[对话历史摘要]"))
  });
  ```
  Alternatively, have `ContextManager` own summary lifecycle: tag summary
  messages with a dedicated `MessageMetadata` flag and replace-on-write
  like replaceable protected markers. The retain approach is the smaller
  blast radius.
- Regression validation: add a test that runs `SummaryCompressor` twice
  through `ContextManager::prepare()` (with a mock LLM) and asserts the
  system region contains exactly one `[对话历史摘要]` message. Add a test
  that after 5 compression cycles, `messages.iter().filter(summary).count()
  == 1`.
- Validation reports: [V03-01](../validations/F-CMP-01/V03-01.md).

### F-CMP-01-P2-02: Compressors hard-code HeuristicTokenizer, diverging from the ContextManager's configured (Calibrated)Tokenizer

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-state/src/compression/compressor/sliding_window.rs:32`
    — `let tokenizer = HeuristicTokenizer;`
  - `echo-agent/echo-state/src/compression/compressor/summary.rs:284, 596`
    — `let tokenizer = HeuristicTokenizer;`
  - `echo-agent/echo-state/src/compression/compressor/hybrid.rs:40, 166-167`
    — `tokenizer: HeuristicTokenizer` struct field, default in builder.
  - `echo-agent/echo-state/src/compression/horizon.rs:115-116, 122` —
    `tokenizer: Box<dyn Tokenizer>` initialised to `Box::new(HeuristicTokenizer)`.
  - `echo-agent/echo-state/src/compression/levels.rs:134, 143` —
    `tokenizer: HeuristicTokenizer` struct field.
  - `echo-agent/echo-state/src/compression/mod.rs:1295, 1541-1547` —
    `ContextManager::estimate_tokens` uses `&*self.tokenizer` (the
    configured tokenizer, typically `CalibratedTokenizer` per
    `src/agent/react/mod.rs:333-335`).
  - `echo-agent/echo-state/src/compression/levels.rs:107-114` —
    `tune_for_model` sets thresholds as percentages of the real context
    window, but comparisons at `:186, 207, 216, 226, 301, 316` use the
    uncalibrated heuristic count.
  - `echo-agent/echo-state/src/compression/compressor/hybrid.rs:66-83` —
    short-circuit compares uncalibrated `current_tokens` against
    `token_limit` (derived from the calibrated path).
- Reachability: every compression pass where the ContextManager's tokenizer
  is calibrated (the default on the ReactAgent path). Affects Adaptive
  threshold triggering (when levels fire), Hybrid short-circuit (whether
  later stages run), and checkpoint `token_before`/`token_after`
  (observability).
- Expected invariant: the same text yields consistent token estimates
  across the decision-to-compress path and the compressor internals, so
  thresholds and short-circuits fire at the intended real-token boundaries.
- Observed behavior: the ContextManager may believe tokens are at 130K
  (calibrated) while the Adaptive compressor's heuristic reports 100K,
  causing L1 (set at 120K) to not fire when it should. Conversely, if
  calibration lowers the estimate, levels fire too early.
- Impact: incorrect escalation timing for Adaptive; unnecessary or
  insufficient compression for Hybrid short-circuit; inconsistent
  observability between `/context` and compression checkpoints. Not a
  crash, but a correctness gap on the compression-decision path.
- Root cause: compressors were written before the pluggable
  `CalibratedTokenizer` was wired into `ContextManager`, and were not
  retrofitted. The `CompressionInput` struct does not carry a tokenizer
  reference.
- Direction: add `tokenizer: Arc<dyn Tokenizer>` to `CompressionInput`
  (or pass it via a separate field), populated by `ContextManager::prepare`
  from `self.tokenizer`. Each compressor uses `input.tokenizer` instead of
  its hard-coded `HeuristicTokenizer`. This is a trait-level change but
  backward-compatible (additive field with a default). Alternatively,
  give each compressor an `Option<Arc<dyn Tokenizer>>` field (default
  `HeuristicTokenizer`) and have `set_compressor` inject the
  ContextManager's tokenizer — smaller blast radius, no trait change.
- Regression validation: add a test where a `CalibratedTokenizer` with
  factor 2.0 is installed, fill messages to `0.6 * window`, run
  `AdaptiveCompressor`, and assert L1 fires (it would not with the
  uncalibrated heuristic). Add a test that checkpoint `token_after` matches
  `ContextManager::token_estimate()` within rounding.
- Validation reports: [V04-01](../validations/F-CMP-01/V04-01.md).

### F-CMP-01-P2-03: `current_query` is plumbed through CompressionInput but unused by all compressors — no query-aware eviction protection

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/compression.rs:45-46` —
    `CompressionInput.current_query` docstring: "Current user query — used
    to protect active task context from eviction".
  - `echo-agent/echo-state/src/compression/mod.rs:1239` — `prepare()`
    docstring: "`current_query` is a reserved field; pass `None`."
  - `echo-agent/echo-state/src/compression/mod.rs:1342, 1412, 1486` —
    `prepare` passes `current_query` into `CompressionInput`.
  - `echo-agent/echo-state/src/compression/compressor/sliding_window.rs`
    — `current_query` not referenced anywhere in the file.
  - `echo-agent/echo-state/src/compression/compressor/summary.rs` — uses
    `focus_instructions` (`:323`), not `current_query`.
  - `echo-agent/echo-state/src/compression/compressor/hybrid.rs:118` —
    `current_query` used only as `.with_focus(focus_instructions.or(current_query))`
    for checkpoint observability, not for eviction decisions.
  - `echo-agent/echo-state/src/compression/horizon.rs`,
    `echo-agent/echo-state/src/compression/levels.rs` — `current_query`
    not referenced.
- Reachability: every `prepare()` call passes `current_query` through; no
  compressor consumes it for eviction protection.
- Expected invariant: messages relevant to the active user query should be
  prioritised for retention (or summarisation with focus) over older
  topically-unrelated messages.
- Observed behavior: the active query survives only because SlidingWindow
  and Summary both keep the last `N` messages (the query is the tail).
  Earlier turns relevant to the active task (e.g. "refactor auth module"
  stated 5 turns ago) have no query-aware priority and fall into the
  evicted/summarized portion. The verifier's `check_last_query_presence`
  (`verifier.rs:134-190`) checks keyword presence at 50% threshold but
  only when a summary was produced, and only for keywords >3 bytes long.
- Impact: query-relevant context is lost under compression even when the
  framework has the information to protect it. The API exists
  (`CompressionInput.current_query`) but is a no-op — misleading to any
  consumer that relies on the docstring.
- Root cause: the field was added with the right intent but never wired
  into eviction logic. The `focus_instructions` path partially covers this
  for Summary (LLM is told what to focus on), but SlidingWindow/Horizon/
  Adaptive have no focus mechanism.
- Direction: either (a) wire `current_query` into at least the Summary
  family (pass it as additional focus if `focus_instructions` is None), or
  (b) if query-aware eviction is not planned, rename the field and update
  the docstring to reflect that it is observability-only. Option (a) is
  more useful; the Summary path already has the machinery
  (`focus_instructions` → prompt injection). Option (b) is honest about
  current behavior. Either way, the API contract and the implementation
  should converge.
- Regression validation: add a test where `current_query` mentions a
  specific topic, compress with `SummaryCompressor`, and assert the topic
  appears in the generated summary (requires a mock LLM).
- Validation reports: [V04-01](../validations/F-CMP-01/V04-01.md).

### F-CMP-01-P3-01: Horizon compact-summary length check uses byte length, inconsistent with char-weighted tokenizer for CJK content

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-state/src/compression/horizon.rs:295` —
    `let max_chars = self.config.compact_max_tokens * 4;` followed by
    `if summary.len() > max_chars` where `summary.len()` is `str::len()`
    (bytes).
  - `echo-agent/echo-core/src/tokenizer.rs:50-58` — `HeuristicTokenizer`
    weights CJK chars at 2 (not 3 bytes / 4 = 0.75 tokens; rather ~0.5
    tokens per CJK char).
- Reachability: every Horizon compaction of a tool group whose summary
    text contains CJK content.
- Expected invariant (AGENTS.md Rust constraint §1 spirit): length checks
    that decide semantic behavior should use char counts, not byte counts,
    for consistency with the char-weighted tokenizer.
- Observed behavior: a CJK summary of 50 tokens is ~150 bytes;
    `max_chars = 50 * 4 = 200`; `150 < 200` so the check passes, but the
    real heuristic-token cost is ~25 (not 50). The fallback branch
    (`:297-306`) produces an even shorter summary. Net effect: CJK
    summaries may be over-truncated relative to their token budget. No
    panic (comparison only; the fallback does not slice by `max_chars`).
- Impact: minor. CJK Horizon summaries are slightly more aggressively
    compacted than intended. No panic, no data loss.
- Root cause: casual `len()` use for a size comparison; the tokenizer is
    available (`self.tokenizer`) but not used for this check.
- Direction: replace `summary.len() > max_chars` with
    `self.tokenizer.count_tokens(&summary) > self.config.compact_max_tokens`.
    One-line fix.
- Regression validation: add a CJK-summary test to `horizon::tests`.
- Validation reports: [V04-01](../validations/F-CMP-01/V04-01.md).

### F-CMP-01-P3-02: L1 Fold inserts a user message inside tool-result sequences, producing non-contiguous tool results after sanitize

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-state/src/compression/levels.rs:392-398` — L1 Fold
    drains `start..start+to_remove` tool messages and inserts
    `Message::user("[L1 fold: ...]")` at `start`.
  - `echo-agent/echo-state/src/compression/mod.rs:1608-1629` —
    `sanitize_tool_call_pairing` flushes pending placeholders before a
    non-Tool message. For a post-fold sequence
    `[assistant(tc1,tc2,tc3), user(fold), tool(tc2), tool(tc3)]`, sanitize
    produces `[assistant, tool(placeholder tc1), user(fold), tool(tc2),
    tool(tc3)]` — tool results split by a user message.
- Reachability: only when `AdaptiveCompressor` with
  `l1_fold_consecutive_tools = true` (the default in
  `AdaptiveCompressionConfig`, `levels.rs:74`) fires on a multi-call tool
  group. Not the default ReactAgent compressor (SlidingWindow / Summary).
- Expected invariant: the OpenAI Chat Completions spec expects tool
  messages to immediately follow the assistant message that issued the
  calls. Some provider implementations may reject a user message between
  tool results.
- Observed behavior: after sanitize, tool results for the same assistant
  are split across a user fold message. Every call still has a result
  (protocol-valid for pairing), but contiguity is broken.
- Impact: low. Only affects Adaptive (non-default) with multi-call groups
  where the fold boundary lands mid-group. Whether this causes a real API
  rejection depends on the provider's strictness; OpenAI's documented
  requirement is that tool messages follow the assistant call, but
  enforcement varies. No impact on SlidingWindow/Summary/Horizon.
- Root cause: L1 Fold was designed to collapse consecutive tool messages
  without considering that the fold marker (a user message) would break
  tool-result contiguity for the surviving results.
- Direction: either (a) make the fold marker a `Message::tool_result` with
  a synthetic id (so it stays in the tool sequence), or (b) move the fold
  marker to after the kept tool results instead of before them, or (c)
  have `sanitize_tool_call_pairing` reorder so all tool results for one
  assistant are contiguous. Option (b) is the smallest change.
- Regression validation: add a test with a 3-call tool group, L1 fold with
  `keep=2`, and assert the output has no user message between tool results
  for the same assistant.
- Validation reports: [V02-01](../validations/F-CMP-01/V02-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Compressor/invariant matrix + duplicate search | yes | **passed** (1 trait, 6 impls, no duplicates; matrix documented) | [V01-01](../validations/F-CMP-01/V01-01.md) |
| V02 | Tool-pair preservation (sanitize safety net) | yes | **passed** (sanitize correct for all 3 orphan cases; 1 minor L1 Fold edge case → P3-02) | [V02-01](../validations/F-CMP-01/V02-01.md) |
| V03 | Repeated compression stability / convergence | yes | **failed** (Summary/IncrementalSummary accumulate system messages → P2-01; SlidingWindow/Horizon/Adaptive converge) | [V03-01](../validations/F-CMP-01/V03-01.md) |
| V04 | Multilingual/CJK + large schema + token estimation | yes | **failed** (tokenizer divergence → P2-02; current_query unused → P2-03; CJK safety PASS; horizon byte-length → P3-01) | [V04-01](../validations/F-CMP-01/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — `invariants.rs` and `horizon.rs` module docs make auditable claims) | done — see Historical Claim Status table below | — |

Targeted executable checks run as part of V01–V04:

| Command | Exit | Result |
|---|---:|---|
| `cargo test -p echo_state --lib compression:: --locked` | 0 | 69 passed, 0 failed |
| `cargo test -p echo_core --lib tokenizer:: --locked` | 0 | 12 passed, 0 failed |

No conditional feature/GUI/frontend matrix was run: this task touches only
framework compression code compiled under the default feature set.
F-FEAT-01 owns the full matrix.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `invariants.rs:1-19` — "1. Tool call/result pairing integrity … 8. Compression idempotency (re-compress doesn't lose more) … 10. VisibilityHorizon leaves no orphaned calls" | partial drift | Invariants 1, 10 verified current (V02). Invariant 8 (idempotency) holds for SlidingWindow but NOT for Summary/IncrementalSummary — summary accumulation means re-compress DOES add more (P2-01). The idempotency test only covers SlidingWindow. |
| `horizon.rs:1-26` — three-layer visibility model, transient tool traces compacted to symbolic summaries | current | V01/V02 confirm; `compact_horizon` correctly strips tool_calls + replaces results. |
| `summary.rs:138-146` — "压缩后的消息结构: [原有 system 消息] [system] [对话历史摘要] <-- 新插入 [最近 keep_recent 条]" | partial drift | The structure is correct for a single pass. The docstring does not mention that on the NEXT pass, the previous `[对话历史摘要]` survives in "[原有 system 消息]" and accumulates (P2-01). |
| `summary.rs:384-397` — IncrementalSummary "maintains the previous summary and only sends the previous summary + new messages … reduces LLM cost" | current (cost claim) | The Mutex state correctly prevents re-summarizing from scratch. But the accumulated system messages still grow the buffer (P2-01). The cost reduction is real; the token growth is the gap. |
| `hybrid.rs:11-18` — "Short-circuit: When enabled (default), the pipeline skips remaining stages once the estimated token count drops to or below `token_limit`" | current (behavior) but uses wrong tokenizer | The short-circuit fires as described, but compares uncalibrated heuristic tokens vs the calibrated limit (P2-02). |
| `levels.rs:88-96` — "Thresholds are set as percentages of the context window … tune_for_model" | partial drift | The tuning is correct, but the comparison uses the uncalibrated heuristic, so the percentages are applied to the wrong base (P2-02). |
| `echo-core/src/compression.rs:45-46` — "current_query — used to protect active task context from eviction" | stale | No compressor uses it (P2-03). |
| `echo-core/src/tokenizer.rs:34-47` — "HeuristicTokenizer … recommended for mixed Chinese/English" | current | V04 confirms CJK-safe counting, no panics. |
| `echo-state/src/compression/mod.rs:874-904` — canonical re-injection "keeps the history segment's byte positions stable, preserving cache breakpoints" | current (design intent) | F-CTX-01 V02 confirmed `sys_end` insertion + dedup. Cache hit-rate not measured here. |

## Coverage And Uncertainty

Inspected in full: all six compressor implementations, `ContextManager`
(key ranges above; full read of `:300-1548, 1560-1710, 1712-1830`),
`sanitize_tool_call_pairing`, `verifier`, `invariants` test suite,
`echo_core::compression` (trait + types + `StructuredSummary`), and the
`HeuristicTokenizer`/`CalibratedTokenizer` code.

Not inspected (out of scope or deferred):

- `MemoryPromoter` implementations (application-side) — F-MEM-01 owns the
  sink. Only the trait contract (`mod.rs:69-79`) and the call sites
  (`:1266, :1358-1367, :836-845`) were inspected.
- `pre_compaction_flush` (`run/context.rs`) — the pre-compression LLM
  flush that gates on `should_compress()`. Only `should_compress`
  (`mod.rs:468-479`) was inspected.
- Subagent context builder — F-SUB-01.

Environmental constraints:

- Two `cargo test` commands run (echo_state compression, echo_core
  tokenizer) — both green at `9b0e0fa`. No feature matrix, no frontend
  build, no GUI check (out of scope).
- The summary-accumulation defect (P2-01) was verified by static code
  trace (high confidence) rather than an executable test, because
  reproducing it requires a mock LLM client and multiple `prepare()`
  cycles. The trace is unambiguous: `summary.rs:346-348` appends without
  removal, and no dedup exists in the prepare path.

Uncertain claims:

- Whether the L1 Fold ordering issue (P3-02) causes a real API rejection
  depends on provider-specific enforcement of tool-result contiguity.
  Classified medium confidence / P3 because Adaptive is not the default
  and the scenario is narrow.
- The exact rate of summary accumulation in production depends on how
  often `needs_compression` fires per session, which depends on the
  model's window and the conversation pace. The direction (unbounded
  growth) is certain; the rate is estimate-only.

## Handoff

Conclusions downstream tasks may rely on:

1. **Tool-pair protocol validity is guaranteed.** `sanitize_tool_call_pairing`
   is the single, correct choke point. Every compression path (primary,
   fallback, verifier-fallback, force_compress*) runs it. Downstream tasks
   that reason about post-compression message validity (F-SUB-01 subagent
   context, F-LLM-01 request construction) can rely on the output being
   protocol-valid. The one caveat is L1 Fold ordering (P3-02, Adaptive
   only).
2. **Protected content survives compression.** Markers, projections,
   canonical context, and replaceable markers all survive (confirmed by
   F-CTX-01 V02 and re-verified here). The gap is token *accounting*
   (F-CTX-01-P2-02), not survival.
3. **SlidingWindow and Horizon converge under repeated compression.**
   Summary and IncrementalSummary do NOT (P2-01). Any downstream task
   that models long-session behavior should account for summary
   accumulation until P2-01 is fixed.
4. **CJK content is safe.** No UTF-8 panics anywhere in the compression
   path. Char-aware truncation (`safe_truncate`, `chars().take()`) is used
   throughout. The one inconsistency (P3-01) is a byte-length comparison,
   not a slice.
5. **The compressor menu is a legitimate framework API.** All six
   compressors are retained per AGENTS.md. No deletion recommended.

Reports they must read:

- This report (F-CMP-01) for the compressor matrix, sanitize correctness,
  and the four defects.
- `tasks/F-CTX-01.md` for the budget/selection invariants and the phantom
  reservation + protected-deduction defects that compound with P2-01.
- `validations/F-CMP-01/V01-01.md` through `V04-01.md` for per-claim
  evidence and the executable test results.

Conditions that make this report stale:

- Removal of old summary system messages before appending new ones in
  `SummaryCompressor` / `IncrementalSummaryCompressor` — resolves
  F-CMP-01-P2-01, requires re-running V03-01.
- Wiring the ContextManager's tokenizer into compressors (via
  `CompressionInput` or constructor injection) — resolves F-CMP-01-P2-02,
  requires re-running V04-01.
- Implementing `current_query`-aware eviction or renaming the field —
  resolves F-CMP-01-P2-03, requires re-running V04-01.
- Replacing `summary.len()` with `tokenizer.count_tokens()` in
  `horizon.rs:295` — resolves F-CMP-01-P3-01, requires re-running V04-01.
- Reordering L1 Fold output or making the fold marker a tool_result —
  resolves F-CMP-01-P3-02, requires re-running V02-01.

Follow-up task IDs (no implementation in this review task):

- A dedicated compression-fix task should land P2-01 (summary dedup) first
  — it is the highest-impact defect and the smallest fix (a `retain`
  before append). P2-02 (tokenizer plumbing) and P2-03 (current_query)
  are larger and can follow.
- **F-LLM-01/02/03** should consume the tokenizer-divergence conclusion
  (P2-02) when auditing request construction: checkpoint token counts
  differ from `/context` estimates today.
- A future cleanup task could add a `-D clippy::arithmetic_side_effects`
  gate (cross-referenced from F-CTX-01-P3-01) — the compression path has
  multiple `.sum()` sites that would be flagged.
