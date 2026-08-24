# F-PLG-01: Plugin manifest, registry, and lifecycle

> Status: complete
> Reviewer: Codex primary reviewer
> Review date: 2026-08-12
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; Codex reports only

## Question

Does the plugin framework resolve dependencies, component ownership,
activation, replacement, unloading, and rollback without leaked registrations?

## Scope

- `echo-core` plugin manifest, scopes, variables, registry, dependency graph,
  persistence and lifecycle manager.
- Root facade PluginIntegrator, public targeted helpers and legacy NativePlugin.
- Framework Skill/Hook/MCP component registration and unload identity.
- Narrow EKO PluginRuntimeService trace only to prove production reachability,
  framework/application ownership and whole-set compensation.
- Root exports, bilingual plugin documentation, examples and existing tests.

## Out Of Scope

- Skill activation internals owned by F-SKL-01, generic Tool collisions owned by
  F-EXT-01 and MCP transport correctness owned by F-INT-01.
- EKO plugin UI/preferences/product policy owned by A-PLG-01.
- Source changes, Cargo/rustc/build/test execution, fixtures and network access.

## Inputs

- Root AGENTS; shared README, REPORTING and exact F-PLG-01 task card; Codex
  reviewer rules.
- Completed Codex dependencies B-REF-01 and F-SKL-01.
- Current committed source. The framework HEAD advanced during review from
  `9b0e0faf` to `3aa79299`; V00-02 proves the commit did not change plugin scope.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Manifest/schema validation, stable source identity, dependency ordering/version checks, component ownership receipts, reversible registration, lifecycle state and unload are reusable framework responsibilities. |
| EKO product policy | Which local roots are enabled, Subagent construction, scheduler monitors, LSP process policy, active theme/output style and UI projections remain EKO concerns. Local extensions are user-trusted; fixes below prevent framework inconsistency/data loss and do not add cloud-style permission gates. |
| Adapter boundary | EKO prepares product-owned files and brackets them around a framework component transaction. It must not own a second plugin dependency graph or Skill/MCP ownership registry. |
| Duplicate search | Searched both repositories for PluginManifest/Registry/Integrator/Lifecycle, scope/source/config, dependency, wire/unwire/reload, Skill/Hook/MCP ownership, EKO callers, exports, examples, docs and tests. |
| Migration deletion | Strengthen the existing registry/integrator/lifecycle. Do not add another plugin host. Remove lossy helper/legacy surfaces after callers use one canonical source-owned transaction. |

## Current Path

```text
PluginScope::all (User -> Project -> Local)
  -> scan_scope_dir -> HashMap<manifest.name, PluginEntry>
  -> load_state by same name -> enabled/config
  -> resolve_enabled_dependencies -> ordered plugin names
  -> PluginIntegrator::wire_all
       -> source-tagged Skills
       -> HookSource::Plugin hooks
       -> MCP manager keyed by server name
       -> application-owned file outputs

EKO PluginRuntimeService
  -> scan candidate -> prepare product components
  -> deactivate callbacks
  -> unload old set -> wire candidate
  -> register Subagents/LSP/monitors/theme/style
  -> on failure unload candidate and try to restore old set
```

Positive conclusions:

- Dependency resolution enforces enabled dependencies, semantic version
  constraints and cycles with deterministic topological output.
- Manifest names/config/path traversal receive validation; strict manual plugin
  validation resolves all declared paths.
- Skill batches receive plugin source tags and hooks use HookSource::Plugin.
- EKO uses one shared PluginRuntimeService across its interaction modes and
  correctly retains product-only construction outside the framework.
- The plugin capability is a reasonable reusable public framework surface; its
  existence must not be judged by whether EKO uses every component option.

## Findings

