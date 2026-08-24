# F-PLG-01: Plugin manifest, registry, and lifecycle

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean (test logs in /tmp)

## Question

Does the plugin framework resolve dependencies, component ownership, activation,
replacement, unloading, and rollback without leaked registrations?

## Scope

- Framework (full reads): `echo-core/src/plugin/{mod,manifest,registry,scope,lifecycle,
  capability,variables}.rs`, facade `echo-agent/src/plugin.rs` (`PluginIntegrator`,
  `PluginWiringResult`, legacy `NativePlugin`), generic hook types
  `echo-core/src/hooks/types.rs` (`HookEvent` incl. `PluginLoaded`/`PluginDisabled`),
  `echo-execution/src/skills/hooks.rs` `HookRegistry` register/unregister paths
  (lines 538-700), `echo-agent/src/agent/react/capabilities.rs` skill discovery/tagging/
  unload/`disconnect_mcp`/`unregister_subagent` (lines 395-425, 560-1070, 1241-1332).
- EKO application (adapter review): `echo-agent-app-core/src/plugin_runtime.rs` (full
  1884 lines incl. tests), `plugin_components.rs` (full 609 lines), composition root
  `runtime.rs:279-280`, CLI/TUI/Tauri plugin command surfaces, `config_discovery.rs`
  plugin-manifest inventory.
- Framework example `demo56_plugin_system.rs` (compile check).
- Executed tests: `cargo test -p echo_core --lib --locked plugin` (41 passed),
  `cargo test -p echo-agent-app-core --lib --locked plugin` (10 passed),
  `cargo check --example demo56_plugin_system --locked` (exit 0).

## Out Of Scope

- Skill/hook engine internals (frontmatter, dependency probing, hook execution
  ordering) -> F-SKL-01 (cross-referenced for unload integration only).
- Checkpoint resume asymmetry of the two skill registries -> F-SKL-01-P1-02
  (same divergence class is noted for plugin unload in Coverage).
