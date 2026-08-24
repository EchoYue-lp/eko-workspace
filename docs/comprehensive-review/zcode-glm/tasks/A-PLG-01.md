# A-PLG-01: Skills, plugins, hooks, and reload lifecycle

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean

## Question

Does EKO discovery/activation/reload correctly apply product components while
framework registrations unload and roll back cleanly?

## Scope

Primary source paths and behaviors inspected (application layer; the framework
plugin primitives were audited in F-PLG-01 and are referenced, not re-audited):

- `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs` —
  `PluginRuntimeService` (sole orchestrator), `new`/`new_with_source`/`
  new_for_test`, `reload`/`enable`/`disable`/`install`/`uninstall`/`configure`/
  `register_lifecycle`/`bind_scheduler`, theme/output-style/active-preference
  accessors, `apply_candidate` (2-phase atomic swap + 4-checkpoint rollback),
  `replace_agent_components` (unload/wire/re-wire-on-failure inside agent write
  lock), `validate_agent_collisions`, `prepare_lsp`, `unload_agent_components`,
  `replace_plugin_monitors`/`rollback_plugin_monitors`, `fire_loaded_events`/
  `fire_plugin_disabled`, atomic preference persistence (tmp+rename, 0o600).
- `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs` (full,
  609 lines) — the application-owned component preparation that F-PLG-01
  deferred: `PreparedApplicationComponents`, `PreparedPluginAgent`,
  `prepare_application_components` (per-plugin agents/LSP/monitors/themes/
  output-styles with within-batch dedup HashSets), `validate_application_
  component_files`, `register_plugin_agents` (definition + instance + factory
  per agent), `build_plugin_agent`, `framework_definition`, the
  `read_*_with_variables` readers (JSON/YAML/frontmatter parsing with
  `PluginVariables::substitute`).
- `echo-agent-cli/echo-agent-app-core/src/hook_config_loader.rs` (full,
  379 lines) — the single user-hook loader (audit P0-1 fix): three-source
  merge (inline `echo-agent.yaml` + `~/.eko/hooks.yaml` + `.eko/hooks.yaml`)
  into one `HooksDefinition`, `load_merged`/`load_merged_from_disk`/
  `load_merged_from_disk_at`, `try_load_yaml` last-known-good preservation.
- `echo-agent-cli/echo-agent-app-core/src/config_watcher.rs` (header + scope)
  — read to confirm the hot-reload boundary (user hooks + webhook + ConfigChange
  hook) and that it has zero plugin integration.
- `echo-agent-cli/echo-agent-app-core/src/skills_hub/` (registry.rs, install.rs,
  mod.rs, enabled_skills.rs) — application install/index UI over `~/.eko/skills/`;
  `SkillsHub` projection with `loaded_skills` fed from runtime skill names.
- `echo-agent-cli/echo-agent-app-core/src/runtime.rs:279-386,499-591` —
  `PluginRuntimeService::new` single construction in `AgentRuntime::bootstrap`,
  `PluginLspRuntime` assembly, `with_plugin_runtime` hand-off to `AppState`.
- `echo-agent-cli/echo-agent-app-core/src/state.rs:443-449,592,621-625` —
  `AppState.plugin_runtime: Option<Arc<PluginRuntimeService>>` single handle,
  `AppState.skills_hub: Arc<RwLock<SkillsHub>>`.
- `echo-agent-cli/src/tauri/commands/plugins.rs` (full) — Tauri IPC surface;
  `require_service` resolves the shared `Arc`; 13 commands delegate.
- `echo-agent-cli/src/cli/cmd_impls/plugins.rs` (full) — CLI slash-command
  surface; resolves `ctx.plugin_runtime.as_ref()`; delegates to the same
  service.
- `echo-agent-cli/src/tui/events.rs:3305-3348` and
  `echo-agent-cli/src/tauri/commands/panels.rs:428-641` — TUI/Tauri skills
  panel feeding `set_loaded_skills` from `runtime_skill_names`.
- Framework touchpoints (referenced from F-PLG-01, re-read for the application
  contract only):
  - `echo-agent/echo-core/src/plugin/lifecycle.rs` — `PluginLifecycleManager`
    (register/activate/deactivate/unregister/shutdown), init-once + active
    flags, remove-before-cleanup, `Drop` impl.
  - `echo-agent/echo-execution/src/skills/hooks.rs:740-836,1092-1160` —
    `run_hooks` synchronous execution model; `execute_command_hook`.
  - `echo-agent/src/agent/react/capabilities.rs:300-424` —
    `register_subagent_with_definition`/`register_subagent_factory`/
    `unregister_subagent` (no-op-safe removal).