### F-PLG-01-P1-01: Scope-blind plugin identity silently replaces sources and rebinds persisted configuration

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/plugin/registry.rs:13`, `:126`, `:189`,
  `:217`, `:235`, `:949`; `echo-agent/echo-core/src/plugin/scope.rs:52`
- Reachability: public scan_all and the live EKO registry scan on every reload or
  mutation traverse all three plugin scopes.
- Expected invariant: one stable plugin identity identifies its source/scope and
  persisted config; an explicit deterministic alias/override policy may choose
  a visible winner without transferring another source's state.
- Observed behavior: PluginId is documented as optionally scoped but the map key
  is only manifest name. User, Project and Local are scanned in order and later
  entries silently replace earlier entries; same-scope read_dir order is also
  unsorted. Persisted enabled/config values are then merged by name into the
  winner regardless of root, scope, version or manifest identity. Discovery
  count includes overwritten entries while list/count expose only the winner.
- Impact: a workspace/local plugin can receive another plugin's configuration
  and enabled state; which same-scope plugin owns a name can vary by filesystem,
  and status/counts do not describe the live source truthfully.
- Root cause: human name simultaneously serves as stable identity, scope
  precedence and persistence key.
- Direction: introduce source-qualified stable IDs and an explicit sorted
  precedence alias; bind config to source/manifest version and return typed
  collision/override information. Keep local override capability without adding
  permission approval gates.
- Regression validation: duplicate name in every scope/order, changed version/
  config schema, restart and reported discovery/live counts.
- Validation reports: [V03](../validations/F-PLG-01/V03-01.md)

### F-PLG-01-P1-02: Four manifest array fields silently discard every component after the first

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/plugin/manifest.rs:112`, `:128`, `:132`,
  `:136`, `:140`, `:166`; `echo-agent/echo-core/src/plugin/registry.rs:686`,
  `:698`, `:716`, `:728`
- Reachability: public YAML manifest accepts arrays for hooks, MCP servers, LSP
  servers and monitors; resolve_components is used by framework integration and
  EKO preparation.
- Expected invariant: every accepted declaration is resolved and activated, or
  unsupported cardinality is rejected during validation.
- Observed behavior: all eight component fields deserialize StringOrArray and
  validation visits every value. Skills/agents/themes/styles iterate all paths,
  but hooks/MCP/LSP/monitors call `first()` and the resolved type has only one
  Option slot. Remaining valid paths vanish without an error or warning.
- Impact: authored hooks, servers and monitors are absent while validation and
  installation report success, producing incomplete extension behavior that is
  hard to diagnose.
- Root cause: a shared flexible manifest type was paired with singular resolved
  fields without a cardinality invariant.
- Direction: either represent and aggregate every path with explicit collision
  rules, or make singular fields accept only one string and reject arrays.
- Regression validation: zero/one/two paths for all eight families, missing
  second path and cross-file duplicate identities.
- Validation reports: [V02](../validations/F-PLG-01/V02-01.md)

### F-PLG-01-P1-03: Install and discovery can commit enabled plugins before declared components are valid

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/plugin/registry.rs:141`, `:202`, `:275`,
  `:290`, `:313`, `:316`, `:328`, `:636`, `:1008`
- Reachability: direct PluginRegistry install/scan is public and documented; EKO
  calls install before apply_candidate performs deferred component preparation.
- Expected invariant: copy, strict component resolution, dependency validation
  and state persistence form one staged commit; failure leaves no installed path
  or enabled registry entry.
- Observed behavior: strict validate_plugin_dir exists but install_local uses
  only manifest validation, copies the tree, inserts/persists an enabled entry
  and never resolves declared paths. Scan similarly defers path validation until
  wiring. A recursive copy failure returns before cleanup and can leave a
  destination that blocks every later install as “already installed”.
- Impact: framework callers can successfully install/enable an unusable plugin;
  a transient copy failure can require manual filesystem recovery. EKO catches
  some later apply failures but cannot make the public registry contract true.
- Root cause: validation, filesystem copy, registry state and runtime activation
  are separate commits rather than one prepare/commit sequence.
- Direction: copy to a unique staging directory, strictly resolve the staged
  destination, validate dependencies, atomically rename/persist, and clean every
  failure path. Reuse validate_plugin_dir rather than adding another validator.
- Regression validation: missing declared path, partial copy, dependency/state
  failure, stale staging directory and retry.
- Validation reports: [V02](../validations/F-PLG-01/V02-01.md)

### F-PLG-01-P1-04: Public wire_all returns errors after leaving a partially live component set

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/plugin.rs:128`, `:259`, `:287`, `:310`, `:348`,
  `:386`; `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:804`
