# A-FE-02: Task, Subagent, and tool projections

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: not-applicable (frontend-only review; backend contract read at 9b0e0fa via A-TSK-06)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Do frontend projections preserve attempt identity, terminal
monotonicity, lazy output, results, and Task acceptance distinctions?

## Scope

Primary source paths and behaviors inspected:

- **Identity / reducer stores (read in full)**:
  - `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts`
    (536 lines) — the per-attempt execution-id aggregation key
    (`subagentRunStoreKey`, line 157), the terminal lock (lines 458-460),
    the `latestSubagentRunsByTask` attempt selector (lines 417-441), the
    `taskRuntimeSubagentExecutionEvents` durable-events adapter
    (lines 330-405), the `terminalResult` projection (lines 205-226), the
    `STORED_SUBAGENT_EVENTS` set + `MAX_EVENTS_PER_RUN=300` cap.
  - `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.ts`
    (255 lines) — the DUAL identity scheme (`tool.id` for live ingest
    line 211 vs `executionIdentity` = `ownerKey\u0000call_id` for
    merge/hydrate line 47), the live `ingest` direct-overwrite
    (lines 206-217), the `mergeToolExecution` status-rank policy
    (lines 58-86), the `mergeTaskRuntimeBoundary` durable-fact policy
    (lines 88-104), `taskRuntimeToolExecutions` projection
    (lines 135-185), `ingestTaskRuntimeToolExecutions` incremental
    merge (lines 241-254).
  - `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts`
    (394 lines) — the single-active-run model, the `loadGeneration` +
    `refreshRequestGeneration` counters (lines 18-19, 166-217, 219-283),
    the `refreshInFlight` polling-overlap guard (lines 17, 141, 170,
    215), the `lastSeq` string cursor for incremental polling
    (lines 87, 179, 234, 245), the `MAX_EVENTS=500` cap (line 14).
- **Rendering components (read in full)**:
  - `echo-agent-cli/web-frontend/src/components/chat/InlineToolCall.tsx`
    (269 lines) — the pull-based, expanded-only tool detail with cursor
    pagination (`loadPage`, lines 61-84), the `LIVE_DETAIL_AUTOLOAD_CHARS
    = 256 * 1024` live cap (line 23), the 500 ms auto-refresh while
    running (lines 91-105), the terminal-status one-shot refetch
    (lines 107-117), the `manifest.truncated` agent-context-truncation
    flag (line 206), the `max-h-96 overflow-auto` output container
    (line 229), the manual "load more" button (lines 249-258).
  - `echo-agent-cli/web-frontend/src/components/chat/SubagentStreamBlock.tsx`
    (162 lines) — the three-tab layout (task / process / result), the
    auto-collapse on terminal (lines 52-58), the result-tab
    `SubagentResultView` rendering (line 150).
  - `echo-agent-cli/web-frontend/src/components/chat/ParallelExecutionBlock.tsx`
    (109 lines) — the `visibleSubagentRuns` message-association rules
    (lines 31-63), the `latestSubagentRunsByTask` selection + sort
    (line 62).
  - `echo-agent-cli/web-frontend/src/components/subagent/SubagentResultView.tsx`
    (70 lines) — the `result.verification` rendering with
    `{item.status} · {item.source}` flat label (lines 27-37), the
    artifacts section, the remaining-work section.
  - `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx`
    (head + 425-845 slices) — `traceRunForTodo` / `displayedTodoStatus`
    / `todoStatusDescription` (lines 429-515), the todo list render
    (lines 700-791), the new-task insertion with empty
    `execution_checks` / `acceptance_criteria` (lines 819-820), the
    recovery-blocker render (lines 630-674).
  - `echo-agent-cli/web-frontend/src/components/chat/MessageBubble.tsx`
    (head, 1-120) — `isExecutionProcessCompleted`, `flattenSteps`.
  - `echo-agent-cli/web-frontend/src/utils/subagentResult.ts` (full,
    35 lines) — `stripTerminalContract` envelope removal.
- **Type / API contract (read in full)**:
  - `echo-agent-cli/web-frontend/src/generated/SubagentVerificationResult.ts`,
    `SubagentVerificationSource.ts` (`'observed' | 'reported'`),
    `SubagentVerificationStatus.ts` (`'passed' | 'failed' | 'not_run'`),
    `SubagentTaskResult.ts`, `RecoveryBlocker.ts`.
  - `echo-agent-cli/web-frontend/src/api/endpoints.ts:493-613` —
    `taskRuntimeApi` (including the NEVER-CALLED `listReviews` at
    lines 559-562), `toolExecutionApi`.
- **Backend cross-checks (read-only, for V04 semantics)**:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:1385-1394`
    (`RecoveryBlocker` schema — workspace/tool state, NOT review
    issues), `:1511-1563` (`SubagentVerificationSource` Observed /
    Reported semantics), `:1003-1010` (execution_checks vs
    acceptance_criteria field docs, via A-TSK-06).
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:2080-2130`
    (`list_recovery_blockers` — folds `RecoveryBlocked` events only;
    does NOT include review issues).
  - `echo-agent-cli/src/tauri/commands/task_runtime.rs:125`
    (`list_task_reviews` command — registered and implemented, never
    invoked from the frontend).
- **Tests inspected**:
  - `src/stores/subagentRunStore.test.ts` (5 tests — terminal
    preservation, duplicate-started-after-terminal, retry isolation,
    cross-run isolation, durable restoration).
  - `src/stores/toolExecutionStore.test.ts` (9 tests — runtime
    reconstruction, persisted-over-fallback, durable-fact-over-stale,
    incremental terminal, merge paths, cross-run, hydrate isolation).
  - `src/stores/taskRuntimeStore.test.ts` (7 tests — recovery controls,
    polling start/stop, hydration, stale-load ignore, stale-refresh
    ignore).
  - `src/components/chat/InlineToolCall.test.tsx`,
    `SubagentStreamBlock.test.tsx`, `MessageBubble.completed.test.tsx`.
  - Full suite executed: `npx vitest run` → 26 files, 101 tests, exit 0.

