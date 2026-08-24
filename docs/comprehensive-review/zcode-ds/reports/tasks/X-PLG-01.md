# X-PLG-01: Skill/plugin/hook lifecycle conformance

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories; `target/` written by the test run only)

## Question

Are framework lifecycle primitives and EKO activation policy joined through
reversible, source-scoped, failure-safe adapters?

## Scope

Seam verification across both repositories (cross-repository contract task; the
dependency reports did the deep subsystem reads, this task re-anchors and
re-traces the seam):

- Framework primitives (re-verified anchors): `echo-agent/src/plugin.rs`
  (`PluginIntegrator::wire_all` :128-395, `wire_skills`/`wire_hooks`/`wire_mcp`
  :398-444, `components_by_plugin`), `echo-agent/src/agent/react/capabilities.rs`
  (skill discovery/tagging :635-731, `unregister_skills_by_source` :915-958,
  `unregister_subagent` :406-424, `disconnect_mcp` :1315-1332, `load_mcp_config`
  :1251-1272), `echo-execution/src/skills/registry.rs` (:95-232, :430-510),
  `echo-execution/src/skills/hooks.rs` (:538-655), `echo-core/src/hooks/types.rs`
  (`HookSource` :1150-1166, `PluginLoaded`/`PluginDisabled` :190-192),
  `echo-core/src/plugin/lifecycle.rs` (:75-210), `echo-agent/src/agent/subagent/
  registry.rs` (:275-293), `echo-integration/src/mcp/mod.rs` (:56-111).
- EKO activation policy (re-verified): `echo-agent-cli/echo-agent-app-core/src/
  plugin_runtime.rs` (boot :139-178, reload/enable/disable/install/uninstall
  :201-340, `apply_candidate` :551-802, `replace_agent_components` :804-909,
  `unload_agent_components` :1157-1176, `fire_loaded_events` :1048-1071),
  `plugin_components.rs` (:444-509), `runtime.rs` (bootstrap :110, :148-281),
  `state.rs` (`switch_workspace`/`exit_workspace` :844-1185), `skills_hub/*`,
  `src/tauri/commands/panels.rs` (:447-661), `src/tauri/commands/plugins.rs`,
  `src/tauri/commands/hooks.rs`, `src/cli/cmd_impls/{plugins,skills}.rs`,
  `agent_pool.rs` (:495-521, :885-940).
- Executed check: `cargo test -p echo-agent-app-core --lib --locked plugin_runtime`
  (8 passed, exit 0; see V04-02).

## Out Of Scope

- Skill engine internals (frontmatter, activation, hook execution ordering) ->
  F-SKL-01 (canonical findings cross-referenced, not re-reviewed).
- Plugin manifest/registry internals -> F-PLG-01 (canonical findings
  cross-referenced).
- EKO hub marketplace mechanics (git install/sync internals) -> A-PLG-01.
- Workspace-switch config/watcher freeze (the config arm of the switch defect) ->
  A-CFG-01-P1-01 (cross-referenced only).
- Task/Subagent hook event translation table -> A-TSK-04.
- MCP transport lifecycle and reconnect -> F-INT-01.
- Frontend plugin/skill panels -> A-FE-*, A-SRF-03.

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (X-PLG-01 card), `zcode-ds/README.md`, report templates.
- Dependency task reports read in full: `F-SKL-01` (complete), `F-PLG-01`
  (complete), `A-PLG-01` (complete) — all in `zcode-ds/reports/tasks/`.
- Historical documents treated as hypotheses: the MASTER-PLAN and docs claims
  already classified in the dependency reports (F-PLG-01 V05-01, A-PLG-01
  V05-01); this task re-verified only the code anchors, not the documents.

## Layering Decision

- Generic mechanism (framework): `SkillRegistry` source tagging + group unload,
  `HookRegistry` + `HookSource` identities + `unregister`, `SubagentRegistry`
  remove (instance+definition+factory), `PluginLifecycleManager`,
  `PluginIntegrator::wire_all`, `McpManager` connect/disconnect, `ToolManager`
  register/unregister. Correctly placed; independently usable (demo56,
  docs/32-plugin-system.md).
