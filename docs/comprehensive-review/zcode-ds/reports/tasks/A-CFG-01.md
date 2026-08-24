# A-CFG-01: Configuration, providers, and workspace lifecycle

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Are global/project config discovery, provider selection, workspace switching,
validation, and hot-reload scope coherent?

## Scope

- `echo-agent-app-core/src/config_discovery.rs` (full), `config_watcher.rs`
  (full), `model_config.rs` (full), `context_window.rs` (UI projection only),
  `hook_config_loader.rs` (full), `instruction_provider.rs` (path-resolution
  portion), `workspace/{mod,layout,registry,migration,templates}.rs`,
  `workspace_routing.rs` (full), `state.rs` (`switch_workspace` :844,
  `exit_workspace` :1053, `WebConfig` :32-51, `tasks_db_path` :1197),
  `infra.rs` (window wiring :23/:215-219/:258-262, hook load :1919,
  `refresh_dynamic_context` :528, agent creation :194-330),
  `tasks/task_runtime/store.rs` + `file_shadow.rs` (default root),
  `agent_pool.rs` (`apply_runtime_model` :425-466, `apply_workspace_routing` :563).
- `echo-agent-cli/src`: `main.rs` (config load :100, watcher :229-237),
  `tauri/desktop.rs` (:124-271), `tauri/commands/{config,providers,workspace}.rs`
  (full), `tauri/mod.rs` (registration :98-107), `cli/args.rs` (full),
  `cli/cmd_impls/{context.rs /model, workspace.rs}`, `tui/events.rs`
  (`/model` :2673-2740), `tui/commands.rs` (SlashCommand inventory).
- `echo-agent-cli/web-frontend`: `api/endpoints.ts` (config/workspace/
  provider endpoints), `stores/workspaceStore.ts`, `components/config/ConfigPanel.tsx`,
  `components/layout/LeftSidebar.tsx`.
- Framework: `echo-agent/src/config.rs` (`config_search_paths` :666,
  `load_config` :725, `save_config` :691, `resolve_context_window` :90-104,
  `to_agent_config` :111-162, `apply_compressor` :186), `src/paths.rs`
  (user-data root), `src/llm/core/capabilities.rs` (`infer_context_window`,
  cross-ref only).

## Out Of Scope

- Hook execution semantics / hook rule engine → F-SKL-01, A-PLG-01 (only
  hook-file discovery and reload scope are reviewed here).
- Workspace routing prompt content (workspace_routing.rs prompt text) and
  skill activation behavior → A-MEM-01/A-SUB-01 (routing call chain only).
- MCP config discovery (`mcp.json` paths) → A-INT-01 (inventory path claims
  only).
- Conversation/session persistence internals → A-STATE-01 (only the
  workspace-switch rebinding is traced here).
- TaskRuntime store semantics → A-TSK-01..04 (only its workspace binding).
- Window/budget arithmetic and compression → F-CTX-01 (cross-referenced for
  the window wiring path).
- CLI command surface parity in general → A-SRF-01/A-SRF-04 (only
  `/workspace` and `/model` reachability).

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-CFG-01 card), `zcode-ds/README.md`, templates.
- Dependency report read in full: `A-BOOT-01` (entry composition, watcher
  spawn sites, config-watcher cancel order; its P2-01/P3-06 cross-refs are
  used, not duplicated).
- Cross-reference reports read: `F-CTX-01` (P1-01 window mapping bypass),
  `F-EVO-01` (P3-03 echo-agent-eval crate removal).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/MASTER-PLAN.md`,
  `docs/configuration.md`, `docs/getting-started.md`,
  `docs/system-deep-dive/06-skills.md`, root `AGENTS.md` (echo-agent-eval
  rows), `echo-agent/docs/en|zh/28-config-reference.md`.

## Layering Decision

- Generic mechanism (framework): `echo_agent::config` (search paths, load/
  save with 0600 write), `config_search_paths`, `resolve_context_window`/
  `infer_context_window`, `paths::user_data_dir` override — correctly placed.
- EKO product policy (application): config file inventory/scopes
  (`ConfigDiscovery`), watcher reload scope (hooks/webhooks only), provider
  selection precedence (`model_config::resolve_runtime_model`), workspace
  switch/exit state replacement, workspace routing, GUI/TUI/REPL command
  surfaces, `docs/configuration.md`.
- Adapter boundary: `resolve_config_path` (config_watcher.rs:45-52) over
  framework search paths; `hook_config_loader` merge over framework
  `HooksDefinition`; `infra::create_agent` inline window resolution is an
  adapter that duplicates the framework's `resolve_context_window` instead of
  calling it (F-CTX-01-P1-01 family; this task adds the EKO-internal
  boot-vs-switch divergence).
- Duplicate search (terms + results in V01-01): `ConfigDiscovery`,
  `discover_config`, `ConfigInventory`, `config_search_paths`, `load_config`,
  `load_config_file`, `save_config`, `resolve_config_path`,
  `resolve_runtime_model`, `set_default_model`, `upsert_configured_model`,
  `effective_context_window`, `DEFAULT_CONTEXT_WINDOW` (2 sites),
  `to_agent_config`, `resolve_context_window`, `switch_workspace`,
  `exit_workspace`, `WorkspaceRegistry`, `WorkspaceLayout`, `LegacyMigrator`,
  `spawn_config_watcher`, `HookConfigLoader`, `load_merged`,
  `load_merged_from_disk`, `WebConfig`/`web_config` (write-only),
  `tasks_db_path` (dead), `worker`. One authoritative loader per semantic;
  the duplicates found are the findings below.

## Current Path

Verified call graph (V02-01):

- Config load: headless `main.rs:100` / GUI `desktop.rs:133` →
  `echo_agent::config::load_config(args.config)` (explicit path or
  `$ECHO_AGENT_CONFIG` → `./echo-agent.yaml` → `~/.eko/config.yaml`); GUI
  argv is fixed, so GUI never passes `--config`. Malformed explicit file →
  `AppConfig::default()` with error log; malformed search-path file → warn +
  continue. `AppConfig` is `#[serde(default)]` without
  `deny_unknown_fields`.
