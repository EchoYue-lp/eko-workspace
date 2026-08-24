# F-CTX-01: Context selection and budget accounting

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Are canonical instructions, history, tools, attachments, memory, and reserved
output selected deterministically within model limits?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-core/src/budget.rs` (310 lines) — `TokenBudget`,
  `TokenBudgetConfig`, `TokenAllocation`, percentage-based allocation.
- `echo-agent/echo-state/src/compression/mod.rs` (2584 lines, key ranges
  `:311-355`, `:380-446`, `:468-510`, `:647-793`, `:858-929`,
  `:969-1019`, `:1243-1547`, `:1660-1708`, `:1712-1830`) — `ContextManager`,
  `push` / `apply_hard_cap`, `should_compress` / `token_estimate`,
  `protected_markers` / `replaceable_protected_markers`, `split_protected` /
  `merge_protected`, `reinject_canonical_context`, `token_breakdown`,
  `prepare`, `estimate_tokens`, `sanitize_tool_call_pairing`,
  `ContextManagerBuilder`.
- `echo-agent/echo-core/src/compression.rs` (`:345-402`) — `CanonicalContext`
  and `to_reinjection_messages`.
- `echo-agent/echo-core/src/tokenizer.rs` (510 lines) — `Tokenizer` trait,
  `HeuristicTokenizer`, `SimpleTokenizer`, `CalibratedTokenizer`,
  `TokenUsageTracker`.
- `echo-agent/echo-core/src/llm/capabilities.rs` (`:195-217, 219-385,
  489-571`) — `infer_context_window`, `ModelProfile`, `ModelProfileResolver`,
  `CachePolicy`.
- `echo-agent/echo-core/src/llm/types.rs` (`:43-147, 525-540`) —
  `MessageContent`, `Role`, `ChatCompletionRequest.tools` out-of-band field.
- `echo-agent/src/config.rs` (`:99-284, 855-862`) — `resolve_context_window`,
  `AppConfig::to_agent_config`, `apply_compressor`.
- `echo-agent/src/context/mod.rs` (259 lines) — parallel `ContextAssembler`
  and `ContextBudget` building block.
- `echo-agent/src/context/selector.rs` (141 lines) — file-relevance
  `ContextSelector` (off the model-context path; light scan only).
- `echo-agent/src/agent/react/mod.rs` (`:325-382`) — `ContextManager` builder
  wiring inside `new_inner` (budget + tokenizer + canonical context).
- `echo-agent/src/agent/react/run/context.rs` (799 lines) — runtime context
  preparation (`prepare_stream_context`,
  `prepare_stream_context_with_message`, `pre_compaction_flush`,
  `runtime_context_note`, memory recall injection).

Cross-checks (lighter): `echo-agent/src/tokenizer.rs` (re-export shim),
`echo-agent/echo-core/src/llm/capabilities.rs` test block.

## Out Of Scope

Deferred to named task IDs:

- Compression *strategy* correctness (Summary / IncrementalSummary / Hybrid /
  Adaptive / SlidingWindow algorithms, summary verification, P0 fallback) →
  **F-MEM-01** and a compression-strategy-focused task.
- Visibility-horizon compaction algorithm → a horizon-focused task.
- Memory recall ranking / `MemoryRecaller` composite-score correctness →
  **F-MEM-01**.
- Subagent context builder (`src/agent/subagent/context*.rs`) — separate
  subagent task.
- `ContextSelector` file-scoring heuristics (`src/context/selector.rs`) —
  not on the model-context path; deferred to a codebase-indexing task.
- LLM client construction and provider request serialization beyond
  `ChatCompletionRequest.tools` → **F-LLM-01/02/03**.
- Construction-path invariants (single tool registry, canonical prompt
  assembly) → already covered by **F-RCT-01** (read as dependency).

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  system reminder — especially the Rust UTF-8 / no-panic constraints, the
  framework-vs-application layering rule, the "first check if it already
  exists" rule, and the local-assistant threat model).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-RCT-01.md` (in full). F-RCT-01
  establishes the construction path that wires `TokenBudget`,
  `CalibratedTokenizer`, and `CanonicalContext` into `ContextManager`
  (`mod.rs:336-382`). Its findings F-RCT-01-P3-02 (project-rules duplication
  post-compression) and F-RCT-01-P3-04 (`enable_task` naming) are referenced
  where they intersect this task.

Historical documents treated as hypotheses:

- `echo-core/src/budget.rs:1-10` module docstring claims the budget "divides
  the model's context window into four categories … and reserves a safety
  margin." Verified: the *structure* divides correctly, but the *wiring* never
  feeds real per-category sizes (V01 §Deviations, V04). Treated as partially
  stale.