## Out Of Scope

- Framework plugin primitives (manifest, registry, dependency resolution,
  component resolution, scope, capability, variables, `PluginIntegrator::wire_all`)
  — owned by F-PLG-01; this task references its conclusions and re-reads only
  the application contract.
- MCP transport lifecycle (connect/reconnect/cancel) — F-INT-01; this task
  only confirms the plugin-driven MCP wiring + disconnect-on-unload contract
  (deferred from F-PLG-01).
- LSP server process lifecycle internals — F-INT-02; this task audits only the
  plugin-driven LSP replacement + `shutdown_all` on rollback.
- Skill discovery internals (frontmatter, DFS, progressive disclosure) —
  F-SKL-01 (read as dependency).
- Config watcher reload scope / workspace switch — A-CFG-01 (read as
  dependency); this task only confirms the plugin-runtime ↔ watcher boundary.
- Sandbox internals — F-SEC-01.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — framework-vs-application layering gate (the
    "uncertain ownership defaults to application" rule drives the
    `PluginRuntimeService` placement judgment), dead-code cleanup rule,
    "first search whether it already exists" rule, UTF-8/no-panic rules,
    local-personal-assistant threat model (informs the Drop-only-shutdown
    and no-fs-watch calibration of findings).
  - `docs/comprehensive-review/REPORTING.md`.
  - `docs/comprehensive-review/templates/task-report.md`,
    `templates/validation-report.md`.
- Dependency task reports read:
  - `F-PLG-01` (this reviewer) — relied on for the framework plugin
    primitive contract: single `PluginRegistry`/`PluginIntegrator`/
    `PluginLifecycleManager`, the 4-checkpoint rollback choreography in
    `apply_candidate`, source-scoped registration (`"plugin:{id}"` /
    `HookSource::Plugin(id)`), and the deferral note that A-PLG-01 should
    audit `plugin_components.rs` and the GUI/CLI/TUI delegation. This task
    extends that to the application-owned component path and confirms no
    divergent authority.
  - `F-SKL-01` (this reviewer) — relied on for the skill loader/registry
    contract and the handoff note that A-PLG-01 should verify `SkillsHub`
    reflects the framework registry state and does not maintain a divergent
    "loaded" set.
  - `A-CFG-01` (this reviewer) — relied on for the config watcher's hot-reload
    boundary (user hooks + webhook + ConfigChange hook only) and its
    workspace-switch gaps. This task confirms the watcher has zero plugin
    integration and that plugin reload is manual.
- Historical documents treated as hypotheses: none. No prior audit finding
  about the application plugin runtime was accepted as evidence; all claims
  are verified against code at the audited commits.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | N/A here (framework primitives audited in F-PLG-01). The framework `PluginLifecycleManager`, `PluginIntegrator`, `PluginRegistry`, and `HookRegistry` are the generic primitives; this task confirms EKO uses them without re-implementing them. |
| EKO product policy | Yes (correctly placed). `PluginRuntimeService` (`plugin_runtime.rs`), `PreparedApplicationComponents`/`register_plugin_agents` (`plugin_components.rs`), `HookConfigLoader` (`hook_config_loader.rs`), `SkillsHub` (`skills_hub/`), and the monitor/theme/output-style/LSP/preference orchestration are all EKO product decisions: local-assistant live-reload UX, atomic swap semantics, three-source hook merge, single-skill install UI. None of this belongs in the framework. This matches AGENTS.md's "uncertain ownership defaults to application". |
| Adapter boundary | Thin. `PluginRuntimeService` calls framework `PluginRegistry` (scan/enable/disable/resolve), `PluginIntegrator::wire_all` (skills/hooks/MCP), `PluginLifecycleManager` (activate/deactivate/unregister), and the framework `unregister_subagent`/`unregister_skills_by_source`/`disconnect_mcp`. It does NOT re-implement dependency resolution, source tagging, hook validation, or subagent dispatch. The application-owned semantic is the atomic swap + rollback choreography + application-component preparation (agents/themes/styles/monitors/LSP) — a product policy, not a duplicate authority. `PreparedApplicationComponents` is swappable as a whole (V03/V04 confirm); no transformation loss between wire and unload. |
| Duplicate search | Searched across the CLI tree: `PluginRuntimeService`, `PluginRegistry::new`, `PluginIntegrator::new`, `build_registry` (the pre-P0-4 helper), `register_plugin_agents`, `PreparedApplicationComponents`, `HookConfigLoader`, `load_merged`, `SkillsHub`, `set_loaded_skills`. Result: one definition site each. `PluginRegistry::new` appears only inside `PluginRuntimeService::registry_for` (+ `#[cfg(test)]`); `PluginIntegrator::new` only inside `replace_agent_components`. No command surface (Tauri/CLI/TUI) builds its own registry or integrator. `SkillsHub` is a filesystem install/index projection, NOT a runtime loader (delegates actual loading to the framework `SkillLoader` per F-SKL-01). `HookConfigLoader` is the single user-hook merge authority (the prior multi-source clear+register clobber was the P0-1 bug it fixed). |
| Migration deletion | No migration proposed in this task. No dead application code identified in the audited paths. |

