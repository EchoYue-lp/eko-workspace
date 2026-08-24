# F-CMP-01: Compression correctness

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Do compressors preserve protocol pairs, instructions, active tasks, recent
evidence, and recovery facts under repeated compression?

## Scope

- `echo-state/src/compression/` full read: `mod.rs` (ContextManager,
  `prepare`/`force_compress*`/protected markers/projections/
  `sanitize_tool_call_pairing`/`reinject_canonical_context`, 2584 lines),
  `compressor/{sliding_window,summary,hybrid}.rs`, `horizon.rs`, `levels.rs`
  (AdaptiveCompressor + `tune_for_model`), `verifier.rs`, `invariants.rs`.
- `echo-core/src/compression.rs` (`ContextCompressor`,
  `CompressionCheckpoint`, `CanonicalContext`/`to_reinjection_messages`,
  `StructuredSummary`/`merge_with`), `echo-core/src/tokenizer.rs`
  (HeuristicTokenizer CJK weighting, spot-checked).
- Root adapters: `echo-agent/src/compression.rs` (pure re-export),
  `src/config.rs` `apply_compressor`/`has_compressor` :186-283,
  `src/agent/react/mod.rs` `new_inner` :322-384, `set_working_dir` :939,
  `src/agent/react/capabilities.rs` `add_skill` :563-612, `force_compress*`
  :185-270, `src/agent/react/run/phases/compact.rs` (run_compact,
  pre-compaction flush), `src/agent/react/run/context.rs:676-800`
  (`pre_compaction_flush`), `src/memory_promoter.rs`, `src/agent/react/builder.rs`
  `visibility_horizon` :1045-1055.
- EKO reachability: `echo-agent-cli/echo-agent-app-core/src/infra.rs:205-300`,
  `runtime.rs:114`, `agent_pool.rs:948`,
  `tasks/task_runtime/compact_context.rs` (protected task-brief marker +
  runtime-recovery capsule projection), `src/cli/cmd_impls/context.rs`
  (`/compress`, `/compact`).
- Executed tests: `echo_state compression::` (69) and `echo_core compression`
  (1) — green (V04).

## Out Of Scope

- Budget allocation percentages / system-tool-output bucket enforcement
  (F-CTX-01-P2-01) and window inference (F-CTX-01-P1-01) — budget-layer
  concerns, cross-referenced only.
- LLM summary fidelity / prompt quality of the summary templates (a model
  behavior question, not a code invariant).