- `echo-core/src/tokenizer.rs:34-47` claims `HeuristicTokenizer` is
  "recommended for mixed Chinese/English" and "should not be used for
  scenarios requiring exact token counting (e.g., quota management, billing)".
  Verified current.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `TokenBudget`, `Tokenizer` trait + `HeuristicTokenizer` / `SimpleTokenizer` / `CalibratedTokenizer`, `ContextManager` (with budget + protected markers + canonical re-injection), `CanonicalContext`, `infer_context_window`, `ModelProfileResolver`, `ContextAssembler`, and `ContextBudget` are all generic context-management machinery any `echo-agent` consumer needs. They correctly live in `echo_core` (budget, tokenizer, compression trait, types, capabilities) and `echo_state` (`ContextManager` implementation). |
| EKO product policy | None at this layer. The budget takes pure framework inputs (`total_window`, percentages, `TokenBudgetConfig.enabled`); it does not bake in any EKO-specific decision. The EKO YAML (`AppConfig`) is the *consumer* of this API, not part of it. |
| Adapter boundary | The `AppConfig::to_agent_config` / `resolve_context_window` translation in `src/config.rs` is the application-side adapter that turns YAML + provider name into a `TokenBudgetConfig`. It is thin and lossless. The framework-side `infer_context_window` is the seam being adapted *to*; its incomplete coverage (V03) is a framework defect, not an adapter defect. |
| Duplicate search | Searched names: `TokenBudget`, `TokenBudgetConfig`, `TokenAllocation`, `BudgetReport`, `ContextBudget`, `ContextAssembler`, `ContextSelector`, `ContextManager`, `ContextManagerBuilder`, `CanonicalContext`, `ProtectedContent`, `protected_markers`, `replaceable_protected_markers`, `Tokenizer`, `HeuristicTokenizer`, `SimpleTokenizer`, `CalibratedTokenizer`, `infer_context_window`, `resolve_context_window`, `ModelProfile`, `ModelProfileResolver`, `estimate_tokens`, `token_estimate`, `token_breakdown`, `should_compress`, `prepare`, `to_reinjection_messages`. Searched fields: all `TokenBudget` / `TokenBudgetConfig` / `ContextBudget` / `ContextManager` fields. Searched behaviours: budget allocation, protected-marker survival, canonical re-injection, token estimation. Result: one canonical definition per concept on the model-context path; `ContextAssembler` is a documented second building block for custom loops (`src/context/mod.rs:111-117`) and is not a parallel authority on the ReactAgent hot path. |
| Migration deletion | No migration proposed. The `ContextAssembler` building block is retained per AGENTS.md ("通用框架提供多个 … 是正常的框架设计"); only its estimator divergence is flagged for cleanup (F-CTX-01-P3-02). |

## Current Path

Verified context-selection and budget call graph at commit `9b0e0fa`:

```text
AppConfig (YAML)
   │  model.context_window?  agent.token_limit?  agent.compress_strategy?
   ↓
resolve_context_window(explicit?, provider, model)              [config.rs:99-104]
   = explicit.or(infer_context_window(provider,model))          [capabilities.rs:197-217]
       .unwrap_or(396_000)
       .clamp(1, 10_000_000)
   ↓
TokenBudgetConfig { total_window, system/tool/output/safety_pct, enabled }  [config.rs:129-137]
   ↓
AgentConfig.token_budget_config, AgentConfig.token_limit        [config.rs:139-157]
   ↓
ReactAgent::new_inner(config)                                    [mod.rs:322-583]
   │
   ├─ CalibratedTokenizer(HeuristicTokenizer) shared Arc        [mod.rs:333-335]
   │
   ├─ ContextManager::builder(config.token_limit)               [mod.rs:336-354]
   │      .with_system(system_prompt)
   │      .tokenizer(calibrated)
   │      .budget(config.token_budget_config.build(token_limit))   ← window → pct split
   │      .compressor(SlidingWindow 40)        [if token_limit<MAX or budget.enabled]
   │
   ├─ CanonicalContext { system_prompt, project_rules, skills } [mod.rs:360-381]
   │      ctx.set_canonical_context(canonical)
   │
   └─ (per turn)
       prepare_stream_context(_with_message)                     [run/context.rs:490-623]
          ├─ restore_thread_context / reset_messages
          ├─ recall_long_term_memories → replace_tail_projection  [:514-532]
          ├─ build_workspace_context_block → replace_projection   [:521-528]
          ├─ push history + user message                          [:533-536]
          └─ fire UserPromptSubmit hook → inject_hook_messages    [:544-552]
                ↓
       ContextManager::prepare(current_query)                    [compression/mod.rs:1243-1539]
          ├─ VisibilityHorizon pre-pass (if configured)          [:1251-1293]
          ├─ estimated_tokens = Σ tokenizer.count_tokens(text)   [:1295, 1541-1547]
          ├─ effective_limit / needs_compression                 [:1299-1315]
          │     budget.allocate(system=0, tool=0, conv=est)   ← phantom reservations
          ├─ if needs_compression && compressor:
          │     split_protected → compressor.compress(effective_limit)
          │                       → merge_protected
          │     fallback SlidingWindow(40) on primary failure    [:1407-1441]
          │     verifier P0 fallback SlidingWindow(40)           [:1464-1523]
          ├─ sanitize_tool_call_pairing (always)                 [:1448-1459]
          └─ reinject_canonical_context (if compression ran)      [:1528-1530]
                ↓
       PrepareResult { messages, compressed?, checkpoint?, verification? }
                ↓
       ChatRequest { messages, tools: ToolManager.tool_definitions(), … }
                                                       ↑ NOT seen by estimate_tokens
```

