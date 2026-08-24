# A-CFG-01: Configuration, providers, and workspace lifecycle

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; only Codex review reports
> were added

## Question

Are EKO global/project config discovery and precedence, provider selection,
workspace switching, validation, persistence, and hot-reload scope coherent?

## Scope

- Framework `AppConfig`, model/provider definitions, config path selection,
  parsing, environment overrides, and persistence.
- EKO config discovery, configured-model resolution, startup/provider callers,
  Tauri config/provider/workspace commands, and frontend config save behavior.
- Workspace current-root, process cwd, persistence, conversation/runtime-state
  stores, memory, primary Agent, existing pool Agents, and future pool Agents.
- Config watcher restart-required/live domains, invalid-edit behavior, secret
  file permissions and IPC/log projection.
- CLI-side SQLite absence.

## Out Of Scope

- Initial `--project` versus execution-root mismatch, already owned by
  `A-BOOT-01-P1-01`.
- Composition/shutdown failures already owned by `A-BOOT-01`.
- Conversation restore correctness after the correct store has been selected
  (`A-STATE-01`).
- Instruction and long-term-memory content semantics (`A-MEM-01`).
- Full per-surface command parity (`A-SRF-01` through `A-SRF-04`).
- Provider protocol/client correctness inside the independent framework and
  real paid-network calls.
- Implementation or source mutation.

## Inputs

- Root `AGENTS.md`, including local-product security, mode parity, framework
  independence, no-SQLite, and layering constraints.
- Shared `README.md`, `REPORTING.md`, and the `A-CFG-01` card in `TASKS.md`.
- Codex isolation rules in `docs/comprehensive-review/codex/README.md`.
- Primary-complete Codex dependency report [`A-BOOT-01`](./A-BOOT-01.md), with
  `A-BOOT-01-P1-01` treated as current rather than copied.
- No other reviewer directory or report was read.

## Layering Decision

Parsing a typed config file, returning its source identity, owner-only file
permissions, and explicit save-to-source are generic mechanisms that an
unrelated `echo-agent` consumer could use. Which global/project scopes compose,
which file names EKO advertises, provider environment aliases, workspace
transitions, and which domains reload live are EKO product policy and belong in
`echo-agent-cli`. The adapter boundary should pass one resolved config artifact
and canonical working root into framework constructors; it must not independently
rediscover either one.

Duplicate search covered names and behaviors across both repositories:
`AppConfig`, `ConfigDiscovery`, `config_search_paths`, `load_config`,
`load_config_file`, `save_config`, `resolve_runtime_model`, provider apply and
delete commands, `switch_workspace`, `exit_workspace`, pool resource overrides,
config watcher reload, and every CLI/Tauri caller. There is no existing retained
effective-config-source object and no pool conversation/runtime-store override.
If authority is unified, delete EKO's independent agent/MCP path inventory and
stateless re-search on save; do not keep both as compatibility mechanisms.

## Current Path

### App config and precedence

At both real headless and desktop composition roots, EKO calls
`load_config(args.config)` and then channel/MCP-only `apply_env_overrides`
(`src/main.rs:99-101`, `src/tauri/desktop.rs:132-134`). An explicit `--config`
is parsed alone. Without it, framework search is
`ECHO_AGENT_CONFIG` -> cwd `echo-agent.yaml` -> user-data `config.yaml`, and the
first parseable file replaces the whole `AppConfig`; there is no global/project
merge (`echo-agent/src/config.rs:666-756`). Thus a partial project file gets
serde defaults for absent values rather than inheriting the user's global
provider credential (V10).

`ConfigDiscovery`, reachable only through registered GUI `discover_config`,
does not drive startup. It advertises user-data `echo-agent.yaml` and project
`.mcp.json` (`config_discovery.rs:219-265`), while startup app config uses
user-data `config.yaml` and EKO's MCP loader defaults only to user-data
`mcp.json` (`infra.rs:1069-1095`). `--project` is not a config-selection input;
that initial canonical-root defect remains `A-BOOT-01-P1-01`.

### Persistence and provider selection