- `ContextAssembler`/`ContextSelector` duplicate (F-CTX-01-P2-03).
- Store durability (F-MEM-01), snapshot/resume (F-RCT-05), EKO memory
  projections (A-MEM-01).

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, no-parallel-semantics, layering),
  shared `README.md`, `REPORTING.md`, `TASKS.md` (F-CMP-01 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-CTX-01` (complete), `F-MEM-01`
  (complete).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md` (Phase C
  :271-277, L3 :895, M9 :379), `echo-agent-cli/docs/configuration.md` :61-63,
  `echo-agent/docs/zh/04-compression.md`, `echo-agent/docs/{en,zh}/28-config-reference.md`
  — classified in Historical Claim Status.

## Layering Decision

- Generic mechanism (framework): `ContextManager`, all six compressors,
  sanitizer, canonical reinjection, projections/protected markers, verifier,
  `MemoryPromoter` trait + `StoreMemoryPromoter`, memory-promotion dedup —
  correctly placed in `echo_core`/`echo_state`/`echo-agent` root. V01
  duplicate search (terms: `ContextCompressor`, `SummaryCompressor`,
  `SlidingWindowCompressor`, `HybridCompressor`, `AdaptiveCompressor`,
  `IncrementalSummaryCompressor`, `VisibilityHorizonCompressor`,
  `sanitize_tool_call_pairing`, `ToolPairFix`, `CanonicalContext`,
  `to_reinjection_messages`, `current_query`, `MemoryPromoter`, `compress_*`)
  found exactly one authority per semantic; the `#[compressor]` macro
  (echo-macros) is an extension point with no production user; no EKO-side
  reimplementation.
- EKO product policy (application): strategy selection via `apply_compressor`,
  the task-brief protected marker and runtime-recovery capsule
  (`compact_context.rs`), `/compress` `/compact` manual commands, and the
  documented config knobs.
- Adapter boundary: `apply_compressor` is a thin strategy dispatcher (no
  compression logic of its own); `compact_context.rs` projections ride the
  framework envelope API.
- Dormant framework APIs (registered, no live consumer, kept per framework
  deletion rules): `IncrementalSummaryCompressor` (only re-exported),
  `VisibilityHorizonCompressor` (only via unused builder method),
  `AdaptiveCompressor` (live when strategy "adaptive" is configured).

## Current Path

Verified data flow (anchors in V02):

1. EKO constructs every agent with `token_limit` (396K default, infra.rs:258-262)
   → `new_inner` installs `SlidingWindowCompressor::new(40)` (react/mod.rs:346-353)
   → `apply_compressor` overrides with the strategy (default `"summary"` →
   `SummaryCompressor::new(llm, 20)`, config.rs:203-224; `"sliding"`,
   `"hybrid"`, `"adaptive"` also supported).
2. Each turn: ReAct loop → `run_compact` (compact.rs:20-107) → pre-compaction
   flush (L3 durable facts, gated by `should_compress`) → checkpoint save →
   pre-model projections (EKO runtime-recovery capsule) →
   `prepare(None)` (the single compression decision point).
3. `prepare` (mod.rs:1243-1539): horizon pre-pass (if configured) → token
   estimate → budget/simple over-limit check → `split_protected` →
   compressor (fallback SlidingWindow(40) on error) → `merge_protected` →
   memory promotion → `sanitize_tool_call_pairing` → verifier (summary
   compressors only; P0-fail → SlidingWindow(40) over the ORIGINAL buffer) →
   `reinject_canonical_context` → return messages (no post-compression
   token re-check).
4. Manual: `/compress` `/compact` → `force_compress_with_focus_and_hooks`/
   `force_compress_context` → `force_compress*` (same sanitize + canonical
   chain, hooks fired).

## Findings

### F-CMP-01-P1-01: Message-count windows do not bound tokens — repeated compression stalls above the limit with no post-compression check

- Priority: P1
- Confidence: high (code logic unconditional; trigger depends on message sizes)
- Layer: framework
- Evidence: `echo-state/src/compression/compressor/sliding_window.rs:48-66`
  (early return when `conv_msgs.len() <= window_size` — no token check, so a
  few large messages over the limit compress to nothing); `compressor/summary.rs:299-317`
  (same early-return pattern); `compression/mod.rs:1317-1345` (`prepare`
  passes `effective_limit` to the compressor but never compares the output
  against it); `mod.rs:1531-1538` (returns messages directly, no re-check);
  `verifier.rs:100-120` (the only runtime token check is `after <= before`,
  never `<= token_limit` — the doc comment admits "A stronger 'within target
  limit' check requires model context info not available here");
  `react/mod.rs:346-353` (default window 40 "roughly 20 turns" with a comment
  claiming it "fits within the token limit" — no such guarantee exists);
  `config.rs:202` (`compress_window.max(2)` — EKO default 20 messages).
- Reachability: every agent on the live path (EKO default `summary` window 20
  or framework default sliding window 40) once history contains a few large
  messages (big user pastes, non-spilled tool outputs); `prepare` is called
  before every LLM call (compact.rs:60).
- Expected invariant: after an auto-compression, the messages sent to the LLM
  fit within `token_limit`; repeated compression converges to a bounded state
  (doc claim: "不因上下文过长而崩溃", docs/zh/04-compression.md:9).
- Observed behavior: compressors cut by message count, not tokens; when
  `conv_msgs.len() <= window` they return unchanged even at 10x the limit;
  `prepare` returns the over-limit list every turn; nothing re-checks the
  result.
- Impact: provider context-length rejections (turn failures) and unbounded
  retry pressure on long sessions with large messages — the exact scenario
  compression exists for; the doc's "long conversations don't crash" guarantee
  does not hold.
- Root cause: the compressor trait contract has no "must reduce below
  `token_limit`" obligation; windows are count-based and the pipeline has no
  iterative or fallback escalation when the count window is already small.
- Direction: make the window token-aware (drop oldest messages until
  `estimate <= limit` with a system/protected floor) or add a post-compression
  check that escalates (e.g., truncate/spill oversized messages, then
  emergency L5-style cut) when the count window cannot fit the limit; record
  an explicit over-limit signal in the checkpoint when the invariant cannot be
  met.
- Regression validation: unit test — token_limit small, 5 messages each > 1/3
  of the limit, assert `prepare` result tokens <= limit (or an explicit
  `over_limit` marker); repeated-`prepare` test asserting no stall (each call
  either reduces or reports the bound).
- Validation reports: [V02-01](../validations/F-CMP-01/V02-01.md),
  [V03-01](../validations/F-CMP-01/V03-01.md), [V04-01](../validations/F-CMP-01/V04-01.md)

### F-CMP-01-P1-02: Summary-based compressors accumulate one immortal summary system message per compression — the system region grows without bound under repeated compression

- Priority: P1
- Confidence: high (pure control-flow arithmetic; verified over successive
  passes)
- Layer: framework
- Evidence: `compressor/summary.rs:346-348` (output =
  `system_msgs + [new system summary] + to_keep`); `summary.rs:292-296` and
  `sliding_window.rs:41-45` (all `Role::System` messages are partition-kept,
  never re-summarized, never expired); `summary.rs:319-321` (each pass
  re-summarizes only the messages that fell out of the window — the previous
  summaries remain as-is); no count/size cap on summaries anywhere in
  `mod.rs`; `StructuredSummary::merge_with` (echo-core/src/compression.rs:270-342)
  exists for consolidation but plain `SummaryCompressor` never merges.
- Reachability: the EKO default strategy `"summary"` (V02) — every long
  session that compresses N times ends with N summary messages (~0.5-2K tokens
  each) in the system region; `AdaptiveCompressor` L4 (levels.rs:578-583) and
  L3 (levels.rs:469-472) have the same accumulate-a-marker-per-pass shape.
- Expected invariant: repeated compression keeps total context bounded
  (MASTER-PLAN Phase C and doc claims); each fact survives in at most one
  summary.
- Observed behavior: round k appends summary_k; summaries 1..k-1 are never
  merged or removed; combined with P1-01, the system region itself can
  eventually exceed the window, after which every `prepare` re-fires the
  summary LLM call without converging (cost + rejection loop).
- Impact: unbounded context growth on the EKO default path; repeated LLM
  summarization cost that cannot converge; models receive multiple overlapping
  summaries (instruction/recovery-fact weight dilution).
- Root cause: summaries are modeled as immortal system messages; the
  compressor has no cross-pass memory (only `IncrementalSummaryCompressor`
  keeps state, and it is not wired anywhere — dormant).
- Direction: replace-or-merge — keep a single "running summary" system message
  updated per pass (store the previous summary text in the compressor state or
  find it in the buffer by marker), or cap the number of summaries and merge
  the oldest; wire `IncrementalSummaryCompressor` or a stateful summary stage
  as the default.
- Regression validation: N-pass test — push turns, force-compress N times,
  assert the number of summary messages stays bounded (<= 1) and system-region
  tokens grow sub-linearly; assert each fact appears in at most one summary.
- Validation reports: [V02-01](../validations/F-CMP-01/V02-01.md),
  [V03-01](../validations/F-CMP-01/V03-01.md)

### F-CMP-01-P1-03: AdaptiveCompressor L1 fold inserts a user-role message between an assistant's tool_calls and its kept tool results — the framework's own pairing-contiguity invariant is broken

- Priority: P1
- Confidence: medium (code chain unambiguous; provider rejection is external
  behavior, argued from the framework's own declared contract)
- Layer: framework
- Evidence: `echo-state/src/compression/levels.rs:392-396` (fold summary is
  `Message::user("[L1 fold: …]")` inserted at `start` — i.e., between the
  assistant message that still carries `tool_calls` and the kept `tool`
  results); `levels.rs:364-403` (run scan keeps the LAST `keep` tool messages
  after the insert point); `compression/mod.rs:1550-1557` (the framework's own
  declared invariant: assistant tool_calls "must be followed by tool messages
  for each tool_call_id" and every tool message must have a preceding
  assistant with matching id); `mod.rs:1451-1459` (sanitizer runs after
  compression but only adds placeholders / removes orphans — never reorders);
  `config.rs:256-271` (strategy `"adaptive"` is a documented, registered
  option).
- Reachability: any agent configured with `compress_strategy: "adaptive"`
  (framework YAML or EKO config) under tool-heavy conversations once tokens
  exceed `l1_snip_threshold_tokens` (80K default, or 60% of window after
  `tune_for_model`) — exactly the high-token scenario the strategy targets.
- Expected invariant: after compression + sanitize, every `tool` message
  directly follows the assistant message with the matching `tool_calls`
  (framework-declared, mod.rs:1552-1557).
- Observed behavior: sequence becomes
  `[assistant(tool_calls=[a..e]), user("[L1 fold…]"), tool(a), tool(b), placeholder(c..e)]`
  — the fold summary (user role) sits between the assistant tool_calls and the
  tool results; the sanitizer cannot repair the order; OpenAI-style providers
  reject `tool` messages that do not directly follow their assistant
  tool_calls message.
- Impact: provider 400 errors on the adaptive strategy in tool-dense
  sessions; at minimum a user-role message injected into the middle of a tool
  round-trip changes prompt semantics even on providers that tolerate it
  (Anthropic merges consecutive user turns).
- Root cause: the fold placeholder was authored as a user message inserted at
  the START of the remaining run instead of at the END (or as a
  tool/assistant-role message); the sanitizer was never given reorder
  capability.
- Direction: insert the fold summary AFTER the kept results (tool messages
  then remain contiguous with their assistant), or strip the folded
  tool_call_ids from the assistant message so no dangling reference remains;
  add a sanitize-level regression test for contiguous tool sequences after
  fold.
- Regression validation: unit test — assistant with N tool_calls + N tool
  results, `l1_fold_keep_latest=2`, run compress then `sanitize_tool_call_pairing`,
  assert every tool message's immediate predecessor is the assistant message
  with the matching tool_call_id.
- Validation reports: [V03-01](../validations/F-CMP-01/V03-01.md),
  [V04-01](../validations/F-CMP-01/V04-01.md)

### F-CMP-01-P2-01: Verifier P0-fail fallback discards the LLM summary and window-cuts without promoting the newly evicted band — narrow fact-loss window on heuristic false-fail

- Priority: P2
- Confidence: medium (loss chain is deterministic once the heuristic
  false-fails; false-fail probability estimated)
- Layer: framework
- Evidence: `compression/mod.rs:1464-1515` (P0-fail → re-compress ORIGINAL
  messages with `SlidingWindowCompressor::new(40)`, replace the checkpoint,
  and do NOT call the memory promoter for the fallback's newly evicted set —
  promotion only happened for the first pass at :1358-1367);
  `verifier.rs:134-190` (last-query P0 check: 50% of whitespace words longer
  than 3 bytes must appear in compressed+summary — stop-word-heavy queries
  and summary paraphrasing can false-fail); `verifier.rs:193-244` (file-path
  check on the last 6 tool messages); `verifier.rs:40-96` (verification runs
  only when `checkpoint.summary.is_some()` — count-window compressions are
  never runtime-verified despite `invariants.rs` being advertised as the
  compression "safety net").
- Reachability: every summary-strategy compression (EKO default) is subject
  to the P0 heuristics; the fallback fires on any false-fail.
- Expected invariant: a failed verification must not lose facts the summary
  pass had preserved, and every eviction band must be offered to memory
  promotion.
- Observed behavior: on P0 fail the LLM summary is discarded, and messages in
  the [last 40 .. last 20] band of the original buffer are evicted by the
  window without promotion — their facts vanish from both context and memory.
- Impact: silent knowledge loss (recovery facts) on heuristic false-fails;
  the "safety net" itself degrades summarization to a hard cut.
- Root cause: the fallback treats the summary as disposable and re-runs the
  pipeline without the promotion step; the P0 heuristics are keyword-based
  with no notion of "already preserved via retained messages".
- Direction: keep the summary message alongside the window-cut fallback (or
  re-run promotion on the fallback's evicted set); make the P0 checks
  lenient/configurable (skip last-query check when the query is in the
  retained window); add `tokens <= effective_limit` as a real P0 with the
  limit available in the checkpoint.
- Regression validation: fixture where the last user query is short/stop-word
  heavy and the summary omits it — assert no fallback or that fallback output
  preserves the summary; assert fallback evictions reach the promoter.
- Validation reports: [V03-01](../validations/F-CMP-01/V03-01.md)

### F-CMP-01-P3-01: VisibilityHorizon pre-pass leaves the message buffer empty on compressor error — the defensive comment contradicts the behavior (latent wipe)

- Priority: P3
- Confidence: high (code fact; error currently unreachable)
- Layer: framework
- Evidence: `compression/mod.rs:1251-1293` — `std::mem::take(&mut self.messages)`
  moves the buffer into the horizon input; the `Err(e)` branch only logs and
  comments "Don't fail prepare — horizon is best-effort" / "continuing with
  original messages", but `self.messages` is already empty; the main-path
  compressor error handler restores `original_messages` (:1430-1440) — the
  horizon branch has no such restore.
- Reachability: latent — `VisibilityHorizonCompressor::compress` (horizon.rs:333-368)
  is infallible today, and the horizon is only installed via the unused
  builder method; any future horizon implementation returning `Err` (or a
  panic escape) wipes the conversation buffer on the next `prepare`.
- Expected invariant: a failed best-effort pre-pass leaves the buffer intact.
- Observed behavior: buffer empty; `estimated_tokens = 0`; the LLM call
  proceeds with an empty message list.
- Impact: latent total context loss; misleading defensive code invites the
  bug.
- Root cause: `mem::take` without an error-path restore.
- Direction: clone or restore `self.messages = original` in the error branch
  (mirror the main-path restore at :1433).
- Regression validation: inject a failing horizon compressor (test impl
  returning `Err`) and assert `prepare` keeps the original messages.
- Validation reports: [V03-01](../validations/F-CMP-01/V03-01.md)

### F-CMP-01-P3-02: EKO documentation claims `token_limit: 0` disables compression; compression cannot actually be disabled

- Priority: P3
- Confidence: high
- Layer: application (doc) / adapter boundary
- Evidence: `echo-agent-cli/docs/configuration.md:61` ("token_limit: 0 …
  0 = 禁用"); `echo-agent-cli/echo-agent-app-core/src/infra.rs:258-262`
  (0 → `DEFAULT_CONTEXT_WINDOW` = 396K passed to the builder);
  `echo-agent/src/agent/react/mod.rs:346-353` (any `token_limit < usize::MAX`
  installs SlidingWindow(40)); `src/config.rs:482-483` (serde default
  `compress_strategy = "summary"`, `compress_window = 20`) with
  `apply_compressor` :195-224 (any non-empty strategy installs a compressor).
- Reachability: every EKO agent built from the documented sample config.
- Expected invariant: the documented knob does what the doc says (disable
  compression), or the doc is corrected.
- Observed behavior: with `token_limit: 0` compression still triggers at 396K
  (SlidingWindow 40 and/or the strategy compressor).
- Impact: users who believe they disabled compression get silent history
  eviction; users who want a lower threshold must understand the interaction.
- Root cause: the "0 = disabled" semantics predates the strategy-driven
  compressor installation and was never reconciled.
- Direction: either honor `token_limit: 0` + empty strategy as fully disabled
  (skip both `new_inner` default and `apply_compressor`) or fix the doc;
  also list "adaptive" in the documented strategy set (configuration.md:62
  lists only sliding/summary/hybrid).
- Regression validation: doc/behavior alignment test — config with
  `token_limit: 0` and empty strategy asserts `has_compressor() == false`.
- Validation reports: [V05-01](../validations/F-CMP-01/V05-01.md)

### F-CMP-01-P3-03: Byte-based token budgets applied to CJK content in horizon/L1-snip estimates, and a production byte-slice on the checkpoint id

- Priority: P3
- Confidence: high (no panic; over-truncation magnitude is content-dependent)
- Layer: framework
- Evidence: `horizon.rs:295-296` (`max_chars = compact_max_tokens * 4` applied
  against `summary.len()` — bytes, so CJK summaries hit the fallback at ~1/3
  the intended size); `levels.rs:348-352` (L1 snip `char_limit = max_tokens *
  4` passed as a BYTE limit to `safe_truncate` — CJK tool outputs truncated to
  ~1/3 of the intended token budget); `verifier.rs:146` (keyword filter
  `w.len() > 3` is byte-based — CJK queries produce one giant "keyword");
  `hybrid.rs:95` (`&cp.checkpoint_id[..8]` — a byte slice; provably safe
  because `checkpoint_id` is an ASCII UUID v4, echo-core/src/compression.rs:122,
  but it violates the repository's no-byte-slicing rule).
- Reachability: horizon/L1 only on the dormant/adaptive paths; hybrid on the
  `"hybrid"` strategy; verifier keywords on the EKO default summary path.
- Expected invariant: token budgets expressed in characters or tokenizer
  counts, never bytes, for CJK-safe content; no byte slicing on any string.
- Observed behavior: CJK content over-truncated relative to the intended
  token budget; style-rule violation in production code.
- Impact: slightly over-aggressive truncation of Chinese tool output/summaries
  (quality), no data loss or panic.
- Root cause: byte-vs-char confusion in estimate helpers predating the
  CJK-aware tokenizer.
- Direction: use `HeuristicTokenizer.count_tokens` for the budgets or
  char-count-based limits; replace `[..8]` with a char-safe prefix (or use the
  full id).
- Regression validation: CJK fixture — L1 snip with a Chinese tool output
  asserts retained tokens near `l1_max_output_tokens`; horizon summary with a
  Chinese tool list stays within the char budget.
- Validation reports: [V03-01](../validations/F-CMP-01/V03-01.md)

### Independent cross-verification of F-CTX-01-P2-02 (no new finding — re-traced, not copied)

The three arms of F-CTX-01-P2-02 were re-traced from source in this task and
confirmed: (a) rules truncation `chars().take(2000)` — echo-core/src/compression.rs:385;
(b) `skill_injections` dead field — only `Vec::new()` initializers
(react/mod.rs:379; mod.rs:2142,2193) and never emitted in
`to_reinjection_messages` (:376-401); (c) stale system-prompt re-insertion
after `add_skill` — `add_skill` (capabilities.rs:587-597) calls `update_system`
but never `set_canonical_system_prompt`, while `set_working_dir`
(react/mod.rs:939) does; `reinject_canonical_context` (mod.rs:884-893) then
inserts the pre-skill prompt at index 0 on the first compression. Evidence
anchors here are independent of F-CTX-01's; the finding ID and ownership stay
with F-CTX-01-P2-02, and this report's P1-01/P1-02 and the canonical chain are
cross-consistent with it.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (compressors / canonical / tool-pair / current_query) | yes | passed | [V01-01](../validations/F-CMP-01/V01-01.md) |
| V02 | Registration and reachability trace (EKO default summary; opt-in adaptive/hybrid; dormant incremental/horizon; single `prepare` decision point) | yes | passed | [V02-01](../validations/F-CMP-01/V02-01.md) |
| V03 | Invariant/edge inspection (tool-pair preservation incl. L1 fold; repeated-compression stability; UTF-8 truncation; overflow; multilingual; canonical; protected content) | yes | passed (findings P1-01..P3-03) | [V03-01](../validations/F-CMP-01/V03-01.md) |
| V04 | `cargo test -p echo_state --lib --locked compression::` | yes | passed, exit 0 (69 passed) | [V04-01](../validations/F-CMP-01/V04-01.md) |
| V04 | `cargo test -p echo_core --lib --locked compression` | yes | passed, exit 0 (1 passed) | [V04-02](../validations/F-CMP-01/V04-02.md) |
| V05 | Historical-document drift (MASTER-PLAN Phase C/L3/M9, EKO configuration.md, 04-compression.md) | conditional | passed (drift rows → findings P3-02, stale rows) | [V05-01](../validations/F-CMP-01/V05-01.md) |

All required validations executed; every command has a known exit code; no
validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN Phase C: 压缩后校验 tool_call/tool_result 配对 | current | `sanitize_tool_call_pairing` on every prepare (mod.rs:1451); sanitizer tests green (V04-01); L1-fold exception → P1-03 |
| MASTER-PLAN Phase C: 校验 run/task identity、已完成副作用、pending interaction | stale | runtime verifier checks only tokens/summary/last-query/file-paths/TODO/errors/preferences (verifier.rs:40-96), and only when a summary exists |
| MASTER-PLAN Phase C: 限制 protected message 数量和体积 | stale | no size/count cap; only the 2000-char rules cap in `to_reinjection_messages` (echo-core/src/compression.rs:385) |
| MASTER-PLAN:895: L3 两条并行写入路径可能重复写 | fixed (mitigated) | both writers use content-hash key `l3_{hash}` (memory_promoter.rs; context.rs:790-794 `locate` skip) — dedup implemented |
| MASTER-PLAN M9: auto/manual compression 写入 durable timeline | current | `RunEvent::ContextCompression` (compact.rs:74-84), checkpoint fields, EKO /context (context.rs:222-240) |
| EKO configuration.md:61: token_limit 0 = 禁用 | regressed | infra.rs:258-262 (0 → 396K) + react/mod.rs:346-353 + config.rs:482 default strategy → compression always installed (P3-02) |
| EKO configuration.md:62: strategies sliding/summary/hybrid | stale | code also supports "adaptive" (config.rs:256-271) |
| 04-compression.md: 长对话支持…不因上下文过长而崩溃 | regressed | count-based windows + summary accumulation (P1-01/P1-02) |

## Coverage And Uncertainty

- All findings except the test runs (V04) are static. No dynamic run exercised
  a repeated-compression session end to end or the adaptive fold against a
  real provider, so P1-01/P1-02 magnitudes and P1-03 provider rejection are
  argued from code + framework-declared invariants, not executed.
- `IncrementalSummaryCompressor`'s internal `previous_summary` state was
  inspected for the accumulation analysis; its resume/stale-state edges were
  not deeply reviewed because the compressor has no production consumer
  (dormant — noted for X-BND-01).
- The `ContextAssembler`/`ContextSelector` duplicate and budget-bucket
  enforcement remain owned by F-CTX-01 (P2-03, P2-01); this report adds the
  compressor-side and stability arms of the same over-limit family.
- F-RCT-01-P2-02 (rules duplicated after compression on other paths) was not
  re-audited; canonical behavior here is scoped to the reinjection path.
- Summary LLM output fidelity (fact retention quality) is a model-behavior
  question, explicitly out of scope; `StructuredSummary::merge_with` growth
  semantics were noted in P1-02 evidence but not measured.

## Handoff

- Downstream tasks may rely on: the single-authority compression stack (V01);
  the EKO default path = SummaryCompressor(20) at 396K with the
  sanitize+canonical chain on every prepare (V02); the stability failures
  (P1-01 count-window stall, P1-02 summary accumulation), the adaptive fold
  contiguity break (P1-03), the verifier fallback loss window (P2-01), the
  latent horizon wipe (P3-01), the EKO doc/config mismatch (P3-02), and the
  byte-vs-char estimate notes (P3-03); the independent confirmation of
  F-CTX-01-P2-02's three arms.
- Reports to read: this report + V01-01..V05-01; F-CTX-01 (P1-01 window
  mapping, P2-01 budget buckets, P2-02 canonical, P3-01 multimodal
  estimation); F-MEM-01 (store durability — memory promotion targets).
- Conditions that make this report stale: any change to
  `echo-state/src/compression/` (compressors, `prepare`/`force_compress*`,
  sanitizer, verifier, horizon, levels), `echo-core/src/compression.rs`
  (`to_reinjection_messages`, checkpoint), `react/mod.rs` `new_inner`/
  `set_working_dir`, `capabilities.rs` `add_skill`, `config.rs`
  `apply_compressor`/defaults, `infra.rs` token_limit resolution, or
  `echo-agent-cli/docs/configuration.md` compression rows.
- Follow-up task IDs (fixes are not implemented in this review):
  F-RCT-05 (resume must not reintroduce stale canonical prompt), A-MEM-01 /
  X-MEM-01 (EKO projections ride the same machinery — P1-01/P1-02 change what
  survives), X-BND-01 (dormant compressors: IncrementalSummary/
  VisibilityHorizon deletion-or-wiring decision; P3-02 doc alignment),
  Q-FLT-01 (fault injection for the verifier fallback and L1 fold scenarios),
  Q-TST-01 (P1-01..P2-01 regression-test coverage gaps), Q-DOC-01 (config doc
  drift).