- EKO marketplace / skill install mechanics -> A-PLG-01.
- Hook event payload semantics (Task/Subagent events) -> F-SKL-01 / A-PLG-01.
- Frontend plugin panels (HooksPanel etc.) -> A-PLG-01 / A-FE-*.

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md` (F-PLG-01 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-SKL-01` (complete), `B-REF-01` (complete).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md` section 二十三.5
  (lines 935-1010), `echo-agent/README.md`, `echo-agent-cli/README.md`,
  `echo-agent/docs/en/32-plugin-system.md` — classified in the Historical Claim Status
  section.

## Layering Decision

- Generic mechanism (framework): `PluginRegistry`/manifest/scope/dependency topology,
  `PluginLifecycle`/`PluginLifecycleManager`, `PluginVariables` substitution,
  `PluginIntegrator::wire_all` assembly loop, `HookSource::Plugin` identity,
  `plugin_data_base_dir` configurability, skill source tagging and
  `unregister_skills_by_source`. All correctly placed; the framework is independently
  usable (example demo56, docs/32-plugin-system.md).
- EKO product policy (application): `PluginRuntimeService` (process-level shared
  registry + transactional reload), `PreparedApplicationComponents`/plugin agents/LSP/
  monitors/themes/output-styles assembly, `PluginPreferences` persistence, CLI/TUI/
  Tauri command surfaces, `.eko` brand dir override. Correctly placed; the reload
  transaction is application policy over framework primitives.
- Adapter boundary: thin and correct — EKO calls framework `wire_all`/unload APIs and
  implements framework traits (`PluginLifecycle`), owns no second dependency resolver,
  registry, or lifecycle state machine.
- Duplicate search terms (both repositories): `PluginRegistry`, `PluginManifest`,
  `PluginLifecycle`, `PluginIntegrator`, `PluginRuntimeService`, `PluginCapability`,
  `InstallSource`, `PluginScope`, `.echo-plugin`, `manifest.yaml`,
  `register_plugin_hooks`, `HookSource::Plugin`, `unregister_by_source`,
  `unregister_skills_by_source`, `tag_source`, `resolve_enabled_dependencies`,
  `wire_all`/`wire_skills`/`wire_hooks`/`wire_mcp`, `NativePlugin`,
  `set_plugin_data_base_dir_name`, `data_dir_for`. Results: one framework authority and
  one EKO runtime authority; no parallel lifecycle state machine; retained public facade
  options with zero in-repo callers (`wire_skills`/`wire_hooks`/`wire_mcp`,
  `NativePlugin`, `export_to_env`); `config_discovery` is an informational inventory
  (path divergence finding P2-01).

## Current Path

Verified data flow (anchors in V02-01): composition root constructs one
`PluginRuntimeService` (`plugin_runtime.rs:279-280`); CLI `/plugins` (cmd_impls/
plugins.rs), TUI events (tui/events.rs:5034-5328) and Tauri commands
(tauri/commands/plugins.rs:83) all delegate to it. Every mutation builds a fresh
candidate `PluginRegistry` (scan `registry.rs:118-139`), validates enabled-dependency
topology, prepares application components (`plugin_components.rs:96-222`), checks
Subagent name collisions, prepares LSP servers, then runs the transactional
`apply_candidate` (plugin_runtime.rs:551-802): deactivate lifecycle -> replace monitors
-> swap registry/components -> `wire_all` (facade assembly, plugin.rs:128-395) ->
register plugin Subagents -> swap LSP -> activate lifecycle. Any failure path unloads
the partial candidate (`unload_agent_components`, plugin_runtime.rs:1157-1176:
`unregister_subagent` + `unregister_skills_by_source("plugin:{id}")` +
`unregister(HookSource::Plugin)` + `disconnect_mcp`) and rewires the previous registry,
monitors, and LSP; lifecycle is reactivated for the previous set. Component ownership
is exact: skills tagged `plugin:{id}` in both registries (capabilities.rs:719-730),
hooks under `HookSource::Plugin(id)` (hooks.rs:610-645), subagents tagged
`SubagentKind::Plugin{source}`, MCP servers recorded in
`components_by_plugin` (plugin.rs:269-347). Unload keys are symmetric with
registration keys.

## Findings

### F-PLG-01-P2-01: EKO config discovery lists project-scope plugin manifests at `.eko/plugins`, a path the plugin runtime never scans (and omits the Local scope)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/config_discovery.rs:325-360`
  (Project scope scanned at `project_root.join(".eko").join("plugins")`, line 330-335;
  Local scope absent); framework authority `echo-core/src/plugin/scope.rs:38-50`
  (`Project -> <root>/.echo-agent/plugins`, `Local -> <root>/.echo-agent/plugins.local`);
  EKO's own fixture uses `.echo-agent/plugins` (`plugin_runtime.rs:1407`); user scope
  correctly aligns via `set_plugin_data_base_dir_name(".eko")` (`echo-agent-cli/src/
  main.rs:70`).
- Reachability: Tauri `discover_config` command (`src/tauri/commands/config.rs:315-340`,
  registered `src/tauri/mod.rs:99`) serializes `inventory.plugin_manifests`; the GUI
  config-discovery surface and any future consumer receive the wrong file set. No
  frontend consumer exists today (grep of web-frontend/src returned none).
- Expected invariant: every surface that lists plugin manifests resolves the same scope
  paths as the runtime authority; a listing must never point at directories the runtime
  does not scan.
- Observed behavior: project-scope plugin manifests are reported under
  `<project>/.eko/plugins` (nonexistent for runtime-installed plugins), real
  `.echo-agent/plugins` manifests are missing, and `.echo-agent/plugins.local` is never
  listed.
- Impact: misleading configuration panel/API; a user creating a plugin at
  `<project>/.eko/plugins` per the listing gets a silently never-loaded plugin — a
  capability-loss vector with no error.
- Root cause: `discover_plugin_manifests` was written against the user-scope `.eko`
  override without consulting `PluginScope::resolve_dir`, duplicating scope-path
  resolution instead of calling the authority.
- Direction: replace the hard-coded paths with `PluginScope::all()` + `resolve_dir`
  (or remove the plugin category and let the panel read `PluginRuntimeService::list`),
  deleting the hand-rolled scan; add an inventory-vs-registry parity test on one fixture
  tree.
