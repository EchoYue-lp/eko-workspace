# F-SKL-01: Skill loading and execution

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean (probe crate `/tmp/fskl-probe` is outside both repos)

## Question

Are skill discovery, frontmatter, dependency probing, prompt/script execution, source identity, hooks, and reload behavior deterministic?

## Scope

- `echo-execution/src/skills/` (full reads): `external/types.rs` (descriptor/frontmatter/sandbox policy/tool matcher), `external/loader.rs` (discovery scopes, parse, precedence, dependency validation), `external/activate_tool.rs`, `external/resource_tool.rs`, `external/run_script_tool.rs`, `external/prompt_exec.rs` (inline command + variable substitution), `registry.rs` (activation, unload, source identity, baseline injection), `dependency_probe.rs`, `hooks.rs` (hook engine: actions, matchers, registry, command/http/mcp/agent execution, output parsing, result merge — core engine read fully, test section sampled).
- `echo-core/src/tools/skill.rs` (Skill trait, `is_path_safe`, `minimal_env*`).
- Root facade `echo-agent/src/skills/mod.rs` (re-export only).
- Agent wiring: `echo-agent/src/agent/react/capabilities.rs:560-1068`, `snapshot.rs:195-300,604`, `react/mod.rs:1690-1715,2007`, `react/run/context.rs:440-490`, `react/run/{react_loop,stream_channel}.rs` (activation call sites), `src/plugin.rs:250-410`, `src/hooks_bridge.rs`, `echo-execution/src/tools.rs` (register/unregister), `echo-core/src/tools/mod.rs` (ToolVisibilityState).
- EKO: `echo-agent-cli/echo-agent-app-core/src/skills_hub/{registry,enabled_skills,install}.rs`, `runtime.rs:148-268`, `state.rs:975-1010,1120-1160`, `agent_pool.rs:905-935`, `plugin_runtime.rs:1060-1180`, `src/cli/cmd_impls/skills.rs`.
- Executed tests: `cargo test -p echo_execution --lib --locked skills`, `cargo test -p echo_core --lib --locked skill`, `cargo test -p echo_agent --lib --locked skills`, `cargo test -p echo-agent-app-core --lib --locked skills_hub`; standalone probes for cyclic dependencies, YAML-list metadata, scope precedence, and frontmatter terminator edges (probe source `/tmp/fskl-probe/src/main.rs`).

## Out Of Scope

- Plugin lifecycle itself (manifest/activation/rollback) -> F-PLG-01 (skill load/unload integration points cross-referenced only).
- Checkpoint persistence internals and steer/resume semantics -> F-RCT-05 (only the `active_skills` mark asymmetry is anchored here).
- Hook/pipeline integration beyond the skill hook engine (PreToolUse etc. execution order) -> F-RCT-04.
- `echo-core/src/plugin/*` manifest/variable machinery beyond `PluginVariables` substitution used by the loader.
- Evolution "skill" concept (memory-layer skill extraction) — different concept, no overlap found.
- Frontend skill UI projections -> A-FE-*/A-PLG-01.

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md` (F-SKL-01 card), `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-EXT-01` (complete), `F-RCT-01` (complete).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md` (skill claims at lines 266-269, 991, 997, 1002, 1009), `echo-agent/README.md` (skill API/claims at :220, :686) — classified in the Historical Claim Status section.

## Layering Decision