- Watcher: one `spawn_config_watcher` at boot on headless (main.rs:232) and
  GUI (desktop.rs:166); targets fixed at spawn = explicit config path +
  `~/.eko/hooks.yaml` + `<cwd-at-spawn>/.eko/hooks.yaml`; reload applies only
  hooks + webhook endpoints (config_watcher.rs:227-278) via the single
  `HookConfigLoader` (merge order: inline echo-agent.yaml < global hooks.yaml
  < project hooks.yaml at cwd).
- Provider selection: single app-core `resolve_runtime_model`
  (configured_models by id → default_model_id → first enabled → legacy
  `config.model`; token: provider-config → `config.model.auth_token` when
  provider matches → env; base_url: provider-config → model → provider
  default). Live switch surfaces: GUI IPC (`set_default_model`,
  `upsert_configured_model`, providers.rs) and TUI `/model`
  (tui/events.rs:2673-2740, incl. pool sync); REPL `/model` is a print-only
  stub (cmd_impls/context.rs:53-64).
- Window: boot uses `app_config.agent.token_limit` else hardcoded 396_000
  (infra.rs:23/:215-219/:258-262); `effective_context_window` inference
  (model_config.rs:153-159) is used only for the GUI model view; GUI/TUI
  model switch applies `context_window` → `agent.set_token_limit` live
  (providers.rs:113-115, tui/events.rs:2721-2723).
- Workspace switch: GUI LeftSidebar → `workspaceApi.switch` →
  `switch_workspace` IPC (workspace.rs:131-180) → `AppState::switch_workspace`
  (state.rs:844-1032): sets current workspace, `set_current_dir` to the
  workspace root, agent working_dir/tool artifacts/dynamic context, replaces
  persistence, conversation store, runtime state store, memory store + layer
  manager + curator + skills, pool working dir/memory, and workspace routing.
  It does not touch AppConfig, hooks, the config watcher, or the
  TaskRuntimeStore. `exit_workspace` (state.rs:1053-1185, called only from
  `delete_workspace` workspace.rs:110-114) resets the stores and routing to
  General but never restores CWD; frontend `workspaceStore.exit()` is dead.
- TUI has no workspace surface; REPL `/workspace switch` prints only.

## Findings

