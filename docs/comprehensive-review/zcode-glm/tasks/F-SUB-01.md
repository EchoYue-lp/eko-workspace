# F-SUB-01: Subagent definitions, registry, and prompt context

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Are Subagent identity, catalog snapshot, role prompts, history inheritance,
tool/permission selection, and results coherent?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/agent/subagent/types.rs` (960 lines) — `SubagentDefinition`
  (22 fields), `SubagentKind`, `ExecutionMode`, `ObservedIsolation`,
  `TeamSpec`, `SubagentOutcome`/`SubagentResult`/`SubagentStatus`, the
  `render_result_contract` / `parse_subagent_outcome` parser, and the
  UTF-8-safe bounding helpers.
- `echo-agent/src/agent/subagent/registry.rs` (689 lines) —
  `SubagentRegistry`, `AgentFactory` / `FnAgentFactory`, the six registration
  entry points, factory instantiation guard, and `RegisteredSubagent` view.
- `echo-agent/src/agent/subagent/prompt.rs` (321 lines) —
  `SubagentPromptCompiler` trait, `DefaultSubagentPromptCompiler`,
  `ContextTransferPolicy`, `filter_history`, `with_compiled_task`.
- `echo-agent/src/agent/subagent/context.rs` (468 lines) —
  `ContextInheritance` presets, `SubagentContext`, `from_parent`.
- `echo-agent/src/agent/subagent/context_builder.rs` (441 lines) —
  `ContextBuilder`, `SubagentOutput`, `OutputSchema`.
- `echo-agent/src/agent/subagent/events.rs` (469 lines) — `SubagentEvent`
  lifecycle enum, `SubagentEventBus`.
- `echo-agent/src/agent/subagent/builder.rs` (313 lines) —
  `SubagentBuilder` fluent API.
- `echo-agent/src/agent/subagent/mod.rs` (53 lines) — module facade.
- `echo-agent/src/agent/subagent/executor.rs:36-101, 105-135, 407-660,
  871-895, 992-1160, 1117-1145, 1455-1760` — `DispatchRequest`,
  `SubagentExecutorConfig`, `dispatch`, `dispatch_teammate`,
  `dispatch_team`, `compile_invocation`, `dispatch_sync`, `dispatch_fork`,
  `isolated_dispatch_agent`, `invocation_disabled_tools`.
- `echo-agent/src/tools/builtin/agent_dispatch.rs` (581 lines) —
  `AgentDispatchTool` (the `agent_tool` LLM entry), `ParentContextFactory`,
  `SubagentCatalogEntry`, `serialize_parent_result`, cancel/catalog handles.
- `echo-agent/src/agent/react/capabilities.rs:300-438` —
  `register_subagent_*`, `update_dispatch_catalog`,
  `sync_subagent_dispatch_catalog`, `unregister_subagent`.
- `echo-agent/src/agent/react/mod.rs:460-560` — `AgentDispatchTool`
  construction and `dispatch_catalog_handle` wiring.
- `echo-agent/echo-execution/src/tools.rs:55-66, 275-305, 480-499, 577` —
  `ToolManager` storage, `cached_definitions` cache,
  `invalidate_definition_cache`.

Cross-checks (lighter): `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:480-509`
(plugin → `SubagentDefinition` conversion), the `B-REF-01` task for the
Claude Code delegation pattern, and the existing `subagent::*` and
`agent_dispatch::*` test suites (52 + 11 tests, all passing).

## Out Of Scope

Deferred to named task IDs:

- Execution-mode lifecycle (Sync vs Fork vs Teammate vs Team concurrency,
  semaphore/worktree/workspace acquisition, timeout ownership, cancellation
  propagation) → **F-SUB-02**.
- Handoff / topology multi-agent coordination APIs → **F-MAG-01**.
- Team strategy pipeline (`dispatch_team` internals, manager plan→fan-out)
  → **F-SUB-02**.
- Hook registry / retry / verifier behaviour → a hooks-focused task.
- The full ReAct loop inside the subagent instance → **F-RCT-02**.
- Application-layer (EKO) subagent catalog hydration (plugin integrator,
  task-runtime subagent spawning) → the application task_runtime task.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full, via
  system reminder). Key constraints applied: Subagent-only terminology
  (no Worker), framework-vs-application layering, "first check if it already
  exists," dead-code cleanup (no backward-compat burden), UTF-8 safety,
  cross-repository boundary gate.
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-RCT-01.md` (in full).
  F-RCT-01 established the single-tool-registry invariant and the
  construction-time `register_agent_dispatch_tool` wiring that this task
  traces from the subagent side. Its finding F-RCT-01-P3-01 (feature-gated
  builder methods silently no-op) is the construction-side analogue of the
  dead-definition-field findings here.
- `docs/comprehensive-review/zcode-glm/tasks/B-REF-01.md` (referenced for
  C5: isolation-first delegation with bounded caps and the Claude Code
  single-tool / scoped-tools / single-message-return pattern; read via the
  task summary).

Historical documents treated as hypotheses:

- `types.rs:160-164` — `lightweight` field doc claims it makes the subagent
  share the parent's LLM client/ToolManager/GuardManager. Treated as a
  design intent; **code evidence shows it is unwired** (see
  F-SUB-01-P2-04).
- `prompt.rs:46-61` — `SubagentSystemPromptInput` doc claims it is
  "registration-time facts used to compile a cache-stable role system
  prompt." Treated as design intent; **code evidence shows `compile_system`
  is never called in production** (see F-SUB-01-P2-02).

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `SubagentDefinition`, `SubagentRegistry`, `AgentDispatchTool`, `SubagentPromptCompiler`, `SubagentContext`, `SubagentOutcome`, and the `SubagentEvent` lifecycle are generic agent-delegation machinery. Any `echo-agent` consumer (EKO, a headless CLI, a third-party reuser) needs them, and they correctly live in the `echo-agent` root crate. The `agent_tool` LLM entry follows the Claude Code single-tool pattern (one `agent_tool`, name+description catalog, scoped tools/perms, single-message return). |
| EKO product policy | None at this layer. The framework defines the definition shape, registry, compiler trait, and result contract; the application injects concrete compilers, factories, and isolation factories. `DefaultSubagentPromptCompiler` is the product-neutral fallback. No EKO-specific decision is baked into these types. |
| Adapter boundary | The compiler trait is the adapter boundary: product compilers may render `SubagentSystemPromptInput.environment` / `SubagentPromptInput.payload` / `constraints` into custom sections; the framework default compiler ignores environment/payload and renders constraints verbatim. Conversion is thin and the framework owns the terminal `## Result` envelope (`render_result_contract`). |
| Duplicate search | Searched names: `SubagentDefinition`, `SubagentRegistry`, `AgentDispatchTool`, `SubagentCatalogEntry`, `compile_system`, `compile_invocation`, `SubagentPromptCompiler`, `tool_filter`, `lightweight`, `inherit_history`, `inherit_tools`, `allowed_tools`, `SubagentOutput`, `SubagentOutcome`, `ContextBuilder`, `SubagentContext`, `ContextInheritance`, `SubagentKind::Custom`. Searched both `echo-agent` and `echo-agent-cli`. Result: one canonical registry and one canonical dispatch tool; one canonical result type (`SubagentOutcome`); but **three dead/duplicate surfaces** — `tool_filter` (definition field, never enforced), `SubagentOutput` (duplicate result type in `context_builder.rs`), and `ContextBuilder` (duplicate context constructor in `context_builder.rs`). See findings. |
| Migration deletion | No migration proposed in this review. The dead fields/types identified here are candidates for deletion per the AGENTS.md "code cleanup" rule, but deletion is a follow-up action, not part of this review task. |

## Current Path

Verified subagent definition→registration→dispatch→result call graph at
commit `9b0e0fa`:

```text
SubagentBuilder::new(name)                                 [builder.rs:34-58]
   ↓ (.description/.fork_mode/.tools/.inherit_history/...)
SubagentBuilder::build() → SubagentDefinition              [builder.rs:220-227]
   │
   ├─ ReactAgent::register_subagent_with_definition(def, agent)  [capabilities.rs:305-325]
   │      OR
   │  register_subagent_factory(def, factory)                    [capabilities.rs:360-374]
   │      OR
   │  register_subagent_definition(def)  (late binding)          [capabilities.rs:342-356]
   │      │
   │      ├─ SubagentRegistry::register_sync / register_factory_sync / register_definition_sync
   │      │      → definitions.write().insert(name, def)         [registry.rs:161-244]
   │      │      → agents.write().insert(name, Arc::new(agent))  (when instance supplied)
   │      │      → event_bus.emit(Registered)                    [registry.rs:153-155]
   │      │
   │      └─ update_dispatch_catalog(&def)                       [capabilities.rs:377-401]
   │             → catalog.write() → upsert by name, sort        [capabilities.rs:380-389]
   │             → tool_manager.invalidate_definition_cache()    [capabilities.rs:399]
   │
   ├─ LLM invokes agent_tool(agent_name, task, mode?, constraints?, background?)
   │      [agent_dispatch.rs:196-264]
   │
   ├─ AgentDispatchTool::dispatch_with_context
   │      ├─ delegation_policy_from_context (depth guard)        [agent_dispatch.rs:121-135]
   │      ├─ ParentContextFactory.build / build_with_inheritance [agent_dispatch.rs:35-55]
   │      │      → SubagentContext::from_parent(tools, msgs, store, inheritance)
   │      │                                          [context.rs:229-281]
   │      ├─ child_cancel_token (invocation > shared handle)     [agent_dispatch.rs:164-178]
   │      └─ SubagentExecutor::dispatch(req)                     [executor.rs:407-660]
   │             │
   │             ├─ registry.get(name) → definition + has_instance  [registry.rs:298-309]
   │             ├─ mode selection (def.execution_mode or override) [executor.rs:449-453]
   │             │
   │             ├─ compile_invocation(req, mode, inherit_history) [executor.rs:1117-1145]
   │             │      → prompt_compiler.compile_invocation(...)   [prompt.rs:100]
   │             │           → filter_history(msgs, limit)          [prompt.rs:149-187]
   │             │           → task_input (+ [constraints])          [prompt.rs:131-140]
   │             │
   │             ├─ isolated_dispatch_agent(name)                [executor.rs:1455-1466]
   │             │      → registry.create_fresh_agent / get_agent  [registry.rs:318-419]
   │             │
   │             ├─ dispatch_sync / dispatch_fork / dispatch_teammate / dispatch_team
   │             │      [executor.rs:1469, 1554, 871, 992]
   │             │
   │             └─ execute_agent_streaming → AgentEvent stream  [executor.rs:1148-1162]
   │                    → SubagentResult { output, outcome, ... }
   │
   └─ Result return
          ├─ sync: serialize_parent_result(outcome) → ToolResult::success(json)
          │      [agent_dispatch.rs:336-344]  (single message to parent LLM)
          ├─ background: ToolResult::success({status, execution_id, agent_name})
          │      [agent_dispatch.rs:309-316]  (full outcome arrives via event)
          └─ event: DispatchCompleted/Failed/Cancelled { result: SubagentOutcome, .. }
                 [executor.rs:584-608, 767; events.rs:63-113]
```

