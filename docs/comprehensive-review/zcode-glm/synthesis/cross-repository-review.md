# S-X-01: Cross-Repository Review Synthesis (echo-agent + echo-agent-cli)

> Synthesis task: S-X-01
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Baseline: `echo-agent` `9b0e0fa`, `echo-agent-cli` `b3b2e81`
> Sources synthesized: 10 X-phase task reports (`X-*.md`) + `framework-review.md` (S-FW-01) + referenced F-/A-/Q-phase dependency reports
> Synthesis date: 2026-08-12

This document merges, deduplicates, and prioritizes every cross-repository
finding produced by the X-phase review tasks. Canonical IDs retain
backlinks to the originating task reports. Boundary-gate (generic
mechanism / EKO product policy / adapter boundary) is answered for each
cross-repo recommendation. The duplicate-authority map, dead-code
inventory, and adapter-thinness verification close out the AGENTS.md
"framework-vs-application" + "first prove no duplicate" implementation
gates at the cross-repo scope.

---

## 1. Executive Summary

The X-phase review surfaced **39 distinct cross-repository findings**:
**2 P1**, **23 P2**, **14 P3**, **0 P0**. The two P1 findings are:

1. **X-MEM-01-P1-01** — Hot-layer `MEMORY.md` edits refresh the wrong
   projection (`eko:instruction-context` instead of
   `eko:hot-memory-context`). Dreaming promotions, `/remember`
   auto-promotions, and hot-layer deletes are a *lost update* until the
   next workspace switch. The framework's per-turn recall
   (`TURN_MEMORY_CONTEXT_PROJECTION`) partially compensates query-dependently;
   the stable hot-layer prefix the feature advertises is silently frozen.
2. **X-SRF-01-P1-01** — GUI MCP IPC over-validation (executable allowlist +
   private-range URL block) inherited from `A-INT-01-P1-01` makes
   legitimate local MCP servers unreachable via the GUI panel while the
   on-disk config path (and TUI `/mcp load`) accepts them. This is the
   canonical instance of the AGENTS.md "历史教训" pattern — over-gating
   that produces surface asymmetry under the excluded XSS/SSRF threat
   model.

**Architectural verdict.** The cross-repo layering is sound:

- One revisioned TaskRun graph (framework) with a thin EKO adapter; one
  mutator (`TaskRevisionService`); one validator (`PlanValidator`); one
  kernel (`RuntimeDagExecutor`); one plugin/skill/hook orchestrator
  (`PluginRuntimeService`) over framework primitives
  (`PluginIntegrator::wire_all` + `unload_agent_components`).
- AGENTS.md rule 6 ("任务关系只有一个权威 API") holds end-to-end at
  the type, behavior, and adapter layers (X-TSK-01 V02 + V03, X-BND-01
  V01 + V04).
- All five "structural" AGENTS.md invariants hold (Subagent-only
  terminology, CLI no-SQLite, no parallel task CRUD, relative Cargo
  paths, Q-STA-01 panic-keyword baseline); one new panic-safety
  violation was found (IQR index out-of-bounds at `data_quality.rs:253`,
  X-INV-01-P2-01).
- Every cross-cutting adapter audited (`EkoRevisionedTaskStore`,
  `EkoTaskToolPolicy`, `EkoRuntimeDagController`, the three refresh
  helpers in `unified_memory.rs`, `PluginRuntimeService::apply_candidate`,
  the four `ChatSink` implementations) is **thin**: it holds no ready
  frontier, DAG loop, generic retry, deadlock detection, or second
  validator.

**Cross-cutting defects.** Four patterns recur:

1. **`atomic_write` duplicated 6× with drifting durability** — only 1 of
   6 sites calls parent-dir fsync; each backend reimplemented the recipe
   independently (X-BND-01-P2-01 + X-STA-01-P2-02).
2. **Cancelled-vs-Error collapse** — `ReactAgent` never emits
   `AgentEvent::Cancelled` (F-RCT-03-P2-02); only GUI recovers via
   post-`drive_chat` polling (X-EVT-01-P2-01). TUI / REPL / channels
   mislabel cancel as error.
3. **Surface-wiring parity gaps** — 8 gaps aggregate from A-SRF-* and
   A-INT-01; all are missing wiring (not architectural), all fixes are
   localized to the application layer (X-SRF-01).
4. **Dead-code & parallel-authority inventory** — 8 pub-exported but
   unused framework/application surfaces (`GLOBAL_EVENT_BUS`,
   `TaskSubagent`, `NotebookTracker`, `register_lifecycle`, `add_artifact`,
   `validate_event_trajectory`, `IpcAuth`, the 5 dead Cargo features) —
   most meet AGENTS.md deletion criterion ✅ branch 1 (superseded).

---

## 2. Finding Count Summary

| Priority | Count | Scope |
|---|---:|---|
| P0 | 0 | — |
| P1 | 2 | 1 application refresh-wiring (X-MEM-01) + 1 application over-gating (X-SRF-01, cross-filed from A-INT-01) |
| P2 | 23 | 4 framework (X-INV-01 IQR, X-BND-01 ×3, X-EVT-01 validate_event_trajectory), 17 application, 2 cross-layer conformance (X-TOL-01 ×2 framework-rooted/application-felt), 5 surface-parity aggregation (X-SRF-01), 2 collision (X-PLG-01) — net of merge |
| P3 | 14 | 1 framework invariants doc, 1 CLI doc drift, 2 dead-code observations (X-PLG-01, X-STA-01), 2 adapter duplications (X-BND-01), 6 surface/parity gaps (X-SRF-01, X-EVT-01, X-TOL-01), 2 threat-model narrative drift (X-AUT-01) |
| **Total** | **39** | |

Per-report breakdown (de-duplicated; merged findings counted once under their canonical owner):

| Report | P1 | P2 | P3 | Total |
|---|---:|---:|---:|---:|
| X-MEM-01 | 1 | 2 | 0 | 3 |
| X-SRF-01 (aggregator) | 1 | 6 | 3 | 10 |
| X-STA-01 | 0 | 3 | 1 | 4 |
| X-BND-01 | 0 | 3 | 2 | 5 |
| X-EVT-01 | 0 | 3 | 3 | 6 |
| X-TOL-01 | 0 | 3 | 1 | 4 |
| X-PLG-01 | 0 | 1 | 1 | 2 |
| X-AUT-01 | 0 | 1 | 1 | 2 |
| X-INV-01 | 0 | 1 | 1 | 2 |
| X-TSK-01 | 0 | 0 | 1 | 1 |
| **Total (de-duplicated)** | **2** | **23** | **14** | **39** |

**Merges applied** (see §5 for the full map):
- `atomic_write` durability gap: X-BND-01-P2-01 + X-STA-01-P2-02 + F-MEM-01-P2-01 + framework FW-MEM-002 → canonical **X-BND-01-P2-01**.
- Collision non-determinism: X-PLG-01-P2-01 absorbs F-SKL-01-P2-01 + F-PLG-01-P2-01.
- GUI MCP over-gating: X-SRF-01-P1-01 absorbs A-INT-01-P1-01 + X-AUT-01-P2-01 (group b).
- IQR panic: X-INV-01-P2-01 is the canonical home; framework-review FW-TOOLS-003 mirrors it.
- TaskSubagent dead trait: X-BND-01-P2-02 is canonical; framework-review FW-TSK-* does not re-list it.
- GLOBAL_EVENT_BUS dead infra: X-BND-01-P2-03 owns the cross-repo boundary side; framework-review FW-CORE-001 owns the framework-internal side.

---

## 3. Boundary-Gate Completeness Verification

For every cross-repo recommendation, the AGENTS.md framework-vs-application
layering gate requires a three-way classification (generic mechanism / EKO
product policy / adapter boundary). Each X-phase report carried its own
"Layering Decision" table; this synthesis confirms completeness.