## Out Of Scope

Deferred to downstream tasks:

- **A-SRF-03** owns the chat-transport receive-side contract and the
  tool-execution live-ingest overwrite (A-SRF-03-P2-01). This task
  confirms that finding from the projection-identity angle (V01/V02)
  and cross-references it; the fix is owned there.
- **A-FE-01** owns the static IPC type contract. This task consumes
  the contract as authoritative and audits how stores/components
  project state through it. The `ToolInfo` wire drift (A-FE-01-P2-01)
  is upstream context, not re-audited here.
- **A-TSK-06** owns the backend review/result/artifact preservation.
  This task audits only the frontend projection of those facts; the
  backend two-gate separation is consumed as verified.
- **A-TSK-04** owns the TaskRuntime state-machine semantics and the
  durable-event fold. This task relies on those invariants for the
  recovery-path analysis.
- **A-FE-03** owns the broader frontend architecture / performance /
  accessibility audit (component organization, listener cleanup,
  monolithic state owners). This task touches components only where
  they affect projection identity, monotonicity, lazy output, and the
  acceptance/check distinction.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (multi-mode parity rule; framework-vs-
  application layering gate; "first prove no duplicate exists"
  implementation gate; no-panic / UTF-8 safety; the cleanup rule).
- `docs/comprehensive-review/REPORTING.md`,
  `docs/comprehensive-review/templates/{task-report,validation-report}.md`,
  `docs/comprehensive-review/TASKS.md` (A-FE-02 card + dependency
  list).

Dependency reports read:

- **A-FE-01** (complete) — established the IPC type-contract matrix.
  Load-bearing for V01/V04: the task-runtime family is the only IPC
  surface with a real compile-checked contract (`endpoints.ts:49-59`
  imports from `generated/`); `SubagentTaskResult.verification` and
  `RecoveryBlocker` are part of that contract. The `ToolInfo` wire
  drift (A-FE-01-P2-01) is noted as upstream context for the
  tool-panel rendering, but the tool-execution store contract is
  correct.
- **A-TSK-06** (complete) — established the backend two-gate
  completion assessment (hard-evidence `execution_checks` first,
  reviewer-judged `acceptance_criteria` second, strictly ordered,
  disjoint fields) and the `ReviewResult` / `TaskExecutionSummary` /
  `Artifact` durable schemas. Load-bearing for V04: the backend
  preserves the acceptance/check distinction losslessly; this task
  audits whether the frontend projection preserves it.
- **A-SRF-03** (complete) — established the receive-side reducer
  policy matrix and the tool-execution live-ingest overwrite
  (A-SRF-03-P2-01). Load-bearing for V01/V02: the subagent reducer is
  monotone by construction; the tool-execution reducer is monotone
  only on the hydrate path. This task extends the identity-key and
  old-attempt analysis.

Historical documents treated as hypotheses:

- `subagentRunStore.ts:1-14` module doc — claims the aggregation key
  is the per-attempt execution id and that `task_id` is the stable
  PlanTask join key. Verified current by V01.
- `InlineToolCall.tsx:23` (`LIVE_DETAIL_AUTOLOAD_CHARS`) — the 256 KiB
  cap is treated as the intended lazy-load bound. Verified current by
  V03.
- `TaskRuntimePanel.tsx:450-465` comment — "Persisted authoritative
  statuses must NOT be overwritten by trace signals." Verified current
  by V04.

## Layering Decision

This is an **application-layer** task. All inspected code lives in
`echo-agent-cli/web-frontend/src/` with backend cross-checks in
`echo-agent-cli/{echo-agent-app-core/src, src/tauri}`. The framework
supplies no frontend artifacts; the only framework contracts consumed
are the wire shapes of `SubagentTaskResult`, `SubagentVerification*`,
`RecoveryBlocker`, and `RuntimeTaskEvent` (all projected through the
app-core adapter audited in A-TSK-06).

| Classification | Required answer |
|---|---|
| Generic mechanism | Zustand `create` store + Tauri `listen`/`invoke` IPC + React `memo`/`useEffect` for pull-based detail. All used as transport/state primitives. |
| EKO product policy | The per-attempt identity model, the terminal-lock monotonicity, the generation-counter stale-drop, the cursor-paginated lazy output, the `todoStatusDescription` acceptance-label projection, and the (missing) review-evidence surface are all EKO product policy, correctly in the frontend. |
| Adapter boundary | `taskRuntimeSubagentExecutionEvents` (subagentRunStore.ts:330-405) and `taskRuntimeToolExecutions` (toolExecutionStore.ts:135-185) are thin adapters from durable `RuntimeTaskEvent[]` to the realtime lifecycle stream. `terminalResult` (subagentRunStore.ts:205-226) is a thin projection from `ExecutionEvent` to `SubagentTaskResult`. None own authority over result/review semantics. |
| Duplicate search | Searched the frontend tree for: `listReviews`, `list_task_reviews` (1 hit — endpoints.ts definition only; ZERO callers), `ReviewResult` consumption (only in generated/ + endpoints.ts type; ZERO component readers), `execution_checks`/`acceptance_criteria` reads in components (ZERO hits — only written as empty arrays at TaskRuntimePanel.tsx:819-820), `latestSubagentRunsByTask` (1 definition, 1 caller in ParallelExecutionBlock.tsx:62), `subagentRunStoreKey` (1 definition, callers in ingest + tests). No parallel subagent/tool store remains (A-SRF-03 confirmed the legacy stores were deleted). |
| Migration deletion | No deletion proposed. The findings identify a missing review-evidence surface and under-exposed check/criteria fields; resolution is left to follow-up task IDs. |

