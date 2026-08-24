# F-RCT-01: ReAct construction and canonical prompt assembly

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Does builder/config assembly produce a deterministic Agent with one tool
registry, correct instructions, budgets, hooks, and project rules?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/agent/react/builder.rs` (1226 lines) — `ReactAgentBuilder`
  struct, ~40 builder methods, `simple()`/`standard()`/`full_featured()`
  presets, and the `build()` orchestrator.
- `echo-agent/src/agent/react/capabilities.rs` (1422 lines) — runtime
  capability API: `add_tool`/`add_tools`/`remove_tool`/`replace_tool`,
  `register_subagent_*`, MCP connect, skill discovery, callback mutation.
- `echo-agent/src/agent/config.rs` (1213 lines) — `AgentConfig` struct
  (~45 fields), the parallel `AgentConfig` builder chain, the
  `AgentRole` enum, and the `normalize_permission_mode` helper.
- `echo-agent/src/agent/react/mod.rs:267-583, 676-826` — `ReactAgent::new`,
  `new_with_subagent_registry`, `new_inner` (the actual constructor),
  `build_system_prompt`, `register_feature_gated_tools`, `setup_memory_store`,
  `set_memory_store`/`install_memory_store`.
- `echo-agent/echo-execution/src/tools.rs:55-66, 505-585` — `ToolManager`
  definition, `register`/`register_tools` semantics, name-collision
  behaviour.
- `echo-agent/src/tasks.rs:15-23` — `register_task_tools` helper.
- `echo-agent/echo-core/src/project_rules.rs:180-218` —
  `rules_injection_with_root` and `inject_rules_with_root`.
- `echo-agent/echo-core/src/compression.rs:345-402` — `CanonicalContext`
  and `to_reinjection_messages`.

Cross-checks (lighter): `echo-agent/echo-tools/src/registry.rs`
(`register_all_tools`/`register_readonly_tools`),
`echo-agent/src/agent/react/tests.rs` (existing prompt/callback tests).

## Out Of Scope

Deferred to named task IDs:

- Concrete ReAct loop execution (`think`, `process_steps`,
  `run_react_loop`) → **F-RCT-02** (Non-streaming ReAct loop).
- Streaming variant of the loop → **F-RCT-03** if declared.
- Tool execution pipeline lifecycle (hooks, guards, permission checks
  around each tool call) → a tool-execution-focused task.
- LLM client construction and provider routing → **F-LLM-01/02/03**.
- Memory compression algorithm correctness → **F-MEM-01**.
- Subagent executor and dispatch semantics → the subagent task.
- Public facade re-export coherence (where `ReactAgentBuilder` is
  re-exported) → **F-API-01** (already complete).
- `TaskRun`/`PlanTask` revision semantics → **F-TSK-01** and the
  application task_runtime task.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full,
  via system reminder — especially the framework-vs-application layering
  rule, the "first check if it already exists" rule, the UTF-8 safety
  rule, and the dead-code cleanup rule).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-CORE-01.md` (in full).
  F-CORE-01 establishes the `Agent` trait, `AgentEvent`, `AgentCallback`,
  and the framework/application layering model that this task builds on.
  Its "Dead infra: GLOBAL_EVENT_BUS / EventBus" finding
  (F-CORE-01-P2-01) is relevant context for the runtime-side event bus,
  not for the construction path reviewed here.
- `docs/comprehensive-review/zcode-glm/tasks/F-API-01.md` (in full).
  F-API-01 establishes the public-facade coherence rule. Its finding
  F-API-01-P2-02 (parallel access paths to the same items) is the
  facade-level analogue of what this task checks at the construction
  level (single tool registry, single canonical prompt).

Historical documents treated as hypotheses:

- `echo-agent/src/agent/react/mod.rs` module docstring (lines 1-11)
  claims the module owns struct definition + `new()` + `impl Agent`. The
  code matches that claim. Treated as current; no drift.