Key invariants verified by this graph:

- **Single budget authority.** One `TokenBudget` per `ContextManager`,
  constructed once at agent build time from `TokenBudgetConfig`. No parallel
  budget on the ReactAgent hot path.
- **Single tokenizer.** One `Arc<dyn Tokenizer>` (the calibrated heuristic)
  shared between `ReactAgent` and `ContextManager` so runtime calibration
  flows in (`mod.rs:333-338`).
- **Protected survival.** Markers + projection envelope + canonical context
  + tool-pair sanitisation together guarantee that protected content and
  valid tool-call structure survive every compression pass (V02).
- **Deterministic assembly.** `prepare_stream_context` pushes history and
  user input in fixed order; `replace_projection` /
  `replace_tail_projection` place workspace/memory blocks deterministically;
  `reinject_canonical_context` inserts at the `sys_end` boundary to keep
  history byte-stable for provider prefix caches (`:896-904`).

The graph also exposes four defects (see Findings): phantom per-category
reservations, uncounted tool definitions, an unsafe fallback default for
unknown models, and several unchecked-`usize` arithmetic sites.

## Findings

### F-CTX-01-P2-01: Tool definitions and system prompt are not accounted against the budget; per-category reservations are phantom

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/llm/types.rs:525-530` —
    `ChatCompletionRequest.tools: Option<Vec<ToolDefinition>>` is a request
    field **separate** from `messages`.
  - `echo-agent/echo-state/src/compression/mod.rs:1541-1547` —
    `estimate_tokens` iterates only `messages` (`filter_map(|m|
    m.content.as_text())`); tool definitions are never visible to the
    `ContextManager`.
  - `echo-agent/echo-state/src/compression/mod.rs:1300, 1311, 474` — every
    call to `budget.allocate` passes `system_size=0, tool_defs_size=0,
    conversation_size=estimated_tokens`. Inline comments at `:1309-1310`
    admit it: `// system prompt tokens already counted in messages` and
    `// tool defs not in messages`.
  - `echo-agent/echo-core/src/budget.rs:65-86, 96-104` — the budget reserves
    10% for `system_prompt_budget`, 5% for `tool_definitions_budget`, 10%
    output, 10% safety, leaving 65% for conversation. With `system_size=0`
    and `tool_defs_size=0`, `system_fits` and `tool_defs_fit` are always
    `true`, and the 15% reserved for system+tools sits empty while the system
    prompt (inside `messages`) silently consumes conversation budget and the
    tool defs consume nothing.
- Reachability: every `prepare()` call on every ReactAgent that has
  `token_budget_config.enabled` (the default) hits this path.
- Expected invariant: the budget's per-category reservations correspond to
  real, measured byte/token costs of the categories they name; the safety
  margin absorbs *unseen* overhead, not budgeted categories.
- Observed behavior: the system prompt is counted inside the conversation
  bucket (so `conversation_budget` is silently short-charged), and tool
  definitions are uncounted entirely. The 10% safety margin is computed from
  a budget whose inputs are zero, so it does not actually absorb the
  uncounted tool/system bytes.
