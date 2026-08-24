# A-SUB-01: EKO Subagent catalog, pool, and prompt compilation

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (framework referenced; not directly modified)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean

## Question

Does EKO add domain definitions and product policy while reusing one framework
Subagent lifecycle and immutable effective catalog?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs` (893 lines) —
  the `.md` hot-loader: `BUILTIN_SUBAGENT_FILES` (compiled-in sources),
  `SubagentFrontmatter` schema, `discover_subagents` (project > user > builtin
  merge), `parse_subagent_md`, `split_frontmatter`, `SubagentCatalogSnapshot`
  (`from_definitions` / `from_registered` / `prompt`), `subagent_isolation`.
- `echo-agent-cli/echo-agent-app-core/src/subagent_prompt.rs` (740 lines) —
  `EkoSubagentPromptCompiler` (product compiler implementing the framework
  `SubagentPromptCompiler` trait), `compile_system` (registration-time role
  prompt envelope), `compile_invocation` (direct + planned TaskRuntime
  framings), `EkoPromptPayload::PlannedTask`, `SUBAGENT_LANGUAGE_POLICY`,
  `COMMON_ORCHESTRATION_POLICY`, `SUBAGENT_COMMUNICATION_PROTOCOL`,
  `SUBAGENT_RESULT_QUALITY_POLICY`, `SUGGESTED_TASKS_POLICY`.
- `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs` (1469 lines) —
  `AgentPool`, `SharedResources` (notably **no** `subagent_registry` field),
  `from_runtime`, `acquire`/`release`/`get`, `create_agent` (calls
  `infra::create_agent` → re-runs `discover_subagents` per pool agent),
  `apply_working_dir` / `apply_memory_store` / `apply_runtime_model` /
  `apply_permission_mode` / `refresh_skill_descriptors` /
  `apply_workspace_routing` / `refresh_instruction_context` (the pool's
  propagation surface).
- `echo-agent-cli/echo-agent-app-core/src/infra.rs:220-340, 440-873` —
  `create_agent_with_diagnostics`: `discover_subagents` invocation,
  `SubagentCatalogSnapshot::from_definitions`, `validate_default_subagent_routes`
  bootstrap gate, `assembler.add_subagent_catalog` (bake-into-system-prompt),
  framework `SubagentRegistry::new()` (per-agent), `register_default_subagents`
  (compile_system → build → register → factory), `resolve_subagent_model`,
  `static_subagent_environment`.
- `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:260-552` —
  `read_plugin_agent_with_variables` (reuses `parse_subagent_md` for plugin
  `.md`), `register_plugin_agents` (separate registration path:
  `register_subagent_with_definition` + `register_subagent_factory` directly on
  the primary agent), `framework_definition` (plugin → framework
  `SubagentDefinition` adapter, hardcodes `tool_filter: None`,
  `lightweight: false`, `timeout_secs: 0`, `inherit_history: Some(2)`).
- `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:560-870` —
  plugin lifecycle: `prepare_application_components`, candidate-vs-previous
  swap, `register_plugin_agents` call site (post-bootstrap, primary-agent
  only).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/profiles.rs:1-362`
  — `ProfileTemplate`, `DOMAIN_PROFILES` (5: General / AiCoding / DataAnalysis
  / AcademicResearch / MedicalResearch), `PLAN_TASK_KINDS` (8),
  `default_subagent_for` (route table), `validate_default_subagent_routes`
  (startup gate).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/register.rs:26-75`
  — `task_revision_service_for_agent` / `register_task_tools_on_agent`:
  re-derive `SubagentCatalogSnapshot::from_registered` from the live registry
  when wiring task-management tools (post-bootstrap, includes plugins).
- `echo-agent-cli/echo-agent-app-core/src/prompt_contract.rs:1-200` —
  `audit_prompt`, `PromptContractSpec`, `builtin_subagents_leave_shared_sections_to_the_compiler`
  test (700-token role-prompt cap + compiler-owned section exclusivity).
- `echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:199-220` —
  `config_watch_targets` (confirmed: only config + global/project `hooks.yaml`;
  subagent `.md` not watched).
- `echo-agent-cli/echo-agent-app-core/src/state.rs:844-1010` —
  `switch_workspace` (confirmed: no subagent re-discovery path).
- `echo-agent-cli/echo-agent-app-core/src/subagents/coding/*.md` and
  `subagents/data/*.md` (8 builtin `.md` files) — read `explorer.md` in full
  to verify frontmatter / language split / UTF-8 safety.

Framework cross-references (read-only, for boundary verification):

- `echo-agent/src/agent/subagent/types.rs` — `SubagentDefinition` (22 fields,
  per F-SUB-01), `TeamSpec`, `ExecutionMode`, `SubagentKind`.
- `echo-agent/src/agent/subagent/prompt.rs` — `SubagentPromptCompiler` trait,
  `SubagentSystemPromptInput`, `SubagentPromptInput`, `filter_history`,
  `render_result_contract`, `ContextTransferPolicy`.
- `echo-agent/src/agent/subagent/registry.rs` — `SubagentRegistry`,
  `register_*_sync` (last-write-wins `HashMap::insert`), `list_available`.
- `echo-agent/src/agent/react/capabilities.rs:300-438` —
  `register_subagent_with_definition`, `register_subagent_factory`,
  `update_dispatch_catalog`.

## Out Of Scope

Deferred to named task IDs:

- Framework `SubagentDefinition` field-level liveness (dead `tool_filter`,
  `lightweight`, `compile_system` trait-method call sites, `SubagentOutput`
  duplicate) → **F-SUB-01** (already filed).
- Framework execution-mode lifecycle (Sync/Fork/Teammate/Team) and Team-mode
  cancel/timeout/detached-task gaps → **F-SUB-02** (already filed).
- Pool eviction, capacity counting, background-task concurrency policy
  internals → **A-POOL-*** / **A-TSK-*** (this task only touches the pool's
  subagent refresh surface, which is absent).
- Plugin integrator lifecycle, candidate/previous swap correctness, plugin
  hot-reload topology beyond the subagent surface → a plugin-focused task.
- The full TaskRuntime revision/execute plumbing that consumes the
  `TaskCapabilityCatalog` → **A-TSK-01** / **A-TSK-02**.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  system reminder). Key constraints applied: framework-vs-application
  layering (EKO product policy lives in app-core; generic mechanism lives in
  echo-agent), "first check if it already exists," no-duplicate rule, UTF-8
  safety, "delete over retain" for dead code, TUI/GUI/CLI feature parity,
  Subagent-only terminology.
- `docs/comprehensive-review/REPORTING.md`.
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-SUB-01.md` (in full).
  Established: framework has one canonical `SubagentRegistry`, one
  `AgentDispatchTool`, one `SubagentOutcome` result contract; the dispatch
  catalog (`SubagentCatalogEntry`) is a per-agent projection, not a second
  authority; `SubagentPromptCompiler::compile_system` is a real trait method
  but its framework default impl is inert in production — EKO's product
  compiler is what actually wires it (`infra.rs:627-638`). This task confirms
  the application side honors that contract.
- `docs/comprehensive-review/zcode-glm/tasks/F-SUB-02.md` (in full).
  Established: Sync/Fork/Teammate/Background share one lifecycle; Team mode
  is the outlier. This task does not re-verify execution-mode behavior; it
  only depends on F-SUB-02's handoff that the registry/dispatch path is the
  sole authority.
- `docs/comprehensive-review/zcode-glm/tasks/A-CFG-01.md` (in full).
  Established: hot-reload boundary is {user hooks, ConfigChange hook, webhook
  endpoints}; `AppState.app_config` is stale after first edit; workspace
  switch is storage/memory/skills isolation only. This task confirms
  subagents follow the same "no live reload" pattern (and adds the
  pool-divergence wrinkle A-CFG-01 did not touch).

Historical documents treated as hypotheses:

- `subagent_loader.rs:7-16` "Resolution order: project > user > builtin.
  On name collisions, the higher-priority scope wins." Treated as design
  intent; **code evidence confirms** for the three file-based scopes, but a
  **fourth source (plugins) is not part of this precedence table** and
  silently overrides all three (see A-SUB-01-P3-02).
- `infra.rs:562-574` comment "Only `readonly` subagents are registered here"
  (in the `register_default_subagents` doc). **Stale** — the body registers
  both readonly and writer subagents (Sprint 9+); the comment predates the
  Sprint 9 writer additions.
- `subagent_prompt.rs:1` "EKO-owned prompt compiler for every Subagent
  registration and dispatch path." Treated as design intent; **code evidence
  confirms for the primary agent** but **pool agents compile their own
  subagent prompts with the same compiler** (since each pool agent
  re-runs `create_agent`), and **plugin subagents are also compiled with it
  at primary-agent registration time** — so the claim holds, just not via a
  single shared invocation site.
- `agent_pool.rs:1-31` architecture comment "SharedResources (Arc-shared
  across all pool agents): LlmClient, ToolManager, HookRegistry,
  SandboxManager, Store, ...". **Confirmed** — and notably
  `SubagentRegistry` is **not** in `SharedResources`, which is the root cause
  of A-SUB-01-P2-01.

## Layering Decision

This is an **application-layer** task. The framework boundary is clean and
respected; the findings here are about EKO's product policy on top of the
framework's generic mechanism.

| Classification | Required answer |
|---|---|
| Generic mechanism (framework, correctly placed) | The `SubagentPromptCompiler` trait + `SubagentRegistry` + `AgentDispatchTool` + `SubagentCatalogEntry` projection (per F-SUB-01) are generic delegation machinery. `EkoSubagentPromptCompiler` correctly **implements** the trait rather than forking it. The framework's `render_result_contract()` is reused verbatim as the terminal envelope. The framework `SubagentKind::Custom { path }` variant (an inert placeholder per F-SUB-01-P3-01) is used as metadata in EKO's `register_default_subagents` to carry the source `.md` path — a tasteful reuse of the inert field as a diagnostic tag without depending on its unwired loader. |
| EKO product policy (application, correctly placed) | The `.md` discovery layout (`.eko/subagents/**/*.md`), the four-source precedence (builtin / user / project / plugin), `DomainProfile` × `PlanTaskKind` route table, the `EkoSubagentPromptCompiler` section stack (language policy, communication protocol, result-quality policy, suggested-tasks policy), the readonly/writer/data isolation policy mapped from frontmatter, and the `validate_default_subagent_routes` startup gate all live in `echo-agent-app-core` and depend on EKO product decisions. None of this leaks into the framework. |
| Adapter boundary | `plugin_components::framework_definition` (`plugin_components.rs:478-509`) is the thin adapter from the app-layer `subagent_loader::SubagentDefinition` to the framework `echo_agent::agent::subagent::SubagentDefinition`. It hardcodes `tool_filter: None`, `lightweight: false`, `timeout_secs: 0` — consistent with F-SUB-01's findings that those fields are dead. The adapter is loss-less for the live fields (name, description, system_prompt, model, execution_mode, isolate_*, team, can_delegate, tags, is_background). |
| Duplicate search | Searched names across both `echo-agent` and `echo-agent-cli`: `discover_subagents`, `register_default_subagents`, `register_plugin_agents`, `SubagentCatalogSnapshot`, `EkoSubagentPromptCompiler`, `validate_default_subagent_routes`, `default_subagent_for`, `from_registered`, `from_definitions`. Result: **one canonical loader** (`subagent_loader::discover_subagents`), **one canonical compiler** (`EkoSubagentPromptCompiler`), **one canonical route table** (`profiles::default_subagent_for`), **one canonical startup gate** (`profiles::validate_default_subagent_routes`). The catalog snapshot has two constructors (`from_definitions` for app-layer definitions, `from_registered` for live framework definitions) — these are duals over the same struct, not duplicates. **No parallel app-layer registry or compiler exists.** |
| Migration deletion | No deletion proposed in this review. The findings identify propagation/refresh gaps, not dead code; the fixes add wiring rather than remove it. |

## Current Path

Verified subagent definition → catalog → registration → pool dispatch flow at
`echo-agent-cli` commit `b3b2e81`:

```text
[1] BOOTSTRAP — primary agent (infra::create_agent_with_diagnostics)
   ├─ discover_subagents(project_root, user_home)
   │      [infra.rs:242-245 → subagent_loader.rs:262-317]
   │   ├─ builtin (8): include_str! → parse_subagent_md(builtin_name)
   │   │   → by_name.entry(name).or_insert(def)   [loader.rs:271-284]
   │   ├─ user (~/.eko/subagents): merge_scope → by_name.insert  [loader.rs:287-290]
   │   └─ project (<root>/.eko/subagents): merge_scope → by_name.insert
   │      [loader.rs:293-296]  (last write wins → project > user > builtin)
   │
   ├─ SubagentCatalogSnapshot::from_definitions(&discovered_subagents)
   │      [infra.rs:246-248 → loader.rs:170-183]   (SNAPSHOT — frozen here)
   │
   ├─ validate_default_subagent_routes(&snapshot)
   │      [infra.rs:249-251 → profiles.rs:139-151]  (5 profiles × 8 kinds)
   │      → returns Err → bootstrap fails (Result propagated)
   │
   ├─ assembler.add_subagent_catalog(&snapshot.prompt())
   │      [infra.rs:252-253]   (BAKED into primary agent's system prompt)
   │
   ├─ framework SubagentRegistry::new()  [infra.rs:268]  (per-agent Arc)
   ├─ EkoSubagentPromptCompiler  [infra.rs:266-267]
   │
   ├─ build ReactAgent (system_prompt now includes "## Available Subagents")
   │
   └─ register_default_subagents(...)  [infra.rs:479-496, 576-873]
          for each discovered_subagent:
            ├─ compile_system(role_prompt, environment=static_subagent_environment())
            │      [infra.rs:627-638 → subagent_prompt.rs:178-235]
            │      → 9 ordered sections, ends with render_result_contract()
            ├─ build_readonly_subagent_agent | build_writer_subagent_agent
            ├─ SubagentBuilder::new(name).fork_mode()...build()  → framework def
            │      [infra.rs:687-746]
            │      tags: prompt_source:<scope>, capability:<readonly|writer>,
            │            isolation:<worktree|workspace|context>
            └─ agent.register_subagent_with_definition(framework_def, instance)
                   + agent.register_subagent_factory(framework_def, fork_factory)
                   [infra.rs:847-851]
                   → framework: update_dispatch_catalog (agent_tool schema enum)

[2] POST-BOOTSTRAP — plugin registration (plugin_runtime.rs)
   prepare_application_components → register_plugin_agents(agent, &prepared)
      [plugin_runtime.rs:845 → plugin_components.rs:444-476]
      for each plugin .md (parsed via parse_subagent_md, source = "plugin:<name>:<path>"):
        ├─ build_plugin_agent(definition, resources)  → ReactAgent
        │      [plugin_components.rs:511-551]
        │      (reuses definition.system_prompt verbatim — NO compile_system!)
        ├─ framework_definition(plugin_agent)  → framework SubagentDefinition
        │      [plugin_components.rs:478-509]  (tool_filter=None, lightweight=false)
        └─ agent.register_subagent_with_definition + register_subagent_factory
              [plugin_components.rs:471-472]
              → framework: update_dispatch_catalog (catalog NOW includes plugin)
   ★ Snapshot at infra.rs:246 NOT refreshed; system prompt section NOT refreshed.

[3] POST-BOOTSTRAP — task-management tools (register.rs:45-75)
   register_task_tools_on_agent(agent_handle, store)
      ├─ registry = agent.subagent_registry()  (the LIVE registry)
      ├─ registry.list_available() → SubagentCatalogSnapshot::from_registered
      │      [register.rs:53-56 → loader.rs:185-212]  (re-derived, includes plugins)
      └─ TaskCapabilityCatalog::new(catalog, tool_names)

[4] POOL — pooled conversation agent (agent_pool.rs:824-978)
   pool.acquire("conv-XXX") → pool.create_agent("conv-XXX")
      ├─ infra::create_agent(&params, &app_config)
      │      [agent_pool.rs:850 → infra.rs — RE-RUNS STEPS [1] ABOVE]
      │      → fresh SubagentRegistry, fresh discover_subagents (disk read),
      │        fresh register_default_subagents, fresh agent_tool catalog
      │      ★ Pool agent has its OWN subagent registry. SharedResources
      │        does NOT include subagent_registry. Plugin agents never reach here.
      ├─ inject shared LlmClient / ToolManager / HookRegistry / ...
      │      [agent_pool.rs:856-932]  (subagent registry NOT in the list)
      └─ configure_agent_for_workspace + HITL dispatcher

[5] RUNTIME — LLM invokes agent_tool or task_execute
   ├─ agent_tool schema enum: from framework dispatch_catalog (live registry)
   │      → primary agent: 8 builtins + N plugins
   │      → pool agent: 8 builtins + M user/project overrides (NO plugins)
   ├─ system-prompt section "## Available Subagents": from snapshot baked at [1]
   │      → primary agent: 8 builtins + user/project (frozen at bootstrap)
   │      → pool agent: 8 builtins + user/project (re-baked per pool agent)
   └─ task_execute / TaskRuntime: uses TaskCapabilityCatalog from [3]
          (live registry; primary only — pool agents register their own
           task-management tools via SharedResources.task_runtime_store)
```

Key invariants verified by this graph (full evidence in V01-V04):

- **One framework lifecycle is reused, not forked.** `EkoSubagentPromptCompiler`
  implements `SubagentPromptCompiler`; `register_default_subagents` and
  `register_plugin_agents` both call the framework's
  `register_subagent_with_definition` + `register_subagent_factory`. No
  app-layer registry, no app-layer dispatch tool, no app-layer result type.
  Terminology is Subagent-only across `subagent_loader.rs`,
  `subagent_prompt.rs`, `agent_pool.rs`, `plugin_components.rs`,
  `plugin_runtime.rs`, `profiles.rs`.
- **The `.md` file format is the single source of subagent identity.** Both
  builtin (compiled-in via `include_str!`) and runtime (project / user /
  plugin) definitions flow through the same `parse_subagent_md` parser. No
  hardcoded subagent array exists (the legacy `SUBAGENT_DEFINITIONS` array
  was removed in Sprint 6 per `subagent_loader.rs:1-5`).
- **`compile_system` is wired by EKO.** Unlike the framework default (inert
  per F-SUB-01-P2-02), `register_default_subagents` calls
  `prompt_compiler.compile_system(...)` at `infra.rs:627-638` and feeds the
  compiled output as the agent's system prompt. The application closes the
  gap F-SUB-01 identified at the framework layer.
- **Startup validation gate exists.** `validate_default_subagent_routes`
  runs at bootstrap and fails the agent build on a broken route
  (`infra.rs:249-251`).

Key **violations** of the "immutable effective catalog" + "single refresh"
expectation (full evidence in V01-V04):

- **Plugin subagents register only on the primary agent.** Pool agents never
  see them — each pool agent builds its own registry from
  `discover_subagents` (file-based scopes only). (A-SUB-01-P2-01)
- **No reload mechanism.** The primary agent's subagent registry and the
  baked system-prompt catalog section are frozen at bootstrap. The config
  watcher does not watch `.md` files; `switch_workspace` does not
  re-discover; `AgentPool` has no `apply_subagent_definitions` analog to
  `refresh_skill_descriptors`. (A-SUB-01-P2-02)
- **System-prompt catalog diverges from the live registry after plugin
  registration.** The static `## Available Subagents` section reflects only
  bootstrap-time file-based scopes; plugin subagents appear in the
  `agent_tool` schema enum but not in the system prompt's catalog section.
  (A-SUB-01-P2-03)

## Findings

### A-SUB-01-P2-01: Plugin subagents register only on the primary agent — pool agents never see them

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:93-113` —
    `SharedResources` lists every shared subsystem (LlmClient, ToolManager,
    HookRegistry, SandboxManager, Store, ConversationStore, RunStore,
    TokenUsageTracker, PermissionService, RuntimeStateStore,
    ToolExecutionPipeline, ReviewIntegration, TaskRuntimeStore,
    BrowserRuntime). **`SubagentRegistry` is not in this list.**
  - `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:850` — pool's
    `create_agent` calls `infra::create_agent(&params, &app_config)`.
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:268` — `infra::create_agent`
    constructs a fresh `Arc::new(SubagentRegistry::new())` for every agent
    it builds, including pool agents.
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:479-496` —
    `register_default_subagents` is called inside `infra::create_agent`, so
    pool agents register only the file-based scopes (builtin + user +
    project) discovered by `discover_subagents`.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:833-867` —
    `register_plugin_agents` is called only from the plugin reload path on
    the **primary** `agent: &mut ReactAgent`. There is no
    `register_plugin_agents_on_pool` or pool-side equivalent.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:444-476` —
    `register_plugin_agents(agent: &mut ReactAgent, prepared)` takes a
    single agent reference; no fan-out to pool agents.
- Reachability: user installs a plugin that ships a `researcher.md`
  subagent → `plugin_runtime` calls `register_plugin_agents(primary_agent)`
  → primary's `agent_tool` schema enum includes `researcher` → user starts
  a second GUI conversation → `pool.acquire("conv-2")` → pool's
  `create_agent` runs `infra::create_agent` → fresh registry has only the 8
  builtins (plus user/project `.md` overrides) → pool's `agent_tool`
  schema enum does **not** include `researcher`. If the LLM in conv-2 tries
  `agent_tool(agent_name="researcher")`, the framework rejects it as an
  unknown name.
- Expected invariant: per AGENTS.md "TUI/GUI/CLI 功能对等" and "any
  capability one mode has, the others must have too" — pool-driven
  multi-session GUI conversations and the primary agent should expose the
  same delegation surface. A plugin subagent is a capability; it should be
  uniformly available.
- Observed behavior: the primary agent's registry and a pool agent's
  registry are independent `Arc<SubagentRegistry>` instances with no sync.
  Plugin subagents land only on the primary. The pool's `SharedResources`
  propagation surface (`apply_working_dir`, `apply_memory_store`,
  `apply_runtime_model`, `apply_permission_mode`,
  `refresh_skill_descriptors`, `apply_workspace_routing`,
  `refresh_instruction_context`) covers every other shared subsystem but
  has no subagent-registry entry.
- Impact: in any multi-conversation GUI session (the explicit purpose of
  `AgentPool` per `agent_pool.rs:1-31`), plugin subagents are silently
  unavailable. A user who installs a plugin, sees it work in conversation
  A (primary agent), then opens conversation B (pool agent) gets
  inconsistent behavior with no diagnostic. For TUI (single-agent, uses
  the primary directly) this is invisible; for GUI multi-session it is a
  real capability gap.
- Root cause: `SharedResources` was designed around `AgentHandle`-owned
  subsystems that could be `Arc`-cloned. `SubagentRegistry` is created
  inside `infra::create_agent` rather than extracted from a parent handle,
  so pool agents naturally build their own. The plugin path was added
  later (`plugin_runtime`) and was wired only to the primary agent because
  that was the only agent that existed at the time; pool fan-out was not
  added.
- Direction: pick one of two shapes. (a) **Share the registry**: extract
  the primary agent's `SubagentRegistry` into `SharedResources` (like
  `tool_manager` / `hook_registry`) and have `infra::create_agent` accept
  an optional pre-built registry, so pool agents register their own
  instance subagents into the shared registry. This is the smallest change
  that gives pool agents plugin visibility, but requires care because
  `register_default_subagents` builds `Box<dyn Agent>` instances that
  reference the registry. (b) **Fan out**: add a
  `pool.apply_subagent_definitions(definitions)` method that mirrors
  `refresh_skill_descriptors` and call it from `plugin_runtime` after
  `register_plugin_agents`. Option (a) is cleaner architecturally (one
  registry authority) but a bigger blast radius; option (b) is the
  pattern already used for skills.
- Regression validation: register a plugin subagent, acquire two pool
  agents, assert both pool agents' `agent_tool` schema enums contain the
  plugin subagent name and that dispatch resolves.
- Validation reports: [V01](../validations/A-SUB-01/V01-01.md),
  [V04](../validations/A-SUB-01/V04-01.md).

### A-SUB-01-P2-02: Subagent definitions have no reload mechanism (no watcher, no switch_workspace hook, no pool refresh API)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:199-213` —
    `config_watch_targets` returns the resolved config file,
    `~/.eko/hooks.yaml`, and `<cwd>/.eko/hooks.yaml`. **No
    `subagents/**/*.md` targets.**
  - `echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:227-278` —
    on settle, `handle_config_change` reloads only user hooks and webhook
    emitters; no subagent re-discovery.
  - `echo-agent-cli/echo-agent-app-core/src/state.rs:844-1010` —
    `switch_workspace` re-binds working_dir, persistence,
    conversation_store, runtime_state_store, memory store + layer manager,
    ReviewIntegration, workspace skills. **No `discover_subagents` or
    registry refresh** (grep for `subagent` in this function returns zero
    hits — see V04 method).
  - `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:493-528` — the
    pool's `refresh_skill_descriptors` propagation exists for skills.
    **No analog exists for subagents** — there is no
    `refresh_subagent_definitions`, `apply_subagent_registry`, or similar
    method anywhere on `AgentPool` (grep confirmed).
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:234-245` —
    `discover_subagents` is called only inside `create_agent`. There is no
    public `rediscover_subagents` or `reload_subagent_definitions` entry
    point.
- Reachability: user edits `~/.eko/subagents/explorer.md` (or adds a new
  `~/.eko/subagents/researcher.md`) at runtime → no watcher fires → primary
  agent's registry unchanged → primary's `agent_tool` schema enum
  unchanged → next `agent_tool("researcher")` call fails as unknown.
  Switching workspace does not help: `switch_workspace` does not re-run
  `discover_subagents`. Only a process restart picks up the change.
- Expected invariant: EKO advertises `.md` files as "edited without
  recompiling" (`subagent_loader.rs:1-5`) and "take effect on next agent
  build." For the bootstrap-time primary agent, "next agent build" means
  "next process restart" — which is defensible. But the pool side is
  inconsistent: pool agents lazily re-read `.md` files on each `acquire`
  (because they call `infra::create_agent`), so a user who opens a new
  conversation *does* pick up the edit, while the primary agent that has
  been running all along does not. This split behavior is undocumented and
  surprising.
- Observed behavior: three surfaces diverge after a `.md` edit:
  (1) **Primary agent's subagent registry**: frozen at bootstrap.
  (2) **Primary agent's system-prompt `## Available Subagents` section**:
      frozen at bootstrap.
  (3) **A newly-acquired pool agent's registry and system prompt**: fresh
      from disk at `acquire` time — so pool agents may have *different*
      subagent definitions than the primary agent if `.md` files changed
      after bootstrap.
  The skill system has the opposite-and-better design: a single
  `refresh_skill_descriptors` propagates the primary agent's descriptors to
  all pool agents, so they stay aligned.
- Impact: a user who edits a builtin subagent prompt (a documented use case
  — `subagent_loader.rs:1-5`) and continues chatting in the same session
  sees no effect. If they open a new conversation, the new pool agent has
  the new prompt but the primary agent (background tasks, the
  `__background__` slot, anything routed to the primary) keeps the old one.
  For a local personal assistant this is a usability papercut, not a
  correctness or safety issue.
- Root cause: the loader was designed for "next agent build" semantics
  (Sprint 6), predating the pool and the plugin integrator. No reload hook
  was added when the pool and plugins landed. The skill refresh pattern
  was not mirrored for subagents.
- Direction: the minimal fix is to add a
  `pool.refresh_subagent_definitions(definitions)` method (mirror
  `refresh_skill_descriptors`) plus a hook in `plugin_runtime` /
  `config_watcher` to call it. The deeper fix is to give the primary agent
  a `rediscover_subagents` API that re-runs `discover_subagents` + diffs
  against the live registry + registers/unregisters the delta. Either is a
  product decision; this review only flags the gap. The `.md`-edit case
  may be deemed restart-only (document it), but the **plugin
  register/unregister** case must propagate because plugins can hot-reload
  without restart (`plugin_runtime.rs` swap path) — see A-SUB-01-P2-01.
- Regression validation: edit a builtin `.md` at runtime; assert
  documented behavior (either primary sees it after a refresh API call, or
  docs say restart is required). Install + uninstall a plugin subagent at
  runtime; assert all pool agents converge.
- Validation reports: [V04](../validations/A-SUB-01/V04-01.md).

### A-SUB-01-P2-03: System-prompt-baked subagent catalog diverges from the live registry after plugin registration

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:246-253` —
    `SubagentCatalogSnapshot::from_definitions(&discovered_subagents)` is
    built **once** from the file-based scopes; `assembler.add_subagent_catalog`
    bakes `snapshot.prompt()` into the primary agent's system prompt
    **before** `ReactAgentBuilder::build()`. The snapshot is never rebuilt.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:844-867` —
    `register_plugin_agents` runs **after** `infra::create_agent` returns
    and the system prompt is already fixed. It calls
    `agent.register_subagent_with_definition` which updates only the
    framework's runtime dispatch catalog (the `agent_tool` schema enum),
    not the system-prompt section.
  - `echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs:222-241` —
    `SubagentCatalogSnapshot::prompt()` renders the markdown listing the
    LLM sees in the system prompt (`- \`name\`: description [access=...,
    isolation=..., delegation=...]`). This is the LLM's primary discovery
    surface for `agent_tool`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/register.rs:36-38, 54-56`
    — task-management tools re-derive the snapshot via
    `SubagentCatalogSnapshot::from_registered(&registered_subagents)` from
    the **live registry**, so the TaskRuntime/`task_execute` surface *does*
    see plugin subagents. This makes the inconsistency cross-surface: the
    same plugin subagent is invisible in the system prompt's
    `## Available Subagents` section but visible in `task_execute` /
    `create_complex_task` capability negotiation.
- Reachability: any plugin subagent registration after bootstrap. The
  plugin subagent appears in (a) the `agent_tool` schema enum (framework
  dispatch catalog), (b) the TaskRuntime capability catalog
  (`from_registered`); but NOT in (c) the system prompt's
  `## Available Subagents` section.
- Expected invariant: the LLM sees one coherent catalog of subagents. If
  the system prompt advertises roles `[explorer, reviewer, planner, ...]`
  but the `agent_tool` schema enum advertises `[explorer, reviewer, ...,
  plugin-researcher]`, the LLM may either (i) fail to discover
  `plugin-researcher` (relying on the system prompt section, which is the
  documented discovery surface), or (ii) get confused by the mismatch.
- Observed behavior: the three catalog surfaces (system-prompt section /
  `agent_tool` schema / TaskRuntime capability catalog) can diverge after
  plugin registration. The system prompt is frozen; the other two track
  the live registry. There is no refresh.
- Impact: plugin subagents get a degraded discovery story — the LLM may
  never invoke them because the system prompt's catalog section (the
  prominent listing) does not include them. The `agent_tool` schema enum
  is the fallback, but its descriptions are shorter and may not be enough
  for the LLM to choose the role. This is a real plugin-discoverability
  bug for any plugin that ships subagents.
