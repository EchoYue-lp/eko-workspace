# A-PLG-01: Skills, plugins, hooks, and reload lifecycle

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories; only `target/` written by tests)

## Question

Does EKO discovery/activation/reload correctly apply product components while
framework registrations unload and roll back cleanly?

## Scope

- `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs` (full, 1884
  lines incl. tests), `plugin_components.rs` (full), `skills_hub/`
  (`registry.rs`, `enabled_skills.rs`, `install.rs`, `mod.rs`), `runtime.rs`
  (bootstrap :73-369, LSP runtime :499-592), `state.rs` (switch/exit
  workspace :844-1185, skills_hub field :443/:592), `infra.rs`
  (`load_user_hooks` :1918-1945), `hook_config_loader.rs` (full),
  `config_watcher.rs` (full), `tasks/task_runtime/hook_event_dispatcher.rs`
  (queue :1-130) + `store.rs` flush/shutdown (:247-265),
  `agent_pool.rs` (pool sync :425-466 cross-ref).
- `echo-agent-cli/src`: `main.rs` (path setup :66-70, surface wiring
  :267-425), `tauri/desktop.rs` (:124-271), `tauri/commands/plugins.rs`
  (full), `tauri/commands/hooks.rs` (full), `tauri/commands/panels.rs`
  (skill surfaces :428-660), `cli/cmd_impls/plugins.rs` (full),
  `cli/cmd_impls/skills.rs` (full), `cli/cmd_impls/hooks.rs` (reload path),
  `tui/events.rs` (plugin :5021-5340, hooks :3490-3527, skills :3298-3390),
  `tui/commands.rs` (SlashCommand inventory), `tui/mod.rs` (plugin_runtime
  wiring :378-379/:850/:1963).
- Framework cross-refs (anchors only): `echo-agent/src/plugin.rs` (`wire_all`
  :128-395), `src/agent/react/capabilities.rs` (plugin skill tagging
  :635-731, unload :915-958, unregister_subagent :406-424,
  disconnect_mcp :1315), `echo-execution/src/skills/hooks.rs`
  (register/unregister :538-700, `run_hooks` :792-836),
  `echo-core/src/plugin/{lifecycle,registry,scope}.rs` (lifecycle manager,
  scan, scope paths).

## Out Of Scope

- Framework plugin lifecycle internals themselves -> F-PLG-01 (EKO interplay
  cross-referenced; component ownership map and rollback verified there).
- Skill engine internals (frontmatter, activation, dual registry) ->
  F-SKL-01 (P1-01/P1-02/P2-01/P2-03 cross-referenced).
- Config discovery/precedence, provider selection, watcher scope ->
  A-CFG-01 (P1-01/P1-02/P2-01 cross-referenced).
- TaskRuntime store/executor/claims -> A-TSK-01..04 (only the
  `HookEventDispatcher` queue flush/shutdown is verified here as the task
  card's "hook queue" requirement; its translation table belongs to
  A-TSK-04).
- MCP server topology and `/mcp` command stubs -> A-INT-01.
- Frontend plugin panels -> A-FE-01/A-SRF-03.

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-PLG-01 card), `zcode-ds/README.md`, report templates.
- Dependency task reports read in full: `F-SKL-01`, `F-PLG-01`, `A-CFG-01`
  (all `complete` in this track).
- Historical documents treated as hypotheses:
  `echo-agent-cli/docs/MASTER-PLAN.md` (plugin/skill/hook rows :51,:69,:72-75,
  :237-243,:370-425), `echo-agent-cli/docs/skill-sync.md`,
  `echo-agent-cli/docs/system-deep-dive/06-skills.md` — classified in the
  Historical Claim Status section.

## Layering Decision

- Generic mechanism (framework): `PluginRegistry`/manifest/scope/dependency
  topology, `PluginLifecycleManager`, `PluginIntegrator::wire_all`,
  `SkillRegistry` + discovery/activation, `HookRegistry` +
  register/unregister + `run_hooks`, `HookSource::Plugin` identity. Correctly
  placed; independently usable (demo56, docs/32-plugin-system.md).