- Impact: for models with many MCP tools (EKO is MCP-heavy) the prepared
  request routinely exceeds the real window: a 200K model with 30K of tool
  schemas, 8K system prompt, and 130K of history (the budget's "65%
  conversation") produces a ~168K prompt body before output reservation,
  leaving no headroom and tripping provider 400 / `context_length_exceeded`
  errors. This is a capability failure on the core chat path under normal
  EKO configuration (large tool surface).
- Root cause: `ContextManager` was designed before `ToolManager` integration
  and has no handle to the tool registry; `estimate_tokens` was written to
  count messages only. The budget's `allocate` signature accepts per-category
  sizes but no caller populates them with real measurements.
- Direction: pass the tool-definition token cost (and ideally the
  separately-measured system-prompt cost) into `allocate`. Concretely,
  `ReactAgent` already holds both the `ContextManager` and the
  `ToolManager`; expose a `ToolManager::estimated_definition_tokens(&dyn
  Tokenizer)` helper and have the prepare-phase caller feed
  `budget.allocate(system_tokens, tool_tokens, conversation_tokens)` with
  real values. Alternatively (smaller blast radius), subtract a one-shot
  `tool_definitions_overhead` from `effective_limit` inside `prepare()` and
  document the residual gap. Either way, `protected_token_estimate()` (V02)
  should also be subtracted so re-merged protected content does not blow the
  real window.
- Regression validation: add a test that installs a 20K-token tool schema,
  fills `messages` to `conversation_budget`, runs `prepare()`, and asserts
  the returned messages + tool defs fit within `total_window −
  output_budget`. Add a test that `allocate` is called with non-zero
  `tool_defs_size` on the ReactAgent path.
- Validation reports: [V01-01](../validations/F-CTX-01/V01-01.md),
  [V04-01](../validations/F-CTX-01/V04-01.md).

### F-CTX-01-P2-02: Protected content survives compression but its token cost is not deducted from the compressor's effective limit

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-state/src/compression/mod.rs:656-665` —
    `protected_token_estimate` exists and correctly sums protected-message
    tokens.
  - `echo-agent/echo-state/src/compression/mod.rs:1299-1305` —
    `effective_limit` is derived from `estimated_tokens` (which **includes**
    protected messages, since `estimate_tokens` at `:1541-1547` iterates all
    messages) but the compressor is then asked to fit only the
    **compressible** subset (`split_protected` at `:1336` removes protected
    before compressing). Protected messages are merged back on top at
    `:1353` / `:1418` / `:1492`.
  - The net effect: `effective_limit` targets the post-merge total
    (approximately), but the compressor believes it has the full
    `effective_limit` available for compressible content alone.
- Reachability: every compression pass on a `ContextManager` that has any
  protected marker or projection registered. EKO's runtime registers the
  workspace projection (`run/context.rs:525-528`), the turn-memory tail
  projection (`:529-532`), and (under `project-rules`) the canonical system
  prompt / rules — so every turn is affected.
- Expected invariant: the budget accounts for bytes that will appear in the
  final model request, including protected content that bypasses compression.
- Observed behavior: a large protected block (e.g. a 30K-token subagent brief
  registered via `add_replaceable_protected_marker`) is merged back after the
  compressor fit history into `effective_limit`, so the real request body is
  `compressor_output + protected`, which can exceed the window.
- Impact: same failure mode as F-CTX-01-P2-01 (provider 400) but triggered by
  protected content size rather than tool-schema size. Realistic for EKO when
  a subagent is dispatched with a large brief or when long-lived projections
  accumulate.
- Root cause: `protected_token_estimate` was added for observability
  (`/context` visualisation) but never wired into the limit arithmetic.
- Direction: subtract `protected_token_estimate()` from `effective_limit`
  before passing it to the compressor, and account for it in
  `needs_compression`. Document that `protected` bytes are reserved against
  the conversation bucket.
- Regression validation: add a test that registers a 5K-token protected
  marker, fills compressible history to `conversation_budget − 5K + 1`, runs
  `prepare()`, and asserts compression fires and the post-merge total is
  within `total_window − output_budget − safety`.
- Validation reports: [V02-01](../validations/F-CTX-01/V02-01.md).

### F-CTX-01-P2-03: `infer_context_window` ignores its `provider` argument despite the name and docstring

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/llm/capabilities.rs:197` — signature is
    `pub fn infer_context_window(_provider: &str, model_name: &str) ->
    Option<u32>`; the body (`:198-216`) never reads `_provider`.
  - `echo-agent/echo-core/src/llm/capabilities.rs:195-196` — docstring says
    "根据厂商和模型名称推断上下文窗口大小" (infer based on vendor *and* model
    name).
  - `echo-agent/src/config.rs:112-116` — caller passes `&self.model.provider`
    as the first argument, implying the result depends on it.
  - Coverage table (`:199-216`) lists only 8 name prefixes (`gpt-5.6*`,
    `claude-fable-5/opus-4-8/sonnet-5`, `deepseek-v4`, `qwen3.7-max/plus`,
    `kimi-k2.7/2.6`, `glm-5.2`). All other models — `gpt-4o`, `gpt-4-turbo`,
    `claude-opus-4.6`, `claude-3.7-sonnet`, `deepseek-v3.2`, `qwen3-235b`,
    `glm-4.6`, `llama-3/4`, etc. — return `None`.
- Reachability: every agent build that does not pass an explicit
  `model.context_window` calls this function.
- Expected invariant: a function named `infer_context_window(provider,
  model)` whose docstring promises vendor-aware inference should honour the
  vendor argument, or be renamed.
- Observed behavior: vendor is discarded; inference is model-name-only. A
  model hosted on two backends with different windows (e.g. Qwen via
  DashScope vs. self-hosted Ollama) cannot be distinguished.
- Impact: misleading public API for third-party consumers; the
  `ModelProfileOverride` escape hatch (`capabilities.rs:285-317`) exists but
  is the only way to get vendor-correct windows. Not a runtime crash, but a
  real source of incorrect budgets for any model not in the 8-prefix table
  (which is most of them).
- Root cause: the provider parameter was added for future use but the table
  was never keyed on it.
- Direction: either (a) make the table `(provider, model_prefix) → window`
  and honour the argument (preferred; the data exists in vendor docs), or
  (b) rename to `infer_context_window_by_model(model_name)` and update the
  docstring + call site. (a) is more useful; (b) is cheaper. Either way,
  the `pub` API is retained per AGENTS.md framework-API-preservation rule.
- Regression validation: `infers_current_frontier_context_windows`
  (`capabilities.rs:603-638`) should keep passing; add cases for at least one
  provider-qualified model and one model that needs provider disambiguation.
- Validation reports: [V03-01](../validations/F-CTX-01/V03-01.md).

### F-CTX-01-P2-04: Unknown-model fallback `396_000` is larger than the real windows of the most common 128K/200K models

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/config.rs:102` —
    `resolve_context_window` falls back to `unwrap_or(396_000)`.
  - `echo-agent/src/config.rs:855-862` —
    `unknown_model_uses_396k_default_context_window` codifies the default as
    intentional.
  - `echo-agent/echo-core/src/budget.rs:150-155` and
    `echo-agent/src/context/mod.rs:46-56` — `TokenBudget::default()` and
    `ContextBudget::default()` both hard-code `396_000` as well, so the magic
    number is repeated three times.
- Reachability: any model whose name does not match the 8 prefixes in
  `infer_context_window` *and* whose user did not set `context_window`
  explicitly. This includes the majority of OpenAI / Anthropic / DeepSeek /
  GLM / Qwen models in production use today.
- Expected invariant: a default context window should be conservative —
  smaller than the smallest plausible real window — so an unconfigured user
  cannot prepare a request that exceeds the real window.
- Observed behavior: `396_000` is larger than 128K (gpt-4o, gpt-4-turbo,
  deepseek-v3.2, glm-4.6, claude-3-haiku) and larger than 200K
  (claude-3.7/4.x, gpt-4.6). With compression enabled via
  `compress_strategy = "summary"` (the YAML default) but no explicit
  `context_window`, the framework treats the window as 396K, lets
  `conversation_budget` grow to `0.65 × 396K ≈ 257K`, and sends requests
  that the real 128K/200K model rejects with HTTP 400.
- Impact: hard chat-path breakage for users who enable compression (the
  documented default) without pinning their model's window. Compounds with
  F-CTX-01-P2-01 (uncounted tools) and F-CTX-01-P2-02 (uncounted protected).
- Root cause: 396K was likely chosen as a "safe large" value when the
  framework targeted frontier models; it is unsafe for the long tail.
- Direction: lower the default to a conservative value that no common model
  exceeds (e.g. 32K or 64K), and emit a `tracing::warn!` on the
  `None → default` path telling the user to set `context_window`
  explicitly. Alternatively, refuse to enable the budget until a window is
  known (return `TokenBudgetConfig::disabled()` from `to_agent_config` when
  inference returns `None`). The warning + conservative default is the
  least-disruptive fix.
- Regression validation: update
  `unknown_model_uses_396k_default_context_window` to assert the new
  conservative default; add a test that with the default and a 128K
  `compress_strategy=summary` config, `conversation_budget ≤ 0.65 × 128K`.
- Validation reports: [V03-01](../validations/F-CTX-01/V03-01.md).

### F-CTX-01-P3-01: Unchecked `usize` arithmetic on the budget path violates the AGENTS.md checked-arithmetic rule

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/budget.rs:106` —
    `let total_used = system_size + tool_defs_size + conversation_size;`
    (plain `usize + usize + usize` inside the `pub` `allocate()` API).
  - `echo-agent/echo-state/src/compression/mod.rs:1546` —
    `estimate_tokens`: `.map(|c| tokenizer.count_tokens(&c)).sum();`
    (`Iterator::sum::<usize>` is plain `+=`; this value drives
    `needs_compression` / `effective_limit`).
  - `echo-agent/echo-state/src/compression/mod.rs:664` —
    `protected_token_estimate`: `.map(...).sum()` (observability).
  - `echo-agent/echo-state/src/compression/mod.rs:1003` —
    `let total = system + user + assistant + tool + summary + memory;`
    (six-way chain; observability via `token_breakdown`).
  - `echo-agent/src/context/mod.rs:209, 234` —
    `ContextAssembler::assemble`: `token_est += t;` where
    `t = c.len() / 4` (parallel building-block path).
- Reachability: `estimate_tokens` (`:1546`) runs on every `prepare()` and
  every `should_compress()` — the hot path. The others are observability or
  the `ContextAssembler` path.
- Expected invariant (AGENTS.md Rust constraint #2): integer arithmetic
  that may overflow uses `checked_*` / `saturating_*` / `wrapping_*`; never
  plain `+` / `+=` on externally-influenced sizes.
- Observed behavior: plain addition. In debug builds a sufficiently large
  message set (or a future caller of `allocate` with three large `usize`
  values) panics on overflow; in release it silently wraps.
- Impact: low realistic risk on EKO's local single-user threat model
  (messages are bounded by what one user types), but the pattern is
  non-compliant with the project's own hard constraint and is silently
  accreting because the workspace Clippy gate lists only
  `unwrap_used`/`expect_used`/`panic`/`unreachable` — not
  `arithmetic_side_effects`.
- Root cause: pre-constraint code; no enforcement gate.
- Direction: replace the five sites with `saturating_add` (or `checked_add`
  with an explicit error for the `allocate` API). Optionally add
  `-D clippy::arithmetic_side_effects` to the workspace Clippy gate in
  `AGENTS.md` so the pattern cannot re-enter.
- Regression validation: `cargo clippy --workspace --lib --bins
  --all-features --locked -- -D clippy::arithmetic_side_effects` should pass
  after the fix (this gate is not currently in `AGENTS.md`; adding it is
  itself a small docs change).
- Validation reports: [V01-01](../validations/F-CTX-01/V01-01.md).

### F-CTX-01-P3-02: `ContextAssembler` uses a byte-based token estimator that diverges from the configured `Tokenizer`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/context/mod.rs:208-211` (history) and `:233-237` (tool
    results) — both use
    `let t = m.content.as_text().map(|c| c.len() / 4).unwrap_or(0);`
    followed by `token_est += t;`.
  - `echo-agent/src/context/mod.rs:111-117` — docstring states the default
    `ReactAgent` path does **not** use `ContextAssembler`, so this divergence
    does not affect EKO's default runtime.
  - `echo-agent/echo-core/src/tokenizer.rs:50-58` — `HeuristicTokenizer`
    (the default on the ReactAgent path) uses char-weighted counting, not
    `len() / 4`.
- Reachability: any third-party consumer that builds a custom execution loop
  with `ContextAssembler::with_budget(...)`. EKO's default path is unaffected.
- Expected invariant: the same text yields the same token estimate
  regardless of which framework API a consumer uses.
- Observed behavior: `ContextAssembler` over-estimates CJK content relative
  to `HeuristicTokenizer` (3 bytes / 4 = 0.75 tokens per CJK char vs.
  heuristic's 0.5). The two paths disagree, and `ContextAssembler` ignores
  any consumer-injected `CalibratedTokenizer`.
- Impact: third-party consumer footgun; no EKO impact. Also blocks the
  AGENTS.md "多模式功能对等" guarantee conceptually — if the GUI ever routes
  through `ContextAssembler`, its estimates will diverge from the TUI.
- Root cause: `ContextAssembler` predates the pluggable `Tokenizer` and was
  not retrofitted.
- Direction: give `ContextAssembler` an `Option<Arc<dyn Tokenizer>>` field
  (default `HeuristicTokenizer`) and use it instead of `c.len() / 4`. No
  deletion target — `ContextAssembler` is a legitimate framework building
  block.
- Regression validation: add a test that runs the same CJK text through both
  `ContextAssembler` and `ContextManager` (with the same tokenizer) and
  asserts equal estimates.
- Validation reports: [V04-01](../validations/F-CTX-01/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Budget arithmetic / overflow / phantom reservations | yes | **failed** (2 defects) | [V01-01](../validations/F-CTX-01/V01-01.md) |
| V02 | Protected-content survival + protected-token budget accounting | yes | **failed** (1 defect: protected not deducted) | [V02-01](../validations/F-CTX-01/V02-01.md) |
| V03 | Provider-window mapping | yes | **failed** (2 defects: not per-provider; risky default) | [V03-01](../validations/F-CTX-01/V03-01.md) |
| V04 | Large-schema + multilingual token counting | yes | **failed** (2 defects: tools uncounted; estimator divergence) + safety sub-claim passed | [V04-01](../validations/F-CTX-01/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — `budget.rs` and `tokenizer.rs` module docstrings make auditable claims) | done — see Historical Claim Status table below | — |

Targeted executable checks run as part of V04-01:

| Command | Exit | Result |
|---|---:|---|
| `cargo test -p echo_core --lib tokenizer:: --locked` | 0 | 12 passed, 0 failed |
| `cargo test -p echo_state --lib compression::tests --locked` | 0 | 28 passed, 0 failed |

No conditional feature/GUI/frontend matrix was run: this task touches only
framework context-management code that is compiled under the default feature
set, and `AGENTS.md` reserves the feature matrix for changes to `Cargo.toml`,
feature definitions, `#[cfg]` branches, or cross-crate public API. The
findings here are about *runtime arithmetic and wiring*, not feature-gated
code. F-FEAT-01 owns the full matrix.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `echo-core/src/budget.rs:1-10` — "Divides the model's context window into four categories … and reserves a safety margin" | partial drift | Structure divides correctly, but `allocate()` is always called with `(0, 0, est)`, so the system/tool categories are never measured and the safety margin is fictional (F-CTX-01-P2-01). |
| `echo-core/src/tokenizer.rs:34-47` — "HeuristicTokenizer … recommended for mixed Chinese/English … not for exact counting" | current | V04 confirms char-weighted counting, no UTF-8 panic, CJK tests pass. |
| `echo-state/src/compression/mod.rs:316-324` — protected markers "survive compaction" and replaceable markers carry "latest-wins" semantics | current | V02 confirms `split_protected` / `merge_protected` / `push` retain/replacement semantics. |
| `echo-state/src/compression/mod.rs:874-904` — canonical re-injection "keeps the history segment's byte positions stable, preserving both Anthropic explicit cache breakpoints and OpenAI automatic prefix caches" | current (design intent) | V02 confirms `sys_end` insertion + dedup. The cache *hit-rate* claim is not measured here (deferred to F-LLM-01 / a cache-policy task). |
| `echo-core/src/llm/capabilities.rs:195-196` — "根据厂商和模型名称推断上下文窗口大小" | stale | Vendor argument is ignored (F-CTX-01-P2-03). |
| `echo-core/src/compression.rs:373-375` — "the prompt is not represented twice" | partial drift (already tracked) | F-RCT-01-P3-02 established project_rules is duplicated post-compression; not re-litigated here. |
| `echo-agent/src/context/mod.rs:111-117` — "The default `ReactAgent` streaming path … does NOT use `ContextAssembler`" | current | V04 confirms `ContextAssembler` divergence is off the hot path; the claim protects EKO from F-CTX-01-P3-02 today. |

## Coverage And Uncertainty

Inspected in full: `echo-core/src/budget.rs`, `echo-core/src/tokenizer.rs`,
`echo-core/src/compression.rs`, `echo-core/src/llm/capabilities.rs`,
`echo-core/src/llm/types.rs:43-147, 525-540`, `echo-state/src/compression/mod.rs`
(key ranges above; full read of `:300-1548, 1660-1830`), `src/config.rs:1-300,
855-862`, `src/context/mod.rs`, `src/context/selector.rs`, `src/agent/react/mod.rs:325-400`,
`src/agent/react/run/context.rs`.

Not inspected (out of scope or deferred):

- Compression-strategy implementations (`SummaryCompressor`,
  `IncrementalSummaryCompressor`, `HybridCompressor`, `AdaptiveCompressor`,
  `SlidingWindowCompressor` bodies in `echo_state::compression::compressor`)
  — F-MEM-01 / compression task. Only their `CompressionInput::token_limit`
  contract was inspected.
- Visibility-horizon compaction body (`echo_state::compression::horizon`) —
  horizon task. Only the `prepare()` integration (`:1251-1293`) was inspected.
- Verifier body (`echo_state::compression::verifier`) — only the P0 fallback
  wiring at `:1464-1523` was inspected.
- `src/agent/subagent/context*.rs` — subagent task.
- Image / multimodal content token accounting — `ContentPart::Image` is not
  counted by `as_text()` (`echo-core/src/llm/types.rs:101-114`); multimodal
  budgeting is a separate concern, noted in V04-01 §Deviations.

Environmental constraints:

- Two `cargo test` commands run (echo_core tokenizer, echo_state
  compression) — both green at `9b0e0fa`. No feature matrix, no frontend
  build, no GUI check (out of scope for this framework-only task).

Uncertain claims:

- The exact provider-cache hit-rate impact of the `sys_end` insertion
  strategy is asserted by code comments but not measured here; deferred to a
  cache-policy task or F-LLM-01.
- Whether any third-party `echo-agent` consumer calls `TokenBudget::allocate`
  with non-zero `system_size` / `tool_defs_size` — the framework API permits
  it, but the in-tree caller (`ContextManager::prepare`) does not do so.
  External consumers cannot be inspected from this repository.

## Handoff

Conclusions downstream tasks may rely on:

1. **Budget reservation structure is correct; wiring is broken.**
   `TokenBudget` / `TokenBudgetConfig` / `TokenAllocation` are the right
   abstraction and live in the right layer. The defect is that the sole
   in-tree caller (`ContextManager::prepare`) passes `(0, 0, est)` to
   `allocate`, making the system/tool reservations phantom. Any task that
   touches budget accounting (F-MEM-01, F-LLM-01, F-RCT-02) can rely on the
   types and should fix the wiring, not redesign the types.
2. **Protected survival is solid; protected *budget* is not.**
   Marker / projection / canonical / sanitize mechanics correctly preserve
   content and tool-pair validity across compression. The gap is purely in
   token accounting (`protected_token_estimate` is observability-only). A
   task that adds protected-token deduction should change only
   `prepare()` / `should_compress()`, not the survival path.
3. **`infer_context_window` is model-name-only.** Any task that reasons
   about provider differences (F-LLM-01/02/03) must not assume this function
   distinguishes providers — it does not.
4. **The 396K default is unsafe for sub-frontier models.** Tasks auditing
   EKO's model switcher / GUI config defaults should flag combinations where
   compression is enabled without an explicit `context_window`.
5. **No UTF-8 panic risk on the counting path.** Multilingual content is
   safe; the `HeuristicTokenizer` + `CalibratedTokenizer` combination is the
   project-recommended estimator and remains so.

Reports they must read:

- This report (F-CTX-01) for the budget / selection invariants and defects.
- `tasks/F-RCT-01.md` for the construction-path wiring that produces the
  `ContextManager` + `CanonicalContext` this task inspects.
- `validations/F-CTX-01/V01-01.md` through `V04-01.md` for per-claim
  evidence and the executable test results.

Conditions that make this report stale:

- Wiring of real `system_size` / `tool_defs_size` into
  `ContextManager::prepare`'s `budget.allocate` call — resolves
  F-CTX-01-P2-01, requires re-running V01-01 and V04-01.
- Subtraction of `protected_token_estimate()` from `effective_limit` —
  resolves F-CTX-01-P2-02, requires re-running V02-01.
- Change to `infer_context_window` (vendor-aware, or renamed) or to the
  396K fallback default — resolves F-CTX-01-P2-03 / P2-04, requires
  re-running V03-01 and updating `unknown_model_uses_396k_default_context_window`.
- Replacement of the five unchecked-`usize` sites with saturating arithmetic
  — resolves F-CTX-01-P3-01, requires re-running V01-01.
- `ContextAssembler` adopting a pluggable `Tokenizer` — resolves
  F-CTX-01-P3-02, requires re-running V04-01.

Follow-up task IDs (no implementation in this review task):

- **F-MEM-01** — owns compression-strategy correctness; should consume the
  protected-token deduction fix from F-CTX-01-P2-02 so its summaries do not
  blow the window when protected content is large.
- **F-LLM-01/02/03** — own provider request construction and cache policy;
  should consume the tool-definition-accounting fix from F-CTX-01-P2-01 and
  the vendor-aware window inference from F-CTX-01-P2-03.
- A future cleanup task could add `-D clippy::arithmetic_side_effects` to
  the workspace Clippy gate once the five sites in F-CTX-01-P3-01 are fixed.
