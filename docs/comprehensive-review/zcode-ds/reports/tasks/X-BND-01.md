# X-BND-01: Capability placement and duplicate authority map

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline)
> Worktree state: clean (both repositories; `git status --short` empty)

## Question

Across both repositories, which concepts are correctly framework, EKO policy,
or thin adapters, and where do semantic duplicates remain?

**Answer: the framework/app/adapter split is correct for every core concept;
the remaining semantic duplicates are (a) framework-internal dead parallel
authorities (retry, diff, URL-download, frontmatter, risk classification,
context assembly, handoff, approval, notebook), (b) a small set of
application-side copies (three project-root resolvers, safe_segment copy,
three diff engines, save_trace, second export_conversation), and (c) inert
schema/config fields accepted but never enforced (`execution_mode:
sequential`). No cross-repository live second engine exists for any concept:
every EKO "implementation" is a projection, policy hook, or thin adapter over
a single framework authority. Three recorded-but-unfiled divergences become
new findings (X-BND-01-P2-01, X-BND-01-P2-02, X-BND-01-P3-01); everything else
references canonical IDs from the completed F-*/A-* reports.**

## Scope

Primary paths inspected:

- `echo-agent` root + 7 sub-crates: task graph (`echo-orchestration/src/tasks/`,
  `planning/`), subagent (`src/agent/subagent/`, `src/handoff/`, `src/topology.rs`),
  permission (`echo-core/src/tools/permission.rs`, `src/agent/react/run/approval.rs`),
  memory/context (`echo-state/src/memory/`, `echo-state/src/compression/`,
  `src/context/`, `echo-core/src/compression.rs`, `echo-core/src/retry.rs`,
  `src/agent/react/run/retry.rs`), skills (`echo-execution/src/skills/`),
  plugin (`echo-core/src/plugin/`, `src/plugin.rs`), MCP (`echo-integration/src/mcp/`),
  workflow (`echo-orchestration/src/workflow/`), intent (`src/intent/`),
  notebook (`src/notebook/`), diff (`echo-tools/src/files/diff.rs`),
  URL tools (`echo-tools/src/web/`, `media/`, `research/`, `pdf.rs`),
  instructions (`echo-core/src/project_rules.rs`).
- `echo-agent-cli` (EKO): app-core `tasks/task_runtime/`, `hitl/`,
  `unified_memory.rs`, `instruction_provider.rs`, `subagent_loader.rs`,
  `subagent_prompt.rs`, `skills_hub/`, `plugin_runtime.rs`, `diff.rs`,
  `project/`, `utils.rs`, `evolution/review_integration.rs`, `state.rs`,
  `infra.rs`, `model_config.rs`, `agent_pool.rs`, `runtime.rs`, `observability/`,
  `export/`; tauri `commands/files.rs`, `commands/panels.rs`.

## Out Of Scope

- Behavioral defects inside correctly placed components (each owned by its
  F-*/A-* task; referenced by canonical ID only).
- Frontend store architecture (A-FE-01/02/03), surface parity (X-SRF-01),
  event conformance (X-EVT-01), persistence identity (X-STA-01), tool schema
  conformance (X-TOL-01), permission boundary (X-AUT-01), skill/plugin
  lifecycle conformance (X-PLG-01), task graph conformance (X-TSK-01),
  memory/compression conformance (X-MEM-01).
- Codex and zcode-glm reviewer directories (not read per instructions).

## Inputs