### A-CFG-01-P1-01: Workspace switch mutates the process CWD but leaves the config watcher targets, hook registry, and AppConfig bound to the pre-switch scope — the new workspace's project hooks are neither loaded nor watched, and edits to the old workspace's hooks still trigger a reload that reads the new scope

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-app-core/src/state.rs:854` — `std::env::set_current_dir(&workspace.root)` inside `switch_workspace`; the switch (:844-1032) never reloads `AppConfig`, never reloads user hooks, and never restarts the config watcher.
  - `config_watcher.rs:199-211` — watch targets are computed once at spawn (explicit config path + `~/.eko/hooks.yaml` + `cwd/.eko/hooks.yaml`, cwd captured at spawn); spawn sites are boot-only (`src/main.rs:232`, `src/tauri/desktop.rs:166`).
  - `hook_config_loader.rs:184-185` — project hooks resolve from `std::env::current_dir()` at call time, i.e. the new workspace after a switch, while the registry was populated at boot from the old cwd (infra.rs:1919).
  - `config_watcher.rs:254-278` — a reload triggered by an old-scope hooks file re-merges hooks from the current (new) cwd.
- Reachability: every GUI workspace switch (LeftSidebar `handleSwitch` → `workspaceApi.switch` → IPC → `AppState::switch_workspace`); the watcher keeps running after switch on both GUI and headless boots.
- Expected invariant: hot-reload scope follows the active workspace; after a switch, cwd-derived resolution (hooks, `local.md`, `echo-agent.yaml`, watcher targets) targets the new workspace and the old scope is released.
- Observed behavior: after switching, (a) the new workspace's `.eko/hooks.yaml` project hooks are never registered and never watched (absent until restart); (b) editing the old workspace's hooks file still fires a reload whose merge reads the new workspace's hooks — reload scope is decoupled from the watched file; (c) `echo-agent.yaml` in the new workspace root (now the cwd) is never loaded at switch time, so model/provider/max-iterations settings for the new project are ignored until restart.
- Impact: silent loss of workspace-scoped automation (hooks) on the primary surface; wrong-scope hook registration after stale-file edits; project config not applied on switch. This is the task's "hot-reload scope" coherence question answered in the negative.
- Root cause: workspace switch was implemented as "change CWD + swap stores" without treating cwd-derived subsystems (watcher targets, hook registry, AppConfig) as part of the workspace state.
- Direction: make switch/exit rebuild the watcher targets and re-merge hooks for the new cwd (reuse `HookConfigLoader` + a `spawn_config_watcher` restart), and re-resolve the new cwd's `echo-agent.yaml` (or explicitly document that config is global and the switch must not chdir for config purposes); add a switch fixture asserting watcher targets + hook set follow the workspace.
- Regression validation: unit fixture — boot in dir A with `.eko/hooks.yaml`, `switch_workspace` to dir B with different hooks; assert (a) B's hooks are registered, (b) a save of A's hooks file does not re-register, (c) a save of B's hooks file triggers exactly one reload. Q-A* dynamic suite can add the end-to-end variant.
- Validation reports: [V02-01](../validations/A-CFG-01/V02-01.md), [V03-01](../validations/A-CFG-01/V03-01.md)

### A-CFG-01-P1-02: `exit_workspace` never restores the process CWD — "global mode" keeps resolving cwd (config, local.md, project hooks) and running tools inside the exited workspace, and the only current caller deletes that directory afterwards

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  - `state.rs:854` (switch chdirs) vs `state.rs:1053-1185` (`exit_workspace` resets stores and routing to General but contains no `set_current_dir` back).
  - `infra.rs:136` — `working_dir: None` means "use process cwd"; `exit_workspace` sets `working_dir(None)` (state.rs:1068) so tool fallback = the exited workspace's root.
  - Only production caller: `src/tauri/commands/workspace.rs:110-114` (`delete_workspace` exits the current workspace, then `registry.delete` removes its directory — registry.rs:274-341), leaving the process cwd inside a deleted directory.
  - Frontend `workspaceStore.exit()` (workspaceStore.ts:105-107) is dead code; no backend "exit workspace" IPC command exists.
- Reachability: GUI delete of the active workspace; any future exit-workspace action.
- Expected invariant: exit restores the pre-workspace scope completely (CWD, stores, routing) so tool fallback and cwd-based config discovery return to global.
- Observed behavior: CWD stays in the exited workspace; config discovery, `local.md`, and project hooks still resolve to it; after deletion the CWD is a removed directory, so subsequent tools that fall back to process cwd fail or operate on a dead path.
- Impact: post-exit turns and tools run in the wrong (or deleted) directory; workspace-scope leak contradicts the exit contract.
- Root cause: `switch_workspace` introduced the global CWD mutation, and `exit_workspace` was written as a store-reset without the inverse CWD step.
- Direction: capture the pre-switch CWD in `switch_workspace` (or the workspace manager) and restore it in `exit_workspace` (guarded for existence); add a GUI exit command so the frontend dead `exit()` has a real backend target; regression fixture asserting cwd and tool working_dir revert after exit.
- Regression validation: fixture — switch to workspace W (cwd = W.root), call `exit_workspace`, assert `std::env::current_dir()` equals the pre-switch directory and `agent.config().working_dir` is None with tool execution resolving outside W.root; delete-W path asserts no tool call lands in W.root afterwards.
- Validation reports: [V02-01](../validations/A-CFG-01/V02-01.md), [V03-01](../validations/A-CFG-01/V03-01.md)

### A-CFG-01-P1-03: Workspace switching is GUI-only; the REPL `/workspace switch` and `/model` commands are print-only stubs that claim success, and the TUI has no workspace surface at all

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  - Real switch: Tauri IPC `switch_workspace` (tauri/mod.rs:107, commands/workspace.rs:131-180) + frontend LeftSidebar `handleSwitch` (LeftSidebar.tsx:95-105) → `workspaceApi.switch` (endpoints.ts:1422).
  - REPL stub: `src/cli/cmd_impls/workspace.rs:114-146` (`ws_switch` only prints "Switched to workspace ..."; never calls `AppState::switch_workspace`); `cmd_impls/context.rs:53-64` (`/model` prints "'{m}' was not applied").
  - TUI: `src/tui/commands.rs:58` SlashCommand enum has no workspace variant; grep of `src/tui/*.rs` finds no workspace switch/exit surface (V02-01).
  - AGENTS.md surface-parity invariant (TUI and GUI are feature-equal Agents; "禁止以'某模式不需要'为由拒绝给该模式接入能力").
- Reachability: REPL `/workspace switch` and `/model` are registered for every REPL session (repl.rs:167/:176); the TUI is the default headless surface.
- Expected invariant: workspace switching and provider/model selection available on every Agent surface (GUI, TUI, CLI/REPL), per AGENTS.md parity.
- Observed behavior: GUI can switch workspaces and models; TUI can switch models (`/model`, tui/events.rs:2673-2740) but has no workspace switching; REPL has neither (stubs print success messages).
- Impact: a TUI/REPL user cannot switch workspace or see any error; the printed "Switched to workspace" is misleading; capability parity is broken on the default headless surface.
- Root cause: workspace switching was built for the GUI IPC surface only; the CLI command was added as a placeholder and never wired to `AppState`.
- Direction: wire `/workspace switch|exit` in REPL/TUI to `AppState::switch_workspace`/`exit_workspace` via a shared context handle (CommandContext needs an AppState/workspace handle — currently it carries only AgentHandle), or implement a TUI sidebar workspace tab; either way, replace the print-only stubs with real behavior or remove them; add X-SRF-01 rows for workspace switching per surface.
- Regression validation: TUI/REPL fixture — create workspace W, `/workspace switch W`, assert `state.current_workspace()` and CWD change; `/model <id>` already covered by existing TUI behavior.
- Validation reports: [V02-01](../validations/A-CFG-01/V02-01.md)

### A-CFG-01-P2-01: Three-way contradictory config path map — the discovery inventory names a file the loader never reads, EKO operator docs point at the pre-EKO root, and the inventory scope model disagrees with the loaders

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - Inventory: `config_discovery.rs:221` — global agent config = `~/.eko/echo-agent.yaml`; the loader searches `~/.eko/config.yaml` (`echo-agent/src/config.rs:674` via `user_data_path`, EKO root set at main.rs:66/desktop.rs). `~/.eko/echo-agent.yaml` is never read by any loader (V01-01 grep).
  - Docs: `docs/configuration.md:8-10` — search order includes a nonexistent `./.echo-agent/echo-agent.yaml` tier and `~/.echo-agent/config.yaml`; `configuration.md`/`getting-started.md` contain zero `.eko` mentions (11+8 `.echo-agent` occurrences, V05-01).
  - Scope mismatch: `config_discovery.rs:374` marks every workspace `.workspace.json` as `ConfigScope::Global`; `config_discovery.rs:280` documents project hooks at `<project-root>/.eko/hooks.yaml` while the loader and watcher use `<cwd>/.eko/hooks.yaml` (hook_config_loader.rs:184-185, config_watcher.rs:205-207).
  - The inventory's only consumer, the `discover_config` IPC (config.rs:311-335), is registered (tauri/mod.rs:99) but has zero frontend callers — dead surface (V02-01).
- Reachability: `discover_config` registered but unreachable from any product UI; the wrong docs are the primary operator guidance.
- Expected invariant: one documented, enforced config path map; inventory, loader, watcher, and docs agree on file names, directories, and scope labels.
- Observed behavior: a user following the docs puts config at `~/.echo-agent/config.yaml` (never loaded — EKO uses `~/.eko`); a user following the inventory creates `~/.eko/echo-agent.yaml` (discovered but never loaded); both silently run on defaults.
- Impact: silent misconfiguration on the core config path; the config inventory feature is dead weight and actively misleading.
- Root cause: `ConfigDiscovery` was written against a design-time map and never reconciled with the framework loader or the EKO root override; the GUI surface that would surface it was never built.
- Direction: either (a) align the inventory with the real loader map (global = `~/.eko/config.yaml`, project = `<cwd>/echo-agent.yaml`, hooks at cwd, workspace scope variant) and wire `discover_config` into a real panel, or (b) delete `ConfigDiscovery` + the `discover_config` command per AGENTS.md cleanup; update `docs/configuration.md` to the real search order incl. `$ECHO_AGENT_CONFIG`.
- Regression validation: inventory test asserting the global agent path equals `config_search_paths()` last element; doc grep for `~/.echo-agent|\.echo-agent/echo-agent\.yaml` in EKO docs returns zero hits.
- Validation reports: [V01-01](../validations/A-CFG-01/V01-01.md), [V05-01](../validations/A-CFG-01/V05-01.md)

### A-CFG-01-P2-02: The TaskRuntime graph is not workspace-isolated — `switch_workspace` rebinds conversations/memory/checkpoints but never the task store, `tasks_db_path` is dead code, and the file shadow root is global

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `state.rs:844-1032` — switch replaces persistence, conversation store, runtime state store, memory store; never touches `self.tasks.runtime`.
  - `tasks/task_runtime/store.rs:169-187` (`TaskRuntimeStore::new`/`open` → `FileTaskShadow::default_root()`), `file_shadow.rs:83-85` — root = `echo_agent::paths::user_data_path("tasks")` (global `~/.eko/tasks`, not workspace).
  - `state.rs:1197-1203` — `tasks_db_path` is workspace-aware but has zero callers (V01-01).
  - `workspace/mod.rs:15` documents `tasks/` as part of the per-workspace layout.
- Reachability: the boot-created TaskRuntimeStore (state.rs:548, main.rs:37) is the single store for every workspace on every surface.
- Expected invariant: workspace isolation covers all workspace-scoped data (the workspace doc promises tasks under the workspace layout); or the task graph is documented as intentionally global.
- Observed behavior: task plans/runs created in workspace A remain visible and schedulable in workspace B; the workspace layout promises per-workspace task state that the runtime does not provide; the workspace-aware helper exists but is never called.
- Impact: cross-workspace task leakage (runs from another project visible/executable), contradicting the isolation model the rest of the switch implements; `tasks_db_path` is a maintenance trap.
- Root cause: task storage was migrated from SQLite to the global file shadow before workspace isolation was added; the switch was written around the stores that existed at the time.
- Direction: either rebind `self.tasks.runtime` to a workspace-scoped `FileTaskShadow` (`WorkspaceLayout::tasks(root)`) in `switch_workspace`/`exit_workspace` (and wire the pool binding), or delete `tasks_db_path` and update the workspace doc; coordinate with A-TSK-01/04 (store ownership) and A-PROJ-01 (workspace registry).
- Regression validation: fixture — create a task in workspace A, switch to B, assert `task_list` no longer returns it and background runs created in A do not resume in B; after fix, delete `tasks_db_path` and grep zero callers.
- Validation reports: [V02-01](../validations/A-CFG-01/V02-01.md), [V03-01](../validations/A-CFG-01/V03-01.md)

### A-CFG-01-P2-03: `web_config` is a write-only orphan config store — `update_config` silently drops `token_limit` and cannot apply `max_iterations`, so the ConfigPanel "已保存" flow claims a live sync that does not happen

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `state.rs:340` (`web_config: RwLock<WebConfig>`), `state.rs:479` (init), `src/tauri/commands/config.rs:118` (write) — no reader anywhere (V01-01 grep).
  - `update_config` (config.rs:106-153) applies only `system_prompt` to the agent (:132-141); `token_limit` is written to `web_config` and never read; `max_iterations` is not handled at all.
  - Frontend `ConfigPanel.tsx:52-66` calls `configApi.update` specifically to sync `max_iterations` (comment: "update_full_config only syncs model + system_prompt, not max_iterations"); `update_full_config` persists `max_iterations` to YAML (config.rs:194-196) but never applies it to the live agent (:290-303 applies temperature/max_tokens/system_prompt only).
- Reachability: every ConfigPanel save with a `max_iterations` or `system_prompt` edit (GUI live path); any legacy `update_config` caller.
- Expected invariant: a setting persisted and reported as "已保存" is either applied live or explicitly marked restart-required; no duplicate in-memory config store.
- Observed behavior: `max_iterations` edits silently take effect only after restart while the UI reports success and attempts a sync; `token_limit` via the legacy endpoint is dropped; `web_config` is a dead duplicate authority.
- Impact: users tuning "单轮推理安全上限" from the GUI get no live effect and no warning; the duplicate store invites future divergence (exactly the class AGENTS.md forbids).
- Root cause: legacy web-config hot-update path was partially migrated to `update_full_config`; the orphan `web_config` write and the ineffective `update()` sync were left behind.
- Direction: apply `max_iterations` in `update_config` (or remove the frontend sync and surface restart-required), delete `web_config`/`WebConfig` (or wire it as the single in-memory config), and remove the token_limit branch; UI should mark restart-required fields.
- Regression validation: ConfigPanel fixture — change max_iterations, save, assert `agent.config().get_max_iterations()` reflects the change (or the panel shows "restart required"); grep `web_config` → zero hits after deletion.
- Validation reports: [V03-01](../validations/A-CFG-01/V03-01.md)

### A-CFG-01-P2-04: `save_config` targets the process CWD when no config file exists and every GUI persist call ignores save errors — on a fresh GUI install config changes are silently lost

- Priority: P2
- Confidence: medium (depends on launch CWD and absence of any pre-existing config file)
- Layer: adapter (framework save target) / application (error swallowing)
- Evidence:
  - `echo-agent/src/config.rs:691-708` — `save_config` picks the first existing search path, else `search.get(1)` = `./echo-agent.yaml` (cwd), else `search.first()`; on macOS GUI launch the CWD is typically `/` (Finder/Dock), so the write fails.
  - GUI persist call sites are warn-only: `src/tauri/commands/config.rs:273-275`, `providers.rs:202-204/:226-228/:243-245`; the frontend shows "已保存" unconditionally (ConfigPanel.tsx:68).
- Reachability: every GUI save when no config file exists anywhere (fresh install before first successful save); the CWD write also affects TUI saves via the same `save_config` in a non-writable CWD.
- Expected invariant: persist failures are surfaced to the user; the global config file is written to the user-data root, not the launch CWD.
- Observed behavior: config edits apply in memory and are reported saved, but never reach disk; after restart all GUI config changes are gone.
- Impact: silent loss of user configuration on fresh installs — the exact "prevent silent data loss" class AGENTS.md says deserves protection.
- Root cause: `save_config`'s legacy cwd-first target was never updated for the EKO root, and the callers deliberately log-and-continue.
- Direction: make `save_config` prefer `~/.eko/config.yaml` (user_data_path) over cwd (or accept an explicit target from the GUI), and return the error to `update_full_config`/providers commands so the UI can surface "保存失败"; add a regression test writing from a non-writable CWD.
- Regression validation: unit fixture — empty config search set + cwd read-only, `save_config` must either succeed at `~/.eko/config.yaml` or return Err (no silent success); GUI-level assertion that a failed save surfaces an error message.
- Validation reports: [V03-01](../validations/A-CFG-01/V03-01.md)

### A-CFG-01-P2-05: Provider window mapping is split across three EKO sites and diverges between boot and live switch — the same model gets the hardcoded 396K window at boot and its configured/inferred window after a GUI/TUI model switch

- Priority: P2
- Confidence: high (code facts; the impact arm is owned by F-CTX-01-P1-01)
- Layer: adapter (framework/application wiring boundary)
- Evidence:
  - `infra.rs:23` (`DEFAULT_CONTEXT_WINDOW = 396_000`), `infra.rs:215-219/:258-262` (boot window/token_limit = `app_config.agent.token_limit` else 396K; the runtime model's `context_window` is ignored).
  - `model_config.rs:6` — second `DEFAULT_CONTEXT_WINDOW`; `model_config.rs:153-159` (`effective_context_window` inference) consumed only by the GUI model view (`configured_model_views`).
  - Live switch applies the window: `providers.rs:113-115` and `tui/events.rs:2721-2723` (`agent.set_token_limit(cw as usize)`), pool `agent_pool.rs:433-435` (`apply_runtime_model`).
- Reachability: every boot on any surface (window = 396K unless `agent.token_limit` set) vs every GUI/TUI model switch (window = configured `context_window`).
- Expected invariant: the same configured model yields the same effective window at boot and after any switch, and the window derives from model config/inference (F-CTX-01-P1-01).
- Observed behavior: a kimi k2.x model configured with `context_window: 256000` runs at boot with a 396K budget (overrun risk, F-CTX-01-P1-01) and, after the user opens the model switcher, at 256K; unknown models without explicit window keep 396K on both paths.
- Impact: boot/switch divergence produces model-dependent context errors and inconsistent compression behavior; duplicates the 396K constant in two app-core files.
- Root cause: the EKO boot adapter (`infra::create_agent`) resolved the window before `model_config::effective_context_window`/configured-model resolution existed, and was never reconciled with the live-switch path (same root as F-CTX-01-P1-01).
- Direction: in `infra::create_agent`, derive `token_limit` from the resolved `ModelRuntimeConfig.context_window` (falling back to `infer_context_window`/396K) and delete the duplicated constant; make boot and switch share one resolver; regression fixture asserting a kimi model builds with a 256K token_limit at boot.
- Regression validation: `cargo test -p echo-agent-app-core --lib model_config` plus an infra-level fixture — `create_agent` with a configured kimi model and no `agent.token_limit` yields `agent.config().get_token_limit() == 256_000`; switch path already covered by providers.rs behavior.
- Validation reports: [V01-01](../validations/A-CFG-01/V01-01.md), [V03-01](../validations/A-CFG-01/V03-01.md)

### A-CFG-01-P3-01: EKO operator docs use the pre-EKO root `~/.echo-agent` and a nonexistent `./.echo-agent/echo-agent.yaml` tier; the workspace module doc still says "后台任务 SQLite DB"; the watcher doc's restart justification contradicts the live model-switch surfaces

- Priority: P3
- Confidence: high
- Layer: application (docs)
- Evidence:
  - `docs/configuration.md:8-10` (search order with nonexistent tier + pre-EKO root), `docs/getting-started.md` (8 `.echo-agent` occurrences, zero `.eko` — V05-01).
  - `workspace/mod.rs:15` — "`tasks/` # 后台任务 SQLite DB" in the per-workspace layout (no SQLite in EKO; A-BOOT-01-P3-06 flagged the same family at infra.rs:125 / state.rs:931).
  - `config_watcher.rs:8-10` — "Model selection ... wired into long-lived subsystems at agent construction" vs live application via providers.rs:113-115 and tui/events.rs:2721-2723.
  - Root `AGENTS.md:139,370` still lists the removed `echo-agent-eval` crate (F-EVO-01-P3-03, cross-referenced; root-owned, not fixable by this review).
- Reachability: documentation-only.
- Expected invariant: operator/config docs describe the real search order, root, and reload scope; no SQLite wording in the no-SQLite application layer.
- Observed behavior: four documentation sites disagree with the code on paths, root, storage engine, and reload scope.
- Impact: misconfiguration and maintenance confusion; the watcher doc misrepresents why model changes need a restart while the GUI applies them live.
- Root cause: docs predate the EKO `~/.eko` root switch, the SQLite removal, and the live model-switch commands; each was updated independently or not at all.
- Direction: rewrite `configuration.md` search order (add `$ECHO_AGENT_CONFIG`, cwd `echo-agent.yaml`, `~/.eko/config.yaml`), sed `~/.echo-agent` → `~/.eko` in EKO docs, fix workspace/mod.rs:15 wording, and reword the watcher scope note to reference the live IPC/TUI model paths; AGENTS.md rows owned by root maintainer (F-EVO-01-P3-03).
- Regression validation: grep for `~/.echo-agent|\.echo-agent/echo-agent\.yaml|SQLite DB|echo-agent-eval` in EKO docs returns zero hits (or only intentional references).
- Validation reports: [V05-01](../validations/A-CFG-01/V05-01.md)

### A-CFG-01-P3-02: `update_full_config` never syncs the agent pool — pooled/background agents keep the pre-save model, temperature, max_tokens, and system prompt after a ConfigPanel save

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `src/tauri/commands/config.rs:290-303` syncs only the primary agent; the only pool sync sites are `providers.rs:127-131` and `tui/events.rs:2728` (model commands); `pool.update_app_config` has no other callers (V01-01).
- Reachability: every ConfigPanel save (config.rs:270-303) while pooled agents exist.
- Expected invariant: configuration changes apply to all live Agent surfaces/agents (pooled agents are the same product runtime).
- Observed behavior: background/pooled agents run with stale settings until the next model switch or restart.
- Impact: background tasks use outdated model/settings after the user saves config; low likelihood of user-visible harm, but an inconsistent live-apply contract.
- Root cause: `update_full_config` was written before pool sync existed.
- Direction: extend the sync block (config.rs:290-303) to call `pool.update_app_config(app_config)` + a pool-side apply for the changed fields, mirroring providers.rs:127-131.
- Regression validation: fixture — pooled agent exists, `update_full_config` changes temperature, assert the pooled agent's config reflects it.
- Validation reports: [V03-01](../validations/A-CFG-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (config discovery/precedence, both repos) | yes | passed | [V01-01](../validations/A-CFG-01/V01-01.md) |
| V02 | Registration and runtime reachability (loaders, provider commands, workspace switch, watcher, dead surfaces) | yes | passed | [V02-01](../validations/A-CFG-01/V02-01.md) |
| V03 | Invariant/edge cases: precedence & path map, invalid/partial config, workspace switch state replacement, restart vs live reload | yes | passed (violations → P1-01/02, P2-01..05, P3-01/02) | [V03-01](../validations/A-CFG-01/V03-01.md) |
| V04 | Targeted tests/compile: app-core config/workspace suites (67 tests), framework config suite (23 tests), `cargo check -p echo-agent-cli` | yes | passed, exit 0 each | [V04-01](../validations/A-CFG-01/V04-01.md), [V04-02](../validations/A-CFG-01/V04-02.md), [V04-03](../validations/A-CFG-01/V04-03.md) |
| V05 | Historical-document drift (configuration.md, getting-started, MASTER-PLAN, workspace docs, AGENTS.md eval rows) | yes | passed | [V05-01](../validations/A-CFG-01/V05-01.md) |

All required validations executed; every command has a known exit code; no
validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `docs/configuration.md:8-10` — search order includes `./.echo-agent/echo-agent.yaml` and `~/.echo-agent/config.yaml` | stale | real order: `$ECHO_AGENT_CONFIG` → `./echo-agent.yaml` → `~/.eko/config.yaml` (config.rs:666-676); EKO root `~/.eko` (main.rs:66) → P2-01/P3-01 |
| `docs/configuration.md` — GUI API key priority over env; CLI/TUI read same configured default model | current (partial) | resolve_runtime_model token precedence (model_config.rs:278-293); TUI `/model` works, REPL `/model` stub → P1-03 |
| `MASTER-PLAN.md:253` — workspace switch/exit refresh the primary Agent immediately | partial | projections refresh on both paths (state.rs:883/:1072); exit does not restore CWD, switch does not reload hooks/config → P1-01/P1-02 |
| `MASTER-PLAN.md:375` — plugin project_root derived from working_dir, switch reflected without recreation | current for switch; stale for exit | working_dir=None on exit (state.rs:1068) falls back to CWD which stays in the exited workspace → P1-02 |
| `workspace/mod.rs:15` — per-workspace `tasks/` "后台任务 SQLite DB" | stale | no SQLite; task shadow global `~/.eko/tasks` (file_shadow.rs:83-85); not rebound on switch → P2-02; P3-01 (SQLite wording) |
| `docs/system-deep-dive/06-skills.md:427` — echo-agent-eval crate removed | current | workspace members = app-core only; `evals/` empty (F-EVO-01 V01) |
| Root `AGENTS.md:139,370` — echo-agent-eval is an EKO submodule | stale | F-EVO-01-P3-03 (root-owned) — cross-referenced, not duplicated |
| `echo-agent/docs/zh|en/28-config-reference.md` — `~/.echo-agent/config.yaml` | current for default framework, stale at EKO boundary | DEFAULT_USER_DATA_DIR_NAME = `.echo-agent` (paths.rs:25); EKO overrides to `.eko` (paths.rs:58) — doc omits the override → P3-01 |
| `config_watcher.rs:5-11` — hooks/webhooks live; model/MCP/runtime limits restart-required | current in scope, stale in justification | watcher reloads only hooks+webhooks (:254-278); GUI/TUI apply model live (providers.rs:113-115, tui/events.rs:2721-2723) → P3-01 |
| `A-BOOT-01` P2-01 (temporary AppState) / P3-06 (sqlite comments) | current | not duplicated here; this task adds the workspace/mod.rs:15 SQLite wording site (P3-01) |

## Coverage And Uncertainty

- No process was launched; every behavior claim is a traced code chain (V02/
  V03). Dynamic confirmation of malformed-config boot fallback, GUI
  switch-then-edit-hooks behavior, and exit-then-tool behavior belongs to the
  Q-* dynamic suites; the chains are statically unambiguous.
- `switch_workspace`/`exit_workspace` have zero unit tests (state.rs has no
  test module) — the workspace state-replacement invariants are untested
  (V04-01 notes the gap).
- The malformed-YAML fallback of `load_config` is untested (only no-file and
  env-var cases exist) — V03-01 relies on static evidence.
- `P2-04` (save_config CWD target) depends on the launch CWD being
  non-writable and no config file existing; macOS Finder launch CWD `/` is
  the common case but was not executed here.
- GUI argv fixedness, watcher cancel order, and startup rollback are
  A-BOOT-01 territory; only the config/provider/workspace aspects were
  re-verified here.
- `docs/configuration.md` provider templates (per-provider YAML examples)
  were sampled, not diffed field-by-field against `ProviderTemplate`.

## Handoff

- Downstream tasks may rely on: single-authority config/hook loaders (V01);
  watcher scope = hooks+webhooks, targets frozen at boot CWD (P1-01);
  workspace switch replaces stores but not CWD-derived scope/task store
  (P1-01/P1-02/P2-02); workspace switching is GUI-only (P1-03); provider
  window resolution diverges between boot and switch (P2-05, extends
  F-CTX-01-P1-01); the config path map has three contradictory documented
  forms (P2-01); `web_config` is a write-only orphan and ConfigPanel
  max_iterations sync is ineffective (P2-03); save_config cwd target +
  silent GUI save errors (P2-04); doc drift list (P3-01); pool sync gap
  (P3-02).
- Reports to read: this report + V01-01..V05-01; A-BOOT-01 (entry wiring,
  watcher spawn sites); F-CTX-01 (window authority); F-EVO-01 (eval crate
  removal).
- A-MEM-01: verify instruction projection refresh on switch/exit against the
  CWD-staleness of exit (P1-02 arm); workspace routing refresh (state.rs:1029)
  is already wired.
- A-TSK-01/04 and A-PROJ-01: workspace binding of the TaskRuntimeStore and
  `tasks_db_path` deletion/replacement (P2-02).
- A-SRF-01/A-SRF-04/X-SRF-01: workspace switching and model switching rows
  per surface (P1-03); `/model` REPL stub removal or wiring.
- A-PLG-01: hook reload scope after workspace switch (P1-01) and the
  `ConfigChange` hook path.
- Q-CLI-01/Q-GUI-01/Q-STA-01: dynamic fixtures for P1-01/P1-02/P2-03/P2-04.
- This report becomes stale if: `switch_workspace`/`exit_workspace`
  (state.rs:844/1053), `spawn_config_watcher` targets (config_watcher.rs:199),
  `config_search_paths`/`save_config` (echo-agent config.rs:666/691), the
  boot window wiring (infra.rs:215-262), the model switch commands
  (providers.rs), or the REPL/TUI command surfaces change.