## Current Path

Verified frontend projection flow at `echo-agent-cli` commit `b3b2e81`.

### Identity model (V01)

Three stores, three identity strategies, all anchored to backend
facts:

```text
subagentRunStore
  key = `${runId}\u0000${subagentRunId}`                 [subagentRunStore.ts:157-159]
  where subagentRunId = {run_id}:{task_id}:{plan_revision}:{attempt}
                                                       [subagentRunStore.ts:10-12]
  → STABLE per attempt. A retry bumps `attempt` → fresh key → fresh
    record. The OLD attempt's record is preserved untouched.
  task_id = stable PlanTask join key (used by latestSubagentRunsByTask
    to select one current attempt per task for rendering).

toolExecutionStore (DUAL identity — the root of A-SRF-03-P2-01)
  live ingest:   keyed by `tool.id`                     [toolExecutionStore.ts:211]
  merge/hydrate: keyed by `executionIdentity`           [toolExecutionStore.ts:47]
                   = `${toolExecutionOwnerKey}\u0000${call_id}`
  runtime-projected tool.id =
    `runtime-tool:${run_id}:${subagentRunId}:${callId}` [toolExecutionStore.ts:156]
  → The live path and the merge path disagree on identity. The live
    path overwrites by tool.id with no status-rank guard; the merge
    path dedupes by (owner, call_id) with status-rank. See V02.

taskRuntimeStore
  single active run by `run_id`                         [taskRuntimeStore.ts:79]
  incremental polling cursor = `lastSeq` (string)       [taskRuntimeStore.ts:87]
  → STABLE. Switching conversations bumps loadGeneration, resets
    lastSeq to '0', and clears events to prevent cross-stream mixing.
```

The subagent identity is the strongest: it encodes the attempt number
in the key itself, so retries are structurally isolated without any
guard logic. The tool-execution identity is the weakest (dual scheme,
live-overwrite). The task-runtime identity is solid (single active
run, generation-protected loads).

### Reducer monotonicity (V02)

| Reducer | Monotonicity strategy | Code anchor | Tested? |
|---|---|---|---|
| `subagentRunStore.ingest` | **Terminal lock**: `if (prev && prev.status !== 'running') return s;`. Late `started`, duplicate `usage`, duplicate terminal — all dropped. Retries use a new key (attempt bump), so they never touch the old terminal record. | `subagentRunStore.ts:458-460` | YES (`subagentRunStore.test.ts:65-90` duplicate-started-after-terminal; `:92-108` retry isolation) |
| `taskRuntimeStore.refresh` | **Generation counter** (`refreshRequestGeneration`): a stale `set` from an older request is suppressed when a newer request was issued. **`lastSeq` cursor**: incremental polling skips already-seen events. | `taskRuntimeStore.ts:165-217` | YES (`taskRuntimeStore.test.ts:255-267` stale-refresh-after-newer-terminal) |
| `taskRuntimeStore.loadByConversation` | **Generation counter** (`loadGeneration`): switching conversations mid-load suppresses the in-flight load's `set`. Resets `lastSeq='0'` + `events=[]` to prevent cross-run event mixing. | `taskRuntimeStore.ts:219-283` | YES (`taskRuntimeStore.test.ts:234-253` stale-conversation-load) |
| `taskRuntimeStore` polling overlap | **`refreshInFlight` flag** (module-level, not state): prevents a second `refresh` from starting while the previous one's `Promise.all` is in flight. | `taskRuntimeStore.ts:17,141,170,215` | implicit (covered by refresh tests) |
| `toolExecutionStore.ingest` (live) | **Direct overwrite by `tool.id`** — NO status-rank guard. A late `started` clobbers a terminal. | `toolExecutionStore.ts:206-217` | NO (the merge-path equivalent IS tested at `toolExecutionStore.test.ts:147-165`, but via `mergeHydratedToolExecutions`, not via live `ingest`) |
| `toolExecutionStore.hydrateConversation` (recovery) | **Status-rank + activity-timestamp merge** (`mergeToolExecution`): terminal beats running; newer activity wins at equal rank. | `toolExecutionStore.ts:58-86,106-117,223-235` | YES (`toolExecutionStore.test.ts:147-192`) |
| `ingestTaskRuntimeToolExecutions` (runtime boundary) | **`mergeTaskRuntimeBoundary`**: persisted terminal beats runtime boundary; runtime advances running→terminal only. | `toolExecutionStore.ts:88-104,241-254` | YES (`toolExecutionStore.test.ts:86-145`) |

### Old-attempt completion isolation (V02)

The A-FE-02 headline scenario — "attempt 1 fails, attempt 2 starts, a
late `completed` for attempt 1 arrives" — is handled correctly by
construction in the subagent store:

```text
attempt 1 key = `${runId}\u0000{run_id}:{task_id}:{rev}:1`  → status='failed' (terminal)
attempt 2 key = `${runId}\u0000{run_id}:{task_id}:{rev}:2`  → status='running' or 'completed'

Late `completed` for attempt 1:
  prev = runs[attempt1_key]   (status='failed')
  prev.status !== 'running'   → return s unchanged          [subagentRunStore.ts:458-460]
  attempt 2's record is NOT touched (different key).

latestSubagentRunsByTask picks attempt 2 for rendering:
  executionAttempt(run) parses the `:N` suffix from the key  [subagentRunStore.ts:407-414]
  newerAttempt = nextAttempt > currentAttempt               [subagentRunStore.ts:434-435]
  → attempt 2 (N=2) > attempt 1 (N=1) → attempt 2 wins.     [subagentRunStore.ts:417-441]
```

The isolation is structural (different keys), not guard-based. A late
terminal for an old attempt cannot corrupt the current attempt's
state or its selection as the rendered row.

### Lazy / collapsed large-output (V03)

`InlineToolCall` is the canonical lazy-output surface:

```text
Component mount (collapsed):
  renders only the summary line (name · args_preview · status · duration)
  detail_ref absent → "仅保留工具执行摘要" note                 [InlineToolCall.tsx:179-181]

User clicks expand:
  useEffect[expanded, manifest, tool] → loadPage(initial=true)  [InlineToolCall.tsx:86-89]
    toolExecutionApi.detail(detail_ref)       → manifest (args_full, output_bytes, truncated, failure, metadata)
    toolExecutionApi.readOutput(detail_ref, null) → first page {chunks, next_cursor, complete}
                                                [InlineToolCall.tsx:67-72]

While running (expanded):
  setInterval(loadPage(false), 500ms)                          [InlineToolCall.tsx:91-105]
  pauses when loadedCharacters >= LIVE_DETAIL_AUTOLOAD_CHARS (256 KiB)
                                                [InlineToolCall.tsx:23,92-99,137-138]
  → "实时加载已暂停，避免长日志占满页面内存。" notice   [InlineToolCall.tsx:244-248]

On terminal status change (expanded):
  one-shot loadPage(false) to fetch the final page             [InlineToolCall.tsx:107-117]

Manual "load more":
  button calls loadPage(false) with the current cursor         [InlineToolCall.tsx:249-258]

Container: max-h-96 overflow-auto                               [InlineToolCall.tsx:229]
manifest.truncated → "Agent 上下文已截断" label                [InlineToolCall.tsx:206]
```

Bounded in-memory event logs:

- `subagentRunStore`: `MAX_EVENTS_PER_RUN = 300` per run
  (`subagentRunStore.ts:145,486-488`).
- `taskRuntimeStore`: `MAX_EVENTS = 500` total
  (`taskRuntimeStore.ts:14,185,204,257`). Events are NOT rendered
  (only plan/todos/artifacts are), so the cap is a pure memory bound.

`SubagentResultView` renders `result.verification` / `artifacts` /
`remaining_work` as bounded lists (no pagination needed — the backend
caps these in `SubagentTaskResult::terminal`: summary ≤ 1200 chars,
remaining_work ≤ 64 × 500 chars per A-TSK-06).

`SubagentStreamBlock` renders ALL toolIds for a subagent in the
"process" tab, but each `InlineToolCall` is collapsed by default, so
only the summary lines render until the user expands one. No
virtualization for 100+ tool lists (minor).

### Result preservation (V01 + V03)

The terminal `SubagentTaskResult` is preserved end-to-end:

```text
Backend (A-TSK-06 verified):
  SubagentReleased{ result, full_output } → events.jsonl
  result.verification: Vec<SubagentVerificationResult>
    { check, status: passed|failed|not_run, source: observed|reported, details }

Frontend projection:
  taskRuntimeSubagentExecutionEvents (recovery path)          [subagentRunStore.ts:376-401]
    → projects SubagentReleased.payload.result.{summary,artifacts,verification,remaining_work,touched_files}
      into an ExecutionEvent
  ingest (live or projected) → terminalResult(ev, status)      [subagentRunStore.ts:205-226]
    → SubagentRunState.result: SubagentTaskResult
  SubagentStreamBlock "result" tab → SubagentResultView        [SubagentStreamBlock.tsx:148-155]
    → renders result.verification, result.artifacts, result.remaining_work
```

The projection is lossless for the fields it carries. The gap is in
what it does NOT carry — see V04.

### Acceptance / check distinction (V04)

The backend two-gate separation (A-TSK-06) is:

```text
Gate 1 (hard evidence):  assess_task_execution reads execution_checks + required_artifacts
Gate 2 (reviewer LLM):   review_task reads acceptance_criteria → ReviewResult{outcome, issues}
```

The frontend projection of this distinction is **partial**:

| Backend fact | Frontend projection | Preserved? |
|---|---|---|
| `PlanTask.execution_checks: Vec<String>` (declared hard-evidence commands) | Generated TS field exists; NEVER READ by any component. Written as `[]` for new tasks (TaskRuntimePanel.tsx:819). | NO — invisible |
| `PlanTask.acceptance_criteria: Vec<String>` (declared reviewer-judged prose) | Generated TS field exists; NEVER READ by any component. Written as `[]` for new tasks (TaskRuntimePanel.tsx:820). | NO — invisible |
| `SubagentVerificationResult.source: observed\|reported` (per-check evidence strength) | Rendered in SubagentResultView as `{status} · {source}` (e.g. "passed · observed") with NO semantic label. | PARTIAL — data present, semantics under-exposed |
| `ReviewResult` (reviewer LLM verdict: outcome, issues, severity) — fetched via `list_task_reviews` | `taskRuntimeApi.listReviews` defined (endpoints.ts:559-562) but **NEVER CALLED**. Backend command registered (task_runtime.rs:125). | NO — never fetched |
| Todo `blocked` status (reviewer said NeedsFix/Blocked) | `todoStatusDescription` produces "执行已完成 · 评审未通过" when trace=completed && todo=blocked (TaskRuntimePanel.tsx:496-497). Persisted status is authoritative (lines 450-465). | YES — binary outcome visible |
| `RecoveryBlocker` (workspace/tool state, NOT review) | Rendered with `reason` + retry/skip buttons (TaskRuntimePanel.tsx:630-674). | YES (but this is a DIFFERENT kind of block — workspace, not review) |

The binary outcome of the acceptance gate (did the task pass review?)
surfaces via the persisted todo status. But the gate's EVIDENCE — what
the reviewer found, which issues, what severity — is never fetched or
rendered. A user staring at a `blocked` task with "评审未通过" cannot
see WHY; the retry/skip decision is made blind.

## Findings