- Repository root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`,
  `TASKS.md` (X-BND-01 card), `zcode-ds/README.md`, report templates.
- Dependency reports read (Layering Decision + duplicate-search sections):
  B-ARCH-01, F-TSK-01/02/03, F-SUB-01/02, F-HITL-01, F-EXT-01/02/03,
  F-MEM-01/02, F-SKL-01, F-PLG-01, F-MAG-01, F-WFL-01, F-NBK-01,
  F-INTENT-01, F-CMP-01, F-CTX-01, F-REL-01, F-LLM-01/03, F-RCT-02,
  F-CORE-01, F-API-01, F-OPS-01, F-FEAT-01, F-MAC-01, F-EVO-01, F-TST-01,
  F-SEC-01, F-INT-01/02, A-TSK-01..06, A-TOOL-01, A-HITL-01, A-SUB-01,
  A-MEM-01, A-CFG-01, A-PROJ-01, A-OBS-01, A-OUT-01, A-EVO-01, A-STATE-01,
  A-BOOT-01, A-CHAT-01, A-SRF-01/02/03/04, A-FE-01/02/03, A-PLG-01, A-INT-01,
  A-INP-01, A-DOM-01.
- Historical documents treated as hypotheses: `echo-agent-cli/docs/MASTER-PLAN.md`
  convergence sections (Phase 3 legacy task surface), README facade claims.

## Layering Decision (final placement map)

Verdict per concept — "framework-correct / application-correct /
adapter-correct" — with the authority owner and the remaining duplicates
(canonical IDs; full deletion-target matrix below).

| Concept | Authority (owner) | EKO side | Adapter | Duplicates (canonical ID) | Verdict |
|---|---|---|---|---|---|
| Task graph | `TaskSpec`/`PlanSpec`/`TaskRevisionService`/`PlanValidator`/`RuntimeDagExecutor` (echo-orchestration) | `TaskRun`/`TaskPlan`/`PlanTask`/`TodoItem` projections + `TaskCapabilityCatalog`/ownership wave (app-core tasks/) | `EkoRevisionedTaskStore` (load/CAS only), `EkoTaskToolPolicy`, `EkoRuntimeDagController` — thin, lossless (V04) | Legacy `TaskManager`/`TaskExecutor` (F-TSK-01-P3-01); `execution_mode: sequential` inert (F-TSK-02-P2-02); legacy ready loop (F-TSK-03); per-task cancel dead (A-TSK-03-P2-02); unguarded `set_task_status` (A-TSK-04-P2-01) | **Correct**; boundary defects filed, deletion targets listed |
| Subagent | `SubagentRegistry`/`SubagentExecutor`/`SubagentDefinition` (echo-agent) | loader policy (`subagent_loader.rs`), readonly/writer split, EKO prompt sections | `EkoSubagentPromptCompiler` (2 methods, V04) | 7 dead `SubagentDefinition` fields vs EKO loader (F-SUB-01-P2-01); dead `ContextBuilder`/`OutputSchema`/`MemoryScope` (F-SUB-01-P2-03); dead `TeamCoordinator`/`TeamRunner`/mailbox (F-SUB-02-P2-03); `HandoffManager` second registry + `handoff` second tool (F-MAG-01-P2-01) | **Correct**; deletion targets listed |
| Approval/HITL | `PermissionService` pipeline (echo-core) | `HitlDispatcher` + leaf providers (app-core hitl/) | `DynProviderHandler`/`DefaultPermissionRequestHandler` (boundary mapping defect F-HITL-01-P1-03) | dead `run/approval.rs` parallel authority (F-HITL-01-P2-03, F-RCT-02-P3-02); `TauriHumanLoopHandler` parallel GUI transport (A-HITL-01-P2-01); dead `IpcAuth` (A-HITL-01-P2-04, A-TOOL-01-P3-01, A-SRF-02-P3-01) | **Correct**; boundary defects + deletion targets |
| Memory | `Store`/`ConversationStore` 4+2 impls (echo-state), `MemoryLayerManager` (echo-agent) | `UnifiedMemory` wrap, instruction protocol, `RulePromoter` | thin construction (V04) | `safe_segment` verbatim copy (X-BND-01-P3-01); three project-root resolvers (X-BND-01-P2-01); MEMORY.md two parsers (A-MEM-01 layering); `SessionSearchEngine` dead (A-STATE-01-P3-01) | **Correct**; deletion targets |
| Skill | `SkillRegistry`/`SkillLoader` (echo-execution) | `skills_hub` marketplace, `SkillLoadPolicy` impl | thin trait impls | 5 frontmatter parsers (F-SKL-01-P3-01); 2 binary probes (F-SKL-01-P3-02); dual `SkillRegistry` instances (F-SKL-01-P1-02) | **Correct**; deletion targets |
| Plugin | `PluginRegistry`/`PluginIntegrator`/`PluginLifecycle` (echo-core) | `PluginRuntimeService` (transactional reload) | `PluginLifecycle` impl | 2 plugin data-dir path computations (F-PLG-01-P3-03) | **Correct** |
| MCP | `McpManager` (echo-integration) | consumer only | none | none | **Correct** |
| Context/compression | `ContextManager` + 6 compressors (echo-core/echo-state) | strategy choice + `compact_context.rs` markers; framework `AppConfig::apply_compressor` called from EKO (V04) | thin dispatch (V04) | `ContextAssembler`/`ContextSelector` dead parallel (F-CTX-01-P2-03); `ProviderAdapter`/`AdapterClient` dead (F-LLM-01-P2-03); window inference split (F-CTX-01-P1-01, A-CFG-01-P2-05) | **Correct**; deletion targets |
| Workflow | `echo-orchestration/src/workflow/` engine | `panels.rs` consumer + `StoredWorkflow` | `dsl.rs`/`loader.rs` lowering (thin) | EKO `WorkflowDef`/`WorkflowStep` dead (F-WFL-01-P3-08) | **Correct**; delete dead model |
| Intent | `src/intent/` router family (echo-agent) | `runtime.rs` wiring | thin conversion | dual threshold authority (F-INTENT-01-P2-03) | **Correct** |
| Notebook | `src/notebook/` (echo-agent) | none | none | notebook dead API (F-NBK-01-P2-01) | **Correct**; deletion candidate |
| Diff | `DiffTool` (echo-tools) — natural single engine | `diff.rs` (REPL) + `diff_file` inline (GUI) | none (GUI re-implements hunking inline) | three diff engines + duplicate `DiffHunk`/`DiffLine` + dead `DiffViewer` twin (A-PROJ-01-P2-03) | **Application-side duplicate**; GUI/REPL should delegate to one engine |
| Retry | `echo_core::retry` (`RetryPolicy`, doc: "unified use by all external calls") | none | none | `retry_llm_call` live second authority (F-REL-01-P2-01) + dormant orchestration backoff | **Framework-internal duplicate**, both live on one stack |
| URL fetch | — (none canonical) | none | none | 4 parallel download tools + `parse_page_range` dup (F-EXT-03-P2-01, P3-06) | **Framework-internal duplicate** |
| Tool risk classification | `WRITE_TOOLS`/`risk_level` trait | none | none | `ToolRiskClassifier` third authority, dead (F-EXT-01-P3-02) | **Framework-internal dead** |
| Worktree | `WorktreeFactory` trait (framework) | `RunWorktree` (app-core) | `EkoWorktreeFactory` (thin) | `panels.rs` legacy GUI helpers duplicated + diverged (A-TSK-05-P2-04) | **Correct**; delete panels.rs helpers |
| Trace/export | `trace` module (framework) | `save_trace` (app-core) / `export/` | — | `save_trace` second trace ledger (A-OBS-01-P2-01); second unused `export_conversation` (A-OUT-01-P2-01) | **Application duplicate** |

Cross-repository gate answers (REPORTING.md):

- **Generic mechanism**: task model/validator/executor, subagent dispatch,
  permission pipeline, store trait + impls, compressors, skill/plugin runtime,
  MCP client, workflow engine, intent router, diff tool, retry policy — all
  independently reusable; no EKO policy leaked into the framework (V01).
- **EKO product policy**: task projections + capability catalog + ownership
  wave, loader policy, provider arbitration + leaf providers, instruction
  protocol, skills hub marketplace, plugin reload transaction, worktree
  merge/branch policy — all app-owned (V01).
- **Adapter boundary**: `EkoRevisionedTaskStore`, `EkoRuntimeDagController`,
  `EkoTaskToolPolicy`, `EkoSubagentPromptCompiler`, `apply_compressor` — thin,
  lossless, no scheduling/validation/state authority (V04). Boundary defects
  are behavioral (A-TSK-03-P2-02, A-TSK-04-P2-01, F-HITL-01-P1-03), not
  second authorities.
- **Duplicate search**: terms per concept listed in V01-01; one definition per
  concept; EKO hits are projections/policy; zero `worker`; zero forbidden CRUD.
- **Migration deletion**: full deletion-target matrix below.

### Deletion-target matrix (authority → delete → impact)

| # | What to delete | Canonical finding | Who is authority | Deletion impact (callers/tests/docs) |
|---|---|---|---|---|
| D1 | Legacy `TaskManager`/`TaskExecutor`/`TaskHooks`/`VerifierFactory` (echo-orchestration) | F-TSK-01-P3-01, F-TSK-03 | `RuntimeDagExecutor` + `TaskRevisionService` | Zero production callers; test-only callers (executor.rs:1888-2133); doc convergence Phase 3; delete only via roadmap gate (pub API) |
| D2 | `execution_mode: "sequential"` acceptance/storage | F-TSK-02-P2-02 | single wave frontier (`ready_task_ids` + controller) | Remove field or enforce; schema test + EKO `ExecutionMode` test fixtures executor.rs:4736/5213/5812 |
| D3 | `refresh_in_flight`/`DagRefresh` | F-TSK-02-P3-01 | safe-point reload | Zero production callers; remove fn + tests |
| D4 | 7 dead `SubagentDefinition` fields + builder setters | F-SUB-01-P2-01 | EKO `subagent_loader.rs` | Delete fields + setters (builder.rs:61-155) or wire; `SubagentDefinition::new` callers unaffected |
| D5 | `ContextBuilder`/`SubagentOutput`/`OutputSchema`/`MemoryScope`/`isolated.rs` | F-SUB-01-P2-03 | `SubagentContext` + registry | Zero production callers; delete with tests |
| D6 | `TeamCoordinator`/`TeamRunner`/`mailbox.rs`/`message.rs` | F-SUB-02-P2-03 | `ManagerSubagentOrchestrator` + dispatch loop | Zero production callers; `Team.coordinator` field; docs MASTER-PLAN:152 + README:642-643 |
| D7 | `src/handoff/` (module + feature + re-exports) or re-implement over dispatch | F-MAG-01-P2-01/P2-02 | `SubagentRegistry` + `agent_tool` | Feature-gated, never registered; demos demo21/demo47; lib.rs:77-79,300-303 |
| D8 | `run/approval.rs` + `process_steps` + `execute_tool_feedback*` | F-HITL-01-P2-03, F-RCT-02-P3-01/P3-02 | `PermissionService` + `ToolExecutionPipeline` | Zero production callers (sole caller of `process_steps` is dead); move surviving ask/modified-args semantics into live path first |
| D9 | `TauriHumanLoopHandler` parallel transport + global pending map | A-HITL-01-P2-01 | app-core `HitlDispatcher` | GUI approvals route through dispatcher; remove handler + static map |
| D10 | `IpcAuth`/`IpcPermission` gates (3 modules) | A-HITL-01-P2-04, A-TOOL-01-P3-01, A-SRF-02-P3-01 | framework `PermissionService` | Zero callers; stale "Phase 6.2" doc headers |
| D11 | `ContextAssembler`/`ContextSelector` | F-CTX-01-P2-03 | `ContextManager`/`TokenBudget` | Zero production callers; off-main-path; delete with tests |
| D12 | `ProviderAdapter`/`AdapterClient` | F-LLM-01-P2-03 | live provider adapter (thinking protocol) | Dead second provider contract; move thinking-protocol authority then delete |
| D13 | `ToolRiskClassifier`/`ToolRiskCategory` | F-EXT-01-P3-02 | trait `risk_level` + `WRITE_TOOLS` | Zero live callers (risk.rs); delete or wire |
| D14 | `ToolManager::result_cache` | F-EXT-01-P3-01 | none | Never written/invalidated; remove field + doc claims |
| D15 | 3 of the 4-5 URL-download tools (merge caps into one family) | F-EXT-03-P2-01 | unified fetch tool | Tool-name surfaces (`web_fetch_enhanced`/`image_fetch`/`pdf_fetch`); keep `web_fetch`+`pdf_extract` or explicit menu; size-cap divergence tests |
| D16 | duplicate `parse_page_range` | F-EXT-03-P3-06 | single parser | Merge parsers; limit-divergence tests |
| D17 | 3 of the 5 frontmatter parsers | F-SKL-01-P3-01 | `parse_skill_md` (echo_execution) | loader.rs:407-483 / registry.rs:657-709 / hub naive parser; keep tests |
| D18 | hub inline binary probe | F-SKL-01-P3-02 | framework `dependency_probe` | `skills_hub/registry.rs:314-325`; delete `binary_available` copy |
| D19 | second plugin data-dir computation | F-PLG-01-P3-03 | `plugin_data_base_dir` config | Merge to one path fn |
| D20 | EKO `WorkflowDef`/`WorkflowStep` | F-WFL-01-P3-08 | framework workflow engine | app-core/src/state.rs:220-237; zero producers; delete |
| D21 | GUI inline diff engine + duplicate `DiffHunk`/`DiffLine` + dead `DiffViewer.tsx` | A-PROJ-01-P2-03 | one shared diff engine (framework `DiffTool` or app-core `diff.rs`) | tauri files.rs:310-372; files.rs:40-63; `components/coding/DiffViewer.tsx`; re-point GUI endpoints |
| D22 | `retry_llm_call` backoff math (keep `compute_concurrent_tool_batch_timeout`) | F-REL-01-P2-01 | `echo_core::retry` | react/run/retry.rs:28-68; engine call sites react_loop.rs:81,124; keep breaker hook |
| D23 | EKO `WorkflowDef`…(see D20); `save_trace` second ledger | A-OBS-01-P2-01 | framework `trace`/event ledger | executor.rs:3504 + callers :449/:491/:509/:562; either merge into event ledger or drop diagnostics panel |
| D24 | second `export_conversation` | A-OUT-01-P2-01 | `/export` live path | persistence.rs:286; delete or align source |
| D25 | `panels.rs:1828-1909` worktree helpers | A-TSK-05-P2-04 | app-core `task_runtime/worktree.rs` | `list_worktrees`/`create_worktree`/`remove_worktree` route through shared module |
| D26 | two EKO project-root resolvers (keep one) | X-BND-01-P2-01 (new) | single EKO resolver + framework `InstructionResolver` | see finding |
| D27 | `safe_segment` copy | X-BND-01-P3-01 (new) | one implementation | see finding |
| D28 | dead `reflect_on_session` duplicate | A-EVO-01-P3-02 | live REPL reflection writer | delete dead duplicate |
| D29 | `NotebookTracker` module | F-NBK-01-P2-01 | trace + `data_pipeline` | public module + flag; delete or wire |
| D30 | `web_config` dead config authority + dual `DEFAULT_CONTEXT_WINDOW` | A-CFG-01-P2-03/P2-05 | live config + single constant | model_config.rs:6 vs infra.rs:23; `update_config`; delete `web_config` store |
| D31 | duplicate tool-event projection producer | A-SRF-02-P2-01 / A-FE-02-P2-01 | one `ToolExecutionRepository` writer | mod.rs bridge or chat.rs `TauriExecutionProjector`; frontend dedupe tests |
| D32 | second auth-check authority (frontend) | A-FE-03-P3-05 | `RequireAuth` | module-level interval/focus listener; delete |

## Current Path

Verified at the reviewed commits (details in V02-01/V03-01/V04-01):

1. Task graph: one live authority chain —
   `task_create/task_update/task_list` (echo-orchestration) → `TaskRevisionService`
   → `PlanValidator` → store CAS; `task_execute` (EKO) → `RuntimeDagExecutor`
   → `EkoRuntimeDagController` → EKO store CAS. Legacy `TaskExecutor`
   `execute_ready_tasks` reachable only from its own `#[cfg(test)]` modules.