- Reachability: wire_all is the documented primary framework integration API;
  EKO and external consumers call it against a mutable live ReactAgent.
- Expected invariant: integration prepares all components, rejects conflicts,
  then commits once or returns a reversible receipt that restores the prior
  generation.
- Observed behavior: components register sequentially and failures accumulate.
  A later Skill/Hook/MCP failure leaves earlier registrations/connections live;
  missing servers in a multi-server config leave the successful subset live.
  PluginWiringResult records partial receipts but PluginIntegrator offers no
  matching transaction/unwire operation. EKO implements its own whole-set
  compensation, while direct framework callers keep the partial generation.
- Impact: a failed plugin can still alter prompts, tools, hooks or external
  processes, and a retry can accumulate/reassign registrations.
- Root cause: reporting and mutation are interleaved; result receipts are
  observational rather than an owned commit/rollback handle.
- Direction: split prepare/validate from commit and return an integrator-owned,
  reversible generation receipt. EKO coordinates product components around that
  generic transaction rather than duplicating framework ownership logic.
- Regression validation: inject a failure after every component stage and assert
  exact old-or-new Agent state plus no leaked process/registration.
- Validation reports: [V04](../validations/F-PLG-01/V04-01.md),
  [V06](../validations/F-PLG-01/V06-01.md)

### F-PLG-01-P1-05: MCP server-name replacement breaks per-plugin ownership receipts

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/plugin.rs:330`, `:338`, `:342`;
  `echo-agent/echo-integration/src/mcp/mod.rs:56`, `:72`, `:76`, `:95`;
  `echo-agent/src/agent/react/capabilities.rs:1153`, `:1157`, `:1315`;
  `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:1165`
- Reachability: any two enabled plugin MCP files may use the same ordinary server
  name; wire_all loads them in dependency order and EKO unloads using receipts.
- Expected invariant: connection identity includes owner/generation or a
  duplicate is rejected before mutation; only the owner can unload it.
- Observed behavior: McpManager is keyed only by server name and connect first
  disconnects/replaces an existing same-name server and its tools. Integrator
  records the same name in each plugin's component receipt. The earlier receipt
  now points to the later live server; unloading either plugin disconnects it.
- Impact: enabling, disabling or failing one plugin can silently replace/remove
  another plugin's tools and connection while both receipts claim ownership.
- Root cause: transport display name is used as process identity and ownership
  key; no collision preflight or connection generation exists.
- Direction: namespace by plugin owner plus declared server ID, or reject a
  global collision before connect; receipts must carry an opaque connection
  instance/generation, not a reusable String.
- Regression validation: two plugins with same server/tool names, reload in
  opposite order, unload either owner and partial connection failure.
- Validation reports: [V04](../validations/F-PLG-01/V04-01.md)

### F-PLG-01-P1-06: Corrupt registry state silently falls back to manifest defaults and can re-enable plugins

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/plugin/registry.rs:202`, `:217`, `:220`,
  `:932`, `:934`, `:937`, `:943`
- Reachability: every scan_all/live EKO reload first reconstructs entries from
  manifests and then calls load_state.
- Expected invariant: corrupt durable state is surfaced and recovered from a
  last-good copy or leaves affected plugins disabled until explicit resolution.
- Observed behavior: invalid JSON only logs a warning and returns. Manifest
  `default_enabled` values remain in memory, so plugins the user previously
  disabled can become active after a truncated/corrupt registry file.
- Impact: local extension capabilities can unexpectedly return after state
  corruption; the user sees neither an authoritative recovery error nor the
  prior disabled intent.
- Root cause: discovered defaults are treated as recovery state and persistence
  corruption is not part of the registry API result.
- Direction: make load/scan return a typed corruption outcome, keep a last-good
  atomic backup or fail closed for prior installations, and require explicit
  reset. This is data/lifecycle safety, not a cloud permission gate.
