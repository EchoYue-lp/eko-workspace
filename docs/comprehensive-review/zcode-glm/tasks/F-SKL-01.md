# F-SKL-01: Skill loading and execution

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean

## Question

Are skill discovery, frontmatter, dependency probing, prompt/script execution,
source identity, hooks, and reload behavior deterministic?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-execution/src/skills/mod.rs` — module facade and re-exports.
- `echo-agent/echo-execution/src/skills/external/loader.rs` — `SkillLoader`,
  `DiscoveryScope`, `SkillLoadPolicy`, `parse_frontmatter`, `extract_instructions`,
  `validate_and_sort_dependencies` (DFS cycle detection), `scope_to_dirs`.
- `echo-agent/echo-execution/src/skills/external/types.rs` — `SkillDescriptor`,
  `RawFrontmatter`, `AllowedToolsValue`, `SkillContent`, `SkillSandboxPolicy`,
  `is_skill_control_tool`, `skill_allows_tool`, `tool_matcher`,
  `validate_name`, `validate_paths`, `matches_context_path`.
- `echo-agent/echo-execution/src/skills/external/activate_tool.rs` —
  `ActivateSkillTool` (Tier 2 model-driven activation; `available_names`
  snapshot, `paths` constraint check, `activate_allowed_tools` promotion).
- `echo-agent/echo-execution/src/skills/external/prompt_exec.rs` —
  `process_skill_content`, `substitute_variables`, `run_command` (sandboxed
  + direct fallback), `check_skill_command_safety` (shell-feature gate),
  placeholder/injection-defense algorithm.
- `echo-agent/echo-execution/src/skills/external/run_script_tool.rs` —
  `RunSkillScriptsTool` (interpreter resolution, traversal check +
  canonicalization containment, sandbox vs direct fallback, `kill_on_drop`).
- `echo-agent/echo-execution/src/skills/external/resource_tool.rs` —
  `ReadSkillResourceTool` (`is_path_traversal_safe`, canonicalize-contain
  check, max-bytes gate).
- `echo-agent/echo-execution/src/skills/dependency_probe.rs` —
  `extract_dependencies`, `missing_binary_names`, `binary_available`
  (subprocess `which` probe, conservative-miss semantics).
- `echo-agent/echo-execution/src/skills/hooks.rs` — `HookAction`,
  `HookRule`, `HooksDefinition`, `HookRegistry`, `matches_hook`,
  `execute_command_hook`/`execute_http_hook`, `parse_hook_output`,
  `merge_result`, `validate_hook_rules` (early invalid-action drop).
- `echo-agent/echo-execution/src/skills/registry.rs` — `SkillRegistry`,
  `register_descriptor`/`register_descriptor_with_legacy`, `tag_source`,
  `unregister_by_source`/`remove_descriptor`, `activate_with_args`,
  `activate_dependencies` (recursive), `active_skill_allowed_tools`.
- `echo-agent/src/skills/mod.rs` — root facade re-exports from
  `echo_core::tools::skill` and `echo_execution::skills`.
- `echo-agent/src/agent/react/capabilities.rs` — `add_skill`,
  `discover_skills`/`discover_skills_inner`, `load_skills_from_dir`,
  `load_plugin_skills_from_dir`, `tag_skills_source`,
  `unregister_skills_by_source`, `reconcile_skill_load_policy`,
  `activate_skill` (the single activation authority).
- `echo-agent/echo-execution/src/tools.rs:529` — `ToolManager::register`
  (`DashMap::insert`, silent overwrite).

## Out Of Scope

- Application-level install/index UI (`echo-agent-app-core/src/skills_hub/`)
  — `SkillsHub` is a separate app-layer registry that calls into the
  framework loader; covered briefly for layering context only, full audit
  belongs to A-PLG-01.
- Plugin runtime composition / failure rollback — deferred to F-PLG-01
  (`echo-core/src/plugin`, `app-core/src/plugin_runtime.rs`).
- MCP-sourced skills — `SkillSource::Mcp` is exercised at the contract
  level (no shell execution); MCP transport lifecycle is F-INT-01.
- Sandbox internals (`SandboxManager`, seatbelt/landlock enforcement) —
  F-SEC-01 owns the sandbox boundary; this task only checks the
  skill-facing API surface (`SandboxCommand` builder, `kill_on_drop`).
- LLM tool-call schema mapping for the three progressive-disclosure tools —
  F-LLM-01/02/03.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — Subagent-only terminology (N/A here), UTF-8 /
    no-panic rules, framework-vs-application layering gate, dead-code
    cleanup rule, "first search whether it already exists" rule,
    "investigate mature implementations before designing" rule.
  - `docs/comprehensive-review/REPORTING.md`.
  - `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