2. Subagent: `SubagentDefinition` → `SubagentRegistry` → `SubagentExecutor`
   dispatch; `tool_filter`/`model`/`timeout` definition fields have zero
   production readers; EKO `subagent_loader.rs` actually drives construction.
3. Approval: 15-stage `ToolExecutionPipeline` → `PermissionService`
   (echo-core); human asks only via handler path; EKO `HitlDispatcher` +
   leaf providers; dead `run/approval.rs` is the only ask-capable code and is
   unreachable.
4. Memory/context: framework stores (4+2 impls) + `MemoryLayerManager` +
   `ContextManager`/compressors; EKO `UnifiedMemory`/`apply_compressor` are
   thin; `DEFAULT_CONTEXT_WINDOW` duplicated in two app-core files.
5. Diff: `DiffTool` (framework) vs `diff.rs` (REPL) vs `diff_file` inline
   (GUI) — three live engines with duplicated type families.
6. Retry: `RetryPolicy` (echo-core) live in provider clients AND
   `retry_llm_call` live in the engine LLM path — both on the same stack.
7. Resolvers: `find_project_root` (VCS-first), `discover_project_root`
   (wrapper), `discover_echo_agent_dir` (`.eko`/`.git` only, returns `.eko`),
   framework `InstructionResolver` (git root + AGENTS chain) — four root
   computations with different marker sets.

