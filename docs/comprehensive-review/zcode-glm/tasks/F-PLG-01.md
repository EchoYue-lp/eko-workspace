# F-PLG-01: Plugin manifest, registry, and lifecycle

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean

## Question

Does the plugin framework resolve dependencies, component ownership,
activation, replacement, unloading, and rollback without leaked
registrations?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-core/src/plugin/manifest.rs` — `PluginManifest` struct,
  serde defaults, `from_yaml`/`from_file`/`validate`/`validate_paths`,
  `PluginDependency` (`Simple`/`Versioned`) + `satisfies` (semver
  `VersionReq`), `inferred_capabilities`, `resolve_user_config`,
  `validate_user_config`, `validate_config_value`, kebab/identifier/semver
  helpers.
- `echo-agent/echo-core/src/plugin/registry.rs` — `PluginRegistry`,
  `PluginEntry`, `ResolvedComponents`, `scan_all`/`scan_scopes`/
  `scan_scope_dir`, `install`/`install_local`/`install_git`, `uninstall`,
  `enable`/`disable`, `ensure_no_enabled_dependents`, `configure`,
  `resolve_components`, `resolve_dependencies`/`resolve_enabled_dependencies`
  (Kahn's topological sort + cycle detection + version enforcement),
  `save_state`/`load_state` (atomic tmp+rename, 0o600), `validate_plugin_dir`.
- `echo-agent/echo-core/src/plugin/lifecycle.rs` — `PluginLifecycle` trait,
  `PluginLifecycleManager` (`register`/`activate`/`deactivate`/`deactivate_all`/
  `unregister`/`reconcile`/`shutdown`), `Drop` impl, init-once + active flags,
  remove-before-cleanup ordering.
- `echo-agent/echo-core/src/plugin/scope.rs` — `PluginScope` (User/Project/
  Local), `resolve_dir`, `InstallSource` (`Local`/`Git`), `parse`.
- `echo-agent/echo-core/src/plugin/capability.rs` — `PluginCapability` enum
  (9 variants), `from_str_loose`, `display_name`.
- `echo-agent/echo-core/src/plugin/variables.rs` — `PluginVariables`
  (`substitute`, `resolve_path`, `ensure_data_dir`, `data_dir_for`),
  `substitute_env_vars`, `export_to_env`.
- `echo-agent/echo-core/src/plugin/mod.rs` — module facade, configurable base
  dir `OnceLock` (`plugin_data_base_dir`/`set_plugin_data_base_dir`/
  `set_plugin_data_base_dir_name`), `home_dir` helper.
- `echo-agent/src/plugin.rs` — root facade re-exports, `PluginIntegrator`
  (`wire_all`/`wire_skills`/`wire_hooks`/`wire_mcp`), `PluginWiringResult`,
  `WiredPluginComponents`, legacy `NativePlugin` trait.
- `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs` —
  `PluginRuntimeService` (`reload`/`enable`/`disable`/`install`/`uninstall`/
  `configure`/`register_lifecycle`), `apply_candidate` (2-phase rollback
  orchestrator), `replace_agent_components`, `unload_agent_components`,
  `replace_plugin_monitors`/`rollback_plugin_monitors`, `validate_agent_collisions`,
  `prepare_lsp`, preference persistence.
- `echo-agent/src/agent/react/capabilities.rs:635-960` — `discover_skills_inner`
  (plugin source tagging to both registries), `load_plugin_skills_from_dir`,
  `unregister_skills_by_source`, `load_mcp_config`/`disconnect_mcp`.
- `echo-agent/echo-execution/src/skills/registry.rs:85-180` —
  `register_descriptor` (reverse-index cleanup on overwrite),
  `unregister_by_source`/`unregister_names_by_source`, `tag_source`/
  `tag_source_with_variables`.
- `echo-agent/echo-execution/src/skills/hooks.rs:595-655` —
  `register_plugin_hooks` (`HookSource::Plugin`), `unregister`.
- `echo-agent/src/hooks_bridge.rs` — task/subagent hook bridges (read for
  layering context; not a plugin-hooks path).

## Out Of Scope

- MCP transport lifecycle (connect/reconnect/cancel) — F-INT-01 owns the
  transport boundary; this task only audited the plugin→MCP wiring +
  disconnect-on-unload contract.
- LSP server process lifecycle internals — F-INT-02 owns LSP; this task
  audited only the plugin-driven LSP replacement + `shutdown_all` on rollback.
- SkillsHub application install/index UI (`skills_hub/install.rs`) — A-PLG-01
  territory; read only to confirm it is a separate single-skill installer,
  not a parallel plugin registry.
- Sandbox internals — F-SEC-01.
- Skill discovery internals (frontmatter, DFS, progressive disclosure) —
  F-SKL-01 (read as dependency).

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — framework-vs-application layering gate, dead-code
    cleanup rule, "first search whether it already exists" rule, UTF-8/no-panic
    rules, Subagent-only terminology (N/A here), local-personal-assistant
    threat model (informs the security-priority calibration of findings).
  - `docs/comprehensive-review/REPORTING.md`.
  - `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