The loaded source path is discarded. Every GUI config/provider mutation calls
framework `save_config`, which searches the current environment/cwd again and
overwrites the first existing candidate (`config.rs:687-721`; provider callers
at `providers.rs:202,226,243`; full config at `config.rs:270-275`). Workspace
switch changes process cwd (`state.rs:851-870`), so the target can change during
one process. V08 created a global file instead of updating an explicit source;
V09 proved an existing workspace `echo-agent.yaml` is overwritten.

Runtime model resolution selects requested id -> configured default id -> first
enabled entry -> legacy model, then resolves credentials provider config ->
legacy same-provider token -> provider-specific environment key
(`model_config.rs:237-317`). Startup uses this path; TUI model selection and
Tauri `set_default_model` update primary plus pool. Tauri deletion mutates the
default in config but omits the same runtime-apply step (`providers.rs:218-248`).
GUI responses expose token presence/source, not token values, and saved config
is chmod 0600 on Unix (V12). Imported legacy `EKO_AUTH_TOKEN`, `EKO_BASE_URL`,
and `EKO_MODEL` variables have no consumer.

### Workspace and reload

`switch_workspace` commits `workspace.current` first, treats invalid root/cwd
failure as warnings, changes primary/pool working dirs and memory, then changes
other stores piecemeal (`state.rs:844-1031`). Conversation/runtime-store swaps
on the primary use non-blocking `AgentHandle::try_write`, so lock contention
silently skips them (`state.rs:902-934`; framework `handle.rs:264-272`). Pool
`SharedResources` retains bootstrap conversation/state stores and injects them
into future Agents (`agent_pool.rs:94-159,824-903`). `exit_workspace` neither
restores process cwd nor installs the new global conversation store into the
primary (`state.rs:1053-1184`).

The watcher intentionally reloads only hooks and webhooks; model, MCP, and
runtime limits require restart (`config_watcher.rs:1-11,249-277`). Invalid hook
input preserves prior hooks, but invalid app YAML becomes default config and
therefore clears webhook endpoints (V19). The general full-config GUI endpoint
does not expose this live/restart matrix. It warns and returns success on save
failure, hot-applies only a subset to the primary, and the frontend says
`已保存` (`tauri/commands/config.rs:163-307`, `ConfigPanel.tsx:37-69`).

## Findings

### A-CFG-01-P0-01: Config persistence can overwrite a different project's file

- Priority: P0
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/config.rs:666`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/config.rs:691`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/state.rs:851`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/providers.rs:202`
- Reachability: GUI config/provider mutations call `save_config`; registered
  workspace switch changes process cwd; save searches again instead of using
  the explicit file loaded at boot. V09 reproduced the overwrite through the
  same public load/save APIs.
- Expected invariant: a settings save writes atomically to the effective config
  source selected for this process, independent of navigation/cwd changes.
- Observed behavior: source identity is discarded. After loading explicit A,
  changing cwd to workspace B with an existing `echo-agent.yaml`, saving the
  in-memory A config overwrites B and leaves A unchanged.
- Impact: ordinary workspace navigation followed by a settings/provider edit
  can destroy another project's configuration and still report success.
- Root cause: config value and config-source identity are separate; persistence
  re-runs a mutable cwd/global existence search.
- Direction: retain one resolved EKO config artifact `{value, source, scope}` at
  bootstrap and use explicit, atomic save-to-source. Make navigation independent
  of config authority. Delete stateless path re-selection from EKO mutation
  callers after migration.
- Regression validation: start with explicit A, switch among B/C (with and
  without local configs), save every GUI config/provider mutation, and assert
  only A changes, with temp-write/rename and preserved 0600 permissions.
- Validation reports: [V01](../validations/A-CFG-01/V01-01.md),
  [V08-03](../validations/A-CFG-01/V08-03.md),
  [V09](../validations/A-CFG-01/V09-01.md)