- Regression validation: `ConfigInventory` test asserting project/local manifests
  resolve under `.echo-agent/plugins` / `.echo-agent/plugins.local`, plus a runtime
  scan parity check.
- Validation reports: [V03-01](../validations/F-PLG-01/V03-01.md), [V01-01](../validations/F-PLG-01/V01-01.md)

### F-PLG-01-P3-01: Same-name plugins across scopes silently shadow each other; uninstall of the visible copy can delete the shadowed copy's data directory

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-core/src/plugin/registry.rs:275-311` (`install_local` checks only
  `dest.exists()` in the target scope directory, so a second scope copy is allowed),
  `:235` (HashMap insert — later scan scope overwrites, effective precedence
  Local > Project > User, the reverse of the doc phrase "priority order (user → project
  → local)" at `scope.rs:53-55`), `:429-435` + `:985-997` (uninstall `keep_data=false`
  removes the name-keyed data dir, which may belong to the shadowed copy);
  `disable`/`uninstall`/`configure` all act on the visible entry only.
- Reachability: `/plugins install <dir> --scope user` then `--scope project` with the
  same manifest `name`; or two directories with one name across scopes. The shadowed
  copy is invisible in `list()` while its directory persists on disk.
- Expected invariant: plugin identity is unambiguous; duplicate names across scopes are
  rejected at install or explicitly resolved (e.g. highest-precedence only with a
  warning), and data directories belong to a specific installation.
- Observed behavior: the later-scope copy silently replaces the earlier one in the
  registry; uninstall removes only the visible copy and may delete the shadowed copy's
  data (`plugins/data/<name>`); a subsequent scan resurrects the shadowed copy.
- Impact: user-visible confusion, potential data-directory deletion of a still-installed
  plugin, and a misleading registry (`list` shows one entry, disk has two).
- Root cause: the registry keys installations by bare name with no scope dimension, and
  install-time duplicate detection is target-scope-only.
- Direction: reject `install` when the name exists in another scope (or store per-scope
  entries and make shadowing explicit), and scope the data dir by
  `name@scope`-style keying; add a two-scope same-name fixture test.
- Regression validation: unit test — install same name to User and Project scope,
  assert either an install error or an explicit shadow warning, and that `uninstall`
  never removes a data dir owned by a still-live installation.
- Validation reports: [V03-01](../validations/F-PLG-01/V03-01.md)

### F-PLG-01-P3-02: Reload that drops a plugin (invalid manifest / deleted directory) leaves an inactive `PluginLifecycleManager` registration; `register_lifecycle` then rejects re-registration

- Priority: P3
- Confidence: medium (static chain complete; no dynamic repro — no production caller of
  `register_lifecycle` exists)
- Layer: application (service flow) over framework (manager semantics)
- Evidence: `apply_candidate` calls `deactivate_all` (plugin_runtime.rs:575) which
  deactivates but never removes entries (lifecycle.rs:137-147); a candidate scan skips a
  plugin with an invalid manifest (registry.rs:202-216) so it is absent from
  `candidate_plugins` and never reactivated; the entry remains registered-inactive;
  `PluginLifecycleManager::register` rejects duplicates
  (lifecycle.rs:80-85: "Lifecycle callbacks already registered"). Self-healing: the next
  successful reload reactivates it (init skipped, `initialized=true`), and `uninstall`
  calls `state.lifecycle.unregister` (plugin_runtime.rs:333).
- Reachability: user edits a plugin's manifest to invalid YAML, then `/plugins reload`
  or Tauri `reload_plugins` (cmd_impls/plugins.rs:241, tauri/commands/plugins.rs:227);
  a native-plugin integrator calling `register_lifecycle` afterwards fails. In-repo,
  `register_lifecycle` has no production caller (V02-01), so the impact is latent.
- Expected invariant: a plugin absent from the registry must have no lifecycle entry;
  re-registration after a failed reload must succeed.
- Observed behavior: the entry survives as inactive state; re-registration errors.
- Impact: an explicit but confusing error for native-plugin integrators after a
  transient broken reload; no silent leak and no crash.
- Root cause: the manager has no "drop entries not in the next enabled set" path —
  `reconcile`/`deactivate_not_in` also keep entries; only `unregister` removes them.
- Direction: in `apply_candidate`, unregister lifecycle entries for plugins absent from
  the candidate registry (or add `PluginLifecycleManager::retain(ids)`), with a test:
  register lifecycle -> reload with invalid manifest -> register_lifecycle succeeds
  after the reload.
- Regression validation: extend `native_lifecycle_brackets_reload_configure_and_unregisters_on_uninstall`
  with an invalid-manifest reload step asserting re-registration works.
- Validation reports: [V03-02](../validations/F-PLG-01/V03-02.md)

### F-PLG-01-P3-03: The plugin data-directory path is computed by two parallel implementations

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-core/src/plugin/registry.rs:985-997` (`PluginEntry::data_dir_for`,
  char-sanitize + `base_data_dir.join`) and `echo-core/src/plugin/variables.rs:76-89`
  (`PluginVariables::data_dir_for`, identical sanitize + `plugins_child("data").join`);
  consumers: `registry.rs:431` (uninstall) and `registry.variables_for`
  (`registry.rs:591-593`, which overrides the default via `with_plugin_data`), plus any
  direct `PluginVariables::new` user.