- Dependency task reports read:
  - `F-SKL-01` (this reviewer) — relied on for the skill loader/registry
    contract (`load_plugin_skills_from_dir`, `tag_source`,
    `unregister_by_source`, progressive-disclosure tool refresh,
    within-scope name-collision non-determinism F-SKL-01-P2-01, and the
    `register_descriptor` overwrite-cleanup defect F-SKL-01-P2-02). This task
    extends those to the plugin-owned skill path and confirms the plugin
    integrator tags both registries.
  - `B-REF-01` (this reviewer) — relied on for the cross-system convergence
    that permission/approval is launch-mode + isolation, not a runtime state
    machine (B-REF-01-P3-01), and that plan/skill/plugin hooks fire through a
    unified typed surface (B-REF-01-P1-02). The plugin lifecycle callbacks
    (init/activate/deactivate/shutdown) are the native-code analogue of the
    YAML hook surface; both flow through the same source-tagged registry.
- Historical documents treated as hypotheses: none. No prior audit finding
  about the plugin framework was accepted as evidence; all claims are
  verified against code at the audited commits.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. Manifest parsing (YAML schema, path validation, semver dependency constraints), topological dependency resolution with cycle detection, source-scoped component registration + grouped unload, lifecycle callback manager (init/activate/deactivate/shutdown with remove-before-cleanup), atomic state persistence (tmp+rename, 0o600), configurable base-dir `OnceLock`, and the `PluginIntegrator` wiring loop are all generic capabilities any `echo-agent` consumer needs. They live correctly in `echo-core` (types/resolution) and the facade `echo_agent` (integration glue that touches `ReactAgent` + `HookRegistry` + `McpManager`). V01 confirms single definition sites; no parallel plugin framework exists. |
