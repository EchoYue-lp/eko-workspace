# A-FE-02: Task, Subagent, and tool projections

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline 9b0e0fa)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline b3b2e81)
> Worktree state: dirty — 79 modified files, all `web-frontend/src/generated/*.ts`; formatting-only, pre-existing (A-FE-01-P3-02). No other dirty paths in either repository before, during, or after this review (verified by `git status` and generated-dir md5 `779af40c3770faba1e97d3f118754be3` after every executable validation).

## Question

Do frontend projections preserve attempt identity, terminal monotonicity,
lazy output, results, and Task acceptance distinctions?

**Answer: Mostly yes on the data path, with two material projection defects
and one broken surface.** Attempt identity is preserved per-execution-id
(`{run_id}:{task_id}:{plan_revision}:{attempt}`) with terminal monotonicity
and old-attempt isolation in `subagentRunStore`, lazy tool output (expand to
load, 256 KiB live cap, cursor paging) is implemented as documented, the
result view strips the protocol envelope and renders verification/artifacts/
remaining work as distinct sections, and the right rail separates execution
progress from Task acceptance progress at the summary level. However: (P2)
the tool store's LIVE ingest path keys rows by the producer-assigned wire id
(`detail_ref`), not by the available (owner, call_id) identity, and has no
status-rank guard — the two-producer backend projection (A-SRF-02-P2-01)
renders duplicated tool cards and inflated counts (canonical consequence
A-SRF-03-P2-01), and a same-id out-of-order summary can regress a terminal row
to running; (P2) `latestSubagentRunsByTask` selects the "latest attempt" by
parsing only the trailing `:attempt` segment and ignores `plan_revision`, so
after a plan revision the inline chat view can render the superseded
revision's attempt while the right rail picks the current one (empirically
reproduced); (P2) the Task acceptance/check/artifact distinctions are not
rendered or editable in any GUI surface — the only plan editor is dead code
and the "编辑计划后继续" interrupt affordance is a no-op; plus (P3) two dead
components and a reload gap for non-TaskRuntime inline subagent cards.

## Scope

Primary source paths inspected (deep read):

- Stores: `web-frontend/src/stores/toolExecutionStore.ts` (full),
  `subagentRunStore.ts` (full), `taskRuntimeStore.ts` (full),
  `chatStore.ts` (full), `subagentDetailStore.ts` (full),
  `conversationStore.ts` (tool/subagent hydration slices :296-320).
- Rendering: `components/chat/SubagentStreamBlock.tsx`, `InlineToolCall.tsx`,
  `ToolExecutionGroup.tsx`, `ExecutionProcessGroup.tsx`,
  `ParallelExecutionBlock.tsx`, `MessageBubble.tsx` (execution-items
  derivation), `components/task/TaskRuntimePanel.tsx`,
  `SubagentDetailView.tsx`, `components/subagent/SubagentResultView.tsx`,
  `SubagentCard.tsx`, `ResultFullView.tsx`, `PlanEditor.tsx`,
  `utils/subagentResult.ts`, `utils/subagentProgress.ts`.
- Event wiring: `hooks/useTauriChat.ts` (:93-150 listeners),
  `hooks/chatEventHandler.ts`, `api/endpoints.ts` (toolExecutionApi
  :456-464), `types/api.ts` (ToolExecution/Detail types :49-116).
- Backend producers (wire shape + identity): `src/tauri/mod.rs` bridge
  (:353-768), `src/tauri/commands/chat.rs` (`emit_tool_execution_summary`
  :185-208, `emit_execution_event` :153-183, projector :957-1114),
  `echo-agent-app-core/src/tool_execution.rs` (ToolExecutionSummary :64-77,
  `start` fresh detail_ref :191-239, `execution_key` :594-601),
  `echo-agent-app-core/src/tasks/task_runtime/executor.rs`
  (`subagent_execution_id` :174-180, dispatch :1905-1915, :2337-2343),
  `tasks/task_runtime/types.rs` (SubagentRun :1654-1696),
  `echo-agent/echo-orchestration/src/tasks/runtime.rs` (TaskClaim
  `execution_id` :212-224).
- Tests: `stores/subagentRunStore.test.ts`, `toolExecutionStore.test.ts`,
  `taskRuntimeStore.test.ts`, `chatStore.toolExecution.test.ts`,
  `components/chat/*.test.*`, `components/task/SubagentDetailView.test.tsx`,
  `TaskRuntimePanel.test.ts`, `hooks/chatEventHandler.test.ts`.

## Out Of Scope

