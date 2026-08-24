# A-CFG-01: Configuration, providers, and workspace lifecycle

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: not-applicable (read-only inspection of framework config module at current HEAD)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Are global/project config discovery, provider selection, workspace switching,
validation, and hot-reload scope coherent?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/config.rs:663-784` — `config_search_paths`, `load_config_file`,
  `load_config`, `save_config`, `apply_env_overrides`.
- `echo-agent-cli/echo-agent-app-core/src/model_config.rs` (full, 418 lines) —
  provider templates, configured-model views, `resolve_runtime_model`,
  `find_env_api_key`, `set_default_model`.
- `echo-agent-cli/echo-agent-app-core/src/config_discovery.rs` (full, 462 lines)
  — `ConfigDiscovery`, `ConfigInventory`, per-category discovery.
- `echo-agent-cli/echo-agent-app-core/src/config_watcher.rs` (full, 319 lines)
  — `spawn_config_watcher`, `handle_config_change`, debounce + watch-target
  selection.
- `echo-agent-cli/echo-agent-app-core/src/state.rs:339-600, 834-1010` —
  `AppState` config fields (`app_config`/`web_config`/`sandbox_config`/`mcp_config`),
  `WorkspaceState`, `switch_workspace`.
- `echo-agent-cli/echo-agent-app-core/src/infra.rs:108-145, 194-340` —
  `AgentCreateParams`, `create_agent_with_diagnostics`, `LlmConfig` injection.
- `echo-agent-cli/src/cli/args.rs`, `src/main.rs:95-233`, `src/tauri/desktop.rs:133-167`
  — CLI/env entry-point wiring and watcher spawn.

## Out Of Scope

Deferred to downstream tasks:

- **A-BOOT-01** (complete, read) — entry-point composition and where
  `load_config` / `spawn_config_watcher` are invoked. This task sharpens the
  config-specific behavior only.
- **A-MCP-*** — MCP config file (`mcp.json`) parsing topology, server lifecycle.
  This task only notes that MCP topology is in the restart-required set.
- **A-TASK-*** / **A-POOL-*** — TaskRuntime / AgentPool internals touched by
  `switch_workspace` (store re-bind, pool `apply_working_dir`); correctness of
  those subsystems is owned there.
- **F-LLM-01** — framework `LlmConfig` / provider client correctness.
- **B-PATH-01** — full IPC handler inventory; only the workspace-switch IPC
  surface is touched here.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` — local-personal-assistant threat model (drives
  the save_config 0600 judgment and the no-extra-permission-gates stance),
  layering gate, no-duplicate rule, UTF-8 safety rule.
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.
- `docs/comprehensive-review/REPORTING.md`.

Dependency reports read:

- `zcode-glm/tasks/A-BOOT-01.md` (complete) — confirmed `AgentRuntime::bootstrap`
  is the single composition root; `AppState.app_config: RwLock<AppConfig>` is the
  central config holder; the config watcher is spawned once at startup
  (`main.rs:231`, `desktop.rs:165`).

Historical documents treated as hypotheses: none.

## Layering Decision

This is an **application-layer** task with one framework touchpoint.

- **Generic mechanism (framework, correctly placed):** file-based config search,
  YAML parse, `AppConfig` struct, `save_config` 0600 permission mitigation,
  `apply_env_overrides` for channel/MCP path. These are reusable across any
  echo-agent consumer and live in `echo-agent/src/config.rs`.
- **EKO product policy (application, correctly placed):** provider/model
  resolution (`resolve_runtime_model`), configured-models UI projection
  (`configured_model_views`), the config watcher's "hooks + webhook only"
  reload scope, workspace switch semantics, `LlmConfig` injection at agent
  build. All in `echo-agent-app-core`.
- **Adapter boundary:** CLI arg → `AgentCreateParams` translation in
  `main.rs` / `desktop.rs`. Thin, no policy.

Duplicate-search terms run across the whole `echo-agent-cli` tree:

- `load_config` / `config_search_paths` — single authority in framework
  (`config.rs:666,725`); called from `main.rs:100`, `desktop.rs:133`,
  `config_watcher.rs:256`. No parallel loader.
- `resolve_config_path` — single (`config_watcher.rs:45`), called from both
  entry points. No duplicate.
- `spawn_config_watcher` — single definition, two call sites (headless + GUI).
- `switch_workspace` — single definition (`state.rs:844`); no parallel switch.
- `apply_env_overrides` — single definition (`config.rs:763`); both entry points
  call it. **Not** called by the watcher's reload path — env overrides are
  bootstrap-only (a gap, see P2-03).