- Regression validation: truncated/type-invalid state, crash around rename,
  last-good restore and explicit reset.
- Validation reports: [V03](../validations/F-PLG-01/V03-01.md)

### F-PLG-01-P1-07: Uninstall can remove files and registry ownership but skip lifecycle cleanup

- Priority: P1
- Confidence: high
- Layer: framework/application boundary
- Evidence: `echo-agent/echo-core/src/plugin/registry.rs:411`, `:419`, `:427`,
  `:429`, `:437`; `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:287`,
  `:328`, `:331`, `:333`
- Reachability: EKO and direct framework consumers call uninstall for installed
  plugins, including plugins with registered native lifecycle callbacks.
- Expected invariant: uninstall returns one truthful terminal outcome and
  retains cleanup ownership until files, durable state and callbacks settle.
- Observed behavior: registry uninstall deletes plugin files, removes the
  in-memory entry, ignores data-directory removal errors, then attempts state
  save. If save fails it returns Err after destructive mutations. EKO propagates
  that error before `lifecycle.unregister`, leaving callbacks registered while
  plugin files and live registry ownership are gone.
- Impact: callers receive “failed” but cannot safely retry; callbacks/resources
  can outlive their code/data and future registration can collide with stale
  ownership.
- Root cause: destructive operations and lifecycle cleanup are ordered behind a
  fallible persistence boundary without a partial-outcome state.
- Direction: stage/persist the removal intent, settle runtime cleanup, then
  atomically remove files/state; on partial failure return a typed recoverable
  tombstone and retain cleanup handle.
- Regression validation: state-save, plugin-root delete, data delete and callback
  cleanup failures at every boundary plus restart/retry.
- Validation reports: [V05](../validations/F-PLG-01/V05-01.md)

### F-PLG-01-P2-08: Callback errors are represented as clean pre-transition states despite possible partial effects

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/plugin/lifecycle.rs:97`, `:103`, `:110`,
  `:120`, `:126`, `:136`, `:149`
- Reachability: any public PluginLifecycle callback may start/stop processes,
  connections or caches and then return Err; EKO drives these callbacks during
  activation/replacement.
- Expected invariant: a failed side-effecting transition is explicitly
  attempted/indeterminate and owns a defined compensation/retry path.
- Observed behavior: initialized/active flags change only after Ok. A callback
  that partially mutates then returns Err is reported as never initialized,
  never activated or still fully active. Retry can repeat setup; restore may skip
  needed activation; unregister removes ownership even when cleanup fails.
- Impact: duplicate or leaked local processes/resources and misleading lifecycle
  state after ordinary callback errors.
- Root cause: two booleans encode successful states but not transition attempts
  or cleanup debt.
- Direction: define a small lifecycle transition/outcome model with explicit
  compensation requirements and retained cleanup ownership. Do not add product
  Plan/approval states.
- Regression validation: callbacks mutate then fail for every transition,
  repeated reconcile, unregister and Drop.
- Validation reports: [V05](../validations/F-PLG-01/V05-01.md)

### F-PLG-01-P2-09: Targeted PluginIntegrator helpers bypass source ownership and suppress errors

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/plugin.rs:397`, `:404`, `:412`, `:423`, `:428`,
  `:436`; `echo-agent/docs/en/32-plugin-system.md:447`
- Reachability: bilingual public docs recommend wire_skills/wire_hooks/wire_mcp
  for selective integration even though production uses wire_all.
- Expected invariant: selective helpers are thin adapters to the canonical
  source-owned transaction and return typed errors plus reversible receipts.
- Observed behavior: wire_skills uses generic directory loading without plugin
  source tags and ignores Err; wire_hooks ignores its registration result;
  wire_mcp ignores file/connection failures and returns only reusable names.
- Impact: external framework consumers can create components that cannot be
  precisely unloaded and can mistake partial/no-op integration for success.
- Root cause: convenience APIs predate source ownership and were left public
  beside the canonical path.
- Direction: delete/privatize them after migrating docs, or accept plugin owner
  and return the same typed transaction receipt as wire_all.