Key invariants verified by this graph (full evidence in V01-V04):

- **Single registry, single dispatch tool, single result contract.**
  `SubagentRegistry` is the sole identity authority; `AgentDispatchTool` is
  the sole LLM-facing entry; `SubagentOutcome` is the sole parent-facing
  result contract. No parallel worker/subagent identity layer exists.
  Terminology is Subagent-only across the whole module.
- **Catalog is a projection, not an authority.** `SubagentCatalogEntry`
  carries only name+description; dispatch resolves through the registry
  (`executor.registry().get`), never through the catalog. Catalog staleness
  degrades the LLM-visible enum, never dispatch correctness (V02).
- **Result is single-message, runtime-owned, UTF-8-safe.** Sync dispatch
  returns one `ToolResult::success(json)`; the model cannot set terminal
  status; all fields are char-bounded; artifacts are hydrated with sha256
  (V04).
- **History inheritance is bounded and filtered.** Fork inherits ≤ 2 recent
  turns (mode preset), sanitized of tool/reasoning/projection; Sync/Teammate/
  Team default to Fresh (no transcript) but still carry `parent_goal` (latest
  user request) (V03).

## Findings

### F-SUB-01-P2-01: `SubagentDefinition.tool_filter` is declared and settable but never enforced at dispatch

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/types.rs:143-144` — `pub tool_filter:
    Option<Vec<String>>` with doc "Restrict available tools by name (None =
    inherit all from parent)."
  - `echo-agent/src/agent/subagent/builder.rs:125-128` — `.tools(vec!)`
    populates `tool_filter`.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:495` —
    plugin adapter hardcodes `tool_filter: None`.
  - Whole-repo grep for `tool_filter` read sites (excluding declaration,
    builder setter, and builder test assertion at `builder.rs:269`):
    **zero production read sites**. The field is written but never consulted.
- Reachability: `SubagentBuilder::tools(["search"])` → `definition.tool_filter`
  → `register_subagent_with_definition` → stored in registry → **never read
  by `dispatch_sync`/`dispatch_fork`/`dispatch_teammate`/`dispatch_team`**.
- Expected invariant: a definition field documented as "Restrict available
  tools by name" should restrict the tools the dispatched subagent can call.
- Observed behavior: the subagent's tool surface is determined entirely by
  the pre-built agent instance's own `ToolManager`. The definition's
  `tool_filter` is inert. Tool restriction at dispatch actually happens
  through two *separate* mechanisms that do not read `tool_filter`:
  (1) `ContextInheritance.inherit_tools` (`context.rs:236-252`), consumed by
  `SubagentContext::from_parent`, which filters the *parent tool definitions
  copied into the subagent context* — but `ParentContextFactory.build`
  (`agent_dispatch.rs:51-54`) uses `ContextInheritance::for_mode(mode)`,
  whose presets all set `inherit_tools: None`, so this never filters in the
  default dispatch path; (2) `SubagentContext.allowed_tools`
  (`context_builder.rs:127, 147-154`), consumed by
  `invocation_disabled_tools` (`executor.rs:120-135, 1672-1676, 1744-1745`)
  — but only in `dispatch_fork`, and `ParentContextFactory` never sets
  `allowed_tools` (`from_parent` hardcodes it to `None` at `context.rs:277`).
  So three mechanisms exist, none wired to the definition field, and only
  the Fork-only `invocation_disabled_tools` path actually disables tools at
  invocation time (and only when a caller manually constructs a context
  with `allowed_tools`).
- Impact: a framework consumer calling
  `SubagentBuilder::new("writer").tools(["write_file","shell"]).build()`
  reasonably expects the dispatched subagent to be restricted to those
  tools. It is not — the subagent gets whatever tools its pre-built agent
  instance registered. This is a misleading public API and a real
  capability boundary that silently fails to enforce.
- Root cause: `tool_filter` predates the `ContextInheritance` /
  `allowed_tools` / `invocation_disabled_tools` mechanisms. The field was
  added as declarative metadata; enforcement was later implemented through
  separate context-construction paths, and the definition field was never
  wired into them.