- The `ReactAgentBuilder::register_agent_dispatch_tool` docstring
  (builder.rs:411-420) claims independence from `enable_subagent`.
  Confirmed at the code level (`mod.rs:478-497` registers the tool iff
  `config.register_agent_dispatch_tool`, regardless of
  `config.enable_subagent`). Treated as current.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `ReactAgentBuilder`, `AgentConfig`, `ReactAgent::new`, `build_system_prompt`, the `ToolManager` registration API, and the `CanonicalContext` re-injection mechanism are generic agent-runtime machinery. Any `echo-agent` consumer — EKO, a headless CLI user, a third-party reuser — needs them and they live correctly in `echo-agent` (root crate) plus `echo_execution` (ToolManager) and `echo_core` (CanonicalContext, project-rules helpers). |
| EKO product policy | None at this layer. Construction takes pure framework inputs (`AgentConfig` flags, builder options); it does not bake in any EKO-specific decision. Application policy enters only through injected adapters (`TaskRevisionService`, `WorktreeFactory`, `RuntimeStateStore`, approval providers) — all supplied by the caller, not constructed here. |
| Adapter boundary | The construction path is itself the adapter boundary: `ReactAgentBuilder::build()` translates ~40 builder options into `AgentConfig` + post-construction setters. The conversion is thin and lossless for Category-A and Category-B options (V01). Five Category-C options bypass `AgentConfig` and write directly to agent fields (V01 §Deviations) — they remain lossless but break the uniform pattern. |
| Duplicate search | Searched names: `ReactAgentBuilder`, `ReactAgent::new`, `new_inner`, `new_with_subagent_registry`, `build_system_prompt`, `AgentConfig`, `ToolManager`, `register`, `register_tools`, `register_task_tools`, `register_all_tools`, `register_readonly_tools`, `setup_memory_store`, `set_memory_store`, `install_memory_store`, `CanonicalContext`, `inject_rules_with_root`, `rules_injection_with_root`. Searched fields: all 50+ `ReactAgentBuilder` fields, all ~45 `AgentConfig` fields. Searched behaviours: tool registration, system-prompt assembly, project-rules injection, memory-store wiring, feature-gated tool registration. Result: one canonical definition per concept; the parallel paths converge on the same `ToolManager`/`ContextManager`. |
| Migration deletion | No migration proposed. The duplicate memory-registration blocks (V02 §Deviations) are a cleanup candidate but not a parallel authority. |

## Current Path

Verified agent-construction call graph at commit `9b0e0fa`:

```text
ReactAgentBuilder::new()                                [builder.rs:119-183]
   ↓ (user calls ~40 fluent setters)
ReactAgentBuilder::build()                              [builder.rs:854-1065]
   │
   ├─ validation: model non-empty; subagent ⊃ tools     [builder.rs:856-870]
   ├─ AgentConfig::new(...).role(...).enable_tool(...)   [builder.rs:872-888]
   │      + subagent factories (#[cfg(subagent)])       [builder.rs:897-914]
   │      + response_format / output tokens / artifacts [builder.rs:916-924]
   │      + callbacks.append(...)                        [builder.rs:926-928]
   │      + session_id / conversation_id / working_dir   [builder.rs:930-940]
   │      + react_checkpoint_interval                    [builder.rs:941-943]
   │      + force enable_memory=false if external store  [builder.rs:948-951]
   │
   ├─ ReactAgent::new(config)                            [mod.rs:303-312]
   │      OR
   │  ReactAgent::new_with_subagent_registry(config,reg) [mod.rs:314-320]
   │      │
   │      └─ new_inner(config, registry?)                [mod.rs:322-583]
   │           │
   │           ├─ build_system_prompt(&config)           [mod.rs:326, 676-723]
   │           │      →  system_prompt + CoT + suffix + project_rules(prefix)
   │           │
   │           ├─ ContextManager::builder()              [mod.rs:336-356]
   │           │      .with_system(prompt)
   │           │      .tokenizer(calibrated)
   │           │      .budget(token_budget_config)
   │           │      .compressor(SlidingWindow 40)
   │           │
   │           ├─ CanonicalContext { system_prompt,      [mod.rs:358-382]
   │           │      project_rules, skill_injections, active_skill_names }
   │           │      ctx.set_canonical_context(canonical)
   │           │
   │           ├─ ToolManager::new_with_config(...)      [mod.rs:386]
   │           │      .register(FinalAnswerTool)         [mod.rs:393]
   │           │      .register_tools(build_task_tools(  [mod.rs:398-400]
   │           │           InMemoryRevisionedTaskStore))
   │           │
   │           ├─ feature-gated tool registration:       [mod.rs:442-497]
   │           │      #[cfg(human-loop)] HumanInLoop
   │           │      #[cfg(tasks)] Spawn/Check/List background tasks
   │           │      register_feature_gated_tools(config, tm)
   │           │          → echo_tools::register_all_tools OR
   │           │            echo_tools::register_readonly_tools
   │           │      #[cfg(subagent)] AgentDispatchTool (if register_agent_dispatch_tool)
   │           │
   │           ├─ setup_memory_store(config, tm)         [mod.rs:457, 750-784]
   │           │      (only when enable_memory AND no external store)
   │           │
   │           └─ ToolSearchTool self-reference          [mod.rs:461]
   │
   ├─ register_task_tools(&mut agent, service)           [builder.rs:962-964]
   │      (only when task_revision_service supplied)
   │      → agent.add_tools(build_task_tools(service))
   │      → replaces default InMemory task tools by name
   │
   ├─ agent.set_llm_client(...) / set_llm_config(...)    [builder.rs:966-973]
   ├─ agent.add_tool(custom)*                            [builder.rs:976-978]
   ├─ agent.set_memory_store(store) (if external store)  [builder.rs:981-983]
   ├─ #[cfg(human-loop)] approval_provider / permission  [builder.rs:985-993]
   ├─ agent.set_guard_manager / audit / snapshot / cb    [builder.rs:996-1013]
   ├─ agent.set_sandbox_manager(unwrap_or local_only)    [builder.rs:1017-1020]
   ├─ agent.run_store = ...                              [builder.rs:1023-1025]
   ├─ agent.tool_execution_pipeline = ...                [builder.rs:1028-1030]
   ├─ agent.set_prompt_template_engine(...)              [builder.rs:1033-1035]
   ├─ agent.intent_router = ...                          [builder.rs:1038-1040]
   ├─ agent.memory.state_store = ...                     [builder.rs:1043-1045]
   ├─ ContextManager visibility horizon (try_lock)       [builder.rs:1048-1057]
   └─ agent.tools.intervention_callbacks = ...           [builder.rs:1060-1062]
       ↓
   Result: ReactAgent
```

Key invariants verified by this graph (full evidence in V01/V02):

- **Single tool registry.** Every registration path converges on
  `ToolExecutionSubsystem.tool_manager: Arc<ToolManager>`. The
  `DashMap<String, Box<dyn Tool>>` storage (`tools.rs:56`) makes
  duplicate-name registrations idempotent replacements, never
  accumulations. The task-tool surface has exactly one API
  (`task_create`/`task_update`/`task_list`), and the historical
  `todo_write` is absent — verified by `default_agent_uses_one_task_relation_api`
  (`builder.rs:1170-1183`).
- **Deterministic system prompt at construction time.**
  `build_system_prompt` produces a byte-identical string for identical
  inputs; section order is `[project_rules]\n\n[base]\n\n[CoT]\n\n[suffix]`
  (V03).
- **Budget/hook wiring.** Token budget, run budget, snapshot policy,
  circuit breaker, audit logger, sandbox manager, guardrails, intervention
  callbacks, and hook registry are all wired before the agent is returned
  (V01 Category-A/B). The only gap is the silent-noop behaviour when a
  feature flag is off (V04 → F-RCT-01-P3-01).

## Findings

