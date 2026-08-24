# A-SUB-01: EKO Subagent catalog, pool, and prompt compilation

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Does EKO add domain definitions and product policy while reusing one
framework Subagent lifecycle and immutable effective catalog?

## Scope

Primary source paths inspected (deep read):

- `echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs` (full) —
  `.md` definition discovery/precedence, `SubagentCatalogSnapshot`
  (`from_definitions` vs `from_registered`), `subagent_isolation`.
- `echo-agent-cli/echo-agent-app-core/src/subagent_prompt.rs` (full) —
  `EkoSubagentPromptCompiler`, `compile_system`/`compile_invocation`/
  `compile_primary_invocation`, `EkoPromptPayload`, section cardinality
  tests.
- `echo-agent-cli/echo-agent-app-core/src/infra.rs:194-525, 545-1042` —
  `create_agent_with_diagnostics` (catalog injection + route validation +
  registration), `register_default_subagents`, `build_writer/readonly_subagent_agent`,
  `resolve_subagent_model`.
- `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs` (full) —
  pooled-agent construction, refresh surfaces (`apply_*`), `__task__:` keys.
- `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:444-551` —
  plugin Subagent registration, `framework_definition`, `build_plugin_agent`.
- `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:780-877, 1157-1174` —
  reload/rollback registration, unload.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/{profiles.rs:100-151,
  register.rs, task_tools.rs:29-80, executor.rs:2757-2830}` — default-route
  validation, task capability catalog, runtime contract read.
- `echo-agent-cli/echo-agent-app-core/src/{prompt_contract.rs, tool_exposure.rs:155-164,
  runtime.rs:100-130, state.rs (switch refs — zero subagent hits)}`.
- Framework anchors: `echo-agent/src/agent/subagent/types.rs:81-92`
  (`SubagentKind`), `src/agent/react/capabilities.rs:305-438`
  (registration + dispatch catalog), `src/agent/subagent/registry.rs`
  (`list_available`), `src/agent/subagent/prompt.rs`
  (`DefaultSubagentPromptCompiler`), `src/agent/subagent/types.rs`
  (`parse_subagent_outcome` fallback).

## Out Of Scope

- Framework Subagent definition/registry/prompt/result semantics → F-SUB-01
  (its P1-01/P2-01/P2-02/P2-03 are cross-referenced, not re-derived).
- Execution modes/team internals/cancellation → F-SUB-02.
- Config/hooks/workspace-switch lifecycle → A-CFG-01 (its P1-01 root family
  is cross-referenced for the switch-staleness arm).
- Tool exposure per mode → A-TOOL-01 (its P1-01 writer plan_mode finding is
  the canonical framework-side+EKO-side statement; this task adds the
  definition-source divergence arm).
- Frontend Subagent projections → A-FE-02, X-EVT-01.
- Skills/plugins hook wiring beyond Subagent registration → A-PLG-01.

## Inputs

- Root `AGENTS.md` (Subagent-only terminology, one-authority, layering
  gates, "动手前先查"), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-SUB-01 card), `zcode-ds/README.md`, report templates.
- Dependency task reports read in full: zcode-ds `F-SUB-01`, `F-SUB-02`,
  `A-CFG-01`; cross-reference: `A-TOOL-01` (P1-01), `A-BOOT-01` (via
  A-CFG-01 references only).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/MASTER-PLAN.md`
  (Subagent Prompt Compilation :154-171, Formal Plan Execution :139-141),
  `echo-agent-cli/docs/subagent-unification-plan.md`,
  `echo-agent-cli/docs/2026-07-17-domain-subagent-orchestration.md`,
  root `docs/MASTER-PLAN.md` (M11 :810-815) — classified in V05-01.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | Framework `SubagentRegistry` (single registration/execution authority), dispatch catalog, `SubagentPromptCompiler` trait, `SubagentDefinition`, `SubagentKind`, executor — correctly placed; reused by EKO without duplication. |