### A-CFG-01-P1-01: Explicit invalid config silently starts default DeepSeek

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/config.rs:725`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/main.rs:100`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/desktop.rs:133`
- Reachability: every EKO entry uses the infallible `load_config`; explicit
  missing/malformed input is logged and replaced with `AppConfig::default`.
- Expected invariant: explicit config failure prevents startup with an
  actionable error; it never selects a different provider implicitly.
- Observed behavior: V05-02 proved strict parsing failed while startup returned
  `deepseek/deepseek-v4-flash` and exit code zero.
- Impact: a typo can launch with wrong model/settings or fail later as a vague
  missing-credential/provider error; desktop diagnostics never receive the
  actual config failure.
- Root cause: framework convenience loading collapses absence, read error, and
  parse error into a default value, and EKO uses it as its startup contract.
- Direction: EKO startup should use the fallible parse API, retain source/error,
  and apply defaults only when no source was requested/found by policy. Do not
  add an authorization gate.
- Regression validation: missing, unreadable, malformed, and valid explicit
  fixtures across headless/desktop entries with asserted exit status and path-
  specific diagnostics.
- Validation reports: [V05-02](../validations/A-CFG-01/V05-02.md),
  [V16](../validations/A-CFG-01/V16-01.md)

### A-CFG-01-P1-02: Workspace success can leave cwd and persistence stores split across roots

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/state.rs:844`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/state.rs:902`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/state.rs:1053`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:94`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:824`
- Reachability: registered GUI `switch_workspace`/delete-current-workspace call
  these methods while chat/pool Agents and background tasks are live.
- Expected invariant: one generation change atomically replaces canonical root
  and every root-scoped resource for AppState, primary, existing pool, and
  future pool Agents; failure leaves the previous generation intact.
- Observed behavior: current root commits before fallible work; cwd/store errors
  only warn; primary conversation/state swaps can silently lose `try_write`;
  pool conversation/state stores remain bootstrap values; exit leaves process
  cwd in the old workspace and leaves primary conversation store there.
- Impact: conversations/checkpoints can be written to or restored from another
  workspace while UI reports the new workspace, and post-exit relative paths
  can still target the exited workspace.
- Root cause: workspace lifecycle is a sequence of best-effort mutations with
  no prepared resource bundle, generation identity, commit point, or rollback;
  pool overrides exist only for working dir/memory.
- Direction: EKO should prepare one application-owned workspace resource bundle,
  acquire required locks, then commit primary/AppState/pool/future-pool handles
  together or roll back. Store and restore one canonical process root, or stop
  mutating process cwd once explicit working roots cover consumers. Do not move
  EKO workspace policy into the framework.
- Regression validation: inject invalid root, store-constructor failure, and a
  held Agent write lock; assert old generation on failure. On success/exit,
  assert Arc/path identity across all current and newly created Agents.
- Validation reports: [V11](../validations/A-CFG-01/V11-01.md),
  [V17](../validations/A-CFG-01/V17-01.md),
  [V18](../validations/A-CFG-01/V18-01.md)