No parallel implementations of the same config semantic were found. The
framework/application split is clean.

## Current Path

### Config load (bootstrap, both entry points)

`main.rs:100` and `desktop.rs:133` both execute:
`load_config(args.config.as_deref())` → `apply_env_overrides(&mut app_config)`
→ optional `--verbose` log-level override → pass `&app_config` into
`AgentRuntime::bootstrap` (`runtime.rs:74`) which clones it into
`AgentRuntime { app_config }` (`runtime.rs:361`) and ultimately into
`AppState.app_config: RwLock<AppConfig>` (`state.rs:478`).

### Precedence (verified — see V01)

Substitution, not merge. First existing + parseable file wins:
`--config` arg → `$ECHO_AGENT_CONFIG` → `./echo-agent.yaml` →
`~/.eko/config.yaml` → `AppConfig::default()`. `apply_env_overrides` overlays
only `channels.{qq,feishu}.*` and `mcp.config_path`. CLI `--model` overrides
only the model **name** string (`infra.rs:205`); provider/auth/base_url stay
from the YAML.

### Provider / auth resolution (`model_config.rs:278-293`)

`config.model_providers[p].auth_token` → (if `p == config.model.provider`)
`config.model.auth_token` → `find_env_api_key(p)` (first non-empty env var from
`provider_env_var_names(p)`) → `(None, "none")`. The `None` case is silently
carried into `create_agent_with_diagnostics` which then **skips** LlmConfig
injection (`infra.rs:307-320`).

### Hot reload (`config_watcher.rs`)

Watcher targets = {resolved config file, `~/.eko/hooks.yaml`,
  `<cwd-at-bootstrap>/.eko/hooks.yaml`}. On settle (resettable 500 ms debounce):
fire `ConfigChange` hook → reload user hooks registry (last-known-good retained
on parse error) → `webhook_emitter.reload_from_config(&new_config)`. The loaded
`new_config` is consumed only by the webhook emitter; `AppState.app_config` is
never updated.

### Workspace switch (`state.rs:844-1010`)

Replaces: `workspace.current`, process CWD (`set_current_dir`), primary + pool
agent `working_dir`, `persistence`, `conversation_store`, `RuntimeStateStore`,
memory store + layer manager, `ReviewIntegration` rebind, workspace-curated
skills. Does **not** replace: `app_config`, `web_config`, `sandbox_config`,
`permission_mode`, `mcp_config`, provider auth/base_url, or the watcher's target
list.

### Error handling (`config.rs:681-757`)

All config-load failures are silent fallback: corrupt explicit → `default()`
with `error!`; corrupt search-path → next path with `warn!`; missing API key →
deferred to first LLM call.

## Findings

### A-CFG-01-P1-01: Config watcher targets are not refreshed on workspace switch

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:199-211`
  (target list built once from cwd-at-spawn); `state.rs:853-864`
  (`switch_workspace` mutates process CWD but not the watcher).
- Reachability: `spawn_config_watcher` (`main.rs:231`, `desktop.rs:165`) →
  builds targets from `std::env::current_dir()` once → user invokes
  `switch_workspace` IPC → `set_current_dir(&workspace.root)` → new workspace's
  `.eko/hooks.yaml` is never watched for the rest of the process.
- Expected invariant: after switching to workspace W, edits to
  `W/.eko/hooks.yaml` hot-reload exactly like the bootstrap workspace's hooks
  did.
- Observed behavior: the watcher continues observing the pre-switch cwd's
  `.eko/hooks.yaml` (which may no longer exist or be irrelevant); the new
  workspace's hooks file is invisible to hot-reload until process restart.
- Impact: a user who edits hooks in the switched-to workspace sees no effect
  and has no feedback explaining why. The hooks only apply after a full app
  restart, defeating the purpose of hot-reload in multi-workspace use.
- Root cause: the watcher owns its target list as a `Vec<PathBuf>` captured at
  spawn time and has no reconfiguration API; `switch_workspace` has no handle to
  the watcher task.
- Direction: either (a) give `switch_workspace` a handle to re-register the new
  workspace's hooks file (unwatch stale, watch new), or (b) widen the watcher to
  observe a workspace-relative glob resolved dynamically. Option (a) is the
  smaller blast radius and matches the existing per-file watch model.
- Regression validation: switch workspace, edit `<new>/.eko/hooks.yaml`, assert
  the registry reloads within debounce window; switch back, assert the original
  file is re-watched.
- Validation reports: [V02](../validations/A-CFG-01/V02-01.md),
  [V03](../validations/A-CFG-01/V03-01.md)

### A-CFG-01-P1-02: Missing provider API key does not fail fast at bootstrap

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:303-320`
  (LlmConfig injection guarded by `if let Some(auth_token)`);
  `model_config.rs:287-293` (`None` returned when no config + no env).