## Findings

### X-BND-01-P2-01: Three parallel EKO project-root resolvers with divergent marker sets and return semantics — the same session can bind memory instructions, project context, and rule promotion to different roots

- Priority: P2
- Confidence: high
- Layer: application (framework `InstructionResolver` is correctly placed and
  distinct-purpose; the EKO duplication is the defect)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/utils.rs:12-40`
    (`find_project_root`: VCS markers `.git/.hg/.svn` first, then fallback
    manifests `.eko`/`Cargo.toml`/`package.json`/`go.mod`/`pyproject.toml`/
    `pom.xml`/`Makefile`; returns repo root);
  - `echo-agent-cli/echo-agent-app-core/src/project/context.rs:22`
    (`discover_project_root`: thin wrapper over `find_project_root`);
  - `echo-agent-cli/echo-agent-app-core/src/evolution/review_integration.rs:378-395`
    (`discover_echo_agent_dir`: walks up from CWD looking only for `.eko` or
    `.git` dirs and returns the `.eko` subdirectory; no manifest fallback, no
    VCS-first rule);
  - `echo-agent/echo-core/src/project_rules.rs:57-63` (framework
    `InstructionResolver`: git-root + AGENTS.md chain, `agents_files_only`
    variant; used by `echo-agent-cli/echo-agent-app-core/src/instruction_provider.rs:201`).
- Reachability: `find_project_root` ← instruction file discovery
  (`instruction_provider.rs:291-297`), project context
  (`project/context.rs`); `discover_echo_agent_dir` ←
  `evolution/review_integration.rs` (RulePromoter/memory-curator targets,
  `unified_memory` store resolution per A-MEM-01). All live in the same
  session.
- Expected invariant: one project-root authority with one documented marker
  contract, so memory instructions, project context, and promoted rules agree
  on where "the project" is (AGENTS.md: 严禁平行实现同一语义).
- Observed behavior: marker sets differ (`.git`+manifests vs `.eko`/`.git`
  only) and return semantics differ (repo root vs `.eko` subdir); in a nested
  crate directory without a repo boundary, `find_project_root` stops at the
  nearest package manifest while `discover_echo_agent_dir` keeps walking to
  `.eko`/`.git`; after `exit_workspace` (A-CFG-01-P1-02, CWD never restored),
  `discover_echo_agent_dir` resolves against the exited workspace while
  project context is explicitly cleared (state.rs:1072).
- Impact: a promoted rule can be written to a different `.eko` than the one
  the agent reads (A-MEM-01-P3-03 already demonstrates one concrete instance);
  the memory store and the instruction files can bind to different roots in
  one session; future root-policy fixes must be made in three places.
- Root cause: each subsystem introduced its own walk when the shared
  `find_project_root` predated it; the divergent semantics were never
  reconciled (A-MEM-01 recorded the divergence without a finding ID).
- Direction: make `discover_echo_agent_dir` delegate to a single EKO resolver
  (`find_project_root` + `.eko` join, or an explicit root parameter threaded
  from the workspace registry), delete the inline walk
  (review_integration.rs:378-395), and document the framework
  `InstructionResolver` as the instruction-file authority (distinct purpose:
  AGENTS.md chain, not memory scope).
- Regression validation: fixture with nested crate dir + `.eko` at repo root
  and a `.git`-only dir, asserting all four consumers resolve the same root;
  re-run after `exit_workspace` asserting rule promotion lands inside the new
  active workspace.
- Validation reports: [V01](../validations/X-BND-01/V01-01.md), [V02](../validations/X-BND-01/V02-01.md)

### X-BND-01-P2-02: The framework's dead parallel authorities have no tracked deletion backlog — the map produced here is the only record, and live duplicates keep diverging

- Priority: P2
- Confidence: high (fact) / medium (impact framing)
- Layer: framework + application (process)
- Evidence: 20+ dead/parallel surfaces enumerated in the deletion-target
  matrix (D1-D32), of which at least three are LIVE duplicates with divergent
  behavior: retry (`echo-core/src/retry.rs:35` vs
  `src/agent/react/run/retry.rs:13`, both on one stack, F-REL-01-P2-01), diff
  (three engines, A-PROJ-01-P2-03), URL download (four tools with divergent
  size caps, F-EXT-03-P2-01). B-ARCH-01-P2-01 explicitly delegates "produce
  the authority map" to X-BND-01; no MASTER-PLAN convergence section
  enumerates these surfaces (F-TSK-03's "convergence doc Phase 3" covers only
  the legacy task surface).
- Reachability: all three live duplicates execute on normal user paths
  (LLM call retry, GUI file diff, model-facing download tools).
- Expected invariant: AGENTS.md "严禁平行实现同一语义" — one authority per
  semantic with a written convergence plan when a staged migration is needed
  ("未完全收敛必须显式归档").
- Observed behavior: only the review reports document the duplicates; no
  tracked schedule exists for the framework-side deletions, so fixes do not
  propagate across implementations (e.g., the overflow-safe `RetryPolicy`
  coexists with the non-safe `retry_llm_call` — F-REL-01-P3-01), and new code
  keeps finding two candidate homes (B-ARCH-01-P2-01).
- Impact: material maintainability defect — divergent safety behavior on live
  paths (download size caps, GUI diff hunk headers, retry duration bounds) and
  a permanent standing violation of the one-authority gate.
- Root cause: the review track files findings but the implementation track
  (iteration roadmap, S-RDM-01) is the first consumer; no intermediate artifact
  owns the map before synthesis.
- Direction: S-RDM-01 must consume this report's deletion-target matrix as its
  ordering input; for each framework-side deletion, satisfy the AGENTS.md
  framework gate (framework-wide grep + "capability menu" judgment, V03);
  EKO-side deletions (D20-D32) are unconditional.
- Regression validation: after each deletion, the full framework/CLI gates
  plus the specific fixtures named in D1-D32.
- Validation reports: [V02](../validations/X-BND-01/V02-01.md), [V03](../validations/X-BND-01/V03-01.md), [V05](../validations/X-BND-01/V05-01.md)

### X-BND-01-P3-01: `safe_segment` is a verbatim copy in two framework files with different error types — duplication, not a second authority, but a drift risk

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/file_conversation.rs:465`
  (`fn safe_segment(id: &str) -> Result<String>` for `FileConversationStore`)
  and `echo-agent/src/state/file.rs:253`
  (`fn safe_segment(id: &str) -> Result<String, ReactError>` for the root
  `FileStore`-adjacent file store).
