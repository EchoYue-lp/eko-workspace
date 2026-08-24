# F-CTX-01: Context selection and budget accounting

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Are canonical instructions, history, tools, attachments, memory, and reserved
output selected deterministically within model limits?

## Scope

- `echo-agent/src/context/mod.rs` (`ContextAssembler` + `ContextBudget`,
  full read), `src/context/selector.rs` (`ContextSelector`, full read).
- Tokenizers/budgets: `echo-core/src/tokenizer.rs` (full read),
  `echo-core/src/budget.rs` (full read), `echo-core/src/llm/capabilities.rs`
  (`infer_context_window` :197-217).
- Live context authority: `echo-state/src/compression/mod.rs` (`ContextManager`
  :311-1840 — push/apply_projections/protected/is_protected/split+merge
  protected/prepare/force_compress*/reinject_canonical_context/estimate_tokens/
  builder), `compressor/{sliding_window,summary,hybrid}.rs`,
  `horizon.rs`, `levels.rs` (`tune_for_model`), `verifier.rs`.
- React context assembly: `echo-agent/src/agent/react/mod.rs` `new_inner`
  :322-384 (ContextManager/budget/canonical wiring), `capabilities.rs`
  `add_skill` :563-612, `run/phases/{compact,think}.rs` (prepare/calibration/
  tool-schema budget), `run/stream_channel.rs` (final-only budget path :560-651,
  direct_answer_stream :362+, :453-454), `run/react_loop.rs`
  (`prepare_react_context` :508-591), `agent/config.rs` (DEFAULT_TOKEN_LIMIT,
  token_limit/budget config), `src/config.rs` (`resolve_context_window`/
  `to_agent_config`/`apply_compressor`), `agent/snapshot.rs` (tool-output
  spill/truncation :921-1049, tools_for_llm), `echo-core/src/compression.rs`
  (`CanonicalContext`, `to_reinjection_messages` :376-401).
- EKO wiring (reachability only): `echo-agent-cli/echo-agent-app-core/src/infra.rs`
  (:23, :215-219, :258-264), `agent_pool.rs` (:433-436),
  `model_config.rs` (:153-159), `project/prompt.rs` (PromptAssembler,
  full read), `tasks/task_runtime/compact_context.rs` (protected marker :156-160).
- Executed tests: echo_state `compression::` (69), echo_core
  `tokenizer`/`budget`/`compression`, echo_agent `context::`,
  echo-agent-app-core `project::prompt` — all green (V04).

## Out Of Scope

- Loop state machines, terminal ownership, streaming ordering → F-RCT-02/03
  (only the final-only budget branch and prepare call sites were anchored).
- Compressor summary quality / LLM-summary fidelity → F-CMP-01 (only
  protected-content survival mechanics audited here).
- Provider adapters and usage normalization → F-LLM-01..03 (cross-referenced
  for calibration input only).
- EKO user-input artifactization (Phase E) → A-INP-01.
- Tool execution semantics (timeouts, cancellation) → F-RCT-04, F-EXT-01.

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, layering, no-parallel-semantics),
  shared `README.md`, `REPORTING.md`, `TASKS.md` (F-CTX-01 card), templates.
