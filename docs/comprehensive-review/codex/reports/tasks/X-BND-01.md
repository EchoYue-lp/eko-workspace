# X-BND-01: Capability placement and duplicate authority map

> Status: complete
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework inspected through committed HEAD; external CLI `Cargo.lock` excluded

## Question

Across both repositories, which concepts are correctly framework, EKO policy,
or thin adapters, and where do semantic duplicates remain?

## Scope And Inputs

- All 38 complete framework and 29 complete application atomic reports: 67
  task reports containing 444 exact atomic findings at this review snapshot.
- Current committed definitions/reachability for TaskRevisionService,
  PlanValidator, ManagedTask/TaskManager, ReAct loops, Subagent Team/Handoff,
  Workflow, EKO revision adapter, Task controller, Tauri workflow/diff/event,
  artifact paging and dormant public/application surfaces.
- Root AGENTS.md placement, reuse, no-duplicate, framework-independence,
  no-CLI-SQLite, local threat-model and Subagent-only constraints.

No source, build, test, fixture or network operation is part of this task.

## Capability Placement Map

| Capability | Framework authority | EKO policy/owner | Thin adapter duty | Current verdict |
|---|---|---|---|---|
| ReAct/LLM/Tool loop | Agent/ReAct, provider-neutral messages, Tool/ToolResult/ToolFailure | model/provider selection, credentials, surface defaults | lossless config and event projection | correct placement; internal duplicate provider/loop remnants remain |
| Event lifecycle | AgentEvent/EventEnvelope | durable ordinary turn and surface replay | carry identity/order/terminal unchanged | correct primitive, thick/lossy EKO adapters |
| Task graph | PlanValidator + TaskRevisionService + RuntimeDagExecutor | DomainProfile, attended policy, files/worktree/review/UI | lossless EkoRevisionedTaskStore and dispatch hooks | main path correctly shared; legacy framework and EKO controller semantics remain |
| Subagent | definition/catalog/executor/invocation/cancel/outcome | role sources, EKO prompts, pool generation, worktree and acceptance | source preparation and one catalog projection | Team/Handoff/public DTOs still own parallel lifecycles |
| Tool output/artifact | schema, effective invocation, typed terminal, canonical artifact descriptor/reader | local retention root, conversation/run ownership, lazy UX | persist/render the canonical envelope | EKO reimplements terminal inference and artifact paging |
| Skill/plugin/hook | source identity, dependency/lifecycle receipts, reversible components | enabled roots, product components, pool/surface propagation | one atomic component/catalog generation | framework/app split is right; lifecycle transaction convergence pending X-PLG |
| Memory/context/compression | Store traits/implementations and generic strategies | file-backed EKO selection, instruction precedence, hot memory | configure/refresh without duplicate persistence | correct placement; public framework SQLite remains valid and CLI must not enable it |
| Workflow | generic Graph/checkpoint/loader/runtime | local library, naming, UI/commands/automation | call one app service backed by Graph | GUI Tauri module currently owns product CRUD/execution |
| Project/workspace/diff | generic file/Git/diff primitives | workspace identity, drafts, coding UX | root-bound paths and one diff projection | GUI has second diff; inert index/tracker paths |
| Export/output | generic Tool artifacts/process cancellation | EKO formats, lineage, delivery parity | typed request/result projection | three disconnected EKO output authorities |
| Integrations/security | MCP/LSP/A2A/channel mechanisms, sandbox/guard primitives | enabled servers and local interaction policy | translate without cloud-style over-gating | placement sound; lifecycle/secret defects remain atomic |

## Findings

### X-BND-01-P1-01: The canonical Task graph is shared correctly, but the framework still exposes a second graph/state/store authority

- Priority: P1; confidence: high; layer: framework.
- Evidence: committed `echo-orchestration/src/tasks/revisioned.rs:674`,
  `planning/validator.rs:14`, `tasks/manager.rs:13`, `tasks/task.rs:292`;
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs:26-72,312-322`.
- Positive path: EKO's production adapter implements RevisionedTaskStore and
  constructs the framework TaskRevisionService; PlanValidator is reused rather
  than reimplemented. This is the target architecture.
- Defect: public ManagedTask/TaskManager and their CRUD/readiness/state/store
  semantics remain beside the revisioned authority. Their pause/parser/tool
  behavior already diverges in F-TSK-01/02.
- Impact: framework consumers can choose incompatible graph identities,
  revisions, transitions and readiness rules, and future EKO work can attach to
  the wrong public path.
- Direction: migrate reasonable rich-record callers to adapters over
  TaskRevisionService/RuntimeDagExecutor, then delete displaced graph mutation,
  state, dependency and store authority. Preserve generic hooks/verifiers and
  framework Store implementation choices.
- Validation reports: [V02](../validations/X-BND-01/V02-01.md).

### X-BND-01-P1-02: Team and Handoff remain separate Agent registries and execution lifecycles outside canonical Subagent authority

- Priority: P1; confidence: high; layer: framework.
- Evidence: committed `src/agent/subagent/team/runner.rs:18`,
  `src/handoff/mod.rs:144`; dependencies F-SUB-01/02 and F-MAG-01.
- Expected boundary: typed transfer/team intent may be public, but definition,
  queue, invocation context, cancellation/deadline, outcome and event identity
  must be the same Subagent lifecycle.
- Observed behavior: Team members and Handoff own direct Agent instances,
  spawning/checkpoint/result/callback paths that bypass canonical invocation and
  can detach work or invent Completed.
- Impact: three framework multi-agent models disagree on catalog readiness,
  cancellation, context, result, checkpoint and topology identity.
- Direction: retain useful Team/Handoff intent APIs as thin adapters; execute
  every member/target through canonical Subagent dispatch and delete raw Agent
  registries, schedulers, result classifiers and name-only topology inference.
- Validation reports: [V03](../validations/X-BND-01/V03-01.md).

### X-BND-01-P1-03: EKO adapters repeatedly own semantic engines instead of lossless product adaptation

- Priority: P1; confidence: high; layer: application adapters.
- Evidence: `echo-agent-cli/src/tauri/commands/panels.rs:677-784`;
  `src/tauri/commands/files.rs`; `src/tauri/commands/chat.rs:114-182,1341-1570`;
  `echo-agent-app-core/src/tool_execution.rs:394-745`;
  dependencies A-SRF-02, A-PROJ-01, X-EVT-01 and X-TOL-01.
- Observed behavior: the Tauri layer owns workflow CRUD/Graph invocation and a
  second diff engine; chat strips the framework envelope and reclassifies
  terminal state; EKO owns a weaker artifact reader and parent-to-Tool terminal
  inference.
- Impact: GUI behavior differs from TUI/CLI/channel, generic invariants are lost
  at the first product boundary, and each fix risks a third authority.
- Direction: move EKO workflow/diff/export/turn/Tool policy into one app-core
  service per capability, adapt the framework contract losslessly, and delete
  Tauri algorithms/raw readers/handwritten terminal inference after cutover.
- Validation reports: [V04](../validations/X-BND-01/V04-01.md).

### X-BND-01-P1-04: EKO's Task execution adapter still owns generic retry, cancellation and settlement semantics

- Priority: P1; confidence: high; layer: framework/application boundary.
- Evidence: committed `echo-orchestration/src/tasks/runtime_executor.rs`;
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs`;
  dependency A-TSK-03.