- Direction: either (a) delete `tool_filter` and document that tool scoping
  is done via `ContextInheritance.inherit_tools` / injected factory agent
  construction (preferred per AGENTS.md "delete dead code" rule — the field
  is dead and the replacement mechanisms already exist); or (b) wire
  `tool_filter` into `dispatch_sync`/`dispatch_fork` by feeding it into
  `invocation_disabled_tools` (which would require teaching
  `ParentContextFactory` to set `allowed_tools` from the definition). Option
  (a) is cheaper and aligns with the cleanup rule; option (b) restores the
  documented behaviour. Either way, the field and the docstring must agree.
- Regression validation: after (a), `cargo test --workspace`; grep consumers
  of `.tools(` on `SubagentBuilder` in both repos (only `builder.rs` test
  and `plugin_components.rs` set it). After (b), add a test that dispatches
  a subagent with `tool_filter = ["read_file"]` and asserts
  `write_file` is absent from the dispatched agent's tool surface in Sync
  mode.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md),
  [V03](../validations/F-SUB-01/V03-01.md).

### F-SUB-01-P2-02: Registration-time system-prompt compiler (`compile_system`) is never called in production

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/prompt.rs:96-101` —
    `SubagentPromptCompiler::compile_system` trait method with doc
    "Registration-time compiler result."
  - `echo-agent/src/agent/subagent/prompt.rs:107-118` —
    `DefaultSubagentPromptCompiler::compile_system` returns
    `input.role_prompt` verbatim.
  - `echo-agent/src/agent/subagent/prompt.rs:46-61` —
    `SubagentSystemPromptInput` (name, description, role_prompt, readonly,
    can_delegate, isolation, environment) documented as "Registration-time
    facts used to compile a cache-stable role system prompt."
  - Whole-repo grep `grep -rn "compile_system" echo-agent/src/ echo-*/src/
    --include="*.rs"` returns only: the trait declaration (`prompt.rs:97`),
    the default impl (`prompt.rs:108`), and a test stub
    (`executor.rs:1920`). **Zero production call sites.**
  - The executor's only compiler call is `compile_invocation`
    (`executor.rs:1133-1144`); `compile_system` is never invoked at
    registration, first-dispatch, or any other point.
- Reachability: `SubagentDefinition.system_prompt` → stored in registry →
  never reaches `compile_system` → never becomes the agent's system prompt.
  The subagent's actual system prompt is whatever the pre-built
  `Box<dyn Agent>` instance carries in its `AgentConfig.system_prompt`.
- Expected invariant: a trait method named `compile_system` with a
  dedicated input type (`SubagentSystemPromptInput`) and doc "Registration-
  time facts used to compile a cache-stable role system prompt" should be
  called when a subagent is registered or first dispatched, and its output
  should become (or influence) the subagent's system prompt.
- Observed behavior: the entire registration-time compilation path — trait
  method, `SubagentSystemPromptInput`, `CompiledSubagentSystemPrompt`, and
  the `environment` grounding field — is inert. Product compilers that
  implement `compile_system` to inject role prompts or static environment
  facts see no effect. The definition's `system_prompt` field is metadata
  only.
- Impact: framework consumers who implement a custom
  `SubagentPromptCompiler` expecting `compile_system` to take effect (the
  trait offers it as a first-class method) get silently ignored output.
  The `environment` field — explicitly designed for "OS/arch facts that do
  not change per dispatch" (`prompt.rs:55-60`) — has no consumer. This is a
  designed-but-unwired abstraction, the same class of issue as
  F-RCT-01-P3-01 (feature-gated methods that silently no-op).
- Root cause: `compile_invocation` was wired into the executor; `compile_system`
  was not. The split was likely intended (registration-time vs dispatch-time)
  but only the dispatch-time half was connected. The definition's
  `system_prompt` is carried as data, but no production code compiles it
  into the agent instance's prompt.
- Direction: either (a) wire `compile_system` into the dispatch path by
  calling it once per dispatch (or caching per agent_name) and feeding its
  output as the invocation's system-prompt override; or (b) delete
  `compile_system`, `SubagentSystemPromptInput`,
  `CompiledSubagentSystemPrompt`, and the `environment` field, documenting
  that the subagent's system prompt is owned by the injected agent instance
  (factory author's responsibility). Option (b) is preferred unless a
  concrete consumer needs registration-time compilation — the current state
  is a half-built abstraction that misleads implementers. If keeping (a),
  the `SubagentDefinition.system_prompt` field should flow through
  `compile_system` so the definition actually drives the prompt.
- Regression validation: after (b), `cargo test --workspace`; remove the
  `PrefixPromptCompiler::compile_system` test stub. After (a), add a test
  that registers a subagent with a custom compiler and asserts the
  dispatched agent's system prompt contains the compiled role section.
- Validation reports: [V03](../validations/F-SUB-01/V03-01.md).

### F-SUB-01-P2-03: `context_builder::SubagentOutput` is a duplicate unused result type; `ContextBuilder` is an unused convenience constructor

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/context_builder.rs:189-340` —
    `SubagentOutput { summary, findings, evidence, files_read,
    recommendations, blockers, confidence }` with `to_json(schema)` — 150
    lines of a structured result type with its own `OutputSchema`-driven
    JSON renderer.
  - `echo-agent/src/agent/subagent/context_builder.rs:15-181` —
    `ContextBuilder` fluent API producing `SubagentContext` via
    `build_scoped_context`.
  - Whole-repo grep `SubagentOutput` (excluding declaration, `impl`, the
    `pub use` re-export in `mod.rs:26`, and the two tests at
    `context_builder.rs:406, 421`): **zero production callers** across
    `echo-agent` and `echo-agent-cli`.
  - Whole-repo grep `ContextBuilder` (excluding declaration, `impl`,
    `Default`, `pub use`, `pub mod`, and the three tests at
    `context_builder.rs:348, 358, 395`): **zero production callers** across
    both repos.
  - The live result type is `SubagentOutcome` (`types.rs:329-381`), returned
    by `parse_subagent_outcome` and serialized by
    `serialize_parent_result` (`agent_dispatch.rs:13-17`). The live context
    constructor is `SubagentContext::from_parent` (`context.rs:229-281`)
    invoked by `ParentContextFactory` (`agent_dispatch.rs:33-55`).