| EKO product policy | The `PluginRuntimeService` (`plugin_runtime.rs`) is correctly application-layer: it owns the atomic reload/rollback orchestration (2-phase `apply_candidate`), the LSP/monitor/theme/output-style application-owned components, the scheduler binding, and the preference persistence. These are EKO product decisions (local-assistant live-reload UX, atomic swap semantics). The framework provides the primitives (registry, lifecycle manager, integrator); the application owns the orchestration policy. This matches AGENTS.md's "uncertain ownership defaults to application". |
| Adapter boundary | Thin. `PluginRuntimeService` calls `PluginRegistry` (scan/install/enable/disable/resolve), `PluginIntegrator::wire_all` (skills/hooks/MCP wiring), `PluginLifecycleManager` (activate/deactivate/unregister), and `unload_agent_components` (source-keyed removal). It does NOT re-implement dependency resolution, source tagging, or hook validation — those are framework-owned. The only application-owned semantic is the atomic swap + rollback choreography, which is a product policy, not a duplicate authority. `PluginWiringResult.components_by_plugin` is the unload manifest passed verbatim to `unload_agent_components`; no transformation loss. |
| Duplicate search | Searched across both repos: `PluginRegistry`, `PluginManifest`, `PluginLifecycle`, `PluginLifecycleManager`, `PluginCapability`, `PluginScope`, `InstallSource`, `PluginIntegrator`, `PluginVariables`, `NativePlugin`, `PluginEntry`, `ResolvedComponents`, `PluginWiringResult`, `WiredPluginComponents`, `register_plugin_hooks`, `unregister_by_source`, `load_plugin_skills_from_dir`, `unregister_skills_by_source`. Result: one framework definition site each (`echo-core/src/plugin/*` + facade `echo_agent::plugin`). `echo-agent-cli/.../skills_hub/` is a single-skill installer with its own `validate_subdir`, NOT a parallel plugin registry. The `NativePlugin` trait (facade) and `export_to_env` (variables.rs) have zero callers — see F-PLG-01-P3-01. |
| Migration deletion | No migration proposed in this task. F-PLG-01-P3-01 identifies `NativePlugin` + `export_to_env` as deletion candidates (dead public API with zero callers in either repo). |

## Current Path

Verified plugin framework data flow at commit `9b0e0fa` / `b3b2e81`:

1. **Base directory.** `plugin_data_base_dir()` (`mod.rs:102`) resolves a
   single `OnceLock<PathBuf>` (default `~/.echo-agent`). Applications call
   `set_plugin_data_base_dir_name(".eko")` at startup; the framework default
   stays neutral for other consumers. `home_dir()` (`mod.rs:90`) reads `$HOME`
   with a `~` fallback (never panics). Project/Local scopes stay
   `.echo-agent/plugins` regardless of the user-scope override (team-shared
   VCS convention — `scope.rs:36-37`).

2. **Scan.** `reload()` (`plugin_runtime.rs:201`) builds a fresh candidate
   `PluginRegistry`, `scan_registry` → `scan_all` → `scan_scopes`
   (`registry.rs:126`) clears `plugins`, iterates `[User, Project, Local]`,
   and `scan_scope_dir` (`registry.rs:184`) reads each child dir's
   `.echo-plugin/manifest.yaml`, parses via `PluginManifest::from_file`,
   validates, and inserts keyed by manifest `name`. `load_state`
   (`registry.rs:933`) merges persisted enabled/disabled + validated config
   from `registry.json` (invalid persisted config → disabled + warn).

3. **Dependency resolution.** `apply_candidate` (`plugin_runtime.rs:551`)
   calls `candidate.resolve_enabled_dependencies()` (`registry.rs:803`).
   `resolve_dependencies_matching` (`registry.rs:807`) builds an adjacency
   graph for enabled plugins, enforces each dependency exists + is enabled +
   satisfies the semver constraint (`dep.satisfies(&dep_entry.manifest.version)`,
   `registry.rs:841`), runs Kahn's algorithm with a sorted queue
   (`registry.rs:862`), and detects cycles (`sorted.len() != in_degree.len()`).

4. **Component resolution + wiring.** For each enabled plugin in topo order,
   `wire_all` (`plugin.rs:128`) resolves variables (`registry.variables_for`)
   + `ensure_data_dir`, resolves component paths
   (`registry.resolve_components`, `registry.rs:636`), and collects
   skill_dirs/hooks_defs/mcp_files/agent_files. Then it wires:
   - Skills: `load_plugin_skills_from_dir(dir, "plugin:{id}", variables)`
     (`plugin.rs:264`) → `discover_skills_inner` tags both the catalog and
     progressive skill registries with the source.
   - Hooks: `hook_reg.register_plugin_hooks(name, dir, def)`
     (`plugin.rs:294`) → stored under `HookSource::Plugin(name)`.
   - MCP: `agent.load_mcp_config(config)` (`plugin.rs:332`) → connected
     servers recorded; missing servers mark the plugin failed.
   - Subagents/LSP/monitors/themes/output-styles: reported as
     application-owned discovery outputs.