- Reachability: every bootstrap path with a config that has no
  `model_providers[p].auth_token`, no `model.auth_token`, and no env var →
  `resolve_runtime_model` returns `auth_source: "none"` →
  `create_agent_with_diagnostics` skips `builder.llm_config(...)` → agent
  builds successfully → first user message triggers the provider call → opaque
  401 / "no API key" error.
- Expected invariant: a local assistant should refuse to start (or emit a
  clear, actionable error) when no model authentication is resolvable, rather
  than deferring the failure to the first chat turn.
- Observed behavior: bootstrap succeeds with no auth; the inline comment at
  `infra.rs:304-305` acknowledges the fallback ("env vars +
  echo-agent-models.yaml, which may not exist, especially in GUI apps where
  shell env vars aren't inherited"). GUI launches are the worst case because
  shell env vars are absent.
- Impact: new users (especially GUI) hit a confusing failure on their first
  message instead of a guided "configure your API key" prompt at startup. The
  failure mode is also indistinguishable from a network outage.
- Root cause: there is no validation gate between auth resolution and agent
  construction. The `if let Some` is a silent skip, not an error.
- Direction: in `create_agent_with_diagnostics` (or in `bootstrap`), when
  `runtime_model.auth_token.is_none()` AND no framework-level fallback is
  configured, surface a typed error (or a startup diagnostic event consumed by
  the GUI/TUI to render a setup screen). Keep the env-fallback path for
  headless users who legitimately configure via env vars — the gate should fire
  only when **both** config and env are empty.
- Regression validation: bootstrap with empty config + no env → assert typed
  error / setup surface; bootstrap with `ANTHROPIC_API_KEY` env → assert
  success.
- Validation reports: [V04](../validations/A-CFG-01/V04-01.md)

### A-CFG-01-P2-01: No global→project config merge — first file wholly wins

- Priority: P2
- Confidence: high
- Layer: framework (load behavior) + application (UI projection)
- Evidence: `echo-agent/src/config.rs:741-753` (returns on first parseable file,
  no merge); `config_discovery.rs:219-240` (advertises both global and project
  `echo-agent.yaml` as if both contributed).
- Reachability: every `load_config` call. A user with a global
  `~/.eko/config.yaml` and a project `./echo-agent.yaml` gets **only** the
  project file's contents; any key absent from the project file silently
  reverts to `AppConfig::default()`, not to the global file's value.
- Expected invariant: when both global and project config exist, project keys
  override global keys, and global keys remain in effect for anything the
  project file does not set. This is what `config_discovery`'s
  Global/Project scope labeling implies to the user.
- Observed behavior: winner-takes-all; the loser is invisible. Discovery UI
  shows two files, loading honors one.
- Impact: operators who split provider auth (global) from project-specific
  prompts/limits (project) silently lose the global half when the project file
  is present. Surprising and hard to debug.
- Root cause: `config_search_paths` returns a flat list and `load_config`
  returns the first hit; there is no merge step. `config_discovery` was written
  against a layered model that the loader does not implement.
- Direction: either implement layered merge (global base → project overlay →
  explicit override) in `load_config`, or change `config_discovery` /
  documentation to state plainly that only one file is honored and which one.
  The merge option is a real behavior change; the doc option is the lower-risk
  fix. Recommend the doc/label fix now and treat layered merge as a product
  decision.
- Regression validation: with both files present, assert which keys win and
  that the outcome matches the documented model.
- Validation reports: [V01](../validations/A-CFG-01/V01-01.md)

### A-CFG-01-P2-02: Filename mismatch between loader and discovery inventory

- Priority: P2
- Confidence: high
- Layer: framework (filename) vs application (discovery)
- Evidence: `echo-agent/src/config.rs:674` (global path is
  `paths::user_data_path("config.yaml")`); `config_discovery.rs:221` (global
  agent file advertised as `~/.eko/echo-agent.yaml`); project file is
  `echo-agent.yaml` at `config.rs:673` and `config_discovery.rs:232` (these two
  agree, so the mismatch is only on the global name).
- Reachability: any user who follows the discovery/UI naming and creates
  `~/.eko/echo-agent.yaml` — the loader never reads it (it looks for
  `config.yaml` there), so the user's global config is silently ignored.
- Expected invariant: the filename the UI shows users is the filename the
  loader reads.
- Observed behavior: loader reads `~/.eko/config.yaml`; discovery tells users
  the file is `~/.eko/echo-agent.yaml`.
- Impact: silent configuration loss for users who trust the discovery name.
  Hard to diagnose because the loader logs "No config file found, using
  defaults" (`config.rs:755`) while the user can see their file on disk.
- Root cause: the two modules were authored against different naming
  conventions and never reconciled.
- Direction: pick one canonical global filename and make both modules use it.
  Given `save_config` writes the first search path it finds and the loader's
  authority is `config.yaml`, either (a) update `config_discovery` to advertise
  `config.yaml`, or (b) add `echo-agent.yaml` to `config_search_paths` as an
  alias and have `save_config` prefer it. (a) is the smaller change.
- Regression validation: create the advertised file, bootstrap, assert it is
  loaded (loader log + effective config).
- Validation reports: [V01](../validations/A-CFG-01/V01-01.md)

### A-CFG-01-P2-03: Hot-reload does not refresh `AppState.app_config` (stale snapshot)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `config_watcher.rs:254-277` (`new_config` built but only passed to
  `webhook_emitter.reload_from_config`); `state.rs:339,478`
  (`AppState.app_config: RwLock<AppConfig>`, written once at
  `from_shared`).