- Reachability: both functions are public and currently consistent, so no runtime
  divergence is observed today; the duplication is the defect (AGENTS.md: 严禁平行实现
  同一语义).
- Expected invariant: one authority per computed path.
- Observed behavior: two independent sanitization/join implementations for the same
  semantic.
- Impact: future divergence risk (e.g. different sanitization rules) would silently
  split plugin data across directories; maintenance duplication.
- Root cause: `PluginVariables` predates the registry-owned data dir and kept its own
  copy.
- Direction: make `PluginVariables::data_dir_for` delegate to the registry helper (or
  hoist a shared `sanitize_plugin_name` in `plugin/mod.rs`) and delete the duplicated
  body.
- Regression validation: keep `test_data_dir_sanitization` and
  `plugin_config_persists_and_populates_substitution_variables` green with a
  same-path assertion for both call sites.
- Validation reports: [V03-01](../validations/F-PLG-01/V03-01.md)

### F-PLG-01-P3-04: `PluginVariables::export_to_env` mutates the process environment with unenforceable single-thread preconditions and has zero callers

- Priority: P3
- Confidence: high (code facts; no caller exists)
- Layer: framework
- Evidence: `echo-core/src/plugin/variables.rs:186-212` — `unsafe { std::env::set_var }`
  for `ECHO_PLUGIN_ROOT`/`ECHO_PLUGIN_DATA`/`ECHO_PROJECT_DIR` and per-config
  `ECHO_PLUGIN_OPTION_*`; doc requires callers to guarantee single-threaded init; grep
  across both repositories: zero callers.
- Reachability: public API only; not reached in-repo.
- Expected invariant: a public framework API must not carry an unenforceable
  thread-safety precondition for a whole-process mutation without a safe alternative.
- Observed behavior: none today (unused); the hazard is latent for future consumers who
  call it after runtime threads spawn.
- Impact: potential data race on `libc environ` if ever invoked concurrently; no current
  in-repo impact.
- Root cause: legacy convenience API retained without a caller.
- Direction: either delete it (YAGNI, per AGENTS.md deletion rules for unused internals
  — it is pub but framework-wide unused) or replace with a documented per-process guard
  (OnceLock + explicit opt-in); at minimum note it in X-BND-01 as an unused public API.
- Regression validation: n/a (no behavior to preserve in-repo); if kept, a doc test.
- Validation reports: [V01-01](../validations/F-PLG-01/V01-01.md)

### F-PLG-01-P3-05: `PluginIntegrator::wire_skills` registers skills without a source tag — its registrations can never be unloaded by `unregister_skills_by_source`

- Priority: P3
- Confidence: high
- Layer: framework (facade)
- Evidence: `echo-agent/src/plugin.rs:398-410` (`wire_skills` -> `agent.load_skills_from_dir`,
  which goes through `discover_skills_inner` with `plugin: None`,
  capabilities.rs:635-647 — no `tag_source`); `unregister_skills_by_source` is the only
  bulk-removal path and matches by source tag (capabilities.rs:915-958). `wire_hooks`
  (`plugin.rs:413-426`) registers under `HookSource::Plugin` and is therefore
  unloadable; `wire_mcp` (`plugin.rs:430-444`) connects by name and is disconnectable.