- Dependency task reports read: zcode-ds `F-RCT-01` (complete),
  `F-LLM-01` (complete).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md`
  (M9 archive, Phase C, Phase E) — classified in Historical Claim Status.

## Layering Decision

- Generic mechanism (framework): `ContextManager` message-buffer/budget
  authority, `TokenBudget` percentage allocation, `Tokenizer` trait +
  `HeuristicTokenizer`/`CalibratedTokenizer`, `CanonicalContext` re-injection,
  projection envelopes, `infer_context_window` — correctly placed in
  `echo_state`/`echo_core`.
- EKO product policy (application): `PromptAssembler` module composition
  (`project/prompt.rs`), protected task-context marker registration
  (`compact_context.rs:159`), the hardcoded `DEFAULT_CONTEXT_WINDOW` in
  `infra.rs` and the `model_config.rs` GUI-view window inference.
- Adapter boundary: the framework builder's default `token_limit` (396K) and
  EKO's runtime construction both bypass the framework's own
  `infer_context_window` — a wiring gap at the framework/application boundary
  (finding P1-01).
- Duplicate search terms (both repositories): `ContextAssembler`,
  `ContextSelector`, `ContextManager`, `TokenBudget`, `TokenBudgetConfig`,
  `ContextBudget`, `HeuristicTokenizer`, `SimpleTokenizer`,
  `CalibratedTokenizer`, `TokenUsageTracker`, `estimate_tokens`,
  `count_tokens`, `add_protected_marker`, `protected_marker`,
  `PreModelContextProjector`, `ContextProjection`, `CanonicalContext`,
  `reinject_canonical_context`, `PromptAssembler`, `max_tool_output_tokens`,
  `infer_context_window`. Results (V01): one live authority per semantic
  (`ContextManager`/`TokenBudget`); one off-main-path parallel framework API
  (`ContextAssembler`/`ContextSelector` — P2-03); `PromptAssembler` is a
  different responsibility (static system-prompt composition) with a
  percentage-budget model overlapping `TokenBudget`; window inference is
  split across three consumers (P1-01).

## Current Path

Verified data flow (anchors in V02):

1. Construction: `ReactAgentBuilder::build`/`new_inner`
   (`react/mod.rs:322-384`) — `ContextManager::builder(config.token_limit)`
   with `CalibratedTokenizer(HeuristicTokenizer)`; `budget(...)` when
   `token_budget_config.enabled` (`build(config.token_limit)`); SlidingWindow
   compressor when `token_limit < usize::MAX` or budget enabled; canonical
   context with `system_prompt` + `project_rules`.
2. Turn: `run_core_loop` (shared by streaming `stream_channel.rs:307` and
   non-streaming `react_loop.rs:713`) → `run_compact`
   (`phases/compact.rs:55-60`) → `ContextManager::prepare(None)` — the single
   budget decision point — → `run_think` sends `messages` + tool schemas
   (`think.rs:89,395-414`).
3. `prepare`: horizon pre-pass → `estimate_tokens` (all messages, text only)
   → budget branch `allocate(0, 0, estimated_tokens)` (:1299-1315) → primary
   compressor with `effective_limit`, SlidingWindow fallback on failure,
   P0-verifier fallback, `reinject_canonical_context` (:1525-1530) → final
   messages sent to the LLM without a post-compression window re-check.
4. Protection: projections (envelope) are protected by `is_protected`;
   canonical re-injection restores the system prompt and appends
   `to_reinjection_messages` at `sys_end`; EKO additionally registers a
   replaceable marker for the task brief (`compact_context.rs:159`).
5. Tool output: `process_tool_output_for_call` (`snapshot.rs:921-1049`)
   spills to artifacts or head/tail truncates (UTF-8-safe, saturating).
6. Calibration: after each streaming call `think.rs:171-179` feeds
   `(messages-only estimate, provider pt)` into the shared tokenizer.

## Findings

### F-CTX-01-P1-01: Provider window mapping is bypassed by the builder default and the EKO runtime — small-window models can exceed their real context limit

- Priority: P1
- Confidence: medium (code facts high; the overrun requires a small-window
  model or heavy protected content, argued statically)
- Layer: adapter (framework/application wiring boundary)
- Evidence: `echo-core/src/llm/capabilities.rs:197-217` (`infer_context_window`
  — kimi k2.6/k2.7 = 256_000, claude/deepseek/qwen/glm = 1_000_000);
  `echo-agent/src/agent/config.rs:11-12,234` (`DEFAULT_TOKEN_LIMIT = 396_000`,
  `AgentConfig::new` default); `react/mod.rs:336-343` (ContextManager built
  with `config.token_limit`, budget built from `config.token_limit`);
  `src/config.rs:99-104,129-137` (framework YAML path resolves the inferred
  window — the only framework consumer); `echo-agent-cli/.../infra.rs:23,215-219,258-262`
  (EKO runtime uses hardcoded `DEFAULT_CONTEXT_WINDOW = 396_000`);
  `model_config.rs:153-159` (EKO infers windows only for the GUI model view);
  `agent_pool.rs:433-436` (applies `runtime.context_window` only when the user
  configured it explicitly).
- Reachability: every agent built through `ReactAgentBuilder` without an
  explicit `.token_limit(...)` (framework default) and every EKO agent built
  by `infra.rs` — i.e. the default runtime path — compresses/triggers on a
  396K window regardless of the model. For kimi k2.x (256K real window) the
  budget overestimates the window by ~55%; with output tokens and
  estimate/calibration slack, real usage can exceed the model limit and the
  provider rejects the request. For 1M-window models the 65%-of-396K ≈ 257K
  threshold compresses ~4x earlier than needed (premature summarization).
- Expected invariant: the effective context window equals the provider/model
  window (explicit override > inferred > documented default), on every
  construction path.
- Observed behavior: three divergent consumers — framework YAML path (infers),
  EKO GUI view (infers), builder/EKO runtime (hardcoded 396K). The model name
  is never consulted for window inference on the live path.
- Impact: context-length provider errors and failed turns on small-window
  models; premature history summarization (quality loss) on large-window
  models; `AgentConfig::new` doc "default token limit 396000, matches
  TokenBudget default" (config.rs:11-12) cements a model-independent default.
- Root cause: `AgentConfig.token_limit` was introduced as a plain usize with a
  constant default before the model-window inference existed; the inference
  was later added for the YAML config path and the GUI view but never wired
  into the builder default or EKO runtime construction.
- Direction: derive `token_limit`/budget window from `ModelProfile`/model name
  at construction (`new_inner`), keeping explicit config as override;
  remove the hardcoded `DEFAULT_CONTEXT_WINDOW` from `infra.rs` and reuse
  `effective_context_window` (model_config.rs) or the framework resolver for
  the runtime path; add a regression test building a kimi-k2.x agent with
  defaults and asserting the budget window is 256K.
- Regression validation: unit test asserting `ContextManager.token_limit`
  equals the inferred window for a known model when unset; EKO-level test
  asserting `token_limit` for a kimi model < 396K; keep
  `unknown_model_uses_396k_default_context_window` (config.rs:858-861) only
  for unknown models.
- Validation reports: [V01](../validations/F-CTX-01/V01-01.md),
  [V02](../validations/F-CTX-01/V02-01.md), [V03](../validations/F-CTX-01/V03-01.md)

### F-CTX-01-P2-01: The budget enforces only the conversation bucket — system, tool, and output allocations are never checked

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-state/src/compression/mod.rs:1299-1315` (`prepare` calls
  `budget.allocate(0, 0, estimated_tokens)` — system/tool sizes hardcoded 0;
  comments "system prompt tokens already counted in messages", "tool defs not
  in messages"); `echo-core/src/budget.rs:90-123` (`allocate` computes
  `system_fits`/`tool_defs_fit`/`output_fits` and `usage_pct` — none consumed
  by `prepare`); `budget.rs:181-184` (`needs_compression` =
  `conversation_excess > 0` only); `think.rs:395-414` (tool schema stats are
  logged, never budgeted); `max_tokens` (reserved output) is passed to the
  provider (`think.rs:299,315`) but never subtracted from the window.
