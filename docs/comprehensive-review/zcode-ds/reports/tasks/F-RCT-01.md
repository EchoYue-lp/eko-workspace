# F-RCT-01: ReAct construction and canonical prompt assembly

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Does builder/config assembly produce a deterministic Agent with one tool
registry, correct instructions, budgets, hooks, and project rules?

## Scope

- `echo-agent/src/agent/react/builder.rs` (full read, 1227 lines),
  `echo-agent/src/agent/config.rs` (full read, 1214 lines),
  `echo-agent/src/agent/react/mod.rs` (construction path: `new_inner`
  :322-583, `build_system_prompt` :676-723, `register_feature_gated_tools`
  :740-748, `setup_memory_store` :750-784, setters :870-1444, 2031-2080),
  `echo-agent/src/agent/react/capabilities.rs` (tool/skill registration),
  `echo-agent/src/agent/snapshot.rs` (`tools_for_llm` :269-288, `available`
  :227-236, snapshot config propagation :96-155).
- Prompt assembly: `echo-core/src/project_rules.rs` (full read),
  `echo-core/src/compression.rs` (`CanonicalContext` :350-402),
  `echo-state/src/compression/mod.rs` (`reinject_canonical_context` :877-929
  and call sites :1071, 1144, 1203, 1525-1530).
- Registry: `echo-execution/src/tools.rs` (register :529-536, sorted
  `get_openai_tools` :281-307, `list_tools` :549-553, `ToolSearchTool`
  :125-133, :188-190).
- EKO construction callers: `echo-agent-cli/echo-agent-app-core/src/infra.rs`
  (builder usages, set_plan_mode sites), `agent_pool.rs` (set_tool_manager
  site), `src/tasks.rs` (register_task_tools).
- Executed tests: `cargo test -p echo_agent --lib --locked react::builder`
  and `agent::config`.

## Out Of Scope

- Non-streaming/streaming loop bodies and terminal ownership → F-RCT-02,
  F-RCT-03 (only `max_iterations`/`run_budget` consumption anchors checked).
- Tool batch execution internals → F-RCT-04, F-EXT-01 (tool contract already
  reviewed; P1-01/P1-02 cross-referenced only).
- Context budget arithmetic and provider window mapping → F-CTX-01.
- Steer/snapshot/resume → F-RCT-05 (canonical re-injection impact noted in
  handoff only).
- Skill/plugin/MCP discovery internals (projection mechanics sampled for the
  duplication question only).
- Real end-to-end dynamic runs (read-only task; no fixtures executed).

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, layering, Subagent terminology),
  shared `REPORTING.md`, `TASKS.md` (F-RCT-01 card), `zcode-ds/README.md`,
  report templates.