- Root cause: the system prompt is built once at agent construction
  (framework contract — `ReactAgentBuilder` takes a `&str` system prompt).
  There is no framework API to refresh the system prompt post-build
  without rebuilding the agent. EKO's `assembler.add_subagent_catalog`
  runs at the wrong time relative to plugin registration.
- Direction: either (a) defer plugin subagent registration to before
  `ReactAgentBuilder::build()` (requires reordering bootstrap so plugins
  are discovered before the primary agent is built — bigger change), or
  (b) add a primary-agent method that re-renders the system prompt's
  `## Available Subagents` section from the live registry and patches it
  into the agent's context manager as an override projection (the agent's
  system prompt then has both the stale baked section and a fresh
  projection — needs care to avoid duplication), or (c) document that
  plugin subagents are not in the system prompt catalog and ensure the
  `agent_tool` schema descriptions are rich enough to stand alone. Option
  (c) is the cheapest unblocking fix.
- Regression validation: register a plugin subagent, send a chat turn,
  assert the LLM's system prompt contains the plugin subagent's name in
  the `## Available Subagents` section (after the fix), or assert the
  `agent_tool` schema description is self-contained (after the doc/option-c
  fix).
- Validation reports: [V01](../validations/A-SUB-01/V01-01.md).

### A-SUB-01-P3-01: `validate_default_subagent_routes` only validates the bootstrap snapshot — plugin hot-reload is unvalidated

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/profiles.rs:139-151`
    — `validate_default_subagent_routes` iterates the static
    `DOMAIN_PROFILES × PLAN_TASK_KINDS` table and asserts each route name
    is `== "primary"` OR `catalog.contains(role)`. It runs against a
    caller-supplied snapshot.
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:249-251` — invoked
    exactly once, with the bootstrap snapshot from
    `from_definitions(&discovered_subagents)`. Never re-invoked after
    plugin registration or workspace switch.
  - `echo-agent-cli/echo-agent-app-core/src/profiles.rs:120-136` — the
    route table is `match kind { ... => "explorer" / "reviewer" / ... /
    "primary" }`. Every non-`primary` route is a builtin name that
    `discover_subagents` seeds via `or_insert` first, so the gate can only
    fail if a builtin source file is corrupt — which the
    `builtin_defaults_parse_cleanly` test already guards at unit-test time.