- Reachability: zero in-repo callers (V01-01); the API is retained per MASTER-PLAN:1009
  ("供 partial wire 场景"). Any future consumer using `wire_skills` creates
  registrations with no unload path — reload/unload contract broken for that entry
  point.
- Expected invariant: every framework registration entry point must have a symmetric
  removal entry point.
- Observed behavior: `wire_skills`-loaded skills are invisible to source-scoped unload
  (they also skip the plugin variable substitution path).
- Impact: latent — a partial-wire consumer cannot cleanly reload its skills; misleading
  facade surface.
- Root cause: `wire_skills` predates source tagging and was not retrofitted with a
  source parameter.
- Direction: add a `source` parameter to `wire_skills` (tag before/after load) or delete
  it in favor of `wire_all` + a note in docs; either way document the unload contract.
- Regression validation: unit test — `wire_skills` with a source tag, then
  `unregister_skills_by_source` removes exactly those skills.
- Validation reports: [V01-01](../validations/F-PLG-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search across both repositories | yes | passed | [V01-01](../validations/F-PLG-01/V01-01.md) |
| V02 | Registration and runtime reachability trace | yes | passed | [V02-01](../validations/F-PLG-01/V02-01.md) |
| V03 | Invariants — manifest/path/dependency graph, source-scoped registration, EKO listing parity | yes | passed (3 deviations -> findings) | [V03-01](../validations/F-PLG-01/V03-01.md) |
| V03 | Invariants — activation failure rollback, reload/unload lifecycle, stale registration | yes | passed (1 deviation -> finding) | [V03-02](../validations/F-PLG-01/V03-02.md) |
| V04 | `cargo test -p echo_core --lib --locked plugin` | yes | passed (exit 0, 41 passed) | [V04-01](../validations/F-PLG-01/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked plugin` | yes | passed (exit 0, 10 passed) | [V04-02](../validations/F-PLG-01/V04-02.md) |
| V04 | `cargo check --example demo56_plugin_system --locked` | yes | passed (exit 0) | [V04-03](../validations/F-PLG-01/V04-03.md) |
| V05 | Historical-document drift check | yes | passed | [V05-01](../validations/F-PLG-01/V05-01.md) |

All required validations executed; every command has a known exit code; the three
inspection deviations were promoted to findings (P2-01, P3-01, P3-03) and V03-02's
deviation to P3-02. No validation failed.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:960 P0-3 plugin base-dir unification, `.eko` override, Project/Local stay `.echo-agent` | current | `plugin/mod.rs:102-136`, `main.rs:70`, `scope.rs:38-50`; [V05-01](../validations/F-PLG-01/V05-01.md) |
| MASTER-PLAN:961 P0-2 `HookSource::Plugin` identity + validate-before-register + replace-on-reregister | current | `hooks.rs:610-645`, `plugin.rs:294`; [V05-01](../validations/F-PLG-01/V05-01.md) |
| MASTER-PLAN:982/1002 P0-2b thin adapter (`wire_all` as authority) | current | `plugin_runtime.rs:818-819`; [V02-01](../validations/F-PLG-01/V02-01.md) |
| MASTER-PLAN:988 P0-4 shared `PluginRuntimeService` (was: "each tauri command builds its own registry") | current (fixed) | `runtime.rs:279-280`, `tauri/commands/plugins.rs:3,83`; [V05-01](../validations/F-PLG-01/V05-01.md) |
| MASTER-PLAN:988 P0-4 "disable/uninstall 不卸载已注册组件" (known limitation) | fixed | `unload_agent_components` `plugin_runtime.rs:1157-1176` + P1-reload; [V02-01](../validations/F-PLG-01/V02-01.md) |
| MASTER-PLAN:997 P1-reload real unload (source-tagged skills) | current | `capabilities.rs:719-730, 915-958`; [V02-01](../validations/F-PLG-01/V02-01.md) |
| MASTER-PLAN:997 "MCP 暂不 disconnect (惰性化)" | superseded (improved) | `disconnect_mcp` now called `plugin_runtime.rs:1172-1174`; [V02-01](../validations/F-PLG-01/V02-01.md) |
| MASTER-PLAN:998 P1-frontend (`/plugins reload`, `list_hooks`/`reload_hooks`) | current | `cmd_impls/plugins.rs:241`, `tauri/commands/hooks.rs:54,123`; [V05-01](../validations/F-PLG-01/V05-01.md) |
| MASTER-PLAN:1008 "tauri commands each build own registry" | stale | no `build_registry` remains; shared service; [V05-01](../validations/F-PLG-01/V05-01.md) |
| MASTER-PLAN:1009 `wire_skills`/`wire_mcp` retained, non-duplicate | current (with unload gap) | `plugin.rs:398-444`, zero callers -> P3-05; [V01-01](../validations/F-PLG-01/V01-01.md) |
| `echo-agent/docs/en/32-plugin-system.md` scope table + `~/.echo-agent` defaults | current (framework default) | `scope.rs:38-50`; EKO `.eko` override is documented app policy; [V05-01](../validations/F-PLG-01/V05-01.md) |

## Coverage And Uncertainty

- The F-SKL-01-P1-02 dual-skill-registry divergence (checkpoint resume marks only the
  tracking registry) also affects plugin-unload correctness in principle: if the two
  registries ever diverge on activation state, `unregister_skills_by_source`'s early
  return when the tracking registry finds nothing (`capabilities.rs:917-919`) would
  skip the shared registry. For plugin skills both registries are tagged at load
  (`capabilities.rs:719-730`), so the current plugin path is consistent; this is left
  to the F-SKL-01-P1-02 fix.
- The EKO runtime tests use `MockLlmClient` and a fake LSP shell server; real MCP
  transport and LSP behavior are not exercised (F-INT-01/F-INT-02 scope). The partial-
  MCP-connect rollback was verified statically, not dynamically.
- `register_lifecycle` has no production caller; the P3-02 stale-registration scenario
  was not executed dynamically.
- `demo56_plugin_system` was compile-checked but not run.
- `hooks.rs` beyond the register/unregister surface (execution engine) belongs to
  F-SKL-01; no hook-execution claims are made here.

## Handoff

- Downstream tasks may rely on: one framework plugin authority + one EKO runtime
  authority with a transactional, tested reload/rollback (V03-02/V04-02); symmetric
  source-scoped registration/unregistration for all component types (V02-01); sound
  manifest/path/dependency validation (V03-01); the EKO config-inventory path
  divergence P2-01; MASTER-PLAN 二十三.5 mostly current (V05-01).
- `A-PLG-01`: consumes P2-01 (config listing parity), P3-02 (lifecycle cleanup on
  dropping reload), and the V04-02 fixture patterns for reload/unload GUI tests.
- `X-PLG-01`: component ownership map is complete and symmetric (V02-01); the P3-05
  facade unload gap and P3-04 unused unsafe API are cross-repository conformance input;
  the F-SKL-01-P1-02 divergence note above belongs to its failure-rollback fixtures.
- `X-BND-01`: record P3-03 (duplicate data-dir implementation) and the retained zero-
  caller facade entries (`wire_skills`/`wire_hooks`/`wire_mcp`, `NativePlugin`,
  `export_to_env`) as duplicate/unused-API items.
- `Q-DOC-01`: `echo-execution/src/skills/hooks.rs:52-53` documents plugin hook events
  while the authoritative enum is `echo-core/src/hooks/types.rs:190-192` (doc-location
  lag only).
- Reports to read: this report + V01-01 through V05-01; F-SKL-01 (skill unload
  integration, P1-02 divergence); B-REF-01 (converged lifecycle patterns are respected
  here: re-discovery + rebuild, source identity, terminal-status models).
- Stale triggers: changes to `echo-core/src/plugin/*`, `echo-agent/src/plugin.rs`,
  `capabilities.rs` skill tagging/unload, `echo-execution/src/skills/hooks.rs`
  register/unregister, or `echo-agent-cli` `plugin_runtime.rs` / `plugin_components.rs` /
  `config_discovery.rs` invalidate the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): A-PLG-01, X-PLG-01,
  X-BND-01, Q-DOC-01.