- Reachability: neither `SubagentOutput` nor `ContextBuilder` appears on any
  dispatch, registration, or event path.
- Expected invariant: there should be one structured result contract
  (`SubagentOutcome`) and one primary context constructor
  (`from_parent`/`ParentContextFactory`). Public API types that look like
  authorities but have no callers mislead consumers about which type to use.
- Observed behavior: a framework consumer reading `mod.rs:26` sees both
  `SubagentOutput` and `SubagentOutcome` re-exported; the natural assumption
  is that `SubagentOutput` (with its richer `findings`/`evidence`/
  `confidence` fields) is the result type. It is not — dispatch produces
  `SubagentOutcome`. Similarly, `ContextBuilder` looks like the intended way
  to build a `SubagentContext`, but the live path uses `from_parent`.
- Impact: maintainability and API clarity. ~290 lines of public framework
  API (`SubagentOutput` + `ContextBuilder` + `OutputSchema`) that no live
  code exercises — tests cover them in isolation, but they never run in a
  real dispatch. Risk of drift: `SubagentOutput` could evolve without anyone
  noticing it is unused.
- Root cause: `context_builder.rs` is an earlier iteration of the scoped-
  context/result design. The dispatch path later settled on
  `SubagentContext::from_parent` + `SubagentOutcome` (with its
  `render_result_contract` / `parse_subagent_outcome` parser and runtime-
  owned status), and the older `SubagentOutput`/`ContextBuilder` was not
  removed. Per AGENTS.md "code cleanup": "If you find two systems doing the
  same thing, delete the old one."
- Direction: delete `SubagentOutput`, `OutputSchema` (the context_builder
  one — note `context::OutputSchema` is the same type, re-used), and
  `ContextBuilder`. Keep `SubagentContext::from_parent` and `SubagentOutcome`
  as the sole authorities. If external consumers exist (none found in this
  repo), they should migrate. Per AGENTS.md, no backward-compat burden.
- Regression validation: `cargo test --workspace`; remove the three
  `context_builder` tests that exercise `ContextBuilder`/`SubagentOutput`.
  The `context::OutputSchema` type should be retained if anything else uses
  it (grep shows only `context_builder` does).
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md).

### F-SUB-01-P2-04: `SubagentDefinition.lightweight` is a dead field (no setter, no reader)

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/types.rs:160-164` — `pub lightweight: bool`
    with doc: "Whether this sub-agent uses the lightweight (infrastructure-
    sharing) mode. When true, the sub-agent shares the parent's LLM client,
    ToolManager, and GuardManager instead of creating new instances."
  - `echo-agent/src/agent/subagent/types.rs:221` and
    `builder.rs:51` — initialized to `false`.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:503` —
    plugin adapter hardcodes `lightweight: false`.
  - Whole-repo grep `\.lightweight` / `lightweight:` read sites (excluding
    declarations and initializers): **zero read sites**. No
    `SubagentBuilder::lightweight()` setter exists.
- Reachability: the field is written at construction (always `false`) and
  never read.
- Expected invariant: a definition field documenting a behaviour
  ("shares parent's LLM client/ToolManager/GuardManager") should have a
  builder setter and a dispatch-time consumer.
- Observed behavior: no way to set `lightweight = true` through the builder;
  even if set directly on the struct, no dispatch path checks it. The
  infrastructure-sharing mode it describes does not exist in code.
- Impact: API clutter and a false promise in the docstring. A consumer
  reading the field doc believes a lightweight mode exists; it does not.
- Root cause: the field was added for a planned lightweight mode that was
  never implemented. The isolated-dispatch design (fresh agent per dispatch
  via `isolated_dispatch_agent`, `executor.rs:1455-1466`) is the opposite of
  infrastructure-sharing.
- Direction: delete the `lightweight` field, its initializer, and the
  plugin-adapter line. If lightweight mode is ever needed, add it back with
  a setter and a real consumer at that time (YAGNI).