- Reachability: every run with a configured budget — the conversation bucket
  (65% of window) is the only enforced limit; `usage_pct`/`ok()`/`system_fits`
  are dead output for the compression decision. If system prompt + tool
  schemas + max_tokens exceed their nominal 35% share (large AGENTS.md +
  many skill/MCP tools + large max_tokens), the total sent context can exceed
  the window with no compression signal.
- Expected invariant: the assembled context plus reserved output stays within
  the model window; the budget's system/tool/output allocations are honored or
  the doc says they are advisory.
- Observed behavior: the budget model documents four allocations but enforces
  one; nothing compares total (system+tools+history) or
  (total+reserved output) against the window.
- Impact: request rejection or silent truncation by the provider when
  protected/static content is large; the "within model limits" guarantee of
  the task question holds only under small system/tool/output profiles.
- Root cause: `prepare` passes zeroed system/tool sizes and consumes only
  `needs_compression()`; the richer `TokenAllocation` API was built but not
  wired into the decision.
- Direction: pass real system/tool schema sizes into `allocate` (count the
  system region and the `tools_for_llm` schema via `HeuristicTokenizer`) and
  subtract `max_tokens` from the usable window; or explicitly document that
  only the conversation bucket is enforced; add a fixture where system+tools
  alone exceed 35% of the window and assert a compression decision or an
  explicit over-limit signal.