## Current Path

Verified application plugin data flow at commit `9b0e0fa` / `b3b2e81`:

1. **Single service construction.** `PluginRuntimeService::new`
   (`runtime.rs:280`) is called once in `AgentRuntime::bootstrap`. The
   resulting `Arc<PluginRuntimeService>` is cloned into
   `AppState.plugin_runtime: Option<Arc<...>>` via `with_plugin_runtime`
   (`runtime.rs:386`). Constructor `new_with_source` (`plugin_runtime.rs:144`)
   loads preferences (`load_preferences`) and runs an initial `reload()` that
   on error keeps the (empty) previous runtime with a `warn!` (`:174-176`).

2. **Command delegation (no divergent state).** Tauri `require_service`
   (`tauri/commands/plugins.rs:83-89`) returns
   `state.app_state.plugin_runtime.clone()` or a hard error in headless mode.
   All 13 IPC commands call `service.{list,get,install,uninstall,enable,
   disable,configure,reload,themes,activate_theme,output_styles,
   activate_output_style}`. CLI `/plugins` (`cli/cmd_impls/plugins.rs`) uses
   `ctx.plugin_runtime.as_ref()` and the same methods. No surface constructs
   its own registry.

3. **Reload = fresh candidate + atomic swap.** `reload/enable/disable/install/
   configure` all acquire `state.lock()`, build a fresh candidate registry
   (`registry_for`), `scan_registry`, mutate the candidate, and call
   `apply_candidate(&mut state, candidate)`.

4. **`apply_candidate`** (`plugin_runtime.rs:551-802`) — the atomic swap:
   (a) `resolve_enabled_dependencies`; (b) `prepare_application_components`
   (agents/LSP/monitors/themes/styles); (c) `validate_agent_collisions`; (d)
   `prepare_lsp` (build + start replacement LSP manager); (e)
   `lifecycle.deactivate_all`; (f) `replace_plugin_monitors`; (g) swap
   registry/framework/prepared via `mem::replace`/`mem::take`; (h)
   `replace_agent_components` (unload previous → `wire_all` candidate →
   `register_plugin_agents`, all under agent write lock); (i) swap LSP
   manager; (j) `lifecycle.activate_enabled(candidate)`; (k) re-inject active
   output-style/theme projection; (l) persist preferences; (m)
   `fire_loaded_events`. Each of (e)/(f)/(h)/(j) has a rollback path that
   restores the previous state (covered in V03; framework lens in F-PLG-01 V03).

5. **Application component preparation** (`plugin_components.rs:96-222`).
   `prepare_application_components` iterates enabled plugins in topological
   order, resolves variables + components per plugin, and parses agents
   (`read_plugin_agent_with_variables`), LSP (`LspConfig::from_yaml`), monitors
   (`read_monitors_with_variables`), themes (`read_theme_with_variables`),
   output styles (`read_output_style_with_variables`). Within-batch dedup via
   `agent_names`/`lsp_languages`/`monitor_ids`/`theme_names`/`output_style_names`
   HashSets (`:103-107`). ANY error across ANY plugin → `Err(Vec<String>)`
   (`:217-221`).

6. **Subagent registration** (`plugin_components.rs:444-476`). For each
   `PreparedPluginAgent`: build instance (`build_plugin_agent`), then
   `register_subagent_with_definition(def, instance)` +
   `register_subagent_factory(def, factory)`. The factory (`FnAgentFactory`)
   rebuilds the agent on each isolated dispatch. Framework definition carries
   `SubagentKind::Plugin { source: plugin }` (`:485-487`).

7. **Activation events.** `fire_loaded_events` (`plugin_runtime.rs:1048`)
   fires `HookEvent::PluginLoaded` per candidate plugin via the live
   `hook_registry`. `fire_plugin_disabled` (`:1073`) fires
   `HookEvent::PluginDisabled` on disable/uninstall.