- EKO product policy (application): `PluginRuntimeService` (transactional
  apply, per-surface commands, theme/output-style preferences),
  `PreparedApplicationComponents` parsing (agents/LSP/monitors/themes/styles),
  `HookConfigLoader` merge + watcher reload scope, `SkillsHub` marketplace
  index + enabled-skills.json + git install/sync, workspace switch/exit
  state replacement. All correctly placed.
- Adapter boundary: thin and correct — EKO calls framework `wire_all`/unload
  APIs, implements `PluginLifecycle`, owns no second dependency resolver,
  registry, state machine, or validator. `prepare_application_components`
  reuses framework `ResolvedComponents` and `PluginVariables`; no duplicate
  parse of manifest paths (the hub's naive frontmatter parser is the known
  F-SKL-01-P2-03/P3-01 divergence, cross-referenced).
- Duplicate search (terms in V01-01): `PluginRegistry` constructors,
  `scan_all`/`scan_scopes`, `PluginIntegrator::wire_*`, `unregister_skills_by_source`,
  `register_plugin_hooks`/`HookSource::Plugin`, `clear_user_hooks`,
  `SkillsHub` vs `SkillRegistry`, `HookConfigLoader` vs `HookRegistry`,
  `HookEventDispatcher`/bridges, `set_loaded_skills`/`enable_skill`/
  `disable_skill`, `load_skills_from_dir` call sites, `.eko/plugins` vs
  `.echo-agent/plugins`. Result: one framework authority + one EKO runtime
  owner per semantic; no EKO-side parallel lifecycle found (V01-01).

## Current Path

Verified data flow (anchors in V02-01): every GUI/TUI/CLI plugin command
delegates to the single `PluginRuntimeService` (runtime.rs:279-281;
desktop.rs:195; tui/mod.rs:1963; main.rs:385/:425). Each mutation builds a
scanned candidate registry (`registry_for`+`scan_registry`,
plugin_runtime.rs:993-1014), validates enabled dependencies, prepares
application components (plugin_components.rs:96-222), checks Subagent name
collisions (:911-944), prepares LSP servers (:946-984), then runs the
transactional `apply_candidate` (:551-802): deactivate lifecycle -> replace
monitors -> swap registry/components -> `wire_all` (facade, plugin.rs:128-395)
-> register plugin Subagents (plugin_components.rs:444-476) -> swap LSP ->
activate lifecycle; every failure path unloads the partial candidate
(`unload_agent_components`, plugin_runtime.rs:1157-1176) and restores the
previous set (verified dynamically, V04-01). Plugin skills are tagged
`plugin:{id}` in BOTH skill registries (capabilities.rs:719-730), hooks under
`HookSource::Plugin(id)`, Subagents tagged `SubagentKind::Plugin`, MCP
recorded in `components_by_plugin`; unload keys symmetric with registration
keys. User hooks load once per boot through `HookConfigLoader` (infra.rs:1918)
and reload through the watcher (config_watcher.rs:227-278) and
CLI/TUI/GUI `/hooks reload` (all through the same loader). Task/Subagent hook
events flow through the bounded `HookEventDispatcher` queue with explicit
flush/idempotent shutdown (main.rs:330/:395). Skills hub is an index over
`~/.eko/skills/`; enablement persists `enabled-skills.json` and reaches the
runtime only via `load_skills_from_dir` (additive, no unload path).

## Findings

### A-PLG-01-P1-01: Workspace switch leaves project-scope plugin components live and LSP root boot-frozen — plugins from the previous workspace keep firing hooks, monitors, and Subagents in the new workspace

- Priority: P1
- Confidence: medium (complete static chain; no dynamic switch fixture exists)
- Layer: application
- Evidence:
  - `state.rs:844-1032` (`switch_workspace`) mutates CWD (:854), stores,
    memory, skills, routing — never calls `plugin_runtime`; `exit_workspace`
    (state.rs:1053-1185) likewise (V02-01 grep: zero plugin/hook/watcher
    references in either body).
  - `plugin_runtime.rs:986-991` — `project_root()` reads
    `agent.working_dir()` at call time (used only for the candidate scan);
    `apply_candidate` swaps the base registry with
    `self.registry_for(self.lsp.project_root.clone())` (:608) and
    `prepare_lsp` calls `manager.set_project_root(&self.lsp.project_root)`
    (:966) — the LSP root is the boot-fixed value captured in
    `register_lsp_tools` (runtime.rs:504-508, :591).
  - MASTER-PLAN.md:375 documents "project_root is derived from the agent's
    working_dir so workspace switches are reflected without recreating the
    service" — only the scan half is true; no switch-triggered reload exists.