- Chat-turn lifecycle/status derivation (`chatStore` runStatus,
  error/cancel mislabeling, interrupt ghost turn) — A-SRF-03 (P1-01, P1-02);
  the `InterruptPromptDialog` no-op button is cross-referenced here only.
- Backend double producer of subagent tool summaries — A-SRF-02-P2-01
  (canonical); runtime artifact projection dead — A-TSK-06-P2-01 (canonical).
- DTO/type contract and generated-vs-handwritten drift — A-FE-01.
- Frontend architecture/performance/accessibility — A-FE-03; formatting/build
  gates — Q-WEB-01; dynamic GUI verification — Q-GUI-01/Q-E2E-01.
- TUI/CLI projections of the same facts — A-SRF-01/A-SRF-04.

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety; 防重复造轮子; framework-vs-app
  layering; "动手前先查是不是已经有了"; read-only review).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (A-FE-02 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read: `A-FE-01` (complete — event channel
  contracts, execution://event producers, ToolInfo drift, dormant HTTP
  surface), `A-TSK-06` (complete — full-output persistence/review reuse,
  P2-01 runtime artifact projection dead, P3-01..03).
- Cross-referenced (own track, canonical IDs elsewhere): `A-SRF-02`
  (P1-01 browser://event, P2-01 duplicate tool projection),
  `A-SRF-03` (P1-01 interrupt ghost, P1-02 error/cancel mislabeling,
  P2-01 duplicate tool-card rendering consequence).
- Historical documents treated as hypotheses (classified in V05-01):
  `echo-agent-cli/docs/MASTER-PLAN.md:118-130,190-215`,
  `2026-07-25-gui-tool-execution-lazy-loading.md:15-23,81-106`,
  `subagent-unification-plan.md` §6, `2026-07-16-agent-lifecycle-audit.md`.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correct) | `TaskClaim::execution_id` identity format (echo-orchestration runtime.rs:212-224) — generic execution identity; `ToolExecutionSummary`/`ToolExecutionOwner` (app-core tool_execution.rs:64-77, 46-53) as app-core DTOs; no framework changes proposed. |
| EKO product policy (application, correct) | Frontend stores (`toolExecutionStore`/`subagentRunStore`/`taskRuntimeStore`), lazy-output policy (256 KiB live cap), result-envelope stripping, right-rail execution-vs-acceptance summary, polling lifecycle, `latestSubagentRunsByTask` attempt selection, dead components. |
| Adapter boundary | `useTauriChat.ts` event listeners and `taskRuntimeStore.loadByConversation` replay are thin, lossless adapters into the stores; the replay adapter (`taskRuntimeSubagentExecutionEvents`, subagentRunStore.ts:330-405) converts durable boundaries to lifecycle events without owning a second state machine (monotonic guard lives in the store). |
| Duplicate search (terms, V01-01) | `subagentTraceStore`, `subagentStore`, `executionIdentity`, `toolExecutionOwnerKey`, `subagentRunStoreKey`, `latestSubagentRunsByTask`, `executionAttempt`, `detail_ref`, `call_id`, `subagent_run_id`, `attempt`, `ToolExecution`, `SubagentRun`, `useWebSocket`, `ResultFullView`, `PlanEditor`, `acceptance_criteria`, `execution_checks`, `required_artifacts`, `list_artifacts`, `worker`. Result: one store per projection concept (legacy stores deleted); identity helpers match the backend id format; 2 dead components found; acceptance/check/artifact fields have zero render/edit consumers; zero `worker` terms. |
| Migration deletion | P2-01: make live `ingest` upsert by `executionIdentity` with `mergeToolExecution` (keep `id` for detail loading) — no deletion; the backend single-producer fix (A-SRF-02-P2-01) remains the canonical delete target. P2-02: parse revision+attempt in `executionAttempt`/`latestSubagentRunsByTask` (single selector; `traceRunForTodo` may then delegate). P3-01: delete `PlanEditor.tsx` and `ResultFullView.tsx` (or wire them; if wired, `PlanEditor` must use the generated `TaskSpec` shape). P2-03: wire a plan editor (or delete the dead one) and render acceptance/check/artifact fields on task rows; the interrupt dialog's "编辑计划后继续" button must open the editor or be removed. |

## Current Path

Verified data flow (V01-01/V02-01/V03-01):

1. **Tool projection**: two live producers for subagent tools — bridge
   (mod.rs:353-768, DispatchToolStarted/ToolCompleted) and projector
   (chat.rs:957-1114, ExecEvent ToolStarted/ToolCompleted) — both call
   `tool_executions.start()` which allocates a fresh `detail_ref` and
   overwrites `state.summaries[(owner, call_id)]` (tool_execution.rs:191-248),
   then `emit_tool_execution_summary` -> `emit_execution_event`
   (`kind=tool`, payload = summary + run_id/event, chat.rs:153-208).
   `useTauriChat.ts:132-137` -> `toolExecutionStore.ingest` (keys by
   `tool.id`); `recordToolStart` (dedupe by id) for chat owners. Reload:
   `conversationStore.loadConversation` -> `list_tool_executions` ->
   `hydrateConversation` -> `mergeHydratedToolExecutions` (identity
   (owner+run_id, call_id) with status-rank + timestamp merge);
   `taskRuntimeStore.loadByConversation` -> `taskRuntimeToolExecutions`
   (runtime boundary, empty detail_ref) merged via
   `mergeTaskRuntimeToolExecutions`/`ingestTaskRuntimeToolExecutions`.
2. **Subagent projection**: single live producer (bridge, kind=subagent
   started/completed/failed/cancelled + usage/isolation/artifact) and the
   replay adapter (`taskRuntimeSubagentExecutionEvents` from
   subagent_assigned/subagent_released events, subagentRunStore.ts:330-405).
   Store key `{runId}\0{subagent_run_id}`; monotonic guard drops events after
   terminal (:458-460); retries are separate keys; `latestSubagentRunsByTask`
   (:417-441) selects per task, `traceRunForTodo` (TaskRuntimePanel.tsx:
   429-433) by startedAt. `visibleSubagentRuns` (ParallelExecutionBlock.tsx:
   31-63) filters by messageId/activeRun and applies
   `latestSubagentRunsByTask`.
3. **Task projection**: `taskRuntimeStore` polls (2 s, generation-guarded,
   :133-217), appends events past `lastSeq`, fetches plan/todos/artifacts/
   blockers (`loadRunSnapshot` :50-58); `displayedTodoStatus` keeps persisted
   terminal statuses authoritative and projects only pending/running traces
   (:444-486). Run-level artifacts always `[]` (A-TSK-06-P2-01).
4. **Result/acceptance**: `SubagentResultView` renders summary +
   verification (check/status/source) + artifacts + remaining_work as
   distinct sections; `subagentResultPresentation` strips the `## Result`
   envelope (subagentResult.ts:7-34); `todoStatusDescription` renders
   acceptance failure as "评审未通过" (TaskRuntimePanel.tsx:496-498).

## Findings

### A-FE-02-P2-01: Live tool-event ingest keys rows by producer-assigned `detail_ref` id instead of the (owner, call_id) identity and has no terminal-status guard — duplicate subagent tool projection renders duplicated cards and inflated counts, and a same-id out-of-order summary can regress a terminal row to running

- Priority: P2
- Confidence: high
- Layer: application (frontend store); root cause spans adapter/backend
  (A-SRF-02-P2-01 double producer)
- Evidence:
  - Live path: `web-frontend/src/stores/toolExecutionStore.ts:206-217` —
    `ingest` stores `tools[tool.id]` (wire id = `detail_ref`) and appends
    `tool.id` to `idsByOwner[ownerKey]`; no status-rank or identity merge.
  - Identity helpers exist but are hydration-only: `executionIdentity`
    (toolExecutionStore.ts:46-48 = (owner+run_id, call_id)) is used by
    `mergeHydratedToolExecutions` (:106-117), `mergeTaskRuntimeToolExecutions`
    (:188-200), `ingestTaskRuntimeToolExecutions` (:241-254) — not by
    `ingest`.
  - Two producers emit two summaries with different `detail_ref` ids for the
    same logical call (bridge mod.rs:353-768 + projector chat.rs:957-1114;
    `start()` allocates a fresh UUID per call, tool_execution.rs:191-200;
    A-SRF-02-P2-01 canonical).
  - Rendering consequence: `SubagentStreamBlock.tsx:46-49` renders every id
    in `idsByOwner` -> duplicated `InlineToolCall` rows and "N 工具" inflated
    (canonical consequence A-SRF-03-P2-01).
  - Monotonicity gap: `ingest` blindly overwrites — replayed fixture with
    same id 'succeeded' then 'running' regresses the row to running
    (V03-01 scenario 4); the merge path has the guard (`toolStatusRank`/
    `mergeToolExecution` :50-86) but the live path does not.
  - Test gap: `chatStore.toolExecution.test.ts` "ignores duplicate start
    events for one execution ID" covers only same-id duplicates.
- Reachability: any GUI chat turn where a TaskRuntime/foreground run's
  subagent calls a tool (both producers live, V02-01); chat-owner tools have
  a single producer and are unaffected.
- Expected invariant: one projection row per logical tool execution; reducer
  keys are (owner, call_id) — never producer-assigned ids (AGENTS.md "严禁
  平行实现同一语义"; MASTER-PLAN latest-attempt/monotonic claims; the task
  card's "不得用 (owner, call_id) 之外的身份键").
- Observed behavior: two rows per logical call (two ids, two detail files);
  subagent cards show doubled tool counts; a same-id duplicate/out-of-order
  summary can regress a terminal row to running.
- Impact: misleading tool accounting on the flagship chat surface (duplicated
  cards, inflated counts, wrong status) and redundant detail-file churn on
  disk; the backend double persist (A-SRF-02-P2-01) becomes user-visible
  instead of being absorbed.
- Root cause: the live ingest was written to the wire shape (`id` =
  `detail_ref`) while the natural logical key — (owner, call_id) — was already
  available and already used by every merge path; the two producers were
  wired without idempotency, and no status-rank guard protects the live path.
- Direction: make `ingest` upsert by `executionIdentity` using
  `mergeToolExecution` (prefer the row with a `detail_ref` and the newer
  activity timestamp; keep `id` for detail loading) — one helper reused by
  both live and hydration paths; add a status-rank guard so terminal rows
  never regress to running. The canonical backend fix remains the
  A-SRF-02-P2-01 single-producer delete target.
- Regression validation: fixture ingesting two started/finished summary pairs
  with identical (owner, call_id) but different ids -> exactly one row in
  `tools` and one id in `idsByOwner`; fixture re-emitting a 'running' summary
  for a terminal row -> status stays terminal; `SubagentStreamBlock` shows
  count 1.
- Validation reports: [V01-01](../validations/A-FE-02/V01-01.md),
  [V02-01](../validations/A-FE-02/V02-01.md),
  [V03-01](../validations/A-FE-02/V03-01.md),
  [V04-01](../validations/A-FE-02/V04-01.md)

### A-FE-02-P2-02: `latestSubagentRunsByTask` ignores `plan_revision` — after a plan revision the inline chat view selects the superseded revision's attempt, diverging from the startedAt-based right-rail selector

- Priority: P2
- Confidence: high (empirically reproduced)
- Layer: application (frontend selector)
- Evidence:
  - Execution identity: `{run_id}:{task_id}:{plan_revision}:{attempt}`
    (echo-orchestration runtime.rs:221-223; app-core executor.rs:174-180;
    types.rs:1657-1662).
  - Selector: `latestSubagentRunsByTask` groups by `${runId}\u0000${taskId}`
    (subagentRunStore.ts:426) and orders by `executionAttempt`
    (subagentRunStore.ts:407-414), which parses ONLY the trailing segment
    after the last `:`; `plan_revision` is discarded, so revision 4 attempt 3
    (`...:4:3`) outranks revision 5 attempt 1 (`...:5:1`).
  - Divergence: `traceRunForTodo` (TaskRuntimePanel.tsx:429-433) orders by
    `startedAt` — the right rail picks the current revision's attempt while
    `ParallelExecutionBlock`/`visibleSubagentRuns`
    (ParallelExecutionBlock.tsx:31-63) renders the superseded one inline.
  - Empirical reproduction (V03-02): store contains `run-1:task-1:4:3`
    (completed) and `run-1:task-1:5:1` (running, later started_at);
    `latestSubagentRunsByTask` returns `run-1:task-1:4:3`.
- Reachability: any run whose plan is revised mid-run (plan edit re-dispatches
  an already-executed task — the GUI's updateTasks path and the
  "编辑计划后继续" interrupt flow) while both attempts are present in the
  store; both records share the same `runId`+`taskId` and `messageId`, so
  `visibleSubagentRuns` collapses them to the wrong one.
- Expected invariant: "defaults to the latest attempt when rendering a task"
  (app MASTER-PLAN:196-203) — latest means the current plan revision's
  execution, not the highest attempt ordinal across revisions; one selector
  authority.
- Observed behavior: the old revision's terminal attempt is rendered as the
  task's current subagent in the inline chat stream while the right rail
  shows the new attempt; two "latest attempt" selectors exist with different
  semantics.
- Impact: misleading subagent display during/after plan revision — the user
  sees a stale (superseded) subagent card as current; divergent projections
  of the same task between surfaces (parity violation).
- Root cause: `executionAttempt` was written for the earlier 3-segment id
  (`{run_id}:{task_id}:{attempt}`) and never updated when the id gained
  `plan_revision`; the startedAt-based selector was added later for the
  right rail without unifying the two.
- Direction: parse `{run_id}:{task_id}:{plan_revision}:{attempt}` fully
  (revision first, then attempt) in a single shared helper; make
  `latestSubagentRunsByTask` and `traceRunForTodo` delegate to it (one
  authority); keep superseded records in the store for history but never
  render them as current.
- Regression validation: fixture "revision 4 attempt 3 completed + revision 5
  attempt 1 running -> selector returns revision 5 attempt 1"; assert
  `visibleSubagentRuns` renders only the current-revision attempt; a
  `SubagentStreamBlock` render fixture with both attempts.
- Validation reports: [V03-02](../validations/A-FE-02/V03-02.md),
  [V01-01](../validations/A-FE-02/V01-01.md), [V05-01](../validations/A-FE-02/V05-01.md)

### A-FE-02-P2-03: Task acceptance/check/artifact distinctions are neither rendered nor editable in the GUI — the only plan editor is dead code and the interrupt dialog's "编辑计划后继续" affordance is a no-op

- Priority: P2
- Confidence: high (static; zero consumers)
- Layer: application
- Evidence:
  - `acceptance_criteria`/`execution_checks`/`required_artifacts` exist as
    distinct TaskSpec/TaskPatch fields (generated `TaskSpec.ts:11`,
    `PlanTask.ts:21-32`; `completeTaskPatch` taskRuntimeStore.ts:60-74
    passes them through) but have ZERO rendering or editing consumers in
    components (V01-01; only the insertTask defaults at
    TaskRuntimePanel.tsx:818-820 set them, always empty arrays).
  - The only plan-edit UI, `PlanEditor.tsx` (full file), has zero imports —
    dead; its internal `PlanTask` shape is only {id, title, description,
    status}, which would corrupt the TaskSpec round-trip if wired (P3-01).
  - The interrupt flow's "编辑计划后继续" button only calls `dismiss()`
    (TaskRuntimePanel.tsx:893-902) — no editor is opened; the user cannot
    actually edit the plan from the dialog.
  - Run-level artifact list: `loadRunSnapshot` fetches `listArtifacts`
    (taskRuntimeStore.ts:50-58) from a backend projection with zero
    production writers (A-TSK-06-P2-01) — always `[]`; `ResultFullView`
    (the modal that would show full results) is dead (P3-01).
  - Task-level acceptance evidence reaches the UI only as summary text
    (`todoStatusDescription` "评审未通过", TaskRuntimePanel.tsx:496-498)
    and per-subagent `verification` sections (SubagentResultView.tsx:21-39).
- Reachability: right rail on every TaskRuntime run (blocked/failed todo rows
  show only a reason string); the interrupt dialog on every mid-run message
  (live); `insertTask` in the panel on every new-task click.
- Expected invariant: the plan is an editable, reviewable artifact and the
  GUI can distinguish execution checks, required artifacts, and acceptance
  criteria (AGENTS.md "TaskPlan 只能是可编辑/可审阅的版本化 artifact";
  A-TSK-06 "acceptance/check separation"; app MASTER-PLAN "right rail
  separates execution from acceptance").
- Observed behavior: no surface renders or edits the three distinction
  fields; the GUI's new-task flow always creates them empty; the plan editor
  and the full-result modal are dead; the interrupt dialog advertises plan
  editing that does nothing.
- Impact: users cannot author or inspect acceptance criteria/execution
  checks/required artifacts in the GUI (multi-surface parity gap — TUI/CLI
  can), cannot meaningfully "edit and continue" an interrupted run, and the
  run-level artifact panel is permanently empty (A-TSK-06-P2-01); a
  misleading button on the interrupt path.
- Root cause: the plan-editing UI was written ahead of its wiring (never
  mounted), the interrupt redesign removed the edit entry point, and the
  acceptance/check/artifact projection was never added to the task rows; the
  artifact list consumed a projection that was never given a writer.
- Direction: wire (or delete) `PlanEditor` against the generated
  `TaskSpec`/`TaskPatch` shape and the `updateTasks` API; open it from the
  interrupt dialog's "编辑计划后继续" (or remove the button); render
  `required_artifacts`/`execution_checks`/`acceptance_criteria` on todo rows
  (e.g., blockers distinguish "artifact missing" vs "check failed" vs
  "acceptance failed" — backend already separates these, A-TSK-06); resolve
  the artifact list per A-TSK-06-P2-01 direction (wire `add_artifact` or
  delete the chain).
- Regression validation: fixture "plan with acceptance_criteria opened in the
  editor -> fields round-trip through updateTasks unchanged"; interrupt
  dialog "编辑计划后继续" opens the editor (or button removed); a blocked
  todo distinguishes acceptance vs check vs artifact reason; vitest render
  fixtures for each.
- Validation reports: [V01-01](../validations/A-FE-02/V01-01.md),
  [V02-01](../validations/A-FE-02/V02-01.md), [V05-01](../validations/A-FE-02/V05-01.md)

### A-FE-02-P3-01: `ResultFullView` and `PlanEditor` are dead components — zero imports anywhere in `src/`, and `PlanEditor`'s reduced task shape would corrupt a TaskSpec round-trip if wired

- Priority: P3
- Confidence: high (grep proof, no dynamic imports)
- Layer: application
- Evidence: `src/components/task/ResultFullView.tsx` (full file — full-result
  modal) and `src/components/task/PlanEditor.tsx` (full file — plan form/JSON
  editor) have zero references outside their own files (V01-01; grep across
  all `src/**/*.{ts,tsx}` including dynamic imports). `PlanEditor`'s
  `PlanTask` interface is `{id, title, description, status}` — a reduced
  shape that drops domain_profile/depends_on/parallel_group/files/
  allowed_tools/required_artifacts/execution_checks/acceptance_criteria/
  max_retries/sort_order, so wiring it today would corrupt the revisioned
  plan on save.
- Reachability: none (dead code). The "编辑计划后继续" interrupt button
  (TaskRuntimePanel.tsx:896-902) does not reference it either.
- Expected invariant: no dead UI code under live-sounding names (AGENTS.md
  "删死代码"; REPORTING.md priority definitions).
- Observed behavior: two authored surfaces exist that nothing mounts.
- Impact: maintenance burden; a future developer wiring "edit plan" or
  "full result" would either find them (and hit the reduced-shape corruption)
  or re-author a parallel editor (duplicate authority risk).
- Root cause: written ahead of wiring; the interrupt redesign and the
  result-view consolidation left them behind.
- Direction: delete both files — or, per P2-03, wire `PlanEditor` against the
  generated `TaskSpec`/`TaskPatch` types and mount it from the interrupt
  dialog; wire `ResultFullView` as the "full result" modal or delete it.
- Regression validation: grep zero imports after deletion; if wired, a
  round-trip fixture (edit -> updateTasks -> reloaded plan equals edited
  TaskSpec fields).
- Validation reports: [V01-01](../validations/A-FE-02/V01-01.md)

### A-FE-02-P3-02: Non-TaskRuntime inline subagent runs are realtime-only — their stream cards vanish after a conversation reload (tool rows are restored, subagent lifecycle is not)

- Priority: P3
- Confidence: high (static), medium (impact magnitude)
- Layer: application (adapter/persistence boundary)
- Evidence: `conversationStore.loadConversation` (conversationStore.ts:296-
  320) restores chat messages and tool summaries
  (`hydrateConversation`), but nothing populates `subagentRunStore` for a
  historical conversation without a TaskRuntime run; the only restore path is
  `taskRuntimeStore.loadByConversation` (taskRuntimeStore.ts:219-283,
  triggered by App.tsx:57-60 on conversation switch and by `run_started`,
  useTauriChat.ts:138-148), which replays durable `subagent_assigned`/
  `subagent_released` events — i.e., TaskRuntime-backed runs only. Inline
  subagents dispatched directly by the primary agent (no formal plan) have
  their lifecycle emitted by the bridge (mod.rs:353-768) with no durable
  source.
- Reachability: reloading/opening a historical conversation containing
  inline (non-TaskRuntime) subagent activity: the message text remains, tool
  rows return, but the SubagentStreamBlock cards (lifecycle, result tabs)
  do not.
- Expected invariant: surface parity and restart continuity — facts visible
  during a session remain visible after reload where a durable source exists
  (X-STA-01 scope; "界面状态不是唯一事实源" per the lazy-loading doc).
- Observed behavior: subagent lifecycle for non-TaskRuntime runs is lost
  from the UI after reload; only TaskRuntime-backed runs restore cards.
- Impact: users lose the subagent execution trace (process/result tabs) for
  inline subagent activity in older conversations; inconsistent with tool
  rows which do restore.
- Root cause: subagent lifecycle events were wired realtime-only; the
  durable TaskRuntime event stream covers only formal-plan dispatches.
- Direction: persist inline subagent lifecycle events in the conversation
  record (or a small sidecar) and replay them through
  `subagentRunStore.ingest` on load — or explicitly accept and document the
  realtime-only scope; the replay adapter pattern already exists
  (`taskRuntimeSubagentExecutionEvents`).
- Regression validation: fixture — conversation with an inline subagent
  completed event saved and reloaded -> the subagent run record is restored
  with terminal result; or documented deletion of the expectation.
- Validation reports: [V02-01](../validations/A-FE-02/V02-01.md)

## Cross-Verified Dependency Findings (canonical IDs elsewhere; independently confirmed here)

| Canonical ID | Claim | Independent confirmation |
|---|---|---|
| A-SRF-02-P2-01 | Two live producers persist the same subagent tool events (bridge mod.rs + projector chat.rs) with fresh `detail_ref` per start | Confirmed (V02-01): both producers call `tool_executions.start()` for the same (owner, call_id); tool_execution.rs:191-248. |
| A-SRF-03-P2-01 | Live tool ingest keyed by per-producer `detail_ref`, not (owner, call_id); duplicated cards/counts; hydration dedupe does not cover live path | Confirmed and strengthened (V01-01/V03-01): `ingest` at toolExecutionStore.ts:206-217; fixture scenario 3 shows two rows; additionally the live path has no terminal-status guard (scenario 4) — see A-FE-02-P2-01. |
| A-TSK-06-P2-01 | Runtime `Artifact`/`ArtifactProduced` projection dead; GUI run-level artifact list always empty | Confirmed (V02-01): `loadRunSnapshot` fetch chain taskRuntimeStore.ts:50-58 -> `list_artifacts` -> file_store event scan with zero production writers. |
| A-SRF-03-P1-01 | Interrupt prompt strands the frontend turn state | Not re-validated (chat-turn scope, A-SRF-03); the InterruptPromptDialog "编辑计划后继续" no-op is the A-FE-02-relevant part (P2-03). |
| A-FE-01-P3-02 | 79 generated TS files dirty (formatting-only) | Baseline confirmed identical before/after every executable validation in this task (md5 `779af40c...`). |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---:|---|
| V01 | Definition + duplicate search (identity helpers; store inventory; dead components; backend id format) | yes | passed (P2-01/P2-02/P2-03/P3-01 evidence) | [V01-01](../validations/A-FE-02/V01-01.md) |
| V02 | Registration + runtime reachability (producers, listeners, replay path, lazy detail chain, artifact fetch) | yes | passed (P2-01/P3-02/P2-03 evidence) | [V02-01](../validations/A-FE-02/V02-01.md) |
| V03 | Invariant/edge fixtures: identity keys, duplicate/out-of-order, old attempt, lazy output, results/acceptance (12 replay fixtures) | yes | passed (P2-01 evidence; transient file deleted, baseline verified) | [V03-01](../validations/A-FE-02/V03-01.md) |
| V03 | Revision-aware latest-attempt selector fixture (empirical) | yes | passed (P2-02 reproduced; transient file deleted, baseline verified) | [V03-02](../validations/A-FE-02/V03-02.md) |
| V04 | `npx vitest run` (15 files, 68 tests — stores + chat/task components + hooks) | yes | passed (exit 0) | [V04-01](../validations/A-FE-02/V04-01.md) |
| V05 | Historical-document drift (MASTER-PLAN latest-attempt/monotonic/right-rail claims; lazy-loading doc) | yes | passed (2 regressed claims -> P2-01/P2-02) | [V05-01](../validations/A-FE-02/V05-01.md) |

All required validations executed; every command has a known exit code. No
command that regenerates `web-frontend/src/generated/*.ts` was executed; the
pre-existing 79-file dirty state and the generated-dir md5 were verified
identical before and after every run.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| app MASTER-PLAN:196-203 "frontend stores all attempts independently, keeps terminal state monotonic, defaults to the latest attempt when rendering a task" | regressed (latest-attempt part; monotonic part regressed only on the tool live path) | P2-02 (`latestSubagentRunsByTask` ignores plan_revision, V03-02); P2-01 (live `ingest` no status-rank guard, V03-01) |
| app MASTER-PLAN:205-215 "result view uses complete terminal output without its internal protocol envelope; never promotes a thinking trace" | current | subagentResult.ts:7-34 + SubagentResultView (V03-01 scenario 12) |
| app MASTER-PLAN:210-215 "right rail reports Subagent execution progress separately from Task acceptance progress" | current (summary level) / gap (structured fields) | TaskRuntimePanel.tsx:560-567, 496-498; P2-03 for the structured-level gap (V05-01) |
| 2026-07-25 lazy-loading doc:97-106 "toolExecutionStore 按 detail_ref 归一化;500 ms cursor;256 Ki pause" | current (implementation matches doc; the doc's single-producer assumption is broken by A-SRF-02-P2-01) | toolExecutionStore.ts:206-217; InlineToolCall.tsx:23, 91-105, 244-258 (V05-01) |
| subagent-unification-plan §6 "single source of truth, aggregate by execution id, task_id stable join key" | current | subagentRunStore.ts:1-14, 417-441 (V05-01) |

## Coverage And Uncertainty

- All conclusions are static traces plus the V03 transient fixture replays and
  the V04 suite run; no GUI process was launched (Q-GUI-01/Q-E2E-01 own
  dynamic confirmation, e.g., the duplicated-card rendering on screen and the
  interrupt dialog behavior).
- P2-02's reachability depends on backend plan-revision semantics (plan edit
  re-dispatches an executed task with a new revision claim) — the execution-id
  format and claim behavior are confirmed (V01-01), but a live end-to-end
  revision scenario was not run (A-TSK-01/A-TSK-04 scope).
- P2-01's same-id regression scenario has no confirmed live backend trigger
  today (each producer allocates a fresh detail_ref); it is a defensive
  invariant gap of the live ingest that the identity merge would fix as a
  side effect.
- P3-02's impact was assessed statically; the exact reload flow was verified
  by code trace (conversationStore restores tools only), not on screen.
- The legacy `TasksPanel` (SSE-based background tasks) and `WorkflowPanel`
  were noted but not reviewed (different domains; A-FE-03/A-OBS-01 scope).
- Large-output collapse/expand was verified at the data and component level
  (existing ExecutionProcessGroup/SubagentStreamBlock tests + static
  inspection of InlineToolCall); no DOM-level large-output fixture exists in
  the suite (Q-E2E-01 candidate).
- The `events` log in subagentRunStore can receive duplicate 'started'
  entries when both the live bridge and the replay adapter hit a running run
  (bounded at 300, not rendered) — tolerated by design, no finding filed.

## Handoff

- Downstream tasks may rely on: attempt identity per execution id is
  preserved and old-attempt completion cannot reopen a terminal/newer attempt
  (V03-01); subagent terminal monotonicity holds in the store (V03-01);
  hydration/merge paths dedupe by (owner+run_id, call_id) while the live path
  does not (P2-01); `latestSubagentRunsByTask` is revision-blind (P2-02);
  lazy output, envelope stripping, and the result/acceptance section
  separation work as documented (V03-01/V05-01); run-level artifacts are
  empty end-to-end (A-TSK-06-P2-01); `ResultFullView`/`PlanEditor` are dead
  (P3-01).
- Findings for the roadmap: P2-01 (live ingest identity/monotonicity —
  direction overlaps A-SRF-03-P2-01/A-SRF-02-P2-01), P2-02 (revision-aware
  latest-attempt selector), P2-03 (acceptance/check/artifact GUI surface +
  interrupt dialog no-op), P3-01 (delete or wire the two dead components),
  P3-02 (inline subagent reload restoration).
- Reports to read: this report + V01-01..V05-01; dependency reports A-FE-01
  (event payload contracts, generated drift) and A-TSK-06 (full-output/
  review facts, P2-01); cross-referenced A-SRF-02 (P2-01) and A-SRF-03
  (P1-01/P1-02/P2-01).
- Stale conditions: this report becomes stale if `toolExecutionStore.ingest`/
  `mergeToolExecution`, `subagentRunStore` ingest/`latestSubagentRunsByTask`/
  `executionAttempt`, `taskRuntimeStore` polling/snapshot loading, the bridge
  or projector producers, `SubagentStreamBlock`/`InlineToolCall`/
  `TaskRuntimePanel`, `PlanEditor`/`ResultFullView`, or the conversation
  load/hydration path change; also if a production caller of `add_artifact`
  appears (A-TSK-06-P2-01 fixed).
- Follow-up task IDs (fixes are not implemented in this review): A-SRF-03
  (P1-01/P1-02 chat-turn defects), X-EVT-01 (event lifecycle conformance:
  duplicate/out-of-order terminal across all consumers), X-TSK-01/X-STA-01
  (attempt identity and artifact identity across restart), Q-E2E-01 (GUI
  smoke: duplicate tool cards, interrupt dialog, plan edit, artifact list),
  Q-WEB-01 (frontend gate), S-RDM-01 (roadmap items for P2-01..P2-03,
  P3-01, P3-02).