- Generic mechanism (framework, `echo_core`/`echo_execution`/`echo_agent` facade): `Skill` trait, `SkillDescriptor`/`SkillContent`/frontmatter parsing, `SkillLoader` (discovery + `SkillLoadPolicy` trait as product hook point), `SkillRegistry` (activation/unload/source identity), the three progressive-disclosure tools, `dependency_probe`, `PromptContext`/inline-command engine, `HookRegistry`/`HookAction`/matchers/`merge_result`. All correctly placed; the framework is independently usable (examples `demo07/08/09`).
- EKO product policy (application): `skills_hub` marketplace (`~/.eko/skills/` scan, search, install/sync), `enabled-skills.json` enablement/baseline config, the `ReviewIntegration` implementation of `SkillLoadPolicy`/curator, CLI/GUI skill commands. All correctly placed.
- Adapter boundary: thin and correct — EKO implements framework traits (`SkillLoadPolicy`, curator) and calls framework load/unload APIs; no second scheduling/state authority in EKO.
- Duplicate search terms (both repositories): `discover_skills`, `SkillLoader`, `SkillRegistry`, `SkillDescriptor`, `add_skill`, `SkillLoadPolicy`, `register_descriptor`, `unregister_by_source`, `activate_skill`, `read_skill_resource`, `run_skill_script`, `parse_frontmatter`, `extract_body`/`strip_frontmatter`/`extract_instructions`, `binary_available`, `missing_binary_names`, `SKILL.md`, `hooks.json`, `inject_methodology_baseline`, `enabled_skills`. Results: one framework runtime authority; five frontmatter parse/strip implementations (P3-01); two binary-probing implementations (P3-02); two `SkillRegistry` instances per agent (P1-02); no EKO-side skill runtime duplicate.

## Current Path

Verified data flow (anchors in V02-01): discovery (`capabilities.rs:631-805`) -> loader parse + `SkillLoadPolicy` gate -> descriptors registered into **two** registries (tracking `tools.skill_registry` + shared `progressive_skill_registry`), frontmatter `hooks.json` merge (`loader.rs:227-262`) and hook registration, catalog as replaceable projection `echo-agent:skill-catalog`, `activate_skill`/`read_skill_resource`/`run_skill_script` tool replacement -> model calls `activate_skill` (`activate_tool.rs:96-186`) -> `SkillRegistry::activate_with_args` (deps first, inline commands via `process_skill_content`, resources enumerated, sandbox policy stored) -> instructions injected as tool result; enforcement of `allowed-tools` at snapshot (`snapshot.rs:206-299`); hooks fired from pipeline/react_loop/hooks_bridge with deterministic source ordering (`hooks.rs:711-722`); unload via `unregister_skills_by_source` (`capabilities.rs:915-958`) from plugin disable (`plugin_runtime.rs:1167`). EKO live loaders: builtin (`runtime.rs:148-168`), workspace/global (`state.rs:994,1143`), pool (`agent_pool.rs:925`), plugin `wire_all` (`plugin_runtime.rs:818-854`).

## Findings

### F-SKL-01-P1-01: Circular (or self-referential) `depends_on` causes unbounded recursion — stack overflow — process abort on activation

- Priority: P1
- Confidence: high (empirically reproduced against the real crate)
- Layer: framework
- Evidence: `echo-execution/src/skills/registry.rs:370-377` (deps activated before self is marked), `:468-510` (`activate_dependencies` guards only against the *completed* `activated` set — no in-progress guard), `:441-444` (self-mark happens after deps); loader `loader.rs:492-551` (`detect_cycle` logs a warning but does not remove cycle edges) and `loader.rs:489-491` (doc: cycles "are handled at activation time" — false).
- Reachability: model calls `activate_skill` on such a skill (`activate_tool.rs:163-169`) or a hook `ActivateSkill` action fires (`context.rs:463-480`); EKO loads skills from builtin/workspace/global/plugin dirs (`runtime.rs:156`, `state.rs:994/1143`, `plugin_runtime.rs:818`), so any discovered skill with a typo'd/self dependency crashes the app process.
- Expected invariant: cyclic/missing dependencies are detected at discovery and degrade gracefully (warning), never crash the process.
- Observed behavior: `SkillRegistry::activate("a")` on `a<->b` mutual deps recurses without bound; probe run aborts with `thread 'main' has overflowed its stack / fatal runtime error: stack overflow, aborting` (exit 134). A single skill with `depends_on: [itself]` triggers the same.
- Impact: unrecoverable agent/application crash (Rust stack overflow aborts the process) triggered by any user- or plugin-authored skill with a cyclic dependency — deterministic, no error surfaced.
- Root cause: discovery-time cycle detection is log-only ("handled at activation time" was never implemented in `activate_dependencies`); the completion-marked-after-deps ordering leaves no way for the dedup set to break the recursion.
- Direction: add an in-progress set (or depth cap) in `activate_dependencies` and return an error on cycle; optionally break cycle edges in `validate_and_sort_dependencies`; rename it (`validate_dependencies` — it sorts nothing).
- Regression validation: unit test registering `a->b->a` and a self-dependency, asserting `activate("a")` returns an error and leaves the process alive; keep all existing skills tests green (V04-01).
- Validation reports: [V04-05](../validations/F-SKL-01/V04-05.md)