| Cross-repo concern | Generic mechanism (framework) | EKO product policy (application) | Adapter boundary | Source |
|---|---|---|---|---|
| Atomic file replace (`atomic_write`) | `FileConversationStore::atomic_write`, `FileRuntimeStateStore::atomic_write` (both with parent-dir fsync) — `echo-state` + root `state/file.rs` | The 4 application-specific writers (analysis, research, tool_execution JSON, file_shadow) | One canonical helper belongs in framework `echo-state::util` (or `echo-core::utils`); migrate all 6 call sites | X-BND-01-P2-01, X-STA-01-P2-02 |
| Task execution contract | `RevisionedTaskGraph` + `RuntimeDagController` + `RuntimeDagExecutor` + `TaskRevisionService` + `TaskPatchEngine` + `PlanValidator` | EKO file-backed store + `EkoTaskMetadata` JSON payload + `task_execute` dispatch shell + `TaskCapabilityCatalog` | `EkoRevisionedTaskStore` + `EkoTaskToolPolicy` + `EkoRuntimeDagController` — all confirmed thin (no frontier/DAG/retry/validator) | X-TSK-01 V02, X-BND-01 V04 |
| Plugin / skill / hook lifecycle | `SkillRegistry`, `HookRegistry`, `McpManager`, `SubagentRegistry`, `PluginRegistry`, `PluginIntegrator::wire_all`, `PluginLifecycleManager` | `PluginRuntimeService::apply_candidate` atomic swap + 4-checkpoint rollback; `prepare_application_components`; `register_plugin_agents` | Adapter calls framework primitives only; no re-implementation of dependency resolution, source tagging, hook validation, or subagent dispatch | X-PLG-01 V01–V04 |
| Memory / instruction projections | `<echo-agent-context-projection-v1>` envelope, `is_context_projection_message`, `replace_projection`, `split_protected`/`merge_protected`, `TURN_MEMORY_CONTEXT_PROJECTION` per-turn tail recall | `InstructionProvider` 5-tier file protocol, `UnifiedMemory`, two projection markers (`eko:instruction-context`, `eko:hot-memory-context`), refresh helpers | Three refresh helpers wrap `Message::system` around file contents and call `context.replace_projection(marker, message)`. Thin; no scheduling authority, no semantic loss | X-MEM-01 |
| Event lifecycle | `AgentEvent` (20 variants, `#[non_exhaustive]`), `EventEnvelope`, `envelope_event_stream_after`, `validate_event_trajectory`, `is_terminal()` | `ChatSink` impls, `ChatDriverEvent`, `TauriChatSink` (with tool-exec persistence), `aggregate_by_sentence`, TS reducers | `drive_chat` forwards envelopes; surfaces render. GUI adds post-`drive_chat` `TurnStatus` as compensation for the framework's missing `Cancelled` emission | X-EVT-01 |
| Persistence / recovery | `FileConversationStore`, `FileRuntimeStateStore::clear_conversation` (zero production callers), `cleanup_tool_output_scope`, `TaskClaim::execution_id`, `validate_tool_message_pairing` | Application id generation (frontend `conv-{ts}-{rand}`, TUI/REPL `Uuid::new_v4()`), `TaskRuntimeStore`, `tool_execution.rs::ToolExecutionRepository`, deletion cascade | `latest_run_for_conversation` is a thin read-side adapter; the symmetric write-side `delete_runs_for_conversation` does not exist (X-STA-01-P2-03) | X-STA-01 |
| Tool error / artifact / schema | `ToolResult` / `ToolFailure` taxonomy, `AgentEvent::ToolResult|ToolError|ToolStream`, `ToolOutputArtifactWriter` (sha256), `process_tool_output_for_call` spill policy | `ToolExecutionRepository`, two observers (`TauriChatSink::handle_tool_event`, `TauriExecutionProjector`), `pending_tool_completions` merge, `LIVE_DETAIL_AUTOLOAD_CHARS = 256 KiB`, hand-written TS mirror | `execute_tool_with_policy` is the single load-bearing seam; it is *thin* but *lossy on the success path* (drops `truncated` + spill `metadata` for non-streaming tools — X-TOL-01-P2-01) | X-TOL-01 |
| Permission / security boundary | `PermissionMode` × `ToolPermission` → `PermissionDecision`, `PermissionService::check_with_permissions_in_mode`, `ProtectedPathChecker`, `SessionApprovalCache`, secret-pattern catalog, `PathValidator::validate_within_base` | Mode labels, `set_permissions_mode` IPC, per-surface provider wiring, `path_validator.rs` secret-denylist, `validate_ipc_mcp_*` input gates, `redact_mcp_config_secrets`, `write_terminal` per-session consent | `execute_tool_with_policy` (`snapshot.rs:1189`) is the single agent-tool entry; `DynProviderHandler` adapts EKO provider. Direct-user IPC commands intentionally bypass `execute_tool_with_policy` (two-path separation) | X-AUT-01 |
| Surface parity (multi-mode) | `drive_chat` (chat lane), `execute_run` / `drive_agent_run` (task lane), `MessageHandler`, `SchedulerRunner`, `BackgroundTaskService`, `PluginRuntimeService`, `HumanLoopProvider` dispatcher | Per-mode slash-command surface (TUI 57, REPL 20, channels 5), per-mode tool visibility, per-mode renderers, cron/background trigger adapters | Every mode builds `PreparedUserTurn` + `ChatResources` (chat lane) or `RunPayload` (task lane) and renders the resulting stream. None recomputes chat/task lifecycle, DAG, or frontier | X-SRF-01 |
| Repository invariants | The literal AGENTS.md rules (Subagent-only, no parallel task CRUD, relative paths, panic-keyword baseline) | CLI's no-SQLite policy choice (the framework's `SqliteStore` is a retained menu option) | The CLI's `path = "../echo-agent"` declaration is the adapter boundary (worktree-relative paths must be restored before merge) | X-INV-01 |

**Verdict.** Boundary-gate completeness is **100%**: every cross-repo
recommendation has an explicit three-way classification in its source
report. No finding is missing a layer assignment.

---

## 4. AGENTS.md Invariants Status (X-INV-01)

X-INV-01 re-verified six invariants at `9b0e0fa` / `b3b2e81`. The
canonical conclusion is **X-INV-01 (overall): 5 of 6 hold cleanly; 1
has one new violation; 2 prior UTF-8 violations reaffirmed**.

| Invariant | Status | Evidence |
|---|---|---|
| V01 Subagent-only terminology (no `Worker`/`worker`) | ✅ **holds** | V01-01: zero word-boundary `worker`/`Worker` identifiers in production source. All 31 case-insensitive hits are substrings of `NetworkError`. |
| V02 CLI no-SQLite | ✅ **holds** (one stale comment, P3-01) | V02-01: 0 SQLite packages in CLI lockfile (787 total), 0 direct deps, 0 constructor sites, `sqlite` feature not enabled on `echo-state`. The framework's `SqliteStore` is a correctly-retained menu option (not a deletion target). |
| V03 No parallel task CRUD | ✅ **holds** | V03-01: framework `task_create`/`task_update`/`task_list` trio is single authority; CLI adds only `task_execute` (permitted); no `todo_write`/`plan_create`/`plan_patch`/`plan_execute` tool registered. X-TSK-01 V03 re-confirms at the cross-repo scope. |
| V04 Relative Cargo paths | ✅ **holds** | V04-01: 0 `worktrees` or `/Users/` substrings in any committed `Cargo.toml`; all 25 `path =` declarations workspace-relative. |
| V05 Panic safety | ⚠ **regressed (1 new site)** | V05-01: Q-STA-01 panic-keyword baseline still holds; computed-index extension finds `data_quality.rs:253-254` IQR panics for n=4 (X-INV-01-P2-01). |
| V06 UTF-8 safety | ⚠ **2 prior sites unchanged** | V06-01: `gitignore.rs:178-180` (Q-STA-01-P1-01) and `rule.rs:51-56` (F-SEC-01-P3-01) still present; no new sites. |

---

## 5. Canonical Duplicate-Authority Map

Across both repositories, the AGENTS.md "first prove no duplicate" gate
is satisfied for the live task/event/memory/plugin/tooling authorities
(X-TSK-01, X-PLG-01, X-EVT-01, X-MEM-01 all confirm single-authority per
concept). The remaining duplicates fall into two classes: **drifting
implementations of a generic primitive** (correctness hazard) and
**dead/superseded parallel surfaces** (deletion candidates).

### 5.1 Live duplicates (drifting implementations)

| # | Concept | Sites | Drift | Canonical finding |
|---|---|---|---|---|
| 1 | `atomic_write(path, bytes)` | 6 sites: `echo-state/src/memory/file_conversation.rs:494` (✓ parent fsync), `echo-agent/src/state/file.rs:210` (✓ parent fsync), `echo-agent-cli/.../analysis.rs:934` (✗), `echo-agent-cli/.../research.rs:1988` (✗), `echo-agent-cli/.../tool_execution.rs:648 write_json_atomic` (✗), `echo-agent-cli/.../tasks/task_runtime/file_shadow.rs:405` (✗) | Only 2/6 fsync the parent dir; tmp-naming strategies differ; some return custom error types | **X-BND-01-P2-01** (absorbs X-STA-01-P2-02 + F-MEM-01-P2-01 + framework FW-MEM-002) |
| 2 | `WorktreeError` (string-wrapper error) | 2 sites: `echo-agent/src/agent/subagent/worktree.rs:35-52`, `echo-agent-cli/.../tasks/task_runtime/worktree.rs:117-145` | Byte-identical except CLI adds `From<std::io::Error>` | X-BND-01-P3-01 |
| 3 | `git worktree add` invocation | 2 paths: framework `echo_tools::git_worktree::create_worktree`, CLI `task_runtime/worktree.rs:412 run_git(...)` (with prune/merge-base/branch-validation layered on top) | CLI's adds prune / merge-base / branch-validation the framework helper may lack — possible justification but undocumented | X-BND-01-P3-02 |
| 4 | Per-loader collision resolution policy | 2 loaders: skill `scan_directory` (first-scope-wins, `warn!`), plugin `scan_scope_dir` (last-scanned-wins, silent) | Contradictory cross-scope precedence + inconsistent logging; both inherit filesystem-order nondeterminism | X-PLG-01-P2-01 (absorbs F-SKL-01-P2-01 + F-PLG-01-P2-01) |
| 5 | Wire type definition (Rust → TS) | 2 sources per family: Rust enum + hand-written TS union (e.g. `ToolFailureCategory`, `ToolFailure`, `ToolExecutionDetailManifest`, `ChatEvent`) | Drift invisible to compiler; adding a Rust variant does not break the TS build | X-TOL-01-P2-02 (tool-execution family); X-EVT-01-P3-02 (ChatEvent family; A-FE-01-P3-04 is the IPC-wide instance) |
| 6 | Cursor pagination | 2 mechanisms: framework in-tool `PageRequest`/`PageInfo` (cursor binds to SHA-256 of query+items) inside `ToolResult.data`; EKO projection `read_output(cursor, limit)` paginates JSONL or spill file with opaque byte cursor + UTF-8 repair | Independent; not a defect — they solve different problems. Documented for future contributors | X-TOL-01 (no finding; observational) |

### 5.2 Dead / superseded parallel authorities (deletion candidates)