- Regression validation: `cargo test --workspace --all-features`; update
  `plugin_components.rs:503`.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md).

### F-SUB-01-P3-01: `SubagentKind::Custom { path }` has no `.md` definition loader

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/types.rs:84-89` — `SubagentKind::Custom
    { path: PathBuf }` with doc "Loaded from a `.md` definition file
    (similar to skills)."
  - `echo-agent/src/agent/subagent/builder.rs:73-76` — `.custom(path)` sets
    the variant.
  - Whole-repo grep for a loader that opens `path`: **none**. No code reads
    the file, parses frontmatter, or constructs a definition from it.
- Reachability: `.custom("agents/researcher.md")` stores the path; the path
  is never opened.
- Expected invariant: a variant documented as "Loaded from a `.md`
  definition file" should be loaded.
- Observed behavior: the variant is pure metadata; the path is stored but
  unused. Contrast with skills, which have a real loader.
- Impact: low. A consumer using `.custom(path)` gets a definition with the
  variant set but no loaded content — silently incomplete. The other
  fields (system_prompt, tools, etc.) must still be set programmatically.
- Root cause: the loader was planned ("similar to skills") but not built.
- Direction: either implement the loader (parse the `.md` frontmatter into
  the definition fields, mirroring the skill loader), or delete the variant
  and the `.custom()` builder method until the feature is actually needed.
  Given that skills already have a loader, a subagent `.md` loader is a
  reasonable feature — but per YAGNI, delete until a concrete consumer
  arrives.
- Regression validation: `cargo test --workspace`; update the
  `test_builder_custom_kind` test.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md).

### F-SUB-01-P3-02: `inherit_history` has inconsistent `Some(0)` semantics across its two consumers

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/types.rs:149-151` —
    `SubagentDefinition.inherit_history` doc: "`None` = don't inherit,
    `Some(0)` = inherit all, `Some(n)` = last n messages."
  - `echo-agent/src/agent/subagent/prompt.rs:182-186` — `filter_history`:
    `Some(0)` is filtered out by `.filter(|max| *max > 0)`, so `Some(0)` =
    no limit = inherit all (matches the doc).
  - `echo-agent/src/agent/subagent/context.rs:254-260` —
    `SubagentContext::from_parent`: `Some(n)` →
    `all_messages.get(len.saturating_sub(n)..)`. For `n == 0`:
    `saturating_sub(0) == len`, so `get(len..)` is empty → `Some(0)` =
    inherit nothing (contradicts the doc).
  - `echo-agent/src/agent/subagent/context.rs:40-42` —
    `ContextInheritance.inherit_history` has the same field name but its
    doc only says "Inherit recent N messages. `None` = don't inherit" — no
    `Some(0)` semantics documented.
- Reachability: both consumers are on the Fork dispatch path. The
  definition's `inherit_history` feeds `filter_history` (pass 2, via
  `compile_invocation`); `ContextInheritance.inherit_history` feeds
  `from_parent` (pass 1, via `ParentContextFactory`).
- Expected invariant: a field named `inherit_history` should have one
  documented `Some(0)` semantics shared by all consumers.