### F-SKL-01-P1-02: Dual `SkillRegistry` state divergence — checkpoint resume marks the tracking registry only, so `read_skill_resource`/`run_skill_script` reject after a fresh-process resume

- Priority: P1
- Confidence: medium (static chain fully verified; fresh-process resume was not executed dynamically)
- Layer: framework (wiring in `capabilities.rs`/`mod.rs`)
- Evidence: two registries created and fed identically (`capabilities.rs:660-677` tracking `tool_exec.rs:34` vs shared `progressive_skill_registry`, both registered at `:715-718`); snapshot save collects active names from **both** (`snapshot.rs:206-222`); resume marks only `self.tools.skill_registry` (`react/mod.rs:1703-1704`); the three tools check only the shared registry's `activated` set (`activate_tool.rs:150,191`, `resource_tool.rs:100`, `run_script_tool.rs:191`); hook-driven activation marks only the tracking registry (`capabilities.rs:964-997`).
- Reachability: EKO persists `AgentCheckpoint`s including `active_skills` (`infra.rs:1242`) and restores via `resume_from_state_store` (`react/mod.rs:1690-1712`); after an app restart with checkpoint restore, skills activated via the model tool are in `cp.active_skills` but the fresh shared registry is never marked.
- Expected invariant: activation state is a single authority; after resume, all previously activated skills behave identically (resource reads, script runs) and no content duplication occurs.
- Observed behavior: after fresh-process resume, `read_skill_resource`/`run_skill_script` return "Skill 'X' has not been activated"; if the model re-activates, instructions are injected a second time (tool result + projection paths).
- Impact: recovery inconsistency — the flagship resume path silently breaks Tier-3 skill access and can duplicate instructions; the framework's own save/restore contract (save both, restore one) is asymmetric.
- Root cause: two registries were introduced to share state with async tools (`capabilities.rs:659-677`) without reconciling all activation writers and the resume reader onto one authority; `capabilities.rs:962-964`'s comment ("All activation paths should use this method so registry state and model context cannot diverge") is not enforced.
- Direction: single authority — have resume mark the shared registry too, or have the tools consult a merged state, or drop the tracking registry and route `capabilities::activate_skill` through the shared one; add a save/restore round-trip test asserting both registries agree after restore.
- Regression validation: unit test — activate via `ActivateSkillTool`, snapshot, rebuild agent + restore checkpoint, assert `run_skill_script`/`read_skill_resource` succeed without re-activation and context contains one instruction copy.
- Validation reports: [V02-01](../validations/F-SKL-01/V02-01.md)

### F-SKL-01-P2-01: Re-discovery of an existing skill is a silent no-op — `/skills sync` and workspace reload leave descriptors, hooks, and catalog stale until restart