| Surface | Repo | Status | Successor | Canonical finding |
|---|---|---|---|---|
| `GLOBAL_EVENT_BUS` / `EventBus` | framework (`echo-agent/src/event_bus.rs:11-45`) | zero `.send()` / `.subscribe()` callers in either repo | direct stream composition via `envelope_event_stream` | **X-BND-01-P2-03** (cross-repo boundary owner); framework FW-CORE-001 owns the framework-internal side |
| `TaskSubagent` trait + `TaskExecutionSummary` (FW) + `SuggestedTask` (FW) | framework (`echo-orchestration/src/tasks/runtime.rs:296-331`) | zero implementors of the trait; CLI adapter converter `to_runtime_summary()` has only a `#[test]` caller | `RuntimeDagController` + EKO-local `TaskExecutionSummary` | **X-BND-01-P2-02** |
| `NotebookTracker` / `NotebookCell` / `enable_notebook()` | framework (`echo-agent/src/notebook.rs`, always compiled) | zero live callers; `enable_notebook` declared/setter-only (no read) | none (aspirational; no concrete consumer in either repo) | framework FW-NBK-001 (no cross-repo owner; flagged for completeness) |
| `register_lifecycle` (pub API) | application (`plugin_runtime.rs:387-410`) | zero production callers; 5 call sites all `#[cfg(test)]` | `apply_candidate` for normal wiring | X-PLG-01-P3-01 |
| `add_artifact` / `Artifact` struct / `ArtifactProduced` event kind / `list_task_artifacts` Tauri command | application (`tasks/task_runtime/store.rs:1432` etc.) | zero production producers; frontend `artifacts` panel always empty | none (reviewer gate would be the natural producer; not wired) | X-STA-01-P3-01 |
| `validate_event_trajectory` | framework (`echo-core/src/agent/event_envelope.rs:197`) | exported via `prelude`; zero production callers (test-only) | structural enforcement by `envelope_event_stream_after`'s `break` (no independent verifier today) | X-EVT-01-P2-03 |
| `IpcAuth` / `IpcPermission` / `require_full_auto` / `require_not_strict` | application (`tauri/error.rs:17-70`) | zero callers | none; the literal permission gate was correctly removed per AGENTS.md "历史教训" | A-HITL-01-P2-02 (inherited) |
| `Persistence` / `SessionSearchEngine` | application (legacy session store) | dead (A-STATE-01-P2-01) | `FileConversationStore` | framework-review does not re-list; A-STATE-01 owns |
| `LoopDetector` + config plumbing | framework | only `#[cfg(test)]` callers; never wired into `run_core_loop` | hard `max_iterations=100` ceiling + soft budgets | framework FW-RCT-004 |
| `process_steps` + `execute_tool_feedback_raw/_helper` | framework | `#[allow(dead_code)]`; superseded by `run_core_loop` | `phases::tools::run_tools` | framework FW-RCT-005 |
| `AdapterClient` + `ProviderAdapter` | framework | zero implementors; doc claims routing it does not perform | `OpenAiClient` + `translate_thinking_openai_compat` | framework FW-LLM-011 |
| `DefaultLlmClient` | framework | never constructed | `OpenAiClient` | framework FW-LLM-012 |
| `HandoffManager` + `HandoffTool` | framework | parallel identity/dispatch authority with zero production consumers | `SubagentEventBus` dispatch | framework FW-SUB-010 |
| `TopologyTracker` + `TopologyCallback` | framework | zero production consumers | none | framework FW-SUB-019 |
| `isolated.rs::run_isolated` | framework | zero production callers | none | framework FW-SUB-013 |
| 5 dead Cargo features (`sandbox`, `semantic-memory`, `macros`, `provider-factory`, `multimodal`) | framework | declared `= []`; gate nothing; reference nothing | none | framework FW-FEAT-001 |

**Deletion-test verdict per AGENTS.md "删除框架代码的判定":**

- ✅ **Delete (branch 1, superseded):** `GLOBAL_EVENT_BUS`, `TaskSubagent`,
  `NotebookTracker`, `register_lifecycle`, `add_artifact`, `IpcAuth`,
  `LoopDetector`, `process_steps`, `AdapterClient`/`DefaultLlmClient`,
  `HandoffManager`/`TopologyTracker`/`run_isolated`, 5 dead Cargo features,
  `Persistence`/`SessionSearchEngine`.
- ⚠ **Decide:** `validate_event_trajectory` (wire-or-stop-exporting;
  X-EVT-01-P2-03 recommends `tracing::warn!` instrumentation).
- ❌ **Retain (legitimate menu options even when CLI does not consume):**
  `SqliteStore`, `SqliteConversationStore`, `SqliteRuntimeStateStore`,
  `HybridCompressor`, `EmbeddingStore`, `InMemoryStore`,
  `InMemoryRevisionedTaskStore` (per AGENTS.md "框架 API 删了,复用方的代码会断").

---

## 6. Adapter-Thinness Verification