- Reachability: any config edit after bootstrap → watcher fires → hooks +
  webhook update → IPC handler later reads `app_config.read()` → returns the
  pre-edit snapshot.
- Expected invariant: fields advertised as "live" via the watcher (e.g.
  webhook endpoints) and the central `app_config` agree with disk.
- Observed behavior: `webhook_emitter` gets the new config but `app_config`
  does not. IPC that reads `app_config` (e.g. surfacing current channels
  config, MCP path, token_limit for UI) returns stale values until restart.
- Impact: partial live-reload is inconsistent across surfaces — some domains
  update, others don't, with no signal to the client. The module doc
  (`config_watcher.rs:6-11`) declares the limited scope, but `app_config`'s
  staleness is an implementation consequence, not a documented contract.
- Root cause: `handle_config_change` does not receive an `AppState` handle and
  has no path to write `app_config`.
- Direction: decide explicitly which `app_config` fields are safe to refresh
  live (channels config, webhook config, MCP path are already effectively
  reloaded in their subsystems) and have the watcher write those into
  `app_config` so IPC reads converge. Fields wired into the agent (model,
  provider, token_limit) stay restart-required and should be documented as
  such. Do **not** widen to agent-affecting fields without a rebuild story
  (the module doc warns about this correctly).
- Regression validation: edit a channels/webhook key in the config file,
  assert both the webhook emitter and `app_config.read()` reflect the new value
  within the debounce window.
- Validation reports: [V02](../validations/A-CFG-01/V02-01.md)

### A-CFG-01-P2-04: Workspace switch does not reload config (and does not say so)

- Priority: P2
- Confidence: medium
- Layer: application
- Evidence: `state.rs:844-1010` (full `switch_workspace` body — no
  `load_config` / `app_config.write` / `apply_env_overrides`).
- Reachability: user switches workspace → process CWD changes →
  `config_search_paths()` would now resolve `./echo-agent.yaml` against the new
  root, but no reload occurs.
- Expected invariant: either (a) workspace switch re-resolves config from the
  new root, or (b) the system documents that config is global and unaffected by
  workspace switch.
- Observed behavior: neither. CWD changes (so a future `load_config` would
  behave differently) but the live config does not change; nothing tells the
  user that per-workspace `echo-agent.yaml` is ignored.
- Impact: a user who places `echo-agent.yaml` in a workspace expecting it to
  take effect after switching is silently disappointed. Combined with P1-01,
  both config and hooks are stale after a switch.
- Root cause: `switch_workspace` was designed for storage/memory/skills
  isolation and never wired to the config subsystem.