### F-RCT-01-P3-01: Feature-gated builder methods silently no-op when the Cargo feature is disabled

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/builder.rs:340-349` — `enable_human_in_loop()`
    and `enable_subagent()` are unconditional (no `#[cfg(feature)]` gate on
    the method itself); they set the `AgentConfig` flag and return `Self`.
  - `echo-agent/src/agent/react/builder.rs:417-420` —
    `register_agent_dispatch_tool()` likewise unconditional.
  - `echo-agent/src/agent/react/mod.rs:442-445` — `HumanInLoop` is
    registered inside `#[cfg(feature = "human-loop")] if config.enable_human_in_loop { ... }`.
    With the feature off, the entire block is compiled out and the
    `config.enable_human_in_loop == true` flag is silently ignored.
  - `echo-agent/src/agent/react/mod.rs:477-497` — `AgentDispatchTool`
    registration is inside `#[cfg(feature = "subagent")] if config.register_agent_dispatch_tool { ... }`.
    With the feature off, `register_agent_dispatch_tool == true` is silently
    ignored.
- Reachability: builder method → `AgentConfig` flag → `new_inner`
  `#[cfg(feature)]` gate compiled out → no tool registered. End-to-end
  confirmed by executable probe in V04.
- Expected invariant: if a builder method sets a capability flag, the
  built agent either has that capability OR `build()` returns an error
  explaining the missing feature. The `AgentConfig` accessors
  (`is_human_in_loop_enabled`, `is_subagent_enabled`) should report the
  *actual* runtime state.
- Observed behavior: with `--no-default-features`, calling
  `.enable_tools().enable_human_in_loop().enable_subagent().register_agent_dispatch_tool()`
  succeeds; `build()` returns `Ok(agent)`; the config accessors return
  `true`; but `agent.tool_names()` contains neither `human_in_loop` nor
  `agent_tool`. The agent silently lacks the requested capabilities.
- Impact: misleading state for downstream framework consumers who trim
  features. A caller inspecting `agent.config().is_subagent_enabled()`
  will believe subagent dispatch is available; runtime dispatch attempts
  then fail with no prior diagnostic. The local-assistant product (EKO)
  is unaffected because it builds with all features on; the trap fires
  for third-party reuse.
- Root cause: the builder methods predate the feature split and were not
  annotated with `#[cfg(feature = "...")]`. The construction-time
  validation in `build()` (`builder.rs:855-870`) checks only
  `enable_subagent ⊃ enable_builtin_tools`, not feature-flag consistency.
- Direction: pick one of two fixes.
  (1) Annotate each feature-gated builder method with the matching
  `#[cfg(feature = "...")]` so it disappears when the feature is off
  (consistent with the existing `#[cfg(feature = "subagent")]` block on
  `subagent_worktree_factory` etc. at builder.rs:355-409). This is the
  cleanest option: a downstream consumer trimming features gets a
  compile error at the call site, which is unambiguous.
  (2) Add a `build()`-time normalisation pass that clears
  `config.enable_human_in_loop`/`enable_subagent`/`register_agent_dispatch_tool`
  when the corresponding `cfg!` is false, and emits a `tracing::warn!`.
  Option (1) is preferred because it surfaces the mistake at compile
  time.
- Regression validation: add a `#[cfg(not(feature = "human-loop"))]` test
  that asserts the builder method does not exist
  (`// compile_fail` style) or asserts the runtime flag is `false`.
- Validation reports: [V04](../validations/F-RCT-01/V04-01.md).