- EKO product policy (application): `PluginRuntimeService` transactional
  candidate apply, `PreparedApplicationComponents`, `SkillsHub` enablement +
  `enabled-skills.json`, workspace switch state replacement, per-surface
  commands. Correctly placed.
- Adapter boundary: thin for skills/hooks/Subagents/monitors/LSP/themes/styles —
  EKO calls framework primitives, owns no second registry or state machine
  (V01-01). Two boundary defects found: MCP ownership (X-PLG-01-P2-01) and the
  early-return in `unregister_skills_by_source` (X-PLG-01-P3-01); one
  application-layer propagation gap (X-PLG-01-P3-02).
- Duplicate search (V01-01 terms): one framework authority + one EKO runtime
  owner per semantic; no parallel lifecycle found on either side. Retained
  zero-caller facade entries (`wire_skills`/`wire_hooks`/`wire_mcp`,
  `NativePlugin`, `export_to_env`) remain as archived (F-PLG-01-P3-04/P3-05).

## Current Path

Verified seam data flow (anchors in V01-01/V02-01):

- Boot: builtin skills (`runtime.rs:148-168`) -> baseline injection (:175-210) ->
  user hooks merged once (:220, `infra.rs:1918-1945`) -> user MCP
  (`runtime.rs:110`, `infra.rs:1069-1100`) -> LSP runtime (:270-281) ->
  `PluginRuntimeService::new` (:279-281) -> `reload()` -> `apply_candidate`.
- Every GUI/TUI/CLI plugin mutation builds a scanned candidate registry, then
  `apply_candidate` (:551-802): deactivate lifecycle -> replace monitors ->
  `replace_agent_components` (unload previous via `unload_agent_components` :817,
  `wire_all` :818, `register_plugin_agents` :844) -> swap LSP -> activate
  lifecycle -> `fire_loaded_events` (:790). Failure paths (a-g, V03-01) restore
  the previous set; dynamically verified (V04-02).
- `unload_agent_components` (:1157-1176) is the exact inverse keyed map:
  `unregister_subagent` per agent, `unregister_skills_by_source("plugin:{id}")`
  per plugin, `unregister(&HookSource::Plugin(id))`, `disconnect_mcp` per
  recorded server name.
- Gaps: workspace switch/exit never touches the plugin runtime (V02-01 d1);
  hub enable loads the parent directory and disable cannot unload (V02-01 d2);
  `PluginLoaded` re-fired per reload, dropped plugins get no `PluginDisabled`
  (V02-01 d3); MCP connect is destructive name-keyed replacement (X-PLG-01-P2-01);
  pool agents copy skill descriptors additively with no unload trigger
  (X-PLG-01-P3-02); `unregister_skills_by_source` early-returns on dual-registry
  divergence (X-PLG-01-P3-01).

## Findings

New findings from this task (canonical archived findings are cross-referenced
below in "Canonical Finding Cross-Reference"; all ten re-anchored with matching
current line numbers in V05-01).

### X-PLG-01-P2-01: Plugin MCP wiring is a destructive name-keyed replacement — a plugin declaring an MCP server name that collides with a user/global server silently replaces it, and plugin unload permanently disconnects it without restoring the pre-plugin connection

- Priority: P2
- Confidence: high (mechanism fully static-verified; collision scenario not
  dynamically executed)