5. **Atomic swap.** `apply_candidate` deactivates all lifecycle callbacks,
   replaces monitors (with rollback), swaps the registry, calls
   `replace_agent_components` (unload previous → wire candidate; re-wire
   previous on failure), swaps the LSP manager, then activates the candidate
   lifecycle callbacks. On activation failure, a full reverse swap restores
   the previous state.

6. **Unload.** `unload_agent_components` (`plugin_runtime.rs:1157`) removes
   Subagents (`unregister_subagent`), skills
   (`unregister_skills_by_source("plugin:{name}")` — purges catalog +
   progressive + hooks + context projection + refreshes progressive tools),
   hooks (`hook_registry.unregister(&HookSource::Plugin(name))`), and MCP
   servers (`disconnect_mcp` → removes adapted tools).

7. **Lifecycle.** `PluginLifecycleManager` (`lifecycle.rs:65`) tracks
   `initialized` + `active` per plugin. `activate` calls `init` once then
   `activate` per transition. `unregister` (`lifecycle.rs:153`) removes the
   entry before running deactivate/shutdown, so a failed cleanup cannot block
   re-registration. `Drop` calls `shutdown` on all remaining entries.

8. **Persistence.** `save_state` (`registry.rs:895`) serializes to
   `registry.json.tmp`, sets 0o600 on Unix, then `rename` (atomic).
   `configure`/`enable`/`disable`/`install` all persist after mutation and
   roll back the in-memory change if persistence fails.

## Findings