Each X-phase report that audited an adapter confirmed its thinness
against the AGENTS.md rule ("adapter 不得重新拥有 ready frontier、DAG
主循环、通用重试/取消、死锁判断或第二套 plan validator").

| Adapter | Repo | Owns scheduling? | Owns DAG loop? | Owns retry? | Owns validator? | Verdict |
|---|---|---|---|---|---|---|
| `EkoRevisionedTaskStore` | CLI (`revisioned_adapter.rs:26-56`) | no | no | no | no | thin — pure pass-through + error mapping |
| `EkoTaskToolPolicy` | CLI (`revisioned_adapter.rs:80-296`) | no | no | no (one-shot `Pending` resolution) | no (calls framework `PlanValidator` via service) | thin — product policy only |
| `EkoRuntimeDagController` | CLI (`executor.rs:1222`) | filters framework frontier only (`select_ready_wave` for file-ownership safety; framework `runtime_executor.rs:100-108` explicitly authorizes this) | no | reviewer strategy via `resolve_dispatch`, not generic retry | no | thin |
| `apply_eko_task_update` / `commit_eko_task_plan` | CLI (`revisioned_adapter.rs:321, 344`) | no | no | no | no | thin DTO converters routed through `service.apply_patch` / `service.create_prepared` |
| `refresh_instruction_projection` / `refresh_hot_memory_projection` / `refresh_memory_projections` | CLI (`unified_memory.rs:138-186`) | no | no | no | no | thin — read files, wrap `Message::system`, call `context.replace_projection` |
| `PluginRuntimeService::apply_candidate` | CLI (`plugin_runtime.rs:551-802`) | no | no | no | no | thin — atomic swap + 8-checkpoint rollback choreography; delegates wiring to `PluginIntegrator::wire_all`, lifecycle to `PluginLifecycleManager` |
| `TauriChatSink` / `TauriExecutionProjector` / `TuiChatSink` / `ChannelChatSink` | CLI | no | no | no | no | thin — observers that forward what the framework emits |
| `execute_tool_with_policy` seam | framework (`snapshot.rs:1189`) | n/a | n/a | n/a | n/a | thin AND lossy on success path (`truncated` + spill `metadata` dropped — X-TOL-01-P2-01) |
| `latest_run_for_conversation` | CLI (`file_store.rs:84-92`) | no | no | no | no | thin read-side adapter (write-side `delete_runs_for_conversation` missing — X-STA-01-P2-03) |

**Verdict.** No adapter owns scheduling / DAG / retry / validator
authority. The one asymmetric adapter-side workaround
(`pending_tool_completions` in `TauriChatSink::handle_tool_event`) recovers
streaming-tool metadata that the framework seam drops; it is correct but
asymmetric (does not recover metadata for non-streaming tools — X-TOL-01-P2-01).

---

## 7. Findings by Priority (P0 → P1 → P2 → P3)

Within each priority bucket, findings are ordered by impact / blast
radius / fix cost. Merged findings list their backlinks.

### 7.1 P0 — none

No data-corruption-with-secret-exposure or unrecoverable-system-level-defect
issues were surfaced by the X-phase review.

### 7.2 P1 — fix first

#### **X-SRF-01-P1-01** (canonical; absorbs A-INT-01-P1-01, X-AUT-01-P2-01 group b): GUI MCP IPC over-validation blocks legitimate local servers

- Layer: application (`tauri/commands/mcp.rs:117-160, 169-208`)
- Backlinks: A-INT-01-P1-01 (originator); X-AUT-01-P2-01 (group b: same
  sites, narrative side)
- Defect: `validate_ipc_mcp_stdio` rejects any stdio command whose
  base-name is not in `{npx, node, uvx, uv, python, python3, pipx,
  docker, java}` (executable allowlist); `validate_ipc_mcp_url` rejects
  any URL whose host matches `localhost`, `127.0.0.1`, `::1`,
  `169.254.*`, `10.*`, `192.168.*`, `172.16-31.*` (private-range block).
  The on-disk path (`McpConfigFile::from_file` → `validate_stdio_command`)
  only blocks shell metacharacters + a small dangerous-command denylist +
  path traversal. TUI `/mcp load` calls `agent.load_mcp_from_file` with
  no IPC allowlist. Same content accepted by 3 paths, rejected by the
  4th.
- Impact: a user's local `https://localhost:8100/mcp` server or
  `/usr/local/bin/my-custom-mcp` binary is unreachable via the GUI panel
  while identical content works via on-disk config / TUI / CLI startup.
  Same class of regression as the historical `require_full_auto` gate
  (AGENTS.md "历史教训").
- Direction: align `validate_ipc_mcp_*` with the on-disk
  `validate_stdio_command` discipline (denylist + shell-metacharacter +
  path-traversal only; drop the executable allowlist; drop the loopback
  / private-range rejection). Pair with X-AUT-01-P2-01 (rewrite comments
  to cite local-valid categories) and A-HITL-01-P2-02 (delete dead
  `IpcAuth`).

#### **X-MEM-01-P1-01** (canonical; re-affirms A-MEM-01-P1-01): Hot-layer (`MEMORY.md`) edits refresh the wrong projection — promoted/deleted hot memories are a lost update until workspace switch

- Layer: application
- Defect: eight `MEMORY.md`-mutating call sites call
  `refresh_instruction_projection` (which targets
  `eko:instruction-context` and excludes hot memory) instead of
  `refresh_hot_memory_projection` (which targets `eko:hot-memory-context`).
  `refresh_hot_memory_projection` has zero production callers outside
  workspace switch / bootstrap.
- Sites: `infra.rs:1175-1192` (Dreaming), `tauri/commands/memory.rs:126-145`
  + `:219-238` (GUI add/delete hot), `tui/events.rs:2839-2857` +
  `:2913-2927` (TUI `/remember` / `/forget` Hot),
  `cli/cmd_impls/all.rs:123-138` + `:194-209` (CLI equivalents).
- Impact: the headline capability of Dreaming (recall-driven promotion
  to a stable prompt prefix) and `/remember` auto-promotion is silently
  broken on the primary surface. The framework's per-turn
  `TURN_MEMORY_CONTEXT_PROJECTION` recall partially surfaces promoted
  memories when they match the current query, but the *stable* hot-layer
  prefix is frozen.
- Direction: replace the 8 wrong-target call sites with
  `refresh_memory_projections` (idempotent — refreshes both). Widen
  `AgentPool::refresh_instruction_context` to refresh both projections
  (resolves X-MEM-01-P2-01 simultaneously). The learned-rules.md sites
  (`panels.rs:1561`, `evolution.rs:1489`) stay on
  `refresh_instruction_projection` — they target the correct marker.

### 7.3 P2 — fix next

#### **X-EVT-01-P2-01** (canonical; absorbs A-SRF-04-P2-01 partially): Cancelled-vs-Error collapse — only GUI recovers; TUI / REPL / channels render cancel as error

- Layer: framework root cause (F-RCT-03-P2-02: `ReactAgent` overrides
  `chat_stream_with_cancel` / `execute_stream_with_cancel` without
  wrapping `cancel_aware_stream`, so `AgentEvent::Cancelled` is never
  emitted) + application per-surface compensation asymmetry (only
  `chat.rs:704-712` polls `cancel.is_cancelled()` post-`drive_chat`).
- Defect: every cancelled chat turn on TUI / REPL / channels surfaces as
  `Error{source:"agent_stream", message:"agent stream ended without a
  terminal event"}`. The `Cancelled` arm of every consumer's match is
  dead code for ReactAgent-driven turns.
- Direction: (1) **Framework fix (preferred)**: wrap `cancel_aware_stream`
  in `ReactAgent`'s overrides per F-RCT-03-P2-02. Makes every surface's
  `Cancelled` arm live and obsoletes the GUI's post-hoc compensation.
  (2) **Application fix (parity backstop)**: add post-`drive_chat`
  `TurnStatus{status:"cancelled"}` emission to TUI `send_to_agent`, REPL
  `run_repl_turn`, and channels `handle` — mirroring `chat.rs:704-712`.

#### **X-BND-01-P2-01** (canonical; absorbs X-STA-01-P2-02 + F-MEM-01-P2-01 + framework FW-MEM-002 + A-STATE-01 recurring parent-dir fsync concern): `atomic_write` is duplicated 6 times with inconsistent parent-directory fsync

- Layer: framework + application (the canonical helper belongs in
  framework `echo-state::util` or `echo-core::utils`; 4 of 6 sites are
  in the application)
- Defect: 6 implementations of the same tmp → write → sync_all → rename
  primitive that differ in tmp naming, error type, and — critically —
  whether the parent directory is fsynced after rename. Only 2 of 6
  (`echo-state/src/memory/file_conversation.rs:494`,
  `echo-agent/src/state/file.rs:210`) call `sync_parent_directory`.
- Impact: a crash after rename but before the directory entry is durable
  can lose the rename. On Linux ext4 (default mount) this is a real
  window. The framework is internally inconsistent across its own two
  copies. A future fix (e.g. switching to `tmpfile(2)`, handling
  cross-device rename) must be applied in 6 places.
- Direction: extract one `pub fn atomic_write(path: &Path, bytes: &[u8])
  -> io::Result<()>` (with parent-dir fsync) into framework util,
  re-export through the facade, replace all 6 call sites. `write_json_atomic`
  becomes `serialize` + canonical `atomic_write`. Per AGENTS.md cleanup
  rule, the 5 redundant copies are deleted in the same change.

#### **X-STA-01-P2-03** (NEW cross-cutting): Conversation deletion does not cascade to TaskRuntime runs or RuntimeStateStore checkpoints — both directories leak permanently

- Layer: application
- Defect: Tauri `delete_conversation` (`conversations.rs:585-640`) and
  TUI `/delete-session` (`events.rs:3067-3102`) call
  `store.delete_conversation`, `tool_executions.remove_conversation`
  (GUI only — A-STATE-01-P2-02 prior on TUI),
  `cleanup_tool_output_scope`, and `cleanup_user_input_scope`. Neither
  calls a `delete_runs_for_conversation` (does not exist) or
  `FileRuntimeStateStore::clear_conversation` (zero production callers).
- Impact: orphaned `~/.eko/tasks/{run_id}/` (events.jsonl + plan.json +
  run-state.json, often multiple per conversation) and
  `~/.eko/runtime_state/<safe(id)>/` (checkpoint.json + nodes.json)
  accumulate across create/delete cycles. Each events.jsonl can be large
  (full subagent traces + tool payloads). For reused conversation_id
  (low probability but possible), `latestRunForConversation` surfaces the
  stale run. Privacy concern: a user deleting a sensitive conversation
  expects its content gone.
- Direction: (1) add `delete_runs_for_conversation(conversation_id)` to
  `TaskRuntimeStore`; (2) wire the application `state_store` (constructed
  in `infra.rs:1246-1267`) into `TauriState` so
  `delete_conversation` can call `state_store.clear_conversation(&id)`;
  (3) extract `AppState::delete_conversation_cascade(id)` calling all 6
  cleanup primitives, called from BOTH Tauri and TUI (resolves
  A-STATE-01-P2-02 in the same change).

#### **X-PLG-01-P2-01** (canonical; absorbs F-SKL-01-P2-01 + F-PLG-01-P2-01): Collision non-determinism is systemic across both framework loaders and is inherited asymmetrically by the EKO adapter

- Layer: framework (`echo-execution/src/skills/external/loader.rs:120-147,
  198` + `echo-core/src/plugin/registry.rs:130-134, 190, 235`)
- Defect: skill loader uses `tokio::fs::read_dir` (filesystem order) +
  first-scope-wins + `warn!("shadowed")`; plugin registry uses
  `std::fs::read_dir` (filesystem order) + last-scanned-wins + silent
  insert. The two loaders therefore apply contradictory cross-scope
  precedence for the same conceptual operation.
- Impact: a same-named collision within a scope makes the winner
  filesystem-dependent. Cross-scope: User-scope plugin silently
  overwritten by Project-scope. Adapter's
  `state.framework_components` records source tags for plugins that may
  have been silently overwritten. No security impact (both candidates
  are user-installed under trusted scopes).
- Direction: single coordinated fix across both loaders: sort `read_dir`
  entries by path; emit `warn!` naming both paths on collision; agree
  on first-scope-wins for both. Belongs in framework
  (`echo-execution/src/skills/external/loader.rs` +
  `echo-core/src/plugin/registry.rs`), not the adapter.

#### **X-STA-01-P2-01**: TaskRuntime `events.jsonl` has no partial-tail recovery — a single truncated final line makes the run unreadable and unwritable

- Layer: application (`file_shadow.rs:362-379` read_events errors on first
  malformed line; `next_seq` uses read_events, so appends also fail)
- Defect: a partial last line (e.g. from SIGKILL between `write_all` and
  `sync_all`) bricks the run: unreadable (rewrite_plan fails) AND
  unwritable (next_seq fails). The tool-execution journal has the fix
  (`read_journal_repairing_last_line` at `tool_execution.rs:770-809`:
  truncate the partial last line and continue); the TaskRuntime shadow
  predates it.
- Impact: medium. Trigger window is narrow but failure mode is total
  without manual JSONL editing.
- Direction: factor the tool-execution repair logic into a shared
  `read_jsonl_repairing_last_line` helper; route
  `file_shadow::read_events` through it under the per-run write lock.

#### **X-AUT-01-P2-01**: Web-service XSS/SSRF threat model excluded by AGENTS.md is re-invoked as the justification for gates and guards across the Tauri IPC layer

- Layer: application (documentation / threat-model consistency)
- Defect: AGENTS.md explicitly excludes the XSS/SSRF threat model for
  EKO; the literal `require_full_auto` gate was removed. But the
  narrative persists in 5+ comments: `terminal.rs:46-49, 310-315`,
  `path_validator.rs:7-9`, `mcp.rs:110-119, 162-168, 203-204, 383-385`,
  `error.rs:1-10`. Two groups: (a) legitimate protections
  mis-justified; (b) actual over-gating inherited from A-INT-01-P1-01.
- Impact: future contributors reading these comments will reason from
  the wrong model and produce more over-gated gates "to match existing
  XSS defense."
- Direction: single threat-model-narrowing pass coordinated with
  X-SRF-01-P1-01 (the actual gate fix) and A-HITL-01-P2-02 (`IpcAuth`
  deletion). Rewrite comments to cite local-valid categories (data-loss
  / framework-bug / local-universal secret safety).

#### **X-TOL-01-P2-01**: Non-streaming tools lose `truncated` + spill `metadata` at the framework `execute_tool_with_policy` boundary; the GUI lazy reader cannot reach the spilled artifact content

- Layer: framework root cause (`snapshot.rs:1242` returns
  `Result<String, ToolCallFailure>`; `ctx.result.truncated` +
  `.metadata` in scope but not returned) + application asymmetric
  recovery (`pending_tool_completions` recovers metadata only for
  streaming tools via `ToolStream::Complete`).
- Defect: `AgentEvent::ToolResult` carries only `{call_id, name,
  output: String}`. Non-streaming spilled tools (the majority of the
  tool surface: read_file, write_file, edit_file, git tools,
  web_fetch, sql_query, list_dir, most MCP tools) lose both
  `truncated` and structured `artifact_path` / `artifact_sha256`
  metadata at the framework boundary. The model can still `read_artifact`
  via the textual pointer; the GUI lazy reader cannot.
- Impact: medium. Not strict data loss (artifact on disk) but real
  supervision defect on the common non-streaming path. The
  `manifest.truncated` label silently mis-reports output as complete.
- Direction: (a) widen `execute_tool_with_policy` to return
  `Result<ToolResult, ToolCallFailure>` (cleaner; breaking wire change);
  (b) parse the spill pointer out of output text at the EKO adapter
  (smaller blast radius, brittle to wording changes).

#### **X-TOL-01-P2-02**: Tool-execution wire types (`ToolFailure`, `ToolExecutionDetailManifest`, etc.) are hand-written TypeScript, not generated from Rust source

- Layer: application (`web-frontend/src/types/api.ts:55-116` vs
  `echo-agent-app-core/src/tool_execution.rs:63-150`)
- Defect: the task-runtime family uses `#[derive(TS)]` (gold standard
  per A-FE-01); the tool-execution family does not. Adding a Rust
  enum variant serializes correctly at runtime but the TS union
  silently lacks the literal — no compile error, no type-narrowing arm.
- Direction: add `#[derive(TS)]` with `#[ts(export, rename = ...)]` to
  the tool-execution types in `tool_execution.rs` (and re-exported
  framework types in `echo-core/tools/mod.rs`); delete the hand-written
  copies in `types/api.ts`.

#### **X-TOL-01-P2-03**: No end-to-end fixture drives any of {invalid-args, timeout, cancel, partial-side-effect} through the framework → EKO → frontend path

- Layer: application (test gap)
- Defect: framework tests (F-RCT-04, 29 tests) cover framework-internal
  behavior; EKO repository tests cover the success path; frontend store
  tests cover reducer monotonicity with synthetic records. The seam
  between them has no fixture. The existing
  `tool_failure_boundary_persists_recovery_contract` test
  (`store.rs:2857-2895`) constructs a `PartialSideEffect` failure with
  `postcondition` but asserts ONLY `failure.category`; `recovery`,
  `side_effect`, `postcondition` survival is not verified.
- Direction: cross-layer fixture in `echo-agent-app-core/tests/` or
  `#[cfg(test)]` in `chat.rs` driving each of the 4 categories through
  `TauriChatSink`-equivalent adapter into a real
  `ToolExecutionRepository`, asserting every field.

#### **X-BND-01-P2-02**: The framework `TaskSubagent` contract surface is dead — superseded by `RuntimeDagController`

- Layer: framework (deletion candidate)
- Defect: `TaskSubagent` trait (`runtime.rs:331`) has zero implementors
  in either repo; its return types `TaskExecutionSummary` /
  `SuggestedTask` are never produced in CLI production; CLI's adapter
  converter `to_runtime_summary()` has only a `#[test]` caller. The
  framework's own `RuntimeDagExecutor` does not use it.
- Impact: misleading API surface — a framework consumer reading
  `echo_agent::tasks::TaskSubagent` believes it is the extension point;
  `RuntimeDagController` is.
- Direction: delete `pub trait TaskSubagent`, its return types, and the
  re-exports. Keep `TaskSubagentContext` and `TaskClaim` (used by
  `RuntimeDagController`). Drop the CLI `to_runtime_summary` /
  `to_runtime_suggested_task` converters and their test.

#### **X-BND-01-P2-03** (reaffirms F-CORE-01-P2-01 at cross-repo boundary): `GLOBAL_EVENT_BUS` / `EventBus` are dead framework infrastructure

- Layer: framework (deletion candidate)
- Defect: zero `.send()` / `.subscribe()` / `EventBus::new` /
  `EventBus::default` callers in either repo (outside the definition
  file). Real event distribution uses direct stream composition.
- Direction: delete `echo-agent/src/event_bus.rs`, the `pub mod
  event_bus;` at `lib.rs:39`, and any re-export.

#### **X-INV-01-P2-01** (mirrors framework FW-TOOLS-003): IQR outlier detection panics on a numeric column with exactly 4 values

- Layer: framework (`echo-tools/src/data_quality.rs:253-254`)
- Defect: `sorted[3 * n / 4.min(n - 1)]` evaluates to `sorted[4]` for
  `n=4`, but valid indices are 0..=3. Method-call precedence binds
  `4.min(n-1)` to the divisor, not the index. Existing test uses n=9,
  so the bug stayed latent.
- Impact: any agent invoking `outlier_detection` with `method=iqr` on a
  4-row numeric column crashes the process. Bounded by `data` feature
  gate.
- Direction: replace both `sorted[...]` indexes with `sorted.get(...)`
  and handle `None`, or replace the body with the existing safe
  `quantile()` helper from `statistics.rs:195`. Add n=4 regression test.

#### **X-EVT-01-P2-02**: Channel renderer (`aggregate_by_sentence`) is the only consumer that silently drops unmatched `AgentEvent` variants

- Layer: application (`channels.rs:625` `_ => {}`)
- Defect: swallows `ThinkStart`, `ThinkEnd`, `LlmUsage`, `ToolCall`,
  `ToolResult`, `ToolError`, `ToolStream`, `ToolBatchStart`,
  `ToolBatchEnd`, `ContextCompressed`. Contrast: TUI surfaces them as
  `Notice`; Tauri surfaces them via typed `Notice`. The most-consequential
  drop is `ContextCompressed` — a user whose context was compacted gets
  no indication; answer quality may degrade without an observable signal.
- Direction: at minimum change to `tracing::debug!` log; optionally
  surface `ContextCompressed` as an `OutboundMessage`.

#### **X-EVT-01-P2-03**: `validate_event_trajectory` is exported but unused — terminal monotonicity enforced only by the envelope wrapper's `break`, not independently verified

- Layer: framework (`event_envelope.rs:197-295`)
- Defect: in `prelude` re-export; zero production callers. The
  "exactly one terminal" / "contiguous sequence" / "no duplicate
  event_id" invariants depend entirely on
  `envelope_event_stream_after`'s `break`. If that `break` is ever
  removed or weakened, no validator catches the regression in production.
- Direction: wire `validate_event_trajectory` into
  `envelope_event_stream_after`'s tail as a `tracing::warn!` (not
  `debug_assert!` per AGENTS.md no-panic rule); or stop exporting it and
  document as test-only helper.