8. **Unload** (`plugin_runtime.rs:1157-1176`). `unload_agent_components`
   iterates `application.agents` → `unregister_subagent(name)` (no-op-safe on
   absent names) and `framework` entries →
   `unregister_skills_by_source("plugin:{name}")` +
   `hook_registry.unregister(&HookSource::Plugin(name))` +
   `disconnect_mcp(server)`. LSP replaced wholesale + `shutdown_all`;
   monitors via `replace_plugin_monitors`; themes/styles via `state.prepared`
   swap + projection clear.

9. **User-hook merge** (`hook_config_loader.rs`). The single loader merges
   inline `echo-agent.yaml` hooks + `~/.eko/hooks.yaml` + `.eko/hooks.yaml`
   additively into one `HooksDefinition`; caller does one
   `clear_user_hooks()` + `register_user_hooks(merged)`. `try_load_yaml`
   returns `Err` on parse failure so the reload caller preserves the
   last-known-good set rather than replacing with partial data.

10. **Hook execution model.** `HookRegistry::run_hooks`
    (`hooks.rs:792-836`) is a synchronous `await` loop over sources/rules;
    no `tokio::spawn`, no channel, no pending-task set. Hooks are
    fire-and-await; there is no queue to flush. The only flush obligation is
    the plugin author's sync `shutdown()` callback.

11. **Shutdown.** No explicit `plugin_runtime.shutdown().await` at app exit.
    `PluginLifecycleManager::Drop` (`lifecycle.rs:254-260`) calls `shutdown()`
    → `unregister` per plugin (sync deactivate + shutdown). Async
    `unload_agent_components` (MCP disconnect) runs only on explicit
    reload/disable/uninstall, not on Drop.

12. **Config watcher boundary.** The watcher (`config_watcher.rs`) reloads
    user hooks + webhook + ConfigChange hook only; it has zero plugin
    integration (grep for `plugin` returns no hits). Plugin component files
    are not watched; edits require manual `/plugins reload`.

## Findings

### A-PLG-01-P2-01: A single malformed application component in any plugin aborts the entire reload (coarse all-or-nothing atomicity across the enabled-plugin set)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:217-221` —
    `if errors.is_empty() { Ok(prepared) } else { Err(errors) }`: every
    component error across every plugin is collected into one `Vec<String>`
    and the whole call returns `Err` if any is non-empty.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:110-138` —
    per-plugin component reads push to the shared `errors` vec on failure and
    `continue`, so one plugin's bad theme does not stop parsing the next
    plugin, but ALL collected errors still fail the batch.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:570-571` —
    `apply_candidate` maps `prepare_application_components`'s `Err` to an
    `anyhow::Err` and returns immediately, before any swap — so the previous
    state is preserved (the atomicity is correct; the granularity is the
    issue).
- Reachability: every `reload()`/`enable()`/`disable()`/`install()`/
  `configure()` call funnels through `apply_candidate` →
  `prepare_application_components`. A user who installs plugin B with a
  malformed `themes/example.json` and then runs `/plugins reload` (or
  `/plugins enable A`) gets a failure naming plugin B's theme, and plugin A
  (healthy) is not reloaded either.
- Expected invariant: the task asks whether discovery/activation "correctly
  applies product components." Correctness of the atomic swap is not in
  doubt (V03 confirms rollback). The invariant at issue is operational: a
  single broken plugin should not blockade the entire plugin subsystem for
  every other plugin.
- Observed behavior: all-or-nothing across the full enabled-plugin set. One
  bad theme/LSP/monitor/output-style/agent file anywhere → the whole reload
  fails and the previous state is kept. The user must fix or disable the
  offending plugin to reload any other plugin. Compare the framework skill
  loader (F-SKL-01): a malformed SKILL.md is skipped with a `warn!` and
  discovery continues — the plugin application-component path is
  significantly stricter.
- Impact: operational fragility during plugin development and multi-plugin
  installs. No correctness or security impact (the previous state is
  preserved correctly). The asymmetry with the resilient skill loader is a
  maintainability hazard: a contributor will not expect one plugin's bad
  theme to block an unrelated plugin's enable.
- Root cause: `prepare_application_components` was written to collect errors
  across the whole batch and fail fast, prioritizing atomicity over
  per-plugin resilience. This is defensible (plugins have dependencies;
  partial loads could break a dependency chain), but the granularity — the
  entire enabled set — is coarser than necessary.