### F-PLG-01-P2-01: Plugin name collisions during scan are silently overwritten with no warning and non-deterministic resolution

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/plugin/registry.rs:235` —
    `self.plugins.insert(id, entry);` is a plain `HashMap::insert` with no
    check for an existing key and no `warn!`/`info!` log.
  - `echo-agent/echo-core/src/plugin/registry.rs:190` — the scan iterates
    `std::fs::read_dir(dir)?` whose order is filesystem-dependent (same class
    as F-SKL-01-P2-01's `tokio::fs::read_dir` non-determinism).
  - `echo-agent/echo-core/src/plugin/registry.rs:130-134` — `scan_scopes`
    iterates `[User, Project, Local]` in order, so a later scope silently
    overwrites an earlier scope's same-named plugin (Local > Project > User).
- Reachability: every `reload()` / `scan_all()` call exercises this path.
  Live callers: `PluginRuntimeService::reload/enable/disable/install/configure`
  all call `scan_registry` → `scan_all` → `scan_scopes` → `scan_scope_dir`.
  The collision window opens whenever (a) two directories within one scope
  declare the same manifest `name:` (e.g. a stale dir left by a reinstall, or
  a manual copy), or (b) two scopes (e.g. User + Project) each install the
  same plugin name.
- Expected invariant: the task question asks whether the framework resolves
  dependencies and component ownership without leaked registrations. Name
  uniqueness across the registry is a precondition for both — if two plugin
  dirs resolve to the same `PluginId`, the loser's components are never
  loaded and the winner is non-deterministic (within-scope) or
  last-scope-wins (cross-scope), with no signal to the user.
- Observed behavior:
  - **Within-scope**: two dirs `plugins/foo-a/` and `plugins/foo-b/` both
    with `name: my-tool` → whichever `read_dir` returns last overwrites the
    other silently. Cross-filesystem runs (ext4 vs APFS) can pick different
    winners.
  - **Cross-scope**: User `my-tool` + Project `my-tool` → Project silently
    overwrites User (scanned second). Local would overwrite Project. This is
    last-scanned-wins, which is the *opposite* precedence from the skill
    loader (F-SKL-01: first-scope-wins, with a `warn!("shadowed by existing")`
    log at `loader.rs:147`). The skill loader and the plugin registry
    therefore apply contradictory collision conventions for the same
    conceptual operation.
  - No `warn!` or `info!` is emitted in either case, unlike the skill loader.
- Impact: latent correctness defect. The user installs a plugin, sees it
  absent from the runtime (because a same-named dir shadowed it), and has no
  log to diagnose why. The non-determinism makes it reproducible only on the
  same filesystem. No data corruption (the loser's files remain on disk,
  just not loaded) and no security impact (both candidates are user-installed
  under trusted scopes). The cross-scope inconsistency with skills is a
  maintainability hazard: a contributor fixing collision handling in one
  system will not know the other diverges.
- Root cause: `scan_scope_dir` was written without a collision check or
  warning. The cross-scope last-wins behavior is likely unintentional
  (no documentation states Local should override User), and the within-scope
  non-determinism inherits from `read_dir` without canonicalization.
- Direction: in `scan_scope_dir`, before `self.plugins.insert(id, entry)`,
  check `if self.plugins.contains_key(&id)` and emit a `warn!` naming both
  paths. Decide and document the intended precedence (recommend matching the
  skill loader's first-wins for consistency, OR make same-name across scopes
  a hard error). For within-scope determinism, sort `read_dir` entries by
  path before processing (same fix as F-SKL-01-P2-01). Add a regression test
  creating two same-named plugin dirs and asserting the winner + the warning.
  The fix belongs in `echo-core/src/plugin/registry.rs`, not the application.
- Regression validation: new test in `echo_core::plugin::registry::tests`
  creating same-named plugins in two scopes and within one scope; assert the
  winner is deterministic and a warning is logged. Run
  `cargo test -p echo_core plugin::registry`.
- Validation reports: [V01](../validations/F-PLG-01/V01-01.md)

### F-PLG-01-P3-01: `NativePlugin` trait and `export_to_env` are dead public API with zero callers in either repository

- Priority: P3
- Confidence: high
- Layer: framework (facade + echo-core)
- Evidence:
  - `echo-agent/src/plugin.rs:460` — `pub trait NativePlugin: Send + Sync`
    with default `init`/`shutdown`. A repository-wide grep for
    `dyn NativePlugin`, `Box<dyn NativePlugin>`, `impl NativePlugin`, and
    `NativePlugin` (excluding the definition) returns zero hits. The trait
    is never used, never implemented, never held.
  - `echo-agent/echo-core/src/plugin/variables.rs:186` —
    `pub fn export_to_env(vars: &PluginVariables)`. A repository-wide grep
    for `export_to_env` returns only the definition itself — zero callers.
    The function uses `unsafe { std::env::set_var(...) }` and documents a
    single-threaded-init precondition, but nothing invokes it; plugin
    variables flow through `PluginVariables::substitute` into component
    files instead.
- Reachability: both symbols are `pub` and compile into the public API
  surface, but no code path — framework, application, or test — constructs
  or calls them. The `PluginLifecycle` trait (the live native-callback
  mechanism) is exercised through `PluginLifecycleManager` in
  `plugin_runtime.rs`; `NativePlugin` is a separate, older trait that was
  never wired in.
- Expected invariant: AGENTS.md's dead-code rule ("删除优先于保留") and the
  layering gate ("先搜完整仓库再新增") require that public API with zero
  consumers and no documented retention rationale be deleted rather than
  retained "for the future". A `pub` symbol with an `unsafe` block
  (`export_to_env`) that is never called is an especially clear deletion
  candidate — it cannot be tested in production and its safety precondition
  is unverifiable.
- Observed behavior: both symbols compile, appear in the public API, and are
  never reached. `NativePlugin`'s doc says "Prefer file-based plugins" and
  "retained for backward compatibility", but there is no backward
  compatibility to preserve (the project explicitly drops compatibility per
  AGENTS.md) and no consumer to be compatible with.
- Impact: low. No runtime effect. The cost is API-surface noise (a
  contributor reading `plugin.rs` may implement `NativePlugin` thinking it
  is the integration path, when `PluginLifecycle` is), and an untested
  `unsafe` function shipping in the public API.
- Root cause: `NativePlugin` predates the file-based plugin + lifecycle
  system and was superseded by `PluginLifecycle` + `PluginLifecycleManager`
  without being removed. `export_to_env` was an early variable-export idea
  superseded by in-process `PluginVariables::substitute`.
- Direction: delete `NativePlugin` from `echo-agent/src/plugin.rs:460-478`
  and its re-export if any. Delete `export_to_env` from
  `echo-core/src/plugin/variables.rs:186-212`. Confirm no external consumer
  relies on them (none exist in-repo). If retention is desired, add a
  `#[doc(hidden)]` + a code comment stating why a framework consumer would
  need it, and gate `export_to_env`'s `unsafe` behind a soundness audit.