### F-RCT-01-P3-02: Project rules are duplicated in context after compression

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/mod.rs:326-382` — `new_inner` calls
    `build_system_prompt(&config)` (which prepends project rules via
    `inject_rules_with_root` at `:715-719`) and then stores the same
    full prompt as `canonical.system_prompt` AND stores the raw rules
    separately as `canonical.project_rules` (`:371`).
  - `echo-agent/echo-core/src/compression.rs:376-401` —
    `CanonicalContext::to_reinjection_messages` pushes a
    `[Canonical context — project rules restored]: ...` message even
    though the restored `canonical.system_prompt` already contains the
    same rules text.
  - Doc at `compression.rs:373-375` claims "the prompt is not represented
    twice" — true for the *system-prompt string* (it is never pushed
    into `msgs`), but the *rules content* ends up represented twice
    after compression.
- Reachability: construction → every ReactAgent that boots with
  `feature = "project-rules"` and `auto_project_rules = true` (the
  default) populates both fields. Compression fires when token budget is
  exceeded → `to_reinjection_messages` runs → duplication appears in
  context.
- Expected invariant: information already present in the restored system
  prompt should not be re-injected as a supplemental message.
- Observed behavior: after one compression cycle the LLM sees the
  project rules twice (once inside the system prompt, once as a
  `[Canonical context — project rules restored]: ...` message truncated
  to 2000 chars).
- Impact: bounded context bloat (≤ 2000 chars per compression cycle)
  and a small risk of LLM confusion from duplicated instructions. No
  correctness impact — both copies are the same content.
- Root cause: `CanonicalContext.project_rules` predates the decision to
  embed rules in the system prompt. The two mechanisms were designed for
  different sinks but ended up carrying the same payload.
- Direction: drop the `project_rules` branch from
  `to_reinjection_messages`, OR stop embedding rules in the system prompt
  and rely solely on canonical re-injection. The first option preserves
  the cache-stability argument in `mod.rs:703-720` (rules belong in the
  cache-stable system prefix) and is preferred. Either way, delete the
  now-redundant path.
- Regression validation: add a test that triggers compression and
  asserts the rules text appears exactly once in the resulting context.
- Validation reports: [V03](../validations/F-RCT-01/V03-01.md).

### F-RCT-01-P3-03: Five builder options bypass `AgentConfig` and write directly to `ReactAgent` internals

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/builder.rs:1023-1025` —
    `agent.run_store = Some(store);` (direct field write).
  - `builder.rs:1028-1030` — `agent.tool_execution_pipeline = Some(pipeline);`.
  - `builder.rs:1038-1040` — `agent.intent_router = Some(router);`.
  - `builder.rs:1043-1045` — `agent.memory.state_store = Some(store);`.
  - `builder.rs:1060-1062` —
    `agent.tools.intervention_callbacks = self.intervention_callbacks;`
    (direct list **replacement**, gated by `!is_empty()`).
- Reachability: every `build()` call writes these directly. None of the
  five has an `AgentConfig` counterpart field.
- Expected invariant: all builder options route through `AgentConfig`
  (the documented configuration layer) so that the config struct fully
  describes the agent's static configuration. The `ReactAgentBuilder`
  docstring at `builder.rs:22-25` says it returns "a `Box<dyn Agent>`
  abstraction", implying the builder is a thin wrapper around
  `AgentConfig` + `ReactAgent::new`; the five bypasses break that
  implication.
- Observed behavior: a caller that reads back `agent.config()` cannot
  see these five settings. There is no `AgentConfig::run_store`,
  `AgentConfig::tool_execution_pipeline`, etc. field.
- Impact: maintenance only. Today the bypasses are safe (each target
  field is initialised to a safe empty/`None`/empty-`Vec` in
  `new_inner`, so the assignment does not clobber state). The risk is
  forward-looking: if `new_inner` ever pre-populates any of these fields
  with a non-empty default, the direct assignment in `build()` would
  silently overwrite it. The `intervention_callbacks` replacement is the
  closest to that trap.
- Root cause: these options were added iteratively (run store, pipeline,
  intent router, state store, intervention callbacks, visibility horizon)
  and were wired with the shortest path to the agent rather than
  retrofitted into `AgentConfig`.
- Direction: either (a) add corresponding fields to `AgentConfig` and
  have `new_inner` consume them (preferred for uniformity), or (b)
  document the bypass in the `ReactAgentBuilder` docstring. (a) is
  invasive but small; (b) is cheap. No deletion target.
- Regression validation: after (a), `cargo test --workspace`; add a
  test that round-trips the option through `AgentConfig`.
- Validation reports: [V01](../validations/F-RCT-01/V01-01.md).