- Reachability: both are live in their own store implementations (path
  sanitization of store ids); F-MEM-01's V01 confirmed single authoritative
  definitions for the store semantics with `safe_segment` as the only copy.
- Expected invariant: one id-sanitization implementation shared by both file
  backends (AGENTS.md duplication rule).
- Observed behavior: identical logic duplicated; the two copies have already
  drifted in error type and will drift in edge handling (e.g., traversal
  rejection) independently.
- Impact: localized — a sanitization fix must be applied twice; divergence
  risk for store id handling.
- Root cause: the root `src/state/file.rs` backend and the
  `echo-state` `FileConversationStore` were written/migrated at different
  times and the helper was copied instead of shared.
- Direction: hoist one `safe_segment` into `echo-state/src/memory/mod.rs`
  (or `echo-core`) and have `src/state/file.rs` call it (or, per the
  F-MEM-01/F-MEM-02 file-backend migration direction, delete the root copy
  when the root file store converges on the echo-state backend).
- Regression validation: existing traversal-rejection tests in both files
  (file_conversation.rs:772, state/file.rs counterpart) green after
  unification.
- Validation reports: [V02](../validations/X-BND-01/V02-01.md), [V05](../validations/X-BND-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search across both repositories by concept list | yes | passed | [V01-01](../validations/X-BND-01/V01-01.md) |
| V02 | Behavior/call-path duplicate search (reachability of diff/retry/download/handoff/save_trace/execution_mode/resolvers/constants) | yes | passed | [V02-01](../validations/X-BND-01/V02-01.md) |
| V03 | Public framework option check per suspected duplicate (capability menu vs deletion candidate) | yes | passed | [V03-01](../validations/X-BND-01/V03-01.md) |
| V04 | Adapter-logic/deletion-target matrix (thin-adapter verification) | yes | passed | [V04-01](../validations/X-BND-01/V04-01.md) |
| V05 | Merge/cross-check with existing findings (canonical IDs, no re-filing) | yes | passed | [V05-01](../validations/X-BND-01/V05-01.md) |

No executable validations were required: this task is a static map over
already-executed F-*/A-* validations; V04's target (adapter thinness) is
inspectable statically (REPORTING.md allows read-only validations).

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md "严禁平行实现同一语义 / 一个权威实现" | regressed | Three live duplicates: retry (F-REL-01-P2-01, verified react_loop.rs:81,124 vs echo-core retry.rs:35), diff (A-PROJ-01-P2-03, verified), URL download (F-EXT-03-P2-01, verified) — all un-migrated at 9b0e0fa |
| AGENTS.md "framework public API retained unless framework-wide evidence shows obsolete/fully replaced" | current | Stores/compressors/legacy TaskExecutor retained as options (V03); dead parallel APIs are deletion candidates, not options |
| AGENTS.md "拿不准时的默认:应用层 / 适配器必须薄且转换无损" | current | All five principal adapters verified thin and lossless (V04) |
| AGENTS.md "echo-agent-cli 不需要 SQLite" | current | Zero sqlite usage in EKO (F-MEM-02 V01; A-TSK-01 V01) |
| echo-agent-cli MASTER-PLAN convergence "Phase 3" legacy task surface | current | Legacy `TaskExecutor`/`hooks`/`verifier` documented as Phase-3 deletion but still pub with test-only callers (V03) |
| README "8 crates, 1 import" facade claim | current with caveat | Root facade still owns the largest engine share (B-ARCH-01-P2-01); workspace alias exposes all sub-crates |
| A-MEM-01 layering: "three project-root resolvers with different marker sets" | current (unfiled) | Verified; filed here as X-BND-01-P2-01 |
| F-MEM-01 layering: "safe_segment duplicated with identical logic" | current (unfiled) | Verified; filed here as X-BND-01-P3-01 |

## Coverage And Uncertainty

- The map covers the concept list in the task card plus diff/retry/URL-fetch/
  risk-classification found via V01's name searches; concepts outside that
  list (e.g., a2a, lsp, channels, sandbox, eval) were reviewed by their own
  F-*/A-* tasks and are referenced only via their reports.
- Reachability claims rest on static call-path searches (V02) and the
  dependency reports' own executable validations; no new compilation was
  performed (read-only map task).
- Confidence on "no cross-repository live second engine" is bounded by the
  name/behavior search space of V01/V02; a behavioral duplicate under a
  different name than any searched concept cannot be excluded (each F-*/A-*
  task ran its own duplicate searches, which this map aggregates).
- The deletion-target matrix is a map, not a schedule; execution order,
  per-repo ownership, and acceptance criteria belong to S-RDM-01
  (X-BND-01-P2-02).

## Handoff

- **Conclusions downstream tasks may rely on**: the framework/app/adapter
  split is correct for all core concepts; the remaining duplicates are
  framework-internal dead/parallel APIs (mostly), inert schema fields, and a
  handful of application copies; no EKO-side second engine exists; all five
  principal adapters are thin and lossless (with the filed boundary defects).
- **Reports to read**: this report + its 5 validations; for each canonical ID
  in the matrix, the corresponding F-*/A-* task report.
- **Conditions that make this report stale**: any commit that wires one of the
  dead surfaces (e.g., `tool_filter` readers, handoff registration,
  `execution_mode` enforcement) or deletes one of the D1-D32 targets; the
  phase synthesizer should re-run V02 on the touched concepts.
- **Follow-up task IDs**: X-TSK-01 (task graph conformance, D1-D3), X-PLG-01
  (skill/plugin lifecycle, D17-D19), X-AUT-01 (permission boundary, D8-D10),
  X-SRF-01 (surface parity, D9/D31), X-MEM-01 (memory/compression, D26/D27),
  X-TOL-01 (tool schema, D15/D16), S-RDM-01 (iteration roadmap consuming the
  deletion-target matrix).