- Direction: two options. (a) Keep atomicity but make the offending plugin
  self-contained: drop only the failing plugin's components with a `warn!`
  and proceed with the rest (per-plugin partial failure), matching the skill
  loader's resilience. The dependency resolver already topologically orders
  plugins, so a failed plugin can be reported and its dependents skipped.
  (b) Keep the all-or-nothing behavior but improve the error message to name
  the offending plugin + component file path so the user can fix it quickly
  (today `errors` already carries the plugin name + path, so this is partly
  done — surface it in the Tauri/CLI `summary.errors`). Recommend (a) for
  resilience, (b) as the minimal fix. The fix belongs in
  `plugin_components.rs`/`plugin_runtime.rs`, not the framework.
- Regression validation: new test installing two plugins (A healthy, B with a
  malformed theme); `reload()` should load A and report B's failure as a
  non-fatal warning, not abort both. Run
  `cargo test -p echo-agent-app-core --lib plugin_`.
- Validation reports: [V02](../validations/A-PLG-01/V02-01.md),
  [V03](../validations/A-PLG-01/V03-01.md)

### A-PLG-01-P2-02: Plugin component files have no filesystem-watch integration — edits require manual `/plugins reload`

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:1-11` — the
    watcher doc explicitly scopes live reload to "user hooks and webhook
    endpoints" and states other domains "require a restart." Plugin component
    files are not in scope.
  - Grep for `plugin` in `config_watcher.rs` returns zero hits; grep for
    `notify::`/`RecommendedWatcher`/`plugin.*watch` across
    `echo-agent-app-core/src` returns only the config watcher — there is no
    separate plugin filesystem watcher.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:201-207` —
    `reload()` is the only path that re-scans plugin dirs and re-wires
    components; it is invoked by explicit `reload/enable/disable/install/
    configure` calls, never by a file event.
- Reachability: every plugin development workflow. A developer iterating on
  a plugin's `SKILL.md`, `hooks/hooks.yaml`, `agents/*.md`, `themes/*.json`,
  or `output-styles/*.md` saves the file and sees no change until they run
  `/plugins reload` (or restart the app).
- Expected invariant: the task asks whether EKO "reload" correctly applies
  product components. The reload mechanism itself works (V04 confirms);
  the invariant at issue is UX: a local-assistant user editing an extension
  expects the change to take effect, as it does for user hooks (which ARE
  watched).
- Observed behavior: plugin edits are invisible to the runtime until a manual
  reload. The asymmetry with user hooks (hot-reloaded by the watcher) is
  confusing: editing `~/.eko/hooks.yaml` takes effect immediately, but
  editing `<plugin>/hooks/hooks.yaml` does not.
- Impact: DX friction for plugin authors — the primary audience extending
  EKO. No correctness or security impact. The boundary is deliberate
  (A-CFG-01 documents the watcher's restart-required set), but plugin
  component reload is achievable via the existing `PluginRuntimeService::
  reload()` primitive; it is just not wired to a watcher.
- Root cause: the watcher was scoped to user-config domains and never
  extended to plugin dirs. This is a product decision (the watcher doc warns
  against widening without a teardown story), but the teardown story already
  exists: `PluginRuntimeService::reload()` atomically swaps all components.
- Direction: add a debounced filesystem watcher over the resolved plugin
  scope dirs (User/Project/Local) that calls `PluginRuntimeService::reload()`
  on settle, reusing the config watcher's parent-dir-watch + rename-save-safe
  + resettable-debounce patterns. Gate it behind a preference (`plugins.
  auto_reload`) so users with large plugin trees can opt out. The framework
  provides the reload primitive; the application owns the watcher wiring.
- Regression validation: edit a plugin's `SKILL.md`, assert the runtime
  re-discovery fires within the debounce window without a manual reload;
  edit a malformed file, assert the previous state is preserved (V03
  rollback).
- Validation reports: [V04](../validations/A-PLG-01/V04-01.md)

### A-PLG-01-P3-01: Plugin shutdown relies on `Drop` with no explicit ordered async teardown — MCP disconnect and async cleanup are skipped on process exit

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence:
  - `echo-agent/echo-core/src/plugin/lifecycle.rs:254-260` —
    `Drop for PluginLifecycleManager` calls `shutdown()` which calls
    `unregister` per plugin; the `shutdown`/`deactivate` callbacks are sync
    (`fn(&self) -> Result<(), String>`).
  - `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:1157-1176` —
    `unload_agent_components` (async: `unregister_subagent().await`,
    `unregister_skills_by_source().await`, `disconnect_mcp().await`,
    `hook_registry.write().await`) runs only inside `replace_agent_components`
    (explicit reload/disable/uninstall), not on Drop.
  - Grep of `runtime.rs`/`state.rs` for an explicit plugin shutdown call
    returns nothing — the application relies on `Arc<PluginRuntimeService>`
    drop to trigger `PluginLifecycleManager::Drop`.
- Reachability: every process exit. When `AppState`/`AgentRuntime` is
  dropped, the `Arc<PluginRuntimeService>` refcount drops; if it reaches
  zero, `PluginRuntimeState` (inside `Mutex`) drops, and
  `PluginLifecycleManager::Drop` fires sync shutdown callbacks.
- Expected invariant: the task asks "what happens to hooks on shutdown? Are
  queues flushed?" The hook registry has no queue (V04 confirms hooks are
  synchronous), so flush is a no-op there. The invariant at issue is plugin
  author cleanup: a plugin with async flush requirements (await an MCP
  `shutdown` RPC, await a DB commit, await a child-process graceful exit)
  has no deterministic path on process exit.