- Regression validation: unit test on `prepare` with a large system prompt +
  large tool schema + max_tokens asserting `needs_compression` fires when
  total_used + max_tokens > window; keep existing budget tests green (V04).
- Validation reports: [V02](../validations/F-CTX-01/V02-01.md),
  [V03](../validations/F-CTX-01/V03-01.md)

### F-CTX-01-P2-02: Canonical re-injection truncates project rules to 2000 chars, never re-injects skill injections, and re-inserts a stale system prompt after `add_skill`

- Priority: P2
- Confidence: high (static chain fully verified; not executed dynamically)
- Layer: framework
- Evidence: `echo-core/src/compression.rs:376-401` (`to_reinjection_messages`
  truncates rules with `rules.chars().take(2000)` :385 — silent loss of rule
  text beyond 2000 chars in the re-injected copy; `skill_injections` never
  emitted though the struct doc at echo-state `mod.rs:351-354` promises
  "system prompt, rules, and skill injections" restoration); field
  `skill_injections` is only ever initialized to `Vec::new()`
  (`react/mod.rs:379`; `compression/mod.rs:2142,2193`) — dead field;
  `capabilities.rs:587-597` (`add_skill` appends to `config.system_prompt`
  and replaces the context system message via `update_system`, but never
  updates the canonical context), contrast `react/mod.rs:939`
  (`set_working_dir` refreshes `canonical.system_prompt`);
  `echo-state/src/compression/mod.rs:884-893` (re-injection inserts the
  canonical system prompt at index 0 when no system message text matches —
  after `add_skill` the current system message differs, so the stale
  pre-skill prompt is inserted first).
- Reachability: any agent that installs a code skill after construction
  (`add_skill`/`add_skills` — the plugin/skill activation path) followed by a
  compression-triggering `prepare`; and any agent with project rules longer
  than 2000 chars after the first compression (rules are re-injected both
  inside the restored system prompt and as the truncated canonical message —
  see F-RCT-01-P2-02 for the duplication arm).
- Expected invariant: canonical instructions (system prompt, rules, skill
  injections) survive compression verbatim and match the current agent state.
- Observed behavior: (a) the canonical rules copy is truncated to 2000 chars
  with no marker; (b) skill prompt injections are absent from canonical
  re-injection entirely; (c) after `add_skill`, the first compression inserts
  a stale full-length system prompt (without the skill injection) at position
  0 — the model sees two system prompts with the outdated one first.
- Impact: models can lose or mis-weight instruction content after compression;
  skills that rely on system-prompt injection degrade after the first
  compaction; the truncated rules copy can read as the complete rules.
- Root cause: `CanonicalContext` was designed as a "restore everything"
  authority but `to_reinjection_messages` implements only rules (truncated)
  and active skill names; the skill/plugin path mutates the system prompt
  without notifying the canonical source (the refresh exists only on the
  working-dir path).
- Direction: (1) remove the silent 2000-char cap — re-inject the full rules
  text or bound it via the token budget with an explicit truncation notice;
  (2) either populate and re-inject `skill_injections` or delete the dead
  field and fix the doc; (3) make `add_skill` refresh
  `canonical.system_prompt` (mirror `set_working_dir` at `react/mod.rs:939`);
  (4) align with F-RCT-01-P2-02's single-authority direction (rules embedded
  in the system prompt vs canonical field — pick one).
- Regression validation: ContextManager-level test — skill injected via
  `add_skill`, force-compress, assert exactly one system prompt equal to the
  post-skill prompt and rules text present verbatim; a >2000-char rules
  fixture asserting no silent truncation in the re-injected copy.
- Validation reports: [V02](../validations/F-CTX-01/V02-01.md),
  [V03](../validations/F-CTX-01/V03-01.md)