- Reachability: the gate is structurally a no-op today. It would only
  catch a regression in `default_subagent_for` that referenced a
  non-builtin name, or builtin corruption. It does **not** catch:
  (i) a plugin that registers a subagent which shadows a builtin with a
  mismatched capability profile (e.g. plugin's "explorer" is a writer);
  (ii) workspace switch removing the user/project override for a name
  (does not happen today, but would if switch_workspace ever re-discovers
  — see A-SUB-01-P2-02);
  (iii) a plugin unregister path that removes a builtin name (no such API
  today, but if added would bypass the gate).
- Expected invariant: the route table's referenced roles remain
  resolvable+capability-appropriate across the registry's lifetime, not
  just at bootstrap.
- Observed behavior: the gate runs once on a snapshot, never on the live
  registry. Plugin hot-reload (`plugin_runtime.rs` swap path,
  lines 814-867) does not re-validate.
- Impact: low — today's route table is structurally safe (only references
  builtin names; builtins are always seeded). The gate is correct as far
  as it goes, but it is a one-shot guard, not an invariant. A future
  change that adds a non-builtin default route, or a plugin that
  capability-mismatches a shadowed name, would slip through.
- Root cause: the gate was written when the snapshot was the only catalog.
  Plugin registration and `from_registered` were added later without
  extending the gate.