### F-RCT-01-P3-04: `enable_planning()` builder method is misnamed; `enable_task` docstring is misleading

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/builder.rs:334-337` —
    `pub fn enable_planning(mut self) -> Self { self.enable_task = true; self }`.
    The method name suggests it controls planning tools, but it actually
    toggles `enable_task`.
  - `echo-agent/src/agent/config.rs:63-65` — `enable_task` doc says:
    "Whether to enable task planning capability (plan/create_task/update_task
    tools)". But `new_inner` always registers `task_create`/`task_update`/
    `task_list` via `build_task_tools` regardless of `enable_task`
    (`mod.rs:398-400`, no `if config.enable_task` guard). `enable_task`
    only gates the **background** task tools (`SpawnBackgroundTaskTool`,
    `CheckTaskStatusTool`, `ListBackgroundTasksTool`) at `mod.rs:447-453`.
  - `echo-agent/src/agent/react/builder.rs:1170-1183` —
    `default_agent_uses_one_task_relation_api` confirms task_create/
    task_update/task_list are present even with the default config
    (`enable_task = false`).
- Reachability: every `ReactAgentBuilder::enable_planning()` caller.
- Expected invariant: builder method names match the runtime effect;
  field docstrings match the actual gating behaviour.
- Observed behavior: `enable_planning()` is a no-op for the basic task
  CRUD (always on) and only enables background-task spawning. A user
  reading `enable_planning` expects plan-mode behaviour; a user reading
  the `enable_task` doc expects the basic task tools to be gated.
- Impact: confusion only. The actual behaviour is correct (single task
  API, always available; background spawning opt-in). The naming/doc
  drift can lead downstream consumers to call `enable_planning()`
  unnecessarily, or to expect their custom `task_*` registration to be
  gated by `enable_task` when it is not.
- Root cause: terminology drift across iterations. `enable_task` once
  gated the task tools; the always-on design later won, but the
  docstring and the `enable_planning` alias were not refreshed.
- Direction: rename `enable_planning` to `enable_background_tasks` (or
  remove it and have callers use `enable_task(true)` directly), and
  update the `enable_task` docstring to: "Whether to enable background
  task spawning tools (`spawn_task`/`check_task_status`/
  `list_background_tasks`). The basic `task_create`/`task_update`/
  `task_list` tools are always registered."
- Regression validation: `cargo test --workspace`; grep consumers of
  `enable_planning` in both repositories and update them.
- Validation reports: [V01](../validations/F-RCT-01/V01-01.md),
  [V02](../validations/F-RCT-01/V02-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Builder option-to-runtime map (no orphans) | yes | passed | [V01-01](../validations/F-RCT-01/V01-01.md) |
| V02 | Single tool registry / duplicate registration search | yes | passed | [V02-01](../validations/F-RCT-01/V02-01.md) |
| V03 | Deterministic prompt assembly (system_prompt + CoT + project_rules + canonical) | yes | passed | [V03-01](../validations/F-RCT-01/V03-01.md) |
| V04 | Disabled-feature construction (silent-noop probe) | yes | **failed** | [V04-01](../validations/F-RCT-01/V04-01.md) |
| V05 | Historical-document drift | conditional (not applicable — no historical audit claims about builder/config assembly pre-exist in `AUDIT_REPORT.md` for this scope) | n/a | — |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `builder.rs:22-25` — "Provides a fluent API to configure and build an Agent" | current | V01 confirms all 50+ options map to runtime fields |
| `builder.rs:411-420` — "`register_agent_dispatch_tool` is independent of `enable_subagent`" | current | `mod.rs:478-497` registers the tool keyed only on `config.register_agent_dispatch_tool` |
| `tasks.rs:16-17` — "atomically selects the supplied store/policy adapter without exposing a second task API" | current | `ToolManager::register` uses `DashMap::insert` (replace semantics); `default_agent_uses_one_task_relation_api` test verifies single API |
| `compression.rs:373-375` — "the prompt is not represented twice" | partial drift | True for the system-prompt string; **false** for project_rules content, which is duplicated post-compression (F-RCT-01-P3-02) |
| `config.rs:63-65` — "`enable_task`: Whether to enable task planning capability (plan/create_task/update_task tools)" | stale | `enable_task` only gates background task tools; basic task CRUD is always registered (F-RCT-01-P3-04) |
| `mod.rs:703-705` — "Project rules … can stay in the system prompt since they don't change between requests" | current | Verified — rules are prepended in `build_system_prompt` |

## Coverage And Uncertainty

Inspected in full: `builder.rs`, `config.rs`, `capabilities.rs`,
`mod.rs:1-826` (construction + helpers), `tools.rs:55-66, 505-585`
(ToolManager core), `tasks.rs`, `project_rules.rs:180-218`,
`compression.rs:345-402`.

Not inspected (out of scope or deferred):

- `mod.rs:827+` (runtime LLM/client wiring beyond `set_llm_config`) —
  overlaps with F-LLM-01.
- `mod.rs:1607, 1628` — duplicate `HumanInLoop::new(provider)`
  registrations inside what appear to be test or runtime-setter blocks;
  not on the construction path. Quick scan only.
- Detailed comparison of `echo_tools::register_all_tools` vs
  `register_readonly_tools` tool inventories — F-EXT-01 owns this.
- The `react_smoke.rs` integration tests — not re-run; they exercise
  token-calibration and shell-safety paths not in this task's scope.

Environmental constraints:

- Probe in V04 was added and removed within a single session; final
  worktree state is clean (`git status` reports "nothing to commit,
  working tree clean").
- `cargo test -p echo_agent --no-default-features` and
  `cargo check -p echo_agent --no-default-features --locked` both pass;
  feature matrix beyond `human-loop`/`subagent` not re-run (F-FEAT-01
  owns the full matrix).

Uncertain claims:

- The exact cache-hit implication of prepending project rules to the
  system prompt (V03 §Deterministic ordering) depends on provider
  behaviour (Anthropic, DeepSeek, OpenAI); this task asserts only that
  the assembly is deterministic, not that the cache strategy is optimal.

## Handoff

Conclusions downstream tasks may rely on:

1. **Single tool registry confirmed.** F-RCT-02 (ReAct loop) and any
   tool-execution task can rely on `agent.tools.tool_manager` being the
   sole authority for tool lookups; no parallel registry exists.
2. **System-prompt section order is fixed.** Downstream tasks that need
   to know where project rules or CoT instructions appear can rely on
   the order documented in V03.
3. **`enable_task`/`enable_planning` are naming-only issues.** Behaviour
   is correct; the always-on task CRUD and opt-in background spawning
   are the real invariants.
4. **Feature-flag silent-noop is a real trap for feature-trimming
   consumers.** Any task that audits downstream consumer build configs
   (e.g. an EKO audit task) should flag combinations where the consumer
   calls `enable_human_in_loop()`/`enable_subagent()` without enabling
   the corresponding Cargo feature.

Reports they must read:

- This report (F-RCT-01) for the construction-path invariants.
- `tasks/F-CORE-01.md` for the `Agent` trait and event-envelope
  contract that the construction path produces.
- `tasks/F-API-01.md` for the facade-coherence rule (parallel access
  paths at the public-API level; this task is the construction-level
  analogue).
- `validations/F-RCT-01/V01-01.md` through `V04-01.md` for the
  per-claim evidence.

Conditions that make this report stale:

- Addition or removal of a `ReactAgentBuilder` field without updating
  V01-01's Category-A/B/C/D table.
- Changes to `ToolManager::register` semantics (e.g. switching from
  `DashMap::insert` to a non-replacing API) — would invalidate V02-01's
  "idempotent replacement" claim.
- Refactor of `build_system_prompt` or `CanonicalContext` — would
  invalidate V03-01.
- Addition of `#[cfg(feature = ...)]` gates to the builder methods
  named in F-RCT-01-P3-01 — would resolve that finding and require
  re-running V04.

Follow-up task IDs (no implementation in this review task):

- F-RCT-02 — owns the `set_system_prompt` canonical-drift observation
  noted in V03 §Deviations (runtime prompt mutation does not refresh
  `CanonicalContext`).
- F-EXT-01 — owns the detailed `register_all_tools`/`register_readonly_tools`
  inventory, which this task references but does not audit.
- F-FEAT-01 — owns the full feature-compile matrix; this task's V04
  validates only the `--no-default-features` slice plus `human-loop`/
  `subagent`.
- A future cleanup task could address F-RCT-01-P3-03 (config-layer
  bypasses) once the construction path is otherwise stable.