- Observed behavior: on Drop, only sync `shutdown()` runs. Async MCP
  disconnects do not fire (sockets are killed by the OS). Plugin authors
  needing async cleanup must put it in `deactivate()` (sync, fires on every
  reload/disable) rather than `shutdown()`. The `shutdown()` doc
  (`lifecycle.rs:43-49`) promises "flush buffers, close connections, save
  state" but the Drop-only call site cannot await any of those if they are
  async.
- Impact: low for the local-personal-assistant threat model (OS reclaims
  resources; MCP servers are local subprocesses that tolerate abrupt
  disconnect). The cost is a contract gap: plugin authors who reasonably
  implement async cleanup in `shutdown()` per the doc will find it does not
  run as they expect on exit.
- Root cause: the application never calls an explicit ordered shutdown; it
  relies on Drop, which cannot run async work. The lifecycle trait's
  callbacks are sync by design.
- Direction: either (a) document on `PluginLifecycle::shutdown` that it must
  be sync and that async cleanup belongs in `deactivate()` (cheapest), or
  (b) add an explicit `plugin_runtime.shutdown().await` call at app exit
  (before the runtime is dropped) that runs `unload_agent_components` (MCP
  disconnect, hook unregister) + `lifecycle.shutdown()` in order. Option (b)
  gives plugin authors a deterministic async-capable teardown but requires
  wiring a shutdown signal through `AgentRuntime`/`AppState`. For the local
  threat model, (a) is sufficient.
- Regression validation: (a) add a doc test / rustdoc note on `shutdown`;
  (b) start an MCP server via a plugin, exit the app, assert the server
  receives a clean disconnect (transport-level) rather than a SIGKILL.
- Validation reports: [V04](../validations/A-PLG-01/V04-01.md)

### A-PLG-01-P3-02: `validate()` does not apply plugin variables while `prepare()` does — validate/prepare skew can give a false "valid"

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:224-258` —
    `validate_application_component_files` calls `read_plugin_agent` /
    `read_monitors` / `read_theme` / `read_output_style` (the non-variable
    variants, which pass `None` to `read_component_text`).
  - `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:96-222` —
    `prepare_application_components` calls the `_with_variables` variants
    with `Some(&variables)` from `registry.variables_for(&plugin)`.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:436-442` —
    `read_component_text` only substitutes when `variables` is `Some`.
- Reachability: any user running `/plugins validate <dir>` (or the Tauri
  `validate_plugin` IPC) before install/configure. The validate path parses
  component files with raw `${ECHO_PLUGIN_DATA}` / `${user_config.*}`
  placeholders; the prepare path substitutes them first.
- Expected invariant: `validate()` is the pre-flight check advertised to
  plugin authors; its verdict should predict whether `prepare()` will succeed
  on reload.
- Observed behavior: validate reports the file structurally valid (a raw
  `${...}` is a legal JSON/YAML string); prepare substitutes and may then
  fail if the substituted value is structurally invalid (rare) — or, more
  commonly, validate flags a file that prepare would have accepted after
  substitution. The skew is narrow because variable expansion rarely changes
  structural validity, but it exists.
- Impact: low. `validate()` is a developer-facing diagnostic, not a gate
  (install does not require it). A false "valid" erodes trust in the
  diagnostic; a false "invalid" confuses the author. No runtime effect.
- Root cause: `validate()` was written to run pre-install (when variables /
  user config may not yet be available), so it intentionally skips
  substitution. The tradeoff is reasonable but undocumented.
- Direction: either (a) document on `PluginRuntimeService::validate` that it
  checks structure pre-substitution and that final validity is re-checked at
  reload (cheapest), or (b) have `validate` build a variables snapshot from
  manifest defaults + resolved paths and run the `_with_variables` readers,
  so the verdict matches prepare. Option (a) is the low-risk fix.