| EKO product policy | `.md` loader + project>user>builtin precedence, `SubagentCatalogSnapshot` + `validate_default_subagent_routes`, readonly/writer builders, worktree/workspace/team frontmatter policy, model override (`fast`), EKO prompt sections (language anchor, result quality, protocol, tool discovery), pool (conversation/background/task agents), plugin Subagent registration — application-owned, correctly placed per M11 and the loader's own justification (subagent_loader.rs:18-24). |
| Adapter boundary | `EkoSubagentPromptCompiler` (thin trait impl, one compiler) and the tag-encoding in `SubagentCatalogSnapshot::from_registered` (decodes `capability:`/`isolation:`/`prompt_source:` tags back into catalog entries) — the tag decode is a lossy adapter encoding (P3-01). No repository movement recommended. |
| Duplicate search | Terms searched: `worker`, `SubagentRegistry`, `discover_subagents`, `SubagentCatalogSnapshot`, `from_definitions`, `from_registered`, `validate_default_subagent_routes`, `EkoSubagentPromptCompiler`, `DefaultSubagentPromptCompiler`, `SubagentKind::Custom/Plugin`, `register_default_subagents`, `register_plugin_agents`, `sync_subagent_dispatch_catalog`, `list_available`, `set_plan_mode`, `compile_primary_invocation`, `prompt_source`. Results (V01-01): zero `worker` matches; one loader; one EKO compiler; one registry; no second lifecycle; two catalog builders of one type; plugin registration is a second *path* into the same registry. |
| Migration deletion | If the P2/P3 directions are taken: no framework API deletion; EKO-side deletions: the `SubagentKind::Custom`/`Plugin` writes (infra.rs:695, plugin_components.rs:485) if the framework field is removed per F-SUB-01-P2-01 direction (b); `from_registered` if `from_definitions` becomes the single projection; the "hot-loader" module title if no reload is ever implemented. |

## Current Path

Verified chain (V02-01):

- **Construction**: `create_agent_with_diagnostics` (infra.rs:194) resolves
  project root (params.project → working_dir → auto-discover), runs
  `discover_subagents` (builtin < user < project, last-write-wins),
  builds `SubagentCatalogSnapshot::from_definitions` (infra.rs:246-248),
  runs `validate_default_subagent_routes` with `?` (infra.rs:249-251),
  injects `snapshot.prompt()` into the system prompt via `PromptAssembler`
  (infra.rs:252-253), then `register_default_subagents` (infra.rs:479-496)
  compiles each role with `EkoSubagentPromptCompiler`, builds a readonly or
  writer `ReactAgent`, wraps it in a framework `SubagentDefinition` (with
  `fork_mode`, isolation flags, team spec, model override, `prompt_source:`/
  `capability:`/`isolation:` tags) and registers instance + factory in the
  framework registry; can_delegate instances sync their own dispatch
  catalog (infra.rs:862-872).
- **Pool**: every pooled agent (conversation `conv-*`, background
  `__background__`, task `__task__:*` from `PoolTaskAgentProvider`,
  tasks/service.rs:148) is created through the same `infra::create_agent`
  (agent_pool.rs:824-853), so each pool agent re-discovers/re-registers
  subagents and re-validates routes at creation time; pool agents built with
  `task_runtime_store` also register task tools against their own build-time
  snapshot (infra.rs:505-519).
- **Primary task tools**: the primary is built without `task_runtime_store`
  (runtime.rs:118-121); both entry points call
  `register_task_tools_on_agent` post-hoc (main.rs:177, desktop.rs:201),
  building the capability catalog from `from_registered(list_available)`
  (register.rs:37/:55) — this one includes plugin roles.
- **Plugins**: `register_plugin_agents` (plugin_components.rs:444) registers
  plugin roles into the same registry post-build; reload/rollback
  re-register and `unload_agent_components` unregisters by name
  (plugin_runtime.rs:1157-1174); plugin instances compile with the framework
  `DefaultSubagentPromptCompiler` (plugin_components.rs:541).
- **TaskRuntime**: `subagent_runtime_contract` (executor.rs:2757-2777)
  reads the registry `list_available()` and decodes `prompt_source:`/
  `isolation:` tags; dispatch goes through the framework executor (F-SUB-01/
  F-SUB-02 surfaces unchanged).