- Dependency task reports read:
  - `F-EXT-01` (this reviewer) — relied on its conclusion that
    `ToolManager::register` keys the `DashMap<String, Box<dyn Tool>>` by
    tool name (silent overwrite) and that `ToolResult::error` is the
    canonical in-band error channel that the skill tools use.
  - `F-CORE-01` (this reviewer, per PROGRESS.md) — relied on
    `CancellationToken` being the canonical cancellation primitive; this
    task verifies that skill tools surface cancellation through future-drop
    + `kill_on_drop(true)` rather than a separate primitive.
- Historical documents treated as hypotheses: none. The module-level docs
  cite the [agentskills.io specification](https://agentskills.io/specification)
  as the design reference; this report treats the spec only as the
  documented format contract, not as historical evidence about the code.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. Discovery (filesystem walk, YAML frontmatter parsing), progressive disclosure (catalog/activate/resource tiers), inline-command substitution with injection defense, script execution with traversal/canonicalize containment, sandbox policy plumbing, hook dispatch + merge semantics, and dependency DFS are all generic capabilities any `echo-agent` consumer (EKO, third-party headless, future CLI) needs. They live correctly in `echo-execution` and `echo_core::tools::skill` (V01 confirms single definition sites). |
| EKO product policy | None at the framework layer. `SkillLoadPolicy`/`tag_source`/`unregister_by_source` are explicit seams: the framework provides the *mechanism* (group unload by source tag) and the application provides the *policy* (which skills are enabled, plugin identity, baseline methodology list). `DEFAULT_BASELINE_SKILLS` and `inject_methodology_baseline` are framework constants but they only fire when an application opts in via `enabled_baseline` — the framework does not impose them. `SkillsHub` (echo-agent-cli) is the application-side install/index UI and is correctly NOT duplicated at the framework layer. |
| Adapter boundary | Thin. The application adapter feeds `DiscoveryScope::Custom(dir)` into `agent.load_skills_from_dir` (e.g. `runtime.rs:156`, `runtime.rs:260`, `state.rs:994`, `state.rs:1143`) and reads back a list of names; all parsing, validation, registration, and tool replacement happen inside the framework. The plugin adapter calls `load_plugin_skills_from_dir` + `tag_skills_source` + `unregister_skills_by_source` to manage lifecycle. No second authority exists at the adapter: the framework owns the catalog/registry and the three progressive-disclosure tools. |
| Duplicate search | Searched names/behaviors across both repos: `SkillLoader`, `SkillRegistry`, `SkillManager`, `discover_skills`, `load_skills_from_dir`, `SkillsHub`, `parse_frontmatter`, `parse_skill_md`, `extract_instructions`, `extract_body`, `strip_frontmatter`, `SkillDescriptor`, `RawFrontmatter`, `SkillContent`, `ActivateSkillTool`, `ReadSkillResourceTool`, `RunSkillScriptsTool`, `HooksDefinition`, `HookRegistry`, `tag_source`, `unregister_by_source`, `process_skill_content`, `run_command`, `check_skill_command_safety`. Result: the framework has one loader, one registry, one hook dispatcher, and one set of progressive-disclosure tools. `echo-agent-cli/echo-agent-app-core/src/skills_hub/registry.rs::SkillsHub` is an *application* install/index UI over `~/.eko/skills/` that delegates actual loading to the framework loader — it is NOT a duplicate runtime loader. Two helpers (`extract_body` in registry.rs and `extract_instructions`/`parse_frontmatter` in loader.rs) compute the SKILL.md body independently (see Findings for the duplication this causes in edge cases). |
| Migration deletion | No migration proposed in this task. The `extract_body` vs `extract_instructions` duplication is identified as a deletion/consolidation candidate (P3) but no code is changed in this review. |

## Current Path

Verified skill loading/execution data flow at commit `9b0e0fa` /
`b3b2e81`:

1. **Scope expansion.** `discover(scopes)` (`loader.rs:120`) iterates the
   caller-supplied slice in order and calls `scope_to_dirs` per scope
   (`loader.rs:556`). `Project(root)` expands to
   `[root/skills, root/.agents/skills]`; `User` expands to
   `[~/.agents/skills]` (empty vec if `dirs::home_dir()` is `None`);
   `Custom(p)` expands to `[p]`. Scopes are processed strictly in the
   caller-provided order — there is no implicit reordering.

2. **Recursive scan.** `scan_directory(dir, depth)` (`loader.rs:183`)
   reads entries with `tokio::fs::read_dir` and recurses into each
   subdirectory up to `MAX_SCAN_DEPTH = 4`. Directories named in
   `SKIP_DIRS` (`.git`, `node_modules`, `target`, `__pycache__`, `.venv`,
   `dist`, `build`) are skipped. If a subdirectory contains `SKILL.md`
   it is parsed in place; otherwise the scan recurses to support
   `skills/<category>/<name>/SKILL.md` layouts.

3. **Frontmatter parsing.** `parse_skill_file_with_variables`
   (`loader.rs:340`) reads the file, optionally applies
   `PluginVariables::substitute`, then calls `parse_frontmatter`
   (`loader.rs:407`). `parse_frontmatter` requires the content to start
   with `---`, finds the closing `\n---`, rejects closing markers with
   trailing same-line content, and deserializes the YAML block via
   `serde_yaml_ng`. Empty `description` is a hard error (skill skipped);
   name/dirname mismatch, name-length, name-character, and consecutive
   hyphen issues are soft warnings only.

4. **Hooks merge.** If `hooks.json` sits next to `SKILL.md`
   (`loader.rs:228`), the loader reads it, applies plugin variables,
   parses it as `HooksDefinition`, and either replaces or merges into
   the descriptor's `hooks` field. Parse failures are logged and
   dropped (skill still loads).

5. **Policy gate.** Each parsed descriptor is checked against the
   optional `SkillLoadPolicy` (`loader.rs:135`). Disallowed descriptors
   are dropped silently with an `info!` log. The policy is the
   application's enabled/disabled or curator hook.

6. **Name precedence.** A descriptor is inserted into the loader's
   `descriptors: HashMap<String, SkillDescriptor>` only if no entry with
   the same name exists yet (`loader.rs:147`). The first scope's
   descriptor wins; later duplicates log a `warn!` "shadowed by
   existing" and are dropped. **Within a single `scan_directory` call,
   the iteration order is `tokio::fs::read_dir` order, which is
   filesystem-dependent and not sorted** (see Finding F-SKL-01-P2-01).

7. **Dependency sort.** `validate_and_sort_dependencies`
   (`loader.rs:492`) walks all descriptors, logs missing-dependency
   warnings, and runs DFS cycle detection (`detect_cycle`,
   `loader.rs:527`). Cycles and missing deps are warning-only;
   activation order is enforced later at `activate_dependencies`.

8. **Registration into the runtime.** `discover_skills_inner`
   (`capabilities.rs:635`) creates (or reuses) a `SharedRegistry`
   (`Arc<RwLock<SkillRegistry>>`) and, for each descriptor, calls
   `register_descriptor_with_legacy` on BOTH the agent's catalog
   registry (`self.tools.skill_registry`) and the shared progressive
   registry, then optionally `tag_source_with_variables` for plugin
   ownership. The InstructionsLoaded hook fires once per discovery
   pass. The catalog is written as a *projection*
   (`SKILL_CATALOG_PROJECTION`) into the context, so repeated discovery
   passes replace rather than append.

9. **Progressive-disclosure tools.** After each discovery pass
   (`capabilities.rs:783`), the three tools are re-created with the
   latest `available_names` and re-registered via `replace_tool`
   (`capabilities.rs:72`: unregister-by-name + register). This keeps
   the tool's snapshot of skill names in sync with the registry across
   repeated `discover_skills()` calls.

10. **Activation (Tier 2).** `SkillRegistry::activate_with_args`
    (`registry.rs:370`) recursively activates `depends_on` first
    (missing deps are warning-only, dep failures don't abort), reads
    the SKILL.md body, falls back to legacy frontmatter `instructions`
    when the body is empty, applies plugin variables, then runs
    `process_skill_content` to substitute variables and execute inline
    commands. `enumerate_resources` lists files under `scripts/`,
    `references/`, `assets/` (sorted by relative path) plus top-level
    `.md/.txt/.yaml/.yml/.json` files. The sandbox policy, if declared
    and constraining, is stored in `active_sandbox_policies`. The
    skill name is added to `activated: HashSet<String>` for dedup.

11. **Prompt execution.** `process_skill_content` (`prompt_exec.rs:108`)
    implements a strict injection-defense algorithm: extract command
    regions from the *original* content, replace each with an
    `\x01CMD_OUT_N\x01` placeholder, run variable substitution on the
    placeholder-safe text, scan the substituted text for *new* command
    markers (reject and strip placeholders if any appear — preventing
    `${ARGUMENTS}`-driven injection), then execute the original
    commands and substitute outputs. `SkillSource::Mcp` short-circuits
    to substitution-only (no execution). The direct fallback path
    calls `check_skill_command_safety` which uses the same classifier
    as `ShellTool` (or a conservative metachar/blacklist fallback when
    the `shell` feature is disabled).

12. **Script execution (Tier 3).** `RunSkillScriptsTool::execute`
    (`run_script_tool.rs:158`) rejects absolute paths and `ParentDir`
    components up front (`run_script_tool.rs:178`), requires the skill
    be activated, requires `descriptor.permits_tool(self.name())`,
    canonicalizes both skill dir and script path, then asserts
    `canonical_script_path.starts_with(&canonical_skill_dir)`
    (symlink-traversal safe). Interpreter resolution
    (`resolve_interpreter`) prefers `uv run --script` for Python, then
    python3/python/py; bun/deno/npx-tsx for TypeScript; bash on Unix
    and Git Bash/WSL/PowerShell on Windows for shell. The sandboxed
    path applies `SkillSandboxPolicy` (timeout, network, paths) via
    `ResourceLimits`; the direct fallback uses `kill_on_drop(true)` +
    `tokio::time::timeout(timeout, cmd.output())` + minimal env
    (`minimal_env`, no inherited secrets).

13. **Resource reads (Tier 3).** `ReadSkillResourceTool::execute`
    (`resource_tool.rs:76`) requires activation + permits_tool,
    rejects `ParentDir` components via `is_path_traversal_safe`,
    canonicalizes both sides, asserts containment, enforces
    `DEFAULT_MAX_RESOURCE_BYTES = 1 MiB`, and returns the file content
    wrapped in `<skill_resource>` tags.

14. **Hook dispatch.** `HookRegistry::run_hooks` (`hooks.rs:792`)
    iterates sources in a fixed precedence: `UserConfig` first, then
    `Plugin` (alphabetical), then `Skill` (alphabetical). Each rule's
    matcher is checked via `matches_hook` (wildcard / exact / pipe /
    glob / `Bash(...)` prefix). Command/Http/McpTool/Agent hooks have
    per-action timeouts; `Permission` decisions merge with priority
    `deny > ask > RequireApproval > allow`; `activate_skill` requests
    are first-wins; `injected_context` concatenates with newline;
    `block`/`stop_propagation` short-circuit. Command hooks pipe a
    JSON serialization of `HookContext` to the child's stdin.

15. **Source identity & unload.** `SkillDescriptor::source`
    (`types.rs:108`) is an `Option<String>` tagged by the plugin
    integrator. `tag_source_with_variables` only tags descriptors whose
    `source` is `None` (avoids cross-plugin contamination).
    `unregister_by_source` (`registry.rs:133`) removes the source's
    entry from `by_source`, then `remove_descriptor` each name, which
    in turn purges `legacy_instructions`, `plugin_variables`,
    `activated`, and `active_sandbox_policies`. The agent's
    `unregister_skills_by_source` (`capabilities.rs:915`) mirrors this
    on the catalog registry, the shared progressive registry, the hook
    registry (`HookSource::Skill(name)`), and the context projection
    (`echo-agent:skill:{name}`), then refreshes the three progressive
    tools.

16. **Reload.** There is NO framework-level filesystem watcher for
    skills. Re-discovery happens only via explicit
    `discover_skills()`/`load_skills_from_dir()` calls (or
    `reconcile_skill_load_policy` for policy-driven removal). Hot-reload
    is an application concern: the CLI plugin runtime calls
    `unregister_skills_by_source` then `load_plugin_skills_from_dir` to
    rebuild (see F-PLG-01). The framework provides the primitives
    (source tagging, group unload, tool refresh) but not the watcher.

## Findings

### F-SKL-01-P2-01: Within-scope skill-name collision resolution is non-deterministic across runs

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-execution/src/skills/external/loader.rs:198` —
    `while let Some(entry) = entries.next_entry().await ...` iterates
    `tokio::fs::read_dir` output in filesystem order.
  - `echo-agent/echo-execution/src/skills/external/loader.rs:284` —
    nested recursion `Box::pin(self.scan_directory(&path, depth + 1))`
    preserves the same filesystem-dependent order.
  - `echo-agent/echo-execution/src/skills/external/loader.rs:147` —
    `if let Some(existing) = self.descriptors.get(&desc.name) { warn!(...)
    } else { ... self.descriptors.insert(...) }` resolves collisions by
    first-observed-wins, where "first observed" is the read_dir order.
- Reachability: every `discover_skills()`/`load_skills_from_dir()` call
  exercises this path. Live callers: `runtime.rs:156` (built-in skills),
  `runtime.rs:260` (workspace skills), `state.rs:994` and `state.rs:1143`
  (workspace + global skills), `tool_exposure.rs:303`, and the CLI/Tauri
  panels' skill-load commands.
- Expected invariant: the question asks whether discovery is
  *deterministic*. Cross-scope precedence is documented as "earlier
  scopes take precedence" (`loader.rs:14` and `discover` doc at
  `loader.rs:118`). The within-scope tie-break is unspecified.
- Observed behavior: when two directories under the *same* scope root
  both declare a skill with the same `name:` (e.g. a stale duplicate
  left by a plugin reinstall, or a user mistake), which one wins
  depends on the order `tokio::fs::read_dir` returns entries. That
  order is OS- and filesystem-dependent (ext4 hashes, APFS b-tree,
  inode allocation order, etc.) and is NOT sorted by the loader.
  Cross-platform, cross-filesystem runs of the same project can
  therefore load different skill bodies for the same name, with only a
  `warn!` log to signal it.
- Impact: silent wrong-skill-loaded bug that surfaces only when (a) a
  user has same-named skills in one directory tree and (b) the project
  is touched from a different OS/filesystem. The catalog prompt, the
  `available_names` snapshot, and the activated instructions all
  diverge from the developer's intent. Hard to reproduce in support.
- Root cause: design choice — the loader picks first-observed-wins
  rather than enforcing name uniqueness or sorting entries. `read_dir`
  ordering is not stable across filesystems; the loader does not
  canonicalize it.
- Direction: sort directory entries by name inside `scan_directory`
  before recursing/parsing, OR change the collision rule to
  deterministic last-wins-by-sorted-path, OR escalate same-name
  collisions within one scope from `warn!` to a hard error. Cheapest
  fix: collect all parsed descriptors per scope, sort ties by
  `descriptor.location` (canonical path), then feed into the
  `descriptors` map. The fix belongs in `loader.rs`, not the
  application — it is a generic discovery invariant.
- Regression validation: extend
  `scan_directory_finds_nested_category_skills` (loader.rs:661) or add a
  new test that creates two same-named skills under one root and
  asserts the winner is deterministic across repeated runs and across
  alphabetical/reverse-creation order. Run
  `cargo test -p echo_execution --lib skills::external::loader`.
- Validation reports: [V01](../validations/F-SKL-01/V01-01.md)

### F-SKL-01-P2-02: `SkillRegistry::register_descriptor` silently overwrites same-named skills and leaks stale `legacy_instructions`/`plugin_variables`

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-execution/src/skills/registry.rs:95` —
    `register_descriptor` updates the `by_source` reverse index when the
    previous occupant was source-tagged, but does NOT clear
    `legacy_instructions[name]` or `plugin_variables[name]` before
    `self.descriptors.insert(...)`.
  - `echo-agent/echo-execution/src/skills/registry.rs:193` —
    `register_descriptor_with_legacy` only inserts into
    `legacy_instructions` when the new legacy is non-empty; it never
    *removes* a stale entry left by the previous occupant.
  - `echo-agent/echo-execution/src/skills/registry.rs:412` — at
    activation, `if let Some(variables) = self.plugin_variables.get(name)`
    is consulted regardless of whether the variables belong to the
    current occupant.
  - `echo-agent/echo-execution/src/skills/registry.rs:402` — at
    activation, the legacy-instructions fallback fires whenever the new
    body is empty, reading whatever sits in `legacy_instructions[name]`.
- Reachability: the loader's `discover()` filters duplicates so the
  primary application path (one-shot discovery) is unaffected. The
  defect is reachable when an application *replaces* a descriptor by
  name without going through `remove_descriptor` first — e.g. a plugin
  uninstall+reinstall sequence that calls `register_descriptor` with
  the same name but different content, or a future adapter that
  hot-swaps a descriptor.
- Expected invariant: replacing a descriptor should leave the
  registry's auxiliary maps (`legacy_instructions`,
  `plugin_variables`, `activated`, `active_sandbox_policies`)
  consistent with the new descriptor — the same state that
  `remove_descriptor` produces.
- Observed behavior: `register_descriptor` correctly updates the
  `by_source` reverse index, but leaves `legacy_instructions[name]` and
  `plugin_variables[name]` from the previous occupant in place. If the
  replacement skill has an empty body, activation silently uses the
  *previous* skill's legacy instructions; if the replacement has no
  plugin variables, the *previous* occupant's variables still apply
  during activation. The mismatch is silent (no log).
- Impact: latent correctness defect on plugin swap / hot-reload paths.
  No security impact (legacy instructions and plugin variables are
  both skill-scoped strings authored by the same trust principal in
  practice), but a stale body or stale variable substitution can
  confuse users after a plugin reinstall or a content edit. Today's
  application code happens to call `remove_descriptor`/`unregister_*`
  before re-registering, so the bug does not fire — but the contract
  is fragile.
- Root cause: incomplete mirroring of `remove_descriptor` cleanup in
  `register_descriptor`'s overwrite branch. The reverse-index update
  was added (P1-reload, per the comment) but the
  legacy/plugin_variables cleanup was missed.
- Direction: in `register_descriptor`'s overwrite branch, when the
  incoming descriptor does not carry legacy/plugin_variables, remove
  the stale entries — OR make the overwrite path call
  `remove_descriptor(name)` first, then insert fresh. Add a regression
  test that swaps a descriptor carrying legacy instructions for one
  without, and asserts activation does not fall back to the previous
  legacy body.
- Regression validation: new test in
  `echo_execution::skills::registry::tests` exercising
  legacy/plugin_variables replacement; `activate_with_args` must not
  surface the previous occupant's content. Run
  `cargo test -p echo_execution --lib skills::registry`.
- Validation reports: [V01](../validations/F-SKL-01/V01-01.md),
  [V04](../validations/F-SKL-01/V04-01.md)

### F-SKL-01-P3-01: Skill `name` validation is warning-only, so malformed names leak into the catalog and system prompt

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-execution/src/skills/external/types.rs:179` —
    `validate_name` returns `Vec<String>` of warnings; callers only
    log them.
  - `echo-agent/echo-execution/src/skills/external/loader.rs:380` —
    `for warning in descriptor.validate_name() { warn!(...) }` then
    loading proceeds.
  - `echo-agent/echo-execution/src/skills/external/loader.rs:373` —
    the name-vs-directory mismatch is also warning-only.
- Reachability: every discovery pass. A SKILL.md with `name: Code_Review`
  or `name: ../etc` parses successfully, is keyed into the
  `descriptors` map under that exact string, and is rendered in the
  catalog prompt (`catalog_line`, types.rs:156) that is injected into
  the system prompt.
- Expected invariant: per the agentskills.io spec cited in the module
  docs, skill names should be kebab-case 1-64 chars lowercase. The
  invariant is enforced as a soft warning, not a hard gate.
- Observed behavior: a malformed name still loads. There is no
  security impact (the `name` is used only as a HashMap key and in
  catalog/log output; filesystem access is gated by the canonicalized
  `descriptor.location` and the traversal/containment checks in
  `run_script_tool.rs`/`resource_tool.rs`, both verified in V02). The
  user-visible consequence is that the catalog prompt and
  `available_names` snapshot may contain confusing or
  non-spec-compliant names.
- Impact: low. No correctness or security defect in audited paths.
  Cosmetic / spec-hygiene issue affecting catalog quality.
- Root cause: design choice — lenient validation per the agentskills.io
  integration guide ("name issues produce warnings but don't block
  loading"), which is defensible for forward-compatibility but leaves
  the catalog open to names that violate the spec.
- Direction: either accept the current lenient policy and document it
  on `SkillDescriptor::name`, or escalate length/character violations
  to hard errors (keeping the directory-mismatch as a warning since
  case-only differences are recoverable). No deletion target.
- Regression validation: extend `test_descriptor_validate_name_invalid`
  (types.rs:599) with a positive test confirming the desired
  accept/reject behavior.
- Validation reports: [V02](../validations/F-SKL-01/V02-01.md)

### F-SKL-01-P3-02: User tools named `activate_skill`/`read_skill_resource`/`run_skill_script` are silently replaced on every discovery pass

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/capabilities.rs:785` — after each
    `discover_skills_inner`, the framework calls `self.replace_tool`
    with fresh instances of `ActivateSkillTool`, `ReadSkillResourceTool`,
    and `RunSkillScriptsTool`.
  - `echo-agent/src/agent/react/capabilities.rs:72` — `replace_tool`
    does `unregister(name) + register(tool)`; the previous tool is
    dropped. No warning is emitted if the previous tool was a
    user-supplied instance rather than the framework's own.
  - `echo-agent/echo-execution/src/tools.rs:529` — the underlying
    `ToolManager::register` is `DashMap::insert` keyed by tool name;
    collisions always silently overwrite.
- Reachability: any caller that registers a custom `Tool` whose
  `name()` collides with one of the three progressive-disclosure tool
  names, and then calls `discover_skills`. Live in canonical paths:
  every EKO startup with built-in skills triggers the replacement.
- Expected invariant: tool replacement is intentional and required for
  progressive disclosure to work (the tool's `available_names`
  snapshot must track the registry). The invariant that is missing is
  *observability* — the user should be told their tool was overridden.
- Observed behavior: the user's custom tool is silently dropped. There
  is no log, no error, and no obvious signal in `tool_manager.list_tools()`
  (the name still appears, but it is now the framework's instance).
- Impact: low for the canonical EKO flow (which does not register
  custom tools under those names), but a third-party consumer that
  legitimately wants to override `activate_skill` (e.g. a telemetry
  wrapper) will see its wrapper discarded on the next discovery pass
  with no diagnostic.
- Root cause: the three tool names are owned by the framework but the
  ownership is implicit. `replace_tool` has no concept of "owned"
  vs "user" tools.
- Direction: either (a) log at `info!` when `replace_tool` is about to
  drop a non-framework tool (cheapest), or (b) tag the three
  progressive-disclosure tools with a `framework_owned` marker and have
  `replace_tool` warn when displacing a non-framework tool of the same
  name. Optionally, expose a stable hook for users to wrap the
  framework's progressive-disclosure tools.
- Regression validation: new test in `echo_execution::tools` or
  `agent::react::capabilities` registering a sentinel tool named
  `activate_skill` and asserting the displacement is logged.
- Validation reports: [V03](../validations/F-SKL-01/V03-01.md)

### F-SKL-01-P3-03: `scan_directory` lacks symlink-loop protection (bounded only by `MAX_SCAN_DEPTH`)

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-execution/src/skills/external/loader.rs:204` —
    `if !path.is_dir() { continue; }`. `Path::is_dir()` follows
    symlinks, so a symlinked directory is treated as a directory and
    recursed into.
  - `echo-agent/echo-execution/src/skills/external/loader.rs:188` —
    `if depth > MAX_SCAN_DEPTH { return Ok(vec![]); }` is the only
    recursion bound. There is no visited-set / inode-tracking.
- Reachability: any skill tree containing a symlink cycle. Skills are
  user-authored and plugin-installed, so a malicious or buggy plugin
  can ship a symlink loop. The depth cap prevents runaway recursion
  but a cycle within the first 4 levels is silently walked
  redundantly and a `SKILL.md` reachable via two paths can be parsed
  twice (with the first-wins rule then applying).
- Expected invariant: discovery should be idempotent w.r.t. how a
  `SKILL.md` is reached. Two paths to the same skill should not
  duplicate work or depend on traversal order.
- Observed behavior: bounded but non-idempotent. A symlinked subdirectory
  at depth ≤ 4 is walked; the `descriptors` map dedups by name so the
  end state is correct, but discovery time and log noise grow with the
  cycle. Real-world impact is low because skill directories rarely
  contain symlinks.
- Impact: low. Worst case is wasted scan work and confusing duplicate
  "Discovered skill" logs; no correctness or security impact (paths are
  canonicalized at parse time, so a symlinked skill has a stable
  canonical `location`).
- Root cause: no `visited: HashSet<PathBuf>` (canonicalized) threaded
  through `scan_directory`. The depth cap is a coarse guard against
  runaway recursion but not against cycles.
- Direction: thread a `HashSet<PathBuf>` of canonicalized visited
  directories through `scan_directory` and skip already-visited paths.
  Alternatively, document the depth cap as the only protection and
  accept the redundancy. Prefer the visited-set fix; it is cheap.
- Regression validation: new test creating a symlink loop under a temp
  dir and asserting the looped skill is parsed exactly once.
- Validation reports: [V01](../validations/F-SKL-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Discovery precedence (scopes, dirs, order) and DFS dependency sort are deterministic across scopes; within-scope tie-break inspected. | yes | passed | [V01-01](../validations/F-SKL-01/V01-01.md) |
| V02 | Malformed SKILL.md (bad/missing frontmatter, missing file, path traversal in resource/script, oversized resource) is rejected or safely contained. | yes | passed | [V02-01](../validations/F-SKL-01/V02-01.md) |
| V03 | Two tools with the same name resolve deterministically (`ToolManager::register` last-wins; `replace_tool` unregister+register). | yes | passed | [V03-01](../validations/F-SKL-01/V03-01.md) |
| V04 | Reload/unload path (`unregister_skills_by_source`, `remove_descriptor`) cleans up bookkeeping; script cancellation relies on `kill_on_drop` + CancellationToken drop. | yes | passed | [V04-01](../validations/F-SKL-01/V04-01.md) |
| V05 | Historical-document drift check | not-applicable | n/a | No historical document is reused for a claim in this report. The agentskills.io spec is cited only as the documented format contract, not as evidence about historical code. |

All 167 tests in `echo_execution::skills` pass at the audited commit
(see V01-01 for the command and result).

## Historical Claim Status

No historical documents are cited as evidence for any claim in this
report. All findings are based on code at commit `9b0e0fa` /
`b3b2e81` and the four validation reports.

## Coverage And Uncertainty

- Code not inspected:
  - `echo-core/src/tools/skill.rs` — defines the `Skill` trait,
    `SkillInfo`, `is_path_safe`, `minimal_env`,
    `minimal_hook_env_with_context`. Read indirectly through
    re-exports only; the path-safety helper is also exercised by the
    shell tool (F-EXT-02) which audited it. No deep re-read here.
  - `echo-core/src/hooks/*` — `HookEvent`, `HookContext`,
    `HookResult`, `HookSource` definitions are re-exported from
    `echo_execution::skills::hooks` and used as-is. Their producer /
    consumer wiring is partly F-RCT-04's territory; this task audited
    only the skill-side dispatch and merge semantics.
  - `echo-tools::shell::validate_command_safety` /
    `CommandSafety` — used by `check_skill_command_safety`
    (`prompt_exec.rs:585`). The classifier itself is F-EXT-02's scope;
    this task only verified that the skill fallback path consults it.
  - Sandbox internals (`SandboxManager`, `SandboxCommand`,
    `ResourceLimits`) — F-SEC-01 owns the boundary; this task only
    verified the skill-side API surface and the direct fallback.
- Validations not executed at runtime: V01, V02, V03 are static
  inspections (no `cargo test` invocation in the validation report
  itself; the supporting test run is recorded in V01-01). V04 is a
  static inspection of the unload + cancellation path; no live
  script-cancellation fixture was run. A live fault-injection test
  (start a long-running script, drop the future, observe child kill)
  belongs to Q-FLT-01.
- Environmental limits: none. Both repos are clean at the audited
  commits. Tests ran on darwin 25.5.0 arm64.
- Claims that remain uncertain:
  - The actual cross-filesystem non-determinism window for
    F-SKL-01-P2-01 is not demonstrated with a live fixture here; it is
    inferred from the documented behavior of `tokio::fs::read_dir` and
    the absence of any sort step in the loader. Q-FLT-01 could
    exercise it.
  - The sandbox-path script cancellation (`manager.execute(sandbox_cmd)`
    honoring cancellation) depends on the `SandboxManager`
    implementation, which F-SEC-01 must confirm. The skill tool only
    ensures the *direct fallback* path uses `kill_on_drop(true)`.

## Handoff

- Conclusions downstream tasks may rely on:
  - `SkillLoader` is the single framework discovery primitive. There
    is no duplicate loader or registry at the application layer;
    `SkillsHub` is a UI/install index only. X-BND-01 / X-PLG-01 can
    rely on this.
  - The three progressive-disclosure tools (`activate_skill`,
    `read_skill_resource`, `run_skill_script`) are framework-owned
    singletons refreshed on every discovery pass. F-RCT-04 (tool batch
    execution) can rely on their tool contract being stable.
  - Cancellation of skill scripts/commands flows through future-drop
    + `kill_on_drop(true)` on the direct path; the sandboxed path
    delegates to `SandboxManager`. F-SEC-01 owns the sandbox
    cancellation contract.
  - Hooks (`HookRegistry`, `HooksDefinition`) are a framework
    facility; skills, plugins, and user-config register through the
    same dispatcher with a documented source-precedence order
    (`UserConfig` > `Plugin` alphabetical > `Skill` alphabetical).
    F-HITL-01 and F-PLG-01 can rely on this.
  - `SkillSource::Mcp` short-circuits command execution — MCP-sourced
    skills never execute inline shell blocks. F-INT-01 can rely on
    this boundary.
- Reports they must read:
  - [V01-01](../validations/F-SKL-01/V01-01.md) for the discovery
    walk, scope expansion, and dependency DFS.
  - [V02-01](../validations/F-SKL-01/V02-01.md) for the malformed /
    path-traversal containment checks.
  - [V03-01](../validations/F-SKL-01/V03-01.md) for the tool-name
    collision resolution.
  - [V04-01](../validations/F-SKL-01/V04-01.md) for the unload +
    cancellation path.
- Conditions that make this report stale:
  - Any change to `loader.rs::scan_directory` (sort, depth cap,
    symlink handling) invalidates V01 and F-SKL-01-P2-01 / P3-03.
  - Any change to `registry.rs::register_descriptor`'s overwrite
    branch invalidates F-SKL-01-P2-02 and V04.
  - Any change to the three progressive-disclosure tool names or to
    `ToolManager::register`'s overwrite behavior invalidates V03 and
    F-SKL-01-P3-02.
  - Introduction of a framework-level filesystem watcher for skills
    invalidates the "Reload" current-path paragraph and F-SKL-01's
    V04 claim that reload is application-driven.
- Follow-up task IDs (no fixes implemented in this review):
  - F-PLG-01 should verify the plugin adapter's
    `unregister_skills_by_source` + `load_plugin_skills_from_dir`
    sequence is the only skill reload path and that it leaves no
    stale registrations on failure (relates to F-SKL-01-P2-02's
    overwrite-cleanup defect).
  - A-PLG-01 should verify that the EKO `SkillsHub` UI reflects the
    framework registry state and does not maintain a divergent
    "loaded" set.
  - Q-FLT-01 should run a fault-injection fixture for the
    within-scope name-collision non-determinism (F-SKL-01-P2-01) and
    for live script-cancellation on agent cancel.