- Reachability: every GUI workspace switch (LeftSidebar -> `switch_workspace`
  IPC -> `AppState::switch_workspace`); plugin components from the previous
  workspace's `.echo-agent/plugins` project scope stay wired on the primary
  agent and pool; they are dropped only on the next explicit plugin mutation
  (`reload`/`enable`/`disable`/`install`/`configure`), whose scan uses the new
  working dir.
- Expected invariant: workspace-scoped components follow the active
  workspace — leaving workspace A releases A's project plugins (hooks stop
  firing, monitors stop, Subagents unregistered, LSP project root rebased);
  switching to B activates B's project plugins.
- Observed behavior: after switching A -> B, A's project plugin hooks keep
  firing on B's turns (with process CWD = B), A's plugin monitors keep
  running, A's plugin Subagents remain callable, and the plugin LSP manager
  keeps serving the boot project root; B's project plugins are absent until
  an explicit plugin command runs.
- Impact: cross-workspace automation and behavior leak — the workspace
  isolation the switch implements for conversations/memory/persistence is
  broken for plugin components; hook actions and monitors from the wrong
  project can run with side effects in the new project's directory; the
  documented workspace-rebinding design (MASTER-PLAN:375) is only half
  implemented.
- Root cause: the plugin runtime was built as a process-global service with
  per-mutation scans, and workspace switching was later added as store
  replacement without a plugin unload/reload step; the LSP project root was
  captured once at bootstrap instead of being derived per apply.