- Layer: adapter (EKO unload policy over framework `McpManager` primitive)
- Evidence:
  - `echo-agent/echo-integration/src/mcp/mod.rs:69-87` — `McpManager::connect`
    disconnects an existing same-name client before connecting ("如果已存在同名连接，
    会先断开旧连接再建立新连接"); clients keyed by bare server name (`mod.rs:56-58`).
  - `echo-agent/src/agent/react/capabilities.rs:1251-1272` — `load_mcp_config`
    connects per server with no dedup.
  - `echo-agent/src/plugin.rs:332-347` — `wire_all` connects plugin MCP into the
    same manager; server names recorded under `components_by_plugin[plugin].mcp_servers`.
  - `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:1172-1174` —
    `unload_agent_components` disconnects by server name only.
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:1069-1100` + `runtime.rs:110`
    — user MCP servers connect into the same manager at boot; `connect_mcp_server`
    Tauri command (`src/tauri/commands/mcp.rs:211`) connects user servers at
    runtime into the same manager.
- Reachability: user has server `S` in `~/.eko/mcp.json` (or connects it via the
  GUI); installs/enables a plugin whose `.mcp.json` declares server `S`. Plugin
  load (boot or reload) disconnects the user's `S` client and replaces it with
  the plugin's; `disable`/`uninstall`/failed reload of the plugin then calls
  `disconnect_mcp("S")` and the server — with its tools — is gone with no
  restore and no error. Two plugins declaring the same server name fight the
  same way (later dependency-order wins; unloading either kills the shared
  server for both).
- Expected invariant: unload is the exact inverse of load; a plugin-scoped
  connection must not destroy a pre-existing connection, and unload must
  restore the prior owner (the pattern skills/hooks/Subagents achieve with
  source identity).
- Observed behavior: connect is last-wins destructive replacement; unload is an
  unconditional name-keyed disconnect with no ownership check and no restore
  step.
- Impact: silent loss of user MCP servers and their tools after a plugin
  enable/disable/reload cycle; the user's configuration is intact on disk but
  the live session is permanently disconnected until manual reconnect or
  restart; no error is surfaced on any surface.
- Root cause: `McpManager` has no per-source namespace or reference counting
  (unlike `SkillRegistry` source tags and `HookSource::Plugin`), and the EKO
  unload policy disconnects by name without tracking prior ownership; the
  "disconnect old, connect new" convenience in `connect` makes the seam
  destructive instead of reversible.
- Direction: scope MCP connections by owning source (per-plugin client
  namespace or reference-counted connect with restore-on-unload), or reject
  plugin-declared server names that collide with user-configured servers at
  `prepare` time with a surfaced error; in either case unload must restore or
  keep the pre-plugin connection. Delete nothing in `McpManager` yet — the fix
  is additive ownership tracking; add a per-plugin record next to
  `components_by_plugin.mcp_servers`.
- Regression validation: fixture — user server `S` connected at boot; plugin P
  declaring server `S`; `enable(P)` must not kill the user's `S` tools (or must
  surface a collision error); `disable(P)` must restore the user's `S`
  connection; two plugins sharing `S` must not disconnect each other's
  connection. Extend `real_plugin_load_disable_and_unload_are_live` (V04-02).
- Validation reports: [V01-01](../validations/X-PLG-01/V01-01.md), [V04-02](../validations/X-PLG-01/V04-02.md)

### X-PLG-01-P3-01: `unregister_skills_by_source` early-returns on the tracking registry's removal result and silently skips the shared progressive registry — the plugin unload primitive is not failure-safe across the dual-registry split

- Priority: P3
- Confidence: medium (mechanism high; no concrete current trigger found for the
  divergence)
- Layer: framework (unload primitive used by the adapter)
- Evidence: `echo-agent/src/agent/react/capabilities.rs:915-923` —
  `unregister_skills_by_source` returns immediately when
  `unregister_names_by_source` (tracking registry `by_source`) yields nothing,
  skipping `shared.unregister_by_source` (:921-923), the skill-hook
  unregistration (:925-928), and the projection/tool refresh (:930-955).
  Registration writes both registries only on the discovery path
  (capabilities.rs:719-730); `register_descriptor` (public API, registry.rs:95-124)
  re-indexes `by_source` from the descriptor's own `source` field.
- Reachability: the early return is only reachable if the two registries'
  `by_source` maps diverge — e.g. a descriptor replaced via the public
  `register_descriptor`/`register_descriptor_with_legacy` without a source tag,
  or a future path that registers into one registry only. The archived
  divergence class (F-SKL-01-P1-02) concerns activation marks, not `by_source`,
  so today's plugin path stays consistent (both registries tagged at load,
  V01-01); impact is latent.
- Expected invariant: an unload by source removes the source from every
  registry that holds its registrations, or reports that it could not; it must
  never silently report success ("removed" or empty) while shared descriptors,
  activation state, hooks, or projections remain live.
- Observed behavior: if the tracking registry lacks the source tag, the
  function returns `Vec::new()` with no log, leaving the shared registry's
  descriptors, the skill-hook registrations, and the catalog projection
  untouched — EKO believes the plugin's skills were unloaded.
- Impact: a disabled plugin's skills could keep appearing in
  `read_skill_resource`/`run_skill_script`/`activate_skill` name lists and its
  hooks keep firing after a plugin disable, with zero diagnostics. Latent
  today; becomes live the moment any writer diverges the two registries.
- Root cause: the dual-registry design (capabilities.rs:659-677) created two
  `by_source` authorities and the unload path trusts one of them as the
  predicate for the other, instead of removing by source in both and merging
  results.
- Direction: remove the early return; call `unregister_names_by_source` on the
  tracking registry AND `unregister_by_source` on the shared registry and merge
  the removed name sets before unregistering hooks/projections (idempotent —
  `remove_descriptor` on a missing name is a no-op). Fold into the
  F-SKL-01-P1-02 single-authority fix if that lands first.
- Regression validation: unit test — register a descriptor into both
  registries, drop the tag only in the tracking registry, call
  `unregister_skills_by_source` and assert the shared registry no longer holds
  the descriptor and its skill hooks are unregistered.
- Validation reports: [V01-01](../validations/X-PLG-01/V01-01.md), [V04-01](../validations/X-PLG-01/V04-01.md)

### X-PLG-01-P3-02: Pooled agents retain disabled plugins' skill descriptors — `refresh_skill_descriptors` is additive-only and plugin unload never invokes it

- Priority: P3
- Confidence: medium (static chain complete; pool agent creation order not
  dynamically exercised)
- Layer: application (pool propagation) over framework (descriptor registry)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:495-521` —
    `refresh_skill_descriptors` replaces the pool's descriptor list
    (`*self.skill_descriptors.write() = descriptors`) but then only
    `register_descriptor`s the new list into each pooled agent — it never
    removes descriptors absent from the new list.
  - Call sites only in `src/tauri/commands/panels.rs:536/:572/:624` (hub
    sync/load/enable); `plugin_runtime.rs` never calls it (grep: zero
    references), so `apply_candidate`/`unload_agent_components` does not refresh
    pool descriptors after a plugin disable/reload.
  - Pool seeding: `AgentPool::from_runtime` captures the primary agent's
    `skill_descriptors()` (agent_pool.rs:234-250), which includes plugin skills
    once the plugin is loaded; pool agents copy them (agent_pool.rs:930-935).
- Reachability: pool agents are created for background conversations
  (agent_pool.rs:259, :348); a plugin loaded before pool creation seeds its
  skills into the pool; after `disable`/`uninstall` of that plugin the pool's
  descriptor list and each pooled agent's registry still contain the plugin's
  skill names. Pooled agents share the primary's hook registry
  (agent_pool.rs:885-886), so the plugin's *hooks* do unload — only skill
  descriptors stay stale. Pool agents run no discovery and (per
  `create_agent`, infra.rs:184-224) get descriptors by copy, so the stale
  entries are inert unless a pooled agent exposes the progressive tools or an
  intent router built from its descriptors; the stale names still surface in
  `skill_registry.list_descriptors()` / `available_names()` of pool agents.
- Expected invariant: a plugin disable/unload removes its skills from every
  agent that received them (primary and pool), and `refresh_skill_descriptors`
  is a replace, not an append.
- Observed behavior: the pool keeps a copy of the removed skills' descriptors;
  no error is logged; the pool can only ever accumulate skill names, never
  drop them.
- Impact: stale skill registration in background conversation agents after a
  plugin lifecycle change — a disabled plugin's skill names remain discoverable
  in pool agents; combined with an intent router or progressive tools on pool
  agents this could route to or activate a disabled plugin's skill.
- Root cause: the pool refresh was built for the additive skill-discovery world
  (F-SKL-01-P2-01's "skip duplicate" model) and was never wired into the plugin
  unload transaction; the plugin seam unloads the primary agent only.
- Direction: after `apply_candidate` (or inside `unload_agent_components`'s
  caller), refresh pool descriptors from the post-unload primary
  (`agent.skill_descriptors()`), and make `refresh_skill_descriptors` remove
  descriptors on pooled agents that are no longer in the list (call
  `remove_descriptor` per dropped name), mirroring the primary's unload
  semantics.
- Regression validation: fixture — load plugin P with skill S, create pool
  agent, `disable(P)`, assert the pool agent's `skill_registry.list_descriptors()`
  no longer contains S and the pool's stored descriptor list is empty of S;
  re-enable P and assert S is back.
- Validation reports: [V01-01](../validations/X-PLG-01/V01-01.md), [V04-01](../validations/X-PLG-01/V04-01.md)

### Canonical Finding Cross-Reference (archived findings folded in by canonical ID)

| Canonical ID | Seam violation | Re-anchored evidence (current code) |
|---|---|---|
| F-SKL-01-P1-01 | failure-safety — cyclic `depends_on` stack-overflows the process on activation | `echo-execution/src/skills/registry.rs:468-510` (no in-progress guard), empirically reproduced (F-SKL-01 V04-05, exit 134) |
| F-SKL-01-P1-02 | reversibility/authority — resume marks only the tracking registry; tools consult the shared one | `capabilities.rs:663-677`, `react/mod.rs:1703-1704` |
| F-SKL-01-P2-01 | reversibility — re-discovery of an existing skill is a silent no-op (no replace path) | `capabilities.rs:688-695` |
| F-PLG-01-P2-01 | source-scoping — config discovery lists `.eko/plugins`, runtime scans `.echo-agent/plugins` | `config_discovery.rs:325-360` vs `echo-core/src/plugin/scope.rs:38-50` |
| F-PLG-01-P3-02 | failure-safety — dropped reload leaves an inactive lifecycle entry; `register_lifecycle` then rejects | `echo-core/src/plugin/lifecycle.rs:137-147` |
| F-PLG-01-P3-05 | reversibility — facade `wire_skills` registers without a source tag; no unload path; zero callers | `echo-agent/src/plugin.rs:398-410` |
| A-PLG-01-P1-01 | reversibility/source-scoping — workspace switch leaves the previous project's plugin hooks/monitors/Subagents live; LSP root boot-frozen | `state.rs:844-1032` (no plugin call), `plugin_runtime.rs:966`, `:986-991` |
| A-PLG-01-P2-01 | failure-safety — boot load is all-or-nothing, warn-only; one broken plugin disables the whole set silently | `plugin_runtime.rs:174-176`, `:821-828` |
| A-PLG-01-P2-02 | reversibility — hub enable loads the whole parent dir; disable cannot unload in-session; re-enable after update is a no-op | `panels.rs:606-619`, `:635-661` |
| A-PLG-01-P3-01 | reversibility — `PluginLoaded` re-fired for all candidates on every reload; dropped plugins get no `PluginDisabled` | `plugin_runtime.rs:790`, `:1048-1071`, `:256/:337` |

All ten re-anchored with matching semantics in V05-01; none contradicted.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Component ownership map (framework primitive <-> EKO activation policy, register/unregister key symmetry) | yes | passed (2 deviations -> findings) | [V01-01](../validations/X-PLG-01/V01-01.md) |
| V02 | Load/reload/unload trace across layers (boot, reload, disable, hub sync, workspace switch) | yes | passed (3 deviations -> canonical IDs) | [V02-01](../validations/X-PLG-01/V02-01.md) |
| V03 | Failure rollback (apply_candidate paths a-g, boot, activation) | yes | passed (2 deviations -> canonical IDs) | [V03-01](../validations/X-PLG-01/V03-01.md) |
| V04 | Stale tool/Subagent/hook registration search | yes | passed (6 deviations -> 2 new findings + 4 canonical) | [V04-01](../validations/X-PLG-01/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked plugin_runtime` | yes | passed (exit 0, 8 passed) | [V04-02](../validations/X-PLG-01/V04-02.md) |
| V05 | Cross-reference with canonical findings (F-SKL-01, F-PLG-01, A-PLG-01) | yes | passed | [V05-01](../validations/X-PLG-01/V05-01.md) |

All required validations executed; every command has a known exit code; no
validation failed (inspection deviations promoted to findings or canonical IDs).

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN (echo-agent-cli):375 "project_root derived from working_dir, workspace switches reflected without recreation" | partial/stale | scan half true (plugin_runtime.rs:986-991), no switch trigger + LSP root boot-fixed (:966) -> canonical A-PLG-01-P1-01; [V02-01](../validations/X-PLG-01/V02-01.md) |
| MASTER-PLAN:72/73 transactional apply + exact component unload | current | apply_candidate + unload_agent_components symmetric keys, dynamically tested (V04-02); [V03-01](../validations/X-PLG-01/V03-01.md) |
| MASTER-PLAN:51 skill sync refreshes the agent | stale (partial) | post-sync refresh no-op for installed names -> A-PLG-01-P2-02 / F-SKL-01-P2-01; [V02-01](../validations/X-PLG-01/V02-01.md) |
| F-PLG-01/A-PLG-01 archived finding set | current | all ten re-anchored without contradiction; [V05-01](../validations/X-PLG-01/V05-01.md) |

## Coverage And Uncertainty

- All behavior claims are traced code chains plus the executed plugin_runtime
  test suite; no process was launched for the new defect classes. The MCP
  collision scenario (X-PLG-01-P2-01) and the pool staleness (X-PLG-01-P3-02)
  were not executed dynamically — no fixture harness exists in a read-only
  review; the mechanism chains are complete and unambiguous.
- The F-SKL-01-P1-01 cycle crash was not re-executed (empirical evidence
  archived in F-SKL-01 V04-05 at the same commits).
- `register_lifecycle` has no production caller (F-PLG-01 V02); F-PLG-01-P3-02
  remains latent.
- Pool agent creation order relative to plugin load was not dynamically
  exercised; the seed path (agent_pool.rs:234-250) is read at pool creation,
  which surfaces create after bootstrap in the entry points inspected, so the
  plugin-skill seed is plausible but the stale-impact statement is
  conditional on pool agent progressive tools / routers.
- Whether pooled agents expose the progressive disclosure tools was not traced
  (agent construction via `create_agent` registers descriptors by copy only) —
  recorded as residual uncertainty for X-PLG-01-P3-02.
- The framework's `McpManager` connect semantics were verified against
  `echo-integration/src/mcp/mod.rs`; the F-INT-01 report's MCP lifecycle
  findings were not re-read (out of the declared dependency set).

## Handoff

- Downstream tasks may rely on: the plugin seam is reversible and source-scoped
  for skills/hooks/Subagents/monitors/LSP/themes/styles with a single runtime
  authority and a dynamically verified transactional rollback (V01/V03/V04-02);
  the seam is NOT failure-safe for MCP ownership (X-PLG-01-P2-01), dual-registry
  unload consistency (X-PLG-01-P3-01), pool descriptor freshness
  (X-PLG-01-P3-02), boot all-or-nothing (A-PLG-01-P2-01), or cyclic skill
  activation (F-SKL-01-P1-01); workspace-scoped lifecycle is missing
  (A-PLG-01-P1-01); hub enablement is add-only (A-PLG-01-P2-02).
- Reports to read: this report + V01-01..V05-01; F-SKL-01, F-PLG-01, A-PLG-01
  task reports and their validation sets (esp. F-SKL-01 V04-05, A-PLG-01
  V03-01/V04-01).
- Cross-references for the synthesizer: X-PLG-01-P2-01 is a new adapter
  boundary defect in the same family as F-PLG-01's component-ownership work and
  F-INT-01's MCP lifecycle; X-PLG-01-P3-01 should be folded into the
  F-SKL-01-P1-02 single-authority fix; X-PLG-01-P3-02 is the pool arm of the
  A-PLG-01-P2-02/F-SKL-01-P2-01 add-only discovery model.
- Stale triggers: changes to `plugin_runtime.rs`, `plugin_components.rs`,
  `state.rs` (switch/exit), `panels.rs` skill surfaces, `agent_pool.rs`
  (refresh_skill_descriptors), `capabilities.rs` (unregister_skills_by_source),
  `echo-integration/src/mcp/mod.rs` (`McpManager::connect`/`disconnect`), or
  `echo-agent/src/plugin.rs` invalidate the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): S-X-01
  (synthesis — fold X-PLG-01 findings with F-SKL-01-P1-02/F-PLG-01/A-PLG-01
  canonical set), S-RDM-01 (roadmap — MCP ownership fixture, pool refresh
  fixture, dual-registry unload fix), X-BND-01 (record MCP name-keying and
  pool descriptor copying as duplicate-authority items), Q-CLI-01/Q-STA-01
  (dynamic fixtures for P2-01/P3-01/P3-02 if scoped in).