### A-CFG-01-P1-03: Deleting the default model leaves live Agents on the deleted model

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/providers.rs:218`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/providers.rs:234`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/model_config.rs:214`
- Reachability: the registered GUI delete command calls the pure config
  deletion helper, which may select a new default, but does not call the
  runtime apply helper used by the adjacent set-default command.
- Expected invariant: persisted default, primary Agent, existing pool Agents,
  and future pool config change together.
- Observed behavior: config selects/saves a replacement while live and future
  pool runtime snapshots retain the deleted model until another switch/restart.
- Impact: settings UI and actual chats disagree about provider/model/credential;
  later pool conversations can continue using a supposedly removed model.
- Root cause: mutation and runtime propagation are separate command-local steps,
  and deletion omits the propagation half.
- Direction: route set/delete through one application model-transition
  operation that persists then applies (or reports persistence failure) to all
  Agent identities; delete branch-local partial orchestration.
- Regression validation: delete active default with primary plus existing and
  subsequently created pool conversation; assert all use replacement identity.
- Validation reports: [V04](../validations/A-CFG-01/V04-01.md),
  [V14](../validations/A-CFG-01/V14-01.md)

### A-CFG-01-P1-04: Invalid hot-reload clears last-known-good webhook endpoints

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:254`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:258`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:275`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/webhook/emitter.rs:88`
- Reachability: desktop/headless watcher invokes `handle_config_change` on
  create/modify/remove events; editor partial writes are included.
- Expected invariant: invalid reload input preserves every live domain's last-
  known-good state.
- Observed behavior: hooks preserve prior state through a fallible loader, but
  app config reload uses the default-on-error API and unconditionally replaces
  webhook endpoints with the default empty set.
- Impact: a transient malformed save silently stops configured webhook event
  delivery until a later valid event restores it.
- Root cause: two reload domains use different parse/error contracts inside one
  handler.
- Direction: parse the app source once fallibly and commit hooks/webhooks only
  from a valid prepared snapshot; retain both prior states on error.
- Regression validation: malformed partial write, atomic rename save, delete/
  recreate, and valid restoration with endpoint identity/event delivery checks.
- Validation reports: [V05-02](../validations/A-CFG-01/V05-02.md),
  [V15](../validations/A-CFG-01/V15-01.md),
  [V19](../validations/A-CFG-01/V19-01.md)

### A-CFG-01-P1-05: GUI reports durable success when config persistence fails

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/config.rs:270`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/providers.rs:202`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/providers.rs:226`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/config/ConfigPanel.tsx:68`
- Reachability: every full-config and provider write logs save failure but
  returns a success payload; frontend displays `已保存`.
- Expected invariant: success means the setting will survive restart; failed
  durability is an IPC error and runtime/config mutation is rolled back or
  explicitly reported as live-only.
- Observed behavior: in-memory cfg and sometimes runtime change before/without a
  successful save, while callers receive success.
- Impact: users lose settings at restart and may unknowingly run with a live
  model/config different from disk.
- Root cause: persistence is best-effort inside mutation commands and the
  response contract has no durability state.
- Direction: make persistence a required step in one config transaction; return
  structured failure before claiming saved and reconcile/roll back live state.
- Regression validation: read-only/unwritable target fixture for every config/
  provider mutation; assert error, no success toast, and coherent live/disk state.
- Validation reports: [V08](../validations/A-CFG-01/V08-01.md),
  [V13](../validations/A-CFG-01/V13-01.md)

### A-CFG-01-P2-01: Discovery, whole-file precedence, and documented aliases describe different config systems

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/config.rs:666`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/config_discovery.rs:219`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/config_discovery.rs:243`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/infra.rs:1069`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/infra.rs:1455`
- Reachability: registered GUI discovery displays the independent inventory;
  startup and MCP runtime use different resolvers; desktop imports documented
  `EKO_*` aliases which no later code reads.
- Expected invariant: one explicit precedence table drives discovery and
  runtime, with project overrides either merged field-wise or clearly declared
  whole-file replacements; documented aliases are reachable.
- Observed behavior: discovery lists non-runtime global app/project MCP paths;
  partial project config wholesale replaces global values (V10); three EKO
  provider aliases are imported but ignored.
- Impact: diagnostics can identify the wrong file as active, and adding a small
  project override silently drops a working global provider credential.
- Root cause: inventory, runtime loading, and compatibility env handling evolved
  independently without a single resolved-config model.
- Direction: define EKO's global/project composition policy once and derive
  discovery, startup, watcher, and persistence from it. Remove dead aliases or
  wire them at the documented precedence; delete duplicate inventory rules.
- Regression validation: table-driven global/project/env/explicit app and MCP
  fixtures that assert value, active source(s), discovery display, and save target.
- Validation reports: [V02](../validations/A-CFG-01/V02-01.md),
  [V03](../validations/A-CFG-01/V03-01.md),
  [V10](../validations/A-CFG-01/V10-01.md)

### A-CFG-01-P2-02: Full-config save has no truthful live-versus-restart contract

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/config.rs:163`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/config.rs:278`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:7`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/config/ConfigPanel.tsx:44`
- Reachability: the GUI configuration panel can mutate model limits, Agent
  fields, MCP, channels, server, and logging in one request.
- Expected invariant: each field is classified live or restart-required;
  claimed live changes reach primary, existing pool, and future pool Agents.