- Regression validation: a plugin theme with `"accent":
  "${user_config.accent}"` and a matching config default; assert validate +
  prepare agree.
- Validation reports: [V02](../validations/A-PLG-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Prepare/activate ownership: single `PluginRuntimeService`, framework lifecycle used correctly (init-once, activate-per-swap, remove-before-cleanup), GUI/CLI/TUI delegate to the same shared Arc. | yes | passed | [V01-01](../validations/A-PLG-01/V01-01.md) |
| V02 | Real component registration: skills/hooks/MCP wired by framework `wire_all`; Subagents registered as definition+instance+factory; themes/styles/monitors/LSP applied to live subsystems; `PluginLoaded` fires. | yes | passed | [V02-01](../validations/A-PLG-01/V02-01.md) |
| V03 | Failed activation rollback: prepare-phase failure aborts before swap; Subagent partial registration safely unwound (no-op-safe unregister); post-swap activation failure full reverse swap; lifecycle remove-before-cleanup. | yes | passed | [V03-01](../validations/A-PLG-01/V03-01.md) |
| V04 | Reload/unload and hook flush: full load→disable→enable→uninstall reversible; hooks synchronous (no queue); `HookConfigLoader` single-slot 3-source merge; `Drop` runs sync shutdown; preferences survive restart. | yes | passed | [V04-01](../validations/A-PLG-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | No historical document is cited as evidence for any claim in this report. |

All 10 `plugin_` tests in `echo-agent-app-core`, all 7 `hook_config_loader`
tests, and all 4 framework `plugin::lifecycle` tests pass at the audited
commits (see V01-01 and V04-01 for commands and results).

## Historical Claim Status

No historical documents are cited as evidence for any claim in this report.
All findings are based on code at commit `9b0e0fa` / `b3b2e81` and the four
validation reports. The `hook_config_loader.rs` module header (`:1-28`)
documents the prior P0-1 multi-source clobber bug as fixed design context,
not as evidence about current behavior; this report verifies the fix holds.

## Coverage And Uncertainty

- Code not inspected:
  - `echo-agent-cli/echo-agent-app-core/src/skills_hub/install.rs` (full
    git-install path) — read at the signature level (`sync_skills`,
    `check_updates`, `read_source_record`, `validate_subdir` per F-PLG-01).
    Confirmed it is a single-skill git installer with its own
    `validate_subdir`, NOT a parallel plugin registry. The install/sync
    internals are out of scope for the reload-lifecycle question.
  - `echo-agent-cli/echo-agent-app-core/src/skills_hub/enabled_skills.rs` —
    the enabled-skills persistence (`EnabledSkillsConfig`). Read at the
    signature level; it persists the user's enabled-skills set to
    `~/.eko/enabled-skills.json` and is consumed by the skills panel, not by
    the plugin runtime.
  - `echo-agent-cli/src/tui/` plugin/skills rendering beyond
    `events.rs:3305-3348` — the TUI delegate path was inspected; the visual
    rendering is out of scope.
  - Web-frontend plugin/skills panels (`web-frontend/src/...`) — only the
    Rust IPC surface (`tauri/commands/plugins.rs`, `panels.rs`) was audited.
- Validations not executed at runtime: V01 is a static inspection (no
  `cargo test` in the report itself; the supporting test run is in V02/V04).
  V04's filesystem-watch claim is a negative confirmation (grep showing no
  watcher) — a live fault-injection (edit a plugin file, observe no reload)
  belongs to Q-FLT-01.
- Environmental limits: none. Both repos are clean at the audited commits.
  Disk was at ~47 GiB free / 37 GiB of `target/` during the review; no
  `cargo clean` was needed (incremental builds were no-ops at these commits).
  Tests ran on darwin 25.5.0 arm64.
- Claims that remain uncertain:
  - The TOCTOU window between `validate_agent_collisions`
    (`plugin_runtime.rs:911`, under `agent_handle.read()`) and
    `register_plugin_agents` (under `agent_handle.write_async()`) is
    theoretically reachable if a non-plugin caller registers a colliding
    subagent between the two lock acquisitions. In practice
    `apply_candidate` holds `state.lock()` throughout, so no plugin
    operation can interleave; only a non-plugin concurrent registration
    could win the race, and `register_subagent_with_definition` would then
    silently overwrite. This is not promoted to a finding because the
    consequence (a plugin subagent shadowing a concurrently-registered
    non-plugin subagent of the same name) requires a pathological
    interleaving and has no security impact in the local-assistant model.
  - The cross-filesystem determinism of `SkillsHub::scan`
    (`skills_hub/registry.rs:184`, `std::fs::read_dir` order) is the same
    class as F-SKL-01-P2-01 / F-PLG-01-P2-01 but is display-only (the hub
    sorts entries by name in `list()`, `:147-149`), so it does not affect
    which skill loads — only iteration order during scan, which is
    deduped-by-name into the entries map.

## Handoff

- Conclusions downstream tasks may rely on:
  - There is exactly one application orchestrator (`PluginRuntimeService`).
    GUI (Tauri), CLI, and TUI all delegate to the same shared
    `Arc<PluginRuntimeService>`; no surface maintains a divergent
    enabled/loaded set. X-BND-01 / X-PLG-01 can rely on this.
  - Activation registers real, live components across all eight categories
    (skills/hooks/MCP/Subagents/LSP/monitors/themes/output-styles). Subagents
    get definition+instance+factory; application-owned components reach their
    live subsystems. F-SUB-01/F-SUB-02 can rely on the plugin-Subagent
    registration contract (`SubagentKind::Plugin`, factory-backed dispatch).
  - `SkillsHub` is a filesystem install/index projection, NOT a runtime
    loader; it delegates actual loading to the framework `SkillLoader` (per
    F-SKL-01) and is fed `loaded_skills` from `runtime_skill_names` where the
    runtime is accessible (Tauri/TUI). F-SKL-01's handoff concern is
    resolved: no divergent runtime authority.
  - `HookConfigLoader` is the single user-hook merge authority; the prior
    multi-source clear+register clobber (P0-1) is fixed. F-HITL-01 and any
    hook consumer can rely on the three sources landing in one
    `HookSource::UserConfig` slot without mutual destruction.
  - Activation/reload failure rollback is comprehensive (four checkpoints,
    tested, no leaked registrations) — confirmed by F-PLG-01 V03 for the
    framework lens and A-PLG-01 V03 for the application-component lens.
  - Plugin shutdown is Drop-only with sync callbacks; async cleanup
    (including MCP disconnect) does not fire on process exit. F-INT-01 (MCP)
    should account for this: plugin-sourced MCP servers are disconnected on
    explicit reload/disable/uninstall but not on app exit.
- Reports they must read:
  - [V01-01](../validations/A-PLG-01/V01-01.md) for the single-service
    ownership and command-surface delegation.
  - [V02-01](../validations/A-PLG-01/V02-01.md) for real component
    registration (definition+instance+factory Subagents; live-subsystem
    application of themes/styles/monitors/LSP).
  - [V03-01](../validations/A-PLG-01/V03-01.md) for the application-component
    rollback lens (prepare-phase abort, Subagent partial-registration
    safety, post-swap reverse swap).
  - [V04-01](../validations/A-PLG-01/V04-01.md) for the full reload cycle,
    synchronous hook model, `HookConfigLoader` merge, and Drop-only
    shutdown.
- Conditions that make this report stale:
  - Any change to `prepare_application_components`'s error aggregation
    invalidates A-PLG-01-P2-01 and V02/V03.
  - Introduction of a plugin filesystem watcher invalidates A-PLG-01-P2-02
    and the V04 "no fs-watch" claim.
  - Any change to `PluginLifecycleManager::Drop` or the addition of an
    explicit async shutdown path invalidates A-PLG-01-P3-01 and the V04
    shutdown paragraph.
  - Any change to `validate_application_component_files` vs
    `prepare_application_components` variable handling invalidates
    A-PLG-01-P3-02.
  - Any change to `unload_agent_components` or `register_plugin_agents`
    invalidates V02 and V03.
- Follow-up task IDs (no fixes implemented in this review):
  - Q-FLT-01 should run a fault-injection fixture for A-PLG-01-P2-01
    (two plugins, one with a malformed component, assert the healthy one
    still loads or the failure is clearly attributed) and for A-PLG-01-P2-02
    (edit a plugin file, observe whether a reload fires).
  - F-INT-01 owns the MCP transport boundary; this task confirmed
    plugin-sourced MCP servers are disconnected on explicit unload but not
    on Drop (A-PLG-01-P3-01) — F-INT-01 should decide whether that is
    acceptable for the MCP clean-disconnect contract.
  - A future "plugin author DX" task could batch A-PLG-01-P2-02 (fs-watch),
    A-PLG-01-P3-01 (shutdown doc/async path), and A-PLG-01-P3-02 (validate
    doc) as a coherence pass.