- Regression validation: selective load error/collision/unload for every family.
- Validation reports: [V07](../validations/F-PLG-01/V07-01.md)

### F-PLG-01-P2-10: NativePlugin is a documented lifecycle API with no framework host

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/plugin.rs:453`, `:460`;
  `echo-agent/docs/en/32-plugin-system.md:784`;
  `echo-agent/docs/zh/32-plugin-system.md:771`
- Reachability: the trait is exported and public guides tell external consumers
  to implement it for custom Rust logic.
- Expected invariant: a documented extension trait has a registry/host that
  owns it, invokes init/shutdown and exposes its capabilities, or is absent.
- Observed behavior: repository-wide search finds only the trait and guide
  example. PluginRegistry/Integrator/LifecycleManager never store a
  NativePlugin; its init/shutdown and capability list cannot become live.
- Impact: consumers implement an API that compiles but provides no integration
  or lifecycle behavior, while a separate PluginLifecycle contract is real.
- Root cause: a compatibility stub survived after file plugins/lifecycle became
  canonical. The project explicitly does not require backward compatibility.
- Direction: delete the trait and stale guide or implement a real host through
  PluginLifecycleManager and the canonical component transaction. Do not keep a
  second lifecycle authority.
- Regression validation: compile/documentation contract for the chosen single
  native extension path and exact once init/activate/deactivate/shutdown.
- Validation reports: [V07](../validations/F-PLG-01/V07-01.md)

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Start baseline | yes | passed | [01](../validations/F-PLG-01/V00-01.md) |
| V00 | Commit transition/isolation | yes | passed | [02](../validations/F-PLG-01/V00-02.md) |
| V01 | Definition, duplicate and layering map | yes | passed | [report](../validations/F-PLG-01/V01-01.md) |
| V02 | Manifest cardinality and install validation | yes | failed | [report](../validations/F-PLG-01/V02-01.md) |
| V03 | Scope identity and persistence | yes | failed | [report](../validations/F-PLG-01/V03-01.md) |
| V04 | Wiring transaction and MCP ownership | yes | failed | [report](../validations/F-PLG-01/V04-01.md) |
| V05 | Uninstall and lifecycle terminal behavior | yes | failed | [report](../validations/F-PLG-01/V05-01.md) |
| V06 | Public reachability/layering/positive paths | yes | passed | [report](../validations/F-PLG-01/V06-01.md) |
| V07 | Targeted helpers and NativePlugin | yes | failed | [report](../validations/F-PLG-01/V07-01.md) |
| V08 | Existing test coverage inventory | yes | passed | [report](../validations/F-PLG-01/V08-01.md) |
| V09 | Dynamic regression matrix | no by user direction | not_run | [report](../validations/F-PLG-01/V09-01.md) |
| V10 | Final report/integrity gate | yes | 01 inconclusive; 02 passed | [01](../validations/F-PLG-01/V10-01.md), [02](../validations/F-PLG-01/V10-02.md) |

## Coverage And Uncertainty

- No Cargo, rustc, build, test, fixture or network command was run. Static
  branch/type/ownership evidence is conclusive for the reported contracts;
  runtime failure injection remains V09 future work.
- Component payload semantics are delegated to their owning reviews. This task
  only follows source identity, mutation, rollback and lifecycle boundaries.
- EKO's current compensation is positive but best effort; A-PLG-01 may find
  product projection issues without duplicating the framework findings here.
- The mid-review commit transition was independently path-scoped and plugin
  sources did not change. Current source repositories are clean.

## Handoff

- Implement stable source identity before fixing individual collision symptoms;
  receipts, persistence and lifecycle should use that identity end to end.
- Converge on one prepare/commit/reversible component transaction, then delete
  lossy targeted helpers and the unhosted NativePlugin surface.
- Preserve EKO's application-owned LSP/monitor/theme/style policy and the local
  trusted-extension threat model. These defects do not justify automated-action
  permission gates for user-installed plugins.
- F-PLG-01 is primary-complete. A-PLG-01 should consume its identity/transaction
  contracts and review only EKO-specific policy and mode projections.