- Regression validation: `cargo check --workspace --all-features` after
  deletion; `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Validation reports: [V01](../validations/F-PLG-01/V01-01.md),
  [V02](../validations/F-PLG-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Manifest/path/dependency graph: single definition site, semver enforcement, cycle detection, no duplicate framework. | yes | passed | [V01-01](../validations/F-PLG-01/V01-01.md) |
| V02 | Source-scoped registration: skills/hooks/MCP/Subagents tagged per-plugin, reversible, distinct from skill/user-config sources. | yes | passed | [V02-01](../validations/F-PLG-01/V02-01.md) |
| V03 | Activation failure rollback: four failure checkpoints restore previous state; lifecycle remove-before-cleanup. | yes | passed | [V03-01](../validations/F-PLG-01/V03-01.md) |
| V04 | Reload/unload lifecycle: full load→disable→enable→uninstall cycle removes all components; lifecycle counts match; preferences persist. | yes | passed | [V04-01](../validations/F-PLG-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | No historical document is cited as evidence for any claim in this report. |

All 41 `echo_core::plugin` tests and all 8 `plugin_runtime` tests pass at the
audited commits (see V01-01 and V04-01 for commands and results).

## Historical Claim Status

No historical documents are cited as evidence for any claim in this report.
All findings are based on code at commit `9b0e0fa` / `b3b2e81` and the four
validation reports.

## Coverage And Uncertainty

- Code not inspected:
  - `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs` (609 lines)
    — the application-owned Subagent/theme/output-style/monitor/LSP
    preparation. Read at the signature level (`prepare_application_components`,
    `register_plugin_agents`, `validate_application_component_files`,
    `PreparedApplicationComponents`); the component parsing internals are
    A-PLG-01's scope. The rollback contract (this task) only requires that
    `PreparedApplicationComponents` be swappable, which V03/V04 confirm.
  - `echo-agent-cli/src/tauri/commands/plugins.rs` and
    `echo-agent-cli/src/cli/cmd_impls/plugins.rs` — the command surfaces that
    call `PluginRuntimeService`. Read only to confirm they delegate to the
    service and do not maintain a parallel registry.
  - `echo-agent-cli/echo-agent-app-core/src/skills_hub/install.rs` — the
    single-skill git installer. Confirmed it has its own `validate_subdir`
    (line 569) and is NOT a parallel plugin registry.
- Unreachable-but-missing-validation observation (not promoted to a finding
  because no caller can trigger it today):
  `PluginRegistry::install_git` (`registry.rs:354`) accepts an `Option<&str>`
  `subdir` and does `tmp_dir.join(sub)` without validating that `sub` is
  relative and contained. An absolute `subdir` (e.g. `/etc`) would replace
  the clone path; a traversal `../../etc` would escape it. `InstallSource::parse`
  (`scope.rs:89`) always sets `subdir: None`, and all three command callers
  (tauri/tui/cli) use `parse`, so user input cannot reach `subdir` today. The
  asymmetry with SkillsHub's `validate_subdir` is worth a one-line
  containment check if a programmatic caller is ever added, but there is no
  concrete negative impact at the audited commits.
- Environmental limits: none. Both repos are clean at the audited commits.
  Tests ran on darwin 25.5.0 arm64.
- Claims that remain uncertain:
  - The cross-scope precedence direction (last-scanned-wins for plugins vs
    first-scope-wins for skills) may be intentional rather than a bug. The
    finding F-PLG-01-P2-01 frames the *silence + non-determinism* as the
    defect, not the precedence direction itself; the direction is flagged
    for the maintainer to decide and document.

## Handoff

- Conclusions downstream tasks may rely on:
  - There is exactly one plugin framework (`echo-core/src/plugin` + facade
    `PluginIntegrator`). No parallel registry exists in the application;
    `PluginRuntimeService` is the sole orchestrator. A-PLG-01 and X-PLG-01
    can rely on this.
  - The `PluginIntegrator::wire_all` + `unload_agent_components` pair is the
    single component wiring/unwinding authority. All four live component
    categories (skills/hooks/MCP/Subagents) are source-tagged and reversibly
    removable. F-INT-01 (MCP) and F-INT-02 (LSP) can rely on the
    connect/disconnect and replace+shutdown_all contracts respectively.
  - Activation/reload failure rollback is comprehensive (four checkpoints,
    tested). No leaked registrations on the failure path. F-RCT-05 (snapshot/
    resume) can rely on the plugin runtime being atomically swappable.
  - The `PluginLifecycleManager` remove-before-cleanup ordering guarantees
    failed cleanup cannot block re-registration. F-HITL-01 and any native
    callback consumer can rely on this.
  - `NativePlugin` and `export_to_env` are confirmed dead (zero callers);
    downstream tasks should not assume they are the integration path.
    `PluginLifecycle` + `PluginLifecycleManager` is the live mechanism.
- Reports they must read:
  - [V01-01](../validations/F-PLG-01/V01-01.md) for manifest parsing,
    dependency resolution, and the single-definition-site confirmation.
  - [V02-01](../validations/F-PLG-01/V02-01.md) for source-scoped registration
    and the unload manifest.
  - [V03-01](../validations/F-PLG-01/V03-01.md) for the four rollback
    checkpoints and lifecycle remove-before-cleanup.
  - [V04-01](../validations/F-PLG-01/V04-01.md) for the full load/disable/
    enable/uninstall cycle and lifecycle callback counts.
- Conditions that make this report stale:
  - Any change to `scan_scope_dir`'s insert path invalidates
    F-PLG-01-P2-01 and V01.
  - Any change to `apply_candidate`'s rollback choreography or
    `replace_agent_components` invalidates V03.
  - Any change to `unload_agent_components` or the source-tagging in
    `discover_skills_inner` invalidates V02 and V04.
  - Introduction of a caller for `NativePlugin` or `export_to_env`
  invalidates F-PLG-01-P3-01.
- Follow-up task IDs (no fixes implemented in this review):
  - A-PLG-01 should verify the EKO UI (Tauri/TUI) plugin panels delegate to
    `PluginRuntimeService` and do not maintain a divergent enabled/loaded set,
    and should audit `plugin_components.rs` (application-owned Subagent/theme
    preparation) for atomic registration.
  - Q-FLT-01 should run a fault-injection fixture for the within-scope
    name-collision non-determinism (F-PLG-01-P2-01) — create two same-named
    plugin dirs and assert the winner is stable across repeated scans.
  - F-INT-01 owns the MCP transport boundary that this task touched only at
    the connect/disconnect contract.