## Findings

### A-SUB-01-P2-01: Writer-subagent capability is definition-source-dependent — `.md`-loaded writers are stripped of write tools by `set_plan_mode(true)` while plugin writer subagents keep the full write surface (confirms and sharpens A-TOOL-01-P1-01)

- Priority: P2
- Confidence: high
- Layer: application (wiring)
- Evidence: `infra.rs:963` `subagent.set_plan_mode(true)` in
  `build_writer_subagent_agent` (the A-TOOL-01-P1-01 defect) vs
  `plugin_components.rs:511-551` `build_plugin_agent` — zero
  `set_plan_mode`/`plan_mode` occurrences in the plugin path (grep
  V03-01: only infra.rs:963/:1040 exist in app-core); plugin writers get
  `register_all_tools` (no `.readonly_tools()`), `.sandbox_manager` only
  when non-readonly (plugin_components.rs:529-533).
- Reachability: `run_writer_subagent` → fork factory — for `.md` roles the
  factory is `infra.rs:757-823` (build_writer_subagent_agent, plan_mode);
  for plugin roles the factory is `plugin_components.rs:463-470`
  (build_plugin_agent, no plan_mode). Same declared role semantics
  (`readonly: false` + `worktree: true`), different tool surface.
- Expected invariant: one declared capability (writer) yields one tool
  surface regardless of the definition source.
- Observed behavior: `.md` writer roles (builtin implementer, project/user
  overrides) are silently read-only in the LLM-visible and executable tool
  surface (A-TOOL-01-P1-01 chain); plugin writer roles are genuinely
  write-capable. The behavior flips with the registration path.
- Impact: the same user-authored `.md` semantics behave differently when
  shipped as a plugin; a fix that removes plan_mode from the writer builder
  must converge both paths and not touch the readonly builder (infra.rs:1040).
  Today it also means the framework plan-mode filter bug is masked for one
  half of the writer population, making the A-TOOL-01-P1-01 defect harder to
  observe.
- Root cause: the plan_mode flag was copied from the readonly builder tail
  into the `.md` writer builder only; the plugin builder was written later
  without it (per F-SUB-01/A-TOOL-01 root-cause family).
- Direction: resolve A-TOOL-01-P1-01 (remove `set_plan_mode(true)` from the
  writer builder, keep infra.rs:1040); add an EKO regression fixture that
  builds a writer role from both the `.md` path and the plugin path and
  asserts identical write-tool visibility.
- Regression validation: EKO unit fixture — writer role via
  `register_default_subagents` and via `register_plugin_agents` produce
  `tools_for_llm`-equivalent surfaces both containing write_file/shell/
  run_code; readonly role surfaces stay write-free.
- Validation reports: [V03-01](../validations/A-SUB-01/V03-01.md),
  [V02-01](../validations/A-SUB-01/V02-01.md)
- Cross-reference: canonical finding `A-TOOL-01-P1-01` (framework+EKO
  chain) and `F-EXT-01-P1-01` (framework plan-mode filter). This finding
  records the plugin-vs-md divergence arm for the synthesizer.

### A-SUB-01-P2-02: Plugin-registered Subagents are invisible to the EKO immutable catalog — the build-time snapshot (system-prompt catalog + pooled-agent task capability catalog) omits plugin roles while the framework dispatch catalog and the post-hoc `from_registered` catalog include them

- Priority: P2
- Confidence: high
- Layer: application (adapter boundary between EKO snapshot and framework catalog)
- Evidence: snapshot built from `discover_subagents` only (infra.rs:242-248)
  — plugin roles are registered later via `register_plugin_agents`
  (plugin_components.rs:444-476; activation after build, runtime.rs:279);
  system-prompt catalog injected at build (infra.rs:252-253); pooled-agent
  task tools built with the same snapshot (infra.rs:509-515, agent_pool.rs:847);
  primary task tools use `from_registered(list_available)` post-hoc
  (register.rs:37/:55) which includes plugin roles; framework `agent_tool`
  enum comes from the host dispatch catalog updated at registration
  (capabilities.rs:377-401) and also includes plugin roles.