### F-CTX-01-P2-03: `ContextAssembler`/`ContextSelector` is a parallel framework context-selection authority with a byte-based token estimate and dead budget fields

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/context/mod.rs:109-117` (doc: the default
  ReactAgent path does NOT use `ContextAssembler`); `assemble` budget
  arithmetic uses `m.content.as_text().map(|c| c.len() / 4)` — **bytes** per 4
  — at :208 (history) and :234 (tool results), while the comment at :200
  claims "estimate tokens ≈ chars/4" (for CJK, bytes/4 = 3x the claimed
  chars/4, so history/tool-result retention is cut ~3x early on mixed
  Chinese/English content); memory truncation treats `memory_max` (a token
  budget, default 5_000) as a character count (:186-190); `ContextBudget`
  fields `total_tokens` and `user_reserve` are never read in `assemble`
  (:35-65 — set-only dead fields); only consumers are
  `examples/demo65_context_assembler.rs`, `examples/demo66_context_selector.rs`,
  and the deprecated `AgentRunner` docstring (`src/runner.rs:7`).
- Reachability: not registered on any live runtime path; public framework API
  with examples and docs, so not deletable without a consumer decision, but
  it is a second message-list assembly + budget implementation with a
  different budget model than `ContextManager`/`TokenBudget` (per AGENTS.md
  "严禁平行实现同一语义").
- Expected invariant: one authoritative context-assembly/budget semantic; any
  off-path duplicate either delegates to the authority or is removed.
- Observed behavior: two framework assembly APIs with divergent token
  estimates (bytes/4 vs char-weight/4), divergent budget structs
  (`ContextBudget` vs `TokenBudget`), and dead budget fields.
- Impact: framework consumers following `examples/demo65` get different
  truncation behavior than the live path and a budget struct whose half the
  fields do nothing; maintenance surface for a semantic that has one
  authoritative implementation.
- Root cause: `ContextAssembler` predates/parallels `ContextManager`'s budget
  support and was never converged; the byte/char discrepancy and dead fields
  were never revisited.
- Direction: either (a) delete `ContextAssembler`/`ContextSelector` and the
  demos + `runner.rs` doc references (with X-BND-01 confirming no external
  consumer), or (b) reimplement `assemble` on top of the `Tokenizer` trait
  (HeuristicTokenizer) + `TokenBudget` and remove the dead fields; fix the
  bytes/4 estimate either way.
- Regression validation: grep for `ContextAssembler`/`ContextBudget`/
  `ContextSelector` returns only intended references after the change; `cargo
  check -p echo_agent --examples` stays green.
- Validation reports: [V01](../validations/F-CTX-01/V01-01.md),
  [V03](../validations/F-CTX-01/V03-01.md)

### F-CTX-01-P3-01: Multimodal attachments are invisible to token estimation

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-state/src/compression/mod.rs:1541-1547` (`estimate_tokens`
  uses `m.content.as_text()`) and `:978-987` (`token_breakdown` same);
  `echo-core/src/llm/types.rs:92-117` (`as_text` returns `None` for
  `Parts` containing only non-text parts); `think.rs:81-86` (LlmCall estimate
  uses `text_content()` — same exclusion).
- Reachability: any agent turn carrying image/audio parts — the messages are
  sent to the provider but never counted toward `token_limit`/budget.
- Expected invariant: every context-consuming message contributes to the
  budget estimate.
- Observed behavior: image-only `Parts` messages contribute zero tokens; a
  long multimodal history can exceed the window while the estimate stays low.
- Impact: budget undercount on multimodal turns (each image is typically
  1K+ tokens); combined with P2-01/P1-01 this can push real usage over the
  window without a compression signal.
- Root cause: `as_text` was designed as the text projection and the budget
  estimator never gained a parts-aware counter.
- Direction: add a `count_tokens` path for `ContentPart::Image`/other parts
  (fixed per-image token allowance or provider-specific) and use it in
  `estimate_tokens`/`token_breakdown`; add a multimodal fixture asserting
  image messages inflate the estimate.
- Regression validation: unit test pushing a `Parts` image message and
  asserting `token_estimate()` > 0 and that compression fires earlier than
  without the image.
- Validation reports: [V02](../validations/F-CTX-01/V02-01.md),
  [V03](../validations/F-CTX-01/V03-01.md)