- Positive path: ready-frontier/DAG traversal uses RuntimeDagExecutor and EKO
  legitimately owns files, write/shell/LLM limits, worktree/review and dispatch.
- Defect: dispatch errors versus returned failures use different retry paths;
  outer drain and multiple helpers infer cancellation/terminal settlement and
  swallow store errors; background TaskRun `depends_on` adds another scheduler.
- Impact: reusable retry/cancel/safe-point invariants change with the product
  adapter, while application-specific policy is interleaved with them.
- Direction: move attempt-scoped retry admission, cancellation join and one
  fallible run settlement primitive into the generic executor; retain only EKO
  dispatch/resources/review/disposition hooks and delete cross-run dependency polling.
- Validation reports: [V05](../validations/X-BND-01/V05-01.md).

### X-BND-01-P2-05: Disconnected advertised authorities obscure the live architecture in both repositories

- Priority: P2; confidence: high; layer: cleanup/public contract.
- Evidence: committed ProviderAdapter (F-LLM-01), old `process_steps`
  (`src/agent/react/run/react_loop.rs:179`), TeamRunner/Coordinator/Mailbox and
  ContextBuilder/SubagentOutput; application StreamingEvent/ServerMessage,
  output-format models, ProjectIndex/FileChangeTracker/backup claims.
- Expected invariant: one advertised type either has a real production owner or
  is a deliberate, documented framework option with a coherent public contract.
- Observed behavior: these types are disconnected, definition/test-only, or
  superseded by another live path; several silently promise behavior that no
  caller can reach.
- Impact: maintainers and AI agents repeatedly design against non-authoritative
  APIs, inflating review, tests, feature surface and migration risk.
- Direction: for framework public APIs, first confirm reasonable external use
  and migrate it to the canonical capability; then delete only displaced
  authority. Application-private/test-only disconnected models should be
  deleted directly with their docs/tests. Do not add deprecation compatibility.
- Validation reports: [V06](../validations/X-BND-01/V06-01.md).

## Non-Deletion Guardrails

- Do not delete framework SQLite stores, compressors, memory strategies,
  Workflow, integration clients, sandbox modes or Tool domains because EKO does
  not select them. They are reasonable public framework capabilities.
- Keep `echo-state` SQLite for external consumers; ensure EKO does not enable it.
- DomainProfile, interaction mode, review/worktree/file ownership, UI fields,
  local delivery and surface composition stay in EKO.
- Public framework removal requires repository-wide call search plus a judgment
  that no reasonable external capability remains; default is retain.

## Validation Matrix

| ID | Claim | Status | Report |
|---|---|---|---|
| V00 | Complete F/A catalog and source-boundary snapshot | passed | [V00-01](../validations/X-BND-01/V00-01.md) |
| V01 | Capability placement/ownership map | passed | [V01-01](../validations/X-BND-01/V01-01.md) |
| V02 | Task graph authority and adapter reachability | failed | [V02-01](../validations/X-BND-01/V02-01.md) |
| V03 | Subagent/Team/Handoff lifecycle authority | failed | [V03-01](../validations/X-BND-01/V03-01.md) |
| V04 | Thick EKO semantic adapter inventory | failed | [V04-01](../validations/X-BND-01/V04-01.md) |
| V05 | Generic versus EKO Task executor responsibility | failed | [V05-01](../validations/X-BND-01/V05-01.md) |
| V06 | Disconnected/deletion-target classification | failed | [V06-01](../validations/X-BND-01/V06-01.md) |
| V07 | Framework reasonable-public-option non-deletion guard | passed | [V07-01](../validations/X-BND-01/V07-01.md) |
| V08 | Dynamic consumer-compatibility/migration probes | not_run | [V08-01](../validations/X-BND-01/V08-01.md) |
| V99 | Exact link/finding/isolation integrity | passed | [V99-01](../validations/X-BND-01/V99-01.md) |

## Handoff

Use the placement map as the implementation gate before every roadmap item.
Migrate one real caller and delete one displaced authority in the same phase;
do not add facades while leaving old loops live. Cross-contract tasks remain the
field-level source of truth for events, Tools, persistence, plugins and memory.