- Reachability: any plugin declaring a Subagent (plugin manifest `agents/`)
  on any surface; the primary system prompt and every pooled agent's
  system prompt then list a role set that omits the plugin role, while
  `task_create`/`task_update` on the primary validate against the
  plugin-inclusive catalog (register.rs:55) and `agent_tool` accepts the
  plugin role.
- Expected invariant: MASTER-PLAN.md:169-171 — "The effective Subagent
  catalog is an immutable snapshot derived from the same definitions used
  for registration, including project and user roles" — one effective
  catalog for all LLM-facing surfaces.
- Observed behavior: three projections coexist — build-time snapshot
  (no plugins), `from_registered` (plugins), framework dispatch catalog
  (plugins). A pooled agent's `task_create` rejects a plugin role with
  "unknown Subagent" (task_tools.rs:53-65, boot snapshot) while the primary
  accepts it; the LLM never learns plugin roles from the system-prompt
  catalog section and must discover them only via tool schemas.
- Impact: inconsistent role acceptance between primary and pooled/background
  agents for the same `task_create` payload; stale LLM guidance about the
  available role catalog; the "immutable effective catalog" claim holds only
  for the `.md` surface.
- Root cause: the snapshot is taken before plugin activation and is never
  recomputed; plugin registration only refreshes the framework-side
  projections.
- Direction: after plugin (re)activation/unload, rebuild the effective
  catalog from the registry (`from_registered` or a single
  registry-derived snapshot) and refresh (a) the primary system-prompt
  catalog projection, (b) pooled-agent task capability catalogs — or
  document that the system-prompt catalog is `.md`-only while tool schemas
  are the authoritative role list. Delete the `from_definitions`/
  `from_registered` pair in favor of one builder if the tag encoding is
  kept (see P3-01).
- Regression validation: fixture — register a plugin Subagent on a primary
  with a pool, then assert the system-prompt catalog, the primary task
  capability catalog, and a pooled agent's catalog all contain the plugin
  role; and that `task_create` validation accepts the role on both agents.
- Validation reports: [V02-01](../validations/A-SUB-01/V02-01.md),
  [V03-01](../validations/A-SUB-01/V03-01.md), [V05-01](../validations/A-SUB-01/V05-01.md)

### A-SUB-01-P2-03: Plugin Subagents bypass the EKO prompt compiler and the can_delegate catalog sync — plugin roles get the raw role body (no language anchor, no result contract, no protocol) and their `agent_tool` schema is never populated

- Priority: P2
- Confidence: high
- Layer: application (adapter boundary)
- Evidence: `build_plugin_agent` uses `DefaultSubagentPromptCompiler`
  (plugin_components.rs:541) — its `compile_system` returns
  `role_prompt.trim()` only and its `compile_invocation` does not render the
  EKO sections (echo-agent/src/agent/subagent/prompt.rs); infra.rs:862-872
  syncs the dispatch catalog only for `.md` can_delegate instances, while
  plugin can_delegate instances are never synced (register path
  plugin_components.rs:463-472 has no sync call); `parse_subagent_outcome`
  (echo-agent/src/agent/subagent/types.rs) degrades to
  `split_subagent_output` (contract_version 0, no artifacts/verification/
  touched_files) when the model does not emit the JSON contract.
- Reachability: any plugin Subagent with `can_delegate: true` (agent_tool
  registered at build, empty/stale dispatch catalog); every plugin Subagent
  dispatch — its system prompt lacks `SUBAGENT_LANGUAGE_POLICY`,
  `SUBAGENT_RESULT_QUALITY_POLICY`, `SUBAGENT_COMMUNICATION_PROTOCOL`, and
  `render_result_contract` that `.md` roles receive (subagent_prompt.rs:208-229).