#### **X-MEM-01-P2-01** (re-affirms A-MEM-01-P2-01): `AgentPool::refresh_instruction_context` doc claims hot-memory refresh but only refreshes instructions

- Layer: application (`agent_pool.rs:686-710`)
- Direction: widen to refresh both projections (preferred — minimal
  building block for X-MEM-01-P1-01's fix) or fix the doc to name
  instructions only.

#### **X-MEM-01-P2-02** (re-affirms A-MEM-01-P2-02): CLI `/remember`, `/forget`, and rule-promote refresh only the primary agent — pooled/background agents diverge

- Layer: application (`cli/cmd_impls/all.rs:123-142, 194-214`,
  `evolution.rs:1471-1496`)
- Defect: CLI paths mutate the file and refresh the primary agent but
  not the pool. Contrast: GUI/TUI paths follow with
  `pool.refresh_instruction_context()`. Inconsistent with GUI/TUI.
- Direction: extract a single
  `refresh_memory_after_edit(primary, pool, root)` helper in
  `unified_memory.rs` and call it from all 8 sites (4 MEMORY.md +
  4 learned-rules.md) so fan-out cannot drift again.

#### **X-SRF-01-P2-01** (absorbs A-SRF-04-P2-01): REPL and channels Chat/Auto turns have no externally reachable cancel — TUI parity gap

- Layer: application (`repl.rs:533`, `channels.rs:244`,
  `chat_driver.rs:240-252`)
- Defect: per-turn `CancellationToken` created but never registered
  with `register_run_cancellation` (which fires only for
  `InteractionMode::Task`). TUI solves it via `app.active_cancel`; REPL
  + channels did not.
- Direction: install `tokio::signal::ctrl_c()` handler on REPL; add
  `/cancel` slash command on channels; surface an accessible cancel
  handle on both.

#### **X-SRF-01-P2-02** (absorbs A-SRF-04-P2-03 + A-BOOT-01-P2-02): Channels-only entry skips SchedulerRunner + BackgroundTaskService — cron and background capabilities unavailable

- Layer: application (`main.rs:357-403`, `modes.rs:32-64, 118-235`)
- Direction: call `start_headless_services` in the channels-only branch
  before spawning `run_channels_mode`, mirroring TUI/CLI branches at
  `main.rs:258-274`.

#### **X-SRF-01-P2-03** (absorbs A-SRF-04-P2-02): Cron runs recovered to Paused on restart but never auto-resumed — recovery parity gap vs background

- Layer: application (`tasks/service.rs:474, 516, 541, 552, 563` all
  filter on `conversation_id.starts_with("background:")`)
- Defect: cron runs reconcile to Paused on next boot but nothing wakes
  them; the next cron tick fires a NEW run for the same `CronTask`,
  duplicating the work.
- Direction: extend `BackgroundTaskService::resume_pending`'s filter to
  also cover `conversation_id.starts_with("cron:")`, or drop the prefix
  filter and resume any Paused run the recovery blockers allow.

#### **X-SRF-01-P2-04** (absorbs A-SRF-01-P2-02): TUI subagent internal lifecycle collapses to a counter — 11 of 16 framework `SubagentEvent` variants silently dropped

- Layer: application (`tui/events.rs:5343-5434`,
  catch-all `_ => {}` at `:5428`)
- Defect: TUI handles 5 variants (`DispatchStarted`, `DispatchToolStarted`,
  `DispatchCompleted`, `DispatchFailed`, `DispatchCancelled`); drops
  `DispatchToolCompleted`, `DispatchTokenDelta`, `DispatchThinkingDelta` /
  `Started` / `Ended`, `DispatchLlmUsage`, `DispatchIsolationObserved`,
  `Registered` / `Unregistered`, `TeamCreated` / `TeamDissolved`. GUI
  persists per-tool detail into `ToolExecutionRepository` and emits to
  `execution://event` for the frontend dashboard.
- Direction: extend `update_subagent_runs` to handle at least
  `DispatchLlmUsage` and the thinking-trace variants.

#### **X-SRF-01-P2-05** (absorbs A-INT-01-P2-02): No LSP interactive management surface on any mode — framework `LspManager::restart_server` has zero application callers

- Layer: application
- Defect: framework's restart primitive exists (`lsp/manager.rs:105-108`);
  no `/lsp` slash command, no `lsp.rs` Tauri module. After the first LSP
  crash/hang, every subsequent call silently no-ops or hangs until full
  EKO restart.
- Direction: add Tauri command `restart_lsp_server(language)` and TUI
  `/lsp restart <lang>` slash command.

#### **X-SRF-01-P2-06** (absorbs A-SRF-02-P2-01): GUI window close orphans PTY shells — `TerminalManager.close_all()` never invoked

- Layer: application (`tauri/terminal.rs:256-267`,
  `desktop.rs:256-268`, `tauri/mod.rs:69-310`)
- Defect: no `on_window_event` handler registered; `close_all` exists but
  is unreachable. On window close, every live `PtySession`'s child shell
  is reparented to launchd and keeps running.
- Direction: register `on_window_event(CloseRequested)` →
  `app.state::<TauriState>().terminal_manager.close_all()`.

### 7.4 P3 — incremental / opportunistic

Per AGENTS.md "随手清理是强制要求" — address when touching the module.

#### **X-TSK-01-P3-01** (carries A-TSK-01-P2-02 + A-TSK-03-P3-02): The `types.rs:917-920` doc claim "shared `TaskStatus` remains authoritative and lossless" is inaccurate for the persisted event-stream path

- Layer: adapter (documentation)
- Defect: `append_task_status_event` writes 8-state `TodoStatus` to the
  authoritative event stream, not the 10-state framework `TaskStatus`.
  `Retrying`/`Paused` cannot survive a `rewrite_plan`. Latent because
  the EKO executor never produces those statuses today.
- Direction: narrow the doc to state that `Retrying`/`Paused` are never
  produced by the EKO executor path (deliberate projection boundary).

#### **X-INV-01-P3-01**: Stale "sqlite-backed" doc comment in CLI `infra.rs:125`

- Layer: application (documentation)
- Direction: replace "sqlite-backed" with "file-backed
  (RuntimeStateStore-backed in EKO)".

#### **X-BND-01-P3-01**: `WorktreeError` is defined byte-identically in framework and CLI

- Layer: adapter (`echo-agent/src/agent/subagent/worktree.rs:35-52` vs
  `echo-agent-cli/.../tasks/task_runtime/worktree.rs:117-145`)
- Direction: add `From<std::io::Error>` to the framework type; reuse it
  from the CLI.

#### **X-BND-01-P3-02**: CLI task-runtime worktree reimplements the `git worktree add` invocation

- Layer: adapter
- Direction: reuse `echo_tools::git_worktree::create_worktree` for the
  primitive and layer the prune/merge-base logic on top, OR document
  why a separate git invocation is required.

#### **X-PLG-01-P3-01**: `register_lifecycle` is a pub adapter API with zero production callers; deactivated-but-not-unregistered semantics are a latent stale-entry risk

- Layer: application (`plugin_runtime.rs:387-410`)
- Direction: (a) document the caller responsibility (cheapest); (b) make
  `apply_candidate` call `lifecycle.reconcile` instead of
  `deactivate_all` + `activate_enabled` if a production caller is added;
  (c) `#[cfg(test)]`-gate or `#[doc(hidden)]` until a real consumer
  arrives.

#### **X-AUT-01-P3-01**: `path_validator.rs` secret-denylist doc-comment invokes the excluded XSS threat model; the gate itself is an appropriate local secret-protection

- Layer: application (documentation)
- Direction: rewrite `path_validator.rs:7-9` to cite local-universal
  secret safety; drop the "XSS exfiltrating credentials" framing.

#### **X-SRF-01-P3-01** (absorbs A-SRF-04-P3-01): Channels slash-command surface is reduced — only 5 of ~20 REPL commands wired

- Layer: application (`channels.rs:313, 339, 381, 394, 407` — `/mode`,
  `/trace`, `/analysis`, `/papers`, `/skills` only)
- Direction: factor a `ChannelCommandDispatcher` mirroring REPL's
  `CommandRegistry` for the IM-applicable subset (at minimum `/cron`,
  `/mode`, `/skills`, `/trace`).

#### **X-SRF-01-P3-02** (absorbs A-TOOL-01-P3-02): TUI has no interactive terminal pane — only the GUI PTY exists

- Layer: application
- Direction: a future TUI terminal widget should reuse the same consent
  + audit semantics from `tauri/terminal.rs`, not a parallel
  implementation.

#### **X-SRF-01-P3-03**: Tool-execution persistence is GUI-only — TUI/CLI/channels/cron/background all drop tool-execution detail on session exit

- Layer: application
- Direction: long-term, extract a unified `ToolExecutionObserver` into
  `ChatResources` (resolves A-CHAT-01-P2-01 + A-SRF-02-P2-03 + this in
  one pass); short-term, document the asymmetry.

#### **X-EVT-01-P3-01**: Chat `AgentEvent`s are not persisted for replay — asymmetric durability vs subagent / tool / task-runtime events

- Layer: application
- Defect: `drive_chat_inner` forwards each envelope to `sink.on_event`
  and discards; reload recovers the final assistant content but not the
  thinking segments / budget notices / guard notices / context-compressed
  notices.
- Direction: (1) persist the envelope stream (structural); (2) document
  the asymmetry (no-code-change floor); (3) persist only the
  audit-relevant subset (best ROI).

#### **X-EVT-01-P3-02**: `chatEventHandler.ts` is a non-exhaustive switch — TypeScript cannot catch a future `ChatEvent` variant addition

- Layer: application (`web-frontend/src/hooks/chatEventHandler.ts:25-220`)
- Direction: add `default: { const _exhaustive: never = event;
  console.warn('[chatEventHandler] unhandled ChatEvent', _exhaustive);
  return; }` so TS flags any future variant addition at compile time.

#### **X-EVT-01-P3-03**: `chatStore.setRunStatus` has no terminal lock — terminal monotonicity relies on indirect `message_key` scoping

- Layer: application (`web-frontend/src/stores/chatStore.ts:391-397`)
- Direction: add a terminal-status guard mirroring `subagentRunStore`'s
  pattern (`if (prev && prev.status !== 'running') return s;`).

#### **X-STA-01-P3-01**: `add_artifact` and the `Artifact` struct have zero production callers — artifact persistence is dead duplicate authority

- Layer: application (`store.rs:1432-1460`, `types.rs:1418-1426`)
- Direction: (1) delete the dead surface (recommended under YAGNI — also
  resolves A-FE-02-P2-01's parallel `listReviews` gap); or (2) wire a
  producer (reviewer gate or executor). Owned by A-TSK-06's
  artifact-preservation follow-up.

#### **X-TOL-01-P3-01**: `analysis.rs::run_status` flattens `ToolFailureCategory` (7 variants) to `AnalysisRunStatus` (3 values); `recovery` / `side_effect` / `postcondition` discarded

- Layer: application (`analysis.rs:866-875`)
- Direction: one-line comment documenting the intentional flattening;
  widen if analysis ever needs finer granularity.

---

## 8. Cross-Cutting Patterns

These recur across multiple subsystems and represent systemic issues.
Fixing the pattern fixes a whole class of findings at once.

### 8.1 Dead Infrastructure & Dormant APIs

Same "scaffolded, never wired, pub-exported, doc-overstates" shape.
Recurring across at least 16 distinct sites in the two repos (extends
framework-review §4.1's 11 sites with cross-repo additions):

| Canonical finding | Item | Repo | Status |
|---|---|---|---|
| X-BND-01-P2-03 / FW-CORE-001 | `GLOBAL_EVENT_BUS` / `EventBus` | framework | zero producers/consumers |
| X-BND-01-P2-02 | `TaskSubagent` trait + FW `TaskExecutionSummary` / `SuggestedTask` | framework | zero implementors |
| FW-NBK-001 | `NotebookTracker` module | framework | zero live callers |
| X-PLG-01-P3-01 | `register_lifecycle` | application | 5 callers all `#[cfg(test)]` |
| X-STA-01-P3-01 | `add_artifact` / `Artifact` / `ArtifactProduced` / `list_task_artifacts` | application | zero production producers |
| X-EVT-01-P2-03 | `validate_event_trajectory` | framework | test-only, exported via prelude |
| A-HITL-01-P2-02 | `IpcAuth` / `IpcPermission` / `require_full_auto` / `require_not_strict` | application | zero callers |
| A-STATE-01-P2-01 | `Persistence` / `SessionSearchEngine` | application | dead (superseded by `FileConversationStore`) |
| FW-RCT-004 | `LoopDetector` + config plumbing | framework | only `#[cfg(test)]` |
| FW-RCT-005 | `process_steps` + `execute_tool_feedback_*` | framework | `#[allow(dead_code)]`; superseded by `run_core_loop` |
| FW-LLM-011/012 | `AdapterClient`, `DefaultLlmClient` | framework | superseded by `OpenAiClient` |
| FW-SUB-010/019/013 | `HandoffManager`/`HandoffTool`, `TopologyTracker`, `isolated::run_isolated` | framework | zero production consumers |
| FW-FEAT-001 | 5 dead Cargo features | framework | `= []`, gate nothing |
| FW-QUAL-002 | ~50 `#[allow(dead_code)]` annotations | framework | suppressed lint across many modules |

**Pattern**: scaffolding was added ahead of integration, the integration
never landed, and the `pub` surface + doc comments make the gap
invisible to `cargo check`. Only reachability grep finds it. AGENTS.md
"代码清理" + "删除框架代码的判定" branch 1 (superseded) directly apply.

### 8.2 Atomic-Write Drift (the largest live-duplicate pattern)

Six independent reimplementations of the atomic-replace recipe, only one
of which got the parent-dir fsync lesson right (A-STATE-01). The drift
spans both repositories and even the framework is internally
inconsistent. Closing this pattern is a single refactor (extract
`atomic_write` to framework util, migrate all 6 call sites) that
resolves X-BND-01-P2-01 + X-STA-01-P2-02 + F-MEM-01-P2-01 +
framework FW-MEM-002 in one pass.

### 8.3 Cancelled-vs-Error Collapse Across Surfaces

The framework's `ReactAgent` never emits `AgentEvent::Cancelled`
(F-RCT-03-P2-02). The GUI added a post-`drive_chat` polling workaround;
TUI / REPL / channels did not. The framework fix unblocks every
surface's existing `Cancelled` arm and obsoletes the GUI compensation.
The application fix is a localized backstop. This pattern spreads
across X-EVT-01-P2-01 + framework F-RCT-003 + every surface reducer.

### 8.4 Threat-Model Narrative Drift (XSS/SSRF)

The historical `require_full_auto` gate was correctly removed, but the
narrative that motivated it persists in 5+ comments across the Tauri
IPC layer (X-AUT-01-P2-01). The pattern is self-propagating: each new
gate added "to match existing XSS defense" extends the wrong model.
The fix is a single coordinated pass bundling: (a) the actual over-gating
fix (X-SRF-01-P1-01), (b) the narrative narrowing (X-AUT-01-P2-01 /
P3-01), (c) the dead `IpcAuth` deletion (A-HITL-01-P2-02).

### 8.5 Surface-Parity Gaps (multi-mode equivalence)

Eight parity gaps aggregate from A-SRF-* + A-INT-01 dependencies. All
are missing surface wiring (not architectural gaps); the framework
supplies every primitive the surfaces need (cancel tokens, slash-command
routing, service starters, `restart_server`, `close_all`,
`ToolExecutionRepository`). AGENTS.md's "X 模式 doesn't use Y"
anti-pattern is absent: no comment justifies a missing capability as
product policy; every gap is an undocumented absence the AGENTS.md
rule explicitly classifies as "待补的缺口,不是产品定位."

### 8.6 Hand-Written TypeScript Mirrors of Rust Enums

The task-runtime family uses `#[derive(TS)]` (gold standard); the
tool-execution family (X-TOL-01-P2-02) and the chat-event family
(X-EVT-01-P3-02, A-FE-01-P3-04) use hand-written TS unions. Adding a
Rust variant serializes correctly at runtime but the TS union silently
lacks the literal — no compile error, no exhaustiveness guard. Pattern
fix: migrate the stragglers to `#[derive(TS)]`.

### 8.7 Lossy Framework Seams on the Success Path

The framework's `execute_tool_with_policy` returns
`Result<String, ToolCallFailure>` — `String` on success, typed
`ToolCallFailure` on failure. The failure path preserves the full
taxonomy (`category`, `recovery`, `side_effect`, `retry_after_ms`,
`idempotency_key`, `postcondition`) end-to-end; the success path drops
`truncated` + spill `metadata` for non-streaming tools (X-TOL-01-P2-01).
The asymmetry is by accident: the seam predates the `TruncationStage`
enrichment. The streaming tools' `pending_tool_completions` workaround
in `TauriChatSink` exists because the seam was never widened.

### 8.8 Conversation-Deletion Cascade Incompleteness

The cascade was assembled incrementally: conversation store cleanup came
first; tool-execution got its own `remove_conversation`; tool-output
and user-input artifacts came via framework helpers; TaskRuntime and
RuntimeStateStore were never added (X-STA-01-P2-03). The TUI side also
misses `tool_executions.remove_conversation` (A-STATE-01-P2-02). A
single `AppState::delete_conversation_cascade(id)` helper called from
both surfaces closes both gaps.

### 8.9 UTF-8 / Byte-Length Safety Violations (cross-repo)

Two pre-existing sites reaffirmed unchanged (X-INV-01 V06-01):
`gitignore.rs:178-180` (Q-STA-01-P1-01, P1) and `rule.rs:51-56`
(F-SEC-01-P3-01, P3). No new violation introduced. The IQR site
(X-INV-01-P2-01) is a direct-index panic, not a UTF-8 violation.

---

## 9. Contradiction Resolution

The X-phase reports were produced sequentially with dependency handoffs
and are largely consistent. Three explicit reconciliations:

1. **`atomic_write` parent-dir fsync (converged):** A-STATE-01 first
   flagged the missing parent-dir fsync as an application concern;
   X-BND-01 V02-01 generalized it to a duplicate-authority pattern
   (6 sites); X-STA-01 V02-01 confirmed the TaskRuntime shadow is one
   of the 5 incorrect sites. Resolution: canonical helper belongs in
   framework, used by all 6. Recorded as X-BND-01-P2-01 (canonical),
   not three separate findings.
2. **Collision resolution precedence (contradictory across loaders):**
   F-SKL-01 (skill loader) and F-PLG-01 (plugin registry) each
   characterized the collision behavior from one side without noting
   the other side diverges. X-PLG-01 V01 surfaced the contradiction
   (skill: first-scope-wins + warn; plugin: last-scanned-wins + silent).
   Resolution: single coordinated fix in framework, both loaders
   converge on first-scope-wins + `warn!`.
3. **GUI MCP over-gating severity (aligned, not contradictory):**
   A-INT-01 filed it as P1 (user-capability regression); X-SRF-01 V01
   confirmed it is the only P1 parity gap (makes a capability
   unreachable on one surface); X-AUT-01 V04 confirmed it is the
   canonical instance of the excluded threat-model pattern. Resolution:
   X-SRF-01-P1-01 owns the priority; X-AUT-01-P2-01 owns the narrative;
   A-INT-01-P1-01 owns the implementation detail. All three should land
   in one patch.

Plus the framework-review's three reconciliations (SQLite durability
hypothesis confirmed; `AdapterClient`/`DefaultLlmClient` deletion valid
under branch 1; headless equivalence aligned with the parity mandate)
remain current.

---

## 10. Prioritized Action List

Ordered by severity × blast-radius × fix-cost (cheaper fixes that
unblock other work ranked higher). Each item links to its canonical
finding ID(s).

### Tier A — Fix first (P1 + cheapest P2 unblocks)

1. **X-SRF-01-P1-01 + X-AUT-01-P2-01 + A-HITL-01-P2-02**: drop the
   executable allowlist + private-range URL block in
   `validate_ipc_mcp_*`; rewrite the threat-model-narrative comments
   across `terminal.rs`, `path_validator.rs`, `mcp.rs`, `error.rs`;
   delete `IpcAuth`/`IpcPermission`/`require_full_auto`. Lands three
   findings in one patch and resolves the historical over-gating lesson
   at its root.
2. **X-MEM-01-P1-01 + X-MEM-01-P2-01 + X-MEM-01-P2-02**: replace the
   8 wrong-target MEMORY.md refresh sites with
   `refresh_memory_projections`; widen
   `AgentPool::refresh_instruction_context` to refresh both projections;
   extract `refresh_memory_after_edit(primary, pool, root)` and call
   from all 8 sites (4 MEMORY.md + 4 learned-rules.md). Lands three
   findings + resolves A-MEM-01's prior trio.
3. **X-BND-01-P2-01 (atomic_write consolidation)**: extract one
   canonical `atomic_write` with parent-dir fsync into framework
   `echo-state::util`; migrate all 6 call sites; delete the 5 redundant
   copies. Closes X-BND-01-P2-01 + X-STA-01-P2-02 + F-MEM-01-P2-01 +
   framework FW-MEM-002 in one refactor.
4. **X-INV-01-P2-01 (IQR panic)**: replace `sorted[...]` direct indexes
   with `sorted.get(...)` or use the existing safe `quantile()` helper.
   One-line fix; add n=4 regression test. (Same fix as framework
   FW-TOOLS-003.)
5. **X-EVT-01-P2-01 framework root-cause fix**: wrap `cancel_aware_stream`
   in `ReactAgent`'s overrides per F-RCT-03-P2-02. Makes every surface's
   `Cancelled` arm live and obsoletes the GUI's post-hoc compensation.

### Tier B — Fix next (remaining P2, high-impact)

6. **X-STA-01-P2-03 + A-STATE-01-P2-02 (deletion cascade)**: add
   `delete_runs_for_conversation`; wire `state_store.clear_conversation`;
   extract `AppState::delete_conversation_cascade(id)` called from both
   Tauri and TUI.
7. **X-STA-01-P2-01 (events.jsonl partial-tail recovery)**: factor the
   `read_journal_repairing_last_line` logic into a shared
   `read_jsonl_repairing_last_line`; route `file_shadow::read_events`
   through it.
8. **X-PLG-01-P2-01 (collision non-determinism)**: single coordinated
   fix across both framework loaders — sort `read_dir`, `warn!` on
   collision, converge on first-scope-wins.
9. **X-TOL-01-P2-01 (success-path spill drop)**: decide between (a)
   widening `execute_tool_with_policy` to return
   `Result<ToolResult, ToolCallFailure>` or (b) parsing the spill
   pointer at the EKO adapter.
10. **X-TOL-01-P2-02 + X-EVT-01-P3-02 (TS wire generation)**: migrate
    the tool-execution family to `#[derive(TS)]`; add the `default:
    never` exhaustiveness guard to `chatEventHandler.ts`.
11. **X-TOL-01-P2-03 (cross-layer fixtures)**: add 4 end-to-end tests
    for `{InvalidArguments, Timeout, Cancelled, PartialSideEffect}`
    through `TauriChatSink` into `ToolExecutionRepository`.
12. **X-EVT-01-P2-03 (validate_event_trajectory)**: wire it into
    `envelope_event_stream_after`'s tail as `tracing::warn!`, or stop
    exporting from `prelude`.
13. **X-BND-01-P2-02 (TaskSubagent deletion)**: delete the trait + its
    return types + re-exports; drop the CLI test-only converters.
14. **X-BND-01-P2-03 (GLOBAL_EVENT_BUS deletion)**: delete the file +
    re-export.
15. **X-SRF-01-P2-01 (REPL + channels cancel)**: install `ctrl_c`
    handler on REPL; add `/cancel` on channels; surface accessible
    cancel handles.
16. **X-SRF-01-P2-02 (channels-only services)**: route channels-only
    branch through `start_headless_services`.
17. **X-SRF-01-P2-03 (cron auto-resume)**: extend
    `BackgroundTaskService::resume_pending`'s filter to cover `cron:`.
18. **X-SRF-01-P2-04 (TUI subagent detail)**: extend
    `update_subagent_runs` to handle `DispatchLlmUsage` + thinking-trace
    variants.
19. **X-SRF-01-P2-05 (LSP surface)**: add `/lsp` (TUI) + `lsp.rs`
    (Tauri).
20. **X-SRF-01-P2-06 (terminal cleanup)**: register
    `on_window_event(CloseRequested)` → `close_all`.
21. **X-EVT-01-P2-02 (channel silent drop)**: change `_ => {}` to
    `tracing::debug!`; surface `ContextCompressed` as `OutboundMessage`.

### Tier C — Dedupe and converge (cross-cutting)

22. **Dead-infra sweep**: batch-delete the items in §5.2 (or wire them).
    Per AGENTS.md "随手清理是强制要求."
23. **Hand-written TS mirror sweep**: migrate the remaining families to
    `#[derive(TS)]`; delete the hand-written copies.
24. **Threat-model narrative sweep**: complete the pass started in Tier
    A item 1 across any remaining Tauri IPC sites.

### Tier D — Lower-priority P3

The 14 P3 findings are real but lower-impact. Address incrementally
when touching the relevant module. Notable clusters:

- **Documentation drift** (X-INV-01-P3-01 sqlite comment,
  X-TSK-01-P3-01 losslessness claim, X-AUT-01-P3-01 path_validator,
  X-TOL-01-P3-01 run_status flattening): one-line doc fixes.
- **Adapter duplication** (X-BND-01-P3-01 WorktreeError,
  X-BND-01-P3-02 worktree reimplementation): converge when next
  touching the worktree subsystem.
- **Surface polish** (X-SRF-01-P3-01 channels slash set,
  X-SRF-01-P3-02 TUI terminal, X-SRF-01-P3-03 tool-execution
  persistence asymmetry, X-EVT-01-P3-01 chat events fire-and-forget,
  X-EVT-01-P3-03 chatStore terminal lock): long-term unified
  `ToolExecutionObserver` resolves several at once.
- **Dead application surface** (X-STA-01-P3-01 `Artifact` /
  `add_artifact`): delete or wire.
- **Dormant pub API** (X-PLG-01-P3-01 `register_lifecycle`): document
  or `#[cfg(test)]`-gate.

---

## 11. What Is Clean (Positive Conclusions)

To balance the defect inventory, the cross-repo review confirmed several
structural invariants hold:

- **One revisioned TaskRun graph model.** Framework `TaskSpec` is the
  canonical task specification; framework `RevisionedTaskGraph` is the
  canonical graph; framework `TaskRevisionService` is the sole mutator;
  framework `TaskPatchEngine::apply_operations` is the sole patch
  semantics authority. EKO's projection is field-by-field lossless for
  every spec field (X-TSK-01 V01 + V02).
- **No parallel task/plan/todo CRUD.** Zero `todo_write`/`plan_create`/
  `plan_patch`/`plan_execute` tools anywhere; only `task_execute`
  (permitted) added by CLI (X-TSK-01 V03, X-INV-01 V03).
- **Plugin/skill/hook lifecycle is fully reversible.** Discover →
  prepare → activate → use → reload → unload each have a defined
  inverse. Reload is a full unload+rewire (atomic swap), not in-place
  mutation. The 8-checkpoint failure rollback is comprehensive;
  checkpoints 1-6 exact, 7-8 best-effort with the candidate fully
  unloaded before previous is restored (X-PLG-01 V02 + V03).
- **Adapters are thin.** No EKO adapter owns scheduling / DAG loop /
  generic retry / validator / deadlock-detection authority (X-TSK-01
  V02, X-BND-01 V04, X-PLG-01 V01, X-MEM-01 § Layering Decision).
- **Per-attempt subagent identity is deterministic.**
  `subagent_run_id = execution_id = {run_id}:{task_id}:{revision}:{attempt}`
  is stable across frontend and backend; the frontend
  `subagentRunStoreKey` is a faithful echo, not a parallel authority
  (X-STA-01 V01, A-FE-02 V01).
- **Tool failure taxonomy survives end-to-end.** `category`, `recovery`,
  `side_effect`, `retry_after_ms`, `idempotency_key`, `postcondition`
  are preserved from `ToolResult.failure` → `AgentEvent::ToolError` →
  `tool_executions.finish(false, ...)` → `manifest.failure` → TS
  (X-TOL-01 V02).
- **The two-lane architecture is sound.** Every mode enters through
  `drive_chat` (chat lane) or `execute_run` / `drive_agent_run` (task
  lane). No third lane, no parallel implementation (X-SRF-01 V01).
- **Surface parity is high on primary paths.** 61 of 102 cells in the
  17-capability × 6-mode matrix are full-parity ✅ (X-SRF-01 V01).
- **AGENTS.md invariants are largely intact.** 5 of 6 invariants hold
  cleanly (X-INV-01); the only regression is the IQR panic
  (X-INV-01-P2-01), which is a single localized fix.
- **The agent-vs-user permission boundary is correctly separated.**
  `permission_mode` controls agent automation only; direct-user terminal
  / file picker / MCP / browser / TUI shell escape bypass the
  `execute_tool_with_policy` pipeline by design (X-AUT-01 V01 + V04).
- **Projections survive compression.** Both EKO markers carry the
  `<echo-agent-context-projection-v1>` envelope, are protected by
  `is_context_projection_message` → `is_protected`, and never enter
  compressor input (X-MEM-01 V03).
- **Atomic-write durability is correct at the framework layer** for the
  two framework sites that include parent-dir fsync
  (`FileConversationStore`, `FileRuntimeStateStore`); the application
  TaskRuntime shadow is the inconsistent one (X-STA-01 V02).

---

## Appendix A: Source Index

All 10 X-phase task reports + the framework-review synthesis, in the
order referenced:

X-AUT-01, X-BND-01, X-EVT-01, X-INV-01, X-MEM-01, X-PLG-01, X-SRF-01,
X-STA-01, X-TOL-01, X-TSK-01. Plus `framework-review.md` (S-FW-01).

Each report lives at
`docs/comprehensive-review/zcode-glm/tasks/<ID>.md` with its validation
evidence at `docs/comprehensive-review/zcode-glm/validations/<ID>/`. All
reports are at baseline `echo-agent` `9b0e0fa` / `echo-agent-cli`
`b3b2e81`.

### Key F-/A-/Q-phase dependency reports cited

F-CORE-01, F-RCT-03, F-RCT-04, F-RCT-05, F-MEM-01, F-MEM-02, F-CTX-01,
F-CMP-01, F-EXT-01, F-EXT-02, F-EXT-03, F-HITL-01, F-INT-01, F-INT-02,
F-LLM-02, F-LLM-03, F-OPS-01, F-PLG-01, F-SEC-01, F-SKL-01, F-SUB-01,
F-SUB-02, F-TSK-01, F-TSK-02, F-TSK-03, F-WFL-01, F-NBK-01, F-MAG-01,
F-MAC-01, F-INTENT-01, F-EVO-01, F-API-01, F-FEAT-01, F-TST-01,
F-REL-01, F-OPS-01, F-CORE-01, Q-STA-01, Q-DEP-01.

A-BOOT-01, A-CHAT-01, A-DOM-01, A-EVO-01, A-FE-01, A-FE-02, A-FE-03,
A-HITL-01, A-INT-01, A-INP-01, A-MEM-01, A-OBS-01, A-OUT-01, A-PLG-01,
A-PROJ-01, A-SRF-01, A-SRF-02, A-SRF-03, A-SRF-04, A-STATE-01,
A-SUB-01, A-TOOL-01, A-TSK-01, A-TSK-02, A-TSK-03, A-TSK-04, A-TSK-05,
A-TSK-06, A-CFG-01.

### Conditions that make this synthesis stale

- Any commit that resolves one of the 39 canonical findings invalidates
  that finding's row in §7 and the corresponding merge in §5.
- A baseline change (`echo-agent` `9b0e0fa` or `echo-agent-cli`
  `b3b2e81`) requires re-running the source X-phase reports' V*
  validations and re-checking the boundary-gate table in §3.
- A new mode added (e.g. `--daemon`, plugin-supplied channel, HTTP
  webhook entry) requires a new column in X-SRF-01's capability matrix.
- A new framework `AgentEvent` / `ChatEvent` / `ToolFailureCategory`
  variant requires re-running the X-EVT-01 / X-TOL-01 cross-surface
  conformance matrices.
