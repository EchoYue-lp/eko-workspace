# X-PLG-01: Skill/plugin/hook lifecycle conformance

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only cross-cutting synthesis; both repos
> `git status --short` empty)

## Question

Are framework lifecycle primitives and EKO activation policy joined
through reversible, source-scoped, failure-safe adapters?

## Scope

This is a **cross-cutting synthesis task**. It consumes the three
dependency reports (F-SKL-01, F-PLG-01, A-PLG-01) and re-verifies the
adapter join they describe against the live code at the pinned commits.
Primary source paths inspected directly (not via the dependencies):

- `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:551-909` —
  the adapter join: `apply_candidate` (atomic swap with four rollback
  checkpoints), `replace_agent_components` (unload → wire → rewind-on-
  failure inside the agent write lock), `validate_agent_collisions`
  (pre-swap TOCTOU check), `unload_agent_components` (source-keyed
  removal of all four component types).
- `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:387-410` —
  `register_lifecycle`, the parallel lifecycle-registration entry point
  that bypasses `apply_candidate`.
- `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:287-343` —
  `uninstall` (disable-then-uninstall ordering, explicit
  `lifecycle.unregister`).
- `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:444-476` —
  `register_plugin_agents` (definition + instance + factory per agent,
  `SubagentKind::Plugin`).
- `echo-agent/echo-core/src/plugin/lifecycle.rs:65-260` —
  `PluginLifecycleManager` (register/activate/deactivate/unregister/
  shutdown), init-once + active flags, remove-before-cleanup ordering,
  `Drop` impl.
- `echo-agent/src/plugin.rs:128-395` — `PluginIntegrator::wire_all`
  (single framework wiring authority; skills/hooks/MCP wired and tagged;
  agents/LSP/monitors/themes/styles reported as application outputs).
- `echo-agent/src/agent/react/capabilities.rs:300-424` —
  `register_subagent_with_definition`/`register_subagent_factory`/
  `unregister_subagent` (no-op-safe removal; dispatch-catalog + tool-def
  cache invalidation).
- `echo-agent/src/agent/react/capabilities.rs:635-958` —
  `discover_skills_inner` (dual-registry registration + source tagging),
  `unregister_skills_by_source` (dual-registry + hooks + projections +
  progressive-tool refresh).
- `echo-agent/src/agent/react/capabilities.rs:1315-1332` —
  `disconnect_mcp` (enumerates adapted tool names, removes each, then
  disconnects the client).
- `echo-agent/src/agent/subagent/registry.rs:161-293` —
  `SubagentRegistry::register_sync`/`register_definition_sync`/
  `register_factory_sync`/`remove` (three-map cleanup: agents +
  definitions + factories + event emission).
- `echo-agent/echo-execution/src/tools.rs:528-547` —
  `ToolManager::register`/`unregister` (DashMap insert/remove + cache
  invalidation).
- `echo-agent/echo-execution/src/skills/hooks.rs:595-650` —
  `register_plugin_hooks` (`HookSource::Plugin` distinct identity),
  `unregister` (source-keyed removal).

## Out Of Scope

Deferred to named task IDs:

- Skill discovery internals (frontmatter parsing, DFS, progressive-
  disclosure tool contracts, sandbox policy plumbing) — owned by
  **F-SKL-01** (complete). This task consumes its contract and re-checks
  only the adapter-join invariants.
- Plugin manifest parsing, dependency resolution, registry persistence,
  scope resolution — owned by **F-PLG-01** (complete). This task
  consumes its contract and re-checks only the adapter-join invariants.
- EKO application-component preparation (agents/LSP/monitors/themes/
  output-styles), command-surface delegation, `HookConfigLoader` merge —
  owned by **A-PLG-01** (complete). This task consumes its contract and
  re-checks only the cross-cutting join.
- MCP transport lifecycle (connect/reconnect/cancel) — **F-INT-01**.
- LSP server process lifecycle internals — **F-INT-02**.
- Sandbox internals — **F-SEC-01**.
- Plugin-author DX concerns (fs-watch, async shutdown, validate/prepare
  skew) — already covered by A-PLG-01-P2-02 / P3-01 / P3-02; not
  re-audited here.

## Inputs

Required repository documents read in full:

- Root `AGENTS.md` via system reminder. Load-bearing sections for this
  task: the framework-vs-application layering gate (the "uncertain
  ownership defaults to application" rule, the "adapter must stay thin"
  rule, the "no parallel implementation of the same semantic" rule, the
  "adapter must not own ready frontier / DAG main loop / second
  validator" rule), the no-duplicate-authority rule, the dead-code
  cleanup rule, and the Subagent-only terminology rule.
- `docs/comprehensive-review/REPORTING.md`, `templates/task-report.md`,
  `templates/validation-report.md`.

Dependency task reports read in full:

- `zcode-glm/tasks/F-SKL-01.md` — establishes the skill loader/registry
  contract: single `SkillLoader` + `SkillRegistry`, source-tagged
  registration (`"plugin:{id}"`), dual-registry tagging (catalog +
  progressive), `unregister_skills_by_source` as the grouped-unload
  primitive, and the within-scope name-collision non-determinism
  (F-SKL-01-P2-01) + the `register_descriptor` overwrite-cleanup defect
  (F-SKL-01-P2-02). Load-bearing for V01/V02/V04: the skill-side of the
  adapter join.
- `zcode-glm/tasks/F-PLG-01.md` — establishes the plugin framework
  contract: single `PluginRegistry`/`PluginIntegrator`/
  `PluginLifecycleManager`, the 4-checkpoint rollback choreography in
  `apply_candidate`, source-scoped registration, the
  remove-before-cleanup lifecycle guarantee, and the plugin name-
  collision non-determinism (F-PLG-01-P2-01). Load-bearing for V01/V03:
  the framework-primitives side of the adapter join.
- `zcode-glm/tasks/A-PLG-01.md` — establishes the EKO application
  contract: single `PluginRuntimeService` orchestrator, GUI/CLI/TUI
  delegation to the same shared `Arc`, atomic swap semantics,
  `PreparedApplicationComponents` swappability, and the coarse all-or-
  nothing atomicity (A-PLG-01-P2-01). Load-bearing for V01/V02/V03: the
  EKO-policy side of the adapter join.

Historical documents treated as hypotheses:

- F-SKL-01 handoff note: "the plugin adapter's
  `unregister_skills_by_source` + `load_plugin_skills_from_dir` sequence
  is the only skill reload path and leaves no stale registrations on
  failure" — re-verified in V02/V04. Classified **current (supported)**.
- F-PLG-01 handoff note: "`PluginIntegrator::wire_all` +
  `unload_agent_components` is the single component wiring/unwinding
  authority" — re-verified in V01. Classified **current (supported)**.
- A-PLG-01 handoff note: "activation/reload failure rollback is
  comprehensive (four checkpoints, tested). No leaked registrations on
  the failure path" — re-verified in V03. Classified **current
  (supported) at the `apply_candidate` level**, with one cross-cutting
  consistency finding (X-PLG-01-P2-01) and one pub-API observation
  (X-PLG-01-P3-01).

## Layering Decision

This task spans both repositories but introduces no new code; it
synthesises the adapter-join classification across the four component
types (skills / hooks / MCP / Subagents).

| Classification | Answer |
|---|---|
| Generic mechanism (framework) | `SkillRegistry` + `SkillLoader`, `HookRegistry`, `McpManager`, `SubagentRegistry`, `ToolManager`, `PluginRegistry`, `PluginIntegrator::wire_all`, `PluginLifecycleManager`. Any `echo-agent` consumer needs these. Correctly placed in `echo-execution` / `echo-core` / `echo_agent` / root `agent`. None of them encode an EKO product decision. V01 confirms single definition sites and no parallel framework authority for any of the four component types. |
| EKO product policy (application) | `PluginRuntimeService::apply_candidate` (atomic swap + 4-checkpoint rollback choreography), `prepare_application_components` (agents/LSP/monitors/themes/styles), `register_plugin_agents` (Subagent definition+instance+factory), `HookConfigLoader` (three-source user-hook merge), `SkillsHub` (filesystem install/index UI). All correctly in `echo-agent-cli`. The framework never references these. |
| Adapter boundary | Thin and reversible. `PluginRuntimeService` calls `PluginRegistry` (scan/enable/disable/resolve), `PluginIntegrator::wire_all` (skills/hooks/MCP), `PluginLifecycleManager` (activate/deactivate/unregister), and framework `unregister_subagent` / `unregister_skills_by_source` / `disconnect_mcp`. It does NOT re-implement dependency resolution, source tagging, hook validation, or subagent dispatch. `PluginWiringResult.components_by_plugin` is the unload manifest passed verbatim to `unload_agent_components`; no transformation loss between wire and unload. The adapter owns exactly one application semantic: the atomic swap + rollback choreography (a product policy, not a duplicate authority). V02 confirms each lifecycle step is reversible; V03 confirms the rollback checkpoints. |
| Duplicate search | Searched both repos for `PluginRuntimeService`, `PluginIntegrator::new`, `wire_all`, `unload_agent_components`, `register_plugin_agents`, `register_subagent_with_definition`, `load_plugin_skills_from_dir`, `register_plugin_hooks`, `disconnect_mcp`, `unregister_skills_by_source`, `PluginLifecycleManager::register`. Result: one definition site each. `PluginIntegrator::new` appears only inside `replace_agent_components` (+ `#[cfg(test)]`); `register_subagent_with_definition` appears at `plugin_components.rs:471` (plugin path) and `infra.rs:847` (built-in bootstrap path — separate, non-plugin). No command surface (Tauri/CLI/TUI) builds its own integrator or registry. The two non-plugin registration paths (`runtime.rs:110` base MCP, `infra.rs:851` built-in Subagent) are bootstrap-time, not lifecycle operations; they do not bypass the plugin runtime. |
| Migration deletion | No migration proposed. No dead adapter code identified in the audited paths. X-PLG-01-P3-01 flags `register_lifecycle` as a pub API with zero production callers but does not recommend deletion (it is tested and contracts a future capability); the decision is the maintainer's. |

## Current Path

### 1. The adapter join (V01)

The join between framework primitives and EKO activation policy has
exactly one adapter (`PluginRuntimeService`) and one wiring authority
(`PluginIntegrator::wire_all` + `unload_agent_components`). The four
component types are owned as follows:

```text
COMPONENT   FRAMEWORK PRIMITIVE (owner)          EKO POLICY (adapter)                SOURCE TAG                  UNLOAD PRIMITIVE
Skills      SkillRegistry + SkillLoader          PluginRuntimeService                "plugin:{name}"             unregister_skills_by_source
            (echo-execution)                     → PluginIntegrator::wire_all         (tag_source_with_variables) (catalog + progressive + hooks
                                                 → load_plugin_skills_from_dir                                    + projections + tool refresh)
            Two registries: catalog (skill_registry) + progressive (progressive_skill_registry)
            — both tagged, both unloaded by unregister_skills_by_source.

Hooks       HookRegistry                          PluginRuntimeService                HookSource::Plugin(name)    hook_registry.unregister
            (echo-execution)                     → PluginIntegrator::wire_all         (register_plugin_hooks)     (&HookSource::Plugin(name))
                                                 → register_plugin_hooks
            User hooks: HookSource::UserConfig (single slot, HookConfigLoader three-source merge)
            Skill hooks: HookSource::Skill(name) — removed by unregister_skills_by_source too.

MCP         McpManager + connect_mcp_from_config  PluginRuntimeService                WiredPluginComponents       disconnect_mcp(server_name)
            / disconnect_mcp                      → PluginIntegrator::wire_all         .mcp_servers: Vec<String>   (per server: remove adapted tools
            (echo_agent)                          → load_mcp_config                                                  + disconnect client)
            Adapted tools registered as mcp__{server}__{tool} in ToolManager.

Subagents   SubagentRegistry                      PluginRuntimeService                PreparedApplicationComponents.agents  unregister_subagent(name)
            + register_subagent_with_definition   → register_plugin_agents            (PreparedPluginAgent.name)            (no-op-safe: remove from
            / register_subagent_factory           → register_subagent_with_definition                                        registry + catalog + cache)
            / unregister_subagent                 + register_subagent_factory
            (echo_agent)                          SubagentKind::Plugin { source }
            Three internal maps: agents + definitions + factories — all cleared by remove().
            Dispatch catalog (RwLock<Vec<...>>) + tool_def_cache invalidated on unregister.
```

No component type has a parallel registration authority. The adapter
delegates every registration to a framework primitive; the framework
never references the adapter. The two bootstrap registration paths
(`runtime.rs:110` base MCP, `infra.rs:851` built-in Subagent) are not
lifecycle operations — they run once at startup and are not owned by
`PluginRuntimeService`.

### 2. The lifecycle trace (V02)

Verified discover → prepare → activate → use → reload → unload at commit
`9b0e0fa` / `b3b2e81`:

| Step | Authority | Reversible? | Inverse |
|---|---|---|---|
| **Discover** | `PluginRegistry::scan_all` (framework) → `PluginManifest::from_file` per plugin dir | yes | `PluginRegistry::uninstall` (removes entry) |
| **Prepare** | `prepare_application_components` (EKO) — parses agents/LSP/monitors/themes/styles with `PluginVariables::substitute` | yes (pure parse, no side effects on live agent) | discard `PreparedApplicationComponents` |
| **Activate** | `apply_candidate` → `wire_all` (skills/hooks/MCP into live agent) + `register_plugin_agents` (Subagents) + `lifecycle.activate_enabled` | yes | `unload_agent_components` + `lifecycle.deactivate_all` |
| **Use** | model calls tools; Subagent dispatch via `FnAgentFactory`; skill activation via `ActivateSkillTool` | n/a | n/a |
| **Reload** | `reload()` → `apply_candidate` → `unload_agent_components(previous)` + `wire_all(candidate)` (full atomic swap, NOT in-place mutation) | yes (this IS the inverse of activate) | another `reload()` |
| **Unload** | `unload_agent_components` → `unregister_subagent` + `unregister_skills_by_source` + `hook_registry.unregister` + `disconnect_mcp` | yes (this IS the inverse of wire) | `wire_all` |

Each step is reversible. The critical property: **reload is a full
unload+rewire, not an in-place mutation**. `apply_candidate` (line 606-
620) `mem::replace`s the previous registry/framework/prepared out of
`state`, then `replace_agent_components` unloads the previous from the
live agent before wiring the candidate. There is no "patch the live
agent's tool set" path.

### 3. The failure-rollback checkpoints (V03)

`apply_candidate` (line 551-802) has eight failure checkpoints, each
with a defined rollback:

| # | Checkpoint | Failure action | Previous state preserved? |
|---|---|---|---|
| 1 | `resolve_enabled_dependencies` | return Err immediately (no swap) | yes — state untouched |
| 2 | `prepare_application_components` | return Err immediately (no swap) | yes — state untouched |
| 3 | `validate_agent_collisions` | return Err immediately (no swap) | yes — state untouched |
| 4 | `prepare_lsp` (start replacement servers) | `replacement_lsp.shutdown_all()`, return Err | yes — original LSP manager still in `self.lsp.manager` |
| 5 | `lifecycle.deactivate_all` | `replacement_lsp.shutdown_all()` + `lifecycle.activate_enabled(previous)`, return Err | yes — lifecycle re-activated, components not yet swapped |
| 6 | `replace_plugin_monitors` | `replacement_lsp.shutdown_all()` + `lifecycle.activate_enabled(previous)`, return Err | yes — monitors not yet swapped |
| 7 | `replace_agent_components` (wire candidate) | inner rollback: `unload_agent_components(candidate partial)` + `wire_all(previous)` + `register_plugin_agents(previous)`; then outer: restore `state.registry/framework/prepared`, `lifecycle.activate_enabled(previous)`, return Err | best-effort — see below |
| 8 | `lifecycle.activate_enabled(candidate)` (post-swap) | full reverse: `deactivate_all` + `replace_agent_components(candidate → previous)` + monitor rollback + LSP swap-back + `activate_enabled(previous)`, return Err | best-effort — same as #7 on the reverse path |

Checkpoints 1-6 are exact: no mutation has reached the live agent or
the persisted state, so the previous state is byte-identical.

Checkpoints 7-8 are best-effort: the rollback re-runs `wire_all` on the
previous registry, which may produce a subset if a transient failure
(MCP server down, skill file deleted) prevents a component from
re-wiring. The `restored.errors` are captured and appended to the
returned error message; `state.framework_components` is set to
`restored.components_by_plugin` (the actually-wired subset), so the
state stays consistent with the live agent. This is the correct design
— the alternative (snapshotting live tool registrations) is infeasible
because tools hold live network resources.

The inner rollback in `replace_agent_components` (line 823-843) is
notable: if `wire_all(candidate)` fails, it unloads the partially-wired
candidate, re-wires the previous, and re-registers previous Subagents.
If `register_plugin_agents(candidate)` fails (line 844-867), the same
unload+rewire+re-register path runs. In both cases, the candidate is
fully unloaded before the previous is restored — no partial candidate
state leaks into the live agent.

### 4. Stale-registration search (V04)

After `unload_agent_components` (line 1157-1176), each component type
is cleaned as follows:

| Component | Cleanup path | Stale-entry risk |
|---|---|---|
| Subagent | `unregister_subagent(name)` → `SubagentRegistry::remove` (clears agents + definitions + factories maps) + dispatch catalog `retain` + `tool_def_cache.invalidate` | none — three-map removal is exhaustive |
| Skill | `unregister_skills_by_source("plugin:{name}")` → `skill_registry.unregister_names_by_source` + `progressive_registry.unregister_by_source` + skill hooks (`HookSource::Skill(name)`) + context projections (`echo-agent:skill:{name}` + `SKILL_CATALOG_PROJECTION`) + 3 progressive-tool refresh | none — dual-registry + hooks + projections + tools all refreshed |
| Hook | `hook_registry.unregister(&HookSource::Plugin(name))` → `sources.remove(source)` | none — single HashMap entry removed |
| MCP | `disconnect_mcp(server)` → enumerate `McpToolAdapter::exposed_name_for(server, tool)` for each client tool, `remove_tool` each, then `mcp_manager.disconnect(name)` | none — adapted tools enumerated from the live client before disconnect |

Framework-owned tools (`activate_skill`, `read_skill_resource`,
`run_skill_scripts`, `agent_dispatch`) are refreshed (not removed) on
skill unload — correct, because they are framework singletons, not
plugin-owned. No plugin-owned tool name survives `unload_agent_components`.

The `uninstall` path (line 287-343) adds `lifecycle.unregister(name)`
which removes the lifecycle entry (remove-before-cleanup ordering, so a
failed shutdown callback does not block re-registration). The `disable`
path goes through `apply_candidate`, which calls `deactivate_all` +
`activate_enabled(candidate_without_disabled)` — the lifecycle entry
remains (deactivated) so re-enable can re-activate without re-registering.

## Findings

### X-PLG-01-P2-01: Collision non-determinism is systemic across both framework loaders and is inherited asymmetrically by the EKO adapter

- Priority: P2
- Confidence: high
- Layer: framework (echo-execution + echo-core), cross-cutting into the
  adapter
- Evidence:
  - `echo-agent/echo-execution/src/skills/external/loader.rs:198` —
    skill `scan_directory` iterates `tokio::fs::read_dir` output in
    filesystem order; `:147` resolves same-name collisions by
    first-observed-wins with a `warn!("shadowed by existing")` log.
  - `echo-agent/echo-core/src/plugin/registry.rs:190` — plugin
    `scan_scope_dir` iterates `std::fs::read_dir` in filesystem order;
    `:235` does `self.plugins.insert(id, entry)` with no collision check
    and no warning (F-PLG-01-P2-01).
  - `echo-agent/echo-core/src/plugin/registry.rs:130-134` — `scan_scopes`
    iterates `[User, Project, Local]` in order, so a later scope silently
    overwrites an earlier scope's same-named plugin (last-scanned-wins).
  - `echo-agent/echo-execution/src/skills/external/loader.rs:120-147` —
    `discover` iterates caller-provided scopes in order and the FIRST
    scope's descriptor wins (first-scope-wins). The two loaders therefore
    apply **contradictory** cross-scope precedence for the same
    conceptual operation.
  - `echo-agent/src/plugin.rs:262-285` — `PluginIntegrator::wire_all`
    loads plugin-owned skills via `load_plugin_skills_from_dir` in the
    order the skill loader returns them, inheriting the skill loader's
    non-determinism for any within-plugin same-name skill collision.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:551-558` —
    `apply_candidate` calls `candidate.resolve_enabled_dependencies()`
    which topologically sorts the plugin set in `scan_scope_dir` order;
    a within-scope collision makes the winner filesystem-dependent, so
    the adapter's notion of "which plugins are enabled" is
    non-deterministic on collision.
- Reachability: every `reload()` / `scan_all()` call exercises both
  loaders. The collision window opens whenever (a) two directories
  within one scope declare the same name (skill or plugin), or (b) two
  scopes install the same name. Live callers: all
  `PluginRuntimeService` mutator methods (`reload`/`enable`/`disable`/
  `install`/`uninstall`/`configure`).
- Expected invariant: the X-PLG-01 question asks whether the join is
  "source-scoped" and "failure-safe." Source-scoping requires that the
  adapter can identify and remove each plugin's components by source
  tag. If a same-named collision causes the loser's components to never
  load, the adapter's `unload_agent_components` for the loser's source
  tag is a no-op (the components were never wired) — but the adapter's
  `state.framework_components` records the loser's source tag as if it
  had loaded (because `wire_all` returns `components_by_plugin` keyed by
  plugin id, not by the actual content loaded). The divergence between
  "expected components" and "actually wired components" is the failure-
  safety gap.
- Observed behavior:
  - **Within-scope skill collision**: two skills named `code-review` in
    one plugin's `skills/` tree → winner depends on `read_dir` order
    (filesystem-dependent). The loser is never loaded; no error at the
    adapter level.
  - **Cross-scope plugin collision**: User `my-tool` + Project `my-tool`
    → Project silently overwrites User (last-scanned-wins). The User
    plugin's lifecycle, skills, hooks, and MCP are never wired. The
    adapter's `state.registry` has only one entry; `unload` for the
    User plugin's source tag is never attempted because the User plugin
    is not in the registry.
  - **Contradictory precedence**: a contributor fixing collision
    handling in the skill loader (first-scope-wins) will not know the
    plugin registry diverges (last-scoped-wins), and vice versa.
- Impact: latent correctness and maintainability defect. The adapter
  join is source-scoped and failure-safe for the NORMAL case (no
  collisions), but the collision case produces silent wrong-plugin/skill-
  loaded bugs that are filesystem-dependent and hard to reproduce. No
  security impact (both candidates are user-installed under trusted
  scopes). The contradictory precedence is a cross-cutting consistency
  hazard that F-SKL-01-P2-01 and F-PLG-01-P2-01 each see from one side;
  X-PLG-01 surfaces it as a single systemic pattern.
- Root cause: both loaders were written independently without a shared
  collision-resolution policy. `read_dir` ordering is not stable across
  filesystems; neither loader canonicalises it. The skill loader emits
  a warning; the plugin registry is silent.
- Direction: a single collision-resolution pass across both loaders,
  landing the fixes from F-SKL-01-P2-01 and F-PLG-01-P2-01 in one patch
  so the precedence direction and the warning behaviour agree. Recommend:
  (a) sort `read_dir` entries by path in both `scan_directory` and
  `scan_scope_dir`; (b) emit a `warn!` naming both paths on collision in
  both loaders; (c) agree on first-scope-wins (matching the skill
  loader's documented behaviour) for both. The fix belongs in
  `echo-execution/src/skills/external/loader.rs` and
  `echo-core/src/plugin/registry.rs`, not the adapter.
- Regression validation: new tests creating same-named skills under one
  root and same-named plugins across scopes; assert the winner is
  deterministic across repeated runs and a warning is logged. Run
  `cargo test -p echo_execution skills::external::loader` and
  `cargo test -p echo_core plugin::registry`.
- Validation reports: [V01-01](../validations/X-PLG-01/V01-01.md),
  [V02-01](../validations/X-PLG-01/V02-01.md),
  [V04-01](../validations/X-PLG-01/V04-01.md).

### X-PLG-01-P3-01: `register_lifecycle` is a pub adapter API with zero production callers; its deactivated-but-not-unregistered semantics are a latent stale-entry risk

- Priority: P3
- Confidence: high
- Layer: application (adapter pub-API surface)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:387-410` —
    `pub async fn register_lifecycle(...)` is a public method on
    `PluginRuntimeService` that registers lifecycle callbacks directly
    into `state.lifecycle` and synchronises activation, bypassing
    `apply_candidate`'s atomic swap.
  - Repository-wide grep for `register_lifecycle` in `echo-agent-cli`
    (excluding `mod tests`): the definition at `:387` is the only hit.
    All five call sites (`:1660`, `:1690`, `:1708`, `:1721`, `:1737`)
    are inside `#[cfg(test)] mod tests` (boundary at `:1360`). Zero
    production callers.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:574-588` —
    `apply_candidate` calls `state.lifecycle.deactivate_all()` which
    deactivates ALL registered entries (including those registered via
    `register_lifecycle`), then `state.lifecycle.activate_enabled
    (candidate_plugins)` re-activates only the candidate set. If a
    plugin registered via `register_lifecycle` is removed from the
    registry (e.g. by external deletion of its directory + `reload()`),
    its lifecycle entry is deactivated but never unregistered — it
    persists until `Drop`.
  - `echo-agent/echo-core/src/plugin/lifecycle.rs:153-179` — `unregister`
    removes the entry before running callbacks (remove-before-cleanup),
    so a stale entry cannot block re-registration. But the stale entry
    still occupies a slot in the `plugins` HashMap and holds an `Arc<
    dyn PluginLifecycle>`.
- Reachability: NOT reachable in production today (zero non-test
  callers). The risk materialises only if a future production caller
  invokes `register_lifecycle` without also adding an explicit
  `uninstall`/`unregister` path for plugins that leave the enabled set
  by means other than `uninstall` (e.g. directory deletion + `reload`).
  The `uninstall` method (`:333`) does call `lifecycle.unregister`, so
  the normal uninstall path is clean.
- Expected invariant: the X-PLG-01 question asks whether the join is
  "failure-safe." The adapter's atomic swap is failure-safe for the
  components it manages (skills/hooks/MCP/Subagents). But lifecycle
  callbacks registered via the parallel `register_lifecycle` entry point
  are NOT managed by the atomic swap — they survive `apply_candidate`'s
  deactivate/activate cycle as entries in the manager, and a plugin that
  leaves the registry by non-uninstall means leaves a stale (deactivated)
  entry behind.
- Observed behavior: in production, `register_lifecycle` is never
  called, so no stale entries arise. In tests, the
  `failed_native_lifecycle_registration_shuts_down_and_can_retry` test
  (`:1700`) confirms that a failed activation triggers `unregister`,
  which is correct. But no test covers the "plugin leaves the enabled
  set by non-uninstall means while lifecycle callbacks are registered"
  scenario, because no production code registers lifecycle callbacks.
- Impact: low today (zero production callers). The cost is a pub-API
  contract gap: a future contributor wiring native lifecycle callbacks
  into a plugin (the documented use case per the method's doc and the
  `PluginLifecycle` trait) will not get automatic cleanup on
  non-uninstall departure, and there is no test or doc warning about it.
- Root cause: `register_lifecycle` was added as a pub API for native
  lifecycle callbacks but was never wired into the production plugin
  load path (EKO plugins today are file-based, not native). The atomic-
  swap contract in `apply_candidate` does not account for entries
  registered outside it.
- Direction: three options. (a) Document on `register_lifecycle` that
  the caller is responsible for calling `uninstall` (which runs
  `lifecycle.unregister`) before the plugin leaves the registry, and
  that non-uninstall departure leaves a deactivated entry until Drop
  (cheapest). (b) Make `apply_candidate` call
  `lifecycle.reconcile(candidate_plugins)` instead of just
  `deactivate_all` + `activate_enabled`, so entries for plugins no
  longer in the candidate set are unregistered (not just deactivated).
  (c) If native lifecycle callbacks are not a near-term product goal,
  gate `register_lifecycle` behind `#[cfg(test)]` or mark it
  `#[doc(hidden)]` until a production consumer arrives. Recommend (a)
  now, (b) if/when a production caller is added.
- Regression validation: (a) doc-only; (b) new test registering a
  lifecycle callback, then removing the plugin dir + reloading, assert
  the lifecycle entry is unregistered (not just deactivated).
- Validation reports: [V01-01](../validations/X-PLG-01/V01-01.md),
  [V03-01](../validations/X-PLG-01/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Component ownership map: for each of skills/hooks/MCP/Subagents, a single framework primitive owns registration and a single EKO adapter owns activation policy; no parallel authority. | yes | passed | [V01-01](../validations/X-PLG-01/V01-01.md) |
| V02 | Full lifecycle discover→prepare→activate→use→reload→unload is reversible at each step; reload is a full unload+rewire (not in-place). | yes | passed | [V02-01](../validations/X-PLG-01/V02-01.md) |
| V03 | Failure rollback: eight checkpoints in `apply_candidate` each restore the previous state; inner `replace_agent_components` rollback unloads candidate before restoring previous. | yes | passed | [V03-01](../validations/X-PLG-01/V03-01.md) |
| V04 | Stale-registration search: after `unload_agent_components`, no plugin-owned tool/Subagent/hook/MCP entry survives in any framework registry. | yes | passed | [V04-01](../validations/X-PLG-01/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a | No prior X-PLG-01 report exists. The dependency handoff notes (F-SKL-01, F-PLG-01, A-PLG-01) are re-verified and classified under "Historical Claim Status". |

No cargo command was executed for this task: it is a static cross-
cutting synthesis that consumes the executable evidence of its three
dependencies (F-SKL-01 V01–V04, F-PLG-01 V01–V04, A-PLG-01 V01–V04) and
adds only static reachability grep + adapter-join inspection at the
pinned commits.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-SKL-01: "the plugin adapter's `unregister_skills_by_source` + `load_plugin_skills_from_dir` sequence is the only skill reload path" | current (supported) | V01 confirms `PluginIntegrator::wire_all` → `load_plugin_skills_from_dir` and `unload_agent_components` → `unregister_skills_by_source` are the only plugin-skill wire/unload paths; V04 confirms no stale skill registration survives. |
| F-SKL-01: "`register_descriptor` overwrite leaves stale `legacy_instructions`/`plugin_variables`" (F-SKL-01-P2-02) | current (supported, not reachable via adapter) | The adapter always calls `unregister_skills_by_source` before re-wiring, so the overwrite branch in `register_descriptor` is not exercised by the plugin path. The framework gap remains (F-SKL-01-P2-02); the adapter join does not amplify it. |
| F-PLG-01: "`PluginIntegrator::wire_all` + `unload_agent_components` is the single component wiring/unwinding authority" | current (supported) | V01 confirms one wiring authority; V02 confirms `unload_agent_components` is the inverse; V04 confirms no stale registration. |
| F-PLG-01: "plugin name collisions silently overwritten" (F-PLG-01-P2-01) | current (supported) | Re-verified at `registry.rs:235`; cross-cut into X-PLG-01-P2-01 (systemic with F-SKL-01-P2-01). |
| F-PLG-01: "activation failure rollback is comprehensive (four checkpoints)" | current (supported, expanded) | V03 confirms EIGHT checkpoints (F-PLG-01 counted four from the framework lens; the full `apply_candidate` has eight). All eight are correct. |
| A-PLG-01: "single `PluginRuntimeService`; GUI/CLI/TUI delegate to the same shared Arc" | current (supported) | V01 confirms no parallel orchestrator; the two non-plugin bootstrap paths (base MCP, built-in Subagent) are not lifecycle operations. |
| A-PLG-01: "`PreparedApplicationComponents` swappable as a whole; no transformation loss between wire and unload" | current (supported) | V02 confirms `wire_all`'s `components_by_plugin` is the unload manifest passed verbatim to `unload_agent_components`. |
| A-PLG-01: "no leaked registrations on the failure path" | current (supported at apply_candidate level) | V03 confirms checkpoints 1-6 are exact; checkpoints 7-8 are best-effort (re-run `wire_all`), with the candidate fully unloaded before the previous is restored. No partial candidate state leaks. |
| A-PLG-01: "Plugin shutdown is Drop-only with sync callbacks; async cleanup does not fire on exit" (A-PLG-01-P3-01) | current (supported) | `PluginLifecycleManager::Drop` (`lifecycle.rs:254-260`) calls `shutdown()` synchronously. The async `unload_agent_components` runs only on explicit reload/disable/uninstall. Not re-audited; deferred to A-PLG-01. |

## Coverage And Uncertainty

Inspected in full (directly, not via dependencies):

- `apply_candidate` (line 551-802), `replace_agent_components`
  (line 804-909), `unload_agent_components` (line 1157-1176),
  `validate_agent_collisions` (line 911-944), `register_lifecycle`
  (line 387-410), `uninstall` (line 287-343) — the entire adapter-join
  surface.
- `PluginIntegrator::wire_all` (line 128-395) — the framework wiring
  authority.
- `PluginLifecycleManager` (full, 260 lines + tests) — the framework
  lifecycle primitive.
- `unregister_skills_by_source` (capabilities.rs:915-958),
  `unregister_subagent` (capabilities.rs:406-424),
  `disconnect_mcp` (capabilities.rs:1315-1332) — the framework unload
  primitives.
- `SubagentRegistry::remove` (registry.rs:275-293) — confirms three-map
  cleanup.
- `ToolManager::register`/`unregister` (tools.rs:528-547) — confirms
  DashMap insert/remove + cache invalidation.
- `HookRegistry::register_plugin_hooks`/`unregister` (hooks.rs:595-650)
  — confirms source-keyed registration and removal.

Not inspected (out of scope or deferred):

- **`PluginRegistry::scan_scope_dir` internals** (manifest parsing,
  path validation) — owned by F-PLG-01; this task only re-checks the
  collision-insert path at `:235`.
- **`SkillLoader::scan_directory` internals** (frontmatter parsing, DFS)
  — owned by F-SKL-01; this task only re-checks the collision-resolution
  path at `:147`.
- **`prepare_application_components` internals** (JSON/YAML/frontmatter
  readers) — owned by A-PLG-01; this task only confirms the
  `PreparedApplicationComponents` is swappable.
- **Frontend plugin/skills panels** — A-PLG-01 confirmed the Rust IPC
  surface delegates; this task does not re-audit the frontend.
- **No executable cargo run.** This is a static synthesis task; the
  executable evidence is inherited from the three dependency tasks.

Environmental constraints:

- Both repos verified at the pinned commits (`echo-agent` 9b0e0fa,
  `echo-agent-cli` b3b2e81), both clean.
- No `cargo clean` needed (no build performed; static synthesis only).

Uncertain claims:

- The best-effort nature of rollback checkpoints 7-8 (re-run `wire_all`
  on the previous registry) means the restored state may be a subset if
  a transient failure prevents a component from re-wiring. This is the
  correct design (the state must reflect what is actually wired in the
  live agent), but it means "previous state restored" is not a byte-
  identical guarantee under transient failures. The `restored.errors`
  are surfaced in the error message. This is not promoted to a finding
  because it is the inherent semantics of rolling back live network
  resources, not a defect.
- The TOCTOU window between `validate_agent_collisions` (under
  `agent_handle.read()`) and `register_plugin_agents` (under
  `agent_handle.write_async()`) is theoretically reachable by a
  concurrent non-plugin subagent registration. A-PLG-01 already noted
  this and did not promote it (pathological interleaving, no security
  impact in the local-assistant model). This task concurs.

## Handoff

Conclusions downstream tasks may rely on:

1. **The adapter join is thin, source-scoped, and failure-safe.** One
   adapter (`PluginRuntimeService`), one wiring authority
   (`PluginIntegrator::wire_all` + `unload_agent_components`), one
   unload primitive per component type. All four component types are
   per-plugin source-tagged and reversibly removable. No parallel
   registration authority in either repo. X-BND-01 and any downstream
   task can rely on this.
2. **The lifecycle is fully reversible.** Discover → prepare → activate
   → use → reload → unload each have a defined inverse. Reload is a
   full unload+rewire (atomic swap), not an in-place mutation. Unload
   is the exhaustive inverse of wire (verified for all four component
   types).
3. **The failure rollback is comprehensive (eight checkpoints).**
   Checkpoints 1-6 are exact (no mutation before the checkpoint).
   Checkpoints 7-8 are best-effort (re-run `wire_all`), with the
   candidate fully unloaded before the previous is restored — no
   partial candidate state leaks into the live agent. The state always
   reflects what is actually wired.
4. **No stale registrations after unload.** `unload_agent_components`
   exhaustively clears Subagent (three-map), skill (dual-registry +
   hooks + projections + tools), hook (source-keyed), and MCP (adapted-
   tool-enumerate + disconnect) entries. Framework-owned tools are
   refreshed, not removed.
5. **The one systemic cross-cutting defect is collision non-determinism
   (X-PLG-01-P2-01).** Both framework loaders (skill + plugin) use
   `read_dir` order without sorting, and they apply contradictory
   cross-scope precedence (first-scope-wins vs last-scanned-wins). The
   adapter inherits both. This should be fixed in one coordinated pass
   across `echo-execution/src/skills/external/loader.rs` and
   `echo-core/src/plugin/registry.rs`, not in the adapter.
6. **`register_lifecycle` is a pub API with zero production callers
   (X-PLG-01-P3-01).** Its deactivated-but-not-unregistered semantics
   are a latent stale-entry risk if a production caller is added without
   a matching unregister path. Downstream tasks should not assume
   native lifecycle callbacks are wired in production today.

Reports they must read:

- This report (X-PLG-01) for the adapter-join synthesis and the two
  cross-cutting findings.
- `tasks/F-SKL-01.md` for the skill loader/registry contract and the
  skill-side collision non-determinism (F-SKL-01-P2-01) + overwrite-
  cleanup defect (F-SKL-01-P2-02).
- `tasks/F-PLG-01.md` for the plugin framework contract and the
  plugin-side collision non-determinism (F-PLG-01-P2-01).
- `tasks/A-PLG-01.md` for the EKO application contract, the coarse
  all-or-nothing atomicity (A-PLG-01-P2-01), and the Drop-only shutdown
  gap (A-PLG-01-P3-01).

Conditions that make this report stale:

- Any change to `apply_candidate`'s checkpoint sequence or to
  `replace_agent_components`'s inner rollback invalidates V03.
- Any change to `unload_agent_components` or to the source-tagging in
  `wire_all` / `discover_skills_inner` invalidates V02 and V04.
- Introduction of a parallel registration path (a second orchestrator,
  a direct `ToolManager::register` caller for plugin components, a
  second skill/hook registry) invalidates V01.
- Resolution of X-PLG-01-P2-01 (sorting `read_dir` + unifying
  precedence) will resolve the collision half of this report; the
  adapter join itself is unaffected.
- Addition of a production caller for `register_lifecycle` invalidates
  X-PLG-01-P3-01's "zero production callers" claim and should trigger
  the (b) fix direction (make `apply_candidate` call `reconcile`).

Follow-up task IDs (no fixes implemented in this review):

- A **collision-resolution coherence task** should action X-PLG-01-P2-01
  by landing the F-SKL-01-P2-01 and F-PLG-01-P2-01 fixes in one patch:
  sort `read_dir` entries in both loaders, emit a `warn!` on collision
  in both, and agree on first-scope-wins for both. This is the single
  highest-value cross-cutting fix.
- A **native-lifecycle wiring task** (if/when EKO adds native plugin
  callbacks) should action X-PLG-01-P3-01's (b) direction: make
  `apply_candidate` reconcile (not just deactivate) lifecycle entries so
  non-uninstall departure cleans up.
- Q-FLT-01 should run a fault-injection fixture for the best-effort
  rollback at checkpoints 7-8 (inject a transient MCP failure during
  the restore path, assert the error message names the failed component
  and `state.framework_components` reflects the actually-wired subset).