### A-FE-02-P2-01: Reviewer verdict (`ReviewResult`) is never fetched or rendered — the acceptance gate's evidence is invisible in the GUI

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/api/endpoints.ts:559-562` —
    `listReviews` is defined:
    ```ts
    listReviews: (runId: string, taskId: string) =>
      isTauri()
        ? apiInvoke<ReviewResult[]>('list_task_reviews', { runId, taskId })
        : get<ReviewResult[]>(`/task_runtime/runs/${runId}/tasks/${taskId}/reviews`),
    ```
  - `grep -rn "listReviews\|list_task_reviews" web-frontend/src`
    returns exactly ONE hit: the definition at endpoints.ts:559. ZERO
    callers anywhere in `src/`.
  - `echo-agent-cli/src/tauri/commands/task_runtime.rs:125` —
    `list_task_reviews` IS registered (mod.rs:140) and implemented,
    returning `Vec<ReviewResult>` from the store. The backend is
    ready; the frontend never invokes it.
  - `echo-agent-cli/src/tauri/mod.rs:140` — the command is in the
    Tauri invoke-handler list, so the IPC path works.
  - `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:496-497`
    — when a task is `blocked` due to review failure, the user sees:
    ```
    执行已完成 · 评审未通过
    ```
    (when trace.status === 'completed' && todo.status === 'blocked').
    No review issues, no severity, no reviewer reasoning is shown.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:2080-2130`
    — `list_recovery_blockers` folds ONLY `RecoveryBlocked` events
    (workspace/tool state). Review blocks (`ReviewNeedsFix` /
    `ReviewBlocked`) do NOT produce recovery blockers. So the
    recovery-blocker UI (TaskRuntimePanel.tsx:630-674) does NOT
    surface review issues either — it is for a different kind of
    block.
  - `echo-agent-cli/web-frontend/src/generated/ReviewResult.ts` —
    the generated type exists and is re-exported from
    `generated/index.ts`, so the contract is available. It is just
    never consumed.
- Reachability: every complex task that triggers the reviewer gate
  and gets `NeedsFix` or `Blocked`. Per A-TSK-06, the reviewer gate
  fires on every `Implementation` / `Debugging` task with non-empty
  `acceptance_criteria`. When the reviewer returns issues, the task
  goes `blocked`, the run suspends, and the user is expected to
  decide retry/skip. The `retryBlockedTask` button
  (TaskRuntimePanel.tsx:745-757) and the `resolveRecoveryTask`
  buttons (lines 656-673) are the user's only levers — and they act
  without seeing the review issues.
- Expected invariant: the frontend should project the same
  acceptance/check distinction the backend preserves (A-TSK-06 V02).
  A user supervising a complex task must be able to see WHY a task
  was blocked by review, not just THAT it was blocked.
- Observed behavior: the binary outcome (blocked) surfaces via the
  persisted todo status and `todoStatusDescription`. The review
  evidence (issues, severity, reviewer reasoning) is available on
  the backend (`list_task_reviews`) and the IPC path is wired, but
  no frontend component fetches or renders it. The `ReviewResult`
  generated type is dead weight in the frontend bundle.
- Impact: medium-high for the acceptance-gate UX. The EKO positioning
  (per AGENTS.md) is a "local personal super-intelligence assistant"
  that the user supervises. A supervised task system that says
  "review failed" without showing the review issues forces the user
  to retry blind or dig into `~/.eko/tasks/{run_id}/events.jsonl`
  manually. For implementation tasks (the majority of complex runs),
  this is the primary HITL surface — and it is missing its key piece
  of evidence. Severity is P2 (not P1) because the task is not lost
  (the backend preserves the review) and the user can still
  retry/skip; the gap is in decision support, not data integrity.