- Direction: pick a stance. If config is global (reasonable for a personal
  assistant), document it on `switch_workspace` and in the workspace UI, and
  consider not mutating process CWD (use explicit roots instead) so search
  paths stay stable. If per-workspace config is desired, call
  `load_config` + `apply_env_overrides` on switch and write into `app_config`
  (with the same restart-required caveats for agent-bound fields).
- Regression validation: place `echo-agent.yaml` in workspace B, switch A→B,
  assert documented behavior (either reloaded or explicitly ignored with a log).
- Validation reports: [V03](../validations/A-CFG-01/V03-01.md)

### A-CFG-01-P2-05: Corrupt explicit config file silently falls back to defaults

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/config.rs:726-738` (on parse failure of an
  explicit `--config` path: `error!` + `AppConfig::default()`); `:741-753`
  (search-path parse failure: `warn!` + continue to next path).
- Reachability: any malformed YAML at the resolved config path.
- Expected invariant: a user-provided explicit config (`--config` or
  `$ECHO_AGENT_CONFIG`) that fails to parse should be a hard error, not a
  silent downgrade to defaults.
- Observed behavior: explicit path parse error returns `AppConfig::default()`
  and the process boots as if no config were supplied. The only signal is a
  log line.
- Impact: operator thinks their config is active; reality is defaults.
  Particularly bad for sandbox/permission tuning where silent defaults may be
  more permissive than the operator intended.
- Root cause: `load_config` treats "file unreadable" and "file unparseable" the
  same as "file absent" for the explicit case.
- Direction: split the cases. For an explicit `--config`/`ECHO_AGENT_CONFIG`,
  parse failure should `Result::Err` (or `process::exit` with a clear message)
  at bootstrap. For search-path resolution, keep fallthrough but distinguish
  "absent" (silent, normal) from "present but corrupt" (warn loudly, possibly
  surface in UI).
- Regression validation: bootstrap with `--config broken.yaml`; assert a clear
  startup error and non-zero exit (or surfaced setup screen).
- Validation reports: [V04](../validations/A-CFG-01/V04-01.md)

### A-CFG-01-P3-01: CLI `--model` overrides only the name, not provider — undocumented

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `infra.rs:205`
  (`params.model.as_deref().unwrap_or(&runtime_model.model)`); `args.rs:37-39`
  (`--model` help text: "模型名称（不指定则使用配置文件中的值）").
- Reachability: every launch with `--model <name>` (or `MODEL_NAME` env).
- Expected invariant: `--model` either selects a fully-configured model
  (provider + auth) or its docs state it is a name-only override.
- Observed behavior: only the model name string is replaced; provider,
  auth_token, base_url stay from the YAML's default. Passing
  `--model gpt-4o` when the config default is an Anthropic model keeps the
  Anthropic provider/auth and sends `gpt-4o` as the model id — a request that
  will fail or be billed against the wrong account.
- Impact: footgun for CLI/TUI users who reasonably expect `--model` to select a
  model end-to-end.
- Root cause: `--model` predates the `configured_models` array; it was a simple
  name override and was never upgraded to a model-id selector.
- Direction: either (a) reinterpret `--model` as a `configured_model.id`
  selector (use `set_default_model` semantics), or (b) keep name-only and
  update `--model`'s help text to say "overrides the model name only; provider
  and auth come from the configured default". (b) is the safe, minimal fix.
- Regression validation: launch with `--model <id>` against a multi-provider
  config; assert documented behavior.
- Validation reports: [V01](../validations/A-CFG-01/V01-01.md)

### A-CFG-01-P3-02: `apply_env_overrides` is not re-run by the hot-reload path

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `config_watcher.rs:254-257` (reload calls `load_config` only, no
  `apply_env_overrides`); `main.rs:101` / `desktop.rs:134`
  (`apply_env_overrides` is bootstrap-only).
- Reachability: watcher reload after a config edit.
- Expected invariant: a config reloaded by the watcher goes through the same
  overlay pipeline as bootstrap.
- Observed behavior: env overlays (channel secrets, MCP path) are applied once
  at bootstrap and never again; a watcher-triggered reload discards them.
- Impact: minor — the only consumer of the reloaded config is the webhook
  emitter, and the env-overlaid fields are channels/MCP which are not in the
  webhook's domain. But the inconsistency is a latent bug the moment the
  reload path is widened (P2-03).
- Root cause: the reload path was written for hooks/webhook only and never
  needed the full bootstrap pipeline.
- Direction: if P2-03 is fixed by refreshing `app_config`, route the reload
  through the same `load_config` + `apply_env_overrides` helper used at
  bootstrap so the two paths cannot diverge.
- Regression validation: set `FEISHU_APP_ID`, edit config file, assert the
  reloaded `app_config` still has the env-overlaid value.
- Validation reports: [V02](../validations/A-CFG-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Config precedence and provider/auth resolution | yes | passed | [V01-01](../validations/A-CFG-01/V01-01.md) |
| V02 | Watcher reload scope and restart-required set | yes | passed | [V02-01](../validations/A-CFG-01/V02-01.md) |
| V03 | Workspace switch state replacement | yes | passed | [V03-01](../validations/A-CFG-01/V03-01.md) |
| V04 | Invalid / partial config handling | yes | passed | [V04-01](../validations/A-CFG-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | skipped | no historical docs in scope |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `config_watcher.rs:1-11` "hooks and webhook endpoints are reloaded live; other domains require restart" | current | `config_watcher.rs:227-278` reloads exactly that scope; V02-01 confirms |
| `config_discovery.rs:8-20` global agent file is `~/.eko/echo-agent.yaml` | stale/regressed | loader reads `~/.eko/config.yaml` (`config.rs:674`); P2-02 |
| `infra.rs:304-305` "falls back to env vars + echo-agent-models.yaml" comment | current | confirmed; the fallback is the source of P1-02 |
| A-BOOT-01: "Config watcher is spawned at startup" | current | `main.rs:231`, `desktop.rs:165`; this task adds "and never reconfigured" (P1-01) |

## Coverage And Uncertainty

- **Not executed at runtime:** all four validations are static reads; no
  process was started with a broken config or switched workspace. The
  behavior claims follow directly from the code paths cited and are
  high-confidence, but a runtime confirmation of P1-01 (watcher stale after
  switch) and P1-02 (no fast-fail) would raise them from high to confirmed.
- **Framework config struct internals:** `AppConfig` field-by-field
  serialization (e.g. exactly which fields `serde(default)`) was not audited;
  this affects how a partial YAML interacts with defaults but not the findings
  here, which are about load/error/reload behavior.
- **GUI env-var inheritance:** the P1-02 impact claim for GUI relies on the
  comment at `infra.rs:305`; actual env-var inheritance on each target OS was
  not measured.
- **`echo-agent-models.yaml` fallback:** the framework's
  `echo-agent-models.yaml` resolution (mentioned in `infra.rs:305`) was not
  traced; it is a separate framework path and out of scope for this task.
- **Web-frontend config views:** how the React UI renders config / configured
  models was not inspected; only the Rust projection
  (`configured_model_views`) was read.

## Handoff

Conclusions downstream tasks may rely on:

- Config loading is **single-file, first-existing-wins, no merge**. Any task
  that assumes global+project layering must account for P2-01 before relying on
  it.
- The authoritative global config filename is `config.yaml` (loader),
  **not** `echo-agent.yaml` (discovery). Downstream docs/UI should use the
  loader's name until P2-02 is resolved.
- The hot-reload boundary is exactly {user hooks, ConfigChange hook, webhook
  endpoints}. `AppState.app_config` is stale after the first post-bootstrap
  edit. Any task adding IPC that reads `app_config` for live behavior must
  either fix P2-03 or document the restart requirement.
- Workspace switch is storage/memory/skills isolation only; config and hooks
  do **not** follow the workspace (P1-01, P2-04).
- Missing API key does not block bootstrap (P1-02) — onboarding / first-run
  UX tasks should consume this as a prerequisite.

Reports downstream tasks must read:

- `zcode-glm/tasks/A-BOOT-01.md` (read; provides the composition-root context).
- This report's V01–V04 for precedence / watcher / switch / error-handling
  detail.

Conditions that make this report stale:

- Any change to `load_config` / `config_search_paths` (P2-01, P2-02, P2-05).
- Any change to `spawn_config_watcher` or `handle_config_change` (P1-01 in
  part, P2-03, P3-02).
- Any change to `switch_workspace` (P1-01, P2-04).
- Adding a fast-fail gate in `create_agent_with_diagnostics` (P1-02).

Follow-up task IDs (no fixes implemented in this review):

- Recommend a dedicated task for the watcher ↔ workspace integration (P1-01)
  and the fast-fail API-key gate (P1-02) — both are behavioral changes worth
  scoping separately.
- P2-* items can batch into a "config coherence" cleanup task.
- P3-* items are documentation/help-text only and can ride along with any
  nearby CLI change.