- Expected invariant: MASTER-PLAN.md:154-163 — one product compiler
  ("EKO compiles a cache-stable system prompt from role Markdown, common
  orchestration rules, ..., one language anchor, ..., and the canonical
  result contract") for every registered Subagent; every delegation-capable
  Subagent sees the same dispatch catalog.
- Observed behavior: plugin Subagent prompts are role-body-only unless the
  plugin author hand-writes every shared section; their results routinely
  fall back to unstructured summaries (parent summaries then lack artifact/
  verification/touched_files metadata); plugin can_delegate Subagents
  cannot meaningfully dispatch children (empty agent_tool schema), unlike
  `.md` roles.
- Impact: mixed-quality delegation from plugin roles (missing language
  anchoring can produce wrong-language replies; degraded structured
  results for TaskRuntime review), and a silent capability gap for
  plugin-declared delegation.
- Root cause: the plugin builder predates (or ignores) the EKO compiler and
  the catalog-sync contract that the `.md` builder implements.
- Direction: use `EkoSubagentPromptCompiler` in `build_plugin_agent`
  (compile_system with the same sections), and call
  `sync_subagent_dispatch_catalog` on can_delegate plugin instances after
  registration (mirror infra.rs:862-872); add a plugin-path cardinality test
  mirroring `builtin_system_prompts_have_single_owned_sections`.
- Regression validation: fixture building a plugin Subagent asserts the
  compiled system prompt contains the EKO language/result-quality/contract
  sections exactly once and that a can_delegate plugin instance's dispatch
  catalog lists registered roles.
- Validation reports: [V03-01](../validations/A-SUB-01/V03-01.md),
  [V01-01](../validations/A-SUB-01/V01-01.md), [V05-01](../validations/A-SUB-01/V05-01.md)

### A-SUB-01-P2-04: No runtime reload of Subagent definitions and no refresh on workspace switch — the "hot-loader" only reloads on a full agent rebuild; the primary and existing pooled agents keep the pre-switch project scope while newly created pooled agents discover the new scope, splitting the pool's effective catalog

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `discover_subagents` has exactly one production call site —
  `create_agent_with_diagnostics` (V01-01/V03-01), so `.md` edits apply
  only on the next agent build (documented at infra.rs:564-568); the config
  watcher watches config+hooks only (A-CFG-01-P1-01); `state.rs` has zero
  Subagent references so `switch_workspace` (state.rs:844-1032) never
  re-discovers/re-registers; `agent_pool.rs` has no subagent refresh method
  (only `apply_runtime_model`/`apply_permission_mode`/
  `refresh_skill_descriptors`/`apply_working_dir`/`apply_workspace_routing`/
  `apply_memory_store`/`refresh_instruction_context`); pooled agents created
  after a switch re-discover from the new cwd (agent_pool.rs:850 →
  infra.rs:235-245) while pre-existing pooled agents keep the old registry;
  module title "Subagent `.md` hot-loader" (subagent_loader.rs:1).
- Reachability: every GUI workspace switch (LeftSidebar → `switch_workspace`
  IPC → state.rs:844) with a pool containing pre-existing agents or a
  primary whose role prompts/catalog matter for the new project; every
  `.md` edit during a long-lived session.
- Expected invariant: the loader's documented resolution order
  ("project scope `<project_root>/.eko/subagents`", subagent_loader.rs:7-16)
  is re-evaluated when the project scope changes; all live Agents observe
  the same effective catalog (surface parity / one-authority).
- Observed behavior: after a switch, the primary and existing pool agents
  keep delegating with the old project's role prompts and catalog section,
  while new pool agents (and any background task agent acquired later) use
  the new project's; the same role name can resolve to different prompts in
  different pool entries; plugin reload (P2-02) refreshes only the registry,
  not these agents.
- Impact: cross-project role-prompt leakage and intra-pool catalog
  divergence after workspace switch; silent staleness of the LLM-visible
  role list; "hot" in the module name is misleading. Same root family as
  A-CFG-01-P1-01 (cwd-derived subsystems not treated as workspace state).
- Root cause: Subagent scope was designed as a build-time concern ("next
  agent build") while the product gained runtime workspace switching and a
  pool with long-lived agents; no refresh path was added.
- Direction: either (a) add a pool/state refresh that re-runs
  `discover_subagents` for the new project root, rebuilds the effective
  catalog, and re-registers the changed definitions (registry
  `register_sync` overwrites by name) on the primary and every pooled
  agent, wired into `switch_workspace`/`exit_workspace`; or (b) explicitly
  document subagent scope as boot-only and rename the module. Prefer (a)
  for parity with the documented hot-loading contract.
- Regression validation: fixture — boot in dir A with `A/.eko/subagents/
  explorer.md`, switch to dir B with a different `explorer.md`, assert the
  primary's registry definition and a pre-existing pooled agent's system
  prompt reflect B; a newly acquired pooled agent agrees with B.
- Validation reports: [V03-01](../validations/A-SUB-01/V03-01.md),
  [V02-01](../validations/A-SUB-01/V02-01.md), [V05-01](../validations/A-SUB-01/V05-01.md)
- Cross-reference: A-CFG-01-P1-01 (workspace switch leaves watcher/hooks/
  config bound to pre-switch scope) — same defect family, different
  subsystem.

### A-SUB-01-P3-01: Two catalog builders decode the same entry differently — `from_registered` re-derives readonly/isolation from tags, losing plugin isolation and diverging from `from_definitions`; the framework `SubagentKind::Custom/Plugin` field written by both EKO paths is never read

- Priority: P3
- Confidence: high
- Layer: application (adapter encoding) + framework (inert field)
- Evidence: `from_definitions` reads `readonly`/`isolate_worktree`/
  `isolate_workspace` fields (subagent_loader.rs:170-183); `from_registered`
  re-decodes `capability:readonly`/`readonly` and `isolation:` prefix tags
  (subagent_loader.rs:185-212) — the `.md` builder stamps those tags
  (infra.rs:730-742) but `framework_definition` for plugins stamps only
  frontmatter tags (plugin_components.rs:502), so plugin `worktree: true`
  writers project as isolation "context" in `from_registered` while the
  `.md` path projects "worktree"; TaskRuntime's own tag read has a kind
  fallback (executor.rs:2768-2777) masking the loss; `SubagentKind::Custom`
  (infra.rs:695) and `SubagentKind::Plugin` (plugin_components.rs:485) are
  written but zero-read at dispatch (V01-01) — real provenance lives in the
  `prompt_source:` tag (executor.rs:2761-2765).
- Reachability: `register.rs:37/:55` (primary task capability catalogs,
  both entry points) with any plugin Subagent present; any consumer of
  `SubagentKind` (none today).
- Expected invariant: one projection of a definition is identical no matter
  which builder produced it; no write-only fields.
- Observed behavior: plugin role isolation is misreported in the task
  capability catalog (only the kind-based fallback in
  `subagent_runtime_contract` keeps runtime diagnostics honest); two
  builders are a maintenance trap for future fields.
- Impact: misleading capability-catalog values; dead framework field
  written by EKO (F-SUB-01-P2-01 family — EKO-side arm).
- Root cause: the snapshot pair evolved independently — fields for the
  loader, tags for the registry round-trip — and the plugin path was never
  stamped with the tag vocabulary.
- Direction: if P2-02's single-builder direction is taken, delete
  `from_registered` and stamp `isolation:`/`capability:` tags in
  `framework_definition` (plugin path) for any remaining tag consumers; if
  F-SUB-01-P2-01 direction (b) deletes the framework dead fields, remove the
  `SubagentKind` writes here.
- Regression validation: fixture — plugin role with `worktree: true`
  registered, `from_registered` reports isolation "worktree"; grep
  `SubagentKind::Plugin|SubagentKind::Custom` returns zero production hits
  after deletion.
- Validation reports: [V01-01](../validations/A-SUB-01/V01-01.md),
  [V03-01](../validations/A-SUB-01/V03-01.md)

### A-SUB-01-P3-02: Default-route startup validation checks the parsed-definition catalog, not the registry — a role whose agent build fails is silently skipped at registration while the catalog and system prompt still advertise it

- Priority: P3
- Confidence: medium (requires a subagent build failure, which is rare)
- Layer: application
- Evidence: `validate_default_subagent_routes` checks `catalog.contains(role)`
  on `from_definitions` output (profiles.rs:139-151), which only depends on
  `.md` parse success; `register_default_subagents` maps build errors to
  `tracing::warn!` and continues (infra.rs:836-841), so the registry can
  miss a role the catalog lists; `subagent_runtime_contract` then reports
  `prompt_source = "unknown"` (executor.rs:2757-2765) and dispatch fails at
  registry lookup ("not found") while the system prompt catalog section
  still lists the role.
- Reachability: any runtime failure in `build_writer_subagent_agent`/
  `build_readonly_subagent_agent` (e.g. an invalid model id resolved from
  frontmatter `model:`), on the primary or any pooled agent creation.
- Expected invariant: the startup validation guarantees every advertised
  default route is executable (MASTER-PLAN.md:169-171 "startup validates
  every default route").
- Observed behavior: validation is name-level only; a registered-catalog/
  registry divergence passes silently and surfaces as a dispatch-time
  "not found".
- Impact: misleading catalog + delayed failure; low likelihood because
  builtin roles are compile-tested and builds rarely fail.
- Root cause: validation was written against the parse-time snapshot before
  registration existed; registration failures were deliberately
  non-fatal.
- Direction: after `register_default_subagents`, re-validate the default
  routes against `list_available()` and either fail creation or drop the
  role from the injected catalog prompt; add a fixture with a forced build
  failure (e.g. bad model spec) asserting the catalog is consistent with
  the registry.
- Regression validation: fixture forcing `build_*` to fail for one role
  asserts `validate_default_subagent_routes` against the registry errors
  (or the catalog prompt omits the role); V04 suites stay green.
- Validation reports: [V02-01](../validations/A-SUB-01/V02-01.md),
  [V03-01](../validations/A-SUB-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (loader precedence, catalog pair, compilers, SubagentKind readers, worker terms) | yes | passed | [V01-01](../validations/A-SUB-01/V01-01.md) |
| V02 | Registration and runtime reachability (create chain, route validation wiring, plugin registration, pool re-creation, TaskRuntime registry read) | yes | passed | [V02-01](../validations/A-SUB-01/V02-01.md) |
| V03 | Invariants (default-route startup validation; prompt cardinality/language; reload and pooled-Agent refresh; plugin envelope) | yes | passed (violations → P2-01..P2-04, P3-01/02) | [V03-01](../validations/A-SUB-01/V03-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked subagent_loader` | yes | passed (exit 0; 25 passed) | [V04-01](../validations/A-SUB-01/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked subagent_prompt` | yes | passed (exit 0; 7 passed) | [V04-02](../validations/A-SUB-01/V04-02.md) |
| V04 | `cargo test ... 'profiles::'` + `... prompt_contract` | yes | passed (exit 0; 7 + 5 passed) | [V04-03](../validations/A-SUB-01/V04-03.md) |
| V04 | `cargo check -p echo-agent-app-core --locked` + `agent_pool::tests` + `plugin_components` | yes | passed (exit 0; 27 + 2 passed) | [V04-04](../validations/A-SUB-01/V04-04.md) |
| V05 | Historical-document drift (CLI/root MASTER-PLAN, unification plan, orchestration doc) | conditional | passed | [V05-01](../validations/A-SUB-01/V05-01.md) |

All required validations executed with known exit codes; no validation is
pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| CLI MASTER-PLAN.md:169-171 — "The effective Subagent catalog is an immutable snapshot derived from the same definitions used for registration, including project and user roles, and startup validates every default route against it" | current (`.md` surface) / regressed in part (plugin roles; switch refresh) | infra.rs:246-253, 479-496; register.rs:37/:55; P2-02, P2-04 |
| CLI MASTER-PLAN.md:154-163 — one product compiler; EKO compiles system prompt from role Markdown + common rules + language anchor + result-quality + result contract | current (`.md` subagents) / regressed in part (plugin subagents) | subagent_prompt.rs:177-235; plugin_components.rs:541; P2-03 |
| CLI MASTER-PLAN.md:139-141 — `agent_tool` is the single ad-hoc Subagent mechanism in Chat mode; Auto/Task physically hide it | current | tool_exposure.rs:155-164 |
| CLI MASTER-PLAN.md:148-152 — factory-backed fresh instances; TeamAgent persistent mailbox lifecycle separate | current (first part) / stale (mailbox) | F-SUB-02-P2-03; not re-derived here |
| subagent-unification-plan.md:79 — no second execution-role layer; pool entries are Subagent-runtime implementation details | current | V01-01; agent_pool.rs `__task__:` keys (tasks/service.rs:148) |
| domain-subagent-orchestration.md:49 — `subagent` is an open role name extensible via project/user `.md` | current | task_tools.rs:49-80; validate_task_spec against snapshot |
| Root MASTER-PLAN.md:815 — DomainProfile/domain prompts/default-role routing stay in `echo-agent-cli`; framework keeps the generic Subagent execution contract | current | profiles.rs, subagent_prompt.rs, subagent_loader.rs app-layer; registry/executor framework |
| subagent_loader.rs:1 — "Subagent `.md` hot-loader" | stale (naming) | reload only via agent rebuild (infra.rs:564-568); P2-04 |

## Coverage And Uncertainty

- All conclusions are static except the V04 test runs; no process was
  launched and no live LLM dispatch or GUI workspace switch was executed.
- The plugin-activation boot order was traced (runtime.rs:279, post-build)
  but not dynamically executed; the P2-02 divergence rests on the static
  order plus the absence of any refresh call in state.rs/agent_pool.rs
  (grep-verified).
- `parse_subagent_outcome` fallback behavior for plugin roles is derived
  statically from types.rs; no live plugin dispatch was run.
- The primary-system-prompt catalog is one section among several prompt
  sections; its exact token weight and interaction with the tool-schema
  role list were not measured.
- `DefaultSubagentPromptCompiler` is framework API (F-SUB-01 scope); only
  its EKO-side usage was reviewed here.
- Whether P2-02/P2-04 should be fixed by refresh-APIs vs documented
  boot-only scope is a product decision; the findings document the
  divergence and give both directions.

## Handoff

- Conclusions downstream tasks may rely on: one loader with project >
  user > builtin precedence (V01/V04-01); one EKO compiler with tested
  section cardinality and language anchor on the `.md` surface (V04-02);
  default-route startup validation wired and effective at name level
  (V02/V04-03); the catalog/capability projections diverge for plugin
  roles (P2-02/P2-03/P3-01); no subagent reload exists and workspace
  switch splits the pool's effective catalog (P2-04); writer capability is
  source-dependent (P2-01, canonical A-TOOL-01-P1-01); zero worker
  terminology (V01).
- Reports to read: this report + V01-01..V05-01; dependency reports
  F-SUB-01, F-SUB-02, A-CFG-01; A-TOOL-01 (P1-01 canonical chain).
- A-TOOL-01: the P2-01 arm confirms the plan_mode defect is confined to the
  `.md` writer path and gives a second regression surface for the fix.
- A-PLG-01/X-PLG-01: plugin Subagent registration/reload/unload semantics
  (P2-02/P2-03) and the can_delegate catalog-sync gap.
- X-BND-01: record the two-catalog projection decision (P2-02/P3-01) and
  the boot-only-vs-refresh subagent scope decision (P2-04).
- X-SRF-01/Q-E2E-01: workspace-switch + pooled-agent subagent catalog
  parity rows (P2-04).
- Conditions that make this report stale: changes to `subagent_loader.rs`
  (precedence/snapshot builders), `subagent_prompt.rs`,
  `infra.rs` (create_agent/register_default_subagents/writer-readonly
  builders), `plugin_components.rs` (build_plugin_agent/framework_definition),
  `agent_pool.rs` (create_agent/refresh methods), `register.rs`,
  `profiles.rs:139-151`, `executor.rs:2757-2777`, `state.rs` (switch
  subagent handling), or `capabilities.rs` registration/catalog paths.
- Follow-up task IDs (fixes not implemented in this review): X-BND-01,
  A-PLG-01, A-SRF-03/A-FE-02 (catalog projections), Q-E2E-01 (switch
  fixture), S-RDM-01 (P2-01..P2-04, P3-01/02 direction decisions).