- Dependency task reports read: zcode-ds `F-CORE-01` (complete),
  `F-API-01` (complete), `F-EXT-01` (complete — required for V02 and V05).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md`
  claims quoted inside F-EXT-01 (plan-mode read-only filtering; M13 task-API
  unification) — classified in the Historical Claim Status section.

## Layering Decision

- Generic mechanism (framework): builder/`AgentConfig` assembly, the single
  `ToolManager` per agent, `ToolSearchTool`, the `echo_core::project_rules`
  discovery authority, `CanonicalContext` re-injection, sorted deterministic
  tool schema — all correctly placed in `echo_agent`/`echo_core`/
  `echo_state`/`echo_execution`.
- EKO product policy (application): AgentPool shared-manager pattern
  (`agent_pool.rs:96,127,882`), writer/read-only subagent builders with
  `set_plan_mode(true)` (`infra.rs:963,1040`) — the application wiring layer
  owns F-EXT-01-P1-01's fix.
- Adapter boundary: none new; no repository movement is recommended by this
  task, so the boundary gate table is satisfied by the statements above plus
  the duplicate-search results below.
- Duplicate-search terms (both repositories): `ToolManager`, `set_tool_manager`,
  `add_tool`, `add_tools`, `register_tool`, `register_tools`, `allowed_tools`,
  `plan_mode`, `set_plan_mode`, `_reasoning_effort`, `enable_cot`,
  `COT_INSTRUCTION`, `max_iterations`, `inject_rules_with_root`,
  `rules_injection_with_root`, `load_project_rules`, `to_reinjection_messages`,
  `set_canonical_context`, `tool_names`, `final_answer`, `with_memory_tools`,
  `set_memory_store`. Results: one `ToolManager` authority per agent (V02);
  one project-rules authority (V03); two `tool_names()` methods with
  different semantics (P3-05); three divergent `max_iterations` defaults and
  two divergent `enable_cot` defaults (P2-03); one documented-but-unenforced
  allowlist (P2-01).

## Current Path

Construction data flow (verified, with anchors): `ReactAgentBuilder::build()`
(builder.rs:854-1065) validates (empty model → `ConfigError::MissingConfig`
:856-862; `enable_subagent` without tools → `ConfigFileError` :864-870),
builds `AgentConfig` (:872-943: role, enable_tool, readonly_tools,
enable_memory/task/human_in_loop/subagent, register_agent_dispatch_tool,
enable_cot, tool_error_feedback, tool_execution, max_iterations, run_budget,
token_limit, max_tokens, temperature, model_profile, project_root,
response_format, max_tool_output_tokens, tool_output_artifacts, callbacks,
session/conversation/working_dir, react_checkpoint_interval) → disables
`enable_memory` when an external store is present (:948-951) → constructs
`ReactAgent::new`/`new_with_subagent_registry` (mod.rs:303-320). `new_inner`
(mod.rs:322-583): `build_system_prompt` (mod.rs:676-723: project rules
prepended + user prompt + CoT (only `enable_tool && enable_cot`) +
model-profile suffix) → `ContextManager::builder` with system + tokenizer +
budget + SlidingWindow compressor (:336-354) → canonical context carrying
the same rules twice (system_prompt AND project_rules, :360-382) → one
`ToolManager::new_with_config` (:386) with unconditional `final_answer` +
`build_task_tools` (:393-400), feature-gated human_in_loop/task tools
(:442-453), `register_feature_gated_tools` (register_all_tools /
register_readonly_tools under `enable_tool`, :740-748), memory tools under
`enable_memory` (:750-784), `ToolSearchTool` with `Weak` handle (:461),
`AgentDispatchTool` under `register_agent_dispatch_tool` (:478-497). Back in
`build()`: optional `register_task_tools` replacement (:962-964), LLM
client/config (:966-973), custom tools via `add_tool` loop (:976-978),
`set_memory_store` (:981-983), approval/guards/audit/snapshot/circuit-breaker/
sandbox/run-store/pipeline/template-engine/intent-router/state-store/horizon/
intervention callbacks (:985-1062). Runtime surface: `tools_for_llm`
(snapshot.rs:269-288) filters disabled/visibility/skill-allowed/plan-mode
(no `enable_tool`, no `allowed_tools`); plan mode defaults false
(config.rs:242), settable only at runtime (`set_plan_mode` mod.rs:2031-2033)
and propagated into execution (execution.rs:293, PlanModeStage
pipeline.rs:1000-1018). Determinism: tool schema is name-sorted and
version-cached (tools.rs:281-307); identical inputs ⇒ identical model-facing
surface.

## Findings

### F-RCT-01-P2-01: `AgentConfig::allowed_tools` is documented as an enforcement mechanism but is never enforced at visibility or call time

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/config.rs:55` (field doc: "Tool allowlist
  (empty = no restriction, all registered tools can be called)"), `:478-481`
  (setter doc: "If the list is non-empty, only tools in the list can be
  called"), `:487-489` (getter); the only consumer is the registration filter
  in `capabilities.rs:48-57` (`add_tools`); `add_tool` (capabilities.rs:37-40)
  does not filter; `tools_for_llm` (snapshot.rs:269-288) filters only
  disabled_tools/visibility/skill_allowed_tools/plan_mode; `available`
  (snapshot.rs:227-236) likewise; grep for `allowed_tools` in
  `echo-agent-cli` shows only EKO's own task/subagent types, never
  `AgentConfig::allowed_tools`.
- Reachability: any agent constructed with a non-empty allowlist — via
  `AgentConfig` directly (builder exposes no allowlist option, V01) — still
  exposes every registered tool (register_all_tools path, mod.rs:740-748) to
  the LLM and allows calling them; the only effect is that `add_tools`
  (MCP/batch/skill-registry calls, capabilities.rs:1133-1135, tasks.rs:18-26)
  silently skips non-allowed tools.
- Expected invariant: the documented contract "only tools in the list can be
  called" holds.
- Observed behavior: no visibility-time or call-time filtering exists;
  registration-time filtering is partial (batch-only) and silent.
- Impact: consumers that set `allowed_tools` believing the doc get an
  unprotected tool surface — a false security/scope guarantee; additionally,
  `register_task_tools` replacement can be silently skipped when the
  allowlist excludes task tools, making the builder's `task_revision_service`
  option silently ineffective (builder.rs:962-964 → capabilities.rs:48-57).
- Root cause: the allowlist was implemented as a registration filter, then the
  registration paths diverged (`add_tool` vs `add_tools`, and the builder's
  batch iterates `add_tool`, builder.rs:976-978), and no enforcement layer was
  ever added at snapshot/execution time.
- Direction: either (a) enforce the allowlist in `tools_for_llm`/`available`
  and the pipeline (single authority at the snapshot), or (b) re-document the
  field as a registration-time filter only and fix `add_tool` to apply the
  same filter as `add_tools`; add a test that a non-empty allowlist excludes
  out-of-list tools from `tools_for_llm` and blocks their execution.
- Regression validation: unit test building an agent with
  `allowed_tools(vec!["read_file"])` + full tool set, asserting
  `tools_for_llm()` contains only allowlisted names (after fix) and that
  calling an out-of-list tool is rejected; keep the builder
  `default_agent_uses_one_task_relation_api` test green (V04).
- Validation reports: [V01](../validations/F-RCT-01/V01-01.md),
  [V02](../validations/F-RCT-01/V02-01.md), [V04](../validations/F-RCT-01/V04-01.md)

### F-RCT-01-P2-02: Project rules are stored twice at construction and re-injected twice after compression; workspace switches can re-inject stale rules

- Priority: P2
- Confidence: medium (static chain fully verified; the triggering
  compression-eviction scenario was not executed dynamically)
- Layer: framework
- Evidence: `echo-agent/src/agent/react/mod.rs:360-382` (canonical context
  built with `system_prompt: Some(sp_for_canonical)` — which already embeds
  the rules via `build_system_prompt` :676-723, `inject_rules_with_root`
  prepends at `echo-core/src/project_rules.rs:209-218` — AND
  `project_rules: rules_injection_with_root(...)` :371-374);
  `echo-state/src/compression/mod.rs:877-929` (`reinject_canonical_context`:
  restores the system prompt :884-893, then appends
  `to_reinjection_messages()` output at sys_end :904-927; exact-text dedup
  :912-921 compares whole message texts, but the canonical message has the
  `[Canonical context — project rules restored]:` prefix,
  `echo-core/src/compression.rs:384-390`, so the two copies never match);
  call sites at compression/mod.rs:1071, 1144, 1203, 1525-1530;
  `echo-core/src/compression.rs:371-375` (doc contract: "the prompt is not
  represented twice").
- Reachability: any agent with `auto_project_rules` default true
  (config.rs:259) in a project with instruction files, after the first
  compression that triggers canonical re-injection; staleness additionally
  requires `set_working_dir` (mod.rs:907-946), which refreshes
  `canonical.system_prompt` (:939) but never `canonical.project_rules`.
- Expected invariant: project rules appear exactly once in the final context;
  canonical re-injection preserves the "not represented twice" contract; the
  rules correspond to the current working directory.
- Observed behavior: after the first compression, the rules text appears both
  inside the restored system message and as a separate canonical message
  (subsequent compressions keep one canonical copy via exact-text dedup); a
  working-directory switch leaves `canonical.project_rules` pointing at the
  previous workspace's rules.
- Impact: duplicated instruction weight and wasted tokens (project rules can
  be large — AGENTS.md is multi-KB here); after workspace switches the LLM
  receives rules from the wrong project on the next compression; the
  documented cache-stability rationale (mod.rs:677-682) is weakened because
  the canonical message is inserted into the system region.
- Root cause: two independent storage sites for the same content
  (system prompt embedding vs canonical field) were introduced without a
  single authority; the re-injection dedup matches exact text only.
- Direction: pick one authority — keep the rules in the built system prompt
  (cache-stable per project) and set `canonical.project_rules = None`
  (mod.rs:362-375), or move rules entirely to canonical re-injection and stop
  embedding them in `build_system_prompt`; update
  `refresh_root_system_prompt` to also refresh `canonical.project_rules` if it
  stays; align the doc at echo-core/src/compression.rs:371-375. Note the
  `project-rules` feature gate: without the feature both sites are already
  inactive, which is consistent.
- Regression validation: ContextManager-level unit test — canonical with
  system prompt embedding rules + project_rules set, force-compress with
  system eviction, assert the rules text occurs exactly once in the resulting
  messages; a working-dir-switch test asserting the re-injected rules match
  the new directory.
- Validation reports: [V03](../validations/F-RCT-01/V03-01.md)

### F-RCT-01-P2-03: Divergent construction defaults — `max_iterations` (10/100/0) and `enable_cot` (true/false) depend on which constructor is used

- Priority: P2
- Confidence: high (facts); medium (impact)
- Layer: framework
- Evidence: builder default `max_iterations: 10` (builder.rs:149, no doc
  comment) vs `AgentConfig::new` default 100 (config.rs:212, doc "default:
  100, effectively unlimited for most tasks" config.rs:48) vs YAML config
  default 0 = unlimited (src/config.rs:469; `to_agent_config` passes 0
  at :147; "0 means unlimited" stream_channel.rs:521-528); builder default
  `enable_cot: true` (builder.rs:146) vs `AgentConfig::new` default false
  (config.rs:240, doc "default false" config.rs:120-121); builder always
  overwrites both on build (builder.rs:881, 884).
- Reachability: `ReactAgentBuilder::new().model(...).build()` (and the
  `simple()`/`standard()` presets) silently cap iterations at 10 and enable
  the cot flag; `ReactAgent::from_config_file` (mod.rs:669-672) yields 0
  (unlimited); `AgentConfig::new(...)` direct construction yields 100;
  EKO overrides explicitly (infra.rs:290 `max_iterations(app_config... )`) so
  the app is unaffected.
- Expected invariant: one documented default per option, independent of
  construction path.
- Observed behavior: three authorities for the iteration default with
  different values; a framework consumer following the config.rs doc gets a
  builder-built agent capped at 10 iterations (premature termination for
  multi-step tasks).
- Impact: surprising behavior differences between construction paths;
  framework docs are wrong for builder users; iteration cap can truncate
  legitimate multi-step runs without error.
- Root cause: the builder predates the config defaults and was never aligned;
  the "0 = unlimited" semantic was added later in the loop (stream_channel.rs)
  without revisiting builder/config defaults.
- Direction: align defaults — either builder default becomes 0 (unlimited,
  matching YAML) or 100 (matching AgentConfig) with a doc comment; align
  `enable_cot` (recommend builder default false to match config) and update
  the config.rs:48 doc; add a test asserting
  `ReactAgentBuilder::new().build()` and `AgentConfig::new()` produce equal
  defaults for these two fields.
- Regression validation: extend builder tests (V04) with a default-equality
  assertion between the two constructors; loop-limit tests in
  stream_channel.rs (1723, 1828, 1880) stay green.
- Validation reports: [V01](../validations/F-RCT-01/V01-01.md),
  [V04](../validations/F-RCT-01/V04-01.md)

### F-RCT-01-P3-01: `AgentConfig._reasoning_effort` is a dead field with no reader or setter

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/config.rs:130` (field, doc "Reasoning
  effort: low(quick)/medium(standard)/high(thorough)"), `:243` (init
  "medium"); grep for `_reasoning_effort` across both repositories: zero
  readers beyond the definition/init; underscore prefix marks it
  intentionally unused.
- Reachability: none — the field is never read; no API sets it; the
  `ThinkingConfig`/`to_reasoning_effort` path (echo-core/src/llm/thinking.rs)
  is a separate, live mechanism.
- Expected invariant: either reasoning effort is configurable and consumed,
  or the dead field is removed (AGENTS.md cleanup).
- Observed behavior: a documented config surface exists that cannot be set and
  has no effect; readers may assume it is wired.
- Impact: misleading public surface; dead code per cleanup rules.
- Root cause: field scaffolded during planning; the ThinkingConfig mechanism
  superseded it and the field was never wired or deleted.
- Direction: delete the field and its initializer; if effort control is
  wanted, expose it through the existing `ThinkingConfig`/`ModelProfile` path.
- Regression validation: `cargo check -p echo_agent` after removal; grep for
  `_reasoning_effort` returns nothing.
- Validation reports: [V01](../validations/F-RCT-01/V01-01.md)

### F-RCT-01-P3-02: `enable_tool=false` does not remove framework tools from the LLM surface — `minimal()`/`disable_tools()` docs are inaccurate

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: unconditional registration of `final_answer` + task tools
  (`echo-agent/src/agent/react/mod.rs:393-400`) regardless of
  `config.enable_tool`; `tools_for_llm` (snapshot.rs:269-288) and `available`
  (snapshot.rs:227-236) have no `enable_tool` filter; snapshot copies the flag
  (snapshot.rs:96,141) but no run-loop consumer exists; `enable_tool` gates
  only `register_feature_gated_tools` (mod.rs:740-748) and CoT injection
  (mod.rs:683-691); `AgentConfig::minimal` doc "no tools, no memory"
  (config.rs:277-283); `disable_tools()` doc "Disable built-in tools"
  (builder.rs:290-293); test `default_agent_uses_one_task_relation_api`
  (builder.rs:1171-1183) pins task-tool presence by default.
- Reachability: a default-built agent with `enable_tool=false` still presents
  `final_answer`, `task_create`, `task_update`, `task_list` to the model and
  executes them; only business tools are absent.
- Expected invariant: "minimal/no tools" means the model cannot see or call
  any tool.
- Observed behavior: the framework core tools remain visible; the invariant
  holds only for business tools.
- Impact: doc/behavior mismatch; consumers believing `disable_tools()` yields
  a tool-free agent get a model that can mutate the task relation graph;
  memory/task tool naming collisions with the agent's own surface are
  possible.
- Root cause: the framework core tools (final_answer + task API) were made
  unconditional after the feature split, without updating the enable_tool
  docs.
- Direction: either gate final_answer/task tools on `enable_tool` (and update
  the builder test), or re-document that `enable_tool` controls business
  tools only and update `minimal()`/`disable_tools()` docs accordingly.
- Regression validation: adjust/extend `default_agent_uses_one_task_relation_api`
  to assert the chosen contract; visibility fixture asserting the model-facing
  tool list under `enable_tool=false`.
- Validation reports: [V01](../validations/F-RCT-01/V01-01.md),
  [V04](../validations/F-RCT-01/V04-01.md)

### F-RCT-01-P3-03: CoT guidance is silently inert when tools are disabled; `is_cot_enabled()` disagrees with the prompt content

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: injection gated on `enable_tool && enable_cot`
  (`echo-agent/src/agent/react/mod.rs:683-691`); builder default
  `enable_cot: true` (builder.rs:146) with `enable_builtin_tools: false`
  (builder.rs:129) — the default `simple()` agent (builder.rs:190-195) and
  any `.enable_cot()`-only builder (builder.rs:423-426) have
  `is_cot_enabled()==true` (config.rs:723-725) but no CoT text in the prompt;
  `enable_cot()` is a no-op relative to the default.
- Reachability: any builder-built agent without `enable_tools()`.
- Expected invariant: `is_cot_enabled()` truthfully reports the injected
  guidance.
- Observed behavior: flag true, prompt without the instruction.
- Impact: config introspection (EKO and other consumers rely on
  `is_cot_enabled` style checks) misreports behavior; `.enable_cot()` is
  misleading.
- Root cause: COT injection was coupled to tool enablement (the instruction is
  tool-oriented) after the builder default was already set to true.
- Direction: either inject the CoT guidance regardless of tool enablement, or
  drop the tool gating and document CoT as tool-independent; align the two
  defaults per P2-03.
- Regression validation: builder test asserting that
  `.enable_cot().enable_tools()` produces a system prompt containing the CoT
  text and `.enable_cot()` alone does not (or does, per the chosen contract).
- Validation reports: [V01](../validations/F-RCT-01/V01-01.md)

### F-RCT-01-P3-04: `.store(store)` registers memory tools contrary to its doc; `.with_memory_tools` leaves `is_memory_enabled()` false

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `.store` doc "Set long-term memory Store" (builder.rs:654-658) vs
  `.with_memory_tools` doc "equivalent to `.store(store).enable_memory()`"
  (builder.rs:660-665) implying store() alone does not register tools; but
  build() calls `agent.set_memory_store(store)` for both (builder.rs:981-983)
  and `set_memory_store` always registers the four memory tools
  (mod.rs:1075-1111); `with_memory_tools` sets `enable_memory=true`
  (builder.rs:681-685) and build() then disables it (`has_external_store` →
  `config.enable_memory(false)`, builder.rs:948-951), so
  `is_memory_enabled()` returns false while the tools exist and the store is
  set.
- Reachability: `.store(x)` alone — an agent with memory tools the caller did
  not ask for; `.with_memory_tools(x)` — `is_memory_enabled()==false` for any
  consumer that inspects the config flag.
- Expected invariant: docs match behavior; the flag truthfully reflects the
  installed memory subsystem.
- Observed behavior: doc/behavior mismatch in both directions.
- Impact: memory tools appear unexpectedly (or are assumed absent) depending
  on which API and which inspection path a consumer uses.
- Root cause: `set_memory_store` conflates "set store" and "register memory
  tools"; the builder then compensates by clearing the flag it just set.
- Direction: split `set_memory_store` into store-set and tool-registration
  (or have build() keep `enable_memory` as configured and register tools only
  when either flag is on); update both docs; add a builder test asserting
  tool names and flag state for `.store()` vs `.with_memory_tools()`.
- Regression validation: builder test asserting `tool_names()` contains
  remember/recall/search_memory/forget and `is_memory_enabled()` matches the
  documented contract for both entry points.
- Validation reports: [V01](../validations/F-RCT-01/V01-01.md)

### F-RCT-01-P3-05: Two `tool_names()` methods with different filtering semantics on the same agent

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence: inherent `ReactAgent::tool_names` returns `list_tools()`
  unfiltered (mod.rs:1265-1267) vs `Agent` trait impl filters out
  `final_answer` (mod.rs:2941-2949); same divergence for
  `tool_definitions`/trait `tool_definitions` (mod.rs:2952-2959);
  `tool_search_is_hidden_when_deferred_visibility_is_disabled` test uses both
  APIs on the same agent and observes the difference (snapshot.rs:1675).
- Reachability: consumers calling `agent.tool_names()` on a concrete
  `ReactAgent` vs through `Box<dyn Agent>` get different results for the same
  agent.
- Expected invariant: same method name ⇒ same semantics regardless of
  call-site typing.
- Observed behavior: final_answer is included or excluded depending on the
  receiver type.
- Impact: tool counting/surface inspection (e.g. EKO UI projections, the
  "67 registered tools" README claim) depends on the receiver type; subtle
  divergence for consumers.
- Root cause: the inherent method predates the trait; the trait version
  deliberately filters the internal final_answer tool and the inherent one
  was never aligned.
- Direction: make the inherent method filter final_answer (or expose both
  under distinct names, e.g. `all_tool_names` vs `tool_names`), and update the
  snapshot test to pin one contract.
- Regression validation: unit test calling both forms and asserting equal
  results for the same agent.
- Validation reports: [V01](../validations/F-RCT-01/V01-01.md),
  [V02](../validations/F-RCT-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Builder option-to-runtime map; dead-option search (incl. plan_mode absence) | yes | passed | [V01-01](../validations/F-RCT-01/V01-01.md) |
| V02 | Duplicate registry search (set_tool_manager vs add_tool/register paths; F-EXT-01-P1-02 cross-ref) | yes | passed | [V02-01](../validations/F-RCT-01/V02-01.md) |
| V03 | Prompt section/cardinality fixtures; project-rules duplicate/stale injection | yes | passed | [V03-01](../validations/F-RCT-01/V03-01.md) |
| V04 | `cargo test -p echo_agent --lib --locked react::builder` (+ `agent::config`) | yes | passed (exit 0; 7 passed / 15 passed) | [V04-01](../validations/F-RCT-01/V04-01.md) |
| V05 | plan_mode default/semantics + F-EXT-01 P1-01 cross-reference | conditional | passed | [V05-01](../validations/F-RCT-01/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN (via F-EXT-01): plan-mode = read-only tool filtering | current | `snapshot.rs:230-236,282-285`; `pipeline.rs:1000-1018`; `execution.rs:293`; test `snapshot.rs:1610-1648`; [V05-01](../validations/F-RCT-01/V05-01.md) |
| MASTER-PLAN (M13): task API unified into `task_create/task_update/task_list`, `todo_write`/`plan_*` deleted | current | unconditional `build_task_tools` `mod.rs:394-400`; builder test `default_agent_uses_one_task_relation_api` green; [V04-01](../validations/F-RCT-01/V04-01.md) |
| `AgentConfig::new` doc "max_iterations default 100, effectively unlimited" (config.rs:48) | stale (for builder users) | builder default 10 (builder.rs:149); YAML default 0 (src/config.rs:469); [V01-01](../validations/F-RCT-01/V01-01.md) |
| `AgentConfig` doc "enable_cot default false" (config.rs:120-121) | stale (for builder users) | builder default true (builder.rs:146); [V01-01](../validations/F-RCT-01/V01-01.md) |
| `CanonicalContext::to_reinjection_messages` doc "prompt is not represented twice" (echo-core/src/compression.rs:371-375) | regressed | system prompt embeds rules + canonical project_rules both re-injected; dedup misses; [V03-01](../validations/F-RCT-01/V03-01.md) |

## Coverage And Uncertainty

- All conclusions are static except two test runs (V04); no dynamic run
  exercised compression-triggered re-injection, so P2-02's trigger is
  logically derived, not executed.
- `max_tokens`/`temperature`/`cache_user_id`/`response_format` consumers were
  verified to exist in the LLM call path but not traced end-to-end (F-LLM
  scope).
- Loop behavior (iteration semantics, run_budget consumption details,
  mutable_system_prompt runtime composition at mod.rs:2753) belongs to
  F-RCT-02/03 and was not audited here.
- Skill/plugin/MCP discovery internals beyond projection mechanics not
  reviewed (F-SKL/MCP tasks).
- The `project-rules` feature gate: without the feature both injection sites
  are inactive; all P2-02/P3-02 claims assume the default all-features build.
- Whether `enable_tool=false` + visible task tools is "intended" is a product
  decision; the finding documents the doc/behavior mismatch, not the choice.

## Handoff

- Downstream tasks may rely on: the option-to-runtime map (V01); single
  registry per agent with deterministic sorted schema (V02); single
  project-rules authority and the duplicate/stale re-injection chain (V03);
  green builder/config test state (V04); plan_mode classification and
  F-EXT-01-P1-01 consistency (V05).
- `F-RCT-02/03`: run-loop behavior must honor `max_iterations=0` unlimited
  semantics; treat the canonical re-injection duplication as known context
  overhead.
- `F-CTX-01`: duplicated project rules after compression are an additional
  budget consumer; the P2-02 fix changes context composition.
- `F-RCT-05`: resume paths must be checked against the stale
  `canonical.project_rules` after workspace switches (P2-02 staleness arm).
- `F-FEAT-01`: the `project-rules` gate interplay with P2-02; `X-BND-01`:
  record the allowed_tools authority decision (P2-01).
- Reports to read: this report + [V01-01](../validations/F-RCT-01/V01-01.md)
  through [V05-01](../validations/F-RCT-01/V05-01.md); F-EXT-01 for the
  P1-01/P1-02/P2-01 tool-surface findings.
- Stale triggers: any change to `react/builder.rs`, `agent/config.rs`
  defaults, `react/mod.rs` `new_inner`/`build_system_prompt`,
  `echo-state` `reinject_canonical_context`, or `echo-core::project_rules`
  invalidates the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): F-RCT-02,
  F-CTX-01, F-RCT-05, F-FEAT-01, X-BND-01, Q-DOC-01 (config/builder doc
  rewrites for P3-02/P3-03/P3-04 and the stale default docs).