- Observed behavior: backend mutates all in-memory fields but hot-applies only
  primary temperature/max-tokens/system prompt. The frontend's follow-up for
  `max_iterations` sends a field not present in Rust `UpdateConfigRequest`, so
  it is ignored; pool changes are omitted. No restart metadata is returned.
- Impact: current chat behavior differs from the settings response, and users
  cannot know whether restarting is required.
- Root cause: persistence schema is reused as a runtime-control API without a
  per-field application owner or effect contract.
- Direction: publish one field matrix (`live`, `restart_required`, or rejected)
  from the application service. Apply live fields through shared primary/pool
  operations; return restart requirements for constructed services.
- Regression validation: field-by-field GUI request matrix asserting disk,
  response metadata, primary/pool/future-pool value, and post-restart behavior.
- Validation reports: [V13](../validations/A-CFG-01/V13-01.md),
  [V15](../validations/A-CFG-01/V15-01.md)

### A-CFG-01-P2-03: Unknown config keys are accepted and silently discarded

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/config.rs:68`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/config.rs:295`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/config.rs:681`
- Reachability: all YAML sources deserialize these types directly; no startup
  validation pass checks unconsumed keys.
- Expected invariant: likely typos in operational/credential fields are
  rejected or surfaced with field path before runtime.
- Observed behavior: V07's `auth_tokne` fixture parsed successfully and simply
  produced no credential.
- Impact: users receive later provider/auth failures disconnected from the
  actual typo; unknown settings can appear saved but never take effect.
- Root cause: pervasive `serde(default)` is not paired with unknown-field
  diagnostics or a schema validation layer.
- Direction: add fallible unknown-key diagnostics at the generic parser or an
  EKO strict adapter while retaining defaults for omitted known fields. This is
  input correctness, not local permission gating.
- Regression validation: typo fixtures at each nested config section plus valid
  partial fixtures proving omission defaults still work.
- Validation reports: [V06](../validations/A-CFG-01/V06-01.md),
  [V07](../validations/A-CFG-01/V07-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate-authority search | yes | passed | [V01](../validations/A-CFG-01/V01-01.md) |
| V02 | Global/project/runtime/discovery path map | yes | failed -> finding | [V02](../validations/A-CFG-01/V02-01.md) |
| V03 | Provider and secret/log reachability | yes | passed with deviation | [V03](../validations/A-CFG-01/V03-01.md) |
| V04 | Default-model deletion runtime propagation | yes | failed -> finding | [V04](../validations/A-CFG-01/V04-01.md) |
| V05 | Invalid explicit fixture, attempts 01/02 | yes | env failure then behavior failure | [01](../validations/A-CFG-01/V05-01.md), [02](../validations/A-CFG-01/V05-02.md) |
| V06 | Valid partial fixture | yes | passed | [V06](../validations/A-CFG-01/V06-01.md) |
| V07 | Unknown-field fixture | yes | failed -> finding | [V07](../validations/A-CFG-01/V07-01.md) |
| V08 | Explicit-source/cwd save, attempts 01/02/03 | yes | env/insufficient then failed | [01](../validations/A-CFG-01/V08-01.md), [02](../validations/A-CFG-01/V08-02.md), [03](../validations/A-CFG-01/V08-03.md) |
| V09 | Existing workspace file overwrite fixture | yes | failed -> finding | [V09](../validations/A-CFG-01/V09-01.md) |
| V10 | Layered global/project precedence fixture | yes | failed -> finding | [V10](../validations/A-CFG-01/V10-01.md) |
| V11 | Workspace switch/exit resource identity | yes | failed -> finding | [V11](../validations/A-CFG-01/V11-01.md) |
| V12 | Secret permission/projection, attempts 01/02 | yes | probe error then passed | [01](../validations/A-CFG-01/V12-01.md), [02](../validations/A-CFG-01/V12-02.md) |
| V13 | Restart-required versus live field matrix | yes | failed -> findings | [V13](../validations/A-CFG-01/V13-01.md) |
| V14 | `cargo test -p echo-agent-app-core model_config --locked` | yes | passed | [V14](../validations/A-CFG-01/V14-01.md) |
| V15 | `cargo test -p echo-agent-app-core config_watcher --locked` | yes | passed | [V15](../validations/A-CFG-01/V15-01.md) |
| V16 | `cargo test -p echo_agent config::tests --locked` | yes | passed | [V16](../validations/A-CFG-01/V16-01.md) |
| V17 | A-BOOT dependency and finding de-duplication | yes | passed | [V17](../validations/A-CFG-01/V17-01.md) |
| V18 | CLI no-SQLite constraint | yes | passed | [V18](../validations/A-CFG-01/V18-01.md) |
| V19 | Invalid reload preserves hooks and webhooks | yes | failed -> finding | [V19](../validations/A-CFG-01/V19-01.md) |
| V20 | Primary source/behavior sampling and report gate | primary gate | passed after preserved path/session/ID failures and exact correction allowlist | [01](../validations/A-CFG-01/V20-01.md), [02](../validations/A-CFG-01/V20-02.md), [03](../validations/A-CFG-01/V20-03.md), [04](../validations/A-CFG-01/V20-04.md), [05](../validations/A-CFG-01/V20-05.md), [06](../validations/A-CFG-01/V20-06.md), [07](../validations/A-CFG-01/V20-07.md), [08](../validations/A-CFG-01/V20-08.md), [09](../validations/A-CFG-01/V20-09.md), [10](../validations/A-CFG-01/V20-10.md), [11](../validations/A-CFG-01/V20-11.md) |
| V21 | Subagent report/isolation preflight, attempts 01/02/03 | yes | harness error then passed | [01](../validations/A-CFG-01/V21-01.md), [02](../validations/A-CFG-01/V21-02.md), [03](../validations/A-CFG-01/V21-03.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `A-BOOT-01-P1-01`: `--project` is not the execution root | current | [V17](../validations/A-CFG-01/V17-01.md) |
| A-BOOT handoff: A-CFG must define one canonical root and restart-required config | current and deepened | [V11](../validations/A-CFG-01/V11-01.md), [V13](../validations/A-CFG-01/V13-01.md) |
| Watcher docs: hooks/webhooks live; models/MCP/runtime limits restart | current as intent, incomplete as behavior | [V13](../validations/A-CFG-01/V13-01.md), [V19](../validations/A-CFG-01/V19-01.md) |
| Config save docs: first existing or first current path | stale/internally contradictory when explicit startup args are used | [V08-03](../validations/A-CFG-01/V08-03.md), [V09](../validations/A-CFG-01/V09-01.md) |

## Coverage And Uncertainty

- No paid provider network call was made; provider reachability stops at client
  construction/config application. This avoids placing real secrets in logs.
- The full Tauri application was not launched. Registered command and frontend
  reachability was proven statically; public config behavior was replayed in
  isolated executable fixtures.
- Workspace transition was not dynamically fault-injected because constructing
  full AppState starts broad runtime services; exact mutation/lock/resource
  identity is directly visible and currently has no targeted tests.
- Unix mode was executed; Windows ACL behavior was not tested.
- Three targeted Cargo commands passed. This review did not run submission
  gates because source code was not changed.
- Primary independently sampled all finding paths and rebuilt executable
  evidence for P0-01 and P1-01. Full AppState workspace transition still lacks
  an isolated fault-injection seam; V20-09 confirms the exact lock/resource
  branches statically.

## Handoff

- `A-SRF-01`/`A-SRF-02` should consume P1-03, P1-05, and P2-02 for GUI/TUI
  settings/model behavior.
- `A-STATE-01` should consume P1-02's conversation/runtime-store identity facts;
  `A-MEM-01` should consume only the workspace generation/root facts, not
  duplicate the memory-content review.
- `Q-E2E-01` should own the multi-surface invalid-config, save-target, workspace
  fault, model-delete, invalid-reload, and restart/live executable matrix.
- This report becomes stale when config source/path selection, provider command
  orchestration, workspace resource propagation, or watcher reload changes.