- Direction: either (a) re-run `validate_default_subagent_routes` against
  `SubagentCatalogSnapshot::from_registered(&registry.list_available())`
  inside `register_plugin_agents` (cheap, catches the missing-role case),
  or (b) extend the gate to verify capability tags (e.g. `ReadOnlyReview`
  route should resolve to a `readonly` tag) so a plugin shadow with the
  wrong capability is caught. Option (a) is the minimum fix.
- Regression validation: register a plugin that unregisters or shadows a
  default route target; assert the validation fires.
- Validation reports: [V02](../validations/A-SUB-01/V02-01.md).

### A-SUB-01-P3-02: No documented precedence between plugin subagents and the three file-based scopes (last-write-wins, silent)

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs:7-16` — the
    docstring documents the resolution order for the three file-based
    scopes (project > user > builtin). It says nothing about plugins.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:844-867` —
    plugins are registered **after** bootstrap, directly into the primary
    agent's framework `SubagentRegistry` via
    `register_subagent_with_definition`. The framework registry is
    `HashMap::insert` (last-write-wins) per F-SUB-01.
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:847-851` —
    `register_default_subagents` runs first (seeds builtins + user +
    project). Then `register_plugin_agents` runs. If a plugin defines
    `explorer`, it overwrites the builtin `explorer` in the registry
    silently.
- Reachability: a plugin that ships `explorer.md` shadows the builtin
  `explorer` (and any user/project override of the same name). No warning,
  no diagnostic. The `agent_tool` schema enum now advertises the plugin's
  description for `explorer`; the system-prompt `## Available Subagents`
  section still shows the builtin's description (A-SUB-01-P2-03). The
  framework dispatch resolves to the plugin's factory. The capability
  profile may differ (e.g. plugin's `explorer` is a writer); the route
  gate does not catch this (A-SUB-01-P3-01).
- Expected invariant: either (a) plugins cannot shadow builtin/user/project
  names (rejected as too strict — overriding builtins is a legitimate
  plugin use case), or (b) the precedence is documented and a shadow
  emits a log line so the user can diagnose "why is my explorer behaving
  differently."
- Observed behavior: silent last-write-wins. No `info!` / `warn!` when a
  plugin overwrites an existing name. The loader docstring claims a
  three-tier precedence but the registry actually has four tiers with no
  documented rule for the fourth.
- Impact: low for the common case (plugins add new names, not override
  builtins). When a plugin does override, the user is left to debug
  silently-different behavior. Combined with A-SUB-01-P2-03, the system
  prompt and the schema enum disagree on what "explorer" is, compounding
  the confusion.
- Root cause: plugins were added as a separate registration path without
  reconciling their precedence against the file-based scopes. The loader
  docstring was not updated to mention plugins.
- Direction: (a) update `subagent_loader.rs:7-16` to document the
    effective four-tier precedence (plugin > project > user > builtin, by
    registration order). (b) In `register_plugin_agents`, log a `warn!`
    when a plugin's name collides with an already-registered definition,
    including both sources. (c) Consider exposing the source tag
    (`prompt_source:plugin:<name>`) in the `agent_tool` schema description
    so the LLM and the user can see which definition is live.
- Regression validation: register a plugin whose name collides with a
  builtin; assert a `warn!` log line; assert the system-prompt section and
  the schema enum agree on which description is shown (after A-SUB-01-P2-03
  fix).
- Validation reports: [V01](../validations/A-SUB-01/V01-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition source precedence (builtin/user/project + plugin path) and duplicate search | yes | passed | [V01-01](../validations/A-SUB-01/V01-01.md) |
| V02 | Default route startup validation gate | yes | passed | [V02-01](../validations/A-SUB-01/V02-01.md) |
| V03 | Prompt cardinality, language consistency, UTF-8 safety | yes | passed | [V03-01](../validations/A-SUB-01/V03-01.md) |
| V04 | Reload behavior and pooled-Agent refresh | yes | passed | [V04-01](../validations/A-SUB-01/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — three code comments treated as hypotheses; classifications in the Inputs section) | passed | classified inline (one stale, two current) |

Executed cargo commands (all exit 0):

```text
cd echo-agent-cli
cargo test -p echo-agent-app-core --lib subagent_loader::           (25 passed)
cargo test -p echo-agent-app-core --lib -- subagent_prompt prompt_contract profiles   (21 passed)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `subagent_loader.rs:7-16` "Resolution order: project > user > builtin" | partial drift | True for the three file-based scopes; **silent on the fourth source (plugins)** which register last and override all three (A-SUB-01-P3-02) |
| `infra.rs:562-574` "Only `readonly` subagents are registered here" | stale | The body registers both readonly and writer subagents (Sprint 9+); the docstring predates Sprint 9 |
| `subagent_prompt.rs:1` "EKO-owned prompt compiler for every Subagent registration and dispatch path" | current | Confirmed: `EkoSubagentPromptCompiler` is the sole compiler wired at `infra.rs:266-267` and used by `register_default_subagents`. Pool agents re-create it via `infra::create_agent`. Plugin agents bypass `compile_system` (they use the parsed `.md` system_prompt verbatim) but their *dispatch* still goes through the framework compiler path. |
| `agent_pool.rs:1-31` SharedResources architecture list | current (and load-bearing for A-SUB-01-P2-01) | Confirmed: list is accurate and notably excludes `SubagentRegistry`, which is the root cause of pool agents not inheriting plugin subagents |
| `subagent_loader.rs:1-5` "Subagent prompts now live in `.md` files ... can be edited without recompiling" | current | Confirmed: `include_str!` + `parse_subagent_md`; but "without recompiling" is not the same as "without restarting" — see A-SUB-01-P2-02 |
| `F-SUB-01` handoff — "single registry, single dispatch tool, single result contract" | current (framework) | This task confirms the application layer reuses that single authority — no app-layer parallel registry exists |
| `F-SUB-01-P2-02` — "framework `compile_system` is inert" | current (framework) | This task confirms the **application closes the gap**: `EkoSubagentPromptCompiler::compile_system` is invoked at `infra.rs:627-638` and its output drives the subagent's system prompt |
| `A-CFG-01` handoff — "hot-reload boundary is {hooks, ConfigChange, webhook}" | current | This task confirms subagents follow the same "no live reload" boundary (A-SUB-01-P2-02) and adds that pool agents lazily re-read `.md`, creating primary-vs-pool divergence A-CFG-01 did not address |
| `AGENTS.md` — "Only Subagent, no Worker" | current | Zero `Worker`/`worker_` hits in `subagent_loader.rs`, `subagent_prompt.rs`, `agent_pool.rs`, `plugin_components.rs`, `plugin_runtime.rs`, `profiles.rs` |

## Coverage And Uncertainty

Inspected in full: `subagent_loader.rs`, `subagent_prompt.rs`,
`agent_pool.rs`, `plugin_components.rs:240-552`, `plugin_runtime.rs:820-870`,
`tasks/task_runtime/profiles.rs`, `tasks/task_runtime/register.rs`,
`prompt_contract.rs:1-200`, `config_watcher.rs:195-220`,
`state.rs:844-1010`, `infra.rs:220-340, 440-873`,
`subagents/coding/explorer.md`.

Inspected partially (relevant slices only):

- `plugin_runtime.rs` outside the 820-870 window — read only the
  `register_plugin_agents` call sites and the candidate/previous swap
  error-recovery shape. The full plugin integrator lifecycle (load order,
  dependency resolution, monitor/theme/output-style wiring) is out of scope.
- `infra.rs:340-440` (workspace memory wiring) and `:873-1050`
  (`build_*_subagent_agent` helpers) — skimmed; they are construction
  plumbing that does not affect the catalog/refresh findings.
- Framework files (`echo-agent/src/agent/subagent/*`) — read-only; this
  task relies on F-SUB-01's full audit rather than re-inspecting the
  framework surface.

Not inspected (out of scope):

- The Tauri IPC handlers that surface the subagent catalog to the React
  frontend — only the Rust projection was inspected.
- The TUI subagent picker UI (if any) — `tasks/task_runtime/task_tools.rs`
  was inspected only at the catalog-construction surface
  (`TaskCapabilityCatalog::new`), not the full tool-result rendering.
- Worktree-isolation and data-workspace factory runtime behavior — owned
  by F-SUB-02; this task only verified the frontmatter → `isolate_*` mapping
  is wired through to the framework `SubagentBuilder` (`infra.rs:702-710`).

Environmental constraints:

- All 25 `subagent_loader::*` tests and 21 `subagent_prompt::*` /
  `prompt_contract::*` / `tasks::task_runtime::profiles::*` tests pass at
  `echo-agent-cli` commit `b3b2e81`. Worktree clean. The findings are
  based on static code inspection; no runtime scenario (e.g. actually
  installing a plugin, editing a `.md` at runtime, acquiring a pool agent)
  was exercised — the behavior claims follow directly from the cited code
  paths.
- The feature matrix beyond `echo-agent-app-core`'s default features was
  not re-run (F-FEAT-01 owns it).

Uncertain claims:

- Whether the `agent_tool` schema enum's description text alone (without
  the system-prompt `## Available Subagents` section) is rich enough for
  the LLM to discover plugin subagents in practice — this is an empirical
  LLM-behavior question, not a code question. The finding A-SUB-01-P2-03
  flags the divergence; whether it materially degrades plugin invocation
  rates would need a runtime probe.
- Whether any plugin in the wild ships subagents that shadow builtin
  names. The code path is silent on this; if no plugin does it today,
  A-SUB-01-P3-02 is latent rather than active.

## Handoff

Conclusions downstream tasks may rely on:

1. **The application layer reuses the framework's single Subagent
   lifecycle cleanly.** No parallel app-layer registry, dispatch tool, or
   result contract exists. `EkoSubagentPromptCompiler` implements the
   framework trait; `register_default_subagents` /
   `register_plugin_agents` both go through
   `register_subagent_with_definition` /
   `register_subagent_factory`; the result envelope is the framework's
   `render_result_contract()`. Downstream tasks can rely on F-SUB-01's
   "single authority" handoff holding at the application boundary.
2. **EKO closes the F-SUB-01 `compile_system` gap.** F-SUB-01-P2-02 flagged
   that the framework default `compile_system` is inert; this task
   confirms `EkoSubagentPromptCompiler::compile_system` IS invoked at
   `infra.rs:627-638` and produces the live system prompt. Any task
   re-examining `compile_system` liveness must account for the application
   compiler, not just the framework default.
3. **The catalog has three divergent surfaces.** The system-prompt
   `## Available Subagents` section (frozen at bootstrap), the
   `agent_tool` schema enum (live registry), and the TaskRuntime
   `TaskCapabilityCatalog` (live registry, but only on the primary agent).
   Plugin subagents land in the latter two, not the first. Any task
   touching LLM-facing subagent discovery must account for all three.
4. **Pool agents do not inherit plugin subagents.** Any task that adds
   plugin/subagent features must extend the propagation to pool agents
   (mirror `refresh_skill_descriptors`) or share the registry.
5. **`.md` edits require restart for the primary agent.** Pool agents
   lazily pick them up on `acquire`. The skill refresh pattern is the
   template for a future subagent refresh API.

Reports downstream tasks must read:

- This report (A-SUB-01) for the application-side catalog/registry/pool
  invariants and the three divergent catalog surfaces.
- `tasks/F-SUB-01.md` for the framework definition/registry/catalog/result
  invariants (this task assumes they hold at the framework layer).
- `tasks/F-SUB-02.md` for the framework execution-mode lifecycle (this
  task assumes the dispatch path is sound).
- `tasks/A-CFG-01.md` for the hot-reload boundary and workspace-switch
  scope (this task extends both to subagents).
- `validations/A-SUB-01/V01-01.md` through `V04-01.md` for per-claim
  evidence.

Conditions that make this report stale:

- Adding a subagent refresh API to `AgentPool` / `AppState` — resolves
  A-SUB-01-P2-01 / A-SUB-01-P2-02, requires re-running V04.
- Threading `SubagentRegistry` through `SharedResources` — resolves
  A-SUB-01-P2-01 (option a), requires re-running V01 / V04.
- Refreshing the system-prompt section after plugin registration —
  resolves A-SUB-01-P2-03, requires re-running V01.
- Re-running `validate_default_subagent_routes` against the live registry
  in `register_plugin_agents` — resolves A-SUB-01-P3-01, requires
  re-running V02.
- Documenting plugin precedence + logging shadow collisions — resolves
  A-SUB-01-P3-02, requires re-running V01.

Follow-up task IDs (no implementation in this review):

- **A pool/subagent parity task** should fix A-SUB-01-P2-01 (pool agents
  see plugin subagents) and A-SUB-01-P2-02 (reload mechanism). These are
  coupled — both need a way to push subagent definitions from the primary
  agent to pool agents, mirroring `refresh_skill_descriptors`.
- **A plugin-discoverability task** should fix A-SUB-01-P2-03 (system
  prompt catalog divergence) and A-SUB-01-P3-02 (silent shadowing). Both
  affect how plugins surface to the LLM.
- A-SUB-01-P3-01 (route gate coverage) can ride along with either of the
  above; it is the lowest-priority finding.