### F-CTX-01-P3-02: `direct_answer_stream` hardcodes zero protected-context counts in the LlmCall trace

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `src/agent/react/run/stream_channel.rs:453-454`
  (`protected_context_tokens: 0, protected_message_count: 0` in
  `direct_answer_stream`'s `RunEvent::LlmCall`) vs `phases/think.rs:91-97`
  (real values read from the ContextManager lock).
- Reachability: every `direct_answer_stream` invocation (the IntentRouter
  direct-answer path, `stream_channel.rs:190-220`).
- Expected invariant: the trace reports the same protected-context facts on
  all LLM-call paths.
- Observed behavior: direct-answer runs report zero protected tokens/messages
  regardless of the actual buffer.
- Impact: run diagnostics understate protected context on direct-answer
  turns (same path-dependence family as F-LLM-01-P2-02 usage undercount).
- Root cause: the direct-answer path emits the event without locking the
  context manager (the values were stubbed).
- Direction: read `protected_message_count`/`protected_token_estimate` from
  the context in `direct_answer_stream` (it already has the context Arc for
  `push_runtime_context_note` calls); add a trace-level assertion in the
  direct-answer tests.
- Regression validation: extend the direct-answer streaming test to assert
  nonzero protected counts when a protected message exists.
- Validation reports: [V03](../validations/F-CTX-01/V03-01.md)

### F-CTX-01-P3-03: `CalibratedTokenizer` calibration input is asymmetric — messages-only estimate vs prompt tokens that include tool schemas and cache

- Priority: P3
- Confidence: medium (bias direction verified; magnitude estimated)
- Layer: framework
- Evidence: `src/agent/react/run/phases/think.rs:171-179` — `estimated`
  counts `messages` text only (:172-176) while `pt` =
  `usage.effective_prompt_tokens()` includes tool definitions and cached
  prefix components (F-LLM-01 normalization); the EMA factor
  (`echo-core/src/tokenizer.rs:104-203`) therefore converges to
  (messages+extras)/messages > 1, inflating all later estimates and firing
  compression earlier than needed; calibration uses the already-factor-scaled
  estimate, feeding the bias back.
- Reachability: every streaming turn with provider-reported usage; the shared
  tokenizer drives `prepare`'s threshold for all subsequent turns of the
  agent.
- Expected invariant: the calibration factor reflects the ratio of real token
  counting to the estimator over the same input scope.
- Observed behavior: the factor systematically overestimates (tool schemas
  are sizable in EKO's full tool set), shifting the compression threshold down
  over time.
- Impact: slightly premature summarization on long sessions (quality/cost
  side); not a correctness failure.
- Root cause: the estimator scope (messages) and the reference scope
  (full prompt) were not aligned when calibration was added.
- Direction: compute `estimated` over messages + the request tool schema
  (reuse `tools_for_request` output) before calibrating, or calibrate only
  against `usage.prompt_tokens - tool_schema_estimate`; document the scope.
- Regression validation: unit test simulating tool-schema-bearing usage and
  asserting the steady-state factor stays near 1.0 for pure-message inputs.
- Validation reports: [V02](../validations/F-CTX-01/V02-01.md),
  [V03](../validations/F-CTX-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (context selection / budgets / protected content) | yes | passed | [V01-01](../validations/F-CTX-01/V01-01.md) |
| V02 | Registration and reachability trace (ContextManager, tokenizer, budget, canonical) | yes | passed | [V02-01](../validations/F-CTX-01/V02-01.md) |
| V03 | Budget arithmetic/overflow, UTF-8, window mapping, protected-content, multilingual/large-schema inspection | yes | passed | [V03-01](../validations/F-CTX-01/V03-01.md) |
| V04 | Targeted tests: echo_state compression::, echo_core tokenizer/budget/compression, echo_agent context::, app-core project::prompt | yes | passed (exit 0 each; 69/12/1/10/3/3 passed) | [V04-01](../validations/F-CTX-01/V04-01.md) |
| V05 | Historical-document drift (MASTER-PLAN M9 / Phase C) | conditional | passed | [V05-01](../validations/F-CTX-01/V05-01.md) |

All required validations executed; every command has a known exit code; no
validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN M9: provider usage/cache/context breakdown/protected context/compression in one durable run diagnostic | current | `trace/mod.rs:273-299`; `think.rs:91-97,149-165`; `echo-state/src/compression/mod.rs:106-115`; [V05-01](../validations/F-CTX-01/V05-01.md) |
| MASTER-PLAN M9: provider without usage → estimate only, not accurate totals | current | `usage_reported` semantics `think.rs:147`; F-LLM-01 [V03] |
| MASTER-PLAN Phase C: "限制 protected message 数量和体积" | stale | no size/count limit on protected content; only lossy 2000-char rules cap in `to_reinjection_messages` (`echo-core/src/compression.rs:385`); P2-02 |
| MASTER-PLAN Phase C acceptance: "protected token 超预算有明确诊断" | current (observability only) | `protected_token_estimate`/`protected_message_count` in trace + EKO diagnostics; no budget-triggered signal (P2-01) |

## Coverage And Uncertainty

- All conclusions are static except the test runs (V04); no dynamic run
  exercised compression after `add_skill` or a small-window model end to end,
  so P1-01's overrun and P2-02's stale-prompt insertion are argued from code
  chains, not executed.
- The `CalibratedTokenizer` bias magnitude (P3-03) is estimated, not measured;
  real-provider calibration behavior is environment-dependent.
- `horizon.rs`/`levels.rs` were inspected at the budget-relevant sites only;
  their compression-fidelity semantics belong to F-CMP-01.
- EKO `PromptAssembler` module budgets were reviewed at the assembly level;
  the app-level `/context` command and run-inspector rendering are A-* scope.
- F-RCT-01-P2-02 (rules duplicated after compression; stale rules after
  workspace switch) is cross-referenced; this report adds the 2000-char
  truncation and skill-staleness arms of the same authority problem.
- No P0 findings: no data-loss/corruption, secret exposure, or core-path
  breakage was established; P1-01 is the closest (provider rejection on
  small-window models) but its trigger depends on model/config, hence medium
  confidence.

## Handoff

- Downstream tasks may rely on: the single live budget decision point
  (`ContextManager::prepare`, V02); overflow-safe and UTF-8-safe arithmetic
  everywhere audited (V03); green unit suites (V04); the classification of
  window inference across three consumers (P1-01); the bucket-enforcement gap
  (P2-01); the canonical truncation/staleness arms (P2-02); the parallel
  `ContextAssembler` authority (P2-03).
- F-CMP-01: treat canonical re-injection as a lossy path for rules > 2000
  chars and for skill injections (P2-02) when verifying compression fidelity;
  verify horizon/adaptive levels against the budget-bucket gap (P2-01).
- F-RCT-05 (resume): canonical state after resume must not re-introduce the
  stale pre-skill system prompt (P2-02 arm c).
- A-MEM-01 / X-MEM-01: EKO instruction/memory projections ride on the same
  canonical/projection machinery — the P2-02 fixes change what survives
  compression; A-* tasks should re-check the EKO runtime window wiring
  (P1-01) for kimi/small-window models.
- X-BND-01: confirm deletion of `ContextAssembler`/`ContextSelector` and the
  demos has no external consumer (P2-03 direction); record the window-mapping
  authority decision (P1-01).
- Reports to read: this report + V01-01..V05-01; F-RCT-01 (P2-02 duplication
  arm) and F-LLM-01 (usage authority, direct-answer path-dependence).
- Stale triggers: any change to `echo-state/src/compression/mod.rs`
  (prepare/allocate/protected/canonical), `echo-core/src/budget.rs`,
  `echo-core/src/tokenizer.rs`, `echo-core/src/compression.rs`
  (`to_reinjection_messages`), `agent/config.rs` defaults,
  `react/mod.rs` `new_inner`/`set_working_dir`, `capabilities.rs` `add_skill`,
  `think.rs` calibration, or `infra.rs`/`agent_pool.rs`/`model_config.rs`
  window wiring invalidates the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): F-CMP-01,
  F-RCT-05, A-MEM-01, X-MEM-01, X-BND-01, Q-DOC-01 (budget doc alignment for
  P2-01/P2-03).