- Priority: P2
- Confidence: high (static; no dynamic reload run)
- Layer: framework (skip logic) + application (CLI reload expectation)
- Evidence: `capabilities.rs:687-695` (`is_installed` -> "already installed, skipping duplicate", no content comparison), `:702-711` (hooks registered only for new names), `:734-736` (early return without catalog refresh when nothing new); CLI `/skills sync` then calls `load_skills_from_dir(root)` expecting a runtime reload (`echo-agent-cli/src/cli/cmd_impls/skills.rs:61-71`); plugin skills avoid the issue only because disable unregisters first (`plugin_runtime.rs:1167`).
- Reachability: any user skill updated in place (git sync, manual edit) while the agent is running, then `/skills sync` or a workspace-switch reload (`state.rs:994`).
- Expected invariant: reload reflects current SKILL.md content (instructions, allowed-tools, hooks, catalog description).
- Observed behavior: updated content never reaches the runtime; the reload call returns success with zero new names.
- Impact: the agent executes stale skill instructions and stale hooks after an explicit sync/reload; the sync command reports success without indicating the runtime did not update.
- Root cause: "skip duplicates" was designed as a discovery-time shadow guard and reused as a reload strategy; no content-hash comparison or replace path exists for non-plugin skills.
- Direction: on re-discovery, detect changed content and replace descriptor/hooks in place (mirror `register_descriptor`'s replace semantics + hook re-registration), or expose an explicit `refresh_skills` API and make the CLI call it; add a test: discover, edit SKILL.md, re-discover, assert new description/instructions in registry and catalog.
- Regression validation: `cargo test -p echo_agent --lib` skills tests plus a new reload test; keep `discover_skills_refreshes_activate_skill_registry` green (it pins add-only semantics).
- Validation reports: [V03-02](../validations/F-SKL-01/V03-02.md), [V04-03](../validations/F-SKL-01/V04-03.md)

### F-SKL-01-P2-02: Baseline methodology skill injection into the system prompt is order-nondeterministic (two HashMap iterations)

- Priority: P2
- Confidence: high (code facts; cross-process variance not executed)
- Layer: application (enabled-skills) + framework (injection)
- Evidence: `enabled_skills.rs:110-116` (`enabled_baseline_names` iterates `self.skills` HashMap — nondeterministic order), `registry.rs:615` (`inject_methodology_baseline` iterates `self.descriptors.values()` HashMap — nondeterministic order), consumer chain `runtime.rs:180-204` (`enabled_baseline_names` -> `inject_methodology_baseline` -> `set_system_prompt`); MASTER-PLAN:269 acceptance "相同 workspace/skills 下连续 turn 的稳定前缀 hash 不变" holds only in-process.
- Reachability: every EKO session startup with methodology skills enabled (default: 4 core skills).
- Expected invariant: identical inputs produce a byte-identical system prompt (deterministic prompt assembly per F-RCT-01).
- Observed behavior: the `<skill>` baseline blocks are ordered by per-process randomized HashMap iteration; two processes with identical skill sets get different prompt bytes.
- Impact: prompt-level nondeterminism (LLM output can differ across runs); the MASTER-PLAN stable-prefix acceptance is only in-process; debugging prompt diffs across restarts is confusing.
- Root cause: both iteration sites use `HashMap` without sorting (the catalog projection, by contrast, sorts at `registry.rs:274-280` and `loader.rs:298-309`).
- Direction: sort names in `enabled_baseline_names` and sort descriptors in `inject_methodology_baseline`; add a determinism test asserting identical baseline prompts across two registry instances.
- Regression validation: unit test constructing two `SkillRegistry`s with the same descriptors and asserting equal `inject_methodology_baseline` output.
- Validation reports: [V05-01](../validations/F-SKL-01/V05-01.md)

### F-SKL-01-P2-03: YAML-list values in `metadata` silently drop the whole skill from discovery; the `dependency_probe.rs` documented format is wrong and EKO's hub disagrees with the framework on the same file

- Priority: P2
- Confidence: high (empirically reproduced)
- Layer: framework (parse contract) + application (hub inconsistency)
- Evidence: `loader.rs:448` (`serde_yaml_ng::from_str` into `RawFrontmatter` with `metadata: Option<HashMap<String,String>>`); probe: `requires-binaries: [soffice, pdftoppm]` -> `invalid type: sequence, expected a string` -> skill skipped (`loader.rs:270-276`); `dependency_probe.rs:52-65` comment claims lists "come through as `[soffice, pdftoppm]`" (empirically false); EKO hub's naive parser accepts the same line and probes broken names `[soffice`/`pdftoppm]` (`skills_hub/registry.rs:275-287,331-388`).
- Reachability: any skill whose frontmatter uses YAML list syntax for `requires-binaries` (or any metadata key) — a natural authoring form; EKO marketplace (`SkillsHub::scan`) lists the skill while the agent never loads it.
- Expected invariant: documented metadata formats parse; malformed input degrades per-document, not whole-skill.
- Observed behavior: the entire skill is excluded from the catalog with only a `warn!` log; hub and agent surfaces diverge on the same directory.
- Impact: silent capability loss (skill never activates) with a misleading in-code doc and cross-surface inconsistency.
- Root cause: `metadata` typed as `HashMap<String,String>` with no tolerant deserialization; the probe helper's doc was written against intended behavior, not actual serde_yaml_ng behavior.
- Direction: deserialize metadata values with `#[serde(untagged)] String|Vec<String>` (or a custom visitor) and add a loader test for both forms; fix the `dependency_probe.rs` comment; make the hub reuse the framework parser to converge surfaces.
- Regression validation: loader test with `requires-binaries: [soffice, pdftoppm]` and comma-string form asserting both discover and `extract_dependencies` yields both binaries.
- Validation reports: [V04-06](../validations/F-SKL-01/V04-06.md), [V03-01](../validations/F-SKL-01/V03-01.md)

### F-SKL-01-P3-01: Five parallel frontmatter parse/strip implementations with divergent edge behavior

- Priority: P3
- Confidence: high
- Layer: framework + application
- Evidence: `loader.rs:407-483` (`parse_frontmatter` strict + `extract_instructions` lenient), `registry.rs:657-709` (`strip_frontmatter`/`extract_body` lenient, empty-on-missing-terminator), `skills_hub/registry.rs:331-388` (naive line parser, no terminator-own-line check, name falls back to dir name).
- Reachability: loader path, activation path, baseline injection path, and EKO marketplace scan each use a different parser; the hub can list a skill the loader rejects (P2-03 observed exactly this divergence).
- Expected invariant: one frontmatter authority with one documented edge contract (AGENTS.md: 严禁平行实现同一语义).
- Observed behavior: strictness differs (closing-`---` trailing content, missing description, name fallback); fixes must be applied in five places.
- Impact: maintainability and cross-surface behavior divergence (P2-03's concrete instance).
- Root cause: parsers evolved independently (legacy format support, plugin variables, hub's no-crate-coupling choice).
- Direction: single parser in `echo_execution` (public `parse_skill_md`) used by registry internals and the EKO hub (hub already depends on `echo_agent`); delete the naive hub parser and the registry-local strippers.
- Regression validation: keep existing loader/registry tests green; hub scan test asserting parity with loader acceptance on the same directory.
- Validation reports: [V01-01](../validations/F-SKL-01/V01-01.md), [V04-04](../validations/F-SKL-01/V04-04.md)

### F-SKL-01-P3-02: Duplicate binary-probing implementations with divergent list parsing (framework `dependency_probe` vs EKO hub inline copy)

- Priority: P3
- Confidence: medium
- Layer: application (duplicate) over framework (authority)
- Evidence: `dependency_probe.rs:101-109` (`binary_available`) and `:56-65` (`parse_metadata_list` strips brackets); `skills_hub/registry.rs:314-325` (explicitly inlined copy, comment: "此处 inline 一份避免 registry 反向依赖 echo_execution 的类型") and `:275-287` (hub split does not strip brackets).
- Reachability: EKO hub scan (`SkillsHub::scan` -> `missing_dependencies`) vs framework probe consumers; on the same comma-string input both behave the same, on the YAML-list input they diverge (framework: skill dropped; hub: false "missing" warnings for `[soffice`).
- Expected invariant: one probing implementation per semantic (AGENTS.md).
- Observed behavior: two implementations with different edge behavior.
- Impact: false "missing dependency" UI warnings and divergent parsing; future fixes duplicated.
- Root cause: hub avoided the framework type dependency though `echo_agent` re-exports `missing_binary_names` publicly and the app already depends on `echo_agent`.
- Direction: call `echo_agent::skills::dependency_probe::missing_binary_names` from the hub and delete the inline copy + `binary_available`.
- Regression validation: hub scan fixture with a known missing binary asserting the same result via both paths.
- Validation reports: [V01-01](../validations/F-SKL-01/V01-01.md)

### F-SKL-01-P3-03: `SkillRegistry::list()` documents a unified view but returns code skills only

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `registry.rs:586-594` (`list` -> `list_code_skills`, doc "List all installed skills as `SkillInfo` (unified view)"; `get` likewise); consumers `capabilities.rs:1019-1026` (`list_skills` -> `list`).
- Reachability: `list_skills` used by any surface listing installed skills; file-based skills absent from the result despite `is_installed`/`count` including them.
- Expected invariant: API docs match behavior.
- Observed behavior: doc/behavior mismatch; a consumer listing skills misses all file-based skills.
- Impact: misleading public surface (EKO currently uses the hub for listings, so in-repo impact is latent).
- Root cause: `SkillInfo` predates file-based descriptors; the unified query was never implemented.
- Direction: either add descriptor-based entries to `list()` or rename/document it as code-skills-only.
- Regression validation: unit test asserting `list()` covers both kinds (or the documented subset).
- Validation reports: [V01-01](../validations/F-SKL-01/V01-01.md)

### F-SKL-01-P3-04: Skill tool registration silently overwrites colliding tool names (same class as F-EXT-01-P1-02, skill-specific entry point)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `capabilities.rs:583-585` (`add_skill` registers each skill tool via `tool_manager.register` with no collision check); `echo-execution/src/tools.rs:529-536` (silent last-wins insert); skill-level dedup only by skill name (`capabilities.rs:566-573`).
- Reachability: two skills (or a skill and a builtin) providing the same tool name; the second silently replaces the first.
- Expected invariant: duplicate tool registration is observable (F-EXT-01-P1-02 direction) at every entry point.
- Observed behavior: silent overwrite with no log; model may call the wrong implementation.
- Impact: same as F-EXT-01-P1-02 — invisible wrong-binding risk, here reachable through skill packs.
- Root cause: `ToolManager::register` last-wins and `add_skill` never checks tool-name overlap.
- Direction: fold into the F-EXT-01-P1-02 fix (observable registration), plus an `add_skill`-level overlap check.
- Regression validation: unit test adding two skills with one shared tool name asserting a warning/rejection.
- Validation reports: [V03-02](../validations/F-SKL-01/V03-02.md)

### F-SKL-01-P3-05: No-sandbox script/inline-command execution ignores the declared sandbox-policy timeout and cannot kill process trees on timeout

- Priority: P3
- Confidence: medium (static; derived from `kill_on_drop` semantics)
- Layer: framework
- Evidence: policy timeout applied only in the sandbox path (`run_script_tool.rs:287-295`), direct fallback uses only `self.timeout_secs` (`:313-354`); `kill_on_drop(true)` + `tokio::time::timeout` kill only the direct child (`run_script_tool.rs:319,332`, `prompt_exec.rs:426,436`, `hooks.rs:1178,1222`); policy doc "tool execution within this skill's context is constrained according to the policy" (`types.rs:139-143`); `RunSkillScriptTool` implements `execute` (no `ToolContext`), so `ctx.cancel` is unobservable (`run_script_tool.rs:158`).
- Reachability: skill scripts/inline commands run without a configured `SandboxManager` (EKO wires a sandbox on its main agent, so the direct path is a framework-consumer fallback and demo path).
- Expected invariant: declared policy timeout is honored on all execution paths; timeout/cancel terminates the process tree.
- Observed behavior: `bash -c "sleep 100 & echo ok"`-style commands leave orphaned grandchildren after the timeout kills only the shell; policy `timeout_secs` has no effect without a sandbox.
- Impact: resource leaks and unenforced declared policy on the fallback path; silent divergence between sandboxed and bare runs.
- Root cause: the sandbox abstraction owns tree-kill; the direct fallback was written as a best-effort copy without process-group handling, and policy application was not ported.
- Direction: apply `SkillSandboxPolicy.timeout_secs` in the direct path; use process-group kill (`killpg`/setsid) on timeout for all three direct paths; implement `execute_with_context` so `ctx.cancel` is honored; document the fallback limitation (the existing ⚠️ comment at `prompt_exec.rs:396-408` partially covers it).
- Regression validation: timeout fixture with `bash -c "sleep 60 & echo ok"` asserting the child group is gone after timeout; policy-timeout fixture on the direct path.
- Validation reports: [V03-02](../validations/F-SKL-01/V03-02.md)

### F-SKL-01-P3-06: README skill claims are stale (`load_skill` API and pre-built skill pack names do not exist)

- Priority: P3
- Confidence: high
- Layer: application/documentation (framework README)
- Evidence: `echo-agent/README.md:220` ("`agent.load_skill(\"web_research\")`" — grep for `load_skill` across the repo: zero matches), `:686` ("Pre-built skills: `code_review`, `data_analyst`, `project-stats`, `python-linter`, `web_researcher`" — no such packs under `echo-agent/skills/`, which does not exist; builtin code skills are `filesystem`/`shell`).
- Reachability: framework users following the README.
- Expected invariant: documented APIs and artifacts exist.
- Observed behavior: the documented single-skill API and pack names do not exist; the real API is `load_skills_from_dir`/`discover_skills`.
- Impact: misleading framework documentation.
- Root cause: README written against an earlier skill API/pack set before the agentskills.io rewrite.
- Direction: update README to `load_skills_from_dir`/`discover_skills` and current pack names (or remove the pack list).
- Regression validation: none required beyond doc review (Q-DOC-01).
- Validation reports: [V05-01](../validations/F-SKL-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search across both repositories | yes | passed | [V01-01](../validations/F-SKL-01/V01-01.md) |
| V02 | Registration and runtime reachability trace | yes | passed | [V02-01](../validations/F-SKL-01/V02-01.md) |
| V03 | Invariants — discovery precedence, malformed/frontmatter/path, YAML-list metadata | yes | passed | [V03-01](../validations/F-SKL-01/V03-01.md) |
| V03 | Invariants — tool name collision, reload/unload, script cancellation | yes | passed | [V03-02](../validations/F-SKL-01/V03-02.md) |
| V04 | `cargo test -p echo_execution --lib --locked skills` | yes | passed (exit 0, 167 passed) | [V04-01](../validations/F-SKL-01/V04-01.md) |
| V04 | `cargo test -p echo_core --lib --locked skill` | yes | passed (exit 0, 5 passed) | [V04-02](../validations/F-SKL-01/V04-02.md) |
| V04 | `cargo test -p echo_agent --lib --locked skills` | yes | passed (exit 0, 3 passed) | [V04-03](../validations/F-SKL-01/V04-03.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked skills_hub` | yes | passed (exit 0, 6 passed) | [V04-04](../validations/F-SKL-01/V04-04.md) |
| V04 | Probe: cyclic dependency activation | yes | failed (exit 134, stack overflow) | [V04-05](../validations/F-SKL-01/V04-05.md) |
| V04 | Probe: YAML-list metadata parsing | yes | failed (skill dropped) | [V04-06](../validations/F-SKL-01/V04-06.md) |
| V04 | Probe: discovery scope precedence | yes | passed (exit 0) | [V04-07](../validations/F-SKL-01/V04-07.md) |
| V04 | Probe: frontmatter terminator edges | yes | passed (exit 0) | [V04-08](../validations/F-SKL-01/V04-08.md) |
| V05 | Historical-document drift check | conditional | passed | [V05-01](../validations/F-SKL-01/V05-01.md) |

All required validations executed; every reported command has a known exit code; two executions failed and became findings (V04-05 -> P1-01, V04-06 -> P2-03).

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:266-269 skill catalog marker/latest-wins + "稳定前缀 hash 不变" acceptance | current (in-process) | `SKILL_CATALOG_PROJECTION` replace-only, single projection pinned by test; cross-process order nondeterminism = F-SKL-01-P2-02; [V05-01](../validations/F-SKL-01/V05-01.md) |
| MASTER-PLAN:991 "SkillRegistry 无来源追踪" (old gap) | fixed | `source` field + `by_source` + `unregister_by_source` + `tag_source` (`registry.rs:101-190,133-151`), plugin unload wired, tests green; [V02-01](../validations/F-SKL-01/V02-01.md), [V04-01](../validations/F-SKL-01/V04-01.md) |
| MASTER-PLAN:997 plugin disable/uninstall unregisters skills+hooks | current | `plugin_runtime.rs:1167` + `capabilities.rs:915-958`; [V02-01](../validations/F-SKL-01/V02-01.md) |
| MASTER-PLAN:1002 `load_plugins` thin adapter via `PluginIntegrator::wire_all` | current | `plugin_runtime.rs:818-854`; [V02-01](../validations/F-SKL-01/V02-01.md) |
| MASTER-PLAN:1009 `wire_skills`/`wire_mcp` single-component entries retained | current | `plugin.rs:398-410`; [V02-01](../validations/F-SKL-01/V02-01.md) |
| README:220 `agent.load_skill(...)` API | stale | no `load_skill` anywhere; real API `load_skills_from_dir`/`discover_skills`; [V05-01](../validations/F-SKL-01/V05-01.md) |
| README:686 pre-built skill pack names | stale | `echo-agent/skills/` does not exist; builtin code skills are `filesystem`/`shell`; [V05-01](../validations/F-SKL-01/V05-01.md) |
| `dependency_probe.rs:52-65` YAML-list metadata doc | regressed | YAML list fails parse and drops the skill; [V04-06](../validations/F-SKL-01/V04-06.md) |

## Coverage And Uncertainty

- `hooks.rs` was read for the core engine (actions, matchers, registry, execution, parsing, merge); the full 2925-line file's test section was sampled only. No hook-execution finding was produced because source ordering is deterministic; command-hook fail-open-on-timeout (`hooks.rs:1246-1254` returns default/no-block) was noted but not promoted (local-trust model, user-authored hooks).
- The P1-02 resume divergence was not executed dynamically (requires the EKO checkpoint-restore flow, F-RCT-05 scope); the static chain is complete and the asymmetry is unambiguous.
- The P2-01 reload no-op and P2-02 cross-process prompt-order variance were not executed dynamically (no fixture harness in a read-only review); both are direct code-fact chains.
- Nested same-name duplicates within one directory resolve by `read_dir` order (filesystem-dependent); recorded as a note in V03-01, not a standalone finding.
- `install.rs` git mechanics (clone/update/force) were read at structure level only; they are marketplace policy, not runtime determinism, and their edge behavior belongs to A-PLG-01.
- In-process `HashMap` iteration is stable for the same map instance, so within-session prompt stability claims hold; only cross-process ordering varies.

## Handoff

- Downstream tasks may rely on: single framework skill runtime authority with EKO product policy on top (V01); full registration/reachability chain incl. the two-registry wiring (V02); deterministic scope precedence and path containment, with the two empirically reproduced defects P1-01/P2-03 (V03/V04); green skill test suites on both repos (V04-01..04).
- `F-PLG-01`: plugin skill load/unload integration points (`plugin.rs:265,398-410`, `plugin_runtime.rs:818-854,1167`); the P2-01 refresh gap means plugin disable-then-reload is currently the only correct refresh path.
- `F-RCT-05`: checkpoint resume must reconcile the skill activation authorities (P1-02 direction); `active_skills` save/restore round-trip belongs to its fixture set.
- `X-PLG-01` / `A-PLG-01`: reload/unload determinism findings (P2-01) and hub/framework parser divergence (P2-03, P3-01) are cross-repository lifecycle-conformance input.
- `Q-DOC-01`: README skill API/pack claims (P3-06), `dependency_probe` doc (P2-03).
- `X-BND-01`: record the five-parser duplication (P3-01) and the binary-probe duplication (P3-02) as duplicate-authority items; the `SkillLoadPolicy` adapter boundary is an example of correct thin application policy.
- Reports to read: this report + [V01-01](../validations/F-SKL-01/V01-01.md) through [V05-01](../validations/F-SKL-01/V05-01.md); F-EXT-01 (P1-02 tool-registry collision class, P3-04 cross-ref), F-RCT-01 (prompt assembly determinism context for P2-02).
- Stale triggers: changes to `echo-execution/src/skills/*` (loader/registry/hooks/execution), `echo-agent/src/agent/react/capabilities.rs` skill section, `snapshot.rs` skill-state capture, `react/mod.rs` resume path, `echo-agent-cli` `skills_hub/*` or `runtime.rs` skill bootstrap invalidate the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): F-PLG-01, F-RCT-05, X-PLG-01, A-PLG-01, X-BND-01, Q-DOC-01.