- Direction: in `switch_workspace`/`exit_workspace` (after the CWD/working_dir
  change), invoke `plugin_runtime.reload()` (its candidate scan already uses
  `agent.working_dir()`, so one call both unloads A's project plugins and
  loads B's); derive the LSP project root in `prepare_lsp` from
  `self.project_root()` instead of the fixed `self.lsp.project_root` (add a
  setter or rebase in `apply_candidate`); add a switch fixture asserting A's
  project plugin hooks/skills/Subagents are unregistered after switching and
  B's load.
- Regression validation: fixture — plugin P in workspace A's project scope
  with a hook, monitor, and Subagent; `switch_workspace(B)`; assert P's hook
  no longer matches, `list_tasks` on the scheduler no longer contains P's
  monitor, `subagent_registry` lacks P's Subagent, and `running_servers`
  rebased to B; `switch_workspace(A)` restores them.
- Validation reports: [V02-01](../validations/A-PLG-01/V02-01.md), [V03-01](../validations/A-PLG-01/V03-01.md), [V05-01](../validations/A-PLG-01/V05-01.md)

### A-PLG-01-P2-01: Boot-time plugin load is all-or-nothing with a warn-only failure — one broken enabled plugin silently disables the entire plugin set at startup, and a single unusable component (e.g. missing MCP server) blocks every runtime mutation for the whole set

- Priority: P2
- Confidence: high (code facts; dynamic mutation rollback tested, boot path static)
- Layer: application
- Evidence:
  - `new_with_source` (plugin_runtime.rs:174-176): boot `reload()` error is
    `tracing::warn!` only; the service continues with the empty initial state
    (empty registry/framework/prepared) — all plugins absent with no error
    surfaced to any surface.
  - `replace_agent_components` (plugin_runtime.rs:821-828): ANY non-empty
    `wiring.errors` (per-plugin failures collected by `wire_all`,
    plugin.rs:140-395) fails the whole candidate; `prepare_application_components`
    (plugin_components.rs:217-221) likewise aborts the whole candidate on any
    component error; there is no per-plugin skip/disable-and-continue path in
    EKO (the framework collects per-plugin errors but EKO treats the candidate
    atomically).
- Reachability: any enabled plugin whose manifest/config fails to parse or
  whose MCP server cannot connect — at every boot (all plugins silently off),
  and on every mutation while the broken plugin remains enabled (mutations
  fail and roll back, blocking installs/enables of other plugins).
- Expected invariant: a broken plugin is isolated (reported, disabled, or
  skipped) without affecting unrelated plugins — the industry-consensus model
  (VS Code per-extension isolation) — and boot failures are surfaced.
- Observed behavior: at boot one malformed component = the whole plugin
  ecosystem silently absent (single warn log line); at runtime the whole
  candidate rolls back, so a single broken component repeatedly blocks all
  other plugin mutations until the user edits the offending plugin.
- Impact: silent capability loss at startup (the headline plugin feature
  appears as "no plugins installed" with no error), and an unblockable
  install/enable path for the rest of the plugin set; the warning message
  "previous runtime kept" is misleading at boot (there is no previous
  runtime).
- Root cause: the transactional all-or-nothing apply was designed for
  in-session mutation rollback (correct and tested), but was reused for boot
  without a rollback target, and no per-plugin failure isolation was added on
  top of the framework's per-plugin error collection.
- Direction: at boot, degrade per-plugin (skip/report broken plugins and load
  the rest — `wire_all` already reports per-plugin failures; EKO can convert
  per-plugin errors into a plugin-level disabled state with a surfaced
  warning in `list()`/UI), or at minimum persist a "failed plugins" report
  the surfaces can show; keep the atomic mutation rollback for in-session
  reloads. Add a boot fixture: one valid + one broken plugin -> valid loads,
  broken is reported.
- Regression validation: fixture — two plugins, one with an invalid
  `monitors.yaml`; `new_for_test` boot must load the valid one and report the
  broken one; keep `failed_real_reload_restores_previous_live_components`
  green (atomic rollback semantics unchanged).
- Validation reports: [V03-01](../validations/A-PLG-01/V03-01.md), [V04-01](../validations/A-PLG-01/V04-01.md)

### A-PLG-01-P2-02: Skills-hub enablement is not a reversible runtime lifecycle authority — GUI `enable_skill` activates the whole parent directory (all siblings), `disable_skill` cannot unload in-session, and re-enable after a content update is a silent no-op

- Priority: P2
- Confidence: high (code facts; F-SKL-01-P2-01 cross-referenced)
- Layer: application
- Evidence:
  - `panels.rs:606-619` — `enable_skill` computes
    `load_root = skill_path.parent()` and calls
    `agent.load_skills_from_dir(load_root)`: for the flat hub layout the
    parent is the whole `~/.eko/skills/` root, so enabling one skill
    registers every sibling; `disable_skill` (panels.rs:635-660) only
    persists `enabled=false` and returns `requires_restart` ("当前运行中的
    agent 已发现的技能不能热卸载") — the runtime has no unload path for
    non-plugin skills.
  - `skills.rs:61-71` and `panels.rs:525-534` — post-sync runtime refresh
    calls `load_skills_from_dir(hub root)`, which for already-installed names
    is a silent no-op (framework skip logic, F-SKL-01-P2-01,
    capabilities.rs:687-695), so updated content never reaches the runtime.
  - Bootstrap only consumes `enabled-skills.json` for baseline injection
    (runtime.rs:175-210), not for loading the hub.
- Reachability: every GUI SkillsPanel enable/disable toggle and every
  CLI/TUI/GUI `/skills sync` (CLI skills.rs:27-79, GUI panels.rs:514-538, TUI
  tui/events.rs:3345-3390).
- Expected invariant: per-skill enable state is a reversible lifecycle
  authority — enable activates exactly that skill, disable unloads it
  in-session (or explicitly marks restart-required with the runtime state
  updated), and re-sync reflects current content.
- Observed behavior: enabling one skill silently activates all siblings in
  its directory; disabling leaves the skill live until restart; a sync that
  updates an installed skill's content reports success while the runtime
  keeps serving stale instructions/hooks; GUI and runtime "loaded" views can
  diverge silently.
- Impact: over-activation (skills the user never enabled become callable),
  stale-skill execution after an explicit sync, and an enable/disable cycle
  that cannot correct itself in-session — the hub's enablement model and the
  framework's add-only discovery are not joined into one lifecycle.
- Root cause: hub enablement was implemented as "persist flag + add-only dir
  load" on top of a discovery layer with no content-compare and no
  source-scoped unload for non-plugin skills (only plugin skills are
  source-tagged, F-SKL-01).
- Direction: make `enable_skill` load exactly the skill directory
  (`skill_path`, not the parent) or add a source tag + `unregister_by_source`
  for hub skills; implement in-session disable via the same source-tagged
  unload (mirror `unregister_skills_by_source`); fix the reload no-op
  (content-hash replace, F-SKL-01-P2-01 direction); keep `enabled-skills.json`
  as the durable enable state.
- Regression validation: fixture — hub with two sibling skills; GUI-enable A,
  assert only A is registered and B's `activate_skill` fails; disable A,
  assert A's resource tool rejects; sync with changed SKILL.md, assert the
  runtime serves the new description/instructions.
- Validation reports: [V03-02](../validations/A-PLG-01/V03-02.md), [V02-01](../validations/A-PLG-01/V02-01.md)

### A-PLG-01-P3-01: Every successful plugin reload re-fires `PluginLoaded` for all candidate plugins, and a plugin dropped by reload never receives `PluginDisabled`

- Priority: P3
- Confidence: high (code facts)
- Layer: application
- Evidence: `fire_loaded_events(&candidate_plugins)` on every successful
  `apply_candidate` (plugin_runtime.rs:790, impl :1048-1071 fires for all
  names); `fire_plugin_disabled` is called only from `disable()` (:256) and
  `uninstall()` (:337) — a reload that drops a plugin (directory removed,
  invalid manifest) unloads its components (V03-01) but fires no
  `PluginDisabled` lifecycle hook.
- Reachability: every `/plugins reload` (CLI cmd_impls/plugins.rs:241, GUI
  tauri/commands/plugins.rs:227, TUI tui/events.rs:5184) while any plugin is
  enabled; user hooks listening on `PluginLoaded`/`PluginDisabled`
  (`echo-core/src/hooks/types.rs` lifecycle events) receive duplicate
  `PluginLoaded` notifications on each reload and no drop notification.
- Expected invariant: lifecycle hook notifications are exact — `PluginLoaded`
  only for newly loaded plugins, `PluginDisabled` for every plugin removed by
  a reload.
- Observed behavior: unchanged plugins are re-notified `PluginLoaded` on
  every reload (user hooks re-execute side effects), and silently dropped
  plugins get no notification at all.
- Impact: incorrect event semantics for hook consumers; duplicated
  side effects on routine reloads; no hook-based cleanup path for dropped
  plugins.
- Root cause: `fire_loaded_events` was written as "notify the candidate set"
  and no "diff vs previous set" was computed; drop notifications were wired
  only into the explicit disable/uninstall commands.
- Direction: compute the diff between `previous_plugins` and
  `candidate_plugins` in `apply_candidate`: fire `PluginLoaded` for new ids
  and `PluginDisabled` for dropped ids; add a reload fixture with a removed
  plugin asserting both events fire exactly once.
- Regression validation: fixture — reload twice without changes: exactly one
  `PluginLoaded` per plugin total; delete a plugin dir + reload: its
  `PluginDisabled` fires once and no `PluginLoaded`.
- Validation reports: [V03-02](../validations/A-PLG-01/V03-02.md)

### A-PLG-01-P3-02: EKO skill docs still use the pre-EKO root `~/.echo-agent` for the hub and enabled-skills.json, and the sync doc overstates the live runtime refresh

- Priority: P3
- Confidence: high
- Layer: application (docs)
- Evidence: `echo-agent-cli/docs/skill-sync.md` ("用户安装技能
  `~/.echo-agent/skills/`", "`~/.echo-agent/enabled-skills.json`", "同步完成
  后 CLI、TUI、GUI 和 channel 都会刷新当前 Agent 的技能目录");
  `docs/system-deep-dive/06-skills.md:263,:272` (hub root
  `~/.echo-agent/skills/`); code: `main.rs:66`
  (`set_user_data_dir_name(".eko")`) + `registry.rs:101-103`
  (`user_data_path("skills")` = `~/.eko/skills`), `enabled_skills.rs:22`
  (loaded from `user_data_path("enabled-skills.json")`); the post-sync
  refresh claim is only partially true (F-SKL-01-P2-01 no-op -> P2-02).
- Reachability: documentation-only (operators following skill-sync.md create
  directories the runtime never scans).
- Expected invariant: operator docs use the actual `.eko` root and describe
  the real refresh semantics.
- Observed behavior: three doc sites disagree with the code on the hub root,
  and the sync-refresh claim is inaccurate for already-installed skills.
- Impact: misconfiguration (skills installed at the documented path are never
  discovered) and misleading operator expectations.
- Root cause: docs predate the EKO `.eko` root switch (same drift family as
  A-CFG-01-P2-01/P3-01) and the reload no-op was never reflected.
- Direction: sed `~/.echo-agent` -> `~/.eko` in skill-sync.md and 06-skills.md
  (hub path rows), and reword the sync-refresh sentence to state that updated
  skills require re-discovery support (tracking F-SKL-01-P2-01).
- Regression validation: grep for `~/.echo-agent` in EKO skill docs returns
  zero hits (or only intentional framework references).
- Validation reports: [V05-01](../validations/A-PLG-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (plugin/skill/hook lifecycle, both repos) | yes | passed | [V01-01](../validations/A-PLG-01/V01-01.md) |
| V02 | Registration and runtime reachability trace (surfaces -> runtime -> framework) | yes | passed (deviations -> P1-01) | [V02-01](../validations/A-PLG-01/V02-01.md) |
| V03 | Invariants: prepare/activate ownership, real component registration, failed activation rollback | yes | passed (deviation -> P2-01) | [V03-01](../validations/A-PLG-01/V03-01.md) |
| V03 | Invariants: reload/unload, hook queue flush/shutdown, hub lifecycle | yes | passed (deviations -> P2-02, P3-01) | [V03-02](../validations/A-PLG-01/V03-02.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked plugin_runtime` | yes | passed (exit 0, 8 passed) | [V04-01](../validations/A-PLG-01/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked skills_hub` | yes | passed (exit 0, 6 passed) | [V04-02](../validations/A-PLG-01/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked hook` | yes | passed (exit 0, 20 passed) | [V04-03](../validations/A-PLG-01/V04-03.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked config_watcher` | yes | passed (exit 0, 3 passed) | [V04-04](../validations/A-PLG-01/V04-04.md) |
| V04 | `cargo test -p echo_core --lib --locked plugin` | yes | passed (exit 0, 41 passed) | [V04-05](../validations/A-PLG-01/V04-05.md) |
| V04 | `cargo test -p echo_execution --lib --locked hooks` | yes | passed (exit 0, 89 passed) | [V04-06](../validations/A-PLG-01/V04-06.md) |
| V04 | `cargo test -p echo_agent --lib --locked plugin` | yes | passed (exit 0, 1 passed) | [V04-07](../validations/A-PLG-01/V04-07.md) |
| V05 | Historical-document drift check | yes | passed (stale claims -> P1-01, P3-02) | [V05-01](../validations/A-PLG-01/V05-01.md) |

All required validations executed; every command has a known exit code; no
validation failed (four inspection deviations were promoted to findings).

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:72 P0-4 shared `PluginRuntimeService`, transactional candidate apply, failure restores previous set and callbacks | current | `apply_candidate` + tests; [V03-01](../validations/A-PLG-01/V03-01.md), [V04-01](../validations/A-PLG-01/V04-01.md) |
| MASTER-PLAN:73 P1 component wiring — framework Skills/Hooks/MCP + app agents/LSP/monitors/themes/styles exactly unloaded; GUI/TUI theme sync | current | `unload_agent_components` symmetric keys; [V02-01](../validations/A-PLG-01/V02-01.md), [V04-01](../validations/A-PLG-01/V04-01.md) |
| MASTER-PLAN:75 / :412-425 `HookEventDispatcher` bounded queue, backpressure, FIFO flush, idempotent shutdown | current | [V04-03](../validations/A-PLG-01/V04-03.md) |
| MASTER-PLAN:69 watcher hot-reloads hooks+webhooks, deletion removes registrations | current (scope) | config_watcher.rs:227-278 + A-CFG-01 P1-01 (switch freeze) |
| MASTER-PLAN:375 plugin `project_root` derived from `working_dir`, switch reflected without recreation | partial/stale | scan reads working_dir (:986-991) but LSP/base root boot-frozen (:608/:966, runtime.rs:591) and no switch trigger -> P1-01; [V05-01](../validations/A-PLG-01/V05-01.md) |
| MASTER-PLAN:51 skill upstream check/sync complete; sync refreshes the agent | current (with caveat) | hub sync works; post-sync runtime refresh is a no-op for existing skills -> P2-02 / F-SKL-01-P2-01 |
| skill-sync.md `~/.echo-agent/skills/` + `~/.echo-agent/enabled-skills.json` | stale | runtime root is `~/.eko` -> P3-02; [V05-01](../validations/A-PLG-01/V05-01.md) |
| 06-skills.md:263/:272 hub root `~/.echo-agent/skills/`; two-activation-path asymmetry | stale (root) / current (asymmetry) | root rename -> P3-02; asymmetry = F-SKL-01-P1-02 documented |
| F-PLG-01-P2-01 `.eko/plugins` listing vs runtime `.echo-agent/plugins` | current | config_discovery.rs:333 vs scope.rs:38-50; cross-referenced |
| F-PLG-01-P3-02 stale lifecycle entry after dropping reload | current | lifecycle `deactivate_all` keeps entries (lifecycle.rs:137-147); cross-referenced |
| F-SKL-01-P2-01 re-discovery silent no-op | current | capabilities.rs:687-695; cross-referenced (feeds P2-02) |

## Coverage And Uncertainty

- No process was launched; all behavior claims are traced code chains plus
  the executed test suites. The workspace-switch plugin behavior (P1-01) was
  not executed dynamically (no fixture harness in a read-only review; the
  switch path has zero unit tests, consistent with A-CFG-01's note that
  state.rs has no test module).
- The boot-time all-or-nothing plugin failure (P2-01) was verified
  statically; the mutation-path rollback it contrasts with is dynamically
  tested (V04-01).
- `register_lifecycle` has no production caller (F-PLG-01 V02); its P3-02
  stale-entry scenario is cross-referenced, not re-verified.
- Channels-only mode does not pass `plugin_runtime` to the channels handler
  (main.rs:357-403); no plugin surface exists there — recorded as
  informational, consistent with the headless-mode limitation comment in
  tauri/commands/plugins.rs:79-89.
- The frontend SkillsPanel/plugins panels were not reviewed (A-FE-* scope);
  only the Rust IPC surfaces were traced.
- Pool skill-descriptor refresh after hub mutations (`refresh_pool_skill_descriptors`,
  panels.rs:447-457) was noted but not traced into `agent_pool.rs` internals
  (A-SUB-01 scope).

## Handoff

- Downstream tasks may rely on: single framework plugin authority + single
  EKO runtime owner with a dynamically tested transactional reload/rollback
  (V01/V03-01/V04-01); symmetric source-scoped registration/unload for all
  plugin component types (V02); the task hook queue with explicit
  flush/idempotent shutdown (V03-02/V04-03); the four findings above.
- Reports to read: this report + V01-01..V05-01; F-PLG-01 (component
  ownership map, P2-01/P3-01/P3-02 cross-refs); F-SKL-01 (P1-01/P1-02/P2-01
  cross-refs, hub parser divergence P2-03/P3-01); A-CFG-01 (P1-01/P1-02
  workspace/hook freeze, P2-01 path map).
- Cross-references for the synthesizer: A-PLG-01-P1-01 is the plugin-runtime
  arm of A-CFG-01-P1-01 (workspace scope) — both should be fixed together in
  the switch path; A-PLG-01-P2-02 folds F-SKL-01-P2-01 (framework no-op) with
  the EKO-specific directory-granular enablement; F-PLG-01-P2-01 (discovery
  path divergence) is a prerequisite for any plugin-listing UI work;
  F-SKL-01-P1-01 (cyclic deps) and F-SKL-01-P1-02 (dual registry) remain
  reachable through EKO's plugin skill load (capabilities.rs:719-730).
- Stale triggers: changes to `plugin_runtime.rs`, `plugin_components.rs`,
  `skills_hub/*`, `hook_config_loader.rs`, `config_watcher.rs`,
  `tasks/task_runtime/hook_event_dispatcher.rs`, `state.rs`
  (switch_workspace/exit_workspace), `panels.rs` skill surfaces,
  `cmd_impls/{plugins,skills}.rs`, `tui/events.rs` plugin/hook sections, or
  `runtime.rs` bootstrap invalidate the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): X-PLG-01
  (lifecycle conformance, ownership map), X-SRF-01 (surface parity rows),
  Q-CLI-01/Q-STA-01 (dynamic fixtures for P1-01/P2-01), S-APP-01 (synthesis).