- Observed behavior: `Some(0)` means "all" in one consumer and "nothing" in
  the other. A consumer setting `definition.inherit_history = Some(0)`
  expecting "inherit all" (per the definition's own doc) gets the
  `filter_history` behaviour (all, after the mode preset already sliced to
  2). A consumer setting `ContextInheritance.inherit_history = Some(0)`
  gets empty.
- Impact: low in practice (no caller sets `Some(0)` today — the Fork preset
  uses `Some(2)`, Sync uses `None`). But the inconsistency is a latent trap
  and the `SubagentDefinition` doc is wrong for the `from_parent` consumer.
- Root cause: two separate filtering implementations were written
  independently; the `Some(0)` edge was handled differently in each.
- Direction: pick one semantics (recommend "Some(0) = inherit all," matching
  the definition doc and `filter_history`), and make `from_parent` treat
  `Some(0)` as `None` (no limit). Update `ContextInheritance.inherit_history`
  doc to match. Alternatively, simplify to `Option<NonZeroUsize>`.
- Regression validation: add a test that `from_parent` with
  `inherit_history = Some(0)` returns all messages (after the change).
- Validation reports: [V03](../validations/F-SUB-01/V03-01.md).

### F-SUB-01-P3-03: Sync dispatch skips the tool-allowlist enforcement that Fork applies

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/subagent/executor.rs:1495-1499` — `dispatch_sync`
    builds `AgentInvocationContext { runtime, history,
    ..Default::default() }`. `disabled_tools` defaults to `None`.
  - `echo-agent/src/agent/subagent/executor.rs:1672-1676, 1744-1745` —
    `dispatch_fork` reads `req.parent_context.allowed_tools`, computes
    `invocation_disabled_tools(agent.tool_names(), allowed)`, and sets
    `invocation.disabled_tools`.
  - `echo-agent/src/tools/builtin/agent_dispatch.rs:35-55` —
    `ParentContextFactory.build_with_inheritance` calls `from_parent`, which
    hardcodes `allowed_tools: None` (`context.rs:277`). So even Fork's
    allowlist is inert in the default `agent_tool` path — but the asymmetry
    between Sync and Fork remains.
- Reachability: any Sync dispatch with a `parent_context` carrying
  `allowed_tools` (currently none, since `ParentContextFactory` never sets
  it) would not be enforced. The asymmetry is structural.
- Expected invariant: tool allowlisting should be mode-independent — a
  subagent restricted to `[read_file]` should be restricted whether
  dispatched Sync or Fork.
- Observed behavior: Fork honours `allowed_tools`; Sync ignores it.
- Impact: low today (no caller sets `allowed_tools`), but a future caller
  that constructs a `DispatchRequest.parent_context` with `allowed_tools`
  and dispatches Sync would get an unrestricted subagent. The asymmetry is a
  latent trap.
- Root cause: `dispatch_sync` was written before
  `invocation_disabled_tools`; the allowlist was added only to the Fork
  path.
- Direction: extract the `invocation_disabled_tools` computation into the
  shared `compile_invocation` path or apply it in `dispatch_sync` the same
  way `dispatch_fork` does.
- Regression validation: add a test that dispatches a Sync subagent with
  `parent_context.allowed_tools = ["read_file"]` and asserts `write_file`
  is disabled.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md),
  [V03](../validations/F-SUB-01/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition-to-registration trace + duplicate/Worker search | yes | passed | [V01-01](../validations/F-SUB-01/V01-01.md) |
| V02 | Catalog route validation (schema enum + registry resolution) | yes | passed | [V02-01](../validations/F-SUB-01/V02-01.md) |
| V03 | Prompt envelope/cardinality (compile_system + filter_history + inheritance) | yes | passed | [V03-01](../validations/F-SUB-01/V03-01.md) |
| V04 | Result protocol round-trip (executable: parse/serialize/hydrate/UTF-8) | yes | passed | [V04-01](../validations/F-SUB-01/V04-01.md) |
| V05 | Historical-document drift | conditional (not applicable — no prior `AUDIT_REPORT.md` claims about the subagent definition/registry/prompt surface pre-exist for this scope) | n/a | — |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `types.rs:160-164` — `lightweight` "shares the parent's LLM client, ToolManager, and GuardManager" | stale | Field is dead: no setter, no reader (F-SUB-01-P2-04) |
| `types.rs:84-89` — `SubagentKind::Custom` "Loaded from a `.md` definition file (similar to skills)" | stale | No loader exists (F-SUB-01-P3-01) |
| `prompt.rs:46-61` — `SubagentSystemPromptInput` "Registration-time facts used to compile a cache-stable role system prompt" | stale | `compile_system` has zero production callers (F-SUB-01-P2-02) |
| `types.rs:143-144` — `tool_filter` "Restrict available tools by name" | stale | Never enforced at dispatch (F-SUB-01-P2-01) |
| `types.rs:149-151` — `inherit_history` "`Some(0)` = inherit all" | partial drift | True for `filter_history`; false for `from_parent` (F-SUB-01-P3-02) |
| `context.rs:26-27` — "Parent system prompts are never transferred as user text" | current | `from_parent` carries only `parent_goal` (latest user request), not the system prompt |
| `agent_dispatch.rs:363-377` — `agent_tool` description "Use only agent_name values listed in the schema" | current | V02 confirms the enum-constrained schema |
| `events.rs:63-113` — DispatchCompleted/Failed/Cancelled share one `SubagentOutcome` shape | current | V04 confirms identical contract across terminal events |
| `registry.rs:1-4` — "Wraps the existing `SubAgentMap` with declarative definitions, factory support, and lifecycle events" | current | V01 confirms the `SubAgentMap`-compatible `agents` map plus `definitions`/`factories` |
| `AGENTS.md` — "Only Subagent, no Worker" | current | Zero `Worker`/`worker_` hits in `echo-agent/src/agent/subagent/` |

## Coverage And Uncertainty

Inspected in full: `types.rs`, `registry.rs`, `prompt.rs`, `context.rs`,
`context_builder.rs`, `events.rs`, `builder.rs`, `mod.rs`,
`agent_dispatch.rs`, `capabilities.rs:300-438`, `mod.rs:460-560`,
`echo-execution/src/tools.rs:55-66, 275-305, 480-499, 577`.

Inspected partially (relevant slices only):
- `executor.rs` (151K lines) — read `DispatchRequest` (36-101),
  `invocation_disabled_tools` (118-135), `dispatch` (407-660),
  `dispatch_teammate` (871-895), `dispatch_team` (992-1095),
  `compile_invocation` (1117-1145), `execute_agent_streaming` entry
  (1148-1230), `isolated_dispatch_agent` (1455-1466), `dispatch_sync`
  (1469-1551), `dispatch_fork` entry + allowlist (1554-1600, 1672-1760).
  Did not read every line of `execute_agent_streaming`'s 600-line event
  loop — F-SUB-02 owns the streaming/event-detail behaviour.
- `team/` module — read only the `TeamSpec`/`TeamStrategy` surface in
  `types.rs` and `mod.rs`; F-SUB-02 owns team lifecycle.
- `hooks.rs`, `isolated.rs`, `workspace.rs`, `worktree.rs`, `usage.rs` —
  read only the re-exports in `mod.rs`; F-SUB-02 owns isolation and hooks.

Not inspected (out of scope):
- The application-layer subagent wiring in `echo-agent-cli` (plugin
  integrator, task-runtime subagent spawning, EKO DomainProfile). Only
  `plugin_components.rs:480-509` was read to confirm the plugin→definition
  conversion.
- The Tauri bridge's consumption of `SubagentEvent` for frontend rendering.

Environmental constraints:
- All 52 `agent::subagent::*` unit tests and 11 `agent_dispatch::*` /
  react-dispatch tests pass (`cargo test --lib -p echo_agent --features
  subagent`). Worktree state clean.
- The feature matrix beyond `subagent` was not re-run (F-FEAT-01 owns it).
- No probe was added/removed — all validations are read-only or use
  pre-existing tests.

Uncertain claims:
- Whether any external (out-of-repo) `echo-agent` consumer relies on
  `SubagentOutput`, `ContextBuilder`, `tool_filter`, or `compile_system`.
  Per AGENTS.md, the framework is not echo-agent-cli's private library, so
  these pub APIs might have unknown consumers. The findings are framed as
  "dead within this workspace" + "delete or wire," with the AGENTS.md
  default (retain uncertain pub API) noted where relevant — but the
  evidence here (designed-but-unwired, with live replacements already
  present) supports either deletion or completion, not silent retention.

## Handoff

Conclusions downstream tasks may rely on:

1. **Single registry, single dispatch tool, single result contract
   confirmed.** F-SUB-02 (execution modes), F-MAG-01 (handoff/topology), and
   any tool-execution task can rely on `SubagentRegistry` being the sole
   identity authority, `AgentDispatchTool` being the sole LLM entry, and
   `SubagentOutcome` being the sole parent-facing result. No parallel worker
   layer exists.
2. **Catalog is a per-agent projection of the registry.** Multi-agent
   shared-registry deployments must call `sync_subagent_dispatch_catalog`
   per agent. The catalog is name+description only; dispatch resolves
   through the registry.
3. **Result protocol is sound and Claude-Code-aligned.** Single-message JSON
   return, runtime-owned status, UTF-8-safe bounding, hydrated artifacts,
   model self-certification blocked. F-SUB-02 can assume this contract holds
   across modes and focus on lifecycle/concurrency.
4. **Four definition-level surfaces are dead/duplicate.**
   `tool_filter` (F-SUB-01-P2-01), `compile_system` (F-SUB-01-P2-02),
   `SubagentOutput`/`ContextBuilder` (F-SUB-01-P2-03), `lightweight`
   (F-SUB-01-P2-04). Any downstream task that assumes these work should be
   disabused: tool scoping is done via `ContextInheritance`/factory
   construction; the system prompt comes from the agent instance; the
   result type is `SubagentOutcome`; lightweight mode does not exist.
5. **History inheritance is bounded (≤ 2 for Fork, 0 for others).** The
   `Some(0)` inconsistency is documented but not currently triggered.

Reports they must read:

- This report (F-SUB-01) for the definition/registry/catalog/prompt/result
  invariants and the four dead surfaces.
- `tasks/F-RCT-01.md` for the construction-path single-tool-registry
  invariant and the `register_agent_dispatch_tool` wiring (this task is the
  runtime-side complement).
- `validations/F-SUB-01/V01-01.md` through `V04-01.md` for per-claim
  evidence.

Conditions that make this report stale:

- Wiring `tool_filter` into the dispatch path — resolves F-SUB-01-P2-01,
  requires re-running V01/V03.
- Wiring `compile_system` into registration/dispatch — resolves
  F-SUB-01-P2-02, requires re-running V03.
- Deleting `SubagentOutput`/`ContextBuilder`/`lightweight`/`SubagentKind::Custom`
  — resolves F-SUB-01-P2-03/P2-04/P3-01, requires re-running V01.
- Changes to `filter_history` or `from_parent` `Some(0)` handling —
  invalidates F-SUB-01-P3-02, requires re-running V03.
- Adding tool-allowlist enforcement to `dispatch_sync` — resolves
  F-SUB-01-P3-03, requires re-running V01/V03.

Follow-up task IDs (no implementation in this review task):

- **F-SUB-02** — owns execution-mode lifecycle (Sync/Fork/Teammate/Team),
  timeout ownership, cancellation propagation, and team partial-failure
  cleanup. This task established that the result contract and identity
  model are sound; F-SUB-02 validates that the four modes share one
  lifecycle without detached execution.
- **F-MAG-01** — owns handoff/topology coherence. This task confirmed no
  parallel identity authority exists in the subagent module; F-MAG-01
  checks whether handoff/topology APIs create overlapping routing or
  ownership.
- A future cleanup task could address the four dead surfaces
  (F-SUB-01-P2-01 through P2-04) and the two minor inconsistencies
  (F-SUB-01-P3-01 through P3-03) once the execution-mode work in F-SUB-02
  settles.