- Root cause: the `listReviews` endpoint was added (presumably when
  the reviewer gate was implemented, per A-TSK-06's review.rs) but
  the TaskRuntimePanel was never extended to call it. The
  `todoStatusDescription` label was the only projection added, and
  it stopped at the binary outcome.
- Direction: extend `TaskRuntimePanel` (or the todo detail view) to
  call `taskRuntimeApi.listReviews(runId, taskId)` when a todo is
  `blocked` (or always, lazily on expand) and render the `ReviewResult`
  list: outcome, issues (with category + severity), and the reviewer's
  summary. Pair with A-TSK-06's `ReviewResult` / `ReviewIssue` /
  `IssueSeverity` generated types (already in `generated/`). The
  recovery-blocker UI is the wrong place (different block kind); add
  a dedicated review-issues section to the todo row or a detail
  drawer.
- Regression validation: a vitest test that renders a `blocked` todo
  with a mocked `listReviews` response and asserts the review issues
  are rendered. Pair with the existing `taskRuntimeStore.test.ts`
  pattern (mock `taskRuntimeApi.listReviews`).
- Validation reports: [V04-01](../validations/A-FE-02/V04-01.md).

### A-FE-02-P3-01: `PlanTask.execution_checks` / `acceptance_criteria` are never displayed or editable for existing tasks

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/generated/PlanTask.ts` — the
    generated type includes `execution_checks: Array<string>` and
    `acceptance_criteria: Array<string>` as separate fields (per
    A-TSK-06 types.rs:1007-1011, with distinct doc comments
    separating hard-evidence commands from reviewer-judged prose).
  - `grep -rn "task\.execution_checks\|task\.acceptance_criteria\|\.execution_checks\b\|\.acceptance_criteria\b" web-frontend/src/components`
    returns ZERO hits in any `.tsx` component.
  - `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:819-820`
    — the ONLY write site: when inserting a NEW task, both fields are
    passed as empty arrays:
    ```tsx
    execution_checks: [],
    acceptance_criteria: [],
    ```
    No existing-task view reads them.
  - `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts:70-71`
    — `completeTaskPatch` includes both fields as `null`-able patch
    fields, so the update API supports editing them — but no UI
    surfaces an editor.
- Reachability: every complex-task run. The plan's tasks carry
  checks and criteria (populated by the plan-generation LLM or the
  domain profile), but the user cannot see them in the todo list. The
  todo row shows only: title, kind label, owner_agent, status
  description (TaskRuntimePanel.tsx:725-743).
- Expected invariant: the user supervising a task should be able to
  see what hard-evidence checks and what acceptance criteria were
  declared for it, so they understand what the executor and reviewer
  are evaluating against.
- Observed behavior: the fields are black-boxed. The user sees the
  task title and status but not the success criteria. When a task is
  blocked, the user cannot compare the review issues (P2-01) against
  the declared criteria because the criteria themselves are not
  shown.
- Impact: medium for transparency. Combined with P2-01, the entire
  acceptance-gate UX is opaque: the user can't see the criteria
  (this finding) or the review verdict against them (P2-01). For the
  hard-evidence gate, the `SubagentResultView.verification` items
  partially compensate (they show per-check results), but the
  DECLARED checks on the PlanTask are still invisible.
- Root cause: the todo list was designed as a compact progress view
  (title + status), and the task-detail surface was never built out.
  The fields exist in the type system and the patch API but have no
  rendering.
- Direction: add an expandable detail section to the todo row (or a
  detail drawer) that shows `task.execution_checks`,
  `task.acceptance_criteria`, and `task.required_artifacts` when
  present. Render them with distinct labels (e.g. "执行检查" /
  "验收标准" / "必需产物") matching the backend's distinct doc
  comments (types.rs:1003-1010). Optionally allow editing via the
  existing `updateTask` patch API.
- Regression validation: a vitest test that renders a todo whose
  plan task has non-empty `execution_checks` / `acceptance_criteria`
  and asserts both are rendered with distinct labels when expanded.
- Validation reports: [V04-01](../validations/A-FE-02/V04-01.md).

### A-FE-02-P3-02: `SubagentResultView` flattens verification `source` (observed/reported) into a technical label with no user-facing semantics

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/components/subagent/SubagentResultView.tsx:27-37`
    — the verification rendering:
    ```tsx
    {result?.verification.map((item) => (
      <div key={`${item.check}-${item.source}`}>
        <span className="font-medium text-[var(--text-primary)]">{item.check}</span>
        <span className="ml-1 text-[var(--text-tertiary)]">
          {item.status} · {item.source}
        </span>
        ...
      </div>
    ))}
    ```
    The `source` value (`'observed'` or `'reported'`) is rendered
    verbatim with no label, no tooltip, no legend.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:1520-1526,1537-1563`
    — `SubagentVerificationSource::Observed` means the framework
    actually observed the check (hard evidence: a command ran, exit
    code recorded). `Reported` means the subagent self-reported the
    result without hard evidence. This distinction maps directly to
    the execution_checks (observed+passed required) vs self-reported
    claims split that `assess_task_execution` enforces (A-TSK-06).
  - `echo-agent-cli/web-frontend/src/generated/SubagentVerificationSource.ts`
    — `export type SubagentVerificationSource = 'observed' | 'reported';`
- Reachability: every subagent result with non-empty `verification`.
  The user sees "passed · observed" or "failed · reported" and must
  know the framework's evidence-strength taxonomy to interpret it.
- Expected invariant: the evidence strength of a verification item
  should be visually distinguishable to a user who doesn't know the
  `observed`/`reported` jargon. A hard-evidence pass and a
  self-reported pass are not equally trustworthy.
- Observed behavior: both render as plain text in the same style.
  A user sees "cargo test --workspace · passed · observed" and "code
  follows style guide · passed · reported" with no visual hint that
  the first is hard evidence and the second is a self-claim.
- Impact: low-medium. The data is preserved and technically
  available; the gap is interpretive. A sophisticated user can read
  the raw values. A non-expert user may over-trust self-reported
  checks.
- Root cause: the verification rendering was written as a flat list
  without incorporating the evidence-strength semantics that the
  backend's two-gate design centers on.
- Direction: add a visual distinction for `source === 'observed'`
  (e.g. a small "实测" / hard-evidence badge or icon) vs
  `source === 'reported'` (e.g. a "自报" / soft-claim badge). Group
  observed checks separately from reported checks, mirroring the
  backend's execution_checks vs acceptance_criteria split. Add a
  tooltip explaining the distinction.
- Regression validation: a vitest test (renderToStaticMarkup) that
  feeds a verification list with both `observed` and `reported`
  items and asserts they are visually distinguished (e.g. different
  badge text/class).
- Validation reports: [V04-01](../validations/A-FE-02/V04-01.md).

### A-FE-02-P3-03: `toolExecutionStore.ingest` (live path) test gap — the live-overwrite regression (A-SRF-03-P2-01) is not covered by a driven-through-`ingest` test

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.ts:206-217`
    — the live `ingest` is a direct overwrite by `tool.id` with no
    status-rank guard (A-SRF-03-P2-01, confirmed from the
    projection-identity angle).
  - `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.test.ts:147-165`
    — the test "keeps a live terminal detail when a stale running
    snapshot arrives later" exercises `mergeHydratedToolExecutions`,
    NOT `useToolExecutionStore.getState().ingest`. It asserts the
    MERGE function is monotone, but the LIVE ingest path is never
    driven through this scenario.
  - `echo-agent-cli/web-frontend/src/components/chat/InlineToolCall.test.tsx`
    — the three tests ingest single tools and check the summary
    rendering; none ingests succeeded-then-running for the same id.
- Reachability: the live `ingest` is called from
  `useTauriChat.ts:134` on every `execution://event` with
  `kind: "tool"`. The defect (a late `started` clobbering a
  terminal) is reachable on hydrate-then-live interleaving or
  cross-thread emit races (per A-SRF-03-P2-01). The test gap means a
  future fix that routes `ingest` through `mergeToolExecution` would
  have no failing test to guide it, and a regression that reverts
  the fix would not be caught.
- Expected invariant: a test should exist that drives
  `useToolExecutionStore.getState().ingest` with `succeeded` then
  `running` for the same `tool.id` and asserts the store still holds
  `succeeded`.
- Observed behavior: no such test exists. The merge-path test
  (`toolExecutionStore.test.ts:147-165`) is the closest, but it
  tests a different entry point.
- Impact: the live-overwrite defect (A-SRF-03-P2-01) is currently
  unguarded by a test. When it is fixed, the fix needs a test; if
  the fix regresses later, nothing catches it.
- Root cause: the merge logic was added for the recovery path and
  tested there; the live path was not retrofitted with a parallel
  test.
- Direction: add a test to `toolExecutionStore.test.ts` that calls
  `useToolExecutionStore.getState().ingest(terminal)` then
  `useToolExecutionStore.getState().ingest(running)` (same id) and
  asserts `tools[id].status === 'succeeded'`. This test will FAIL
  today (documenting A-SRF-03-P2-01) and PASS once the live path is
  routed through `mergeToolExecution`.
- Regression validation: this finding IS the regression validation
  gap for A-SRF-03-P2-01 from the frontend-projection angle.
- Validation reports: [V02-01](../validations/A-FE-02/V02-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Reducer identity keys: stable, per-attempt, backend-anchored | yes | passed (with finding) | [V01-01](../validations/A-FE-02/V01-01.md) |
| V02 | Duplicate / out-of-order / old-attempt handling: terminal lock, generation counters, attempt isolation | yes | passed (with finding) | [V02-01](../validations/A-FE-02/V02-01.md) |
| V03 | Collapsed / expanded large-output: lazy pagination, 256 KiB cap, bounded event logs | yes | passed | [V03-01](../validations/A-FE-02/V03-01.md) |
| V04 | Acceptance / check separation in UI: review evidence, declared checks/criteria, verification source semantics | yes | failed | [V04-01](../validations/A-FE-02/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — three code/module comments treated as hypotheses; classifications inline) | passed | classified inline in Historical Claim Status |

Executed command (exit 0):

```text
cd echo-agent-cli/web-frontend
npx vitest run --reporter=dot
  Test Files  26 passed (26)
  Tests       101 passed (101)
```

No `cargo` command was required: this is a frontend-only review. The
backend cross-checks were read statically at A-TSK-06's verified
commits (no Rust code changed).

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `subagentRunStore.ts:10-12` — "Aggregation key is the concrete execution id in `subagent_run_id` (normally `{run_id}:{task_id}:{plan_revision}:{attempt}`). `task_id` remains the stable PlanTask join key. This separation keeps retries independent while still allowing task-oriented UI to select the latest attempt." | current | Verified by V01: `subagentRunStoreKey` (line 157-159) composes `${runId}\u0000${subagentRunId}`; `latestSubagentRunsByTask` (lines 417-441) selects by `task_id` key and picks the highest attempt. The retry-isolation test (`subagentRunStore.test.ts:92-108`) confirms two attempts get two keys. |
| `InlineToolCall.tsx:23` (`LIVE_DETAIL_AUTOLOAD_CHARS = 256 * 1024`) — the 256 KiB live-load cap | current | Verified by V03: the cap is enforced at lines 92-99 (pause condition) and surfaced at lines 137-138 / 244-248 (pause notice). |
| `TaskRuntimePanel.tsx:450-465` — "Persisted authoritative statuses must NOT be overwritten by trace signals. A task that the executor marked Blocked (acceptance pending) or Failed (terminal) stays that way even if a SubagentRun trace later reports completed/failed." | current | Verified by V04: `displayedTodoStatus` returns `todo.status` unchanged for all non-pending statuses. The trace signal only projects onto `pending` todos (lines 471-484). |
| A-FE-01-P2-01 (ToolInfo wire drift — `input_schema` vs `parameters`) | current (upstream context) | This task does not re-audit the tool-panel contract; A-FE-01 owns it. The tool-execution STORE contract (ToolExecution) is correct — the drift is in the tool-panel metadata, not in the execution projection. |
| A-SRF-03-P2-01 (tool-execution live ingest overwrite) | current (load-bearing, confirmed) | V01/V02 confirm from the projection-identity angle: the live `ingest` (toolExecutionStore.ts:206-217) is a direct overwrite by `tool.id` with no status-rank guard; the merge/hydrate path (lines 58-86, 106-117) IS monotone. The dual-identity scheme (`tool.id` vs `executionIdentity`) is the structural root. A-FE-02-P3-03 documents the test gap. |
| A-TSK-06 V02 (backend two-gate separation: execution_checks vs acceptance_criteria, distinct fields/gates/prompt sections) | current (load-bearing) | V04 confirms the backend separation is sound but ONLY PARTIALLY projected to the frontend. The binary outcome (blocked) surfaces via todo status; the review evidence (ReviewResult) is never fetched (A-FE-02-P2-01); the declared checks/criteria are never displayed (A-FE-02-P3-01); the verification source semantics are under-exposed (A-FE-02-P3-02). |

## Coverage And Uncertainty

Inspected in full: the three identity/reducer stores
(`subagentRunStore`, `toolExecutionStore`, `taskRuntimeStore`), the
four primary rendering components (`InlineToolCall`,
`SubagentStreamBlock`, `ParallelExecutionBlock`,
`SubagentResultView`), the `TaskRuntimePanel` head + status helpers +
todo list + recovery-blocker render + new-task insertion, the
`MessageBubble` head (flattenSteps, isExecutionProcessCompleted), the
`subagentResult` util, the relevant `generated/` types
(`SubagentVerificationResult/Source/Status`, `SubagentTaskResult`,
`RecoveryBlocker`, `ReviewResult`), the `taskRuntimeApi` /
`toolExecutionApi` endpoint definitions, and all 7 store tests + 3
rendering tests. Backend cross-checks: `list_recovery_blockers` fold
(store.rs:2080-2130), `RecoveryBlocker` + `SubagentVerificationSource`
schemas (types.rs:1385-1394, 1520-1563), `list_task_reviews` command
registration (mod.rs:140, task_runtime.rs:125). Full vitest suite
executed (26 files, 101 tests, exit 0).

Not inspected (out of scope or deferred):

- The `PlanEditor.tsx` component beyond confirming it does not
  reference `execution_checks` / `acceptance_criteria` (grep returned
  zero hits). If a future task adds check/criteria editing, this is
  where it would land.
- The TUI's projection of the same state (AGENTS.md multi-mode
  parity rule). The TUI likely has parallel rendering; this task
  audited only the GUI. The TUI may or may not surface review
  evidence — a separate audit is needed for TUI parity.
- The `endpoints.ts` `*Api` objects beyond `taskRuntimeApi` and
  `toolExecutionApi` (the highest-cardinality ones for this task).
  Lower-traffic APIs were not audited for projection gaps.
- E2E / Playwright coverage of the blocked-task UX (if any). Only
  vitest unit/integration tests were executed.

Environmental constraints:

- Read-only static review against `echo-agent-cli` commit `b3b2e81`.
  No code was modified. The vitest run used the existing incremental
  cache.

Uncertain claims:

- Whether ANY user has hit the "blocked without review evidence" UX
  gap in practice. The gap is structural (the code path is
  deterministic), but its user-visible frequency depends on how often
  the reviewer gate blocks tasks with non-trivial issues. No bug
  report was searched.
- Whether the TUI surfaces review evidence (parity question, not
  audited here). If the TUI does and the GUI does not, that is a
  multi-mode parity gap per AGENTS.md.

## Handoff

Conclusions downstream tasks may rely on:

1. **Subagent attempt identity is structurally isolated.** The
   per-attempt execution id in the store key means retries, late
   terminals, and out-of-order events cannot corrupt the current
   attempt. Downstream tasks auditing subagent lifecycle can trust
   this invariant (V01, V02).
2. **TaskRuntime load/refresh is generation-protected.** Stale
   conversation loads and stale refreshes are suppressed by the dual
   generation counters; polling overlap is prevented by
   `refreshInFlight`. Downstream tasks auditing recovery can rely on
   this (V02).
3. **Tool-execution live ingest is NOT monotone (A-SRF-03-P2-01
   confirmed).** The live `ingest` is a direct overwrite; only the
   hydrate/merge path is status-rank-guarded. Any task touching the
   tool-execution store must be aware of this dual policy. The test
   gap (A-FE-02-P3-03) means the fix needs a driven-through-`ingest`
   test.
4. **Lazy large-output handling is solid.** `InlineToolCall` is
   pull-based, cursor-paginated, 256 KiB-capped for live, with manual
   "load more" and terminal one-shot refetch. Event logs are bounded
   (300/run, 500 total). Downstream performance audits can rely on
   these bounds (V03).
5. **The acceptance/check distinction is only PARTIALLY projected.**
   The binary gate outcome (blocked) surfaces via todo status, but
   the review evidence (`ReviewResult`) is never fetched
   (A-FE-02-P2-01), the declared checks/criteria are never displayed
   (A-FE-02-P3-01), and the verification source semantics are
   under-exposed (A-FE-02-P3-02). The backend two-gate separation
   (A-TSK-06) is sound; the frontend drops the evidence layer.

Reports downstream tasks must read:

- This report (A-FE-02) for the identity model, the reducer
  monotonicity matrix, the lazy-output bounds, and the
  acceptance-gate projection gap.
- `tasks/A-SRF-03.md` for the tool-execution live-ingest overwrite
  (P2-01) — the same defect confirmed here from the identity angle.
- `tasks/A-FE-01.md` for the IPC type contract (especially the
  task-runtime family as the gold standard).
- `tasks/A-TSK-06.md` for the backend two-gate separation and the
  `ReviewResult` / `TaskExecutionSummary` schemas.

Conditions that make this report stale:

- Adding a caller to `taskRuntimeApi.listReviews` (resolving
  A-FE-02-P2-01) invalidates the "never fetched" claim.
- Routing `toolExecutionStore.ingest` through `mergeToolExecution`
  (resolving A-SRF-03-P2-01) invalidates the live-overwrite evidence
  in V02.
- Rendering `task.execution_checks` / `task.acceptance_criteria` in a
  component (resolving A-FE-02-P3-01) invalidates the
  "never displayed" claim.
- Adding visual distinction for verification `source` (resolving
  A-FE-02-P3-02) invalidates the "under-exposed semantics" claim.
- Adding a driven-through-`ingest` monotonicity test (resolving
  A-FE-02-P3-03) invalidates the test-gap claim.

Follow-up task IDs (no fixes implemented in this review):

- A **review-evidence surfacing** task — resolve A-FE-02-P2-01 by
  calling `taskRuntimeApi.listReviews` in the TaskRuntimePanel for
  blocked tasks and rendering the `ReviewResult` issues/severity.
  This is the primary UX gap; it pairs with A-TSK-06's backend
  (which already persists and exposes the reviews).
- A **task-detail expansion** task — resolve A-FE-02-P3-01 by
  showing `execution_checks` / `acceptance_criteria` /
  `required_artifacts` in an expandable todo detail section. Pair
  with the review-evidence task so the user sees criteria alongside
  the review verdict.
- A **verification-source semantics** task — resolve A-FE-02-P3-02
  by adding visual distinction for `observed` vs `reported`
  verification items in `SubagentResultView`.
- A **tool-execution live-ingest test** task — resolve A-FE-02-P3-03
  (and confirm A-SRF-03-P2-01's fix) by adding the
  ingest-succeeded-then-running test. This should land alongside the
  A-SRF-03-P2-01 fix.
